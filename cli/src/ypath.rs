//! The path grammar of `from`, `to`, `into`, `cover`, `universe`, `metric`,
//! and `field`.
//!
//! Dot-separated segments where `[]` after a segment iterates a sequence and a
//! trailing segment after `[]` names a key inside each item, so
//! `observations[].id` resolves to the set of `id` values of the `observations`
//! sequence. An empty sequence at a `[]` segment resolves to the empty set. A
//! path one of whose segments is absent from the document is `DFA-E345`.

use serde_yaml_ng::Value;

/// A segment of the path is absent from the document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Missing {
    /// The segment that did not resolve.
    pub segment: String,
    /// The whole path, for the message.
    pub path: String,
}

/// Resolve `path` against `root`, returning every value it names.
pub fn resolve<'a>(root: &'a Value, path: &str) -> Result<Vec<&'a Value>, Missing> {
    let mut current: Vec<&Value> = vec![root];
    // Set when a `[]` segment resolved and the sequence it named held nothing.
    // The distinction the rest of the engine rests on is between a segment the
    // document does not define, which is `DFA-E345`, and a set that is legibly
    // empty, which is an answer: `requirements[].actor` over `requirements: []`
    // names no actor because there is no requirement, not because `actor` is
    // misspelled.
    let mut emptied_by_sequence = false;

    for raw in path.split('.') {
        if raw.is_empty() {
            continue;
        }
        if current.is_empty() && emptied_by_sequence {
            return Ok(Vec::new());
        }
        let (name, iterate) = match raw.strip_suffix("[]") {
            Some(n) => (n, true),
            None => (raw, false),
        };

        let mut next: Vec<&Value> = Vec::new();
        let mut found_any = false;

        for value in &current {
            let child = if name.is_empty() {
                Some(*value)
            } else {
                value.get(Value::String(name.to_string()))
            };
            let Some(child) = child else {
                continue;
            };
            found_any = true;
            if iterate {
                match child.as_sequence() {
                    // An empty sequence resolves to the empty set.
                    Some(seq) => next.extend(seq.iter()),
                    None => next.push(child),
                }
            } else {
                next.push(child);
            }
        }

        if !found_any {
            return Err(Missing {
                segment: name.to_string(),
                path: path.to_string(),
            });
        }
        emptied_by_sequence = iterate && next.is_empty();
        current = next;
    }

    Ok(current)
}

/// Resolve `path` and return the values as strings, dropping non-scalars.
pub fn resolve_strings(root: &Value, path: &str) -> Result<Vec<String>, Missing> {
    Ok(resolve(root, path)?.iter().filter_map(scalar).collect())
}

/// The scalar text of a value: a string verbatim, a number or boolean
/// rendered, anything else `None`.
pub fn scalar(v: &&Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// The single value a path names, or `None` when it names none or many.
pub fn resolve_one<'a>(root: &'a Value, path: &str) -> Result<Option<&'a Value>, Missing> {
    let found = resolve(root, path)?;
    Ok(match found.len() {
        1 => Some(found[0]),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc() -> Value {
        serde_yaml_ng::from_str(
            r#"
id: IDEA-003
decision: promote
personas:
  - id: PERSONA-001
    name: Shopper
requirements:
  - id: REQ-001
    actor: PERSONA-001
    status: draft
  - id: REQ-002
    actor: PERSONA-001
    status: withdrawn
epics:
  - id: EPIC-001
    requirements: [REQ-001, REQ-002]
empty: []
nested:
  deep:
    value: 7
"#,
        )
        .expect("fixture parses")
    }

    #[test]
    fn a_plain_segment_reads_a_key() {
        let d = doc();
        assert_eq!(
            resolve_strings(&d, "decision").expect("resolves"),
            vec!["promote"]
        );
        assert_eq!(
            resolve_strings(&d, "nested.deep.value").expect("resolves"),
            vec!["7"]
        );
    }

    #[test]
    fn a_bracket_segment_iterates_and_a_trailing_segment_reads_each_item() {
        let d = doc();
        assert_eq!(
            resolve_strings(&d, "requirements[].id").expect("resolves"),
            vec!["REQ-001", "REQ-002"]
        );
        assert_eq!(
            resolve_strings(&d, "requirements[].actor").expect("resolves"),
            vec!["PERSONA-001", "PERSONA-001"]
        );
        assert_eq!(
            resolve_strings(&d, "personas[].id").expect("resolves"),
            vec!["PERSONA-001"]
        );
    }

    #[test]
    fn a_nested_sequence_flattens() {
        let d = doc();
        // `epics[].requirements` names one list per epic; the flattened set is
        // what `set_cover` compares.
        let values = resolve(&d, "epics[].requirements").expect("resolves");
        assert_eq!(values.len(), 1);
        assert_eq!(
            resolve_strings(&d, "epics[].requirements[]").expect("resolves"),
            vec!["REQ-001", "REQ-002"]
        );
    }

    #[test]
    fn an_empty_sequence_resolves_to_the_empty_set() {
        let d = doc();
        assert!(resolve_strings(&d, "empty[]").expect("resolves").is_empty());
        // An empty set is an answer, not a missing segment: `empty[].id` names
        // no id because there is nothing to have one. The `unwrap_or_default`
        // this replaces hid the difference from a check that has to tell them
        // apart to decide between `fail` and `DFA-E345`.
        assert!(resolve(&d, "empty[].id")
            .expect("an empty sequence is the empty set, not a missing segment")
            .is_empty());
        assert!(resolve(&d, "empty[].id.deeper")
            .expect("and stays empty however deep the path goes")
            .is_empty());
    }

    #[test]
    fn an_absent_segment_is_reported() {
        let d = doc();
        let e = resolve(&d, "nowhere").expect_err("absent");
        assert_eq!(e.segment, "nowhere");
        let e = resolve(&d, "requirements[].nothing").expect_err("absent inside items");
        assert_eq!(e.segment, "nothing");
    }

    #[test]
    fn resolve_one_names_a_single_value() {
        let d = doc();
        assert_eq!(
            resolve_one(&d, "decision")
                .expect("resolves")
                .and_then(|v| v.as_str()),
            Some("promote")
        );
        assert!(
            resolve_one(&d, "requirements[].id")
                .expect("resolves")
                .is_none(),
            "a set is not a single value"
        );
    }

    #[test]
    fn a_bracket_on_a_scalar_keeps_the_value() {
        let d = doc();
        // `stories[]` in a `length_between` check names the sequence itself
        // when the key holds one, and the value when it does not.
        assert_eq!(
            resolve_strings(&d, "decision[]").expect("resolves"),
            vec!["promote"]
        );
    }
}
