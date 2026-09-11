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
