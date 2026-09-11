# F6b inventory — what F6 landed and what is left

Taken before any edit of this pass. One targeted grep per brief item; `evidence` names the
line the grep returned. Brief: `<scratchpad>/F6-brief.md`, sections A–L, plus the
coordinator's six late additions (closure verifier A, `specs/audit/CLOSURE-A.md`).

## A. `specs/00-conventions.md`

| Item | File | Done? | Evidence |
|---|---|---|---|
| A.1 §6/§7 Stop hook four cases, `[last_cross]`, SessionStart stdout | `specs/00-conventions.md` | yes | §7 rewritten; `01-cli.md:1549` carries the budget, `:1551` the cross-cutting join |
| A.2 §8 trust: five blocking events, fail-closed set of three | `specs/00-conventions.md`, `specs/01-cli.md:1616-1619` | yes | `01-cli.md:2968` "The fail-closed trust set is three registrations" |
| A.3 §4b merged command shape, `$ARGUMENTS[0]` | `specs/00-conventions.md:140` | yes | "written `$ARGUMENTS[0]` and never `$1`" |
| A.4 §2 ceremony rule scoped, MoSCoW restored, references the script | `specs/00-conventions.md:49`, `specs/03-discover.md:89,134,803` | yes | `must, should, could, wont` restored |
| A.5 §10 subagents, AGT-041 rule in both places | `specs/00-conventions.md:274`, `specs/questions.md` Q-020 | yes | Q-020 states the copy-verbatim rule |
| A.6 CLAUDE.md block is 29/31 lines, not 33 | `specs/01-cli.md:800,2970` | yes | "31 lines including the two markers, 29 between them"; no `33` remains |

## B. `specs/01-cli.md`

| Item | Done? | Evidence |
|---|---|---|
| B.1 `## Hooks` settings block verbatim | yes | fence at line 2334 is byte-identical to `hooks/settings.hooks.json` (148 lines) |
| B.1 `hook run` arms incl. `trust-check`, `prompt-expansion`; no `--json` in a settings string | yes | `:1529`, `:1536`, `:1538` |
| B.1 SubagentStop key list | yes | `:54` names `agent_type`, `last_assistant_message`, `agent_transcript_path` |
| B.2 `[last_cross]`, `[stop_hook].blocked_session` / counter | yes | `:365`, `:371`, `:383`, `:2603` |
| B.3 `init` `trusted`, CLAUDE.md line, permissions merge, no `commands/`/`evals/` | yes | `:810`, `:830`, `:832`, `:794`, `:1517` |
| B.4 `--allocate` reservation under `.devforgeai/.allocated/` | yes | `:1072` |
| B.5 refusing-git-hook error row | **n/a, recorded** | no code row exists; `questions.md` Q-018 names the gap and the closing change |
| B.5 `DFA-E420` deleted, `DFA-W420`/`E349`/`E350` present | yes | `:748`, `:749`, `:777`; no `DFA-E420` |
| B.6 gate keys (`fields_present.field`, `length_between.field`, `set_cover.status_field`) | yes | `:254`, `:256`, `:258` |
| B.6 `--no-run` skip set, degraded skip set, `DFA-E321` on `not_implemented` | yes | `:318`, `:311`, `:940`, `:921` |
| B.6 per-arm `gate require` id-shape table normative | yes | `:903`, `:909-917` |
| B.7 envelope `payload`; AGT-029 `total == 0` → `1.0` in all three places | yes | `:284`, `:967`, `:564` |
| B.8 `Bash\|PowerShell` matchers everywhere | yes | `:2909` |
| **B.8 `## Command` preamble idiom still written `$1`** | **NO** | `:575`, `:576`, `:579` — contradicts conventions §4b |
| **Integration table quotes preambles as `$1`** | **NO** | `:2282`, `:2283`, `:2284` |
| L `DFA-E202` neighbour line: a fenced YAML template is one document | **NO** | no such sentence found |

## C / I. Phase specs `02`–`10`

| Item | Done? | Evidence |
|---|---|---|
| C.1 nine `## Command` fences == SKILL.md frontmatter+preamble | yes | `f6b_cmdcheck.py`: 9 OK, exit 0 |
| C.2 explore step 2a/2b/2c, Decision 19, `WebSearch, WebFetch` sanction | yes | `02-explore.md:195,197,937`, `:424` |
| C.3 merged verifier shapes under `payload`; per-agent schema strings gone | yes | 71 `devforgeai/verifier/1`; zero `devforgeai/<agent>/1` |
| C.3 `figma_context` in `08-design.md` step 13 | yes | `08-design.md:211`, `:348`, `:876` |
| C.3 `blocks_deployment` off `findings[]`, `kind` enum instead | yes | `09-release.md:253` |
| C.3 `04-constitute` three gate metric paths gain `.payload.` | yes | `04-constitute.md:393,401,409` |
| **C.3 `unlisted_role` in the `flow-integrity-auditor` schema** | **NO** | `03-discover.md:343` enum is `["empty","not_a_persona"]`; `agents/flow-integrity-auditor.md:49` has three |
| C.4 `07-verify` truncation sentences gone, FIND bands 100 wide | yes | `07-verify.md:228`, `:1132`; no `truncat` in the file |
| C.5 platform / Figma proper-noun sanction | yes | conventions §2; `questions.md` Q-013 |
| C.6 `10-reflect` six-row target table | yes | `10-reflect.md:604` `templates/rec-targets.md` |
| C.7 SKL-002 `---` fence | yes | Templates fence at `10-reflect.md:524` is byte-identical to the 33-line file |
| C.8 `06-build` `payload.checks`, `verdict met\|unmet` | yes | `06-build.md:134`, `:397`, `:399` |
| **C.8 Integration row still reads `verifiers.story_ac.checks[]`** | **NO** | `06-build.md:665` |
| C.8 `03-discover` priority MoSCoW | yes | `03-discover.md:89,134,252,803` |
| I `07-verify` `## Gate` keys on no `truncated` flag | yes | zero `truncat` in `07-verify.md` |
| **CLI-calls rows quote preambles as `$1`** | **NO** | `03-discover.md:389,395`; `05-plan.md:391,392`; `06-build.md:493,494`; `07-verify.md:624,625,627` |

## D. `specs/11-subagent-catalog.md`

| Item | Done? | Evidence |
|---|---|---|
| 46 entries exist, one per `agents/*.md` | yes | scripted: 46 catalog entries, 0 agents missing |
| `tools` line matches each agent's frontmatter | yes | scripted: 0 mismatches across 46 |
| no `AskUserQuestion` in the permitted-tool list | yes | zero hits in the catalog |
| envelope with `payload`; no `truncated`, no `blocks_deployment` boolean | yes | `:2137` |
| entry 1 `idea-interrogator`: `tools: Read`, `selected_segments` in, `candidate_segments` out | yes | `:115-150` |
| worktree codes `DFA-E271`/`E272` | yes | `01-cli.md:2945` |

## E / F / G

| Item | Done? | Evidence |
|---|---|---|
| E `BUILD-BRIEF.md` permitted keys, `quick_validate` caveat, no `commands/` | yes | `BUILD-BRIEF.md:64,89,93,95` |
| E `README.md` install steps, script reference | yes | `README.md:24,105` |
| F `scripts/ceremony_scan.py` exists, case-insensitive, `--paths` | yes | 300 lines; `no ceremony in 94 file(s)`, exit 0 |
| G `questions.md` Q-013/015/016/017 + Q-018…Q-021 | yes | Q-018 error-table gap, Q-019 AGT-029, Q-020 AGT-041, Q-021 skill dirs |

## L

| Item | Done? | Evidence |
|---|---|---|
| SKL-002 reflect-report fence | yes | byte-identical, 33 lines, opens on `schema:` |
| AGT-002/R4 catalog entry 1 | yes | see D |
| R5 explore `## CLI calls` row, design Decision 24 | yes | `02-explore.md:435` reads "workflow step 1"; `08-design.md:869` |
| ceremony_scan case-insensitive with word boundaries | yes | docstring lines 22–27; clean run |
| SKL-053 build reads `explore/seed-data.json` | yes | `06-build.md` inputs; `02-explore.md` Integration |
| F4 third pass template-fill wording, conventions §5 envelope | yes | conventions §5; `04-constitute` steps 4–10 |
| **01-cli one-document line near `DFA-E202`** | **NO** | see B |

## Coordinator's late additions (CLOSURE-A)

| # | Item | Done? | Evidence |
|---|---|---|---|
| 1 | CLI-030 source-digest walk, `trust digest` subcommand | **NO** | `01-cli.md:1580` still says "every tracked file under `cli/` … excluding `cli/DIGEST`"; decision 39 (`:2913`) the same; `trust digest` appears nowhere in the spec though `cli/src/cli.rs:393` declares it |
| 2 | Stop budget three blocks per `session_id` alone | **NO** (conventions §7) | `01-cli.md:1549` keys on the triple; F2 owns the 01-cli lines |
| 3 | Shell-write guard: Stop-time producer scan, `[stop_hook].scanned_at` | **NO** | zero `scanned_at` in `01-cli.md` and in `cli/src/hooks/run.rs` |
| 4 | `phase set plan --id SPRINT-nnn --epic EPIC-nnn` | **NO** in specs | `skills/planning-work/SKILL.md:48,128` already carries it; `01-cli.md` and `05-plan.md` do not |
| 5 | `## Evals` four verification gates, `bypassPermissions` reason | **NO** | `01-cli.md:2783,2792` still `acceptEdits` only; no gate list |
| 6 | `questions.md` first acceptance run | **NO** | last entry is Q-021 |

## What F6b then changed

Every row marked **NO** above is now closed, except the one that is a code-side gap:

| Item | File | What landed |
|---|---|---|
| B.8 / K | `specs/01-cli.md` §Command | preamble idiom rewritten to `$ARGUMENTS[0]`, with the measured `simple_expansion` refusal and the §4b merged-frontmatter key list in place of the pre-merge `description, argument-hint, allowed-tools` |
| B.8 | `specs/01-cli.md` Integration table | three rows quote `$ARGUMENTS[0]`; the Discover row quotes `doc load discover-entry "$ARGUMENTS[0]"` |
| C / I | `specs/03-discover.md`, `05-plan.md`, `06-build.md`, `07-verify.md` CLI-calls rows | nine quoted preamble commands moved to `$ARGUMENTS[0]` |
| L | `specs/01-cli.md` frontmatter grammar | a YAML template is one document; a leading `---` makes `doc validate` read an empty first document, which is `DFA-E202` |
| C.3 | `specs/03-discover.md` | `flow-integrity-auditor` fence replaced with the agent's; `unlisted_role` now in the schema, in workflow step 4, and in the send-back row |
| I | `specs/02-explore.md` | `kill-case-builder` fence replaced with the agent's envelope-and-`payload` shape |
| I | `specs/06-build.md` | `context-validator` gains `payload` and finding `confidence`; `ac-test-writer` `payload.required` drops `conflicts_with`; the Plan integration row reads `verifiers.story_ac.payload.checks[]` |
| I | `specs/08-design.md` | `requirement-coverage-auditor` finding `id` carries `maxLength: 60` |
| D | `specs/11-subagent-catalog.md` | `requirement-drafter` `priority` enum restored to MoSCoW; `recommendation-drafter` `target.kind` drops `command` |
| F4 pass 5 | `specs/10-reflect.md` | `recommendation-drafter` `kind` drops `command`; the `## Outputs` target-kind table and the `## Templates` fence both carry the narrowed `framework_file` shape and the first-match precedence rule |
| CLOSURE-A 1 | `specs/01-cli.md` | `trust digest` documented as the release step; the walk's real exclusions (`target`, `.git`, `cli/REVISION`, `cli/DIGEST`); `unversioned` when the root is no git work tree; decision 39 rewritten to agree |
| CLOSURE-A 2 | `specs/00-conventions.md` §7 | the Stop budget is three blocks per `session_id` and on nothing else, with the reason |
| CLOSURE-A 3 | `specs/01-cli.md` | `[stop_hook].scanned_at` in the state table and in the defaults sentence; the Stop-time producer scan written from `cli/src/hooks/run.rs::stop`, which has landed |
| CLOSURE-A 4 | `specs/01-cli.md`, `specs/05-plan.md` | `phase set plan --epic <EPIC-nnn>`, required with no `sprint.yaml` on disk, `DFA-E013` on a mismatch; step 4, R4, and the CLI-calls row |
| CLOSURE-A 5 | `specs/01-cli.md` `## Evals` | `--permission-mode bypassPermissions` with the headless-approval reason; the four verification gates and the `--preflight` shape |
| CLOSURE-A 6 | `specs/questions.md` | Q-022 records the first acceptance run; Q-016 now points at `trust digest` in place of the scratchpad script |
| K | `specs/01-cli.md` `init` | `data.commands` and the human `Commands` line, matching `cli/src/cmd/init.rs:187,264` |
| B.5 | — | left as recorded: no code row exists for a refusing git hook, and Q-018 names the gap |

## Verification run before editing

```
python scripts/ceremony_scan.py            -> no ceremony in 94 file(s), exit 0
python <scratchpad>/review_spec.py         -> exit 0, no output
python <scratchpad>/f6b_cmdcheck.py        -> 9 OK, exit 0
cargo test --test errors_table --test default_gates -> 14 passed, 6 passed
reflect-report fence vs template            -> IDENTICAL, 33 lines
```

## Verification run after editing

```
python scripts/ceremony_scan.py            -> no ceremony in 94 file(s), exit 0
python <scratchpad>/review_spec.py         -> exit 0, no output
python <scratchpad>/f6b_cmdcheck.py        -> 9 OK, exit 0
cargo test --test errors_table --test default_gates -> 14 passed, 6 passed
reflect-report fence vs template            -> IDENTICAL, 33 lines
rec-targets fence vs template               -> IDENTICAL, 39 lines
agent output schemas vs catalog and specs   -> 1 semantic mismatch, in the agent file (routed)
grep 'verifiers\.<x>\.<y>' without payload   -> no hits outside the four envelope keys
```

The one remaining schema mismatch is `agents/requirement-drafter.md:44`, which still carries the pre-MoSCoW `required | expected | optional | excluded` priority enum. `skills/discovering-requirements/templates/requirements.yaml`, `specs/03-discover.md`, and now the catalog all carry `must | should | could | wont`. The agent file is not F6's to edit; it is in `## Routed changes`.

## Second pass — F2's landed facts (934 tests)

Every fact the coordinator sent was checked against the code before it was written, and all seven held.

| Fact | Read at | Spec now says |
|---|---|---|
| `scanned_at` window closes at the end of the recorded second | `hooks/run.rs:1215-1223` (`+ 1`) | the reason is in `## Hooks`: an RFC 3339 stamp carries whole seconds and an mtime carries more, so a same-second write would be read twice |
| scan set and exclusions | `hooks/run.rs:1130-1140` | seven directories, three extensions; `reports/`, `state.toml`, `config.toml`, `gates.toml`, `.allocated/` excluded, each with its reason |
| refusals fold into the FAIL reason as `DFA-E212 <path>: <message>` | `hooks/run.rs:1320` | stated in `## Hooks` and in conventions §7 |
| budget is per `session_id`; `blocked_phase`/`blocked_id` are a record | `hooks/run.rs` stop, `state.rs:290-299` | the `[stop_hook]` comments say which key the budget reads; conventions §7 says nothing reads the other two |
| `[plan].epic`, `[active].plan` still the sprint | `state.rs:218-229`, `cmd/phase.rs:31-87,304` | new `[plan]` table in the annotated `state.toml` block, absent from the initial file |
| `phase set plan --epic`: `DFA-E011` / `DFA-E013` | `cmd/phase.rs:41,50,61,86` | the two codes divide by what went wrong; `05-plan.md` CLI-calls row corrected from my first pass, which had put both on `DFA-E013` |
| `trust digest` lines, 40 hex, walk exclusions | `cmd/trust.rs`, `trust.rs:389-414` | line 1 is forty lowercase hex or `unversioned`; the test `the_walk_digest_matches_the_shipped_revision` is named as the lock on line 2 |
| `hook install` de-duplicates identical handlers | `hooks/settings.rs:93-110` | the intra-entry collapse is now distinguished from the cross-entry identity check that was already documented |

The `<!-- verify against cli/src/cmd/phase.rs -->` marker from my first pass is removed: the flag has landed.

## Routed changes

Five things F6b found and does not own.

1. **`agents/requirement-drafter.md:44`** — the `priority` enum still reads `["required","expected","optional","excluded"]`. MoSCoW was restored everywhere else: `skills/discovering-requirements/templates/requirements.yaml`, `specs/03-discover.md` (`## Outputs`, Decision 7), `specs/questions.md` Q-007, and now `specs/11-subagent-catalog.md:345`. The agent is the last holdout, and an agent drafting `priority: required` writes a value `doc validate` refuses. → F3/F4.
2. **`cli/src/cmd/trust.rs:122-123`** — `trust digest` prints its second human line labelled `REVISION` where it holds the source digest; the label should be `SOURCE`. `specs/01-cli.md` now documents `REVISION`, `SOURCE`, `DIGEST`. → F2.
3. **`cli/src/cmd/trust.rs:113`** — `let revision = crate::trust::REVISION_UNVERSIONED.to_string();` hard-codes `unversioned`; the spec and `specs/questions.md` Q-016 say line 1 is `git rev-parse HEAD` when the framework root is a git work tree. Until this lands, a repository with git still gets `unversioned`. → F2.
4. **`cli/src/cmd/phase.rs`** — no `--epic` flag. `skills/planning-work/SKILL.md` steps 4 and R4 already pass it, and `specs/01-cli.md` `phase set` and `specs/05-plan.md` are now written to match, with a `<!-- verify against cli/src/cmd/phase.rs -->` marker on the 01-cli paragraph. One code question with it: both the skill and the spec say a `--epic` that contradicts `sprint.yaml` `epic` is `DFA-E013`, whose table message is `'<value>' is not an ID; expected PREFIX-nnn with three digits`. That message does not describe a contradiction. Either the arm raises a different code, or `DFA-E013`'s message widens — and either way the bijection test decides it, so the spec row and the code row move together. → F2.

5. **`cli/REVISION` line 2 is stale** → whoever owns the release step. `cargo test --test trust` fails on `the_walk_digest_matches_the_shipped_revision`: the walk over `cli/` no longer produces the digest the shipped `cli/REVISION` records, because the fix wave changed files under `cli/src/`. Everything else in that suite passes (25 of 26). This is live rather than cosmetic — `trust verify` compares the source digest while a Claude session is active, so after Bryan runs the `trust pin` of Q-016 every in-session verify would fail with `DFA-E504` and the fail-closed path would refuse every write. The remedy is the release step the spec now documents: `devforgeai trust digest --framework C:\Projects\DevForgeAI`, writing both files, run after the last code change lands rather than before. It is not a spec defect and F6b changed no file under `cli/`.

One note beside it, not a defect: `cli/src/state.rs` carries a `SPEC_EXAMPLE` constant described as the spec's `## Outputs` `state.toml` block verbatim. That block now carries the `[plan]` table. The constant is its own copy and no test compares the two, so nothing is red, but the two will drift further unless the constant is refreshed. → F2, at their discretion.

## Third pass — closure verifier C (`specs/audit/CLOSURE-C.md` `## Open items`, `## Regressions`)

Nine loci, all closed. None had landed before this pass.

| # | Finding | Locus | What landed |
|---|---|---|---|
| 1 | FWK-004 / R3 | `04-constitute:161`, `05-plan:185`, `06-build:140`, `07-verify:242`, `02-explore:941` | each rewritten to the amended decision-1 contract: a FAIL exits 2 with `decision: "block"` under three blocks per `session_id`, and at the last block the hook exits 0 with the FAIL handoff in `systemMessage`. `06-build` and `07-verify` also say a SEND BACK exits 0 and blocks nothing. No `exit 1 blocks` or `blocks the stop once` remains in any spec |
| 2 | FWK-027 | `06-build:518-522` | `Bash` alone → `Bash\|PowerShell`; three `Write, Edit` rows → `Write\|Edit\|NotebookEdit`, and a fourth the verifier did not name at `:522`. No `Bash`-alone matcher is left in any spec |
| 3 | FWK-021 | `03-discover:189/:227/:804/:805`, `07-verify:572/:586`, `09-release:257` | seven clauses now state the fact rather than a policy: the harness strips `AskUserQuestion` from every subagent whatever its `tools` list holds, so the tool could not have been kept and the question belongs to the invoking skill. `03-discover:805` was a fourth locus in the same table, not named by the finding |
| 4 | FWK-026 / R4 | `00-conventions:119` | caller column now reads "workflow step; the predecessor gate is enforced by the `UserPromptExpansion` hook and by the preamble's `gate require`" |
| 5 | FWK-083 / R1 | `BUILD-BRIEF:66`, `04-constitute:1071`, `05-plan:195`, and `01-cli` decision 86 | all four now say the pattern matches case-insensitively and that the scoping, not the case, is what releases ordinary English. `05-plan`'s live lower-case `always` is reworded to the shipped skill's own sentence, "A remedy run reaches R4 with `sprint.yaml` on disk". Decision 86 in `01-cli` was a fourth copy of the case claim, not named by the finding |
| 6 | R2 | `ANTHROPIC-GUIDANCE:58` | the substitution list carries the measured caveat: `$1` inside a `!` preamble is refused with `Contains simple_expansion` and aborts the invocation, `$ARGUMENTS[0]` works in the same position, and the `arguments:` named form did not load. Scoped to the `!` position, since `$N` elsewhere was not measured |
| 7 | R6 | `README.md` | `--preflight` documented beside the other runner flags with what it checks |
| 8 | R5 | six agent output fences | five aligned to the agent file's exact text (`epic-grouper`, `persona-mapper`, `requirement-coverage-auditor`, `story-ac-verifier`, `requirement-drafter`); `frontend-implementer`'s missing fence added to `06-build.md` from the catalog, which is byte-identical to the agent. `f6b_schemacheck.py` now exits 0 |
| 9 | review_spec scoping | `<scratchpad>/review_spec.py` | its private ceremony regex is replaced by a call to `scripts/ceremony_scan.py`'s `scan_text`, so the rule is stated once. `## Subagents` is dropped from the scanned set; `## Command`, `## Templates`, `## Workflow`, `## Scope` are kept |

Before the rescope, `review_spec.py` reported nine FAILs across five specs — lower-case `never` in `## Subagents` contract prose and a `must` inside a `## Templates` YAML fence, both released by clauses 1 to 3. After it, all nine phase specs are `0 FAIL, 0 WARN` and the script exits 0. It is not vacuous: a poisoned copy carrying "You must always verify that the sprint closed before proceeding." in `## Workflow` still returns four FAILs.

`requirement-drafter`'s stale priority enum, routed as item 1 of `## Routed changes`, has been fixed by F3/F4 — `agents/requirement-drafter.md:44` now reads `must | should | could | wont`. That routed item is closed.

## Fourth pass — F5's runner facts, and Discover's `success_metric`

Each fact read from `evals/runner/run_jsonl.py` before it was written. The per-case `timeout` key is the one exception: it is not in `parse_args` or the invoke path yet, so it is written from the coordinator's statement that F5 is adding it.

| Fact | Read at | Where it landed |
|---|---|---|
| `preflight` key and `--preflight` mode; throwaway copy of the workspace | `run_jsonl.py:873-913`, `parse_args` | `01-cli` `## Evals` gate list and conventions §9 |
| `cli_defaults()` runs `init` once per process in a throwaway directory; a case seeds only what it cares about | `run_jsonl.py:496-520` | both |
| a seeded file opening `#!` is written executable | `run_jsonl.py:558-563` | `01-cli` `## Evals`, conventions §9 |
| `--permission-mode bypassPermissions` and its four reasons | `run_jsonl.py:118-125` | both (the `01-cli` paragraph was already there; §9 gained it) |
| `limit` status and the sixth summary column | `run_jsonl.py:167,794,885,1105-1114` | `01-cli` status enum, summary example, exit codes; conventions §9; README |
| a timed-out run records no cost | `run_jsonl.py:780,860` | `01-cli` and conventions §9 |
| per-case `timeout` overriding `--timeout`; Constitute carries 1500 | not yet in `parse_args` — written from the coordinator | `01-cli` flag table, conventions §9, README |
| Discover `success_metric` shape | `skills/discovering-requirements/SKILL.md:44,52` and `references/requirements-shape.md:55-60` | `03-discover` step 7, step 11, the `epics[]` constraint cell, and a shape note under the table |

The `epics[]` constraint moved from "one sentence containing one number and one unit" to the skill's own "one sentence carrying one measurable quantity, its unit or count, and a comparison; at least one digit". The round-2 question template fence is byte-identical to `templates/questions.md`, verified after the edit.

Acceptance status added to `questions.md` Q-022: Explore, Design, Reflect and Plan pass end to end headless; Discover and Verify complete the workflow and fail on content — a non-numeric `success_metric`, which is the defect the suite exists to find, and a `Next` line only the Stop hook renders, which is unmeasurable until the Q-016 pin exists; Build waits on a fixture and Release on a CLI fix.
