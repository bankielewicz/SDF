//! The global surface, driven through the real binary: the version line, root
//! resolution, the two usage refusals, and one case of each output mode.

mod common;

use assert_cmd::Command;
use common::Project;

fn cli(p: &Project) -> Command {
    let mut c = Command::cargo_bin("devforgeai").expect("the binary builds");
    c.arg("--project").arg(p.root());
    // A resolvable working directory that is not the project, so `--project`
    // is what resolves the root.
    c.current_dir(p.root());
    c
}

// --------------------------------------------------- the surface that is built

#[test]
fn version_prints_and_exits_zero() {
    let mut c = Command::cargo_bin("devforgeai").expect("binary");
    let out = c.arg("--version").output().expect("run");
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(text.starts_with("devforgeai 1.0.0 ("), "stdout was {text}");
}

#[test]
fn no_root_gives_e030_exit_three() {
    let bare = tempfile::tempdir().expect("temp");
    let mut c = Command::cargo_bin("devforgeai").expect("binary");
    let out = c
        .current_dir(bare.path())
        .args(["doc", "load", "requirements", "-"])
        .output()
        .expect("run");

    assert_eq!(out.status.code(), Some(3));
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(stderr.contains("DFA-E030"), "stderr was {stderr}");
}

#[test]
fn project_flag_absent_gives_e031_exit_three() {
    let bare = tempfile::tempdir().expect("temp");
    let mut c = Command::cargo_bin("devforgeai").expect("binary");
    let out = c
        .current_dir(bare.path())
        .arg("--project")
        .arg(bare.path().join("nowhere"))
        .args(["doc", "load", "requirements", "-"])
        .output()
        .expect("run");

    assert_eq!(out.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&out.stderr).contains("DFA-E031"));
}

#[test]
fn an_unknown_subcommand_exits_three() {
    let mut c = Command::cargo_bin("devforgeai").expect("binary");
    let out = c.arg("invent").output().expect("run");
    assert_eq!(out.status.code(), Some(3));
}

#[test]
fn doc_load_prints_the_file_byte_for_byte() {
    let p = Project::new();
    let body = common::story("STORY-014", "ready", &[], "# Order checkout\n");
    p.write(".devforgeai/stories/STORY-014.md", &body);

    let out = cli(&p)
        .args(["doc", "load", "story", "STORY-014"])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        body,
        "no added header, no trailing newline of its own"
    );
}

#[test]
fn doc_validate_allocate_prints_the_id_alone() {
    let p = Project::new();
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-003", "drafting"),
    );
    let out = cli(&p)
        .args(["doc", "validate", "--allocate", "REQ"])
        .output()
        .expect("run");

    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "REQ-003\n");
}

#[test]
fn gate_check_exits_two_on_send_back() {
    let p = Project::new();
    p.write(
        ".devforgeai/gates.toml",
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
  on_fail = "send_back"

  [[gate.check]]
  kind = "field_in_enum"
  id = "decision-enum"
  path = "explore/decision.yaml"
  field = "decision"
  values = ["kill", "park", "promote"]
"#,
    );
    p.set_phase("explore", "IDEA-003");

    let out = cli(&p)
        .args(["gate", "check", "--phase", "explore", "--id", "IDEA-003"])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(2), "SEND BACK is exit 2");
}

#[test]
fn quiet_suppresses_human_stdout_but_not_the_exit_code() {
    let p = Project::new();
    p.write(
        ".devforgeai/stories/STORY-014.md",
        &common::story("STORY-014", "ready", &[], "# T\n"),
    );
    let out = cli(&p)
        .args([
            "--quiet",
            "doc",
            "validate",
            ".devforgeai/stories/STORY-014.md",
        ])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stdout.is_empty(), "no human stdout under --quiet");
}

// --- one envelope per --json invocation ----------------------------------

/// Parse stdout as exactly one JSON object.
fn one_object(bytes: &[u8]) -> serde_json::Value {
    let text = String::from_utf8_lossy(bytes).to_string();
    let v: serde_json::Value = serde_json::from_str(text.trim())
        .unwrap_or_else(|e| panic!("stdout is one JSON object ({e}): {text}"));
    assert!(v.is_object(), "the envelope is an object: {text}");
    v
}

#[test]
fn json_parse_failure_prints_one_envelope() {
    // A caller parsing stdout as JSON must not crash on a typo.
    for args in [
        vec!["--json", "gate", "check", "--bogus"],
        vec!["--json", "frobnicate"],
        vec!["--json", "gate", "check"],
    ] {
        let mut c = Command::cargo_bin("devforgeai").expect("binary");
        let out = c.args(&args).output().expect("run");
        assert_eq!(out.status.code(), Some(3), "{args:?}");
        let v = one_object(&out.stdout);
        assert_eq!(v["exit"], 3, "{args:?}");
        assert_eq!(v["ok"], false);
        assert_eq!(v["errors"][0]["code"], "DFA-E010", "{args:?}");
        assert!(
            v["errors"][0]["message"].as_str().expect("message").len() > 3,
            "clap's own text is carried: {v}"
        );
    }
}

#[test]
fn json_version_prints_an_envelope() {
    let mut c = Command::cargo_bin("devforgeai").expect("binary");
    let out = c.args(["--json", "--version"]).output().expect("run");
    assert_eq!(out.status.code(), Some(0));
    let v = one_object(&out.stdout);
    assert_eq!(v["command"], "version");
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["version"], "1.0.0");
    assert!(v["data"]["revision"].is_string());
}

#[test]
fn json_with_no_subcommand_prints_an_envelope() {
    let mut c = Command::cargo_bin("devforgeai").expect("binary");
    let out = c.arg("--json").output().expect("run");
    assert_eq!(out.status.code(), Some(3));
    let v = one_object(&out.stdout);
    assert_eq!(v["exit"], 3);
    assert_eq!(v["errors"][0]["code"], "DFA-E011");
}

#[test]
fn an_error_envelope_carries_the_project_root() {
    let p = Project::new();
    let out = cli(&p)
        .args([
            "--json",
            "doc",
            "validate",
            ".devforgeai/stories/STORY-001.md",
        ])
        .output()
        .expect("run");

    let v = one_object(&out.stdout);
    assert_eq!(v["errors"][0]["code"], "DFA-E200");
    // A caller collating envelopes across projects has to be able to attribute
    // the failure.
    assert!(
        !v["project"].as_str().expect("project").is_empty(),
        "the resolved root travels on the error branch too: {v}"
    );
}

#[test]
fn a_failing_gate_carries_its_codes_in_errors() {
    let p = Project::new();
    p.set_phase("explore", "IDEA-001");
    let out = cli(&p)
        .args([
            "--json", "gate", "check", "--phase", "explore", "--id", "IDEA-001",
        ])
        .output()
        .expect("run");

    assert_eq!(out.status.code(), Some(1));
    let v = one_object(&out.stdout);
    assert_eq!(v["ok"], false);
    let codes: Vec<&str> = v["errors"]
        .as_array()
        .expect("errors")
        .iter()
        .filter_map(|e| e["code"].as_str())
        .collect();
    // A caller that reads `errors[]` to decide what to fix must not be left
    // reading free text out of `checks[].reason`.
    assert!(!codes.is_empty(), "a failing gate names its codes: {v}");
    for e in v["errors"].as_array().expect("errors") {
        assert!(
            e["path"].as_str().expect("path").contains("IDEA-001"),
            "each row names the report it came from: {e}"
        );
    }
}

#[test]
fn json_still_writes_diagnostics_to_stderr() {
    let p = Project::new();
    let out = cli(&p)
        .args([
            "--json",
            "doc",
            "validate",
            ".devforgeai/stories/STORY-001.md",
        ])
        .output()
        .expect("run");

    // A caller piping stdout to a parser still watches stderr for diagnostics.
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        stderr.contains("DFA-E200"),
        "stderr carries the diagnostic under --json too: {stderr:?}"
    );
}

#[test]
fn a_failing_check_with_no_code_still_reaches_errors() {
    // Every failing check names its own `DFA-Ennn`, so a reason with none is a
    // contract violation by the engine. It is still reported: a failing check
    // that appears in `checks[]` and nowhere in `errors[]` is invisible to a
    // caller that reads `errors[]` to decide what to fix.
    let (code, message) = devforgeai::cmd::gate::split_code_for_test("the command went wrong");
    assert_eq!(code, "DFA-E350");
    assert_eq!(message, "the command went wrong");

    let (code, message) =
        devforgeai::cmd::gate::split_code_for_test("DFA-E311 test command exited 101");
    assert_eq!(code, "DFA-E311");
    assert_eq!(message, "test command exited 101");

    // A `DFA-` shape that is not a table row is not a code either.
    let (code, _) = devforgeai::cmd::gate::split_code_for_test("DFA-E999 invented");
    assert_eq!(code, "DFA-E350");
}

// --- the hook's stdout is one JSON object and nothing else ----------------

/// `hook run <event>` through the real binary, with `payload` on stdin and the
/// home redirected to `home`.
fn hook_run(
    project: &Project,
    home: &std::path::Path,
    event: &str,
    payload: &str,
) -> std::process::Output {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = std::process::Command::new(assert_cmd::cargo::cargo_bin("devforgeai"))
        .args(["--project"])
        .arg(project.root())
        .args(["hook", "run", event])
        // The spawned binary is built with default features, so it reads the
        // platform home rather than the test-home override.
        .env("USERPROFILE", home)
        .env("HOME", home)
        .env("DEVFORGEAI_HOME", home)
        .current_dir(project.root())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary spawns");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(payload.as_bytes())
        .expect("write the payload");
    child.wait_with_output().expect("the hook runs")
}

/// Assert the bytes are exactly one JSON object, with nothing before or after.
fn exactly_one_json_object(stdout: &[u8]) -> serde_json::Value {
    let text = String::from_utf8_lossy(stdout).to_string();
    // The harness parses stdout that starts with `{` and ends with `}`. A
    // human line printed beside the object makes the parse a non-blocking
    // error, which silently disables the hook.
    let trimmed = text.trim();
    assert!(
        trimmed.starts_with('{') && trimmed.ends_with('}'),
        "stdout is one object and nothing else, was: {text:?}"
    );
    let mut stream = serde_json::Deserializer::from_str(trimmed).into_iter::<serde_json::Value>();
    let first = stream
        .next()
        .expect("one value")
        .unwrap_or_else(|e| panic!("stdout parses ({e}): {text:?}"));
    assert!(stream.next().is_none(), "exactly one object, was: {text:?}");
    assert!(first.is_object(), "{text:?}");
    first
}

#[test]
fn stop_hook_stdout_is_one_object_when_untrusted() {
    let _guard = common::RealTrustStoreGuard::new();
    let p = Project::new();
    let home = tempfile::tempdir().expect("temp home");

    // No `trust.toml` at all: the trust-failure path.
    let out = hook_run(
        &p,
        home.path(),
        "stop",
        r#"{"session_id":"s1","hook_event_name":"Stop","stop_hook_active":false}"#,
    );

    assert_eq!(
        out.status.code(),
        Some(2),
        "a trust failure blocks the turn"
    );
    let v = exactly_one_json_object(&out.stdout);
    assert_eq!(v["decision"], "block");
    assert!(v["systemMessage"]
        .as_str()
        .expect("systemMessage")
        .contains("TRUST FAIL"));
    // The reason is on stderr too, so a schema change upstream degrades to the
    // stderr path rather than to silence.
    assert!(!out.stderr.is_empty());
}

#[test]
fn stop_hook_stdout_is_one_object_on_a_failing_gate() {
    let _guard = common::RealTrustStoreGuard::new();
    let p = Project::new();
    let home = tempfile::tempdir().expect("temp home");

    // A pin for the binary under test, written by hand: `trust pin` refuses
    // inside a session, and the permission layer refuses it too.
    let exe = assert_cmd::cargo::cargo_bin("devforgeai");
    let digest = devforgeai::trust::digest_file(&exe).expect("the binary digest");
    std::fs::create_dir_all(home.path()).expect("mkdir");
    std::fs::write(
        home.path().join("trust.toml"),
        format!(
            "schema = \"devforgeai/trust/1\"\nupdated_at = \"\"\n\n[[pin]]\nbinary_path = '{}'\ndigest = '{digest}'\nrevision = 'unversioned'\nsource_digest = ''\nrelease_digest = ''\nframework_path = ''\npinned_at = ''\npinned_by = ''\n",
            exe.display()
        ),
    )
    .expect("write trust.toml");
    p.set_phase("explore", "IDEA-003");

    let out = hook_run(
        &p,
        home.path(),
        "stop",
        r#"{"session_id":"s1","hook_event_name":"Stop","stop_hook_active":false}"#,
    );

    assert_eq!(out.status.code(), Some(2), "the explore gate fails");
    let v = exactly_one_json_object(&out.stdout);
    assert_eq!(v["decision"], "block");
    assert!(v["reason"].as_str().expect("reason").contains("FAIL"));
    assert!(v["systemMessage"].is_string());
    // `Outcome.human` carries the gate lines and the handoff, and none of it
    // may reach stdout beside the object. The object is written with one
    // trailing newline, so one line is the whole of stdout; the gate lines
    // appearing inside `reason` is the point of the field, not a leak.
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert_eq!(
        text.trim_end_matches('\n').lines().count(),
        1,
        "the human block is suppressed when the decision object is emitted: {text:?}"
    );
}
