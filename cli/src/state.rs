//! `.devforgeai/state.toml`: the current phase, the active id per phase, the
//! last gate result, the last handoff, and the Stop counter.

use crate::errors::CliError;
use crate::project;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// The fixed `schema` value.
pub const SCHEMA: &str = "devforgeai/state/1";

/// The seven phase names, in the conventions section 5 order.
pub const PHASES: &[&str] = &[
    "explore",
    "discover",
    "constitute",
    "plan",
    "build",
    "verify",
    "release",
];

/// The phase names a gate may carry: the seven plus `reflect`.
pub const GATE_PHASES: &[&str] = &[
    "explore",
    "discover",
    "constitute",
    "plan",
    "build",
    "verify",
    "release",
    "reflect",
];

/// The index of `phase` in the section 5 order, used to render the handoff
/// `Phase` line and to break a send-back tie toward the earlier phase.
pub fn phase_index(phase: &str) -> Option<usize> {
    PHASES.iter().position(|p| *p == phase)
}

/// The display name of a phase, as the handoff prints it.
pub fn phase_name(phase: &str) -> String {
    match phase {
        "design" => "Design".to_string(),
        "reflect" => "Reflect".to_string(),
        other => {
            let mut c = other.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        }
    }
}

/// `.devforgeai/state.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    /// Fixed: `devforgeai/state/1`.
    pub schema: String,
    /// RFC 3339 UTC.
    pub updated_at: String,
    /// The pair every skill reads.
    #[serde(default)]
    pub current: Current,
    /// One key per phase.
    #[serde(default)]
    pub active: Active,
    /// Explore's phase-scoped fields.
    #[serde(default)]
    pub explore: ExploreState,
    /// Constitute's phase-scoped fields.
    #[serde(default)]
    pub constitute: ConstituteState,
    /// Plan's phase-scoped fields.
    #[serde(default)]
    pub plan: PlanState,
    /// The result of the last `gate check`.
    #[serde(default)]
    pub last_gate: LastGate,
    /// The last rendered handoff block.
    #[serde(default)]
    pub last_handoff: LastHandoff,
    /// The Stop hook counter.
    #[serde(default)]
    pub stop_hook: StopHook,
    /// The cross-cutting run whose handoff the next Stop still owes.
    #[serde(default)]
    pub last_cross: LastCross,
}

/// `[current]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Current {
    /// The phase enum.
    pub phase: String,
    /// The active id of `phase`; mirrors `[active].<phase>`.
    pub id: String,
}

impl Default for Current {
    fn default() -> Self {
        Current {
            phase: "explore".into(),
            id: String::new(),
        }
    }
}

/// `[active]`, one key per phase.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Active {
    /// `IDEA-nnn` or `""`.
    #[serde(default)]
    pub explore: String,
    /// `IDEA-nnn` or `""`.
    #[serde(default)]
    pub discover: String,
    /// `IDEA-nnn` or `""`.
    #[serde(default)]
    pub constitute: String,
    /// `SPRINT-nnn` or `""`.
    #[serde(default)]
    pub plan: String,
    /// `STORY-nnn` or `""`.
    #[serde(default)]
    pub build: String,
    /// `STORY-nnn` or `""`.
    #[serde(default)]
    pub verify: String,
    /// `vX.Y.Z` or `""`.
    #[serde(default)]
    pub release: String,
}

impl Active {
    /// The active id of `phase`, or `None` for a name with no `[active]` key.
    pub fn get(&self, phase: &str) -> Option<&str> {
        Some(match phase {
            "explore" => &self.explore,
            "discover" => &self.discover,
            "constitute" => &self.constitute,
            "plan" => &self.plan,
            "build" => &self.build,
            "verify" => &self.verify,
            "release" => &self.release,
            _ => return None,
        })
    }

    /// Set the active id of `phase`; a name with no key is a no-op.
    pub fn set(&mut self, phase: &str, id: &str) {
        let slot = match phase {
            "explore" => &mut self.explore,
            "discover" => &mut self.discover,
            "constitute" => &mut self.constitute,
            "plan" => &mut self.plan,
            "build" => &mut self.build,
            "verify" => &mut self.verify,
            "release" => &mut self.release,
            _ => return,
        };
        *slot = id.to_string();
    }
}

/// `[explore]`, the phase-scoped table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploreState {
    /// The idea.
    #[serde(default)]
    pub idea_id: String,
    /// RFC 3339 or `""`.
    #[serde(default)]
    pub started_at: String,
    /// From `config.toml` `[explore].timebox_days`.
    #[serde(default = "five")]
    pub timebox_days: i64,
    /// RFC 3339 or `""`.
    #[serde(default)]
    pub remedy_started_at: String,
    /// From `config.toml` `[explore].remedy_timebox_days`.
    #[serde(default = "one")]
    pub remedy_timebox_days: i64,
    /// `FLOW-nnn` ids.
    #[serde(default)]
    pub remedy_flows: Vec<String>,
}

fn five() -> i64 {
    5
}
fn one() -> i64 {
    1
}

impl Default for ExploreState {
    fn default() -> Self {
        ExploreState {
            idea_id: String::new(),
            started_at: String::new(),
            timebox_days: 5,
            remedy_started_at: String::new(),
            remedy_timebox_days: 1,
            remedy_flows: Vec::new(),
        }
    }
}

/// `[constitute]`, the phase-scoped table.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConstituteState {
    /// `CON-nnn` or `""`.
    #[serde(default)]
    pub remedy_con: String,
}

/// `[plan]`, the phase-scoped table.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlanState {
    /// The `EPIC-nnn` the plan run is for, or `""`.
    ///
    /// `[active].plan` holds the sprint, which is the subject the phase
    /// advances, and the sprint document is not written until the end of the
    /// run. The epic is what the gate resolves against in the meantime, so it
    /// is recorded here when `phase set plan` is given one.
    #[serde(default)]
    pub epic: String,
}

/// `[last_gate]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastGate {
    /// The phase enum or `""`.
    #[serde(default)]
    pub phase: String,
    /// The gate subject.
    #[serde(default)]
    pub id: String,
    /// PASS, FAIL, SEND_BACK, TRUST_FAIL, or NOT_RUN.
    #[serde(default = "not_run")]
    pub result: String,
    /// RFC 3339 UTC or `""`.
    #[serde(default)]
    pub at: String,
    /// The report path.
    #[serde(default)]
    pub report: String,
    /// The phase enum or `""`.
    #[serde(default)]
    pub send_back_to: String,
    /// The check ids that failed.
    #[serde(default)]
    pub failed_checks: Vec<String>,
}

fn not_run() -> String {
    "NOT_RUN".to_string()
}

impl Default for LastGate {
    fn default() -> Self {
        LastGate {
            phase: String::new(),
            id: String::new(),
            result: "NOT_RUN".into(),
            at: String::new(),
            report: String::new(),
            send_back_to: String::new(),
            failed_checks: Vec::new(),
        }
    }
}

/// `[last_handoff]`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LastHandoff {
    /// RFC 3339 UTC or `""`.
    #[serde(default)]
    pub rendered_at: String,
    /// The phase enum, `design`, or `""`.
    #[serde(default)]
    pub phase: String,
    /// The subject.
    #[serde(default)]
    pub id: String,
    /// The rendered block, at most twelve entries.
    #[serde(default)]
    pub lines: Vec<String>,
}

/// `[stop_hook]`.
///
/// A record, not a decision. The harness owns the loop guard: `stop_hook_active`
/// is true while Claude Code is already continuing because of a Stop hook, and
/// the harness ends the turn after eight consecutive blocks. This table survives
/// the session, so a counter kept here would tolerate a FAIL for ever after the
/// first block; the dispatcher reads `stop_hook_active` instead and writes these
/// fields for the debug log and the `--json` envelope.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StopHook {
    /// Incremented when the Stop hook blocks, and reset when the session
    /// changes.
    #[serde(default)]
    pub block_count: i64,
    /// The phase enum or `""`.
    #[serde(default)]
    pub blocked_phase: String,
    /// The subject.
    #[serde(default)]
    pub blocked_id: String,
    /// The `session_id` of the payload that last blocked, so the record is
    /// legible per session rather than accumulating across them.
    #[serde(default)]
    pub blocked_session: String,
    /// RFC 3339 UTC of the last Stop-time document scan, or `""`.
    ///
    /// The scan window is the turn: every document under `.devforgeai/` whose
    /// modification time is later than this was written since the last Stop,
    /// whatever wrote it. That is what catches a shell redirection, which no
    /// `Write`-matched hook ever sees.
    #[serde(default)]
    pub scanned_at: String,
}

/// `[last_cross]`.
///
/// Design and Reflect leave `[current].phase` where they found it, so a Stop at
/// the end of a cross-cutting run has two blocks to render. The cross-cutting
/// skill's own `handoff` call site writes this table, and the Stop that renders
/// the pair clears it, so a second Stop in the same session prints one block.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LastCross {
    /// `design`, `reflect`, or `""`.
    #[serde(default)]
    pub phase: String,
    /// The subject of the cross-cutting run.
    #[serde(default)]
    pub id: String,
    /// RFC 3339 UTC of the turn that recorded it, or `""`.
    #[serde(default)]
    pub turn: String,
}

impl Default for State {
    fn default() -> Self {
        State {
            schema: SCHEMA.to_string(),
            updated_at: String::new(),
            current: Current::default(),
            active: Active::default(),
            explore: ExploreState::default(),
            constitute: ConstituteState::default(),
            plan: PlanState::default(),
            last_gate: LastGate::default(),
            last_handoff: LastHandoff::default(),
            stop_hook: StopHook::default(),
            last_cross: LastCross::default(),
        }
    }
}

/// The keys each phase table defines. A key outside its list is `DFA-E108`, so
/// a typo fails rather than being silently unread.
const EXPLORE_KEYS: &[&str] = &[
    "idea_id",
    "started_at",
    "timebox_days",
    "remedy_started_at",
    "remedy_timebox_days",
    "remedy_flows",
];
const CONSTITUTE_KEYS: &[&str] = &["remedy_con"];
const PLAN_KEYS: &[&str] = &["epic"];

/// The four phase names reserved as table names that hold no key in v1.
const RESERVED_PHASE_TABLES: &[&str] = &["discover", "build", "verify", "release"];

/// Read and validate `.devforgeai/state.toml`.
pub fn load(root: &Path) -> Result<State, CliError> {
    let path = project::dot(root).join(project::STATE);
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(CliError::at(
                "DFA-E103",
                ".devforgeai/state.toml not found; run 'devforgeai init'",
                ".devforgeai/state.toml",
            ))
        }
        Err(e) => return Err(CliError::io("reading", path.display(), &e)),
    };
    parse(&text)
}

/// Parse and validate state TOML text.
pub fn parse(text: &str) -> Result<State, CliError> {
    let raw: toml::Value = toml::from_str(text).map_err(|e| {
        CliError::at(
            "DFA-E106",
            format!("state.toml is not valid TOML: {e}"),
            ".devforgeai/state.toml",
        )
    })?;

    check_phase_table(&raw, "explore", EXPLORE_KEYS)?;
    check_phase_table(&raw, "constitute", CONSTITUTE_KEYS)?;
    check_phase_table(&raw, "plan", PLAN_KEYS)?;
    for phase in RESERVED_PHASE_TABLES {
        check_phase_table(&raw, phase, &[])?;
    }

    let state: State = raw.try_into().map_err(|e| {
        CliError::at(
            "DFA-E106",
            format!("state.toml is not valid TOML: {e}"),
            ".devforgeai/state.toml",
        )
    })?;

    if state.schema != SCHEMA {
        return Err(CliError::at(
            "DFA-E107",
            format!(
                "state.toml: schema '{}' is unsupported; this binary reads {SCHEMA}",
                state.schema
            ),
            ".devforgeai/state.toml",
        ));
    }

    // `[current].id` equals `[active].<current.phase>` in every valid file.
    if let Some(active) = state.active.get(&state.current.phase) {
        if active != state.current.id {
            return Err(CliError::at(
                "DFA-E106",
                format!(
                    "state.toml [current].id is '{}' and [active].{} is '{active}'; the two agree in every valid file",
                    state.current.id, state.current.phase
                ),
                ".devforgeai/state.toml",
            ));
        }
    }

    Ok(state)
}

fn check_phase_table(raw: &toml::Value, phase: &str, allowed: &[&str]) -> Result<(), CliError> {
    let Some(table) = raw.get(phase).and_then(toml::Value::as_table) else {
        return Ok(());
    };
    for key in table.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(CliError::at(
                "DFA-E108",
                format!(
                    "state.toml [{phase}] has key '{key}', which the state schema does not define"
                ),
                ".devforgeai/state.toml",
            ));
        }
    }
    Ok(())
}

/// Write `state.toml` atomically, stamping `updated_at`.
pub fn store(root: &Path, state: &mut State) -> Result<(), CliError> {
    state.updated_at = crate::time::now_rfc3339();
    let text = to_toml(state)?;
    project::atomic_write(&project::dot(root).join(project::STATE), text.as_bytes())
}

/// Serialise a state file.
pub fn to_toml(state: &State) -> Result<String, CliError> {
    toml::to_string_pretty(state)
        .map_err(|e| CliError::new("DFA-E900", format!("serialising state.toml failed: {e}")))
}

/// The `templates/state.initial.toml` content, with `@@NOW@@` replaced.
pub fn initial_toml(now: &str) -> String {
    INITIAL_TEMPLATE.replace("@@NOW@@", now)
}

/// `templates/state.initial.toml`, verbatim from the spec's `## Templates`.
pub const INITIAL_TEMPLATE: &str = r#"schema = "devforgeai/state/1"
updated_at = "@@NOW@@"

[current]
phase = "explore"
id = ""

[active]
explore = ""
discover = ""
constitute = ""
plan = ""
build = ""
verify = ""
release = ""

[explore]
idea_id = ""
started_at = ""
timebox_days = 5
remedy_started_at = ""
remedy_timebox_days = 1
remedy_flows = []

[constitute]
remedy_con = ""

[last_gate]
phase = ""
id = ""
result = "NOT_RUN"
at = ""
report = ""
send_back_to = ""
failed_checks = []

[last_handoff]
rendered_at = ""
phase = ""
id = ""
lines = []

[stop_hook]
block_count = 0
blocked_phase = ""
blocked_id = ""
"#;

#[cfg(test)]
mod tests {
    use super::*;

    /// The `state.toml` schema block of the spec's `## Outputs`, verbatim minus
    /// the trailing comments.
    const SPEC_EXAMPLE: &str = r#"
schema = "devforgeai/state/1"
updated_at = "2026-09-10T14:02:11Z"

[current]
phase = "build"
id = "STORY-014"

[active]
explore = "IDEA-003"
discover = "IDEA-003"
constitute = "IDEA-003"
plan = "SPRINT-001"
build = "STORY-014"
verify = "STORY-014"
release = "v0.3.0"

[explore]
idea_id = "IDEA-003"
started_at = "2026-09-06T09:00:00Z"
timebox_days = 5
remedy_started_at = ""
remedy_timebox_days = 1
remedy_flows = []

[constitute]
remedy_con = ""

[last_gate]
phase = "build"
id = "STORY-014"
result = "PASS"
at = "2026-09-10T14:02:11Z"
report = ".devforgeai/reports/STORY-014-build.yaml"
send_back_to = ""
failed_checks = []

[last_handoff]
rendered_at = "2026-09-10T14:02:12Z"
phase = "build"
id = "STORY-014"
lines = []

[stop_hook]
block_count = 0
blocked_phase = ""
blocked_id = ""
"#;

    #[test]
    fn spec_example_round_trips() {
        let s = parse(SPEC_EXAMPLE).expect("the spec example parses");
        assert_eq!(s.schema, SCHEMA);
        assert_eq!(s.current.phase, "build");
        assert_eq!(s.current.id, "STORY-014");
        assert_eq!(s.active.plan, "SPRINT-001");
        assert_eq!(s.active.release, "v0.3.0");
        assert_eq!(s.explore.idea_id, "IDEA-003");
        assert_eq!(s.explore.timebox_days, 5);
        assert_eq!(s.last_gate.result, "PASS");
        assert_eq!(
            s.last_gate.report,
            ".devforgeai/reports/STORY-014-build.yaml"
        );
        assert_eq!(s.last_handoff.phase, "build");
        assert_eq!(s.stop_hook.block_count, 0);

        let text = to_toml(&s).expect("serialise");
        let again = parse(&text).expect("reparse");
        assert_eq!(again.current.id, "STORY-014");
        assert_eq!(again.active.explore, "IDEA-003");
        assert_eq!(again.last_gate.at, "2026-09-10T14:02:11Z");
    }

    #[test]
    fn initial_template_parses_and_matches_the_spec_defaults() {
        let s = parse(&initial_toml("2026-09-10T14:02:11Z")).expect("the initial template parses");
        assert_eq!(s.current.phase, "explore");
        assert_eq!(s.current.id, "");
        for phase in PHASES {
            assert_eq!(s.active.get(phase), Some(""), "{phase} starts empty");
        }
        assert_eq!(s.last_gate.result, "NOT_RUN");
        assert_eq!(s.stop_hook.block_count, 0);
        assert_eq!(s.updated_at, "2026-09-10T14:02:11Z");
    }

    #[test]
    fn unknown_key_in_a_phase_table_gives_e108() {
        let text = SPEC_EXAMPLE.replace("idea_id = \"IDEA-003\"", "idae_id = \"IDEA-003\"");
        let err = parse(&text).expect_err("a typo fails rather than being silently unread");
        assert_eq!(err.code(), "DFA-E108");
        assert_eq!(err.exit(), 1);
        assert!(err.diag().expect("diag").message.contains("idae_id"));
    }

    #[test]
    fn a_reserved_phase_table_holds_no_key_in_v1() {
        let text = format!("{SPEC_EXAMPLE}\n[build]\nanything = 1\n");
        let err = parse(&text).expect_err("[build] holds no key in v1");
        assert_eq!(err.code(), "DFA-E108");
    }

    #[test]
    fn unparsable_state_gives_e106() {
        let err = parse("= = not toml").expect_err("bad TOML");
        assert_eq!(err.code(), "DFA-E106");
    }

    #[test]
    fn current_id_disagreeing_with_active_gives_e106() {
        let text = SPEC_EXAMPLE.replace("build = \"STORY-014\"", "build = \"STORY-099\"");
        let err = parse(&text).expect_err("[current].id mirrors [active].<phase>");
        assert_eq!(err.code(), "DFA-E106");
        assert!(err.diag().expect("diag").message.contains("STORY-099"));
    }

    #[test]
    fn active_get_and_set_cover_the_seven_phases() {
        let mut a = Active::default();
        for phase in PHASES {
            a.set(phase, "X-001");
            assert_eq!(a.get(phase), Some("X-001"));
        }
        assert_eq!(a.get("design"), None, "design has no [active] key");
        assert_eq!(a.get("reflect"), None, "reflect has no [active] key");
    }

    #[test]
    fn phase_index_is_the_section_five_order() {
        assert_eq!(phase_index("explore"), Some(0));
        assert_eq!(phase_index("build"), Some(4));
        assert_eq!(phase_index("release"), Some(6));
        assert_eq!(phase_index("design"), None);
    }
}
