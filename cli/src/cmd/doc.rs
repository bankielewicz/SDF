//! `doc validate`, `doc load`, `doc accept requirements`, `doc reopen
//! requirements`.

use crate::cli::{DocAcceptArgs, DocReopenArgs, DocValidateArgs};
use crate::ctx::Ctx;
use crate::doc::{self, ids};
use crate::errors::CliError;
use crate::{docops, Outcome};
use std::path::PathBuf;

/// `devforgeai doc validate`
pub fn validate(
    ctx: &mut Ctx,
    args: &DocValidateArgs,
    stdin: Option<String>,
) -> Result<Outcome, CliError> {
    if let Some(prefix) = &args.allocate {
        if args.all || !args.paths.is_empty() {
            return Err(CliError::new(
                "DFA-E010",
                "'doc validate --allocate' takes no <path> and no --all",
            ));
        }
        let index = ids::build(&ctx.root);
        // The ID is reserved on disk before it is printed, so two worktrees
        // sharing one `.devforgeai/` cannot be handed the same number.
        let (id, reservation) = index.allocate_reserved(&ctx.root, prefix)?;
        let scanned = index.scanned();
        let mut out = Outcome::data(serde_json::json!({
            "prefix": prefix, "id": id, "scanned": scanned, "reservation": reservation
        }));
        // `--allocate` prints the ID alone, with a trailing newline and nothing
        // else.
        out.human = vec![id];
        out.project = ctx.root.display().to_string();
        return Ok(out);
    }

    let targets: Vec<PathBuf> = if args.all {
        doc::all_documents(&ctx.root)
    } else if args.paths.is_empty() {
        return Err(CliError::new(
            "DFA-E011",
            "'doc validate' requires <path>, --all or --allocate <prefix>",
        ));
    } else {
        args.paths
            .iter()
            .map(|p| {
                if p.is_absolute() {
                    p.clone()
                } else {
                    ctx.root.join(p)
                }
            })
            .collect()
    };

    if args.producer_check {
        return producer_check(ctx, &targets, args, stdin);
    }

    let index = ids::build(&ctx.root);
    let dctx = doc::Ctx {
        root: &ctx.root,
        index: &index,
        frontmatter_only: false,
        stdin_content: stdin.as_deref(),
    };

    let mut files = Vec::new();
    let mut human = Vec::new();
    let mut warnings = Vec::new();
    let mut failed = 0usize;
    let mut checked = 0usize;

    for path in &targets {
        let rel = ctx.rel(path);
        if doc::doc_type_for(&rel).is_none() {
            // A path under `.devforgeai/` matching no row is skipped with no
            // diagnostic, so a skill may keep scratch files there.
            continue;
        }
        let d = doc::validate(path, &dctx)?;
        checked += 1;
        if !d.valid() {
            failed += 1;
        }
        human.push(format!(
            "{}  {}  {}  {}",
            if d.valid() { "ok" } else { "fail" },
            d.path,
            d.doc_type,
            d.id
        ));
        warnings.extend(d.warnings.iter().cloned());
        files.push(serde_json::json!({
            "path": d.path,
            "doc_type": d.doc_type,
            "id": d.id,
            "status": d.status,
            "valid": d.valid(),
            "errors": d.errors.iter().map(diag_json).collect::<Vec<_>>(),
            "warnings": d.warnings.iter().map(diag_json).collect::<Vec<_>>(),
        }));
        for e in &d.errors {
            warnings.push(e.clone());
        }
    }

    let mut out = Outcome::data(serde_json::json!({
        "checked": checked, "failed": failed, "files": files
    }));
    out.human = human;
    out.warnings = warnings;
    out.project = ctx.root.display().to_string();
    out.exit = Some(if failed > 0 { 1 } else { 0 });
    Ok(out)
}

fn diag_json(d: &crate::Diag) -> serde_json::Value {
    serde_json::json!({ "code": d.code, "message": d.message, "line": d.line })
}

fn producer_check(
    ctx: &mut Ctx,
    targets: &[PathBuf],
    args: &DocValidateArgs,
    stdin: Option<String>,
) -> Result<Outcome, CliError> {
    let [path] = targets else {
        return Err(CliError::new(
            "DFA-E011",
            "'doc validate --producer-check' requires exactly one <path>",
        ));
    };
    let rel = ctx.rel(path);
    let Some(row) = doc::doc_type_for(&rel) else {
        // A path matching no row is not this phase's to refuse.
        let mut out = Outcome::data(serde_json::json!({
            "path": rel, "doc_type": null, "allowed": true
        }));
        out.project = ctx.root.display().to_string();
        return Ok(out);
    };

    let state = ctx.state()?;
    let current_phase = state.current.phase.clone();
    let current_id = state.current.id.clone();
    // A phase name outside the enum maps to no producer skill, so no row can
    // be matched against it. Refusing here names the bad state file; carrying
    // on would refuse every write with a message blaming the document.
    if doc::producer_of_phase_opt(&current_phase).is_none() {
        return Err(CliError::at(
            "DFA-E012",
            format!(
                "'{current_phase}' is not a phase; expected one of {}",
                crate::state::PHASES.join(", ")
            ),
            ".devforgeai/state.toml",
        ));
    }
    let allowed = doc::producer_allows(&current_phase, &row);

    let data = serde_json::json!({
        "path": rel,
        "doc_type": row.name,
        "expected_producer": row.producer,
        "current": { "phase": current_phase, "id": current_id },
        "allowed": allowed,
    });

    if !allowed {
        return Err(CliError::at(
            "DFA-E212",
            format!(
                "phase '{current_phase}' does not write {}; '{}' does",
                row.name, row.producer
            ),
            rel,
        ));
    }

    // The frontmatter and producer checks run; heading order and
    // cross-references are skipped.
    let index = ids::IdIndex::default();
    let dctx = doc::Ctx {
        root: &ctx.root,
        index: &index,
        frontmatter_only: true,
        stdin_content: stdin.as_deref(),
    };
    // With `--stdin-content` the file on disk is not read and its absence is
    // not an error in that mode.
    if args.stdin_content || path.exists() {
        let d = doc::validate(path, &dctx)?;
        if !d.valid() {
            let first = d.errors[0].clone();
            return Err(CliError::Coded(first));
        }
    }

    let mut out = Outcome::data(data);
    out.project = ctx.root.display().to_string();
    Ok(out)
}

/// `devforgeai doc load <name> <id>`
pub fn load(ctx: &mut Ctx, name: &str, id: &str) -> Result<Outcome, CliError> {
    let dot = ctx.dot();

    // `discover-entry` is the one name whose absence is not an error.
    if name == "discover-entry" {
        return discover_entry(ctx, id);
    }

    let paths: Vec<PathBuf> = match name {
        "explore-brief" => vec![dot.join("explore/brief.md")],
        "explore-decision" => vec![dot.join("explore/decision.yaml")],
        "requirements" => vec![dot.join("requirements.yaml")],
        "sprint" => vec![dot.join("stories/sprint.yaml")],
        "tokens" => vec![dot.join("brand/tokens.json")],
        "brand-kit" => vec![dot.join("brand/brand-kit.md")],
        "context" => {
            if id == "all" {
                doc::CONTEXT_STEMS
                    .iter()
                    .map(|s| dot.join(format!("context/{s}.md")))
                    .collect()
            } else {
                if !doc::CONTEXT_STEMS.contains(&id) {
                    return Err(CliError::new(
                        "DFA-E011",
                        format!(
                            "'context' takes one of {} or all",
                            doc::CONTEXT_STEMS.join(", ")
                        ),
                    ));
                }
                vec![dot.join(format!("context/{id}.md"))]
            }
        }
        "adr" => {
            if id == "all" {
                let mut found: Vec<(u32, PathBuf)> = Vec::new();
                if let Ok(entries) = std::fs::read_dir(dot.join("adr")) {
                    for e in entries.filter_map(Result::ok) {
                        let n = e.file_name().to_string_lossy().into_owned();
                        if let Some(stem) = n.strip_suffix(".md") {
                            if let Some((_, num)) = ids::split_id(stem) {
                                found.push((num, e.path()));
                            }
                        }
                    }
                }
                found.sort_by_key(|(n, _)| *n);
                found.into_iter().map(|(_, p)| p).collect()
            } else {
                vec![dot.join(format!("adr/{id}.md"))]
            }
        }
        "story" => vec![dot.join(format!("stories/{id}.md"))],
        "qa-report" => vec![dot.join(format!("reports/{id}-qa.yaml"))],
        "build-report" => vec![dot.join(format!("reports/{id}-build.yaml"))],
        "ui-spec" => vec![dot.join(format!("ui-specs/{id}.md"))],
        "release" => vec![dot.join(format!("releases/{id}.yaml"))],
        "reflect-report" => {
            if id == "latest" {
                let mut found: Vec<(String, PathBuf)> = Vec::new();
                if let Ok(entries) = std::fs::read_dir(dot.join("reports")) {
                    for e in entries.filter_map(Result::ok) {
                        let n = e.file_name().to_string_lossy().into_owned();
                        if let Some(rest) = n.strip_prefix("reflect-") {
                            if let Some(date) = rest.strip_suffix(".yaml") {
                                if doc::is_date(date) {
                                    found.push((date.to_string(), e.path()));
                                }
                            }
                        }
                    }
                }
                found.sort();
                match found.pop() {
                    Some((_, p)) => vec![p],
                    None => {
                        return Err(CliError::at(
                            "DFA-E200",
                            format!("{}/reports/reflect-*.yaml not found", ctx.rel(&dot)),
                            "reports/reflect-*.yaml",
                        ))
                    }
                }
            } else {
                vec![dot.join(format!("reports/reflect-{id}.yaml"))]
            }
        }
        other => {
            return Err(CliError::new(
                "DFA-E250",
                format!(
                    "'{other}' is not a document; the names are {}",
                    DOC_NAMES.join(", ")
                ),
            ))
        }
    };

    // The six context files concatenated are separated by a line of three
    // dashes; every other name prints its file byte for byte.
    let mut bytes: Vec<u8> = Vec::new();
    for (i, p) in paths.iter().enumerate() {
        let content = std::fs::read(p).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CliError::at("DFA-E200", format!("{} not found", ctx.rel(p)), ctx.rel(p))
            } else {
                CliError::io("reading", p.display(), &e)
            }
        })?;
        if i > 0 {
            if !bytes.ends_with(b"\n") {
                bytes.push(b'\n');
            }
            bytes.extend_from_slice(b"---\n");
        }
        bytes.extend_from_slice(&content);
    }

    let path_text = paths.first().map(|p| ctx.rel(p)).unwrap_or_default();
    let mut out = Outcome::data(serde_json::json!({
        "name": name,
        "id": id,
        "path": path_text,
        "bytes": bytes.len(),
        "content": String::from_utf8_lossy(&bytes),
    }));
    out.raw = Some(bytes);
    out.project = ctx.root.display().to_string();
    Ok(out)
}

/// The document names `doc load` accepts, for the `DFA-E250` message.
pub const DOC_NAMES: &[&str] = &[
    "explore-brief",
    "explore-decision",
    "requirements",
    "context",
    "adr",
    "story",
    "sprint",
    "ui-spec",
    "tokens",
    "brand-kit",
    "build-report",
    "qa-report",
    "release",
    "reflect-report",
    "discover-entry",
];

/// `doc load discover-entry "<arg>"`. Prints the brief and the decision when a
/// brief exists, the requirements when they exist and no brief does, both when
/// both exist, and nothing when the id matches no document. Exits 0 in every
/// one of those cases.
fn discover_entry(ctx: &mut Ctx, id: &str) -> Result<Outcome, CliError> {
    let dot = ctx.dot();
    let mut paths: Vec<PathBuf> = Vec::new();

    if ids::split_id(id).map(|(p, _)| p == "IDEA").unwrap_or(false) {
        let brief = dot.join("explore/brief.md");
        let decision = dot.join("explore/decision.yaml");
        let requirements = dot.join("requirements.yaml");

        let brief_matches = frontmatter_id_is(&brief, id);
        if brief_matches {
            paths.push(brief);
            if decision.is_file() {
                paths.push(decision);
            }
        }
        if frontmatter_id_is(&requirements, id) {
            paths.push(requirements);
        }
    }

    let mut bytes: Vec<u8> = Vec::new();
    for p in &paths {
        if let Ok(c) = std::fs::read(p) {
            bytes.extend_from_slice(&c);
            if !bytes.ends_with(b"\n") {
                bytes.push(b'\n');
            }
        }
    }

    let mut out = Outcome::data(serde_json::json!({
        "name": "discover-entry",
        "id": id,
        "path": paths.first().map(|p| ctx.rel(p)).unwrap_or_default(),
        "bytes": bytes.len(),
        "content": String::from_utf8_lossy(&bytes),
    }));
    out.raw = Some(bytes);
    out.project = ctx.root.display().to_string();
    Ok(out)
}

fn frontmatter_id_is(path: &std::path::Path, id: &str) -> bool {
    let Ok(text) = std::fs::read_to_string(path) else {
        return false;
    };
    let kind = doc::source_for(path);
    doc::frontmatter::parse(&text, kind, "")
        .map(|fm| fm.str_key("id") == Some(id))
        .unwrap_or(false)
}

/// `devforgeai doc accept requirements --id <IDEA-nnn>`
pub fn accept(ctx: &mut Ctx, args: &DocAcceptArgs) -> Result<Outcome, CliError> {
    if args.name != "requirements" {
        return Err(CliError::new(
            "DFA-E250",
            format!(
                "'{}' is not a document; the names are requirements",
                args.name
            ),
        ));
    }
    let accepted = docops::accept_requirements(ctx, &args.id)?;
    let mut out = Outcome::data(serde_json::json!({
        "id": args.id,
        "accepted_at": accepted.accepted_at,
        "requirements_moved": accepted.moved,
        "status": "accepted",
    }));
    out.human = vec![format!(
        "Accepted  {} · {} requirements · {} epics",
        args.id, accepted.requirements, accepted.epics
    )];
    out.project = ctx.root.display().to_string();
    Ok(out)
}

/// `devforgeai doc reopen requirements --id <IDEA-nnn>`
pub fn reopen(ctx: &mut Ctx, args: &DocReopenArgs) -> Result<Outcome, CliError> {
    if args.name != "requirements" {
        return Err(CliError::new(
            "DFA-E250",
            format!(
                "'{}' is not a document; the names are requirements",
                args.name
            ),
        ));
    }
    if !matches!(args.from.as_str(), "plan" | "constitute" | "design") {
        return Err(CliError::new(
            "DFA-E012",
            format!(
                "'{}' is not a phase; expected one of plan, constitute, design",
                args.from
            ),
        ));
    }
    let cited: Vec<String> = args
        .ids
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();

    let done = docops::reopen_requirements(ctx, &args.id, &cited, &args.from)?;
    let mut out = Outcome::data(serde_json::json!({
        "id": args.id,
        "revision": done.revision,
        "reopened": done.reopened,
        "allocated": done.allocated,
        "from": args.from,
        "status": "reopened",
    }));
    out.human = vec![format!(
        "Reopened  {} · revision {} · {} reopened · {} allocated",
        args.id,
        done.revision,
        done.reopened.join(", "),
        done.allocated.join(", ")
    )];
    out.project = ctx.root.display().to_string();
    Ok(out)
}
