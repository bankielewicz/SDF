//! `devforgeai handoff` over the library: the block with no report, the
//! `[last_handoff]` record, and the `--json` `data` object.
//!
//! The dispatcher is not wired for every subcommand yet, so these drive
//! `devforgeai::cmd::handoff::run` with a `Ctx` over a temporary project. No
//! test reads the real home directory.

mod common;

use common::Project;

/// A build report for a story, written where `handoff` looks for it.
fn write_build_report(p: &Project, id: &str) {
    p.write(
        &format!(".devforgeai/reports/{id}-build.yaml"),
        &format!(
            r#"schema: devforgeai/report/1
id: {id}
phase: build
status: pass
produced_by: devforgeai-cli
consumes: []
open_questions: []
started_at: 2026-09-10T14:01:03Z
finished_at: 2026-09-10T14:02:11Z
cli_version: 1.0.0
degraded: false
coverage:
  overall: 87.4
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
        passed: 214
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
"#
        ),
    );
}

/// The story document the `Phase` line takes its slug from.
fn write_story(p: &Project, id: &str) {
    p.write(
        &format!(".devforgeai/stories/{id}.md"),
        &common::story(id, "building", &[], "# Order checkout\n\n- AC-001: one\n"),
    );
}

#[test]
fn handoff_without_report_renders_not_run() {
    std::env::set_var("DEVFORGEAI_NOW", "2026-09-10T14:02:12Z");
    let p = Project::new();
    p.set_phase("build", "STORY-014");
    write_story(&p, "STORY-014");

    let mut ctx = p.ctx();
    let out = devforgeai::cmd::handoff::run(&mut ctx, None, None).expect("handoff renders");

    assert_eq!(out.exit, None, "every rendering case exits 0");
    assert!(
        out.human.iter().any(|l| l == "Gate      NOT RUN"),
        "{:#?}",
        out.human
    );
    assert_eq!(out.human.last().expect("a last line"), "Full report: none");
    assert_eq!(
        out.human[0], "Phase     4 · Build           STORY-014 · order-checkout",
        "the block still names the phase and the slug"
    );
    assert!(out.human.len() <= 12, "the twelve-line cap holds");
    std::env::remove_var("DEVFORGEAI_NOW");
}

#[test]
fn handoff_writes_last_handoff() {
    std::env::set_var("DEVFORGEAI_NOW", "2026-09-10T14:02:12Z");
    let p = Project::new();
    p.set_phase("build", "STORY-014");
    write_story(&p, "STORY-014");
    write_build_report(&p, "STORY-014");

    let mut ctx = p.ctx();
    let out = devforgeai::cmd::handoff::run(&mut ctx, None, None).expect("handoff renders");

    let state = p.state();
    assert_eq!(state.last_handoff.phase, "build");
    assert_eq!(state.last_handoff.id, "STORY-014");
    assert_eq!(state.last_handoff.rendered_at, "2026-09-10T14:02:12Z");
    assert_eq!(state.last_handoff.lines, out.human);
    assert!(
        state
            .last_handoff
            .lines
            .iter()
            .any(|l| l == "Gate      PASS  1 checks"),
        "{:#?}",
        state.last_handoff.lines
    );
    assert!(state
        .last_handoff
        .lines
        .iter()
        .any(|l| l == "Done      214 tests · 87.4% coverage"));
    assert!(state
        .last_handoff
        .lines
        .iter()
        .any(|l| l == "Verified  ac-compliance-verifier · 7/7 ACs"));
    std::env::remove_var("DEVFORGEAI_NOW");
}

#[test]
fn handoff_json_carries_lines() {
    std::env::set_var("DEVFORGEAI_NOW", "2026-09-10T14:02:12Z");
    let p = Project::new();
    p.set_phase("build", "STORY-014");
    write_story(&p, "STORY-014");
    write_build_report(&p, "STORY-014");
    // `[active].release` non-empty makes the build PASS row render `Then`.
    let mut s = p.state();
    s.active.release = "v0.3.0".into();
    s.active.verify = "STORY-014".into();
    p.write_state(&s);

    let mut ctx = p.ctx();
    let out = devforgeai::cmd::handoff::run(&mut ctx, None, None).expect("handoff renders");
    let data = &out.data;

    let lines = data["lines"].as_array().expect("lines is an array");
    assert_eq!(lines.len(), out.human.len());
    assert_eq!(lines[0].as_str(), Some(out.human[0].as_str()));
    assert_eq!(data["phase"], "build");
    assert_eq!(data["id"], "STORY-014");
    assert_eq!(data["result"], "PASS");
    assert_eq!(data["next"], "/verify STORY-014");
    assert_eq!(data["then"], "/release v0.3.0");
    assert_eq!(data["blocked"], "none");
    assert_eq!(data["report"], ".devforgeai/reports/STORY-014-build.yaml");
    std::env::remove_var("DEVFORGEAI_NOW");
}
