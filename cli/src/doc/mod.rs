//! The doc-type table and `doc validate`.
//!
//! The doc type comes from the path. Every other row value follows from it.

pub mod frontmatter;
pub mod ids;
pub mod xref;

use crate::errors::{CliError, Diag};
use crate::pattern::Pattern;
use frontmatter::{Frontmatter, Source, KEYS};
use ids::IdIndex;
use std::path::{Path, PathBuf};

/// The six context file stems, in the conventions section 3 order.
pub const CONTEXT_STEMS: &[&str] = &[
    "tech-stack",
    "source-tree",
    "dependencies",
    "coding-standards",
    "architecture-constraints",
    "anti-patterns",
];

/// Whether a doc type carries top-level keys beyond the seven.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Extras {
    /// `DFA-E204` on any eighth key.
    Refused,
    /// Payload beyond the seven is part of the document.
    Permitted,
}

/// One row of the doc-type table.
#[derive(Debug, Clone)]
pub struct DocType {
    /// The row name, as `doc load` and the JSON output use it.
    pub name: &'static str,
    /// The skill that produces the document.
    pub producer: &'static str,
    /// The phase the row belongs to.
    pub phase: String,
    /// The `schema` value, or the pattern for `explore-payload`.
    pub schema: String,
    /// True when `schema` is a pattern rather than a literal.
    pub schema_is_pattern: bool,
    /// The `id` grammar, as a pattern, or `None` when the id is a fixed string.
    pub id_pattern: Option<String>,
    /// The literal `id` value, for the six context files.
    pub id_literal: Option<String>,
    /// The status enum, or empty for `explore-payload`, which takes any
    /// non-empty string.
    pub status: Vec<&'static str>,
    /// The ID prefixes the document may define.
    pub prefixes: Vec<&'static str>,
    /// Whether an eighth top-level key is refused.
    pub extras: Extras,
    /// Where the frontmatter lives.
    pub source: Source,
}

/// Where the frontmatter of a file at `path` lives.
pub fn source_for(path: &Path) -> Source {
    match path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default()
        .as_str()
    {
        "md" => Source::Markdown,
        "json" => Source::Json,
        _ => Source::Yaml,
    }
}

/// One row of the doc-type table, taken column by column.
///
/// The nine parameters are the nine columns the spec's table carries, in its
/// order, so a row below reads as that table reads.
#[allow(clippy::too_many_arguments)]
fn row(
    name: &'static str,
    producer: &'static str,
    phase: &str,
    schema: &str,
    id_pattern: Option<&str>,
    status: &[&'static str],
    prefixes: &[&'static str],
    extras: Extras,
    source: Source,
) -> DocType {
    DocType {
        name,
        producer,
        phase: phase.to_string(),
        schema: schema.to_string(),
        schema_is_pattern: schema.starts_with('^'),
        id_pattern: id_pattern.map(str::to_string),
        id_literal: None,
        status: status.to_vec(),
        prefixes: prefixes.to_vec(),
        extras,
        source,
    }
}

/// The doc type of a path under `.devforgeai/`, or `None` when the path matches
/// no row, which is skipped with no diagnostic so a skill may keep scratch
/// files there.
pub fn doc_type_for(rel: &str) -> Option<DocType> {
    let rel = rel.trim_start_matches("./");
    let rel = rel.strip_prefix(".devforgeai/").unwrap_or(rel);
    let file = rel.rsplit('/').next().unwrap_or(rel);

    // The reports directory carries four rows; the more specific ones first.
    if let Some(stem) = file.strip_suffix(".yaml") {
        if rel.starts_with("reports/") {
            if let Some(date) = stem.strip_prefix("reflect-") {
                if is_date(date) {
                    return Some(row(
                        "reflect-report",
                        "improving-framework",
                        "reflect",
                        "devforgeai/reflect-report/1",
                        Some("^[0-9]{4}-[0-9]{2}-[0-9]{2}$"),
                        &["draft", "final"],
                        &["OBS", "REC"],
                        Extras::Permitted,
                        Source::Yaml,
                    ));
                }
            }
            if let Some(subject) = stem.strip_suffix("-build") {
                if ids::is_id(subject) {
                    return Some(row(
                        "build-report",
                        "devforgeai-cli",
                        "build",
                        "devforgeai/report/1",
                        Some("^STORY-[0-9]{3}$"),
                        &["pass", "fail", "send_back", "skip"],
                        &[],
                        Extras::Permitted,
                        Source::Yaml,
                    ));
                }
            }
            if let Some(subject) = stem.strip_suffix("-qa") {
                if ids::is_id(subject) {
                    return Some(row(
                        "qa-report",
                        "validating-quality",
                        "verify",
                        "devforgeai/qa-report/1",
                        Some("^STORY-[0-9]{3}$"),
                        &["pass", "fail", "send_back"],
                        &["FIND"],
                        Extras::Permitted,
                        Source::Yaml,
                    ));
                }
            }
            for phase in [
                "explore",
                "discover",
                "constitute",
                "plan",
                "verify",
                "release",
                "design",
            ] {
                if let Some(subject) = stem.strip_suffix(&format!("-{phase}")) {
                    if !subject.is_empty() {
                        return Some(row(
                            "gate-report",
                            "devforgeai-cli",
                            phase,
                            "devforgeai/report/1",
                            None,
                            &["pass", "fail", "send_back", "skip"],
                            &[],
                            Extras::Permitted,
                            Source::Yaml,
                        ));
                    }
                }
            }
            return None;
        }
    }

    match rel {
        "explore/brief.md" => Some(row(
            "explore-brief",
            "exploring-ideas",
            "explore",
            "devforgeai/explore-brief/1",
            Some("^IDEA-[0-9]{3}$"),
            &["drafting", "scanned", "specified", "mocked", "decided"],
            &["IDEA", "FLOW"],
            Extras::Refused,
            Source::Markdown,
        )),
        "explore/decision.yaml" => Some(row(
            "explore-decision",
            "exploring-ideas",
            "explore",
            "devforgeai/explore-decision/1",
            Some("^IDEA-[0-9]{3}$"),
            &["recorded"],
            &["IDEA", "FLOW"],
            Extras::Permitted,
            Source::Yaml,
        )),
        "requirements.yaml" => Some(row(
            "requirements",
            "discovering-requirements",
            "discover",
            "devforgeai/requirements/1",
            Some("^IDEA-[0-9]{3}$"),
            &["drafting", "awaiting_acceptance", "accepted", "reopened"],
            &["REQ", "EPIC", "PERSONA", "IDEA", "FLOW", "UI"],
            Extras::Permitted,
            Source::Yaml,
        )),
        "stories/sprint.yaml" => Some(row(
            "sprint",
            "planning-work",
            "plan",
            "devforgeai/sprint/1",
            Some("^SPRINT-[0-9]{3}$"),
            &["planned", "active", "closed"],
            &["SPRINT", "STORY"],
            Extras::Permitted,
            Source::Yaml,
        )),
        "brand/tokens.json" => Some(row(
            "tokens",
            "designing-interfaces",
            "design",
            "devforgeai/tokens/1",
            Some("^TOKEN-[0-9]{3}$"),
            &["draft", "approved"],
            &["TOKEN"],
            Extras::Permitted,
            Source::Json,
        )),
        "brand/brand-kit.md" => Some(row(
            "brand-kit",
            "designing-interfaces",
            "design",
            "devforgeai/brand-kit/1",
            Some("^TOKEN-[0-9]{3}$"),
            &["draft", "approved"],
            &["TOKEN"],
            Extras::Refused,
            Source::Markdown,
        )),
        _ => {
            if let Some(stem) = rel
                .strip_prefix("context/")
                .and_then(|f| f.strip_suffix(".md"))
            {
                if CONTEXT_STEMS.contains(&stem) {
                    let mut r = row(
                        "context",
                        "establishing-context",
                        "constitute",
                        &format!("devforgeai/context-{stem}/1"),
                        None,
                        &["draft", "accepted"],
                        &["CON", "AP"],
                        Extras::Refused,
                        Source::Markdown,
                    );
                    r.id_literal = Some(stem.to_string());
                    return Some(r);
                }
                return None;
            }
            if let Some(stem) = rel.strip_prefix("adr/").and_then(|f| f.strip_suffix(".md")) {
                if stem.starts_with("ADR-") && ids::is_id(stem) {
                    return Some(row(
                        "adr",
                        "establishing-context",
                        "constitute",
                        "devforgeai/adr/1",
                        Some("^ADR-[0-9]{3}$"),
                        &["proposed", "accepted", "rejected", "superseded"],
                        &["ADR", "CON", "REQ"],
                        Extras::Refused,
                        Source::Markdown,
                    ));
                }
                return None;
            }
            if let Some(stem) = rel
                .strip_prefix("stories/")
                .and_then(|f| f.strip_suffix(".md"))
            {
                if stem.starts_with("STORY-") && ids::is_id(stem) {
                    return Some(row(
                        "story",
                        "planning-work",
                        "plan",
                        "devforgeai/story/1",
                        Some("^STORY-[0-9]{3}$"),
                        &["draft", "ready", "building", "built", "released"],
                        &["STORY", "AC"],
                        Extras::Refused,
                        Source::Markdown,
                    ));
                }
                return None;
            }
            if let Some(stem) = rel
                .strip_prefix("ui-specs/")
                .and_then(|f| f.strip_suffix(".md"))
            {
                if stem.starts_with("UI-") && ids::is_id(stem) {
                    return Some(row(
                        "ui-spec",
                        "designing-interfaces",
                        "design",
                        "devforgeai/ui-spec/1",
                        Some("^UI-[0-9]{3}$"),
                        &["draft", "approved"],
                        &["UI"],
                        Extras::Refused,
                        Source::Markdown,
                    ));
                }
                return None;
            }
            if let Some(stem) = rel
                .strip_prefix("releases/")
                .and_then(|f| f.strip_suffix(".yaml"))
            {
                if is_version(stem) {
                    return Some(row(
                        "release",
                        "releasing-software",
                        "release",
                        "devforgeai/release/1",
                        Some(r"^v[0-9]+\.[0-9]+\.[0-9]+$"),
                        &["draft", "released"],
                        &[],
                        Extras::Permitted,
                        Source::Yaml,
                    ));
                }
                return None;
            }
            if rel.starts_with("explore/") && rel.ends_with(".json") {
                return Some(row(
                    "explore-payload",
                    "exploring-ideas",
                    "explore",
                    "^devforgeai/[a-z-]+/1$",
                    Some("^IDEA-[0-9]{3}$"),
                    &[],
                    &["IDEA", "FLOW"],
                    Extras::Permitted,
                    Source::Json,
                ));
            }
            None
        }
    }
}

/// True when `s` is `vX.Y.Z`.
pub fn is_version(s: &str) -> bool {
    let Some(rest) = s.strip_prefix('v') else {
        return false;
    };
    let parts: Vec<&str> = rest.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

/// The `vX.Y.Z` triple, for the `release` ordering rule.
pub fn version_triple(s: &str) -> Option<(u64, u64, u64)> {
    let rest = s.strip_prefix('v')?;
    let mut it = rest.split('.');
    let a = it.next()?.parse().ok()?;
    let b = it.next()?.parse().ok()?;
    let c = it.next()?.parse().ok()?;
    if it.next().is_some() {
        return None;
    }
    Some((a, b, c))
}

/// True when `s` is `YYYY-MM-DD`.
pub fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b[..4].iter().all(u8::is_ascii_digit)
        && b[5..7].iter().all(u8::is_ascii_digit)
        && b[8..].iter().all(u8::is_ascii_digit)
}

/// One validated document.
#[derive(Debug, Clone)]
pub struct Document {
    /// The path, project-relative in forward-slash form.
    pub path: String,
    /// The doc type row.
    pub doc_type: String,
    /// The frontmatter `id`, or `""`.
    pub id: String,
    /// The frontmatter `status`, or `""`.
    pub status: String,
    /// The errors found.
    pub errors: Vec<Diag>,
    /// The warnings found.
    pub warnings: Vec<Diag>,
}

impl Document {
    /// True when no error was found.
    pub fn valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// What `validate` needs beyond the file itself.
pub struct Ctx<'a> {
    /// The project root.
    pub root: &'a Path,
    /// The ID index over `.devforgeai/`.
    pub index: &'a IdIndex,
    /// Skip heading order and cross-references, for `--producer-check`.
    pub frontmatter_only: bool,
    /// The document body, when a write has not landed yet.
    pub stdin_content: Option<&'a str>,
}

/// Validate one document.
pub fn validate(path: &Path, ctx: &Ctx) -> Result<Document, CliError> {
    let rel = crate::project::rel_display(ctx.root, path);
    let Some(row) = doc_type_for(&rel) else {
        return Err(CliError::new(
            "DFA-E010",
            format!("{rel} matches no doc-type row"),
        ));
    };

    let text = match ctx.stdin_content {
        Some(s) => s.to_string(),
        None => crate::project::read_doc(path)?,
    };

    let mut doc = Document {
        path: rel.clone(),
        doc_type: row.name.to_string(),
        id: String::new(),
        status: String::new(),
        errors: Vec::new(),
        warnings: Vec::new(),
    };

    let fm = match frontmatter::parse(&text, row.source, &rel) {
        Ok(fm) => fm,
        Err(d) => {
            doc.errors.push(d);
            return Ok(doc);
        }
    };

    doc.id = fm.str_key("id").unwrap_or("").to_string();
    doc.status = fm.str_key("status").unwrap_or("").to_string();

    check_keys(&fm, &row, &rel, &mut doc);
    check_values(&fm, &row, &rel, ctx, &mut doc);

    if !ctx.frontmatter_only {
        check_uniqueness(&fm, &rel, ctx, &mut doc);
        check_references(&fm, &text, &rel, ctx, &mut doc);
    }

    Ok(doc)
}

fn check_keys(fm: &Frontmatter, row: &DocType, rel: &str, doc: &mut Document) {
    // The seven keys are checked in every row, first and in order.
    for (want, key) in KEYS.iter().enumerate() {
        match fm.order.iter().position(|k| k == key) {
            None => doc.errors.push(Diag::at_line(
                "DFA-E203",
                format!("{rel} frontmatter has no key '{key}'"),
                rel,
                fm.start_line,
            )),
            Some(found) if found != want => doc.errors.push(Diag::at_line(
                "DFA-E205",
                format!(
                    "{rel} frontmatter key '{key}' is at position {}; conventions section 5 places it at {}",
                    found + 1,
                    want + 1
                ),
                rel,
                fm.line_of(key),
            )),
            Some(_) => {}
        }
    }

    if row.extras == Extras::Refused {
        for key in &fm.order {
            if !KEYS.contains(&key.as_str()) {
                doc.errors.push(Diag::at_line(
                    "DFA-E204",
                    format!(
                        "{rel} frontmatter has key '{key}', which conventions section 5 does not define"
                    ),
                    rel,
                    fm.line_of(key),
                ));
            }
        }
    }
}

fn check_values(fm: &Frontmatter, row: &DocType, rel: &str, ctx: &Ctx, doc: &mut Document) {
    // schema
    if let Some(schema) = fm.str_key("schema") {
        let ok = if row.schema_is_pattern {
            Pattern::compile(&row.schema)
                .map(|p| p.is_match(schema))
                .unwrap_or(false)
        } else {
            schema == row.schema
        };
        if !ok {
            doc.errors.push(Diag::at_line(
                "DFA-E207",
                format!(
                    "{rel} schema is '{schema}'; this document type is '{}'",
                    row.schema
                ),
                rel,
                fm.line_of("schema"),
            ));
        }
    } else if fm.value("schema").is_some() {
        doc.errors.push(type_error(fm, rel, "schema", "a string"));
    }

    // id
    if let Some(id) = fm.str_key("id") {
        if let Some(literal) = &row.id_literal {
            if id != literal {
                doc.errors.push(Diag::at_line(
                    "DFA-E209",
                    format!("{rel} id '{id}' is malformed"),
                    rel,
                    fm.line_of("id"),
                ));
            }
        } else if let Some(pat) = &row.id_pattern {
            let ok = Pattern::compile(pat)
                .map(|p| p.is_match(id))
                .unwrap_or(false);
            if !ok {
                let code = if row.name == "release" {
                    "DFA-E217"
                } else {
                    "DFA-E209"
                };
                let message = if row.name == "release" {
                    format!("{rel} id '{id}' is not vX.Y.Z")
                } else {
                    format!("{rel} id '{id}' is malformed")
                };
                doc.errors
                    .push(Diag::at_line(code, message, rel, fm.line_of("id")));
            } else if ids::is_id(id) {
                // Where the grammar is an ID form, its prefix is in the row's list.
                if let Some((prefix, _)) = ids::split_id(id) {
                    if !row.prefixes.is_empty() && !row.prefixes.contains(&prefix) {
                        doc.errors.push(Diag::at_line(
                            "DFA-E209",
                            format!("{rel} id '{id}' is malformed"),
                            rel,
                            fm.line_of("id"),
                        ));
                    }
                }
            }
        }
        // A release version is greater than the highest existing one.
        if row.name == "release" && is_version(id) {
            if let Some(prev) = highest_other_release(ctx.root, id) {
                if version_triple(id) <= version_triple(&prev) {
                    doc.errors.push(Diag::at_line(
                        "DFA-E218",
                        format!("{rel} version {id} is not greater than {prev}"),
                        rel,
                        fm.line_of("id"),
                    ));
                }
            }
        }
    } else if fm.value("id").is_some() {
        doc.errors.push(type_error(fm, rel, "id", "a string"));
    }

    // phase
    if let Some(phase) = fm.str_key("phase") {
        if row.name != "reflect-report" && phase != row.phase {
            doc.errors.push(Diag::at_line(
                "DFA-E206",
                format!(
                    "{rel} frontmatter key 'phase' is '{phase}'; expected '{}'",
                    row.phase
                ),
                rel,
                fm.line_of("phase"),
            ));
        }
    } else if fm.value("phase").is_some() {
        doc.errors.push(type_error(fm, rel, "phase", "a string"));
    }

    // status
    if let Some(status) = fm.str_key("status") {
        let ok = if row.status.is_empty() {
            !status.is_empty()
        } else {
            row.status.contains(&status)
        };
        if !ok {
            doc.errors.push(Diag::at_line(
                "DFA-E208",
                format!(
                    "{rel} status is '{status}'; {} allows {}",
                    row.name,
                    if row.status.is_empty() {
                        "any non-empty string".to_string()
                    } else {
                        row.status.join(", ")
                    }
                ),
                rel,
                fm.line_of("status"),
            ));
        }
    } else if fm.value("status").is_some() {
        doc.errors.push(type_error(fm, rel, "status", "a string"));
    }

    // produced_by
    if let Some(p) = fm.str_key("produced_by") {
        let shape = Pattern::compile("^[a-z][a-z0-9-]*$")
            .map(|x| x.is_match(p))
            .unwrap_or(false);
        if !shape || p != row.producer {
            doc.errors.push(Diag::at_line(
                "DFA-E211",
                format!(
                    "{rel} produced_by is '{p}'; {} is produced by '{}'",
                    row.name, row.producer
                ),
                rel,
                fm.line_of("produced_by"),
            ));
        }
    } else if fm.value("produced_by").is_some() {
        doc.errors
            .push(type_error(fm, rel, "produced_by", "a string"));
    }

    // consumes and open_questions are arrays of strings.
    for key in ["consumes", "open_questions"] {
        if let Some(v) = fm.value(key) {
            if fm.seq_key(key).is_none() {
                doc.errors.push(Diag::at_line(
                    "DFA-E206",
                    format!(
                        "{rel} frontmatter key '{key}' is {}; expected an array of strings",
                        frontmatter::type_name(v)
                    ),
                    rel,
                    fm.line_of(key),
                ));
            }
        }
    }

    // Each consumes entry matches the ID form.
    if let Some(list) = fm.seq_key("consumes") {
        for entry in list {
            if !ids::is_id(&entry) && !is_version(&entry) {
                doc.errors.push(Diag::at_line(
                    "DFA-E206",
                    format!(
                        "{rel} frontmatter key 'consumes' holds '{entry}'; expected an array of strings"
                    ),
                    rel,
                    fm.line_of("consumes"),
                ));
            }
        }
    }
}

fn type_error(fm: &Frontmatter, rel: &str, key: &str, expected: &str) -> Diag {
    let actual = fm
        .value(key)
        .map(frontmatter::type_name)
        .unwrap_or("absent");
    Diag::at_line(
        "DFA-E206",
        format!("{rel} frontmatter key '{key}' is {actual}; expected {expected}"),
        rel,
        fm.line_of(key),
    )
}

fn check_uniqueness(fm: &Frontmatter, rel: &str, ctx: &Ctx, doc: &mut Document) {
    let Some(id) = fm.str_key("id") else {
        return;
    };
    let schema = fm.str_key("schema").unwrap_or("");
    let Some(sites) = ctx.index.definitions.get(id) else {
        return;
    };
    // ID uniqueness is scoped per `schema` value.
    let same_schema: Vec<&ids::Site> = sites
        .iter()
        .filter(|s| s.schema == schema && s.path != rel)
        .collect();
    if let Some(other) = same_schema.first() {
        doc.errors.push(Diag::at_line(
            "DFA-E209",
            format!("{rel} id '{id}' duplicates {}", other.path),
            rel,
            fm.line_of("id"),
        ));
    }
}

fn check_references(fm: &Frontmatter, text: &str, rel: &str, ctx: &Ctx, doc: &mut Document) {
    let own_id = fm.str_key("id").unwrap_or("").to_string();
    let consumes = fm.seq_key("consumes").unwrap_or_default();

    // Body references: every ID occurrence below the frontmatter block that the
    // index did not record as a definition in this file.
    let mut body_refs: Vec<(String, u32)> = Vec::new();
    let mut local = IdIndex::default();
    ids::index_text(
        text,
        rel,
        fm.str_key("schema").unwrap_or(""),
        Some(&own_id),
        fm.body_line,
        &mut local,
    );
    for (id, sites) in &local.references {
        for s in sites {
            if s.line >= fm.body_line {
                body_refs.push((id.clone(), s.line));
            }
        }
    }
    body_refs.sort_by_key(|(_, l)| *l);
    let own_definitions: Vec<String> = local.definitions.keys().cloned().collect();

    let diags = xref::resolve(
        &xref::Subject {
            path: rel,
            own_id: &own_id,
            consumes: &consumes,
            body_refs: &body_refs,
            own_definitions: &own_definitions,
        },
        ctx.index,
    );
    for d in diags {
        if d.is_warning() {
            doc.warnings.push(d);
        } else {
            doc.errors.push(d);
        }
    }
}

fn highest_other_release(root: &Path, self_id: &str) -> Option<String> {
    let dir = crate::project::dot(root).join("releases");
    let entries = std::fs::read_dir(dir).ok()?;
    let mut best: Option<(u64, u64, u64)> = None;
    let mut best_name = String::new();
    for e in entries.filter_map(Result::ok) {
        let name = e.file_name().to_string_lossy().into_owned();
        let Some(stem) = name.strip_suffix(".yaml") else {
            continue;
        };
        if stem == self_id || !is_version(stem) {
            continue;
        }
        if let Some(t) = version_triple(stem) {
            if best.is_none_or(|b| t > b) {
                best = Some(t);
                best_name = stem.to_string();
            }
        }
    }
    best.map(|_| best_name)
}

/// Every path under `.devforgeai/` that matches a doc-type row.
pub fn all_documents(root: &Path) -> Vec<PathBuf> {
    ids::document_paths(root)
        .into_iter()
        .filter(|p| doc_type_for(&crate::project::rel_display(root, p)).is_some())
        .collect()
}

/// The producer check: the current phase maps to a producer skill through the
/// doc-type table, and that skill is compared to the doc type of the path.
pub fn producer_allows(current_phase: &str, row: &DocType) -> bool {
    // The design and reflect doc types are allowed from every phase, since both
    // skills are cross-cutting.
    if row.phase == "design" || row.phase == "reflect" {
        return true;
    }
    // A document whose producer is the binary is written by `report::write`,
    // which does not pass through this check. No phase maps to that producer,
    // so a skill cannot hand-write a build report or a gate report and have it
    // read back as evidence.
    if row.producer == "devforgeai-cli" {
        return false;
    }
    // The comparison is between producer skills, not between phases: the two
    // coincide for most rows and part company exactly where a row's producer is
    // not the skill of the row's phase. A phase outside the enum has no
    // producer and therefore allows nothing.
    match producer_of_phase_opt(current_phase) {
        Some(expected) => row.producer == expected,
        None => false,
    }
}

/// The project-relative path of the explore decision document.
pub const DECISION: &str = "explore/decision.yaml";

/// Read `explore/decision.yaml`, keeping absent and unparsable apart.
///
/// `Ok(None)` means the file is not there; `Err` means it is there and does not
/// parse. Collapsing the two — `.ok().and_then(…)` over the read — is what let
/// a malformed decision file read as "no explore phase happened", so the gate
/// skipped its own predecessor while `explore prune` refused the same file.
/// One implementation keeps the two commands from disagreeing about it.
pub fn read_decision(root: &Path) -> Result<Option<serde_yaml_ng::Value>, CliError> {
    let path = crate::project::dot(root).join(DECISION);
    if !path.exists() {
        return Ok(None);
    }
    let text = crate::project::read_doc(&path)?;
    let value: serde_yaml_ng::Value = serde_yaml_ng::from_str(&text).map_err(|e| {
        CliError::at(
            "DFA-E401",
            format!("{DECISION} is not valid YAML: {e}"),
            DECISION.to_string(),
        )
    })?;
    Ok(Some(value))
}

/// The producer skill of the phase, for the `DFA-E212` message.
///
/// A phase outside the enum has no producer, and answering `""` for one made
/// every comparison against it quietly false. `None` is the answer a caller has
/// to handle, so an unrecognised phase surfaces as a coded refusal rather than
/// as a rule that silently never matches.
pub fn producer_of_phase_opt(phase: &str) -> Option<&'static str> {
    Some(match phase {
        "explore" => "exploring-ideas",
        "discover" => "discovering-requirements",
        "constitute" => "establishing-context",
        "plan" => "planning-work",
        "build" => "implementing-stories",
        "verify" => "validating-quality",
        "release" => "releasing-software",
        "design" => "designing-interfaces",
        "reflect" => "improving-framework",
        _ => return None,
    })
}

/// The producer skill of the phase, or `""` for a phase outside the enum.
///
/// Kept for the message-rendering call sites, which want a string and have
/// already refused an unknown phase. A decision must use
/// [`producer_of_phase_opt`].
pub fn producer_of_phase(phase: &str) -> &'static str {
    producer_of_phase_opt(phase).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_doc_type_table_maps_every_row_by_path() {
        let cases: &[(&str, &str)] = &[
            ("explore/brief.md", "explore-brief"),
            ("explore/decision.yaml", "explore-decision"),
            ("explore/scan.json", "explore-payload"),
            ("requirements.yaml", "requirements"),
            ("context/tech-stack.md", "context"),
            ("context/anti-patterns.md", "context"),
            ("adr/ADR-004.md", "adr"),
            ("stories/STORY-014.md", "story"),
            ("stories/sprint.yaml", "sprint"),
            ("ui-specs/UI-002.md", "ui-spec"),
            ("brand/tokens.json", "tokens"),
            ("brand/brand-kit.md", "brand-kit"),
            ("reports/STORY-014-build.yaml", "build-report"),
            ("reports/STORY-014-qa.yaml", "qa-report"),
            ("reports/STORY-014-verify.yaml", "gate-report"),
            ("reports/IDEA-003-explore.yaml", "gate-report"),
            ("releases/v0.3.0.yaml", "release"),
            ("reports/reflect-2026-09-10.yaml", "reflect-report"),
        ];
        for (path, want) in cases {
            let row = doc_type_for(path).unwrap_or_else(|| panic!("{path} matches a row"));
            assert_eq!(&row.name, want, "{path}");
        }
    }

    #[test]
    fn a_path_matching_no_row_is_skipped() {
        assert!(doc_type_for("scratch.md").is_none());
        assert!(doc_type_for("context/not-a-context-file.md").is_none());
        assert!(doc_type_for("stories/NOTSTORY.md").is_none());
        assert!(doc_type_for("releases/latest.yaml").is_none());
    }

    #[test]
    fn the_devforgeai_prefix_is_optional() {
        assert_eq!(
            doc_type_for(".devforgeai/stories/STORY-014.md")
                .expect("row")
                .name,
            "story"
        );
    }

    #[test]
    fn the_reports_rows_are_ordered_most_specific_first() {
        // `-build` and `-qa` win over the general gate-report row.
        assert_eq!(
            doc_type_for("reports/STORY-014-build.yaml")
                .expect("row")
                .name,
            "build-report"
        );
        assert_eq!(
            doc_type_for("reports/STORY-014-qa.yaml").expect("row").name,
            "qa-report"
        );
        // And the reflect document is distinct from the reflect gate report.
        assert_eq!(
            doc_type_for("reports/reflect-2026-09-10.yaml")
                .expect("row")
                .name,
            "reflect-report"
        );
    }

    #[test]
    fn extras_are_refused_by_nine_rows_and_permitted_by_seven() {
        let refused = [
            "explore/brief.md",
            "context/tech-stack.md",
            "adr/ADR-001.md",
            "stories/STORY-001.md",
            "ui-specs/UI-001.md",
            "brand/brand-kit.md",
        ];
        for p in refused {
            assert_eq!(doc_type_for(p).expect("row").extras, Extras::Refused, "{p}");
        }
        let permitted = [
            "explore/decision.yaml",
            "requirements.yaml",
            "stories/sprint.yaml",
            "brand/tokens.json",
            "reports/STORY-001-build.yaml",
            "releases/v1.0.0.yaml",
        ];
        for p in permitted {
            assert_eq!(
                doc_type_for(p).expect("row").extras,
                Extras::Permitted,
                "{p}"
            );
        }
    }

    #[test]
    fn the_context_row_takes_its_schema_and_id_from_the_stem() {
        let row = doc_type_for("context/source-tree.md").expect("row");
        assert_eq!(row.schema, "devforgeai/context-source-tree/1");
        assert_eq!(row.id_literal.as_deref(), Some("source-tree"));
    }

    #[test]
    fn the_explore_payload_schema_is_a_pattern() {
        let row = doc_type_for("explore/scan.json").expect("row");
        assert!(row.schema_is_pattern);
        let p = Pattern::compile(&row.schema).expect("compiles");
        assert!(p.is_match("devforgeai/landscape-scan/1"));
        assert!(!p.is_match("devforgeai/Scan/1"));
    }

    #[test]
    fn version_and_date_shapes() {
        assert!(is_version("v0.3.0"));
        assert!(!is_version("0.3.0"));
        assert!(!is_version("v0.3"));
        assert_eq!(version_triple("v1.2.3"), Some((1, 2, 3)));
        assert!(is_date("2026-09-10"));
        assert!(!is_date("2026-9-10"));
    }

    #[test]
    fn producer_check_allows_design_and_reflect_from_any_phase() {
        let ui = doc_type_for("ui-specs/UI-001.md").expect("row");
        assert!(producer_allows("build", &ui), "design is cross-cutting");
        let reflect = doc_type_for("reports/reflect-2026-09-10.yaml").expect("row");
        assert!(
            producer_allows("build", &reflect),
            "reflect is cross-cutting"
        );

        let story = doc_type_for("stories/STORY-014.md").expect("row");
        assert!(producer_allows("plan", &story));
        assert!(
            !producer_allows("build", &story),
            "Plan owns the story document"
        );
    }

    #[test]
    fn the_story_status_enum_has_five_values_without_verified() {
        let row = doc_type_for("stories/STORY-014.md").expect("row");
        assert_eq!(
            row.status,
            vec!["draft", "ready", "building", "built", "released"]
        );
        assert!(!row.status.contains(&"verified"));
    }

    #[test]
    fn producer_names_follow_the_skill_naming_decision() {
        assert_eq!(producer_of_phase("build"), "implementing-stories");
        assert_eq!(producer_of_phase("plan"), "planning-work");
        assert_eq!(producer_of_phase("reflect"), "improving-framework");
    }
}
