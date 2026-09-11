# `/plan` — a worked walkthrough

Phase 3. One epic's accepted requirements become one `STORY-nnn.md` per unit of work and one `sprint.yaml` naming the subset that fits capacity, in dependency order.

Plan writes two document types and nothing else. It edits no upstream document — a defect in `requirements.yaml`, in a context file, or in an ADR leaves as a SEND BACK citing ids.

---

## Entry

| Form | Run kind |
|---|---|
| `/plan EPIC-nnn` | full — decompose that epic |
| `/plan SPRINT-nnn` | full — the epic is the `epic` key of `stories/sprint.yaml` |
| `/plan EPIC-nnn --remedy AC-nnn,AC-nnn` | remedy — Build or Verify cited those criteria |
| `/plan EPIC-nnn --resume` | resume — re-enter at step 2 after a send-back returned |

An id in `$1` whose prefix is neither `EPIC` nor `SPRINT` stops the run with one `Blocked` line naming those two accepted prefixes.

Three preamble lines:

```
!`devforgeai gate require plan $ARGUMENTS[0]`
!`devforgeai doc load requirements $ARGUMENTS[0]`
!`devforgeai doc load context all`
```

- `gate require plan` — exit 1 names the missing predecessor gate and the body does not load, which is the gate.
- `doc load requirements` — the whole `.devforgeai/requirements.yaml`. The subcommand ignores its `<id>` argument, so either prefix reaches the same file.
- `doc load context all` — the six context files.

None of the three allocates an id. `SPRINT`, `STORY` and `AC` ids are allocated at steps 4 and 8, once the run kind is known, so a remedy run that allocates nothing cannot abort on an exhausted prefix.

---

## The exchange

```
> /plan EPIC-001
```

**Step 1 — establish the run.** `remedy` when `$ARGUMENTS` holds `--remedy`, `resume` when it holds `--resume`, `full` otherwise. The epic is `$1` for an `EPIC` prefix, and the `epic` key of `sprint.yaml` for a `SPRINT` prefix.

**Step 2 — read the epic.** From the `requirements.yaml` text on the preamble's stdout: the `epics[]` entry whose `id` is the run's epic, then the `requirements[]` records its `requirements` list names whose `status` is `accepted`. A record at `status: withdrawn`, or at `priority: wont`, is dropped here and named in step 8's `## Out of scope`. An epic id resolving to no `epics[]` entry stops the run with one `Blocked` line naming the id and the epic ids the file does hold.

**Step 3 — read the context set.** From the six files on the preamble's stdout:

| Source | Taken |
|---|---|
| `source-tree.md` `## Layers` second table | the layer name set |
| `source-tree.md` | `## File placement rules`, `## Naming conventions`, `## Generated and excluded paths` rows |
| `architecture-constraints.md` | `## Constraint index` rows at `Status: active`, and `## Layer dependency rules` rows |
| `anti-patterns.md` | `## Anti-pattern index` rows |
| `coding-standards.md` | `## Testing standards` rows |
| `tech-stack.md` / `dependencies.md` | `## Excluded technologies` and `## Forbidden dependencies`, which keep an excluded name out of an acceptance criterion |

A `## Layers` table with no data row stops the run with one `Blocked` line naming `source-tree.md ## Layers`.

**Step 4 — open the sprint.** When `sprint.yaml` exists and its `epic` key equals the run's epic, reuse that file's `id` and allocate nothing. Otherwise:

```
devforgeai doc validate --allocate SPRINT
devforgeai phase set plan --id SPRINT-001 --epic EPIC-001
```

`--epic` carries the `EPIC-nnn` step 1 settled. The call needs it whenever `sprint.yaml` is absent, which is every full run — the file is written at step 12 — and every resume that follows a send-back, where step 12 did not run. With the file present the flag repeats the file's own `epic` key, and a value differing from it is `DFA-E013`.

That call sets `state.toml` `current_phase` to `plan` and `[active].plan` to this id, which is where `report ingest` resolves its target report at step 9. Exit 1 on `DFA-E320` means the constitute gate is not PASS; the run stops and the Stop hook prints the gate result.

**Step 5 — decompose.** `story-decomposer`, alone. It returns one draft per story with `req_ids`, `story_lines`, `acceptance_criteria`, `layer`, `con_ids`, `ap_ids`, `ui_ids`, `needs_screen`, `depends_on`, `points`, and `out_of_scope`, plus `uncovered_reqs` and `notes`. `depends_on` holds run-local `key` values, because no `STORY-nnn` exists yet.

Failure path: output that does not parse re-invokes the subagent once with the parse error appended; a second failure leaves the gate metric absent, which `gate check` reports as exit 1. Every subagent in this phase follows that same budget.

**Step 6 — specify screens.** A draft whose `needs_screen` is `true` — its `layer` is `interface` and every `REQ-nnn` in its `req_ids` carries an empty `traces_to` — has no screen specification yet. Through the Skill tool, invoke the `design` skill with `STORY-nnn --spec` once per such draft, then read each returned id back:

```
devforgeai doc load ui-spec UI-004
```

Four sections of that document feed the story: `## Requirements` fixes the `REQ-nnn` set the screen realizes, `## States` fixes the state list section 6 records, `## Breakpoints` fixes the four layouts one `AC-nnn` covers, and `## Out of scope` fixes what section 10 records.

This step and step 8 interleave: the allocation comes from step 8 and the ids come back into step 8's `## Interface` table and frontmatter `consumes`.

When Design returns a SEND BACK to Discover, write the stories held so far at `status: draft`, write no `sprint.yaml`, and let the handoff carry Design's `Next` line.

**Step 7 — plan the file sets.** `story-file-set-planner` returns one `files[]` table per draft — a `path`, a `kind`, and a `layer` per row — plus `overlaps` and `unplaceable`. A non-empty `overlaps` or `unplaceable` list goes back through one further invocation with those entries appended.

**Step 8 — write the stories.**

```
devforgeai doc validate --allocate STORY
devforgeai doc validate --allocate AC
```

once per draft and once per criterion, then write each `.devforgeai/stories/STORY-nnn.md` from `templates/story.md` at `status: draft`. Ten sections in template order:

```markdown
---
schema: devforgeai/story/1
id: STORY-001
phase: plan
status: draft
produced_by: planning-work
consumes: [REQ-001, CON-002, AP-001]
open_questions: []
---

# STORY-001: Match a statement line to a ledger entry

## Story
## Requirements
## Acceptance Criteria
## Constraints
## Anti-patterns
## Interface
## Layer
## Files
## Dependencies
## Out of scope
```

Section names reproduced from `skills/planning-work/templates/story.md`.

A story's `consumes` holds at least one `REQ-nnn` and each entry matches `^(REQ|CON|AP|UI)-\d{3}$`.

The `## Files` table is the writable set Build is held to. The `PreToolUse` `story files --check` hook tests every Build write against it, and a path outside it is `DFA-E239`.

Exit 1 on `DFA-E215` from `--allocate` stops the run with one `Blocked` line naming the exhausted prefix. The `PostToolUse` hook runs `doc validate` on each written path and hands the result back as `additionalContext`.

**Step 9 — audit.** `story-invest-auditor`, this phase's registered verifier. It returns the `devforgeai/verifier/1` envelope with `passed`, `total`, `unit: stories`, and `findings[]`. The `SubagentStop` hook runs `devforgeai report ingest story-invest-auditor -`, which writes `verifiers.story_invest` into `.devforgeai/reports/SPRINT-001-plan.yaml` and creates that file when it is absent.

**Step 10 — rewrite on judgment findings.** Each `findings[]` entry at `severity: warn` cites a `STORY-nnn` and names an independence, size, or wording problem. Edit those stories, then invoke the auditor once more over the edited set. **The loop runs once.** A `warn` finding surviving the second pass stays in the report, annotates the gate, and changes no gate result. A finding at `severity: block` sets no edit here — that is the send-back.

**Step 11 — sequence and fill.** `sprint-sequencer` takes every story id with its `points` estimate and its `## Dependencies` id list, plus `points_max` from `[plan].sprint_capacity_points` or the default `20`, and which of the two it came from. It returns the `stories[]` list in topological order with `order` and `points`, the `deferred[]` list with a `reason` of `capacity` or `dependency`, the `capacity` object, `longest_chain`, and `cycle`.

A non-empty `cycle` stops the run with one `Blocked` line naming the cycle path; `devforgeai story validate` reports the same condition as `DFA-E232`:

```
dependency cycle: <A> -> <B> -> <A>
```

**Step 12 — write the sprint.**

```yaml
schema: devforgeai/sprint/1
id: SPRINT-001
phase: plan
status: active
produced_by: planning-work
consumes: [EPIC-001]
open_questions: []
epic: EPIC-001
capacity: {points_max: 20, points_planned: 3}
stories:
  - id: STORY-001
    order: 1
    points: 3
    status: ready
deferred: []
```

This exact file was written and read by the binary for the reproduced outputs below.

Then edit each story named in `stories[]` and in `deferred[]` to carry the frontmatter `status` value `ready`. `ready` is Plan's value to write: it is absent from `devforgeai phase set`'s transition table, which owns `building`, `built`, and `released`.

Sprint `status` runs `planned`, `active`, `closed`. Plan writes `active` at step 12, because the gate's `plan-sprint-status` check reads that value; Release writes `closed`.

---

## The gate

Five checks, from `.devforgeai/gates.toml`:

```toml
[[gate]]
phase = "plan"
requires = "constitute"
description = "Stories and sprint parse, every AC is testable, references resolve, no dependency cycle."

  [[gate.check]]
  kind = "doc_valid"
  id = "plan-docs"
  docs = ["stories/STORY-*.md", "stories/sprint.yaml"]

  [[gate.check]]
  kind = "story_valid"
  id = "plan-stories"
  scope = "sprint"

  [[gate.check]]
  kind = "ids_resolve"
  id = "plan-ids"
  prefixes = ["STORY", "AC", "REQ", "SPRINT", "UI"]
  on_fail = "send_back"

  [[gate.check]]
  kind = "field_in_enum"
  id = "plan-sprint-status"
  path = ".devforgeai/stories/sprint.yaml"
  field = "status"
  values = ["active"]

  [[gate.check]]
  kind = "verifier_pass"
  id = "plan-invest"
  verifiers = ["story-invest-auditor"]
  min_ratio = 1.0
  on_fail = "send_back"
```

Reproduced from the file `init` wrote.

Two checks carry `on_fail = "send_back"`, which is what makes this gate able to return exit 2 rather than exit 1. `plan-stories` runs `devforgeai story validate --scope sprint` and reports through it:

| Code | Condition |
|---|---|
| `DFA-E230` | an AC has no testable predicate; the grammar is Given/When/Then or a single assertion line |
| `DFA-E231` | a story consumes a `REQ-nnn` `requirements.yaml` does not define |
| `DFA-E232` | a dependency cycle |
| `DFA-E233` | `sprint.yaml` lists a `STORY-nnn` with no file |
| `DFA-E234` | a story defines no acceptance criteria |
| `DFA-E235` | an epic requirement is covered by no story |
| `DFA-E236` | an `AC-nnn` appears in no `Covered by` cell of `## Requirements` |
| `DFA-E237` | two stories declare the same path |
| `DFA-E238` | a story cites a `UI-nnn` that `.devforgeai/ui-specs/` does not hold |
| `DFA-E245` | one `AC-nnn` appears in the `Covered by` cell of two requirements |

`plan-invest` carries `min_ratio = 1.0`, which is **not** a compiled floor for this phase — the `verifier_pass.min_ratio ≥ 1.0` floor binds the `verify` gate alone. Lowering plan's value is accepted; lowering verify's is `DFA-E303`.

---

## The handoff

```
Phase     3 · Plan            SPRINT-001 · -
Done      1 stories · 4 ACs
Gate      PASS  5 checks
Verified  story-invest-auditor · 1/1 stories

Next      /build STORY-001
Then      /verify STORY-001
Blocked   none

Full report: .devforgeai/reports/SPRINT-001-plan.yaml
```

The `Done` counts are `STORY-*.md` files in `stories/` and `AC-nnn` ids defined under `.devforgeai/stories/`. The slug comes from the first H1 of `sprint.yaml`.

---

## Send-back

Two upstream targets, both raised by a `story-invest-auditor` finding at `severity: block`. The blocking finding's `id` prefix routes the trip: `REQ` to Discover, `CON` and `AP` to Constitute, `UI` to Design. Blocking findings of two prefixes route to Constitute, because a missing constraint is repaired before the requirement it would have decided.

| Id | Condition | Ids cited | Target |
|---|---|---|---|
| SB-1 | a requirement's `statement` admits two incompatible readings, each yielding a different `Then` clause | the `REQ-nnn` | Discover |
| SB-2 | a requirement's `acceptance_signal` names no outcome a test reads | the `REQ-nnn` | Discover |
| SB-3 | the constraint set decides no case the story meets, in a layer the `## Layer dependency rules` table covers | the `CON-nnn` in that layer row's `Constraint` column | Constitute |
| SB-4 | two active constraints bind one path set in opposed directions | both `CON-nnn` | Constitute |

On either target the stories written so far stay on disk at `status: draft`, `sprint.yaml` is not written, `requirements.yaml` and the six context files keep every byte, and `state.toml` keeps `current_phase` of `plan`.

```
Next      /discover IDEA-001 --remedy REQ-014,REQ-019
Then      /plan EPIC-001 --resume
```

`IDEA-nnn` is the top-level `id` of `.devforgeai/requirements.yaml`, which the preamble printed — the document's own id, **not** the epic the run took as `$1`. A send-back to Constitute prints `/constitute IDEA-001 --remedy CON-004` on the `Next` line and the same `Then` line.

A missing or thin `UI-nnn` is **not** a send-back. It is repaired inside the run by invoking the `design` skill with `UI-nnn --remedy AC-nnn,...`, because Design is cross-cutting and holds no gate.

---

## Receiving a send-back — the remedy run

`/plan SPRINT-001 --remedy AC-003,AC-004` arrives from Build or from Verify, and runs four steps in place of steps 2 to 13. This is the `Next` line the Verify send-back reproduced below prints:

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

**R1. Locate the criteria.** For each cited id:

```
devforgeai report show STORY-001 build
devforgeai report show STORY-001 verify
```

and pair the id with its `findings[]` entry, producing one `(STORY-nnn, AC-nnn, finding summary)` triple. A cited `AC-nnn` appearing in no story file stops the run with one `Blocked` line naming the id.

**R2. Triage.** `spec-gap-triager` takes the triples, the criterion text, the `REQ-nnn` whose `Covered by` cell holds each criterion with its `statement` and `acceptance_signal`, the active constraints binding the story's layer or file paths, and the `UI-nnn` ids in its `## Interface`. It returns one `disposition` per triple from a closed four:

| Disposition | What R3 does |
|---|---|
| `rewrite_ac` | edit that one criterion line in place in its `STORY-nnn.md`, leaving every other byte unchanged |
| `send_to_design` | invoke the `design` skill with `UI-nnn --remedy AC-nnn,...` and rewrite section 6 from the returned spec |
| `send_back_discover` | write no file |
| `send_back_constitute` | write no file |

An edit that breaks the `Given … When … Then …` grammar reaches the model after the write as `additionalContext` from the `PostToolUse` `doc validate`, carrying `DFA-E230`.

**R4. Reset and close.** Write `status: ready` into each story edited at R3 and into that story's entry in `sprint.yaml` `stories[]`, changing no other key of `sprint.yaml`, then:

```
devforgeai phase set plan --id SPRINT-001 --epic EPIC-001
```

with the id and the `epic` key the file on disk carries. A remedy run reaches R4 with `sprint.yaml` present, so both values are read from it.

The handoff `Next` line reads `/build STORY-001 --resume` when every disposition was `rewrite_ac` or `send_to_design`, and the send-back form above otherwise.

### `--resume`

Re-enters at step 2 and re-reads `requirements.yaml` and the six context files from the preamble's stdout, so a repaired `REQ-nnn` or a new `CON-nnn` is the text the run works from. Step 4 reuses the `SPRINT-nnn` that `[active].plan` holds and allocates none. Every `STORY-nnn.md` already on disk at `status: draft` is kept and re-audited at step 9, and the ids it holds are not re-allocated. `sprint.yaml`, absent after a send-back, is written at step 12.

---

## Files this phase owns

| Path | Template |
|---|---|
| `.devforgeai/stories/STORY-nnn.md`, one per unit of work | `templates/story.md` |
| `.devforgeai/stories/sprint.yaml`, one per project | `templates/sprint.yaml` |

Both carry the seven envelope keys in order; the story as YAML frontmatter, the sprint as its first seven keys followed by `epic`, `capacity`, `stories`, `deferred`. `phase` is `plan` and `produced_by` is `planning-work` in both. The sprint's `consumes` holds exactly one entry, the epic.
