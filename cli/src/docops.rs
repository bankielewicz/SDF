//! `doc accept requirements` and `doc reopen requirements`.
//!
//! Both edit `requirements.yaml` in place and leave every other byte untouched,
//! so the edits are line surgery rather than a YAML round trip, which would
//! reorder keys and drop comments.

use crate::ctx::Ctx;
use crate::doc::ids;
use crate::errors::CliError;
use crate::project;

/// What `accept_requirements` did.
#[derive(Debug, Clone)]
pub struct Accepted {
    /// The RFC 3339 UTC timestamp written.
    pub accepted_at: String,
    /// The count of requirements moved to `accepted`.
    pub moved: usize,
    /// The count of requirements in the document.
    pub requirements: usize,
    /// The count of epics in the document.
    pub epics: usize,
}

/// What `reopen_requirements` did.
#[derive(Debug, Clone)]
pub struct Reopened {
    /// The new revision number.
    pub revision: i64,
    /// The `REQ-nnn` ids moved to `reopened`.
    pub reopened: Vec<String>,
    /// The `REQ-nnn` ids allocated, one per cited `UI-nnn`.
    pub allocated: Vec<String>,
}

/// The zero-based line range of the top-level block `key:` opens, and the
/// indent its entries use.
struct Block {
    start: usize,
    end: usize,
}

fn top_level_block(lines: &[&str], key: &str) -> Option<Block> {
    let head = format!("{key}:");
    let start = lines
        .iter()
        .position(|l| l.trim_end() == head || l.starts_with(&format!("{head} ")))?;
    let mut end = start + 1;
    while end < lines.len() {
        let l = lines[end];
        if l.trim().is_empty() {
            end += 1;
            continue;
        }
        // A top-level key ends the block.
        if !l.starts_with(' ') && !l.starts_with('\t') && !l.starts_with('-') {
            break;
        }
        end += 1;
    }
    Some(Block { start, end })
}

/// The zero-based index of a top-level scalar key, if present.
fn top_level_key(lines: &[&str], key: &str) -> Option<usize> {
    let head = format!("{key}:");
    lines
        .iter()
        .position(|l| l.trim_end() == head || l.starts_with(&format!("{head} ")))
}

/// Replace the value of `key:` on `line`, keeping its indent.
fn set_value(line: &str, key: &str, value: &str) -> String {
    let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
    let dash = line.trim_start().starts_with("- ");
    if dash {
        format!("{indent}- {key}: {value}")
    } else {
        format!("{indent}{key}: {value}")
    }
}

/// The count of `- id:` entries inside a top-level block.
fn count_entries(lines: &[&str], block: &Block) -> usize {
    lines[block.start + 1..block.end]
        .iter()
        .filter(|l| {
            let t = l.trim_start();
            t.starts_with("- id:") || t.starts_with("- ")
        })
        .filter(|l| l.trim_start().starts_with("- "))
        .count()
}

/// Write `accepted_by: user`, `accepted_at`, the document `status: accepted`,
/// and move every requirement at `draft` or `reopened` to `accepted`.
pub fn accept_requirements(ctx: &mut Ctx, id: &str) -> Result<Accepted, CliError> {
    let path = ctx.dot().join("requirements.yaml");
    let text = project::read_doc(&path)?;
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let ends_with_newline = text.ends_with('\n');

    {
        let view: Vec<&str> = lines.iter().map(String::as_str).collect();
        let reqs = top_level_block(&view, "requirements");
        let epics = top_level_block(&view, "epics");
        let req_count = reqs.as_ref().map(|b| count_entries(&view, b)).unwrap_or(0);
        let epic_count = epics.as_ref().map(|b| count_entries(&view, b)).unwrap_or(0);
        if req_count == 0 || epic_count == 0 {
            let which = if epic_count == 0 {
                "epics"
            } else {
                "requirements"
            };
            return Err(CliError::at(
                "DFA-E260",
                format!("requirements.yaml has no {which}; nothing accepted"),
                ctx.rel(&path),
            ));
        }
    }

    let now = crate::time::now_rfc3339();
    let mut moved = 0usize;

    // Move every requirement at draft or reopened to accepted.
    let (req_start, req_end) = {
        let view: Vec<&str> = lines.iter().map(String::as_str).collect();
        match top_level_block(&view, "requirements") {
            Some(b) => (b.start, b.end),
            None => (0, 0),
        }
    };
    for line in lines.iter_mut().take(req_end).skip(req_start) {
        let t = line.trim_start().to_string();
        let Some(rest) = t
            .strip_prefix("status:")
            .or_else(|| t.strip_prefix("- status:"))
        else {
            continue;
        };
        let value = rest.trim().trim_matches(['"', '\'']);
        if value != "draft" && value != "reopened" {
            continue;
        }
        let key_is_dashed = t.starts_with("- ");
        let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
        *line = if key_is_dashed {
            format!("{indent}- status: accepted")
        } else {
            format!("{indent}status: accepted")
        };
        moved += 1;
    }

    // The document status.
    let view: Vec<&str> = lines.iter().map(String::as_str).collect();
    let status_line = top_level_key(&view, "status");
    if let Some(i) = status_line {
        lines[i] = set_value(&lines[i], "status", "accepted");
    }

    // accepted_by and accepted_at: replace when present, append when absent.
    let view: Vec<&str> = lines.iter().map(String::as_str).collect();
    let by = top_level_key(&view, "accepted_by");
    let at = top_level_key(&view, "accepted_at");
    match by {
        Some(i) => lines[i] = set_value(&lines[i], "accepted_by", "user"),
        None => lines.push("accepted_by: user".to_string()),
    }
    match at {
        Some(i) => lines[i] = set_value(&lines[i], "accepted_at", &now),
        None => lines.push(format!("accepted_at: {now}")),
    }

    let (requirements, epics) = {
        let view: Vec<&str> = lines.iter().map(String::as_str).collect();
        (
            top_level_block(&view, "requirements")
                .map(|b| count_entries(&view, &b))
                .unwrap_or(0),
            top_level_block(&view, "epics")
                .map(|b| count_entries(&view, &b))
                .unwrap_or(0),
        )
    };

    let mut out = lines.join("\n");
    if ends_with_newline {
        out.push('\n');
    }
    project::atomic_write(&path, out.as_bytes())?;

    let _ = id;
    Ok(Accepted {
        accepted_at: now,
        moved,
        requirements,
        epics,
    })
}

/// Raise `revision`, append the `revision_log` entry, move each cited
/// `REQ-nnn` to `reopened`, allocate one `REQ-nnn` per cited `UI-nnn`, null the
/// acceptance, and set the document `status: reopened`.
pub fn reopen_requirements(
    ctx: &mut Ctx,
    id: &str,
    cited: &[String],
    from: &str,
) -> Result<Reopened, CliError> {
    for c in cited {
        let ok = ids::split_id(c)
            .map(|(p, _)| p == "REQ" || p == "UI")
            .unwrap_or(false);
        if !ok {
            return Err(CliError::new(
                "DFA-E261",
                format!("'{c}' is neither a REQ nor a UI id"),
            ));
        }
    }

    let path = ctx.dot().join("requirements.yaml");
    let text = project::read_doc(&path)?;
    let rel = ctx.rel(&path);
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let ends_with_newline = text.ends_with('\n');

    let reqs: Vec<&String> = cited
        .iter()
        .filter(|c| ids::split_id(c).map(|(p, _)| p == "REQ").unwrap_or(false))
        .collect();
    let uis: Vec<&String> = cited
        .iter()
        .filter(|c| ids::split_id(c).map(|(p, _)| p == "UI").unwrap_or(false))
        .collect();

    // Every cited REQ is in the document.
    let index = ids::build(&ctx.root);
    for r in &reqs {
        if !index.defines(r) {
            return Err(CliError::at(
                "DFA-E210",
                format!("{rel} references {r}, which no document defines"),
                rel.clone(),
            ));
        }
    }

    // Move each cited REQ to reopened, in place.
    let mut reopened: Vec<String> = Vec::new();
    let (req_start, req_end) = {
        let view: Vec<&str> = lines.iter().map(String::as_str).collect();
        match top_level_block(&view, "requirements") {
            Some(b) => (b.start, b.end),
            None => (0, 0),
        }
    };
    let mut current: Option<String> = None;
    for line in lines.iter_mut().take(req_end).skip(req_start) {
        let t = line.trim_start().to_string();
        if let Some(rest) = t.strip_prefix("- id:").or_else(|| {
            if t.starts_with("id:") {
                t.strip_prefix("id:")
            } else {
                None
            }
        }) {
            current = Some(rest.trim().trim_matches(['"', '\'']).to_string());
            continue;
        }
        if t.starts_with("status:") {
            if let Some(cur) = &current {
                if reqs.contains(&cur) {
                    let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
                    *line = format!("{indent}status: reopened");
                    reopened.push(cur.clone());
                }
            }
        }
    }

    // Allocate one REQ per cited UI.
    let mut allocated: Vec<String> = Vec::new();
    let mut working = index.clone();
    for ui in &uis {
        let new_id = working.allocate("REQ")?;
        working.definitions.insert(
            new_id.clone(),
            vec![ids::Site {
                path: rel.clone(),
                line: 0,
                schema: String::new(),
            }],
        );
        allocated.push(new_id);
        let _ = ui;
    }
    if !allocated.is_empty() {
        let view: Vec<&str> = lines.iter().map(String::as_str).collect();
        if let Some(b) = top_level_block(&view, "requirements") {
            let mut insert: Vec<String> = Vec::new();
            for (new_id, ui) in allocated.iter().zip(uis.iter()) {
                insert.push(format!("  - id: {new_id}"));
                insert.push("    source: user".to_string());
                insert.push(format!("    traces_to: [{ui}]"));
                insert.push("    status: reopened".to_string());
            }
            let at = b.end.min(lines.len());
            for (k, l) in insert.into_iter().enumerate() {
                lines.insert(at + k, l);
            }
        }
    }

    // Raise revision.
    let view: Vec<&str> = lines.iter().map(String::as_str).collect();
    let revision = match top_level_key(&view, "revision") {
        Some(i) => {
            let cur: i64 = lines[i]
                .split_once(':')
                .map(|(_, v)| v.trim())
                .and_then(|v| v.parse().ok())
                .unwrap_or(1);
            let next = cur + 1;
            lines[i] = set_value(&lines[i], "revision", &next.to_string());
            next
        }
        None => {
            // An absent key reads as the document's first revision, so the
            // first reopen writes 2, which is what the spec's example shows.
            lines.push("revision: 2".to_string());
            2
        }
    };

    // Null the acceptance and set the document status.
    for key in ["accepted_by", "accepted_at"] {
        let found = {
            let view: Vec<&str> = lines.iter().map(String::as_str).collect();
            top_level_key(&view, key)
        };
        if let Some(i) = found {
            lines[i] = set_value(&lines[i], key, "null");
        }
    }
    let view: Vec<&str> = lines.iter().map(String::as_str).collect();
    if let Some(i) = top_level_key(&view, "status") {
        lines[i] = set_value(&lines[i], "status", "reopened");
    }

    // Append the revision_log entry. The entry shape is this spec's, since
    // `specs/03-discover.md` owns the document and 01-cli.md names the key
    // without fixing its fields.
    let now = crate::time::now_rfc3339();
    let entry = vec![
        format!("  - revision: {revision}"),
        format!("    at: {now}"),
        format!("    from: {from}"),
        format!("    ids: [{}]", cited.join(", ")),
    ];
    let view: Vec<&str> = lines.iter().map(String::as_str).collect();
    match top_level_block(&view, "revision_log") {
        Some(b) => {
            let at = b.end.min(lines.len());
            for (k, l) in entry.into_iter().enumerate() {
                lines.insert(at + k, l);
            }
        }
        None => {
            lines.push("revision_log:".to_string());
            lines.extend(entry);
        }
    }

    let mut out = lines.join("\n");
    if ends_with_newline {
        out.push('\n');
    }
    project::atomic_write(&path, out.as_bytes())?;

    let _ = id;
    Ok(Reopened {
        revision,
        reopened,
        allocated,
    })
}
