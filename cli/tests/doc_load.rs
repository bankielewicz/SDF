//! `doc load`: the selector per document name, `discover-entry`, and the
//! byte-for-byte output.

mod common;

use common::Project;
use devforgeai::cmd::doc;

fn raw(out: &devforgeai::Outcome) -> String {
    String::from_utf8(out.raw.clone().expect("raw bytes")).expect("UTF-8")
}

#[test]
fn load_story_prints_bytes() {
    let p = Project::new();
    let body = common::story("STORY-014", "ready", &[], "# Order checkout\n");
    p.write(".devforgeai/stories/STORY-014.md", &body);

    let mut ctx = p.ctx();
    let out = doc::load(&mut ctx, "story", "STORY-014").expect("load");
    assert_eq!(raw(&out), body, "the file goes to stdout byte for byte");
    assert_eq!(out.data["name"], "story");
    assert_eq!(out.data["bytes"], body.len());
}

#[test]
fn load_json_carries_content() {
    let p = Project::new();
    let body = common::requirements("IDEA-003", "accepted");
    p.write(".devforgeai/requirements.yaml", &body);

    let mut ctx = p.ctx();
    let out = doc::load(&mut ctx, "requirements", "-").expect("load");
    assert_eq!(out.data["content"], body);
    assert_eq!(out.data["path"], ".devforgeai/requirements.yaml");
}

#[test]
fn load_context_all_concatenates_six() {
    let p = Project::new();
    for stem in devforgeai::doc::CONTEXT_STEMS {
        p.write(
            &format!(".devforgeai/context/{stem}.md"),
            &format!("---\nschema: devforgeai/context-{stem}/1\nid: {stem}\nphase: constitute\nstatus: accepted\nproduced_by: establishing-context\nconsumes: []\nopen_questions: []\n---\n\n# {stem}\n"),
        );
    }

    let mut ctx = p.ctx();
    let out = doc::load(&mut ctx, "context", "all").expect("load");
    let text = raw(&out);

    // The six, in the conventions section 3 order, separated by a line of
    // three dashes.
    let mut cursor = 0usize;
    for stem in devforgeai::doc::CONTEXT_STEMS {
        let needle = format!("# {stem}\n");
        let at = text[cursor..]
            .find(&needle)
            .unwrap_or_else(|| panic!("{stem} appears after the ones before it"));
        cursor += at + needle.len();
    }
    // Each file carries its own two frontmatter fences, so six files bring
    // twelve; the five separators the concatenation inserts take the count of
    // lines that are exactly three dashes to seventeen.
    let fences = text.lines().filter(|l| l.trim_end() == "---").count();
    assert_eq!(fences, 17, "twelve frontmatter fences plus five separators");
}

#[test]
fn load_context_one_file() {
    let p = Project::new();
    p.write(
        ".devforgeai/context/tech-stack.md",
        "---\nschema: devforgeai/context-tech-stack/1\nid: tech-stack\nphase: constitute\nstatus: accepted\nproduced_by: establishing-context\nconsumes: []\nopen_questions: []\n---\n\n# tech-stack\n",
    );
    let mut ctx = p.ctx();
    let out = doc::load(&mut ctx, "context", "tech-stack").expect("load");
    assert!(raw(&out).contains("# tech-stack"));
}

#[test]
fn load_adr_all_orders_by_number() {
    let p = Project::new();
    for n in ["012", "003", "007"] {
        p.write(
            &format!(".devforgeai/adr/ADR-{n}.md"),
            &format!("---\nschema: devforgeai/adr/1\nid: ADR-{n}\nphase: constitute\nstatus: accepted\nproduced_by: establishing-context\nconsumes: []\nopen_questions: []\n---\n\n# Decision {n}\n"),
        );
    }
    let mut ctx = p.ctx();
    let out = doc::load(&mut ctx, "adr", "all").expect("load");
    let text = raw(&out);

    let a = text.find("Decision 003").expect("003");
    let b = text.find("Decision 007").expect("007");
    let c = text.find("Decision 012").expect("012");
    assert!(a < b && b < c, "ordered by number, not lexically");
}

#[test]
fn load_reflect_latest_picks_newest() {
    let p = Project::new();
    for date in ["2026-08-01", "2026-09-10", "2026-07-15"] {
        p.write(
            &format!(".devforgeai/reports/reflect-{date}.yaml"),
            &format!("schema: devforgeai/reflect-report/1\nid: {date}\nphase: reflect\nstatus: final\nproduced_by: improving-framework\nconsumes: []\nopen_questions: []\nmarker: {date}\n"),
        );
    }
    let mut ctx = p.ctx();
    let out = doc::load(&mut ctx, "reflect-report", "latest").expect("load");
    assert!(
        raw(&out).contains("marker: 2026-09-10"),
        "the newest date wins"
    );
}

#[test]
fn load_missing_gives_e200() {
    let p = Project::new();
    let mut ctx = p.ctx();
    let err = doc::load(&mut ctx, "story", "STORY-099").expect_err("absent");
    assert_eq!(err.code(), "DFA-E200");
    assert_eq!(err.exit(), 1);
}

#[test]
fn load_unknown_name_gives_e250() {
    let p = Project::new();
    let mut ctx = p.ctx();
    let err = doc::load(&mut ctx, "manifesto", "-").expect_err("not a document name");
    assert_eq!(err.code(), "DFA-E250");
    assert_eq!(err.exit(), 3);
    assert!(err.diag().expect("diag").message.contains("explore-brief"));
}

#[test]
fn load_discover_entry_prints_brief_and_decision() {
    let p = Project::new();
    p.write(
        ".devforgeai/explore/brief.md",
        &common::brief("IDEA-003", "decided"),
    );
    p.write(
        ".devforgeai/explore/decision.yaml",
        &common::decision("IDEA-003", "promote"),
    );

    let mut ctx = p.ctx();
    let out = doc::load(&mut ctx, "discover-entry", "IDEA-003").expect("load");
    let text = raw(&out);
    assert!(text.contains("# Order checkout"), "the brief");
    assert!(text.contains("decision: promote"), "the decision");
}

#[test]
fn load_discover_entry_prints_requirements_without_brief() {
    let p = Project::new();
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-003", "drafting"),
    );

    let mut ctx = p.ctx();
    let out = doc::load(&mut ctx, "discover-entry", "IDEA-003").expect("load");
    let text = raw(&out);
    assert!(text.contains("REQ-001"), "the requirements print");
    assert!(!text.contains("Core flows"), "no brief exists");
}

#[test]
fn load_discover_entry_prints_both_when_both_exist() {
    let p = Project::new();
    p.write(
        ".devforgeai/explore/brief.md",
        &common::brief("IDEA-003", "decided"),
    );
    p.write(
        ".devforgeai/explore/decision.yaml",
        &common::decision("IDEA-003", "promote"),
    );
    p.write(
        ".devforgeai/requirements.yaml",
        &common::requirements("IDEA-003", "drafting"),
    );

    let mut ctx = p.ctx();
    let out = doc::load(&mut ctx, "discover-entry", "IDEA-003").expect("load");
    let text = raw(&out);
    assert!(text.contains("Core flows"));
    assert!(text.contains("REQ-001"));
}

#[test]
fn load_discover_entry_unknown_arg_exits_zero_silent() {
    let p = Project::new();
    p.write(
        ".devforgeai/explore/brief.md",
        &common::brief("IDEA-003", "decided"),
    );

    let mut ctx = p.ctx();

    // Any argument that is not an IDEA prints nothing and exits 0.
    let out = doc::load(&mut ctx, "discover-entry", "a checkout idea").expect("exits 0");
    assert_eq!(raw(&out), "", "nothing on stdout");
    assert_eq!(out.exit, None);

    // An IDEA matching no document prints nothing and exits 0.
    let out = doc::load(&mut ctx, "discover-entry", "IDEA-099").expect("exits 0");
    assert_eq!(raw(&out), "");
}

#[test]
fn load_discover_entry_in_a_bare_project_exits_zero() {
    let p = Project::new();
    let mut ctx = p.ctx();
    let out = doc::load(&mut ctx, "discover-entry", "IDEA-001")
        .expect("the preamble runs before the entry point is known");
    assert_eq!(raw(&out), "");
}
