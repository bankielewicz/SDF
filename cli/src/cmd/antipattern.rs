//! `antipattern scan`: apply the anti-pattern index to a candidate set.

use crate::antipattern::{self, Match};
use crate::ctx::Ctx;
use crate::errors::{CliError, Diag};
use crate::{story, Outcome};
use std::path::Path;

/// Scan a candidate set and report every match at or above `min_severity`.
pub fn scan(
    ctx: &mut Ctx,
    id: Option<&str>,
    paths: Option<&str>,
    min_severity: &str,
) -> Result<Outcome, CliError> {
    if !antipattern::SEVERITIES.contains(&min_severity) {
        return Err(CliError::new(
            "DFA-E013",
            format!(
                "'{min_severity}' is not a severity; expected one of {}",
                antipattern::SEVERITIES.join(", ")
            ),
        ));
    }
    if let Some(want) = id {
        if !want.starts_with("STORY-") || !crate::doc::ids::is_id(want) {
            return Err(CliError::new(
                "DFA-E013",
                format!("'{want}' is not an ID; expected PREFIX-nnn with three digits"),
            ));
        }
    }

    let root = ctx.root.clone();
    let candidates = candidates(ctx, id, paths)?;
    let rules = antipattern::rules(&root)?;
    let applied = rules
        .iter()
        .filter(|r| antipattern::at_or_above(&r.severity, min_severity))
        .count();
    let matches = antipattern::scan(&rules, &candidates, min_severity, &root);

    let warnings: Vec<Diag> = matches
        .iter()
        .map(|m| {
            Diag::at_line(
                "DFA-E270",
                format!("{} matched {}:{}: {}", m.id, m.path, m.line, m.text),
                m.path.clone(),
                m.line,
            )
        })
        .collect();

    Ok(Outcome {
        human: vec![format!(
            "antipattern scan  {} files · {applied} rules · {} {}",
            candidates.len(),
            matches.len(),
            if matches.len() == 1 {
                "match"
            } else {
                "matches"
            }
        )],
        data: serde_json::json!({
            "scanned": candidates.len(),
            "rules": applied,
            "matches": matches.iter().map(match_json).collect::<Vec<_>>(),
        }),
        warnings,
        degraded: ctx.degraded(),
        project: root.display().to_string(),
        exit: Some(if matches.is_empty() { 0 } else { 1 }),
        raw: None,
        stderr: Vec::new(),
        ..Default::default()
    })
}

fn match_json(m: &Match) -> serde_json::Value {
    serde_json::json!({
        "id": m.id,
        "severity": m.severity,
        "path": m.path,
        "line": m.line,
        "text": m.text,
    })
}

/// `--paths` when given, the story's `## Files` set when `--id` is given, and
/// every file under `[[stack]].source_roots` otherwise.
pub fn candidates(
    ctx: &mut Ctx,
    id: Option<&str>,
    paths: Option<&str>,
) -> Result<Vec<String>, CliError> {
    let root = ctx.root.clone();
    if let Some(list) = paths {
        let mut out: Vec<String> = list
            .split(',')
            .map(story::normalise_path)
            .filter(|p| !p.is_empty())
            .collect();
        out.sort();
        out.dedup();
        return Ok(out);
    }
    if let Some(want) = id {
        let s = story::load(&root, want)?;
        let mut out: Vec<String> = s.files.into_iter().map(|f| f.path).collect();
        out.sort();
        out.dedup();
        return Ok(out);
    }

    let roots: Vec<String> = ctx
        .config()?
        .stack
        .iter()
        .flat_map(|s| s.source_roots.clone())
        .collect();
    Ok(walk(&root, &roots))
}

/// Every file under the source roots, project-relative with forward slashes.
fn walk(root: &Path, source_roots: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for dir in source_roots {
        let base = root.join(dir);
        if !base.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&base)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            out.push(crate::project::rel_display(root, entry.path()));
        }
    }
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_walk_is_sorted_and_project_relative() {
        let dir = tempfile::tempdir().expect("temp");
        std::fs::create_dir_all(dir.path().join("src").join("b")).expect("mkdir");
        std::fs::write(dir.path().join("src").join("z.rs"), "x").expect("write");
        std::fs::write(dir.path().join("src").join("b").join("a.rs"), "x").expect("write");
        assert_eq!(
            walk(dir.path(), &["src".to_string()]),
            vec!["src/b/a.rs".to_string(), "src/z.rs".to_string()]
        );
    }

    #[test]
    fn an_absent_source_root_contributes_nothing() {
        let dir = tempfile::tempdir().expect("temp");
        assert!(walk(dir.path(), &["nowhere".to_string()]).is_empty());
    }
}
