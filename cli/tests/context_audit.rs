//! `context audit`: the eight checks `specs/04-constitute.md` owns.
//!
//! The two list constants the audit compiles in — the H2 section lists and the
//! closed key namespace — are compared against the spec text at test time, so
//! a spec edit fails here rather than drifting silently.

mod common;

use assert_cmd::Command;
use common::Project;

fn cli(p: &Project) -> Command {
    let mut c = Command::cargo_bin("devforgeai").expect("the binary builds");
    c.arg("--project").arg(p.root());
    c.current_dir(p.root());
    c
}

fn run(p: &Project) -> (i32, String, String) {
    let out = cli(p).args(["context", "audit"]).output().expect("run");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

fn json(p: &Project) -> serde_json::Value {
    let out = cli(p)
        .args(["--json", "context", "audit"])
        .output()
        .expect("run");
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    serde_json::from_str(text.trim()).unwrap_or_else(|e| panic!("{e} in {text}"))
}

fn spec(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("specs")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

// ------------------------------------------ the compiled lists match the spec

#[test]
fn the_section_lists_are_the_spec_section_list_table() {
    let text = spec("04-constitute.md");
    let mut want: Vec<(String, Vec<String>)> = Vec::new();
    for line in text.lines() {
        // `| tech-stack.md | `.devforgeai/…` | `## Languages`, `## Runtimes` |`
        if !line.starts_with("| ") || !line.contains(".devforgeai/") {
            continue;
        }
        let cells: Vec<&str> = line
            .trim()
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        if cells.len() != 3 || !cells[2].starts_with("`## ") {
            continue;
        }
        let stem = cells[0].trim_end_matches(".md").to_string();
        let headings: Vec<String> = cells[2]
            .split(',')
            .map(|h| h.trim().trim_matches('`').to_string())
            .collect();
        want.push((stem, headings));
    }
    assert_eq!(want.len(), 7, "six context files and the ADR row");

    for (stem, headings) in &want {
        if stem == "ADR-nnn" {
            assert_eq!(
                devforgeai::audit::ADR_SECTIONS.to_vec(),
                headings.iter().map(String::as_str).collect::<Vec<_>>(),
                "the ADR section list"
            );
            continue;
        }
        let compiled = devforgeai::audit::CONTEXT_SECTIONS
            .iter()
            .find(|(s, _)| s == stem)
            .unwrap_or_else(|| panic!("{stem} is compiled in"));
        assert_eq!(
            compiled.1.to_vec(),
            headings.iter().map(String::as_str).collect::<Vec<_>>(),
            "{stem} section list"
        );
    }
}

#[test]
fn the_key_namespace_is_the_spec_key_table() {
    let text = spec("04-constitute.md");
    let start = text
        .find("### The key namespace")
        .expect("the key namespace section");
    let end = text[start..]
        .find("### Closed enumerations")
        .map(|o| start + o)
        .unwrap_or(text.len());
    let mut want: Vec<String> = Vec::new();
    for line in text[start..end].lines() {
        if !line.starts_with("| `") {
            continue;
        }
        let cells: Vec<&str> = line
            .trim()
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        want.push(cells[0].trim_matches('`').to_string());
    }
    assert!(want.len() > 15, "the table was found");

    let compiled: Vec<&str> = devforgeai::audit::KEY_NAMESPACE
        .iter()
        .map(|(k, _)| *k)
        .collect();
    assert_eq!(compiled, want, "the closed key namespace, row for row");
}

// --------------------------------------------------------------- the happy path

#[test]
fn a_conforming_set_passes_every_check() {
    let p = Project::new();
    common::write_context_set(&p, "accepted");
    let (code, out, err) = run(&p);
    assert_eq!(code, 0, "stdout {out} stderr {err}");
    assert!(out.contains("6/6 files"), "stdout was {out}");
    assert!(out.contains("1 constraints"), "stdout was {out}");
    assert!(out.contains("1 anti-patterns"), "stdout was {out}");
}

#[test]
fn the_json_data_object_carries_files_counts_checks_and_findings() {
    let p = Project::new();
    common::write_context_set(&p, "accepted");
    let v = json(&p);
    assert_eq!(v["command"], "context audit");
    assert_eq!(v["exit"], 0);
    assert_eq!(v["data"]["files"].as_array().expect("files").len(), 6);
    assert_eq!(v["data"]["files"][0]["name"], "tech-stack");
    assert_eq!(v["data"]["files"][0]["present"], true);
    assert_eq!(v["data"]["files"][0]["status"], "accepted");
    assert_eq!(v["data"]["files"][0]["keys"], 2);
    assert_eq!(v["data"]["constraints"], 1);
    assert_eq!(v["data"]["anti_patterns"], 1);
    assert_eq!(v["data"]["adrs"], 1);
    assert_eq!(v["data"]["checks"].as_array().expect("checks").len(), 8);
    assert_eq!(v["data"]["findings"].as_array().expect("findings").len(), 0);
}

// --------------------------------------------------------------- CA-1 and CA-2

#[test]
fn ca1_names_every_missing_context_file() {
    let p = Project::new();
    let (code, out, err) = run(&p);
    assert_eq!(code, 1, "stdout {out}");
    for stem in devforgeai::doc::CONTEXT_STEMS {
        assert!(
            err.contains(&format!(".devforgeai/context/{stem}.md")),
            "{stem} is named; stderr was {err}"
        );
    }
    assert!(err.contains("DFA-E220"), "stderr was {err}");
    assert!(out.contains("0/6 files"), "stdout was {out}");
}

#[test]
fn ca2_refuses_a_draft_context_file() {
    let p = Project::new();
    common::write_context_set(&p, "draft");
    let (code, _, err) = run(&p);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E223"), "stderr was {err}");
    assert!(err.contains("is status 'draft'"), "stderr was {err}");
}

#[test]
fn ca2_refuses_an_open_question() {
    let p = Project::new();
    common::write_context_set(&p, "accepted");
    let text = p
        .read(".devforgeai/context/dependencies.md")
        .replace("open_questions: []", "open_questions: [is this right]");
    p.write(".devforgeai/context/dependencies.md", &text);
    let (code, _, err) = run(&p);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E223"), "stderr was {err}");
    assert!(err.contains("1 open questions"), "stderr was {err}");
}

// ------------------------------------------------------------------- CA-3

#[test]
fn ca3_refuses_a_missing_heading() {
    let p = Project::new();
    common::write_context_set(&p, "accepted");
    let text = p
        .read(".devforgeai/context/tech-stack.md")
        .replace("## Tooling", "## Tools");
    p.write(".devforgeai/context/tech-stack.md", &text);
    let (code, _, err) = run(&p);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E213"), "stderr was {err}");
    assert!(err.contains("'## Tooling' is absent"), "stderr was {err}");
    assert!(
        err.contains("'## Tools' is not in the section list"),
        "the extra heading is named too; stderr was {err}"
    );
}

#[test]
fn ca3_refuses_a_heading_out_of_order() {
    let p = Project::new();
    common::write_context_set(&p, "accepted");
    let text = p.read(".devforgeai/context/anti-patterns.md");
    let swapped = text
        .replace("## Anti-patterns\n", "@@FIRST@@\n")
        .replace("## Anti-pattern index\n", "## Anti-patterns\n")
        .replace("@@FIRST@@\n", "## Anti-pattern index\n");
    p.write(".devforgeai/context/anti-patterns.md", &swapped);
    let (code, _, err) = run(&p);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E213"), "stderr was {err}");
    assert!(err.contains("expected"), "stderr was {err}");
}

// ------------------------------------------------------------------- CA-4

#[test]
fn ca4_refuses_an_active_con_with_no_adr_and_no_req() {
    let p = Project::new();
    common::write_context_set(&p, "accepted");
    // Break both routes: no ADR introduces it, and the source names no REQ.
    let text = p
        .read(".devforgeai/context/architecture-constraints.md")
        .replace(
        "| CON-001 | boundary | active | The core holds no IO | REQ-001 | ADR-001 | AP-001 |",
        "| CON-001 | boundary | active | The core holds no IO | init --analyze | none | AP-001 |",
    );
    p.write(".devforgeai/context/architecture-constraints.md", &text);
    let adr = p.read(".devforgeai/adr/ADR-001.md").replace(
        "| CON-001 | boundary | The core holds no IO |",
        "| none | none | none |",
    );
    p.write(".devforgeai/adr/ADR-001.md", &adr);

    let (code, _, err) = run(&p);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E224"), "stderr was {err}");
    assert!(
        err.contains("CON-001 is introduced by no ADR"),
        "stderr was {err}"
    );
}

#[test]
fn ca4_accepts_a_con_that_traces_to_a_requirement_alone() {
    let p = Project::new();
    common::write_context_set(&p, "accepted");
    let adr = p.read(".devforgeai/adr/ADR-001.md").replace(
        "| CON-001 | boundary | The core holds no IO |",
        "| none | none | none |",
    );
    p.write(".devforgeai/adr/ADR-001.md", &adr);
    let (code, _, err) = run(&p);
    assert_eq!(code, 0, "the REQ-001 source carries it; stderr was {err}");
}

#[test]
fn a_constraint_defined_in_two_files_is_e222() {
    let p = Project::new();
    common::write_context_set(&p, "accepted");
    let extra = p
        .read(".devforgeai/adr/ADR-001.md")
        .replace("## Context\n", "## Context\n\n### CON-001 a second home\n");
    p.write(".devforgeai/adr/ADR-001.md", &extra);
    let (code, _, err) = run(&p);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E222"), "stderr was {err}");
}

// ------------------------------------------------------------------- CA-5

#[test]
fn ca5_finds_two_values_for_one_key_across_two_files() {
    let p = Project::new();
    common::write_context_set(&p, "accepted");
    // source-tree.md restates the primary language with a different value.
    let text = p.read(".devforgeai/context/source-tree.md").replace(
        "| source.root | src | layout |",
        "| source.root | src | layout |\n| language.primary | go | guess |",
    );
    p.write(".devforgeai/context/source-tree.md", &text);

    let (code, _, err) = run(&p);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E221"), "stderr was {err}");

    let v = json(&p);
    let f = v["data"]["findings"]
        .as_array()
        .expect("findings")
        .iter()
        .find(|f| f["check"] == "CA-5")
        .expect("a CA-5 finding");
    assert_eq!(f["key"], "language.primary");
    assert_eq!(f["a"]["path"], ".devforgeai/context/tech-stack.md");
    assert_eq!(f["a"]["value"], "rust");
    assert_eq!(f["b"]["path"], ".devforgeai/context/source-tree.md");
    assert_eq!(f["b"]["value"], "go");
}

#[test]
fn ca5_accepts_the_same_key_with_the_same_value_in_two_files() {
    let p = Project::new();
    common::write_context_set(&p, "accepted");
    let text = p.read(".devforgeai/context/source-tree.md").replace(
        "| source.root | src | layout |",
        "| source.root | src | layout |\n| language.primary |  RUST  | guess |",
    );
    p.write(".devforgeai/context/source-tree.md", &text);
    let (code, _, err) = run(&p);
    assert_eq!(
        code, 0,
        "comparison is after trimming, collapsing, and lowercasing; stderr was {err}"
    );
}

#[test]
fn ca5_refuses_a_key_outside_the_closed_namespace() {
    let p = Project::new();
    common::write_context_set(&p, "accepted");
    let text = p.read(".devforgeai/context/source-tree.md").replace(
        "| source.root | src | layout |",
        "| source.root | src | layout |\n| layer.domain.core.path | src/domain/core | guess |",
    );
    p.write(".devforgeai/context/source-tree.md", &text);
    let (code, _, err) = run(&p);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E221"), "stderr was {err}");
    assert!(
        err.contains("outside the closed key namespace"),
        "stderr was {err}"
    );
}

#[test]
fn ca5_reads_accepted_adrs_and_not_proposed_ones() {
    let p = Project::new();
    common::write_context_set(&p, "accepted");
    let contradiction =
        "## Decision\n\n| Key | Value | Source |\n|---|---|---|\n| language.primary | go | the ADR |\n";
    let proposed = p
        .read(".devforgeai/adr/ADR-001.md")
        .replace("status: accepted", "status: proposed")
        .replace("## Decision\n\nThe binary reads TOML.\n", contradiction);
    p.write(".devforgeai/adr/ADR-001.md", &proposed);
    let (code, _, err) = run(&p);
    assert_eq!(
        code, 0,
        "a proposed ADR contributes nothing; stderr was {err}"
    );

    let accepted = p
        .read(".devforgeai/adr/ADR-001.md")
        .replace("status: proposed", "status: accepted");
    p.write(".devforgeai/adr/ADR-001.md", &accepted);
    let (code, _, err) = run(&p);
    assert_eq!(code, 1, "an accepted ADR does");
    assert!(err.contains("DFA-E221"), "stderr was {err}");
}

// ------------------------------------------------------------------- CA-6

#[test]
fn ca6_refuses_an_incomplete_anti_pattern_row() {
    let p = Project::new();
    common::write_context_set(&p, "accepted");
    for (broken, expect) in [
        (
            "| AP-001 | layer | high | src/**/*.rs | literal |  | CON-001 |",
            "no detector",
        ),
        (
            "| AP-001 | layer | high | src/**/*.rs | grep | TcpStream | CON-001 |",
            "detector_kind 'grep'",
        ),
        (
            "| AP-001 | layer | high | src/**/*.rs | literal | TcpStream | CON-404 |",
            "source 'CON-404'",
        ),
    ] {
        common::write_context_set(&p, "accepted");
        let text = p.read(".devforgeai/context/anti-patterns.md").replace(
            "| AP-001 | layer | high | src/**/*.rs | literal | TcpStream | CON-001 |",
            broken,
        );
        p.write(".devforgeai/context/anti-patterns.md", &text);
        let (code, _, err) = run(&p);
        assert_eq!(code, 1, "{broken}");
        assert!(err.contains("DFA-E225"), "{broken}: stderr was {err}");
        assert!(err.contains(expect), "{broken}: stderr was {err}");
    }
}

// ------------------------------------------------------------------- CA-7

#[test]
fn ca7_refuses_an_unresolved_consumes_reference() {
    let p = Project::new();
    common::write_context_set(&p, "accepted");
    let text = p
        .read(".devforgeai/adr/ADR-001.md")
        .replace("consumes: [REQ-001]", "consumes: [REQ-404]");
    p.write(".devforgeai/adr/ADR-001.md", &text);
    let (code, _, err) = run(&p);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E226"), "stderr was {err}");
    assert!(err.contains("REQ-404"), "stderr was {err}");
}

#[test]
fn ca7_refuses_a_duplicate_adr_id() {
    let p = Project::new();
    common::write_context_set(&p, "accepted");
    let text = p.read(".devforgeai/adr/ADR-001.md");
    p.write(".devforgeai/adr/ADR-002.md", &text);
    let (code, _, err) = run(&p);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E226"), "stderr was {err}");
    assert!(err.contains("duplicates"), "stderr was {err}");
}

#[test]
fn ca7_refuses_an_introduced_con_the_index_omits() {
    let p = Project::new();
    common::write_context_set(&p, "accepted");
    let text = p.read(".devforgeai/adr/ADR-001.md").replace(
        "| CON-001 | boundary | The core holds no IO |",
        "| CON-001 | boundary | The core holds no IO |\n| CON-909 | layering | Nothing indexes this |",
    );
    p.write(".devforgeai/adr/ADR-001.md", &text);
    let (code, _, err) = run(&p);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E226"), "stderr was {err}");
    assert!(err.contains("CON-909"), "stderr was {err}");
}

// ------------------------------------------------------------------- CA-8

#[test]
fn ca8_refuses_a_supersession_the_target_does_not_record() {
    let p = Project::new();
    common::write_context_set(&p, "accepted");
    p.write(
        ".devforgeai/adr/ADR-002.md",
        &common::adr(
            "ADR-002",
            "accepted",
            &[],
            "| none | none | none |\n",
            "| ADR-001 | CON-001 |\n",
        ),
    );
    let (code, _, err) = run(&p);
    assert_eq!(code, 1);
    assert!(err.contains("DFA-E227"), "stderr was {err}");
    assert!(
        err.contains("supersedes ADR-001, which is status 'accepted'"),
        "stderr was {err}"
    );
    assert!(
        err.contains("retires CON-001, which is status 'active'"),
        "stderr was {err}"
    );
}

#[test]
fn ca8_accepts_a_complete_supersession() {
    let p = Project::new();
    common::write_context_set(&p, "accepted");
    let old = p
        .read(".devforgeai/adr/ADR-001.md")
        .replace("status: accepted", "status: superseded");
    p.write(".devforgeai/adr/ADR-001.md", &old);
    let index = p
        .read(".devforgeai/context/architecture-constraints.md")
        .replace(
            "| CON-001 | boundary | active |",
            "| CON-001 | boundary | retired |",
        )
        .replace("| status | active |", "| status | retired |");
    p.write(".devforgeai/context/architecture-constraints.md", &index);
    // CA-6 wants an active CON behind every anti-pattern, so the retired
    // constraint takes its anti-pattern with it.
    let aps = p.read(".devforgeai/context/anti-patterns.md").replace(
        "| AP-001 | layer | high | src/**/*.rs | literal | TcpStream | CON-001 |
",
        "",
    );
    p.write(".devforgeai/context/anti-patterns.md", &aps);
    p.write(
        ".devforgeai/adr/ADR-002.md",
        &common::adr(
            "ADR-002",
            "accepted",
            &[],
            "| none | none | none |\n",
            "| ADR-001 | CON-001 |\n",
        ),
    );
    let (code, _, err) = run(&p);
    assert_eq!(code, 0, "stderr was {err}");
}

// --------------------------------------------------- the `context_audit` kind

#[test]
fn the_context_audit_check_passes_over_a_conforming_set() {
    let p = Project::new();
    common::write_context_set(&p, "accepted");
    p.write(
        ".devforgeai/gates.toml",
        r#"schema = "devforgeai/gates/1"
cli_min_version = "1.0.0"

[[gate]]
phase = "constitute"
requires = ""
on_fail = "fail"

  [[gate.check]]
  kind = "context_audit"
  id = "context-clean"
"#,
    );
    p.set_phase("constitute", "IDEA-001");
    let out = cli(&p)
        .args(["gate", "check", "--phase", "constitute", "--id", "IDEA-001"])
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
fn the_context_audit_check_fails_with_e327() {
    let p = Project::new();
    p.write(
        ".devforgeai/gates.toml",
        r#"schema = "devforgeai/gates/1"
cli_min_version = "1.0.0"

[[gate]]
phase = "constitute"
requires = ""
on_fail = "fail"

  [[gate.check]]
  kind = "context_audit"
  id = "context-clean"
"#,
    );
    p.set_phase("constitute", "IDEA-001");
    let out = cli(&p)
        .args(["gate", "check", "--phase", "constitute", "--id", "IDEA-001"])
        .output()
        .expect("run");
    assert_eq!(out.status.code(), Some(1));
    let report = p.read(".devforgeai/reports/IDEA-001-constitute.yaml");
    assert!(report.contains("DFA-E327"), "report was {report}");
    assert!(
        report.contains("context audit found"),
        "report was {report}"
    );
}
