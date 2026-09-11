//! `worktree ensure|list|remove` and `commit`, driven against a real git
//! repository created inside a temporary directory.
//!
//! Identity and signing are passed on every `git` command line and
//! `DEVFORGEAI_HOME` is set per invocation, so no test reads or writes the
//! machine's git configuration or the real home directory.

mod common;

use assert_cmd::Command;
use common::Project;

/// A temporary home for `trust.toml`, so an installed hook's `trust verify`
/// reads a fixture rather than the real `~/.devforgeai`.
struct Home(tempfile::TempDir);

impl Home {
    fn new() -> Home {
        let dir = tempfile::tempdir().expect("temp home");
        // The child's home is redirected here, and git reads its identity from
        // `~/.gitconfig`, so a home with no git config leaves `devforgeai
        // commit` unable to commit. Writing the identity here rather than
        // falling back to the machine's own config keeps the isolation whole:
        // no test reads the developer's git configuration either.
        std::fs::write(
            dir.path().join(".gitconfig"),
            "[user]\n\tname = devforgeai test\n\temail = test@example.invalid\n[commit]\n\tgpgsign = false\n",
        )
        .expect("write .gitconfig");
        Home(dir)
    }
    fn path(&self) -> &std::path::Path {
        self.0.path()
    }
}

fn cli(p: &Project, home: &Home) -> Command {
    let mut c = Command::cargo_bin("devforgeai").expect("the binary builds");
    // The `test-home` feature is unified on for *test targets* through the
    // dev-dependency, and this is not one: `cargo test` builds `src/main.rs`
    // with default features, where `DEVFORGEAI_HOME` is read by nothing. A
    // child told only through that variable therefore resolved the real
    // `~/.devforgeai` and wrote a pin into the developer's own trust store.
    //
    // The redirect the binary does honour is the platform home, which is what
    // `trust::user_home` reads with no feature gate, so that is what the child
    // gets. `DEVFORGEAI_HOME` stays for the in-process helpers.
    c.env("DEVFORGEAI_HOME", home.path());
    c.env("USERPROFILE", home.path());
    c.env("HOME", home.path());
    // `trust pin` refuses inside a Claude session; the test harness may be one.
    for var in devforgeai::trust::SESSION_VARS {
        c.env_remove(var);
    }
    c.arg("--project").arg(p.root());
    c.current_dir(p.root());
    c
}

fn run(p: &Project, home: &Home, args: &[&str]) -> (i32, String, String) {
    let out = cli(p, home).args(args).output().expect("run");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

fn json(p: &Project, home: &Home, args: &[&str]) -> serde_json::Value {
    let out = cli(p, home).arg("--json").args(args).output().expect("run");
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    serde_json::from_str(text.trim()).unwrap_or_else(|e| panic!("{e} in {text}"))
}

/// `git -C <dir>` with identity and signing fixed on the command line.
fn git_in(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    let mut c = std::process::Command::new("git");
    c.arg("-C").arg(dir);
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

fn git(p: &Project, args: &[&str]) -> std::process::Output {
    git_in(p.root(), args)
}

/// A story with a configurable declared file set.
fn story(id: &str, files: &[&str]) -> String {
    let rows: String = files
        .iter()
        .map(|f| format!("| {f} | source | application |\n"))
        .collect();
    format!(
        "---\nschema: devforgeai/story/1\nid: {id}\nphase: plan\nstatus: building\nproduced_by: planning-work\nconsumes: [REQ-001]\nopen_questions: []\n---\n\n# Order checkout\n\n## Requirements\n\n| REQ | Statement | Covered by |\n|---|---|---|\n| REQ-001 | The shopper places an order | AC-001 |\n\n## Acceptance Criteria\n\n- AC-001: Given a cart When the shopper pays Then an order row exists.\n\n## Files\n\n| Path | Kind | Layer |\n|---|---|---|\n{rows}\n## Dependencies\n\nnone\n"
    )
}

/// A project that is a git repository with one commit and one story.
fn repo() -> Project {
    let p = Project::new();
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-001", "accepted"),
    );
    p.write(
        ".devforgeai/stories/STORY-014.md",
        &story("STORY-014", &["src/place_order.rs"]),
    );
    p.set_phase("build", "STORY-014");

    // The base ref and the worktree holder, fixed before the first commit so
    // the committed tree and the work tree agree.
    let mut cfg = devforgeai::config::load(p.root()).expect("config");
    cfg.build.base_ref = "main".into();
    cfg.build.worktree_root = "wt".into();
    p.write_config(&cfg);

    let out = std::process::Command::new("git")
        .args(["init", "-b", "main"])
        .arg(p.root())
        .output()
        .expect("git init");
    assert!(out.status.success(), "git init: {out:?}");
    p.write("README.md", "base\n");
    p.write(".gitignore", "wt/\n");
    git(&p, &["add", "-A"]);
    let out = git(&p, &["commit", "-m", "base"]);
    assert!(out.status.success(), "first commit: {out:?}");
    p
}

// -------------------------------------------------------------- worktree ensure

#[test]
fn ensure_creates_the_worktree_and_seeds_the_state_file() {
    let p = repo();
    let h = Home::new();
    let (code, out, err) = run(&p, &h, &["worktree", "ensure", "STORY-014"]);
    assert_eq!(code, 0, "stderr was {err}");
    assert_eq!(out, "wt/STORY-014\n");
    assert!(p.exists("wt/STORY-014"), "the directory exists");
    assert!(
        p.exists("wt/STORY-014/.devforgeai/state.toml"),
        "the state file is seeded"
    );

    let branches = String::from_utf8_lossy(&git(&p, &["branch", "--list"]).stdout).to_string();
    assert!(branches.contains("story/STORY-014"), "branches: {branches}");
}

#[test]
fn ensure_json_is_the_shape_the_spec_fixes() {
    let p = repo();
    let h = Home::new();
    let v = json(&p, &h, &["worktree", "ensure", "STORY-014"]);
    assert_eq!(v["data"]["id"], "STORY-014");
    assert_eq!(v["data"]["path"], "wt/STORY-014");
    assert_eq!(v["data"]["branch"], "story/STORY-014");
    assert_eq!(v["data"]["created"], true);
    assert_eq!(v["data"]["state_seeded"], true);
}

#[test]
fn ensure_is_idempotent() {
    let p = repo();
    let h = Home::new();
    assert_eq!(run(&p, &h, &["worktree", "ensure", "STORY-014"]).0, 0);
    let v = json(&p, &h, &["worktree", "ensure", "STORY-014"]);
    assert_eq!(v["exit"], 0);
    assert_eq!(
        v["data"]["created"], false,
        "the second call creates nothing"
    );
}

#[test]
fn ensure_refuses_an_overlapping_file_set_with_e272() {
    let p = repo();
    let h = Home::new();
    p.write(
        ".devforgeai/stories/STORY-015.md",
        &story("STORY-015", &["src/place_order.rs", "src/tax.rs"]),
    );
    assert_eq!(run(&p, &h, &["worktree", "ensure", "STORY-014"]).0, 0);

    let (code, _, err) = run(&p, &h, &["worktree", "ensure", "STORY-015"]);
    assert_eq!(code, 1, "stderr was {err}");
    assert!(err.contains("DFA-E272"), "stderr was {err}");
    assert!(err.contains("STORY-014"), "stderr was {err}");
    assert!(err.contains("STORY-015"), "stderr was {err}");
    assert!(err.contains("src/place_order.rs"), "stderr was {err}");
}

#[test]
fn ensure_allows_a_disjoint_file_set() {
    let p = repo();
    let h = Home::new();
    p.write(
        ".devforgeai/stories/STORY-015.md",
        &story("STORY-015", &["src/tax.rs"]),
    );
    assert_eq!(run(&p, &h, &["worktree", "ensure", "STORY-014"]).0, 0);
    let (code, _, err) = run(&p, &h, &["worktree", "ensure", "STORY-015"]);
    assert_eq!(code, 0, "stderr was {err}");
}

#[test]
fn ensure_outside_a_repository_is_e271() {
    let p = Project::new();
    let h = Home::new();
    p.write(
        ".devforgeai/stories/STORY-014.md",
        &story("STORY-014", &["src/a.rs"]),
    );
    let (code, _, err) = run(&p, &h, &["worktree", "ensure", "STORY-014"]);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E271"), "stderr was {err}");
}

#[test]
fn a_malformed_story_id_is_refused_at_exit_three() {
    let p = repo();
    let h = Home::new();
    let (code, _, err) = run(&p, &h, &["worktree", "ensure", "STORY-14"]);
    assert_eq!(code, 3, "stderr was {err}");
    assert!(err.contains("DFA-E012"), "stderr was {err}");
}

// ---------------------------------------------------------------- worktree list

#[test]
fn list_prints_one_line_per_worktree() {
    let p = repo();
    let h = Home::new();
    run(&p, &h, &["worktree", "ensure", "STORY-014"]);

    let (code, out, err) = run(&p, &h, &["worktree", "list"]);
    assert_eq!(code, 0, "stderr was {err}");
    assert_eq!(
        out, "STORY-014  wt/STORY-014  story/STORY-014  clean  0 ahead\n",
        "one line per worktree"
    );
}

#[test]
fn list_reports_a_dirty_worktree_and_its_commit_count() {
    let p = repo();
    let h = Home::new();
    run(&p, &h, &["worktree", "ensure", "STORY-014"]);
    let wt = p.root().join("wt").join("STORY-014");
    std::fs::write(wt.join("scratch.txt"), "x\n").expect("write");

    let v = json(&p, &h, &["worktree", "list"]);
    assert_eq!(v["data"]["worktrees"][0]["dirty"], true);
    assert_eq!(v["data"]["worktrees"][0]["ahead"], 0);

    git_in(&wt, &["add", "-A"]);
    assert!(git_in(&wt, &["commit", "-m", "STORY-014 scratch"])
        .status
        .success());
    let v = json(&p, &h, &["worktree", "list"]);
    assert_eq!(v["data"]["worktrees"][0]["dirty"], false);
    assert_eq!(v["data"]["worktrees"][0]["ahead"], 1);
}

#[test]
fn list_with_no_worktree_is_an_empty_list_at_exit_zero() {
    let p = repo();
    let h = Home::new();
    let (code, out, _) = run(&p, &h, &["worktree", "list"]);
    assert_eq!(code, 0);
    assert!(out.is_empty(), "stdout was {out}");
    assert_eq!(
        json(&p, &h, &["worktree", "list"])["data"]["worktrees"],
        serde_json::json!([])
    );
}

// -------------------------------------------------------------- worktree remove

#[test]
fn remove_takes_a_clean_worktree_and_its_branch() {
    let p = repo();
    let h = Home::new();
    run(&p, &h, &["worktree", "ensure", "STORY-014"]);

    let v = json(&p, &h, &["worktree", "remove", "STORY-014"]);
    assert_eq!(v["exit"], 0);
    assert_eq!(v["data"]["removed"], true);
    assert_eq!(v["data"]["branch_deleted"], true);
    assert!(!p.exists("wt/STORY-014"));
    let branches = String::from_utf8_lossy(&git(&p, &["branch", "--list"]).stdout).to_string();
    assert!(
        !branches.contains("story/STORY-014"),
        "branches: {branches}"
    );
}

#[test]
fn remove_refuses_uncommitted_work_with_e273() {
    let p = repo();
    let h = Home::new();
    run(&p, &h, &["worktree", "ensure", "STORY-014"]);
    std::fs::write(p.root().join("wt/STORY-014/scratch.txt"), "x\n").expect("write");

    let (code, _, err) = run(&p, &h, &["worktree", "remove", "STORY-014"]);
    assert_eq!(code, 1, "stderr was {err}");
    assert!(err.contains("DFA-E273"), "stderr was {err}");
    assert!(err.contains("pass --force"), "stderr was {err}");
    assert!(p.exists("wt/STORY-014"), "the worktree survives");
}

#[test]
fn remove_refuses_commits_absent_from_the_base_ref() {
    let p = repo();
    let h = Home::new();
    run(&p, &h, &["worktree", "ensure", "STORY-014"]);
    let wt = p.root().join("wt").join("STORY-014");
    std::fs::write(wt.join("scratch.txt"), "x\n").expect("write");
    git_in(&wt, &["add", "-A"]);
    assert!(git_in(&wt, &["commit", "-m", "STORY-014 scratch"])
        .status
        .success());

    let (code, _, err) = run(&p, &h, &["worktree", "remove", "STORY-014"]);
    assert_eq!(code, 1, "stderr was {err}");
    assert!(err.contains("DFA-E273"), "stderr was {err}");
    assert!(
        err.contains("1 commits absent from main"),
        "stderr was {err}"
    );
}

#[test]
fn force_removes_despite_unfinished_work() {
    let p = repo();
    let h = Home::new();
    run(&p, &h, &["worktree", "ensure", "STORY-014"]);
    std::fs::write(p.root().join("wt/STORY-014/scratch.txt"), "x\n").expect("write");

    let (code, _, err) = run(&p, &h, &["worktree", "remove", "STORY-014", "--force"]);
    assert_eq!(code, 0, "stderr was {err}");
    assert!(!p.exists("wt/STORY-014"));
}

#[test]
fn removing_an_absent_worktree_is_success() {
    let p = repo();
    let h = Home::new();
    let v = json(&p, &h, &["worktree", "remove", "STORY-014"]);
    assert_eq!(v["exit"], 0);
    assert_eq!(v["data"]["removed"], false);
}

// ------------------------------------------------------------------- commit

#[test]
fn commit_stages_the_declared_paths_and_prefixes_the_message() {
    let p = repo();
    let h = Home::new();
    p.write("src/place_order.rs", "fn place() {}\n");

    let (code, out, err) = run(&p, &h, &["commit", "STORY-014", "-m", "AC-001 green"]);
    assert_eq!(code, 0, "stderr was {err}");
    assert!(out.starts_with("Commit    "), "stdout was {out}");
    assert!(out.contains("STORY-014: AC-001 green"), "stdout was {out}");

    let log = String::from_utf8_lossy(&git(&p, &["log", "-1", "--pretty=%s"]).stdout).to_string();
    assert_eq!(log.trim(), "STORY-014: AC-001 green");
}

#[test]
fn commit_json_is_the_shape_the_spec_fixes() {
    let p = repo();
    let h = Home::new();
    p.write("src/place_order.rs", "fn place() {}\n");

    let v = json(&p, &h, &["commit", "STORY-014", "-m", "AC-001 green"]);
    assert_eq!(v["data"]["id"], "STORY-014");
    assert_eq!(v["data"]["message"], "STORY-014: AC-001 green");
    assert_eq!(v["data"]["staged"], 1);
    assert!(
        v["data"]["commit"].as_str().expect("sha").len() >= 7,
        "a short sha"
    );
}

#[test]
fn a_message_already_carrying_an_id_is_committed_verbatim() {
    let p = repo();
    let h = Home::new();
    p.write("src/place_order.rs", "fn place() {}\n");
    run(&p, &h, &["commit", "STORY-014", "-m", "ADR-004 the choice"]);
    let log = String::from_utf8_lossy(&git(&p, &["log", "-1", "--pretty=%s"]).stdout).to_string();
    assert_eq!(log.trim(), "ADR-004 the choice");
}

#[test]
fn commit_refuses_an_undeclared_path_with_e239_and_stages_nothing() {
    let p = repo();
    let h = Home::new();
    p.write("src/sneaky.rs", "fn sneak() {}\n");

    let (code, _, err) = run(&p, &h, &["commit", "STORY-014", "-m", "AC-001 green"]);
    assert_eq!(code, 1, "stderr was {err}");
    assert!(err.contains("DFA-E239"), "stderr was {err}");
    let staged = String::from_utf8_lossy(&git(&p, &["diff", "--cached", "--name-only"]).stdout)
        .trim()
        .to_string();
    assert!(staged.is_empty(), "nothing staged, was {staged}");
}

#[test]
fn an_empty_stage_set_is_w243_at_exit_zero() {
    let p = repo();
    let h = Home::new();
    let (code, _, err) = run(&p, &h, &["commit", "STORY-014", "-m", "AC-001 green"]);
    assert_eq!(code, 0, "stderr was {err}");
    assert!(err.contains("DFA-W243"), "stderr was {err}");
}

#[test]
fn explicit_paths_are_staged_and_checked() {
    let p = repo();
    let h = Home::new();
    p.write("src/place_order.rs", "fn place() {}\n");
    p.write("src/sneaky.rs", "fn sneak() {}\n");

    let (code, _, err) = run(
        &p,
        &h,
        &[
            "commit",
            "STORY-014",
            "-m",
            "AC-001 green",
            "--paths",
            "src/place_order.rs",
        ],
    );
    assert_eq!(
        code, 0,
        "the undeclared file was not named; stderr was {err}"
    );

    let (code, _, err) = run(
        &p,
        &h,
        &[
            "commit",
            "STORY-014",
            "-m",
            "more",
            "--paths",
            "src/sneaky.rs",
        ],
    );
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E239"), "stderr was {err}");
}

#[test]
fn a_devforgeai_path_is_staged_without_being_declared() {
    let p = repo();
    let h = Home::new();
    p.write("src/place_order.rs", "fn place() {}\n");
    p.write(".devforgeai/build/note.yaml", "key: value\n");

    let (code, _, err) = run(&p, &h, &["commit", "STORY-014", "-m", "AC-001 green"]);
    assert_eq!(code, 0, "stderr was {err}");
    let files = String::from_utf8_lossy(
        &git(&p, &["show", "--name-only", "--pretty=format:", "HEAD"]).stdout,
    )
    .to_string();
    assert!(
        files.contains(".devforgeai/build/note.yaml"),
        "files: {files}"
    );
}

#[test]
fn commit_outside_a_repository_is_e271() {
    let p = Project::new();
    let h = Home::new();
    p.write(
        ".devforgeai/stories/STORY-014.md",
        &story("STORY-014", &["src/a.rs"]),
    );
    let (code, _, err) = run(&p, &h, &["commit", "STORY-014", "-m", "AC-001 green"]);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E271"), "stderr was {err}");
}

// ------------------------------------------------------- the installed hooks

/// A framework root whose `cli/DIGEST` is the digest of the binary under test,
/// so `trust pin` accepts it and the hook's `trust verify` passes. The source
/// digest is not checked outside a Claude session, and these tests clear both
/// session variables, so the placeholder on line 2 is enough.
fn fake_framework() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("temp framework");
    let cli = dir.path().join("cli");
    std::fs::create_dir_all(cli.join("src")).expect("mkdir");
    std::fs::write(
        cli.join("Cargo.toml"),
        "[package]
name = \"x\"
",
    )
    .expect("write");
    std::fs::write(
        cli.join("src").join("main.rs"),
        "fn main() {}
",
    )
    .expect("write");
    std::fs::write(
        cli.join("REVISION"),
        format!(
            "{}
sha256:{}
",
            "a".repeat(40),
            "0".repeat(64)
        ),
    )
    .expect("REVISION");

    let exe = assert_cmd::cargo::cargo_bin("devforgeai");
    let digest = devforgeai::trust::digest_file(&exe).expect("the binary digest");
    std::fs::write(
        cli.join("DIGEST"),
        format!(
            "{digest}
"
        ),
    )
    .expect("DIGEST");
    dir
}

/// Install the git hooks and pin the binary, so `trust verify` inside a hook
/// resolves against the temporary home rather than the real one.
fn install_hooks(p: &Project, h: &Home, framework: &std::path::Path) {
    // `trust pin` is the one call in this file that writes outside the project,
    // so the guard sits with it.
    let _guard = common::RealTrustStoreGuard::new();
    let out = cli(p, h)
        .args(["trust", "pin", "--framework"])
        .arg(framework)
        .output()
        .expect("trust pin");
    assert!(out.status.success(), "trust pin: {out:?}");
    let out = cli(p, h)
        .args(["hook", "install"])
        .output()
        .expect("install");
    assert!(
        out.status.success(),
        "hook install: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(p.exists(".git/hooks/pre-commit"), "pre-commit written");
    assert!(p.exists(".git/hooks/commit-msg"), "commit-msg written");
}

#[test]
fn an_installed_hook_runs_and_a_refusal_stops_the_commit() {
    let p = repo();
    let h = Home::new();
    let framework = fake_framework();
    install_hooks(&p, &h, framework.path());

    // A context directory with no files fails `context audit`, so the
    // `pre-commit` hook exits 1 and the commit does not land.
    std::fs::create_dir_all(p.dot().join("context")).expect("mkdir");
    p.write("src/place_order.rs", "fn place() {}\n");

    let before = String::from_utf8_lossy(&git(&p, &["rev-parse", "HEAD"]).stdout).to_string();
    let (code, _, err) = run(
        &p,
        &h,
        &[
            "commit",
            "STORY-014",
            "-m",
            "AC-001 green",
            "--paths",
            "src/place_order.rs",
        ],
    );
    assert_eq!(code, 1, "stderr was {err}");
    assert!(
        err.contains("DFA-E220"),
        "the hook's own stderr reaches stderr; it was {err}"
    );
    let after = String::from_utf8_lossy(&git(&p, &["rev-parse", "HEAD"]).stdout).to_string();
    assert_eq!(before, after, "nothing was committed");
}

#[test]
fn an_installed_hook_lets_a_clean_commit_through() {
    let p = repo();
    let h = Home::new();
    let framework = fake_framework();
    install_hooks(&p, &h, framework.path());
    // The `pre-commit` hook audits the context directory, which `init` creates.
    common::write_context_set(&p, "accepted");
    p.write("src/place_order.rs", "fn place() {}\n");

    let v = json(
        &p,
        &h,
        &[
            "commit",
            "STORY-014",
            "-m",
            "AC-001 green",
            "--paths",
            "src/place_order.rs",
        ],
    );
    assert_eq!(v["exit"], 0, "envelope was {v}");
    assert_eq!(
        v["data"]["hooks"],
        serde_json::json!(["pre-commit", "commit-msg"]),
        "both hooks are installed and ran"
    );
}
