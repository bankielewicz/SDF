//! `gate check`: every check kind this milestone evaluates, the report writer,
//! the send-back resolution, and the `--partial` and `--no-run` flags.

mod common;

use common::Project;
use devforgeai::cli::GateCheckArgs;
use devforgeai::cmd::gate;
use devforgeai::Outcome;

/// A `gates.toml` whose explore gate carries the two kinds the compiled
/// minimums require of that phase, both satisfied, plus the checks under test.
/// A gate-level `document` is the only source of a subject path for the kinds
/// whose key set holds no `path`; an empty one leaves each check to name its
/// own.
fn gates_with_document(document: &str, extra: &str) -> String {
    let doc_line = if document.is_empty() {
        String::new()
    } else {
        format!("document = \"{document}\"\n")
    };
    format!(
        r#"schema = "devforgeai/gates/1"
cli_min_version = "1.0.0"

[[gate]]
phase = "explore"
requires = ""
on_fail = "fail"
send_back_to = ""
{doc_line}

  [[gate.check]]
  kind = "file_exists"
  id = "baseline-exists"
  severity = "block"
  paths = ["explore/decision.yaml"]
  min_count = 1

  [[gate.check]]
  kind = "field_in_enum"
  id = "baseline-enum"
  severity = "block"
  path = "explore/decision.yaml"
  field = "decision"
  values = ["kill", "park", "promote"]

{extra}
"#
    )
}

/// A project whose explore gate baseline passes.
fn seeded(extra: &str) -> Project {
    seeded_with_document("", extra)
}

/// The same, with a gate-level `document`.
fn seeded_with_document(document: &str, extra: &str) -> Project {
    let p = Project::new();
    p.write(
        ".devforgeai/gates.toml",
        &gates_with_document(document, extra),
    );
    p.write(
        ".devforgeai/explore/decision.yaml",
        &common::decision("IDEA-003", "promote"),
    );
    p.set_phase("explore", "IDEA-003");
    p
}

fn run(p: &Project) -> Outcome {
    let mut ctx = p.ctx();
    let args = GateCheckArgs {
        phase: "explore".into(),
        id: Some("IDEA-003".into()),
        partial: false,
        no_run: false,
    };
    gate::check(&mut ctx, &args).expect("gate check runs")
}

fn status_of(out: &Outcome, id: &str) -> String {
    out.data["checks"]
        .as_array()
        .expect("checks")
        .iter()
        .find(|c| c["id"] == id)
        .unwrap_or_else(|| panic!("check {id} is in the report"))["status"]
        .as_str()
        .unwrap_or("")
        .to_string()
}

fn reason_of(out: &Outcome, id: &str) -> String {
    out.data["checks"]
        .as_array()
        .expect("checks")
        .iter()
        .find(|c| c["id"] == id)
        .expect("check")["reason"]
        .as_str()
        .unwrap_or("")
        .to_string()
}

// ---------------------------------------------------------------- check kinds

#[test]
fn file_exists_absent_flag() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "file_exists"
  id = "prototype-pruned"
  paths = [".explore-prototype"]
  absent = true
"#,
    );
    assert_eq!(status_of(&run(&p), "prototype-pruned"), "pass");

    std::fs::create_dir_all(p.root().join(".explore-prototype")).expect("mkdir");
    let out = run(&p);
    assert_eq!(status_of(&out, "prototype-pruned"), "fail");
    assert!(reason_of(&out, "prototype-pruned").contains("DFA-E323"));
}

#[test]
fn file_exists_min_count_and_non_empty() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "file_exists"
  id = "two-of-three"
  paths = ["a.md", "b.md", "c.md"]
  min_count = 2
"#,
    );
    p.write(".devforgeai/a.md", "content\n");
    p.write(".devforgeai/b.md", "");
    assert_eq!(
        status_of(&run(&p), "two-of-three"),
        "fail",
        "an empty file does not count"
    );
    p.write(".devforgeai/b.md", "content\n");
    assert_eq!(status_of(&run(&p), "two-of-three"), "pass");
}

#[test]
fn field_in_enum_passes_and_fails() {
    let p = seeded("");
    assert_eq!(status_of(&run(&p), "baseline-enum"), "pass");

    p.write(
        ".devforgeai/explore/decision.yaml",
        &common::decision("IDEA-003", "maybe"),
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "baseline-enum"), "fail");
    assert!(reason_of(&out, "baseline-enum").contains("DFA-E322"));
}

#[test]
fn field_is_date_parses_format_and_after_field() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "field_is_date"
  id = "dated"
  path = "explore/decision.yaml"
  field = "decided_on"

  [[gate.check]]
  kind = "field_is_date"
  id = "revisit"
  path = "explore/decision.yaml"
  field = "revisit_on"
  after_field = "decided_on"
"#,
    );
    p.write(
        ".devforgeai/explore/decision.yaml",
        "schema: devforgeai/explore-decision/1\nid: IDEA-003\nphase: explore\nstatus: recorded\nproduced_by: exploring-ideas\nconsumes: []\nopen_questions: []\ndecision: park\ndecided_on: 2026-09-08\nrevisit_on: 2026-10-08\n",
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "dated"), "pass");
    assert_eq!(status_of(&out, "revisit"), "pass");

    // Earlier than `after_field` fails.
    p.write(
        ".devforgeai/explore/decision.yaml",
        "schema: devforgeai/explore-decision/1\nid: IDEA-003\nphase: explore\nstatus: recorded\nproduced_by: exploring-ideas\nconsumes: []\nopen_questions: []\ndecision: park\ndecided_on: 2026-09-08\nrevisit_on: 2026-09-01\n",
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "revisit"), "fail");
    assert!(reason_of(&out, "revisit").contains("DFA-E330"));

    // A malformed date fails.
    p.write(
        ".devforgeai/explore/decision.yaml",
        "schema: devforgeai/explore-decision/1\nid: IDEA-003\nphase: explore\nstatus: recorded\nproduced_by: exploring-ideas\nconsumes: []\nopen_questions: []\ndecision: park\ndecided_on: 8 September\nrevisit_on: 2026-10-08\n",
    );
    assert_eq!(status_of(&run(&p), "dated"), "fail");
}

#[test]
fn length_between_excludes_status() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "length_between"
  id = "epic-not-empty"
  path = "requirements.yaml"
  field = "epics[].requirements"
  min = 1

  [[gate.check]]
  kind = "length_between"
  id = "live-requirements"
  path = "requirements.yaml"
  field = "requirements"
  min = 2
  exclude_status = ["withdrawn"]
"#,
    );
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-003", "accepted"),
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "epic-not-empty"), "pass");
    assert_eq!(status_of(&out, "live-requirements"), "pass");

    // Withdraw one: the live count drops below the minimum.
    let text = common::requirements("IDEA-003", "accepted").replace(
        "    priority: should\n    source: user\n    status: draft",
        "    priority: should\n    source: user\n    status: withdrawn",
    );
    p.write(".devforgeai/requirements.yaml", &text);
    let out = run(&p);
    assert_eq!(status_of(&out, "live-requirements"), "fail");
    assert!(reason_of(&out, "live-requirements").contains("DFA-E331"));
}

#[test]
fn fields_present_collection_form() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "fields_present"
  id = "req-fields"
  path = "requirements.yaml"
  collection = "requirements"
  fields = ["id", "actor", "statement", "priority"]
"#,
    );
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-003", "accepted"),
    );
    assert_eq!(status_of(&run(&p), "req-fields"), "pass");

    let text = common::requirements("IDEA-003", "accepted").replace("    priority: must\n", "");
    p.write(".devforgeai/requirements.yaml", &text);
    let out = run(&p);
    assert_eq!(status_of(&out, "req-fields"), "fail");
    assert!(reason_of(&out, "req-fields").contains("REQ-001"));
}

#[test]
fn fields_present_path_form_is_field_set() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "fields_present"
  id = "accepted"
  path = "requirements.yaml"
"#,
    );
    // With `path` and empty `fields`, the value at `path` is non-null.
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-003", "accepted"),
    );
    assert_eq!(status_of(&run(&p), "accepted"), "pass");
}

#[test]
fn set_cover_detects_uncovered_and_duplicate() {
    let p = seeded_with_document(
        "requirements.yaml",
        r#"  [[gate.check]]
  kind = "set_cover"
  id = "no-ungrouped-req"
  cover = "epics[].requirements"
  universe = "requirements[].id"
  exclude_status = ["withdrawn"]
"#,
    );
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-003", "accepted"),
    );
    assert_eq!(status_of(&run(&p), "no-ungrouped-req"), "pass");

    // A requirement in no epic.
    let text = common::requirements("IDEA-003", "accepted").replace(
        "requirements: [REQ-001, REQ-002]",
        "requirements: [REQ-001]",
    );
    p.write(".devforgeai/requirements.yaml", &text);
    let out = run(&p);
    assert_eq!(status_of(&out, "no-ungrouped-req"), "fail");
    assert!(reason_of(&out, "no-ungrouped-req").contains("REQ-002"));

    // The same requirement in two epics.
    let text = common::requirements("IDEA-003", "accepted").replace(
        "  - id: EPIC-001\n    title: Checkout\n    requirements: [REQ-001, REQ-002]\n",
        "  - id: EPIC-001\n    title: Checkout\n    requirements: [REQ-001, REQ-002]\n  - id: EPIC-002\n    title: Extras\n    requirements: [REQ-002]\n",
    );
    p.write(".devforgeai/requirements.yaml", &text);
    let out = run(&p);
    assert_eq!(status_of(&out, "no-ungrouped-req"), "fail");
    assert!(reason_of(&out, "no-ungrouped-req").contains("twice"));
}

#[test]
fn row_count_between_counts_table_rows() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "row_count_between"
  id = "flow-count"
  path = "explore/brief.md"
  section = "Core flows"
  min = 3
  max = 5
  message = "{value} core flows, expected 3 to 5"
"#,
    );
    p.write(
        ".devforgeai/explore/brief.md",
        &common::brief("IDEA-003", "decided"),
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "flow-count"), "pass");

    // Two rows is below the minimum; the separator row is not data.
    let text = common::brief("IDEA-003", "decided").replace("| FLOW-003 | Receipt |\n", "");
    p.write(".devforgeai/explore/brief.md", &text);
    let out = run(&p);
    assert_eq!(status_of(&out, "flow-count"), "fail");
    assert!(reason_of(&out, "flow-count").contains("holds 2 rows"));
}

#[test]
fn row_count_between_counts_body_lines_without_table() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "row_count_between"
  id = "prose-lines"
  path = "explore/brief.md"
  section = "Notes"
  min = 2
  max = 2
"#,
    );
    p.write(
        ".devforgeai/explore/brief.md",
        "---\nschema: devforgeai/explore-brief/1\nid: IDEA-003\nphase: explore\nstatus: decided\nproduced_by: exploring-ideas\nconsumes: []\nopen_questions: []\n---\n\n# T\n\n## Notes\n\nfirst line\n\nsecond line\n",
    );
    assert_eq!(status_of(&run(&p), "prose-lines"), "pass");
}

#[test]
fn column_matches_pattern_and_unique() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "column_matches"
  id = "flow-id-shape"
  path = "explore/brief.md"
  section = "Core flows"
  column = "ID"
  pattern = "^FLOW-[0-9]{3}$"
  unique = true
"#,
    );
    p.write(
        ".devforgeai/explore/brief.md",
        &common::brief("IDEA-003", "decided"),
    );
    assert_eq!(status_of(&run(&p), "flow-id-shape"), "pass");

    let text = common::brief("IDEA-003", "decided").replace("| FLOW-003 |", "| FLOW-3 |");
    p.write(".devforgeai/explore/brief.md", &text);
    let out = run(&p);
    assert_eq!(status_of(&out, "flow-id-shape"), "fail");
    assert!(reason_of(&out, "flow-id-shape").contains("malformed"));

    let text = common::brief("IDEA-003", "decided").replace("| FLOW-003 |", "| FLOW-001 |");
    p.write(".devforgeai/explore/brief.md", &text);
    let out = run(&p);
    assert_eq!(status_of(&out, "flow-id-shape"), "fail");
    assert!(reason_of(&out, "flow-id-shape").contains("duplicated"));
}

#[test]
fn column_contains_all_reads_state_array() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "column_contains_all"
  id = "remedy-flows-present"
  path = "explore/brief.md"
  section = "Core flows"
  column = "ID"
  state_field = "explore.remedy_flows"
  skip_when = { state_field = "explore.remedy_flows", is_empty = true }
"#,
    );
    p.write(
        ".devforgeai/explore/brief.md",
        &common::brief("IDEA-003", "decided"),
    );

    // An empty remedy list records `skip` with `reason: condition`.
    let out = run(&p);
    assert_eq!(status_of(&out, "remedy-flows-present"), "skip");
    assert_eq!(reason_of(&out, "remedy-flows-present"), "condition");

    let mut s = p.state();
    s.explore.remedy_flows = vec!["FLOW-002".into()];
    p.write_state(&s);
    assert_eq!(status_of(&run(&p), "remedy-flows-present"), "pass");

    let mut s = p.state();
    s.explore.remedy_flows = vec!["FLOW-009".into()];
    p.write_state(&s);
    let out = run(&p);
    assert_eq!(status_of(&out, "remedy-flows-present"), "fail");
    assert!(reason_of(&out, "remedy-flows-present").contains("DFA-E336"));
}

#[test]
fn elapsed_days_uses_calendar_days_and_prefers_the_remedy_pair() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "elapsed_days_at_most"
  id = "time-box"
  started_field = "explore.started_at"
  limit_field = "explore.timebox_days"
  remedy_started_field = "explore.remedy_started_at"
  remedy_limit_field = "explore.remedy_timebox_days"
"#,
    );
    // The clock is process-global, so this test holds the shared lock for as
    // long as it needs `now` to stand still.
    let _guard = common::ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    std::env::set_var("DEVFORGEAI_NOW", "2026-09-11T00:00:00Z");

    let mut s = p.state();
    s.explore.started_at = "2026-09-08T09:00:00Z".into();
    s.explore.timebox_days = 5;
    p.write_state(&s);
    assert_eq!(
        status_of(&run(&p), "time-box"),
        "pass",
        "three of five days"
    );

    let mut s = p.state();
    s.explore.started_at = "2026-09-01T09:00:00Z".into();
    p.write_state(&s);
    let out = run(&p);
    assert_eq!(status_of(&out, "time-box"), "fail", "ten of five days");
    assert!(reason_of(&out, "time-box").contains("DFA-E337"));

    // With both remedy fields set and a non-empty remedy start, the remedy
    // pair is used instead.
    let mut s = p.state();
    s.explore.remedy_started_at = "2026-09-11T00:00:00Z".into();
    s.explore.remedy_timebox_days = 1;
    p.write_state(&s);
    assert_eq!(
        status_of(&run(&p), "time-box"),
        "pass",
        "the remedy pair wins over an exhausted main box"
    );

    std::env::remove_var("DEVFORGEAI_NOW");
}

#[test]
fn verifier_pass_zero_total_is_one() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "verifier_pass"
  id = "kill-case"
  verifiers = ["kill-case-builder"]
  min_ratio = 1.0
"#,
    );
    p.write(
        ".devforgeai/reports/IDEA-003-explore.yaml",
        "schema: devforgeai/report/1\nid: IDEA-003\nphase: explore\nstatus: pass\nproduced_by: devforgeai-cli\nconsumes: []\nopen_questions: []\ngate:\n  result: PASS\n  send_back_to: ''\n  checks: []\nverifiers:\n  kill_case:\n    subagent: kill-case-builder\n    ingested_at: ''\n    passed: 0\n    total: 0\n    unit: signals\n    findings: []\nfindings: []\nhandoff: []\n",
    );
    assert_eq!(
        status_of(&run(&p), "kill-case"),
        "pass",
        "a verifier with no unit to count passes"
    );
}

#[test]
fn verifier_pass_missing_block_and_low_ratio() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "verifier_pass"
  id = "kill-case"
  verifiers = ["kill-case-builder"]
  min_ratio = 1.0
"#,
    );
    // No report at all: the block is absent.
    let out = run(&p);
    assert_eq!(status_of(&out, "kill-case"), "fail");
    assert!(reason_of(&out, "kill-case").contains("DFA-E316"));

    p.write(
        ".devforgeai/reports/IDEA-003-explore.yaml",
        "schema: devforgeai/report/1\nid: IDEA-003\nphase: explore\nstatus: pass\nproduced_by: devforgeai-cli\nconsumes: []\nopen_questions: []\ngate:\n  result: PASS\n  send_back_to: ''\n  checks: []\nverifiers:\n  kill_case:\n    subagent: kill-case-builder\n    ingested_at: ''\n    passed: 2\n    total: 5\n    unit: signals\n    findings: []\nfindings: []\nhandoff: []\n",
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "kill-case"), "fail");
    assert!(reason_of(&out, "kill-case").contains("DFA-E317"));
    assert!(reason_of(&out, "kill-case").contains("2/5"));
}

#[test]
fn report_metric_reads_length_suffix_and_an_absent_metric_fails() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "report_metric"
  id = "send-back-requirements"
  metric = "verifiers.architecture_reviewer.payload.send_back_requirements.length"
  op = "eq"
  value = 0

  [[gate.check]]
  kind = "report_metric"
  id = "absent-metric"
  metric = "verifiers.nothing.here"
  op = "eq"
  value = 0
"#,
    );
    p.write(
        ".devforgeai/reports/IDEA-003-explore.yaml",
        // One object per verifier: the envelope keys stay at the top of the
        // block and the agent's own fields sit under `payload`, so a metric
        // path into an agent field carries that segment.
        "schema: devforgeai/report/1\nid: IDEA-003\nphase: explore\nstatus: pass\nproduced_by: devforgeai-cli\nconsumes: []\nopen_questions: []\ngate:\n  result: PASS\n  send_back_to: ''\n  checks: []\nverifiers:\n  architecture_reviewer:\n    subagent: architecture-reviewer\n    ingested_at: ''\n    passed: 1\n    total: 1\n    unit: findings\n    findings: []\n    payload:\n      send_back_requirements: []\nfindings: []\nhandoff: []\n",
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "send-back-requirements"), "pass");
    assert_eq!(status_of(&out, "absent-metric"), "fail");
    assert!(reason_of(&out, "absent-metric").contains("has no metric"));
}

#[test]
fn no_open_questions_reads_the_frontmatter_list() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "no_open_questions"
  id = "brief-settled"
  docs = ["explore/brief.md"]
"#,
    );
    p.write(
        ".devforgeai/explore/brief.md",
        &common::brief("IDEA-003", "decided"),
    );
    assert_eq!(status_of(&run(&p), "brief-settled"), "pass");

    let text = common::brief("IDEA-003", "decided")
        .replace("open_questions: []", "open_questions:\n  - who pays");
    p.write(".devforgeai/explore/brief.md", &text);
    let out = run(&p);
    assert_eq!(status_of(&out, "brief-settled"), "fail");
    assert!(reason_of(&out, "brief-settled").contains("DFA-E324"));
}

#[test]
fn doc_valid_wraps_the_validator() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "doc_valid"
  id = "explore-docs"
  docs = ["explore/brief.md"]
"#,
    );
    p.write(
        ".devforgeai/explore/brief.md",
        &common::brief("IDEA-003", "decided"),
    );
    assert_eq!(status_of(&run(&p), "explore-docs"), "pass");

    let text = common::brief("IDEA-003", "decided").replace("status: decided", "status: invented");
    p.write(".devforgeai/explore/brief.md", &text);
    let out = run(&p);
    assert_eq!(status_of(&out, "explore-docs"), "fail");
    assert!(reason_of(&out, "explore-docs").contains("DFA-E326"));
}

#[test]
fn ids_resolve_from_to_form() {
    let p = seeded_with_document(
        "requirements.yaml",
        r#"  [[gate.check]]
  kind = "ids_resolve"
  id = "actor-resolves"
  from = "requirements[].actor"
  to = "personas[].id"
"#,
    );
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-003", "accepted"),
    );
    assert_eq!(status_of(&run(&p), "actor-resolves"), "pass");

    let text = common::requirements("IDEA-003", "accepted").replace(
        "    actor: PERSONA-001\n    statement: The shopper sees",
        "    actor: PERSONA-009\n    statement: The shopper sees",
    );
    p.write(".devforgeai/requirements.yaml", &text);
    let out = run(&p);
    assert_eq!(status_of(&out, "actor-resolves"), "fail");
    assert!(reason_of(&out, "actor-resolves").contains("PERSONA-009"));
}

#[test]
fn ids_resolve_prefix_form_over_the_index() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "ids_resolve"
  id = "explore-ids"
  prefixes = ["FLOW"]
"#,
    );
    p.write(
        ".devforgeai/explore/brief.md",
        &common::brief("IDEA-003", "decided"),
    );
    // The brief's Core flows table defines FLOW-001..003 in a table, which is
    // not a definition form, so a reference to one must be defined elsewhere.
    p.write(
        ".devforgeai/explore/flows.json",
        "{\"meta\":{\"schema\":\"devforgeai/flows/1\",\"id\":\"IDEA-003\",\"phase\":\"explore\",\"status\":\"recorded\",\"produced_by\":\"exploring-ideas\",\"consumes\":[],\"open_questions\":[]}}\n",
    );
    p.write(
        ".devforgeai/explore/notes.md",
        "---\nschema: devforgeai/explore-brief/1\nid: IDEA-003\n---\n\n### FLOW-001 Add to cart\n### FLOW-002 Pay\n### FLOW-003 Receipt\n",
    );
    assert_eq!(status_of(&run(&p), "explore-ids"), "pass");
}

#[test]
fn no_cycle_detects_a_cycle_through_the_root() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "no_cycle"
  id = "deferral-cycle"
  docs = ["reports/STORY-*-qa.yaml"]
  root = "STORY-014"
  from = "id"
  to = "deferrals[].target"
  prefix = "STORY"
"#,
    );
    let qa = |id: &str, target: &str| {
        format!("schema: devforgeai/qa-report/1\nid: {id}\nphase: verify\nstatus: pass\nproduced_by: validating-quality\nconsumes: []\nopen_questions: []\ndeferrals:\n  - target: {target}\n")
    };
    p.write(
        ".devforgeai/reports/STORY-014-qa.yaml",
        &qa("STORY-014", "STORY-015"),
    );
    p.write(
        ".devforgeai/reports/STORY-015-qa.yaml",
        &qa("STORY-015", "STORY-016"),
    );
    p.write(
        ".devforgeai/reports/STORY-016-qa.yaml",
        &qa("STORY-016", "STORY-020"),
    );
    assert_eq!(
        status_of(&run(&p), "deferral-cycle"),
        "pass",
        "a target outside docs is a leaf"
    );

    p.write(
        ".devforgeai/reports/STORY-016-qa.yaml",
        &qa("STORY-016", "STORY-014"),
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "deferral-cycle"), "fail");
    assert!(reason_of(&out, "deferral-cycle").contains("DFA-E340"));
}

#[test]
fn no_cycle_ignores_a_cycle_off_the_root() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "no_cycle"
  id = "deferral-cycle"
  docs = ["reports/STORY-*-qa.yaml"]
  root = "STORY-014"
  from = "id"
  to = "deferrals[].target"
"#,
    );
    let qa = |id: &str, target: &str| {
        format!("schema: devforgeai/qa-report/1\nid: {id}\nphase: verify\nstatus: pass\nproduced_by: validating-quality\nconsumes: []\nopen_questions: []\ndeferrals:\n  - target: {target}\n")
    };
    p.write(
        ".devforgeai/reports/STORY-014-qa.yaml",
        &qa("STORY-014", "STORY-020"),
    );
    // A cycle between two other stories does not pass through the root.
    p.write(
        ".devforgeai/reports/STORY-015-qa.yaml",
        &qa("STORY-015", "STORY-016"),
    );
    p.write(
        ".devforgeai/reports/STORY-016-qa.yaml",
        &qa("STORY-016", "STORY-015"),
    );
    assert_eq!(status_of(&run(&p), "deferral-cycle"), "pass");
}

#[test]
fn yaml_cites_resolves_a_union_and_reports_both_failures() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "yaml_cites"
  id = "rec-cites-obs"
  doc = "reports/reflect-2026-09-10.yaml"
  from = "recommendations"
  field = "observations"
  into = ["observations[].id"]
  min = 1
"#,
    );
    let doc = |cites: &str| {
        format!("schema: devforgeai/reflect-report/1\nid: 2026-09-10\nphase: reflect\nstatus: final\nproduced_by: improving-framework\nconsumes: []\nopen_questions: []\nobservations:\n  - id: OBS-001\n  - id: OBS-002\nrecommendations:\n  - id: REC-001\n    observations: [{cites}]\n")
    };
    p.write(
        ".devforgeai/reports/reflect-2026-09-10.yaml",
        &doc("OBS-001"),
    );
    assert_eq!(status_of(&run(&p), "rec-cites-obs"), "pass");

    p.write(".devforgeai/reports/reflect-2026-09-10.yaml", &doc(""));
    let out = run(&p);
    assert_eq!(status_of(&out, "rec-cites-obs"), "fail");
    assert!(reason_of(&out, "rec-cites-obs").contains("DFA-E346"));

    p.write(
        ".devforgeai/reports/reflect-2026-09-10.yaml",
        &doc("OBS-009"),
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "rec-cites-obs"), "fail");
    assert!(reason_of(&out, "rec-cites-obs").contains("DFA-E347"));
}

#[test]
fn no_threshold_decrease_gives_e348() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "no_threshold_decrease"
  id = "no-lowered-floor"
  doc = "reports/reflect-2026-09-10.yaml"
  from = "recommendations"
  target_field = "target.path"
  key_field = "target.key"
  value_field = "proposed_value"
  files = ["gates.toml", "config.toml"]
"#,
    );
    let doc = |value: &str| {
        format!("schema: devforgeai/reflect-report/1\nid: 2026-09-10\nphase: reflect\nstatus: final\nproduced_by: improving-framework\nconsumes: []\nopen_questions: []\nrecommendations:\n  - id: REC-001\n    target:\n      path: .devforgeai/config.toml\n      key: layer.domain.coverage_min\n    proposed_value: {value}\n")
    };
    p.write(".devforgeai/reports/reflect-2026-09-10.yaml", &doc("92.0"));
    assert_eq!(status_of(&run(&p), "no-lowered-floor"), "pass");

    p.write(".devforgeai/reports/reflect-2026-09-10.yaml", &doc("85.0"));
    let out = run(&p);
    assert_eq!(status_of(&out, "no-lowered-floor"), "fail");
    assert!(reason_of(&out, "no-lowered-floor").contains("DFA-E348"));
}

#[test]
fn files_declared_without_a_story_subject_fails() {
    // The kind is the exit code of `story files --diff`, which needs a story to
    // read a declared file set from. Spec 273 defines no skip for this kind, so
    // a gate whose subject is not a story records the failure rather than a
    // skip that counts as passing.
    let p = seeded(
        r#"  [[gate.check]]
  kind = "files_declared"
  id = "build-files"
  base = ""
"#,
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "build-files"), "fail");
    assert!(reason_of(&out, "build-files").contains("DFA-E239"));
    assert_eq!(out.data["result"], "FAIL");
}

#[test]
fn files_declared_outside_a_work_tree_fails() {
    // A project that is not a git work tree cannot produce a changed-file set,
    // and a block check that cannot be evaluated is not a check that passed.
    let p = seeded(
        r#"  [[gate.check]]
  kind = "files_declared"
  id = "build-files"
  base = ""
"#,
    );
    p.write(
        ".devforgeai/stories/STORY-014.md",
        &common::story("STORY-014", "building", &[], "# A story\n\n## Files\n\n| Path | Kind | Layer |\n|---|---|---|\n| src/a.rs | source | domain |\n"),
    );
    p.set_phase("explore", "STORY-014");
    let mut ctx = p.ctx();
    let args = devforgeai::cli::GateCheckArgs {
        phase: "explore".into(),
        id: Some("STORY-014".into()),
        partial: false,
        no_run: false,
    };
    let out = gate::check(&mut ctx, &args).expect("gate check runs");
    assert_eq!(status_of(&out, "build-files"), "fail");
    assert!(
        reason_of(&out, "build-files").contains("DFA-E239"),
        "reason was {}",
        reason_of(&out, "build-files")
    );
}

#[test]
fn release_stories_checks_status_and_report() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "release_stories"
  id = "release-stories"
  path = "releases/v0.3.0.yaml"
  require_status = ["built", "released"]
  require_report = "verify"
  require_result = "PASS"
"#,
    );
    p.write(
        ".devforgeai/releases/v0.3.0.yaml",
        "schema: devforgeai/release/1\nid: v0.3.0\nphase: release\nstatus: released\nproduced_by: releasing-software\nconsumes: []\nopen_questions: []\nstories:\n  - id: STORY-014\n",
    );
    p.write(
        ".devforgeai/stories/STORY-014.md",
        &common::story("STORY-014", "built", &[], "# T\n"),
    );
    p.write(
        ".devforgeai/reports/STORY-014-verify.yaml",
        &common::report("STORY-014", "verify", "PASS"),
    );
    assert_eq!(status_of(&run(&p), "release-stories"), "pass");

    p.write(
        ".devforgeai/stories/STORY-014.md",
        &common::story("STORY-014", "ready", &[], "# T\n"),
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "release-stories"), "fail");
    assert!(reason_of(&out, "release-stories").contains("DFA-E341"));
}

// ------------------------------------------------------------- condition keys

#[test]
fn required_when_false_records_skip() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "field_is_date"
  id = "park-has-revisit"
  path = "explore/decision.yaml"
  field = "revisit_on"
  after_field = "decided_on"
  required_when = { path = "explore/decision.yaml", field = "decision", equals = "park" }
"#,
    );
    // The decision is `promote`, so the check is not required.
    let out = run(&p);
    assert_eq!(status_of(&out, "park-has-revisit"), "skip");
    assert_eq!(reason_of(&out, "park-has-revisit"), "not_required");
}

#[test]
fn null_when_and_empty_when() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "field_is_date"
  id = "park-has-revisit"
  path = "explore/decision.yaml"
  field = "revisit_on"
  null_when = { path = "explore/decision.yaml", field = "decision", in = ["kill", "promote"] }

  [[gate.check]]
  kind = "length_between"
  id = "promote-carries-forward"
  path = "explore/decision.yaml"
  field = "carry_forward"
  min = 7
  max = 7
  empty_when = { path = "explore/decision.yaml", field = "decision", in = ["kill", "park"] }
"#,
    );
    // A promote decision: `revisit_on` must be absent, `carry_forward` is
    // checked normally and holds none, which is below the minimum.
    let out = run(&p);
    assert_eq!(
        status_of(&out, "park-has-revisit"),
        "pass",
        "revisit_on is absent"
    );
    assert_eq!(status_of(&out, "promote-carries-forward"), "fail");

    // A park decision: `carry_forward` must be empty, and it is.
    p.write(
        ".devforgeai/explore/decision.yaml",
        &common::decision("IDEA-003", "park"),
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "promote-carries-forward"), "pass");

    // A promote decision that still carries revisit_on fails the null rule.
    p.write(
        ".devforgeai/explore/decision.yaml",
        "schema: devforgeai/explore-decision/1\nid: IDEA-003\nphase: explore\nstatus: recorded\nproduced_by: exploring-ideas\nconsumes: []\nopen_questions: []\ndecision: promote\ndecided_on: 2026-09-08\nrevisit_on: 2026-10-08\ncarry_forward: []\n",
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "park-has-revisit"), "fail");
}

#[test]
fn skip_when_doc_exists_records_skip() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "elapsed_days_at_most"
  id = "time-box"
  started_field = "explore.started_at"
  limit_field = "explore.timebox_days"
  skip_when = { doc_exists = "explore/decision.yaml" }
"#,
    );
    // The decision exists, so the time box no longer binds.
    let out = run(&p);
    assert_eq!(status_of(&out, "time-box"), "skip");
    assert_eq!(reason_of(&out, "time-box"), "condition");
}

#[test]
fn gate_document_supplies_the_default_path() {
    let p = Project::new();
    p.write(
        ".devforgeai/gates.toml",
        r#"schema = "devforgeai/gates/1"
cli_min_version = "1.0.0"

[[gate]]
phase = "explore"
requires = ""
document = "explore/decision.yaml"

  [[gate.check]]
  kind = "file_exists"
  id = "baseline-exists"
  paths = ["explore/decision.yaml"]

  [[gate.check]]
  kind = "field_in_enum"
  id = "decision-enum"
  field = "decision"
  values = ["kill", "park", "promote"]
"#,
    );
    p.write(
        ".devforgeai/explore/decision.yaml",
        &common::decision("IDEA-003", "promote"),
    );
    p.set_phase("explore", "IDEA-003");
    assert_eq!(
        status_of(&run(&p), "decision-enum"),
        "pass",
        "the check inherited the gate's document as its path"
    );
}

#[test]
fn id_token_substituted_in_paths() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "file_exists"
  id = "subject-report"
  paths = ["reports/{id}-{phase}.yaml"]
"#,
    );
    p.write(
        ".devforgeai/reports/IDEA-003-explore.yaml",
        &common::report("IDEA-003", "explore", "PASS"),
    );
    assert_eq!(status_of(&run(&p), "subject-report"), "pass");
}

// ------------------------------------------------------------- gate behaviour

#[test]
fn check_pass_writes_report_and_state() {
    let p = seeded("");
    let out = run(&p);
    assert_eq!(out.data["result"], "PASS");
    assert_eq!(out.exit, Some(0));

    assert!(p.exists(".devforgeai/reports/IDEA-003-explore.yaml"));
    let text = p.read(".devforgeai/reports/IDEA-003-explore.yaml");
    assert!(text.contains("result: PASS"));
    assert!(text.contains("produced_by: devforgeai-cli"));

    let s = p.state();
    assert_eq!(s.last_gate.result, "PASS");
    assert_eq!(s.last_gate.phase, "explore");
    assert_eq!(s.last_gate.id, "IDEA-003");
    assert_eq!(
        s.last_gate.report,
        ".devforgeai/reports/IDEA-003-explore.yaml"
    );
}

#[test]
fn check_fail_names_failing_check() {
    let p = seeded("");
    p.write(
        ".devforgeai/explore/decision.yaml",
        &common::decision("IDEA-003", "maybe"),
    );
    let out = run(&p);
    assert_eq!(out.data["result"], "FAIL");
    assert_eq!(out.exit, Some(1));
    assert_eq!(p.state().last_gate.failed_checks, vec!["baseline-enum"]);
}

#[test]
fn check_send_back_exits_two() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "doc_valid"
  id = "brief-valid"
  docs = ["explore/brief.md"]
  on_fail = "send_back"
"#,
    );
    let text = common::brief("IDEA-003", "decided").replace("status: decided", "status: invented");
    p.write(".devforgeai/explore/brief.md", &text);

    let out = run(&p);
    assert_eq!(out.data["result"], "SEND BACK");
    assert_eq!(out.exit, Some(2));
    assert_eq!(p.state().last_gate.result, "SEND_BACK");
}

#[test]
fn check_warn_check_does_not_change_result() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "file_exists"
  id = "advisory"
  severity = "warn"
  paths = ["nowhere.md"]
"#,
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "advisory"), "fail");
    assert_eq!(out.data["result"], "PASS", "a warn check leaves the result");
    assert_eq!(out.exit, Some(0));
}

#[test]
fn check_preserves_existing_verifiers_block() {
    let p = seeded("");
    p.write(
        ".devforgeai/reports/IDEA-003-explore.yaml",
        "schema: devforgeai/report/1\nid: IDEA-003\nphase: explore\nstatus: pass\nproduced_by: devforgeai-cli\nconsumes: []\nopen_questions: []\ngate:\n  result: NOT_RUN\n  send_back_to: ''\n  checks: []\nverifiers:\n  kill_case:\n    subagent: kill-case-builder\n    ingested_at: '2026-09-10T10:00:00Z'\n    passed: 3\n    total: 3\n    unit: signals\n    findings: []\nfindings: []\nhandoff: []\n",
    );
    run(&p);
    let text = p.read(".devforgeai/reports/IDEA-003-explore.yaml");
    assert!(
        text.contains("kill-case-builder"),
        "the ingested block survives"
    );
    assert!(text.contains("passed: 3"));
}

#[test]
fn check_partial_leaves_state_untouched_and_exits_zero() {
    let p = seeded("");
    p.write(
        ".devforgeai/explore/decision.yaml",
        &common::decision("IDEA-003", "maybe"),
    );
    let before = p.read(".devforgeai/state.toml");

    let mut ctx = p.ctx();
    let args = GateCheckArgs {
        phase: "explore".into(),
        id: Some("IDEA-003".into()),
        partial: true,
        no_run: false,
    };
    let out = gate::check(&mut ctx, &args).expect("partial runs");

    assert_eq!(out.exit, Some(0), "--partial exits 0 whatever the result");
    assert_eq!(
        out.data["checks"].as_array().expect("checks").len(),
        0,
        "--partial evaluates the four command kinds only"
    );
    assert_eq!(p.read(".devforgeai/state.toml"), before);
    assert!(p
        .read(".devforgeai/reports/IDEA-003-explore.yaml")
        .contains("partial: true"));
}

#[test]
fn check_defaults_the_id_from_state_and_reports_e011_when_empty() {
    let p = seeded("");
    let mut ctx = p.ctx();
    let args = GateCheckArgs {
        phase: "explore".into(),
        id: None,
        partial: false,
        no_run: false,
    };
    let out = gate::check(&mut ctx, &args).expect("the active id is used");
    assert_eq!(out.data["id"], "IDEA-003");

    // Reflect has no `[active]` key, so omitting `--id` there is DFA-E011.
    let mut ctx = p.ctx();
    let args = GateCheckArgs {
        phase: "reflect".into(),
        id: None,
        partial: false,
        no_run: false,
    };
    let err = gate::check(&mut ctx, &args).expect_err("no active id for reflect");
    assert_eq!(err.code(), "DFA-E011");
    assert_eq!(err.exit(), 3);
}

#[test]
fn check_design_gives_e300() {
    let p = seeded("");
    let mut ctx = p.ctx();
    let args = GateCheckArgs {
        phase: "design".into(),
        id: Some("UI-002".into()),
        partial: false,
        no_run: false,
    };
    let err = gate::check(&mut ctx, &args).expect_err("Design holds no gate");
    assert_eq!(err.code(), "DFA-E300");
}

#[test]
fn check_json_shape() {
    let p = seeded("");
    let out = run(&p);
    for key in [
        "phase",
        "id",
        "result",
        "send_back_to",
        "partial",
        "degraded",
        "report",
        "checks",
        "findings",
    ] {
        assert!(out.data.get(key).is_some(), "{key} is in the data object");
    }
    let first = &out.data["checks"][0];
    for key in ["id", "kind", "status", "severity", "reason", "evidence"] {
        assert!(first.get(key).is_some(), "checks[].{key}");
    }
}

#[test]
fn check_human_output_columns() {
    let p = seeded("");
    let out = run(&p);
    assert_eq!(out.human[0], "Gate      explore · IDEA-003");
    assert!(out.human.iter().any(|l| l.starts_with("Result    PASS")));
    assert!(out
        .human
        .iter()
        .any(|l| l.starts_with("Report    .devforgeai/reports/IDEA-003-explore.yaml")));
}

#[test]
fn degraded_skips_the_command_kinds() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "tests_pass"
  id = "build-tests"
  stacks = []
"#,
    );
    p.write_config(&devforgeai::config::Config {
        degraded: true,
        generated_at: "2026-09-10T14:02:11Z".into(),
        ..Default::default()
    });

    let out = run(&p);
    assert_eq!(status_of(&out, "build-tests"), "skip");
    assert_eq!(reason_of(&out, "build-tests"), "degraded");
    assert_eq!(out.data["result"], "PASS", "a skip counts as passing");
    assert_eq!(out.data["degraded"], true);
}

#[test]
fn no_run_skips_the_command_kinds() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "tests_pass"
  id = "build-tests"
  stacks = []
"#,
    );
    let mut ctx = p.ctx();
    let args = GateCheckArgs {
        phase: "explore".into(),
        id: Some("IDEA-003".into()),
        partial: false,
        no_run: true,
    };
    let out = gate::check(&mut ctx, &args).expect("--no-run executes no command");
    assert_eq!(status_of(&out, "build-tests"), "skip");
    assert_eq!(reason_of(&out, "build-tests"), "no_run");
}

#[test]
fn send_back_to_resolves_from_the_blocking_finding_prefixes() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "ids_resolve"
  id = "explore-ids"
  prefixes = ["REQ", "CON"]
  on_fail = "send_back"
"#,
    );
    // A CON and a REQ reference that resolve nowhere. CON maps to constitute
    // and REQ to discover; the earlier phase in section 5 order wins.
    p.write(
        ".devforgeai/explore/brief.md",
        &common::brief("IDEA-003", "decided").replace(
            "# Order checkout",
            "# Order checkout\n\nSee REQ-004 and CON-002.\n",
        ),
    );

    let out = run(&p);
    assert_eq!(out.data["result"], "SEND BACK");
    assert_eq!(
        out.data["send_back_to"], "discover",
        "REQ is earlier than CON in the section 5 order"
    );
}

// ------------------------------------------------ the fail-closed rule per kind
//
// Every kind reports `fail` when the document, field, path, pattern, list, or
// state field it reads is absent, null, unparsable, or of the wrong type. The
// tests below are the negatives the suite lacked, one per shortcut.

#[test]
fn required_when_over_an_absent_document_fails() {
    // A condition that could not be evaluated silenced the check it guarded:
    // one missing file skipped three block checks and the gate reached PASS.
    let p = seeded(
        r#"  [[gate.check]]
  kind = "file_exists"
  id = "conditional"
  paths = ["explore/brief.md"]
  min_count = 1
  required_when = { path = "explore/absent.yaml", field = "decision", equals = "park" }
"#,
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "conditional"), "fail");
    assert!(reason_of(&out, "conditional").contains("DFA-E200"));
}

#[test]
fn skip_when_over_an_unparsable_document_fails() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "file_exists"
  id = "conditional"
  paths = ["explore/brief.md"]
  min_count = 1
  skip_when = { path = "explore/broken.yaml", field = "decision", equals = "kill" }
"#,
    );
    p.write(".devforgeai/explore/broken.yaml", "decision: [kill\n");
    let out = run(&p);
    assert_eq!(status_of(&out, "conditional"), "fail");
    assert!(reason_of(&out, "conditional").contains("DFA-E401"));
}

#[test]
fn null_when_reads_the_checks_own_document_and_field() {
    // `null_when` read `path` directly rather than through the gate document,
    // so on every kind whose key set holds no `path` the rule passed whatever
    // was on disk.
    let p = seeded_with_document(
        "explore/decision.yaml",
        r#"  [[gate.check]]
  kind = "length_between"
  id = "carry-forward-null"
  field = "carry_forward"
  min = 1
  null_when = { path = "explore/decision.yaml", field = "decision", equals = "promote" }
"#,
    );
    // `decision: promote` makes the rule apply, and `carry_forward: []` is not
    // null, so the rule is evaluated rather than passing unconditionally.
    let out = run(&p);
    assert_eq!(status_of(&out, "carry-forward-null"), "fail");
    assert!(reason_of(&out, "carry-forward-null").contains("DFA-E332"));
}

#[test]
fn field_in_enum_null_value_fails() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "field_in_enum"
  id = "story-status"
  path = "stories/STORY-014.md"
  field = "status"
  values = ["building", "built"]
"#,
    );
    p.write(
        ".devforgeai/stories/STORY-014.md",
        "---\nschema: devforgeai/story/1\nid: STORY-014\nphase: plan\nstatus:\nproduced_by: planning-work\nconsumes: []\nopen_questions: []\n---\n\n# A story\n",
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "story-status"), "fail");
    assert!(reason_of(&out, "story-status").contains("DFA-E322"));
    assert!(
        reason_of(&out, "story-status").contains("null"),
        "the message renders the held null rather than an empty string"
    );
}

#[test]
fn length_between_distinguishes_null_from_a_scalar() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "length_between"
  id = "epic-not-empty"
  path = "requirements.yaml"
  field = "epics[].requirements"
  min = 1
"#,
    );
    // A key written with no value is a list of zero, which fails `min = 1`.
    p.write(
        ".devforgeai/requirements.yaml",
        "schema: devforgeai/requirements/1\nid: IDEA-003\nphase: discover\nstatus: draft\nproduced_by: discovering-requirements\nconsumes: []\nopen_questions: []\nepics:\n  - id: EPIC-001\n    requirements:\n",
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "epic-not-empty"), "fail");
    assert!(reason_of(&out, "epic-not-empty").contains("DFA-E331"));

    // A scalar at the location is a defect in the document, not a list.
    p.write(
        ".devforgeai/requirements.yaml",
        "schema: devforgeai/requirements/1\nid: IDEA-003\nphase: discover\nstatus: draft\nproduced_by: discovering-requirements\nconsumes: []\nopen_questions: []\nepics:\n  - id: EPIC-001\n    requirements: REQ-001\n",
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "epic-not-empty"), "fail");
    assert!(reason_of(&out, "epic-not-empty").contains("not a list"));
}

#[test]
fn no_open_questions_fails_on_an_absent_document_and_a_non_list() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "no_open_questions"
  id = "brief-questions"
  docs = ["explore/brief.md"]
"#,
    );
    let out = run(&p);
    assert_eq!(
        status_of(&out, "brief-questions"),
        "fail",
        "an absent document is not a document with no open questions"
    );

    p.write(
        ".devforgeai/explore/brief.md",
        &common::brief("IDEA-003", "decided")
            .replace("open_questions: []", "open_questions: who pays"),
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "brief-questions"), "fail");
    assert!(reason_of(&out, "brief-questions").contains("DFA-E324"));
}

#[test]
fn no_open_questions_over_an_empty_docs_list_fails() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "no_open_questions"
  id = "nothing-named"
  docs = []
"#,
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "nothing-named"), "fail");
    assert!(reason_of(&out, "nothing-named").contains("DFA-E345"));
}

#[test]
fn doc_valid_over_an_empty_docs_list_fails() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "doc_valid"
  id = "nothing-validated"
  docs = []
"#,
    );
    let out = run(&p);
    assert_eq!(
        status_of(&out, "nothing-validated"),
        "fail",
        "evidence of documents: 0 is not an all-green run"
    );
}

#[test]
fn doc_valid_over_a_path_the_table_does_not_cover_fails() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "doc_valid"
  id = "unmatched"
  docs = ["explore/notes.md"]
"#,
    );
    p.write(".devforgeai/explore/notes.md", "# Notes\n");
    let out = run(&p);
    assert_eq!(status_of(&out, "unmatched"), "fail");
    assert!(reason_of(&out, "unmatched").contains("DFA-E200"));
}

#[test]
fn doc_valid_reports_every_invalid_document() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "doc_valid"
  id = "plan-docs"
  docs = ["stories/STORY-*.md"]
"#,
    );
    for id in ["STORY-014", "STORY-015", "STORY-016"] {
        p.write(
            &format!(".devforgeai/stories/{id}.md"),
            &format!("---\nschema: devforgeai/story/1\nid: {id}\nphase: plan\nstatus: invented\nproduced_by: planning-work\nconsumes: []\nopen_questions: []\n---\n\n# A story\n"),
        );
    }
    let out = run(&p);
    assert_eq!(status_of(&out, "plan-docs"), "fail");
    let reason = reason_of(&out, "plan-docs");
    for id in ["STORY-014", "STORY-015", "STORY-016"] {
        assert!(
            reason.contains(id),
            "the report states the whole of the work remaining; {id} is absent from {reason}"
        );
    }
}

#[test]
fn a_leading_dot_glob_resolves_against_the_project_root() {
    // Spec 245 gives a leading-dot path the project root. The literal branch
    // honoured it and the glob branch did not, so a glob over project-root
    // files matched nothing and passed.
    let p = seeded(
        r#"  [[gate.check]]
  kind = "no_open_questions"
  id = "prototype-questions"
  docs = [".explore-prototype/*.md"]
"#,
    );
    p.write(
        ".explore-prototype/sketch.md",
        "---\nschema: devforgeai/explore-brief/1\nid: IDEA-003\nphase: explore\nstatus: draft\nproduced_by: exploring-ideas\nconsumes: []\nopen_questions: [who pays]\n---\n\n# Sketch\n",
    );
    let out = run(&p);
    assert_eq!(
        status_of(&out, "prototype-questions"),
        "fail",
        "the glob matched the project-root file"
    );
    assert!(reason_of(&out, "prototype-questions").contains("DFA-E324"));
}

#[test]
fn column_contains_all_separates_a_typo_from_an_empty_array() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "column_contains_all"
  id = "remedy-flows-present"
  path = "explore/brief.md"
  section = "Core flows"
  column = "ID"
  state_field = "explore.remedy_flowz"
"#,
    );
    p.write(
        ".devforgeai/explore/brief.md",
        &common::brief("IDEA-003", "decided"),
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "remedy-flows-present"), "fail");
    assert!(reason_of(&out, "remedy-flows-present").contains("DFA-E345"));

    // The same check over the real field, which state holds empty, passes for
    // the right reason.
    let p = seeded(
        r#"  [[gate.check]]
  kind = "column_contains_all"
  id = "remedy-flows-present"
  path = "explore/brief.md"
  section = "Core flows"
  column = "ID"
  state_field = "explore.remedy_flows"
"#,
    );
    p.write(
        ".devforgeai/explore/brief.md",
        &common::brief("IDEA-003", "decided"),
    );
    assert_eq!(status_of(&run(&p), "remedy-flows-present"), "pass");
}

#[test]
fn elapsed_days_fails_closed_on_both_halves() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "elapsed_days_at_most"
  id = "time-box"
  started_field = "explore.started_at"
  limit_field = "explore.timebox_days"
"#,
    );
    // An absent start disarmed the time box permanently.
    let out = run(&p);
    assert_eq!(status_of(&out, "time-box"), "fail");
    assert!(reason_of(&out, "time-box").contains("DFA-E345"));

    // So did an unparsable one, by editing a single state string.
    let mut s = p.state();
    s.explore.started_at = "yesterday".into();
    s.explore.timebox_days = 3;
    p.write_state(&s);
    let out = run(&p);
    assert_eq!(status_of(&out, "time-box"), "fail");
    assert!(reason_of(&out, "time-box").contains("DFA-E337"));
}

#[test]
fn row_count_between_separates_an_absent_section_from_an_empty_one() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "row_count_between"
  id = "flow-count"
  path = "explore/brief.md"
  section = "Core flows"
  max = 5
"#,
    );
    p.write(
        ".devforgeai/explore/brief.md",
        &common::brief("IDEA-003", "decided").replace("## Core flows", "## Other flows"),
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "flow-count"), "fail");
    assert!(
        reason_of(&out, "flow-count").contains("Core flows"),
        "the reason names the absent section"
    );

    // A section that exists and holds nothing reports zero.
    p.write(
        ".devforgeai/explore/brief.md",
        &common::brief("IDEA-003", "decided")
            .replace("| FLOW-001 | Add to cart |\n", "")
            .replace("| FLOW-002 | Pay |\n", "")
            .replace("| FLOW-003 | Receipt |\n", ""),
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "flow-count"), "pass");
}

#[test]
fn row_count_between_does_not_count_a_subheading_as_a_body_line() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "row_count_between"
  id = "one-signal"
  path = "explore/brief.md"
  section = "Success signal"
  min = 1
  max = 1
"#,
    );
    p.write(
        ".devforgeai/explore/brief.md",
        "---\nschema: devforgeai/explore-brief/1\nid: IDEA-003\nphase: explore\nstatus: decided\nproduced_by: exploring-ideas\nconsumes: []\nopen_questions: []\n---\n\n# Order checkout\n\n## Success signal\n\n### Detail\n\nOrders per day reach 100.\n",
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "one-signal"), "pass");
}

#[test]
fn set_cover_reports_a_cover_value_naming_no_record() {
    let p = seeded_with_document(
        "requirements.yaml",
        r#"  [[gate.check]]
  kind = "set_cover"
  id = "no-ungrouped-req"
  cover = "epics[].requirements"
  universe = "requirements[].id"
"#,
    );
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-003", "accepted").replace(
            "    requirements: [REQ-001, REQ-002]\n",
            "    requirements: [REQ-001, REQ-002, REQ-999]\n",
        ),
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "no-ungrouped-req"), "fail");
    assert!(
        reason_of(&out, "no-ungrouped-req").contains("REQ-999"),
        "a fabricated requirement id inside an epic is named"
    );
}

#[test]
fn set_cover_reads_the_universe_leaf() {
    let p = seeded_with_document(
        "keys.yaml",
        r#"  [[gate.check]]
  kind = "set_cover"
  id = "keyed"
  cover = "epics[].requirements"
  universe = "requirements[].key"
"#,
    );
    p.write(
        ".devforgeai/keys.yaml",
        "requirements:\n  - key: REQ-001\n  - key: REQ-002\nepics:\n  - id: EPIC-001\n    requirements: [REQ-001]\n",
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "keyed"), "fail");
    assert!(reason_of(&out, "keyed").contains("REQ-002"));
}

#[test]
fn field_is_date_parses_in_the_named_format() {
    let p = seeded_with_document(
        "explore/dates.yaml",
        r#"  [[gate.check]]
  kind = "field_is_date"
  id = "dated"
  field = "decided_on"
  format = "%d/%m/%Y"
"#,
    );
    p.write(".devforgeai/explore/dates.yaml", "decided_on: 08/09/2026\n");
    assert_eq!(status_of(&run(&p), "dated"), "pass");

    // The ISO shape is not that format.
    p.write(".devforgeai/explore/dates.yaml", "decided_on: 2026-09-08\n");
    assert_eq!(status_of(&run(&p), "dated"), "fail");
}

#[test]
fn field_is_date_refuses_an_impossible_day() {
    let p = seeded_with_document(
        "explore/dates.yaml",
        r#"  [[gate.check]]
  kind = "field_is_date"
  id = "dated"
  field = "decided_on"
"#,
    );
    p.write(
        ".devforgeai/explore/dates.yaml",
        "decided_on: \"2026-99-99\"\n",
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "dated"), "fail");
    assert!(reason_of(&out, "dated").contains("DFA-E330"));
}

#[test]
fn field_is_date_checks_every_value_a_location_names() {
    let p = seeded_with_document(
        "explore/dates.yaml",
        r#"  [[gate.check]]
  kind = "field_is_date"
  id = "milestones"
  field = "milestones[].due"
"#,
    );
    p.write(
        ".devforgeai/explore/dates.yaml",
        "milestones:\n  - due: 2026-01-01\n  - due: not a date\n  - due: \"2026-03-99\"\n",
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "milestones"), "fail");
}

#[test]
fn ids_resolve_treats_a_null_at_from_as_unresolved() {
    let p = seeded_with_document(
        "requirements.yaml",
        r#"  [[gate.check]]
  kind = "ids_resolve"
  id = "actor-resolves"
  from = "requirements[].actor"
  to = "personas[].id"
"#,
    );
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-003", "accepted").replace("actor: PERSONA-001", "actor:"),
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "actor-resolves"), "fail");
    assert!(reason_of(&out, "actor-resolves").contains("DFA-E325"));
}

#[test]
fn fields_present_path_names_a_document_even_with_a_gate_document() {
    let p = seeded_with_document(
        "requirements.yaml",
        r#"  [[gate.check]]
  kind = "fields_present"
  id = "release-accepted"
  path = "releases/{id}.yaml"
  field = "accepted_by"
"#,
    );
    p.write(".devforgeai/releases/IDEA-003.yaml", "accepted_by: Bryan\n");
    assert_eq!(status_of(&run(&p), "release-accepted"), "pass");
}

#[test]
fn fields_present_with_no_field_tests_the_document_itself() {
    let p = seeded_with_document(
        "requirements.yaml",
        r#"  [[gate.check]]
  kind = "fields_present"
  id = "document-holds-something"
"#,
    );
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-003", "accepted"),
    );
    assert_eq!(status_of(&run(&p), "document-holds-something"), "pass");
}

#[test]
fn fields_present_needs_every_value_a_location_names() {
    let p = seeded_with_document(
        "requirements.yaml",
        r#"  [[gate.check]]
  kind = "fields_present"
  id = "rationale-present"
  field = "requirements[].rationale"
"#,
    );
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-003", "accepted").replace("rationale: trust", "rationale:"),
    );
    let out = run(&p);
    assert_eq!(
        status_of(&out, "rationale-present"),
        "fail",
        "one non-null value of two does not carry the other"
    );
}

#[test]
fn fields_present_does_not_count_an_empty_mapping_as_a_value() {
    let p = seeded_with_document(
        "requirements.yaml",
        r#"  [[gate.check]]
  kind = "fields_present"
  id = "req-fields"
  collection = "requirements"
  fields = ["rationale"]
"#,
    );
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-003", "accepted").replace("rationale: trust", "rationale: {}"),
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "req-fields"), "fail");
}

#[test]
fn file_exists_with_an_empty_paths_list_fails() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "file_exists"
  id = "release-file"
  paths = []
"#,
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "release-file"), "fail");
    assert!(reason_of(&out, "release-file").contains("DFA-E323"));
}

#[test]
fn file_exists_does_not_count_an_empty_directory() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "file_exists"
  id = "release-file"
  paths = ["releases/IDEA-003.yaml"]
  min_count = 1
"#,
    );
    std::fs::create_dir_all(p.dot().join("releases/IDEA-003.yaml")).expect("mkdir");
    let out = run(&p);
    assert_eq!(status_of(&out, "release-file"), "fail");
    assert!(reason_of(&out, "release-file").contains("DFA-E323"));
}

#[test]
fn verifier_pass_refuses_a_block_that_counts_nothing() {
    let seed = |block: &str| -> Project {
        let p = seeded(
            r#"  [[gate.check]]
  kind = "verifier_pass"
  id = "kill-case-answered"
  verifiers = ["kill-case-builder"]
  min_ratio = 1.0
"#,
        );
        p.write(
            ".devforgeai/reports/IDEA-003-explore.yaml",
            &common::report("IDEA-003", "explore", "PASS").replace(
                "findings: []",
                &format!("verifiers:\n  kill_case:\n{block}findings: []"),
            ),
        );
        p
    };

    // Neither key present: the spec's `total == 0` free pass belongs to a
    // verifier that counted, not to a block that carries no counts.
    let out = run(&seed("    subagent: kill-case-builder\n"));
    assert_eq!(status_of(&out, "kill-case-answered"), "fail");
    assert!(reason_of(&out, "kill-case-answered").contains("DFA-E317"));

    // Quoted counts are not integers.
    let out = run(&seed("    passed: \"2\"\n    total: \"2\"\n"));
    assert_eq!(status_of(&out, "kill-case-answered"), "fail");

    // More passed than counted is not a ratio.
    let out = run(&seed("    passed: 3\n    total: 2\n"));
    assert_eq!(status_of(&out, "kill-case-answered"), "fail");

    // Below the ratio.
    let out = run(&seed("    passed: 0\n    total: 1\n"));
    assert_eq!(status_of(&out, "kill-case-answered"), "fail");

    // A genuine zero-unit run still passes, which spec 282 grants.
    let out = run(&seed("    passed: 0\n    total: 0\n"));
    assert_eq!(status_of(&out, "kill-case-answered"), "pass");
}

#[test]
fn verifier_pass_names_config_when_the_verifier_is_unregistered() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "verifier_pass"
  id = "unknown-verifier"
  verifiers = ["nonexistent-agent"]
  min_ratio = 1.0
"#,
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "unknown-verifier"), "fail");
    assert!(
        reason_of(&out, "unknown-verifier").contains("config.toml"),
        "the message sends the reader to the file that can be fixed"
    );
}

#[test]
fn report_metric_eq_holds_above_magnitude_one() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "report_metric"
  id = "count"
  metric = "verifiers.kill_case.total"
  op = "eq"
  value = 214.0
"#,
    );
    p.write(
        ".devforgeai/reports/IDEA-003-explore.yaml",
        &common::report("IDEA-003", "explore", "PASS").replace(
            "findings: []",
            "verifiers:\n  kill_case:\n    passed: 214\n    total: 214\nfindings: []",
        ),
    );
    assert_eq!(status_of(&run(&p), "count"), "pass");
}

#[test]
fn no_cycle_refuses_an_empty_glob_and_an_unreadable_document() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "no_cycle"
  id = "deferral-cycle"
  docs = ["reports/STORY-*-qa.yaml"]
  from = "id"
  to = "deferrals[].target"
"#,
    );
    let out = run(&p);
    assert_eq!(
        status_of(&out, "deferral-cycle"),
        "fail",
        "a glob matching nothing holds no cycle for the wrong reason"
    );

    // A document in the set that does not parse was dropped, and dropping it
    // turns the edge into it into a leaf.
    p.write(
        ".devforgeai/reports/STORY-014-qa.yaml",
        "id: STORY-014\ndeferrals:\n  - target: STORY-015\n",
    );
    p.write(
        ".devforgeai/reports/STORY-015-qa.yaml",
        "id: STORY-015\ndeferrals:\n\t- target: STORY-014\n",
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "deferral-cycle"), "fail");
    assert!(reason_of(&out, "deferral-cycle").contains("DFA-E401"));
}

#[test]
fn release_stories_refuses_a_list_that_names_nothing() {
    let head = "schema: devforgeai/release/1\nid: IDEA-003\nphase: release\nstatus: released\nproduced_by: releasing-software\nconsumes: []\nopen_questions: []\n";
    let p = seeded(
        r#"  [[gate.check]]
  kind = "release_stories"
  id = "release-stories"
  path = "releases/{id}.yaml"
"#,
    );

    p.write(
        ".devforgeai/releases/IDEA-003.yaml",
        &format!("{head}story_list: []\n"),
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "release-stories"), "fail");
    assert!(reason_of(&out, "release-stories").contains("DFA-E345"));

    p.write(
        ".devforgeai/releases/IDEA-003.yaml",
        &format!("{head}stories: []\n"),
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "release-stories"), "fail");
    assert!(reason_of(&out, "release-stories").contains("DFA-E341"));

    p.write(
        ".devforgeai/releases/IDEA-003.yaml",
        &format!("{head}stories:\n  - title: no id here\n"),
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "release-stories"), "fail");
    assert!(reason_of(&out, "release-stories").contains("carries no id"));
}

#[test]
fn no_threshold_decrease_locates_the_floor_by_shape() {
    let seed = |key: &str, value: &str| -> Project {
        let p = seeded(
            r#"  [[gate.check]]
  kind = "no_threshold_decrease"
  id = "no-lowered-floor"
  doc = "reports/reflect.yaml"
"#,
        );
        p.write(
            ".devforgeai/reports/reflect.yaml",
            &format!(
                "recommendations:\n  - id: REC-001\n    target:\n      path: .devforgeai/gates.toml\n      key: {key}\n    proposed_value: {value}\n"
            ),
        );
        p
    };

    // The explore gate carries no min_ratio floor, so restating the shipped
    // default is not a decrease.
    assert_eq!(
        status_of(
            &run(&seed("gate.explore.kill-case-answered.min_ratio", "0.0")),
            "no-lowered-floor"
        ),
        "pass"
    );
    // The verify gate does carry one.
    assert_eq!(
        status_of(
            &run(&seed("gate.verify.verify-acs.min_ratio", "0.5")),
            "no-lowered-floor"
        ),
        "fail"
    );
    // A quoted number is still a number.
    let out = run(&seed("layer.domain.coverage_min", "\"40\""));
    assert_eq!(status_of(&out, "no-lowered-floor"), "fail");
    assert!(reason_of(&out, "no-lowered-floor").contains("DFA-E348"));
    // A threshold named in no recognised shape cannot be located, which is a
    // defect in the recommendation rather than a reason to let it through.
    let out = run(&seed("coverage_min", "10.0"));
    assert_eq!(status_of(&out, "no-lowered-floor"), "fail");
}

#[test]
fn column_matches_reports_a_row_with_no_cell_in_the_column() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "column_matches"
  id = "flow-id-shape"
  path = "explore/brief.md"
  section = "Core flows"
  column = "Flow"
  pattern = "^[A-Za-z ]+$"
"#,
    );
    p.write(
        ".devforgeai/explore/brief.md",
        &common::brief("IDEA-003", "decided").replace("| FLOW-002 | Pay |", "| FLOW-002 |"),
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "flow-id-shape"), "fail");
    assert!(reason_of(&out, "flow-id-shape").contains("no cell"));
}

// ---------------------------------------------------- the command-running kinds

#[test]
fn coverage_with_no_coverage_paths_is_e312() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "coverage_min"
  id = "build-coverage"
  source = "read"
"#,
    );
    p.write_config(&devforgeai::config::Config {
        degraded: false,
        stack: vec![devforgeai::config::Stack {
            id: "rust".into(),
            coverage_format: "none".into(),
            test_command: "exit 0".into(),
            ..Default::default()
        }],
        generated_at: "2026-09-10T14:02:11Z".into(),
        ..Default::default()
    });
    let out = run(&p);
    assert_eq!(
        status_of(&out, "build-coverage"),
        "fail",
        "a story cannot reach Release with no coverage evidence"
    );
    assert!(reason_of(&out, "build-coverage").contains("DFA-E312"));
}

#[test]
fn tests_pass_with_a_selection_that_names_nothing_is_e310() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "tests_pass"
  id = "build-tests"
  stacks = ["typo-rust"]
"#,
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "build-tests"), "fail");
    let reason = reason_of(&out, "build-tests");
    assert!(reason.contains("DFA-E310"));
    assert!(
        reason.contains("typo-rust"),
        "the message names the unmatched id"
    );
}

#[test]
fn tests_pass_with_no_stack_at_all_is_e310() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "tests_pass"
  id = "build-tests"
  stacks = []
"#,
    );
    p.write_config(&devforgeai::config::Config {
        degraded: false,
        stack: Vec::new(),
        generated_at: "2026-09-10T14:02:11Z".into(),
        ..Default::default()
    });
    let out = run(&p);
    assert_eq!(status_of(&out, "build-tests"), "fail");
    assert!(reason_of(&out, "build-tests").contains("DFA-E310"));
}

#[test]
fn lint_with_a_selection_that_names_nothing_is_e314() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "lint_clean"
  id = "build-lint"
  stacks = ["node"]
"#,
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "build-lint"), "fail");
    assert!(reason_of(&out, "build-lint").contains("DFA-E314"));
}

#[test]
fn a_degraded_project_skips_complexity_clean_too() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "complexity_clean"
  id = "build-complexity"
  stacks = []
"#,
    );
    p.write_config(&devforgeai::config::Config {
        degraded: true,
        stack: vec![devforgeai::config::Stack {
            id: "rust".into(),
            complexity_command: "exit 1".into(),
            ..Default::default()
        }],
        generated_at: "2026-09-10T14:02:11Z".into(),
        ..Default::default()
    });
    let out = run(&p);
    assert_eq!(status_of(&out, "build-complexity"), "skip");
    assert_eq!(reason_of(&out, "build-complexity"), "degraded");
}

/// A project whose explore gate carries `extra` and whose `[release]` block
/// names `command` as the API symbol command.
fn seeded_with_symbols(extra: &str, command: &str) -> Project {
    let p = seeded(extra);
    let mut cfg = devforgeai::config::Config {
        degraded: false,
        stack: vec![devforgeai::config::Stack {
            id: "rust".into(),
            test_command: "exit 0".into(),
            timeout_secs: 60,
            ..Default::default()
        }],
        generated_at: "2026-09-10T14:02:11Z".into(),
        ..Default::default()
    };
    cfg.release.api_symbols_command = command.to_string();
    p.write_config(&cfg);
    p.write(
        ".devforgeai/releases/IDEA-003.yaml",
        "schema: devforgeai/release/1\nid: IDEA-003\nphase: release\nstatus: released\nproduced_by: releasing-software\nconsumes: []\nopen_questions: []\ndocs:\n  api: [docs/api.md]\n",
    );
    p
}

const DOCS_COVER: &str = r#"  [[gate.check]]
  kind = "docs_cover"
  id = "docs-cover"
  path = "releases/{id}.yaml"
  min_ratio = 1.0
"#;

#[test]
fn docs_cover_fails_when_the_command_fails() {
    let p = seeded_with_symbols(DOCS_COVER, "exit 7");
    let out = run(&p);
    assert_eq!(
        status_of(&out, "docs-cover"),
        "fail",
        "a non-zero exit is not an empty API"
    );
    assert!(reason_of(&out, "docs-cover").contains("DFA-E344"));
}

#[test]
fn docs_cover_ignores_a_line_with_no_tab() {
    // Spec: each symbol line is `<kind>\t<symbol>\t<path>`. A line without a
    // tab is not a symbol, so a warning on stderr is not a phantom one; a
    // command that prints only such lines printed no symbol at all.
    let p = seeded_with_symbols(DOCS_COVER, "echo place_order");
    let out = run(&p);
    assert_eq!(status_of(&out, "docs-cover"), "fail");
    assert!(reason_of(&out, "docs-cover").contains("printed no symbol line"));
}

#[test]
fn docs_cover_matches_a_heading_exactly() {
    // A literal tab in the echoed text, so the line carries the three columns
    // spec 296 fixes under every shell the runner uses.
    let p = seeded_with_symbols(DOCS_COVER, "echo fn\tget\tsrc/a.rs");
    p.write("docs/api.md", "# API\n\n### budget_getter\n\nText.\n");
    let out = run(&p);
    assert_eq!(
        status_of(&out, "docs-cover"),
        "fail",
        "### budget_getter does not document the symbol get"
    );

    p.write("docs/api.md", "# API\n\n### get\n\nText.\n");
    assert_eq!(status_of(&run(&p), "docs-cover"), "pass");
}

#[test]
fn docs_cover_under_no_run_skips() {
    let p = seeded_with_symbols(DOCS_COVER, "printf 'x' > sentinel.txt");
    let mut ctx = p.ctx();
    let args = GateCheckArgs {
        phase: "explore".into(),
        id: Some("IDEA-003".into()),
        partial: false,
        no_run: true,
    };
    let out = gate::check(&mut ctx, &args).expect("--no-run executes no command");
    assert_eq!(status_of(&out, "docs-cover"), "skip");
    assert_eq!(reason_of(&out, "docs-cover"), "no_run");
    assert!(
        !p.exists("sentinel.txt"),
        "--no-run spawned no child process"
    );
}

#[test]
fn deploy_manifest_reads_a_bare_string_entry() {
    // An entry that is a string rather than a mapping was dropped, so a
    // non-empty list read as empty and the `none` rule accepted it.
    let p = seeded(
        r#"  [[gate.check]]
  kind = "deploy_manifest"
  id = "release-manifest"
  path = "releases/{id}.yaml"
  platform = "none"
"#,
    );
    p.write(
        ".devforgeai/releases/IDEA-003.yaml",
        "schema: devforgeai/release/1\nid: IDEA-003\nphase: release\nstatus: released\nproduced_by: releasing-software\nconsumes: []\nopen_questions: []\ndeploy:\n  manifests: [k8s/deployment.yaml]\n",
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "release-manifest"), "fail");
    assert!(reason_of(&out, "release-manifest").contains("DFA-E342"));
}

#[test]
fn deploy_manifest_scans_the_workflow_for_a_secret() {
    // The secret scan ran over `deploy.manifests[]` alone, so the workflow the
    // platform rules read was never scanned.
    let p = seeded(
        r#"  [[gate.check]]
  kind = "deploy_manifest"
  id = "release-manifest"
  path = "releases/{id}.yaml"
  platform = "github-actions"
"#,
    );
    p.write(
        ".devforgeai/releases/IDEA-003.yaml",
        "schema: devforgeai/release/1\nid: IDEA-003\nphase: release\nstatus: released\nproduced_by: releasing-software\nconsumes: []\nopen_questions: []\ndeploy:\n  manifests: []\n",
    );
    p.write(
        ".github/workflows/deploy.yml",
        "on: push\njobs:\n  deploy:\n    steps:\n      - run: echo\n    env:\n      api_key: AKIAIOSFODNN7EXAMPLE\n",
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "release-manifest"), "fail");
    assert!(reason_of(&out, "release-manifest").contains("DFA-E343"));
}

#[test]
fn a_trailing_reference_comment_does_not_excuse_a_literal_secret() {
    let p = seeded(
        r#"  [[gate.check]]
  kind = "deploy_manifest"
  id = "release-manifest"
  path = "releases/{id}.yaml"
  platform = "github-actions"
"#,
    );
    p.write(
        ".devforgeai/releases/IDEA-003.yaml",
        "schema: devforgeai/release/1\nid: IDEA-003\nphase: release\nstatus: released\nproduced_by: releasing-software\nconsumes: []\nopen_questions: []\ndeploy:\n  manifests: []\n",
    );
    p.write(
        ".github/workflows/deploy.yml",
        "on: push\njobs:\n  deploy:\n    env:\n      token: ghp_realsecrethere1234  # or use ${TOKEN}\n",
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "release-manifest"), "fail");
    assert!(reason_of(&out, "release-manifest").contains("DFA-E343"));
}

#[test]
fn story_valid_reads_the_scope_rather_than_the_gate_subject() {
    // Passing the gate subject as a single-story override discarded the scope,
    // so `scope = "all"` read one story and a malformed sibling was invisible.
    let p = seeded(
        r#"  [[gate.check]]
  kind = "story_valid"
  id = "plan-stories"
  scope = "all"
"#,
    );
    p.write(
        ".devforgeai/stories/STORY-014.md",
        &common::story(
            "STORY-014",
            "ready",
            &[],
            "# A story\n\n## Acceptance criteria\n\n- AC-001: Given a cart, when I pay, then an order exists\n\n## Files\n\n| Path | Kind | Layer |\n|---|---|---|\n| src/a.rs | source | domain |\n",
        ),
    );
    p.write(
        ".devforgeai/stories/STORY-015.md",
        &common::story("STORY-015", "ready", &[], "# A story with no AC\n"),
    );
    p.set_phase("explore", "STORY-014");
    let mut ctx = p.ctx();
    let args = GateCheckArgs {
        phase: "explore".into(),
        id: Some("STORY-014".into()),
        partial: false,
        no_run: false,
    };
    let out = gate::check(&mut ctx, &args).expect("gate check runs");
    assert_eq!(status_of(&out, "plan-stories"), "fail");
    assert!(
        reason_of(&out, "plan-stories").contains("found 2 problems"),
        "the scope, not the subject, decides the set, so both stories were read; reason was {}",
        reason_of(&out, "plan-stories")
    );
}

#[test]
fn complexity_clean_carries_a_code_of_its_own() {
    // A reason whose first token is not a `DFA-Ennn` code is silent in the
    // envelope's `errors[]`, so the one kind the table had no row for now
    // carries DFA-E349 on both of its failing paths.
    let p = seeded(
        r#"  [[gate.check]]
  kind = "complexity_clean"
  id = "build-complexity"
  stacks = []
"#,
    );
    p.write_config(&devforgeai::config::Config {
        degraded: false,
        stack: vec![devforgeai::config::Stack {
            id: "rust".into(),
            complexity_command: "exit 3".into(),
            timeout_secs: 60,
            ..Default::default()
        }],
        generated_at: "2026-09-10T14:02:11Z".into(),
        ..Default::default()
    });
    let out = run(&p);
    assert_eq!(status_of(&out, "build-complexity"), "fail");
    assert!(
        reason_of(&out, "build-complexity").starts_with("DFA-E349"),
        "reason was {}",
        reason_of(&out, "build-complexity")
    );

    // And on a selection that names no stack this project holds.
    let p = seeded(
        r#"  [[gate.check]]
  kind = "complexity_clean"
  id = "build-complexity"
  stacks = ["node"]
"#,
    );
    let out = run(&p);
    assert_eq!(status_of(&out, "build-complexity"), "fail");
    assert!(reason_of(&out, "build-complexity").starts_with("DFA-E349"));
}

#[test]
fn every_failing_reason_opens_with_a_code() {
    // The envelope's `errors[]` is built from the leading `DFA-` token of each
    // failing reason, so a reason without one reports nothing to the caller.
    // A freshly initialised project fails most of the shipped explore gate,
    // which is the widest single sweep available here.
    let p = Project::new();
    p.set_phase("explore", "IDEA-003");
    let mut ctx = p.ctx();
    let args = GateCheckArgs {
        phase: "explore".into(),
        id: Some("IDEA-003".into()),
        partial: false,
        no_run: false,
    };
    let out = gate::check(&mut ctx, &args).expect("gate check runs");
    let checks = out.data["checks"].as_array().expect("checks");
    let mut failed = 0usize;
    for c in checks {
        if c["status"] != "fail" {
            continue;
        }
        failed += 1;
        let reason = c["reason"].as_str().unwrap_or("");
        let head = reason.split_whitespace().next().unwrap_or("");
        let coded = head
            .strip_prefix("DFA-E")
            .map(|n| n.len() == 3 && n.chars().all(|c| c.is_ascii_digit()))
            .unwrap_or(false);
        assert!(
            coded,
            "check '{}' of kind '{}' reports '{reason}', which opens with no code",
            c["id"], c["kind"]
        );
    }
    assert!(failed >= 3, "the sweep saw {failed} failing checks");
}

#[test]
fn docs_cover_tolerates_a_carriage_return_at_the_end_of_a_line() {
    // Windows tooling writes CRLF; the line is still the three tab-separated
    // columns once the `\r` is off.
    let p = seeded_with_symbols(DOCS_COVER, "echo fn\tget\tsrc/a.rs\r");
    p.write("docs/api.md", "# API\n\n### get\n\nText.\n");
    assert_eq!(status_of(&run(&p), "docs-cover"), "pass");
}

// ------------------------------------------------- the ingest-to-gate path
//
// A registered verifier prints one object: the envelope keys at the top and
// the agent's own fields under `payload`. These run the whole path the
// constitute gate depends on — `report ingest` writes the block, `gate check`
// reads it through the shipped default metrics — so a field dropped between
// the two is a failing test rather than a check that silently reads nothing.

/// One `devforgeai/verifier/1` envelope with `payload` merged in.
fn verifier_envelope(subagent: &str, id: &str, payload: serde_json::Value) -> String {
    serde_json::to_string(&serde_json::json!({
        "schema": "devforgeai/verifier/1",
        "subagent": subagent,
        "id": id,
        "passed": 1,
        "total": 1,
        "unit": "checks",
        "findings": [],
        "payload": payload,
    }))
    .expect("the envelope serialises")
}

/// Ingest one envelope for `subagent` at phase constitute.
fn ingest(p: &Project, subagent: &str, payload: serde_json::Value) {
    let mut ctx = p.ctx();
    let args = devforgeai::cli::ReportIngestArgs {
        subagent: subagent.to_string(),
        source: "-".to_string(),
        id: Some("IDEA-001".to_string()),
        phase: Some("constitute".to_string()),
    };
    let out = devforgeai::cmd::report::ingest(
        &mut ctx,
        &args,
        Some(verifier_envelope(subagent, "IDEA-001", payload)),
    )
    .expect("ingest runs");
    assert!(
        out.warnings.is_empty(),
        "ingest of {subagent} warned: {:?}",
        out.warnings
    );
}

/// The shipped constitute gate over the project's own context set.
fn constitute_gate(p: &Project) -> Outcome {
    let mut ctx = p.ctx();
    let args = GateCheckArgs {
        phase: "constitute".into(),
        id: Some("IDEA-001".into()),
        partial: false,
        no_run: false,
    };
    gate::check(&mut ctx, &args).expect("gate check runs")
}

/// A project at phase constitute whose context audit is clean.
fn constituted() -> Project {
    let p = Project::new();
    common::write_context_set(&p, "accepted");
    p.set_phase("constitute", "IDEA-001");
    p
}

#[test]
fn an_ingested_payload_reaches_the_shipped_constitute_metrics() {
    let p = constituted();
    ingest(
        &p,
        "architecture-reviewer",
        serde_json::json!({ "blocking_findings": 0, "send_back_requirements": [] }),
    );
    ingest(
        &p,
        "alignment-auditor",
        serde_json::json!({ "blocking_findings": 0 }),
    );

    let out = constitute_gate(&p);
    assert_eq!(
        status_of(&out, "CTX-AUDIT"),
        "pass",
        "the fixture audits clean"
    );
    assert_eq!(status_of(&out, "REVIEW"), "pass");
    assert_eq!(status_of(&out, "ALIGNMENT"), "pass");
    assert_eq!(status_of(&out, "SEND-BACK-REQUIREMENTS"), "pass");
    assert_eq!(out.data["result"], "PASS");
}

#[test]
fn the_constitute_result_follows_the_ingested_payload_values() {
    // The same gate, the same fixture, one payload number apart.
    let p = constituted();
    ingest(
        &p,
        "alignment-auditor",
        serde_json::json!({ "blocking_findings": 0 }),
    );
    ingest(
        &p,
        "architecture-reviewer",
        serde_json::json!({ "blocking_findings": 2, "send_back_requirements": [] }),
    );
    let out = constitute_gate(&p);
    assert_eq!(status_of(&out, "REVIEW"), "fail");
    assert!(reason_of(&out, "REVIEW").contains("DFA-E338"));
    assert_eq!(out.data["result"], "FAIL");

    // A non-empty send-back list sends the gate back rather than failing it.
    let p = constituted();
    ingest(
        &p,
        "alignment-auditor",
        serde_json::json!({ "blocking_findings": 0 }),
    );
    ingest(
        &p,
        "architecture-reviewer",
        serde_json::json!({ "blocking_findings": 0, "send_back_requirements": ["REQ-004"] }),
    );
    let out = constitute_gate(&p);
    assert_eq!(status_of(&out, "REVIEW"), "pass");
    assert_eq!(status_of(&out, "SEND-BACK-REQUIREMENTS"), "fail");
    assert_eq!(out.data["result"], "SEND BACK");
}

#[test]
fn an_unknown_payload_key_round_trips_untouched() {
    // The CLI models nothing inside `payload`, so a verifier may report
    // whatever its own schema names and a rewrite of the report preserves it.
    let p = constituted();
    ingest(
        &p,
        "architecture-reviewer",
        serde_json::json!({
            "blocking_findings": 0,
            "send_back_requirements": [],
            "invented": { "depth": 3, "notes": ["a", "b"] },
        }),
    );
    // A second ingest rewrites the report around the first block.
    ingest(
        &p,
        "alignment-auditor",
        serde_json::json!({ "blocking_findings": 0 }),
    );
    // And so does a gate run.
    constitute_gate(&p);

    let text = p.read(".devforgeai/reports/IDEA-001-constitute.yaml");
    let value: serde_yaml_ng::Value = serde_yaml_ng::from_str(&text).expect("the report parses");
    let invented = devforgeai::ypath::resolve(
        &value,
        "verifiers.architecture_reviewer.payload.invented.depth",
    )
    .expect("the unknown key survives");
    assert_eq!(
        invented.first().and_then(|v| v.as_i64()),
        Some(3),
        "the report was rewritten twice and kept the key: {text}"
    );
    assert!(
        text.contains("notes"),
        "the whole subtree survives, not the scalar alone"
    );
}

#[test]
fn a_long_check_id_does_not_run_into_its_kind() {
    let p = common::Project::new();
    p.set_phase("explore", "IDEA-003");

    let mut ctx = p.ctx();
    let args = devforgeai::cli::GateCheckArgs {
        phase: "explore".to_string(),
        id: Some("IDEA-003".to_string()),
        partial: false,
        no_run: false,
    };
    let out = devforgeai::cmd::gate::check(&mut ctx, &args).expect("the gate runs");

    // `kill-case-answered` is exactly the column width, and the line read
    // `kill-case-answeredverifier_pass`.
    let line = out
        .human
        .iter()
        .find(|l| l.contains("kill-case-answered"))
        .expect("the check line");
    assert!(
        line.contains("kill-case-answered verifier_pass"),
        "columns are separated whatever their width: {line:?}"
    );
    for l in out.human.iter().filter(|l| l.starts_with("  ")) {
        assert!(
            !l.contains("verifier_pass") || l.contains(" verifier_pass"),
            "no column runs into the next: {l:?}"
        );
    }
}
