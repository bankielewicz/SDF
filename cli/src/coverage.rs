//! The five coverage formats the `coverage_min` check reads, and the layer
//! assignment that turns per-file counts into per-layer percentages.
//!
//! Every parser produces the same shape: one `(path, covered, instrumented)`
//! triple per file, with the path project-relative and forward-slashed.

use crate::config::Config;
use crate::errors::CliError;
use std::path::Path;

/// The `coverage_format` enum, `none` excluded: a stack with `none` runs no
/// coverage at all.
pub const FORMATS: &[&str] = &["lcov", "cobertura", "jacoco", "go-cover", "devforgeai-json"];

/// One file's line counts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileCoverage {
    /// Project-relative, forward slashes.
    pub path: String,
    /// Lines executed at least once.
    pub covered: u64,
    /// Lines instrumented.
    pub total: u64,
}

/// One layer's roll-up.
#[derive(Debug, Clone, PartialEq)]
pub struct LayerCoverage {
    /// The layer name, or `unassigned`.
    pub name: String,
    /// Covered lines.
    pub covered: u64,
    /// Instrumented lines.
    pub total: u64,
    /// `covered * 100 / total`, one decimal, half away from zero.
    pub percent: f64,
    /// The threshold from `config.toml`.
    pub min: f64,
    /// `pass` or `fail`.
    pub status: &'static str,
}

/// A percentage, one decimal, half away from zero. A file set with no
/// instrumented line scores 100.0.
pub fn percent(covered: u64, total: u64) -> f64 {
    if total == 0 {
        return 100.0;
    }
    let raw = covered as f64 * 100.0 / total as f64;
    (raw * 10.0).round() / 10.0
}

/// Parse one artifact by format name.
pub fn parse(format: &str, path: &Path, text: &str) -> Result<Vec<FileCoverage>, CliError> {
    let bad = |message: String| {
        CliError::at(
            "DFA-E312",
            format!(
                "coverage report {} is not {format}: {message}",
                path.display()
            ),
            path.display().to_string(),
        )
    };
    match format {
        "lcov" => parse_lcov(text).map_err(bad),
        "cobertura" => parse_cobertura(text).map_err(bad),
        "jacoco" => parse_jacoco(text).map_err(bad),
        "go-cover" => parse_go_cover(text).map_err(bad),
        "devforgeai-json" => parse_devforgeai_json(text).map_err(bad),
        other => Err(CliError::at(
            "DFA-E312",
            format!(
                "coverage report {} has format '{other}'; the formats are {}",
                path.display(),
                FORMATS.join(", ")
            ),
            path.display().to_string(),
        )),
    }
}

/// Normalise a path to project-relative forward-slash form.
pub fn normalise(root: &Path, raw: &str) -> String {
    let cleaned = raw.trim().replace('\\', "/");
    let as_path = Path::new(&cleaned);
    let stripped = as_path
        .strip_prefix(crate::project::normalise(root).as_path())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or(cleaned);
    stripped
        .trim_start_matches("./")
        .replace('\\', "/")
        .to_string()
}

// ---------------------------------------------------------------------- lcov

/// `SF:<path>` opens a record, `DA:<line>,<hits>` is one line, `end_of_record`
/// closes it. `LF`/`LH` are read when no `DA` record appeared.
fn parse_lcov(text: &str) -> Result<Vec<FileCoverage>, String> {
    let mut out: Vec<FileCoverage> = Vec::new();
    let mut path = String::new();
    let mut covered = 0u64;
    let mut total = 0u64;
    let mut saw_da = false;
    let mut lf = 0u64;
    let mut lh = 0u64;
    let mut open = false;

    for line in text.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("SF:") {
            path = rest.trim().to_string();
            covered = 0;
            total = 0;
            saw_da = false;
            lf = 0;
            lh = 0;
            open = true;
        } else if let Some(rest) = t.strip_prefix("DA:") {
            let mut parts = rest.split(',');
            let _line = parts.next();
            let hits: u64 = parts.next().unwrap_or("0").trim().parse().unwrap_or(0);
            total += 1;
            if hits > 0 {
                covered += 1;
            }
            saw_da = true;
        } else if let Some(rest) = t.strip_prefix("LF:") {
            lf = rest.trim().parse().unwrap_or(0);
        } else if let Some(rest) = t.strip_prefix("LH:") {
            lh = rest.trim().parse().unwrap_or(0);
        } else if t == "end_of_record" && open {
            out.push(FileCoverage {
                path: std::mem::take(&mut path),
                covered: if saw_da { covered } else { lh },
                total: if saw_da { total } else { lf },
            });
            open = false;
        }
    }
    if open && !path.is_empty() {
        out.push(FileCoverage {
            path,
            covered: if saw_da { covered } else { lh },
            total: if saw_da { total } else { lf },
        });
    }
    if out.is_empty() {
        return Err("no SF record".to_string());
    }
    Ok(out)
}

// ----------------------------------------------------------------- cobertura

/// `<sources><source>` prefixes a `<class filename=...>`, whose `<line>`
/// elements carry `hits`.
fn parse_cobertura(text: &str) -> Result<Vec<FileCoverage>, String> {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_str(text);
    reader.config_mut().trim_text(true);

    let mut out: Vec<FileCoverage> = Vec::new();
    let mut source_prefix = String::new();
    let mut in_sources = false;
    let mut current: Option<FileCoverage> = None;
    let mut saw_coverage = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => return Err(e.to_string()),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "coverage" => saw_coverage = true,
                    "sources" => in_sources = true,
                    "class" => {
                        if let Some(done) = current.take() {
                            out.push(done);
                        }
                        let filename = attribute(&e, "filename").unwrap_or_default();
                        let path = if source_prefix.is_empty() {
                            filename
                        } else {
                            format!("{}/{filename}", source_prefix.trim_end_matches('/'))
                        };
                        current = Some(FileCoverage {
                            path,
                            covered: 0,
                            total: 0,
                        });
                    }
                    "line" => {
                        if let Some(c) = current.as_mut() {
                            let hits: u64 = attribute(&e, "hits")
                                .and_then(|h| h.parse().ok())
                                .unwrap_or(0);
                            c.total += 1;
                            if hits > 0 {
                                c.covered += 1;
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(t)) => {
                if in_sources && source_prefix.is_empty() {
                    source_prefix = t.unescape().unwrap_or_default().trim().to_string();
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "sources" {
                    in_sources = false;
                }
                if name == "class" {
                    out.extend(current.take());
                }
            }
            _ => {}
        }
        buf.clear();
    }
    if let Some(done) = current.take() {
        out.push(done);
    }
    if !saw_coverage {
        return Err("no <coverage> element".to_string());
    }
    Ok(out)
}

/// One attribute of a start tag.
fn attribute(e: &quick_xml::events::BytesStart, name: &str) -> Option<String> {
    e.attributes().filter_map(Result::ok).find_map(|a| {
        if a.key.as_ref() == name.as_bytes() {
            Some(String::from_utf8_lossy(&a.value).to_string())
        } else {
            None
        }
    })
}

// -------------------------------------------------------------------- jacoco

/// `<package name=...>` and `<sourcefile name=...>` join to the path, and the
/// `LINE` counter of each sourcefile carries `covered` and `missed`.
fn parse_jacoco(text: &str) -> Result<Vec<FileCoverage>, String> {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_str(text);
    reader.config_mut().trim_text(true);

    let mut out: Vec<FileCoverage> = Vec::new();
    let mut package = String::new();
    let mut current: Option<FileCoverage> = None;
    let mut saw_report = false;
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => return Err(e.to_string()),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                match name.as_str() {
                    "report" => saw_report = true,
                    "package" => {
                        package = attribute(&e, "name").unwrap_or_default().replace('\\', "/")
                    }
                    "sourcefile" => {
                        if let Some(done) = current.take() {
                            out.push(done);
                        }
                        let file = attribute(&e, "name").unwrap_or_default();
                        let path = if package.is_empty() {
                            file
                        } else {
                            format!("{package}/{file}")
                        };
                        current = Some(FileCoverage {
                            path,
                            covered: 0,
                            total: 0,
                        });
                    }
                    "counter" => {
                        let line_counter = attribute(&e, "type").as_deref() == Some("LINE");
                        if let (true, Some(c)) = (line_counter, current.as_mut()) {
                            let covered: u64 = attribute(&e, "covered")
                                .and_then(|v| v.parse().ok())
                                .unwrap_or(0);
                            let missed: u64 = attribute(&e, "missed")
                                .and_then(|v| v.parse().ok())
                                .unwrap_or(0);
                            c.covered = covered;
                            c.total = covered + missed;
                        }
                    }
                    _ => {}
                }
            }
            // `take` only for this one end tag: a `</counter>` must not drop
            // the sourcefile that is still open.
            Ok(Event::End(e)) if e.name().as_ref() == b"sourcefile" => {
                out.extend(current.take());
            }
            _ => {}
        }
        buf.clear();
    }
    if let Some(done) = current.take() {
        out.push(done);
    }
    if !saw_report {
        return Err("no <report> element".to_string());
    }
    Ok(out)
}

// ------------------------------------------------------------------ go-cover

/// `mode: <set|count|atomic>` then `path:startLine.col,endLine.col n count`.
fn parse_go_cover(text: &str) -> Result<Vec<FileCoverage>, String> {
    let mut lines = text.lines().map(str::trim).filter(|l| !l.is_empty());
    let Some(first) = lines.next() else {
        return Err("empty file".to_string());
    };
    if !first.starts_with("mode:") {
        return Err("line 1 is not a mode line".to_string());
    }

    let mut by_path: std::collections::BTreeMap<String, (u64, u64)> =
        std::collections::BTreeMap::new();
    for line in lines {
        // `path:1.2,3.4 5 6`
        let Some((location, rest)) = line.rsplit_once(' ') else {
            continue;
        };
        let count: u64 = rest.trim().parse().unwrap_or(0);
        let Some((span, statements)) = location.rsplit_once(' ') else {
            continue;
        };
        let statements: u64 = statements.trim().parse().unwrap_or(0);
        let Some((path, _range)) = span.rsplit_once(':') else {
            continue;
        };
        let entry = by_path.entry(path.trim().to_string()).or_insert((0, 0));
        entry.1 += statements;
        if count > 0 {
            entry.0 += statements;
        }
    }
    if by_path.is_empty() {
        return Err("no coverage block".to_string());
    }
    Ok(by_path
        .into_iter()
        .map(|(path, (covered, total))| FileCoverage {
            path,
            covered,
            total,
        })
        .collect())
}

// ----------------------------------------------------------- devforgeai-json

/// `{ "schema": "devforgeai/coverage/1", "files": [ { path, covered, total } ] }`
fn parse_devforgeai_json(text: &str) -> Result<Vec<FileCoverage>, String> {
    let v: serde_json::Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if v.get("schema").and_then(|s| s.as_str()) != Some("devforgeai/coverage/1") {
        return Err("schema is not devforgeai/coverage/1".to_string());
    }
    let files = v
        .get("files")
        .and_then(|f| f.as_array())
        .ok_or_else(|| "files is not an array".to_string())?;
    Ok(files
        .iter()
        .filter_map(|f| {
            Some(FileCoverage {
                path: f.get("path")?.as_str()?.to_string(),
                covered: f.get("covered")?.as_u64()?,
                total: f.get("total")?.as_u64()?,
            })
        })
        .collect())
}

// ------------------------------------------------------------ layer roll-up

/// The layer set: every `[[layer]]` in `config.toml` order, filled in with the
/// compiled defaults for a layer the project deleted.
pub fn layer_order(cfg: &Config) -> Vec<(String, Vec<String>, f64)> {
    let mut out: Vec<(String, Vec<String>, f64)> = cfg
        .layer
        .iter()
        .map(|l| (l.name.clone(), l.globs.clone(), l.coverage_min))
        .collect();
    for d in crate::config::default_layers() {
        if !out.iter().any(|(n, _, _)| n == &d.name) {
            out.push((d.name, d.globs, d.coverage_min));
        }
    }
    out
}

/// The result of rolling per-file counts up into layers.
#[derive(Debug, Clone)]
pub struct RollUp {
    /// One entry per layer, in `config.toml` order.
    pub layers: Vec<LayerCoverage>,
    /// Files matching no layer glob.
    pub unassigned_files: usize,
    /// The percentage of the unassigned set.
    pub unassigned_percent: f64,
    /// The figure over every non-excluded file.
    pub overall: f64,
    /// Files dropped by `[coverage].exclude`.
    pub excluded: usize,
}

/// Drop excluded files, assign the rest to layers, and compute the figures.
pub fn roll_up(cfg: &Config, files: &[FileCoverage]) -> RollUp {
    let exclude = globs(&cfg.coverage.exclude);
    let kept: Vec<&FileCoverage> = files
        .iter()
        .filter(|f| !exclude.is_match(&f.path))
        .collect();
    let excluded = files.len() - kept.len();

    let order = layer_order(cfg);
    let matchers: Vec<globset::GlobSet> = order.iter().map(|(_, g, _)| globs(g)).collect();

    let mut sums: Vec<(u64, u64)> = vec![(0, 0); order.len()];
    let mut unassigned: (u64, u64, usize) = (0, 0, 0);

    for f in &kept {
        match matchers.iter().position(|m| m.is_match(&f.path)) {
            Some(i) => {
                sums[i].0 += f.covered;
                sums[i].1 += f.total;
            }
            None => {
                unassigned.0 += f.covered;
                unassigned.1 += f.total;
                unassigned.2 += 1;
            }
        }
    }

    let layers = order
        .iter()
        .zip(&sums)
        .map(|((name, _, min), (covered, total))| {
            let p = percent(*covered, *total);
            LayerCoverage {
                name: name.clone(),
                covered: *covered,
                total: *total,
                percent: p,
                min: *min,
                status: if p + f64::EPSILON >= *min {
                    "pass"
                } else {
                    "fail"
                },
            }
        })
        .collect();

    let all_covered: u64 = kept.iter().map(|f| f.covered).sum();
    let all_total: u64 = kept.iter().map(|f| f.total).sum();

    RollUp {
        layers,
        unassigned_files: unassigned.2,
        unassigned_percent: percent(unassigned.0, unassigned.1),
        overall: percent(all_covered, all_total),
        excluded,
    }
}

fn globs(patterns: &[String]) -> globset::GlobSet {
    let mut b = globset::GlobSetBuilder::new();
    for p in patterns {
        if let Ok(g) = globset::Glob::new(p) {
            b.add(g);
        }
    }
    b.build().unwrap_or_else(|_| globset::GlobSet::empty())
}

/// The most recently modified match of the globs, `None` when none matched.
pub fn newest_match(root: &Path, patterns: &[String]) -> Option<std::path::PathBuf> {
    let set = globs(patterns);
    let mut best: Option<(std::time::SystemTime, std::path::PathBuf)> = None;
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| e.file_name() != ".git")
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let rel = crate::project::rel_display(root, entry.path());
        if !set.is_match(&rel) {
            continue;
        }
        let when = entry
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        if best.as_ref().map(|(t, _)| when >= *t).unwrap_or(true) {
            best = Some((when, entry.path().to_path_buf()));
        }
    }
    best.map(|(_, p)| p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_percentage_rounds_half_away_from_zero_at_one_decimal() {
        assert_eq!(percent(1, 3), 33.3);
        assert_eq!(percent(2, 3), 66.7);
        assert_eq!(percent(881, 1000), 88.1);
        assert_eq!(percent(1, 8), 12.5);
    }

    #[test]
    fn a_file_set_with_no_instrumented_line_scores_a_hundred() {
        assert_eq!(percent(0, 0), 100.0);
    }

    #[test]
    fn lcov_counts_da_records() {
        let text = "TN:\nSF:src/domain/order.rs\nDA:1,1\nDA:2,0\nDA:3,4\nend_of_record\nSF:src/api/http.rs\nDA:1,0\nend_of_record\n";
        let f = parse_lcov(text).expect("parses");
        assert_eq!(f.len(), 2);
        assert_eq!(f[0].path, "src/domain/order.rs");
        assert_eq!((f[0].covered, f[0].total), (2, 3));
        assert_eq!((f[1].covered, f[1].total), (0, 1));
    }

    #[test]
    fn lcov_falls_back_to_lf_and_lh() {
        let text = "SF:src/a.rs\nLF:10\nLH:7\nend_of_record\n";
        let f = parse_lcov(text).expect("parses");
        assert_eq!((f[0].covered, f[0].total), (7, 10));
    }

    #[test]
    fn lcov_with_no_record_is_an_error() {
        assert!(parse_lcov("nothing\n").is_err());
    }

    #[test]
    fn cobertura_joins_the_sources_prefix() {
        let text = r#"<?xml version="1.0"?>
<coverage>
  <sources><source>/build/repo</source></sources>
  <packages><package name="p"><classes>
    <class filename="src/domain/order.py">
      <lines><line number="1" hits="1"/><line number="2" hits="0"/></lines>
    </class>
  </classes></package></packages>
</coverage>"#;
        let f = parse_cobertura(text).expect("parses");
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].path, "/build/repo/src/domain/order.py");
        assert_eq!((f[0].covered, f[0].total), (1, 2));
    }

    #[test]
    fn cobertura_without_a_coverage_element_is_an_error() {
        assert!(parse_cobertura("<other/>").is_err());
    }

    #[test]
    fn jacoco_joins_the_package_and_sourcefile() {
        let text = r#"<?xml version="1.0"?>
<report name="r">
  <package name="com/example/domain">
    <sourcefile name="Order.java">
      <counter type="INSTRUCTION" missed="3" covered="9"/>
      <counter type="LINE" missed="2" covered="8"/>
    </sourcefile>
  </package>
</report>"#;
        let f = parse_jacoco(text).expect("parses");
        assert_eq!(f[0].path, "com/example/domain/Order.java");
        assert_eq!((f[0].covered, f[0].total), (8, 10));
    }

    #[test]
    fn go_cover_sums_statements_per_file() {
        let text = "mode: set\nexample/a/order.go:3.20,5.2 2 1\nexample/a/order.go:7.20,9.2 3 0\n";
        let f = parse_go_cover(text).expect("parses");
        assert_eq!(f.len(), 1);
        assert_eq!((f[0].covered, f[0].total), (2, 5));
    }

    #[test]
    fn go_cover_without_a_mode_line_is_an_error() {
        assert!(parse_go_cover("example/a.go:1.1,2.2 1 1\n").is_err());
    }

    #[test]
    fn the_json_fallback_reads_its_own_schema() {
        let text = r#"{ "schema": "devforgeai/coverage/1",
            "files": [ { "path": "src/domain/order.rs", "covered": 120, "total": 130 } ] }"#;
        let f = parse_devforgeai_json(text).expect("parses");
        assert_eq!(f[0].path, "src/domain/order.rs");
        assert_eq!((f[0].covered, f[0].total), (120, 130));
    }

    #[test]
    fn the_json_fallback_refuses_another_schema() {
        assert!(parse_devforgeai_json(r#"{"schema":"other","files":[]}"#).is_err());
    }

    fn config() -> Config {
        Config {
            layer: crate::config::default_layers(),
            ..Default::default()
        }
    }

    #[test]
    fn a_file_takes_the_first_layer_whose_globs_match() {
        let files = vec![
            FileCoverage {
                path: "src/domain/order.rs".into(),
                covered: 9,
                total: 10,
            },
            FileCoverage {
                path: "src/api/http.rs".into(),
                covered: 5,
                total: 10,
            },
            FileCoverage {
                path: "src/util.rs".into(),
                covered: 1,
                total: 10,
            },
        ];
        let r = roll_up(&config(), &files);
        let domain = r
            .layers
            .iter()
            .find(|l| l.name == "domain")
            .expect("domain");
        assert_eq!(domain.percent, 90.0);
        assert_eq!(domain.status, "fail", "below the 95.0 default");
        let interface = r
            .layers
            .iter()
            .find(|l| l.name == "interface")
            .expect("interface");
        assert_eq!(interface.percent, 50.0);
        assert_eq!(r.unassigned_files, 1);
        assert_eq!(r.overall, 50.0, "15 of 30 lines");
    }

    #[test]
    fn an_excluded_file_is_dropped_before_anything_else() {
        let mut cfg = config();
        cfg.coverage.exclude = vec!["**/tests/**".to_string()];
        let files = vec![
            FileCoverage {
                path: "src/domain/order.rs".into(),
                covered: 10,
                total: 10,
            },
            FileCoverage {
                path: "src/tests/helper.rs".into(),
                covered: 0,
                total: 10,
            },
        ];
        let r = roll_up(&cfg, &files);
        assert_eq!(r.excluded, 1);
        assert_eq!(r.overall, 100.0);
    }

    #[test]
    fn a_deleted_layer_keeps_its_compiled_default() {
        let mut cfg = config();
        cfg.layer.retain(|l| l.name != "domain");
        let r = roll_up(&cfg, &[]);
        let domain = r
            .layers
            .iter()
            .find(|l| l.name == "domain")
            .expect("domain");
        assert_eq!(domain.min, 95.0, "deleting a layer lowers nothing");
        assert_eq!(domain.total, 0);
        assert_eq!(domain.percent, 100.0);
    }

    #[test]
    fn a_path_is_normalised_against_the_project_root() {
        let root = Path::new("/tmp/project");
        assert_eq!(normalise(root, "/tmp/project/src/a.rs"), "src/a.rs");
        assert_eq!(normalise(root, "./src/a.rs"), "src/a.rs");
        assert_eq!(normalise(root, r"src\a.rs"), "src/a.rs");
    }
}
