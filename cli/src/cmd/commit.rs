//! `commit <STORY-nnn> -m <message>`: stage the story's declared paths and
//! commit through git, so the installed `pre-commit` and `commit-msg` hooks
//! run unchanged.

use crate::cli::CommitArgs;
use crate::ctx::Ctx;
use crate::errors::{CliError, Diag};
use crate::{git, story, Outcome};

/// The git hooks this command's commit can trigger.
const HOOKS: &[&str] = &["pre-commit", "commit-msg"];

/// Stage and commit.
pub fn run(ctx: &mut Ctx, a: &CommitArgs) -> Result<Outcome, CliError> {
    if !a.id.starts_with("STORY-") || !crate::doc::ids::is_id(&a.id) {
        return Err(CliError::new(
            "DFA-E012",
            format!("'{}' is not a STORY id; expected STORY-nnn", a.id),
        ));
    }
    if a.message.trim().is_empty() {
        return Err(CliError::new("DFA-E011", "'commit' requires -m <message>"));
    }

    let root = ctx.root.clone();
    git::require_work_tree(&root)?;
    let s = story::load(&root, &a.id)?;
    let declared: Vec<&str> = s.files.iter().map(|f| f.path.as_str()).collect();

    let paths: Vec<String> = match &a.paths {
        Some(list) => list
            .split(',')
            .map(story::normalise_path)
            .filter(|p| !p.is_empty())
            .collect(),
        None => git::changed(&root, "HEAD")?
            .into_iter()
            .map(|c| c.path)
            .collect(),
    };

    // A path under `.devforgeai/` is a framework artifact, not story work: the
    // PreToolUse map routes those to `doc validate` rather than to the
    // declared-set rule, and the `pre-commit` hook expects to see them staged.
    let (framework, work): (Vec<String>, Vec<String>) = paths
        .into_iter()
        .partition(|p| p.starts_with(".devforgeai/"));

    for p in &work {
        if !declared.contains(&p.as_str()) {
            return Err(CliError::new(
                "DFA-E239",
                format!("{p} is outside the declared file set of {}", a.id),
            ));
        }
    }

    let staged: Vec<String> = work.into_iter().chain(framework).collect();
    if staged.is_empty() {
        return Ok(Outcome {
            human: vec!["no declared path changed; nothing committed".to_string()],
            data: serde_json::json!({
                "id": a.id, "commit": "", "staged": 0,
                "message": "", "hooks": installed(ctx),
            }),
            warnings: vec![Diag::new(
                "DFA-W243",
                "no declared path changed; nothing committed",
            )],
            degraded: ctx.degraded(),
            project: root.display().to_string(),
            ..Default::default()
        });
    }

    let mut add: Vec<&str> = vec!["add", "--"];
    add.extend(staged.iter().map(String::as_str));
    let out = git::run(&root, &add)?;
    if !out.ok() {
        return Err(CliError::new(
            "DFA-E900",
            format!("git add failed: {}", out.stderr.trim()),
        ));
    }

    let message = message_for(&a.id, &a.message);
    let commit = git::run(&root, &["commit", "-m", &message])?;
    if !commit.ok() {
        // A refusing hook is the common case. The spec defines no code for it:
        // the hook's own stderr goes to stderr and the command exits 1.
        let text = if commit.stderr.trim().is_empty() {
            commit.stdout.trim()
        } else {
            commit.stderr.trim()
        };
        return Ok(Outcome {
            human: Vec::new(),
            data: serde_json::json!({
                "id": a.id,
                "commit": "",
                "staged": 0,
                "message": message,
                "hooks": installed(ctx),
                "hook_exit": commit.code,
                "hook_stderr": text,
            }),
            degraded: ctx.degraded(),
            project: root.display().to_string(),
            exit: Some(1),
            // The refusing hook's own stderr is the diagnostic; the error table
            // defines no code for it, so `errors[]` stays empty here and the
            // text travels on `stderr` and in `data.hook_stderr`.
            stderr: text.lines().map(str::to_string).collect(),
            ..Default::default()
        });
    }

    let sha = git::run(&root, &["rev-parse", "--short", "HEAD"])?
        .stdout
        .trim()
        .to_string();

    Ok(Outcome {
        human: vec![format!(
            "Commit    {sha}  {message}  {} files",
            staged.len()
        )],
        data: serde_json::json!({
            "id": a.id,
            "commit": sha,
            "staged": staged.len(),
            "message": message,
            "hooks": installed(ctx),
        }),
        degraded: ctx.degraded(),
        project: root.display().to_string(),
        ..Default::default()
    })
}

/// `<STORY-nnn>: <message>`, or the message verbatim when it already carries a
/// `STORY-nnn` or `ADR-nnn` token, which is what the `commit-msg` hook wants.
pub fn message_for(id: &str, message: &str) -> String {
    if carries_id(message) {
        message.to_string()
    } else {
        format!("{id}: {message}")
    }
}

/// True when the text holds a `STORY-nnn` or `ADR-nnn` token.
fn carries_id(text: &str) -> bool {
    crate::doc::ids::scan_ids(text)
        .into_iter()
        .any(|(_, id)| id.starts_with("STORY-") || id.starts_with("ADR-"))
}

/// The hook scripts that exist in `.git/hooks/`, among the two a commit runs.
fn installed(ctx: &Ctx) -> Vec<String> {
    let dir = ctx.root.join(".git").join("hooks");
    HOOKS
        .iter()
        .filter(|h| dir.join(h).exists())
        .map(|h| (*h).to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_message_without_an_id_takes_the_story_prefix() {
        assert_eq!(
            message_for("STORY-014", "AC-003 green"),
            "STORY-014: AC-003 green"
        );
    }

    #[test]
    fn a_message_already_carrying_the_story_id_is_verbatim() {
        assert_eq!(
            message_for("STORY-014", "STORY-014 AC-003 green"),
            "STORY-014 AC-003 green"
        );
    }

    #[test]
    fn an_adr_id_also_satisfies_the_commit_msg_hook() {
        assert_eq!(
            message_for("STORY-014", "ADR-004 records the choice"),
            "ADR-004 records the choice"
        );
    }

    #[test]
    fn another_prefix_does_not_count() {
        assert_eq!(
            message_for("STORY-014", "REQ-001 done"),
            "STORY-014: REQ-001 done"
        );
    }
}
