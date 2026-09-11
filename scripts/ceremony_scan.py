#!/usr/bin/env python3
"""The `specs/00-conventions.md` section 2 ceremony rule, implemented once.

Usage
-----

    python scripts/ceremony_scan.py                     # the default file set
    python scripts/ceremony_scan.py skills/explore agents/idea-interrogator.md
    python scripts/ceremony_scan.py --paths skills agents

With no argument the default set is every `skills/**/SKILL.md`, every
`skills/**/references/*.md`, and every `agents/*.md` below the repository root.
A positional argument or a `--paths` value may be a file or a directory; a
directory is walked for `*.md`.

Output is one `file:line: token` line per hit on stdout. Exit 0 when the scan
found no hit, 1 when it found one, 2 on a usage error.

The rule
--------

Two patterns, both matched case-insensitively. The capitalised form is what the
rule is aimed at -- current models over-trigger on it -- and the ordinary
lower-case English uses of the same words are released by the six clauses below
rather than by case. Scoping is the sturdier of the two mechanisms: a backticked
span, an enum cell, or a YAML scalar is out of scope whichever case it carries,
while a case rule lets `You must never skip this` through.

A line is instruction prose, and therefore in scope, when none of these holds,
tested in this order:

1. The line is inside a fenced block, at any fence depth.
2. The line is inside YAML frontmatter, or is a YAML or JSON scalar, key, or
   enum value.
3. The token that matched lies inside a backticked span.
4. The token that matched lies inside a table cell whose column holds enum
   values, identifiers, paths, or file names, as declared by that table's
   header row.
5. The token that matched is part of a path segment or a file name.
6. The line is a heading.

Two of the six are decided by heuristic, and the heuristic is stated here
rather than left for a reader to infer:

* Clause 2 outside frontmatter recognises a YAML mapping line by a lower-case
  key: `^\\s*(- )?[a-z_][a-z0-9_.-]*:\\s`. That is how this framework writes
  YAML, and it is not how it writes a sentence, so `Note: the model MUST ...`
  stays in scope while `priority: must` does not.
* Clause 4 reads the table's header row and exempts a cell whose column header
  is one of the declared data-column names in `DATA_COLUMNS`. A table with no
  header row above it exempts no cell.

The pattern is written out here and nowhere else. `specs/00-conventions.md`
states the rule in prose and names this script; `specs/BUILD-BRIEF.md` calls
it; a grader that needs the check calls it.
"""

import argparse
import pathlib
import re
import sys

SINGLE = re.compile(r"\b(MUST|ALWAYS|NEVER|SHALL|CRITICAL|IMPORTANT|MANDATORY)\b",
                    re.IGNORECASE)
PHRASE = re.compile(
    r"\b(verify that|ensure that|confirm that|do not skip|before proceeding|"
    r"self-check|checklist)\b",
    re.IGNORECASE,
)

# Clause 4: a column whose header is one of these holds data rather than prose.
DATA_COLUMNS = {
    "arm", "category", "check", "code", "column", "command", "detector",
    "detector kind", "enum", "event", "field", "fields", "file", "files",
    "flag", "form", "glob", "id", "ids", "key", "keys", "kind", "matcher",
    "mode", "name", "names", "origin", "path", "paths", "prefix", "priority",
    "scope", "severity", "source", "status", "subcommand", "tool", "tools",
    "type", "value", "values", "verifier",
}

# Clause 5: a token sitting next to one of these is part of a path or a name.
PATH_NEIGHBOUR = re.compile(r"[/\\.]")

DEFAULT_GLOBS = ("skills/*/SKILL.md", "skills/*/references/*.md", "agents/*.md")


def mask_backticks(line: str) -> str:
    """Clause 3: blank every backticked span, keeping the line's length."""
    out = list(line)
    i = 0
    n = len(line)
    while i < n:
        if line[i] == "`":
            run = 1
            while i + run < n and line[i + run] == "`":
                run += 1
            fence = "`" * run
            close = line.find(fence, i + run)
            if close == -1:
                i += run
                continue
            for j in range(i, close + run):
                out[j] = " "
            i = close + run
            continue
        i += 1
    return "".join(out)


def is_yaml_line(line: str) -> bool:
    """Clause 2 outside frontmatter: a lower-case YAML mapping key."""
    return re.match(r"^\s*(?:-\s+)?[a-z_][a-z0-9_.-]*:\s", line) is not None


def cells_of(row: str):
    """The cells of a markdown table row, with each cell's span in the line."""
    spans = []
    start = None
    for i, ch in enumerate(row):
        if ch == "|":
            if start is not None:
                spans.append((start, i))
            start = i + 1
    if start is not None and start < len(row):
        spans.append((start, len(row)))
    return spans


def exempt_columns(header: str):
    """The indices of the header row's data columns."""
    out = set()
    for idx, (a, b) in enumerate(cells_of(header)):
        if header[a:b].strip().strip("`").lower() in DATA_COLUMNS:
            out.add(idx)
    return out


def token_is_path(line: str, start: int, end: int) -> bool:
    """Clause 5: the match is a path segment or part of a file name.

    A separator on either side settles it. A dot settles it only when it joins
    the token to another word — `tests/MUST.md`, `a.MUST` — so a sentence that
    ends on the token is prose, not a file name.
    """
    prev = line[start - 1] if start > 0 else ""
    nxt = line[end] if end < len(line) else ""
    if (prev and prev in "/\\") or (nxt and nxt in "/\\"):
        return True
    if prev == "." and start >= 2 and line[start - 2].isalnum():
        return True
    if nxt == "." and end + 1 < len(line) and line[end + 1].isalnum():
        return True
    return False


def scan_text(text: str, name: str):
    """Every hit in one file, as `(line_number, token)` pairs."""
    hits = []
    lines = text.splitlines()
    in_fence = False
    fence_mark = ""
    in_front = False
    header = None          # the most recent table header row
    header_dashes = False  # whether its `|---|` separator has been seen

    for no, raw in enumerate(lines, start=1):
        stripped = raw.strip()

        # Clause 2, the frontmatter half.
        if no == 1 and stripped == "---":
            in_front = True
            continue
        if in_front:
            if stripped == "---":
                in_front = False
            continue

        # Clause 1.
        fence = re.match(r"^\s*(`{3,}|~{3,})", raw)
        if fence:
            mark = fence.group(1)[0] * 3
            if not in_fence:
                in_fence, fence_mark = True, mark
            elif mark == fence_mark:
                in_fence, fence_mark = False, ""
            continue
        if in_fence:
            continue

        # Clause 6.
        if re.match(r"^\s{0,3}#{1,6}\s", raw):
            continue

        # Table bookkeeping for clause 4.
        if stripped.startswith("|"):
            if re.match(r"^\s*\|[\s:|-]+\|?\s*$", raw):
                header_dashes = header is not None
                continue
            if header is None or not header_dashes:
                header, header_dashes = raw, False
                continue
        else:
            header, header_dashes = None, False

        # Clause 2, the scalar half.
        if is_yaml_line(raw):
            continue

        # Clause 3.
        line = mask_backticks(raw)

        exempt = exempt_columns(header) if (header and header_dashes) else set()
        spans = cells_of(line) if stripped.startswith("|") else []

        for pattern in (SINGLE, PHRASE):
            for m in pattern.finditer(line):
                # Clause 4.
                if spans:
                    idx = next(
                        (i for i, (a, b) in enumerate(spans) if a <= m.start() < b),
                        None,
                    )
                    if idx is not None and idx in exempt:
                        continue
                # Clause 5.
                if token_is_path(line, m.start(), m.end()):
                    continue
                hits.append((no, m.group(0)))

    return [(name, no, tok) for no, tok in hits]


def files_for(targets, root: pathlib.Path):
    out = []
    if not targets:
        for pattern in DEFAULT_GLOBS:
            out.extend(sorted(root.glob(pattern)))
        return out
    for t in targets:
        p = pathlib.Path(t)
        if not p.is_absolute():
            p = root / p
        if p.is_dir():
            out.extend(sorted(p.rglob("*.md")))
        elif p.exists():
            out.append(p)
        else:
            print(f"ceremony_scan: no such path: {t}", file=sys.stderr)
            raise SystemExit(2)
    return out


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Scan instruction prose for the conventions section 2 ceremony pattern."
    )
    ap.add_argument("targets", nargs="*", help="files or directories to scan")
    ap.add_argument(
        "--paths",
        nargs="+",
        default=[],
        metavar="PATH",
        help="files or directories to scan, replacing the default set",
    )
    ap.add_argument(
        "--root",
        default=str(pathlib.Path(__file__).resolve().parent.parent),
        help="the repository root the default set and relative paths resolve against",
    )
    args = ap.parse_args()

    root = pathlib.Path(args.root).resolve()
    targets = list(args.targets) + list(args.paths)
    paths = files_for(targets, root)

    hits = []
    for path in paths:
        try:
            text = path.read_text(encoding="utf-8")
        except OSError as e:
            print(f"ceremony_scan: reading {path}: {e}", file=sys.stderr)
            return 2
        try:
            rel = path.relative_to(root).as_posix()
        except ValueError:
            rel = path.as_posix()
        hits.extend(scan_text(text, rel))

    for name, no, tok in hits:
        print(f"{name}:{no}: {tok}")

    if hits:
        print(f"\n{len(hits)} hit(s) in {len(paths)} file(s)", file=sys.stderr)
        return 1
    print(f"no ceremony in {len(paths)} file(s)", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
