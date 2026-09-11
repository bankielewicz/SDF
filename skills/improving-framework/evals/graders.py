"""Graders for the improving-framework skill.

Every function has the signature

    def <name>(workspace: str, transcript: str, args: dict) -> tuple[bool, str]

and reads only files under `workspace` and the `transcript` string. Standard
library only: no network, no subprocess, no randomness, no YAML package.

The reflect report and the fixtures it is built from use block mappings, block
sequences, plain and quoted scalars, one flow sequence per `consumes` and
`observations` list, and one folded scalar per `detail` and `change` field. The
reader below covers exactly those shapes, which keeps the module free of a
third-party import and the eval harness free of an install step.
"""

import datetime
import os
import re


def _reflect_document(workspace, args):
    """The reflect document the run is supposed to have written.

    The artifact half of the handoff assertion (AUDIT-5 EVL-005): the block a
    model prints costs nothing, and `.devforgeai/reports/reflect-<date>.yaml` at
    `status: final` with at least one OBS and one REC costs the work.
    """
    if not args.get("requires_reflect_document"):
        return True, "no document required"
    root = os.path.join(workspace, ".devforgeai", "reports")
    names = ([n for n in sorted(os.listdir(root))
              if n.startswith("reflect-") and n.endswith(".yaml")]
             if os.path.isdir(root) else [])
    if len(names) != 1:
        return False, ("reports/ holds %d reflect-<date>.yaml documents, "
                       "expected exactly 1" % len(names))
    with open(os.path.join(root, names[0]), "r", encoding="utf-8") as handle:
        body = handle.read()
    if not re.search(r"^status:\s*final\s*$", body, re.M):
        return False, "%s is not at status: final" % names[0]
    for prefix in ("OBS-", "REC-"):
        if prefix not in body:
            return False, "%s names no %snnn" % (names[0], prefix)
    return True, "%s written at status final" % names[0]


REPORT_RE = re.compile(r"^reflect-[0-9]{4}-[0-9]{2}-[0-9]{2}\.yaml$")
KEY_RE = re.compile(r"^([^\s:][^:]*):(?:\s|$)")
INT_RE = re.compile(r"^-?[0-9]+$")
FLOAT_RE = re.compile(r"^-?[0-9]+\.[0-9]+$")
SEVERITIES = {"low", "medium", "high"}
LABEL_WIDTH = 10


class _Fail(Exception):
    """Raised with the evidence line a failing grader returns."""


# ---------------------------------------------------------------- primitives


def _path(workspace, rel):
    parts = [p for p in rel.replace("\\", "/").split("/") if p]
    return os.path.join(workspace, *parts)


def _read(path):
    with open(path, "r", encoding="utf-8") as handle:
        return handle.read()


def _indent(line):
    return len(line) - len(line.lstrip(" "))


def _split_flow(inner):
    """Split a flow sequence on commas that sit outside a quoted entry.

    An `open_questions` drop line carries a comma of its own — `REC dropped:
    <key> proposed <value>, floor <value>` — so a naive split would cut one
    entry into two.
    """
    parts, buf, quote = [], [], ""
    for char in inner:
        if quote:
            buf.append(char)
            if char == quote:
                quote = ""
        elif char in "\"'":
            quote = char
            buf.append(char)
        elif char == ",":
            parts.append("".join(buf))
            buf = []
        else:
            buf.append(char)
    parts.append("".join(buf))
    return [part for part in parts if part.strip() != "" or len(parts) == 1]


def _scalar(raw):
    raw = raw.strip()
    if raw.startswith("[") and raw.endswith("]"):
        inner = raw[1:-1].strip()
        if not inner:
            return []
        return [_scalar(part) for part in _split_flow(inner)]
    if raw.startswith("{") and raw.endswith("}"):
        return {}
    if len(raw) >= 2 and raw[0] == raw[-1] and raw[0] in "\"'":
        return raw[1:-1]
    if raw in ("null", "~", ""):
        return None
    if raw == "true":
        return True
    if raw == "false":
        return False
    if INT_RE.match(raw):
        return int(raw)
    if FLOAT_RE.match(raw):
        return float(raw)
    return raw


def _clean(text):
    """Drop blank lines, whole-line comments, and document markers."""
    out = []
    for line in text.replace("\r\n", "\n").split("\n"):
        stripped = line.strip()
        if not stripped:
            continue
        if stripped.startswith("#"):
            continue
        if stripped in ("---", "..."):
            continue
        out.append(line.rstrip())
    return out


def _is_item(line):
    stripped = line.strip()
    return stripped == "-" or stripped.startswith("- ")


def _parse(lines, i, indent):
    if i >= len(lines):
        return None, i
    if _is_item(lines[i]):
        return _parse_seq(lines, i, indent)
    return _parse_map(lines, i, indent)


def _parse_seq(lines, i, indent):
    items = []
    while i < len(lines):
        line = lines[i]
        if _indent(line) != indent or not _is_item(line):
            break
        rest = line[indent + 1:]
        content = rest.strip()
        if not content:
            if i + 1 >= len(lines):
                items.append(None)
                return items, i + 1
            value, i = _parse(lines, i + 1, _indent(lines[i + 1]))
            items.append(value)
            continue
        column = indent + 1 + (len(rest) - len(rest.lstrip(" ")))
        lines[i] = " " * column + content
        if KEY_RE.match(content):
            value, i = _parse_map(lines, i, column)
        else:
            value, i = _scalar(content), i + 1
        items.append(value)
    return items, i


def _parse_map(lines, i, indent):
    out = {}
    while i < len(lines):
        line = lines[i]
        if _indent(line) != indent or _is_item(line):
            break
        match = KEY_RE.match(line.strip())
        if not match:
            break
        key = match.group(1).strip()
        rest = line.strip()[len(match.group(1)) + 1:].strip()
        if rest in (">", "|", ">-", "|-", ">+", "|+"):
            i += 1
            buf = []
            while i < len(lines) and _indent(lines[i]) > indent:
                buf.append(lines[i].strip())
                i += 1
            out[key] = " ".join(buf)
        elif rest:
            out[key] = _scalar(rest)
            i += 1
        elif i + 1 < len(lines) and (
            _indent(lines[i + 1]) > indent
            or (_indent(lines[i + 1]) == indent and _is_item(lines[i + 1]))
        ):
            out[key], i = _parse(lines, i + 1, _indent(lines[i + 1]))
        else:
            out[key] = None
            i += 1
    return out, i


def _yaml(text):
    lines = _clean(text)
    if not lines:
        return {}
    value, _ = _parse(lines, 0, _indent(lines[0]))
    return value


def _load(workspace, args):
    directory = _path(workspace, args["dir"])
    if not os.path.isdir(directory):
        raise _Fail("no directory at %s" % args["dir"])
    names = sorted(n for n in os.listdir(directory) if REPORT_RE.match(n))
    if len(names) != 1:
        raise _Fail("expected one reflect-<date>.yaml in %s, found %s"
                    % (args["dir"], names or "none"))
    document = _yaml(_read(os.path.join(directory, names[0])))
    if not isinstance(document, dict):
        raise _Fail("%s did not parse as a mapping" % names[0])
    return names[0], document


def _seq(document, key):
    value = document.get(key)
    if value is None:
        return []
    if not isinstance(value, list):
        raise _Fail("%s is not a sequence" % key)
    return value


def _gate_word(transcript, expected):
    for line in transcript.replace("\r\n", "\n").split("\n"):
        if line.startswith("Gate" + " " * 6):
            found = line[LABEL_WIDTH:].strip().split(" ")[0]
            if found != expected:
                raise _Fail("Gate line reads %r against the expected %r"
                            % (found, expected))
            return found
    raise _Fail("the transcript holds no line starting 'Gate'")


def _blocks(transcript):
    lines = transcript.replace("\r\n", "\n").split("\n")
    found = []
    for index, line in enumerate(lines):
        if line.startswith("Phase" + " " * 5):
            block = []
            for candidate in lines[index:]:
                block.append(candidate)
                if candidate.startswith("Full report:"):
                    break
            found.append(block)
    return found


def _reflect_block(transcript):
    """The run's own block.

    A `/reflect` run prints two blocks: this skill's, then the one the Stop
    hook renders for the phase the user was in. The `Phase` line of the first
    names Reflect, so the block is selected by that name and falls back to the
    last block when no line names it.
    """
    found = _blocks(transcript)
    if not found:
        raise _Fail("the transcript holds no block starting 'Phase'")
    named = [block for block in found if "Reflect" in block[0]]
    return named[-1] if named else found[-1]


def _grader(function):
    def wrapper(workspace, transcript, args):
        try:
            return function(workspace, transcript, args)
        except _Fail as failure:
            return False, str(failure)
    wrapper.__name__ = function.__name__
    wrapper.__doc__ = function.__doc__
    return wrapper


# ------------------------------------------------------------------ graders


@_grader
def reflect_report_shape(workspace, transcript, args):
    """The twelve top-level keys in order, the sessions block, the obs kinds."""
    name, document = _load(workspace, args)
    keys = list(document.keys())
    if keys != args["keys"]:
        raise _Fail("%s holds keys %s against the expected order %s"
                    % (name, keys, args["keys"]))
    if document.get("status") != args["status"]:
        raise _Fail("status is %r against the expected %r"
                    % (document.get("status"), args["status"]))
    sessions = (document.get("sources") or {}).get("sessions") or {}
    if sessions.get("status") != args["sessions_status"]:
        raise _Fail("sources.sessions.status is %r against the expected %r"
                    % (sessions.get("status"), args["sessions_status"]))
    if args["sessions_status"] != "present":
        files = sessions.get("files")
        if files not in ([], None):
            raise _Fail("sources.sessions.files holds %s at status %s"
                        % (files, args["sessions_status"]))
    kinds = {}
    for entry in _seq(document, "observations"):
        kind = entry.get("kind")
        if kind not in args["obs_kinds"]:
            raise _Fail("%s carries kind %r, outside %s"
                        % (entry.get("id"), kind, args["obs_kinds"]))
        if entry.get("severity") not in SEVERITIES:
            raise _Fail("%s carries severity %r, outside %s"
                        % (entry.get("id"), entry.get("severity"),
                           sorted(SEVERITIES)))
        kinds[kind] = kinds.get(kind, 0) + 1
        for source in entry.get("sources") or []:
            if isinstance(source, str) and source.startswith(".devforgeai/"):
                if not os.path.exists(_path(workspace, source)):
                    raise _Fail("%s cites %s, which the workspace does not hold"
                                % (entry.get("id"), source))
    gate = _gate_word(transcript, args["gate"])
    return True, ("%s keys in order; sessions.status %s; kinds %s; Gate %s"
                  % (name, sessions.get("status"), sorted(kinds.items()), gate))


@_grader
def rec_cites_obs(workspace, transcript, args):
    """Every REC cites an OBS of the same document and names one target kind."""
    name, document = _load(workspace, args)
    ids = {entry.get("id") for entry in _seq(document, "observations")}
    mapping = []
    for entry in _seq(document, "recommendations"):
        cited = entry.get("observations") or []
        if isinstance(cited, str):
            cited = [cited]
        if len(cited) < args["min_obs_per_rec"]:
            raise _Fail("%s cites %d of %d required observations"
                        % (entry.get("id"), len(cited), args["min_obs_per_rec"]))
        for obs in cited:
            if obs not in ids:
                raise _Fail("%s cites %s, which this document does not define"
                            % (entry.get("id"), obs))
        target = entry.get("target") or {}
        kind = target.get("kind")
        if kind not in args["target_kinds"]:
            raise _Fail("%s carries target.kind %r, outside %s"
                        % (entry.get("id"), kind, args["target_kinds"]))
        path = target.get("path") or ""
        under = any(path.startswith(prefix) for prefix in args["threshold_prefixes"])
        if kind == "gate_threshold" and not under:
            raise _Fail("%s is a gate_threshold naming %s, outside %s"
                        % (entry.get("id"), path, args["threshold_prefixes"]))
        if kind != "gate_threshold" and under:
            raise _Fail("%s is a %s naming %s, which only a gate_threshold names"
                        % (entry.get("id"), kind, path))
        mapping.append("%s -> %s" % (entry.get("id"), ",".join(cited)))
    for entry in _seq(document, "observations"):
        if not (entry.get("sources") or []):
            raise _Fail("%s carries an empty sources list" % entry.get("id"))
    return True, "%s: %s" % (name, "; ".join(mapping) or "no recommendations")


@_grader
def obs_names_pattern(workspace, transcript, args):
    """One observation of the asked-for kind names the pattern and its lines."""
    name, document = _load(workspace, args)
    selected = []
    for entry in _seq(document, "observations"):
        if entry.get("kind") != args["kind"]:
            continue
        if "phase" in args and entry.get("phase") != args["phase"]:
            continue
        selected.append(entry)
    if not selected:
        raise _Fail("%s holds no observation of kind %s%s"
                    % (name, args["kind"],
                       " in phase " + args["phase"] if "phase" in args else ""))
    entry = max(selected, key=lambda item: item.get("count") or 0)
    if entry.get("count") != args["count"]:
        raise _Fail("%s carries count %s against the expected %s"
                    % (entry.get("id"), entry.get("count"), args["count"]))
    summary = entry.get("summary") or ""
    for needle in args["summary_contains"]:
        if needle not in summary:
            raise _Fail("%s summary %r does not name %s"
                        % (entry.get("id"), summary, needle))
    sources = set(entry.get("sources") or [])
    missing = set(args["session_ids"]) - sources
    if missing:
        raise _Fail("%s cites %s and leaves out %s"
                    % (entry.get("id"), sorted(sources), sorted(missing)))
    lines = sorted((item or {}).get("line") for item in entry.get("evidence") or [])
    if lines != sorted(args["evidence_lines"]):
        raise _Fail("%s cites lines %s against the expected %s"
                    % (entry.get("id"), lines, sorted(args["evidence_lines"])))
    sessions = (document.get("sources") or {}).get("sessions") or {}
    if sessions.get("status") != args["sessions_status"]:
        raise _Fail("sources.sessions.status is %r against the expected %r"
                    % (sessions.get("status"), args["sessions_status"]))
    if args.get("cited_by_rec"):
        kinds = args.get("rec_target_kinds") or []
        citing = [rec for rec in _seq(document, "recommendations")
                  if entry.get("id") in (rec.get("observations") or [])
                  and (rec.get("target") or {}).get("kind") in kinds]
        if not citing:
            raise _Fail("no recommendation with a target kind in %s cites %s"
                        % (kinds, entry.get("id")))
    return True, ("%s %s count %s summary %r lines %s"
                  % (entry.get("id"), entry.get("kind"), entry.get("count"),
                     summary, lines))


@_grader
def debt_groups(workspace, transcript, args):
    """The debt section groups by constraint, in order, with ages in days."""
    name, document = _load(workspace, args)
    debt = document.get("technical_debt") or {}
    groups = debt.get("groups") or []
    found = [[group.get("constraint"), group.get("kind"), group.get("count")]
             for group in groups]
    expected = [list(row) for row in args["expected_groups"]]
    if found != expected:
        raise _Fail("%s groups %s against the expected %s"
                    % (name, found, expected))
    if debt.get("total") != args["total"]:
        raise _Fail("technical_debt.total is %s against the expected %s"
                    % (debt.get("total"), args["total"]))
    if debt.get("oldest_days") != args["oldest_days"]:
        raise _Fail("technical_debt.oldest_days is %s against the expected %s"
                    % (debt.get("oldest_days"), args["oldest_days"]))
    as_of = datetime.date.fromisoformat(str(args["as_of"]))
    ages = []
    for group in groups:
        for row in group.get("items") or []:
            story = row.get("story")
            if story not in args["deferred_at"]:
                raise _Fail("%s names %s, which the case records no date for"
                            % (name, story))
            recorded = str(args["deferred_at"][story])
            if str(row.get("deferred_at")) != recorded:
                raise _Fail("%s carries deferred_at %s against the recorded %s"
                            % (story, row.get("deferred_at"), recorded))
            computed = (as_of - datetime.date.fromisoformat(recorded)).days
            if row.get("age_days") != computed:
                raise _Fail("%s carries age_days %s against the computed %s"
                            % (story, row.get("age_days"), computed))
            ages.append("%s %s=%d" % (story, recorded, computed))
    return True, "%s groups %s; ages %s" % (name, found, ", ".join(ages))


@_grader
def no_lowered_floor(workspace, transcript, args):
    """No threshold recommendation proposes a value below its compiled floor."""
    name, document = _load(workspace, args)
    floors = {key: float(value) for key, value in args["floors"].items()}
    seen = []
    for entry in _seq(document, "recommendations"):
        target = entry.get("target") or {}
        if target.get("kind") != "gate_threshold":
            continue
        key = target.get("key") or ""
        if key not in floors:
            continue
        raw = str(entry.get("proposed_value"))
        try:
            proposed = float(raw)
        except ValueError:
            raise _Fail("%s proposes %r for %s, which does not parse as a number"
                        % (entry.get("id"), raw, key))
        if proposed < floors[key]:
            raise _Fail("%s proposes %s = %s, below the compiled floor %s"
                        % (entry.get("id"), key, proposed, floors[key]))
        seen.append("%s %s=%s floor %s" % (entry.get("id"), key, proposed,
                                           floors[key]))
    for line in _seq(document, "open_questions"):
        text = str(line)
        for key, floor in floors.items():
            if key not in text:
                continue
            numbers = [float(n) for n in re.findall(r"[0-9]+(?:\.[0-9]+)?", text)]
            dropped = [n for n in numbers if n != floor]
            if dropped and max(dropped) >= floor:
                raise _Fail("open_questions names %s at %s, at or above the "
                            "floor %s, so the proposal was not a drop"
                            % (key, max(dropped), floor))
    gate = _gate_word(transcript, args["gate"])
    return True, ("%s: %s; Gate %s"
                  % (name, "; ".join(seen) or "no threshold recommendation", gate))


@_grader
def handoff_returns(workspace, transcript, args):
    """The handoff block returns the user to the phase they were in."""
    block = _reflect_block(transcript)
    if len(block) > args["max_lines"]:
        raise _Fail("the block holds %d lines against the cap of %d"
                    % (len(block), args["max_lines"]))
    values = {}
    for line in block:
        for label in ("Gate", "Next", "Then"):
            if line.startswith(label) and line[:LABEL_WIDTH].strip() == label:
                values[label] = line[LABEL_WIDTH:].rstrip()
    if values.get("Gate", "").split(" ")[0] != args["gate"]:
        raise _Fail("Gate line reads %r against the expected %r"
                    % (values.get("Gate"), args["gate"]))
    if values.get("Next", "").rstrip() != args["next"]:
        raise _Fail("Next line reads %r against the expected %r"
                    % (values.get("Next"), args["next"]))
    if values.get("Then", "").rstrip() != args["then"]:
        raise _Fail("Then line reads %r against the expected %r"
                    % (values.get("Then"), args["then"]))
    for line in block:
        for label in args["absent_labels"]:
            if line.startswith(label):
                raise _Fail("the block holds a %s line: %r" % (label, line))
    held, why = _reflect_document(workspace, args)
    if not held:
        raise _Fail(why)
    return True, "%d lines; Next %r; Then %r; %s" % (
        len(block), values.get("Next"), values.get("Then"), why)
