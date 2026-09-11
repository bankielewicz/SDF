//! `stack detect`: write `[[stack]]`, `[frontend].globs`, and `degraded` into
//! `.devforgeai/config.toml`, leaving every other table at its previous value.

use crate::config::{self, Config};
use crate::ctx::Ctx;
use crate::errors::CliError;
use crate::{project, Outcome};
use std::path::Path;

/// The repo-relative path the `--json` `data` object names.
const CONFIG_REL: &str = ".devforgeai/config.toml";

/// The two globs a `node` stack adds to `[frontend].globs`.
const NODE_GLOBS: &[&str] = &["**/*.ts", "**/*.js"];

/// Detect the stack and write `config.toml`, or, under `dry_run`, print the
/// result and write nothing. Exit 0 in every case, the undetected one included.
pub fn detect(ctx: &mut Ctx, dry_run: bool) -> Result<Outcome, CliError> {
    let detected = crate::stack::detect(&ctx.root);
    let path = project::dot(&ctx.root).join(project::CONFIG);
    let previous = read_previous(&path)?;
    let mut cfg = match &previous {
        Some((_, cfg)) => cfg.clone(),
        None => Config::default(),
    };

    // The values detection owns. Everything else in `cfg` arrived either from
    // the previous file or from `Config::default()`.
    cfg.schema = config::SCHEMA.to_string();
    cfg.cli_version = crate::VERSION.to_string();
    cfg.frontend.globs = merge_globs(&cfg.frontend.globs, &frontend_globs(&detected));
    cfg.stack = merge_stacks(&cfg.stack, &detected);
    // `degraded` turns off every command-running check, so it follows the
    // table that results rather than the detection alone: a project whose
    // commands are written by hand is not degraded merely because the detector
    // recognised nothing.
    cfg.degraded = cfg.stack.is_empty();

    // The hook runs this at every session start, so an unchanged result must
    // leave the file alone: rewriting it churns the mtime that the Stop-time
    // scan and any file watcher read, and a `generated_at` that moves on every
    // session says a detection happened when none did.
    let rendered_with_previous_stamp = {
        let mut probe = cfg.clone();
        probe.generated_at = previous
            .as_ref()
            .map(|(_, p)| p.generated_at.clone())
            .unwrap_or_default();
        config::to_toml(&probe)?
    };
    let changed = match &previous {
        Some((text, _)) => {
            without_generated_at(text) != without_generated_at(&rendered_with_previous_stamp)
        }
        None => true,
    };

    if changed {
        cfg.generated_at = crate::time::now_rfc3339();
    } else if let Some((_, p)) = &previous {
        cfg.generated_at = p.generated_at.clone();
    }

    let written = !dry_run && changed;
    if written {
        let text = config::to_toml(&cfg)?;
        project::atomic_write(&path, text.as_bytes())?;
    }

    let stacks_json = serde_json::to_value(&cfg.stack).map_err(|e| {
        CliError::new(
            "DFA-E900",
            format!("rendering the stack tables failed: {e}"),
        )
    })?;

    let mut human: Vec<String> = cfg.stack.iter().map(human_line).collect();
    human.push(format!("degraded: {}", cfg.degraded));
    if cfg.degraded {
        // Deviation: the spec puts this line on stderr. `Outcome` carries no
        // stderr channel and no code fits the warning table, so it rides along
        // as the last human line. Revisit when the dispatcher gains one.
        human.push("Stack undetected; command checks skipped.".to_string());
    }

    Ok(Outcome {
        human,
        data: serde_json::json!({
            "stacks": stacks_json,
            "degraded": cfg.degraded,
            "written": written,
            "path": CONFIG_REL,
        }),
        warnings: Vec::new(),
        degraded: cfg.degraded,
        project: ctx.root.display().to_string(),
        exit: None,
        raw: None,
        stderr: Vec::new(),
        ..Default::default()
    })
}

/// The previous `config.toml` as both its bytes and its parse, or `None`.
fn read_previous(path: &Path) -> Result<Option<(String, Config)>, CliError> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(CliError::io("reading", path.display(), &e)),
    };
    let cfg: Config = toml::from_str(&text).map_err(|e| {
        CliError::at(
            "DFA-E104",
            format!("config.toml is not valid TOML: {e}"),
            CONFIG_REL,
        )
    })?;
    Ok(Some((text, cfg)))
}

/// A rendered config with the `generated_at` line removed, for comparing two
/// renderings by everything except when they were made.
fn without_generated_at(text: &str) -> String {
    text.lines()
        .filter(|l| !l.trim_start().starts_with("generated_at"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The stack table a detection leaves behind.
///
/// A `manual` entry survives in position, untouched: it is the only record of
/// a command the detector cannot infer, and losing it turns every
/// command-running gate check into a skip. A `detected` entry is replaced by
/// the fresh detection of the same id, or dropped when that ecosystem's
/// markers are gone. Newly detected ids are appended, unless a manual entry
/// already claims that id — where the two disagree, the human's entry stands.
fn merge_stacks(existing: &[config::Stack], detected: &[config::Stack]) -> Vec<config::Stack> {
    let mut out: Vec<config::Stack> = Vec::new();
    for e in existing {
        if !e.is_detected() {
            out.push(e.clone());
            continue;
        }
        if let Some(fresh) = detected.iter().find(|d| d.id == e.id) {
            out.push(fresh.clone());
        }
    }
    for d in detected {
        if !out.iter().any(|o| o.id == d.id) {
            out.push(d.clone());
        }
    }
    out
}

/// `[frontend].globs`: everything the file already held, plus anything
/// detection adds that is not there yet.
///
/// The globs carry no per-entry source, so the union is the only rule that
/// cannot lose a hand-written pattern.
fn merge_globs(existing: &[String], detected: &[String]) -> Vec<String> {
    let mut out: Vec<String> = existing.to_vec();
    for g in detected {
        if !out.iter().any(|have| have == g) {
            out.push(g.clone());
        }
    }
    out
}

/// `[frontend].globs`: the default list, extended for a `node` stack with the
/// TypeScript and JavaScript globs.
fn frontend_globs(stacks: &[config::Stack]) -> Vec<String> {
    let mut globs = config::Frontend::default().globs;
    if stacks.iter().any(|s| s.id == "node") {
        for g in NODE_GLOBS {
            if !globs.iter().any(|have| have == g) {
                globs.push((*g).to_string());
            }
        }
    }
    globs
}

/// One human output line: the id and the package manager in eight-column
/// fields, then the test command, the coverage format, and the lint command.
fn human_line(s: &config::Stack) -> String {
    format!(
        "{:<8}{:<8}test: {}   coverage: {}   lint: {}",
        s.id, s.package_manager, s.test_command, s.coverage_format, s.lint_command
    )
}
