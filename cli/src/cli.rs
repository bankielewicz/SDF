//! The clap derive types. One struct per subcommand, in the grammar
//! `## CLI calls` fixes.

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

/// `devforgeai [--json] [--project <path>] [--quiet] <subcommand>`
#[derive(Debug, Parser)]
#[command(
    name = "devforgeai",
    version,
    disable_version_flag = true,
    about = "The enforcement core of the DevForgeAI framework"
)]
pub struct Cli {
    /// One JSON envelope on stdout, no human text on stdout.
    #[arg(long, global = true)]
    pub json: bool,
    /// Project root override; `init` defaults to the working directory.
    #[arg(long, global = true, value_name = "path")]
    pub project: Option<PathBuf>,
    /// Suppress human stdout; stderr is unaffected.
    #[arg(long, global = true)]
    pub quiet: bool,
    /// Print `devforgeai <semver> (<revision>)` and exit 0.
    #[arg(long)]
    pub version: bool,
    /// The subcommand.
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Every subcommand of the binary.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create `.devforgeai/`, copy skills, merge hooks, detect the stack.
    Init(InitArgs),
    /// Stack detection.
    #[command(subcommand)]
    Stack(StackCmd),
    /// Gate evaluation.
    #[command(subcommand)]
    Gate(GateCmd),
    /// Document operations.
    #[command(subcommand)]
    Doc(DocCmd),
    /// Story operations.
    #[command(subcommand)]
    Story(StoryCmd),
    /// Explore-phase operations.
    #[command(subcommand)]
    Explore(ExploreCmd),
    /// Design-token linting.
    #[command(name = "design", subcommand)]
    Design(DesignCmd),
    /// The six context files.
    #[command(subcommand)]
    Context(ContextCmd),
    /// Hook installation and dispatch.
    #[command(subcommand)]
    Hook(HookCmd),
    /// Report operations.
    #[command(subcommand)]
    Report(ReportCmd),
    /// The binary trust pin.
    #[command(subcommand)]
    Trust(TrustCmd),
    /// Phase state.
    #[command(subcommand)]
    Phase(PhaseCmd),
    /// Git worktrees for concurrent stories.
    #[command(subcommand)]
    Worktree(WorktreeCmd),
    /// Configuration reads.
    #[command(subcommand)]
    Config(ConfigCmd),
    /// Anti-pattern detection.
    #[command(name = "antipattern", subcommand)]
    Antipattern(AntipatternCmd),
    /// Print the conventions section 6 handoff block.
    Handoff(HandoffArgs),
    /// Commit the declared file set of a story.
    Commit(CommitArgs),
}

/// `devforgeai init`
#[derive(Debug, Args)]
pub struct InitArgs {
    /// Run the brownfield analysis after `stack detect`.
    #[arg(long)]
    pub analyze: bool,
    /// Overwrite an existing `.devforgeai/` and existing git hooks.
    #[arg(long)]
    pub force: bool,
    /// The DevForgeAI repo root supplying skills, agents, commands, hooks.
    #[arg(long, value_name = "path")]
    pub from: Option<PathBuf>,
    /// Skip the settings merge and the git hooks.
    #[arg(long)]
    pub no_hooks: bool,
}

/// `devforgeai stack ...`
#[derive(Debug, Subcommand)]
pub enum StackCmd {
    /// Write `config.toml` with the detected stacks.
    Detect {
        /// Print the result and write nothing.
        #[arg(long)]
        dry_run: bool,
    },
}

/// `devforgeai gate ...`
#[derive(Debug, Subcommand)]
pub enum GateCmd {
    /// Exit 0 when the predecessor gate of `<phase>` passed for `<id>`.
    Require {
        /// The phase whose predecessor is tested.
        phase: String,
        /// The gate subject.
        id: String,
    },
    /// Evaluate the gate in `gates.toml`.
    Check(GateCheckArgs),
}

/// `devforgeai gate check`
#[derive(Debug, Args)]
pub struct GateCheckArgs {
    /// The gate evaluated.
    #[arg(long, value_name = "phase")]
    pub phase: String,
    /// The gate subject; defaults to `state.toml` `[active].<phase>`.
    #[arg(long, value_name = "id")]
    pub id: Option<String>,
    /// Evaluate the four command kinds only and leave `state.toml` untouched.
    #[arg(long)]
    pub partial: bool,
    /// Execute no command.
    #[arg(long)]
    pub no_run: bool,
}

/// `devforgeai doc ...`
#[derive(Debug, Subcommand)]
pub enum DocCmd {
    /// Frontmatter, ids, cross-references, status enum, producer match.
    Validate(DocValidateArgs),
    /// Print a named upstream document.
    Load {
        /// The doc type.
        name: String,
        /// The selector; `-` where the row ignores it.
        id: String,
    },
    /// Move `requirements.yaml` to accepted.
    Accept(DocAcceptArgs),
    /// Re-open cited requirements.
    Reopen(DocReopenArgs),
}

/// `devforgeai doc validate`
#[derive(Debug, Args)]
pub struct DocValidateArgs {
    /// The documents validated.
    pub paths: Vec<PathBuf>,
    /// Validate every matching file under `.devforgeai/`.
    #[arg(long)]
    pub all: bool,
    /// Print the next free ID for a prefix and exit.
    #[arg(long, value_name = "prefix")]
    pub allocate: Option<String>,
    /// Run the frontmatter and producer checks only, over exactly one path.
    #[arg(long)]
    pub producer_check: bool,
    /// Read the document body from stdin instead of the file.
    #[arg(long)]
    pub stdin_content: bool,
}

/// `devforgeai doc accept`
#[derive(Debug, Args)]
pub struct DocAcceptArgs {
    /// The document name; `requirements` is the only accepted value.
    pub name: String,
    /// The `IDEA-nnn` the document carries.
    #[arg(long, value_name = "IDEA-nnn")]
    pub id: String,
}

/// `devforgeai doc reopen`
#[derive(Debug, Args)]
pub struct DocReopenArgs {
    /// The document name; `requirements` is the only accepted value.
    pub name: String,
    /// The `IDEA-nnn` the document carries.
    #[arg(long, value_name = "IDEA-nnn")]
    pub id: String,
    /// The cited `REQ-nnn` and `UI-nnn` ids.
    #[arg(long, value_name = "ID,ID")]
    pub ids: String,
    /// The phase the send-back came from.
    #[arg(long, value_name = "phase")]
    pub from: String,
}

/// `devforgeai story ...`
#[derive(Debug, Subcommand)]
pub enum StoryCmd {
    /// ACs testable, REQ references resolve, no dependency cycle.
    Validate {
        /// Restrict the run to one story.
        id: Option<String>,
        /// `active`, `sprint`, or `all`.
        #[arg(long, default_value = "active")]
        scope: String,
    },
    /// The declared file set of a story.
    Files(StoryFilesArgs),
    /// One line per story.
    List {
        /// Filter to the listed status values.
        #[arg(long)]
        status: Option<String>,
        /// Filter to the ids under a sprint.
        #[arg(long)]
        sprint: Option<String>,
    },
}

/// `devforgeai story files`
#[derive(Debug, Args)]
pub struct StoryFilesArgs {
    /// Test one path against the declared set.
    #[arg(long, value_name = "path")]
    pub check: Option<PathBuf>,
    /// Print the declared set.
    #[arg(long)]
    pub list: bool,
    /// Test every changed path against the declared set.
    #[arg(long)]
    pub diff: bool,
    /// The story; defaults to `state.toml` `[active].build`.
    #[arg(long, value_name = "STORY-nnn")]
    pub id: Option<String>,
    /// The diff base.
    #[arg(long, value_name = "ref")]
    pub base: Option<String>,
}

/// `devforgeai explore ...`
#[derive(Debug, Subcommand)]
pub enum ExploreCmd {
    /// Remove `.explore-prototype/` after a kill or promote decision.
    Prune {
        /// The idea.
        #[arg(long, value_name = "IDEA-nnn")]
        id: String,
    },
}

/// `devforgeai design ...`
#[derive(Debug, Subcommand)]
pub enum DesignCmd {
    /// Colours and type in frontend files resolve to `brand/tokens.json`.
    Lint {
        /// The files linted.
        paths: Vec<PathBuf>,
        /// Check the token file's own shape instead.
        #[arg(long)]
        tokens: bool,
    },
}

/// `devforgeai context ...`
#[derive(Debug, Subcommand)]
pub enum ContextCmd {
    /// The eight CA checks over the six context files.
    Audit,
}

/// `devforgeai hook ...`
#[derive(Debug, Subcommand)]
pub enum HookCmd {
    /// Write the Claude hooks and the three git hooks.
    Install(HookInstallArgs),
    /// The stdin JSON dispatcher.
    Run {
        /// The event name.
        event: String,
    },
}

/// `devforgeai hook install`
#[derive(Debug, Args)]
pub struct HookInstallArgs {
    /// Replace a git hook this binary did not write.
    #[arg(long)]
    pub force: bool,
    /// Merge the settings and skip the git hooks.
    #[arg(long)]
    pub claude_only: bool,
    /// Write the git hooks and skip the settings.
    #[arg(long)]
    pub git_only: bool,
}

/// `devforgeai report ...`
#[derive(Debug, Subcommand)]
pub enum ReportCmd {
    /// Print a report.
    Show {
        /// The report subject.
        id: String,
        /// The report phase.
        phase: String,
        /// Print one check entry.
        #[arg(long, value_name = "check-id")]
        check: Option<String>,
    },
    /// Write a verifier block into a report.
    Ingest(ReportIngestArgs),
    /// Write a skill's note under its own report key.
    Note(ReportNoteArgs),
    /// Collect a window of reports.
    Aggregate {
        /// The window subject.
        id: Option<String>,
        /// The window start.
        #[arg(long, value_name = "YYYY-MM-DD")]
        since: Option<String>,
        /// The session root override.
        #[arg(long, value_name = "path")]
        session_root: Option<PathBuf>,
    },
}

/// `devforgeai report ingest`
#[derive(Debug, Args)]
pub struct ReportIngestArgs {
    /// The kebab-case subagent name.
    pub subagent: String,
    /// A file path, or `-` for stdin.
    pub source: String,
    /// The report subject.
    #[arg(long)]
    pub id: Option<String>,
    /// The report phase.
    #[arg(long)]
    pub phase: Option<String>,
}

/// `devforgeai report note`
#[derive(Debug, Args)]
pub struct ReportNoteArgs {
    /// The report subject.
    pub id: String,
    /// The report phase.
    pub phase: String,
    /// The top-level report key written.
    #[arg(long)]
    pub key: String,
    /// The YAML file holding one mapping.
    #[arg(long, value_name = "path")]
    pub file: PathBuf,
}

/// `devforgeai trust ...`
#[derive(Debug, Subcommand)]
pub enum TrustCmd {
    /// Pin the binary's SHA-256.
    Pin {
        /// The binary pinned; defaults to the running executable.
        #[arg(long, value_name = "path")]
        binary: Option<PathBuf>,
        /// The framework repo root.
        #[arg(long, value_name = "path")]
        framework: Option<PathBuf>,
    },
    /// Verify the running binary against its pin.
    Verify {
        /// The binary verified; defaults to the running executable.
        #[arg(long, value_name = "path")]
        binary: Option<PathBuf>,
    },
    /// Print the two release digests this binary computes, for the release
    /// step that writes `cli/REVISION` and `cli/DIGEST`. Hidden: it is a build
    /// tool, not part of the nine-command surface, and it exists so the files
    /// are written by the same code path that later reads them, rather than by
    /// an external script that can drift from it.
    #[command(hide = true)]
    Digest {
        /// The framework repo root holding `cli/`.
        #[arg(long, value_name = "path")]
        framework: Option<PathBuf>,
        /// The binary hashed for the `DIGEST` line; defaults to the running
        /// executable.
        #[arg(long, value_name = "path")]
        binary: Option<PathBuf>,
    },
}

/// `devforgeai phase ...`
#[derive(Debug, Subcommand)]
pub enum PhaseCmd {
    /// Update `state.toml`.
    Set {
        /// The phase set.
        phase: String,
        /// The active id of that phase.
        #[arg(long)]
        id: String,
        /// The cited ids of a send-back.
        #[arg(long, value_name = "ID,ID")]
        remedy: Option<String>,
        /// The epic a `plan` run is for, when `stories/sprint.yaml` does not
        /// exist yet.
        #[arg(long, value_name = "EPIC-nnn")]
        epic: Option<String>,
    },
}

/// `devforgeai worktree ...`
#[derive(Debug, Subcommand)]
pub enum WorktreeCmd {
    /// Create the worktree of a story when it is absent.
    Ensure {
        /// The story.
        id: String,
    },
    /// One line per worktree.
    List,
    /// Remove the worktree of a story.
    Remove {
        /// The story.
        id: String,
        /// Remove despite uncommitted or unmerged work.
        #[arg(long)]
        force: bool,
    },
}

/// `devforgeai config ...`
#[derive(Debug, Subcommand)]
pub enum ConfigCmd {
    /// Print one configuration value.
    Get {
        /// The key.
        key: String,
        /// Select a stack by id.
        #[arg(long)]
        stack: Option<String>,
    },
}

/// `devforgeai antipattern ...`
#[derive(Debug, Subcommand)]
pub enum AntipatternCmd {
    /// Apply the anti-pattern index to a candidate set.
    Scan {
        /// The story whose declared set is scanned.
        #[arg(long)]
        id: Option<String>,
        /// The paths scanned.
        #[arg(long)]
        paths: Option<String>,
        /// The lowest severity reported.
        #[arg(long, default_value = "high")]
        min_severity: String,
    },
}

/// `devforgeai handoff`
#[derive(Debug, Args)]
pub struct HandoffArgs {
    /// The phase rendered; defaults to `[current].phase`.
    #[arg(long)]
    pub phase: Option<String>,
    /// The subject; defaults to `[active].<phase>`.
    #[arg(long)]
    pub id: Option<String>,
}

/// `devforgeai commit`
#[derive(Debug, Args)]
pub struct CommitArgs {
    /// The story.
    pub id: String,
    /// The commit message.
    #[arg(short = 'm', long)]
    pub message: String,
    /// The paths staged.
    #[arg(long)]
    pub paths: Option<String>,
}

impl Command {
    /// The `command` value of the JSON envelope: the subcommand name with its
    /// space, as the spec writes it.
    pub fn name(&self) -> &'static str {
        match self {
            Command::Init(_) => "init",
            Command::Stack(StackCmd::Detect { .. }) => "stack detect",
            Command::Gate(GateCmd::Require { .. }) => "gate require",
            Command::Gate(GateCmd::Check(_)) => "gate check",
            Command::Doc(DocCmd::Validate(_)) => "doc validate",
            Command::Doc(DocCmd::Load { .. }) => "doc load",
            Command::Doc(DocCmd::Accept(_)) => "doc accept",
            Command::Doc(DocCmd::Reopen(_)) => "doc reopen",
            Command::Story(StoryCmd::Validate { .. }) => "story validate",
            Command::Story(StoryCmd::Files(_)) => "story files",
            Command::Story(StoryCmd::List { .. }) => "story list",
            Command::Explore(ExploreCmd::Prune { .. }) => "explore prune",
            Command::Design(DesignCmd::Lint { .. }) => "design lint",
            Command::Context(ContextCmd::Audit) => "context audit",
            Command::Hook(HookCmd::Install(_)) => "hook install",
            Command::Hook(HookCmd::Run { .. }) => "hook run",
            Command::Report(ReportCmd::Show { .. }) => "report show",
            Command::Report(ReportCmd::Ingest(_)) => "report ingest",
            Command::Report(ReportCmd::Note(_)) => "report note",
            Command::Report(ReportCmd::Aggregate { .. }) => "report aggregate",
            Command::Trust(TrustCmd::Pin { .. }) => "trust pin",
            Command::Trust(TrustCmd::Verify { .. }) => "trust verify",
            Command::Trust(TrustCmd::Digest { .. }) => "trust digest",
            Command::Phase(PhaseCmd::Set { .. }) => "phase set",
            Command::Worktree(WorktreeCmd::Ensure { .. }) => "worktree ensure",
            Command::Worktree(WorktreeCmd::List) => "worktree list",
            Command::Worktree(WorktreeCmd::Remove { .. }) => "worktree remove",
            Command::Config(ConfigCmd::Get { .. }) => "config get",
            Command::Antipattern(AntipatternCmd::Scan { .. }) => "antipattern scan",
            Command::Handoff(_) => "handoff",
            Command::Commit(_) => "commit",
        }
    }
}
