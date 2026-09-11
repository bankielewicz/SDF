//! Project root discovery, atomic writes, and the path constants under
//! `.devforgeai/`.

use crate::errors::CliError;
use std::path::{Path, PathBuf};

/// The directory every project artifact lives under.
pub const DOT: &str = ".devforgeai";

/// `.devforgeai/config.toml`.
pub const CONFIG: &str = "config.toml";
/// `.devforgeai/gates.toml`.
pub const GATES: &str = "gates.toml";
/// `.devforgeai/state.toml`.
pub const STATE: &str = "state.toml";

/// The subdirectories `init` creates under `.devforgeai/`.
pub const SUBDIRS: &[&str] = &[
    "explore", "context", "adr", "stories", "ui-specs", "brand", "reports", "releases",
];

/// Resolve the project root: `--project` when given, else the nearest ancestor
/// of `cwd` holding a `.devforgeai/` directory.
pub fn resolve_root(cwd: &Path, flag: Option<&Path>) -> Result<PathBuf, CliError> {
    if let Some(p) = flag {
        if !p.exists() {
            return Err(CliError::new(
                "DFA-E031",
                format!("--project path '{}' does not exist", p.display()),
            ));
        }
        return Ok(normalise(p));
    }
    discover(cwd).ok_or_else(|| {
        CliError::new(
            "DFA-E030",
            format!(
                "no .devforgeai/ directory in {} or any ancestor; run 'devforgeai init'",
                cwd.display()
            ),
        )
    })
}

/// The nearest ancestor of `cwd`, `cwd` included, holding `.devforgeai/`.
///
/// The trust store is `~/.devforgeai/`, which shares the marker directory's
/// name. Without the guard below, every directory under the home directory on a
/// machine that has ever been pinned resolves its project root to the home
/// directory itself, and the CLI reads the trust store as a project. A real
/// project at `~` still resolves, because that one holds a `config.toml` and
/// the trust store does not.
pub fn discover(cwd: &Path) -> Option<PathBuf> {
    let mut here: &Path = cwd;
    loop {
        let dot = here.join(DOT);
        if dot.is_dir() && !is_trust_store_only(&dot) {
            return Some(main_checkout_of(here));
        }
        here = here.parent()?;
    }
}

/// The main checkout of `found`, when `found` is a linked git worktree.
///
/// A worktree is a second checkout of the same repository, so it carries its
/// own tracked copy of `.devforgeai/` — the documents, `config.toml`,
/// `gates.toml` — and discovery stops there. `state.toml` is the exception:
/// it is gitignored and exists in the main checkout alone, because a phase
/// record in two places is two phase records. A Build run inside a worktree
/// therefore has to resolve to the main checkout, or `phase set` writes one
/// copy while the Stop hook, whose working directory is the session root,
/// reads the other and blocks on a phase the project has already left.
///
/// The absent `state.toml` is the signal, and it is a file test rather than a
/// subprocess, so the common case costs nothing. Only when it is missing does
/// this ask git where the real checkout is.
fn main_checkout_of(found: &Path) -> PathBuf {
    if found.join(DOT).join(STATE).is_file() {
        return normalise(found);
    }
    match linked_worktree_main(found) {
        // The main checkout is only the answer when it is a project too.
        Some(main) if main.join(DOT).is_dir() => normalise(&main),
        _ => normalise(found),
    }
}

/// The main checkout `dir` is a linked worktree of, or `None`.
///
/// In the main checkout `--git-dir` and `--git-common-dir` name the same
/// directory; in a linked worktree the first is `<main>/.git/worktrees/<name>`
/// and the second is `<main>/.git`. A cwd that is not a work tree at all, or
/// where git is not on `PATH`, answers `None` and discovery proceeds as it
/// always did.
fn linked_worktree_main(dir: &Path) -> Option<PathBuf> {
    let ask = |flag: &str| -> Option<PathBuf> {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["rev-parse", flag])
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let text = String::from_utf8(out.stdout).ok()?.trim().to_string();
        if text.is_empty() {
            return None;
        }
        let p = PathBuf::from(&text);
        Some(if p.is_absolute() { p } else { dir.join(p) })
    };

    let git_dir = ask("--git-dir")?;
    let common = ask("--git-common-dir")?;
    if normalise(&git_dir) == normalise(&common) {
        // The main checkout: nothing to redirect to.
        return None;
    }
    // `--git-common-dir` is `<main>/.git`, so its parent is the checkout.
    common.parent().map(Path::to_path_buf)
}

/// True when this `.devforgeai/` is the trust store and not a project.
fn is_trust_store_only(dot: &Path) -> bool {
    if dot.join(CONFIG).exists() {
        return false;
    }
    normalise(dot) == normalise(&crate::trust::trust_home())
}

/// Canonicalise where the platform allows it, and fall back to the path as
/// given. Canonicalisation is never required for correctness here, so a failure
/// is not an error.
pub fn normalise(p: &Path) -> PathBuf {
    match std::fs::canonicalize(p) {
        Ok(c) => strip_verbatim(&c),
        Err(_) => p.to_path_buf(),
    }
}

/// Drop the Windows `\\?\` verbatim prefix, which is correct as a path and
/// wrong in every message and every TOML value the CLI writes.
fn strip_verbatim(p: &Path) -> PathBuf {
    let s = p.to_string_lossy();
    match s.strip_prefix(r"\\?\") {
        Some(rest) => PathBuf::from(rest),
        None => p.to_path_buf(),
    }
}

/// `.devforgeai/` inside `root`.
pub fn dot(root: &Path) -> PathBuf {
    root.join(DOT)
}

/// Resolve a path the way `gates.toml` resolves one: a relative path against
/// `.devforgeai/`, except one beginning with `.`, which resolves against the
/// project root. An absolute path is returned unchanged.
pub fn resolve_doc_path(root: &Path, rel: &str) -> PathBuf {
    let p = Path::new(rel);
    if p.is_absolute() {
        return p.to_path_buf();
    }
    if rel.starts_with('.') {
        root.join(rel)
    } else {
        dot(root).join(rel)
    }
}

/// Write `bytes` to `path` atomically: a temporary file beside the destination,
/// flushed, then renamed over it.
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), CliError> {
    use std::io::Write;

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    std::fs::create_dir_all(parent).map_err(|e| CliError::io("creating", parent.display(), &e))?;

    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "devforgeai".to_string());
    let tmp = parent.join(format!("{name}.tmp-{}", std::process::id()));

    {
        let mut f =
            std::fs::File::create(&tmp).map_err(|e| CliError::io("creating", tmp.display(), &e))?;
        f.write_all(bytes)
            .map_err(|e| CliError::io("writing", tmp.display(), &e))?;
        f.flush()
            .map_err(|e| CliError::io("flushing", tmp.display(), &e))?;
        f.sync_all()
            .map_err(|e| CliError::io("syncing", tmp.display(), &e))?;
    }

    std::fs::rename(&tmp, path).map_err(|e| {
        CliError::at(
            "DFA-E901",
            format!(
                "replacing {} failed: {e}; the temporary file is {}",
                path.display(),
                tmp.display()
            ),
            path.display().to_string(),
        )
    })
}

/// Read a file, mapping an absent file to `DFA-E200` and every other failure to
/// `DFA-E900`.
pub fn read_doc(path: &Path) -> Result<String, CliError> {
    match std::fs::read_to_string(path) {
        Ok(s) => Ok(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(CliError::at(
            "DFA-E200",
            format!("{} not found", path.display()),
            path.display().to_string(),
        )),
        Err(e) => Err(CliError::io("reading", path.display(), &e)),
    }
}

/// Render `path` relative to `root` in forward-slash form, falling back to the
/// full path when it lies outside the root.
pub fn rel_display(root: &Path, path: &Path) -> String {
    match path.strip_prefix(root) {
        Ok(r) => r.to_string_lossy().replace('\\', "/"),
        Err(_) => path.to_string_lossy().replace('\\', "/"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp() -> tempfile::TempDir {
        tempfile::tempdir().expect("temp dir")
    }

    #[test]
    fn root_found_in_ancestor() {
        let t = temp();
        let root = t.path();
        std::fs::create_dir_all(root.join(DOT)).expect("mkdir");
        let deep = root.join("src").join("domain");
        std::fs::create_dir_all(&deep).expect("mkdir");

        let found = discover(&deep).expect("root discovered from a descendant");
        assert_eq!(found, normalise(root));
    }

    #[test]
    fn root_missing_gives_e030() {
        let t = temp();
        let err = resolve_root(t.path(), None).expect_err("no .devforgeai anywhere above");
        assert_eq!(err.code(), "DFA-E030");
        assert_eq!(err.exit(), 3);
    }

    #[test]
    fn project_flag_overrides_discovery() {
        let t = temp();
        let a = t.path().join("a");
        let b = t.path().join("b");
        std::fs::create_dir_all(a.join(DOT)).expect("mkdir");
        std::fs::create_dir_all(b.join(DOT)).expect("mkdir");

        let resolved = resolve_root(&a, Some(&b)).expect("flag wins");
        assert_eq!(resolved, normalise(&b));
    }

    #[test]
    fn project_flag_missing_gives_e031() {
        let t = temp();
        let absent = t.path().join("nowhere");
        let err = resolve_root(t.path(), Some(&absent)).expect_err("flag path absent");
        assert_eq!(err.code(), "DFA-E031");
        assert_eq!(err.exit(), 3);
    }

    #[test]
    fn atomic_write_replaces_file() {
        let t = temp();
        let p = t.path().join("sub").join("config.toml");
        atomic_write(&p, b"first").expect("first write");
        assert_eq!(std::fs::read(&p).expect("read"), b"first");

        atomic_write(&p, b"second").expect("second write");
        assert_eq!(std::fs::read(&p).expect("read"), b"second");

        // No temporary file survives a successful write.
        let leftovers: Vec<_> = std::fs::read_dir(p.parent().expect("parent"))
            .expect("read_dir")
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp-"))
            .collect();
        assert!(leftovers.is_empty(), "no .tmp- file survives");
    }

    #[test]
    fn atomic_write_leaves_tmp_on_rename_failure() {
        let t = temp();
        // A directory at the destination path makes the rename fail on every
        // platform the binary supports.
        let p = t.path().join("occupied");
        std::fs::create_dir_all(p.join("child")).expect("mkdir");

        let err = atomic_write(&p, b"payload").expect_err("rename over a directory fails");
        assert_eq!(err.code(), "DFA-E901");
        assert_eq!(err.exit(), 5);

        let tmp = t
            .path()
            .join(format!("occupied.tmp-{}", std::process::id()));
        assert!(tmp.exists(), "the temporary file is left for the operator");
        assert_eq!(std::fs::read(&tmp).expect("read tmp"), b"payload");
        assert!(
            err.diag().expect("diag").message.contains("occupied.tmp-"),
            "the message names the temporary file"
        );
    }

    #[test]
    fn doc_path_rooting_follows_the_dot_rule() {
        let root = Path::new("/proj");
        assert_eq!(
            resolve_doc_path(root, "explore/decision.yaml"),
            root.join(".devforgeai").join("explore/decision.yaml")
        );
        assert_eq!(
            resolve_doc_path(root, ".devforgeai/stories/sprint.yaml"),
            root.join(".devforgeai/stories/sprint.yaml")
        );
        assert_eq!(
            resolve_doc_path(root, ".explore-prototype"),
            root.join(".explore-prototype")
        );
    }
}
