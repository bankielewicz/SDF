//! The `[[verifier]]` registry `init` writes, compared against the spec table
//! at test time rather than against a second transcription.

mod common;

use common::Project;

/// The spec's registry table, read from the repo: the five columns of
/// `specs/01-cli.md` `## Outputs` -> the twenty `[[verifier]]` tables.
fn spec_rows() -> Vec<(String, String, String, String, bool)> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("specs")
        .join("01-cli.md");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));

    let start = text
        .find("`init` writes twenty `[[verifier]]` tables")
        .expect("the registry paragraph");
    let end = text[start..]
        .find("Twenty registrations across")
        .map(|o| start + o)
        .expect("the paragraph after the table");

    let mut rows = Vec::new();
    for line in text[start..end].lines() {
        if !line.starts_with("| ") || line.starts_with("| Phase") || line.starts_with("|---") {
            continue;
        }
        let cells: Vec<String> = line
            .trim()
            .trim_matches('|')
            .split('|')
            .map(|c| c.trim().trim_matches('`').to_string())
            .collect();
        assert_eq!(cells.len(), 5, "five columns in {line}");
        rows.push((
            cells[0].clone(),
            cells[1].clone(),
            cells[2].clone(),
            cells[3].clone(),
            cells[4] == "true",
        ));
    }
    assert_eq!(rows.len(), 20, "twenty registrations");
    rows
}

#[test]
fn the_compiled_registry_is_the_spec_table_row_for_row() {
    let want = spec_rows();
    let have = devforgeai::config::default_verifiers();
    assert_eq!(have.len(), want.len());
    for (i, (phase, name, field, unit, required)) in want.iter().enumerate() {
        let v = &have[i];
        assert_eq!(&v.phase, phase, "row {i} phase");
        assert_eq!(&v.name, name, "row {i} name");
        assert_eq!(&v.report_field, field, "row {i} report_field");
        assert_eq!(&v.unit, unit, "row {i} unit");
        assert_eq!(v.required, *required, "row {i} required");
    }
}

#[test]
fn the_config_init_writes_carries_the_registry_in_that_order() {
    // `Project::new()` writes `config.toml` through the same serialiser `init`
    // uses, so the round trip is the one the file takes on disk.
    let p = Project::new();
    let cfg = devforgeai::config::load(p.root()).expect("config");
    let want = spec_rows();
    assert_eq!(cfg.verifier.len(), 20);
    for (i, (phase, name, field, unit, required)) in want.iter().enumerate() {
        let v = &cfg.verifier[i];
        assert_eq!(&v.phase, phase, "row {i}");
        assert_eq!(&v.name, name, "row {i}");
        assert_eq!(&v.report_field, field, "row {i}");
        assert_eq!(&v.unit, unit, "row {i}");
        assert_eq!(v.required, *required, "row {i}");
    }
}

#[test]
fn the_written_toml_holds_one_table_per_registration() {
    let p = Project::new();
    let text = p.read(".devforgeai/config.toml");
    assert_eq!(
        text.matches("[[verifier]]").count(),
        20,
        "twenty tables in the file"
    );
    for (_, name, _, _, _) in spec_rows() {
        assert!(
            text.contains(&format!("name = \"{name}\"")),
            "{name} is written"
        );
    }
}

#[test]
fn every_required_verifier_is_resolvable_by_name() {
    let cfg = devforgeai::config::Config::default();
    for v in cfg.verifier.iter().filter(|v| v.required) {
        assert!(
            cfg.verifier_by_name(&v.name).is_some(),
            "{} resolves for a verifier_pass check",
            v.name
        );
    }
}
