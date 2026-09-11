//! The command runner the `tests_pass`, `coverage_min`, `lint_clean`, and
//! `complexity_clean` checks share.
//!
//! The command string goes to `cmd.exe /C` on Windows and `/bin/sh -c`
//! elsewhere, with the working directory at the project root, the environment
//! extended by the stack's own `env` plus `DEVFORGEAI=1` and `CI=1`, both
//! streams captured, and a wall-clock limit.

use crate::config::Stack;
use crate::errors::CliError;
use std::collections::BTreeMap;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// The last bytes of the merged output the report records.
pub const TAIL_BYTES: usize = 4096;

/// What one command produced.
#[derive(Debug, Clone)]
pub struct Run {
    /// The command string, verbatim.
    pub command: String,
    /// The process exit code, or -1 when it was killed.
    pub exit_code: i32,
    /// Wall-clock milliseconds.
    pub duration_ms: u128,
    /// stdout and stderr merged, in arrival order per stream.
    pub output: String,
    /// True when the wall-clock limit was reached and the child terminated.
    pub timed_out: bool,
}

impl Run {
    /// True when the command exited 0 and did not time out.
    pub fn ok(&self) -> bool {
        self.exit_code == 0 && !self.timed_out
    }

    /// The last `TAIL_BYTES` of the merged output, cut on a character boundary.
    pub fn tail(&self) -> String {
        let bytes = self.output.as_bytes();
        if bytes.len() <= TAIL_BYTES {
            return self.output.clone();
        }
        let mut at = bytes.len() - TAIL_BYTES;
        while at < bytes.len() && !self.output.is_char_boundary(at) {
            at += 1;
        }
        self.output[at..].to_string()
    }

    /// The `evidence` mapping the report records for a command check.
    pub fn evidence(&self) -> serde_yaml_ng::Value {
        let mut m = serde_yaml_ng::Mapping::new();
        m.insert("command".into(), self.command.clone().into());
        m.insert("exit_code".into(), self.exit_code.into());
        m.insert(
            "duration_ms".into(),
            serde_yaml_ng::Value::Number((self.duration_ms as u64).into()),
        );
        m.insert("output_tail".into(), self.tail().into());
        serde_yaml_ng::Value::Mapping(m)
    }
}

/// The shell the spec names for this platform.
fn shell() -> (&'static str, &'static str) {
    if cfg!(windows) {
        ("cmd.exe", "/C")
    } else {
        ("/bin/sh", "-c")
    }
}

/// Run one command string at `root`, `DFA-E319` when it cannot start and
/// `DFA-E318` when it outlives `timeout_secs`.
pub fn run(
    root: &Path,
    command: &str,
    env: &BTreeMap<String, String>,
    timeout_secs: u64,
) -> Result<Run, CliError> {
    let (program, flag) = shell();
    let started = Instant::now();

    let mut child = Command::new(program)
        .arg(flag)
        .arg(command)
        .current_dir(root)
        .envs(env)
        .env("DEVFORGEAI", "1")
        .env("CI", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            CliError::new(
                "DFA-E319",
                format!("command '{command}' could not start: {e}"),
            )
        })?;

    // The wall-clock limit, polled rather than blocking on `wait`, so a child
    // that outlives it is terminated rather than waited on forever.
    let limit = Duration::from_secs(timeout_secs.max(1));
    let mut timed_out = false;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) => {
                if started.elapsed() >= limit {
                    let _ = child.kill();
                    let _ = child.wait();
                    timed_out = true;
                    break;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(e) => {
                return Err(CliError::new(
                    "DFA-E319",
                    format!("command '{command}' could not start: {e}"),
                ))
            }
        }
    }

    let out = child.wait_with_output().map_err(|e| {
        CliError::new(
            "DFA-E319",
            format!("command '{command}' could not start: {e}"),
        )
    })?;

    let mut output = String::from_utf8_lossy(&out.stdout).to_string();
    output.push_str(&String::from_utf8_lossy(&out.stderr));

    Ok(Run {
        command: command.to_string(),
        exit_code: if timed_out {
            -1
        } else {
            out.status.code().unwrap_or(-1)
        },
        duration_ms: started.elapsed().as_millis(),
        output,
        timed_out,
    })
}

/// The test-count pattern of each stack, applied to the merged output.
///
/// Counts are informational: the check result comes from the exit code alone,
/// and a pattern that does not match leaves both counts `None`.
pub fn test_counts(stack: &str, output: &str) -> (Option<u64>, Option<u64>) {
    let one = |pattern: &str, group: usize| -> Option<u64> {
        regex::Regex::new(pattern)
            .ok()?
            .captures(output)?
            .get(group)?
            .as_str()
            .parse()
            .ok()
    };

    match stack {
        "rust" => (
            one(r"test result: ok\. (\d+) passed; (\d+) failed", 1),
            one(r"test result: ok\. (\d+) passed; (\d+) failed", 2),
        ),
        "node" => (one(r"Tests\s+(\d+) passed", 1), one(r"(\d+) failed", 1)),
        "python" => (one(r"(\d+) passed", 1), one(r"(\d+) failed", 1)),
        "go" => {
            let passed = output.lines().filter(|l| ok_line(l)).count() as u64;
            let failed = output.lines().filter(|l| l.starts_with("FAIL")).count() as u64;
            if passed == 0 && failed == 0 {
                (None, None)
            } else {
                (Some(passed), Some(failed))
            }
        }
        "dotnet" => (
            one(r"Failed:\s+(\d+),\s+Passed:\s+(\d+)", 2),
            one(r"Failed:\s+(\d+),\s+Passed:\s+(\d+)", 1),
        ),
        "jvm" => (
            one(r"Tests run: (\d+), Failures: (\d+)", 1),
            one(r"Tests run: (\d+), Failures: (\d+)", 2),
        ),
        "ruby" => (
            one(r"(\d+) examples?, (\d+) failures?", 1),
            one(r"(\d+) examples?, (\d+) failures?", 2),
        ),
        _ => (None, None),
    }
}

/// `^ok\s+\S+`, the go test line that counts as one passing package.
fn ok_line(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("ok") else {
        return false;
    };
    let trimmed = rest.trim_start();
    rest.len() != trimmed.len() && !trimmed.is_empty()
}

/// The environment a stack contributes.
pub fn stack_env(stack: &Stack) -> BTreeMap<String, String> {
    stack.env.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp() -> tempfile::TempDir {
        tempfile::tempdir().expect("temp dir")
    }

    #[test]
    fn a_command_that_exits_zero_is_ok() {
        let dir = temp();
        let r = run(dir.path(), "exit 0", &BTreeMap::new(), 30).expect("runs");
        assert!(r.ok());
        assert_eq!(r.exit_code, 0);
        assert_eq!(r.command, "exit 0");
    }

    #[test]
    fn a_command_that_exits_one_carries_its_code() {
        let dir = temp();
        let r = run(dir.path(), "exit 1", &BTreeMap::new(), 30).expect("runs");
        assert!(!r.ok());
        assert_eq!(r.exit_code, 1);
    }

    #[test]
    fn the_evidence_mapping_carries_the_four_keys() {
        let dir = temp();
        let r = run(dir.path(), "exit 0", &BTreeMap::new(), 30).expect("runs");
        let e = r.evidence();
        for key in ["command", "exit_code", "duration_ms", "output_tail"] {
            assert!(e.get(key).is_some(), "{key} is recorded");
        }
    }

    #[test]
    fn the_tail_keeps_the_last_bytes() {
        let r = Run {
            command: "x".into(),
            exit_code: 0,
            duration_ms: 0,
            output: "a".repeat(TAIL_BYTES + 10),
            timed_out: false,
        };
        assert_eq!(r.tail().len(), TAIL_BYTES);
    }

    #[test]
    fn the_rust_pattern_reads_both_counts() {
        let out = "test result: ok. 214 passed; 0 failed; 0 ignored";
        assert_eq!(test_counts("rust", out), (Some(214), Some(0)));
    }

    #[test]
    fn an_unmatched_pattern_leaves_the_counts_absent() {
        assert_eq!(test_counts("rust", "nothing here"), (None, None));
        assert_eq!(test_counts("invented", "214 passed"), (None, None));
    }

    #[test]
    fn the_go_pattern_counts_lines() {
        let out = "ok  \texample/a\t0.1s\nok  \texample/b\t0.2s\nFAIL\texample/c\n";
        assert_eq!(test_counts("go", out), (Some(2), Some(1)));
    }

    #[test]
    fn the_dotnet_pattern_reads_the_columns_in_its_own_order() {
        assert_eq!(
            test_counts("dotnet", "Failed:     2, Passed:    40, Skipped: 0"),
            (Some(40), Some(2))
        );
    }

    #[test]
    fn the_ruby_and_jvm_patterns_read_both_counts() {
        assert_eq!(
            test_counts("ruby", "12 examples, 1 failure"),
            (Some(12), Some(1))
        );
        assert_eq!(
            test_counts("jvm", "Tests run: 30, Failures: 2, Errors: 0"),
            (Some(30), Some(2))
        );
    }
}
