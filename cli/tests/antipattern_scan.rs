//! `antipattern scan` and the `antipattern_clean` check kind.

mod common;

use assert_cmd::Command;
use common::Project;

fn cli(p: &Project) -> Command {
    let mut c = Command::cargo_bin("devforgeai").expect("the binary builds");
    c.arg("--project").arg(p.root());
    c.current_dir(p.root());
    c
}

fn run(p: &Project, args: &[&str]) -> (i32, String, String) {
    let out = cli(p).args(args).output().expect("run");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

fn json(p: &Project, args: &[&str]) -> serde_json::Value {
    let out = cli(p).arg("--json").args(args).output().expect("run");
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    serde_json::from_str(text.trim()).unwrap_or_else(|e| panic!("{e} in {text}"))
}

/// An `anti-patterns.md` whose index carries the given rows.
fn index(rows: &str) -> String {
    common::context_file(
        "anti-patterns",
        "accepted",
        &[(
            "## Anti-pattern index",
            &format!(
                "| AP | Category | Severity | Scope | Detector kind | Detector | Source |\n|---|---|---|---|---|---|---|\n{rows}"
            ),
        )],
    )
}

/// A project with a source tree and one anti-pattern rule.
fn seeded(rows: &str) -> Project {
    let p = Project::new();
    p.write(".devforgeai/context/anti-patterns.md", &index(rows));
    p.write(
        "src/place_order.rs",
        "fn place() {\n    TcpStream::connect();\n}\n",
    );
    p.write("src/tax.rs", "fn tax() -> u32 { 0 }\n");
    p
}

#[test]
fn a_literal_rule_matches_and_names_the_line() {
    let p = seeded("| AP-001 | layer | high | src/**/*.rs | literal | TcpStream | CON-001 |\n");
    let (code, out, err) = run(&p, &["antipattern", "scan"]);
    assert_eq!(code, 1, "stdout {out} stderr {err}");
    assert!(err.contains("DFA-E270"), "stderr was {err}");
    assert!(err.contains("AP-001"), "stderr was {err}");
    assert!(err.contains("src/place_order.rs:2"), "stderr was {err}");
    assert!(err.contains("TcpStream"), "stderr was {err}");
}

#[test]
fn a_clean_tree_exits_zero() {
    let p = seeded("| AP-001 | layer | high | src/**/*.rs | literal | UdpSocket | CON-001 |\n");
    let (code, out, err) = run(&p, &["antipattern", "scan"]);
    assert_eq!(code, 0, "stderr was {err}");
    assert!(out.contains("0 matches"), "stdout was {out}");
}

#[test]
fn the_json_data_object_is_the_shape_the_spec_fixes() {
    let p = seeded("| AP-002 | layer | high | src/**/*.rs | literal | TcpStream | CON-001 |\n");
    let v = json(&p, &["antipattern", "scan"]);
    assert_eq!(v["exit"], 1);
    assert_eq!(v["data"]["scanned"], 2, "both source files");
    assert_eq!(v["data"]["rules"], 1);
    let m = &v["data"]["matches"][0];
    assert_eq!(m["id"], "AP-002");
    assert_eq!(m["severity"], "high");
    assert_eq!(m["path"], "src/place_order.rs");
    assert_eq!(m["line"], 2);
    assert!(
        m["text"].as_str().expect("text").contains("TcpStream"),
        "text was {}",
        m["text"]
    );
}

#[test]
fn a_regex_rule_uses_the_regex_crate_dialect() {
    let p = seeded(
        r"| AP-003 | smell | high | src/**/*.rs | regex | Tcp[A-Z][a-z]+ | CON-001 |
",
    );
    let (code, _, err) = run(&p, &["antipattern", "scan"]);
    assert_eq!(code, 1, "stderr was {err}");
    assert!(err.contains("AP-003"), "stderr was {err}");
}

#[test]
fn a_glob_rule_matches_the_path_and_reads_no_file() {
    let p = seeded("| AP-004 | structure | high | src/** | glob | src/**/tax.rs | CON-001 |\n");
    let v = json(&p, &["antipattern", "scan"]);
    assert_eq!(v["data"]["matches"][0]["path"], "src/tax.rs");
    assert_eq!(v["data"]["matches"][0]["line"], 0);
}

#[test]
fn the_scope_glob_narrows_the_candidate_set() {
    let p = seeded("| AP-001 | layer | high | src/tax.rs | literal | TcpStream | CON-001 |\n");
    let (code, _, err) = run(&p, &["antipattern", "scan"]);
    assert_eq!(code, 0, "the matching file is outside the scope; {err}");
}

#[test]
fn the_min_severity_filter_drops_a_lower_rule() {
    let p = seeded("| AP-001 | layer | medium | src/**/*.rs | literal | TcpStream | CON-001 |\n");
    let (code, _, _) = run(&p, &["antipattern", "scan"]);
    assert_eq!(code, 0, "medium is below the default high");

    let (code, _, err) = run(&p, &["antipattern", "scan", "--min-severity", "medium"]);
    assert_eq!(code, 1, "stderr was {err}");
    assert_eq!(
        json(&p, &["antipattern", "scan", "--min-severity", "medium"])["data"]["rules"],
        1
    );
    assert_eq!(
        json(&p, &["antipattern", "scan"])["data"]["rules"],
        0,
        "a filtered rule is not counted"
    );
}

#[test]
fn a_blocker_is_reported_under_every_floor() {
    let p =
        seeded("| AP-001 | security | blocker | src/**/*.rs | literal | TcpStream | CON-001 |\n");
    for floor in ["blocker", "high", "medium", "low"] {
        let (code, _, err) = run(&p, &["antipattern", "scan", "--min-severity", floor]);
        assert_eq!(code, 1, "{floor}: stderr was {err}");
    }
}

#[test]
fn an_unknown_severity_is_e013_exit_three() {
    let p = seeded("| AP-001 | layer | high | src/**/*.rs | literal | TcpStream | CON-001 |\n");
    let (code, _, err) = run(&p, &["antipattern", "scan", "--min-severity", "urgent"]);
    assert_eq!(code, 3, "stderr was {err}");
    assert!(err.contains("DFA-E013"), "stderr was {err}");
}

#[test]
fn the_paths_flag_replaces_the_candidate_set() {
    let p = seeded("| AP-001 | layer | high | src/**/*.rs | literal | TcpStream | CON-001 |\n");
    let v = json(&p, &["antipattern", "scan", "--paths", "src/tax.rs"]);
    assert_eq!(v["data"]["scanned"], 1);
    assert_eq!(v["exit"], 0);
}

#[test]
fn the_id_flag_takes_the_story_file_set() {
    let p = seeded("| AP-001 | layer | high | src/**/*.rs | literal | TcpStream | CON-001 |\n");
    p.write(
        ".devforgeai/stories/STORY-014.md",
        "---\nschema: devforgeai/story/1\nid: STORY-014\nphase: plan\nstatus: building\nproduced_by: planning-work\nconsumes: []\nopen_questions: []\n---\n\n# T\n\n## Files\n\n| Path | Kind | Layer |\n|---|---|---|\n| src/tax.rs | source | application |\n",
    );
    let v = json(&p, &["antipattern", "scan", "--id", "STORY-014"]);
    assert_eq!(v["data"]["scanned"], 1);
    assert_eq!(v["exit"], 0, "the offending file is not in the story");
}

#[test]
fn an_absent_story_is_e200_exit_one() {
    let p = seeded("| AP-001 | layer | high | src/**/*.rs | literal | TcpStream | CON-001 |\n");
    let (code, _, err) = run(&p, &["antipattern", "scan", "--id", "STORY-999"]);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E200"), "stderr was {err}");
}

#[test]
fn an_absent_index_file_is_a_clean_scan() {
    let p = Project::new();
    p.write("src/a.rs", "fn a() {}\n");
    let (code, _, err) = run(&p, &["antipattern", "scan"]);
    assert_eq!(code, 0, "stderr was {err}");
    assert_eq!(json(&p, &["antipattern", "scan"])["data"]["rules"], 0);
}

// ------------------------------------------------ the antipattern_clean kind

/// The explore gate baseline plus one `antipattern_clean` check. The baseline
/// carries the two kinds the explore phase requires, so the gate is complete
/// and the added check is the only thing under test.
fn gates(scope: &str) -> String {
    format!(
        r#"schema = "devforgeai/gates/1"
cli_min_version = "1.0.0"

[[gate]]
phase = "explore"
requires = ""
on_fail = "fail"

  [[gate.check]]
  kind = "file_exists"
  id = "decision-exists"
  paths = ["explore/decision.yaml"]

  [[gate.check]]
  kind = "field_in_enum"
  id = "decision-enum"
  path = "explore/decision.yaml"
  field = "decision"
  values = ["kill", "park", "promote"]

  [[gate.check]]
  kind = "antipattern_clean"
  id = "no-antipatterns"
  scope = "{scope}"
"#
    )
}

#[test]
fn the_check_passes_over_a_clean_story_set() {
    let p = seeded(
        "| AP-001 | layer | high | src/**/*.rs | literal | TcpStream | CON-001 |
",
    );
    p.write(
        ".devforgeai/explore/decision.yaml",
        &common::decision("IDEA-003", "promote"),
    );
    p.write(
        ".devforgeai/stories/STORY-014.md",
        "---
schema: devforgeai/story/1
id: STORY-014
phase: plan
status: building
produced_by: planning-work
consumes: []
open_questions: []
---

# T

## Files

| Path | Kind | Layer |
|---|---|---|
| src/tax.rs | source | application |
",
    );
    p.write(".devforgeai/gates.toml", &gates("story"));
    let out = cli(&p)
        .args(["gate", "check", "--phase", "explore", "--id", "STORY-014"])
        .output()
        .expect("run");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn the_check_fails_over_a_project_scope_match() {
    let p = seeded(
        "| AP-001 | layer | high | src/**/*.rs | literal | TcpStream | CON-001 |
",
    );
    p.write(
        ".devforgeai/explore/decision.yaml",
        &common::decision("IDEA-003", "promote"),
    );
    p.write(".devforgeai/gates.toml", &gates("project"));
    let out = cli(&p)
        .args(["gate", "check", "--phase", "explore", "--id", "IDEA-003"])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(1));
    let report = p.read(".devforgeai/reports/IDEA-003-explore.yaml");
    assert!(report.contains("DFA-E270"), "report was {report}");
    assert!(report.contains("AP-001"), "report was {report}");
}

#[test]
fn the_check_skips_when_the_story_scope_has_no_story_subject() {
    let p = seeded(
        "| AP-001 | layer | high | src/**/*.rs | literal | TcpStream | CON-001 |
",
    );
    p.write(
        ".devforgeai/explore/decision.yaml",
        &common::decision("IDEA-003", "promote"),
    );
    p.write(".devforgeai/gates.toml", &gates("story"));
    let out = cli(&p)
        .args(["gate", "check", "--phase", "explore", "--id", "IDEA-003"])
        .output()
        .expect("run");
    // `story` scope with no story subject has nothing to scan, so it cannot
    // report a clean tree. A skip here is a pass the phase did not earn; the
    // check fails with the code that names the missing subject.
    assert_eq!(out.status.code(), Some(1));
    let report = p.read(".devforgeai/reports/IDEA-003-explore.yaml");
    assert!(report.contains("DFA-E270"), "report was {report}");
    assert!(
        report.contains("is not a story"),
        "the reason names why there is no file set: {report}"
    );
}

/// Register a worktree for `STORY-014` and make it the active build story.
fn register_worktree(p: &Project, dir: &str) {
    std::fs::create_dir_all(p.root().join(dir)).expect("mkdir");
    let mut s = p.state();
    s.current.phase = "build".to_string();
    s.current.id = "STORY-014".to_string();
    s.active.build = "STORY-014".to_string();
    s.worktree = vec![devforgeai::state::WorktreeEntry {
        story: "STORY-014".to_string(),
        path: dir.to_string(),
        branch: "story/STORY-014".to_string(),
        created_at: "2026-09-10T14:02:11Z".to_string(),
    }];
    p.write_state(&s);
}

#[test]
fn scan_reads_the_worktree_during_a_registered_build() {
    let p = Project::new();
    // The rules are a context document and stay at the root.
    p.write(
        ".devforgeai/context/anti-patterns.md",
        &index("| AP-001 | layer | high | src/**/*.rs | literal | TcpStream | CON-001 |\n"),
    );
    register_worktree(&p, "wt");
    // The offending source is in the worktree, where a Build run's work is.
    p.write(
        "wt/src/place_order.rs",
        "fn place() {\n    TcpStream::connect();\n}\n",
    );

    let (code, _, err) = run(
        &p,
        &["antipattern", "scan", "--paths", "src/place_order.rs"],
    );

    assert_eq!(code, 1, "the match is found: {err}");
    assert!(err.contains("DFA-E270"), "stderr was {err}");
    assert!(
        err.contains("place_order.rs"),
        "the worktree's file was read: {err}"
    );
}

#[test]
fn scan_finds_nothing_when_only_the_root_holds_the_match() {
    let p = Project::new();
    p.write(
        ".devforgeai/context/anti-patterns.md",
        &index("| AP-001 | layer | high | src/**/*.rs | literal | TcpStream | CON-001 |\n"),
    );
    register_worktree(&p, "wt");
    // A stale copy at the root is not the build's source, so it is not scanned.
    p.write(
        "src/place_order.rs",
        "fn place() {\n    TcpStream::connect();\n}\n",
    );

    let (code, _, err) = run(
        &p,
        &["antipattern", "scan", "--paths", "src/place_order.rs"],
    );
    assert_eq!(code, 0, "stderr was {err}");
}
