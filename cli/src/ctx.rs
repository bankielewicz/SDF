//! `Ctx`, the resolved project every subcommand works against: the root, and
//! the three project files loaded on demand.

use crate::config::Config;
use crate::errors::CliError;
use crate::gates::Gates;
use crate::state::State;
use crate::{config, gates, state};
use std::path::{Path, PathBuf};

/// A resolved project. The three files load lazily, so `init`, `trust pin`,
/// and `trust verify` never touch them.
pub struct Ctx {
    /// The project root.
    pub root: PathBuf,
    config: Option<Config>,
    gates: Option<Gates>,
    state: Option<State>,
}

impl Ctx {
    /// A context over an already resolved root.
    pub fn new(root: PathBuf) -> Self {
        Ctx {
            root,
            config: None,
            gates: None,
            state: None,
        }
    }

    /// `.devforgeai/` inside the root.
    pub fn dot(&self) -> PathBuf {
        crate::project::dot(&self.root)
    }

    /// Resolve a `gates.toml`-style relative path against the root.
    pub fn doc_path(&self, rel: &str) -> PathBuf {
        crate::project::resolve_doc_path(&self.root, rel)
    }

    /// Render a path relative to the root in forward-slash form.
    pub fn rel(&self, p: &Path) -> String {
        crate::project::rel_display(&self.root, p)
    }

    /// Where this project's *source* lives right now.
    ///
    /// The root holds `.devforgeai/` — the state, the config, the gates, the
    /// documents, the reports — and is the same directory whether a command
    /// runs from the main checkout or from inside a worktree. The source is
    /// the half that moves: during a Build run in a worktree the code and the
    /// tests being changed are there, so that is where a test command runs and
    /// what a source path resolves against.
    ///
    /// The two are the same directory whenever no worktree is open, which is
    /// every phase but Build and most Build runs too.
    pub fn source_root(&mut self) -> PathBuf {
        // An unreadable state resolves no worktree, and the root is the only
        // other answer. The error itself surfaces from every command that
        // reads state for its own work, so it is not swallowed, only not
        // raised twice.
        let Ok(state) = self.state() else {
            return self.root.clone();
        };
        let active = state.active.build.clone();
        if active.is_empty() {
            return self.root.clone();
        }
        self.worktree_path(&active)
            .unwrap_or_else(|| self.root.clone())
    }

    /// The directory of the worktree registered for `story`, when it exists.
    ///
    /// A worktree the operator removed by hand leaves its entry behind, and
    /// resolving source paths against a directory that is not there would
    /// report every file missing rather than reading the checkout that is.
    pub fn worktree_path(&mut self, story: &str) -> Option<PathBuf> {
        let rel = self
            .state()
            .ok()?
            .worktree
            .iter()
            .find(|w| w.story == story)
            .map(|w| w.path.clone())?;
        let candidate = self.root.join(&rel);
        candidate.is_dir().then_some(candidate)
    }

    /// The story whose registered worktree contains `path`, if any.
    ///
    /// A write into `wt/STORY-015/` is STORY-015's whatever the current run is
    /// on, so the declared-set check reads the story the path belongs to
    /// rather than the one the session happens to be building.
    pub fn worktree_story_of(&mut self, path: &Path) -> Option<String> {
        let root = self.root.clone();
        let key = crate::hooks::run::hook_path_key(&path.to_string_lossy());
        let entries: Vec<(String, String)> = self
            .state()
            .ok()?
            .worktree
            .iter()
            .map(|w| (w.story.clone(), w.path.clone()))
            .collect();
        for (story, rel) in entries {
            if rel.is_empty() {
                continue;
            }
            let dir = root.join(&rel);
            let prefix = crate::hooks::run::hook_path_key(&dir.to_string_lossy());
            // The key carries a trailing slash, so this is a directory-prefix
            // test rather than a partial-name match.
            if key.starts_with(prefix.trim_end_matches('/')) && key.len() > prefix.len() - 1 {
                return Some(story);
            }
        }
        None
    }

    /// `config.toml`, loaded once and validated against the compiled floors.
    pub fn config(&mut self) -> Result<&Config, CliError> {
        if self.config.is_none() {
            self.config = Some(config::load(&self.root)?);
        }
        Ok(self.config.as_ref().expect("just loaded"))
    }

    /// `gates.toml`, loaded once and validated against the compiled minimums.
    pub fn gates(&mut self) -> Result<&Gates, CliError> {
        if self.gates.is_none() {
            self.gates = Some(gates::load(&self.root)?);
        }
        Ok(self.gates.as_ref().expect("just loaded"))
    }

    /// `state.toml`, loaded once.
    pub fn state(&mut self) -> Result<&State, CliError> {
        if self.state.is_none() {
            self.state = Some(state::load(&self.root)?);
        }
        Ok(self.state.as_ref().expect("just loaded"))
    }

    /// A mutable borrow of `state.toml`, loaded first when it is not already.
    pub fn state_mut(&mut self) -> Result<&mut State, CliError> {
        if self.state.is_none() {
            self.state = Some(state::load(&self.root)?);
        }
        Ok(self.state.as_mut().expect("just loaded"))
    }

    /// Write `state.toml` back and drop the cached copy.
    pub fn store_state(&mut self) -> Result<(), CliError> {
        if let Some(mut s) = self.state.take() {
            state::store(&self.root, &mut s)?;
            self.state = Some(s);
        }
        Ok(())
    }

    /// `degraded` for the JSON envelope, `false` when no config loaded.
    pub fn degraded(&self) -> bool {
        self.config.as_ref().map(|c| c.degraded).unwrap_or(false)
    }
}
