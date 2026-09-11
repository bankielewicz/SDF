---
schema: devforgeai-audit/1
doc: closure
verifier: D
scope: AUDIT-5-evals
produced_by: closure-verifier-d
---

# Closure D — AUDIT-5 evals

Scope: every blocker and high in `specs/audit/AUDIT-5-evals.md`, plus every medium. Lows are recorded only where a medium or high depends on one. Evidence is a file path with a line number, a command run on this machine with its output, or a byte comparison. No fixer report was accepted as evidence. No real `claude -p` case was launched by this verifier; the acceptance evidence is read from the scratchpad `accept/` directory produced earlier in the wave.

## Blockers

| ID | severity | status | evidence |
|---|---|---|---|
| EVL-010 | blocker | closed | `evals/runner/run_jsonl.py:619-641` `claude_argv()` emits `--output-format stream-json --verbose --include-partial-messages`; `read_stream()` at `:664` and `_render_tool_use()` at `:643` render every `tool_use` block into the transcript, with the `Agent(...)` `)`→`]` rule the `criterion_set` regex needs. Measured end to end: `accept/improving-framework.results.jsonl` records `"tool_calls": {"Bash": 10, "Read": 22, "Agent": 3, "Glob": 3, "Write": 1, "Edit": 3}` from a real run. `expect_calls` is therefore satisfiable and `forbid_calls` no longer vacuous. Dry-run argv confirmed on all nine skills. |
| EVL-012 | blocker | closed | `--devforgeai-bin` added at `run_jsonl.py:1017-1018`, defaulted from `default_binary()` at `:1060`, and a missing file is a usage error exit 3 at `:1063-1066` (observed in the unittest output: `usage error: devforgeai binary not found: …\nope.exe; build cli/ or pass --devforgeai-bin`). `child_environment()` at `:762-780` prepends the binary's directory to the child `PATH`. |
| EVL-013 | blocker | closed | Measured directly, not inferred. A script materialised all 67 workspaces through `run_jsonl.make_workspace` and executed every `` !`…` `` line of each `SKILL.md` in its own workspace with the release binary: **`TOTAL cases 67 aborting 0`** (was 33 of 67). Per skill: designing-interfaces 8/no preamble, discovering-requirements 6/1 line/0 abort, establishing-context 8/2/0, exploring-ideas 8/no preamble, implementing-stories 8/4/0, improving-framework 7/1/0, planning-work 7/3/0, releasing-software 9/1/0, validating-quality 6/4/0. |
| EVL-040 | blocker | closed | `skills/implementing-stories/evals/cases.jsonl` BLD-07 no longer carries `../wt/STORY-014/.git`; it seeds `.devforgeai/state.toml: FIXTURE:state-build-story-014-worktree.toml` and `setup.git.worktrees: [{path: "wt/STORY-014", branch: "story/STORY-014"}]`. `grep -n '\.\./' skills/*/evals/cases.jsonl` returns nothing across all nine files. `python evals/runner/run_jsonl.py --skill skills/implementing-stories --dry-run` exits 0 with all 8 cases materialised (was exit 2 with `ERROR BLD-07 path escapes the workspace`). |
| EVL-041 | blocker | closed | `_setup_path()` is gone from `skills/planning-work/evals/graders.py`; `send_back` reads an `upstream_unchanged` digest map at `:586` and `remedy_ac` reads `_fixture(args["baseline_story"])` / `_fixture(args["baseline_sprint"])` at `:608`/`:637`. Verified the digests are not merely present but correct: a script recomputed SHA-256 over the fixture bytes each case writes and compared against the recorded value — PLAN-05 `.devforgeai/requirements.yaml` MATCH, PLAN-06 `.devforgeai/context/architecture-constraints.md` MATCH; PLAN-07's `story-104.md` and `sprint-001.yaml` both exist under `skills/planning-work/evals/fixtures/`. The three cases can now pass. |
| EVL-050 | blocker | closed | The audit's decision — rewrite the case, do not give the runner a sandbox parent — was taken. `_inside()` is unchanged, and the confinement is real: `skills/implementing-stories/evals/fixtures/config-build.toml:21` sets `worktree_root = "wt"`, so `worktree ensure` writes inside the workspace; BLD-02's `args.worktree_root` is `"wt"` and `"wt"` is in its `args.baseline`, which also removes the shared-parent walk EVL-006 described. `specs/06-build.md:886` carries the rewritten BLD-07 line verbatim. |

## Highs

| ID | severity | status | evidence |
|---|---|---|---|
| EVL-001 | high | closed | `isolate_claude_config()` at `run_jsonl.py:733-760` builds `<home>/claude-config` and `child_environment()` at `:767-768` sets `CLAUDE_CONFIG_DIR` to it under the default `--claude-config isolate`. One file crosses, `.credentials.json`, which the audit's fix spec permits; `.claude.json` is written fresh rather than copied. `HOME`/`USERPROFILE` are deliberately left alone and the reason is documented at `:740-745` and in `specs/01-cli.md` §Evals (the binary resolves `~/.devforgeai/trust.toml` from them). Covered by `test_claude_config_is_redirected_into_the_workspace_home` and `test_claude_config_inherit_sets_nothing`. |
| EVL-002 | high | closed | `invoke()` at `run_jsonl.py:805-813` writes `<log_dir>/<case id>.stdout.txt` / `.stderr.txt`; `log_dir` defaults to `<workdir>/logs`, a sibling of the workspaces (`parse_args` `:1055-1058`). No `.dfa-claude-*` path remains anywhere. Confirmed live: `accept/improving-framework.results.jsonl` carries `log_stdout`/`log_stderr` pointing at `…/improving-framework/logs/`, outside the workspace. Covered by `test_raw_streams_land_outside_every_workspace`. BLD-02's `writes_inside_declared_set` walk is therefore clean. |
| EVL-003 | high | closed | The prescribed fix was a spec statement, and more landed. `specs/01-cli.md` §Evals ("Hooks during an eval") and `specs/00-conventions.md` §9 both now state that the workspace carries the framework hook block from `hooks/settings.hooks.json` by default, that this depends on a human `devforgeai trust pin`, and that the runner falls back to `--no-hooks` and prints the pin command when `trust verify` fails. Code: `workspace_settings()` at `run_jsonl.py:427-454` merges `framework_hooks()` unless `--no-hooks`; `trust_gate()` at `:1083` and `main()` at `:1166-1176` implement the fallback and refuse (exit 3) when `--hooks` was explicit. **Residual, see `## Residual risks`:** on this machine no pin exists, so every acceptance run recorded `"hooks": false` and measured no enforcement. |
| EVL-005 | high | closed | Six of the seven named handoff graders gained an artifact half that the same run must satisfy, and the seventh always had one. Verified by reading each function: `discovering-requirements.sendback_to_explore` (absent-document + `_phase_report_not_pass`), `establishing-context.send_back` (`upstream_sha256` digests + `_phase_report_not_pass`), `planning-work.send_back` (`_unchanged` digest map + absent paths + `_phase_report_not_pass`), `designing-interfaces.sendback_block` (absent paths + `_phase_report_not_pass`), `releasing-software.sendback_block` (story frontmatter `status` + `_phase_report_not_pass`), `validating-quality.sendback_block` (`_read_report` + `story_sha256`/`source_sha256` + `_phase_report_not_pass`), `improving-framework.handoff_returns` (`_reflect_document(workspace, args)`). `_phase_report_not_pass` is defined at `skills/validating-quality/evals/graders.py:19` and mirrored per skill. `asked_decision` was rewritten with two paths: the hook path asserts `.claude/eval-answers-used.json` records a call with the expected `header` **and** that the transcript holds a matching `AskUserQuestion(` `tool_use` event offering the labels; the prose fallback additionally requires `decision.yaml` to exist, be non-empty, and record a `decision:` inside the label set. `figma_fallback` and `version_refused` left as the audit directed. |
| EVL-011 | high | closed | `ALLOWED_TOOLS` at `run_jsonl.py:136-142` carries `Bash(devforgeai *)`, `PowerShell(devforgeai *)`, `Bash(devforgeai:*)`, `PowerShell(devforgeai:*)` plus `Read,Write,Edit,Glob,Grep,Agent,Skill,AskUserQuestion,WebSearch,WebFetch,TodoWrite`, and it is passed on every argv (observed in all nine dry runs). `permission_denials` is now recorded per case, so a denial is visible rather than silent (`accept/validating-quality.results.jsonl` names one: a `Bash` call that tried to `cd /c/Projects/DevForgeAI/cli`). The deviation from the audit's prescription — `bypassPermissions` instead of `acceptEdits` — is deliberate and documented; see `## Deviations`. |
| EVL-014 | high | closed | `parse_args` `:1015` `--timeout` default 900. `specs/01-cli.md` §Evals argument table records the default with the measured 828 s / $2.85 figure beside it. Per-case override honoured (`run_case` `:889`, `test_a_case_timeout_overrides_the_run_default`), and `test_every_long_phase_case_declares_a_timeout` holds the Constitute cases at 1500. No acceptance run hit the timeout; the longest was 650.4 s (`ex-01-fresh-brief`). |
| EVL-030 | high | closed | The `answers` key exists in the case schema, is validated (`_check_answers` at `:246`, exit 3 on a bad type, `test_answers_of_the_wrong_type_is_a_usage_error`), and is seeded through a real `PreToolUse` handler: `workspace_settings()` writes `eval-answers.json`, copies `evals/runner/answer_hook.py` into `<workspace>/.claude/`, and inserts the handler at index 0 of `PreToolUse` in exec form (`command` + `args`). 23 of 67 cases now carry an `answers` map (dry-run headers observed: `Decision, Holders, Prototype`; `Color, Density, Logo, Tone, Type`; `Accept epics, Actors, Boundaries, Outcomes`; `*`), against the nine the audit asked for. The pseudo-answer prompt lines are gone: **0 of 67 prompts contain a newline**. |
| EVL-051 | high | closed | The prompt left argv: `claude_argv()` builds no positional prompt, and `invoke()` passes `input=prompt` at `:795`. `dry_run_case()` prints `prompt (<n> bytes, <m> lines) on stdin` followed by the indented prompt (observed in all nine dry runs). Covered by `test_prompt_reaches_the_child_on_stdin_with_its_newlines` and `test_argv_carries_stream_json_and_the_shell_grant_and_no_prompt`. `specs/01-cli.md` §Evals step 7 states the `.cmd`-shim reason. |

## Mediums

| ID | severity | status | evidence |
|---|---|---|---|
| EVL-004 | medium | open | No fix was specified and none was made, and the permission-mode change widened the surface: `PERMISSION_MODE = "bypassPermissions"` (`run_jsonl.py:154`) lets a model rewrite `.devforgeai/gates.toml`, `.devforgeai/state.toml` and anything under `DEVFORGEAI_HOME` with no prompt and, on an unpinned machine, no hook. Reachable path named. It buys no pass today — verified: `grep -ln "state.toml\|gates.toml" skills/*/evals/graders.py` matches **no** grader module — so this stays latent until a grader asserts on either file. |
| EVL-015 | medium | closed | `run_case()` at `:872-879` seeds and `:893-895` populates `session_id`, `total_cost_usd`, `num_turns`, `permission_denials`, `is_error`, `tool_calls`. Confirmed in a real record: `accept/improving-framework.results.jsonl` carries `"session_id": "3d3fc196-…", "total_cost_usd": 1.39115255, "num_turns": 34, "permission_denials": [], "is_error": false`. |
| EVL-016 | medium | closed | `invoke()` at `:817-818`: `if finished.returncode != 0 or meta.get("is_error"): return "error", …`, and `run_case()` records status `error` with `see <log_stderr>` without calling the grader. A new `limit` status distinguishes a usage-limit stop from a real error (`LIMIT_RE` at `:176`, `test_a_usage_limit_is_recorded_apart_from_an_error`). |
| EVL-031 | medium | closed | Decision 42 amended in both places rather than silently violated. `workspace_settings()` docstring at `:427-433` states the amendment; `specs/00-conventions.md` §9 and `specs/01-cli.md` §Evals step 6 name the answer handler as the one handler an eval registers of its own, beside the framework block. `test_no_hooks_registers_only_the_answer_seed` holds the `--no-hooks` shape. |
| EVL-033 | medium | closed | `version_supports_prompts_none()` at `:722-731` parses `claude --version` and gates on `>= (2, 1, 259)`. Measured here: `claude --version` → `2.1.268 (Claude Code)`; `run_jsonl.version_supports_prompts_none('claude')` → `True`; the resulting unseeded argv ends `--add-dir WS --permission-prompts none`. A seeded case takes `--mcp-config … --permission-prompt-tool mcp__dfa-permissions__approve` instead (`test_a_seeded_case_names_the_permission_host_instead`). |
| EVL-042 | medium | open | Neither remedy landed. `specs/00-conventions.md` §9 item 2 still reads "At least 2 exercise the SEND BACK path" with no entry/terminal-phase exemption, and no phase spec's `## Evals` states one. Against that floor four skills fail, not the two the audit named: exploring-ideas **0**, improving-framework **0**, implementing-stories **0**, designing-interfaces **1** (`dz-06-sendback-discover`). The five that meet it: discovering-requirements 2, establishing-context 2, planning-work 2, releasing-software 2, validating-quality 2. |
| EVL-045 | medium | closed | `skills/implementing-stories/evals/fixtures/gates-build.toml` lines 1-2 are `schema = "devforgeai/gates/1"` / `cli_min_version = "1.0.0"`. `skills/validating-quality/evals/fixtures/gates-verify.toml:17` carries `kind = "tests_pass"`. Both confirmed by behaviour, not inspection alone: the preamble sweep ran `gate require build` and `gate require verify` in every one of those 14 workspaces at exit 0. A regression test exists: `test_every_gates_fixture_declares_the_schema_and_minimum_version`. |
| EVL-046 | medium | closed | Fixed in the CLI. In the kept `rf-01-no-sessions` workspace, whose `config.toml:51` reads `session_root = ".sessions"`, `devforgeai report aggregate --since 2026-08-28 --json` exits **0** with `sessions.status: "absent"`, `reason: "no session directory for this project"`, `errors: []`. `DFA-E421` is no longer emitted for an absent relative root. All 7 improving-framework preambles pass, so the five reflect aborts are gone and the `--workdir` dependence the audit described is removed. |
| EVL-061 | medium | open | No trigger set was written. `ls skills/*/evals/trigger-set.json` → nothing. `evals.json` was correctly left alone (all nine still conformant, see EVL-060). |
| EVL-063 | medium | closed | `disable-model-invocation: true` is present in eight `SKILL.md` files and absent from `skills/designing-interfaces/SKILL.md`, exactly the split the finding required. The consequence the finding drew — plan for one trigger set, not nine — is the unfinished half and is tracked as EVL-061. |
| EVL-070 | medium | closed | The recommendation was explicitly *not* to write `grading.json` / `benchmark.json`, and to add fields so a later `to_grading.py` needs no second run. Neither file is written by the runner; the fields landed (EVL-071) and `SCHEMA` is `devforgeai/eval-result/2` at `:124`. `specs/01-cli.md` §Evals carries the `devforgeai/eval-result/2` line schema. |
| EVL-071 | medium | closed | See EVL-015 for the recorded fields. `summarise()` at `:1120-1136` totals `total_cost_usd` and prints it: `cases 1 pass 1 fail 0 error 0 timeout 0 limit 0 duration 406.7s cost $1.39` — observed verbatim in `accept/improving-framework.log`. `specs/00-conventions.md` §9 documents the amended summary line and the five-status set including `limit`. |

Counts: blocker 6 closed / 0 open. High 8 closed / 0 open. Medium 9 closed / 3 open (EVL-004, EVL-042, EVL-061). Lows were not swept; three are referenced above and two more appear under Regressions.

## Mechanical checks

**1. `python -m unittest evals.runner.test_runner` (from the repo root)**

```
Ran 59 tests in 7.650s

OK
```

Three deliberate usage-error messages print to stdout during the run (absent binary, absent fixture, absent graders module, unknown case id) and five `ResourceWarning: unclosed file` warnings come from `test_runner.py:436`. No failures, no errors.

**2. `python evals/runner/run_jsonl.py --skill skills/<name> --dry-run --devforgeai-bin cli/target/release/devforgeai.exe --workdir <scratch>`, all nine**

| Skill | Cases | Exit | `ERROR` lines |
|---|---|---|---|
| designing-interfaces | 8 | 0 | 0 |
| discovering-requirements | 6 | 0 | 0 |
| establishing-context | 8 | 0 | 0 |
| exploring-ideas | 8 | 0 | 0 |
| implementing-stories | 8 | 0 | 0 |
| improving-framework | 7 | 0 | 0 |
| planning-work | 7 | 0 | 0 |
| releasing-software | 9 | 0 | 0 |
| validating-quality | 6 | 0 | 0 |

**67 cases, nine exits of 0, zero materialisation errors.** The audit's one failure (BLD-07) is gone. Answer seeding across the corpus: 23 cases carry an `answers` map, 44 carry none.

The argv every dry run prints (workspace elided):

```
claude -p --model sonnet --output-format stream-json --verbose
  --include-partial-messages --permission-mode bypassPermissions
  --allowedTools "Bash(devforgeai *),PowerShell(devforgeai *),Bash(devforgeai:*),
  PowerShell(devforgeai:*),Read,Write,Edit,Glob,Grep,Agent,Skill,AskUserQuestion,
  WebSearch,WebFetch,TodoWrite" --add-dir <workspace>
  prompt (<n> bytes, 1 lines) on stdin
```

**3. `python evals/runner/run_jsonl.py --skill skills/<name> --preflight --devforgeai-bin cli/target/release/devforgeai.exe --workdir <scratch>`, all nine**

| Skill | Exit | ok | SKIP | FAIL |
|---|---|---|---|---|
| designing-interfaces | 0 | 8 | 0 | 0 |
| discovering-requirements | 0 | 6 | 0 | 0 |
| establishing-context | 0 | 8 | 0 | 0 |
| exploring-ideas | 0 | 8 | 0 | 0 |
| implementing-stories | 0 | 7 | 1 | 0 |
| improving-framework | 0 | 7 | 0 | 0 |
| planning-work | 0 | 7 | 0 | 0 |
| releasing-software | **2** | 3 | 0 | **6** |
| validating-quality | 0 | 6 | 0 | 0 |

Totals 60 ok, 1 SKIP (`BLD-07  no preflight declared`), **6 FAIL**, one non-zero exit. The six failures verbatim:

```
FAIL  rl-02-compose-detect       devforgeai phase set release --id v0.3.0 -> exit 1: DFA-E320 … release needs verify gate PASS for STORY-017; no report at .devforgeai/reports/STORY-017-verify.yaml
FAIL  rl-03-none-library         (same, STORY-017)
FAIL  rl-04-deferral-sendback    (same, STORY-021)
FAIL  rl-05-verify-fail-sendback (same, STORY-017)
FAIL  rl-07-version-not-ahead    devforgeai phase set release --id v0.2.0 -> exit 1: (same, STORY-017)
FAIL  rl-08-api-coverage         (same, STORY-017)
```

**4. Preamble abort sweep (the EVL-013 measurement, re-run)**

Every `` !`…` `` line of each `SKILL.md`, executed with the release binary in each case's own materialised workspace, `$ARGUMENTS`/`$1` substituted from the case prompt:

```
designing-interfaces       no preamble, 8 cases
discovering-requirements   cases 6  preamble lines 1  ABORT 0
establishing-context       cases 8  preamble lines 2  ABORT 0
exploring-ideas            no preamble, 8 cases
implementing-stories       cases 8  preamble lines 4  ABORT 0
improving-framework        cases 7  preamble lines 1  ABORT 0
planning-work              cases 7  preamble lines 3  ABORT 0
releasing-software         cases 9  preamble lines 1  ABORT 0
validating-quality         cases 6  preamble lines 4  ABORT 0
TOTAL cases 67  aborting 0
```

Was 33 of 67.

**5. Grader and fixture conformance, all nine**

```
skills 9 cases 67
UNUSED-GRADERS discovering-requirements ['acceptance_recorded', 'personas_resolve']
UNUSED-GRADERS planning-work            ['design_called', 'sprint_shape']
UNUSED-GRADERS validating-quality       ['load_yaml']
```

Zero missing graders, zero arity mismatches (every named grader parses as `(workspace, transcript, args)`), zero unresolved `FIXTURE:` references across all 67 cases. `load_yaml` is a module helper, not a grader.

**6. `gates-default.toml` fixtures vs `cli/templates/gates.default.toml`**

```
template sha f28eb0bee7094544  size 13982  count 5
SAME  skills\designing-interfaces\evals\fixtures\gates-default.toml      13982 f28eb0bee7094544
SAME  skills\discovering-requirements\evals\fixtures\gates-default.toml  13982 f28eb0bee7094544
SAME  skills\establishing-context\evals\fixtures\gates-default.toml      13982 f28eb0bee7094544
SAME  skills\exploring-ideas\evals\fixtures\gates-default.toml           13982 f28eb0bee7094544
SAME  skills\planning-work\evals\fixtures\gates-default.toml             13982 f28eb0bee7094544
```

All five byte-identical.

**7. `evals/` excluded from `init`'s copy**

`cli/src/cmd/init.rs:20` `const EXCLUDED_SUBTREES: &[&str] = &["evals"];`, applied in `copy_tree()` at `:332-340` through `WalkDir::filter_entry` on any non-root directory with that name; `copy_skills()` at `:420` calls `copy_tree` per skill directory, so `skills/<name>/evals/` is filtered. Verified empirically — `devforgeai init --from C:/Projects/DevForgeAI` into a fresh git repo:

```
Copied     102 skills, 46 agents
Commands   /build, /constitute, /design, /discover, /explore, /plan, /reflect, /release, /verify
```

then `find .claude \( -name evals -o -name cases.jsonl -o -name graders.py -o -name __pycache__ -o -name results.jsonl \)` returned nothing. `__pycache__` is not in `EXCLUDED_SUBTREES` but every one under `skills/` lives inside an `evals/` directory, so it is excluded transitively.

**8. Runner isolation and gating properties (asked for by name)**

| Property | Where | Verdict |
|---|---|---|
| strips session env vars | `child_environment()` `:775-779` removes every `CLAUDE_CODE_*` plus `CLAUDECODE`, `CLAUDE_PID`, `AI_AGENT`, `CLAUDE_EFFORT`, `ANTHROPIC_API_KEY` | yes, **conditional** on `CLAUDECODE` being set — outside a Claude Code session nothing is removed, which is stated in the comment and in `specs/01-cli.md` §Evals |
| redirects `CLAUDE_CONFIG_DIR` | `isolate_claude_config()` `:733`, wired at `:767-768` | yes, default `isolate`; `--claude-config inherit` opts out; `.credentials.json` is the one file carried across |
| gates hooks on `trust verify` | `trust_gate()` `:1083`, `main()` `:1166-1176` | yes — falls back to `--no-hooks` with the pin command, or exits 3 when `--hooks` was explicit. Observed in every acceptance log: `hooks off: devforgeai: DFA-E501 trust verify: …trust.toml not found` |
| permission host for seeded cases | `permission_host()` `:408`, `claude_argv()` `:635-638` | yes — `--mcp-config <workspace>/.claude/mcp-permissions.json --permission-prompt-tool mcp__dfa-permissions__approve`, and only for a case with a non-empty `answers` map |
| `--permission-mode bypassPermissions` | `PERMISSION_MODE` `:154` | yes. Reason given, verbatim at `:147-153`: `acceptEdits` covers Write and Edit alone, `claude -p` has no approval surface so anything else is denied outright with no one to ask, and a compound shell command (`cd wt/STORY-014 && ./ci/test`, what Build step 7.2 runs) matches no `Bash(devforgeai *)` rule; the blast radius is bounded because the workspace is a throwaway under `mkdtemp`, the child sees no user configuration, and the framework's `PreToolUse` deny is claimed to fire in every permission mode. The same reason is in `specs/01-cli.md` §Evals and `specs/00-conventions.md` §9 |

**9. Case-to-spec drift and `evals.json` conformance (EVL-060 regression check)**

Every case id and every grader name in all nine `cases.jsonl` still appears in the owning spec's `## Evals` section — zero drift after the fix wave. All nine `evals.json` remain skill-creator conformant: `skill_name` equals the directory, every `id` an integer, all five keys present, every `expectations` list 3-6 entries, counts 8/6/7/8/6/7/6/9/6, all at or above the §9 floor of 6.

**10. Digest arguments recomputed against the bytes the cases write**

Every `upstream_unchanged`, `upstream_sha256`, `baseline_sha256`, `source_sha256` and `story_sha256` entry across all 67 cases was recomputed from the fixture or inline content the case seeds: **14 of 14 MATCH.** A recorded digest that silently disagreed with its fixture would have reintroduced the EVL-041 failure mode under a new name; none does.

**11. Acceptance evidence read from `accept/` (no case launched by this verifier)**

Nine one-case real runs, all with `"hooks": false` because `trust verify` fails on this machine:

| Case | Status | Duration | Cost | Evidence |
|---|---|---|---|---|
| `dz-04-spec-sections` | pass | 530.5 s | $1.72 | — |
| `ex-01-fresh-brief` | pass | 650.4 s | $1.77 | `tool_calls` includes `AskUserQuestion: 1` |
| `rf-02-rec-cites-obs` | pass | 406.7 s | $1.39 | `REC-001 -> OBS-001,OBS-002; REC-002 -> OBS-003` |
| `disc-brief-happy` | fail | 95.5 s | $0.26 | `.devforgeai/requirements.yaml is absent from the workspace` |
| `CON-01-greenfield-headings` | fail | 164.6 s | $0.66 | `.devforgeai/context/tech-stack.md is absent` |
| `BLD-01` | fail | 196.9 s | $0.78 | `no note at .devforgeai/build/STORY-014-note.yaml` |
| `PLAN-01-headings-and-coverage` | fail | 357.0 s | $0.98 | `found 0 story files under .devforgeai/stories, expected at least 2` |
| `rl-01-k8s-pass` | fail | 37.0 s | $0.14 | `.devforgeai/releases/v0.3.0.yaml does not exist` |
| `vq-01-light-pass` | fail | 61.7 s | $0.25 | `.devforgeai/reports/STORY-014-qa.yaml is absent`; one `permission_denials` entry, a `Bash` call attempting `cd /c/Projects/DevForgeAI/cli` |

3 pass, 6 fail. These are skill-behaviour outcomes, not runner defects — the runner reached the grader in all nine, recorded the full `devforgeai/eval-result/2` line, and wrote the raw streams outside the workspace. `disc-brief-happy` (`tool_calls: {"Bash": 9}` and nothing else in 10 turns), `rl-01` (37 s) and `vq-01` (10 turns, 6 Bash + 3 Read) are early-stop shaped and warrant a separate look by whoever owns the skills. A re-run of establishing-context, implementing-stories and releasing-software was launched into the same directory at 14:05 while this verification was in progress, overwriting those three result files; the figures above were read before it started.

## Deviations from the audit's fix specification

1. **`--permission-mode bypassPermissions`, not `acceptEdits`.** The fix spec at AUDIT-5 `### EVL-010 + EVL-011 + EVL-012 + EVL-014` prescribes `acceptEdits`. The runner uses `bypassPermissions`. The reason is recorded in code (`run_jsonl.py:147-154`), in `specs/01-cli.md` §Evals and in `specs/00-conventions.md` §9, and it is empirical: under `acceptEdits` a compound shell command matches no allow rule and headless has no one to ask. Accepted, with the widened EVL-004 surface recorded as open.
2. **`--include-partial-messages` added** beyond the prescribed argv, with the reader skipping the duplicated `stream_event` text. Additive, documented, covered by `test_read_stream_renders_text_tool_calls_results_and_meta`.
3. **A stdio permission host was added**, which the fix spec did not anticipate. It is load-bearing, not decoration: the runner and both specs record the measurement that `claude -p` with no host exposes 33 tools and no `AskUserQuestion`, so the answer hook would have had nothing to intercept and EVL-030 would have closed on paper only. Accepted.
4. **`devforgeai worktree list` was not added to BLD-07's `expect_calls`**, which the EVL-040 fix spec asked for as the final step ("so the case asserts the model looked before it wrote"). BLD-07's `expect_calls` is `["devforgeai worktree ensure STORY-015"]` alone, in both `cases.jsonl` and `specs/06-build.md:886`. Minor; recorded under Open items.
5. **A `preflight` key and `--preflight` mode were added**, beyond anything AUDIT-5 asked for. Good addition — it is what surfaced the releasing-software defect below — but it is only as strong as the cases that populate it, and BLD-07 declares an empty one.

## Open items

**EVL-004 — tamperable gate and state files, widened.** `bypassPermissions` plus an unpinned machine means a model in an eval can rewrite `.devforgeai/gates.toml`, `.devforgeai/state.toml`, and anything under `DEVFORGEAI_HOME`, unprompted and unblocked. Reachable path: any case, today. What keeps it harmless is that no grader reads either file — confirmed, `grep -ln "state.toml\|gates.toml" skills/*/evals/graders.py` matches nothing. What remains: either keep the invariant explicit (a test asserting no grader reads `gates.toml`/`state.toml`, so a future grader cannot silently make tampering profitable), or restore a permission mode that does not bypass. Owner: F5.

**EVL-042 — the SEND BACK floor is unmet and unamended.** `specs/00-conventions.md` §9 item 2 still says "At least 2 exercise the SEND BACK path" and no spec grants an exemption. Four skills fall short, two more than the audit named: exploring-ideas 0, improving-framework 0, implementing-stories 0, designing-interfaces 1. What remains: either amend §9 to exempt entry and terminal phases and say so per skill (and decide whether Build's `Blocked` path counts, which BLD-07/BLD-08 exercise), or add cases. Owner: F6 for the §9 wording, F5 for any cases. Note the audit assumed implementing-stories and designing-interfaces were fine; they are not, against the rule as written.

**EVL-061 — no trigger set exists.** `skills/*/evals/trigger-set.json` does not exist for any skill. The dependency the audit named is now satisfied — EVL-063 landed, and `designing-interfaces` is the one model-invocable skill — so the single trigger set the audit asked for can be written. `evals.json` was correctly left untouched. Owner: F5.

**BLD-07's `expect_calls` is short one literal.** The EVL-040 fix spec's last step, adding `devforgeai worktree list` so the case asserts the model looked before it wrote, was not applied. Owner: F5, with a matching edit to `specs/06-build.md:886`.

**EVL-044's §9 exception was never written.** `specs/00-conventions.md` §9 item 3 still describes graders as pure functions with no statement about reading outside the workspace, while `skills/planning-work/evals/graders.py:106` `_fixture()` and `skills/discovering-requirements/evals/graders.py` both read their own `evals/fixtures/` by design — and EVL-041's fix *increased* the framework's dependence on that. The exception is stated locally at `specs/05-plan.md:768` but not in the convention that governs all nine. Owner: F6. Low severity, but it is now load-bearing rather than incidental.

**EVL-043 is unchanged.** Four graders are still defined and named by no case — `personas_resolve`, `acceptance_recorded` (discovering-requirements), `design_called`, `sprint_shape` (planning-work). `design_called` remains the only grader covering the Plan→Design hand-off and no case names it. `establishing-context.no_ceremony` has gone. Low severity, unfixed.

## Regressions

**R1 — six of nine releasing-software cases fail their own declared preflight.** `python evals/runner/run_jsonl.py --skill skills/releasing-software --preflight` exits **2**. `rl-02`, `rl-03`, `rl-04`, `rl-05`, `rl-07` and `rl-08` each declare `preflight: ["phase set release --id vX.Y.Z"]` and that call exits 1 with `DFA-E320 … release needs verify gate PASS for STORY-017` (STORY-021 for `rl-04`). Root cause: `skills/releasing-software/evals/fixtures/sprint-active.yaml` names STORY-014, STORY-017 and STORY-021 (lines 14, 18, 22), but those six cases seed only `.devforgeai/reports/STORY-014-verify.yaml`. `rl-01`, `rl-06` and `rl-09`, which seed all three verify reports, pass. This is not a preamble abort — `/release`'s only preamble line is `` !`devforgeai story list --status built --json` `` and it exits 0 — it is step 2 of `skills/releasing-software/SKILL.md:38`, "Activate the version", which every one of those six runs reaches and fails. Functionally this is EVL-013's failure mode relocated from the preamble into the workflow's second step, and the new `--preflight` mode is what exposed it. The acceptance run for `rl-01` is consistent with an early stop (37 s, $0.14, `.devforgeai/releases/v0.3.0.yaml does not exist`) although `rl-01` itself passes preflight, so there may be a second cause behind it. Fix: seed the missing verify reports (the fixtures `verify-pass-014.yaml` and its siblings already exist) or narrow the sprint fixture those six cases inherit. Owner: F5.

**R2 — the dry run prints an argv the real run does not use.** `parse_args` at `run_jsonl.py:1079-1081` forces `options.prompts_none = False` under `--dry-run` and `--preflight`, so `--permission-prompts none` never appears in any dry-run output even though the installed `claude` is 2.1.268 and a real run appends it to all 44 unseeded cases. `specs/01-cli.md` §Evals step 1 calls the dry run the place where "the argv and the prompt are printed". The version probe is one `claude --version` exec and skipping it is understandable, but the printed argv is then not the argv. Fix: probe once and cache, or print the flag with a "(version-gated)" annotation. Owner: F5. Low severity, fidelity only.

**R3 — a fixture was orphaned by the BLD-07 rewrite.** `skills/implementing-stories/evals/fixtures/wt-story-014-git.txt` is now referenced by no case and by no grader; the rewrite replaced it with `setup.git.worktrees`. Three other orphans predate the wave and are EVL-043 class: `digests.txt` (establishing-context, exploring-ideas, improving-framework), `requirements-IDEA-021.yaml` and `requirements-IDEA-022.yaml` (establishing-context). Owner: F5. Low severity.

**R4 — `BLD-07` declares `"preflight": []`.** It is the one case in the corpus that opts out of the new mode, and it is also the case whose whole subject is a CLI precondition (`worktree ensure` returning `DFA-E272` against a seeded overlapping worktree). The preflight run reports `SKIP BLD-07 no preflight declared`, so the seeded overlap is never checked before a real, paid run. Owner: F5. Low severity.

No test regressed: the Python suite is 59/59 green, no spec fence diverged from a shipped fixture (all five `gates-default.toml` copies are byte-identical, and BLD-07's spec line matches its `cases.jsonl` line), and no case names a grader, fixture or agent that does not exist.

## Residual risks

**The enforcement half of every eval is still unexercised on this machine, and nothing refuses.** EVL-003's mechanism landed — the framework hook block is written into the workspace by default — but it is conditional on a `devforgeai trust pin` that only a human can make outside Claude Code. Without it `trust_gate()` prints `hooks off: …` and the run continues. All nine acceptance runs recorded `"hooks": false`, so the producer check, the gates, the Stop block and `SubagentStop` ingest were measured by nothing, exactly as the audit described. The runner offers `--hooks` to turn this into a hard refusal; no acceptance run used it. Until a pin exists and a hooks-on sweep is run, the framework's enforcement behaviour is covered by the Rust suite alone.

**The `bypassPermissions` safety argument rests on an untested claim.** `specs/01-cli.md` §Evals and `specs/00-conventions.md` §9 both assert that "a `PreToolUse` deny fires in every permission mode, `bypassPermissions` included, and no allow overrides it". That is what bounds the blast radius of bypassing the prompt, and it is the load-bearing half of the deviation in item 1 above. It has not been exercised here: the combination that matters — `bypassPermissions` **with** the framework hooks registered — has never run, because every run on this machine was hooks-off. The first hooks-on sweep should assert it directly, for example a case whose model attempts a write the producer check must deny.

**Grader field names were cross-checked and are clean, but the check has a floor.** Every string constant in every grader's key-order and enum lists (`TOP_KEYS`-shaped module constants across all nine modules) resolves to a token that appears in the owning skill's `SKILL.md`, `agents.md`, `templates/`, `references/`, or in some `agents/*.md` — zero unmatched across nine modules. Separately, every `subagents` and `forbid_calls` agent name a case asserts on exists both as `agents/<name>.md` and in the owning skill's `agents.md` (`security-auditor`, `code-quality-auditor`, `adr-conformance-reviewer`, `ac-test-writer`), and both `expect_calls` literals correspond to real workflow steps (`skills/implementing-stories/SKILL.md:51` for `worktree ensure`, `:55` for `config get stack.test_command`). What this does not prove is that a key a grader reads out of a *produced* document is a key the skill instructs the model to write in that exact position; that is the skills verifier's ground, not this one's.
