//! `hook install`: merging the Claude settings block and writing the three git
//! hook scripts.
//!
//! `hook run`, the stdin dispatcher, lives beside this module and is not part
//! of it.

pub mod run;

pub mod githooks;
pub mod settings;

use crate::errors::{CliError, Diag};
use crate::project::{atomic_write, rel_display};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// What one `hook install` did, in the shape the `--json` `data` object and the
/// human line are built from.
#[derive(Debug, Default, Clone)]
pub struct InstallReport {
    /// `merged` when the settings block was merged, `skipped` under
    /// `--git-only`.
    pub settings: String,
    /// The event keys the block installs, in the spec's order. Empty under
    /// `--git-only`.
    pub events: Vec<String>,
    /// The git hook file names written, in write order. Empty under
    /// `--claude-only` and when no git directory was found.
    pub git_hooks: Vec<String>,
    /// The backup file, relative to the project root, or `""` when there was no
    /// previous settings file to copy.
    pub backup: String,
    /// The `permissions.allow` rules this install added. Empty when they were
    /// already present.
    pub permissions: Vec<String>,
    /// `DFA-W130` for an entry already present, `DFA-E130` for a project with
    /// no git directory. Both leave the exit code at 0.
    pub warnings: Vec<Diag>,
}

/// Install the hooks into `root`.
///
/// `force` replaces a git hook this binary did not write. `claude_only` skips
/// the git hooks; `git_only` skips the settings merge. The caller rejects the
/// combination of both, which is `DFA-E010`.
pub fn install(
    root: &Path,
    force: bool,
    claude_only: bool,
    git_only: bool,
) -> Result<InstallReport, CliError> {
    let token = githooks::token_value();
    let mut report = InstallReport {
        settings: "skipped".to_string(),
        ..Default::default()
    };

    if !git_only {
        merge_settings(root, &token, &mut report)?;
    }
    if !claude_only {
        write_git_hooks(root, &token, force, &mut report)?;
    }

    Ok(report)
}

/// Read, merge, back up, and write `.claude/settings.json`.
fn merge_settings(root: &Path, token: &str, report: &mut InstallReport) -> Result<(), CliError> {
    let path = root.join(".claude").join("settings.json");
    let rel = rel_display(root, &path);

    let previous = match std::fs::read_to_string(&path) {
        Ok(s) => Some(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(CliError::io("reading", path.display(), &e)),
    };

    let mut existing: Value = match &previous {
        Some(text) => serde_json::from_str(text).map_err(|e| {
            CliError::at(
                "DFA-E131",
                format!("{rel} is not valid JSON: {e}; nothing merged"),
                rel.clone(),
            )
        })?,
        None => Value::Object(serde_json::Map::new()),
    };

    let block_text = githooks::substitute(settings::TEMPLATE, token);
    let mut block: Value = serde_json::from_str(&block_text).map_err(|e| {
        CliError::new(
            "DFA-E900",
            format!("the compiled-in settings template is not valid JSON: {e}"),
        )
    })?;

    // The shell `if` rule and the `SubagentStop` matcher are per-project and
    // cannot be a static template. A project with no config yet resolves
    // neither, which drops both rather than writing a filter that never fires.
    let (test_commands, verifiers) = project_tokens(root);
    settings::resolve_project_tokens(&mut block, &test_commands, &verifiers);

    let merged = settings::merge(&mut existing, &block);
    let granted = settings::merge_permissions(&mut existing);
    for event in &merged.already_present {
        report.warnings.push(Diag::at(
            "DFA-W130",
            format!("{event} hook already present; left unchanged"),
            rel.clone(),
        ));
    }

    // The copy happens before the write, so a failure part way through leaves
    // the operator the file as it stood.
    if let Some(text) = &previous {
        let name = format!("settings.json.bak-{}", crate::time::now_basic());
        let backup = path.with_file_name(&name);
        std::fs::write(&backup, text).map_err(|e| CliError::io("writing", backup.display(), &e))?;
        report.backup = rel_display(root, &backup);
    }

    atomic_write(&path, settings::render(&existing).as_bytes())?;
    report.settings = "merged".to_string();
    report.events = merged.events;
    report.permissions = granted;
    Ok(())
}

/// The project's test commands and registered verifier names, for the two
/// per-project substitution tokens. A project whose `config.toml` is absent or
/// unparsable yields neither, which drops both filters.
fn project_tokens(root: &Path) -> (Vec<String>, Vec<String>) {
    match crate::config::load(root) {
        Ok(cfg) => (
            cfg.stack.iter().map(|s| s.test_command.clone()).collect(),
            cfg.verifier.iter().map(|v| v.name.clone()).collect(),
        ),
        Err(_) => (Vec::new(), Vec::new()),
    }
}

/// Write the three scripts into the git hooks directory.
fn write_git_hooks(
    root: &Path,
    token: &str,
    force: bool,
    report: &mut InstallReport,
) -> Result<(), CliError> {
    let Some(git_dir) = git_common_dir(root) else {
        report.warnings.push(Diag::new(
            "DFA-E130",
            "no .git directory; git hooks not installed",
        ));
        return Ok(());
    };

    let hooks_dir = git_dir.join("hooks");
    std::fs::create_dir_all(&hooks_dir)
        .map_err(|e| CliError::io("creating", hooks_dir.display(), &e))?;

    for (name, body) in githooks::SCRIPTS {
        let target = hooks_dir.join(name);
        if target.exists() && !force {
            let current = std::fs::read(&target)
                .map_err(|e| CliError::io("reading", target.display(), &e))?;
            let text = String::from_utf8_lossy(&current);
            if !githooks::has_marker(&text) {
                return Err(CliError::at(
                    "DFA-E132",
                    format!(
                        "{name} exists and is not a devforgeai hook; pass --force to replace it"
                    ),
                    rel_display(root, &target),
                ));
            }
        }

        atomic_write(&target, githooks::substitute(body, token).as_bytes())?;
        make_executable(&target)?;
        report.git_hooks.push((*name).to_string());
    }

    Ok(())
}

/// Set mode `0755` on Unix. Windows has no mode to set: Git for Windows runs
/// the hook through its bundled `sh` whatever the file's attributes.
#[cfg(unix)]
fn make_executable(path: &Path) -> Result<(), CliError> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
        .map_err(|e| CliError::io("setting the mode of", path.display(), &e))
}

/// The Windows arm: no mode call.
#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<(), CliError> {
    Ok(())
}

/// The git common directory for `root`, so a linked worktree and a repository
/// whose git directory lives outside the work tree both resolve.
///
/// `git rev-parse --git-common-dir` prints a path relative to the work tree at
/// the repository root, so a relative answer is joined onto `root`. When git is
/// absent or fails, a `.git` directory inside `root` is the fallback; a project
/// with neither yields `None`, which the caller reports as `DFA-E130`.
fn git_common_dir(root: &Path) -> Option<PathBuf> {
    from_git(root).or_else(|| {
        let fallback = root.join(".git");
        fallback.is_dir().then_some(fallback)
    })
}

/// Ask git, returning `None` when git is absent, fails, or names a path that is
/// not a directory.
fn from_git(root: &Path) -> Option<PathBuf> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--git-common-dir"])
        .current_dir(root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if text.is_empty() {
        return None;
    }
    let p = Path::new(&text);
    let resolved = if p.is_absolute() {
        p.to_path_buf()
    } else {
        root.join(p)
    };
    resolved.is_dir().then_some(resolved)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_common_dir_falls_back_to_a_plain_dot_git() {
        let t = tempfile::tempdir().expect("temp dir");
        std::fs::create_dir_all(t.path().join(".git")).expect("mkdir .git");
        let found = git_common_dir(t.path()).expect("a .git directory resolves");
        assert!(found.ends_with(".git"));
    }
}
