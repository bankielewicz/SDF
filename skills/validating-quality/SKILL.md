---
name: verify
description: Phase 5 of DevForgeAI, run by /verify. Reviews one built story from a context that did not write the code and records what it found in .devforgeai/reports/STORY-nnn-qa.yaml, holding FIND-nnn findings, their dispositions, and the deferrals[] sequence of Definition-of-Done items pushed to a later story. Light mode is seven judgements - acceptance-criteria compliance, a code review against coding-standards.md, an anti-pattern scan against the AP-nnn index, constraint validation against the CON-nnn index, a coverage review of the figures Build measured, dead-code detection, and deferral validation - and --deep adds a security audit, complexity and duplication metrics, and an ADR conformance review. Reach for it whenever /verify is typed, whenever a built story is being reviewed, whenever a QA report, a FIND-nnn, a deferral, a blocker finding, or a spec gap is in play, and whenever a Release send-back arrives as /verify STORY-nnn --remedy FIND-nnn.
argument-hint: <STORY-nnn> [--deep] [--remedy FIND-nnn,FIND-nnn] [--resume]
allowed-tools: Bash(devforgeai:*), PowerShell(devforgeai:*), Read, Write, Glob, Grep, Agent
disable-model-invocation: true
---

!`devforgeai gate require verify $ARGUMENTS[0]`
!`devforgeai doc load story $ARGUMENTS[0]`
!`devforgeai doc load context all`
!`devforgeai report show $ARGUMENTS[0] build`

# validating-quality

Verify reads what Build produced and records what it found. It writes one document,
`reports/STORY-nnn-qa.yaml`, and touches no source file, no test, no story, and no
context file. It runs no test command, no coverage command, and no lint command:
`devforgeai gate check` ran those during Build and runs the coverage and lint checks
again at this gate, and this run copies the numbers from the build report. It
decides no threshold either — coverage floors live in `config.toml`
`[[layer]].coverage_min`, the complexity and duplication ceilings in
`config.toml` `[verify]`, and the gate criteria in `gates.toml`. A defect it finds
in the implementation leaves as a send-back to Build; a defect it finds in an
acceptance criterion leaves as a send-back to Plan.

## Entry

`/verify` invokes this skill. Arguments:

| Form | Run kind | Mode | Meaning |
|---|---|---|---|
| `/verify STORY-nnn` | full | `config.toml` `[verify].mode` | review that built story |
| `/verify STORY-nnn --deep` | full | deep | the same run with the three extra reviews |
| `/verify STORY-nnn --remedy FIND-nnn,...` | remedy | as above | Release cited those findings and sent them back |
| `/verify STORY-nnn --remedy STORY-nnn` | remedy | as above | Release found no PASS report for that story; R1 opens the full run for it |
| `/verify STORY-nnn --resume` | resume | as above | Build or Plan repaired the cited ids and the run re-enters at step 3 |

An id in `$1` whose prefix is not `STORY` stops the run with one `Blocked` line
naming that accepted prefix. `--deep` and `--remedy` combine; the mode is `deep`
when `$ARGUMENTS` holds `--deep`, and `config.toml` `[verify].mode` otherwise, so a
project that reviews deeply on every story sets the key once.

The four preamble lines at the head of this file run before the body loads:

- `devforgeai gate require verify $ARGUMENTS[0]` — exit 1 names the missing build gate and the
  body does not load, which is the gate.
- `devforgeai doc load story $ARGUMENTS[0]` — `.devforgeai/stories/STORY-nnn.md`.
- `devforgeai doc load context all` — the six context files.
- `devforgeai report show $ARGUMENTS[0] build` — `.devforgeai/reports/STORY-nnn-build.yaml`.

None of the four allocates an id. The `FIND` band is allocated at step 5, once the run
kind is known, so a remedy run that re-disposes existing findings cannot abort on an
exhausted prefix.

Read from disk as the run needs them: `.devforgeai/config.toml` (`[verify].*`,
`[[layer]].coverage_min`), `.devforgeai/stories/sprint.yaml`,
`.devforgeai/adr/ADR-nnn.md` in deep mode, `.devforgeai/ui-specs/UI-nnn.md` for
each `UI-nnn` in the story's `## Interface`, and every
`.devforgeai/reports/STORY-*-qa.yaml` already on disk.

The story sections this phase locates by heading text are `## Acceptance Criteria`
(the `AC-nnn` list), `## Constraints` (the `CON-nnn` table), `## Anti-patterns`
(the `AP-nnn` table), `## Interface` (the `UI-nnn` table), `## Layer` (the one layer
name), `## Files` (the `Path | Kind | Layer` table that bounds every scan), and
`## Out of scope` (the behaviours a finding does not name). From the build report it
reads `gate.result`, `gate.checks[]` of `id` and `status`, and the `coverage` block
of `format`, `source`, `overall`, `layers[]`, and `unassigned`. From a `UI-nnn` it
reads `## Accessibility`, whose eight rows are `Landmark`, `Heading order`, `Name`,
`Role`, `Keyboard path`, `Focus visible`, `Contrast`, and `Motion`; an absent
`UI-nnn` file is a `spec-gap` finding citing that id.

## Workflow

Steps 1 to 11 are the full run, with step 7 running in deep mode alone. A remedy or
resume run goes to `## Remedy and resume` after step 1.

**1. Establish the run — model.** From `$ARGUMENTS` and the preamble stdout, take
the run kind (`remedy` when the string holds `--remedy`, `resume` when it holds
`--resume`, `full` otherwise), the mode (`deep` when the string holds `--deep`, and
`config.toml` `[verify].mode` otherwise), and the `STORY-nnn` of `$1`. A `$1`
prefix other than `STORY` stops the run with one `Blocked` line naming the accepted
prefix.

**2. Read the story — model.** From the `doc load story` stdout, take the `AC-nnn`
list, the `Path | Kind | Layer` rows of `## Files`, the layer name of `## Layer`,
the `CON-nnn` rows of `## Constraints`, the `AP-nnn` rows of `## Anti-patterns`, the
`UI-nnn` rows of `## Interface`, and the lines of `## Out of scope`. A `## Files`
section with no data row stops the run with one `Blocked` line naming the story and
that heading, because the file set bounds every scan.

**3. Read the build report — model, CLI.** Run `devforgeai report show $1 build`
through Bash. Take `gate.result`, the `gate.checks[]` list, and the whole `coverage`
block, which is copied into the QA report unchanged at step 10 and handed to
`coverage-gap-auditor` at step 6. Exit 1 on `DFA-E400` or `DFA-E401` stops the run
with one `Blocked` line naming the path, because this phase measures no coverage of
its own.

**4. Open the phase — model, CLI.** Run `devforgeai phase set verify --id
<STORY-nnn>` through Bash. It writes `state.toml` `[current]` and `[active].verify`
and sets the story's frontmatter `status` to `built`; the CLI is the one writer of
that value. Exit 1 on `DFA-E320` means the build gate is not `PASS`; the run stops
and the Stop hook prints the gate result.

**5. Allocate the finding band — model, CLI.** Run `devforgeai doc validate
--allocate FIND` once and take the returned id as `base`. The subagent at position
`k` of the `agents.md` `## Contracts` order, counting from zero, gets the band
`FIND-(base + 100k)` through `FIND-(base + 100k + 99)`, and numbers its findings from
the low end. One allocation per run rather than one per finding, because the
subagents are read-only and the finding count is unknown before they run. The band is
100 wide so that it fixes ids and bounds nothing else: every subagent reports every
finding it has, at every severity, and the dispositions of step 8 and the gate do the
filtering. Exit 1 on `DFA-E215` means the prefix is exhausted; the run stops with one
`Blocked` line naming it.

**6. Review — seven subagents, one parallel batch.** Invoke
`ac-compliance-verifier`, `standards-reviewer`, `anti-pattern-scanner`,
`constraint-auditor`, `coverage-gap-auditor`, `dead-code-detector`, and
`deferral-validator` in one message, each with its id band and the inputs
`references/light-checks.md` lists. `deferral-validator` sees the deferrals already
on disk here: the entries of every `reports/STORY-*-qa.yaml` whose `target` is this
story, plus, on a resume or remedy run, the `deferrals[]` of this story's own prior
report. Each returns one `devforgeai/verifier/1` object on stdout; the `SubagentStop`
hook runs `devforgeai report ingest <name> -`, which writes
`verifiers.<report_field>` into `.devforgeai/reports/STORY-nnn-verify.yaml` and
appends the findings. JSON that does not parse is ingested with `status: unparsed`;
invoke that one subagent once more with the parse error appended. A block still
unparsed leaves the `verifier_pass` check failing with `DFA-E316`.

**7. Deep review — three subagents, one parallel batch, deep mode alone.** When the
mode is `deep`, invoke `security-auditor`, `code-quality-auditor`, and
`adr-conformance-reviewer` in one message, with the inputs
`references/deep-checks.md` lists. Output and failure path are step 6's. In light
mode this step does not run, `checks[]` holds seven entries rather than ten, and the
`verify-deep` gate check records `skip` with `reason: condition` from its
`skip_when` on the report's `mode` field.

**8. Disposition — model.** Give each ingested finding one `disposition` from `fix`,
`defer`, `accept`. A finding of `severity: blocker` takes `fix`. A finding of
`category: spec-gap` takes `fix` and routes to Plan by `## Send-back`. A finding
whose remedy is one Definition-of-Done item another story carries takes `defer`. A
finding the story's `## Out of scope` already excludes takes `accept`. A finding
citing a `file` value absent from `## Files` is dropped and its `evidence` line is
recorded under `open_questions`. The enum and the reasoning behind each value are in
`references/findings-and-deferrals.md`.

**9. Run `deferral-validator` over the drafted deferrals — model, subagent.** Write one
`deferrals[]` entry per finding of `disposition: defer`, with the seven fields the
reference lists, then invoke `deferral-validator` once more over the drafted
entries with the `sprint.yaml` `stories[]` list and the `adr/` frontmatter statuses.
An entry whose `target` resolves to no document, whose `reason` the evidence does
not support, or whose `target` story already defers back to this story comes back as
a `block` finding in that subagent's second block, which fails `verifier_pass` and
routes the run to `## Send-back`. Unparsed JSON on the second invocation leaves the
block at `status: unparsed` and the gate fails with `DFA-E316`.

**10. Write the report — model.** Write `.devforgeai/reports/STORY-nnn-qa.yaml`
from `templates/qa-report.yaml`: the sixteen top-level keys in template order, the
`coverage` block copied from step 3, one `checks[]` row per subagent built from its
`passed` and `total`, the findings with their dispositions, the `deferrals[]`
entries of step 9, the `blockers` list holding every `findings[].id` of
`severity: blocker` in `findings[]` order, and `summary` holding the seven counts.
The PostToolUse `doc validate` hook returns its result to the model after the write
as `hookSpecificOutput.additionalContext`, naming a key; rewrite that key.

**11. Close — CLI, Stop hook.** The Stop hook runs `devforgeai gate check --phase
verify`, which writes `.devforgeai/reports/STORY-nnn-verify.yaml`, then
`devforgeai handoff`. The Stop hook renders the closing block; this skill writes no
part of it. On a gate FAIL the Stop hook holds the turn with the failing checks as
its `reason`, and the block appears when the turn ends; on a SEND BACK it renders the
block with no block on the turn, because the next step is a command the user types.

## Subagents

Ten agents, every one a registered verifier, every one read-only. Contracts, tools,
models, invocation order, and the `[[verifier]]` registry rows are in `agents.md`;
full schemas are in each `agents/<name>.md` `## Output`.

| Subagent | Invoked at | What to pass | What comes back |
|---|---|---|---|
| `ac-compliance-verifier` | step 6, batch of seven | the story path, the `AC-nnn` list with full text, the `## Files` rows of `Kind` `source` and `test`, the `## Out of scope` lines, its id band | `devforgeai/verifier/1`: `passed`, `total`, `unit: ACs`, `findings[]` |
| `standards-reviewer` | step 6, batch of seven | the `## Files` rows of `Kind` `source`, the seven `coding-standards.md` sections, the `CON-nnn` each rule is enforced by, the `## Accessibility` rows for an `interface` file, its id band | `devforgeai/verifier/1`: `unit: files` |
| `anti-pattern-scanner` | step 6, batch of seven | the `## Anti-pattern index` rows with `Category`, `Severity`, `Scope`, `Detector kind`, `Detector`, `Source`, the `## Files` rows each `Scope` glob matches, its id band | `devforgeai/verifier/1`: `unit: anti-patterns` |
| `constraint-auditor` | step 6, batch of seven | the `## Constraints` rows, the matching `### CON-nnn` blocks, the `## Layer dependency rules` row for the story's layer, the `## Files` rows, the `AC-nnn` list, its id band | `devforgeai/verifier/1`: `unit: constraints` |
| `coverage-gap-auditor` | step 6, batch of seven | the build report `coverage` block, the story's `## Layer`, the `## Files` rows, the `AC-nnn` list, its id band | `devforgeai/verifier/1`: `unit: layers` |
| `dead-code-detector` | step 6, batch of seven | the `## Files` rows of `Kind` `source`, the `## Roots` and `## Generated and excluded paths` entries, `[verify].call_graph_command`, its id band | `devforgeai/verifier/1`: `unit: symbols`, `findings[]` each with `confidence`, and `payload` holding `method` and `dead` |
| `deferral-validator` | step 6 over the deferrals on disk; step 9 over the drafted entries | the entries examined, the `sprint.yaml` `stories[]` list, the ADR frontmatter statuses, the `## Out of scope` lines, its id band | `devforgeai/verifier/1`: `unit: deferrals` |
| `security-auditor` | step 7, batch of three, deep mode | the `## Files` rows of `Kind` `source` and `config`, the `CON-nnn` of `kind: security`, the `AP-nnn` of `Category` `security`, the `## Forbidden dependencies` rows, its id band | `devforgeai/verifier/1`: `unit: OWASP categories`, `findings[]` each with `confidence` and `owasp`; `payload` is `{}` |
| `code-quality-auditor` | step 7, batch of three, deep mode | the `## Files` rows of `Kind` `source`, `[verify].complexity_max`, `[verify].duplication_max_percent`, `[verify].duplication_min_lines`, `[verify].metrics_command`, its id band | `devforgeai/verifier/1`: `unit: files`, `findings[]` each with `confidence`, `measured` and `limit`, and `payload` holding `method` and `over_ceiling` |
| `adr-conformance-reviewer` | step 7, batch of three, deep mode | every accepted `ADR-nnn` with `## Decision`, `## Consequences`, `## Constraints introduced`, the story's `## Files`, `## Layer`, `## Constraints`, the `## Layer dependency rules` table, its id band | `devforgeai/verifier/1`: `unit: ADRs` |

Every envelope carries the four fields this phase adds — `category`, `file`, `line`,
`relates_to` — which `report ingest` copies through unread. The envelope `severity`
enum is `block`, `warn`, `info`; `blocker` emits `block` and `high`, `medium`, and
`low` emit `warn`. A `block` finding lowers that subagent's `passed` below its
`total`, which fails `verifier_pass` at the compiled floor `min_ratio = 1.0`, and
that is how a blocker-severity match reaches the gate.

## Documents

| Document | Path | Template |
|---|---|---|
| QA report | `.devforgeai/reports/STORY-nnn-qa.yaml`, one per story per run | `templates/qa-report.yaml` |

Sixteen top-level keys, in this order, with no other top-level key: `schema`, `id`,
`phase`, `status`, `produced_by`, `consumes`, `open_questions`, `mode`,
`verified_on`, `build_report`, `coverage`, `checks`, `findings`, `blockers`,
`deferrals`, `summary`. The first seven are the conventions §5 set: `schema` is
`devforgeai/qa-report/1`, `id` is the `STORY-nnn` reviewed, `phase` is `verify`,
`produced_by` is `validating-quality`, `consumes` holds the `STORY-nnn`, every
`AC-nnn` reviewed, and every `CON-nnn`, `AP-nnn`, `ADR-nnn`, and `UI-nnn` a finding
or deferral cites, and each `open_questions` entry is one sentence of 1 to 200
characters.

`status` is `pass` when `blockers` is empty and every check ran, `send_back` when
the run leaves by `## Send-back`, and `fail` otherwise. `mode` is `light` or `deep`.
`verified_on` is the `YYYY-MM-DD` the run wrote the file, and every
`deferrals[].opened_on` equals it. `build_report` is the path step 3 read, equal to
`coverage.source`. `coverage` is copied from the build report and recomputed by
nothing. `checks` holds seven entries in light mode and ten in deep.
`references/findings-and-deferrals.md` carries the `findings[]`, `deferrals[]`,
`blockers`, and `summary` shapes.

Two report files carry this phase's name and they are different documents.
`reports/STORY-nnn-qa.yaml` is this skill's document, the one above.
`reports/STORY-nnn-verify.yaml` is the CLI's gate report, written by
`gate check --phase verify` and by `report ingest`, and it is the file the handoff
`Full report:` line cites. This skill writes the first; the CLI writes the second.

The PreToolUse hook runs `doc validate --producer-check` on every write under
`.devforgeai/`, and a write to `stories/STORY-nnn.md`, which is Plan's document,
comes back as hook exit 2. The PostToolUse hook runs `doc validate` and returns
its result to the model after the write as `hookSpecificOutput.additionalContext`,
naming the key to rewrite. No file under the
project's source roots is opened for writing by this skill or by any of its
subagents.

## Send-back

Two upstream targets, each exit 2 from `gate check`. The gate's static
`send_back_to` is `build`; `gate check` resolves the report's own target from the
blocking findings — a blocking finding of `category: spec-gap` resolves `plan`,
every other blocking finding resolves `build`, and a set holding both resolves
`build`, because a defect in the code is repaired before the criterion that failed
to catch it.

**To Build.** Six conditions, each a subagent finding at `severity: block`:

| Id | Condition | Detected by |
|---|---|---|
| SB-1 | An `AP-nnn` of `Severity` `blocker` matches a file of the story's `## Files` | `anti-pattern-scanner` |
| SB-2 | A `CON-nnn` the story binds is violated by its file set | `constraint-auditor` |
| SB-3 | No test reads the outcome an `AC-nnn` names | `ac-compliance-verifier` |
| SB-4 | A layer's covered-line share is below its `config.toml` floor | `coverage-gap-auditor`, and check `verify-coverage` |
| SB-5 | An OWASP category is reachable and exploitable in the file set | `security-auditor`, deep mode |
| SB-6 | A deferral names an unresolved target, an unsupported reason, or a chain that returns to this story | `deferral-validator`, and checks `verify-ids`, `verify-deferral-cycle` |

The ids cited are the `FIND-nnn` of each such finding. The handoff lines the Stop
hook prints:

```
Next      /build STORY-014 --remedy FIND-061,FIND-021
Then      /verify STORY-014 --resume
```

The story keeps `status: built` on this path, the code keeps every byte until Build
edits it, and `state.toml` keeps `[current].phase` of `verify`.

**To Plan.** Two conditions, both a finding of `category: spec-gap`:

| Id | Condition | Detected by |
|---|---|---|
| SB-7 | The code meets the `AC-nnn` as written and the criterion asserts less than its `REQ-nnn` states | `ac-compliance-verifier` |
| SB-8 | A `CON-nnn` the story binds holds, and no `AC-nnn` asserts the outcome the constraint requires | `constraint-auditor` |

The ids cited are the `AC-nnn`, not the `FIND-nnn`, because Plan re-opens criteria
and the `FIND-nnn` stays in this report as the evidence for the rewrite.
`<EPIC-nnn>` is the `epic` key of `.devforgeai/stories/sprint.yaml`:

```
Next      /plan EPIC-004 --remedy AC-007
Then      /verify STORY-014 --resume
```

The Stop hook's `devforgeai handoff` call composes both blocks from `state.toml` and
the latest report; this skill writes no part of them.

**Received.** Release sends `/verify STORY-nnn --remedy FIND-nnn,...` when a
deferral blocks deployment, and `/verify STORY-nnn --remedy STORY-nnn` when a story
is at `built` with no PASS verify report. Build sends
`/verify STORY-nnn --resume` after repairing the cited findings, and Plan sends the
same form after rewriting a cited criterion.

## Remedy and resume

**`--remedy FIND-nnn,...`** arrives from Release and runs four steps in place of
steps 2 to 11.

**R1. Locate the cited findings — model, CLI.** Read
`.devforgeai/reports/STORY-nnn-qa.yaml` from disk and run
`devforgeai report show <vX.Y.Z> release` for the release gate report's
`findings[]`, producing one `(FIND-nnn, deferrals[] entry, release finding summary)`
triple per cited id. Exit 1 from that call means the cited ids come from
`$ARGUMENTS` alone. A cited id carrying the `STORY` prefix rather than `FIND` opens
the full run from step 2 for that story. A cited `FIND-nnn` appearing in no
`findings[]` entry of this story's report stops the run with one `Blocked` line
naming the id.

**R2. Re-dispose — model.** A cited finding whose remedy is now code takes
`disposition: fix`, its `deferrals[]` entry is removed, and `blockers` and `summary`
are recomputed. A cited finding whose evidence shows the deferral stands keeps
`disposition: defer` with a new `target` and a `reason` of `external_blocker`, which
Release re-reads on its `--resume` run. Both outcomes are recorded in the same file.

**R3. Re-review the cited paths — subagents.** Invoke, in one batch, the subagents
whose name appears in the `found_by` value of the cited findings, scoped to the
`file` values of those findings and restricted to rows of the story's `## Files`.
Their fresh blocks replace the prior ones for those subagents; the failure path is
step 6's. A remedy run raises no new finding outside the cited paths.

**R4. Close — model, CLI.** Rewrite `.devforgeai/reports/STORY-nnn-qa.yaml` with
the re-disposed findings, then run `devforgeai phase set verify --id <STORY-nnn>`.
The Stop hook prints the block, whose `Next` is
`/build STORY-nnn --remedy FIND-nnn,...` when a re-disposed finding is now `fix`,
and `/release vX.Y.Z --resume` when every cited deferral closed without code work.

**`--resume`** re-enters at step 3 and re-reads `reports/STORY-nnn-build.yaml`, so a
rebuilt story's fresh coverage and gate results are the numbers the run works from.
Step 5 allocates a new `base`, because the prior run's ids are in the index and the
allocator returns the next free one. Every `findings[]` entry of the prior report at
`disposition: defer` or `accept` is carried into the new report unchanged, including
its `deferrals[]` entry and its original `opened_on`; every entry at
`disposition: fix` is dropped and re-raised by the step 6 batch when the defect
stands. `state.toml` `[active].verify` already holds the story, so step 4 re-runs
`phase set` and changes nothing.

## References

- `references/light-checks.md` — read before step 6: the seven check names with
  their verifiers and units, what each of the seven subagents receives, how a
  `total` of `0` becomes `result: skip`, and the unparsed-block path.
- `references/deep-checks.md` — read before step 7: the three deep check names, the
  closed OWASP enum, the two `config.toml` ceilings and the Grep fallback, and what
  `verify-deep` skips on in light mode.
- `references/findings-and-deferrals.md` — read before step 8 and before step 10:
  the id band arithmetic, the `findings[]` fields with the eleven-value category
  enum, the severity mapping between the two report files, the three dispositions,
  the `deferrals[]` fields with the five-value reason enum, and how `blockers` and
  `summary` are derived.
