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
    let stacks = crate::stack::detect(&ctx.root);
    let mut cfg = load_or_default(&ctx.root)?;

    // The four values detection owns. Everything else in `cfg` arrived either
    // from the previous file or from `Config::default()`.
    cfg.schema = config::SCHEMA.to_string();
    cfg.generated_at = crate::time::now_rfc3339();
    cfg.cli_version = crate::VERSION.to_string();
    cfg.degraded = stacks.is_empty();
    cfg.frontend.globs = frontend_globs(&stacks);
    cfg.stack = stacks;

    if !dry_run {
        let text = config::to_toml(&cfg)?;
        let path = project::dot(&ctx.root).join(project::CONFIG);
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
            "written": !dry_run,
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

/// The existing `config.toml`, or the defaults when the file is absent.
///
/// The file is read directly rather than through `config::load`, which treats
/// an absent file as an error and enforces the compiled coverage floors. The
/// spec's exit list for this subcommand names `DFA-E104` alone: a file that
/// does not parse stops the run, and nothing else about its contents does.
fn load_or_default(root: &Path) -> Result<Config, CliError> {
    let path = project::dot(root).join(project::CONFIG);
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Config::default()),
        Err(e) => return Err(CliError::io("reading", path.display(), &e)),
    };
    toml::from_str(&text).map_err(|e| {
        CliError::at(
            "DFA-E104",
            format!("config.toml is not valid TOML: {e}"),
            CONFIG_REL,
        )
    })
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
