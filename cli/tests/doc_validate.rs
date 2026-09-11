//! `doc validate`: the path set, the producer check, `--allocate`, and the
//! per-file JSON shape.

mod common;

use common::Project;
use devforgeai::cli::DocValidateArgs;
use devforgeai::cmd::doc;

fn args() -> DocValidateArgs {
    DocValidateArgs {
        paths: Vec::new(),
        all: false,
        allocate: None,
        producer_check: false,
        stdin_content: false,
    }
}

fn valid_story(p: &Project, id: &str) {
    p.write(
        &format!(".devforgeai/stories/{id}.md"),
        &common::story(id, "ready", &[], "# Order checkout\n"),
    );
}

#[test]
fn validate_all_walks_devforgeai() {
    let p = Project::new();
    valid_story(&p, "STORY-014");
    valid_story(&p, "STORY-015");
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-003", "drafting"),
    );

    let mut ctx = p.ctx();
    let a = DocValidateArgs {
        all: true,
        ..args()
    };
    let out = doc::validate(&mut ctx, &a, None).expect("validate --all");

    assert_eq!(out.exit, Some(0), "every document is valid");
    assert_eq!(
        out.data["checked"], 3,
        "three documents matched a doc-type row"
    );
    assert_eq!(out.data["failed"], 0);
}

#[test]
fn validate_unmatched_path_is_skipped() {
    let p = Project::new();
    valid_story(&p, "STORY-014");
    // A skill may keep scratch files under `.devforgeai/`.
    p.write(".devforgeai/scratch.md", "not a document\n");
    p.write(".devforgeai/stories/notes.md", "loose notes\n");

    let mut ctx = p.ctx();
    let a = DocValidateArgs {
        all: true,
        ..args()
    };
    let out = doc::validate(&mut ctx, &a, None).expect("validate --all");

    assert_eq!(
        out.data["checked"], 1,
        "the scratch files carry no diagnostic"
    );
    assert_eq!(out.exit, Some(0));
}

#[test]
fn validate_reports_a_failing_file() {
    let p = Project::new();
    // `phase` and `status` out of order.
    p.write(
        ".devforgeai/stories/STORY-014.md",
        "---\nschema: devforgeai/story/1\nid: STORY-014\nstatus: ready\nphase: plan\nproduced_by: planning-work\nconsumes: []\nopen_questions: []\n---\n\n# T\n",
    );
    let mut ctx = p.ctx();
    let a = DocValidateArgs {
        paths: vec![".devforgeai/stories/STORY-014.md".into()],
        ..args()
    };
    let out = doc::validate(&mut ctx, &a, None).expect("validate");
    assert_eq!(out.exit, Some(1));
    assert_eq!(out.data["failed"], 1);
    let codes: Vec<&str> = out.data["files"][0]["errors"]
        .as_array()
        .expect("errors")
        .iter()
        .filter_map(|e| e["code"].as_str())
        .collect();
    assert!(codes.contains(&"DFA-E205"), "keys out of order: {codes:?}");
}

#[test]
fn validate_json_lists_per_file_errors() {
    let p = Project::new();
    valid_story(&p, "STORY-014");
    p.write(
        ".devforgeai/stories/STORY-015.md",
        "---\nschema: devforgeai/story/1\nid: STORY-015\nphase: plan\nstatus: nonsense\nproduced_by: planning-work\nconsumes: []\nopen_questions: []\n---\n\n# T\n",
    );

    let mut ctx = p.ctx();
    let a = DocValidateArgs {
        all: true,
        ..args()
    };
    let out = doc::validate(&mut ctx, &a, None).expect("validate");

    assert_eq!(out.data["checked"], 2);
    assert_eq!(out.data["failed"], 1);
    let files = out.data["files"].as_array().expect("files");
    let bad = files
        .iter()
        .find(|f| f["id"] == "STORY-015")
        .expect("the failing file");
    assert_eq!(bad["doc_type"], "story");
    assert_eq!(bad["valid"], false);
    assert_eq!(bad["errors"][0]["code"], "DFA-E208");
    let good = files
        .iter()
        .find(|f| f["id"] == "STORY-014")
        .expect("the passing file");
    assert_eq!(good["valid"], true);
    assert_eq!(good["status"], "ready");
}

#[test]
fn allocate_prints_id_alone() {
    let p = Project::new();
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-003", "drafting"),
    );

    let mut ctx = p.ctx();
    let a = DocValidateArgs {
        allocate: Some("REQ".into()),
        ..args()
    };
    let out = doc::validate(&mut ctx, &a, None).expect("allocate");

    assert_eq!(out.human, vec!["REQ-003"], "the ID alone and nothing else");
    assert_eq!(out.data["prefix"], "REQ");
    assert_eq!(out.data["id"], "REQ-003");
    assert_eq!(out.exit, None, "allocate exits 0");
}

#[test]
fn allocate_unknown_prefix_gives_e214() {
    let p = Project::new();
    let mut ctx = p.ctx();
    let a = DocValidateArgs {
        allocate: Some("WIDGET".into()),
        ..args()
    };
    let err = doc::validate(&mut ctx, &a, None).expect_err("outside the closed list");
    assert_eq!(err.code(), "DFA-E214");
    assert_eq!(err.exit(), 3);
}

#[test]
fn no_path_no_all_no_allocate_gives_e011() {
    let p = Project::new();
    let mut ctx = p.ctx();
    let err = doc::validate(&mut ctx, &args(), None).expect_err("nothing to do");
    assert_eq!(err.code(), "DFA-E011");
    assert_eq!(err.exit(), 3);
}

#[test]
fn producer_check_matching_phase_exits_zero() {
    let p = Project::new();
    valid_story(&p, "STORY-014");
    p.set_phase("plan", "SPRINT-001");

    let mut ctx = p.ctx();
    let a = DocValidateArgs {
        paths: vec![".devforgeai/stories/STORY-014.md".into()],
        producer_check: true,
        ..args()
    };
    let out = doc::validate(&mut ctx, &a, None).expect("plan writes the story document");
    assert_eq!(out.data["allowed"], true);
    assert_eq!(out.data["expected_producer"], "planning-work");
    assert_eq!(out.data["current"]["phase"], "plan");
}

#[test]
fn producer_check_mismatch_gives_e212() {
    let p = Project::new();
    valid_story(&p, "STORY-014");
    p.set_phase("build", "STORY-014");

    let mut ctx = p.ctx();
    let a = DocValidateArgs {
        paths: vec![".devforgeai/stories/STORY-014.md".into()],
        producer_check: true,
        ..args()
    };
    let err = doc::validate(&mut ctx, &a, None).expect_err("Plan owns the story document");
    assert_eq!(err.code(), "DFA-E212");
    assert_eq!(err.exit(), 1);
    let msg = &err.diag().expect("diag").message;
    assert!(msg.contains("planning-work"), "names the producer: {msg}");
}

#[test]
fn producer_check_design_allowed_from_any_phase() {
    let p = Project::new();
    p.write(
        ".devforgeai/ui-specs/UI-002.md",
        "---\nschema: devforgeai/ui-spec/1\nid: UI-002\nphase: design\nstatus: draft\nproduced_by: designing-interfaces\nconsumes: []\nopen_questions: []\n---\n\n# Checkout screen\n",
    );
    for phase in ["explore", "plan", "build", "verify"] {
        p.set_phase(phase, "X-001");
        let mut ctx = p.ctx();
        let a = DocValidateArgs {
            paths: vec![".devforgeai/ui-specs/UI-002.md".into()],
            producer_check: true,
            ..args()
        };
        let out = doc::validate(&mut ctx, &a, None)
            .unwrap_or_else(|e| panic!("design is cross-cutting, from {phase}: {}", e.code()));
        assert_eq!(out.data["allowed"], true, "{phase}");
    }
}

#[test]
fn producer_check_stdin_content_without_file() {
    let p = Project::new();
    p.set_phase("plan", "SPRINT-001");
    let pending = common::story("STORY-020", "draft", &[], "# Not yet written\n");

    let mut ctx = p.ctx();
    let a = DocValidateArgs {
        paths: vec![".devforgeai/stories/STORY-020.md".into()],
        producer_check: true,
        stdin_content: true,
        ..args()
    };
    assert!(
        !p.exists(".devforgeai/stories/STORY-020.md"),
        "the write has not landed yet"
    );
    let out = doc::validate(&mut ctx, &a, Some(pending))
        .expect("the file being absent is not an error in this mode");
    assert_eq!(out.data["allowed"], true);
}

#[test]
fn producer_check_stdin_content_catches_a_bad_frontmatter() {
    let p = Project::new();
    p.set_phase("plan", "SPRINT-001");
    let pending = "---\nschema: devforgeai/story/1\nid: STORY-020\nphase: plan\nstatus: invented\nproduced_by: planning-work\nconsumes: []\nopen_questions: []\n---\n\n# T\n";

    let mut ctx = p.ctx();
    let a = DocValidateArgs {
        paths: vec![".devforgeai/stories/STORY-020.md".into()],
        producer_check: true,
        stdin_content: true,
        ..args()
    };
    let err = doc::validate(&mut ctx, &a, Some(pending.to_string()))
        .expect_err("the pending content is checked");
    assert_eq!(err.code(), "DFA-E208");
}

// --- the producer rule compares skills, not phases ------------------------

#[test]
fn producer_check_refuses_a_report_the_binary_owns() {
    let p = Project::new();
    p.set_phase("verify", "STORY-014");
    p.write(
        ".devforgeai/reports/STORY-014-verify.yaml",
        "schema: devforgeai/report/1\nid: STORY-014\nphase: verify\nstatus: pass\nproduced_by: devforgeai-cli\nconsumes: []\nopen_questions: []\n",
    );

    let mut ctx = p.ctx();
    let a = DocValidateArgs {
        paths: vec![".devforgeai/reports/STORY-014-verify.yaml".into()],
        producer_check: true,
        ..args()
    };
    let err = doc::validate(&mut ctx, &a, None).expect_err("the CLI owns the gate report");
    assert_eq!(err.code(), "DFA-E212");
    assert_eq!(err.exit(), 1);
}

#[test]
fn producer_check_refuses_a_build_report_from_build() {
    let p = Project::new();
    p.set_phase("build", "STORY-014");
    p.write(
        ".devforgeai/reports/STORY-014-build.yaml",
        "schema: devforgeai/report/1\nid: STORY-014\nphase: build\nstatus: pass\nproduced_by: devforgeai-cli\nconsumes: []\nopen_questions: []\n",
    );

    let mut ctx = p.ctx();
    let a = DocValidateArgs {
        paths: vec![".devforgeai/reports/STORY-014-build.yaml".into()],
        producer_check: true,
        ..args()
    };
    // The phase of the row and the current phase coincide here, which is
    // exactly the case a phase comparison waves through.
    let err = doc::validate(&mut ctx, &a, None).expect_err("the CLI owns the build report");
    assert_eq!(err.code(), "DFA-E212");
}

#[test]
fn producer_check_allows_the_owning_skill() {
    let p = Project::new();
    p.set_phase("plan", "SPRINT-001");
    p.write(
        ".devforgeai/stories/STORY-014.md",
        &common::story("STORY-014", "draft", &[], "# Checkout\n"),
    );

    let mut ctx = p.ctx();
    let a = DocValidateArgs {
        paths: vec![".devforgeai/stories/STORY-014.md".into()],
        producer_check: true,
        ..args()
    };
    let out = doc::validate(&mut ctx, &a, None).expect("Plan owns the story document");
    assert_eq!(out.data["allowed"], true, "the fix does not over-refuse");
}

#[test]
fn producer_check_refuses_the_plan_document_from_verify() {
    let p = Project::new();
    p.set_phase("verify", "STORY-014");
    p.write(
        ".devforgeai/stories/STORY-014.md",
        &common::story("STORY-014", "draft", &[], "# Checkout\n"),
    );

    let mut ctx = p.ctx();
    let a = DocValidateArgs {
        paths: vec![".devforgeai/stories/STORY-014.md".into()],
        producer_check: true,
        ..args()
    };
    let err = doc::validate(&mut ctx, &a, None).expect_err("Plan owns the story document");
    assert_eq!(err.code(), "DFA-E212");
}

// --- a table row is a definition site -------------------------------------

/// A brief whose `## Core flows` table states three flows, as the framework's
/// own template does, plus the `## Mockups` and `## Prototype` tables that
/// cite them.
fn brief_with_flow_table() -> String {
    concat!(
        "---\n",
        "schema: devforgeai/explore-brief/1\n",
        "id: IDEA-003\n",
        "phase: explore\n",
        "status: decided\n",
        "produced_by: exploring-ideas\n",
        "consumes: []\n",
        "open_questions: []\n",
        "---\n\n",
        "# Order checkout\n\n",
        "## Core flows\n\n",
        "| ID | Actor | Trigger | Steps | Outcome |\n",
        "|---|---|---|---|---|\n",
        "| FLOW-001 | Shopper | Adds an item | open -> add | The cart holds it |\n",
        "| FLOW-002 | Shopper | Pays | open -> pay | The order exists |\n",
        "| FLOW-003 | Shopper | Returns | open -> view | The receipt renders |\n\n",
        "## Mockups\n\n",
        "| Flow | Screen | Path | State |\n",
        "|---|---|---|---|\n",
        "| FLOW-001 | FLOW-001-01 | .devforgeai/explore/mockups/FLOW-001-01.html | default |\n\n",
        "## Prototype\n\n",
        "| Field | Value |\n",
        "|---|---|\n",
        "| Flows covered | FLOW-001, FLOW-002 |\n",
    )
    .to_string()
}

#[test]
fn a_table_row_defines_the_id_in_its_id_column() {
    let p = Project::new();
    p.write(".devforgeai/explore/brief.md", &brief_with_flow_table());

    let index = devforgeai::doc::ids::build(p.root());

    // The `## Core flows` table is the only place a flow is stated, so a row
    // there is its definition site, the way a heading or a list item is.
    for id in ["FLOW-001", "FLOW-002", "FLOW-003"] {
        assert_eq!(
            index.definitions.get(id).map(Vec::len),
            Some(1),
            "{id} is defined exactly once: {:?}",
            index.definitions.get(id)
        );
    }
}

#[test]
fn a_table_without_an_id_column_defines_nothing() {
    let p = Project::new();
    p.write(".devforgeai/explore/brief.md", &brief_with_flow_table());

    let index = devforgeai::doc::ids::build(p.root());

    // `## Mockups` also leads with a `FLOW-nnn`, and `## Prototype` cites two
    // in a value cell. Reading either as a definition would make every flow a
    // duplicate, so only a column headed `ID` defines.
    assert_eq!(index.definitions.get("FLOW-001").map(Vec::len), Some(1));
    assert!(
        index.references.get("FLOW-001").map(Vec::len).unwrap_or(0) >= 2,
        "the other two tables cite it: {:?}",
        index.references.get("FLOW-001")
    );
}

#[test]
fn a_brief_from_the_template_validates() {
    let p = Project::new();
    p.write(".devforgeai/explore/brief.md", &brief_with_flow_table());

    let mut ctx = p.ctx();
    let args = devforgeai::cli::DocValidateArgs {
        paths: vec![".devforgeai/explore/brief.md".into()],
        all: false,
        allocate: None,
        producer_check: false,
        stdin_content: false,
    };
    let out = devforgeai::cmd::doc::validate(&mut ctx, &args, None)
        .expect("a brief written from the framework's own template validates");

    // `DFA-E210` said the flows were referenced and defined nowhere; `DFA-W202`
    // said the body cited an id the frontmatter did not consume.
    assert_eq!(out.exit.unwrap_or(0), 0, "{:?}", out.warnings);
    for code in ["DFA-E210", "DFA-W202"] {
        assert!(
            !out.warnings.iter().any(|d| d.code == code),
            "{code}: {:?}",
            out.warnings
        );
    }
}

#[test]
fn allocate_flow_follows_the_table() {
    let p = Project::new();
    p.write(".devforgeai/explore/brief.md", &brief_with_flow_table());

    let mut ctx = p.ctx();
    let args = devforgeai::cli::DocValidateArgs {
        paths: Vec::new(),
        all: false,
        allocate: Some("FLOW".to_string()),
        producer_check: false,
        stdin_content: false,
    };
    let out = devforgeai::cmd::doc::validate(&mut ctx, &args, None).expect("allocate");

    // Handing back `FLOW-001` over a brief that already states it is how two
    // flows end up sharing an id.
    assert_eq!(out.data["id"], "FLOW-004");
}

#[test]
fn the_handoff_counts_the_flows_the_table_states() {
    let p = Project::new();
    p.write(".devforgeai/explore/brief.md", &brief_with_flow_table());
    p.set_phase("explore", "IDEA-003");

    let mut ctx = p.ctx();
    let out = devforgeai::cmd::handoff::run(&mut ctx, None, None).expect("handoff");

    let done = out
        .human
        .iter()
        .find(|l| l.starts_with("Done"))
        .expect("the Done line");
    assert!(
        done.contains("3 flows"),
        "the flows are counted from their definition sites: {done}"
    );
}

// --- PREFIX-000 is a placeholder, not an id -------------------------------

#[test]
fn a_placeholder_id_is_neither_defined_nor_referenced() {
    let p = Project::new();
    // What the decision template leaves: a `carry_forward` note naming the ADR
    // the project's own reason will one day become.
    p.write(
        ".devforgeai/explore/decision.yaml",
        concat!(
            "schema: devforgeai/explore-decision/1\n",
            "id: IDEA-003\n",
            "phase: explore\n",
            "status: recorded\n",
            "produced_by: exploring-ideas\n",
            "consumes: []\n",
            "open_questions: []\n",
            "decision: promote\n",
            "decided_on: 2026-09-11\n",
            "reason: The scan found no product that reconciles a week in one screen.\n",
            "revisit_on: null\n",
            "elapsed_days: 2\n",
            "remedied_flows: []\n",
            "carry_forward:\n",
            "  - path: .devforgeai/explore/decision.yaml\n",
            "    sections: []\n",
            "    becomes: ADR-000, the reason this project exists\n",
            "    consumer: establishing-context\n",
        ),
    );

    let index = devforgeai::doc::ids::build(p.root());
    assert!(
        !index.definitions.contains_key("ADR-000"),
        "allocation starts at 001, so 000 can never be a subject"
    );
    assert!(
        !index.references.contains_key("ADR-000"),
        "and a placeholder resolves to nothing, so it is not a reference either"
    );

    let mut ctx = p.ctx();
    let args = devforgeai::cli::DocValidateArgs {
        paths: vec![".devforgeai/explore/decision.yaml".into()],
        all: false,
        allocate: None,
        producer_check: false,
        stdin_content: false,
    };
    let out = devforgeai::cmd::doc::validate(&mut ctx, &args, None)
        .expect("a decision written from its own template validates");
    assert_eq!(out.exit.unwrap_or(0), 0, "{:?}", out.warnings);
    assert!(
        !out.warnings.iter().any(|d| d.code == "DFA-E210"),
        "{:?}",
        out.warnings
    );
}

#[test]
fn a_placeholder_is_ignored_wherever_it_appears() {
    assert!(devforgeai::doc::ids::is_placeholder("ADR-000"));
    assert!(devforgeai::doc::ids::is_placeholder("STORY-000"));
    assert!(!devforgeai::doc::ids::is_placeholder("ADR-001"));
    assert!(!devforgeai::doc::ids::is_placeholder("STORY-010"));
    assert!(!devforgeai::doc::ids::is_placeholder("not-an-id"));
}
