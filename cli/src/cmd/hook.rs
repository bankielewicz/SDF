//! The `hook` subcommand family. This module carries `install`; `hook run`,
//! the stdin dispatcher, is added beside it.

use crate::ctx::Ctx;
use crate::errors::CliError;
use crate::Outcome;

/// `devforgeai hook install`: merge the Claude settings block and write the
/// three git hook scripts.
pub fn install(ctx: &mut Ctx, args: &crate::cli::HookInstallArgs) -> Result<Outcome, CliError> {
    if args.claude_only && args.git_only {
        return Err(CliError::new(
            "DFA-E010",
            "'hook install' takes --claude-only or --git-only, not both",
        ));
    }

    let report = crate::hooks::install(&ctx.root, args.force, args.claude_only, args.git_only)?;

    let mut parts: Vec<String> = Vec::new();
    if !args.git_only {
        parts.push(format!(
            ".claude/settings.json merged ({} events)",
            report.events.len()
        ));
    }
    if !report.git_hooks.is_empty() {
        parts.push(format!("git hooks {}", report.git_hooks.join(", ")));
    }

    let mut outcome = Outcome::data(serde_json::json!({
        "settings": report.settings,
        "events": report.events,
        "git_hooks": report.git_hooks,
        "permissions": report.permissions,
        "backup": report.backup,
    }));
    outcome.human = vec![format!("Hooks  {}", parts.join("; "))];
    outcome.warnings = report.warnings;
    outcome.project = ctx.root.to_string_lossy().replace('\\', "/");
    Ok(outcome)
}

/// `devforgeai hook run <event>`: the stdin JSON dispatcher.
pub fn run(ctx: &mut Ctx, event: &str, payload: &str) -> Result<Outcome, CliError> {
    crate::hooks::run::dispatch(ctx, event, payload)
}
