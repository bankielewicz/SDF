//! `init --analyze`: the brownfield draft of the six context files.

mod common;

use common::Project;
use devforgeai::analyze;
use devforgeai::config::{Config, Stack};

/// A project with a `config.toml` and a small brownfield tree.
fn brownfield() -> Project {
    let p = Project::new();
    p.write_config(&Config {
        degraded: false,
        stack: vec![Stack {
            id: "rust".into(),
            markers: vec!["Cargo.toml".into()],
            package_manager: "cargo".into(),
            source_roots: vec!["src".into()],
            test_command: "cargo test".into(),
            coverage_command: String::new(),
            lint_command: String::new(),
            coverage_format: "lcov".into(),
            timeout_secs: 900,
            ..Default::default()
        }],
        generated_at: "2026-09-10T14:02:11Z".into(),
        ..Default::default()
    });
    p.write(
        "Cargo.toml",
        "[package]\nname = \"acme\"\n\n[dependencies]\nserde = \"1\"\nclap = { version = \"4.5\" }\n",
    );
    p.write("Cargo.lock", "# lock\n");
    p.write("rust-toolchain.toml", "[toolchain]\nchannel = \"1.97\"\n");
    p.write(".editorconfig", "root = true\n");
    p.write("src/main.rs", "fn main() {}\n");
    p.write("src/domain/order.rs", "pub struct Order;\n");
    p.write("src/api/http.rs", "pub fn serve() {}\n");
    p.write("tests/order_test.rs", "#[test] fn t() {}\n");
    p
}

fn run(p: &Project, force: bool) -> analyze::Analysis {
    let cfg = devforgeai::config::load(p.root()).expect("config");
    analyze::run(p.root(), &cfg, force).expect("the analysis runs")
}

fn context(p: &Project, stem: &str) -> String {
    p.read(&format!(".devforgeai/context/{stem}.md"))
}

#[test]
fn all_six_files_are_written_as_drafts() {
    let p = brownfield();
    let a = run(&p, false);
    assert_eq!(a.derived, vec!["tech-stack", "source-tree", "dependencies"]);
    assert_eq!(
        a.stubs,
        vec![
            "coding-standards",
            "architecture-constraints",
            "anti-patterns"
        ]
    );
    assert!(a.skipped.is_empty());
    assert!(!a.truncated);

    for stem in devforgeai::doc::CONTEXT_STEMS {
        let text = context(&p, stem);
        assert!(text.contains("status: draft"), "{stem}");
        assert!(text.contains("phase: constitute"), "{stem}");
        assert!(text.contains("consumes: []"), "{stem}");
        assert!(text.contains(&format!("id: {stem}")), "{stem}");
    }
}

#[test]
fn a_stub_carries_its_open_question_and_a_derived_file_does_not() {
    let p = brownfield();
    run(&p, false);
    for stem in analyze::STUBS {
        assert!(
            context(&p, stem).contains(analyze::STUB_QUESTION),
            "{stem} names the skill that fills it"
        );
    }
    assert!(
        !context(&p, "source-tree").contains(analyze::STUB_QUESTION),
        "a derived file is not a stub"
    );
}

#[test]
fn tech_stack_carries_the_six_derived_sections() {
    let p = brownfield();
    run(&p, false);
    let text = context(&p, "tech-stack");
    for heading in [
        "## Languages",
        "## Package managers",
        "## Test tooling",
        "## Lint tooling",
        "## Runtime versions",
        "## Open",
    ] {
        assert!(text.contains(heading), "{heading} in {text}");
    }
    assert!(text.contains("| rust | Cargo.toml |"), "{text}");
    assert!(text.contains("Cargo.lock"), "the lockfile is named: {text}");
    assert!(text.contains("1.97"), "the toolchain pin: {text}");
    assert!(
        text.contains("- stack 'rust' has no lint command"),
        "the open list mirrors the gaps: {text}"
    );
}

#[test]
fn source_tree_maps_directories_to_layers_and_names_the_entry_point() {
    let p = brownfield();
    run(&p, false);
    let text = context(&p, "source-tree");
    for heading in ["## Tree", "## Layers", "## Entry points", "## Test roots"] {
        assert!(text.contains(heading), "{heading} in {text}");
    }
    assert!(text.contains("| src/domain | domain |"), "{text}");
    assert!(text.contains("| src/api | interface |"), "{text}");
    assert!(text.contains("- src/main.rs"), "{text}");
    assert!(text.contains("- tests"), "the test root: {text}");
}

#[test]
fn dependencies_lists_the_manifest_entries_and_the_lockfile() {
    let p = brownfield();
    run(&p, false);
    let text = context(&p, "dependencies");
    for heading in [
        "## Direct dependencies",
        "## Lockfile status",
        "## Unverified",
    ] {
        assert!(text.contains(heading), "{heading} in {text}");
    }
    assert!(text.contains("| serde | 1 | Cargo.toml |"), "{text}");
    assert!(text.contains("| clap | 4.5 | Cargo.toml |"), "{text}");
    assert!(
        text.contains("| Cargo.toml | Cargo.lock | present |"),
        "{text}"
    );
}

#[test]
fn the_coding_standards_stub_names_the_configuration_it_found() {
    let p = brownfield();
    run(&p, false);
    let text = context(&p, "coding-standards");
    assert!(text.contains("## Detected configuration"), "{text}");
    assert!(text.contains("- .editorconfig"), "{text}");
    for heading in [
        "## Naming",
        "## Formatting",
        "## Error handling",
        "## Testing",
    ] {
        assert!(text.contains(heading), "{heading} in {text}");
    }
    assert_eq!(
        text.matches(analyze::STUB_LINE).count(),
        4,
        "one body line per stub heading: {text}"
    );
}

#[test]
fn the_architecture_stub_carries_the_layer_map_and_the_boundaries() {
    let p = brownfield();
    run(&p, false);
    let text = context(&p, "architecture-constraints");
    assert!(text.contains("## Layer map"), "{text}");
    assert!(text.contains("| domain | **/domain/**"), "{text}");
    assert!(text.contains("## Module boundaries"), "{text}");
    assert!(text.contains("- src/domain"), "{text}");
    assert!(text.contains("- src/api"), "{text}");
    assert!(
        text.contains(&format!("## Constraints\n\n{}", analyze::STUB_LINE)),
        "{text}"
    );
    assert!(
        !text.contains("CON-"),
        "the stub carries no constraint entry: {text}"
    );
}

#[test]
fn the_anti_pattern_stub_reports_no_rule_set_when_none_exists() {
    let p = brownfield();
    run(&p, false);
    let text = context(&p, "anti-patterns");
    assert!(text.contains("## Enabled lint rule sets"), "{text}");
    assert!(text.contains("None detected."), "{text}");
    assert!(
        text.contains(&format!("## Patterns\n\n{}", analyze::STUB_LINE)),
        "{text}"
    );
    assert!(!text.contains("AP-"), "the stub carries no entry: {text}");
}

#[test]
fn the_anti_pattern_stub_reads_a_linter_configuration_when_one_exists() {
    let p = brownfield();
    p.write(
        ".eslintrc.json",
        "{\n  \"extends\": [\"eslint:recommended\"]\n}\n",
    );
    run(&p, false);
    let text = context(&p, "anti-patterns");
    assert!(text.contains(".eslintrc.json"), "{text}");
}

#[test]
fn an_existing_context_file_is_skipped_without_force() {
    let p = brownfield();
    p.write(".devforgeai/context/tech-stack.md", "# mine\n");
    let a = run(&p, false);
    assert_eq!(a.skipped, vec!["tech-stack"]);
    assert_eq!(a.derived, vec!["source-tree", "dependencies"]);
    assert_eq!(context(&p, "tech-stack"), "# mine\n", "left untouched");
}

#[test]
fn force_overwrites_an_existing_context_file() {
    let p = brownfield();
    p.write(".devforgeai/context/tech-stack.md", "# mine\n");
    let a = run(&p, true);
    assert!(a.skipped.is_empty());
    assert!(context(&p, "tech-stack").contains("status: draft"));
}

#[test]
fn the_walk_skips_the_excluded_directories() {
    let p = brownfield();
    p.write("node_modules/pkg/index.js", "module.exports = {};\n");
    p.write("target/debug/build.rs", "fn main() {}\n");
    let a = run(&p, false);
    let text = context(&p, "source-tree");
    assert!(!text.contains("node_modules"), "{text}");
    assert!(!text.contains("target/debug"), "{text}");
    assert!(a.files_scanned > 0);
}

#[test]
fn an_analysed_project_still_fails_the_context_audit_while_it_is_a_draft() {
    // CA-2 is what keeps a brownfield draft out of a commit.
    let p = brownfield();
    run(&p, false);
    let out = assert_cmd::Command::cargo_bin("devforgeai")
        .expect("binary")
        .arg("--project")
        .arg(p.root())
        .args(["context", "audit"])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(stderr.contains("DFA-E223"), "stderr was {stderr}");
    assert!(stderr.contains("is status 'draft'"), "stderr was {stderr}");
}
