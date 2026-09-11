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
* the raw streams are written to a log directory beside the workspaces, never
  inside one, so no grader's tree walk counts them as a write.

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
        if relative in files or name not in defaults:
            continue
        _write_text(os.path.join(workspace, ".devforgeai", name), defaults[name])

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

    workspace_settings(case, claude_dir, options)
    git_spec = (case.get("setup") or {}).get("git")
    if git_spec:
        init_git(workspace, git_spec)
    # Decision 62: a home inside the workspace, so a debug build reads no trust
    # pin from the real machine. A release build ignores it (no `test-home`
    # feature), which is why hooks-on runs are gated on a real pin instead.
    home = os.path.join(claude_dir, "dfa-home")
    os.makedirs(home, exist_ok=True)
    return workspace, home


# ---------------------------------------------------------------------------
# claude
# ---------------------------------------------------------------------------

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


def version_supports_prompts_none(binary):
    """True when the installed claude is 2.1.259 or newer (guidance §4)."""
    try:
        out = subprocess.run([shutil.which(binary) or binary, "--version"],
                             capture_output=True, text=True, timeout=60).stdout
    except (OSError, subprocess.SubprocessError):
        return False
    match = re.search(r"(\d+)\.(\d+)\.(\d+)", out or "")
    return bool(match) and tuple(int(g) for g in match.groups()) >= (2, 1, 259)


def isolate_claude_config(home):
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
    """
    config = os.path.join(home, "claude-config")
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


def child_environment(home, devforgeai_bin, claude_config="isolate"):
    """The environment the child gets: binary on PATH, session variables gone."""
    environment = dict(os.environ)
    environment["DEVFORGEAI_HOME"] = home
    if claude_config == "isolate":
        environment["CLAUDE_CONFIG_DIR"] = isolate_claude_config(home)
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


def invoke(argv, workspace, home, timeout, prompt, options, log_dir, case_id):
    """Run claude in the workspace. Returns (status, transcript, exit, meta)."""
    environment = child_environment(home, options.devforgeai_bin,
                                    options.claude_config)
    # PATH lookup here rather than in the spawn, so a .cmd shim on Windows
    # resolves the same way a bare name does on a shell.
    argv = [shutil.which(argv[0]) or argv[0]] + argv[1:]
    try:
        finished = subprocess.run(
            argv, cwd=workspace, env=environment, input=prompt,
            capture_output=True, timeout=timeout, text=True,
            encoding="utf-8", errors="replace")
    except subprocess.TimeoutExpired:
        return "timeout", "", None, {}
    except OSError as exc:
        raise CaseError("claude did not start: %s" % exc)
    # EVL-002: the raw streams land beside the workspaces, never inside one, so
    # no grader's tree walk counts them as a write.
    if log_dir:
        try:
            _write_text(os.path.join(log_dir, case_id + ".stdout.txt"),
                        finished.stdout or "")
            _write_text(os.path.join(log_dir, case_id + ".stderr.txt"),
                        finished.stderr or "")
        except OSError:
            pass
    transcript, meta = read_stream(finished.stdout or "")
    if meta.get("is_error") and LIMIT_RE.search(meta.get("result_text") or ""):
        return "limit", transcript, finished.returncode, meta
    if finished.returncode != 0 or meta.get("is_error"):   # EVL-016
        return "error", transcript, finished.returncode, meta
    return "ran", transcript, finished.returncode, meta


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

    The workspace is removed afterwards, because the calls mutate it.
    """
    commands = case.get("preflight")
    if not commands:
        print("SKIP  %s  no preflight declared" % case["id"])
        return True
    workspace = None
    try:
        workspace, home = make_workspace(case, files, options)
        environment = child_environment(home, options.devforgeai_bin, "inherit")
        for command in commands:
            argv = [options.devforgeai_bin] + shlex.split(command, posix=True)
            finished = subprocess.run(
                argv, cwd=workspace, env=environment, capture_output=True,
                text=True, timeout=300, encoding="utf-8", errors="replace")
            if finished.returncode != 0:
                detail = (finished.stderr or finished.stdout or "").strip()
                print("FAIL  %s  devforgeai %s -> exit %d: %s"
                      % (case["id"], command, finished.returncode,
                         detail.splitlines()[0] if detail else ""))
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
    parser.add_argument("--timeout", type=int, default=900)     # EVL-014
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
    options.prompts_none = (
        False if (options.dry_run or options.preflight)
        else version_supports_prompts_none(options.claude_bin))
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
    counts = {"pass": 0, "fail": 0, "error": 0, "timeout": 0, "limit": 0}
    cost = 0.0
    for record in records:
        counts[record["status"]] += 1
        if isinstance(record.get("total_cost_usd"), (int, float)):
            cost += float(record["total_cost_usd"])
    print("cases %d  pass %d  fail %d  error %d  timeout %d  limit %d  "
          "duration %.1fs  cost $%.2f"
          % (len(records), counts["pass"], counts["fail"], counts["error"],
             counts["timeout"], counts["limit"], elapsed, cost))
    for record in records:
        if record["status"] == "pass":
            continue
        print("%s  %s  %s: %s" % (record["status"].upper(), record["case_id"],
                                  record["grader"], record["evidence"]))
    print("results: %s" % out_path)
    if counts["error"] or counts["timeout"] or counts["limit"]:
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
