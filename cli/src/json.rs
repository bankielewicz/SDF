//! The `--json` envelope. Every `--json` invocation prints exactly one object,
//! on failure as well as on success.

use crate::errors::Diag;
use serde::Serialize;
use std::io::Write;

/// One `errors[]` or `warnings[]` entry of the envelope.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct EnvelopeDiag {
    /// The `DFA-` code.
    pub code: String,
    /// The message text.
    pub message: String,
    /// `""` when the diagnostic has no location.
    pub path: String,
    /// `0` when the diagnostic has no location.
    pub line: u32,
}

impl From<&Diag> for EnvelopeDiag {
    fn from(d: &Diag) -> Self {
        EnvelopeDiag {
            code: d.code.to_string(),
            message: d.message.clone(),
            path: d.path.clone(),
            line: d.line,
        }
    }
}

/// The `devforgeai/cli-json/1` envelope, in the spec's key order.
#[derive(Debug, Clone, Serialize)]
pub struct Envelope {
    /// Fixed: `devforgeai/cli-json/1`.
    pub schema: String,
    /// The subcommand name with its space, as in `gate check`.
    pub command: String,
    /// `exit == 0`.
    pub ok: bool,
    /// The process exit code.
    pub exit: i32,
    /// Copied from `config.toml`; `false` when no config was loaded.
    pub degraded: bool,
    /// The resolved project root, or `""` when the subcommand resolved none.
    pub project: String,
    /// RFC 3339 UTC.
    pub at: String,
    /// The per-subcommand object.
    pub data: serde_json::Value,
    /// Error diagnostics.
    pub errors: Vec<EnvelopeDiag>,
    /// Warning diagnostics.
    pub warnings: Vec<EnvelopeDiag>,
}

impl Envelope {
    /// An envelope with every field at its empty value for `command`.
    pub fn new(command: impl Into<String>, at: impl Into<String>) -> Self {
        Envelope {
            schema: "devforgeai/cli-json/1".to_string(),
            command: command.into(),
            ok: true,
            exit: 0,
            degraded: false,
            project: String::new(),
            at: at.into(),
            data: serde_json::json!({}),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Set the exit code, keeping `ok` in step with it.
    pub fn with_exit(mut self, exit: i32) -> Self {
        self.exit = exit;
        self.ok = exit == 0;
        self
    }

    /// Push one diagnostic into `errors` or `warnings` by its code band.
    pub fn push_diag(&mut self, d: &Diag) {
        if d.is_warning() {
            self.warnings.push(d.into());
        } else {
            self.errors.push(d.into());
        }
    }
}

/// Write one envelope, followed by a newline.
pub fn emit(envelope: &Envelope, w: &mut dyn Write) -> std::io::Result<()> {
    let text = serde_json::to_string(envelope).map_err(std::io::Error::other)?;
    w.write_all(text.as_bytes())?;
    w.write_all(b"\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn emitted(e: &Envelope) -> String {
        let mut buf = Vec::new();
        emit(e, &mut buf).expect("emit");
        assert!(buf.ends_with(b"\n"), "envelope ends with a newline");
        String::from_utf8(buf).expect("UTF-8")
    }

    fn parse(e: &Envelope) -> serde_json::Value {
        serde_json::from_str(&emitted(e)).expect("one JSON object")
    }

    #[test]
    fn envelope_serialises_every_field() {
        let e = Envelope::new("gate check", "2026-09-10T14:02:11Z");
        // Key order is asserted on the emitted text: a parsed `Value` sorts
        // its map and would hide the order the spec writes.
        let text = emitted(&e);
        let mut cursor = 0usize;
        for key in [
            "schema", "command", "ok", "exit", "degraded", "project", "at", "data", "errors",
            "warnings",
        ] {
            let needle = format!("\"{key}\":");
            let at = text[cursor..]
                .find(&needle)
                .unwrap_or_else(|| panic!("{key} appears after the keys before it"));
            cursor += at + needle.len();
        }

        let v = parse(&e);
        assert_eq!(
            v.as_object().expect("object").len(),
            10,
            "ten keys and no more"
        );
        assert_eq!(v["schema"], "devforgeai/cli-json/1");
        assert_eq!(v["command"], "gate check");
        assert_eq!(v["ok"], true);
        assert_eq!(v["exit"], 0);
        assert_eq!(v["degraded"], false);
        assert_eq!(v["project"], "");
        assert_eq!(v["at"], "2026-09-10T14:02:11Z");
        assert_eq!(v["data"], serde_json::json!({}));
        assert_eq!(v["errors"], serde_json::json!([]));
        assert_eq!(v["warnings"], serde_json::json!([]));
    }

    #[test]
    fn envelope_emitted_on_failure() {
        let mut e = Envelope::new("gate check", "2026-09-10T14:02:11Z").with_exit(1);
        e.push_diag(&Diag::at_line(
            "DFA-E311",
            "test command 'cargo test' exited 101 for stack 'rust'",
            ".devforgeai/gates.toml",
            22,
        ));
        e.push_diag(&Diag::new(
            "DFA-W310",
            "check 'x' failed with severity warn",
        ));
        let v = parse(&e);
        assert_eq!(v["ok"], false);
        assert_eq!(v["exit"], 1);
        assert_eq!(v["errors"].as_array().expect("errors").len(), 1);
        assert_eq!(v["errors"][0]["code"], "DFA-E311");
        assert_eq!(v["errors"][0]["path"], ".devforgeai/gates.toml");
        assert_eq!(v["errors"][0]["line"], 22);
        assert_eq!(v["warnings"].as_array().expect("warnings").len(), 1);
        assert_eq!(v["warnings"][0]["code"], "DFA-W310");
        assert_eq!(v["warnings"][0]["path"], "");
        assert_eq!(v["warnings"][0]["line"], 0);
    }
}
