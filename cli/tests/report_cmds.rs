//! `report show`, `report ingest`, `report note`.

mod common;

use common::Project;
use devforgeai::cli::{ReportIngestArgs, ReportNoteArgs};
use devforgeai::cmd::report;

const VERIFIER_JSON: &str = r#"{
  "schema": "devforgeai/verifier/1",
  "subagent": "kill-case-builder",
  "id": "IDEA-003",
  "passed": 3,
  "total": 4,
  "unit": "signals",
  "findings": [
    { "id": "FIND-001", "severity": "block", "summary": "no kill signal named", "evidence": "brief: none" }
  ]
}"#;

fn ingest_args(subagent: &str) -> ReportIngestArgs {
    ReportIngestArgs {
        subagent: subagent.into(),
        source: "-".into(),
        id: None,
        phase: None,
    }
}

fn seeded() -> Project {
    let p = Project::new();
    p.set_phase("explore", "IDEA-003");
    p
}

// ------------------------------------------------------------- report ingest

#[test]
fn ingest_writes_verifier_block_from_stdin() {
    let p = seeded();
    let mut ctx = p.ctx();
    let out = report::ingest(
        &mut ctx,
        &ingest_args("kill-case-builder"),
        Some(VERIFIER_JSON.into()),
    )
    .expect("ingest");

    assert_eq!(out.data["status"], "ingested");
    assert_eq!(out.data["field"], "verifiers.kill_case");
    assert_eq!(out.data["passed"], 3);
    assert_eq!(out.data["total"], 4);
    assert_eq!(
        out.data["report"],
        ".devforgeai/reports/IDEA-003-explore.yaml"
    );

    let text = p.read(".devforgeai/reports/IDEA-003-explore.yaml");
    assert!(text.contains("kill_case:"));
    assert!(text.contains("subagent: kill-case-builder"));
    assert!(text.contains("unit: signals"));
}

#[test]
fn ingest_creates_report_when_absent() {
    let p = seeded();
    assert!(!p.exists(".devforgeai/reports/IDEA-003-explore.yaml"));
    let mut ctx = p.ctx();
    report::ingest(
        &mut ctx,
        &ingest_args("kill-case-builder"),
        Some(VERIFIER_JSON.into()),
    )
    .expect("ingest");

    let text = p.read(".devforgeai/reports/IDEA-003-explore.yaml");
    assert!(text.contains("schema: devforgeai/report/1"));
    assert!(text.contains("produced_by: devforgeai-cli"));
}

#[test]
fn ingest_from_a_file_source() {
    let p = seeded();
    p.write("verifier.json", VERIFIER_JSON);
    let mut ctx = p.ctx();
    let args = ReportIngestArgs {
        subagent: "kill-case-builder".into(),
        source: "verifier.json".into(),
        id: None,
        phase: None,
    };
    let out = report::ingest(&mut ctx, &args, None).expect("ingest from a file");
    assert_eq!(out.data["passed"], 3);
}

#[test]
fn ingest_replaces_previous_block_same_subagent() {
    let p = seeded();
    let mut ctx = p.ctx();
    report::ingest(
        &mut ctx,
        &ingest_args("kill-case-builder"),
        Some(VERIFIER_JSON.into()),
    )
    .expect("first");

    let second = VERIFIER_JSON.replace("\"passed\": 3", "\"passed\": 4");
    let mut ctx = p.ctx();
    report::ingest(&mut ctx, &ingest_args("kill-case-builder"), Some(second)).expect("second");

    let text = p.read(".devforgeai/reports/IDEA-003-explore.yaml");
    assert_eq!(text.matches("subagent: kill-case-builder").count(), 1);
    assert!(text.contains("passed: 4"));
    assert!(!text.contains("passed: 3"));
}

#[test]
fn ingest_merges_findings_by_id() {
    let p = seeded();
    let mut ctx = p.ctx();
    report::ingest(
        &mut ctx,
        &ingest_args("kill-case-builder"),
        Some(VERIFIER_JSON.into()),
    )
    .expect("first");

    let second = VERIFIER_JSON.replace("no kill signal named", "restated with more detail");
    let mut ctx = p.ctx();
    report::ingest(&mut ctx, &ingest_args("kill-case-builder"), Some(second)).expect("second");

    let text = p.read(".devforgeai/reports/IDEA-003-explore.yaml");
    assert_eq!(
        text.matches("id: FIND-001").count(),
        2,
        "once in the verifier block, once in the merged findings"
    );
    assert!(
        text.contains("restated with more detail"),
        "the newest entry wins"
    );
    assert!(!text.contains("no kill signal named"));
}

#[test]
fn ingest_unknown_subagent_gives_w411_exit_zero() {
    let p = seeded();
    let mut ctx = p.ctx();
    let out = report::ingest(
        &mut ctx,
        &ingest_args("not-registered"),
        Some(VERIFIER_JSON.into()),
    )
    .expect("a miss is a no-op at exit 0");

    assert_eq!(out.exit, None, "exit 0");
    assert_eq!(out.data["status"], "unregistered");
    assert!(out.warnings.iter().any(|w| w.code == "DFA-W411"));
    assert!(
        !p.exists(".devforgeai/reports/IDEA-003-explore.yaml"),
        "nothing is written"
    );
}

#[test]
fn ingest_unparsable_gives_e410_and_status_unparsed() {
    let p = seeded();
    let mut ctx = p.ctx();
    let out = report::ingest(
        &mut ctx,
        &ingest_args("kill-case-builder"),
        Some("this is prose, not JSON".into()),
    )
    .expect("exit 0 so SubagentStop does not block");

    assert_eq!(out.exit, None);
    assert_eq!(out.data["status"], "unparsed");
    assert!(out.warnings.iter().any(|w| w.code == "DFA-E410"));

    let text = p.read(".devforgeai/reports/IDEA-003-explore.yaml");
    assert!(text.contains("status: unparsed"));
    assert!(text.contains("passed: 0"));
    assert!(text.contains("total: 0"));
}

#[test]
fn ingest_without_active_id_gives_e412() {
    let p = Project::new();
    // `[active].explore` is empty.
    let mut ctx = p.ctx();
    let out = report::ingest(
        &mut ctx,
        &ingest_args("kill-case-builder"),
        Some(VERIFIER_JSON.into()),
    )
    .expect("exit 0");

    assert_eq!(out.exit, None);
    assert!(out.warnings.iter().any(|w| w.code == "DFA-E412"));
    assert_eq!(out.data["status"], "no_active_id");
}

#[test]
fn ingest_takes_the_phase_from_the_registry() {
    let p = Project::new();
    p.set_phase("verify", "STORY-014");
    let json = VERIFIER_JSON
        .replace("kill-case-builder", "ac-compliance-verifier")
        .replace("IDEA-003", "STORY-014")
        .replace("signals", "ACs");

    let mut ctx = p.ctx();
    let out = report::ingest(&mut ctx, &ingest_args("ac-compliance-verifier"), Some(json))
        .expect("ingest");
    assert_eq!(
        out.data["report"], ".devforgeai/reports/STORY-014-verify.yaml",
        "the [[verifier]] row fixes the phase"
    );
    assert_eq!(out.data["field"], "verifiers.ac_compliance");
}

// --------------------------------------------------------------- report show

#[test]
fn show_prints_yaml() {
    let p = Project::new();
    p.write(
        ".devforgeai/reports/STORY-014-build.yaml",
        &common::report("STORY-014", "build", "PASS"),
    );
    let mut ctx = p.ctx();
    let out = report::show(&mut ctx, "STORY-014", "build", None).expect("show");

    assert_eq!(out.data["id"], "STORY-014");
    assert_eq!(out.data["gate"]["result"], "PASS");
    assert_eq!(out.human[0], "Gate      build · STORY-014");
    assert!(out.human.iter().any(|l| l.starts_with("Result    PASS")));
}

#[test]
fn show_check_filters_one_entry() {
    let p = Project::new();
    p.write(
        ".devforgeai/reports/STORY-014-build.yaml",
        "schema: devforgeai/report/1\nid: STORY-014\nphase: build\nstatus: fail\nproduced_by: devforgeai-cli\nconsumes: []\nopen_questions: []\ngate:\n  result: FAIL\n  send_back_to: ''\n  checks:\n    - id: build-docs\n      kind: doc_valid\n      status: pass\n      severity: block\n      reason: ''\n      evidence: {}\n    - id: build-tests\n      kind: tests_pass\n      status: fail\n      severity: block\n      reason: 'DFA-E311 exited 101'\n      evidence: {}\nfindings: []\nhandoff: []\n",
    );
    let mut ctx = p.ctx();
    let out = report::show(&mut ctx, "STORY-014", "build", Some("build-tests")).expect("show");

    assert_eq!(out.data["id"], "build-tests");
    assert_eq!(out.data["kind"], "tests_pass");
    assert_eq!(out.data["status"], "fail");
    assert_eq!(out.human.len(), 1, "only that check's entry is printed");
}

#[test]
fn show_missing_gives_e400() {
    let p = Project::new();
    let mut ctx = p.ctx();
    let err = report::show(&mut ctx, "STORY-099", "build", None).expect_err("absent");
    assert_eq!(err.code(), "DFA-E400");
    assert_eq!(err.exit(), 1);
}

#[test]
fn show_unparsable_gives_e401() {
    let p = Project::new();
    p.write(
        ".devforgeai/reports/STORY-014-build.yaml",
        "gate: [unterminated\n",
    );
    let mut ctx = p.ctx();
    let err = report::show(&mut ctx, "STORY-014", "build", None).expect_err("bad YAML");
    assert_eq!(err.code(), "DFA-E401");
    assert_eq!(err.exit(), 1);
}

#[test]
fn show_unknown_phase_gives_e012() {
    let p = Project::new();
    let mut ctx = p.ctx();
    let err = report::show(&mut ctx, "STORY-014", "invent", None).expect_err("outside the enum");
    assert_eq!(err.code(), "DFA-E012");
    assert_eq!(err.exit(), 3);
}

// --------------------------------------------------------------- report note

fn note_args(key: &str, file: &str) -> ReportNoteArgs {
    ReportNoteArgs {
        id: "STORY-014".into(),
        phase: "build".into(),
        key: key.into(),
        file: file.into(),
    }
}

#[test]
fn report_note_writes_key() {
    let p = Project::new();
    p.write(
        "note.yaml",
        "schema: devforgeai/build-note/1\nid: STORY-014\napproach: outside-in\nfiles_touched: 3\ncommits: [a0d4f19]\n",
    );
    let mut ctx = p.ctx();
    let out = report::note(&mut ctx, &note_args("build", "note.yaml")).expect("note");

    assert_eq!(out.data["key"], "build");
    assert_eq!(out.data["keys_written"], 3, "schema and id are dropped");

    let text = p.read(".devforgeai/reports/STORY-014-build.yaml");
    assert!(text.contains("build:"));
    assert!(text.contains("approach: outside-in"));
    assert!(text.contains("files_touched: 3"));
    assert!(
        !text.contains("devforgeai/build-note/1"),
        "the note's own schema key is dropped"
    );
}

#[test]
fn report_note_keeps_produced_by() {
    let p = Project::new();
    p.write(
        "note.yaml",
        "schema: devforgeai/build-note/1\nid: STORY-014\napproach: outside-in\n",
    );
    let mut ctx = p.ctx();
    report::note(&mut ctx, &note_args("build", "note.yaml")).expect("note");

    let text = p.read(".devforgeai/reports/STORY-014-build.yaml");
    assert!(
        text.contains("produced_by: devforgeai-cli"),
        "the note changes no section 5 key of the report"
    );
    assert!(text.contains("schema: devforgeai/report/1"));
}

#[test]
fn report_note_bad_schema_gives_e413() {
    let p = Project::new();
    p.write(
        "note.yaml",
        "schema: something/else/1\napproach: outside-in\n",
    );
    let mut ctx = p.ctx();
    let err = report::note(&mut ctx, &note_args("build", "note.yaml"))
        .expect_err("the note schema is fixed");
    assert_eq!(err.code(), "DFA-E413");
    assert_eq!(err.exit(), 1);
    assert!(
        !p.exists(".devforgeai/reports/STORY-014-build.yaml"),
        "nothing is written"
    );
}

#[test]
fn report_note_key_outside_the_per_phase_set_gives_e013() {
    let p = Project::new();
    p.write("note.yaml", "schema: devforgeai/build-note/1\n");
    let mut ctx = p.ctx();

    let err = report::note(&mut ctx, &note_args("prose", "note.yaml"))
        .expect_err("the build phase accepts the single value build");
    assert_eq!(err.code(), "DFA-E013");
    assert_eq!(err.exit(), 3);

    // Every other phase accepts no key at all.
    let mut ctx = p.ctx();
    let args = ReportNoteArgs {
        id: "IDEA-003".into(),
        phase: "explore".into(),
        key: "build".into(),
        file: "note.yaml".into(),
    };
    let err = report::note(&mut ctx, &args).expect_err("explore accepts no note key");
    assert_eq!(err.code(), "DFA-E013");
}

#[test]
fn report_note_absent_file_gives_e400() {
    let p = Project::new();
    let mut ctx = p.ctx();
    let err = report::note(&mut ctx, &note_args("build", "nowhere.yaml")).expect_err("absent");
    assert_eq!(err.code(), "DFA-E400");
    assert_eq!(err.exit(), 1);
}

#[test]
fn report_note_preserves_the_gate_block() {
    let p = Project::new();
    p.write(
        ".devforgeai/reports/STORY-014-build.yaml",
        &common::report("STORY-014", "build", "PASS"),
    );
    p.write(
        "note.yaml",
        "schema: devforgeai/build-note/1\nid: STORY-014\napproach: outside-in\n",
    );
    let mut ctx = p.ctx();
    report::note(&mut ctx, &note_args("build", "note.yaml")).expect("note");

    let text = p.read(".devforgeai/reports/STORY-014-build.yaml");
    assert!(text.contains("result: PASS"), "the gate block survives");
    assert!(text.contains("approach: outside-in"));
}
