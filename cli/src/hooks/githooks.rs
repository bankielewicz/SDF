//! The three git hook scripts `hook install` writes, and the `@@DEVFORGEAI@@`
//! token substitution every template shares.
//!
//! The bodies are transcribed from the spec's `## Templates` section byte for
//! byte. They are POSIX `sh`: no `bash` extension, no array, no `[[`, so Git
//! for Windows runs the same copy through its bundled `sh`.

use std::path::{Path, PathBuf};

/// The line every script carries immediately after the shebang. A hook file
/// without it was written by someone else, and `--force` is then required.
pub const MARKER: &str = "# devforgeai-hook v1";

/// The token every template carries where the invocation of this binary goes.
pub const TOKEN: &str = "@@DEVFORGEAI@@";

/// `templates/pre-commit`: validate the staged `.devforgeai/` documents and
/// audit the context directory when one exists.
pub const PRE_COMMIT: &str = r#"#!/bin/sh
# devforgeai-hook v1
DFA="@@DEVFORGEAI@@"
"$DFA" trust verify || exit 4

# A failure here must not read as "nothing staged": an unwritable list or a
# failed rev-parse would otherwise let the commit through with no document
# validated.
git_dir=$(git rev-parse --git-dir) || exit 1
[ -n "$git_dir" ] || exit 1
list="$git_dir/devforgeai-staged"
git diff --cached --name-only --diff-filter=ACM -- .devforgeai > "$list" || exit 1

status=0
while IFS= read -r f; do
  [ -n "$f" ] || continue
  "$DFA" doc validate "$f" || status=1
done < "$list"
rm -f "$list"

if [ -d .devforgeai/context ]; then
  "$DFA" context audit || status=1
fi

exit $status
"#;

/// `templates/commit-msg`: require a `STORY-nnn` or `ADR-nnn` token in the
/// commit message.
pub const COMMIT_MSG: &str = r#"#!/bin/sh
# devforgeai-hook v1
DFA="@@DEVFORGEAI@@"
"$DFA" trust verify || exit 4

if grep -Eq '(STORY|ADR)-[0-9][0-9][0-9]' "$1"; then
  exit 0
fi

echo "devforgeai: commit message has no STORY-nnn or ADR-nnn token" >&2
exit 1
"#;

/// `templates/pre-push`: run the build gate for every story the sprint file
/// marks `building`.
pub const PRE_PUSH: &str = r#"#!/bin/sh
# devforgeai-hook v1
DFA="@@DEVFORGEAI@@"
"$DFA" trust verify || exit 4

sprint=".devforgeai/stories/sprint.yaml"
[ -f "$sprint" ] || exit 0

ids=$(awk '
  /^[[:space:]]*-[[:space:]]*id:/ { id = $3 }
  /^[[:space:]]*status:[[:space:]]*building[[:space:]]*$/ { if (id != "") print id }
' "$sprint")

status=0
for id in $ids; do
  "$DFA" gate check --phase build --id "$id" || status=1
done

exit $status
"#;

/// The three scripts in the order `hook install` writes them, each paired with
/// the hook file name it takes.
pub const SCRIPTS: &[(&str, &str)] = &[
    ("pre-commit", PRE_COMMIT),
    ("commit-msg", COMMIT_MSG),
    ("pre-push", PRE_PUSH),
];

/// Replace `@@DEVFORGEAI@@` with `token_value`, and normalise the result to LF
/// endings.
///
/// The normalisation is deliberate: a Rust string literal carries whatever
/// bytes the source file holds, so a checkout that landed the sources with CRLF
/// would otherwise leak CRLF into the hook files the spec fixes as LF.
pub fn substitute(body: &str, token_value: &str) -> String {
    body.replace("\r\n", "\n").replace(TOKEN, token_value)
}

/// The value `@@DEVFORGEAI@@` takes at install time: the bare word
/// `devforgeai` when that name resolves on `PATH`, and otherwise the absolute
/// path of the running binary in forward-slash form.
pub fn token_value() -> String {
    if which("devforgeai").is_some() {
        return "devforgeai".to_string();
    }
    match std::env::current_exe() {
        Ok(p) => forward_slash(&p),
        Err(_) => "devforgeai".to_string(),
    }
}

/// Render a path with forward slashes and without the Windows `\\?\` verbatim
/// prefix, which is correct as a path and wrong inside a shell script.
fn forward_slash(p: &Path) -> String {
    let s = p.to_string_lossy();
    let s = s.strip_prefix(r"\\?\").unwrap_or(&s);
    s.replace('\\', "/")
}

/// The first entry on `PATH` naming an existing file called `name`, trying the
/// Windows executable extensions when the platform has them.
fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        let plain = dir.join(name);
        if plain.is_file() {
            return Some(plain);
        }
        for ext in exts() {
            let candidate = dir.join(format!("{name}{ext}"));
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// The executable extensions to try beside the bare name. Empty off Windows.
fn exts() -> Vec<String> {
    if !cfg!(windows) {
        return Vec::new();
    }
    match std::env::var("PATHEXT") {
        Ok(v) if !v.trim().is_empty() => v
            .split(';')
            .map(str::trim)
            .filter(|e| !e.is_empty())
            .map(|e| e.to_ascii_lowercase())
            .collect(),
        _ => vec![".exe".to_string(), ".cmd".to_string(), ".bat".to_string()],
    }
}

/// True when `text` carries the marker within its opening lines, which is what
/// identifies a hook file this binary wrote.
pub fn has_marker(text: &str) -> bool {
    text.lines().take(3).any(|l| l.trim() == MARKER)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_script_opens_with_the_shebang_and_the_marker() {
        for (name, body) in SCRIPTS {
            let mut lines = body.lines();
            assert_eq!(lines.next(), Some("#!/bin/sh"), "{name} shebang");
            assert_eq!(lines.next(), Some(MARKER), "{name} marker");
        }
    }

    #[test]
    fn substitution_replaces_every_token_and_normalises_endings() {
        let crlf = "#!/bin/sh\r\nDFA=\"@@DEVFORGEAI@@\"\r\n\"$DFA\" trust verify\r\n";
        let out = substitute(crlf, "devforgeai");
        assert!(!out.contains('\r'), "CRLF is normalised away");
        assert!(!out.contains(TOKEN), "no token survives");
        assert!(out.contains("DFA=\"devforgeai\""));
    }

    #[test]
    fn marker_detection_rejects_a_foreign_script() {
        assert!(has_marker(PRE_PUSH));
        assert!(!has_marker("#!/bin/sh\necho someone else\n"));
    }
}
