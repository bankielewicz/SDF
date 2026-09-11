//! The frontmatter grammar of conventions section 5.
//!
//! For a Markdown document line 1 is exactly `---`, the block ends at the next
//! line that is exactly `---`, and the text between is a YAML mapping. For a
//! YAML document the same keys are top-level keys and a leading `---` marker is
//! permitted and ignored. For `brand/tokens.json` the keys live under the
//! top-level object key `meta`.

use crate::errors::Diag;

/// The seven keys, in the order conventions section 5 fixes.
pub const KEYS: &[&str] = &[
    "schema",
    "id",
    "phase",
    "status",
    "produced_by",
    "consumes",
    "open_questions",
];

/// Where a document keeps its frontmatter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// A fenced block at the top of a Markdown file.
    Markdown,
    /// The top level of a YAML document.
    Yaml,
    /// The top level of a JSON document.
    Json,
    /// The `meta` object of a JSON document.
    ///
    /// `brand/tokens.json` alone: its top level is the token tree, so the
    /// envelope needs somewhere else to sit. Every other JSON document the
    /// framework produces carries the seven keys at the top level, the way a
    /// YAML document does.
    JsonMeta,
}

/// A parsed frontmatter block: the mapping, the key order as written, and the
/// line the body starts on.
#[derive(Debug, Clone)]
pub struct Frontmatter {
    /// The mapping, in the order the file writes it.
    pub map: serde_yaml_ng::Mapping,
    /// The top-level key names, in the order the file writes them.
    pub order: Vec<String>,
    /// The one-based line the frontmatter block starts on.
    pub start_line: u32,
    /// The one-based line the body starts on.
    pub body_line: u32,
    /// The body, for a Markdown document; the whole text otherwise.
    pub body: String,
}

impl Frontmatter {
    /// A string value, or `None` when the key is absent or not a string.
    pub fn str_key(&self, key: &str) -> Option<&str> {
        self.map
            .get(serde_yaml_ng::Value::String(key.to_string()))
            .and_then(serde_yaml_ng::Value::as_str)
    }

    /// The raw value at `key`.
    pub fn value(&self, key: &str) -> Option<&serde_yaml_ng::Value> {
        self.map.get(serde_yaml_ng::Value::String(key.to_string()))
    }

    /// A string-sequence value, or `None` when the key is absent or is not a
    /// sequence of strings.
    pub fn seq_key(&self, key: &str) -> Option<Vec<String>> {
        let seq = self.value(key)?.as_sequence()?;
        seq.iter()
            .map(|v| v.as_str().map(str::to_string))
            .collect::<Option<Vec<String>>>()
    }

    /// The one-based line the named key sits on, counted from the file start.
    pub fn line_of(&self, key: &str) -> u32 {
        match self.order.iter().position(|k| k == key) {
            Some(i) => self.start_line + i as u32,
            None => self.start_line,
        }
    }
}

/// Parse the frontmatter of `src`, reading it from where `kind` places it.
pub fn parse(src: &str, kind: Source, path: &str) -> Result<Frontmatter, Diag> {
    match kind {
        Source::Markdown => parse_markdown(src, path),
        Source::Yaml => parse_yaml(src, path),
        Source::Json => parse_json_flat(src, path),
        Source::JsonMeta => parse_json_meta(src, path),
    }
}

fn parse_markdown(src: &str, path: &str) -> Result<Frontmatter, Diag> {
    let lines: Vec<&str> = src.lines().collect();
    if lines.first().map(|l| l.trim_end()) != Some("---") {
        return Err(Diag::at_line(
            "DFA-E201",
            format!("{path} has no frontmatter; line 1 opens with three dashes"),
            path,
            1,
        ));
    }
    let close = lines
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, l)| l.trim_end() == "---")
        .map(|(i, _)| i);
    let Some(close) = close else {
        return Err(Diag::at_line(
            "DFA-E201",
            format!("{path} has no frontmatter; line 1 opens with three dashes"),
            path,
            1,
        ));
    };

    let block = lines[1..close].join("\n");
    let (map, order) = mapping(&block, path, 2)?;
    Ok(Frontmatter {
        map,
        order,
        start_line: 2,
        body_line: close as u32 + 2,
        body: lines[close + 1..].join("\n"),
    })
}

fn parse_yaml(src: &str, path: &str) -> Result<Frontmatter, Diag> {
    // A leading `---` document marker is permitted and ignored.
    let mut start_line = 1u32;
    let text = match src.strip_prefix("---\n") {
        Some(rest) => {
            start_line = 2;
            rest
        }
        None => match src.strip_prefix("---\r\n") {
            Some(rest) => {
                start_line = 2;
                rest
            }
            None => src,
        },
    };
    let (map, order) = mapping(text, path, start_line)?;
    Ok(Frontmatter {
        map,
        order,
        start_line,
        body_line: start_line,
        body: src.to_string(),
    })
}

/// The seven keys at the top level of a JSON document.
fn parse_json_flat(src: &str, path: &str) -> Result<Frontmatter, Diag> {
    let v: serde_json::Value = serde_json::from_str(src).map_err(|e| {
        Diag::at_line(
            "DFA-E202",
            format!("{path} is not valid JSON: {e}"),
            path,
            1,
        )
    })?;
    let obj = v.as_object().ok_or_else(|| {
        Diag::at_line(
            "DFA-E202",
            format!("{path} frontmatter is not a YAML mapping"),
            path,
            1,
        )
    })?;

    // A `meta` wrapper is the shape `brand/tokens.json` uses, and the one
    // every other JSON document is mistaken for. Saying so is worth more than
    // reporting seven absent keys one at a time.
    if !obj.contains_key(KEYS[0]) {
        if let Some(meta) = obj.get("meta") {
            if meta.is_object() {
                return Err(Diag::at_line(
                    "DFA-E203",
                    format!(
                        "{path} frontmatter has no key '{}' at the top level; `meta` is not a wrapper for this document",
                        KEYS[0]
                    ),
                    path,
                    1,
                ));
            }
        }
    }

    Ok(from_json_object(obj, src))
}

/// The seven keys inside the `meta` object.
fn parse_json_meta(src: &str, path: &str) -> Result<Frontmatter, Diag> {
    let v: serde_json::Value = serde_json::from_str(src).map_err(|e| {
        Diag::at_line(
            "DFA-E202",
            format!("{path} is not valid JSON: {e}"),
            path,
            1,
        )
    })?;
    let meta = v.get("meta").ok_or_else(|| {
        Diag::at_line(
            "DFA-E203",
            format!("{path} frontmatter has no key 'meta'"),
            path,
            1,
        )
    })?;
    let obj = meta.as_object().ok_or_else(|| {
        Diag::at_line(
            "DFA-E202",
            format!("{path} frontmatter is not a YAML mapping"),
            path,
            1,
        )
    })?;

    Ok(from_json_object(obj, src))
}

/// A `Frontmatter` over a JSON object's keys, in the order they were written.
fn from_json_object(obj: &serde_json::Map<String, serde_json::Value>, src: &str) -> Frontmatter {
    let order: Vec<String> = obj.keys().cloned().collect();
    let mut map = serde_yaml_ng::Mapping::new();
    for (k, val) in obj {
        let converted = serde_yaml_ng::to_value(val).unwrap_or(serde_yaml_ng::Value::Null);
        map.insert(serde_yaml_ng::Value::String(k.clone()), converted);
    }
    Frontmatter {
        map,
        order,
        start_line: 1,
        body_line: 1,
        body: src.to_string(),
    }
}

fn mapping(
    text: &str,
    path: &str,
    start_line: u32,
) -> Result<(serde_yaml_ng::Mapping, Vec<String>), Diag> {
    let value: serde_yaml_ng::Value = serde_yaml_ng::from_str(text).map_err(|e| {
        Diag::at_line(
            "DFA-E202",
            format!("{path} frontmatter is not a YAML mapping: {e}"),
            path,
            start_line,
        )
    })?;
    let map = match value {
        serde_yaml_ng::Value::Mapping(m) => m,
        _ => {
            return Err(Diag::at_line(
                "DFA-E202",
                format!("{path} frontmatter is not a YAML mapping"),
                path,
                start_line,
            ))
        }
    };
    let order: Vec<String> = map
        .keys()
        .map(|k| k.as_str().unwrap_or("").to_string())
        .collect();
    Ok((map, order))
}

/// The YAML type name of a value, for the `DFA-E206` message.
pub fn type_name(v: &serde_yaml_ng::Value) -> &'static str {
    match v {
        serde_yaml_ng::Value::Null => "null",
        serde_yaml_ng::Value::Bool(_) => "a boolean",
        serde_yaml_ng::Value::Number(_) => "a number",
        serde_yaml_ng::Value::String(_) => "a string",
        serde_yaml_ng::Value::Sequence(_) => "an array",
        serde_yaml_ng::Value::Mapping(_) => "a mapping",
        serde_yaml_ng::Value::Tagged(_) => "a tagged value",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL: &str = "---\nschema: devforgeai/story/1\nid: STORY-014\nphase: plan\nstatus: ready\nproduced_by: planning-work\nconsumes: []\nopen_questions: []\n---\n\n# Order checkout\n";

    #[test]
    fn frontmatter_parses_minimal() {
        let fm = parse(MINIMAL, Source::Markdown, "s.md").expect("parses");
        assert_eq!(fm.order, KEYS);
        assert_eq!(fm.str_key("schema"), Some("devforgeai/story/1"));
        assert_eq!(fm.str_key("id"), Some("STORY-014"));
        assert_eq!(fm.seq_key("consumes"), Some(vec![]));
        assert!(fm.body.contains("# Order checkout"));
        assert_eq!(fm.start_line, 2);
        assert_eq!(fm.line_of("status"), 5);
    }

    #[test]
    fn frontmatter_missing_fence_gives_e201() {
        let d = parse("# no fence\n", Source::Markdown, "s.md").expect_err("no fence");
        assert_eq!(d.code, "DFA-E201");
        assert_eq!(d.line, 1);

        let d = parse("---\nschema: x\n", Source::Markdown, "s.md").expect_err("unterminated");
        assert_eq!(d.code, "DFA-E201");
    }

    #[test]
    fn frontmatter_that_is_not_a_mapping_gives_e202() {
        let d = parse("---\n- a\n- b\n---\n", Source::Markdown, "s.md").expect_err("a sequence");
        assert_eq!(d.code, "DFA-E202");
    }

    #[test]
    fn yaml_doc_reads_top_level_keys() {
        let src = "schema: devforgeai/requirements/1\nid: IDEA-003\nphase: discover\nstatus: accepted\nproduced_by: discovering-requirements\nconsumes: []\nopen_questions: []\nrequirements: []\n";
        let fm = parse(src, Source::Yaml, "r.yaml").expect("parses");
        assert_eq!(fm.str_key("id"), Some("IDEA-003"));
        assert_eq!(fm.order[0], "schema");
        assert_eq!(fm.order[7], "requirements");
        assert_eq!(fm.start_line, 1);
    }

    #[test]
    fn a_leading_document_marker_is_ignored() {
        let src = "---\nschema: devforgeai/sprint/1\nid: SPRINT-001\n";
        let fm = parse(src, Source::Yaml, "sprint.yaml").expect("parses");
        assert_eq!(fm.str_key("id"), Some("SPRINT-001"));
        assert_eq!(
            fm.start_line, 2,
            "the marker shifts the line numbers by one"
        );
    }

    #[test]
    fn tokens_json_reads_meta_key() {
        let src = r##"{
  "meta": {
    "schema": "devforgeai/tokens/1",
    "id": "TOKEN-001",
    "phase": "design",
    "status": "approved",
    "produced_by": "designing-interfaces",
    "consumes": [],
    "open_questions": []
  },
  "color": { "primary": { "light": "#3355ff", "dark": "#88aaff" } }
}"##;
        let fm = parse(src, Source::JsonMeta, "tokens.json").expect("parses");
        assert_eq!(fm.order, KEYS, "meta carries the seven keys in order");
        assert_eq!(fm.str_key("produced_by"), Some("designing-interfaces"));
    }

    #[test]
    fn tokens_json_without_meta_gives_e203() {
        let d = parse(r#"{"color":{}}"#, Source::JsonMeta, "t.json").expect_err("no meta");
        assert_eq!(d.code, "DFA-E203");
    }

    #[test]
    fn key_order_is_preserved_as_written() {
        let src = "---\nid: STORY-014\nschema: devforgeai/story/1\n---\n";
        let fm = parse(src, Source::Markdown, "s.md").expect("parses");
        assert_eq!(fm.order, vec!["id", "schema"], "order is read, not sorted");
    }

    #[test]
    fn type_names_render_for_the_e206_message() {
        assert_eq!(
            type_name(&serde_yaml_ng::Value::String("x".into())),
            "a string"
        );
        assert_eq!(
            type_name(&serde_yaml_ng::Value::Sequence(vec![])),
            "an array"
        );
        assert_eq!(type_name(&serde_yaml_ng::Value::Null), "null");
    }
}
