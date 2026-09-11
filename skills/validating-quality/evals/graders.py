"""Graders for the validating-quality skill.

Every function has the signature

    def <name>(workspace: str, transcript: str, args: dict) -> tuple[bool, str]

and reads only files under `workspace` and the `transcript` string. Standard
library only: no network, no subprocess, no randomness, no YAML package. The
reader below covers the one shape this phase writes — a flat-keyed YAML mapping
whose payload is sequences of mappings — and it keeps key order, because two of
the five graders compare key order rather than key membership.
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


CATEGORIES = [
    "standards",
    "anti-pattern",
    "constraint",
    "coverage",
    "dead-code",
    "deferral",
    "ac-compliance",
    "spec-gap",
    "security",
    "complexity",
    "duplication",
]

SEVERITIES = ["blocker", "high", "medium", "low"]

DISPOSITIONS = ["fix", "defer", "accept"]

HANDOFF_LABEL_WIDTH = 10
HANDOFF_BLOCK_LINES = 12

PHASE_LINE_RE = re.compile(r"^Phase {5}\S")
AC_ID_RE = re.compile(r"^AC-[0-9]{3}$")
TARGET_ID_RE = re.compile(r"^(STORY|ADR)-[0-9]{3}$")


# --------------------------------------------------------------------------
# YAML reading
# --------------------------------------------------------------------------


def _strip_comment(line):
    """Drop a trailing comment, leaving text inside quotes alone."""
    out = []
    quote = ""
    i = 0
    while i < len(line):
        ch = line[i]
        if quote:
            out.append(ch)
            if ch == quote:
                quote = ""
        elif ch in "\"'":
            quote = ch
            out.append(ch)
        elif ch == "#" and (i == 0 or line[i - 1] in " \t"):
            break
        else:
            out.append(ch)
        i += 1
    return "".join(out).rstrip()


def _scalar(text):
    text = text.strip()
    if text == "":
        return ""
    if len(text) >= 2 and text[0] == text[-1] and text[0] in "\"'":
        return text[1:-1]
    if text.startswith("[") and text.endswith("]"):
        inner = text[1:-1].strip()
        if inner == "":
            return []
        return [_scalar(part) for part in inner.split(",")]
    if text in ("true", "True"):
        return True
    if text in ("false", "False"):
        return False
    if text in ("null", "~"):
        return None
    try:
        return int(text)
    except ValueError:
        pass
    try:
        return float(text)
    except ValueError:
        pass
    return text


def _rows(text):
    rows = []
    for raw in text.splitlines():
        stripped = _strip_comment(raw)
        if stripped.strip() == "" or stripped.strip() == "---":
            continue
        indent = len(stripped) - len(stripped.lstrip(" "))
        rows.append((indent, stripped.strip()))
    return rows


def _parse_block(rows, start, indent):
    """Parse the rows at `indent` starting at `start`; return (value, next)."""
    if start >= len(rows):
        return {}, start
    if rows[start][1].startswith("- "):
        return _parse_sequence(rows, start, indent)
    if rows[start][1] == "-":
        return _parse_sequence(rows, start, indent)
    return _parse_mapping(rows, start, indent)


def _parse_mapping(rows, start, indent):
    out = {}
    i = start
    while i < len(rows):
        cur_indent, text = rows[i]
        if cur_indent < indent:
            break
        if cur_indent > indent:
            i += 1
            continue
        if text.startswith("- "):
            break
        if ":" not in text:
            i += 1
            continue
        key, _, rest = text.partition(":")
        key = key.strip()
        rest = rest.strip()
        if rest != "":
            out[key] = _scalar(rest)
            i += 1
            continue
        j = i + 1
        if j < len(rows) and rows[j][0] > indent:
            value, i = _parse_block(rows, j, rows[j][0])
            out[key] = value
        else:
            out[key] = {}
            i = j
    return out, i


def _parse_sequence(rows, start, indent):
    out = []
    i = start
    while i < len(rows):
        cur_indent, text = rows[i]
        if cur_indent != indent or not (text == "-" or text.startswith("- ")):
            break
        body = text[2:].strip() if text.startswith("- ") else ""
        if body == "":
            j = i + 1
            if j < len(rows) and rows[j][0] > indent:
                value, i = _parse_block(rows, j, rows[j][0])
                out.append(value)
            else:
                out.append("")
                i = j
            continue
        if ":" in body and not body.startswith(("\"", "'", "[")):
            key, _, rest = body.partition(":")
            entry = {}
            entry[key.strip()] = _scalar(rest)
            item_indent = indent + 2
            i += 1
            while i < len(rows) and rows[i][0] >= item_indent and not rows[i][1].startswith("- "):
                if rows[i][0] != item_indent:
                    i += 1
                    continue
                sub_key, _, sub_rest = rows[i][1].partition(":")
                sub_key = sub_key.strip()
                sub_rest = sub_rest.strip()
                if sub_rest != "":
                    entry[sub_key] = _scalar(sub_rest)
                    i += 1
                else:
                    j = i + 1
                    if j < len(rows) and rows[j][0] > item_indent:
                        value, i = _parse_block(rows, j, rows[j][0])
                        entry[sub_key] = value
                    else:
                        entry[sub_key] = {}
                        i = j
            out.append(entry)
            continue
        out.append(_scalar(body))
        i += 1
    return out, i


def _load_yaml(path):
    with open(path, "r", encoding="utf-8") as handle:
        text = handle.read()
    rows = _rows(text)
    if not rows:
        return {}
    value, _ = _parse_block(rows, 0, rows[0][0])
    return value


def _read_report(workspace, rel_path):
    full = os.path.join(workspace, rel_path.replace("/", os.sep))
    if not os.path.isfile(full):
        return None, "%s is absent from the workspace" % rel_path
    return _load_yaml(full), ""


def _sha256(path):
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        digest.update(handle.read())
    return digest.hexdigest()


def _digests_hold(workspace, mapping):
    for rel_path, expected in (mapping or {}).items():
        full = os.path.join(workspace, rel_path.replace("/", os.sep))
        if not os.path.isfile(full):
            return "%s is absent; the case set it up" % rel_path
        actual = _sha256(full)
        if actual != str(expected).lower():
            return "%s digest is %s, the case set up %s" % (rel_path, actual, expected)
    return ""


# --------------------------------------------------------------------------
# Handoff reading
# --------------------------------------------------------------------------


def _handoff_block(transcript):
    lines = transcript.splitlines()
    starts = [i for i, line in enumerate(lines) if PHASE_LINE_RE.match(line)]
    if not starts:
        return []
    start = starts[-1]
    return lines[start:start + HANDOFF_BLOCK_LINES]


def _handoff_field(block, label):
    for line in block:
        if line.startswith(label) and line[:HANDOFF_LABEL_WIDTH].strip() == label:
            return line[HANDOFF_LABEL_WIDTH:].strip()
    return None


def _transcript_has_next(transcript, value):
    wanted = "Next".ljust(HANDOFF_LABEL_WIDTH) + value
    for line in transcript.splitlines():
        if line.rstrip() == wanted:
            return True
    return False


# --------------------------------------------------------------------------
# Graders
# --------------------------------------------------------------------------


def qa_report_shape(workspace, transcript, args):
    report, problem = _read_report(workspace, args["report"])
    if problem:
        return False, problem

    keys = list(report.keys())
    if keys != list(args["top_keys"]):
        return False, "top-level keys are %s, expected %s" % (keys, args["top_keys"])

    if report.get("mode") != args["mode"]:
        return False, "mode is %r, expected %r" % (report.get("mode"), args["mode"])

    checks = report.get("checks") or []
    names = sorted(entry.get("name") for entry in checks)
    if names != sorted(args["check_names"]):
        return False, "checks names are %s, expected %s" % (names, sorted(args["check_names"]))

    coverage = report.get("coverage") or {}
    if float(coverage.get("overall", -1)) != float(args["build_overall"]):
        return False, "coverage.overall is %r, the build report holds %r" % (
            coverage.get("overall"), args["build_overall"])

    for entry in report.get("findings") or []:
        for field, allowed in (("category", CATEGORIES), ("severity", SEVERITIES),
                               ("disposition", DISPOSITIONS)):
            if entry.get(field) not in allowed:
                return False, "%s %s is %r, outside %s" % (
                    entry.get("id"), field, entry.get(field), allowed)

    problem = _digests_hold(workspace, args.get("source_sha256"))
    if problem:
        return False, problem

    if not _transcript_has_next(transcript, args["next_line"]):
        return False, "no handoff Next line reads %r" % args["next_line"]

    return True, "%d top-level keys in order, %d checks, coverage.overall %s, Next %s" % (
        len(keys), len(checks), coverage.get("overall"), args["next_line"])


def deep_mode_checks(workspace, transcript, args):
    report, problem = _read_report(workspace, args["report"])
    if problem:
        return False, problem

    if report.get("mode") != "deep":
        return False, "mode is %r, expected 'deep'" % report.get("mode")

    checks = report.get("checks") or []
    names = sorted(entry.get("name") for entry in checks)
    if names != sorted(args["check_names"]):
        return False, "checks names are %s, expected %s" % (names, sorted(args["check_names"]))

    for entry in report.get("findings") or []:
        category = entry.get("category")
        if category == "security" and entry.get("owasp") not in args["owasp_enum"]:
            return False, "%s owasp is %r, outside the ten categories" % (
                entry.get("id"), entry.get("owasp"))
        if category == "complexity":
            if entry.get("measured") is None:
                return False, "%s carries no measured value" % entry.get("id")
            if float(entry.get("limit", -1)) != float(args["complexity_max"]):
                return False, "%s limit is %r, config.toml holds %r" % (
                    entry.get("id"), entry.get("limit"), args["complexity_max"])
        if category == "duplication":
            if entry.get("measured") is None:
                return False, "%s carries no measured value" % entry.get("id")
            if float(entry.get("limit", -1)) != float(args["duplication_max_percent"]):
                return False, "%s limit is %r, config.toml holds %r" % (
                    entry.get("id"), entry.get("limit"), args["duplication_max_percent"])

    # EVL-010: the stream reader renders a subagent call as
    # `Agent(<subagent_type> <prompt>)`, so this is an assertion over a tool_use
    # event rather than over a name the model happened to type.
    for name in args["subagents"]:
        if ("Agent(%s " % name) not in transcript and                 ("Agent(%s)" % name) not in transcript:
            return False, "the transcript holds no Agent call for %s" % name

    return True, "deep mode with %d checks and %d findings, three deep subagents named" % (
        len(checks), len(report.get("findings") or []))


def sendback_block(workspace, transcript, args):
    block = _handoff_block(transcript)
    if not block:
        return False, "the transcript holds no handoff block"

    gate = _handoff_field(block, "Gate")
    wanted_gate = "SEND BACK to " + args["to"]
    if gate is None or wanted_gate not in gate:
        return False, "Gate line is %r, expected %r" % (gate, wanted_gate)

    nxt = _handoff_field(block, "Next")
    if nxt is None or not nxt.startswith(args["next_prefix"]):
        return False, "Next line is %r, expected it to start %r" % (nxt, args["next_prefix"])

    then = _handoff_field(block, "Then")
    if then != args["then_line"]:
        return False, "Then line is %r, expected %r" % (then, args["then_line"])

    joined = "\n".join(block)
    for banned in args.get("forbidden_substrings") or []:
        if banned in joined:
            return False, "the handoff block holds %r" % banned

    report, problem = _read_report(workspace, args["report"])
    if problem:
        return False, problem

    findings = report.get("findings") or []

    if args.get("blocker_relates_to"):
        matches = [f for f in findings
                   if f.get("severity") == "blocker"
                   and f.get("relates_to") == args["blocker_relates_to"]]
        if not matches:
            return False, "no blocker finding cites %s" % args["blocker_relates_to"]
        for entry in matches:
            if entry.get("disposition") != args.get("blocker_disposition"):
                return False, "%s disposition is %r, expected %r" % (
                    entry.get("id"), entry.get("disposition"), args.get("blocker_disposition"))
        blocker_ids = [f.get("id") for f in findings if f.get("severity") == "blocker"]
        listed = report.get("blockers") or []
        if list(listed) != blocker_ids:
            return False, "blockers holds %s, the blocker findings are %s" % (listed, blocker_ids)

    if args.get("require_category"):
        matches = [f for f in findings if f.get("category") == args["require_category"]]
        if not matches:
            return False, "no finding carries category %r" % args["require_category"]
        if not any(AC_ID_RE.match(str(f.get("relates_to") or "")) for f in matches):
            return False, "no %r finding cites an AC-nnn" % args["require_category"]

    for key in ("story_sha256", "source_sha256"):
        problem = _digests_hold(workspace, args.get(key))
        if problem:
            return False, problem

    held, why = _phase_report_not_pass(workspace, args.get("report_phase", "verify"))
    if not held:
        return False, why

    return True, "Gate %r, Next %r, Then %r, %d findings, %s" % (
        gate, nxt, then, len(findings), why)


def deferral_recorded(workspace, transcript, args):
    report, problem = _read_report(workspace, args["report"])
    if problem:
        return False, problem

    deferrals = report.get("deferrals") or []
    if not deferrals:
        return False, "the deferrals list is empty"

    findings = report.get("findings") or []
    deferred_ids = [f.get("id") for f in findings if f.get("disposition") == "defer"]

    for entry in deferrals:
        keys = list(entry.keys())
        if keys != list(args["entry_keys"]):
            return False, "entry %r keys are %s, expected %s" % (
                entry.get("id"), keys, args["entry_keys"])
        if entry.get("reason") not in args["reason_enum"]:
            return False, "entry %r reason is %r, outside the enum" % (
                entry.get("id"), entry.get("reason"))
        if entry.get("story") != args["story"]:
            return False, "entry %r story is %r, expected %r" % (
                entry.get("id"), entry.get("story"), args["story"])
        if entry.get("opened_on") != report.get("verified_on"):
            return False, "entry %r opened_on is %r, verified_on is %r" % (
                entry.get("id"), entry.get("opened_on"), report.get("verified_on"))
        if args.get("con_or_ap") is not None and entry.get("con_or_ap") != args["con_or_ap"]:
            return False, "entry %r con_or_ap is %r, expected %r" % (
                entry.get("id"), entry.get("con_or_ap"), args["con_or_ap"])
        if entry.get("id") not in deferred_ids:
            return False, "entry %r matches no finding at disposition defer" % entry.get("id")
        target = str(entry.get("target") or "")
        if not TARGET_ID_RE.match(target):
            return False, "entry %r target is %r" % (entry.get("id"), target)
        if args.get("target_must_exist"):
            prefix = target.split("-")[0]
            folder = "stories" if prefix == "STORY" else "adr"
            path = os.path.join(workspace, ".devforgeai", folder, target + ".md")
            if not os.path.isfile(path):
                return False, "entry %r target %s resolves to no document" % (
                    entry.get("id"), target)

    return True, "%d deferral entries, keys in order, targets resolving" % len(deferrals)


def cycle_detected(workspace, transcript, args):
    report, problem = _read_report(workspace, args["report"])
    if problem:
        return False, problem
    other, problem = _read_report(workspace, args["other_report"])
    if problem:
        return False, problem

    findings = report.get("findings") or []
    blocking = [f for f in findings
                if f.get("category") == "deferral" and f.get("severity") == "blocker"]
    if not blocking:
        return False, "no finding carries category deferral at severity blocker"

    basename = args["other_report"].rsplit("/", 1)[-1]
    if not any(basename in str(f.get("evidence") or "") for f in blocking):
        return False, "no blocking deferral finding names %s in its evidence" % basename

    for entry in report.get("deferrals") or []:
        if entry.get("target") == args["forbidden_target"]:
            return False, "deferral %r still targets %s" % (
                entry.get("id"), args["forbidden_target"])

    block = _handoff_block(transcript)
    if not block:
        return False, "the transcript holds no handoff block"

    gate = _handoff_field(block, "Gate")
    if gate is None or not any(name in gate for name in args["gate_names"]):
        return False, "Gate line is %r, expected one of %s" % (gate, args["gate_names"])

    nxt = _handoff_field(block, "Next")
    if nxt is None or not nxt.startswith(args["next_prefix"]):
        return False, "Next line is %r, expected it to start %r" % (nxt, args["next_prefix"])

    cited = [f for f in findings if f.get("id") == args["cited"]]
    if not cited:
        return False, "the report holds no finding %s" % args["cited"]
    if cited[0].get("disposition") == args["prior_disposition"]:
        return False, "%s still carries disposition %r" % (
            args["cited"], args["prior_disposition"])

    return True, "cycle reported by %s, Gate %r, Next %r, %s re-disposed to %r" % (
        blocking[0].get("id"), gate, nxt, args["cited"], cited[0].get("disposition"))
