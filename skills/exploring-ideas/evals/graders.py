"""Graders for the exploring-ideas skill.

Every function has the signature

    def <name>(workspace: str, transcript: str, args: dict) -> tuple[bool, str]

reads only files under ``workspace`` and the ``transcript`` string, and returns
``(passed, evidence)``.

Standard library only: the runner environment ships PyYAML, but a grade that
depends on an installed package fails for a reason that has nothing to do with
the skill, so the YAML and Markdown readers below are in-module and cover the
shapes these documents use.
"""

import datetime
import hashlib
import json
import os
import re


def _flat_json_keys(workspace, rel, first, notes):
    """The file is one flat object whose first keys are `first`, in order.

    The ex-08 run wrote a correct sketch request with the seven envelope keys
    nested under a `meta` wrapper, which every downstream reader would miss. The
    documents this phase writes as JSON carry the envelope at the top level of
    one flat object, so the assertion is on the shape as well as the presence.
    Returns True when the file holds. Appends its own evidence to `notes`.
    """
    if not _exists(workspace, rel):
        notes.append("%s is absent" % rel)
        return False
    try:
        value = json.loads(_read(workspace, rel))
    except ValueError as error:
        notes.append("%s does not parse as JSON: %s" % (rel, error))
        return False
    if not isinstance(value, dict):
        notes.append("%s holds a %s, expected one object"
                     % (rel, type(value).__name__))
        return False
    keys = list(value)
    missing = [key for key in first if key not in keys]
    if missing:
        notes.append("%s is missing top-level keys: %s; it holds %s"
                     % (rel, ", ".join(missing), ", ".join(keys[:10])))
        return False
    if keys[:len(first)] != list(first):
        notes.append("%s opens with %s, expected %s first"
                     % (rel, ", ".join(keys[:len(first)]), ", ".join(first)))
        return False
    nested = [key for key in keys
              if isinstance(value[key], dict)
              and all(one in value[key] for one in first)]
    if nested:
        notes.append("%s repeats the envelope inside %s; it belongs at the top "
                     "level of one flat object" % (rel, ", ".join(nested)))
        return False
    notes.append("%s: %s at the top level" % (rel, ", ".join(first)))
    return True


QUOTES = "\"'"

SEVEN_KEYS = ["schema", "id", "phase", "status", "produced_by", "consumes",
              "open_questions"]

TWELVE_HEADINGS = ["Problem statement", "Target user", "What they do today",
                   "Why now", "Competitor scan", "Technology scan",
                   "Core flows", "Non-goals", "Success signal", "Mockups",
                   "Seed data", "Prototype"]

FLOW_ID = re.compile(r"^FLOW-[0-9]{3}$")


# --------------------------------------------------------------------------
# file access
# --------------------------------------------------------------------------

def _path(workspace, rel):
    return os.path.join(workspace, rel.replace("/", os.sep).rstrip(os.sep))


def _read(workspace, rel):
    with open(_path(workspace, rel), "r", encoding="utf-8") as handle:
        return handle.read()


def _exists(workspace, rel):
    return os.path.exists(_path(workspace, rel))


# --------------------------------------------------------------------------
# a small YAML reader: block mappings, block and flow lists, block scalars
# --------------------------------------------------------------------------

def _strip_comment(text):
    out, quote = [], None
    index = 0
    while index < len(text):
        char = text[index]
        if quote:
            out.append(char)
            if char == "\\" and index + 1 < len(text):
                out.append(text[index + 1])
                index += 2
                continue
            if char == quote:
                quote = None
        elif char in "\"'":
            quote = char
            out.append(char)
        elif char == "#" and (not out or out[-1] in " \t"):
            break
        else:
            out.append(char)
        index += 1
    return "".join(out).rstrip()


def _split_flow(text):
    parts, depth, quote, buf = [], 0, None, []
    for char in text:
        if quote:
            buf.append(char)
            if char == quote:
                quote = None
            continue
        if char in "\"'":
            quote = char
            buf.append(char)
        elif char in "[{":
            depth += 1
            buf.append(char)
        elif char in "]}":
            depth -= 1
            buf.append(char)
        elif char == "," and depth == 0:
            parts.append("".join(buf))
            buf = []
        else:
            buf.append(char)
    if "".join(buf).strip():
        parts.append("".join(buf))
    return [part.strip() for part in parts]


def _scalar(text):
    text = text.strip()
    if text == "" or text in ("null", "~", "Null", "NULL"):
        return None
    if text in ("true", "True", "TRUE"):
        return True
    if text in ("false", "False", "FALSE"):
        return False
    if len(text) >= 2 and text[0] == text[-1] and text[0] in "\"'":
        body = text[1:-1]
        if text[0] == '"':
            body = body.replace('\\"', '"').replace("\\n", "\n")
        return body
    if text.startswith("[") and text.endswith("]"):
        return [_scalar(part) for part in _split_flow(text[1:-1])]
    if text.startswith("{") and text.endswith("}"):
        mapping = {}
        for part in _split_flow(text[1:-1]):
            if ":" in part:
                key, value = part.split(":", 1)
                mapping[_scalar(key)] = _scalar(value)
        return mapping
    try:
        return int(text)
    except ValueError:
        pass
    try:
        return float(text)
    except ValueError:
        pass
    return text


def _indent(line):
    return len(line) - len(line.lstrip(" "))


def _block_scalar(lines, index, style, parent_indent):
    collected = []
    while index < len(lines):
        line = lines[index]
        if line.strip() == "":
            collected.append("")
            index += 1
            continue
        if _indent(line) <= parent_indent:
            break
        collected.append(line)
        index += 1
    while collected and collected[-1] == "":
        collected.pop()
    if not collected:
        return "", index
    inner = min(_indent(line) for line in collected if line.strip())
    body = [line[inner:] if line.strip() else "" for line in collected]
    if style.startswith("|"):
        text = "\n".join(body)
    else:
        folded, run = [], []
        for line in body:
            if line == "":
                folded.append(" ".join(run))
                run = []
                folded.append("")
            else:
                run.append(line.strip())
        folded.append(" ".join(run))
        text = "\n".join(part for part in folded if part != "").strip()
    if style.endswith("-"):
        text = text.rstrip("\n")
    return text, index


def _parse(lines, index, indent):
    first = index
    while first < len(lines) and lines[first].strip() == "":
        first += 1
    if first >= len(lines):
        return None, first
    if lines[first].lstrip().startswith("- "):
        return _parse_list(lines, first, _indent(lines[first]))
    return _parse_map(lines, first, _indent(lines[first]))


def _parse_map(lines, index, indent):
    mapping = {}
    while index < len(lines):
        line = lines[index]
        if line.strip() == "":
            index += 1
            continue
        if _indent(line) < indent:
            break
        if _indent(line) > indent or line.lstrip().startswith("- "):
            break
        content = _strip_comment(line.strip())
        if content == "":
            index += 1
            continue
        if ":" not in content:
            index += 1
            continue
        key, rest = content.split(":", 1)
        key, rest = key.strip(), rest.strip()
        if rest in (">", ">-", ">+", "|", "|-", "|+"):
            value, index = _block_scalar(lines, index + 1, rest, indent)
            mapping[key] = value
            continue
        if rest == "":
            value, next_index = _parse(lines, index + 1, indent + 1)
            probe = index + 1
            while probe < len(lines) and lines[probe].strip() == "":
                probe += 1
            if probe < len(lines) and _indent(lines[probe]) > indent:
                mapping[key] = value
                index = next_index
            elif (probe < len(lines) and _indent(lines[probe]) == indent
                  and lines[probe].lstrip().startswith("- ")):
                mapping[key] = value
                index = next_index
            else:
                mapping[key] = None
                index += 1
            continue
        mapping[key] = _scalar(rest)
        index += 1
    return mapping, index


def _parse_list(lines, index, indent):
    items = []
    while index < len(lines):
        line = lines[index]
        if line.strip() == "":
            index += 1
            continue
        if _indent(line) != indent or not line.lstrip().startswith("- "):
            break
        rest = _strip_comment(line.lstrip()[2:])
        block = [" " * (indent + 2) + rest]
        index += 1
        while index < len(lines):
            follow = lines[index]
            if follow.strip() == "":
                block.append(follow)
                index += 1
                continue
            if _indent(follow) > indent and not (
                    _indent(follow) == indent and follow.lstrip().startswith("- ")):
                block.append(follow)
                index += 1
                continue
            break
        stripped = rest.strip()
        if ":" in stripped and not stripped.startswith(("[", "{", "\"", "'")):
            value, _ = _parse_map(block, 0, indent + 2)
        else:
            value = _scalar(stripped)
        items.append(value)
    return items, index


def _load_yaml(text):
    lines = text.replace("\r\n", "\n").replace("\r", "\n").split("\n")
    lines = [line for line in lines if line.strip() != "---"]
    value, _ = _parse(lines, 0, 0)
    return value if isinstance(value, (dict, list)) else {}


# --------------------------------------------------------------------------
# Markdown readers
# --------------------------------------------------------------------------

def _frontmatter(text):
    text = text.replace("\r\n", "\n")
    if not text.startswith("---\n"):
        return {}, [], text
    end = text.find("\n---", 3)
    if end == -1:
        return {}, [], text
    block = text[4:end]
    keys = [line.split(":", 1)[0].strip()
            for line in block.split("\n")
            if line.strip() and not line.startswith((" ", "\t", "#"))
            and ":" in line]
    body = text[end + 4:]
    if body.startswith("\n"):
        body = body[1:]
    return _load_yaml(block), keys, body


def _sections(body):
    order, current, buf = [], None, []
    for line in body.replace("\r\n", "\n").split("\n"):
        if line.startswith("## "):
            if current is not None:
                order.append((current, buf))
            current, buf = line[3:].strip(), []
        elif current is not None:
            buf.append(line)
    if current is not None:
        order.append((current, buf))
    return order


def _section_lines(body, heading):
    for name, lines in _sections(body):
        if name == heading:
            return lines
    return []


def _section_digest(lines):
    trimmed = [line.rstrip() for line in lines]
    while trimmed and trimmed[0] == "":
        trimmed.pop(0)
    while trimmed and trimmed[-1] == "":
        trimmed.pop()
    return hashlib.sha256("\n".join(trimmed).encode("utf-8")).hexdigest()


def _table(lines):
    rows, header = [], None
    for line in lines:
        stripped = line.strip()
        if not stripped.startswith("|"):
            continue
        cells = [cell.strip() for cell in stripped.strip("|").split("|")]
        if header is None:
            header = cells
            continue
        if all(set(cell) <= set("-: ") for cell in cells) and cells:
            continue
        rows.append((cells, stripped))
    return header or [], rows


def _flow_rows(body):
    header, rows = _table(_section_lines(body, "Core flows"))
    index = header.index("ID") if "ID" in header else 0
    out = []
    for cells, raw in rows:
        out.append((cells[index] if index < len(cells) else "", raw, cells))
    return out


def _collapse(text):
    return " ".join(text.split())


def _date(value):
    return datetime.datetime.strptime(str(value), "%Y-%m-%d").date()


def _presence(workspace, args, notes):
    ok = True
    for rel in args.get("absent", []):
        if _exists(workspace, rel):
            ok = False
            notes.append("present but expected absent: " + rel)
        else:
            notes.append("absent: " + rel)
    for rel in args.get("present", []):
        if _exists(workspace, rel):
            notes.append("present: " + rel)
        else:
            ok = False
            notes.append("missing but expected present: " + rel)
    return ok


# --------------------------------------------------------------------------
# graders
# --------------------------------------------------------------------------

def brief_shape(workspace, transcript, args):
    rel = args.get("path", ".devforgeai/explore/brief.md")
    if not _exists(workspace, rel):
        return False, "no brief at " + rel
    front, keys, body = _frontmatter(_read(workspace, rel))
    notes, ok = [], True
    if keys != SEVEN_KEYS:
        ok = False
    notes.append("frontmatter keys: " + ", ".join(keys))
    headings = [name for name, _ in _sections(body)]
    if headings != TWELVE_HEADINGS:
        ok = False
    notes.append("headings: " + " | ".join(headings))
    flows = _flow_rows(body)
    ids = [flow_id for flow_id, _, _ in flows]
    notes.append("flow ids: " + ", ".join(ids))
    if not args.get("min_flows", 3) <= len(ids) <= args.get("max_flows", 5):
        ok = False
        notes.append("flow count %d outside %s to %s"
                     % (len(ids), args.get("min_flows", 3), args.get("max_flows", 5)))
    for flow_id in ids:
        if not FLOW_ID.match(flow_id):
            ok = False
            notes.append("malformed flow id: " + flow_id)
    if len(set(ids)) != len(ids):
        ok = False
        notes.append("duplicate flow id")
    signal = [line for line in _section_lines(body, "Success signal") if line.strip()]
    if len(signal) != 1:
        ok = False
    notes.append("success signal lines: %d" % len(signal))
    if front.get("id") is None:
        ok = False
        notes.append("frontmatter carries no id")
    seed_rel = args.get("seed_data", ".devforgeai/explore/seed-data.json")
    if seed_rel and not _flat_json_keys(workspace, seed_rel, SEVEN_KEYS, notes):
        ok = False

    return ok, "; ".join(notes)


def decision_shape(workspace, transcript, args):
    rel = args.get("path", ".devforgeai/explore/decision.yaml")
    if not _exists(workspace, rel):
        return False, "no decision record at " + rel
    doc = _load_yaml(_read(workspace, rel))
    notes, ok = [], True
    decision = doc.get("decision")
    notes.append("decision: %s" % decision)
    if decision != args["decision"]:
        ok = False
        notes.append("expected decision %s" % args["decision"])
    decided_on = doc.get("decided_on")
    notes.append("decided_on: %s" % decided_on)
    try:
        decided = _date(decided_on)
    except (ValueError, TypeError):
        ok = False
        decided = None
        notes.append("decided_on does not parse as YYYY-MM-DD")
    revisit = doc.get("revisit_on")
    notes.append("revisit_on: %s" % revisit)
    if args.get("revisit"):
        try:
            if decided is None or _date(revisit) <= decided:
                ok = False
                notes.append("revisit_on is not later than decided_on")
        except (ValueError, TypeError):
            ok = False
            notes.append("revisit_on does not parse as YYYY-MM-DD")
    elif revisit is not None:
        ok = False
        notes.append("revisit_on carries a value on a %s decision" % decision)
    carry = doc.get("carry_forward") or []
    notes.append("carry_forward entries: %d" % len(carry))
    if len(carry) != args.get("carry_forward_len", 0):
        ok = False
    reason = doc.get("reason") or ""
    words = len(str(reason).split())
    notes.append("reason words: %d" % words)
    if not 15 <= words <= 40:
        ok = False
    if not _presence(workspace, args, notes):
        ok = False
    return ok, "; ".join(notes)


def carry_forward_exact(workspace, transcript, args):
    rel = args.get("path", ".devforgeai/explore/decision.yaml")
    if not _exists(workspace, rel):
        return False, "no decision record at " + rel
    doc = _load_yaml(_read(workspace, rel))
    carry = doc.get("carry_forward") or []
    notes, ok = [], True
    notes.append("entries: %d" % len(carry))
    if len(carry) != 7:
        ok = False
    consumers = []
    for entry in carry:
        if not isinstance(entry, dict):
            ok = False
            notes.append("entry is not a mapping: %r" % (entry,))
            continue
        missing = [key for key in ("path", "sections", "becomes", "consumer")
                   if key not in entry]
        if missing:
            ok = False
            notes.append("entry missing %s" % ", ".join(missing))
        consumers.append(entry.get("consumer"))
        target = entry.get("path")
        if target:
            if _exists(workspace, target):
                notes.append("%s -> %s" % (target, entry.get("consumer")))
            else:
                ok = False
                notes.append("carried path missing: %s" % target)
    if consumers != args.get("consumers", consumers):
        ok = False
        notes.append("consumer order: " + ", ".join(str(c) for c in consumers))
    if not _presence(workspace, args, notes):
        ok = False
    return ok, "; ".join(notes)


def remedy_touched_only(workspace, transcript, args):
    rel = args.get("path", ".devforgeai/explore/brief.md")
    if not _exists(workspace, rel):
        return False, "no brief at " + rel
    front, _, body = _frontmatter(_read(workspace, rel))
    rows = {flow_id: raw for flow_id, raw, _ in _flow_rows(body)}
    baseline = args.get("baseline_rows", {})
    notes, ok = [], True
    for flow_id in args.get("changed", []):
        now = rows.get(flow_id)
        if now is None:
            ok = False
            notes.append("%s absent from Core flows" % flow_id)
        elif _collapse(now) == _collapse(baseline.get(flow_id, "")):
            ok = False
            notes.append("%s unchanged" % flow_id)
        else:
            notes.append("%s differs" % flow_id)
    for flow_id in args.get("unchanged", []):
        now = rows.get(flow_id)
        if now is None:
            ok = False
            notes.append("%s absent from Core flows" % flow_id)
        elif _collapse(now) != _collapse(baseline.get(flow_id, "")):
            ok = False
            notes.append("%s was rewritten" % flow_id)
        else:
            notes.append("%s held" % flow_id)
    texts = args.get("baseline_sections", {})
    digests = args.get("baseline_section_sha256", {})
    for heading in args.get("unchanged_sections", []):
        found = _section_digest(_section_lines(body, heading))
        if heading in texts:
            wanted = _section_digest(str(texts[heading]).split("\n"))
        else:
            wanted = digests.get(heading)
        if wanted is None:
            ok = False
            notes.append("%s: no baseline recorded" % heading)
        elif found != wanted:
            ok = False
            notes.append("%s: digest %s, baseline %s" % (heading, found[:12], str(wanted)[:12]))
        else:
            notes.append("%s: digest held" % heading)
    decision_rel = args.get("decision_path", ".devforgeai/explore/decision.yaml")
    if _exists(workspace, decision_rel):
        remedied = _load_yaml(_read(workspace, decision_rel)).get("remedied_flows") or []
        notes.append("remedied_flows: " + ", ".join(str(item) for item in remedied))
        if set(remedied) != set(args.get("remedied_flows", [])):
            ok = False
            notes.append("remedied_flows does not match the cited ids")
    else:
        ok = False
        notes.append("no decision record at " + decision_rel)
    return ok, "; ".join(notes)


def open_questions_mentions(workspace, transcript, args):
    rel = args.get("path", ".devforgeai/explore/brief.md")
    if not _exists(workspace, rel):
        return False, "no brief at " + rel
    front, _, body = _frontmatter(_read(workspace, rel))
    questions = front.get("open_questions") or []
    entries = [str(item) for item in questions]
    notes, ok = ["open_questions: " + " / ".join(entries)], True
    for wanted in args.get("ids", []):
        if not any(wanted in entry for entry in entries):
            ok = False
            notes.append("no open question names %s" % wanted)
    rows = {flow_id: raw for flow_id, raw, _ in _flow_rows(body)}
    notes.append("flow ids: " + ", ".join(sorted(rows)))
    for unwanted in args.get("absent_rows", []):
        if unwanted in rows:
            ok = False
            notes.append("%s was invented as a Core flows row" % unwanted)
    baseline = args.get("baseline_rows", {})
    for flow_id in args.get("changed", []):
        now = rows.get(flow_id)
        if now is None:
            ok = False
            notes.append("%s absent from Core flows" % flow_id)
        elif _collapse(now) == _collapse(baseline.get(flow_id, "")):
            ok = False
            notes.append("%s unchanged" % flow_id)
        else:
            notes.append("%s differs" % flow_id)
    return ok, "; ".join(notes)


def sketch_request_contract(workspace, transcript, args):
    rel = args.get("path", ".devforgeai/explore/sketch-request.json")
    if not _exists(workspace, rel):
        return False, "no sketch request at " + rel
    try:
        request = json.loads(_read(workspace, rel))
    except ValueError as error:
        return False, "sketch request does not parse as JSON: %s" % error
    notes, ok = [], True
    missing = [key for key in SEVEN_KEYS if key not in request]
    if missing:
        ok = False
        notes.append("missing top-level keys: " + ", ".join(missing))
    for key, wanted in (("mode", "sketch"),
                        ("fidelity", "wireframe"),
                        ("out_dir", args.get("out_dir")),
                        ("seed_data_path", args.get("seed_data_path"))):
        if wanted is not None and request.get(key) != wanted:
            ok = False
            notes.append("%s is %r, expected %r" % (key, request.get(key), wanted))
    constraints = request.get("constraints") or {}
    if sorted(constraints.keys()) != sorted(args.get("constraint_keys", [])):
        ok = False
        notes.append("constraint keys: " + ", ".join(sorted(constraints.keys())))
    for key in ("no_backend", "no_auth", "no_persistence"):
        if constraints.get(key) is not True:
            ok = False
            notes.append("%s is %r" % (key, constraints.get(key)))
    flows = request.get("flows") or []
    request_ids = [flow.get("flow_id") for flow in flows if isinstance(flow, dict)]
    brief_rel = args.get("brief", ".devforgeai/explore/brief.md")
    brief_ids = []
    if _exists(workspace, brief_rel):
        _, _, body = _frontmatter(_read(workspace, brief_rel))
        brief_ids = [flow_id for flow_id, _, _ in _flow_rows(body)]
    else:
        ok = False
        notes.append("no brief at " + brief_rel)
    notes.append("request flows: " + ", ".join(str(i) for i in request_ids))
    notes.append("brief flows: " + ", ".join(brief_ids))
    if request_ids != brief_ids:
        ok = False
        notes.append("request flow ids do not match the Core flows order")
    for flow in flows:
        if not isinstance(flow, dict):
            ok = False
            notes.append("flow entry is not an object")
            continue
        absent = [key for key in ("flow_id", "name", "actor", "steps", "outcome")
                  if key not in flow]
        if absent:
            ok = False
            notes.append("%s missing %s" % (flow.get("flow_id"), ", ".join(absent)))
        steps = flow.get("steps")
        if not isinstance(steps, list) or not 2 <= len(steps) <= 7 or \
                not all(isinstance(step, str) for step in steps):
            ok = False
            notes.append("%s steps: %r" % (flow.get("flow_id"), steps))
    return ok, "; ".join(notes)


def asked_decision(workspace, transcript, args):
    """The run put the decision to the user with the expected header and labels.

    Two paths, because the tool is there only under a permission host.
    Measured on `claude 2.1.268`, same argv apart from the host flags: a
    plain `claude -p` run lists 33 tools and no `AskUserQuestion`, so the
    eval's PreToolUse answer hook has nothing to intercept and the skill asks
    in prose instead; the same run under `--permission-prompt-tool` lists it
    and the hook fires. The runner supplies a host for every case that seeds
    answers, so the hook path is the live one and prose is the degradation.

    * Hook path, strong: `.claude/eval-answers-used.json` records the header the
      call carried, and the transcript holds the `AskUserQuestion(...)`
      `tool_use` event the stream reader renders, offering the expected labels.
    * Prose path: the question text carries the header and every label, and the
      run really reached the decision step — `args.artifact` names a document
      the workflow writes before it asks, so a model that types the labels
      without doing the work still fails (AUDIT-5 EVL-005, EVL-030).
    """
    header = args.get("header", "Decision")
    labels = args.get("labels", [])
    used_path = os.path.join(workspace, ".claude", "eval-answers-used.json")

    if os.path.isfile(used_path):
        try:
            with open(used_path, "r", encoding="utf-8") as handle:
                used = json.load(handle)
        except (OSError, ValueError) as exc:
            return False, ".claude/eval-answers-used.json does not parse: %s" % exc
        matched = [entry for entry in used if isinstance(entry, dict)
                   and entry.get("header") == header]
        if not matched:
            return False, ("the answer hook recorded no call with header %r; it "
                           "recorded %s" % (header, [e.get("header") for e in used
                                                     if isinstance(e, dict)]))
        calls = [line for line in transcript.splitlines()
                 if line.startswith("AskUserQuestion(")]
        carrying = [line for line in calls if ('"header": "' + header) in line]
        if not carrying:
            return False, ("the hook recorded a %s call and the transcript holds "
                           "no matching AskUserQuestion tool_use event" % header)
        missing = [label for label in labels if label not in carrying[0]]
        if missing:
            return False, "the %s call offered no option labelled %s" % (
                header, ", ".join(missing))
        answer = matched[0].get("answer")
        wanted = args.get("answer")
        if wanted is not None and answer != wanted:
            return False, "the %s call was answered %r, the case seeded %r" % (
                header, answer, wanted)
        return True, "%s asked through AskUserQuestion and answered %r" % (
            header, answer)

    position = transcript.find(header)
    if position == -1:
        return False, "the transcript never names the header %r" % header
    window = transcript[position:position + 4000]
    missing = [label for label in labels if label not in window]
    if missing:
        return False, ("the %s question offered no option labelled %s"
                       % (header, ", ".join(missing)))

    # The artifact half is the document the *answer* produces, not the one the
    # step before it writes: the brief is on disk before the question is put, so
    # gating on the brief would let a model that types the labels and stops
    # through. `decision.yaml` exists only when an answer came back.
    artifact = args.get("artifact")
    if artifact:
        path = os.path.join(workspace, *artifact.split("/"))
        if not os.path.isfile(path) or os.path.getsize(path) == 0:
            return False, ("%s is absent or empty: the %s question was put but no "
                           "answer came back, so the run stopped at it" % (
                               artifact, header))
        with open(path, "r", encoding="utf-8") as handle:
            body = handle.read()
        recorded = re.search(r"^decision:\s*(\S+)\s*$", body, re.M)
        value = recorded.group(1).strip(QUOTES).lower() if recorded else None
        if value not in [label.lower() for label in labels]:
            return False, "%s records decision %r, outside %s" % (
                artifact, value, labels)
    return True, ("%s put to the user in prose with labels %s, answered %s. "
                  "No answer-hook record, so this run had no permission host "
                  "and AskUserQuestion was not in the tool set" % (
                      header, ", ".join(labels),
                      value if artifact else "n/a"))


def blocked_on_cli(workspace, transcript, args):
    """The run stopped on a CLI refusal and wrote none of the phase's documents.

    Conventions section 9 asks each skill for two SEND BACK cases. Explore is
    phase 0: `## Send-back` states it plainly — "This phase sends nothing back.
    It is Phase 0: there is no upstream document to cite." The comparable
    accountability is the stop path the same section documents: a CLI call
    refuses, the run halts with one `Blocked` line carrying the binary's stderr,
    and nothing is written. This grader is that case's, and it has both halves —
    the line, and the absence of every document the phase would otherwise
    produce.
    """
    code = args.get("code", "")
    needed = list(args.get("names", []))
    if code:
        needed.append(code)
    matched = None
    for line in transcript.splitlines():
        if not line.startswith("Blocked"):
            continue
        if all(name in line for name in needed):
            matched = line.strip()
            break
    if matched is None:
        # The Stop hook renders the block, and an eval runs with hooks off
        # unless the machine carries a trust pin, so the stderr reaching the
        # transcript at all is the assertion that survives either way.
        if not all(name in transcript for name in needed):
            absent = [n for n in needed if n not in transcript]
            return False, "the transcript never names %s" % ", ".join(absent)
        matched = "stderr in the transcript, no Blocked line (hooks off)"

    for rel in args.get("absent", []):
        path = os.path.join(workspace, *rel.split("/"))
        if os.path.exists(path):
            return False, "%s was written; the run was supposed to stop before it" % rel

    return True, "%s; %d documents absent" % (matched, len(args.get("absent", [])))
