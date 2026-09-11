# `/build` — a worked walkthrough

Phase 4. One `STORY-nnn` that Plan marked `ready` becomes committed source and test files inside a git worktree, test-first: one failing test per `AC-nnn` before the code that satisfies it, one commit per red and per green.

Build decides nothing about what the story says. It edits no `STORY-nnn.md`, no `sprint.yaml`, no context file, and no `UI-nnn.md`, and a defect in any of them leaves as a SEND BACK citing ids. It runs no git command of its own — every worktree operation and every commit goes through `devforgeai`, so the hooks run on each one. It interprets no test count, coverage figure, lint result, or complexity number; those are `gate check` evidence.

---

## Entry

| Form | Run kind |
|---|---|
| `/build STORY-nnn` | full — implement every criterion |
| `/build STORY-nnn --remedy FIND-nnn,FIND-nnn` | remedy — Verify cited those findings |
| `/build STORY-nnn --resume` | resume — re-enter at the first unfinished criterion |

A `$1` whose prefix is not `STORY` stops the run with one `Blocked` line naming the accepted prefix.

Four preamble lines:

```
!`devforgeai gate require build $ARGUMENTS[0]`
!`devforgeai doc load story $ARGUMENTS[0]`
!`devforgeai doc load context all`
!`devforgeai doc load sprint -`
```

- `gate require build` — exit 1 names the missing plan gate and the body does not load.
- `doc load story` — the whole `STORY-nnn.md`, its ten sections and its frontmatter.
- `doc load context all` — the six context files.
- `doc load sprint -` — `stories/sprint.yaml`, whose `epic` key is the `EPIC-nnn` a send-back cites.

None of the four allocates an id. **This phase allocates none at all**: every id it writes was allocated by Plan, so no preamble line and no step can abort on an exhausted prefix.

---

## The exchange

```
> /build STORY-001
```

**Step 1 — establish the run.** Run kind from `$ARGUMENTS`; subject from `$1`; epic from the `epic` key of the printed `sprint.yaml`.

**Step 2 — read the story and the context set.** Output: the ordered `AC-nnn` list of `## Acceptance Criteria`; the `Path` set of `## Files` with each row's `Kind` and `Layer`; the `## Layer` value; the `CON-nnn` rows of `## Constraints`; the `AP-nnn` rows of `## Anti-patterns`; the `STORY-nnn` list of `## Dependencies`; the `## Testing standards` rows of `coding-standards.md`; the `## File placement rules` and `## Naming conventions` rows of `source-tree.md`.

A frontmatter `status` of `draft` stops the run with one `Blocked` line naming the status and `/plan <EPIC-nnn> --resume`.

**Step 3 — read the screen specs.** For each `UI-nnn` in `## Interface`, zero to four of them:

```
devforgeai doc load ui-spec UI-004
devforgeai config get frontend.tokens_path
```

```
$ devforgeai config get frontend.tokens_path
.devforgeai/brand/tokens.json
```

Reproduced, exit 0. Then Read the printed path. Exit 1 on `DFA-E200` stops the run with one `Blocked` line naming the `UI-nnn` and `/design <UI-nnn> --spec`.

**Step 4 — open the worktree.**

```
devforgeai worktree ensure STORY-001
```

prints the worktree path. Output: a worktree at `<[build].worktree_root>/<STORY-nnn>` on branch `<[build].branch_prefix><STORY-nnn>`, carrying **its own `state.toml`**, and the path the rest of the run works in. With the defaults `init` writes, that is `../wt/STORY-001` on branch `story/STORY-001`:

```toml
[build]
worktree_root = "../wt"
branch_prefix = "story/"
base_ref = "HEAD"
complexity_max = 10
duplication_max_percent = 5.0
```

Reproduced from `.devforgeai/config.toml`.

| Exit | Code | Means |
|---|---|---|
| 1 | `DFA-E272` | a concurrent worktree's story declares a shared path; the run stops naming both story ids and the path |
| 1 | `DFA-E271` | the project root is not a git work tree; the run stops naming `git init` |

Two `/build` runs in two worktrees under `worktree_root` are the supported parallel shape: each keeps its own `state.toml` and therefore its own `[active].build`, so each run's write guard tests its own story's `## Files` set and a write allowed in one worktree is unaffected by the other.

**Step 5 — activate the phase.**

```
devforgeai phase set build --id STORY-001
```

inside the worktree. Writes `[current].phase`, `[current].id`, `[active].build`, and the story frontmatter `status` value `building`. From here the `PreToolUse` `story files --check` hook resolves `--id` from `[active].build` and guards every write, the model's and each subagent's alike.

Exit 1 on `DFA-E320` means the plan gate is not PASS; the run stops and the Stop hook prints the gate result.

**Step 6 — read the command.**

```
$ devforgeai config get stack.test_command
npm run test
```

Reproduced, exit 0, on a project where `stack detect` found a `package.json`. That one string is what steps 7.2, 7.5, 7.8, 8, and 10 run with Bash. Reading it from the CLI is what keeps a runner name out of the skill.

An empty printed value — which `degraded = true` produces — stops the run with one `Blocked` line naming `devforgeai stack detect`.

---

## Step 7 — the per-criterion loop

Once per `AC-nnn` in `## Acceptance Criteria` order, to exhaustion or to the first stop. The criterion set is every criterion on a `full` run, the R1 criteria on a `remedy` run, and the criteria from `resumed_at` onward on a `resume` run.

| Sub-step | Actor | What happens |
|---|---|---|
| **7.1** | `ac-test-writer` | turns the one criterion into one failing test inside the declared test files. A returned `payload.testable` of `false` stops the loop; the run continues at step 12, which is the send-back |
| **7.2** | model, Bash | runs the step 6 command in the worktree. A **zero** exit means the new test passed before any implementation; the loop stops with `green_commit` at `""` and the run continues at step 12 |
| **7.3** | CLI | `devforgeai commit STORY-001 -m "AC-001 red"` records the failing test. The printed sha is `red_commit` |
| **7.4** | `backend-implementer`, or `frontend-implementer` when `## Layer` is `interface` and `## Interface` carries a row | writes the smallest code inside the declared set that satisfies the test |
| **7.5** | model, Bash | runs the command again. A non-zero exit re-invokes the same implementer once with the failing output appended; a second non-zero exit stops the loop |
| **7.6** | CLI | `devforgeai commit STORY-001 -m "AC-001 green"` records the code. The printed sha is `green_commit` |
| **7.7** | CLI | `devforgeai report show STORY-001 build --check build-lint` and `--check build-complexity` return the two entries the `PostToolUse` Bash hook wrote |
| **7.8** | `refactor-surgeon` | runs **only** when at least one of those two entries carries `status: fail`, then the command, then a commit `-m "AC-001 refactor"` on a zero exit |

Two different shas in one cycle entry are the record that the test existed and failed before the code existed.

7.8 is the point of the whole arrangement: a rewrite is opened by a number rather than by taste. `refactor-surgeon` does not run on a passing pair.

At 7.7, exit 1 on `DFA-E400` means no partial report exists yet; running the test command once more makes the `PostToolUse` hook write it. That hook is registered `async` on a `Bash` or `PowerShell` command matching a `config.toml` test command, calls `gate check --phase build --partial`, and returns its result as `additionalContext` on the next turn.

At 7.3 and 7.6, exit 1 from the `pre-commit` hook's `doc validate` or `context audit` stops the run with one `Blocked` line carrying the hook's stderr.

---

## Steps 8 to 13

**8. Integration tests.** `integration-test-writer` runs when `## Layer` is `interface` or `## Interface` carries at least one row, and does not run otherwise. Then the test command and `devforgeai commit STORY-001 -m "integration"`. A non-zero exit re-invokes once with the failing output appended; a second non-zero exit ends the loop and the run continues at step 12.

**9. `context-validator`.** One invocation over:

```
devforgeai story files --diff --id STORY-001 --json
```

plus the six context file paths, the `CON-nnn` rows, and the `## Layer` value. `SubagentStop` ingests its stdout into `verifiers.context`. A block still unparsed after one retry sits at `status: unparsed`, which fails the `build-context` check.

**10. Anti-pattern scan.**

```
devforgeai antipattern scan --id STORY-001
```

Exit 1 prints one `DFA-E270` line per match:

```
<AP-nnn> matched <path>:<line>: <text>
```

The remedy is `refactor-surgeon` with the match list, the test command, a commit `-m "AP remediation"`, and the scan once more. A second exit 1 leaves the matches for the `build-antipatterns` check.

**11. `story-ac-verifier`.** Three prompt fields and no others: the absolute path of `STORY-001.md`, the `data` object of `story files --diff --json`, and the merged stdout and stderr of the last test command. The prompt carries **no part of this conversation** and no output of steps 7.1 through 10, which is what makes the verdict independent of the session that wrote the code. It returns one `payload.checks` entry per criterion, each with a `verdict` of `met` or `unmet`. Its stdout reaches `verifiers.story_ac` through `SubagentStop`.

**12. Record the run.** Write `.devforgeai/build/STORY-001-note.yaml` from `templates/build-note.yaml` with the cycle data, the refactor entries, the integration entry, and the commit shas. That path matches no `doc validate` doc-type row, so no producer check applies and no hook reads it; it carries `schema` and `id` as ordinary top-level keys.

A finished cycle entry carries two different 40-hex values in `red_commit` and `green_commit`. An unfinished one carries `green_commit` of `""` and `source_paths` of `[]`, which is the state `--resume` re-enters at.

**13. Merge the record into the report.**

```
devforgeai report note STORY-001 build --key build --file .devforgeai/build/STORY-001-note.yaml
```

The CLI drops the fragment's `schema` and `id` keys and writes the rest at the report's top-level `build` key. This is the one path by which this skill contributes to a CLI-owned report; a Write to the report itself is refused with `DFA-E212`. Exit 1 on `DFA-E413` names the offending key.

---

## What the write guard does

`[current].phase` is `build`, so the `PreToolUse` hook tests every write outside `.devforgeai/` against the active story's `## Files` table. A path outside it is denied with `DFA-E239`:

```
<path> is outside the declared file set of <STORY-nnn>
```

A write **under** `.devforgeai/` still takes the producer check instead:

```
$ echo '{"tool_name":"Write","tool_input":{"file_path":".devforgeai/stories/STORY-001.md","content":"..."}}' \
    | devforgeai hook run pre-tool-use
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"DFA-E212 phase 'explore' does not write story; 'planning-work' does"}}
```

Reproduced, exit 2 — that transcript was taken with `[current].phase` at `explore`; from Build the same write is refused with `phase 'build' does not write story`.

A path outside the project root is refused too while a Build run has an active story, because the declared set is the story's own scope and a path outside the project is outside it by construction. The message reads:

```
devforgeai: '<path>' does not resolve under the project root <root>, so it is outside the active story's declared file set
```

read from `cli/src/hooks/run.rs`.

---

## The gate

The Stop hook runs `devforgeai gate check --phase build`, which writes `.devforgeai/reports/STORY-001-build.yaml`. Its required check kinds are `doc_valid`, `tests_pass`, and `coverage_min`, each with `severity = "block"`. Three registered verifiers each have their own `verifier_pass` check, so one ratio does not compensate for another:

| Verifier | Report field | Unit |
|---|---|---|
| `ac-test-writer` | `verifiers.ac_testable` | AC |
| `story-ac-verifier` | `verifiers.story_ac` | ACs |
| `context-validator` | `verifiers.context` | files |

Reproduced from the `[[verifier]]` rows in `.devforgeai/config.toml`.

The compiled coverage floors bind here, in `config.toml`, while `degraded = false`: domain ≥ 90.0, application ≥ 80.0, infrastructure ≥ 70.0, interface ≥ 60.0, `[coverage].overall_min` ≥ 75.0. Lowering one is refused:

```
$ devforgeai gate check --phase build --id STORY-001 --no-run
devforgeai: DFA-E303 gate check: config.toml sets layer.domain.coverage_min to 80, below the compiled minimum 90
```

Reproduced, exit 1.

---

## The handoff

```
Phase     4 · Build           STORY-001 · match-a-statement-line-t
Done      214 tests · 87.4% coverage
Gate      PASS  6 checks
Verified  story-ac-verifier · 7/7 ACs +2 more

Next      /verify STORY-001
Then      /release v0.1.0
Blocked   none

Full report: .devforgeai/reports/STORY-001-build.yaml
```

The `Done` line reads the `passed` evidence of a `tests_pass` check and the `coverage.overall` value from the report, rendering `- tests · - coverage` when either is absent. The `Then` line is omitted while `state.toml` `[active].release` is empty.

---

## Send-back

Build has one upstream target, Plan. Four conditions, each a blocking finding in a verifier block.

| Id | Condition | Detected by | Ids cited |
|---|---|---|---|
| SB-1 | a criterion's `Then` clause names no outcome a test reads, its `When` clause names no action, or its `Given` clause names no reachable state | an `ac-test-writer` finding at `severity: block` with `reason` in `then_names_no_readable_outcome`, `when_names_no_action`, `given_names_no_reachable_state` | the `AC-nnn` |
| SB-2 | two criteria of the story assert opposed outcomes for one state and action | `reason` of `contradicts_other_ac` | the criterion worked and every id in its `payload.conflicts_with` |
| SB-3 | a criterion's outcome lives in a file `## Files` does not declare | `reason` of `outcome_outside_declared_files` | the `AC-nnn` |
| SB-4 | a criterion the run implemented is not satisfied by the diff and the test output read in a fresh context | a `story-ac-verifier` finding at `severity: block` | the `AC-nnn` |

On any of the four the worktree stays on disk with every commit the run made, the story file keeps `status: building`, `state.toml` keeps `[current].phase` of `build` and `[active].build` of the story id, and no byte of `.devforgeai/stories/`, `.devforgeai/context/`, or `.devforgeai/ui-specs/` changes.

```
Next      /plan EPIC-001 --remedy AC-003,AC-004
Then      /build STORY-001 --resume
```

`EPIC-nnn` is the `epic` key of `sprint.yaml`, which the preamble printed.

On a SEND BACK the Stop hook renders the block with **no** block on the turn, at exit 0, because the next step is a command you type.

---

## Receiving a send-back — the remedy run

`/build STORY-001 --remedy FIND-061,FIND-021` arrives from Verify. Step R1 replaces step 1's tail, and steps 4 through 14 then run with the R1 criterion set and `run: remedy`.

**R1.**

```
devforgeai doc load qa-report STORY-001
```

For each cited id, take the `findings[]` entry whose `id` matches, producing one `(FIND-nnn, AC-nnn, summary, evidence)` quadruple. The criterion set of step 7 is the distinct `AC-nnn` values in `## Acceptance Criteria` order, and step 7.1 receives each criterion's `summary` and `evidence` as two further prompt fields, so the new test asserts the gap the finding named.

A cited `FIND-nnn` appearing in no `findings[]` entry stops the run with one `Blocked` line naming the id.

Step 4 reuses the existing worktree. Criteria outside the set keep their tests, their code, and their commits: no file they own is written, which `story files --diff` records as an unchanged path. The note's `remedy` key carries the cited `FIND-nnn` list and `cycles` holds one entry per criterion of the R1 set alone.

### `--resume`

Re-enters at step 2 and re-reads the story and the six context files from the preamble's stdout, so a criterion Plan rewrote is the text the run works from. Step 4 reuses the worktree and step 5 is a no-op that leaves `status` at `building`.

The criterion set begins at the first `AC-nnn` with no `cycles` entry in `.devforgeai/build/STORY-001-note.yaml`, or whose entry carries an empty `green_commit`, and runs to the end of `## Acceptance Criteria`. The note's `resumed_at` records that id and `cycles` keeps every earlier entry byte for byte. An absent note makes the criterion set the whole list, which is the `full` run.

---

## Files this phase owns

| Path | Template |
|---|---|
| `.devforgeai/build/STORY-nnn-note.yaml`, one per story | `templates/build-note.yaml` |

Source, test, config, migration, and asset files inside the story's `## Files` set are the run's other output. They carry no frontmatter and live in the worktree, not under `.devforgeai/`.

`.devforgeai/reports/STORY-nnn-build.yaml` is this phase's typed document, and the CLI writes it: `produced_by` is `devforgeai-cli`, `report ingest` fills the three verifier blocks from `SubagentStop`, `report note` fills the top-level `build` key at step 13, and `gate check` fills the rest.
