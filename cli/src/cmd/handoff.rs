//! `devforgeai handoff`: render the conventions section 6 block for a phase and
//! a subject, record it in `state.toml` `[last_handoff]`, and exit 0 in every
//! rendering case, the one with no report included.

use crate::ctx::Ctx;
use crate::errors::CliError;
use crate::handoff;
use crate::report::{self, Report};
use crate::Outcome;
use serde_json::json;

/// Render the handoff block.
///
/// `phase` defaults to `[current].phase`, and `id` to `[active].<phase>`, which
/// leaves `design` and `reflect` taking the id from the flag alone. An absent
/// report renders `Gate      NOT RUN` and `Full report: none`; an unparsable
/// one is `DFA-E401`.
pub fn run(ctx: &mut Ctx, phase: Option<&str>, id: Option<&str>) -> Result<Outcome, CliError> {
    let state = ctx.state()?.clone();
    let phase = phase
        .map(str::to_string)
        .unwrap_or_else(|| state.current.phase.clone());
    let id = match id {
        Some(v) => v.to_string(),
        None => state
            .active
            .get(&phase)
            .map(str::to_string)
            .unwrap_or_default(),
    };

    let report = load_report(ctx, &id, &phase)?;
    let cfg = ctx.config().cloned().unwrap_or_default();
    let inputs = handoff::measure(ctx, &phase, &id, report.as_ref());
    let lines = handoff::render(&state, report.as_ref(), &cfg, &phase, &id, &inputs);

    let result = handoff::result_of(report.as_ref(), &inputs);
    let (next, then) = handoff::next_then(&state, report.as_ref(), &phase, &id, &inputs);
    // The `Then` line the block dropped to fit the cap is not part of the
    // rendered block, so the `data` object follows the lines.
    let then = then.filter(|t| {
        lines
            .iter()
            .any(|l| l.starts_with("Then") && l.ends_with(t))
    });

    {
        let now = crate::time::now_rfc3339();
        let s = ctx.state_mut()?;
        s.last_handoff.rendered_at = now.clone();
        s.last_handoff.phase = phase.clone();
        s.last_handoff.id = id.clone();
        s.last_handoff.lines = lines.clone();
        // A cross-cutting run leaves `[current].phase` where it found it, so
        // the Stop that ends the turn owes two blocks. Recording the run here,
        // at the call site the skill makes, is what tells that Stop so. The
        // Stop clears the table after it renders the pair.
        if phase == "design" || phase == "reflect" {
            s.last_cross.phase = phase.clone();
            s.last_cross.id = id.clone();
            s.last_cross.turn = now;
        }
    }
    ctx.store_state()?;

    Ok(Outcome {
        human: lines.clone(),
        data: json!({
            "lines": lines,
            "phase": phase,
            "id": id,
            "result": result.token(),
            "next": next,
            "then": then,
            "blocked": inputs.blocked,
            "report": inputs.report.clone().unwrap_or_else(|| "none".to_string()),
        }),
        degraded: ctx.degraded(),
        project: ctx.root.display().to_string(),
        ..Default::default()
    })
}

/// The report for this subject, with an absent file read as `NOT RUN` and every
/// other failure propagated.
fn load_report(ctx: &Ctx, id: &str, phase: &str) -> Result<Option<Report>, CliError> {
    let path = report::report_path(&ctx.root, id, phase);
    match report::load(&path) {
        Ok(r) => Ok(Some(r)),
        Err(e) if e.code() == "DFA-E400" => Ok(None),
        Err(e) => Err(e),
    }
}
