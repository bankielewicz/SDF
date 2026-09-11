//! `story validate`, `story list`, and `story files`.
//!
//! Every fixture is a temporary project. The `--diff` tests create a real git
//! repository inside that temporary directory and pass identity on the command
//! line, so no test reads or writes the machine's git configuration.

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

/// A story with one requirement, two ACs, one file, and one dependency.
fn story(id: &str, status: &str) -> String {
    format!(
        "---\nschema: devforgeai/story/1\nid: {id}\nphase: plan\nstatus: {status}\nproduced_by: planning-work\nconsumes: [REQ-001]\nopen_questions: []\n---\n\n# Order checkout\n\n## Requirements\n\n| REQ | Statement | Covered by |\n|---|---|---|\n| REQ-001 | The shopper places an order | AC-001 AC-002 |\n\n## Acceptance Criteria\n\n- AC-001: Given a cart When the shopper pays Then an order row exists.\n- AC-002: The endpoint returns 201.\n\n## Files\n\n| Path | Kind | Layer |\n|---|---|---|\n| src/application/place_order.rs | source | application |\n| tests/place_order.rs | test | application |\n\n## Dependencies\n\nnone\n"
    )
}

/// A project holding one valid story, its requirements, and an active build.
fn seeded() -> Project {
    let p = Project::new();
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-001", "accepted"),
    );
    p.write(
        ".devforgeai/stories/STORY-014.md",
        &story("STORY-014", "ready"),
    );
    p.set_phase("build", "STORY-014");
    p
}

// ---------------------------------------------------------- story validate

#[test]
fn a_valid_story_passes_every_check() {
    let p = seeded();
    let (code, out, err) = run(&p, &["story", "validate"]);
    assert_eq!(code, 0, "stderr was {err}");
    assert_eq!(out, "ok    STORY-014  2 ACs  0 dependencies\n");
}

#[test]
fn the_json_data_object_is_the_shape_the_spec_fixes() {
    let p = seeded();
    let v = json(&p, &["story", "validate"]);
    assert_eq!(v["data"]["scope"], "active");
    assert_eq!(v["data"]["checked"], 1);
    assert_eq!(v["data"]["failed"], 0);
    assert_eq!(v["data"]["stories"][0]["id"], "STORY-014");
    assert_eq!(v["data"]["stories"][0]["valid"], true);
    assert_eq!(v["data"]["stories"][0]["acs"], 2);
    assert_eq!(v["data"]["stories"][0]["deps"], serde_json::json!([]));
    assert_eq!(v["data"]["stories"][0]["errors"], serde_json::json!([]));
}

#[test]
fn check_2_refuses_a_story_with_no_acceptance_criterion() {
    let p = seeded();
    let text = p
        .read(".devforgeai/stories/STORY-014.md")
        .replace(
            "- AC-001: Given a cart When the shopper pays Then an order row exists.\n",
            "",
        )
        .replace("- AC-002: The endpoint returns 201.\n", "");
    p.write(".devforgeai/stories/STORY-014.md", &text);
    let (code, _, err) = run(&p, &["story", "validate"]);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E234"), "stderr was {err}");
    assert!(err.contains("defines no acceptance criteria"), "{err}");
}

#[test]
fn check_3_refuses_an_untestable_acceptance_criterion() {
    let p = seeded();
    let text = p.read(".devforgeai/stories/STORY-014.md").replace(
        "- AC-002: The endpoint returns 201.",
        "- AC-002: It is good.",
    );
    p.write(".devforgeai/stories/STORY-014.md", &text);
    let (code, _, err) = run(&p, &["story", "validate"]);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E230"), "stderr was {err}");
    assert!(err.contains("AC-002"), "stderr was {err}");
}

#[test]
fn check_3_accepts_either_testable_form() {
    let p = seeded();
    for text in [
        "- AC-002: Given a cart When the tax runs Then the total rises.",
        "- AC-002: The API rejects a blank name.",
        "- AC-002: The job emits one event per order.",
    ] {
        let body = p
            .read(".devforgeai/stories/STORY-014.md")
            .replace("- AC-002: The endpoint returns 201.", text);
        p.write(".devforgeai/stories/STORY-014.md", &body);
        let (code, _, err) = run(&p, &["story", "validate"]);
        assert_eq!(code, 0, "{text}: stderr was {err}");
        p.write(
            ".devforgeai/stories/STORY-014.md",
            &story("STORY-014", "ready"),
        );
    }
}

#[test]
fn check_4_refuses_an_unresolved_requirement_reference() {
    let p = seeded();
    let text = p
        .read(".devforgeai/stories/STORY-014.md")
        .replace("consumes: [REQ-001]", "consumes: [REQ-404]")
        .replace("| REQ-001 |", "| REQ-404 |");
    p.write(".devforgeai/stories/STORY-014.md", &text);
    let (code, _, err) = run(&p, &["story", "validate"]);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E231"), "stderr was {err}");
    assert!(err.contains("REQ-404"), "stderr was {err}");
}

#[test]
fn check_5_reports_a_dependency_cycle_as_its_path() {
    let p = seeded();
    let a = story("STORY-014", "ready").replace(
        "## Dependencies\n\nnone\n",
        "## Dependencies\n\n- STORY-015: the cart\n",
    );
    let b = story("STORY-015", "ready").replace(
        "## Dependencies\n\nnone\n",
        "## Dependencies\n\n- STORY-014: the order\n",
    );
    p.write(".devforgeai/stories/STORY-014.md", &a);
    p.write(".devforgeai/stories/STORY-015.md", &b);
    let (code, _, err) = run(&p, &["story", "validate", "--scope", "all"]);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E232"), "stderr was {err}");
    assert!(
        err.contains("STORY-014 -> STORY-015 -> STORY-014"),
        "stderr was {err}"
    );
}

#[test]
fn check_6_refuses_a_sprint_entry_with_no_file() {
    let p = seeded();
    p.write(
        ".devforgeai/stories/sprint.yaml",
        "schema: devforgeai/sprint/1\nid: SPRINT-001\nphase: plan\nstatus: active\nproduced_by: planning-work\nconsumes: [EPIC-001]\nopen_questions: []\nepic: EPIC-001\ncapacity:\n  points_max: 20\n  points_planned: 3\n  source: default\nstories:\n  - id: STORY-014\n    status: ready\n    order: 1\n    points: 3\n  - id: STORY-099\n    status: ready\n    order: 2\n    points: 3\ndeferred: []\n",
    );
    let (code, _, err) = run(&p, &["story", "validate", "--scope", "sprint"]);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E233"), "stderr was {err}");
    assert!(err.contains("STORY-099"), "stderr was {err}");
}

#[test]
fn check_7_refuses_an_epic_requirement_no_story_carries() {
    let p = seeded();
    // The epic lists REQ-001 and REQ-002; the one story covers REQ-001 alone.
    p.write(
        ".devforgeai/stories/sprint.yaml",
        "schema: devforgeai/sprint/1\nid: SPRINT-001\nphase: plan\nstatus: active\nproduced_by: planning-work\nconsumes: [EPIC-001]\nopen_questions: []\nepic: EPIC-001\ncapacity:\n  points_max: 20\n  points_planned: 3\n  source: default\nstories:\n  - id: STORY-014\n    status: ready\n    order: 1\n    points: 3\ndeferred: []\n",
    );
    let (code, _, err) = run(&p, &["story", "validate", "--scope", "sprint"]);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E235"), "stderr was {err}");
    assert!(err.contains("REQ-002"), "stderr was {err}");
}

#[test]
fn check_8_refuses_an_ac_in_no_covered_by_cell() {
    let p = seeded();
    let text = p.read(".devforgeai/stories/STORY-014.md").replace(
        "| REQ-001 | The shopper places an order | AC-001 AC-002 |",
        "| REQ-001 | The shopper places an order | AC-001 |",
    );
    p.write(".devforgeai/stories/STORY-014.md", &text);
    let (code, _, err) = run(&p, &["story", "validate"]);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E236"), "stderr was {err}");
    assert!(err.contains("AC-002"), "stderr was {err}");
}

#[test]
fn check_8_refuses_an_ac_in_two_covered_by_cells() {
    let p = seeded();
    let text = p.read(".devforgeai/stories/STORY-014.md").replace(
        "| REQ-001 | The shopper places an order | AC-001 AC-002 |",
        "| REQ-001 | The shopper places an order | AC-001 AC-002 |\n| REQ-002 | The shopper sees a receipt | AC-002 |",
    ).replace("consumes: [REQ-001]", "consumes: [REQ-001, REQ-002]");
    p.write(".devforgeai/stories/STORY-014.md", &text);
    let (code, _, err) = run(&p, &["story", "validate"]);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E245"), "stderr was {err}");
}

#[test]
fn check_9_refuses_a_path_two_concurrent_stories_declare() {
    let p = seeded();
    p.write(
        ".devforgeai/stories/STORY-015.md",
        &story("STORY-015", "ready"),
    );
    p.write(
        ".devforgeai/stories/sprint.yaml",
        "schema: devforgeai/sprint/1\nid: SPRINT-001\nphase: plan\nstatus: active\nproduced_by: planning-work\nconsumes: [EPIC-001]\nopen_questions: []\nepic: EPIC-001\ncapacity:\n  points_max: 20\n  points_planned: 6\n  source: default\nstories:\n  - id: STORY-014\n    status: ready\n    order: 1\n    points: 3\n  - id: STORY-015\n    status: ready\n    order: 2\n    points: 3\ndeferred: []\n",
    );
    let (code, _, err) = run(&p, &["story", "validate", "--scope", "sprint"]);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E237"), "stderr was {err}");
    assert!(
        err.contains("src/application/place_order.rs"),
        "stderr was {err}"
    );
}

#[test]
fn check_10_refuses_a_ui_reference_with_no_spec_file() {
    let p = seeded();
    let text = p
        .read(".devforgeai/stories/STORY-014.md")
        .replace("consumes: [REQ-001]", "consumes: [REQ-001, UI-004]");
    p.write(".devforgeai/stories/STORY-014.md", &text);
    let (code, _, err) = run(&p, &["story", "validate"]);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E238"), "stderr was {err}");

    p.write(".devforgeai/ui-specs/UI-004.md", "# a screen\n");
    let (code, _, err) = run(&p, &["story", "validate"]);
    assert_eq!(code, 0, "the spec file resolves it; stderr was {err}");
}

#[test]
fn an_explicit_id_ignores_the_scope() {
    let p = seeded();
    p.write(
        ".devforgeai/stories/STORY-015.md",
        &story("STORY-015", "draft"),
    );
    let v = json(&p, &["story", "validate", "STORY-015", "--scope", "all"]);
    assert_eq!(v["data"]["checked"], 1);
    assert_eq!(v["data"]["stories"][0]["id"], "STORY-015");
}

#[test]
fn an_unknown_scope_is_e012_exit_three() {
    let p = seeded();
    let (code, _, err) = run(&p, &["story", "validate", "--scope", "everything"]);
    assert_eq!(code, 3, "stderr was {err}");
    assert!(err.contains("DFA-E012"), "stderr was {err}");
}

#[test]
fn a_malformed_id_is_e013_exit_three() {
    let p = seeded();
    let (code, _, err) = run(&p, &["story", "validate", "STORY-14"]);
    assert_eq!(code, 3);
    assert!(err.contains("DFA-E013"), "stderr was {err}");
}

#[test]
fn an_absent_story_is_e200_exit_one() {
    let p = seeded();
    let (code, _, err) = run(&p, &["story", "validate", "STORY-999"]);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E200"), "stderr was {err}");
}

// -------------------------------------------------------------- story list

#[test]
fn story_list_prints_one_line_per_story() {
    let p = seeded();
    p.write(
        ".devforgeai/stories/STORY-015.md",
        &story("STORY-015", "built"),
    );
    let (code, out, err) = run(&p, &["story", "list"]);
    assert_eq!(code, 0, "stderr was {err}");
    assert_eq!(
        out,
        "STORY-014  ready  Order checkout\nSTORY-015  built  Order checkout\n"
    );
}

#[test]
fn story_list_json_carries_the_count_and_the_consumes_list() {
    let p = seeded();
    let v = json(&p, &["story", "list"]);
    assert_eq!(v["data"]["count"], 1);
    assert_eq!(v["data"]["stories"][0]["id"], "STORY-014");
    assert_eq!(v["data"]["stories"][0]["status"], "ready");
    assert_eq!(v["data"]["stories"][0]["title"], "Order checkout");
    assert_eq!(
        v["data"]["stories"][0]["path"],
        ".devforgeai/stories/STORY-014.md"
    );
    assert_eq!(
        v["data"]["stories"][0]["consumes"],
        serde_json::json!(["REQ-001"])
    );
}

#[test]
fn story_list_filters_by_status() {
    let p = seeded();
    p.write(
        ".devforgeai/stories/STORY-015.md",
        &story("STORY-015", "built"),
    );
    let v = json(&p, &["story", "list", "--status", "built"]);
    assert_eq!(v["data"]["count"], 1);
    assert_eq!(v["data"]["stories"][0]["id"], "STORY-015");

    let v = json(&p, &["story", "list", "--status", "ready,built"]);
    assert_eq!(v["data"]["count"], 2);
}

#[test]
fn story_list_filters_by_sprint() {
    let p = seeded();
    p.write(
        ".devforgeai/stories/STORY-015.md",
        &story("STORY-015", "built"),
    );
    p.write(
        ".devforgeai/stories/sprint.yaml",
        "schema: devforgeai/sprint/1\nid: SPRINT-001\nphase: plan\nstatus: active\nproduced_by: planning-work\nconsumes: [EPIC-001]\nopen_questions: []\nepic: EPIC-001\ncapacity:\n  points_max: 20\n  points_planned: 3\n  source: default\nstories:\n  - id: STORY-015\n    status: built\n    order: 1\n    points: 3\ndeferred: []\n",
    );
    let v = json(&p, &["story", "list", "--sprint", "SPRINT-001"]);
    assert_eq!(v["data"]["count"], 1);
    assert_eq!(v["data"]["stories"][0]["id"], "STORY-015");
}

#[test]
fn story_list_with_no_story_is_a_count_of_zero_at_exit_zero() {
    let p = Project::new();
    let (code, out, _) = run(&p, &["story", "list"]);
    assert_eq!(code, 0);
    assert!(out.is_empty(), "stdout was {out}");
    assert_eq!(json(&p, &["story", "list"])["data"]["count"], 0);
}

#[test]
fn an_unknown_status_value_is_refused_at_exit_three() {
    let p = seeded();
    let (code, _, err) = run(&p, &["story", "list", "--status", "invented"]);
    assert_eq!(code, 3, "stderr was {err}");
    assert!(err.contains("DFA-E012"), "stderr was {err}");
}

#[test]
fn an_absent_stories_directory_is_e231_exit_one() {
    let p = Project::bare();
    std::fs::create_dir_all(p.dot()).expect("mkdir");
    let (code, _, err) = run(&p, &["story", "list"]);
    assert_eq!(code, 1, "stderr was {err}");
    assert!(err.contains("DFA-E231"), "stderr was {err}");
}

// ------------------------------------------------------------- story files

#[test]
fn story_files_list_prints_the_triples() {
    let p = seeded();
    let (code, out, err) = run(&p, &["story", "files", "--list"]);
    assert_eq!(code, 0, "stderr was {err}");
    assert_eq!(
        out,
        "src/application/place_order.rs  source  application\ntests/place_order.rs  test  application\n"
    );
}

#[test]
fn story_files_check_allows_a_declared_path() {
    let p = seeded();
    let (code, _, err) = run(
        &p,
        &[
            "story",
            "files",
            "--check",
            "src/application/place_order.rs",
        ],
    );
    assert_eq!(code, 0, "stderr was {err}");

    let v = json(
        &p,
        &[
            "story",
            "files",
            "--check",
            "src/application/place_order.rs",
        ],
    );
    assert_eq!(v["data"]["id"], "STORY-014");
    assert_eq!(v["data"]["path"], "src/application/place_order.rs");
    assert_eq!(v["data"]["allowed"], true);
    assert_eq!(v["data"]["declared"], 2);
}

#[test]
fn story_files_check_refuses_an_undeclared_path_with_e239() {
    let p = seeded();
    let (code, _, err) = run(&p, &["story", "files", "--check", "src/other.rs"]);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E239"), "stderr was {err}");
    assert!(
        err.contains("outside the declared file set of STORY-014"),
        "stderr was {err}"
    );
}

#[test]
fn story_files_check_takes_an_absolute_path_as_repo_relative() {
    let p = seeded();
    let abs = p
        .root()
        .join("src")
        .join("application")
        .join("place_order.rs");
    let (code, _, err) = run(
        &p,
        &["story", "files", "--check", &abs.display().to_string()],
    );
    assert_eq!(code, 0, "stderr was {err}");
}

#[test]
fn story_files_needs_exactly_one_mode() {
    let p = seeded();
    for args in [
        vec!["story", "files"],
        vec!["story", "files", "--list", "--diff"],
    ] {
        let (code, _, err) = run(&p, &args);
        assert_eq!(code, 3, "{args:?}: stderr was {err}");
        assert!(err.contains("DFA-E011"), "{args:?}: stderr was {err}");
    }
}

#[test]
fn an_absent_active_build_is_e412_at_exit_zero() {
    let p = seeded();
    p.set_phase("plan", "SPRINT-001");
    let mut s = p.state();
    s.active.build = String::new();
    p.write_state(&s);

    let (code, _, err) = run(&p, &["story", "files", "--check", "src/anything.rs"]);
    assert_eq!(
        code, 0,
        "the PreToolUse hook must not block; stderr was {err}"
    );
    assert!(err.contains("DFA-E412"), "stderr was {err}");
}

// ----------------------------------------------------------- story files --diff

/// A real git repository inside the temporary project, with identity passed on
/// the command line so no machine configuration is read or written.
fn git(p: &Project, args: &[&str]) -> std::process::Output {
    let mut c = std::process::Command::new("git");
    c.arg("-C").arg(p.root());
    c.args([
        "-c",
        "user.name=devforgeai test",
        "-c",
        "user.email=test@example.invalid",
        "-c",
        "commit.gpgsign=false",
    ]);
    c.args(args);
    c.output().expect("git runs")
}

fn init_repo(p: &Project) {
    let out = std::process::Command::new("git")
        .arg("init")
        .arg("-b")
        .arg("main")
        .arg(p.root())
        .output()
        .expect("git init");
    assert!(out.status.success(), "git init: {out:?}");
    p.write(".gitignore", ".devforgeai/reports/\n");
    git(p, &["add", "-A"]);
    let out = git(p, &["commit", "-m", "base"]);
    assert!(out.status.success(), "first commit: {out:?}");
}

#[test]
fn story_files_diff_is_clean_when_every_change_is_declared() {
    let p = seeded();
    init_repo(&p);
    p.write("src/application/place_order.rs", "fn place() {}\n");
    p.write("tests/place_order.rs", "fn t() {}\n");

    let (code, out, err) = run(&p, &["story", "files", "--diff"]);
    assert_eq!(code, 0, "stdout {out} stderr {err}");
}

#[test]
fn story_files_diff_names_each_undeclared_path() {
    let p = seeded();
    init_repo(&p);
    p.write("src/application/place_order.rs", "fn place() {}\n");
    p.write("src/sneaky.rs", "fn sneak() {}\n");

    let (code, _, err) = run(&p, &["story", "files", "--diff"]);
    assert_eq!(code, 1, "stderr was {err}");
    assert!(err.contains("DFA-E239"), "stderr was {err}");
    assert!(err.contains("src/sneaky.rs"), "stderr was {err}");
    assert!(
        !err.contains("src/application/place_order.rs"),
        "the declared path is not reported; stderr was {err}"
    );

    let v = json(&p, &["story", "files", "--diff"]);
    assert_eq!(v["data"]["id"], "STORY-014");
    assert_eq!(v["data"]["undeclared"], 1);
    let paths = v["data"]["paths"].as_array().expect("paths");
    let sneaky = paths
        .iter()
        .find(|x| x["path"] == "src/sneaky.rs")
        .expect("the undeclared path");
    assert_eq!(sneaky["declared"], false);
    assert_eq!(sneaky["status"], "untracked");
}

#[test]
fn story_files_diff_reads_committed_changes_against_the_base() {
    let p = seeded();
    init_repo(&p);
    p.write("src/sneaky.rs", "fn sneak() {}\n");
    git(&p, &["add", "-A"]);
    let out = git(&p, &["commit", "-m", "STORY-014 sneak"]);
    assert!(out.status.success(), "{out:?}");

    let base = String::from_utf8_lossy(&git(&p, &["rev-parse", "HEAD~1"]).stdout)
        .trim()
        .to_string();
    let (code, _, err) = run(&p, &["story", "files", "--diff", "--base", &base]);
    assert_eq!(code, 1, "stderr was {err}");
    assert!(err.contains("src/sneaky.rs"), "stderr was {err}");
}

#[test]
fn story_files_diff_outside_a_repository_is_e271() {
    let p = seeded();
    let (code, _, err) = run(&p, &["story", "files", "--diff"]);
    assert_eq!(code, 1, "stderr was {err}");
    assert!(err.contains("DFA-E271"), "stderr was {err}");
}

// ---------------------------------------------------------- the story_valid kind

#[test]
fn the_story_valid_check_passes_over_a_valid_story() {
    let p = seeded();
    p.write(
        ".devforgeai/gates.toml",
        r#"schema = "devforgeai/gates/1"
cli_min_version = "1.0.0"

[[gate]]
phase = "plan"
requires = ""
on_fail = "fail"

  [[gate.check]]
  kind = "doc_valid"
  id = "plan-docs"
  docs = ["stories/STORY-014.md"]

  [[gate.check]]
  kind = "ids_resolve"
  id = "plan-ids"
  prefixes = ["REQ"]

  [[gate.check]]
  kind = "story_valid"
  id = "stories-valid"
  scope = "all"
"#,
    );
    let out = cli(&p)
        .args(["gate", "check", "--phase", "plan", "--id", "STORY-014"])
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
fn the_story_valid_check_fails_with_e328() {
    let p = seeded();
    let text = p.read(".devforgeai/stories/STORY-014.md").replace(
        "- AC-002: The endpoint returns 201.",
        "- AC-002: It is good.",
    );
    p.write(".devforgeai/stories/STORY-014.md", &text);
    p.write(
        ".devforgeai/gates.toml",
        r#"schema = "devforgeai/gates/1"
cli_min_version = "1.0.0"

[[gate]]
phase = "plan"
requires = ""
on_fail = "fail"

  [[gate.check]]
  kind = "doc_valid"
  id = "plan-docs"
  docs = ["stories/STORY-014.md"]

  [[gate.check]]
  kind = "ids_resolve"
  id = "plan-ids"
  prefixes = ["REQ"]

  [[gate.check]]
  kind = "story_valid"
  id = "stories-valid"
  scope = "all"
"#,
    );
    let out = cli(&p)
        .args(["gate", "check", "--phase", "plan", "--id", "STORY-014"])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(1));
    let report = p.read(".devforgeai/reports/STORY-014-plan.yaml");
    assert!(report.contains("DFA-E328"), "report was {report}");
}

#[test]
fn story_validate_refuses_a_bad_scope_even_with_an_id() {
    let p = Project::new();
    // The scope was validated only on the `None` arm, so `--scope bogus --id
    // STORY-001` was accepted in silence and the caller believed a scope had
    // been honoured that the command never read.
    let mut ctx = p.ctx();
    let err = devforgeai::cmd::story::validate(&mut ctx, Some("STORY-001"), "bogus")
        .expect_err("the scope is validated whether or not an id narrows the run");
    assert_eq!(err.code(), "DFA-E012");
    assert_eq!(err.exit(), 3);
    let msg = &err.diag().expect("diag").message;
    assert!(msg.contains("bogus"), "{msg}");
}
