# `/verify` — a worked walkthrough

Phase 5. One built story is reviewed from a context that did not write the code, and what the review found is recorded in `.devforgeai/reports/STORY-nnn-qa.yaml`.

Verify touches no source file, no test, no story, and no context file. It runs no test command, no coverage command, and no lint command: `gate check` ran those during Build and runs the coverage and lint checks again at this gate, and this run copies the numbers from the build report. It decides no threshold either — coverage floors live in `config.toml` `[[layer]].coverage_min`, the complexity and duplication ceilings in `config.toml` `[verify]`, and the gate criteria in `gates.toml`.

---

## Entry

| Form | Run kind | Mode |
|---|---|---|
| `/verify STORY-nnn` | full | `config.toml` `[verify].mode` |
| `/verify STORY-nnn --deep` | full | deep — the same run with three extra reviews |
| `/verify STORY-nnn --remedy FIND-nnn,...` | remedy | Release cited those findings |
| `/verify STORY-nnn --remedy STORY-nnn` | remedy | Release found no PASS report for that story; R1 opens the full run for it |
| `/verify STORY-nnn --resume` | resume | Build or Plan repaired the cited ids |

`--deep` and `--remedy` combine. The mode is `deep` when `$ARGUMENTS` holds `--deep` and `config.toml` `[verify].mode` otherwise, so a project that reviews deeply on every story sets the key once:

```toml
[verify]
mode = "light"
complexity_max = 10
duplication_max_percent = 5.0
duplication_min_lines = 20
metrics_command = ""
call_graph_command = ""
```

Reproduced from `.devforgeai/config.toml`.

Four preamble lines:

```
!`devforgeai gate require verify $ARGUMENTS[0]`
!`devforgeai doc load story $ARGUMENTS[0]`
!`devforgeai doc load context all`
!`devforgeai report show $ARGUMENTS[0] build`
```

None of the four allocates an id. The `FIND` band is allocated at step 5, once the run kind is known, so a remedy run that re-disposes existing findings cannot abort on an exhausted prefix.

---

## The exchange

```
> /verify STORY-001
```

**Step 1 — establish the run.** Run kind, mode, and the `STORY-nnn` of `$1`. A `$1` prefix other than `STORY` stops the run with one `Blocked` line naming the accepted prefix.

**Step 2 — read the story.** The sections this phase locates by heading text:

| Heading | Taken |
|---|---|
| `## Acceptance Criteria` | the `AC-nnn` list |
| `## Constraints` | the `CON-nnn` table |
| `## Anti-patterns` | the `AP-nnn` table |
| `## Interface` | the `UI-nnn` table |
| `## Layer` | the one layer name |
| `## Files` | the `Path \| Kind \| Layer` table that bounds every scan |
| `## Out of scope` | the behaviours a finding does not name |

A `## Files` section with no data row stops the run with one `Blocked` line naming the story and that heading, because the file set bounds every scan.

**Step 3 — read the build report.**

```
devforgeai report show STORY-001 build
```

Take `gate.result`, the `gate.checks[]` list of `id` and `status`, and the whole `coverage` block of `format`, `source`, `overall`, `layers[]`, and `unassigned`. That block is copied into the QA report unchanged at step 10 and handed to `coverage-gap-auditor` at step 6.

Exit 1 on `DFA-E400` or `DFA-E401` stops the run with one `Blocked` line naming the path, because this phase measures no coverage of its own.

**Step 4 — open the phase.**

```
devforgeai phase set verify --id STORY-001
```

Writes `state.toml` `[current]` and `[active].verify`, and sets the story's frontmatter `status` to `built`. The CLI is the one writer of that value. Exit 1 on `DFA-E320` means the build gate is not PASS; the run stops and the Stop hook prints the gate result.

**Step 5 — allocate the finding band.**

```
devforgeai doc validate --allocate FIND
```

once, and take the returned id as `base`. The subagent at position `k` of the `agents.md` `## Contracts` order, counting from zero, gets the band `FIND-(base + 100k)` through `FIND-(base + 100k + 99)`, and numbers its findings from the low end.

One allocation per run rather than one per finding, because the subagents are read-only and the finding count is unknown before they run. The band is 100 wide so that it fixes ids and bounds nothing else: **every subagent reports every finding it has, at every severity, with no cap and no self-filtering**, and the dispositions of step 8 and the gate do the filtering. An agent told to report only what matters under-reports.

**Step 6 — review, seven subagents, one parallel batch.**

| Subagent | Unit | What it gets |
|---|---|---|
| `ac-compliance-verifier` | ACs | the story path, the `AC-nnn` list with full text, the `## Files` rows of `Kind` `source` and `test`, the `## Out of scope` lines, its id band |
| `standards-reviewer` | files | the `## Files` rows of `Kind` `source`, the seven `coding-standards.md` sections, the `CON-nnn` each rule is enforced by, the `## Accessibility` rows for an `interface` file |
| `anti-pattern-scanner` | anti-patterns | the `## Anti-pattern index` rows with `Category`, `Severity`, `Scope`, `Detector kind`, `Detector`, `Source`, and the `## Files` rows each `Scope` glob matches |
| `constraint-auditor` | constraints | the `## Constraints` rows, the matching `### CON-nnn` blocks, the `## Layer dependency rules` row for the story's layer, the `## Files` rows, the `AC-nnn` list |
| `coverage-gap-auditor` | layers | the build report `coverage` block, the story's `## Layer`, the `## Files` rows, the `AC-nnn` list |
| `dead-code-detector` | symbols | the `## Files` rows of `Kind` `source`, the `## Roots` and `## Generated and excluded paths` entries, `[verify].call_graph_command` |
| `deferral-validator` | deferrals | the deferrals already on disk: entries of every `reports/STORY-*-qa.yaml` whose `target` is this story, plus this story's own prior report on a resume or remedy run |

Each returns one `devforgeai/verifier/1` object on stdout. The `SubagentStop` hook runs `devforgeai report ingest <name> -`, which writes `verifiers.<report_field>` into `.devforgeai/reports/STORY-001-verify.yaml` and appends the findings:

```
$ devforgeai report ingest ac-compliance-verifier - --id STORY-001 --phase verify
Ingested  ac-compliance-verifier · 5/7 ACs -> .devforgeai/reports/STORY-001-verify.yaml
```

Reproduced, exit 0, from this envelope on stdin:

```json
{"schema":"devforgeai/verifier/1","subagent":"ac-compliance-verifier",
 "passed":5,"total":7,"unit":"ACs",
 "findings":[
   {"id":"FIND-001","severity":"block","summary":"AC-003 has no test covering the empty statement path","evidence":"tests/test_match.py has no case for an empty file","category":"missing-test","file":"tests/test_match.py"},
   {"id":"FIND-002","severity":"block","summary":"AC-004 asserts no error path","evidence":"src/match.py raises nothing on a malformed line","category":"missing-test","file":"src/match.py"},
   {"id":"FIND-003","severity":"warn","summary":"the matcher duplicates a guard","evidence":"src/match.py:40 and :81","category":"duplication","file":"src/match.py"}],
 "payload":{}}
```

JSON that does not parse is ingested with `status: unparsed`; invoke that one subagent once more with the parse error appended. A block still unparsed leaves the `verifier_pass` check failing with `DFA-E316`.

The envelope `severity` enum is `block`, `warn`, `info`. `blocker` in the QA report emits `block`, and `high`, `medium`, `low` emit `warn`. A `block` finding lowers that subagent's `passed` below its `total`, which fails `verifier_pass` at the compiled floor `min_ratio = 1.0` — and that is how a blocker-severity match reaches the gate.

**Step 7 — deep review, three subagents, deep mode alone.**

| Subagent | Unit | Extra |
|---|---|---|
| `security-auditor` | OWASP categories | `findings[]` each with `confidence` and `owasp`; `payload` is `{}` |
| `code-quality-auditor` | files | `findings[]` each with `confidence`, `measured` and `limit`; `payload` holding `method` and `over_ceiling` |
| `adr-conformance-reviewer` | ADRs | every accepted `ADR-nnn` with `## Decision`, `## Consequences`, `## Constraints introduced` |

In light mode this step does not run, `checks[]` holds seven entries rather than ten, and the `verify-deep` gate check records `skip` with `reason: condition`.

Two of these agents run one command each, and the `PreToolUse` shell arm is what permits it. Naming the permitted command in `config.toml` rather than in the agent's tool grant means the project decides what they may run:

```
$ echo '{"tool_name":"Bash","agent_type":"code-quality-auditor","tool_input":{"command":"radon cc -s src"}}' \
    | devforgeai hook run pre-tool-use
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"devforgeai: code-quality-auditor may run only the command config.toml records at [verify].metrics_command. It asked to run 'radon cc -s src'. Set that key, or have the agent run what it names."}}
```

Reproduced, exit 2. With `[verify].metrics_command` set to exactly that string, the same call returns `permissionDecision: allow` — these agents carry no other shell grant, so the allow is what lets the one command through. `dead-code-detector` is held to `[verify].call_graph_command` the same way.

An agent may read a tool's numeric output **only to copy it into its finding and its `payload` verbatim** — `measured` and `limit` on a complexity finding, `payload.dead` on a dead-code block. Comparing that number against a ceiling is a `gate check` check kind reading `config.toml` and `gates.toml`.

**Step 8 — disposition.** Each ingested finding takes one value from `fix`, `defer`, `accept`:

| Finding | Disposition |
|---|---|
| `severity: blocker` | `fix` |
| `category: spec-gap` | `fix`, and routes to Plan |
| remedy is one Definition-of-Done item another story carries | `defer` |
| the story's `## Out of scope` already excludes it | `accept` |
| cites a `file` value absent from `## Files` | dropped; its `evidence` line is recorded under `open_questions` |

**Step 9 — validate the drafted deferrals.** Write one `deferrals[]` entry per finding of `disposition: defer`, then invoke `deferral-validator` once more over the drafted entries with the `sprint.yaml` `stories[]` list and the `adr/` frontmatter statuses. An entry whose `target` resolves to no document, whose `reason` the evidence does not support, or whose `target` story already defers back to this story comes back as a `block` finding in that subagent's second block, which fails `verifier_pass` and routes the run to the send-back.

**Step 10 — write the report.** `.devforgeai/reports/STORY-001-qa.yaml`, sixteen top-level keys in template order:

```yaml
schema: devforgeai/qa-report/1
id: STORY-001
phase: verify
status: send_back
produced_by: validating-quality
consumes: [STORY-001, AC-001, AC-002, AC-003, AC-004, CON-002, AP-001]
open_questions: []
mode: light
verified_on: 2026-09-11
build_report: .devforgeai/reports/STORY-001-build.yaml
coverage:
  ...copied from step 3, recomputed by nothing...
checks:
  ...one row per subagent, built from its passed and total...
findings:
  - id: FIND-001
    severity: blocker
    summary: AC-003 has no test covering the empty statement path
    ...
    disposition: fix
blockers: [FIND-001, FIND-002]
deferrals: []
summary:
  ...the seven counts...
```

Shaped from `templates/qa-report.yaml`.

`status` is `pass` when `blockers` is empty and every check ran, `send_back` when the run leaves by the send-back, and `fail` otherwise. `verified_on` is the `YYYY-MM-DD` the run wrote the file, and every `deferrals[].opened_on` equals it. `build_report` equals `coverage.source`. `checks` holds seven entries in light mode and ten in deep.

The `PostToolUse` `doc validate` hook returns its result as `additionalContext`, naming a key to rewrite.

### Two report files, one phase

| Path | Written by | Role |
|---|---|---|
| `reports/STORY-nnn-qa.yaml` | this skill | the review, its findings, its dispositions, its deferrals |
| `reports/STORY-nnn-verify.yaml` | `gate check --phase verify` and `report ingest` | the gate report; the file the handoff's `Full report:` line cites |

A write to `stories/STORY-nnn.md`, which is Plan's document, comes back as hook exit 2. No file under the project's source roots is opened for writing by this skill or by any of its subagents.

---

## The gate

```toml
[[gate]]
phase = "verify"
requires = "build"
on_fail = "fail"
send_back_to = "build"
description = "The QA report parses, every registered verifier passed, no blocker finding stands, every deferral resolves without a cycle, coverage by layer holds, lint is clean."

  [[gate.check]]
  kind = "doc_valid"
  id = "verify-docs"
  docs = ["reports/{id}-qa.yaml"]

  [[gate.check]]
  kind = "verifier_pass"
  id = "verify-acs"
  verifiers = ["ac-compliance-verifier"]
  min_ratio = 1.0
  on_fail = "send_back"
```

Reproduced from the file `init` wrote; the gate holds ten checks in total.

A run with the report absent:

```
$ devforgeai gate check --phase verify --id STORY-001
Gate      verify · STORY-001
  fail    verify-docs       doc_valid       DFA-E200 ...\reports/STORY-001-qa.yaml not found
  fail    verify-acs        verifier_pass   DFA-E316 report .devforgeai/reports/STORY-001-verify.yaml has no verifiers block for 'ac-compliance-verifier'
  fail    verify-light      verifier_pass   DFA-E316 report .devforgeai/reports/STORY-001-verify.yaml has no verifiers block for 'standards-reviewer'
  fail    verify-deep       verifier_pass   DFA-E200 ...\reports/STORY-001-qa.yaml not found
  fail    verify-blockers   length_between  DFA-E200 ...\reports/STORY-001-qa.yaml not found
  fail    verify-ids        ids_resolve     DFA-E325 1 references do not resolve: ADR-000
  fail    verify-deferral-cycleno_cycle        DFA-E345 gates.toml check 'verify-deferral-cycle': docs ["reports/STORY-*-qa.yaml"] matched no document
  skip    verify-coverage   coverage_min    degraded
  skip    verify-lint       lint_clean      degraded
  fail    verify-story-statusfield_in_enum   DFA-E200 ...\stories/STORY-001.md not found
Result    SEND BACK
Report    .devforgeai/reports/STORY-001-verify.yaml
```

Reproduced, exit 2, paths shortened.

`min_ratio = 1.0` for this gate is a **compiled floor**. Lowering it is refused before anything else runs:

```
$ devforgeai gate check --phase verify --id STORY-001
devforgeai: DFA-E303 gate check: gates.toml gate 'verify' sets verifier_pass.min_ratio to 0.5, below the compiled minimum 1

$ devforgeai gate require verify STORY-001
devforgeai: DFA-E303 gate require: gates.toml gate 'verify' sets verifier_pass.min_ratio to 0.5, below the compiled minimum 1
```

Reproduced, both exit 1.

---

## The handoff — a SEND BACK

```
$ devforgeai handoff
Phase     5 · Verify          STORY-001 · match-a-statement-line-t
Done      5/7 ACs · 6 findings
Gate      SEND BACK to Plan  verify-docs DFA-E200...
Verified  ac-compliance-verifier · 5/7 ACs
Found     AC-003 DFA-E325 2 references do not resolve: AC-003, AC-004
Found     +5 more in report

Next      /plan SPRINT-001 --remedy AC-003,AC-004
Then      /verify STORY-001 --resume
Blocked   none

Full report: .devforgeai/reports/STORY-001-verify.yaml
```

Reproduced, exit 0.

Three things to read out of it.

**The `Gate` line says Plan, and `gates.toml` says `send_back_to = "build"`.** The static value is the default; `gate check` resolves the report's own target from the blocking findings. A blocking finding of `category: spec-gap` resolves `plan`, every other blocking finding resolves `build`, and a set holding both resolves `build` — a defect in the code is repaired before the criterion that failed to catch it.

**`Found` is capped.** Three lines maximum, and the last folds into `+n more in report` when the budget is spent. Order is severity `block`, then `warn`, then `info`, then ID ascending. The `Verified` line costs one of the twelve lines, which is why only one finding rendered here.

**The Stop hook does not block on this.** Exit 0, `systemMessage` alone, no blocking decision. The next step is a command you type, and holding the model in the turn cannot produce it.

---

## Send-back

### To Build — six conditions

| Id | Condition | Detected by |
|---|---|---|
| SB-1 | an `AP-nnn` of `Severity` `blocker` matches a file of the story's `## Files` | `anti-pattern-scanner` |
| SB-2 | a `CON-nnn` the story binds is violated by its file set | `constraint-auditor` |
| SB-3 | no test reads the outcome an `AC-nnn` names | `ac-compliance-verifier` |
| SB-4 | a layer's covered-line share is below its `config.toml` floor | `coverage-gap-auditor`, and check `verify-coverage` |
| SB-5 | an OWASP category is reachable and exploitable in the file set | `security-auditor`, deep mode |
| SB-6 | a deferral names an unresolved target, an unsupported reason, or a chain that returns to this story | `deferral-validator`, and checks `verify-ids`, `verify-deferral-cycle` |

The ids cited are the `FIND-nnn` of each such finding:

```
Next      /build STORY-014 --remedy FIND-061,FIND-021
Then      /verify STORY-014 --resume
```

The story keeps `status: built` on this path, the code keeps every byte until Build edits it, and `state.toml` keeps `[current].phase` of `verify`.

### To Plan — two conditions

| Id | Condition | Detected by |
|---|---|---|
| SB-7 | the code meets the `AC-nnn` as written and the criterion asserts less than its `REQ-nnn` states | `ac-compliance-verifier` |
| SB-8 | a `CON-nnn` the story binds holds, and no `AC-nnn` asserts the outcome the constraint requires | `constraint-auditor` |

The ids cited are the `AC-nnn`, **not** the `FIND-nnn`, because Plan re-opens criteria and the `FIND-nnn` stays in this report as the evidence for the rewrite. `<EPIC-nnn>` is the `epic` key of `stories/sprint.yaml`:

```
Next      /plan EPIC-004 --remedy AC-007
Then      /verify STORY-014 --resume
```

---

## Receiving a send-back — the remedy run

`/verify STORY-001 --remedy FIND-nnn,...` arrives from Release when a deferral blocks deployment, and runs four steps in place of steps 2 to 11. `/verify STORY-nnn --remedy STORY-nnn` arrives when a story is at `built` with no PASS verify report; R1 opens the full run from step 2 for that story.

**R1.** Read `reports/STORY-001-qa.yaml` from disk and run `devforgeai report show v0.1.0 release` for the release gate report's `findings[]`, producing one `(FIND-nnn, deferrals[] entry, release finding summary)` triple per cited id. Exit 1 from that call means the cited ids come from `$ARGUMENTS` alone. A cited `FIND-nnn` appearing in no `findings[]` entry of this story's report stops the run with one `Blocked` line naming the id.

**R2. Re-dispose.** A cited finding whose remedy is now code takes `disposition: fix`, its `deferrals[]` entry is removed, and `blockers` and `summary` are recomputed. A cited finding whose evidence shows the deferral stands keeps `disposition: defer` with a new `target` and a `reason` of `external_blocker`, which Release re-reads on its `--resume` run. Both outcomes are recorded in the same file.

**R3. Re-review the cited paths.** Invoke, in one batch, the subagents whose name appears in the `found_by` value of the cited findings, scoped to the `file` values of those findings and restricted to rows of the story's `## Files`. Their fresh blocks replace the prior ones for those subagents. **A remedy run raises no new finding outside the cited paths.**

**R4. Close.** Rewrite the QA report with the re-disposed findings, then `devforgeai phase set verify --id STORY-001`. The handoff `Next` is `/build STORY-001 --remedy FIND-nnn,...` when a re-disposed finding is now `fix`, and `/release v0.1.0 --resume` when every cited deferral closed without code work.

### `--resume`

Re-enters at step 3 and re-reads `reports/STORY-001-build.yaml`, so a rebuilt story's fresh coverage and gate results are the numbers the run works from. Step 5 allocates a **new** `base`, because the prior run's ids are in the index and the allocator returns the next free one.

Every `findings[]` entry of the prior report at `disposition: defer` or `accept` is carried into the new report unchanged, including its `deferrals[]` entry and its original `opened_on`; every entry at `disposition: fix` is dropped and re-raised by the step 6 batch when the defect stands. `[active].verify` already holds the story, so step 4 re-runs `phase set` and changes nothing.

---

## Files this phase owns

| Path | Template |
|---|---|
| `.devforgeai/reports/STORY-nnn-qa.yaml`, one per story per run | `templates/qa-report.yaml` |

Sixteen top-level keys in this order: `schema`, `id`, `phase`, `status`, `produced_by`, `consumes`, `open_questions`, `mode`, `verified_on`, `build_report`, `coverage`, `checks`, `findings`, `blockers`, `deferrals`, `summary`. `consumes` holds the `STORY-nnn`, every `AC-nnn` reviewed, and every `CON-nnn`, `AP-nnn`, `ADR-nnn`, and `UI-nnn` a finding or deferral cites.
