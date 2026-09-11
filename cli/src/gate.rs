//! The gate engine: `gate require`'s id-resolution chain and `gate check`'s
//! per-kind evaluation, report writing, and send-back resolution.

use crate::ctx::Ctx;
use crate::doc::{self, ids};
use crate::errors::{CliError, Diag};
use crate::gates::{Check, Condition, Gate};
use crate::report::{CheckEntry, Finding, Report};
use crate::{gates, project, report, state, time, ypath};
use serde_yaml_ng::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

/// The outcome of one check.
struct Outcome {
    status: &'static str,
    reason: String,
    evidence: Value,
    findings: Vec<Finding>,
    /// The report's top-level `coverage` block, set by `coverage_min` alone.
    coverage: Option<Value>,
}

impl Outcome {
    fn pass() -> Outcome {
        Outcome {
            status: "pass",
            reason: String::new(),
            evidence: Value::Mapping(Default::default()),
            findings: Vec::new(),
            coverage: None,
        }
    }
    fn skip(reason: &str) -> Outcome {
        Outcome {
            status: "skip",
            reason: reason.to_string(),
            evidence: Value::Mapping(Default::default()),
            findings: Vec::new(),
            coverage: None,
        }
    }
    fn fail(reason: impl Into<String>) -> Outcome {
        Outcome {
            status: "fail",
            reason: reason.into(),
            evidence: Value::Mapping(Default::default()),
            findings: Vec::new(),
            coverage: None,
        }
    }
    fn with_findings(mut self, f: Vec<Finding>) -> Outcome {
        self.findings = f;
        self
    }
    fn with_evidence(mut self, e: Value) -> Outcome {
        self.evidence = e;
        self
    }
    fn with_coverage(mut self, c: Value) -> Outcome {
        self.coverage = Some(c);
        self
    }
}

/// Replace `{id}` and `{phase}` in a path or document value.
pub fn substitute(text: &str, id: &str, phase: &str) -> String {
    text.replace("{id}", id).replace("{phase}", phase)
}

/// Replace the message placeholders `{id}`, `{phase}`, `{value}`, `{limit}`.
pub fn message(template: &str, id: &str, phase: &str, value: &str, limit: &str) -> String {
    template
        .replace("{id}", id)
        .replace("{phase}", phase)
        .replace("{value}", value)
        .replace("{limit}", limit)
}

/// Read a document as a YAML value: a Markdown file contributes its
/// frontmatter mapping, a YAML file its whole document, a JSON file its object.
fn load_value(ctx: &Ctx, rel: &str) -> Result<(PathBuf, Value), CliError> {
    let path = ctx.doc_path(rel);
    let text = project::read_doc(&path)?;
    let kind = doc::source_for(&path);
    let value = match kind {
        doc::frontmatter::Source::Yaml => serde_yaml_ng::from_str(&text).map_err(|e| {
            CliError::at(
                "DFA-E401",
                format!("{rel} is not valid YAML: {e}"),
                rel.to_string(),
            )
        })?,
        doc::frontmatter::Source::Json => {
            let j: serde_json::Value = serde_json::from_str(&text).map_err(|e| {
                CliError::at(
                    "DFA-E401",
                    format!("{rel} is not valid JSON: {e}"),
                    rel.to_string(),
                )
            })?;
            serde_yaml_ng::to_value(j).unwrap_or(Value::Null)
        }
        doc::frontmatter::Source::Markdown => {
            let fm = doc::frontmatter::parse(&text, kind, rel).map_err(CliError::Coded)?;
            Value::Mapping(fm.map)
        }
    };
    Ok((path, value))
}

/// Resolve `path` against `root` under the spec's path grammar, with an empty
/// sequence at a `[]` segment resolving to the empty set rather than to a
/// missing path.
///
/// `ypath::resolve` returns `Missing` once a `[]` segment has yielded zero
/// items and a further segment finds nothing to read, which makes
/// `requirements: []` indistinguishable from an absent `requirements` key.
/// Spec 280 separates the two: only a segment absent from the document is
/// `DFA-E345`.
fn resolve_at<'a>(root: &'a Value, path: &str) -> Result<Vec<&'a Value>, ypath::Missing> {
    let mut current: Vec<&Value> = vec![root];
    let mut emptied = false;

    for raw in path.split('.') {
        if raw.is_empty() {
            continue;
        }
        if emptied {
            continue;
        }
        let (name, iterate) = match raw.strip_suffix("[]") {
            Some(n) => (n, true),
            None => (raw, false),
        };

        let mut next: Vec<&Value> = Vec::new();
        let mut found_any = false;
        for value in &current {
            let child = if name.is_empty() {
                Some(*value)
            } else {
                value.get(Value::String(name.to_string()))
            };
            let Some(child) = child else {
                continue;
            };
            found_any = true;
            if iterate {
                match child.as_sequence() {
                    Some(seq) => next.extend(seq.iter()),
                    None => next.push(child),
                }
            } else {
                next.push(child);
            }
        }
        if !found_any {
            return Err(ypath::Missing {
                segment: name.to_string(),
                path: path.to_string(),
            });
        }
        if iterate && next.is_empty() {
            emptied = true;
        }
        current = next;
    }
    Ok(current)
}

/// A value rendered for a diagnostic: a null is `null` rather than the empty
/// string, so a message can tell a held null from an absent field.
fn render(v: &Value) -> String {
    match v {
        Value::Null => "null".to_string(),
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Sequence(s) => format!("a list of {}", s.len()),
        Value::Mapping(m) => format!("a mapping of {}", m.len()),
        Value::Tagged(t) => render(&t.value),
    }
}

/// Evaluate one condition inline table.
///
/// The three subjects are exclusive and `gates::condition` refuses a table that
/// names more than one, so the branch order below drops no key. A document the
/// condition names that cannot be read propagates rather than reading as
/// `false`: a condition that could not be evaluated must never silence the
/// check it guards.
fn condition_true(ctx: &mut Ctx, c: &Condition, id: &str, phase: &str) -> Result<bool, CliError> {
    if let Some(doc_path) = &c.doc_exists {
        let p = ctx.doc_path(&substitute(doc_path, id, phase));
        return Ok(p.exists());
    }
    if let Some(field) = &c.state_field {
        let s = ctx.state()?;
        let value = serde_yaml_ng::to_value(s).unwrap_or(Value::Null);
        let found = resolve_at(&value, field).map_err(|m| {
            CliError::at(
                "DFA-E345",
                format!(
                    "condition state_field '{}' does not resolve in state.toml",
                    m.path
                ),
                ".devforgeai/state.toml",
            )
        })?;
        if let Some(want_empty) = c.is_empty {
            // Only a sequence can be empty; a scalar or a mapping at the
            // location is a state defect, not an empty list.
            let empty = match found.as_slice() {
                [] => true,
                [Value::Sequence(s)] => s.is_empty(),
                [Value::Null] => true,
                _ => false,
            };
            return Ok(empty == want_empty);
        }
        let texts: Vec<String> = found.iter().filter_map(ypath::scalar).collect();
        if let Some(want) = &c.equals {
            return Ok(texts.iter().any(|v| v == want));
        }
        if let Some(set) = &c.r#in {
            return Ok(texts.iter().any(|v| set.contains(v)));
        }
        return Ok(!found.is_empty());
    }
    let Some(path) = &c.path else {
        // `gates::condition` refuses a table with no subject, so this is
        // unreachable from a parsed file.
        return Ok(false);
    };
    let field = c.field.clone().unwrap_or_default();
    let rel = substitute(path, id, phase);
    let (_, value) = load_value(ctx, &rel)?;
    let found: Vec<String> = resolve_at(&value, &field)
        .unwrap_or_default()
        .iter()
        .filter_map(ypath::scalar)
        .collect();
    if let Some(want) = &c.equals {
        return Ok(found.iter().any(|v| v == want));
    }
    if let Some(set) = &c.r#in {
        return Ok(found.iter().any(|v| set.contains(v)));
    }
    Ok(!found.is_empty())
}

/// Evaluate every gate check and write the report.
pub fn check(
    ctx: &mut Ctx,
    phase: &str,
    id: &str,
    partial: bool,
    no_run: bool,
) -> Result<Report, CliError> {
    let started_at = time::now_rfc3339();
    let degraded = ctx.config()?.degraded;
    let gate = ctx.gates()?.gate_for(phase)?.clone();

    // Keep the verifiers block of an existing report.
    let path = report::report_path(&ctx.root, id, phase);
    let mut rep = match report::load(&path) {
        Ok(r) => r,
        Err(e) if e.code() == "DFA-E400" => Report::skeleton(id, phase),
        Err(e) => return Err(e),
    };
    rep.id = id.to_string();
    rep.phase = phase.to_string();
    rep.schema = report::SCHEMA.to_string();
    rep.produced_by = report::PRODUCER.to_string();
    rep.started_at = started_at;
    rep.cli_version = crate::VERSION.to_string();
    rep.degraded = degraded;
    rep.partial = partial;
    rep.gate.checks.clear();

    const PARTIAL_KINDS: &[&str] = &[
        "tests_pass",
        "coverage_min",
        "lint_clean",
        "complexity_clean",
    ];

    let mut new_findings: Vec<Finding> = Vec::new();

    // A check kind this milestone does not evaluate is recorded in the report
    // and then reported as not implemented, so the evidence survives and the
    // caller still sees exit 5 rather than a result the binary cannot stand
    // behind.
    let mut unimplemented: Vec<String> = Vec::new();

    for c in &gate.check {
        if partial && !PARTIAL_KINDS.contains(&c.kind.as_str()) {
            continue;
        }
        let outcome = match evaluate(ctx, &gate, c, id, phase, degraded, no_run) {
            Ok(o) => o,
            Err(e) if e.code() == crate::errors_table::NOT_IMPLEMENTED => {
                unimplemented.push(c.kind.clone());
                // `skip` is the only status the report schema offers for a
                // check that produced no result; `fail` would claim the check
                // ran and lost, which this build cannot stand behind.
                Outcome::skip(report::NOT_IMPLEMENTED_REASON)
            }
            Err(e) => return Err(e),
        };
        new_findings.extend(outcome.findings.iter().cloned());
        if let Some(block) = &outcome.coverage {
            rep.coverage = Some(block.clone());
        }
        rep.gate.checks.push(CheckEntry {
            id: c.id.clone(),
            kind: c.kind.clone(),
            status: outcome.status.to_string(),
            severity: c.severity.clone(),
            reason: outcome.reason,
            evidence: outcome.evidence,
        });
    }

    rep.merge_findings(&new_findings);

    // The result is PASS when every block check is pass or skip.
    let blocking_failures: Vec<&CheckEntry> = rep
        .gate
        .checks
        .iter()
        .filter(|e| e.severity == "block" && (e.status == "fail" || e.status == "warn"))
        .collect();

    let result = if blocking_failures.is_empty() {
        "PASS"
    } else {
        let any_send_back = blocking_failures.iter().any(|e| {
            gate.check
                .iter()
                .find(|c| c.id == e.id)
                .map(|c| c.on_fail == "send_back")
                .unwrap_or(false)
        });
        if any_send_back {
            "SEND_BACK"
        } else {
            "FAIL"
        }
    };

    rep.gate.result = result.to_string();
    rep.status = match result {
        "PASS" => "pass",
        "SEND_BACK" => "send_back",
        _ => "fail",
    }
    .to_string();
    rep.gate.send_back_to = if result == "SEND_BACK" {
        resolve_send_back(&rep, &gate, phase)
    } else {
        String::new()
    };
    rep.finished_at = time::now_rfc3339();

    report::write(&path, &rep)?;

    // The report is on disk before the refusal, so the checks that did run
    // keep their evidence. State is deliberately not touched: the skips above
    // make `result` read PASS, and recording that would let `gate require`
    // admit the next phase on a gate this build never evaluated.
    if !unimplemented.is_empty() {
        // A set, not `dedup`, which only collapses adjacent repeats: a gate may
        // interleave two stubbed kinds and name one of them twice.
        let unique: BTreeSet<&str> = unimplemented.iter().map(String::as_str).collect();
        let kinds: Vec<String> = unique.iter().map(|k| format!("'{k}'")).collect();
        return Err(CliError::not_implemented(format!(
            "check kind {} (gate '{}', report {})",
            kinds.join(", "),
            phase,
            ctx.rel(&path)
        )));
    }

    if !partial {
        let failed: Vec<String> = blocking_failures.iter().map(|e| e.id.clone()).collect();
        let s = ctx.state_mut()?;
        s.last_gate.phase = phase.to_string();
        s.last_gate.id = id.to_string();
        s.last_gate.result = result.to_string();
        s.last_gate.at = rep.finished_at.clone();
        s.last_gate.report = format!(".devforgeai/reports/{id}-{phase}.yaml");
        s.last_gate.send_back_to = rep.gate.send_back_to.clone();
        s.last_gate.failed_checks = failed;
        if result == "PASS" {
            s.stop_hook.block_count = 0;
            s.stop_hook.blocked_phase = String::new();
            s.stop_hook.blocked_id = String::new();
        }
        ctx.store_state()?;
    }

    Ok(rep)
}

/// The send-back destination, resolved from the blocking findings' id prefixes
/// and falling back to the gate's static value.
fn resolve_send_back(rep: &Report, gate: &Gate, phase: &str) -> String {
    let mut phases: BTreeSet<usize> = BTreeSet::new();
    let mut spec_gap = false;
    let mut other_blocking = false;

    for f in &rep.findings {
        if f.severity != "block" {
            continue;
        }
        if phase == "verify" && f.category.as_deref() == Some("spec-gap") {
            spec_gap = true;
            continue;
        }
        other_blocking = true;
        let Some((prefix, _)) = ids::split_id(&f.id) else {
            continue;
        };
        let target = match prefix {
            "REQ" => "discover",
            "CON" | "AP" => "constitute",
            "UI" => "design",
            "AC" => "plan",
            "FIND" => "build",
            _ => continue,
        };
        // Design is not in the section 5 phase order; it sorts after the seven
        // so a phase tie never resolves to it over a real predecessor.
        match state::phase_index(target) {
            Some(i) => {
                phases.insert(i);
            }
            None => {
                phases.insert(usize::MAX);
            }
        }
    }

    // The verify gate adds one rule of its own.
    if phase == "verify" && spec_gap {
        if other_blocking {
            return "build".to_string();
        }
        return "plan".to_string();
    }

    match phases.iter().next() {
        Some(&usize::MAX) => "design".to_string(),
        Some(&i) => state::PHASES
            .get(i)
            .map(|s| s.to_string())
            .unwrap_or_else(|| gate.send_back_to.clone()),
        None => gate.send_back_to.clone(),
    }
}

#[allow(clippy::too_many_arguments)]
fn evaluate(
    ctx: &mut Ctx,
    gate: &Gate,
    c: &Check,
    id: &str,
    phase: &str,
    degraded: bool,
    no_run: bool,
) -> Result<Outcome, CliError> {
    // A document a check names that is absent, unparsable, or missing the
    // location the check reads fails that check and leaves the rest of the gate
    // to run. Aborting the gate would leave no report to read, which is the
    // artifact the phase is judged by. The same holds for a check kind that
    // delegates to another subcommand: any code in the document (`DFA-E1xx`),
    // project-data (`DFA-E2xx`), or gate (`DFA-E3xx`) bands names a defect in
    // the project, so it becomes this check's failure. Only the usage band
    // (`DFA-E0xx`), the report band (`DFA-E4xx`), the trust band (`DFA-E5xx`),
    // and the internal band (`DFA-E9xx`) still abort, since those name a defect
    // in the invocation or in the binary rather than in the work under gate.
    // The conditions are inside the conversion too: a `required_when` whose
    // document is missing is a failure, never a silent `not_required` skip.
    let evaluated = evaluate_conditioned(ctx, gate, c, id, phase, degraded, no_run);
    match evaluated {
        Err(e) if is_check_failure_code(e.code()) => {
            let message = e
                .diag()
                .map(|d| format!("{} {}", d.code, d.message))
                .unwrap_or_else(|| e.code().to_string());
            Ok(Outcome::fail(message))
        }
        other => other,
    }
}

#[allow(clippy::too_many_arguments)]
fn evaluate_conditioned(
    ctx: &mut Ctx,
    gate: &Gate,
    c: &Check,
    id: &str,
    phase: &str,
    degraded: bool,
    no_run: bool,
) -> Result<Outcome, CliError> {
    // The four condition keys, in the order the spec lists them.
    if let Some(cond) = &c.skip_when {
        if condition_true(ctx, cond, id, phase)? {
            return Ok(Outcome::skip("condition"));
        }
    }
    if let Some(cond) = &c.required_when {
        if !condition_true(ctx, cond, id, phase)? {
            return Ok(Outcome::skip("not_required"));
        }
    }
    if let Some(cond) = &c.null_when {
        if condition_true(ctx, cond, id, phase)? {
            return expect_null(ctx, gate, c, id, phase);
        }
    }
    if let Some(cond) = &c.empty_when {
        if condition_true(ctx, cond, id, phase)? {
            return expect_empty(ctx, gate, c, id, phase);
        }
    }

    // The command-running kinds under the degradation rule. `complexity_clean`
    // runs a command too, so a degraded project skips it with the others.
    const COMMAND_KINDS: &[&str] = &[
        "tests_pass",
        "coverage_min",
        "lint_clean",
        "complexity_clean",
    ];
    if degraded && COMMAND_KINDS.contains(&c.kind.as_str()) {
        return Ok(Outcome::skip("degraded"));
    }
    // `--no-run` executes no command. `coverage_min` is absent from this list
    // deliberately: under `--no-run` it takes its `source = "read"` path, which
    // runs nothing and still judges the artifact on disk, which is stronger
    // than a skip that counts as passing.
    const NO_RUN_KINDS: &[&str] = &["tests_pass", "lint_clean", "complexity_clean", "docs_cover"];
    if no_run && NO_RUN_KINDS.contains(&c.kind.as_str()) {
        return Ok(Outcome::skip("no_run"));
    }

    evaluate_kind(ctx, gate, c, id, phase, no_run)
}

/// True when a `CliError` raised inside a check kind is a defect in the project
/// rather than in the invocation, and so is recorded as that check's failure.
fn is_check_failure_code(code: &str) -> bool {
    if code == "DFA-E401" {
        return true;
    }
    let Some(rest) = code.strip_prefix("DFA-E") else {
        return false;
    };
    matches!(
        rest.as_bytes().first(),
        Some(b'1') | Some(b'2') | Some(b'3')
    )
}

fn evaluate_kind(
    ctx: &mut Ctx,
    gate: &Gate,
    c: &Check,
    id: &str,
    phase: &str,
    no_run: bool,
) -> Result<Outcome, CliError> {
    match c.kind.as_str() {
        "doc_valid" => doc_valid(ctx, c, id, phase),
        "ids_resolve" => ids_resolve(ctx, gate, c, id, phase),
        "file_exists" => file_exists(ctx, c, id, phase),
        "fields_present" => fields_present(ctx, gate, c, id, phase),
        "field_in_enum" => field_in_enum(ctx, gate, c, id, phase),
        "field_is_date" => field_is_date(ctx, gate, c, id, phase),
        "length_between" => length_between(ctx, gate, c, id, phase),
        "no_open_questions" => no_open_questions(ctx, c, id, phase),
        "set_cover" => set_cover(ctx, gate, c, id, phase),
        "row_count_between" => row_count_between(ctx, gate, c, id, phase),
        "column_matches" => column_matches(ctx, gate, c, id, phase),
        "column_contains_all" => column_contains_all(ctx, gate, c, id, phase),
        "elapsed_days_at_most" => elapsed_days(ctx, c, id, phase),
        "verifier_pass" => verifier_pass(ctx, c, id, phase),
        "report_metric" => report_metric(ctx, c, id, phase),
        "no_cycle" => no_cycle(ctx, c, id, phase),
        "yaml_cites" => yaml_cites(ctx, c, id, phase),
        "no_threshold_decrease" => no_threshold_decrease(ctx, c, id, phase),
        "release_stories" => release_stories(ctx, c, id, phase),
        "context_audit" => context_audit(ctx, c),
        "story_valid" => story_valid(ctx, c, id),
        "antipattern_clean" => antipattern_clean(ctx, c, id),
        "design_tokens" => design_tokens(ctx, c),
        "tests_pass" => tests_pass(ctx, c),
        "lint_clean" => command_check(ctx, c, Commanded::Lint),
        "complexity_clean" => command_check(ctx, c, Commanded::Complexity),
        "coverage_min" => coverage_min(ctx, c, no_run),
        "deploy_manifest" => deploy_manifest(ctx, c, id, phase),
        "docs_cover" => docs_cover(ctx, c, id, phase),
        "files_declared" => files_declared(ctx, c, id),
        other => Err(CliError::at(
            "DFA-E301",
            format!(
                "gates.toml gate '{}' check '{}' has kind '{other}'; the kinds are {}",
                gate.phase,
                c.id,
                gates::CHECK_KINDS.join(", ")
            ),
            ".devforgeai/gates.toml",
        )),
    }
}

fn subject_path(gate: &Gate, c: &Check, id: &str, phase: &str) -> String {
    let raw = c.str_key("path", &gate.document);
    substitute(&raw, id, phase)
}

/// The `null_when` rule: the check's own `path` and `field` are expected to
/// hold null or to be absent. The document is `subject_path`, so a gate-level
/// `document` supplies it on every kind, and a document that cannot be read is
/// a failure rather than a free pass.
fn expect_null(
    ctx: &mut Ctx,
    gate: &Gate,
    c: &Check,
    id: &str,
    phase: &str,
) -> Result<Outcome, CliError> {
    let rel = subject_path(gate, c, id, phase);
    let field = c.str_key("field", "");
    let (_, value) = load_value(ctx, &rel)?;
    match resolve_at(&value, &field) {
        // An absent field satisfies the rule.
        Err(_) => Ok(Outcome::pass()),
        Ok(found) => {
            if found.iter().all(|v| matches!(v, Value::Null)) {
                Ok(Outcome::pass())
            } else {
                let held = found.first().map(|v| render(v)).unwrap_or_default();
                Ok(Outcome::fail(format!(
                    "DFA-E332 {rel} '{field}' holds '{held}' where null was expected"
                )))
            }
        }
    }
}

/// The `empty_when` rule: the check's own `path` and `field` are expected to
/// hold an empty list.
fn expect_empty(
    ctx: &mut Ctx,
    gate: &Gate,
    c: &Check,
    id: &str,
    phase: &str,
) -> Result<Outcome, CliError> {
    let rel = subject_path(gate, c, id, phase);
    let field = c.str_key("field", "");
    let (_, value) = load_value(ctx, &rel)?;
    match resolve_at(&value, &field) {
        Err(_) => Ok(Outcome::pass()),
        Ok(found) => {
            let empty = found.iter().all(|v| match v {
                Value::Null => true,
                Value::Sequence(s) => s.is_empty(),
                _ => false,
            });
            if empty {
                Ok(Outcome::pass())
            } else {
                Ok(Outcome::fail(format!(
                    "DFA-E331 {rel} '{field}' holds entries where an empty list was expected"
                )))
            }
        }
    }
}

/// Expand a list of document patterns into concrete paths under the project.
fn expand_docs(ctx: &Ctx, patterns: &[String], id: &str, phase: &str) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    for raw in patterns {
        let rel = substitute(raw, id, phase);
        if !rel.contains('*') && !rel.contains('?') {
            out.push(ctx.doc_path(&rel));
            continue;
        }
        let Ok(glob) = globset::Glob::new(&rel) else {
            continue;
        };
        let matcher = glob.compile_matcher();
        // Spec 245: a relative path resolves against `.devforgeai/`, except one
        // beginning with `.`, which resolves against the project root. A glob
        // follows the same rule as a literal path, so the two branches agree.
        if rel.starts_with('.') {
            for entry in walkdir::WalkDir::new(&ctx.root)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
            {
                let candidate = project::rel_display(&ctx.root, entry.path());
                if matcher.is_match(&candidate) {
                    out.push(entry.path().to_path_buf());
                }
            }
            continue;
        }
        for p in ids::document_paths(&ctx.root) {
            let candidate = project::rel_display(&ctx.dot(), &p);
            if matcher.is_match(&candidate) {
                out.push(p);
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

fn doc_valid(ctx: &mut Ctx, c: &Check, id: &str, phase: &str) -> Result<Outcome, CliError> {
    let patterns = c.arr_key("docs", &[]);
    let paths = expand_docs(ctx, &patterns, id, phase);
    // A `doc_valid` that validates nothing is a gates.toml defect, not a pass:
    // the evidence `documents: 0` is otherwise indistinguishable from an
    // all-green run, and this kind is a compiled minimum for four phases.
    if paths.is_empty() {
        return Err(CliError::at(
            "DFA-E345",
            format!(
                "gates.toml check '{}': docs {:?} matched no document",
                c.id, patterns
            ),
            ".devforgeai/gates.toml",
        ));
    }
    let index = ids::build(&ctx.root);
    let dctx = doc::Ctx {
        root: &ctx.root,
        index: &index,
        frontmatter_only: false,
        stdin_content: None,
    };

    let mut checked = 0usize;
    let mut failed: Vec<String> = Vec::new();
    let mut findings: Vec<Finding> = Vec::new();
    for path in &paths {
        let rel = ctx.rel(path);
        // A path the doc-type table does not cover cannot be validated, so
        // naming it in `docs` is a gates.toml defect rather than a pass.
        if doc::doc_type_for(&rel).is_none() {
            return Err(CliError::at(
                "DFA-E200",
                format!(
                    "gates.toml check '{}': {rel} matches no doc-type row, so 'doc validate' has no rule for it",
                    c.id
                ),
                rel,
            ));
        }
        checked += 1;
        let d = doc::validate(path, &dctx)?;
        if !d.valid() {
            let first = &d.errors[0];
            let summary = format!(
                "{rel} failed doc validate: {} {}",
                first.code, first.message
            );
            failed.push(summary.clone());
            findings.push(Finding {
                id: d.id.clone(),
                severity: "block".into(),
                summary: cut(&summary, 90),
                evidence: String::new(),
                category: None,
            });
        }
    }
    if !failed.is_empty() {
        // Every invalid document is named, so the report states the whole of
        // the work remaining rather than the first of it.
        return Ok(Outcome::fail(format!(
            "DFA-E326 {} of {checked} documents failed doc validate: {}",
            failed.len(),
            failed.join("; ")
        ))
        .with_findings(findings)
        .with_evidence(map(&[("documents", Value::from(checked as i64))])));
    }
    Ok(Outcome::pass().with_evidence(map(&[("documents", Value::from(checked as i64))])))
}

fn ids_resolve(
    ctx: &mut Ctx,
    gate: &Gate,
    c: &Check,
    id: &str,
    phase: &str,
) -> Result<Outcome, CliError> {
    let from = c.str_key("from", "");
    let to = c.str_key("to", "");

    // `gates::parse_check` refuses `from` without `to` and either with
    // `prefixes`, so reaching this branch means both are set.
    if !from.is_empty() || !to.is_empty() {
        let rel = subject_path(gate, c, id, phase);
        let (_, value) = load_value(ctx, &rel)?;
        // A null at either end is a present value, not an absent one: dropping
        // it made a document whose every `actor:` is null pass, and made a null
        // at `to` empty the target set and fail everything.
        let from_values: Vec<String> = resolve_at(&value, &from)
            .map_err(|m| missing_path(&c.id, &m.path, &rel))?
            .iter()
            .map(|v| render(v))
            .collect();
        let to_values: BTreeSet<String> = resolve_at(&value, &to)
            .map_err(|m| missing_path(&c.id, &m.path, &rel))?
            .iter()
            .filter(|v| !matches!(v, Value::Null))
            .map(|v| render(v))
            .collect();
        let unresolved: Vec<String> = from_values
            .into_iter()
            .filter(|v| !to_values.contains(v))
            .collect();
        if unresolved.is_empty() {
            return Ok(Outcome::pass());
        }
        let first: Vec<String> = unresolved.iter().take(5).cloned().collect();
        return Ok(Outcome::fail(format!(
            "DFA-E325 {} references do not resolve: {}",
            unresolved.len(),
            first.join(", ")
        )));
    }

    let prefixes = c.arr_key("prefixes", &[]);
    let index = ids::build(&ctx.root);
    let mut unresolved: Vec<String> = Vec::new();
    for rid in index.references.keys() {
        let Some((prefix, _)) = ids::split_id(rid) else {
            continue;
        };
        if !prefixes.is_empty() && !prefixes.contains(&prefix.to_string()) {
            continue;
        }
        if !index.defines(rid) {
            unresolved.push(rid.clone());
        }
    }
    unresolved.sort();
    unresolved.dedup();

    if unresolved.is_empty() {
        return Ok(Outcome::pass());
    }
    let cited: Vec<String> = unresolved.iter().take(5).cloned().collect();
    let summary = format!(
        "DFA-E325 {} references do not resolve: {}",
        unresolved.len(),
        cited.join(", ")
    );
    let findings = cited
        .iter()
        .map(|u| Finding {
            id: u.clone(),
            severity: "block".into(),
            summary: cut(&summary, 90),
            evidence: String::new(),
            category: None,
        })
        .collect();
    Ok(Outcome::fail(summary).with_findings(findings))
}

fn file_exists(ctx: &mut Ctx, c: &Check, id: &str, phase: &str) -> Result<Outcome, CliError> {
    let paths = c.arr_key("paths", &[]);
    let absent = c.bool_key("absent", false);
    // `gates::parse_check` refuses `min_count = 0` with `absent = false` and
    // refuses `min_count` at all with `absent = true`, so the value read here
    // always asserts something about the filesystem.
    let min_count = c.int_key("min_count", paths.len() as i64);
    if paths.is_empty() {
        return Ok(Outcome::fail(format!(
            "DFA-E323 gates.toml check '{}' names no path to test",
            c.id
        )));
    }

    let resolved: Vec<(String, PathBuf)> = paths
        .iter()
        .map(|p| {
            let rel = substitute(p, id, phase);
            let full = ctx.doc_path(&rel);
            (rel, full)
        })
        .collect();

    if absent {
        let present: Vec<&String> = resolved
            .iter()
            .filter(|(_, p)| p.exists())
            .map(|(r, _)| r)
            .collect();
        if present.is_empty() {
            return Ok(Outcome::pass());
        }
        return Ok(Outcome::fail(format!(
            "DFA-E323 {} still present",
            present
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )));
    }

    // A path counts when it exists and is non-empty. A directory counts only
    // when it holds an entry: an empty directory created at a document's path
    // is not the document.
    let mut found = 0i64;
    let mut missing: Vec<&String> = Vec::new();
    for (rel, full) in &resolved {
        let ok = std::fs::metadata(full)
            .map(|m| {
                if m.is_dir() {
                    std::fs::read_dir(full)
                        .map(|mut d| d.next().is_some())
                        .unwrap_or(false)
                } else {
                    m.len() > 0
                }
            })
            .unwrap_or(false);
        if ok {
            found += 1;
        } else {
            missing.push(rel);
        }
    }
    if found >= min_count {
        return Ok(Outcome::pass().with_evidence(map(&[("found", Value::from(found))])));
    }
    Ok(Outcome::fail(format!(
        "DFA-E323 found {found} of {min_count} paths; missing {}",
        missing
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    )))
}

fn fields_present(
    ctx: &mut Ctx,
    gate: &Gate,
    c: &Check,
    id: &str,
    phase: &str,
) -> Result<Outcome, CliError> {
    let collection = c.str_key("collection", "");
    let fields = c.arr_key("fields", &[]);
    // `path` names a document in every branch, defaulting to the gate's
    // `document`; `field` names a location inside it. Reading `path` as a field
    // whenever the gate carried a `document` gave the same key two meanings and
    // made a check with neither key fail unconditionally.
    let rel = subject_path(gate, c, id, phase);
    let field = substitute(&c.str_key("field", ""), id, phase);
    let (_, value) = load_value(ctx, &rel)?;

    if collection.is_empty() {
        // With no `collection`, the value at `field` is non-null; with no
        // `field` either, the document itself is non-null, which spec 252 makes
        // the default behaviour.
        let found = resolve_at(&value, &field).map_err(|m| missing_path(&c.id, &m.path, &rel))?;
        // Every value the location names is non-null: one non-null entry of
        // five does not carry the other four.
        let held = !found.is_empty() && found.iter().all(|v| !matches!(v, Value::Null));
        if held {
            return Ok(Outcome::pass());
        }
        return Ok(Outcome::fail(format!("DFA-E332 {rel} '{field}' is null")));
    }

    let path = collection.trim_end_matches("[]");
    let entries =
        resolve_at(&value, &format!("{path}[]")).map_err(|m| missing_path(&c.id, &m.path, &rel))?;

    for entry in entries {
        let entry_id = entry
            .get(Value::String("id".into()))
            .and_then(Value::as_str)
            .unwrap_or("");
        for field in &fields {
            let held = entry
                .get(Value::String(field.clone()))
                .map(|v| match v {
                    Value::Null => false,
                    Value::String(s) => !s.trim().is_empty(),
                    Value::Sequence(s) => !s.is_empty(),
                    // An empty mapping holds nothing, so it is not a value.
                    Value::Mapping(m) => !m.is_empty(),
                    _ => true,
                })
                .unwrap_or(false);
            if !held {
                return Ok(Outcome::fail(format!(
                    "DFA-E332 {rel} {collection} entry '{entry_id}' has no '{field}'"
                )));
            }
        }
    }
    Ok(Outcome::pass())
}

fn field_in_enum(
    ctx: &mut Ctx,
    gate: &Gate,
    c: &Check,
    id: &str,
    phase: &str,
) -> Result<Outcome, CliError> {
    let rel = subject_path(gate, c, id, phase);
    let field = substitute(&c.str_key("field", ""), id, phase);
    let values = c.arr_key("values", &[]);
    let (_, value) = load_value(ctx, &rel)?;
    // A null, a list, and a mapping at the location are present values of the
    // wrong shape, not absences: rendering them and testing membership fails
    // the check rather than skipping the value.
    let found: Vec<String> = resolve_at(&value, &field)
        .map_err(|m| missing_path(&c.id, &m.path, &rel))?
        .iter()
        .map(|v| render(v))
        .collect();

    for v in &found {
        if !values.contains(v) {
            return Ok(Outcome::fail(format!(
                "DFA-E322 {rel} field '{field}' is '{v}'; gate '{}' check '{}' allows {}",
                gate.phase,
                c.id,
                values.join(", ")
            ))
            .with_evidence(map(&[("value", Value::from(v.clone()))])));
        }
    }
    Ok(Outcome::pass().with_evidence(map(&[(
        "value",
        Value::from(found.first().cloned().unwrap_or_default()),
    )])))
}

/// Parse `text` in the strftime `format` the check names, returning the date so
/// an `after_field` comparison orders dates rather than strings.
fn parse_date(text: &str, format: &str) -> Option<::time::Date> {
    let desc = ::time::format_description::parse_strftime_borrowed(format).ok()?;
    ::time::Date::parse(text, &desc).ok()
}

fn field_is_date(
    ctx: &mut Ctx,
    gate: &Gate,
    c: &Check,
    id: &str,
    phase: &str,
) -> Result<Outcome, CliError> {
    let rel = subject_path(gate, c, id, phase);
    let field = substitute(&c.str_key("field", ""), id, phase);
    let format = c.str_key("format", "%Y-%m-%d");
    let after_field = substitute(&c.str_key("after_field", ""), id, phase);
    let (_, value) = load_value(ctx, &rel)?;

    // An absent path is a defect in gates.toml, reported as DFA-E345 naming
    // the path, rather than as a document defect naming a value nobody wrote.
    let found = resolve_at(&value, &field).map_err(|m| missing_path(&c.id, &m.path, &rel))?;
    if found.is_empty() {
        return Ok(Outcome::fail(format!(
            "DFA-E330 {rel} field '{field}' names no value, expected {format}"
        )));
    }
    // Every value the location names parses, not just the first.
    let mut dates: Vec<::time::Date> = Vec::new();
    for v in &found {
        let text = render(v);
        match parse_date(&text, &format) {
            Some(d) => dates.push(d),
            None => {
                return Ok(Outcome::fail(format!(
                    "DFA-E330 {rel} field '{field}' is '{text}', expected {format}"
                ))
                .with_evidence(map(&[("value", Value::from(text))])))
            }
        }
    }
    if !after_field.is_empty() {
        let other =
            resolve_at(&value, &after_field).map_err(|m| missing_path(&c.id, &m.path, &rel))?;
        let Some(other_text) = other.first().map(|v| render(v)) else {
            return Ok(Outcome::fail(format!(
                "DFA-E330 {rel} field '{after_field}' names no value to compare '{field}' against"
            )));
        };
        let Some(other_date) = parse_date(&other_text, &format) else {
            return Ok(Outcome::fail(format!(
                "DFA-E330 {rel} field '{after_field}' is '{other_text}', expected {format}"
            )));
        };
        if dates.iter().any(|d| *d <= other_date) {
            return Ok(Outcome::fail(format!(
                "DFA-E330 {rel} field '{field}' is not later than '{after_field}'"
            )));
        }
    }
    Ok(Outcome::pass())
}

fn length_between(
    ctx: &mut Ctx,
    gate: &Gate,
    c: &Check,
    id: &str,
    phase: &str,
) -> Result<Outcome, CliError> {
    let rel = subject_path(gate, c, id, phase);
    let field = substitute(&c.str_key("field", ""), id, phase);
    let min = c.int_key("min", 0);
    let max = c.int_key("max", i64::from(i32::MAX));
    let exclude = c.arr_key("exclude_status", &[]);
    let (_, value) = load_value(ctx, &rel)?;

    let base = field.trim_end_matches("[]");
    let lists = resolve_at(&value, base).map_err(|m| missing_path(&c.id, &m.path, &rel))?;

    for list in lists {
        // A null at the location is a list of zero; a scalar or a mapping there
        // is a defect in the document, so the three shapes stay distinct rather
        // than all three being skipped as "not a sequence".
        let seq = match list {
            Value::Sequence(s) => s,
            Value::Null => {
                if 0 < min {
                    return Ok(Outcome::fail(format!(
                        "DFA-E331 {rel} {field} is null, expected {min} to {max} entries"
                    ))
                    .with_evidence(map(&[("value", Value::from(0))])));
                }
                continue;
            }
            other => {
                return Ok(Outcome::fail(format!(
                    "DFA-E331 {rel} {field} holds '{}', which is not a list",
                    render(other)
                )))
            }
        };
        let kept = seq
            .iter()
            .filter(|item| {
                let status = item
                    .get(Value::String("status".into()))
                    .and_then(Value::as_str)
                    .unwrap_or("");
                !exclude.iter().any(|e| e == status)
            })
            .count() as i64;
        if kept < min || kept > max {
            return Ok(Outcome::fail(format!(
                "DFA-E331 {rel} {field} holds {kept} entries, expected {min} to {max}"
            ))
            .with_evidence(map(&[("value", Value::from(kept))])));
        }
    }
    Ok(Outcome::pass())
}

fn no_open_questions(ctx: &mut Ctx, c: &Check, id: &str, phase: &str) -> Result<Outcome, CliError> {
    let patterns = c.arr_key("docs", &[]);
    let paths = expand_docs(ctx, &patterns, id, phase);
    if paths.is_empty() {
        return Err(CliError::at(
            "DFA-E345",
            format!(
                "gates.toml check '{}': docs {:?} matched no document",
                c.id, patterns
            ),
            ".devforgeai/gates.toml",
        ));
    }
    for path in paths {
        let rel = ctx.rel(&path);
        // An absent or unparsable document propagates, so the E200 or E401
        // reaches the conversion in `evaluate` and fails this check.
        let (_, value) = load_value(ctx, &rel)?;
        let open = match value.get(Value::String("open_questions".into())) {
            None | Some(Value::Null) => 0,
            Some(Value::Sequence(s)) => s.len(),
            Some(other) => {
                return Ok(Outcome::fail(format!(
                    "DFA-E324 {rel} open_questions holds '{}', which is not a list",
                    render(other)
                )))
            }
        };
        if open > 0 {
            return Ok(Outcome::fail(format!(
                "DFA-E324 {rel} has {open} open questions"
            )));
        }
    }
    Ok(Outcome::pass())
}

fn set_cover(
    ctx: &mut Ctx,
    gate: &Gate,
    c: &Check,
    id: &str,
    phase: &str,
) -> Result<Outcome, CliError> {
    let rel = subject_path(gate, c, id, phase);
    let cover_path = c.str_key("cover", "");
    let universe_path = c.str_key("universe", "");
    let exclude = c.arr_key("exclude_status", &[]);
    let (_, value) = load_value(ctx, &rel)?;

    // The cover is a multiset: nested sequences flatten.
    let mut cover: Vec<String> = Vec::new();
    for v in resolve_at(&value, &cover_path).map_err(|m| missing_path(&c.id, &m.path, &rel))? {
        match v {
            Value::Sequence(s) => cover.extend(s.iter().filter_map(|x| ypath::scalar(&x))),
            other => {
                if let Some(s) = ypath::scalar(&other) {
                    cover.push(s);
                }
            }
        }
    }

    // The universe drops excluded records. The identity of a record is read
    // from the leaf of `universe`, not from a hardcoded `id`, so a document
    // whose records key on `key` is a set rather than nothing; the status is
    // read from `status_field`, which defaults to `status`.
    let status_field = c.str_key("status_field", "status");
    let (base, leaf) = match universe_path.rsplit_once('.') {
        Some((b, l)) if b.ends_with("[]") => (b.trim_end_matches("[]").to_string(), l.to_string()),
        _ => (
            universe_path.trim_end_matches("[]").to_string(),
            "id".to_string(),
        ),
    };
    let records =
        resolve_at(&value, &format!("{base}[]")).map_err(|m| missing_path(&c.id, &m.path, &rel))?;
    let mut universe: Vec<String> = Vec::new();
    let mut every_record: BTreeSet<String> = BTreeSet::new();
    for r in records {
        let status = r
            .get(Value::String(status_field.clone()))
            .and_then(Value::as_str)
            .unwrap_or("");
        let Some(rid) = r.get(Value::String(leaf.clone())).and_then(Value::as_str) else {
            continue;
        };
        every_record.insert(rid.to_string());
        if exclude.iter().any(|e| e == status) {
            continue;
        }
        universe.push(rid.to_string());
    }

    let mut seen: BTreeMap<&String, usize> = BTreeMap::new();
    for v in &cover {
        *seen.entry(v).or_insert(0) += 1;
    }
    if let Some((dup, _)) = seen.iter().find(|(_, n)| **n > 1) {
        return Ok(
            Outcome::fail(format!("DFA-E333 {dup} appears twice in {cover_path}"))
                .with_evidence(map(&[("value", Value::from((*dup).clone()))])),
        );
    }

    let covered: BTreeSet<&String> = cover.iter().collect();
    let uncovered: Vec<&String> = universe.iter().filter(|u| !covered.contains(u)).collect();
    if !uncovered.is_empty() {
        let first: Vec<&str> = uncovered.iter().take(5).map(|s| s.as_str()).collect();
        return Ok(Outcome::fail(format!(
            "DFA-E333 {} ids at {universe_path} are absent from {cover_path}: {}",
            uncovered.len(),
            first.join(", ")
        ))
        .with_evidence(map(&[("value", Value::from(uncovered[0].clone()))])));
    }

    // The equality is two-directional: a cover value naming no universe record
    // is a fabricated reference, invisible while only the uncovered half ran.
    // A value naming an excluded record is not fabricated, so the second
    // direction tests membership of every record rather than of the filtered
    // universe.
    let extra: Vec<&String> = cover
        .iter()
        .filter(|v| !every_record.contains(*v))
        .collect();
    if !extra.is_empty() {
        let first: Vec<&str> = extra.iter().take(5).map(|s| s.as_str()).collect();
        return Ok(Outcome::fail(format!(
            "DFA-E333 {} values at {cover_path} name no record at {universe_path}: {}",
            extra.len(),
            first.join(", ")
        ))
        .with_evidence(map(&[("value", Value::from(extra[0].clone()))])));
    }
    Ok(Outcome::pass())
}

/// The markdown table rows, or the non-blank body lines, of an H2 section.
struct Section {
    header: Vec<String>,
    rows: Vec<Vec<String>>,
    body_lines: usize,
}

/// The named H2 section, or `None` when the file holds no such heading. An
/// absent section and a present-but-empty one are different defects and the
/// caller reports them differently.
fn read_section(ctx: &Ctx, rel: &str, name: &str) -> Result<Option<Section>, CliError> {
    let path = ctx.doc_path(rel);
    let text = project::read_doc(&path)?;
    let mut inside = false;
    let mut seen = false;
    let mut header: Vec<String> = Vec::new();
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut body_lines = 0usize;

    for line in text.lines() {
        let t = line.trim_end();
        if let Some(rest) = t.strip_prefix("##") {
            // A heading of depth three or more is structure inside the section
            // rather than a body line, and does not close it. `## Name` and
            // `##Name` name the same section.
            if t.chars().take_while(|ch| *ch == '#').count() >= 3 {
                continue;
            }
            if inside {
                break;
            }
            inside = rest.trim() == name;
            seen |= inside;
            continue;
        }
        if t.starts_with("# ") && inside {
            break;
        }
        if !inside {
            continue;
        }
        let trimmed = t.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('|') {
            let cells: Vec<String> = trimmed
                .trim_matches('|')
                .split('|')
                .map(|c| c.trim().to_string())
                .collect();
            // The separator row is not data.
            if cells
                .iter()
                .all(|c| !c.is_empty() && c.chars().all(|ch| ch == '-' || ch == ':'))
            {
                continue;
            }
            if header.is_empty() {
                header = cells;
            } else {
                rows.push(cells);
            }
            continue;
        }
        body_lines += 1;
    }

    if !seen {
        return Ok(None);
    }
    Ok(Some(Section {
        header,
        rows,
        body_lines,
    }))
}

/// The named section, or a failure naming it. A section the document does not
/// hold is not a section holding nothing.
fn section_or_fail(
    ctx: &Ctx,
    rel: &str,
    name: &str,
    code: &str,
) -> Result<Result<Section, Outcome>, CliError> {
    match read_section(ctx, rel, name)? {
        Some(s) => Ok(Ok(s)),
        None => Ok(Err(Outcome::fail(format!(
            "{code} {rel} has no H2 section '{name}'"
        )))),
    }
}

fn row_count_between(
    ctx: &mut Ctx,
    gate: &Gate,
    c: &Check,
    id: &str,
    phase: &str,
) -> Result<Outcome, CliError> {
    let rel = subject_path(gate, c, id, phase);
    let name = c.str_key("section", "");
    let min = c.int_key("min", 0);
    let max = c.int_key("max", i64::from(i32::MAX));
    let s = match section_or_fail(ctx, &rel, &name, "DFA-E334")? {
        Ok(s) => s,
        Err(out) => return Ok(out),
    };

    // The table data rows, or that many non-blank body lines when the section
    // holds no table.
    let count = if s.header.is_empty() {
        s.body_lines as i64
    } else {
        s.rows.len() as i64
    };
    if count < min || count > max {
        return Ok(Outcome::fail(format!(
            "DFA-E334 {rel} section '{name}' holds {count} rows, expected {min} to {max}"
        ))
        .with_evidence(map(&[("value", Value::from(count))])));
    }
    Ok(Outcome::pass().with_evidence(map(&[("value", Value::from(count))])))
}

fn column_of(s: &Section, name: &str) -> Option<usize> {
    s.header.iter().position(|h| h == name)
}

fn column_matches(
    ctx: &mut Ctx,
    gate: &Gate,
    c: &Check,
    id: &str,
    phase: &str,
) -> Result<Outcome, CliError> {
    let rel = subject_path(gate, c, id, phase);
    let name = c.str_key("section", "");
    let column = c.str_key("column", "");
    let pattern = c.str_key("pattern", "");
    let unique = c.bool_key("unique", false);
    let s = match section_or_fail(ctx, &rel, &name, "DFA-E335")? {
        Ok(s) => s,
        Err(out) => return Ok(out),
    };

    let Some(col) = column_of(&s, &column) else {
        return Ok(Outcome::fail(format!(
            "DFA-E335 {rel} section '{name}' has no column '{column}'"
        )));
    };
    let compiled = crate::pattern::Pattern::compile(&pattern)?;
    let mut seen: BTreeSet<String> = BTreeSet::new();

    for (at, row) in s.rows.iter().enumerate() {
        // A row with fewer cells than the column index is a malformed table,
        // not a row the check cannot see.
        let Some(cell) = row.get(col) else {
            return Ok(Outcome::fail(format!(
                "DFA-E335 {rel} section '{name}' row {} has no cell in column '{column}'",
                at + 1
            )));
        };
        if !compiled.is_match(cell) {
            return Ok(Outcome::fail(format!(
                "DFA-E335 {rel} section '{name}' column '{column}' value '{cell}' is malformed"
            ))
            .with_evidence(map(&[("value", Value::from(cell.clone()))])));
        }
        if unique && !seen.insert(cell.clone()) {
            return Ok(Outcome::fail(format!(
                "DFA-E335 {rel} section '{name}' column '{column}' value '{cell}' is duplicated"
            ))
            .with_evidence(map(&[("value", Value::from(cell.clone()))])));
        }
    }
    Ok(Outcome::pass())
}

fn column_contains_all(
    ctx: &mut Ctx,
    gate: &Gate,
    c: &Check,
    id: &str,
    phase: &str,
) -> Result<Outcome, CliError> {
    let rel = subject_path(gate, c, id, phase);
    let name = c.str_key("section", "");
    let column = c.str_key("column", "");
    let state_field = c.str_key("state_field", "");

    // An absent or misspelled `state_field` raises rather than emptying the
    // wanted set: an empty set passes the check, so swallowing the miss made a
    // typo in gates.toml indistinguishable from a satisfied requirement.
    let wanted: Vec<String> = {
        let s = ctx.state()?;
        let value = serde_yaml_ng::to_value(s).unwrap_or(Value::Null);
        resolve_at(&value, &state_field)
            .map_err(|m| {
                CliError::at(
                    "DFA-E345",
                    format!(
                        "gates.toml check '{}': state_field '{}' does not resolve in state.toml",
                        c.id, m.path
                    ),
                    ".devforgeai/state.toml",
                )
            })?
            .iter()
            .flat_map(|v| match v {
                Value::Sequence(seq) => seq.iter().filter_map(|x| ypath::scalar(&x)).collect(),
                other => ypath::scalar(other).into_iter().collect::<Vec<_>>(),
            })
            .collect()
    };

    let s = match section_or_fail(ctx, &rel, &name, "DFA-E336")? {
        Ok(s) => s,
        Err(out) => return Ok(out),
    };
    let Some(col) = column_of(&s, &column) else {
        return Ok(Outcome::fail(format!(
            "DFA-E336 {rel} section '{name}' has no column '{column}'"
        )));
    };
    let present: BTreeSet<&String> = s.rows.iter().filter_map(|r| r.get(col)).collect();

    for w in &wanted {
        if !present.contains(w) {
            return Ok(Outcome::fail(format!(
                "DFA-E336 {w} is in {state_field} and absent from {rel} section '{name}' column '{column}'"
            ))
            .with_evidence(map(&[("value", Value::from(w.clone()))])));
        }
    }
    Ok(Outcome::pass())
}

fn elapsed_days(ctx: &mut Ctx, c: &Check, id: &str, phase: &str) -> Result<Outcome, CliError> {
    // `{id}` and `{phase}` are replaced in every path a check names, state
    // paths included, so `phases.{phase}.started_at` resolves.
    let started_field = substitute(&c.str_key("started_field", ""), id, phase);
    let limit_field = substitute(&c.str_key("limit_field", ""), id, phase);
    let remedy_started = substitute(&c.str_key("remedy_started_field", ""), id, phase);
    let remedy_limit = substitute(&c.str_key("remedy_limit_field", ""), id, phase);

    let s = ctx.state()?;
    let value = serde_yaml_ng::to_value(s).unwrap_or(Value::Null);
    let read = |p: &str| -> Option<String> {
        resolve_at(&value, p)
            .ok()
            .and_then(|v| v.first().map(|x| render(x)))
            .filter(|t| t != "null")
    };

    // The remedy pair is used instead when both are set and the remedy start
    // is non-empty.
    let (start_key, limit_key) = if !remedy_started.is_empty()
        && !remedy_limit.is_empty()
        && read(&remedy_started)
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    {
        (remedy_started, remedy_limit)
    } else {
        (started_field, limit_field)
    };

    // Both halves of the check fail closed. An absent start disarms the time
    // box permanently, and an unparsable one does the same by editing a single
    // state string, so neither is a pass.
    let Some(started) = read(&start_key).filter(|s| !s.is_empty()) else {
        return Ok(Outcome::fail(format!(
            "DFA-E345 state.toml '{start_key}' holds no timestamp to measure the time box from"
        )));
    };
    let limit: i64 = read(&limit_key).and_then(|v| v.parse().ok()).unwrap_or(0);
    let Some(from) = time::parse_rfc3339(&started) else {
        return Ok(Outcome::fail(format!(
            "DFA-E337 state.toml '{start_key}' holds '{started}', which is not an RFC 3339 timestamp"
        )));
    };
    let elapsed = time::calendar_days_between(from, time::now());
    if elapsed > limit {
        return Ok(Outcome::fail(format!(
            "DFA-E337 {elapsed} days elapsed of {limit} allowed since {start_key}"
        ))
        .with_evidence(map(&[
            ("value", Value::from(elapsed)),
            ("limit", Value::from(limit)),
        ])));
    }
    Ok(Outcome::pass().with_evidence(map(&[
        ("value", Value::from(elapsed)),
        ("limit", Value::from(limit)),
    ])))
}

fn verifier_pass(ctx: &mut Ctx, c: &Check, id: &str, phase: &str) -> Result<Outcome, CliError> {
    let names = c.arr_key("verifiers", &[]);
    let min_ratio = c.float_key("min_ratio", 1.0);
    let path = report::report_path(&ctx.root, id, phase);
    let rep = match report::load(&path) {
        Ok(r) => r,
        Err(e) if e.code() == "DFA-E400" => Report::skeleton(id, phase),
        Err(e) => return Err(e),
    };
    let cfg = ctx.config()?.clone();

    // A `verifier_pass` naming nobody counts nothing: `verifiers` is required,
    // and a present-but-empty list is refused here for the same reason.
    if names.is_empty() {
        return Ok(Outcome::fail(format!(
            "DFA-E316 gates.toml check '{}' names no verifier",
            c.id
        )));
    }

    for name in &names {
        let Some(v) = cfg.verifier_by_name(name) else {
            // The fault is in config.toml, not in the report: saying so sends
            // the reader to the file that can be fixed.
            return Ok(Outcome::fail(format!(
                "DFA-E316 config.toml registers no [[verifier]] named '{name}'"
            )));
        };
        let Some(block) = rep.verifier_block(&v.report_field) else {
            return Ok(Outcome::fail(format!(
                "DFA-E316 report {} has no verifiers block for '{name}'",
                ctx.rel(&path)
            )));
        };
        // `passed` and `total` are read as present integers. Defaulting a
        // missing or wrongly typed key to 0 made a block that counts nothing
        // read as the spec's `total == 0` free pass, which belongs to a
        // verifier that genuinely counted zero units.
        let read = |key: &str| -> Result<i64, String> {
            match block.get(key) {
                Some(Value::Number(n)) if n.is_i64() => Ok(n.as_i64().unwrap_or(0)),
                Some(other) => Err(format!(
                    "DFA-E317 verifier '{name}' block has '{key}' = '{}', which is not an integer",
                    render(other)
                )),
                None => Err(format!("DFA-E317 verifier '{name}' block has no '{key}'")),
            }
        };
        let passed = match read("passed") {
            Ok(v) => v,
            Err(reason) => return Ok(Outcome::fail(reason)),
        };
        let total = match read("total") {
            Ok(v) => v,
            Err(reason) => return Ok(Outcome::fail(reason)),
        };
        if passed < 0 || total < 0 || passed > total {
            return Ok(Outcome::fail(format!(
                "DFA-E317 verifier '{name}' reports {passed}/{total}, which is not a ratio"
            ))
            .with_evidence(map(&[
                ("passed", Value::from(passed)),
                ("total", Value::from(total)),
            ])));
        }
        // A verifier with no unit to count passes.
        let ratio = if total == 0 {
            1.0
        } else {
            passed as f64 / total as f64
        };
        if ratio < min_ratio {
            let findings: Vec<Finding> = rep
                .findings
                .iter()
                .filter(|f| f.severity == "block")
                .cloned()
                .collect();
            return Ok(Outcome::fail(format!(
                "DFA-E317 verifier '{name}' passed {passed}/{total}, below min_ratio {min_ratio}"
            ))
            .with_evidence(map(&[
                ("passed", Value::from(passed)),
                ("total", Value::from(total)),
            ]))
            .with_findings(findings));
        }
    }
    Ok(Outcome::pass())
}

fn report_metric(ctx: &mut Ctx, c: &Check, id: &str, phase: &str) -> Result<Outcome, CliError> {
    let metric = c.str_key("metric", "");
    let op = c.str_key("op", "eq");
    let want = c.float_key("value", 0.0);
    let path = report::report_path(&ctx.root, id, phase);
    let rep = match report::load(&path) {
        Ok(r) => r,
        Err(e) if e.code() == "DFA-E400" => Report::skeleton(id, phase),
        Err(e) => return Err(e),
    };

    let Some(actual) = rep.metric(&metric) else {
        return Ok(Outcome::fail(format!(
            "DFA-E338 report {} has no metric '{metric}'",
            ctx.rel(&path)
        )));
    };
    // A relative tolerance, so `eq` and `ne` stay meaningful above magnitude 1
    // where an absolute `f64::EPSILON` degenerates into exact equality.
    let close = (actual - want).abs() <= f64::EPSILON * actual.abs().max(want.abs()).max(1.0);
    let ok = match op.as_str() {
        "eq" => close,
        "ne" => !close,
        "lt" => actual < want,
        "lte" => actual <= want,
        "gt" => actual > want,
        "gte" => actual >= want,
        // `gates::parse_check` refuses an `op` outside the enum, so this arm
        // is unreachable from a parsed file.
        _ => false,
    };
    let evidence = map(&[("value", Value::from(actual)), ("limit", Value::from(want))]);
    if ok {
        return Ok(Outcome::pass().with_evidence(evidence));
    }
    Ok(Outcome::fail(format!(
        "DFA-E338 {metric} is {actual}, expected {op} {want}"
    ))
    .with_evidence(evidence))
}

fn no_cycle(ctx: &mut Ctx, c: &Check, id: &str, phase: &str) -> Result<Outcome, CliError> {
    let patterns = c.arr_key("docs", &[]);
    let root_node = substitute(&c.str_key("root", ""), id, phase);
    let from = c.str_key("from", "");
    let to = c.str_key("to", "");

    let mut edges: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut nodes: BTreeSet<String> = BTreeSet::new();

    let paths = expand_docs(ctx, &patterns, id, phase);
    // A glob matching nothing builds an empty graph, which holds no cycle for
    // the wrong reason.
    if paths.is_empty() {
        return Err(CliError::at(
            "DFA-E345",
            format!(
                "gates.toml check '{}': docs {:?} matched no document",
                c.id, patterns
            ),
            ".devforgeai/gates.toml",
        ));
    }
    for path in paths {
        let rel = ctx.rel(&path);
        // A document that cannot be read, or that defines no node, is a defect
        // in the `docs` set rather than a document to drop from the graph: a
        // dropped document turns the edge into it into a leaf, which is exactly
        // how a cycle disappears.
        let (_, value) = load_value(ctx, &rel)?;
        let node = resolve_at(&value, &from)
            .map_err(|m| missing_path(&c.id, &m.path, &rel))?
            .first()
            .map(|v| render(v))
            .ok_or_else(|| missing_path(&c.id, &from, &rel))?;
        nodes.insert(node.clone());
        let targets: Vec<String> = resolve_at(&value, &to)
            .unwrap_or_default()
            .iter()
            .filter(|v| !matches!(v, Value::Null))
            .map(|v| render(v))
            .collect();
        edges.entry(node).or_default().extend(targets);
    }

    // A `to` value naming a document outside `docs` is a leaf.
    let mut stack: Vec<String> = Vec::new();
    let mut visiting: BTreeSet<String> = BTreeSet::new();
    let mut done: BTreeSet<String> = BTreeSet::new();

    fn walk(
        node: &str,
        edges: &BTreeMap<String, Vec<String>>,
        nodes: &BTreeSet<String>,
        visiting: &mut BTreeSet<String>,
        done: &mut BTreeSet<String>,
        stack: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        if done.contains(node) {
            return None;
        }
        if !visiting.insert(node.to_string()) {
            let at = stack.iter().position(|n| n == node).unwrap_or(0);
            let mut cycle = stack[at..].to_vec();
            cycle.push(node.to_string());
            return Some(cycle);
        }
        stack.push(node.to_string());
        if let Some(targets) = edges.get(node) {
            for t in targets {
                if !nodes.contains(t) {
                    continue;
                }
                if let Some(cycle) = walk(t, edges, nodes, visiting, done, stack) {
                    return Some(cycle);
                }
            }
        }
        stack.pop();
        visiting.remove(node);
        done.insert(node.to_string());
        None
    }

    let roots: Vec<String> = if root_node.is_empty() {
        nodes.iter().cloned().collect()
    } else {
        vec![root_node.clone()]
    };

    for r in roots {
        if !nodes.contains(&r) {
            continue;
        }
        visiting.clear();
        done.clear();
        stack.clear();
        if let Some(cycle) = walk(&r, &edges, &nodes, &mut visiting, &mut done, &mut stack) {
            return Ok(Outcome::fail(format!(
                "DFA-E340 deferral cycle: {}",
                cycle.join(" -> ")
            )));
        }
    }
    Ok(Outcome::pass())
}

fn yaml_cites(ctx: &mut Ctx, c: &Check, id: &str, phase: &str) -> Result<Outcome, CliError> {
    let rel = substitute(&c.str_key("doc", ""), id, phase);
    let from = c.str_key("from", "");
    let field = c.str_key("field", "");
    let into = c.arr_key("into", &[]);
    let min = c.int_key("min", 1);
    let (_, value) = load_value(ctx, &rel)?;

    // An empty sequence at a `[]` segment is the empty set, not a missing path:
    // `observations: []` makes the union empty and the `from` list empty, which
    // spec 277 says passes.
    let mut universe: BTreeSet<String> = BTreeSet::new();
    for p in &into {
        let found = resolve_at(&value, p).map_err(|m| missing_path(&c.id, &m.path, &rel))?;
        universe.extend(found.iter().filter_map(ypath::scalar));
    }

    let items = match resolve_at(&value, &format!("{}[]", from.trim_end_matches("[]"))) {
        Ok(i) => i,
        Err(m) => return Err(missing_path(&c.id, &m.path, &rel)),
    };

    for (n, item) in items.iter().enumerate() {
        let item_id = item
            .get(Value::String("id".into()))
            .and_then(Value::as_str)
            .unwrap_or("");
        let cites: Vec<String> = item
            .get(Value::String(field.clone()))
            .and_then(Value::as_sequence)
            .map(|s| s.iter().filter_map(|x| ypath::scalar(&x)).collect())
            .unwrap_or_default();
        if (cites.len() as i64) < min {
            return Ok(Outcome::fail(format!(
                "DFA-E346 {rel}: {from}[{n}] id '{item_id}' cites {} of {min} required",
                cites.len()
            )));
        }
        for cite in &cites {
            if !universe.contains(cite) {
                return Ok(Outcome::fail(format!(
                    "DFA-E347 {rel}: {from}[{n}] id '{item_id}' cites '{cite}', which {} does not define",
                    into.join(", ")
                )));
            }
        }
    }
    Ok(Outcome::pass())
}

fn no_threshold_decrease(
    ctx: &mut Ctx,
    c: &Check,
    id: &str,
    phase: &str,
) -> Result<Outcome, CliError> {
    let rel = substitute(&c.str_key("doc", ""), id, phase);
    let from = c.str_key("from", "recommendations");
    let target_field = c.str_key("target_field", "target.path");
    let key_field = c.str_key("key_field", "target.key");
    let value_field = c.str_key("value_field", "proposed_value");
    let files = c.arr_key("files", &["gates.toml", "config.toml"]);
    let (_, value) = load_value(ctx, &rel)?;

    let items = ypath::resolve(&value, &format!("{}[]", from.trim_end_matches("[]")))
        .map_err(|m| missing_path(&c.id, &m.path, &rel))?;

    for (n, item) in items.iter().enumerate() {
        let item_id = item
            .get(Value::String("id".into()))
            .and_then(Value::as_str)
            .unwrap_or("");
        let target = ypath::resolve_strings(item, &target_field)
            .unwrap_or_default()
            .first()
            .cloned()
            .unwrap_or_default();
        if !files.iter().any(|f| target.ends_with(f.as_str())) {
            continue;
        }
        let key = ypath::resolve_strings(item, &key_field)
            .unwrap_or_default()
            .first()
            .cloned()
            .unwrap_or_default();
        let leaf = key.rsplit('.').next().unwrap_or(&key);
        if !THRESHOLD_LEAVES.contains(&leaf) {
            continue;
        }
        // A key ending in a threshold name that is written in none of the
        // three recognised shapes cannot be located in the floor table, which
        // is a defect in the recommendation rather than a reason to skip it.
        if !recognised_threshold_shape(&key) {
            return Ok(Outcome::fail(format!(
                "DFA-E348 {rel}: {from}[{n}] id '{item_id}' names threshold '{key}', which is written in no recognised shape"
            )));
        }
        // A recognised shape whose phase or layer carries no floor is not
        // compared.
        let Some(floor) = compiled_floor(&key) else {
            continue;
        };
        // A quoted number is still a number: reading only `as_f64` let
        // `proposed_value: "40"` walk past the floor.
        let proposed = resolve_at(item, &value_field)
            .ok()
            .and_then(|v| v.first().copied())
            .and_then(|x| match x {
                Value::Number(n) => n.as_f64(),
                Value::String(s) => s.trim().parse::<f64>().ok(),
                _ => None,
            });
        let Some(proposed) = proposed else {
            return Ok(Outcome::fail(format!(
                "DFA-E348 {rel}: {from}[{n}] id '{item_id}' proposes {key} with no numeric value"
            )));
        };
        if proposed < floor {
            return Ok(Outcome::fail(format!(
                "DFA-E348 {rel}: {from}[{n}] id '{item_id}' proposes {key} = {proposed}, below the compiled floor {floor}"
            )));
        }
    }
    Ok(Outcome::pass())
}

/// The compiled floor for a threshold key named by a reflect recommendation.
///
/// The key is parsed into its shape rather than matched by substring:
/// `gate.<phase>.<check-id>.min_ratio` reads the floor from the compiled
/// per-phase table, so a phase with no floor is not compared at all, and
/// `layer.<name>.coverage_min` and `coverage.overall_min` read theirs from the
/// config table. A key naming a threshold in no recognised shape has no floor
/// and the item is left to the reader.
fn compiled_floor(key: &str) -> Option<f64> {
    let parts: Vec<&str> = key.split('.').collect();
    match parts.as_slice() {
        ["coverage", "overall_min"] => Some(crate::config::OVERALL_FLOOR),
        ["layer", name, "coverage_min"] => crate::config::LAYER_FLOORS
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, f)| *f),
        ["gate", phase, _check, "min_ratio"] => gates::min_ratio_floor(phase),
        _ => None,
    }
}

/// The leaf names of the compiled floor table. A key ending in one of these in
/// a shape `compiled_floor` does not recognise names a threshold the floor
/// table governs and cannot be located, which is a defect in the
/// recommendation rather than a reason to let it through.
const THRESHOLD_LEAVES: &[&str] = &["coverage_min", "overall_min", "min_ratio"];

/// True when `key` is written in one of the three shapes `compiled_floor`
/// reads, whether or not that shape carries a floor.
fn recognised_threshold_shape(key: &str) -> bool {
    let parts: Vec<&str> = key.split('.').collect();
    matches!(
        parts.as_slice(),
        ["coverage", "overall_min"] | ["layer", _, "coverage_min"] | ["gate", _, _, "min_ratio"]
    )
}

fn release_stories(ctx: &mut Ctx, c: &Check, id: &str, phase: &str) -> Result<Outcome, CliError> {
    let rel = substitute(&c.str_key("path", "releases/{id}.yaml"), id, phase);
    let require_status = c.arr_key("require_status", &["built", "released"]);
    let require_report = c.str_key("require_report", "verify");
    let require_result = c.str_key("require_result", "PASS");
    let (_, value) = load_value(ctx, &rel)?;

    // A release naming no story is the condition this kind exists to catch, so
    // an absent, misspelled, or non-sequence `stories` key is a failure rather
    // than a loop that never runs.
    let stories = match value.get(Value::String("stories".into())) {
        None => {
            return Err(missing_path(&c.id, "stories", &rel));
        }
        Some(Value::Sequence(s)) => s.clone(),
        Some(Value::Null) => Vec::new(),
        Some(other) => {
            return Ok(Outcome::fail(format!(
                "DFA-E341 {rel} stories holds '{}', which is not a list",
                render(other)
            )))
        }
    };
    if stories.is_empty() {
        return Ok(Outcome::fail(format!("DFA-E341 {rel} names no story")));
    }
    for (at, s) in stories.iter().enumerate() {
        let Some(sid) = s.get(Value::String("id".into())).and_then(Value::as_str) else {
            return Ok(Outcome::fail(format!(
                "DFA-E341 {rel} stories entry {at} carries no id"
            )));
        };
        let story_rel = format!("stories/{sid}.md");
        let status = load_value(ctx, &story_rel)
            .ok()
            .and_then(|(_, v)| {
                v.get(Value::String("status".into()))
                    .and_then(Value::as_str)
                    .map(str::to_string)
            })
            .unwrap_or_default();
        let report_path = report::report_path(&ctx.root, sid, &require_report);
        let result = report::load(&report_path)
            .map(|r| r.gate.result)
            .unwrap_or_default();

        if !require_status.contains(&status) || result != require_result {
            return Ok(Outcome::fail(format!(
                "DFA-E341 {sid} is status '{status}' with verify result '{result}'; the release needs {} and {require_result}",
                require_status.join(", ")
            ))
            .with_evidence(map(&[("value", Value::from(sid.to_string()))]))
            .with_findings(vec![Finding {
                id: sid.to_string(),
                severity: "block".into(),
                summary: cut(&format!("{sid} is not built with a PASS verify report"), 90),
                evidence: String::new(),
                category: None,
            }]));
        }
    }
    Ok(Outcome::pass())
}

/// `context_audit`: `context audit` exits 0, or exits 1 with warnings only and
/// `allow_warn = true`. The eight checks carry error codes alone, so the flag
/// only ever matters to a future warning-class finding.
fn context_audit(ctx: &mut Ctx, c: &Check) -> Result<Outcome, CliError> {
    let allow_warn = c.bool_key("allow_warn", false);
    let audit = crate::audit::run(ctx)?;
    let evidence = serde_yaml_ng::to_value(crate::audit::to_json(&audit))
        .unwrap_or(Value::Mapping(Default::default()));

    if audit.clean() {
        return Ok(Outcome::pass().with_evidence(evidence));
    }
    if allow_warn && audit.findings.iter().all(|f| f.code.starts_with("DFA-W")) {
        return Ok(Outcome::pass().with_evidence(evidence));
    }

    let n = audit.findings.len();
    let first = audit
        .findings
        .first()
        .map(|f| f.message.clone())
        .unwrap_or_default();
    let findings = audit
        .findings
        .iter()
        .map(|f| Finding {
            id: f.check.to_string(),
            severity: "block".to_string(),
            summary: cut(&f.message, 90),
            evidence: String::new(),
            category: None,
        })
        .collect();
    Ok(Outcome::fail(format!(
        "DFA-E327 context audit found {n} problems; first: {first}"
    ))
    .with_evidence(evidence)
    .with_findings(findings))
}

/// `story_valid`: `story validate` exits 0 for the scope.
fn story_valid(ctx: &mut Ctx, c: &Check, id: &str) -> Result<Outcome, CliError> {
    let scope = c.str_key("scope", "active");
    // The scope alone decides the set. Passing the gate subject as a single-id
    // override short-circuited `story validate` and discarded the scope, so a
    // `scope = "all"` check read one story. The single-story form is
    // `scope = "active"` with `[current].id` naming the story, which is what
    // `active` means.
    let out = crate::cmd::story::validate(ctx, None, &scope)?;
    let problems: Vec<String> = out
        .warnings
        .iter()
        .filter(|d| !d.is_warning())
        .map(|d| d.message.clone())
        .collect();

    let evidence = serde_yaml_ng::to_value(&out.data).unwrap_or(Value::Mapping(Default::default()));
    if problems.is_empty() {
        return Ok(Outcome::pass().with_evidence(evidence));
    }
    let n = problems.len();
    let findings = out
        .warnings
        .iter()
        .filter(|d| !d.is_warning())
        .map(|d| Finding {
            id: id.to_string(),
            severity: "block".to_string(),
            summary: cut(&d.message, 90),
            evidence: d.path.clone(),
            category: None,
        })
        .collect();
    Ok(Outcome::fail(format!(
        "DFA-E328 story validate found {n} problems; first: {}",
        problems[0]
    ))
    .with_evidence(evidence)
    .with_findings(findings))
}

/// `files_declared`: `story files --diff` exits 0 for the gate subject.
fn files_declared(ctx: &mut Ctx, c: &Check, id: &str) -> Result<Outcome, CliError> {
    // Spec 273 defines no skip for this kind: a subject the check cannot be run
    // for, and a root that is not a work tree, are both conditions the check
    // exists to refuse rather than reasons to count it as passing.
    if !id.starts_with("STORY-") {
        return Ok(Outcome::fail(format!(
            "DFA-E239 gate subject '{id}' is not a story, so no declared file set exists"
        )));
    }
    if !crate::git::is_work_tree(&ctx.root) {
        return Ok(Outcome::fail(format!(
            "DFA-E239 {} is not a git work tree, so the changed file set cannot be read",
            ctx.root.display()
        )));
    }
    let base = c.str_key("base", "");
    let args = crate::cli::StoryFilesArgs {
        check: None,
        list: false,
        diff: true,
        id: Some(id.to_string()),
        base: if base.is_empty() { None } else { Some(base) },
    };
    let out = crate::cmd::story::files(ctx, &args)?;
    let evidence = serde_yaml_ng::to_value(&out.data).unwrap_or(Value::Mapping(Default::default()));
    if out.exit.unwrap_or(0) == 0 {
        return Ok(Outcome::pass().with_evidence(evidence));
    }
    let n = out.warnings.len();
    let first = out
        .warnings
        .first()
        .map(|d| d.message.clone())
        .unwrap_or_default();
    let findings = out
        .warnings
        .iter()
        .map(|d| Finding {
            id: id.to_string(),
            severity: "block".to_string(),
            summary: cut(&d.message, 90),
            evidence: String::new(),
            category: None,
        })
        .collect();
    Ok(Outcome::fail(format!(
        "DFA-E239 {n} changed paths are undeclared; first: {first}"
    ))
    .with_evidence(evidence)
    .with_findings(findings))
}

/// `antipattern_clean`: `antipattern scan` exits 0 over the story's `## Files`
/// set with `scope = "story"`, and over `[[stack]].source_roots` with
/// `scope = "project"`. The error table names no code of its own for the
/// failure, so the reason carries `antipattern scan`'s own `DFA-E270`.
fn antipattern_clean(ctx: &mut Ctx, c: &Check, id: &str) -> Result<Outcome, CliError> {
    let min_severity = c.str_key("min_severity", "high");
    let scope = c.str_key("scope", "story");
    // `gates::parse_check` validates `scope` against its enum, so the two arms
    // below are the whole of it.
    let subject = if scope == "story" { Some(id) } else { None };
    if scope == "story" && !id.starts_with("STORY-") {
        return Ok(Outcome::fail(format!(
            "DFA-E270 gate subject '{id}' is not a story, so scope 'story' has no file set to scan"
        )));
    }

    let out = crate::cmd::antipattern::scan(ctx, subject, None, &min_severity)?;
    let evidence = serde_yaml_ng::to_value(&out.data).unwrap_or(Value::Mapping(Default::default()));
    if out.warnings.is_empty() {
        return Ok(Outcome::pass().with_evidence(evidence));
    }
    let n = out.warnings.len();
    let first = out
        .warnings
        .first()
        .map(|d| d.message.clone())
        .unwrap_or_default();
    let findings = out
        .warnings
        .iter()
        .map(|d| Finding {
            id: id.to_string(),
            severity: "block".to_string(),
            summary: cut(&d.message, 90),
            evidence: d.path.clone(),
            category: None,
        })
        .collect();
    Ok(Outcome::fail(format!(
        "DFA-E270 antipattern scan found {n} matches; first: {first}"
    ))
    .with_evidence(evidence)
    .with_findings(findings))
}

/// `design_tokens`: `design lint` exits 0 over the matched files, with
/// `paths` at `[]` meaning the frontend globs.
fn design_tokens(ctx: &mut Ctx, c: &Check) -> Result<Outcome, CliError> {
    let paths: Vec<std::path::PathBuf> = c
        .arr_key("paths", &[])
        .into_iter()
        .map(std::path::PathBuf::from)
        .collect();

    let out = match crate::cmd::design::lint(ctx, &paths, false) {
        Ok(o) => o,
        // An absent or unparsable token file is the Design skill's to write;
        // the check records the code rather than stopping the gate.
        Err(e) => {
            let Some(d) = e.diag() else { return Err(e) };
            if d.code == "DFA-E120" || d.code == "DFA-E121" {
                return Ok(Outcome::fail(format!("DFA-E329 {}", d.message)));
            }
            return Err(e);
        }
    };

    let evidence = serde_yaml_ng::to_value(&out.data).unwrap_or(Value::Mapping(Default::default()));
    if out.warnings.is_empty() {
        return Ok(Outcome::pass().with_evidence(evidence));
    }
    let n = out.warnings.len();
    let first = out
        .warnings
        .first()
        .map(|d| d.message.clone())
        .unwrap_or_default();
    let findings = out
        .warnings
        .iter()
        .map(|d| Finding {
            id: d.code.to_string(),
            severity: "block".to_string(),
            summary: cut(&d.message, 90),
            evidence: d.path.clone(),
            category: None,
        })
        .collect();
    Ok(Outcome::fail(format!(
        "DFA-E329 design lint found {n} problems; first: {first}"
    ))
    .with_evidence(evidence)
    .with_findings(findings))
}

// --------------------------------------------------- the command-running kinds

/// The `[[stack]]` tables a `stacks` key names, or every table when it is empty.
fn selected_stacks(cfg: &crate::config::Config, names: &[String]) -> Vec<crate::config::Stack> {
    if names.is_empty() {
        return cfg.stack.clone();
    }
    cfg.stack
        .iter()
        .filter(|s| names.iter().any(|n| n == &s.id))
        .cloned()
        .collect()
}

/// The reason an empty selection is a failure rather than a vacuous pass, or
/// `None` when the selection holds at least one stack.
fn empty_selection(
    cfg: &crate::config::Config,
    names: &[String],
    stacks: &[crate::config::Stack],
    code: &str,
    what: &str,
) -> Option<String> {
    if !stacks.is_empty() {
        return None;
    }
    if !names.is_empty() {
        let known: Vec<&str> = cfg.stack.iter().map(|s| s.id.as_str()).collect();
        return Some(format!(
            "{code} the {what} check names stacks {} and config.toml holds {}",
            names.join(", "),
            if known.is_empty() {
                "no [[stack]] table".to_string()
            } else {
                known.join(", ")
            }
        ));
    }
    Some(format!(
        "{code} config.toml holds no [[stack]] table, so the {what} check ran nothing"
    ))
}

/// One stack's run, under that stack's environment and wall-clock limit.
///
/// A command that cannot start is recorded as a failed run rather than raised,
/// so the report carries the four keys spec 929 names and says why the command
/// did not run.
fn run_for(
    root: &std::path::Path,
    stack: &crate::config::Stack,
    command: &str,
) -> Result<crate::run::Run, CliError> {
    match crate::run::run(
        root,
        command,
        &crate::run::stack_env(stack),
        stack.timeout_secs,
    ) {
        Ok(r) => Ok(r),
        Err(e) if e.code() == "DFA-E319" => Ok(crate::run::Run {
            command: command.to_string(),
            exit_code: -1,
            duration_ms: 0,
            output: e.diag().map(|d| d.message.clone()).unwrap_or_default(),
            timed_out: false,
        }),
        Err(e) => Err(e),
    }
}

/// `tests_pass`: every named stack's `test_command` exits 0.
fn tests_pass(ctx: &mut Ctx, c: &Check) -> Result<Outcome, CliError> {
    let root = ctx.root.clone();
    let allow_empty = c.bool_key("allow_empty", false);
    let names = c.arr_key("stacks", &[]);
    let cfg = ctx.config()?.clone();
    let stacks = selected_stacks(&cfg, &names);
    // An empty selection ran nothing, so there is nothing to pass. The two
    // causes read differently and the message says which one happened.
    if let Some(reason) = empty_selection(&cfg, &names, &stacks, "DFA-E310", "tests_pass") {
        return Ok(Outcome::fail(reason));
    }

    let mut runs: Vec<Value> = Vec::new();
    let mut failure: Option<String> = None;

    for stack in &stacks {
        if stack.test_command.trim().is_empty() {
            if allow_empty {
                continue;
            }
            return Ok(Outcome::fail(format!(
                "DFA-E310 stack '{}' has no test_command; set one in config.toml",
                stack.id
            )));
        }
        let run = run_for(&root, stack, &stack.test_command)?;
        let (passed, failed) = crate::run::test_counts(&stack.id, &run.output);
        let mut evidence = run.evidence();
        if let Value::Mapping(m) = &mut evidence {
            m.insert("stack".into(), stack.id.clone().into());
            m.insert("passed".into(), count_value(passed));
            m.insert("failed".into(), count_value(failed));
        }
        runs.push(evidence);

        if run.timed_out && failure.is_none() {
            failure = Some(format!(
                "DFA-E318 command '{}' exceeded {}s and was terminated",
                run.command, stack.timeout_secs
            ));
        } else if !run.ok() && failure.is_none() {
            failure = Some(format!(
                "DFA-E311 test command '{}' exited {} for stack '{}'",
                run.command, run.exit_code, stack.id
            ));
        }
    }

    let evidence = runs_evidence(runs);
    match failure {
        Some(reason) => Ok(Outcome::fail(reason).with_evidence(evidence)),
        None => Ok(Outcome::pass().with_evidence(evidence)),
    }
}

/// The two command kinds that share one shape.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Commanded {
    Lint,
    Complexity,
}

impl Commanded {
    fn command<'a>(&self, s: &'a crate::config::Stack) -> &'a str {
        match self {
            Commanded::Lint => &s.lint_command,
            Commanded::Complexity => &s.complexity_command,
        }
    }
    /// The `reason` of a skip over an empty command string.
    fn empty_reason(&self) -> &'static str {
        match self {
            Commanded::Lint => "no_lint_command",
            Commanded::Complexity => "no_complexity_command",
        }
    }
    fn word(&self) -> &'static str {
        match self {
            Commanded::Lint => "lint",
            Commanded::Complexity => "complexity",
        }
    }
    /// The `DFA-` code of a non-zero exit. Every failing reason opens with a
    /// code, because that prefix is what reaches the envelope's `errors[]`; a
    /// reason without one is silent there.
    fn code(&self) -> &'static str {
        match self {
            Commanded::Lint => "DFA-E315",
            Commanded::Complexity => "DFA-E349",
        }
    }
    /// The `DFA-` code of a selection that names no stack this project holds.
    fn empty_selection_code(&self) -> &'static str {
        match self {
            Commanded::Lint => "DFA-E314",
            Commanded::Complexity => "DFA-E349",
        }
    }
}

/// `lint_clean` and `complexity_clean`: an empty command is a skip, a non-zero
/// exit is a failure.
fn command_check(ctx: &mut Ctx, c: &Check, kind: Commanded) -> Result<Outcome, CliError> {
    let root = ctx.root.clone();
    let names = c.arr_key("stacks", &[]);
    let cfg = ctx.config()?.clone();
    let stacks = selected_stacks(&cfg, &names);
    // A selection that names no stack this project holds is a defect in
    // gates.toml or config.toml, not a project with no command to run. Spec 314
    // grants the skip to a stack that exists and carries an empty command, and
    // the two are distinguished below.
    if let Some(reason) = empty_selection(
        &cfg,
        &names,
        &stacks,
        kind.empty_selection_code(),
        kind.word(),
    ) {
        return Ok(Outcome::fail(reason));
    }

    let mut runs: Vec<Value> = Vec::new();
    let mut failure: Option<String> = None;
    let mut ran = 0usize;

    for stack in &stacks {
        let command = kind.command(stack).to_string();
        if command.trim().is_empty() {
            continue;
        }
        ran += 1;
        let run = run_for(&root, stack, &command)?;
        let mut evidence = run.evidence();
        if let Value::Mapping(m) = &mut evidence {
            m.insert("stack".into(), stack.id.clone().into());
        }
        runs.push(evidence);

        if run.timed_out && failure.is_none() {
            failure = Some(format!(
                "DFA-E318 command '{}' exceeded {}s and was terminated",
                run.command, stack.timeout_secs
            ));
        } else if !run.ok() && failure.is_none() {
            failure = Some(format!(
                "{} {} command '{}' exited {} for stack '{}'",
                kind.code(),
                kind.word(),
                run.command,
                run.exit_code,
                stack.id
            ));
        }
    }

    if ran == 0 {
        return Ok(Outcome::skip(kind.empty_reason()));
    }
    let evidence = runs_evidence(runs);
    match failure {
        Some(reason) => Ok(Outcome::fail(reason).with_evidence(evidence)),
        None => Ok(Outcome::pass().with_evidence(evidence)),
    }
}

/// A count as the report records it: the number, or null when the stack's
/// pattern did not match.
fn count_value(n: Option<u64>) -> Value {
    match n {
        Some(v) => Value::Number(v.into()),
        None => Value::Null,
    }
}

/// One run is its own mapping; two or more are a `runs` sequence.
fn runs_evidence(mut runs: Vec<Value>) -> Value {
    match runs.len() {
        0 => Value::Mapping(Default::default()),
        1 => runs.remove(0),
        _ => {
            let mut m = serde_yaml_ng::Mapping::new();
            m.insert("runs".into(), Value::Sequence(runs));
            Value::Mapping(m)
        }
    }
}

/// `coverage_min`: every named layer, and the overall figure, at or above the
/// thresholds `config.toml` carries.
fn coverage_min(ctx: &mut Ctx, c: &Check, no_run: bool) -> Result<Outcome, CliError> {
    use crate::coverage;

    let root = ctx.root.clone();
    let cfg = ctx.config()?.clone();
    let source = c.str_key("source", "run");
    // `--no-run` executes no command, so the check parses the artifact already
    // on disk, which is what `source = "read"` does.
    let read_only = no_run || source == "read";
    let want_overall = c.bool_key("overall", true);
    let named = c.arr_key("layers", &[]);

    if !read_only {
        for stack in &cfg.stack {
            if stack.coverage_command.trim().is_empty() {
                continue;
            }
            let run = run_for(&root, stack, &stack.coverage_command)?;
            if run.timed_out {
                return Ok(Outcome::fail(format!(
                    "DFA-E318 command '{}' exceeded {}s and was terminated",
                    run.command, stack.timeout_secs
                )));
            }
            // A coverage command that failed did not regenerate the artifact,
            // so whatever `newest_match` finds is yesterday's.
            if !run.ok() {
                return Ok(Outcome::fail(format!(
                    "DFA-E312 coverage command '{}' exited {} for stack '{}', so no coverage was produced",
                    run.command, run.exit_code, stack.id
                ))
                .with_evidence(run.evidence()));
            }
        }
    }

    // The artifact: the most recently modified match of every stack's globs.
    let patterns: Vec<String> = cfg
        .stack
        .iter()
        .filter(|s| s.coverage_format != "none")
        .flat_map(|s| s.coverage_paths.clone())
        .collect();
    // No skip here: spec 282 makes an absent artifact `DFA-E312` rather than a
    // skip, so a story cannot reach Release with no coverage evidence. A
    // project with no stack at all is already skipped by the degradation rule
    // before this function runs.
    let Some(path) = coverage::newest_match(&root, &patterns) else {
        return Ok(Outcome::fail(if patterns.is_empty() {
            "DFA-E312 no stack declares coverage_paths, so there is no coverage artifact to read"
                .to_string()
        } else {
            format!(
                "DFA-E312 coverage report not found at {}",
                patterns.join(", ")
            )
        }));
    };

    let rel = crate::project::rel_display(&root, &path);
    let format = cfg
        .stack
        .iter()
        .find(|s| {
            s.coverage_paths.iter().any(|p| {
                globset::Glob::new(p)
                    .map(|g| g.compile_matcher().is_match(&rel))
                    .unwrap_or(false)
            })
        })
        .map(|s| s.coverage_format.clone())
        .unwrap_or_else(|| "lcov".to_string());

    let text = crate::project::read_doc(&path)?;
    let parsed = match coverage::parse(&format, &path, &text) {
        Ok(f) => f,
        Err(e) => {
            let message = e
                .diag()
                .map(|d| format!("{} {}", d.code, d.message))
                .unwrap_or_else(|| e.code().to_string());
            return Ok(Outcome::fail(message));
        }
    };

    let files: Vec<coverage::FileCoverage> = parsed
        .into_iter()
        .map(|f| coverage::FileCoverage {
            path: coverage::normalise(&root, &f.path),
            covered: f.covered,
            total: f.total,
        })
        .collect();

    let roll = coverage::roll_up(&cfg, &files);

    // A `layers` name matching no layer is a defect in gates.toml: dropping it
    // silently left the check judging fewer layers than it names.
    for want in &named {
        if !roll.layers.iter().any(|l| &l.name == want) {
            return Ok(Outcome::fail(format!(
                "DFA-E312 gates.toml check '{}' names layer '{want}', which config.toml does not define",
                c.id
            )));
        }
    }

    let mut failures: Vec<String> = Vec::new();
    for layer in &roll.layers {
        if !named.is_empty() && !named.iter().any(|n| n == &layer.name) {
            continue;
        }
        if layer.status == "fail" {
            failures.push(format!(
                "DFA-E313 coverage for layer '{}' is {}%, below the {}% in config.toml",
                layer.name, layer.percent, layer.min
            ));
        }
    }
    if want_overall && roll.overall + f64::EPSILON < cfg.coverage.overall_min {
        failures.push(format!(
            "DFA-E313 coverage for layer 'overall' is {}%, below the {}% in config.toml",
            roll.overall, cfg.coverage.overall_min
        ));
    }
    if roll.unassigned_files > 0 && cfg.coverage.unassigned_policy == "fail" {
        failures.push(format!(
            "DFA-E313 {} covered files match no layer glob",
            roll.unassigned_files
        ));
    }

    let block = serde_yaml_ng::to_value(serde_json::json!({
        "format": format,
        "source": rel,
        "overall": roll.overall,
        "layers": roll.layers.iter().map(|l| serde_json::json!({
            "name": l.name,
            "covered": l.covered,
            "total": l.total,
            "percent": l.percent,
            "min": l.min,
            "status": l.status,
        })).collect::<Vec<_>>(),
        "unassigned": {
            "files": roll.unassigned_files,
            "percent": roll.unassigned_percent,
        },
    }))
    .unwrap_or(Value::Mapping(Default::default()));

    let evidence = serde_yaml_ng::to_value(serde_json::json!({
        "source": rel,
        "format": format,
        "overall": roll.overall,
        "excluded": roll.excluded,
    }))
    .unwrap_or(Value::Mapping(Default::default()));

    let outcome = match failures.first() {
        Some(first) => Outcome::fail(first.clone()),
        None if roll.unassigned_files > 0 => Outcome {
            status: "pass",
            reason: format!(
                "DFA-W320 {} covered files match no layer glob",
                roll.unassigned_files
            ),
            evidence: Value::Mapping(Default::default()),
            findings: Vec::new(),
            coverage: None,
        },
        None => Outcome::pass(),
    };
    Ok(outcome.with_evidence(evidence).with_coverage(block))
}

// ------------------------------------------------ the release-facing kinds

/// The five platform values `deploy_manifest` dispatches on.
const PLATFORMS: &[&str] = &["kubernetes", "compose", "github-actions", "vps", "none"];

/// The secret pattern, applied line by line to every listed manifest.
fn secret_pattern() -> regex::Regex {
    regex::Regex::new(
        r#"(?i)(password|secret|token|api[_-]?key|private[_-]?key)\s*[:=]\s*["']?[A-Za-z0-9+/=_-]{12,}"#,
    )
    .expect("the secret pattern compiles")
}

/// A line the secret pattern matches that is not a reference or a substitution.
fn is_literal_secret(re: &regex::Regex, line: &str) -> bool {
    if !re.is_match(line) {
        return false;
    }
    // The exclusions apply to the value, not to the whole line: a trailing
    // comment that names a reference excused the literal beside it.
    let value = match line.split_once('#') {
        Some((before, _)) => before,
        None => line,
    };
    if !re.is_match(value) {
        return false;
    }
    !(value.contains("${")
        || value.contains("$(")
        || value.contains("secretKeyRef")
        || value.contains("valueFrom"))
}

/// `DFA-E342`, the one message shape the spec gives this kind.
fn e342(path: &str, what: &str, platform: &str) -> String {
    format!("DFA-E342 {path} {what} for platform '{platform}'")
}

/// Walk a dotted path through a YAML value, taking `[n]` as a sequence index.
fn dig<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut at = value;
    for raw in path.split('.') {
        let (key, index) = match raw.split_once('[') {
            Some((k, rest)) => (k, rest.trim_end_matches(']').parse::<usize>().ok()),
            None => (raw, None),
        };
        if !key.is_empty() {
            at = at.get(key)?;
        }
        if let Some(i) = index {
            at = at.as_sequence()?.get(i)?;
        }
    }
    Some(at)
}

/// `deploy_manifest`: the platform rules hold for every listed manifest.
fn deploy_manifest(ctx: &mut Ctx, c: &Check, id: &str, phase: &str) -> Result<Outcome, CliError> {
    let rel = substitute(&c.str_key("path", "releases/{id}.yaml"), id, phase);
    let (_, release) = load_value(ctx, &rel)?;

    let configured = ctx.config()?.release.platform.clone();
    let platform = match c.str_key("platform", "") {
        p if !p.is_empty() => p,
        _ if !configured.is_empty() => configured,
        _ => dig(&release, "platform.target")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    };
    if !PLATFORMS.contains(&platform.as_str()) {
        return Ok(Outcome::fail(e342(
            &rel,
            "has no platform.target",
            &platform,
        )));
    }

    // An entry is a mapping carrying `path`, or a bare string. Dropping the
    // shapes it does not recognise made a non-empty list read as empty, which
    // the `none` rule below then accepted.
    let mut manifests: Vec<String> = Vec::new();
    if let Some(seq) = dig(&release, "deploy.manifests").and_then(|v| v.as_sequence()) {
        for (at, e) in seq.iter().enumerate() {
            match e {
                Value::String(s) => manifests.push(s.clone()),
                other => match other.get("path").and_then(|p| p.as_str()) {
                    Some(p) => manifests.push(p.to_string()),
                    None => {
                        return Ok(Outcome::fail(e342(
                            &rel,
                            &format!("deploy.manifests entry {at} names no path"),
                            &platform,
                        )))
                    }
                },
            }
        }
    }
    let rollback = dig(&release, "deploy.rollback")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let ci_workflow = dig(&release, "deploy.ci_workflow")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let root = ctx.root.clone();
    let evidence = serde_yaml_ng::to_value(serde_json::json!({
        "platform": platform,
        "manifests": manifests.len(),
    }))
    .unwrap_or(Value::Mapping(Default::default()));

    // `none` is a rule of its own rather than a skip.
    if platform == "none" {
        if !manifests.is_empty() {
            return Ok(Outcome::fail(e342(&rel, "lists manifests", &platform)));
        }
        if !rollback.is_empty() {
            return Ok(Outcome::fail(e342(
                &rel,
                "names a rollback note",
                &platform,
            )));
        }
        return Ok(Outcome::pass().with_evidence(evidence));
    }

    // Every listed path exists, every `.yaml` parses, and no line holds a
    // literal secret.
    let re = secret_pattern();
    let mut parsed: BTreeMap<String, Value> = BTreeMap::new();
    for m in &manifests {
        let path = root.join(m);
        if !path.exists() {
            return Ok(Outcome::fail(e342(m, "is absent", &platform)));
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Ok(Outcome::fail(e342(m, "is absent", &platform)));
        };
        for (i, line) in text.lines().enumerate() {
            if is_literal_secret(&re, line) {
                return Ok(Outcome::fail(format!(
                    "DFA-E343 {m}:{} holds a literal secret",
                    i + 1
                )));
            }
        }
        if m.ends_with(".yaml") || m.ends_with(".yml") {
            match serde_yaml_ng::from_str::<Value>(&text) {
                Ok(v) => {
                    parsed.insert(m.clone(), v);
                }
                Err(_) => return Ok(Outcome::fail(e342(m, "does not parse", &platform))),
            }
        }
    }

    let named = |stem: &str| -> Option<(&String, &Value)> {
        parsed
            .iter()
            .find(|(k, _)| k.rsplit('/').next() == Some(stem))
    };
    let count_named = |stem: &str| -> usize {
        manifests
            .iter()
            .filter(|k| k.rsplit('/').next() == Some(stem))
            .count()
    };

    let failure: Option<String> = match platform.as_str() {
        "kubernetes" => {
            let mut problem = None;
            for stem in [
                "deployment.yaml",
                "service.yaml",
                "ingress.yaml",
                "kustomization.yaml",
            ] {
                match count_named(stem) {
                    1 => {}
                    0 => {
                        problem = Some(e342(&rel, &format!("has no {stem}"), &platform));
                        break;
                    }
                    n => {
                        problem = Some(e342(
                            &rel,
                            &format!("lists {n} manifests named {stem}, expected exactly 1"),
                            &platform,
                        ));
                        break;
                    }
                }
            }
            problem.or_else(|| {
                let (path, doc) = named("deployment.yaml")?;
                for key in [
                    "spec.template.spec.containers[0].image",
                    "spec.template.spec.containers[0].readinessProbe",
                    "spec.template.spec.containers[0].livenessProbe",
                    "spec.template.spec.containers[0].resources.limits",
                ] {
                    if dig(doc, key).is_none() {
                        return Some(e342(path, &format!("has no {key}"), &platform));
                    }
                }
                None
            })
        }
        "compose" => {
            let compose = named("docker-compose.yaml").or_else(|| named("docker-compose.yml"));
            match compose {
                None => Some(e342(&rel, "has no docker-compose.yaml", &platform)),
                Some((path, doc)) => {
                    let services = doc.get("services").and_then(|v| v.as_mapping());
                    match services {
                        None => Some(e342(path, "has no services", &platform)),
                        Some(map) if map.is_empty() => {
                            Some(e342(path, "has no services", &platform))
                        }
                        Some(map) => {
                            let bad = map.iter().find(|(_, v)| {
                                v.get("image").is_none() && v.get("build").is_none()
                            });
                            match bad {
                                Some(_) => Some(e342(path, "has no image", &platform)),
                                None => env_example(&root, &manifests, path, &platform),
                            }
                        }
                    }
                }
            }
        }
        "github-actions" => {
            let workflow = root.join(".github/workflows/deploy.yml");
            if !workflow.exists() {
                Some(e342(".github/workflows/deploy.yml", "is absent", &platform))
            } else {
                let text = std::fs::read_to_string(&workflow).unwrap_or_default();
                match serde_yaml_ng::from_str::<Value>(&text) {
                    Err(_) => Some(e342(
                        ".github/workflows/deploy.yml",
                        "does not parse",
                        &platform,
                    )),
                    Ok(doc) => {
                        // YAML 1.1 reads a bare `on:` key as the boolean true,
                        // which is what a GitHub workflow file carries.
                        let has_on = doc.get("on").is_some()
                            || doc
                                .as_mapping()
                                .map(|m| m.contains_key(Value::Bool(true)))
                                .unwrap_or(false);
                        let jobs = doc.get("jobs").and_then(|v| v.as_mapping());
                        if !has_on {
                            Some(e342(".github/workflows/deploy.yml", "has no on", &platform))
                        } else if jobs.is_none() {
                            Some(e342(
                                ".github/workflows/deploy.yml",
                                "has no jobs",
                                &platform,
                            ))
                        } else if ci_workflow.is_empty() {
                            None
                        } else {
                            let needs_gate = jobs
                                .map(|m| {
                                    m.values().any(|j| {
                                        let needs = j.get("needs");
                                        match needs {
                                            Some(Value::String(one)) => {
                                                one == "devforgeai-release-gate"
                                            }
                                            Some(Value::Sequence(many)) => many.iter().any(|n| {
                                                n.as_str() == Some("devforgeai-release-gate")
                                            }),
                                            _ => false,
                                        }
                                    })
                                })
                                .unwrap_or(false);
                            if needs_gate {
                                None
                            } else {
                                Some(e342(
                                    ".github/workflows/deploy.yml",
                                    "has no needs devforgeai-release-gate",
                                    &platform,
                                ))
                            }
                        }
                    }
                }
            }
        }
        "vps" => {
            let script = manifests
                .iter()
                .find(|m| m.ends_with("vps/deploy.sh"))
                .cloned()
                .unwrap_or_else(|| "vps/deploy.sh".to_string());
            let unit = manifests
                .iter()
                .find(|m| m.ends_with("vps/app.service"))
                .cloned()
                .unwrap_or_else(|| "vps/app.service".to_string());

            let body = std::fs::read_to_string(root.join(&script)).unwrap_or_default();
            let first = body.lines().next().unwrap_or("").trim();
            if body.trim().is_empty() {
                Some(e342(&script, "is absent", &platform))
            } else if first != "#!/bin/sh" && first != "#!/usr/bin/env sh" {
                Some(e342(&script, "has no shebang", &platform))
            } else {
                let service = std::fs::read_to_string(root.join(&unit)).unwrap_or_default();
                if service.contains("ExecStart=") {
                    None
                } else {
                    Some(e342(&unit, "has no ExecStart=", &platform))
                }
            }
        }
        _ => None,
    };

    if let Some(reason) = failure {
        return Ok(Outcome::fail(reason).with_evidence(evidence));
    }

    // The secret scan reaches every file the platform rules read, not only the
    // entries of `deploy.manifests[]`: the workflow, the vps script, and the
    // vps unit ship the same secrets and were never scanned.
    let mut extra: Vec<String> = Vec::new();
    match platform.as_str() {
        "github-actions" => extra.push(".github/workflows/deploy.yml".to_string()),
        "vps" => {
            extra.push("vps/deploy.sh".to_string());
            extra.push("vps/app.service".to_string());
            extra.extend(
                manifests
                    .iter()
                    .filter(|m| m.ends_with("vps/deploy.sh") || m.ends_with("vps/app.service"))
                    .cloned(),
            );
        }
        _ => {}
    }
    extra.sort();
    extra.dedup();
    for f in &extra {
        if manifests.contains(f) {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(root.join(f)) else {
            continue;
        };
        for (i, line) in text.lines().enumerate() {
            if is_literal_secret(&re, line) {
                return Ok(
                    Outcome::fail(format!("DFA-E343 {f}:{} holds a literal secret", i + 1))
                        .with_evidence(evidence),
                );
            }
        }
    }

    Ok(Outcome::pass().with_evidence(evidence))
}

/// The compose rule over `env.example`: every `${NAME}` has a `NAME=` line.
fn env_example(
    root: &std::path::Path,
    manifests: &[String],
    compose_path: &str,
    platform: &str,
) -> Option<String> {
    let example = manifests
        .iter()
        .find(|m| m.ends_with("env.example"))
        .cloned()
        .unwrap_or_else(|| {
            let dir = compose_path.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
            if dir.is_empty() {
                "env.example".to_string()
            } else {
                format!("{dir}/env.example")
            }
        });
    let Ok(env) = std::fs::read_to_string(root.join(&example)) else {
        return Some(e342(&example, "is absent", platform));
    };
    let Ok(compose) = std::fs::read_to_string(root.join(compose_path)) else {
        return Some(e342(compose_path, "is absent", platform));
    };
    let re =
        regex::Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)").expect("the reference rule compiles");
    for caps in re.captures_iter(&compose) {
        let name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let declared = env.lines().any(|l| {
            l.trim_start()
                .trim_start_matches('#')
                .trim_start()
                .starts_with(&format!("{name}="))
        });
        if !declared {
            return Some(e342(&example, &format!("has no {name}="), platform));
        }
    }
    None
}

/// `docs_cover`: every public symbol has an H3 heading in one of `docs.api[]`.
fn docs_cover(ctx: &mut Ctx, c: &Check, id: &str, phase: &str) -> Result<Outcome, CliError> {
    let rel = substitute(&c.str_key("path", "releases/{id}.yaml"), id, phase);
    let (_, release) = load_value(ctx, &rel)?;
    let min_ratio = c.float_key("min_ratio", 1.0);

    let configured = ctx.config()?.release.api_symbols_command.clone();
    let command = match c.str_key("command", "") {
        v if !v.is_empty() => v,
        _ => configured,
    };
    if command.trim().is_empty() {
        return Ok(Outcome::skip("no_api_symbols_command"));
    }

    let root = ctx.root.clone();
    let stack = ctx.config()?.stack.first().cloned().unwrap_or_default();
    let run = crate::run::run(
        &root,
        &command,
        &crate::run::stack_env(&stack),
        stack.timeout_secs.max(1),
    )?;
    if run.timed_out {
        return Ok(Outcome::fail(format!(
            "DFA-E318 command '{}' exceeded {}s and was terminated",
            run.command, stack.timeout_secs
        )));
    }

    // A command that exited non-zero printed no symbol list; parsing its output
    // read a failure as an empty API and passed the check.
    if !run.ok() {
        return Ok(Outcome::fail(format!(
            "DFA-E344 api symbols command '{}' exited {}",
            run.command, run.exit_code
        ))
        .with_evidence(run.evidence()));
    }

    // `<kind>\t<symbol>\t<path>`: the three tab-separated columns are fixed, so
    // a line with no tab is not a symbol. Taking it as one turned a deprecation
    // warning on stderr into a phantom symbol.
    let symbols: Vec<String> = run
        .output
        .lines()
        // Windows tooling writes CRLF, and `lines` leaves the `\r` on the end
        // of the last column. The line is still the three tab-separated
        // columns once it is off.
        .map(|l| l.trim_end_matches('\r').trim())
        .filter(|l| !l.is_empty())
        .filter_map(|l| {
            let parts: Vec<&str> = l.split('\t').collect();
            if parts.len() >= 2 {
                Some(parts[1].trim().to_string())
            } else {
                None
            }
        })
        .filter(|s| !s.is_empty())
        .collect();
    // The free pass belongs to an empty `command`, not to a command that ran
    // and printed nothing.
    if symbols.is_empty() {
        return Ok(Outcome::fail(format!(
            "DFA-E344 api symbols command '{}' printed no symbol line",
            run.command
        ))
        .with_evidence(run.evidence()));
    }

    let pages: Vec<String> = dig(&release, "docs.api")
        .and_then(|v| v.as_sequence())
        .map(|s| {
            s.iter()
                .filter_map(|p| p.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let mut headings: BTreeSet<String> = BTreeSet::new();
    for page in &pages {
        let Ok(text) = std::fs::read_to_string(root.join(page)) else {
            continue;
        };
        for h in crate::md::h3(&text) {
            headings.insert(h.trim().to_string());
        }
    }

    // An H3 heading covers a symbol when it names it, not when it contains it:
    // `### budget_getter` does not document `get`.
    let missing: Vec<&String> = symbols
        .iter()
        .filter(|sym| !headings.iter().any(|h| h.trim() == sym.as_str()))
        .collect();

    let total = symbols.len();
    let covered = total - missing.len();
    let ratio = if total == 0 {
        1.0
    } else {
        covered as f64 / total as f64
    };

    let evidence = serde_yaml_ng::to_value(serde_json::json!({
        "command": command,
        "symbols": total,
        "covered": covered,
        "ratio": (ratio * 1000.0).round() / 1000.0,
        "pages": pages.len(),
    }))
    .unwrap_or(Value::Mapping(Default::default()));

    if ratio + f64::EPSILON >= min_ratio {
        return Ok(Outcome::pass().with_evidence(evidence));
    }
    let first: Vec<String> = missing.iter().take(5).map(|s| (*s).clone()).collect();
    Ok(Outcome::fail(format!(
        "DFA-E344 {} of {total} public symbols have no H3 heading in docs.api[]: {}",
        missing.len(),
        first.join(", ")
    ))
    .with_evidence(evidence))
}

fn missing_path(check_id: &str, path: &str, doc: &str) -> CliError {
    CliError::at(
        "DFA-E345",
        format!("gates.toml check '{check_id}': path '{path}' does not resolve in {doc}"),
        ".devforgeai/gates.toml",
    )
}

fn map(pairs: &[(&str, Value)]) -> Value {
    let mut m = serde_yaml_ng::Mapping::new();
    for (k, v) in pairs {
        m.insert(Value::String((*k).to_string()), v.clone());
    }
    Value::Mapping(m)
}

fn cut(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        return s.to_string();
    }
    s.chars().take(n).collect()
}

/// What `gate require` resolved.
#[derive(Debug, Clone)]
pub struct Required {
    /// The phase asked about.
    pub phase: String,
    /// The subject asked about.
    pub id: String,
    /// The predecessor phase, or `""`.
    pub requires: String,
    /// The subject of the predecessor gate.
    pub subject: String,
    /// The reports read.
    pub reports: Vec<String>,
    /// PASS when every report read carries `gate.result: PASS`.
    pub result: String,
    /// Warnings the resolution raised without refusing the call.
    pub warnings: Vec<Diag>,
}

/// The id shape each `gate require` arm accepts.
///
/// The spec's per-arm table is normative and contradicts its own generic
/// id-grammar sentence: an argument whose prefix belongs to another arm is
/// `DFA-E013` before any predecessor is resolved, rather than a report path
/// that cannot exist. `reflect` is the one arm that fixes no shape, because it
/// returns before the id is used.
enum ArmId {
    /// One of these ID prefixes.
    Prefix(&'static [&'static str], &'static str),
    /// A `vX.Y.Z` version.
    Version,
    /// Any of the grammar's shapes; the arm does not read the id.
    Any,
}

fn require_id_shape(phase: &str) -> ArmId {
    match phase {
        "explore" | "discover" | "constitute" => ArmId::Prefix(&["IDEA"], "an IDEA-nnn"),
        "plan" => ArmId::Prefix(&["EPIC", "SPRINT"], "an EPIC-nnn or a SPRINT-nnn"),
        "build" | "verify" => ArmId::Prefix(&["STORY"], "a STORY-nnn"),
        "release" => ArmId::Version,
        // Design holds no gate, so `gate_for` refuses it with `DFA-E300`
        // before this table is consulted; the row records the shape the
        // subject would carry if it ever did.
        "design" => ArmId::Prefix(&["UI"], "a UI-nnn"),
        _ => ArmId::Any,
    }
}

/// `DFA-E013` when `id` does not carry the shape `phase`'s arm accepts.
fn check_arm_id(phase: &str, id: &str) -> Result<(), CliError> {
    let (ok, expected) = match require_id_shape(phase) {
        ArmId::Any => return Ok(()),
        ArmId::Version => (doc::is_version(id), "a version such as v1.0.0"),
        ArmId::Prefix(prefixes, expected) => (
            ids::split_id(id)
                .map(|(p, _)| prefixes.contains(&p))
                .unwrap_or(false),
            expected,
        ),
    };
    if ok {
        return Ok(());
    }
    Err(CliError::new(
        "DFA-E013",
        format!("'{id}' is not an ID for phase '{phase}'; expected {expected}"),
    ))
}

/// `gate require <phase> <id>`: resolve the predecessor's subject through the
/// chain the spec fixes, then read that report.
pub fn require(ctx: &mut Ctx, phase: &str, id: &str) -> Result<Required, CliError> {
    // Design is not a phase and holds no gate.
    let gate = ctx.gates()?.gate_for(phase)?.clone();
    // The prefix belongs to the arm before anything is resolved.
    check_arm_id(phase, id)?;

    let mut out = Required {
        phase: phase.to_string(),
        id: id.to_string(),
        requires: String::new(),
        subject: String::new(),
        reports: Vec::new(),
        result: "PASS".to_string(),
        warnings: Vec::new(),
    };

    // The `gate require` table is normative per phase for subject and report
    // resolution; the `requires = ""` short-circuit applies where that table
    // says "none; exit 0", which is explore and reflect.
    match phase {
        "explore" | "reflect" => return Ok(out),
        "discover" => {
            // The predecessor is explore when a decision exists with that id.
            // The three cases are distinct: an absent file means Discover is an
            // entry phase, a parsable file naming another idea means the same,
            // and an unparsable one is a defect in the document that `explore
            // decide` reports as DFA-E401 and this call must not read as "no
            // explore ran". `doc::read_decision` is the single implementation,
            // so `gate require` and `explore decide` cannot disagree about the
            // file again.
            let matches = match doc::read_decision(&ctx.root)? {
                None => false,
                Some(v) => {
                    v.get(Value::String("id".into()))
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        == id
                }
            };
            if !matches {
                return Ok(out);
            }
            out.requires = "explore".into();
            out.subject = id.to_string();
        }
        "constitute" => {
            out.requires = "discover".into();
            out.subject = id.to_string();
        }
        "plan" => {
            out.requires = "constitute".into();
            out.subject = resolve_plan_subject(ctx, id)?;
        }
        "build" => {
            out.requires = "plan".into();
            out.subject = resolve_build_subject(ctx, id)?;
        }
        "verify" => {
            out.requires = "build".into();
            out.subject = id.to_string();
        }
        "release" => {
            out.requires = "verify".into();
            return require_release(ctx, id, out);
        }
        _ => {
            if gate.requires.is_empty() {
                return Ok(out);
            }
            out.requires = gate.requires.clone();
            out.subject = id.to_string();
        }
    }

    let path = report::report_path(&ctx.root, &out.subject, &out.requires);
    out.reports.push(ctx.rel(&path));
    check_report_pass(ctx, &path, phase, &out.requires, &out.subject)?;
    Ok(out)
}

fn check_report_pass(
    ctx: &Ctx,
    path: &std::path::Path,
    phase: &str,
    requires: &str,
    subject: &str,
) -> Result<(), CliError> {
    match report::load(path) {
        // A kind outside this milestone is recorded `skip`, and a skip counts
        // as passing, so a report can read PASS while `gate check` refused at
        // exit 5. The result enum holds no value for "never ran", so the
        // refusal lives here rather than in what the report says.
        Ok(r)
            if r.gate.result == "PASS"
                && r.gate
                    .checks
                    .iter()
                    .any(|c| c.reason == report::NOT_IMPLEMENTED_REASON) =>
        {
            let kinds: BTreeSet<&str> = r
                .gate
                .checks
                .iter()
                .filter(|c| c.reason == report::NOT_IMPLEMENTED_REASON)
                .map(|c| c.kind.as_str())
                .collect();
            let kinds: Vec<String> = kinds.iter().map(|k| format!("'{k}'")).collect();
            Err(CliError::at(
                "DFA-E321",
                format!(
                    "{phase} needs {requires} gate PASS for {subject}; the {requires} report \
                     reads PASS only because check kind {} is not implemented in this build",
                    kinds.join(", ")
                ),
                ctx.rel(path),
            ))
        }
        Ok(r) if r.gate.result == "PASS" => Ok(()),
        Ok(r) => Err(CliError::at(
            "DFA-E321",
            format!(
                "{phase} needs {requires} gate PASS for {subject}; the last result is {} at {}",
                r.gate.result, r.finished_at
            ),
            ctx.rel(path),
        )),
        Err(e) if e.code() == "DFA-E400" => Err(CliError::at(
            "DFA-E321",
            format!(
                "{phase} needs {requires} gate PASS for {subject}; no report at {}",
                ctx.rel(path)
            ),
            ctx.rel(path),
        )),
        Err(e) => Err(e),
    }
}

fn require_release(ctx: &mut Ctx, id: &str, mut out: Required) -> Result<Required, CliError> {
    let rel = format!("releases/{id}.yaml");
    // A release that verifies nothing is the condition this arm exists to
    // refuse, so the result starts at FAIL and only a story that passed can
    // raise it.
    out.result = "FAIL".to_string();

    // The release file is written during the run, not before it: Release runs
    // `phase set release --id <version>` at step 2 and writes
    // `releases/<version>.yaml` at step 13. Refusing the absent file with
    // `DFA-E200` stopped a first release before it began, so an absent file is
    // the first-release case: it warns, and the story set comes from the sprint
    // instead, which keeps the verify gates binding. A file that is present and
    // does not parse is a defect rather than an absence and still refuses.
    let ids = if ctx.doc_path(&rel).exists() {
        let (_, value) = load_value(ctx, &rel)?;
        release_story_ids(&value, &rel)?
    } else {
        out.warnings.push(Diag::at(
            "DFA-W210",
            format!("status not written; .devforgeai/{rel} not found"),
            format!(".devforgeai/{rel}"),
        ));
        sprint_story_ids(ctx)
    };

    out.subject = ids.join(", ");
    // With no release file and a sprint that names nobody, nothing was
    // verified: the call still exits 0 with its warning, and the result stays
    // FAIL, so `phase set`'s empty-report guard refuses to advance on it.
    if ids.is_empty() {
        return Ok(out);
    }
    for sid in &ids {
        let path = report::report_path(&ctx.root, sid, "verify");
        out.reports.push(ctx.rel(&path));
        check_report_pass(ctx, &path, "release", "verify", sid)?;
    }
    out.result = "PASS".to_string();
    Ok(out)
}

/// The `stories[].id` of a release document, or the `DFA-E321` naming what the
/// document holds instead. A release that names no story is the condition this
/// arm exists to refuse.
fn release_story_ids(value: &Value, rel: &str) -> Result<Vec<String>, CliError> {
    let e321 = |what: &str| {
        CliError::at(
            "DFA-E321",
            format!("release needs a verify gate PASS for every story; {rel} {what}"),
            rel.to_string(),
        )
    };
    let Some(stories) = value.get(Value::String("stories".into())) else {
        return Err(e321("has no stories key"));
    };
    let Some(stories) = stories.as_sequence() else {
        return Err(e321("has a stories key that is not a sequence"));
    };
    if stories.is_empty() {
        return Err(e321("names no story"));
    }
    let mut ids: Vec<String> = Vec::new();
    for (at, s) in stories.iter().enumerate() {
        match s.get(Value::String("id".into())).and_then(Value::as_str) {
            Some(sid) => ids.push(sid.to_string()),
            None => return Err(e321(&format!("entry {at} carries no id"))),
        }
    }
    Ok(ids)
}

/// The stories the sprint names, for the first release, before the release
/// document exists. An absent or unreadable sprint names none, which the
/// caller reports as nothing verified rather than as a defect: the sprint is
/// not the document this arm is about.
fn sprint_story_ids(ctx: &mut Ctx) -> Vec<String> {
    let Ok((_, sprint)) = load_value(ctx, "stories/sprint.yaml") else {
        return Vec::new();
    };
    ypath::resolve_strings(&sprint, "stories[].id").unwrap_or_default()
}

/// Plan keys on an `EPIC-nnn` or a `SPRINT-nnn`; either resolves to the
/// `IDEA-nnn` at the top-level `id` of `requirements.yaml`.
fn resolve_plan_subject(ctx: &mut Ctx, id: &str) -> Result<String, CliError> {
    let mut epic = id.to_string();
    if ids::split_id(id)
        .map(|(p, _)| p == "SPRINT")
        .unwrap_or(false)
    {
        // A SPRINT argument resolves to its epic through `sprint.yaml` `epic`.
        // An absent file, an unparsable one, and one with no `epic` key are
        // three different defects and none of them leaves the SPRINT id
        // standing in for an epic.
        let rel = "stories/sprint.yaml";
        if !ctx.doc_path(rel).exists() {
            return Err(CliError::at(
                "DFA-E013",
                format!(
                    "'{id}' names a sprint and {rel} does not exist, so it resolves to no epic"
                ),
                rel,
            ));
        }
        let (_, sprint) = load_value(ctx, rel)?;
        let Some(e) = sprint
            .get(Value::String("epic".into()))
            .and_then(Value::as_str)
            .filter(|e| !e.is_empty())
        else {
            return Err(CliError::at(
                "DFA-E013",
                format!("'{id}' names a sprint and {rel} carries no epic key"),
                rel,
            ));
        };
        epic = e.to_string();
    }

    let (_, req) = load_value(ctx, "requirements.yaml")?;
    // The epic must be one the document holds: the top-level `id` is the answer
    // only for an epic `requirements.yaml` actually defines.
    let holds = ypath::resolve_strings(&req, "epics[].id")
        .unwrap_or_default()
        .contains(&epic);
    if !holds {
        return Err(CliError::at(
            "DFA-E013",
            format!("'{epic}' names no epic in requirements.yaml"),
            "requirements.yaml",
        ));
    }
    let idea = req
        .get(Value::String("id".into()))
        .and_then(Value::as_str)
        .unwrap_or("");
    if idea.is_empty() {
        return Err(CliError::at(
            "DFA-E013",
            "requirements.yaml carries no top-level id, so the plan gate has no IDEA subject"
                .to_string(),
            "requirements.yaml",
        ));
    }
    Ok(idea.to_string())
}

/// Build keys on a `STORY-nnn`; the predecessor subject is the `SPRINT-nnn` of
/// the sprint whose `stories[]` or `deferred[]` lists the story.
fn resolve_build_subject(ctx: &mut Ctx, id: &str) -> Result<String, CliError> {
    let (_, sprint) = load_value(ctx, "stories/sprint.yaml")?;
    let listed = ["stories[].id", "deferred[].id"].iter().any(|p| {
        ypath::resolve_strings(&sprint, p)
            .unwrap_or_default()
            .iter()
            .any(|s| s == id)
    });
    if !listed {
        // A story listed in no sprint is DFA-E013, exit 3.
        return Err(CliError::at(
            "DFA-E013",
            format!("'{id}' is listed in no sprint in stories/sprint.yaml"),
            "stories/sprint.yaml",
        ));
    }
    let sid = sprint
        .get(Value::String("id".into()))
        .and_then(Value::as_str)
        .unwrap_or("");
    if sid.is_empty() {
        // An empty predecessor subject makes the report path `reports/-plan.yaml`,
        // which names no document a reader can fix.
        return Err(CliError::at(
            "DFA-E013",
            "stories/sprint.yaml carries no top-level id, so the plan report has no subject"
                .to_string(),
            "stories/sprint.yaml",
        ));
    }
    Ok(sid.to_string())
}
