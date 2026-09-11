//! Stack detection: the marker table, the package-manager rules, and the
//! default command set each `[[stack]]` table is written from.
//!
//! Markers are searched at the project root and at directory depth 1 and 2
//! below it, excluding the directories the brownfield analysis excludes.
//! Detection order is the table order, and every ecosystem whose markers match
//! produces one `[[stack]]` table.

use crate::config::Stack;
use std::path::Path;

/// Directory names the marker walk never descends into.
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

/// The greatest directory depth below the project root a marker is searched at.
pub const MAX_MARKER_DEPTH: usize = 2;

/// The ecosystem ids, in the spec's table order, which is detection order.
pub const ECOSYSTEMS: &[&str] = &["rust", "node", "python", "go", "dotnet", "jvm", "ruby"];

/// Every `[[stack]]` table the project's markers produce, in detection order.
pub fn detect(root: &Path) -> Vec<Stack> {
    let files = walk(root);
    ECOSYSTEMS
        .iter()
        .filter_map(|id| {
            let markers = markers_for(id, &files);
            if markers.is_empty() {
                None
            } else {
                Some(build(id, root, markers))
            }
        })
        .collect()
}

/// Every file at the project root and at directory depth 1 and 2 below it, as
/// repo-relative forward-slash paths, sorted so the result is the same on every
/// platform: the directory order the operating system hands back never reaches
/// the `markers` array.
pub fn walk(root: &Path) -> Vec<String> {
    let mut out: Vec<String> = walkdir::WalkDir::new(root)
        // A marker at directory depth 2 is a file at walk depth 3.
        .max_depth(MAX_MARKER_DEPTH + 1)
        .into_iter()
        .filter_entry(|e| e.depth() == 0 || !is_excluded(e))
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| crate::project::rel_display(root, e.path()))
        .collect();
    out.sort();
    out
}

/// True when this entry is one of the directories the walk never descends into.
fn is_excluded(e: &walkdir::DirEntry) -> bool {
    e.file_type().is_dir() && EXCLUDED_DIRS.contains(&e.file_name().to_string_lossy().as_ref())
}

/// The repo-relative marker paths of one ecosystem, in path order.
pub fn markers_for(id: &str, files: &[String]) -> Vec<String> {
    files.iter().filter(|f| is_marker(id, f)).cloned().collect()
}

/// True when this repo-relative path is a marker of `id`.
fn is_marker(id: &str, path: &str) -> bool {
    let name = base(path);
    match id {
        "rust" => name == "Cargo.toml",
        "node" => name == "package.json",
        "python" => matches!(name, "pyproject.toml" | "setup.py" | "requirements.txt"),
        "go" => name == "go.mod",
        "dotnet" => is_project_file(name) || has_ext(name, "sln"),
        "jvm" => matches!(name, "pom.xml" | "build.gradle" | "build.gradle.kts"),
        "ruby" => name == "Gemfile",
        _ => false,
    }
}

/// True for the two .NET project file globs, `*.csproj` and `*.fsproj`.
fn is_project_file(name: &str) -> bool {
    has_ext(name, "csproj") || has_ext(name, "fsproj")
}

/// True when `name` carries this extension, matched without regard to case,
/// which is how every platform the CLI runs on treats these globs.
fn has_ext(name: &str, ext: &str) -> bool {
    match name.rsplit_once('.') {
        Some((stem, found)) => !stem.is_empty() && found.eq_ignore_ascii_case(ext),
        None => false,
    }
}

/// The last segment of a repo-relative forward-slash path.
fn base(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// The directory part of a repo-relative forward-slash path, `"."` at the root.
fn parent(path: &str) -> String {
    match path.rsplit_once('/') {
        Some((dir, _)) => dir.to_string(),
        None => ".".to_string(),
    }
}

/// One `[[stack]]` table, at the ecosystem's defaults from the spec's table.
fn build(id: &str, root: &Path, markers: Vec<String>) -> Stack {
    let mut s = Stack {
        id: id.to_string(),
        // Everything this function builds is the detector's own, so a later
        // rerun may replace it. A human's entry is never marked this way and
        // is never overwritten.
        source: crate::config::STACK_DETECTED.to_string(),
        markers,
        timeout_secs: 900,
        ..Stack::default()
    };
    match id {
        "rust" => fill_rust(&mut s),
        "node" => fill_node(&mut s, root),
        "python" => fill_python(&mut s, root),
        "go" => fill_go(&mut s),
        "dotnet" => fill_dotnet(&mut s),
        "jvm" => fill_jvm(&mut s, root),
        "ruby" => fill_ruby(&mut s, root),
        _ => {}
    }
    s
}

fn fill_rust(s: &mut Stack) {
    s.package_manager = "cargo".into();
    s.source_roots = vec!["src".into()];
    s.test_command = "cargo test --all-features".into();
    s.coverage_command =
        "cargo llvm-cov --all-features --lcov --output-path target/devforgeai/lcov.info".into();
    s.coverage_format = "lcov".into();
    s.coverage_paths = vec!["target/devforgeai/lcov.info".into()];
    s.lint_command = "cargo clippy --all-targets --all-features -- -D warnings".into();
}

fn fill_node(s: &mut Stack, root: &Path) {
    let pm = if root.join("bun.lockb").exists() {
        "bun"
    } else if root.join("pnpm-lock.yaml").exists() {
        "pnpm"
    } else if root.join("yarn.lock").exists() {
        "yarn"
    } else {
        "npm"
    };
    s.package_manager = pm.into();
    let scripts = package_scripts(root);
    let cmd = |script: &str| {
        if scripts.iter().any(|n| n == script) {
            format!("{pm} run {script}")
        } else {
            String::new()
        }
    };
    s.test_command = cmd("test");
    s.coverage_command = cmd("coverage");
    s.lint_command = cmd("lint");
    s.coverage_format = "lcov".into();
    s.coverage_paths = vec!["coverage/lcov.info".into()];
    s.source_roots = if root.join("src").is_dir() {
        vec!["src".into()]
    } else {
        vec![".".into()]
    };
}

/// The script names `package.json` declares. An absent or unreadable file, and
/// one that is not an object with a `scripts` table, declare none: detection
/// never fails on a file the project owns.
fn package_scripts(root: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(root.join("package.json")) else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return Vec::new();
    };
    match value.get("scripts").and_then(|s| s.as_object()) {
        Some(map) => map.keys().cloned().collect(),
        None => Vec::new(),
    }
}

fn fill_python(s: &mut Stack, root: &Path) {
    s.package_manager = if root.join("uv.lock").exists() {
        "uv".into()
    } else if root.join("poetry.lock").exists() {
        "poetry".into()
    } else {
        "pip".into()
    };
    s.test_command = "python -m pytest -q".into();
    s.coverage_command =
        "python -m pytest -q --cov --cov-report=xml:.devforgeai/coverage.xml".into();
    s.coverage_format = "cobertura".into();
    s.coverage_paths = vec![".devforgeai/coverage.xml".into()];
    s.lint_command = if has_ruff_config(root) {
        "python -m ruff check .".into()
    } else {
        String::new()
    };
    s.source_roots = if root.join("src").is_dir() {
        vec!["src".into()]
    } else {
        top_level_packages(root)
    };
}

/// True when `ruff.toml`, `.ruff.toml`, or a `[tool.ruff]` table exists.
fn has_ruff_config(root: &Path) -> bool {
    if root.join("ruff.toml").exists() || root.join(".ruff.toml").exists() {
        return true;
    }
    let Ok(text) = std::fs::read_to_string(root.join("pyproject.toml")) else {
        return false;
    };
    let Ok(value) = text.parse::<toml::Value>() else {
        return false;
    };
    value.get("tool").and_then(|t| t.get("ruff")).is_some()
}

/// The directories directly below the root holding an `__init__.py`, sorted.
fn top_level_packages(root: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut out: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir() && e.path().join("__init__.py").is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| !EXCLUDED_DIRS.contains(&n.as_str()))
        .collect();
    out.sort();
    out
}

fn fill_go(s: &mut Stack) {
    s.package_manager = "go".into();
    s.source_roots = vec![".".into()];
    s.test_command = "go test ./...".into();
    s.coverage_command = "go test ./... -coverprofile=.devforgeai/coverage.out".into();
    s.coverage_format = "go-cover".into();
    s.coverage_paths = vec![".devforgeai/coverage.out".into()];
    s.lint_command = "go vet ./...".into();
}

fn fill_dotnet(s: &mut Stack) {
    s.package_manager = "dotnet".into();
    s.test_command = "dotnet test".into();
    s.coverage_command =
        "dotnet test --collect:\"XPlat Code Coverage\" --results-directory .devforgeai/coverage"
            .into();
    s.coverage_format = "cobertura".into();
    s.coverage_paths = vec![".devforgeai/coverage/**/coverage.cobertura.xml".into()];
    s.lint_command = "dotnet format --verify-no-changes".into();
    // The directories holding the project files. A solution file names no
    // sources, so a project whose only marker is a .sln falls back to the root.
    let mut roots: Vec<String> = s
        .markers
        .iter()
        .filter(|m| is_project_file(base(m)))
        .map(|m| parent(m))
        .collect();
    roots.sort();
    roots.dedup();
    if roots.is_empty() {
        roots.push(".".into());
    }
    s.source_roots = roots;
}

fn fill_jvm(s: &mut Stack, root: &Path) {
    let maven = s.markers.iter().any(|m| base(m) == "pom.xml");
    s.package_manager = if maven {
        "maven".into()
    } else {
        "gradle".into()
    };
    s.test_command = if maven {
        "mvn -q -B test"
    } else {
        "./gradlew test"
    }
    .into();
    s.coverage_command = if maven {
        "mvn -q -B jacoco:report"
    } else {
        "./gradlew jacocoTestReport"
    }
    .into();
    s.coverage_format = "jacoco".into();
    s.coverage_paths = vec![if maven {
        "target/site/jacoco/jacoco.xml".into()
    } else {
        "build/reports/jacoco/**/*.xml".into()
    }];
    // The spec fixes no linter for the JVM: `lint_clean` skips with
    // `reason: no_lint_command`.
    s.lint_command = String::new();
    s.source_roots = existing(root, &["src/main/java", "src/main/kotlin"]);
}

fn fill_ruby(s: &mut Stack, root: &Path) {
    s.package_manager = "bundler".into();
    let test = if root.join("spec").is_dir() {
        "bundle exec rspec"
    } else {
        "bundle exec rake test"
    };
    s.test_command = test.into();
    // The coverage run is the test run with COVERAGE=1 in the environment.
    s.coverage_command = test.into();
    s.env.insert("COVERAGE".into(), "1".into());
    s.coverage_format = "lcov".into();
    s.coverage_paths = vec!["coverage/lcov.info".into()];
    s.lint_command = if root.join(".rubocop.yml").exists() {
        "bundle exec rubocop".into()
    } else {
        String::new()
    };
    s.source_roots = existing(root, &["lib", "app"]);
}

/// The candidates that exist as directories below the root, in the order given.
fn existing(root: &Path, candidates: &[&str]) -> Vec<String> {
    candidates
        .iter()
        .filter(|c| root.join(c).is_dir())
        .map(|c| (*c).to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp() -> tempfile::TempDir {
        tempfile::tempdir().expect("temp dir")
    }

    /// Create `root/rel` and every parent, with `body` as its contents.
    fn file(root: &Path, rel: &str, body: &str) {
        let p = root.join(rel);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).expect("mkdir");
        }
        std::fs::write(&p, body).expect("write");
    }

    fn dir(root: &Path, rel: &str) {
        std::fs::create_dir_all(root.join(rel)).expect("mkdir");
    }

    fn only(root: &Path, id: &str) -> Stack {
        detect(root)
            .into_iter()
            .find(|s| s.id == id)
            .unwrap_or_else(|| panic!("a {id} stack is detected"))
    }

    #[test]
    fn markers_rust() {
        let t = temp();
        file(t.path(), "Cargo.toml", "[package]\nname = \"x\"\n");
        dir(t.path(), "src");
        let s = only(t.path(), "rust");
        assert_eq!(s.markers, vec!["Cargo.toml"]);
        assert_eq!(s.package_manager, "cargo");
        assert_eq!(s.source_roots, vec!["src"]);
        assert_eq!(s.test_command, "cargo test --all-features");
        assert_eq!(
            s.coverage_command,
            "cargo llvm-cov --all-features --lcov --output-path target/devforgeai/lcov.info"
        );
        assert_eq!(s.coverage_format, "lcov");
        assert_eq!(s.coverage_paths, vec!["target/devforgeai/lcov.info"]);
        assert_eq!(
            s.lint_command,
            "cargo clippy --all-targets --all-features -- -D warnings"
        );
        assert_eq!(s.complexity_command, "");
        assert_eq!(s.timeout_secs, 900);
        assert!(s.env.is_empty());
    }

    #[test]
    fn markers_node_with_pnpm_lock() {
        let t = temp();
        file(
            t.path(),
            "package.json",
            r#"{"scripts":{"test":"vitest","coverage":"vitest --coverage","lint":"eslint ."}}"#,
        );
        file(t.path(), "pnpm-lock.yaml", "lockfileVersion: 9\n");
        dir(t.path(), "src");
        let s = only(t.path(), "node");
        assert_eq!(s.markers, vec!["package.json"]);
        assert_eq!(s.package_manager, "pnpm");
        assert_eq!(s.source_roots, vec!["src"]);
        assert_eq!(s.test_command, "pnpm run test");
        assert_eq!(s.coverage_command, "pnpm run coverage");
        assert_eq!(s.coverage_format, "lcov");
        assert_eq!(s.coverage_paths, vec!["coverage/lcov.info"]);
        assert_eq!(s.lint_command, "pnpm run lint");
    }

    #[test]
    fn markers_python_with_uv_lock() {
        let t = temp();
        file(
            t.path(),
            "pyproject.toml",
            "[tool.ruff]\nline-length = 100\n",
        );
        file(t.path(), "uv.lock", "version = 1\n");
        file(t.path(), "pkg/__init__.py", "");
        let s = only(t.path(), "python");
        assert_eq!(s.markers, vec!["pyproject.toml"]);
        assert_eq!(s.package_manager, "uv");
        assert_eq!(s.source_roots, vec!["pkg"], "no src/, so the packages");
        assert_eq!(s.test_command, "python -m pytest -q");
        assert_eq!(
            s.coverage_command,
            "python -m pytest -q --cov --cov-report=xml:.devforgeai/coverage.xml"
        );
        assert_eq!(s.coverage_format, "cobertura");
        assert_eq!(s.coverage_paths, vec![".devforgeai/coverage.xml"]);
        assert_eq!(
            s.lint_command, "python -m ruff check .",
            "[tool.ruff] exists"
        );
    }

    #[test]
    fn markers_go() {
        let t = temp();
        file(t.path(), "go.mod", "module example.com/x\n");
        let s = only(t.path(), "go");
        assert_eq!(s.markers, vec!["go.mod"]);
        assert_eq!(s.package_manager, "go");
        assert_eq!(s.source_roots, vec!["."]);
        assert_eq!(s.test_command, "go test ./...");
        assert_eq!(
            s.coverage_command,
            "go test ./... -coverprofile=.devforgeai/coverage.out"
        );
        assert_eq!(s.coverage_format, "go-cover");
        assert_eq!(s.coverage_paths, vec![".devforgeai/coverage.out"]);
        assert_eq!(s.lint_command, "go vet ./...");
    }

    #[test]
    fn markers_dotnet_glob() {
        let t = temp();
        file(t.path(), "App.sln", "");
        file(t.path(), "src/App/App.csproj", "<Project/>");
        file(t.path(), "src/Lib/Lib.fsproj", "<Project/>");
        let s = only(t.path(), "dotnet");
        assert_eq!(
            s.markers,
            vec!["App.sln", "src/App/App.csproj", "src/Lib/Lib.fsproj"],
            "the three globs match, in path order"
        );
        assert_eq!(s.package_manager, "dotnet");
        assert_eq!(
            s.source_roots,
            vec!["src/App", "src/Lib"],
            "the directories holding the project files"
        );
        assert_eq!(s.test_command, "dotnet test");
        assert_eq!(
            s.coverage_command,
            "dotnet test --collect:\"XPlat Code Coverage\" --results-directory .devforgeai/coverage"
        );
        assert_eq!(s.coverage_format, "cobertura");
        assert_eq!(
            s.coverage_paths,
            vec![".devforgeai/coverage/**/coverage.cobertura.xml"]
        );
        assert_eq!(s.lint_command, "dotnet format --verify-no-changes");
    }

    #[test]
    fn markers_jvm_maven() {
        let t = temp();
        file(t.path(), "pom.xml", "<project/>");
        file(t.path(), "build.gradle", "");
        dir(t.path(), "src/main/java");
        let s = only(t.path(), "jvm");
        assert_eq!(s.markers, vec!["build.gradle", "pom.xml"]);
        assert_eq!(s.package_manager, "maven", "pom.xml wins over gradle");
        assert_eq!(s.test_command, "mvn -q -B test");
        assert_eq!(s.coverage_command, "mvn -q -B jacoco:report");
        assert_eq!(s.coverage_format, "jacoco");
        assert_eq!(s.coverage_paths, vec!["target/site/jacoco/jacoco.xml"]);
        assert_eq!(
            s.source_roots,
            vec!["src/main/java"],
            "src/main/kotlin does not exist"
        );
    }

    #[test]
    fn markers_jvm_gradle_kts() {
        let t = temp();
        file(t.path(), "build.gradle.kts", "plugins { java }\n");
        dir(t.path(), "src/main/kotlin");
        let s = only(t.path(), "jvm");
        assert_eq!(s.markers, vec!["build.gradle.kts"]);
        assert_eq!(s.package_manager, "gradle");
        assert_eq!(s.test_command, "./gradlew test");
        assert_eq!(s.coverage_command, "./gradlew jacocoTestReport");
        assert_eq!(s.coverage_format, "jacoco");
        assert_eq!(s.coverage_paths, vec!["build/reports/jacoco/**/*.xml"]);
        assert_eq!(s.source_roots, vec!["src/main/kotlin"]);
    }

    #[test]
    fn markers_ruby_with_spec_dir() {
        let t = temp();
        file(t.path(), "Gemfile", "source 'https://rubygems.org'\n");
        file(t.path(), ".rubocop.yml", "AllCops:\n");
        dir(t.path(), "spec");
        dir(t.path(), "lib");
        let s = only(t.path(), "ruby");
        assert_eq!(s.markers, vec!["Gemfile"]);
        assert_eq!(s.package_manager, "bundler");
        assert_eq!(s.test_command, "bundle exec rspec", "spec/ exists");
        assert_eq!(
            s.coverage_command, "bundle exec rspec",
            "the test command, with COVERAGE=1 in env"
        );
        assert_eq!(s.env.get("COVERAGE").map(String::as_str), Some("1"));
        assert_eq!(s.coverage_format, "lcov");
        assert_eq!(s.coverage_paths, vec!["coverage/lcov.info"]);
        assert_eq!(s.lint_command, "bundle exec rubocop");
        assert_eq!(s.source_roots, vec!["lib"], "app/ does not exist");
    }

    #[test]
    fn node_without_test_script_gives_empty_command() {
        let t = temp();
        file(t.path(), "package.json", r#"{"scripts":{"build":"tsc"}}"#);
        let s = only(t.path(), "node");
        assert_eq!(s.package_manager, "npm", "no lockfile, so npm");
        assert_eq!(s.test_command, "");
        assert_eq!(s.coverage_command, "");
        assert_eq!(s.lint_command, "");
        assert_eq!(s.source_roots, vec!["."], "no src/");
        assert_eq!(
            s.coverage_format, "lcov",
            "the format is fixed by the table, not by the command"
        );
    }

    #[test]
    fn ruby_without_rubocop_gives_empty_lint() {
        let t = temp();
        file(t.path(), "Gemfile", "source 'https://rubygems.org'\n");
        let s = only(t.path(), "ruby");
        assert_eq!(s.lint_command, "");
        assert_eq!(s.test_command, "bundle exec rake test", "no spec/");
        assert_eq!(s.coverage_command, "bundle exec rake test");
        assert_eq!(s.env.get("COVERAGE").map(String::as_str), Some("1"));
        assert!(s.source_roots.is_empty(), "neither lib/ nor app/ exists");
    }

    #[test]
    fn jvm_lint_is_empty() {
        let t = temp();
        file(t.path(), "pom.xml", "<project/>");
        let maven = only(t.path(), "jvm");
        assert_eq!(maven.lint_command, "");

        let g = temp();
        file(g.path(), "build.gradle", "");
        let gradle = only(g.path(), "jvm");
        assert_eq!(gradle.lint_command, "");
    }

    #[test]
    fn detection_order_is_table_order() {
        let t = temp();
        file(t.path(), "Gemfile", "");
        file(t.path(), "pom.xml", "<project/>");
        file(t.path(), "App.csproj", "<Project/>");
        file(t.path(), "go.mod", "module x\n");
        file(t.path(), "requirements.txt", "pytest\n");
        file(t.path(), "package.json", "{}");
        file(t.path(), "Cargo.toml", "[package]\n");
        let ids: Vec<String> = detect(t.path()).into_iter().map(|s| s.id).collect();
        assert_eq!(
            ids,
            vec!["rust", "node", "python", "go", "dotnet", "jvm", "ruby"],
            "the table order, whatever order the files were created in"
        );
    }

    #[test]
    fn depth_two_marker_found() {
        let t = temp();
        file(t.path(), "services/api/go.mod", "module example.com/api\n");
        let s = only(t.path(), "go");
        assert_eq!(s.markers, vec!["services/api/go.mod"]);

        // Depth three is past the limit.
        let d = temp();
        file(d.path(), "a/b/c/go.mod", "module example.com/deep\n");
        assert!(
            detect(d.path()).is_empty(),
            "a marker at directory depth 3 is not searched"
        );
    }

    #[test]
    fn ignored_directory_marker_skipped() {
        let t = temp();
        file(t.path(), "node_modules/left-pad/package.json", "{}");
        file(t.path(), "target/debug/Cargo.toml", "[package]\n");
        file(t.path(), "vendor/x/go.mod", "module x\n");
        file(t.path(), ".venv/lib/pyproject.toml", "");
        assert!(
            detect(t.path()).is_empty(),
            "every marker lies under an excluded directory"
        );
    }
}
