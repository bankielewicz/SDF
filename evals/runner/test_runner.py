#!/usr/bin/env python3
"""Unit tests for the shared eval runner.

The tests build a small framework tree in a temporary directory and put a fake
``claude`` on PATH, so no test reaches the real CLI or the network. The fake
reads the prompt from stdin and answers in ``stream-json``, which is what the
runner now sends and parses.

Run: ``python -m unittest evals/runner/test_runner.py``
"""

import io
import json
import os
import shutil
import sys
import tempfile
import unittest
import contextlib

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import run_jsonl  # noqa: E402


FIXTURE_TEXT = "fixture body\nsecond line\n"

GRADERS = '''
def ok_grader(workspace, transcript, args):
    return True, "saw %d bytes" % len(transcript)


def fail_grader(workspace, transcript, args):
    return False, "no marker in the transcript"


def boom_grader(workspace, transcript, args):
    raise ValueError("kaboom")
'''

# Reads the prompt from stdin (never argv) and emits stream-json: one partial
# `stream_event` whose text a later `assistant` event repeats, one tool_use, one
# tool_result, and the `result` envelope with the meta fields.
FAKE_CLAUDE = r'''
import json, os, sys
argv = sys.argv[1:]
if "--version" in argv:
    print("2.1.268 (Claude Code)")
    raise SystemExit(0)
prompt = sys.stdin.read()
with open(os.path.join(os.getcwd(), "claude-ran.txt"), "w") as handle:
    handle.write(prompt)
with open(os.path.join(os.getcwd(), "claude-argv.txt"), "w") as handle:
    handle.write("\n".join(argv))
def emit(obj):
    sys.stdout.write(json.dumps(obj) + "\n")
emit({"type": "stream_event", "event": {"type": "content_block_delta",
      "delta": {"type": "text_delta", "text": "TRANSCRIPT for " + prompt}}})
emit({"type": "assistant", "message": {"content": [
      {"type": "text", "text": "TRANSCRIPT for " + prompt},
      {"type": "tool_use", "name": "Bash",
       "input": {"command": "devforgeai worktree ensure STORY-015"}},
      {"type": "tool_use", "name": "Agent",
       "input": {"subagent_type": "ac-test-writer", "prompt": "AC-001 (x)"}}]}})
emit({"type": "user", "message": {"content": [
      {"type": "tool_result", "content": "TOOLRESULT ok"}]}})
emit({"type": "result", "result": "FINAL LINE", "session_id": "sess-1",
      "total_cost_usd": 0.25, "num_turns": 3, "permission_denials": [],
      "is_error": False, "subtype": "success"})
'''

CASES = [
    {"id": "c1", "prompt": "first",
     "setup": {"files": {"a.txt": "one", "b.txt": "FIXTURE:hello.txt"}},
     "expect": {"grader": "ok_grader", "args": {}}},
    {"id": "c2", "prompt": "second\nwith a newline",
     "answers": {"Decision": "Promote"},
     "setup": {"extends": "c1", "files": {"a.txt": "two", "c.txt": "three"}},
     "expect": {"grader": "ok_grader", "args": {}}},
    {"id": "c3", "prompt": "third",
     "setup": {"files": {}},
     "expect": {"grader": "fail_grader", "args": {}}},
    {"id": "c4", "prompt": "fourth",
     "setup": {"files": {}},
     "expect": {"grader": "absent_grader", "args": {}}},
    {"id": "c5", "prompt": "fifth",
     "setup": {"files": {"../outside.txt": "escaped"}},
     "expect": {"grader": "ok_grader", "args": {}}},
]

HOOK_TEMPLATE = {
    "hooks": {
        "PreToolUse": [
            {"matcher": "Write|Edit|NotebookEdit|Bash|PowerShell|Agent",
             "hooks": [{"type": "command", "command": "@@DEVFORGEAI@@",
                        "args": ["hook", "run", "trust-check"], "timeout": 600}]}
        ],
        "PostToolUse": [
            {"matcher": "Bash|PowerShell",
             "hooks": [{"type": "command", "command": "@@DEVFORGEAI@@",
                        "args": ["hook", "run", "post-tool-use"],
                        "if": "Bash(@@TEST_COMMAND@@)", "timeout": 900}]}
        ],
        "SubagentStop": [
            {"matcher": "@@VERIFIERS@@",
             "hooks": [{"type": "command", "command": "@@DEVFORGEAI@@",
                        "args": ["hook", "run", "subagent-stop"], "timeout": 60}]}
        ],
    }
}


def read_lines(path):
    with open(path, encoding="utf-8") as handle:
        return [json.loads(line) for line in handle if line.strip()]


def write(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8", newline="\n") as handle:
        handle.write(text)


class RunnerTestCase(unittest.TestCase):

    def setUp(self):
        self.root = tempfile.mkdtemp(prefix="dfa-runner-test-")
        self.addCleanup(shutil.rmtree, self.root, True)

        self.skill = os.path.join(self.root, "skills", "exploring-ideas")
        write(os.path.join(self.skill, "SKILL.md"), "---\nname: explore\n---\n")
        write(os.path.join(self.skill, "agents.md"),
              "# Subagents\n\n## `flow-drafter`\n\ntext\n\n## `Read`\n\nnot an agent\n")
        write(os.path.join(self.skill, "evals", "fixtures", "hello.txt"), FIXTURE_TEXT)
        write(os.path.join(self.skill, "evals", "graders.py"), GRADERS)
        write(os.path.join(self.skill, "evals", "cases.jsonl"),
              "".join(json.dumps(case) + "\n" for case in CASES))
        os.makedirs(os.path.join(self.skill, "evals", "__pycache__"))
        write(os.path.join(self.skill, "evals", "__pycache__", "graders.pyc"), "x")

        write(os.path.join(self.root, "agents", "flow-drafter.md"), "flow drafter\n")
        write(os.path.join(self.root, "agents", "unused-agent.md"), "unused\n")
        write(os.path.join(self.root, "hooks", "settings.hooks.json"),
              json.dumps(HOOK_TEMPLATE, indent=1))

        self.cases_path = os.path.join(self.skill, "evals", "cases.jsonl")
        self.workdir = os.path.join(self.root, "work")
        os.makedirs(self.workdir)

        bindir = os.path.join(self.root, "bin")
        os.makedirs(bindir)
        fake = os.path.join(bindir, "fake_claude.py")
        write(fake, FAKE_CLAUDE)
        write(os.path.join(bindir, "claude.cmd"),
              '@echo off\r\n"%s" "%%~dp0fake_claude.py" %%*\r\n' % sys.executable)
        shell = os.path.join(bindir, "claude")
        write(shell, '#!/bin/sh\nexec "%s" "$(dirname "$0")/fake_claude.py" "$@"\n'
              % sys.executable)
        os.chmod(shell, 0o755)
        # A stand-in for the release binary: the runner needs the file to exist
        # to put its directory on PATH, and runs it only for `trust verify`.
        self.devforgeai = os.path.join(
            bindir, "devforgeai.exe" if os.name == "nt" else "devforgeai")
        write(self.devforgeai, "not a real binary\n")
        self.previous_path = os.environ["PATH"]
        os.environ["PATH"] = bindir + os.pathsep + self.previous_path
        self.addCleanup(self._restore_path)

    def _restore_path(self):
        os.environ["PATH"] = self.previous_path

    def _cases(self):
        return run_jsonl.load_cases(self.cases_path)

    def _files(self, case_id):
        cases = self._cases()
        case = [item for item in cases if item["id"] == case_id][0]
        return run_jsonl.resolve_fixtures(
            run_jsonl.merged_files(case, cases), self.skill)

    def _argv(self, extra):
        return ["--skill", self.skill, "--workdir", self.workdir,
                "--framework-root", self.root,
                "--devforgeai-bin", self.devforgeai, "--no-hooks"] + list(extra)

    def _main(self, extra):
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code = run_jsonl.main(self._argv(extra))
        return code, buffer.getvalue()

    def _options(self, extra=()):
        return run_jsonl.parse_args(self._argv(extra))

    # -- FIXTURE substitution ------------------------------------------------

    def test_fixture_value_is_replaced_by_the_fixture_bytes(self):
        files = self._files("c1")
        self.assertEqual(files["b.txt"], FIXTURE_TEXT)
        self.assertEqual(files["a.txt"], "one")

    def test_absent_fixture_is_a_usage_error(self):
        with self.assertRaises(run_jsonl.UsageError):
            run_jsonl.resolve_fixtures({"x": "FIXTURE:nope.md"}, self.skill)

    def test_absent_fixture_exits_three(self):
        write(self.cases_path, json.dumps(
            {"id": "z", "prompt": "p", "setup": {"files": {"x": "FIXTURE:nope.md"}},
             "expect": {"grader": "ok_grader"}}) + "\n")
        code, _ = self._main([])
        self.assertEqual(code, 3)

    # -- setup.extends -------------------------------------------------------

    def test_extends_merges_parent_first_and_child_over_it(self):
        files = self._files("c2")
        self.assertEqual(files["a.txt"], "two")
        self.assertEqual(files["b.txt"], FIXTURE_TEXT)
        self.assertEqual(files["c.txt"], "three")
        self.assertEqual(list(files), ["a.txt", "b.txt", "c.txt"])

    def test_extends_chain_resolves_outermost_first(self):
        cases = [
            {"id": "base", "prompt": "p", "setup": {"files": {"f": "base", "k": "b"}},
             "expect": {"grader": "ok_grader"}},
            {"id": "mid", "prompt": "p",
             "setup": {"extends": "base", "files": {"f": "mid", "m": "m"}},
             "expect": {"grader": "ok_grader"}},
            {"id": "leaf", "prompt": "p",
             "setup": {"extends": "mid", "files": {"f": "leaf"}},
             "expect": {"grader": "ok_grader"}},
        ]
        merged = run_jsonl.merged_files(cases[2], cases)
        self.assertEqual(merged, {"f": "leaf", "k": "b", "m": "m"})

    def test_extends_forward_reference_is_a_usage_error(self):
        cases = [
            {"id": "first", "prompt": "p", "setup": {"extends": "second"},
             "expect": {"grader": "ok_grader"}},
            {"id": "second", "prompt": "p", "setup": {"files": {}},
             "expect": {"grader": "ok_grader"}},
        ]
        with self.assertRaises(run_jsonl.UsageError):
            run_jsonl.merged_files(cases[0], cases)

    def test_extends_cycle_is_a_usage_error(self):
        cases = [
            {"id": "a", "prompt": "p", "setup": {"extends": "b"},
             "expect": {"grader": "ok_grader"}},
            {"id": "b", "prompt": "p", "setup": {"extends": "a"},
             "expect": {"grader": "ok_grader"}},
        ]
        with self.assertRaises(run_jsonl.UsageError):
            run_jsonl.merged_files(cases[1], cases)

    # -- answers validation (EVL-030) ---------------------------------------

    def test_answers_may_be_a_string_or_a_list_of_strings(self):
        cases = self._cases()
        self.assertEqual(cases[1]["answers"], {"Decision": "Promote"})

    def test_answers_of_the_wrong_type_is_a_usage_error(self):
        write(self.cases_path, json.dumps(
            {"id": "z", "prompt": "p", "answers": {"Decision": 7},
             "expect": {"grader": "ok_grader"}}) + "\n")
        with self.assertRaises(run_jsonl.UsageError):
            run_jsonl.load_cases(self.cases_path)

    # -- argv and the prompt on stdin (EVL-010, EVL-011, EVL-051) ------------

    def test_argv_carries_stream_json_and_the_shell_grant_and_no_prompt(self):
        argv = run_jsonl.claude_argv("claude", "sonnet", "/ws", False, True)
        self.assertNotIn("PROMPT", " ".join(argv))
        joined = " ".join(argv)
        self.assertIn("--output-format stream-json --verbose "
                      "--include-partial-messages", joined)
        # Headless has no approval surface, so a compound shell command is
        # denied under every prompting mode; the workspace is a throwaway.
        self.assertIn("--permission-mode bypassPermissions", joined)
        self.assertIn("Bash(devforgeai *)", joined)
        self.assertIn("PowerShell(devforgeai *)", joined)
        self.assertIn("--permission-prompts none", joined)
        self.assertEqual(argv[1], "-p")
        self.assertNotIn("--output-format json", joined.replace("stream-json", "x"))

    def test_a_seeded_case_names_the_permission_host_instead(self):
        argv = run_jsonl.claude_argv("claude", "sonnet", "/ws", True, True)
        self.assertNotIn("--permission-prompts", argv)
        self.assertIn("--permission-prompt-tool", argv)
        self.assertEqual(argv[argv.index("--permission-prompt-tool") + 1],
                         "mcp__dfa-permissions__approve")
        config = argv[argv.index("--mcp-config") + 1]
        self.assertTrue(config.endswith("mcp-permissions.json"))
        self.assertTrue(run_jsonl._inside("/ws", config))

    def test_a_seeded_case_gets_the_host_script_and_its_config(self):
        options = self._options(["--case", "c2"])
        workspace, _home = run_jsonl.make_workspace(
            self._cases()[1], self._files("c2"), options)
        claude = os.path.join(workspace, ".claude")
        self.assertTrue(os.path.isfile(os.path.join(claude, "permission_host.py")))
        with open(os.path.join(claude, "mcp-permissions.json"),
                  encoding="utf-8") as handle:
            config = json.load(handle)
        server = config["mcpServers"]["dfa-permissions"]
        self.assertEqual(server["command"], sys.executable)
        self.assertTrue(server["args"][0].endswith("permission_host.py"))
        self.assertTrue(run_jsonl._inside(workspace, server["args"][0]))

    def test_an_unseeded_case_gets_no_host(self):
        options = self._options(["--case", "c1"])
        workspace, _home = run_jsonl.make_workspace(
            self._cases()[0], self._files("c1"), options)
        self.assertFalse(os.path.exists(
            os.path.join(workspace, ".claude", "mcp-permissions.json")))

    def test_permission_host_allows_a_call_with_the_input_unchanged(self):
        import subprocess
        script = os.path.join(os.path.dirname(os.path.abspath(
            run_jsonl.__file__)), "permission_host.py")
        messages = [
            {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}},
            {"jsonrpc": "2.0", "method": "notifications/initialized"},
            {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
            {"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {
                "name": "approve",
                "arguments": {"tool_name": "AskUserQuestion",
                              "input": {"questions": [{"header": "Decision"}]}}}},
        ]
        finished = subprocess.run(
            [sys.executable, script],
            input="\n".join(json.dumps(m) for m in messages) + "\n",
            capture_output=True, text=True, timeout=60)
        replies = [json.loads(line) for line in finished.stdout.splitlines()
                   if line.strip()]
        by_id = {r["id"]: r for r in replies}
        self.assertEqual(by_id[1]["result"]["serverInfo"]["name"],
                         "dfa-permissions")
        self.assertEqual([t["name"] for t in by_id[2]["result"]["tools"]],
                         ["approve"])
        decision = json.loads(by_id[3]["result"]["content"][0]["text"])
        self.assertEqual(decision["behavior"], "allow")
        self.assertEqual(decision["updatedInput"],
                         {"questions": [{"header": "Decision"}]})

    def test_prompt_reaches_the_child_on_stdin_with_its_newlines(self):
        code, _ = self._main(["--case", "c2", "--keep"])
        self.assertEqual(code, 0)
        workspace = [os.path.join(self.workdir, name)
                     for name in os.listdir(self.workdir)
                     if name.startswith("dfa-eval-c2-")][0]
        with open(os.path.join(workspace, "claude-ran.txt"), encoding="utf-8") as h:
            self.assertEqual(h.read(), "second\nwith a newline")
        with open(os.path.join(workspace, "claude-argv.txt"), encoding="utf-8") as h:
            argv = h.read().splitlines()
        self.assertNotIn("second", "".join(argv))
        self.assertIn("stream-json", argv)

    def test_binary_directory_is_on_the_child_path(self):
        environment = run_jsonl.child_environment("/home", self.devforgeai)
        first = environment["PATH"].split(os.pathsep)[0]
        self.assertEqual(os.path.abspath(first),
                         os.path.dirname(os.path.abspath(self.devforgeai)))
        self.assertEqual(environment["DEVFORGEAI_HOME"], "/home")

    def test_claude_config_is_redirected_into_the_workspace_home(self):
        home = os.path.join(self.root, "home")
        os.makedirs(home)
        environment = run_jsonl.child_environment(home, self.devforgeai)
        config = environment["CLAUDE_CONFIG_DIR"]
        self.assertTrue(run_jsonl._inside(home, config))
        self.assertTrue(os.path.isfile(os.path.join(config, ".claude.json")))
        # HOME and USERPROFILE stay where they were: the binary's home checks
        # and git's identity read the real profile.
        self.assertEqual(environment.get("USERPROFILE"),
                         os.environ.get("USERPROFILE"))
        self.assertEqual(environment.get("HOME"), os.environ.get("HOME"))

    def test_claude_config_inherit_sets_nothing(self):
        home = os.path.join(self.root, "home2")
        os.makedirs(home)
        environment = run_jsonl.child_environment(home, self.devforgeai, "inherit")
        self.assertEqual(environment.get("CLAUDE_CONFIG_DIR"),
                         os.environ.get("CLAUDE_CONFIG_DIR"))

    def test_absent_binary_is_a_usage_error(self):
        code, _ = self._main(["--devforgeai-bin",
                              os.path.join(self.root, "nope.exe")])
        self.assertEqual(code, 3)

    def test_a_case_timeout_overrides_the_run_default(self):
        cases = [dict(CASES[0]), dict(CASES[1])]
        cases[0]["timeout"] = 1500
        write(self.cases_path,
              "".join(json.dumps(case) + chr(10) for case in cases))
        parsed = run_jsonl.load_cases(self.cases_path)
        self.assertEqual(parsed[0]["timeout"], 1500)
        self.assertIsNone(parsed[1].get("timeout"))

        seen = []
        real_invoke = run_jsonl.invoke

        def spy(argv, workspace, home, timeout, prompt, options, log_dir, case_id):
            seen.append((case_id, timeout))
            return real_invoke(argv, workspace, home, timeout, prompt, options,
                               log_dir, case_id)

        run_jsonl.invoke = spy
        self.addCleanup(setattr, run_jsonl, "invoke", real_invoke)
        self._main(["--out", os.path.join(self.root, "r.jsonl"),
                    "--timeout", "300"])
        self.assertEqual(dict(seen)["c1"], 1500)
        self.assertEqual(dict(seen)["c2"], 300)

    def test_a_timeout_that_is_not_a_positive_integer_is_a_usage_error(self):
        for bad in ("900", 0, -1, True, 1.5):
            write(self.cases_path, json.dumps(
                {"id": "z", "prompt": "p", "timeout": bad,
                 "expect": {"grader": "ok_grader"}}) + chr(10))
            with self.assertRaises(run_jsonl.UsageError):
                run_jsonl.load_cases(self.cases_path)

    def test_every_long_phase_case_declares_a_timeout(self):
        import glob
        root = run_jsonl.framework_root()
        # The phases whose `## Documents` table names six or more files, plus
        # the two that measured past the 900s default: Plan at 850s on PLAN-01,
        # and Build, which times out running a three-criterion TDD loop with two
        # subagents per criterion and the lint, coverage and complexity commands.
        # Build is 2400: with hooks on, the Stop hook re-runs the gate — the
        # test command included — once per block, and 1500 was not enough.
        wanted = {"implementing-stories": 2400}
        for skill in ("establishing-context", "designing-interfaces",
                      "exploring-ideas", "releasing-software", "planning-work",
                      "implementing-stories", "validating-quality"):
            path = os.path.join(root, "skills", skill, "evals", "cases.jsonl")
            if not os.path.isfile(path):
                self.skipTest("%s is not in this tree" % skill)
            for line in open(path, encoding="utf-8"):
                if not line.strip():
                    continue
                case = json.loads(line)
                self.assertEqual(case.get("timeout"), wanted.get(skill, 1500),
                                 "%s %s" % (skill, case["id"]))

    # -- the stream reader (EVL-010) -----------------------------------------

    def test_read_stream_renders_text_tool_calls_results_and_meta(self):
        stdout = "\n".join([
            json.dumps({"type": "stream_event", "event": {
                "type": "content_block_delta",
                "delta": {"type": "text_delta", "text": "partial"}}}),
            "not json at all",
            json.dumps({"type": "assistant", "message": {"content": [
                {"type": "text", "text": "hello"},
                {"type": "tool_use", "name": "Bash", "input": {
                    "command": "devforgeai worktree ensure STORY-015"}},
                {"type": "tool_use", "name": "Agent", "input": {
                    "subagent_type": "ac-test-writer",
                    "prompt": "write AC-003 (now)"}}]}}),
            json.dumps({"type": "user", "message": {"content": [
                {"type": "tool_result", "content": [
                    {"type": "text", "text": "Gate      PASS"}]}]}}),
            json.dumps({"type": "result", "result": "the handoff block",
                        "session_id": "s", "total_cost_usd": 1.5,
                        "num_turns": 9, "permission_denials": [{"tool": "Bash"}],
                        "is_error": False, "subtype": "success"}),
        ])
        transcript, meta = run_jsonl.read_stream(stdout)
        self.assertEqual(transcript.count("partial"), 0)
        self.assertIn("devforgeai worktree ensure STORY-015", transcript)
        self.assertIn("Gate      PASS", transcript)
        self.assertTrue(transcript.endswith("the handoff block"))
        self.assertRegex(transcript, r"Agent\(ac-test-writer[^)]*?(AC-\d{3})")
        self.assertEqual(meta["total_cost_usd"], 1.5)
        self.assertEqual(meta["num_turns"], 9)
        self.assertEqual(meta["session_id"], "s")
        self.assertEqual(meta["permission_denials"], [{"tool": "Bash"}])
        self.assertEqual(meta["tool_calls"], {"Bash": 1, "Agent": 1})

    def test_read_stream_keeps_a_plain_string_user_turn(self):
        """A refused preamble arrives as `message.content` being a bare string.

Claude Code emits one when it refuses a `!` injection — measured:
a preamble carrying `$1` is refused with "Shell command permission check
        failed ... Contains simple_expansion", and the refusal reaches the
transcript as `<local-command-stderr>...`. The reader used to call
`.get` on it and the whole run died with AttributeError, so the case
that would have shown the refusal recorded a runner crash instead.
"""
        stdout = "\n".join([
            json.dumps({"type": "user", "message": {"content":
                "<local-command-stderr>Shell command permission check failed for "
                "command: devforgeai gate require plan $1. Contains "
                "simple_expansion</local-command-stderr>"}}),
            json.dumps({"type": "assistant", "message": {"content": [
                {"type": "text", "text": "stopping"}]}}),
            json.dumps({"type": "user", "message": {"content": [
                "a bare string inside the list", 7,
                {"type": "tool_result", "content": "result text"}]}}),
            json.dumps({"type": "result", "result": "done", "session_id": "s",
                        "total_cost_usd": 0.1, "num_turns": 2,
                        "permission_denials": [], "is_error": False,
                        "subtype": "success"}),
        ])
        transcript, meta = run_jsonl.read_stream(stdout)
        self.assertIn("Contains simple_expansion", transcript)
        self.assertIn("<local-command-stderr>", transcript)
        self.assertIn("result text", transcript)
        self.assertIn("stopping", transcript)
        self.assertNotIn("a bare string inside the list", transcript)
        self.assertTrue(transcript.endswith("done"))
        self.assertEqual(meta["num_turns"], 2)

    def test_a_usage_limit_is_recorded_apart_from_an_error(self):
        stdout = json.dumps({
            "type": "result",
            "result": "You've hit your session limit - resets 1pm",
            "session_id": "s", "total_cost_usd": 0, "num_turns": 1,
            "permission_denials": [], "is_error": True, "subtype": "success"})
        transcript, meta = run_jsonl.read_stream(stdout)
        self.assertTrue(meta["is_error"])
        self.assertTrue(run_jsonl.LIMIT_RE.search(meta["result_text"]))
        self.assertIn("session limit", transcript)
        # A genuine failure is still an error, not a limit.
        _t, other = run_jsonl.read_stream(json.dumps({
            "type": "result", "result": "the tool crashed", "is_error": True}))
        self.assertIsNone(run_jsonl.LIMIT_RE.search(other["result_text"]))

    def test_a_run_that_rewrites_a_guarded_file_is_recorded_tampered(self):
        before = {".devforgeai/gates.toml": "a" * 64,
                  ".devforgeai/config.toml": "b" * 64,
                  ".devforgeai/state.toml": "c" * 64,
                  ".claude/settings.json": "d" * 64}
        self.assertEqual(run_jsonl.tampering(before, dict(before)), "")
        # state.toml is rewritten by `phase set` on every run; that is the run
        # working, not tampering.
        moved = dict(before, **{".devforgeai/state.toml": "e" * 64})
        self.assertEqual(run_jsonl.tampering(before, moved), "")
        # A deleted state.toml is nobody's doing but the model's.
        gone = dict(before, **{".devforgeai/state.toml": None})
        self.assertIn("state.toml", run_jsonl.tampering(before, gone))
        for guarded in (".devforgeai/gates.toml", ".devforgeai/config.toml",
                        ".claude/settings.json"):
            after = dict(before, **{guarded: "f" * 64})
            evidence = run_jsonl.tampering(before, after)
            self.assertIn(guarded, evidence)
            self.assertIn("aaaaaaaaaaaa" if "gates" in guarded else "", evidence)

    def test_guard_digests_hashes_what_exists_and_nulls_what_does_not(self):
        workspace = os.path.join(self.root, "guarded")
        write(os.path.join(workspace, ".devforgeai", "gates.toml"), "gates" + chr(10))
        digests = run_jsonl.guard_digests(workspace)
        self.assertEqual(sorted(digests), sorted(run_jsonl.GUARDED))
        self.assertEqual(len(digests[".devforgeai/gates.toml"]), 64)
        self.assertIsNone(digests[".claude/settings.json"])

    def test_tampering_overrides_a_passing_grader(self):
        out = os.path.join(self.root, "results.jsonl")
        # The fake claude writes claude-ran.txt; point a guarded path at it so
        # the run itself changes one, which is what a tampering model would do.
        cases = [{"id": "t1", "prompt": "x",
                  "setup": {"files": {".devforgeai/gates.toml": "seeded\n"}},
                  "expect": {"grader": "ok_grader", "args": {}}}]
        write(self.cases_path,
              "".join(json.dumps(case) + chr(10) for case in cases))
        real = run_jsonl.invoke

        def meddling(argv, workspace, home, timeout, prompt, options,
                     log_dir, case_id):
            result = real(argv, workspace, home, timeout, prompt, options,
                          log_dir, case_id)
            write(os.path.join(workspace, ".devforgeai", "gates.toml"),
                  "rewritten by the model" + chr(10))
            return result

        run_jsonl.invoke = meddling
        self.addCleanup(setattr, run_jsonl, "invoke", real)
        code, text = self._main(["--out", out])
        self.assertEqual(code, 2)
        line = read_lines(out)[-1]
        self.assertEqual(line["status"], "tampered")
        self.assertFalse(line["passed"])
        self.assertIn(".devforgeai/gates.toml", line["tampered"])
        self.assertIn("grader said pass", line["evidence"])
        self.assertIn("tampered 1", text)

    def test_no_grader_reads_the_guarded_project_files(self):
        """A grader that read gates.toml would make tampering profitable."""
        import glob
        root = run_jsonl.framework_root()
        found = sorted(glob.glob(os.path.join(root, "skills", "*", "evals",
                                              "graders.py")))
        if not found:
            self.skipTest("no grader modules here")
        for path in found:
            with open(path, encoding="utf-8") as handle:
                body = handle.read()
            for guarded in ("gates.toml", "state.toml"):
                self.assertNotIn(guarded, body,
                                 "%s reads %s; the tamper guard assumes no "
                                 "grader does" % (path, guarded))

    # -- results.jsonl schema ------------------------------------------------

    def test_results_line_schema(self):
        out = os.path.join(self.root, "results.jsonl")
        code, text = self._main(["--out", out, "--model", "haiku"])
        self.assertEqual(code, 2)  # c4 and c5 record error
        lines = read_lines(out)
        self.assertEqual([line["case_id"] for line in lines],
                         ["c1", "c2", "c3", "c4", "c5"])
        expected = ["schema", "run_id", "case_id", "skill", "model", "grader",
                    "status", "passed", "evidence", "started_at", "finished_at",
                    "duration_ms", "exit_code", "workspace", "workspace_kept",
                    "transcript_bytes", "hooks", "answers", "log_stdout",
                    "log_stderr", "session_id", "total_cost_usd", "num_turns",
                    "permission_denials", "is_error", "tool_calls", "tampered"]
        run_ids = set()
        for line in lines:
            self.assertEqual(list(line), expected)
            self.assertEqual(line["schema"], "devforgeai/eval-result/2")
            self.assertEqual(line["skill"], "exploring-ideas")
            self.assertEqual(line["model"], "haiku")
            self.assertIn(line["status"], ("pass", "fail", "error", "timeout"))
            self.assertEqual(line["passed"], line["status"] == "pass")
            self.assertLessEqual(len(line["evidence"]), 2000)
            self.assertTrue(line["started_at"].endswith("Z"))
            self.assertIsInstance(line["duration_ms"], int)
            run_ids.add(line["run_id"])
        self.assertEqual(len(run_ids), 1)
        by_id = {line["case_id"]: line for line in lines}
        self.assertEqual(by_id["c1"]["status"], "pass")
        self.assertEqual(by_id["c1"]["exit_code"], 0)
        self.assertEqual(by_id["c1"]["total_cost_usd"], 0.25)
        self.assertEqual(by_id["c1"]["num_turns"], 3)
        self.assertEqual(by_id["c1"]["session_id"], "sess-1")
        self.assertEqual(by_id["c1"]["tool_calls"], {"Bash": 1, "Agent": 1})
        self.assertEqual(by_id["c2"]["answers"], ["Decision"])
        self.assertEqual(by_id["c3"]["status"], "fail")
        self.assertEqual(by_id["c4"]["evidence"], "grader_missing: absent_grader")
        self.assertEqual(by_id["c5"]["status"], "error")
        self.assertIn("escapes the workspace", by_id["c5"]["evidence"])
        self.assertIn("cases 5  pass 2  fail 1  error 2  timeout 0  limit 0  tampered 0  "
                      "duration", text)
        self.assertIn("cost $", text)
        self.assertIn("FAIL  c3  fail_grader: no marker in the transcript", text)
        self.assertIn("results: %s" % out, text)
        self.assertFalse(os.path.exists(os.path.join(self.root, "outside.txt")))

    def test_raw_streams_land_outside_every_workspace(self):
        out = os.path.join(self.root, "results.jsonl")
        self._main(["--out", out, "--case", "c1", "--keep"])
        line = read_lines(out)[0]
        self.assertTrue(os.path.isfile(line["log_stdout"]))
        self.assertTrue(os.path.isfile(line["log_stderr"]))
        self.assertFalse(run_jsonl._inside(line["workspace"], line["log_stdout"]))
        leaked = [name for name in os.listdir(line["workspace"])
                  if name.startswith(".dfa-claude-")]
        self.assertEqual(leaked, [])

    def test_out_is_appended_and_case_filter_selects_one(self):
        out = os.path.join(self.root, "results.jsonl")
        write(out, json.dumps({"case_id": "earlier"}) + "\n")
        code, _ = self._main(["--out", out, "--case", "c1"])
        self.assertEqual(code, 0)
        lines = read_lines(out)
        self.assertEqual([line["case_id"] for line in lines], ["earlier", "c1"])

    def test_unknown_case_id_exits_three(self):
        code, _ = self._main(["--case", "nope"])
        self.assertEqual(code, 3)

    def test_workspace_is_removed_unless_keep(self):
        out = os.path.join(self.root, "results.jsonl")
        self._main(["--out", out, "--case", "c1"])
        first = read_lines(out)[0]
        self.assertFalse(os.path.exists(first["workspace"]))
        self.assertFalse(first["workspace_kept"])
        self._main(["--out", out, "--case", "c1", "--keep"])
        kept = read_lines(out)[-1]
        self.assertTrue(kept["workspace_kept"])
        self.assertTrue(os.path.isdir(kept["workspace"]))
        self.assertTrue(os.path.isfile(
            os.path.join(kept["workspace"], "claude-ran.txt")))

    # -- graders -------------------------------------------------------------

    def test_grader_module_loads_from_a_path(self):
        module = run_jsonl.load_graders(
            os.path.join(self.skill, "evals", "graders.py"), self.cases_path)
        self.assertEqual(module.__name__, "dfa_graders_cases")
        self.assertTrue(callable(module.ok_grader))
        self.assertEqual(run_jsonl.grade(module, "ok_grader", ".", "abc", {}),
                         ("pass", "saw 3 bytes"))
        self.assertEqual(run_jsonl.grade(module, "fail_grader", ".", "", {})[0], "fail")

    def test_missing_grader_and_raising_grader_record_error(self):
        module = run_jsonl.load_graders(
            os.path.join(self.skill, "evals", "graders.py"), self.cases_path)
        status, evidence = run_jsonl.grade(module, "absent_grader", ".", "", {})
        self.assertEqual(status, "error")
        self.assertEqual(evidence, "grader_missing: absent_grader")
        status, evidence = run_jsonl.grade(module, "boom_grader", ".", "", {})
        self.assertEqual(status, "error")
        self.assertIn("kaboom", evidence)

    def test_absent_grader_module_exits_three(self):
        code, _ = self._main(["--graders", os.path.join(self.root, "nope.py")])
        self.assertEqual(code, 3)

    # -- workspace settings: the answer hook and the framework hooks ---------

    def test_no_hooks_registers_only_the_answer_seed(self):
        options = self._options(["--case", "c2"])
        cases = self._cases()
        workspace, _home = run_jsonl.make_workspace(
            cases[1], self._files("c2"), options)
        with open(os.path.join(workspace, ".claude", "settings.json"),
                  encoding="utf-8") as handle:
            settings = json.load(handle)
        self.assertEqual(list(settings["hooks"]), ["PreToolUse"])
        entry = settings["hooks"]["PreToolUse"][0]
        self.assertEqual(entry["matcher"], "AskUserQuestion")
        handler = entry["hooks"][0]
        self.assertEqual(handler["command"], sys.executable)
        self.assertEqual(len(handler["args"]), 2)
        self.assertTrue(handler["args"][0].endswith("answer_hook.py"))
        self.assertTrue(os.path.isfile(handler["args"][0]))
        with open(handler["args"][1], encoding="utf-8") as handle:
            self.assertEqual(json.load(handle), {"Decision": "Promote"})
        self.assertTrue(run_jsonl._inside(workspace, handler["args"][0]))

    def test_hooks_on_writes_the_framework_block_with_the_binary_resolved(self):
        options = self._options(["--case", "c1"])
        options.hooks = True
        cases = self._cases()
        workspace, _home = run_jsonl.make_workspace(
            cases[0], self._files("c1"), options)
        with open(os.path.join(workspace, ".claude", "settings.json"),
                  encoding="utf-8") as handle:
            settings = json.load(handle)
        self.assertEqual(sorted(settings["hooks"]),
                         ["PostToolUse", "PreToolUse", "SubagentStop"])
        pre = settings["hooks"]["PreToolUse"]
        self.assertEqual(pre[0]["matcher"], "AskUserQuestion")
        self.assertEqual(pre[1]["hooks"][0]["command"],
                         os.path.abspath(self.devforgeai))
        self.assertNotIn("if", settings["hooks"]["PostToolUse"][0]["hooks"][0])
        self.assertNotIn("matcher", settings["hooks"]["SubagentStop"][0])
        blob = json.dumps(settings)
        for token in ("@@DEVFORGEAI@@", "@@TEST_COMMAND@@", "@@VERIFIERS@@"):
            self.assertNotIn(token, blob)

    def test_trust_gate_falls_back_to_no_hooks_and_says_why(self):
        # The stand-in binary is not executable, so `trust verify` cannot run.
        options = self._options([])
        options.hooks = True
        options.hooks_explicit = False
        ok, reason = run_jsonl.trust_gate(options)
        self.assertFalse(ok)
        self.assertTrue(reason)

    def test_hooks_off_skips_the_trust_gate(self):
        options = self._options([])
        self.assertFalse(options.hooks)
        self.assertEqual(run_jsonl.trust_gate(options), (True, ""))

    # -- the answer hook itself ---------------------------------------------

    def _hook(self, answers, payload):
        import subprocess
        script = os.path.join(os.path.dirname(os.path.abspath(
            run_jsonl.__file__)), "answer_hook.py")
        answers_path = os.path.join(self.root, "answers", "eval-answers.json")
        write(answers_path, json.dumps(answers))
        finished = subprocess.run(
            [sys.executable, script, answers_path], input=json.dumps(payload),
            capture_output=True, text=True, timeout=60)
        return json.loads(finished.stdout)["hookSpecificOutput"], answers_path

    def test_answer_hook_allows_and_records_a_configured_header(self):
        payload = {"tool_input": {"questions": [
            {"header": "Decision", "question": "Kill, park or promote?",
             "options": [{"label": "Kill"}, {"label": "Promote"}]}]}}
        out, answers_path = self._hook({"Decision": "Promote"}, payload)
        self.assertEqual(out["permissionDecision"], "allow")
        # Both keys carry the value: the templates use `header`, the guidance
        # describes `answers` as keyed by the question text.
        self.assertEqual(out["updatedInput"]["answers"],
                         {"Decision": "Promote",
                          "Kill, park or promote?": "Promote"})
        self.assertEqual(out["updatedInput"]["questions"],
                         payload["tool_input"]["questions"])
        used = os.path.join(os.path.dirname(answers_path),
                            "eval-answers-used.json")
        with open(used, encoding="utf-8") as handle:
            self.assertEqual(json.load(handle)[0]["header"], "Decision")

    def test_answer_hook_matches_on_the_question_text_too(self):
        payload = {"tool_input": {"questions": [
            {"header": "Choice", "question": "Which typeface pairing?",
             "options": [{"label": "One sans"}]}]}}
        out, _ = self._hook({"typeface pairing": "One sans"}, payload)
        self.assertEqual(out["permissionDecision"], "allow")

    def test_answer_hook_denies_an_unconfigured_header_by_name(self):
        payload = {"tool_input": {"questions": [
            {"header": "Prototype", "question": "Build one?",
             "options": [{"label": "Yes"}]}]}}
        out, _ = self._hook({"Decision": "Promote"}, payload)
        self.assertEqual(out["permissionDecision"], "deny")
        self.assertIn("Prototype", out["permissionDecisionReason"])

    def test_answer_hook_denies_a_label_the_options_do_not_offer(self):
        payload = {"tool_input": {"questions": [
            {"header": "Decision", "question": "Which?",
             "options": [{"label": "Kill"}]}]}}
        out, _ = self._hook({"Decision": "Promote"}, payload)
        self.assertEqual(out["permissionDecision"], "deny")

    def test_answer_hook_resolves_a_positional_selector(self):
        payload = {"tool_input": {"questions": [
            {"header": "Holders", "question": "who holds this problem today?",
             "multiSelect": True,
             "options": [{"label": "Practice managers"},
                         {"label": "Front desk"},
                         {"label": "Someone else"}]}]}}
        out, _ = self._hook({"Holders": ["@first"]}, payload)
        self.assertEqual(out["permissionDecision"], "allow")
        self.assertEqual(out["updatedInput"]["answers"]["Holders"],
                         ["Practice managers"])

    def test_answer_hook_star_is_the_default_for_an_unnamed_question(self):
        payload = {"tool_input": {"questions": [
            {"header": "Prototype", "question": "build one?",
             "options": [{"label": "Yes"}, {"label": "No"}]}]}}
        out, _ = self._hook({"Decision": "Kill", "*": "@last"}, payload)
        self.assertEqual(out["updatedInput"]["answers"]["Prototype"], "No")

    def test_answer_hook_denies_a_selector_past_the_end(self):
        payload = {"tool_input": {"questions": [
            {"header": "Revisit", "question": "when?",
             "options": [{"label": "a"}, {"label": "b"}]}]}}
        out, _ = self._hook({"Revisit": "@5"}, payload)
        self.assertEqual(out["permissionDecision"], "deny")

    def test_answer_hook_denies_when_the_case_configures_nothing(self):
        payload = {"tool_input": {"questions": [
            {"header": "Decision", "question": "Which?", "options": []}]}}
        out, _ = self._hook({}, payload)
        self.assertEqual(out["permissionDecision"], "deny")
        self.assertIn("configures no answers", out["permissionDecisionReason"])

    # -- CLI-written defaults and the preflight mode -------------------------

    def test_cli_defaults_fill_only_what_the_case_did_not_seed(self):
        run_jsonl._DEFAULTS.clear()
        self.addCleanup(run_jsonl._DEFAULTS.clear)
        run_jsonl._DEFAULTS["files"] = {
            "state.toml": "DEFAULT STATE\n",
            "config.toml": "DEFAULT CONFIG\n",
            "gates.toml": "DEFAULT GATES\n"}
        options = self._options(["--case", "c1"])
        case = dict(self._cases()[0])
        case["setup"] = {"files": dict(case["setup"]["files"])}
        case["setup"]["files"][".devforgeai/config.toml"] = "SEEDED CONFIG\n"
        files = run_jsonl.resolve_fixtures(case["setup"]["files"], self.skill)
        workspace, _home = run_jsonl.make_workspace(case, files, options)

        def read(name):
            with open(os.path.join(workspace, ".devforgeai", name),
                      encoding="utf-8") as handle:
                return handle.read()

        self.assertEqual(read("config.toml"), "SEEDED CONFIG\n")
        self.assertEqual(read("state.toml"), "DEFAULT STATE\n")
        self.assertEqual(read("gates.toml"), "DEFAULT GATES\n")

    def test_cli_defaults_survive_a_binary_that_cannot_run(self):
        run_jsonl._DEFAULTS.clear()
        self.addCleanup(run_jsonl._DEFAULTS.clear)
        # The stand-in binary is not executable; a case then gets exactly what
        # its own setup.files holds and the run is not stopped.
        self.assertEqual(
            run_jsonl.cli_defaults(self.devforgeai, self.root), {})

    def test_preflight_runs_the_declared_calls_and_reports_each(self):
        run_jsonl._DEFAULTS.clear()
        self.addCleanup(run_jsonl._DEFAULTS.clear)
        run_jsonl._DEFAULTS["files"] = {}
        cases = [
            {"id": "p1", "prompt": "x", "preflight": ["--version"],
             "setup": {"files": {}}, "expect": {"grader": "ok_grader"}},
            {"id": "p2", "prompt": "x", "setup": {"files": {}},
             "expect": {"grader": "ok_grader"}},
        ]
        write(self.cases_path,
              "".join(json.dumps(case) + "\n" for case in cases))
        # A stand-in that exits 0 for any argument, so the mode itself is what
        # the test exercises.
        if os.name == "nt":
            write(self.devforgeai.replace(".exe", ".cmd"), "@echo off\r\nexit 0\r\n")
            binary = self.devforgeai.replace(".exe", ".cmd")
        else:
            write(self.devforgeai, "#!/bin/sh\nexit 0\n")
            os.chmod(self.devforgeai, 0o755)
            binary = self.devforgeai
        code, text = self._main(["--preflight", "--devforgeai-bin", binary])
        self.assertEqual(code, 0)
        self.assertIn("ok    p1  1 call", text)
        self.assertIn("SKIP  p2  no preflight declared", text)
        self.assertIn("cases 2  preflight, claude not invoked", text)
        # The workspaces the mode builds are removed: the calls mutate them.
        self.assertEqual([n for n in os.listdir(self.workdir)
                          if n.startswith("dfa-eval-")], [])

    # -- gates fixtures track the CLI template -------------------------------

    def test_gates_default_fixtures_are_byte_copies_of_the_cli_template(self):
        """Every `gates-default.toml` fixture is the template, byte for byte.

These five started as output from a stale release binary and drifted:
`path = "accepted_by"` for `field`, and three `verifiers.*` metrics
without the `.payload.` segment the one-envelope contract added. A copy
that is not the template is a copy that can drift again.
"""
        root = run_jsonl.framework_root()
        template = os.path.join(root, "cli", "templates", "gates.default.toml")
        if not os.path.isfile(template):
            self.skipTest("the CLI template is not in this tree")
        with open(template, "rb") as handle:
            want = handle.read()
        checked = []
        for skill in ("designing-interfaces", "discovering-requirements",
                      "establishing-context", "exploring-ideas", "planning-work"):
            path = os.path.join(root, "skills", skill, "evals", "fixtures",
                                "gates-default.toml")
            self.assertTrue(os.path.isfile(path), path)
            with open(path, "rb") as handle:
                self.assertEqual(handle.read(), want,
                                 "%s has drifted from %s" % (path, template))
            checked.append(skill)
        self.assertEqual(len(checked), 5)

    def test_every_gates_fixture_declares_the_schema_and_minimum_version(self):
        """A gates file with no `schema` is `DFA-E107` at the first CLI call."""
        import glob
        root = run_jsonl.framework_root()
        pattern = os.path.join(root, "skills", "*", "evals", "fixtures", "gates*.toml")
        found = sorted(glob.glob(pattern))
        if not found:
            self.skipTest("no gates fixture in this tree")
        for path in found:
            with open(path, encoding="utf-8") as handle:
                head = handle.read(200)
            self.assertIn('schema = "devforgeai/gates/1"', head, path)
            self.assertIn("cli_min_version", head, path)

    def test_every_fixture_is_referenced_by_a_case_grader_or_digest(self):
        """No fixture is dead weight, and no digest names a file that has gone.

`wt-story-014-git.txt` outlived the BLD-07 rewrite, which moved the
seeded overlap from a hand-written `.git` file to a real
`setup.git.worktrees` entry. `digests.txt` is the one file that is
documentation rather than a fixture: it records the recipe and the
pinned values, so it is exempt from needing a referrer and is instead
required to name only files that exist.
"""
        import glob
        root = run_jsonl.framework_root()
        skills = sorted(glob.glob(os.path.join(root, "skills", "*", "evals")))
        if not skills:
            self.skipTest("no skill tree here")
        for evals_dir in skills:
            fixtures_dir = os.path.join(evals_dir, "fixtures")
            if not os.path.isdir(fixtures_dir):
                continue
            blob = ""
            for name in ("cases.jsonl", "graders.py", "evals.json"):
                path = os.path.join(evals_dir, name)
                if os.path.isfile(path):
                    with open(path, encoding="utf-8") as handle:
                        blob += handle.read()
            digests = os.path.join(fixtures_dir, "digests.txt")

    def test_every_public_grader_is_named_by_a_case(self):
        """No grader is defined and nameless (AUDIT-5 EVL-043).

A grader nothing calls is either dead weight or the tell that a case was
dropped — `design_called` was the only check on the Plan-to-Design
hand-off and no case named it. Helpers carry a leading underscore, which
is what separates them from graders here.
"""
        import glob
        import importlib.util
        root = run_jsonl.framework_root()
        found = sorted(glob.glob(os.path.join(root, "skills", "*", "evals",
                                              "graders.py")))
        if not found:
            self.skipTest("no grader modules here")
        for path in found:
            evals_dir = os.path.dirname(path)
            skill = os.path.basename(os.path.dirname(evals_dir))
            spec = importlib.util.spec_from_file_location(
                "check_" + skill.replace("-", "_"), path)
            module = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(module)
            named = set()
            with open(os.path.join(evals_dir, "cases.jsonl"),
                      encoding="utf-8") as handle:
                for line in handle:
                    if line.strip():
                        named.add(json.loads(line)["expect"]["grader"])
            defined = {n for n in dir(module)
                       if not n.startswith("_")
                       and callable(getattr(module, n))
                       and getattr(getattr(module, n), "__module__", "")
                       == module.__name__}
            self.assertEqual(sorted(defined - named), [],
                             "%s defines graders no case names" % path)
            self.assertEqual(sorted(named - defined), [],
                             "%s names graders it does not define" % path)

    def test_every_skill_meets_the_send_back_floor_or_documents_why(self):
        """Conventions section 9 wants two SEND BACK cases per skill.

Explore and Reflect can have none — each `## Send-back` says so in
terms: Explore is phase 0 with no upstream document to cite, and a
Reflect `REC-nnn` is prose rather than a gate result. Both carry a
documented stop-path case instead, which is what this asserts.
"""
        import glob
        root = run_jsonl.framework_root()
        paths = sorted(glob.glob(os.path.join(root, "skills", "*", "evals",
                                              "cases.jsonl")))
        if not paths:
            self.skipTest("no case files here")
        exempt = {"exploring-ideas": "blocked_on_cli",
                  "improving-framework": "blocked_on_preamble"}
        for path in paths:
            skill = os.path.basename(os.path.dirname(os.path.dirname(path)))
            graders, sendbacks = [], 0
            with open(path, encoding="utf-8") as handle:
                for line in handle:
                    if not line.strip():
                        continue
                    case = json.loads(line)
                    graders.append(case["expect"]["grader"])
                    blob = json.dumps(case["expect"])
                    if "SEND BACK" in blob or "send_back" in blob                             or "sendback" in case["expect"]["grader"]:
                        sendbacks += 1
            if skill in exempt:
                self.assertIn(exempt[skill], graders,
                              "%s is exempt from the floor and carries no "
                              "documented stop-path case" % skill)
                continue
            self.assertGreaterEqual(sendbacks, 2,
                                    "%s has %d SEND BACK cases" % (skill, sendbacks))

    # -- canonical config, invoked skills ------------------------------------

    def test_stack_detect_runs_before_the_guard_baseline(self):
        import subprocess
        calls = []
        real = subprocess.run

        def spy(argv, *a, **kw):
            if isinstance(argv, list) and argv[1:3] == ["stack", "detect"]:
                calls.append(kw.get("cwd"))
            return real(argv, *a, **kw)

        run_jsonl.subprocess.run = spy
        self.addCleanup(setattr, run_jsonl.subprocess, "run", real)
        options = self._options(["--case", "c1"])
        case = dict(self._cases()[0])
        case["setup"] = {"files": dict(case["setup"]["files"])}
        case["setup"]["files"][".devforgeai/config.toml"] = "degraded = false\n"
        files = run_jsonl.resolve_fixtures(case["setup"]["files"], self.skill)
        workspace, _home = run_jsonl.make_workspace(case, files, options)
        self.assertIn(workspace, calls)

    def test_config_degraded_reads_the_flag(self):
        path = os.path.join(self.root, "c.toml")
        write(path, 'schema = "x"\ndegraded = true\n')
        self.assertIs(run_jsonl.config_degraded(path), True)
        write(path, 'schema = "x"\ndegraded = false\n')
        self.assertIs(run_jsonl.config_degraded(path), False)
        self.assertIsNone(run_jsonl.config_degraded(
            os.path.join(self.root, "absent.toml")))

    def test_preflight_fails_when_degraded_does_not_match(self):
        write(self.cases_path, json.dumps(
            {"id": "d1", "prompt": "p", "degraded": False,
             "setup": {"files": {".devforgeai/config.toml":
                                 'schema = "x"\ndegraded = true\n'}},
             "expect": {"grader": "ok_grader"}}) + "\n")
        code, text = self._main(["--preflight"])
        self.assertEqual(code, 2)
        self.assertIn("degraded is True", text)
        self.assertIn("expects False", text)

    def test_timeout_default_is_fifteen_hundred(self):
        self.assertEqual(self._options().timeout, 1500)

    def test_a_skill_invoked_through_the_skill_tool_is_installed_too(self):
        # A second skill in the tree whose frontmatter name is `design`, which
        # the skill under test says it invokes. The hooks-on ex-01 run stalled
        # because the workspace held only the skill under test and Claude Code
        # resolved `design` against the machine instead.
        other = os.path.join(self.root, "skills", "designing-interfaces")
        write(os.path.join(other, "SKILL.md"), "---\nname: design\n---\n")
        write(os.path.join(other, "evals", "cases.jsonl"), "")
        write(os.path.join(self.skill, "SKILL.md"),
              "---\nname: explore\n---\n\n"
              "Through the Skill tool, invoke the `design` skill with the request.\n")
        self.assertEqual(
            run_jsonl.invoked_skills(self.skill, self.root), ["design"])
        options = self._options(["--case", "c1"])
        workspace, _home = run_jsonl.make_workspace(
            self._cases()[0], self._files("c1"), options)
        installed = os.path.join(workspace, ".claude", "skills")
        self.assertEqual(sorted(os.listdir(installed)), ["design", "explore"])
        self.assertTrue(os.path.isfile(
            os.path.join(installed, "design", "SKILL.md")))
        self.assertFalse(os.path.isdir(
            os.path.join(installed, "design", "evals")))

    def test_setup_skills_installs_an_explicit_extra(self):
        other = os.path.join(self.root, "skills", "validating-quality")
        write(os.path.join(other, "SKILL.md"), "---\nname: verify\n---\n")
        options = self._options(["--case", "c1"])
        case = dict(self._cases()[0])
        case["setup"] = dict(case["setup"], skills=["verify"])
        workspace, _home = run_jsonl.make_workspace(
            case, self._files("c1"), options)
        self.assertTrue(os.path.isfile(os.path.join(
            workspace, ".claude", "skills", "verify", "SKILL.md")))

    def test_an_invoked_skill_that_does_not_exist_stops_the_case(self):
        write(os.path.join(self.skill, "SKILL.md"),
              "---\nname: explore\n---\n\ninvoke the `nowhere` skill\n")
        options = self._options(["--case", "c1"])
        with self.assertRaises(run_jsonl.CaseError):
            run_jsonl.make_workspace(self._cases()[0], self._files("c1"), options)

    # -- config merge and incremental logging --------------------------------

    def test_a_seeded_config_inherits_the_verifier_registry(self):
        run_jsonl._DEFAULTS.clear()
        self.addCleanup(run_jsonl._DEFAULTS.clear)
        default = (
            'schema = "devforgeai/config/1"\n'
            'generated_at = "2026-09-10T09:00:00Z"\n'
            "degraded = true\n"
            "\n"
            "[frontend]\n"
            'globs = ["**/*.css"]\n'
            "\n"
            "[build]\n"
            "complexity_max = 10\n"
            "\n"
            "[coverage]\n"
            "overall_min = 80.0\n"
            + "".join('[[verifier]]\nname = "v%02d"\nphase = "build"\n\n' % n
                      for n in range(20)))
        run_jsonl._DEFAULTS["files"] = {"config.toml": default}

        seeded = ('schema = "devforgeai/config/1"\n'
                  'generated_at = "2026-09-10T09:00:00Z"\n'
                  "degraded = false\n"
                  "\n"
                  "[[stack]]\n"
                  'id = "primary"\n'
                  'source = "manual"\n'
                  'test_command = "sh ci/test"\n'
                  "\n"
                  "[build]\n"
                  "complexity_max = 4\n")
        options = self._options(["--case", "c1"])
        case = dict(self._cases()[0])
        case["setup"] = {"files": dict(case["setup"]["files"])}
        case["setup"]["files"][".devforgeai/config.toml"] = seeded
        files = run_jsonl.resolve_fixtures(case["setup"]["files"], self.skill)
        workspace, _home = run_jsonl.make_workspace(case, files, options)
        with open(os.path.join(workspace, ".devforgeai", "config.toml"),
                  encoding="utf-8") as handle:
            merged = handle.read()
        # Absent tables come from the default; the registry is why this exists.
        self.assertEqual(merged.count("[[verifier]]"), 20)
        self.assertIn("[frontend]", merged)
        self.assertIn("[coverage]", merged)
        # A table the case states replaces the default's outright.
        self.assertIn("complexity_max = 4", merged)
        self.assertNotIn("complexity_max = 10", merged)
        self.assertIn('test_command = "sh ci/test"', merged)
        self.assertIn("degraded = false", merged)

    def test_merge_config_returns_the_seed_when_there_is_no_default(self):
        seeded = 'schema = "x"\n\n[build]\ncomplexity_max = 4\n'
        self.assertEqual(run_jsonl.merge_config(seeded, ""), seeded)

    def test_split_config_tables_keeps_a_subtable_with_its_parent(self):
        text = ('schema = "x"\n\n[[stack]]\nid = "primary"\n\n'
                '[stack.env]\nCI = "1"\n\n[build]\ncomplexity_max = 4\n')
        preamble, blocks = run_jsonl.split_config_tables(text)
        self.assertIn("schema", preamble)
        self.assertEqual(sorted(blocks), ["build", "stack"])
        self.assertIn("[stack.env]", blocks["stack"][0])

    def test_stdout_is_written_as_it_arrives_so_a_timeout_leaves_evidence(self):
        # A child that prints, then hangs past the timeout. Without incremental
        # writing the kill would leave no log at all, which is what BLD-01 hit.
        script = os.path.join(self.root, "bin", "slow.py")
        write(script,
              "import sys, time\n"
              "sys.stdin.read()\n"
              'sys.stdout.write(\'{"type": "system", "subtype": "init"}\' + "\\n")\n'
              "sys.stdout.flush()\n"
              "time.sleep(30)\n")
        logs = os.path.join(self.root, "logs")
        out, err, code = run_jsonl.run_streaming(
            [sys.executable, script], self.root, dict(os.environ), "p", 3,
            os.path.join(logs, "slow.stdout.txt"))
        self.assertIsNone(code)          # the timeout struck
        self.assertIn("system", out)     # the partial stdout came back
        with open(os.path.join(logs, "slow.stdout.txt"), encoding="utf-8") as h:
            self.assertIn("system", h.read())   # and it is on disk

    def test_streaming_returns_the_whole_stream_and_the_code(self):
        script = os.path.join(self.root, "bin", "quick.py")
        write(script,
              "import sys\n"
              "prompt = sys.stdin.read()\n"
              'sys.stdout.write("saw " + prompt + "\\n")\n'
              'sys.stderr.write("noise\\n")\n'
              "raise SystemExit(3)\n")
        logs = os.path.join(self.root, "logs2")
        out, err, code = run_jsonl.run_streaming(
            [sys.executable, script], self.root, dict(os.environ), "hello", 60,
            os.path.join(logs, "quick.stdout.txt"))
        self.assertEqual(code, 3)
        self.assertIn("saw hello", out)
        self.assertIn("noise", err)
        with open(os.path.join(logs, "quick.stdout.txt"), encoding="utf-8") as h:
            self.assertIn("saw hello", h.read())

    def test_verifiers_merge_by_name_and_keep_the_whole_catalogue(self):
        default = ['[[verifier]]\nname = "kill-case-builder"\nrequired = true',
                   '[[verifier]]\nname = "ac-test-writer"\nrequired = true',
                   '[[verifier]]\nname = "deferral-auditor"\nrequired = true']
        seeded = ['[[verifier]]\nname = "ac-test-writer"\nrequired = false',
                  '[[verifier]]\nname = "local-only"\nrequired = true']
        merged = run_jsonl.merge_verifiers(seeded, default)
        names = [run_jsonl._block_name(one) for one in merged]
        # Every default name survives, in order, and the fixture's extra follows.
        self.assertEqual(names, ["kill-case-builder", "ac-test-writer",
                                 "deferral-auditor", "local-only"])
        # The seeded row replaced the default row of the same name.
        self.assertIn("required = false", merged[1])
        self.assertNotIn("required = false", merged[0])

    def test_an_absent_verifier_table_keeps_the_default_registry(self):
        default = ['[[verifier]]\nname = "a"', '[[verifier]]\nname = "b"']
        self.assertEqual(run_jsonl.merge_verifiers([], default), default)

    def test_a_fixture_registry_does_not_shrink_the_catalogue(self):
        run_jsonl._DEFAULTS.clear()
        self.addCleanup(run_jsonl._DEFAULTS.clear)
        run_jsonl._DEFAULTS["files"] = {"config.toml": (
            'schema = "devforgeai/config/1"\ndegraded = true\n\n'
            + "".join('[[verifier]]\nname = "v%02d"\nrequired = true\n\n' % n
                      for n in range(20)))}
        seeded = ('schema = "devforgeai/config/1"\ndegraded = false\n\n'
                  '[[stack]]\nid = "primary"\nsource = "manual"\n\n'
                  '[[verifier]]\nname = "v03"\nrequired = false\n')
        options = self._options(["--case", "c1"])
        case = dict(self._cases()[0])
        case["setup"] = {"files": dict(case["setup"]["files"])}
        case["setup"]["files"][".devforgeai/config.toml"] = seeded
        files = run_jsonl.resolve_fixtures(case["setup"]["files"], self.skill)
        workspace, _home = run_jsonl.make_workspace(case, files, options)
        with open(os.path.join(workspace, ".devforgeai", "config.toml"),
                  encoding="utf-8") as handle:
            merged = handle.read()
        # A fixture naming one row keeps the other nineteen: `gate check` fails
        # DFA-E316 on a phase whose verifier the file does not register.
        self.assertEqual(merged.count("[[verifier]]"), 20)
        self.assertIn('required = false', merged)
        self.assertIn('name = "v19"', merged)

    # -- installed skill name ------------------------------------------------

    def test_installed_skill_name_is_the_frontmatter_name(self):
        self.assertEqual(run_jsonl.installed_skill_name(self.skill), "explore")

    def test_installed_skill_name_falls_back_to_the_directory(self):
        write(os.path.join(self.skill, "SKILL.md"), "# no frontmatter" + chr(10))
        self.assertEqual(run_jsonl.installed_skill_name(self.skill),
                         "exploring-ideas")

    # -- setup.git -----------------------------------------------------------

    def test_setup_git_makes_a_work_tree_with_a_commit_and_a_worktree(self):
        import subprocess
        if shutil.which("git") is None:
            self.skipTest("git is not on PATH")
        options = self._options(["--case", "c1"])
        case = dict(self._cases()[0])
        case["setup"] = dict(case["setup"])
        case["setup"]["git"] = {"worktrees": [{"path": "wt/STORY-014",
                                               "branch": "story/STORY-014"}]}
        workspace, _home = run_jsonl.make_workspace(case, self._files("c1"), options)
        listed = subprocess.run(["git", "worktree", "list", "--porcelain"],
                                cwd=workspace, capture_output=True, text=True)
        self.assertEqual(listed.returncode, 0)
        self.assertIn("story/STORY-014", listed.stdout)
        self.assertTrue(os.path.isdir(os.path.join(workspace, "wt", "STORY-014")))

    def test_no_setup_git_leaves_the_workspace_without_a_repository(self):
        options = self._options(["--case", "c1"])
        workspace, _home = run_jsonl.make_workspace(
            self._cases()[0], self._files("c1"), options)
        self.assertFalse(os.path.exists(os.path.join(workspace, ".git")))

    # -- dry run -------------------------------------------------------------

    def test_dry_run_materialises_a_workspace_and_calls_nothing(self):
        out = os.path.join(self.root, "results.jsonl")
        code, text = self._main(["--out", out, "--dry-run", "--case", "c2"])
        self.assertEqual(code, 0)
        self.assertFalse(os.path.exists(out))
        self.assertIn("case c2  grader ok_grader  skill exploring-ideas  files 3", text)
        self.assertIn("would run", text)
        self.assertIn("--output-format stream-json", text)
        self.assertIn("prompt (21 bytes, 2 lines) on stdin", text)
        self.assertIn("    with a newline", text)
        self.assertIn("answers Decision", text)
        workspaces = [name for name in os.listdir(self.workdir)
                      if name.startswith("dfa-eval-")]
        self.assertEqual(len(workspaces), 1)
        workspace = os.path.join(self.workdir, workspaces[0])

        def read(*parts):
            with open(os.path.join(workspace, *parts), encoding="utf-8") as handle:
                return handle.read()

        self.assertEqual(read("a.txt"), "two")
        self.assertEqual(read("b.txt"), FIXTURE_TEXT)
        self.assertEqual(read("c.txt"), "three")
        self.assertTrue(os.path.isdir(os.path.join(workspace, ".claude", "dfa-home")))
        # The copy lands under the frontmatter `name:`, which is the slash
        # command Claude Code derives from the directory.
        self.assertTrue(os.path.isfile(os.path.join(
            workspace, ".claude", "skills", "explore", "SKILL.md")))
        self.assertFalse(os.path.isdir(os.path.join(
            workspace, ".claude", "skills", "exploring-ideas")))
        self.assertFalse(os.path.isdir(os.path.join(
            workspace, ".claude", "skills", "explore", "evals")))
        self.assertEqual(read(".claude", "agents", "flow-drafter.md"), "flow drafter\n")
        self.assertFalse(os.path.isfile(os.path.join(
            workspace, ".claude", "agents", "unused-agent.md")))
        self.assertFalse(os.path.isdir(os.path.join(workspace, ".claude", "commands")))
        self.assertFalse(os.path.isfile(os.path.join(workspace, "claude-ran.txt")))

    def test_dry_run_reports_an_escaping_path_without_writing_it(self):
        code, text = self._main(["--dry-run", "--case", "c5"])
        self.assertEqual(code, 2)
        self.assertIn("ERROR c5", text)
        self.assertFalse(os.path.exists(os.path.join(self.root, "outside.txt")))

    # -- owned agents --------------------------------------------------------

    def test_owned_agents_reads_contracts_table(self):
        rows = ['# Agents', '', '## Invocation order', '', '## Contracts', '',
                '| Agent | Model |', '|---|---|',
                '| `flow-drafter` | opus |', '| `not-a-file` | opus |', '',
                '## Registered verifiers', '']
        with open(os.path.join(self.skill, 'agents.md'), 'w', encoding='utf-8') as h:
            h.write(chr(10).join(rows))
        self.assertEqual(run_jsonl.owned_agents(self.skill, self.root), ['flow-drafter'])

    def test_owned_agents_reads_agents_md_headings(self):
        self.assertEqual(run_jsonl.owned_agents(self.skill, self.root),
                         ["flow-drafter"])


if __name__ == "__main__":
    unittest.main()


class TamperMessageOnNonGradedPaths(unittest.TestCase):
    """A guarded-file change after a timeout, limit or error must not crash.

    The grader's `status` and `evidence` locals do not exist on those paths, and
    a real hooks-on run died with UnboundLocalError reaching for them. The fix
    reads them back from the record; this exercises that rather than inspecting
    the source, so a later rewrite reintroducing the bug fails here too.
    """

    def test_a_timed_out_run_that_changed_a_guarded_file_reports_both(self):
        root = tempfile.mkdtemp(prefix="dfa-tamper-")
        self.addCleanup(shutil.rmtree, root, True)
        skill = os.path.join(root, "skills", "exploring-ideas")
        write(os.path.join(skill, "SKILL.md"), "---\nname: explore\n---\n")
        write(os.path.join(skill, "evals", "graders.py"), GRADERS)
        write(os.path.join(skill, "evals", "cases.jsonl"), json.dumps(
            {"id": "t1", "prompt": "p",
             "setup": {"files": {".devforgeai/gates.toml": "seeded\n"}},
             "expect": {"grader": "ok_grader", "args": {}}}) + "\n")
        write(os.path.join(root, "hooks", "settings.hooks.json"), "{}")
        binary = os.path.join(root, "bin", "devforgeai")
        write(binary, "stand-in\n")
        workdir = os.path.join(root, "work")
        os.makedirs(workdir)

        real = run_jsonl.invoke

        def times_out(argv, workspace, home, timeout, prompt, options,
                      log_dir, case_id):
            # What a model would do, on the one path where no grader runs.
            write(os.path.join(workspace, ".devforgeai", "gates.toml"),
                  "rewritten\n")
            return "timeout", "", None, {}

        run_jsonl.invoke = times_out
        self.addCleanup(setattr, run_jsonl, "invoke", real)
        out = os.path.join(root, "results.jsonl")
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code = run_jsonl.main(["--skill", skill, "--workdir", workdir,
                                   "--framework-root", root, "--out", out,
                                   "--devforgeai-bin", binary, "--no-hooks"])
        self.assertEqual(code, 2)
        line = read_lines(out)[-1]
        self.assertEqual(line["status"], "tampered")
        self.assertIn(".devforgeai/gates.toml", line["tampered"])
        # The prior status and its evidence survive into the message.
        self.assertIn("grader said timeout", line["evidence"])
        self.assertIn("no result within", line["evidence"])


class ConfigFingerprintTestCase(unittest.TestCase):
    """`config.toml` is compared by content, not bytes.

    The SessionStart hook runs `stack detect`, which rewrites the file on every
    hooks-on run and stamps a fresh `generated_at`. That is the harness working.
    A changed command or threshold in the same file is not.
    """

    BASE = ('schema = "devforgeai/config/1"\n'
            'generated_at = "2026-09-10T09:00:00Z"\n'
            'cli_version = "1.0.0"\n'
            "degraded = false\n"
            "\n"
            "[[stack]]\n"
            'id = "primary"\n'
            'source = "manual"\n'
            'test_command = "sh ci/test"\n'
            "timeout_secs = 900\n")

    def test_a_new_timestamp_alone_is_not_tampering(self):
        restamped = self.BASE.replace("2026-09-10T09:00:00Z",
                                      "2026-09-11T18:00:00Z")
        self.assertNotEqual(self.BASE, restamped)
        self.assertEqual(run_jsonl.config_fingerprint(self.BASE),
                         run_jsonl.config_fingerprint(restamped))

    def test_a_changed_test_command_is_tampering(self):
        edited = self.BASE.replace("sh ci/test", "echo always-green")
        self.assertNotEqual(run_jsonl.config_fingerprint(self.BASE),
                            run_jsonl.config_fingerprint(edited))

    def test_a_changed_threshold_is_tampering(self):
        edited = self.BASE.replace("timeout_secs = 900", "timeout_secs = 1")
        self.assertNotEqual(run_jsonl.config_fingerprint(self.BASE),
                            run_jsonl.config_fingerprint(edited))

    def test_the_guard_uses_the_fingerprint_for_config_alone(self):
        parent = tempfile.mkdtemp(prefix="dfa-fp-")
        self.addCleanup(shutil.rmtree, parent, True)
        workspace = os.path.join(parent, "ws")
        write(os.path.join(workspace, ".devforgeai", "config.toml"), self.BASE)
        write(os.path.join(workspace, ".devforgeai", "gates.toml"), "gates\n")
        before = run_jsonl.guard_digests(workspace)
        write(os.path.join(workspace, ".devforgeai", "config.toml"),
              self.BASE.replace("2026-09-10T09:00:00Z", "2026-09-11T18:00:00Z"))
        self.assertEqual(
            run_jsonl.tampering(before, run_jsonl.guard_digests(workspace)), "")
        write(os.path.join(workspace, ".devforgeai", "config.toml"),
              self.BASE.replace("sh ci/test", "echo always-green"))
        self.assertIn(
            ".devforgeai/config.toml",
            run_jsonl.tampering(before, run_jsonl.guard_digests(workspace)))


if __name__ == "__main__":
    unittest.main()
