//! A small backtracking matcher for the anchored patterns `gates.toml` carries.
//!
//! The spec's pinned dependency list names no regex crate, and two check kinds
//! take a `pattern` key: `column_matches`, whose only use in the default file is
//! `^FLOW-[0-9]{3}$`, and `antipattern_clean`, which is outside this milestone.
//! Rather than add a dependency the spec does not name, the subset those
//! patterns need lives here: anchors, literals, `.`, character classes with
//! ranges and negation, and the quantifiers `*`, `+`, `?`, `{n}`, `{n,}`,
//! `{n,m}`. Alternation and capture groups are not supported; a pattern using
//! one is a compile error the caller reports.

use crate::errors::CliError;

/// One compiled term: an atom and its repetition bounds.
#[derive(Debug, Clone)]
struct Term {
    atom: Atom,
    min: usize,
    max: usize,
}

#[derive(Debug, Clone)]
enum Atom {
    Literal(char),
    Any,
    Class {
        negated: bool,
        items: Vec<ClassItem>,
    },
}

#[derive(Debug, Clone)]
enum ClassItem {
    Single(char),
    Range(char, char),
}

/// A compiled pattern.
#[derive(Debug, Clone)]
pub struct Pattern {
    terms: Vec<Term>,
    anchored_start: bool,
    anchored_end: bool,
    source: String,
}

const UNBOUNDED: usize = usize::MAX;

impl Pattern {
    /// Compile a pattern, or report the construct this matcher does not carry.
    pub fn compile(src: &str) -> Result<Pattern, CliError> {
        // An empty pattern matches everything, so a check configured with one
        // reports a clean result it never tested. Refusing at compile time is
        // what turns a blank `pattern =` into a gates.toml defect rather than
        // into a gate that silently always passes.
        if src.is_empty() {
            return Err(CliError::new(
                "DFA-E302",
                "pattern is empty; a pattern that matches every value tests nothing",
            ));
        }

        let chars: Vec<char> = src.chars().collect();
        let mut i = 0usize;

        let anchored_start = chars.first() == Some(&'^');
        if anchored_start {
            i += 1;
        }

        let mut terms: Vec<Term> = Vec::new();
        let mut anchored_end = false;

        while i < chars.len() {
            let c = chars[i];

            if c == '$' && i + 1 == chars.len() {
                anchored_end = true;
                break;
            }

            let atom = match c {
                '(' | ')' | '|' => {
                    return Err(CliError::new(
                        "DFA-E302",
                        format!(
                            "pattern '{src}' uses '{c}'; this matcher carries anchors, literals, '.', classes and quantifiers"
                        ),
                    ))
                }
                '[' => {
                    let (a, next) = parse_class(&chars, i, src)?;
                    i = next;
                    a
                }
                '.' => {
                    i += 1;
                    Atom::Any
                }
                '\\' => {
                    i += 1;
                    let e = chars.get(i).copied().ok_or_else(|| {
                        CliError::new("DFA-E302", format!("pattern '{src}' ends in a backslash"))
                    })?;
                    i += 1;
                    match e {
                        'd' => Atom::Class {
                            negated: false,
                            items: vec![ClassItem::Range('0', '9')],
                        },
                        'w' => Atom::Class {
                            negated: false,
                            items: vec![
                                ClassItem::Range('a', 'z'),
                                ClassItem::Range('A', 'Z'),
                                ClassItem::Range('0', '9'),
                                ClassItem::Single('_'),
                            ],
                        },
                        's' => Atom::Class {
                            negated: false,
                            items: vec![
                                ClassItem::Single(' '),
                                ClassItem::Single('\t'),
                                ClassItem::Single('\r'),
                                ClassItem::Single('\n'),
                            ],
                        },
                        other => Atom::Literal(other),
                    }
                }
                other => {
                    i += 1;
                    Atom::Literal(other)
                }
            };

            let (min, max, next) = parse_quantifier(&chars, i, src)?;
            i = next;
            terms.push(Term { atom, min, max });
        }

        Ok(Pattern {
            terms,
            anchored_start,
            anchored_end,
            source: src.to_string(),
        })
    }

    /// The pattern text this was compiled from.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// True when the pattern matches `text`, anchored where the pattern anchors
    /// and searched otherwise.
    pub fn is_match(&self, text: &str) -> bool {
        let chars: Vec<char> = text.chars().collect();
        if self.anchored_start {
            return self.match_at(&chars, 0).is_some();
        }
        for start in 0..=chars.len() {
            if self.match_at(&chars, start).is_some() {
                return true;
            }
        }
        false
    }

    fn match_at(&self, text: &[char], start: usize) -> Option<usize> {
        self.match_terms(text, start, 0)
    }

    fn match_terms(&self, text: &[char], pos: usize, term: usize) -> Option<usize> {
        if term == self.terms.len() {
            if self.anchored_end && pos != text.len() {
                return None;
            }
            return Some(pos);
        }
        let t = &self.terms[term];

        // How far the atom can repeat from here.
        let mut count = 0usize;
        let mut cursor = pos;
        while count < t.max && cursor < text.len() && atom_matches(&t.atom, text[cursor]) {
            cursor += 1;
            count += 1;
        }
        // Give back one at a time until the tail matches.
        while count + 1 > t.min {
            if let Some(end) = self.match_terms(text, pos + count, term + 1) {
                return Some(end);
            }
            if count == 0 {
                break;
            }
            count -= 1;
        }
        if t.min == 0 {
            return self.match_terms(text, pos, term + 1);
        }
        if count >= t.min {
            return self.match_terms(text, pos + count, term + 1);
        }
        None
    }
}

fn atom_matches(atom: &Atom, c: char) -> bool {
    match atom {
        Atom::Literal(l) => *l == c,
        Atom::Any => c != '\n',
        Atom::Class { negated, items } => {
            let hit = items.iter().any(|it| match it {
                ClassItem::Single(s) => *s == c,
                ClassItem::Range(a, b) => c >= *a && c <= *b,
            });
            hit != *negated
        }
    }
}

fn parse_class(chars: &[char], mut i: usize, src: &str) -> Result<(Atom, usize), CliError> {
    i += 1; // consume '['
    let negated = chars.get(i) == Some(&'^');
    if negated {
        i += 1;
    }
    let mut items = Vec::new();
    let mut closed = false;
    while i < chars.len() {
        if chars[i] == ']' {
            i += 1;
            closed = true;
            break;
        }
        let c = if chars[i] == '\\' {
            i += 1;
            chars.get(i).copied().ok_or_else(|| {
                CliError::new("DFA-E302", format!("pattern '{src}' ends inside a class"))
            })?
        } else {
            chars[i]
        };
        i += 1;
        if chars.get(i) == Some(&'-') && chars.get(i + 1).is_some_and(|n| *n != ']') {
            let hi = chars[i + 1];
            i += 2;
            items.push(ClassItem::Range(c, hi));
        } else {
            items.push(ClassItem::Single(c));
        }
    }
    if !closed {
        return Err(CliError::new(
            "DFA-E302",
            format!("pattern '{src}' has an unterminated character class"),
        ));
    }
    Ok((Atom::Class { negated, items }, i))
}

fn parse_quantifier(
    chars: &[char],
    mut i: usize,
    src: &str,
) -> Result<(usize, usize, usize), CliError> {
    match chars.get(i) {
        Some('*') => Ok((0, UNBOUNDED, i + 1)),
        Some('+') => Ok((1, UNBOUNDED, i + 1)),
        Some('?') => Ok((0, 1, i + 1)),
        Some('{') => {
            i += 1;
            let mut lo = String::new();
            while chars.get(i).is_some_and(|c| c.is_ascii_digit()) {
                lo.push(chars[i]);
                i += 1;
            }
            let min: usize = lo.parse().map_err(|_| {
                CliError::new(
                    "DFA-E302",
                    format!("pattern '{src}' has an empty repeat count"),
                )
            })?;
            let max = match chars.get(i) {
                Some(',') => {
                    i += 1;
                    let mut hi = String::new();
                    while chars.get(i).is_some_and(|c| c.is_ascii_digit()) {
                        hi.push(chars[i]);
                        i += 1;
                    }
                    if hi.is_empty() {
                        UNBOUNDED
                    } else {
                        hi.parse().unwrap_or(UNBOUNDED)
                    }
                }
                _ => min,
            };
            if chars.get(i) != Some(&'}') {
                return Err(CliError::new(
                    "DFA-E302",
                    format!("pattern '{src}' has an unterminated repeat"),
                ));
            }
            Ok((min, max, i + 1))
        }
        _ => Ok((1, 1, i)),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn an_empty_pattern_is_refused() {
        // An empty pattern matches every value, so a check configured with one
        // reports a clean result it never tested.
        let e = Pattern::compile("").expect_err("an empty pattern tests nothing");
        assert_eq!(e.code(), "DFA-E302");
        let msg = &e.diag().expect("diag").message;
        assert!(msg.contains("empty"), "{msg}");
    }

    use super::*;

    fn m(pattern: &str, text: &str) -> bool {
        Pattern::compile(pattern)
            .expect("the pattern compiles")
            .is_match(text)
    }

    #[test]
    fn the_default_gates_flow_pattern_matches() {
        assert!(m("^FLOW-[0-9]{3}$", "FLOW-001"));
        assert!(m("^FLOW-[0-9]{3}$", "FLOW-999"));
        assert!(!m("^FLOW-[0-9]{3}$", "FLOW-01"));
        assert!(!m("^FLOW-[0-9]{3}$", "FLOW-0012"));
        assert!(!m("^FLOW-[0-9]{3}$", "flow-001"));
        assert!(!m("^FLOW-[0-9]{3}$", "xFLOW-001"));
    }

    #[test]
    fn the_doc_type_id_patterns_match() {
        assert!(m("^IDEA-[0-9]{3}$", "IDEA-003"));
        assert!(m("^ADR-[0-9]{3}$", "ADR-014"));
        assert!(m(r"^v[0-9]+\.[0-9]+\.[0-9]+$", "v0.3.0"));
        assert!(m(r"^v[0-9]+\.[0-9]+\.[0-9]+$", "v10.20.30"));
        assert!(!m(r"^v[0-9]+\.[0-9]+\.[0-9]+$", "v0.3"));
        assert!(m("^[0-9]{4}-[0-9]{2}-[0-9]{2}$", "2026-09-10"));
        assert!(m("^devforgeai/[a-z-]+/1$", "devforgeai/explore-payload/1"));
        assert!(!m("^devforgeai/[a-z-]+/1$", "devforgeai/Thing/1"));
        assert!(m("^[a-z][a-z0-9-]*$", "planning-work"));
        assert!(!m("^[a-z][a-z0-9-]*$", "Planning-Work"));
    }

    #[test]
    fn quantifiers_backtrack() {
        assert!(m("^a+b$", "aaab"));
        assert!(m("^a*b$", "b"));
        assert!(m("^a?b$", "ab"));
        assert!(m("^a?b$", "b"));
        assert!(!m("^a+b$", "b"));
        assert!(m("^[0-9]{2,4}$", "12"));
        assert!(m("^[0-9]{2,4}$", "1234"));
        assert!(!m("^[0-9]{2,4}$", "1"));
        assert!(!m("^[0-9]{2,4}$", "12345"));
        assert!(m("^[0-9]{2,}$", "123456"));
    }

    #[test]
    fn a_negated_class_excludes() {
        assert!(m("^[^0-9]+$", "abc"));
        assert!(!m("^[^0-9]+$", "ab1"));
    }

    #[test]
    fn an_unanchored_pattern_searches() {
        assert!(m("FLOW-[0-9]{3}", "see FLOW-002 here"));
        assert!(!m("FLOW-[0-9]{3}", "see FLOW-2 here"));
    }

    #[test]
    fn escapes_and_dot() {
        assert!(m(r"^a\.b$", "a.b"));
        assert!(!m(r"^a\.b$", "axb"));
        assert!(m("^a.b$", "axb"));
        assert!(m(r"^\d{3}$", "123"));
    }

    #[test]
    fn an_unsupported_construct_is_reported_not_ignored() {
        let err = Pattern::compile("^(a|b)$").expect_err("alternation is not carried");
        assert_eq!(err.code(), "DFA-E302");
        assert!(Pattern::compile("^[a-z$").is_err(), "an unterminated class");
        assert!(Pattern::compile("^a{2$").is_err(), "an unterminated repeat");
    }
}
