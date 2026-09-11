//! The `hook run <event>` dispatcher, reading the Claude Code hook JSON from
//! stdin and answering on the channel the event actually has.
//!
//! The reference fixes which events block and which cannot: exit 2 blocks on
//! `PreToolUse`, `UserPromptExpansion`, `Stop`, and `SubagentStop` and nowhere
//! else. On every other event the channel to a reader is the hook's own JSON —
//! `systemMessage` for the user, `hookSpecificOutput.additionalContext` for
//! Claude — so a diagnostic written to stdout or stderr at exit 0 reaches the
//! debug log and no one else.

use crate::cli::{DocValidateArgs, GateCheckArgs, ReportIngestArgs};
use crate::ctx::Ctx;
use crate::errors::{CliError, Diag};
use crate::{cmd, trust, Outcome};
use serde_json::{json, Value};
use std::path::Path;

/// The events the dispatcher accepts.
pub const EVENTS: &[&str] = &[
    "session-start",
    "prompt-expansion",
    "pre-tool-use",
    "trust-check",
    "post-tool-use",
    "stop",
    "subagent-stop",
];

/// The command names whose expansion carries a predecessor gate, with the ID
/// prefixes that name a subject the gate can resolve.
///
/// `explore` is absent because it has no predecessor; `design` and `reflect`
/// are cross-cutting and have none either. The prefix list is what keeps the
/// arm quiet on the invocations that carry no id at all: `/explore "<idea>"`,
/// `/design --sketch …`, `/reflect --since <date>`, and a bare `/reflect`.
const EXPANSION_GATES: &[(&str, &[&str])] = &[
    ("discover", &["IDEA"]),
    ("constitute", &["IDEA"]),
    ("plan", &["EPIC", "SPRINT"]),
    ("build", &["STORY"]),
    ("verify", &["STORY"]),
    ("release", &[]),
];

/// How many times one Stop may block in a session before it lets the turn end.
///
/// The harness overrides a Stop hook after eight consecutive blocks and ends
/// the turn with nothing shown. Stopping at three keeps the decision here, so
/// the closing block is rendered by this hook rather than discarded by the cap.
/// `stop_hook_active` is not read: this budget is strictly tighter than the
/// ceiling that flag exists to keep a hook clear of.
const STOP_BLOCK_CAP: i64 = 3;

/// The cap the reference puts on a hook's output string, after which the
/// harness writes the text to a file and hands Claude a preview instead.
const OUTPUT_CAP: usize = 10_000;

/// True when this event blocks with exit 2, so an internal error, a trust
/// failure, or a refusal has an enforcing channel.
fn blocks(event: &str) -> bool {
    matches!(
        event,
        "pre-tool-use" | "trust-check" | "stop" | "subagent-stop" | "prompt-expansion"
    )
}

/// The `hookEventName` an event's `hookSpecificOutput` carries.
fn hook_event_name(event: &str) -> &'static str {
    match event {
        "session-start" => "SessionStart",
        "prompt-expansion" => "UserPromptExpansion",
        "pre-tool-use" | "trust-check" => "PreToolUse",
        "post-tool-use" => "PostToolUse",
        "stop" => "Stop",
        "subagent-stop" => "SubagentStop",
        _ => "",
    }
}

/// A `PreToolUse` denial object.
fn deny(reason: &str) -> Value {
    json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": cap(reason),
        }
    })
}

/// A top-level blocking decision, which is the form `Stop`, `SubagentStop`,
/// and `UserPromptExpansion` read.
fn block_decision(reason: &str) -> Value {
    json!({ "decision": "block", "reason": cap(reason) })
}

/// Truncate to the reference's output cap, marking what was dropped.
fn cap(text: &str) -> String {
    if text.len() <= OUTPUT_CAP {
        return text.to_string();
    }
    let mut end = OUTPUT_CAP;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}… [+{} chars]", &text[..end], text.len() - end)
}

/// The parsed hook payload.
struct Payload(Value);

impl Payload {
    fn parse(stdin: &str) -> Result<Payload, CliError> {
        let v: Value = serde_json::from_str(stdin.trim())
            .map_err(|e| CliError::new("DFA-E021", format!("stdin is not JSON: {e}")))?;
        Ok(Payload(v))
    }

    fn str_key(&self, key: &str, event: &str) -> Result<String, CliError> {
        self.0
            .get(key)
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| {
                CliError::new(
                    "DFA-E020",
                    format!("hook input has no key '{key}' for event {event}"),
                )
            })
    }

    fn opt_str(&self, path: &[&str]) -> Option<String> {
        let mut cur = &self.0;
        for seg in path {
            cur = cur.get(seg)?;
        }
        cur.as_str().map(str::to_string)
    }

    /// A field that is either a string or an array of typed content blocks, as
    /// `last_assistant_message` may be. The `text` of every `text` block is
    /// concatenated; anything else yields `None`.
    fn opt_text(&self, key: &str) -> Option<String> {
        let v = self.0.get(key)?;
        text_of(v)
    }
}

/// The text of a string or of an array of content blocks.
fn text_of(v: &Value) -> Option<String> {
    if let Some(s) = v.as_str() {
        return Some(s.to_string());
    }
    let blocks = v.as_array()?;
    let mut out = String::new();
    for b in blocks {
        if b.get("type").and_then(Value::as_str) != Some("text") {
            continue;
        }
        if let Some(t) = b.get("text").and_then(Value::as_str) {
            out.push_str(t);
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Dispatch one hook event.
pub fn dispatch(ctx: &mut Ctx, event: &str, stdin: &str) -> Result<Outcome, CliError> {
    if !EVENTS.contains(&event) {
        return Err(CliError::new(
            "DFA-E012",
            format!(
                "'{event}' is not a hook event; expected one of {}",
                EVENTS.join(", ")
            ),
        ));
    }

    // The payload is parsed before the trust check, because the Stop trust
    // branch needs `session_id` to charge the same block budget the gate
    // branch charges.
    let p = match Payload::parse(stdin) {
        Ok(p) => p,
        Err(e) => return fail_closed(ctx, event, &e),
    };

    // Every event runs `trust verify` first, per conventions section 8.
    if let Err(e) = trust::verify(None) {
        return trust_failure(ctx, event, &p, &e);
    }

    let result = match event {
        "session-start" => session_start(ctx, &p),
        "prompt-expansion" => prompt_expansion(ctx, &p),
        "pre-tool-use" => pre_tool_use(ctx, &p),
        "trust-check" => Ok(outcome("trust-check", &["trust verify"], 0, false)),
        "post-tool-use" => post_tool_use(ctx, &p),
        "stop" => stop(ctx, &p),
        "subagent-stop" => subagent_stop(ctx, &p),
        _ => unreachable!("the event enum is closed above"),
    };

    match result {
        Ok(o) => Ok(o),
        // A blocking event that cannot evaluate its own rule refuses, rather
        // than exiting 1, 3, or 5, each of which the harness reads as a
        // non-blocking error and lets the action through.
        Err(e) => fail_closed(ctx, event, &e),
    }
}

/// An internal error on a blocking event becomes a refusal; on every other
/// event no exit code helps, so the error propagates as it stands.
fn fail_closed(ctx: &mut Ctx, event: &str, e: &CliError) -> Result<Outcome, CliError> {
    if !blocks(event) {
        return Err(e.clone());
    }
    let diag = e.diag().cloned();
    let detail = diag
        .as_ref()
        .map(|d| format!("{} {}", d.code, d.message))
        .unwrap_or_else(|| "no diagnostic".to_string());
    let reason = format!("devforgeai could not evaluate this event: {detail}");

    let mut out = outcome(event, &["fail-closed"], 2, true);
    out.hook_json = Some(if hook_event_name(event) == "PreToolUse" {
        deny(&reason)
    } else {
        block_decision(&reason)
    });
    out.stderr = vec![reason];
    if let Some(d) = diag {
        out.errors.push(d);
    }
    out.project = ctx.root.display().to_string();
    Ok(out)
}

/// Charge one Stop block against this session's budget.
///
/// Returns true when the block is allowed, and records it. The budget is per
/// session and on nothing else: the subject is exactly what a continuation
/// turn can change, so keying on it lets an advancing story refill the budget
/// and block without end. Both reasons a Stop can refuse — a failing gate and
/// a trust failure — draw on this one counter, because there is one turn to
/// hold open and the harness's own ceiling of eight ends it either way.
///
/// `blocked_phase` and `blocked_id` record which subject was blocked last;
/// they key nothing.
fn charge_block_budget(ctx: &mut Ctx, session_id: &str, phase: &str, id: &str) -> bool {
    let Ok(s) = ctx.state_mut() else {
        // The state file is unreadable. Allowing the block is the fail-closed
        // answer: the turn is held rather than let through unchecked.
        return true;
    };
    if s.stop_hook.blocked_session != session_id {
        s.stop_hook.blocked_session = session_id.to_string();
        s.stop_hook.block_count = 0;
    }
    if s.stop_hook.block_count >= STOP_BLOCK_CAP {
        return false;
    }
    s.stop_hook.blocked_phase = phase.to_string();
    s.stop_hook.blocked_id = id.to_string();
    s.stop_hook.block_count += 1;
    true
}

/// The pin command an operator runs to repair a trust failure.
fn pin_command() -> String {
    let framework = trust::load_trust_file()
        .ok()
        .flatten()
        .and_then(|t| t.pin.first().map(|p| p.framework_path.clone()))
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| "<framework root>".to_string());
    format!("devforgeai trust pin --framework {framework}")
}

/// A trust failure, answered on the channel the event has.
fn trust_failure(
    ctx: &mut Ctx,
    event: &str,
    p: &Payload,
    e: &CliError,
) -> Result<Outcome, CliError> {
    let code = e.code().to_string();
    let detail = e
        .diag()
        .map(|d| d.message.clone())
        .unwrap_or_else(|| "trust verify failed".to_string());
    let pin = pin_command();
    let blocking = blocks(event);
    let exit = if blocking { 2 } else { 4 };

    // `[last_gate].result` records the failure so the next handoff renders
    // `Gate      TRUST FAIL`. It is a record, not the channel the refusal
    // depends on.
    if let Ok(s) = ctx.state_mut() {
        s.last_gate.result = "TRUST_FAIL".to_string();
        s.last_gate.failed_checks = vec![code.clone()];
    }
    // A Stop that refuses on trust spends the same budget a Stop that refuses
    // on a gate does. There is one counter because there is one turn: blocking
    // for ever cannot repair a pin, and a trust failure that arrives mid-run —
    // the source digest changing while the framework's own sources are being
    // edited — otherwise blocked every Stop until the harness's eight-block
    // cap ended the session with nothing shown.
    let stop_may_block = if event == "stop" {
        let session_id = p.opt_str(&["session_id"]).unwrap_or_default();
        let (phase, id) = ctx
            .state()
            .map(|s| (s.current.phase.clone(), s.current.id.clone()))
            .unwrap_or_default();
        charge_block_budget(ctx, &session_id, &phase, &id)
    } else {
        true
    };
    let store = ctx.store_state();

    let reason = format!(
        "devforgeai trust verify failed: {code} {detail}. \
         In a terminal outside Claude Code, run: {pin}. Then start a new session."
    );

    let mut out = outcome(event, &["trust verify"], exit, blocking);
    out.hook_json = Some(match event {
        "pre-tool-use" | "trust-check" => deny(&reason),
        "stop" => {
            let system = format!(
                "Gate      TRUST FAIL  {code}\n\
                 Blocked   you: run {pin} outside Claude Code\n\n\
                 No gate ran for this turn."
            );
            if stop_may_block {
                json!({
                    "decision": "block",
                    "reason": cap(&format!("{reason} No gate ran for this turn.")),
                    "systemMessage": system,
                })
            } else {
                // The budget is spent. Nothing a continuation can do repairs a
                // pin, so the turn ends holding the refusal rather than looping
                // against it until the harness overrides the hook and shows
                // nothing at all.
                out.exit = Some(0);
                out.data["exit"] = json!(0);
                out.data["blocked"] = json!(false);
                json!({ "systemMessage": system })
            }
        }
        "subagent-stop" | "prompt-expansion" => block_decision(&reason),
        // Neither event honours any exit code, so the user-visible notice is
        // the only channel left.
        _ => json!({ "systemMessage": cap(&reason) }),
    });
    out.stderr = vec![reason];
    if let Some(d) = e.diag() {
        out.warnings.push(d.clone());
    }
    if let Err(se) = store {
        if let Some(d) = se.diag() {
            out.warnings.push(d.clone());
        }
    }
    out.project = ctx.root.display().to_string();
    Ok(out)
}

fn outcome(event: &str, actions: &[&str], exit: i32, blocked: bool) -> Outcome {
    let mut out = Outcome::data(json!({
        "event": event,
        "actions": actions,
        "blocked": blocked,
        "exit": exit,
        "block_count": 0,
    }));
    out.exit = Some(exit);
    out
}

// ---------------------------------------------------------------- path keys

/// The project-relative form of `p`, or `None` when `p` is not under `root`.
///
/// Windows path comparison is the defect this closes: `Path::strip_prefix` is a
/// case-sensitive component compare, while Claude emits a lower-case drive
/// letter freely and the root is canonicalised to the on-disk case. A short
/// name, a junction, and a UNC spelling name the same file too, so the path is
/// canonicalised — through its parent when the file does not exist yet, which
/// is the ordinary case on `PreToolUse` — before the segments are compared.
pub fn rel_under(root: &Path, p: &Path) -> Option<String> {
    let root = resolve(root);
    let p = resolve(p);
    let root_parts: Vec<String> = parts(&root);
    let p_parts: Vec<String> = parts(&p);
    if p_parts.len() < root_parts.len() {
        return None;
    }
    for (a, b) in root_parts.iter().zip(p_parts.iter()) {
        if !same_segment(a, b) {
            return None;
        }
    }
    Some(p_parts[root_parts.len()..].join("/"))
}

/// A hook path normalised for segment matching: forward slashes, lower-cased
/// where the filesystem is case-insensitive, with a leading and a trailing
/// slash so a segment test cannot match a partial name.
pub fn hook_path_key(rel: &str) -> String {
    let slashed = rel.replace('\\', "/");
    let folded = if cfg!(windows) {
        slashed.to_lowercase()
    } else {
        slashed
    };
    format!("/{}/", folded.trim_matches('/'))
}

/// True when a project-relative path names a file under `.devforgeai/`.
pub fn under_dot(rel: &str) -> bool {
    hook_path_key(rel).contains("/.devforgeai/")
}

fn same_segment(a: &str, b: &str) -> bool {
    if cfg!(windows) {
        a.eq_ignore_ascii_case(b)
    } else {
        a == b
    }
}

fn parts(p: &Path) -> Vec<String> {
    p.to_string_lossy()
        .replace('\\', "/")
        .split('/')
        .filter(|s| !s.is_empty() && *s != ".")
        .map(str::to_string)
        .collect()
}

/// The canonical form of `p`, falling back to the canonical parent plus the
/// file name for a path that does not exist yet, and to `p` itself otherwise.
fn resolve(p: &Path) -> std::path::PathBuf {
    let strip = |q: std::path::PathBuf| -> std::path::PathBuf {
        let s = q.to_string_lossy().to_string();
        std::path::PathBuf::from(s.strip_prefix(r"\\?\").unwrap_or(&s).to_string())
    };
    if let Ok(c) = std::fs::canonicalize(p) {
        return strip(c);
    }
    if let (Some(parent), Some(name)) = (p.parent(), p.file_name()) {
        if let Ok(c) = std::fs::canonicalize(parent) {
            return strip(c).join(name);
        }
    }
    p.to_path_buf()
}

// ------------------------------------------------------------ session start

fn session_start(ctx: &mut Ctx, _p: &Payload) -> Result<Outcome, CliError> {
    let mut human: Vec<String> = Vec::new();
    let mut warnings: Vec<Diag> = Vec::new();

    match cmd::stack::detect(ctx, false) {
        Ok(o) => human.extend(o.human),
        Err(e) => {
            if let Some(d) = e.diag() {
                warnings.push(d.clone());
            }
        }
    }
    match cmd::handoff::run(ctx, None, None) {
        Ok(o) => human.extend(o.human),
        Err(e) => {
            if let Some(d) = e.diag() {
                warnings.push(d.clone());
            }
        }
    }

    // Plain stdout: this event adds stdout to Claude's context, which is the
    // audience for a session-opening block. No `systemMessage`, because the
    // user has just opened the session and did not ask for it.
    let mut out = outcome("session-start", &["stack detect", "handoff"], 0, false);
    out.human = cap_lines(human);
    out.warnings = warnings;
    out.project = ctx.root.display().to_string();
    Ok(out)
}

/// Trim a block of lines to the reference's output cap.
fn cap_lines(lines: Vec<String>) -> Vec<String> {
    let mut total = 0usize;
    let mut kept = Vec::new();
    for l in lines {
        total += l.len() + 1;
        if total > OUTPUT_CAP {
            kept.push("… [output capped]".to_string());
            break;
        }
        kept.push(l);
    }
    kept
}

// --------------------------------------------------------- prompt expansion

fn prompt_expansion(ctx: &mut Ctx, p: &Payload) -> Result<Outcome, CliError> {
    let name = p
        .opt_str(&["command_name"])
        .unwrap_or_default()
        .trim()
        .trim_start_matches('/')
        .to_string();
    let quiet = || Ok(outcome("prompt-expansion", &["trust verify"], 0, false));

    let Some((_, prefixes)) = EXPANSION_GATES.iter().find(|(n, _)| *n == name) else {
        // `explore` has no predecessor; `design` and `reflect` are
        // cross-cutting; anything else is not ours.
        return quiet();
    };

    let args = p.opt_str(&["command_args"]).unwrap_or_default();
    let id = args.split_whitespace().next().unwrap_or("").to_string();
    // A first argument that is not a subject this phase's gate can resolve —
    // a quoted idea, a flag, a date, nothing at all — leaves the expansion
    // alone. The skill's own preamble raises the usage error where the
    // argument belongs, and blocking here would refuse the invocations that
    // legitimately carry no id.
    if !subject_matches(&id, prefixes) {
        return quiet();
    }

    let phase = name.as_str();
    match cmd::gate::require(ctx, phase, &id) {
        Ok(_) => {
            // A pass emits nothing, so the expansion proceeds untouched.
            let mut out = outcome(
                "prompt-expansion",
                &["trust verify", "gate require"],
                0,
                false,
            );
            out.project = ctx.root.display().to_string();
            Ok(out)
        }
        Err(e) => {
            let d = e.diag().cloned();
            // The skill body never loads, so this `reason` is the only
            // navigation the user gets: it names the command that repairs the
            // state, not only the diagnostic.
            // `gate require` already appends the repair command to its own
            // `DFA-E321` message, so only the no-diagnostic arm adds one.
            let reason = match &d {
                Some(d) => format!(
                    "{} {}\nThe predecessor gate for /{name} {id} has not passed, so the skill was not loaded.",
                    d.code, d.message
                ),
                None => format!(
                    "/{name} {id} is blocked by its predecessor gate. Run: {}",
                    repair_command(phase, &id)
                ),
            };
            let mut out = outcome(
                "prompt-expansion",
                &["trust verify", "gate require"],
                2,
                true,
            );
            out.hook_json = Some(block_decision(&reason));
            out.stderr = vec![reason];
            if let Some(d) = d {
                out.errors.push(d);
            }
            out.project = ctx.root.display().to_string();
            Ok(out)
        }
    }
}

/// True when `id` is a subject this phase's gate can resolve.
///
/// An empty `prefixes` list means the phase takes a version rather than an ID,
/// which is `release`.
fn subject_matches(id: &str, prefixes: &[&str]) -> bool {
    if id.is_empty() || id.starts_with('-') || id.starts_with('"') {
        return false;
    }
    if prefixes.is_empty() {
        return crate::doc::is_version(id);
    }
    match crate::doc::ids::split_id(id) {
        Some((p, _)) => prefixes.contains(&p),
        None => false,
    }
}

/// The command that repairs a phase whose predecessor gate has not passed:
/// the predecessor's own slash command, which is what the user types next.
fn repair_command(phase: &str, id: &str) -> String {
    match phase {
        "discover" => format!("/explore, then /discover {id}"),
        "constitute" => format!("/discover {id}, then /constitute {id}"),
        "plan" => "/constitute <IDEA-nnn>, then /plan again".to_string(),
        "build" => "/plan <EPIC-nnn>, then /build again".to_string(),
        "verify" => format!("/build {id}, then /verify {id}"),
        "release" => format!("/verify each story, then /release {id}"),
        _ => format!("devforgeai gate check --phase {phase} --id {id}"),
    }
}

// ------------------------------------------------------------- pre tool use

/// The shell constructs that can land bytes in a file, with the token that
/// introduces the target path.
const SHELL_WRITES: &[&str] = &[
    "tee",
    "Out-File",
    "Set-Content",
    "Add-Content",
    "New-Item",
    "Copy-Item",
    "Move-Item",
    "cp",
    "mv",
];

fn pre_tool_use(ctx: &mut Ctx, p: &Payload) -> Result<Outcome, CliError> {
    let tool = p.str_key("tool_name", "pre-tool-use")?;
    if tool == "Bash" || tool == "PowerShell" {
        return pre_tool_use_shell(ctx, p);
    }

    // `NotebookEdit` names its target `notebook_path`; every other write tool
    // uses `file_path`. An addition to the matcher whose input shape differs
    // must not turn every call into a non-blocking hook error.
    let Some(file_path) = p
        .opt_str(&["tool_input", "file_path"])
        .or_else(|| p.opt_str(&["tool_input", "notebook_path"]))
    else {
        return Err(CliError::new(
            "DFA-E020",
            "hook input has no key 'tool_input.file_path' for event pre-tool-use",
        ));
    };

    let path = Path::new(&file_path);
    let Some(rel) = rel_under(&ctx.root, path) else {
        return Ok(outside_root(ctx, path));
    };

    // Every arm is handed the path rebuilt from the resolved relative form,
    // so the subcommands below see the on-disk spelling rather than whatever
    // casing or short name the payload carried.
    let path = ctx.root.join(&rel);
    let path = path.as_path();

    let mut actions: Vec<&str> = Vec::new();
    let mut warnings: Vec<Diag> = Vec::new();
    let mut blocked = false;
    let inside = under_dot(&rel);

    if inside {
        actions.push("doc validate --producer-check");
        // Write sends the whole content with `--stdin-content`; Edit sends
        // none, because `new_string` is a fragment rather than a document.
        let content = if tool == "Write" {
            p.opt_str(&["tool_input", "content"])
        } else {
            None
        };
        let args = DocValidateArgs {
            paths: vec![path.to_path_buf()],
            all: false,
            allocate: None,
            producer_check: true,
            stdin_content: content.is_some(),
        };
        if let Err(e) = cmd::doc::validate(ctx, &args, content) {
            if let Some(d) = e.diag() {
                warnings.push(d.clone());
            }
            blocked = true;
        }
    }

    // The declared-set arm: a path outside `.devforgeai/` during a Build run
    // is tested against a story's `## Files` table.
    //
    // Which story is the path's own, not the session's: with two worktrees
    // open, a write into `wt/STORY-015/` belongs to STORY-015 however far
    // `[active].build` has moved. Falling back to the active story is what
    // covers the ordinary single-checkout run, where there is no worktree to
    // resolve against.
    if !inside && ctx.state()?.current.phase == "build" {
        actions.push("story files --check");
        let owner = ctx.worktree_story_of(path);
        let args = crate::cli::StoryFilesArgs {
            check: Some(path.to_path_buf()),
            list: false,
            diff: false,
            id: owner,
            base: None,
        };
        match cmd::story::files(ctx, &args) {
            Ok(o) => {
                if o.exit.unwrap_or(0) != 0 {
                    warnings.extend(o.warnings);
                    blocked = true;
                }
            }
            Err(e) => {
                if let Some(d) = e.diag() {
                    warnings.push(d.clone());
                }
                blocked = true;
            }
        }
    }

    // The token arm: a path matching `[frontend].globs` minus `.exclude` is
    // linted against `brand/tokens.json`.
    if !inside && matches_frontend(ctx, &rel)? {
        actions.push("design lint");
        match cmd::design::lint(ctx, &[path.to_path_buf()], false) {
            Ok(o) => {
                if o.exit.unwrap_or(0) != 0 {
                    warnings.extend(o.warnings);
                    blocked = true;
                }
            }
            Err(e) => {
                if let Some(d) = e.diag() {
                    warnings.push(d.clone());
                }
                blocked = true;
            }
        }
    }

    let exit = if blocked { 2 } else { 0 };
    let mut out = outcome("pre-tool-use", &actions, exit, blocked);
    if blocked {
        let reason = warnings
            .iter()
            .map(|d| format!("{} {}", d.code, d.message))
            .collect::<Vec<_>>()
            .join("\n");
        out.hook_json = Some(deny(&reason));
        out.stderr = vec![reason];
    }
    out.warnings = warnings;
    out.project = ctx.root.display().to_string();
    Ok(out)
}

/// What a write to a path outside the project root gets.
///
/// Refusing every such path would refuse the session scratchpad, which Claude
/// Code writes to legitimately, so the refusal is narrowed to the two places
/// where a write outside the project is a defect rather than ordinary work:
///
/// 1. The trust store and the pinned framework tree. A write there is how an
///    unpinned binary would be made to look pinned, or how the framework the
///    pin covers would be altered under a running session. Always refused.
/// 2. A Build run whose active story declares a file set. The declared set is
///    the story's own scope, and a path outside the project is outside it by
///    construction, so the arm that owns that rule owns this path too.
///
/// Anything else is allowed with no output: no arm of this hook has a rule that
/// could speak to it. The token arm needs no case here, because a glob rooted
/// at the project cannot match a path outside it.
fn outside_root(ctx: &mut Ctx, path: &Path) -> Outcome {
    let key = hook_path_key(&path.to_string_lossy());

    let mut protected: Vec<String> = vec![hook_path_key(&trust::trust_home().to_string_lossy())];
    if let Ok(Some(t)) = trust::load_trust_file() {
        for pin in &t.pin {
            if !pin.framework_path.is_empty() {
                protected.push(hook_path_key(&pin.framework_path));
            }
        }
    }
    for p in &protected {
        // The key carries a trailing slash, so this is a directory-prefix test
        // and not a partial-name match.
        if key.starts_with(p.as_str()) {
            let reason = format!(
                "devforgeai: writes to the trust store or framework tree are refused: {}",
                path.display()
            );
            let mut out = outcome("pre-tool-use", &["path resolve"], 2, true);
            out.hook_json = Some(deny(&reason));
            out.stderr = vec![reason];
            out.project = ctx.root.display().to_string();
            return out;
        }
    }

    let building = ctx
        .state()
        .map(|s| s.current.phase == "build" && !s.active.build.is_empty())
        .unwrap_or(false);
    if building {
        let reason = format!(
            "devforgeai: '{}' does not resolve under the project root {}, so it is outside the active story's declared file set",
            path.display(),
            ctx.root.display()
        );
        let mut out = outcome("pre-tool-use", &["story files --check"], 2, true);
        out.hook_json = Some(deny(&reason));
        out.stderr = vec![reason];
        out.project = ctx.root.display().to_string();
        return out;
    }

    let mut out = outcome("pre-tool-use", &[], 0, false);
    out.project = ctx.root.display().to_string();
    out
}

/// The shell arm: a `PostToolUse` hook matching `Write|Edit` never fires for a
/// redirection or a heredoc, so the producer gate is one `cat > …` away from
/// being bypassed. Recognised write forms whose target lands under
/// `.devforgeai/` take the producer check the write tools take; anything the
/// scan does not recognise passes untouched, so ordinary shell use is
/// unaffected.
/// The two read-only analysis agents whose whole shell budget is one command
/// from `config.toml`.
const METRICS_AGENTS: &[(&str, &[(&str, &str)])] = &[
    (
        "code-quality-auditor",
        &[("[verify].metrics_command", "metrics")],
    ),
    (
        "dead-code-detector",
        &[("[verify].call_graph_command", "call_graph")],
    ),
];

fn pre_tool_use_shell(ctx: &mut Ctx, p: &Payload) -> Result<Outcome, CliError> {
    let Some(command) = p.opt_str(&["tool_input", "command"]) else {
        return Ok(outcome("pre-tool-use", &[], 0, false));
    };

    // `trust pin` refuses to run inside a session by reading `CLAUDECODE`, and
    // a child process that strips its own environment walks straight past that
    // guard — which is how a test spawning the binary pinned a developer's real
    // trust store from inside a session. The environment check cannot be made
    // reliable, because the thing it inspects is the thing the caller controls.
    // Refusing the call at the permission layer does not depend on the child's
    // environment at all.
    if mentions_trust_pin(&command) {
        let reason =
            "devforgeai: trust pin runs only in a terminal outside Claude Code".to_string();
        let mut out = outcome("pre-tool-use", &["trust pin guard"], 2, true);
        out.hook_json = Some(deny(&reason));
        out.stderr = vec![reason];
        out.project = ctx.root.display().to_string();
        return Ok(out);
    }

    // The two analysis agents run one configured command and nothing else.
    // Naming the permitted command in `config.toml` rather than in the agent's
    // tool grant means the project decides what they may run, and an agent that
    // improvises a different shell command is refused with the key to edit.
    if let Some(out) = metrics_agent_decision(ctx, p, &command) {
        return Ok(out);
    }

    let targets = shell_write_targets(&command);
    let mut actions: Vec<&str> = Vec::new();
    let mut warnings: Vec<Diag> = Vec::new();
    let mut blocked = false;

    for target in &targets {
        let abs = if Path::new(target).is_absolute() {
            std::path::PathBuf::from(target)
        } else {
            ctx.root.join(target)
        };
        let rel = rel_under(&ctx.root, &abs).unwrap_or_else(|| target.clone());
        if !under_dot(&rel) {
            continue;
        }
        actions.push("doc validate --producer-check");
        let args = DocValidateArgs {
            paths: vec![ctx.root.join(&rel)],
            all: false,
            allocate: None,
            producer_check: true,
            stdin_content: false,
        };
        if let Err(e) = cmd::doc::validate(ctx, &args, None) {
            if let Some(d) = e.diag() {
                warnings.push(d.clone());
            }
            blocked = true;
        }
    }

    let exit = if blocked { 2 } else { 0 };
    let mut out = outcome("pre-tool-use", &actions, exit, blocked);
    if blocked {
        let reason = format!(
            "devforgeai: this command writes under .devforgeai/ from a phase that does not own the document.\n{}",
            warnings
                .iter()
                .map(|d| format!("{} {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("\n")
        );
        out.hook_json = Some(deny(&reason));
        out.stderr = vec![reason];
    }
    out.warnings = warnings;
    out.project = ctx.root.display().to_string();
    Ok(out)
}

/// True when a shell command names `trust pin`.
///
/// Whitespace is normalised so `trust   pin` and a line-broken invocation read
/// the same, and the comparison is case-insensitive because the shells this
/// runs under are. The test is deliberately broad: it costs a false refusal on
/// a command that merely mentions the phrase, which a human can reword, and it
/// buys a guard that no child process can strip.
pub fn mentions_trust_pin(command: &str) -> bool {
    let normalised = command
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    normalised.contains("trust pin")
}

/// The decision for a shell call made by one of the two analysis agents, or
/// `None` when the payload names any other agent.
///
/// `permissionDecision: "allow"` is returned rather than exit 0 with no output,
/// because these agents carry no other shell grant: the allow is what lets the
/// one configured command through. A deny names the `config.toml` key to edit,
/// so the refusal is actionable by the person who owns the project's tooling
/// rather than only by whoever wrote the agent.
fn metrics_agent_decision(ctx: &mut Ctx, p: &Payload, command: &str) -> Option<Outcome> {
    let agent = p.opt_str(&["agent_type"])?;
    let (_, keys) = METRICS_AGENTS.iter().find(|(name, _)| *name == agent)?;

    let cfg = ctx.config().ok()?.verify.clone();
    let permitted: Vec<(&str, String)> = keys
        .iter()
        .map(|(key, field)| {
            let value = match *field {
                "metrics" => cfg.metrics_command.clone(),
                _ => cfg.call_graph_command.clone(),
            };
            (*key, value)
        })
        .collect();

    let want = command.trim();
    let allowed = permitted
        .iter()
        .any(|(_, value)| !value.trim().is_empty() && value.trim() == want);

    let mut out = if allowed {
        let mut out = outcome("pre-tool-use", &["metrics command"], 0, false);
        out.hook_json = Some(json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "allow",
                "permissionDecisionReason": cap(&format!(
                    "devforgeai: '{want}' is the configured {} for {agent}",
                    permitted[0].0
                )),
            }
        }));
        out
    } else {
        let names: Vec<&str> = permitted.iter().map(|(k, _)| *k).collect();
        let reason = format!(
            "devforgeai: {agent} may run only the command config.toml records at {}. \
             It asked to run '{want}'. Set that key, or have the agent run what it names.",
            names.join(" or ")
        );
        let mut out = outcome("pre-tool-use", &["metrics command"], 2, true);
        out.hook_json = Some(deny(&reason));
        out.stderr = vec![reason];
        out
    };
    out.project = ctx.root.display().to_string();
    Some(out)
}

/// Every path a shell command appears to write, from a redirection, a heredoc
/// redirection, or one of the recognised copying and writing commands.
pub fn shell_write_targets(command: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut push = |s: &str| {
        let t = s.trim().trim_matches(['"', '\'']).to_string();
        if !t.is_empty() && !out.contains(&t) {
            out.push(t);
        }
    };

    // Redirections: the token after `>` or `>>`, which is also where a heredoc
    // (`<<'EOF' > path`) lands its bytes.
    let bytes: Vec<char> = command.chars().collect();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == '>' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j] == '>' {
                j += 1;
            }
            while j < bytes.len() && bytes[j] == ' ' {
                j += 1;
            }
            let start = j;
            while j < bytes.len() && !bytes[j].is_whitespace() && bytes[j] != ';' && bytes[j] != '|'
            {
                j += 1;
            }
            if j > start {
                push(&bytes[start..j].iter().collect::<String>());
            }
            i = j;
            continue;
        }
        i += 1;
    }

    // The recognised commands: every following token that is not a flag.
    let tokens: Vec<&str> = command.split_whitespace().collect();
    for (n, tok) in tokens.iter().enumerate() {
        let bare = tok.trim_start_matches(['(', '&', ';', '|']);
        if !SHELL_WRITES.iter().any(|w| w.eq_ignore_ascii_case(bare)) {
            continue;
        }
        for next in tokens.iter().skip(n + 1) {
            if next.starts_with('-') {
                continue;
            }
            if next.starts_with('|') || next.starts_with('&') || next.starts_with(';') {
                break;
            }
            push(next);
        }
    }

    out.retain(|t| under_dot(t));
    out
}

/// True when a project-relative path matches `[frontend].globs` and no
/// `[frontend].exclude` pattern.
fn matches_frontend(ctx: &mut Ctx, rel: &str) -> Result<bool, CliError> {
    let frontend = ctx.config()?.frontend.clone();
    let set = |patterns: &[String]| {
        let mut b = globset::GlobSetBuilder::new();
        for p in patterns {
            if let Ok(g) = globset::Glob::new(p) {
                b.add(g);
            }
        }
        b.build().unwrap_or_else(|_| globset::GlobSet::empty())
    };
    Ok(set(&frontend.globs).is_match(rel) && !set(&frontend.exclude).is_match(rel))
}

// ------------------------------------------------------------ post tool use

fn post_tool_use(ctx: &mut Ctx, p: &Payload) -> Result<Outcome, CliError> {
    let tool = p.str_key("tool_name", "post-tool-use")?;
    let mut actions: Vec<&str> = Vec::new();
    let mut warnings: Vec<Diag> = Vec::new();
    let mut human: Vec<String> = Vec::new();

    if tool == "Write" || tool == "Edit" || tool == "NotebookEdit" {
        if let Some(file_path) = p
            .opt_str(&["tool_input", "file_path"])
            .or_else(|| p.opt_str(&["tool_input", "notebook_path"]))
        {
            let path = Path::new(&file_path);
            if let Some(rel) = rel_under(&ctx.root, path) {
                if under_dot(&rel) {
                    actions.push("doc validate");
                    let args = DocValidateArgs {
                        paths: vec![path.to_path_buf()],
                        all: false,
                        allocate: None,
                        producer_check: false,
                        stdin_content: false,
                    };
                    match cmd::doc::validate(ctx, &args, None) {
                        // Diagnostics annotate; the write stands.
                        Ok(o) => warnings.extend(o.warnings),
                        Err(e) => {
                            if let Some(d) = e.diag() {
                                warnings.push(d.clone());
                            }
                        }
                    }
                }
            }
        }
    } else if tool == "Bash" || tool == "PowerShell" {
        if let Some(command) = p.opt_str(&["tool_input", "command"]) {
            let matches_test = ctx
                .config()
                .map(|c| {
                    c.stack.iter().any(|s| {
                        !s.test_command.is_empty() && s.test_command.trim() == command.trim()
                    })
                })
                .unwrap_or(false);
            if matches_test {
                actions.push("gate check --phase build --partial");
                let args = GateCheckArgs {
                    phase: "build".to_string(),
                    id: None,
                    partial: true,
                    no_run: false,
                };
                match cmd::gate::check(ctx, &args) {
                    Ok(o) => human.extend(o.human),
                    Err(e) => {
                        if let Some(d) = e.diag() {
                            warnings.push(d.clone());
                        }
                    }
                }
            }
        }
    }

    // This event honours no exit code, and its stdout and stderr both reach
    // the debug log alone. `additionalContext` is the only channel that puts
    // the diagnostic in front of Claude.
    let mut context: Vec<String> = warnings
        .iter()
        .map(|d| {
            format!(
                "devforgeai {}: {} {}",
                actions.join(", "),
                d.code,
                d.message
            )
        })
        .collect();
    context.extend(human.iter().cloned());

    let mut out = outcome("post-tool-use", &actions, 0, false);
    if !context.is_empty() {
        // An empty object is still a parsed object and costs a notice on every
        // no-op call, so nothing to say emits nothing.
        out.hook_json = Some(json!({
            "hookSpecificOutput": {
                "hookEventName": "PostToolUse",
                "additionalContext": cap(&context.join("\n")),
            }
        }));
    }
    out.human = human;
    out.warnings = warnings;
    out.project = ctx.root.display().to_string();
    Ok(out)
}

// ------------------------------------------------------- the Stop-time scan

/// The `.devforgeai/` subdirectories the scan reads.
///
/// `reports/` is absent because the binary writes it — `gate check` writes a
/// report during the very Stop that would then flag it. `state.toml`,
/// `config.toml` and `gates.toml` are absent for the same reason, and
/// `.allocated/` holds reservation markers that are not documents. What is
/// left is exactly the set a skill authors and a producer owns.
const SCANNED_DIRS: &[&str] = &[
    "explore", "context", "adr", "stories", "ui-specs", "brand", "releases",
];

/// The extensions a document carries.
const SCANNED_EXTENSIONS: &[&str] = &["md", "yaml", "json"];

/// Every document written since `since`, project-relative.
fn documents_written_since(root: &Path, since: std::time::SystemTime) -> Vec<String> {
    let dot = crate::project::dot(root);
    let mut found: Vec<String> = Vec::new();
    for sub in SCANNED_DIRS {
        let dir = dot.join(sub);
        if !dir.is_dir() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&dir).follow_links(false) {
            let Ok(entry) = entry else { continue };
            if !entry.file_type().is_file() {
                continue;
            }
            let ext = entry
                .path()
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if !SCANNED_EXTENSIONS.contains(&ext.as_str()) {
                continue;
            }
            let Ok(modified) = std::fs::metadata(entry.path()).and_then(|m| m.modified()) else {
                continue;
            };
            if modified <= since {
                continue;
            }
            if let Some(rel) = rel_under(root, entry.path()) {
                found.push(rel);
            }
        }
    }
    found.sort();
    found
}

/// The producer refusals among the documents written since the last scan.
///
/// This is the enforcing half of the shell-write guard. A `PostToolUse` hook
/// matching `Write|Edit` never fires for a heredoc or a redirection, and the
/// `PreToolUse` shell arm can only recognise the write forms it was taught, so
/// a construct nobody anticipated still lands. Reading the tree at the turn's
/// end needs no command parsing and cannot be evaded by spelling: whatever
/// wrote the file, the file is there and its producer is checked.
fn scan_for_producer_violations(ctx: &mut Ctx, since: std::time::SystemTime) -> Vec<Diag> {
    let root = ctx.root.clone();
    let mut refusals: Vec<Diag> = Vec::new();
    for rel in documents_written_since(&root, since) {
        let args = DocValidateArgs {
            paths: vec![root.join(&rel)],
            all: false,
            allocate: None,
            producer_check: true,
            stdin_content: false,
        };
        if let Err(e) = cmd::doc::validate(ctx, &args, None) {
            if let Some(d) = e.diag() {
                refusals.push(d.clone());
            }
        }
    }
    refusals
}

/// Parse an RFC 3339 stamp into the end of the second it names.
///
/// The stamp is written to whole seconds while a file's modification time
/// carries sub-second precision, so a document written at `12:00:00.500` is
/// later than a window recorded as `12:00:00`. Comparing against the start of
/// the second would report that file again on the next Stop, and again after
/// that, wedging the turn on a write the model has already been told about.
/// The window therefore closes at the end of the recorded second: a write in
/// the same second reads as already seen, which errs toward reporting once.
fn stamp_to_system_time(stamp: &str) -> Option<std::time::SystemTime> {
    let parsed =
        time::OffsetDateTime::parse(stamp, &time::format_description::well_known::Rfc3339).ok()?;
    let seconds = parsed.unix_timestamp();
    if seconds < 0 {
        return None;
    }
    Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(seconds as u64 + 1))
}

// --------------------------------------------------------------------- stop

fn stop(ctx: &mut Ctx, p: &Payload) -> Result<Outcome, CliError> {
    let session_id = p.opt_str(&["session_id"]).unwrap_or_default();
    let (phase, id) = {
        let s = ctx.state()?;
        (s.current.phase.clone(), s.current.id.clone())
    };

    // The Stop-time scan runs before the gate, so a document written from the
    // wrong phase is refused whatever the gate goes on to say about it.
    //
    // The window is from the last scan. With none recorded the previous gate
    // run is the nearest turn boundary state carries; with neither, this Stop
    // establishes the baseline and reads nothing, because every document in a
    // freshly initialised project would otherwise look new.
    let scan_since = {
        let s = ctx.state()?;
        let stamp = if !s.stop_hook.scanned_at.is_empty() {
            s.stop_hook.scanned_at.clone()
        } else {
            s.last_gate.at.clone()
        };
        stamp_to_system_time(&stamp)
    };
    let scan_refusals = match scan_since {
        Some(since) => scan_for_producer_violations(ctx, since),
        None => Vec::new(),
    };
    {
        let now = crate::time::now_rfc3339();
        let s = ctx.state_mut()?;
        s.stop_hook.scanned_at = now;
    }

    // The gate runs on every Stop, the continuation included. Claude has been
    // working on the failing checks since the last block, so re-running is the
    // only way to notice that the work now passes; printing the previous
    // report's FAIL handoff instead would forfeit the re-check and announce a
    // failure that no longer exists.
    let args = GateCheckArgs {
        phase: phase.clone(),
        id: if id.is_empty() {
            None
        } else {
            Some(id.clone())
        },
        partial: false,
        no_run: false,
    };

    let mut warnings: Vec<Diag> = Vec::new();
    let mut gate_lines: Vec<String> = Vec::new();
    let mut failed: Vec<String> = Vec::new();
    let result = match cmd::gate::check(ctx, &args) {
        Ok(o) => {
            let result = o.data["result"].as_str().unwrap_or("FAIL").to_string();
            gate_lines.extend(o.human.iter().cloned());
            if let Some(checks) = o.data["checks"].as_array() {
                for c in checks {
                    if c["status"].as_str() == Some("fail") {
                        let cid = c["id"].as_str().unwrap_or("");
                        let reason = c["reason"].as_str().unwrap_or("");
                        failed.push(if reason.is_empty() {
                            cid.to_string()
                        } else {
                            format!("{cid} {reason}")
                        });
                    }
                }
            }
            warnings.extend(o.warnings);
            result
        }
        Err(e) => {
            if let Some(d) = e.diag() {
                warnings.push(d.clone());
            }
            "FAIL".to_string()
        }
    };

    // A document written from a phase that does not own it fails the turn
    // alongside the gate, and its checks are named the same way, so the model
    // is told which file it wrote from the wrong phase rather than only that
    // something is wrong.
    let result = if scan_refusals.is_empty() {
        result
    } else {
        for d in &scan_refusals {
            // The path is the point of this check: the model has to know which
            // file it wrote from the wrong phase, not only that one exists.
            failed.push(if d.path.is_empty() {
                format!("{} {}", d.code, d.message)
            } else {
                format!("{} {}: {}", d.code, d.path, d.message)
            });
            warnings.push(d.clone());
        }
        "FAIL".to_string()
    };

    // The block budget is per session, and on nothing else.
    //
    // Keying it on the subject as well is what the harness's own loop
    // protection exists to prevent: Claude continues, the turn runs
    // `phase set` and moves to the next story, the pair changes, the budget
    // resets, and the chain of blocks never ends. The subject is exactly the
    // thing a continuation can change, so it cannot be part of the guard.
    // `stop_hook_active` is not consulted here either, for the same reason —
    // a fresh turn that keeps failing is the same unresolved gate.
    //
    // Three blocks per session, then the turn is let go with the FAIL handoff
    // rendered by this hook rather than discarded at the harness's cap of
    // eight. `[current].phase` and `[current].id` are still recorded, as the
    // record of which subject was blocked last.
    let blocked = result == "FAIL" && charge_block_budget(ctx, &session_id, &phase, &id);
    let block_count = {
        let s = ctx.state_mut()?;
        if result == "PASS" {
            s.stop_hook.block_count = 0;
            s.stop_hook.blocked_phase = String::new();
            s.stop_hook.blocked_id = String::new();
        }
        s.stop_hook.block_count
    };
    ctx.store_state()?;

    let (block, handoff_warnings) = system_message(ctx, &phase, &id);
    warnings.extend(handoff_warnings);

    let exit = if blocked { 2 } else { 0 };
    let mut out = Outcome::data(json!({
        "event": "stop",
        "actions": ["document scan", "gate check", "handoff"],
        "blocked": blocked,
        "exit": exit,
        "block_count": block_count,
    }));

    if blocked {
        // Case B: one object carrying both keys. `reason` is addressed to
        // Claude and names the checks; `systemMessage` is addressed to the user
        // and is the handoff block.
        let mut reason = vec![
            format!("Gate      {phase} · {id}"),
            "Result    FAIL".to_string(),
        ];
        if !failed.is_empty() {
            reason.push(format!("Checks    {}", failed.join("; ")));
        }
        reason.push(
            "Fix the failing checks and stop again; the handoff prints when the turn ends."
                .to_string(),
        );
        let reason = reason.join("\n");
        out.hook_json = Some(json!({
            "decision": "block",
            "reason": cap(&reason),
            "systemMessage": cap(&block),
        }));
        // The same text on stderr, so a schema change upstream degrades to the
        // stderr path rather than to silence.
        out.stderr = vec![reason];
    } else if !block.is_empty() {
        // Case A and case D: the block travels alone, with no decision. On a
        // send back the next step is a different command the user types, and
        // holding the model in the turn cannot produce it.
        out.hook_json = Some(json!({ "systemMessage": cap(&block) }));
    }

    out.human = gate_lines;
    out.human.extend(block.lines().map(str::to_string));
    out.warnings = warnings;
    out.exit = Some(exit);
    out.project = ctx.root.display().to_string();
    Ok(out)
}

/// The handoff block for one Stop, joining the cross-cutting block when a
/// Design or Reflect run recorded one in this turn.
///
/// Design and Reflect leave `[current].phase` where they found it, so one Stop
/// renders two blocks: the cross-cutting one first, then the phase's own,
/// joined by a blank line. `[last_cross]` is cleared afterwards, so a second
/// Stop in the same session prints one block.
fn system_message(ctx: &mut Ctx, phase: &str, id: &str) -> (String, Vec<Diag>) {
    let mut warnings: Vec<Diag> = Vec::new();
    let mut blocks: Vec<String> = Vec::new();

    let cross = ctx.state().ok().map(|s| s.last_cross.clone());
    if let Some(cross) = cross {
        if !cross.phase.is_empty() {
            match cmd::handoff::run(ctx, Some(&cross.phase), Some(&cross.id)) {
                Ok(o) => blocks.push(o.human.join("\n")),
                Err(e) => {
                    if let Some(d) = e.diag() {
                        warnings.push(d.clone());
                    }
                }
            }
            if let Ok(s) = ctx.state_mut() {
                s.last_cross = crate::state::LastCross::default();
            }
            if let Err(e) = ctx.store_state() {
                if let Some(d) = e.diag() {
                    warnings.push(d.clone());
                }
            }
        }
    }

    let want_id = if id.is_empty() { None } else { Some(id) };
    match cmd::handoff::run(ctx, Some(phase), want_id) {
        Ok(o) => blocks.push(o.human.join("\n")),
        Err(e) => {
            if let Some(d) = e.diag() {
                warnings.push(d.clone());
            }
        }
    }

    blocks.retain(|b| !b.trim().is_empty());
    (blocks.join("\n\n"), warnings)
}

// ----------------------------------------------------------- subagent stop

fn subagent_stop(ctx: &mut Ctx, p: &Payload) -> Result<Outcome, CliError> {
    let agent = p.str_key("agent_type", "subagent-stop")?;

    // `last_assistant_message` carries the text of the subagent's final
    // response, so the envelope is read without parsing a transcript.
    // `transcript_path` names the parent session's transcript and is never
    // read here: its last assistant message is the orchestrator's text.
    let content = p.opt_text("last_assistant_message").or_else(|| {
        p.opt_str(&["agent_transcript_path"])
            .and_then(last_assistant)
    });

    let Some(content) = content else {
        let mut out = outcome("subagent-stop", &[], 0, false);
        out.warnings.push(Diag::new(
            "DFA-W411",
            format!("subagent '{agent}' carried no output; nothing ingested"),
        ));
        out.project = ctx.root.display().to_string();
        return Ok(out);
    };

    let args = ReportIngestArgs {
        subagent: agent.clone(),
        source: "-".to_string(),
        id: None,
        phase: None,
    };
    match cmd::report::ingest(ctx, &args, Some(content)) {
        Ok(o) => {
            // An unparsable envelope is `DFA-E410`: `report ingest` writes the
            // block with `status: unparsed` and returns Ok, so the failure has
            // to be recognised here. This event blocks, and `reason` reaches
            // the subagent as its next instruction, so the envelope is asked
            // for again rather than logged and lost — the gate would otherwise
            // evaluate `verifier_pass` against a block of zeros.
            if let Some(d) = o.warnings.iter().find(|d| d.code == "DFA-E410") {
                return Ok(ingest_block(
                    ctx,
                    &format!("{} {}", d.code, d.message),
                    d.clone(),
                ));
            }
            let mut out = o;
            out.data = json!({
                "event": "subagent-stop",
                "actions": ["report ingest"],
                "blocked": false,
                "exit": 0,
                "block_count": 0,
                "ingest": out.data,
            });
            out.exit = Some(0);
            out.project = ctx.root.display().to_string();
            Ok(out)
        }
        Err(e) => {
            let d = e.diag().cloned();
            let detail = d
                .as_ref()
                .map(|d| format!("{} {}", d.code, d.message))
                .unwrap_or_else(|| "the envelope did not parse".to_string());
            let mut out = ingest_block_bare(ctx, &detail);
            if let Some(d) = d {
                out.errors.push(d);
            }
            Ok(out)
        }
    }
}

/// The blocking outcome an unusable verifier report produces, carrying the
/// diagnostic that named it.
fn ingest_block(ctx: &mut Ctx, detail: &str, diag: Diag) -> Outcome {
    let mut out = ingest_block_bare(ctx, detail);
    out.errors.push(diag);
    out
}

/// The blocking outcome alone.
fn ingest_block_bare(ctx: &mut Ctx, detail: &str) -> Outcome {
    let reason = format!(
        "devforgeai could not ingest your report: {detail}\n\
         Re-emit your final message as one devforgeai/verifier/1 envelope and nothing else."
    );
    let mut out = outcome("subagent-stop", &["report ingest"], 2, true);
    out.hook_json = Some(block_decision(&reason));
    out.stderr = vec![reason];
    out.project = ctx.root.display().to_string();
    out
}

/// The content of the last assistant message of a JSONL transcript.
fn last_assistant(path: String) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let mut found: Option<String> = None;
    for line in text.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let is_assistant = v.get("role").and_then(Value::as_str) == Some("assistant")
            || v.get("type").and_then(Value::as_str) == Some("assistant");
        if !is_assistant {
            continue;
        }
        // `content` is a string in some records and an array of typed blocks in
        // a Claude Code transcript; both shapes resolve to text.
        if let Some(c) = v.get("content").and_then(text_of) {
            found = Some(c);
        } else if let Some(c) = v
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(text_of)
        {
            found = Some(c);
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_path_key_does_not_match_a_partial_segment() {
        assert!(under_dot(".devforgeai/stories/STORY-001.md"));
        assert!(under_dot("nested/.devforgeai/config.toml"));
        assert!(!under_dot("my.devforgeai.bak/stories/STORY-001.md"));
        assert!(!under_dot("src/devforgeai/main.rs"));
    }

    #[test]
    #[cfg(windows)]
    fn the_dot_directory_test_is_case_folded_on_windows() {
        assert!(under_dot(".DevForgeAI/stories/STORY-001.md"));
        assert!(under_dot(r"nested\.DEVFORGEAI\config.toml"));
    }

    #[test]
    fn shell_write_targets_finds_a_redirection_and_a_copy() {
        let t = shell_write_targets("cat > .devforgeai/stories/STORY-007.md <<'EOF'");
        assert_eq!(t, vec![".devforgeai/stories/STORY-007.md"]);

        let t = shell_write_targets("printf '%s' \"$doc\" >> .devforgeai/reports/x.yaml");
        assert_eq!(t, vec![".devforgeai/reports/x.yaml"]);

        let t = shell_write_targets("cp /tmp/a.md .devforgeai/stories/STORY-009.md");
        assert_eq!(t, vec![".devforgeai/stories/STORY-009.md"]);

        let t = shell_write_targets("Set-Content -Path .devforgeai/state.toml -Value x");
        assert_eq!(t, vec![".devforgeai/state.toml"]);
    }

    #[test]
    fn shell_write_targets_ignores_ordinary_commands() {
        assert!(shell_write_targets("cargo test --all-features").is_empty());
        assert!(shell_write_targets("ls -la").is_empty());
        assert!(shell_write_targets("echo hi > /tmp/out.txt").is_empty());
        assert!(shell_write_targets("grep -r devforgeai src/").is_empty());
    }

    #[test]
    fn text_of_reads_a_string_and_a_block_array() {
        assert_eq!(text_of(&json!("hello")), Some("hello".to_string()));
        assert_eq!(
            text_of(&json!([
                {"type": "thinking", "thinking": "hmm"},
                {"type": "text", "text": "schema: devforgeai/verifier/1"}
            ])),
            Some("schema: devforgeai/verifier/1".to_string())
        );
        assert_eq!(text_of(&json!([])), None);
    }

    #[test]
    fn cap_marks_what_it_dropped() {
        let long = "x".repeat(OUTPUT_CAP + 50);
        let out = cap(&long);
        assert!(out.starts_with(&"x".repeat(100)));
        assert!(out.ends_with("[+50 chars]"));
        assert_eq!(cap("short"), "short");
    }
}
