"""Graders for the implementing-stories skill.

Every function has the signature

    def <name>(workspace: str, transcript: str, args: dict) -> tuple[bool, str]

and reads only files under `workspace` and the `transcript` string. Standard
library only: no network, no subprocess, no randomness, no YAML package. The
reader below covers the flat mappings, nested mappings, and block and flow
sequences this framework writes, which is all these graders parse.
"""

import os
import re


def _artifact_half(workspace, args):
    """The disk-side assertions a Blocked or SEND BACK run has to satisfy.

    A handoff line is text the model authors, so every case that grades one
    pairs it with what the run did or did not write (AUDIT-5 EVL-005):

    * ``forbid_paths`` - paths a run that stopped must not have created;
    * ``unchanged_status`` - a document whose frontmatter ``status`` a stopped
      run leaves where it found it.
    """
    for rel in args.get("forbid_paths", []):
        if os.path.exists(os.path.join(workspace, *rel.split("/"))):
            return False, "%s exists and this run writes none" % rel
    wanted = args.get("unchanged_status")
    if wanted:
        rel = wanted["path"]
        full = os.path.join(workspace, *rel.split("/"))
        if not os.path.isfile(full):
            return False, "%s is absent from the workspace" % rel
        with open(full, "r", encoding="utf-8") as handle:
            head = handle.read()
        match = re.search(r"^status:\s*(\S+)\s*$", head, re.M)
        actual = match.group(1) if match else None
        if actual != wanted["status"]:
            return False, "%s status is %r, the case recorded %r" % (
                rel, actual, wanted["status"])
    return True, "%d forbidden paths absent" % len(args.get("forbid_paths", []))


HEX40_RE = re.compile(r"^[0-9a-f]{40}$")
NOTE_SCHEMA = "devforgeai/build-note/1"


# --------------------------------------------------------------------------
# A small YAML reader
# --------------------------------------------------------------------------


def _strip_comment(text):
    """Drop a trailing ` #` comment that starts outside a quoted run."""
    quote = ""
    for i, ch in enumerate(text):
        if quote:
            if ch == quote:
                quote = ""
        elif ch in "\"'":
            quote = ch
        elif ch == "#" and i > 0 and text[i - 1] in " \t":
            return text[:i]
    return text


def _scalar(text):
    text = text.strip()
    if text.startswith("[") and text.endswith("]"):
        inner = text[1:-1].strip()
        if not inner:
            return []
        return [_scalar(part) for part in inner.split(",")]
    if len(text) >= 2 and text[0] == text[-1] and text[0] in "\"'":
        return text[1:-1]
    if text in ("true", "True"):
        return True
    if text in ("false", "False"):
        return False
    if text in ("null", "~", ""):
        return None
    if re.match(r"^-?\d+$", text) and len(text.lstrip("-")) < 19:
        stripped = text.lstrip("-")
        if stripped == "0" or not stripped.startswith("0"):
            return int(text)
        return text
    if re.match(r"^-?\d+\.\d+$", text):
        return float(text)
    return text


def _lines(text):
    out = []
    for raw in text.splitlines():
        body = _strip_comment(raw.rstrip())
        if not body.strip():
            continue
        if body.strip() in ("---", "..."):
            continue
        indent = len(body) - len(body.lstrip(" "))
        out.append((indent, body.strip()))
    return out


def _parse(lines, i, indent):
    if i >= len(lines) or lines[i][0] < indent:
        return None, i
    if lines[i][1].startswith("- ") or lines[i][1] == "-":
        seq = []
        while i < len(lines) and lines[i][0] == indent and (
            lines[i][1].startswith("- ") or lines[i][1] == "-"
        ):
            rest = lines[i][1][2:] if lines[i][1].startswith("- ") else ""
            child = []
            if rest.strip():
                child.append((indent + 2, rest.strip()))
            i += 1
            while i < len(lines) and lines[i][0] >= indent + 2:
                child.append(lines[i])
                i += 1
            if child:
                value, _ = _parse(child, 0, indent + 2)
            else:
                value = None
            seq.append(value)
        return seq, i
    if ":" not in lines[i][1]:
        return _scalar(lines[i][1]), i + 1
    mapping = {}
    while i < len(lines) and lines[i][0] == indent and not lines[i][1].startswith("- "):
        body = lines[i][1]
        if ":" not in body:
            break
        key, _, rest = body.partition(":")
        key = key.strip()
        rest = rest.strip()
        i += 1
        if rest:
            mapping[key] = _scalar(rest)
            continue
        if i < len(lines) and lines[i][0] > indent:
            mapping[key], i = _parse(lines, i, lines[i][0])
        elif i < len(lines) and lines[i][0] == indent and lines[i][1].startswith("- "):
            mapping[key], i = _parse(lines, i, indent)
        else:
            mapping[key] = None
    return mapping, i


def _load_yaml(text):
    lines = _lines(text)
    if not lines:
        return {}
    value, _ = _parse(lines, 0, lines[0][0])
    return value


def _read(path):
    with open(path, "r", encoding="utf-8", errors="replace") as handle:
        return handle.read()


def _note_path(workspace, story_id):
    return os.path.join(workspace, ".devforgeai", "build", story_id + "-note.yaml")


def _load_note(workspace, story_id):
    path = _note_path(workspace, story_id)
    if not os.path.isfile(path):
        return None, "no note at .devforgeai/build/%s-note.yaml" % story_id
    note = _load_yaml(_read(path))
    if not isinstance(note, dict):
        return None, "the note at %s did not parse as a mapping" % path
    return note, ""


def _cycles(note):
    cycles = note.get("cycles")
    if not isinstance(cycles, list):
        return []
    return [entry for entry in cycles if isinstance(entry, dict)]


def _as_list(value):
    if value is None:
        return []
    if isinstance(value, list):
        return value
    return [value]


# --------------------------------------------------------------------------
# The five graders
# --------------------------------------------------------------------------


def build_note_shape(workspace, transcript, args):
    """The build note names every criterion, in order, with a red and a green."""
    note, problem = _load_note(workspace, args["id"])
    if note is None:
        return False, problem

    if note.get("schema") != NOTE_SCHEMA:
        return False, "schema is %r, expected %r" % (note.get("schema"), NOTE_SCHEMA)
    if note.get("id") != args["id"]:
        return False, "id is %r, expected %r" % (note.get("id"), args["id"])
    if note.get("run") != args["run"]:
        return False, "run is %r, expected %r" % (note.get("run"), args["run"])

    cycles = _cycles(note)
    if len(cycles) < args["min_cycles"]:
        return False, "cycles holds %d entries, expected at least %d" % (
            len(cycles),
            args["min_cycles"],
        )

    found = [entry.get("ac") for entry in cycles]
    if found != args["ac_ids"]:
        return False, "cycles ac order is %r, expected %r" % (found, args["ac_ids"])

    declared = set(args["files"])
    for entry in cycles:
        ac = entry.get("ac")
        paths = _as_list(entry.get("test_paths")) + _as_list(entry.get("source_paths"))
        outside = [path for path in paths if path not in declared]
        if outside:
            return False, "%s names %r, which the ## Files set does not declare" % (
                ac,
                sorted(outside),
            )
        red = entry.get("red_commit") or ""
        green = entry.get("green_commit") or ""
        if not HEX40_RE.match(str(red)):
            return False, "%s red_commit is %r, expected 40 lowercase hex" % (ac, red)
        if not HEX40_RE.match(str(green)):
            return False, "%s green_commit is %r, expected 40 lowercase hex" % (ac, green)
        if red == green:
            return False, "%s carries one sha for red and green: %r" % (ac, red)

    return True, "%d cycles checked, each with a distinct red and green sha" % len(cycles)


def writes_inside_declared_set(workspace, transcript, args):
    """No file outside the story's ## Files table was written."""
    baseline = set(entry.rstrip("/\\") for entry in args["baseline"])
    declared = set(args["files"])
    roots = [(workspace, workspace)]

    worktree_root = os.path.normpath(os.path.join(workspace, args["worktree_root"]))
    story_tree = os.path.join(worktree_root, args["id"])
    if os.path.isdir(story_tree):
        roots.append((story_tree, story_tree))

    collected = []
    for root, base in roots:
        for current, directories, names in os.walk(root):
            directories[:] = [
                name
                for name in directories
                if os.path.normpath(os.path.join(current, name)) != worktree_root
            ]
            for name in names:
                full = os.path.join(current, name)
                relative = os.path.relpath(full, base).replace("\\", "/")
                head = relative.split("/")[0]
                if head in baseline:
                    continue
                collected.append(relative)

    outside = sorted(set(path for path in collected if path not in declared))
    if outside:
        return False, "outside the declared set: %r" % outside
    return True, "%d paths checked, all inside the declared set" % len(collected)


def _handoff_block(transcript):
    lines = transcript.splitlines()
    start = None
    for index, line in enumerate(lines):
        if line.startswith("Phase     "):
            start = index
    if start is None:
        return None
    for index in range(start, len(lines)):
        if lines[index].startswith("Full report: "):
            return lines[start : index + 1]
    return None


def remedy_line(workspace, transcript, args):
    """The SEND BACK handoff cites the right epic, the right ids, and the return trip."""
    wanted = [
        "/plan " + args["epic"] + " --remedy " + ",".join(args["remedy"]),
        "Gate      " + args["gate"],
        "/build " + args["story"] + " --resume",
    ]
    for literal in wanted:
        if literal not in transcript:
            return False, "the transcript does not hold %r" % literal

    for literal in args["forbid_next"]:
        if literal in transcript:
            return False, "the transcript holds the forbidden %r" % literal

    block = _handoff_block(transcript)
    if block is None:
        return False, "no handoff block runs from a 'Phase     ' line to a 'Full report: ' line"
    if len(block) > 12:
        return False, "the handoff block is %d lines:\n%s" % (len(block), "\n".join(block))

    held, why = _artifact_half(workspace, args)
    if not held:
        return False, why

    return True, "handoff block, %d lines, %s:\n%s" % (
        len(block), why, "\n".join(block))


def _invoked_acs(transcript):
    return set(re.findall(r"Agent\(ac-test-writer[^)]*?(AC-\d{3})", transcript))


def criterion_set(workspace, transcript, args):
    """A remedy or resume run works the cited criteria and leaves the rest alone."""
    note, problem = _load_note(workspace, args["id"])
    if note is None:
        return False, problem

    if note.get("run") != args["run"]:
        return False, "run is %r, expected %r" % (note.get("run"), args["run"])

    prior = args.get("prior_cycles", [])
    expected = list(args["expect_acs"])
    if args["run"] == "resume":
        expected = [entry["ac"] for entry in prior] + expected

    cycles = _cycles(note)
    found = [entry.get("ac") for entry in cycles]
    if found != expected:
        return False, "cycles ac list is %r, expected %r" % (found, expected)

    invoked = _invoked_acs(transcript)
    forbidden = sorted(invoked.intersection(args["forbid_acs"]))
    if forbidden:
        return False, "ac-test-writer was invoked for %r, which this run leaves alone" % forbidden

    remedy = _as_list(note.get("remedy"))
    if remedy != args.get("remedy", []):
        return False, "remedy is %r, expected %r" % (remedy, args.get("remedy", []))

    resumed = note.get("resumed_at") or ""
    if resumed != args.get("resumed_at", ""):
        return False, "resumed_at is %r, expected %r" % (resumed, args.get("resumed_at", ""))

    by_ac = dict((entry.get("ac"), entry) for entry in cycles)
    for entry in prior:
        held = by_ac.get(entry["ac"])
        if held is None:
            return False, "the note dropped the earlier cycle %r" % entry["ac"]
        if (held.get("green_commit") or "") != entry["green_commit"]:
            return False, "%s green_commit is %r, expected the earlier %r" % (
                entry["ac"],
                held.get("green_commit"),
                entry["green_commit"],
            )

    return True, "cycles %r against expected %r" % (found, expected)


def blocked_line(workspace, transcript, args):
    """The run stopped on one Blocked line naming the condition and the way out."""
    needed = list(args["names"])
    if args["code"]:
        needed.append(args["code"])

    matched = None
    for line in transcript.splitlines():
        if not line.startswith("Blocked   you: "):
            continue
        if all(name in line for name in needed):
            matched = line
            break
    if matched is None:
        return False, "no 'Blocked   you: ' line holds all of %r" % needed

    for literal in args["expect_calls"]:
        if literal not in transcript:
            return False, "the transcript does not hold %r" % literal

    for literal in args["forbid_calls"]:
        if literal in transcript:
            return False, "the transcript holds the forbidden %r" % literal

    held, why = _artifact_half(workspace, args)
    if not held:
        return False, why

    return True, "%s | %s" % (matched, why)
