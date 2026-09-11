"""Graders for the releasing-software skill.

Every function has the signature

    def <name>(workspace: str, transcript: str, args: dict) -> tuple[bool, str]

and reads only files under `workspace` and the `transcript` string. Standard
library only -- `hashlib`, `os`, and `re` -- so no network call, no subprocess,
no model, no random source, and no YAML package. The reader below covers the
subset these documents use: ordered mappings, block sequences of mappings and
of scalars, inline flow lists, quoted and bare strings, integers, floats, and
booleans.

Prior state a grader compares against arrives inside `args`. Case 6 carries the
SHA-256 of each manifest as it stood before the run, so an untouched-file
assertion needs no mirror of a previous workspace.
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


GATE_PREFIX = "Gate      "
NEXT_PREFIX = "Next      "
THEN_PREFIX = "Then      "
FOUND_PREFIX = "Found     "

FLOW_LIST_RE = re.compile(r"^\[(.*)\]$")
MAP_HEAD_RE = re.compile(r"^[A-Za-z_][A-Za-z0-9_.-]*:( |$)")
INT_RE = re.compile(r"^-?[0-9]+$")
FLOAT_RE = re.compile(r"^-?[0-9]+\.[0-9]+$")
H2_RE = re.compile(r"^## (.+)$")
H3_RE = re.compile(r"^### (.+)$")
ENV_REF_RE = re.compile(r"\$\{([A-Z0-9_]+)\}")
ENV_LINE_RE = re.compile(r"^([A-Z0-9_]+)=")


# ---------------------------------------------------------------- file paths


def _path(workspace, rel):
    parts = [p for p in str(rel).replace("\\", "/").split("/") if p]
    return os.path.join(workspace, *parts)


def _read(workspace, rel):
    with open(_path(workspace, rel), "r", encoding="utf-8") as handle:
        return handle.read()


def _exists(workspace, rel):
    return os.path.isfile(_path(workspace, rel))


def _non_empty(workspace, rel):
    full = _path(workspace, rel)
    return os.path.isfile(full) and os.path.getsize(full) > 0


def _files_under(workspace, rel):
    root = _path(workspace, rel)
    found = []
    for base, _dirs, names in os.walk(root):
        for name in names:
            full = os.path.join(base, name)
            found.append(os.path.relpath(full, workspace).replace("\\", "/"))
    return sorted(found)


# -------------------------------------------------------------- YAML subset


def _scalar(raw):
    raw = raw.strip()
    if raw == "":
        return ""
    flow = FLOW_LIST_RE.match(raw)
    if flow:
        inner = flow.group(1).strip()
        if not inner:
            return []
        return [_scalar(part) for part in inner.split(",")]
    if len(raw) >= 2 and raw[0] == raw[-1] and raw[0] in "\"'":
        return raw[1:-1]
    if raw in ("true", "false"):
        return raw == "true"
    if raw in ("null", "~"):
        return None
    if INT_RE.match(raw):
        return int(raw)
    if FLOAT_RE.match(raw):
        return float(raw)
    return raw


def _lines(text):
    out = []
    for raw in text.split("\n"):
        stripped = raw.strip()
        if not stripped or stripped.startswith("#"):
            continue
        indent = len(raw) - len(raw.lstrip(" "))
        out.append((indent, stripped))
    return out


def _parse_block(lines, index, indent):
    if index < len(lines) and lines[index][1].startswith("- "):
        return _parse_seq(lines, index, indent)
    return _parse_map(lines, index, indent)


def _parse_map(lines, index, indent):
    out = {}
    while index < len(lines) and lines[index][0] == indent:
        stripped = lines[index][1]
        if stripped.startswith("- "):
            break
        key, _sep, rest = stripped.partition(":")
        key = key.strip()
        rest = rest.strip()
        if rest:
            out[key] = _scalar(rest)
            index += 1
            continue
        child = index + 1
        if child < len(lines) and lines[child][0] > indent:
            value, index = _parse_block(lines, child, lines[child][0])
            out[key] = value
        else:
            out[key] = None
            index += 1
    return out, index


def _parse_seq(lines, index, indent):
    out = []
    while index < len(lines) and lines[index][0] == indent \
            and lines[index][1].startswith("- "):
        head = lines[index][1][2:].strip()
        after = index + 1
        sub = []
        while after < len(lines) and lines[after][0] > indent:
            sub.append(lines[after])
            after += 1
        if MAP_HEAD_RE.match(head):
            block = [(indent + 2, head)] + sub
            value, _ = _parse_map(block, 0, indent + 2)
            out.append(value)
        elif head == "" and sub:
            value, _ = _parse_block(sub, 0, sub[0][0])
            out.append(value)
        else:
            out.append(_scalar(head))
        index = after
    return out, index


def _yaml_load(text):
    body = text
    if body.startswith("---\n"):
        end = body.find("\n---", 3)
        if end != -1:
            body = body[end + 4:]
    lines = _lines(body)
    if not lines:
        return {}
    value, _ = _parse_block(lines, 0, lines[0][0])
    return value


def _load(workspace, rel):
    return _yaml_load(_read(workspace, rel))


def _frontmatter(text):
    lines = text.split("\n")
    if not lines or lines[0].strip() != "---":
        return {}
    out = {}
    index = 1
    while index < len(lines) and lines[index].strip() != "---":
        line = lines[index]
        if line.strip() and not line.startswith((" ", "\t", "#")) and ":" in line:
            key, _sep, rest = line.partition(":")
            out[key.strip()] = _scalar(rest)
        index += 1
    return out


def _get(obj, dotted):
    node = obj
    for part in dotted.split("."):
        if not isinstance(node, dict) or part not in node:
            return None
        node = node[part]
    return node


# ----------------------------------------------------------- markdown parts


def _h2_sequence(text):
    return [H2_RE.match(line).group(1).strip()
            for line in text.split("\n") if H2_RE.match(line)]


def _h3_texts(text):
    return [H3_RE.match(line).group(1).strip()
            for line in text.split("\n") if H3_RE.match(line)]


def _section(text, heading):
    out = []
    collecting = False
    for line in text.split("\n"):
        match = H2_RE.match(line)
        if match:
            collecting = match.group(1).strip() == heading
            continue
        if collecting:
            out.append(line)
    return "\n".join(out)


def _table_rows(section_text):
    rows = []
    for line in section_text.split("\n"):
        stripped = line.strip()
        if not stripped.startswith("|"):
            continue
        cells = [cell.strip() for cell in stripped.strip("|").split("|")]
        if all(re.match(r"^:?-{2,}:?$", cell) for cell in cells if cell):
            continue
        rows.append(cells)
    return rows[1:] if rows else []


# ------------------------------------------------------------- transcript


def _prefixed(transcript, prefix):
    return [line[len(prefix):].strip()
            for line in transcript.split("\n") if line.startswith(prefix)]


def _gate_token(transcript):
    lines = _prefixed(transcript, GATE_PREFIX)
    if not lines:
        return None, None
    return lines[0].split()[0] if lines[0].split() else "", lines[0]


# ------------------------------------------------------------------ graders


def release_shape(workspace, transcript, args):
    """The release file's key order, story set, platform, manifests, signoff."""
    path = args["path"]
    if not _exists(workspace, path):
        return False, "%s does not exist" % path
    obj = _load(workspace, path)
    notes = []
    ok = True

    keys = list(obj.keys())
    notes.append("keys=%s" % ",".join(keys))
    if keys != args["keys"]:
        ok = False
        notes.append("key order differs from %s" % ",".join(args["keys"]))

    if obj.get("status") != args["status"]:
        ok = False
        notes.append("status=%r wanted %r" % (obj.get("status"), args["status"]))

    stem = os.path.basename(path)
    if stem.endswith(".yaml"):
        stem = stem[:-5]
    if obj.get("id") != stem:
        ok = False
        notes.append("id=%r wanted the file stem %r" % (obj.get("id"), stem))

    story_ids = [entry.get("id") for entry in (obj.get("stories") or [])]
    notes.append("stories=%s" % ",".join(str(sid) for sid in story_ids))
    if story_ids != args["stories"]:
        ok = False
        notes.append("story set differs from %s" % ",".join(args["stories"]))

    target = _get(obj, "platform.target")
    source = _get(obj, "platform.source")
    if target != args["platform_target"] or source != args["platform_source"]:
        ok = False
        notes.append("platform=%r/%r wanted %r/%r"
                     % (target, source, args["platform_target"],
                        args["platform_source"]))

    manifests = _get(obj, "deploy.manifests") or []
    basenames = sorted({os.path.basename(str(entry.get("path")))
                        for entry in manifests})
    notes.append("manifests=%s" % ",".join(basenames))
    for wanted in args["manifest_basenames"]:
        if wanted not in basenames:
            ok = False
            notes.append("%s is absent from deploy.manifests" % wanted)
    for entry in manifests:
        rel = entry.get("path")
        if not _non_empty(workspace, rel):
            ok = False
            notes.append("%s is absent or empty" % rel)

    if _get(obj, "signoff.gate") != args["signoff_gate"]:
        ok = False
        notes.append("signoff.gate=%r wanted %r"
                     % (_get(obj, "signoff.gate"), args["signoff_gate"]))
    passed = _get(obj, "signoff.checks_passed")
    total = _get(obj, "signoff.checks_total")
    if passed != total:
        ok = False
        notes.append("signoff %s of %s checks passed" % (passed, total))

    return ok, "; ".join(notes)


def platform_resolution(workspace, transcript, args):
    """The three platform values, the written files, the variable pairing."""
    path = args["path"]
    if not _exists(workspace, path):
        return False, "%s does not exist" % path
    obj = _load(workspace, path)
    notes = []
    ok = True

    target = _get(obj, "platform.target")
    source = _get(obj, "platform.source")
    marker = _get(obj, "platform.marker")
    notes.append("platform=%r/%r/%r" % (target, source, marker))
    if target != args["target"]:
        ok = False
        notes.append("target wanted %r" % args["target"])
    if source != args["source"]:
        ok = False
        notes.append("source wanted %r" % args["source"])
    if marker != args["marker"]:
        ok = False
        notes.append("marker wanted %r" % args["marker"])

    for rel in args.get("required_files", []):
        if not _non_empty(workspace, rel):
            ok = False
            notes.append("%s is absent or empty" % rel)

    pairs = args.get("env_pairs")
    if pairs:
        compose_rel = pairs["compose"]
        env_rel = pairs["env"]
        if _exists(workspace, compose_rel) and _exists(workspace, env_rel):
            referenced = set(ENV_REF_RE.findall(_read(workspace, compose_rel)))
            declared = set()
            for line in _read(workspace, env_rel).split("\n"):
                match = ENV_LINE_RE.match(line.strip())
                if match:
                    declared.add(match.group(1))
            notes.append("referenced=%s" % ",".join(sorted(referenced)))
            notes.append("declared=%s" % ",".join(sorted(declared)))
            missing = sorted(referenced - declared)
            if missing:
                ok = False
                notes.append("no env line for %s" % ",".join(missing))
        else:
            ok = False
            notes.append("the compose pair is not both present")

    return ok, "; ".join(notes)


def docs_layout(workspace, transcript, args):
    """The documentation set exists, the empty keys are empty, headings match."""
    notes = []
    ok = True

    for rel in args.get("required", []):
        if not _non_empty(workspace, rel):
            ok = False
            notes.append("%s is absent or empty" % rel)
    notes.append("required=%d" % len(args.get("required", [])))

    absent_dir = args.get("absent_dir")
    if absent_dir:
        found = _files_under(workspace, absent_dir)
        if found:
            ok = False
            notes.append("%s holds %s" % (absent_dir, ",".join(found[:5])))
        else:
            notes.append("%s holds no file" % absent_dir)

    path = args["path"]
    if not _exists(workspace, path):
        return False, "; ".join(notes + ["%s does not exist" % path])
    obj = _load(workspace, path)

    for dotted in args.get("empty_keys", []):
        value = _get(obj, dotted)
        if value not in ([], None):
            ok = False
            notes.append("%s=%r wanted []" % (dotted, value))
    for dotted in args.get("empty_strings", []):
        value = _get(obj, dotted)
        if value != "":
            ok = False
            notes.append("%s=%r wanted the empty string" % (dotted, value))

    for rel, headings in (args.get("headings") or {}).items():
        if not _exists(workspace, rel):
            ok = False
            notes.append("%s does not exist" % rel)
            continue
        found = _h2_sequence(_read(workspace, rel))
        notes.append("%s H2=%s" % (rel, ",".join(found)))
        if found != list(headings):
            ok = False
            notes.append("%s heading order wanted %s" % (rel, ",".join(headings)))

    wanted_gate = args.get("transcript_gate")
    if wanted_gate:
        token, line = _gate_token(transcript)
        notes.append("gate line=%r" % line)
        if token != wanted_gate:
            ok = False
            notes.append("gate token wanted %r" % wanted_gate)

    return ok, "; ".join(notes)


def sendback_block(workspace, transcript, args):
    """The handoff routes to Verify, cites the right ids, and moves no story."""
    notes = []
    ok = True

    gate_lines = _prefixed(transcript, GATE_PREFIX)
    gate_line = gate_lines[0] if gate_lines else ""
    notes.append("Gate=%r" % gate_line)
    wanted_gate = "SEND BACK to %s" % args["to"]
    if not gate_line.startswith(wanted_gate):
        ok = False
        notes.append("gate line does not open with %r" % wanted_gate)
    for token in args.get("gate_must_name", []):
        if token not in gate_line:
            ok = False
            notes.append("gate line omits %s" % token)

    next_lines = _prefixed(transcript, NEXT_PREFIX)
    next_line = next_lines[0] if next_lines else ""
    notes.append("Next=%r" % next_line)
    if not next_line.startswith(args["next_prefix"]):
        ok = False
        notes.append("next line does not open with %r" % args["next_prefix"])
    for token in args.get("must_cite", []):
        if token not in next_line:
            ok = False
            notes.append("next line omits %s" % token)
    for token in args.get("forbidden_substrings", []):
        if token in next_line:
            ok = False
            notes.append("next line holds %s" % token)

    then_lines = _prefixed(transcript, THEN_PREFIX)
    then_line = then_lines[0] if then_lines else ""
    notes.append("Then=%r" % then_line)
    if then_line != args["then_line"]:
        ok = False
        notes.append("then line wanted %r" % args["then_line"])

    found_lines = _prefixed(transcript, FOUND_PREFIX)
    if args.get("found_must_name"):
        notes.append("Found=%s" % " | ".join(found_lines))
    for group in args.get("found_must_name", []):
        if not any(all(token in line for token in group) for line in found_lines):
            ok = False
            notes.append("no Found line holds %s" % ",".join(group))

    release_rel = args.get("release_file")
    if release_rel:
        if not _exists(workspace, release_rel):
            ok = False
            notes.append("%s does not exist" % release_rel)
        else:
            gate = _get(_load(workspace, release_rel), "signoff.gate")
            notes.append("signoff.gate=%r" % gate)
            if gate != args["release_signoff"]:
                ok = False
                notes.append("signoff.gate wanted %r" % args["release_signoff"])

    for rel in args.get("stories_not_released", []):
        if not _exists(workspace, rel):
            ok = False
            notes.append("%s does not exist" % rel)
            continue
        status = _frontmatter(_read(workspace, rel)).get("status")
        notes.append("%s status=%r" % (rel, status))
        if status == "released":
            ok = False
            notes.append("%s moved to released" % rel)

    held, why = _phase_report_not_pass(workspace, args.get("report_phase", "release"))
    notes.append(why)
    if not held:
        ok = False

    return ok, "; ".join(notes)


def resume_untouched(workspace, transcript, args):
    """The manifests keep their bytes and the rerun closes at PASS."""
    notes = []
    ok = True

    for rel, digest in (args.get("baseline_sha256") or {}).items():
        full = _path(workspace, rel)
        if not os.path.isfile(full):
            ok = False
            notes.append("%s is absent" % rel)
            continue
        with open(full, "rb") as handle:
            current = hashlib.sha256(handle.read()).hexdigest()
        if current != digest:
            ok = False
            notes.append("%s digest %s wanted %s" % (rel, current[:12], digest[:12]))
        else:
            notes.append("%s unchanged" % rel)

    path = args["path"]
    if not _exists(workspace, path):
        return False, "; ".join(notes + ["%s does not exist" % path])
    obj = _load(workspace, path)

    if args.get("no_blocking_deferral"):
        # AGT-014: a deferral finding carries no `blocks_deployment` boolean.
        # `kind: blocks_deployment` at `severity: block` says it once, and an
        # entry that ships carries `kind: accepted` at `severity: info`.
        blocking = [entry.get("find")
                    for entry in (_get(obj, "notes.deferred") or [])
                    if entry.get("kind") == "blocks_deployment"
                    or entry.get("severity") == "block"]
        notes.append("blocking deferrals=%s" % (",".join(str(b) for b in blocking) or "none"))
        if blocking:
            ok = False

    wanted_gate = args.get("transcript_gate")
    if wanted_gate:
        token, line = _gate_token(transcript)
        notes.append("Gate=%r" % line)
        if token != wanted_gate:
            ok = False
            notes.append("gate token wanted %r" % wanted_gate)

    wanted_next = args.get("next_line")
    if wanted_next:
        next_lines = _prefixed(transcript, NEXT_PREFIX)
        next_line = next_lines[0] if next_lines else ""
        notes.append("Next=%r" % next_line)
        if next_line != wanted_next:
            ok = False
            notes.append("next line wanted %r" % wanted_next)

    return ok, "; ".join(notes)


def version_refused(workspace, transcript, args):
    """A version at or below the previous one writes nothing."""
    notes = []
    ok = True

    for rel in args.get("absent", []):
        if _exists(workspace, rel):
            ok = False
            notes.append("%s was written" % rel)
        else:
            notes.append("%s absent" % rel)

    for rel in args.get("absent_dirs", []):
        found = _files_under(workspace, rel)
        if found:
            ok = False
            notes.append("%s holds %s" % (rel, ",".join(found[:5])))
        else:
            notes.append("%s holds no file" % rel)

    for token in args.get("transcript_must_contain", []):
        if token not in transcript:
            ok = False
            notes.append("the run never named %s" % token)

    pass_lines = [line for line in transcript.split("\n")
                  if line.startswith(GATE_PREFIX + "PASS")]
    if pass_lines:
        ok = False
        notes.append("a PASS gate line was printed: %r" % pass_lines[0])
    else:
        notes.append("no PASS gate line")

    return ok, "; ".join(notes)


def api_coverage(workspace, transcript, args):
    """Every enumerated symbol has an H3 on an API page and a row in the index."""
    path = args["path"]
    if not _exists(workspace, path):
        return False, "%s does not exist" % path
    obj = _load(workspace, path)
    notes = []
    ok = True

    symbols = _get(obj, "docs.api_symbols")
    documented = _get(obj, "docs.api_documented")
    notes.append("api_symbols=%r api_documented=%r" % (symbols, documented))
    if symbols != args["expect_symbols"]:
        ok = False
        notes.append("api_symbols wanted %s" % args["expect_symbols"])
    if documented != args["expect_documented"]:
        ok = False
        notes.append("api_documented wanted %s" % args["expect_documented"])

    index_rel = args["index"]
    if not _exists(workspace, index_rel):
        ok = False
        notes.append("%s does not exist" % index_rel)
    else:
        rows = _table_rows(_section(_read(workspace, index_rel), "Symbols"))
        notes.append("index rows=%d" % len(rows))
        if len(rows) != args["expect_symbols"]:
            ok = False
            notes.append("index row count wanted %s" % args["expect_symbols"])

    headings = set()
    pages = [rel for rel in (_get(obj, "docs.api") or [])
             if os.path.basename(str(rel)) != "index.md"]
    for rel in pages:
        if _exists(workspace, rel):
            headings.update(_h3_texts(_read(workspace, rel)))
        else:
            ok = False
            notes.append("%s does not exist" % rel)
    missing = [name for name in args["symbols"] if name not in headings]
    notes.append("pages=%d headings=%d" % (len(pages), len(headings)))
    if missing:
        ok = False
        notes.append("no H3 for %s" % ",".join(missing))

    for rel in args.get("brand", []):
        if not _non_empty(workspace, rel):
            ok = False
            notes.append("%s is absent or empty" % rel)

    return ok, "; ".join(notes)


def release_notes(workspace, transcript, args):
    """The derived kinds, the requirement union, and the summary length."""
    path = args["path"]
    if not _exists(workspace, path):
        return False, "%s does not exist" % path
    obj = _load(workspace, path)
    notes = []
    ok = True

    stories = obj.get("stories") or []
    entries = _get(obj, "notes.entries") or []
    notes.append("stories=%d entries=%d" % (len(stories), len(entries)))
    if len(entries) != len(stories):
        ok = False
        notes.append("notes.entries and stories differ in length")

    for index, entry in enumerate(entries):
        story_id = entry.get("story")
        if index < len(stories) and story_id != stories[index].get("id"):
            ok = False
            notes.append("entry %d names %r, stories names %r"
                         % (index, story_id, stories[index].get("id")))

        kind = entry.get("kind")
        if kind not in ("feature", "fix", "internal"):
            ok = False
            notes.append("%s kind=%r is outside the enum" % (story_id, kind))

        requirements = entry.get("requirements") or []
        story_rel = ".devforgeai/stories/%s.md" % story_id
        consumes = []
        if _exists(workspace, story_rel):
            consumes = _frontmatter(_read(workspace, story_rel)).get("consumes") or []
        has_find = any(str(cid).startswith("FIND-") for cid in consumes)
        if not requirements:
            wanted = "internal"
        elif has_find:
            wanted = "fix"
        else:
            wanted = "feature"
        notes.append("%s kind=%r rule=%r" % (story_id, kind, wanted))
        if kind != wanted:
            ok = False
            notes.append("%s kind wanted %r" % (story_id, wanted))

        expected = (args.get("expect_kinds") or {}).get(story_id)
        if expected and kind != expected:
            ok = False
            notes.append("%s kind wanted %r by the case" % (story_id, expected))

    union = []
    for entry in entries:
        for rid in entry.get("requirements") or []:
            if rid not in union:
                union.append(rid)
    union.sort()
    listed = [row.get("id") for row in (_get(obj, "notes.requirements") or [])]
    notes.append("union=%s listed=%s" % (",".join(union), ",".join(str(r) for r in listed)))
    if listed != union:
        ok = False
        notes.append("notes.requirements is not the ascending de-duplicated union")
    if args.get("expect_requirements") and listed != args["expect_requirements"]:
        ok = False
        notes.append("notes.requirements wanted %s"
                     % ",".join(args["expect_requirements"]))

    summary = _get(obj, "notes.summary") or ""
    notes.append("summary=%d chars" % len(summary))
    if len(summary) > args.get("summary_max", 72):
        ok = False
        notes.append("summary is longer than %s characters" % args.get("summary_max", 72))

    return ok, "; ".join(notes)
