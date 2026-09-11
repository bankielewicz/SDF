//! `gate require` and `gate check`.

use crate::cli::GateCheckArgs;
use crate::ctx::Ctx;
use crate::errors::CliError;
use crate::{gate, state, Outcome};

/// Reject a phase name outside the enum, and an id outside the two ID shapes.
fn check_phase(phase: &str) -> Result<(), CliError> {
    if state::GATE_PHASES.contains(&phase) || phase == "design" {
        return Ok(());
    }
    Err(CliError::new(
        "DFA-E012",
        format!(
            "'{phase}' is not a phase; expected one of {}",
            state::GATE_PHASES.join(", ")
        ),
    ))
}

fn check_id(id: &str) -> Result<(), CliError> {
    let ok = crate::doc::ids::is_id(id) || crate::doc::is_version(id) || crate::doc::is_date(id);
    if ok {
        return Ok(());
    }
    Err(CliError::new(
        "DFA-E013",
        format!("'{id}' is not an ID; expected PREFIX-nnn with three digits"),
    ))
}

/// The code a failing check with no code of its own is reported under.
///
/// Every failing check's reason begins with its own `DFA-Ennn`, so this is a
/// contract violation by the engine rather than an ordinary outcome. It is
/// reported rather than dropped because a failing check that appears in
/// `checks[]` and nowhere in `errors[]` is invisible to a caller that reads
/// `errors[]` to decide what to fix. The row is its own, in the gate band at
/// exit 1: the internal band it used to borrow exits 5 and names a filesystem
/// error, which describes neither the cause nor the consequence.
const UNCODED_CHECK: &str = "DFA-E350";

/// Split a `DFA-` code off the head of a check's reason.
///
/// `Diag::at` takes a `&'static str` code, so the head of the reason is matched
/// against the compiled error table rather than borrowed out of the string.
#[doc(hidden)]
pub fn split_code_for_test(reason: &str) -> (&'static str, &str) {
    split_code(reason)
}

fn split_code(reason: &str) -> (&'static str, &str) {
    let head = reason.split_whitespace().next().unwrap_or("");
    if head.starts_with("DFA-") {
        if let Some(row) = crate::errors_table::ERROR_TABLE
            .iter()
            .find(|row| row.code == head)
        {
            return (row.code, reason[head.len()..].trim_start());
        }
    }
    (UNCODED_CHECK, reason)
}

/// Append the command that repairs a refused `gate require` to its message.
fn repair_hint(e: CliError, phase: &str, id: &str) -> CliError {
    let Some(d) = e.diag() else { return e };
    if d.code != "DFA-E321" {
        return e;
    }
    let repair = match phase {
        "discover" => format!("run /explore, then /discover {id}"),
        "constitute" => format!("run /discover {id}, then /constitute {id}"),
        "plan" => "run /constitute <IDEA-nnn>, then /plan again".to_string(),
        "build" => "run /plan <EPIC-nnn>, then /build again".to_string(),
        "verify" => format!("run /build {id}, then /verify {id}"),
        "release" => format!("run /verify on each story, then /release {id}"),
        _ => format!("run devforgeai gate check --phase {phase} --id {id}"),
    };
    CliError::at(
        "DFA-E321",
        format!("{}; to repair, {repair}", d.message),
        d.path.clone(),
    )
}

/// `devforgeai gate require <phase> <id>`
pub fn require(ctx: &mut Ctx, phase: &str, id: &str) -> Result<Outcome, CliError> {
    check_phase(phase)?;
    check_id(id)?;

    // A skill's `!` preamble aborts the whole invocation on a non-zero exit
    // and the skill body never loads, so this diagnostic is all the user sees.
    // It names the command that repairs the state as well as the defect.
    let required = gate::require(ctx, phase, id).map_err(|e| repair_hint(e, phase, id))?;
    let mut out = Outcome::data(serde_json::json!({
        "phase": required.phase,
        "id": required.id,
        "requires": required.requires,
        "subject": required.subject,
        "reports": required.reports,
        "result": required.result,
    }));
    // Human output on pass: nothing.
    out.warnings = required.warnings;
    out.project = ctx.root.display().to_string();
    out.degraded = ctx.degraded();
    Ok(out)
}

/// `devforgeai gate check --phase <phase> [--id <id>]`
pub fn check(ctx: &mut Ctx, args: &GateCheckArgs) -> Result<Outcome, CliError> {
    check_phase(&args.phase)?;

    let id = match &args.id {
        Some(v) if !v.is_empty() => v.clone(),
        _ => {
            // `--phase reflect` has no `[active]` key, so omitting `--id` there
            // is DFA-E011, and so is an empty active id.
            let s = ctx.state()?;
            let found = s.active.get(&args.phase).unwrap_or("").to_string();
            if found.is_empty() {
                return Err(CliError::new(
                    "DFA-E011",
                    format!("'gate check' requires --id for phase '{}'", args.phase),
                ));
            }
            found
        }
    };
    check_id(&id)?;

    let rep = gate::check(ctx, &args.phase, &id, args.partial, args.no_run)?;
    let degraded = rep.degraded;

    let checks: Vec<serde_json::Value> = rep
        .gate
        .checks
        .iter()
        .map(|c| {
            serde_json::json!({
                "id": c.id,
                "kind": c.kind,
                "status": c.status,
                "severity": c.severity,
                "reason": c.reason,
                "evidence": serde_json::to_value(&c.evidence).unwrap_or_default(),
            })
        })
        .collect();

    let result_text = match rep.gate.result.as_str() {
        "SEND_BACK" => "SEND BACK",
        other => other,
    };

    let mut human = vec![format!("Gate      {} · {}", args.phase, id)];
    for c in &rep.gate.checks {
        let note = if c.reason.is_empty() {
            String::new()
        } else {
            c.reason.clone()
        };
        // A value at or past its column width would otherwise run into the
        // next one: `kill-case-answered` is eighteen characters, and the line
        // read `kill-case-answeredverifier_pass`. Padding to one less than the
        // width and joining with a space keeps the columns aligned for every
        // value that fits and still separates one that does not.
        human.push(format!(
            "  {:<7} {:<17} {:<15} {}",
            c.status, c.id, c.kind, note
        ));
    }
    human.push(format!("Result    {result_text}"));
    human.push(format!(
        "Report    .devforgeai/reports/{id}-{}.yaml",
        args.phase
    ));

    let exit = if args.partial {
        // `--partial` exits 0 whatever the result.
        0
    } else {
        match rep.gate.result.as_str() {
            "PASS" => 0,
            "SEND_BACK" => 2,
            _ => 1,
        }
    };

    let coverage = rep
        .coverage
        .as_ref()
        .and_then(|c| serde_json::to_value(c).ok())
        .unwrap_or(serde_json::Value::Null);

    let mut out = Outcome::data(serde_json::json!({
        "phase": args.phase,
        "id": id,
        "result": result_text,
        "send_back_to": rep.gate.send_back_to,
        "partial": args.partial,
        "degraded": degraded,
        "report": format!(".devforgeai/reports/{id}-{}.yaml", args.phase),
        "checks": checks,
        "coverage": coverage,
        "findings": rep.findings.iter().map(|f| serde_json::json!({
            "id": f.id, "severity": f.severity, "summary": f.summary
        })).collect::<Vec<_>>(),
    }));
    out.human = human;
    out.degraded = degraded;
    // The envelope carries `errors[]` on failure as well as on success, so a
    // caller that reads it to decide what to fix is not left reading free text
    // out of `checks[].reason`. The code travels at the head of the reason;
    // a check with no code still yields a row, under the gate's own code.
    if exit != 0 {
        for c in &rep.gate.checks {
            if c.status != "fail" {
                continue;
            }
            let (code, message) = split_code(&c.reason);
            let message = if message.is_empty() {
                format!("{} ({}) failed and named no reason", c.id, c.kind)
            } else {
                format!("{} ({}): {message}", c.id, c.kind)
            };
            out.errors.push(crate::errors::Diag::at(
                code,
                message,
                format!(".devforgeai/reports/{id}-{}.yaml", args.phase),
            ));
        }
    }
    if degraded {
        // The degradation rule: the line goes to stderr whenever the flag is
        // set, in both output modes.
        out.stderr
            .push("Stack undetected; command checks skipped.".to_string());
    }
    out.project = ctx.root.display().to_string();
    out.exit = Some(exit);
    Ok(out)
}
