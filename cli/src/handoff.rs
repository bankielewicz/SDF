//! The conventions section 6 handoff block: the twelve-line fixed format the
//! Stop hook prints, rendered from `state.toml`, the gate report, and a few
//! counts the CLI measures on the file system.
//!
//! The rendering is pure: [`render`] takes the state, the report, the config,
//! and a [`RenderInputs`] carrying every value that needs a file system to
//! measure. [`measure`] fills that struct in from a [`crate::ctx::Ctx`].

use crate::config::Config;
use crate::ctx::Ctx;
use crate::doc;
use crate::report::{Finding, Report};
use crate::state::{self, phase_name, State};
use std::path::Path;

/// The label column width: content starts at character 11.
const LABEL_W: usize = 10;

/// The `<n> · <Name>` column width inside the `Phase` line's content.
const PHASE_W: usize = 20;

/// The longest content kept verbatim.
const CONTENT_MAX: usize = 89;

/// The scalar the truncation looks back from for a space.
const CUT_AT: usize = 86;

/// The line width cap of rendering rule 2.
const LINE_MAX: usize = 100;

/// The twelve-line cap of section 6.
const MAX_LINES: usize = 12;

/// The most `Found` lines section 6 allows.
const FOUND_MAX: usize = 3;

/// The separator between a label's content and its evidence.
const EV_SEP: &str = "  ";

/// The trust error code used when `[last_gate]` names none.
const TRUST_CODE_FALLBACK: &str = "DFA-E503";

/// The gate result the `Gate` line renders and the transition table keys on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateResult {
    /// Every blocking check passed.
    Pass,
    /// A blocking check failed inside the phase.
    Fail,
    /// The gate sent the work back to the named phase.
    SendBack(String),
    /// `trust verify` failed, so no gate ran.
    TrustFail,
    /// No report, or a report whose gate never ran.
    NotRun,
}

impl GateResult {
    /// The machine token, matching `[last_gate].result` of `state.toml`.
    pub fn token(&self) -> &'static str {
        match self {
            GateResult::Pass => "PASS",
            GateResult::Fail => "FAIL",
            GateResult::SendBack(_) => "SEND_BACK",
            GateResult::TrustFail => "TRUST_FAIL",
            GateResult::NotRun => "NOT_RUN",
        }
    }

    /// The text the `Gate` line opens with.
    pub fn label(&self) -> String {
        match self {
            GateResult::Pass => "PASS".to_string(),
            GateResult::Fail => "FAIL".to_string(),
            GateResult::SendBack(to) if to.is_empty() => "SEND BACK".to_string(),
            GateResult::SendBack(to) => format!("SEND BACK to {}", phase_name(to)),
            GateResult::TrustFail => "TRUST FAIL".to_string(),
            GateResult::NotRun => "NOT RUN".to_string(),
        }
    }
}

/// Everything the renderer cannot derive from the state, the report, and the
/// config: the measured counts, the slug, the blocked line, and the two values
/// the verify `Next` row needs. Every field is public so a test builds one
/// without touching a file system.
#[derive(Debug, Clone)]
pub struct RenderInputs {
    /// The `Phase` line slug: the first H1 of the phase's document.
    pub slug: String,
    /// The `Done` line content.
    pub done: String,
    /// The `Blocked` line content: `none` or `you: <question>`.
    pub blocked: String,
    /// The project-relative report path, or `None` for `Full report: none`.
    pub report: Option<String>,
    /// The trust error code when `trust verify` failed, which forces the
    /// `TRUST FAIL` result.
    pub trust_code: Option<String>,
    /// The `sprint.yaml` story the verify `PASS` row advances to.
    pub next_story: Option<String>,
    /// The version the verify `PASS` row releases when no story is ready.
    pub next_version: String,
}

impl Default for RenderInputs {
    fn default() -> Self {
        RenderInputs {
            slug: "-".to_string(),
            done: "-".to_string(),
            blocked: "none".to_string(),
            report: None,
            trust_code: None,
            next_story: None,
            next_version: "v0.1.0".to_string(),
        }
    }
}

/// How the Found budget resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundPlan {
    /// Findings rendered one per line.
    pub full: usize,
    /// The count the `+<n> more in report` line carries, when one is rendered.
    pub more: Option<usize>,
    /// True when the optional `Then` line was dropped to make room.
    pub drop_then: bool,
}

/// One entry of the report's `verifiers` block, resolved against the config.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierEntry {
    /// The subagent name.
    pub subagent: String,
    /// The count that passed.
    pub passed: i64,
    /// The count attempted.
    pub total: i64,
    /// The unit word the `Verified` line prints.
    pub unit: String,
}

impl VerifierEntry {
    /// `passed / total`, with an empty attempt counted as whole.
    pub fn ratio(&self) -> f64 {
        if self.total <= 0 {
            1.0
        } else {
            self.passed as f64 / self.total as f64
        }
    }
}

/// Pad `content` to the label column and cap the line, per rendering rules 1
/// and 2. No line of the block legitimately ends in a space, so the result is
/// trimmed at the end.
pub fn line(label: &str, content: &str) -> String {
    let padded = format!("{label:<w$}{}", cap(content), w = LABEL_W);
    padded.trim_end().to_string()
}

/// Cut `content` to the width rule: kept verbatim at 89 scalars or fewer, and
/// otherwise cut at the last space at or before scalar 86 with `...` appended.
pub fn cap(content: &str) -> String {
    cap_to(content, CONTENT_MAX, CUT_AT)
}

/// The cut of rendering rule 2 at an arbitrary pair of widths, so the
/// `Full report:` line, whose label is thirteen wide rather than ten, holds the
/// same hundred-scalar ceiling.
fn cap_to(text: &str, keep: usize, cut_at: usize) -> String {
    if text.chars().count() <= keep {
        return text.to_string();
    }
    let head: String = text.chars().take(cut_at).collect();
    let kept = match head.rfind(' ') {
        Some(i) => &head[..i],
        None => head.as_str(),
    };
    format!("{}...", kept.trim_end())
}

/// The result the block renders, with a trust failure overriding the report.
pub fn result_of(report: Option<&Report>, inputs: &RenderInputs) -> GateResult {
    if inputs.trust_code.is_some() {
        return GateResult::TrustFail;
    }
    let Some(r) = report else {
        return GateResult::NotRun;
    };
    match r.gate.result.as_str() {
        "PASS" => GateResult::Pass,
        "FAIL" => GateResult::Fail,
        "SEND_BACK" | "SEND BACK" => GateResult::SendBack(r.gate.send_back_to.clone()),
        "TRUST_FAIL" => GateResult::TrustFail,
        _ => GateResult::NotRun,
    }
}

/// The `Gate` line content: the result, then two spaces, then the evidence.
pub fn gate_content(result: &GateResult, report: Option<&Report>, inputs: &RenderInputs) -> String {
    let evidence = match result {
        GateResult::Pass => match report {
            Some(r) => format!("{} checks", r.gate.checks.len()),
            None => String::new(),
        },
        GateResult::Fail | GateResult::SendBack(_) => match report {
            Some(r) => r
                .gate
                .checks
                .iter()
                .filter(|c| c.status == "fail")
                .take(3)
                .map(|c| format!("{} {}", c.id, c.reason).trim_end().to_string())
                .collect::<Vec<_>>()
                .join(", "),
            None => String::new(),
        },
        GateResult::TrustFail => inputs.trust_code.clone().unwrap_or_default(),
        GateResult::NotRun => String::new(),
    };
    if evidence.is_empty() {
        result.label()
    } else {
        format!("{}{EV_SEP}{evidence}", result.label())
    }
}

/// The report's `verifiers` block, in the order the report writes it.
pub fn verifier_entries(report: Option<&Report>, cfg: &Config) -> Vec<VerifierEntry> {
    let Some(map) = report.and_then(|r| r.verifiers.as_ref()) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (key, value) in map {
        let key_name = key.as_str().unwrap_or_default();
        let subagent = value
            .get("subagent")
            .and_then(|v| v.as_str())
            .unwrap_or(key_name)
            .to_string();
        let passed = value.get("passed").and_then(|v| v.as_i64()).unwrap_or(0);
        let total = value.get("total").and_then(|v| v.as_i64()).unwrap_or(0);
        let unit = value
            .get("unit")
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .or_else(|| cfg.verifier_by_name(&subagent).map(|v| v.unit.clone()))
            .unwrap_or_else(|| "checks".to_string());
        out.push(VerifierEntry {
            subagent,
            passed,
            total,
            unit,
        });
    }
    out
}

/// The entry the `Verified` and verify `Done` lines show: the lowest
/// `passed/total`, a tie broken by `config.toml` `[[verifier]]` order and then
/// by the order the report writes the blocks.
pub fn lowest_verifier(entries: &[VerifierEntry], cfg: &Config) -> Option<usize> {
    let key = |i: usize| {
        let e = &entries[i];
        let order = cfg
            .verifier
            .iter()
            .position(|v| v.name == e.subagent)
            .unwrap_or(usize::MAX);
        (e.ratio(), order, i)
    };
    (0..entries.len()).min_by(|a, b| {
        let (ra, oa, ia) = key(*a);
        let (rb, ob, ib) = key(*b);
        ra.total_cmp(&rb).then(oa.cmp(&ob)).then(ia.cmp(&ib))
    })
}

/// The `Verified` line content, or `None` when the report has no `verifiers`
/// block.
pub fn verified_content(report: Option<&Report>, cfg: &Config) -> Option<String> {
    let entries = verifier_entries(report, cfg);
    let i = lowest_verifier(&entries, cfg)?;
    let e = &entries[i];
    let mut s = format!("{} · {}/{} {}", e.subagent, e.passed, e.total, e.unit);
    let rest = entries.len() - 1;
    if rest > 0 {
        s.push_str(&format!(" +{rest} more"));
    }
    Some(s)
}

/// The findings in the `Found` order: severity `block`, `warn`, `info`, then ID
/// ascending.
pub fn ordered_findings(report: Option<&Report>) -> Vec<&Finding> {
    let Some(r) = report else {
        return Vec::new();
    };
    let mut v: Vec<&Finding> = r.findings.iter().collect();
    v.sort_by(|a, b| {
        severity_rank(&a.severity)
            .cmp(&severity_rank(&b.severity))
            .then_with(|| id_key(&a.id).cmp(&id_key(&b.id)))
    });
    v
}

/// The severity order section 6 fixes.
fn severity_rank(severity: &str) -> u8 {
    match severity {
        "block" => 0,
        "warn" => 1,
        "info" => 2,
        _ => 3,
    }
}

/// An ID sort key: the prefix, then the number, so `FIND-009` precedes
/// `FIND-010` whatever the padding.
fn id_key(id: &str) -> (String, u32, String) {
    match doc::ids::split_id(id) {
        Some((p, n)) => (p.to_string(), n, id.to_string()),
        None => (id.to_string(), 0, id.to_string()),
    }
}

/// Resolve the Found budget: the fixed line count, the number of findings, and
/// whether an optional `Then` line is present to drop.
pub fn plan_found(fixed: usize, findings: usize, has_then: bool) -> FoundPlan {
    let budget = |fixed: usize| MAX_LINES.saturating_sub(fixed).min(FOUND_MAX);
    let mut drop_then = false;
    let mut b = budget(fixed);
    if b == 0 && has_then {
        drop_then = true;
        b = budget(fixed - 1);
    }
    if findings <= b {
        return FoundPlan {
            full: findings,
            more: None,
            drop_then,
        };
    }
    if b == 0 {
        // Unreachable through `render`: ten fixed lines leave a budget of two.
        return FoundPlan {
            full: 0,
            more: None,
            drop_then,
        };
    }
    FoundPlan {
        full: b - 1,
        more: Some(findings - (b - 1)),
        drop_then,
    }
}

/// `[active].<phase>`, falling back to the current subject.
fn active_or(state: &State, phase: &str, id: &str) -> String {
    match state.active.get(phase) {
        Some(a) if !a.is_empty() => a.to_string(),
        _ => id.to_string(),
    }
}

/// A `/command <id>` line, with the id omitted when there is none.
fn cmd(name: &str, id: &str) -> String {
    if id.is_empty() {
        format!("/{name}")
    } else {
        format!("/{name} {id}")
    }
}

/// The `--remedy <ID>,<ID>` argument for a send-back: the `block` findings
/// whose ID carries `prefix`, falling back to every finding carrying it.
///
/// The spec's worked example cites two of five findings on a send-back, which
/// this reads as the blocking ones.
fn remedy(report: Option<&Report>, prefix: &str) -> Option<String> {
    let all = ordered_findings(report);
    let matching: Vec<&Finding> = all
        .into_iter()
        .filter(|f| {
            doc::ids::split_id(&f.id)
                .map(|(p, _)| p == prefix)
                .unwrap_or(false)
        })
        .collect();
    let blocking: Vec<&&Finding> = matching.iter().filter(|f| f.severity == "block").collect();
    let ids: Vec<String> = if blocking.is_empty() {
        matching.iter().map(|f| f.id.clone()).collect()
    } else {
        blocking.iter().map(|f| f.id.clone()).collect()
    };
    if ids.is_empty() {
        return None;
    }
    Some(ids.join(","))
}

/// `<command> --remedy <ids>`, dropping the flag when no ID resolves.
fn remedy_cmd(name: &str, id: &str, report: Option<&Report>, prefix: &str) -> String {
    match remedy(report, prefix) {
        Some(ids) => format!("{} --remedy {ids}", cmd(name, id)),
        None => cmd(name, id),
    }
}

/// The `Next` line and the optional `Then` line, from the transition table.
pub fn next_then(
    state: &State,
    report: Option<&Report>,
    phase: &str,
    id: &str,
    inputs: &RenderInputs,
) -> (String, Option<String>) {
    let result = result_of(report, inputs);
    transition(state, report, phase, id, &result, inputs, 0)
}

/// The transition table. `depth` guards the `design` and `reflect` rows, which
/// name the row of the phase `[current].phase` holds.
#[allow(clippy::too_many_arguments)]
fn transition(
    state: &State,
    report: Option<&Report>,
    phase: &str,
    id: &str,
    result: &GateResult,
    inputs: &RenderInputs,
    depth: u8,
) -> (String, Option<String>) {
    let a = |p: &str| active_or(state, p, id);

    // `trust pin` is a shell line, and it overrides every phase row.
    if matches!(result, GateResult::TrustFail) {
        return ("devforgeai trust pin".to_string(), None);
    }

    // Design advances no phase, so it names the row the current phase holds.
    if (phase == "design" || (phase == "reflect" && matches!(result, GateResult::Pass)))
        && depth == 0
    {
        let current = state.current.phase.clone();
        let inner = if state.last_gate.phase == current {
            match state.last_gate.result.as_str() {
                "PASS" => GateResult::Pass,
                "FAIL" => GateResult::Fail,
                "SEND_BACK" => GateResult::SendBack(state.last_gate.send_back_to.clone()),
                _ => GateResult::NotRun,
            }
        } else {
            GateResult::NotRun
        };
        let inner_id = active_or(state, &current, &state.current.id);
        let (next, _) = transition(state, None, &current, &inner_id, &inner, inputs, 1);
        return (next, None);
    }

    if matches!(result, GateResult::NotRun) {
        return (cmd(phase, &a(phase)), None);
    }

    match (phase, result) {
        ("explore", GateResult::Pass) => (
            cmd("discover", &a("discover")),
            Some(cmd("constitute", &a("constitute"))),
        ),
        ("discover", GateResult::Pass) => (
            cmd("constitute", &a("constitute")),
            Some(cmd("plan", &a("plan"))),
        ),
        ("discover", GateResult::SendBack(_)) => (
            remedy_cmd("explore", &a("explore"), report, "FLOW"),
            Some(format!("{} --resume", cmd("discover", &a("discover")))),
        ),
        ("constitute", GateResult::Pass) => {
            (cmd("plan", &a("plan")), Some(cmd("build", &a("build"))))
        }
        ("constitute", GateResult::SendBack(_)) => (
            remedy_cmd("discover", &a("discover"), report, "REQ"),
            Some(format!("{} --resume", cmd("constitute", &a("constitute")))),
        ),
        ("plan", GateResult::Pass) => {
            (cmd("build", &a("build")), Some(cmd("verify", &a("verify"))))
        }
        ("plan", GateResult::SendBack(to)) => {
            let next = if to == "constitute" {
                remedy_cmd("constitute", &a("constitute"), report, "CON")
            } else {
                remedy_cmd("discover", &a("discover"), report, "REQ")
            };
            (next, Some(format!("{} --resume", cmd("plan", &a("plan")))))
        }
        ("build", GateResult::Pass) => {
            let release = a("release");
            let then = match state.active.release.is_empty() {
                true => None,
                false => Some(cmd("release", &release)),
            };
            (cmd("verify", &a("verify")), then)
        }
        ("build", GateResult::SendBack(_)) => (
            remedy_cmd("plan", &a("plan"), report, "AC"),
            Some(format!("{} --resume", cmd("build", &a("build")))),
        ),
        ("verify", GateResult::Pass) => match &inputs.next_story {
            Some(s) => (cmd("build", s), Some(cmd("verify", s))),
            None => (cmd("release", &inputs.next_version), None),
        },
        ("verify", GateResult::SendBack(to)) => {
            let next = if to == "plan" {
                remedy_cmd("plan", &a("plan"), report, "AC")
            } else {
                remedy_cmd("build", &a("build"), report, "FIND")
            };
            (
                next,
                Some(format!("{} --resume", cmd("verify", &a("verify")))),
            )
        }
        ("release", GateResult::Pass) => ("/reflect".to_string(), None),
        ("release", GateResult::SendBack(_)) => (
            remedy_cmd("verify", &a("verify"), report, "FIND"),
            Some(format!("{} --resume", cmd("release", &a("release")))),
        ),
        // Every FAIL row, and any pairing the table does not name, repeats the
        // phase's own command.
        _ => (cmd(phase, &a(phase)), None),
    }
}

/// Render the block. The result is at most twelve lines, the last of which is
/// the `Full report:` line section 6 exempts from the column rule.
pub fn render(
    state: &State,
    report: Option<&Report>,
    cfg: &Config,
    phase: &str,
    id: &str,
    ctx: &RenderInputs,
) -> Vec<String> {
    let result = result_of(report, ctx);
    let (next, then) = next_then(state, report, phase, id, ctx);
    let verified = verified_content(report, cfg);

    let findings = match result {
        GateResult::SendBack(_) => ordered_findings(report),
        _ => Vec::new(),
    };

    // Phase, Done, Gate, [Verified], blank, Next, [Then], Blocked, blank, Full.
    let fixed = 8 + usize::from(verified.is_some()) + usize::from(then.is_some());
    let plan = plan_found(fixed, findings.len(), then.is_some());
    let then = if plan.drop_then { None } else { then };

    let mut out = Vec::new();
    out.push(line("Phase", &phase_content(phase, id, &ctx.slug)));
    out.push(line("Done", &ctx.done));
    out.push(line("Gate", &gate_content(&result, report, ctx)));
    if let Some(v) = verified {
        out.push(line("Verified", &v));
    }
    for f in findings.iter().take(plan.full) {
        out.push(line("Found", &format!("{} {}", f.id, f.summary)));
    }
    if let Some(n) = plan.more {
        out.push(line("Found", &format!("+{n} more in report")));
    }
    out.push(String::new());
    out.push(line("Next", &next));
    if let Some(t) = then {
        out.push(line("Then", &t));
    }
    out.push(line("Blocked", &ctx.blocked));
    out.push(String::new());
    // Section 6 writes this line without the label column, so it is exempt from
    // the column rule and capped as a whole line instead.
    let full = format!("Full report: {}", ctx.report.as_deref().unwrap_or("none"));
    out.push(cap_to(&full, LINE_MAX, LINE_MAX - 3));
    out
}

/// The `Phase` line content: `<n> · <Name>` padded to twenty, then the subject
/// and the slug. A phase outside the section 5 sequence takes an em dash.
pub fn phase_content(phase: &str, id: &str, slug: &str) -> String {
    let number = match state::phase_index(phase) {
        Some(i) => i.to_string(),
        None => "\u{2014}".to_string(),
    };
    let head = format!("{number} \u{b7} {}", phase_name(phase));
    let subject = if id.is_empty() { "-" } else { id };
    let slug = if slug.is_empty() { "-" } else { slug };
    format!("{head:<w$}{subject} \u{b7} {slug}", w = PHASE_W)
}

// ---------------------------------------------------------------------------
// The measured half: everything that needs the file system.
// ---------------------------------------------------------------------------

/// Fill in every value [`render`] cannot derive, reading the project tree.
pub fn measure(ctx: &mut Ctx, phase: &str, id: &str, report: Option<&Report>) -> RenderInputs {
    let root = ctx.root.clone();
    let cfg = ctx.config().cloned().unwrap_or_default();
    let trust_code = ctx.state().ok().and_then(|s| trust_code(s, phase));
    RenderInputs {
        slug: slug_of(&root, phase, id),
        done: done_content(&root, phase, id, report, &cfg),
        blocked: blocked_of(&root, phase, id),
        report: report.map(|_| {
            crate::project::rel_display(&root, &crate::report::report_path(&root, id, phase))
        }),
        trust_code,
        next_story: next_ready_story(&root),
        next_version: next_release_version(&root),
    }
}

/// The trust error code when `[last_gate]` records a trust failure for this
/// phase, which is the only place the handoff learns of one.
pub fn trust_code(state: &State, phase: &str) -> Option<String> {
    if state.last_gate.result != "TRUST_FAIL" || state.last_gate.phase != phase {
        return None;
    }
    Some(
        state
            .last_gate
            .failed_checks
            .first()
            .cloned()
            .unwrap_or_else(|| TRUST_CODE_FALLBACK.to_string()),
    )
}

/// The document whose first H1 gives the slug and whose `open_questions` give
/// the `Blocked` line.
///
/// Constitute writes the six context files rather than one document, so its
/// row is the first context file that exists.
pub fn phase_doc(root: &Path, phase: &str, id: &str) -> Option<std::path::PathBuf> {
    let dot = crate::project::dot(root);
    let p = match phase {
        "explore" => dot.join("explore").join("brief.md"),
        "discover" => dot.join("requirements.yaml"),
        "constitute" => {
            return doc::CONTEXT_STEMS
                .iter()
                .map(|s| dot.join("context").join(format!("{s}.md")))
                .find(|p| p.is_file())
        }
        "plan" => dot.join("stories").join("sprint.yaml"),
        "build" | "verify" => dot.join("stories").join(format!("{id}.md")),
        "release" => dot.join("releases").join(format!("{id}.yaml")),
        "design" => dot.join("ui-specs").join(format!("{id}.md")),
        "reflect" => dot.join("reports").join(format!("reflect-{id}.yaml")),
        _ => return None,
    };
    p.is_file().then_some(p)
}

/// The `Phase` line slug: the first H1 of the phase's document, lowercased,
/// non-alphanumerics collapsed to single hyphens, cut to twenty-four scalars.
/// A document with no H1, or no document at all, gives `-`.
pub fn slug_of(root: &Path, phase: &str, id: &str) -> String {
    let Some(path) = phase_doc(root, phase, id) else {
        return "-".to_string();
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return "-".to_string();
    };
    match first_h1(&text) {
        Some(h) => slugify(&h),
        None => "-".to_string(),
    }
}

/// The first `# ` heading of a Markdown body.
fn first_h1(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim_start)
        .find(|l| l.starts_with("# "))
        .map(|l| l.trim_start_matches('#').trim().to_string())
}

/// Lowercase, non-alphanumerics collapsed to single hyphens, cut to
/// twenty-four scalars.
pub fn slugify(title: &str) -> String {
    let mut out = String::new();
    for c in title.chars() {
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let out: String = out.chars().take(24).collect();
    let trimmed = out.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "-".to_string()
    } else {
        trimmed
    }
}

/// The `Blocked` line: `none`, or `you: <first open question>`.
pub fn blocked_of(root: &Path, phase: &str, id: &str) -> String {
    let Some(path) = phase_doc(root, phase, id) else {
        return "none".to_string();
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return "none".to_string();
    };
    let rel = crate::project::rel_display(root, &path);
    let Ok(fm) = doc::frontmatter::parse(&text, doc::source_for(&path), &rel) else {
        return "none".to_string();
    };
    match fm
        .seq_key("open_questions")
        .unwrap_or_default()
        .into_iter()
        .find(|q| !q.trim().is_empty())
    {
        Some(q) => format!("you: {}", q.trim()),
        None => "none".to_string(),
    }
}

/// The `Done` line, one row per phase.
pub fn done_content(
    root: &Path,
    phase: &str,
    id: &str,
    report: Option<&Report>,
    cfg: &Config,
) -> String {
    match phase {
        "explore" => {
            let index = doc::ids::build(root);
            let ideas = defined_under(&index, "IDEA", ".devforgeai/explore/");
            let flows = defined_under(&index, "FLOW", ".devforgeai/explore/");
            format!("{ideas} ideas \u{b7} {flows} flows")
        }
        "discover" => {
            let index = doc::ids::build(root);
            let req = defined_under(&index, "REQ", ".devforgeai/requirements.yaml");
            let epic = defined_under(&index, "EPIC", ".devforgeai/requirements.yaml");
            let personas = defined_under(&index, "PERSONA", ".devforgeai/requirements.yaml");
            format!("{req} REQ \u{b7} {epic} EPIC \u{b7} {personas} personas")
        }
        "constitute" => {
            let dot = crate::project::dot(root);
            let context = doc::CONTEXT_STEMS
                .iter()
                .filter(|s| non_empty(&dot.join("context").join(format!("{s}.md"))))
                .count();
            let adr = count_files(&dot.join("adr"), "ADR-", ".md");
            format!("{context}/6 context \u{b7} {adr} ADR")
        }
        "plan" => {
            let dot = crate::project::dot(root);
            let stories = count_files(&dot.join("stories"), "STORY-", ".md");
            let index = doc::ids::build(root);
            let acs = defined_under(&index, "AC", ".devforgeai/stories/");
            format!("{stories} stories \u{b7} {acs} ACs")
        }
        "build" => {
            let tests = report
                .and_then(tests_passed)
                .map(|n| n.to_string())
                .unwrap_or_else(|| "-".to_string());
            let coverage = report
                .and_then(|r| r.coverage.as_ref())
                .and_then(|c| c.get("overall"))
                .and_then(|v| v.as_f64());
            match coverage {
                Some(c) => format!("{tests} tests \u{b7} {c:.1}% coverage"),
                None => format!("{tests} tests \u{b7} - coverage"),
            }
        }
        "verify" => {
            let entries = verifier_entries(report, cfg);
            let (p, t) = match lowest_verifier(&entries, cfg) {
                Some(i) => (entries[i].passed, entries[i].total),
                None => (0, 0),
            };
            let findings = report.map(|r| r.findings.len()).unwrap_or(0);
            format!("{p}/{t} ACs \u{b7} {findings} findings")
        }
        "release" => {
            let path = crate::project::dot(root)
                .join("releases")
                .join(format!("{id}.yaml"));
            let stories = std::fs::read_to_string(&path)
                .ok()
                .and_then(|t| serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&t).ok())
                .and_then(|v| v.get("stories").and_then(|s| s.as_sequence()).map(Vec::len))
                .unwrap_or(0);
            let version = if id.is_empty() { "-" } else { id };
            format!("{stories} stories \u{b7} {version}")
        }
        // Design and reflect own no count of their own; the block still needs a
        // `Done` line, so it carries the subject.
        _ => {
            if id.is_empty() {
                "-".to_string()
            } else {
                id.to_string()
            }
        }
    }
}

/// Every ID with `prefix` defined at a site under `prefix_path`.
fn defined_under(index: &doc::ids::IdIndex, prefix: &str, prefix_path: &str) -> usize {
    index
        .definitions
        .iter()
        .filter(|(k, sites)| {
            doc::ids::split_id(k)
                .map(|(p, _)| p == prefix)
                .unwrap_or(false)
                && sites.iter().any(|s| s.path.starts_with(prefix_path))
        })
        .count()
}

/// True when a file exists and holds more than whitespace.
fn non_empty(path: &Path) -> bool {
    std::fs::read_to_string(path)
        .map(|t| !t.trim().is_empty())
        .unwrap_or(false)
}

/// Files in `dir` whose name opens with `prefix` and ends with `suffix`.
fn count_files(dir: &Path, prefix: &str, suffix: &str) -> usize {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .filter_map(Result::ok)
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.starts_with(prefix) && name.ends_with(suffix)
        })
        .count()
}

/// The passing test count the build `Done` row reads: the `passed` evidence of
/// a `tests_pass` check, falling back to any check carrying one.
fn tests_passed(report: &Report) -> Option<i64> {
    let passed = |c: &crate::report::CheckEntry| c.evidence.get("passed").and_then(|v| v.as_i64());
    report
        .gate
        .checks
        .iter()
        .find(|c| c.kind == "tests_pass")
        .and_then(passed)
        .or_else(|| report.gate.checks.iter().find_map(passed))
}

/// The `sprint.yaml` story with the lowest `order` whose `status` is `ready`.
/// An entry with no `order` takes its position in the file.
pub fn next_ready_story(root: &Path) -> Option<String> {
    let path = crate::project::dot(root)
        .join("stories")
        .join("sprint.yaml");
    let text = std::fs::read_to_string(path).ok()?;
    let value: serde_yaml_ng::Value = serde_yaml_ng::from_str(&text).ok()?;
    let stories = value.get("stories")?.as_sequence()?;
    stories
        .iter()
        .enumerate()
        .filter(|(_, s)| s.get("status").and_then(|v| v.as_str()) == Some("ready"))
        .min_by_key(|(i, s)| s.get("order").and_then(|v| v.as_i64()).unwrap_or(*i as i64))
        .and_then(|(_, s)| s.get("id").and_then(|v| v.as_str()).map(str::to_string))
}

/// The highest version under `releases/` with the minor incremented and the
/// patch zeroed; `v0.1.0` when the directory is empty or absent.
pub fn next_release_version(root: &Path) -> String {
    let dir = crate::project::dot(root).join("releases");
    let highest = std::fs::read_dir(&dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            let stem = name.strip_suffix(".yaml")?.to_string();
            doc::version_triple(&stem)
        })
        .max();
    match highest {
        Some((major, minor, _)) => format!("v{major}.{}.0", minor + 1),
        None => "v0.1.0".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::{CheckEntry, GateBlock};

    /// The state the spec's worked examples run against.
    fn spec_state() -> State {
        let mut s = State::default();
        s.current.phase = "build".into();
        s.current.id = "STORY-014".into();
        s.active.explore = "IDEA-003".into();
        s.active.discover = "IDEA-003".into();
        s.active.constitute = "IDEA-003".into();
        s.active.plan = "SPRINT-001".into();
        s.active.build = "STORY-014".into();
        s.active.verify = "STORY-014".into();
        s.active.release = "v0.3.0".into();
        s
    }

    fn check(id: &str, status: &str, reason: &str) -> CheckEntry {
        CheckEntry {
            id: id.into(),
            kind: "tests_pass".into(),
            status: status.into(),
            severity: "block".into(),
            reason: reason.into(),
            evidence: serde_yaml_ng::Value::Null,
        }
    }

    fn finding(id: &str, severity: &str, summary: &str) -> Finding {
        Finding {
            id: id.into(),
            severity: severity.into(),
            summary: summary.into(),
            evidence: String::new(),
            category: None,
        }
    }

    fn verifiers(subagent: &str, passed: i64, total: i64, unit: &str) -> serde_yaml_ng::Mapping {
        let mut map = serde_yaml_ng::Mapping::new();
        let mut block = serde_yaml_ng::Mapping::new();
        block.insert("subagent".into(), subagent.into());
        block.insert("passed".into(), passed.into());
        block.insert("total".into(), total.into());
        block.insert("unit".into(), unit.into());
        map.insert(
            serde_yaml_ng::Value::String(subagent.replace('-', "_")),
            serde_yaml_ng::Value::Mapping(block),
        );
        map
    }

    /// The PASS report of the spec's first worked example: six checks and the
    /// `7/7 ACs` verifier block.
    fn pass_report() -> Report {
        let mut r = Report::skeleton("STORY-014", "build");
        r.status = "pass".into();
        r.gate = GateBlock {
            result: "PASS".into(),
            send_back_to: String::new(),
            checks: (1..=6)
                .map(|i| check(&format!("c{i}"), "pass", ""))
                .collect(),
        };
        r.verifiers = Some(verifiers("ac-compliance-verifier", 7, 7, "ACs"));
        r
    }

    /// The SEND BACK report of the spec's second worked example: one failing
    /// check, the `5/7 ACs` verifier block, and five findings of which two
    /// block.
    fn send_back_report() -> Report {
        let mut r = Report::skeleton("STORY-014", "verify");
        r.status = "send_back".into();
        r.gate = GateBlock {
            result: "SEND_BACK".into(),
            send_back_to: "build".into(),
            checks: vec![check("verify-acs", "fail", "5/7")],
        };
        r.verifiers = Some(verifiers("ac-compliance-verifier", 5, 7, "ACs"));
        r.findings = vec![
            finding(
                "FIND-001",
                "block",
                "AC-003 has no test covering the empty cart path",
            ),
            finding("FIND-002", "block", "AC-004 asserts no error path"),
            finding(
                "FIND-003",
                "warn",
                "the checkout handler duplicates a guard",
            ),
            finding("FIND-004", "warn", "a magic number sits in the total"),
            finding("FIND-005", "info", "the module doc is stale"),
        ];
        r
    }

    fn pass_inputs() -> RenderInputs {
        RenderInputs {
            slug: "order-checkout".into(),
            done: "214 tests \u{b7} 87.4% coverage".into(),
            blocked: "none".into(),
            report: Some(".devforgeai/reports/STORY-014-build.yaml".into()),
            ..Default::default()
        }
    }

    fn send_back_inputs() -> RenderInputs {
        RenderInputs {
            slug: "order-checkout".into(),
            done: "5/7 ACs \u{b7} 5 findings".into(),
            blocked: "none".into(),
            report: Some(".devforgeai/reports/STORY-014-verify.yaml".into()),
            ..Default::default()
        }
    }

    // -- The two worked examples, byte for byte -----------------------------

    /// `01-cli.md` lines 2210 to 2221.
    const SPEC_PASS: &str = "Phase     4 · Build           STORY-014 · order-checkout\n\
Done      214 tests · 87.4% coverage\n\
Gate      PASS  6 checks\n\
Verified  ac-compliance-verifier · 7/7 ACs\n\
\n\
Next      /verify STORY-014\n\
Then      /release v0.3.0\n\
Blocked   none\n\
\n\
Full report: .devforgeai/reports/STORY-014-build.yaml";

    /// `01-cli.md` lines 2225 to 2238.
    const SPEC_SEND_BACK: &str = "Phase     5 · Verify          STORY-014 · order-checkout\n\
Done      5/7 ACs · 5 findings\n\
Gate      SEND BACK to Build  verify-acs 5/7\n\
Verified  ac-compliance-verifier · 5/7 ACs\n\
Found     FIND-001 AC-003 has no test covering the empty cart path\n\
Found     +4 more in report\n\
\n\
Next      /build STORY-014 --remedy FIND-001,FIND-002\n\
Then      /verify STORY-014 --resume\n\
Blocked   none\n\
\n\
Full report: .devforgeai/reports/STORY-014-verify.yaml";

    #[test]
    fn spec_pass_example_renders_byte_for_byte() {
        let lines = render(
            &spec_state(),
            Some(&pass_report()),
            &Config::default(),
            "build",
            "STORY-014",
            &pass_inputs(),
        );
        assert_eq!(lines.join("\n"), SPEC_PASS);
    }

    #[test]
    fn spec_send_back_example_renders_byte_for_byte() {
        let lines = render(
            &spec_state(),
            Some(&send_back_report()),
            &Config::default(),
            "verify",
            "STORY-014",
            &send_back_inputs(),
        );
        assert_eq!(lines.join("\n"), SPEC_SEND_BACK);
    }

    // -- The column rules ---------------------------------------------------

    #[test]
    fn design_phase_renders_em_dash() {
        let inputs = RenderInputs {
            slug: "checkout-drawer".into(),
            ..Default::default()
        };
        let lines = render(
            &spec_state(),
            None,
            &Config::default(),
            "design",
            "UI-004",
            &inputs,
        );
        assert!(
            lines[0].starts_with("Phase     \u{2014} \u{b7} Design"),
            "{}",
            lines[0]
        );
        assert!(lines[0].ends_with("UI-004 \u{b7} checkout-drawer"));
    }

    #[test]
    fn send_back_prints_remedy_and_resume_lines() {
        let lines = render(
            &spec_state(),
            Some(&send_back_report()),
            &Config::default(),
            "verify",
            "STORY-014",
            &send_back_inputs(),
        );
        assert!(
            lines.contains(&"Next      /build STORY-014 --remedy FIND-001,FIND-002".to_string())
        );
        assert!(lines.contains(&"Then      /verify STORY-014 --resume".to_string()));
    }

    #[test]
    fn label_column_is_ten_wide() {
        let lines = render(
            &spec_state(),
            Some(&pass_report()),
            &Config::default(),
            "build",
            "STORY-014",
            &pass_inputs(),
        );
        for l in &lines {
            if l.is_empty() || l.starts_with("Full report: ") {
                continue;
            }
            let chars: Vec<char> = l.chars().collect();
            assert_eq!(
                chars[LABEL_W - 1],
                ' ',
                "the label column ends in a space: {l}"
            );
            assert_ne!(chars[LABEL_W], ' ', "content starts at character 11: {l}");
        }
    }

    #[test]
    fn phase_column_padded_to_twenty() {
        let content = phase_content("build", "STORY-014", "order-checkout");
        let chars: Vec<char> = content.chars().collect();
        assert_eq!(chars[PHASE_W - 1], ' ');
        assert_eq!(
            chars[PHASE_W..].iter().collect::<String>(),
            "STORY-014 \u{b7} order-checkout"
        );
    }

    #[test]
    fn line_capped_at_hundred_with_ellipsis() {
        let long = "word ".repeat(40);
        let l = line("Done", &long);
        assert!(l.chars().count() <= 100, "{} scalars", l.chars().count());
        assert!(l.ends_with("..."));
        assert!(!l.ends_with(" ..."), "the cut trims the space it cut at");
        // A content of exactly the limit is kept verbatim.
        let exact = "x".repeat(CONTENT_MAX);
        assert_eq!(line("Done", &exact).chars().count(), LABEL_W + CONTENT_MAX);
    }

    #[test]
    fn full_report_line_exempt_from_column_rule() {
        let lines = render(
            &spec_state(),
            Some(&pass_report()),
            &Config::default(),
            "build",
            "STORY-014",
            &pass_inputs(),
        );
        let last = lines.last().expect("a last line");
        assert!(last.starts_with("Full report: "));
        assert_eq!(
            &last["Full report: ".len()..],
            ".devforgeai/reports/STORY-014-build.yaml"
        );
    }

    // -- The twelve-line cap ------------------------------------------------

    #[test]
    fn pass_block_is_ten_lines() {
        let lines = render(
            &spec_state(),
            Some(&pass_report()),
            &Config::default(),
            "build",
            "STORY-014",
            &pass_inputs(),
        );
        assert_eq!(lines.len(), 10);
    }

    #[test]
    fn send_back_three_findings_fits_twelve() {
        // With no verifier block the fixed lines number nine, so the budget is
        // the section 6 maximum of three and every finding renders.
        let mut r = send_back_report();
        r.verifiers = None;
        r.findings.truncate(3);
        let lines = render(
            &spec_state(),
            Some(&r),
            &Config::default(),
            "verify",
            "STORY-014",
            &send_back_inputs(),
        );
        assert_eq!(lines.len(), 12, "{lines:#?}");
        assert_eq!(lines.iter().filter(|l| l.starts_with("Found")).count(), 3);
        assert!(!lines.iter().any(|l| l.contains("more in report")));

        // With the `Verified` line present the budget is two, so a third
        // finding folds into the `+n more` line rather than growing the block.
        let mut v = send_back_report();
        v.findings.truncate(3);
        let lines = render(
            &spec_state(),
            Some(&v),
            &Config::default(),
            "verify",
            "STORY-014",
            &send_back_inputs(),
        );
        assert_eq!(lines.len(), 12);
        assert!(lines.contains(&"Found     +2 more in report".to_string()));
    }

    #[test]
    fn send_back_five_findings_truncates_to_plus_three() {
        // A budget of three renders two findings and folds the other three.
        let mut r = send_back_report();
        r.verifiers = None;
        assert_eq!(r.findings.len(), 5);
        let lines = render(
            &spec_state(),
            Some(&r),
            &Config::default(),
            "verify",
            "STORY-014",
            &send_back_inputs(),
        );
        assert_eq!(lines.len(), 12, "{lines:#?}");
        assert_eq!(lines.iter().filter(|l| l.starts_with("Found")).count(), 3);
        assert!(lines.contains(&"Found     +3 more in report".to_string()));

        // The spec's worked example keeps `Verified`, which leaves a budget of
        // two: one finding renders and four fold.
        let lines = render(
            &spec_state(),
            Some(&send_back_report()),
            &Config::default(),
            "verify",
            "STORY-014",
            &send_back_inputs(),
        );
        assert!(lines.contains(&"Found     +4 more in report".to_string()));
    }

    #[test]
    fn budget_zero_drops_then_line() {
        // Unreachable through `render`, where ten fixed lines are the maximum,
        // so the rule is pinned at the function the spec writes it for.
        let p = plan_found(12, 4, true);
        assert!(p.drop_then, "the Then line goes first");
        assert_eq!(p.full, 0);
        assert_eq!(p.more, Some(4));

        let q = plan_found(10, 5, true);
        assert!(!q.drop_then);
        assert_eq!(q.full, 1);
        assert_eq!(q.more, Some(4));
    }

    // -- Verified -----------------------------------------------------------

    #[test]
    fn verified_omitted_without_verifiers() {
        let mut r = pass_report();
        r.verifiers = None;
        let lines = render(
            &spec_state(),
            Some(&r),
            &Config::default(),
            "build",
            "STORY-014",
            &pass_inputs(),
        );
        assert!(!lines.iter().any(|l| l.starts_with("Verified")));
        assert_eq!(lines.len(), 9);
    }

    #[test]
    fn verified_picks_lowest_ratio() {
        let mut map = verifiers("ac-compliance-verifier", 7, 7, "ACs");
        let mut low = serde_yaml_ng::Mapping::new();
        low.insert("subagent".into(), "standards-reviewer".into());
        low.insert("passed".into(), 3.into());
        low.insert("total".into(), 6.into());
        low.insert("unit".into(), "files".into());
        map.insert("standards".into(), serde_yaml_ng::Value::Mapping(low));
        let mut r = pass_report();
        r.verifiers = Some(map);

        let content = verified_content(Some(&r), &Config::default()).expect("a Verified line");
        assert_eq!(content, "standards-reviewer \u{b7} 3/6 files +1 more");

        // A tie at the lowest ratio goes to the first `[[verifier]]` entry.
        let mut tie = verifiers("standards-reviewer", 3, 6, "files");
        let mut other = serde_yaml_ng::Mapping::new();
        other.insert("subagent".into(), "ac-compliance-verifier".into());
        other.insert("passed".into(), 3.into());
        other.insert("total".into(), 6.into());
        other.insert("unit".into(), "ACs".into());
        tie.insert("ac_compliance".into(), serde_yaml_ng::Value::Mapping(other));
        r.verifiers = Some(tie);
        let content = verified_content(Some(&r), &Config::default()).expect("a Verified line");
        assert_eq!(
            content, "ac-compliance-verifier \u{b7} 3/6 ACs +1 more",
            "config.toml order breaks the tie"
        );
    }

    // -- Found --------------------------------------------------------------

    #[test]
    fn found_ordered_by_severity_then_id() {
        let mut r = send_back_report();
        r.findings = vec![
            finding("FIND-010", "info", "i"),
            finding("FIND-002", "warn", "w"),
            finding("FIND-009", "block", "b9"),
            finding("FIND-003", "block", "b3"),
        ];
        let ids: Vec<&str> = ordered_findings(Some(&r))
            .iter()
            .map(|f| f.id.as_str())
            .collect();
        assert_eq!(ids, ["FIND-003", "FIND-009", "FIND-002", "FIND-010"]);
    }

    // -- Gate ---------------------------------------------------------------

    #[test]
    fn trust_fail_renders_gate_line() {
        let inputs = RenderInputs {
            trust_code: Some("DFA-E503".into()),
            ..pass_inputs()
        };
        let lines = render(
            &spec_state(),
            Some(&pass_report()),
            &Config::default(),
            "build",
            "STORY-014",
            &inputs,
        );
        assert_eq!(lines[2], "Gate      TRUST FAIL  DFA-E503");
        assert!(lines.contains(&"Next      devforgeai trust pin".to_string()));
        assert!(!lines.iter().any(|l| l.starts_with("Then")));
    }

    // -- The two tables -----------------------------------------------------

    #[test]
    fn next_then_table_per_result() {
        let s = spec_state();
        let cases: &[(&str, &str, &str, Option<&str>)] = &[
            (
                "explore",
                "PASS",
                "/discover IDEA-003",
                Some("/constitute IDEA-003"),
            ),
            ("explore", "FAIL", "/explore IDEA-003", None),
            (
                "discover",
                "PASS",
                "/constitute IDEA-003",
                Some("/plan SPRINT-001"),
            ),
            ("discover", "FAIL", "/discover IDEA-003", None),
            (
                "constitute",
                "PASS",
                "/plan SPRINT-001",
                Some("/build STORY-014"),
            ),
            (
                "plan",
                "PASS",
                "/build STORY-014",
                Some("/verify STORY-014"),
            ),
            ("plan", "FAIL", "/plan SPRINT-001", None),
            (
                "build",
                "PASS",
                "/verify STORY-014",
                Some("/release v0.3.0"),
            ),
            ("build", "FAIL", "/build STORY-014", None),
            ("verify", "FAIL", "/verify STORY-014", None),
            ("release", "PASS", "/reflect", None),
            ("release", "FAIL", "/release v0.3.0", None),
        ];
        for (phase, result, next, then) in cases {
            let mut r = pass_report();
            r.gate.result = (*result).to_string();
            let (n, t) = next_then(&s, Some(&r), phase, "STORY-014", &RenderInputs::default());
            assert_eq!(&n, next, "{phase} {result} Next");
            assert_eq!(t.as_deref(), *then, "{phase} {result} Then");
        }

        // build PASS omits `Then` while `[active].release` is empty.
        let mut empty = s.clone();
        empty.active.release = String::new();
        let (_, t) = next_then(
            &empty,
            Some(&pass_report()),
            "build",
            "STORY-014",
            &RenderInputs::default(),
        );
        assert_eq!(t, None);

        // verify PASS advances to the next ready story, else to the release.
        let inputs = RenderInputs {
            next_story: Some("STORY-015".into()),
            ..Default::default()
        };
        let (n, t) = next_then(&s, Some(&pass_report()), "verify", "STORY-014", &inputs);
        assert_eq!(n, "/build STORY-015");
        assert_eq!(t.as_deref(), Some("/verify STORY-015"));
        let inputs = RenderInputs {
            next_version: "v0.4.0".into(),
            ..Default::default()
        };
        let (n, t) = next_then(&s, Some(&pass_report()), "verify", "STORY-014", &inputs);
        assert_eq!(n, "/release v0.4.0");
        assert_eq!(t, None);

        // A send-back cites the blocking ids upstream and resumes downstream.
        let mut plan_back = send_back_report();
        plan_back.gate.send_back_to = "constitute".into();
        plan_back.findings = vec![
            finding("CON-004", "block", "the constraint is contradicted"),
            finding("CON-009", "warn", "the wording drifts"),
        ];
        let (n, t) = next_then(
            &s,
            Some(&plan_back),
            "plan",
            "SPRINT-001",
            &RenderInputs::default(),
        );
        assert_eq!(n, "/constitute IDEA-003 --remedy CON-004");
        assert_eq!(t.as_deref(), Some("/plan SPRINT-001 --resume"));

        // A verify send-back routes by `send_back_to`: to build on a finding,
        // to plan on an untestable AC.
        let (n, t) = next_then(
            &s,
            Some(&send_back_report()),
            "verify",
            "STORY-014",
            &RenderInputs::default(),
        );
        assert_eq!(n, "/build STORY-014 --remedy FIND-001,FIND-002");
        assert_eq!(t.as_deref(), Some("/verify STORY-014 --resume"));
        let mut to_plan = send_back_report();
        to_plan.gate.send_back_to = "plan".into();
        to_plan.findings = vec![
            finding("AC-003", "block", "the criterion is untestable"),
            finding("AC-007", "info", "the wording is loose"),
        ];
        let (n, t) = next_then(
            &s,
            Some(&to_plan),
            "verify",
            "STORY-014",
            &RenderInputs::default(),
        );
        assert_eq!(n, "/plan SPRINT-001 --remedy AC-003");
        assert_eq!(t.as_deref(), Some("/verify STORY-014 --resume"));

        // NOT RUN repeats the phase's own command with the active id.
        let (n, t) = next_then(&s, None, "build", "STORY-014", &RenderInputs::default());
        assert_eq!(n, "/build STORY-014");
        assert_eq!(t, None);

        // Design names the row the current phase holds.
        let mut designing = s.clone();
        designing.last_gate.phase = "build".into();
        designing.last_gate.result = "PASS".into();
        let (n, t) = next_then(
            &designing,
            None,
            "design",
            "UI-004",
            &RenderInputs::default(),
        );
        assert_eq!(n, "/verify STORY-014");
        assert_eq!(t, None);

        // Reflect FAIL repeats its own command with the date.
        let mut r = pass_report();
        r.gate.result = "FAIL".into();
        let (n, _) = next_then(
            &s,
            Some(&r),
            "reflect",
            "2026-09-10",
            &RenderInputs::default(),
        );
        assert_eq!(n, "/reflect 2026-09-10");
    }

    // -- The measured half --------------------------------------------------

    #[test]
    fn done_counts_per_phase() {
        let t = tempfile::tempdir().expect("temp");
        let root = t.path();
        let dot = root.join(".devforgeai");
        let write = |rel: &str, body: &str| {
            let p = dot.join(rel);
            std::fs::create_dir_all(p.parent().expect("parent")).expect("mkdir");
            std::fs::write(&p, body).expect("write");
        };

        write(
            "explore/brief.md",
            "---\nschema: devforgeai/explore-brief/1\nid: IDEA-003\nphase: explore\nstatus: decided\nproduced_by: exploring-ideas\nconsumes: []\nopen_questions: []\n---\n\n# Order checkout\n\n- FLOW-001: browse\n- FLOW-002: pay\n",
        );
        write(
            "requirements.yaml",
            "schema: devforgeai/requirements/1\nid: IDEA-003\nphase: discover\nstatus: accepted\nproduced_by: discovering-requirements\nconsumes: []\nopen_questions: []\nrequirements:\n  - id: REQ-001\n  - id: REQ-002\nepics:\n  - id: EPIC-001\npersonas:\n  - id: PERSONA-001\n",
        );
        for stem in ["tech-stack", "source-tree", "dependencies"] {
            write(&format!("context/{stem}.md"), &format!("# {stem}\n"));
        }
        write("adr/ADR-001.md", "# one\n");
        write("adr/ADR-002.md", "# two\n");
        write(
            "stories/STORY-014.md",
            "---\nschema: devforgeai/story/1\nid: STORY-014\nphase: plan\nstatus: ready\nproduced_by: planning-work\nconsumes: []\nopen_questions: []\n---\n\n# Order checkout\n\n- AC-001: one\n- AC-002: two\n",
        );
        write("releases/v0.3.0.yaml", "schema: devforgeai/release/1\nid: v0.3.0\nstories:\n  - id: STORY-014\n  - id: STORY-015\n");

        let cfg = Config::default();
        assert_eq!(
            done_content(root, "explore", "IDEA-003", None, &cfg),
            "1 ideas \u{b7} 2 flows"
        );
        assert_eq!(
            done_content(root, "discover", "IDEA-003", None, &cfg),
            "2 REQ \u{b7} 1 EPIC \u{b7} 1 personas"
        );
        assert_eq!(
            done_content(root, "constitute", "IDEA-003", None, &cfg),
            "3/6 context \u{b7} 2 ADR"
        );
        assert_eq!(
            done_content(root, "plan", "SPRINT-001", None, &cfg),
            "1 stories \u{b7} 2 ACs"
        );
        assert_eq!(
            done_content(root, "release", "v0.3.0", None, &cfg),
            "2 stories \u{b7} v0.3.0"
        );

        // Build and verify read the report alone.
        let mut b = pass_report();
        b.gate.checks[0].evidence = serde_yaml_ng::from_str("passed: 214").expect("evidence");
        b.coverage = Some(serde_yaml_ng::from_str("overall: 87.4").expect("coverage"));
        assert_eq!(
            done_content(root, "build", "STORY-014", Some(&b), &cfg),
            "214 tests \u{b7} 87.4% coverage"
        );
        let bare = pass_report();
        assert_eq!(
            done_content(root, "build", "STORY-014", Some(&bare), &cfg),
            "- tests \u{b7} - coverage"
        );
        assert_eq!(
            done_content(root, "verify", "STORY-014", Some(&send_back_report()), &cfg),
            "5/7 ACs \u{b7} 5 findings"
        );

        // The slug and the blocked line come from the same documents.
        assert_eq!(slug_of(root, "build", "STORY-014"), "order-checkout");
        assert_eq!(blocked_of(root, "build", "STORY-014"), "none");
        assert_eq!(next_release_version(root), "v0.4.0");
    }
}
