---
schema: devforgeai-spec/1
doc: plan
status: draft
produced_by: plan-spec-author
consumes: [00-conventions]
open_questions: []
---

# Plan — the `planning-work` skill

Phase 3 of the DevForgeAI spec-driven development framework. Slash command `/plan`, per conventions §4b.

## Scope

Plan takes one epic and turns its requirements into buildable units. For the `EPIC-nnn` named on the command line it reads that epic's `REQ-nnn` records from `.devforgeai/requirements.yaml` and the six context files, writes one `.devforgeai/stories/STORY-nnn.md` per unit of work, and writes one `.devforgeai/stories/sprint.yaml` naming the subset of those stories that fits the configured capacity, in dependency order. Each story carries acceptance criteria with `AC-nnn` ids, the `REQ-nnn` it implements, the `CON-nnn` and `AP-nnn` it is bound by, the `UI-nnn` it renders, the layer it lives in, the file paths the Build phase writes, and its dependencies on other stories. When a story renders a screen no `UI-nnn` covers, Plan calls the Design skill in spec mode and reads the returned specs back into the story.

Plan writes no code, no test, and no configuration outside `.devforgeai/stories/`. It edits neither `requirements.yaml` nor `.devforgeai/context/*.md` nor any `adr/ADR-nnn.md`: a defect found in one of those leaves as a SEND BACK citing IDs, per §1.6. It estimates in story points drawn from a closed set and in no other unit; it carries no calendar date, no hour figure, and no assignee. It advances a story's `status` from `draft` to `ready` and no further, because `devforgeai phase set` owns the transitions to `building`, `built`, and `released` (`specs/01-cli.md` §`phase set`). It runs no test, reads no coverage figure, and interprets no lint output: `devforgeai gate check --phase plan` decides whether the phase passed.

## Inputs

| Document | Path | IDs read | What Plan reads |
|---|---|---|---|
| requirements | `.devforgeai/requirements.yaml` | `IDEA-nnn` (top-level `id`), `EPIC-nnn`, `REQ-nnn`, `PERSONA-nnn` | `epics[]` entry whose `id` equals `$1`, with `title`, `scope`, `out_of_scope`, `success_metric`, `requirements`; each `requirements[]` record it names, with `actor`, `statement`, `rationale`, `acceptance_signal`, `priority`, `status`, `open_questions`, `traces_to`; `personas[]` `name` for the `## Story` line |
| context · source tree | `.devforgeai/context/source-tree.md` | none | `## Layers` second table, column `Layer`, the closed set a story's `## Layer` line is drawn from; `## File placement rules`; `## Naming conventions`; `## Generated and excluded paths` |
| context · architecture constraints | `.devforgeai/context/architecture-constraints.md` | `CON-nnn` | `## Constraint index` rows with `Status` of `active`; `## Layer dependency rules` rows, columns `Layer`, `Depends on`, `Does not depend on`, `Constraint` |
| context · anti-patterns | `.devforgeai/context/anti-patterns.md` | `AP-nnn` | `## Anti-pattern index` rows, columns `AP`, `Severity`, `Scope` |
| context · coding standards | `.devforgeai/context/coding-standards.md` | none | `## Testing standards` rows, which fix the test-file paths a story declares |
| context · tech stack | `.devforgeai/context/tech-stack.md` | none | `## Excluded technologies`, which keeps an excluded name out of an AC |
| context · dependencies | `.devforgeai/context/dependencies.md` | none | `## Forbidden dependencies`, which keeps a forbidden name out of an AC |
| UI spec | `.devforgeai/ui-specs/UI-nnn.md` | `UI-nnn` | `## Requirements`, `## States`, `## Breakpoints`, `## Out of scope` |
| config | `.devforgeai/config.toml` | none | `[plan].sprint_capacity_points`, `[plan].story_points`, `[[layer]].name` |
| state | `.devforgeai/state.toml` | none | `current_phase`, `[active].constitute`, `[active].plan` |
| build report | `.devforgeai/reports/STORY-nnn-build.yaml` | `AC-nnn`, `STORY-nnn` | `findings[]`, remedy runs only |
| QA report | `.devforgeai/reports/STORY-nnn-qa.yaml` | `AC-nnn`, `FIND-nnn`, `STORY-nnn` | `findings[]`, remedy runs only |

The epic list lives in one `requirements.yaml` for the whole project (`specs/03-discover.md` §Outputs). `/plan` selects one `epics[]` entry from it and leaves the rest of the file unread.

## Outputs

Two document types. Both live under `.devforgeai/stories/`.

### 1. `.devforgeai/stories/STORY-nnn.md`

Frontmatter, in this order, with no other top-level key:

```yaml
---
schema: devforgeai/story/1
id: STORY-101
phase: plan
status: draft
produced_by: planning-work
consumes: [REQ-007, REQ-011, CON-003, AP-002, UI-004]
open_questions: []
---
```

| Key | Type | Required | Default | Constraint |
|---|---|---|---|---|
| `schema` | string | yes | none | constant `devforgeai/story/1` |
| `id` | string | yes | none | `^STORY-\d{3}$`, allocated by `devforgeai doc validate --allocate STORY` |
| `phase` | string | yes | none | constant `plan` |
| `status` | string | yes | `draft` | enum below |
| `produced_by` | string | yes | none | constant `planning-work` |
| `consumes` | list of string | yes | none | each matches `^(REQ\|CON\|AP\|UI)-\d{3}$`; at least one `REQ-nnn`; every `REQ-nnn` belongs to the epic in `sprint.yaml` `epic` |
| `open_questions` | list of string | yes | `[]` | each one sentence, 1 to 200 characters |

`status` enum, in progression order: `draft`, `ready`, `building`, `built`, `released`. Plan writes `draft` at workflow step 8 and `ready` at step 12. `devforgeai phase set` writes `building`, `built`, and `released` (`specs/01-cli.md` §`phase set`; `phase set release` writes `built` to `released`). The value `ready` is absent from `phase set`'s transition table, so Plan owns it.

Sections, in this order, with these headings verbatim. `devforgeai doc validate` compares the H2 list against the template in `## Templates`; `devforgeai story validate` locates content by the strings `## Requirements`, `## Acceptance Criteria`, `## Interface`, `## Files`, and `## Dependencies`.

| # | Heading | Content |
|---|---|---|
| 1 | `## Story` | Exactly three lines: `As a <personas[].name>`, `I want <one clause>`, `So that <one clause>`. |
| 2 | `## Requirements` | Table, columns `REQ \| Statement \| Covered by`. One row per `REQ-nnn` in `consumes`, 1 to 4 rows. `Statement` is the `requirements[].statement` value copied byte for byte. `Covered by` is a space-separated list of `AC-nnn` ids from section 3, 1 to 8 entries. |
| 3 | `## Acceptance Criteria` | Unnumbered list, 1 to 12 items, each `- AC-nnn: Given <state> When <action> Then <measurable outcome>.` Every `AC-nnn` appears in exactly one `Covered by` cell of section 2. |
| 4 | `## Constraints` | Table, columns `CON \| Statement \| Binds`. One row per `CON-nnn` in `consumes`, 0 to 8 rows. `Statement` is the `## Constraint index` row's `Title` and the `## Constraints` block's `statement` value joined by a colon. `Binds` names a `Path` value from section 8 or the `## Layer` value from section 7. A story bound by no constraint carries the single line `none` and no table. |
| 5 | `## Anti-patterns` | Table, columns `AP \| Severity \| Scope`. One row per `AP-nnn` in `consumes` whose `Scope` glob matches a `Path` value in section 8, 0 to 8 rows. `Severity` is the `## Anti-pattern index` value, one of `blocker`, `high`, `medium`, `low`. A story matching no anti-pattern carries the single line `none` and no table. |
| 6 | `## Interface` | Table, columns `UI \| Screen \| States covered`. One row per `UI-nnn` in `consumes`, 0 to 4 rows. `Screen` is the `UI-nnn` document's `## Purpose` first sentence. `States covered` is a space-separated list drawn from that document's `## States` `State` column. A story that renders no screen carries the single line `none` and no table. |
| 7 | `## Layer` | One line, a `Layer` value from `.devforgeai/context/source-tree.md` `## Layers` second table. |
| 8 | `## Files` | Table, columns `Path \| Kind \| Layer`. 1 to 20 rows. `Path` is repo-relative and matches a `Path pattern` glob from `## File placement rules`. `Kind` is one of `source`, `test`, `config`, `migration`, `asset`. `Layer` is a `Layer` value from `## Layers`, equal to section 7 for every row of `Kind` `source`. |
| 9 | `## Dependencies` | Unnumbered list, 0 to 6 items, each `- STORY-nnn: <one sentence naming what this story takes from it>`. A story with no dependency carries the single line `none`. |
| 10 | `## Out of scope` | Unnumbered lines, 0 to 8, each one sentence naming a behaviour this story does not carry and, when one exists, the `STORY-nnn` that does. |

Every `AC-nnn` is testable by three properties, each of which `devforgeai story validate` or `story-invest-auditor` decides: the text matches `Given .+ When .+ Then .+`, the `Then` clause names one outcome a test reads (a value, a status, a count, a stored row, an emitted event, or a rendered region), and the id appears in exactly one `Covered by` cell, so the criterion references at most one requirement.

### 2. `.devforgeai/stories/sprint.yaml`

One file per project, rewritten by each `/plan` run. Its `id` is allocated once per epic at workflow step 4 and reused by every later run for the same epic; a run for a different epic allocates the next id. Top-level keys, in this order, with no other top-level key:

```yaml
schema: devforgeai/sprint/1
id: SPRINT-001
phase: plan
status: active
produced_by: planning-work
consumes: [EPIC-001]
open_questions: []
epic: EPIC-001
capacity:
  points_max: 20
  points_planned: 17
  source: default
stories:
  - id: STORY-101
    status: ready
    order: 1
    points: 3
deferred:
  - id: STORY-109
    reason: capacity
```

| Key | Type | Required | Default | Constraint |
|---|---|---|---|---|
| `schema` | string | yes | none | constant `devforgeai/sprint/1` |
| `id` | string | yes | none | `^SPRINT-\d{3}$`, allocated by `devforgeai doc validate --allocate SPRINT` |
| `phase` | string | yes | none | constant `plan` |
| `status` | string | yes | `planned` | enum `planned`, `active`, `closed` |
| `produced_by` | string | yes | none | constant `planning-work` |
| `consumes` | list of string | yes | none | exactly one entry, the `EPIC-nnn` this sprint plans |
| `open_questions` | list of string | yes | `[]` | each one sentence, 1 to 200 characters |
| `epic` | string | yes | none | `^EPIC-\d{3}$`, equal to `consumes[0]` |
| `capacity` | object | yes | none | schema below |
| `stories` | list of object | yes | none | length `>= 1`; entry schema below; `id` precedes `status` in every entry, which the `pre-push` hook relies on (`specs/01-cli.md` §Decisions 24) |
| `deferred` | list of object | yes | `[]` | entry schema below |

`capacity`:

| Field | Type | Required | Default | Constraint |
|---|---|---|---|---|
| `points_max` | integer | yes | `20` | `>= 1`; the value of `config.toml` `[plan].sprint_capacity_points` when that key is present |
| `points_planned` | integer | yes | none | the sum of `stories[].points` |
| `source` | string | yes | none | enum `config` (the key was present), `default` (the key was absent and 20 was used) |

`stories[]`:

| Field | Type | Required | Default | Constraint |
|---|---|---|---|---|
| `id` | string | yes | none | `^STORY-\d{3}$` resolving to a `stories/STORY-nnn.md` file; first key in the entry |
| `status` | string | yes | none | a member of the story `status` enum; second key in the entry |
| `order` | integer | yes | none | `1` to `len(stories)`, unique, dense; a topological order of the dependency graph |
| `points` | integer | yes | none | a member of `config.toml` `[plan].story_points`, default set `[1, 2, 3, 5, 8]` |

`deferred[]`:

| Field | Type | Required | Default | Constraint |
|---|---|---|---|---|
| `id` | string | yes | none | `^STORY-\d{3}$` resolving to a `stories/STORY-nnn.md` file, absent from `stories[]` |
| `reason` | string | yes | none | enum `capacity` (adding it exceeds `points_max`), `dependency` (a story it depends on is deferred) |

The union of `stories[].id` and `deferred[].id` is every story written for the epic, so `devforgeai story validate --scope sprint` reaches all of them.

### Capacity rule

`points_max` is `config.toml` `[plan].sprint_capacity_points`, and `20` when that key is absent; `capacity.source` records which of the two applied. `sprint-sequencer` walks the topological order and appends a story while `points_planned + points <= points_max`, so a story landing exactly on `points_max` enters the sprint. A story that would take the sum past `points_max` is deferred with `reason: capacity`, and so is every story downstream of it in the dependency graph, with `reason: dependency`. A story whose own `points` alone exceed `points_max` enters the sprint as the sole entry of `stories[]`, `points_planned` equals its `points`, and every other story is deferred, because an empty `stories[]` leaves Build with no subject and fails the `plan-docs` check.

## Workflow

**1. Establish the run — model, CLI.** Input: `$ARGUMENTS`, the stdout of the three preamble lines, `.devforgeai/state.toml`. The run kind is `remedy` when `$ARGUMENTS` contains `--remedy`, `resume` when it contains `--resume`, and `full` otherwise. The epic is `$1` when `$1` has the `EPIC` prefix, and the `epic` key of `.devforgeai/stories/sprint.yaml` when `$1` has the `SPRINT` prefix. Output: a run kind and an `EPIC-nnn`. Failure path: `$1` carries a prefix that is neither `EPIC` nor `SPRINT`; the run stops with one `Blocked` line naming the two accepted prefixes.

**2. Read the epic — model.** Input: the `requirements.yaml` text on the preamble's stdout, the `EPIC-nnn` from step 1. Output: the `epics[]` entry and the ordered list of `requirements[]` records whose `id` it names and whose `status` is `accepted`. A record with `status` of `withdrawn` or a `priority` of `wont` is dropped here and named in step 8's `## Out of scope`. Failure path: the epic id resolves to no `epics[]` entry; the run stops with one `Blocked` line naming the id and the epic ids the file does hold.

**3. Read the context set — model.** Input: the six context files on the preamble's stdout. Output: the layer name set from `source-tree.md` `## Layers`, the active `CON-nnn` rows from `architecture-constraints.md` `## Constraint index`, the `## Layer dependency rules` rows, the `AP-nnn` rows from `anti-patterns.md` `## Anti-pattern index`, the test-path rules from `coding-standards.md` `## Testing standards`. Failure path: `## Layers` holds no data row; the run stops with one `Blocked` line naming `source-tree.md ## Layers`.

**4. Open the sprint — model, CLI.** Input: the `EPIC-nnn` from step 1, `.devforgeai/stories/sprint.yaml` when it exists. The run reuses that file's `id` when its `epic` key equals the run's epic, and otherwise runs `devforgeai doc validate --allocate SPRINT`, which returns the next free id because the prior sprint's id survives in the frontmatter `id` of `.devforgeai/reports/<SPRINT-nnn>-plan.yaml`, a file the ID index walks and no subcommand deletes. Actor: the model then runs `devforgeai phase set plan --id <SPRINT-nnn> --epic <EPIC-nnn>`. `--epic` carries the epic step 1 settled; the call needs it whenever `sprint.yaml` is absent, which is every full run, because the file is written at step 12, and every resume that follows a send-back, where step 12 did not run. With the file present the flag repeats the file's own `epic` key; absent with no file on disk is `DFA-E011`, and a malformed epic or one differing from the file's key is `DFA-E013`. Output: a `SPRINT-nnn`, `state.toml` `current_phase` of `plan`, and `[active].plan` of that id, which is the id `report ingest` resolves its target report from at step 9. Failure path: `phase set` exits 1 on `DFA-E320`, meaning the constitute gate is not PASS; the run stops and the Stop hook prints the gate result.

**5. Decompose — `story-decomposer`, one invocation.** Input: the epic entry, the requirement records, the layer set, the constraint rows, the anti-pattern rows, the `traces_to` list of each requirement. Output: the subagent's JSON, one draft per story with `req_ids`, `story_lines`, `acceptance_criteria`, `layer`, `con_ids`, `ap_ids`, `ui_ids`, `depends_on`, `out_of_scope`. Failure path: the JSON does not parse against the schema in `## Subagents`; the subagent is invoked once more with the parse error appended, and a second failure leaves the gate metric absent, which `gate check` reports as exit 1.

**6. Specify screens — Design skill, spec mode, zero or more invocations.** Input: each draft whose `layer` is `interface` and for which no `REQ-nnn` in its `req_ids` carries a non-empty `traces_to`. Actor: the model, through the Skill tool, as `/design STORY-nnn --spec` once per such draft, after step 8 has allocated that draft's `STORY-nnn`. Output: the `UI-nnn` ids Design allocated, read back through `devforgeai doc load ui-spec <UI-nnn>`: `## Requirements` fixes the `REQ-nnn` set the screen realizes, `## States` fixes the state list section 6 records and the size the story carries, `## Breakpoints` fixes the four layouts one `AC-nnn` covers, `## Out of scope` fixes what section 10 records. Failure path: Design returns a SEND BACK to Discover; Plan writes the stories it holds with `status: draft`, writes no `sprint.yaml`, and the handoff carries Design's `Next` line.

**7. Plan the file sets — `story-file-set-planner`, one invocation.** Input: every draft from step 5, the `## File placement rules` rows, the `## Naming conventions` rows, the `## Generated and excluded paths` globs, the `## Testing standards` rows. Output: one `files[]` table per draft, each row a `path`, a `kind`, and a `layer`, with no path appearing in two drafts. Failure path: the JSON does not parse; the retry rule of step 5 applies.

**8. Write the stories — model.** Input: the drafts, the file sets, the `UI-nnn` ids from step 6. Actor: the model calls `devforgeai doc validate --allocate STORY` once per draft and `devforgeai doc validate --allocate AC` once per criterion, then writes `.devforgeai/stories/STORY-nnn.md` from `templates/story.md` with `status: draft`. Output: one file per draft. Failure path: `--allocate` exits 1 on `DFA-E215`; the run stops with one `Blocked` line naming the exhausted prefix.

**9. Audit — `story-invest-auditor`, one invocation.** Input: every written story path, the `SPRINT-nnn` from step 4, the epic entry, the requirement records, the active `CON-nnn` rows. Output: the verifier JSON of `specs/01-cli.md` §Subagents, with `id` set to the `SPRINT-nnn`, `passed`, `total`, and `findings[]`. The `SubagentStop` hook runs `devforgeai report ingest story-invest-auditor -`, which writes `verifiers.story_invest` into `.devforgeai/reports/SPRINT-nnn-plan.yaml`, creating that file from the report skeleton when absent. Failure path: the JSON does not parse; `report ingest` writes the block with `status: unparsed`, and the retry rule of step 5 applies.

**10. Rewrite on judgment findings — model.** Input: the `findings[]` entries of step 9 whose `severity` is `warn`. Output: the edited `STORY-nnn.md` files, then one further invocation of `story-invest-auditor` over the edited set. The loop runs once; a `warn` finding surviving the second pass stays in the report and annotates the gate without changing its result. Failure path: none; a `warn` finding sets no gate.

**11. Sequence and fill — `sprint-sequencer`, one invocation.** Input: every story id, its `points` estimate from step 5, its `## Dependencies` list, `points_max` from `config.toml` `[plan].sprint_capacity_points` or the default `20`. Output: the `stories[]` list in topological order with `order` and `points`, the `deferred[]` list, and the `capacity` object. Failure path: the dependency graph holds a cycle; the subagent returns `cycle` with the path, and the run stops with one `Blocked` line naming the cycle, which `devforgeai story validate` also reports as `DFA-E232`.

**12. Write the sprint — model.** Input: the step 11 output and the `SPRINT-nnn` from step 4. Actor: the model writes `.devforgeai/stories/sprint.yaml` from `templates/sprint.yaml` with that `id` and `status: active`, then edits each story named in `stories[]` and `deferred[]` to `status: ready`. Output: `sprint.yaml` and the updated stories. Failure path: the `PostToolUse` hook returns the `doc validate` diagnostic in `hookSpecificOutput.additionalContext`; the model rewrites the key it names.

**13. Close — CLI, Stop hook.** Input: `state.toml`. Actor: the Stop hook runs `devforgeai gate check --phase plan`, then `devforgeai handoff`. Output: `.devforgeai/reports/SPRINT-nnn-plan.yaml` and the §6 block. Failure path: a FAIL exits 2 with `decision: "block"` and the failing checks in `reason`, under a budget of three blocks per `session_id`; at the last block of that budget the hook exits 0 and the FAIL block renders in `systemMessage`, per §7.

### Remedy workflow, `/plan <EPIC-nnn> --remedy AC-nnn,...`

**R1. Locate the criteria — model, CLI.** Input: the id list after `--remedy`, `.devforgeai/reports/<STORY-nnn>-build.yaml` or `<STORY-nnn>-qa.yaml` through `devforgeai report show <STORY-nnn> build` and `devforgeai report show <STORY-nnn> verify`. Output: one `(STORY-nnn, AC-nnn, finding summary)` triple per cited id. Failure path: a cited `AC-nnn` appears in no story file; the run stops with one `Blocked` line naming the id.

**R2. Triage — `spec-gap-triager`, one invocation.** Input: the triples from R1, the `REQ-nnn` each criterion's row covers, the active `CON-nnn` rows, the `UI-nnn` the story cites. Output: one `disposition` per triple from the closed enum `rewrite_ac`, `send_back_discover`, `send_back_constitute`, `send_to_design`. Failure path: the JSON does not parse; the retry rule of step 5 applies.

**R3. Act — model, Design skill.** Input: the dispositions. `rewrite_ac` edits the criterion line in place in its `STORY-nnn.md` and leaves every other byte of the file unchanged. `send_to_design` calls `/design UI-nnn --remedy AC-nnn,...` and rewrites section 6 from the returned spec. `send_back_discover` and `send_back_constitute` write no file. Output: the edited stories. Failure path: an edit that breaks the AC grammar is reported by the PostToolUse `doc validate` as `DFA-E230`; the model rewrites the line.

**R4. Reset and close — model, CLI.** Input: the stories edited at R3. Actor: the model sets each edited story's frontmatter `status` to `ready` and the `status` of that story's entry in `sprint.yaml` `stories[]` to `ready`, changes no other key of `sprint.yaml`, and runs `devforgeai phase set plan --id <SPRINT-nnn> --epic <EPIC-nnn>` with the `id` and the `epic` key the file on disk carries. A remedy run reaches R4 with `sprint.yaml` on disk, so both values are read from it. Output: the Stop hook block, whose `Next` is `/build <STORY-nnn> --resume` when every disposition was `rewrite_ac` or `send_to_design`, and the send-back form of `## Send-back` otherwise.

### Resume workflow, `/plan <EPIC-nnn> --resume`

Re-enters at step 2 and re-reads `requirements.yaml` and the six context files from the preamble's stdout, so a repaired `REQ-nnn` or a new `CON-nnn` is the text the run works from. Step 4 reuses the `SPRINT-nnn` that `state.toml` `[active].plan` already holds and allocates none. Every `STORY-nnn.md` already on disk with `status: draft` is kept and re-audited at step 9; the ids it holds are not re-allocated. `sprint.yaml`, absent after a send-back, is written at step 12.

## Subagents

### story-decomposer

- **name**: `story-decomposer`
- **derives_from**: `C:\Users\bryan\.claude\agents\story-requirements-analyst.md`
- **purpose**: Split one epic's accepted requirements into story drafts, each with acceptance criteria in Given/When/Then form, a layer, a constraint set, and a dependency list.
- **tools**: Read, Grep, Glob
- **model**: `opus` — the split decides how much a single Build run carries, and a wrong split propagates through every later phase.
- **input**: the `epics[]` entry (`EPIC-nnn`, `title`, `scope`, `out_of_scope`, `success_metric`); the `requirements[]` records it names (`REQ-nnn`, `actor`, `statement`, `rationale`, `acceptance_signal`, `priority`, `traces_to`); the `Layer` value set from `source-tree.md` `## Layers`; the active rows of `architecture-constraints.md` `## Constraint index` (`CON-nnn`, `Kind`, `Title`); the `## Layer dependency rules` rows; the rows of `anti-patterns.md` `## Anti-pattern index` (`AP-nnn`, `Severity`, `Scope`); the `[plan].story_points` set.
- **output**:

```json
{
  "schema": "devforgeai/story-decomposer/1",
  "epic": "EPIC-001",
  "drafts": [
    {
      "key": "d1",
      "req_ids": ["REQ-007"],
      "story_lines": { "as_a": "Shopper", "i_want": "…", "so_that": "…" },
      "acceptance_criteria": [
        { "key": "a1", "req_id": "REQ-007", "given": "…", "when": "…", "then": "…" }
      ],
      "layer": "application",
      "con_ids": ["CON-003"],
      "ap_ids": ["AP-002"],
      "ui_ids": [],
      "needs_screen": false,
      "depends_on": ["d0"],
      "points": 3,
      "out_of_scope": ["…"]
    }
  ],
  "uncovered_reqs": ["REQ-012"],
  "notes": []
}
```

  `key` is a run-local handle the skill maps to an allocated `STORY-nnn` at step 7; `depends_on` holds `key` values, not ids, because the ids are unallocated when the subagent runs. `points` is a member of `[plan].story_points`. `needs_screen` is `true` when `layer` is `interface` and every `req_ids` entry carries an empty `traces_to`.
- **invoked_at**: workflow step 5, alone.
- **registered_verifier**: no.

### story-file-set-planner

- **name**: `story-file-set-planner`
- **derives_from**: `C:\Users\bryan\.claude\agents\api-designer.md` — replaced, not adapted. The existing agent designs REST, GraphQL, and gRPC contracts and writes structured YAML API components; that output is subsumed by the story's `## Files` table and by the `Then` clause of an acceptance criterion, and Build owns the implementation. This subagent keeps the "fix the contract before code is written" idea and applies it to file paths.
- **purpose**: Assign each story draft a disjoint set of repo-relative paths, drawn from the source tree's placement rules, that the Build phase writes.
- **tools**: Read, Grep, Glob
- **model**: `sonnet` — the task is pattern application against two tables, and the overlap result is checked mechanically by `devforgeai story validate`.
- **input**: the `drafts[]` array from `story-decomposer`; `source-tree.md` `## Roots`, `## Directory map`, `## File placement rules` (`Artifact kind`, `Path pattern`, `Example`), `## Naming conventions`, `## Generated and excluded paths`; `coding-standards.md` `## Testing standards` rows (`Rule`, `Scope`); the existing repo tree under the `source.root` and `test.root` values.
- **output**:

```json
{
  "schema": "devforgeai/story-file-set-planner/1",
  "sets": [
    { "key": "d1",
      "files": [ { "path": "src/application/checkout/place_order.ext", "kind": "source", "layer": "application" },
                 { "path": "tests/application/checkout/place_order_test.ext", "kind": "test", "layer": "application" } ] }
  ],
  "overlaps": [ { "path": "src/shared/clock.ext", "keys": ["d1", "d4"] } ],
  "unplaceable": [ { "key": "d7", "reason": "no File placement rules row matches an interface asset" } ]
}
```

  `kind` is one of `source`, `test`, `config`, `migration`, `asset`. A non-empty `overlaps` or `unplaceable` list sends the draft set back through one further invocation with those entries appended to the prompt.
- **invoked_at**: workflow step 7, alone, after step 5.
- **registered_verifier**: no.

### story-invest-auditor

- **name**: `story-invest-auditor`
- **derives_from**: `C:\Users\bryan\.claude\agents\requirements-analyst.md` — adapted. The existing agent both writes stories and judges them, and holds Write and Edit. This one judges only and is read-only, because `story-decomposer` writes and §1.1 keeps the mechanical half of INVEST in `devforgeai story validate`. The existing agent's INVEST list is narrowed to the three judgments no parser makes: independence, size, and the wording of a `Then` clause.
- **purpose**: Judge each written story on independence, size, and the measurability of every `Then` clause, and report a requirement that admits two readings or a constraint the context set does not state.
- **tools**: Read, Grep, Glob
- **model**: `opus` — independence and measurability are judgments about meaning, and the finding routes the phase to a send-back.
- **input**: the path of every `.devforgeai/stories/STORY-nnn.md` written this run; the `epics[]` entry; the `requirements[]` records (`REQ-nnn`, `statement`, `acceptance_signal`); the active rows of `architecture-constraints.md` `## Constraint index` and the `## Layer dependency rules` rows.
- **output**: the verifier contract of `specs/01-cli.md` §Subagents, `devforgeai/verifier/1`:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "story-invest-auditor",
  "id": "SPRINT-001",
  "passed": 6,
  "total": 8,
  "unit": "stories",
  "findings": [
    { "id": "REQ-011", "severity": "block",
      "summary": "\"recent orders\" fixes no window and no count",
      "evidence": "requirements.yaml REQ-011 statement; STORY-104 AC-019 and AC-020 read it two ways" },
    { "id": "STORY-107", "severity": "warn",
      "summary": "8 points and two layers in one story",
      "evidence": "STORY-107 ## Files rows 1-4 layer domain, rows 5-9 layer interface" }
  ]
}
```

  `severity` is the closed enum `block`, `warn`, `info` that `specs/01-cli.md` fixes. A finding's `id` is the upstream id it cites — a `REQ-nnn` for an ambiguous requirement, a `CON-nnn` for a missing or contradictory constraint — so `devforgeai handoff` renders the `Found` line and composes the `--remedy` list from `findings[].id` without a further field. A judgment about a story carries that story's `STORY-nnn` as its `id` and `severity` of `warn`. `passed` counts stories against which no `block` finding stands; `total` is the story count.
- **invoked_at**: workflow step 9, and once more at step 10 over the edited set.
- **registered_verifier**: yes. The `config.toml` entry is in `## CLI calls`.

### sprint-sequencer

- **name**: `sprint-sequencer`
- **derives_from**: `C:\Users\bryan\.claude\agents\sprint-planner.md` — adapted. The existing agent writes the sprint file, edits story frontmatter, and appends workflow history; those writes move to workflow step 11, because a subagent that writes documents defeats the PostToolUse validation the skill's own writes pass through. Its capacity arithmetic and story selection are kept, its point thresholds move to `config.toml` `[plan].sprint_capacity_points`, and its dependency handling absorbs the topological ordering that `dependency-graph-analyzer` performed.
- **purpose**: Order the epic's stories by dependency and fill one sprint to the configured point capacity.
- **tools**: Read, Glob, Grep
- **model**: `sonnet` — the work is a topological sort and a running sum against a stated bound.
- **input**: for each story, its `STORY-nnn`, its `points`, and its `## Dependencies` id list; `points_max` and whether it came from `config.toml` or the default.
- **output**:

```json
{
  "schema": "devforgeai/sprint-sequencer/1",
  "epic": "EPIC-001",
  "capacity": { "points_max": 20, "points_planned": 17, "source": "default" },
  "stories": [ { "id": "STORY-101", "order": 1, "points": 3 } ],
  "deferred": [ { "id": "STORY-109", "reason": "capacity" } ],
  "longest_chain": ["STORY-101", "STORY-103", "STORY-107"],
  "cycle": []
}
```

  `reason` is the closed enum `capacity`, `dependency`. `longest_chain` is the longest dependency chain in the sprint, reported so the skill can report it and used by no check. A non-empty `cycle` holds the id path of one cycle and stops workflow step 10.
- **invoked_at**: workflow step 11, alone.
- **registered_verifier**: no.

### spec-gap-triager

- **name**: `spec-gap-triager`
- **derives_from**: `new`
- **purpose**: Decide, for one acceptance criterion a downstream phase sent back, whether the criterion is rewritten here, or the defect belongs to Discover, Constitute, or Design.
- **tools**: Read, Grep, Glob
- **model**: `opus` — the decision routes the send-back and a wrong route costs a full phase.
- **input**: one triple per cited id — `story_id`, `ac_id`, and the finding `summary` and `evidence` from the build or QA report; the `REQ-nnn` whose `Covered by` cell holds the criterion, with its `statement` and `acceptance_signal`; the active `CON-nnn` rows whose `Binds` value matches the story's `## Layer` or a `Path` in its `## Files`; the `UI-nnn` ids in the story's `## Interface`.
- **output**:

```json
{
  "schema": "devforgeai/spec-gap-triager/1",
  "decisions": [
    { "story_id": "STORY-104", "ac_id": "AC-019",
      "disposition": "rewrite_ac",
      "cited_id": "AC-019",
      "rationale": "The Then clause names a state and no value a test reads.",
      "replacement": "Given a shopper with 3 saved orders When the list loads Then the response holds 3 rows ordered by placed_at descending." }
  ]
}
```

  `disposition` is the closed enum `rewrite_ac`, `send_back_discover`, `send_back_constitute`, `send_to_design`. `cited_id` is the `AC-nnn` for `rewrite_ac`, the `REQ-nnn` for `send_back_discover`, the `CON-nnn` for `send_back_constitute`, and the `UI-nnn` for `send_to_design`. `replacement` is the rewritten criterion text for `rewrite_ac` and `""` for the other three.
- **invoked_at**: remedy workflow step R2, alone. It runs in no `full` and no `resume` run.
- **registered_verifier**: no. Its output routes the run; `story-invest-auditor` supplies the report block.

### Existing agents this skill does not use

| Agent | Disposition | Reason |
|---|---|---|
| `dependency-graph-analyzer.md` | retired | Cycle detection is `devforgeai story validate` check 5, error `DFA-E232`. Transitive resolution and ordering are `sprint-sequencer`. Its status validation reads a story status vocabulary (`Dev Complete`, `QA Approved`) that this framework replaces with the six-value enum `phase set` owns. |
| `epic-coverage-result-interpreter.md` | retired | Coverage counting is `devforgeai story validate` check `DFA-E235`. Its four display templates are `devforgeai handoff`, which §6 makes the one block a phase prints. |
| `context-preservation-validator.md` | not invoked here (Discover adapts it) | Provenance is the `consumes` key plus the `doc validate` ID index and the `plan-ids` `ids_resolve` check, which resolve every `REQ-nnn`, `CON-nnn`, `AP-nnn`, and `UI-nnn` a story names. Its `<provenance>` XML section has no place in the fixed heading list. |
| `ui-spec-formatter.md` | not invoked here (Design replaces it) | `UI-nnn.md` is Design's document with nine fixed headings; Plan reads it through `devforgeai doc load ui-spec` and formats nothing. |
| `api-designer.md` | replaced by `story-file-set-planner` | Its structured-YAML API component output is carried by the story's `## Files` table and the `Then` clause of an acceptance criterion. `specs/04-constitute.md` §Decisions 17 named Plan its owner; Plan's answer is that no skill invokes it. Recorded for `specs/11-subagent-catalog.md`. |

## Command

The entry point is the skill itself: `skills/planning-work/SKILL.md`, installed to `.claude/skills/plan/`, whose frontmatter `name` is the slash command. There is no command file. This is the frontmatter and the preamble, verbatim, with three preamble lines.

```markdown
---
name: plan
description: Phase 3 of DevForgeAI, run by /plan. Turns one epic's accepted requirements into one .devforgeai/stories/STORY-nnn.md per unit of work and one stories/sprint.yaml naming the subset that fits the configured point capacity, in dependency order. Each story carries AC-nnn acceptance criteria in Given/When/Then form, the REQ-nnn it implements, the CON-nnn and AP-nnn it is bound by, the UI-nnn it renders, its layer, the file paths Build writes, and its dependencies on other stories. Reach for it whenever /plan is typed, whenever an epic is being decomposed into stories, whenever an acceptance criterion, story point, sprint capacity, dependency order, or declared file set is being written for this framework, and whenever STORY-nnn, SPRINT-nnn, AC-nnn, stories/sprint.yaml, or a Plan send-back to Discover or Constitute appears in a story, a report, or a handoff.
argument-hint: <EPIC-nnn | SPRINT-nnn> [--remedy AC-nnn,AC-nnn] [--resume]
allowed-tools: Bash(devforgeai:*), PowerShell(devforgeai:*), Read, Write, Edit, Glob, Grep, Agent, AskUserQuestion, Skill
disable-model-invocation: true
---

!`devforgeai gate require plan $ARGUMENTS[0]`
!`devforgeai doc load requirements $ARGUMENTS[0]`
!`devforgeai doc load context all`
```

The `gate require` line leads, so a failing predecessor gate aborts the invocation before any document loads.

## CLI calls

| Call, exact arguments | Caller | When | Exit handling |
|---|---|---|---|
| `devforgeai gate require plan $ARGUMENTS[0]` | command `!` preamble | steps 1, R1 | 0 continues; 1 prints the missing gate and stops the command body |
| `devforgeai doc load requirements $ARGUMENTS[0]` | command `!` preamble | steps 1, 2, R1 | 0 prints the file; 1 on `DFA-E200` stops the run |
| `devforgeai doc load context all` | command `!` preamble | step 3 | 0 prints the six files; 1 on `DFA-E200` stops the run |
| `devforgeai doc load ui-spec <UI-nnn>` | model, Bash | step 6, once per returned id | 0 prints the spec; 1 on `DFA-E200` means Design wrote no file and step 5 stops |
| `devforgeai doc validate --allocate STORY` | model, Bash | step 8, once per draft | 0 prints `STORY-nnn`; 1 on `DFA-E215` stops the run |
| `devforgeai doc validate --allocate AC` | model, Bash | step 8, once per criterion | 0 prints `AC-nnn`; 1 on `DFA-E215` stops the run |
| `devforgeai doc validate --allocate SPRINT` | model, Bash | step 4, when `sprint.yaml` is absent or names another epic | 0 prints `SPRINT-nnn`; 1 on `DFA-E215` stops the run |
| `devforgeai report ingest story-invest-auditor -` | `SubagentStop` hook | steps 9, 10 | 0 in every case; an unparsed block carries `status: unparsed` |
| `devforgeai report show <STORY-nnn> build` | model, Bash | step R1 | 0 prints the report; 1 means no build report and R1 stops |
| `devforgeai report show <STORY-nnn> verify` | model, Bash | step R1 | 0 prints the report; 1 means no verify report and R1 stops |
| `devforgeai phase set plan --id <SPRINT-nnn> --epic <EPIC-nnn>` | model, Bash | steps 4, R4 | 0 continues; 1 on `DFA-E320` stops the run; 3 on `DFA-E011` when `--epic` is absent with no `sprint.yaml` on disk, and on `DFA-E013` when the epic is malformed or contradicts the file's `epic` key |
| `devforgeai story validate --scope sprint` | `Stop` hook, through `gate check` | step 13 | 0 passes the `plan-stories` check; 1 fails it |
| `devforgeai gate check --phase plan` | `Stop` hook | step 13 | 0 PASS, 1 FAIL, 2 SEND BACK |
| `devforgeai handoff` | `Stop` hook | step 13 | 0 in every case |
| `devforgeai doc validate <path>` | `PostToolUse` hook on `Write\|Edit\|NotebookEdit` under `.devforgeai/` | steps 8, 12, R3 | returns the diagnostic in `hookSpecificOutput.additionalContext`; the model rewrites the key it names |
| `devforgeai doc validate --producer-check <path>` | `PreToolUse` hook on Write and Edit under `.devforgeai/` | steps 8, 12, R3 | 1 maps to hook exit 2 and blocks a write to a document this phase does not produce |
| `devforgeai story files --check <path>` | `PreToolUse` hook on Write and Edit outside `.devforgeai/` while `current_phase` is `build` | Build phase | 1 on `DFA-E239` maps to hook exit 2 and blocks a write outside the active story's `## Files` set |

`story files --check` and the `AC` and `SPRINT` arguments to `doc validate --allocate` are proposed additions to conventions §4 and to `specs/01-cli.md`, recorded in `## Decisions` with their grammar. Every other call uses a §4 name.

The `config.toml` tables this phase adds:

```toml
[plan]
sprint_capacity_points = 20         # integer; default 20; the maximum sum of stories[].points in one sprint
story_points = [1, 2, 3, 5, 8]      # array[integer]; default as shown; the closed set stories[].points is drawn from

[[verifier]]
name = "story-invest-auditor"       # string; kebab-case subagent name
phase = "plan"                      # string; the report this verifier writes into
report_field = "verifiers.story_invest"   # string; dotted path inside the report YAML
unit = "stories"                    # string; the unit word on the handoff Verified line
required = true                     # bool; the plan gate's verifier_pass check names it
```

## Gate

The `.devforgeai/gates.toml` entry for this phase. The first four checks are `specs/01-cli.md` §Gate verbatim. The fifth block, `plan-invest`, is added by this spec and is proposed as an amendment to the default file in `## Decisions` item 7; its `kind` is the existing `verifier_pass` kind, reused by name.

```toml
[[gate]]
phase = "plan"
requires = "constitute"
on_fail = "fail"
send_back_to = "discover"
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

The gate's static `send_back_to` is `discover`, and Plan has two upstream targets. `gate check` resolves the report's `gate.send_back_to` from the prefixes of the blocking findings' `id` values through the fixed map `REQ` → `discover`, `CON` → `constitute`, `AP` → `constitute`, `UI` → `design`, and falls back to the gate's `send_back_to` when no blocking finding is present. Blocking findings of two prefixes produce `constitute`, because a missing constraint is repaired before the requirement it would have decided. This resolution rule is proposed in `## Decisions` item 8.

### The INVEST split

| Property | Decided by | Rule |
|---|---|---|
| Every requirement of the epic is covered by at least one story | `devforgeai story validate --scope sprint` | `DFA-E235` |
| Every acceptance criterion references a requirement that exists | `devforgeai story validate` | `DFA-E236` when the id appears in no `Covered by` cell, `DFA-E231` when the cell's `REQ-nnn` is absent from `requirements.yaml` |
| Every acceptance criterion references at most one requirement | `devforgeai story validate` | `DFA-E245` |
| Every acceptance criterion has a testable predicate | `devforgeai story validate` | `DFA-E230`, the `Given … When … Then …` grammar |
| No dependency cycle | `devforgeai story validate` | `DFA-E232` |
| Declared file sets do not overlap inside `stories[]` | `devforgeai story validate --scope sprint` | `DFA-E237` |
| Every `UI-nnn` cited exists | `devforgeai story validate` | `DFA-E238` |
| Every story listed in the sprint has a file | `devforgeai story validate --scope sprint` | `DFA-E233` |
| Every story has at least one criterion | `devforgeai story validate` | `DFA-E234` |
| Independence: the story delivers value with its dependencies met and nothing further | `story-invest-auditor` | `severity: warn`, cited by `STORY-nnn` |
| Size: the story fits one layer and one Build run at its point value | `story-invest-auditor` | `severity: warn`, cited by `STORY-nnn` |
| Testability of wording: the `Then` clause names one outcome a test reads | `story-invest-auditor` | `severity: warn`, cited by `STORY-nnn` |
| The requirement admits two incompatible readings | `story-invest-auditor` | `severity: block`, cited by `REQ-nnn` |
| The constraint set is silent or self-opposed on a case the story meets | `story-invest-auditor` | `severity: block`, cited by `CON-nnn` |

A `warn` finding is repaired inside the run at workflow step 10 and sets no gate. A `block` finding lowers `passed` below `total`, fails the `plan-invest` check, and produces the SEND BACK of the next section.

## Send-back

### To Discover, exit 2

| Condition | Detected by | IDs cited | Handoff `Next` |
|---|---|---|---|
| SB-1 · A requirement's `statement` admits two incompatible readings, each yielding a different `Then` clause | `story-invest-auditor` finding, `severity: block`, `id` prefix `REQ` | the `REQ-nnn` of each such finding | `/discover <IDEA-nnn> --remedy <REQ ids>` |
| SB-2 · A requirement's `acceptance_signal` names no outcome a test reads | `story-invest-auditor` finding, `severity: block`, `id` prefix `REQ` | the `REQ-nnn` of each such finding | `/discover <IDEA-nnn> --remedy <REQ ids>` |

`<IDEA-nnn>` is the top-level `id` of `.devforgeai/requirements.yaml`, which the preamble printed. It is not the `EPIC-nnn` the run took as `$1`: Discover re-opens the document by its own id and records the epic in `revision_log[].epic` (`specs/03-discover.md` §Command, entry point C). The return trip is `/plan <EPIC-nnn> --resume`.

### To Constitute, exit 2

| Condition | Detected by | IDs cited | Handoff `Next` |
|---|---|---|---|
| SB-3 · The constraint set decides no case the story meets, in a layer the `## Layer dependency rules` table covers | `story-invest-auditor` finding, `severity: block`, `id` prefix `CON` | the `CON-nnn` in the `Constraint` column of that layer's row | `/constitute <IDEA-nnn> --remedy <CON ids>` |
| SB-4 · Two active constraints bind one path set in opposed directions | `story-invest-auditor` finding, `severity: block`, `id` prefix `CON` | both `CON-nnn` | `/constitute <IDEA-nnn> --remedy <CON ids>` |

SB-3 cites an existing id because `.devforgeai/context/architecture-constraints.md` `## Layer dependency rules` carries one `CON-nnn` per layer row (`specs/04-constitute.md` §Templates), so the constraint that governs the layer and decides no case the story meets has a name. The return trip is `/plan <EPIC-nnn> --resume`.

On either target the stories written so far stay on disk with `status: draft`, `sprint.yaml` is not written, `requirements.yaml` and the six context files keep every byte (§1.6), and `state.toml` keeps `current_phase` of `plan`.

### To Design

Not a SEND BACK. A story citing a `UI-nnn` that `.devforgeai/ui-specs/` does not hold, or a `UI-nnn` whose `## States` list does not carry a state an `AC-nnn` asserts, is repaired by calling `/design UI-nnn --remedy AC-nnn,...` inside the run at step R3. Design is cross-cutting and holds no gate (`specs/08-design.md` §Gate), so the trip sets no phase and the `Then` line carries `/plan <EPIC-nnn> --resume`.

### Received

| From | Command | Effect |
|---|---|---|
| Build | `/plan <EPIC-nnn> --remedy AC-nnn,...` | The criterion is untestable as written. R2 triages it; `rewrite_ac` edits the one criterion line in its story and leaves every other byte of the file and all of `sprint.yaml` unchanged; the story returns to `status: ready`; `Next` is `/build <STORY-nnn> --resume` |
| Verify | `/plan <EPIC-nnn> --remedy AC-nnn,...` | A `FIND-nnn` names a gap between the criterion and what the story specifies. R2 triages it by the same four dispositions; `Next` is `/build <STORY-nnn> --resume` when the criterion was rewritten, and the send-back form above when the gap is upstream |
| Design | `/plan <EPIC-nnn> --resume` | Design's remedy run rewrote the cited `UI-nnn`; step 6 re-reads it and step 8 rewrites the story's `## Interface` |
| Discover | `/plan <EPIC-nnn> --resume` | A cited `REQ-nnn` was re-opened and its `statement` or `acceptance_signal` rewritten; the resume workflow re-enters at step 2 |
| Constitute | `/plan <EPIC-nnn> --resume` | A cited `CON-nnn` was superseded or a replacement allocated; the resume workflow re-enters at step 2. Constitute's SB-3 outcome, where the constraint stands, arrives on the same line and the run rewrites the stories that assumed otherwise |

## Integration

| Skill | Consumes (doc, IDs) | Produces for (doc, IDs) | Sends back to (condition) | Receives send-back from (condition) | Shared subagents | `state.toml` fields read / written |
|---|---|---|---|---|---|---|
| 0 Explore · `exploring-ideas` | none — Explore's `brief.md` and `decision.yaml` are consumed by Discover, which carries their content into `requirements.yaml`; Plan reads no `IDEA-nnn` or `FLOW-nnn` body | none — Explore runs before Discover and reads no story | none — §5 gives Plan's send-back targets as Discover and Constitute | none — §5 gives Explore's send-back column as `none` | none | none |
| 1 Discover · `discovering-requirements` | `.devforgeai/requirements.yaml`: top-level `id` (`IDEA-nnn`), `epics[]` (`EPIC-nnn`) with `title`, `scope`, `out_of_scope`, `success_metric`, `requirements`; `requirements[]` (`REQ-nnn`) with `actor`, `statement`, `rationale`, `acceptance_signal`, `priority`, `status`, `traces_to`; `personas[]` (`PERSONA-nnn`) `name` | `.devforgeai/stories/STORY-nnn.md` `consumes` and `## Requirements`, which trace each `REQ-nnn` to the `AC-nnn` that test it; Discover reads them through `report show <IDEA-nnn> plan` at its entry point C | yes: SB-1 a `REQ-nnn` `statement` admits two readings, SB-2 its `acceptance_signal` names no measurable outcome; leaves as `/discover <IDEA-nnn> --remedy REQ-nnn,...`, re-opened with `--from plan` | none — Discover runs before Plan and cites no `STORY-nnn` | none | reads `[active].plan` |
| 2 Constitute · `establishing-context` | `.devforgeai/context/source-tree.md` `## Layers`, `## File placement rules`, `## Naming conventions`, `## Generated and excluded paths`; `architecture-constraints.md` `## Constraint index` (`CON-nnn`) and `## Layer dependency rules`; `anti-patterns.md` `## Anti-pattern index` (`AP-nnn`); `coding-standards.md` `## Testing standards`; `tech-stack.md` `## Excluded technologies`; `dependencies.md` `## Forbidden dependencies` | `.devforgeai/stories/STORY-nnn.md` `## Constraints` and `## Anti-patterns`, which name every `CON-nnn` and `AP-nnn` a story is bound by; Constitute reads them in the Plan report at its Remedy workflow step R1 | yes: SB-3 the constraint set is silent on a case a story meets, SB-4 two active constraints bind one path set in opposed directions; leaves as `/constitute <IDEA-nnn> --remedy CON-nnn,...` | yes: `specs/04-constitute.md` SB-3, a constraint Plan sent back stands; arrives as `/plan <EPIC-nnn> --resume` and the run rewrites the stories that assumed the constraint would move | none | reads `[active].constitute` through `gate require plan`; writes `[active].plan` through `phase set` |
| 3 Plan · `planning-work` | self | self | self | self | self | reads `current_phase`, `[active].constitute`, `[active].plan`; `phase set plan` writes `current_phase` and `[active].plan`; `gate check --phase plan` writes `[last_gate]` |
| 4 Build · `implementing-stories` | none — Build produces `reports/STORY-nnn-build.yaml`, which Plan reads on a remedy run only, through `report show <STORY-nnn> build` | `.devforgeai/stories/STORY-nnn.md`: `## Acceptance Criteria` (`AC-nnn`) is the test list, `## Files` is the path set the `PreToolUse` `story files --check` hook allows, `## Layer` selects the coverage layer, `## Constraints` and `## Anti-patterns` are the rules `context-validator` and `anti-pattern-scanner` apply, `## Interface` names the `UI-nnn` the component contract comes from; `.devforgeai/stories/sprint.yaml` `stories[].order` fixes which story Build takes next | none — a story is Plan's document to rewrite, and a defect in the implementation is Build's to fix | yes: an `AC-nnn` is untestable as written; arrives as `/plan <EPIC-nnn> --remedy AC-nnn,...` and is triaged by `spec-gap-triager` | none; `story-invest-auditor` judges specifications and `ac-compliance-verifier` judges implementations | reads `[active].build` on a remedy run |
| 5 Verify · `validating-quality` | none — Verify produces `reports/STORY-nnn-qa.yaml`, which Plan reads on a remedy run only, through `report show <STORY-nnn> verify` | `.devforgeai/stories/STORY-nnn.md` `## Acceptance Criteria` (`AC-nnn`), the list `ac-compliance-verifier` checks one by one, and `## Out of scope`, which keeps a finding off a behaviour the story excluded | none — Plan cites `REQ-nnn` and `CON-nnn` upstream, and no `FIND-nnn` downstream | yes: a `FIND-nnn` names a gap between an `AC-nnn` and what the story specifies; arrives as `/plan <EPIC-nnn> --remedy AC-nnn,...` | none | reads `[active].verify` on a remedy run |
| 6 Release · `releasing-software` | none — Plan reads no release manifest | `.devforgeai/stories/sprint.yaml` `stories[].id`, the list `releases/vX.Y.Z.yaml` names and `phase set release` moves to `status: released` | none — a release defect is upstream of nothing Plan owns | none — §5 gives Release's send-back target as Verify | none | none |
| Design · `designing-interfaces` | `.devforgeai/ui-specs/UI-nnn.md` `## Requirements` (the `REQ-nnn` set the screen realizes), `## States` (the state list and the story's size), `## Breakpoints` (the four layouts one `AC-nnn` covers), `## Out of scope` | `.devforgeai/stories/STORY-nnn.md` `consumes` and `## Interface`, which attach the `UI-nnn` to the story; Design reads them at its step 10 when spec mode is called with a `STORY-nnn` | none — a screen with no requirement is Design's own send-back to Discover | no send-back; Plan calls `/design UI-nnn --remedy AC-nnn,...` inside the run and Design returns on the `Then` line as `/plan <EPIC-nnn> --resume` | none; Design owns `mockup-designer`, `brand-designer`, and `requirement-coverage-auditor`, and Plan invokes none of them | reads `current_phase`; Design writes none |
| Reflect · `improving-framework` | none — Reflect returns recommendations, which §5 excludes from gates | `.devforgeai/reports/SPRINT-nnn-plan.yaml`, written by `gate check --phase plan` and by `report ingest story-invest-auditor`, as one input to `OBS-nnn` and `REC-nnn` extraction | none — Plan cites requirement and constraint gaps, not framework observations | none — Reflect emits recommendations, not send-backs | none | none |
| CLI · `devforgeai` | `.devforgeai/config.toml` keys `[plan].sprint_capacity_points`, `[plan].story_points`, `[[layer]].name`, `[[verifier]]`; `.devforgeai/gates.toml` `[[gate]]` with `phase = "plan"`; `.devforgeai/state.toml` `current_phase`, `[active].constitute`, `[active].plan` | `.devforgeai/stories/STORY-nnn.md` and `.devforgeai/stories/sprint.yaml` for `doc validate`, `doc load story`, `doc load sprint`, `story validate`, `story files --check`, and `gate check --phase plan\|build\|verify`; `state.toml` through `phase set plan` | not applicable | not applicable | not applicable | reads `current_phase`, `[active].constitute`; `phase set plan` writes `current_phase` and `[active].plan`; `gate check` writes `[last_gate]`; `handoff` writes `[last_handoff]` |

## Handoff

Printed by `devforgeai handoff`. The phase document is `stories/sprint.yaml`, a YAML file with no H1, so rendering rule 3 of `specs/01-cli.md` §Handoff gives the slug `-`.

PASS, one epic decomposed into nine stories, eight of them fitting the default capacity:

```
Phase     3 · Plan            SPRINT-001 · -
Done      9 stories · 34 ACs
Gate      PASS  5 checks
Verified  story-invest-auditor · 9/9 stories

Next      /build STORY-101
Then      /verify STORY-101
Blocked   none

Full report: .devforgeai/reports/SPRINT-001-plan.yaml
```

Ten lines. `Next` names the `stories[]` entry whose `order` is `1`, which is the forward handoff `/build STORY-nnn` for the first story in sprint order.

SEND BACK to Discover:

```
Phase     3 · Plan            SPRINT-001 · -
Done      6 stories · 19 ACs
Gate      SEND BACK to Discover  2 requirements ambiguous
Verified  story-invest-auditor · 6/8 stories
Found     REQ-011 "recent orders" fixes no window and no count
Found     REQ-014 acceptance_signal names no outcome a test reads

Next      /discover IDEA-004 --remedy REQ-011,REQ-014
Then      /plan EPIC-004 --resume
Blocked   none

Full report: .devforgeai/reports/SPRINT-001-plan.yaml
```

Twelve lines. The `Found` labels and the `--remedy` list are the `id` values of the blocking `findings[]` entries, in the order `report ingest` stored them.

## Templates

### templates/story.md

    ---
    schema: devforgeai/story/1
    id: STORY-nnn
    phase: plan
    status: draft
    produced_by: planning-work
    consumes: []
    open_questions: []
    ---

    # STORY-nnn: <title, 1 to 60 characters>

    ## Story

    As a <persona name>
    I want <one clause>
    So that <one clause>

    ## Requirements

    | REQ | Statement | Covered by |
    |---|---|---|
    | REQ-nnn | <the requirements[].statement value, copied byte for byte> | AC-nnn AC-nnn |

    ## Acceptance Criteria

    - AC-nnn: Given <state> When <action> Then <one outcome a test reads>.

    ## Constraints

    | CON | Statement | Binds |
    |---|---|---|
    | CON-nnn | <index Title>: <constraint statement> | <a Path value from ## Files, or the ## Layer value> |

    ## Anti-patterns

    | AP | Severity | Scope |
    |---|---|---|
    | AP-nnn | <blocker \| high \| medium \| low> | <glob> |

    ## Interface

    | UI | Screen | States covered |
    |---|---|---|
    | UI-nnn | <the UI spec's ## Purpose first sentence> | default loading empty |

    ## Layer

    <a Layer value from source-tree.md ## Layers>

    ## Files

    | Path | Kind | Layer |
    |---|---|---|
    | <repo-relative path> | <source \| test \| config \| migration \| asset> | <a Layer value> |

    ## Dependencies

    - STORY-nnn: <one sentence naming what this story takes from it>

    ## Out of scope

    <one sentence naming a behaviour this story does not carry, and the STORY-nnn that does when one exists>

A section with no rows carries the single line `none` in place of its table or list, in sections `## Constraints`, `## Anti-patterns`, `## Interface`, and `## Dependencies`. `## Out of scope` with nothing to record carries no line. The other six sections carry at least one row.

### templates/sprint.yaml

    schema: devforgeai/sprint/1
    id: SPRINT-nnn
    phase: plan
    status: active
    produced_by: planning-work
    consumes: [EPIC-nnn]
    open_questions: []
    epic: EPIC-nnn
    capacity:
      points_max: 20
      points_planned: 0
      source: default
    stories:
      - id: STORY-nnn
        status: ready
        order: 1
        points: 3
    deferred:
      - id: STORY-nnn
        reason: capacity

### templates/plan-config.toml

The fragment `devforgeai init` merges into `.devforgeai/config.toml` for this phase.

    [plan]
    sprint_capacity_points = 20
    story_points = [1, 2, 3, 5, 8]

    [[verifier]]
    name = "story-invest-auditor"
    phase = "plan"
    report_field = "verifiers.story_invest"
    unit = "stories"
    required = true

## Evals

Three artifacts under `skills/planning-work/evals/`, per §9.

### evals/evals.json

```json
{
  "skill": "planning-work",
  "evals": [
    {
      "prompt": "/plan EPIC-001",
      "expected_output": "Nine STORY files under .devforgeai/stories/ and one sprint.yaml, with the handoff naming /build for the order-1 story.",
      "expectations": [
        "Every file matching .devforgeai/stories/STORY-*.md carries the ten H2 headings in the order Story, Requirements, Acceptance Criteria, Constraints, Anti-patterns, Interface, Layer, Files, Dependencies, Out of scope.",
        "Every acceptance criterion line matches the form '- AC-nnn: Given ... When ... Then ...'.",
        "Every REQ id listed in the epic's requirements key appears in the Requirements table of at least one story.",
        "sprint.yaml carries the eleven top-level keys in the order schema, id, phase, status, produced_by, consumes, open_questions, epic, capacity, stories, deferred.",
        "The handoff Next line is '/build STORY-nnn' for the stories entry whose order is 1."
      ]
    },
    {
      "prompt": "/plan EPIC-002",
      "expected_output": "Stories whose declared file sets do not overlap and whose sprint order respects every declared dependency.",
      "expectations": [
        "No repo-relative path appears in the Files table of two stories listed in sprint.yaml stories.",
        "For every '- STORY-nnn:' item in a story's Dependencies section, that story's order value in sprint.yaml is lower than the dependent story's order value.",
        "Every stories entry carries id first and status second.",
        "The sum of stories[].points is at or below capacity.points_max."
      ]
    },
    {
      "prompt": "/plan EPIC-003",
      "expected_output": "A sprint filled to the configured capacity with the remainder deferred by reason.",
      "expectations": [
        "capacity.points_max equals the config.toml [plan].sprint_capacity_points value and capacity.source is 'config'.",
        "Every story of the epic appears exactly once across sprint.yaml stories and deferred.",
        "Every deferred entry carries a reason of 'capacity' or 'dependency'.",
        "capacity.points_planned equals the sum of stories[].points."
      ]
    },
    {
      "prompt": "/plan EPIC-004",
      "expected_output": "A story whose layer is interface gains a UI-nnn through the Design skill in spec mode, and its Interface section cites that id.",
      "expectations": [
        "The transcript holds an invocation of the designing-interfaces skill in spec mode for a STORY id.",
        "The story whose Layer line reads 'interface' lists a UI-nnn in its Interface table and in its frontmatter consumes list.",
        "A file .devforgeai/ui-specs/UI-nnn.md exists for every UI id a story cites.",
        "The States covered cell holds only values that appear in that UI spec's States table."
      ]
    },
    {
      "prompt": "/plan EPIC-005",
      "expected_output": "A SEND BACK to Discover citing the ambiguous requirement, with requirements.yaml untouched.",
      "expectations": [
        "The handoff Gate line reads 'SEND BACK to Discover'.",
        "The handoff Next line is '/discover IDEA-nnn --remedy REQ-nnn'.",
        ".devforgeai/requirements.yaml is byte-identical to the file the case set up.",
        "No .devforgeai/stories/sprint.yaml is written."
      ]
    },
    {
      "prompt": "/plan EPIC-006 --remedy AC-019",
      "expected_output": "The cited criterion is rewritten in place and the run returns to Build.",
      "expectations": [
        "The AC-019 line in its story differs from the line the case set up.",
        "Every other acceptance criterion line in that story is byte-identical to the line the case set up.",
        "The sprint.yaml stories entry for that story reads status ready and every other line of the file is byte-identical to the file the case set up.",
        "The handoff Next line is '/build STORY-nnn --resume'."
      ]
    }
  ]
}
```

### evals/cases.jsonl

Seven cases, of which `PLAN-05` and `PLAN-06` exercise the SEND BACK path. There is no `.setup/` mirror: the runner writes `setup.files` into the workspace and nowhere else, and a grader that compares a produced file against its input reads the baseline from the skill's own `evals/fixtures/`, which sits outside the workspace. Every case line below is the shipped `cases.jsonl` line verbatim, `FIXTURE:<name>` references included.

```jsonl
{"id": "PLAN-01-headings-and-coverage", "prompt": "/plan EPIC-001", "setup": {"files": {".devforgeai/requirements.yaml": "FIXTURE:requirements-epic-001.yaml", ".devforgeai/context/source-tree.md": "FIXTURE:source-tree.md", ".devforgeai/context/architecture-constraints.md": "FIXTURE:architecture-constraints.md", ".devforgeai/context/anti-patterns.md": "FIXTURE:anti-patterns.md", ".devforgeai/config.toml": "FIXTURE:config-plan.toml", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml", ".devforgeai/reports/IDEA-004-constitute.yaml": "FIXTURE:report-IDEA-004-constitute.yaml", ".devforgeai/context/tech-stack.md": "FIXTURE:tech-stack.md", ".devforgeai/context/coding-standards.md": "FIXTURE:coding-standards.md", ".devforgeai/context/dependencies.md": "FIXTURE:dependencies.md"}}, "expect": {"grader": "story_headings", "args": {"min_stories": 2}}}
{"id": "PLAN-02-req-coverage", "prompt": "/plan EPIC-001", "setup": {"files": {".devforgeai/requirements.yaml": "FIXTURE:requirements-epic-001.yaml", ".devforgeai/context/source-tree.md": "FIXTURE:source-tree.md", ".devforgeai/context/architecture-constraints.md": "FIXTURE:architecture-constraints.md", ".devforgeai/context/anti-patterns.md": "FIXTURE:anti-patterns.md", ".devforgeai/config.toml": "FIXTURE:config-plan.toml", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml", ".devforgeai/reports/IDEA-004-constitute.yaml": "FIXTURE:report-IDEA-004-constitute.yaml", ".devforgeai/context/tech-stack.md": "FIXTURE:tech-stack.md", ".devforgeai/context/coding-standards.md": "FIXTURE:coding-standards.md", ".devforgeai/context/dependencies.md": "FIXTURE:dependencies.md"}}, "expect": {"grader": "req_coverage", "args": {"epic": "EPIC-001", "reqs": ["REQ-001", "REQ-002"]}}}
{"id": "PLAN-03-ac-grammar", "prompt": "/plan EPIC-001", "setup": {"files": {".devforgeai/requirements.yaml": "FIXTURE:requirements-epic-001.yaml", ".devforgeai/context/source-tree.md": "FIXTURE:source-tree.md", ".devforgeai/context/architecture-constraints.md": "FIXTURE:architecture-constraints.md", ".devforgeai/context/anti-patterns.md": "FIXTURE:anti-patterns.md", ".devforgeai/config.toml": "FIXTURE:config-plan.toml", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml", ".devforgeai/reports/IDEA-004-constitute.yaml": "FIXTURE:report-IDEA-004-constitute.yaml", ".devforgeai/context/tech-stack.md": "FIXTURE:tech-stack.md", ".devforgeai/context/coding-standards.md": "FIXTURE:coding-standards.md", ".devforgeai/context/dependencies.md": "FIXTURE:dependencies.md"}}, "expect": {"grader": "ac_grammar", "args": {"min_acs": 2}}}
{"id": "PLAN-04-disjoint-files-and-order", "prompt": "/plan EPIC-001", "setup": {"files": {".devforgeai/requirements.yaml": "FIXTURE:requirements-epic-001.yaml", ".devforgeai/context/source-tree.md": "FIXTURE:source-tree.md", ".devforgeai/context/architecture-constraints.md": "FIXTURE:architecture-constraints.md", ".devforgeai/context/anti-patterns.md": "FIXTURE:anti-patterns.md", ".devforgeai/config.toml": "FIXTURE:config-plan.toml", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml", ".devforgeai/reports/IDEA-004-constitute.yaml": "FIXTURE:report-IDEA-004-constitute.yaml", ".devforgeai/context/tech-stack.md": "FIXTURE:tech-stack.md", ".devforgeai/context/coding-standards.md": "FIXTURE:coding-standards.md", ".devforgeai/context/dependencies.md": "FIXTURE:dependencies.md"}}, "expect": {"grader": "file_sets_disjoint", "args": {}}}
{"id": "PLAN-05-sendback-ambiguous-req", "prompt": "/plan EPIC-002", "setup": {"files": {".devforgeai/requirements.yaml": "FIXTURE:requirements-epic-002.yaml", ".devforgeai/context/source-tree.md": "FIXTURE:source-tree.md", ".devforgeai/context/architecture-constraints.md": "FIXTURE:architecture-constraints.md", ".devforgeai/context/anti-patterns.md": "FIXTURE:anti-patterns.md", ".devforgeai/config.toml": "FIXTURE:config-plan.toml", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml", ".devforgeai/reports/IDEA-004-constitute.yaml": "FIXTURE:report-IDEA-004-constitute.yaml", ".devforgeai/context/tech-stack.md": "FIXTURE:tech-stack.md", ".devforgeai/context/coding-standards.md": "FIXTURE:coding-standards.md", ".devforgeai/context/dependencies.md": "FIXTURE:dependencies.md"}}, "expect": {"grader": "send_back", "args": {"to": "Discover", "next": "/discover IDEA-004 --remedy REQ-011", "ids": ["REQ-011"], "upstream_unchanged": {".devforgeai/requirements.yaml": "5539adbc2df72dd440294136331ee056ec6a6734e2a0f85bf85a9f79a6647bf4"}, "absent": [".devforgeai/stories/sprint.yaml"]}}}
{"id": "PLAN-06-sendback-silent-constraint", "prompt": "/plan EPIC-003", "setup": {"files": {".devforgeai/requirements.yaml": "FIXTURE:requirements-epic-003.yaml", ".devforgeai/context/source-tree.md": "FIXTURE:source-tree.md", ".devforgeai/context/architecture-constraints.md": "FIXTURE:architecture-constraints.md", ".devforgeai/context/anti-patterns.md": "FIXTURE:anti-patterns.md", ".devforgeai/config.toml": "FIXTURE:config-plan.toml", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml", ".devforgeai/reports/IDEA-004-constitute.yaml": "FIXTURE:report-IDEA-004-constitute.yaml", ".devforgeai/context/tech-stack.md": "FIXTURE:tech-stack.md", ".devforgeai/context/coding-standards.md": "FIXTURE:coding-standards.md", ".devforgeai/context/dependencies.md": "FIXTURE:dependencies.md"}}, "expect": {"grader": "send_back", "args": {"to": "Constitute", "next": "/constitute IDEA-004 --remedy CON-003", "ids": ["CON-003"], "upstream_unchanged": {".devforgeai/context/architecture-constraints.md": "6bc8dc64bfcf6d360f20cd845ebbe0edbcbb4fe657620ffc92efab586e021c51"}, "absent": [".devforgeai/stories/sprint.yaml"]}}}
{"id": "PLAN-07-remedy-rewrites-one-ac", "prompt": "/plan EPIC-001 --remedy AC-019", "setup": {"files": {".devforgeai/requirements.yaml": "FIXTURE:requirements-epic-001.yaml", ".devforgeai/context/source-tree.md": "FIXTURE:source-tree.md", ".devforgeai/context/architecture-constraints.md": "FIXTURE:architecture-constraints.md", ".devforgeai/context/anti-patterns.md": "FIXTURE:anti-patterns.md", ".devforgeai/config.toml": "FIXTURE:config-plan.toml", ".devforgeai/stories/STORY-104.md": "FIXTURE:story-104.md", ".devforgeai/stories/sprint.yaml": "FIXTURE:sprint-001.yaml", ".devforgeai/reports/STORY-104-build.yaml": "FIXTURE:story-104-build.yaml", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml", ".devforgeai/reports/IDEA-004-constitute.yaml": "FIXTURE:report-IDEA-004-constitute.yaml", ".devforgeai/context/tech-stack.md": "FIXTURE:tech-stack.md", ".devforgeai/context/coding-standards.md": "FIXTURE:coding-standards.md", ".devforgeai/context/dependencies.md": "FIXTURE:dependencies.md"}}, "expect": {"grader": "remedy_ac", "args": {"story": "STORY-104", "ac": "AC-019", "untouched_acs": ["AC-018"], "sprint_status": {"STORY-104": "ready"}, "next": "/build STORY-104 --resume", "baseline_story": "story-104.md", "baseline_sprint": "sprint-001.yaml"}}}
```

`__same_as__` and `__ctx__` are notation in this spec only; the shipped `cases.jsonl` repeats the full `setup.files` mapping on every line, because the runner reads one self-contained JSON object per line.

### evals/graders.py

Pure functions, no network, no subprocess to a model, no randomness. Each returns `(passed, evidence)`.

```python
def story_headings(workspace: str, transcript: str, args: dict) -> tuple[bool, str]: ...
```
Globs `.devforgeai/stories/STORY-*.md`. Fails when the count is below `args["min_stories"]`. For each file, extracts the lines beginning `## ` in file order and compares the list to the ten headings of `## Outputs` section 1 by exact string equality. Evidence names the first file and the first index at which the lists differ.

```python
def req_coverage(workspace: str, transcript: str, args: dict) -> tuple[bool, str]: ...
```
Reads every story's `## Requirements` table, collects the first-column values, and compares the set to `args["reqs"]`. Fails on a missing id and on an id absent from `args["reqs"]`. Evidence lists both differences.

```python
def ac_grammar(workspace: str, transcript: str, args: dict) -> tuple[bool, str]: ...
```
For each story, collects lines under `## Acceptance Criteria` matching `^- (AC-\d{3}): (.+)$`. Fails when the total is below `args["min_acs"]`, when a text does not match `Given .+ When .+ Then .+`, when an id appears in no `Covered by` cell of that story's `## Requirements` table, or when it appears in more than one. Evidence names the first offending id and which of the four conditions it hit.

```python
def file_sets_disjoint(workspace: str, transcript: str, args: dict) -> tuple[bool, str]: ...
```
Reads `sprint.yaml` `stories[]`, reads each named story's `## Files` first column, and fails when one path appears under two ids. Also checks the order property: for each `- STORY-nnn:` item under a story's `## Dependencies`, the dependency's `order` is lower. Evidence names the duplicated path or the offending pair.

```python
def sprint_shape(workspace: str, transcript: str, args: dict) -> tuple[bool, str]: ...
```
Parses `sprint.yaml` with a line-oriented reader that records top-level keys in file order, and compares that list to the eleven of `## Outputs` section 2. Fails when `capacity.points_planned` does not equal the sum of `stories[].points`, when that sum exceeds `capacity.points_max` while `len(stories) > 1`, when a `points` value is outside `args["points"]`, when `order` is not a dense permutation of `1..len(stories)`, or when an id appears in both `stories` and `deferred`. Evidence names the failing property.

```python
def design_called(workspace: str, transcript: str, args: dict) -> tuple[bool, str]: ...
```
Fails when the transcript holds no `/design ` occurrence followed by `--spec` on the same line. Reads the story whose `## Layer` line equals `interface`, collects the `UI-nnn` values from its `## Interface` table and its frontmatter `consumes`, and fails when either is empty, when the two sets differ, when `.devforgeai/ui-specs/<UI-nnn>.md` is absent, or when a `States covered` value is absent from that spec's `## States` first column. Evidence names the id and the failing condition.

```python
def send_back(workspace: str, transcript: str, args: dict) -> tuple[bool, str]: ...
```
Fails when the transcript holds no line beginning `Gate` whose content starts `SEND BACK to ` followed by `args["to"]`, when the `Next` line does not equal `args["next"]`, when an id of `args["ids"]` appears on no `Found` line, when a path of `args["upstream_unchanged"]` differs byte for byte from the fixture it was written from, or when a path of `args["absent"]` exists. Evidence names the first failing condition and the line read.

```python
def remedy_ac(workspace: str, transcript: str, args: dict) -> tuple[bool, str]: ...
```
Reads `.devforgeai/stories/<args["story"]>.md` and the fixture named by `args["baseline_story"]`. Fails when the `args["ac"]` line is byte-identical to the setup line, when the rewritten line does not match `Given .+ When .+ Then .+`, when an id of `args["untouched_acs"]` differs from its setup line, when the story's frontmatter `status` is not `ready`, when the `sprint.yaml` `stories[]` entry for `args["story"]` does not read `status: ready`, when any other line of `sprint.yaml` differs from its setup copy, or when the transcript's `Next` line does not equal `args["next"]`. Evidence names the first failing condition.

## Decisions

1. `produced_by` for both documents is `planning-work`, the §4b skill name for `/plan`. `specs/01-cli.md` §`doc validate` doc-type table gives `devforgeai-plan` and its §Decisions 21 gives the pattern `devforgeai-<phase>`; `specs/04-constitute.md` eval fixture `CON-07` gives `planning-stories`. Conventions §4b is authoritative over both. Amendment to `specs/01-cli.md` doc-type table rows `story` and `sprint`, and to `specs/04-constitute.md` fixture `CON-07`. `specs/08-design.md` §Integration already names this skill `planning-work`.

2. The story `status` enum is the five values `specs/01-cli.md` fixes: `draft`, `ready`, `building`, `built`, `released`. Plan writes `draft` at step 7 and `ready` at step 11; `phase set` writes `building`, `built`, `released`. `ready` is absent from `phase set`'s transition table, which is why Plan owns it.

3. The sprint `status` enum is the three values `specs/01-cli.md` fixes: `planned`, `active`, `closed`. Plan writes `active` at step 11, because the `plan-sprint-status` check requires it. `planned` is the template default before step 11 and `closed` is written by Release.

4. The Plan phase report is `.devforgeai/reports/SPRINT-nnn-plan.yaml`, because `state.toml` `[active].plan` holds a `SPRINT-nnn` and `gate check --phase plan` names the report after the active id. Confirmed by `specs/01-cli.md` §`gate require` `--json` example and by `specs/08-design.md` §Integration row 3. This changes one fixture path in `specs/04-constitute.md` eval case `CON-07`, from `.devforgeai/reports/STORY-041-plan.yaml` to `.devforgeai/reports/SPRINT-001-plan.yaml`, which that spec's §Decisions 30 delegates here.

   The `SPRINT-nnn` is therefore allocated and set as `[active].plan` at workflow step 4, before any subagent runs, so `report ingest story-invest-auditor -` at step 9 resolves its target to this path rather than to `""` or to the previous epic's sprint. `sprint.yaml` is one file the next epic's run overwrites, and the overwritten id survives in the frontmatter `id` of that epic's `reports/SPRINT-nnn-plan.yaml`, which the ID index walks, so `doc validate --allocate SPRINT` returns a fresh number and no report is overwritten. A run whose epic equals the `epic` key of the `sprint.yaml` on disk reuses that file's `id` and allocates nothing, which is what a `--resume` run and a second pass over the same epic do.

5. The send-back payload in the Plan report is the CLI-written `findings[]` array plus `gate.send_back_to`, with no additional top-level key. `specs/04-constitute.md` fixture `CON-07` carries a top-level `send_back` mapping with `to`, `constraint`, and `reason`; those three values are `gate.send_back_to`, `findings[].id`, and `findings[].summary`. Amendment to that fixture, delegated here by its §Decisions 30.

6. A `story-invest-auditor` finding's `id` is the upstream id it cites — a `REQ-nnn`, a `CON-nnn`, or the `STORY-nnn` it judges — rather than an allocated `FIND-nnn`, because §5 assigns the `FIND` prefix to Verify. `devforgeai handoff` then renders `Found <id> <summary>` and composes the SEND BACK `--remedy` list from the blocking findings' `id` values with no further field, which is the mechanism `specs/04-constitute.md` §Handoff already depends on for its `Next` line.

7. Proposed addition to the default `.devforgeai/gates.toml` in `specs/01-cli.md` §Gate: a fifth `[[gate.check]]` in the `plan` gate, using the existing `verifier_pass` kind.

   ```toml
     [[gate.check]]
     kind = "verifier_pass"
     id = "plan-invest"
     verifiers = ["story-invest-auditor"]
     min_ratio = 1.0
     on_fail = "send_back"
   ```

   Without it a blocking judgment finding leaves `gate.result` at PASS and Plan has no send-back path. The compiled `min_ratio` floor of 1.0 is a verify-phase floor and binds nothing here; the value is stated at 1.0 anyway.

8. Proposed addition to `specs/01-cli.md` §`gate check`: the report's `gate.send_back_to` is resolved from the prefixes of the blocking `findings[].id` values through the fixed map `REQ` → `discover`, `CON` → `constitute`, `AP` → `constitute`, `UI` → `design`, falling back to the gate's static `send_back_to` when no blocking finding is present, and resolving to `constitute` when findings of two prefixes are present. The plan gate's static `send_back_to` is `discover` and Plan has two upstream targets, which `specs/01-cli.md` §Handoff already anticipates with the row "`/discover <EPIC-nnn>` or `/constitute <ADR-nnn>`, by `send_back_to`".

9. Proposed amendment to `specs/01-cli.md` §`doc validate` "ID index, definitions, references": a Markdown list item whose text begins `<PREFIX>-<nnn>:` at the start of the line defines that ID, alongside the three existing forms. This is load-bearing: `story validate` check 2 fixes the acceptance-criterion grammar as `^- (AC-[0-9]{3}): (.+)$`, the `plan-ids` check resolves the `AC` prefix, and without the amendment every `AC-nnn` is an unresolved reference and the plan gate cannot pass.

10. Proposed amendment to `specs/01-cli.md` §`doc validate` doc-type table: the `sprint` doc type permits top-level keys beyond the §5 seven, joining `build-report`, `qa-report`, `release`, and `reflect-report`. The same spec's `templates/sprint.min.yaml` already carries a top-level `stories` key, which `DFA-E204` would otherwise reject.

11. `devforgeai doc validate --allocate AC` and `--allocate SPRINT` are existing behaviour and no addition is proposed: `--allocate` rejects a prefix outside the `specs/01-cli.md` doc-type table with `DFA-E214`, and that table's `story` row lists the ID prefixes `STORY, AC` while its `sprint` row lists `SPRINT`. Recorded because workflow steps 4 and 8 call both forms.

12. Proposed addition to conventions §4 and to `specs/01-cli.md`: a `story files` subcommand, so the declared file set has an enforcement point. Grammar:

    ```
    devforgeai story files --check <path> [--id <STORY-nnn>] [--json] [--project <path>]
    devforgeai story files --list [--id <STORY-nnn>] [--json] [--project <path>]
    ```

    `--id` defaults to `state.toml` `[active].build`. `--check` exits 0 when `<path>`, made repo-relative, equals a `Path` value in that story's `## Files` table, and exits 1 with `DFA-E239` otherwise. `--list` prints one `Path`, `Kind`, `Layer` triple per line. `--json` `data` for `--check`: `{"id":"STORY-104","path":"src/application/checkout/place_order.ext","allowed":true,"declared":2}`. An absent story is `DFA-E200`, exit 1. An absent `[active].build` is `DFA-E412`, exit 0, so the hook does not block outside a Build run.

13. Proposed addition to conventions §7 hooks table: a second `PreToolUse` Write and Edit entry calling `devforgeai story files --check <path>` when the path is outside `.devforgeai/` and `state.toml` `current_phase` is `build`. The `hook run pre-tool-use` dispatcher maps exit 1 to hook exit 2, the same mapping `specs/01-cli.md` §Decisions 6 applies to `doc validate --producer-check` and `design lint`.

14. Proposed additions to the error table of `specs/01-cli.md`, in that table's format:

    | Code | Subcommand | Condition | Message | Exit |
    |---|---|---|---|---|
    | DFA-E235 | story validate | an epic requirement is covered by no story | `<EPIC-nnn> requirement <REQ-nnn> appears in the Requirements table of no story` | 1 |
    | DFA-E236 | story validate | AC appears in no Covered by cell | `<STORY-nnn> <AC-nnn> appears in no Covered by cell of ## Requirements` | 1 |
    | DFA-E237 | story validate | declared file sets overlap | `<STORY-nnn> and <STORY-nnn> both declare <path>` | 1 |
    | DFA-E238 | story validate | cited UI spec absent | `<STORY-nnn> cites <UI-nnn>, which .devforgeai/ui-specs/ does not hold` | 1 |
    | DFA-E239 | story files | path outside the declared set | `<path> is outside the declared file set of <STORY-nnn>` | 1 |
    | DFA-E245 | story validate | AC covers two requirements | `<STORY-nnn> <AC-nnn> appears in the Covered by cell of <REQ-nnn> and <REQ-nnn>` | 1 |

15. Proposed additions to the checks `devforgeai story validate` performs, numbered from the six in `specs/01-cli.md` §`story validate`. Check 7: every `requirements[].id` the epic named in `sprint.yaml` `epic` lists is present in the first column of the `## Requirements` table of some story in the union of `stories[].id` and `deferred[].id`, else `DFA-E235`; `--scope sprint` only, and the epic comes from the `epic` key, so no `--epic` flag is added. Check 8: each `AC-nnn` appears in exactly one `Covered by` cell of its own story's `## Requirements` table, else `DFA-E236` for none and `DFA-E245` for more than one. Check 9: no `Path` value appears in the `## Files` table of two stories listed in `sprint.yaml` `stories[]`, which is the concurrent set and excludes `deferred[]`, else `DFA-E237`; `--scope sprint` only. Check 10: every `UI-nnn` in a story's `consumes` resolves to `.devforgeai/ui-specs/<UI-nnn>.md`, else `DFA-E238`.

16. Proposed addition to the `.devforgeai/config.toml` schema in `specs/01-cli.md`: a `[plan]` table with `sprint_capacity_points` (integer, default 20) and `story_points` (array of integer, default `[1, 2, 3, 5, 8]`). `stack detect` leaves it at its previous value when the file exists and writes the defaults when it does not, the rule that already governs `[[layer]]`, `[coverage]`, and `[[verifier]]`.

17. Sprint capacity defaults to 20 points. The default is stated in `[plan].sprint_capacity_points` and `capacity.source` records for each sprint whether the number came from the file or the default, so the file's history says which.

18. Story points are drawn from `[1, 2, 3, 5, 8]`. A draft `story-decomposer` sizes above 8 is split into two drafts before it returns, because 13 and above carries no distinct meaning at a one-story Build granularity.

19. `$1` of `/plan` accepts the `EPIC` and `SPRINT` prefixes. `SPRINT-nnn` resolves to an epic through `sprint.yaml` `epic`. Both are needed: `specs/01-cli.md` §Handoff routes a plan FAIL to `/plan <SPRINT-nnn>` and `specs/08-design.md` §Send-back routes a Design remedy back to `/plan <SPRINT-nnn> --resume`, while `specs/04-constitute.md` §Handoff and §Decisions 29 route to `/plan <EPIC-nnn>` and `/plan <EPIC-nnn> --resume`. `gate require plan $1` resolves for both, because it falls back to `reports/<[active].constitute>-constitute.yaml` when `reports/<$1>-constitute.yaml` is absent.

20. The `--remedy` id set of `/plan` is closed at the `AC` prefix. Build and Verify both cite an `AC-nnn`; Verify's `FIND-nnn` names the criterion, which is the id the remedy re-opens. §4c fixes the flag name and the comma-separated form.

21. `--resume` re-enters at workflow step 2, re-reading `requirements.yaml` and the six context files, because the repair a send-back asked for changed one of them. Stories already on disk with `status: draft` keep their allocated ids.

22. The send-back to Constitute cites a `CON-nnn` in both conditions, including the "missing constraint" condition, because `## Layer dependency rules` carries one `CON-nnn` per layer row and a constraint that governs the layer and decides no case the story meets is the id to re-open. A `--remedy` list with two `CON-nnn` entries widens the `argument-hint` of `specs/04-constitute.md` §Command from `[--remedy CON-nnn]` to `[--remedy CON-nnn,CON-nnn]`; proposed amendment to that spec.

23. The send-back to Discover cites `<IDEA-nnn>`, the top-level `id` of `requirements.yaml`, not the `EPIC-nnn` the run took. `specs/03-discover.md` §Integration gives the arriving form as `/discover IDEA-nnn --remedy REQ-nnn,...`; `specs/04-constitute.md` §Send-back gives `/discover <EPIC-nnn> --remedy ...`. Discover owns its own command signature, so `IDEA-nnn` wins, and `specs/04-constitute.md` §Send-back is amended.

24. Plan calls the Design skill in spec mode when a draft's `layer` is `interface` and every `REQ-nnn` in its `req_ids` carries an empty `traces_to` list. Both are typed fields: `layer` is drawn from `source-tree.md` `## Layers`, and `traces_to` is `list of string, each ^UI-\d{3}$` in `specs/03-discover.md` §Outputs. The call is `/design STORY-nnn --spec`, which `specs/08-design.md` §Command accepts as spec mode with a `STORY` id in `$1`.

25. Plan reads four sections of each returned `UI-nnn.md`: `## Requirements`, `## States`, `## Breakpoints`, `## Out of scope`. This is the set `specs/08-design.md` §Integration row 3 names, quoted rather than widened.

26. The verifier finding `severity` vocabulary is `block`, `warn`, `info`, fixed by `specs/01-cli.md` §Subagents. The `Severity` column of a story's `## Anti-patterns` table is `blocker`, `high`, `medium`, `low`, fixed by `specs/04-constitute.md` §Outputs. The two are different fields of different documents and are not merged; neither vocabulary uses an all-caps token the §2 ceremony pattern matches.

27. A subagent whose JSON does not parse against its schema is invoked once more with the parse error appended; a second failure leaves the gate metric absent, which `gate check` reports as exit 1. This mirrors `specs/04-constitute.md` §Decisions 26.

28. `story-invest-auditor` is the one registered verifier of this phase, so the handoff renders one `Verified` line with no tie to break. `story-decomposer`, `story-file-set-planner`, `sprint-sequencer`, and `spec-gap-triager` return run-local JSON that the skill consumes and `SubagentStop` ignores, because a name absent from the `[[verifier]]` registry is a no-op.

29. The handoff `Phase` line renders the slug `-`, because rendering rule 3 of `specs/01-cli.md` §Handoff takes the slug from the phase document's first H1 and `stories/sprint.yaml` is YAML with no H1.

30. The `Done` line reads `<n> stories · <m> ACs`, which `specs/01-cli.md` §`Done` counts already fix for this phase, counted over `.devforgeai/stories/`. Deferred stories are counted, because their files exist.

31. `dependency-graph-analyzer`, `epic-coverage-result-interpreter`, `context-preservation-validator` and `ui-spec-formatter` are not invoked here (Discover and Design own them), and `api-designer` is replaced, with reasons in `## Subagents`. `specs/04-constitute.md` §Decisions 17 named Plan the owner of `api-designer`; Plan's answer is that no skill invokes it. Recorded for `specs/11-subagent-catalog.md`.

32. Dependency: the forward handoff `Next` line is `/build <STORY-nnn>` for the `stories[]` entry whose `order` is `1`. `specs/06-build.md` settles `/build`'s `argument-hint`; a different first-argument type changes this one line and the `sprint_shape` grader's `order` check alone.

33. Dependency: the receiving form `/plan <EPIC-nnn> --remedy AC-nnn,...` binds what `specs/06-build.md` and `specs/07-verify.md` print on their SEND BACK `Next` lines, because §1.7 has the user type what the handoff printed. A different id prefix in those specs changes the `## Send-back` Received table and the `remedy_ac` grader arg alone.

34. Blocker: none. Every capability this spec names is a Read, Write, Edit, Bash, Grep, Glob, Agent, AskUserQuestion, or Skill call, a hook entry in `.claude/settings.json`, or a `devforgeai` subcommand that exists or is proposed above with its grammar.
