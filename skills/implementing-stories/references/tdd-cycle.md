# The per-criterion cycle

Read this before the first criterion of workflow step 7. The loop body runs once per `AC-nnn` in `## Acceptance Criteria` order over the run's criterion set, and it runs the criteria to exhaustion or to the first stop. Every step below names its actor, what it takes in, what it leaves behind, and where the run goes when it fails.

The shape of the cycle is one criterion at a time, test before code. The record that the discipline held is two different 40-hex commit shas in one `cycles` entry of the build note: the test existed and failed at `red_commit`, and the code that satisfies it landed at `green_commit`.

## 7.1 · Write the failing test case — `ac-test-writer`

One invocation per criterion, serially. Nothing runs beside it.

| Prompt field | Source |
|---|---|
| `ac` and its full `Given … When … Then …` line | the story's `## Acceptance Criteria` list |
| the other `AC-nnn` lines of the story | the same list |
| declared test files as `{path, layer}` | the `## Files` rows of `Kind` `test` |
| testing standards | `coding-standards.md` `## Testing standards` |
| naming conventions | `source-tree.md` `## Naming conventions` |
| the current content of each declared test file | Read of those paths |
| `seed_rows` as `entities[]` with their `name`, `fields`, and `rows` | `.devforgeai/explore/seed-data.json` when it exists; absent file means the field is omitted |
| `finding_summary`, `finding_evidence` | remedy runs only, from the R1 quadruple |

`seed_rows` carries the example data Explore shaped in Phase 0 and `decision.yaml` `carry_forward` names Build as the consumer of. A fixture drawn from those rows asserts against data the user already recognises; with the field omitted the agent composes its own rows, which is the shape a project that ran no Explore phase leaves.

It returns the `devforgeai/verifier/1` envelope with `findings[]` and a `payload` holding `testable`, `ac`, `test_paths`, `assertion`, and `conflicts_with`, and it writes one or more test files inside the declared set. `SubagentStop` runs `devforgeai report ingest ac-test-writer -`, which writes `verifiers.ac_testable`.

A returned `payload.testable` of `false` ends the loop. The criteria after the failing one stay unworked, so `cycles` is shorter than `## Acceptance Criteria`, and the run continues at step 12. Its blocking findings are SB-1, SB-2, and SB-3 of `## Send-back`.

## 7.2 · Run the tests — model

The model runs the string `devforgeai config get stack.test_command` printed at workflow step 6, with Bash, in the worktree — that is where the sources and tests sit. The `devforgeai` calls of this cycle need no particular directory; they reach the main checkout's `.devforgeai/` from either. The `PostToolUse` Bash hook matches the command against `config.toml` and runs `devforgeai gate check --phase build --partial`, which writes the report with `partial: true`. `PostToolUse` blocks on no exit code; its result reaches the model as `hookSpecificOutput.additionalContext` after the command. That partial report is what step 7.7 reads.

A non-zero exit code is the expected result: the test asserts something no code does yet.

A zero exit code ends the loop. The new test passed before any implementation, so the criterion adds nothing this run can build. The cycle entry keeps `green_commit` of `""`, the run continues at step 12, and the criterion is caught downstream: `ac-test-writer` returned `payload.testable` of `true` here, so `build-testable` reads 1.0 and `story-ac-verifier` at step 11 reads a diff with no hunk implementing the criterion and returns `no_diff_hunk_implements_it`, which is SB-4.

## 7.3 · Commit the red — CLI

```
devforgeai commit <STORY-nnn> -m "<AC-nnn> red"
```

The CLI stages every changed path, refuses any path outside the story's `## Files` set with `DFA-E239`, prefixes the message with `<STORY-nnn>: `, and commits, so the `pre-commit` and `commit-msg` hooks run on it. The printed sha is the cycle's `red_commit`. Exit 1 from a hook stops the run with one `Blocked` line carrying that hook's stderr.

## 7.4 · Make it pass — `backend-implementer` or `frontend-implementer`

One invocation per criterion, serially. `frontend-implementer` takes the criterion when the story's `## Layer` is `interface` and its `## Interface` table carries at least one row; `backend-implementer` takes it otherwise.

| Prompt field | Source |
|---|---|
| `ac` and its line | `## Acceptance Criteria` |
| failing test paths, their content, and the failing output | step 7.1 and step 7.2 |
| declared source files as `{path, layer}` | the `## Files` rows of `Kind` `source`, `config`, `migration`, `asset` |
| `layer` | the story's `## Layer` |
| `constraints` as `{id, statement, binds}` | the story's `## Constraints` |
| `anti_patterns` as `{id, severity, scope}` | the story's `## Anti-patterns` |
| approved and forbidden dependencies | `dependencies.md`, `backend-implementer` only |
| error handling, logging, formatting | `coding-standards.md` |
| the source paths earlier cycles wrote | the run's own `cycles` list |
| the `UI-nnn` tables and the token leaf names | workflow step 3, `frontend-implementer` only |

A returned `status: blocked` ends the loop and the run continues at step 12. The `blocked.reason` names which of the four — or, for `frontend-implementer`, which of the six — conditions held, and `blocked.ids` names the `CON-nnn` or `AP-nnn` behind it.

## 7.5 · Run the tests again — model

The same command, the same hook. A zero exit code goes to 7.6.

A non-zero exit code re-invokes the same implementer once, with the failing output appended to the prompt. A second non-zero exit ends the loop and the run continues at step 12. The budget is one retry per criterion, because a second failure on the same test is a signal about the criterion rather than about the attempt.

## 7.6 · Commit the green — CLI

```
devforgeai commit <STORY-nnn> -m "<AC-nnn> green"
```

The printed sha is the cycle's `green_commit`, and the paths the implementer returned are the cycle's `source_paths`. Failure path as 7.3.

## 7.7 · Read the quality checks — CLI

```
devforgeai report show <STORY-nnn> build --check build-lint
devforgeai report show <STORY-nnn> build --check build-complexity
```

Each prints one check entry with a `status` of `pass`, `fail`, or `skip` and a `reason` string. The numbers behind them belong to the gate; what the loop takes from them is which of the two, if either, says `fail`.

Exit 1 on `DFA-E400` means no partial report exists yet, which happens when the `PostToolUse` hook has not fired for this story. Running the test command once more makes the hook write it, into the main checkout's `.devforgeai/reports/` where the report lives.

## 7.8 · Refactor — `refactor-surgeon`

Invoked when at least one of the two entries carries `status: fail`, and not invoked otherwise. A rewrite opened by a CLI number rather than by taste is what keeps this pass from touching code no check objects to.

| Prompt field | Source |
|---|---|
| `trigger` | `build-lint` or `build-complexity`, the failing check id |
| `reason` | that check entry's `reason` string |
| `paths` | the paths this cycle wrote |
| `complexity_max`, `duplication_max_percent` | `devforgeai config get build.complexity_max` and `build.duplication_max_percent` |
| formatting and naming | `coding-standards.md` `## Formatting` and `## Naming` |

It returns `status: rewritten` with a `changes[]` list, or `status: declined` with a reason from its three-value enum. The model then runs the test command and, on a zero exit code, runs `devforgeai commit <STORY-nnn> -m "<AC-nnn> refactor"`. A non-zero exit re-invokes the subagent once with the failing output appended; a second non-zero exit ends the loop and the run continues at step 12.

A `declined` result leaves the check failing, which the gate records: `build-lint` carries the gate's default block severity, and `build-complexity` carries `severity = "warn"` and annotates the result rather than setting it.

## The commit messages

Five messages, one per kind of pass. `devforgeai commit` prefixes each with `<STORY-nnn>: `, which is what satisfies the `commit-msg` hook.

| Step | Message |
|---|---|
| 7.3 | `<AC-nnn> red` |
| 7.6 | `<AC-nnn> green` |
| 7.8 | `<AC-nnn> refactor` |
| 8 | `integration` |
| 10 | `AP remediation` |

## What the loop leaves in the note

One `cycles` entry per criterion the run worked, in `## Acceptance Criteria` order:

```yaml
- ac: AC-003
  test_paths: [<a ## Files Path of Kind test>]
  red_commit: <40 lowercase hex>
  implementer: backend-implementer
  green_commit: <40 lowercase hex, or "" while unfinished>
  source_paths: [<a ## Files Path of Kind source, config, migration or asset>]
```

One `refactors` entry per pass that committed, with `trigger`, the copied `reason`, `paths`, and `commit`; `[]` when no pass ran. The `integration` mapping is present in every note, carrying `ran: false` and `reason: not-applicable` when step 8 did not run.
