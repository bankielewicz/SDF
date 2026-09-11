//! `report aggregate`: the `devforgeai/aggregate/1` object Reflect reads.
//!
//! Every key is present on every run; an empty result is an empty array or a
//! zero. Nothing here judges: it counts reports, differences timestamps, and
//! tallies check ids.

use crate::ctx::Ctx;
use crate::errors::CliError;
use crate::report::Report;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Fixed.
pub const SCHEMA: &str = "devforgeai/aggregate/1";

/// The `sessions.status` enum.
pub const SESSION_STATUS: &[&str] = &["present", "absent", "empty", "unreadable"];

/// The nine slash commands a session line may carry.
pub const SLASH_COMMANDS: &[&str] = &[
    "explore",
    "discover",
    "constitute",
    "plan",
    "build",
    "verify",
    "release",
    "design",
    "reflect",
];

/// The project key: the absolute project root with every character outside
/// `[A-Za-z0-9]` replaced by `-`.
pub fn project_key(root: &Path) -> String {
    crate::project::normalise(root)
        .to_string_lossy()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

/// Expand a leading `~` against the user's home directory.
fn expand_home(raw: &str) -> PathBuf {
    let Some(rest) = raw.strip_prefix('~') else {
        return PathBuf::from(raw);
    };
    let home = home_dir();
    let tail = rest.trim_start_matches(['/', '\\']);
    if tail.is_empty() {
        home
    } else {
        home.join(tail)
    }
}

/// The user's home directory, or the current directory when the environment
/// names none.
fn home_dir() -> PathBuf {
    #[cfg(feature = "test-home")]
    {
        if let Some(v) = std::env::var_os("DEVFORGEAI_HOME") {
            return PathBuf::from(v);
        }
    }
    for var in ["HOME", "USERPROFILE"] {
        if let Some(v) = std::env::var_os(var) {
            if !v.is_empty() {
                return PathBuf::from(v);
            }
        }
    }
    PathBuf::from(".")
}

/// One line the session reader recognised.
#[derive(Debug, Clone)]
struct SessionCommand {
    command: String,
    args: String,
    at: String,
    session_id: String,
    line: u32,
    remedy_ids: Vec<String>,
    resume: bool,
}

/// The whole aggregate.
pub struct Aggregate {
    /// The `data` object.
    pub data: serde_json::Value,
    /// The human lines.
    pub human: Vec<String>,
    /// The exit code.
    pub exit: i32,
    /// Warnings raised while reading.
    pub warnings: Vec<crate::errors::Diag>,
}

/// Build the aggregate for one window.
pub fn run(
    ctx: &mut Ctx,
    id: Option<&str>,
    since: Option<&str>,
    session_root: Option<&Path>,
) -> Result<Aggregate, CliError> {
    let root = ctx.root.clone();
    let generated_at = crate::time::now_rfc3339();

    // Exactly one of the two selects a window.
    let has_since = since.is_some();
    if id.is_some() == has_since {
        return Err(CliError::new(
            "DFA-E430",
            "pass one id (IDEA-nnn, EPIC-nnn, STORY-nnn, vX.Y.Z) or --since <YYYY-MM-DD>",
        ));
    }
    if let Some(subject) = id {
        if !subject_ok(subject) {
            return Err(CliError::new(
                "DFA-E013",
                format!("'{subject}' is not an ID; expected PREFIX-nnn with three digits"),
            ));
        }
    }

    let cfg = ctx.config()?.clone();
    let mut warnings: Vec<crate::errors::Diag> = Vec::new();

    // ------------------------------------------------------------- the window
    let mut wanted: Vec<String> = Vec::new();
    let mut from = String::new();
    let mode = if let Some(subject) = id {
        wanted.push(subject.to_string());
        wanted.extend(expand_subject(&root, subject));
        "id"
    } else {
        from = resolve_since(since.unwrap_or(""), cfg.reflect.window_days, &generated_at);
        "since"
    };

    let mut reports: Vec<(String, Report)> = Vec::new();
    for path in report_paths(&root) {
        let rel = crate::project::rel_display(&root, &path);
        // One unparsable report among many excludes that entry and says so.
        // It is a warning rather than an error because the envelope contract
        // ties the two bands to the exit: a non-empty `errors[]` means
        // `ok: false`, and refusing the whole window over one stray file would
        // make the command unusable on a directory holding any scratch YAML.
        // `DFA-W420` keeps `ok: true` and still names the file to repair.
        let rep = match crate::report::load(&path) {
            Ok(r) => r,
            Err(e) => {
                warnings.push(crate::errors::Diag::at(
                    "DFA-W420",
                    format!(
                        "{rel} does not parse; it is excluded from the window: {}",
                        e.diag().map(|d| d.message.clone()).unwrap_or_default()
                    ),
                    rel.clone(),
                ));
                continue;
            }
        };
        let keep = if mode == "id" {
            wanted.iter().any(|w| w == &rep.id)
        } else {
            at_or_after(&rep.finished_at, &from)
        };
        if keep {
            reports.push((rel, rep));
        }
    }
    reports.sort_by(|a, b| a.0.cmp(&b.0));

    // --------------------------------------------------------------- sessions
    let key = if cfg.reflect.session_key.is_empty() {
        project_key(&root)
    } else {
        cfg.reflect.session_key.clone()
    };
    let configured_root = match session_root {
        Some(p) => p.to_path_buf(),
        None => expand_home(&cfg.reflect.session_root),
    };
    let (sessions, session_error) = read_sessions(&configured_root, &key);
    if let Some(e) = &session_error {
        warnings.push(e.clone());
    }

    // ---------------------------------------------------------------- the data
    let report_entries: Vec<serde_json::Value> =
        reports.iter().map(|(rel, r)| report_json(rel, r)).collect();
    let phase_time = phase_time(&reports);
    let gate_failures = gate_failures(&reports);
    let send_backs = send_backs(&reports);
    let verifier_failures = verifier_failures(&reports);
    let deferrals = deferrals(&reports, &generated_at);

    let state = ctx.state()?.clone();
    let data = serde_json::json!({
        "schema": SCHEMA,
        "window": {
            "mode": mode,
            "subject": id.unwrap_or(""),
            "from": if mode == "since" { from.clone() } else { first_date(&reports) },
            "to": date_of(&generated_at),
            "generated_at": generated_at,
            "cli_version": crate::VERSION,
            "degraded": cfg.degraded,
        },
        "reports": report_entries,
        "phase_time": phase_time,
        "gate_failures": gate_failures,
        "send_backs": send_backs,
        "verifier_failures": verifier_failures,
        "deferrals": deferrals,
        "sessions": sessions,
        "state": {
            "current_phase": state.current.phase,
            "active": active_json(&state),
            "last_gate": {
                "phase": state.last_gate.phase,
                "id": state.last_gate.id,
                "result": state.last_gate.result,
                "at": state.last_gate.at,
                "failed_checks": state.last_gate.failed_checks,
            },
            "last_handoff_at": state.last_handoff.rendered_at,
        },
        "floors": floors(),
        "counts": {
            "reports": reports.len(),
            "sessions": sessions["files"].as_array().map(Vec::len).unwrap_or(0),
            "gate_failures": gate_failures.len(),
            "send_backs": send_backs.len(),
            "deferrals": deferrals.len(),
            "verifier_failures": verifier_failures.len(),
        },
    });

    let human = vec![
        format!("reports            {}", reports.len()),
        format!("phase_time         {}", phase_time.len()),
        format!("gate_failures      {}", gate_failures.len()),
        format!("send_backs         {}", send_backs.len()),
        format!("verifier_failures  {}", verifier_failures.len()),
        format!("deferrals          {}", deferrals.len()),
        format!(
            "sessions           {} ({})",
            sessions["files"].as_array().map(Vec::len).unwrap_or(0),
            sessions["status"].as_str().unwrap_or("")
        ),
    ];

    let exit = if session_error.is_some() { 1 } else { 0 };
    Ok(Aggregate {
        data,
        human,
        exit,
        warnings,
    })
}

/// `^(IDEA|EPIC|STORY)-[0-9]{3}$` or `^v[0-9]+\.[0-9]+\.[0-9]+$`.
fn subject_ok(s: &str) -> bool {
    if crate::doc::is_version(s) {
        return true;
    }
    match crate::doc::ids::split_id(s) {
        Some((prefix, _)) => matches!(prefix, "IDEA" | "EPIC" | "STORY"),
        None => false,
    }
}

/// The extra ids a version or an epic pulls into the window.
fn expand_subject(root: &Path, subject: &str) -> Vec<String> {
    if crate::doc::is_version(subject) {
        let path = root
            .join(".devforgeai")
            .join("releases")
            .join(format!("{subject}.yaml"));
        let Ok(text) = std::fs::read_to_string(&path) else {
            return Vec::new();
        };
        let Ok(v) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&text) else {
            return Vec::new();
        };
        return v
            .get("stories")
            .and_then(|s| s.as_sequence())
            .map(|s| {
                s.iter()
                    .filter_map(|e| e.get("id").and_then(|x| x.as_str()).map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
    }

    if subject.starts_with("EPIC-") {
        return crate::story::all_paths(root)
            .iter()
            .filter_map(|p| crate::story::read(root, p).ok())
            .filter(|s| s.consumes.iter().any(|c| c == subject))
            .map(|s| s.id)
            .collect();
    }
    Vec::new()
}

/// Every `.devforgeai/reports/*.yaml`, sorted.
fn report_paths(root: &Path) -> Vec<PathBuf> {
    let dir = root.join(".devforgeai").join("reports");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("yaml"))
        .collect();
    out.sort();
    out
}

/// `--since` with no date takes `[reflect].window_days` back from today.
fn resolve_since(raw: &str, window_days: i64, generated_at: &str) -> String {
    if !raw.is_empty() {
        return raw.to_string();
    }
    let Some(now) = crate::time::parse_rfc3339(generated_at) else {
        return String::new();
    };
    let then = now - time::Duration::days(window_days.max(0));
    date_of(&crate::time::format(then))
}

/// The `YYYY-MM-DD` prefix of an RFC 3339 timestamp.
fn date_of(s: &str) -> String {
    s.chars().take(10).collect()
}

/// True when `finished_at` is at or after `date` at 00:00:00Z.
fn at_or_after(finished_at: &str, date: &str) -> bool {
    if date.is_empty() {
        return true;
    }
    date_of(finished_at).as_str() >= date
}

/// The earliest `finished_at` date in the window, or `""`.
fn first_date(reports: &[(String, Report)]) -> String {
    reports
        .iter()
        .map(|(_, r)| date_of(&r.finished_at))
        .filter(|d| !d.is_empty())
        .min()
        .unwrap_or_default()
}

/// Milliseconds between two RFC 3339 timestamps, 0 when either is unreadable.
fn duration_ms(started: &str, finished: &str) -> i64 {
    let (Some(a), Some(b)) = (
        crate::time::parse_rfc3339(started),
        crate::time::parse_rfc3339(finished),
    ) else {
        return 0;
    };
    ((b - a).whole_milliseconds()).max(0) as i64
}

/// One `reports[]` entry.
fn report_json(rel: &str, r: &Report) -> serde_json::Value {
    let checks: Vec<serde_json::Value> = r
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
                "duration_ms": c.evidence.get("duration_ms")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            })
        })
        .collect();

    let verifiers: Vec<serde_json::Value> = r
        .verifiers
        .as_ref()
        .map(|m| {
            m.values()
                .map(|v| {
                    serde_json::json!({
                        "subagent": v.get("subagent").and_then(|x| x.as_str()).unwrap_or(""),
                        "status": v.get("status").and_then(|x| x.as_str()).unwrap_or("ingested"),
                        "passed": v.get("passed").and_then(|x| x.as_i64()).unwrap_or(0),
                        "total": v.get("total").and_then(|x| x.as_i64()).unwrap_or(0),
                        "unit": v.get("unit").and_then(|x| x.as_str()).unwrap_or(""),
                        "code": v.get("code").and_then(|x| x.as_str()).unwrap_or(""),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let coverage = match &r.coverage {
        Some(c) => serde_json::to_value(c).unwrap_or(serde_json::Value::Null),
        None => serde_json::Value::Null,
    };

    serde_json::json!({
        "path": rel,
        "id": r.id,
        "phase": r.phase,
        "status": r.status,
        "started_at": r.started_at,
        "finished_at": r.finished_at,
        "duration_ms": duration_ms(&r.started_at, &r.finished_at),
        "gate": {
            "result": r.gate.result,
            "send_back_to": r.gate.send_back_to,
            "failed_checks": r.gate.checks.iter()
                .filter(|c| c.status == "fail")
                .map(|c| c.id.clone())
                .collect::<Vec<_>>(),
        },
        "checks": checks,
        "verifiers": verifiers,
        "findings": r.findings.iter().map(|f| serde_json::json!({
            "id": f.id, "severity": f.severity, "summary": f.summary,
        })).collect::<Vec<_>>(),
        "coverage": coverage,
    })
}

/// `phase_time[]`, one entry per phase in the window.
fn phase_time(reports: &[(String, Report)]) -> Vec<serde_json::Value> {
    let mut by_phase: BTreeMap<String, Vec<&Report>> = BTreeMap::new();
    for (_, r) in reports {
        by_phase.entry(r.phase.clone()).or_default().push(r);
    }
    by_phase
        .into_iter()
        .map(|(phase, rs)| {
            let mut durations: Vec<i64> = rs
                .iter()
                .map(|r| duration_ms(&r.started_at, &r.finished_at))
                .collect();
            durations.sort_unstable();
            let total: i64 = durations.iter().sum();
            let median = if durations.is_empty() {
                0
            } else {
                durations[durations.len() / 2]
            };
            let mut stamps: Vec<&str> = rs
                .iter()
                .map(|r| r.finished_at.as_str())
                .filter(|s| !s.is_empty())
                .collect();
            stamps.sort_unstable();
            serde_json::json!({
                "phase": phase,
                "runs": rs.len(),
                "total_ms": total,
                "median_ms": median,
                "first_at": stamps.first().copied().unwrap_or(""),
                "last_at": stamps.last().copied().unwrap_or(""),
            })
        })
        .collect()
}

/// The accumulator of one `gate_failures[]` group: kind, phase, count, the
/// subject ids, and the distinct reasons.
type FailureGroup = (String, String, usize, Vec<String>, Vec<String>);

/// The accumulator of one `send_backs[]` group: count, the timestamps, the
/// failing check ids, and the finding ids.
type SendBackGroup = (usize, Vec<String>, Vec<String>, Vec<String>);

/// `gate_failures[]`, grouped by check id.
fn gate_failures(reports: &[(String, Report)]) -> Vec<serde_json::Value> {
    let mut by_check: BTreeMap<String, FailureGroup> = BTreeMap::new();
    for (_, r) in reports {
        for c in r.gate.checks.iter().filter(|c| c.status == "fail") {
            let e = by_check.entry(c.id.clone()).or_insert((
                c.kind.clone(),
                r.phase.clone(),
                0,
                Vec::new(),
                Vec::new(),
            ));
            e.2 += 1;
            if !e.3.contains(&r.id) {
                e.3.push(r.id.clone());
            }
            if !c.reason.is_empty() && !e.4.contains(&c.reason) {
                e.4.push(c.reason.clone());
            }
        }
    }
    by_check
        .into_iter()
        .map(|(check_id, (kind, phase, count, ids, reasons))| {
            serde_json::json!({
                "check_id": check_id,
                "kind": kind,
                "phase": phase,
                "count": count,
                "ids": ids,
                "reasons": reasons,
            })
        })
        .collect()
}

/// `send_backs[]`, grouped by the from/to/id triple.
fn send_backs(reports: &[(String, Report)]) -> Vec<serde_json::Value> {
    let mut by_key: BTreeMap<(String, String, String), SendBackGroup> = BTreeMap::new();
    for (_, r) in reports {
        if r.gate.result != "SEND_BACK" {
            continue;
        }
        let key = (r.phase.clone(), r.gate.send_back_to.clone(), r.id.clone());
        let e = by_key
            .entry(key)
            .or_insert((0, Vec::new(), Vec::new(), Vec::new()));
        e.0 += 1;
        if !r.finished_at.is_empty() {
            e.1.push(r.finished_at.clone());
        }
        for c in r.gate.checks.iter().filter(|c| c.status == "fail") {
            if !e.2.contains(&c.id) {
                e.2.push(c.id.clone());
            }
        }
        for f in &r.findings {
            if !e.3.contains(&f.id) {
                e.3.push(f.id.clone());
            }
        }
    }
    by_key
        .into_iter()
        .map(|((from, to, id), (count, at, check_ids, finding_ids))| {
            serde_json::json!({
                "from": from,
                "to": to,
                "id": id,
                "count": count,
                "at": at,
                "check_ids": check_ids,
                "finding_ids": finding_ids,
            })
        })
        .collect()
}

/// `verifier_failures[]`: every ingested block that is not clean.
fn verifier_failures(reports: &[(String, Report)]) -> Vec<serde_json::Value> {
    let mut by_key: BTreeMap<(String, String, String, String), usize> = BTreeMap::new();
    for (rel, r) in reports {
        let Some(m) = &r.verifiers else { continue };
        for v in m.values() {
            let status = v
                .get("status")
                .and_then(|x| x.as_str())
                .unwrap_or("ingested")
                .to_string();
            let passed = v.get("passed").and_then(|x| x.as_i64()).unwrap_or(0);
            let total = v.get("total").and_then(|x| x.as_i64()).unwrap_or(0);
            let clean = status == "ingested" && (total == 0 || passed >= total);
            if clean {
                continue;
            }
            let subagent = v
                .get("subagent")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let code = v
                .get("code")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            *by_key
                .entry((subagent, status, code, rel.clone()))
                .or_insert(0) += 1;
        }
    }
    by_key
        .into_iter()
        .map(|((subagent, status, code, report), count)| {
            serde_json::json!({
                "subagent": subagent,
                "status": status,
                "code": code,
                "report": report,
                "count": count,
            })
        })
        .collect()
}

/// `deferrals[]`, read from each qa report's top-level `deferrals[]`.
fn deferrals(reports: &[(String, Report)], generated_at: &str) -> Vec<serde_json::Value> {
    let now = crate::time::parse_rfc3339(generated_at);
    let mut out = Vec::new();
    for (rel, r) in reports {
        let Some(seq) = r.extra.get("deferrals").and_then(|v| v.as_sequence()) else {
            continue;
        };
        for e in seq {
            let s = |key: &str| {
                e.get(key)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string()
            };
            let opened_on = s("opened_on");
            let age_days = match (
                &now,
                crate::time::parse_rfc3339(&format!("{opened_on}T00:00:00Z")),
            ) {
                (Some(n), Some(then)) if !opened_on.is_empty() => {
                    crate::time::calendar_days_between(then, *n)
                }
                _ => 0,
            };
            out.push(serde_json::json!({
                "id": s("id"),
                "story": s("story"),
                "dod_item": s("dod_item"),
                "deferred_at": opened_on,
                "age_days": age_days,
                "constraint": s("con_or_ap"),
                "reason": s("reason"),
                "report": rel,
            }));
        }
    }
    out
}

/// The `[active]` table as an object.
fn active_json(state: &crate::state::State) -> serde_json::Value {
    let mut m = serde_json::Map::new();
    for phase in crate::state::PHASES {
        if let Some(v) = state.active.get(phase) {
            if !v.is_empty() {
                m.insert(
                    (*phase).to_string(),
                    serde_json::Value::String(v.to_string()),
                );
            }
        }
    }
    serde_json::Value::Object(m)
}

/// The compiled minimums of both files.
fn floors() -> serde_json::Value {
    let mut config = serde_json::Map::new();
    for (name, floor) in crate::config::LAYER_FLOORS {
        config.insert(
            format!("layer.{name}.coverage_min"),
            serde_json::Value::from(*floor),
        );
    }
    config.insert(
        "coverage.overall_min".to_string(),
        serde_json::Value::from(crate::config::OVERALL_FLOOR),
    );
    serde_json::json!({
        "config.toml": config,
        "gates.toml": { "verifier_pass.min_ratio": crate::gates::min_ratio_floor("verify").unwrap_or(1.0) },
    })
}

// ------------------------------------------------------------------ sessions

/// Read `<root>/<key>/*.jsonl`, reporting the status rather than failing.
fn read_sessions(root: &Path, key: &str) -> (serde_json::Value, Option<crate::errors::Diag>) {
    let empty = |status: &str, reason: &str| {
        serde_json::json!({
            "status": status,
            "root": root.display().to_string().replace('\\', "/"),
            "key": key,
            "reason": reason,
            "files": [],
            "commands": [],
            "repeats": [],
        })
    };

    // A root that does not exist has nothing to be outside of. `normalise`
    // falls back to joining a relative path onto the working directory, so a
    // configured `sessions/` that was never created resolved to a path under
    // the cwd and was then refused as "outside the home directory" — a
    // confusing DFA-E421 for what is simply an absent directory. Absence is
    // reported as absence, at exit 0, and the containment rule is applied to
    // roots that exist and could therefore be read.
    if !root.exists() {
        return (
            empty("absent", "no session directory for this project"),
            None,
        );
    }

    // A session root outside the user's home directory is refused.
    let home = crate::project::normalise(&home_dir());
    let resolved = crate::project::normalise(root);
    if !resolved.starts_with(&home) {
        return (
            empty(
                "unreadable",
                "the session root is outside the home directory",
            ),
            Some(crate::errors::Diag::at(
                "DFA-E421",
                format!(
                    "{} is outside the home directory {}",
                    resolved.display(),
                    home.display()
                ),
                resolved.display().to_string(),
            )),
        );
    }

    let dir = resolved.join(key);
    if !dir.is_dir() {
        return (
            empty("absent", "no session directory for this project"),
            None,
        );
    }

    let Ok(entries) = std::fs::read_dir(&dir) else {
        return (
            empty("absent", "no session directory for this project"),
            None,
        );
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("jsonl"))
        .collect();
    paths.sort();
    if paths.is_empty() {
        return (
            empty("empty", "the session directory holds no jsonl file"),
            None,
        );
    }

    let mut files = Vec::new();
    let mut commands: Vec<SessionCommand> = Vec::new();
    for path in &paths {
        let session_id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        let mut stamps: Vec<String> = Vec::new();
        let mut lines = 0u32;
        for (i, raw) in text.lines().enumerate() {
            if raw.trim().is_empty() {
                continue;
            }
            lines += 1;
            let Ok(v) = serde_json::from_str::<serde_json::Value>(raw) else {
                continue;
            };
            if let Some(at) = v.get("timestamp").and_then(|x| x.as_str()) {
                stamps.push(at.to_string());
            }
            if let Some(c) = session_command(&v, &session_id, i as u32 + 1) {
                commands.push(c);
            }
        }
        stamps.sort();
        files.push(serde_json::json!({
            "session_id": session_id,
            "path": crate::project::normalise(path).display().to_string().replace('\\', "/"),
            "from": stamps.first().cloned().unwrap_or_default(),
            "to": stamps.last().cloned().unwrap_or_default(),
            "lines": lines,
        }));
    }

    let repeats = repeats(&commands);
    let sessions = serde_json::json!({
        "status": "present",
        "root": resolved.display().to_string().replace('\\', "/"),
        "key": key,
        "reason": "",
        "files": files,
        "commands": commands.iter().map(|c| serde_json::json!({
            "command": c.command,
            "args": c.args,
            "at": c.at,
            "session_id": c.session_id,
            "line": c.line,
            "remedy_ids": c.remedy_ids,
            "resume": c.resume,
        })).collect::<Vec<_>>(),
        "repeats": repeats,
    });
    (sessions, None)
}

/// One session line as a slash command, when it is one.
fn session_command(v: &serde_json::Value, session_id: &str, line: u32) -> Option<SessionCommand> {
    if v.get("type").and_then(|x| x.as_str()) != Some("user") {
        return None;
    }
    if v.get("isMeta").and_then(|x| x.as_bool()).unwrap_or(false) {
        return None;
    }
    let content = v
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())?
        .trim()
        .to_string();

    let rest = content.strip_prefix('/')?;
    let (name, args) = match rest.split_once(char::is_whitespace) {
        Some((n, a)) => (n.to_string(), a.trim().to_string()),
        None => (rest.to_string(), String::new()),
    };
    if !SLASH_COMMANDS.contains(&name.as_str()) {
        return None;
    }

    let mut remedy_ids: Vec<String> = Vec::new();
    let tokens: Vec<&str> = args.split_whitespace().collect();
    for (i, t) in tokens.iter().enumerate() {
        if *t == "--remedy" {
            if let Some(list) = tokens.get(i + 1) {
                remedy_ids.extend(list.split(',').map(|s| s.trim().to_string()));
            }
        } else if let Some(list) = t.strip_prefix("--remedy=") {
            remedy_ids.extend(list.split(',').map(|s| s.trim().to_string()));
        }
    }
    remedy_ids.retain(|s| !s.is_empty());
    remedy_ids.sort();
    let resume = tokens.contains(&"--resume");

    Some(SessionCommand {
        command: format!("/{name}"),
        args,
        at: v
            .get("timestamp")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
        session_id: session_id.to_string(),
        line,
        remedy_ids,
        resume,
    })
}

/// The accumulator of one `repeats[]` group: count, the session ids, the lines.
type RepeatGroup = (usize, Vec<String>, Vec<u32>);

/// `repeats[]`: groups of two or more with the same command and key.
fn repeats(commands: &[SessionCommand]) -> Vec<serde_json::Value> {
    let mut by_key: BTreeMap<(String, String), RepeatGroup> = BTreeMap::new();
    for c in commands {
        let first_arg = c.args.split_whitespace().next().unwrap_or("").to_string();
        let key = if c.remedy_ids.is_empty() {
            first_arg
        } else {
            format!("{first_arg}|{}", c.remedy_ids.join("|"))
        };
        let e = by_key
            .entry((c.command.clone(), key))
            .or_insert((0, Vec::new(), Vec::new()));
        e.0 += 1;
        if !e.1.contains(&c.session_id) {
            e.1.push(c.session_id.clone());
        }
        e.2.push(c.line);
    }
    by_key
        .into_iter()
        .filter(|(_, (count, _, _))| *count >= 2)
        .map(|((command, key), (count, session_ids, lines))| {
            serde_json::json!({
                "command": command,
                "key": key,
                "count": count,
                "session_ids": session_ids,
                "lines": lines,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_project_key_replaces_every_character_outside_the_alphabet() {
        assert_eq!(
            project_key(Path::new("C:/Projects/DevForgeAI")),
            "C--Projects-DevForgeAI"
        );
    }

    #[test]
    fn a_subject_is_one_of_the_four_shapes() {
        assert!(subject_ok("IDEA-001"));
        assert!(subject_ok("EPIC-002"));
        assert!(subject_ok("STORY-014"));
        assert!(subject_ok("v0.3.0"));
        assert!(!subject_ok("REQ-001"));
        assert!(!subject_ok("STORY-14"));
    }

    #[test]
    fn a_duration_is_the_difference_in_milliseconds() {
        assert_eq!(
            duration_ms("2026-09-03T10:58:11Z", "2026-09-03T11:02:41Z"),
            270000
        );
        assert_eq!(duration_ms("", "2026-09-03T11:02:41Z"), 0);
    }

    #[test]
    fn a_date_window_compares_on_the_date_alone() {
        assert!(at_or_after("2026-09-03T11:02:41Z", "2026-09-03"));
        assert!(!at_or_after("2026-09-02T23:59:59Z", "2026-09-03"));
        assert!(at_or_after("anything", ""));
    }

    fn command_line(content: &str) -> Option<SessionCommand> {
        let v = serde_json::json!({
            "type": "user",
            "timestamp": "2026-09-01T09:12:00Z",
            "message": { "content": content },
        });
        session_command(&v, "s1", 7)
    }

    #[test]
    fn a_slash_command_line_is_read_with_its_remedy_ids() {
        let c = command_line("/explore IDEA-001 --remedy FLOW-002,FLOW-001").expect("a command");
        assert_eq!(c.command, "/explore");
        assert_eq!(c.args, "IDEA-001 --remedy FLOW-002,FLOW-001");
        assert_eq!(c.remedy_ids, vec!["FLOW-001", "FLOW-002"], "sorted");
        assert!(!c.resume);
        assert_eq!(c.line, 7);
    }

    #[test]
    fn a_resume_flag_is_recorded() {
        let c = command_line("/build STORY-014 --resume").expect("a command");
        assert!(c.resume);
    }

    #[test]
    fn a_line_that_is_not_a_slash_command_is_skipped() {
        assert!(command_line("please build it").is_none());
        assert!(command_line("/invented IDEA-001").is_none());
    }

    #[test]
    fn a_meta_line_is_skipped() {
        let v = serde_json::json!({
            "type": "user",
            "isMeta": true,
            "message": { "content": "/explore IDEA-001" },
        });
        assert!(session_command(&v, "s1", 1).is_none());
    }

    #[test]
    fn an_assistant_line_is_skipped() {
        let v = serde_json::json!({
            "type": "assistant",
            "message": { "content": "/explore IDEA-001" },
        });
        assert!(session_command(&v, "s1", 1).is_none());
    }

    #[test]
    fn repeats_hold_only_groups_of_two_or_more() {
        let one = command_line("/explore IDEA-001 --remedy FLOW-002").expect("a command");
        let two = command_line("/explore IDEA-001 --remedy FLOW-002").expect("a command");
        let alone = command_line("/plan SPRINT-001").expect("a command");
        let r = repeats(&[one, two, alone]);
        assert_eq!(r.len(), 1);
        assert_eq!(r[0]["command"], "/explore");
        assert_eq!(r[0]["key"], "IDEA-001|FLOW-002");
        assert_eq!(r[0]["count"], 2);
    }
}
