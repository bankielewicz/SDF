//! `phase set`: the transition table, `--remedy`, and the story `status`
//! writes, which this subcommand alone performs.

mod common;

use common::Project;
use devforgeai::cmd::phase;

fn sprint(id: &str, epic: &str, stories: &[&str]) -> String {
    let rows: String = stories
        .iter()
        .map(|s| format!("  - id: {s}\n    status: ready\n"))
        .collect();
    format!(
        "schema: devforgeai/sprint/1\nid: {id}\nphase: plan\nstatus: active\nproduced_by: planning-work\nconsumes: []\nopen_questions: []\nepic: {epic}\nstories:\n{rows}"
    )
}

/// A project whose plan gate passed for SPRINT-001, so `phase set build` is
/// allowed.
fn ready_for_build() -> Project {
    let p = Project::new();
    p.write(
        ".devforgeai/stories/sprint.yaml",
        &sprint("SPRINT-001", "EPIC-001", &["STORY-014"]),
    );
    p.write(
        ".devforgeai/reports/SPRINT-001-plan.yaml",
        &common::report("SPRINT-001", "plan", "PASS"),
    );
    p.write(
        ".devforgeai/stories/STORY-014.md",
        &common::story("STORY-014", "ready", &[], "# Order checkout\n"),
    );
    p
}

#[test]
fn set_entry_phase_without_predecessor_exits_zero() {
    let p = Project::new();
    let mut ctx = p.ctx();
    let out = phase::set(&mut ctx, "explore", "IDEA-003", None, None)
        .expect("an entry phase with no predecessor sets and exits 0");
    assert_eq!(out.data["to"], "explore");
    assert_eq!(out.data["required"], "");
}

#[test]
fn set_writes_current_and_active() {
    let p = Project::new();
    let mut ctx = p.ctx();
    phase::set(&mut ctx, "explore", "IDEA-003", None, None).expect("set");

    let s = p.state();
    assert_eq!(s.current.phase, "explore");
    assert_eq!(s.current.id, "IDEA-003");
    assert_eq!(s.active.explore, "IDEA-003");
}

#[test]
fn set_keeps_other_active_ids() {
    let p = ready_for_build();
    let mut s = p.state();
    s.active.explore = "IDEA-003".into();
    s.active.discover = "IDEA-003".into();
    s.active.plan = "SPRINT-001".into();
    // `[current].id` mirrors `[active].<current.phase>` in every valid file.
    s.current.phase = "plan".into();
    s.current.id = "SPRINT-001".into();
    p.write_state(&s);

    let mut ctx = p.ctx();
    phase::set(&mut ctx, "build", "STORY-014", None, None).expect("set");

    let s = p.state();
    assert_eq!(s.active.build, "STORY-014");
    assert_eq!(s.active.explore, "IDEA-003", "other keys keep their values");
    assert_eq!(s.active.discover, "IDEA-003");
    assert_eq!(s.active.plan, "SPRINT-001");
}

#[test]
fn set_explore_writes_timebox_from_config() {
    let p = Project::new();
    let mut cfg = devforgeai::config::Config {
        generated_at: "2026-09-10T14:02:11Z".into(),
        degraded: false,
        ..Default::default()
    };
    cfg.explore.timebox_days = 9;
    cfg.explore.remedy_timebox_days = 2;
    p.write_config(&cfg);

    let mut ctx = p.ctx();
    common::with_now("2026-09-11T08:00:00Z", || {
        phase::set(&mut ctx, "explore", "IDEA-003", None, None).expect("set")
    });

    let s = p.state();
    assert_eq!(s.explore.idea_id, "IDEA-003");
    assert_eq!(s.explore.started_at, "2026-09-11T08:00:00Z");
    assert_eq!(s.explore.timebox_days, 9, "from config.toml");
    assert!(s.explore.remedy_flows.is_empty());
}

#[test]
fn set_explore_remedy_keeps_started_at() {
    let p = Project::new();
    let mut ctx = p.ctx();
    common::with_now("2026-09-06T09:00:00Z", || {
        phase::set(&mut ctx, "explore", "IDEA-003", None, None).expect("set")
    });
    let first_start = p.state().explore.started_at.clone();
    assert_eq!(first_start, "2026-09-06T09:00:00Z");

    let mut ctx = p.ctx();
    common::with_now("2026-09-11T09:00:00Z", || {
        phase::set(
            &mut ctx,
            "explore",
            "IDEA-003",
            Some("FLOW-002,FLOW-003"),
            None,
        )
        .expect("remedy")
    });

    let s = p.state();
    assert_eq!(
        s.explore.started_at, first_start,
        "[explore].started_at is unchanged on a remedy"
    );
    assert_eq!(s.explore.remedy_started_at, "2026-09-11T09:00:00Z");
    assert_eq!(s.explore.remedy_flows, vec!["FLOW-002", "FLOW-003"]);
    assert_eq!(s.explore.remedy_timebox_days, 1);
}

#[test]
fn set_constitute_remedy_writes_remedy_con() {
    let p = Project::new();
    p.write(
        ".devforgeai/reports/IDEA-003-discover.yaml",
        &common::report("IDEA-003", "discover", "PASS"),
    );
    let mut ctx = p.ctx();
    phase::set(&mut ctx, "constitute", "IDEA-003", Some("CON-004"), None).expect("set");
    assert_eq!(p.state().constitute.remedy_con, "CON-004");
}

#[test]
fn set_remedy_on_other_phase_gives_e011() {
    let p = ready_for_build();
    let mut ctx = p.ctx();
    let err = phase::set(&mut ctx, "build", "STORY-014", Some("FIND-001"), None)
        .expect_err("every other phase takes no --remedy");
    assert_eq!(err.code(), "DFA-E011");
    assert_eq!(err.exit(), 3);
}

#[test]
fn set_explore_remedy_rejects_a_non_flow_id() {
    let p = Project::new();
    let mut ctx = p.ctx();
    let err = phase::set(&mut ctx, "explore", "IDEA-003", Some("REQ-001"), None)
        .expect_err("explore takes FLOW ids");
    assert_eq!(err.code(), "DFA-E011");
}

#[test]
fn set_without_predecessor_pass_gives_e320() {
    let p = Project::new();
    p.write(
        ".devforgeai/stories/sprint.yaml",
        &sprint("SPRINT-001", "EPIC-001", &["STORY-014"]),
    );
    p.write(
        ".devforgeai/reports/SPRINT-001-plan.yaml",
        &common::report("SPRINT-001", "plan", "FAIL"),
    );
    let before = p.read(".devforgeai/state.toml");

    let mut ctx = p.ctx();
    let err =
        phase::set(&mut ctx, "build", "STORY-014", None, None).expect_err("the plan gate failed");
    assert_eq!(err.code(), "DFA-E320");
    assert_eq!(err.exit(), 1);
    assert_eq!(
        p.read(".devforgeai/state.toml"),
        before,
        "nothing is written on a refusal"
    );
}

#[test]
fn set_resets_stop_counter() {
    let p = ready_for_build();
    let mut s = p.state();
    s.stop_hook.block_count = 1;
    s.stop_hook.blocked_phase = "build".into();
    s.stop_hook.blocked_id = "STORY-013".into();
    p.write_state(&s);

    let mut ctx = p.ctx();
    phase::set(&mut ctx, "build", "STORY-014", None, None).expect("set");

    let s = p.state();
    assert_eq!(s.stop_hook.block_count, 0);
    assert_eq!(s.stop_hook.blocked_phase, "");
    assert_eq!(s.stop_hook.blocked_id, "");
}

#[test]
fn set_build_writes_story_status_building() {
    let p = ready_for_build();
    let mut ctx = p.ctx();
    let out = phase::set(&mut ctx, "build", "STORY-014", None, None).expect("set");

    let text = p.read(".devforgeai/stories/STORY-014.md");
    assert!(text.contains("status: building"));
    assert!(text.contains("# Order checkout"), "no other byte moves");
    assert_eq!(
        out.data["status_writes"][0]["path"],
        ".devforgeai/stories/STORY-014.md"
    );
    assert_eq!(out.data["status_writes"][0]["status"], "building");
}

#[test]
fn set_verify_writes_story_status_built() {
    let p = ready_for_build();
    p.write(
        ".devforgeai/stories/STORY-014.md",
        &common::story("STORY-014", "building", &[], "# Order checkout\n"),
    );
    p.write(
        ".devforgeai/reports/STORY-014-build.yaml",
        &common::report("STORY-014", "build", "PASS"),
    );

    let mut ctx = p.ctx();
    phase::set(&mut ctx, "verify", "STORY-014", None, None).expect("set");
    assert!(p
        .read(".devforgeai/stories/STORY-014.md")
        .contains("status: built"));
}

#[test]
fn set_release_writes_every_story_in_the_manifest() {
    let p = Project::new();
    p.write(
        ".devforgeai/releases/v0.3.0.yaml",
        "schema: devforgeai/release/1\nid: v0.3.0\nphase: release\nstatus: released\nproduced_by: releasing-software\nconsumes: []\nopen_questions: []\nstories:\n  - id: STORY-014\n  - id: STORY-015\n",
    );
    for id in ["STORY-014", "STORY-015"] {
        p.write(
            &format!(".devforgeai/stories/{id}.md"),
            &common::story(id, "built", &[], "# T\n"),
        );
        p.write(
            &format!(".devforgeai/reports/{id}-verify.yaml"),
            &common::report(id, "verify", "PASS"),
        );
    }

    let mut ctx = p.ctx();
    let out = phase::set(&mut ctx, "release", "v0.3.0", None, None).expect("set");
    assert_eq!(
        out.data["status_writes"].as_array().expect("writes").len(),
        2
    );
    assert!(p
        .read(".devforgeai/stories/STORY-014.md")
        .contains("status: released"));
    assert!(p
        .read(".devforgeai/stories/STORY-015.md")
        .contains("status: released"));
}

#[test]
fn set_leaves_a_document_already_at_the_target_value() {
    let p = ready_for_build();
    p.write(
        ".devforgeai/stories/STORY-014.md",
        &common::story("STORY-014", "building", &[], "# Order checkout\n"),
    );
    let before = p.read(".devforgeai/stories/STORY-014.md");

    let mut ctx = p.ctx();
    let out = phase::set(&mut ctx, "build", "STORY-014", None, None).expect("set");
    assert_eq!(p.read(".devforgeai/stories/STORY-014.md"), before);
    assert!(
        out.data["status_writes"]
            .as_array()
            .expect("writes")
            .is_empty(),
        "a document already carrying the value is left untouched"
    );
}

#[test]
fn set_absent_document_gives_w210_and_still_advances() {
    let p = ready_for_build();
    std::fs::remove_file(p.root().join(".devforgeai/stories/STORY-014.md")).expect("remove");

    let mut ctx = p.ctx();
    let out = phase::set(&mut ctx, "build", "STORY-014", None, None).expect("exit 0");
    assert!(out.warnings.iter().any(|w| w.code == "DFA-W210"));
    assert_eq!(
        p.state().current.id,
        "STORY-014",
        "state.toml still advanced"
    );
}

#[test]
fn set_phases_that_write_no_status() {
    let p = Project::new();
    p.write(
        ".devforgeai/reports/IDEA-003-discover.yaml",
        &common::report("IDEA-003", "discover", "PASS"),
    );
    let mut ctx = p.ctx();
    let out = phase::set(&mut ctx, "constitute", "IDEA-003", None, None).expect("set");
    assert!(out.data["status_writes"]
        .as_array()
        .expect("writes")
        .is_empty());
}

#[test]
fn set_unknown_phase_gives_e012() {
    let p = Project::new();
    let mut ctx = p.ctx();
    let err = phase::set(&mut ctx, "invent", "IDEA-003", None, None).expect_err("outside the enum");
    assert_eq!(err.code(), "DFA-E012");
    assert_eq!(err.exit(), 3);
}

#[test]
fn set_json_shape() {
    let p = ready_for_build();
    let mut ctx = p.ctx();
    let out = phase::set(&mut ctx, "build", "STORY-014", None, None).expect("set");
    for key in [
        "from",
        "to",
        "id",
        "required",
        "required_result",
        "remedy",
        "status_writes",
    ] {
        assert!(out.data.get(key).is_some(), "{key} is in the data object");
    }
    assert_eq!(out.data["to"], "build");
    assert_eq!(out.data["required"], "plan");
    assert_eq!(out.data["required_result"], "PASS");
}

#[test]
fn set_human_output_names_the_phase() {
    let p = ready_for_build();
    let mut ctx = p.ctx();
    let out = phase::set(&mut ctx, "build", "STORY-014", None, None).expect("set");
    assert!(
        out.human[0].starts_with("Phase     4 · Build"),
        "{}",
        out.human[0]
    );
    assert!(out.human[0].contains("STORY-014"));
}

// --- the two cross-cutting phases ----------------------------------------

#[test]
fn phase_set_design_records_the_run_and_advances_nothing() {
    let p = Project::new();
    p.set_phase("build", "STORY-014");

    let mut ctx = p.ctx();
    let out = devforgeai::cmd::phase::set(&mut ctx, "design", "UI-001", None, None)
        .expect("design is entered from any phase");

    // Design runs beside the seven rather than inside them: the story is still
    // in Build, and moving `[current]` would strand it.
    let s = p.state();
    assert_eq!(s.current.phase, "build", "the delivery chain did not move");
    assert_eq!(s.current.id, "STORY-014");
    assert_eq!(s.active.build, "STORY-014");
    assert_eq!(s.last_cross.phase, "design");
    assert_eq!(s.last_cross.id, "UI-001");
    assert!(!s.last_cross.turn.is_empty(), "the turn is recorded");
    assert_eq!(out.data["cross_cutting"], true);
    assert_eq!(out.data["from"], "build");
    assert_eq!(out.data["to"], "design");
}

#[test]
fn phase_set_reflect_records_the_run_and_advances_nothing() {
    let p = Project::new();
    p.set_phase("verify", "STORY-014");

    let mut ctx = p.ctx();
    devforgeai::cmd::phase::set(&mut ctx, "reflect", "2026-09-11", None, None).expect("reflect");

    let s = p.state();
    assert_eq!(s.current.phase, "verify");
    assert_eq!(s.last_cross.phase, "reflect");
    assert_eq!(s.last_cross.id, "2026-09-11");
}

#[test]
fn phase_set_cross_cutting_takes_no_remedy() {
    let p = Project::new();
    p.set_phase("build", "STORY-014");

    let mut ctx = p.ctx();
    let err = devforgeai::cmd::phase::set(&mut ctx, "design", "UI-001", Some("FLOW-001"), None)
        .expect_err("design takes no --remedy");
    assert_eq!(err.code(), "DFA-E011");
}

#[test]
fn phase_set_cross_cutting_needs_no_predecessor_gate() {
    // A freshly initialised project at phase explore with no gate anywhere:
    // Design is still reachable, because it gates nothing in the chain.
    let p = Project::new();

    let mut ctx = p.ctx();
    devforgeai::cmd::phase::set(&mut ctx, "design", "UI-001", None, None)
        .expect("no predecessor gate stands between a phase and Design");
    assert_eq!(p.state().last_cross.id, "UI-001");
}

#[test]
fn phase_set_refuses_a_predecessor_that_produced_no_report() {
    let p = Project::new();
    // A release naming no story: the predecessor loop runs zero times, so the
    // result keeps the value it was initialised with and the whole phase chain
    // becomes skippable in one call.
    p.write(
        ".devforgeai/releases/v1.0.0.yaml",
        "schema: devforgeai/release/1\nid: v1.0.0\nphase: release\nstatus: draft\nproduced_by: releasing-software\nconsumes: []\nopen_questions: []\nstories: []\n",
    );

    let mut ctx = p.ctx();
    let err = devforgeai::cmd::phase::set(&mut ctx, "release", "v1.0.0", None, None)
        .expect_err("a release that verifies nothing does not advance the phase");
    assert_eq!(err.code(), "DFA-E320");
    assert_eq!(err.exit(), 1);

    let s = p.state();
    assert_eq!(s.current.phase, "explore", "nothing was written");
    assert_eq!(s.active.release, "");
}

// --- plan is entered before its own document exists -----------------------

/// A project whose constitute gate has passed for `EPIC-001`'s idea, so a plan
/// run is reachable.
fn ready_to_plan() -> Project {
    let p = Project::new();
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-004", "accepted"),
    );
    p.write(
        ".devforgeai/reports/IDEA-004-constitute.yaml",
        &common::report("IDEA-004", "constitute", "PASS"),
    );
    p
}

#[test]
fn phase_set_plan_requires_an_epic_before_the_sprint_file_exists() {
    let p = ready_to_plan();

    // The skill allocates the sprint id at step 4 and writes the file at step
    // 12, so this call runs against a file that is not there.
    let mut ctx = p.ctx();
    let err = devforgeai::cmd::phase::set(&mut ctx, "plan", "SPRINT-002", None, None)
        .expect_err("a sprint that does not exist resolves to no epic");
    assert_eq!(err.code(), "DFA-E011");
    let msg = &err.diag().expect("diag").message;
    assert!(msg.contains("--epic"), "the flag is named: {msg}");
}

#[test]
fn phase_set_plan_with_an_epic_records_it_and_advances() {
    let p = ready_to_plan();

    let mut ctx = p.ctx();
    devforgeai::cmd::phase::set(&mut ctx, "plan", "SPRINT-002", None, Some("EPIC-001"))
        .expect("the epic supplies what the absent sprint file would have said");

    let s = p.state();
    assert_eq!(s.current.phase, "plan");
    assert_eq!(s.current.id, "SPRINT-002");
    assert_eq!(s.active.plan, "SPRINT-002", "the sprint is the subject");
    assert_eq!(
        s.plan.epic, "EPIC-001",
        "the epic is what the gate resolved"
    );
}

#[test]
fn phase_set_plan_refuses_an_epic_that_is_not_an_id() {
    let p = ready_to_plan();
    let mut ctx = p.ctx();
    let err = devforgeai::cmd::phase::set(&mut ctx, "plan", "SPRINT-002", None, Some("STORY-001"))
        .expect_err("a STORY is not an epic");
    assert_eq!(err.code(), "DFA-E013");
}

#[test]
fn phase_set_plan_refuses_an_epic_that_contradicts_the_sprint_file() {
    let p = ready_to_plan();
    p.write(
        ".devforgeai/stories/sprint.yaml",
        "schema: devforgeai/sprint/1\nid: SPRINT-002\nphase: plan\nstatus: planned\nproduced_by: planning-work\nconsumes: []\nopen_questions: []\nepic: EPIC-001\nstories: []\n",
    );

    // With the file present the flag is a cross-check: a disagreement means
    // the caller and the document name different epics, and guessing which is
    // right would either advance the wrong plan or ignore the argument.
    let mut ctx = p.ctx();
    let err = devforgeai::cmd::phase::set(&mut ctx, "plan", "SPRINT-002", None, Some("EPIC-009"))
        .expect_err("the two disagree");
    assert_eq!(err.code(), "DFA-E013");
    let msg = &err.diag().expect("diag").message;
    assert!(msg.contains("EPIC-009"), "{msg}");
    assert!(msg.contains("EPIC-001"), "{msg}");
}

#[test]
fn phase_set_plan_accepts_an_epic_that_matches_the_sprint_file() {
    let p = ready_to_plan();
    p.write(
        ".devforgeai/stories/sprint.yaml",
        "schema: devforgeai/sprint/1\nid: SPRINT-002\nphase: plan\nstatus: planned\nproduced_by: planning-work\nconsumes: []\nopen_questions: []\nepic: EPIC-001\nstories: []\n",
    );

    let mut ctx = p.ctx();
    devforgeai::cmd::phase::set(&mut ctx, "plan", "SPRINT-002", None, Some("EPIC-001"))
        .expect("the flag agrees with the document");
    assert_eq!(p.state().current.id, "SPRINT-002");
}

#[test]
fn phase_set_refuses_an_epic_on_any_other_phase() {
    let p = Project::new();
    let mut ctx = p.ctx();
    let err = devforgeai::cmd::phase::set(&mut ctx, "explore", "IDEA-003", None, Some("EPIC-001"))
        .expect_err("only plan takes an epic");
    assert_eq!(err.code(), "DFA-E011");
}
