"""Graders for the discovering-requirements skill.

Every function has the signature

    def <name>(workspace: str, transcript: str, args: dict) -> tuple[bool, str]

reads only files under ``workspace``, the fixture bytes beside this module, and
the ``transcript`` string, and returns ``(passed, evidence)``.

Standard library only. The runner environment ships PyYAML, but a grade that
depends on an installed package fails for a reason that has nothing to do with
the skill, so the YAML reader below is in-module and covers the shapes
``requirements.yaml`` uses: block mappings, block sequences of mappings, flow
sequences, block scalars, and nulls.

Two graders compare a post-run document against the document the case started
from. ``specs/01-cli.md`` gives the runner no pre-run snapshot -- it writes
``setup.files`` into the workspace and mirrors them nowhere else -- so the
baseline travels as fixture bytes beside this module. ``fixtures/baselines.txt``
records which fixture holds which case's starting document and how to
regenerate them from ``cases.jsonl``. A case may name a different file through
the ``baseline`` key of its ``args``.
"""

import hashlib
import json
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


TOP_LEVEL_KEYS = ["schema", "id", "phase", "status", "produced_by", "consumes",
                  "open_questions", "revision", "revision_log", "accepted_by",
                  "accepted_at", "personas", "epics", "requirements"]

REQUIREMENTS_PATH = ".devforgeai/requirements.yaml"

REQ_ID = re.compile(r"^REQ-[0-9]{3}$")
PERSONA_ID = re.compile(r"^PERSONA-[0-9]{3}$")
EPIC_ID = re.compile(r"^EPIC-[0-9]{3}$")
RFC3339 = re.compile(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$")
MAP_ENTRY = re.compile(r"^([A-Za-z_][A-Za-z0-9_.\-]*):(\s|$)")


# --------------------------------------------------------------------------
# file access
# --------------------------------------------------------------------------

def _path(workspace, rel):
    return os.path.join(workspace, rel.replace("/", os.sep))


def _exists(workspace, rel):
    return os.path.exists(_path(workspace, rel))


def _read(workspace, rel):
    with open(_path(workspace, rel), "r", encoding="utf-8", newline="") as handle:
        return handle.read()


def _fixture(name):
    here = os.path.dirname(os.path.abspath(__file__))
    with open(os.path.join(here, "fixtures", name), "r",
              encoding="utf-8", newline="") as handle:
        return handle.read()


# --------------------------------------------------------------------------
# a small YAML reader
# --------------------------------------------------------------------------

def _strip_comment(text):
    out, quote, index = [], None, 0
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


def _lines(text):
    rows = []
    for raw in text.replace("\r\n", "\n").split("\n"):
        stripped = raw.lstrip(" ")
        if stripped == "" or stripped.startswith("#") or stripped in ("---", "..."):
            continue
        content = _strip_comment(raw).rstrip()
        if content.strip() == "":
            continue
        rows.append((len(raw) - len(stripped), content.lstrip(" ")))
    return rows


def _parse_node(rows, index, indent):
    if rows[index][1].startswith("- "):
        return _parse_seq(rows, index, indent)
    return _parse_map(rows, index, indent)


def _parse_seq(rows, index, indent):
    items = []
    while index < len(rows):
        ind, content = rows[index]
        if ind != indent or not content.startswith("- "):
            break
        rest = content[2:].strip()
        if MAP_ENTRY.match(rest):
            block = [(indent + 2, rest)]
            cursor = index + 1
            while cursor < len(rows) and rows[cursor][0] > indent:
                block.append(rows[cursor])
                cursor += 1
            value, _ = _parse_map(block, 0, indent + 2)
            items.append(value)
            index = cursor
        else:
            items.append(_scalar(rest))
            index += 1
    return items, index


def _parse_map(rows, index, indent):
    mapping = {}
    while index < len(rows):
        ind, content = rows[index]
        if ind != indent or content.startswith("- "):
            break
        match = re.match(r"^([^:]+):\s*(.*)$", content)
        if not match:
            break
        key = match.group(1).strip()
        rest = match.group(2).strip()
        cursor = index + 1
        if rest in (">", ">-", "|", "|-", ">+", "|+"):
            buf = []
            while cursor < len(rows) and rows[cursor][0] > indent:
                buf.append(rows[cursor][1].strip())
                cursor += 1
            mapping[key] = " ".join(buf)
            index = cursor
        elif rest == "":
            if cursor < len(rows) and rows[cursor][0] > indent:
                value, cursor = _parse_node(rows, cursor, rows[cursor][0])
                mapping[key] = value
            elif (cursor < len(rows) and rows[cursor][0] == indent
                  and rows[cursor][1].startswith("- ")):
                value, cursor = _parse_seq(rows, cursor, indent)
                mapping[key] = value
            else:
                mapping[key] = None
            index = cursor
        else:
            mapping[key] = _scalar(rest)
            index = cursor
    return mapping, index


def _yaml(text):
    rows = _lines(text)
    if not rows:
        return {}
    value, _ = _parse_node(rows, 0, rows[0][0])
    return value


# --------------------------------------------------------------------------
# shared helpers
# --------------------------------------------------------------------------

def _entries(document, key):
    value = document.get(key)
    return value if isinstance(value, list) else []


def _by_id(entries):
    return {entry.get("id"): entry for entry in entries
            if isinstance(entry, dict) and entry.get("id")}


def _digest(entry):
    canonical = json.dumps(entry, sort_keys=True, default=str, ensure_ascii=False)
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()[:16]


def _digests(entries):
    return {entry_id: _digest(entry) for entry_id, entry in _by_id(entries).items()}


def _grouping(document):
    """requirement id -> list of epic ids that name it."""
    placement = {}
    for epic in _entries(document, "epics"):
        for req_id in epic.get("requirements") or []:
            placement.setdefault(req_id, []).append(epic.get("id"))
    return placement


def _load_requirements(workspace):
    if not _exists(workspace, REQUIREMENTS_PATH):
        return None, "%s is absent from the workspace" % REQUIREMENTS_PATH
    try:
        document = _yaml(_read(workspace, REQUIREMENTS_PATH))
    except Exception as error:                                   # noqa: BLE001
        return None, "%s did not parse: %s" % (REQUIREMENTS_PATH, error)
    if not isinstance(document, dict):
        return None, "%s is not a mapping" % REQUIREMENTS_PATH
    return document, ""


def _baseline(args, default_name):
    name = args.get("baseline") or default_name
    return _yaml(_fixture(name)), name


def _transcript_lines(transcript, label):
    found = []
    for raw in (transcript or "").replace("\r\n", "\n").split("\n"):
        if raw.strip().startswith(label):
            found.append(raw.strip())
    return found


# --------------------------------------------------------------------------
# graders
# --------------------------------------------------------------------------

def requirements_document_wellformed(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    """The written document carries the fourteen top-level keys in template
    order, meets the minimum counts, resolves every actor, groups every
    requirement in exactly one epic, gives every epic an exclusion and a
    numeric success metric, and carries the expected ``consumes`` list.

    ``consumes`` is compared as a set: the spec's docstring says "equals", and a
    document whose flow ids sit in another order carries the same references, so
    ordering alone does not fail the grade."""
    document, problem = _load_requirements(workspace)
    if document is None:
        return False, problem

    present = [key for key in document.keys() if key in TOP_LEVEL_KEYS]
    missing = [key for key in TOP_LEVEL_KEYS if key not in document]
    if missing:
        return False, "top-level keys absent: %s" % ", ".join(missing)
    if present != TOP_LEVEL_KEYS:
        return False, "top-level key order is %s" % ", ".join(present)

    personas = _entries(document, "personas")
    epics = _entries(document, "epics")
    requirements = _entries(document, "requirements")
    for key, entries, floor in (("personas", personas, args.get("min_personas", 1)),
                                ("epics", epics, args.get("min_epics", 1)),
                                ("requirements", requirements,
                                 args.get("min_requirements", 1))):
        if len(entries) < floor:
            return False, "%s holds %d entries, below the %d the case names" % (
                key, len(entries), floor)

    persona_ids = set(_by_id(personas))
    for requirement in requirements:
        actor = requirement.get("actor")
        if actor not in persona_ids:
            return False, "%s names actor %r, which is no personas[].id" % (
                requirement.get("id"), actor)

    placement = _grouping(document)
    for requirement in requirements:
        req_id = requirement.get("id")
        holders = placement.get(req_id, [])
        if len(holders) != 1:
            return False, "%s sits in %d epics: %s" % (
                req_id, len(holders), ", ".join(str(one) for one in holders) or "none")
    requirement_ids = set(_by_id(requirements))
    for req_id in placement:
        if req_id not in requirement_ids:
            return False, "an epic names %s, which is in no requirements[] entry" % req_id

    for epic in epics:
        exclusions = epic.get("out_of_scope") or []
        if not exclusions:
            return False, "%s carries an empty out_of_scope" % epic.get("id")
        metric = epic.get("success_metric") or ""
        if not re.search(r"[0-9]", str(metric)):
            return False, "%s success_metric holds no number: %r" % (
                epic.get("id"), metric)

    expected = args.get("expect_consumes")
    if expected is not None:
        actual = document.get("consumes") or []
        if sorted(str(one) for one in actual) != sorted(str(one) for one in expected):
            return False, "consumes is %s, the case names %s" % (actual, expected)

    fixed_source = args.get("expect_all_sources")
    if fixed_source:
        for requirement in requirements:
            if requirement.get("source") != fixed_source:
                return False, "%s carries source %r, the case names %r" % (
                    requirement.get("id"), requirement.get("source"), fixed_source)

    return True, "%d personas, %d epics, %d requirements, consumes %s" % (
        len(personas), len(epics), len(requirements), document.get("consumes"))


def sendback_to_explore(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    """The run stopped before writing a document and the handoff sent the cited
    flow ids back to Explore."""
    absent = args.get("absent_path", REQUIREMENTS_PATH)
    if _exists(workspace, absent):
        return False, "%s was written; a send-back writes no document" % absent

    gate_lines = [line for line in _transcript_lines(transcript, "Gate")
                  if "SEND BACK to Explore" in line]
    if not gate_lines:
        return False, "no transcript line starts Gate and holds 'SEND BACK to Explore'"

    next_lines = [line for line in _transcript_lines(transcript, "Next")
                  if "--remedy" in line]
    if not next_lines:
        return False, "no transcript line starts Next and holds '--remedy'"

    found_lines = _transcript_lines(transcript, "Found")
    carrier = " ".join(found_lines + next_lines)
    absent_ids = [one for one in args.get("cited", []) if one not in carrier]
    if absent_ids:
        return False, "%s named on no Found line and on no Next line; Found: %s" % (
            ", ".join(absent_ids), " | ".join(found_lines) or "none")

    held, why = _phase_report_not_pass(workspace, args.get("report_phase", "discover"))
    if not held:
        return False, why

    return True, "%s | %s" % (" | ".join(gate_lines + found_lines + next_lines), why)


def remedy_preserved_other_requirements(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    """A remedy pass rewrote the cited requirements, raised the revision, logged
    the pass, and left every other record as it was."""
    document, problem = _load_requirements(workspace)
    if document is None:
        return False, problem
    before, fixture_name = _baseline(args, "requirements-remedy-plan.yaml")

    expect_revision = args.get("expect_revision")
    if expect_revision is not None and document.get("revision") != expect_revision:
        return False, "revision is %r, the case names %r" % (
            document.get("revision"), expect_revision)

    log_before = _entries(before, "revision_log")
    log_after = _entries(document, "revision_log")
    if len(log_after) != len(log_before) + 1:
        return False, "revision_log went from %d entries to %d; one entry is added per pass" % (
            len(log_before), len(log_after))
    entry = log_after[-1]
    if entry.get("from") != args.get("expect_from"):
        return False, "revision_log entry names from %r, the case names %r" % (
            entry.get("from"), args.get("expect_from"))
    reopened = sorted(str(one) for one in (entry.get("reopened") or []))
    if reopened != sorted(str(one) for one in args.get("reopened", [])):
        return False, "revision_log reopened is %s, the case names %s" % (
            entry.get("reopened"), args.get("reopened"))

    before_reqs = _digests(_entries(before, "requirements"))
    after_reqs = _digests(_entries(document, "requirements"))
    if set(before_reqs) - set(after_reqs):
        return False, "ids present before the remedy and absent after: %s" % ", ".join(
            sorted(set(before_reqs) - set(after_reqs)))

    for req_id in args.get("untouched", []):
        if req_id not in after_reqs:
            return False, "%s is absent after the remedy" % req_id
        if before_reqs.get(req_id) != after_reqs.get(req_id):
            return False, "%s changed: %s before, %s after" % (
                req_id, before_reqs.get(req_id), after_reqs.get(req_id))
    for req_id in args.get("reopened", []):
        if req_id not in after_reqs:
            return False, "%s is absent after the remedy" % req_id
        if before_reqs.get(req_id) == after_reqs.get(req_id):
            return False, "%s holds the text it held before the remedy" % req_id

    for collection in ("personas", "epics"):
        left = _digests(_entries(before, collection))
        right = _digests(_entries(document, collection))
        for entry_id, digest in left.items():
            if right.get(entry_id) != digest:
                return False, "%s %s changed: %s before, %s after" % (
                    collection, entry_id, digest, right.get(entry_id))

    if document.get("accepted_by") is None:
        return False, "accepted_by is null; the remedy pass ended without acceptance"

    return True, "baseline %s; revision %s; reopened %s; untouched %s" % (
        fixture_name, document.get("revision"), reopened,
        ", ".join("%s=%s" % (one, after_reqs.get(one))
                  for one in args.get("untouched", [])))


def design_remedy_created_requirement(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    """A Design remedy allocated one requirement for the cited interface id,
    drafted its text, placed it in one existing epic, and changed nothing
    else."""
    document, problem = _load_requirements(workspace)
    if document is None:
        return False, problem
    before, fixture_name = _baseline(args, "requirements-remedy-design.yaml")
    ui_id = args.get("ui_id")

    expect_revision = args.get("expect_revision")
    if expect_revision is not None and document.get("revision") != expect_revision:
        return False, "revision is %r, the case names %r" % (
            document.get("revision"), expect_revision)

    log_after = _entries(document, "revision_log")
    if len(log_after) != len(_entries(before, "revision_log")) + 1:
        return False, "revision_log holds %d entries; one entry is added per pass" % len(
            log_after)
    entry = log_after[-1]
    if entry.get("from") != "design":
        return False, "revision_log entry names from %r, not design" % entry.get("from")
    added = entry.get("added") or []
    if len(added) != 1:
        return False, "revision_log added is %s; one id is allocated per cited UI id" % added

    requirements = _entries(document, "requirements")
    tracing = [one for one in requirements
               if [str(each) for each in (one.get("traces_to") or [])] == [ui_id]]
    if len(tracing) != 1:
        return False, "%d requirements carry traces_to [%s]" % (len(tracing), ui_id)
    created = tracing[0]
    if created.get("source") != "user":
        return False, "%s carries source %r, not user" % (
            created.get("id"), created.get("source"))
    for field in ("statement", "rationale", "acceptance_signal"):
        if not str(created.get(field) or "").strip():
            return False, "%s carries an empty %s" % (created.get("id"), field)
    new_id = created.get("id")
    if new_id != added[0]:
        return False, "revision_log added %s, the traced requirement is %s" % (
            added[0], new_id)

    holders = _grouping(document).get(new_id, [])
    if len(holders) != 1:
        return False, "%s sits in %d epics: %s" % (
            new_id, len(holders), ", ".join(str(one) for one in holders) or "none")

    before_reqs = _digests(_entries(before, "requirements"))
    after_reqs = _digests(_entries(document, "requirements"))
    for req_id in args.get("untouched", []):
        if before_reqs.get(req_id) != after_reqs.get(req_id):
            return False, "%s changed: %s before, %s after" % (
                req_id, before_reqs.get(req_id), after_reqs.get(req_id))

    persona_before = _digests(_entries(before, "personas"))
    persona_after = _digests(_entries(document, "personas"))
    for entry_id, digest in persona_before.items():
        if persona_after.get(entry_id) != digest:
            return False, "persona %s changed: %s before, %s after" % (
                entry_id, digest, persona_after.get(entry_id))

    epics_before = _by_id(_entries(before, "epics"))
    for epic in _entries(document, "epics"):
        epic_id = epic.get("id")
        if epic_id not in epics_before:
            return False, "epic %s is new; a design remedy allocates no epic" % epic_id
        trimmed = dict(epic)
        trimmed["requirements"] = [one for one in (epic.get("requirements") or [])
                                   if one != new_id]
        if _digest(trimmed) != _digest(epics_before[epic_id]):
            return False, "epic %s changed beyond the appended %s" % (epic_id, new_id)

    return True, "baseline %s; %s traces to %s in %s, source %s" % (
        fixture_name, new_id, ui_id, holders[0], created.get("source"))


def personas_resolve(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    """Every actor names a persona and every persona is named by a
    requirement."""
    document, problem = _load_requirements(workspace)
    if document is None:
        return False, problem
    personas = set(_by_id(_entries(document, "personas")))
    actors = [one.get("actor") for one in _entries(document, "requirements")]
    unresolved = sorted({str(one) for one in actors if one not in personas})
    unreferenced = sorted(personas - {one for one in actors})
    if unresolved or unreferenced:
        return False, "actors naming no persona: %s; personas named by no requirement: %s" % (
            ", ".join(unresolved) or "none", ", ".join(unreferenced) or "none")
    return True, "%d personas, each named by a requirement; %d actors resolve" % (
        len(personas), len(actors))


def acceptance_recorded(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    """The acceptance the user gave at round 4 is on the document."""
    document, problem = _load_requirements(workspace)
    if document is None:
        return False, problem
    accepted_by = document.get("accepted_by")
    accepted_at = str(document.get("accepted_at") or "")
    status = document.get("status")
    if accepted_by != "user":
        return False, "accepted_by is %r, not user" % accepted_by
    if not RFC3339.match(accepted_at):
        return False, "accepted_at is %r, which is no RFC 3339 UTC timestamp" % accepted_at
    if status != "accepted":
        return False, "document status is %r, not accepted" % status
    unfrozen = [one.get("id") for one in _entries(document, "requirements")
                if one.get("status") not in ("accepted", "withdrawn")]
    if unfrozen:
        return False, "requirements left below accepted: %s" % ", ".join(
            str(one) for one in unfrozen)
    return True, "accepted_by %s, accepted_at %s, status %s, %d requirements frozen" % (
        accepted_by, accepted_at, status, len(_entries(document, "requirements")))
