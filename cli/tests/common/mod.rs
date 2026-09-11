//! A project skeleton in a temporary directory, shared by the integration
//! tests. No test reads or writes the real home directory: every fixture lives
//! inside its own `TempDir` and every command takes that root explicitly.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// A temporary project with `.devforgeai/` laid out as `init` lays it out.
pub struct Project {
    /// Kept alive so the directory outlives the test.
    pub dir: tempfile::TempDir,
}

impl Project {
    /// An empty project with the directory tree, `config.toml`, `gates.toml`
    /// and `state.toml` in place.
    pub fn new() -> Project {
        let dir = tempfile::tempdir().expect("temp dir");
        let p = Project { dir };
        let dot = p.dot();
        for sub in devforgeai::project::SUBDIRS {
            std::fs::create_dir_all(dot.join(sub)).expect("mkdir");
        }
        p.write_config(&devforgeai::config::Config {
            degraded: false,
            stack: vec![devforgeai::config::Stack {
                id: "rust".into(),
                // What `init` leaves behind: `init` runs `stack detect`, and
                // every entry detection writes is its own to refresh. A
                // hand-written entry carries `manual` and is never replaced.
                source: devforgeai::config::STACK_DETECTED.into(),
                markers: vec!["Cargo.toml".into()],
                package_manager: "cargo".into(),
                source_roots: vec!["src".into()],
                test_command: "cargo test".into(),
                coverage_format: "lcov".into(),
                timeout_secs: 900,
                ..Default::default()
            }],
            generated_at: "2026-09-10T14:02:11Z".into(),
            ..Default::default()
        });
        p.write(".devforgeai/gates.toml", devforgeai::gates::DEFAULT_GATES);
        p.write(
            ".devforgeai/state.toml",
            &devforgeai::state::initial_toml("2026-09-10T14:02:11Z"),
        );
        p
    }

    /// A project with no `.devforgeai/` at all.
    pub fn bare() -> Project {
        Project {
            dir: tempfile::tempdir().expect("temp dir"),
        }
    }

    /// The project root.
    pub fn root(&self) -> &Path {
        self.dir.path()
    }

    /// `.devforgeai/` inside the root.
    pub fn dot(&self) -> PathBuf {
        self.dir.path().join(".devforgeai")
    }

    /// A fresh context over this project.
    pub fn ctx(&self) -> devforgeai::ctx::Ctx {
        devforgeai::ctx::Ctx::new(self.root().to_path_buf())
    }

    /// Write a project-relative file, creating parent directories.
    pub fn write(&self, rel: &str, content: &str) -> PathBuf {
        let path = self.dir.path().join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("mkdir");
        }
        std::fs::write(&path, content).expect("write");
        path
    }

    /// Read a project-relative file.
    pub fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.dir.path().join(rel))
            .unwrap_or_else(|e| panic!("reading {rel}: {e}"))
    }

    /// True when a project-relative path exists.
    pub fn exists(&self, rel: &str) -> bool {
        self.dir.path().join(rel).exists()
    }

    /// Replace `config.toml`.
    pub fn write_config(&self, cfg: &devforgeai::config::Config) {
        let text = devforgeai::config::to_toml(cfg).expect("serialise config");
        self.write(".devforgeai/config.toml", &text);
    }

    /// Replace `state.toml`, keeping the schema valid.
    pub fn write_state(&self, state: &devforgeai::state::State) {
        let text = devforgeai::state::to_toml(state).expect("serialise state");
        self.write(".devforgeai/state.toml", &text);
    }

    /// Load `state.toml`.
    pub fn state(&self) -> devforgeai::state::State {
        devforgeai::state::load(self.root()).expect("load state")
    }

    /// Set `[current]` and the matching `[active]` key.
    pub fn set_phase(&self, phase: &str, id: &str) {
        let mut s = self.state();
        s.current.phase = phase.to_string();
        s.current.id = id.to_string();
        s.active.set(phase, id);
        self.write_state(&s);
    }
}

impl Default for Project {
    fn default() -> Self {
        Project::new()
    }
}

/// A valid story document.
pub fn story(id: &str, status: &str, consumes: &[&str], body: &str) -> String {
    format!(
        "---\nschema: devforgeai/story/1\nid: {id}\nphase: plan\nstatus: {status}\nproduced_by: planning-work\nconsumes: [{}]\nopen_questions: []\n---\n\n{body}",
        consumes.join(", ")
    )
}

/// A valid requirements document with one persona, one requirement, one epic.
pub fn requirements(id: &str, status: &str) -> String {
    format!(
        "schema: devforgeai/requirements/1\nid: {id}\nphase: discover\nstatus: {status}\nproduced_by: discovering-requirements\nconsumes: []\nopen_questions: []\nrevision: 1\naccepted_by: null\naccepted_at: null\npersonas:\n  - id: PERSONA-001\n    name: Shopper\nrequirements:\n  - id: REQ-001\n    actor: PERSONA-001\n    statement: The shopper places an order\n    rationale: revenue\n    acceptance_signal: an order row exists\n    priority: must\n    source: user\n    status: draft\n  - id: REQ-002\n    actor: PERSONA-001\n    statement: The shopper sees a receipt\n    rationale: trust\n    acceptance_signal: a receipt renders\n    priority: should\n    source: user\n    status: draft\nepics:\n  - id: EPIC-001\n    title: Checkout\n    requirements: [REQ-001, REQ-002]\n"
    )
}

/// A valid explore brief with three core flows.
///
/// The flows are defined as list items under `## Flow detail`. A Markdown table
/// cell is not one of the four ID definition forms, so the `## Core flows` table
/// alone would leave every `FLOW-nnn` an unresolved reference.
pub fn brief(id: &str, status: &str) -> String {
    format!(
        "---\nschema: devforgeai/explore-brief/1\nid: {id}\nphase: explore\nstatus: {status}\nproduced_by: exploring-ideas\nconsumes: []\nopen_questions: []\n---\n\n# Order checkout\n\n## Core flows\n\n| ID | Flow |\n|---|---|\n| FLOW-001 | Add to cart |\n| FLOW-002 | Pay |\n| FLOW-003 | Receipt |\n\n## Flow detail\n\n- FLOW-001: Add to cart\n- FLOW-002: Pay\n- FLOW-003: Receipt\n\n## Success signal\n\n| Signal | Target |\n|---|---|\n| Orders per day | 100 |\n"
    )
}

/// A valid explore decision.
pub fn decision(id: &str, kind: &str) -> String {
    format!(
        "schema: devforgeai/explore-decision/1\nid: {id}\nphase: explore\nstatus: recorded\nproduced_by: exploring-ideas\nconsumes: []\nopen_questions: []\ndecision: {kind}\ndecided_on: 2026-09-08\ncarry_forward: []\n"
    )
}

/// A CLI-written gate report.
pub fn report(id: &str, phase: &str, result: &str) -> String {
    let status = match result {
        "PASS" => "pass",
        "SEND_BACK" => "send_back",
        _ => "fail",
    };
    format!(
        "schema: devforgeai/report/1\nid: {id}\nphase: {phase}\nstatus: {status}\nproduced_by: devforgeai-cli\nconsumes: []\nopen_questions: []\nstarted_at: 2026-09-10T14:01:03Z\nfinished_at: 2026-09-10T14:02:11Z\ncli_version: 1.0.0\ndegraded: false\ngate:\n  result: {result}\n  send_back_to: ''\n  checks: []\nfindings: []\nhandoff: []\n"
    )
}

/// The process-wide lock guarding `DEVFORGEAI_NOW` and `DEVFORGEAI_HOME`.
///
/// Both are read through the process environment, which Rust's test harness
/// shares across threads, so a test that sets one takes this lock for as long
/// as it needs the value to hold.
pub static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Run `body` with `DEVFORGEAI_NOW` fixed, restoring the previous value.
pub fn with_now<T>(value: &str, body: impl FnOnce() -> T) -> T {
    let guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let previous = std::env::var_os("DEVFORGEAI_NOW");
    std::env::set_var("DEVFORGEAI_NOW", value);
    let out = body();
    match previous {
        Some(v) => std::env::set_var("DEVFORGEAI_NOW", v),
        None => std::env::remove_var("DEVFORGEAI_NOW"),
    }
    drop(guard);
    out
}

// --------------------------------------------------------- context fixtures

/// A context file that conforms to its fixed H2 list. `bodies` supplies the
/// text under the named sections; a section not named is left empty.
pub fn context_file(stem: &str, status: &str, bodies: &[(&str, &str)]) -> String {
    let sections = devforgeai::audit::CONTEXT_SECTIONS
        .iter()
        .find(|(s, _)| *s == stem)
        .map(|(_, h)| *h)
        .unwrap_or(&[]);
    let mut out = format!(
        "---\nschema: devforgeai/context-{stem}/1\nid: {stem}\nphase: constitute\nstatus: {status}\nproduced_by: establishing-context\nconsumes: []\nopen_questions: []\n---\n\n# {stem}\n"
    );
    for h in sections {
        out.push_str(&format!("\n{h}\n\n"));
        let body = bodies
            .iter()
            .find(|(n, _)| n == h)
            .map(|(_, b)| *b)
            .unwrap_or("Nothing yet.\n");
        out.push_str(body);
    }
    out
}

/// An ADR with the fixed H2 list.
pub fn adr(
    id: &str,
    status: &str,
    consumes: &[&str],
    introduces: &str,
    supersedes: &str,
) -> String {
    format!(
        "---\nschema: devforgeai/adr/1\nid: {id}\nphase: constitute\nstatus: {status}\nproduced_by: establishing-context\nconsumes: [{}]\nopen_questions: []\n---\n\n# {id}: a decision\n\n## Context\n\nThe forces.\n\n## Decision\n\nThe binary reads TOML.\n\n## Consequences\n\n| Consequence | Direction | Affected |\n|---|---|---|\n| It parses | gain | src |\n\n## Constraints introduced\n\n| CON | Kind | Statement |\n|---|---|---|\n{introduces}\n## Supersedes\n\n| Supersedes ADR | Retires CON |\n|---|---|\n{supersedes}",
        consumes.join(", ")
    )
}

/// The six context files, one ADR, and a `requirements.yaml`, all consistent.
pub fn write_context_set(p: &Project, status: &str) {
    let constraints = "### CON-001 The core holds no IO\n\n| Field | Value |\n|---|---|\n| kind | boundary |\n| status | active |\n| statement | The core holds no IO |\n| source | REQ-001 |\n| introduced_by | ADR-001 |\n| enforced_by | AP-001 |\n";
    let index = "| CON | Kind | Status | Title | Source | Introduced by | Enforced by |\n|---|---|---|---|---|---|---|\n| CON-001 | boundary | active | The core holds no IO | REQ-001 | ADR-001 | AP-001 |\n";
    p.write(
        ".devforgeai/context/architecture-constraints.md",
        &context_file(
            "architecture-constraints",
            status,
            &[
                ("## Constraints", constraints),
                ("## Constraint index", index),
            ],
        ),
    );

    let aps = "### AP-001 A raw socket in the core\n\n| Field | Value |\n|---|---|\n| category | layer |\n| severity | high |\n| scope | src/**/*.rs |\n| detector_kind | literal |\n| detector | TcpStream |\n| remediation | The adapter owns the socket |\n| source | CON-001 |\n";
    let ap_index = "| AP | Category | Severity | Scope | Detector kind | Detector | Source |\n|---|---|---|---|---|---|---|\n| AP-001 | layer | high | src/**/*.rs | literal | TcpStream | CON-001 |\n";
    p.write(
        ".devforgeai/context/anti-patterns.md",
        &context_file(
            "anti-patterns",
            status,
            &[
                ("## Anti-patterns", aps),
                ("## Anti-pattern index", ap_index),
            ],
        ),
    );

    p.write(
        ".devforgeai/context/tech-stack.md",
        &context_file(
            "tech-stack",
            status,
            &[(
                "## Languages",
                "| Key | Value | Source |\n|---|---|---|\n| language.primary | rust | Cargo.toml |\n| language.primary.version | 1.97 | Cargo.toml |\n",
            )],
        ),
    );
    p.write(
        ".devforgeai/context/source-tree.md",
        &context_file(
            "source-tree",
            status,
            &[(
                "## Roots",
                "| Key | Value | Source |\n|---|---|---|\n| source.root | src | layout |\n",
            )],
        ),
    );
    p.write(
        ".devforgeai/context/dependencies.md",
        &context_file("dependencies", status, &[]),
    );
    p.write(
        ".devforgeai/context/coding-standards.md",
        &context_file(
            "coding-standards",
            status,
            &[(
                "## Formatting",
                "| Key | Value | Source |\n|---|---|---|\n| style.indent | 4 | rustfmt |\n",
            )],
        ),
    );

    p.write(
        ".devforgeai/adr/ADR-001.md",
        &adr(
            "ADR-001",
            "accepted",
            &["REQ-001"],
            "| CON-001 | boundary | The core holds no IO |\n",
            "| none | none |\n",
        ),
    );
    p.write(
        ".devforgeai/requirements.yaml",
        &requirements("IDEA-001", "accepted"),
    );
}

/// Fails the test if the developer's real `~/.devforgeai/trust.toml` is
/// touched while the guard is alive.
///
/// The trust store is the one piece of state outside the temp tree that a test
/// can reach: `trust pin` writes it, and a subprocess spawned from a test does
/// not inherit the `test-home` feature, so a redirect set only through
/// `DEVFORGEAI_HOME` is read by nothing in that child. The guard turns that
/// class of leak from a silent edit of the developer's machine into a failing
/// test, which is the only way it gets noticed.
///
/// The real home is read from the parent process's own environment, which no
/// test alters — the tests that redirect a home do it for a child process or
/// through `DEVFORGEAI_HOME`, never by rewriting `USERPROFILE` here.
pub struct RealTrustStoreGuard {
    path: std::path::PathBuf,
    before: Option<std::time::SystemTime>,
}

impl Default for RealTrustStoreGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl RealTrustStoreGuard {
    pub fn new() -> RealTrustStoreGuard {
        let key = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
        let path = std::env::var_os(key)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".devforgeai")
            .join("trust.toml");
        let before = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        RealTrustStoreGuard { path, before }
    }
}

impl Drop for RealTrustStoreGuard {
    fn drop(&mut self) {
        let after = std::fs::metadata(&self.path)
            .and_then(|m| m.modified())
            .ok();
        if after == self.before {
            return;
        }
        // Panicking while already unwinding aborts, which turns a plain test
        // failure into a harness crash and hides it.
        if std::thread::panicking() {
            eprintln!(
                "warning: {} changed during a failing test",
                self.path.display()
            );
            return;
        }
        panic!(
            "a test wrote to the real trust store at {}: no test may touch state outside its temp tree",
            self.path.display()
        );
    }
}
