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
