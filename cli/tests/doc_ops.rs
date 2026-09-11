//! `doc accept requirements` and `doc reopen requirements`. Both edit the
//! document in place and leave every other byte untouched, which is what the
//! `leaves_other_bytes` tests hold.

mod common;

use common::Project;
use devforgeai::cli::{DocAcceptArgs, DocReopenArgs};
use devforgeai::cmd::doc;

const REQUIREMENTS: &str = "schema: devforgeai/requirements/1\n\
id: IDEA-004\n\
phase: discover\n\
status: drafting\n\
produced_by: discovering-requirements\n\
consumes: []\n\
open_questions: []\n\
revision: 1\n\
accepted_by: null\n\
accepted_at: null\n\
# a human comment the CLI must not disturb\n\
personas:\n\
\x20 - id: PERSONA-001\n\
\x20   name: Shopper\n\
requirements:\n\
\x20 - id: REQ-001\n\
\x20   actor: PERSONA-001\n\
\x20   statement: The shopper places an order\n\
\x20   status: draft\n\
\x20 - id: REQ-002\n\
\x20   actor: PERSONA-001\n\
\x20   statement: The shopper sees a receipt\n\
\x20   status: reopened\n\
\x20 - id: REQ-003\n\
\x20   actor: PERSONA-001\n\
\x20   statement: The shopper cancels\n\
\x20   status: withdrawn\n\
epics:\n\
\x20 - id: EPIC-001\n\
\x20   title: Checkout\n\
\x20   requirements: [REQ-001, REQ-002]\n";

fn accept_args() -> DocAcceptArgs {
    DocAcceptArgs {
        name: "requirements".into(),
        id: "IDEA-004".into(),
    }
}

fn reopen_args(ids: &str, from: &str) -> DocReopenArgs {
    DocReopenArgs {
        name: "requirements".into(),
        id: "IDEA-004".into(),
        ids: ids.into(),
        from: from.into(),
    }
}

fn seeded() -> Project {
    let p = Project::new();
    p.write(".devforgeai/requirements.yaml", REQUIREMENTS);
    p
}

#[test]
fn accept_sets_accepted_by_and_status() {
    let p = seeded();
    let mut ctx = p.ctx();
    let out = common::with_now("2026-09-10T14:22:05Z", || {
        doc::accept(&mut ctx, &accept_args()).expect("accept")
    });

    let text = p.read(".devforgeai/requirements.yaml");
    assert!(text.contains("status: accepted"), "the document status");
    assert!(text.contains("accepted_by: user"));
    assert!(text.contains("accepted_at: 2026-09-10T14:22:05Z"));
    assert_eq!(out.data["status"], "accepted");
    assert_eq!(out.data["accepted_at"], "2026-09-10T14:22:05Z");
}

#[test]
fn accept_moves_draft_and_reopened_requirements() {
    let p = seeded();
    let mut ctx = p.ctx();
    let out = doc::accept(&mut ctx, &accept_args()).expect("accept");

    let text = p.read(".devforgeai/requirements.yaml");
    assert_eq!(
        out.data["requirements_moved"], 2,
        "REQ-001 at draft and REQ-002 at reopened move; REQ-003 at withdrawn does not"
    );
    assert!(
        text.contains("status: withdrawn"),
        "a withdrawn requirement is left alone"
    );
    assert_eq!(
        text.matches("status: accepted").count(),
        3,
        "two requirements plus the document"
    );
}

#[test]
fn accept_leaves_other_bytes() {
    let p = seeded();
    let mut ctx = p.ctx();
    doc::accept(&mut ctx, &accept_args()).expect("accept");
    let after = p.read(".devforgeai/requirements.yaml");

    assert!(
        after.contains("# a human comment the CLI must not disturb"),
        "comments survive; the edit is line surgery, not a YAML round trip"
    );
    assert!(after.contains("statement: The shopper places an order"));
    assert!(after.contains("  - id: PERSONA-001"));
    assert!(after.contains("requirements: [REQ-001, REQ-002]"));

    // Only the lines the spec names changed.
    let before_lines: Vec<&str> = REQUIREMENTS.lines().collect();
    let after_lines: Vec<&str> = after.lines().collect();
    assert_eq!(
        before_lines.len(),
        after_lines.len(),
        "no line added or removed"
    );
    let changed: Vec<usize> = before_lines
        .iter()
        .zip(&after_lines)
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(
        changed.len(),
        5,
        "status, accepted_by, accepted_at, two requirement statuses"
    );
}

#[test]
fn accept_empty_epics_gives_e260() {
    let p = Project::new();
    let text = REQUIREMENTS
        .split("epics:")
        .next()
        .expect("split")
        .to_string()
        + "epics: []\n";
    p.write(".devforgeai/requirements.yaml", &text);

    let mut ctx = p.ctx();
    let err = doc::accept(&mut ctx, &accept_args()).expect_err("no epics");
    assert_eq!(err.code(), "DFA-E260");
    assert_eq!(err.exit(), 1);
    assert!(err.diag().expect("diag").message.contains("epics"));
}

#[test]
fn accept_missing_file_gives_e200() {
    let p = Project::new();
    let mut ctx = p.ctx();
    let err = doc::accept(&mut ctx, &accept_args()).expect_err("absent");
    assert_eq!(err.code(), "DFA-E200");
}

#[test]
fn accept_of_another_document_gives_e250() {
    let p = seeded();
    let mut ctx = p.ctx();
    let args = DocAcceptArgs {
        name: "story".into(),
        id: "IDEA-004".into(),
    };
    let err = doc::accept(&mut ctx, &args).expect_err("requirements is the only name");
    assert_eq!(err.code(), "DFA-E250");
}

#[test]
fn reopen_raises_revision() {
    let p = seeded();
    let mut ctx = p.ctx();
    let out = doc::reopen(&mut ctx, &reopen_args("REQ-001", "plan")).expect("reopen");

    assert_eq!(out.data["revision"], 2);
    assert!(p
        .read(".devforgeai/requirements.yaml")
        .contains("revision: 2"));
}

#[test]
fn reopen_appends_revision_log() {
    let p = seeded();
    let mut ctx = p.ctx();
    common::with_now("2026-09-11T09:00:00Z", || {
        doc::reopen(&mut ctx, &reopen_args("REQ-001", "design")).expect("reopen")
    });

    let text = p.read(".devforgeai/requirements.yaml");
    assert!(text.contains("revision_log:"), "the log key");
    assert!(text.contains("- revision: 2"));
    assert!(text.contains("from: design"));
    assert!(text.contains("at: 2026-09-11T09:00:00Z"));
    assert!(text.contains("ids: [REQ-001]"));
}

#[test]
fn reopen_moves_cited_requirements_in_place() {
    let p = seeded();
    let mut ctx = p.ctx();
    let out = doc::reopen(&mut ctx, &reopen_args("REQ-001", "plan")).expect("reopen");

    assert_eq!(out.data["reopened"], serde_json::json!(["REQ-001"]));
    let text = p.read(".devforgeai/requirements.yaml");
    // REQ-001 moved; REQ-003 is untouched.
    let req1 = text
        .split("- id: REQ-001")
        .nth(1)
        .expect("REQ-001 block")
        .to_string();
    assert!(
        req1.split("- id:")
            .next()
            .expect("block")
            .contains("status: reopened"),
        "REQ-001 is reopened in place"
    );
    assert!(text.contains("status: withdrawn"), "REQ-003 untouched");
}

#[test]
fn reopen_allocates_req_per_ui() {
    let p = seeded();
    p.write(
        ".devforgeai/ui-specs/UI-002.md",
        "---\nschema: devforgeai/ui-spec/1\nid: UI-002\nphase: design\nstatus: draft\nproduced_by: designing-interfaces\nconsumes: []\nopen_questions: []\n---\n\n# Checkout\n",
    );

    let mut ctx = p.ctx();
    let out = doc::reopen(&mut ctx, &reopen_args("UI-002", "design")).expect("reopen");

    let allocated = out.data["allocated"].as_array().expect("allocated");
    assert_eq!(allocated.len(), 1, "one REQ per cited UI");
    assert_eq!(allocated[0], "REQ-004", "the next free REQ");

    let text = p.read(".devforgeai/requirements.yaml");
    assert!(text.contains("- id: REQ-004"));
    assert!(text.contains("source: user"));
    assert!(text.contains("traces_to: [UI-002]"));
    assert!(
        !text.contains("REQ-004\n    epic:"),
        "the allocated requirement carries no epic"
    );
}

#[test]
fn reopen_nulls_acceptance() {
    let p = seeded();
    let mut ctx = p.ctx();
    // Accept first, so there is an acceptance to null.
    doc::accept(&mut ctx, &accept_args()).expect("accept");
    assert!(p
        .read(".devforgeai/requirements.yaml")
        .contains("accepted_by: user"));

    let mut ctx = p.ctx();
    doc::reopen(&mut ctx, &reopen_args("REQ-001", "constitute")).expect("reopen");

    let text = p.read(".devforgeai/requirements.yaml");
    assert!(text.contains("accepted_by: null"));
    assert!(text.contains("accepted_at: null"));
    assert!(text.contains("status: reopened"), "the document status");
}

#[test]
fn reopen_bad_id_gives_e261() {
    let p = seeded();
    let mut ctx = p.ctx();
    let err = doc::reopen(&mut ctx, &reopen_args("STORY-001", "plan"))
        .expect_err("neither a REQ nor a UI id");
    assert_eq!(err.code(), "DFA-E261");
    assert_eq!(err.exit(), 3);
    assert!(err.diag().expect("diag").message.contains("STORY-001"));
}

#[test]
fn reopen_unknown_req_gives_e210() {
    let p = seeded();
    let mut ctx = p.ctx();
    let err = doc::reopen(&mut ctx, &reopen_args("REQ-099", "plan"))
        .expect_err("the document does not define it");
    assert_eq!(err.code(), "DFA-E210");
    assert_eq!(err.exit(), 1);
}

#[test]
fn reopen_bad_from_gives_e012() {
    let p = seeded();
    let mut ctx = p.ctx();
    let err =
        doc::reopen(&mut ctx, &reopen_args("REQ-001", "build")).expect_err("outside the enum");
    assert_eq!(err.code(), "DFA-E012");
    assert_eq!(err.exit(), 3);
}

#[test]
fn reopen_leaves_other_bytes() {
    let p = seeded();
    let mut ctx = p.ctx();
    doc::reopen(&mut ctx, &reopen_args("REQ-001", "plan")).expect("reopen");
    let after = p.read(".devforgeai/requirements.yaml");

    assert!(
        after.contains("# a human comment the CLI must not disturb"),
        "comments survive"
    );
    assert!(after.contains("statement: The shopper places an order"));
    assert!(after.contains("  - id: PERSONA-001"));
    assert!(after.contains("requirements: [REQ-001, REQ-002]"));
    assert!(
        after.contains("statement: The shopper sees a receipt"),
        "an uncited requirement keeps every byte"
    );
}

// --- `--allocate` reserves the id it hands out ----------------------------

/// `doc validate --allocate <PREFIX>` against a project.
fn allocate(p: &Project, prefix: &str) -> devforgeai::Outcome {
    let mut ctx = p.ctx();
    let a = devforgeai::cli::DocValidateArgs {
        paths: Vec::new(),
        all: false,
        allocate: Some(prefix.to_string()),
        producer_check: false,
        stdin_content: false,
    };
    doc::validate(&mut ctx, &a, None).expect("allocate")
}

#[test]
fn allocate_twice_gives_two_ids() {
    let p = Project::new();
    p.write(
        ".devforgeai/stories/STORY-007.md",
        &common::story("STORY-007", "draft", &[], "- AC-007: Given a cart\n"),
    );

    // Two worktrees sharing one `.devforgeai/` both read the same high-water
    // mark. Without a reservation both are handed AC-008 and the collision
    // surfaces only when the branches merge.
    let first = allocate(&p, "AC");
    let second = allocate(&p, "AC");

    assert_eq!(first.data["id"], "AC-008");
    assert_eq!(second.data["id"], "AC-009", "the first id was reserved");
    assert_eq!(first.human, vec!["AC-008".to_string()]);
}

#[test]
fn allocate_records_the_reservation_on_disk() {
    let p = Project::new();
    let out = allocate(&p, "REQ");

    let rel = out.data["reservation"].as_str().expect("reservation path");
    assert_eq!(rel, ".devforgeai/.allocated/REQ-001");
    assert!(p.exists(rel), "the marker is what makes the id reserved");
}

#[test]
fn allocate_skips_a_reserved_id() {
    let p = Project::new();
    std::fs::create_dir_all(p.root().join(".devforgeai").join(".allocated")).expect("mkdir");
    std::fs::write(
        p.root()
            .join(".devforgeai")
            .join(".allocated")
            .join("AC-001"),
        "",
    )
    .expect("write marker");

    let out = allocate(&p, "AC");
    assert_eq!(
        out.data["id"], "AC-002",
        "a reserved-but-unwritten id is never handed out twice"
    );
}

#[test]
fn allocate_refuses_a_path_beside_it() {
    let p = Project::new();
    let mut ctx = p.ctx();
    let a = devforgeai::cli::DocValidateArgs {
        paths: Vec::new(),
        all: true,
        allocate: Some("AC".to_string()),
        producer_check: false,
        stdin_content: false,
    };
    let err = doc::validate(&mut ctx, &a, None).expect_err("--allocate takes no --all");
    assert_eq!(err.code(), "DFA-E010");
}
