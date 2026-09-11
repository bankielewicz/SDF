//! `.devforgeai/reports/<ID>-<phase>.yaml`: the gate report the CLI owns, the
//! verifier blocks `report ingest` writes into it, and the merged findings the
//! handoff `Found` lines read.

use crate::errors::CliError;
use crate::project;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// The fixed `schema` value.
pub const SCHEMA: &str = "devforgeai/report/1";

/// The `produced_by` of every CLI-written report.
pub const PRODUCER: &str = "devforgeai-cli";

/// The `reason` a check carries when its kind is outside this milestone.
///
/// The schema's check-status enum is `pass | fail | skip | warn`, so such a
/// check is recorded `skip`, and a skip counts as passing. That makes the
/// reason the only mark distinguishing "never evaluated" from "evaluated and
/// deliberately not run", which is why `gate require` reads it: a report whose
/// PASS rests on one of these is not a predecessor any phase may start on.
pub const NOT_IMPLEMENTED_REASON: &str = "not_implemented";

/// The verifier stdout contract `report ingest` parses.
pub const VERIFIER_SCHEMA: &str = "devforgeai/verifier/1";

/// `.devforgeai/reports/<id>-<phase>.yaml`.
pub fn report_path(root: &Path, id: &str, phase: &str) -> PathBuf {
    project::dot(root)
        .join("reports")
        .join(format!("{id}-{phase}.yaml"))
}

/// One `gate.checks[]` entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckEntry {
    /// The check id from `gates.toml`.
    pub id: String,
    /// The check kind.
    pub kind: String,
    /// pass, fail, skip, or warn.
    pub status: String,
    /// block or warn.
    pub severity: String,
    /// `""` on pass; the DFA code and text on fail; the reason word on skip.
    #[serde(default)]
    pub reason: String,
    /// Kind-specific evidence.
    #[serde(default)]
    pub evidence: serde_yaml_ng::Value,
}

/// The `gate` block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateBlock {
    /// PASS, FAIL, or SEND_BACK.
    pub result: String,
    /// The phase enum or `""`.
    #[serde(default)]
    pub send_back_to: String,
    /// One entry per `gate.check`, in `gates.toml` order.
    #[serde(default)]
    pub checks: Vec<CheckEntry>,
}

impl Default for GateBlock {
    fn default() -> Self {
        GateBlock {
            result: "NOT_RUN".into(),
            send_back_to: String::new(),
            checks: Vec::new(),
        }
    }
}

/// One entry of the merged `findings` list.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Finding {
    /// The cited ID.
    pub id: String,
    /// block, warn, or info.
    pub severity: String,
    /// One line, cut to ninety characters for a CLI-generated finding.
    pub summary: String,
    /// The verifier's evidence, when it supplied any.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub evidence: String,
    /// The `spec-gap` marker the verify gate reads.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
}

/// One ingested verifier block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifierBlock {
    /// The subagent name.
    pub subagent: String,
    /// RFC 3339 UTC.
    pub ingested_at: String,
    /// The count that passed.
    pub passed: i64,
    /// The count attempted.
    pub total: i64,
    /// The unit word of the handoff `Verified` line.
    pub unit: String,
    /// The findings this verifier emitted.
    #[serde(default)]
    pub findings: Vec<Finding>,
    /// The agent's own fields, verbatim.
    ///
    /// A registered verifier prints one object: the envelope keys above, and
    /// everything the agent reports of its own under `payload`. The CLI models
    /// none of it, so it is carried as an opaque value and a `report_metric`
    /// check reads into it by path. An unknown key round-trips untouched.
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub payload: serde_json::Value,
    /// `unparsed` when the subagent's stdout did not parse.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// The whole report. Payload the CLI does not model is kept in `extra` so a
/// rewrite preserves a skill's note and a verifier's own keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    /// Fixed: `devforgeai/report/1`.
    pub schema: String,
    /// The gate subject ID.
    pub id: String,
    /// The phase enum.
    pub phase: String,
    /// pass, fail, send_back, or skip.
    pub status: String,
    /// Fixed: `devforgeai-cli`.
    pub produced_by: String,
    /// The IDs the gate read.
    #[serde(default)]
    pub consumes: Vec<String>,
    /// `[]` in every CLI-written report.
    #[serde(default)]
    pub open_questions: Vec<String>,
    /// RFC 3339 UTC.
    #[serde(default)]
    pub started_at: String,
    /// RFC 3339 UTC.
    #[serde(default)]
    pub finished_at: String,
    /// The semver of the binary that wrote the file.
    #[serde(default)]
    pub cli_version: String,
    /// Copied from `config.toml`.
    #[serde(default)]
    pub degraded: bool,
    /// True when `--partial` wrote this report.
    #[serde(default, skip_serializing_if = "is_false")]
    pub partial: bool,
    /// The gate block.
    #[serde(default)]
    pub gate: GateBlock,
    /// Present when a `coverage_min` check ran.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coverage: Option<serde_yaml_ng::Value>,
    /// Written by `report ingest`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verifiers: Option<serde_yaml_ng::Mapping>,
    /// The merged findings the handoff `Found` lines read.
    #[serde(default)]
    pub findings: Vec<Finding>,
    /// The block rendered for this gate, at most twelve entries.
    #[serde(default)]
    pub handoff: Vec<String>,
    /// Every other top-level key, preserved across a rewrite.
    #[serde(flatten)]
    pub extra: serde_yaml_ng::Mapping,
}

fn is_false(b: &bool) -> bool {
    !*b
}

impl Report {
    /// The `## Outputs` skeleton for a subject and a phase.
    pub fn skeleton(id: &str, phase: &str) -> Self {
        let now = crate::time::now_rfc3339();
        Report {
            schema: SCHEMA.to_string(),
            id: id.to_string(),
            phase: phase.to_string(),
            status: "skip".to_string(),
            produced_by: PRODUCER.to_string(),
            consumes: Vec::new(),
            open_questions: Vec::new(),
            started_at: now.clone(),
            finished_at: now,
            cli_version: crate::VERSION.to_string(),
            degraded: false,
            partial: false,
            gate: GateBlock::default(),
            coverage: None,
            verifiers: None,
            findings: Vec::new(),
            handoff: Vec::new(),
            extra: serde_yaml_ng::Mapping::new(),
        }
    }

    /// The verifier block at a dotted `report_field`, as in
    /// `verifiers.ac_compliance`.
    pub fn verifier_block(&self, report_field: &str) -> Option<&serde_yaml_ng::Value> {
        let leaf = report_field.strip_prefix("verifiers.")?;
        self.verifiers
            .as_ref()?
            .get(serde_yaml_ng::Value::String(leaf.to_string()))
    }

    /// Write a verifier block at a dotted `report_field`, replacing any
    /// previous block from the same subagent.
    pub fn set_verifier_block(&mut self, report_field: &str, block: &VerifierBlock) {
        let leaf = report_field
            .strip_prefix("verifiers.")
            .unwrap_or(report_field)
            .to_string();
        let map = self
            .verifiers
            .get_or_insert_with(serde_yaml_ng::Mapping::new);
        let value = serde_yaml_ng::to_value(block).unwrap_or(serde_yaml_ng::Value::Null);
        map.insert(serde_yaml_ng::Value::String(leaf), value);
    }

    /// Append findings, de-duplicated by `id`, keeping the newest per ID.
    pub fn merge_findings(&mut self, incoming: &[Finding]) {
        for f in incoming {
            match self.findings.iter_mut().find(|e| e.id == f.id) {
                Some(existing) => *existing = f.clone(),
                None => self.findings.push(f.clone()),
            }
        }
    }

    /// Read a dotted metric path, with an optional `.length` suffix reading a
    /// sequence's length.
    pub fn metric(&self, path: &str) -> Option<f64> {
        let (base, want_len) = match path.strip_suffix(".length") {
            Some(b) => (b, true),
            None => (path, false),
        };
        let root = serde_yaml_ng::to_value(self).ok()?;
        let mut cur = &root;
        for seg in base.split('.') {
            cur = cur.get(serde_yaml_ng::Value::String(seg.to_string()))?;
        }
        if want_len {
            return cur.as_sequence().map(|s| s.len() as f64);
        }
        match cur {
            serde_yaml_ng::Value::Number(n) => n.as_f64(),
            serde_yaml_ng::Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }
}

/// Read a report, mapping an absent file to `DFA-E400` and a parse failure to
/// `DFA-E401`.
pub fn load(path: &Path) -> Result<Report, CliError> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(CliError::at(
                "DFA-E400",
                format!("{} not found", path.display()),
                path.display().to_string(),
            ))
        }
        Err(e) => return Err(CliError::io("reading", path.display(), &e)),
    };
    parse(&text, path)
}

/// Parse report YAML text.
pub fn parse(text: &str, path: &Path) -> Result<Report, CliError> {
    serde_yaml_ng::from_str(text).map_err(|e| {
        CliError::at(
            "DFA-E401",
            format!("{} is not valid YAML: {e}", path.display()),
            path.display().to_string(),
        )
    })
}

/// Write a report atomically.
pub fn write(path: &Path, report: &Report) -> Result<(), CliError> {
    let text = to_yaml(report)?;
    project::atomic_write(path, text.as_bytes())
}

/// Serialise a report.
pub fn to_yaml(report: &Report) -> Result<String, CliError> {
    serde_yaml_ng::to_string(report)
        .map_err(|e| CliError::new("DFA-E900", format!("serialising the report failed: {e}")))
}

/// The one JSON object a registered verifier prints.
#[derive(Debug, Clone, Deserialize)]
pub struct VerifierOutput {
    /// Fixed: `devforgeai/verifier/1`.
    pub schema: String,
    /// The subagent name.
    #[serde(default)]
    pub subagent: String,
    /// The subject; absent falls back to `[active].<phase>`.
    #[serde(default)]
    pub id: String,
    /// The count that passed.
    #[serde(default)]
    pub passed: i64,
    /// The count attempted.
    #[serde(default)]
    pub total: i64,
    /// The unit word.
    #[serde(default)]
    pub unit: String,
    /// The findings, defaulting to `[]`.
    #[serde(default)]
    pub findings: Vec<Finding>,
    /// The agent's own fields, merged in under one key and modelled by
    /// nothing here, so a verifier can report whatever its schema names and a
    /// `report_metric` check can read it by path.
    #[serde(default)]
    pub payload: serde_json::Value,
}

/// Parse a verifier's stdout. Any other shape is `DFA-E410`.
pub fn parse_verifier(stdout: &str, name: &str) -> Result<VerifierOutput, CliError> {
    let v: VerifierOutput = serde_json::from_str(stdout.trim()).map_err(|e| {
        CliError::new(
            "DFA-E410",
            format!("subagent '{name}' output is not {VERIFIER_SCHEMA}: {e}"),
        )
    })?;
    if v.schema != VERIFIER_SCHEMA {
        return Err(CliError::new(
            "DFA-E410",
            format!(
                "subagent '{name}' output is not {VERIFIER_SCHEMA}: schema is '{}'",
                v.schema
            ),
        ));
    }
    for f in &v.findings {
        if !matches!(f.severity.as_str(), "block" | "warn" | "info") {
            return Err(CliError::new(
                "DFA-E410",
                format!(
                    "subagent '{name}' output is not {VERIFIER_SCHEMA}: severity '{}' is outside block, warn, info",
                    f.severity
                ),
            ));
        }
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The report schema block of the spec's `## Outputs`, verbatim minus the
    /// trailing comments.
    const SPEC_EXAMPLE: &str = r#"
schema: devforgeai/report/1
id: STORY-014
phase: build
status: pass
produced_by: devforgeai-cli
consumes: [STORY-014, SPRINT-001]
open_questions: []
started_at: 2026-09-10T14:01:03Z
finished_at: 2026-09-10T14:02:11Z
cli_version: 1.0.0
degraded: false
gate:
  result: PASS
  send_back_to: ""
  checks:
    - id: build-tests
      kind: tests_pass
      status: pass
      severity: block
      reason: ""
      evidence:
        command: cargo test --all-features
        exit_code: 0
        duration_ms: 41203
        passed: 214
        failed: 0
verifiers:
  ac_compliance:
    subagent: ac-compliance-verifier
    ingested_at: 2026-09-10T14:00:02Z
    passed: 7
    total: 7
    unit: ACs
    findings: []
findings: []
handoff: []
"#;

    #[test]
    fn spec_example_round_trips() {
        let r = parse(SPEC_EXAMPLE, Path::new("x.yaml")).expect("the spec example parses");
        assert_eq!(r.schema, SCHEMA);
        assert_eq!(r.id, "STORY-014");
        assert_eq!(r.phase, "build");
        assert_eq!(r.status, "pass");
        assert_eq!(r.produced_by, PRODUCER);
        assert_eq!(r.consumes, vec!["STORY-014", "SPRINT-001"]);
        assert_eq!(r.gate.result, "PASS");
        assert_eq!(r.gate.checks.len(), 1);
        assert_eq!(r.gate.checks[0].kind, "tests_pass");
        assert_eq!(
            r.gate.checks[0]
                .evidence
                .get("passed")
                .and_then(|v| v.as_i64()),
            Some(214)
        );

        let block = r
            .verifier_block("verifiers.ac_compliance")
            .expect("the ingested block");
        assert_eq!(
            block.get("subagent").and_then(|v| v.as_str()),
            Some("ac-compliance-verifier")
        );

        let text = to_yaml(&r).expect("serialise");
        let again = parse(&text, Path::new("x.yaml")).expect("reparse");
        assert_eq!(again.id, r.id);
        assert_eq!(again.gate.checks[0].id, "build-tests");
        assert!(
            again.verifier_block("verifiers.ac_compliance").is_some(),
            "the verifiers block survives a rewrite"
        );
    }

    #[test]
    fn unmodelled_top_level_keys_survive_a_rewrite() {
        // `report note` lands a skill's note under its own top-level key.
        let text = format!("{SPEC_EXAMPLE}\nbuild:\n  approach: outside-in\n  files: 3\n");
        let r = parse(&text, Path::new("x.yaml")).expect("parse");
        let again = parse(&to_yaml(&r).expect("serialise"), Path::new("x.yaml")).expect("reparse");
        let note = again
            .extra
            .get(serde_yaml_ng::Value::String("build".into()))
            .expect("the note key survives");
        assert_eq!(
            note.get("approach").and_then(|v| v.as_str()),
            Some("outside-in")
        );
    }

    #[test]
    fn metric_reads_a_dotted_path_and_a_length_suffix() {
        // A registered verifier prints one object: the envelope keys at the
        // top of the block, the agent's own fields merged in under `payload`.
        // A metric path into an agent field therefore carries that segment,
        // while `verifier_pass` keeps reading `passed` and `total` at the top.
        let text = r#"
schema: devforgeai/report/1
id: IDEA-003
phase: constitute
status: pass
produced_by: devforgeai-cli
verifiers:
  architecture_reviewer:
    subagent: architecture-reviewer
    ingested_at: ""
    passed: 3
    total: 3
    unit: findings
    payload:
      blocking_findings: 0
      send_back_requirements: [REQ-004, REQ-009]
"#;
        let r = parse(text, Path::new("x.yaml")).expect("parse");
        assert_eq!(
            r.metric("verifiers.architecture_reviewer.payload.blocking_findings"),
            Some(0.0)
        );
        assert_eq!(
            r.metric("verifiers.architecture_reviewer.payload.send_back_requirements.length"),
            Some(2.0)
        );
        assert_eq!(
            r.metric("verifiers.architecture_reviewer.total"),
            Some(3.0),
            "the envelope counts stay at the top of the block"
        );
        assert_eq!(
            r.metric("verifiers.nothing.here"),
            None,
            "an absent metric is None"
        );
    }

    #[test]
    fn findings_merge_by_id_keeping_the_newest() {
        let mut r = Report::skeleton("STORY-014", "verify");
        r.merge_findings(&[Finding {
            id: "FIND-001".into(),
            severity: "block".into(),
            summary: "first".into(),
            evidence: String::new(),
            category: None,
        }]);
        r.merge_findings(&[
            Finding {
                id: "FIND-001".into(),
                severity: "warn".into(),
                summary: "second".into(),
                evidence: String::new(),
                category: None,
            },
            Finding {
                id: "FIND-002".into(),
                severity: "block".into(),
                summary: "other".into(),
                evidence: String::new(),
                category: None,
            },
        ]);
        assert_eq!(r.findings.len(), 2);
        assert_eq!(
            r.findings[0].summary, "second",
            "the newest entry per ID wins"
        );
        assert_eq!(r.findings[1].id, "FIND-002");
    }

    #[test]
    fn verifier_output_parses_the_subagent_contract() {
        let json = r#"{
  "schema": "devforgeai/verifier/1",
  "subagent": "ac-compliance-verifier",
  "id": "STORY-014",
  "passed": 7,
  "total": 7,
  "unit": "ACs",
  "findings": [
    { "id": "FIND-001", "severity": "block", "summary": "AC-003 has no test", "evidence": "tests/: no case names AC-003" }
  ]
}"#;
        let v = parse_verifier(json, "ac-compliance-verifier").expect("the contract parses");
        assert_eq!(v.id, "STORY-014");
        assert_eq!(v.passed, 7);
        assert_eq!(v.total, 7);
        assert_eq!(v.unit, "ACs");
        assert_eq!(v.findings.len(), 1);
        assert_eq!(v.findings[0].evidence, "tests/: no case names AC-003");
    }

    #[test]
    fn findings_default_to_empty() {
        let json = r#"{"schema":"devforgeai/verifier/1","subagent":"x","passed":0,"total":0,"unit":"checks"}"#;
        let v = parse_verifier(json, "x").expect("parse");
        assert!(v.findings.is_empty());
        assert_eq!(v.id, "", "an absent id falls back to the active id later");
    }

    #[test]
    fn any_other_shape_gives_e410() {
        let err = parse_verifier("not json at all", "x").expect_err("not JSON");
        assert_eq!(err.code(), "DFA-E410");
        assert_eq!(err.exit(), 0, "SubagentStop does not block");

        let err = parse_verifier(r#"{"schema":"other/1"}"#, "x").expect_err("wrong schema");
        assert_eq!(err.code(), "DFA-E410");

        let bad = r#"{"schema":"devforgeai/verifier/1","subagent":"x","passed":1,"total":1,"unit":"u","findings":[{"id":"FIND-001","severity":"fatal","summary":"s"}]}"#;
        let err = parse_verifier(bad, "x").expect_err("severity outside the closed enum");
        assert_eq!(err.code(), "DFA-E410");
    }

    #[test]
    fn missing_report_gives_e400_and_unparsable_gives_e401() {
        let t = tempfile::tempdir().expect("temp");
        let p = t.path().join("absent.yaml");
        assert_eq!(load(&p).expect_err("absent").code(), "DFA-E400");

        let q = t.path().join("bad.yaml");
        std::fs::write(&q, "schema: [unterminated\n").expect("write");
        assert_eq!(load(&q).expect_err("unparsable").code(), "DFA-E401");
    }

    #[test]
    fn skeleton_carries_the_fixed_keys() {
        let r = Report::skeleton("STORY-014", "build");
        assert_eq!(r.schema, SCHEMA);
        assert_eq!(r.produced_by, PRODUCER);
        assert!(r.open_questions.is_empty());
        assert_eq!(r.cli_version, crate::VERSION);
    }

    #[test]
    fn report_path_is_id_dash_phase() {
        let p = report_path(Path::new("/proj"), "STORY-014", "build");
        assert!(
            p.ends_with("reports/STORY-014-build.yaml")
                || p.ends_with(r"reports\STORY-014-build.yaml")
        );
    }
}
