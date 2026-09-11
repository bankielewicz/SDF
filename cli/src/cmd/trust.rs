//! `devforgeai trust pin` and `devforgeai trust verify`: the human lines and
//! the `data` object each one renders, over the logic in [`crate::trust`].

use crate::errors::CliError;
use crate::trust;
use crate::Outcome;
use serde_json::json;
use std::path::Path;

/// A digest as the human output shows it: `sha256:` and the first six hex
/// characters, then an ellipsis.
fn short_digest(digest: &str) -> String {
    match digest.strip_prefix("sha256:") {
        Some(hex) => format!("sha256:{}...", &hex[..hex.len().min(6)]),
        None => digest.to_string(),
    }
}

/// A revision as the human output shows it: the first twelve characters.
fn short_revision(revision: &str) -> String {
    revision.chars().take(12).collect()
}

/// Pin the binary's SHA-256 in `~/.devforgeai/trust.toml`.
pub fn pin(binary: Option<&Path>, framework: Option<&Path>) -> Result<Outcome, CliError> {
    // Whether an entry already exists is cosmetic, so it is read best-effort:
    // a corrupt trust file must still surface as the error `trust::pin` raises
    // at the step the spec puts it, not earlier and not as a different code.
    let replaced = already_pinned(binary);

    let entry = trust::pin(binary, framework)?;
    let path = trust::trust_path();

    let mut outcome = Outcome::data(json!({
        "binary": entry.binary_path,
        "digest": entry.digest,
        "revision": entry.revision,
        "source_digest": entry.source_digest,
        "framework_path": entry.framework_path,
        "trust_file": path.display().to_string(),
        "replaced": replaced,
    }));
    outcome.human = vec![
        format!("{:<10}{}", "Pinned", entry.binary_path),
        format!("{:<10}{}", "Digest", short_digest(&entry.digest)),
        format!("{:<10}{}", "Revision", short_revision(&entry.revision)),
        format!("{:<10}{}", "Trust", path.display()),
    ];
    Ok(outcome)
}

/// True when `trust.toml` already carries a `[[pin]]` for this binary path.
/// Every failure answers `false`: this decides one boolean in the envelope.
fn already_pinned(binary: Option<&Path>) -> bool {
    let Ok(bin) = trust::resolve_binary(binary) else {
        return false;
    };
    let Ok(Some(file)) = trust::load_trust_file() else {
        return false;
    };
    let bin = bin.to_string_lossy();
    file.pin.iter().any(|p| {
        if cfg!(windows) {
            p.binary_path.eq_ignore_ascii_case(&bin)
        } else {
            p.binary_path == bin
        }
    })
}

/// Verify the binary against its pin. Writes nothing, exits 0 or 4.
pub fn verify(binary: Option<&Path>, quiet: bool) -> Result<Outcome, CliError> {
    let report = trust::verify(binary)?;

    let mut outcome = Outcome::data(json!({
        "binary": report.binary,
        "digest": report.digest,
        "pinned": report.pinned,
        "match": report.digest == report.pinned,
        "session_active": report.session_active,
        "source_digest_checked": report.source_digest_checked,
    }));
    if !quiet {
        outcome.human = vec![format!(
            "trust verify  ok  {}",
            short_digest(&report.digest)
        )];
    }
    outcome.warnings = report.warnings;
    Ok(outcome)
}

/// Print the two release digests, for the release step that writes
/// `cli/REVISION` and `cli/DIGEST`.
///
/// The release files are read back by `trust verify` through
/// [`crate::trust::source_digest`] and [`crate::trust::digest_file`]. Writing
/// them from the binary's own code path is what keeps the two from drifting:
/// an external script reimplementing the walk would diverge silently the first
/// time an exclusion changed, and the drift would surface only as `DFA-E504`
/// refusing every write on a developer's machine.
pub fn digest(framework: Option<&Path>, binary: Option<&Path>) -> Result<Outcome, CliError> {
    let root = match framework {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir().map_err(|e| CliError::io("reading", "cwd", &e))?,
    };
    let source_digest = trust::source_digest(&root.join("cli"))?;
    let bin = trust::resolve_binary(binary)?;
    let binary_digest = trust::digest_file(&bin)?;

    // `cli/REVISION` line 1 is the revision and line 2 the source digest;
    // `cli/DIGEST` is the binary's own hash. A build made from a work tree
    // records the commit it came from, which is what lets an operator tell two
    // binaries apart and what `--version` shows; a build made from an exported
    // tree has no commit to name and says so.
    let revision =
        head_revision(&root).unwrap_or_else(|| crate::trust::REVISION_UNVERSIONED.into());
    let mut outcome = Outcome::data(json!({
        "framework": root.display().to_string(),
        "binary": bin.display().to_string(),
        "revision": revision,
        "source_digest": source_digest,
        "binary_digest": binary_digest,
    }));
    outcome.human = vec![
        format!("REVISION  {revision}"),
        format!("SOURCE    {source_digest}"),
        format!("DIGEST    {binary_digest}"),
    ];
    Ok(outcome)
}

/// The commit `HEAD` names in `root`, or `None` when it is not a work tree.
///
/// Every failure answers `None`: git absent, the directory not a repository,
/// a detached or unborn HEAD. The revision is a label, so an unavailable one
/// degrades to `unversioned` rather than failing the release step.
fn head_revision(root: &Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?.trim().to_string();
    // `REVISION` line 1 is validated as a 40-character lowercase SHA-1 or the
    // literal `unversioned`, so anything else is not written.
    let ok = text.len() == 40
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b));
    ok.then_some(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_digest_is_shown_as_seven_characters_and_an_ellipsis() {
        assert_eq!(
            short_digest(&format!("sha256:{}", "7f0c1a2b3c".repeat(6) + "7f0c")),
            "sha256:7f0c1a..."
        );
        assert_eq!(short_digest("nonsense"), "nonsense");
    }

    #[test]
    fn a_revision_is_shown_as_twelve_characters() {
        assert_eq!(short_revision("a1b2c3d4e5f60718293a"), "a1b2c3d4e5f6");
        assert_eq!(short_revision("unversioned"), "unversioned");
    }
}
