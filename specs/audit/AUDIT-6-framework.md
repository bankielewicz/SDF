---
schema: devforgeai-audit/1
area: framework
produced_by: audit-framework
---

# Audit 6 · contract documents and cross-cutting design

Sources read in full: `specs/ANTHROPIC-GUIDANCE.md`, `specs/00-conventions.md`, `specs/BUILD-BRIEF.md`, `specs/questions.md`, `README.md`, `hooks/settings.hooks.json`, `specs/audit/AUDIT-2-hooks.md`, `specs/audit/AUDIT-4-agents.md`. Read by grep: `specs/01-cli.md` through `specs/11-subagent-catalog.md`, all nine `skills/*/SKILL.md` and `agents.md`, all nine `commands/*.md`, `cli/src/cmd/init.rs`, `evals/runner/run_jsonl.py`, `skills/establishing-context/evals/graders.py`.

Findings that AUDIT-2 or AUDIT-4 own are cited by their ID and not restated. Where this audit names the same defect at a location those audits did not reach, the row says so.

Counts: 5 blocker, 14 high, 16 medium, 14 low.

## 1. Conventions §6 and §7 against the real hook contract

| ID | Severity | Location | Guidance or spec says | What exists | Failure scenario |
|---|---|---|---|---|---|
| FWK-001 | blocker | `specs/00-conventions.md:164`, and §1 rule 3 at `:17` | Guidance §1: for most events Claude Code writes hook stdout to the debug log; the exceptions are `UserPromptSubmit`, `UserPromptExpansion`, `SessionStart`, `PostModelSwitch`. `Stop` is not one of them | "The Stop hook prints this block." Rule 3 makes that the end of every phase | The contract document every other spec conforms to states a delivery mechanism the harness does not provide. AUDIT-2 HOOK-021 fixes the code; this row is the contract text, which nine skill specs, nine SKILL.md files, and the CLI spec's §Handoff all derive from. Fixing the code without the contract leaves the next author writing to the same false premise |
| FWK-002 | high | `specs/00-conventions.md:168` | Guidance §1: exit 4 is a non-blocking error on every event; exit 2 or a JSON decision is what enforces | "All hooks begin with `devforgeai trust verify` and exit 4 if it fails." AUDIT-2 HOOK-051 records the same claim at §8 bullet 2 | Two sentences in the contract, §7 and §8, both assert a block that exit 4 does not produce. §7 is the one the hook table sits under and the one a hook author reads first. `specs/01-cli.md:2680` Decision 7 already resolved this correctly for the implementation; the conventions text was left behind |
| FWK-003 | high | `specs/02-explore.md:201`, `:244`; `specs/05-plan.md:183`; `specs/07-verify.md:240`; `specs/08-design.md:203`; `specs/09-release.md:204`; `specs/10-reflect.md:185` | Guidance §1: `PostToolUse` reaches Claude through `hookSpecificOutput.additionalContext` or top-level `systemMessage`; plain stdout on that event lands in the debug log | Seven workflow steps in six specs close with a repair loop: the `PostToolUse` hook annotates a `doc validate` failure and the model rewrites the section the annotation names | AUDIT-2 HOOK-010 establishes that no annotation channel is emitted. These seven steps are the framework's only self-repair path for a malformed phase document, and each one waits on a message that lands in a log file. The model writes the document, sees nothing, and proceeds. The defect surfaces at the next `gate check` as a whole-phase failure rather than at the line that caused it |
| FWK-004 | high | `specs/00-conventions.md:176` (§7 Stop row); `specs/04-constitute.md:159`; `specs/05-plan.md:185`; `specs/06-build.md:137`; `specs/07-verify.md:242`; `skills/establishing-context/SKILL.md:164`; `skills/implementing-stories/SKILL.md:68`; `skills/planning-work/SKILL.md:57`; `skills/validating-quality/SKILL.md:154` | Guidance §1: exit 1 is a non-blocking error; the action proceeds with a hook-error notice | "gate FAIL (exit 1) blocks stop once; second Stop passes with FAIL handoff", restated in four specs and four shipped skills | A failing gate ends the turn. The claim is load-bearing for the phase model — a phase that fails its gate is supposed to hold the model in the turn until it is fixed — and it rests on the one exit code the reference documents as a silent pass. AUDIT-2 HOOK-011 and HOOK-032 cover the dispatcher and the counter; the exit-1 claim itself lives in these nine places |
| FWK-005 | medium | `specs/00-conventions.md:174-175` (§7 PostToolUse rows) | Guidance §1: `PostToolUse` honours no exit 2 and reaches Claude only through JSON | Both rows record the event as non-blocking with an annotation, with no channel named | The rows read as a complete specification of the event. An implementer satisfying them exactly produces HOOK-010's defect, which is what happened |
| FWK-006 | medium | `specs/00-conventions.md:164` second sentence; the same rule at `skills/exploring-ideas/SKILL.md:72`, `:112`, `skills/discovering-requirements/SKILL.md:55`, `skills/establishing-context/SKILL.md:165`, `skills/implementing-stories/SKILL.md:68`, `skills/planning-work/SKILL.md:57`, `skills/validating-quality/SKILL.md:154`, `skills/releasing-software/SKILL.md:68`, `skills/improving-framework/SKILL.md:62`, `skills/designing-interfaces/SKILL.md:128`, `specs/02-explore.md:248`, `specs/08-design.md:217`, `specs/09-release.md:214`, `specs/10-reflect.md:189` | Under a hook-rendered handoff the model contributes no part of the closing output | "The model may write one sentence before it and nothing after it", carried into nine skills and four specs | The rule exists to stop the model composing the block. Once the hook renders the block into `systemMessage`, the model's one sentence lands above a block it cannot see the content of, and a model that writes a summary sentence duplicates the `Done` line. Fourteen files carry the instruction and each one is a place the model spends tokens on a slot the design has closed |
| FWK-007 | low | `specs/00-conventions.md:173` (§7 PreToolUse row) | Guidance §1: the `PreToolUse` decision travels in `hookSpecificOutput.permissionDecision`; top-level `decision`/`reason` are deprecated for that event | The row names blocking conditions and no output shape | Correct in outcome and silent on mechanism, which is how HOOK-020 reached a code base with no `hookSpecificOutput` anywhere |
| FWK-008 | low | `specs/00-conventions.md:177` (§7 SubagentStop row) | Guidance §1: `SubagentStop` blocks on exit 2 and its `reason` reaches the subagent as its next instruction | The row records the event as non-blocking | AUDIT-2 HOOK-014 covers the missed enforcement. Recorded here because the conventions row is what makes the dispatcher's exit 0 conformant |

## 2. Trust failure delivery

AUDIT-2 HOOK-050 and HOOK-051 establish that three of the five registered events fail open on a tampered binary, and that exit 4 blocks on none. This section adds the parts of the trust posture no other audit covers: which surfaces the current registration leaves untouched, and what a fail-closed set looks like.

| ID | Severity | Location | Guidance or spec says | What exists | Failure scenario |
|---|---|---|---|---|---|
| FWK-010 | blocker | `hooks/settings.hooks.json:13`; `specs/00-conventions.md:185` | Guidance §1: a `PreToolUse` deny fires in every permission mode and cannot be bypassed by mode. Guidance §1 matcher rule: shell tools are `Bash\|PowerShell` | The only blocking `PreToolUse` handler matches `Write\|Edit`. A tampered or unpinned binary leaves every `Bash` and `PowerShell` call, every `Agent` spawn, and every `Read` unexamined | The trust model's stated purpose is that nothing passes a gate on an unpinned binary. As registered, an unpinned binary stops file writes through two tools and stops nothing else. A session can run the whole test suite, spawn every subagent, and commit through Bash while the trust gate reports a failure into a log file |
| FWK-011 | high | `specs/questions.md:168-181` (Q-016); `cli/src/cmd/init.rs` (whole file); `README.md:31` | Q-016: until a human runs `trust pin`, every hook exits with `DFA-E501`, "which is the fail-closed path" | The fresh-install state of every target project is a trust failure. Combined with FWK-010 and HOOK-050, that state blocks writes through two tools and produces a hook-error notice on the other three events | The default state of a new install is the failure state, and the failure state is mostly silent. A user who installs the framework and does not read Q-016 gets a session where the Stop gate, the producer check, the partial build gate, and verifier ingest are all inert, with no statement anywhere in the product surface that this is what is happening. `init`'s closing `Next` line reads `/explore` |
| FWK-012 | high | `hooks/settings.hooks.json` (no `UserPromptExpansion` group); `specs/00-conventions.md:166-181` | Guidance §1: `UserPromptExpansion` fires when the user types `/name`, matches on the command name, and blocks on exit 2 or `decision: block`. It covers the path a `PreToolUse` handler on `Skill` misses | No registration. Gate enforcement for a phase entry point is the command file's `!` preamble, which runs after expansion | AUDIT-2 HOOK-090 records the token-cost half of this. The trust half is larger: typing `/build STORY-003` on an unpinned binary loads the whole skill body, runs the preamble, and starts the phase. The one event that can refuse the phase before any of that is unregistered |
| FWK-013 | medium | `specs/00-conventions.md:185`; `cli/src/hooks/run.rs` per HOOK-050 | Guidance §1: a hook that exits 1 or 4 is a non-blocking error | "the handoff shows `Gate      TRUST FAIL`" — the handoff is rendered by the Stop hook, which is running the same untrusted binary that failed the check | The one user-visible symptom the conventions promise is produced by the component under suspicion. A binary substituted to pass its own trust check also renders a passing handoff. The check has value against accident and none against substitution unless the refusal reaches a channel the binary does not compose |
| FWK-014 | low | `commands/*.md` frontmatter, all nine; `specs/00-conventions.md:110` | Guidance §1: on Windows the PowerShell tool is primary and a `Bash`-only matcher does not fire. Permission rules name the tool | Every command grants `Bash(devforgeai:*)` and no PowerShell equivalent | The `!` preamble runs under the harness's own shell and is unaffected. A workflow step where the model runs a `devforgeai` subcommand itself — `phase set`, `doc load`, `report show`, `story files --diff`, each named in the skill bodies — resolves to the PowerShell tool on a Windows target and finds no grant, so it prompts on every call |

## 3. Aspirational claims across specs 01 to 11

Each row names the claim, the location, and the statement that is true under the guidance. Rows whose defect AUDIT-2 or AUDIT-4 owns carry the cross-reference and add only locations those audits did not name.

| ID | Severity | Location | Corrected statement | What the spec claims | Failure scenario |
|---|---|---|---|---|---|
| FWK-020 | blocker | `specs/02-explore.md:257`; `specs/11-subagent-catalog.md:121`, `:2072` | A subagent has no `AskUserQuestion` tool whatever `tools` holds; a question belongs to the invoking skill | `idea-interrogator` carries the tool, and catalog rule 2 lists it among the permitted read-only set | AUDIT-4 AGT-001 and AGT-002 own this, with a six-file fix spec. Listed for completeness of the aspiration sweep |
| FWK-021 | medium | `specs/03-discover.md:189`, `:227`, `:809`, `:810`; `specs/07-verify.md:575`, `:589`, `:592`; `specs/09-release.md:254`; `specs/11-subagent-catalog.md:277`, `:317`, `:1335`, `:2284` | The tool is absent from every subagent, so removing it from a `tools` list restores nothing and costs nothing | Twelve passages justify dropping the tool from an adapted agent on the grounds that a registered verifier runs unattended, or that the skill owns the questions | The reason given is a policy choice, which reads as though keeping the tool were an option. That is the reading under which `idea-interrogator` kept it. The corrected reason is availability, and it applies to all 46 agents rather than to the adapted ones |
| FWK-022 | high | `specs/00-conventions.md:174`, `:175`; `specs/01-cli.md:515`; `specs/02-explore.md:201`, `:244`, `:424`; `specs/05-plan.md:183`, `:408`; `specs/07-verify.md:240`, `:640`; `specs/08-design.md:203`, `:377`; `specs/09-release.md:204`; `specs/10-reflect.md:185` | `PostToolUse` output reaches Claude through `hookSpecificOutput.additionalContext` or top-level `systemMessage`; an annotation on plain stdout reaches the debug log | Fourteen passages describe a `doc validate` annotation the model reads and acts on | FWK-003 carries the scenario. The count matters: the annotation loop is named in six of the nine phase specs and in the conventions, so the fix is a contract change plus fourteen edits, not one code change |
| FWK-023 | high | `specs/08-design.md:217`, `:378`; `specs/10-reflect.md:189`, `:384`; `skills/improving-framework/SKILL.md:62`; `skills/designing-interfaces/SKILL.md:128` | The Stop hook runs `devforgeai handoff`; a model running it through Bash produces tool output, not a rendered handoff, and conventions §1 rule 3 places composition outside the model | Four passages instruct the model to run `devforgeai handoff --phase design`/`--phase reflect` as the last workflow step | Design and Reflect are cross-cutting: they leave `[current].phase` alone, so the Stop hook renders the phase the user was in, and each spec compensates by having the model run its own handoff. Under a `systemMessage`-delivered block the two collide — one Stop emits one JSON object, so a model-run handoff appears in a tool result above a hook-rendered block for a different phase. The delivery design in §Fix specifications resolves this by concatenating both blocks into one `systemMessage` |
| FWK-024 | medium | `specs/00-conventions.md:177`; `specs/02-explore.md:442`; `specs/04-constitute.md:326`, `:327` | The `SubagentStop` payload carries `agent_type`, `agent_id`, `last_assistant_message`, `agent_transcript_path`, `stop_hook_active` | Four passages pass `<stdout>` or `<source>` to `report ingest` as though the event carried the subagent's stdout as a field | AUDIT-2 HOOK-040 and HOOK-042 own the code and the three `specs/01-cli.md` lines. These four are the remaining loci; each is a one-token edit to `<last_assistant_message>` and each is invisible to a grep for `tool_response` |
| FWK-025 | medium | `specs/04-constitute.md:157`; `skills/exploring-ideas/agents.md:17`, and the equivalent paragraph in the other eight `agents.md` | The model sees an `Agent` tool result and can re-invoke on a parse failure; it has no channel that reports an ingest failure, because `SubagentStop` output on exit 0 reaches the debug log | "A subagent returning output that does not parse against its schema is re-invoked once with the parse error appended to the prompt" | The re-invocation works for the model's own parse of the returned text. The claim is written as though it covered the ingest path too, which is where AGT-014 through AGT-018's two-object agents fail. An agent whose text parses for the model and fails for `report ingest` is re-invoked zero times and leaves the gate metric absent |
| FWK-026 | medium | `specs/00-conventions.md:101`; `specs/01-cli.md:2729`; `skills/exploring-ideas/SKILL.md:32`, and the `phase set` steps of the other eight skills | A non-zero exit from a `!` preamble aborts the skill invocation, which is enforcement; a non-zero exit from a model-run Bash call is a message the model may act on or not | "`phase set` refuses unless `gate require` would pass", with the refusal treated as binding wherever the call appears | The refusal is real inside the binary. Its force depends on the caller: in the `!` preamble it is a gate, and in a mid-workflow step it is a request. Six of the nine skills run `phase set` from a workflow step rather than the preamble. A model that reads the stderr and continues advances the phase in its own narrative while `state.toml` holds the old value, and the next `gate require` fails on an id nothing set |
| FWK-027 | high | `specs/00-conventions.md:175`; `specs/01-cli.md:2306`; `specs/06-build.md:498`; `hooks/settings.hooks.json:27` | Shell-command matchers are `Bash\|PowerShell` | Three specs and the shipped template register `Bash` alone | AUDIT-2 HOOK-001 owns the template and the dispatcher. The three spec loci are where a regenerated template would pick the defect back up, and `specs/00-conventions.md:175` is the row the other two conform to |
| FWK-028 | low | `skills/validating-quality/SKILL.md` (333 lines), `skills/establishing-context/SKILL.md` (317 lines) | Guidance §3: SKILL.md is the standing instruction for the whole phase and is not re-read; keep it under 500 lines and say when to read each supporting file | Both are well under 500 lines. No skill claims a re-read | No defect on the re-read premise: the sweep found no passage expecting Claude Code to reload a SKILL.md. Recorded as a negative result, with one qualifier — a phase whose turn is long enough to compact relies on the 25,000-token re-attachment budget, and the two largest skills are the ones whose phases run longest |
| FWK-029 | low | `specs/00-conventions.md:38` | Guidance §2 and §3: the tool set includes `PowerShell`, `NotebookEdit`, `ToolSearch`, `TodoWrite`, `Artifact`, `Monitor`, `SendMessage`; commands are merged into skills | The aspiration rule's primitive list names eleven tools, omits the shell tool that is primary on the target platform, and names `.claude/commands/*.md` as a first-class primitive | Every spec closes its `## Decisions` with "every step runs on the §2 primitive list", checked against a list that omits the tool the specs' own Bash steps resolve to on Windows. The list is the framework's definition of buildable, and it is two releases behind |
| FWK-030 | low | `specs/10-reflect.md:476`; `specs/11-subagent-catalog.md:1726`; `skills/improving-framework/references/recommendations.md:11`, `:25`, `:29` | After the §4 collapse there is no `commands/` directory and no command file for a recommendation to target | Reflect's seven-row target-kind table carries a `command` row pointing at `commands/build.md` | A recommendation naming a file the layout no longer holds. Sequenced with the §4 decision |
| FWK-031 | low | `specs/01-cli.md:1563`, `:1565`, `:1567` | Guidance §1, matching these rows | The CLI spec's per-event trust table states plainly that `SessionStart`, `PostToolUse`, and `SubagentStop` exit 4 and the action stands | Correct, and in direct contradiction with `specs/00-conventions.md:168` and `:185`. Recorded so the §7 rewrite is read as bringing the conventions to the CLI spec rather than the reverse |
| FWK-032 | low | `specs/02-explore.md:930`; `specs/05-plan.md:926`; `specs/08-design.md:872`; `specs/09-release.md:1158`; `specs/10-reflect.md:833`; `specs/11-subagent-catalog.md:2299` | The aspiration rule is a real constraint only if its primitive list is current (FWK-029) and if the blockers the guidance identifies are counted | Six specs close with "blockers: none" | Each of the six was written before the guidance digest existed. Three of them contain a capability the guidance contradicts (FWK-020, FWK-022, FWK-023). The `## Decisions` blocker count is the framework's own tally of what it cannot do, and it reads zero |

## 4. Commands and skills

Decision: **collapse**. `commands/` is deleted and each SKILL.md carries the frontmatter and the `!` preamble the command file carried. This concurs with `AUDIT-3-skills.md` SKL-024, which owns the eight-step migration; the memo below states the decision's grounds independently and adds the change points SKL-024's steps do not name.

| ID | Severity | Location | Guidance or spec says | What exists | Failure scenario |
|---|---|---|---|---|---|
| FWK-040 | high | `commands/*.md` and `skills/*/SKILL.md`, all nine pairs | Guidance §3: a skill and a command file of the same name both create `/name`, and the skill wins. Guidance §7.8: the thin command's preamble works identically inside SKILL.md, and a failing preamble aborts the invocation | Eighteen files for nine entry points. The nine SKILL.md bodies assert the preamble already ran; the nine command files carry it | Two files govern one entry point and only one of them is read by the model at run time. An edit to a command's preamble (adding a `gate require` line, changing `allowed-tools`) leaves the SKILL.md's `## Entry` description stale with no signal. The failure is silent drift, and it has already produced one instance: `commands/design.md`'s allocation line duplicates `designing-interfaces/SKILL.md:24` |
| FWK-041 | high | `skills/*/SKILL.md` frontmatter, all nine | Guidance §3: `disable-model-invocation: true` gives user-only invocation and keeps the description out of context until invoked; recommended for workflows with side effects | No skill declares it. Nine descriptions, each a full paragraph, sit in context for every session | AUDIT-3 SKL-017 owns the declaration. The side-effect half is what makes it more than a token argument: a model that reaches for `implementing-stories` on its own starts a phase that writes a worktree, commits, and advances `state.toml`, with no `gate require` having run, because the preamble runs on invocation and the gate it enforces is for the phase the user asked for |
| FWK-042 | medium | `cli/src/cmd/init.rs:12`, `:117-120`; `cli/tests/init.rs:137`, `:228`; `evals/runner/run_jsonl.py:46`, `:231-232`; `specs/11-subagent-catalog.md:1924` | The layout after the collapse holds `skills/` and `agents/` | Three code paths and one spec sentence pin `commands/` as a shipped directory | These are the breakage points of a partial migration. `init` walks a missing directory (returns zero files, so the failure is a count of 0 in the summary line rather than an error); the eval runner fails every workspace build; the catalog's sentence sends a future author to nine `## Command` sections that no longer describe a file |
| FWK-043 | medium | `README.md:22`, `:36`, `:38-49`; `specs/00-conventions.md:18` (rule 4), `:57`, `:106-110` (§4b), `:247`; `specs/BUILD-BRIEF.md:29`, `:51`, `:72` | Same | Rule 4 ("One thin slash command per skill"), the §4b command-file-shape paragraph, the two layout blocks, the install paragraph, the build brief's output list, and its grep line all describe the two-file shape | The contract documents are the input to every future build. A collapse that edits code and skills and leaves these six places produces the eighteen-file shape again on the next build wave |
| FWK-044 | low | `agents/*.md` `description` lines, 46 files; `skills/*/agents.md` `## Invocation order` tables, nine files | The slash name is decided by the skill's `name:` field, so `/build` survives the collapse unchanged | Agent descriptions name slash commands (`/verify`, `/build`); the nine `agents.md` invocation tables key on workflow step numbers and name no command file | No change is needed in either place, and the reason is worth recording: `name: build` on the collapsed skill keeps every slash reference in the 46 agent descriptions, in `cli/src/aggregate.rs` `SLASH_COMMANDS`, and in every `cases.jsonl` prompt valid. AGT-009 removes the slash names from the descriptions for a different reason |

## 5. Target-project CLAUDE.md

| ID | Severity | Location | Guidance or spec says | What exists | Failure scenario |
|---|---|---|---|---|---|
| FWK-050 | blocker | `cli/src/cmd/init.rs` (whole file); `specs/01-cli.md` `## Workflow` init steps; `README.md:36` | Guidance §6 and §7.13: CLAUDE.md under 200 lines is the standing layer for what Claude cannot infer; `devforgeai init` writes a section for the target project under 40 lines with the nine commands, where the handoff comes from, and that gates are hooks | `init` writes `.devforgeai/`, copies two or three directories, merges hooks, appends two `.gitignore` lines, and installs git hooks. The string `CLAUDE.md` appears nowhere in `cli/`, in `README.md`, or in any spec | A target project has nine skills whose descriptions are out of context until invoked (after FWK-041), a hook layer that enforces silently, and no standing statement that any of it exists. A user who types a plain request rather than a phase command gets a session that writes code outside the phase model, trips the producer check on its first `.devforgeai/` write, and receives a denial with no context for it. The one layer designed to carry that context is absent |
| FWK-051 | low | `cli/src/cmd/init.rs:14`, `:284-306` | The idempotent-append pattern already exists in this file | No defect. `IGNORE_LINES` and `append_gitignore` skip lines already present, line by line | The mechanism the CLAUDE.md block needs is written and tested (`init_appends_gitignore_lines`). Recorded as the precedent the fix spec follows rather than as a defect |
| FWK-052 | medium | `cli/src/cmd/init.rs:151` | `init`'s closing line is the user's first instruction | `Next       /explore` | With no CLAUDE.md and a fresh install in the trust-failure state (FWK-011), the first instruction the product gives is a command whose gate layer is inert. The `Next` line and the CLAUDE.md block are the two places to state the pin step |
| FWK-053 | low | `.gitignore` lines at `cli/src/cmd/init.rs:14` | A target project gains `.claude/skills/`, `.claude/agents/`, and a merged `.claude/settings.json` | Two lines: `.explore-prototype/` and `.devforgeai/state.toml` | `.devforgeai/state.toml` is ignored while `.devforgeai/reports/` and the phase documents are committed, which is the right split. Nothing ignores the copied `.claude/skills/*/evals/` trees that FWK-072 describes, so each install commits the framework's grader and fixture set into the target's history |

## 6. Questions file, Q-001 to Q-017

| Q | Resolved by the guidance | Draft resolution |
|---|---|---|
| Q-001 | no — a working-mode decision for Bryan | Leave open. The guidance's Fable 5.1 paragraph, which says an orchestrator operating autonomously blocks the work by asking permission mid-run, supports the assumption, and the guidance settles no question about this user's preference. Keep the assumption and the log. |
| Q-002 | yes, indirectly | Agreed as assumed. `explore prune` is a CLI subcommand, which is where conventions §1 rule 1 places a destructive filesystem operation. The guidance adds nothing and contradicts nothing. Mark answered. |
| Q-003 | no — a project parameter | Leave as assumed. Two numbers in `config.toml [explore]`; the guidance has no bearing. |
| Q-004 | no — internal reconciliation | Leave as assumed; superseded by Q-010, which covers the second round. |
| Q-005 | yes | Resolved and reinforced. The nine names are now the skill `name:` values after the §4 collapse, so they decide the slash command, the Skill-tool target, and `cli/src/aggregate.rs` `SLASH_COMMANDS` from one field. Mark answered, with the note that a rename now moves three things at once. |
| Q-006 | no — CLI surface | Leave as assumed. One correction: `report ingest`'s second argument is the subagent's `last_assistant_message` from the `SubagentStop` payload rather than its stdout (FWK-024, HOOK-040). |
| Q-007 | yes, in the direction of restoring MoSCoW | Resolved by §8 below: the ceremony pattern applies to instruction prose, not to data values in a document the framework produces. With the scoping rule in place the exemption Q-007 asks for exists, and the enum returns to the four MoSCoW levels it had. The cost is listed in §8. |
| Q-008 | no — structural | Leave as assumed. The guidance settles nothing about id keying. |
| Q-009 | no — product scope | Leave as assumed. |
| Q-010 | no — internal reconciliation | Leave as assumed. |
| Q-011 | no — external tool policy | Leave as assumed, with one reinforcement: the guidance's rule that an agent's tool set is its `tools` list plus `mcpServers` and nothing else is why the two `config.toml` commands need a permission path (AGT-003, AGT-004). The Treelint decision is unchanged; the mechanism that runs any configured command needs the fix AGT-003 specifies. |
| Q-012 | no — enum choice | Leave as assumed; it is settled across three specs. |
| Q-013 | yes | Resolved. Plugin namespacing is correct: a plugin skill resolves as `plugin:skill` and a first-party skill resolves bare, which is what the patched names carry. Two follow-ons the guidance adds: a subagent holding the `Skill` tool can load a skill and still lack the MCP tools that skill drives (AGT-005), and `skills:` in agent frontmatter preloads the content without spending a turn (AGT-047). Mark answered with those two notes. |
| Q-014 | yes, indirectly | Resolved and extended. Excluding `evals/` from the eval workspace closes the gaming vector. The guidance's install model makes the second half visible: `init` copies `skills/` wholesale into the target, so the graders and fixtures land in every installed project (FWK-072). The same exclusion belongs in `init`'s copy. |
| Q-015 | no — an operational decision | Leave open, narrowed. The nested-session environment fix is in place. BLD-07 and the shell question are Bryan's to answer; neither turns on the guidance. |
| Q-016 | no — an action only Bryan can take | Leave open. The guidance does not change the pin step. Two additions from this audit: the unpinned state is mostly silent rather than fail-closed (FWK-010, FWK-011, HOOK-050), so the pin step belongs in `init`'s output and in the target CLAUDE.md block (§5), and the `--framework` argument is required by the binary and absent from `README.md:31`. |
| Q-017 | yes | Resolved by option (a), with the mechanism named. Guidance §4: a `PreToolUse` hook on `AskUserQuestion` returning `permissionDecision: "allow"` with `updatedInput` that echoes `questions` and adds `answers` mapping question text to the chosen label is the supported pre-seed path; the tool then runs with no prompt, so the model takes the case's answer through the same code path it takes a user's. `--permission-prompts none` covers cases that should reach no question, and `defer` pauses a run for a caller that resumes. The runner writes a per-case settings block registering that handler, seeded from the case's `answers` map; the fixed headers already in the question templates are what the handler keys on. No skill changes. |

## 7. No-silos check on `init` and the README

| ID | Severity | Location | Guidance or spec says | What exists | Failure scenario |
|---|---|---|---|---|---|
| FWK-070 | blocker | `skills/improving-framework/SKILL.md:32`, `:52`; `skills/improving-framework/references/recommendations.md:11`, `:25`, `:29`; `specs/11-subagent-catalog.md:1726` | Conventions §3: a shipped skill runs inside the target project, whose tree holds `.claude/` and `.devforgeai/` and no framework directories | The Reflect skill reads "the framework files under `skills/`, `agents/`, `commands/`, `hooks/`, and `specs/`" from the target's working directory, and passes the paths `Glob` returns to `recommendation-drafter`. The target holds none of those paths; it holds `.claude/skills/` and `.claude/agents/` | `/reflect` in an installed project globs five directories that do not exist, gets an empty list, and hands `recommendation-drafter` nothing to target. Every recommendation it can produce names a file outside the project. The phase runs, writes a report, and its `target` field is unfillable. A target that happens to be a JavaScript project with its own `skills/` directory is worse: the agent proposes edits to unrelated files |
| FWK-071 | high | `skills/validating-quality/agents.md:52`; `skills/discovering-requirements/evals/graders.py:17`; `skills/establishing-context/references/constraints.md:23`; `skills/establishing-context/evals/fixtures/digests.txt:41`; `skills/exploring-ideas/evals/fixtures/digests.txt:46`; `skills/discovering-requirements/evals/fixtures/baselines.txt:7`, `:9` | Same | Seven shipped files cite `specs/01-cli.md`, `specs/02-explore.md`, `specs/03-discover.md`, or `specs/04-constitute.md` as the authority for a contract | The specs are not installed. A reader in a target project who follows the citation finds nothing. `skills/validating-quality/agents.md:52` is the worst case: it points at the verifier envelope contract, which is the one thing an agent author needs and the one thing the target does not hold. The fix is to state the contract in the shipped file or to point at the installed `agents/<name>.md` |
| FWK-072 | high | `cli/src/cmd/init.rs:12`, `:220-260` (`copy_tree`) | Q-014 excludes `evals/` from an eval workspace for a gaming reason; the same reasoning applies to a target install, and a target has no use for graders or fixtures | `copy_tree` walks every file under `skills/`, so each target gets nine `evals/` trees: `evals.json`, `cases.jsonl`, `graders.py`, and every fixture | Every installed project carries the framework's grader source and its fixture corpus under `.claude/skills/`, committed to the target's history (FWK-053). A model working in that project can read `graders.py` and the expected outputs. The eval workspaces are protected and the real projects are not |
| FWK-073 | medium | `README.md:31`, `:36`; `cli/src/cli.rs:373-379`; `cli/src/cmd/init.rs:177-208` (`resolve_framework`) | `trust pin` takes `--framework <path>`, and `init`'s `--from` defaults to the `framework_path` of the matching pin | The README's install sequence is `devforgeai trust pin` then `devforgeai init [--analyze]`, with no `--framework` and no `--from` | `trust pin` with no `--framework` records no framework path, so `resolve_framework` finds nothing to default to and `init` exits `DFA-E111` telling the user to pass `--from`. The documented sequence fails at step two. Q-016 has the correct command; the README does not |
| FWK-074 | medium | `README.md:28-49` | The install sequence's first requirement is a built binary | The README opens the install section with `devforgeai trust pin` and says nothing about building `cli/`, where the binary lands, or how it reaches `PATH` | A reader following the README has no binary. The framework's only distribution step is missing from its only distribution document |
| FWK-075 | medium | `README.md:36`; `cli/src/cmd/init.rs:284-306` | `init` appends two lines to the target's `.gitignore` | The README's `init` paragraph lists five actions and omits the `.gitignore` append and the CLAUDE.md block that §5 adds | A user reviewing what `init` touched sees an unexplained edit to a file they own. Small, and it is the one file in the target root that `init` modifies today |
| FWK-076 | medium | `README.md:67-83`; `evals/runner/run_jsonl.py` | The runner reads `--skill skills/<name>`, which is a framework-repo path | The README documents the eval commands with no statement of where they run | Answering the audit question directly: evals run from the framework repo, not from a target project. A target holds `.claude/skills/<name>/evals/` after FWK-072 and holds no runner, so the commands as written resolve against nothing there. One sentence fixes it, and the same sentence is the argument for FWK-072's exclusion |
| FWK-077 | low | `README.md:28-34`; `~/.devforgeai/trust.toml` | The trust file is per-machine and outside both repositories | The README's sequence implies it without saying it | Answering the audit question: the trust files are not copied by `init` and are not project state. `~/.devforgeai/trust.toml` is written once per machine by a human; `cli/REVISION` and `cli/DIGEST` ship with the framework repo. A target project holds neither, which is the intended shape |
| FWK-078 | low | `cli/src/cmd/init.rs:63-70`, `:85`, `:89` | Conventions §3's installed-tree block | `init` writes `gates.toml` from the compiled default, `state.toml` from the initial template, `config.toml` through `stack detect`, merges the hooks, and installs three git hooks | Answering the audit question: the three project files, the hook merge, and the git hooks are all covered. The gaps are the CLAUDE.md block (FWK-050), the `evals/` exclusion (FWK-072), and the `commands/` entry that the §4 collapse removes (FWK-042) |

## 8. Conventions §2 regex false positives

| ID | Severity | Location | Guidance or spec says | What exists | Failure scenario |
|---|---|---|---|---|---|
| FWK-080 | high | `skills/establishing-context/evals/graders.py:480-507` | Conventions §2 scopes the pattern to prose lines of a SKILL.md, command, or subagent — text whose function is to make the model check itself. A context file is the target project's own rule set, and a requirements record is data | `no_ceremony` walks every file under `.devforgeai/` in the eval workspace, splits on lines, skips fenced blocks, and fails the case on any match in any file | The grader fails a correct output. A project whose error-handling rule reads "a store failure is wrapped at the adapter boundary and does not surface its driver type" trips the pattern — that exact sentence is in `skills/implementing-stories/evals/fixtures/coding-standards.md:35` with the flagged wording. The six context files are where a project states absolute rules, so the phase whose whole output is absolute rules is the phase graded against a rule forbidding them |
| FWK-081 | high | `specs/questions.md:76-84` (Q-007); `skills/discovering-requirements/templates/requirements.yaml:35` and 20 fixture lines | The pattern applies to instruction prose; a YAML scalar in a data document is not prose | The MoSCoW enum was renamed to `required / expected / optional / excluded` to avoid a token match | A vocabulary every stakeholder knows was replaced to satisfy a lint that was misapplied. The rename propagated into one template and twenty fixture lines across four skills, and into two pinned SHA-256 digests |
| FWK-082 | medium | `specs/00-conventions.md:28`; `specs/BUILD-BRIEF.md:72` | A scoping rule states what a line is before matching it | §2's parenthetical is "applied to prose lines, not to quoted CLI output or templates", which names two exclusions and leaves table cells, backticked spans, YAML scalars, JSON string values, file paths, and headings unsaid. The build brief's grep line carries no exclusion at all | Two authorities disagree: §2 excludes two categories and the brief's verification command excludes none. A builder running the brief's grep on a spec-conformant file gets hits and has no rule for dispositioning them. `specs/11-subagent-catalog.md:2298` had to write a paragraph explaining two expected hits, which is the cost of the missing rule paid once per document |
| FWK-083 | medium | `specs/00-conventions.md:28` | The pattern's function is to catch an instruction to the model | The pattern is case-insensitive, so it matches the lower-case ordinary-English uses of the same words in explanatory prose | Case-insensitivity is what turns a rule about emphasis into a rule about vocabulary. The guidance's own reason for the rule — current models over-trigger on capitalised emphasis — points at the upper-case forms. A case-sensitive pattern over the four imperatives, plus the case-insensitive multi-word phrases, catches the behaviour and releases the vocabulary |
| FWK-084 | low | `specs/00-conventions.md:31`; `skills/establishing-context/evals/graders.py:37-40`; `specs/BUILD-BRIEF.md:72` | One pattern, one definition | The pattern is written out three times in three files, with a fourth instance in each spec that quotes it | A change to the pattern edits four or more places. The scoping rule below multiplies that. One definition in the conventions, referenced by the other two, keeps them in step |

**Recommendation on the enum: restore MoSCoW.** Grounds: the four words are the industry vocabulary a stakeholder reading `requirements.yaml` recognises, and the rename makes every such reader translate; `required / expected / optional / excluded` loses the distinction MoSCoW's third and fourth levels carry ("could" is a nice-to-have, "excluded" reads as a prohibition rather than as out-of-scope-for-now); and the reason for the rename was a lint defect rather than a design judgment, so it is a change with a cost and no benefit once the lint is scoped. With the scoping rule in §Fix specifications, a YAML scalar is not a prose line and the enum matches nothing.

**Cost of restoring.** `skills/discovering-requirements/templates/requirements.yaml:35`; the `priority:` lines in `skills/discovering-requirements/evals/fixtures/requirements-remedy-design.yaml`, `requirements-remedy-plan.yaml`; `skills/establishing-context/evals/fixtures/requirements-IDEA-021.yaml`, `requirements-IDEA-022.yaml`; `skills/planning-work/evals/fixtures/requirements-epic-001.yaml`, `-002`, `-003`; `skills/releasing-software/evals/fixtures/requirements-8-reqs.yaml` (eight lines); the inline `setup.files` copies of the same documents in four `cases.jsonl` files; the enum definition in `specs/03-discover.md` `## Outputs` and its restatement in `specs/05-plan.md`, `specs/07-verify.md`, and `specs/09-release.md` where a story or report cites a requirement's priority. Two pinned digests move with it: `upstream_sha256` for cases `CON-05-sendback-infeasible` and `CON-06-sendback-contradiction` in `skills/establishing-context/evals/cases.jsonl`, recorded again in `skills/establishing-context/evals/fixtures/digests.txt:26-30` and in `specs/04-constitute.md` `## Evals`; the recipe for recomputing them is in that digests file. Both the Discover and the Constitute fixture sets change, which is the scope Q-007's "if you decide otherwise" line anticipated.

---

## Fix specifications

### FWK-001, FWK-004, FWK-005, FWK-006 — conventions §6 and §7, replacement text, and the handoff delivery design

Replace the closing paragraph of §6 and the whole of §7 with the text below. The four-case Stop design that the §7 table summarises is specified in full after it. The PASS case is `AUDIT-2-hooks.md` HOOK-021's `systemMessage`; the FAIL, SEND BACK, and `stop_hook_active` cases are new here.

**§6, replacing line 164.**

> The Stop hook emits this block; the model composes no part of it and writes nothing at the close of a phase. On a PASS the block travels in the `systemMessage` field of the hook's JSON, which the harness renders to the user and which Claude also sees. On a FAIL the hook returns a blocking decision whose `reason` names the failing checks, and the same block travels in `systemMessage` beside it. On a SEND BACK the block travels in `systemMessage` with no blocking decision, because the next step is a different command the user types. The model's final message is its own work; the block appears after it.

**§7, replacing lines 166 to 181.**

> All hooks call the binary. All hooks run `devforgeai trust verify` first. Exit 2 is what blocks, on the events that can block: `PreToolUse`, `UserPromptSubmit`, `UserPromptExpansion`, `Stop`, `SubagentStop`. On every other event no exit code blocks, and the framework's channel to a reader is the hook's JSON: `systemMessage` reaches the user, `hookSpecificOutput.additionalContext` reaches Claude. Exit 4 marks a trust failure for the debug log and for the `--json` envelope; on a blocking event the dispatcher exits 2 instead.
>
> | Event | Matcher | Calls | Channel to a reader | Blocks |
> |---|---|---|---|---|
> | SessionStart | `startup\|resume\|clear\|fork` | `stack detect`, `handoff` | plain stdout, which this event adds to Claude's context | no channel; a trust failure emits `systemMessage` |
> | UserPromptExpansion | the nine skill names | `trust verify`, `gate require <phase> <id>` | `reason` of the blocking decision | trust failure; predecessor gate not passed |
> | PreToolUse | `Write\|Edit\|NotebookEdit` | `doc validate --producer-check`, `design lint`, `story files --check` | `hookSpecificOutput.permissionDecisionReason` | producer mismatch; token violation; undeclared file; an internal error the hook cannot evaluate past |
> | PreToolUse | `Write\|Edit\|NotebookEdit\|Bash\|PowerShell` | `trust verify` | same | trust failure |
> | PostToolUse | `Write\|Edit\|NotebookEdit` | `doc validate` | `hookSpecificOutput.additionalContext` | no |
> | PostToolUse | `Bash\|PowerShell`, command matching `config.toml` | `gate check --phase build --partial`, async | `additionalContext` on the next turn | no |
> | PostToolUse | `Agent` | ingested verifier counts to the parent session | `additionalContext` | no |
> | Stop | none | `gate check --phase <current>`, `handoff` | `systemMessage`, plus `reason` on a FAIL | gate FAIL, once per turn, with `stop_hook_active` honoured |
> | SubagentStop | the registered verifier names | `report ingest <name> <last_assistant_message>` | `reason` of the blocking decision | envelope parse failure; trust failure |
> | FileChanged | none | `doc validate` on a watched path | `systemMessage` | no; this event fires after the change |
> | pre-commit (git) | | `doc validate` on staged `.devforgeai/` files, `context audit` | the hook's stderr | any failure |
> | commit-msg (git) | | a `STORY-nnn` or `ADR-nnn` token in the message | the hook's stderr | missing |
> | pre-push (git) | | `gate check --phase build` for every story in `sprint.yaml` at status `building` | the hook's stderr | any FAIL |

**The four Stop cases.** `fn stop` in `cli/src/hooks/run.rs` reads `stop_hook_active` and `session_id` from the payload. The gate's result selects the case. Stdout carries one JSON object and nothing else; `Outcome.human` is suppressed whenever the object is set.

*Case A — gate PASS, `stop_hook_active` false.* Exit 0.

```json
{
  "systemMessage": "Phase     4 · Build        STORY-003 · checkout-flow\nDone      3/3 ACs, 41 tests\nGate      PASS  coverage 86%\nVerified  story-ac-verifier · 3/3 ACs\n\nNext      /verify STORY-003\nThen      /release\nBlocked   none\n\nFull report: .devforgeai/reports/STORY-003-build.yaml"
}
```

*Case B — gate FAIL, `stop_hook_active` false.* Exit 2, one object carrying both keys.

```json
{
  "decision": "block",
  "reason": "Gate      build · STORY-003\nResult    FAIL\nChecks    build-coverage 71 < 80; build-tests 3 failing\nFix the failing checks and stop again; the handoff prints when the turn ends.",
  "systemMessage": "Phase     4 · Build        STORY-003 · checkout-flow\nDone      3/3 ACs, 41 tests\nGate      FAIL  coverage 71% < 80%\n\nNext      /build STORY-003 --resume\nThen      /verify STORY-003\nBlocked   none\n\nFull report: .devforgeai/reports/STORY-003-build.yaml"
}
```

`decision` and `systemMessage` are both top-level and coexist in one object; the reference's constraint is that stdout holds one object, which this satisfies. `reason` is addressed to Claude and names the checks; `systemMessage` is addressed to the user and is the twelve-line block. The same `reason` text is copied into `Outcome.stderr`, so a schema change upstream degrades to the stderr path rather than to silence, which is the mechanism HOOK-011 specifies.

*Case C — `stop_hook_active` true, any gate result.* Exit 0, no gate re-run.

```json
{
  "systemMessage": "Phase     4 · Build        STORY-003 · checkout-flow\nDone      3/3 ACs, 41 tests\nGate      FAIL  coverage 71% < 80%\n\nNext      /build STORY-003 --resume\nThen      /verify STORY-003\nBlocked   none\n\nFull report: .devforgeai/reports/STORY-003-build.yaml"
}
```

The handoff is read from the report the previous Stop's `gate check` wrote, so this case needs no gate run and no test suite run (HOOK-030, HOOK-033). When `[last_gate]` holds no result for the current phase and id — the flag is set because Claude continued for a reason other than this hook, or the previous Stop was a PASS whose block already printed — the case exits 0 and emits no JSON, so no block is printed twice. This is the case that puts the FAIL handoff in front of the user: on a FAIL the user sees nothing at the blocking Stop, and sees the block when the harness ends the continuation. State that consequence in the skill bodies that describe a FAIL close.

*Case D — gate SEND BACK (`gate check` exit 2).* Exit 0, no block.

```json
{
  "systemMessage": "Phase     3 · Plan         SPRINT-002 · -\nDone      7 stories, 24 ACs\nGate      SEND BACK to Discover  2 findings\nFound     REQ-011 acceptance signal names no observable outcome\nFound     REQ-014 contradicts CON-003\n\nNext      /discover IDEA-004 --remedy REQ-011,REQ-014\nThen      /plan EPIC-002 --resume\nBlocked   none\n\nFull report: .devforgeai/reports/SPRINT-002-plan.yaml"
}
```

No blocking decision: the next step is a command the user types, and holding the model in the turn cannot produce it.

*Cross-cutting skills.* Design and Reflect leave `[current].phase` where they found it, so one Stop renders two blocks. This needs a state field that `state.toml` does not carry today: `specs/01-cli.md:367` defines `[current]` and `[active]`, and `[active].design` records an id with no marker for which turn produced it. Add `[last_cross]` with `phase` (`design` or `reflect`), `id`, and `turn`, written by the cross-cutting skill's own `handoff` call site and cleared by the Stop that consumes it. The dispatcher calls `handoff --phase <last_cross.phase> --id <last_cross.id>` when that table is populated, then `handoff --phase <current>`, and joins the two with a blank line into one `systemMessage`, clearing `[last_cross]` afterwards so a second Stop in the same session prints one block. This replaces the model-run handoff of FWK-023. Cap the joined string at 10,000 characters (HOOK-024); two twelve-line blocks are far under it.

*SessionStart.* Plain stdout, exit 0, carrying the last handoff — this event adds stdout to Claude's context, which is the audience for a session-opening block. No `systemMessage`: the user has just opened the session and did not ask for it. Switch to the JSON form only when `sessionTitle` or `watchPaths` travel with it (HOOK-093, HOOK-095).

**Consequence for the "one sentence before, nothing after" rule.** The rule is deleted. The model prints no sentence at the close of a phase and no block; the hook renders the block, and it appears after the model's final message. Delete the sentence from `specs/00-conventions.md:164` (replaced by the §6 text above) and from the fourteen files listed in FWK-006: `skills/exploring-ideas/SKILL.md:72` and `:112`, `skills/discovering-requirements/SKILL.md:55`, `skills/establishing-context/SKILL.md:165`, `skills/implementing-stories/SKILL.md:68`, `skills/planning-work/SKILL.md:57`, `skills/validating-quality/SKILL.md:154`, `skills/releasing-software/SKILL.md:68`, `skills/improving-framework/SKILL.md:62`, `skills/designing-interfaces/SKILL.md:128`, `specs/02-explore.md:248`, `specs/08-design.md:217`, `specs/09-release.md:214`, `specs/10-reflect.md:189`. In each place the replacement sentence is: *the Stop hook renders the closing block; this skill writes no part of it.*

**Test first** (`cli/tests/hook_run.rs`): `stop_pass_emits_handoff_as_system_message`; `stop_fail_emits_block_reason_and_system_message` (one object, both keys, exit 2); `stop_active_flag_emits_handoff_and_runs_no_gate` (exit 0, `actions` holds no `gate check`); `stop_send_back_emits_system_message_without_block` (exit 0, no `decision` key); `stop_cross_cutting_joins_two_blocks` (a Design run recorded in `state.toml`, one `systemMessage` holding two `Full report:` lines).

### FWK-010, FWK-011, FWK-012, FWK-013 — the fail-closed trust set

Three registrations, one dispatcher change, one `init` change. The design goal is that an unpinned or altered binary stops the session's work at the three points where work starts: the phase command, the write, and the turn end.

**1. `PreToolUse`, trust only, wide matcher.** A second `PreToolUse` group in the settings template:

```json
{
  "matcher": "Write|Edit|NotebookEdit|Bash|PowerShell|Agent",
  "hooks": [
    {
      "type": "command",
      "command": "@@DEVFORGEAI@@",
      "args": ["hook", "run", "trust-check"],
      "timeout": 600,
      "statusMessage": "devforgeai: trust"
    }
  ]
}
```

`trust-check` is a new arm in the dispatcher's `EVENTS` enum. It runs `trust verify` and nothing else — no `config.toml` read, no `state.toml` read, no path resolution — so it adds one hash to the write path and cannot fail for any reason other than trust (which keeps it clear of HOOK-012's class). On a pass it exits 0 and writes nothing to stdout, which leaves the permission flow untouched. On a failure it exits 2 with:

```json
{
  "hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "permissionDecision": "deny",
    "permissionDecisionReason": "devforgeai trust verify failed: DFA-E501 no pin recorded for this binary. In a terminal outside Claude Code, run: devforgeai trust pin --framework <framework root>. Then start a new session."
  }
}
```

A deny fires in every permission mode, including `bypassPermissions`, and a hook allow elsewhere cannot override it, so every Write, Edit, NotebookEdit, Bash, PowerShell, and Agent call in the session is refused. That is the intended effect of an unpinned binary: the session can read and reason and can change nothing. Separately from this group, the existing `pre-tool-use` handler keeps its own `trust verify` first line, so removing this group does not reopen the write path.

**2. `Stop`.** In `fn stop`, ahead of the gate, a trust failure with `stop_hook_active` false exits 2 with:

```json
{
  "decision": "block",
  "reason": "devforgeai trust verify failed: DFA-E501 no pin recorded for this binary. No gate ran for this turn. Run `devforgeai trust pin --framework <framework root>` in a terminal outside Claude Code, then start a new session.",
  "systemMessage": "Gate      TRUST FAIL  DFA-E501\nBlocked   you: run devforgeai trust pin --framework <framework root> outside Claude Code\n\nNo gate ran for this turn."
}
```

With `stop_hook_active` true it exits 0 carrying the `systemMessage` alone, so the harness's continuation ends and the user is left holding the refusal rather than a loop. `[last_gate].result` takes `TRUST_FAIL` in both branches, which is what FWK-013 names as the weak signal — it is kept as a record and is not the channel the refusal depends on.

**3. `UserPromptExpansion`.**

```json
{
  "matcher": "explore|discover|constitute|plan|build|verify|release|design|reflect",
  "hooks": [
    {
      "type": "command",
      "command": "@@DEVFORGEAI@@",
      "args": ["hook", "run", "prompt-expansion"],
      "timeout": 30
    }
  ]
}
```

The matcher is the nine skill `name:` values, which are the nine slash names before and after the §4 collapse. The `prompt-expansion` arm runs `trust verify`, then `gate require <phase> <id>` with the phase taken from the matched command name and the id parsed from the prompt's first argument. On either failure it exits 2 with `{"decision":"block","reason":"<the diagnostic, plus the command that repairs it>"}`. On a pass it exits 0 and emits nothing, and the expansion proceeds into the skill.

Two consequences worth stating in the spec: the refusal reaches the user before the skill body enters context, so a gate that will fail costs no tokens (HOOK-090); and `gate require` now runs in two places for the same phase — here and in the skill's `!` preamble — which is deliberate, because the preamble is the one that survives a session where the user pastes the skill invocation rather than typing the slash name. The two calls are idempotent reads.

**4. `init`.** The closing `Next` line becomes conditional: when `trust verify` fails at the end of `init`, `init` prints `Next       devforgeai trust pin --framework <framework root>, in a terminal outside Claude Code` instead of `Next       /explore`, and repeats it as a warning line. This is the FWK-011 and FWK-052 fix.

**What the user sees.** On an unpinned binary: typing `/build STORY-003` produces a refusal naming the pin command, and the skill does not load. If the session started before the pin was broken, the first write produces a permission denial carrying the same text, and the turn end produces the block plus the `TRUST FAIL` line. Three surfaces, one message, and no path that advances a phase.

**Spec edits.** `specs/00-conventions.md` §8 bullet 2 becomes: *a trust failure blocks on `PreToolUse`, `Stop`, `SubagentStop`, and `UserPromptExpansion`, through exit 2 and a decision object; on `SessionStart` and `PostToolUse` no exit code blocks and the hook emits `systemMessage` so the failure is visible.* `specs/01-cli.md:1489` and the table at `:1563-1568` take the same four-event list plus the new `trust-check` and `prompt-expansion` arms.

**Test first**: `trust_check_denies_bash_on_failure` (exit 2, `permissionDecision` deny, matcher covers `PowerShell`); `stop_trust_failure_blocks_with_pin_command`; `stop_trust_failure_with_active_flag_exits_zero`; `prompt_expansion_blocks_on_missing_predecessor_gate`; `prompt_expansion_passes_silently_when_gate_passes` (exit 0, empty stdout); `init_next_line_names_pin_when_untrusted`.

### FWK-040 through FWK-044 — the keep-or-collapse memo

**Decision: collapse.** `commands/` is deleted; each SKILL.md carries the frontmatter and the preamble.

**Grounds.**

1. A skill and a command file of the same name both produce `/name`, and the skill wins. The two-file shape therefore has one live file and one shadow file for every entry point, and the shadow is the one a builder edits when they think they are editing the command (FWK-040).
2. The preamble is the gate. A `!` command inside SKILL.md runs before the model sees the content, and a non-zero exit aborts the whole invocation — which is exactly the gate behaviour `gate require` is written for, and it is the same behaviour in either file. Nothing is lost by moving it.
3. `disable-model-invocation: true` is available on a skill and not on a command file. Without it, nine full paragraphs of description sit in every session's context, and the model can start a side-effecting phase on its own judgment (FWK-041). This is the capability that the collapse buys and the keep option cannot.
4. Frontmatter `hooks` are available on a skill and not on a command file, which is where the phase-scoped guards of HOOK-092 would live.
5. The slash names are preserved by `name:`, so the migration changes no user-facing string, no `cases.jsonl` prompt, and no session-mining constant.

**Grounds against, weighed.** The command file is the one place a reader sees the entry point's arguments without opening a 300-line skill. That is answered by the `argument-hint` frontmatter field and the `## Entry` section, both of which already carry the arguments in each SKILL.md.

**Exact frontmatter for one collapsed skill** — `skills/implementing-stories/SKILL.md`:

```yaml
---
name: build
description: <the current description, unchanged>
argument-hint: STORY-nnn [--remedy FIND-nnn,...] [--resume]
allowed-tools: Bash(devforgeai:*), PowerShell(devforgeai:*), Read, Write, Edit, Grep, Glob, Agent
disable-model-invocation: true
---

!`devforgeai gate require build $1`
!`devforgeai doc load story $1`
!`devforgeai doc load context all`
!`devforgeai doc load sprint -`

# Implementing stories
```

`name: build` rather than a directory rename: the directory name is what `produced_by: implementing-stories` in every template resolves against and what the producer check reads, while `name:` decides the slash command and the Skill-tool target. The `PowerShell(devforgeai:*)` grant is FWK-014. The `gate require` line leads, so a failing gate aborts the invocation before any document loads.

**The flag set.** The seven phase skills take `disable-model-invocation: true`: `exploring-ideas`, `discovering-requirements`, `establishing-context`, `planning-work`, `implementing-stories`, `validating-quality`, `releasing-software`. `improving-framework` takes it on the same reasoning — no skill invokes it through the Skill tool and it is user-started — which makes eight rather than the seven the brief names; the departure is recorded here rather than assumed. `designing-interfaces` omits the flag, because `exploring-ideas` step 6 and `planning-work` step 6 invoke it through the Skill tool, and a skill the model cannot invoke cannot be reached that way.

**Every file and code path that changes.** Steps 1 to 8 of `AUDIT-3-skills.md` SKL-024 cover the nine SKILL.md files, the preamble moves, the deletion of `commands/`, `cli/src/cmd/init.rs`, `evals/runner/run_jsonl.py`, the two cross-skill Skill-tool call sites, and `specs/00-conventions.md` §3 and §4b. The following are the additional points:

- `evals/runner/run_jsonl.py:46` `SKILL_COMMANDS` and `:231-232`: the map and the copy are deleted. The runner's workspace build then installs `skills/<name>/` minus `evals/` and `agents/`, and the case prompt `"/build STORY-014"` resolves through the skill's `name:`.
- `cli/src/cmd/init.rs:12`: `COPIED` becomes `&["skills", "agents"]`. `:117-120`: the summary line drops the commands count.
- `cli/tests/init.rs:137` `init_creates_directory_tree` and `:228` `init_json_envelope_lists_counts`: both assert a `.claude/commands` tree and a commands count; both change.
- `cli/src/aggregate.rs:18-30` `SLASH_COMMANDS`: unchanged, and the reason is `name:` (FWK-044).
- `specs/00-conventions.md:18` rule 4: "One skill per phase ... plus two cross-cutting skills. Each skill is its own entry point." `:57`: the layout line for `commands\<name>.md` is deleted. `:106-110` §4b: the heading stays and the "Command file shape" paragraph becomes the SKILL.md frontmatter and preamble description above. `:247` acceptance criterion (a): `## Command` stays in the H2 list per §11 and now describes frontmatter plus preamble.
- `specs/BUILD-BRIEF.md:29`: the `commands/<command>.md` output line is deleted. `:51`: "the command file is copied byte-identical" becomes the frontmatter rule. `:72`: the verification grep drops `commands/<command>.md` from its file list.
- `README.md:22`: the layout line for `commands\<name>.md` is deleted. `:38-49`: the installed-tree block drops `commands` from `.claude\{skills,agents,commands}\`. `:36`: the `init` paragraph drops "and commands". The phase-command table at `:55-63` is unchanged — the names survive.
- `specs/11-subagent-catalog.md:1924`: "The nine command files are specified in the nine skill specs" becomes "The nine entry points are the nine skills' own frontmatter". `:1726`: the `recommendation-drafter` input list drops `commands/`.
- `specs/10-reflect.md:476` and the target-kind table it feeds; `skills/improving-framework/references/recommendations.md:11`, `:25`, `:29`; `skills/improving-framework/SKILL.md:32`, `:52`; `skills/improving-framework/templates/rec-targets.md`: the seven-row target table loses its `command` row and becomes six. Sequenced with FWK-070, which rewrites the same paths for a different reason.
- The nine spec `## Command` sections (`specs/02-explore.md` through `specs/10-reflect.md`): each becomes the collapsed frontmatter block plus the preamble lines, keeping the H2 that conventions §11 fixes.
- `hooks/settings.hooks.json`: the `UserPromptExpansion` matcher of the trust fix is the nine `name:` values, which are the nine command names, so the two answers do not collide.
- Permission rules: after the collapse an entry point is addressed as `Skill(build)` rather than as a command path, and `skillOverrides` can hide one without editing it. Nothing in the framework ships such a rule today; recorded so that a future permissions block is written in the skill vocabulary.
- The 46 agent `description` lines that name a slash command are valid before and after (FWK-044); AGT-009 removes them for its own reason.

### FWK-050, FWK-051, FWK-052 — the CLAUDE.md block `init` appends

**The block**, 33 lines between markers, written verbatim:

```markdown
<!-- devforgeai:begin -->
## DevForgeAI

Delivery runs as phases. Each phase is one skill with one slash command, writes one
document under `.devforgeai/`, and ends with a handoff block naming the next command.

| Command | Phase | Writes |
|---|---|---|
| `/explore "<idea>"` | 0 Explore | `explore/brief.md`, `explore/decision.yaml` |
| `/discover <IDEA-nnn>` | 1 Discover | `requirements.yaml` |
| `/constitute <IDEA-nnn>` | 2 Constitute | `context/*.md`, `adr/ADR-nnn.md` |
| `/plan <EPIC-nnn>` | 3 Plan | `stories/STORY-nnn.md`, `stories/sprint.yaml` |
| `/build <STORY-nnn>` | 4 Build | source and tests, `reports/STORY-nnn-build.yaml` |
| `/verify <STORY-nnn>` | 5 Verify | `reports/STORY-nnn-qa.yaml` |
| `/release <vX.Y.Z>` | 6 Release | `releases/vX.Y.Z.yaml` |
| `/design <id>` | any | `brand/tokens.json`, `ui-specs/UI-nnn.md` |
| `/reflect <window>` | any | `reports/reflect-<date>.yaml` |

The handoff comes from the Stop hook at the end of each phase. Its `Next` line is the
command to type next, copied as printed, arguments included.

Phase documents live under `.devforgeai/`. Source and tests live where
`.devforgeai/config.toml` records the project's roots.

Gates are the `devforgeai` binary and the Claude Code hooks that call it, not
instructions in this file: a write to `.devforgeai/` is checked as it happens, a phase
ends when its gate passes, and a failing gate holds the turn open.

A defect found in an upstream document is cited, not edited. The handoff prints the
send-back command that reopens the cited ids.
<!-- devforgeai:end -->
```

Nothing in the block restates a hook's rule, names a language or a test runner, or tells the model to check itself: the hooks do the checking and the block says that they do. The nine commands are in order with one line each, the handoff's origin and the copy-the-`Next`-line rule are stated once, and the document locations are one sentence.

**The `init` change.** `cli/src/cmd/init.rs`:

- A `const CLAUDE_MD_BLOCK: &str` holding the text above, and `const CLAUDE_MD_BEGIN: &str = "<!-- devforgeai:begin -->"`, `const CLAUDE_MD_END: &str = "<!-- devforgeai:end -->"`.
- `fn append_claude_md(root: &Path) -> Result<(), CliError>`, called from `run` immediately after `append_gitignore(&root)?` at line 85, following that function's shape: read `CLAUDE.md` at the project root (absent reads as empty); when `CLAUDE_MD_BEGIN` is present, replace the span from `CLAUDE_MD_BEGIN` through `CLAUDE_MD_END` inclusive with the current block and write; when it is absent, append a blank line and the block; write through `project::atomic_write`. An absent `CLAUDE.md` is created holding the block alone with a leading `# <project directory name>` H1.
- The `created` vector gains `CLAUDE.md` when the file did not exist, so `--json` `created` reports it.
- The human summary gains one line between the `Hooks` and `Stack` lines: `CLAUDE.md  devforgeai section <written|updated>`.
- `--force` is not consulted: the replace-between-markers path is idempotent and touches no text the user wrote outside the markers, which is why this differs from the `DFA-E110` guard on `.devforgeai/`.

**Tests** (`cli/tests/init.rs`, in the style of `init_appends_gitignore_lines`):

- `init_creates_claude_md_when_absent` — no `CLAUDE.md` in the fixture; after `init`, the file exists, holds both markers once each, and its line count between the markers is 33.
- `init_appends_block_to_existing_claude_md` — a `CLAUDE.md` holding two lines of the user's own text; after `init`, both lines survive byte for byte and the block follows.
- `init_twice_replaces_between_markers` — run `init --force` twice; the file holds one `devforgeai:begin` and one `devforgeai:end`, and the user's text outside the markers is unchanged.
- `init_claude_md_block_names_nine_commands` — the block between the markers holds each of the nine slash names once.
- `init_json_envelope_lists_claude_md` — `created` holds `CLAUDE.md` on the first run and omits it on the second.
- `specs/01-cli.md` `## Workflow` init step and `## Outputs` file table gain the `CLAUDE.md` row; `README.md:36` gains the append to its list of what `init` does (FWK-075).

### FWK-080 through FWK-084 — scoping the ceremony pattern

**The rule**, replacing the parenthetical at `specs/00-conventions.md:28` and stated once so the grader and the build brief reference it rather than restating it:

> The pattern applies to instruction prose in a SKILL.md, a command preamble, a subagent file, or a spec's workflow and template sections — text whose reader is the model executing it. A line is instruction prose when none of the following holds, tested in this order:
>
> 1. The line is inside a fenced block, at any fence depth.
> 2. The line is inside YAML frontmatter, or is a YAML or JSON scalar, key, or enum value.
> 3. The token that matched lies inside a backticked span.
> 4. The token that matched lies inside a table cell whose column holds enum values, identifiers, paths, or file names, as declared by that table's header row.
> 5. The token that matched is part of a path segment or a file name.
> 6. The line is a heading.
>
> The pattern is case-sensitive over the single-word alternatives and case-insensitive over the multi-word phrases. The single-word forms are emphasis when capitalised; the ordinary-English lower-case uses are not what the rule is for.
>
> The pattern does not apply to a document the framework produces about a target project — the six context files, an ADR, a requirements record, a story, a UI spec, a report. Those state the project's own rules, and an absolute rule is what a project is entitled to write.

**The grader.** `skills/establishing-context/evals/graders.py:480-507` `no_ceremony`: the walk over `.devforgeai/` is deleted and the function is deleted with it, because clause 7 removes its entire subject. The case that calls it takes a grader that checks what the phase is accountable for instead — that each of the six context files holds its declared H2 set and that each rule line carries a `CON-nnn` — which is already what the neighbouring graders in that file do. `specs/04-constitute.md` `## Evals` takes the same change, and the `CEREMONY` constant at `:37-40` is deleted.

**The review script.** `specs/BUILD-BRIEF.md:72`'s grep gains the clause-1 and clause-2 exclusions, which a line-oriented grep cannot express, so the line becomes a call to one script rather than an inline pattern: `python scripts/ceremony_scan.py skills/<name> agents/<each agent>`, with the script implementing clauses 1 to 6 and the case rule and printing `file:line token` for each hit. One definition, referenced from `specs/00-conventions.md`, `specs/BUILD-BRIEF.md`, and any grader that needs it (FWK-084).

**The enum.** With clause 2 in place, a `priority:` value is a YAML scalar and matches nothing, whichever of the four MoSCoW words it holds. Restore MoSCoW per the recommendation in §8, with the file list and the two pinned digests recorded there.
