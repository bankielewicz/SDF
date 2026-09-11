//! `stack detect`, driven through the library: the dispatcher is not wired to
//! this subcommand yet, so these tests build a `Ctx` over a temporary project
//! and call `cmd::stack::detect` directly. No test reads or writes the real
//! home directory or the real working tree.

use devforgeai::config;
use devforgeai::ctx::Ctx;
use std::path::Path;

/// The literal line the spec prints for a Rust project.
const RUST_LINE: &str = "rust    cargo   test: cargo test --all-features   coverage: lcov   lint: cargo clippy --all-targets --all-features -- -D warnings";

/// The line `stack detect` emits when no marker matched.
const UNDETECTED: &str = "Stack undetected; command checks skipped.";

fn temp() -> tempfile::TempDir {
    tempfile::tempdir().expect("temp dir")
}

fn file(root: &Path, rel: &str, body: &str) {
    let p = root.join(rel);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).expect("mkdir");
    }
    std::fs::write(&p, body).expect("write");
}

fn dir(root: &Path, rel: &str) {
    std::fs::create_dir_all(root.join(rel)).expect("mkdir");
}

/// A project root with `.devforgeai/` already present, as `init` leaves it.
fn project(root: &Path) {
    dir(root, ".devforgeai");
}

/// Write a config into the project, as a previous run would have left it.
fn write_config(root: &Path, cfg: &config::Config) {
    file(
        root,
        ".devforgeai/config.toml",
        &config::to_toml(cfg).expect("render"),
    );
}

fn config_path(root: &Path) -> std::path::PathBuf {
    root.join(".devforgeai").join("config.toml")
}

#[test]
fn detect_writes_config() {
    let t = temp();
    let root = t.path();
    project(root);
    file(root, "Cargo.toml", "[package]\nname = \"x\"\n");
    dir(root, "src");

    let mut ctx = Ctx::new(root.to_path_buf());
    let out = devforgeai::cmd::stack::detect(&mut ctx, false).expect("detection succeeds");

    assert_eq!(out.exit, None, "exit 0 in every case");
    assert!(!out.degraded);
    assert_eq!(out.human.first().map(String::as_str), Some(RUST_LINE));
    assert_eq!(
        out.human.last().map(String::as_str),
        Some("degraded: false")
    );

    assert!(config_path(root).exists(), "config.toml was written");
    let cfg = config::load(root).expect("the written file loads");
    assert_eq!(cfg.schema, config::SCHEMA);
    assert_eq!(cfg.cli_version, devforgeai::VERSION);
    assert!(
        devforgeai::time::parse_rfc3339(&cfg.generated_at).is_some(),
        "generated_at is RFC 3339: {}",
        cfg.generated_at
    );
    assert!(!cfg.degraded);
    assert_eq!(cfg.stack.len(), 1);
    assert_eq!(cfg.stack[0].id, "rust");
    assert_eq!(cfg.stack[0].markers, vec!["Cargo.toml"]);
    assert_eq!(cfg.stack[0].source_roots, vec!["src"]);
    assert_eq!(cfg.stack[0].timeout_secs, 900);
    assert_eq!(
        cfg.frontend.globs.len(),
        11,
        "no node stack, so the default list stands"
    );
    // The untouched tables arrive at their defaults when the file was absent.
    assert_eq!(cfg.layer.len(), 4);
    assert_eq!(cfg.verifier.len(), 20);
}

#[test]
fn detect_polyglot_writes_two_stacks() {
    let t = temp();
    let root = t.path();
    project(root);
    file(root, "Cargo.toml", "[package]\nname = \"x\"\n");
    file(
        root,
        "package.json",
        r#"{"scripts":{"test":"vitest","lint":"eslint ."}}"#,
    );
    file(root, "yarn.lock", "");
    dir(root, "src");

    let mut ctx = Ctx::new(root.to_path_buf());
    let out = devforgeai::cmd::stack::detect(&mut ctx, false).expect("detection succeeds");

    let cfg = config::load(root).expect("the written file loads");
    let ids: Vec<&str> = cfg.stack.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(ids, vec!["rust", "node"], "detection order is table order");
    assert_eq!(cfg.stack[1].package_manager, "yarn");
    assert_eq!(cfg.stack[1].test_command, "yarn run test");
    assert_eq!(cfg.stack[1].coverage_command, "", "no coverage script");
    assert_eq!(cfg.stack[1].lint_command, "yarn run lint");

    assert!(
        cfg.frontend.globs.contains(&"**/*.ts".to_string())
            && cfg.frontend.globs.contains(&"**/*.js".to_string()),
        "a node stack extends the frontend globs"
    );
    assert_eq!(cfg.frontend.globs.len(), 13);

    assert_eq!(out.human.len(), 3, "one line per stack, then the flag");
    assert!(!out.degraded);
}

#[test]
fn detect_unknown_sets_degraded() {
    let t = temp();
    let root = t.path();
    project(root);
    file(root, "README.md", "# nothing to detect\n");

    let mut ctx = Ctx::new(root.to_path_buf());
    let out = devforgeai::cmd::stack::detect(&mut ctx, false).expect("detection succeeds");

    assert_eq!(out.exit, None, "the undetected case still exits 0");
    assert!(out.degraded);
    assert_eq!(
        out.human,
        vec!["degraded: true".to_string(), UNDETECTED.to_string()],
        "the undetected line rides along on human output until the dispatcher \
         gains a stderr channel"
    );
    assert!(out.warnings.is_empty(), "the line carries no DFA- code");

    let cfg = config::load(root).expect("the written file loads");
    assert!(cfg.stack.is_empty(), "stack = []");
    assert!(cfg.degraded);
    assert_eq!(out.data["degraded"], serde_json::json!(true));
}

#[test]
fn detect_preserves_layers_and_verifiers() {
    let t = temp();
    let root = t.path();
    project(root);
    file(root, "Cargo.toml", "[package]\nname = \"x\"\n");
    dir(root, "src");

    // A config the project has already edited: every table `stack detect`
    // leaves alone carries a non-default value.
    let mut before = config::Config {
        generated_at: "2020-01-01T00:00:00Z".into(),
        cli_version: "0.0.1".into(),
        coverage: config::CoverageCfg {
            overall_min: 91.0,
            unassigned_policy: "fail".into(),
            ..Default::default()
        },
        frontend: config::Frontend {
            tokens_path: "brand/tokens.json".into(),
            exclude: vec!["**/fixtures/**".into()],
            ..Default::default()
        },
        explore: config::ExploreCfg {
            timebox_days: 9,
            ..Default::default()
        },
        plan: config::PlanCfg {
            sprint_capacity_points: 34,
            ..Default::default()
        },
        build: config::BuildCfg {
            branch_prefix: "feat/".into(),
            ..Default::default()
        },
        verify: config::VerifyCfg {
            mode: "deep".into(),
            ..Default::default()
        },
        release: config::ReleaseCfg {
            service_port: 9090,
            ..Default::default()
        },
        reflect: config::ReflectCfg {
            window_days: 30,
            ..Default::default()
        },
        ..config::Config::default()
    };
    before.layer[0].coverage_min = 97.5;
    before.layer.retain(|l| l.name != "interface");
    before.verifier.truncate(2);
    before.verifier[0].name = "house-verifier".into();
    let text = config::to_toml(&before).expect("serialise");
    file(root, ".devforgeai/config.toml", &text);

    let mut ctx = Ctx::new(root.to_path_buf());
    devforgeai::cmd::stack::detect(&mut ctx, false).expect("detection succeeds");

    let after = config::load(root).expect("the rewritten file loads");
    assert_eq!(after.layer.len(), 3, "the deleted layer stays deleted");
    assert_eq!(after.layer[0].coverage_min, 97.5);
    assert_eq!(after.verifier.len(), 2);
    assert_eq!(after.verifier[0].name, "house-verifier");
    assert_eq!(after.coverage.overall_min, 91.0);
    assert_eq!(after.coverage.unassigned_policy, "fail");
    assert_eq!(after.frontend.tokens_path, "brand/tokens.json");
    assert_eq!(after.frontend.exclude, vec!["**/fixtures/**"]);
    assert_eq!(after.explore.timebox_days, 9);
    assert_eq!(after.plan.sprint_capacity_points, 34);
    assert_eq!(after.build.branch_prefix, "feat/");
    assert_eq!(after.verify.mode, "deep");
    assert_eq!(after.release.service_port, 9090);
    assert_eq!(after.reflect.window_days, 30);

    // What detection does own is rewritten.
    assert_eq!(after.stack.len(), 1);
    assert!(!after.degraded);
    assert_eq!(after.cli_version, devforgeai::VERSION);
    assert_ne!(after.generated_at, "2020-01-01T00:00:00Z");
    assert_eq!(after.frontend.globs.len(), 11, "globs are detection's own");
}

#[test]
fn detect_dry_run_writes_nothing() {
    let t = temp();
    let root = t.path();
    project(root);
    file(root, "go.mod", "module example.com/x\n");

    let mut ctx = Ctx::new(root.to_path_buf());
    let out = devforgeai::cmd::stack::detect(&mut ctx, true).expect("detection succeeds");

    assert!(!config_path(root).exists(), "--dry-run writes nothing");
    assert_eq!(out.data["written"], serde_json::json!(false));
    assert_eq!(
        out.human.first().map(String::as_str),
        Some("go      go      test: go test ./...   coverage: go-cover   lint: go vet ./..."),
        "the result is printed all the same"
    );

    // An existing file survives a dry run byte for byte.
    let text = config::to_toml(&config::Config::default()).expect("serialise");
    file(root, ".devforgeai/config.toml", &text);
    devforgeai::cmd::stack::detect(&mut ctx, true).expect("detection succeeds");
    assert_eq!(
        std::fs::read_to_string(config_path(root)).expect("read"),
        text,
        "the existing file is untouched"
    );
}

#[test]
fn detect_json_shape() {
    let t = temp();
    let root = t.path();
    project(root);
    file(root, "Cargo.toml", "[package]\nname = \"x\"\n");
    dir(root, "src");

    let mut ctx = Ctx::new(root.to_path_buf());
    let out = devforgeai::cmd::stack::detect(&mut ctx, false).expect("detection succeeds");

    let data = &out.data;
    let keys: Vec<&str> = data
        .as_object()
        .expect("data is an object")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        vec!["stacks", "degraded", "written", "path"],
        "the four keys the spec names, in the spec's order"
    );
    assert_eq!(data["degraded"], serde_json::json!(false));
    assert_eq!(data["written"], serde_json::json!(true));
    assert_eq!(data["path"], serde_json::json!(".devforgeai/config.toml"));

    let stacks = data["stacks"].as_array().expect("stacks is an array");
    assert_eq!(stacks.len(), 1);
    assert_eq!(stacks[0]["id"], serde_json::json!("rust"));
    assert_eq!(
        stacks[0]["test_command"],
        serde_json::json!("cargo test --all-features")
    );
    assert_eq!(stacks[0]["coverage_format"], serde_json::json!("lcov"));
    assert_eq!(
        stacks[0]["coverage_paths"],
        serde_json::json!(["target/devforgeai/lcov.info"])
    );
    assert_eq!(stacks[0]["timeout_secs"], serde_json::json!(900));
    assert_eq!(stacks[0]["env"], serde_json::json!({}));
}

// --- a hand-written stack survives detection ------------------------------

/// A `config.toml` holding one hand-written stack and nothing the detector
/// would recognise.
fn manual_only_config() -> devforgeai::config::Config {
    devforgeai::config::Config {
        degraded: false,
        stack: vec![devforgeai::config::Stack {
            id: "elixir".into(),
            source: "manual".into(),
            markers: vec!["mix.exs".into()],
            package_manager: "mix".into(),
            source_roots: vec!["lib".into()],
            test_command: "sh ci/test".into(),
            coverage_format: "none".into(),
            timeout_secs: 900,
            ..Default::default()
        }],
        generated_at: "2026-09-10T14:02:11Z".into(),
        ..Default::default()
    }
}

#[test]
fn a_manual_stack_survives_a_detect_that_finds_nothing() {
    let t = temp();
    let root = t.path();
    project(root);
    write_config(root, &manual_only_config());
    // No marker file of any ecosystem the detector knows.

    let mut ctx = Ctx::new(root.to_path_buf());
    let out = devforgeai::cmd::stack::detect(&mut ctx, false).expect("detect");

    // The hook runs this at every session start. Overwriting the table here
    // discards the only record of the project's test command, and `degraded`
    // then turns off every command-running gate check.
    let cfg = devforgeai::config::load(root).expect("config still loads");
    assert_eq!(cfg.stack.len(), 1, "{:?}", cfg.stack);
    assert_eq!(cfg.stack[0].id, "elixir");
    assert_eq!(cfg.stack[0].test_command, "sh ci/test");
    assert_eq!(cfg.stack[0].source, "manual");
    assert!(!cfg.degraded, "a project with commands is not degraded");
    assert_eq!(out.data["degraded"], false);
}

#[test]
fn a_stack_with_no_source_key_reads_as_manual() {
    let t = temp();
    let root = t.path();
    project(root);
    // A file written before the field existed.
    let text = concat!(
        "schema = \"devforgeai/config/1\"\n",
        "generated_at = \"2026-09-10T14:02:11Z\"\n",
        "cli_version = \"1.0.0\"\n",
        "degraded = false\n\n",
        "[[stack]]\n",
        "id = \"elixir\"\n",
        "markers = [\"mix.exs\"]\n",
        "package_manager = \"mix\"\n",
        "source_roots = [\"lib\"]\n",
        "test_command = \"sh ci/test\"\n",
        "coverage_command = \"\"\n",
        "coverage_format = \"none\"\n",
        "coverage_paths = []\n",
        "lint_command = \"\"\n",
        "complexity_command = \"\"\n",
        "timeout_secs = 900\n\n",
        "[stack.env]\n",
    );
    file(root, ".devforgeai/config.toml", text);

    let mut ctx = Ctx::new(root.to_path_buf());
    devforgeai::cmd::stack::detect(&mut ctx, false).expect("detect");

    let cfg = devforgeai::config::load(root).expect("config");
    assert_eq!(cfg.stack.len(), 1, "an absent source is protected");
    assert_eq!(cfg.stack[0].test_command, "sh ci/test");
}

#[test]
fn a_detected_entry_is_refreshed_and_a_manual_one_keeps_its_place() {
    let t = temp();
    let root = t.path();
    project(root);
    let mut cfg = manual_only_config();
    // A stale detected entry whose command the detector will overwrite.
    cfg.stack.push(devforgeai::config::Stack {
        id: "rust".into(),
        source: "detected".into(),
        test_command: "cargo test --stale".into(),
        timeout_secs: 900,
        ..Default::default()
    });
    write_config(root, &cfg);
    file(root, "Cargo.toml", "[package]\nname = \"x\"\n");

    let mut ctx = Ctx::new(root.to_path_buf());
    devforgeai::cmd::stack::detect(&mut ctx, false).expect("detect");

    let cfg = devforgeai::config::load(root).expect("config");
    assert_eq!(cfg.stack.len(), 2, "{:?}", cfg.stack);
    assert_eq!(
        cfg.stack[0].id, "elixir",
        "the manual entry keeps its place"
    );
    assert_eq!(cfg.stack[0].test_command, "sh ci/test");
    assert_eq!(cfg.stack[1].id, "rust");
    assert_ne!(
        cfg.stack[1].test_command, "cargo test --stale",
        "the detected entry was refreshed"
    );
    assert_eq!(cfg.stack[1].source, "detected");
}

#[test]
fn a_detected_entry_is_dropped_when_its_markers_are_gone() {
    let t = temp();
    let root = t.path();
    project(root);
    let mut cfg = manual_only_config();
    cfg.stack.push(devforgeai::config::Stack {
        id: "rust".into(),
        source: "detected".into(),
        test_command: "cargo test".into(),
        timeout_secs: 900,
        ..Default::default()
    });
    write_config(root, &cfg);
    // No `Cargo.toml`: the ecosystem is gone.

    let mut ctx = Ctx::new(root.to_path_buf());
    devforgeai::cmd::stack::detect(&mut ctx, false).expect("detect");

    let cfg = devforgeai::config::load(root).expect("config");
    assert_eq!(cfg.stack.len(), 1);
    assert_eq!(
        cfg.stack[0].id, "elixir",
        "only the detector's own entry is dropped"
    );
}

#[test]
fn a_manual_entry_wins_over_a_detection_of_the_same_id() {
    let t = temp();
    let root = t.path();
    project(root);
    let mut cfg = manual_only_config();
    cfg.stack[0].id = "rust".into();
    cfg.stack[0].test_command = "sh ci/test".into();
    write_config(root, &cfg);
    file(root, "Cargo.toml", "[package]\nname = \"x\"\n");

    let mut ctx = Ctx::new(root.to_path_buf());
    devforgeai::cmd::stack::detect(&mut ctx, false).expect("detect");

    let cfg = devforgeai::config::load(root).expect("config");
    assert_eq!(cfg.stack.len(), 1, "no second rust entry: {:?}", cfg.stack);
    assert_eq!(
        cfg.stack[0].test_command, "sh ci/test",
        "where the two disagree the human's entry stands"
    );
}

#[test]
fn a_hand_written_frontend_glob_survives() {
    let t = temp();
    let root = t.path();
    project(root);
    let mut cfg = manual_only_config();
    cfg.frontend.globs = vec!["ui/**/*.svelte".into()];
    write_config(root, &cfg);
    file(root, "package.json", "{\"name\":\"x\"}\n");

    let mut ctx = Ctx::new(root.to_path_buf());
    devforgeai::cmd::stack::detect(&mut ctx, false).expect("detect");

    let cfg = devforgeai::config::load(root).expect("config");
    assert!(
        cfg.frontend.globs.iter().any(|g| g == "ui/**/*.svelte"),
        "the globs carry no source, so the union is the only safe rule: {:?}",
        cfg.frontend.globs
    );
    assert!(
        cfg.frontend.globs.iter().any(|g| g == "**/*.ts"),
        "and detection still adds its own: {:?}",
        cfg.frontend.globs
    );
}

#[test]
fn a_detect_that_changes_nothing_leaves_the_file_untouched() {
    let t = temp();
    let root = t.path();
    project(root);
    file(root, "Cargo.toml", "[package]\nname = \"x\"\n");

    // First run writes the file.
    let mut ctx = Ctx::new(root.to_path_buf());
    let first = devforgeai::cmd::stack::detect(&mut ctx, false).expect("detect");
    assert_eq!(first.data["written"], true);

    let path = config_path(root);
    let before_bytes = std::fs::read(&path).expect("read");
    let before_mtime = std::fs::metadata(&path)
        .and_then(|m| m.modified())
        .expect("mtime");

    // The hook reruns this at every session start; an unchanged result must
    // not churn the mtime a file watcher or the Stop-time scan reads, and
    // `generated_at` must not claim a detection that did not happen.
    let mut ctx = Ctx::new(root.to_path_buf());
    let second = devforgeai::cmd::stack::detect(&mut ctx, false).expect("detect");
    assert_eq!(
        second.data["written"], false,
        "nothing changed, so nothing was written"
    );

    assert_eq!(
        std::fs::read(&path).expect("read"),
        before_bytes,
        "byte for byte"
    );
    assert_eq!(
        std::fs::metadata(&path)
            .and_then(|m| m.modified())
            .expect("mtime"),
        before_mtime,
        "and the mtime is untouched"
    );
}

#[test]
fn an_invalid_source_value_is_e104() {
    let t = temp();
    let root = t.path();
    project(root);
    let text = concat!(
        "schema = \"devforgeai/config/1\"\n",
        "generated_at = \"2026-09-10T14:02:11Z\"\n",
        "cli_version = \"1.0.0\"\n",
        "degraded = false\n\n",
        "[[stack]]\n",
        "id = \"rust\"\n",
        "source = \"inferred\"\n",
        "timeout_secs = 900\n\n",
        "[stack.env]\n",
    );
    file(root, ".devforgeai/config.toml", text);

    let err = devforgeai::config::load(root).expect_err("'inferred' is neither");
    assert_eq!(err.code(), "DFA-E104");
    let msg = &err.diag().expect("diag").message;
    assert!(msg.contains("inferred"), "{msg}");
}
