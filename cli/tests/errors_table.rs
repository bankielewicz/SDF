//! The `DFA-` table in `src/errors_table.rs` is generated from the table in
//! `specs/01-cli.md`. These tests keep the two in a two-way bijection, so a
//! code added to the spec and not to the binary, or the reverse, fails here.

use devforgeai::errors_table::{exit_for, ERROR_TABLE, NOT_IMPLEMENTED};
use std::collections::BTreeMap;

/// The spec's error and warning table, read from the repo rather than from a
/// copy: `| DFA-Ennn | subcommand | condition | message | exit |`.
fn spec_rows() -> BTreeMap<String, i32> {
    let spec = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("specs")
        .join("01-cli.md");
    let text = std::fs::read_to_string(&spec)
        .unwrap_or_else(|e| panic!("reading {}: {e}", spec.display()));

    let mut rows = BTreeMap::new();
    for line in text.lines() {
        // Only table rows, never the many prose mentions of a code.
        if !line.starts_with("| DFA-") {
            continue;
        }
        let protected = line.replace(r"\|", "\u{0}");
        let cells: Vec<&str> = protected
            .trim()
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        assert_eq!(
            cells.len(),
            5,
            "five columns in the error table row: {line}"
        );
        let code = cells[0].to_string();
        let exit: i32 = cells[4]
            .parse()
            .unwrap_or_else(|e| panic!("exit column of {code} is an integer: {e}"));
        assert!(
            rows.insert(code.clone(), exit).is_none(),
            "{code} appears twice in the spec table"
        );
    }
    assert!(rows.len() > 100, "the spec table was found and parsed");
    rows
}

#[test]
fn every_spec_code_appears_once() {
    let spec = spec_rows();

    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
    for row in ERROR_TABLE {
        *seen.entry(row.code).or_insert(0) += 1;
    }

    for code in spec.keys() {
        let count = seen.get(code.as_str()).copied().unwrap_or(0);
        assert_eq!(
            count, 1,
            "{code} is in the spec table and appears {count} times in errors_table.rs"
        );
    }
}

#[test]
fn the_module_carries_no_code_outside_the_spec_table() {
    let spec = spec_rows();
    for row in ERROR_TABLE {
        assert!(
            spec.contains_key(row.code),
            "{} is in errors_table.rs and not in the spec table",
            row.code
        );
    }
}

#[test]
fn the_stub_code_is_a_spec_row_in_the_exit_five_band() {
    let spec = spec_rows();
    assert!(
        spec.contains_key(NOT_IMPLEMENTED),
        "the not-implemented code is a row of the spec table"
    );
    assert_eq!(
        exit_for(NOT_IMPLEMENTED),
        Some(5),
        "the stub is the spec's internal-error class"
    );
    // It does not collide with the trust band, which is exit 4.
    assert!(
        NOT_IMPLEMENTED.starts_with("DFA-E9"),
        "the stub sits in the exit-5 band, not the E5xx trust band"
    );
}

#[test]
fn exits_match_the_spec_table_row_for_row() {
    let spec = spec_rows();
    for (code, exit) in spec {
        assert_eq!(
            exit_for(&code),
            Some(exit),
            "{code} maps to the exit the spec table gives it"
        );
    }
}

#[test]
fn warning_codes_all_exit_zero() {
    for row in ERROR_TABLE {
        if row.code.starts_with("DFA-W") {
            assert_eq!(row.exit, 0, "{} is a warning and exits 0", row.code);
        }
    }
}

/// Every `DFA-` literal that appears in `src/`, with the file it was found in.
fn source_code_literals() -> BTreeMap<String, String> {
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut found: BTreeMap<String, String> = BTreeMap::new();
    let mut stack = vec![src];
    while let Some(dir) = stack.pop() {
        let entries =
            std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()));
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().map(|e| e != "rs").unwrap_or(true) {
                continue;
            }
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
            let bytes: Vec<char> = text.chars().collect();
            for (i, _) in text.match_indices("DFA-") {
                // A code is `DFA-` then E or W then three digits. Byte offsets
                // are safe here: the needle is ASCII and so is the code.
                let start = text[..i].chars().count();
                let tail: String = bytes.iter().skip(start).take(8).collect();
                if tail.chars().count() < 8 {
                    continue;
                }
                let kind = tail.as_bytes()[4];
                let digits = &tail[5..8];
                if (kind == b'E' || kind == b'W') && digits.chars().all(|c| c.is_ascii_digit()) {
                    found
                        .entry(tail)
                        .or_insert_with(|| path.display().to_string());
                }
            }
        }
    }
    found
}

/// The reverse direction the bijection tests do not cover: a code emitted from
/// source that no table row defines falls through `exit_for` to `None` and then
/// to the internal-error exit, so a data defect reads as a crash.
#[test]
fn every_code_literal_in_src_is_a_table_row() {
    let table: std::collections::BTreeSet<&str> = ERROR_TABLE.iter().map(|r| r.code).collect();
    let literals = source_code_literals();
    assert!(
        literals.len() > 100,
        "the scan found the source codes: {}",
        literals.len()
    );
    let strays: Vec<String> = literals
        .iter()
        .filter(|(code, _)| !table.contains(code.as_str()))
        .map(|(code, file)| format!("{code} in {file}"))
        .collect();
    assert!(
        strays.is_empty(),
        "every code a source file emits is a table row; these are not: {strays:?}"
    );
}
