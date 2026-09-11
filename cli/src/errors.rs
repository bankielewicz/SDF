//! `CliError`, the one error type every subcommand returns, and `Diag`, the
//! located diagnostic that carries a `DFA-` code.

use crate::errors_table;

/// A located diagnostic: a code from the spec's error table, the message text
/// with its substitutions already made, and an optional location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diag {
    /// The `DFA-E<nnn>` or `DFA-W<nnn>` code.
    pub code: &'static str,
    /// The message, with the spec's angle-bracket substitutions filled in.
    pub message: String,
    /// The path the diagnostic points at, or `""` when it has no location.
    pub path: String,
    /// The line the diagnostic points at, or `0` when it has no location.
    pub line: u32,
}

impl Diag {
    /// A diagnostic with no location.
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Diag {
            code,
            message: message.into(),
            path: String::new(),
            line: 0,
        }
    }

    /// A diagnostic pointing at a path, with no line.
    pub fn at(code: &'static str, message: impl Into<String>, path: impl Into<String>) -> Self {
        Diag {
            code,
            message: message.into(),
            path: path.into(),
            line: 0,
        }
    }

    /// A diagnostic pointing at a path and a line.
    pub fn at_line(
        code: &'static str,
        message: impl Into<String>,
        path: impl Into<String>,
        line: u32,
    ) -> Self {
        Diag {
            code,
            message: message.into(),
            path: path.into(),
            line,
        }
    }

    /// True when the code names a warning row rather than an error row.
    pub fn is_warning(&self) -> bool {
        self.code.starts_with("DFA-W")
    }

    /// The two stderr lines the spec fixes: `devforgeai: <code> <subcommand>:
    /// <message>`, and where a location exists `  at <path>:<line>`.
    pub fn render(&self, subcommand: &str) -> String {
        let mut out = format!("devforgeai: {} {}: {}", self.code, subcommand, self.message);
        if !self.path.is_empty() {
            out.push_str(&format!("\n  at {}:{}", self.path, self.line));
        }
        out
    }
}

/// The error type `run` returns. Every variant carries a `Diag`, so the code
/// and the exit status come from one table rather than from the call site.
#[derive(Debug, Clone, thiserror::Error)]
pub enum CliError {
    /// A diagnostic whose code appears in the spec's error table.
    #[error("{}", .0.message)]
    Coded(Diag),
    /// A result, not an error: the gate resolved SEND BACK. Exit 2.
    #[error("send back to {0}")]
    SendBack(String),
}

impl CliError {
    /// A coded error with no location.
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        CliError::Coded(Diag::new(code, message))
    }

    /// A coded error pointing at a path.
    pub fn at(code: &'static str, message: impl Into<String>, path: impl Into<String>) -> Self {
        CliError::Coded(Diag::at(code, message, path))
    }

    /// A filesystem error, `DFA-E900`.
    pub fn io(op: &str, path: impl std::fmt::Display, source: &std::io::Error) -> Self {
        CliError::at(
            "DFA-E900",
            format!("{op} on {path} failed: {source}"),
            path.to_string(),
        )
    }

    /// The milestone-1 stub: a subcommand or check kind the spec defines and
    /// this binary does not yet evaluate.
    pub fn not_implemented(what: impl std::fmt::Display) -> Self {
        CliError::new(
            errors_table::NOT_IMPLEMENTED,
            format!("{what} is specified and not implemented in this build"),
        )
    }

    /// The `DFA-` code this error carries.
    pub fn code(&self) -> &'static str {
        match self {
            CliError::Coded(d) => d.code,
            CliError::SendBack(_) => "",
        }
    }

    /// The process exit code, taken from the spec's table for a coded error.
    pub fn exit(&self) -> i32 {
        match self {
            CliError::Coded(d) => errors_table::exit_for(d.code).unwrap_or(5),
            CliError::SendBack(_) => 2,
        }
    }

    /// The diagnostic, when this error carries one.
    pub fn diag(&self) -> Option<&Diag> {
        match self {
            CliError::Coded(d) => Some(d),
            CliError::SendBack(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_maps_to_exit() {
        // One code from each exit class the spec's table defines.
        assert_eq!(CliError::new("DFA-E010", "usage").exit(), 3);
        assert_eq!(CliError::new("DFA-E101", "no config").exit(), 1);
        assert_eq!(CliError::new("DFA-E501", "no trust file").exit(), 4);
        assert_eq!(CliError::new("DFA-E900", "io").exit(), 5);
        assert_eq!(CliError::new("DFA-W201", "drift").exit(), 0);
        assert_eq!(CliError::SendBack("build".into()).exit(), 2);
        assert_eq!(CliError::not_implemented("gate check").exit(), 5);
    }

    #[test]
    fn diag_renders_two_lines_with_a_location() {
        let d = Diag::at_line("DFA-E210", "references REQ-001", ".devforgeai/x.md", 7);
        assert_eq!(
            d.render("doc validate"),
            "devforgeai: DFA-E210 doc validate: references REQ-001\n  at .devforgeai/x.md:7"
        );
    }

    #[test]
    fn diag_renders_one_line_without_a_location() {
        let d = Diag::new("DFA-E011", "'gate check' requires --id");
        assert_eq!(
            d.render("gate check"),
            "devforgeai: DFA-E011 gate check: 'gate check' requires --id"
        );
    }
}
