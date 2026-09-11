//! `explore prune --id <IDEA-nnn>`: remove `.explore-prototype/` after a kill
//! or a promote decision, leave it in place after a park.

mod common;

use assert_cmd::Command;
use common::Project;

fn cli(p: &Project) -> Command {
    let mut c = Command::cargo_bin("devforgeai").expect("the binary builds");
    c.arg("--project").arg(p.root());
    c.current_dir(p.root());
    c
}

/// Lay down a prototype directory with `n` files across two levels.
fn prototype(p: &Project, n: usize) {
    for i in 0..n {
        let rel = if i % 2 == 0 {
            format!(".explore-prototype/f{i}.txt")
        } else {
            format!(".explore-prototype/nested/f{i}.txt")
        };
        p.write(&rel, "x\n");
    }
}

fn run(p: &Project, id: &str) -> (i32, String, String) {
    let out = cli(p)
        .args(["explore", "prune", "--id", id])
        .output()
        .expect("run");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

#[test]
fn a_promote_decision_removes_the_prototype_directory() {
    let p = Project::new();
    p.write(
        ".devforgeai/explore/decision.yaml",
        &common::decision("IDEA-003", "promote"),
    );
    prototype(&p, 14);

    let (code, out, err) = run(&p, "IDEA-003");
    assert_eq!(code, 0, "stderr was {err}");
    assert!(!p.exists(".explore-prototype"), "the directory is gone");
    assert!(
        out.contains(".explore-prototype") && out.contains("14 files") && out.contains("promote"),
        "stdout was {out}"
    );
}

#[test]
fn a_kill_decision_removes_the_prototype_directory() {
    let p = Project::new();
    p.write(
        ".devforgeai/explore/decision.yaml",
        &common::decision("IDEA-003", "kill"),
    );
    prototype(&p, 3);

    let (code, _, err) = run(&p, "IDEA-003");
    assert_eq!(code, 0, "stderr was {err}");
    assert!(!p.exists(".explore-prototype"));
}

#[test]
fn a_park_decision_leaves_the_directory_in_place() {
    let p = Project::new();
    p.write(
        ".devforgeai/explore/decision.yaml",
        &common::decision("IDEA-003", "park"),
    );
    prototype(&p, 4);

    let (code, _, err) = run(&p, "IDEA-003");
    assert_eq!(code, 0, "stderr was {err}");
    assert!(p.exists(".explore-prototype"), "park keeps the prototype");
}

#[test]
fn an_absent_directory_is_success() {
    let p = Project::new();
    p.write(
        ".devforgeai/explore/decision.yaml",
        &common::decision("IDEA-003", "kill"),
    );
    let (code, _, err) = run(&p, "IDEA-003");
    assert_eq!(code, 0, "stderr was {err}");
}

#[test]
fn an_absent_decision_file_is_e200_exit_one() {
    let p = Project::new();
    let (code, _, err) = run(&p, "IDEA-003");
    assert_eq!(code, 1, "stderr was {err}");
    assert!(err.contains("DFA-E200"), "stderr was {err}");
}

#[test]
fn an_unparsable_decision_file_is_e401_exit_one() {
    let p = Project::new();
    p.write(".devforgeai/explore/decision.yaml", "decision: [unclosed\n");
    let (code, _, err) = run(&p, "IDEA-003");
    assert_eq!(code, 1, "stderr was {err}");
    assert!(err.contains("DFA-E401"), "stderr was {err}");
}

#[test]
fn a_malformed_id_is_e013_exit_three() {
    let p = Project::new();
    let (code, _, err) = run(&p, "IDEA-3");
    assert_eq!(code, 3, "stderr was {err}");
    assert!(err.contains("DFA-E013"), "stderr was {err}");
}

#[test]
fn the_json_data_object_is_the_shape_the_spec_fixes() {
    let p = Project::new();
    p.write(
        ".devforgeai/explore/decision.yaml",
        &common::decision("IDEA-001", "promote"),
    );
    prototype(&p, 14);

    let out = cli(&p)
        .args(["--json", "explore", "prune", "--id", "IDEA-001"])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let v: serde_json::Value = serde_json::from_str(text.trim()).expect("json");
    assert_eq!(v["command"], "explore prune");
    assert_eq!(v["data"]["id"], "IDEA-001");
    assert_eq!(v["data"]["decision"], "promote");
    assert_eq!(v["data"]["removed"], true);
    assert_eq!(v["data"]["path"], ".explore-prototype");
    assert_eq!(v["data"]["files"], 14);
}

#[test]
fn a_park_decision_reports_removed_false() {
    let p = Project::new();
    p.write(
        ".devforgeai/explore/decision.yaml",
        &common::decision("IDEA-001", "park"),
    );
    prototype(&p, 2);

    let out = cli(&p)
        .args(["--json", "explore", "prune", "--id", "IDEA-001"])
        .output()
        .expect("run");
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let v: serde_json::Value = serde_json::from_str(text.trim()).expect("json");
    assert_eq!(v["data"]["removed"], false);
    assert_eq!(v["data"]["files"], 2, "the count is what would have gone");
}
