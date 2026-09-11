//! One module per subcommand. Each exposes a `run` taking the parsed arguments
//! and returning the `Outcome` the envelope and the human output are built
//! from.

pub mod antipattern;
pub mod commit;
pub mod config;
pub mod context;
pub mod design;
pub mod doc;
pub mod explore;
pub mod gate;
pub mod handoff;
pub mod hook;
pub mod init;
pub mod phase;
pub mod report;
pub mod stack;
pub mod story;
pub mod trust;
pub mod worktree;
