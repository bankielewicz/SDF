//! The `devforgeai` enforcement core.
//!
//! Every subcommand is deterministic given the file system, the environment,
//! and the exit status of the commands named in `config.toml`. The binary
//! authors no content: it parses, resolves, evaluates, and renders.

pub mod aggregate;
pub mod analyze;
pub mod antipattern;
pub mod audit;
pub mod cli;
pub mod cmd;
pub mod config;
pub mod coverage;
pub mod ctx;
pub mod design;
pub mod doc;
pub mod docops;
pub mod errors;
pub mod errors_table;
pub mod gate;
pub mod gates;
pub mod git;
pub mod handoff;
pub mod hooks;
pub mod json;
pub mod md;
pub mod pattern;
pub mod project;
pub mod report;
pub mod run;
pub mod stack;
pub mod state;
pub mod story;
pub mod time;
pub mod trust;
pub mod ypath;

pub use errors::{CliError, Diag};
pub use json::Envelope;

use cli::{Cli, Command};
use std::io::Write;

/// The semver this binary reports.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The three streams a subcommand may touch, so tests can drive `run` without
/// a process.
pub struct Io {
    /// Human output and the JSON envelope.
    pub out: Box<dyn Write>,
    /// Diagnostics.
    pub err: Box<dyn Write>,
    /// The hook JSON, when the caller supplies it instead of the real stdin.
    pub stdin: Option<String>,
}

/// A stream a test can read back after `run` returns.
pub type Captured = std::sync::Arc<std::sync::Mutex<Vec<u8>>>;

impl Io {
    /// An `Io` collecting both streams into vectors, for tests.
    pub fn capturing() -> (Self, Captured, Captured) {
        use std::sync::{Arc, Mutex};
        let out = Arc::new(Mutex::new(Vec::new()));
        let err = Arc::new(Mutex::new(Vec::new()));
        let io = Io {
            out: Box::new(Shared(out.clone())),
            err: Box::new(Shared(err.clone())),
            stdin: None,
        };
        (io, out, err)
    }

    /// Read the hook payload: the injected string when a test supplied one,
    /// else the real stdin to EOF.
    pub fn read_stdin(&mut self) -> Result<String, CliError> {
        use std::io::Read;
        if let Some(s) = self.stdin.take() {
            return Ok(s);
        }
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| CliError::io("reading", "stdin", &e))?;
        Ok(buf)
    }
}

struct Shared(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

impl Write for Shared {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut g = self
            .0
            .lock()
            .map_err(|_| std::io::Error::other("poisoned"))?;
        g.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// What a subcommand produced: the human lines it wants printed, the `data`
/// object for the envelope, and any warnings it collected.
#[derive(Debug, Default)]
pub struct Outcome {
    /// Human-readable stdout lines.
    pub human: Vec<String>,
    /// The per-subcommand `data` object.
    pub data: serde_json::Value,
    /// Warnings, emitted on stderr and in the envelope.
    pub warnings: Vec<Diag>,
    /// Error diagnostics a subcommand collected while still returning `Ok`
    /// with a non-zero `exit`, so the envelope's `errors[]` is populated on a
    /// failing `gate check` or a refusing `commit` the way the spec's worked
    /// example shows.
    pub errors: Vec<Diag>,
    /// `degraded` for the envelope.
    pub degraded: bool,
    /// The resolved project root for the envelope.
    pub project: String,
    /// An exit code the subcommand fixes itself, overriding the default 0.
    pub exit: Option<i32>,
    /// Bytes written to stdout verbatim, for `doc load`, which prints a file
    /// byte for byte with no added header.
    pub raw: Option<Vec<u8>>,
    /// Lines written to stderr verbatim, with no `DFA-` prefix. `commit`
    /// carries a refusing git hook's own stderr through here, which the spec
    /// asks for and the error table defines no code for.
    pub stderr: Vec<String>,
    /// The Claude Code hook decision object. When it is set and `--json` is
    /// not, `run` writes this object to stdout and nothing else, because the
    /// harness parses stdout that starts with `{` and a second line of text
    /// makes the parse a non-blocking error. Under `--json` it is never
    /// written: the DevForgeAI envelope is also a `{`-shaped object and the
    /// harness would reject it on schema.
    pub hook_json: Option<serde_json::Value>,
}

impl Outcome {
    /// An outcome carrying a `data` object and nothing else.
    pub fn data(v: serde_json::Value) -> Self {
        Outcome {
            data: v,
            ..Default::default()
        }
    }
}

/// Dispatch one parsed invocation and return the process exit code.
pub fn run(mut args: Cli, io: &mut Io) -> i32 {
    let at = time::now_rfc3339();

    if args.version {
        if args.json {
            let mut env = Envelope::new("version", at);
            env.data = serde_json::json!({
                "version": VERSION,
                "revision": revision_short(),
            });
            let _ = json::emit(&env, &mut io.out);
            let _ = io.out.flush();
            return 0;
        }
        let _ = writeln!(io.out, "devforgeai {} ({})", VERSION, revision_short());
        return 0;
    }

    let Some(command) = args.command.take() else {
        let diag = Diag::new("DFA-E011", "'devforgeai' requires <subcommand>");
        if args.json {
            let mut env = Envelope::new("any", at).with_exit(3);
            env.push_diag(&diag);
            let _ = json::emit(&env, &mut io.out);
            let _ = io.out.flush();
            return 3;
        }
        let _ = writeln!(io.err, "{}", diag.render("any"));
        return 3;
    };

    let name = command.name();
    // The root is resolved here as well as inside `dispatch`, so the error
    // branch can report the project it failed inside. `init`, `trust pin` and
    // `trust verify` resolve none by design.
    let resolved_root = resolved_root(&command, &args);
    let result = dispatch(&command, &args, io);

    match result {
        Ok(outcome) => {
            let exit = outcome.exit.unwrap_or(0);
            if args.json {
                let mut env = Envelope::new(name, at).with_exit(exit);
                env.degraded = outcome.degraded;
                env.project = outcome.project.clone();
                env.data = outcome.data.clone();
                for d in &outcome.errors {
                    env.push_diag(d);
                }
                for w in &outcome.warnings {
                    env.push_diag(w);
                }
                let _ = json::emit(&env, &mut io.out);
            } else if let Some(hook_json) = &outcome.hook_json {
                // The hook decision object is the whole of stdout: `human` and
                // `raw` are suppressed beside it.
                if let Ok(text) = serde_json::to_string(hook_json) {
                    let _ = writeln!(io.out, "{text}");
                }
            } else if let Some(raw) = &outcome.raw {
                if !args.quiet {
                    let _ = io.out.write_all(raw);
                }
            } else if !args.quiet {
                for line in &outcome.human {
                    let _ = writeln!(io.out, "{line}");
                }
            }
            // Diagnostics reach stderr in both output modes: a caller that
            // pipes stdout to a parser still watches stderr for them.
            for d in &outcome.errors {
                let _ = writeln!(io.err, "{}", d.render(name));
            }
            for w in &outcome.warnings {
                let _ = writeln!(io.err, "{}", w.render(name));
            }
            for line in &outcome.stderr {
                let _ = writeln!(io.err, "{line}");
            }
            let _ = io.out.flush();
            let _ = io.err.flush();
            exit
        }
        Err(e) => {
            let exit = e.exit();
            if args.json {
                let mut env = Envelope::new(name, at).with_exit(exit);
                env.project = resolved_root.unwrap_or_default();
                if let Some(d) = e.diag() {
                    env.push_diag(d);
                }
                let _ = json::emit(&env, &mut io.out);
            }
            if let Some(d) = e.diag() {
                let _ = writeln!(io.err, "{}", d.render(name));
            }
            let _ = io.out.flush();
            let _ = io.err.flush();
            exit
        }
    }
}

/// The project root this invocation resolves, for the envelope's `project`
/// field on the error branch. `None` for the three commands that resolve none
/// and whenever discovery fails.
fn resolved_root(command: &Command, args: &Cli) -> Option<String> {
    use cli::TrustCmd;
    match command {
        Command::Init(_) | Command::Trust(TrustCmd::Pin { .. }) => return None,
        Command::Trust(TrustCmd::Verify { .. }) => return None,
        _ => {}
    }
    let cwd = std::env::current_dir().ok()?;
    project::resolve_root(&cwd, args.project.as_deref())
        .ok()
        .map(|p| p.display().to_string())
}

fn dispatch(command: &Command, args: &Cli, io: &mut Io) -> Result<Outcome, CliError> {
    use cli::{
        AntipatternCmd, ConfigCmd, ContextCmd, DesignCmd, DocCmd, ExploreCmd, GateCmd, HookCmd,
        PhaseCmd, ReportCmd, StackCmd, StoryCmd, TrustCmd, WorktreeCmd,
    };

    // `init`, `trust pin` and `trust verify` resolve no project root.
    match command {
        Command::Init(a) => return cmd::init::run(args.project.as_deref(), a),
        Command::Trust(TrustCmd::Pin { binary, framework }) => {
            return cmd::trust::pin(binary.as_deref(), framework.as_deref())
        }
        Command::Trust(TrustCmd::Verify { binary }) => {
            return cmd::trust::verify(binary.as_deref(), args.quiet)
        }
        Command::Trust(TrustCmd::Digest { framework, binary }) => {
            return cmd::trust::digest(framework.as_deref(), binary.as_deref())
        }
        _ => {}
    }

    let cwd = std::env::current_dir()
        .map_err(|e| CliError::io("reading", "the working directory", &e))?;
    let root = project::resolve_root(&cwd, args.project.as_deref())?;
    let mut ctx = ctx::Ctx::new(root);

    match command {
        Command::Init(_) | Command::Trust(_) => unreachable!("handled above"),
        Command::Stack(StackCmd::Detect { dry_run }) => cmd::stack::detect(&mut ctx, *dry_run),
        Command::Gate(GateCmd::Require { phase, id }) => cmd::gate::require(&mut ctx, phase, id),
        Command::Gate(GateCmd::Check(a)) => cmd::gate::check(&mut ctx, a),
        Command::Doc(DocCmd::Validate(a)) => {
            let stdin = if a.stdin_content {
                Some(io.read_stdin()?)
            } else {
                None
            };
            cmd::doc::validate(&mut ctx, a, stdin)
        }
        Command::Doc(DocCmd::Load { name, id }) => cmd::doc::load(&mut ctx, name, id),
        Command::Doc(DocCmd::Accept(a)) => cmd::doc::accept(&mut ctx, a),
        Command::Doc(DocCmd::Reopen(a)) => cmd::doc::reopen(&mut ctx, a),
        Command::Hook(HookCmd::Install(a)) => cmd::hook::install(&mut ctx, a),
        Command::Hook(HookCmd::Run { event }) => {
            let payload = io.read_stdin()?;
            cmd::hook::run(&mut ctx, event, &payload)
        }
        Command::Report(ReportCmd::Show { id, phase, check }) => {
            cmd::report::show(&mut ctx, id, phase, check.as_deref())
        }
        Command::Report(ReportCmd::Ingest(a)) => {
            let stdin = if a.source == "-" {
                Some(io.read_stdin()?)
            } else {
                None
            };
            cmd::report::ingest(&mut ctx, a, stdin)
        }
        Command::Report(ReportCmd::Note(a)) => cmd::report::note(&mut ctx, a),
        Command::Phase(PhaseCmd::Set {
            phase,
            id,
            remedy,
            epic,
        }) => cmd::phase::set(&mut ctx, phase, id, remedy.as_deref(), epic.as_deref()),
        Command::Handoff(a) => cmd::handoff::run(&mut ctx, a.phase.as_deref(), a.id.as_deref()),

        // Outside this milestone: each refuses with the not-implemented code at
        // exit 5 rather than returning a wrong answer.
        Command::Story(StoryCmd::Validate { id, scope }) => {
            cmd::story::validate(&mut ctx, id.as_deref(), scope)
        }
        Command::Story(StoryCmd::Files(a)) => cmd::story::files(&mut ctx, a),
        Command::Story(StoryCmd::List { status, sprint }) => {
            cmd::story::list(&mut ctx, status.as_deref(), sprint.as_deref())
        }
        Command::Explore(ExploreCmd::Prune { id }) => cmd::explore::prune(&mut ctx, id),
        Command::Design(DesignCmd::Lint { paths, tokens }) => {
            cmd::design::lint(&mut ctx, paths, *tokens)
        }
        Command::Context(ContextCmd::Audit) => cmd::context::run(&mut ctx),
        Command::Report(ReportCmd::Aggregate {
            id,
            since,
            session_root,
        }) => cmd::report::aggregate(
            &mut ctx,
            id.as_deref(),
            since.as_deref(),
            session_root.as_deref(),
        ),
        Command::Worktree(WorktreeCmd::Ensure { id }) => cmd::worktree::ensure(&mut ctx, id),
        Command::Worktree(WorktreeCmd::List) => cmd::worktree::list(&mut ctx),
        Command::Worktree(WorktreeCmd::Remove { id, force }) => {
            cmd::worktree::remove(&mut ctx, id, *force)
        }
        Command::Config(ConfigCmd::Get { key, stack }) => {
            cmd::config::get(&mut ctx, key, stack.as_deref())
        }
        Command::Antipattern(AntipatternCmd::Scan {
            id,
            paths,
            min_severity,
        }) => cmd::antipattern::scan(&mut ctx, id.as_deref(), paths.as_deref(), min_severity),
        Command::Commit(a) => cmd::commit::run(&mut ctx, a),
    }
}

/// The first twelve characters of `cli/REVISION` line 1, or `unversioned`.
fn revision_short() -> String {
    "unversioned".chars().take(12).collect()
}
