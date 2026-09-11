//! `gate require`: the id-resolution chain, the predecessor report, and the
//! rule that nothing is written.

mod common;

use common::Project;
use devforgeai::cmd::gate;

fn sprint(id: &str, epic: &str, stories: &[&str]) -> String {
    let rows: String = stories
        .iter()
        .map(|s| format!("  - id: {s}\n    status: ready\n"))
        .collect();
    format!(
        "schema: devforgeai/sprint/1\nid: {id}\nphase: plan\nstatus: active\nproduced_by: planning-work\nconsumes: []\nopen_questions: []\nepic: {epic}\nstories:\n{rows}"
    )
}

#[test]
fn require_with_empty_predecessor_exits_zero() {
    let p = Project::new();
    let mut ctx = p.ctx();
    // Explore has no predecessor in any project.
    let out = gate::require(&mut ctx, "explore", "IDEA-003").expect("exit 0");
    assert_eq!(out.data["requires"], "");
    assert_eq!(out.data["reports"], serde_json::json!([]));
}

#[test]
fn require_discover_without_explore_exits_zero() {
    let p = Project::new();
    let mut ctx = p.ctx();
    // No decision.yaml: Discover is an entry phase.
    let out = gate::require(&mut ctx, "discover", "IDEA-003").expect("exit 0");
    assert_eq!(out.data["requires"], "");
    assert_eq!(out.data["reports"], serde_json::json!([]));
}

#[test]
fn require_discover_with_an_unparsable_decision_is_e401() {
    // An unparsable decision file is a defect in the document, not evidence
    // that no explore run happened: collapsing the two lets one corrupted file
    // skip the explore predecessor entirely.
    let p = Project::new();
    p.write(".devforgeai/explore/decision.yaml", "decision: [promote\n");
    let mut ctx = p.ctx();
    let err = gate::require(&mut ctx, "discover", "IDEA-003")
        .expect_err("an unparsable decision is an error");
    assert_eq!(err.code(), "DFA-E401");
}

#[test]
fn require_discover_with_a_decision_for_another_idea_exits_zero() {
    let p = Project::new();
    p.write(
        ".devforgeai/explore/decision.yaml",
        &common::decision("IDEA-009", "promote"),
    );
    let mut ctx = p.ctx();
    let out = gate::require(&mut ctx, "discover", "IDEA-003").expect("exit 0");
    assert_eq!(out.data["requires"], "");
}

#[test]
fn require_discover_with_explore_reads_the_report() {
    let p = Project::new();
    p.write(
        ".devforgeai/explore/decision.yaml",
        &common::decision("IDEA-003", "promote"),
    );
    p.write(
        ".devforgeai/reports/IDEA-003-explore.yaml",
        &common::report("IDEA-003", "explore", "PASS"),
    );

    let mut ctx = p.ctx();
    let out = gate::require(&mut ctx, "discover", "IDEA-003").expect("the explore gate passed");
    assert_eq!(out.data["requires"], "explore");
    assert_eq!(out.data["subject"], "IDEA-003");
}

#[test]
fn require_pass_report_exits_zero() {
    let p = Project::new();
    p.write(
        ".devforgeai/reports/IDEA-003-discover.yaml",
        &common::report("IDEA-003", "discover", "PASS"),
    );
    let mut ctx = p.ctx();
    let out = gate::require(&mut ctx, "constitute", "IDEA-003").expect("PASS");
    assert_eq!(out.data["result"], "PASS");
    assert_eq!(out.data["requires"], "discover");
}

#[test]
fn require_refuses_a_report_holding_an_unimplemented_check() {
    // `gate check` records a kind this milestone does not evaluate as a skip,
    // and a skip counts as passing, so the report it leaves on disk reads
    // `gate.result: PASS` even though the command refused at exit 5. The
    // report schema's result enum is PASS | FAIL | SEND_BACK, with no value
    // for "never ran", so `require` is the place that must refuse: without
    // this, the next phase starts on a gate whose stubbed checks never ran.
    let p = Project::new();
    p.write(
        ".devforgeai/reports/IDEA-003-discover.yaml",
        &common::report("IDEA-003", "discover", "PASS").replace(
            "  checks: []",
            "  checks:\n    - id: discover-tests\n      kind: tests_pass\n      status: skip\n      severity: block\n      reason: not_implemented\n      evidence: {}",
        ),
    );
    let mut ctx = p.ctx();
    let err = gate::require(&mut ctx, "constitute", "IDEA-003")
        .expect_err("a PASS resting on an unevaluated check is not a PASS");
    assert_eq!(err.code(), "DFA-E321");
    let message = err.diag().expect("diag").message.clone();
    assert!(
        message.contains("tests_pass"),
        "the refusal names the kind that never ran; message was {message}"
    );
}

#[test]
fn require_accepts_a_report_whose_skips_are_ordinary() {
    // A `degraded` or `no_run` skip is a real evaluation outcome, not a gap in
    // the build, so it must keep passing.
    let p = Project::new();
    p.write(
        ".devforgeai/reports/IDEA-003-discover.yaml",
        &common::report("IDEA-003", "discover", "PASS").replace(
            "  checks: []",
            "  checks:\n    - id: discover-lint\n      kind: lint_clean\n      status: skip\n      severity: block\n      reason: degraded\n      evidence: {}",
        ),
    );
    let mut ctx = p.ctx();
    let out = gate::require(&mut ctx, "constitute", "IDEA-003").expect("an ordinary skip passes");
    assert_eq!(out.data["result"], "PASS");
}

#[test]
fn require_fail_report_gives_e321() {
    let p = Project::new();
    p.write(
        ".devforgeai/reports/IDEA-003-discover.yaml",
        &common::report("IDEA-003", "discover", "FAIL"),
    );
    let mut ctx = p.ctx();
    let err = gate::require(&mut ctx, "constitute", "IDEA-003").expect_err("not PASS");
    assert_eq!(err.code(), "DFA-E321");
    assert_eq!(err.exit(), 1);
    assert!(err.diag().expect("diag").message.contains("FAIL"));
}

#[test]
fn require_missing_report_gives_e321() {
    let p = Project::new();
    let mut ctx = p.ctx();
    let err = gate::require(&mut ctx, "constitute", "IDEA-003").expect_err("no report");
    assert_eq!(err.code(), "DFA-E321");
    assert!(err.diag().expect("diag").message.contains("no report at"));
}

#[test]
fn require_plan_resolves_idea_through_requirements() {
    let p = Project::new();
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-003", "accepted"),
    );
    p.write(
        ".devforgeai/reports/IDEA-003-constitute.yaml",
        &common::report("IDEA-003", "constitute", "PASS"),
    );

    let mut ctx = p.ctx();
    let out = gate::require(&mut ctx, "plan", "EPIC-001")
        .expect("the epic resolves to the document's IDEA");
    assert_eq!(out.data["subject"], "IDEA-003");
    assert_eq!(out.data["requires"], "constitute");
}

#[test]
fn require_plan_resolves_a_sprint_through_its_epic() {
    let p = Project::new();
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-003", "accepted"),
    );
    p.write(
        ".devforgeai/stories/sprint.yaml",
        &sprint("SPRINT-001", "EPIC-001", &["STORY-014"]),
    );
    p.write(
        ".devforgeai/reports/IDEA-003-constitute.yaml",
        &common::report("IDEA-003", "constitute", "PASS"),
    );

    let mut ctx = p.ctx();
    let out = gate::require(&mut ctx, "plan", "SPRINT-001")
        .expect("a SPRINT argument resolves to its epic first");
    assert_eq!(out.data["subject"], "IDEA-003");
}

#[test]
fn require_build_resolves_the_sprint_holding_the_story() {
    let p = Project::new();
    p.write(
        ".devforgeai/stories/sprint.yaml",
        &sprint("SPRINT-001", "EPIC-001", &["STORY-014"]),
    );
    p.write(
        ".devforgeai/reports/SPRINT-001-plan.yaml",
        &common::report("SPRINT-001", "plan", "PASS"),
    );

    let mut ctx = p.ctx();
    let out = gate::require(&mut ctx, "build", "STORY-014").expect("the sprint lists the story");
    assert_eq!(out.data["subject"], "SPRINT-001");
    assert_eq!(
        out.data["reports"],
        serde_json::json!([".devforgeai/reports/SPRINT-001-plan.yaml"])
    );
}

#[test]
fn require_build_story_in_no_sprint_gives_e013() {
    let p = Project::new();
    p.write(
        ".devforgeai/stories/sprint.yaml",
        &sprint("SPRINT-001", "EPIC-001", &["STORY-014"]),
    );
    let mut ctx = p.ctx();
    let err = gate::require(&mut ctx, "build", "STORY-099").expect_err("listed in no sprint");
    assert_eq!(err.code(), "DFA-E013");
    assert_eq!(err.exit(), 3);
}

#[test]
fn require_verify_reads_build_report_same_story() {
    let p = Project::new();
    p.write(
        ".devforgeai/reports/STORY-014-build.yaml",
        &common::report("STORY-014", "build", "PASS"),
    );
    let mut ctx = p.ctx();
    let out = gate::require(&mut ctx, "verify", "STORY-014").expect("PASS");
    assert_eq!(out.data["subject"], "STORY-014");
    assert_eq!(out.data["requires"], "build");
}

#[test]
fn require_release_checks_every_story_in_manifest() {
    let p = Project::new();
    p.write(
        ".devforgeai/releases/v0.3.0.yaml",
        "schema: devforgeai/release/1\nid: v0.3.0\nphase: release\nstatus: released\nproduced_by: releasing-software\nconsumes: []\nopen_questions: []\nstories:\n  - id: STORY-014\n  - id: STORY-015\n",
    );
    p.write(
        ".devforgeai/reports/STORY-014-verify.yaml",
        &common::report("STORY-014", "verify", "PASS"),
    );
    p.write(
        ".devforgeai/reports/STORY-015-verify.yaml",
        &common::report("STORY-015", "verify", "PASS"),
    );

    let mut ctx = p.ctx();
    let out = gate::require(&mut ctx, "release", "v0.3.0").expect("every story passes");
    assert_eq!(out.data["reports"].as_array().expect("reports").len(), 2);

    // One story short of PASS fails the call.
    p.write(
        ".devforgeai/reports/STORY-015-verify.yaml",
        &common::report("STORY-015", "verify", "FAIL"),
    );
    let mut ctx = p.ctx();
    let err = gate::require(&mut ctx, "release", "v0.3.0").expect_err("one story is not PASS");
    assert_eq!(err.code(), "DFA-E321");
    assert!(err.diag().expect("diag").message.contains("STORY-015"));
}

#[test]
fn require_release_before_the_release_file_exists_warns_and_reads_the_sprint() {
    // `releasing-software` runs `phase set release --id $1` at step 2 and
    // writes `releases/$1.yaml` at step 13, so on a first release the file
    // cannot exist yet. Refusing with DFA-E200 there stops the run before it
    // starts; the absent file is the first-release case, and the story set
    // comes from the sprint instead.
    let p = Project::new();
    p.write(
        ".devforgeai/stories/sprint.yaml",
        &sprint("SPRINT-001", "EPIC-001", &["STORY-014", "STORY-015"]),
    );
    p.write(
        ".devforgeai/reports/STORY-014-verify.yaml",
        &common::report("STORY-014", "verify", "PASS"),
    );
    p.write(
        ".devforgeai/reports/STORY-015-verify.yaml",
        &common::report("STORY-015", "verify", "PASS"),
    );

    let mut ctx = p.ctx();
    let out = gate::require(&mut ctx, "release", "v0.3.0")
        .expect("an absent release file is the first-release case, not an error");
    assert_eq!(out.exit.unwrap_or(0), 0);
    assert!(
        out.warnings.iter().any(|d| d.code == "DFA-W210"),
        "the absent release file is reported as a warning: {:?}",
        out.warnings
    );
    assert_eq!(out.data["requires"], "verify");
    assert_eq!(out.data["result"], "PASS");
    assert_eq!(
        out.data["reports"].as_array().expect("reports").len(),
        2,
        "the verify gate of every story the sprint names is still read"
    );

    // The verify gates still bind: one story short of PASS refuses.
    p.write(
        ".devforgeai/reports/STORY-015-verify.yaml",
        &common::report("STORY-015", "verify", "FAIL"),
    );
    let mut ctx = p.ctx();
    let err = gate::require(&mut ctx, "release", "v0.3.0").expect_err("one story is not PASS");
    assert_eq!(err.code(), "DFA-E321");
    assert!(err.diag().expect("diag").message.contains("STORY-015"));
}

#[test]
fn require_release_with_neither_a_release_file_nor_a_sprint_reads_nothing() {
    // Nothing names a story, so nothing was verified. The call still exits 0
    // with the warning, and the result never reaches PASS on an empty read,
    // which is what keeps `phase set` from advancing on it.
    let p = Project::new();
    let mut ctx = p.ctx();
    let out = gate::require(&mut ctx, "release", "v0.3.0").expect("exit 0");
    assert!(out.warnings.iter().any(|d| d.code == "DFA-W210"));
    assert_eq!(out.data["reports"], serde_json::json!([]));
    assert_ne!(
        out.data["result"], "PASS",
        "a release that verified nothing is not a PASS"
    );
}

#[test]
fn require_release_with_an_unparsable_release_file_is_still_an_error() {
    // A file that is present and malformed is a defect, not an absence.
    let p = Project::new();
    p.write(".devforgeai/releases/v0.3.0.yaml", "stories: [STORY-014\n");
    let mut ctx = p.ctx();
    let err =
        gate::require(&mut ctx, "release", "v0.3.0").expect_err("a malformed file is an error");
    assert_eq!(err.code(), "DFA-E401");
}

// ------------------------------------------------- the plan arm's resolution

#[test]
fn require_plan_with_an_epic_no_document_defines_is_e013() {
    let p = Project::new();
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-003", "accepted"),
    );
    p.write(
        ".devforgeai/reports/IDEA-003-constitute.yaml",
        &common::report("IDEA-003", "constitute", "PASS"),
    );
    let mut ctx = p.ctx();
    let err = gate::require(&mut ctx, "plan", "EPIC-999")
        .expect_err("requirements.yaml holds no EPIC-999");
    assert_eq!(err.code(), "DFA-E013");
    assert_eq!(err.exit(), 3);
    assert!(err.diag().expect("diag").message.contains("EPIC-999"));
}

#[test]
fn require_plan_sprint_naming_an_unknown_epic_is_e013() {
    let p = Project::new();
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-003", "accepted"),
    );
    p.write(
        ".devforgeai/stories/sprint.yaml",
        &sprint("SPRINT-001", "EPIC-909", &["STORY-014"]),
    );
    p.write(
        ".devforgeai/reports/IDEA-003-constitute.yaml",
        &common::report("IDEA-003", "constitute", "PASS"),
    );
    let mut ctx = p.ctx();
    let err = gate::require(&mut ctx, "plan", "SPRINT-001")
        .expect_err("the sprint's epic is in no requirements document");
    assert_eq!(err.code(), "DFA-E013");
    assert!(err.diag().expect("diag").message.contains("EPIC-909"));
}

#[test]
fn require_plan_sprint_without_sprint_yaml_is_e013() {
    let p = Project::new();
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-003", "accepted"),
    );
    let mut ctx = p.ctx();
    let err =
        gate::require(&mut ctx, "plan", "SPRINT-042").expect_err("no sprint.yaml to resolve with");
    assert_eq!(err.code(), "DFA-E013");
    assert_eq!(err.exit(), 3);
}

#[test]
fn require_plan_sprint_without_an_epic_key_is_e013() {
    let p = Project::new();
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-003", "accepted"),
    );
    p.write(
        ".devforgeai/stories/sprint.yaml",
        "schema: devforgeai/sprint/1\nid: SPRINT-001\nphase: plan\nstatus: active\nproduced_by: planning-work\nconsumes: []\nopen_questions: []\nstories:\n  - id: STORY-014\n    status: ready\n",
    );
    let mut ctx = p.ctx();
    let err =
        gate::require(&mut ctx, "plan", "SPRINT-001").expect_err("sprint.yaml carries no epic");
    assert_eq!(err.code(), "DFA-E013");
    assert!(err.diag().expect("diag").message.contains("epic"));
}

#[test]
fn require_plan_sprint_with_an_unparsable_sprint_yaml_is_e401() {
    let p = Project::new();
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-003", "accepted"),
    );
    p.write(".devforgeai/stories/sprint.yaml", "epic: [EPIC-001\n");
    let mut ctx = p.ctx();
    let err = gate::require(&mut ctx, "plan", "SPRINT-001").expect_err("sprint.yaml is malformed");
    assert_eq!(err.code(), "DFA-E401");
}

#[test]
fn require_build_with_a_sprint_lacking_an_id_is_e013() {
    let p = Project::new();
    p.write(
        ".devforgeai/stories/sprint.yaml",
        "schema: devforgeai/sprint/1\nphase: plan\nstatus: active\nproduced_by: planning-work\nconsumes: []\nopen_questions: []\nepic: EPIC-001\nstories:\n  - id: STORY-014\n    status: ready\n",
    );
    let mut ctx = p.ctx();
    let err =
        gate::require(&mut ctx, "build", "STORY-014").expect_err("the sprint has no id of its own");
    assert_eq!(err.code(), "DFA-E013");
    assert!(err.diag().expect("diag").message.contains("sprint.yaml"));
}

// --------------------------------------------------------- the release arm

#[test]
fn require_release_with_an_empty_stories_list_is_e321() {
    let p = Project::new();
    p.write(
        ".devforgeai/releases/v1.0.0.yaml",
        "schema: devforgeai/release/1\nid: v1.0.0\nphase: release\nstatus: released\nproduced_by: releasing-software\nconsumes: []\nopen_questions: []\nstories: []\n",
    );
    let mut ctx = p.ctx();
    let err = gate::require(&mut ctx, "release", "v1.0.0")
        .expect_err("a release verifying nothing is not a PASS");
    assert_eq!(err.code(), "DFA-E321");
    assert_eq!(err.exit(), 1);
}

#[test]
fn require_release_with_an_absent_stories_key_is_e321() {
    let p = Project::new();
    p.write(
        ".devforgeai/releases/v1.0.0.yaml",
        "schema: devforgeai/release/1\nid: v1.0.0\nphase: release\nstatus: released\nproduced_by: releasing-software\nconsumes: []\nopen_questions: []\nstory_list: []\n",
    );
    let mut ctx = p.ctx();
    let err =
        gate::require(&mut ctx, "release", "v1.0.0").expect_err("stories[] names no sequence");
    assert_eq!(err.code(), "DFA-E321");
}

#[test]
fn require_release_with_an_entry_lacking_id_is_e321() {
    let p = Project::new();
    p.write(
        ".devforgeai/releases/v1.0.0.yaml",
        "schema: devforgeai/release/1\nid: v1.0.0\nphase: release\nstatus: released\nproduced_by: releasing-software\nconsumes: []\nopen_questions: []\nstories:\n  - title: no id here\n",
    );
    let mut ctx = p.ctx();
    let err = gate::require(&mut ctx, "release", "v1.0.0").expect_err("an entry carries no id");
    assert_eq!(err.code(), "DFA-E321");
}

// ------------------------------------------- the id shape each arm accepts

#[test]
fn require_constitute_with_a_story_id_is_e013() {
    let p = Project::new();
    let mut ctx = p.ctx();
    let err =
        gate::require(&mut ctx, "constitute", "STORY-001").expect_err("constitute keys on IDEA");
    assert_eq!(err.code(), "DFA-E013");
    assert_eq!(err.exit(), 3);
}

#[test]
fn require_verify_with_an_idea_id_is_e013() {
    let p = Project::new();
    let mut ctx = p.ctx();
    let err = gate::require(&mut ctx, "verify", "IDEA-001").expect_err("verify keys on STORY");
    assert_eq!(err.code(), "DFA-E013");
    assert_eq!(err.exit(), 3);
}

#[test]
fn require_plan_with_a_story_id_is_e013() {
    let p = Project::new();
    let mut ctx = p.ctx();
    let err =
        gate::require(&mut ctx, "plan", "STORY-001").expect_err("plan keys on EPIC or SPRINT");
    assert_eq!(err.code(), "DFA-E013");
}

#[test]
fn require_release_with_an_idea_id_is_e013() {
    let p = Project::new();
    let mut ctx = p.ctx();
    let err =
        gate::require(&mut ctx, "release", "IDEA-001").expect_err("release keys on a version");
    assert_eq!(err.code(), "DFA-E013");
}

#[test]
fn each_arm_refuses_an_id_of_another_arms_shape() {
    // The per-arm table, exhaustively: a wrong-prefix id gives DFA-E013 at
    // exit 3 before any predecessor is resolved, rather than a DFA-E321
    // naming a report path that could never exist.
    let cases: &[(&str, &str)] = &[
        ("explore", "STORY-001"),
        ("explore", "EPIC-001"),
        ("discover", "SPRINT-001"),
        ("constitute", "UI-001"),
        ("plan", "IDEA-001"),
        ("build", "EPIC-001"),
        ("verify", "SPRINT-001"),
        ("release", "STORY-001"),
    ];
    for (phase, id) in cases {
        let p = Project::new();
        let mut ctx = p.ctx();
        let err = match gate::require(&mut ctx, phase, id) {
            Ok(o) => panic!("{phase} takes only its own id shape, not {id}: {o:?}"),
            Err(e) => e,
        };
        assert_eq!(err.code(), "DFA-E013", "{phase} with {id}");
        assert_eq!(err.exit(), 3, "{phase} with {id}");
    }
}

#[test]
fn the_reflect_arm_fixes_no_id_shape() {
    // Reflect returns before the id is used, and the spec's generic grammar
    // and its per-arm table disagree about which shape it takes, so the arm
    // imposes none.
    for id in ["2026-09-11", "IDEA-003", "v1.0.0"] {
        let p = Project::new();
        let mut ctx = p.ctx();
        let out = gate::require(&mut ctx, "reflect", id).expect("reflect reads no id");
        assert_eq!(out.data["requires"], "");
    }
}

#[test]
fn the_design_arm_is_refused_before_its_id_shape_is_read() {
    // Design holds no gate, so `gate_for` refuses it with DFA-E300 whatever
    // the id carries; the UI row of the table records the shape a design
    // subject would have.
    let p = Project::new();
    for id in ["UI-002", "STORY-001"] {
        let mut ctx = p.ctx();
        let err = gate::require(&mut ctx, "design", id).expect_err("Design holds no gate");
        assert_eq!(err.code(), "DFA-E300");
    }
}

#[test]
fn require_design_gives_e300() {
    let p = Project::new();
    let mut ctx = p.ctx();
    let err = gate::require(&mut ctx, "design", "UI-002").expect_err("Design holds no gate");
    assert_eq!(err.code(), "DFA-E300");
    assert_eq!(err.exit(), 1);
}

#[test]
fn require_unknown_phase_gives_e012() {
    let p = Project::new();
    let mut ctx = p.ctx();
    let err = gate::require(&mut ctx, "invent", "IDEA-003").expect_err("outside the enum");
    assert_eq!(err.code(), "DFA-E012");
    assert_eq!(err.exit(), 3);
}

#[test]
fn require_malformed_id_gives_e013() {
    let p = Project::new();
    let mut ctx = p.ctx();
    let err = gate::require(&mut ctx, "explore", "IDEA-3").expect_err("not three digits");
    assert_eq!(err.code(), "DFA-E013");
    assert_eq!(err.exit(), 3);
}

#[test]
fn require_writes_nothing() {
    let p = Project::new();
    p.write(
        ".devforgeai/reports/IDEA-003-discover.yaml",
        &common::report("IDEA-003", "discover", "PASS"),
    );
    let before_state = p.read(".devforgeai/state.toml");
    let before_report = p.read(".devforgeai/reports/IDEA-003-discover.yaml");

    let mut ctx = p.ctx();
    gate::require(&mut ctx, "constitute", "IDEA-003").expect("PASS");

    assert_eq!(p.read(".devforgeai/state.toml"), before_state);
    assert_eq!(
        p.read(".devforgeai/reports/IDEA-003-discover.yaml"),
        before_report
    );
}
