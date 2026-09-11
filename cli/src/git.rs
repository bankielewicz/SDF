//! The `git` calls `story files --diff`, `worktree`, and `commit` make.
//!
//! Every call runs `git -C <root>` with the arguments given, so the binary
//! never changes its own working directory and two calls never race.

use crate::errors::CliError;
use std::path::Path;
use std::process::Command;

/// What one `git` invocation produced.
#[derive(Debug, Clone)]
pub struct Output {
    /// The process exit code, or -1 when it was killed by a signal.
    pub code: i32,
    /// stdout, lossily decoded.
    pub stdout: String,
    /// stderr, lossily decoded.
    pub stderr: String,
}

impl Output {
    /// True when git exited 0.
    pub fn ok(&self) -> bool {
        self.code == 0
    }

    /// stdout split into trimmed non-empty lines.
    pub fn lines(&self) -> Vec<String> {
        self.stdout
            .lines()
            .map(str::trim_end)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect()
    }
}

/// Run `git -C <dir> <args>`, whatever its exit status.
pub fn run(dir: &Path, args: &[&str]) -> Result<Output, CliError> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .map_err(|e| {
            CliError::new(
                "DFA-E900",
                format!("running git {} failed: {e}", args.join(" ")),
            )
        })?;
    Ok(Output {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).to_string(),
        stderr: String::from_utf8_lossy(&out.stderr).to_string(),
    })
}

/// `DFA-E271` when `root` is not a git work tree.
pub fn require_work_tree(root: &Path) -> Result<(), CliError> {
    let out = run(root, &["rev-parse", "--is-inside-work-tree"])?;
    if out.ok() && out.stdout.trim() == "true" {
        return Ok(());
    }
    Err(CliError::at(
        "DFA-E271",
        format!("{} is not a git work tree; run git init", root.display()),
        root.display().to_string(),
    ))
}

/// True when `root` is a git work tree.
pub fn is_work_tree(root: &Path) -> bool {
    run(root, &["rev-parse", "--is-inside-work-tree"])
        .map(|o| o.ok() && o.stdout.trim() == "true")
        .unwrap_or(false)
}

/// The merge base of `a` and `HEAD`, or `a` itself when there is none, which
/// is the state of a repository with no commit on the base ref.
pub fn merge_base(root: &Path, a: &str) -> Result<String, CliError> {
    let out = run(root, &["merge-base", a, "HEAD"])?;
    if out.ok() && !out.stdout.trim().is_empty() {
        return Ok(out.stdout.trim().to_string());
    }
    Ok(a.to_string())
}

/// True when the repository has at least one commit.
pub fn has_commit(root: &Path) -> bool {
    run(root, &["rev-parse", "--verify", "HEAD"])
        .map(|o| o.ok())
        .unwrap_or(false)
}

/// One changed path, with the porcelain status letter mapped to a word.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    /// Repo-relative, forward slashes.
    pub path: String,
    /// `added`, `modified`, `deleted`, `renamed`, or `untracked`.
    pub status: String,
}

/// The status letter as the word the `data` object carries.
fn word(letter: char) -> &'static str {
    match letter {
        'A' => "added",
        'D' => "deleted",
        'R' => "renamed",
        'C' => "copied",
        '?' => "untracked",
        _ => "modified",
    }
}

/// The union of the paths changed between `base` and `HEAD`, the tracked
/// changes of the work tree, and its untracked files, sorted and deduplicated.
///
/// The work-tree half reads `git diff --name-status HEAD` and
/// `git ls-files --others` rather than `git status --porcelain`, because
/// `status` reports a line-ending renormalisation as a modification where
/// `diff` reports no content change at all. On Windows, with
/// `core.autocrlf = true`, that difference is the whole answer for a file the
/// binary writes itself.
pub fn changed(root: &Path, base: &str) -> Result<Vec<Change>, CliError> {
    let mut out: Vec<Change> = Vec::new();

    if has_commit(root) {
        if base != "HEAD" {
            let diff = run(root, &["diff", "--name-status", &format!("{base}..HEAD")])?;
            if diff.ok() {
                out.extend(name_status(&diff));
            }
        }
        let work = run(root, &["diff", "--name-status", "HEAD"])?;
        if work.ok() {
            out.extend(name_status(&work));
        }
    }

    let untracked = run(root, &["ls-files", "--others", "--exclude-standard"])?;
    if untracked.ok() {
        for path in untracked.lines() {
            out.push(Change {
                path: path.trim().replace('\\', "/"),
                status: "untracked".to_string(),
            });
        }
    }

    out.sort_by(|a, b| a.path.cmp(&b.path));
    out.dedup_by(|a, b| a.path == b.path);
    Ok(out)
}

/// The rows of a `--name-status` diff.
fn name_status(out: &Output) -> Vec<Change> {
    let mut rows = Vec::new();
    for line in out.lines() {
        let mut parts = line.split('\t');
        let letter = parts.next().and_then(|s| s.chars().next()).unwrap_or('M');
        // A rename row carries the old path then the new one.
        let path = parts.next_back().unwrap_or("").trim();
        if !path.is_empty() {
            rows.push(Change {
                path: path.replace('\\', "/"),
                status: word(letter).to_string(),
            });
        }
    }
    rows
}

/// The uncommitted changes of a work tree, content-wise, plus its untracked
/// files: the count `worktree list` and `worktree remove` report.
pub fn uncommitted(root: &Path) -> Result<Vec<Change>, CliError> {
    changed(root, "HEAD")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_status_letter_becomes_a_word() {
        assert_eq!(word('A'), "added");
        assert_eq!(word('?'), "untracked");
        assert_eq!(word('M'), "modified");
        assert_eq!(word('X'), "modified");
    }

    #[test]
    fn a_directory_that_is_no_repository_refuses_with_e271() {
        let dir = tempfile::tempdir().expect("temp");
        let err = require_work_tree(dir.path()).expect_err("not a work tree");
        assert_eq!(err.code(), "DFA-E271");
        assert_eq!(err.exit(), 1);
    }
}
