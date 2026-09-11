//! Cross-reference resolution: every reference resolves in the ID index, and
//! the two directions of drift between `consumes` and the body are reported.

use super::ids::IdIndex;
use crate::errors::Diag;

/// A document's own identity, for the two drift warnings.
pub struct Subject<'a> {
    /// The path, project-relative in forward-slash form.
    pub path: &'a str,
    /// The document's own `id`, which is never a reference to itself.
    pub own_id: &'a str,
    /// The `consumes` list as written.
    pub consumes: &'a [String],
    /// The IDs the body cites, with the line each sits on.
    pub body_refs: &'a [(String, u32)],
    /// The IDs this document defines itself.
    ///
    /// `consumes` names what a document takes from elsewhere, so an id this
    /// one states and then cites again is not a consumption. A brief defines
    /// its flows in `## Core flows` and cites them in `## Mockups`; without
    /// this the second mention would ask for a `consumes` entry naming the
    /// document's own work.
    pub own_definitions: &'a [String],
}

/// `DFA-E210` for every reference with no definition, and `DFA-W201` and
/// `DFA-W202` for the two directions of drift.
pub fn resolve(subject: &Subject, index: &IdIndex) -> Vec<Diag> {
    let mut out = Vec::new();

    // Every entry of `consumes` with no definition is DFA-E210.
    for id in subject.consumes {
        if !index.defines(id) {
            out.push(Diag::at(
                "DFA-E210",
                format!(
                    "{} references {id}, which no document defines",
                    subject.path
                ),
                subject.path,
            ));
        }
    }

    // Every body reference with no definition is DFA-E210, reported once per ID.
    let mut seen: Vec<&str> = Vec::new();
    for (id, line) in subject.body_refs {
        if id == subject.own_id || seen.contains(&id.as_str()) {
            continue;
        }
        seen.push(id);
        if !index.defines(id) {
            out.push(Diag::at_line(
                "DFA-E210",
                format!(
                    "{} references {id}, which no document defines",
                    subject.path
                ),
                subject.path,
                *line,
            ));
        }
    }

    // DFA-W201: consumes lists an ID the body does not cite.
    for id in subject.consumes {
        if !subject.body_refs.iter().any(|(r, _)| r == id) {
            out.push(Diag::at(
                "DFA-W201",
                format!(
                    "{} consumes {id}, which the body does not cite",
                    subject.path
                ),
                subject.path,
            ));
        }
    }

    // DFA-W202: the body cites an ID absent from consumes.
    let mut warned: Vec<&str> = Vec::new();
    for (id, line) in subject.body_refs {
        if id == subject.own_id
            || subject.own_definitions.iter().any(|d| d == id)
            || subject.consumes.iter().any(|c| c == id)
            || warned.contains(&id.as_str())
        {
            continue;
        }
        warned.push(id);
        out.push(Diag::at_line(
            "DFA-W202",
            format!("{} cites {id}, which consumes does not list", subject.path),
            subject.path,
            *line,
        ));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::super::ids::{IdIndex, Site};
    use super::*;

    fn index_with(ids: &[&str]) -> IdIndex {
        let mut ix = IdIndex::default();
        for id in ids {
            ix.definitions.insert(
                (*id).to_string(),
                vec![Site {
                    path: "other.yaml".into(),
                    line: 1,
                    schema: "s".into(),
                }],
            );
        }
        ix
    }

    #[test]
    fn reference_without_definition_gives_e210() {
        let ix = index_with(&["REQ-001"]);
        let consumes = vec!["REQ-001".to_string()];
        let refs = vec![("REQ-001".to_string(), 9), ("REQ-777".to_string(), 11)];
        let d = resolve(
            &Subject {
                path: "s.md",
                own_id: "STORY-014",
                consumes: &consumes,
                body_refs: &refs,
                own_definitions: &[],
            },
            &ix,
        );
        let e210: Vec<_> = d.iter().filter(|x| x.code == "DFA-E210").collect();
        assert_eq!(e210.len(), 1);
        assert!(e210[0].message.contains("REQ-777"));
        assert_eq!(e210[0].line, 11);
    }

    #[test]
    fn consumes_without_body_gives_w201() {
        let ix = index_with(&["REQ-001", "REQ-002"]);
        let consumes = vec!["REQ-001".to_string(), "REQ-002".to_string()];
        let refs = vec![("REQ-001".to_string(), 9)];
        let d = resolve(
            &Subject {
                path: "s.md",
                own_id: "STORY-014",
                consumes: &consumes,
                body_refs: &refs,
                own_definitions: &[],
            },
            &ix,
        );
        let w: Vec<_> = d.iter().filter(|x| x.code == "DFA-W201").collect();
        assert_eq!(w.len(), 1);
        assert!(w[0].message.contains("REQ-002"));
        assert_eq!(w[0].code, "DFA-W201");
    }

    #[test]
    fn body_without_consumes_gives_w202() {
        let ix = index_with(&["REQ-001", "REQ-003"]);
        let consumes = vec!["REQ-001".to_string()];
        let refs = vec![("REQ-001".to_string(), 9), ("REQ-003".to_string(), 12)];
        let d = resolve(
            &Subject {
                path: "s.md",
                own_id: "STORY-014",
                consumes: &consumes,
                body_refs: &refs,
                own_definitions: &[],
            },
            &ix,
        );
        let w: Vec<_> = d.iter().filter(|x| x.code == "DFA-W202").collect();
        assert_eq!(w.len(), 1);
        assert!(w[0].message.contains("REQ-003"));
        assert_eq!(w[0].line, 12);
    }

    #[test]
    fn a_document_does_not_reference_itself() {
        let ix = index_with(&[]);
        let refs = vec![("STORY-014".to_string(), 2)];
        let d = resolve(
            &Subject {
                path: "s.md",
                own_id: "STORY-014",
                consumes: &[],
                body_refs: &refs,
                own_definitions: &[],
            },
            &ix,
        );
        assert!(d.is_empty(), "its own id is neither unresolved nor drift");
    }

    #[test]
    fn an_unresolved_consumes_entry_gives_e210() {
        let ix = index_with(&[]);
        let consumes = vec!["REQ-004".to_string()];
        let d = resolve(
            &Subject {
                path: "s.md",
                own_id: "STORY-014",
                consumes: &consumes,
                body_refs: &[],
                own_definitions: &[],
            },
            &ix,
        );
        assert!(d.iter().any(|x| x.code == "DFA-E210"));
        assert!(d.iter().any(|x| x.code == "DFA-W201"));
    }
}
