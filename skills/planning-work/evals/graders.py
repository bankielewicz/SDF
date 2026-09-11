"""Graders for the planning-work skill.

Every function has the signature

    def <name>(workspace: str, transcript: str, args: dict) -> tuple[bool, str]

and reads only files under `workspace` and the `transcript` string. Standard
library only: no network, no subprocess, no randomness, no YAML package. The
readers below cover the two shapes this phase writes — a Markdown document with
flat seven-key frontmatter, and the one nested `sprint.yaml` mapping — which is
all these graders parse.

The runner mirrors `setup.files` nowhere, so a grader proves an upstream file
was not edited two ways instead (AUDIT-5 EVL-041): a SHA-256 the case records in
`expect.args`, and — where a line-by-line comparison is needed — the fixture the
case wrote from, read through `_fixture()` out of this skill's own
`evals/fixtures/` directory and nothing else outside the workspace.
"""

import hashlib
import os
import re


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


STORY_HEADINGS = [
    "Story",
    "Requirements",
    "Acceptance Criteria",
    "Constraints",
    "Anti-patterns",
    "Interface",
    "Layer",
    "Files",
    "Dependencies",
    "Out of scope",
]

SPRINT_KEYS = [
    "schema",
    "id",
    "phase",
    "status",
    "produced_by",
    "consumes",
    "open_questions",
    "epic",
    "capacity",
    "stories",
    "deferred",
]

DEFAULT_POINTS = [1, 2, 3, 5, 8]

STORIES_DIR = ".devforgeai/stories"
SPRINT_PATH = ".devforgeai/stories/sprint.yaml"
UI_SPEC_DIR = ".devforgeai/ui-specs"

STORY_FILE_RE = re.compile(r"^STORY-[0-9]{3}\.md$")
AC_LINE_RE = re.compile(r"^- (AC-[0-9]{3}): (.+)$")
AC_GRAMMAR_RE = re.compile(r"Given .+ When .+ Then .+")
DEP_LINE_RE = re.compile(r"^- (STORY-[0-9]{3}):")
REQ_ID_RE = re.compile(r"^REQ-[0-9]{3}$")
UI_ID_RE = re.compile(r"^UI-[0-9]{3}$")
DESIGN_CALL_RE = re.compile(r"/design .*--spec")
ENTRY_STATUS_RE = re.compile(r"^ +status: ")


# ---------------------------------------------------------------- primitives


def _path(workspace, rel):
    parts = [p for p in rel.replace("\\", "/").split("/") if p]
    return os.path.join(workspace, *parts)


def _fixture(name):
    """Read a file from this skill's own ``evals/fixtures/`` directory.

    Conventions section 9 scopes a grader to the workspace with one exception,
    written for exactly this case: there is no pre-run snapshot of the case's
    input, so a grader that must prove an upstream file was not edited reads the
    fixture the case wrote from, and nothing else outside the workspace
    (AUDIT-5 EVL-041, EVL-044).
    """
    here = os.path.dirname(os.path.abspath(__file__))
    with open(os.path.join(here, "fixtures", name), "r", encoding="utf-8") as handle:
        return handle.read()


def _digest(path):
    with open(path, "rb") as handle:
        return hashlib.sha256(handle.read()).hexdigest()


def _unchanged(workspace, rel, expected):
    """True when the workspace file still carries the digest the case recorded."""
    path = _path(workspace, rel)
    if not os.path.isfile(path):
        return False, "%s is absent from the workspace" % rel
    actual = _digest(path)
    if actual != str(expected).lower():
        return False, "%s digest is %s, the case recorded %s" % (rel, actual, expected)
    return True, ""


def _read(path):
    with open(path, "r", encoding="utf-8") as handle:
        return handle.read()


def _read_rel(workspace, rel):
    return _read(_path(workspace, rel))


def _scalar(raw):
    raw = raw.strip()
    if raw.startswith("[") and raw.endswith("]"):
        inner = raw[1:-1].strip()
        if not inner:
            return []
        return [_scalar(part) for part in inner.split(",")]
    if len(raw) >= 2 and raw[0] == raw[-1] and raw[0] in "\"'":
        return raw[1:-1]
    if re.match(r"^-?[0-9]+$", raw):
        return int(raw)
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


def _section(body, heading):
    for name, text in _sections(body):
        if name == heading:
            return text
    return ""


def _cells(line):
    return [cell.strip() for cell in line.strip().strip("|").split("|")]


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


def _list_items(section_text, pattern):
    out = []
    for line in section_text.split("\n"):
        match = pattern.match(line.strip())
        if match:
            out.append(match)
    return out


# ------------------------------------------------------------- yaml subset


def _indent(line):
    return len(line) - len(line.lstrip(" "))


def _yaml_block(lines, start, indent):
    """Parse the block of `lines` at or below `indent`, starting at `start`.

    Returns (value, next index). A block whose first non-blank line begins
    with '- ' is a sequence; otherwise it is a mapping. This covers the one
    nested shape this phase writes.
    """
    index = start
    while index < len(lines) and not lines[index].strip():
        index += 1
    if index >= len(lines) or _indent(lines[index]) < indent:
        return None, start
    if lines[index].lstrip(" ").startswith("- "):
        return _yaml_sequence(lines, index, _indent(lines[index]))
    return _yaml_mapping(lines, index, _indent(lines[index]))


def _yaml_mapping(lines, start, indent):
    keys, values = [], {}
    index = start
    while index < len(lines):
        line = lines[index]
        if not line.strip():
            index += 1
            continue
        if _indent(line) < indent:
            break
        if _indent(line) > indent or line.lstrip(" ").startswith("- "):
            break
        key, _, rest = line.strip().partition(":")
        key = key.strip()
        if rest.strip():
            keys.append(key)
            values[key] = _scalar(rest)
            index += 1
            continue
        keys.append(key)
        nested, index = _yaml_block(lines, index + 1, indent + 1)
        values[key] = nested if nested is not None else []
    return (keys, values), index


def _yaml_sequence(lines, start, indent):
    items = []
    index = start
    while index < len(lines):
        line = lines[index]
        if not line.strip():
            index += 1
            continue
        if _indent(line) < indent or not line.lstrip(" ").startswith("- "):
            break
        head = line.lstrip(" ")[2:]
        inner_indent = indent + 2
        entry_lines = [" " * inner_indent + head]
        index += 1
        while index < len(lines):
            following = lines[index]
            if following.strip() and _indent(following) < inner_indent:
                break
            if following.strip() and following.lstrip(" ").startswith("- ") and _indent(following) == indent:
                break
            entry_lines.append(following)
            index += 1
        entry, _ = _yaml_mapping(entry_lines, 0, inner_indent)
        items.append(entry)
    return items, index


def _load_sprint(text):
    """Return (ordered top-level key list, mapping) for sprint.yaml."""
    lines = [line.rstrip("\r") for line in text.split("\n")]
    (keys, values), _ = _yaml_mapping(lines, 0, 0)
    return keys, values


def _entries(value):
    """Normalise a parsed sequence of mappings to [(keys, mapping)]."""
    if not isinstance(value, list):
        return []
    out = []
    for item in value:
        if isinstance(item, tuple) and len(item) == 2:
            out.append(item)
    return out


# ---------------------------------------------------------------- transcript


def _labelled(transcript, label):
    """Return the content of every handoff line carrying `label` in column one."""
    out = []
    for line in transcript.split("\n"):
        stripped = line.rstrip()
        if stripped.startswith(label):
            rest = stripped[len(label):]
            if rest[:1] in ("", " "):
                out.append(rest.strip())
    return out


# ------------------------------------------------------------------- stories


def _story_files(workspace):
    directory = _path(workspace, STORIES_DIR)
    if not os.path.isdir(directory):
        return []
    names = sorted(name for name in os.listdir(directory) if STORY_FILE_RE.match(name))
    return [STORIES_DIR + "/" + name for name in names]


def _story_docs(workspace):
    out = []
    for rel in _story_files(workspace):
        keys, values, body = _frontmatter(_read_rel(workspace, rel))
        out.append((rel, keys, values, body))
    return out


def _covered_by(body):
    """Return {AC id: count of Covered by cells holding it} for one story."""
    counts = {}
    _, rows = _table(_section(body, "Requirements"))
    for row in rows:
        if len(row) < 3:
            continue
        for token in row[2].split():
            counts[token] = counts.get(token, 0) + 1
    return counts


def _file_paths(body):
    _, rows = _table(_section(body, "Files"))
    return [row[0] for row in rows if row and row[0]]


# -------------------------------------------------------------------- graders


def story_headings(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    """Every story file carries the ten H2 headings in template order."""
    docs = _story_docs(workspace)
    minimum = args.get("min_stories", 1)
    if len(docs) < minimum:
        return False, "found %d story files under %s, expected at least %d" % (
            len(docs), STORIES_DIR, minimum)
    for rel, _keys, _values, body in docs:
        found = [name for name, _text in _sections(body)]
        if found != STORY_HEADINGS:
            for index in range(max(len(found), len(STORY_HEADINGS))):
                got = found[index] if index < len(found) else "(absent)"
                want = STORY_HEADINGS[index] if index < len(STORY_HEADINGS) else "(extra)"
                if got != want:
                    return False, "%s heading %d is %r, expected %r" % (rel, index, got, want)
    return True, "%d story files carry the ten headings in order" % len(docs)


def req_coverage(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    """The REQ ids across every story's Requirements table equal the epic's set."""
    expected = set(args.get("reqs", []))
    docs = _story_docs(workspace)
    if not docs:
        return False, "no story files under %s" % STORIES_DIR
    found = set()
    for _rel, _keys, _values, body in docs:
        _, rows = _table(_section(body, "Requirements"))
        for row in rows:
            if row and REQ_ID_RE.match(row[0]):
                found.add(row[0])
    missing = sorted(expected - found)
    extra = sorted(found - expected)
    if missing or extra:
        return False, "epic %s: missing %s, unexpected %s" % (
            args.get("epic", "?"), missing or "none", extra or "none")
    return True, "%d requirements of %s each appear in a Requirements table" % (
        len(expected), args.get("epic", "?"))


def ac_grammar(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    """Every acceptance criterion parses, reads testably, and is covered once."""
    docs = _story_docs(workspace)
    if not docs:
        return False, "no story files under %s" % STORIES_DIR
    total = 0
    for rel, _keys, _values, body in docs:
        counts = _covered_by(body)
        items = _list_items(_section(body, "Acceptance Criteria"), AC_LINE_RE)
        for match in items:
            ac_id, text = match.group(1), match.group(2)
            total += 1
            if not AC_GRAMMAR_RE.search(text):
                return False, "%s %s does not match 'Given .+ When .+ Then .+'" % (rel, ac_id)
            seen = counts.get(ac_id, 0)
            if seen == 0:
                return False, "%s %s appears in no Covered by cell" % (rel, ac_id)
            if seen > 1:
                return False, "%s %s appears in %d Covered by cells" % (rel, ac_id, seen)
    minimum = args.get("min_acs", 1)
    if total < minimum:
        return False, "found %d acceptance criteria, expected at least %d" % (total, minimum)
    return True, "%d acceptance criteria parse, read testably, and are covered once" % total


def file_sets_disjoint(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    """No path is declared twice, and every dependency is ordered earlier."""
    try:
        _keys, sprint = _load_sprint(_read_rel(workspace, SPRINT_PATH))
    except OSError:
        return False, "%s is absent" % SPRINT_PATH
    orders, listed = {}, []
    for entry_keys, entry in _entries(sprint.get("stories")):
        del entry_keys
        listed.append(entry.get("id"))
        orders[entry.get("id")] = entry.get("order")
    bodies = {}
    for rel, _k, values, body in _story_docs(workspace):
        del rel
        bodies[values.get("id")] = body
    owner = {}
    for story_id in listed:
        body = bodies.get(story_id)
        if body is None:
            return False, "%s is listed in sprint.yaml stories and has no file" % story_id
        for path in _file_paths(body):
            if path in owner:
                return False, "%s and %s both declare %s" % (owner[path], story_id, path)
            owner[path] = story_id
    for story_id in listed:
        for match in _list_items(_section(bodies[story_id], "Dependencies"), DEP_LINE_RE):
            dependency = match.group(1)
            if dependency not in orders:
                continue
            if orders[dependency] >= orders[story_id]:
                return False, "%s depends on %s, whose order %s is not below %s" % (
                    story_id, dependency, orders[dependency], orders[story_id])
    return True, "%d declared paths are disjoint across %d sprint stories, order respects dependencies" % (
        len(owner), len(listed))


def sprint_shape(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    """sprint.yaml carries the eleven keys, a dense order, and consistent capacity."""
    try:
        keys, sprint = _load_sprint(_read_rel(workspace, SPRINT_PATH))
    except OSError:
        return False, "%s is absent" % SPRINT_PATH
    if keys != SPRINT_KEYS:
        return False, "top-level keys are %s, expected %s" % (keys, SPRINT_KEYS)
    capacity = sprint.get("capacity")
    capacity_map = capacity[1] if isinstance(capacity, tuple) else {}
    stories = _entries(sprint.get("stories"))
    deferred = _entries(sprint.get("deferred"))
    allowed = args.get("points", DEFAULT_POINTS)
    total = 0
    orders = []
    for _entry_keys, entry in stories:
        points = entry.get("points")
        if points not in allowed:
            return False, "%s carries points %r, outside %s" % (entry.get("id"), points, allowed)
        total += points
        orders.append(entry.get("order"))
    planned = capacity_map.get("points_planned")
    if planned != total:
        return False, "capacity.points_planned is %r and the sum of stories[].points is %d" % (
            planned, total)
    points_max = capacity_map.get("points_max")
    if not isinstance(points_max, int):
        return False, "capacity.points_max is %r, expected an integer" % (points_max,)
    if len(stories) > 1 and total > points_max:
        return False, "%d stories sum to %d points, above points_max %r" % (
            len(stories), total, points_max)
    if sorted(orders) != list(range(1, len(stories) + 1)):
        return False, "order values are %s, expected a dense permutation of 1..%d" % (
            sorted(orders), len(stories))
    listed = [entry.get("id") for _entry_keys, entry in stories]
    put_off = [entry.get("id") for _entry_keys, entry in deferred]
    both = sorted(set(listed) & set(put_off))
    if both:
        return False, "%s appears in both stories and deferred" % ", ".join(both)
    return True, "eleven keys in order, %d stories summing to %d of %r points, %d deferred" % (
        len(stories), total, points_max, len(deferred))


def design_called(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    """The interface story gained a UI spec through the Design skill in spec mode."""
    if not DESIGN_CALL_RE.search(transcript):
        return False, "the transcript holds no '/design ... --spec' invocation"
    target = None
    for rel, _keys, values, body in _story_docs(workspace):
        if _section(body, "Layer").strip() == "interface":
            target = (rel, values, body)
            break
    if target is None:
        return False, "no story carries a Layer line of 'interface'"
    rel, values, body = target
    _, rows = _table(_section(body, "Interface"))
    cited = [row[0] for row in rows if row and UI_ID_RE.match(row[0])]
    consumed = [item for item in values.get("consumes", []) if UI_ID_RE.match(str(item))]
    if not cited:
        return False, "%s ## Interface cites no UI-nnn" % rel
    if not consumed:
        return False, "%s frontmatter consumes holds no UI-nnn" % rel
    if set(cited) != set(consumed):
        return False, "%s cites %s in ## Interface and %s in consumes" % (
            rel, sorted(cited), sorted(consumed))
    for row in rows:
        if not row or not UI_ID_RE.match(row[0]):
            continue
        spec_rel = UI_SPEC_DIR + "/" + row[0] + ".md"
        try:
            spec = _read_rel(workspace, spec_rel)
        except OSError:
            return False, "%s is absent, and %s cites %s" % (spec_rel, rel, row[0])
        _spec_keys, _spec_values, spec_body = _frontmatter(spec)
        _, state_rows = _table(_section(spec_body, "States"))
        states = {state_row[0] for state_row in state_rows if state_row}
        for state in row[2].split() if len(row) > 2 else []:
            if state not in states:
                return False, "%s States covered holds %r, absent from %s ## States" % (
                    rel, state, spec_rel)
    return True, "%s cites %s, each with a spec whose States cover the row" % (rel, sorted(cited))


def send_back(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    """The handoff routes upstream, the cited ids appear, and upstream is untouched."""
    target = args.get("to", "")
    gate_lines = _labelled(transcript, "Gate")
    expected_gate = "SEND BACK to " + target
    if not any(line.startswith(expected_gate) for line in gate_lines):
        return False, "no Gate line starts %r; Gate lines read %s" % (expected_gate, gate_lines)
    next_lines = _labelled(transcript, "Next")
    if args.get("next") not in next_lines:
        return False, "Next line is %s, expected %r" % (next_lines, args.get("next"))
    found_lines = _labelled(transcript, "Found")
    for cited in args.get("ids", []):
        if not any(cited in line for line in found_lines):
            return False, "%s appears on no Found line; Found lines read %s" % (cited, found_lines)
    for rel, expected in sorted((args.get("upstream_unchanged") or {}).items()):
        held, why = _unchanged(workspace, rel, expected)
        if not held:
            return False, why
    for rel in args.get("absent", []):
        if os.path.exists(_path(workspace, rel)):
            return False, "%s exists and the send-back writes none" % rel
    held, why = _phase_report_not_pass(workspace, args.get("report_phase", "plan"))
    if not held:
        return False, why
    return True, "SEND BACK to %s citing %s, %d upstream files unchanged, %s" % (
        target, args.get("ids", []), len(args.get("upstream_unchanged", {})), why)


def remedy_ac(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    """One criterion was rewritten in place and nothing else in either file moved."""
    story = args.get("story", "")
    story_rel = STORIES_DIR + "/" + story + ".md"
    try:
        produced = _read(_path(workspace, story_rel))
    except OSError:
        return False, "%s is absent from the workspace" % story_rel
    original = _fixture(args["baseline_story"])

    def criteria(text):
        _keys, values, body = _frontmatter(text)
        out = {}
        for match in _list_items(_section(body, "Acceptance Criteria"), AC_LINE_RE):
            out[match.group(1)] = match.group(2)
        return values, out

    new_values, new_acs = criteria(produced)
    _old_values, old_acs = criteria(original)
    cited = args.get("ac", "")
    if cited not in new_acs:
        return False, "%s holds no %s line" % (story_rel, cited)
    if new_acs[cited] == old_acs.get(cited):
        return False, "%s %s is byte-identical to the line the case set up" % (story_rel, cited)
    if not AC_GRAMMAR_RE.search(new_acs[cited]):
        return False, "%s %s does not match 'Given .+ When .+ Then .+'" % (story_rel, cited)
    for other in args.get("untouched_acs", []):
        if new_acs.get(other) != old_acs.get(other):
            return False, "%s %s differs from the line the case set up" % (story_rel, other)
    if new_values.get("status") != "ready":
        return False, "%s frontmatter status is %r, expected 'ready'" % (
            story_rel, new_values.get("status"))

    try:
        sprint_new = _read(_path(workspace, SPRINT_PATH))
    except OSError:
        return False, "%s is absent from the workspace" % SPRINT_PATH
    sprint_old = _fixture(args["baseline_sprint"])
    _keys, parsed = _load_sprint(sprint_new)
    wanted = args.get("sprint_status", {})
    seen = {}
    for _entry_keys, entry in _entries(parsed.get("stories")):
        seen[entry.get("id")] = entry.get("status")
    for story_id, status in wanted.items():
        if seen.get(story_id) != status:
            return False, "%s stories entry for %s reads status %r, expected %r" % (
                SPRINT_PATH, story_id, seen.get(story_id), status)
    new_lines = sprint_new.split("\n")
    old_lines = sprint_old.split("\n")
    if len(new_lines) != len(old_lines):
        return False, "%s has %d lines and the fixture the case wrote from has %d" % (
            SPRINT_PATH, len(new_lines), len(old_lines))
    for index, (new_line, old_line) in enumerate(zip(new_lines, old_lines)):
        if new_line == old_line:
            continue
        if ENTRY_STATUS_RE.match(new_line) and ENTRY_STATUS_RE.match(old_line):
            continue
        return False, "%s line %d reads %r and the fixture the case wrote from reads %r" % (
            SPRINT_PATH, index + 1, new_line, old_line)

    next_lines = _labelled(transcript, "Next")
    if args.get("next") not in next_lines:
        return False, "Next line is %s, expected %r" % (next_lines, args.get("next"))
    return True, "%s %s rewritten, %d criteria untouched, sprint entry at status ready" % (
        story, cited, len(args.get("untouched_acs", [])))
