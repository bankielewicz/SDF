//! The anti-pattern index and the three detector kinds it drives.
//!
//! A rule is one row of `.devforgeai/context/anti-patterns.md`
//! `## Anti-pattern index`. Nothing here reads meaning: `literal` is a
//! substring of the file bytes, `regex` is a match in the `regex` crate
//! dialect, and `glob` matches the path and reads no file.

use crate::errors::CliError;
use crate::md;
use std::path::Path;

/// The `severity` enum, worst first. The index of a value is its rank.
pub const SEVERITIES: &[&str] = &["blocker", "high", "medium", "low"];

/// The `detector_kind` enum.
pub const DETECTOR_KINDS: &[&str] = &["literal", "regex", "glob"];

/// One row of `## Anti-pattern index`.
#[derive(Debug, Clone)]
pub struct Rule {
    /// `AP-nnn`.
    pub id: String,
    /// One of the six categories.
    pub category: String,
    /// One of `SEVERITIES`.
    pub severity: String,
    /// The glob the candidate path must match.
    pub scope: String,
    /// One of `DETECTOR_KINDS`.
    pub detector_kind: String,
    /// The literal, pattern, or glob.
    pub detector: String,
    /// The `CON-nnn` the rule enforces.
    pub source: String,
}

/// One match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    /// The rule id.
    pub id: String,
    /// The rule severity.
    pub severity: String,
    /// The project-relative path.
    pub path: String,
    /// The one-based line, or 0 for a `glob` rule.
    pub line: u32,
    /// The matched text, cut to a readable length.
    pub text: String,
}

/// The rank of a severity, `SEVERITIES.len()` for an unknown value, so an
/// unknown severity is never reported under a `--min-severity` filter.
pub fn rank(severity: &str) -> usize {
    SEVERITIES
        .iter()
        .position(|s| *s == severity)
        .unwrap_or(SEVERITIES.len())
}

/// True when `severity` is at or above `floor`.
pub fn at_or_above(severity: &str, floor: &str) -> bool {
    rank(severity) <= rank(floor) && rank(severity) < SEVERITIES.len()
}

/// Read `## Anti-pattern index`, `Ok(vec![])` when the file is absent.
pub fn rules(root: &Path) -> Result<Vec<Rule>, CliError> {
    let path = root
        .join(".devforgeai")
        .join("context")
        .join("anti-patterns.md");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = crate::project::read_doc(&path)?;
    let tables = md::tables(&text);
    let Some(t) = md::section_table(&tables, "## Anti-pattern index") else {
        return Ok(Vec::new());
    };
    Ok(t.rows
        .iter()
        .map(|r| Rule {
            id: t.get(r, "AP").to_string(),
            category: t.get(r, "Category").to_string(),
            severity: t.get(r, "Severity").to_string(),
            scope: t.get(r, "Scope").to_string(),
            detector_kind: t.get(r, "Detector kind").to_string(),
            detector: t.get(r, "Detector").to_string(),
            source: t.get(r, "Source").to_string(),
        })
        .filter(|r| r.id.starts_with("AP-"))
        .collect())
}

/// A compiled `globset` matcher over one pattern, `None` when it does not
/// compile, which makes the rule match nothing rather than everything.
fn matcher(pattern: &str) -> Option<globset::GlobMatcher> {
    globset::Glob::new(pattern)
        .ok()
        .map(|g| g.compile_matcher())
}

/// Apply every rule at or above `min_severity` to the candidate set.
///
/// `read` supplies a candidate's bytes; a candidate that cannot be read
/// contributes no match rather than an error, so a deleted path in a diff does
/// not stop the scan.
pub fn scan(rules: &[Rule], candidates: &[String], min_severity: &str, root: &Path) -> Vec<Match> {
    let mut out = Vec::new();
    for rule in rules
        .iter()
        .filter(|r| at_or_above(&r.severity, min_severity))
    {
        let Some(scope) = matcher(&rule.scope) else {
            continue;
        };
        for candidate in candidates.iter().filter(|c| scope.is_match(c.as_str())) {
            out.extend(apply(rule, candidate, root));
        }
    }
    out.sort_by(|a, b| (&a.path, a.line, &a.id).cmp(&(&b.path, b.line, &b.id)));
    out
}

/// One rule against one candidate.
fn apply(rule: &Rule, candidate: &str, root: &Path) -> Vec<Match> {
    if rule.detector.is_empty() {
        return Vec::new();
    }
    if rule.detector_kind == "glob" {
        return match matcher(&rule.detector) {
            Some(m) if m.is_match(candidate) => vec![Match {
                id: rule.id.clone(),
                severity: rule.severity.clone(),
                path: candidate.to_string(),
                line: 0,
                text: candidate.to_string(),
            }],
            _ => Vec::new(),
        };
    }

    let Ok(bytes) = std::fs::read(root.join(candidate)) else {
        return Vec::new();
    };
    let text = String::from_utf8_lossy(&bytes);

    let spans: Vec<(usize, String)> = match rule.detector_kind.as_str() {
        "literal" => text
            .match_indices(rule.detector.as_str())
            .map(|(at, m)| (at, m.to_string()))
            .collect(),
        "regex" => match regex::Regex::new(&rule.detector) {
            Ok(re) => re
                .find_iter(&text)
                .map(|m| (m.start(), m.as_str().to_string()))
                .collect(),
            Err(_) => Vec::new(),
        },
        _ => Vec::new(),
    };

    spans
        .into_iter()
        .map(|(at, matched)| {
            // The whole line reads better in a diagnostic than the fragment,
            // and the fragment is the fallback when the line is empty.
            let line_text = line_at(&text, at);
            Match {
                id: rule.id.clone(),
                severity: rule.severity.clone(),
                path: candidate.to_string(),
                line: line_of(&text, at),
                text: cut(
                    if line_text.is_empty() {
                        &matched
                    } else {
                        &line_text
                    },
                    120,
                ),
            }
        })
        .collect()
}

/// The one-based line the byte offset `at` sits on.
fn line_of(text: &str, at: usize) -> u32 {
    text[..at.min(text.len())]
        .bytes()
        .filter(|b| *b == b'\n')
        .count() as u32
        + 1
}

/// The whole line the byte offset `at` sits on, trimmed.
fn line_at(text: &str, at: usize) -> String {
    let at = at.min(text.len());
    let start = text[..at].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let end = text[at..].find('\n').map(|i| at + i).unwrap_or(text.len());
    text[start..end].trim().to_string()
}

/// Cut to `n` characters with an ellipsis.
fn cut(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        return s.to_string();
    }
    let mut out: String = s.chars().take(n).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(kind: &str, detector: &str) -> Rule {
        Rule {
            id: "AP-001".into(),
            category: "layer".into(),
            severity: "high".into(),
            scope: "src/**/*.rs".into(),
            detector_kind: kind.into(),
            detector: detector.into(),
            source: "CON-001".into(),
        }
    }

    #[test]
    fn severity_ranks_worst_first() {
        assert!(at_or_above("blocker", "high"));
        assert!(at_or_above("high", "high"));
        assert!(!at_or_above("medium", "high"));
        assert!(at_or_above("low", "low"));
        assert!(
            !at_or_above("invented", "low"),
            "an unknown value is skipped"
        );
    }

    #[test]
    fn a_literal_detector_reports_the_line_it_sits_on() {
        let dir = tempfile::tempdir().expect("temp");
        std::fs::create_dir_all(dir.path().join("src")).expect("mkdir");
        std::fs::write(
            dir.path().join("src").join("a.rs"),
            "fn a() {}\nlet s = TcpStream::connect();\n",
        )
        .expect("write");

        let m = scan(
            &[rule("literal", "TcpStream")],
            &["src/a.rs".to_string()],
            "high",
            dir.path(),
        );
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].line, 2);
        assert!(m[0].text.contains("TcpStream"));
    }

    #[test]
    fn a_regex_detector_uses_the_regex_crate_dialect() {
        let dir = tempfile::tempdir().expect("temp");
        std::fs::create_dir_all(dir.path().join("src")).expect("mkdir");
        std::fs::write(dir.path().join("src").join("a.rs"), "let x = unwrap();\n").expect("write");
        let m = scan(
            &[rule("regex", r"\bunwrap\(\)")],
            &["src/a.rs".to_string()],
            "high",
            dir.path(),
        );
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].line, 1);
    }

    #[test]
    fn a_glob_detector_matches_the_path_and_reads_no_file() {
        let dir = tempfile::tempdir().expect("temp");
        let mut r = rule("glob", "src/**/legacy_*.rs");
        r.scope = "src/**".into();
        let m = scan(
            &[r],
            &["src/legacy_order.rs".to_string()],
            "high",
            dir.path(),
        );
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].line, 0, "no file was read");
    }

    #[test]
    fn a_candidate_outside_the_scope_glob_is_skipped() {
        let dir = tempfile::tempdir().expect("temp");
        let m = scan(
            &[rule("literal", "x")],
            &["docs/readme.md".to_string()],
            "high",
            dir.path(),
        );
        assert!(m.is_empty());
    }

    #[test]
    fn a_malformed_regex_matches_nothing() {
        let dir = tempfile::tempdir().expect("temp");
        std::fs::create_dir_all(dir.path().join("src")).expect("mkdir");
        std::fs::write(dir.path().join("src").join("a.rs"), "anything\n").expect("write");
        let m = scan(
            &[rule("regex", "(unclosed")],
            &["src/a.rs".to_string()],
            "high",
            dir.path(),
        );
        assert!(m.is_empty());
    }
}
