//! Argument dispatch. Maps `CliError` to a process exit code and nothing else.

use clap::Parser;
use devforgeai::cli::Cli;
use devforgeai::{json, run, time, Diag, Envelope, Io};

fn main() -> std::process::ExitCode {
    let args = match Cli::try_parse() {
        Ok(a) => a,
        Err(e) => {
            // clap prints help and version to stdout at exit 0; every other
            // parse failure is the spec's DFA-E010 usage class, exit 3.
            let kind = e.kind();
            let display = matches!(
                kind,
                clap::error::ErrorKind::DisplayHelp
                    | clap::error::ErrorKind::DisplayVersion
                    | clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
            );
            // Under `--json` every invocation prints exactly one object, a
            // parse failure included, so a caller parsing stdout as JSON does
            // not crash on a typo.
            if !display && wants_json() {
                let mut env = Envelope::new("any", time::now_rfc3339()).with_exit(3);
                env.push_diag(&Diag::new("DFA-E010", clap_message(&e)));
                let mut out = std::io::stdout();
                let _ = json::emit(&env, &mut out);
                return std::process::ExitCode::from(3);
            }
            let _ = e.print();
            return if display {
                std::process::ExitCode::from(0)
            } else {
                std::process::ExitCode::from(3)
            };
        }
    };

    let stdout = std::io::stdout();
    let stderr = std::io::stderr();
    let mut io = Io {
        out: Box::new(stdout.lock()),
        err: Box::new(stderr.lock()),
        stdin: None,
    };

    let exit = run(args, &mut io);
    std::process::ExitCode::from(exit as u8)
}

/// True when `--json` is on the command line. The parse failed, so the flag is
/// read from the raw arguments rather than from `Cli`.
fn wants_json() -> bool {
    std::env::args().any(|a| a == "--json")
}

/// clap's rendered error text as one line, so it fits a `DFA-` message.
fn clap_message(e: &clap::Error) -> String {
    let text = e.render().to_string();
    let first = text
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("invalid arguments");
    first
        .strip_prefix("error: ")
        .unwrap_or(first)
        .trim()
        .to_string()
}
