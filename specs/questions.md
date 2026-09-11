# Open questions for Bryan

Logged by the orchestrator while you were away. Each entry states the ambiguity, the assumption I proceeded under, and what changes if you decide otherwise. Answer inline under each entry, or mark it "agreed".

Format: `Q-nnn` · phase or component · status (open / assumed / answered)

---

## Q-001 · orchestration · assumed

**Ambiguity.** You asked for AskUserQuestion on any ambiguity, then went to bed. Blocking on a question would stall all three waves and the build.

**Assumption.** I proceed under stated assumptions and log each one here. Nothing is committed to git. Every artifact is a file you can delete.

**If you decide otherwise.** Say "stop and ask" and future sessions will block on questions instead.

---

## Q-002 · Explore · answered

**Ambiguity.** The Explore spec needs a CLI subcommand not in conventions §4: `devforgeai explore prune --id IDEA-nnn` (deletes `.explore-prototype/` on kill or promote, leaves it on park).

**Answer.** Accepted and built. `explore prune` is a CLI subcommand, which is where conventions §1 rule 1 places a destructive filesystem operation.

**If you decide otherwise.** The skill would have to run `rm -rf` itself, which puts a destructive filesystem rule in prose, against rule 1.

---

## Q-003 · Explore · assumed

**Ambiguity.** Time box length was never stated.

**Assumption.** 5 calendar days for a fresh run, 1 calendar day for a remedy run, both in `config.toml` under `[explore]`, enforced by a gate check kind `elapsed_days_at_most` on the Stop hook. Calendar days, not working days.

**If you decide otherwise.** Change the two numbers in config.toml defaults in the CLI spec. Nothing else moves.

---

## Q-004 · CLI · assumed

**Ambiguity.** Skill specs propose gate check kinds (Explore proposes nine, two of which parse markdown tables). The CLI spec author defines the closed enum without seeing them.

**Assumption.** After wave 1 lands I send every proposed kind to the CLI spec author for reconciliation, and the CLI spec's enum becomes authoritative. Kinds the CLI refuses fall back per the drop order each skill spec records.

**If you decide otherwise.** Say which kinds you reject and I re-run the affected skill spec.

---

## Q-005 · conventions · answered

**Ambiguity.** Slash-command names were not fixed in the conventions, so parallel spec authors could name them differently (Explore wrote `/create-context` once).

**Answer.** One entry point per skill, named after the phase: /explore /discover /constitute /plan /build /verify /release /design /reflect. Conventions §4b. Older names in your agents (`/dev`, `/qa`, `/ideate`, `/create-context`, `/create-story`) are not carried forward.

Since the command files collapsed into the skills, those nine names are each a skill's frontmatter `name`, and one field now decides three things: the slash command in a target project, the Skill-tool target one skill uses to reach another, and `cli/src/aggregate.rs` `SLASH_COMMANDS`, which Reflect reads session history with. A rename moves all three at once.

**If you decide otherwise.** Give the names you want; nothing structural depends on them, and the change is the nine `name:` values plus the `UserPromptExpansion` matcher in `hooks/settings.hooks.json`.

---

## Q-006 · CLI · assumed

**Ambiguity.** Skill specs proposed subcommands beyond conventions §4. All are mechanical operations that rule 1 forbids leaving in prose.

**Assumption.** Accepted into the CLI surface during reconciliation:
- `devforgeai explore prune --id IDEA-nnn` (Explore): delete `.explore-prototype/` on kill or promote.
- `devforgeai doc accept requirements --id IDEA-nnn` (Discover): stamp `accepted_by`/`accepted_at`, flip status. The model never edits those fields.
- `devforgeai doc reopen requirements --id IDEA-nnn --ids REQ-...,REQ-... --from plan|constitute|design` (Discover): re-open only the cited ids, bump `revision`, keep every other byte identical.
- `devforgeai doc load discover-entry "<arg>"` (Discover): prints the brief, or requirements.yaml, or nothing with exit 0, depending on what `$1` is, so the command preamble stays one silent line.
- `devforgeai report ingest <subagent> <source>` (Constitute; already in §7 hooks, missing from §4 table). One correction since: the second argument is the subagent's `last_assistant_message` from the `SubagentStop` payload, not its stdout — the event carries no stdout field. The dispatcher passes it on stdin as `-`.
- `devforgeai design lint --tokens` (Design): validate tokens.json itself, not only frontend files.
- Gate check kinds proposed by Explore (9), Discover (5), Constitute (2 grammar forms): folded into the CLI's closed enum; the CLI spec's enum is authoritative after reconciliation.

**If you decide otherwise.** Name the subcommand to reject and I re-run the owning skill spec with that constraint.

---

## Q-007 · Discover · answered

**Ambiguity.** MoSCoW priority enum (`must/should/could/wont`) tripped the ceremony regex because `must` was a matched token, even as a data value.

**Answer.** MoSCoW is restored. The rename was a workaround for a lint defect rather than a design judgment. Conventions §2 now scopes the pattern to instruction prose through six clauses, and clause 2 makes a YAML scalar out of scope, so `priority: must` matches nothing. The four words are also the vocabulary a stakeholder reading `requirements.yaml` already knows, and the rename had lost the distinction the third and fourth levels carry — `could` is a nice-to-have while `excluded` reads as a prohibition rather than as out-of-scope-for-now.

**Where it landed.** `skills/discovering-requirements/templates/requirements.yaml`, every eval fixture and inline `setup.files` copy across four skills, `specs/03-discover.md` `## Outputs` and its Decision 7, and the restatement in `specs/05-plan.md`. Two pinned digests moved with it, recorded in `skills/establishing-context/evals/fixtures/digests.txt`.

---

## Q-008 · pipeline · assumed

**Ambiguity.** Which id the pipeline carries between phases. Discover keyed on IDEA-nnn; Constitute assumed EPIC-nnn.

**Assumption.** Explore, Discover, and Constitute key on IDEA-nnn (one requirements.yaml per project, holding all epics; context files are project singletons). Plan keys on EPIC-nnn. Build and Verify key on STORY-nnn. Release keys on a version. A project that skipped Explore still gets an IDEA-nnn allocated by Discover.

**If you decide otherwise.** This one is structural; say so before wave 3 and I re-key Constitute and Plan.

---

## Q-009 · Release · assumed

**Ambiguity.** Which deployment platforms v1 supports, and where docs live.

**Assumption.** Platform enum closed at five: `kubernetes`, `compose`, `github-actions`, `vps`, `none` (library). Detected by marker files in order, then one AskUserQuestion, answer written to `config.toml [release].platform`. The GitHub Actions gate workflow (`devforgeai-release.yml`, runs `gate check --phase release`) is always written, separate from a `github-actions` deploy target. Docs are four fixed directories: API reference (one page per stack and source root), user guide (one H3 per story), architecture note (from ADRs), release notes.

**If you decide otherwise.** Add or remove platforms in the enum and the marker list; nothing else depends on it.

---

## Q-010 · CLI · assumed

**Ambiguity.** After the first reconciliation round the CLI check-kind enum closed at 21. Plan, Reflect, and Release then proposed more: `report_metric` usage (Plan), `yaml_cites` and `no_threshold_decrease` (Reflect), `release_stories`, `deploy_manifest`, `docs_cover` (Release), plus new subcommands `story files --check`, `story list`, `report aggregate`, `worktree`, `commit` (Plan, Release, Reflect, Build).

**Assumption.** A second reconciliation round folds them in after Build and Verify land. The enum grows; nothing already accepted changes name again.

**If you decide otherwise.** Reject specific kinds and the owning spec records a fallback using existing kinds.

---

## Q-011 · Verify · assumed

**Ambiguity.** Whether Treelint (an AST search tool your existing agents reference) is a framework dependency.

**Assumption.** No. It is not named in any skill. Its capability becomes two optional `config.toml [verify]` commands (complexity, duplication) with a Grep-based fallback when absent. Same rule for every external scanner: config.toml names it, the skill does not.

**If you decide otherwise.** Add Treelint to `stack detect`'s tool detection and the fallback goes away.

---

## Q-012 · Verify · assumed

**Ambiguity.** Whether a story reaches a `verified` status. Plan defined the enum draft/ready/building/built/verified/released; Verify found no phase writes `verified` (phase set verify writes `built`).

**Assumption.** Resolved: `verified` is dropped. The enum is `draft | ready | building | built | released`; `phase set release` writes `built → released`. Plan, Verify, and the CLI spec agree.

**If you decide otherwise.** Say which enum you want; it is a one-line change in three specs.

---

## Q-013 · Design · answered

**Ambiguity.** The Design spec named the built-in Claude Code skills bare (`figma-use`, `frontend-design`). They resolve only under their plugin namespace (`figma:figma-use`, `frontend-design:frontend-design`); a first-party skill resolves bare.

**Answer.** The namespaced names are correct and are what ships. Two things the guidance added since:

- A subagent's tool set is its `tools` list plus its own `mcpServers` and nothing else. A `Skill` grant lets an agent load a plugin's skill and still leaves it without that plugin's MCP tools, so a Figma call cannot be delegated. `specs/08-design.md` workflow step 13 therefore runs `figma:figma-design-to-code` in the main conversation and passes what it returns to `ui-spec-writer` as a `figma_context` field, `""` when there is no node URL, no plugin, or an authentication error.
- `skills:` in agent frontmatter preloads a skill's content without spending a turn, which is the cheap way to give an agent standing material — and still gives it no tools.

Both plugin names are sanctioned proper nouns under conventions §1 rule 2: a first-party Claude Code capability is addressed by the name the harness gives it.

---

## Q-014 · evals · answered

**Ambiguity.** The CLI spec had the eval runner copy a skill's whole directory into each test workspace, including `evals/` with the graders and fixtures, so the model under test could read the graders and shape its output to pass.

**Answer.** `evals/` is excluded from the runner's copy, and graders stay outside the workspace.

The same reasoning has a second half the install model made visible: `devforgeai init` copied `skills/` wholesale into a target project, so every installed project carried nine `evals/` trees — `graders.py`, `cases.jsonl`, and the whole fixture corpus — committed into that project's history, where a model working there could read the expected outputs. The eval workspaces were protected and the real projects were not. `init` now skips every directory named `evals` below the top of each copied tree. A target project runs no eval and holds no runner, so it loses nothing.

---

## Q-015 · evals · answered

**Ambiguity.** Two eval cases were unrunnable as written, and a third problem appeared on the first real run.

**Answer, all three closed in the runner rather than in the cases' subject matter.**

- BLD-07 no longer writes an escaping path. A case declares `setup.git`, either `true` for a plain work tree with one commit or `{"worktrees": [{"path": "wt/STORY-014", "branch": "devforgeai/STORY-014"}]}` to register worktrees inside the workspace up front. The case keeps its subject — a Build run that expects a worktree — and the runner stops refusing it.
- The newline problem is gone: the prompt travels on stdin rather than in argv, so an npm `.cmd` shim never parses it and a multi-line prompt survives on any shell.
- The nested-session hang is fixed by stripping `CLAUDECODE`, `CLAUDE_PID`, `AI_AGENT`, `CLAUDE_EFFORT`, and `ANTHROPIC_API_KEY` from the child environment. From a plain terminal nothing is stripped.

**Where you run evals.** From this repository. `--skill skills/<name>` is a path here; a target project holds `.claude/skills/<name>/` with no `evals/` and no runner. Hooks-on additionally needs the pin of Q-016.

---

## Q-016 · trust · needs you

**Ambiguity.** None. This is an action only you can take.

**What.** The binary refuses `trust pin` inside any Claude session by design (it checks `CLAUDECODE` and `CLAUDE_CODE_ENTRYPOINT`). `cli/REVISION` and `cli/DIGEST` are written for the current release build. To arm the hooks, open a plain terminal (not Claude Code) and run:

```
cd C:\Projects\DevForgeAI
cli\target\release\devforgeai.exe trust pin --framework C:\Projects\DevForgeAI
```

That writes `~/.devforgeai/trust.toml`. Until then every hook exits with `DFA-E501` (no trust file), which is the fail-closed path: nothing passes a gate on an unpinned binary.

**Also.** `REVISION` line 1 reads `unversioned` because the repo has no git yet. After `git init` and a first commit, rewrite both files with the binary's own hidden subcommand so line 1 carries the commit SHA:

```
devforgeai trust digest --framework C:\Projects\DevForgeAI
```

It prints `REVISION`, `SOURCE`, and `DIGEST`, which are `cli/REVISION` line 1, `cli/REVISION` line 2, and `cli/DIGEST`. Use it rather than any script: the walk it performs is the one `trust verify` reads the files back with, and a separate implementation drifts the first time an exclusion changes, surfacing only as `DFA-E504` refusing every write.

**Two corrections since this was written.** The unpinned state is now genuinely fail-closed rather than mostly silent. Three registrations enforce it: a `PreToolUse` `trust-check` arm denies every `Write`, `Edit`, `NotebookEdit`, `Bash`, `PowerShell`, and `Agent` call; `UserPromptExpansion` refuses each of the nine phase commands before the skill body loads; and the Stop hook blocks with the pin command and runs no gate. All three carry the same sentence naming the command above. `init` also runs `trust verify` at the end and prints the pin command as its `Next` line, in place of `/explore`, when it fails.

The second correction is that the pin is what hooks-on evals depend on. A release binary reads `~/.devforgeai/trust.toml` and honours `DEVFORGEAI_HOME` only under the `test-home` cargo feature, which a release build does not carry, so no redirect can stand in for the pin. `evals/runner/run_jsonl.py` runs `devforgeai trust verify` once before a suite and falls back to `--no-hooks` with the pin command printed when it fails; passing `--hooks` explicitly makes it refuse to start instead. Until you run the command above, every eval measures a skill with the framework's hooks off.

---

## Q-017 · evals · answered

**Ambiguity.** Headless evals versus questions to the user. Several workflows ask the user a fixed question mid-run (Explore: who holds the problem? build a prototype? kill, park, or promote?; Discover: three elicitation rounds; Design: five brand questions). A case whose path crosses one either stalled or ended early.

**Answer.** Option (a), through the supported mechanism rather than through extra prompt lines, plus one precondition the documentation leaves implicit.

The mechanism: a case carries an `answers` map from a question's fixed header to the label to choose. The runner writes it into the workspace and registers `evals/runner/answer_hook.py` as a `PreToolUse` handler on `AskUserQuestion` ahead of every other handler. The handler returns `permissionDecision: "allow"` with an `updatedInput` that echoes `questions` and adds `answers`, so the tool runs with no prompt and the model takes the case's answer through the same code path it takes a user's. `@first`, `@last`, `@n`, and `@all` select by position, and `"*"` answers any question the map does not name. No skill changes: the fixed headers already in the question templates are what the handler keys on.

The precondition, measured on `claude 2.1.268` with the same argv apart from the host flags: under `claude -p` the `AskUserQuestion` tool is in the tool set only when a permission host is supplied. With no host the run lists 33 tools and no `AskUserQuestion`, which is what the earlier pilot saw when the model asked in prose instead. With `--mcp-config <workspace>/.claude/mcp-permissions.json --permission-prompt-tool mcp__dfa-permissions__approve` the run lists 36 tools including it, and `mcp_servers` reports `dfa-permissions: connected`. `--permission-prompts host` alone does not add the tool, and `--allowedTools` permits without adding. The runner therefore ships `evals/runner/permission_host.py`, a stdio MCP server whose one tool, `approve`, allows every request unchanged, and names it for any case that seeds answers; a case that seeds none runs under `--permission-prompts none` so a question it was never meant to reach is denied rather than left hanging. With the host in place the pre-seed hook fires live and a `@first` resolves against the labels the run produced.

**Cost, for the timeout.** A measured full-phase case ran 828 seconds at $2.85 on sonnet, which is why `--timeout` defaults to 900.

---

## Q-018 · CLI · answered

**Ambiguity.** `AUDIT-1` CLI-217 asks for a refusing git hook to reach the `--json` envelope's `errors[]` with a code, rather than surviving only in `data.hook_stderr`. The spec's error table and `cli/src/errors_table.rs` are held in a two-way bijection by `cli/tests/errors_table.rs`, so a spec row exists when and only when a code row does.

**Answer, with the gap named.** No `DFA-` code covers "a git hook refused the commit" today. `cli/src/cmd/commit.rs` emits `DFA-E011`, `DFA-E012`, `DFA-E239`, and `DFA-E900`, and returns `Ok` with `exit = Some(1)` on a hook refusal, so the envelope reads `ok: false` with an empty `errors[]`. The spec table therefore carries no row for it either: adding one alone would break the bijection in the other direction.

**What it costs.** A caller reading `--json` sees a failure with no code to branch on, and the hook's own text only under `data.hook_stderr`. **What closes it.** One row in the `DFA-E2xx` band the worktree and commit codes already use — `DFA-E274` is the next free number — added to `cli/src/errors_table.rs` and to `specs/01-cli.md` `## CLI calls` in the same change, with `cmd/commit.rs` raising it. That is a code change and belongs to the CLI fixer, not to this pass.

---

## Q-019 · CLI · answered

**Ambiguity.** `AUDIT-4` AGT-029 asked what `verifier_pass` does when a verifier reports `total: 0`. Three places in `specs/01-cli.md` disagreed: two said the ratio is `1.0` and one said `DFA-E317` reporting `0/0`.

**Answer.** The ratio is `1.0` and the check passes, which is what `cli/src/gate.rs::verifier_pass` does. A verifier with no unit to count is not a verifier that failed. All three places now say so.

The related tightening is worth recording beside it: `passed` and `total` are read as **present integers**. A missing key, or one holding anything but an integer, is `DFA-E317`. Defaulting a missing key to `0` made a block that counted nothing read as the free pass that belongs to a verifier which genuinely counted zero units, which is the failure this question was really about.

---

## Q-020 · agents · answered

**Ambiguity.** `AUDIT-4` AGT-041: conventions §2 forbids a subagent from running tests, coverage, or linters and interpreting the numbers, while `specs/07-verify.md` places exactly that reading in `code-quality-auditor` and `dead-code-detector`, each of which runs one configured command.

**Answer, stated in both places.** An agent may read a tool's numeric output **only to copy it into its finding and its `payload` verbatim** — `measured` and `limit` on a complexity finding, `payload.dead` on a dead-code block, `payload.method` saying which path produced the number. Comparing that number against a ceiling or a floor is a `gate check` check kind reading `config.toml` and `gates.toml`. The reading is the agent's and the judgment is the gate's, which is what §2 means and what the two agents now do.

The permission that lets them run a command at all is a hook, not a tool grant: each holds an unscoped `Bash`, and the `PreToolUse` shell arm reads the payload's `agent_type` and admits exactly the `[verify].metrics_command` or `[verify].call_graph_command` string, denying anything else with the `config.toml` key to edit. A `Bash(<tool>:*)` scope would name a tool in an agent file, which §1 rule 2 forbids.

---

## Q-021 · conventions · answered

**Ambiguity.** Whether the source tree's skill directories should be renamed to the slash names, since Claude Code derives a project skill's command from its **directory** name and treats frontmatter `name` as a display label.

**Answer.** The source tree keeps the long directory names — `skills/exploring-ideas/`, `skills/implementing-stories/` — and `devforgeai init` installs each skill at `.claude/skills/<frontmatter name>/`, so `/build` is the command in a target project. The eval runner builds its workspaces the same way, which is why a case prompt of `/build STORY-014` resolves there.

**Why not rename the source directories.** `produced_by: implementing-stories` in every template and every document resolves against the directory name, and the `PreToolUse` producer check reads that value; a rename moves over 1200 references across specs, templates, fixtures, and pinned digests. **If you decide otherwise**, say so and the rename is one mechanical pass — it is deferred rather than declined.

---

## Q-022 · evals · answered

**Ambiguity.** What the first end-to-end acceptance run under `claude -p` actually established, and whether the failures it produced are skill defects or eval-layer defects.

**Answer.** Nine cases ran, one per phase. Two passed end to end: Explore `ex-01` and Design `dz-04`. The other seven failed on eval-layer preconditions rather than on the behaviour each case exists to measure — a fixture missing a `state.toml` or `config.toml` field the current binary reads, a fixture missing the predecessor report the preamble's `gate require` needs, a headless permission denial on a compound shell command, and `phase set plan` refusing without the `--epic` the skill now passes. All four classes are routed and being fixed: the fixtures at the eval layer, the permission denial by moving the runner to `--permission-mode bypassPermissions` inside its throwaway workspace, and `--epic` in `cli/src/cmd/phase.rs` with `specs/01-cli.md` and `specs/05-plan.md` written to match.

One failure was not an eval-layer defect. The Reflect case's recommendations did not follow `templates/rec-targets.md`: the target-kind rows were applied without their precedence rule, so a change to `.devforgeai/gates.toml` was drafted as `framework_file` rather than `gate_threshold`. That is skill adherence, and the fix is in the template and in `specs/10-reflect.md` — the rows now say they are ordered by specificity, that a target takes the first row it matches, and that `framework_file` names neither of the two configuration files.

**Where it stands after the fixtures and the preflight pass.** Four phases now pass end to end headless: Explore, Design, Reflect, and Plan.

Two complete the workflow and fail on content rather than on plumbing. Discover writes an epic whose `success_metric` is not a number, so the metric a gate would read is a sentence; that is a skill defect and it is the kind the suite exists to find. Verify's case asserts a `Next` line, which only the Stop hook renders — with no trust pin the hooks do not run, so the assertion cannot be measured at all rather than measured and failed. It stays failing until Q-016 is done, and the case is a hooks-on case whichever way it lands.

Two are still blocked on something other than the skill. Build waits on a fixture, and Release waits on a CLI fix.

**The first hooks-on run, with a real pin.** `ex-01` under `claude -p`, with `trust.toml` written by a human outside Claude Code. Two things were measured that no hooks-off run could show.

The Stop hook ran the Explore gate, blocked the turn three times, and then exited 0 with the FAIL handoff in `systemMessage`. That is the per-session budget of §7 working live, and it confirms the key: the budget is `session_id` and nothing else, so a continuation that changes the subject does not reset it.

Then `trust verify` failed mid-run with `DFA-E504`, the source-digest drift. The cause is not a defect: a fixer was editing `cli/src` while the session was open, and `trust verify` compares the source digest whenever a Claude session is active. Editing the framework's own source while any session is running invalidates trust for every session until a human re-pins, by design — that is what the check is for, and the framework is the one project where its own source is also the code under edit. Working on `cli/` means either no session open, or a re-pin before the next one.

What the run did surface is a defect: the trust-failure branch of the Stop hook kept blocking past three, because it carried its own exit condition rather than sharing the gate branch's counter. Six blocks between the two branches reaches the harness's eight-block ceiling, which shows the user nothing. F2 is folding the trust branch into the shared three-block budget, and `specs/01-cli.md` `## Hooks` and conventions §7 and §8 now state that the budget is one counter both branches spend.

**What this leaves.** The suite's four verification gates — dry-run materialisation, preamble exit 0, `config.toml` and `state.toml` parse, and `gates.toml` loading by the real binary across every declared phase — exist because six of the seven failures would have been caught by them before a paid run. They are cheap and they run first. Hooks-on measurement still waits on the pin of Q-016.

---

## Q-023 · skills · answered

**Ambiguity.** An eval run reached the Design phase and the skill that answered was not ours. The nine slash names are common English words, and a user-level or plugin skill of the same name shadows the project's for the `Skill` tool.

**What was measured.** `design` collided. A skill installed at `~/.claude/skills/design/` — or exposed by a plugin under that name — takes the name, and a project skill at `.claude/skills/design/` does not win. So `exploring-ideas` invoking `design` in sketch mode reached someone else's skill, which knows nothing of `sketch-request.json`, and the step returned content the workflow could not use rather than failing outright. The failure is quiet, which is the part worth recording: a shadowed skill runs and returns something.

**Where it is fixed.** In the runner, now: a case's workspace installs every skill the case's own skill co-invokes, not just the one under test, so the eval measures our Design rather than whatever the host machine has. In a real project the fix is the user's — remove or rename the colliding skill, since two skills cannot hold one name and the framework cannot claim it.

**Proposed, not built.** `init` could check `~/.claude/skills/<name>` and the installed plugins for each of the nine names and print a warning naming the collisions and what they shadow. It is a warning rather than a refusal: the user's own skill may be the one they want, and `init` has no standing to decide that. This is recorded as a proposal because nothing has been written for it.

**If you decide otherwise** and want the nine renamed to something unlikely to collide — `dfa-design`, or similar — say so. It is the same mechanical pass as Q-021, and it moves the frontmatter `name` of nine skills plus the `UserPromptExpansion` matcher; `produced_by` is the directory name and does not move.
