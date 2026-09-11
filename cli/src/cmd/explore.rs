//! `explore prune --id <IDEA-nnn>`: remove the prototype directory once the
//! explore decision is a kill or a promote.

use crate::ctx::Ctx;
use crate::errors::CliError;
use crate::Outcome;
use std::path::Path;

/// The directory the Explore skill writes its throwaway prototype into.
pub const PROTOTYPE: &str = ".explore-prototype";

/// The explore decision document, relative to `.devforgeai/`.
const DECISION: &str = "explore/decision.yaml";

/// Remove `.explore-prototype/` after a kill or a promote decision.
pub fn prune(ctx: &mut Ctx, id: &str) -> Result<Outcome, CliError> {
    if !crate::doc::ids::is_id(id) {
        return Err(CliError::new(
            "DFA-E013",
            format!("'{id}' is not an ID; expected PREFIX-nnn with three digits"),
        ));
    }

    let decision = read_decision(ctx)?;
    let dir = ctx.root.join(PROTOTYPE);
    let files = count_files(&dir);
    let remove = matches!(decision.as_str(), "kill" | "promote");

    let removed = if remove && dir.exists() {
        remove_tree(&dir)?;
        true
    } else {
        false
    };

    let human = if removed {
        vec![format!(
            "Pruned    {PROTOTYPE} ({files} files) after a {decision} decision"
        )]
    } else if remove {
        vec![format!(
            "Pruned    {PROTOTYPE} is absent; nothing to remove after a {decision} decision"
        )]
    } else {
        vec![format!(
            "Kept      {PROTOTYPE} ({files} files) after a {decision} decision"
        )]
    };

    Ok(Outcome {
        human,
        data: serde_json::json!({
            "id": id,
            "decision": decision,
            "removed": removed,
            "path": PROTOTYPE,
            "files": files,
        }),
        degraded: ctx.degraded(),
        project: ctx.root.display().to_string(),
        ..Default::default()
    })
}

/// The `decision` field of `explore/decision.yaml`.
fn read_decision(ctx: &Ctx) -> Result<String, CliError> {
    // One implementation of the read, in `doc`, so this command and the gate
    // cannot disagree about whether a malformed decision file is absent.
    // `prune` needs the document to exist, so absent is `DFA-E200` here; the
    // gate reads the same `None` as "no explore predecessor".
    let Some(value) = crate::doc::read_decision(&ctx.root)? else {
        return Err(CliError::at(
            "DFA-E200",
            format!("{DECISION} not found"),
            DECISION.to_string(),
        ));
    };
    Ok(value
        .get("decision")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string())
}

/// Every regular file under `dir`, or zero when it does not exist.
fn count_files(dir: &Path) -> usize {
    if !dir.exists() {
        return 0;
    }
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .count()
}

/// Remove the tree, reporting the first path that resisted as `DFA-E262`.
fn remove_tree(dir: &Path) -> Result<(), CliError> {
    std::fs::remove_dir_all(dir).map_err(|e| {
        CliError::at(
            "DFA-E262",
            format!("removing {} failed: {e}", dir.display()),
            dir.display().to_string(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_absent_directory_counts_zero_files() {
        let dir = tempfile::tempdir().expect("temp");
        assert_eq!(count_files(&dir.path().join("nowhere")), 0);
    }

    #[test]
    fn the_count_is_over_files_and_not_directories() {
        let dir = tempfile::tempdir().expect("temp");
        let nested = dir.path().join("a").join("b");
        std::fs::create_dir_all(&nested).expect("mkdir");
        std::fs::write(nested.join("f.txt"), "x").expect("write");
        std::fs::write(dir.path().join("g.txt"), "x").expect("write");
        assert_eq!(count_files(dir.path()), 2);
    }
}
