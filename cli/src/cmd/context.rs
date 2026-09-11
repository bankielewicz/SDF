//! `context audit`: the eight checks over the six context files and the ADRs.

use crate::audit;
use crate::ctx::Ctx;
use crate::errors::{CliError, Diag};
use crate::Outcome;

/// Run the audit and render it.
pub fn run(ctx: &mut Ctx) -> Result<Outcome, CliError> {
    let a = audit::run(ctx)?;

    let present = a.files.iter().filter(|f| f.present).count();
    let tail = if a.clean() {
        format!("{} checks passed", audit::CHECK_IDS.len())
    } else {
        let failed = a.failed();
        format!(
            "{} failed",
            failed
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let human = vec![format!(
        "context audit  {present}/{} files · {} constraints · {} anti-patterns · {tail}",
        a.files.len(),
        a.constraints,
        a.anti_patterns,
    )];

    let warnings: Vec<Diag> = a
        .findings
        .iter()
        .map(|f| Diag::at_line(f.code, f.message.clone(), f.path.clone(), f.line))
        .collect();

    Ok(Outcome {
        human,
        data: audit::to_json(&a),
        warnings,
        degraded: ctx.degraded(),
        project: ctx.root.display().to_string(),
        exit: Some(if a.clean() { 0 } else { 1 }),
        raw: None,
        stderr: Vec::new(),
        ..Default::default()
    })
}
