//! `phase set <phase> --id <id>`, which is also the one writer of a story's
//! `status` frontmatter value.

use crate::ctx::Ctx;
use crate::doc::ids;
use crate::errors::{CliError, Diag};
use crate::{gate, project, state, ypath, Outcome};
use serde_yaml_ng::Value;

/// The two phases that run beside the seven rather than inside them.
///
/// Design and Reflect are entered and left without advancing the delivery
/// chain: the story or idea a run was opened for is still in the phase it was
/// in. Moving `[current]` would strand it, so these two record themselves in
/// `[last_cross]` and touch nothing else. The Stop that ends the turn renders
/// their block beside the current phase's and clears the table.
const CROSS_CUTTING: &[&str] = &["design", "reflect"];

/// The epic a `phase set plan` run is for, or `None` for any other phase.
///
/// Plan is the one phase entered before its own document exists: the skill
/// allocates the sprint id at step 4 and writes `stories/sprint.yaml` at step
/// 12, so `phase set plan --id SPRINT-nnn` runs against a file that is not
/// there and the sprint resolves to no epic. `--epic` supplies what the file
/// would have said.
///
/// With the file present the flag is a cross-check rather than an input: a
/// disagreement means the caller and the document name different epics, and
/// guessing which is right would either advance the wrong plan or silently
/// ignore the argument.
fn resolve_plan_epic(
    ctx: &mut Ctx,
    phase: &str,
    id: &str,
    epic: Option<&str>,
) -> Result<Option<String>, CliError> {
    let given = epic.map(str::trim).filter(|e| !e.is_empty());
    if phase != "plan" {
        if let Some(e) = given {
            return Err(CliError::new(
                "DFA-E011",
                format!("'phase set {phase}' takes no --epic; '{e}' was given"),
            ));
        }
        return Ok(None);
    }
    if let Some(e) = given {
        if !ids::split_id(e).map(|(p, _)| p == "EPIC").unwrap_or(false) {
            return Err(CliError::new(
                "DFA-E013",
                format!("'{e}' is not an ID; expected EPIC-nnn with three digits"),
            ));
        }
    }

    let rel = "stories/sprint.yaml";
    let path = ctx.doc_path(rel);
    if !path.exists() {
        let Some(e) = given else {
            return Err(CliError::at(
                "DFA-E011",
                format!("'phase set plan --id {id}' requires --epic <EPIC-nnn> until {rel} exists"),
                rel,
            ));
        };
        return Ok(Some(e.to_string()));
    }

    let Some(given) = given else {
        return Ok(None);
    };
    let text = project::read_doc(&path)?;
    let value: Value = serde_yaml_ng::from_str(&text).map_err(|e| {
        CliError::at(
            "DFA-E401",
            format!("{rel} is not valid YAML: {e}"),
            rel.to_string(),
        )
    })?;
    let recorded = value
        .get(Value::String("epic".into()))
        .and_then(Value::as_str)
        .unwrap_or("");
    if recorded != given {
        return Err(CliError::at(
            "DFA-E013",
            format!("--epic is '{given}' and {rel} records '{recorded}'"),
            rel.to_string(),
        ));
    }
    Ok(None)
}

/// `phase set design|reflect`: record the cross-cutting run, advance nothing.
fn set_cross_cutting(
    ctx: &mut Ctx,
    phase: &str,
    id: &str,
    remedy: Option<&str>,
) -> Result<Outcome, CliError> {
    if remedy.map(|r| !r.trim().is_empty()).unwrap_or(false) {
        return Err(CliError::new(
            "DFA-E011",
            format!("'phase set {phase}' takes no --remedy"),
        ));
    }

    let from = ctx.state()?.current.phase.clone();
    let now = crate::time::now_rfc3339();
    {
        let s = ctx.state_mut()?;
        s.last_cross.phase = phase.to_string();
        s.last_cross.id = id.to_string();
        s.last_cross.turn = now.clone();
    }
    ctx.store_state()?;

    let mut out = Outcome::data(serde_json::json!({
        "from": from,
        "to": phase,
        "id": id,
        "cross_cutting": true,
        "requires": "",
        "gate": "",
        "status_writes": [],
        "remedy": [],
        "at": now,
    }));
    out.human = vec![format!("Phase     {phase} · {id}  (cross-cutting)")];
    out.project = ctx.root.display().to_string();
    out.degraded = ctx.degraded();
    Ok(out)
}

/// `devforgeai phase set <phase> --id <id> [--remedy <ID,ID>]`
pub fn set(
    ctx: &mut Ctx,
    phase: &str,
    id: &str,
    remedy: Option<&str>,
    epic: Option<&str>,
) -> Result<Outcome, CliError> {
    if !state::PHASES.contains(&phase) && !CROSS_CUTTING.contains(&phase) {
        return Err(CliError::new(
            "DFA-E012",
            format!(
                "'{phase}' is not a phase; expected one of {}",
                state::PHASES.join(", ")
            ),
        ));
    }
    if CROSS_CUTTING.contains(&phase) {
        return set_cross_cutting(ctx, phase, id, remedy);
    }
    // Plan is entered before its own document exists, so the epic the gate
    // resolves against has to be given rather than read.
    let plan_epic = resolve_plan_epic(ctx, phase, id, epic)?;
    let cited: Vec<String> = remedy
        .unwrap_or("")
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();

    match phase {
        "explore" => {
            for c in &cited {
                if !ids::split_id(c).map(|(p, _)| p == "FLOW").unwrap_or(false) {
                    return Err(CliError::new(
                        "DFA-E011",
                        format!("'phase set explore' takes FLOW-nnn ids in --remedy, not '{c}'"),
                    ));
                }
            }
        }
        "constitute" => {
            if cited.len() > 1 {
                return Err(CliError::new(
                    "DFA-E011",
                    "'phase set constitute' takes one CON-nnn in --remedy",
                ));
            }
            for c in &cited {
                if !ids::split_id(c).map(|(p, _)| p == "CON").unwrap_or(false) {
                    return Err(CliError::new(
                        "DFA-E011",
                        format!("'phase set constitute' takes one CON-nnn in --remedy, not '{c}'"),
                    ));
                }
            }
        }
        _ => {
            if !cited.is_empty() {
                return Err(CliError::new(
                    "DFA-E011",
                    format!("'phase set {phase}' takes no --remedy"),
                ));
            }
        }
    }

    // The command runs the `gate require` logic first, and refuses only when a
    // predecessor gate exists for the phase and its result is not PASS. A plan
    // run whose sprint file does not exist yet is resolved against the epic
    // instead, which is the subject the constitute gate is recorded under.
    let gate_subject = plan_epic.clone().unwrap_or_else(|| id.to_string());
    let required = gate::require(ctx, phase, &gate_subject);
    let (requires, required_result) = match required {
        // A predecessor named but no report read is a vacuous pass: the loop
        // over the predecessor's reports ran zero times and `result` kept the
        // value it was initialised with. Advancing on it skips the whole chain,
        // so an empty report list under a named predecessor refuses here.
        Ok(r) if !r.requires.is_empty() && r.reports.is_empty() => {
            return Err(CliError::at(
                "DFA-E320",
                format!(
                    "phase '{phase}' needs gate PASS for {id}; the '{}' gate produced no report to read",
                    r.requires
                ),
                ".devforgeai/state.toml",
            ));
        }
        Ok(r) => (r.requires.clone(), r.result.clone()),
        Err(e) if e.code() == "DFA-E321" => {
            let msg = e.diag().map(|d| d.message.clone()).unwrap_or_default();
            let last = ctx
                .state()
                .map(|s| s.last_gate.result.clone())
                .unwrap_or_else(|_| "NOT_RUN".into());
            return Err(CliError::at(
                "DFA-E320",
                format!(
                    "phase '{phase}' needs gate PASS for {id}; the last result is {last}. {msg}"
                ),
                ".devforgeai/state.toml",
            ));
        }
        Err(e) => return Err(e),
    };

    let from = ctx.state()?.current.phase.clone();

    // The status writes, by the transition table.
    let mut status_writes: Vec<serde_json::Value> = Vec::new();
    let mut warnings: Vec<Diag> = Vec::new();
    match phase {
        "build" => write_status(
            ctx,
            &format!("stories/{id}.md"),
            "building",
            &mut status_writes,
            &mut warnings,
        )?,
        "verify" => write_status(
            ctx,
            &format!("stories/{id}.md"),
            "built",
            &mut status_writes,
            &mut warnings,
        )?,
        "release" => {
            let rel = format!("releases/{id}.yaml");
            let path = ctx.doc_path(&rel);
            match project::read_doc(&path) {
                Ok(text) => {
                    let value: Value = serde_yaml_ng::from_str(&text).unwrap_or(Value::Null);
                    let stories =
                        ypath::resolve_strings(&value, "stories[].id").unwrap_or_default();
                    for sid in stories {
                        write_status(
                            ctx,
                            &format!("stories/{sid}.md"),
                            "released",
                            &mut status_writes,
                            &mut warnings,
                        )?;
                    }
                }
                Err(_) => warnings.push(Diag::at(
                    "DFA-W210",
                    format!("status not written; {rel} not found"),
                    rel,
                )),
            }
        }
        _ => {}
    }

    let now = crate::time::now_rfc3339();
    let (timebox, remedy_timebox) = {
        let cfg = ctx.config()?;
        (cfg.explore.timebox_days, cfg.explore.remedy_timebox_days)
    };

    let s = ctx.state_mut()?;
    s.current.phase = phase.to_string();
    s.current.id = id.to_string();
    // `[active].build` is the story the current run is on, and `phase set
    // build` is how a run says which. Called from inside a registered
    // worktree it is also how switching directories switches the story: the
    // state it writes is the main checkout's, so both checkouts agree.
    s.active.set(phase, id);
    s.stop_hook.block_count = 0;
    s.stop_hook.blocked_phase = String::new();
    s.stop_hook.blocked_id = String::new();

    if let Some(e) = &plan_epic {
        s.plan.epic = e.clone();
    }

    match phase {
        "explore" => {
            if cited.is_empty() {
                s.explore.idea_id = id.to_string();
                s.explore.started_at = now.clone();
                s.explore.timebox_days = timebox;
                s.explore.remedy_flows = Vec::new();
            } else {
                // `[explore].started_at` is unchanged on a remedy.
                s.explore.remedy_flows = cited.clone();
                s.explore.remedy_started_at = now.clone();
                s.explore.remedy_timebox_days = remedy_timebox;
            }
        }
        "constitute" => {
            if let Some(c) = cited.first() {
                s.constitute.remedy_con = c.clone();
            }
        }
        _ => {}
    }
    ctx.store_state()?;

    let mut out = Outcome::data(serde_json::json!({
        "from": from,
        "to": phase,
        "id": id,
        "required": requires,
        "required_result": if requires.is_empty() { String::new() } else { required_result },
        "remedy": cited,
        "status_writes": status_writes,
    }));
    let index = state::phase_index(phase)
        .map(|i| i.to_string())
        .unwrap_or_else(|| "—".to_string());
    out.human = vec![format!(
        "Phase     {index} · {:<12} {id}",
        state::phase_name(phase)
    )];
    out.warnings = warnings;
    out.project = ctx.root.display().to_string();
    Ok(out)
}

/// Replace the `status` value in a document's frontmatter and touch no other
/// byte. A document already carrying the target value is left untouched.
fn write_status(
    ctx: &mut Ctx,
    rel: &str,
    status: &str,
    writes: &mut Vec<serde_json::Value>,
    warnings: &mut Vec<Diag>,
) -> Result<(), CliError> {
    let path = ctx.doc_path(rel);
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            warnings.push(Diag::at(
                "DFA-W210",
                format!("status not written; {rel} not found"),
                rel.to_string(),
            ));
            return Ok(());
        }
        Err(e) => return Err(CliError::io("reading", path.display(), &e)),
    };

    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let ends_with_newline = text.ends_with('\n');

    // The frontmatter block of a Markdown document, so a `status:` in the body
    // is not touched.
    let close = lines
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, l)| l.trim_end() == "---")
        .map(|(i, _)| i)
        .unwrap_or(lines.len());

    let mut changed = false;
    for line in lines.iter_mut().take(close) {
        if let Some(rest) = line.strip_prefix("status:") {
            if rest.trim() == status {
                return Ok(());
            }
            *line = format!("status: {status}");
            changed = true;
            break;
        }
    }
    if !changed {
        return Ok(());
    }

    let mut out = lines.join("\n");
    if ends_with_newline {
        out.push('\n');
    }
    project::atomic_write(&path, out.as_bytes())?;
    writes.push(serde_json::json!({
        "path": format!(".devforgeai/{rel}"),
        "status": status,
    }));
    Ok(())
}
