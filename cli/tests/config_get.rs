//! `config get <key> [--stack <id>]`: one configuration value on stdout.
//!
//! Every fixture lives in its own temporary project; no test reads the real
//! home directory.

mod common;

use assert_cmd::Command;
use common::Project;

fn cli(p: &Project) -> Command {
    let mut c = Command::cargo_bin("devforgeai").expect("the binary builds");
    c.arg("--project").arg(p.root());
    c.current_dir(p.root());
    c
}

/// A project whose `config.toml` carries two stacks and non-default tables.
fn project() -> Project {
    let p = Project::new();
    let mut cfg = devforgeai::config::Config {
        degraded: false,
        generated_at: "2026-09-10T14:02:11Z".into(),
        ..Default::default()
    };
    cfg.stack = vec![
        devforgeai::config::Stack {
            id: "rust".into(),
            markers: vec!["Cargo.toml".into()],
            package_manager: "cargo".into(),
            source_roots: vec!["src".into(), "crates".into()],
            test_command: "cargo test --all-features".into(),
            coverage_command: "cargo llvm-cov --lcov".into(),
            lint_command: "cargo clippy".into(),
            complexity_command: String::new(),
            coverage_format: "lcov".into(),
            timeout_secs: 900,
            ..Default::default()
        },
        devforgeai::config::Stack {
            id: "node".into(),
            markers: vec!["package.json".into()],
            package_manager: "npm".into(),
            source_roots: vec!["web/src".into()],
            test_command: "npm test".into(),
            coverage_format: "lcov".into(),
            timeout_secs: 900,
            ..Default::default()
        },
    ];
    p.write_config(&cfg);
    p
}

fn stdout_of(p: &Project, args: &[&str]) -> (i32, String, String) {
    let out = cli(p).args(args).output().expect("run");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

fn json_of(p: &Project, args: &[&str]) -> serde_json::Value {
    let out = cli(p).arg("--json").args(args).output().expect("run");
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    serde_json::from_str(text.trim()).unwrap_or_else(|e| panic!("{e} in {text}"))
}

// ------------------------------------------------------------------ stack.*

#[test]
fn a_stack_string_key_takes_the_first_stack_table() {
    let p = project();
    let (code, out, err) = stdout_of(&p, &["config", "get", "stack.test_command"]);
    assert_eq!(code, 0, "stderr was {err}");
    assert_eq!(out, "cargo test --all-features\n");
}

#[test]
fn the_stack_flag_selects_by_id() {
    let p = project();
    let (code, out, _) = stdout_of(
        &p,
        &["config", "get", "stack.test_command", "--stack", "node"],
    );
    assert_eq!(code, 0);
    assert_eq!(out, "npm test\n");
}

#[test]
fn every_stack_key_resolves() {
    let p = project();
    for (key, want) in [
        ("stack.test_command", "cargo test --all-features\n"),
        ("stack.coverage_command", "cargo llvm-cov --lcov\n"),
        ("stack.lint_command", "cargo clippy\n"),
        ("stack.complexity_command", "\n"),
        ("stack.source_roots", "src\ncrates\n"),
        ("stack.package_manager", "cargo\n"),
    ] {
        let (code, out, err) = stdout_of(&p, &["config", "get", key]);
        assert_eq!(code, 0, "{key}: stderr was {err}");
        assert_eq!(out, want, "{key}");
    }
}

#[test]
fn an_empty_string_prints_an_empty_line_and_exits_zero() {
    let p = project();
    let (code, out, _) = stdout_of(&p, &["config", "get", "stack.complexity_command"]);
    assert_eq!(code, 0);
    assert_eq!(out, "\n");
}

#[test]
fn an_array_prints_one_element_per_line() {
    let p = project();
    let (code, out, _) = stdout_of(&p, &["config", "get", "stack.source_roots"]);
    assert_eq!(code, 0);
    assert_eq!(out, "src\ncrates\n");
}

#[test]
fn an_unmatched_stack_id_is_e013_exit_three() {
    let p = project();
    let (code, _, err) = stdout_of(
        &p,
        &["config", "get", "stack.test_command", "--stack", "ruby"],
    );
    assert_eq!(code, 3, "stderr was {err}");
    assert!(err.contains("DFA-E013"), "stderr was {err}");
    assert!(err.contains("ruby"), "stderr was {err}");
}

// ---------------------------------------------------------- the other tables

#[test]
fn every_non_stack_key_resolves() {
    let p = project();
    for (key, want) in [
        ("build.worktree_root", "../wt\n"),
        ("build.branch_prefix", "story/\n"),
        ("build.base_ref", "HEAD\n"),
        ("build.complexity_max", "10\n"),
        ("build.duplication_max_percent", "5.0\n"),
        ("coverage.overall_min", "80.0\n"),
        ("layer.domain.coverage_min", "95.0\n"),
        ("layer.application.coverage_min", "85.0\n"),
        ("layer.infrastructure.coverage_min", "80.0\n"),
        ("layer.interface.coverage_min", "70.0\n"),
        ("frontend.tokens_path", ".devforgeai/brand/tokens.json\n"),
        ("degraded", "false\n"),
    ] {
        let (code, out, err) = stdout_of(&p, &["config", "get", key]);
        assert_eq!(code, 0, "{key}: stderr was {err}");
        assert_eq!(out, want, "{key}");
    }
}

#[test]
fn frontend_globs_prints_one_glob_per_line() {
    let p = project();
    let (code, out, _) = stdout_of(&p, &["config", "get", "frontend.globs"]);
    assert_eq!(code, 0);
    assert!(!out.is_empty());
    let cfg = devforgeai::config::load(p.root()).expect("config");
    assert_eq!(out.lines().count(), cfg.frontend.globs.len());
}

#[test]
fn an_unknown_key_is_e013_exit_three() {
    let p = project();
    for key in ["stack.invented", "nope", "layer.nowhere.coverage_min"] {
        let (code, _, err) = stdout_of(&p, &["config", "get", key]);
        assert_eq!(code, 3, "{key}: stderr was {err}");
        assert!(err.contains("DFA-E013"), "{key}: stderr was {err}");
    }
}

#[test]
fn a_stack_key_with_no_stack_table_is_e013() {
    let p = Project::new();
    p.write_config(&devforgeai::config::Config {
        degraded: true,
        stack: vec![],
        generated_at: "2026-09-10T14:02:11Z".into(),
        ..Default::default()
    });
    let (code, _, err) = stdout_of(&p, &["config", "get", "stack.test_command"]);
    assert_eq!(code, 3, "stderr was {err}");
    assert!(err.contains("DFA-E013"), "stderr was {err}");
}

// -------------------------------------------------------------------- json

#[test]
fn the_json_data_object_is_the_shape_the_spec_fixes() {
    let p = project();
    let v = json_of(&p, &["config", "get", "stack.test_command"]);
    assert_eq!(v["ok"], true);
    assert_eq!(v["exit"], 0);
    assert_eq!(v["data"]["key"], "stack.test_command");
    assert_eq!(v["data"]["stack"], "rust");
    assert_eq!(v["data"]["value"], "cargo test --all-features");
    assert_eq!(v["data"]["kind"], "string");
}

#[test]
fn an_array_value_is_a_json_array_of_kind_array() {
    let p = project();
    let v = json_of(&p, &["config", "get", "stack.source_roots"]);
    assert_eq!(v["data"]["kind"], "array");
    assert_eq!(
        v["data"]["value"],
        serde_json::json!(["src", "crates"]),
        "the array goes through as an array"
    );
}

#[test]
fn a_numeric_value_is_a_json_number() {
    let p = project();
    let v = json_of(&p, &["config", "get", "build.complexity_max"]);
    assert_eq!(v["data"]["kind"], "integer");
    assert_eq!(v["data"]["value"], 10);

    let v = json_of(&p, &["config", "get", "coverage.overall_min"]);
    assert_eq!(v["data"]["kind"], "number");
    assert_eq!(v["data"]["value"], 80.0);
}

#[test]
fn degraded_is_a_json_boolean_with_an_empty_stack_field() {
    let p = project();
    let v = json_of(&p, &["config", "get", "degraded"]);
    assert_eq!(v["data"]["kind"], "boolean");
    assert_eq!(v["data"]["value"], false);
    assert_eq!(v["data"]["stack"], "", "no stack was selected");
}

#[test]
fn an_unknown_key_carries_the_code_in_the_envelope() {
    let p = project();
    let out = cli(&p)
        .args(["--json", "config", "get", "nope"])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(3));
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let v: serde_json::Value = serde_json::from_str(text.trim()).expect("json");
    assert_eq!(v["errors"][0]["code"], "DFA-E013");
    assert_eq!(v["command"], "config get");
}

#[test]
fn quiet_suppresses_the_value_but_not_the_exit_code() {
    let p = project();
    let (code, out, _) = stdout_of(&p, &["--quiet", "config", "get", "stack.test_command"]);
    assert_eq!(code, 0);
    assert!(out.is_empty(), "stdout was {out}");
}
