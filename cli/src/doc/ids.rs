//! The ID index: definitions, references, and allocation.
//!
//! An ID is *defined* by any of: the frontmatter `id` of a document; an ID at
//! the start of a Markdown heading line; a Markdown list item whose text begins
//! `<PREFIX>-<nnn>:` at the start of the line; a mapping key `id:` inside a YAML
//! sequence item. Every other occurrence of `\b[A-Z]+-[0-9]{3}\b` is a
//! *reference*.

use crate::errors::CliError;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The reservation directory under `.devforgeai/`. It matches no doc-type row,
/// so it is scratch the producer check skips with no diagnostic.
pub const RESERVED_DIR: &str = ".allocated";

/// The closed list of allocatable prefixes.
pub const ALLOCATABLE: &[&str] = &[
    "IDEA", "FLOW", "PERSONA", "REQ", "EPIC", "CON", "AP", "ADR", "STORY", "AC", "SPRINT", "UI",
    "TOKEN", "FIND", "OBS", "REC",
];

/// Where an ID was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Site {
    /// The file, project-relative in forward-slash form.
    pub path: String,
    /// The one-based line.
    pub line: u32,
    /// The `schema` of the document the site sits in; ID uniqueness is scoped
    /// per schema value.
    pub schema: String,
}

/// Definitions and references across `.devforgeai/`.
#[derive(Debug, Clone, Default)]
pub struct IdIndex {
    /// Every definition site, keyed by ID.
    pub definitions: BTreeMap<String, Vec<Site>>,
    /// Every reference site, keyed by ID.
    pub references: BTreeMap<String, Vec<Site>>,
}

impl IdIndex {
    /// True when some document defines `id`.
    pub fn defines(&self, id: &str) -> bool {
        self.definitions.contains_key(id)
    }

    /// Every ID whose prefix is `prefix`.
    pub fn with_prefix(&self, prefix: &str) -> Vec<&String> {
        self.definitions
            .keys()
            .filter(|k| split_id(k).map(|(p, _)| p == prefix).unwrap_or(false))
            .collect()
    }

    /// The next free ID for a prefix: the highest numeric suffix plus one,
    /// zero-padded to three digits. An unused prefix returns `<prefix>-001`.
    pub fn allocate(&self, prefix: &str) -> Result<String, CliError> {
        if !ALLOCATABLE.contains(&prefix) {
            return Err(CliError::new(
                "DFA-E214",
                format!(
                    "'{prefix}' is not an ID prefix; conventions section 5 defines {}",
                    ALLOCATABLE.join(", ")
                ),
            ));
        }
        let highest = self
            .definitions
            .keys()
            .filter_map(|k| split_id(k))
            .filter(|(p, _)| *p == prefix)
            .map(|(_, n)| n)
            .max()
            .unwrap_or(0);
        if highest >= 999 {
            return Err(CliError::new(
                "DFA-E215",
                format!("prefix {prefix} has no free ID below 999"),
            ));
        }
        Ok(format!("{prefix}-{:03}", highest + 1))
    }

    /// The next free ID for a prefix, reserved on disk before it is returned.
    ///
    /// `allocate` alone reads the index and writes nothing, so two worktrees
    /// sharing one `.devforgeai/` hand out the same number and the collision
    /// surfaces only when the branches merge. The reservation is a marker file
    /// created with `create_new`, which is atomic: the loser of the race sees
    /// `AlreadyExists` and takes the next number. The high-water mark is the
    /// union of the document definitions and the reservations, so an ID that is
    /// reserved but not yet written is never handed out twice.
    pub fn allocate_reserved(
        &self,
        root: &Path,
        prefix: &str,
    ) -> Result<(String, String), CliError> {
        // Validate the prefix and the 999 ceiling through the read-only form.
        let first = self.allocate(prefix)?;
        let dir = crate::project::dot(root).join(RESERVED_DIR);
        std::fs::create_dir_all(&dir).map_err(|e| CliError::io("writing", dir.display(), &e))?;

        let start = split_id(&first).map(|(_, n)| n).unwrap_or(1);
        let reserved_high = reserved_high_water(&dir, prefix);
        let mut n = start.max(reserved_high + 1);
        while n <= 999 {
            let id = format!("{prefix}-{n:03}");
            let marker = dir.join(&id);
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&marker)
            {
                Ok(_) => {
                    let rel = format!(".devforgeai/{RESERVED_DIR}/{id}");
                    return Ok((id, rel));
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    n += 1;
                }
                Err(e) => return Err(CliError::io("writing", marker.display(), &e)),
            }
        }
        Err(CliError::new(
            "DFA-E215",
            format!("prefix {prefix} has no free ID below 999"),
        ))
    }

    /// The number of files the index walked.
    pub fn scanned(&self) -> usize {
        let mut files: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
        for sites in self.definitions.values().chain(self.references.values()) {
            for s in sites {
                files.insert(&s.path);
            }
        }
        files.len()
    }
}

/// The highest number already reserved for `prefix`, or `0`.
fn reserved_high_water(dir: &Path, prefix: &str) -> u32 {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .filter_map(|e| e.file_name().into_string().ok())
        .filter_map(|name| split_id(&name).map(|(p, n)| (p.to_string(), n)))
        .filter(|(p, _)| p == prefix)
        .map(|(_, n)| n)
        .max()
        .unwrap_or(0)
}

/// Split `PREFIX-nnn` into its prefix and number.
pub fn split_id(id: &str) -> Option<(&str, u32)> {
    let (prefix, num) = id.rsplit_once('-')?;
    if prefix.is_empty() || !prefix.chars().all(|c| c.is_ascii_uppercase()) {
        return None;
    }
    if num.len() != 3 || !num.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    num.parse().ok().map(|n| (prefix, n))
}

/// True when `s` is exactly an ID in the section 5 shape.
pub fn is_id(s: &str) -> bool {
    split_id(s).is_some()
}

/// Every `\b[A-Z]+-[0-9]{3}\b` occurrence in `line`, with its zero-based column.
pub fn scan_ids(line: &str) -> Vec<(usize, String)> {
    let chars: Vec<char> = line.chars().collect();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < chars.len() {
        if !chars[i].is_ascii_uppercase() {
            i += 1;
            continue;
        }
        // A word boundary before the run.
        if i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_') {
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            continue;
        }
        let start = i;
        while i < chars.len() && chars[i].is_ascii_uppercase() {
            i += 1;
        }
        if i >= chars.len() || chars[i] != '-' {
            continue;
        }
        let dash = i;
        i += 1;
        let digits_start = i;
        while i < chars.len() && chars[i].is_ascii_digit() {
            i += 1;
        }
        if i - digits_start != 3 {
            continue;
        }
        // A word boundary after the run.
        if i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
            continue;
        }
        let id: String = chars[start..dash].iter().collect::<String>()
            + "-"
            + &chars[digits_start..i].iter().collect::<String>();
        out.push((start, id));
    }
    out
}

/// Classify every ID occurrence in one document's text.
///
/// `frontmatter_id` is the document's own `id`, which is a definition wherever
/// it appears in the frontmatter block.
pub fn index_text(
    text: &str,
    path: &str,
    schema: &str,
    frontmatter_id: Option<&str>,
    body_line: u32,
    index: &mut IdIndex,
) {
    let lines: Vec<&str> = text.lines().collect();

    for (i, raw) in lines.iter().enumerate() {
        let line_no = i as u32 + 1;
        let in_frontmatter = line_no < body_line;
        let trimmed = raw.trim_start();

        // Which IDs on this line are definitions rather than references.
        let mut defined_here: Vec<String> = Vec::new();

        if in_frontmatter {
            // The frontmatter `id:` value defines the document's own ID.
            if let Some(rest) = trimmed.strip_prefix("id:") {
                let value = rest.trim().trim_matches(['"', '\'']);
                if is_id(value) && frontmatter_id == Some(value) {
                    defined_here.push(value.to_string());
                }
            }
        } else {
            // An ID at the start of a Markdown heading line.
            if let Some(rest) = trimmed.strip_prefix('#') {
                let heading = rest.trim_start_matches('#').trim_start();
                if let Some((0, id)) = scan_ids(heading).first().cloned() {
                    defined_here.push(id);
                }
            }
            // A Markdown list item whose text begins `<PREFIX>-<nnn>:`.
            for bullet in ["- ", "* ", "+ "] {
                if let Some(rest) = trimmed.strip_prefix(bullet) {
                    let head = rest.trim_start();
                    if let Some((id, _)) = head.split_once(':') {
                        if is_id(id) {
                            defined_here.push(id.to_string());
                        }
                    }
                }
            }
            // A mapping key `id:` inside a YAML sequence item.
            let yaml_id = trimmed
                .strip_prefix("- id:")
                .or_else(|| trimmed.strip_prefix("id:"));
            if let Some(rest) = yaml_id {
                let value = rest.trim().trim_matches(['"', '\'']);
                if is_id(value) {
                    defined_here.push(value.to_string());
                }
            }
        }

        for (_, id) in scan_ids(raw) {
            let site = Site {
                path: path.to_string(),
                line: line_no,
                schema: schema.to_string(),
            };
            if defined_here.contains(&id) {
                index.definitions.entry(id).or_default().push(site);
            } else {
                index.references.entry(id).or_default().push(site);
            }
        }
    }
}

/// Walk `.devforgeai/` and build the index.
pub fn build(root: &Path) -> IdIndex {
    let mut index = IdIndex::default();
    let dot = crate::project::dot(root);
    if !dot.is_dir() {
        return index;
    }
    for entry in walkdir::WalkDir::new(&dot)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let p = entry.path();
        let ext = p
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if !matches!(ext.as_str(), "md" | "yaml" | "yml" | "json") {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(p) else {
            continue;
        };
        index_one(root, p, &text, &mut index);
    }
    index
}

/// Index one already-read document.
pub fn index_one(root: &Path, path: &Path, text: &str, index: &mut IdIndex) {
    let rel = crate::project::rel_display(root, path);
    let kind = super::source_for(path);
    let (schema, fm_id, body_line) = match super::frontmatter::parse(text, kind, &rel) {
        Ok(fm) => (
            fm.str_key("schema").unwrap_or("").to_string(),
            fm.str_key("id").map(str::to_string),
            fm.body_line,
        ),
        Err(_) => (String::new(), None, 1),
    };
    index_text(text, &rel, &schema, fm_id.as_deref(), body_line, index);
}

/// Every path under `.devforgeai/` the index would walk.
pub fn document_paths(root: &Path) -> Vec<PathBuf> {
    let dot = crate::project::dot(root);
    let mut out = Vec::new();
    if !dot.is_dir() {
        return out;
    }
    for entry in walkdir::WalkDir::new(&dot)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        if entry.file_type().is_file() {
            out.push(entry.path().to_path_buf());
        }
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index_of(text: &str, schema: &str, fm_id: Option<&str>, body_line: u32) -> IdIndex {
        let mut ix = IdIndex::default();
        index_text(text, "d.md", schema, fm_id, body_line, &mut ix);
        ix
    }

    #[test]
    fn scan_finds_ids_at_word_boundaries() {
        assert_eq!(
            scan_ids("see REQ-001 and EPIC-002."),
            vec![(4, "REQ-001".to_string()), (16, "EPIC-002".to_string())]
        );
        // `XREQ` is itself a run of capitals, so `XREQ-001` is an ID with the
        // prefix `XREQ`, exactly as the pattern reads. A word character that is
        // not a capital before the run is what removes the boundary.
        assert_eq!(scan_ids("XREQ-001"), vec![(0, "XREQ-001".to_string())]);
        assert!(scan_ids("xREQ-001").is_empty(), "no boundary before");
        assert!(scan_ids("1REQ-001").is_empty(), "no boundary before");
        assert!(scan_ids("REQ-0011").is_empty(), "four digits is not an ID");
        assert!(scan_ids("REQ-01").is_empty(), "two digits is not an ID");
        assert!(scan_ids("req-001").is_empty(), "lowercase is not an ID");
        assert!(
            scan_ids("TOKEN-color-primary").is_empty(),
            "a token reference is not an ID-index entry"
        );
    }

    #[test]
    fn index_defines_from_heading() {
        let ix = index_of(
            "---\nid: STORY-014\n---\n\n### AC-003 the empty cart\n",
            "s",
            Some("STORY-014"),
            4,
        );
        assert!(
            ix.defines("AC-003"),
            "an ID at the start of a heading defines it"
        );
        assert!(ix.defines("STORY-014"), "the frontmatter id defines it");
    }

    #[test]
    fn index_defines_from_list_item() {
        let ix = index_of(
            "---\nid: STORY-014\n---\n\n## Acceptance Criteria\n\n- AC-003: Given a cart When empty Then rejects\n",
            "s",
            Some("STORY-014"),
            4,
        );
        assert!(
            ix.defines("AC-003"),
            "the acceptance-criterion list-item form defines the ID"
        );
    }

    #[test]
    fn index_defines_from_yaml_item() {
        let ix = index_of(
            "schema: devforgeai/requirements/1\nid: IDEA-003\nrequirements:\n  - id: REQ-001\n    actor: PERSONA-001\n",
            "devforgeai/requirements/1",
            Some("IDEA-003"),
            1,
        );
        assert!(
            ix.defines("REQ-001"),
            "a mapping key id: in a sequence item"
        );
        assert!(!ix.defines("PERSONA-001"), "an actor value is a reference");
        assert!(ix.references.contains_key("PERSONA-001"));
    }

    #[test]
    fn a_heading_id_not_at_the_start_is_a_reference() {
        let ix = index_of(
            "---\nid: STORY-014\n---\n\n### About AC-003\n",
            "s",
            Some("STORY-014"),
            4,
        );
        assert!(!ix.defines("AC-003"));
        assert!(ix.references.contains_key("AC-003"));
    }

    #[test]
    fn allocate_first_id_is_001() {
        let ix = IdIndex::default();
        assert_eq!(ix.allocate("REQ").expect("allocate"), "REQ-001");
    }

    #[test]
    fn allocate_next_after_gap() {
        let mut ix = IdIndex::default();
        for id in ["REQ-001", "REQ-002", "REQ-013"] {
            ix.definitions.insert(
                id.to_string(),
                vec![Site {
                    path: "r.yaml".into(),
                    line: 1,
                    schema: "s".into(),
                }],
            );
        }
        assert_eq!(
            ix.allocate("REQ").expect("allocate"),
            "REQ-014",
            "the highest suffix plus one, not the first gap"
        );
    }

    #[test]
    fn allocate_unknown_prefix_gives_e214() {
        let ix = IdIndex::default();
        let err = ix.allocate("WIDGET").expect_err("outside the closed list");
        assert_eq!(err.code(), "DFA-E214");
        assert_eq!(err.exit(), 3);
    }

    #[test]
    fn allocate_at_999_gives_e215() {
        let mut ix = IdIndex::default();
        ix.definitions.insert(
            "REQ-999".to_string(),
            vec![Site {
                path: "r.yaml".into(),
                line: 1,
                schema: "s".into(),
            }],
        );
        let err = ix.allocate("REQ").expect_err("exhausted");
        assert_eq!(err.code(), "DFA-E215");
        assert_eq!(err.exit(), 1);
    }

    #[test]
    fn the_allocatable_prefixes_are_the_closed_list() {
        assert_eq!(ALLOCATABLE.len(), 16);
        assert!(
            ALLOCATABLE.contains(&"AP"),
            "Constitute's anti-pattern prefix"
        );
        assert!(!ALLOCATABLE.contains(&"FIXME"));
    }

    #[test]
    fn split_id_rejects_malformed_shapes() {
        assert_eq!(split_id("REQ-014"), Some(("REQ", 14)));
        assert_eq!(split_id("REQ-14"), None);
        assert_eq!(split_id("req-014"), None);
        assert_eq!(split_id("014"), None);
    }
}
