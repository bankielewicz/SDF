//! The `trust pin` and `trust verify` contract, driven through the library.
//!
//! `CLAUDECODE`, `CLAUDE_CODE_ENTRYPOINT` and the test-home override variable
//! are process-global, and the harness runs tests in threads, so every test
//! that touches the environment takes `ENV_LOCK` first and restores what it
//! found before releasing it. No test reads or writes the real home directory:
//! the trust directory always resolves inside the test's own `TempDir`.

use std::path::PathBuf;
use std::sync::Mutex;

/// Serialises every test that reads or writes a process-global variable.
static ENV_LOCK: Mutex<()> = Mutex::new(());

/// Take the environment lock, ignoring poisoning so one failing test does not
/// cascade into the rest of the file.
fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

/// The variables these tests set, restored on drop.
const MANAGED: &[&str] = &["DEVFORGEAI_HOME", "CLAUDECODE", "CLAUDE_CODE_ENTRYPOINT"];

/// Saves the managed variables on construction and puts them back on drop.
struct EnvGuard {
    saved: Vec<(&'static str, Option<std::ffi::OsString>)>,
}

impl EnvGuard {
    fn new() -> Self {
        EnvGuard {
            saved: MANAGED.iter().map(|k| (*k, std::env::var_os(k))).collect(),
        }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (k, v) in &self.saved {
            match v {
                Some(val) => std::env::set_var(k, val),
                None => std::env::remove_var(k),
            }
        }
    }
}

/// The `sha256:`-prefixed lowercase hex digest of `bytes`, computed here rather
/// than by the code under test so the byte format is pinned independently.
#[cfg_attr(not(feature = "test-home"), allow(dead_code))]
fn sha256_of(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    format!("sha256:{:x}", h.finalize())
}

// --- fixtures ------------------------------------------------------------

/// The git SHA-1 every fixture's `REVISION` line 1 carries.
#[cfg(feature = "test-home")]
const REV: &str = "a1b2c3d4e5f60718293a4b5c6d7e8f9012345678";

/// A temp root holding a home directory, a framework repo with a `cli/` tree,
/// and a synthetic binary.
#[cfg(feature = "test-home")]
struct Fixture {
    /// Kept alive so the temp tree outlives the test.
    tmp: tempfile::TempDir,
    /// The directory the test-home override points at.
    home: PathBuf,
    /// The framework repo root.
    framework: PathBuf,
    /// The synthetic binary being pinned.
    binary: PathBuf,
}

#[cfg(feature = "test-home")]
impl Fixture {
    /// The `cli/` directory of the framework repo.
    fn cli(&self) -> PathBuf {
        self.framework.join("cli")
    }
}

/// Build a fixture whose `cli/DIGEST` matches `bytes`, unless `digest`
/// overrides it with some other value.
#[cfg(feature = "test-home")]
fn fixture(bytes: &[u8], digest: Option<&str>) -> Fixture {
    use std::fs;

    let tmp = tempfile::tempdir().expect("temp dir");
    let root = tmp.path().to_path_buf();

    let home = root.join("home").join(".devforgeai");
    fs::create_dir_all(&home).expect("home dir");

    let framework = root.join("framework");
    let cli = framework.join("cli");
    fs::create_dir_all(cli.join("src")).expect("cli/src");
    fs::write(cli.join("src").join("main.rs"), b"fn main() {}\n").expect("main.rs");
    fs::write(cli.join("Cargo.toml"), b"[package]\nname = \"x\"\n").expect("Cargo.toml");
    // Line 2 is a placeholder. The source digest covers every file under `cli/`
    // including REVISION itself, so no value written into REVISION can equal
    // the digest of the tree that contains it. Tests that need the recorded
    // value to agree with the tree call `reconcile`.
    fs::write(
        cli.join("REVISION"),
        format!("{REV}\nsha256:{}\n", "0".repeat(64)),
    )
    .expect("REVISION");

    let binary = root.join("devforgeai-under-test.bin");
    fs::write(&binary, bytes).expect("binary");

    let d = digest
        .map(str::to_string)
        .unwrap_or_else(|| sha256_of(bytes));
    fs::write(cli.join("DIGEST"), format!("{d}\n")).expect("DIGEST");

    Fixture {
        tmp,
        home,
        framework,
        binary,
    }
}

/// Point the trust directory at the fixture's home.
#[cfg(feature = "test-home")]
fn use_home(f: &Fixture) {
    std::env::set_var("DEVFORGEAI_HOME", &f.home);
}

/// Clear both session variables.
#[cfg(feature = "test-home")]
fn no_session() {
    std::env::remove_var("CLAUDECODE");
    std::env::remove_var("CLAUDE_CODE_ENTRYPOINT");
}

/// Rewrite every pin's `source_digest` to the tree's current value.
///
/// A release cannot reach this state on its own: `cli/REVISION` line 2 is the
/// digest of a tree that includes `cli/REVISION`. The tests that exercise step
/// 5 need a pin whose recorded digest agrees with the tree, so they patch it
/// here and then introduce the drift they mean to detect.
#[cfg(feature = "test-home")]
fn reconcile(f: &Fixture) {
    let path = devforgeai::trust::trust_path();
    let text = std::fs::read_to_string(&path).expect("read trust.toml");
    let mut tf: devforgeai::trust::TrustFile = toml::from_str(&text).expect("parse trust.toml");
    let d = devforgeai::trust::source_digest(&f.cli()).expect("source digest");
    for p in tf.pin.iter_mut() {
        p.source_digest = d.clone();
    }
    std::fs::write(&path, toml::to_string(&tf).expect("serialise")).expect("write trust.toml");
}

/// Read the trust file the code under test wrote.
#[cfg(feature = "test-home")]
fn read_trust() -> devforgeai::trust::TrustFile {
    let text = std::fs::read_to_string(devforgeai::trust::trust_path()).expect("read trust.toml");
    toml::from_str(&text).expect("parse trust.toml")
}

// --- trust pin -----------------------------------------------------------

#[cfg(feature = "test-home")]
#[test]
fn pin_writes_entry() {
    use devforgeai::trust;

    let _g = env_lock();
    let _e = EnvGuard::new();
    let f = fixture(b"binary bytes", None);
    use_home(&f);
    no_session();

    let p = trust::pin(Some(&f.binary), Some(&f.framework)).expect("pin succeeds");

    assert_eq!(p.digest, sha256_of(b"binary bytes"));
    assert_eq!(p.revision, REV);
    assert_eq!(p.source_digest, format!("sha256:{}", "0".repeat(64)));
    assert_eq!(p.release_digest, sha256_of(b"binary bytes"));
    assert_eq!(
        PathBuf::from(&p.framework_path),
        devforgeai::project::normalise(&f.framework),
        "framework_path is the canonical repo root"
    );
    assert!(p.pinned_at.ends_with('Z'), "pinned_at is RFC 3339 UTC");
    assert!(p.pinned_by.contains('\\'), "pinned_by is <host>\\<user>");

    let tf = read_trust();
    assert_eq!(tf.schema, "devforgeai/trust/1");
    assert!(tf.updated_at.ends_with('Z'));
    assert_eq!(tf.pin.len(), 1, "one pin written");
    assert_eq!(tf.pin[0], p, "the written entry is the one returned");
    assert!(
        trust::trust_path().starts_with(&f.home),
        "the trust file lives under the test home"
    );
}

#[cfg(feature = "test-home")]
#[test]
fn pin_refuses_when_claudecode_set() {
    use devforgeai::trust;

    let _g = env_lock();
    let _e = EnvGuard::new();
    let f = fixture(b"binary bytes", None);
    use_home(&f);
    no_session();
    std::env::set_var("CLAUDECODE", "1");

    let err = trust::pin(Some(&f.binary), Some(&f.framework)).expect_err("refused");
    assert_eq!(err.code(), "DFA-E500");
    assert_eq!(err.exit(), 4);
    assert!(
        err.diag().expect("diag").message.contains("CLAUDECODE"),
        "the message names the variable"
    );
    assert!(
        !trust::trust_path().exists(),
        "nothing is written when the pin is refused"
    );
}

#[cfg(feature = "test-home")]
#[test]
fn pin_refuses_when_entrypoint_set() {
    use devforgeai::trust;

    let _g = env_lock();
    let _e = EnvGuard::new();
    let f = fixture(b"binary bytes", None);
    use_home(&f);
    no_session();
    std::env::set_var("CLAUDE_CODE_ENTRYPOINT", "cli");

    let err = trust::pin(Some(&f.binary), Some(&f.framework)).expect_err("refused");
    assert_eq!(err.code(), "DFA-E500");
    assert!(err
        .diag()
        .expect("diag")
        .message
        .contains("CLAUDE_CODE_ENTRYPOINT"));
    assert!(!trust::trust_path().exists(), "nothing written");
}

#[cfg(feature = "test-home")]
#[test]
fn pin_allows_empty_variable() {
    use devforgeai::trust;

    let _g = env_lock();
    let _e = EnvGuard::new();
    let f = fixture(b"binary bytes", None);
    use_home(&f);
    no_session();
    // Present but empty, and present but whitespace only, are both absent.
    std::env::set_var("CLAUDECODE", "");
    std::env::set_var("CLAUDE_CODE_ENTRYPOINT", "  \t ");

    assert_eq!(
        trust::session_active(),
        None,
        "an empty value is not active"
    );
    trust::pin(Some(&f.binary), Some(&f.framework)).expect("pin succeeds");
    assert_eq!(read_trust().pin.len(), 1);
}

#[cfg(feature = "test-home")]
#[test]
fn pin_digest_mismatch_gives_e503() {
    use devforgeai::trust;

    let _g = env_lock();
    let _e = EnvGuard::new();
    let other = sha256_of(b"some other build");
    let f = fixture(b"binary bytes", Some(&other));
    use_home(&f);
    no_session();

    let err = trust::pin(Some(&f.binary), Some(&f.framework)).expect_err("mismatch");
    assert_eq!(err.code(), "DFA-E503");
    assert_eq!(err.exit(), 4);
    assert!(!trust::trust_path().exists(), "nothing written");
}

/// The spec's claim that a `test-home` build has a different digest and so
/// cannot be trusted is the digest-mismatch path: a `cli/DIGEST` that is not
/// this binary's digest refuses the pin with `DFA-E503`, so no pin for such a
/// binary ever reaches `trust verify`.
#[cfg(feature = "test-home")]
#[test]
fn test_home_build_fails_verify_by_construction() {
    use devforgeai::trust;

    let _g = env_lock();
    let _e = EnvGuard::new();
    let release_digest = sha256_of(b"the release binary built with default features");
    let f = fixture(b"a binary built with test-home", Some(&release_digest));
    use_home(&f);
    no_session();

    let err = trust::pin(Some(&f.binary), Some(&f.framework)).expect_err("mismatch");
    assert_eq!(err.code(), "DFA-E503");
    assert!(!trust::trust_path().exists(), "no pin is recorded");
}

#[cfg(feature = "test-home")]
#[test]
fn pin_malformed_digest_file_gives_e510() {
    use devforgeai::trust;

    let _g = env_lock();
    let _e = EnvGuard::new();
    let f = fixture(b"binary bytes", Some("not-a-digest"));
    use_home(&f);
    no_session();

    let err = trust::pin(Some(&f.binary), Some(&f.framework)).expect_err("malformed DIGEST");
    assert_eq!(err.code(), "DFA-E510");
    assert_eq!(err.exit(), 4);

    // A malformed REVISION is the same code.
    std::fs::write(
        f.cli().join("DIGEST"),
        format!("{}\n", sha256_of(b"binary bytes")),
    )
    .expect("DIGEST");
    std::fs::write(f.cli().join("REVISION"), "not-a-sha\nnot-a-digest\n").expect("REVISION");
    let err = trust::pin(Some(&f.binary), Some(&f.framework)).expect_err("malformed REVISION");
    assert_eq!(err.code(), "DFA-E510");

    // So is a framework directory with no release files.
    let err = trust::pin(Some(&f.binary), Some(&f.tmp.path().join("nowhere")))
        .expect_err("absent framework");
    assert_eq!(err.code(), "DFA-E510");
    assert!(!trust::trust_path().exists(), "nothing written");
}

#[cfg(feature = "test-home")]
#[test]
fn pin_replaces_entry_for_same_path() {
    use devforgeai::trust;

    let _g = env_lock();
    let _e = EnvGuard::new();
    let f = fixture(b"binary bytes", None);
    use_home(&f);
    no_session();

    trust::pin(Some(&f.binary), Some(&f.framework)).expect("first pin");

    // A second release of the same framework, pinning the same path.
    std::fs::write(
        f.cli().join("REVISION"),
        format!("{}\nsha256:{}\n", "b".repeat(40), "1".repeat(64)),
    )
    .expect("REVISION");
    let p = trust::pin(Some(&f.binary), Some(&f.framework)).expect("second pin");

    let tf = read_trust();
    assert_eq!(tf.pin.len(), 1, "the entry is replaced, not appended");
    assert_eq!(tf.pin[0].revision, "b".repeat(40));
    assert_eq!(
        tf.pin[0].source_digest,
        format!("sha256:{}", "1".repeat(64))
    );
    assert_eq!(p.revision, "b".repeat(40));
}

#[cfg(feature = "test-home")]
#[test]
fn pin_outcome_reports_replacement() {
    use devforgeai::trust;

    let _g = env_lock();
    let _e = EnvGuard::new();
    let f = fixture(b"binary bytes", None);
    use_home(&f);
    no_session();

    let first = devforgeai::cmd::trust::pin(Some(&f.binary), Some(&f.framework)).expect("pin");
    assert_eq!(first.data["replaced"], serde_json::Value::Bool(false));
    assert_eq!(first.human.len(), 4, "four human lines");
    assert!(first.human[0].starts_with("Pinned    "));
    assert!(first.human[1].starts_with("Digest    sha256:"));
    assert!(first.human[1].ends_with("..."));
    assert!(first.human[2].starts_with("Revision  "));
    assert!(first.human[3].starts_with("Trust     "));
    assert_eq!(
        first.data["trust_file"],
        serde_json::Value::String(trust::trust_path().display().to_string())
    );

    let second = devforgeai::cmd::trust::pin(Some(&f.binary), Some(&f.framework)).expect("pin");
    assert_eq!(second.data["replaced"], serde_json::Value::Bool(true));
}

// --- trust verify --------------------------------------------------------

#[cfg(feature = "test-home")]
#[test]
fn verify_matching_digest_exits_zero() {
    use devforgeai::trust;

    let _g = env_lock();
    let _e = EnvGuard::new();
    let f = fixture(b"binary bytes", None);
    use_home(&f);
    no_session();

    trust::pin(Some(&f.binary), Some(&f.framework)).expect("pin");
    let r = trust::verify(Some(&f.binary)).expect("verify passes");

    assert_eq!(r.digest, r.pinned);
    assert!(!r.session_active);
    assert!(!r.source_digest_checked);
    assert!(r.warnings.is_empty(), "no warning on a fresh pin");

    let out = devforgeai::cmd::trust::verify(Some(&f.binary), false).expect("outcome");
    assert_eq!(out.exit, None, "exit 0");
    assert_eq!(out.human.len(), 1);
    assert!(
        out.human[0].starts_with("trust verify  ok  sha256:"),
        "human line was {:?}",
        out.human[0]
    );
    assert!(out.human[0].ends_with("..."), "the digest is truncated");

    let quiet = devforgeai::cmd::trust::verify(Some(&f.binary), true).expect("outcome");
    assert!(quiet.human.is_empty(), "--quiet prints nothing");
}

#[cfg(feature = "test-home")]
#[test]
fn verify_without_trust_file_gives_e501() {
    use devforgeai::trust;

    let _g = env_lock();
    let _e = EnvGuard::new();
    let f = fixture(b"binary bytes", None);
    std::env::set_var("DEVFORGEAI_HOME", f.tmp.path().join("no-such-home"));
    no_session();

    let err = trust::verify(Some(&f.binary)).expect_err("no trust file");
    assert_eq!(err.code(), "DFA-E501");
    assert_eq!(err.exit(), 4);
}

#[cfg(feature = "test-home")]
#[test]
fn verify_without_entry_gives_e502() {
    use devforgeai::trust;

    let _g = env_lock();
    let _e = EnvGuard::new();
    let f = fixture(b"binary bytes", None);
    use_home(&f);
    no_session();

    trust::pin(Some(&f.binary), Some(&f.framework)).expect("pin");

    let other = f.tmp.path().join("another.bin");
    std::fs::write(&other, b"binary bytes").expect("write");
    let err = trust::verify(Some(&other)).expect_err("no pin for this path");
    assert_eq!(err.code(), "DFA-E502");
    assert_eq!(err.exit(), 4);
}

#[cfg(feature = "test-home")]
#[test]
fn verify_modified_binary_gives_e503() {
    use devforgeai::trust;

    let _g = env_lock();
    let _e = EnvGuard::new();
    let f = fixture(b"binary bytes", None);
    use_home(&f);
    no_session();

    trust::pin(Some(&f.binary), Some(&f.framework)).expect("pin");
    std::fs::write(&f.binary, b"tampered bytes").expect("modify the binary");

    let err = trust::verify(Some(&f.binary)).expect_err("digest drift");
    assert_eq!(err.code(), "DFA-E503");
    assert_eq!(err.exit(), 4);
}

#[cfg(feature = "test-home")]
#[test]
fn verify_source_drift_in_session_gives_e504() {
    use devforgeai::trust;

    let _g = env_lock();
    let _e = EnvGuard::new();
    let f = fixture(b"binary bytes", None);
    use_home(&f);
    no_session();

    trust::pin(Some(&f.binary), Some(&f.framework)).expect("pin");
    reconcile(&f);

    std::env::set_var("CLAUDECODE", "1");
    let r = trust::verify(Some(&f.binary)).expect("an untouched tree verifies in a session");
    assert!(r.session_active);
    assert!(r.source_digest_checked, "the source digest is recomputed");

    std::fs::write(
        f.cli().join("src").join("main.rs"),
        b"fn main() { evil() }\n",
    )
    .expect("edit cli/");
    let err = trust::verify(Some(&f.binary)).expect_err("source drift");
    assert_eq!(err.code(), "DFA-E504");
    assert_eq!(err.exit(), 4);
}

#[cfg(feature = "test-home")]
#[test]
fn verify_source_drift_outside_session_exits_zero() {
    use devforgeai::trust;

    let _g = env_lock();
    let _e = EnvGuard::new();
    let f = fixture(b"binary bytes", None);
    use_home(&f);
    no_session();

    trust::pin(Some(&f.binary), Some(&f.framework)).expect("pin");
    reconcile(&f);
    std::fs::write(
        f.cli().join("src").join("main.rs"),
        b"fn main() { edited() }\n",
    )
    .expect("edit cli/");

    let r = trust::verify(Some(&f.binary)).expect("no session, so no source check");
    assert!(!r.session_active);
    assert!(!r.source_digest_checked, "step 5 is skipped");
}

#[cfg(feature = "test-home")]
#[test]
fn verify_older_pin_gives_w500() {
    use devforgeai::trust;

    let _g = env_lock();
    let _e = EnvGuard::new();
    let f = fixture(b"binary bytes", None);
    use_home(&f);
    no_session();

    trust::pin(Some(&f.binary), Some(&f.framework)).expect("pin");

    // The pin predates the binary file time; the digests still match.
    let path = trust::trust_path();
    let mut tf: trust::TrustFile =
        toml::from_str(&std::fs::read_to_string(&path).expect("read")).expect("parse");
    for p in tf.pin.iter_mut() {
        p.pinned_at = "2000-01-01T00:00:00Z".to_string();
    }
    std::fs::write(&path, toml::to_string(&tf).expect("serialise")).expect("write");

    let r = trust::verify(Some(&f.binary)).expect("a stale pin is still a pass");
    assert_eq!(r.warnings.len(), 1, "one warning");
    assert_eq!(r.warnings[0].code, "DFA-W500");

    let out = devforgeai::cmd::trust::verify(Some(&f.binary), false).expect("outcome");
    assert_eq!(out.exit, None, "the warning does not change the exit code");
    assert_eq!(out.warnings.len(), 1);
    assert_eq!(out.warnings[0].code, "DFA-W500");
}

#[cfg(feature = "test-home")]
#[test]
fn verify_json_data_carries_the_specified_keys() {
    use devforgeai::trust;

    let _g = env_lock();
    let _e = EnvGuard::new();
    let f = fixture(b"binary bytes", None);
    use_home(&f);
    no_session();

    trust::pin(Some(&f.binary), Some(&f.framework)).expect("pin");
    let out = devforgeai::cmd::trust::verify(Some(&f.binary), false).expect("outcome");
    for key in [
        "binary",
        "digest",
        "pinned",
        "match",
        "session_active",
        "source_digest_checked",
    ] {
        assert!(out.data.get(key).is_some(), "data carries {key}");
    }
    assert_eq!(out.data["match"], serde_json::Value::Bool(true));
}

// --- the source digest ---------------------------------------------------

/// Build a `cli/` tree, creating the files in the order given.
#[cfg(feature = "test-home")]
fn tree(files: &[(&str, &[u8])]) -> tempfile::TempDir {
    let t = tempfile::tempdir().expect("temp dir");
    let cli = t.path().join("cli");
    for (rel, bytes) in files {
        let p = cli.join(rel);
        std::fs::create_dir_all(p.parent().expect("parent")).expect("mkdir");
        std::fs::write(&p, bytes).expect("write");
    }
    t
}

#[cfg(feature = "test-home")]
#[test]
fn source_digest_is_order_independent_of_walk() {
    use sha2::{Digest, Sha256};

    let forward: &[(&str, &[u8])] = &[
        ("DIGEST", b"sha256:ignored"),
        ("a.txt", b"alpha"),
        ("sub/b.txt", b"beta"),
        ("z.txt", b"zeta"),
        ("target/junk.bin", b"build output"),
        (".git/config", b"[core]"),
    ];
    let mut reverse: Vec<(&str, &[u8])> = forward.to_vec();
    reverse.reverse();

    let a = tree(forward);
    let b = tree(&reverse);
    let da = devforgeai::trust::source_digest(&a.path().join("cli")).expect("digest a");
    let db = devforgeai::trust::source_digest(&b.path().join("cli")).expect("digest b");
    assert_eq!(da, db, "the walk order does not reach the hash");

    // The byte format itself, computed here from the spec's description: the
    // forward-slash path relative to the framework root, a NUL, the length as
    // eight big-endian bytes, then the bytes. `cli/DIGEST`, `target/` and
    // `.git/` take no part.
    let mut h = Sha256::new();
    for (name, bytes) in [
        ("cli/a.txt", &b"alpha"[..]),
        ("cli/sub/b.txt", &b"beta"[..]),
        ("cli/z.txt", &b"zeta"[..]),
    ] {
        h.update(name.as_bytes());
        h.update([0u8]);
        h.update((bytes.len() as u64).to_be_bytes());
        h.update(bytes);
    }
    let expected = format!("sha256:{:x}", h.finalize());
    assert_eq!(da, expected, "the concatenation is the one the spec fixes");
}

#[cfg(feature = "test-home")]
#[test]
fn digest_file_is_the_sha256_of_the_bytes() {
    let t = tempfile::tempdir().expect("temp dir");
    let p = t.path().join("some.bin");
    std::fs::write(&p, b"binary bytes").expect("write");
    assert_eq!(
        devforgeai::trust::digest_file(&p).expect("digest"),
        sha256_of(b"binary bytes")
    );
}

// --- the two guard tests the harness rules name --------------------------

/// The test-home override variable must be read by no code path when the
/// feature is off. Two feature sets cannot be compiled in one run, so the
/// guarantee is checked by scanning the source: the variable's name occurs once
/// in `src/trust.rs`, inside a `#[cfg(feature = "test-home")]` region.
#[test]
fn trust_home_env_ignored_without_test_feature() {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("trust.rs");
    let text = std::fs::read_to_string(&src).expect("read src/trust.rs");
    let needle = concat!("DEVFORGEAI", "_HOME");

    let hits: Vec<usize> = text.match_indices(needle).map(|(i, _)| i).collect();
    assert_eq!(
        hits.len(),
        1,
        "the override variable is named exactly once in {}; found {} occurrences",
        src.display(),
        hits.len()
    );

    let before = &text[..hits[0]];
    let attr = before
        .rfind("#[cfg(")
        .expect("the read is guarded by a cfg attribute");
    assert!(
        text[attr..].starts_with("#[cfg(feature = \"test-home\")]"),
        "the nearest cfg above the read is the test-home gate, not {:?}",
        &text[attr..attr + 40.min(text.len() - attr)]
    );
}

/// With default features the override is unread, so the trust directory is the
/// real home whatever the variable says.
#[cfg(not(feature = "test-home"))]
#[test]
fn trust_home_ignores_the_override_without_the_feature() {
    let _g = env_lock();
    let _e = EnvGuard::new();
    let t = tempfile::tempdir().expect("temp dir");
    std::env::set_var("DEVFORGEAI_HOME", t.path());
    assert!(
        !devforgeai::trust::trust_home().starts_with(t.path()),
        "the variable is read by no code path with the feature off"
    );
}

/// The resolved trust directory is inside the test's own temp root, so no test
/// in this file can reach the real home.
#[cfg(feature = "test-home")]
#[test]
fn no_test_touches_real_home() {
    let _g = env_lock();
    let _e = EnvGuard::new();
    let t = tempfile::tempdir().expect("temp dir");
    let home = t.path().join("home").join(".devforgeai");
    std::env::set_var("DEVFORGEAI_HOME", &home);

    let resolved = devforgeai::trust::trust_home();
    assert!(
        resolved.starts_with(t.path()),
        "{} is inside {}",
        resolved.display(),
        t.path().display()
    );
    assert!(
        devforgeai::trust::trust_path().starts_with(t.path()),
        "the trust file is inside the temp root too"
    );

    // And it is never the real `~/.devforgeai`. On Windows the temp root itself
    // lives under `%USERPROFILE%`, so the real home is ruled out by the
    // directory the CLI would otherwise pick, not by a prefix test.
    let real = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from);
    if let Some(real) = real {
        assert_ne!(
            resolved,
            real.join(".devforgeai"),
            "the resolved trust directory is not the real one"
        );
    }
}

// --- the release files are not part of the digest they record -------------

#[test]
fn source_digest_excludes_both_release_files() {
    let t = tempfile::tempdir().expect("temp");
    let cli = t.path().join("cli");
    std::fs::create_dir_all(cli.join("src")).expect("mkdir");
    std::fs::write(cli.join("src").join("main.rs"), "fn main() {}\n").expect("write");

    let bare = devforgeai::trust::source_digest(&cli).expect("digest");

    // `cli/DIGEST` holds the binary's hash and `cli/REVISION` line 2 holds this
    // very value: a file cannot contain its own digest, so including either
    // makes the result unreproducible.
    std::fs::write(cli.join("DIGEST"), "sha256:00\n").expect("write");
    std::fs::write(cli.join("REVISION"), "unversioned\nsha256:00\n").expect("write");
    let with_release = devforgeai::trust::source_digest(&cli).expect("digest");

    assert_eq!(
        bare, with_release,
        "writing the release files does not change the digest they record"
    );
}

#[test]
fn source_digest_changes_when_a_source_file_changes() {
    let t = tempfile::tempdir().expect("temp");
    let cli = t.path().join("cli");
    std::fs::create_dir_all(cli.join("src")).expect("mkdir");
    std::fs::write(cli.join("src").join("main.rs"), "fn main() {}\n").expect("write");
    let before = devforgeai::trust::source_digest(&cli).expect("digest");

    std::fs::write(cli.join("src").join("main.rs"), "fn main() { }\n").expect("write");
    let after = devforgeai::trust::source_digest(&cli).expect("digest");

    assert_ne!(before, after, "the positive control");
}

#[test]
fn source_digest_is_stable_across_runs() {
    let t = tempfile::tempdir().expect("temp");
    let cli = t.path().join("cli");
    std::fs::create_dir_all(cli.join("src")).expect("mkdir");
    std::fs::write(cli.join("src").join("a.rs"), "// a\n").expect("write");
    std::fs::write(cli.join("src").join("b.rs"), "// b\n").expect("write");

    let first = devforgeai::trust::source_digest(&cli).expect("digest");
    let second = devforgeai::trust::source_digest(&cli).expect("digest");
    assert_eq!(first, second, "the walk order is fixed");
    assert!(first.starts_with("sha256:"));
}

/// The shipped `cli/REVISION` and the walk this binary runs must agree.
///
/// They are the two halves of the same claim, written at different times by
/// different steps, and a drift between them makes `trust verify` refuse every
/// write with `DFA-E504` on a tree nobody touched. The pair is checked here so
/// a change to the walk cannot ship without the file being regenerated.
///
/// Skipped when `cli/REVISION` is absent, which is every checkout that has not
/// been through a release build.
#[test]
fn the_walk_digest_matches_the_shipped_revision() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the framework root")
        .to_path_buf();
    let revision = repo.join("cli").join("REVISION");
    let Ok(text) = std::fs::read_to_string(&revision) else {
        return;
    };
    let Some(recorded) = text.lines().nth(1).map(str::trim) else {
        panic!("cli/REVISION carries a second line holding the source digest");
    };

    let walked = devforgeai::trust::source_digest(&repo.join("cli")).expect("the walk runs");
    assert_eq!(
        walked, recorded,
        "cli/REVISION line 2 and trust::source_digest disagree; regenerate with \
         `devforgeai trust digest --framework .`"
    );
}

#[test]
fn source_digest_excludes_target_only_at_the_root() {
    let t = tempfile::tempdir().expect("temp");
    let cli = t.path().join("cli");
    std::fs::create_dir_all(cli.join("src")).expect("mkdir");
    std::fs::write(cli.join("src").join("main.rs"), "fn main() {}\n").expect("write");
    let before = devforgeai::trust::source_digest(&cli).expect("digest");

    // `cli/target/` is the build directory and is excluded.
    std::fs::create_dir_all(cli.join("target")).expect("mkdir");
    std::fs::write(cli.join("target").join("a.bin"), "x").expect("write");
    assert_eq!(
        devforgeai::trust::source_digest(&cli).expect("digest"),
        before,
        "the build directory is not source"
    );

    // A source directory that happens to be called `target` is source. Matching
    // the name at any depth would drop it from the digest, so a change inside
    // it would not alter the source digest and `trust verify` would not see it.
    std::fs::create_dir_all(cli.join("src").join("target")).expect("mkdir");
    std::fs::write(cli.join("src").join("target").join("mod.rs"), "// x\n").expect("write");
    assert_ne!(
        devforgeai::trust::source_digest(&cli).expect("digest"),
        before,
        "cli/src/target/ is source and must be covered"
    );
}
