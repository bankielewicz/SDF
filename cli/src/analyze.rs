//! `init --analyze`: the brownfield draft of the six context files.
//!
//! Three files are derived from code, three are stubs whose body the Constitute
//! skill fills. All six carry `status: draft`, so `context audit` CA-2 keeps
//! them out of a commit until the user accepts them.

use crate::config::Config;
use crate::errors::CliError;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The directory names the walk never descends into.
pub const EXCLUDED_DIRS: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    "out",
    "vendor",
    ".venv",
    "__pycache__",
    "bin",
    "obj",
];

/// The walk stops at this depth.
pub const MAX_DEPTH: usize = 6;

/// Each file is read to this cap.
pub const READ_CAP: usize = 256 * 1024;

/// The whole analysis stops after this many seconds.
pub const TIME_CAP_SECS: u64 = 120;

/// The three files derived from code.
pub const DERIVED: &[&str] = &["tech-stack", "source-tree", "dependencies"];

/// The three files whose body the Constitute skill fills.
pub const STUBS: &[&str] = &[
    "coding-standards",
    "architecture-constraints",
    "anti-patterns",
];

/// The `open_questions` entry every stub carries.
pub const STUB_QUESTION: &str = "body drafted by the Constitute skill";

/// The body line every stub section carries.
pub const STUB_LINE: &str = "Drafted by the Constitute skill.";

/// The formatter and linter configuration files the stub notes.
const STYLE_CONFIGS: &[&str] = &[
    ".editorconfig",
    "rustfmt.toml",
    ".prettierrc",
    ".prettierrc.json",
    ".prettierrc.yaml",
    ".prettierrc.yml",
    ".eslintrc",
    ".eslintrc.json",
    ".eslintrc.yaml",
    ".eslintrc.yml",
    ".eslintrc.cjs",
    "ruff.toml",
    ".rubocop.yml",
    "checkstyle.xml",
];

/// The manifests `## Direct dependencies` reads.
const MANIFESTS: &[&str] = &[
    "Cargo.toml",
    "package.json",
    "pyproject.toml",
    "requirements.txt",
    "go.mod",
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
    "Gemfile",
];

/// The lockfile of each ecosystem, by manifest name.
const LOCKFILES: &[(&str, &str)] = &[
    ("Cargo.toml", "Cargo.lock"),
    ("package.json", "package-lock.json"),
    ("pyproject.toml", "poetry.lock"),
    ("requirements.txt", "requirements.lock"),
    ("go.mod", "go.sum"),
    ("Gemfile", "Gemfile.lock"),
];

/// What the analysis did.
#[derive(Debug, Clone, Default)]
pub struct Analysis {
    /// The stems written from code.
    pub derived: Vec<String>,
    /// The stems written as stubs.
    pub stubs: Vec<String>,
    /// The stems left untouched because a file already existed.
    pub skipped: Vec<String>,
    /// The files the walk read.
    pub files_scanned: usize,
    /// True when the time cap stopped the walk.
    pub truncated: bool,
}

impl Analysis {
    /// The `data.analyze` object.
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "derived": self.derived,
            "stubs": self.stubs,
            "skipped": self.skipped,
            "files_scanned": self.files_scanned,
            "truncated": self.truncated,
        })
    }
}

/// One file the walk saw.
struct Seen {
    rel: String,
    depth: usize,
}

/// Walk the project, honouring the depth, the exclusions, and the time cap.
fn walk(root: &Path, cfg: &Config) -> (Vec<Seen>, bool) {
    let started = std::time::Instant::now();
    let exclude = {
        let mut b = globset::GlobSetBuilder::new();
        for p in &cfg.coverage.exclude {
            if let Ok(g) = globset::Glob::new(p) {
                b.add(g);
            }
        }
        b.build().unwrap_or_else(|_| globset::GlobSet::empty())
    };

    let mut out = Vec::new();
    let mut truncated = false;
    let walker = walkdir::WalkDir::new(root)
        .max_depth(MAX_DEPTH)
        .into_iter()
        .filter_entry(|e| {
            !(e.file_type().is_dir()
                && e.depth() > 0
                && e.file_name()
                    .to_str()
                    .map(|n| EXCLUDED_DIRS.contains(&n))
                    .unwrap_or(false))
        });

    for entry in walker.filter_map(Result::ok) {
        if started.elapsed().as_secs() >= TIME_CAP_SECS {
            truncated = true;
            break;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = crate::project::rel_display(root, entry.path());
        if exclude.is_match(&rel) {
            continue;
        }
        out.push(Seen {
            rel,
            depth: entry.depth(),
        });
    }
    (out, truncated)
}

/// Read a file to the cap, `""` when it cannot be read.
fn read_capped(path: &Path) -> String {
    let Ok(bytes) = std::fs::read(path) else {
        return String::new();
    };
    let cut = bytes.len().min(READ_CAP);
    String::from_utf8_lossy(&bytes[..cut]).to_string()
}

/// Run the analysis and write the six files.
pub fn run(root: &Path, cfg: &Config, force: bool) -> Result<Analysis, CliError> {
    let (files, truncated) = walk(root, cfg);
    let mut a = Analysis {
        files_scanned: files.len(),
        truncated,
        ..Default::default()
    };

    let extra_question = if truncated {
        Some(format!(
            "analysis truncated after {} files",
            a.files_scanned
        ))
    } else {
        None
    };

    let bodies: BTreeMap<&str, String> = BTreeMap::from([
        ("tech-stack", tech_stack(root, cfg)),
        ("source-tree", source_tree(root, cfg, &files)),
        ("dependencies", dependencies(root, &files)),
        ("coding-standards", coding_standards(root, &files)),
        (
            "architecture-constraints",
            architecture_constraints(root, cfg),
        ),
        ("anti-patterns", anti_patterns(root, &files)),
    ]);

    for stem in DERIVED.iter().chain(STUBS) {
        let path = root
            .join(".devforgeai")
            .join("context")
            .join(format!("{stem}.md"));
        if path.exists() && !force {
            a.skipped.push((*stem).to_string());
            continue;
        }

        let mut questions: Vec<String> = Vec::new();
        if STUBS.contains(stem) {
            questions.push(STUB_QUESTION.to_string());
        }
        if let Some(q) = &extra_question {
            questions.push(q.clone());
        }

        let body = bodies.get(stem).cloned().unwrap_or_default();
        let text = format!("{}{body}", frontmatter(stem, &questions));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CliError::io("creating", parent.display(), &e))?;
        }
        crate::project::atomic_write(&path, text.as_bytes())?;

        if DERIVED.contains(stem) {
            a.derived.push((*stem).to_string());
        } else {
            a.stubs.push((*stem).to_string());
        }
    }

    a.derived
        .sort_by_key(|s| DERIVED.iter().position(|d| d == s).unwrap_or(usize::MAX));
    a.stubs
        .sort_by_key(|s| STUBS.iter().position(|d| d == s).unwrap_or(usize::MAX));
    Ok(a)
}

/// The §5 frontmatter of one context file.
fn frontmatter(stem: &str, questions: &[String]) -> String {
    let list = if questions.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", questions.join(", "))
    };
    format!(
        "---\nschema: devforgeai/context-{stem}/1\nid: {stem}\nphase: constitute\nstatus: draft\nproduced_by: establishing-context\nconsumes: []\nopen_questions: {list}\n---\n\n# {stem}\n"
    )
}

// ----------------------------------------------------------------- derived

/// The runtime pin files, by the key each carries.
fn runtime_pins(root: &Path) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let read = |name: &str| -> Option<String> {
        let p = root.join(name);
        p.is_file().then(|| read_capped(&p))
    };

    if let Some(text) = read("rust-toolchain.toml") {
        let value = text
            .lines()
            .find_map(|l| l.split_once("channel"))
            .map(|(_, v)| {
                v.trim_start_matches([' ', '=', '"'])
                    .trim_end_matches('"')
                    .trim()
                    .to_string()
            })
            .unwrap_or_default();
        out.push(("rust-toolchain.toml".into(), value));
    }
    if let Some(text) = read(".nvmrc") {
        out.push((".nvmrc".into(), text.trim().to_string()));
    }
    if let Some(text) = read("package.json") {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(engines) = v.get("engines").and_then(|e| e.as_object()) {
                for (k, val) in engines {
                    out.push((
                        format!("package.json engines.{k}"),
                        val.as_str().unwrap_or("").to_string(),
                    ));
                }
            }
        }
    }
    if let Some(text) = read(".python-version") {
        out.push((".python-version".into(), text.trim().to_string()));
    }
    if let Some(text) = read("go.mod") {
        if let Some(line) = text.lines().find(|l| l.trim_start().starts_with("go ")) {
            out.push((
                "go.mod".into(),
                line.trim().trim_start_matches("go ").trim().to_string(),
            ));
        }
    }
    if let Some(text) = read("global.json") {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            let value = v
                .get("sdk")
                .and_then(|s| s.get("version"))
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            out.push(("global.json".into(), value));
        }
    }
    if let Some(text) = read(".ruby-version") {
        out.push((".ruby-version".into(), text.trim().to_string()));
    }
    if let Some(text) = read("pom.xml") {
        if let Some(at) = text.find("<maven.compiler.release>") {
            let rest = &text[at + "<maven.compiler.release>".len()..];
            let value = rest.split('<').next().unwrap_or("").trim().to_string();
            out.push(("pom.xml maven.compiler.release".into(), value));
        }
    }
    out
}

/// `context/tech-stack.md`, derived.
fn tech_stack(root: &Path, cfg: &Config) -> String {
    let mut s = String::new();

    s.push_str("\n## Languages\n\n| Stack | Markers |\n|---|---|\n");
    if cfg.stack.is_empty() {
        s.push_str("| none detected | none |\n");
    } else {
        for st in &cfg.stack {
            s.push_str(&format!(
                "| {} | {} |\n",
                st.id,
                join_or(&st.markers, "none")
            ));
        }
    }

    s.push_str("\n## Package managers\n\n| Stack | Manager | Lockfile |\n|---|---|---|\n");
    if cfg.stack.is_empty() {
        s.push_str("| none detected | none | absent |\n");
    } else {
        for st in &cfg.stack {
            let lock = LOCKFILES
                .iter()
                .find(|(m, _)| st.markers.iter().any(|x| x == m))
                .map(|(_, l)| *l)
                .unwrap_or("");
            let present = !lock.is_empty() && root.join(lock).is_file();
            s.push_str(&format!(
                "| {} | {} | {} |\n",
                st.id,
                blank_as(&st.package_manager, "none detected"),
                if present { lock } else { "absent" }
            ));
        }
    }

    s.push_str("\n## Test tooling\n\n| Stack | Test command | Coverage command | Format |\n|---|---|---|---|\n");
    for st in &cfg.stack {
        s.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            st.id,
            blank_as(&st.test_command, "none detected"),
            blank_as(&st.coverage_command, "none detected"),
            blank_as(&st.coverage_format, "none")
        ));
    }
    if cfg.stack.is_empty() {
        s.push_str("| none detected | none detected | none detected | none |\n");
    }

    s.push_str("\n## Lint tooling\n\n| Stack | Lint command |\n|---|---|\n");
    for st in &cfg.stack {
        s.push_str(&format!(
            "| {} | {} |\n",
            st.id,
            blank_as(&st.lint_command, "none detected")
        ));
    }
    if cfg.stack.is_empty() {
        s.push_str("| none detected | none detected |\n");
    }

    let pins = runtime_pins(root);
    s.push_str("\n## Runtime versions\n\n| Source | Version |\n|---|---|\n");
    if pins.is_empty() {
        s.push_str("| none detected | none |\n");
    } else {
        for (source, value) in &pins {
            s.push_str(&format!("| {source} | {} |\n", blank_as(value, "unset")));
        }
    }

    s.push_str("\n## Open\n\n");
    let mut open: Vec<String> = Vec::new();
    if cfg.stack.is_empty() {
        open.push("no stack was detected".to_string());
    }
    for st in &cfg.stack {
        if st.lint_command.trim().is_empty() {
            open.push(format!("stack '{}' has no lint command", st.id));
        }
        if st.coverage_command.trim().is_empty() {
            open.push(format!("stack '{}' has no coverage command", st.id));
        }
    }
    if pins.is_empty() {
        open.push("no runtime version is pinned".to_string());
    }
    if open.is_empty() {
        s.push_str("Nothing unresolved.\n");
    } else {
        for o in &open {
            s.push_str(&format!("- {o}\n"));
        }
    }
    s
}

/// `context/source-tree.md`, derived.
fn source_tree(root: &Path, cfg: &Config, files: &[Seen]) -> String {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for f in files {
        let dir = f.rel.rsplit_once('/').map(|(d, _)| d).unwrap_or(".");
        // Depth 3 of the tree, counted from the project root.
        let trimmed: Vec<&str> = dir.split('/').take(3).collect();
        *counts.entry(trimmed.join("/")).or_insert(0) += 1;
        let _ = f.depth;
    }

    let mut s = String::new();
    s.push_str("\n## Tree\n\n| Directory | Files |\n|---|---|\n");
    for (dir, n) in &counts {
        s.push_str(&format!("| {dir} | {n} |\n"));
    }
    if counts.is_empty() {
        s.push_str("| . | 0 |\n");
    }

    let layers = crate::coverage::layer_order(cfg);
    let matchers: Vec<globset::GlobSet> = layers
        .iter()
        .map(|(_, globs, _)| {
            let mut b = globset::GlobSetBuilder::new();
            for g in globs {
                if let Ok(c) = globset::Glob::new(g) {
                    b.add(c);
                }
            }
            b.build().unwrap_or_else(|_| globset::GlobSet::empty())
        })
        .collect();

    s.push_str("\n## Layers\n\n| Directory | Layer |\n|---|---|\n");
    for dir in counts.keys() {
        let probe = format!("{dir}/x");
        let layer = matchers
            .iter()
            .position(|m| m.is_match(&probe) || m.is_match(dir.as_str()))
            .map(|i| layers[i].0.clone())
            .unwrap_or_else(|| "unassigned".to_string());
        s.push_str(&format!("| {dir} | {layer} |\n"));
    }
    if counts.is_empty() {
        s.push_str("| . | unassigned |\n");
    }

    s.push_str("\n## Entry points\n\n");
    let entries: Vec<&str> = files
        .iter()
        .map(|f| f.rel.as_str())
        .filter(|rel| is_entry_point(rel))
        .collect();
    if entries.is_empty() {
        s.push_str("None detected.\n");
    } else {
        for e in &entries {
            s.push_str(&format!("- {e}\n"));
        }
    }

    s.push_str("\n## Test roots\n\n");
    let mut test_dirs: Vec<String> = Vec::new();
    let mut b = globset::GlobSetBuilder::new();
    for p in &cfg.coverage.exclude {
        if let Ok(g) = globset::Glob::new(p) {
            b.add(g);
        }
    }
    let tests = b.build().unwrap_or_else(|_| globset::GlobSet::empty());
    for entry in walkdir::WalkDir::new(root)
        .max_depth(MAX_DEPTH)
        .into_iter()
        .filter_entry(|e| {
            !(e.file_type().is_dir()
                && e.depth() > 0
                && e.file_name()
                    .to_str()
                    .map(|n| EXCLUDED_DIRS.contains(&n))
                    .unwrap_or(false))
        })
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_dir() && e.depth() > 0)
    {
        let rel = crate::project::rel_display(root, entry.path());
        if tests.is_match(format!("{rel}/x")) && !test_dirs.contains(&rel) {
            test_dirs.push(rel);
        }
    }
    if test_dirs.is_empty() {
        s.push_str("None detected.\n");
    } else {
        test_dirs.sort();
        for d in &test_dirs {
            s.push_str(&format!("- {d}\n"));
        }
    }
    let _ = root;
    s
}

/// `main.*`, `index.*`, `Program.cs`, `app.*`, `cmd/*/main.go`.
fn is_entry_point(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    let stem = name.rsplit_once('.').map(|(s, _)| s).unwrap_or(name);
    if name == "Program.cs" {
        return true;
    }
    if rel.starts_with("cmd/") && name == "main.go" {
        return true;
    }
    matches!(stem, "main" | "index" | "app")
}

/// `context/dependencies.md`, derived.
fn dependencies(root: &Path, files: &[Seen]) -> String {
    let mut rows: Vec<(String, String, String)> = Vec::new();
    let mut unverified: Vec<(String, String)> = Vec::new();
    let mut manifests: Vec<String> = files
        .iter()
        .map(|f| f.rel.clone())
        .filter(|rel| {
            let name = rel.rsplit('/').next().unwrap_or(rel);
            MANIFESTS.contains(&name) || (name.ends_with(".csproj"))
        })
        .collect();
    manifests.sort();

    for rel in &manifests {
        let text = read_capped(&root.join(rel));
        let name = rel.rsplit('/').next().unwrap_or(rel);
        for (dep, version) in parse_manifest(name, &text) {
            if version.is_empty() || version == "*" {
                unverified.push((dep.clone(), rel.clone()));
            }
            rows.push((dep, version, rel.clone()));
        }
    }

    let mut s = String::new();
    s.push_str("\n## Direct dependencies\n\n| Name | Version | Manifest |\n|---|---|---|\n");
    if rows.is_empty() {
        s.push_str("| none detected | none | none |\n");
    } else {
        for (name, version, manifest) in &rows {
            s.push_str(&format!(
                "| {name} | {} | {manifest} |\n",
                blank_as(version, "open")
            ));
        }
    }

    s.push_str("\n## Lockfile status\n\n| Ecosystem | Lockfile | Status |\n|---|---|---|\n");
    let mut any = false;
    for (manifest, lock) in LOCKFILES {
        if !root.join(manifest).is_file() {
            continue;
        }
        any = true;
        s.push_str(&format!(
            "| {manifest} | {lock} | {} |\n",
            if root.join(lock).is_file() {
                "present"
            } else {
                "absent"
            }
        ));
    }
    if !any {
        s.push_str("| none detected | none | absent |\n");
    }

    s.push_str("\n## Unverified\n\n");
    if unverified.is_empty() {
        s.push_str("None.\n");
    } else {
        for (name, manifest) in &unverified {
            s.push_str(&format!("- {name} in {manifest} leaves its version open\n"));
        }
    }
    s
}

/// The `(name, version)` pairs of one manifest, by file name.
fn parse_manifest(name: &str, text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    match name {
        "Cargo.toml" | "pyproject.toml" => {
            let Ok(doc) = text.parse::<toml::Value>() else {
                return out;
            };
            let table = if name == "Cargo.toml" {
                doc.get("dependencies").and_then(|d| d.as_table()).cloned()
            } else {
                doc.get("project")
                    .and_then(|p| p.get("dependencies"))
                    .and_then(|d| d.as_array())
                    .map(|a| {
                        let mut m = toml::map::Map::new();
                        for e in a {
                            if let Some(s) = e.as_str() {
                                let (dep, version) = split_requirement(s);
                                m.insert(dep, toml::Value::String(version));
                            }
                        }
                        m
                    })
            };
            for (dep, value) in table.unwrap_or_default() {
                let version = value
                    .as_str()
                    .map(str::to_string)
                    .or_else(|| {
                        value
                            .get("version")
                            .and_then(|v| v.as_str())
                            .map(str::to_string)
                    })
                    .unwrap_or_default();
                out.push((dep, version));
            }
        }
        "package.json" => {
            let Ok(v) = serde_json::from_str::<serde_json::Value>(text) else {
                return out;
            };
            if let Some(deps) = v.get("dependencies").and_then(|d| d.as_object()) {
                for (dep, version) in deps {
                    out.push((dep.clone(), version.as_str().unwrap_or("").to_string()));
                }
            }
        }
        "requirements.txt" => {
            for line in text.lines() {
                let l = line.trim();
                if l.is_empty() || l.starts_with('#') {
                    continue;
                }
                let (dep, version) = split_requirement(l);
                out.push((dep, version));
            }
        }
        "go.mod" => {
            let mut inside = false;
            for line in text.lines() {
                let l = line.trim();
                if l.starts_with("require (") {
                    inside = true;
                    continue;
                }
                if inside && l == ")" {
                    inside = false;
                    continue;
                }
                let body = if inside {
                    Some(l)
                } else {
                    l.strip_prefix("require ")
                };
                if let Some(b) = body {
                    let mut parts = b.split_whitespace();
                    if let (Some(dep), Some(version)) = (parts.next(), parts.next()) {
                        out.push((dep.to_string(), version.to_string()));
                    }
                }
            }
        }
        "Gemfile" => {
            for line in text.lines() {
                let l = line.trim();
                let Some(rest) = l.strip_prefix("gem ") else {
                    continue;
                };
                let mut parts = rest.split(',').map(|p| p.trim().trim_matches(['"', '\'']));
                let dep = parts.next().unwrap_or("").to_string();
                let version = parts.next().unwrap_or("").to_string();
                if !dep.is_empty() {
                    out.push((dep, version));
                }
            }
        }
        _ => {
            // `pom.xml`, `build.gradle*`, and `*.csproj` are XML or Groovy; the
            // analysis records the manifest and leaves the entries to the skill.
        }
    }
    out.sort();
    out
}

/// `serde==1.0` or `serde>=1` split into a name and a constraint.
fn split_requirement(s: &str) -> (String, String) {
    let at = s.find(['=', '>', '<', '~', '!', '^']);
    match at {
        Some(i) => (s[..i].trim().to_string(), s[i..].trim().to_string()),
        None => (s.trim().to_string(), String::new()),
    }
}

// ------------------------------------------------------------------- stubs

/// `context/coding-standards.md`, a stub.
fn coding_standards(root: &Path, files: &[Seen]) -> String {
    let found: Vec<&str> = STYLE_CONFIGS
        .iter()
        .copied()
        .filter(|name| {
            root.join(name).is_file()
                || files
                    .iter()
                    .any(|f| f.rel.rsplit('/').next() == Some(*name))
        })
        .collect();

    let mut s = String::new();
    s.push_str("\n## Detected configuration\n\n");
    if found.is_empty() {
        s.push_str("None detected.\n");
    } else {
        for f in &found {
            s.push_str(&format!("- {f}\n"));
        }
    }
    for heading in [
        "## Naming",
        "## Formatting",
        "## Error handling",
        "## Testing",
    ] {
        s.push_str(&format!("\n{heading}\n\n{STUB_LINE}\n"));
    }
    s
}

/// `context/architecture-constraints.md`, a stub.
fn architecture_constraints(root: &Path, cfg: &Config) -> String {
    let mut s = String::new();
    s.push_str("\n## Layer map\n\n| Layer | Globs |\n|---|---|\n");
    for (name, globs, _) in crate::coverage::layer_order(cfg) {
        s.push_str(&format!("| {name} | {} |\n", join_or(&globs, "none")));
    }

    s.push_str("\n## Module boundaries\n\n");
    let mut dirs: Vec<String> = Vec::new();
    for st in &cfg.stack {
        for source_root in &st.source_roots {
            let base = root.join(source_root);
            let Ok(entries) = std::fs::read_dir(&base) else {
                continue;
            };
            for e in entries.filter_map(Result::ok) {
                if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                    let rel = crate::project::rel_display(root, &e.path());
                    if !dirs.contains(&rel) {
                        dirs.push(rel);
                    }
                }
            }
        }
    }
    dirs.sort();
    if dirs.is_empty() {
        s.push_str("None detected.\n");
    } else {
        for d in &dirs {
            s.push_str(&format!("- {d}\n"));
        }
    }

    s.push_str(&format!("\n## Constraints\n\n{STUB_LINE}\n"));
    s
}

/// `context/anti-patterns.md`, a stub.
fn anti_patterns(root: &Path, files: &[Seen]) -> String {
    let mut rules: Vec<String> = Vec::new();
    for name in [".eslintrc", ".eslintrc.json", "ruff.toml", ".rubocop.yml"] {
        let path = root.join(name);
        if !path.is_file() {
            continue;
        }
        let text = read_capped(&path);
        for line in text.lines() {
            let l = line.trim();
            if let Some(rest) = l.strip_prefix("\"extends\":") {
                rules.push(format!("{name}: {}", rest.trim().trim_matches([',', ' '])));
            } else if let Some(rest) = l.strip_prefix("select =") {
                rules.push(format!("{name}: {}", rest.trim()));
            } else if let Some(rest) = l.strip_prefix("inherit_gem:") {
                rules.push(format!("{name}: {}", rest.trim()));
            }
        }
        if !rules.iter().any(|r| r.starts_with(name)) {
            rules.push(format!("{name}: present, rule sets unread"));
        }
    }
    let _ = files;

    let mut s = String::new();
    s.push_str("\n## Enabled lint rule sets\n\n");
    if rules.is_empty() {
        s.push_str("None detected.\n");
    } else {
        for r in &rules {
            s.push_str(&format!("- {r}\n"));
        }
    }
    s.push_str(&format!("\n## Patterns\n\n{STUB_LINE}\n"));
    s
}

// ------------------------------------------------------------------ helpers

fn join_or(values: &[String], empty: &str) -> String {
    if values.is_empty() {
        empty.to_string()
    } else {
        values.join(", ")
    }
}

fn blank_as(value: &str, empty: &str) -> String {
    if value.trim().is_empty() {
        empty.to_string()
    } else {
        value.to_string()
    }
}

/// The path of one context file.
pub fn context_path(root: &Path, stem: &str) -> PathBuf {
    root.join(".devforgeai")
        .join("context")
        .join(format!("{stem}.md"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_six_stems_split_three_and_three() {
        assert_eq!(DERIVED.len(), 3);
        assert_eq!(STUBS.len(), 3);
        let all: Vec<&str> = DERIVED.iter().chain(STUBS).copied().collect();
        let mut sorted = all.clone();
        sorted.sort_unstable();
        let mut want = crate::doc::CONTEXT_STEMS.to_vec();
        want.sort_unstable();
        assert_eq!(sorted, want, "the six stems are the six context files");
    }

    #[test]
    fn an_entry_point_is_one_of_the_five_shapes() {
        assert!(is_entry_point("src/main.rs"));
        assert!(is_entry_point("web/index.tsx"));
        assert!(is_entry_point("Program.cs"));
        assert!(is_entry_point("src/app.py"));
        assert!(is_entry_point("cmd/server/main.go"));
        assert!(!is_entry_point("src/order.rs"));
    }

    #[test]
    fn a_requirement_splits_at_its_first_operator() {
        assert_eq!(
            split_requirement("serde==1.0"),
            ("serde".to_string(), "==1.0".to_string())
        );
        assert_eq!(
            split_requirement("flask"),
            ("flask".to_string(), String::new())
        );
    }

    #[test]
    fn a_cargo_manifest_yields_its_direct_dependencies() {
        let text = "[package]\nname = \"x\"\n\n[dependencies]\nserde = \"1\"\nclap = { version = \"4.5\" }\n";
        assert_eq!(
            parse_manifest("Cargo.toml", text),
            vec![
                ("clap".to_string(), "4.5".to_string()),
                ("serde".to_string(), "1".to_string()),
            ]
        );
    }

    #[test]
    fn a_go_mod_yields_its_require_block() {
        let text = "module example\n\ngo 1.22\n\nrequire (\n\tgithub.com/x/y v1.2.3\n)\n";
        assert_eq!(
            parse_manifest("go.mod", text),
            vec![("github.com/x/y".to_string(), "v1.2.3".to_string())]
        );
    }

    #[test]
    fn the_frontmatter_carries_the_seven_keys_in_order() {
        let fm = frontmatter("tech-stack", &[]);
        let keys: Vec<&str> = fm
            .lines()
            .filter_map(|l| l.split_once(':').map(|(k, _)| k))
            .collect();
        assert_eq!(
            keys,
            crate::doc::frontmatter::KEYS.to_vec(),
            "the §5 keys in §5 order"
        );
        assert!(fm.contains("status: draft"));
    }
}
