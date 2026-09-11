//! The story document as `story validate`, `story list`, and `story files`
//! read it: the frontmatter, the acceptance criteria, the `## Requirements`
//! and `## Files` tables, and the dependency list.

use crate::errors::CliError;
use crate::md;
use std::path::{Path, PathBuf};

/// The verbs an acceptance criterion may use instead of Given/When/Then.
pub const TESTABLE_VERBS: &[&str] = &[
    "returns", "rejects", "renders", "persists", "emits", "exits", "responds",
];

/// The separator the AC grammar fixes: a colon and a space.
const AC_SEPARATOR: &str = ": ";

/// The `Kind` enum of a `## Files` row.
pub const FILE_KINDS: &[&str] = &["source", "test", "config", "migration", "asset"];

/// One acceptance criterion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ac {
    /// `AC-nnn`.
    pub id: String,
    /// The text after the colon.
    pub text: String,
    /// The one-based line.
    pub line: u32,
}

/// One row of `## Requirements`.
#[derive(Debug, Clone)]
pub struct ReqRow {
    /// `REQ-nnn`.
    pub id: String,
    /// The `Covered by` ids.
    pub covered_by: Vec<String>,
    /// The one-based line.
    pub line: u32,
}

/// One row of `## Files`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileRow {
    /// Repo-relative, forward slashes.
    pub path: String,
    /// One of `FILE_KINDS`.
    pub kind: String,
    /// The layer name.
    pub layer: String,
    /// The one-based line.
    pub line: u32,
}

/// One story document.
#[derive(Debug, Clone)]
pub struct Story {
    /// `STORY-nnn`.
    pub id: String,
    /// The project-relative path.
    pub rel: String,
    /// The frontmatter `status`.
    pub status: String,
    /// The frontmatter `consumes`.
    pub consumes: Vec<String>,
    /// The first H1, with the `# ` removed.
    pub title: String,
    /// The acceptance criteria, in file order.
    pub acs: Vec<Ac>,
    /// The `## Requirements` rows.
    pub requirements: Vec<ReqRow>,
    /// The `## Files` rows.
    pub files: Vec<FileRow>,
    /// The `STORY-nnn` ids under `## Dependencies`.
    pub deps: Vec<String>,
    /// The whole file.
    pub text: String,
}

/// The path of a story id.
pub fn path_of(root: &Path, id: &str) -> PathBuf {
    root.join(".devforgeai")
        .join("stories")
        .join(format!("{id}.md"))
}

/// Every `.devforgeai/stories/STORY-nnn.md`, sorted by id.
pub fn all_paths(root: &Path) -> Vec<PathBuf> {
    let dir = root.join(".devforgeai").join("stories");
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| {
                    n.starts_with("STORY-")
                        && n.ends_with(".md")
                        && crate::doc::ids::is_id(n.trim_end_matches(".md"))
                })
                .unwrap_or(false)
        })
        .collect();
    out.sort();
    out
}

/// Read one story by id, `DFA-E200` when the file is absent.
pub fn load(root: &Path, id: &str) -> Result<Story, CliError> {
    let path = path_of(root, id);
    if !path.exists() {
        let rel = crate::project::rel_display(root, &path);
        return Err(CliError::at("DFA-E200", format!("{rel} not found"), rel));
    }
    read(root, &path)
}

/// Read one story from a path.
pub fn read(root: &Path, path: &Path) -> Result<Story, CliError> {
    let rel = crate::project::rel_display(root, path);
    let text = crate::project::read_doc(path)?;
    Ok(parse(&rel, &text))
}

/// Parse a story body. A frontmatter that does not parse leaves the metadata
/// empty; `story validate`'s check 1 is what reports that.
pub fn parse(rel: &str, text: &str) -> Story {
    let (id, status, consumes) = match crate::doc::frontmatter::parse(
        text,
        crate::doc::frontmatter::Source::Markdown,
        rel,
    ) {
        Ok(fm) => (
            fm.str_key("id").unwrap_or("").to_string(),
            fm.str_key("status").unwrap_or("").to_string(),
            fm.seq_key("consumes").unwrap_or_default(),
        ),
        Err(_) => (String::new(), String::new(), Vec::new()),
    };

    let title = text
        .lines()
        .find_map(|l| l.strip_prefix("# "))
        .unwrap_or("")
        .trim()
        .to_string();

    let tables = md::tables(text);
    let requirements = md::section_table(&tables, "## Requirements")
        .map(|t| {
            t.rows
                .iter()
                .map(|r| ReqRow {
                    id: t.get(r, "REQ").to_string(),
                    covered_by: t
                        .get(r, "Covered by")
                        .split_whitespace()
                        .map(str::to_string)
                        .collect(),
                    line: r.line,
                })
                .filter(|r| r.id.starts_with("REQ-"))
                .collect()
        })
        .unwrap_or_default();

    let files = md::section_table(&tables, "## Files")
        .map(|t| {
            t.rows
                .iter()
                .map(|r| FileRow {
                    path: normalise_path(t.get(r, "Path")),
                    kind: t.get(r, "Kind").to_string(),
                    layer: t.get(r, "Layer").to_string(),
                    line: r.line,
                })
                .filter(|r| !r.path.is_empty())
                .collect()
        })
        .unwrap_or_default();

    Story {
        id,
        rel: rel.to_string(),
        status,
        consumes,
        title,
        acs: acs_of(text),
        requirements,
        files,
        deps: deps_of(text),
        text: text.to_string(),
    }
}

/// Repo-relative, forward slashes, no leading `./`.
pub fn normalise_path(p: &str) -> String {
    p.trim()
        .trim_matches('`')
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_string()
}

/// The list items under `## Acceptance Criteria` matching the fixed grammar.
pub fn acs_of(text: &str) -> Vec<Ac> {
    let mut out = Vec::new();
    for (line, raw) in lines_of_section(text, "## Acceptance Criteria") {
        let Some(rest) = raw.trim().strip_prefix("- ") else {
            continue;
        };
        let Some((id, body)) = rest.split_once(AC_SEPARATOR) else {
            continue;
        };
        let id = id.trim();
        let body = body.trim();
        if !is_ac_id(id) || body.is_empty() {
            continue;
        }
        out.push(Ac {
            id: id.to_string(),
            text: body.to_string(),
            line,
        });
    }
    out
}

/// `AC-` followed by three digits.
fn is_ac_id(s: &str) -> bool {
    s.len() == 6 && s.starts_with("AC-") && s[3..].chars().all(|c| c.is_ascii_digit())
}

/// True when an AC text is testable: Given/When/Then, or one of the verbs
/// with text on both sides.
pub fn testable(text: &str) -> bool {
    if given_when_then(text) {
        return true;
    }
    let lower = text.to_lowercase();
    TESTABLE_VERBS.iter().any(|verb| {
        let needle = format!(" {verb} ");
        match lower.find(&needle) {
            Some(at) => {
                !lower[..at].trim().is_empty() && !lower[at + needle.len()..].trim().is_empty()
            }
            None => false,
        }
    })
}

/// `Given .+ When .+ Then .+`, in that order, each with text after it.
fn given_when_then(text: &str) -> bool {
    let Some(g) = text.find("Given ") else {
        return false;
    };
    let Some(w) = text[g..].find(" When ").map(|o| g + o) else {
        return false;
    };
    let Some(t) = text[w..].find(" Then ").map(|o| w + o) else {
        return false;
    };
    !text[g + 6..w].trim().is_empty()
        && !text[w + 6..t].trim().is_empty()
        && !text[t + 6..].trim().is_empty()
}

/// The `STORY-nnn` ids under `## Dependencies`.
pub fn deps_of(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (_, raw) in lines_of_section(text, "## Dependencies") {
        let Some(rest) = raw.trim().strip_prefix("- ") else {
            continue;
        };
        let id = rest.split([':', ' ']).next().unwrap_or("").trim();
        if id.starts_with("STORY-") && crate::doc::ids::is_id(id) && !out.contains(&id.to_string())
        {
            out.push(id.to_string());
        }
    }
    out
}

/// Every line under the H2 `name`, with its one-based line number.
fn lines_of_section(text: &str, name: &str) -> Vec<(u32, String)> {
    let mut out = Vec::new();
    let mut inside = false;
    for (i, raw) in text.lines().enumerate() {
        let t = raw.trim_end();
        if t.starts_with("## ") {
            if inside {
                break;
            }
            inside = t.trim() == name;
            continue;
        }
        if inside && t.starts_with("# ") {
            break;
        }
        if inside {
            out.push((i as u32 + 1, t.to_string()));
        }
    }
    out
}

// ------------------------------------------------------------------ sprint

/// One `stories[]` or `deferred[]` entry.
#[derive(Debug, Clone)]
pub struct SprintEntry {
    /// `STORY-nnn`.
    pub id: String,
    /// The story status, or the deferral reason.
    pub status: String,
}

/// `stories/sprint.yaml`.
#[derive(Debug, Clone, Default)]
pub struct Sprint {
    /// The sprint id.
    pub id: String,
    /// `EPIC-nnn`.
    pub epic: String,
    /// The concurrent set.
    pub stories: Vec<SprintEntry>,
    /// The deferred set.
    pub deferred: Vec<SprintEntry>,
}

impl Sprint {
    /// Every id of `stories[]` and `deferred[]`.
    pub fn all_ids(&self) -> Vec<String> {
        self.stories
            .iter()
            .chain(&self.deferred)
            .map(|e| e.id.clone())
            .collect()
    }
}

/// Read `stories/sprint.yaml`, `None` when it is absent.
pub fn load_sprint(root: &Path) -> Result<Option<Sprint>, CliError> {
    let rel = ".devforgeai/stories/sprint.yaml";
    let path = root.join(".devforgeai").join("stories").join("sprint.yaml");
    if !path.exists() {
        return Ok(None);
    }
    let text = crate::project::read_doc(&path)?;
    let v: serde_yaml_ng::Value = serde_yaml_ng::from_str(&text).map_err(|e| {
        CliError::at(
            "DFA-E401",
            format!("{rel} is not valid YAML: {e}"),
            rel.to_string(),
        )
    })?;

    let entries = |key: &str| -> Vec<SprintEntry> {
        v.get(key)
            .and_then(|x| x.as_sequence())
            .map(|s| {
                s.iter()
                    .filter_map(|e| {
                        let id = e.get("id").and_then(|x| x.as_str())?.to_string();
                        let status = e
                            .get("status")
                            .or_else(|| e.get("reason"))
                            .and_then(|x| x.as_str())
                            .unwrap_or("")
                            .to_string();
                        Some(SprintEntry { id, status })
                    })
                    .collect()
            })
            .unwrap_or_default()
    };

    Ok(Some(Sprint {
        id: v
            .get("id")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
        epic: v
            .get("epic")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string(),
        stories: entries("stories"),
        deferred: entries("deferred"),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BODY: &str = "---\nschema: devforgeai/story/1\nid: STORY-014\nphase: plan\nstatus: ready\nproduced_by: planning-work\nconsumes: [REQ-001]\nopen_questions: []\n---\n\n# Order checkout\n\n## Requirements\n\n| REQ | Statement | Covered by |\n|---|---|---|\n| REQ-001 | The shopper places an order | AC-001 AC-002 |\n\n## Acceptance Criteria\n\n- AC-001: Given a cart When the shopper pays Then an order row exists.\n- AC-002: The endpoint returns 201.\n\n## Files\n\n| Path | Kind | Layer |\n|---|---|---|\n| src/application/place_order.rs | source | application |\n\n## Dependencies\n\n- STORY-011: the cart\n";

    #[test]
    fn a_story_parses_its_four_sections() {
        let s = parse(".devforgeai/stories/STORY-014.md", BODY);
        assert_eq!(s.id, "STORY-014");
        assert_eq!(s.status, "ready");
        assert_eq!(s.title, "Order checkout");
        assert_eq!(s.consumes, vec!["REQ-001".to_string()]);
        assert_eq!(s.acs.len(), 2);
        assert_eq!(s.acs[0].id, "AC-001");
        assert_eq!(s.requirements.len(), 1);
        assert_eq!(s.requirements[0].covered_by, vec!["AC-001", "AC-002"]);
        assert_eq!(s.files.len(), 1);
        assert_eq!(s.files[0].kind, "source");
        assert_eq!(s.deps, vec!["STORY-011".to_string()]);
    }

    #[test]
    fn given_when_then_is_testable() {
        assert!(testable(
            "Given a cart When the shopper pays Then an order row exists."
        ));
        assert!(!testable("Given a cart Then an order row exists."));
    }

    #[test]
    fn a_verb_with_text_on_both_sides_is_testable() {
        assert!(testable("The endpoint returns 201."));
        assert!(testable("The form rejects a blank name."));
        assert!(!testable("returns"));
        assert!(!testable("The system is good."));
    }

    #[test]
    fn a_trailing_verb_with_nothing_after_it_is_not_testable() {
        assert!(!testable("The endpoint returns "));
    }

    #[test]
    fn a_none_line_yields_no_dependency() {
        let text = "## Dependencies\n\nnone\n";
        assert!(deps_of(text).is_empty());
    }

    #[test]
    fn a_path_is_normalised_to_forward_slashes() {
        assert_eq!(normalise_path("./src\\a.rs"), "src/a.rs");
    }
}
