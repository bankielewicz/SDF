//! `init`: create `.devforgeai/`, copy the framework directories, write the
//! three project files, run `stack detect`, append to `.gitignore`, and install
//! the hooks.

use crate::cli::InitArgs;
use crate::ctx::Ctx;
use crate::errors::{CliError, Diag};
use crate::{gates, hooks, project, state, trust, Outcome};
use std::path::{Path, PathBuf};

/// The directories copied from the framework root. `commands/` is gone: a
/// skill and a command file of the same name both produce `/name` and the skill
/// wins, so each entry point carries its own frontmatter and preamble and there
/// is no shadow file to edit by mistake.
const COPIED: &[&str] = &["skills", "agents"];

/// A directory name excluded from every copied tree. The eval suites are the
/// framework's own tests of its skills; they are megabytes of fixtures and
/// transcripts that a target project never runs.
const EXCLUDED_SUBTREES: &[&str] = &["evals"];

/// The two lines `init` appends to the target project's `.gitignore`.
const IGNORE_LINES: &[&str] = &[".explore-prototype/", ".devforgeai/state.toml"];

/// The markers the CLAUDE.md section sits between, so a re-run replaces the
/// span and leaves every line the user wrote outside it untouched.
const CLAUDE_MD_BEGIN: &str = "<!-- devforgeai:begin -->";
const CLAUDE_MD_END: &str = "<!-- devforgeai:end -->";

/// The section `init` writes into the target project's CLAUDE.md.
///
/// Nothing here restates a hook's rule, names a language or a test runner, or
/// tells the model to check itself: the hooks do the checking and this says
/// that they do. The nine commands are one line each, and the handoff's origin
/// and the copy-the-`Next`-line rule are stated once.
const CLAUDE_MD_BLOCK: &str = r#"<!-- devforgeai:begin -->
## DevForgeAI

Delivery runs as phases. Each phase is one skill with one slash command, writes one
document under `.devforgeai/`, and ends with a handoff block naming the next command.

| Command | Phase | Writes |
|---|---|---|
| `/explore "<idea>"` | 0 Explore | `explore/brief.md`, `explore/decision.yaml` |
| `/discover <IDEA-nnn>` | 1 Discover | `requirements.yaml` |
| `/constitute <IDEA-nnn>` | 2 Constitute | `context/*.md`, `adr/ADR-nnn.md` |
| `/plan <EPIC-nnn>` | 3 Plan | `stories/STORY-nnn.md`, `stories/sprint.yaml` |
| `/build <STORY-nnn>` | 4 Build | source and tests, `reports/STORY-nnn-build.yaml` |
| `/verify <STORY-nnn>` | 5 Verify | `reports/STORY-nnn-qa.yaml` |
| `/release <vX.Y.Z>` | 6 Release | `releases/vX.Y.Z.yaml` |
| `/design <id>` | any | `brand/tokens.json`, `ui-specs/UI-nnn.md` |
| `/reflect <window>` | any | `reports/reflect-<date>.yaml` |

The handoff comes from the Stop hook at the end of each phase. Its `Next` line is the
command to type next, copied as printed, arguments included.

Phase documents live under `.devforgeai/`. Source and tests live where
`.devforgeai/config.toml` records the project's roots.

Gates are the `devforgeai` binary and the Claude Code hooks that call it, not
instructions in this file: a write to `.devforgeai/` is checked as it happens, a phase
ends when its gate passes, and a failing gate holds the turn open.

A defect found in an upstream document is cited, not edited. The handoff prints the
send-back command that reopens the cited ids.
<!-- devforgeai:end -->"#;

/// `devforgeai init`
pub fn run(project_flag: Option<&Path>, args: &InitArgs) -> Result<Outcome, CliError> {
    // `--project` defaults to the working directory for `init`.
    let root = match project_flag {
        Some(p) => {
            if !p.exists() {
                return Err(CliError::new(
                    "DFA-E031",
                    format!("--project path '{}' does not exist", p.display()),
                ));
            }
            project::normalise(p)
        }
        None => std::env::current_dir()
            .map_err(|e| CliError::io("reading", "the working directory", &e))?,
    };

    let dot = project::dot(&root);
    if dot.join(project::CONFIG).exists() && !args.force {
        return Err(CliError::at(
            "DFA-E110",
            "project is already initialised; pass --force to overwrite .devforgeai/",
            ".devforgeai/config.toml",
        ));
    }

    let from = resolve_framework(args.from.as_deref())?;

    let mut created: Vec<String> = Vec::new();
    let mut warnings: Vec<Diag> = Vec::new();

    // Create `.devforgeai/` and its subdirectories.
    mkdir(&dot, &root, &mut created)?;
    for sub in project::SUBDIRS {
        mkdir(&dot.join(sub), &root, &mut created)?;
    }

    // Copy the framework directories, file by file.
    let mut copied = serde_json::Map::new();
    let mut skill_names: Vec<String> = Vec::new();
    for name in COPIED {
        let src = from.join(name);
        let dst = root.join(".claude").join(name);
        let n = if *name == "skills" {
            let (count, names) = copy_skills(&src, &dst)?;
            skill_names = names;
            count
        } else {
            copy_tree(&src, &dst)?
        };
        copied.insert((*name).to_string(), serde_json::json!(n));
    }

    // The default gates file, byte for byte.
    project::atomic_write(&dot.join(project::GATES), gates::DEFAULT_GATES.as_bytes())?;
    // The initial state file.
    let now = crate::time::now_rfc3339();
    project::atomic_write(
        &dot.join(project::STATE),
        state::initial_toml(&now).as_bytes(),
    )?;

    // `stack detect` as the last write of the project files.
    let mut ctx = Ctx::new(root.clone());
    let detect = super::stack::detect(&mut ctx, false)?;
    let stacks: Vec<String> = detect.data["stacks"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|s| s["id"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let degraded = detect.data["degraded"].as_bool().unwrap_or(true);

    append_gitignore(&root)?;
    let claude_md_created = append_claude_md(&root)?;
    if claude_md_created {
        created.push("CLAUDE.md".to_string());
    }

    let mut hooks_data = serde_json::json!({ "settings": "skipped", "git": [] });
    if !args.no_hooks {
        let report = hooks::install(&root, args.force, false, false)?;
        warnings.extend(report.warnings.iter().cloned());
        hooks_data = serde_json::json!({
            "settings": report.settings,
            "git": report.git_hooks,
        });
    }

    let analyze = if args.analyze {
        let cfg = crate::config::load(&root)?;
        let a = crate::analyze::run(&root, &cfg, args.force)?;
        if a.truncated {
            warnings.push(Diag::new(
                "DFA-E113",
                format!(
                    "analysis stopped after {}s; {} files unread",
                    crate::analyze::TIME_CAP_SECS,
                    0
                ),
            ));
        }
        Some(a)
    } else {
        None
    };

    let mut human = vec![format!("Initialised .devforgeai/ in {}", root.display())];
    human.push(format!(
        "Copied     {} skills, {} agents",
        copied.get("skills").and_then(|v| v.as_u64()).unwrap_or(0),
        copied.get("agents").and_then(|v| v.as_u64()).unwrap_or(0),
    ));
    if !skill_names.is_empty() {
        // The slash names are what the user types, and they differ from the
        // framework's own directory names, so the summary says which commands
        // this install actually produced.
        human.push(format!("Commands   /{}", skill_names.join(", /")));
    }
    if !args.no_hooks {
        // The spec's example line shows the three-hook case only. An empty
        // list reads `none` rather than trailing off after the label, matching
        // the handoff convention for an empty field.
        let git_hooks = hooks_data["git"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();
        let git_hooks = if git_hooks.is_empty() {
            "none".to_string()
        } else {
            git_hooks
        };
        human.push(format!(
            "Hooks      .claude/settings.json {}; git hooks {git_hooks}",
            hooks_data["settings"].as_str().unwrap_or(""),
        ));
    }
    human.push(format!(
        "CLAUDE.md  devforgeai section {}",
        if claude_md_created {
            "written"
        } else {
            "updated"
        }
    ));
    human.push(format!(
        "Stack      {}",
        if stacks.is_empty() {
            "undetected".to_string()
        } else {
            stacks.join(", ")
        }
    ));
    if let Some(a) = &analyze {
        human.push(format!(
            "Analysis   {} context files drafted, {} stubbed, {} files scanned",
            a.derived.len(),
            a.stubs.len(),
            a.files_scanned
        ));
    }
    // An unpinned or altered binary refuses every write, every phase command,
    // and every turn end, so the next thing to do is the pin, not `/explore`.
    // Naming it here is the only place the message reaches an operator before
    // the refusals start.
    let trusted = trust::verify(None);
    match &trusted {
        Ok(_) => human.push("Next       /explore".to_string()),
        Err(e) => {
            let pin = format!(
                "devforgeai trust pin --framework {}",
                from.display().to_string().replace('\\', "/")
            );
            human.push(format!(
                "Next       {pin}, in a terminal outside Claude Code"
            ));
            warnings.push(Diag::new(
                e.code(),
                format!(
                    "{} hooks will refuse every write until the binary is pinned: {pin}",
                    e.diag().map(|d| d.message.clone()).unwrap_or_default()
                ),
            ));
        }
    }

    let mut out = Outcome::data(serde_json::json!({
        "created": created,
        "copied": copied,
        "commands": skill_names,
        "hooks": hooks_data,
        "stacks": stacks,
        "trusted": trusted.is_ok(),
        "degraded": degraded,
        "analyze": analyze.as_ref().map(|a| a.to_json()).unwrap_or(serde_json::Value::Null),
    }));
    out.human = human;
    out.warnings = warnings;
    out.degraded = degraded;
    out.project = root.display().to_string();
    Ok(out)
}

/// `--from` defaults to the `framework_path` of the matching `[[pin]]` in
/// `trust.toml`; absent, `DFA-E111`.
fn resolve_framework(flag: Option<&Path>) -> Result<PathBuf, CliError> {
    if let Some(p) = flag {
        if !p.is_dir() {
            return Err(CliError::new(
                "DFA-E111",
                "framework source not found; pass --from <path> to the DevForgeAI repo root",
            ));
        }
        return Ok(project::normalise(p));
    }
    // The default is the `framework_path` of the `[[pin]]` matching the running
    // binary, which ties installation to the binary a human already pinned.
    let pinned = trust::resolve_binary(None)
        .ok()
        .and_then(|bin| {
            let file = trust::load_trust_file().ok().flatten()?;
            let want = bin.to_string_lossy().to_string();
            file.pin
                .into_iter()
                .find(|p| p.binary_path == want)
                .map(|p| PathBuf::from(p.framework_path))
        })
        .filter(|p| p.is_dir());

    match pinned {
        Some(p) => Ok(p),
        None => Err(CliError::new(
            "DFA-E111",
            "framework source not found; pass --from <path> to the DevForgeAI repo root",
        )),
    }
}

fn mkdir(path: &Path, root: &Path, created: &mut Vec<String>) -> Result<(), CliError> {
    if path.is_dir() {
        return Ok(());
    }
    std::fs::create_dir_all(path).map_err(|e| CliError::io("creating", path.display(), &e))?;
    created.push(project::rel_display(root, path));
    Ok(())
}

/// Copy a directory tree file by file, returning the count of files copied.
/// An absent source is zero files, not an error: a framework that ships no
/// `agents/` is a smaller framework, not a broken one.
fn copy_tree(src: &Path, dst: &Path) -> Result<u64, CliError> {
    if !src.is_dir() {
        return Ok(0);
    }
    let mut n = 0u64;
    let walk = walkdir::WalkDir::new(src)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            !(e.file_type().is_dir()
                && e.path() != src
                && e.file_name()
                    .to_str()
                    .map(|n| EXCLUDED_SUBTREES.contains(&n))
                    .unwrap_or(false))
        });
    for entry in walk {
        let entry = entry.map_err(|e| {
            CliError::new(
                "DFA-E112",
                format!("copying {} to {} failed: {e}", src.display(), dst.display()),
            )
        })?;
        let rel = match entry.path().strip_prefix(src) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&target).map_err(|e| {
                CliError::new(
                    "DFA-E112",
                    format!(
                        "copying {} to {} failed: {e}",
                        entry.path().display(),
                        target.display()
                    ),
                )
            })?;
            continue;
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                CliError::new(
                    "DFA-E112",
                    format!(
                        "copying {} to {} failed: {e}",
                        src.display(),
                        parent.display()
                    ),
                )
            })?;
        }
        std::fs::copy(entry.path(), &target).map_err(|e| {
            CliError::new(
                "DFA-E112",
                format!(
                    "copying {} to {} failed: {e}",
                    entry.path().display(),
                    target.display()
                ),
            )
        })?;
        n += 1;
    }
    Ok(n)
}

/// Append the two lines when they are absent, creating the file when it does
/// not exist.
/// Copy each skill into `.claude/skills/<slash name>/`.
///
/// Claude Code derives a project skill's slash command from its *directory*
/// name; the frontmatter `name:` is a display label. The framework tree names
/// its directories for what the skill is (`exploring-ideas`), because that is
/// what `produced_by:` resolves against and what the producer check reads,
/// while the entry point the user types is `/explore`. Installing under the
/// frontmatter name is what makes the two agree, and it is read from each
/// SKILL.md rather than from a table here, so adding a skill needs no edit.
///
/// Returns the number of files copied and the slash names installed.
fn copy_skills(src: &Path, dst: &Path) -> Result<(u64, Vec<String>), CliError> {
    if !src.is_dir() {
        return Ok((0, Vec::new()));
    }
    let entries = std::fs::read_dir(src)
        .map_err(|e| CliError::io("reading", src.display(), &e))?
        .flatten()
        .filter(|e| e.path().is_dir());

    let mut total = 0u64;
    let mut installed: Vec<(String, String)> = Vec::new();
    for entry in entries {
        let dir = entry.file_name().to_string_lossy().to_string();
        let slash = skill_slash_name(&entry.path().join("SKILL.md")).unwrap_or_else(|| dir.clone());
        total += copy_tree(&entry.path(), &dst.join(&slash))?;
        installed.push((dir, slash));
    }

    // An install made before the directories were named for their commands
    // left `.claude/skills/exploring-ideas/` beside the new
    // `.claude/skills/explore/`, and both would register, so the same skill
    // would offer two slash commands. Only a directory whose own SKILL.md
    // claims the name we just installed is removed, so a project's own skill
    // that happens to share a directory name survives.
    for (dir, slash) in &installed {
        if dir == slash {
            continue;
        }
        let stale = dst.join(dir);
        if skill_slash_name(&stale.join("SKILL.md")).as_deref() == Some(slash.as_str()) {
            std::fs::remove_dir_all(&stale)
                .map_err(|e| CliError::io("removing", stale.display(), &e))?;
        }
    }

    let mut names: Vec<String> = installed.into_iter().map(|(_, slash)| slash).collect();
    names.sort();
    Ok((total, names))
}

/// The `name:` value of a SKILL.md's YAML frontmatter, or `None`.
fn skill_slash_name(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut lines = text.lines();
    // The frontmatter is the block between the first two `---` fences; a file
    // that does not open with one carries none.
    if lines.next()?.trim() != "---" {
        return None;
    }
    for line in lines {
        let trimmed = line.trim_end();
        if trimmed.trim() == "---" {
            return None;
        }
        if let Some(value) = trimmed.strip_prefix("name:") {
            let value = value.trim().trim_matches(['"', '\'']).to_string();
            return (!value.is_empty()).then_some(value);
        }
    }
    None
}

/// Write the DevForgeAI section into the project's CLAUDE.md.
///
/// An existing section is replaced between its markers, so the operation is
/// idempotent and touches no text the user wrote outside them. `--force` is not
/// consulted for that reason: unlike `.devforgeai/`, nothing here can be lost.
/// An absent file is created holding the block under an H1 naming the project
/// directory. Returns true when the file did not exist before.
fn append_claude_md(root: &Path) -> Result<bool, CliError> {
    let path = root.join("CLAUDE.md");
    let existing = match std::fs::read_to_string(&path) {
        Ok(t) => Some(t),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(CliError::io("reading", path.display(), &e)),
    };
    let created = existing.is_none();

    let text = match &existing {
        Some(text) if text.contains(CLAUDE_MD_BEGIN) => {
            let start = text.find(CLAUDE_MD_BEGIN).expect("just found");
            let end = match text[start..].find(CLAUDE_MD_END) {
                Some(rel) => start + rel + CLAUDE_MD_END.len(),
                // A begin marker with no end is repaired by replacing to the
                // end of the file rather than by writing a second section.
                None => text.len(),
            };
            format!("{}{}{}", &text[..start], CLAUDE_MD_BLOCK, &text[end..])
        }
        Some(text) => {
            let mut out = text.clone();
            while out.ends_with('\n') {
                out.pop();
            }
            if !out.is_empty() {
                out.push_str("\n\n");
            }
            out.push_str(CLAUDE_MD_BLOCK);
            out.push('\n');
            out
        }
        None => {
            let name = root
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Project".to_string());
            format!("# {name}\n\n{CLAUDE_MD_BLOCK}\n")
        }
    };

    project::atomic_write(&path, text.as_bytes())?;
    Ok(created)
}

fn append_gitignore(root: &Path) -> Result<(), CliError> {
    let path = root.join(".gitignore");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let present: Vec<&str> = existing.lines().map(str::trim).collect();

    let missing: Vec<&&str> = IGNORE_LINES
        .iter()
        .filter(|l| !present.contains(&l.trim()))
        .collect();
    if missing.is_empty() {
        return Ok(());
    }

    let mut out = existing.clone();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    for l in missing {
        out.push_str(l);
        out.push('\n');
    }
    project::atomic_write(&path, out.as_bytes())
}
