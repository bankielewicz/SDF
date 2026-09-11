//! `hook install`: the settings merge, the three git hook scripts, and the
//! token substitution. Every test drives the library against a temporary
//! project directory and never touches the real home directory.

use devforgeai::cli::HookInstallArgs;
use devforgeai::ctx::Ctx;
use devforgeai::hooks;
use serde_json::Value;
use std::path::{Path, PathBuf};

/// The instant every test pins the clock to, so the backup file name is fixed.
const NOW: &str = "2026-09-10T14:02:11Z";
/// The backup file name `NOW` produces.
const BACKUP: &str = ".claude/settings.json.bak-20260910T140211Z";

/// Pin the clock. Every test sets the same value, so the tests in this binary
/// may run in parallel; no test ever clears it.
fn pin_clock() {
    std::env::set_var("DEVFORGEAI_NOW", NOW);
}

/// A temporary project root with a `.devforgeai/` directory.
fn project() -> (tempfile::TempDir, PathBuf) {
    pin_clock();
    let t = tempfile::tempdir().expect("temp dir");
    let root = t.path().to_path_buf();
    std::fs::create_dir_all(root.join(".devforgeai")).expect("mkdir .devforgeai");
    (t, root)
}

/// Write `.claude/settings.json` with the given text.
fn write_settings(root: &Path, text: &str) {
    let p = root.join(".claude");
    std::fs::create_dir_all(&p).expect("mkdir .claude");
    std::fs::write(p.join("settings.json"), text).expect("write settings");
}

/// Read `.claude/settings.json` as JSON.
fn read_settings(root: &Path) -> Value {
    let bytes = std::fs::read(root.join(".claude").join("settings.json")).expect("read settings");
    serde_json::from_slice(&bytes).expect("settings parse")
}

/// Every `command` string anywhere under one event's array.
fn commands(settings: &Value, event: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(entries) = settings["hooks"][event].as_array() {
        for entry in entries {
            if let Some(inner) = entry["hooks"].as_array() {
                for h in inner {
                    let Some(c) = h["command"].as_str() else {
                        continue;
                    };
                    // Exec form: every handler shares one `command`, the
                    // binary, and is told apart by `args`.
                    let args: Vec<&str> = h["args"]
                        .as_array()
                        .map(|a| a.iter().filter_map(Value::as_str).collect())
                        .unwrap_or_default();
                    if args.is_empty() {
                        out.push(c.to_string());
                    } else {
                        out.push(format!("{c} {}", args.join(" ")));
                    }
                }
            }
        }
    }
    out
}

/// Assert a byte slice carries no CRLF pair. A string compare hides one.
fn assert_lf(bytes: &[u8], what: &str) {
    assert!(
        !bytes.windows(2).any(|w| w == b"\r\n"),
        "{what} is written with LF endings"
    );
}

/// Create a bare `.git` directory so the fallback path resolves.
fn fake_git(root: &Path) {
    std::fs::create_dir_all(root.join(".git")).expect("mkdir .git");
}

#[test]
fn install_creates_settings_when_absent() {
    let (_t, root) = project();
    let args = HookInstallArgs {
        force: false,
        claude_only: true,
        git_only: false,
    };
    let mut ctx = Ctx::new(root.clone());
    let outcome = devforgeai::cmd::hook::install(&mut ctx, &args).expect("install");

    let settings = read_settings(&root);
    for event in devforgeai::hooks::settings::EVENTS {
        assert!(
            settings["hooks"][event].is_array(),
            "{event} written into a fresh settings file"
        );
    }
    assert_eq!(
        outcome.data["events"],
        serde_json::json!([
            "SessionStart",
            "UserPromptExpansion",
            "PreToolUse",
            "PostToolUse",
            "Stop",
            "SubagentStop"
        ]),
        "the events keep the order the spec fixes"
    );
    assert_eq!(outcome.data["settings"], "merged");
    assert_eq!(
        outcome.data["backup"], "",
        "no previous file means no backup"
    );
    assert!(
        outcome.human[0].starts_with("Hooks  .claude/settings.json merged (6 events)"),
        "human line was {:?}",
        outcome.human
    );

    let strays: Vec<_> = std::fs::read_dir(root.join(".claude"))
        .expect("read_dir .claude")
        .filter_map(Result::ok)
        .filter(|e| e.file_name().to_string_lossy().contains(".bak-"))
        .collect();
    assert!(strays.is_empty(), "no backup is written when none existed");
}

#[test]
fn install_merges_into_existing_settings() {
    let (_t, root) = project();
    write_settings(
        &root,
        r#"{
  "hooks": {
    "SessionStart": [
      { "matcher": "*", "hooks": [ { "type": "command", "command": "echo hello" } ] }
    ]
  }
}"#,
    );

    hooks::install(&root, false, true, false).expect("install");

    let settings = read_settings(&root);
    let cmds = commands(&settings, "SessionStart");
    assert!(
        cmds.iter().any(|c| c == "echo hello"),
        "the existing entry survives: {cmds:?}"
    );
    assert!(
        cmds.iter().any(|c| c.ends_with("hook run session-start")),
        "the devforgeai entry is appended: {cmds:?}"
    );
    assert_eq!(
        settings["hooks"]["SessionStart"]
            .as_array()
            .expect("array")
            .len(),
        2,
        "appended, not replaced"
    );
}

#[test]
fn install_preserves_foreign_hooks_and_keys() {
    let (_t, root) = project();
    write_settings(
        &root,
        r#"{
  "model": "opus",
  "permissions": { "allow": ["Bash(git:*)"] },
  "hooks": {
    "PreToolUse": [
      { "matcher": "Bash", "hooks": [ { "type": "command", "command": "other-tool guard" } ] }
    ],
    "Notification": [
      { "matcher": "*", "hooks": [ { "type": "command", "command": "notify-send hi" } ] }
    ]
  }
}"#,
    );

    hooks::install(&root, false, true, false).expect("install");

    let settings = read_settings(&root);
    assert_eq!(
        settings["model"], "opus",
        "a foreign top-level key survives"
    );
    assert_eq!(
        settings["permissions"]["allow"][0], "Bash(git:*)",
        "a foreign nested key survives"
    );
    assert_eq!(
        commands(&settings, "Notification"),
        vec!["notify-send hi".to_string()],
        "a foreign event key survives untouched"
    );

    let pre = commands(&settings, "PreToolUse");
    assert!(
        pre.iter().any(|c| c == "other-tool guard"),
        "the foreign entry in a shared event survives: {pre:?}"
    );
    assert!(
        pre.iter().any(|c| c.ends_with("hook run pre-tool-use")),
        "the devforgeai entry is appended beside it: {pre:?}"
    );
}

#[test]
fn install_grants_the_binary_a_standing_permission() {
    let (_t, root) = project();
    let report = hooks::install(&root, false, true, false).expect("install");

    // A skill's `allowed-tools` grant clears on the next user message, while a
    // phase spans many turns that each call the binary. Without the standing
    // rule the second turn of every phase stops for a permission prompt.
    let settings = read_settings(&root);
    let allow: Vec<&str> = settings["permissions"]["allow"]
        .as_array()
        .expect("permissions.allow")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(allow.contains(&"Bash(devforgeai *)"), "{allow:?}");
    assert!(allow.contains(&"PowerShell(devforgeai *)"), "{allow:?}");
    assert_eq!(report.permissions.len(), 2);

    // Idempotent, and no other entry is disturbed.
    let second = hooks::install(&root, false, true, false).expect("second install");
    let settings = read_settings(&root);
    assert_eq!(
        settings["permissions"]["allow"]
            .as_array()
            .expect("array")
            .len(),
        2,
        "no duplicate rules"
    );
    assert!(second.permissions.is_empty(), "nothing was added twice");
}

#[test]
fn install_keeps_a_foreign_permission_rule() {
    let (_t, root) = project();
    write_settings(
        &root,
        r#"{ "permissions": { "allow": ["Bash(git:*)"], "deny": ["Read(.env)"] } }"#,
    );

    hooks::install(&root, false, true, false).expect("install");

    let settings = read_settings(&root);
    let allow: Vec<&str> = settings["permissions"]["allow"]
        .as_array()
        .expect("allow")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert_eq!(allow[0], "Bash(git:*)", "the project's own rule is first");
    assert_eq!(allow.len(), 3);
    assert_eq!(
        settings["permissions"]["deny"][0], "Read(.env)",
        "a sibling key survives"
    );
}

#[test]
fn the_template_carries_the_matchers_and_timeouts_the_reference_fixes() {
    let (_t, root) = project();
    hooks::install(&root, false, true, false).expect("install");
    let s = read_settings(&root);

    // A `Bash`-only matcher never fires where the PowerShell tool is primary.
    assert_eq!(s["hooks"]["PostToolUse"][1]["matcher"], "Bash|PowerShell");
    assert_eq!(s["hooks"]["PreToolUse"][1]["matcher"], "Bash|PowerShell");
    assert_eq!(
        s["hooks"]["PreToolUse"][0]["matcher"],
        "Write|Edit|NotebookEdit"
    );
    // The wide trust group: an unpinned session can read and change nothing.
    assert_eq!(
        s["hooks"]["PreToolUse"][2]["matcher"],
        "Write|Edit|NotebookEdit|Bash|PowerShell|Agent"
    );

    // A timed-out PreToolUse hook does not block, so a short timeout buys
    // latency at the cost of the gate.
    assert_eq!(s["hooks"]["PreToolUse"][0]["hooks"][0]["timeout"], 600);

    // `Stop` and `FileChanged` have no matcher support; a `"*"` there reads as
    // deliberate filtering to the next person who edits the file.
    assert!(s["hooks"]["Stop"][0].get("matcher").is_none());
    // `SessionStart` drops `compact`, so a mid-turn auto-compaction does not
    // re-run `stack detect` under a gate that is already evaluating.
    assert_eq!(
        s["hooks"]["SessionStart"][0]["matcher"],
        "startup|resume|clear|fork"
    );

    // The partial build gate runs the project's test suite, so it must not
    // stall the agent loop.
    assert_eq!(s["hooks"]["PostToolUse"][1]["hooks"][0]["async"], true);
}

#[test]
fn install_writes_the_template_byte_for_byte_but_for_the_tokens() {
    // The shipped copy and the compiled-in copy are one file.
    let shipped = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("hooks")
            .join("settings.hooks.json"),
    )
    .expect("read hooks/settings.hooks.json");
    assert_lf(&shipped, "hooks/settings.hooks.json");
    assert_eq!(
        String::from_utf8(shipped).expect("utf-8"),
        devforgeai::hooks::settings::TEMPLATE,
        "hooks/settings.hooks.json and cli/templates/settings.hooks.json are one file"
    );
}

/// The block this binary wrote before the exec-form change: the whole
/// invocation inside `command`, `Bash` alone on the shell handler, and the
/// PreToolUse timeout that lets a slow check fail open.
const SUPERSEDED_BLOCK: &str = r#"{
  "hooks": {
    "SessionStart": [ { "matcher": "*", "hooks": [ { "type": "command", "command": "@@ hook run session-start", "timeout": 120 } ] } ],
    "PreToolUse": [ { "matcher": "Write|Edit", "hooks": [ { "type": "command", "command": "@@ hook run pre-tool-use", "timeout": 30 } ] } ],
    "PostToolUse": [
      { "matcher": "Write|Edit", "hooks": [ { "type": "command", "command": "@@ hook run post-tool-use", "timeout": 60 } ] },
      { "matcher": "Bash", "hooks": [ { "type": "command", "command": "@@ hook run post-tool-use", "timeout": 900 } ] }
    ],
    "Stop": [ { "matcher": "*", "hooks": [ { "type": "command", "command": "@@ hook run stop", "timeout": 900 } ] } ],
    "SubagentStop": [ { "matcher": "*", "hooks": [ { "type": "command", "command": "@@ hook run subagent-stop", "timeout": 60 } ] } ]
  }
}"#;

#[test]
fn install_replaces_the_superseded_shell_form_block() {
    let (_t, root) = project();
    // The bare PATH name is the case that matters: the old entry's whole
    // command string then equals the new entry's command plus its args, so a
    // dedup on that string alone reads the replacement as already present and
    // the upgrade silently installs nothing.
    write_settings(&root, &SUPERSEDED_BLOCK.replace("@@", "devforgeai"));

    hooks::install(&root, false, true, false).expect("install");
    let s = read_settings(&root);

    for event in devforgeai::hooks::settings::EVENTS {
        for entry in s["hooks"][event].as_array().expect("array") {
            for h in entry["hooks"].as_array().expect("hooks") {
                assert!(
                    h["args"].is_array(),
                    "{event} carries no superseded shell-form handler: {entry}"
                );
            }
        }
    }

    // The matchers the upgrade exists for.
    let pre: Vec<&str> = s["hooks"]["PreToolUse"]
        .as_array()
        .expect("array")
        .iter()
        .map(|e| e["matcher"].as_str().unwrap_or(""))
        .collect();
    assert!(pre.contains(&"Bash|PowerShell"), "{pre:?}");
    assert!(pre.contains(&"Write|Edit|NotebookEdit"), "{pre:?}");
    assert_eq!(s["hooks"]["PreToolUse"][0]["hooks"][0]["timeout"], 600);
    assert_eq!(s["hooks"]["PostToolUse"][1]["matcher"], "Bash|PowerShell");
    assert!(
        s["hooks"]["UserPromptExpansion"].is_array(),
        "the new event lands too"
    );
}

#[test]
fn install_keeps_a_foreign_shell_form_handler() {
    let (_t, root) = project();
    write_settings(
        &root,
        r#"{
  "hooks": {
    "PreToolUse": [
      { "matcher": "Bash", "hooks": [ { "type": "command", "command": "other-tool hook run guard" } ] },
      { "matcher": "Write", "hooks": [ { "type": "command", "command": "my-own-linter" } ] }
    ]
  }
}"#,
    );

    hooks::install(&root, false, true, false).expect("install");

    // Only this binary's own superseded handlers are replaced. Another tool's
    // entry, even one whose command happens to read `hook run`, is foreign.
    let pre = commands(&settings_of(&root), "PreToolUse");
    assert!(
        pre.iter().any(|c| c == "other-tool hook run guard"),
        "{pre:?}"
    );
    assert!(pre.iter().any(|c| c == "my-own-linter"), "{pre:?}");
}

/// `read_settings` under a name the two tests above read better with.
fn settings_of(root: &Path) -> Value {
    read_settings(root)
}

#[test]
fn install_duplicate_command_gives_w130() {
    let (_t, root) = project();
    hooks::install(&root, false, true, false).expect("first install");
    let first = read_settings(&root);

    let report = hooks::install(&root, false, true, false).expect("second install");

    assert!(
        report.warnings.iter().any(|w| w.code == "DFA-W130"),
        "a second install warns: {:?}",
        report.warnings
    );
    let w = report
        .warnings
        .iter()
        .find(|w| w.code == "DFA-W130")
        .expect("the warning");
    assert!(
        w.message.contains("hook already present; left unchanged"),
        "message was {:?}",
        w.message
    );

    let second = read_settings(&root);
    assert_eq!(first["hooks"], second["hooks"], "nothing is appended twice");
}

#[test]
fn install_backs_up_settings() {
    let (_t, root) = project();
    let original = "{\n  \"model\": \"opus\"\n}\n";
    write_settings(&root, original);

    let report = hooks::install(&root, false, true, false).expect("install");

    assert_eq!(report.backup, BACKUP, "the backup name carries the clock");
    let bak = root
        .join(".claude")
        .join("settings.json.bak-20260910T140211Z");
    assert_eq!(
        std::fs::read_to_string(&bak).expect("read backup"),
        original,
        "the backup is the file as it stood before the merge"
    );
}

#[test]
fn install_unparsable_settings_gives_e131() {
    let (_t, root) = project();
    write_settings(&root, "{ this is not json");

    let err = hooks::install(&root, false, true, false).expect_err("unparsable settings");
    assert_eq!(err.code(), "DFA-E131");
    assert_eq!(err.exit(), 1);
}

#[test]
fn install_writes_three_scripts_with_marker() {
    let (_t, root) = project();
    fake_git(&root);

    let report = hooks::install(&root, false, false, true).expect("install");
    assert_eq!(
        report.git_hooks,
        vec![
            "pre-commit".to_string(),
            "commit-msg".to_string(),
            "pre-push".to_string()
        ]
    );

    for name in ["pre-commit", "commit-msg", "pre-push"] {
        let p = root.join(".git").join("hooks").join(name);
        let bytes = std::fs::read(&p).unwrap_or_else(|e| panic!("read {name}: {e}"));
        assert_lf(&bytes, name);

        let text = String::from_utf8(bytes).expect("utf-8");
        let mut lines = text.lines();
        assert_eq!(
            lines.next(),
            Some("#!/bin/sh"),
            "{name} opens with a shebang"
        );
        assert_eq!(
            lines.next(),
            Some(hooks::githooks::MARKER),
            "{name} carries the marker on the line after the shebang"
        );

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&p)
                .expect("metadata")
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o755, "{name} is executable");
        }
    }
}

#[test]
fn install_substitutes_devforgeai_token() {
    let (_t, root) = project();
    fake_git(&root);
    hooks::install(&root, false, false, false).expect("install");

    for name in ["pre-commit", "commit-msg", "pre-push"] {
        let text = std::fs::read_to_string(root.join(".git").join("hooks").join(name))
            .unwrap_or_else(|e| panic!("read {name}: {e}"));
        assert!(
            !text.contains("@@DEVFORGEAI@@"),
            "{name} still carries the token"
        );
    }
    let settings =
        std::fs::read_to_string(root.join(".claude").join("settings.json")).expect("read settings");
    assert!(
        !settings.contains("@@DEVFORGEAI@@"),
        "the settings block still carries the token"
    );

    // Both branches of the resolution, forced.
    let bare = hooks::githooks::substitute(hooks::githooks::PRE_COMMIT, "devforgeai");
    assert!(bare.contains("DFA=\"devforgeai\""), "the bare-name branch");
    let abs = hooks::githooks::substitute(hooks::githooks::COMMIT_MSG, "C:/tools/devforgeai.exe");
    assert!(
        abs.contains("DFA=\"C:/tools/devforgeai.exe\""),
        "the absolute-path branch, in forward-slash form"
    );
    assert!(
        !hooks::githooks::token_value().is_empty(),
        "the resolution always yields a value"
    );
}

#[test]
fn install_foreign_hook_gives_e132() {
    let (_t, root) = project();
    fake_git(&root);
    let hooks_dir = root.join(".git").join("hooks");
    std::fs::create_dir_all(&hooks_dir).expect("mkdir hooks");
    let foreign = "#!/bin/sh\necho someone else wrote this\n";
    std::fs::write(hooks_dir.join("pre-commit"), foreign).expect("write foreign hook");

    let err = hooks::install(&root, false, false, true).expect_err("a foreign hook blocks");
    assert_eq!(err.code(), "DFA-E132");
    assert_eq!(err.exit(), 1);
    assert_eq!(
        std::fs::read_to_string(hooks_dir.join("pre-commit")).expect("read"),
        foreign,
        "the foreign hook is left exactly as it was"
    );
}

#[test]
fn install_force_replaces_foreign_hook() {
    let (_t, root) = project();
    fake_git(&root);
    let hooks_dir = root.join(".git").join("hooks");
    std::fs::create_dir_all(&hooks_dir).expect("mkdir hooks");
    std::fs::write(hooks_dir.join("pre-commit"), "#!/bin/sh\necho foreign\n").expect("write");

    hooks::install(&root, true, false, true).expect("--force replaces it");

    let bytes = std::fs::read(hooks_dir.join("pre-commit")).expect("read");
    assert_lf(&bytes, "pre-commit");
    let text = String::from_utf8(bytes).expect("utf-8");
    assert!(
        text.lines().nth(1) == Some(hooks::githooks::MARKER),
        "the replacement carries the marker"
    );
    assert!(!text.contains("echo foreign"), "the old body is gone");
}

#[test]
fn install_uses_git_common_dir() {
    pin_clock();
    let t = tempfile::tempdir().expect("temp dir");
    let proj = t.path().join("proj");
    let gitdir = t.path().join("gitdir");
    std::fs::create_dir_all(&proj).expect("mkdir proj");

    // A repository whose git directory lives outside the work tree: the
    // `.git` entry is a file, so only `git rev-parse --git-common-dir`
    // resolves the hooks directory.
    let init = std::process::Command::new("git")
        .arg("init")
        .arg("--separate-git-dir")
        .arg(&gitdir)
        .arg(&proj)
        .output();
    let Ok(out) = init else {
        eprintln!("git is not on PATH; skipping install_uses_git_common_dir");
        return;
    };
    if !out.status.success() {
        eprintln!("git init failed; skipping install_uses_git_common_dir");
        return;
    }

    hooks::install(&proj, true, false, true).expect("install into the separate git dir");

    assert!(
        gitdir.join("hooks").join("pre-commit").is_file(),
        "the hook lands in the git common dir"
    );
    assert!(
        !proj.join(".git").is_dir(),
        "the work tree holds a .git file, not a directory"
    );
}

#[test]
fn install_git_only_skips_settings() {
    let (_t, root) = project();
    fake_git(&root);

    let report = hooks::install(&root, false, false, true).expect("install");

    assert!(
        !root.join(".claude").join("settings.json").exists(),
        "--git-only writes no settings file"
    );
    assert!(report.events.is_empty(), "no events are reported");
    assert_eq!(report.settings, "skipped");
    assert_eq!(report.git_hooks.len(), 3, "the three scripts are written");
}

#[test]
fn the_shell_handlers_collapse_to_one_when_no_rule_renders() {
    let (_t, root) = project();
    hooks::install(&root, false, true, false).expect("install");
    let s = read_settings(&root);

    // The `Bash(...)` and `PowerShell(...)` pair differ only by their `if`
    // rule. With no test command to render, dropping both rules leaves two
    // identical handlers, which spawns the process twice for one tool call.
    let shell = s["hooks"]["PostToolUse"][1]["hooks"]
        .as_array()
        .expect("the shell handlers");
    assert_eq!(
        shell.len(),
        1,
        "one handler, not two identical ones: {shell:?}"
    );
    assert!(shell[0].get("if").is_none());
    assert_eq!(
        shell[0]["async"], true,
        "it is still the async partial gate"
    );
}

#[test]
fn the_shell_handlers_stay_a_pair_when_the_rule_renders() {
    let (_t, root) = project();
    let mut cfg = devforgeai::config::Config {
        degraded: false,
        ..Default::default()
    };
    cfg.stack = vec![devforgeai::config::Stack {
        id: "rust".into(),
        test_command: "cargo test".into(),
        ..Default::default()
    }];
    std::fs::write(
        root.join(".devforgeai").join("config.toml"),
        devforgeai::config::to_toml(&cfg).expect("render config"),
    )
    .expect("write config");

    hooks::install(&root, false, true, false).expect("install");
    let s = read_settings(&root);

    // Two distinct rules, two handlers: the shells are named separately
    // because a matcher cannot express both.
    let shell = s["hooks"]["PostToolUse"][1]["hooks"]
        .as_array()
        .expect("the shell handlers");
    assert_eq!(shell.len(), 2, "{shell:?}");
    assert_eq!(shell[0]["if"], "Bash(cargo test)");
    assert_eq!(shell[1]["if"], "PowerShell(cargo test)");
}
