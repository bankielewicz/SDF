#!/usr/bin/env python3
"""Shared eval runner for DevForgeAI skills.

Reads a skill's ``evals/cases.jsonl``, builds one temporary workspace per case,
runs ``claude -p`` inside it with the skill under test installed, calls the
grader the case names, and appends one line per case to ``results.jsonl``.

Scope: this runner and the graders it calls evaluate skills alone. No phase gate
reads its output, no ``gates.toml`` check kind names it, and no ``devforgeai``
subcommand invokes it. A case passes or fails by the return value of the grader
function in the skill's ``graders.py``; the runner records what the grader
returned and nothing here decides a gate.

What the child sees (AUDIT-5 EVL-010 to EVL-014, EVL-051):

* the release binary's directory is prepended to ``PATH``, because every skill
  preamble opens with ``!`devforgeai ...``` and a non-zero exit there aborts the
  whole skill invocation;
* the prompt travels on stdin, not in argv, because on Windows ``claude`` is an
  npm ``.cmd`` shim and an argument carrying newlines does not survive
  ``cmd.exe``;
* the output format is ``stream-json --verbose --include-partial-messages``, so
  the transcript a grader sees carries tool calls, tool inputs and tool results
  and not only the final ``result`` string;
* the redirected ``CLAUDE_CONFIG_DIR`` sits beside the workspaces too, so
  ``devforgeai commit`` does not see its ``.claude.json`` as an untracked change;
* a git-seeded workspace carries a ``.gitignore`` for what the stand-in commands
  write, so a commit is not refused over a coverage report;
* the raw streams are written to a log directory beside the workspaces, never
  inside one, so no grader's tree walk counts them as a write, and stdout is
  written line by line as it arrives, so a case killed at its timeout still
  carries the evidence of where it reached;
* a seeded ``config.toml`` is completed from the one ``init`` writes, table by
  table — a stated table replaces the default's, except ``[[verifier]]``, which
  merges by ``name``, so every workspace carries the full twenty-name catalogue
  the gates read while a fixture can still change one row;
* ``config.toml`` is put into the binary's canonical form before the run, so the
  SessionStart ``stack detect`` changes nothing the tamper guard can see;
* every skill the skill under test reaches through the Skill tool is installed
  beside it, under its own frontmatter name.

Slash names collide, and the workspace is the only defence. A skill installed at
``.claude/skills/<name>/`` takes the slash name ``<name>``, and a user-level or
plugin skill of the same name on the machine running the eval shadows or blocks
it. Measured: the hooks-on ``ex-01`` run carried only ``.claude/skills/explore/``
and its step 6 asked the Skill tool for ``design``; Claude Code resolved that to
a different ``design`` on this machine and refused — "Skill design cannot be used
with Skill tool due to disable-model-invocation" — and the run stalled. Installing
the invoked skill into the workspace puts the framework's own copy first. It does
not make the eval immune to a collision on the *outer* name, which is a property
of the machine.

Hooks during an eval (EVL-003). By default the workspace settings carry the
framework's own hook block from ``hooks/settings.hooks.json`` with
``@@DEVFORGEAI@@`` resolved to the binary, so the producer check, the gates and
the Stop block run exactly as in a target project. That depends on a trust pin
a human made outside Claude Code: the release binary reads
``~/.devforgeai/trust.toml`` and honours ``DEVFORGEAI_HOME`` only under the
``test-home`` cargo feature, which a release build does not carry. The runner
therefore runs ``devforgeai trust verify`` once before the suite; when it fails
it falls back to ``--no-hooks`` and prints the pin command, unless ``--hooks``
was passed explicitly, in which case it refuses to start. ``--no-hooks`` leaves
only the AskUserQuestion answer hook registered, for a case that measures skill
behaviour in isolation.

Pre-seeded answers (EVL-030), and why a permission host is part of the runner.
A case's ``answers`` map is written into the workspace and a PreToolUse handler
on ``AskUserQuestion`` answers from it. That handler needs the tool to exist,
and in ``claude -p`` the tool exists only when a permission host does.

Measured on ``claude 2.1.268``, same argv both times apart from the host flags,
reading ``system.init.tools``:

* no host — 33 tools, ``AskUserQuestion`` **absent**. The skill notices and asks
  in prose instead, which is what the Q-017 pilot saw.
* with ``--mcp-config <workspace>/.claude/mcp-permissions.json
  --permission-prompt-tool mcp__dfa-permissions__approve`` — ``AskUserQuestion``
  **present**, and ``mcp_servers`` lists ``dfa-permissions: connected``.

``--allowedTools`` permits; it does not add. So a case that seeds answers names
the stdio host in ``permission_host.py``, which allows every request unchanged,
and a case that seeds none takes ``--permission-prompts none`` so a question it
was never meant to reach is denied rather than left hanging.

Permission mode (revised after the acceptance set). The runner passes
``--permission-mode bypassPermissions``. ``acceptEdits`` covers Write and Edit
and nothing else, and a ``claude -p`` run has no approval surface, so every
compound shell command is denied outright: Build step 7.2 runs
``cd wt/STORY-014 && ./ci/test``, and neither that nor the project's own test
command can match a ``Bash(devforgeai *)`` rule. ``--allowedTools`` is kept as
the documented statement of what a run is meant to need. The bypass is bounded:
the workspace is a throwaway directory the runner creates and deletes, the
framework hooks still run inside it when a trust pin exists, and the child reads
no user configuration.

A case's preconditions are checked by the binary, not by a parser. Each case
declares, under ``preflight``, the CLI calls the skill makes before it writes
anything; ``--preflight`` materialises the case into a throwaway workspace and
requires exit 0 from each. A TOML parse is not enough — the binary rejects a
``config.toml`` with no ``generated_at`` that parses perfectly — and a case that
fails here grades the fixture rather than the skill. A case that seeds no
``state.toml``, ``config.toml`` or ``gates.toml`` inherits the one
``devforgeai init`` writes, so no run starts on a file the CLI refuses.

Timeouts. ``--timeout`` sets the ceiling for every case, and a case that needs
longer declares its own integer ``timeout`` in seconds beside ``prompt``. The
phases that write six or more documents need it: Constitute writes the six
context files and their ADRs, Explore's ``ex-08`` measured 828s of the 900s
default, and Plan's ``PLAN-01`` measured 850s. A timed-out case records
``total_cost_usd: null`` — the cost arrives in the ``result`` event, and a run
that was killed never emits one — so the spend of a timeout is real and
unrecorded, and the summary's cost total is a floor rather than the bill.

Tampering (EVL-004). ``bypassPermissions`` means a model in an eval can rewrite
the files that decide whether it passed. ``gates.toml``, ``config.toml``,
``state.toml`` and the workspace ``settings.json`` are hashed before the run and
after it; any change marks the case ``status: tampered`` with the path,
whatever the grader returned. Two of the four carry an exemption, each for a
rewrite the framework itself performs: ``state.toml`` is rewritten by every
``phase set`` and is reported only when it was deleted, and ``config.toml`` is
compared by content with ``generated_at`` removed, because the SessionStart
hook runs ``stack detect`` on every hooks-on run. A changed ``[[stack]]``
command or coverage floor is still tampering.

That content comparison is exact on 3.11+, where ``tomllib`` parses the file,
and textual below it. What makes the textual form exact in practice is that the
runner runs ``stack detect`` once itself, after materialising and before taking
the baseline: the file is then already in the binary's canonical form, so the
SessionStart detect rewrites nothing and the comparison never has to see a
reformatting. Three acceptance runs were marked ``tampered`` before that landed.

No grader reads these files today, so tampering buys nothing — but that
is a property of the current graders, not of the harness, and this makes it one.

Preamble refusals reach the transcript as a plain string. Claude Code refuses a
``!`...``` injection that carries a shell expansion — measured, a preamble
written ``devforgeai gate require plan $1`` is refused with "Shell command
permission check failed ... Contains simple_expansion", while
``$ARGUMENTS[0]`` is accepted — and the refusal arrives as a ``user`` event
whose ``message.content`` is a bare string rather than a block list. The stream
reader keeps it, so a case that aborts on a refused preamble records the reason
instead of a runner crash.

Contract: ``specs/01-cli.md``, section ``## Evals``, "Python, the shared eval
runner". Standard library only; the processes spawned are ``claude``,
``devforgeai trust verify``, and ``git`` for a case that declares
``setup.git``.

Exit codes: 0 every case passed, 1 a case failed, 2 a case errored or timed out,
3 usage error.
"""

import argparse
import concurrent.futures
import datetime
import fnmatch
import hashlib
import importlib.util
import json
import os
import re
import secrets
import shlex
import shutil
import stat
import subprocess
import sys
import threading
import time

SCHEMA = "devforgeai/eval-result/2"
EVIDENCE_CAP = 2000
FIXTURE = re.compile(r"^FIXTURE:(.+)$")
# agents.md "## Contracts" table: the first cell of each data row names the agent.
# The older per-agent "## `name`" heading form is accepted as a fallback.
AGENT_ROW = re.compile(r"^\|\s*`([a-z0-9][a-z0-9-]*)`\s*\|", re.MULTILINE)
AGENT_HEADING = re.compile(r"^##\s+`([a-z0-9][a-z0-9-]*)`\s*$", re.MULTILINE)

# EVL-011. `--permission-mode acceptEdits` covers Write and Edit and nothing
# else; the shell grant is what every skill preamble needs, in both spellings
# the frontmatter and the guidance use. The list is additive, so naming a tool
# here grants it and denies nothing.
ALLOWED_TOOLS = (
    "Bash(devforgeai *),PowerShell(devforgeai *),"
    "Bash(devforgeai:*),PowerShell(devforgeai:*),"
    "Read,Write,Edit,Glob,Grep,Agent,Skill,AskUserQuestion,"
    "WebSearch,WebFetch,TodoWrite")

# The substitution tokens `hooks/settings.hooks.json` carries.
BINARY_TOKEN = "@@DEVFORGEAI@@"
TEST_COMMAND_TOKEN = "@@TEST_COMMAND@@"
VERIFIERS_TOKEN = "@@VERIFIERS@@"

# EVL-011, revised after the acceptance set. `acceptEdits` covers Write and Edit
# and nothing else, and `claude -p` has no approval surface, so every compound
# shell command is denied outright: `cd wt/STORY-014 && ./ci/test` is what Build
# step 7.2 runs, and no `Bash(devforgeai *)` rule can match it or the project's
# own test command. The workspace is a throwaway the runner creates and deletes,
# the hooks still run inside it, and the child sees no user configuration, so the
# blast radius of bypassing the prompt is that directory.
PERMISSION_MODE = "bypassPermissions"

ANSWERS_NAME = "eval-answers.json"
ANSWER_HOOK_NAME = "answer_hook.py"

# The stdio permission host and the tool reference `--permission-prompt-tool`
# takes. Measured on `claude 2.1.268`: without a host the headless tool set holds
# 33 tools and no `AskUserQuestion`, so the answer hook has nothing to intercept;
# with this host it holds `AskUserQuestion` and the hook fires.
PERMISSION_HOST_NAME = "permission_host.py"
PERMISSION_SERVER = "dfa-permissions"
PERMISSION_TOOL = "mcp__%s__approve" % PERMISSION_SERVER
MCP_CONFIG_NAME = "mcp-permissions.json"

# The session variables a nested `claude -p` must not inherit.
SESSION_VARIABLES = ("CLAUDECODE", "CLAUDE_PID", "AI_AGENT", "CLAUDE_EFFORT",
                     "ANTHROPIC_API_KEY")

# A `result` event whose text says the account ran out of session budget. The
# run produced no verdict about the skill, so it is recorded apart from a real
# error: an acceptance sweep that hits the limit half way through would
# otherwise read as six failing skills.
LIMIT_RE = re.compile(
    r"(session|usage) limit|rate[- ]limit|resets \d|out of (credits|usage)",
    re.I)

PIN_HINT = ("devforgeai trust pin --framework %s   (a human runs it outside "
            "Claude Code)")


class UsageError(Exception):
    """A malformed invocation, case file, or fixture reference: exit 3."""


class CaseError(Exception):
    """A condition that stops one case with status ``error``."""


# ---------------------------------------------------------------------------
# small helpers
# ---------------------------------------------------------------------------

def _utc(stamp):
    return datetime.datetime.fromtimestamp(
        stamp, datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def _run_id(stamp):
    basic = datetime.datetime.fromtimestamp(
        stamp, datetime.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    return "%s-%s" % (basic, secrets.token_hex(4))


def _on_rm_error(func, path, _info):
    os.chmod(path, stat.S_IWRITE)
    func(path)


def _rmtree(path):
    shutil.rmtree(path, onerror=_on_rm_error)


def _write_text(path, text):
    parent = os.path.dirname(path)
    if parent:
        os.makedirs(parent, exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as handle:
        handle.write(text)


def _inside(root, candidate):
    root = os.path.abspath(root)
    candidate = os.path.abspath(candidate)
    return candidate == root or candidate.startswith(root + os.sep)


def framework_root():
    """The repository root: two levels above ``evals/runner``."""
    return os.path.dirname(os.path.dirname(os.path.dirname(
        os.path.abspath(__file__))))


def default_binary(root=None):
    root = root or framework_root()
    name = "devforgeai.exe" if os.name == "nt" else "devforgeai"
    return os.path.join(root, "cli", "target", "release", name)


# ---------------------------------------------------------------------------
# cases
# ---------------------------------------------------------------------------

def _check_answers(case, path, number):
    answers = case.get("answers")
    if answers is None:
        return
    if not isinstance(answers, dict):
        raise UsageError("%s line %d: answers is not an object" % (path, number))
    for key, value in answers.items():
        ok = isinstance(value, str) or (
            isinstance(value, list) and value
            and all(isinstance(one, str) for one in value))
        if not ok:
            raise UsageError(
                "%s line %d: answers[%r] is neither a string nor a non-empty "
                "list of strings" % (path, number, key))


def load_cases(path):
    """Parse cases.jsonl into a list of dicts, keeping file order."""
    if not os.path.isfile(path):
        raise UsageError("cases file not found: %s" % path)
    cases = []
    with open(path, "r", encoding="utf-8") as handle:
        for number, line in enumerate(handle, 1):
            if not line.strip():
                continue
            try:
                case = json.loads(line)
            except ValueError as exc:
                raise UsageError("%s line %d is not JSON: %s" % (path, number, exc))
            for key in ("id", "prompt", "expect"):
                if key not in case:
                    raise UsageError("%s line %d has no %r" % (path, number, key))
            if "grader" not in case.get("expect", {}):
                raise UsageError("%s line %d has no expect.grader" % (path, number))
            _check_answers(case, path, number)
            for entry in case.get("preflight") or []:
                if isinstance(entry, str):
                    continue
                if not isinstance(entry, dict) or "command" not in entry:
                    raise UsageError(
                        "%s line %d: a preflight entry is neither a command "
                        "string nor an object carrying 'command'"
                        % (path, number))
            limit = case.get("timeout")
            if limit is not None and (not isinstance(limit, int)
                                      or isinstance(limit, bool) or limit < 1):
                raise UsageError(
                    "%s line %d: timeout is %r, expected a positive number of "
                    "seconds" % (path, number, limit))
            cases.append(case)
    if not cases:
        raise UsageError("no cases in %s" % path)
    return cases


def merged_files(case, cases):
    """Resolve setup.extends and return the merged setup.files map.

    An earlier case's map is written first and this case's map over it, so a
    path in both carries this case's content. The chain resolves outermost
    first. A cycle, or an id no earlier line defines, is a usage error.
    """
    order = {case_["id"]: index for index, case_ in enumerate(cases)}
    by_id = {case_["id"]: case_ for case_ in cases}
    chain = []
    seen = set()
    node = case
    while True:
        if node["id"] in seen:
            raise UsageError("setup.extends cycle at case %r" % node["id"])
        seen.add(node["id"])
        chain.append(node)
        parent = node.get("setup", {}).get("extends")
        if parent is None:
            break
        if parent not in by_id or order[parent] >= order[node["id"]]:
            raise UsageError(
                "case %r extends %r, which no earlier line defines"
                % (node["id"], parent))
        node = by_id[parent]
    files = {}
    for node in reversed(chain):
        files.update(node.get("setup", {}).get("files", {}))
    return files


def resolve_fixtures(files, skill_dir):
    """Replace every ``FIXTURE:<name>`` value with the fixture's text."""
    fixtures = os.path.join(skill_dir, "evals", "fixtures")
    out = {}
    for relative, value in files.items():
        match = FIXTURE.match(value) if isinstance(value, str) else None
        if match is None:
            out[relative] = value
            continue
        source = os.path.join(fixtures, match.group(1))
        if not os.path.isfile(source):
            raise UsageError("fixture not found: %s" % source)
        with open(source, "r", encoding="utf-8") as handle:
            out[relative] = handle.read()
    return out


# ---------------------------------------------------------------------------
# workspace
# ---------------------------------------------------------------------------

def owned_agents(skill_dir, root):
    """Agent names the skill's agents.md documents that exist in agents/."""
    catalogue = os.path.join(skill_dir, "agents.md")
    directory = os.path.join(root, "agents")
    if not (os.path.isfile(catalogue) and os.path.isdir(directory)):
        return []
    with open(catalogue, "r", encoding="utf-8") as handle:
        text = handle.read()
    names = []
    found = AGENT_ROW.findall(text) or AGENT_HEADING.findall(text)
    for name in found:
        if name not in names and os.path.isfile(
                os.path.join(directory, name + ".md")):
            names.append(name)
    return names


def _resolve_tokens(block, binary):
    """Resolve the three tokens of the framework hook block, in place.

    ``@@DEVFORGEAI@@`` becomes the binary path. The other two are per-project
    and this workspace registers no test command and no verifier, so the CLI's
    own rule applies: an unresolvable ``if`` filter is dropped rather than
    guessed, and an unresolvable ``matcher`` is dropped so the group fires for
    every subagent rather than for none.
    """
    for _event, entries in sorted((block.get("hooks") or {}).items()):
        for entry in entries or []:
            matcher = entry.get("matcher")
            if isinstance(matcher, str) and VERIFIERS_TOKEN in matcher:
                entry.pop("matcher", None)
            for handler in entry.get("hooks") or []:
                command = handler.get("command")
                if isinstance(command, str):
                    handler["command"] = command.replace(BINARY_TOKEN, binary)
                if TEST_COMMAND_TOKEN in str(handler.get("if") or ""):
                    handler.pop("if", None)
    return block


def framework_hooks(binary, root=None):
    """``hooks/settings.hooks.json`` with its tokens resolved."""
    path = os.path.join(root or framework_root(), "hooks", "settings.hooks.json")
    if not os.path.isfile(path):
        raise UsageError("hook template not found: %s" % path)
    with open(path, "r", encoding="utf-8") as handle:
        return _resolve_tokens(json.load(handle), binary)


def answer_hook_entry(python, script, answers_path):
    """The one PreToolUse group the answer seed registers.

    Exec form: guidance section 1 wants a real executable on Windows, and shell
    form risks a shell profile echoing into stdout and breaking JSON parsing.
    ``python`` is an absolute interpreter path, so no PATH lookup is involved.
    """
    return {"matcher": "AskUserQuestion",
            "hooks": [{"type": "command",
                       "command": python,
                       "args": [script, answers_path],
                       "timeout": 20}]}


def permission_host(claude_dir):
    """Copy the stdio permission host in and write its ``--mcp-config`` file.

    Returns the config path. A host is what keeps ``AskUserQuestion`` in the
    headless tool set, so a case that seeds answers gets one; the host allows
    every request unchanged, because the eval's real decisions are the
    permission mode, the allowed-tool list and the answer hook.
    """
    script = os.path.join(claude_dir, PERMISSION_HOST_NAME)
    shutil.copyfile(
        os.path.join(os.path.dirname(os.path.abspath(__file__)),
                     PERMISSION_HOST_NAME),
        script)
    config = os.path.join(claude_dir, MCP_CONFIG_NAME)
    _write_text(config, json.dumps({"mcpServers": {PERMISSION_SERVER: {
        "command": sys.executable, "args": [script]}}}, indent=1))
    return config


def workspace_settings(case, claude_dir, options):
    """Write ``.claude/settings.json``, the answers file and the hook copy.

    Decision 42, amended (EVL-031): an eval registers the AskUserQuestion
    answer seed always, and the framework's own hook block whenever hooks are
    on, and no other handler of its own.
    """
    answers = case.get("answers") or {}
    answers_path = os.path.join(claude_dir, ANSWERS_NAME)
    _write_text(answers_path, json.dumps(answers, ensure_ascii=False, indent=1))
    script = os.path.join(claude_dir, ANSWER_HOOK_NAME)
    shutil.copyfile(
        os.path.join(os.path.dirname(os.path.abspath(__file__)),
                     ANSWER_HOOK_NAME),
        script)

    if answers:
        permission_host(claude_dir)

    settings = (framework_hooks(options.devforgeai_bin, options.framework_root)
                if options.hooks else {"hooks": {}})
    hooks = settings.setdefault("hooks", {})
    hooks.setdefault("PreToolUse", []).insert(
        0, answer_hook_entry(sys.executable, script, answers_path))
    _write_text(os.path.join(claude_dir, "settings.json"),
                json.dumps(settings, indent=1))
    return settings


# What the stand-in commands and the redirected home write into a workspace.
# A git-seeded case commits through `devforgeai commit`, which refuses on an
# unexpected untracked file, so these are ignored rather than left to surprise
# the run.
WORKSPACE_GITIGNORE = """# Written by the eval harness and its stand-in commands.
coverage/
out/
.dfa-*.txt
.dfa-expect.tmp
.claude/dfa-home/
.explore-prototype/
"""


def init_git(workspace, spec):
    """Make the workspace a git work tree with one commit, and seed worktrees.

    Opt in per case with ``"setup": {"git": true}``, or with an object naming
    worktrees to register up front::

        "git": {"worktrees": [{"path": "wt/STORY-014",
                               "branch": "story/STORY-014"}]}

    Build's whole worktree path — ``devforgeai worktree ensure``,
    ``worktree list``, ``commit`` — exits ``DFA-E271`` outside a work tree, and
    ``worktree add`` needs a base revision, so the initial commit is part of the
    fixture rather than a convenience. A seeded worktree is what makes the
    overlap refusal ``DFA-E272`` reachable: the binary reads
    ``git worktree list --porcelain``, not a ``.git`` file someone wrote.
    Identity is set on the repository so the commit does not depend on the
    machine's ``~/.gitconfig``.
    """
    ignore = os.path.join(workspace, ".gitignore")
    if not os.path.exists(ignore):
        _write_text(ignore, WORKSPACE_GITIGNORE)
    steps = [["init", "-q"],
             ["config", "user.email", "evals@devforgeai.invalid"],
             ["config", "user.name", "DevForgeAI evals"],
             ["add", "-A"],
             ["commit", "-q", "-m", "eval fixture"]]
    for entry in (spec.get("worktrees") or []) if isinstance(spec, dict) else []:
        path = entry["path"].replace("/", os.sep)
        steps.append(["worktree", "add", "-q", "-b", entry["branch"], path, "HEAD"])
    for step in steps:
        finished = subprocess.run(["git"] + step, cwd=workspace,
                                  capture_output=True, text=True, timeout=180)
        if finished.returncode != 0:
            raise CaseError("git %s failed: %s"
                            % (" ".join(step), (finished.stderr or "").strip()))


def installed_skill_name(skill_dir):
    """The directory name the skill installs under: its frontmatter ``name:``.

    Claude Code derives a project skill's slash command from the directory it
    sits in, not from the frontmatter, so a skill at
    ``.claude/skills/exploring-ideas/`` is ``/exploring-ideas`` however its
    frontmatter reads. ``init`` installs ``skills/<long-dir>/`` at
    ``.claude/skills/<name>/`` for that reason, and the eval workspace copies the
    skill the same way or it measures a command the target project does not have.
    Falls back to the source directory name when the frontmatter carries no
    ``name:``.
    """
    path = os.path.join(skill_dir, "SKILL.md")
    fallback = os.path.basename(os.path.normpath(skill_dir))
    try:
        with open(path, "r", encoding="utf-8") as handle:
            head = handle.read(4096)
    except OSError:
        return fallback
    if not head.startswith("---"):
        return fallback
    end = head.find("\n---", 3)
    front = head[3:end] if end != -1 else head[3:]
    match = re.search(r"^name:\s*(\S+)\s*$", front, re.M)
    return match.group(1).strip("\"'") if match else fallback


_DEFAULTS = {}

DEFAULT_FILES = ("state.toml", "config.toml", "gates.toml")


def cli_defaults(binary, framework):
    """The three project files `devforgeai init` writes, read from the binary.

    A case seeds what it cares about and inherits the rest. Before this, a case
    with no `state.toml` met `DFA-E103` at the skill's first `phase set` and the
    model either stopped or ran `devforgeai init` inside the workspace, which
    merged the framework hooks over the eval's settings and left the fail-closed
    trust check denying every tool call. The defaults come from the binary
    rather than from a fixture so they cannot drift from what a real `init`
    produces; the throwaway `init` runs outside every workspace, so the skills
    and agents it copies never reach one.
    """
    if _DEFAULTS:
        return _DEFAULTS["files"]
    import tempfile
    files = {}
    root = tempfile.mkdtemp(prefix="dfa-defaults-")
    try:
        subprocess.run([binary, "init", "--from", framework, "--quiet"],
                       cwd=root, capture_output=True, text=True, timeout=300)
        for name in DEFAULT_FILES:
            path = os.path.join(root, ".devforgeai", name)
            if os.path.isfile(path):
                with open(path, "r", encoding="utf-8") as handle:
                    files[name] = handle.read()
    except (OSError, subprocess.SubprocessError):
        # A stand-in binary in a unit test cannot write defaults; a case then
        # gets exactly what its own `setup.files` holds, as before.
        files = {}
    finally:
        shutil.rmtree(root, ignore_errors=True)
    _DEFAULTS["files"] = files
    return files


def canonicalise_config(workspace, binary):
    """Run ``stack detect`` once so ``config.toml`` is already in canonical form.

    A materialised fixture is hand-written and the binary's form is not: it
    carries `stack = []`, `layer = []`, `verifier = []`, `[frontend]` globs and a
    `degraded` the merged tables decide. The framework's SessionStart hook runs
    `stack detect` on every hooks-on run, which rewrites the file into that form
    once — and the tamper guard, whose 3.10 fallback compares text, reads the
    reformatting as a model editing the gate's inputs. Three acceptance runs were
    marked `tampered` for exactly that.

    Detecting here, before the baseline is taken, makes the SessionStart detect a
    no-op (measured: `written = false`, bytes identical) and byte equality exact
    for all four guarded paths. Returns the file's ``degraded`` value, or
    ``None`` when there is no config or the binary did not run.
    """
    config = os.path.join(workspace, ".devforgeai", "config.toml")
    if not os.path.isfile(config):
        return None
    try:
        subprocess.run([binary, "stack", "detect", "--quiet"], cwd=workspace,
                       capture_output=True, text=True, timeout=300)
    except (OSError, subprocess.SubprocessError):
        # A stand-in binary in a unit test cannot detect; the workspace is then
        # exactly what the case seeded, as before.
        return None
    return config_degraded(config)


def config_degraded(path):
    """The ``degraded`` top-level value of a config file, or ``None``."""
    try:
        with open(path, "r", encoding="utf-8") as handle:
            for line in handle:
                if line.startswith("degraded"):
                    return "true" in line.split("=", 1)[-1]
    except OSError:
        pass
    return None


# A skill body reaching another skill through the Skill tool: "invoke the
# `design` skill", "the `design` skill through the Skill tool". Explore step 6
# and Plan steps 6 and R3 are the three such call sites today.
SKILL_CALL = re.compile(
    r"invoke(?:s)?\s+the\s+`([a-z][a-z0-9-]*)`\s+skill"
    r"|`([a-z][a-z0-9-]*)`\s+skill\s+through\s+the\s+Skill\s+tool")


def invoked_skills(skill_dir, root):
    """The slash names this skill invokes through the Skill tool.

    Each resolves to a sibling under ``skills/`` by its frontmatter ``name``,
    and the workspace needs it installed: a workspace holding only the skill
    under test lets Claude Code resolve the name against whatever else is on the
    machine. Measured on the hooks-on `ex-01` run — the workspace carried only
    `.claude/skills/explore/`, step 6 asked for `design`, and Claude Code found
    a different `design` skill on this machine and refused it with "Skill design
    cannot be used with Skill tool due to disable-model-invocation". The run
    stalled there.
    """
    path = os.path.join(skill_dir, "SKILL.md")
    try:
        with open(path, "r", encoding="utf-8") as handle:
            body = handle.read()
    except OSError:
        return []
    wanted = []
    for match in SKILL_CALL.finditer(body):
        name = match.group(1) or match.group(2)
        if name and name not in wanted:
            wanted.append(name)
    mine = installed_skill_name(skill_dir)
    return [name for name in wanted if name != mine]


def skill_source(name, root):
    """The directory under ``skills/`` whose frontmatter ``name`` is ``name``."""
    directory = os.path.join(root, "skills")
    if not os.path.isdir(directory):
        return None
    for entry in sorted(os.listdir(directory)):
        candidate = os.path.join(directory, entry)
        if os.path.isdir(candidate) and installed_skill_name(candidate) == name:
            return candidate
    return None


def install_skill(source, claude_dir, root):
    """Copy one skill under its frontmatter name, with the agents it owns."""
    name = installed_skill_name(source)
    target = os.path.join(claude_dir, "skills", name)
    if os.path.isdir(target):
        return name
    shutil.copytree(source, target,
                    ignore=shutil.ignore_patterns("__pycache__", "evals"))
    agents = owned_agents(source, root)
    if agents:
        os.makedirs(os.path.join(claude_dir, "agents"), exist_ok=True)
        for agent in agents:
            origin = os.path.join(root, "agents", agent + ".md")
            if os.path.isfile(origin):
                shutil.copyfile(origin,
                                os.path.join(claude_dir, "agents", agent + ".md"))
    return name


# The top-level tables of `config.toml`, in the order `init` writes them. A
# seeded fixture states the few its case is about; the rest come from the
# default, because the gate reads all of them.
CONFIG_TABLES = ("stack", "frontend", "layer", "explore", "plan", "build",
                 "verify", "release", "reflect", "coverage", "verifier")

# A top-level table header and nothing else: `[stack.env]` is a subtable and
# belongs to the `[[stack]]` above it, so the closing bracket has to follow the
# name immediately or the block would be split from its parent.
_TABLE_HEADER = re.compile(
    r"^(?:\[\[([a-z_][a-z0-9_]*)\]\]|\[([a-z_][a-z0-9_]*)\])\s*$")


def split_config_tables(text):
    """``(preamble, {table: [block, ...]})`` for one config file.

    A block is the header line and everything to the next top-level header, so
    `[stack.env]` travels with the `[[stack]]` it belongs to. The preamble is
    the key-value head — `schema`, `generated_at`, `cli_version`, `degraded`.
    """
    lines = text.split("\n")
    starts = [i for i, line in enumerate(lines) if _TABLE_HEADER.match(line)]
    if not starts:
        return text, {}
    preamble = "\n".join(lines[:starts[0]])
    blocks = {}
    for index, start in enumerate(starts):
        end = starts[index + 1] if index + 1 < len(starts) else len(lines)
        found = _TABLE_HEADER.match(lines[start])
        name = found.group(1) or found.group(2)
        blocks.setdefault(name, []).append("\n".join(lines[start:end]).rstrip())
    return preamble, blocks


_NAME_KEY = re.compile(r"""^\s*name\s*=\s*["']([^"']+)["']""", re.M)


def _block_name(block):
    """The ``name`` key of one ``[[verifier]]`` block, or ``""``."""
    found = _NAME_KEY.search(block)
    return found.group(1) if found else ""


def merge_verifiers(seeded, default):
    """The default registry with the fixture's rows overriding by ``name``.

    ``[[verifier]]`` is the one table that merges rather than replaces. It is a
    catalogue rather than a choice: `gate check` fails `DFA-E316` on a phase
    whose verifier the file does not register, so a fixture stating the three
    rows its own phase invokes would hide the other seventeen from every other
    phase — which is how the Explore gate came to fail and the model came to
    edit `config.toml`. A seeded row replaces the default row of the same name
    field for field, so a case can still change one row's `required`; a default
    row the fixture does not name is kept; a seeded row naming nothing in the
    default is appended.
    """
    if not seeded:
        return list(default)
    overrides = {}
    extras = []
    for block in seeded:
        name = _block_name(block)
        if name:
            overrides[name] = block
        else:
            extras.append(block)
    out = []
    for block in default:
        name = _block_name(block)
        out.append(overrides.pop(name, block) if name else block)
    for block in seeded:
        name = _block_name(block)
        if name and name in overrides:
            out.append(overrides.pop(name))
    return out + extras


def merge_config(seeded, default):
    """A seeded config completed from the default, one top-level table at a time.

    A case states the tables it is about and inherits the rest. `init` writes a
    twenty-row `[[verifier]]` registry, and a fixture that omits it made the
    Explore gate fail `DFA-E316 config.toml registers no [[verifier]] named
    'kill-case-builder'` — after which the model edited `config.toml` to add the
    registry and the tamper guard flagged it, correctly. A table the fixture
    does state replaces the default's outright, so a seeded `[[stack]]` is the
    only stack and a seeded `[build]` is the only build table — with one
    exception, `[[verifier]]`, which merges by name because it is a
    catalogue every phase reads rather than a choice this case makes.
    See `merge_verifiers`.
    """
    if not default:
        return seeded
    preamble, seeded_blocks = split_config_tables(seeded)
    default_preamble, default_blocks = split_config_tables(default)
    if not seeded_blocks and not default_blocks:
        return seeded
    head = preamble.strip() or default_preamble.strip()
    out = [head] if head else []
    for name in CONFIG_TABLES:
        if name == "verifier":
            blocks = merge_verifiers(seeded_blocks.get(name) or [],
                                     default_blocks.get(name) or [])
        else:
            blocks = seeded_blocks.get(name) or default_blocks.get(name) or []
        for block in blocks:
            out.append(block)
    # Any table neither list names, kept so a future key is not dropped.
    for name in list(seeded_blocks) + list(default_blocks):
        if name in CONFIG_TABLES:
            continue
        for block in seeded_blocks.get(name) or default_blocks.get(name) or []:
            if block not in out:
                out.append(block)
    return "\n\n".join(part.strip("\n") for part in out if part.strip()) + "\n"


def make_workspace(case, files, options):
    """Build one workspace and return (path, environment home)."""
    skill_dir = options.skill
    skill_name = installed_skill_name(skill_dir)
    workspace = os.path.join(
        options.workdir, "dfa-eval-%s-%s" % (case["id"], secrets.token_hex(4)))
    os.makedirs(workspace, exist_ok=True)

    for relative in files:
        target = os.path.join(workspace, relative.replace("/", os.sep))
        if not _inside(workspace, target):
            raise CaseError("path escapes the workspace: %s" % relative)
    for relative, content in files.items():
        target = os.path.join(workspace, relative.replace("/", os.sep))
        _write_text(target, content)
        # A seeded file that opens with a shebang is a command the case expects
        # the run to execute — `config.toml` names `./ci/test` and Build's
        # red-green loop turns on its exit code — so it is written executable.
        if isinstance(content, str) and content.startswith("#!"):
            os.chmod(target, os.stat(target).st_mode | stat.S_IXUSR
                     | stat.S_IXGRP | stat.S_IXOTH)

    defaults = cli_defaults(options.devforgeai_bin, options.framework_root)
    for name in DEFAULT_FILES:
        relative = ".devforgeai/" + name
        if name not in defaults:
            continue
        if relative not in files:
            _write_text(os.path.join(workspace, ".devforgeai", name), defaults[name])
        elif name == "config.toml":
            # Completed rather than replaced: the case keeps the tables it
            # states and inherits every other, the verifier registry included.
            _write_text(os.path.join(workspace, ".devforgeai", name),
                        merge_config(files[relative], defaults[name]))

    claude_dir = os.path.join(workspace, ".claude")
    # The destination directory is the frontmatter `name:`, because that is the
    # slash command: `.claude/skills/explore/` is `/explore`. `commands/` no
    # longer exists.
    shutil.copytree(
        skill_dir,
        os.path.join(claude_dir, "skills", skill_name),
        ignore=shutil.ignore_patterns("__pycache__", "evals"))

    agents = owned_agents(skill_dir, options.framework_root)
    if agents:
        os.makedirs(os.path.join(claude_dir, "agents"), exist_ok=True)
        for name in agents:
            shutil.copyfile(
                os.path.join(options.framework_root, "agents", name + ".md"),
                os.path.join(claude_dir, "agents", name + ".md"))

    # Every skill this one reaches through the Skill tool, plus whatever the
    # case names in `setup.skills`, installed under its own frontmatter name.
    wanted = list(invoked_skills(skill_dir, options.framework_root))
    for extra in (case.get("setup") or {}).get("skills", []):
        if extra not in wanted:
            wanted.append(extra)
    for name in wanted:
        source = skill_source(name, options.framework_root)
        if source is None:
            raise CaseError(
                "%s invokes the %r skill and no directory under skills/ carries "
                "that frontmatter name" % (skill_name, name))
        install_skill(source, claude_dir, options.framework_root)

    workspace_settings(case, claude_dir, options)
    git_spec = (case.get("setup") or {}).get("git")
    if git_spec:
        init_git(workspace, git_spec)

    # Before the guard baseline: the SessionStart hook would otherwise do this
    # rewrite mid-run and the guard would call it tampering.
    canonicalise_config(workspace, options.devforgeai_bin)
    # Decision 62: a home inside the workspace, so a debug build reads no trust
    # pin from the real machine. A release build ignores it (no `test-home`
    # feature), which is why hooks-on runs are gated on a real pin instead.
    home = os.path.join(claude_dir, "dfa-home")
    os.makedirs(home, exist_ok=True)
    return workspace, home


# ---------------------------------------------------------------------------
# claude
# ---------------------------------------------------------------------------

# EVL-004. `bypassPermissions` lets a model in an eval rewrite the files that
# decide whether it passed. No grader reads them today, so tampering buys
# nothing — but "buys nothing" is a property of the current graders, not of the
# harness. These four are hashed before the run and after it, and any change
# marks the case `tampered` whatever the grader returned.
GUARDED = (".devforgeai/gates.toml",
           ".devforgeai/config.toml",
           ".devforgeai/state.toml",
           ".claude/settings.json")



# `config.toml` is compared by content rather than by bytes. The framework's
# SessionStart hook runs `stack detect`, which rewrites the file on every
# hooks-on run and stamps a fresh `generated_at`; that is the harness working,
# not a model editing the gate's inputs. Everything else in the file — a
# `[[stack]]` command, a coverage floor, a verifier entry — still counts.
VOLATILE_CONFIG_KEYS = ("generated_at",)


def config_fingerprint(text):
    """A digest of ``config.toml`` that ignores only its timestamp.

    Parsed where the standard library can (``tomllib``, 3.11+), and otherwise by
    dropping the volatile top-level keys from the text. The fallback is exact
    for the case it exists for: `stack detect` re-serialises the same values the
    same way, so a no-op rewrite differs in the stamp alone.
    """
    try:
        import tomllib
    except ImportError:
        kept = [line for line in text.splitlines()
                if not any(line.lstrip().startswith(key + " ")
                           or line.lstrip().startswith(key + "=")
                           for key in VOLATILE_CONFIG_KEYS)]
        body = "\n".join(line.rstrip() for line in kept if line.strip())
        return hashlib.sha256(body.encode("utf-8")).hexdigest()
    try:
        parsed = tomllib.loads(text)
    except (ValueError, TypeError):
        # An unparsable config is itself a change worth reporting, so fall back
        # to the bytes rather than treating every broken file as equal.
        return hashlib.sha256(text.encode("utf-8")).hexdigest()
    for key in VOLATILE_CONFIG_KEYS:
        parsed.pop(key, None)
    canonical = json.dumps(parsed, sort_keys=True, separators=(",", ":"),
                           default=str)
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def guard_digests(workspace):
    """A digest of each guarded path, or ``None`` where the file is absent."""
    out = {}
    for relative in GUARDED:
        path = os.path.join(workspace, *relative.split("/"))
        try:
            with open(path, "rb") as handle:
                raw = handle.read()
        except OSError:
            out[relative] = None
            continue
        if relative == ".devforgeai/config.toml":
            out[relative] = config_fingerprint(
                raw.decode("utf-8", errors="replace"))
        else:
            out[relative] = hashlib.sha256(raw).hexdigest()
    return out


def tampering(before, after):
    """The guarded paths the run changed, as one line of evidence, or ``""``.

    ``state.toml`` is the exception among the four: the CLI rewrites it on every
    ``phase set``, which is the run doing its job. It is hashed anyway so the
    record carries the before and after, and reported only when the file is
    absent afterwards — a deletion is never the CLI's doing.
    """
    changed = []
    for relative in GUARDED:
        old, new = before.get(relative), after.get(relative)
        if old == new:
            continue
        if relative == ".devforgeai/state.toml" and new is not None:
            continue
        changed.append("%s %s -> %s" % (
            relative,
            "absent" if old is None else old[:12],
            "absent" if new is None else new[:12]))
    return "; ".join(changed)


def claude_argv(binary, model, workspace, seeded, prompts_none):
    """The prompt travels on stdin, not in argv (EVL-051).

    A case that seeds answers names the stdio permission host, because that is
    what keeps `AskUserQuestion` in the tool set for the answer hook to
    intercept. A case that seeds none takes `--permission-prompts none`, so a
    question it was never meant to reach is denied rather than left hanging.
    """
    argv = [binary, "-p",
            "--model", model,
            "--output-format", "stream-json", "--verbose",
            "--include-partial-messages",
            "--permission-mode", PERMISSION_MODE,
            "--allowedTools", ALLOWED_TOOLS,
            "--add-dir", workspace]
    if seeded:
        argv += ["--mcp-config",
                 os.path.join(workspace, ".claude", MCP_CONFIG_NAME),
                 "--permission-prompt-tool", PERMISSION_TOOL]
    elif prompts_none:
        argv += ["--permission-prompts", "none"]
    return argv


def _render_tool_use(block, tools):
    """One transcript line for a ``tool_use`` block.

    An Agent call renders as ``Agent(<subagent_type> <prompt>)`` with ``)``
    replaced by ``]`` inside the parentheses, because implementing-stories'
    ``criterion_set`` matches ``Agent\\(ac-test-writer[^)]*?(AC-\\d{3})`` and a
    bare ``)`` would end the match early. Every other call renders as
    ``<Name>(<compact JSON input>)``, which puts a Bash command string such as
    ``devforgeai worktree ensure STORY-015`` into the transcript verbatim.
    """
    name = block.get("name") or "?"
    tools[name] = tools.get(name, 0) + 1
    payload = block.get("input") or {}
    if name == "Agent":
        inner = "%s %s" % (payload.get("subagent_type") or "",
                           payload.get("prompt") or "")
        return "Agent(%s)" % inner.replace(")", "]")
    return "%s(%s)" % (
        name, json.dumps(payload, ensure_ascii=False).replace(")", "]"))


def read_stream(stdout):
    """Assemble a transcript from stream-json events and pull the result meta.

    ``--include-partial-messages`` adds ``stream_event`` lines carrying the same
    text a later ``assistant`` event repeats; they are skipped so no text is
    counted twice. The ``result`` text is appended last, so every grader that
    reads the final handoff block still finds it at the end.
    """
    lines, meta, tools = [], {}, {}
    for raw in stdout.splitlines():
        raw = raw.strip()
        if not raw.startswith("{"):
            continue
        try:
            event = json.loads(raw)
        except ValueError:
            continue
        kind = event.get("type")
        if kind == "assistant":
            for block in (event.get("message") or {}).get("content") or []:
                if block.get("type") == "text":
                    lines.append(block.get("text") or "")
                elif block.get("type") == "tool_use":
                    lines.append(_render_tool_use(block, tools))
        elif kind == "user":
            content = (event.get("message") or {}).get("content") or []
            if isinstance(content, str):
                # A plain-text user turn: Claude Code emits one when a skill
                # preamble is refused (`<local-command-stderr>`); keep it so
                # the transcript shows why the run stopped.
                lines.append(content)
                content = []
            for block in content:
                if not isinstance(block, dict):
                    continue
                if block.get("type") == "tool_result":
                    body = block.get("content")
                    if isinstance(body, str):
                        lines.append(body)
                    elif isinstance(body, list):
                        for part in body:
                            if isinstance(part, dict) and part.get("type") == "text":
                                lines.append(part.get("text") or "")
        elif kind == "result":
            value = event.get("result")
            if isinstance(value, str):
                lines.append(value)
            meta = {"session_id": event.get("session_id"),
                    "total_cost_usd": event.get("total_cost_usd"),
                    "num_turns": event.get("num_turns"),
                    "permission_denials": event.get("permission_denials") or [],
                    "is_error": bool(event.get("is_error")),
                    "subtype": event.get("subtype"),
                    "result_text": value if isinstance(value, str) else ""}
    meta["tool_calls"] = tools
    return "\n".join(lines), meta


_VERSION_PROBE = {}


def version_supports_prompts_none(binary):
    """True when the installed claude is 2.1.259 or newer (guidance §4)."""
    if binary in _VERSION_PROBE:
        return _VERSION_PROBE[binary]
    _VERSION_PROBE[binary] = _probe_prompts_none(binary)
    return _VERSION_PROBE[binary]


def _probe_prompts_none(binary):
    try:
        out = subprocess.run([shutil.which(binary) or binary, "--version"],
                             capture_output=True, text=True, timeout=60).stdout
    except (OSError, subprocess.SubprocessError):
        return False
    match = re.search(r"(\d+)\.(\d+)\.(\d+)", out or "")
    return bool(match) and tuple(int(g) for g in match.groups()) >= (2, 1, 259)


def isolate_claude_config(config):
    """Build the redirected ``CLAUDE_CONFIG_DIR`` and return its path (EVL-001).

    Inherited, ``~/.claude`` loads the machine's own skills, agents, plugins,
    settings and memory into every eval, so a grade depends on the machine it
    ran on. Redirecting the config directory removes all of that. The redirect
    is ``CLAUDE_CONFIG_DIR`` and not ``HOME``/``USERPROFILE`` on purpose: the
    binary's home checks (``report aggregate``'s session root) and git's
    identity both read the real profile, and moving it breaks them.

    One file crosses: ``.credentials.json``, without which the child cannot
    authenticate. ``.claude.json`` is written fresh rather than copied, so the
    user's project list does not travel into a workspace the model can read.
    Verified on this machine: a ``claude -p`` run under the redirect returns a
    result with ``is_error: false``.

    The directory sits beside the workspaces, never inside one. It used to live
    at ``<workspace>/.claude/dfa-home/claude-config``, and ``devforgeai commit``
    then saw its ``.claude.json`` as an untracked change in the story's worktree.
    """
    os.makedirs(config, exist_ok=True)
    source = os.path.join(os.path.expanduser("~"), ".claude", ".credentials.json")
    if os.path.isfile(source):
        try:
            shutil.copyfile(source, os.path.join(config, ".credentials.json"))
        except OSError:
            pass
    _write_text(os.path.join(config, ".claude.json"),
                json.dumps({"hasCompletedOnboarding": True}, indent=1))
    return config


def child_environment(home, devforgeai_bin, claude_config="isolate",
                      config_dir=""):
    """The environment the child gets: binary on PATH, session variables gone."""
    environment = dict(os.environ)
    environment["DEVFORGEAI_HOME"] = home
    if claude_config == "isolate":
        environment["CLAUDE_CONFIG_DIR"] = isolate_claude_config(
            config_dir or os.path.join(home, "claude-config"))
    # EVL-012: the skill preambles call `devforgeai`; a non-zero exit from a
    # !`...` injection aborts the whole skill invocation.
    environment["PATH"] = (os.path.dirname(os.path.abspath(devforgeai_bin))
                           + os.pathsep + environment.get("PATH", ""))
    # Inside a Claude Code session the inherited session variables and the
    # session-scoped API key make a nested `claude -p` hang. Strip them so the
    # child authenticates on its own; outside a session nothing is removed.
    if environment.get("CLAUDECODE"):
        for key in list(environment):
            if key.startswith("CLAUDE_CODE_") or key in SESSION_VARIABLES:
                environment.pop(key, None)
    return environment


def run_streaming(argv, workspace, environment, prompt, timeout, stdout_path):
    """Spawn the child and write its stdout to ``stdout_path`` as it arrives.

    ``subprocess.run`` buffers both streams and returns them at the end, so a
    case killed at its timeout left no log at all and no evidence of where it
    had reached — BLD-01 at 1500s produced an empty directory. Reading the pipe
    line by line on a thread and flushing each line means a kill preserves
    everything that ran.

    Returns ``(stdout, stderr, returncode)``; ``returncode`` is ``None`` when the
    timeout struck, and the partial stdout is returned and already on disk.
    """
    captured = []

    def pump(stream, sink, handle):
        for line in iter(stream.readline, ""):
            sink.append(line)
            if handle is not None:
                handle.write(line)
                handle.flush()
        stream.close()

    errors = []
    if stdout_path:
        os.makedirs(os.path.dirname(stdout_path), exist_ok=True)
    handle = (open(stdout_path, "w", encoding="utf-8", newline="\n")
              if stdout_path else None)
    try:
        child = subprocess.Popen(
            argv, cwd=workspace, env=environment, stdin=subprocess.PIPE,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
            encoding="utf-8", errors="replace", bufsize=1)
    except OSError:
        if handle is not None:
            handle.close()
        raise
    readers = [threading.Thread(target=pump, args=(child.stdout, captured, handle)),
               threading.Thread(target=pump, args=(child.stderr, errors, None))]
    for reader in readers:
        reader.daemon = True
        reader.start()
    try:
        try:
            child.stdin.write(prompt)
        finally:
            child.stdin.close()
    except OSError:
        pass
    try:
        code = child.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        child.kill()
        child.wait()
        code = None
    for reader in readers:
        reader.join(timeout=30)
    if handle is not None:
        handle.close()
    return "".join(captured), "".join(errors), code


def invoke(argv, workspace, home, timeout, prompt, options, log_dir, case_id):
    """Run claude in the workspace. Returns (status, transcript, exit, meta)."""
    environment = child_environment(
        home, options.devforgeai_bin, options.claude_config,
        os.path.join(options.workdir, "homes", case_id) if case_id else "")
    # PATH lookup here rather than in the spawn, so a .cmd shim on Windows
    # resolves the same way a bare name does on a shell.
    argv = [shutil.which(argv[0]) or argv[0]] + argv[1:]
    # EVL-002: the raw streams land beside the workspaces, never inside one, so
    # no grader's tree walk counts them as a write. stdout is written as it
    # arrives, so a case killed at its timeout still says where it reached.
    stdout_path = os.path.join(log_dir, case_id + ".stdout.txt") if log_dir else ""
    try:
        out, err, code = run_streaming(argv, workspace, environment, prompt,
                                       timeout, stdout_path)
    except OSError as exc:
        raise CaseError("claude did not start: %s" % exc)
    if log_dir:
        try:
            _write_text(os.path.join(log_dir, case_id + ".stderr.txt"), err or "")
        except OSError:
            pass
    if code is None:
        return "timeout", "", None, {}
    transcript, meta = read_stream(out or "")
    if meta.get("is_error") and LIMIT_RE.search(meta.get("result_text") or ""):
        return "limit", transcript, code, meta
    if code != 0 or meta.get("is_error"):   # EVL-016
        return "error", transcript, code, meta
    return "ran", transcript, code, meta


# ---------------------------------------------------------------------------
# graders
# ---------------------------------------------------------------------------

def load_graders(path, cases_path):
    if not os.path.isfile(path):
        raise UsageError("graders module not found: %s" % path)
    name = "dfa_graders_%s" % os.path.splitext(os.path.basename(cases_path))[0]
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise UsageError("graders module did not load: %s" % path)
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def grade(module, name, workspace, transcript, args):
    """Call the named grader. Returns (status, evidence)."""
    function = getattr(module, name, None)
    if function is None:
        return "error", "grader_missing: %s" % name
    try:
        passed, evidence = function(workspace, transcript, args)
    except Exception as exc:  # a grader defect stops one case, not the run
        import traceback
        last = traceback.format_exc().strip().splitlines()[-1]
        return "error", last or repr(exc)
    return ("pass" if passed else "fail"), str(evidence)


# ---------------------------------------------------------------------------
# one case
# ---------------------------------------------------------------------------

def run_case(case, files, options, module, run_id, skill_name):
    started = time.time()
    record = {
        "schema": SCHEMA,
        "run_id": run_id,
        "case_id": case["id"],
        "skill": skill_name,
        "model": options.model,
        "grader": case["expect"]["grader"],
        "status": "error",
        "passed": False,
        "evidence": "",
        "started_at": _utc(started),
        "finished_at": _utc(started),
        "duration_ms": 0,
        "exit_code": None,
        "workspace": "",
        "workspace_kept": bool(options.keep),
        "transcript_bytes": 0,
        "hooks": bool(options.hooks),
        "answers": sorted((case.get("answers") or {}).keys()),
        "log_stdout": "",
        "log_stderr": "",
        "session_id": None,
        "total_cost_usd": None,
        "num_turns": None,
        "permission_denials": [],
        "is_error": False,
        "tool_calls": {},
        "tampered": "",
    }
    workspace = None
    try:
        workspace, home = make_workspace(case, files, options)
        record["workspace"] = workspace
        record["log_stdout"] = os.path.join(options.log_dir, case["id"] + ".stdout.txt")
        record["log_stderr"] = os.path.join(options.log_dir, case["id"] + ".stderr.txt")
        argv = claude_argv(options.claude_bin, options.model, workspace,
                           bool(case.get("answers")), options.prompts_none)
        limit = case.get("timeout") or options.timeout
        before = guard_digests(workspace)
        status, transcript, code, meta = invoke(
            argv, workspace, home, limit, case["prompt"], options,
            options.log_dir, case["id"])
        record["exit_code"] = code
        record["transcript_bytes"] = len(transcript.encode("utf-8"))
        record.update({key: meta.get(key, record[key]) for key in (
            "session_id", "total_cost_usd", "num_turns", "permission_denials",
            "is_error", "tool_calls")})
        if status == "timeout":
            record["status"] = "timeout"
            record["evidence"] = "no result within %ds" % limit
        elif status == "limit":
            record["status"] = "limit"
            record["evidence"] = (
                "the account hit its usage limit before the run finished: %s"
                % (meta.get("result_text") or "").strip()[:200])
        elif status == "error":
            record["status"] = "error"
            record["evidence"] = (
                "claude exited %s%s; see %s"
                % (code,
                   " with is_error" if meta.get("is_error") else "",
                   record["log_stderr"]))
        else:
            status, evidence = grade(
                module, case["expect"]["grader"], workspace, transcript,
                case["expect"].get("args", {}))
            record["status"] = status
            record["evidence"] = evidence
        # After grading, and overriding it: a run that rewrote the files the
        # gate turns on has not earned whatever the grader said.
        changed = tampering(before, guard_digests(workspace))
        if changed:
            prior_status, prior_evidence = record["status"], record["evidence"]
            record["tampered"] = changed
            record["status"] = "tampered"
            record["evidence"] = (
                "the run changed a guarded file: %s (grader said %s: %s)"
                % (changed, prior_status, str(prior_evidence)[:200]))
    except CaseError as exc:
        record["evidence"] = str(exc)
    finally:
        if workspace and os.path.isdir(workspace) and not options.keep:
            _rmtree(workspace)
    finished = time.time()
    record["finished_at"] = _utc(finished)
    record["duration_ms"] = int(round((finished - started) * 1000))
    record["passed"] = record["status"] == "pass"
    record["evidence"] = record["evidence"][:EVIDENCE_CAP]
    return record


def preflight_case(case, files, options, skill_name):
    """Run the case's declared CLI preconditions in a throwaway workspace.

    A case names, under ``preflight``, the CLI calls the skill makes before it
    writes anything — the ``phase set`` of the phase, the id allocation that
    precedes it, the ``worktree ensure`` Build opens with. Each must exit 0 on
    the materialised fixture, because a non-zero exit there is the skill
    stopping on a defect the case shipped rather than on the behaviour the case
    grades. A TOML parse is not enough: the binary rejects a ``config.toml``
    with no ``generated_at`` that parses perfectly.

    An entry is a command string, or an object when the refusal *is* the
    precondition::

        {"command": "worktree ensure STORY-015", "exit": 1, "code": "DFA-E272"}

    BLD-07 is that case: its subject is the overlap refusal, so the check is
    that the call fails, with the code the grader then looks for, rather than
    that it succeeds. ``forbid_code``, with ``"exit": "any"``
    beside it, is the other shape: Build's last entry runs
    ``gate check --partial``, whose exit code is not the point — a story with no
    code yet fails its checks and a degraded fixture skips them — while
    ``DFA-E311``, the configured command not spawning at all, is a defect on
    either path and the one that entry exists to catch. ``forbid_code`` is the other half: Build's entry runs
    ``gate check --partial``, which is *expected* to report failing checks —
    there is no code yet — but must not report ``DFA-E311``, the configured
    command not spawning at all.

    The workspace is removed afterwards, because the calls mutate it.
    """
    commands = case.get("preflight")
    if not commands and case.get("degraded") is None:
        print("SKIP  %s  no preflight declared" % case["id"])
        return True
    workspace = None
    try:
        workspace, home = make_workspace(case, files, options)
        environment = child_environment(home, options.devforgeai_bin, "inherit")

        # Every seeded document under `.devforgeai/` resolves its references.
        # A case that cites an id nothing defines fails `build-docs` mid-run
        # with DFA-E210 and leaves as a SEND BACK before writing anything, which
        # grades the fixture rather than the skill (BLD-01 on REQ-007, vq-01 on
        # ADR-002 and AP-002). Warnings are not failures here; the gate's
        # `doc_valid` check does not fail on them either.
        unresolved = []
        for relative in sorted(files):
            if not relative.startswith(".devforgeai/"):
                continue
            if not relative.endswith((".md", ".yaml")):
                continue
            probe = subprocess.run(
                [options.devforgeai_bin, "doc", "validate", relative],
                cwd=workspace, env=environment, capture_output=True, text=True,
                timeout=300, encoding="utf-8", errors="replace")
            detail = (probe.stderr or probe.stdout or "")
            for line in detail.splitlines():
                if "DFA-E210" not in line:
                    continue
                found = re.search(r"references ([A-Z]+)-(\d{3})", line)
                # Scoped to the cross-references a phase's own readers resolve:
                # a CON, AP or ADR cited by a context file or an ADR, which is
                # what `standards-reviewer` reported as DFA-E325 on vq-01. An
                # `IDEA-` or `REQ-` citation points outside the workspace a
                # phase fixture builds and is not this check's business.
                if found and found.group(1) in ("CON", "AP", "ADR", "REQ"):
                    unresolved.append("%s: %s" % (relative, line.strip()[:120]))
        if unresolved:
            # A hard failure. An unresolved CON, AP, ADR or REQ is what made the
            # Verify reviewer report DFA-E325 on vq-01 and the build gate report
            # DFA-E210 on BLD-01, and Build's `context-validator` reads the same
            # set as Verify's `standards-reviewer`. A case citing an id nothing
            # defines grades its own fixture rather than the skill.
            print("FAIL  %s  %d seeded reference(s) resolve to nothing:\n    %s"
                  % (case["id"], len(unresolved), "\n    ".join(unresolved[:5])))
            return False

        # `make_workspace` has already run `stack detect`. A case that seeds
        # manual stacks must not come out of it degraded: a degraded config
        # skips every command check, so the gate would pass on nothing.
        wanted = case.get("degraded")
        if wanted is not None:
            actual = config_degraded(
                os.path.join(workspace, ".devforgeai", "config.toml"))
            if actual != wanted:
                print("FAIL  %s  config.toml degraded is %r after stack detect, "
                      "the case expects %r" % (case["id"], actual, wanted))
                return False
        for entry in commands:
            if isinstance(entry, str):
                command, wanted_exit, wanted_code, forbidden = entry, 0, "", ""
            else:
                command = entry["command"]
                wanted_exit = entry.get("exit", 0)
                wanted_code = entry.get("code", "")
                forbidden = entry.get("forbid_code", "")
            argv = [options.devforgeai_bin] + shlex.split(command, posix=True)
            finished = subprocess.run(
                argv, cwd=workspace, env=environment, capture_output=True,
                text=True, timeout=300, encoding="utf-8", errors="replace")
            detail = (finished.stderr or finished.stdout or "").strip()
            first = detail.splitlines()[0] if detail else ""
            if wanted_exit != "any" and finished.returncode != wanted_exit:
                print("FAIL  %s  devforgeai %s -> exit %d, the case expects %d: %s"
                      % (case["id"], command, finished.returncode, wanted_exit,
                         first))
                return False
            if forbidden and forbidden in detail:
                print("FAIL  %s  devforgeai %s -> %s: %s"
                      % (case["id"], command, forbidden, first))
                return False
            if wanted_code and wanted_code not in detail:
                print("FAIL  %s  devforgeai %s -> exit %d without %s: %s"
                      % (case["id"], command, finished.returncode, wanted_code,
                         first))
                return False
        print("ok    %s  %d call%s" % (case["id"], len(commands),
                                       "" if len(commands) == 1 else "s"))
        return True
    except CaseError as exc:
        print("ERROR %s  %s" % (case["id"], exc))
        return False
    finally:
        if workspace and os.path.isdir(workspace):
            _rmtree(workspace)


def dry_run_case(case, files, options, skill_name):
    """Materialise the workspace, print the invocation, run no process."""
    try:
        workspace, _home = make_workspace(case, files, options)
    except CaseError as exc:
        print("ERROR %s  %s" % (case["id"], exc))
        return False
    argv = claude_argv(options.claude_bin, options.model, workspace,
                       bool(case.get("answers")), options.prompts_none)
    print("case %s  grader %s  skill %s  files %d"
          % (case["id"], case["expect"]["grader"], skill_name, len(files)))
    print("  workspace %s" % workspace)
    print("  would run %s" % " ".join(
        ('"%s"' % part) if " " in part or "\n" in part else part for part in argv))
    # EVL-051: the prompt is no longer an argument, so a dry run prints it
    # separately or it would not be shown at all.
    prompt = case["prompt"]
    print("  prompt (%d bytes, %d lines) on stdin"
          % (len(prompt.encode("utf-8")), len(prompt.splitlines()) or 1))
    for line in prompt.splitlines() or [""]:
        print("    %s" % line)
    print("  hooks %s  answers %s"
          % ("framework + answer seed" if options.hooks else "answer seed only",
             ", ".join(sorted(case.get("answers") or {})) or "none"))
    return True


# ---------------------------------------------------------------------------
# command line
# ---------------------------------------------------------------------------

def parse_args(argv):
    parser = argparse.ArgumentParser(
        prog="run_jsonl.py", add_help=True,
        description="Run a skill's cases.jsonl through claude and its graders.")
    parser.add_argument("--cases", default=None)
    parser.add_argument("--skill", default=None)
    parser.add_argument("--graders", default=None)
    parser.add_argument("--out", default=None)
    parser.add_argument("--model", default="sonnet")
    # EVL-014, raised after the hooks-on runs: 900 came from `ex-08` measuring
    # 828s with hooks off, and enforcement adds a gate evaluation and a Stop
    # hook to every turn. PLAN-01 measured 850s, Verify and Constitute timed out
    # outright. A case may still narrow or widen it with its own `timeout`.
    parser.add_argument("--timeout", type=int, default=1500)
    parser.add_argument("--jobs", type=int, default=1)
    parser.add_argument("--filter", dest="filter", default="*")
    parser.add_argument("--case", dest="case", action="append", default=[])
    parser.add_argument("--workdir", default=None)
    parser.add_argument("--logdir", dest="log_dir", default=None)
    parser.add_argument("--keep", action="store_true")
    parser.add_argument("--claude-bin", dest="claude_bin", default="claude")
    parser.add_argument("--devforgeai-bin", dest="devforgeai_bin",
                        default=None)                           # EVL-012
    parser.add_argument("--framework-root", dest="framework_root", default=None)
    parser.add_argument("--claude-config", dest="claude_config",
                        choices=("isolate", "inherit"), default="isolate")
    parser.add_argument("--hooks", dest="hooks", action="store_true",
                        default=None)
    parser.add_argument("--no-hooks", dest="hooks", action="store_false")
    parser.add_argument("--dry-run", dest="dry_run", action="store_true")
    parser.add_argument("--preflight", dest="preflight", action="store_true")
    options = parser.parse_args(argv)

    options.framework_root = os.path.abspath(
        options.framework_root or framework_root())
    if not options.skill:
        raise UsageError("--skill is required")
    options.skill = os.path.abspath(options.skill)
    if not os.path.isdir(options.skill):
        raise UsageError("skill directory not found: %s" % options.skill)
    if options.cases is None:
        options.cases = os.path.join(options.skill, "evals", "cases.jsonl")
    options.cases = os.path.abspath(options.cases)
    if options.graders is None:
        options.graders = os.path.join(os.path.dirname(options.cases), "graders.py")
    options.graders = os.path.abspath(options.graders)
    if options.out is None:
        options.out = os.path.join(os.path.dirname(options.cases), "results.jsonl")
    options.out = os.path.abspath(options.out)
    if options.workdir is None:
        import tempfile
        options.workdir = tempfile.mkdtemp(prefix="dfa-evals-")
    options.workdir = os.path.abspath(options.workdir)
    os.makedirs(options.workdir, exist_ok=True)
    if options.log_dir is None:
        options.log_dir = os.path.join(options.workdir, "logs")
    options.log_dir = os.path.abspath(options.log_dir)
    if options.devforgeai_bin is None:
        options.devforgeai_bin = default_binary(options.framework_root)
    options.devforgeai_bin = os.path.abspath(options.devforgeai_bin)
    if not os.path.isfile(options.devforgeai_bin):
        raise UsageError(
            "devforgeai binary not found: %s; build cli/ or pass "
            "--devforgeai-bin" % options.devforgeai_bin)
    if options.timeout < 1:
        raise UsageError("--timeout takes a positive number of seconds")
    if options.jobs < 1:
        raise UsageError("--jobs takes a positive count")
    if options.dry_run:
        options.keep = True
    if options.preflight:
        options.keep = False
    options.hooks_explicit = options.hooks is True
    if options.hooks is None:
        options.hooks = True
    # R2: the dry run prints the argv the real run uses, so the version probe
    # runs in every mode. One `claude --version` exec, cached for the process.
    options.prompts_none = version_supports_prompts_none(options.claude_bin)
    return options


def trust_gate(options):
    """Hooks-on evals depend on a trust pin a human made on this machine.

    The release binary reads ``~/.devforgeai/trust.toml`` and honours
    ``DEVFORGEAI_HOME`` only under the ``test-home`` cargo feature, so an
    isolated home does not stand in for a pin. Without one the PreToolUse
    ``trust-check`` arm denies every Write, Edit and Bash in every case.
    """
    if not options.hooks:
        return True, ""
    try:
        finished = subprocess.run(
            [options.devforgeai_bin, "trust", "verify"],
            capture_output=True, text=True, timeout=120,
            encoding="utf-8", errors="replace")
    except (OSError, subprocess.SubprocessError) as exc:
        return False, "trust verify did not run: %s" % exc
    if finished.returncode == 0:
        return True, ""
    reason = (finished.stderr or finished.stdout or "").strip().splitlines()
    return False, (reason[0] if reason else
                   "trust verify exited %d" % finished.returncode)


def select(cases, options):
    chosen = [case for case in cases if fnmatch.fnmatch(case["id"], options.filter)]
    if options.case:
        wanted = set(options.case)
        unknown = wanted - set(case["id"] for case in cases)
        if unknown:
            raise UsageError("no case with id %s" % ", ".join(sorted(unknown)))
        chosen = [case for case in chosen if case["id"] in wanted]
    if not chosen:
        raise UsageError("no case id matched")
    return chosen


def summarise(records, elapsed, out_path):
    counts = {"pass": 0, "fail": 0, "error": 0, "timeout": 0, "limit": 0,
              "tampered": 0}
    cost = 0.0
    for record in records:
        counts[record["status"]] += 1
        if isinstance(record.get("total_cost_usd"), (int, float)):
            cost += float(record["total_cost_usd"])
    print("cases %d  pass %d  fail %d  error %d  timeout %d  limit %d  "
          "tampered %d  duration %.1fs  cost $%.2f"
          % (len(records), counts["pass"], counts["fail"], counts["error"],
             counts["timeout"], counts["limit"], counts["tampered"],
             elapsed, cost))
    for record in records:
        if record["status"] == "pass":
            continue
        print("%s  %s  %s: %s" % (record["status"].upper(), record["case_id"],
                                  record["grader"], record["evidence"]))
    print("results: %s" % out_path)
    if (counts["error"] or counts["timeout"] or counts["limit"]
            or counts["tampered"]):
        return 2
    if counts["fail"]:
        return 1
    return 0


def main(argv=None):
    try:
        options = parse_args(sys.argv[1:] if argv is None else argv)
        cases = load_cases(options.cases)
        chosen = select(cases, options)
        prepared = [(case, resolve_fixtures(merged_files(case, cases), options.skill))
                    for case in chosen]
    except UsageError as exc:
        sys.stderr.write("usage error: %s\n" % exc)
        return 3

    skill_name = os.path.basename(os.path.normpath(options.skill))

    if options.preflight:
        clean = True
        for case, files in prepared:
            clean = preflight_case(case, files, options, skill_name) and clean
        print("cases %d  preflight, claude not invoked" % len(prepared))
        return 0 if clean else 2

    if not options.dry_run:
        ok, reason = trust_gate(options)
        if not ok:
            if options.hooks_explicit:
                sys.stderr.write(
                    "usage error: --hooks needs a trust pin: %s\n  %s\n"
                    % (reason, PIN_HINT % options.framework_root))
                return 3
            options.hooks = False
            print("hooks off: %s" % reason)
            print("  pin to run the enforcement path: %s"
                  % (PIN_HINT % options.framework_root))

    if options.dry_run:
        clean = True
        for case, files in prepared:
            clean = dry_run_case(case, files, options, skill_name) and clean
        print("cases %d  dry run, claude not invoked, workspaces kept in %s"
              % (len(prepared), options.workdir))
        return 0 if clean else 2

    try:
        module = load_graders(options.graders, options.cases)
    except UsageError as exc:
        sys.stderr.write("usage error: %s\n" % exc)
        return 3

    run_id = _run_id(time.time())
    started = time.time()
    lock = threading.Lock()
    records = []

    def one(case_files):
        case, files = case_files
        record = run_case(case, files, options, module, run_id, skill_name)
        with lock:
            with open(options.out, "a", encoding="utf-8", newline="\n") as handle:
                handle.write(json.dumps(record) + "\n")
            records.append(record)
        return record

    if options.jobs == 1:
        for item in prepared:
            one(item)
    else:
        with concurrent.futures.ThreadPoolExecutor(max_workers=options.jobs) as pool:
            list(pool.map(one, prepared))

    order = {case["id"]: index for index, (case, _files) in enumerate(prepared)}
    records.sort(key=lambda record: order[record["case_id"]])
    return summarise(records, time.time() - started, options.out)


if __name__ == "__main__":
    sys.exit(main())
