---
schema: devforgeai-spec/1
doc: build
status: draft
produced_by: orchestrator
consumes: [00-conventions]
open_questions: []
---

# Phase 4 · Build · `implementing-stories`

## Scope

This component takes one `STORY-nnn` that Plan marked `ready` and turns it into committed source and test files inside a git worktree of the project, under test-first discipline: one failing test per `AC-nnn` before the code that satisfies it, one commit per red and per green, a refactor pass opened by a CLI number rather than by taste, an integration pass for stories that carry an interface, and a fresh-context verifier that reads the story, the diff, and the test output and decides each `AC-nnn` on its own. Its subject is a single story. Its writable set is the `## Files` table of that story and nothing else, which the `PreToolUse` `story files --check` hook enforces at the moment of the write.

This component decides nothing about what the story says. It does not edit `.devforgeai/stories/STORY-nnn.md`, `sprint.yaml`, the six context files, or any `UI-nnn.md`: the producer check refuses those writes from the build phase, and §1.6 sends a defect upstream instead. It runs no git command of its own — every worktree operation and every commit goes through `devforgeai`, so the hooks that §7 installs run on each one. It interprets no test count, coverage figure, lint result, or complexity number: those are `gate check` evidence. It releases nothing and it judges no finished story against the requirement behind it, which is Verify's subject.

## Inputs

| Document | Path | IDs read | Sections or keys read |
|---|---|---|---|
| Story | `.devforgeai/stories/STORY-nnn.md` | `STORY-nnn`, `AC-nnn`, `CON-nnn`, `AP-nnn`, `UI-nnn`, `REQ-nnn` | all ten H2 sections and the frontmatter `status` and `consumes` |
| Sprint | `.devforgeai/stories/sprint.yaml` | `SPRINT-nnn`, `EPIC-nnn`, `STORY-nnn` | `epic`, `stories[].id`, `stories[].status`, `stories[].order` |
| Context, six files | `.devforgeai/context/{tech-stack,source-tree,dependencies,coding-standards,architecture-constraints,anti-patterns}.md` | `CON-nnn`, `AP-nnn` | `## Layers`, `## File placement rules`, `## Naming conventions`, `## Testing standards`, `## Error handling`, `## Logging`, `## Approved dependencies`, `## Forbidden dependencies`, `## Constraints`, `## Constraint index`, `## Layer dependency rules`, `## Anti-patterns`, `## Anti-pattern index` |
| UI spec | `.devforgeai/ui-specs/UI-nnn.md` | `UI-nnn`, `TOKEN-<name>` | `## Purpose`, `## Anatomy`, `## States`, `## Breakpoints`, `## Interaction`, `## Accessibility`, `## Tokens used` |
| Design tokens | `.devforgeai/brand/tokens.json` | `TOKEN-<name>` | `color`, `type`, `spacing`, `radius`, `elevation`, `motion` |
| QA report | `.devforgeai/reports/STORY-nnn-qa.yaml` | `FIND-nnn`, `AC-nnn` | `findings[]`, read on a `--remedy` run alone |
| Configuration | `.devforgeai/config.toml` | none | `[[stack]].test_command`, `[[stack]].lint_command`, `[[stack]].complexity_command`, `[[stack]].source_roots`, `[[layer]].coverage_min`, `[build]`, `[frontend].globs`, `degraded` |
| State | `.devforgeai/state.toml` | `STORY-nnn`, `SPRINT-nnn` | `[current].phase`, `[current].id`, `[active].build`, `[active].plan`, `[active].release` |
| Explore seed data | `.devforgeai/explore/seed-data.json` | none | `entities[].rows`, read when the file exists |

The story reaches the model on the preamble's stdout through `devforgeai doc load story $1`, the six context files through `devforgeai doc load context all`, and the sprint through `devforgeai doc load sprint -`. The `UI-nnn.md` files reach it at workflow step 3, one `devforgeai doc load ui-spec <UI-nnn>` call per row of the story's `## Interface` table. The QA report reaches it at remedy step R1 through `devforgeai doc load qa-report <STORY-nnn>`.

`.devforgeai/explore/seed-data.json` is the example data Explore shaped in Phase 0 and carried forward on a `promote` decision, which is what `carry_forward` entry 6 of `explore/decision.yaml` names `implementing-stories` for. Its `entities[].rows` are the fixture rows step 7.1 passes to `ac-test-writer`, so a test asserts against data the user recognises rather than against invented rows. A project that skipped Explore, or one whose decision was `park` or `kill`, holds no such file; the step reads none and `ac-test-writer` invents its own rows.

The `## Files` table is the writable set. The `## Acceptance Criteria` list is the work list, in file order. `## Layer` selects the implementer subagent and the coverage layer. `## Constraints` and `## Anti-patterns` are the rules `context-validator` applies and `devforgeai antipattern scan` matches. `## Dependencies` names the `STORY-nnn` values whose `status` is at `built` or beyond before this run begins.

## Outputs

Three artifacts, in two places.

### 1. Source and test files inside the story's `## Files` set

Written into the worktree at `<config.toml [build].worktree_root>/<STORY-nnn>`, one file per `Path` value of `## Files` the run touches. No file outside that set is written: the `PreToolUse` hook runs `devforgeai story files --check <path>` and its exit 1 becomes hook exit 2, which blocks the write and returns `DFA-E239` to the model. Files of `Kind` `test` carry the tests, files of `Kind` `source` the implementation, and `config`, `migration`, and `asset` files whatever their kind names.

### 2. `.devforgeai/build/STORY-nnn-note.yaml`

The skill's own record of the run, written with Write and Edit. Its path matches no row of the `doc validate` doc-type table, so `doc validate` skips it and the producer check does not apply (`specs/01-cli.md` §`doc validate`, doc-type table, closing paragraph). It is the file `devforgeai report note` reads at workflow step 13.

```yaml
schema: devforgeai/build-note/1     # string; fixed
id: STORY-014                       # string; ^STORY-\d{3}$
run: full                           # string; enum: full | remedy | resume
worktree: ../wt/STORY-014           # string; project-relative path of the worktree
branch: story/STORY-014             # string; the branch the worktree checked out
base_commit: 9f2c1ab…               # string; 40 lowercase hex; the [build].base_ref commit the branch left
head_commit: 41ee07d…               # string; 40 lowercase hex; the last commit this run made
layer: application                  # string; the ## Layer value, a [[layer]].name
cycles:                             # array; one entry per AC the run worked, in ## Acceptance Criteria order
  - ac: AC-003                      # string; ^AC-\d{3}$
    test_paths: [tests/application/checkout/place_order_spec.ext]
                                    # array[string]; each a ## Files Path of Kind test; length >= 1
    red_commit: 3b70c12…            # string; 40 hex; the commit holding the failing test
    implementer: backend-implementer   # string; enum: backend-implementer | frontend-implementer
    green_commit: a0d4f19…          # string; 40 hex, or "" while unfinished
    source_paths: [src/application/checkout/place_order.ext]
                                    # array[string]; each a ## Files Path of Kind source, config, migration or asset
refactors:                          # array; default []; one entry per refactor pass
  - trigger: build-lint             # string; enum: build-lint | build-complexity
    reason: "<the rule name the lint or complexity command printed> at src/application/checkout/place_order.ext:41"
                                    # string; the report check's reason field, copied
    paths: [src/application/checkout/place_order.ext]   # array[string]
    commit: 7c1990e…                # string; 40 hex
integration:                        # mapping; present in every note
  ran: true                         # bool
  reason: layer                     # string; enum: layer | interface | not-applicable
  test_paths: [tests/interface/checkout/place_order_api_spec.ext]   # array[string]; [] when ran is false
  commit: c40ab72…                  # string; 40 hex, or "" when ran is false
remedy: []                          # array[string]; ^FIND-\d{3}$; the findings this run re-opened
resumed_at: ""                      # string; ^AC-\d{3}$ or ""; the AC a --resume run re-entered at
```

`cycles` holds one entry per `AC-nnn` the run worked: every criterion of `## Acceptance Criteria` on a `full` run, the criteria the cited `FIND-nnn` name on a `remedy` run, and the criteria from `resumed_at` onward on a `resume` run. Every `test_paths` and `source_paths` entry is a `Path` value of the story's `## Files` table. A finished entry carries two different 40-hex values in `red_commit` and `green_commit`, which is the record that the test existed and failed before the code existed. An entry whose criterion the run wrote a test for and did not finish carries `green_commit` of `""` and `source_paths` of `[]`, which is the state `--resume` re-enters at.

### 3. `.devforgeai/reports/STORY-nnn-build.yaml`

The §5 document of this phase, written by `devforgeai gate check --phase build` and `devforgeai report ingest`, with `produced_by: devforgeai-cli`. Its schema is `specs/01-cli.md` §Outputs. The skill contributes exactly two things to it and writes neither of them itself.

| Contribution | Path in the report | Written by |
|---|---|---|
| Three verifier blocks | `verifiers.ac_testable`, `verifiers.story_ac`, `verifiers.context` | `report ingest`, run by the `SubagentStop` hook, from the `last_assistant_message` of `## Subagents` |
| The run record | the top-level key `build`, holding the mapping of output 2 with its `schema` and `id` keys dropped | `report note`, run by the model at workflow step 13 |

The `build-report` doc-type row permits top-level keys beyond the §5 seven (`specs/01-cli.md` §Decisions 44), so the `build` key passes the `pre-commit` `doc validate` run. `findings[]` merges the findings of the three verifier blocks, de-duplicated by `id`, which is the list `devforgeai handoff` renders `Found` lines from and composes the `--remedy` list from.

## Workflow

**1. Establish the run — model.** Input: `$ARGUMENTS`, the stdout of the four preamble lines. The run kind is `remedy` when `$ARGUMENTS` contains `--remedy`, `resume` when it contains `--resume`, and `full` otherwise. The subject is `$1`, a `STORY-nnn`. The epic is the `epic` key of the `sprint.yaml` on the preamble's stdout. Output: a run kind, a `STORY-nnn`, an `EPIC-nnn`. Failure path: `$1` carries a prefix other than `STORY`; the run stops with one `Blocked` line naming the accepted prefix.

**2. Read the story and the context set — model.** Input: the story text and the six context files on the preamble's stdout. Output: the ordered `AC-nnn` list from `## Acceptance Criteria`, the `Path` set from `## Files` with each row's `Kind` and `Layer`, the `## Layer` value, the `CON-nnn` rows of `## Constraints`, the `AP-nnn` rows of `## Anti-patterns`, the `STORY-nnn` list of `## Dependencies`, the `## Testing standards` rows of `coding-standards.md`, and the `## File placement rules` and `## Naming conventions` rows of `source-tree.md`. Failure path: the story's frontmatter `status` is `draft`; the run stops with one `Blocked` line naming the status and `/plan <EPIC-nnn> --resume`.

**3. Read the screen specs — model, CLI.** Input: the `UI-nnn` values of the story's `## Interface` table, zero to four of them. Actor: the model runs `devforgeai doc load ui-spec <UI-nnn>` once per row, and `devforgeai config get frontend.tokens_path` followed by a Read of the printed path when at least one row exists. Output: the `## Anatomy`, `## States`, `## Breakpoints`, `## Interaction`, `## Accessibility`, and `## Tokens used` tables of each screen, and the token leaf set. Failure path: `doc load` exits 1 on `DFA-E200`; the run stops with one `Blocked` line naming the `UI-nnn` and `/design <UI-nnn> --spec`.

**4. Open the worktree — CLI.** Input: the `STORY-nnn` from step 1. Actor: the model runs `devforgeai worktree ensure <STORY-nnn>`, which prints the worktree path on stdout. Output: a worktree at `<[build].worktree_root>/<STORY-nnn>` on branch `<[build].branch_prefix><STORY-nnn>`, the path the rest of the run works in, and one `[[worktree]]` entry appended to the main checkout's `.devforgeai/state.toml` carrying `story`, `path`, `branch` and `created_at`. The worktree carries no state file of its own — there is one, at the root, and that entry is how the rest of the framework finds the sources. `ensure` is idempotent per story, so a resume run re-running this step appends nothing. Failure path: exit 1 on `DFA-E272`, meaning another worktree under `worktree_root` holds a story whose `## Files` set intersects this one; the run stops with one `Blocked` line naming both story ids and the shared path. Exit 1 on `DFA-E271`, meaning the project root is not a git work tree; the run stops with one `Blocked` line naming `git init`.

**5. Activate the phase — CLI.** Input: the `STORY-nnn`. Actor: the model runs `devforgeai phase set build --id <STORY-nnn>`. Output: `state.toml` `[current].phase` of `build`, `[current].id` and `[active].build` of the story id, and the story file's frontmatter `status` moved to `building` by the CLI. The call may be made from the worktree or from the root and reaches the same file either way: project discovery inside a registered worktree resolves `.devforgeai/` at the main checkout through `git rev-parse --git-common-dir`. From this point the `PreToolUse` `story files --check` hook guards every write: a path under a registered worktree is checked against that entry's `story`, and any other path against `[active].build`. That is what lets a second non-overlapping story build in parallel — `worktree ensure` has already refused an overlapping file set with `DFA-E272`. Running this call from inside a registered worktree sets `[active].build` to that worktree's story. Failure path: exit 1 on `DFA-E320`, meaning the plan gate is not PASS; the run stops and the Stop hook prints the gate result.

**6. Read the commands — CLI.** Input: `config.toml`. Actor: the model runs `devforgeai config get stack.test_command`. Output: the command string the model runs with Bash at steps 7.2 and 7.5. Failure path: the printed value is empty, which `degraded = true` produces; the run stops with one `Blocked` line naming `devforgeai stack detect`.

**7. The per-criterion loop — subagents, model, CLI.** Input: the `AC-nnn` list from step 2, filtered to the run's criterion set: every criterion on a `full` run, the criteria that remedy step R1 resolved on a `remedy` run, and the criteria from `resumed_at` onward on a `resume` run. The loop body runs once per criterion, in `## Acceptance Criteria` order, and the criterion set is worked to exhaustion or to the first stop.

**7.1 Write the failing test — `ac-test-writer`, one invocation per criterion.** Input: the one `AC-nnn` line, the `## Files` rows of `Kind` `test`, the `## Testing standards` rows, the `## Naming conventions` rows, the existing content of the test files the story declares, and the other `AC-nnn` lines of the story for the contradiction judgment. Output: the subagent's JSON, and one or more test files written inside the declared set. The `SubagentStop` hook runs `devforgeai report ingest ac-test-writer -`, which writes `verifiers.ac_testable` into the report. Failure path: the JSON carries `payload.testable: false` with `passed: 0` and one `block` finding; the loop stops and the run continues at step 12, which produces the send-back of `## Send-back`.

**7.2 Run the tests — model.** Input: the command string from step 6. Actor: the model runs it with Bash in the worktree. Output: the command's exit code and output on the model's transcript. The `PostToolUse` Bash hook matches the command against `config.toml` and runs `devforgeai gate check --phase build --partial`, which writes `reports/STORY-nnn-build.yaml` with `partial: true` and returns its lines in `hookSpecificOutput.additionalContext` on the next turn, blocking nothing. Failure path: the exit code is 0, meaning the new test passed before any implementation; the loop stops, the cycle entry keeps `green_commit` of `""`, and the run continues at step 12. `ac-test-writer` returned `payload.testable: true` for this criterion and its block reads 1.0, so `build-testable` passes and `build-acs` catches the criterion instead: `story-ac-verifier` at step 11 reads a diff with no hunk implementing it and returns `no_diff_hunk_implements_it`, which produces SB-4.

**7.3 Commit the red — CLI.** Input: the test paths the subagent wrote. Actor: the model runs `devforgeai commit <STORY-nnn> -m "<AC-nnn> red"`. Output: the commit sha, printed on stdout, recorded as the cycle's `red_commit`. Failure path: exit 1 from the `pre-commit` hook's `doc validate` or `context audit`; the run stops with one `Blocked` line carrying the hook's stderr.

**7.4 Make it pass — `backend-implementer` or `frontend-implementer`, one invocation per criterion.** Input: the one `AC-nnn` line, the failing test paths and their content, the `## Files` rows of `Kind` `source`, `config`, `migration`, and `asset`, the `## Layer` value, the `CON-nnn` rows, the `AP-nnn` rows, the `## Approved dependencies` and `## Forbidden dependencies` tables, the `## Error handling`, `## Logging`, and `## Formatting` sections; and for `frontend-implementer` the `UI-nnn` tables of step 3 and the token leaf set. The subagent is `frontend-implementer` when the story's `## Layer` is `interface` and `## Interface` carries at least one row, and `backend-implementer` otherwise. Output: the subagent's JSON and the source files it wrote inside the declared set. Failure path: the JSON carries `blocked` with a reason from its enum; the loop stops and the run continues at step 12.

**7.5 Run the tests again — model.** Input: the command string from step 6. Actor: the model runs it with Bash in the worktree; the same `PostToolUse` hook fires. Output: the exit code. Failure path: a non-zero exit code invokes `backend-implementer` or `frontend-implementer` once more with the failing output appended; a second non-zero exit stops the loop and the run continues at step 12.

**7.6 Commit the green — CLI.** Input: the source paths the subagent wrote. Actor: the model runs `devforgeai commit <STORY-nnn> -m "<AC-nnn> green"`. Output: the commit sha, recorded as the cycle's `green_commit`. Failure path: as step 7.3.

**7.7 Read the quality checks — CLI.** Input: the report the `PostToolUse` hook wrote at step 7.5. Actor: the model runs `devforgeai report show <STORY-nnn> build --check build-lint` and `devforgeai report show <STORY-nnn> build --check build-complexity`. Output: two check entries, each with a `status` of `pass`, `fail`, or `skip`, and a `reason` string. Failure path: exit 1 on `DFA-E400`, meaning no partial report exists; the model runs the test command once more, which makes the hook write it.

**7.8 Refactor — `refactor-surgeon`, zero or one invocation per criterion.** The subagent is invoked when at least one of the two check entries of step 7.7 carries `status: fail`, and is not invoked otherwise. Input: the failing check ids and their `reason` strings, the paths the cycle wrote, `[build].complexity_max` and `[build].duplication_max_percent` from `devforgeai config get`, and the `## Formatting` and `## Naming` rows. Output: the subagent's JSON and the rewritten source files. The model then runs the test command, and runs `devforgeai commit <STORY-nnn> -m "<AC-nnn> refactor"` when the exit code is 0. Failure path: a non-zero exit code after the rewrite invokes `refactor-surgeon` once more with the failing output appended; a second non-zero exit stops the loop and the run continues at step 12.

**8. Write the integration tests — `integration-test-writer`, zero or one invocation.** The subagent is invoked when the story's `## Layer` is `interface`, or when `## Interface` carries at least one row, and is not invoked otherwise. Input: every `AC-nnn` line, the `## Files` rows of `Kind` `test`, the source paths of every cycle, the `UI-nnn` `## Interaction` and `## States` tables when they exist, and the `## Layer dependency rules` rows. Output: the subagent's JSON and the integration test files inside the declared set. The model then runs the test command and runs `devforgeai commit <STORY-nnn> -m "integration"`. Failure path: a non-zero exit code invokes the subagent once more with the failing output appended; a second non-zero exit stops the run and continues at step 12.

**9. Validate the context — `context-validator`, one invocation.** Input: the path list of `devforgeai story files --diff --id <STORY-nnn>`, the six context files, the `CON-nnn` rows of `## Constraints`, and the story's `## Layer`. Output: the verifier JSON, ingested into `verifiers.context` by the `SubagentStop` hook. Failure path: the JSON does not parse against the schema in `## Subagents`; the subagent is invoked once more with the parse error appended, and a second failure leaves `verifiers.context` with `status: unparsed`, which fails the `build-context` check.

**10. Scan the anti-patterns — CLI.** Input: the story's `## Files` set. Actor: the model runs `devforgeai antipattern scan --id <STORY-nnn>`. Output: exit 0, or exit 1 with one `DFA-E270` line per match. Failure path: exit 1; the model invokes `refactor-surgeon` with the match list as its input, then runs the test command and `devforgeai commit <STORY-nnn> -m "AP remediation"`, then runs the scan once more. A second exit 1 leaves the matches for the `build-antipatterns` gate check.

**11. Verify the criteria — `story-ac-verifier`, one invocation.** Input: the story path, the output of `devforgeai story files --diff --id <STORY-nnn> --json`, and the merged stdout and stderr of the last test command run, passed as three prompt fields. The subagent receives no other text and holds no part of this conversation. Output: the verifier JSON with one `payload.checks[]` entry per `AC-nnn` of the story, each carrying `verdict` of `met` or `unmet` and a `confidence`, ingested into `verifiers.story_ac` by the `SubagentStop` hook from the payload's `last_assistant_message`. Failure path: as step 9, with `verifiers.story_ac` carrying `status: unparsed`, which fails the `build-acs` check.

**12. Record the run — model.** Input: the cycle data of step 7, the refactor entries of step 7.8, the integration entry of step 8, and the commit shas. Actor: the model writes `.devforgeai/build/STORY-nnn-note.yaml` from `templates/build-note.yaml`. Output: the file. Failure path: none; the path matches no doc-type row and no hook reads it.

**13. Merge the record into the report — CLI.** Input: the file of step 12. Actor: the model runs `devforgeai report note <STORY-nnn> build --key build --file .devforgeai/build/STORY-nnn-note.yaml`. Output: the top-level `build` key of `.devforgeai/reports/STORY-nnn-build.yaml`. Failure path: exit 1 on `DFA-E413`, meaning the fragment fails the `devforgeai/build-note/1` schema; the model rewrites the named key and runs the call once more.

**14. Close — CLI, Stop hook.** Input: `state.toml`. Actor: the Stop hook runs `devforgeai gate check --phase build`, then `devforgeai handoff`. Output: the full `reports/STORY-nnn-build.yaml` and the §6 block. Failure path: a FAIL exits 2 with `decision: "block"` and the failing checks in `reason`, under a budget of three blocks per `session_id`; at the last block of that budget the hook exits 0 and the FAIL block renders in `systemMessage`, per §7. A SEND BACK exits 0 with the SEND BACK block of `## Handoff` in `systemMessage` and blocks nothing, because the next step is a command the user types.

### Remedy workflow, `/build <STORY-nnn> --remedy FIND-nnn,...`

**R1. Resolve the findings — model, CLI.** Input: the id list after `--remedy`, and `.devforgeai/reports/STORY-nnn-qa.yaml` through `devforgeai doc load qa-report <STORY-nnn>`. Output: one `(FIND-nnn, AC-nnn, summary, evidence)` quadruple per cited id, taken from the `findings[]` entry whose `id` matches. The criterion set of step 7 is the distinct `AC-nnn` values of those quadruples, in `## Acceptance Criteria` order. Failure path: a cited `FIND-nnn` appears in no `findings[]` entry; the run stops with one `Blocked` line naming the id.

**R2. Re-enter — model.** The run executes steps 4 through 14 with the criterion set of R1 and `run: remedy`. Step 4 reuses the existing worktree. Step 7.1 receives each criterion's `summary` and `evidence` as two further prompt fields, so the new test asserts the gap the finding named. Criteria outside the set keep their tests, their code, and their commits: no file they own is written, which `story files --diff` records as an unchanged path.

**R3. Record — model.** The note's `remedy` key carries the cited `FIND-nnn` list, and `cycles` holds one entry per criterion of the R1 set alone.

### Resume workflow, `/build <STORY-nnn> --resume`

Re-enters at step 2 and re-reads the story and the six context files from the preamble's stdout, so a criterion Plan rewrote is the text the run works from. Step 4 reuses the worktree, and step 5 is a no-op that leaves `status` at `building`. The criterion set of step 7 begins at the first `AC-nnn` whose entry is absent from the `cycles` list of `.devforgeai/build/STORY-nnn-note.yaml`, or whose entry carries an empty `green_commit`, and runs to the end of `## Acceptance Criteria`. The note's `resumed_at` records that id, and `cycles` keeps every earlier entry byte for byte. An absent note file makes the criterion set the whole list, which is the `full` run.

## Subagents

Seven subagents. Three are registered verifiers. None of the seven writes under `.devforgeai/`, and the three that write source or test files write inside the story's `## Files` set, which the `PreToolUse` hook enforces against the subagent's writes as it does against the model's.

### ac-test-writer

- **name**: `ac-test-writer`
- **derives_from**: `C:\Users\bryan\.claude\agents\test-automator.md`
- **purpose**: Turn one acceptance criterion into one failing test, or report that the criterion cannot be turned into one.
- **tools**: Read, Write, Edit, Grep, Glob
- **model**: `opus` — its `payload.testable` and `payload.conflicts_with` verdicts produce the send-back to Plan, and a wrong verdict either hides a specification defect or returns a good story.
- **input**: the `AC-nnn` id and its full `Given … When … Then …` line; the other `AC-nnn` lines of the story; the `## Files` rows of `Kind` `test` as `{path, layer}`; the `## Testing standards` rows of `coding-standards.md`; the `## Naming conventions` rows of `source-tree.md`; the current content of each declared test file; on a remedy run the `FIND-nnn` summary and evidence.
- **output**:

```json
{
  "type": "object",
  "required": ["schema", "subagent", "id", "passed", "total", "unit", "findings", "payload"],
  "properties": {
    "schema":   { "const": "devforgeai/verifier/1" },
    "subagent": { "const": "ac-test-writer" },
    "id":       { "type": "string", "pattern": "^STORY-[0-9]{3}$" },
    "passed":   { "type": "integer", "enum": [0, 1] },
    "total":    { "type": "integer", "const": 1 },
    "unit":     { "const": "AC" },
    "findings": { "type": "array", "items": {
      "type": "object",
      "required": ["id", "severity", "confidence", "summary", "evidence"],
      "properties": {
        "id":       { "type": "string", "pattern": "^AC-[0-9]{3}$" },
        "severity": { "enum": ["block", "warn", "info"] },
        "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
        "reason":   { "enum": ["then_names_no_readable_outcome", "when_names_no_action",
                               "given_names_no_reachable_state", "contradicts_other_ac",
                               "outcome_outside_declared_files"] },
        "summary":  { "type": "string", "maxLength": 120 },
        "evidence": { "type": "string", "maxLength": 300 }
      } } },
    "payload": {
      "type": "object",
      "required": ["testable", "ac", "test_paths"],
      "properties": {
        "testable": { "type": "boolean" },
        "ac":       { "type": "string", "pattern": "^AC-[0-9]{3}$" },
        "test_paths": { "type": "array", "items": { "type": "string" } },
        "assertion":  { "type": "string", "maxLength": 200 },
        "conflicts_with": { "type": "array", "items": { "type": "string", "pattern": "^AC-[0-9]{3}$" } }
      } }
  }
}
```

  One object, never two: the envelope keys at the top and this agent's own fields under `payload`. `total` is `1` on every invocation, because one call reads one criterion. `passed` is `1` with `payload.testable` true and `0` otherwise, which is `total` minus one when a `block` finding stands. `findings` is `[]` when `payload.testable` is true, and every finding this agent raises is `block`, so there is no `warn` here that could lower `passed`. Each finding carries `confidence`, `0.0` to `1.0`, for how firmly the clause reads; an uncertain finding is reported at a lower confidence rather than withheld. A `contradicts_other_ac` reason emits one finding per id in `payload.conflicts_with` plus one for `payload.ac`.
- **invoked_at**: workflow step 7.1, once per criterion, serially. Nothing runs in parallel with it.
- **registered_verifier**: yes, `verifiers.ac_testable`.

### backend-implementer

- **name**: `backend-implementer`
- **derives_from**: `C:\Users\bryan\.claude\agents\backend-architect.md`
- **purpose**: Write the smallest code inside the declared file set that makes one named failing test pass without breaking the tests already green.
- **tools**: Read, Write, Edit, Grep, Glob
- **model**: `opus` — it places code into a layer under `## Layer dependency rules` and the `CON-nnn` set, and a layering error costs the gate two checks and a rewrite.
- **input**: the `AC-nnn` line; the failing test paths and their content; the failing test output; the `## Files` rows of `Kind` `source`, `config`, `migration`, and `asset` as `{path, layer}`; the `## Layer` value; the `CON-nnn` rows of `## Constraints` as `{id, statement, binds}`; the `AP-nnn` rows of `## Anti-patterns` as `{id, severity, scope}`; the `## Approved dependencies` and `## Forbidden dependencies` tables; the `## Error handling`, `## Logging`, and `## Formatting` sections; the source paths earlier cycles wrote.
- **output**:

```json
{
  "type": "object",
  "required": ["subagent", "ac", "status", "source_paths", "notes"],
  "properties": {
    "subagent": { "const": "backend-implementer" },
    "ac":       { "type": "string", "pattern": "^AC-[0-9]{3}$" },
    "status":   { "enum": ["implemented", "blocked"] },
    "source_paths": { "type": "array", "items": { "type": "string" }, "minItems": 0 },
    "blocked":  { "type": "object",
      "required": ["reason", "detail"],
      "properties": {
        "reason": { "enum": ["no_declared_path_fits", "constraint_forbids_the_only_shape",
                             "dependency_not_approved", "test_asserts_outside_declared_files"] },
        "detail": { "type": "string", "maxLength": 300 },
        "ids":    { "type": "array", "items": { "type": "string", "pattern": "^(CON|AP)-[0-9]{3}$" } }
      } },
    "notes": { "type": "array", "items": { "type": "string", "maxLength": 160 }, "maxItems": 5 }
  }
}
```

  `blocked` is present when `status` is `blocked` and absent otherwise.
- **invoked_at**: workflow step 7.4, once per criterion, serially, and once more at step 7.5 on a failing rerun.
- **registered_verifier**: no.

### frontend-implementer

- **name**: `frontend-implementer`
- **derives_from**: `C:\Users\bryan\.claude\agents\frontend-developer.md`
- **purpose**: Write the component code inside the declared file set that makes one named failing test pass and renders the screen the `UI-nnn` spec describes.
- **tools**: Read, Write, Edit, Grep, Glob
- **model**: `sonnet` — the `UI-nnn` tables fix the anatomy, the states, the four breakpoints, the interactions, and the eight accessibility rows, `tokens.json` fixes every colour and type value, and the `PreToolUse` `design lint` hook blocks a literal that resolves to no token, so the remaining choice is transcription.
- **input**: the `AC-nnn` line; the failing test paths, their content, and the failing output; the `## Files` rows of `Kind` `source` and `asset`; the `UI-nnn` `## Anatomy`, `## States`, `## Breakpoints`, `## Interaction`, `## Accessibility`, and `## Tokens used` tables; the `tokens.json` leaf names per group; the `## Formatting` and `## Naming` sections.
- **output**: One JSON object on stdout and nothing else. This agent is not a registered verifier, so the object carries no `devforgeai/verifier/1` envelope and no `SubagentStop` ingest reads it. The schema is `backend-implementer`'s with `subagent` set to this name, two further required properties, and a `blocked.reason` enum widened by two values:

```json
{
  "type": "object",
  "required": ["subagent", "ac", "status", "source_paths", "notes", "ui", "tokens_used"],
  "properties": {
    "subagent": { "const": "frontend-implementer" },
    "ac":       { "type": "string", "pattern": "^AC-[0-9]{3}$" },
    "status":   { "enum": ["implemented", "blocked"] },
    "source_paths": { "type": "array", "items": { "type": "string" }, "minItems": 0 },
    "ui": { "type": "string", "pattern": "^UI-[0-9]{3}$|^$" },
    "tokens_used": { "type": "array", "items": { "type": "string", "pattern": "^TOKEN-[a-z]+-[a-z0-9-]+$" } },
    "blocked":  { "type": "object",
      "required": ["reason", "detail"],
      "properties": {
        "reason": { "enum": ["no_declared_path_fits", "constraint_forbids_the_only_shape",
                             "dependency_not_approved", "test_asserts_outside_declared_files",
                             "token_missing_for_required_value", "ui_spec_omits_a_state_the_test_asserts"] },
        "detail": { "type": "string", "maxLength": 300 },
        "ids":    { "type": "array", "items": { "type": "string", "pattern": "^(CON|AP)-[0-9]{3}$" } }
      } },
    "notes": { "type": "array", "items": { "type": "string", "maxLength": 160 }, "maxItems": 5 }
  }
}
```

  `blocked` is present when `status` is `blocked` and absent otherwise. `ui` carries the screen id, or `""` for a criterion the story cites no screen for. The whole object is given rather than the two-property fragment this entry once carried: a fragment plus a sentence pointing at another entry is a schema a generator cannot read.
- **invoked_at**: workflow step 7.4, once per criterion, serially, in place of `backend-implementer`.
- **registered_verifier**: no.

### refactor-surgeon

- **name**: `refactor-surgeon`
- **derives_from**: `C:\Users\bryan\.claude\agents\refactoring-specialist.md`
- **purpose**: Rewrite the paths a failing lint, complexity, or anti-pattern result names, leaving every test green.
- **tools**: Read, Write, Edit, Grep, Glob
- **model**: `sonnet` — the trigger, the target paths, and the two numbers come from the CLI, so the subagent applies a rewrite to a named location rather than deciding whether one is warranted.
- **input**: the failing check ids from the closed set `build-lint`, `build-complexity`, `build-antipatterns`; each check's `reason` string; the `AP-nnn` match list of `devforgeai antipattern scan --json` when the trigger is `build-antipatterns`; the paths the cycle wrote; `[build].complexity_max` and `[build].duplication_max_percent`; the `## Formatting` and `## Naming` sections.
- **output**:

```json
{
  "type": "object",
  "required": ["subagent", "trigger", "status", "paths", "changes"],
  "properties": {
    "subagent": { "const": "refactor-surgeon" },
    "trigger":  { "enum": ["build-lint", "build-complexity", "build-antipatterns"] },
    "status":   { "enum": ["rewritten", "declined"] },
    "paths":    { "type": "array", "items": { "type": "string" } },
    "changes":  { "type": "array", "items": {
      "type": "object",
      "required": ["path", "pattern", "before", "after"],
      "properties": {
        "path":    { "type": "string" },
        "pattern": { "enum": ["extract-function", "extract-type", "inline", "rename",
                              "replace-conditional", "move-to-layer", "remove-duplicate"] },
        "before":  { "type": "string", "maxLength": 160 },
        "after":   { "type": "string", "maxLength": 160 }
      } } },
    "declined": { "type": "object",
      "required": ["reason", "detail"],
      "properties": {
        "reason": { "enum": ["rewrite_leaves_declared_file_set", "constraint_forbids_the_rewrite",
                             "threshold_breach_is_in_a_file_the_story_does_not_declare"] },
        "detail": { "type": "string", "maxLength": 300 } } }
  }
}
```

- **invoked_at**: workflow step 7.8, zero or one time per criterion, and workflow step 10 on an anti-pattern match. Serially.
- **registered_verifier**: no.

### integration-test-writer

- **name**: `integration-test-writer`
- **derives_from**: `C:\Users\bryan\.claude\agents\integration-tester.md`
- **purpose**: Write tests that exercise the story's criteria across the layer boundary the story crosses, rather than one unit at a time.
- **tools**: Read, Write, Edit, Grep, Glob
- **model**: `sonnet` — the contract is already written down in the `UI-nnn` `## Interaction` table and the `## Layer dependency rules` rows, and the unit tests of step 7 fix the shapes it composes.
- **input**: every `AC-nnn` line; the `## Files` rows of `Kind` `test`; the source paths of every cycle and their content; the `UI-nnn` `## Interaction` and `## States` tables when the story cites one; the `## Layer dependency rules` rows; the `## Testing standards` rows.
- **output**:

```json
{
  "type": "object",
  "required": ["subagent", "status", "test_paths", "scenarios"],
  "properties": {
    "subagent": { "const": "integration-test-writer" },
    "status":   { "enum": ["written", "not-applicable"] },
    "test_paths": { "type": "array", "items": { "type": "string" } },
    "scenarios": { "type": "array", "items": {
      "type": "object",
      "required": ["name", "acs", "boundary"],
      "properties": {
        "name": { "type": "string", "maxLength": 120 },
        "acs":  { "type": "array", "items": { "type": "string", "pattern": "^AC-[0-9]{3}$" }, "minItems": 1 },
        "boundary": { "enum": ["interface-application", "application-infrastructure",
                               "application-domain", "interface-external"] }
      } } },
    "not_applicable_reason": { "enum": ["story_crosses_no_layer_boundary", "no_declared_test_path_for_the_boundary"] }
  }
}
```

- **invoked_at**: workflow step 8, zero or one time, after the loop of step 7 ends.
- **registered_verifier**: no.

### story-ac-verifier

- **name**: `story-ac-verifier`
- **derives_from**: `C:\Users\bryan\.claude\agents\ac-compliance-verifier.md`
- **purpose**: Decide each `AC-nnn` of the story from the story text, the diff, and the test output alone.
- **tools**: Read, Grep, Glob
- **model**: `opus` — its ratio is the `build-acs` gate check, and a false pass ships an unimplemented criterion into Verify.
- **input**: three prompt fields and no others — the absolute path of `.devforgeai/stories/STORY-nnn.md`, the `data` object of `devforgeai story files --diff --id <STORY-nnn> --json` holding one entry per changed path, and the merged stdout and stderr of the last test command run. The prompt carries no part of the conversation, no subagent output from steps 7.1 through 10, and no summary of them.
- **output**:

```json
{
  "type": "object",
  "required": ["schema", "subagent", "id", "passed", "total", "unit", "findings", "payload"],
  "properties": {
    "schema":   { "const": "devforgeai/verifier/1" },
    "subagent": { "const": "story-ac-verifier" },
    "id":       { "type": "string", "pattern": "^STORY-[0-9]{3}$" },
    "passed":   { "type": "integer", "minimum": 0 },
    "total":    { "type": "integer", "minimum": 1 },
    "unit":     { "const": "ACs" },
    "findings": { "type": "array", "items": {
      "type": "object",
      "required": ["id", "severity", "confidence", "summary", "evidence"],
      "properties": {
        "id":       { "type": "string", "pattern": "^AC-[0-9]{3}$" },
        "severity": { "enum": ["block", "warn", "info"] },
        "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
        "reason":   { "enum": ["no_test_asserts_the_then_clause", "test_asserts_a_weaker_outcome",
                               "no_diff_hunk_implements_it", "test_output_shows_it_skipped"] },
        "summary":  { "type": "string", "maxLength": 120 },
        "evidence": { "type": "string", "maxLength": 300 }
      } } },
    "payload": {
      "type": "object",
      "required": ["checks"],
      "properties": {
        "checks": { "type": "array", "items": {
          "type": "object",
          "required": ["ac", "verdict", "confidence", "test_path", "source_path", "evidence"],
          "properties": {
            "ac":      { "type": "string", "pattern": "^AC-[0-9]{3}$" },
            "verdict": { "enum": ["met", "unmet"] },
            "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
            "test_path":   { "type": "string" },
            "source_path": { "type": "string" },
            "evidence":    { "type": "string", "maxLength": 300 }
          } } } } }
  }
}
```

  One object, with `checks[]` under `payload`. `total` equals the number of `AC-nnn` lines in the story, `payload.checks` holds one entry per id, and `passed` is `total` minus the number of criteria carrying a `block` finding, which is the count of `verdict: met`. Each `unmet` verdict emits one finding at `severity: block` with the same `ac` as its `id`.

  `verdict` reads `met` or `unmet` rather than `PASS` or `FAIL`: conventions §6 reserves `PASS` and `FAIL` for the handoff `Gate` line the CLI prints, and a report showing `FAIL` beside a criterion under a `Gate      PASS` line for the same run reads as a contradiction it is not.

  Each check and each finding carries `confidence`, `0.0` to `1.0`, for how far the reading of the diff and the log carries. A criterion this agent is unsure about is reported `unmet` at a lower confidence rather than passed over: the gate and the user's remedy run are what filter, and an unreported gap reaches neither.
- **invoked_at**: workflow step 11, once, after steps 8, 9, and 10 have ended.
- **registered_verifier**: yes, `verifiers.story_ac`.

### context-validator

- **name**: `context-validator`
- **derives_from**: `C:\Users\bryan\.claude\agents\context-validator.md`
- **purpose**: Decide whether each changed file obeys the six context files on the points no CLI check covers.
- **tools**: Read, Grep, Glob
- **model**: `opus` — layer-boundary and coding-standard judgments over unfamiliar code are its whole output, and the ratio is the `build-context` gate check.
- **input**: the `data` object of `devforgeai story files --diff --id <STORY-nnn> --json`; the six context file paths; the `CON-nnn` rows of the story's `## Constraints`; the story's `## Layer` value.
- **output**:

```json
{
  "type": "object",
  "required": ["schema", "subagent", "id", "passed", "total", "unit", "findings", "payload"],
  "properties": {
    "schema":   { "const": "devforgeai/verifier/1" },
    "subagent": { "const": "context-validator" },
    "id":       { "type": "string", "pattern": "^STORY-[0-9]{3}$" },
    "passed":   { "type": "integer", "minimum": 0 },
    "total":    { "type": "integer", "minimum": 0 },
    "unit":     { "const": "files" },
    "payload":  { "type": "object" },
    "findings": { "type": "array", "items": {
      "type": "object",
      "required": ["id", "severity", "confidence", "summary", "evidence"],
      "properties": {
        "id":       { "type": "string", "pattern": "^(CON|AP)-[0-9]{3}$" },
        "severity": { "enum": ["block", "warn", "info"] },
        "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
        "kind":     { "enum": ["layer_boundary_crossed", "library_substituted", "dependency_unapproved",
                               "naming_convention_broken", "error_handling_shape_broken",
                               "logging_shape_broken", "file_outside_placement_rule"] },
        "path":     { "type": "string" },
        "line":     { "type": "integer", "minimum": 1 },
        "summary":  { "type": "string", "maxLength": 120 },
        "evidence": { "type": "string", "maxLength": 300 }
      } } }
  }
}
```

  `total` equals the number of changed paths and `passed` the number carrying no `severity: block` finding.
- **invoked_at**: workflow step 9, once, after step 8 ends and before step 11.
- **registered_verifier**: yes, `verifiers.context`.

### Agents this skill replaces or retires

| Existing agent | Disposition | Reason |
|---|---|---|
| `test-automator.md` | adapted into `ac-test-writer` | Scope narrows from a suite to one criterion, the coverage-gap and test-pyramid duties move to the `coverage_min` gate check, the observation-file write is dropped, and the `testable` and `conflicts_with` verdicts are added because the send-back needs them |
| `backend-architect.md` | adapted into `backend-implementer` | Bash is removed from `tools`, since running tests is the model's step and every git call is a CLI call; the fixed `devforgeai/specs/` paths become prompt fields; `permissionMode: plan` is dropped, since the write guard is the `PreToolUse` hook; prose output becomes the JSON schema above |
| `frontend-developer.md` | adapted into `frontend-implementer` | Its package-manager Bash grant is removed and its three named component frameworks leave the description, both under §1.2; the design source becomes `UI-nnn.md` and `tokens.json`; `model` drops to `sonnet` |
| `refactoring-specialist.md` | adapted into `refactor-surgeon` | Its three test-runner Bash grants and its `Update` tool are removed; the "complexity exceeds 10" and "duplication above 5%" triggers become the `build-lint` and `build-complexity` check results with the numbers read from `config.toml`; its AST-tool reference file is dropped |
| `integration-tester.md` | adapted into `integration-test-writer` | Its container-runtime and test-runner Bash grants are removed under §1.2; the anti-gaming step is dropped, since `story-ac-verifier` reads the diff and the output in a fresh context |
| `ac-compliance-verifier.md` | adapted into `story-ac-verifier` | The XML `<acceptance_criteria>` parser is dropped, because a Plan criterion is the Markdown list item `- AC-nnn: Given … When … Then …`; the input narrows to three fields; the output becomes the `devforgeai/verifier/1` contract; the name changes because a `[[verifier]]` entry binds one name to one phase and `ac-compliance-verifier` is Verify's |
| `context-validator.md` | adapted, name kept | The `devforgeai/specs/context/` paths become `.devforgeai/context/`; the Markdown report becomes the `devforgeai/verifier/1` contract; the checks a CLI call already makes (file placement against the declared set, anti-pattern detectors) narrow to the judgment cases |
| `git-validator.md` | retired | `devforgeai worktree ensure` reports `DFA-E271` when the project root is not a git work tree, and the git hooks §7 installs carry the rest. §1.1 leaves the agent nothing to do |
| `git-worktree-manager.md` | replaced by `devforgeai worktree ensure`, `list`, `remove` | Worktree creation, idle detection, and limit enforcement are file and process operations with no judgment in them |
| `file-overlap-detector.md` | replaced by `devforgeai worktree ensure` and `devforgeai story validate` | Pre-flight overlap is `story validate --scope sprint` check 9, `DFA-E237`; concurrent overlap is the `DFA-E272` refusal of `worktree ensure`; post-flight drift is `story files --diff` |
| `dev-result-interpreter.md` | retired | §1.3 gives the end-of-phase block to `devforgeai handoff`, which renders it from the report |
| `tech-stack-detector.md` | retired | Already retired by `specs/04-constitute.md` §Decisions 16; detection is `devforgeai stack detect` |
| The three AST-tool reference files under `C:\\Users\\bryan\\.claude\\agents\\` that `backend-architect` and `refactoring-specialist` carry, and the shared one under `references/` | retired with their parents | Each instructs a subagent to invoke one named external binary, which §1.2 forbids |

## Command

The entry point is the skill itself: `skills/implementing-stories/SKILL.md`, installed to `.claude/skills/build/`, whose frontmatter `name` is the slash command. There is no command file. This is the frontmatter and the preamble, verbatim, with four preamble lines.

```
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
```
The `gate require` line leads, so a failing predecessor gate aborts the invocation before any document loads.

## CLI calls

Calls the skill makes. Each is a §4 name, or an addition listed in `## Decisions`.

| Step | Call | Exit handling |
|---|---|---|
| preamble | `devforgeai gate require build $ARGUMENTS[0]` | 1 stops the command body; stderr names the plan gate |
| preamble | `devforgeai doc load story $ARGUMENTS[0]` | 1 stops the command body on `DFA-E200` |
| preamble | `devforgeai doc load context all` | 1 stops the command body on `DFA-E200` |
| preamble | `devforgeai doc load sprint -` | 1 stops the command body on `DFA-E200` |
| 3 | `devforgeai doc load ui-spec <UI-nnn>` | 1 stops the run with a `Blocked` line |
| 3 | `devforgeai config get frontend.tokens_path` | 3 stops the run with a `Blocked` line |
| 4 | `devforgeai worktree ensure <STORY-nnn>` | 1 stops the run with a `Blocked` line naming `DFA-E271` or `DFA-E272` |
| 5 | `devforgeai phase set build --id <STORY-nnn>` | 1 on `DFA-E320` stops the run |
| 6 | `devforgeai config get stack.test_command` | an empty value stops the run with a `Blocked` line |
| 7.3, 7.6, 7.8, 8, 10 | `devforgeai commit <STORY-nnn> -m "<message>"` | 1 stops the run and carries the hook stderr into the `Blocked` line |
| 7.7 | `devforgeai report show <STORY-nnn> build --check build-lint` | 1 on `DFA-E400` reruns the test command once |
| 7.7 | `devforgeai report show <STORY-nnn> build --check build-complexity` | as above |
| 7.8 | `devforgeai config get build.complexity_max` | 3 stops the run with a `Blocked` line |
| 7.8 | `devforgeai config get build.duplication_max_percent` | as above |
| 9, 11 | `devforgeai story files --diff --id <STORY-nnn> --json` | 1 lists the undeclared paths, which the `build-files` check fails on |
| 10 | `devforgeai antipattern scan --id <STORY-nnn>` | 1 opens the remediation of step 10 |
| R1 | `devforgeai doc load qa-report <STORY-nnn>` | 1 stops the run with a `Blocked` line |
| 13 | `devforgeai report note <STORY-nnn> build --key build --file .devforgeai/build/STORY-nnn-note.yaml` | 1 on `DFA-E413` names the failing key and the call repeats |

Calls hooks make during this phase, listed so the skill's steps are not written twice. The skill invokes none of them.

| Hook | Event and matcher | Call |
|---|---|---|
| §7 PreToolUse | `Write\|Edit\|NotebookEdit`, path outside `.devforgeai/`, `[current].phase` of `build` | `devforgeai story files --check <path>`; exit 1 becomes hook exit 2 |
| §7 PreToolUse | `Write\|Edit\|NotebookEdit`, path matching `[frontend].globs` | `devforgeai design lint <path>`; exit 1 becomes hook exit 2 |
| §7 PreToolUse | `Write\|Edit\|NotebookEdit`, path under `.devforgeai/` | `devforgeai doc validate --producer-check <path>` |
| §7 PostToolUse | `Bash\|PowerShell`, command matching `[[stack]].test_command` | `devforgeai gate check --phase build --partial` |
| §7 PostToolUse | `Write\|Edit\|NotebookEdit`, path under `.devforgeai/` | `devforgeai doc validate <path>` |
| §7 SubagentStop | any | `devforgeai report ingest <subagent> -` for `ac-test-writer`, `story-ac-verifier`, `context-validator`; a no-op for the other four |
| §7 Stop | any | `devforgeai gate check --phase build`, then `devforgeai handoff` |
| §7 pre-commit | git, through `devforgeai commit` | `devforgeai doc validate` on staged `.devforgeai/` files, `devforgeai context audit` |
| §7 commit-msg | git, through `devforgeai commit` | the `STORY-nnn` requirement, which `devforgeai commit` satisfies by construction |
| §7 pre-push | git | `devforgeai gate check --phase build` per `sprint.yaml` story with `status: building` |

## Gate

The `.devforgeai/gates.toml` entry for this phase. The first six checks are `specs/01-cli.md` §Gate verbatim. The six blocks after them are added by this spec and proposed as an amendment to the default file in `## Decisions` item 9; `verifier_pass` is an existing kind reused by name, and `complexity_clean`, `antipattern_clean`, and `files_declared` are proposed kinds.

```toml
[[gate]]
phase = "build"
requires = "plan"
on_fail = "fail"
send_back_to = "plan"
description = "Story implemented, tests green, coverage by layer met, lint clean."

  [[gate.check]]
  kind = "doc_valid"
  id = "build-docs"
  docs = ["stories/{id}.md"]
  on_fail = "send_back"

  [[gate.check]]
  kind = "field_in_enum"
  id = "build-status"
  path = "stories/{id}.md"
  field = "status"
  values = ["building", "built"]

  [[gate.check]]
  kind = "tests_pass"
  id = "build-tests"
  stacks = []
  allow_empty = false

  [[gate.check]]
  kind = "coverage_min"
  id = "build-coverage"
  layers = ["domain", "application", "infrastructure", "interface"]
  overall = true
  source = "run"

  [[gate.check]]
  kind = "lint_clean"
  id = "build-lint"
  stacks = []

  [[gate.check]]
  kind = "design_tokens"
  id = "build-design"
  paths = []
  severity = "warn"

  [[gate.check]]
  kind = "complexity_clean"
  id = "build-complexity"
  stacks = []
  severity = "warn"

  [[gate.check]]
  kind = "files_declared"
  id = "build-files"
  message = "{value} is outside the declared file set of {id}"

  [[gate.check]]
  kind = "antipattern_clean"
  id = "build-antipatterns"
  min_severity = "high"
  scope = "story"

  [[gate.check]]
  kind = "verifier_pass"
  id = "build-testable"
  verifiers = ["ac-test-writer"]
  min_ratio = 1.0
  on_fail = "send_back"

  [[gate.check]]
  kind = "verifier_pass"
  id = "build-context"
  verifiers = ["context-validator"]
  min_ratio = 1.0

  [[gate.check]]
  kind = "verifier_pass"
  id = "build-acs"
  verifiers = ["story-ac-verifier"]
  min_ratio = 1.0
  on_fail = "send_back"
```

### What each gate condition of this phase is decided by

| Condition | Check | Decided by |
|---|---|---|
| Tests pass | `build-tests` | every `[[stack]].test_command` exits 0 |
| Coverage per layer meets threshold | `build-coverage` | each `[[layer]].coverage_min` and `[coverage].overall_min` against the parsed coverage file |
| Verifier PASS on every AC | `build-acs` | `verifiers.story_ac.passed / total` at 1.0 |
| Every AC could be turned into a test | `build-testable` | `verifiers.ac_testable.passed / total` at 1.0 |
| Every changed file is in `## Files` | `build-files` | `devforgeai story files --diff` exit 0 |
| Context validation clean | `build-context` | `verifiers.context.passed / total` at 1.0 |
| No `AP-nnn` pattern matched | `build-antipatterns` | `devforgeai antipattern scan --id <STORY-nnn> --min-severity high` exit 0 |
| The story document still parses and carries a build status | `build-docs`, `build-status` | `doc validate` and the frontmatter `status` |
| Lint clean, complexity under the ceiling, tokens resolve | `build-lint`, `build-complexity`, `build-design` | the three commands' exit codes; the last two carry `severity = "warn"` and annotate the result rather than setting it |

`build-lint` carries the gate's default `severity = "block"`, so a lint result that survives the refactor pass of step 7.8 fails the gate. `build-complexity` and `build-design` carry `severity = "warn"`: a complexity ceiling is a target the refactor pass aims at rather than a bound on shipping, and `design_tokens` is `warn` in the default file already.

Three verifiers report into this gate. `devforgeai handoff` rendering rule 6 shows the entry with the lowest `passed/total` and appends `+2 more`, so a PASS run renders whichever of the three has the smallest ratio and a SEND BACK run renders the failing one.

## Send-back

### To Plan, exit 2

| Condition | Detected by | IDs cited | Handoff `Next` |
|---|---|---|---|
| SB-1 · A criterion's `Then` clause names no outcome a test reads, its `When` clause names no action, or its `Given` clause names no reachable state | `ac-test-writer` finding, `severity: block`, `reason` in `then_names_no_readable_outcome`, `when_names_no_action`, `given_names_no_reachable_state` | the `AC-nnn` of each such finding | `/plan <EPIC-nnn> --remedy <AC ids>` |
| SB-2 · Two criteria of the story assert opposed outcomes for one state and action | `ac-test-writer` finding, `severity: block`, `reason` of `contradicts_other_ac` | the `AC-nnn` the subagent was working and every id in its `conflicts_with` list | `/plan <EPIC-nnn> --remedy <AC ids>` |
| SB-3 · A criterion's outcome lives in a file the story's `## Files` table does not declare | `ac-test-writer` finding, `severity: block`, `reason` of `outcome_outside_declared_files` | the `AC-nnn` of each such finding | `/plan <EPIC-nnn> --remedy <AC ids>` |
| SB-4 · A criterion the run implemented is not satisfied by the diff and the test output read in a fresh context | `story-ac-verifier` finding, `severity: block` | the `AC-nnn` of each such finding | `/plan <EPIC-nnn> --remedy <AC ids>` |

`<EPIC-nnn>` is the `epic` key of `.devforgeai/stories/sprint.yaml`, which the preamble printed. The three `AC` findings routes carry the `AC` prefix, which `specs/05-plan.md` §Decisions 8's prefix map does not name, so `gate check` falls back to the gate's static `send_back_to` of `plan`. The return trip is `/build <STORY-nnn> --resume`.

On a send-back the worktree stays on disk with every commit the run made, the story file keeps `status: building`, `state.toml` keeps `[current].phase` of `build` and `[active].build` of the story id, and no byte of `.devforgeai/stories/`, `.devforgeai/context/`, or `.devforgeai/ui-specs/` changes (§1.6).

SB-1, SB-2, and SB-3 stop the run at step 7.1 with the criteria after the failing one unworked, so `cycles` in the note is shorter than `## Acceptance Criteria` and the `--resume` run re-enters at the first absent entry. SB-4 arrives after step 11, so every criterion has a cycle and the `--resume` run re-enters at the first entry whose `green_commit` the model cleared when rewriting.

### Received

| From | Command | Effect |
|---|---|---|
| Verify | `/build <STORY-nnn> --remedy FIND-nnn,...` | Remedy step R1 resolves each `FIND-nnn` to its `AC-nnn` in `reports/STORY-nnn-qa.yaml`; the criterion set of step 7 is those ids alone; every other criterion keeps its test, its code, and its commits; `Next` is `/verify <STORY-nnn>` on PASS and the send-back form above when a cited criterion turns out to be untestable |
| Plan | `/build <STORY-nnn> --resume` | Plan's remedy run rewrote one or more criterion lines in the story; the resume workflow re-reads the story and works the criteria from the first absent or ungreen `cycles` entry onward |

## Integration

| Skill | Consumes (doc, IDs) | Produces for (doc, IDs) | Sends back to (condition) | Receives send-back from (condition) | Shared subagents | `state.toml` fields read / written |
|---|---|---|---|---|---|---|
| 0 Explore · `exploring-ideas` | `.devforgeai/explore/seed-data.json` `entities[].rows`, read at step 7.1 when the file exists. `explore/brief.md` and `explore/decision.yaml` reach Build only as text Discover copied into `requirements.yaml` and Plan copied into a story; Build reads no `IDEA-nnn` or `FLOW-nnn` body | none — Explore runs once per idea, before any story exists, and reads no code | none — §5 gives Build one send-back target, Plan | none — §5 gives Explore's send-back column as `none` | none | none |
| 1 Discover · `discovering-requirements` | none — a `REQ-nnn` reaches Build as the `## Requirements` table of the story, whose `Statement` cells Plan copied byte for byte; Build opens no `requirements.yaml` | none — Discover runs before Plan and reads no build report | none — a requirement defect surfaces as an untestable criterion, which Plan triages and forwards with `spec-gap-triager` | none — Discover cites no `STORY-nnn` | none | none |
| 2 Constitute · `establishing-context` | the six `context/*.md` files: `source-tree.md` `## Layers`, `## File placement rules`, `## Naming conventions`; `coding-standards.md` `## Formatting`, `## Naming`, `## Error handling`, `## Logging`, `## Testing standards`; `dependencies.md` `## Approved dependencies`, `## Forbidden dependencies`; `architecture-constraints.md` `## Constraints`, `## Constraint index` (`CON-nnn`), `## Layer dependency rules`; `anti-patterns.md` `## Anti-pattern index` (`AP-nnn`); `tech-stack.md` `## Excluded technologies` | `.devforgeai/reports/STORY-nnn-build.yaml` `verifiers.context.findings[]`, whose `id` is a `CON-nnn` or `AP-nnn`, read by Constitute at its remedy step when Plan forwards one | none — §5 gives Build one send-back target; a constraint that decides no case reaches Constitute through Plan's SB-3 | none — Constitute's send-back target is Discover | `context-validator`, which Constitute's `alignment-auditor` does not invoke and which Verify's `anti-pattern-scanner` complements | none |
| 3 Plan · `planning-work` | `.devforgeai/stories/STORY-nnn.md`, all ten sections and the frontmatter `status` and `consumes`; `.devforgeai/stories/sprint.yaml` `epic`, `stories[].id`, `stories[].status`, `stories[].order` | `.devforgeai/reports/STORY-nnn-build.yaml`: the top-level `build` key, `verifiers.ac_testable`, `verifiers.story_ac`, and the merged `findings[]`, which Plan reads at its remedy step R1 through `report show <STORY-nnn> build` | yes: SB-1 a criterion has no testable predicate, SB-2 two criteria contradict, SB-3 a criterion's outcome lies outside the declared file set, SB-4 the fresh-context verifier fails a criterion; leaves as `/plan <EPIC-nnn> --remedy AC-nnn,...` | yes: Plan's remedy run rewrote a criterion line; arrives as `/build <STORY-nnn> --resume` | none — Plan's `story-invest-auditor` judges specifications and Build's `story-ac-verifier` judges implementations | reads `[active].plan` for the `SPRINT-nnn` the plan gate report is named after; `phase set build` writes `[current].phase`, `[current].id`, `[active].build` |
| 4 Build · `implementing-stories` | self | self | self | self | self | reads `[current].phase`, `[current].id`, `[active].build`, `[active].release`; `phase set build` writes `[current]` and `[active].build`; `gate check --phase build` writes `[last_gate]`; `handoff` writes `[last_handoff]` |
| 5 Verify · `validating-quality` | none on a `full` run; on a remedy run `.devforgeai/reports/STORY-nnn-qa.yaml` `findings[]` (`FIND-nnn`, and the `AC-nnn` each names), through `doc load qa-report <STORY-nnn>` | `.devforgeai/reports/STORY-nnn-build.yaml`: `gate.result`, which `gate require verify <STORY-nnn>` tests; `coverage`; `verifiers.story_ac.payload.checks[]`, the per-criterion verdicts Verify re-decides; the top-level `build` key, whose `cycles[].test_paths` and `refactors[].paths` name what changed. The commits on branch `story/STORY-nnn` are the diff Verify reads | none — a defect Verify would find is Build's own to fix before the gate passes | yes: a `FIND-nnn` names a gap between a criterion and the code; arrives as `/build <STORY-nnn> --remedy FIND-nnn,...` and re-opens the cited findings' criteria alone | `ac-compliance-verifier` is Verify's and `story-ac-verifier` is Build's; they are two `[[verifier]]` entries because one name binds to one phase | writes `[active].build`, which `phase set verify` reads as its subject |
| 6 Release · `releasing-software` | none — Build reads no release manifest | `.devforgeai/stories/STORY-nnn.md` frontmatter `status`, moved to `building` by `phase set build` and to `released` by `phase set release`; the commits on `story/STORY-nnn`, which the release branch merges | none — a release defect is downstream of everything Build owns | none — §5 gives Release's send-back target as Verify | none | reads `[active].release`, which is `""` before the first release and which decides whether the handoff `Then` line renders |
| Design · `designing-interfaces` | `.devforgeai/ui-specs/UI-nnn.md` `## Anatomy`, `## States`, `## Breakpoints`, `## Interaction`, `## Accessibility`, `## Tokens used`, through `doc load ui-spec <UI-nnn>`; `.devforgeai/brand/tokens.json` `color`, `type`, `spacing`, `radius`, `elevation`, `motion` leaf names | none — Design reads no build report; the frontend files Build writes are checked against `tokens.json` by `design lint`, which is a CLI call rather than a document | no send-back; a screen the story needs and `ui-specs/` does not hold stops step 3 with a `Blocked` line naming `/design <UI-nnn> --spec`, which the user runs and then resumes | none — Design holds no gate (`specs/08-design.md` §Gate) and emits no send-back to Build | none — Design owns `mockup-designer`, `brand-designer`, and `requirement-coverage-auditor`, and Build invokes none of them | reads `[current].phase`; Design writes none |
| Reflect · `improving-framework` | none — Reflect returns recommendations, which §5 excludes from gates | `.devforgeai/reports/STORY-nnn-build.yaml`, whose `gate.checks`, `coverage`, `verifiers`, and top-level `build` key are one input to `OBS-nnn` and `REC-nnn` extraction; the `cycles[]` red-to-green record is where a repeated implementer retry shows up | none — Build cites criteria, not framework observations | none — Reflect emits recommendations, not send-backs | none | none |
| CLI · `devforgeai` | `.devforgeai/config.toml` `[[stack]].test_command`, `.lint_command`, `.complexity_command`, `.source_roots`, `[[layer]]`, `[coverage]`, `[build]`, `[frontend]`, `[[verifier]]`, `degraded`; `.devforgeai/gates.toml` `[[gate]]` with `phase = "build"`; `.devforgeai/state.toml` `[current]`, `[active].build`, `[active].plan`, `[active].release` | `.devforgeai/reports/STORY-nnn-build.yaml` through `gate check`, `report ingest`, and `report note`; `.devforgeai/stories/STORY-nnn.md` frontmatter `status` through `phase set build`; the worktree and its commits through `worktree ensure` and `commit`; `state.toml` through `phase set` and `handoff` | not applicable | not applicable | not applicable | reads `[current].phase`, `[active].build`, `[active].plan`, `[active].release`; `phase set build` writes `[current]` and `[active].build`; `gate check` writes `[last_gate]`; `handoff` writes `[last_handoff]`; `hook run stop` writes `[stop_hook]` |

## Handoff

Printed by `devforgeai handoff`. The phase document is `reports/STORY-nnn-build.yaml`, a YAML file with no H1, so rendering rule 3 of `specs/01-cli.md` §Handoff gives the slug `-`.

PASS, a seven-criterion application-layer story, with no release cut yet so `[active].release` is empty and the optional `Then` line is dropped:

```
Phase     4 · Build           STORY-014 · -
Done      214 tests · 87.4% coverage
Gate      PASS  12 checks
Verified  ac-test-writer · 1/1 AC  +2 more

Next      /verify STORY-014
Blocked   none

Full report: .devforgeai/reports/STORY-014-build.yaml
```

Nine lines. `Verified` renders one of three entries by rendering rule 6, which shows the entry with the lowest `passed/total` and appends `+<k> more`; the three are `ac_testable` at 1/1, `story_ac` at 7/7, and `context` at 6/6, all at a ratio of 1.0, so the tie-break of `## Decisions` item 40 takes the first in `config.toml` `[[verifier]]` order.

SEND BACK to Plan, a criterion that names no readable outcome and a second that contradicts it:

```
Phase     4 · Build           STORY-014 · -
Done      118 tests · 71.2% coverage
Gate      SEND BACK to Plan  build-testable, build-acs
Verified  ac-test-writer · 0/1 AC
Found     AC-007 Then clause names no outcome a test reads
Found     AC-003 asserts a different total for the same cart

Next      /plan EPIC-002 --remedy AC-007,AC-003
Then      /build STORY-014 --resume
Blocked   none

Full report: .devforgeai/reports/STORY-014-build.yaml
```

Twelve lines. The fixed lines are `Phase`, `Done`, `Gate`, `Verified`, one blank, `Next`, `Then`, `Blocked`, one blank, and `Full report:`, which is ten with both optional lines present, so the Found budget is two and both findings render in full, filling the block to the cap. `Verified` carries no `+<k> more` suffix here: the run stopped at workflow step 7.1, so `ac-test-writer` is the one subagent that reached `SubagentStop` and the report holds one verifier block. The `Found` labels and the `--remedy` list are the `id` values of the blocking `findings[]` entries in the order `report ingest` stored them, and `EPIC-002` is the `epic` key of `sprint.yaml`.

## Templates

### templates/build-note.yaml

    schema: devforgeai/build-note/1
    id: STORY-nnn
    run: full
    worktree: ../wt/STORY-nnn
    branch: story/STORY-nnn
    base_commit: <40 lowercase hex>
    head_commit: <40 lowercase hex>
    layer: <a [[layer]].name: domain | application | infrastructure | interface>
    cycles:
      - ac: AC-nnn
        test_paths:
          - <a ## Files Path of Kind test>
        red_commit: <40 lowercase hex>
        implementer: <backend-implementer | frontend-implementer>
        green_commit: <40 lowercase hex, or "" while the criterion is unfinished>
        source_paths:
          - <a ## Files Path of Kind source, config, migration or asset>
    refactors:
      - trigger: <build-lint | build-complexity>
        reason: <the report check entry's reason field, copied>
        paths:
          - <a ## Files Path>
        commit: <40 lowercase hex>
    integration:
      ran: false
      reason: not-applicable
      test_paths: []
      commit: ""
    remedy: []
    resumed_at: ""

`refactors` with no pass carries `[]`. `cycles` carries one entry per criterion the run worked; an unfinished entry carries `green_commit` of `""` and `source_paths` of `[]`. `integration.ran` is `true` with `reason` of `layer` or `interface`, and `false` with `reason` of `not-applicable`.

### templates/build-config.toml

The fragment `devforgeai init` merges into `.devforgeai/config.toml` for this phase.

    [build]
    worktree_root = "../wt"
    branch_prefix = "story/"
    base_ref = "HEAD"
    complexity_max = 10
    duplication_max_percent = 5.0

    [[verifier]]
    name = "ac-test-writer"
    phase = "build"
    report_field = "verifiers.ac_testable"
    unit = "AC"
    required = true

    [[verifier]]
    name = "story-ac-verifier"
    phase = "build"
    report_field = "verifiers.story_ac"
    unit = "ACs"
    required = true

    [[verifier]]
    name = "context-validator"
    phase = "build"
    report_field = "verifiers.context"
    unit = "files"
    required = true

`stack detect` leaves `[build]` and `[[verifier]]` at their previous values when `config.toml` exists and writes these defaults when it does not, the rule that already governs `[[layer]]` and `[coverage]`. Each `[[stack]]` table gains `complexity_command`, default `""` per ecosystem, whose exit code is the `complexity_clean` signal and whose `""` value makes the check `skip`.

## Evals

Three artifacts under `skills/implementing-stories/evals/`, per §9. Every case ships `.devforgeai/config.toml`, `.devforgeai/gates.toml`, `.devforgeai/state.toml`, `.devforgeai/stories/STORY-014.md`, `.devforgeai/stories/sprint.yaml`, and the six context files in `setup.files`, and the graders read the workspace and the transcript alone.

### evals/evals.json

```json
{
  "skill": "implementing-stories",
  "evals": [
    {
      "prompt": "/build STORY-014",
      "expected_output": "One worktree, one failing test per AC before its implementation, one commit per red and per green, a build note listing every cycle, and a handoff whose Next line is /verify STORY-014.",
      "expectations": [
        "Calls devforgeai worktree ensure STORY-014 before writing any file.",
        "Calls devforgeai phase set build --id STORY-014 before the first Write.",
        "Invokes ac-test-writer once per AC-nnn in ## Acceptance Criteria order.",
        "Runs no git command directly; every commit is devforgeai commit STORY-014 -m <message>.",
        "Writes .devforgeai/build/STORY-014-note.yaml with one cycles entry per AC.",
        "Writes no file whose path is absent from the story's ## Files table."
      ]
    },
    {
      "prompt": "/build STORY-014",
      "expected_output": "The run stops at the first criterion whose Then clause names no readable outcome and prints a SEND BACK handoff to Plan citing that AC id.",
      "expectations": [
        "ac-test-writer returns testable false with reason then_names_no_readable_outcome.",
        "No implementer subagent is invoked for that criterion.",
        "The handoff Next line is /plan EPIC-002 --remedy AC-007.",
        "The handoff Then line is /build STORY-014 --resume.",
        "The story file .devforgeai/stories/STORY-014.md is unchanged by the run."
      ]
    },
    {
      "prompt": "/build STORY-014",
      "expected_output": "The run stops when two criteria of the story assert opposed outcomes and cites both ids.",
      "expectations": [
        "ac-test-writer returns conflicts_with holding the other AC id.",
        "The remedy list on the Next line holds both AC ids, comma separated, with no space.",
        "The handoff block is twelve lines or fewer.",
        "No source file is written after the conflict is reported."
      ]
    },
    {
      "prompt": "/build STORY-014 --remedy FIND-002,FIND-005",
      "expected_output": "Only the criteria the two findings name are re-opened; the other criteria keep their tests and their code.",
      "expectations": [
        "Calls devforgeai doc load qa-report STORY-014 to resolve the finding ids.",
        "Invokes ac-test-writer for AC-002 and AC-006 alone.",
        "The build note's remedy key lists FIND-002 and FIND-005.",
        "The build note's cycles list holds two entries."
      ]
    },
    {
      "prompt": "/build STORY-014 --resume",
      "expected_output": "The run re-enters at the first criterion with no green commit and keeps every earlier cycle entry.",
      "expectations": [
        "Reuses the existing worktree rather than creating a second one.",
        "The build note's resumed_at names the first ungreen criterion.",
        "The cycles entries before that criterion keep their original commit shas.",
        "No implementer subagent is invoked for a criterion already carrying a green commit."
      ]
    },
    {
      "prompt": "/build STORY-015",
      "expected_output": "The run refuses to open a second worktree whose story declares a file another building story declares, and prints one Blocked line naming both stories.",
      "expectations": [
        "devforgeai worktree ensure STORY-015 exits 1 with DFA-E272.",
        "The Blocked line names STORY-014, STORY-015, and the shared path.",
        "No file under the project is written.",
        "No devforgeai phase set build call is made."
      ]
    }
  ]
}
```

### evals/cases.jsonl

```jsonl
{"id": "BLD-01", "prompt": "/build STORY-014", "timeout": 1500, "preflight": ["worktree ensure STORY-014", "phase set build --id STORY-014", {"command": "gate check --phase build --partial --id STORY-014 --quiet", "exit": "any", "forbid_code": "DFA-E311"}], "setup": {"files": {".devforgeai/stories/STORY-014.md": "FIXTURE:story-014.md", ".devforgeai/stories/sprint.yaml": "FIXTURE:sprint-002.yaml", ".devforgeai/config.toml": "FIXTURE:config-build.toml", ".devforgeai/gates.toml": "FIXTURE:gates-build.toml", ".devforgeai/state.toml": "FIXTURE:state-plan.toml", ".devforgeai/context/tech-stack.md": "FIXTURE:tech-stack.md", ".devforgeai/context/source-tree.md": "FIXTURE:source-tree.md", ".devforgeai/context/dependencies.md": "FIXTURE:dependencies.md", ".devforgeai/context/coding-standards.md": "FIXTURE:coding-standards.md", ".devforgeai/context/architecture-constraints.md": "FIXTURE:architecture-constraints.md", ".devforgeai/context/anti-patterns.md": "FIXTURE:anti-patterns.md", ".devforgeai/reports/SPRINT-001-plan.yaml": "FIXTURE:report-SPRINT-001-plan.yaml", "ci/test": "FIXTURE:ci-test.sh", "ci/coverage": "FIXTURE:ci-coverage.sh", "ci/lint": "FIXTURE:ci-lint.sh", "ci/complexity": "FIXTURE:ci-complexity.sh"}, "git": true}, "expect": {"grader": "build_note_shape", "args": {"id": "STORY-014", "ac_ids": ["AC-001", "AC-002", "AC-003"], "files": ["src/application/checkout/place_order.ext", "tests/application/checkout/place_order_spec.ext", "src/application/checkout/order_total.ext"], "run": "full", "min_cycles": 3}}}
{"id": "BLD-02", "prompt": "/build STORY-014", "timeout": 1500, "preflight": ["worktree ensure STORY-014", "phase set build --id STORY-014", {"command": "gate check --phase build --partial --id STORY-014 --quiet", "exit": "any", "forbid_code": "DFA-E311"}], "setup": {"extends": "BLD-01", "files": {}, "git": true}, "expect": {"grader": "writes_inside_declared_set", "args": {"id": "STORY-014", "files": ["src/application/checkout/place_order.ext", "tests/application/checkout/place_order_spec.ext", "src/application/checkout/order_total.ext"], "baseline": [".devforgeai", "README.md", ".claude", ".git", "wt"], "worktree_root": "wt"}}}
{"id": "BLD-03", "prompt": "/build STORY-014", "timeout": 1500, "preflight": ["worktree ensure STORY-014", "phase set build --id STORY-014", {"command": "gate check --phase build --partial --id STORY-014 --quiet", "exit": "any", "forbid_code": "DFA-E311"}], "setup": {"extends": "BLD-01", "files": {".devforgeai/stories/STORY-014.md": "FIXTURE:story-014-untestable.md"}, "git": true}, "expect": {"grader": "remedy_line", "args": {"epic": "EPIC-002", "story": "STORY-014", "remedy": ["AC-007"], "gate": "SEND BACK to Plan", "forbid_next": ["/verify STORY-014"], "forbid_paths": [".devforgeai/build/STORY-014-note.yaml"]}}}
{"id": "BLD-04", "prompt": "/build STORY-014", "timeout": 1500, "preflight": ["worktree ensure STORY-014", "phase set build --id STORY-014", {"command": "gate check --phase build --partial --id STORY-014 --quiet", "exit": "any", "forbid_code": "DFA-E311"}], "setup": {"extends": "BLD-01", "files": {".devforgeai/stories/STORY-014.md": "FIXTURE:story-014-contradiction.md"}, "git": true}, "expect": {"grader": "remedy_line", "args": {"epic": "EPIC-002", "story": "STORY-014", "remedy": ["AC-007", "AC-003"], "gate": "SEND BACK to Plan", "forbid_next": ["/verify STORY-014"], "forbid_paths": [".devforgeai/build/STORY-014-note.yaml"]}}}
{"id": "BLD-05", "prompt": "/build STORY-014 --remedy FIND-002,FIND-005", "timeout": 1500, "preflight": ["worktree ensure STORY-014", "phase set build --id STORY-014", {"command": "gate check --phase build --partial --id STORY-014 --quiet", "exit": "any", "forbid_code": "DFA-E311"}], "setup": {"extends": "BLD-01", "files": {".devforgeai/stories/STORY-014.md": "FIXTURE:story-014-six-acs.md", ".devforgeai/reports/STORY-014-qa.yaml": "FIXTURE:story-014-qa.yaml", ".devforgeai/build/STORY-014-note.yaml": "FIXTURE:note-014-complete.yaml"}, "git": true}, "expect": {"grader": "criterion_set", "args": {"id": "STORY-014", "run": "remedy", "expect_acs": ["AC-002", "AC-006"], "forbid_acs": ["AC-001", "AC-003", "AC-004", "AC-005"], "remedy": ["FIND-002", "FIND-005"]}}}
{"id": "BLD-06", "prompt": "/build STORY-014 --resume", "timeout": 1500, "preflight": ["worktree ensure STORY-014", "phase set build --id STORY-014", {"command": "gate check --phase build --partial --id STORY-014 --quiet", "exit": "any", "forbid_code": "DFA-E311"}], "setup": {"extends": "BLD-01", "files": {".devforgeai/build/STORY-014-note.yaml": "FIXTURE:note-014-partial.yaml"}, "git": true}, "expect": {"grader": "criterion_set", "args": {"id": "STORY-014", "run": "resume", "expect_acs": ["AC-003"], "forbid_acs": ["AC-001", "AC-002"], "resumed_at": "AC-003", "prior_cycles": [{"ac": "AC-001", "green_commit": "1111111111111111111111111111111111111111"}, {"ac": "AC-002", "green_commit": "2222222222222222222222222222222222222222"}]}}}
{"id": "BLD-07", "prompt": "/build STORY-015", "timeout": 1500, "preflight": [{"command": "worktree ensure STORY-015", "exit": 1, "code": "DFA-E272"}], "setup": {"extends": "BLD-01", "files": {".devforgeai/stories/STORY-015.md": "FIXTURE:story-015-overlap.md", ".devforgeai/stories/sprint.yaml": "FIXTURE:sprint-002-building.yaml", ".devforgeai/state.toml": "FIXTURE:state-build-story-014-worktree.toml"}, "git": {"worktrees": [{"path": "wt/STORY-014", "branch": "story/STORY-014"}]}}, "expect": {"grader": "blocked_line", "args": {"code": "DFA-E272", "names": ["STORY-014", "STORY-015", "src/application/checkout/place_order.ext"], "forbid_calls": ["devforgeai phase set build"], "expect_calls": ["devforgeai worktree ensure STORY-015"], "forbid_paths": [".devforgeai/build/STORY-015-note.yaml", "wt/STORY-015"], "unchanged_status": {"path": ".devforgeai/stories/STORY-015.md", "status": "ready"}}}}
{"id": "BLD-08", "prompt": "/build STORY-014", "timeout": 1500, "preflight": ["worktree ensure STORY-014", "phase set build --id STORY-014", {"command": "gate check --phase build --partial --id STORY-014 --quiet", "exit": "any", "forbid_code": "DFA-E311"}], "setup": {"extends": "BLD-01", "files": {".devforgeai/config.toml": "FIXTURE:config-degraded.toml"}, "git": true}, "expect": {"grader": "blocked_line", "args": {"code": "", "names": ["devforgeai stack detect"], "forbid_calls": ["Agent(ac-test-writer"], "expect_calls": ["devforgeai config get stack.test_command"], "forbid_paths": [".devforgeai/build/STORY-014-note.yaml"], "unchanged_status": {"path": ".devforgeai/stories/STORY-014.md", "status": "ready"}}}}
```
Eight cases. BLD-03 and BLD-04 exercise the SEND BACK path, which satisfies the §9 minimum of two. Every case after BLD-01 carries `setup.extends`, proposed in `## Decisions` item 42: the named case's `setup.files` map is written into the workspace first and this case's map is written over it, so a path key appearing in both takes this case's content and a path appearing in neither is absent. Each `expect.args` carries the prior state its grader compares against — the criterion ids, the declared `## Files` set, the remedy list, the resume entry point, and the earlier cycles' commit shas — so no grader re-derives them from the workspace.

### evals/graders.py

Five pure functions, signature `def <grader>(workspace: str, transcript: str, args: dict) -> tuple[bool, str]`. No network, no subprocess, no randomness. Each reads files under `workspace` and searches `transcript` for literal substrings.

```python
def build_note_shape(workspace, transcript, args): ...
```
Parses `<workspace>/.devforgeai/build/<args["id"]>-note.yaml` as YAML. Passes when `schema` is `devforgeai/build-note/1`, `id` equals `args["id"]`, `run` equals `args["run"]`, `len(cycles) >= args["min_cycles"]`, `[c["ac"] for c in cycles]` equals `args["ac_ids"]`, every `test_paths` and `source_paths` entry is a member of `args["files"]`, and every cycle's `red_commit` differs from its `green_commit` and both are forty hex characters. Evidence: the first cycle entry that fails, or the count of cycles checked.

```python
def writes_inside_declared_set(workspace, transcript, args): ...
```
Walks `<workspace>` and `<workspace>/<args["worktree_root"]>`, collecting every file path whose first path segment is absent from `args["baseline"]`. Passes when every collected path, made repo-relative, is a member of `args["files"]`. Evidence: the sorted list of paths outside the set, or the count of paths checked.

```python
def remedy_line(workspace, transcript, args): ...
```
Passes when `transcript` contains the literal `"/plan " + args["epic"] + " --remedy " + ",".join(args["remedy"])`, contains `"Gate      " + args["gate"]`, contains `"/build " + args["story"] + " --resume"`, contains none of `args["forbid_next"]`, and the block from the line starting `Phase     ` to the line starting `Full report: ` is twelve lines or fewer. Evidence: the extracted block, or the first absent literal.

```python
def criterion_set(workspace, transcript, args): ...
```
Parses the note as `build_note_shape` does. Passes when `run` equals `args["run"]`, `[c["ac"] for c in cycles]` equals `args["expect_acs"]` for a `remedy` run and equals `args["expect_acs"]` appended to the `ac` values of `args["prior_cycles"]` for a `resume` run, no id in `args["forbid_acs"]` appears in the transcript inside an `Agent(ac-test-writer` invocation, `remedy` equals `args.get("remedy", [])`, `resumed_at` equals `args.get("resumed_at", "")`, and every entry of `args["prior_cycles"]` matches the note entry with the same `ac` on `green_commit`. Evidence: the two id lists side by side.

```python
def blocked_line(workspace, transcript, args): ...
```
Passes when `transcript` contains a line starting `Blocked   you: ` that holds every string in `args["names"]` and, when `args["code"]` is non-empty, that code; contains every literal in `args["expect_calls"]`; and contains no literal in `args["forbid_calls"]`. Evidence: the matched `Blocked` line, or the first absent or forbidden literal.

The shared runner `evals/runner/run_jsonl.py` writes an empty `.claude/settings.json` into each workspace (`specs/01-cli.md` §Decisions 42), so no hook fires during a case. The graders therefore read the skill's own artifacts — the note file, the files written inside the worktree, the CLI calls visible in the transcript, and the printed handoff block — and read no `verifiers.*` block and no `partial: true` report, because `SubagentStop` and the Bash `PostToolUse` entry do not run.

## Decisions

1. `produced_by` for this phase's §5 document is `devforgeai-cli`, because `specs/01-cli.md` §`doc validate` doc-type table gives the `build-report` row that producer and the CLI writes the file. The skill's own name is `implementing-stories`, the §4b name for `/build`, which appears in no frontmatter because the skill produces no document with frontmatter of its own.

2. The skill's contribution to `reports/STORY-nnn-build.yaml` reaches the file through a CLI call and not through Write. The report's producer is `devforgeai-cli`, so `doc validate --producer-check` on `PreToolUse` refuses a build-phase Write to it with `DFA-E212`, which the dispatcher turns into hook exit 2. The skill writes `.devforgeai/build/STORY-nnn-note.yaml`, a path matching no doc-type row and therefore skipped by `doc validate`, and hands it to `report note`.

3. Proposed addition to conventions §4 and to `specs/01-cli.md` §CLI calls: `report note`. Grammar:

   ```
   devforgeai report note <id> <phase> --key <key> --file <path> [--json] [--project <path>]
   ```

   `<id>` and `<phase>` name the report, resolved as `report show` resolves them. `--key` is drawn from a closed per-phase set; the `build` phase accepts the single value `build`, and every other phase accepts none, so a `--key` outside the set is `DFA-E013`, exit 3. `--file` is a path to a YAML file holding one mapping. The CLI parses it, validates it against the `devforgeai/build-note/1` schema of `## Outputs`, drops its `schema` and `id` keys, and writes the remainder at the report's top-level `<key>`, creating the report from the `## Outputs` skeleton when it is absent. A schema failure is `DFA-E413`, exit 1, naming the first offending key, and nothing is written. An absent `--file` path is `DFA-E400`, exit 1. `--json` `data`: `{"id":"STORY-014","report":".devforgeai/reports/STORY-014-build.yaml","key":"build","keys_written":9}`. Human output: `Noted     build -> .devforgeai/reports/STORY-014-build.yaml`. Exit codes: 0 written; 1 on `DFA-E400`, `DFA-E401`, `DFA-E413`; 3 on `DFA-E012`, `DFA-E013`; 5 on `DFA-E9xx`.

4. Proposed addition to conventions §4 and to `specs/01-cli.md` §CLI calls: `worktree`. Grammar:

   ```
   devforgeai worktree ensure <STORY-nnn> [--json] [--project <path>]
   devforgeai worktree list [--json] [--project <path>]
   devforgeai worktree remove <STORY-nnn> [--force] [--json] [--project <path>]
   ```

   The worktree path is `<[build].worktree_root>/<STORY-nnn>`, resolved against the project root, default `../wt`. The branch is `<[build].branch_prefix><STORY-nnn>`, default prefix `story/`.

   `ensure` is idempotent: a path that is already a worktree of this repository on the matching branch is printed and the command exits 0. Otherwise the command reads `git worktree list`, takes the `STORY-nnn` encoded in each worktree directory name under `worktree_root`, runs the `story files --list` set for each, and exits 1 with `DFA-E272` naming both story ids and the first shared `Path` when the requested story's set intersects one of them. With no intersection it creates the worktree from `[build].base_ref` on a new branch, appends one `[[worktree]]` entry to the main checkout's `.devforgeai/state.toml` carrying `story`, `path`, `branch` and `created_at`, and prints the path on stdout. A project root that is not a git work tree is `DFA-E271`, exit 1. `--force` is not accepted.

   `list` prints one line per worktree under `worktree_root`: `<STORY-nnn>  <path>  <branch>  <clean|dirty>  <n> ahead`.

   `remove` exits 1 with `DFA-E273` when the worktree holds uncommitted changes or commits absent from `[build].base_ref`, unless `--force` is passed. On success it removes the worktree and deletes the branch when the branch is merged into `base_ref`.

   `--json` `data` for `ensure`: `{"id":"STORY-014","path":"../wt/STORY-014","branch":"story/STORY-014","created":true,"registered":true}`. For `list`: `{"worktrees":[{"id":"STORY-014","path":"../wt/STORY-014","branch":"story/STORY-014","dirty":false,"ahead":3}]}`. For `remove`: `{"id":"STORY-014","removed":true,"branch_deleted":true}`. Exit codes: 0 success; 1 on `DFA-E271`, `DFA-E272`, `DFA-E273`; 3 on `DFA-E012`; 5 on `DFA-E9xx`.

5. Proposed addition to conventions §4 and to `specs/01-cli.md` §CLI calls: `commit`. Grammar:

   ```
   devforgeai commit <STORY-nnn> -m <message> [--paths <path>,...] [--json] [--project <path>]
   ```

   `--paths` defaults to every path git reports as changed in the current work tree. Each path is tested by the `story files --check` rule first; a path outside the story's `## Files` set is `DFA-E239`, exit 1, and nothing is staged. The committed message is `<STORY-nnn>: <message>` when `<message>` does not already hold the story id and `<message>` verbatim when it does, which satisfies the §7 `commit-msg` hook by construction rather than by instructing the model. The command stages the paths and commits, so the `pre-commit` and `commit-msg` hooks run unchanged; a non-zero hook exit becomes exit 1 with the hook's stderr on stderr and nothing committed. An empty stage set is `DFA-W243`, exit 0, nothing committed. `--json` `data`: `{"id":"STORY-014","commit":"a0d4f19…","staged":2,"message":"STORY-014: AC-003 green","hooks":["pre-commit","commit-msg"]}`. Human output: `Commit    a0d4f19  STORY-014: AC-003 green  2 files`. Exit codes: 0 committed; 1 on `DFA-E239`, `DFA-E271`, a hook failure; 3 on `DFA-E011`, `DFA-E012`; 5 on `DFA-E9xx`.

6. Proposed addition to conventions §4 and to `specs/01-cli.md` §CLI calls: `config get`. Grammar:

   ```
   devforgeai config get <key> [--stack <id>] [--json] [--project <path>]
   ```

   `<key>` is drawn from the closed list `stack.test_command`, `stack.coverage_command`, `stack.lint_command`, `stack.complexity_command`, `stack.source_roots`, `stack.package_manager`, `build.worktree_root`, `build.branch_prefix`, `build.base_ref`, `build.complexity_max`, `build.duplication_max_percent`, `coverage.overall_min`, `layer.<name>.coverage_min`, `frontend.tokens_path`, `frontend.globs`, `degraded`. Any other value is `DFA-E013`, exit 3. A `stack.*` key with no `--stack` takes the first `[[stack]]` table; `--stack <id>` selects by `id` and an unmatched id is `DFA-E013`. The value goes to stdout with a trailing newline; an array prints one element per line; an empty string prints an empty line and exits 0. `--json` `data`: `{"key":"stack.test_command","stack":"<a [[stack]] id value>","value":"<the command>","kind":"string"}`. Exit codes: 0 printed; 1 on `DFA-E10x`; 3 on `DFA-E011`, `DFA-E013`; 5 on `DFA-E9xx`. Without this subcommand the skill has no way to reach a test command, and §1.2 forbids naming one.

7. Proposed addition to conventions §4 and to `specs/01-cli.md` §CLI calls: `antipattern scan`. Grammar:

   ```
   devforgeai antipattern scan [--id <STORY-nnn>] [--paths <path>,...] [--min-severity <blocker|high|medium|low>] [--json] [--project <path>]
   ```

   The candidate set is `--paths` when given, the story's `## Files` `Path` values when `--id` is given, and every file under `[[stack]].source_roots` otherwise. For each row of `.devforgeai/context/anti-patterns.md` `## Anti-pattern index` whose `Severity` is at or above `--min-severity`, default `high`, the command intersects the candidate set with the row's `Scope` glob and applies the row's `Detector` under its `Detector kind`: `literal` is a substring match over the file bytes, `regex` is a match of the `regex` crate dialect, `glob` matches the path and reads no file. One match is `DFA-E270`, exit 1, one stderr line per match carrying `AP-nnn`, path, line, and the matched text. No match is exit 0. `--json` `data`: `{"scanned":21,"rules":4,"matches":[{"id":"AP-002","severity":"high","path":"src/…","line":44,"text":"…"}]}`. Exit codes: 0 clean; 1 on `DFA-E270`, `DFA-E200`; 3 on `DFA-E013`; 5 on `DFA-E9xx`. `context audit` check CA-6 tests that a detector exists and nothing tests that it fails to match; this closes that.

8. Proposed extension of the `story files` subcommand `specs/05-plan.md` §Decisions 12 proposes, adding a third mode:

   ```
   devforgeai story files --diff [--id <STORY-nnn>] [--base <ref>] [--json] [--project <path>]
   ```

   `--base` defaults to the merge-base of `[build].base_ref` and the current work tree's branch head, which is the commit the story branch left and which equals `[build].base_ref` itself in a work tree with no branch of its own. The path set is the union of the paths changed between that base and `HEAD` and the uncommitted changes of the work tree. Each path is tested by the `--check` rule; a path outside the story's `## Files` set is `DFA-E239`, exit 1, one stderr line per path. `--json` `data`: `{"id":"STORY-014","base":"HEAD","paths":[{"path":"src/…","declared":true,"kind":"source","status":"modified"}],"undeclared":0}`. This is the surface the `files_declared` gate check wraps and the input `context-validator` and `story-ac-verifier` receive.

9. Proposed additions to the default `.devforgeai/gates.toml` in `specs/01-cli.md` §Gate: six `[[gate.check]]` blocks appended to the `build` gate, given verbatim in `## Gate`. Three use existing kinds (`verifier_pass`, three times) and three use the kinds of item 10. Without `build-testable` and `build-acs` an untestable or unimplemented criterion leaves `gate.result` at PASS and Build has no send-back path; without `build-files` the declared file set is enforced at the write and not at the gate, so a file no hook saw goes undetected; without `build-context` and `build-antipatterns` the constraint and anti-pattern sets Constitute wrote govern nothing at build time.

10. Proposed additions to the check-kind enum of `specs/01-cli.md` §Outputs, which closes at twenty-one and would close at twenty-four. Each wraps a command's exit code rather than parsing its output, which is the shape `context_audit` and `design_tokens` already use.

    | `kind` | Keys, types, defaults | Passes when |
    |---|---|---|
    | `complexity_clean` | `stacks` array[string], default `[]` meaning every stack | every named stack's `[[stack]].complexity_command` exits 0; a `""` command is `skip` with `reason: no_complexity_command` |
    | `antipattern_clean` | `min_severity` string, enum `blocker` \| `high` \| `medium` \| `low`, default `high`; `scope` string, enum `story` \| `project`, default `story` | `antipattern scan` exits 0 over the story's `## Files` set with `scope = "story"`, and over `[[stack]].source_roots` with `scope = "project"` |
    | `files_declared` | `base` string, default `""` | `story files --diff` exits 0 for the gate subject. With `base` at `""` the check resolves the subject's worktree through `worktree list`, runs the diff inside it, and takes the base as the merge-base of `[build].base_ref` and that worktree's branch head; with no worktree for the subject it runs the diff in the project root against `[build].base_ref`. A non-empty `base` names a ref and replaces the merge-base. The `pre-push` hook evaluates this gate from the main checkout, where the worktree lookup is what keeps the diff from comparing a branch against itself |

11. Proposed amendment to `specs/01-cli.md` §`gate check`, the `--partial` row: the evaluated kind set widens from three to four, `tests_pass`, `coverage_min`, `lint_clean`, `complexity_clean`. The `PostToolUse` Bash hook is the only writer of the partial report, and the refactor trigger of workflow step 7.8 reads `build-lint` and `build-complexity` from it; the second is unreachable without this.

12. The refactor threshold source is `config.toml`. The breach is an exit code: `[[stack]].lint_command` for `build-lint` and `[[stack]].complexity_command` for `build-complexity`, both already command strings whose thresholds live in the project's own tool configuration. The two numbers `[build].complexity_max`, default 10, and `[build].duplication_max_percent`, default 5.0, are the targets `refactor-surgeon` aims below and the one place a project changes them. A complexity kind that compared a parsed number would need a per-ecosystem output parser, which `specs/01-cli.md` carries for coverage alone and which §2 rules out here.

13. Proposed additions to the `.devforgeai/config.toml` schema in `specs/01-cli.md` §Outputs: the `[build]` table of `templates/build-config.toml` with the five keys and defaults given there, and a `complexity_command` key on `[[stack]]`, string, default `""` per ecosystem. `stack detect` leaves `[build]` at its previous value when the file exists and writes the defaults when it does not, the rule that already governs `[[layer]]`, `[coverage]`, and `[[verifier]]`.

14. Proposed addition to `specs/01-cli.md` §Outputs, the `state.toml` section: `.devforgeai/state.toml` is untracked and every other path under `.devforgeai/` is tracked. This is what makes a worktree work, and the mechanism settled the other way from what this decision first proposed. A worktree holds no `state.toml`: discovery inside a registered worktree resolves `.devforgeai/` at the main checkout through `git rev-parse --git-common-dir`, so one file answers for every worktree, and `worktree ensure` appends one `[[worktree]]` entry per open worktree. Copying the file gave the Stop hook and the build two different answers to the same question — the hook fires in the root and the work happens in the worktree — and whichever the user last looked at was wrong. Concurrency is unaffected: the array holds one entry per open worktree, each on its own branch over its own source tree, and the entry whose `story` is `[active].build` says which one the command-running checks run in.

15. Proposed amendment to `specs/01-cli.md` §`gate require`, the `gate require build <STORY-nnn>` row. The row reads the subject from "the `EPIC-nnn` in the story's frontmatter `consumes`", and `specs/05-plan.md` §Outputs constrains a story's `consumes` entries to `^(REQ|CON|AP|UI)-\d{3}$`, so no `EPIC-nnn` is there; the same spec's §Decisions 4 names the Plan report `reports/SPRINT-nnn-plan.yaml` and not `reports/EPIC-nnn-plan.yaml`. Amended row: the subject is the `SPRINT-nnn` of the `stories/sprint.yaml` whose `stories[]` or `deferred[]` lists `<STORY-nnn>`, and the report read is `reports/<SPRINT-nnn>-plan.yaml`; a story listed in no sprint is `DFA-E013`, exit 3.

16. Proposed amendment to `specs/01-cli.md` §`handoff`, the `build` `SEND BACK` row of the Next and Then table. The `Next` cell reads `/plan <EPIC-nnn> --remedy <STORY-nnn>`; `specs/05-plan.md` §Send-back Received fixes the arriving form as `/plan <EPIC-nnn> --remedy AC-nnn,...` and its §Decisions 20 closes Plan's remedy id set at the `AC` prefix. Amended cell: `/plan <EPIC-nnn> --remedy <AC ids>`. The `verify` `SEND BACK` row already reads `/build <STORY-nnn> --remedy <FIND ids>`, which is the form `## Send-back` Received takes, and is left alone.

17. Proposed amendment to `specs/01-cli.md` §`handoff`, the `build` `PASS` row: the `Then` cell `/release <vX.Y.Z>` is omitted when `state.toml` `[active].release` is empty, which it is before the first release of a project. §6 marks `Then` optional, and a `Then` line naming a version that does not exist is a line the user cannot type (§1.7).

18. Build registers three verifiers, not one. `ac-test-writer` decides testability, `story-ac-verifier` decides fulfilment, `context-validator` decides constraint compliance, and each is a separate `verifier_pass` check because a single ratio over all three would let one compensate for another. The handoff `Verified` line renders the entry with the lowest `passed/total` and appends `+2 more`, by `specs/01-cli.md` §Handoff rendering rule 6. This differs from `specs/05-plan.md` §Decisions 28, where one verifier left no tie to break.

19. `ac-test-writer` is a registered verifier whose block describes one criterion at a time. `report ingest` replaces a subagent's previous block and appends its findings de-duplicated by `id`, so after a run the block holds the last invocation's `passed`/`total` and `findings[]` holds every criterion any invocation flagged. The loop stops at the first criterion the subagent judged untestable, so the last block on such a run is that criterion's with `passed: 0, total: 1`, and a run whose loop reached the end carries `passed: 1, total: 1`. `build-testable` with `min_ratio = 1.0` therefore reads true on a complete run and false on one stopped by SB-1, SB-2, or SB-3.

    The one loop stop this check does not see is step 7.2's, a new test that passes before any implementation: `ac-test-writer` returned `testable: true` there and its block reads 1.0. That criterion is caught by `build-acs`, because `story-ac-verifier` at step 11 reads a diff carrying no hunk that implements it and returns `no_diff_hunk_implements_it`. Both paths end in a SEND BACK to Plan citing the `AC-nnn`, through different checks.

20. `story-ac-verifier` is a new name rather than a second registration of `ac-compliance-verifier`. A `[[verifier]]` table binds one `name` to one `phase`, and the `SubagentStop` hook calls `report ingest <subagent> -` with no `--phase`, so one name writes into one phase's report. `ac-compliance-verifier` is registered for `verify` in `specs/01-cli.md` §Gate's closing paragraph and stays Verify's. Recorded for `specs/11-subagent-catalog.md`.

21. `story-ac-verifier` receives three prompt fields and no conversation: the story path, the `--json` `data` of `story files --diff`, and the merged test output. Its freshness is a property of what the skill puts in the prompt, and a subagent's context is its prompt plus the files it reads, so no further mechanism is needed and none is claimed.

22. The integration pass is triggered by `## Layer` and `## Interface`, not by a named API. The task shape for this phase describes `## Interface` as naming an API; `specs/05-plan.md` §Outputs section 6 defines `## Interface` as the table `UI | Screen | States covered`, which names a screen. The `interface` layer is the one whose `config.toml` globs are `**/api/**`, `**/controllers/**`, `**/routes/**`, `**/cli/**`, `**/ui/**`, `**/components/**`, `**/pages/**`, so it is the layer an API lives in. `integration-test-writer` therefore runs when `## Layer` is `interface` or `## Interface` carries a row, and `integration.reason` in the note records which of the two fired.

23. The optional external AST tool the existing agent set leans on is dropped, and no fallback for its absence is stated because nothing depends on it. §1.2 forbids a skill, command, subagent, or hook from naming a build tool, and the three AST-tool reference files under `C:\\Users\\bryan\\.claude\\agents\\` instruct a subagent to invoke one named external binary with a named flag. The smell detection they provided is replaced by the `build-complexity` exit code and the `AP-nnn` detectors of `antipattern scan`, both of which read numbers and patterns the project itself owns. The three files retire with their parents; recorded for `specs/11-subagent-catalog.md`.

24. The skill runs the test command with Bash and every other operation through `devforgeai`. The command file's `allowed-tools` lists `Bash(devforgeai:*)` so the four `!` preamble lines run; that frontmatter key governs the command file, and the skill body the command invokes runs under the session's own tool permissions, which is where the test command from `config get` is executed. No language, runner, or package manager appears in the command file, the skill, or any subagent.

25. Git is reached through `worktree ensure`, `worktree list`, `worktree remove`, and `commit`, and through no other path. §1.1 puts enforcement in the CLI, and a commit that bypassed `devforgeai commit` would skip the declared-file-set check that runs before staging. `git-validator` and `git-worktree-manager` retire for the same reason.

26. Parallel builds are two `/build` runs in two worktrees under `[build].worktree_root`, each with its own `state.toml` and therefore its own `[active].build`. The `PreToolUse` `story files --check` guard resolves `--id` from that value, so each run's guard tests its own story's `## Files` set and a write allowed in one worktree is unaffected by the other.

27. When two concurrent stories declare a shared path anyway, `worktree ensure` refuses. `specs/05-plan.md` §Gate forbids overlap inside one sprint through `story validate --scope sprint` check 9, `DFA-E237`, which covers `stories[]` and excludes `deferred[]`; an overlap survives that check when a story is deferred and built anyway, when a `## Files` table is edited after the plan gate passed, or when two sprints are open. In each of those cases `worktree ensure` reads `git worktree list`, intersects the requested story's `story files --list` set with the set of every other `STORY-nnn` holding a worktree, and exits 1 with `DFA-E272` naming both stories and the first shared path. The run stops with one `Blocked` line carrying that text, and the user finishes or removes the other worktree. Detection uses the worktree directory names rather than a story `status` frontmatter value, because a directory name is on disk whatever a run has or has not written, and a `status` read depends on the story document having been reached at all.

28. If the refusal is bypassed — a worktree created outside `worktree ensure` — the two guards both allow the shared path, each run commits it on its own branch, and the collision surfaces as a merge conflict when the second branch merges. No further mechanism is proposed: a cross-worktree lock is outside the primitives §2 lists.

29. The handoff `Phase` line renders the slug `-`, because rendering rule 3 takes the slug from the phase document's first H1 and `reports/STORY-nnn-build.yaml` is YAML with no H1. This matches `specs/05-plan.md` §Decisions 29.

30. The `Done` line reads `<n> tests · <c>% coverage`, which `specs/01-cli.md` §`Done` counts already fix for this phase, read from the report's `gate.checks` evidence and `coverage.overall`. `tests` renders `-` when the stack's count pattern did not match the output.

31. A subagent whose JSON does not parse against its schema is invoked once more with the parse error appended; a second failure leaves the gate metric absent for a registered verifier, which `gate check` reports as `DFA-E316`, and stops the run for the other four. This mirrors `specs/05-plan.md` §Decisions 27.

32. A criterion whose new test passes before any implementation stops the loop and is reported by `ac-test-writer` as a blocking finding, because the test asserts something the code already does and the criterion adds nothing this run can implement. It reaches Plan as an `AC-nnn` on the `--remedy` list alongside the other blocking findings.

33. Commit granularity is one commit per red, one per green, one per refactor pass, and one per integration pass, with the messages `<AC-nnn> red`, `<AC-nnn> green`, `<AC-nnn> refactor`, `integration`, and `AP remediation`. `devforgeai commit` prefixes each with `<STORY-nnn>: `, so the `commit-msg` hook of §7 passes without the skill carrying an instruction about message content.

34. `--resume` re-enters at the first `AC-nnn` with no `cycles` entry or with an empty `green_commit`, read from `.devforgeai/build/STORY-nnn-note.yaml`. An absent note makes the criterion set the whole `## Acceptance Criteria` list, which is the `full` run, so a `--resume` after a crash before step 12 loses no correctness and repeats work.

35. `--remedy` on `/build` takes `FIND-nnn` ids and no others, because Verify allocates the `FIND` prefix (§5) and its findings are what a Verify send-back cites. Each `FIND-nnn` resolves to one `AC-nnn` through the `findings[]` entry of `reports/STORY-nnn-qa.yaml`, and the criterion set is the distinct `AC-nnn` values. §4c fixes the flag name and the comma-separated form.

36. Coverage is measured over the whole project by `coverage_min`, not over the story's files. The story's `## Layer` selects which `[[layer]].coverage_min` its source sits under; every layer's threshold is evaluated on every build gate, which is `specs/01-cli.md` §Gate's `layers = ["domain", "application", "infrastructure", "interface"]` verbatim. A story that lowers another layer's percentage fails the gate, which is the intent.

37. `DFA-E271`, `DFA-E272`, `DFA-E273`, `DFA-E270`, `DFA-E413`, and `DFA-W243` are proposed additions to the error table of `specs/01-cli.md`, in that table's format:

    | Code | Subcommand | Condition | Message | Exit |
    |---|---|---|---|---|
    | DFA-E270 | antipattern scan | a detector matched | `<AP-nnn> matched at <path>:<line>: <text>` | 1 |
    | DFA-E271 | worktree, commit | the project root is not a git work tree | `<path> is not a git work tree; worktrees and commits need one` | 1 |
    | DFA-E272 | worktree ensure | a concurrent worktree's story declares a shared path | `<STORY-nnn> and <STORY-nnn> both declare <path>; finish or remove <path-of-worktree> first` | 1 |
    | DFA-E273 | worktree remove | the worktree holds unmerged work | `<path> holds <n> uncommitted files and <m> commits absent from <ref>` | 1 |
    | DFA-E413 | report note | the fragment fails its schema | `<path> key <key> is <problem> for schema devforgeai/build-note/1` | 1 |
    | DFA-W243 | commit | nothing to stage | `no changed file inside the declared set of <STORY-nnn>; nothing committed` | 0 |

38. Dependency: the forward handoff `Next` line is `/verify <STORY-nnn>`, and `specs/07-verify.md` settles `/verify`'s `argument-hint`. A different first-argument type there changes this one line and the `remedy_line` grader's `forbid_next` argument alone.

39. Dependency: the receiving form `/build <STORY-nnn> --remedy FIND-nnn,...` binds what `specs/07-verify.md` prints on its SEND BACK `Next` line, because §1.7 has the user type what the handoff printed. `specs/01-cli.md` §Handoff already gives that row as `/build <STORY-nnn> --remedy <FIND ids>`, so the two agree and no amendment is proposed.

40. Proposed amendment to `specs/01-cli.md` §`handoff`, rendering rule 6: when two or more verifier blocks share the lowest `passed/total`, the entry rendered is the first of them in `config.toml` `[[verifier]]` order. The rule names the lowest ratio and leaves a tie open, and Build registers three verifiers that all read 1.0 on a PASS run, so the rendered `Verified` line would otherwise be unspecified.

41. Proposed amendment to `specs/01-cli.md` §`handoff`, rendering rule 5: the `<n> checks` evidence of a PASS counts every `[[gate.check]]` entry the gate evaluated, at either severity and including entries recorded `skip`. The rule gives the text without saying what `n` counts, and this gate holds twelve entries of which `build-complexity` and `build-design` carry `severity = "warn"`, so a count over block-severity entries alone would print `10` for a gate whose report shows twelve rows.

42. Proposed addition to the `cases.jsonl` schema in `specs/01-cli.md` §Evals: an optional `setup.extends` key holding the `id` of an earlier case in the same file. The runner writes that case's `setup.files` map into the workspace first, then this case's map over it, so a path in both takes this case's content. A chain of `extends` resolves outermost first; a cycle, or an `id` that no earlier line defines, is a runner usage error, exit 3. Without it every case of this skill repeats an eleven-file fixture, and a pseudo-path key standing for the other files is a path the runner would create literally.

43. Blocker: none. Every step runs inside a Claude Code terminal session with the primitives §2 lists, and every capability this spec adds is a `devforgeai` subcommand, a `gates.toml` check kind, a `config.toml` key, or a hook entry that conventions §7 already shapes.
