---
schema: devforgeai-audit/1
area: evals
produced_by: audit-evals
---

# AUDIT-5 evals

Scope read: `specs/ANTHROPIC-GUIDANCE.md` §3, §4, §7; `specs/00-conventions.md` §9; `specs/01-cli.md` `## Evals` (lines 2503-2700) including the `### Python, the shared eval runner` H3; `evals/runner/run_jsonl.py`; `evals/runner/test_runner.py`; all nine `skills/*/evals/` trees (67 cases, 61 grader functions, 8 fixture directories); `commands/*.md`; the skill-creator plugin's `references/schemas.md` and `scripts/run_eval.py`.

Measured on this machine: `claude 2.1.268`; `python 3.10.11`; `devforgeai` **not on PATH**, binary present at `C:\Projects\DevForgeAI\cli\target\release\devforgeai.exe` reporting `devforgeai 1.0.0 (unversioned)`.

Counts: 39 findings — 6 blocker, 8 high, 12 medium, 13 low.

## 1. Gaming vectors

Verified negatives first, so the rest of the table is read against them.

- `make_workspace` copies `--skill` with `ignore_patterns("__pycache__", "evals")`. `ignore_patterns` matches basenames at every depth, so no nested `evals/` survives either. Graders, fixtures, cases and `results.jsonl` all live under `skills/<name>/evals/`, outside every workspace.
- No symlink exists anywhere under `skills/` (`find skills -type l` is empty), so `copytree(symlinks=False)` cannot dereference one past the ignore filter.
- No grader function name appears in any file the copy carries (`SKILL.md`, `agents.md`, `references/`, `templates/`). The two hits for `send_back` are the `send_back_to` gate key, not a grader reference.
- `--add-dir` names only the workspace; cwd is the workspace; PATH carries no framework directory; the framework root is not an ancestor of the workspace (it sits under the system temp directory).
- No grader reads `.devforgeai/gates.toml` or `.devforgeai/state.toml`. Editing either buys no pass today.

| ID | Severity | Location | Guidance or spec says | What exists | Failure scenario |
|---|---|---|---|---|---|
| EVL-001 | high | `run_jsonl.py` `invoke()`, `make_workspace()` | Conventions §9 and `01-cli.md` §Evals: the workspace is the measurement boundary; Decision 62 redirects the home "so no trust pin on the real machine is read" | Only `DEVFORGEAI_HOME` is redirected. `HOME`/`USERPROFILE`/`CLAUDE_CONFIG_DIR` are inherited, so `~/.claude/settings.json`, `~/.claude/skills/`, `~/.claude/agents/` (46 files), `~/.claude/plugins/` and a user `CLAUDE.md` all load into every eval | The eval measures the machine, not the framework. A user-level skill or agent, a user `permissions.allow` entry, or a user hook changes a grade with no change to the framework. Checked on this machine: no user `permissions.allow`, no user `CLAUDE.md`, no name collision between `~/.claude/agents/*.md` and the 46 framework agents — so no leak fires here, but nothing in the runner prevents one |
| EVL-002 | high | `run_jsonl.py` `invoke()` lines writing `.dfa-claude-stdout.txt` / `.dfa-claude-stderr.txt` | `01-cli.md` §Evals step 7 to 9: take the transcript, call the grader, append a line; nothing sanctions writing into the workspace before grading | Both files are written at the workspace root *before* `grade()` runs | `writes_inside_declared_set` (BLD-02) walks the workspace, skips only the `baseline` roots `.devforgeai`, `README.md`, `.claude`, `.git`, and reports `.dfa-claude-stdout.txt` and `.dfa-claude-stderr.txt` as "outside the declared set". Scope check before rating: `no_ceremony` walks `.devforgeai/` only and no other grader walks the workspace root, so the blast radius is exactly one case — and BLD-02 is already dead twice over from EVL-013 (its gates fixture aborts the preamble) and from `worktree_root: "../wt"` being unreachable under `--add-dir <workspace>`. Rated high, not blocker: the runner writing into the measurement boundary is a real defect with a correct fix, but it is not the sole cause of any failing case |
| EVL-003 | high | `make_workspace()` "Decision 42: an empty settings file, so no hook runs during an eval" | `ANTHROPIC-GUIDANCE.md` §1: "An instruction in CLAUDE.md or a skill is a request, not a guarantee. A PreToolUse hook that blocks the edit is enforcement." §7 item 2 and item 4 hang the producer check and the Stop gate on hooks | `.claude/settings.json` is `{}`; `DEVFORGEAI_HOME` points inside the workspace; no `hook install` runs | Stated plainly: **during an eval nothing enforces the producer check, no gate is evaluated, no Stop hook blocks, and no `SubagentStop` ingests a verifier envelope.** Every eval measures only whether the model's prose follows the skill. A skill that writes a document with a wrong `produced_by`, skips `gate check`, or emits an unparsable verifier envelope passes its evals. The enforcement half of the framework is covered only by the Rust suite, and no eval case crosses it |
| EVL-004 | medium | workspace `.devforgeai/gates.toml`, `.devforgeai/state.toml`, `.claude/dfa-home/` | Conventions §8 trust model; `01-cli.md` §Evals step 5 | `--permission-mode acceptEdits` auto-accepts edits and no hook runs, so the model can read and rewrite `gates.toml`, `state.toml` and anything under `DEVFORGEAI_HOME` | Today no grader reads those files, so tampering buys nothing. It becomes live the moment a grader asserts on `state.toml` or a case grades the outcome of a preamble `gate require`. The `DEVFORGEAI_HOME` directory is writable, so a model can plant `trust.toml` and make any `devforgeai trust verify` it reaches succeed |
| EVL-005 | high | 12 grader functions, listed in the strength table | `01-cli.md` §Evals: "Graders decide pass/fail" | 12 of 56 graders in use rest, in their discriminating half, on a phrase in model-authored text | Answering the question directly: **yes, a grader can be satisfied by output that merely names the expected strings.** `asked_decision` (ex-07) passes if the final message contains the literal `"header": "Decision"` followed within 2000 characters by the option labels — the model can type that without ever calling `AskUserQuestion`. `blocked_line` (BLD-07, BLD-08), `remedy_line` (BLD-03, BLD-04), `handoff_returns` (rf-07), and the five `send_back`/`sendback_block` graders pass on a handoff block the model prints, with no artifact cross-check |
| EVL-006 | low | `parse_args()` `--workdir` default | `01-cli.md` §Evals: `--workdir` defaults to "the system temp directory" | Workspaces are siblings in the bare system temp directory, forever with `--keep` and `--dry-run` (which forces `--keep`) | BLD-02's `worktree_root: "../wt"` makes `writes_inside_declared_set` walk the *shared parent*. A `wt/` directory left by an earlier `--keep` run, or created by a concurrent `--jobs > 1` case, is collected as a write. Recommend `tempfile.mkdtemp(prefix="dfa-evals-")` per run, under the system temp, and keep `--workdir` as an override |
| EVL-007 | low | verified negative | — | The skill copy, PATH, `--add-dir`, symlinks and the copied files are all clean of grader, fixture, case and result content | Recorded so a later change that adds `--add-dir <framework root>`, drops the `evals` ignore pattern, or moves fixtures inside the skill body is caught as a regression |

### Grader strength

`strong` = the decision parses a produced artifact's structure **and** cross-references ids or values between artifacts, against `args`, or against a recorded digest. `substring` = the discriminating condition is a phrase in text the model itself authored. `presence` = the decision is file existence alone.

Totals over the 56 grader functions the 67 cases name: **44 strong, 12 substring, 0 presence.** Five further grader functions are defined and used by no case (EVL-043).

| Skill | Grader | Class | Why |
|---|---|---|---|
| designing-interfaces | `tokens_shape` | strong | group order, leaf regex, per-group counts, light/dark hex shape |
| designing-interfaces | `brand_kit_shape` | strong | heading order, per-section row counts, Figma row labels |
| designing-interfaces | `figma_fallback` | **substring** | two table cells plus `os.path.getsize() > 0`, and a required `transcript.count("Gate      PASS") >= 1` |
| designing-interfaces | `ui_spec_shape` | strong | frontmatter keys, heading order, breakpoint pairs, eight a11y rows, default state |
| designing-interfaces | `tokens_only` | strong | cross-references every `TOKEN-` reference in every UI spec against `tokens.json` |
| designing-interfaces | `sendback_block` | **substring** | Gate/Next/Then lines from the transcript plus a UI-spec file count ceiling |
| designing-interfaces | `remedy_touched_only` | strong | per-section SHA-256 against recorded baselines |
| designing-interfaces | `sketch_contract` | strong | cross-references transcript JSON flow ids against the request and every screen path against disk |
| discovering-requirements | `requirements_document_wellformed` | strong | 14 keys in order, actor→persona resolution, one epic per requirement, numeric success metric |
| discovering-requirements | `sendback_to_explore` | **substring** | absence of `requirements.yaml` plus Gate/Next/Found transcript lines |
| discovering-requirements | `remedy_preserved_other_requirements` | strong | per-entry digests against a fixture baseline, revision log arithmetic |
| discovering-requirements | `design_remedy_created_requirement` | strong | cross-references `traces_to`, `revision_log.added`, epic membership, persona digests |
| establishing-context | `context_headings` | strong | H2 list equality per file |
| establishing-context | `con_referenced` | strong | cross-references every active CON to an ADR table or a resolving REQ |
| establishing-context | `non_goal_constraints` | strong | source attribution split plus index membership |
| establishing-context | `drafts_accepted` | strong | six files, status, empty `open_questions`, no empty section |
| establishing-context | `send_back` | **substring** | handoff block lines; the SHA-256 upstream check only proves the input was not edited |
| establishing-context | `remedy_supersession` | strong | ADR supersession table, CON row retirement, replacement allocation, untouched rows |
| establishing-context | `key_consistency` | strong | key/value table across six files |
| exploring-ideas | `brief_shape` | strong | 7 frontmatter keys, 12 headings in order, FLOW id shape and count |
| exploring-ideas | `decision_shape` | strong | decision enum, date ordering, carry-forward length, reason word count |
| exploring-ideas | `carry_forward_exact` | strong | 7 entries, required keys, every carried path resolves on disk |
| exploring-ideas | `remedy_touched_only` | strong | row-level and section-digest comparison, `remedied_flows` set equality |
| exploring-ideas | `open_questions_mentions` | strong | open-question id citation plus row change/hold matrix |
| exploring-ideas | `sketch_request_contract` | strong | cross-references request flow ids against the brief's Core flows order |
| exploring-ideas | `asked_decision` | **substring** | transcript only; searches for `"header": "Decision"` then the labels within 2000 chars |
| implementing-stories | `build_note_shape` | strong | schema, AC order, declared-set membership, distinct 40-hex red/green shas |
| implementing-stories | `writes_inside_declared_set` | strong | walks the tree (broken by EVL-002) |
| implementing-stories | `remedy_line` | **substring** | three transcript literals, a forbidden-literal list, a block-length cap |
| implementing-stories | `criterion_set` | strong | note cycle list, remedy list, resumed_at, prior green shas; its `forbid_acs` half is vacuous (EVL-010) |
| implementing-stories | `blocked_line` | **substring** | one `Blocked   you: ` line plus `expect_calls`/`forbid_calls` transcript literals |
| improving-framework | `reflect_report_shape` | strong | 12 keys in order, obs kind/severity enums, cited sources resolve on disk |
| improving-framework | `rec_cites_obs` | strong | every REC cites a defined OBS, target-kind/path consistency |
| improving-framework | `obs_names_pattern` | strong | count, summary tokens, session-id citation, evidence line numbers |
| improving-framework | `debt_groups` | strong | group tuples, totals, per-item `age_days` recomputed from `args.as_of` |
| improving-framework | `no_lowered_floor` | strong | every threshold recommendation compared to a compiled floor |
| improving-framework | `handoff_returns` | **substring** | transcript block only: line count, Gate/Next/Then equality, absent labels |
| planning-work | `story_headings` | strong | ten H2 headings per story, in order |
| planning-work | `req_coverage` | strong | REQ set across all stories equals the epic's set |
| planning-work | `ac_grammar` | strong | Given/When/Then regex, covered-once cross-check against the Requirements table |
| planning-work | `file_sets_disjoint` | strong | declared paths disjoint, dependency order below dependent order |
| planning-work | `send_back` | **substring** | Gate/Next/Found lines, plus a `.setup` comparison that cannot run (EVL-041) |
| planning-work | `remedy_ac` | strong | AC-level diff, sprint line-by-line diff — but reads `.setup` (EVL-041) |
| releasing-software | `release_shape` | strong | key order, story set, platform pair, manifest basenames non-empty, signoff arithmetic |
| releasing-software | `platform_resolution` | strong | target/source/marker triple plus compose→`.env` variable pairing |
| releasing-software | `docs_layout` | strong | required files non-empty, empty keys empty, H2 order per page |
| releasing-software | `sendback_block` | **substring** | Gate/Next/Then/Found lines; the release-file and story-status checks are secondary |
| releasing-software | `resume_untouched` | strong | SHA-256 per manifest, blocking-deferral scan |
| releasing-software | `version_refused` | **substring** | absent paths, transcript tokens, and "no `Gate      PASS` line" |
| releasing-software | `api_coverage` | strong | cross-references every symbol to an H3 on an API page and a row in the index |
| releasing-software | `release_notes` | strong | kind derived from the story's own `consumes`, requirement union, summary length |
| validating-quality | `qa_report_shape` | strong | key order, check-name set, coverage copied from the build report, enum fields, source digests |
| validating-quality | `deep_mode_checks` | strong | mode, check set, OWASP enum, complexity/duplication limits from config; its subagent-name check is unsatisfiable (EVL-010) |
| validating-quality | `sendback_block` | **substring** | handoff Gate/Next/Then plus blocker/deferral report assertions |
| validating-quality | `deferral_recorded` | strong | entry key order, reason enum, `opened_on` == `verified_on`, target document resolves |
| validating-quality | `cycle_detected` | strong | cross-report evidence citation, re-disposition, forbidden target |

## 2. Runner versus the headless contract

Current argv, `claude_argv()`:

```
claude -p "<prompt>" --model sonnet --output-format json --permission-mode acceptEdits --add-dir <workspace>
```

with `cwd=workspace`, `stdin=DEVNULL`, `env["DEVFORGEAI_HOME"]=<workspace>/.claude/dfa-home`.

| ID | Severity | Location | Guidance or spec says | What exists | Failure scenario |
|---|---|---|---|---|---|
| EVL-010 | blocker | `transcript_of()`, `claude_argv()` | `ANTHROPIC-GUIDANCE.md` §4: "`--output-format json` returns `result` … `stream-json --verbose` streams events incl. subagent messages with `parent_tool_use_id`" | The transcript is the `result` string alone, so no tool call, no tool input and no subagent message is ever in it | Asymmetric failure. Every `expect_calls` assertion can **never** be satisfied (`devforgeai worktree ensure STORY-015`, `devforgeai config get stack.test_command`), and every `forbid_calls` / `forbid_acs` / `forbid_next` assertion **passes vacuously on absence of evidence**: BLD-07 and BLD-08's `forbid_calls`, BLD-05/BLD-06's `Agent\(ac-test-writer…AC-\d{3}` regex, vq-02's `args.subagents` list, and `design_called`'s `/design … --spec` regex. Two are strictly unsatisfiable (BLD-07/BLD-08 `expect_calls`, vq-02 `subagents`), four are strictly vacuous |
| EVL-011 | high | `claude_argv()` | §4: "`claude -p` starts in Manual permission mode on every plan; pass `--permission-mode acceptEdits \| auto \| dontAsk` or `--allowedTools`". §3: `allowed-tools` is a "grant for the invoking turn only" | `acceptEdits` auto-accepts Edit/Write and nothing else; no `--allowedTools`; the workspace `settings.json` is `{}` so it carries no `permissions.allow`. `--permission-prompts` defaults to `host`, and plain `claude -p` has no host, so anything that would prompt is denied | The brief's premise is partly wrong and the correction matters: `commands/*.md` already carry `allowed-tools: Bash(devforgeai:*), …`, and a `-p` run is one turn, so a case whose prompt expands the slash command *does* get the Bash grant. The residual holes: (a) no `SKILL.md` carries `allowed-tools`, so a `Skill`-tool invocation or a prompt that fails to expand as a command has no Bash grant at all; (b) a prompt that does not start with `/name` (none today, but `--resume`-shaped cases would) gets nothing; (c) the copied subagents carry their own `tools:` lists and inherit no grant; (d) the denial is silent, and with `result`-only capture it is invisible in the record (see EVL-015) |
| EVL-012 | blocker | environment; `invoke()` | §3: "`` !`command` `` … **a non-zero exit aborts the whole skill invocation**" | `devforgeai` is not on PATH on this machine and the runner neither prepends the binary's directory to the child PATH nor accepts a `--devforgeai-bin` option. Every one of the nine command files opens with at least one `` !`devforgeai …` `` line | With the binary absent, the shell returns 127 for every preamble line and **all 67 cases abort before the skill body is loaded**. Every grade in the suite would be a false negative attributable to an unstated prerequisite |
| EVL-013 | blocker | `skills/*/evals/cases.jsonl` and `skills/*/evals/fixtures/` | Same §3 rule | Measured with the real binary against every materialised workspace: **33 of 67 cases** have a preamble line that exits non-zero. Breakdown below | Those 33 cases abort before the skill runs and grade as failures of the skill |
| EVL-014 | high | `parse_args()` `--timeout` default 300 | `01-cli.md` §Evals: `--timeout` default 300. `specs/questions.md` Q-017 records the one observed real run: **858 s**, \$2.82, on sonnet, for `ex-08` | 300 s default against a 2.9× longer observed run | Every case records `status: timeout` with evidence `no result within 300s`. Raise the default to 900 s and document the observed figure beside it |
| EVL-015 | medium | `run_case()` record construction | §4: "`--output-format json` returns `result`, `session_id`, `total_cost_usd`, `permission_denials`, `num_turns`" | The record keeps `exit_code`, `transcript_bytes`, `duration_ms`. `session_id`, `total_cost_usd`, `permission_denials`, `num_turns`, `is_error` and `subtype` are parsed away and discarded | A denied `Bash(devforgeai …)` is invisible: the case fails on a grader message that names a missing artifact, while the cause was a permission denial that `permission_denials` names exactly. No cost accounting either, which the 858 s / \$2.82 pilot shows is material |
| EVL-016 | medium | `invoke()`, `transcript_of()` | — | `finished.returncode` is stored but never gates grading, and `transcript_of` returns the **raw stdout** when JSON parsing fails | A crashed or usage-error run still reaches the grader, and the raw stdout of an error envelope echoes the prompt. A substring grader whose literal appears in the prompt (`blocked_line` args include ids that are in the prompt; `remedy_line` expects `/build STORY-014 --resume`) can pass on an error envelope. Treat a non-zero exit or a parse failure as `status: error` before grading |
| EVL-017 | low | `claude_argv()` | §4: "`--json-schema` constrains `structured_output`" | Not used | It has one narrow place, not a general one: `sketch_contract` and `sketch_request_contract` grade a JSON object the model emits *in prose*, found by scanning the transcript for the last `{…}` carrying a `screens` key. Those two cases would be sounder with `--json-schema` and a grader reading `structured_output`. Every other grader reads files, so a blanket `--json-schema` would constrain output for no gain |
| EVL-018 | low | `claude_argv()` | §4: "`--bare` … loads skills (not commands or agents) from `--add-dir`" | `--add-dir <workspace>` is passed while `cwd` is already the workspace | Redundant but not harmful, and it is the property that keeps the framework root out of reach. Keep it and record *why* in the spec, so a later "cleanup" does not replace it with `--add-dir <framework root>`. `--bare` is not usable here: it would drop the copied commands and agents the cases depend on |
| EVL-019 | low | `run_case()` `finally` block | `01-cli.md` §Evals line schema: `"workspace": "/tmp/dfa-eval-…"` | `record["workspace"]` is re-assigned *after* `_rmtree()` | Without `--keep`, every results line names a directory that does not exist. Either record it only under `--keep`, or add a `workspace_kept` boolean |
| EVL-020 | low | `main()` `--jobs` path | `01-cli.md` §Evals: `--jobs <n>` cases run in parallel | All jobs share one `options.workdir`, one graders module and one output file handle (lock-protected) | `writes_inside_declared_set` resolving `../wt` into the shared workdir means a parallel BLD case can see another case's tree. Give each case a `mkdtemp` under the run directory |

### Preamble abort breakdown (measured, real binary, materialised workspaces)

Method and its one approximation: each case's workspace was materialised through `run_jsonl.make_workspace`, then every `` !`…` `` line of that skill's command file was run in it with the real binary. `$1` and `$ARGUMENTS` were substituted from the case prompt's first line, which is close to but not identical to Claude Code's own slash-command argument parsing — `/reflect --since 2026-08-28` yields `$1 == "--since"` under the real parser. The file-level causes are argument-independent and hold exactly: the absent `gates.toml` for `/constitute` and `/plan`, the missing `schema` key in `gates-build.toml`, the missing `tests_pass` kind in `gates-verify.toml`. The `/reflect` row is the one to treat as directional: `rf-02` and `rf-04` diverged under identical substitution, so the per-case attribution there may shift under the real parser even though the `DFA-E421` defect itself is confirmed.

| Skill | Command | Cases | Abort | Cause |
|---|---|---|---|---|
| designing-interfaces | `/design` | 8 | 0 | — |
| discovering-requirements | `/discover` | 6 | 0 | — |
| establishing-context | `/constitute` | 8 | **8** | `gate require constitute` → `DFA-E102 .devforgeai/gates.toml not found`; no case supplies a gates file |
| exploring-ideas | `/explore` | 8 | 0 | — |
| implementing-stories | `/build` | 8 | **7** (+1 cannot materialise) | `gate require build` → `DFA-E107 gates.toml: schema '' is unsupported`; `fixtures/gates-build.toml` has no `schema` key |
| improving-framework | `/reflect` | 7 | **5** | `report aggregate` → `DFA-E421 .sessions is outside the home directory C:\Users\bryan`, emitted whenever the `.sessions` directory does not exist |
| planning-work | `/plan` | 7 | **7** | `gate require plan` → `DFA-E102` (no gates file) **and** `doc load context all` → `DFA-E200 tech-stack.md not found` |
| releasing-software | `/release` | 9 | 0 | — |
| validating-quality | `/verify` | 6 | **6** | `gate require verify` → `DFA-E303 gates.toml gate 'build' omits required check kind 'tests_pass'`; `fixtures/gates-verify.toml` is short a required kind |

## 3. AskUserQuestion headless

| ID | Severity | Location | Guidance or spec says | What exists | Failure scenario |
|---|---|---|---|---|---|
| EVL-030 | high | all `cases.jsonl` with multi-line prompts; `specs/questions.md` Q-017 | §7 item 12: "The eval runner pre-seeds AskUserQuestion answers through a `PreToolUse` hook returning `allow` + `updatedInput.answers`, or runs with `--permission-prompts none`" | No hook, no `answers` key, no flag. Six cases append pseudo-answers as extra prompt lines (`"…\nPlain\nCool neutral\nOne sans\nComfortable\nWordmark"`) | Those lines are fiction: `AskUserQuestion` is never answered by prompt text. The one observed pilot passed only because the model *noticed the tool was unavailable* and asked in prose instead — a behaviour no case asserts and no grader depends on, except `asked_decision`, which passes on the prose. The whole inline-answer convention has to go |
| EVL-031 | medium | `make_workspace()` Decision 42 | Decision 42 writes `{}` "so no hook runs during an eval and the skill is measured alone" | Decision 42 and §7 item 12 are in direct conflict | The fix requires amending Decision 42 with one named exception: the workspace `settings.json` carries exactly one `PreToolUse` handler, matcher `AskUserQuestion`, and nothing else. Without the amendment the fix looks like a spec violation |
| EVL-032 | low | agent files under `agents/` | §2: "Tools removed from every subagent regardless of `tools`: … `AskUserQuestion` … **A subagent cannot ask the user a question.**" | — | The hook covers only main-conversation questions. If a skill routes a question into a subagent it is unanswerable in any mode, hook or not. Stated so the fix is not mistaken for full coverage |
| EVL-033 | medium | `claude_argv()` | §4: "**`--permission-prompts none`** (v2.1.259+) removes `AskUserQuestion` from the tool set so Claude cannot call it, denies anything that would prompt, and tells Claude not to retry" | Not used, and the runner never inspects the installed version | Measured: the installed `claude` is **2.1.268**, so the flag is available; `claude --help` confirms `--permission-prompts <target>` with choices `host`, `none`, default `host`. The runner must version-gate, because a machine on an older build silently drops the flag |

## 4. Case and grader conformance

Conformance checks that pass on all nine skills, verified mechanically over all 67 cases and all 61 grader functions:

- Every case id and every grader name in every `cases.jsonl` also appears in that skill's spec `## Evals` section. No drift.
- Every `FIXTURE:<name>` resolves to a file under `skills/<name>/evals/fixtures/`. Zero unresolved.
- Every `expect.grader` exists in the skill's `graders.py` with the exact §9 signature `(workspace, transcript, args)`. Zero missing, zero arity mismatch.
- Every `setup.extends` names an earlier case in the same file. No cycles.
- Imports are standard library only: `ast`, `hashlib`, `json`, `os`, `re`, plus `datetime` in two modules. No third-party import, no `subprocess`, no `socket`/`urllib`, no `random`, no `secrets`, no `uuid`.
- `datetime` is used only for deterministic parsing (`strptime("%Y-%m-%d")`, `date.fromisoformat`) and for an `as_of` value supplied by the case `args`. No `datetime.now()`, no wall-clock read, anywhere.
- Every `evals.json` matches the skill-creator schema: `skill_name` matching the skill directory, `evals[].id` an integer, `prompt`, `expected_output`, `files`, `expectations` with 3 to 6 entries each. Counts 6, 6, 7, 8, 6, 7, 6, 9, 6 — all at or above the §9 minimum of 6.

| ID | Severity | Location | Guidance or spec says | What exists | Failure scenario |
|---|---|---|---|---|---|
| EVL-040 | blocker | `skills/implementing-stories/evals/cases.jsonl` BLD-07 | `01-cli.md` §Evals step 3: "A path escaping the workspace stops the case with status `error`" | `"../wt/STORY-014/.git": "FIXTURE:wt-story-014-git.txt"` | Dry run reports `ERROR BLD-07  path escapes the workspace: ../wt/STORY-014/.git` and the skill exits 2. See EVL-050 for the fix |
| EVL-041 | blocker | `skills/planning-work/evals/graders.py` `_setup_path()`, used by `send_back` and `remedy_ac` | §9: graders read `workspace`; `01-cli.md` §Evals step 5: "the runner mirrors `setup.files` nowhere else" | Both graders read `<workspace>/.setup/<path>` and compare against the produced file. The runner never creates `.setup/`, and `.setup` appears in **zero** cases across all nine `cases.jsonl` | PLAN-05, PLAN-06 and PLAN-07 fail unconditionally with "`…` or its `.setup` copy is absent". Three of planning-work's seven cases — including both of its SEND BACK cases — can never pass |
| EVL-042 | medium | `skills/exploring-ideas/evals/cases.jsonl`, `skills/improving-framework/evals/cases.jsonl` | §9: "At least 2 exercise the SEND BACK path" | exploring-ideas has **0** send-back cases (ex-05/ex-06 are the *receiving* remedy leg). improving-framework has **0** | Neither skill meets the §9 floor as written. Both have a defensible reason — Explore is phase 0 with nothing upstream, Reflect sends back to nobody — but the rule as written fails them. Either amend §9 to exempt entry and terminal phases and say so per skill, or add cases (Explore's own Discover-remedy leg already exists; Reflect has no candidate at all) |
| EVL-043 | low | five grader functions; eight fixture files | §9 attaches the three artifacts to the skill; nothing forbids spares | Graders defined and used by no case: `personas_resolve`, `acceptance_recorded` (discovering-requirements), `no_ceremony` (establishing-context), `sprint_shape`, `design_called` (planning-work). Fixtures referenced by no case: `digests.txt` (×4 skills), `baselines.txt`, `requirements-remedy-design.yaml`, `requirements-remedy-plan.yaml` (both are loaded by graders via `_fixture()`, not by cases), `requirements-IDEA-021.yaml`, `requirements-IDEA-022.yaml` | Dead weight, or the tell that cases were dropped. `design_called` in particular is the only grader that checks the Plan→Design hand-off, and no case names it |
| EVL-044 | low | `skills/discovering-requirements/evals/graders.py` `_fixture()` | §9: "pure functions … No network, no subprocess to an LLM, no randomness" — it does not scope file reads | `here = os.path.dirname(os.path.abspath(__file__))` then reads `fixtures/<name>`, i.e. **outside the workspace**, by design and documented in the module docstring | The only grader module that reads outside `workspace`. It is deliberate and sound (there is no pre-run snapshot, so the baseline has to come from somewhere), but §9 does not sanction it. Write the exception into §9: a grader may read its own `evals/fixtures/` and nothing else outside the workspace |
| EVL-045 | medium | `skills/implementing-stories/evals/fixtures/gates-build.toml`, `skills/validating-quality/evals/fixtures/gates-verify.toml` | `01-cli.md`: `gates.toml` carries `schema = "devforgeai/gates/1"`; compiled minimums require `tests_pass` under `build` | `gates-build.toml` starts at `[[gate]]` with no `schema` key → `DFA-E107`. `gates-verify.toml` has the schema but its `build` gate omits `tests_pass` → `DFA-E303` | Root cause of 13 of the 33 preamble aborts in EVL-013 |
| EVL-046 | medium | `devforgeai report aggregate`; `skills/improving-framework/evals/fixtures/config-rust-reflect.toml` line 51 | — | `session_root = ".sessions"`. When that directory exists the command exits 0; when it does not, the command emits a complete aggregate with `sessions.status: "unreadable"` **and exits 1** with `DFA-E421 .sessions is outside the home directory C:\Users\bryan` | Two consequences. (a) A cross-area CLI defect: an absent relative session root is reported as "outside the home directory", so the absent case cannot be distinguished from a real escape — and it aborts the `/reflect` preamble on rf-01, rf-02, rf-05, rf-06, rf-07. (b) It makes `--workdir` load-bearing: the default system temp happens to sit under `C:\Users\bryan`, so rf-03 and rf-04 pass the check by accident. A `--workdir D:\evals` breaks those two as well |

### Dry run, all nine

Command: `python evals/runner/run_jsonl.py --skill skills/<name> --dry-run --workdir <scratch>`.

| Skill | Cases | Exit | Materialisation |
|---|---|---|---|
| designing-interfaces | 8 | 0 | all 8 clean |
| discovering-requirements | 6 | 0 | all 6 clean |
| establishing-context | 8 | 0 | all 8 clean |
| exploring-ideas | 8 | 0 | all 8 clean |
| implementing-stories | 8 | **2** | `ERROR BLD-07  path escapes the workspace: ../wt/STORY-014/.git`; other 7 clean |
| improving-framework | 7 | 0 | all 7 clean |
| planning-work | 7 | 0 | all 7 clean |
| releasing-software | 9 | 0 | all 9 clean |
| validating-quality | 6 | 0 | all 6 clean |

One failure, exactly the one EVL-040 predicts. No real evals were run.

## 5. Known open items

| ID | Severity | Location | Guidance or spec says | What exists | Failure scenario |
|---|---|---|---|---|---|
| EVL-050 | blocker | BLD-07; `specs/questions.md` Q-015 | `01-cli.md` §Evals step 3 refuses an escaping path | Q-015 leaves the choice open between rewriting the case and giving the runner a sandbox parent | See the fix specification: **rewrite the case**, do not change the workspace layout |
| EVL-051 | high | `invoke()` argv construction and `stdin=DEVNULL`; `specs/questions.md` Q-015 | The headless page documents `claude -p` reading the prompt from stdin when no positional prompt is given | Six cases carry embedded newlines (`dz-01`, `dz-02`, `dz-03`, `ex-02`, `ex-03`, `ex-04`). On Windows `claude` is an npm `.cmd` shim; `shutil.which` resolves it and `subprocess.run` spawns it through `cmd.exe`, which does not preserve embedded newlines in an argument | The multi-line prompt arrives truncated or mangled, so the case runs against a different prompt than the one recorded. Measured: **`--prompt-file` does not exist on `claude 2.1.268`** (`claude --help` has no such option), so the spec's preferred fix is unavailable. The stdin path is the fix |
| EVL-052 | low | `skills/*/evals/cases.jsonl` prompt contents | — | Once EVL-030's `answers` map lands, the six multi-line prompts collapse to single lines and EVL-051 stops being reachable through the current corpus | Fix EVL-051 anyway: the stdin path is also what makes a prompt with a quote, a backtick or a `%VAR%` safe under the `.cmd` shim |

## 6. skill-creator alignment

| ID | Severity | Location | Guidance or spec says | What exists | Failure scenario |
|---|---|---|---|---|---|
| EVL-060 | low | `skills/*/evals/evals.json` | skill-creator `references/schemas.md` `evals.json`: `skill_name`, `evals[].id` (unique integer), `prompt`, `expected_output`, `files` (optional, relative to skill root), `expectations` | All nine match exactly: `skill_name` equals the directory name; every `id` is an integer; every entry carries all five keys; every `expectations` list holds 3 to 6 statements. `files` is populated in two skills (`evals/fixtures/…`, which resolve) and an empty list in the other seven | Conformant. No change needed |
| EVL-061 | medium | skill-creator `scripts/run_eval.py` | — | `--eval-set` is read as `json.loads(path.read_text())` and then iterated as a **top-level array** of objects with `item["query"]` and `item["should_trigger"]`. Our `evals.json` is an object `{skill_name, evals:[…]}` with `prompt`, not `query`, and no `should_trigger` | `run_eval.py` **cannot consume our files unchanged**: iterating a dict yields its keys, and `item["query"]` on a string raises `TypeError: string indices must be integers`. A trigger-rate run needs a separate `trigger-set.json`: a flat array of `{query, should_trigger}` pairs, with should-trigger queries drawn from each skill's `description` use cases and should-not-trigger queries drawn from the neighbouring phases |
| EVL-062 | low | skill-creator `scripts/run_eval.py` `run_single_query()` | — | Uses `select.select([process.stdout], …)` on a subprocess pipe | `select` on a pipe is POSIX-only; on Windows `select` accepts sockets only. The plugin's trigger-rate script cannot run on this machine at all. Any trigger-rate plan has to target WSL or a Linux runner |
| EVL-063 | medium | all nine `SKILL.md` frontmatter | §7 item 8: "Phase skills that only the user should start get `disable-model-invocation: true`; skills another skill invokes (designing-interfaces from exploring-ideas) stay model-invocable" | No skill sets `disable-model-invocation`. No skill sets `allowed-tools` either (cross-reference EVL-011) | What a trigger-rate run would measure today: whether the description causes Claude to pick the skill up unprompted — for nine skills that are all meant to be started by the user typing a slash command. **If the skills audit adds `disable-model-invocation: true` to the eight user-started phase skills, their descriptions leave the always-on context entirely and trigger-rate testing is moot for them** — there is nothing left to trigger. Only `designing-interfaces` stays model-invocable, because `exploring-ideas` step 6 invokes it; it is the one skill where a trigger-rate number means anything. Plan for one trigger set, not nine |

## 7. Results and reporting

| ID | Severity | Location | Guidance or spec says | What exists | Failure scenario |
|---|---|---|---|---|---|
| EVL-070 | medium | `results.jsonl` line schema; skill-creator `grading.json` / `benchmark.json` | §3: "the skill-creator plugin stores `evals/evals.json`, runs isolated subagents, writes `grading.json` and `benchmark.json`" | `results.jsonl` carries one boolean and one evidence string per case | `results.jsonl` **cannot** produce either shape, and the gap is a different measurement, not a reshaping. `grading.json` needs `expectations[]` — one entry per `evals.json` expectation with its own `passed` and `evidence` — plus `execution_metrics.tool_calls` (per-tool counts), `total_steps`, `errors_encountered`, and `timing`. Our runner grades one function per case and records no per-tool counts. `benchmark.json` additionally needs `with_skill` and `without_skill` runs per eval with `runs_per_configuration` repeats; the runner never runs a case without the skill and never repeats one. Recommendation: **do not have the runner write those files.** Keep `results.jsonl` as the framework's own record, and add the fields below so a separate `to_grading.py` can be written later without another eval run |
| EVL-071 | medium | `run_case()` record | §4 names the JSON result fields | `total_cost_usd` is discarded | **Yes, record it** — the one observed run cost \$2.82, so a nine-skill suite is a two-figure sum and a budget question. Add in the same change: `num_turns`, `permission_denials` (the array, not a count — it names the denied tool), `session_id` (so a run can be resumed with `--resume`), `is_error`, and, once EVL-010 lands, `tool_calls` as a name→count map derived from the stream. Bump the schema to `devforgeai/eval-result/2` |
| EVL-072 | low | `summarise()` | `01-cli.md` §Evals gives the exact summary line and exit codes 0/1/2/3 | Matches the spec verbatim, including the per-failure line and the `results:` footer | Conformant. Once EVL-071 lands, add a cost total to the summary line; the spec's line has no slot for it, so amend the spec in the same change |

## Fix specifications

### EVL-002 — do not write the raw streams into the workspace

In `invoke()`, replace the two `open(os.path.join(workspace, ".dfa-claude-*.txt"))` writes with writes into a per-run log directory outside the workspace, and pass that directory in:

```python
def invoke(argv, workspace, home, timeout, prompt, log_dir, case_id):
    ...
    if log_dir:
        os.makedirs(log_dir, exist_ok=True)
        _write_text(os.path.join(log_dir, case_id + ".stdout.txt"), finished.stdout or "")
        _write_text(os.path.join(log_dir, case_id + ".stderr.txt"), finished.stderr or "")
```

`log_dir` is `<run root>/logs`, a sibling of the workspaces, never inside one. `run_case` passes `options.log_dir` and `case["id"]`. Add `"log_stdout"` and `"log_stderr"` paths to the results line so a failed case is still readable after the run. Amend `01-cli.md` §Evals step 7 to name the log directory, since the current text is silent about where the raw streams go.

### EVL-001 — redirect the Claude config directory into the workspace

`--bare` is not usable: it would drop the commands and agents the cases depend on, and every preamble with it. Redirect the config directory instead, beside the existing `DEVFORGEAI_HOME` redirect, in `invoke()`:

```python
    # Decision 62, extended: the eval reads no user-level configuration either,
    # so a grade does not depend on the machine it ran on.
    environment["CLAUDE_CONFIG_DIR"] = os.path.join(home, "claude-config")
    os.makedirs(environment["CLAUDE_CONFIG_DIR"], exist_ok=True)
```

`home` is already `<workspace>/.claude/dfa-home`, which every grader's workspace walk skips through the `.claude` baseline entry. Verify after the change that a run still authenticates: if credentials live under the redirected directory on this platform, copy only the credential file into the redirected directory and nothing else, and record in `01-cli.md` §Evals which single file is carried across and why. If the redirect proves to break authentication outright, fall back to asserting the isolation instead of enforcing it: at run start, compare the set of names under `~/.claude/agents/` and `~/.claude/skills/` against the framework's own and refuse to run, exit 3, on a collision — measured today as zero collisions across 46 framework agents and 46 user agents, so the guard costs nothing until something changes.

Amend `01-cli.md` §Evals step 5 to name both redirects.

### EVL-003 — state the enforcement gap in the spec

No code change. Add a paragraph to `01-cli.md` §Evals, after the Decision 42 sentence:

> Because the workspace `settings.json` registers no hook, an eval exercises no gate, no producer check, no Stop block and no `SubagentStop` ingest. The Python suite measures whether a skill's workflow produces the right documents and the right handoff text. Every enforcement behaviour in `specs/00-conventions.md` §7 is measured by the Rust suite alone. A skill that would be blocked by a hook in a real session still passes its evals.

and a matching sentence in `00-conventions.md` §9 so the boundary is stated where the three artifacts are mandated.

### EVL-005 — reduce the 12 substring graders

Two changes, neither of which removes the transcript check:

1. Land EVL-010 first. A `stream-json` transcript containing tool calls turns `expect_calls` from unsatisfiable into a real assertion, and `forbid_calls` from vacuous into a real one. That alone moves `blocked_line` and `criterion_set` out of the "passes on model prose" class.
2. For the seven handoff graders — `sendback_to_explore`, `establishing-context.send_back`, `planning-work.send_back`, `designing-interfaces.sendback_block`, `releasing-software.sendback_block`, `validating-quality.sendback_block`, `improving-framework.handoff_returns` — pair each transcript assertion with an artifact assertion that the same run must satisfy: the upstream document's SHA-256 unchanged (three already do this), the downstream document absent (two already do this), and the phase's report file either absent or carrying `result: SEND BACK`. A model that prints the handoff block without doing the work then fails on the artifact half.

The remaining two of the twelve, `figma_fallback` and `version_refused`, already carry an artifact half — a Figma table cell pair and a file-size floor in one, an absent-path and absent-directory sweep in the other. They are classed `substring` because a transcript literal is also a necessary condition, not because the artifact half is missing. Leave both as they are; landing EVL-010 is enough, since their transcript literals (`Gate      PASS`, `transcript_must_contain`) become assertions over real events rather than over prose.

`asked_decision` is the one grader with no artifact half available; replace it once the EVL-030 hook lands, by asserting that the hook's answer file was consumed — the hook writes `<workspace>/.claude/eval-answers-used.json` recording each matched header, which the grader reads. That makes it a record of a real `AskUserQuestion` call rather than a phrase in prose. This depends on `ex-07-timebox-spent` gaining an `answers` map in the EVL-030 change: without one, the runner passes `--permission-prompts none`, `AskUserQuestion` leaves the tool set, the hook never fires and no `eval-answers-used.json` is written. `ex-07` is listed in the EVL-030 case-corpus change for exactly this reason.

### EVL-010 + EVL-011 + EVL-012 + EVL-014 — the exact runner argv change

Replace `claude_argv()` and the call site in `run_case()`/`dry_run_case()`:

```python
ALLOWED_TOOLS = ("Bash(devforgeai:*),Read,Write,Edit,Glob,Grep,"
                 "Agent,Skill,AskUserQuestion,WebSearch,WebFetch,TodoWrite")


def claude_argv(binary, model, workspace, seeded, prompts_none):
    """The prompt travels on stdin, not in argv (EVL-051)."""
    argv = [binary, "-p",
            "--model", model,
            "--output-format", "stream-json", "--verbose",
            "--permission-mode", "acceptEdits",
            "--allowedTools", ALLOWED_TOOLS,
            "--add-dir", workspace]
    if prompts_none and not seeded:
        argv += ["--permission-prompts", "none"]
    return argv
```

- `--output-format stream-json --verbose` replaces `--output-format json` (EVL-010).
- `--allowedTools` is added (EVL-011). It is belt and braces beside the command files' own `allowed-tools`, and it is the only grant a `Skill`-tool invocation gets. `AskUserQuestion` stays in the list so the EVL-030 hook has a tool to intercept; it is removed for a case with no `answers` by `--permission-prompts none`.
- The positional prompt is gone; `invoke()` passes it on stdin (EVL-051).
- `--permission-prompts none` is appended only when the installed version allows it and the case seeds no answers (EVL-033).

`parse_args()` gains:

```python
parser.add_argument("--timeout", type=int, default=900)        # EVL-014
parser.add_argument("--devforgeai-bin", dest="devforgeai_bin", # EVL-012
                    default=None)
```

with `--devforgeai-bin` defaulting to `<framework root>/cli/target/release/devforgeai.exe` (`.../devforgeai` off Windows) and a usage error, exit 3, when the file is absent — the suite must refuse to start rather than produce 67 false negatives.

`invoke()` becomes:

```python
def _version_supports_prompts_none(binary):
    """True when the installed claude is 2.1.259 or newer (guidance section 4)."""
    try:
        out = subprocess.run([binary, "--version"], capture_output=True,
                             text=True, timeout=30).stdout
    except (OSError, subprocess.SubprocessError):
        return False
    match = re.search(r"(\d+)\.(\d+)\.(\d+)", out or "")
    return bool(match) and tuple(int(g) for g in match.groups()) >= (2, 1, 259)


def invoke(argv, workspace, home, timeout, prompt, devforgeai_bin):
    environment = dict(os.environ)
    environment["DEVFORGEAI_HOME"] = home
    # EVL-012: the command preambles call `devforgeai`; a non-zero exit from a
    # !`...` injection aborts the whole skill invocation.
    environment["PATH"] = (os.path.dirname(os.path.abspath(devforgeai_bin))
                           + os.pathsep + environment.get("PATH", ""))
    if environment.get("CLAUDECODE"):
        for key in list(environment):
            if key.startswith("CLAUDE_CODE_") or key in (
                    "CLAUDECODE", "CLAUDE_PID", "AI_AGENT", "CLAUDE_EFFORT",
                    "ANTHROPIC_API_KEY"):
                environment.pop(key, None)
    argv = [shutil.which(argv[0]) or argv[0]] + argv[1:]
    try:
        finished = subprocess.run(
            argv, cwd=workspace, env=environment,
            input=prompt,                       # EVL-051: prompt on stdin
            capture_output=True, timeout=timeout, text=True,
            encoding="utf-8", errors="replace")
    except subprocess.TimeoutExpired:
        return "timeout", "", None, {}
    except OSError as exc:
        raise CaseError("claude did not start: %s" % exc)
    transcript, meta = read_stream(finished.stdout or "")
    if finished.returncode != 0 or meta.get("is_error"):   # EVL-016
        return "error", transcript, finished.returncode, meta
    return "ran", transcript, finished.returncode, meta
```

and `transcript_of()` is replaced by a stream reader:

```python
def read_stream(stdout):
    """Assemble a transcript from stream-json events and pull the result meta.

    Tool calls are rendered so the graders' literals appear: an Agent call as
    ``Agent(<subagent_type> <prompt>)`` with ``)`` replaced by ``]`` inside the
    parentheses, because implementing-stories' criterion_set matches
    ``Agent\\(ac-test-writer[^)]*?(AC-\\d{3})`` and a bare ``)`` would end the
    match early. Every other call renders as ``<Name>(<compact JSON input>)``,
    which puts a Bash command string such as
    ``devforgeai worktree ensure STORY-015`` into the transcript verbatim.
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
                    name = block.get("name") or "?"
                    tools[name] = tools.get(name, 0) + 1
                    payload = block.get("input") or {}
                    if name == "Agent":
                        inner = "%s %s" % (payload.get("subagent_type") or "",
                                           payload.get("prompt") or "")
                        lines.append("Agent(%s)" % inner.replace(")", "]"))
                    else:
                        lines.append("%s(%s)" % (
                            name,
                            json.dumps(payload, ensure_ascii=False).replace(")", "]")))
        elif kind == "user":
            for block in (event.get("message") or {}).get("content") or []:
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
                    "subtype": event.get("subtype")}
    meta["tool_calls"] = tools
    return "\n".join(lines), meta
```

The `result` text is appended last, so every grader that reads the final handoff block with "the last `Phase     ` line" still finds it at the end.

`run_case()` copies `meta` into the record (EVL-015, EVL-071) and bumps `SCHEMA` to `devforgeai/eval-result/2`:

```python
record.update({k: meta.get(k) for k in (
    "session_id", "total_cost_usd", "num_turns", "permission_denials",
    "is_error", "tool_calls")})
```

Amend `01-cli.md` §Evals step 6 and 7 to the new argv and the stream reader, the argument table with `--devforgeai-bin` and the 900 s default, and the results line schema with the six new fields.

### EVL-013 + EVL-045 — make the preambles pass

Three fixture and case changes; no runner change.

1. `skills/implementing-stories/evals/fixtures/gates-build.toml` — prepend the two lines the parser requires:

```toml
schema = "devforgeai/gates/1"
cli_min_version = "1.0.0"
```

2. `skills/validating-quality/evals/fixtures/gates-verify.toml` — add the missing required kind to the `build` gate so `gate require verify` can read the predecessor gate. Take the `[[gate.check]]` block for `tests_pass` verbatim from the default file that `devforgeai init` writes.

3. `skills/establishing-context/evals/cases.jsonl` (8 cases) and `skills/planning-work/evals/cases.jsonl` (7 cases) — add `".devforgeai/gates.toml": "FIXTURE:gates-default.toml"` to the first case of each file and let the rest inherit through `setup.extends`, which neither file currently uses. planning-work additionally needs the six context files under `.devforgeai/context/` for `doc load context all`; four of the six fixtures already exist in that directory (`anti-patterns.md`, `architecture-constraints.md`, `source-tree.md`, plus a `tech-stack.md` to add), so add the two missing fixtures rather than inventing new content.

EVL-046 is a CLI defect and belongs to that area's fix: `report aggregate` must report an absent relative `session_root` as `sessions.status: "absent"` with exit 0, and reserve `DFA-E421` for a root that resolves outside the home after canonicalisation. Until that lands, the five reflect cases abort regardless of anything the eval layer does.

### EVL-030 + EVL-031 + EVL-033 — pre-seeded AskUserQuestion answers

**Case-line schema addition.** One optional top-level key per case line, beside `id`, `prompt`, `setup` and `expect`:

```json
{ "id": "ex-04-promote",
  "prompt": "/explore \"invoice chasing for dental practices\"",
  "answers": { "Decision": "Promote", "Prototype": "No" },
  "setup": { "files": { … } },
  "expect": { "grader": "carry_forward_exact", "args": { … } } }
```

- Keys are the `header` of the question the skill asks (preferred, because the headers are fixed in the skill templates), or the full `question` text when a skill asks two questions under one header.
- Values are the exact `label` of the option to choose, or an array of labels for a `multiSelect` question.
- `answers` absent or `{}` means the case should never reach a question; the runner then passes `--permission-prompts none` so any attempt is denied and the case fails visibly instead of hanging.
- The six multi-line prompts lose their trailing pseudo-answer lines, which move into `answers` (EVL-052).

**Case-corpus change that goes with it.** Nine cases gain an `answers` map: the six multi-line ones (`dz-01`, `dz-02`, `dz-03` with the five brand headers; `ex-02`, `ex-03`, `ex-04` with the Decision header), plus `ex-07-timebox-spent`, which asks the timebox decision question and carries no trailing answer lines today — its grader `asked_decision` is rewritten to read `eval-answers-used.json` (EVL-005) and therefore needs the hook to fire, which only happens when `answers` is non-empty. `ex-01` and `ex-08` reach no question on their path and stay without the key, so they run under `--permission-prompts none` and fail visibly if a skill change ever routes them into one. Every other skill's cases are audited the same way as part of this change: a case whose workflow path crosses a fixed question gets the map, a case whose path does not is left bare on purpose.

`load_cases()` validates the key: a value that is neither a string nor a list of strings is a usage error, exit 3.

**Runner change in `make_workspace()`.** Replace the `{}` settings write with:

```python
    answers = case.get("answers") or {}
    hook_script = os.path.join(claude_dir, "ask-user-question-hook.py")
    _write_text(hook_script, ASK_HOOK_SOURCE)
    _write_text(os.path.join(claude_dir, "eval-answers.json"),
                json.dumps(answers, ensure_ascii=False, indent=1))
    # Decision 42, amended: the one hook an eval registers is the
    # AskUserQuestion answer seed. No other event has a handler.
    settings = {}
    if answers:
        settings = {"hooks": {"PreToolUse": [{
            "matcher": "AskUserQuestion",
            "hooks": [{"type": "command",
                       "command": sys.executable,
                       "args": [hook_script],
                       "timeout": 20}]}]}}
    _write_text(os.path.join(claude_dir, "settings.json"),
                json.dumps(settings, indent=1))
```

Exec form (`command` plus `args`) is used because guidance §1 requires a real executable on Windows for exec form and warns that shell form risks a shell profile echoing into stdout and breaking JSON parsing. `sys.executable` is an absolute `python.exe` path, so no PATH lookup is involved.

**The hook script**, standard library only, written verbatim into each workspace:

```python
#!/usr/bin/env python3
"""PreToolUse handler for AskUserQuestion during a DevForgeAI eval.

Reads the hook payload on stdin, matches each question against the case's
answers map, and allows the call with an updatedInput that echoes `questions`
and adds `answers`. A question with no configured answer is denied, naming the
header, so the case fails visibly instead of stalling.

Contract: specs/ANTHROPIC-GUIDANCE.md section 4 (AskUserQuestion headless) and
section 1 (PreToolUse decisions live in hookSpecificOutput; updatedInput
replaces the whole input object).
"""

import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ANSWERS = os.path.join(HERE, "eval-answers.json")
USED = os.path.join(HERE, "eval-answers-used.json")


def emit(payload):
    sys.stdout.write(json.dumps(payload))
    sys.exit(0)


def allow(questions, answers):
    emit({"hookSpecificOutput": {
        "hookEventName": "PreToolUse",
        "permissionDecision": "allow",
        "updatedInput": {"questions": questions, "answers": answers}}})


def deny(reason):
    emit({"hookSpecificOutput": {
        "hookEventName": "PreToolUse",
        "permissionDecision": "deny",
        "permissionDecisionReason": reason}})


def load(path, fallback):
    try:
        with open(path, "r", encoding="utf-8") as handle:
            return json.load(handle)
    except (OSError, ValueError):
        return fallback


def key_of(question):
    """Match on header first, then on the question text."""
    return (str(question.get("header") or "").strip(),
            str(question.get("question") or "").strip())


def lookup(configured, header, text):
    lowered = {str(k).strip().lower(): v for k, v in configured.items()}
    for candidate in (header, text):
        if candidate and candidate.strip().lower() in lowered:
            return lowered[candidate.strip().lower()]
    # A question asked under a header the case spells differently still
    # resolves when one configured key is a substring of the question text.
    for k, v in lowered.items():
        if k and text and k in text.strip().lower():
            return v
    return None


def labels_of(question):
    out = []
    for option in question.get("options") or []:
        if isinstance(option, dict) and option.get("label") is not None:
            out.append(str(option["label"]))
        elif isinstance(option, str):
            out.append(option)
    return out


def main():
    try:
        payload = json.load(sys.stdin)
    except ValueError as exc:
        deny("eval hook: the payload on stdin is not JSON: %s" % exc)
    tool_input = payload.get("tool_input") or {}
    questions = tool_input.get("questions") or []
    if not isinstance(questions, list) or not questions:
        deny("eval hook: AskUserQuestion carried no questions list")

    configured = load(ANSWERS, {})
    if not isinstance(configured, dict) or not configured:
        deny("eval hook: this case configures no answers; "
             "AskUserQuestion is not available in a headless eval")

    answers, matched = {}, []
    for question in questions:
        if not isinstance(question, dict):
            deny("eval hook: a questions[] entry is not an object")
        header, text = key_of(question)
        chosen = lookup(configured, header, text)
        if chosen is None:
            deny("eval hook: no answer is configured for question header %r "
                 "(question %r); add it to the case's answers map" % (header, text))
        allowed = labels_of(question)
        wanted = chosen if isinstance(chosen, list) else [chosen]
        if question.get("multiSelect") is not True and len(wanted) != 1:
            deny("eval hook: header %r is single-select and the case "
                 "configures %d labels" % (header, len(wanted)))
        unknown = [one for one in wanted if allowed and str(one) not in allowed]
        if unknown:
            deny("eval hook: header %r configures label(s) %s, and the options "
                 "offered are %s" % (header, unknown, allowed))
        answers[header or text] = chosen
        matched.append({"header": header, "question": text, "answer": chosen})

    try:
        with open(USED, "w", encoding="utf-8") as handle:
            json.dump(load(USED, []) + matched, handle, ensure_ascii=False)
    except OSError:
        pass

    allow(questions, answers)


if __name__ == "__main__":
    main()
```

`updatedInput` replaces the whole input object, so `questions` is echoed verbatim and `answers` is added beside it — guidance §1 is explicit that a partial `updatedInput` would drop the rest. The decision sits in `hookSpecificOutput`, not in the deprecated top-level `decision`/`reason`. The `eval-answers-used.json` file is what EVL-005 has `asked_decision` read, so a question that was really asked can be told from a phrase the model typed.

**Version gate and the no-answers path.** In `parse_args()`:

```python
    options.prompts_none = _version_supports_prompts_none(options.claude_bin)
```

and in `claude_argv()`, `--permission-prompts none` is appended when `options.prompts_none` is true and the case seeds no answers. On a build older than 2.1.259 the flag is dropped and the hook's own "this case configures no answers" deny is the backstop — the hook is registered for every case, and its empty-map branch denies, so a question in a no-answers case fails visibly on any version.

**Spec amendments.** `01-cli.md` §Evals step 5 loses "containing `{}`" and gains the one-exception wording; the per-case procedure gains a step for writing `eval-answers.json` and the hook script; the argument table is unchanged. `00-conventions.md` §9 item 2 gains `answers` in the `cases.jsonl` object shape. Decision 42 in whichever spec carries it is amended to name the single permitted handler.

### EVL-041 — planning-work's `.setup` baseline

The two graders need a byte-for-byte copy of the case's input to prove an upstream file was not edited. Seven other graders in the framework solve the same problem two ways that already work: a recorded SHA-256 in `args` (`establishing-context.send_back`'s `upstream_sha256`, `releasing-software.resume_untouched`'s `baseline_sha256`, `validating-quality`'s `_digests_hold`), or a fixture read beside the grader module (`discovering-requirements._fixture`). Use the digest form, because it needs no new runner behaviour and no new file in the workspace.

Delete `_setup_path()` and rewrite the two call sites:

```python
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
```

In `send_back`, `args.upstream_unchanged` changes from a list of paths to a map of path to SHA-256, matching `establishing-context.send_back`'s `upstream_sha256` exactly, so the two skills read the same way.

`remedy_ac` compares the produced story and sprint against their *starting* content, not just their digests, so a digest alone is not enough there. Give it the same treatment `discovering-requirements` uses: move the starting `STORY-104.md` and `sprint.yaml` into `skills/planning-work/evals/fixtures/` — they are already fixtures (`story-104.md`, `sprint-001.yaml`, named by PLAN-07's `setup.files`) — and read them with a `_fixture()` helper copied from `discovering-requirements/evals/graders.py`, with `args.baseline_story` and `args.baseline_sprint` naming the fixture files. PLAN-07's `setup.files` already writes exactly those fixtures, so the baseline is the same bytes by construction and no new fixture is authored.

Add the exception to `00-conventions.md` §9 in the same change as EVL-044: a grader may read files under its own `evals/fixtures/` directory, and nothing else outside the workspace. `specs/05-plan.md` `## Evals` carries both grader bodies and is amended to match.

### EVL-040 + EVL-050 — BLD-07

**Rewrite the case. Do not give the runner a sandbox parent directory.**

Reasons, in order of weight:

1. A sandbox parent weakens the one anti-gaming property that is currently airtight. `_inside(workspace, target)` is what keeps `setup.files` from reaching the framework tree; relaxing it to "inside the sandbox parent" means every case can write to a directory shared with every other case's workspace, and `--jobs > 1` turns that into cross-case contamination. EVL-006 argues for tightening that boundary, not loosening it.
2. The case does not need a real worktree. BLD-07's grader is `blocked_line`, and its `args` are `code: "DFA-E272"`, three names, `forbid_calls: ["devforgeai phase set build"]`, `expect_calls: ["devforgeai worktree ensure STORY-015"]`. Nothing in the grader reads `../wt`. The fixture exists only so that `devforgeai worktree ensure STORY-015` finds an overlapping worktree and returns `DFA-E272`.
3. The same overlap is observable from inside the workspace. `devforgeai worktree list --json` reports the registered worktrees, and `.devforgeai/state.toml` carries the seeded worktree entries that `worktree ensure` writes.

The rewrite: drop the `"../wt/STORY-014/.git"` entry, and seed the overlap where the binary already looks for it — the `state.toml` worktree table plus a workspace-internal worktree root. Change `setup.files` to add

```json
".devforgeai/state.toml": "FIXTURE:state-build-story-014-worktree.toml",
"wt/STORY-014/.git": "FIXTURE:wt-story-014-git.txt"
```

(the existing fixture, now at `wt/` inside the workspace rather than `../wt/`), and set `.devforgeai/config.toml`'s worktree root to `wt` in the `config-build.toml` fixture. Then change BLD-02's `args.worktree_root` from `"../wt"` to `"wt"` and add `"wt"` to its `args.baseline`, which also removes the shared-parent walk EVL-006 describes. Finally add to BLD-07's `args.expect_calls` the literal `devforgeai worktree list`, so the case asserts the model looked before it wrote — which it can only satisfy once EVL-010 puts tool calls in the transcript.

`specs/06-build.md` `## Evals` carries BLD-07 verbatim and is amended in the same change; `specs/questions.md` Q-015 is closed with this decision.

### EVL-051 — multi-line prompts through the npm `.cmd` shim

Measured: `claude 2.1.268` has no `--prompt-file`. The fix is stdin, which the headless page documents and which the shim passes through unaltered.

Covered by the `invoke()` rewrite above: the prompt leaves argv, `stdin=subprocess.DEVNULL` becomes `input=prompt`, and `claude -p` with no positional prompt reads it. Two consequences to record in the spec:

- `dry_run_case()` must print the prompt separately rather than inside the would-run line, since it is no longer an argument. Print `  prompt (<n> bytes, <m> lines) on stdin` followed by the prompt indented, so a dry run still shows exactly what the case sends.
- The `--replay-user-messages` and `--input-format stream-json` options are *not* wanted: plain text on stdin is what `-p` expects, and `--input-format stream-json` would require a framed message object.

Amend `01-cli.md` §Evals step 6 from "Invoke `<claude-bin> -p <case prompt> …` with the working directory set to the workspace, stdin closed" to "… with the working directory set to the workspace and the case prompt written to stdin, because on Windows `claude` is an npm `.cmd` shim and an argument carrying newlines does not survive `cmd.exe`."

### EVL-061 — a trigger set, separate from evals.json

Add `skills/<name>/evals/trigger-set.json`, a flat array in the shape `scripts/run_eval.py` actually reads:

```json
[ {"query": "we need to figure out who the users are before we plan anything",
   "should_trigger": true},
  {"query": "write the stories for EPIC-002", "should_trigger": false} ]
```

Six to ten queries per skill, half of them drawn from the neighbouring phases so a description that over-triggers is caught. Run it only for `designing-interfaces` until the skills audit settles `disable-model-invocation` (EVL-063), and run it under WSL, because `run_eval.py` calls `select.select()` on a pipe (EVL-062). Do not change `evals.json`: it matches the plugin's schema exactly and the grader agent reads it as-is.
