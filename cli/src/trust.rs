//! The binary trust pin: `~/.devforgeai/trust.toml`, the SHA-256 of the binary
//! a human pinned outside Claude, and the source digest of the framework's
//! `cli/` tree that the pin was taken against.
//!
//! A pin is taken by a human (`trust pin` refuses to run inside a Claude
//! session) and checked by every hook (`trust verify`). Both subcommands are
//! deterministic given the file system and the environment, and `trust verify`
//! writes nothing.

use crate::errors::{CliError, Diag};
use crate::project;
use crate::time;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// The fixed `schema` value `trust.toml` carries.
pub const SCHEMA: &str = "devforgeai/trust/1";

/// The directory name the trust file lives in under the user's home.
const TRUST_DIR: &str = ".devforgeai";

/// The file name inside the trust directory.
const TRUST_FILE: &str = "trust.toml";

/// The two variables whose presence marks a live Claude session.
pub const SESSION_VARS: &[&str] = &["CLAUDECODE", "CLAUDE_CODE_ENTRYPOINT"];

/// `~/.devforgeai/trust.toml`, parsed.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustFile {
    /// Fixed: `devforgeai/trust/1`.
    #[serde(default)]
    pub schema: String,
    /// RFC 3339 UTC instant of the last write.
    #[serde(default)]
    pub updated_at: String,
    /// One entry per pinned binary path, unique by `binary_path`.
    #[serde(default)]
    pub pin: Vec<Pin>,
}

/// One `[[pin]]` table: everything known about one pinned binary.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pin {
    /// The absolute, canonicalised path of the pinned binary.
    #[serde(default)]
    pub binary_path: String,
    /// `sha256:` and 64 lowercase hex: the SHA-256 of the binary's bytes.
    #[serde(default)]
    pub digest: String,
    /// `cli/REVISION` line 1: a git SHA-1, or `unversioned`.
    #[serde(default)]
    pub revision: String,
    /// `cli/REVISION` line 2: the digest of the `cli/` source tree.
    #[serde(default)]
    pub source_digest: String,
    /// `cli/DIGEST`: the digest of the release binary.
    #[serde(default)]
    pub release_digest: String,
    /// The framework repo root, or `""` when unknown.
    #[serde(default)]
    pub framework_path: String,
    /// RFC 3339 UTC instant the pin was taken.
    #[serde(default)]
    pub pinned_at: String,
    /// `<host>\<user>` at pin time.
    #[serde(default)]
    pub pinned_by: String,
}

/// What `verify` established, for the envelope and the human line.
#[derive(Debug, Clone)]
pub struct VerifyReport {
    /// The canonical path of the binary that was verified.
    pub binary: String,
    /// The digest computed from the binary's bytes.
    pub digest: String,
    /// The digest the pin recorded, equal to `digest` on success.
    pub pinned: String,
    /// True when a Claude session variable is set.
    pub session_active: bool,
    /// True when step 5 recomputed the `cli/` source digest.
    pub source_digest_checked: bool,
    /// `DFA-W500` when the pin predates the binary file time.
    pub warnings: Vec<Diag>,
}

// --- home resolution -----------------------------------------------------

/// The trust directory the `test-home` feature substitutes, when it is set.
///
/// This is the crate's only read of the test-home override variable, and it
/// sits inside the feature gate so a released binary cannot be redirected.
/// `tests/trust.rs` scans this file and asserts the name appears exactly once,
/// immediately under the `test-home` `cfg`; do not add a second occurrence.
#[cfg(feature = "test-home")]
fn home_override() -> Option<PathBuf> {
    let raw = std::env::var_os("DEVFORGEAI_HOME")?;
    let p = PathBuf::from(raw);
    if p.as_os_str().is_empty() {
        None
    } else {
        Some(p)
    }
}

/// No override exists with default features: the variable is read by nothing.
#[cfg(not(feature = "test-home"))]
fn home_override() -> Option<PathBuf> {
    None
}

/// The user's home directory: `%USERPROFILE%` on Windows, `$HOME` elsewhere,
/// falling back to the current directory when neither is set.
fn user_home() -> PathBuf {
    #[cfg(windows)]
    let key = "USERPROFILE";
    #[cfg(not(windows))]
    let key = "HOME";
    std::env::var_os(key)
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| PathBuf::from("."))
}

/// `~/.devforgeai`, or the directory the `test-home` override names.
pub fn trust_home() -> PathBuf {
    match home_override() {
        Some(p) => p,
        None => user_home().join(TRUST_DIR),
    }
}

/// `~/.devforgeai/trust.toml`.
pub fn trust_path() -> PathBuf {
    trust_home().join(TRUST_FILE)
}

// --- digests -------------------------------------------------------------

/// Render a finished hash in the `sha256:` and 64 lowercase hex form every
/// digest in the trust file takes.
fn render(h: Sha256) -> String {
    format!("sha256:{:x}", h.finalize())
}

/// The SHA-256 of a file's bytes, in `sha256:<64 lowercase hex>` form.
pub fn digest_file(p: &Path) -> Result<String, CliError> {
    let mut f = std::fs::File::open(p).map_err(|e| CliError::io("reading", p.display(), &e))?;
    let mut h = Sha256::new();
    std::io::copy(&mut f, &mut h).map_err(|e| CliError::io("reading", p.display(), &e))?;
    Ok(render(h))
}

/// The `cli/` directory, whether the caller named it or named the framework
/// root that holds it.
fn cli_root(p: &Path) -> PathBuf {
    if p.file_name().is_some_and(|n| n == "cli") {
        return p.to_path_buf();
    }
    let nested = p.join("cli");
    if nested.is_dir() {
        nested
    } else {
        p.to_path_buf()
    }
}

/// The two release files the source digest never covers.
const EXCLUDED_FILES: &[&str] = &["cli/DIGEST", "cli/REVISION"];

/// The digest of the framework's source tree.
///
/// For every file under `cli/` in byte-wise ascending path order, excluding
/// `cli/DIGEST` and `cli/REVISION` and anything under `target/` or `.git/`, the hash absorbs the
/// path in forward-slash form relative to the framework root (`cli/src/main.rs`
/// and so on), a `0x00` byte, the file length as eight big-endian bytes, then
/// the file's bytes.
///
/// The spec computes this over "every tracked file", which names git. The
/// framework repo is not always a work tree where the CLI runs, so this walks
/// the directory instead and excludes the two paths git would never track.
pub fn source_digest(cli_dir: &Path) -> Result<String, CliError> {
    let root = cli_root(cli_dir);
    if !root.is_dir() {
        return Err(CliError::at(
            "DFA-E510",
            format!("{} is missing; release builds write it", root.display()),
            root.display().to_string(),
        ));
    }

    let mut files: Vec<(String, PathBuf)> = Vec::new();
    let walk = walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_entry(|e| !(e.file_type().is_dir() && is_excluded_at_root(&root, e.path())));
    for entry in walk {
        let entry = entry.map_err(|e| {
            CliError::at(
                "DFA-E900",
                format!("walking {} failed: {e}", root.display()),
                root.display().to_string(),
            )
        })?;
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = match entry.path().strip_prefix(&root) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let name = format!("cli/{}", rel.to_string_lossy().replace('\\', "/"));
        // Neither release file is part of the digest it records: `cli/DIGEST`
        // holds the binary's hash, and `cli/REVISION` line 2 holds this very
        // value, so including either makes the result unreproducible — a file
        // cannot contain its own digest.
        if EXCLUDED_FILES.contains(&name.as_str()) {
            continue;
        }
        files.push((name, entry.path().to_path_buf()));
    }
    files.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

    let mut h = Sha256::new();
    for (name, path) in &files {
        let bytes = std::fs::read(path).map_err(|e| CliError::io("reading", path.display(), &e))?;
        h.update(name.as_bytes());
        h.update([0u8]);
        h.update((bytes.len() as u64).to_be_bytes());
        h.update(&bytes);
    }
    Ok(render(h))
}

/// Build output and git metadata take no part in the source digest.
fn is_excluded_dir(name: &std::ffi::OsStr) -> bool {
    name == "target" || name == ".git"
}

/// True when `dir` is `cli/target` or `cli/.git`, the two directories git
/// would never track, at depth 1 under the `cli/` root.
///
/// Matching the names at any depth instead would silently drop a source
/// directory that happened to be called `target` — a plausible name for a
/// build-target abstraction — from the digest, so a change inside it would not
/// alter the source digest and would not be noticed by `trust verify`.
fn is_excluded_at_root(root: &Path, dir: &Path) -> bool {
    match dir.strip_prefix(root) {
        Ok(rel) => {
            let mut parts = rel.components();
            let first = parts.next();
            parts.next().is_none()
                && first
                    .map(|c| is_excluded_dir(c.as_os_str()))
                    .unwrap_or(false)
        }
        Err(_) => false,
    }
}

// --- the session gate ----------------------------------------------------

/// The name of the session variable that is set, when one is.
///
/// A variable that is present but empty or whitespace only counts as absent,
/// so a shell that exports an empty value does not block `trust pin`.
pub fn session_active() -> Option<String> {
    for name in SESSION_VARS {
        if let Some(v) = std::env::var_os(name) {
            let s = v.to_string_lossy();
            if !s.trim_matches(|c: char| c.is_ascii_whitespace()).is_empty() {
                return Some((*name).to_string());
            }
        }
    }
    None
}

// --- release metadata ----------------------------------------------------

/// `DFA-E510` for a release file that is absent or the wrong shape.
fn e510(path: &Path, what: &str) -> CliError {
    CliError::at(
        "DFA-E510",
        format!("{} is {what}; release builds write it", path.display()),
        path.display().to_string(),
    )
}

/// True for `sha256:` followed by 64 lowercase hex characters.
fn is_digest(s: &str) -> bool {
    match s.strip_prefix("sha256:") {
        Some(hex) => {
            hex.len() == 64
                && hex
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        }
        None => false,
    }
}

/// `cli/REVISION` line 1 for a build made outside a git work tree.
pub const REVISION_UNVERSIONED: &str = "unversioned";

/// True for a 40-character lowercase git SHA-1, or the literal `unversioned`.
fn is_revision(s: &str) -> bool {
    s == REVISION_UNVERSIONED
        || (s.len() == 40
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
}

/// Read a release file, mapping an absent file to `DFA-E510`.
fn read_release(path: &Path) -> Result<String, CliError> {
    match std::fs::read_to_string(path) {
        Ok(s) => Ok(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(e510(path, "missing")),
        Err(e) => Err(CliError::io("reading", path.display(), &e)),
    }
}

/// The lines of a release file with trailing carriage returns and trailing
/// blank lines removed, so a file written on Windows parses.
fn release_lines(text: &str) -> Vec<&str> {
    let mut lines: Vec<&str> = text.lines().map(str::trim).collect();
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    lines
}

/// `<framework>/cli/REVISION`: line 1 the revision, line 2 the source digest.
fn read_revision(framework: &Path) -> Result<(String, String), CliError> {
    let path = framework.join("cli").join("REVISION");
    let text = read_release(&path)?;
    let lines = release_lines(&text);
    if lines.len() != 2 || !is_revision(lines[0]) || !is_digest(lines[1]) {
        return Err(e510(&path, "malformed"));
    }
    Ok((lines[0].to_string(), lines[1].to_string()))
}

/// `<framework>/cli/DIGEST`: the digest of the release binary.
fn read_digest(framework: &Path) -> Result<String, CliError> {
    let path = framework.join("cli").join("DIGEST");
    let text = read_release(&path)?;
    let lines = release_lines(&text);
    if lines.len() != 1 || !is_digest(lines[0]) {
        return Err(e510(&path, "malformed"));
    }
    Ok(lines[0].to_string())
}

// --- path resolution -----------------------------------------------------

/// Canonicalise a path, dropping the Windows verbatim prefix, and raise
/// `DFA-E511` when it cannot be resolved.
fn canonical(p: &Path, what: &str) -> Result<PathBuf, CliError> {
    let c = std::fs::canonicalize(p).map_err(|e| {
        CliError::new(
            "DFA-E511",
            format!("the {what} path is unavailable: {} : {e}", p.display()),
        )
    })?;
    Ok(project::normalise(&c))
}

/// The binary being pinned or verified: the flag when given, else the running
/// executable. A failure to resolve either is `DFA-E511`.
pub fn resolve_binary(binary: Option<&Path>) -> Result<PathBuf, CliError> {
    match binary {
        Some(p) => canonical(p, "binary"),
        None => {
            let exe = std::env::current_exe().map_err(|e| {
                CliError::new(
                    "DFA-E511",
                    format!("the running executable path is unavailable: {e}"),
                )
            })?;
            canonical(&exe, "running executable")
        }
    }
}

/// The framework repo: the flag when given, else the nearest ancestor of the
/// current directory holding a `cli/` with both `REVISION` and `DIGEST`.
fn resolve_framework(framework: Option<&Path>) -> Result<PathBuf, CliError> {
    if let Some(p) = framework {
        if !has_release_files(p) {
            return Err(e510(&p.join("cli"), "missing"));
        }
        return Ok(project::normalise(p));
    }
    let cwd = std::env::current_dir().map_err(|e| {
        CliError::new(
            "DFA-E511",
            format!("the current directory is unavailable: {e}"),
        )
    })?;
    let mut here: &Path = &cwd;
    loop {
        if has_release_files(here) {
            return Ok(project::normalise(here));
        }
        match here.parent() {
            Some(p) => here = p,
            None => {
                return Err(e510(&cwd.join("cli"), "missing"));
            }
        }
    }
}

/// True when `root/cli/` holds both release files.
fn has_release_files(root: &Path) -> bool {
    let cli = root.join("cli");
    cli.is_dir() && cli.join("REVISION").is_file() && cli.join("DIGEST").is_file()
}

/// `<host>\<user>`, with either half empty when the environment does not say.
fn pinned_by() -> String {
    let host = first_var(&["COMPUTERNAME", "HOSTNAME"]);
    let user = first_var(&["USERNAME", "USER"]);
    format!("{host}\\{user}")
}

/// The first of `keys` that is set to a non-empty value, or `""`.
fn first_var(keys: &[&str]) -> String {
    for k in keys {
        if let Some(v) = std::env::var_os(k) {
            let s = v.to_string_lossy().trim().to_string();
            if !s.is_empty() {
                return s;
            }
        }
    }
    String::new()
}

// --- the trust file ------------------------------------------------------

/// Read `~/.devforgeai/trust.toml`, returning `None` when it does not exist.
/// An empty file parses to an empty trust file; anything else unparsable is
/// `DFA-E512`.
pub fn load_trust_file() -> Result<Option<TrustFile>, CliError> {
    let path = trust_path();
    let text = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(CliError::io("reading", path.display(), &e)),
    };
    if text.trim().is_empty() {
        return Ok(Some(TrustFile::default()));
    }
    toml::from_str(&text).map(Some).map_err(|e| {
        CliError::at(
            "DFA-E512",
            format!("trust.toml is not valid TOML: {e}"),
            path.display().to_string(),
        )
    })
}

/// Two binary paths naming the same file. Windows paths compare without case.
fn same_path(a: &str, b: &Path) -> bool {
    let b = b.to_string_lossy();
    if cfg!(windows) {
        a.eq_ignore_ascii_case(&b)
    } else {
        a == b
    }
}

// --- trust pin -----------------------------------------------------------

/// Pin a binary: the spec's eight steps, in order.
///
/// Refuses inside a Claude session (`DFA-E500`), reads the release files
/// (`DFA-E510`), compares the binary's digest with `cli/DIGEST` (`DFA-E503`),
/// then replaces or appends the `[[pin]]` for the canonical binary path and
/// writes `trust.toml` atomically. Nothing is written on any error.
pub fn pin(binary: Option<&Path>, framework: Option<&Path>) -> Result<Pin, CliError> {
    // 1. A pin is a human act, taken outside Claude.
    if let Some(var) = session_active() {
        return Err(CliError::new(
            "DFA-E500",
            format!("trust pin does not run inside a Claude session; {var} is set"),
        ));
    }

    // 2 and 3. The release metadata.
    let framework = resolve_framework(framework)?;
    let (revision, src_digest) = read_revision(&framework)?;
    let release_digest = read_digest(&framework)?;

    // 4 and 5. The binary, and the comparison that makes the pin meaningful.
    let bin = resolve_binary(binary)?;
    let digest = digest_file(&bin)?;
    if digest != release_digest {
        return Err(CliError::at(
            "DFA-E503",
            format!("binary digest {digest} does not match the pin {release_digest}"),
            bin.display().to_string(),
        ));
    }

    // 6. The trust file, created when absent.
    let path = trust_path();
    if !path.exists() {
        let dir = trust_home();
        std::fs::create_dir_all(&dir).map_err(|e| CliError::io("creating", dir.display(), &e))?;
        std::fs::write(&path, b"").map_err(|e| CliError::io("creating", path.display(), &e))?;
    }
    let mut file = load_trust_file()?.unwrap_or_default();

    // 7. Replace the entry for this path, or append one.
    let now = time::now_rfc3339();
    let entry = Pin {
        binary_path: bin.display().to_string(),
        digest,
        revision,
        source_digest: src_digest,
        release_digest,
        framework_path: framework.display().to_string(),
        pinned_at: now.clone(),
        pinned_by: pinned_by(),
    };
    match file
        .pin
        .iter()
        .position(|p| same_path(&p.binary_path, &bin))
    {
        Some(i) => file.pin[i] = entry.clone(),
        None => file.pin.push(entry.clone()),
    }
    file.schema = SCHEMA.to_string();
    file.updated_at = now;

    // 8. One atomic write.
    let rendered = toml::to_string(&file).map_err(|e| {
        CliError::at(
            "DFA-E512",
            format!("trust.toml could not be rendered: {e}"),
            path.display().to_string(),
        )
    })?;
    project::atomic_write(&path, rendered.as_bytes())?;

    Ok(entry)
}

// --- trust verify --------------------------------------------------------

/// Verify a binary against its pin: the spec's seven steps, in order.
///
/// Nothing is written. A pin that predates the binary's file time with matching
/// digests is the warning `DFA-W500` and still a pass.
pub fn verify(binary: Option<&Path>) -> Result<VerifyReport, CliError> {
    // 1. The binary.
    let bin = resolve_binary(binary)?;

    // 2. The trust file.
    let path = trust_path();
    let file = load_trust_file()?.ok_or_else(|| {
        CliError::at(
            "DFA-E501",
            format!(
                "{} not found; a human runs 'devforgeai trust pin' outside Claude",
                path.display()
            ),
            path.display().to_string(),
        )
    })?;

    // 3. The entry for this path.
    let entry = file
        .pin
        .iter()
        .find(|p| same_path(&p.binary_path, &bin))
        .ok_or_else(|| {
            CliError::at(
                "DFA-E502",
                format!("no pin for {} in trust.toml", bin.display()),
                path.display().to_string(),
            )
        })?;

    // 4. The digest of the bytes on disk.
    let digest = digest_file(&bin)?;
    if digest != entry.digest {
        return Err(CliError::at(
            "DFA-E503",
            format!(
                "binary digest {digest} does not match the pin {}",
                entry.digest
            ),
            bin.display().to_string(),
        ));
    }

    // 5 and 6. The source tree, checked only inside a session.
    let session = session_active();
    let mut source_digest_checked = false;
    if session.is_some() && !entry.framework_path.is_empty() {
        let cli = Path::new(&entry.framework_path).join("cli");
        if cli.is_dir() {
            source_digest_checked = true;
            let actual = source_digest(&cli)?;
            if actual != entry.source_digest {
                return Err(CliError::at(
                    "DFA-E504",
                    format!(
                        "cli/ source digest {actual} differs from the pinned REVISION {} while a Claude session is active",
                        entry.source_digest
                    ),
                    cli.display().to_string(),
                ));
            }
        }
    }

    // 7. A pin older than the file it pins, with the digests still agreeing.
    let mut warnings = Vec::new();
    if pin_predates_binary(entry, &bin) {
        warnings.push(Diag::at(
            "DFA-W500",
            "the pin predates the binary file time; digests still match",
            bin.display().to_string(),
        ));
    }

    Ok(VerifyReport {
        binary: bin.display().to_string(),
        digest: digest.clone(),
        pinned: entry.digest.clone(),
        session_active: session.is_some(),
        source_digest_checked,
        warnings,
    })
}

/// True when `pinned_at` precedes the binary's modification time. Both sides
/// are truncated to whole seconds, the precision `pinned_at` is written at, so
/// a pin taken in the same second as the write does not warn.
fn pin_predates_binary(entry: &Pin, bin: &Path) -> bool {
    let Some(pinned) = time::parse_rfc3339(&entry.pinned_at) else {
        return false;
    };
    let Ok(meta) = std::fs::metadata(bin) else {
        return false;
    };
    let Ok(modified) = meta.modified() else {
        return false;
    };
    let modified = ::time::OffsetDateTime::from(modified);
    let truncate = |t: ::time::OffsetDateTime| t.replace_nanosecond(0).unwrap_or(t);
    truncate(pinned) < truncate(modified)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_shape_is_recognised() {
        assert!(is_digest(&format!("sha256:{}", "a".repeat(64))));
        assert!(!is_digest(&format!("sha256:{}", "A".repeat(64))));
        assert!(!is_digest(&format!("sha256:{}", "a".repeat(63))));
        assert!(!is_digest(&"a".repeat(64)));
        assert!(!is_digest("sha256:"));
    }

    #[test]
    fn revision_shape_is_a_sha1_or_the_literal() {
        assert!(is_revision("unversioned"));
        assert!(is_revision(&"0123456789abcdef".repeat(3)[..40]));
        assert!(!is_revision("UNVERSIONED"));
        assert!(!is_revision("a1b2c3"));
    }

    #[test]
    fn release_lines_drop_carriage_returns_and_trailing_blanks() {
        assert_eq!(release_lines("one\r\ntwo\r\n\r\n"), vec!["one", "two"]);
        assert_eq!(release_lines("only\n"), vec!["only"]);
    }

    #[test]
    fn cli_root_accepts_the_directory_or_its_parent() {
        let t = tempfile::tempdir().expect("temp dir");
        let cli = t.path().join("cli");
        std::fs::create_dir_all(&cli).expect("mkdir");
        assert_eq!(cli_root(&cli), cli);
        assert_eq!(cli_root(t.path()), cli);
    }

    #[test]
    fn pinned_by_always_carries_the_separator() {
        assert!(pinned_by().contains('\\'));
    }
}
