//! Markdown reading: H2 headings and pipe tables, with the line number of
//! every row, so a diagnostic can point at the place a value was stated.
//!
//! The grammar is the one the templates use: a table is a run of lines that
//! begin with `|`, the first is the header, a row of dashes and colons is the
//! separator and carries no data, and `\|` inside a cell is a literal pipe.

/// One row of a table, with the one-based line it sits on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    /// The trimmed cells, outer pipes removed.
    pub cells: Vec<String>,
    /// The one-based line number.
    pub line: u32,
}

impl Row {
    /// The cell at `i`, or `""`.
    pub fn cell(&self, i: usize) -> &str {
        self.cells.get(i).map(String::as_str).unwrap_or("")
    }
}

/// One pipe table, with the H2 section it sits under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    /// The header cells.
    pub header: Vec<String>,
    /// The data rows.
    pub rows: Vec<Row>,
    /// The nearest preceding `## ` heading text, or `""`.
    pub section: String,
    /// The nearest preceding `### ` heading text, or `""`.
    pub subsection: String,
}

impl Table {
    /// The index of the column named `name`, case-insensitively.
    pub fn column(&self, name: &str) -> Option<usize> {
        self.header
            .iter()
            .position(|h| h.eq_ignore_ascii_case(name))
    }

    /// The value of column `name` in `row`, or `""`.
    pub fn get<'a>(&self, row: &'a Row, name: &str) -> &'a str {
        self.column(name).map(|i| row.cell(i)).unwrap_or("")
    }
}

/// Split one table line into cells, restoring escaped pipes.
fn cells_of(line: &str) -> Vec<String> {
    let protected = line.trim().replace(r"\|", "\u{0}");
    protected
        .trim_matches('|')
        .split('|')
        .map(|c| c.trim().replace('\u{0}', "|"))
        .collect()
}

/// True when every cell is a run of dashes and colons: the separator row.
fn is_separator(cells: &[String]) -> bool {
    !cells.is_empty()
        && cells
            .iter()
            .all(|c| !c.is_empty() && c.chars().all(|ch| ch == '-' || ch == ':'))
}

/// Every `## ` heading, in file order, with the `## ` prefix kept.
pub fn h2(text: &str) -> Vec<String> {
    text.lines()
        .filter(|l| !l.starts_with("###"))
        .filter_map(|l| l.strip_prefix("## "))
        .map(|rest| format!("## {}", rest.trim()))
        .collect()
}

/// Every `### ` heading, in file order, without the prefix.
pub fn h3(text: &str) -> Vec<String> {
    text.lines()
        .filter(|l| !l.starts_with("####"))
        .filter_map(|l| l.strip_prefix("### "))
        .map(|rest| rest.trim().to_string())
        .collect()
}

/// Every pipe table in the text, in file order.
pub fn tables(text: &str) -> Vec<Table> {
    let mut out: Vec<Table> = Vec::new();
    let mut section = String::new();
    let mut subsection = String::new();
    let mut open: Option<Table> = None;

    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim_end();
        let trimmed = line.trim_start();

        if trimmed.starts_with('|') {
            let cells = cells_of(trimmed);
            match &mut open {
                None => {
                    open = Some(Table {
                        header: cells,
                        rows: Vec::new(),
                        section: section.clone(),
                        subsection: subsection.clone(),
                    });
                }
                Some(t) => {
                    if !is_separator(&cells) {
                        t.rows.push(Row {
                            cells,
                            line: i as u32 + 1,
                        });
                    }
                }
            }
            continue;
        }

        if let Some(t) = open.take() {
            out.push(t);
        }
        if let Some(rest) = trimmed.strip_prefix("### ") {
            if !trimmed.starts_with("####") {
                subsection = rest.trim().to_string();
            }
        } else if let Some(rest) = trimmed.strip_prefix("## ") {
            section = format!("## {}", rest.trim());
            subsection.clear();
        }
    }
    if let Some(t) = open.take() {
        out.push(t);
    }
    out
}

/// The tables whose header row is exactly `columns`, case-insensitively.
pub fn tables_with_header<'a>(all: &'a [Table], columns: &[&str]) -> Vec<&'a Table> {
    all.iter()
        .filter(|t| {
            t.header.len() == columns.len()
                && t.header
                    .iter()
                    .zip(columns)
                    .all(|(h, c)| h.eq_ignore_ascii_case(c))
        })
        .collect()
}

/// The first table under the `## ` section named `name` (with its prefix).
pub fn section_table<'a>(all: &'a [Table], name: &str) -> Option<&'a Table> {
    all.iter().find(|t| t.section == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "# Title\n\n## Alpha\n\n| A | B |\n|---|---|\n| 1 | 2 |\n| 3 | 4 |\n\n## Beta\n\n### AP-001 thing\n\n| Field | Value |\n|---|---|\n| detector | a \\| b |\n";

    #[test]
    fn h2_keeps_the_prefix_and_skips_h3() {
        assert_eq!(h2(DOC), vec!["## Alpha".to_string(), "## Beta".to_string()]);
    }

    #[test]
    fn h3_drops_the_prefix() {
        assert_eq!(h3(DOC), vec!["AP-001 thing".to_string()]);
    }

    #[test]
    fn a_table_carries_its_section_and_line_numbers() {
        let t = tables(DOC);
        assert_eq!(t.len(), 2);
        assert_eq!(t[0].section, "## Alpha");
        assert_eq!(t[0].header, vec!["A".to_string(), "B".to_string()]);
        assert_eq!(t[0].rows.len(), 2, "the separator row is not data");
        assert_eq!(t[0].rows[0].line, 7);
        assert_eq!(t[0].rows[1].line, 8);
    }

    #[test]
    fn an_escaped_pipe_stays_inside_its_cell() {
        let t = tables(DOC);
        assert_eq!(t[1].subsection, "AP-001 thing");
        assert_eq!(
            t[1].rows[0].cells,
            vec!["detector".to_string(), "a | b".to_string()]
        );
    }

    #[test]
    fn a_header_match_is_case_insensitive_and_exact_in_width() {
        let all = tables(DOC);
        assert_eq!(tables_with_header(&all, &["a", "b"]).len(), 1);
        assert_eq!(tables_with_header(&all, &["A"]).len(), 0);
    }

    #[test]
    fn a_column_lookup_reads_by_name() {
        let all = tables(DOC);
        let t = section_table(&all, "## Alpha").expect("a table under Alpha");
        assert_eq!(t.get(&t.rows[0], "B"), "2");
        assert_eq!(t.get(&t.rows[0], "missing"), "");
    }
}
