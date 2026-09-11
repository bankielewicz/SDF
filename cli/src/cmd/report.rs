//! `report show`, `report ingest`, `report note`.

use crate::cli::{ReportIngestArgs, ReportNoteArgs};
use crate::ctx::Ctx;
use crate::errors::{CliError, Diag};
use crate::report::{self, Report, VerifierBlock};
use crate::{project, state, Outcome};
use serde_yaml_ng::Value;

fn check_phase(phase: &str) -> Result<(), CliError> {
    if state::GATE_PHASES.contains(&phase) || phase == "design" {
        return Ok(());
    }
    Err(CliError::new(
        "DFA-E012",
        format!(
            "'{phase}' is not a phase; expected one of {}, design",
            state::GATE_PHASES.join(", ")
        ),
    ))
}

/// `devforgeai report show <id> <phase> [--check <check-id>]`
pub fn show(
    ctx: &mut Ctx,
    id: &str,
    phase: &str,
    check: Option<&str>,
) -> Result<Outcome, CliError> {
    check_phase(phase)?;
    let path = report::report_path(&ctx.root, id, phase);
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(CliError::at(
                "DFA-E400",
                format!(".devforgeai/reports/{id}-{phase}.yaml not found"),
                ctx.rel(&path),
            ))
        }
        Err(e) => return Err(CliError::io("reading", path.display(), &e)),
    };
    let rep = report::parse(&text, &path)?;

    if let Some(want) = check {
        let Some(entry) = rep.gate.checks.iter().find(|c| c.id == want) else {
            return Err(CliError::at(
                "DFA-E400",
                format!("report {} has no check '{want}'", ctx.rel(&path)),
                ctx.rel(&path),
            ));
        };
        let one = serde_json::to_value(entry)
            .map_err(|e| CliError::new("DFA-E900", format!("serialising the check failed: {e}")))?;
        let mut out = Outcome::data(one);
        out.human = vec![format!(
            "  {:<8}{:<18}{:<16}{}",
            entry.status, entry.id, entry.kind, entry.reason
        )];
        out.project = ctx.root.display().to_string();
        return Ok(out);
    }

    let data = serde_json::to_value(&rep)
        .map_err(|e| CliError::new("DFA-E900", format!("serialising the report failed: {e}")))?;
    let mut human = vec![format!("Gate      {phase} · {id}")];
    for c in &rep.gate.checks {
        human.push(format!(
            "  {:<8}{:<18}{:<16}{}",
            c.status, c.id, c.kind, c.reason
        ));
    }
    human.push(format!("Result    {}", rep.gate.result));
    if let Some(cov) = &rep.coverage {
        if let Some(overall) = cov.get("overall").and_then(Value::as_f64) {
            human.push(format!("Coverage  {overall}% overall"));
        }
    }
    if let Some(vs) = &rep.verifiers {
        for (_, block) in vs {
            let name = block.get("subagent").and_then(Value::as_str).unwrap_or("");
            let passed = block.get("passed").and_then(Value::as_i64).unwrap_or(0);
            let total = block.get("total").and_then(Value::as_i64).unwrap_or(0);
            let unit = block
                .get("unit")
                .and_then(Value::as_str)
                .unwrap_or("checks");
            human.push(format!("Verified  {name} · {passed}/{total} {unit}"));
        }
    }

    let mut out = Outcome::data(data);
    out.human = human;
    out.project = ctx.root.display().to_string();
    Ok(out)
}

/// The body of a single Markdown code fence, or the text unchanged.
///
/// A verifier is asked for one JSON object and nothing else, and models very
/// often deliver exactly that inside a ```json fence. Refusing it cost five
/// `DFA-E410` blocks in one run for output that was correct in every way that
/// matters. The fence is a transport wrapper, so it is unwrapped here, at the
/// edge, rather than in the envelope parser.
///
/// Only a whole-input fence is unwrapped. Prose before or after it is still
/// refused: that is a subagent saying something other than the envelope, which
/// is the case the check exists for.
fn unfence(text: &str) -> &str {
    let t = text.trim();
    let Some(rest) = t.strip_prefix("```") else {
        return t;
    };
    let Some(inner) = rest.strip_suffix("```") else {
        // An opening fence with no closing one is not a fenced object.
        return t;
    };
    // The opening fence may carry a language tag on the same line.
    let body = match inner.split_once('\n') {
        Some((tag, body)) if tag.trim().chars().all(|c| c.is_ascii_alphanumeric()) => body,
        _ => inner,
    };
    body.trim()
}

/// `devforgeai report ingest <subagent> <source>`
pub fn ingest(
    ctx: &mut Ctx,
    args: &ReportIngestArgs,
    stdin: Option<String>,
) -> Result<Outcome, CliError> {
    // A name absent from the registry is a no-op with exit 0.
    let registered = ctx.config()?.verifier_by_name(&args.subagent).cloned();
    let Some(v) = registered else {
        let mut out = Outcome::data(serde_json::json!({
            "subagent": args.subagent, "status": "unregistered"
        }));
        out.warnings.push(Diag::new(
            "DFA-W411",
            format!(
                "subagent '{}' is not in config.toml [[verifier]]; nothing ingested",
                args.subagent
            ),
        ));
        out.project = ctx.root.display().to_string();
        return Ok(out);
    };

    let phase = args.phase.clone().unwrap_or_else(|| v.phase.clone());
    check_phase(&phase)?;

    let id = match &args.id {
        Some(i) if !i.is_empty() => i.clone(),
        _ => ctx.state()?.active.get(&phase).unwrap_or("").to_string(),
    };
    if id.is_empty() {
        let mut out = Outcome::data(serde_json::json!({
            "subagent": args.subagent, "status": "no_active_id"
        }));
        out.warnings.push(Diag::new(
            "DFA-E412",
            format!("state.toml has no active id for phase '{phase}'; nothing ingested"),
        ));
        out.project = ctx.root.display().to_string();
        return Ok(out);
    }

    let source_text = if args.source == "-" {
        stdin.unwrap_or_default()
    } else {
        let p = std::path::Path::new(&args.source);
        let full = if p.is_absolute() {
            p.to_path_buf()
        } else {
            ctx.root.join(p)
        };
        project::read_doc(&full)?
    };

    let path = report::report_path(&ctx.root, &id, &phase);
    let mut rep = match report::load(&path) {
        Ok(r) => r,
        Err(e) if e.code() == "DFA-E400" => Report::skeleton(&id, &phase),
        Err(e) => return Err(e),
    };

    let now = crate::time::now_rfc3339();
    let mut warnings: Vec<Diag> = Vec::new();

    let (block, findings) = match report::parse_verifier(unfence(&source_text), &args.subagent) {
        Ok(parsed) => {
            let block = VerifierBlock {
                subagent: args.subagent.clone(),
                ingested_at: now.clone(),
                passed: parsed.passed,
                total: parsed.total,
                unit: if parsed.unit.is_empty() {
                    v.unit.clone()
                } else {
                    parsed.unit.clone()
                },
                findings: parsed.findings.clone(),
                // The agent's own fields, verbatim: a `report_metric` check
                // reads into them by path, so dropping them here left every
                // metric over a verifier field resolving to nothing.
                payload: parsed.payload.clone(),
                status: None,
            };
            (block, parsed.findings)
        }
        Err(e) => {
            // Unparsable output is DFA-E410 on stderr, exit 0 from the hook,
            // and the block is written with status: unparsed.
            if let Some(d) = e.diag() {
                warnings.push(d.clone());
            }
            (
                VerifierBlock {
                    subagent: args.subagent.clone(),
                    ingested_at: now.clone(),
                    passed: 0,
                    total: 0,
                    unit: v.unit.clone(),
                    findings: Vec::new(),
                    payload: serde_json::Value::Null,
                    status: Some("unparsed".to_string()),
                },
                Vec::new(),
            )
        }
    };

    rep.set_verifier_block(&v.report_field, &block);
    rep.merge_findings(&findings);
    rep.finished_at = now;
    report::write(&path, &rep)?;

    let mut out = Outcome::data(serde_json::json!({
        "subagent": args.subagent,
        "report": format!(".devforgeai/reports/{id}-{phase}.yaml"),
        "field": v.report_field,
        "passed": block.passed,
        "total": block.total,
        "findings": block.findings.len(),
        "status": block.status.clone().unwrap_or_else(|| "ingested".to_string()),
    }));
    out.human = vec![format!(
        "Ingested  {} · {}/{} {} -> .devforgeai/reports/{id}-{phase}.yaml",
        args.subagent, block.passed, block.total, block.unit
    )];
    out.warnings = warnings;
    out.project = ctx.root.display().to_string();
    // Exit 0 in every case, so SubagentStop does not block.
    Ok(out)
}

/// The closed per-phase `--key` set: the `build` phase accepts the single value
/// `build`, and every other phase accepts none.
fn note_keys(phase: &str) -> &'static [&'static str] {
    match phase {
        "build" => &["build"],
        _ => &[],
    }
}

/// `devforgeai report note <id> <phase> --key <key> --file <path>`
pub fn note(ctx: &mut Ctx, args: &ReportNoteArgs) -> Result<Outcome, CliError> {
    check_phase(&args.phase)?;
    if !note_keys(&args.phase).contains(&args.key.as_str()) {
        return Err(CliError::new(
            "DFA-E013",
            format!(
                "'{}' is not a note key for phase '{}'",
                args.key, args.phase
            ),
        ));
    }

    let file = if args.file.is_absolute() {
        args.file.clone()
    } else {
        ctx.root.join(&args.file)
    };
    let text = match std::fs::read_to_string(&file) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(CliError::at(
                "DFA-E400",
                format!("{} not found", ctx.rel(&file)),
                ctx.rel(&file),
            ))
        }
        Err(e) => return Err(CliError::io("reading", file.display(), &e)),
    };

    let value: Value = serde_yaml_ng::from_str(&text).map_err(|e| {
        CliError::at(
            "DFA-E401",
            format!("{} is not valid YAML: {e}", ctx.rel(&file)),
            ctx.rel(&file),
        )
    })?;
    let Value::Mapping(mut mapping) = value else {
        return Err(CliError::at(
            "DFA-E413",
            format!(
                "{} key 'root' is not a mapping, expected a mapping; nothing written",
                ctx.rel(&file)
            ),
            ctx.rel(&file),
        ));
    };

    // The `devforgeai/build-note/1` schema.
    let schema = mapping
        .get(Value::String("schema".into()))
        .and_then(Value::as_str)
        .unwrap_or("");
    let expected = format!("devforgeai/{}-note/1", args.key);
    if schema != expected {
        return Err(CliError::at(
            "DFA-E413",
            format!(
                "{} key 'schema' is '{schema}', expected {expected}; nothing written",
                ctx.rel(&file)
            ),
            ctx.rel(&file),
        ));
    }

    // Its `schema` and `id` keys are dropped; the remainder lands at the key.
    mapping.remove(Value::String("schema".into()));
    mapping.remove(Value::String("id".into()));
    let keys_written = mapping.len();

    let path = report::report_path(&ctx.root, &args.id, &args.phase);
    let mut rep = match report::load(&path) {
        Ok(r) => r,
        Err(e) if e.code() == "DFA-E400" => Report::skeleton(&args.id, &args.phase),
        Err(e) => return Err(e),
    };
    // `produced_by` stays `devforgeai-cli`: the note lands under its own key.
    rep.extra
        .insert(Value::String(args.key.clone()), Value::Mapping(mapping));
    report::write(&path, &rep)?;

    let report_rel = format!(".devforgeai/reports/{}-{}.yaml", args.id, args.phase);
    let mut out = Outcome::data(serde_json::json!({
        "id": args.id,
        "report": report_rel,
        "key": args.key,
        "keys_written": keys_written,
    }));
    out.human = vec![format!("Noted     {} -> {report_rel}", args.key)];
    out.project = ctx.root.display().to_string();
    Ok(out)
}

/// `report aggregate`: the `devforgeai/aggregate/1` object Reflect reads.
pub fn aggregate(
    ctx: &mut Ctx,
    id: Option<&str>,
    since: Option<&str>,
    session_root: Option<&std::path::Path>,
) -> Result<Outcome, CliError> {
    let a = crate::aggregate::run(ctx, id, since, session_root)?;
    Ok(Outcome {
        human: a.human,
        data: a.data,
        warnings: a.warnings,
        degraded: ctx.degraded(),
        project: ctx.root.display().to_string(),
        exit: Some(a.exit),
        raw: None,
        stderr: Vec::new(),
        ..Default::default()
    })
}
