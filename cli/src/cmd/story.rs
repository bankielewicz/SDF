//! `story validate`, `story list`, and `story files`.

use crate::cli::StoryFilesArgs;
use crate::ctx::Ctx;
use crate::errors::{CliError, Diag};
use crate::story::{self, Story};
use crate::{git, Outcome};
use std::collections::{BTreeMap, BTreeSet};

/// The `--scope` enum.
pub const SCOPES: &[&str] = &["active", "sprint", "all"];

/// The `status` enum of a story, in progression order.
pub const STATUSES: &[&str] = &["draft", "ready", "building", "built", "released"];

// ------------------------------------------------------------ story validate

/// One story's verdict.
struct Verdict {
    story: Story,
    errors: Vec<Diag>,
}

/// Run the ten checks over the scope.
pub fn validate(ctx: &mut Ctx, id: Option<&str>, scope: &str) -> Result<Outcome, CliError> {
    let root = ctx.root.clone();
    let project = root.display().to_string();

    // The scope is validated whether or not an id narrows the run. Checking it
    // only on the `None` arm let `--scope bogus --id STORY-001` through: the
    // typo was accepted in silence, and the caller believed a scope had been
    // honoured that the command had never read.
    if !SCOPES.contains(&scope) {
        return Err(CliError::new(
            "DFA-E012",
            format!(
                "'{scope}' is not a scope; expected one of {}",
                SCOPES.join(", ")
            ),
        ));
    }
    if let Some(want) = id {
        if !crate::doc::ids::is_id(want) || !want.starts_with("STORY-") {
            return Err(CliError::new(
                "DFA-E013",
                format!("'{want}' is not an ID; expected PREFIX-nnn with three digits"),
            ));
        }
    }

    let effective = if id.is_some() { "id" } else { scope };
    let mut warnings: Vec<Diag> = Vec::new();

    // The subject set.
    let sprint = story::load_sprint(&root)?;
    let ids: Vec<String> = match (id, effective) {
        (Some(want), _) => vec![want.to_string()],
        (None, "sprint") => sprint.as_ref().map(|s| s.all_ids()).unwrap_or_default(),
        (None, "all") => story::all_paths(&root)
            .iter()
            .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(str::to_string))
            .collect(),
        _ => {
            let active = ctx.state()?.active.build.clone();
            if active.is_empty() {
                // Scope `active` with no active story validates nothing, and
                // exiting 0 made that read as "every story is sound". The
                // subject is a required argument the caller did not supply,
                // which is what `DFA-E011` names and why its exit is the usage
                // class. The gate's `story_valid` check reads this list rather
                // than the exit code, so it still fails the check the same way.
                warnings.push(Diag::new(
                    "DFA-E011",
                    "'story validate' requires --id, or an active build story in state.toml; nothing validated",
                ));
                Vec::new()
            } else {
                vec![active]
            }
        }
    };

    // Check 6: every sprint id has a file.
    let mut missing: Vec<Diag> = Vec::new();
    let mut present: Vec<String> = Vec::new();
    for want in &ids {
        if story::path_of(&root, want).exists() {
            present.push(want.clone());
        } else if effective == "sprint" {
            missing.push(Diag::at(
                "DFA-E233",
                format!("sprint.yaml lists {want}, which has no file"),
                ".devforgeai/stories/sprint.yaml",
            ));
        } else {
            return Err(CliError::at(
                "DFA-E200",
                format!(".devforgeai/stories/{want}.md not found"),
                format!(".devforgeai/stories/{want}.md"),
            ));
        }
    }

    let index = crate::doc::ids::build(&root);
    let reqs = requirement_ids(&root);
    let mut verdicts: Vec<Verdict> = Vec::new();

    for want in &present {
        let s = story::load(&root, want)?;
        let mut errors: Vec<Diag> = Vec::new();

        // 1. The frontmatter grammar.
        let path = story::path_of(&root, want);
        let dctx = crate::doc::Ctx {
            root: &root,
            index: &index,
            frontmatter_only: true,
            stdin_content: None,
        };
        match crate::doc::validate(&path, &dctx) {
            Ok(doc) => errors.extend(doc.errors.iter().cloned()),
            Err(e) => {
                if let Some(d) = e.diag() {
                    errors.push(d.clone());
                }
            }
        }

        // 2. At least one AC.
        if s.acs.is_empty() {
            errors.push(Diag::at(
                "DFA-E234",
                format!("{want} defines no acceptance criteria"),
                s.rel.clone(),
            ));
        }

        // 3. Each AC is testable.
        for ac in &s.acs {
            if !story::testable(&ac.text) {
                errors.push(Diag::at_line(
                    "DFA-E230",
                    format!(
                        "{want} {} has no testable predicate; the grammar is Given/When/Then or a single assertion line",
                        ac.id
                    ),
                    s.rel.clone(),
                    ac.line,
                ));
            }
        }

        // 4. Each `REQ-nnn` in `consumes` exists.
        for c in s.consumes.iter().filter(|c| c.starts_with("REQ-")) {
            if !reqs.contains(c) {
                errors.push(Diag::at(
                    "DFA-E231",
                    format!("{want} consumes {c}, which requirements.yaml does not define"),
                    s.rel.clone(),
                ));
            }
        }

        // 8. Each AC appears in exactly one `Covered by` cell.
        for ac in &s.acs {
            let owners: Vec<&str> = s
                .requirements
                .iter()
                .filter(|r| r.covered_by.iter().any(|c| c == &ac.id))
                .map(|r| r.id.as_str())
                .collect();
            match owners.len() {
                0 => errors.push(Diag::at_line(
                    "DFA-E236",
                    format!(
                        "{want} {} appears in no Covered by cell of ## Requirements",
                        ac.id
                    ),
                    s.rel.clone(),
                    ac.line,
                )),
                1 => {}
                _ => errors.push(Diag::at_line(
                    "DFA-E245",
                    format!(
                        "{want} {} appears in the Covered by cell of {} and {}",
                        ac.id, owners[0], owners[1]
                    ),
                    s.rel.clone(),
                    ac.line,
                )),
            }
        }

        // 10. Each `UI-nnn` in `consumes` resolves.
        for c in s.consumes.iter().filter(|c| c.starts_with("UI-")) {
            let spec = root
                .join(".devforgeai")
                .join("ui-specs")
                .join(format!("{c}.md"));
            if !spec.exists() {
                errors.push(Diag::at(
                    "DFA-E238",
                    format!("{want} cites {c}, which .devforgeai/ui-specs/ does not hold"),
                    s.rel.clone(),
                ));
            }
        }

        verdicts.push(Verdict { story: s, errors });
    }

    // 5. No cycle over the dependency graph.
    let graph: BTreeMap<String, Vec<String>> = verdicts
        .iter()
        .map(|v| (v.story.id.clone(), v.story.deps.clone()))
        .collect();
    let mut cycle_errors: Vec<Diag> = Vec::new();
    for path in cycles(&graph) {
        cycle_errors.push(Diag::new(
            "DFA-E232",
            format!("dependency cycle: {}", path.join(" -> ")),
        ));
    }

    // 7 and 9, sprint scope only.
    let mut sprint_errors: Vec<Diag> = missing;
    if effective == "sprint" {
        if let Some(sp) = &sprint {
            sprint_errors.extend(epic_cover(&root, sp, &verdicts));
            sprint_errors.extend(no_shared_path(sp, &verdicts));
        }
    }

    let checked = verdicts.len();
    let failed = verdicts.iter().filter(|v| !v.errors.is_empty()).count();
    let extra = cycle_errors.len() + sprint_errors.len();

    let mut human: Vec<String> = Vec::new();
    for v in &verdicts {
        let n = v.story.deps.len();
        human.push(format!(
            "{}  {}  {} ACs  {n} {}",
            if v.errors.is_empty() { "ok  " } else { "fail" },
            v.story.id,
            v.story.acs.len(),
            if n == 1 { "dependency" } else { "dependencies" }
        ));
    }

    let data = serde_json::json!({
        "scope": effective,
        "stories": verdicts.iter().map(|v| serde_json::json!({
            "id": v.story.id,
            "valid": v.errors.is_empty(),
            "acs": v.story.acs.len(),
            "deps": v.story.deps,
            "errors": v.errors.iter().map(diag_json).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
        "checked": checked,
        "failed": failed,
    });

    let mut diags: Vec<Diag> = Vec::new();
    for v in &verdicts {
        diags.extend(v.errors.iter().cloned());
    }
    diags.extend(cycle_errors);
    diags.extend(sprint_errors);
    let bad = failed > 0 || extra > 0;
    warnings.extend(diags);
    // A run that validated nothing is not a run that found nothing wrong.
    // The exit follows the diagnostic's own row, so the pair a caller reads
    // out of the error table is the pair the process gives.
    let nothing_validated = warnings.iter().any(|d| d.code == "DFA-E011");

    Ok(Outcome {
        human,
        data,
        warnings,
        degraded: ctx.degraded(),
        project,
        exit: Some(if nothing_validated {
            3
        } else if bad {
            1
        } else {
            0
        }),
        raw: None,
        stderr: Vec::new(),
        ..Default::default()
    })
}

fn diag_json(d: &Diag) -> serde_json::Value {
    serde_json::json!({
        "code": d.code,
        "message": d.message,
        "path": d.path,
        "line": d.line,
    })
}

/// Every `REQ-nnn` `requirements.yaml` defines.
fn requirement_ids(root: &std::path::Path) -> BTreeSet<String> {
    let path = root.join(".devforgeai").join("requirements.yaml");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return BTreeSet::new();
    };
    let Ok(v) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&text) else {
        return BTreeSet::new();
    };
    v.get("requirements")
        .and_then(|x| x.as_sequence())
        .map(|s| {
            s.iter()
                .filter_map(|e| e.get("id").and_then(|x| x.as_str()).map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// The `requirements[]` of one epic.
fn epic_requirements(root: &std::path::Path, epic: &str) -> Vec<String> {
    let path = root.join(".devforgeai").join("requirements.yaml");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(v) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&text) else {
        return Vec::new();
    };
    v.get("epics")
        .and_then(|x| x.as_sequence())
        .and_then(|s| {
            s.iter()
                .find(|e| e.get("id").and_then(|x| x.as_str()) == Some(epic))
        })
        .and_then(|e| e.get("requirements"))
        .and_then(|x| x.as_sequence())
        .map(|s| {
            s.iter()
                .filter_map(|r| r.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Check 7: every requirement of the sprint's epic is covered by some story.
fn epic_cover(root: &std::path::Path, sprint: &story::Sprint, verdicts: &[Verdict]) -> Vec<Diag> {
    if sprint.epic.is_empty() {
        return Vec::new();
    }
    let covered: BTreeSet<&str> = verdicts
        .iter()
        .flat_map(|v| v.story.requirements.iter().map(|r| r.id.as_str()))
        .collect();
    epic_requirements(root, &sprint.epic)
        .into_iter()
        .filter(|r| !covered.contains(r.as_str()))
        .map(|r| {
            Diag::at(
                "DFA-E235",
                format!(
                    "{} requirement {r} appears in the Requirements table of no story",
                    sprint.epic
                ),
                ".devforgeai/stories/sprint.yaml",
            )
        })
        .collect()
}

/// Check 9: no `Path` appears in the `## Files` table of two concurrent stories.
fn no_shared_path(sprint: &story::Sprint, verdicts: &[Verdict]) -> Vec<Diag> {
    let concurrent: BTreeSet<&str> = sprint.stories.iter().map(|e| e.id.as_str()).collect();
    let mut owner: BTreeMap<&str, &str> = BTreeMap::new();
    let mut out = Vec::new();
    for v in verdicts
        .iter()
        .filter(|v| concurrent.contains(v.story.id.as_str()))
    {
        for f in &v.story.files {
            match owner.get(f.path.as_str()) {
                Some(first) => out.push(Diag::at_line(
                    "DFA-E237",
                    format!("{first} and {} both declare {}", v.story.id, f.path),
                    v.story.rel.clone(),
                    f.line,
                )),
                None => {
                    owner.insert(&f.path, &v.story.id);
                }
            }
        }
    }
    out
}

/// Every cycle of the dependency graph, each rendered as `A -> B -> A`.
fn cycles(graph: &BTreeMap<String, Vec<String>>) -> Vec<Vec<String>> {
    let mut out: Vec<Vec<String>> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();

    for start in graph.keys() {
        if seen.contains(start) {
            continue;
        }
        let mut stack: Vec<String> = Vec::new();
        let mut on_stack: BTreeSet<String> = BTreeSet::new();
        walk(graph, start, &mut stack, &mut on_stack, &mut seen, &mut out);
    }
    out
}

fn walk(
    graph: &BTreeMap<String, Vec<String>>,
    at: &str,
    stack: &mut Vec<String>,
    on_stack: &mut BTreeSet<String>,
    seen: &mut BTreeSet<String>,
    out: &mut Vec<Vec<String>>,
) {
    if let Some(at_index) = stack.iter().position(|s| s == at) {
        let mut path: Vec<String> = stack[at_index..].to_vec();
        path.push(at.to_string());
        for node in &path {
            seen.insert(node.clone());
        }
        if !out.contains(&path) {
            out.push(path);
        }
        return;
    }
    if seen.contains(at) && !on_stack.contains(at) {
        return;
    }
    stack.push(at.to_string());
    on_stack.insert(at.to_string());
    for next in graph.get(at).map(Vec::as_slice).unwrap_or(&[]) {
        walk(graph, next, stack, on_stack, seen, out);
    }
    stack.pop();
    on_stack.remove(at);
    seen.insert(at.to_string());
}

// ---------------------------------------------------------------- story list

/// One line per story under `.devforgeai/stories/`.
pub fn list(
    ctx: &mut Ctx,
    status: Option<&str>,
    sprint: Option<&str>,
) -> Result<Outcome, CliError> {
    let root = ctx.root.clone();
    let dir = root.join(".devforgeai").join("stories");
    if !dir.exists() {
        return Err(CliError::at(
            "DFA-E231",
            ".devforgeai/stories/ not found; run 'devforgeai init'",
            ".devforgeai/stories",
        ));
    }

    let wanted: Option<Vec<String>> = match status {
        Some(list) => {
            let values: Vec<String> = list.split(',').map(|s| s.trim().to_string()).collect();
            for v in &values {
                if !STATUSES.contains(&v.as_str()) {
                    return Err(CliError::new(
                        "DFA-E012",
                        format!(
                            "'{v}' is not a story status; expected one of {}",
                            STATUSES.join(", ")
                        ),
                    ));
                }
            }
            Some(values)
        }
        None => None,
    };

    let sprint_ids: Option<Vec<String>> = match sprint {
        Some(want) => {
            let sp = story::load_sprint(&root)?;
            let ids = sp
                .filter(|s| s.id == want || want.is_empty())
                .map(|s| s.stories.iter().map(|e| e.id.clone()).collect())
                .unwrap_or_default();
            Some(ids)
        }
        None => None,
    };

    let mut rows: Vec<Story> = Vec::new();
    for path in story::all_paths(&root) {
        let s = story::read(&root, &path)?;
        if let Some(w) = &wanted {
            if !w.contains(&s.status) {
                continue;
            }
        }
        if let Some(ids) = &sprint_ids {
            if !ids.contains(&s.id) {
                continue;
            }
        }
        rows.push(s);
    }

    let human = rows
        .iter()
        .map(|s| format!("{}  {}  {}", s.id, s.status, s.title))
        .collect();

    let data = serde_json::json!({
        "count": rows.len(),
        "stories": rows.iter().map(|s| serde_json::json!({
            "id": s.id,
            "status": s.status,
            "title": s.title,
            "path": s.rel,
            "consumes": s.consumes,
        })).collect::<Vec<_>>(),
    });

    Ok(Outcome {
        human,
        data,
        degraded: ctx.degraded(),
        project: root.display().to_string(),
        ..Default::default()
    })
}

// --------------------------------------------------------------- story files

/// `--check`, `--list`, or `--diff` over a story's declared file set.
pub fn files(ctx: &mut Ctx, a: &StoryFilesArgs) -> Result<Outcome, CliError> {
    let modes = [a.check.is_some(), a.list, a.diff]
        .iter()
        .filter(|x| **x)
        .count();
    if modes != 1 {
        return Err(CliError::new(
            "DFA-E011",
            "'story files' requires exactly one of --check, --list, --diff",
        ));
    }

    let root = ctx.root.clone();
    let project = root.display().to_string();

    if let Some(want) = &a.id {
        if !crate::doc::ids::is_id(want) || !want.starts_with("STORY-") {
            return Err(CliError::new(
                "DFA-E013",
                format!("'{want}' is not an ID; expected PREFIX-nnn with three digits"),
            ));
        }
    }

    let id = match &a.id {
        Some(want) => want.clone(),
        None => ctx.state()?.active.build.clone(),
    };
    if id.is_empty() {
        // Outside a Build run the PreToolUse hook must not block.
        return Ok(Outcome {
            human: vec!["no active build story; nothing checked".to_string()],
            data: serde_json::json!({ "id": "", "files": [], "undeclared": 0 }),
            warnings: vec![Diag::new(
                "DFA-E412",
                "state.toml has no active id for phase 'build'; nothing checked",
            )],
            degraded: ctx.degraded(),
            project,
            exit: Some(0),
            raw: None,
            stderr: Vec::new(),
            ..Default::default()
        });
    }

    let s = story::load(&root, &id)?;
    let declared: Vec<String> = s.files.iter().map(|f| f.path.clone()).collect();

    if let Some(path) = &a.check {
        let rel = repo_relative(&root, path);
        let allowed = declared.contains(&rel);
        let mut warnings = Vec::new();
        if !allowed {
            warnings.push(Diag::new(
                "DFA-E239",
                format!("{rel} is outside the declared file set of {id}"),
            ));
        }
        return Ok(Outcome {
            human: vec![format!(
                "{}  {rel}",
                if allowed { "allowed  " } else { "undeclared" }
            )],
            data: serde_json::json!({
                "id": id,
                "path": rel,
                "allowed": allowed,
                "declared": declared.len(),
            }),
            warnings,
            degraded: ctx.degraded(),
            project,
            exit: Some(if allowed { 0 } else { 1 }),
            raw: None,
            stderr: Vec::new(),
            ..Default::default()
        });
    }

    if a.list {
        return Ok(Outcome {
            human: s
                .files
                .iter()
                .map(|f| format!("{}  {}  {}", f.path, f.kind, f.layer))
                .collect(),
            data: serde_json::json!({
                "id": id,
                "files": s.files.iter().map(|f| serde_json::json!({
                    "path": f.path, "kind": f.kind, "layer": f.layer,
                })).collect::<Vec<_>>(),
            }),
            degraded: ctx.degraded(),
            project,
            ..Default::default()
        });
    }

    // --diff
    diff(ctx, &s, a.base.as_deref())
}

/// Test every changed path against the declared set.
fn diff(ctx: &mut Ctx, s: &Story, base: Option<&str>) -> Result<Outcome, CliError> {
    // The diff is of the story's own work, and during a worktree build that
    // work is committed on the worktree's branch. Running git in the main
    // checkout compares the wrong branch and reports no change at all, so
    // `files_declared` would measure nothing and pass every story.
    let source = ctx.source_root();
    git::require_work_tree(&source)?;
    let base_ref = ctx.config()?.build.base_ref.clone();
    let base = match base {
        Some(b) => b.to_string(),
        None => git::merge_base(&source, &base_ref)?,
    };

    // A path under `.devforgeai/` is a framework artifact, not story work: the
    // PreToolUse map routes those to `doc validate` rather than to the
    // declared-set rule, and `gate check` writes one on every run.
    let changes: Vec<crate::git::Change> = git::changed(&source, &base)?
        .into_iter()
        .filter(|c| !c.path.starts_with(".devforgeai/"))
        .collect();
    let declared: Vec<&str> = s.files.iter().map(|f| f.path.as_str()).collect();

    let mut warnings = Vec::new();
    let mut rows = Vec::new();
    let mut undeclared = 0usize;
    for c in &changes {
        let ok = declared.contains(&c.path.as_str());
        if !ok {
            undeclared += 1;
            warnings.push(Diag::new(
                "DFA-E239",
                format!("{} is outside the declared file set of {}", c.path, s.id),
            ));
        }
        let row = s.files.iter().find(|f| f.path == c.path);
        rows.push(serde_json::json!({
            "path": c.path,
            "declared": ok,
            "kind": row.map(|f| f.kind.clone()).unwrap_or_default(),
            "status": c.status,
        }));
    }

    Ok(Outcome {
        human: vec![format!(
            "{} changed · {undeclared} outside the declared set of {}",
            changes.len(),
            s.id
        )],
        data: serde_json::json!({
            "id": s.id,
            "base": base,
            "paths": rows,
            "undeclared": undeclared,
        }),
        warnings,
        degraded: ctx.degraded(),
        project: ctx.root.display().to_string(),
        exit: Some(if undeclared == 0 { 0 } else { 1 }),
        raw: None,
        stderr: Vec::new(),
        ..Default::default()
    })
}

/// A path made repo-relative in forward-slash form.
pub fn repo_relative(root: &std::path::Path, path: &std::path::Path) -> String {
    let normalised = crate::project::normalise(path);
    let stripped = normalised
        .strip_prefix(crate::project::normalise(root).as_path())
        .unwrap_or(&normalised);
    story::normalise_path(&stripped.to_string_lossy())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph(pairs: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
        pairs
            .iter()
            .map(|(k, v)| {
                (
                    (*k).to_string(),
                    v.iter().map(|s| (*s).to_string()).collect(),
                )
            })
            .collect()
    }

    #[test]
    fn an_acyclic_graph_has_no_cycle() {
        let g = graph(&[("A", &["B"]), ("B", &["C"]), ("C", &[])]);
        assert!(cycles(&g).is_empty());
    }

    #[test]
    fn a_two_node_cycle_is_reported_as_a_path() {
        let g = graph(&[("A", &["B"]), ("B", &["A"])]);
        let c = cycles(&g);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0], vec!["A", "B", "A"]);
    }

    #[test]
    fn a_self_edge_is_a_cycle() {
        let g = graph(&[("A", &["A"])]);
        assert_eq!(cycles(&g), vec![vec!["A".to_string(), "A".to_string()]]);
    }

    #[test]
    fn a_relative_path_stays_relative() {
        let root = std::path::Path::new("/tmp/project");
        assert_eq!(
            repo_relative(root, std::path::Path::new("src/a.rs")),
            "src/a.rs"
        );
    }
}
