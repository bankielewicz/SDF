//! `context audit`: the eight mechanical checks `specs/04-constitute.md` owns
//! over the six context files and the ADRs.
//!
//! Nothing here reads meaning. CA-5, the contradiction check, compares key and
//! value text after trimming, collapsing whitespace, and lowercasing; every
//! other check compares ids and enum members.

use crate::ctx::Ctx;
use crate::errors::CliError;
use crate::md;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The H2 section list of each of the six context files, in order. The source
/// is `specs/04-constitute.md` `## Outputs` -> `Section lists`.
pub const CONTEXT_SECTIONS: &[(&str, &[&str])] = &[
    (
        "tech-stack",
        &[
            "## Languages",
            "## Runtimes",
            "## Frameworks",
            "## Data stores",
            "## Tooling",
            "## Excluded technologies",
        ],
    ),
    (
        "source-tree",
        &[
            "## Roots",
            "## Layers",
            "## Directory map",
            "## File placement rules",
            "## Naming conventions",
            "## Generated and excluded paths",
        ],
    ),
    (
        "dependencies",
        &[
            "## Approved dependencies",
            "## Forbidden dependencies",
            "## Version policy",
            "## License policy",
            "## Addition procedure",
        ],
    ),
    (
        "coding-standards",
        &[
            "## Formatting",
            "## Naming",
            "## Error handling",
            "## Logging",
            "## Testing standards",
            "## Documentation",
            "## Design tokens",
        ],
    ),
    (
        "architecture-constraints",
        &[
            "## Constraints",
            "## Layer dependency rules",
            "## Constraint index",
        ],
    ),
    (
        "anti-patterns",
        &["## Anti-patterns", "## Anti-pattern index"],
    ),
];

/// The H2 section list of an ADR, in order.
pub const ADR_SECTIONS: &[&str] = &[
    "## Context",
    "## Decision",
    "## Consequences",
    "## Constraints introduced",
    "## Supersedes",
];

/// The closed key namespace of the `| Key | Value | Source |` tables, in the
/// order `specs/04-constitute.md` `## Outputs` -> `The key namespace` lists it.
/// The second element is the value set a wildcard segment is drawn from, empty
/// when the row states none.
pub const KEY_NAMESPACE: &[(&str, &[&str])] = &[
    ("language.primary", &[]),
    ("language.primary.version", &[]),
    ("language.secondary.<n>", &[]),
    ("language.secondary.<n>.version", &[]),
    ("runtime.name", &[]),
    ("runtime.version", &[]),
    (
        "framework.<role>",
        &[
            "web", "api", "ui", "orm", "test", "build", "lint", "format", "package",
        ],
    ),
    (
        "framework.<role>.version",
        &[
            "web", "api", "ui", "orm", "test", "build", "lint", "format", "package",
        ],
    ),
    (
        "datastore.<role>",
        &["primary", "cache", "search", "queue", "blob"],
    ),
    (
        "datastore.<role>.version",
        &["primary", "cache", "search", "queue", "blob"],
    ),
    ("source.root", &[]),
    ("test.root", &[]),
    ("build.output.root", &[]),
    ("layer.<name>.path", &[]),
    ("dep.<name>.version", &[]),
    ("dep.<name>.scope", &[]),
    ("style.indent", &[]),
    ("style.line.max", &[]),
    ("style.quote", &[]),
    (
        "naming.<entity>",
        &["type", "function", "variable", "constant"],
    ),
    ("naming.<entity>", &["file", "directory", "test-file"]),
    ("tokens.path", &[]),
];

/// The `detector_kind` enum.
pub const DETECTOR_KINDS: &[&str] = &["literal", "regex", "glob"];

/// The `severity` enum of an anti-pattern, worst first.
pub const SEVERITIES: &[&str] = &["blocker", "high", "medium", "low"];

/// The eight check ids, in spec order.
pub const CHECK_IDS: &[&str] = &[
    "CA-1", "CA-2", "CA-3", "CA-4", "CA-5", "CA-6", "CA-7", "CA-8",
];

/// One side of a CA-5 contradiction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Site {
    /// The project-relative path.
    pub path: String,
    /// The one-based line.
    pub line: u32,
    /// The value as written.
    pub value: String,
}

/// One problem the audit found.
#[derive(Debug, Clone)]
pub struct Finding {
    /// The `CA-n` the finding belongs to.
    pub check: &'static str,
    /// The `DFA-` code.
    pub code: &'static str,
    /// The rendered message.
    pub message: String,
    /// The path the finding points at.
    pub path: String,
    /// The one-based line, or 0.
    pub line: u32,
    /// The contradicting key, for CA-5.
    pub key: String,
    /// The first statement, for CA-5.
    pub a: Option<Site>,
    /// The second statement, for CA-5.
    pub b: Option<Site>,
}

/// What the audit learned about one context file.
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// The stem.
    pub name: String,
    /// True when the path exists.
    pub present: bool,
    /// True when the frontmatter parsed.
    pub valid: bool,
    /// The frontmatter `status`, or `""`.
    pub status: String,
    /// The number of `| Key | Value | Source |` rows the file contributes.
    pub keys: usize,
}

/// The result of one of the eight checks.
#[derive(Debug, Clone)]
pub struct CheckResult {
    /// `CA-n`.
    pub id: &'static str,
    /// `pass` or `fail`.
    pub status: &'static str,
    /// The number of findings.
    pub count: usize,
}

/// The whole audit.
#[derive(Debug, Clone, Default)]
pub struct Audit {
    /// The six context files.
    pub files: Vec<FileInfo>,
    /// Rows of `## Constraint index`.
    pub constraints: usize,
    /// Rows of `## Anti-pattern index`.
    pub anti_patterns: usize,
    /// ADR files found.
    pub adrs: usize,
    /// The eight checks.
    pub checks: Vec<CheckResult>,
    /// Every finding, in check order.
    pub findings: Vec<Finding>,
}

impl Audit {
    /// True when every check passed.
    pub fn clean(&self) -> bool {
        self.findings.is_empty()
    }

    /// The `CA-n` ids that failed, in order.
    pub fn failed(&self) -> Vec<&'static str> {
        self.checks
            .iter()
            .filter(|c| c.status == "fail")
            .map(|c| c.id)
            .collect()
    }
}

/// One constraint row of `## Constraint index`.
struct Con {
    id: String,
    status: String,
    source: String,
    line: u32,
}

/// One ADR, read once.
struct Adr {
    rel: String,
    id: String,
    status: String,
    consumes: Vec<String>,
    text: String,
    line_of_id: u32,
}

/// Run the eight checks.
pub fn run(ctx: &mut Ctx) -> Result<Audit, CliError> {
    let root = ctx.root.clone();
    let mut audit = Audit::default();
    let mut by_check: BTreeMap<&'static str, Vec<Finding>> = BTreeMap::new();

    // ------------------------------------------------- CA-1, CA-2, CA-3
    let index = crate::doc::ids::build(&root);
    // In the spec's file order, so CA-5 names the owning file first.
    let mut texts: Vec<(String, String)> = Vec::new();

    for (stem, sections) in CONTEXT_SECTIONS {
        let rel = format!(".devforgeai/context/{stem}.md");
        let path = root
            .join(".devforgeai")
            .join("context")
            .join(format!("{stem}.md"));
        let mut info = FileInfo {
            name: (*stem).to_string(),
            present: path.exists(),
            valid: false,
            status: String::new(),
            keys: 0,
        };

        if !info.present {
            by_check.entry("CA-1").or_default().push(Finding {
                check: "CA-1",
                code: "DFA-E220",
                message: format!("{rel} not found; conventions section 3 lists six context files"),
                path: rel.clone(),
                line: 0,
                key: String::new(),
                a: None,
                b: None,
            });
            audit.files.push(info);
            continue;
        }

        let dctx = crate::doc::Ctx {
            root: &root,
            index: &index,
            frontmatter_only: true,
            stdin_content: None,
        };
        match crate::doc::validate(&path, &dctx) {
            Ok(doc) => {
                info.valid = doc.valid();
                info.status = doc.status.clone();
                for e in &doc.errors {
                    by_check.entry("CA-1").or_default().push(Finding {
                        check: "CA-1",
                        code: e.code,
                        message: e.message.clone(),
                        path: rel.clone(),
                        line: e.line,
                        key: String::new(),
                        a: None,
                        b: None,
                    });
                }
            }
            Err(e) => {
                if let Some(d) = e.diag() {
                    by_check.entry("CA-1").or_default().push(Finding {
                        check: "CA-1",
                        code: d.code,
                        message: d.message.clone(),
                        path: rel.clone(),
                        line: d.line,
                        key: String::new(),
                        a: None,
                        b: None,
                    });
                }
            }
        }

        let text = crate::project::read_doc(&path)?;

        // CA-2: accepted with no open questions.
        if let Ok(fm) =
            crate::doc::frontmatter::parse(&text, crate::doc::frontmatter::Source::Markdown, &rel)
        {
            let status = fm.str_key("status").unwrap_or("").to_string();
            info.status = status.clone();
            let open = fm.seq_key("open_questions").unwrap_or_default();
            if status != "accepted" || !open.is_empty() {
                by_check.entry("CA-2").or_default().push(Finding {
                    check: "CA-2",
                    code: "DFA-E223",
                    message: format!(
                        "{rel} is status '{status}' with {} open questions; context audit needs accepted and []",
                        open.len()
                    ),
                    path: rel.clone(),
                    line: fm.line_of("status"),
                    key: String::new(),
                    a: None,
                    b: None,
                });
            }
        }

        // CA-3: the H2 list, in order, with no extra.
        heading_findings(&rel, &text, sections, &mut by_check);

        info.keys = key_rows(&md::tables(&text)).len();
        texts.push((rel, text));
        audit.files.push(info);
    }

    // ---------------------------------------------------------- the ADRs
    let adrs = read_adrs(&root)?;
    audit.adrs = adrs.len();
    for a in &adrs {
        heading_findings(&a.rel, &a.text, ADR_SECTIONS, &mut by_check);
    }

    // ------------------------------------------------- the constraint index
    let ac = text_of(&texts, ".devforgeai/context/architecture-constraints.md");
    let ac_tables = md::tables(&ac);
    let constraints = constraint_index(&ac_tables);
    audit.constraints = constraints.len();

    let ap = text_of(&texts, ".devforgeai/context/anti-patterns.md");
    let ap_tables = md::tables(&ap);
    let ap_rows = md::section_table(&ap_tables, "## Anti-pattern index")
        .map(|t| t.rows.len())
        .unwrap_or(0);
    audit.anti_patterns = ap_rows;

    let reqs = requirement_ids(&root);

    // CA-4 and the duplicated-CON rule.
    ca4(&constraints, &adrs, &reqs, &mut by_check);
    ca_duplicate_con(&root, &texts, &adrs, &mut by_check);

    // CA-5, the contradiction check.
    ca5(&texts, &adrs, &mut by_check);

    // CA-6, the anti-pattern index.
    ca6(&ap_tables, &constraints, &mut by_check);

    // CA-7 and CA-8 over the ADRs.
    ca7(&adrs, &reqs, &constraints, &mut by_check);
    ca8(&adrs, &constraints, &mut by_check);

    for id in CHECK_IDS {
        let count = by_check.get(id).map(Vec::len).unwrap_or(0);
        audit.checks.push(CheckResult {
            id,
            status: if count == 0 { "pass" } else { "fail" },
            count,
        });
    }
    for id in CHECK_IDS {
        if let Some(f) = by_check.remove(id) {
            audit.findings.extend(f);
        }
    }
    Ok(audit)
}

/// The text read for `rel`, or `""` when the file was absent.
fn text_of(texts: &[(String, String)], rel: &str) -> String {
    texts
        .iter()
        .find(|(k, _)| k == rel)
        .map(|(_, v)| v.clone())
        .unwrap_or_default()
}

/// CA-3: the H2 lines equal `want`, in that order, with no extra H2.
fn heading_findings(
    rel: &str,
    text: &str,
    want: &[&str],
    by_check: &mut BTreeMap<&'static str, Vec<Finding>>,
) {
    let found = md::h2(text);
    for (i, heading) in want.iter().enumerate() {
        match found.iter().position(|f| f == heading) {
            None => by_check.entry("CA-3").or_default().push(Finding {
                check: "CA-3",
                code: "DFA-E213",
                message: format!("{rel} heading '{heading}' is absent"),
                path: rel.to_string(),
                line: 0,
                key: String::new(),
                a: None,
                b: None,
            }),
            Some(at) if at != i => by_check.entry("CA-3").or_default().push(Finding {
                check: "CA-3",
                code: "DFA-E213",
                message: format!(
                    "{rel} heading '{heading}' is at position {}, expected {}",
                    at + 1,
                    i + 1
                ),
                path: rel.to_string(),
                line: 0,
                key: String::new(),
                a: None,
                b: None,
            }),
            Some(_) => {}
        }
    }
    for extra in found.iter().filter(|f| !want.contains(&f.as_str())) {
        by_check.entry("CA-3").or_default().push(Finding {
            check: "CA-3",
            code: "DFA-E213",
            message: format!("{rel} heading '{extra}' is not in the section list"),
            path: rel.to_string(),
            line: 0,
            key: String::new(),
            a: None,
            b: None,
        });
    }
}

/// Read every `.devforgeai/adr/ADR-nnn.md`.
fn read_adrs(root: &Path) -> Result<Vec<Adr>, CliError> {
    let dir = root.join(".devforgeai").join("adr");
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Ok(out);
    };
    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("ADR-") && n.ends_with(".md"))
                .unwrap_or(false)
        })
        .collect();
    paths.sort();

    for path in paths {
        let rel = crate::project::rel_display(root, &path);
        let text = crate::project::read_doc(&path)?;
        let (id, status, consumes, line) = match crate::doc::frontmatter::parse(
            &text,
            crate::doc::frontmatter::Source::Markdown,
            &rel,
        ) {
            Ok(fm) => (
                fm.str_key("id").unwrap_or("").to_string(),
                fm.str_key("status").unwrap_or("").to_string(),
                fm.seq_key("consumes").unwrap_or_default(),
                fm.line_of("id"),
            ),
            Err(_) => (String::new(), String::new(), Vec::new(), 0),
        };
        out.push(Adr {
            rel,
            id,
            status,
            consumes,
            text,
            line_of_id: line,
        });
    }
    Ok(out)
}

/// The `## Constraint index` rows.
fn constraint_index(tables: &[md::Table]) -> Vec<Con> {
    let Some(t) = md::section_table(tables, "## Constraint index") else {
        return Vec::new();
    };
    t.rows
        .iter()
        .map(|r| Con {
            id: t.get(r, "CON").to_string(),
            status: t.get(r, "Status").to_string(),
            source: t.get(r, "Source").to_string(),
            line: r.line,
        })
        .filter(|c| c.id.starts_with("CON-"))
        .collect()
}

/// Every `REQ-nnn` defined in `requirements.yaml`.
fn requirement_ids(root: &Path) -> Vec<String> {
    let path = root.join(".devforgeai").join("requirements.yaml");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(v) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&text) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for key in ["requirements", "personas", "epics"] {
        if let Some(seq) = v.get(key).and_then(|x| x.as_sequence()) {
            for item in seq {
                if let Some(id) = item.get("id").and_then(|x| x.as_str()) {
                    out.push(id.to_string());
                }
            }
        }
    }
    if let Some(id) = v.get("id").and_then(|x| x.as_str()) {
        out.push(id.to_string());
    }
    out
}

/// CA-4: an active CON is introduced by an ADR or traces to a REQ.
fn ca4(
    constraints: &[Con],
    adrs: &[Adr],
    reqs: &[String],
    by_check: &mut BTreeMap<&'static str, Vec<Finding>>,
) {
    let introduced: Vec<String> = adrs
        .iter()
        .flat_map(|a| {
            let tables = md::tables(&a.text);
            md::section_table(&tables, "## Constraints introduced")
                .map(|t| {
                    t.rows
                        .iter()
                        .map(|r| t.get(r, "CON").to_string())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .collect();

    for c in constraints.iter().filter(|c| c.status == "active") {
        if introduced.iter().any(|i| i == &c.id) {
            continue;
        }
        let traces = crate::doc::ids::scan_ids(&c.source)
            .into_iter()
            .any(|(_, id)| id.starts_with("REQ-") && reqs.iter().any(|r| r == &id));
        if traces {
            continue;
        }
        by_check.entry("CA-4").or_default().push(Finding {
            check: "CA-4",
            code: "DFA-E224",
            message: format!(
                "{} is introduced by no ADR and its source '{}' resolves to no requirement",
                c.id, c.source
            ),
            path: ".devforgeai/context/architecture-constraints.md".to_string(),
            line: c.line,
            key: String::new(),
            a: None,
            b: None,
        });
    }
}

/// A `CON-nnn` defined by an `### ` heading in two distinct files.
fn ca_duplicate_con(
    _root: &Path,
    texts: &[(String, String)],
    adrs: &[Adr],
    by_check: &mut BTreeMap<&'static str, Vec<Finding>>,
) {
    let mut seen: BTreeMap<String, String> = BTreeMap::new();
    let all = texts
        .iter()
        .cloned()
        .chain(adrs.iter().map(|a| (a.rel.clone(), a.text.clone())));

    for (rel, text) in all {
        for heading in md::h3(&text) {
            let Some(id) = heading.split_whitespace().next() else {
                continue;
            };
            if !id.starts_with("CON-") {
                continue;
            }
            match seen.get(id) {
                Some(first) if first != &rel => {
                    by_check.entry("CA-4").or_default().push(Finding {
                        check: "CA-4",
                        code: "DFA-E222",
                        message: format!("{id} is defined in {first} and {rel}"),
                        path: rel.clone(),
                        line: 0,
                        key: String::new(),
                        a: None,
                        b: None,
                    });
                }
                Some(_) => {}
                None => {
                    seen.insert(id.to_string(), rel.clone());
                }
            }
        }
    }
}

/// Every `| Key | Value | Source |` row of a set of tables.
fn key_rows(tables: &[md::Table]) -> Vec<(String, String, u32)> {
    md::tables_with_header(tables, &["Key", "Value", "Source"])
        .into_iter()
        .flat_map(|t| {
            t.rows
                .iter()
                .map(|r| (r.cell(0).to_string(), r.cell(1).to_string(), r.line))
        })
        .filter(|(k, _, _)| !k.is_empty())
        .collect()
}

/// Trim, collapse runs of whitespace, lowercase.
pub fn normalise(s: &str) -> String {
    s.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// True when `key` is inside the closed namespace.
pub fn in_namespace(key: &str) -> bool {
    KEY_NAMESPACE
        .iter()
        .any(|(pattern, values)| matches_pattern(key, pattern, values))
}

/// A concrete key against one namespace row, segment by segment.
fn matches_pattern(key: &str, pattern: &str, values: &[&str]) -> bool {
    let k: Vec<&str> = key.split('.').collect();
    let p: Vec<&str> = pattern.split('.').collect();
    if k.len() != p.len() {
        return false;
    }
    for (ks, ps) in k.iter().zip(&p) {
        let ok = match *ps {
            "<n>" => !ks.is_empty() && ks.len() <= 3 && ks.chars().all(|c| c.is_ascii_digit()),
            "<role>" | "<entity>" => !ks.is_empty() && values.contains(ks),
            "<name>" => !ks.is_empty(),
            literal => ks == &literal,
        };
        if !ok {
            return false;
        }
    }
    true
}

/// CA-5: no key holds two distinct values, and every key is in the namespace.
fn ca5(
    texts: &[(String, String)],
    adrs: &[Adr],
    by_check: &mut BTreeMap<&'static str, Vec<Finding>>,
) {
    let mut first: BTreeMap<String, (Site, String)> = BTreeMap::new();
    let sources = texts.iter().cloned().chain(
        adrs.iter()
            .filter(|a| a.status == "accepted")
            .map(|a| (a.rel.clone(), a.text.clone())),
    );

    for (rel, text) in sources {
        for (key, value, line) in key_rows(&md::tables(&text)) {
            let nk = normalise(&key);
            let nv = normalise(&value);
            let site = Site {
                path: rel.clone(),
                line,
                value: value.clone(),
            };

            if !in_namespace(&key) {
                by_check.entry("CA-5").or_default().push(Finding {
                    check: "CA-5",
                    code: "DFA-E221",
                    message: format!(
                        "{rel} line {line} states '{key}', which is outside the closed key namespace"
                    ),
                    path: rel.clone(),
                    line,
                    key: key.clone(),
                    a: Some(site.clone()),
                    b: None,
                });
                continue;
            }

            match first.get(&nk) {
                Some((a, av)) if av != &nv => {
                    by_check.entry("CA-5").or_default().push(Finding {
                        check: "CA-5",
                        code: "DFA-E221",
                        message: format!(
                            "{} line {} states '{}'; {} line {} states '{}'",
                            a.path, a.line, a.value, rel, line, value
                        ),
                        path: rel.clone(),
                        line,
                        key: key.clone(),
                        a: Some(a.clone()),
                        b: Some(site),
                    });
                }
                Some(_) => {}
                None => {
                    first.insert(nk, (site, nv));
                }
            }
        }
    }
}

/// CA-6: every anti-pattern row is complete.
fn ca6(
    ap_tables: &[md::Table],
    constraints: &[Con],
    by_check: &mut BTreeMap<&'static str, Vec<Finding>>,
) {
    let Some(t) = md::section_table(ap_tables, "## Anti-pattern index") else {
        return;
    };
    for r in &t.rows {
        let id = t.get(r, "AP");
        if !id.starts_with("AP-") {
            continue;
        }
        let detector = t.get(r, "Detector");
        let kind = t.get(r, "Detector kind");
        let source = t.get(r, "Source");

        let problem = if detector.is_empty() {
            Some("no detector".to_string())
        } else if !DETECTOR_KINDS.contains(&kind) {
            Some(format!("detector_kind '{kind}'"))
        } else if !constraints
            .iter()
            .any(|c| c.id == source && c.status == "active")
        {
            Some(format!("source '{source}' which is not an active CON"))
        } else {
            None
        };

        if let Some(p) = problem {
            by_check.entry("CA-6").or_default().push(Finding {
                check: "CA-6",
                code: "DFA-E225",
                message: format!("{id} has {p}"),
                path: ".devforgeai/context/anti-patterns.md".to_string(),
                line: r.line,
                key: String::new(),
                a: None,
                b: None,
            });
        }
    }
}

/// CA-7: ADR ids are unique, `consumes` resolves, introduced CONs are indexed.
fn ca7(
    adrs: &[Adr],
    reqs: &[String],
    constraints: &[Con],
    by_check: &mut BTreeMap<&'static str, Vec<Finding>>,
) {
    let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
    for a in adrs {
        if a.id.is_empty() {
            continue;
        }
        if let Some(other) = seen.get(a.id.as_str()) {
            by_check.entry("CA-7").or_default().push(Finding {
                check: "CA-7",
                code: "DFA-E226",
                message: format!("{} id duplicates {other}", a.rel),
                path: a.rel.clone(),
                line: a.line_of_id,
                key: String::new(),
                a: None,
                b: None,
            });
        } else {
            seen.insert(&a.id, &a.rel);
        }

        for c in &a.consumes {
            if c.is_empty() {
                continue;
            }
            if !reqs.iter().any(|r| r == c) {
                by_check.entry("CA-7").or_default().push(Finding {
                    check: "CA-7",
                    code: "DFA-E226",
                    message: format!(
                        "{} consumes {c} which requirements.yaml does not define",
                        a.rel
                    ),
                    path: a.rel.clone(),
                    line: a.line_of_id,
                    key: String::new(),
                    a: None,
                    b: None,
                });
            }
        }

        let tables = md::tables(&a.text);
        if let Some(t) = md::section_table(&tables, "## Constraints introduced") {
            for r in &t.rows {
                let con = t.get(r, "CON");
                if !con.starts_with("CON-") {
                    continue;
                }
                if !constraints.iter().any(|c| c.id == con) {
                    by_check.entry("CA-7").or_default().push(Finding {
                        check: "CA-7",
                        code: "DFA-E226",
                        message: format!(
                            "{} introduces {con} which the constraint index omits",
                            a.rel
                        ),
                        path: a.rel.clone(),
                        line: r.line,
                        key: String::new(),
                        a: None,
                        b: None,
                    });
                }
            }
        }
    }
}

/// CA-8: a supersession retires both the ADR and the CONs it names.
fn ca8(adrs: &[Adr], constraints: &[Con], by_check: &mut BTreeMap<&'static str, Vec<Finding>>) {
    for a in adrs {
        let tables = md::tables(&a.text);
        let Some(t) = md::section_table(&tables, "## Supersedes") else {
            continue;
        };
        for r in &t.rows {
            let target = t.get(r, "Supersedes ADR");
            if target.starts_with("ADR-") {
                let status = adrs
                    .iter()
                    .find(|o| o.id == target)
                    .map(|o| o.status.as_str())
                    .unwrap_or("");
                if status != "superseded" {
                    by_check.entry("CA-8").or_default().push(Finding {
                        check: "CA-8",
                        code: "DFA-E227",
                        message: format!(
                            "{} supersedes {target}, which is status '{status}'",
                            a.rel
                        ),
                        path: a.rel.clone(),
                        line: r.line,
                        key: String::new(),
                        a: None,
                        b: None,
                    });
                }
            }
            let retires = t.get(r, "Retires CON");
            if retires.starts_with("CON-") {
                let status = constraints
                    .iter()
                    .find(|c| c.id == retires)
                    .map(|c| c.status.as_str())
                    .unwrap_or("");
                if status != "retired" {
                    by_check.entry("CA-8").or_default().push(Finding {
                        check: "CA-8",
                        code: "DFA-E227",
                        message: format!("{} retires {retires}, which is status '{status}'", a.rel),
                        path: a.rel.clone(),
                        line: r.line,
                        key: String::new(),
                        a: None,
                        b: None,
                    });
                }
            }
        }
    }
}

/// The `data` object `context audit` and the `context_audit` check publish.
pub fn to_json(a: &Audit) -> serde_json::Value {
    serde_json::json!({
        "files": a.files.iter().map(|f| serde_json::json!({
            "name": f.name,
            "present": f.present,
            "valid": f.valid,
            "status": f.status,
            "keys": f.keys,
        })).collect::<Vec<_>>(),
        "constraints": a.constraints,
        "anti_patterns": a.anti_patterns,
        "adrs": a.adrs,
        "checks": a.checks.iter().map(|c| serde_json::json!({
            "id": c.id,
            "status": c.status,
            "count": c.count,
        })).collect::<Vec<_>>(),
        "findings": a.findings.iter().map(finding_json).collect::<Vec<_>>(),
    })
}

fn finding_json(f: &Finding) -> serde_json::Value {
    let mut o = serde_json::Map::new();
    o.insert("check".into(), f.check.into());
    o.insert("code".into(), f.code.into());
    o.insert("message".into(), f.message.clone().into());
    o.insert("path".into(), f.path.clone().into());
    o.insert("line".into(), f.line.into());
    if !f.key.is_empty() {
        o.insert("key".into(), f.key.clone().into());
    }
    for (name, site) in [("a", &f.a), ("b", &f.b)] {
        if let Some(s) = site {
            o.insert(
                name.into(),
                serde_json::json!({ "path": s.path, "line": s.line, "value": s.value }),
            );
        }
    }
    serde_json::Value::Object(o)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_six_files_are_the_six_stems() {
        let stems: Vec<&str> = CONTEXT_SECTIONS.iter().map(|(s, _)| *s).collect();
        assert_eq!(stems, crate::doc::CONTEXT_STEMS.to_vec());
    }

    #[test]
    fn a_wildcard_segment_is_dot_free() {
        assert!(in_namespace("layer.domain.path"));
        assert!(!in_namespace("layer.domain.core.path"));
        assert!(in_namespace("dep.serde.version"));
    }

    #[test]
    fn a_role_wildcard_is_drawn_from_its_stated_set() {
        assert!(in_namespace("framework.web"));
        assert!(!in_namespace("framework.nowhere"));
        assert!(in_namespace("datastore.primary.version"));
        assert!(!in_namespace("datastore.web"));
    }

    #[test]
    fn an_entity_wildcard_takes_either_stated_set() {
        assert!(in_namespace("naming.type"));
        assert!(in_namespace("naming.directory"));
        assert!(!in_namespace("naming.module"));
    }

    #[test]
    fn the_n_wildcard_is_one_to_three_digits() {
        assert!(in_namespace("language.secondary.2"));
        assert!(in_namespace("language.secondary.123.version"));
        assert!(!in_namespace("language.secondary.1234"));
        assert!(!in_namespace("language.secondary.x"));
    }

    #[test]
    fn normalisation_trims_collapses_and_lowercases() {
        assert_eq!(normalise("  Postgres   16 "), "postgres 16");
        assert_eq!(normalise("POSTGRES\t16"), "postgres 16");
    }
}
