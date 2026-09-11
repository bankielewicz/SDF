//! `init`: the directory tree, the three project files, the framework copy,
//! the `.gitignore` lines, and the hooks.
//!
//! Every test runs against its own `TempDir` and passes `--project` explicitly,
//! so nothing resolves the real working tree. The one test that must not find a
//! real pin redirects `DEVFORGEAI_HOME` under `common::ENV_LOCK`.

mod common;

use common::Project;
use devforgeai::cli::InitArgs;
use devforgeai::cmd::init;
use devforgeai::Outcome;
use std::path::{Path, PathBuf};

// --- fixtures ------------------------------------------------------------

/// A framework root holding two skills, three agents and one command, so the
/// copy counts in the `data` object are assertable.
struct Framework {
    /// Kept alive so the tree outlives the test.
    dir: tempfile::TempDir,
}

impl Framework {
    fn new() -> Framework {
        let dir = tempfile::tempdir().expect("temp framework");
        let f = Framework { dir };
        // Claude Code derives the slash command from the directory name, so
        // the fixture carries the frontmatter `name:` the install renames to.
        f.write(
            "skills/exploring-ideas/SKILL.md",
            "---
name: explore
description: Explore an idea.
---

# Exploring ideas
",
        );
        f.write(
            "skills/planning-work/SKILL.md",
            "---
name: plan
description: Plan the work.
---

# Planning work
",
        );
        f.write("agents/kill-case-builder.md", "# Kill case builder\n");
        f.write(
            "agents/architecture-reviewer.md",
            "# Architecture reviewer\n",
        );
        f.write("agents/code-reviewer.md", "# Code reviewer\n");
        f.write("commands/explore.md", "# /explore\n");
        f
    }

    fn root(&self) -> &Path {
        self.dir.path()
    }

    fn write(&self, rel: &str, body: &str) {
        let path = self.dir.path().join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("mkdir");
        }
        std::fs::write(&path, body).expect("write");
    }
}

/// The default arguments: `--from` the fixture framework, hooks skipped.
fn args(from: &Framework) -> InitArgs {
    InitArgs {
        analyze: false,
        force: false,
        from: Some(from.root().to_path_buf()),
        no_hooks: true,
    }
}

/// Run `init` against a bare project, expecting success.
fn run(p: &Project, a: &InitArgs) -> Outcome {
    init::run(Some(p.root()), a).unwrap_or_else(|e| panic!("init runs: {} {}", e.code(), e.exit()))
}

/// A bare project holding a Rust marker, so `stack detect` has something to
/// find and `config.toml` lands non-degraded.
fn rust_project() -> Project {
    let p = Project::bare();
    p.write("Cargo.toml", "[package]\nname = \"acme\"\n");
    p
}

/// `DEVFORGEAI_HOME` pointed at an empty temporary directory, restored on drop.
///
/// `init` falls back to the `framework_path` of the matching `[[pin]]` when
/// `--from` is absent, so the test for that error must be sure no real
/// `trust.toml` is in reach.
struct EmptyTrustHome {
    /// Serialises the environment write against the rest of the binary.
    _lock: std::sync::MutexGuard<'static, ()>,
    /// Kept alive so the directory outlives the guard.
    _home: tempfile::TempDir,
    /// What the variable held before, put back on drop.
    previous: Option<std::ffi::OsString>,
}

impl EmptyTrustHome {
    fn new() -> EmptyTrustHome {
        let lock = common::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let previous = std::env::var_os("DEVFORGEAI_HOME");
        let home = tempfile::tempdir().expect("temp home");
        std::env::set_var("DEVFORGEAI_HOME", home.path());
        EmptyTrustHome {
            _lock: lock,
            _home: home,
            previous,
        }
    }
}

impl Drop for EmptyTrustHome {
    fn drop(&mut self) {
        match self.previous.take() {
            Some(v) => std::env::set_var("DEVFORGEAI_HOME", v),
            None => std::env::remove_var("DEVFORGEAI_HOME"),
        }
    }
}

/// Initialise a git repository, returning false when git is not on PATH.
fn git_init(root: &Path) -> bool {
    match std::process::Command::new("git")
        .args(["init"])
        .current_dir(root)
        .output()
    {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

/// The three git hook paths `hook install` writes.
fn git_hook_paths(root: &Path) -> Vec<PathBuf> {
    ["pre-commit", "commit-msg", "pre-push"]
        .iter()
        .map(|n| root.join(".git").join("hooks").join(n))
        .collect()
}

// --- the tree and the three files ----------------------------------------

#[test]
fn init_creates_directory_tree() {
    let f = Framework::new();
    let p = rust_project();
    let out = run(&p, &args(&f));

    assert!(p.exists(".devforgeai"), ".devforgeai/ exists");
    for sub in devforgeai::project::SUBDIRS {
        assert!(
            p.root().join(".devforgeai").join(sub).is_dir(),
            ".devforgeai/{sub} exists"
        );
    }
    let created: Vec<&str> = out.data["created"]
        .as_array()
        .expect("created is an array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(created.contains(&".devforgeai"), "{created:?}");
    assert!(created.contains(&".devforgeai/reports"), "{created:?}");
    assert!(
        created.contains(&"CLAUDE.md"),
        "the fixture had no CLAUDE.md, so init created one: {created:?}"
    );
    assert_eq!(
        created.len(),
        devforgeai::project::SUBDIRS.len() + 2,
        "the root, one entry per subdirectory, and CLAUDE.md: {created:?}"
    );
}

#[test]
fn init_writes_default_gates() {
    let f = Framework::new();
    let p = rust_project();
    run(&p, &args(&f));

    let written = std::fs::read(p.root().join(".devforgeai").join("gates.toml")).expect("read");
    assert_eq!(
        written,
        devforgeai::gates::DEFAULT_GATES.as_bytes(),
        "gates.toml is the compiled default, byte for byte"
    );
}

#[test]
fn init_writes_initial_state() {
    let f = Framework::new();
    let p = rust_project();
    run(&p, &args(&f));

    let s = devforgeai::state::load(p.root()).expect("state.toml parses");
    assert_eq!(s.current.phase, "explore");
    assert_eq!(s.current.id, "");
    assert_eq!(s.last_gate.result, "NOT_RUN");
    assert_eq!(s.stop_hook.block_count, 0);
    for phase in [
        "explore",
        "discover",
        "constitute",
        "plan",
        "build",
        "verify",
        "release",
    ] {
        assert_eq!(s.active.get(phase), Some(""), "[active].{phase} is empty");
    }
}

#[test]
fn init_runs_stack_detect() {
    let f = Framework::new();
    let p = rust_project();
    let out = run(&p, &args(&f));

    assert!(p.exists(".devforgeai/config.toml"), "stack detect wrote it");
    let stacks: Vec<&str> = out.data["stacks"]
        .as_array()
        .expect("stacks is an array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(stacks, vec!["rust"], "the Cargo.toml marker matched");
    assert_eq!(out.data["degraded"], false);
    assert!(!out.degraded, "the envelope agrees");
    assert!(
        out.human.iter().any(|l| l.starts_with("Stack      rust")),
        "the human block names the stack: {:?}",
        out.human
    );
}

// --- the framework copy --------------------------------------------------

#[test]
fn init_json_envelope_lists_counts() {
    let f = Framework::new();
    let p = rust_project();
    let out = run(&p, &args(&f));

    assert_eq!(
        out.data["copied"]["skills"], 2,
        "the evals subtree is not copied"
    );
    assert_eq!(out.data["copied"]["agents"], 3);
    assert!(
        out.data["copied"].get("commands").is_none(),
        "commands/ is gone: each entry point is its own skill"
    );
    assert_eq!(out.data["hooks"]["settings"], "skipped");
    assert!(out.data["analyze"].is_null(), "no --analyze, no analysis");
    assert!(p.exists(".claude/skills/explore/SKILL.md"));
    assert!(p.exists(".claude/agents/code-reviewer.md"));
    assert!(!p.exists(".claude/commands/explore.md"));
    assert!(
        !p.exists(".claude/skills/explore/evals/cases.jsonl"),
        "the framework's own eval suite is not shipped into a target project"
    );
    assert!(
        out.human
            .iter()
            .any(|l| l == "Copied     2 skills, 3 agents"),
        "the human block reports the counts: {:?}",
        out.human
    );
}

// --- the hooks -----------------------------------------------------------

#[test]
fn init_merges_settings_hooks() {
    let f = Framework::new();
    let p = rust_project();
    let a = InitArgs {
        no_hooks: false,
        ..args(&f)
    };
    let out = run(&p, &a);

    assert_eq!(out.data["hooks"]["settings"], "merged");
    let text = p.read(".claude/settings.json");
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("settings.json is JSON");
    for event in [
        "SessionStart",
        "PreToolUse",
        "PostToolUse",
        "Stop",
        "SubagentStop",
    ] {
        assert!(
            parsed["hooks"][event].is_array(),
            "{event} is installed: {text}"
        );
    }
}

#[test]
fn init_installs_three_git_hooks() {
    let f = Framework::new();
    let p = rust_project();
    if !git_init(p.root()) {
        eprintln!("git is not on PATH; skipping the git hook assertions");
        return;
    }

    let a = InitArgs {
        no_hooks: false,
        ..args(&f)
    };
    let out = run(&p, &a);

    let installed: Vec<&str> = out.data["hooks"]["git"]
        .as_array()
        .expect("git is an array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(installed, vec!["pre-commit", "commit-msg", "pre-push"]);

    for path in git_hook_paths(p.root()) {
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let text = String::from_utf8_lossy(&bytes);
        assert!(
            text.contains("# devforgeai-hook v1"),
            "{} carries the marker",
            path.display()
        );
        assert!(
            !bytes.windows(2).any(|w| w == b"\r\n"),
            "{} is written with LF endings",
            path.display()
        );
    }
}

#[test]
fn init_without_git_warns_e130_and_exits_zero() {
    let f = Framework::new();
    let p = rust_project();
    let a = InitArgs {
        no_hooks: false,
        ..args(&f)
    };
    let out = run(&p, &a);

    assert_eq!(
        out.exit.unwrap_or(0),
        0,
        "a project without git still initialises"
    );
    assert!(
        out.warnings.iter().any(|w| w.code == "DFA-E130"),
        "the missing git directory is reported: {:?}",
        out.warnings
    );
    assert_eq!(
        out.data["hooks"]["git"].as_array().map(Vec::len),
        Some(0),
        "no git hooks were written"
    );
    // The settings merge is independent of git and still happened.
    assert_eq!(out.data["hooks"]["settings"], "merged");

    // The human line names the empty case rather than trailing off after the
    // label. The spec's `## CLI calls` block shows the three-hook case alone,
    // so `none` follows the conventions section 6 handoff convention for an
    // empty field.
    let hooks_line = out
        .human
        .iter()
        .find(|l| l.starts_with("Hooks "))
        .expect("init prints a Hooks line");
    assert_eq!(
        hooks_line, "Hooks      .claude/settings.json merged; git hooks none",
        "an empty git-hook list reads 'none'"
    );
}

#[test]
fn init_no_hooks_skips_both() {
    let f = Framework::new();
    let p = rust_project();
    let out = run(&p, &args(&f));

    assert_eq!(out.data["hooks"]["settings"], "skipped");
    assert_eq!(out.data["hooks"]["git"].as_array().map(Vec::len), Some(0));
    assert!(
        !p.exists(".claude/settings.json"),
        "the settings file was never created"
    );
    assert!(
        !p.root().join(".git").join("hooks").exists(),
        "no hooks directory was created"
    );
    assert!(
        out.human.iter().all(|l| !l.starts_with("Hooks")),
        "the human block omits the hooks line: {:?}",
        out.human
    );
}

// --- .gitignore ----------------------------------------------------------

#[test]
fn init_appends_gitignore_lines() {
    let f = Framework::new();
    let p = rust_project();
    p.write(".gitignore", "target/\n");
    run(&p, &args(&f));

    let text = p.read(".gitignore");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(
        lines,
        vec!["target/", ".explore-prototype/", ".devforgeai/state.toml"],
        "the two lines are appended after what was there"
    );

    // A second run adds nothing: the lines are already present.
    let again = InitArgs {
        force: true,
        ..args(&f)
    };
    run(&p, &again);
    assert_eq!(p.read(".gitignore"), text, "no duplicate lines");
}

// --- the CLAUDE.md section -----------------------------------------------

/// The text between the markers, exclusive of neither.
fn claude_md_block(p: &common::Project) -> String {
    let text = p.read("CLAUDE.md");
    let start = text
        .find("<!-- devforgeai:begin -->")
        .expect("the begin marker");
    let end = text
        .find("<!-- devforgeai:end -->")
        .expect("the end marker")
        + "<!-- devforgeai:end -->".len();
    text[start..end].to_string()
}

#[test]
fn init_creates_claude_md_when_absent() {
    let f = Framework::new();
    let p = rust_project();
    assert!(!p.exists("CLAUDE.md"));
    run(&p, &args(&f));

    assert!(p.exists("CLAUDE.md"), "init writes the section");
    let text = p.read("CLAUDE.md");
    assert_eq!(text.matches("<!-- devforgeai:begin -->").count(), 1);
    assert_eq!(text.matches("<!-- devforgeai:end -->").count(), 1);
    assert!(
        text.starts_with("# "),
        "an absent file is created under an H1 naming the project: {text}"
    );
    // Short enough to sit inside the 200-line CLAUDE.md budget beside the
    // project's own material.
    let lines = claude_md_block(&p).lines().count();
    assert!(
        (25..=40).contains(&lines),
        "the block is one screen, not a manual: {lines} lines"
    );
}

#[test]
fn init_appends_block_to_existing_claude_md() {
    let f = Framework::new();
    let p = rust_project();
    p.write("CLAUDE.md", "# Acme\n\nRun the tests with `make check`.\n");
    run(&p, &args(&f));

    let text = p.read("CLAUDE.md");
    assert!(text.starts_with("# Acme\n"), "the user's text is first");
    assert!(
        text.contains("Run the tests with `make check`."),
        "the user's own line survives byte for byte: {text}"
    );
    let begin = text.find("<!-- devforgeai:begin -->").expect("marker");
    assert!(
        begin > text.find("make check").expect("the user's line"),
        "the block follows what was there"
    );
}

#[test]
fn init_twice_replaces_between_markers() {
    let f = Framework::new();
    let p = rust_project();
    p.write("CLAUDE.md", "# Acme\n\nMine.\n");
    run(&p, &args(&f));
    let first = p.read("CLAUDE.md");

    // Appending the user's own text after the block, then re-running, is the
    // case the replace-between-markers rule exists for.
    p.write(
        "CLAUDE.md",
        &format!("{first}\nA line of my own after it.\n"),
    );
    run(
        &p,
        &InitArgs {
            force: true,
            ..args(&f)
        },
    );

    let text = p.read("CLAUDE.md");
    assert_eq!(text.matches("<!-- devforgeai:begin -->").count(), 1);
    assert_eq!(text.matches("<!-- devforgeai:end -->").count(), 1);
    assert!(text.contains("Mine."), "text before the block survives");
    assert!(
        text.contains("A line of my own after it."),
        "text after the block survives: {text}"
    );
}

#[test]
fn init_claude_md_block_names_nine_commands() {
    let f = Framework::new();
    let p = rust_project();
    run(&p, &args(&f));

    let block = claude_md_block(&p);
    for name in [
        "/explore",
        "/discover",
        "/constitute",
        "/plan",
        "/build",
        "/verify",
        "/release",
        "/design",
        "/reflect",
    ] {
        assert_eq!(
            block.matches(&format!("`{name}")).count(),
            1,
            "{name} appears once in the block"
        );
    }
    assert!(
        block.contains("Stop hook"),
        "the block says where the handoff comes from"
    );
}

#[test]
fn init_json_envelope_lists_claude_md() {
    let f = Framework::new();
    let p = rust_project();

    let first = run(&p, &args(&f));
    let created = |out: &devforgeai::Outcome| -> Vec<String> {
        out.data["created"]
            .as_array()
            .expect("created")
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect()
    };
    assert!(created(&first).contains(&"CLAUDE.md".to_string()));

    let second = run(
        &p,
        &InitArgs {
            force: true,
            ..args(&f)
        },
    );
    assert!(
        !created(&second).contains(&"CLAUDE.md".to_string()),
        "the second run updates rather than creates"
    );
    assert!(
        second
            .human
            .iter()
            .any(|l| l == "CLAUDE.md  devforgeai section updated"),
        "the summary says which: {:?}",
        second.human
    );
}

// --- refusals ------------------------------------------------------------

#[test]
fn init_twice_gives_e110() {
    let f = Framework::new();
    let p = rust_project();
    run(&p, &args(&f));

    let err = init::run(Some(p.root()), &args(&f)).expect_err("the project is initialised");
    assert_eq!(err.code(), "DFA-E110");
    assert_eq!(err.exit(), 1);
}

#[test]
fn init_force_overwrites() {
    let f = Framework::new();
    let p = rust_project();
    run(&p, &args(&f));

    p.write(".devforgeai/gates.toml", "# hand edited\n");
    let a = InitArgs {
        force: true,
        ..args(&f)
    };
    run(&p, &a);

    let written = std::fs::read(p.root().join(".devforgeai").join("gates.toml")).expect("read");
    assert_eq!(
        written,
        devforgeai::gates::DEFAULT_GATES.as_bytes(),
        "--force replaced the edited file"
    );
}

#[test]
fn init_missing_framework_gives_e111() {
    let _home = EmptyTrustHome::new();
    let p = rust_project();
    let a = InitArgs {
        analyze: false,
        force: false,
        from: None,
        no_hooks: true,
    };

    let err = init::run(Some(p.root()), &a).expect_err("no --from and no pin");
    assert_eq!(err.code(), "DFA-E111");
    assert_eq!(err.exit(), 1);
    assert!(
        !p.exists(".devforgeai"),
        "nothing is written on the error path"
    );
}

#[test]
fn init_analyze_drafts_three_files_and_stubs_three() {
    let f = Framework::new();
    let p = rust_project();
    let a = InitArgs {
        analyze: true,
        ..args(&f)
    };

    let out = init::run(Some(p.root()), &a).expect("the brownfield analysis runs");
    let analyze = &out.data["analyze"];
    assert_eq!(
        analyze["derived"],
        serde_json::json!(["tech-stack", "source-tree", "dependencies"])
    );
    assert_eq!(
        analyze["stubs"],
        serde_json::json!([
            "coding-standards",
            "architecture-constraints",
            "anti-patterns"
        ])
    );
    assert_eq!(analyze["skipped"], serde_json::json!([]));
    assert!(analyze["files_scanned"].as_u64().expect("a count") > 0);
    assert_eq!(analyze["truncated"], false);

    for stem in devforgeai::doc::CONTEXT_STEMS {
        let text = std::fs::read_to_string(
            p.root()
                .join(".devforgeai")
                .join("context")
                .join(format!("{stem}.md")),
        )
        .unwrap_or_else(|e| panic!("reading {stem}: {e}"));
        assert!(
            text.starts_with(
                "---
"
            ),
            "{stem} opens with frontmatter"
        );
        assert!(text.contains("status: draft"), "{stem} is a draft");
        assert!(
            text.contains(&format!("schema: devforgeai/context-{stem}/1")),
            "{stem} carries its schema"
        );
        assert!(
            text.contains("produced_by: establishing-context"),
            "{stem} names the skill, not the CLI"
        );
    }

    assert!(
        out.human
            .iter()
            .any(|l| l.starts_with("Analysis   3 context files drafted, 3 stubbed")),
        "human was {:?}",
        out.human
    );
}

// --- skills install under their slash names -------------------------------

#[test]
fn init_installs_each_skill_under_its_slash_name() {
    let f = Framework::new();
    let p = rust_project();
    let out = run(&p, &args(&f));

    // Claude Code derives a project skill's command from its directory name,
    // so a skill installed at `exploring-ideas/` offers `/exploring-ideas`.
    // The framework keeps the long directory name, because that is what
    // `produced_by:` resolves against, and the install renames to the command.
    assert!(p.exists(".claude/skills/explore/SKILL.md"));
    assert!(p.exists(".claude/skills/plan/SKILL.md"));
    assert!(
        !p.exists(".claude/skills/exploring-ideas"),
        "the long directory name would register a second slash command"
    );
    assert!(!p.exists(".claude/skills/planning-work"));

    assert_eq!(
        out.data["commands"],
        serde_json::json!(["explore", "plan"]),
        "the envelope names the commands the install produced"
    );
    assert!(
        out.human.iter().any(|l| l == "Commands   /explore, /plan"),
        "the summary names them too: {:?}",
        out.human
    );
}

#[test]
fn init_removes_a_skill_installed_under_its_old_directory_name() {
    let f = Framework::new();
    let p = rust_project();
    // An install made before the rename.
    p.write(
        ".claude/skills/exploring-ideas/SKILL.md",
        "---\nname: explore\ndescription: Explore an idea.\n---\n\n# Old\n",
    );

    run(&p, &args(&f));

    assert!(p.exists(".claude/skills/explore/SKILL.md"));
    assert!(
        !p.exists(".claude/skills/exploring-ideas"),
        "both would register, so the same skill would offer two commands"
    );
}

#[test]
fn init_keeps_a_foreign_skill_that_shares_a_directory_name() {
    let f = Framework::new();
    let p = rust_project();
    // A skill of the project's own that happens to sit at the same path and
    // claims a different command.
    p.write(
        ".claude/skills/exploring-ideas/SKILL.md",
        "---\nname: my-own-thing\ndescription: Mine.\n---\n\n# Mine\n",
    );

    run(&p, &args(&f));

    assert!(
        p.exists(".claude/skills/exploring-ideas/SKILL.md"),
        "only a directory whose own SKILL.md claims the installed name is removed"
    );
    assert!(p
        .read(".claude/skills/exploring-ideas/SKILL.md")
        .contains("my-own-thing"));
}

#[test]
fn init_falls_back_to_the_directory_name_when_the_frontmatter_has_none() {
    let f = Framework::new();
    f.write("skills/no-name/SKILL.md", "# No frontmatter here\n");
    let p = rust_project();

    run(&p, &args(&f));

    assert!(p.exists(".claude/skills/no-name/SKILL.md"));
}
