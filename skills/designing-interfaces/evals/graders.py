"""Graders for the designing-interfaces skill.

Every function has the signature

    def <name>(workspace: str, transcript: str, args: dict) -> tuple[bool, str]

and reads only files under `workspace` and the `transcript` string. Standard
library only: no network, no subprocess, no randomness, no YAML package. The
frontmatter reader below covers the flat, seven-key documents this framework
writes, which is all these graders parse.
"""

import hashlib
import json
import os
import re


BRAND_SKETCH_KEYS = ["name", "palette", "type_pair"]


def _brand_sketch_flat(workspace, rel, problems):
    """`brand-sketch.json` is one flat object holding exactly three keys.

    It is the one JSON this skill writes with no envelope: it lives under
    `.devforgeai/explore/mockups/`, outside document validation, and SKILL.md
    step 4 fixes its three keys. A wrapper object around them is the failure the
    ex-08 run showed on the sketch request, caught here before a reader misses
    the palette.
    """
    if not os.path.isfile(_path(workspace, rel)):
        problems.append("%s is absent" % rel)
        return
    try:
        value = _load_json(workspace, rel)
    except (OSError, ValueError) as error:
        problems.append("%s is not readable JSON: %s" % (rel, error))
        return
    if not isinstance(value, dict):
        problems.append("%s holds a %s, expected one flat object"
                        % (rel, type(value).__name__))
        return
    keys = list(value)
    if sorted(keys) != sorted(BRAND_SKETCH_KEYS):
        problems.append("%s holds keys %s, expected exactly %s at the top level"
                        % (rel, ", ".join(keys), ", ".join(BRAND_SKETCH_KEYS)))
        return
    palette = value.get("palette")
    if not isinstance(palette, list) or not 3 <= len(palette) <= 6 or \
            not all(isinstance(one, str) for one in palette):
        problems.append("%s palette is %r, expected 3 to 6 hex strings"
                        % (rel, palette))
    pair = value.get("type_pair")
    if not isinstance(pair, list) or len(pair) != 2 or \
            not all(isinstance(one, str) for one in pair):
        problems.append("%s type_pair is %r, expected two family strings"
                        % (rel, pair))
    if not isinstance(value.get("name"), str) or not value["name"].strip():
        problems.append("%s name is %r" % (rel, value.get("name")))



def _phase_report_not_pass(workspace, phase):
    """No gate report for ``phase`` reads ``result: PASS``.

    The artifact half of a send-back assertion (AUDIT-5 EVL-005): a send-back
    leaves the phase's report absent or carrying SEND BACK, so a model that
    prints the handoff block without doing the work fails here even when every
    transcript line it typed is correct.
    """
    if not phase:
        return True, "no report phase named"
    root = os.path.join(workspace, ".devforgeai", "reports")
    if not os.path.isdir(root):
        return True, "no reports directory, so no %s report reads PASS" % phase
    passing = []
    for name in sorted(os.listdir(root)):
        if not name.endswith("-%s.yaml" % phase):
            continue
        try:
            with open(os.path.join(root, name), "r", encoding="utf-8") as handle:
                text = handle.read()
        except OSError:
            continue
        if re.search(r"^\s*result:\s*[\"']?PASS[\"']?\s*$", text, re.M):
            passing.append(name)
    if passing:
        return False, ("%s reads gate result PASS; a send-back leaves the %s "
                       "report absent or at SEND BACK" % (", ".join(passing), phase))
    return True, "no %s report reads PASS" % phase


META_KEYS = ["schema", "id", "phase", "status", "produced_by", "consumes", "open_questions"]

LEAF_RE = re.compile(r"^[a-z][a-z0-9-]*$")
HEX_RE = re.compile(r"^#[0-9a-f]{6}$")
UI_FILE_RE = re.compile(r"^UI-[0-9]{3}\.md$")
TOKEN_RE = re.compile(r"TOKEN-[a-z0-9-]+")

LITERAL_RULES = [
    ("literal colour", re.compile(r"#[0-9a-fA-F]{3,8}\b")),
    ("rgb literal", re.compile(r"rgba?\(")),
    ("hsl literal", re.compile(r"hsla?\(")),
    ("literal length", re.compile(r"[0-9.]+(px|rem|em|pt)")),
]


# ---------------------------------------------------------------- primitives


def _path(workspace, rel):
    return os.path.join(workspace, *[p for p in rel.replace("\\", "/").split("/") if p])


def _read(workspace, rel):
    with open(_path(workspace, rel), "r", encoding="utf-8") as handle:
        return handle.read()


def _load_json(workspace, rel):
    return json.loads(_read(workspace, rel))


def _scalar(raw):
    raw = raw.strip()
    if raw.startswith("[") and raw.endswith("]"):
        inner = raw[1:-1].strip()
        if not inner:
            return []
        return [_scalar(part) for part in inner.split(",")]
    if len(raw) >= 2 and raw[0] == raw[-1] and raw[0] in "\"'":
        return raw[1:-1]
    return raw


def _frontmatter(text):
    """Return (ordered key list, mapping, body) for a document with frontmatter."""
    lines = text.split("\n")
    if not lines or lines[0].strip() != "---":
        return [], {}, text
    keys, values = [], {}
    index = 1
    while index < len(lines) and lines[index].strip() != "---":
        line = lines[index]
        if line.strip() and not line.startswith((" ", "\t", "#")) and ":" in line:
            key, _, rest = line.partition(":")
            key = key.strip()
            keys.append(key)
            values[key] = _scalar(rest)
        index += 1
    body = "\n".join(lines[index + 1:]) if index < len(lines) else ""
    return keys, values, body


def _sections(body):
    """Return [(heading, section text)] split on lines starting with '## '."""
    out = []
    heading, buffer = None, []
    for line in body.split("\n"):
        if line.startswith("## "):
            if heading is not None:
                out.append((heading, "\n".join(buffer)))
            heading, buffer = line[3:].strip(), []
        elif heading is not None:
            buffer.append(line)
    if heading is not None:
        out.append((heading, "\n".join(buffer)))
    return out


def _cells(line):
    stripped = line.strip()
    return [cell.strip() for cell in stripped.strip("|").split("|")]


def _is_table_line(line):
    stripped = line.strip()
    return stripped.startswith("|") and stripped.endswith("|") and len(stripped) > 1


def _is_separator(line):
    return bool(re.match(r"^\|[\s:|-]+\|$", line.strip()))


def _table(section_text):
    """Return (header cells, [data row cells]) for the first table in a section."""
    header, rows, seen_header = [], [], False
    for line in section_text.split("\n"):
        if not _is_table_line(line):
            if seen_header and rows:
                break
            continue
        if _is_separator(line):
            continue
        if not seen_header:
            header = _cells(line)
            seen_header = True
        else:
            rows.append(_cells(line))
    return header, rows


def _flatten_tokens(obj):
    names = set()
    for group, leaves in obj.items():
        if group == "meta" or not isinstance(leaves, dict):
            continue
        for leaf in leaves:
            names.add("TOKEN-%s-%s" % (group, leaf))
    return names


def _section_digest(section_text):
    """SHA-256 of a section body.

    The body is the text below the '## ' heading line and above the next one,
    with trailing whitespace stripped from every line, leading and trailing
    blank lines dropped, lines joined with a single newline, encoded UTF-8,
    with no trailing newline. `evals/fixtures/digests.txt` records the same
    recipe, and the recorded digests were produced by this function.
    """
    lines = [line.rstrip() for line in section_text.split("\n")]
    while lines and not lines[0]:
        lines.pop(0)
    while lines and not lines[-1]:
        lines.pop()
    return hashlib.sha256("\n".join(lines).encode("utf-8")).hexdigest()


def _ui_files(workspace, rel_dir):
    directory = _path(workspace, rel_dir)
    if not os.path.isdir(directory):
        return []
    return sorted(name for name in os.listdir(directory) if UI_FILE_RE.match(name))


def _last_json_object_with_key(text, key):
    decoder = json.JSONDecoder()
    found = None
    for index, char in enumerate(text):
        if char != "{":
            continue
        try:
            obj, _ = decoder.raw_decode(text[index:])
        except ValueError:
            continue
        if isinstance(obj, dict) and key in obj:
            found = obj
    return found


# ------------------------------------------------------------------ graders


def tokens_shape(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    try:
        obj = _load_json(workspace, args["path"])
    except (OSError, ValueError) as exc:
        return False, "%s is not readable JSON: %s" % (args["path"], exc)

    problems = []
    groups = list(obj.keys())
    if groups != args["groups"]:
        problems.append("top-level keys %s, expected %s" % (groups, args["groups"]))

    meta = obj.get("meta")
    if not isinstance(meta, dict):
        problems.append("meta is %s, expected an object" % type(meta).__name__)
    else:
        if list(meta.keys()) != META_KEYS:
            problems.append("meta keys %s, expected %s" % (list(meta.keys()), META_KEYS))
        if meta.get("status") != args["status"]:
            problems.append("meta.status %r, expected %r" % (meta.get("status"), args["status"]))

    counts = {}
    for group, expected in args["leaf_counts"].items():
        leaves = obj.get(group)
        if not isinstance(leaves, dict):
            problems.append("group %s is %s, expected an object" % (group, type(leaves).__name__))
            continue
        counts[group] = len(leaves)
        if len(leaves) != expected:
            problems.append("group %s holds %d leaves, expected %d" % (group, len(leaves), expected))
        for leaf, value in leaves.items():
            if not LEAF_RE.match(leaf):
                problems.append("leaf %s.%s does not match ^[a-z][a-z0-9-]*$" % (group, leaf))
            if group == "color":
                if not isinstance(value, dict) or set(value.keys()) != set(args["color_keys"]):
                    problems.append("color.%s keys %s, expected %s"
                                    % (leaf, sorted(value.keys()) if isinstance(value, dict) else value,
                                       sorted(args["color_keys"])))
                    continue
                for theme, hexval in value.items():
                    if not isinstance(hexval, str) or not HEX_RE.match(hexval):
                        problems.append("color.%s.%s is %r, expected ^#[0-9a-f]{6}$" % (leaf, theme, hexval))
            elif not isinstance(value, str):
                problems.append("%s.%s is %s, expected a string" % (group, leaf, type(value).__name__))

    evidence = "top-level keys %s; leaf counts %s" % (groups, counts)
    if problems:
        return False, evidence + "; " + "; ".join(problems)
    return True, evidence


def brand_kit_shape(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    try:
        text = _read(workspace, args["path"])
    except OSError as exc:
        return False, "%s is not readable: %s" % (args["path"], exc)

    _, _, body = _frontmatter(text)
    sections = _sections(body)
    headings = [heading for heading, _ in sections]
    by_heading = dict(sections)
    problems = []

    if headings != args["headings"]:
        problems.append("headings %s, expected %s" % (headings, args["headings"]))

    counts = {}
    for heading, expected in args["row_counts"].items():
        _, rows = _table(by_heading.get(heading, ""))
        counts[heading] = len(rows)
        if len(rows) != expected:
            problems.append("%s holds %d rows, expected %d" % (heading, len(rows), expected))

    _, figma_rows = _table(by_heading.get("Figma", ""))
    figma_first = [row[0] for row in figma_rows if row]
    if figma_first != args["figma_rows"]:
        problems.append("Figma rows %s, expected %s" % (figma_first, args["figma_rows"]))

    evidence = "headings %s; row counts %s; figma rows %s" % (headings, counts, figma_first)
    if problems:
        return False, evidence + "; " + "; ".join(problems)
    return True, evidence


def figma_fallback(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    try:
        text = _read(workspace, args["path"])
    except OSError as exc:
        return False, "%s is not readable: %s" % (args["path"], exc)

    _, _, body = _frontmatter(text)
    by_heading = dict(_sections(body))
    _, rows = _table(by_heading.get("Figma", ""))
    figma = {row[0]: row[1] for row in rows if len(row) >= 2}
    problems = []

    if figma.get("Status") != args["status"]:
        problems.append("Figma Status %r, expected %r" % (figma.get("Status"), args["status"]))
    if figma.get("Reason") not in args["reasons"]:
        problems.append("Figma Reason %r, expected one of %s" % (figma.get("Reason"), args["reasons"]))

    checked = []
    for rel in args["present"]:
        full = _path(workspace, rel)
        size = os.path.getsize(full) if os.path.isfile(full) else -1
        checked.append("%s:%d" % (rel, size))
        if size <= 0:
            problems.append("%s is absent or empty" % rel)

    hits = transcript.count("Gate      PASS")
    if hits < 1:
        problems.append("the transcript holds no 'Gate      PASS' line")

    evidence = "Figma %s; paths %s; PASS lines %d" % (figma, checked, hits)
    if problems:
        return False, evidence + "; " + "; ".join(problems)
    return True, evidence


def ui_spec_shape(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    names = _ui_files(workspace, args["dir"])
    problems = []
    if len(names) < args["min_files"]:
        problems.append("%s holds %d UI specs, expected at least %d"
                        % (args["dir"], len(names), args["min_files"]))

    seen = {}
    for name in names:
        rel = args["dir"].rstrip("/") + "/" + name
        try:
            text = _read(workspace, rel)
        except OSError as exc:
            problems.append("%s is not readable: %s" % (rel, exc))
            continue
        keys, _, body = _frontmatter(text)
        if keys != META_KEYS:
            problems.append("%s frontmatter keys %s, expected %s" % (name, keys, META_KEYS))
        sections = _sections(body)
        headings = [heading for heading, _ in sections]
        seen[name] = headings
        by_heading = dict(sections)
        if headings != args["headings"]:
            problems.append("%s headings %s, expected %s" % (name, headings, args["headings"]))

        _, bp_rows = _table(by_heading.get("Breakpoints", ""))
        bp = [[row[0], row[1]] for row in bp_rows if len(row) >= 2]
        if bp != [list(pair) for pair in args["breakpoints"]]:
            problems.append("%s breakpoints %s, expected %s" % (name, bp, args["breakpoints"]))

        _, a11y_rows = _table(by_heading.get("Accessibility", ""))
        a11y = [row[0] for row in a11y_rows if row]
        if a11y != args["a11y_checks"]:
            problems.append("%s accessibility checks %s, expected %s" % (name, a11y, args["a11y_checks"]))

        _, state_rows = _table(by_heading.get("States", ""))
        if not any(row and row[0] == "default" for row in state_rows):
            problems.append("%s ## States holds no default row" % name)

    evidence = "files %s; headings %s" % (names, seen)
    if problems:
        return False, evidence + "; " + "; ".join(problems)
    return True, evidence


def tokens_only(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    try:
        defined = _flatten_tokens(_load_json(workspace, args["tokens"]))
    except (OSError, ValueError) as exc:
        return False, "%s is not readable JSON: %s" % (args["tokens"], exc)

    names = _ui_files(workspace, args["dir"])
    if not names:
        return False, "%s holds no UI spec" % args["dir"]

    exempt = args.get("exempt_columns", {})
    problems = []
    resolved = 0

    for name in names:
        rel = args["dir"].rstrip("/") + "/" + name
        try:
            text = _read(workspace, rel)
        except OSError as exc:
            problems.append("%s is not readable: %s" % (rel, exc))
            continue
        _, _, body = _frontmatter(text)
        body_lines = body.split("\n")

        heading = None
        drop = []
        for number, line in enumerate(body_lines, start=1):
            if line.startswith("## "):
                heading = line[3:].strip()
                drop = []
                continue
            checked = line
            if heading in exempt and _is_table_line(line):
                cells = _cells(line)
                if not drop and not _is_separator(line):
                    drop = [i for i, cell in enumerate(cells) if cell in exempt[heading]]
                checked = " | ".join(cell for i, cell in enumerate(cells) if i not in drop)
            for label, pattern in LITERAL_RULES:
                if pattern.search(checked):
                    problems.append("%s:%d %s in %r" % (name, number, label, checked.strip()))

        for token in TOKEN_RE.findall(body):
            if token in defined:
                resolved += 1
            else:
                problems.append("%s references %s, which %s does not define"
                                % (name, token, args["tokens"]))

    evidence = "%d specs; %d token references resolved against %d defined names" \
               % (len(names), resolved, len(defined))
    if problems:
        return False, evidence + "; " + "; ".join(problems[:12])
    return True, evidence


def sendback_block(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    lines = transcript.split("\n")
    problems = []

    gate_prefix = "Gate      SEND BACK to " + args["to"]
    gate_line = next((line for line in lines if line.strip().startswith(gate_prefix)), None)
    if gate_line is None:
        problems.append("no line starting %r" % gate_prefix)

    next_line = next((line for line in lines if line.startswith("Next      ")), None)
    next_text = next_line[len("Next      "):].strip() if next_line else ""
    if next_line is None:
        problems.append("no line starting 'Next      '")
    elif not next_text.startswith(args["next_prefix"]):
        problems.append("Next line %r does not start with %r" % (next_text, args["next_prefix"]))
    if next_line is not None:
        for needle in args["must_cite"]:
            if needle not in next_text:
                problems.append("Next line %r does not cite %r" % (next_text, needle))
        for needle in args.get("forbidden_substrings", []):
            if needle in next_text:
                problems.append("Next line %r holds the forbidden substring %r" % (next_text, needle))

    then_line = next((line for line in lines if line.startswith("Then      ")), None)
    then_text = then_line[len("Then      "):].rstrip() if then_line else ""
    if then_line is None:
        problems.append("no line starting 'Then      '")
    elif then_text != args["then_line"]:
        problems.append("Then line %r, expected %r" % (then_text, args["then_line"]))

    names = _ui_files(workspace, args["dir"])
    if len(names) > args["max_files"]:
        problems.append("%s holds %d UI specs, expected at most %d"
                        % (args["dir"], len(names), args["max_files"]))

    for rel in args.get("absent", []):
        if os.path.exists(os.path.join(workspace, *rel.split("/"))):
            problems.append("%s exists and a send-back writes none" % rel)
    held, why = _phase_report_not_pass(workspace, args.get("report_phase", ""))
    if not held:
        problems.append(why)

    evidence = "gate %r; next %r; then %r; ui specs %s" \
               % (gate_line.strip() if gate_line else None, next_text, then_text, names)
    if problems:
        return False, evidence + "; " + "; ".join(problems)
    return True, evidence


def remedy_touched_only(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    try:
        text = _read(workspace, args["path"])
    except OSError as exc:
        return False, "%s is not readable: %s" % (args["path"], exc)

    _, front, body = _frontmatter(text)
    by_heading = dict(_sections(body))
    baseline = args.get("baseline_section_sha256", {})
    problems = []
    matched, differed = [], []

    for heading in args["unchanged_sections"]:
        if heading not in by_heading:
            problems.append("%s is absent from the file" % heading)
            continue
        digest = _section_digest(by_heading[heading])
        if heading in baseline and digest == baseline[heading]:
            matched.append(heading)
        else:
            differed.append(heading)
            problems.append("%s digest %s, expected %s" % (heading, digest[:16], baseline.get(heading, "none")[:16]))

    changed = []
    for heading in args["changed_any_of"]:
        if heading not in by_heading:
            continue
        digest = _section_digest(by_heading[heading])
        recorded = baseline.get(heading)
        if recorded is None:
            if by_heading[heading].strip():
                changed.append(heading)
        elif digest != recorded:
            changed.append(heading)
    if not changed:
        problems.append("none of %s differs from its recorded state" % args["changed_any_of"])

    if front.get("status") != args["end_status"]:
        problems.append("status %r, expected %r" % (front.get("status"), args["end_status"]))

    evidence = "unchanged matched %s; unchanged differed %s; changed %s; status %r" \
               % (matched, differed, changed, front.get("status"))
    if problems:
        return False, evidence + "; " + "; ".join(problems)
    return True, evidence


def sketch_contract(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    try:
        request = _load_json(workspace, args["request"])
    except (OSError, ValueError) as exc:
        return False, "%s is not readable JSON: %s" % (args["request"], exc)

    flow_ids = [flow.get("flow_id") for flow in request.get("flows", [])]
    result = _last_json_object_with_key(transcript, "screens")
    if result is None:
        return False, "the transcript holds no JSON object with a 'screens' key"

    problems = []
    if result.get("mode") != "sketch":
        problems.append("mode %r, expected 'sketch'" % result.get("mode"))
    if result.get("idea_id") != request.get("idea_id"):
        problems.append("idea_id %r, expected %r" % (result.get("idea_id"), request.get("idea_id")))

    screen_pattern = re.compile(r"^FLOW-[0-9]{3}-[0-9]{2}$")
    screen_ids = []
    for screen in result.get("screens", []):
        screen_id = screen.get("screen")
        screen_ids.append(screen_id)
        if not isinstance(screen_id, str) or not screen_pattern.match(screen_id):
            problems.append("screen %r does not match ^FLOW-[0-9]{3}-[0-9]{2}$" % screen_id)
        if screen.get("flow_id") not in flow_ids:
            problems.append("screen %r carries flow_id %r, which the request does not hold"
                            % (screen_id, screen.get("flow_id")))
        rel = (screen.get("path") or "").replace("\\", "/")
        out_dir = args["out_dir"].rstrip("/")
        if not rel.startswith(out_dir):
            problems.append("screen %r path %r is outside %s" % (screen_id, rel, out_dir))
        elif not os.path.isfile(_path(workspace, rel)):
            problems.append("screen %r path %r does not exist in the workspace" % (screen_id, rel))

    for flow_id in result.get("uncovered_flows", []):
        if flow_id not in flow_ids:
            problems.append("uncovered_flows holds %r, which the request does not hold" % flow_id)

    brand_rel = args.get("brand_sketch")
    if brand_rel:
        _brand_sketch_flat(workspace, brand_rel, problems)

    for rel in args.get("absent", []):
        if os.path.exists(_path(workspace, rel)):
            problems.append("%s was written and was expected to be absent" % rel)

    for rel in args.get("empty_dirs", []):
        full = _path(workspace, rel)
        if os.path.isdir(full) and any(os.path.isfile(os.path.join(full, e)) for e in os.listdir(full)):
            problems.append("%s holds a file and was expected to hold none" % rel)

    evidence = "screens %s; flows %s; absent %s" % (screen_ids, flow_ids, args.get("absent", []))
    if problems:
        return False, evidence + "; " + "; ".join(problems)
    return True, evidence
