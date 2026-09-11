//! `report aggregate`: the `devforgeai/aggregate/1` object and the session
//! reader.
//!
//! `--session-root` points at a fixture directory inside a temporary home, so
//! no test reads `~/.claude/projects`.

mod common;

use assert_cmd::Command;
use common::Project;

/// A temporary home, so the session-root containment rule resolves against a
/// fixture rather than the real home directory.
struct Home(tempfile::TempDir);

impl Home {
    fn new() -> Home {
        Home(tempfile::tempdir().expect("temp home"))
    }
    fn path(&self) -> &std::path::Path {
        self.0.path()
    }
}

fn cli(p: &Project, h: &Home) -> Command {
    let mut c = Command::cargo_bin("devforgeai").expect("the binary builds");
    c.env("DEVFORGEAI_HOME", h.path());
    c.env("HOME", h.path());
    c.env("USERPROFILE", h.path());
    c.arg("--project").arg(p.root());
    c.current_dir(p.root());
    c
}

fn run(p: &Project, h: &Home, args: &[&str]) -> (i32, String, String) {
    let out = cli(p, h).args(args).output().expect("run");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

fn json(p: &Project, h: &Home, args: &[&str]) -> serde_json::Value {
    let out = cli(p, h).arg("--json").args(args).output().expect("run");
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    serde_json::from_str(text.trim()).unwrap_or_else(|e| panic!("{e} in {text}"))
}

/// A gate report with timestamps, one failing check, and a send-back.
fn report(id: &str, phase: &str, result: &str, started: &str, finished: &str) -> String {
    let status = match result {
        "PASS" => "pass",
        "SEND_BACK" => "send_back",
        _ => "fail",
    };
    let send_back_to = if result == "SEND_BACK" { "build" } else { "" };
    format!(
        "schema: devforgeai/report/1\nid: {id}\nphase: {phase}\nstatus: {status}\nproduced_by: devforgeai-cli\nconsumes: []\nopen_questions: []\nstarted_at: {started}\nfinished_at: {finished}\ncli_version: 1.0.0\ndegraded: false\ngate:\n  result: {result}\n  send_back_to: '{send_back_to}'\n  checks:\n    - id: build-tests\n      kind: tests_pass\n      status: {}\n      severity: block\n      reason: '{}'\n      evidence:\n        duration_ms: 41203\nfindings:\n  - id: FIND-003\n    severity: block\n    summary: AC-003 has no test\nhandoff: []\n",
        if result == "PASS" { "pass" } else { "fail" },
        if result == "PASS" {
            ""
        } else {
            "DFA-E311 test command failed"
        }
    )
}

fn seeded() -> Project {
    let p = Project::new();
    p.write(
        ".devforgeai/reports/STORY-014-build.yaml",
        &report(
            "STORY-014",
            "build",
            "PASS",
            "2026-09-03T10:58:11Z",
            "2026-09-03T11:02:41Z",
        ),
    );
    p.write(
        ".devforgeai/reports/STORY-014-verify.yaml",
        &report(
            "STORY-014",
            "verify",
            "SEND_BACK",
            "2026-09-03T11:30:00Z",
            "2026-09-03T11:41:09Z",
        ),
    );
    p.write(
        ".devforgeai/reports/STORY-015-build.yaml",
        &report(
            "STORY-015",
            "build",
            "FAIL",
            "2026-09-04T09:00:00Z",
            "2026-09-04T09:05:00Z",
        ),
    );
    p
}

// --------------------------------------------------------------- the window

#[test]
fn neither_an_id_nor_since_is_e430_exit_three() {
    let p = seeded();
    let h = Home::new();
    let (code, _, err) = run(&p, &h, &["report", "aggregate"]);
    assert_eq!(code, 3, "stderr was {err}");
    assert!(err.contains("DFA-E430"), "stderr was {err}");
    assert!(err.contains("pass one id"), "stderr was {err}");
}

#[test]
fn both_an_id_and_since_is_e430_exit_three() {
    let p = seeded();
    let h = Home::new();
    let (code, _, err) = run(
        &p,
        &h,
        &["report", "aggregate", "STORY-014", "--since", "2026-09-01"],
    );
    assert_eq!(code, 3, "stderr was {err}");
    assert!(err.contains("DFA-E430"), "stderr was {err}");
}

#[test]
fn an_id_window_holds_only_that_subjects_reports() {
    let p = seeded();
    let h = Home::new();
    let v = json(&p, &h, &["report", "aggregate", "STORY-014"]);
    assert_eq!(v["data"]["schema"], "devforgeai/aggregate/1");
    assert_eq!(v["data"]["window"]["mode"], "id");
    assert_eq!(v["data"]["window"]["subject"], "STORY-014");
    assert_eq!(v["data"]["counts"]["reports"], 2);
}

#[test]
fn a_since_window_holds_reports_at_or_after_the_date() {
    let p = seeded();
    let h = Home::new();
    let v = json(&p, &h, &["report", "aggregate", "--since", "2026-09-04"]);
    assert_eq!(v["data"]["window"]["mode"], "since");
    assert_eq!(v["data"]["window"]["from"], "2026-09-04");
    assert_eq!(v["data"]["counts"]["reports"], 1);
    assert_eq!(v["data"]["reports"][0]["id"], "STORY-015");
}

#[test]
fn a_version_window_pulls_in_the_release_story_reports() {
    let p = seeded();
    let h = Home::new();
    p.write(
        ".devforgeai/releases/v0.3.0.yaml",
        "schema: devforgeai/release/1\nid: v0.3.0\nphase: release\nstatus: released\nproduced_by: releasing-software\nconsumes: []\nopen_questions: []\nstories:\n  - id: STORY-014\n    title: Order checkout\n",
    );
    let v = json(&p, &h, &["report", "aggregate", "v0.3.0"]);
    assert_eq!(v["data"]["counts"]["reports"], 2, "both STORY-014 reports");
}

#[test]
fn an_epic_window_pulls_in_the_stories_that_consume_it() {
    let p = seeded();
    let h = Home::new();
    p.write(
        ".devforgeai/stories/STORY-015.md",
        &common::story("STORY-015", "built", &["EPIC-001"], "# Tax\n"),
    );
    let v = json(&p, &h, &["report", "aggregate", "EPIC-001"]);
    assert_eq!(v["data"]["counts"]["reports"], 1);
    assert_eq!(v["data"]["reports"][0]["id"], "STORY-015");
}

#[test]
fn a_malformed_subject_is_refused_at_exit_three() {
    let p = seeded();
    let h = Home::new();
    let (code, _, err) = run(&p, &h, &["report", "aggregate", "REQ-001"]);
    assert_eq!(code, 3, "stderr was {err}");
    assert!(err.contains("DFA-E013"), "stderr was {err}");
}

// ------------------------------------------------------------ the data shape

#[test]
fn every_key_of_the_schema_is_present_on_every_run() {
    let p = Project::new();
    let h = Home::new();
    let v = json(&p, &h, &["report", "aggregate", "STORY-999"]);
    let d = &v["data"];
    for key in [
        "schema",
        "window",
        "reports",
        "phase_time",
        "gate_failures",
        "send_backs",
        "verifier_failures",
        "deferrals",
        "sessions",
        "state",
        "floors",
        "counts",
    ] {
        assert!(!d[key].is_null(), "{key} is present: {d}");
    }
    for key in [
        "reports",
        "phase_time",
        "gate_failures",
        "send_backs",
        "verifier_failures",
        "deferrals",
    ] {
        assert_eq!(
            d[key],
            serde_json::json!([]),
            "{key} is an empty array, not an absent key"
        );
    }
    for key in [
        "reports",
        "sessions",
        "gate_failures",
        "send_backs",
        "deferrals",
        "verifier_failures",
    ] {
        assert_eq!(d["counts"][key], 0, "counts.{key} is a zero");
    }
}

#[test]
fn a_report_entry_carries_its_duration_checks_and_findings() {
    let p = seeded();
    let h = Home::new();
    let v = json(&p, &h, &["report", "aggregate", "STORY-014"]);
    let r = v["data"]["reports"]
        .as_array()
        .expect("reports")
        .iter()
        .find(|r| r["phase"] == "build")
        .expect("the build report");
    assert_eq!(r["path"], ".devforgeai/reports/STORY-014-build.yaml");
    assert_eq!(r["status"], "pass");
    assert_eq!(r["duration_ms"], 270000);
    assert_eq!(r["gate"]["result"], "PASS");
    assert_eq!(r["gate"]["failed_checks"], serde_json::json!([]));
    assert_eq!(r["checks"][0]["id"], "build-tests");
    assert_eq!(r["checks"][0]["kind"], "tests_pass");
    assert_eq!(r["checks"][0]["duration_ms"], 41203);
    assert_eq!(r["findings"][0]["id"], "FIND-003");
}

#[test]
fn phase_time_groups_the_runs_per_phase() {
    let p = seeded();
    let h = Home::new();
    let v = json(&p, &h, &["report", "aggregate", "--since", "2026-01-01"]);
    let build = v["data"]["phase_time"]
        .as_array()
        .expect("phase_time")
        .iter()
        .find(|e| e["phase"] == "build")
        .expect("the build row");
    assert_eq!(build["runs"], 2);
    assert_eq!(build["total_ms"], 270000 + 300000);
    assert_eq!(build["first_at"], "2026-09-03T11:02:41Z");
    assert_eq!(build["last_at"], "2026-09-04T09:05:00Z");
}

#[test]
fn gate_failures_group_by_check_id() {
    let p = seeded();
    let h = Home::new();
    let v = json(&p, &h, &["report", "aggregate", "--since", "2026-01-01"]);
    let f = &v["data"]["gate_failures"][0];
    assert_eq!(f["check_id"], "build-tests");
    assert_eq!(f["kind"], "tests_pass");
    assert_eq!(f["count"], 2, "the verify send-back and the build failure");
    assert!(
        f["reasons"][0]
            .as_str()
            .expect("reason")
            .contains("DFA-E311"),
        "{f}"
    );
}

#[test]
fn send_backs_carry_the_from_to_pair_and_its_ids() {
    let p = seeded();
    let h = Home::new();
    let v = json(&p, &h, &["report", "aggregate", "STORY-014"]);
    let s = &v["data"]["send_backs"][0];
    assert_eq!(s["from"], "verify");
    assert_eq!(s["to"], "build");
    assert_eq!(s["id"], "STORY-014");
    assert_eq!(s["count"], 1);
    assert_eq!(s["at"][0], "2026-09-03T11:41:09Z");
    assert_eq!(s["check_ids"][0], "build-tests");
    assert_eq!(s["finding_ids"][0], "FIND-003");
}

#[test]
fn the_floors_block_carries_the_compiled_minimums() {
    let p = seeded();
    let h = Home::new();
    let v = json(&p, &h, &["report", "aggregate", "STORY-014"]);
    let f = &v["data"]["floors"];
    assert_eq!(f["config.toml"]["layer.domain.coverage_min"], 90.0);
    assert_eq!(f["config.toml"]["coverage.overall_min"], 75.0);
    assert_eq!(f["gates.toml"]["verifier_pass.min_ratio"], 1.0);
}

#[test]
fn the_state_block_carries_current_active_and_last_gate() {
    let p = seeded();
    let h = Home::new();
    p.set_phase("build", "STORY-014");
    let v = json(&p, &h, &["report", "aggregate", "STORY-014"]);
    assert_eq!(v["data"]["state"]["current_phase"], "build");
    assert_eq!(v["data"]["state"]["active"]["build"], "STORY-014");
    assert!(v["data"]["state"]["last_gate"].is_object());
    assert!(v["data"]["state"]["last_handoff_at"].is_string());
}

// ------------------------------------------------------------- the deferrals

#[test]
fn deferrals_are_renamed_from_the_qa_report_entries() {
    let p = seeded();
    let h = Home::new();
    p.write(
        ".devforgeai/reports/STORY-009-qa.yaml",
        "schema: devforgeai/qa-report/1\nid: STORY-009\nphase: verify\nstatus: pass\nproduced_by: validating-quality\nconsumes: []\nopen_questions: []\nstarted_at: 2026-09-03T10:00:00Z\nfinished_at: 2026-09-03T10:10:00Z\ngate:\n  result: PASS\n  send_back_to: ''\n  checks: []\nfindings: []\nhandoff: []\ndeferrals:\n  - id: FIND-012\n    story: STORY-009\n    dod_item: Integration test for the retry path\n    target: build\n    reason: Retry backoff is unspecified until ADR-011 is accepted\n    opened_on: '2026-08-01'\n    con_or_ap: CON-002\n",
    );
    let v = common::with_now("2026-09-11T09:14:02Z", || {
        json(&p, &h, &["report", "aggregate", "STORY-009"])
    });
    let d = &v["data"]["deferrals"][0];
    assert_eq!(d["id"], "FIND-012");
    assert_eq!(d["story"], "STORY-009");
    assert_eq!(d["dod_item"], "Integration test for the retry path");
    assert_eq!(d["deferred_at"], "2026-08-01");
    assert_eq!(d["age_days"], 41);
    assert_eq!(d["constraint"], "CON-002");
    assert_eq!(
        d["reason"],
        "Retry backoff is unspecified until ADR-011 is accepted"
    );
    assert_eq!(d["report"], ".devforgeai/reports/STORY-009-qa.yaml");
    assert!(d.get("target").is_none(), "target is not carried through");
}

#[test]
fn a_deferral_with_no_opened_on_takes_an_empty_date_and_zero_days() {
    let p = seeded();
    let h = Home::new();
    p.write(
        ".devforgeai/reports/STORY-009-qa.yaml",
        "schema: devforgeai/qa-report/1\nid: STORY-009\nphase: verify\nstatus: pass\nproduced_by: validating-quality\nconsumes: []\nopen_questions: []\nstarted_at: 2026-09-03T10:00:00Z\nfinished_at: 2026-09-03T10:10:00Z\ngate:\n  result: PASS\n  send_back_to: ''\n  checks: []\nfindings: []\nhandoff: []\ndeferrals:\n  - id: FIND-013\n    story: STORY-009\n    dod_item: A note\n    reason: unspecified\n    con_or_ap: ''\n",
    );
    let v = json(&p, &h, &["report", "aggregate", "STORY-009"]);
    let d = &v["data"]["deferrals"][0];
    assert_eq!(d["deferred_at"], "");
    assert_eq!(d["age_days"], 0);
    assert_eq!(d["constraint"], "");
}

// --------------------------------------------------------------- the sessions

/// The project key of a root, as the spec defines it.
fn key_of(p: &Project) -> String {
    devforgeai::aggregate::project_key(p.root())
}

#[test]
fn an_absent_session_directory_is_status_absent_at_exit_zero() {
    let p = seeded();
    let h = Home::new();
    let root = h.path().join("projects");
    std::fs::create_dir_all(&root).expect("mkdir");
    let (code, _, err) = run(
        &p,
        &h,
        &[
            "report",
            "aggregate",
            "STORY-014",
            "--session-root",
            &root.display().to_string(),
        ],
    );
    assert_eq!(code, 0, "stderr was {err}");
    let v = json(
        &p,
        &h,
        &[
            "report",
            "aggregate",
            "STORY-014",
            "--session-root",
            &root.display().to_string(),
        ],
    );
    assert_eq!(v["data"]["sessions"]["status"], "absent");
    assert_eq!(v["data"]["sessions"]["key"], key_of(&p));
}

#[test]
fn an_empty_session_directory_is_status_empty() {
    let p = seeded();
    let h = Home::new();
    let root = h.path().join("projects");
    std::fs::create_dir_all(root.join(key_of(&p))).expect("mkdir");
    let v = json(
        &p,
        &h,
        &[
            "report",
            "aggregate",
            "STORY-014",
            "--session-root",
            &root.display().to_string(),
        ],
    );
    assert_eq!(v["data"]["sessions"]["status"], "empty");
}

#[test]
fn a_session_file_yields_its_commands_and_repeats() {
    let p = seeded();
    let h = Home::new();
    let root = h.path().join("projects");
    let dir = root.join(key_of(&p));
    std::fs::create_dir_all(&dir).expect("mkdir");
    let lines = [
        r#"{"type":"user","timestamp":"2026-09-01T09:12:00Z","message":{"content":"/explore IDEA-001 --remedy FLOW-002"}}"#,
        r#"{"type":"assistant","message":{"content":"working"}}"#,
        r#"{"type":"user","isMeta":true,"message":{"content":"/explore IDEA-001"}}"#,
        r#"{"type":"user","timestamp":"2026-09-01T10:00:00Z","message":{"content":"/explore IDEA-001 --remedy FLOW-002"}}"#,
        r#"{"type":"user","timestamp":"2026-09-01T11:00:00Z","message":{"content":"/build STORY-014 --resume"}}"#,
        r#"{"type":"user","timestamp":"2026-09-01T11:30:00Z","message":{"content":"please continue"}}"#,
    ];
    std::fs::write(dir.join("session-a.jsonl"), lines.join("\n")).expect("write");

    let v = json(
        &p,
        &h,
        &[
            "report",
            "aggregate",
            "STORY-014",
            "--session-root",
            &root.display().to_string(),
        ],
    );
    let s = &v["data"]["sessions"];
    assert_eq!(s["status"], "present");
    assert_eq!(s["files"].as_array().expect("files").len(), 1);
    assert_eq!(s["files"][0]["session_id"], "session-a");
    assert_eq!(s["files"][0]["lines"], 6);
    assert_eq!(s["files"][0]["from"], "2026-09-01T09:12:00Z");
    assert_eq!(s["files"][0]["to"], "2026-09-01T11:30:00Z");

    let commands = s["commands"].as_array().expect("commands");
    assert_eq!(
        commands.len(),
        3,
        "the assistant line, the meta line, and the prose are skipped: {s}"
    );
    assert_eq!(commands[0]["command"], "/explore");
    assert_eq!(commands[0]["args"], "IDEA-001 --remedy FLOW-002");
    assert_eq!(commands[0]["line"], 1);
    assert_eq!(commands[0]["remedy_ids"], serde_json::json!(["FLOW-002"]));
    assert_eq!(commands[0]["resume"], false);
    assert_eq!(commands[2]["command"], "/build");
    assert_eq!(commands[2]["resume"], true);

    let repeats = s["repeats"].as_array().expect("repeats");
    assert_eq!(repeats.len(), 1, "only groups of two or more: {s}");
    assert_eq!(repeats[0]["command"], "/explore");
    assert_eq!(repeats[0]["key"], "IDEA-001|FLOW-002");
    assert_eq!(repeats[0]["count"], 2);
    assert_eq!(repeats[0]["lines"], serde_json::json!([1, 4]));
    assert_eq!(v["data"]["counts"]["sessions"], 1);
}

#[test]
fn a_session_root_outside_the_home_directory_is_e421_exit_one() {
    let p = seeded();
    let h = Home::new();
    let outside = tempfile::tempdir().expect("temp");
    let (code, _, err) = run(
        &p,
        &h,
        &[
            "report",
            "aggregate",
            "STORY-014",
            "--session-root",
            &outside.path().display().to_string(),
        ],
    );
    assert_eq!(code, 1, "stderr was {err}");
    assert!(err.contains("DFA-E421"), "stderr was {err}");

    let v = json(
        &p,
        &h,
        &[
            "report",
            "aggregate",
            "STORY-014",
            "--session-root",
            &outside.path().display().to_string(),
        ],
    );
    assert_eq!(v["data"]["sessions"]["status"], "unreadable");
    assert_eq!(
        v["data"]["counts"]["reports"], 2,
        "the rest of the aggregate is still produced"
    );
}

#[test]
fn the_human_output_is_one_line_per_section_with_its_count() {
    let p = seeded();
    let h = Home::new();
    let (code, out, _) = run(&p, &h, &["report", "aggregate", "STORY-014"]);
    assert_eq!(code, 0);
    assert!(out.contains("reports            2"), "stdout was {out}");
    assert!(out.contains("send_backs         1"), "stdout was {out}");
}

#[test]
fn aggregate_over_only_unparsable_reports_still_answers() {
    let p = Project::new();
    p.write(
        ".devforgeai/reports/IDEA-004-explore.yaml",
        "{{{ not yaml
",
    );

    let mut ctx = p.ctx();
    let out = devforgeai::cmd::report::aggregate(&mut ctx, Some("IDEA-004"), None, None)
        .expect("an unparsable entry is excluded, not fatal");

    // The window is empty and says so, with the file named. Refusing outright
    // would make the command unusable on a directory holding any stray YAML.
    assert_eq!(out.exit.unwrap_or(0), 0);
    assert!(out.errors.is_empty(), "{:?}", out.errors);
    assert!(out.warnings.iter().any(|d| d.code == "DFA-W420"));
    assert_eq!(out.data["reports"].as_array().expect("reports").len(), 0);
}

#[test]
fn aggregate_over_an_unparsable_report_warns_and_keeps_the_rest() {
    let p = Project::new();
    p.write(
        ".devforgeai/reports/IDEA-004-explore.yaml",
        &common::report("IDEA-004", "explore", "PASS"),
    );
    p.write(".devforgeai/reports/scratch.yaml", "{{{ not yaml\n");

    let mut ctx = p.ctx();
    let out = devforgeai::cmd::report::aggregate(&mut ctx, Some("IDEA-004"), None, None)
        .expect("one stray file does not refuse the whole window");

    // `errors[]` non-empty means `ok: false` under the envelope contract, so
    // an excluded entry is a warning: the window is still answerable and the
    // file is still named.
    assert_eq!(out.exit.unwrap_or(0), 0);
    assert!(out.errors.is_empty(), "{:?}", out.errors);
    let w = out
        .warnings
        .iter()
        .find(|d| d.code == "DFA-W420")
        .expect("the excluded entry is reported");
    assert!(w.message.contains("scratch.yaml"), "{}", w.message);
    assert_eq!(out.data["reports"].as_array().expect("reports").len(), 1);
}

#[test]
fn aggregate_with_an_absent_relative_session_root_reports_absent() {
    let p = Project::new();
    p.write(
        ".devforgeai/reports/IDEA-004-explore.yaml",
        &common::report("IDEA-004", "explore", "PASS"),
    );
    let mut cfg = devforgeai::config::load(p.root()).expect("config");
    // A relative root that was never created: `normalise` joins it onto the
    // working directory, which is not under the home, so it used to read as
    // "outside the home directory" rather than as simply absent.
    cfg.reflect.session_root = "sessions".into();
    p.write_config(&cfg);

    let mut ctx = p.ctx();
    let out = devforgeai::cmd::report::aggregate(&mut ctx, Some("IDEA-004"), None, None)
        .expect("an absent session root is not a failure");

    assert_eq!(out.exit.unwrap_or(0), 0);
    assert_eq!(out.data["sessions"]["status"], "absent");
    assert!(
        !out.warnings.iter().any(|d| d.code == "DFA-E421"),
        "absence is not a containment refusal: {:?}",
        out.warnings
    );
}
