//! `config get <key> [--stack <id>]`: print one configuration value.
//!
//! The key comes from a closed list. The binary reads `config.toml` and
//! renders the value; it never writes.

use crate::config::{Config, LAYER_NAMES};
use crate::ctx::Ctx;
use crate::errors::CliError;
use crate::Outcome;

/// The keys the spec fixes, `layer.<name>.coverage_min` written with its
/// placeholder. The list is the enumeration `DFA-E013` names.
pub const KEYS: &[&str] = &[
    "stack.test_command",
    "stack.coverage_command",
    "stack.lint_command",
    "stack.complexity_command",
    "stack.source_roots",
    "stack.package_manager",
    "build.worktree_root",
    "build.branch_prefix",
    "build.base_ref",
    "build.complexity_max",
    "build.duplication_max_percent",
    "coverage.overall_min",
    "layer.<name>.coverage_min",
    "frontend.tokens_path",
    "frontend.globs",
    "degraded",
];

/// One resolved value, in the four shapes the renderer distinguishes.
enum Value {
    Str(String),
    Arr(Vec<String>),
    Int(i64),
    Num(f64),
    Bool(bool),
}

impl Value {
    fn kind(&self) -> &'static str {
        match self {
            Value::Str(_) => "string",
            Value::Arr(_) => "array",
            Value::Int(_) => "integer",
            Value::Num(_) => "number",
            Value::Bool(_) => "boolean",
        }
    }

    /// The human lines: one per array element, one line otherwise. An empty
    /// string is one empty line.
    fn lines(&self) -> Vec<String> {
        match self {
            Value::Str(s) => vec![s.clone()],
            Value::Arr(v) => v.clone(),
            Value::Int(n) => vec![n.to_string()],
            Value::Num(f) => vec![render_float(*f)],
            Value::Bool(b) => vec![b.to_string()],
        }
    }

    fn json(&self) -> serde_json::Value {
        match self {
            Value::Str(s) => serde_json::Value::String(s.clone()),
            Value::Arr(v) => serde_json::Value::Array(
                v.iter()
                    .map(|s| serde_json::Value::String(s.clone()))
                    .collect(),
            ),
            Value::Int(n) => serde_json::Value::from(*n),
            Value::Num(f) => serde_json::Value::from(*f),
            Value::Bool(b) => serde_json::Value::Bool(*b),
        }
    }
}

/// A float in the form `config.toml` carries it: always with a decimal point,
/// so `80` reads as the percent `80.0` rather than as a count.
fn render_float(f: f64) -> String {
    if f.fract() == 0.0 {
        format!("{f:.1}")
    } else {
        format!("{f}")
    }
}

/// The `DFA-E013` an unknown key raises, naming the closed list.
fn unknown_key(key: &str) -> CliError {
    CliError::new(
        "DFA-E013",
        format!(
            "'{key}' is not a config key; expected one of {}",
            KEYS.join(", ")
        ),
    )
}

/// Print one configuration value.
pub fn get(ctx: &mut Ctx, key: &str, stack: Option<&str>) -> Result<Outcome, CliError> {
    let project = ctx.root.display().to_string();
    let cfg: Config = ctx.config()?.clone();

    let (value, stack_id) = if let Some(field) = key.strip_prefix("stack.") {
        let s = select_stack(&cfg, stack)?;
        let v = match field {
            "test_command" => Value::Str(s.test_command.clone()),
            "coverage_command" => Value::Str(s.coverage_command.clone()),
            "lint_command" => Value::Str(s.lint_command.clone()),
            "complexity_command" => Value::Str(s.complexity_command.clone()),
            "source_roots" => Value::Arr(s.source_roots.clone()),
            "package_manager" => Value::Str(s.package_manager.clone()),
            _ => return Err(unknown_key(key)),
        };
        (v, s.id.clone())
    } else {
        (non_stack(&cfg, key)?, String::new())
    };

    let data = serde_json::json!({
        "key": key,
        "stack": stack_id,
        "value": value.json(),
        "kind": value.kind(),
    });

    Ok(Outcome {
        human: value.lines(),
        data,
        degraded: cfg.degraded,
        project,
        ..Default::default()
    })
}

/// The `[[stack]]` table `--stack` names, or the first when it is absent.
fn select_stack<'a>(
    cfg: &'a Config,
    want: Option<&str>,
) -> Result<&'a crate::config::Stack, CliError> {
    match want {
        Some(id) => cfg.stack.iter().find(|s| s.id == id).ok_or_else(|| {
            CliError::new(
                "DFA-E013",
                format!(
                    "config.toml has no [[stack]] with id '{id}'; the ids are {}",
                    ids(cfg)
                ),
            )
        }),
        None => cfg.stack.first().ok_or_else(|| {
            CliError::new(
                "DFA-E013",
                "config.toml has no [[stack]] table; run 'devforgeai stack detect'",
            )
        }),
    }
}

fn ids(cfg: &Config) -> String {
    if cfg.stack.is_empty() {
        "none".to_string()
    } else {
        cfg.stack
            .iter()
            .map(|s| s.id.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// Every key outside `stack.*`.
fn non_stack(cfg: &Config, key: &str) -> Result<Value, CliError> {
    if let Some(name) = key
        .strip_prefix("layer.")
        .and_then(|r| r.strip_suffix(".coverage_min"))
    {
        if !LAYER_NAMES.contains(&name) {
            return Err(CliError::new(
                "DFA-E013",
                format!(
                    "'{name}' is not a coverage layer; the layers are {}",
                    LAYER_NAMES.join(", ")
                ),
            ));
        }
        let min = cfg
            .layer
            .iter()
            .find(|l| l.name == name)
            .map(|l| l.coverage_min)
            .unwrap_or_else(|| {
                crate::config::LAYER_FLOORS
                    .iter()
                    .find(|(n, _)| *n == name)
                    .map(|(_, f)| *f)
                    .unwrap_or(0.0)
            });
        return Ok(Value::Num(min));
    }

    Ok(match key {
        "build.worktree_root" => Value::Str(cfg.build.worktree_root.clone()),
        "build.branch_prefix" => Value::Str(cfg.build.branch_prefix.clone()),
        "build.base_ref" => Value::Str(cfg.build.base_ref.clone()),
        "build.complexity_max" => Value::Int(cfg.build.complexity_max),
        "build.duplication_max_percent" => Value::Num(cfg.build.duplication_max_percent),
        "coverage.overall_min" => Value::Num(cfg.coverage.overall_min),
        "frontend.tokens_path" => Value::Str(cfg.frontend.tokens_path.clone()),
        "frontend.globs" => Value::Arr(cfg.frontend.globs.clone()),
        "degraded" => Value::Bool(cfg.degraded),
        _ => return Err(unknown_key(key)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_float_keeps_its_decimal_point() {
        assert_eq!(render_float(80.0), "80.0");
        assert_eq!(render_float(5.0), "5.0");
        assert_eq!(render_float(82.5), "82.5");
    }

    #[test]
    fn the_key_list_carries_the_layer_placeholder_once() {
        assert_eq!(KEYS.len(), 16);
        assert!(KEYS.contains(&"layer.<name>.coverage_min"));
    }

    #[test]
    fn an_empty_string_renders_as_one_empty_line() {
        assert_eq!(Value::Str(String::new()).lines(), vec![String::new()]);
    }
}
