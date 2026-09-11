"""Graders for the establishing-context skill.

Every function has the signature

    def <name>(workspace: str, transcript: str, args: dict) -> tuple[bool, str]

reads only files under ``workspace`` and the ``transcript`` string, and returns
``(passed, evidence)``.

Standard library only. The runner environment ships PyYAML, but a grade that
depends on an installed package fails for a reason that has nothing to do with
the skill, so the frontmatter and Markdown-table readers below are in-module and
cover the shapes these documents use.
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


STEMS = ["tech-stack", "source-tree", "dependencies", "coding-standards",
         "architecture-constraints", "anti-patterns"]

CONTEXT_HEADINGS = {
    "tech-stack": ["Languages", "Runtimes", "Frameworks", "Data stores",
                   "Tooling", "Excluded technologies"],
    "source-tree": ["Roots", "Layers", "Directory map", "File placement rules",
                    "Naming conventions", "Generated and excluded paths"],
    "dependencies": ["Approved dependencies", "Forbidden dependencies",
                     "Version policy", "License policy", "Addition procedure"],
    "coding-standards": ["Formatting", "Naming", "Error handling", "Logging",
                         "Testing standards", "Documentation", "Design tokens"],
    "architecture-constraints": ["Constraints", "Layer dependency rules",
                                 "Constraint index"],
    "anti-patterns": ["Anti-patterns", "Anti-pattern index"],
}

CON_ID = re.compile(r"CON-[0-9]{3}")
AP_ID = re.compile(r"AP-[0-9]{3}")
REQ_ID = re.compile(r"REQ-[0-9]{3}")
ADR_ID = re.compile(r"ADR-[0-9]{3}")
KV_HEADER = "| Key | Value | Source |"


# ---------------------------------------------------------------------------
# file access
# ---------------------------------------------------------------------------

def _path(workspace, rel):
    return os.path.join(workspace, rel.replace("/", os.sep))


def _exists(workspace, rel):
    return os.path.isfile(_path(workspace, rel))


def _read(workspace, rel):
    with open(_path(workspace, rel), "r", encoding="utf-8", newline="") as handle:
        return handle.read().replace("\r\n", "\n")


def _context_path(stem):
    return ".devforgeai/context/%s.md" % stem


def _adr_paths(workspace):
    root = _path(workspace, ".devforgeai/adr")
    if not os.path.isdir(root):
        return []
    names = sorted(n for n in os.listdir(root)
                   if n.startswith("ADR-") and n.endswith(".md"))
    return [".devforgeai/adr/%s" % n for n in names]


# ---------------------------------------------------------------------------
# document readers
# ---------------------------------------------------------------------------

def _h2(path):
    """Every line beginning "## " in file order, heading text stripped."""
    with open(path, "r", encoding="utf-8", newline="") as handle:
        text = handle.read().replace("\r\n", "\n")
    return [line[3:].strip() for line in text.split("\n") if line.startswith("## ")]


def _frontmatter(path):
    """The key/value pairs between the first two "---" lines, values unparsed."""
    with open(path, "r", encoding="utf-8", newline="") as handle:
        text = handle.read().replace("\r\n", "\n")
    lines = text.split("\n")
    if not lines or lines[0].strip() != "---":
        return {}
    out = {}
    for line in lines[1:]:
        if line.strip() == "---":
            break
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        if ":" not in line:
            continue
        key, _, value = line.partition(":")
        out[key.strip()] = value.strip()
    return out


def _table_rows(text, heading=None):
    """Rows of every pipe table, as lists of trimmed cells.

    ``heading`` restricts the scan to the section under that H2.
    """
    lines = text.split("\n")
    if heading is not None:
        start = None
        for index, line in enumerate(lines):
            if line.startswith("## ") and line[3:].strip() == heading:
                start = index + 1
                break
        if start is None:
            return []
        stop = len(lines)
        for index in range(start, len(lines)):
            if lines[index].startswith("## "):
                stop = index
                break
        lines = lines[start:stop]
    rows = []
    for line in lines:
        stripped = line.strip()
        if not stripped.startswith("|") or not stripped.endswith("|"):
            continue
        cells = [cell.strip() for cell in stripped[1:-1].split("|")]
        if cells and all(set(cell) <= set("-: ") and cell for cell in cells):
            continue
        rows.append(cells)
    return rows


def _kv_rows(path):
    """(key, value, line_no) for every row under a "| Key | Value | Source |" table."""
    with open(path, "r", encoding="utf-8", newline="") as handle:
        text = handle.read().replace("\r\n", "\n")
    out = []
    inside = False
    for number, line in enumerate(text.split("\n"), start=1):
        stripped = line.strip()
        if stripped == KV_HEADER:
            inside = True
            continue
        if not inside:
            continue
        if not stripped.startswith("|") or not stripped.endswith("|"):
            inside = False
            continue
        cells = [cell.strip() for cell in stripped[1:-1].split("|")]
        if cells and all(set(cell) <= set("-: ") and cell for cell in cells):
            continue
        if len(cells) < 2:
            inside = False
            continue
        out.append((cells[0], cells[1], number))
    return out


def _field_blocks(text, prefix):
    """Map an id such as CON-003 to its "| Field | Value |" pairs.

    ``prefix`` is "CON" or "AP"; blocks open at a "### <PREFIX>-nnn " heading.
    """
    lines = text.split("\n")
    blocks = {}
    current = None
    for line in lines:
        if line.startswith("### "):
            match = re.match(r"^###\s+(%s-[0-9]{3})" % prefix, line)
            current = match.group(1) if match else None
            if current:
                blocks[current] = {}
            continue
        if line.startswith("## "):
            current = None
            continue
        if current is None:
            continue
        stripped = line.strip()
        if not stripped.startswith("|") or not stripped.endswith("|"):
            continue
        cells = [cell.strip() for cell in stripped[1:-1].split("|")]
        if len(cells) != 2:
            continue
        if cells[0].lower() == "field" and cells[1].lower() == "value":
            continue
        if all(set(cell) <= set("-: ") and cell for cell in cells):
            continue
        blocks[current][cells[0]] = cells[1]
    return blocks


def _handoff_block(transcript):
    """Lines from the last "Phase     " line through the "Full report:" line."""
    lines = transcript.split("\n")
    start = None
    for index, line in enumerate(lines):
        if line.startswith("Phase     "):
            start = index
    if start is None:
        return []
    for index in range(start, len(lines)):
        if lines[index].startswith("Full report:"):
            return lines[start:index + 1]
    return lines[start:]


def _collapse(text):
    return re.sub(r"\s+", " ", text).strip()


# ---------------------------------------------------------------------------
# graders
# ---------------------------------------------------------------------------

def context_headings(workspace, transcript, args):
    """Each named file's H2 list equals CONTEXT_HEADINGS[stem], in order."""
    wanted = args.get("files", "all")
    stems = STEMS if wanted == "all" else list(wanted)
    for stem in stems:
        rel = _context_path(stem)
        if not _exists(workspace, rel):
            return False, "%s is absent" % rel
        found = _h2(_path(workspace, rel))
        expected = CONTEXT_HEADINGS[stem]
        if found != expected:
            for index in range(max(len(found), len(expected))):
                left = found[index] if index < len(found) else "<none>"
                right = expected[index] if index < len(expected) else "<none>"
                if left != right:
                    return False, ("%s H2 index %d is '%s', expected '%s'"
                                   % (stem, index, left, right))
    return True, "%d files carry their heading list in order" % len(stems)


def con_referenced(workspace, transcript, args):
    """Every active CON is introduced by an ADR or sourced to a resolving REQ."""
    rel = _context_path("architecture-constraints")
    if not _exists(workspace, rel):
        return False, "%s is absent" % rel
    text = _read(workspace, rel)
    index_rows = _table_rows(text, "Constraint index")
    active = {}
    for cells in index_rows:
        if not cells or not CON_ID.fullmatch(cells[0]):
            continue
        status = cells[2] if len(cells) > 2 else ""
        source = cells[4] if len(cells) > 4 else ""
        if status == "active":
            active[cells[0]] = source
    if not active:
        return False, "the constraint index holds no active CON row"

    introduced = set()
    for adr in _adr_paths(workspace):
        adr_text = _read(workspace, adr)
        for cells in _table_rows(adr_text, "Constraints introduced"):
            if cells and CON_ID.fullmatch(cells[0]):
                introduced.add(cells[0])

    reqs = set()
    if _exists(workspace, ".devforgeai/requirements.yaml"):
        reqs = set(REQ_ID.findall(_read(workspace, ".devforgeai/requirements.yaml")))

    orphans = []
    for con, source in sorted(active.items()):
        if con in introduced:
            continue
        cited = REQ_ID.findall(source)
        if cited and all(req in reqs for req in cited):
            continue
        orphans.append("%s source '%s'" % (con, source))
    if orphans:
        return False, "unreferenced: " + "; ".join(orphans)
    return True, "%d active CON rows resolve to an ADR or a REQ" % len(active)


def non_goal_constraints(workspace, transcript, args):
    """The non-goal CON blocks split between the brief heading and the epic id."""
    rel = _context_path("architecture-constraints")
    if not _exists(workspace, rel):
        return False, "%s is absent" % rel
    text = _read(workspace, rel)
    blocks = _field_blocks(text, "CON")
    non_goals = {con: fields for con, fields in blocks.items()
                 if fields.get("kind") == "non-goal"}
    brief_source = args["brief_source"]
    epic_source = args["epic_source"]
    from_brief, from_epic, other = [], [], []
    for con, fields in sorted(non_goals.items()):
        source = fields.get("source", "")
        if source == brief_source:
            from_brief.append(con)
        elif source == epic_source:
            from_epic.append(con)
        else:
            other.append("%s source '%s'" % (con, source))
    if other:
        return False, "non-goal CON with an unexpected source: " + "; ".join(other)
    if len(from_brief) != args["brief"]:
        return False, ("%d non-goal CON cite '%s', expected %d"
                       % (len(from_brief), brief_source, args["brief"]))
    if len(from_epic) != args["epic"]:
        return False, ("%d non-goal CON cite '%s', expected %d"
                       % (len(from_epic), epic_source, args["epic"]))
    indexed = {cells[0] for cells in _table_rows(text, "Constraint index")
               if cells and CON_ID.fullmatch(cells[0])}
    missing = sorted(set(non_goals) - indexed)
    if missing:
        return False, "absent from the constraint index: " + ", ".join(missing)
    return True, ("%d brief non-goals and %d epic non-goals, all indexed"
                  % (len(from_brief), len(from_epic)))


def drafts_accepted(workspace, transcript, args):
    """All six files exist, are accepted, carry no open question, and are filled."""
    for stem in STEMS:
        rel = _context_path(stem)
        if not _exists(workspace, rel):
            return False, "%s is absent" % rel
        front = _frontmatter(_path(workspace, rel))
        if front.get("status") != "accepted":
            return False, "%s status is '%s'" % (rel, front.get("status", "<none>"))
        if front.get("open_questions") != "[]":
            return False, ("%s open_questions is '%s'"
                           % (rel, front.get("open_questions", "<none>")))
        text = _read(workspace, rel)
        lines = text.split("\n")
        starts = [index for index, line in enumerate(lines) if line.startswith("## ")]
        for order, index in enumerate(starts):
            stop = starts[order + 1] if order + 1 < len(starts) else len(lines)
            body = [line for line in lines[index + 1:stop] if line.strip()]
            if not body:
                return False, "%s section '%s' is empty" % (rel, lines[index][3:].strip())
    return True, "six files accepted, no open questions, every section filled"


def send_back(workspace, transcript, args):
    """The handoff routes upstream, cites the ids, and leaves the input untouched."""
    block = _handoff_block(transcript)
    if not block:
        return False, "the transcript holds no handoff block"
    if len(block) > 12:
        return False, "the handoff block is %d lines" % len(block)

    gate = [line for line in block if line.startswith("Gate      ")]
    marker = "SEND BACK to " + args["to"]
    if not gate or marker not in gate[0]:
        return False, "gate line: %s" % (gate[0] if gate else "<absent>")

    nxt = [line for line in block if line.startswith("Next      ")]
    if not nxt:
        return False, "the handoff block has no Next line"
    next_line = nxt[0]
    if args["idea"] not in next_line or " --remedy " not in next_line:
        return False, "next line: %s" % next_line
    remedy = next_line.split(" --remedy ", 1)[1].strip()
    for one in args["ids"]:
        if one not in remedy:
            return False, "%s is absent from the remedy list: %s" % (one, next_line)

    found = [line for line in block if line.startswith("Found     ")]
    for one in args["ids"]:
        if not any(one in line for line in found):
            return False, "no Found line names %s" % one

    for rel, digest in args.get("upstream_sha256", {}).items():
        path = _path(workspace, rel)
        if not os.path.isfile(path):
            return False, "%s is absent from the workspace" % rel
        with open(path, "rb") as handle:
            actual = hashlib.sha256(handle.read()).hexdigest()
        if actual != digest:
            return False, "%s digest is %s, recorded %s" % (rel, actual, digest)

    held, why = _phase_report_not_pass(workspace, args.get("report_phase", "constitute"))
    if not held:
        return False, why

    return True, ("%s · %s · %d lines · upstream unchanged · %s"
                  % (gate[0].strip(), ",".join(args["ids"]), len(block), why))


def remedy_supersession(workspace, transcript, args):
    """One new ADR supersedes the introducing ADR and retires exactly that CON."""
    con = args["con"]
    adr = args["adr"]
    baseline = args["baseline_rows"]

    introducing = ".devforgeai/adr/%s.md" % adr
    if not _exists(workspace, introducing):
        return False, "%s is absent" % introducing
    status = _frontmatter(_path(workspace, introducing)).get("status")
    if status != "superseded":
        return False, "%s status is '%s'" % (introducing, status)

    supersedors = []
    for path in _adr_paths(workspace):
        if path == introducing:
            continue
        for cells in _table_rows(_read(workspace, path), "Supersedes"):
            if len(cells) < 2:
                continue
            if adr in cells[0] and con in cells[1]:
                supersedors.append(path)
                break
    if len(supersedors) != 1:
        return False, ("%d ADRs supersede %s and retire %s"
                       % (len(supersedors), adr, con))

    rel = _context_path("architecture-constraints")
    if not _exists(workspace, rel):
        return False, "%s is absent" % rel
    text = _read(workspace, rel)
    rows = {}
    for line in text.split("\n"):
        stripped = line.strip()
        if not stripped.startswith("|") or not stripped.endswith("|"):
            continue
        cells = [cell.strip() for cell in stripped[1:-1].split("|")]
        if cells and CON_ID.fullmatch(cells[0]):
            rows.setdefault(cells[0], (stripped, cells))

    if con not in rows:
        return False, "%s has no row in the constraint index" % con
    if len(rows[con][1]) < 3 or rows[con][1][2] != "retired":
        return False, "%s row reads status '%s'" % (con, rows[con][1][2:3])
    if _collapse(rows[con][0]) == _collapse(baseline[con]):
        return False, "%s row is unchanged from its baseline" % con

    replacements = [one for one, (_, cells) in rows.items()
                    if one not in baseline and len(cells) > 2 and cells[2] == "active"]
    if not replacements:
        return False, "no replacement CON row exists with status active"

    for one in args["untouched_con"]:
        if one not in rows:
            return False, "%s has no row in the constraint index" % one
        if _collapse(rows[one][0]) != _collapse(baseline[one]):
            return False, ("%s row changed: %s" % (one, rows[one][0]))

    return True, ("%s superseded by %s, %s retired, %s allocated"
                  % (adr, os.path.basename(supersedors[0]), con,
                     ",".join(sorted(replacements))))


def key_consistency(workspace, transcript, args):
    """No key under the six files holds two distinct values."""
    seen = {}
    read = 0
    for stem in STEMS:
        rel = _context_path(stem)
        if not _exists(workspace, rel):
            continue
        read += 1
        for key, value, number in _kv_rows(_path(workspace, rel)):
            seen.setdefault(key, []).append((rel, number, value))
    if not read:
        return False, "no context file is present"

    for key, triples in sorted(seen.items()):
        values = {value for _, _, value in triples}
        if len(values) > 1:
            detail = "; ".join("%s:%d %s" % triple for triple in triples)
            return False, "%s holds %d values: %s" % (key, len(values), detail)

    wanted = args.get("key")
    if wanted is not None and wanted not in seen:
        return False, "%s is present in none of the %d files read" % (wanted, read)
    if wanted is not None:
        triples = seen[wanted]
        return True, ("%s holds '%s' across %d rows in %d files"
                      % (wanted, triples[0][2], len(triples), read))
    return True, "%d keys across %d files, each with one value" % (len(seen), read)
