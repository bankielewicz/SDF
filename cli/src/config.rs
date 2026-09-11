//! `.devforgeai/config.toml`: the detected stack, the layer thresholds, the
//! verifier registry, and the six phase tables.

use crate::errors::CliError;
use crate::project;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// The fixed `schema` value.
pub const SCHEMA: &str = "devforgeai/config/1";

/// The compiled floor for each layer name and for the overall figure. Values
/// below these are rejected, not clamped.
pub const LAYER_FLOORS: &[(&str, f64)] = &[
    ("domain", 90.0),
    ("application", 80.0),
    ("infrastructure", 70.0),
    ("interface", 60.0),
];

/// The compiled floor for `[coverage].overall_min`.
pub const OVERALL_FLOOR: f64 = 75.0;

/// The four layer names, in match order.
pub const LAYER_NAMES: &[&str] = &["domain", "application", "infrastructure", "interface"];

/// `.devforgeai/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Fixed: `devforgeai/config/1`.
    pub schema: String,
    /// RFC 3339 UTC; rewritten on every `stack detect`.
    pub generated_at: String,
    /// The semver of the binary that wrote the file.
    pub cli_version: String,
    /// True when `stack` is empty.
    #[serde(default)]
    pub degraded: bool,
    /// One table per detected ecosystem, in detection order.
    #[serde(default)]
    pub stack: Vec<Stack>,
    /// The frontend globs `design lint` reads.
    #[serde(default)]
    pub frontend: Frontend,
    /// The four coverage layers.
    #[serde(default)]
    pub layer: Vec<Layer>,
    /// Explore's phase table.
    #[serde(default)]
    pub explore: ExploreCfg,
    /// Plan's phase table.
    #[serde(default)]
    pub plan: PlanCfg,
    /// Build's phase table.
    #[serde(default)]
    pub build: BuildCfg,
    /// Verify's phase table.
    #[serde(default)]
    pub verify: VerifyCfg,
    /// Release's phase table.
    #[serde(default)]
    pub release: ReleaseCfg,
    /// Reflect's phase table.
    #[serde(default)]
    pub reflect: ReflectCfg,
    /// The overall coverage figure and its exclusions.
    #[serde(default)]
    pub coverage: CoverageCfg,
    /// The registry of subagents SubagentStop ingests.
    #[serde(default)]
    pub verifier: Vec<Verifier>,
}

/// One `[[stack]]` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stack {
    /// One of rust, node, python, go, dotnet, jvm, ruby.
    pub id: String,
    /// `detected` when `stack detect` wrote this entry, `manual` when a human
    /// did.
    ///
    /// Detection reruns on every session start, and it only knows the
    /// ecosystems it ships markers for. Without this field it overwrote the
    /// whole table, so a project whose commands were written by hand — a
    /// language the detector does not know, or a `sh ci/test` wrapper — lost
    /// them at the next session start and `degraded` flipped to true, which
    /// makes every command-running gate check skip. An absent value reads as
    /// `manual`, so a file written before this field existed is protected.
    #[serde(default = "default_source")]
    pub source: String,
    /// The repo-relative marker paths that matched.
    #[serde(default)]
    pub markers: Vec<String>,
    /// The detected package manager, or `""`.
    #[serde(default)]
    pub package_manager: String,
    /// The source roots of this ecosystem.
    #[serde(default)]
    pub source_roots: Vec<String>,
    /// The project's own test command, or `""`.
    #[serde(default)]
    pub test_command: String,
    /// The project's own coverage command, or `""`.
    #[serde(default)]
    pub coverage_command: String,
    /// One of lcov, cobertura, jacoco, go-cover, devforgeai-json, none.
    #[serde(default)]
    pub coverage_format: String,
    /// Glob patterns naming the coverage artifact.
    #[serde(default)]
    pub coverage_paths: Vec<String>,
    /// The project's own lint command, or `""`.
    #[serde(default)]
    pub lint_command: String,
    /// The project's own complexity command, or `""`.
    #[serde(default)]
    pub complexity_command: String,
    /// Per-command wall clock limit in seconds.
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    /// Added to the command environment.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

fn default_timeout() -> u64 {
    900
}

/// The `source` an entry with no such key carries.
fn default_source() -> String {
    STACK_MANUAL.to_string()
}

/// `[[stack]].source`: this entry was written by `stack detect`.
pub const STACK_DETECTED: &str = "detected";
/// `[[stack]].source`: this entry was written by a human.
pub const STACK_MANUAL: &str = "manual";

impl Default for Stack {
    fn default() -> Self {
        Stack {
            id: String::new(),
            source: default_source(),
            markers: Vec::new(),
            package_manager: String::new(),
            source_roots: Vec::new(),
            test_command: String::new(),
            coverage_command: String::new(),
            coverage_format: String::new(),
            coverage_paths: Vec::new(),
            lint_command: String::new(),
            complexity_command: String::new(),
            timeout_secs: default_timeout(),
            env: BTreeMap::new(),
        }
    }
}

impl Stack {
    /// True when `stack detect` owns this entry and may replace it.
    ///
    /// Anything that is not exactly `detected` is treated as a human's, which
    /// is the safe direction: the cost of misreading a detected entry as
    /// manual is a stale table a rerun leaves alone, and the cost of the
    /// reverse is silently discarding a project's own commands.
    pub fn is_detected(&self) -> bool {
        self.source == STACK_DETECTED
    }
}

/// `[frontend]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontend {
    /// Matched against repo-relative paths.
    pub globs: Vec<String>,
    /// The token file.
    pub tokens_path: String,
    /// Paths `design lint` skips.
    pub exclude: Vec<String>,
}

impl Default for Frontend {
    fn default() -> Self {
        Frontend {
            globs: [
                "**/*.css",
                "**/*.scss",
                "**/*.sass",
                "**/*.less",
                "**/*.vue",
                "**/*.svelte",
                "**/*.jsx",
                "**/*.tsx",
                "**/*.html",
                "**/*.styles.ts",
                "**/*.styles.js",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            tokens_path: ".devforgeai/brand/tokens.json".to_string(),
            exclude: [
                "**/node_modules/**",
                "**/dist/**",
                "**/build/**",
                "**/vendor/**",
                ".explore-prototype/**",
                ".devforgeai/explore/mockups/**",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        }
    }
}

/// One `[[layer]]` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    /// One of domain, application, infrastructure, interface.
    pub name: String,
    /// The globs assigning a file to this layer.
    pub globs: Vec<String>,
    /// The percent of covered lines this layer needs.
    pub coverage_min: f64,
}

/// The four layers at their default globs and thresholds.
pub fn default_layers() -> Vec<Layer> {
    vec![
        Layer {
            name: "domain".into(),
            globs: [
                "**/domain/**",
                "**/core/**",
                "**/entities/**",
                "**/model/**",
                "**/models/**",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            coverage_min: 95.0,
        },
        Layer {
            name: "application".into(),
            globs: [
                "**/application/**",
                "**/services/**",
                "**/usecases/**",
                "**/use_cases/**",
                "**/handlers/**",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            coverage_min: 85.0,
        },
        Layer {
            name: "infrastructure".into(),
            globs: [
                "**/infrastructure/**",
                "**/infra/**",
                "**/adapters/**",
                "**/repositories/**",
                "**/persistence/**",
                "**/db/**",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            coverage_min: 80.0,
        },
        Layer {
            name: "interface".into(),
            globs: [
                "**/api/**",
                "**/controllers/**",
                "**/routes/**",
                "**/cli/**",
                "**/ui/**",
                "**/components/**",
                "**/pages/**",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            coverage_min: 70.0,
        },
    ]
}

/// `[explore]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploreCfg {
    /// Calendar days from `[explore].started_at`.
    pub timebox_days: i64,
    /// Calendar days from `[explore].remedy_started_at`.
    pub remedy_timebox_days: i64,
}

impl Default for ExploreCfg {
    fn default() -> Self {
        ExploreCfg {
            timebox_days: 5,
            remedy_timebox_days: 1,
        }
    }
}

/// `[plan]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanCfg {
    /// The maximum sum of `stories[].points` in one sprint.
    pub sprint_capacity_points: i64,
    /// The closed set `stories[].points` is drawn from.
    pub story_points: Vec<i64>,
}

impl Default for PlanCfg {
    fn default() -> Self {
        PlanCfg {
            sprint_capacity_points: 20,
            story_points: vec![1, 2, 3, 5, 8],
        }
    }
}

/// `[build]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildCfg {
    /// Resolved against the project root.
    pub worktree_root: String,
    /// The branch name prefix.
    pub branch_prefix: String,
    /// The ref a story worktree branches from.
    pub base_ref: String,
    /// The per-function branch ceiling refactor aims below.
    pub complexity_max: i64,
    /// The share of a file's lines that may repeat.
    pub duplication_max_percent: f64,
}

impl Default for BuildCfg {
    fn default() -> Self {
        BuildCfg {
            worktree_root: "../wt".into(),
            branch_prefix: "story/".into(),
            base_ref: "HEAD".into(),
            complexity_max: 10,
            duplication_max_percent: 5.0,
        }
    }
}

/// `[verify]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyCfg {
    /// `light` or `deep`.
    pub mode: String,
    /// The per-function branch ceiling.
    pub complexity_max: i64,
    /// The share of a file's lines that may repeat.
    pub duplication_max_percent: f64,
    /// The shortest repeated run counted as duplication.
    pub duplication_min_lines: i64,
    /// Prints one `devforgeai-metrics/1` JSON object.
    pub metrics_command: String,
    /// Prints one `devforgeai-callgraph/1` JSON object.
    pub call_graph_command: String,
}

impl Default for VerifyCfg {
    fn default() -> Self {
        VerifyCfg {
            mode: "light".into(),
            complexity_max: 10,
            duplication_max_percent: 5.0,
            duplication_min_lines: 20,
            metrics_command: String::new(),
            call_graph_command: String::new(),
        }
    }
}

/// `[release]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseCfg {
    /// kubernetes, compose, github-actions, vps, none, or `""`.
    pub platform: String,
    /// github-actions or none.
    pub ci: String,
    /// Project-relative.
    pub deploy_root: String,
    /// Project-relative.
    pub docs_root: String,
    /// The project's own release build.
    pub build_command: String,
    /// The project's own packaging step.
    pub package_command: String,
    /// Glob patterns naming the release artifacts.
    pub artifact_paths: Vec<String>,
    /// Prints one line per public symbol.
    pub api_symbols_command: String,
    /// The container image reference.
    pub image_name: String,
    /// The service port.
    pub service_port: i64,
}

impl Default for ReleaseCfg {
    fn default() -> Self {
        ReleaseCfg {
            platform: String::new(),
            ci: "github-actions".into(),
            deploy_root: "deploy".into(),
            docs_root: "docs".into(),
            build_command: String::new(),
            package_command: String::new(),
            artifact_paths: Vec::new(),
            api_symbols_command: String::new(),
            image_name: String::new(),
            service_port: 8080,
        }
    }
}

/// `[reflect]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectCfg {
    /// `~` expands to the user's home directory.
    pub session_root: String,
    /// `""` derives the key from the project path.
    pub session_key: String,
    /// The `--since` default when the flag carries no date.
    pub window_days: i64,
}

impl Default for ReflectCfg {
    fn default() -> Self {
        ReflectCfg {
            session_root: "~/.claude/projects".into(),
            session_key: String::new(),
            window_days: 14,
        }
    }
}

/// `[coverage]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageCfg {
    /// Percent over all files under `source_roots`.
    pub overall_min: f64,
    /// `report` or `fail`.
    pub unassigned_policy: String,
    /// Excluded from every layer and the overall figure.
    pub exclude: Vec<String>,
}

impl Default for CoverageCfg {
    fn default() -> Self {
        CoverageCfg {
            overall_min: 80.0,
            unassigned_policy: "report".into(),
            exclude: [
                "**/tests/**",
                "**/test/**",
                "**/*_test.*",
                "**/*.test.*",
                "**/*.spec.*",
                "**/mocks/**",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        }
    }
}

/// One `[[verifier]]` table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verifier {
    /// The kebab-case subagent name.
    pub name: String,
    /// The phase whose report this verifier writes into.
    pub phase: String,
    /// The dotted path inside the report YAML.
    pub report_field: String,
    /// The unit word of the handoff `Verified` line.
    #[serde(default = "default_unit")]
    pub unit: String,
    /// True when a `verifier_pass` check names it.
    #[serde(default)]
    pub required: bool,
}

fn default_unit() -> String {
    "checks".to_string()
}

/// The twenty registrations `init` writes, in the spec table order.
pub fn default_verifiers() -> Vec<Verifier> {
    VERIFIER_ROWS
        .iter()
        .map(|(phase, name, field, unit, required)| Verifier {
            name: (*name).to_string(),
            phase: (*phase).to_string(),
            report_field: (*field).to_string(),
            unit: (*unit).to_string(),
            required: *required,
        })
        .collect()
}

/// The twenty registrations of `specs/01-cli.md` `## Outputs`, in that order,
/// which is the order the handoff tie-break of rendering rule 6 reads.
pub const VERIFIER_ROWS: &[(&str, &str, &str, &str, bool)] = &[
    (
        "explore",
        "kill-case-builder",
        "verifiers.kill_case",
        "objections",
        true,
    ),
    (
        "discover",
        "flow-integrity-auditor",
        "verifiers.flow_integrity",
        "flows",
        true,
    ),
    (
        "constitute",
        "architecture-reviewer",
        "verifiers.architecture_reviewer",
        "requirements",
        false,
    ),
    (
        "constitute",
        "alignment-auditor",
        "verifiers.alignment_auditor",
        "checks",
        false,
    ),
    (
        "plan",
        "story-invest-auditor",
        "verifiers.story_invest",
        "stories",
        true,
    ),
    (
        "build",
        "ac-test-writer",
        "verifiers.ac_testable",
        "AC",
        true,
    ),
    (
        "build",
        "story-ac-verifier",
        "verifiers.story_ac",
        "ACs",
        true,
    ),
    (
        "build",
        "context-validator",
        "verifiers.context",
        "files",
        true,
    ),
    (
        "verify",
        "ac-compliance-verifier",
        "verifiers.ac_compliance",
        "ACs",
        true,
    ),
    (
        "verify",
        "standards-reviewer",
        "verifiers.standards",
        "files",
        true,
    ),
    (
        "verify",
        "anti-pattern-scanner",
        "verifiers.anti_patterns",
        "anti-patterns",
        true,
    ),
    (
        "verify",
        "constraint-auditor",
        "verifiers.constraints",
        "constraints",
        true,
    ),
    (
        "verify",
        "coverage-gap-auditor",
        "verifiers.coverage_gaps",
        "layers",
        true,
    ),
    (
        "verify",
        "dead-code-detector",
        "verifiers.dead_code",
        "symbols",
        true,
    ),
    (
        "verify",
        "deferral-validator",
        "verifiers.deferrals",
        "deferrals",
        true,
    ),
    (
        "verify",
        "security-auditor",
        "verifiers.security",
        "OWASP categories",
        true,
    ),
    (
        "verify",
        "code-quality-auditor",
        "verifiers.quality",
        "files",
        true,
    ),
    (
        "verify",
        "adr-conformance-reviewer",
        "verifiers.adr_conformance",
        "ADRs",
        true,
    ),
    (
        "design",
        "requirement-coverage-auditor",
        "verifiers.requirement_coverage",
        "screens",
        false,
    ),
    (
        "release",
        "deferral-auditor",
        "verifiers.deferrals",
        "deferrals",
        true,
    ),
];

impl Default for Config {
    fn default() -> Self {
        Config {
            schema: SCHEMA.to_string(),
            generated_at: String::new(),
            cli_version: crate::VERSION.to_string(),
            degraded: true,
            stack: Vec::new(),
            frontend: Frontend::default(),
            layer: default_layers(),
            explore: ExploreCfg::default(),
            plan: PlanCfg::default(),
            build: BuildCfg::default(),
            verify: VerifyCfg::default(),
            release: ReleaseCfg::default(),
            reflect: ReflectCfg::default(),
            coverage: CoverageCfg::default(),
            verifier: default_verifiers(),
        }
    }
}

impl Config {
    /// The `[[stack]]` table with this id.
    pub fn stack_by_id(&self, id: &str) -> Option<&Stack> {
        self.stack.iter().find(|s| s.id == id)
    }

    /// The `[[verifier]]` table with this name.
    pub fn verifier_by_name(&self, name: &str) -> Option<&Verifier> {
        self.verifier.iter().find(|v| v.name == name)
    }

    /// The layer table for `name`, filling in the compiled default when the
    /// project deleted the table: deleting a layer lowers nothing.
    pub fn layer_min(&self, name: &str) -> f64 {
        if let Some(l) = self.layer.iter().find(|l| l.name == name) {
            return l.coverage_min;
        }
        default_layers()
            .into_iter()
            .find(|l| l.name == name)
            .map(|l| l.coverage_min)
            .unwrap_or(0.0)
    }

    /// The compiled floors, checked while `degraded` is clear. Values are
    /// rejected, not clamped, so an edited file fails loudly.
    pub fn validate_minimums(&self) -> Result<(), CliError> {
        if self.degraded {
            return Ok(());
        }
        for (name, floor) in LAYER_FLOORS {
            if let Some(l) = self.layer.iter().find(|l| &l.name == name) {
                if l.coverage_min < *floor {
                    return Err(CliError::at(
                        "DFA-E303",
                        format!(
                            "config.toml sets layer.{name}.coverage_min to {}, below the compiled minimum {floor}",
                            l.coverage_min
                        ),
                        ".devforgeai/config.toml",
                    ));
                }
            }
        }
        if self.coverage.overall_min < OVERALL_FLOOR {
            return Err(CliError::at(
                "DFA-E303",
                format!(
                    "config.toml sets coverage.overall_min to {}, below the compiled minimum {OVERALL_FLOOR}",
                    self.coverage.overall_min
                ),
                ".devforgeai/config.toml",
            ));
        }
        Ok(())
    }
}

/// Read and validate `.devforgeai/config.toml`.
pub fn load(root: &Path) -> Result<Config, CliError> {
    let path = project::dot(root).join(project::CONFIG);
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(CliError::at(
                "DFA-E101",
                ".devforgeai/config.toml not found; run 'devforgeai stack detect'",
                ".devforgeai/config.toml",
            ))
        }
        Err(e) => return Err(CliError::io("reading", path.display(), &e)),
    };
    let cfg: Config = toml::from_str(&text).map_err(|e| {
        CliError::at(
            "DFA-E104",
            format!("config.toml is not valid TOML: {e}"),
            ".devforgeai/config.toml",
        )
    })?;
    if cfg.schema != SCHEMA {
        return Err(CliError::at(
            "DFA-E107",
            format!(
                "config.toml: schema '{}' is unsupported; this binary reads {SCHEMA}",
                cfg.schema
            ),
            ".devforgeai/config.toml",
        ));
    }
    for s in &cfg.stack {
        if s.source != STACK_DETECTED && s.source != STACK_MANUAL {
            return Err(CliError::at(
                "DFA-E104",
                format!(
                    "config.toml [[stack]] '{}' has source '{}'; expected {STACK_DETECTED} or {STACK_MANUAL}",
                    s.id, s.source
                ),
                ".devforgeai/config.toml",
            ));
        }
    }
    cfg.validate_minimums()?;
    Ok(cfg)
}

/// Serialise a config in the key order the spec's schema shows.
pub fn to_toml(cfg: &Config) -> Result<String, CliError> {
    toml::to_string_pretty(cfg)
        .map_err(|e| CliError::new("DFA-E900", format!("serialising config.toml failed: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `[[stack]]` block of the spec's `## Outputs` schema, verbatim minus
    /// the trailing comments.
    const SPEC_EXAMPLE: &str = r#"
schema = "devforgeai/config/1"
generated_at = "2026-09-10T14:02:11Z"
cli_version = "1.0.0"
degraded = false

[[stack]]
id = "rust"
markers = ["Cargo.toml"]
package_manager = "cargo"
source_roots = ["src"]
test_command = "cargo test --all-features"
coverage_command = "cargo llvm-cov --all-features --lcov --output-path target/devforgeai/lcov.info"
coverage_format = "lcov"
coverage_paths = ["target/devforgeai/lcov.info"]
lint_command = "cargo clippy --all-targets --all-features -- -D warnings"
complexity_command = ""
timeout_secs = 900
env = {}

[[verifier]]
name = "ac-compliance-verifier"
phase = "verify"
report_field = "verifiers.ac_compliance"
unit = "ACs"
required = true
"#;

    #[test]
    fn spec_example_round_trips() {
        let cfg: Config = toml::from_str(SPEC_EXAMPLE).expect("the spec example parses");
        assert_eq!(cfg.schema, SCHEMA);
        assert_eq!(cfg.generated_at, "2026-09-10T14:02:11Z");
        assert!(!cfg.degraded);
        assert_eq!(cfg.stack.len(), 1);
        let s = &cfg.stack[0];
        assert_eq!(s.id, "rust");
        assert_eq!(s.package_manager, "cargo");
        assert_eq!(s.source_roots, vec!["src"]);
        assert_eq!(s.test_command, "cargo test --all-features");
        assert_eq!(s.coverage_format, "lcov");
        assert_eq!(s.timeout_secs, 900);
        assert!(s.env.is_empty());
        assert_eq!(cfg.verifier.len(), 1);
        assert_eq!(cfg.verifier[0].report_field, "verifiers.ac_compliance");
        assert_eq!(cfg.verifier[0].unit, "ACs");

        // The round trip preserves every value the spec names.
        let text = to_toml(&cfg).expect("serialise");
        let again: Config = toml::from_str(&text).expect("reparse");
        assert_eq!(again.stack[0].coverage_command, s.coverage_command);
        assert_eq!(again.stack[0].coverage_paths, s.coverage_paths);
        assert_eq!(again.verifier[0].name, "ac-compliance-verifier");
    }

    #[test]
    fn defaults_match_the_spec_schema() {
        let cfg = Config::default();
        assert_eq!(cfg.frontend.tokens_path, ".devforgeai/brand/tokens.json");
        assert_eq!(cfg.frontend.globs.len(), 11);
        assert_eq!(cfg.frontend.exclude.len(), 6);
        assert_eq!(cfg.layer.len(), 4);
        assert_eq!(cfg.layer_min("domain"), 95.0);
        assert_eq!(cfg.layer_min("application"), 85.0);
        assert_eq!(cfg.layer_min("infrastructure"), 80.0);
        assert_eq!(cfg.layer_min("interface"), 70.0);
        assert_eq!(cfg.coverage.overall_min, 80.0);
        assert_eq!(cfg.coverage.unassigned_policy, "report");
        assert_eq!(cfg.explore.timebox_days, 5);
        assert_eq!(cfg.plan.sprint_capacity_points, 20);
        assert_eq!(cfg.build.worktree_root, "../wt");
        assert_eq!(cfg.verify.mode, "light");
        assert_eq!(cfg.release.service_port, 8080);
        assert_eq!(cfg.reflect.window_days, 14);
    }

    #[test]
    fn twenty_verifiers_across_eight_phases() {
        let v = default_verifiers();
        assert_eq!(v.len(), 20, "twenty registrations");
        // The spec's prose says seven phases; its own table names eight, one
        // registration per phase plus the three Verify carries beyond one.
        let phases: std::collections::BTreeSet<&str> = v.iter().map(|x| x.phase.as_str()).collect();
        assert_eq!(
            phases.iter().copied().collect::<Vec<_>>(),
            vec![
                "build",
                "constitute",
                "design",
                "discover",
                "explore",
                "plan",
                "release",
                "verify"
            ]
        );
        // One name binds to one phase.
        let names: std::collections::BTreeSet<&str> = v.iter().map(|x| x.name.as_str()).collect();
        assert_eq!(names.len(), 20, "every name is unique");
        // Four are named by no `verifier_pass` check.
        let optional: Vec<&str> = v
            .iter()
            .filter(|x| !x.required)
            .map(|x| x.name.as_str())
            .collect();
        assert_eq!(
            optional,
            vec![
                "architecture-reviewer",
                "alignment-auditor",
                "requirement-coverage-auditor",
            ]
        );
    }

    #[test]
    fn layer_floor_below_minimum_gives_e303() {
        let mut cfg = Config {
            degraded: false,
            ..Default::default()
        };
        cfg.layer[0].coverage_min = 89.9;
        let err = cfg.validate_minimums().expect_err("below the domain floor");
        assert_eq!(err.code(), "DFA-E303");
        assert_eq!(err.exit(), 1);
    }

    #[test]
    fn overall_floor_below_minimum_gives_e303() {
        let mut cfg = Config {
            degraded: false,
            ..Default::default()
        };
        cfg.coverage.overall_min = 74.9;
        let err = cfg
            .validate_minimums()
            .expect_err("below the overall floor");
        assert_eq!(err.code(), "DFA-E303");
    }

    #[test]
    fn degraded_config_skips_the_floors() {
        let mut cfg = Config {
            degraded: true,
            ..Default::default()
        };
        cfg.layer[0].coverage_min = 1.0;
        cfg.coverage.overall_min = 1.0;
        cfg.validate_minimums()
            .expect("with the flag set the numbers govern nothing");
    }

    #[test]
    fn deleting_a_layer_keeps_its_default_threshold() {
        let mut cfg = Config::default();
        cfg.layer.retain(|l| l.name != "domain");
        assert_eq!(cfg.layer_min("domain"), 95.0, "removal lowers nothing");
        cfg.degraded = false;
        cfg.validate_minimums()
            .expect("an absent table is at its default");
    }

    #[test]
    fn missing_config_gives_e101() {
        let t = tempfile::tempdir().expect("temp");
        std::fs::create_dir_all(t.path().join(".devforgeai")).expect("mkdir");
        let err = load(t.path()).expect_err("no config.toml");
        assert_eq!(err.code(), "DFA-E101");
        assert_eq!(err.exit(), 1);
    }

    #[test]
    fn unparsable_config_gives_e104() {
        let t = tempfile::tempdir().expect("temp");
        let dot = t.path().join(".devforgeai");
        std::fs::create_dir_all(&dot).expect("mkdir");
        std::fs::write(dot.join("config.toml"), "this is not = = toml").expect("write");
        let err = load(t.path()).expect_err("bad TOML");
        assert_eq!(err.code(), "DFA-E104");
    }

    #[test]
    fn unknown_schema_gives_e107() {
        let t = tempfile::tempdir().expect("temp");
        let dot = t.path().join(".devforgeai");
        std::fs::create_dir_all(&dot).expect("mkdir");
        std::fs::write(
            dot.join("config.toml"),
            "schema = \"devforgeai/config/2\"\ngenerated_at = \"\"\ncli_version = \"1.0.0\"\n",
        )
        .expect("write");
        let err = load(t.path()).expect_err("unsupported schema");
        assert_eq!(err.code(), "DFA-E107");
    }
}
