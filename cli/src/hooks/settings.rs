//! The `.claude/settings.json` merge: the template block `hook install` owns,
//! and the append-when-absent algorithm the spec fixes.

use serde_json::{Map, Value};

/// The whole `templates/settings.hooks.json` file, compiled in so one copy of
/// the block serves the binary wherever it runs. `hook install` takes no
/// `--from`, so there is no other framework root to read it from.
pub const TEMPLATE: &str = include_str!("../../templates/settings.hooks.json");

/// The event keys the block defines, in the order the spec's `--json` `data`
/// fixes. The merge is driven from this list rather than from iteration over
/// the parsed block, because `serde_json::Map` is a `BTreeMap` here and would
/// yield the keys alphabetically.
pub const EVENTS: &[&str] = &[
    "SessionStart",
    "UserPromptExpansion",
    "PreToolUse",
    "PostToolUse",
    "Stop",
    "SubagentStop",
];

/// The two substitution tokens that are per-project rather than per-install.
const TEST_COMMAND_TOKEN: &str = "@@TEST_COMMAND@@";
const VERIFIERS_TOKEN: &str = "@@VERIFIERS@@";

/// Resolve the two per-project tokens in a parsed block.
///
/// `test_commands` are the non-empty `test_command` values of `config.toml`
/// `[[stack]]`; `verifiers` are the `[[verifier]]` subagent names.
///
/// A `"if": "Bash(<rule>)"` filter stops the process spawning on a shell call
/// the handler would ignore, and nothing else: the dispatcher compares the
/// command with `config.toml` itself. A rule that does not render as permission
/// syntax would leave a handler that never fires and a partial gate that dies
/// with no notice, so an unrenderable rule drops the `if` instead of guessing.
/// The same reasoning applies to the `SubagentStop` matcher: with no registered
/// verifier the matcher is dropped and the group fires for every subagent,
/// which is noisier but cannot lose an ingest.
pub fn resolve_project_tokens(block: &mut Value, test_commands: &[String], verifiers: &[String]) {
    let rule = single_rule_command(test_commands);
    let matcher = verifier_matcher(verifiers);

    let Some(hooks) = block.get_mut("hooks").and_then(Value::as_object_mut) else {
        return;
    };

    for (_event, entries) in hooks.iter_mut() {
        let Some(entries) = entries.as_array_mut() else {
            continue;
        };
        for entry in entries.iter_mut() {
            if let Some(m) = entry.get("matcher").and_then(Value::as_str) {
                if m.contains(VERIFIERS_TOKEN) {
                    match &matcher {
                        Some(names) => {
                            entry["matcher"] = Value::String(m.replace(VERIFIERS_TOKEN, names));
                        }
                        None => {
                            if let Some(o) = entry.as_object_mut() {
                                o.remove("matcher");
                            }
                        }
                    }
                }
            }
            let Some(hs) = entry.get_mut("hooks").and_then(Value::as_array_mut) else {
                continue;
            };
            for h in hs.iter_mut() {
                let Some(cond) = h.get("if").and_then(Value::as_str) else {
                    continue;
                };
                if !cond.contains(TEST_COMMAND_TOKEN) {
                    continue;
                }
                match &rule {
                    Some(cmd) => {
                        h["if"] = Value::String(cond.replace(TEST_COMMAND_TOKEN, cmd));
                    }
                    None => {
                        if let Some(o) = h.as_object_mut() {
                            o.remove("if");
                        }
                    }
                }
            }
            // Dropping the `if` rules can leave two handlers that differed only
            // by the rule — the `Bash(...)` and `PowerShell(...)` pair — as the
            // same object, which would spawn the process twice for one tool
            // call. Whatever remains distinct stays.
            dedupe_hooks(hs);
        }
    }
}

/// Remove handlers that are byte-for-byte identical to an earlier one in the
/// same matcher entry, keeping the first.
fn dedupe_hooks(hooks: &mut Vec<Value>) {
    let mut seen: Vec<String> = Vec::new();
    hooks.retain(|h| {
        let key = serde_json::to_string(h).unwrap_or_default();
        if seen.contains(&key) {
            return false;
        }
        seen.push(key);
        true
    });
}

/// The one test command to render into a permission rule, or `None`.
///
/// Zero commands leaves nothing to filter on. More than one distinct command
/// cannot be expressed, because a handler takes one rule and the syntax has no
/// combinators. A command carrying a parenthesis or a newline would not close
/// the rule.
fn single_rule_command(test_commands: &[String]) -> Option<String> {
    let mut distinct: Vec<&str> = Vec::new();
    for c in test_commands {
        let t = c.trim();
        if !t.is_empty() && !distinct.contains(&t) {
            distinct.push(t);
        }
    }
    let [only] = distinct[..] else { return None };
    if only.contains('(') || only.contains(')') || only.contains('\n') || only.contains('\r') {
        return None;
    }
    Some(only.to_string())
}

/// The `SubagentStop` matcher: the registered verifier names joined with `|`,
/// or `None` when none is registered or a name is outside the exact-list
/// character class, where the matcher would be read as a regex.
fn verifier_matcher(verifiers: &[String]) -> Option<String> {
    let mut names: Vec<&str> = Vec::new();
    for v in verifiers {
        let t = v.trim();
        if t.is_empty()
            || !t
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return None;
        }
        if !names.contains(&t) {
            names.push(t);
        }
    }
    if names.is_empty() {
        return None;
    }
    Some(names.join("|"))
}

/// What one merge did: the event keys the block installs, and the event keys
/// that already carried an identical command and were left alone.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MergeReport {
    /// The event keys the block defines, in the spec's order.
    pub events: Vec<String>,
    /// The event keys that already carried one of the block's commands. Each
    /// name appears once however many of its entries were already present.
    pub already_present: Vec<String>,
}

/// Merge `block` into `existing` in place.
///
/// For each event key the block defines, each matcher entry is appended to the
/// existing array when no entry there already carries an identical `command`
/// string. Every other key and every other array element is left untouched, so
/// a hand-written `model`, `permissions`, or `Notification` block survives.
///
/// Key order: `serde_json` is compiled here without its `preserve_order`
/// feature, so an object is a `BTreeMap` and the keys of the file this writes
/// follow `serde_json`'s map ordering rather than the order they were read in.
/// What the merge does guarantee, and what the spec's behaviour rests on, is
/// that no foreign key and no foreign array element is lost.
pub fn merge(existing: &mut Value, block: &Value) -> MergeReport {
    let mut report = MergeReport::default();

    // A settings file whose root is not an object carries nothing to preserve.
    if !existing.is_object() {
        *existing = Value::Object(Map::new());
    }

    let Some(block_hooks) = block.get("hooks").and_then(Value::as_object) else {
        return report;
    };

    let root = match existing.as_object_mut() {
        Some(o) => o,
        None => return report,
    };
    let hooks = root
        .entry("hooks".to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    if !hooks.is_object() {
        *hooks = Value::Object(Map::new());
    }
    let Some(hooks) = hooks.as_object_mut() else {
        return report;
    };

    for event in EVENTS {
        let Some(incoming) = block_hooks.get(*event).and_then(Value::as_array) else {
            continue;
        };
        report.events.push((*event).to_string());

        let target = hooks
            .entry((*event).to_string())
            .or_insert_with(|| Value::Array(Vec::new()));
        if !target.is_array() {
            *target = Value::Array(Vec::new());
        }
        let Some(target) = target.as_array_mut() else {
            continue;
        };

        // A handler this binary wrote in the superseded shell form is removed
        // before the merge, not left beside its replacement.
        //
        // Two things go wrong otherwise. Where `devforgeai` resolves on PATH
        // the token renders as the bare name, so the old entry's whole command
        // string equals the new entry's command plus args: the dedup below
        // reads the new group as already present and drops it, and a project
        // that upgrades never gets the `Bash|PowerShell` matcher, the
        // `NotebookEdit` matcher, or the shell write guard. Where it does not
        // resolve, both entries survive and every hook runs twice, the old one
        // still under the timeout that lets a slow check fail open. Only this
        // binary's own `hook run` handlers match; a hand-written entry and any
        // other tool's are foreign and are left alone.
        target.retain(|entry| !is_superseded_handler(entry));

        // The snapshot is taken once, before anything is appended: the block's
        // two `PostToolUse` entries share a command and differ only by matcher,
        // so comparing against entries appended in the same pass would install
        // the first and drop the second.
        let present: Vec<String> = target.iter().flat_map(entry_commands).collect();

        for entry in incoming {
            let wanted = entry_commands(entry);
            if wanted.iter().any(|c| present.contains(c)) {
                let name = (*event).to_string();
                if !report.already_present.contains(&name) {
                    report.already_present.push(name);
                }
                continue;
            }
            target.push(entry.clone());
        }
    }

    report
}

/// True when every handler of one matcher entry is a `devforgeai hook run`
/// call written in the superseded shell form, with the whole invocation inside
/// `command` and no `args` array.
///
/// Exec form is what tells the two apart: this binary now writes `command` as
/// the executable alone and the arguments in `args`, so an entry carrying a
/// `hook run` inside `command` can only be one this binary wrote before the
/// change, or a hand-written copy of one.
fn is_superseded_handler(entry: &Value) -> bool {
    let Some(hooks) = entry.get("hooks").and_then(Value::as_array) else {
        return false;
    };
    if hooks.is_empty() {
        return false;
    }
    hooks.iter().all(|h| {
        if h.get("args").is_some() {
            return false;
        }
        h.get("command")
            .and_then(Value::as_str)
            .and_then(|c| c.split_once(" hook run "))
            // The event name has to be one this dispatcher answers, so another
            // tool's `hook run <something else>` is not mistaken for ours.
            .map(|(_, event)| crate::hooks::run::EVENTS.contains(&event.trim()))
            .unwrap_or(false)
    })
}

/// The two permission rules the framework needs standing in the project.
///
/// A skill's `allowed-tools` grant covers the invoking turn and clears on the
/// next user message, while a phase spans many turns that each call the binary.
/// Without a standing rule the second turn of every phase stops for a
/// permission prompt on a command the framework itself installed.
pub const PERMISSION_RULES: &[&str] = &["Bash(devforgeai *)", "PowerShell(devforgeai *)"];

/// Add the framework's permission rules to `permissions.allow`, in place.
///
/// Idempotent: a rule already present is left alone, and no other entry, key,
/// or ordering is touched. Returns the rules it added.
pub fn merge_permissions(existing: &mut Value) -> Vec<String> {
    if !existing.is_object() {
        *existing = Value::Object(Map::new());
    }
    let Some(root) = existing.as_object_mut() else {
        return Vec::new();
    };
    let permissions = root
        .entry("permissions".to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    if !permissions.is_object() {
        *permissions = Value::Object(Map::new());
    }
    let Some(permissions) = permissions.as_object_mut() else {
        return Vec::new();
    };
    let allow = permissions
        .entry("allow".to_string())
        .or_insert_with(|| Value::Array(Vec::new()));
    if !allow.is_array() {
        *allow = Value::Array(Vec::new());
    }
    let Some(allow) = allow.as_array_mut() else {
        return Vec::new();
    };

    let mut added = Vec::new();
    for rule in PERMISSION_RULES {
        let present = allow
            .iter()
            .filter_map(Value::as_str)
            .any(|v| v.trim() == *rule);
        if !present {
            allow.push(Value::String((*rule).to_string()));
            added.push((*rule).to_string());
        }
    }
    added
}

/// The identity of every hook carried by one matcher entry's `hooks` array.
///
/// In exec form every handler shares one `command`, the binary, and is told
/// apart by `args`, so the identity is the command followed by its arguments.
/// Comparing `command` alone would read the second handler of a group as a
/// duplicate of the first and drop it.
fn entry_commands(entry: &Value) -> Vec<String> {
    entry
        .get("hooks")
        .and_then(Value::as_array)
        .map(|hs| hs.iter().map(hook_identity).collect())
        .unwrap_or_default()
}

/// One hook's `command` plus its `args`, joined with spaces.
fn hook_identity(h: &Value) -> String {
    let command = h.get("command").and_then(Value::as_str).unwrap_or_default();
    let args: Vec<&str> = h
        .get("args")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    if args.is_empty() {
        return command.to_string();
    }
    format!("{command} {}", args.join(" "))
}

/// Render a settings document the way `hook install` writes it: two-space
/// indentation and a closing newline.
pub fn render(settings: &Value) -> String {
    let mut s = serde_json::to_string_pretty(settings).unwrap_or_else(|_| "{}".to_string());
    s.push('\n');
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block() -> Value {
        serde_json::from_str(&crate::hooks::githooks::substitute(TEMPLATE, "devforgeai"))
            .expect("the compiled-in template parses")
    }

    #[test]
    fn the_template_defines_the_events_the_spec_names() {
        let b = block();
        let hooks = b["hooks"].as_object().expect("hooks object");
        assert_eq!(hooks.len(), EVENTS.len());
        for e in EVENTS {
            assert!(hooks.contains_key(*e), "{e} is defined");
        }
        assert_eq!(
            b["hooks"]["PostToolUse"].as_array().expect("array").len(),
            2,
            "PostToolUse carries the file entry and the shell entry"
        );
        assert_eq!(
            b["hooks"]["PreToolUse"].as_array().expect("array").len(),
            3,
            "PreToolUse carries the write guard, the shell guard, and trust-check"
        );
    }

    #[test]
    fn every_handler_is_exec_form_naming_a_dispatcher_event() {
        let b = block();
        let hooks = b["hooks"].as_object().expect("hooks object");
        let mut seen = 0usize;
        for entries in hooks.values() {
            for entry in entries.as_array().expect("array") {
                for h in entry["hooks"].as_array().expect("array") {
                    assert_eq!(h["type"], "command");
                    assert_eq!(
                        h["command"], "devforgeai",
                        "exec form spawns the binary directly, with no shell"
                    );
                    let args: Vec<&str> = h["args"]
                        .as_array()
                        .expect("args")
                        .iter()
                        .filter_map(Value::as_str)
                        .collect();
                    assert_eq!(&args[..2], &["hook", "run"]);
                    assert!(
                        crate::hooks::run::EVENTS.contains(&args[2]),
                        "{} is a dispatcher event",
                        args[2]
                    );
                    seen += 1;
                }
            }
        }
        assert!(seen >= 10, "every handler was inspected");
    }

    #[test]
    fn the_shell_rule_is_dropped_when_no_single_test_command_renders() {
        let mut b = block();
        resolve_project_tokens(&mut b, &[], &[]);
        let shell = &b["hooks"]["PostToolUse"][1]["hooks"][0];
        assert!(
            shell.get("if").is_none(),
            "an unrenderable rule drops the filter rather than silencing the handler"
        );
        assert!(
            b["hooks"]["SubagentStop"][0].get("matcher").is_none(),
            "with no registered verifier the group fires for every subagent"
        );
    }

    #[test]
    fn the_shell_rule_and_the_verifier_matcher_render_from_the_project() {
        let mut b = block();
        resolve_project_tokens(
            &mut b,
            &["cargo test".to_string()],
            &["kill-case-builder".to_string(), "ac-verifier".to_string()],
        );
        assert_eq!(
            b["hooks"]["PostToolUse"][1]["hooks"][0]["if"],
            "Bash(cargo test)"
        );
        assert_eq!(
            b["hooks"]["PostToolUse"][1]["hooks"][1]["if"],
            "PowerShell(cargo test)"
        );
        assert_eq!(
            b["hooks"]["SubagentStop"][0]["matcher"],
            "kill-case-builder|ac-verifier"
        );
    }

    #[test]
    fn a_command_that_cannot_close_the_rule_drops_the_filter() {
        assert_eq!(
            single_rule_command(&["cargo test".into()]),
            Some("cargo test".into())
        );
        assert_eq!(single_rule_command(&["a".into(), "b".into()]), None);
        assert_eq!(single_rule_command(&["npm run test(x)".into()]), None);
        assert_eq!(single_rule_command(&[]), None);
    }

    #[test]
    fn merge_into_an_empty_document_installs_every_event_in_order() {
        let mut existing = Value::Object(Map::new());
        let r = merge(&mut existing, &block());
        assert_eq!(
            r.events,
            EVENTS.iter().map(|e| e.to_string()).collect::<Vec<_>>()
        );
        assert!(r.already_present.is_empty());
    }

    #[test]
    fn a_second_merge_appends_nothing_and_reports_every_event() {
        let mut existing = Value::Object(Map::new());
        merge(&mut existing, &block());
        let before = existing.clone();

        let r = merge(&mut existing, &block());
        assert_eq!(existing, before, "idempotent");
        assert_eq!(r.already_present.len(), EVENTS.len());
    }

    #[test]
    fn render_indents_with_two_spaces() {
        let v: Value = serde_json::from_str(r#"{"a":{"b":1}}"#).expect("parse");
        assert_eq!(render(&v), "{\n  \"a\": {\n    \"b\": 1\n  }\n}\n");
    }
}
