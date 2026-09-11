//! `design lint` and the `design_tokens` check kind.

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

/// A well-formed token file: `meta` plus the six groups.
const TOKENS: &str = r##"{
  "meta": {
    "schema": "devforgeai/tokens/1",
    "id": "TOKENS-001",
    "phase": "design",
    "status": "accepted",
    "produced_by": "designing-interfaces",
    "consumes": [],
    "open_questions": []
  },
  "color": {
    "primary": { "light": "#3355ff", "dark": "#7788ff" },
    "surface": { "light": "#ffffff", "dark": "#101010" }
  },
  "type": { "body": "1rem", "heading": "2rem" },
  "spacing": { "sm": "4px" },
  "radius": { "sm": "2px" },
  "elevation": { "low": "0 1px 2px" },
  "motion": { "fast": "120ms" }
}
"##;

fn seeded() -> Project {
    let p = Project::new();
    p.write(".devforgeai/brand/tokens.json", TOKENS);
    p
}

#[test]
fn a_clean_stylesheet_exits_zero() {
    let p = seeded();
    p.write(
        "src/ui/Button.css",
        ".b { color: var(--color-primary); font-size: var(--type-body); }\n",
    );
    let (code, out, err) = run(&p, &["design", "lint"]);
    assert_eq!(code, 0, "stderr was {err}");
    assert!(out.contains("0 violations"), "stdout was {out}");
}

#[test]
fn a_literal_colour_is_e240_naming_the_nearest_token() {
    let p = seeded();
    p.write("src/ui/Button.css", ".b { color: #3356ff; }\n");
    let (code, _, err) = run(&p, &["design", "lint"]);
    assert_eq!(code, 1, "stderr was {err}");
    assert!(err.contains("DFA-E240"), "stderr was {err}");
    assert!(err.contains("TOKEN-color-primary"), "stderr was {err}");
}

#[test]
fn a_literal_font_value_is_e241() {
    let p = seeded();
    p.write("src/ui/Button.css", ".b { font-size: 17px; }\n");
    let (code, _, err) = run(&p, &["design", "lint"]);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E241"), "stderr was {err}");
    assert!(err.contains("TOKEN-type-body"), "stderr was {err}");
}

#[test]
fn an_unresolved_custom_property_is_e242() {
    let p = seeded();
    p.write("src/ui/Button.css", ".b { color: var(--color-nowhere); }\n");
    let (code, _, err) = run(&p, &["design", "lint"]);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E242"), "stderr was {err}");
    assert!(err.contains("--color-nowhere"), "stderr was {err}");
}

#[test]
fn the_json_data_object_is_the_shape_the_spec_fixes() {
    let p = seeded();
    p.write("src/ui/Button.tsx", "const s = { color: '#3356ff' };\n");
    let v = json(&p, &["design", "lint"]);
    assert_eq!(v["data"]["mode"], "paths");
    assert_eq!(v["data"]["files"], 1);
    assert_eq!(v["data"]["skipped"], 0);
    let x = &v["data"]["violations"][0];
    assert_eq!(x["path"], "src/ui/Button.tsx");
    assert_eq!(x["line"], 1);
    assert_eq!(x["code"], "DFA-E240");
    assert_eq!(x["value"], "#3356ff");
    assert_eq!(x["nearest"], "TOKEN-color-primary");
}

#[test]
fn a_path_outside_the_globs_is_skipped_and_counted() {
    let p = seeded();
    p.write("src/ui/Button.css", ".b { color: #3356ff; }\n");
    p.write("docs/notes.md", "#3356ff\n");
    let v = json(&p, &["design", "lint", "docs/notes.md"]);
    assert_eq!(v["exit"], 0);
    assert_eq!(v["data"]["files"], 0);
    assert_eq!(v["data"]["skipped"], 1);
}

#[test]
fn the_sketch_and_prototype_prefixes_are_excluded() {
    let p = seeded();
    p.write(".explore-prototype/page.css", ".b { color: #3356ff; }\n");
    p.write(
        ".devforgeai/explore/mockups/page.css",
        ".b { color: #3356ff; }\n",
    );
    let (code, out, err) = run(&p, &["design", "lint"]);
    assert_eq!(code, 0, "stdout {out} stderr {err}");
    assert_eq!(json(&p, &["design", "lint"])["data"]["files"], 0);
}

#[test]
fn a_configured_glob_narrows_the_set() {
    let p = seeded();
    let mut cfg = devforgeai::config::load(p.root()).expect("config");
    cfg.frontend.globs = vec!["src/**/*.css".to_string()];
    p.write_config(&cfg);
    p.write("src/ui/Button.css", ".b { color: #3356ff; }\n");
    p.write("web/Button.tsx", "const s = '#3356ff';\n");
    assert_eq!(json(&p, &["design", "lint"])["data"]["files"], 1);
}

#[test]
fn a_configured_exclude_drops_a_path() {
    let p = seeded();
    let mut cfg = devforgeai::config::load(p.root()).expect("config");
    cfg.frontend.exclude.push("src/legacy/**".to_string());
    p.write_config(&cfg);
    p.write("src/legacy/Old.css", ".b { color: #3356ff; }\n");
    let (code, _, err) = run(&p, &["design", "lint"]);
    assert_eq!(code, 0, "stderr was {err}");
}

#[test]
fn an_absent_token_file_is_e120_exit_one() {
    let p = Project::new();
    let (code, _, err) = run(&p, &["design", "lint"]);
    assert_eq!(code, 1, "stderr was {err}");
    assert!(err.contains("DFA-E120"), "stderr was {err}");
}

#[test]
fn an_unparsable_token_file_is_e121_exit_one() {
    let p = Project::new();
    p.write(".devforgeai/brand/tokens.json", "{ not json\n");
    let (code, _, err) = run(&p, &["design", "lint"]);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E121"), "stderr was {err}");
}

// -------------------------------------------------------------------- --tokens

#[test]
fn tokens_mode_accepts_a_well_formed_file() {
    let p = seeded();
    let (code, out, err) = run(&p, &["design", "lint", "--tokens"]);
    assert_eq!(code, 0, "stdout {out} stderr {err}");
    assert_eq!(
        json(&p, &["design", "lint", "--tokens"])["data"]["mode"],
        "tokens"
    );
}

#[test]
fn tokens_mode_refuses_a_colour_leaf_that_is_not_light_and_dark() {
    let p = seeded();
    p.write(
        ".devforgeai/brand/tokens.json",
        &TOKENS.replace(
            r##""primary": { "light": "#3355ff", "dark": "#7788ff" }"##,
            r##""primary": "#3355ff""##,
        ),
    );
    let (code, _, err) = run(&p, &["design", "lint", "--tokens"]);
    assert_eq!(code, 1, "stderr was {err}");
    assert!(err.contains("DFA-E243"), "stderr was {err}");
    assert!(err.contains("light, dark"), "stderr was {err}");
}

#[test]
fn tokens_mode_refuses_a_non_string_leaf_outside_colour() {
    let p = seeded();
    p.write(
        ".devforgeai/brand/tokens.json",
        &TOKENS.replace(r#""sm": "4px""#, r#""sm": 4"#),
    );
    let (code, _, err) = run(&p, &["design", "lint", "--tokens"]);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E243"), "stderr was {err}");
}

#[test]
fn tokens_mode_refuses_a_leaf_name_outside_the_pattern() {
    let p = seeded();
    p.write(
        ".devforgeai/brand/tokens.json",
        &TOKENS.replace(r#""body": "1rem""#, r#""Body": "1rem""#),
    );
    let (code, _, err) = run(&p, &["design", "lint", "--tokens"]);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E243"), "stderr was {err}");
}

#[test]
fn tokens_mode_refuses_a_missing_group_and_an_unknown_top_level_key() {
    let p = seeded();
    p.write(
        ".devforgeai/brand/tokens.json",
        &TOKENS.replace(
            r#""motion": { "fast": "120ms" }"#,
            r#""mood": { "fast": "120ms" }"#,
        ),
    );
    let (code, _, err) = run(&p, &["design", "lint", "--tokens"]);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E243"), "stderr was {err}");
    assert!(err.contains("mood"), "stderr was {err}");
    assert!(err.contains("motion"), "stderr was {err}");
}

#[test]
fn tokens_mode_refuses_a_meta_block_missing_a_frontmatter_key() {
    let p = seeded();
    p.write(
        ".devforgeai/brand/tokens.json",
        &TOKENS.replace("    \"produced_by\": \"designing-interfaces\",\n", ""),
    );
    let (code, _, err) = run(&p, &["design", "lint", "--tokens"]);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E243"), "stderr was {err}");
    assert!(err.contains("produced_by"), "stderr was {err}");
}

#[test]
fn tokens_mode_refuses_an_undefined_token_in_a_ui_spec() {
    let p = seeded();
    p.write(
        ".devforgeai/ui-specs/UI-004.md",
        "# a screen\n\nThe header uses TOKEN-color-primary and TOKEN-color-nowhere.\n",
    );
    let (code, _, err) = run(&p, &["design", "lint", "--tokens"]);
    assert_eq!(code, 1, "stderr was {err}");
    assert!(err.contains("DFA-E244"), "stderr was {err}");
    assert!(err.contains("TOKEN-color-nowhere"), "stderr was {err}");
    assert!(
        !err.contains("TOKEN-color-primary'"),
        "the defined token is not reported; stderr was {err}"
    );

    let v = json(&p, &["design", "lint", "--tokens"]);
    assert_eq!(v["data"]["files"], 1, "the UI specs scanned");
}

#[test]
fn tokens_mode_with_a_path_argument_is_e010_exit_three() {
    let p = seeded();
    let (code, _, err) = run(&p, &["design", "lint", "--tokens", "src/a.css"]);
    assert_eq!(code, 3, "stderr was {err}");
    assert!(err.contains("DFA-E010"), "stderr was {err}");
}

// ------------------------------------------------- the `design_tokens` kind

fn gates(paths: &str) -> String {
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
  kind = "design_tokens"
  id = "build-design"
  paths = [{paths}]
"#
    )
}

fn gate_ready(p: &Project) {
    p.write(
        ".devforgeai/explore/decision.yaml",
        &common::decision("IDEA-003", "promote"),
    );
}

#[test]
fn the_design_tokens_check_passes_over_a_clean_tree() {
    let p = seeded();
    gate_ready(&p);
    p.write("src/ui/Button.css", ".b { color: var(--color-primary); }\n");
    p.write(".devforgeai/gates.toml", &gates(""));
    let out = cli(&p)
        .args(["gate", "check", "--phase", "explore", "--id", "IDEA-003"])
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
fn the_design_tokens_check_fails_with_e329() {
    let p = seeded();
    gate_ready(&p);
    p.write("src/ui/Button.css", ".b { color: #3356ff; }\n");
    p.write(".devforgeai/gates.toml", &gates(""));
    let out = cli(&p)
        .args(["gate", "check", "--phase", "explore", "--id", "IDEA-003"])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(1));
    let report = p.read(".devforgeai/reports/IDEA-003-explore.yaml");
    assert!(report.contains("DFA-E329"), "report was {report}");
    assert!(report.contains("design lint found"), "report was {report}");
}

#[test]
fn the_design_tokens_check_narrows_to_its_paths_key() {
    let p = seeded();
    gate_ready(&p);
    p.write("src/ui/Button.css", ".b { color: #3356ff; }\n");
    p.write("src/ui/Card.css", ".c { color: var(--color-primary); }\n");
    p.write(".devforgeai/gates.toml", &gates("\"src/ui/Card.css\""));
    let out = cli(&p)
        .args(["gate", "check", "--phase", "explore", "--id", "IDEA-003"])
        .output()
        .expect("run");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Register a worktree for `STORY-014` and make it the active build story, as
/// `worktree ensure` plus `phase set build` leave the main checkout.
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
fn lint_reads_the_worktree_during_a_registered_build() {
    let p = Project::new();
    p.write(".devforgeai/brand/tokens.json", TOKENS);
    register_worktree(&p, "wt");
    // The violating file is in the worktree, where a Build run's source is.
    // Nothing at the project root matches the frontend globs at all.
    p.write("wt/src/ui/Button.css", ".b { color: #ff0000; }\n");

    let (code, out, err) = run(&p, &["design", "lint"]);

    // The token file is a document and stays at the root; the files it lints
    // are source and moved with the worktree.
    assert_eq!(code, 1, "the literal colour is found: {out} {err}");
    assert!(err.contains("DFA-E240"), "stderr was {err}");
    assert!(
        err.contains("Button.css"),
        "the worktree's file was read: {err}"
    );
}

#[test]
fn lint_reads_the_root_when_no_worktree_is_registered() {
    let p = Project::new();
    p.write(".devforgeai/brand/tokens.json", TOKENS);
    p.write("src/ui/Button.css", ".b { color: #ff0000; }\n");

    // The control: with no worktree open the two directories are the same one.
    let (code, _, err) = run(&p, &["design", "lint"]);
    assert_eq!(code, 1, "stderr was {err}");
    assert!(err.contains("Button.css"), "{err}");
}

#[test]
fn lint_skips_a_root_copy_during_a_registered_build() {
    let p = Project::new();
    p.write(".devforgeai/brand/tokens.json", TOKENS);
    register_worktree(&p, "wt");
    // A stale copy at the project root is not the build's source. The default
    // globs match at any depth, so walking the root would find the worktree's
    // files as well and the positive case alone would not tell the two apart.
    p.write("src/ui/Button.css", ".b { color: #ff0000; }\n");

    let (code, _, err) = run(&p, &["design", "lint"]);
    assert_eq!(code, 0, "the root copy is not linted: {err}");
}
