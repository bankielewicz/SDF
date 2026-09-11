//! `.devforgeai/gates.toml`: the gate definitions, the closed enum of thirty
//! check kinds, the four condition keys, and the compiled minimums.

use crate::errors::CliError;
use crate::project;
use std::collections::BTreeSet;
use std::path::Path;

/// The fixed `schema` value.
pub const SCHEMA: &str = "devforgeai/gates/1";

/// The default `gates.toml` `init` writes, verbatim from the spec's `## Gate`.
pub const DEFAULT_GATES: &str = include_str!("../templates/gates.default.toml");

/// The closed enum of thirty check kinds, as names.
pub const CHECK_KINDS: &[&str] = &[
    "doc_valid",
    "ids_resolve",
    "file_exists",
    "fields_present",
    "field_in_enum",
    "field_is_date",
    "length_between",
    "no_open_questions",
    "set_cover",
    "row_count_between",
    "column_matches",
    "column_contains_all",
    "elapsed_days_at_most",
    "context_audit",
    "story_valid",
    "tests_pass",
    "coverage_min",
    "lint_clean",
    "verifier_pass",
    "report_metric",
    "design_tokens",
    "no_cycle",
    "complexity_clean",
    "antipattern_clean",
    "files_declared",
    "release_stories",
    "deploy_manifest",
    "docs_cover",
    "yaml_cites",
    "no_threshold_decrease",
];

/// The keys every check accepts, whatever its kind.
const COMMON_KEYS: &[&str] = &[
    "kind",
    "id",
    "severity",
    "on_fail",
    "message",
    "skip_when",
    "required_when",
    "null_when",
    "empty_when",
];

/// The closed key set of a condition inline table.
const CONDITION_KEYS: &[&str] = &[
    "path",
    "field",
    "equals",
    "in",
    "state_field",
    "is_empty",
    "doc_exists",
];

/// The kind-specific keys of each check kind, from the `## Outputs` table.
fn kind_keys(kind: &str) -> Option<&'static [&'static str]> {
    Some(match kind {
        "doc_valid" => &["docs"],
        "ids_resolve" => &["prefixes", "from", "to"],
        "file_exists" => &["paths", "min_count", "absent"],
        // `field` names the location inside `path`; without it the same key
        // meant a document on one gate and a field on another.
        "fields_present" => &["collection", "path", "field", "fields"],
        "field_in_enum" => &["path", "field", "values"],
        "field_is_date" => &["path", "field", "format", "after_field"],
        "length_between" => &["path", "field", "min", "max", "exclude_status"],
        "no_open_questions" => &["docs"],
        // `status_field` names the key `exclude_status` reads, so a universe
        // whose leaf is not `id` keeps its exclusions.
        "set_cover" => &["cover", "universe", "exclude_status", "status_field"],
        "row_count_between" => &["path", "section", "min", "max"],
        "column_matches" => &["path", "section", "column", "pattern", "unique"],
        "column_contains_all" => &["path", "section", "column", "state_field"],
        "elapsed_days_at_most" => &[
            "started_field",
            "limit_field",
            "remedy_started_field",
            "remedy_limit_field",
        ],
        "context_audit" => &["allow_warn"],
        "story_valid" => &["scope"],
        "tests_pass" => &["stacks", "allow_empty"],
        "coverage_min" => &["layers", "overall", "source"],
        "lint_clean" => &["stacks"],
        "verifier_pass" => &["verifiers", "min_ratio"],
        "report_metric" => &["metric", "op", "value"],
        "design_tokens" => &["paths"],
        "no_cycle" => &["docs", "root", "from", "to", "prefix"],
        "complexity_clean" => &["stacks"],
        "antipattern_clean" => &["min_severity", "scope"],
        "files_declared" => &["base"],
        "release_stories" => &["path", "require_status", "require_report", "require_result"],
        "deploy_manifest" => &["path", "platform"],
        "docs_cover" => &["path", "min_ratio", "command"],
        "yaml_cites" => &["doc", "from", "field", "into", "min"],
        "no_threshold_decrease" => &[
            "doc",
            "from",
            "target_field",
            "key_field",
            "value_field",
            "files",
        ],
        _ => return None,
    })
}

/// The keys the spec's kind table marks required.
///
/// A required key read through `str_key` or `arr_key` with a silent default
/// made the check pass vacuously while still satisfying the compiled minimum
/// for its phase, so the absence is caught here, at load, and named.
///
/// `doc_valid` and `no_open_questions` are absent from this table because spec
/// 249 and 256 give `docs` a default of `[]`; each raises at evaluation when
/// its pattern list expands to nothing.
fn required_keys(kind: &str) -> &'static [&'static str] {
    match kind {
        "file_exists" => &["paths"],
        "field_in_enum" => &["path", "field", "values"],
        "field_is_date" => &["path", "field"],
        "length_between" => &["path", "field"],
        "set_cover" => &["cover", "universe"],
        "row_count_between" => &["path", "section"],
        "column_matches" => &["path", "section", "column", "pattern"],
        "column_contains_all" => &["path", "section", "column", "state_field"],
        "elapsed_days_at_most" => &["started_field", "limit_field"],
        "verifier_pass" => &["verifiers"],
        "report_metric" => &["metric"],
        "no_cycle" => &["docs", "from", "to"],
        "yaml_cites" => &["doc", "from", "field", "into"],
        "no_threshold_decrease" => &["doc"],
        _ => &[],
    }
}

/// The permitted values of a key the spec's kind table gives an enum.
fn enum_values(kind: &str, key: &str) -> Option<&'static [&'static str]> {
    Some(match (kind, key) {
        ("story_valid", "scope") => &["active", "sprint", "all"],
        ("antipattern_clean", "scope") => &["story", "project"],
        ("antipattern_clean", "min_severity") => &["blocker", "high", "medium", "low"],
        ("coverage_min", "source") => &["run", "read"],
        ("report_metric", "op") => &["eq", "ne", "lt", "lte", "gt", "gte"],
        _ => return None,
    })
}

/// The required check kinds per phase, compiled into the binary. A required
/// kind present with `severity = "warn"` is treated as absent.
pub fn required_kinds(phase: &str) -> &'static [&'static str] {
    match phase {
        "explore" => &["file_exists", "field_in_enum"],
        "discover" => &["fields_present", "ids_resolve"],
        "constitute" => &["context_audit"],
        "plan" => &["doc_valid", "story_valid", "ids_resolve"],
        "build" => &["doc_valid", "tests_pass", "coverage_min"],
        "verify" => &["doc_valid", "verifier_pass"],
        "release" => &["doc_valid", "file_exists"],
        "reflect" => &[
            "file_exists",
            "doc_valid",
            "yaml_cites",
            "no_threshold_decrease",
        ],
        _ => &[],
    }
}

/// The compiled floor for `verifier_pass.min_ratio`. The compiled-minimums
/// table puts this in the verify row alone, and the default file the binary
/// writes sets explore's `kill-case-answered` to `0.0`, so the floor binds the
/// verify gate and no other. `parse` refuses a phase outside the enum, so the
/// `None` arm names a phase with no floor rather than a phase that escaped the
/// table.
pub fn min_ratio_floor(phase: &str) -> Option<f64> {
    match phase {
        "verify" => Some(1.0),
        _ => None,
    }
}

/// One condition inline table.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Condition {
    /// The document the condition reads.
    pub path: Option<String>,
    /// The field inside it.
    pub field: Option<String>,
    /// The value the field equals.
    pub equals: Option<String>,
    /// The set the field is a member of.
    pub r#in: Option<Vec<String>>,
    /// A dotted path into `state.toml`.
    pub state_field: Option<String>,
    /// True when the named list is expected to be empty.
    pub is_empty: Option<bool>,
    /// A document whose existence is the condition.
    pub doc_exists: Option<String>,
}

/// One `[[gate.check]]`.
#[derive(Debug, Clone)]
pub struct Check {
    /// The check kind, a member of `CHECK_KINDS`.
    pub kind: String,
    /// Unique within the gate.
    pub id: String,
    /// `block` or `warn`.
    pub severity: String,
    /// `fail` or `send_back`; defaults to the gate's.
    pub on_fail: String,
    /// Printed in the report reason and the handoff evidence.
    pub message: String,
    /// Recorded `skip` with `reason: condition` when true.
    pub skip_when: Option<Condition>,
    /// Recorded `skip` with `reason: not_required` when absent or false.
    pub required_when: Option<Condition>,
    /// The named field is expected to hold null or be absent.
    pub null_when: Option<Condition>,
    /// The named field is expected to hold an empty list.
    pub empty_when: Option<Condition>,
    /// The kind-specific keys, kept as parsed so each kind reads its own.
    pub keys: toml::value::Table,
}

impl Check {
    /// A string key, or `default`.
    pub fn str_key(&self, key: &str, default: &str) -> String {
        self.keys
            .get(key)
            .and_then(toml::Value::as_str)
            .unwrap_or(default)
            .to_string()
    }

    /// A string-array key, or `default`.
    pub fn arr_key(&self, key: &str, default: &[&str]) -> Vec<String> {
        match self.keys.get(key).and_then(toml::Value::as_array) {
            Some(a) => a
                .iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_string)
                .collect(),
            None => default.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// An integer key, or `default`.
    pub fn int_key(&self, key: &str, default: i64) -> i64 {
        self.keys
            .get(key)
            .and_then(toml::Value::as_integer)
            .unwrap_or(default)
    }

    /// A float key, accepting an integer literal, or `default`.
    pub fn float_key(&self, key: &str, default: f64) -> f64 {
        match self.keys.get(key) {
            Some(toml::Value::Float(f)) => *f,
            Some(toml::Value::Integer(i)) => *i as f64,
            _ => default,
        }
    }

    /// A boolean key, or `default`.
    pub fn bool_key(&self, key: &str, default: bool) -> bool {
        self.keys
            .get(key)
            .and_then(toml::Value::as_bool)
            .unwrap_or(default)
    }

    /// True when the key is present in the file.
    pub fn has(&self, key: &str) -> bool {
        self.keys.contains_key(key)
    }
}

/// One `[[gate]]`.
#[derive(Debug, Clone)]
pub struct Gate {
    /// The phase this gate belongs to; unique across gates.
    pub phase: String,
    /// The predecessor `gate require` tests, or `""`.
    pub requires: String,
    /// `fail` or `send_back`.
    pub on_fail: String,
    /// The phase a send-back names.
    pub send_back_to: String,
    /// The default `path` of every check in this gate.
    pub document: String,
    /// Human text.
    pub description: String,
    /// The checks, in file order.
    pub check: Vec<Check>,
}

/// `.devforgeai/gates.toml`.
#[derive(Debug, Clone)]
pub struct Gates {
    /// Fixed: `devforgeai/gates/1`.
    pub schema: String,
    /// `gate check` exits 1 below it.
    pub cli_min_version: String,
    /// The gates, in file order.
    pub gate: Vec<Gate>,
}

impl Gates {
    /// The gate for `phase`, or `DFA-E300`.
    pub fn gate_for(&self, phase: &str) -> Result<&Gate, CliError> {
        self.gate.iter().find(|g| g.phase == phase).ok_or_else(|| {
            CliError::at(
                "DFA-E300",
                format!("gates.toml has no gate for phase '{phase}'"),
                ".devforgeai/gates.toml",
            )
        })
    }
}

/// Read and validate `.devforgeai/gates.toml`.
pub fn load(root: &Path) -> Result<Gates, CliError> {
    let path = project::dot(root).join(project::GATES);
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(CliError::at(
                "DFA-E102",
                ".devforgeai/gates.toml not found; run 'devforgeai init'",
                ".devforgeai/gates.toml",
            ))
        }
        Err(e) => return Err(CliError::io("reading", path.display(), &e)),
    };
    let gates = parse(&text)?;
    gates.validate_minimums()?;
    Ok(gates)
}

/// Parse gates TOML text, enforcing the shape rules but not the minimums.
pub fn parse(text: &str) -> Result<Gates, CliError> {
    let raw: toml::Value = toml::from_str(text).map_err(|e| {
        CliError::at(
            "DFA-E105",
            format!("gates.toml is not valid TOML: {e}"),
            ".devforgeai/gates.toml",
        )
    })?;

    let schema = raw
        .get("schema")
        .and_then(toml::Value::as_str)
        .unwrap_or("")
        .to_string();
    if schema != SCHEMA {
        return Err(CliError::at(
            "DFA-E107",
            format!("gates.toml: schema '{schema}' is unsupported; this binary reads {SCHEMA}"),
            ".devforgeai/gates.toml",
        ));
    }

    let cli_min_version = raw
        .get("cli_min_version")
        .and_then(toml::Value::as_str)
        .unwrap_or("0.0.0")
        .to_string();
    if semver_gt(&cli_min_version, crate::VERSION) {
        return Err(CliError::at(
            "DFA-E304",
            format!(
                "gates.toml requires devforgeai {cli_min_version} or newer; this binary is {}",
                crate::VERSION
            ),
            ".devforgeai/gates.toml",
        ));
    }

    let empty = Vec::new();
    let raw_gates = raw
        .get("gate")
        .and_then(toml::Value::as_array)
        .unwrap_or(&empty);

    let mut seen_phases: BTreeSet<String> = BTreeSet::new();
    let mut gates = Vec::new();

    for g in raw_gates {
        let phase = g
            .get("phase")
            .and_then(toml::Value::as_str)
            .unwrap_or("")
            .to_string();
        // A phase outside the enum falls through every compiled table to an
        // empty slice, so it is required to carry no check kind and is bound by
        // no floor; refusing it at load is what makes those tables total.
        if !crate::state::GATE_PHASES.contains(&phase.as_str()) {
            return Err(CliError::at(
                "DFA-E302",
                format!(
                    "gates.toml gate has phase '{phase}'; the phases are {}",
                    crate::state::GATE_PHASES.join(", ")
                ),
                ".devforgeai/gates.toml",
            ));
        }
        if !seen_phases.insert(phase.clone()) {
            return Err(CliError::at(
                "DFA-E305",
                format!("gates.toml defines phase '{phase}' twice"),
                ".devforgeai/gates.toml",
            ));
        }
        let on_fail = g
            .get("on_fail")
            .and_then(toml::Value::as_str)
            .unwrap_or("fail")
            .to_string();
        let send_back_to = g
            .get("send_back_to")
            .and_then(toml::Value::as_str)
            .unwrap_or("")
            .to_string();
        if on_fail == "send_back" && !crate::state::GATE_PHASES.contains(&send_back_to.as_str()) {
            return Err(CliError::at(
                "DFA-E306",
                format!(
                    "gates.toml gate '{phase}' has on_fail = \"send_back\" and send_back_to = '{send_back_to}'"
                ),
                ".devforgeai/gates.toml",
            ));
        }

        let default_requires = crate::state::phase_index(&phase)
            .and_then(|i| i.checked_sub(1))
            .and_then(|i| crate::state::PHASES.get(i))
            .unwrap_or(&"")
            .to_string();
        let requires = g
            .get("requires")
            .and_then(toml::Value::as_str)
            .map(str::to_string)
            .unwrap_or(default_requires);

        let document = g
            .get("document")
            .and_then(toml::Value::as_str)
            .unwrap_or("")
            .to_string();

        let mut seen_ids: BTreeSet<String> = BTreeSet::new();
        let mut checks = Vec::new();
        let raw_checks = g
            .get("check")
            .and_then(toml::Value::as_array)
            .unwrap_or(&empty);

        for c in raw_checks {
            let table = c.as_table().ok_or_else(|| {
                CliError::at(
                    "DFA-E105",
                    format!("gates.toml gate '{phase}' has a check that is not a table"),
                    ".devforgeai/gates.toml",
                )
            })?;
            let check = parse_check(&phase, table, &on_fail, &document)?;
            if !seen_ids.insert(check.id.clone()) {
                return Err(CliError::at(
                    "DFA-E305",
                    format!(
                        "gates.toml defines check id '{}' twice in gate '{phase}'",
                        check.id
                    ),
                    ".devforgeai/gates.toml",
                ));
            }
            checks.push(check);
        }

        gates.push(Gate {
            phase,
            requires,
            on_fail,
            send_back_to,
            document,
            description: g
                .get("description")
                .and_then(toml::Value::as_str)
                .unwrap_or("")
                .to_string(),
            check: checks,
        });
    }

    Ok(Gates {
        schema,
        cli_min_version,
        gate: gates,
    })
}

fn parse_check(
    phase: &str,
    table: &toml::value::Table,
    gate_on_fail: &str,
    gate_document: &str,
) -> Result<Check, CliError> {
    let id = table
        .get("id")
        .and_then(toml::Value::as_str)
        .unwrap_or("")
        .to_string();
    let kind = table
        .get("kind")
        .and_then(toml::Value::as_str)
        .unwrap_or("")
        .to_string();

    let Some(allowed) = kind_keys(&kind) else {
        return Err(CliError::at(
            "DFA-E301",
            format!(
                "gates.toml gate '{phase}' check '{id}' has kind '{kind}'; the kinds are {}",
                CHECK_KINDS.join(", ")
            ),
            ".devforgeai/gates.toml",
        ));
    };

    let mut keys = toml::value::Table::new();
    for (key, value) in table {
        if COMMON_KEYS.contains(&key.as_str()) {
            continue;
        }
        if !allowed.contains(&key.as_str()) {
            return Err(CliError::at(
                "DFA-E302",
                format!(
                    "gates.toml gate '{phase}' check '{id}' has key '{key}', which kind '{kind}' does not define"
                ),
                ".devforgeai/gates.toml",
            ));
        }
        keys.insert(key.clone(), value.clone());
    }

    // A check's `path` defaults to the gate's `document`.
    if !gate_document.is_empty() && allowed.contains(&"path") && !keys.contains_key("path") {
        keys.insert(
            "path".to_string(),
            toml::Value::String(gate_document.to_string()),
        );
    }

    let e302 = |what: String| {
        CliError::at(
            "DFA-E302",
            format!("gates.toml gate '{phase}' check '{id}' {what}"),
            ".devforgeai/gates.toml",
        )
    };

    // Every key the spec's kind table marks required is present, after the
    // gate's `document` has supplied `path`.
    for key in required_keys(&kind) {
        if !keys.contains_key(*key) {
            return Err(e302(format!(
                "omits key '{key}', which kind '{kind}' requires"
            )));
        }
    }

    // Every enum-valued key holds a value of its enum.
    for (key, value) in &keys {
        let Some(permitted) = enum_values(&kind, key) else {
            continue;
        };
        let held = value.as_str().unwrap_or("");
        if !permitted.contains(&held) {
            return Err(e302(format!(
                "has key '{key}' = '{held}'; kind '{kind}' permits {}",
                permitted.join(", ")
            )));
        }
    }

    check_cross_keys(&kind, &keys, table, &e302)?;

    let (skip_when, required_when, null_when, empty_when) = (
        condition(phase, &id, table, "skip_when")?,
        condition(phase, &id, table, "required_when")?,
        condition(phase, &id, table, "null_when")?,
        condition(phase, &id, table, "empty_when")?,
    );

    Ok(Check {
        kind,
        id,
        severity: table
            .get("severity")
            .and_then(toml::Value::as_str)
            .unwrap_or("block")
            .to_string(),
        on_fail: table
            .get("on_fail")
            .and_then(toml::Value::as_str)
            .unwrap_or(gate_on_fail)
            .to_string(),
        message: table
            .get("message")
            .and_then(toml::Value::as_str)
            .unwrap_or("")
            .to_string(),
        skip_when,
        required_when,
        null_when,
        empty_when,
        keys,
    })
}

/// The rules that bind two keys of one check together, which the per-key checks
/// above cannot state.
fn check_cross_keys(
    kind: &str,
    keys: &toml::value::Table,
    table: &toml::value::Table,
    e302: &dyn Fn(String) -> CliError,
) -> Result<(), CliError> {
    match kind {
        "ids_resolve" => {
            // The two forms are exclusive and each is whole: `from` with no
            // `to` silently ran the index form instead, with no diagnostic.
            let from = keys.contains_key("from");
            let to = keys.contains_key("to");
            let prefixes = keys.contains_key("prefixes");
            if from != to {
                return Err(e302(
                    "names one of 'from' and 'to'; the pair form needs both".to_string(),
                ));
            }
            if from && prefixes {
                return Err(e302(
                    "names both the 'from'/'to' pair and 'prefixes'; the two forms are exclusive"
                        .to_string(),
                ));
            }
            if !from && !prefixes {
                return Err(e302(
                    "names neither the 'from'/'to' pair nor 'prefixes', so it resolves nothing"
                        .to_string(),
                ));
            }
        }
        "file_exists" => {
            let absent = keys
                .get("absent")
                .and_then(toml::Value::as_bool)
                .unwrap_or(false);
            match (
                absent,
                keys.get("min_count").and_then(toml::Value::as_integer),
            ) {
                // `min_count` carries no meaning beside `absent = true`.
                (true, Some(_)) => {
                    return Err(e302(
                        "carries 'min_count' with 'absent = true', where it means nothing"
                            .to_string(),
                    ))
                }
                // A `min_count` of zero passes whatever is on disk.
                (false, Some(n)) if n < 1 => {
                    return Err(e302(format!(
                        "sets 'min_count' to {n}, which asserts nothing about the filesystem"
                    )))
                }
                _ => {}
            }
        }
        "fields_present" => {
            let collection = keys
                .get("collection")
                .and_then(toml::Value::as_str)
                .unwrap_or("");
            let fields = keys
                .get("fields")
                .and_then(toml::Value::as_array)
                .map(|a| !a.is_empty())
                .unwrap_or(false);
            if collection.is_empty() && fields {
                return Err(e302(
                    "carries 'fields' with no 'collection', where the names are never read"
                        .to_string(),
                ));
            }
        }
        _ => {}
    }

    // `null_when` and `empty_when` are rules about the check's own `field`, so
    // a kind that defines no `field` key can carry neither.
    let has_field = kind_keys(kind)
        .map(|k| k.contains(&"field"))
        .unwrap_or(false);
    for key in ["null_when", "empty_when"] {
        if table.contains_key(key) && !has_field {
            return Err(e302(format!(
                "carries '{key}', which reads the check's own 'field'; kind '{kind}' defines none"
            )));
        }
    }
    Ok(())
}

fn condition(
    phase: &str,
    check_id: &str,
    table: &toml::value::Table,
    key: &str,
) -> Result<Option<Condition>, CliError> {
    let Some(v) = table.get(key) else {
        return Ok(None);
    };
    let t = v.as_table().ok_or_else(|| {
        CliError::at(
            "DFA-E302",
            format!("gates.toml gate '{phase}' check '{check_id}' key '{key}' is not a table"),
            ".devforgeai/gates.toml",
        )
    })?;
    for k in t.keys() {
        if !CONDITION_KEYS.contains(&k.as_str()) {
            return Err(CliError::at(
                "DFA-E302",
                format!(
                    "gates.toml gate '{phase}' check '{check_id}' has key '{k}', which kind '{key}' does not define"
                ),
                ".devforgeai/gates.toml",
            ));
        }
    }
    // A condition names exactly one subject and only the modifiers that
    // subject accepts. Without this the runtime branch order dropped every key
    // outside the branch it took, so a condition carrying `doc_exists` and
    // `equals` tested existence alone and one carrying `equals` and no subject
    // was always false.
    let bad = |what: String| {
        CliError::at(
            "DFA-E302",
            format!("gates.toml gate '{phase}' check '{check_id}' key '{key}' {what}"),
            ".devforgeai/gates.toml",
        )
    };
    let subjects: Vec<&str> = ["doc_exists", "state_field", "path"]
        .into_iter()
        .filter(|s| t.contains_key(*s))
        .collect();
    match subjects.as_slice() {
        [] => {
            return Err(bad(
                "names no subject; it needs one of doc_exists, state_field, or path".to_string(),
            ))
        }
        [_] => {}
        many => {
            return Err(bad(format!(
                "names {} subjects: {}; a condition reads one",
                many.len(),
                many.join(", ")
            )))
        }
    }
    if t.contains_key("path") && !t.contains_key("field") {
        return Err(bad(
            "names a path and no field to read inside it".to_string()
        ));
    }
    if t.contains_key("equals") && t.contains_key("in") {
        return Err(bad("names both equals and in".to_string()));
    }
    if t.contains_key("doc_exists") && (t.contains_key("equals") || t.contains_key("in")) {
        return Err(bad(
            "names doc_exists, which tests existence, beside a value test".to_string(),
        ));
    }
    if t.contains_key("is_empty") && !t.contains_key("state_field") {
        return Err(bad(
            "names is_empty without a state_field to test".to_string()
        ));
    }
    if t.contains_key("field") && !t.contains_key("path") {
        return Err(bad("names a field with no path to read it from".to_string()));
    }

    Ok(Some(Condition {
        path: t
            .get("path")
            .and_then(toml::Value::as_str)
            .map(str::to_string),
        field: t
            .get("field")
            .and_then(toml::Value::as_str)
            .map(str::to_string),
        equals: t
            .get("equals")
            .and_then(toml::Value::as_str)
            .map(str::to_string),
        r#in: t.get("in").and_then(toml::Value::as_array).map(|a| {
            a.iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_string)
                .collect()
        }),
        state_field: t
            .get("state_field")
            .and_then(toml::Value::as_str)
            .map(str::to_string),
        is_empty: t.get("is_empty").and_then(toml::Value::as_bool),
        doc_exists: t
            .get("doc_exists")
            .and_then(toml::Value::as_str)
            .map(str::to_string),
    }))
}

impl Gates {
    /// The compiled minimums: the required check kinds are present with
    /// `severity = "block"`, and `verifier_pass.min_ratio` is at or above the
    /// floor for the gate's phase.
    pub fn validate_minimums(&self) -> Result<(), CliError> {
        for gate in &self.gate {
            for required in required_kinds(&gate.phase) {
                let present = gate
                    .check
                    .iter()
                    .any(|c| &c.kind == required && c.severity == "block");
                if !present {
                    return Err(CliError::at(
                        "DFA-E303",
                        format!(
                            "gates.toml gate '{}' omits required check kind '{required}'",
                            gate.phase
                        ),
                        ".devforgeai/gates.toml",
                    ));
                }
            }
            if let Some(floor) = min_ratio_floor(&gate.phase) {
                for c in &gate.check {
                    if c.kind == "verifier_pass" {
                        let ratio = c.float_key("min_ratio", 1.0);
                        if ratio < floor {
                            return Err(CliError::at(
                                "DFA-E303",
                                format!(
                                    "gates.toml gate '{}' sets verifier_pass.min_ratio to {ratio}, below the compiled minimum {floor}",
                                    gate.phase
                                ),
                                ".devforgeai/gates.toml",
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// True when `a` is a strictly greater semver triple than `b`.
fn semver_gt(a: &str, b: &str) -> bool {
    fn triple(s: &str) -> (u64, u64, u64) {
        let mut it = s.trim().trim_start_matches('v').split('.');
        let p = |x: Option<&str>| x.unwrap_or("0").parse::<u64>().unwrap_or(0);
        (p(it.next()), p(it.next()), p(it.next()))
    }
    triple(a) > triple(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gates_parse_default_file() {
        let g = parse(DEFAULT_GATES).expect("the default file parses");
        assert_eq!(g.schema, SCHEMA);
        assert_eq!(g.cli_min_version, "1.0.0");
        assert_eq!(g.gate.len(), 8, "eight gates");
        let checks: usize = g.gate.iter().map(|x| x.check.len()).sum();
        assert_eq!(checks, 64, "sixty-four checks");
    }

    #[test]
    fn default_gates_satisfy_minimums() {
        let g = parse(DEFAULT_GATES).expect("parse");
        g.validate_minimums()
            .expect("the file satisfies the compiled minimums exactly, with no margin");
    }

    #[test]
    fn every_default_check_kind_is_in_the_closed_enum() {
        let g = parse(DEFAULT_GATES).expect("parse");
        for gate in &g.gate {
            for c in &gate.check {
                assert!(
                    CHECK_KINDS.contains(&c.kind.as_str()),
                    "{} is in the closed enum",
                    c.kind
                );
            }
        }
    }

    #[test]
    fn the_closed_enum_holds_thirty_kinds() {
        assert_eq!(CHECK_KINDS.len(), 30);
        let unique: BTreeSet<&&str> = CHECK_KINDS.iter().collect();
        assert_eq!(unique.len(), 30, "no kind is listed twice");
        for kind in CHECK_KINDS {
            assert!(kind_keys(kind).is_some(), "{kind} declares its key set");
        }
    }

    #[test]
    fn explore_gate_carries_a_zero_min_ratio_and_still_validates() {
        // The floor binds verify alone; explore's kill-case check sets 0.0.
        let g = parse(DEFAULT_GATES).expect("parse");
        let explore = g.gate_for("explore").expect("explore gate");
        let check = explore
            .check
            .iter()
            .find(|c| c.id == "kill-case-answered")
            .expect("the kill-case check");
        assert_eq!(check.float_key("min_ratio", 1.0), 0.0);
        g.validate_minimums().expect("valid");
    }

    #[test]
    fn unknown_kind_gives_e301() {
        let text = DEFAULT_GATES.replace("kind = \"file_exists\"", "kind = \"file_there\"");
        let err = parse(&text).expect_err("kind outside the closed enum");
        assert_eq!(err.code(), "DFA-E301");
        assert_eq!(err.exit(), 1);
    }

    #[test]
    fn unknown_key_gives_e302() {
        let text = r#"
schema = "devforgeai/gates/1"
cli_min_version = "1.0.0"

[[gate]]
phase = "explore"
requires = ""

  [[gate.check]]
  kind = "file_exists"
  id = "x"
  paths = ["a"]
  unexpected = 1
"#;
        let err = parse(text).expect_err("a key the kind does not define");
        assert_eq!(err.code(), "DFA-E302");
        assert!(err.diag().expect("diag").message.contains("unexpected"));
    }

    #[test]
    fn unknown_condition_key_gives_e302() {
        let text = r#"
schema = "devforgeai/gates/1"
cli_min_version = "1.0.0"

[[gate]]
phase = "explore"
requires = ""

  [[gate.check]]
  kind = "file_exists"
  id = "x"
  paths = ["a"]
  skip_when = { path = "a", nope = 1 }
"#;
        let err = parse(text).expect_err("a key outside the closed condition set");
        assert_eq!(err.code(), "DFA-E302");
    }

    #[test]
    fn missing_required_kind_gives_e303() {
        // A constitute gate whose checks are well formed and which omits the
        // one kind the compiled table requires of that phase.
        let text = r#"
schema = "devforgeai/gates/1"
cli_min_version = "1.0.0"

[[gate]]
phase = "constitute"
requires = "discover"

  [[gate.check]]
  kind = "no_open_questions"
  id = "CTX-OPEN"
  docs = ["context/*.md"]
"#;
        let g = parse(text).expect("parses");
        let err = g
            .validate_minimums()
            .expect_err("constitute needs context_audit");
        assert_eq!(err.code(), "DFA-E303");
        assert!(err.diag().expect("diag").message.contains("context_audit"));
    }

    #[test]
    fn required_kind_as_warn_gives_e303() {
        let text = DEFAULT_GATES.replace(
            "  kind = \"context_audit\"\n  id = \"CTX-AUDIT\"\n",
            "  kind = \"context_audit\"\n  id = \"CTX-AUDIT\"\n  severity = \"warn\"\n",
        );
        let g = parse(&text).expect("parses");
        let err = g
            .validate_minimums()
            .expect_err("a required kind at warn is treated as absent");
        assert_eq!(err.code(), "DFA-E303");
    }

    #[test]
    fn verifier_ratio_below_floor_gives_e303() {
        let text = DEFAULT_GATES.replace(
            "  id = \"verify-acs\"\n  verifiers = [\"ac-compliance-verifier\"]\n  min_ratio = 1.0",
            "  id = \"verify-acs\"\n  verifiers = [\"ac-compliance-verifier\"]\n  min_ratio = 0.5",
        );
        let g = parse(&text).expect("parses");
        let err = g.validate_minimums().expect_err("below the verify floor");
        assert_eq!(err.code(), "DFA-E303");
        assert!(err.diag().expect("diag").message.contains("min_ratio"));
    }

    #[test]
    fn cli_min_version_above_binary_gives_e304() {
        let text =
            DEFAULT_GATES.replace("cli_min_version = \"1.0.0\"", "cli_min_version = \"9.9.9\"");
        let err = parse(&text).expect_err("above this binary");
        assert_eq!(err.code(), "DFA-E304");
    }

    #[test]
    fn duplicate_phase_gives_e305() {
        let text = format!("{DEFAULT_GATES}\n[[gate]]\nphase = \"explore\"\nrequires = \"\"\n");
        let err = parse(&text).expect_err("one gate per phase");
        assert_eq!(err.code(), "DFA-E305");
        assert!(err.diag().expect("diag").message.contains("twice"));
    }

    #[test]
    fn duplicate_check_id_gives_e305() {
        let text = r#"
schema = "devforgeai/gates/1"
cli_min_version = "1.0.0"

[[gate]]
phase = "explore"
requires = ""

  [[gate.check]]
  kind = "file_exists"
  id = "dup"
  paths = ["a"]

  [[gate.check]]
  kind = "file_exists"
  id = "dup"
  paths = ["b"]
"#;
        let err = parse(text).expect_err("check ids are unique within a gate");
        assert_eq!(err.code(), "DFA-E305");
    }

    #[test]
    fn send_back_without_target_gives_e306() {
        let text = r#"
schema = "devforgeai/gates/1"
cli_min_version = "1.0.0"

[[gate]]
phase = "build"
requires = "plan"
on_fail = "send_back"
send_back_to = ""
"#;
        let err = parse(text).expect_err("send_back needs a target");
        assert_eq!(err.code(), "DFA-E306");
    }

    #[test]
    fn gate_document_supplies_the_default_path() {
        let g = parse(DEFAULT_GATES).expect("parse");
        let discover = g.gate_for("discover").expect("discover gate");
        assert_eq!(discover.document, "requirements.yaml");
        let c = discover
            .check
            .iter()
            .find(|c| c.id == "epic-not-empty")
            .expect("the epic check");
        assert_eq!(
            c.str_key("path", ""),
            "requirements.yaml",
            "the gate's document is the check's default path"
        );
    }

    #[test]
    fn a_check_overrides_the_gates_on_fail() {
        let g = parse(DEFAULT_GATES).expect("parse");
        let plan = g.gate_for("plan").expect("plan gate");
        assert_eq!(plan.on_fail, "fail");
        let ids = plan
            .check
            .iter()
            .find(|c| c.id == "plan-ids")
            .expect("plan-ids");
        assert_eq!(ids.on_fail, "send_back", "the check overrides the gate");
        let docs = plan
            .check
            .iter()
            .find(|c| c.id == "plan-docs")
            .expect("plan-docs");
        assert_eq!(docs.on_fail, "fail", "and inherits it when silent");
    }

    #[test]
    fn requires_defaults_to_the_preceding_phase() {
        let text = r#"
schema = "devforgeai/gates/1"
cli_min_version = "1.0.0"

[[gate]]
phase = "verify"
"#;
        let g = parse(text).expect("parse");
        assert_eq!(g.gate_for("verify").expect("gate").requires, "build");
    }

    #[test]
    fn no_gate_for_a_phase_gives_e300() {
        let g = parse(DEFAULT_GATES).expect("parse");
        let err = g.gate_for("design").expect_err("design holds no gate");
        assert_eq!(err.code(), "DFA-E300");
        assert_eq!(err.exit(), 1);
    }

    #[test]
    fn condition_keys_parse_from_the_default_file() {
        let g = parse(DEFAULT_GATES).expect("parse");
        let explore = g.gate_for("explore").expect("explore");

        let park = explore
            .check
            .iter()
            .find(|c| c.id == "park-has-revisit")
            .expect("park-has-revisit");
        let req = park.required_when.as_ref().expect("required_when");
        assert_eq!(req.field.as_deref(), Some("decision"));
        assert_eq!(req.equals.as_deref(), Some("park"));
        let nullw = park.null_when.as_ref().expect("null_when");
        assert_eq!(
            nullw.r#in.as_deref(),
            Some(&["kill".to_string(), "promote".to_string()][..])
        );

        let remedy = explore
            .check
            .iter()
            .find(|c| c.id == "remedy-flows-present")
            .expect("remedy-flows-present");
        let skip = remedy.skip_when.as_ref().expect("skip_when");
        assert_eq!(skip.state_field.as_deref(), Some("explore.remedy_flows"));
        assert_eq!(skip.is_empty, Some(true));

        let timebox = explore
            .check
            .iter()
            .find(|c| c.id == "time-box")
            .expect("time-box");
        assert_eq!(
            timebox
                .skip_when
                .as_ref()
                .expect("skip_when")
                .doc_exists
                .as_deref(),
            Some("explore/decision.yaml")
        );

        let promote = explore
            .check
            .iter()
            .find(|c| c.id == "promote-carries-forward")
            .expect("promote-carries-forward");
        assert!(promote.empty_when.is_some(), "empty_when parses");
    }

    #[test]
    fn semver_comparison_is_a_triple() {
        assert!(semver_gt("1.0.1", "1.0.0"));
        assert!(semver_gt("2.0.0", "1.9.9"));
        assert!(!semver_gt("1.0.0", "1.0.0"));
        assert!(!semver_gt("0.9.0", "1.0.0"));
    }
}
