---
schema: devforgeai-audit/1
area: skills
produced_by: audit-skills
---

# Audit 3 — the nine skills and their command files

Scope: `skills/<nine>/{SKILL.md,references/,templates/,agents.md}` and `commands/*.md`, read against
`specs/00-conventions.md`, `specs/BUILD-BRIEF.md`, `specs/ANTHROPIC-GUIDANCE.md` §3 §5 §6 §7, and
`specs/02-explore.md` through `specs/10-reflect.md` sections `## Workflow`, `## Command`, `## Templates`,
`## Send-back`, `## Integration`. Mechanical diffs were run with a script that extracts each spec's
fenced and four-space-indented template blocks and compares them byte for byte against the built files.

Counts: 0 blocker, 4 high, 14 medium, 38 low (56 findings).

## 1. Spec fidelity

Mechanical result first. 9 of 9 `commands/<name>.md` match their spec's `## Command` block byte for byte.
43 of 44 files under `skills/*/templates/` match their spec's `## Templates` block byte for byte after
dedenting four spaces. No template file exists that no spec defines; no spec template is missing a file.
All nine SKILL.md carry the seven BUILD-BRIEF §2 H2 sections, in order, with no extra section.

| ID | Severity | Skill | Location | Spec or guidance says | What exists | Failure scenario |
|---|---|---|---|---|---|---|
| SKL-001 | high | improving-framework | `skills/improving-framework/templates/reflect-report.yaml` lines 36-76 | `specs/10-reflect.md` `### templates/reflect-report.yaml` is one fenced block ending at line 559 with `---`; the second block at 562-601 is prose-introduced ("as the shapes to copy") and is not the file. BUILD-BRIEF §1: templates are the spec's blocks byte-identical | The second block is appended to the file after the closing `---`, so the shipped template carries `observations`, `recommendations` and `technical_debt` twice: once as `[]`/`groups: []` inside the fence and once as populated shapes after it | The skill's step 9 says "write from `templates/reflect-report.yaml`, with the twelve top-level keys". A copy of the file as shipped is a two-document YAML stream whose second document duplicates three keys. `doc validate` on `reports/reflect-<date>.yaml` rejects the extra top-level keys, and the step-9 rewrite loop has no key to fix because the defect is structural |
| SKL-002 | medium | improving-framework | same file, lines 1 and 35 | `specs/10-reflect.md` `## Outputs` prints the same document with no `---` fence; conventions §5 says YAML documents put the seven keys at the top level. The other five YAML templates (`release.yaml`, `decision.yaml`, `sprint.yaml`, `build-note.yaml`, `requirements.yaml`) carry no fence | `reflect-report.yaml` is wrapped in `---` … `---` | A `---`-wrapped YAML file is a document with a directives-end marker; whether `doc validate` reads the keys as top-level or as a frontmatter block is undecided by any spec. The fence came from the spec's own `## Templates` block, so the build is faithful and the spec is self-contradictory |
| SKL-003 | medium | exploring-ideas | `skills/exploring-ideas/SKILL.md:66` | `specs/02-explore.md` step 9 fixes a second `AskUserQuestion(...)` block verbatim for the `Park` follow-up: question text `"IDEA-001: when do you look at this again?"`, header `"Revisit"`, three options with their `description` strings | One sentence of prose: "`Park` is followed by a second question offering `<d+30>`, `<d+90>`, and `<d+180>` … described as thirty days out, ninety days out one quarter, and six months out." No `references/` file carries the block | The `Revisit` header and the question text are lost. `decision.yaml` `revisit_on` is filled from an answer the model composes freshly each run, so two Park decisions produce two different question shapes and the eval fixture that keys on the header does not match |
| SKL-004 | medium | planning-work | `skills/planning-work/SKILL.md:39-53` steps 5, 7, 9, 11 | `specs/05-plan.md` steps 5, 7, 9 each carry "Failure path: the JSON does not parse against the schema … the subagent is invoked once more with the parse error appended, and a second failure leaves the gate metric absent, which `gate check` reports as exit 1." BUILD-BRIEF §2: each workflow step keeps its actor, input, output and failure path | Steps 5, 9 and 11 carry no parse-failure path. Step 7 carries an `overlaps`/`unplaceable` re-invocation, which is a different condition. The rule sits in `skills/planning-work/agents.md:16`, and `## References` schedules no read of `agents.md` | A `story-decomposer` returning malformed JSON has no stated recovery in the standing instructions. The model either retries indefinitely or writes stories from a partial parse; the gate then fails on a metric the run had no instruction to protect |
| SKL-005 | low | implementing-stories | `skills/implementing-stories/SKILL.md:49-53` steps 7.3-7.6 | `specs/06-build.md` 7.3 and 7.6: "Failure path: exit 1 from the `pre-commit` hook's `doc validate` or `context audit`; the run stops with one `Blocked` line carrying the hook's stderr." 7.4: "Failure path: the JSON carries `blocked` with a reason from its enum; the loop stops and the run continues at step 12" | The SKILL.md bullets for 7.3, 7.4, 7.5 and 7.6 carry no failure path | Mitigated: `references/tdd-cycle.md:39` and `:58` carry both paths, and the `## References` line schedules the read before the first criterion of step 7 |
| SKL-006 | low | validating-quality | `skills/validating-quality/SKILL.md:94-99` step 5 | `specs/07-verify.md` step 5: "A subagent with more than twenty items reports the first twenty by descending severity and sets `truncated` to `true`" | The band arithmetic is kept; the twenty-item rule is dropped from the step | Mitigated: `references/findings-and-deferrals.md:14` carries it and the `## References` line schedules the read before step 8 |
| SKL-007 | low | exploring-ideas | `skills/exploring-ideas/SKILL.md:70` step 11 | `specs/02-explore.md` step 11 failure path ends "and the gate's `prototype-pruned` check fails on the next Stop" | The SKILL.md keeps the three error codes and drops the gate consequence | The model reads an exit-1 prune as a local problem rather than as the cause of the FAIL handoff it is about to see |
| SKL-008 | medium | releasing-software | `skills/releasing-software/SKILL.md:55` step 10 and `:76` Subagents table | `specs/09-release.md` step 10: `guide-writer` takes "the four context H2 sections `## Languages`, `## Layers`, `## Approved dependencies`, `## License policy`" | Step 10 names six sections (`## Languages`, `## Runtimes`, `## Layers`, `## Approved dependencies`, `## License policy`, `## Constraint index`); the Subagents table in the same file says "the five context H2 sections" | Three different counts for one prompt field, two of them inside the same SKILL.md. `guide-writer` receives whichever the model resolves, and the architecture note's content varies per run |
| SKL-009 | low | releasing-software | `skills/releasing-software/SKILL.md:64` step 13 | `specs/09-release.md` step 13 says "the eight check entries"; the same spec's `## Gate` defines ten `[[gate.check]]` ids (`release-docs`, `release-file`, `release-status`, `release-set`, `release-entries`, `release-ids`, `release-stories`, `release-deferrals`, `release-manifest`, `release-docs-cover`) | The skill says "ten check entries" and "three of the ten gate checks" | The skill is correct and `specs/09-release.md` step 13 prose is stale. Recorded so the spec is corrected rather than the skill |
| SKL-010 | low | designing-interfaces | `skills/designing-interfaces/SKILL.md:82` step 7 | `specs/08-design.md` step 7: the contrast adjustment is "recorded … in a `notes` field the model writes into `brand-kit.md` `## Do not`" | The skill names "the `adjustments[]` array". `agents/brand-designer.md` `## Output` makes `adjustments` a required array of `{token, from, to, reason}`, and its workflow step 5 reads "Record the move in `adjustments[]` … The invoking skill writes those entries into `brand-kit.md` `## Do not`" | The skill and the agent agree; `specs/08-design.md` step 7 is the stale text. Same shape as SKL-009: the fix lands on the spec, not on the skill |
| SKL-011 | low | discovering-requirements | `skills/discovering-requirements/SKILL.md:88` | `specs/03-discover.md` `## Send-back` says "from entry point A only"; the same spec's step 4 says "entry point A and the resume form" | The skill says "from entry point A and the resume form only" | The skill follows the spec's step over the spec's summary, which is the correct reading (the resume form re-enters at step 4). The spec's `## Send-back` line is what needs the edit |
| SKL-012 | low | all nine | `skills/*/references/` | BUILD-BRIEF §1: one reference file per topic the spec's `## Workflow` defers detail to | 39 reference files across nine skills. Every deferred topic traced to a file: explore 5/5, discover 4/4, constitute 4/4, plan 4/4, build 8/8, verify 3/3, design 3/3, release 5/5, reflect 3/3 | One gap only, SKL-003: the Park follow-up question block is deferred by omission and lands in no reference file |

## 2. Ceremony and ambiguity

| ID | Severity | Skill | Location | Spec or guidance says | What exists | Failure scenario |
|---|---|---|---|---|---|---|
| SKL-013 | low | all nine | `skills/*/SKILL.md`, `skills/*/references/*.md`, `skills/*/agents.md`, `commands/*.md` | Conventions §2 ceremony pattern, applied case-insensitively to prose lines | Zero matches across 9 SKILL.md, 39 reference files, 9 agents.md and 9 command files. Zero matches for the §2 ambiguity terms (`as appropriate`, `as needed`, `if applicable`, `etc.`, `and so on`, `or similar`, `such as`). One `should` in the corpus, at `skills/establishing-context/SKILL.md:3`, inside a description clause ("where a … record should live") where `should` is the user's phrasing, not a softened instruction | No failure. Recorded as the cleared baseline the softer scans below sit against |
| SKL-014 | low | validating-quality, implementing-stories, releasing-software | `validating-quality/SKILL.md:133` "Draft and validate the deferrals"; `implementing-stories/SKILL.md:58` "Validate the context"; `releasing-software/SKILL.md:66` "Validate the release file" | BUILD-BRIEF §3 bars "verify", "confirm", "ensure", "make sure" as instructions to the model | Three step titles use `validate` as the verb. Each names a real actor: step 9 names the `deferral-validator` subagent, step 58 names `context-validator`, step 66 names `devforgeai doc validate <path>` | The titles read as self-checks on a scan. The step bodies name the CLI call or the subagent, so the model does not perform a check of its own. Borderline; a title naming the actor ("Run `doc validate` on the release file") removes the ambiguity |
| SKL-015 | low | planning-work, establishing-context, designing-interfaces, releasing-software | `planning-work/SKILL.md:55` and `:119`; `establishing-context/SKILL.md:146`; `designing-interfaces/SKILL.md:108`; `releasing-software/SKILL.md:60` | Conventions §2 forbids "status-transition instructions ('mark the story as')"; BUILD-BRIEF §3 repeats it as "no status transitions (`phase set` does that)" | Plan step 12 and R4 instruct writing `status: ready` into story frontmatter and `sprint.yaml`; Constitute step 12 instructs editing `status: accepted` into six frontmatters and every ADR; Design step 14 instructs moving files to `status: approved`; Release step 12 instructs editing `status: released` | Each is assigned to the model by its own spec (`specs/05-plan.md` step 12, `specs/04-constitute.md` step 12, `specs/08-design.md` step 14, `specs/09-release.md` step 12), and each writes a document status the skill owns rather than a `state.toml` phase. The conventions §2 rule and the four specs disagree; the skills followed the specs |
| SKL-016 | low | all nine | `skills/*/SKILL.md` | BUILD-BRIEF §3: no instruction to run tests, coverage or linters and interpret the numbers | No skill interprets a number it measured. `implementing-stories` step 6 reads the test command from `devforgeai config get stack.test_command` and states the reason ("Reading it from the CLI is what keeps a runner name out of this skill"); step 7.7 reads `build-lint` and `build-complexity` through `report show --check`, which are CLI-written entries | No failure. The one place a raw exit code is read (7.2, 7.5) is the red-green signal, which the spec assigns to the model |

## 3. Frontmatter against the skills reference

Measured description lengths, all under the 1,536-character truncation point of guidance §3:
designing-interfaces 875, discovering-requirements 723, establishing-context 864, exploring-ideas 834,
implementing-stories 876, improving-framework 889, planning-work 865, releasing-software 803,
validating-quality 957. Total 7,686 characters carried in every request's context under the current
frontmatter.

| ID | Severity | Skill | Location | Spec or guidance says | What exists | Failure scenario |
|---|---|---|---|---|---|---|
| SKL-017 | high | eight of nine (not designing-interfaces) | `skills/*/SKILL.md` frontmatter | `ANTHROPIC-GUIDANCE.md` §3 invocation matrix: `disable-model-invocation: true` is "user only, description not in context (zero cost until invoked; recommended for workflows with side effects)". §7.8: "Phase skills that only the user should start get `disable-model-invocation: true`; skills another skill invokes (designing-interfaces from exploring-ideas) stay model-invocable." The guidance preamble: "Where our framework contradicts a statement, the statement wins" | No SKILL.md carries the key. BUILD-BRIEF §2 says "No other frontmatter keys" beyond `name` and `description`, which is the contradicting framework statement | Two consequences. (a) 7,686 characters of description sit in every request. (b) Every phase skill is model-invocable, so a model that reads "the user wants to plan this epic" can fire `planning-work` directly, and the `!` preamble of `commands/plan.md` — `gate require plan`, `doc load requirements`, `doc load context all` — does not run. The skill body then asserts "the command preamble has already run three CLI calls, so their stdout is in context" and the model works from an empty context with no fallback stated. The predecessor gate survives only because step 4 runs `phase set plan`, which refuses with `DFA-E320`; by then the run has read a requirements file it did not load |
| SKL-018 | medium | all nine | `skills/*/SKILL.md` frontmatter `description` first sentence | `ANTHROPIC-GUIDANCE.md` §3: "combined `description` + `when_to_use` is truncated at 1,536 characters; put the key use case first." The cited good example opens with the action ("Summarizes uncommitted changes and flags anything risky") | Eight of nine open with framework identity: "Phase 0 of DevForgeAI.", "Phase 1 of DevForgeAI.", … "Cross-cutting Reflect phase of DevForgeAI, run by /reflect." The use case reaches the reader in the second clause. `designing-interfaces` opens with "The DevForgeAI design capability, run by /design in three modes", which is the same shape | Under `disable-model-invocation: true` this stops mattering, because the description leaves the request context. While the skills stay model-invocable, the trigger signal sits behind a phase number the model has no reason to match against a user's phrasing |
| SKL-019 | medium | all nine | `skills/*/SKILL.md` frontmatter | `ANTHROPIC-GUIDANCE.md` §3: `allowed-tools` is a "grant for the invoking turn only; clears on the next user message". Conventions §4b: the command file "lists `Bash(devforgeai:*)` so the preamble can run" | The grant lives in `commands/*.md` alone. `commands/build.md` carries `Bash(devforgeai:*), Read, Write, Edit, Grep, Glob, Agent`; `commands/explore.md` adds `AskUserQuestion, Skill, WebSearch, WebFetch` | Two paths. On the `/build` path the grant is correct. On the direct-skill path of SKL-017 there is no grant, so every `devforgeai` call inside the workflow prompts, and a `-p` eval run denies them. Contingent on question 4: a collapse moves the key into SKILL.md and closes both paths |
| SKL-020 | low | all nine | `skills/*/SKILL.md` frontmatter | `ANTHROPIC-GUIDANCE.md` §3 lists `argument-hint` and `arguments` as skill frontmatter | Neither is declared on any SKILL.md; `argument-hint` is declared on all nine command files and matches the `## Entry` table of each skill | Under keep this is correct placement. Under collapse both move into SKILL.md; `arguments` (named positional) would let `$1` be declared rather than described in prose, which is how every `## Entry` table currently reads |
| SKL-021 | low | designing-interfaces, improving-framework, implementing-stories | `skills/*/SKILL.md` frontmatter | `ANTHROPIC-GUIDANCE.md` §3: `context: fork` is a skill frontmatter field | Not used by any skill | Three steps would gain. `designing-interfaces` sketch mode writes up to fifteen wireframe HTML files inside Explore's conversation; a fork keeps that out of the Explore context that later runs the kill case. `improving-framework` reads an aggregate plus session JSONL and returns one report. `implementing-stories` step 11 already achieves isolation through a subagent, so a fork there would duplicate it |
| SKL-022 | low | implementing-stories, establishing-context, designing-interfaces | `skills/*/SKILL.md` frontmatter; `hooks/settings.hooks.json` | `ANTHROPIC-GUIDANCE.md` §3: "Hooks in skill frontmatter register when the skill is invoked and persist for the session". §7.9: "Skill-frontmatter `hooks` can register phase-scoped hooks (for example Build's file-set guard) when `/build` is invoked, instead of every hook being global" | `hooks/settings.hooks.json` registers all five events globally: SessionStart `*`, PreToolUse `Write|Edit`, PostToolUse `Write|Edit`, PostToolUse `Bash`, Stop `*`, SubagentStop `*` | Build's `story files --check` guard, Constitute's `context audit`, and Design's `design lint` fire in every session including ones that run no phase. Phase-scoped registration is the guidance's own worked example and is unused |
| SKL-023 | low | implementing-stories | `hooks/settings.hooks.json` PostToolUse matcher `"Bash"` | `ANTHROPIC-GUIDANCE.md` §7.6: "Shell-command matchers are `Bash|PowerShell`" (§2: "On Windows where the PowerShell tool is primary, a `Bash` matcher never fires") | The matcher is the bare string `"Bash"` | On a PowerShell-primary session the `gate check --phase build --partial` hook does not fire, so no partial report exists and `implementing-stories` step 7.7 takes its `DFA-E400` branch on every criterion. The stated recovery ("running the command once more makes the hook write it") does not succeed, and the run loops. The hooks file is another auditor's area; recorded here because it decides whether a documented step of this skill terminates |
| SKL-054 | low | establishing-context | `commands/constitute.md` frontmatter `allowed-tools` | Conventions §10 applies least privilege to subagent tool lists; `ANTHROPIC-GUIDANCE.md` §3 makes `allowed-tools` a grant for the invoking turn | The list carries `Skill`. A grep of `skills/establishing-context/SKILL.md` finds no skill invocation at any of steps 1 to 13 or R1 to R6; the phase's three subagents are reached through `Agent` | An unused grant. The file is byte-identical to `specs/04-constitute.md` `## Command`, so the edit lands on the spec, as in SKL-009 |
| SKL-055 | low | exploring-ideas | `commands/explore.md` frontmatter `allowed-tools` | Same | The list carries `WebSearch, WebFetch`. The only searching step is step 3, which delegates to `landscape-scanner`, and `agents/landscape-scanner.md:4` carries `tools: [WebSearch, WebFetch, Read]` — a subagent's tools come from its own frontmatter, not from the invoking turn's grant | An unused grant that widens the turn's tool surface past what any step of the skill performs. Byte-identical to `specs/02-explore.md` `## Command`, so the edit lands on the spec |
| SKL-056 | medium | exploring-ideas | `agents/idea-interrogator.md:4` | `ANTHROPIC-GUIDANCE.md` §2: `AskUserQuestion` is "removed from every subagent regardless of `tools`"; §7.1: "Any agent file listing `AskUserQuestion` in `tools` is broken by construction; the question moves to the skill" | `tools: [Read, AskUserQuestion]` | `exploring-ideas` step 2 invokes `idea-interrogator` and reads back `problem_statement`, `holders`, `today`, `why_now`, `weak_signals`, `open_questions`. An agent written to ask the user for the material it cannot infer has no way to ask, so it returns guesses or an empty `holders` array — which step 2 treats as "the idea has no one to sell to" and jumps to step 8 with `## Core flows` empty. The agent file is another auditor's area; recorded here because it decides whether the phase's first step produces a brief |

## 4. Commands merged into skills

Recommendation: **collapse**. The nine skill directories are model-invocable today and each already
creates a second entry point of its own name beside the thin command, so the framework ships eighteen
invocation paths for nine capabilities and half of them skip the `gate require` preamble the design
treats as the gate.

| ID | Severity | Skill | Location | Spec or guidance says | What exists | Failure scenario |
|---|---|---|---|---|---|---|
| SKL-024 | high | all nine | `skills/*/` directory names and `commands/*.md` | `ANTHROPIC-GUIDANCE.md` §3: "`.claude/commands/deploy.md` and `.claude/skills/deploy/SKILL.md` both create `/deploy`". §7.8: "the skill directory name already makes `/explore`". Conventions §1 rule 4: "One thin slash command per skill"; §4b fixes the nine command names | `skills/exploring-ideas/` creates `/exploring-ideas` and `commands/explore.md` creates `/explore`. The same holds for all nine, so the installed project carries `/explore` and `/exploring-ideas`, `/build` and `/implementing-stories`, and so on. Only the nine `commands/*.md` entry points carry the `!` preamble | The eighteen entry points are not equivalent. `/implementing-stories` runs no `gate require build`, loads no story, no context set and no sprint, and the skill's `## Entry` asserts all four already ran. The gate is not bypassed outright — step 5's `phase set build` refuses with `DFA-E320` — but the run reaches step 5 having opened a git worktree on a story whose plan gate failed. `/design` and `/designing-interfaces` are the worst pair, because Design runs no `phase set` at all and nothing downstream catches the missing preamble |
| SKL-025 | medium | planning-work, exploring-ideas | `planning-work/SKILL.md:43` and `:117`; `exploring-ideas/SKILL.md:42` | Conventions §8: every skill names the documents and skills it relates to explicitly. The Skill tool resolves a skill by name from the skill listing | `planning-work` step 6: "Through the Skill tool, call `/design STORY-nnn --spec`"; R3: "`send_to_design` calls `/design UI-nnn --remedy AC-nnn,...`". `exploring-ideas` step 6: "invoke the `designing-interfaces` skill in sketch mode" | Two names for one target. `/design` today resolves to `commands/design.md`, whose body is "Invoke the designing-interfaces skill for $ARGUMENTS" — so the Plan path is an extra hop through a command file, and the Explore path reaches the skill directly. A collapse that renames the directory to `design` breaks the Explore call; a collapse that keeps `designing-interfaces` and sets `name: design` satisfies both, because the Skill tool resolves the frontmatter `name` |
| SKL-026 | low | all nine | `cli/src/cmd/init.rs:12`, `:117-120`; `evals/runner/run_jsonl.py:45`, `:231-232`; `cli/src/aggregate.rs:18-30` | Conventions §3 repository layout lists `commands\<name>.md` as a shipped artifact | `init.rs` line 12: `const COPIED: &[&str] = &["skills", "agents", "commands"];` and a summary line that prints the command count. `run_jsonl.py` builds a `SKILL_COMMANDS` map and copies `commands/<command>.md` into each eval workspace. `aggregate.rs` `SLASH_COMMANDS` pins the nine names `explore` … `reflect` and is what Reflect's `repeated_send_back` observations are mined from | These are the three breakage points a collapse touches. `aggregate.rs` constrains the migration rather than breaking under it: the collapsed skill has to keep producing `/explore` and not `/exploring-ideas`, or session mining goes blind and `improving-framework` steps 4 and 5 return empty |

## 5. Dynamic context injection

Exit-code contract read from `specs/01-cli.md`: `doc validate --allocate` exits 1 on `DFA-E215`;
`gate require` exits 1 on `DFA-E320`/`DFA-E321`; `doc load` exits 1 on `DFA-E200` and 3 on `DFA-E250`;
`report show` exits 1 on `DFA-E400`/`DFA-E401`; `story list` exits 0 at a count of zero and 1 on
`DFA-E231`; `report aggregate` exits 3 on `DFA-E430` and 1 on `DFA-E421`. `doc load discover-entry` is
the one call specified to exit 0 in every case, and `specs/03-discover.md` says so explicitly — the
spec authors reasoned about abort-on-non-zero for `/discover` and for no other command.

| ID | Severity | Skill | Location | Spec or guidance says | What exists | Failure scenario |
|---|---|---|---|---|---|---|
| SKL-027 | high | designing-interfaces, exploring-ideas | `commands/design.md` line 6; `commands/explore.md` line 6 | `ANTHROPIC-GUIDANCE.md` §3: "a non-zero exit aborts the whole skill invocation (exit 1 from search/compare commands is tolerated under bash)". `doc validate --allocate` is not a search or compare command | Both preambles run `!`devforgeai doc validate --allocate <PREFIX>`` unconditionally. `specs/08-design.md` `## Command` states the allocation "is used by spec mode and discarded by the other two"; `skills/designing-interfaces/SKILL.md:24` repeats it | `/design --sketch IDEA-004` and `/design --brand EPIC-002` abort on `DFA-E215` — a UI prefix exhausted at `UI-999` — even though neither mode uses the id. The user sees the allocator's stderr and no handoff. Explore has the same shape on a remedy run, where `SKILL.md:20` says "On a remedy run that allocation is unused; the id comes from `$ARGUMENTS`", so `/explore IDEA-004 --remedy FLOW-002` aborts on an exhausted `IDEA` prefix it does not touch |
| SKL-028 | medium | improving-framework | `commands/reflect.md` line 5 | `specs/01-cli.md` `report aggregate`: "Exactly one of `<ID>` and `--since` is given; neither or both is `DFA-E430`, exit 3." `skills/improving-framework/SKILL.md:23` states the intended behaviour: "`$ARGUMENTS` holding neither a recognised id prefix nor `--since` stops the run in the preamble" and "its stderr line names the four accepted prefixes and the `--since` form" | The preamble is `!`devforgeai report aggregate $ARGUMENTS --json``, and a bare `/reflect` sends no argument | The skill's own step 1 failure path ("the run stops and the stderr line naming the four accepted prefixes reaches the model") cannot execute, because the body does not load. The user types `/reflect`, sees a raw exit-3 diagnostic, and gets no handoff and no `Next` line. The skill documents a recovery that the abort semantics make unreachable |
| SKL-029 | medium | implementing-stories, planning-work, establishing-context, validating-quality | `commands/build.md`, `commands/plan.md`, `commands/constitute.md`, `commands/verify.md` preamble lines | `ANTHROPIC-GUIDANCE.md` §7.8 frames the abort as intended: "a failing preamble aborts the invocation, which is the gate behavior we want". Conventions §6: the Stop hook prints the handoff | `gate require <phase> $1` heads all four preambles. `specs/01-cli.md:512` confirms "exit 1 with the missing predecessor gate named; the command body does not run" | The behaviour is the design, and the finding is the missing half of it. Four SKILL.md bodies describe a handoff on every exit path (`implementing-stories` step 14, `planning-work` step 13, `establishing-context` step 13, `validating-quality` step 11), and on the gate-require abort none of those runs. The user gets one line of stderr — `gate require: build needs plan gate PASS for SPRINT-001; the last result is FAIL at …` — with no `Next` telling them to run `/plan SPRINT-001 --resume`. The `Next` line is the framework's only navigation |
| SKL-030 | medium | releasing-software | `commands/release.md` line 5 | `specs/01-cli.md` `story list`: "Exit codes: 0 when the walk succeeded, including a count of zero; 1 on `DFA-E231` when `.devforgeai/stories/` is absent" | `!`devforgeai story list --status built --json``. `skills/releasing-software/SKILL.md:37` step 4 handles the empty set: "the run stops with `Blocked   you: no story is at status built; run /verify STORY-nnn first`" | The empty-set path works, because `story list` exits 0 at a count of zero — this preamble is correctly chosen for its main case. The absent-directory case is the gap: `/release v0.3.0` in a project that has not run Plan aborts on `DFA-E231` before step 4, and the `Blocked` line naming `/verify` does not print |
| SKL-031 | low | all nine | `commands/*.md` preambles versus `skills/*/SKILL.md` `## Entry` | Question 5 asks which skills would be shorter and more reliable with `doc load` injected in SKILL.md | Per skill: `exploring-ideas` reads no upstream document, so it gains nothing (`specs/02-explore.md` `## Command` states the reason). `discovering-requirements` already has the only exit-0 loader and gains nothing. `establishing-context`, `planning-work`, `implementing-stories` and `validating-quality` each describe two to four preamble calls in `## Entry` prose ("the command preamble has already run …") — under the collapse of question 4 those lines move into SKILL.md verbatim and the prose describing them becomes redundant, cutting `planning-work/SKILL.md:22-29` and `implementing-stories/SKILL.md:21-29` to a sentence each. `designing-interfaces` gains the most from the opposite move: dropping the unconditional `--allocate UI` from the preamble into step 12, where `SKILL.md:102` already runs it per screen, removes SKL-027. `releasing-software` and `improving-framework` keep their single preamble line | No new failure; this is the per-skill recommendation the question asks for |

## 6. The 500-line rule and standing instructions

| ID | Severity | Skill | Location | Spec or guidance says | What exists | Failure scenario |
|---|---|---|---|---|---|---|
| SKL-032 | low | all nine | `skills/*/SKILL.md` | `ANTHROPIC-GUIDANCE.md` §3 and §7.14: "Keep SKILL.md under 500 lines"; it is the standing instruction for the whole phase, since it is not re-read | Line counts: discovering-requirements 121, improving-framework 123, exploring-ideas 128, planning-work 128, releasing-software 134, implementing-stories 140, designing-interfaces 201, establishing-context 317, validating-quality 333. Reference files total 2,652 lines across 39 files, the largest being `establishing-context/references/brownfield.md` at 137 | No failure. All nine sit between 24% and 67% of the cap |
| SKL-033 | low | all nine | `skills/*/SKILL.md` `## References` | `ANTHROPIC-GUIDANCE.md` §7.14: "Skills must say when to read each supporting file" | All 39 reference lines name a step. Seven skills use an explicit verb ("read at step 4", "read before step 8", "read at step 8 when `platform.target` is `kubernetes`"). `exploring-ideas` and `discovering-requirements` use the shorter form "— step 5: …" with no verb | The shorter form still names the trigger. Recorded as the one stylistic split across the nine |
| SKL-034 | low | all nine | `skills/*/SKILL.md` body | The file is the standing instruction for the phase and is not re-read on later turns | Every SKILL.md reads as standing instructions: the `## Entry` table fixes the argument forms for the whole run, `## Workflow` is numbered and re-enterable by step number (`--resume` re-enters at step 2, 3 or 4 by name), and `## Remedy and resume` states what changes without restating the steps | No failure. The re-entry points are addressed by number, which is what makes a `--resume` run work from a file read many turns earlier |

## 7. Prompting quality against §5

| ID | Severity | Skill | Location | Spec or guidance says | What exists | Failure scenario |
|---|---|---|---|---|---|---|
| SKL-035 | medium | all nine | `skills/*/SKILL.md` | `ANTHROPIC-GUIDANCE.md` §5: "Examples (3 to 5) in `<example>` tags, relevant and diverse, are the most reliable format steer. XML tags separate instructions, context, input" | No SKILL.md uses `<example>` tags. The two places a literal format is fixed use a fenced block (`exploring-ideas/SKILL.md:47-63`, the decision `AskUserQuestion`) or prose (SKL-003, the Park follow-up). Every document shape is delegated to a template file or a reference table | The template delegation is the right call for documents. The gap is the three `AskUserQuestion` call sites — Explore's decision, Explore's revisit, Design's five brand questions — where the exact option labels decide whether a downstream enum parse succeeds, and only one of the three carries a literal block |
| SKL-036 | low | all nine | `skills/*/SKILL.md` | `ANTHROPIC-GUIDANCE.md` §5: "Be clear and direct; explain why … Claude generalizes from the reason" | Reasons are given where the spec gives one and are not invented where it does not, which is BUILD-BRIEF §2's rule. Examples: `implementing-stories:44` "Reading it from the CLI is what keeps a runner name out of this skill"; `:62` "The prompt carries no part of this conversation … which is what makes the verdict independent"; `designing-interfaces:94` "a spec written before the brand kit would reference names nothing defines"; `improving-framework:78` "Two agents emitting observations in parallel would collide on a self-allocated id" | No failure. Two compressions drop a reason the spec gave: `implementing-stories` 7.2 loses the SB-4 chain that `references/tdd-cycle.md` restores, and `exploring-ideas` step 11 loses the gate consequence (SKL-007) |
| SKL-037 | low | all nine | `skills/*/SKILL.md` | `ANTHROPIC-GUIDANCE.md` §5: Opus 5 "verifies its own work unprompted; remove explicit verification and double-check instructions or it over-verifies"; "It expands scope; constrain with 'deliver what was asked, at the scope intended'"; "It delegates readily; cap subagent use" | No skill instructs the model to verify or double-check its own output. Every subagent invocation is a fixed count at a named step ("alone", "one invocation per criterion", "one parallel batch", "batch of seven"), and `validating-quality/SKILL.md:158` fixes the number at ten. Scope is bounded per step by the document that supplies the input | No failure. The invocation counts are the scope cap the guidance asks for, stated as facts rather than as prohibitions |
| SKL-038 | low | all nine | `skills/*/SKILL.md` | `ANTHROPIC-GUIDANCE.md` §5: "Tell Claude what to do instead of what not to do" | Prohibitions appear as statements of what a run leaves unchanged rather than as instructions: "Every other CON row is left as it stands", "Sections the cited ids do not name keep every byte", "no file they own is written". `releasing-software/SKILL.md:8` states the negative space of the phase directly: "it applies no deployment, holds no credential, and runs no build of its own" | No failure. The one residual pattern is the recurring "Write one sentence before that block and nothing after it", which appears in all nine and is a positive instruction with a bound |

## 8. Handoff and send-back compliance

| ID | Severity | Skill | Location | Spec or guidance says | What exists | Failure scenario |
|---|---|---|---|---|---|---|
| SKL-039 | low | all nine | `skills/*/SKILL.md` `## Send-back` and the closing workflow step | Conventions §1 rule 3: the handoff is "printed by `devforgeai handoff`, never composed by the model". §6: "The Stop hook prints this block. The model may write one sentence before it and nothing after it" | No skill instructs composition. Seven say so explicitly: `discovering-requirements:105` "This skill composes none of it"; `planning-work:98`, `implementing-stories:113`, `releasing-software:112`, `improving-framework:103` "the Stop hook prints the handoff from the report; this skill composes none of it"; `establishing-context:255` "`devforgeai handoff` composes and prints both blocks"; `validating-quality:262` the same. `exploring-ideas:107` and `designing-interfaces:177` show the `Next`/`Then` lines as an illustration of what the CLI prints, each introduced as the hook's output | No failure |
| SKL-040 | low | all nine | `skills/*/SKILL.md` `## Send-back` | Conventions §4c: "`/<upstream> <ID> --remedy <ID>,<ID>`" upstream and "`/<downstream> <ID> --resume`" downstream, and no other flag names | Every path names both. Discover→Explore `/explore IDEA-004 --remedy FLOW-003,…` + `/discover IDEA-004 --resume`; Constitute→Discover `/discover IDEA-004 --remedy REQ-014,…`; Constitute→Plan `/plan EPIC-001 --resume`; Plan→Discover and Plan→Constitute with `Then /plan EPIC-nnn --resume`; Build→Plan `/plan EPIC-nnn --remedy AC-nnn,…` + `/build STORY-nnn --resume`; Verify→Build and Verify→Plan with `Then /verify STORY-014 --resume`; Release→Verify `/verify STORY-nnn --remedy FIND-nnn,…` + `/release vX.Y.Z --resume`; Design→Discover `/discover <IDEA-nnn> --remedy UI-nnn,FLOW-nnn` + `/design <STORY-nnn> --resume`. No skill invents a flag | No failure |
| SKL-041 | low | all nine | `skills/*/SKILL.md` | Conventions §1 rule 6: "A phase that finds a defect in an upstream document does not edit that document" | Every edit traced to its owner. `planning-work` R3 edits `STORY-nnn.md`, which Plan writes. `establishing-context` R4a edits the superseded ADR frontmatter and the CON index row, both Constitute's. `validating-quality` R2 rewrites `reports/STORY-nnn-qa.yaml`, its own. `designing-interfaces` step 15 rewrites `ui-specs/UI-nnn.md`, its own. `releasing-software/SKILL.md:8` states the rule outright: "It edits no story, no QA report, no ADR, and no context file". `establishing-context/SKILL.md:230` "`requirements.yaml` is left as it arrived; a defect upstream is cited, not edited" | No failure |
| SKL-042 | medium | releasing-software | `skills/releasing-software/SKILL.md:43` step 7 | Conventions §1 rule 6 concerns documents; `config.toml` is not a §5 document, so the rule is silent | Step 7 instructs writing an answered platform value back into `.devforgeai/config.toml` `[release].platform`, and `specs/09-release.md` step 7 assigns it to the model | A phase skill writes a project config key mid-run. No `doc validate --producer-check` covers `config.toml` (the PreToolUse hook fires on paths under `.devforgeai/`, which includes it), so the producer check may refuse the write and the run has no stated failure path for that refusal. Recorded as the one write outside the skill's document set |

## 9. Language and toolchain names in prose

| ID | Severity | Skill | Location | Spec or guidance says | What exists | Failure scenario |
|---|---|---|---|---|---|---|
| SKL-043 | low | releasing-software | `SKILL.md:43`, `:47-48`, `:86`, `:131`; `references/kubernetes.md` throughout; `references/compose.md:11`, `:13`, `:31`; `references/vps.md:13`; `references/github-actions.md` | Conventions §1 rule 2: "No skill, command, subagent, or hook may name a language, package manager, test runner, or build tool" | Named in prose: `Kubernetes`, `Compose`, `GitHub Actions`, `VPS`, `docker-compose.yaml`, `compose.yaml`, `Chart.yaml`, `kustomization.yaml`, `ClusterIP`, `ExecStart=`, `.github/workflows/`. `specs/09-release.md` Decision 7 records the exemption for the *commands* ("the build command, the package command, the symbol-enumeration command, the image reference, and the port are all config values") and records no exemption for the platform names themselves | The §1 rule 2 list is closed at language, package manager, test runner and build tool; a deployment platform is none of the four, and `[release].platform` is a five-value enum the spec fixes. The names are a sanctioned dependency rather than a leak, but the sanction is not written down. The task asks that any hit be recorded |
| SKL-044 | low | designing-interfaces | `SKILL.md:80`, `:84`, `:88`, `:105`; `references/brand.md`; `references/sketch.md:42`, `:50` | Same rule | Named in prose: `figma:figma-use`, `figma:figma-generate-library`, `figma:figma-design-to-code`, `frontend-design:frontend-design`, the built-in `design` skill, and `HTML`/`SVG` as output formats | `specs/08-design.md` Decision 19 records this one explicitly, including the ten Figma skills not invoked and the fallback when the plugin is absent or unauthenticated. `HTML` and `SVG` are the output formats of the phase and appear in the templates the specs fix. Sanctioned; recorded for coverage |
| SKL-045 | low | all nine | `skills/*/SKILL.md`, `skills/*/references/*.md`, `commands/*.md` | Same rule | A scan for 60 language, package manager, test runner, linter, bundler, framework, container and datastore names returned hits in `releasing-software` and `designing-interfaces` alone. No skill names a language, a test runner, a package manager or a linter. `implementing-stories` reaches the test command through `devforgeai config get stack.test_command` and says why at `SKILL.md:44` | No failure. The two hits above are the whole set |

## 10. Cross-skill integration

Every pair was checked field by field against the byte-verified templates rather than against prose.

| ID | Severity | Skill | Location | Spec or guidance says | What exists | Failure scenario |
|---|---|---|---|---|---|---|
| SKL-046 | low | exploring-ideas → discovering-requirements | `templates/brief.md` `## Core flows`, `templates/decision.yaml` | Conventions §11 acceptance (h): every consumed document matches what its producer's spec emits | Brief `## Core flows` columns are `ID \| Actor \| Trigger \| Steps \| Outcome`; `discovering-requirements` step 4 passes `{id, actor, trigger, steps, outcome}` per row. `## Target user`, `## Success signal`, `## Non-goals`, `## Problem statement` all exist and are all consumed by steps 5, 7, 8 and 9. `decision.yaml` carries `decision`, `remedied_flows`, `carry_forward`; Discover step 1 reads `decision`, the resume form reads `remedied_flows`, Constitute step 10 reads `reason`, `decided_on` and `carry_forward[].consumer` | No mismatch |
| SKL-047 | low | discovering-requirements → establishing-context, planning-work, designing-interfaces | `templates/requirements.yaml`, `references/requirements-shape.md:69-73` | Same | `requirements[]` carries `id, actor, statement, rationale, acceptance_signal, priority, source, status, open_questions, traces_to`. Constitute reads `statement, acceptance_signal, priority, source, status` and states "A requirement record carries `statement`, not `text`" — all present. Plan step 2 drops a record at `priority: excluded`, and the `priority` enum at `requirements-shape.md:69` is `required, expected, optional, excluded` — the value exists. Plan step 5 passes `traces_to`, present at `:73` as `^UI-\d{3}$`. Design's flow-coverage rule matches `FLOW-nnn` against `requirements[].source`, and `:70` defines `source` as "`user`, or a `^FLOW-\d{3}$` present in `consumes`" | No mismatch. The `source` field is the one most at risk and it carries the flow form explicitly |
| SKL-048 | low | planning-work → implementing-stories, validating-quality | `templates/story.md` | Same | Ten H2 sections in order: `Story, Requirements, Acceptance Criteria, Constraints, Anti-patterns, Interface, Layer, Files, Dependencies, Out of scope`. Build step 2 reads `## Acceptance Criteria`, `## Files` (`Path \| Kind \| Layer`), `## Layer`, `## Constraints`, `## Anti-patterns`, `## Dependencies`; step 3 reads `## Interface`. Verify step 2 reads the same seven plus `## Out of scope`. Plan R3 rewrites "section 6", which is `## Interface` counting from `## Story` — the section `send_to_design` is meant to rewrite | No mismatch |
| SKL-049 | low | validating-quality → releasing-software, improving-framework | `templates/qa-report.yaml` `deferrals[]`; `templates/reflect-report.yaml` `technical_debt.groups[].items[]`; `specs/01-cli.md` `report aggregate` | Same | qa `deferrals[]` fields: `id, story, dod_item, target, reason, opened_on, con_or_ap`. Reflect `items[]` fields: `story, dod_item, deferred_at, age_days, reason, report`. The rename `opened_on` → `deferred_at` and the added `age_days` and `report` are performed by `report aggregate`, and `specs/01-cli.md` documents exactly that mapping. `debt-aggregator` groups by `con_or_ap`, which the group's `constraint` and `kind` fields hold. Release's `deferral-auditor` reads "the `FIND-nnn` entries each `reports/STORY-nnn-qa.yaml` marks deferred with their stated reason" — `deferrals[].id` is a `FIND-nnn` and `reason` is the five-value enum | No mismatch. The consumer invents no field name |
| SKL-050 | low | designing-interfaces → planning-work, implementing-stories | `templates/ui-spec.md` | Same | Nine H2 sections: `Purpose, Requirements, Anatomy, States, Breakpoints, Interaction, Accessibility, Tokens used, Out of scope`. Plan step 6 reads `## Requirements`, `## States`, `## Breakpoints`, `## Out of scope`. Build step 3 reads `## Anatomy`, `## States`, `## Breakpoints`, `## Interaction`, `## Accessibility`, `## Tokens used`. Every consumed heading exists | No mismatch |
| SKL-051 | low | exploring-ideas ↔ designing-interfaces | `exploring-ideas/templates/sketch-request.json`; `designing-interfaces/references/sketch.md:13-42` | Same | Request emits `mode, idea_id, brief_path, out_dir, seed_data_path, fidelity, flows[], brand, constraints` plus the seven §5 keys. Design step 2 takes `flows[], out_dir, seed_data_path, brand, constraints` — all present; `fidelity` is emitted and unread, which the spec allows. Return object `screens[]` is `{flow_id, screen, path, title, state}` per `sketch.md:31`, and the brief's `## Mockups` columns are `Flow \| Screen \| Path \| State` — a one-to-one fill with `title` spare. `uncovered_flows` and `brand_sketch` are both consumed by Explore step 6 | No mismatch. This is the tightest contract in the framework and it holds on both sides |
| SKL-052 | low | designing-interfaces ↔ its own agent contract | `designing-interfaces/SKILL.md:82`; `agents/brand-designer.md` `## Output` | Conventions §10: every subagent output has a JSON schema the skill and the CLI parse | Resolved under SKL-010: `adjustments` is the agent file's required field name and the skill uses it. `skills/designing-interfaces/agents.md:26` lists input fields only and defers the output schema to the agent file, which is the catalog's own format | No mismatch inside the skill package. Cross-reference to SKL-010 for the spec edit |
| SKL-053 | medium | exploring-ideas → implementing-stories | `specs/02-explore.md` `## Integration` row 4; `skills/exploring-ideas/templates/decision.yaml` `carry_forward` entry 6; `skills/implementing-stories/SKILL.md:29` | Conventions §11 acceptance (h): every consumed document matches what its producer's spec emits, and §1 rule 8 requires both sides to name the relationship | Explore advertises a consumer that does not exist. `specs/02-explore.md` `## Integration` Build row: "`.devforgeai/explore/seed-data.json`: `entities[].rows` become test fixtures and example rows", and `templates/decision.yaml` `carry_forward` entry 6 names `consumer: implementing-stories`. `specs/06-build.md` `## Integration` Explore row says "none — `explore/brief.md` and `explore/decision.yaml` reach Build only as text Discover copied … Build reads no `IDEA-nnn` or `FLOW-nnn` body". A grep for `seed-data` or `seed_data` across `skills/implementing-stories/` returns zero hits: neither `SKILL.md:29`'s read-from-disk list nor `references/tdd-cycle.md` 7.1's prompt-field table names the file | A `promote` decision carries `seed-data.json` forward under a `carry_forward` entry naming `implementing-stories`, and Build does not open it. `ac-test-writer` invents its own fixture rows, so the example data the user shaped in Phase 0 reaches no test. The two `## Integration` tables contradict each other, which conventions §11(h) is written to catch |

## Fix specifications

### SKL-001 — strip the illustrative block from the Reflect template

File: `skills/improving-framework/templates/reflect-report.yaml`.
Delete every line from line 36 (the blank line after the closing `---`) to end of file, leaving the
file at 35 lines ending with `---`. The deleted content is the spec's second fenced block at
`specs/10-reflect.md:562-601`, which the spec introduces as "the shapes to copy" and which belongs in a
reference, not in the template.

Then add the shapes to `skills/improving-framework/references/observations.md` and
`references/recommendations.md`, which the `## References` lines already schedule before steps 6 and 7:
put the `observations[]` entry shape in the first and the `recommendations[]` entry shape in the second,
and put the `technical_debt.groups[]` shape in `references/observations.md` beside the deferral kind.
No change to `SKILL.md`, whose step 9 already says "the twelve top-level keys in `## Documents` order".

### SKL-017 — declare `disable-model-invocation` on the eight phase skills

Rule to apply, one edit per file. In the frontmatter of each of
`skills/exploring-ideas/SKILL.md`, `skills/discovering-requirements/SKILL.md`,
`skills/establishing-context/SKILL.md`, `skills/planning-work/SKILL.md`,
`skills/implementing-stories/SKILL.md`, `skills/validating-quality/SKILL.md`,
`skills/releasing-software/SKILL.md`, `skills/improving-framework/SKILL.md`,
add the line below `description:`:

```yaml
disable-model-invocation: true
```

`skills/designing-interfaces/SKILL.md` keeps the default, because `exploring-ideas` step 6 and
`planning-work` steps 6 and R3 invoke it through the Skill tool.

Amend `specs/BUILD-BRIEF.md` §2 from "No other frontmatter keys" to name the permitted set:
`name`, `description`, `disable-model-invocation`, `allowed-tools`, `argument-hint`, `arguments`,
per `ANTHROPIC-GUIDANCE.md` §3, whose preamble makes the guidance the winner where the framework
disagrees.

### SKL-024 — migration spec: collapse the nine command files into the nine skills

One migration, applied nine times. Worked here for `explore`; the other eight differ only in the
preamble lines and the argument prose.

**Step 1 — move the frontmatter.** `skills/exploring-ideas/SKILL.md` frontmatter becomes:

```yaml
---
name: explore
description: <the current description, unchanged>
argument-hint: "<idea in one line>" | IDEA-nnn [--remedy FLOW-nnn,...]
allowed-tools: Bash(devforgeai:*), Read, Write, Edit, Glob, Grep, Agent, AskUserQuestion, Skill, WebSearch, WebFetch
disable-model-invocation: true
---
```

`name: explore` rather than a directory rename. The directory keeps its name so that
`exploring-ideas` step 6's reference to "the `designing-interfaces` skill" and every
`produced_by: exploring-ideas` frontmatter value in the templates stay valid, while
`name:` is what decides the slash command and what the Skill tool resolves. `aggregate.rs`
`SLASH_COMMANDS` then still sees `/explore` in session history.

`designing-interfaces` takes `name: design` and omits `disable-model-invocation`.

**Step 2 — move the preamble.** The `!` lines go at the top of the body, above `# Exploring ideas`,
in the order the command file has them:

```
!`devforgeai doc validate --allocate IDEA`
```

For `build`, `plan`, `constitute` and `verify` the `gate require` line leads, which is the gate
behaviour `ANTHROPIC-GUIDANCE.md` §7.8 names. For `design`, apply SKL-027 first and move the
`--allocate UI` line out of the preamble into step 12 instead of into the body head.

**Step 3 — fold the command body prose.** The command file's argument sentences
("`--remedy` selects entry point C and `$1` is the IDEA id") are already present in each SKILL.md
`## Entry` table. Delete them rather than duplicating; where the table lacks a line the command file
carries (`commands/design.md`'s "The allocated id on the preamble's stdout is used by spec mode and
discarded by the other two"), the line is already at `designing-interfaces/SKILL.md:24`.

**Step 4 — delete `commands/`.** Remove the nine files and the directory.

**Step 5 — CLI.** `cli/src/cmd/init.rs:12`: `const COPIED: &[&str] = &["skills", "agents"];`.
`cli/src/cmd/init.rs:117-120`: drop the commands count from the summary line, leaving
`"Copied     {} skills, {} agents"`. Update the `init` tests that assert a command count.
`cli/src/aggregate.rs:18-30` is unchanged, and `name:` from step 1 is what keeps it correct.

**Step 6 — eval runner.** `evals/runner/run_jsonl.py`: delete the `SKILL_COMMANDS` map at line 45 and
the command copy at lines 231-232. Every `cases.jsonl` prompt (`"/explore \"a way for …\""`) keeps
working, because `name:` produces the same slash command.

**Step 7 — cross-skill invocation.** `skills/planning-work/SKILL.md:43` and `:117` change
"Through the Skill tool, call `/design STORY-nnn --spec`" to "Through the Skill tool, invoke the
`design` skill with `STORY-nnn --spec`". `skills/exploring-ideas/SKILL.md:42` changes
"invoke the `designing-interfaces` skill in sketch mode" to "invoke the `design` skill in sketch mode".
Both then name the same target.

**Step 8 — specs.** `specs/00-conventions.md` §3 drops `commands\<name>.md` from the repository layout
and §4b's "Command file shape" paragraph moves into the SKILL.md frontmatter description.
`specs/BUILD-BRIEF.md` §1 drops the `commands/<command>.md` line. The nine spec `## Command` sections
become `## Command` sections describing the SKILL.md frontmatter plus preamble, keeping the §11 H2 list
intact.

**What breaks if the migration is partial.** Renaming the directory instead of setting `name:` breaks
the `exploring-ideas` → `designing-interfaces` Skill call, every `produced_by` value in the templates
that `doc validate --producer-check` reads, and `aggregate.rs` session mining, which in turn blinds
`improving-framework` steps 4 and 5. Deleting `commands/` without editing `init.rs` leaves `init`
walking a missing directory. Deleting it without editing `run_jsonl.py` fails every eval workspace
build.

**Why collapse over keep.** Keep leaves eighteen entry points for nine capabilities, nine of which
carry no preamble and whose SKILL.md bodies assert the preamble already ran. Collapse plus SKL-017
leaves nine, each carrying its own gate line, each costing nothing in context until invoked.

### SKL-027 — move the unconditional `--allocate` out of two preambles

`commands/design.md` (or, post-collapse, the head of `skills/designing-interfaces/SKILL.md`):
delete the line `` !`devforgeai doc validate --allocate UI` ``. The allocation already happens inside
the skill at `skills/designing-interfaces/SKILL.md:102` step 12, once per screen that carries no id,
which is the correct granularity. Then edit `SKILL.md:24` from

> The command preamble has already run `devforgeai doc validate --allocate UI`. Its stdout is the next free `UI-nnn`. Spec mode uses it for the first screen that carries no id; sketch mode and brand mode discard it.

to

> Spec mode allocates its ids at step 12, one per screen that carries no `UI-nnn`. Sketch mode and brand mode allocate none.

and delete the second half of `SKILL.md:26` ("and no preamble stdout" stays correct and needs no edit).

`commands/explore.md`: keep the preamble line — Explore has no other allocation point and a fresh run
needs the id at step 1 — and add the remedy guard so the abort cannot fire on a run that discards the
id. Replace the line with:

```
!`devforgeai doc validate --allocate IDEA || true`
```

`ANTHROPIC-GUIDANCE.md` §3 records that "exit 1 from search/compare commands is tolerated under bash",
which is the same shell tolerance `|| true` makes explicit; a `DFA-E215` then reaches `SKILL.md` step 1
as empty stdout, and step 1's remedy branch takes the id from `$ARGUMENTS` as it already does.
Add one sentence to `skills/exploring-ideas/SKILL.md:20`: "Empty stdout means the allocator failed;
a fresh run stops with that stderr on the `Blocked` line and a remedy run continues, because its id
comes from `$ARGUMENTS`."

Update `specs/02-explore.md` `## Command` and `specs/08-design.md` `## Command` to the new blocks, since
the templates and command files are held byte-identical to the specs.
