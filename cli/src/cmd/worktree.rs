//! `worktree ensure`, `worktree list`, and `worktree remove`.
//!
//! Every call shells out to `git`; the binary owns the naming rule, the
//! overlap refusal, and the state seeding, and nothing else.

use crate::ctx::Ctx;
use crate::errors::CliError;
use crate::{git, project, story, Outcome};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// One worktree of this repository.
#[derive(Debug, Clone)]
pub struct Entry {
    /// The `STORY-nnn` encoded in the directory name, or `""`.
    pub id: String,
    /// The path as `worktree_root` names it.
    pub rel: String,
    /// The absolute path.
    pub path: PathBuf,
    /// The branch, without `refs/heads/`.
    pub branch: String,
}

/// The naming rule: `<worktree_root>/<id>` and `<branch_prefix><id>`.
struct Naming {
    root: String,
    prefix: String,
    base_ref: String,
}

fn naming(ctx: &mut Ctx) -> Result<Naming, CliError> {
    let c = ctx.config()?;
    Ok(Naming {
        root: c.build.worktree_root.clone(),
        prefix: c.build.branch_prefix.clone(),
        base_ref: c.build.base_ref.clone(),
    })
}

/// `<worktree_root>/<id>` as written, and resolved against the project root.
fn location(root: &Path, n: &Naming, id: &str) -> (String, PathBuf) {
    let rel = format!("{}/{id}", n.root.trim_end_matches('/'));
    (rel.clone(), project::normalise(&root.join(&rel)))
}

/// Every worktree `git worktree list --porcelain` reports, with the story id
/// taken from the directory name when the path sits under `worktree_root`.
pub fn entries(root: &Path, worktree_root: &str) -> Result<Vec<Entry>, CliError> {
    let out = git::run(root, &["worktree", "list", "--porcelain"])?;
    let holder = project::normalise(&root.join(worktree_root.trim_end_matches('/')));

    let mut all = Vec::new();
    let mut path: Option<PathBuf> = None;
    let mut branch = String::new();

    let flush = |path: &mut Option<PathBuf>, branch: &mut String, all: &mut Vec<Entry>| {
        if let Some(p) = path.take() {
            let under = p.parent().map(|d| d == holder).unwrap_or(false);
            let name = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            let id = if under && name.starts_with("STORY-") && crate::doc::ids::is_id(&name) {
                name.clone()
            } else {
                String::new()
            };
            all.push(Entry {
                id,
                rel: format!("{}/{name}", worktree_root.trim_end_matches('/')),
                path: p,
                branch: std::mem::take(branch),
            });
        }
    };

    for line in out.stdout.lines() {
        if let Some(rest) = line.strip_prefix("worktree ") {
            flush(&mut path, &mut branch, &mut all);
            path = Some(project::normalise(Path::new(rest.trim())));
        } else if let Some(rest) = line.strip_prefix("branch ") {
            branch = rest.trim().trim_start_matches("refs/heads/").to_string();
        }
    }
    flush(&mut path, &mut branch, &mut all);

    Ok(all.into_iter().filter(|e| !e.id.is_empty()).collect())
}

/// The worktree of one story, when it exists.
pub fn find(root: &Path, worktree_root: &str, id: &str) -> Result<Option<Entry>, CliError> {
    Ok(entries(root, worktree_root)?
        .into_iter()
        .find(|e| e.id == id))
}

// ---------------------------------------------------------------- ensure

/// Create the worktree of a story when it is absent, refusing an overlap.
pub fn ensure(ctx: &mut Ctx, id: &str) -> Result<Outcome, CliError> {
    check_id(id)?;
    let root = ctx.root.clone();
    git::require_work_tree(&root)?;
    let n = naming(ctx)?;
    let (rel, abs) = location(&root, &n, id);
    let branch = format!("{}{id}", n.prefix);

    // Idempotent: an existing worktree on the matching branch is the answer.
    if let Some(e) = find(&root, &n.root, id)? {
        if e.branch == branch {
            return Ok(done(ctx, id, &rel, &branch, false, false));
        }
    }

    // The overlap refusal, against every other worktree under the holder.
    let mine: BTreeSet<String> = declared(&root, id)?;
    for other in entries(&root, &n.root)? {
        if other.id == id {
            continue;
        }
        let theirs = declared(&root, &other.id)?;
        if let Some(shared) = mine.intersection(&theirs).next() {
            return Err(CliError::new(
                "DFA-E272",
                format!(
                    "{id} and {} both declare {shared}; one worktree at a time",
                    other.id
                ),
            ));
        }
    }

    let out = git::run(
        &root,
        &[
            "worktree",
            "add",
            "-b",
            &branch,
            &abs.to_string_lossy(),
            &n.base_ref,
        ],
    )?;
    if !out.ok() {
        return Err(CliError::new(
            "DFA-E900",
            format!("git worktree add for {id} failed: {}", out.stderr.trim()),
        ));
    }

    let seeded = seed_state(&root, &abs);
    Ok(done(ctx, id, &rel, &branch, true, seeded))
}

/// Copy the main checkout's `state.toml` into the worktree.
fn seed_state(root: &Path, worktree: &Path) -> bool {
    let src = project::dot(root).join(project::STATE);
    let dst = project::dot(worktree).join(project::STATE);
    let Ok(bytes) = std::fs::read(&src) else {
        return false;
    };
    if let Some(parent) = dst.parent() {
        if std::fs::create_dir_all(parent).is_err() {
            return false;
        }
    }
    project::atomic_write(&dst, &bytes).is_ok()
}

fn done(
    ctx: &Ctx,
    id: &str,
    rel: &str,
    branch: &str,
    created: bool,
    state_seeded: bool,
) -> Outcome {
    Outcome {
        human: vec![rel.to_string()],
        data: serde_json::json!({
            "id": id,
            "path": rel,
            "branch": branch,
            "created": created,
            "state_seeded": state_seeded,
        }),
        degraded: ctx.degraded(),
        project: ctx.root.display().to_string(),
        ..Default::default()
    }
}

/// The `## Files` `Path` set of a story, empty when the story has no file.
fn declared(root: &Path, id: &str) -> Result<BTreeSet<String>, CliError> {
    if !story::path_of(root, id).exists() {
        return Ok(BTreeSet::new());
    }
    Ok(story::load(root, id)?
        .files
        .into_iter()
        .map(|f| f.path)
        .collect())
}

// ------------------------------------------------------------------ list

/// One line per worktree under `worktree_root`.
pub fn list(ctx: &mut Ctx) -> Result<Outcome, CliError> {
    let root = ctx.root.clone();
    git::require_work_tree(&root)?;
    let n = naming(ctx)?;

    let mut human = Vec::new();
    let mut rows = Vec::new();
    for e in entries(&root, &n.root)? {
        let dirty = is_dirty(&e.path);
        let ahead = ahead_count(&e.path, &n.base_ref);
        human.push(format!(
            "{}  {}  {}  {}  {ahead} ahead",
            e.id,
            e.rel,
            e.branch,
            if dirty { "dirty" } else { "clean" }
        ));
        rows.push(serde_json::json!({
            "id": e.id,
            "path": e.rel,
            "branch": e.branch,
            "dirty": dirty,
            "ahead": ahead,
        }));
    }

    Ok(Outcome {
        human,
        data: serde_json::json!({ "worktrees": rows }),
        degraded: ctx.degraded(),
        project: root.display().to_string(),
        ..Default::default()
    })
}

/// True when the work tree holds an uncommitted change.
fn is_dirty(path: &Path) -> bool {
    dirty_count(path) > 0
}

/// The number of uncommitted changes, the seeded `state.toml` excluded: the
/// binary writes that file itself as part of `worktree ensure`, so counting it
/// would leave every worktree permanently unremovable.
fn dirty_count(path: &Path) -> usize {
    let seeded = format!("{}/{}", project::DOT, project::STATE);
    git::uncommitted(path)
        .map(|c| c.iter().filter(|x| x.path != seeded).count())
        .unwrap_or(0)
}

/// Commits on HEAD that `base_ref` does not carry.
fn ahead_count(path: &Path, base_ref: &str) -> usize {
    git::run(path, &["rev-list", "--count", &format!("{base_ref}..HEAD")])
        .ok()
        .filter(|o| o.ok())
        .and_then(|o| o.stdout.trim().parse().ok())
        .unwrap_or(0)
}

// ---------------------------------------------------------------- remove

/// Remove the worktree of a story, refusing unfinished work without `--force`.
pub fn remove(ctx: &mut Ctx, id: &str, force: bool) -> Result<Outcome, CliError> {
    check_id(id)?;
    let root = ctx.root.clone();
    git::require_work_tree(&root)?;
    let n = naming(ctx)?;
    let branch = format!("{}{id}", n.prefix);

    let Some(e) = find(&root, &n.root, id)? else {
        return Ok(Outcome {
            human: vec![format!("{id} has no worktree; nothing removed")],
            data: serde_json::json!({
                "id": id, "removed": false, "branch_deleted": false,
            }),
            degraded: ctx.degraded(),
            project: root.display().to_string(),
            ..Default::default()
        });
    };

    let dirty = dirty_count(&e.path);
    let ahead = ahead_count(&e.path, &n.base_ref);
    if (dirty > 0 || ahead > 0) && !force {
        return Err(CliError::at(
            "DFA-E273",
            format!(
                "{} holds {dirty} uncommitted changes and {ahead} commits absent from {}; pass --force",
                e.rel, n.base_ref
            ),
            e.rel.clone(),
        ));
    }

    // The refusal above is this binary's, and it is the one that governs: git
    // would refuse again over the `state.toml` that `ensure` seeded, which is
    // the binary's own bookkeeping rather than the author's work.
    let path = e.path.to_string_lossy().to_string();
    let out = git::run(&root, &["worktree", "remove", "--force", &path])?;
    if !out.ok() {
        return Err(CliError::new(
            "DFA-E900",
            format!("git worktree remove for {id} failed: {}", out.stderr.trim()),
        ));
    }

    // The branch goes only when `base_ref` already carries its commits.
    let branch_deleted = git::run(&root, &["branch", "-d", &branch])
        .map(|o| o.ok())
        .unwrap_or(false);

    Ok(Outcome {
        human: vec![format!(
            "Removed   {}  {}",
            e.rel,
            if branch_deleted {
                format!("and deleted {branch}")
            } else {
                format!("and kept {branch}")
            }
        )],
        data: serde_json::json!({
            "id": id,
            "removed": true,
            "branch_deleted": branch_deleted,
        }),
        degraded: ctx.degraded(),
        project: root.display().to_string(),
        ..Default::default()
    })
}

/// A `STORY-nnn` argument, `DFA-E012` otherwise, which is the code the
/// subcommand's exit table names for a malformed argument.
fn check_id(id: &str) -> Result<(), CliError> {
    if id.starts_with("STORY-") && crate::doc::ids::is_id(id) {
        return Ok(());
    }
    Err(CliError::new(
        "DFA-E012",
        format!("'{id}' is not a STORY id; expected STORY-nnn with three digits"),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_malformed_story_id_is_e012_exit_three() {
        let err = check_id("STORY-14").expect_err("refused");
        assert_eq!(err.code(), "DFA-E012");
        assert_eq!(err.exit(), 3);
        assert!(check_id("STORY-014").is_ok());
    }

    #[test]
    fn the_location_joins_the_root_and_the_id() {
        let n = Naming {
            root: "../wt/".into(),
            prefix: "story/".into(),
            base_ref: "HEAD".into(),
        };
        let (rel, _) = location(Path::new("/tmp/p"), &n, "STORY-014");
        assert_eq!(rel, "../wt/STORY-014");
    }
}
