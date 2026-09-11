//! `design lint [<paths>...] [--tokens]`.

use crate::ctx::Ctx;
use crate::design::{self, Tokens, Violation};
use crate::errors::{CliError, Diag};
use crate::{story, Outcome};
use std::path::{Path, PathBuf};

/// The frontend rules, in the spec's table order.
struct Rules {
    hex: regex::Regex,
    functional: regex::Regex,
    named: regex::Regex,
    font_size: regex::Regex,
    font_family: regex::Regex,
    line_height: regex::Regex,
    token_ref: regex::Regex,
}

impl Rules {
    fn new() -> Rules {
        let names = design::NAMED_COLOURS.join("|");
        Rules {
            hex: regex::Regex::new(r"#[0-9a-fA-F]{3,8}\b").expect("the hex rule compiles"),
            functional: regex::Regex::new(r"(?i)\b(rgba?|hsla?)\(")
                .expect("the functional-colour rule compiles"),
            named: regex::Regex::new(&format!(r"(?i)\b({names})\b"))
                .expect("the named-colour rule compiles"),
            font_size: regex::Regex::new(r"font-size\s*:\s*([0-9.]+(px|rem|em|pt))")
                .expect("the font-size rule compiles"),
            font_family: regex::Regex::new(r"font-family\s*:\s*([^v\n][^;\n}]*)")
                .expect("the font-family rule compiles"),
            line_height: regex::Regex::new(r"line-height\s*:\s*([0-9.]+)")
                .expect("the line-height rule compiles"),
            token_ref: regex::Regex::new(r"var\(--([a-z0-9-]+)\)")
                .expect("the token-reference rule compiles"),
        }
    }
}

/// Lint the frontend file set, or the token file itself under `--tokens`.
pub fn lint(ctx: &mut Ctx, paths: &[PathBuf], tokens_mode: bool) -> Result<Outcome, CliError> {
    if tokens_mode && !paths.is_empty() {
        return Err(CliError::new(
            "DFA-E010",
            "unknown option '<paths>'; 'design lint --tokens' reads no frontend file",
        ));
    }
    // The token file is a document and lives under `.devforgeai/` in the main
    // checkout; the files it lints are source, and during a worktree build the
    // source is in the worktree. The two halves resolve against different
    // directories for that reason.
    let root = ctx.root.clone();
    let source = ctx.source_root();
    let frontend = ctx.config()?.frontend.clone();
    let (tokens, raw) = design::load_tokens(&root, &frontend.tokens_path)?;

    let (mode, files, skipped, violations) = if tokens_mode {
        let (files, v) = lint_tokens(&root, &raw, &tokens);
        ("tokens", files, 0, v)
    } else {
        let (set, skipped) = file_set(&source, &frontend, paths);
        let v = lint_files(&source, &set, &tokens);
        ("paths", set.len(), skipped, v)
    };

    let warnings: Vec<Diag> = violations
        .iter()
        .map(|v| Diag::at_line(v.code, v.message.clone(), v.path.clone(), v.line))
        .collect();

    Ok(Outcome {
        human: vec![format!(
            "design lint  {files} files · {} {}",
            violations.len(),
            if violations.len() == 1 {
                "violation"
            } else {
                "violations"
            }
        )],
        data: serde_json::json!({
            "mode": mode,
            "files": files,
            "skipped": skipped,
            "violations": violations.iter().map(|v| serde_json::json!({
                "path": v.path,
                "line": v.line,
                "code": v.code,
                "value": v.value,
                "nearest": v.nearest,
            })).collect::<Vec<_>>(),
        }),
        warnings,
        degraded: ctx.degraded(),
        project: root.display().to_string(),
        exit: Some(if violations.is_empty() { 0 } else { 1 }),
        raw: None,
        stderr: Vec::new(),
        ..Default::default()
    })
}

/// The file set and the count of paths the globs skipped.
fn file_set(
    root: &Path,
    frontend: &crate::config::Frontend,
    paths: &[PathBuf],
) -> (Vec<String>, usize) {
    let include = globs(&frontend.globs);
    let exclude = globs(&frontend.exclude);
    let matches = |p: &str| include.is_match(p) && !exclude.is_match(p);

    if paths.is_empty() {
        let mut out: Vec<String> = walk(root).into_iter().filter(|p| matches(p)).collect();
        out.sort();
        out.dedup();
        return (out, 0);
    }

    let mut kept = Vec::new();
    let mut skipped = 0usize;
    for p in paths {
        let rel = crate::cmd::story::repo_relative(root, p);
        if matches(&rel) {
            kept.push(rel);
        } else {
            skipped += 1;
        }
    }
    kept.sort();
    kept.dedup();
    (kept, skipped)
}

/// A `GlobSet` over the patterns, skipping any that does not compile.
fn globs(patterns: &[String]) -> globset::GlobSet {
    let mut b = globset::GlobSetBuilder::new();
    for p in patterns {
        if let Ok(g) = globset::Glob::new(p) {
            b.add(g);
        }
    }
    b.build().unwrap_or_else(|_| globset::GlobSet::empty())
}

/// Every file under the project root, project-relative, skipping `.git/`.
fn walk(root: &Path) -> Vec<String> {
    walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| e.file_name() != ".git" && e.file_name() != "target")
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| crate::project::rel_display(root, e.path()))
        .collect()
}

/// Apply the five rules to every file of the set.
fn lint_files(root: &Path, set: &[String], tokens: &Tokens) -> Vec<Violation> {
    let rules = Rules::new();
    let mut out = Vec::new();
    for rel in set {
        let Ok(bytes) = std::fs::read(root.join(rel)) else {
            continue;
        };
        let text = String::from_utf8_lossy(&bytes);
        for (i, line) in text.lines().enumerate() {
            out.extend(lint_line(&rules, tokens, rel, i as u32 + 1, line));
        }
    }
    out
}

/// The five rules over one line.
fn lint_line(
    rules: &Rules,
    tokens: &Tokens,
    rel: &str,
    line_no: u32,
    line: &str,
) -> Vec<Violation> {
    let mut out = Vec::new();

    let colour = |value: &str| Violation {
        path: rel.to_string(),
        line: line_no,
        code: "DFA-E240",
        value: value.to_string(),
        nearest: design::nearest_colour(tokens, value),
        message: format!(
            "{rel}:{line_no} uses the literal colour '{value}'; brand/tokens.json defines {}",
            design::nearest_colour(tokens, value)
        ),
    };

    for m in rules.hex.find_iter(line) {
        out.push(colour(m.as_str()));
    }
    for m in rules.functional.find_iter(line) {
        out.push(colour(m.as_str().trim_end_matches('(')));
    }
    for m in rules.named.find_iter(line) {
        // A name that is part of a longer identifier, as in `--color-red` or
        // `red-500`, is not a value.
        let before = line[..m.start()].chars().next_back();
        let after = line[m.end()..].chars().next();
        let glued = |c: Option<char>| matches!(c, Some('-') | Some('_'));
        if glued(before) || glued(after) {
            continue;
        }
        out.push(colour(m.as_str()));
    }

    for (re, property) in [
        (&rules.font_size, "font-size"),
        (&rules.font_family, "font-family"),
        (&rules.line_height, "line-height"),
    ] {
        for caps in re.captures_iter(line) {
            let value = caps.get(1).map(|m| m.as_str().trim()).unwrap_or("");
            if value.is_empty() || design::ALLOWED_KEYWORDS.contains(&value.to_lowercase().as_str())
            {
                continue;
            }
            // Every one of the three properties resolves against the `type`
            // group; the spec names no separate group for line height.
            let nearest = design::nearest_type(tokens, value);
            out.push(Violation {
                path: rel.to_string(),
                line: line_no,
                code: "DFA-E241",
                value: value.to_string(),
                nearest: nearest.clone(),
                message: format!(
                    "{rel}:{line_no} uses the literal {property} '{value}'; brand/tokens.json defines {nearest}"
                ),
            });
        }
    }

    for caps in rules.token_ref.captures_iter(line) {
        let name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        if tokens.by_property(name).is_some() {
            continue;
        }
        out.push(Violation {
            path: rel.to_string(),
            line: line_no,
            code: "DFA-E242",
            value: format!("--{name}"),
            nearest: String::new(),
            message: format!(
                "{rel}:{line_no} references token '--{name}', which brand/tokens.json does not define"
            ),
        });
    }

    out
}

// ------------------------------------------------------------------ --tokens

/// Check the token file's own shape and the UI specs that reference it.
fn lint_tokens(root: &Path, raw: &serde_json::Value, tokens: &Tokens) -> (usize, Vec<Violation>) {
    let rel = ".devforgeai/brand/tokens.json";
    let mut out = Vec::new();
    let leaf_name = regex::Regex::new(r"^[a-z][a-z0-9-]*$").expect("the leaf rule compiles");

    let shape = |group: &str, leaf: &str, found: &str, expected: &str| Violation {
        path: rel.to_string(),
        line: 0,
        code: "DFA-E243",
        value: format!("{group}.{leaf}"),
        nearest: String::new(),
        message: format!(
            "brand/tokens.json group '{group}' leaf '{leaf}' is {found}, expected {expected}"
        ),
    };

    match raw.as_object() {
        None => out.push(shape("", "", "not an object", "an object")),
        Some(obj) => {
            for key in obj.keys() {
                if key != "meta" && !design::GROUPS.contains(&key.as_str()) {
                    out.push(shape(
                        key,
                        "",
                        "an unknown top-level key",
                        "meta or one of the six groups",
                    ));
                }
            }
            for group in design::GROUPS {
                if !obj.contains_key(*group) {
                    out.push(shape(group, "", "absent", "an object"));
                }
            }

            // `meta` carries the seven frontmatter keys.
            match obj.get("meta").and_then(|m| m.as_object()) {
                None => out.push(shape("meta", "", "absent or not an object", "an object")),
                Some(meta) => {
                    for key in crate::doc::frontmatter::KEYS {
                        if !meta.contains_key(*key) {
                            out.push(shape("meta", key, "absent", "present"));
                        }
                    }
                }
            }

            for group in design::GROUPS {
                let Some(leaves) = obj.get(*group).and_then(|g| g.as_object()) else {
                    continue;
                };
                for (leaf, value) in leaves {
                    if !leaf_name.is_match(leaf) {
                        out.push(shape(
                            group,
                            leaf,
                            "a name outside the pattern",
                            "^[a-z][a-z0-9-]*$",
                        ));
                    }
                    if *group == "color" {
                        let ok = value
                            .as_object()
                            .map(|o| {
                                o.len() == 2
                                    && o.get("light").and_then(|x| x.as_str()).is_some()
                                    && o.get("dark").and_then(|x| x.as_str()).is_some()
                            })
                            .unwrap_or(false);
                        if !ok {
                            out.push(shape(
                                group,
                                leaf,
                                "not a {light, dark} object",
                                "a {light, dark} object of strings",
                            ));
                        }
                    } else if !value.is_string() {
                        out.push(shape(group, leaf, "not a string", "a string"));
                    }
                }
            }
        }
    }

    // Every `TOKEN-...` in a UI spec resolves.
    let token_ref = regex::Regex::new(r"TOKEN-[a-z0-9-]+").expect("the token rule compiles");
    let dir = root.join(".devforgeai").join("ui-specs");
    let mut specs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n.starts_with("UI-") && n.ends_with(".md"))
                        .unwrap_or(false)
                })
                .collect()
        })
        .unwrap_or_default();
    specs.sort();

    for path in &specs {
        let spec_rel = crate::project::rel_display(root, path);
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        for (i, line) in text.lines().enumerate() {
            for m in token_ref.find_iter(line) {
                if tokens.has(m.as_str()) {
                    continue;
                }
                out.push(Violation {
                    path: spec_rel.clone(),
                    line: i as u32 + 1,
                    code: "DFA-E244",
                    value: m.as_str().to_string(),
                    nearest: String::new(),
                    message: format!(
                        "{spec_rel}:{} references token '{}', which brand/tokens.json does not define",
                        i + 1,
                        m.as_str()
                    ),
                });
            }
        }
    }

    (specs.len(), out)
}

/// A path made repo-relative, shared with `story files --check`.
pub fn repo_relative(root: &Path, path: &Path) -> String {
    story::normalise_path(&crate::cmd::story::repo_relative(root, path))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens() -> Tokens {
        design::flatten(&serde_json::json!({
            "color": { "primary": { "light": "#3355ff", "dark": "#7788ff" } },
            "type": { "body": "1rem" }
        }))
    }

    fn lint_one(line: &str) -> Vec<Violation> {
        lint_line(&Rules::new(), &tokens(), "src/a.css", 1, line)
    }

    #[test]
    fn a_hex_colour_is_a_violation_naming_the_nearest_token() {
        let v = lint_one("color: #3356ff;");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].code, "DFA-E240");
        assert_eq!(v[0].nearest, "TOKEN-color-primary");
    }

    #[test]
    fn a_token_reference_passes() {
        assert!(lint_one("color: var(--color-primary);").is_empty());
    }

    #[test]
    fn an_unknown_token_reference_is_e242() {
        let v = lint_one("color: var(--color-nowhere);");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].code, "DFA-E242");
    }

    #[test]
    fn a_keyword_passes() {
        assert!(lint_one("background: transparent;").is_empty());
        assert!(lint_one("color: currentColor;").is_empty());
    }

    #[test]
    fn a_named_colour_inside_an_identifier_is_not_a_value() {
        // The custom property is an unknown token, which is its own rule; the
        // point here is that the `red` inside the name is not a literal colour.
        let v = lint_one("color: var(--color-red);");
        assert!(v.iter().all(|x| x.code == "DFA-E242"), "{v:?}");
        assert!(lint_one("class=\"text-red-500\"").is_empty());
        assert_eq!(lint_one("color: red;").len(), 1);
    }

    #[test]
    fn a_functional_colour_is_a_violation() {
        assert_eq!(lint_one("color: rgba(1,2,3,0.5);").len(), 1);
        assert_eq!(lint_one("color: hsl(1 2% 3%);").len(), 1);
    }

    #[test]
    fn a_literal_font_size_is_e241() {
        let v = lint_one("font-size: 17px;");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].code, "DFA-E241");
        assert_eq!(v[0].nearest, "TOKEN-type-body");
    }

    #[test]
    fn a_font_size_taken_from_a_token_passes() {
        assert!(lint_one("font-size: var(--type-body);").is_empty());
    }

    #[test]
    fn a_literal_line_height_is_e241_and_normal_passes() {
        assert_eq!(lint_one("line-height: 1.5;").len(), 1);
        assert!(lint_one("line-height: normal;").is_empty());
    }

    #[test]
    fn a_literal_font_family_is_e241() {
        let v = lint_one("font-family: Inter, sans-serif;");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].code, "DFA-E241");
    }
}
