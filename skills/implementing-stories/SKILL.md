---
name: build
description: Phase 4 of DevForgeAI, run by /build. Takes one STORY-nnn that Plan marked ready and turns it into committed source and test files inside a git worktree of the project, test-first - one failing test case per AC-nnn before the code that satisfies it, one commit per red and per green, a refactor pass opened by a CLI number, an integration pass for a story that carries an interface, and a fresh-context verifier that decides each criterion from the story, the diff and the test output alone. Reach for it whenever /build is typed, whenever a story is being implemented, whenever a red-green cycle, a declared file set, a story worktree, a build note, or a build report is in play, and whenever STORY-nnn-build.yaml, .devforgeai/build/STORY-nnn-note.yaml, a Build send-back to Plan, or a Verify remedy arriving as /build STORY-nnn --remedy FIND-nnn appears in a report or a handoff.
argument-hint: STORY-nnn [--remedy FIND-nnn,...] [--resume]
allowed-tools: Bash(devforgeai:*), PowerShell(devforgeai:*), Read, Write, Edit, Grep, Glob, Agent
disable-model-invocation: true
---

!`devforgeai gate require build $ARGUMENTS[0]`
!`devforgeai doc load story $ARGUMENTS[0]`
!`devforgeai doc load context all`
!`devforgeai doc load sprint -`

# implementing-stories

Build takes one story and turns it into code. Its subject is a single `STORY-nnn`; its writable set is the `## Files` table of that story, which the `PreToolUse` `story files --check` hook enforces at the moment of each write. It decides nothing about what the story says: it edits no `STORY-nnn.md`, no `sprint.yaml`, no context file, and no `UI-nnn.md`, and a defect in any of them leaves as a SEND BACK citing ids. It runs no git command of its own — every worktree operation and every commit goes through `devforgeai`, so the hooks run on each one. It interprets no test count, coverage figure, lint result, or complexity number; those are `gate check` evidence.

## Entry

`/build` invokes this skill. Arguments:

| Form | Run kind | Meaning |
|---|---|---|
| `/build STORY-nnn` | full | implement every criterion of the story |
| `/build STORY-nnn --remedy FIND-nnn,FIND-nnn` | remedy | Verify cited those findings and sent the story back |
| `/build STORY-nnn --resume` | resume | re-enter at the first unfinished criterion |

A `$1` whose prefix is not `STORY` stops the run with one `Blocked` line naming the accepted prefix.

The four preamble lines at the head of this file run before the body loads:

- `devforgeai gate require build $ARGUMENTS[0]` — exit 1 names the missing plan gate and the body does not load, which is the gate.
- `devforgeai doc load story $ARGUMENTS[0]` — the whole `.devforgeai/stories/STORY-nnn.md`, its ten sections and its frontmatter.
- `devforgeai doc load context all` — the six context files.
- `devforgeai doc load sprint -` — `.devforgeai/stories/sprint.yaml`, whose `epic` key is the `EPIC-nnn` a send-back cites.

None of the four allocates an id. This phase allocates none at all: every id it writes was allocated by Plan, so no preamble line and no step can abort on an exhausted prefix.

Read from disk as the run needs it: `.devforgeai/config.toml` through `devforgeai config get`, `.devforgeai/state.toml` (`[current].phase`, `[current].id`, `[active].build`, `[active].plan`, `[active].release`), `.devforgeai/brand/tokens.json` at step 3, `.devforgeai/explore/seed-data.json` at step 7.1 when that file exists, `.devforgeai/build/STORY-nnn-note.yaml` on a resume run, and `.devforgeai/reports/STORY-nnn-qa.yaml` on a remedy run through `devforgeai doc load qa-report`.

`.devforgeai/explore/seed-data.json` is the example data Explore shaped in Phase 0 and carried forward on a `promote` decision. Its `entities[].rows` are the fixture rows step 7.1 passes to `ac-test-writer`, so a test asserts against the rows the user recognises rather than against rows the agent invented. An absent file is the shape a project that ran no Explore phase leaves, and 7.1 proceeds with no fixture rows.

## Workflow

**1. Establish the run — model.** Input: `$ARGUMENTS` and the preamble's stdout. The run kind is `remedy` when `$ARGUMENTS` holds `--remedy`, `resume` when it holds `--resume`, and `full` otherwise. The subject is `$1`. The epic is the `epic` key of the printed `sprint.yaml`. A remedy run takes step R1 of `## Remedy and resume` before step 4. Failure path: a `$1` prefix other than `STORY` stops the run with one `Blocked` line naming that prefix.

**2. Read the story and the context set — model.** Input: the story text and the six context files on the preamble's stdout. Output: the ordered `AC-nnn` list of `## Acceptance Criteria`; the `Path` set of `## Files` with each row's `Kind` and `Layer`; the `## Layer` value; the `CON-nnn` rows of `## Constraints`; the `AP-nnn` rows of `## Anti-patterns`; the `STORY-nnn` list of `## Dependencies`; the `## Testing standards` rows of `coding-standards.md`; the `## File placement rules` and `## Naming conventions` rows of `source-tree.md`. Failure path: a frontmatter `status` of `draft` stops the run with one `Blocked` line naming the status and `/plan <EPIC-nnn> --resume`.

**3. Read the screen specs — model, CLI.** Input: the `UI-nnn` values of `## Interface`, zero to four of them. Run `devforgeai doc load ui-spec <UI-nnn>` once per row, then `devforgeai config get frontend.tokens_path` and Read the printed path when at least one row exists. Output: each screen's `## Anatomy`, `## States`, `## Breakpoints`, `## Interaction`, `## Accessibility`, and `## Tokens used` tables, and the token leaf set per group. Failure path: exit 1 on `DFA-E200` stops the run with one `Blocked` line naming the `UI-nnn` and `/design <UI-nnn> --spec`.

**4. Open the worktree — CLI.** Run `devforgeai worktree ensure <STORY-nnn>`, which prints the worktree path. Output: a worktree at `<[build].worktree_root>/<STORY-nnn>` on branch `<[build].branch_prefix><STORY-nnn>`, and the path the run's `sh` and test commands run in. The worktree holds sources and tests; `.devforgeai/` documents, reports and `state.toml` stay in the main checkout, and `worktree ensure` appends a `[[worktree]]` entry there carrying `story`, `path`, `branch`, and `created_at`. A `devforgeai` call reaches the same state from either directory, so the run issues its CLI calls from wherever it stands. Failure path: exit 1 on `DFA-E272` means a concurrent worktree's story declares a shared path; the run stops with one `Blocked` line naming both story ids and the shared path. Exit 1 on `DFA-E271` means the project root is not a git work tree; the run stops with one `Blocked` line naming `git init`. Two `/build` runs in two worktrees under `worktree_root` are the supported parallel shape: the `PreToolUse` write-scope check resolves each write to the story whose registered worktree the path sits under, and `worktree ensure` refuses a story whose `## Files` set intersects a live worktree's. `references/worktree-rules.md` carries both guards.

**5. Activate the phase — CLI.** Run `devforgeai phase set build --id <STORY-nnn>`, from the worktree or from the root; run from inside a registered worktree it sets `[active].build` to that entry's story. The CLI writes `[current].phase`, `[current].id`, `[active].build`, and the story frontmatter `status` value `building`. From here the `PreToolUse` `story files --check` hook resolves `--id` from `[active].build` and guards every write, the model's and each subagent's alike. Failure path: exit 1 on `DFA-E320` means the plan gate is not PASS; the run stops and the Stop hook prints the gate result.

**6. Read the command — CLI.** Run `devforgeai config get stack.test_command`. Output: the one command string steps 7.2, 7.5, 7.8, 8, and 10 run with Bash. Reading it from the CLI is what keeps a runner name out of this skill. Failure path: an empty printed value, which `degraded = true` produces, stops the run with one `Blocked` line naming `devforgeai stack detect`.

**7. The per-criterion loop — subagents, model, CLI.** Input: the `AC-nnn` list of step 2, filtered to the run's criterion set — every criterion on a `full` run, the R1 criteria on a `remedy` run, and the criteria from `resumed_at` onward on a `resume` run. The body runs once per criterion in `## Acceptance Criteria` order, to exhaustion or to the first stop. Read `references/tdd-cycle.md` before the first criterion: it carries steps 7.1 through 7.8 field by field, the retry budget of each subagent, and the six commit messages.

  - **7.1** `ac-test-writer` turns the one criterion into one failing test case inside the declared test files, which the cases of other criteria may share. A returned `payload.testable` of `false` stops the loop and the run continues at step 12, which is the send-back of `## Send-back`.
  - **7.2** The model runs the step 6 command with Bash in the worktree. A zero exit code means the new test passed before any implementation; the loop stops with the cycle's `green_commit` at `""` and the run continues at step 12.
  - **7.3** `devforgeai commit <STORY-nnn> -m "<AC-nnn> red"` records the failing test case. The printed sha is the cycle's `red_commit`. Failure path: exit 1 from the `pre-commit` hook's `doc validate` or `context audit`; the run stops with one `Blocked` line carrying the hook's stderr.
  - **7.4** `backend-implementer`, or `frontend-implementer` when `## Layer` is `interface` and `## Interface` carries a row, writes the smallest code inside the declared set that satisfies the test. Failure path: the returned JSON carries `blocked` with a reason from its enum; the loop stops and the run continues at step 12.
  - **7.5** The model runs the command again. A non-zero exit re-invokes the same implementer once with the failing output appended; a second non-zero exit stops the loop.
  - **7.6** `devforgeai commit <STORY-nnn> -m "<AC-nnn> green"` records the code. The printed sha is the cycle's `green_commit`. Failure path: exit 1 from the `pre-commit` hook's `doc validate` or `context audit`; the run stops with one `Blocked` line carrying the hook's stderr. Two different shas in one cycle entry are the record that the test existed and failed before the code existed.
  - **7.7** `devforgeai report show <STORY-nnn> build --check build-lint` and `--check build-complexity` return the two check entries the `PostToolUse` Bash hook wrote. Exit 1 on `DFA-E400` means no partial report exists; running the command once more makes the hook write it.
  - **7.8** `refactor-surgeon` runs when at least one of those two entries carries `status: fail`, and does not run otherwise, so a rewrite is opened by a number rather than by taste. The model then runs the command and commits with `-m "<AC-nnn> refactor"` on a zero exit code.

**8. Write the integration tests — `integration-test-writer`, zero or one invocation.** The subagent runs when `## Layer` is `interface` or `## Interface` carries at least one row, and does not run otherwise. Input: every `AC-nnn` line, the `## Files` rows of `Kind` `test`, the source paths of every cycle, the `UI-nnn` `## Interaction` and `## States` tables when they exist, and the `## Layer dependency rules` rows. The model then runs the test command and `devforgeai commit <STORY-nnn> -m "integration"`. Failure path: a non-zero exit re-invokes the subagent once with the failing output appended; a second non-zero exit ends the loop and the run continues at step 12.

**9. Run `context-validator` over the written file set — one invocation.** Input: the path list of `devforgeai story files --diff --id <STORY-nnn> --json`, the six context file paths, the `CON-nnn` rows of `## Constraints`, and the `## Layer` value. The `SubagentStop` hook ingests its stdout into `verifiers.context`. Failure path: output that does not parse against the schema in `agents.md` re-invokes the subagent once with the parse error appended; a second failure leaves the block at `status: unparsed`, which fails the `build-context` check.

**10. Scan the anti-patterns — CLI.** Run `devforgeai antipattern scan --id <STORY-nnn>`. Failure path: exit 1 prints one `DFA-E270` line per match; invoke `refactor-surgeon` with the match list, run the test command, commit with `-m "AP remediation"`, and run the scan once more. A second exit 1 leaves the matches for the `build-antipatterns` check.

**11. Verify the criteria — `story-ac-verifier`, one invocation.** Input: three prompt fields and no others — the absolute path of `.devforgeai/stories/STORY-nnn.md`, the `data` object of `devforgeai story files --diff --id <STORY-nnn> --json`, and the merged stdout and stderr of the last test command run. It returns one `payload.checks` entry per criterion, each with a `verdict` of `met` or `unmet`. The prompt carries no part of this conversation and no output of steps 7.1 through 10, which is what makes the verdict independent of the session that wrote the code. Its stdout reaches `verifiers.story_ac` through `SubagentStop`. Failure path: as step 9, with the block at `status: unparsed` failing `build-acs`.

**12. Record the run — model.** Write `.devforgeai/build/STORY-nnn-note.yaml` from `templates/build-note.yaml` with the cycle data of step 7, the refactor entries of 7.8, the integration entry of step 8, and the commit shas. The path matches no `doc validate` doc-type row, so no producer check applies and no hook reads it.

**13. Merge the record into the report — CLI.** Run `devforgeai report note <STORY-nnn> build --key build --file .devforgeai/build/STORY-nnn-note.yaml`. The CLI drops the fragment's `schema` and `id` keys and writes the rest at the report's top-level `build` key. This is the one path by which this skill contributes to a CLI-owned report; a Write to the report itself is refused with `DFA-E212`. Failure path: exit 1 on `DFA-E413` names the offending key; rewrite that key and run the call once more.

**14. Close — CLI, Stop hook.** The Stop hook runs `devforgeai gate check --phase build`, which writes `.devforgeai/reports/STORY-nnn-build.yaml`, then `devforgeai handoff`. The Stop hook renders the closing block; this skill writes no part of it. On a gate FAIL the Stop hook holds the turn with the failing checks as its `reason`, and the block appears when the turn ends; on a SEND BACK it renders the block with no block on the turn, because the next step is a command the user types.

## Subagents

Contracts, tools, models, and invocation order are in `agents.md`. Full schemas are in each `agents/<name>.md` `## Output`.

| Subagent | Invoked at | What to pass | What comes back |
|---|---|---|---|
| `ac-test-writer` | step 7.1, alone, once per criterion, serially | the one `AC-nnn` line, the other criteria of the story, the `## Files` rows of `Kind` `test`, the `## Testing standards` and `## Naming conventions` rows, the current content of each declared test file, the `entities[]` of `.devforgeai/explore/seed-data.json` as `seed_rows` when that file exists, and on a remedy run the `FIND-nnn` summary and evidence | `devforgeai/verifier/1`: `passed`, `total`, `unit: AC`, `findings[]`, and `payload` holding `testable`, `ac`, `test_paths`, `assertion`, `conflicts_with`; a registered verifier |
| `backend-implementer` | step 7.4, alone, once per criterion; once more at 7.5 on a failing rerun | the `AC-nnn` line, the failing test paths with their content and output, the `## Files` rows of `Kind` `source`, `config`, `migration`, `asset`, the `## Layer` value, the `CON-nnn` and `AP-nnn` rows, the approved and forbidden dependency tables, the `## Error handling`, `## Logging`, and `## Formatting` sections, the source paths earlier cycles wrote | `backend-implementer/1`: `ac`, `status`, `source_paths`, `blocked`, `notes` |
| `frontend-implementer` | step 7.4, in place of `backend-implementer` when `## Layer` is `interface` and `## Interface` carries a row | the same fields minus the dependency tables, plus the `UI-nnn` tables of step 3 and the `tokens.json` leaf names per group | the `backend-implementer` object with `subagent` of `frontend-implementer`, plus `ui` and `tokens_used` |
| `refactor-surgeon` | step 7.8 on a failing check, and step 10 on an anti-pattern match; serially | the failing check ids, each `reason` string, the `AP-nnn` match list when the trigger is `build-antipatterns`, the paths the cycle wrote, `[build].complexity_max` and `[build].duplication_max_percent`, the `## Formatting` and `## Naming` rows | `refactor-surgeon/1`: `trigger`, `status`, `paths`, `changes[]`, `declined` |
| `integration-test-writer` | step 8, alone, after the loop ends | every `AC-nnn` line, the `## Files` rows of `Kind` `test`, the source paths of every cycle with their content, the `UI-nnn` `## Interaction` and `## States` tables, the `## Layer dependency rules` and `## Testing standards` rows | `integration-test-writer/1`: `status`, `test_paths`, `scenarios[]`, `not_applicable_reason` |
| `context-validator` | step 9, alone, after step 8 and before step 11 | the `--diff --json` `data` object, the six context file paths, the `CON-nnn` rows, the `## Layer` value | `devforgeai/verifier/1`: `passed`, `total`, `unit: files`, `findings[]`; a registered verifier |
| `story-ac-verifier` | step 11, alone, after steps 8, 9, and 10 | three fields alone: the story path, the `--diff --json` `data` object, the merged test output | `devforgeai/verifier/1`: `passed`, `total`, `unit: ACs`, `findings[]`, and `payload.checks[]` one per criterion, each carrying a `verdict` of `met` or `unmet`; a registered verifier |

Three of the seven are registered verifiers, each with its own `verifier_pass` check, so one ratio does not compensate for another. The other four return run-local JSON this skill consumes, which `SubagentStop` ignores.

## Documents

| Document | Path | Template |
|---|---|---|
| Build note | `.devforgeai/build/STORY-nnn-note.yaml`, one per story | `templates/build-note.yaml` |
| Config fragment | merged into `.devforgeai/config.toml` by `devforgeai init` | `templates/build-config.toml` |

Source, test, config, migration, and asset files inside the story's `## Files` set are the run's third output and carry no frontmatter; they live in the worktree, not under `.devforgeai/`.

The build note is the skill's own record and the only file it writes under `.devforgeai/`. Its path matches no `doc validate` doc-type row, so the seven-key frontmatter rule of conventions §5 does not reach it: it carries `schema` and `id` as ordinary top-level keys and `report note` drops both when merging the rest under the report's `build` key. Every `test_paths` and `source_paths` entry is a `Path` value of the story's `## Files` table. A finished cycle entry carries two different 40-hex values in `red_commit` and `green_commit`; an unfinished one carries `green_commit` of `""` and `source_paths` of `[]`, which is the state `--resume` re-enters at.

`.devforgeai/reports/STORY-nnn-build.yaml` is this phase's conventions §5 document. The CLI writes it: `produced_by` is `devforgeai-cli`, `report ingest` fills the three verifier blocks from `SubagentStop`, `report note` fills the top-level `build` key at step 13, and `gate check` fills the rest.

## Send-back

Build has one upstream target, Plan. Four conditions raise it, each a blocking finding in a verifier block.

| Id | Condition | Detected by | IDs cited |
|---|---|---|---|
| SB-1 | A criterion's `Then` clause names no outcome a test reads, its `When` clause names no action, or its `Given` clause names no reachable state | an `ac-test-writer` finding at `severity: block` with `reason` in `then_names_no_readable_outcome`, `when_names_no_action`, `given_names_no_reachable_state` | the `AC-nnn` of each such finding |
| SB-2 | Two criteria of the story assert opposed outcomes for one state and action | an `ac-test-writer` finding at `severity: block` with `reason` of `contradicts_other_ac` | the criterion worked and every id in its `payload.conflicts_with` list |
| SB-3 | A criterion's outcome lives in a file `## Files` does not declare | an `ac-test-writer` finding at `severity: block` with `reason` of `outcome_outside_declared_files` | the `AC-nnn` of each such finding |
| SB-4 | A criterion the run implemented is not satisfied by the diff and the test output read in a fresh context | a `story-ac-verifier` finding at `severity: block` | the `AC-nnn` of each such finding |

On any of the four the worktree stays on disk with every commit the run made, the story file keeps `status: building`, the main checkout's `state.toml` keeps `[current].phase` of `build` and `[active].build` of the story id, and no byte of `.devforgeai/stories/`, `.devforgeai/context/`, or `.devforgeai/ui-specs/` changes. The Stop hook renders the closing block from the report; this skill writes no part of it. The two lines it carries:

```
Next      /plan EPIC-nnn --remedy AC-nnn,AC-nnn
Then      /build STORY-nnn --resume
```

`EPIC-nnn` is the `epic` key of `sprint.yaml`, which the preamble printed. Build receives one send-back, from Verify, arriving as `/build STORY-nnn --remedy FIND-nnn,...`.

## Remedy and resume

**`--remedy FIND-nnn,...`** arrives from Verify. Step R1 replaces step 1's tail and steps 4 through 14 then run with the R1 criterion set and `run: remedy`.

**R1. Resolve the findings — model, CLI.** Run `devforgeai doc load qa-report <STORY-nnn>` and take, for each cited id, the `findings[]` entry whose `id` matches, producing one `(FIND-nnn, AC-nnn, summary, evidence)` quadruple. The criterion set of step 7 is the distinct `AC-nnn` values in `## Acceptance Criteria` order, and step 7.1 receives each criterion's `summary` and `evidence` as two further prompt fields, so the new test asserts the gap the finding named. Failure path: a cited `FIND-nnn` appearing in no `findings[]` entry stops the run with one `Blocked` line naming the id.

Step 4 reuses the existing worktree. Criteria outside the set keep their tests, their code, and their commits: no file they own is written, which `story files --diff` records as an unchanged path. The note's `remedy` key carries the cited `FIND-nnn` list and `cycles` holds one entry per criterion of the R1 set alone.

**`--resume`** re-enters at step 2 and re-reads the story and the six context files from the preamble's stdout, so a criterion Plan rewrote is the text the run works from. Step 4 reuses the worktree and step 5 is a no-op that leaves `status` at `building`. The criterion set begins at the first `AC-nnn` with no `cycles` entry in `.devforgeai/build/STORY-nnn-note.yaml`, or whose entry carries an empty `green_commit`, and runs to the end of `## Acceptance Criteria`. The note's `resumed_at` records that id and `cycles` keeps every earlier entry byte for byte. An absent note makes the criterion set the whole list, which is the `full` run.

`references/remedy-resume.md` carries both runs step by step.

## References

- `references/tdd-cycle.md` — read before the first criterion of step 7: steps 7.1 through 7.8 field by field, which subagent each invokes, the retry budget, the six commit messages, and the two loop stops that produce a send-back.
- `references/worktree-rules.md` — read at step 4: what `worktree ensure` creates, which directory each kind of command runs in, how two parallel builds stay apart, and the `DFA-E272` refusal and what clears it.
- `references/remedy-resume.md` — read at R1 and before a `--resume` run: how a `FIND-nnn` resolves to a criterion, what each run leaves untouched, and how the resume cursor is read from the note.
- `references/test-shapes.md` — read at step 7.1 before the first criterion: how a criterion's three clauses become the three parts of one failing test case, and the five reasons `ac-test-writer` returns `payload.testable` of `false`.
- `references/negative-paths.md` — read at step 7.1 for a criterion whose `Then` clause names a refusal, an error, or an empty result: the three readings of a failure and the edges worth an assertion.
- `references/layering.md` — read at step 7.4 before the first source path: which part of a criterion belongs at which declared path, the one direction rule of `## Layer dependency rules`, and the four `blocked.reason` values.
- `references/rewrites.md` — read at step 7.8 and at step 10: which of the seven `pattern` values answers `build-lint`, `build-complexity`, or `build-antipatterns`, and the three `declined.reason` values.
- `references/boundary-tests.md` — read at step 8: what a test at a layer edge adds over the unit tests of step 7, and which of the four `boundary` values a scenario carries.
