---
schema: devforgeai-spec/1
doc: discover
status: draft
produced_by: spec-author-discover
consumes: [00-conventions]
open_questions: []
---

# Phase 1 · Discover — the `discovering-requirements` skill

## Scope

Discover turns one of two inputs into one document. The first input is a promoted Explore brief: `.devforgeai/explore/brief.md` plus `.devforgeai/explore/decision.yaml` whose `decision` key holds `promote`. The second input is a one-line description typed by the user when no Explore phase ran. The output is `.devforgeai/requirements.yaml`, holding personas (`PERSONA-nnn`), epics (`EPIC-nnn`), and requirements (`REQ-nnn`), frozen by an explicit user acceptance recorded in `accepted_by` and `accepted_at`. Discover also receives re-open requests from Plan, Constitute, and Design, and sends defects in the brief back to Explore.

Discover is not design and not planning. It names no technology, no repository layout, no test framework, and no interface element; those belong to Constitute, Plan, and Design. It writes no story, no acceptance criterion in Given/When/Then form, and no estimate. It performs no market research, no competitive analysis, and no feasibility study; those belong to Explore. It evaluates no gate and computes no count that appears in a handoff; the `devforgeai` binary does both.

## Inputs

| Document | Path | IDs read | When |
|---|---|---|---|
| Explore brief | `.devforgeai/explore/brief.md` | `IDEA-nnn`, `FLOW-nnn` | entry point A |
| Explore decision | `.devforgeai/explore/decision.yaml` | `IDEA-nnn` | entry point A |
| User description | slash-command argument, no file | none | entry point B |
| Requirements (own prior revision) | `.devforgeai/requirements.yaml` | `IDEA-nnn`, `EPIC-nnn`, `REQ-nnn`, `PERSONA-nnn` | entry point C |
| Citing report | `.devforgeai/reports/<ID>-plan.yaml`, `.devforgeai/reports/<ID>-constitute.yaml`, `.devforgeai/reports/<ID>-design.yaml` | `REQ-nnn`, `UI-nnn` | entry point C |
| Configuration | `.devforgeai/config.toml` | none | every entry point |

Both Explore files reach the skill through the command preamble. What Discover reads from `brief.md` (`schema: devforgeai/explore-brief/1`):

| Brief section | Shape read | Becomes |
|---|---|---|
| frontmatter `id` | `IDEA-nnn` | the `id` of `requirements.yaml` |
| `## Core flows` | table `ID \| Actor \| Trigger \| Steps \| Outcome`, 3 to 5 rows, `ID` is `FLOW-nnn`, `Steps` is one line joined with ` -> ` | `requirements[]` drafts and their `source` |
| `## Problem statement` | one sentence | the rationale seed for the drafted requirements |
| `## Target user` | one segment line plus 2 to 4 attribute lines | the first persona |
| `## Success signal` | one line, `<metric> \| <threshold> \| <observation window>` | a round 2 outcome candidate |
| `## Non-goals` | 1 to 10 present-tense lines | round 3 exclusion candidates |

The first four rows are the two `carry_forward` entries 02-explore.md addresses to `discovering-requirements`. `## Success signal` and `## Non-goals` are read as candidate text for rounds 2 and 3 only; neither becomes a requirement, and `## Non-goals` still reaches `establishing-context` as its own `carry_forward` entry states. Discover reads no other section of the brief.

From `decision.yaml` (`schema: devforgeai/explore-decision/1`) Discover reads two fields. `decision` gates the run: the only value Discover proceeds on is `promote`, and `kill` or `park` ends the run with the handoff `Blocked   you: decision.yaml for <IDEA-nnn> holds <value>, not promote`. `remedied_flows` names the `FLOW-nnn` ids Explore rewrote on its latest remedy run, and is the list the resume form names at step 4.

## Outputs

One document: `.devforgeai/requirements.yaml`.

### Top-level keys, in this order

```yaml
schema: devforgeai/requirements/1
id: IDEA-004
phase: discover
status: accepted
produced_by: discovering-requirements
consumes: [IDEA-004, FLOW-001, FLOW-002]
open_questions: []
revision: 1
revision_log: []
accepted_by: user
accepted_at: 2026-09-10T14:22:05Z
personas: []
epics: []
requirements: []
```

| Key | Type | Required | Default | Constraint |
|---|---|---|---|---|
| `schema` | string | yes | none | constant `devforgeai/requirements/1` |
| `id` | string | yes | none | `^IDEA-\d{3}$` |
| `phase` | string | yes | none | constant `discover` |
| `status` | string | yes | `drafting` | enum below |
| `produced_by` | string | yes | none | constant `discovering-requirements` |
| `consumes` | list of string | yes | `[]` | each matches `^(IDEA\|FLOW)-\d{3}$`; `[]` at entry point B |
| `open_questions` | list of string | yes | `[]` | each matches `^REQ-\d{3}$`; holds every REQ whose own `open_questions` is non-empty |
| `revision` | integer | yes | `1` | `>= 1`; raised by exactly 1 per re-open pass |
| `revision_log` | list of object | yes | `[]` | entry schema below |
| `accepted_by` | string or null | yes | `null` | enum `user`, or null |
| `accepted_at` | string or null | yes | `null` | RFC 3339 UTC, `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$`, or null |
| `personas` | list of object | yes | `[]` | entry schema below; length `>= 1` |
| `epics` | list of object | yes | `[]` | entry schema below; length `>= 1` |
| `requirements` | list of object | yes | `[]` | entry schema below; length `>= 1` |

### Enums, closed

| Enum | Members |
|---|---|
| document `status` | `drafting`, `awaiting_acceptance`, `accepted`, `reopened` |
| requirement `priority` | `must`, `should`, `could`, `wont` |
| requirement `status` | `draft`, `accepted`, `reopened`, `withdrawn` |
| requirement `source` | `user`, or a string matching `^FLOW-\d{3}$` |
| `accepted_by` | `user` |
| `revision_log[].from` | `plan`, `constitute`, `design` |

### `revision_log[]`

| Field | Type | Required | Default | Constraint |
|---|---|---|---|---|
| `revision` | integer | yes | none | the value of `revision` after the raise |
| `at` | string | yes | none | RFC 3339 UTC |
| `from` | string | yes | none | enum `plan`, `constitute`, `design` |
| `reopened` | list of string | yes | `[]` | each `^REQ-\d{3}$`, status moved to `reopened` |
| `added` | list of string | yes | `[]` | each `^REQ-\d{3}$`, allocated by this pass |

### `personas[]`

| Field | Type | Required | Default | Constraint |
|---|---|---|---|---|
| `id` | string | yes | none | `^PERSONA-\d{3}$`, unique in the file |
| `name` | string | yes | none | 1 to 40 characters, a role label |
| `description` | string | yes | none | one sentence, 1 to 200 characters |
| `goal` | string | yes | none | one sentence, 1 to 200 characters |

### `epics[]`

| Field | Type | Required | Default | Constraint |
|---|---|---|---|---|
| `id` | string | yes | none | `^EPIC-\d{3}$`, unique in the file |
| `title` | string | yes | none | 1 to 60 characters |
| `scope` | string | yes | none | one sentence, 1 to 200 characters |
| `out_of_scope` | list of string | yes | none | length `>= 1`, each one sentence, 1 to 200 characters |
| `success_metric` | string | yes | none | one sentence carrying one measurable quantity, its unit or count, and a comparison; at least one digit |
| `requirements` | list of string | yes | none | each `^REQ-\d{3}$`, each resolving to a `requirements[].id`, no duplicates across all epics |

`Every clerk closes the night.` carries no quantity, `Unmatched lines drop` carries a comparison with nothing to compare, and `Reconciliation improves` carries neither; each leaves `success_metric` failing the shape rule and `doc validate` reporting it on the write. The sentence is drafted once, at round 2, and travels to this field word for word. The epic count equals the count of outcomes the user selected at round 2, which is what keeps `success_metric` from having no producer when the user selects fewer outcomes than the grouping would otherwise have produced epics.

### `requirements[]`

| Field | Type | Required | Default | Constraint |
|---|---|---|---|---|
| `id` | string | yes | none | `^REQ-\d{3}$`, unique in the file |
| `actor` | string | yes | none | `^PERSONA-\d{3}$`, resolving to a `personas[].id` |
| `statement` | string | yes | none | one sentence, 1 to 200 characters |
| `rationale` | string | yes | none | one sentence, 1 to 200 characters |
| `acceptance_signal` | string | yes | none | one sentence, 1 to 200 characters, naming one observable result |
| `priority` | string | yes | none | enum `must`, `should`, `could`, `wont` — MoSCoW |
| `source` | string | yes | none | `user`, or `^FLOW-\d{3}$` present in `consumes` |
| `status` | string | yes | `draft` | enum `draft`, `accepted`, `reopened`, `withdrawn` |
| `open_questions` | list of string | yes | `[]` | each one sentence, 1 to 200 characters |
| `traces_to` | list of string | yes | `[]` | each `^UI-\d{3}$`; filled by a re-open from Design |

An ID that once appears in `requirements[]`, `epics[]`, or `personas[]` stays in the file for the life of the project. A requirement the user drops moves to `status: withdrawn` and keeps its `id`, `statement`, and every other field as written.

## Workflow

Steps run in this order. Entry point A is a promoted Explore brief, B is a typed description, C is a re-open from Plan, Constitute, or Design.

1. **Resolve the entry point.** Actor: model. Input: `$ARGUMENTS` and the `!` preamble block from `## Command`. Output: one of A, B, C, or the resume form. `--remedy` in the arguments selects C and `$1` is the `IDEA-nnn`. `--resume` in the arguments re-enters at step 4 for `$1`, which is the only step a run stops at without writing (`## Send-back`); every other step ends at a written document or at a `Blocked` line. A brief printed by the preamble selects A and `$1` is the `IDEA-nnn`. A preamble that printed nothing selects B and `$ARGUMENTS` is the description. Failure path: `$ARGUMENTS` empty ends the run with `Blocked   you: name an IDEA id, a description, or an IDEA id with --remedy`.

2. **Fix the document id.** Actor: CLI. Entry A: the id is the `id` printed by `doc load discover-entry`. Entry B: `devforgeai doc validate --allocate IDEA` prints it. Entry C: the id is the `id` key of the loaded `requirements.yaml`. Output: `IDEA-nnn`, used for `state.toml`, `gate check --id`, and the report path in every later step. Failure path: a non-zero exit from `--allocate` ends the run with the binary's stderr on the `Blocked` line.

3. **Set the phase.** Actor: CLI. Input: `IDEA-nnn`. Call: `devforgeai phase set discover --id <IDEA-nnn>`. Output: `state.toml` `[current].phase` set to `discover`, `[current].id` and `[active].discover` set to `<IDEA-nnn>`. Failure path: exit 3 ends the run with the binary's stderr on the `Blocked` line.

4. **Audit the flows.** Actor: subagent `flow-integrity-auditor`, entry point A and the resume form. Input: the `## Core flows` rows, each carrying `ID`, `Actor`, `Trigger`, `Steps`, `Outcome`. Output: the JSON in `## Subagents`. A row whose `Actor` cell is empty is `actorless` with `reason: empty`; a cell holding text that names no role a person could hold is `reason: not_a_persona`; a cell naming a role the brief does not mention is `reason: unlisted_role`, carried with the `confidence` that says how far the reading goes, because the row may be right and the brief thin. A pair of rows whose `Trigger`, `Steps`, or `Outcome` cells cannot both hold is a contradiction. When `actorless` or `contradictions` is non-empty, the run goes to `## Send-back` and stops here. The resume form audits every row again and names the `remedied_flows` ids in the prompt as the rows Explore rewrote. Entry points B and C skip this step.

5. **Draft the personas.** Actor: subagent `persona-mapper`. Input: the `## Target user` lines plus the `Actor` cells of `## Core flows` (A), the description (B), or the existing `personas[]` block (C). Output: candidate persona records keyed by a kebab-case `key`. Entry C reuses the existing personas and skips this step when the re-open cites no new actor.

6. **Round 1 — actors.** Actor: user, through AskUserQuestion. Input: the candidate personas from step 5. Output: the confirmed persona set. Text and routing in `## Templates`, file `templates/questions.md`. Entry point C skips this step.

7. **Round 2 — outcomes.** Actor: user, through AskUserQuestion. Input: three to four candidate outcomes, the first drafted from the brief's `## Success signal` line and the rest from the `Outcome` cells of `## Core flows` (A), or all of them from the description (B). Output: the outcome set. Step 11 emits exactly one epic per selected outcome, and the nth selected outcome becomes the nth epic's `success_metric` verbatim, so each candidate is drafted in the shape that field takes: one sentence carrying one measurable quantity, its unit or count, and a comparison, as in `Unmatched lines fall below 5 per night.` — the quantity `5`, the unit `lines per night`, the comparison `fall below`. A candidate phrased as a direction with no quantity — `fewer unmatched lines`, `faster nightly close` — reaches `epics[].success_metric` word for word and leaves the field failing the shape rule of `## Outputs`, which `doc validate` reports at step 12. Entry point C skips this step.

8. **Round 3 — boundaries.** Actor: user, through AskUserQuestion. Input: three to four candidate exclusions drafted from the brief's `## Non-goals` lines (A) or from the description (B). Output: the exclusion set. Step 11 assigns each selected exclusion to exactly one epic as an `out_of_scope` entry; an epic drawing no assignment takes the single entry `Nothing was named out of scope at discovery.` Entry point C skips this step. Elicitation ends here. An unknown that survives round 3 becomes an entry in the owning requirement's `open_questions` rather than a fourth round.

9. **Draft the requirements.** Actor: subagent `requirement-drafter`. Input: the confirmed personas, outcomes, exclusions, the `## Problem statement` sentence, and either the `## Core flows` rows (A), the description (B), or the reopened records plus the citing report text (C). Output: the JSON in `## Subagents`. Failure path: output that does not parse as that schema is passed back to the same subagent once with the parser error appended; a second unparseable output ends the run with `Blocked   you: requirement-drafter returned no parseable draft`.

10. **Allocate the IDs.** Actor: CLI. Input: the drafter keys. Call: `devforgeai doc validate --allocate PERSONA`, `--allocate REQ`, once per new record. Output: the key-to-ID map. Entry C allocates only for records in `revision_log[].added`.

11. **Group the epics.** Actor: subagent `epic-grouper`, entry points A and B. Input: the requirement records with their allocated IDs, the outcome set, the exclusion set. Output: the JSON in `## Subagents`, holding exactly one epic per selected outcome, each carrying the nth selected outcome as its `success_metric` unchanged, so the number step 7 put there is the number the field holds. Step 10 runs again with `--allocate EPIC` for each new epic key. At entry point C with an empty `revision_log[].added`, the step is skipped and `epics[]` keeps every byte. At entry point C with a non-empty `revision_log[].added`, the subagent runs in placement mode: its input is the added records and the existing `epics[]` block, its output names one existing `EPIC-nnn` per added `REQ-nnn`, it allocates no epic, and the only change to `epics[]` is the appended ID in the named epic's `requirements` list.

12. **Write the document.** Actor: model, Write tool. Input: the persona, epic, and requirement records. Output: `.devforgeai/requirements.yaml` with `status: awaiting_acceptance`, every requirement at `status: draft` (A, B) or its current status (C), and the top-level `open_questions` list holding, in ascending ID order, every `REQ-nnn` whose own `open_questions` is non-empty. The PostToolUse hook runs `doc validate` on the written path.

13. **Round 4 — acceptance.** Actor: user, through AskUserQuestion. Input: the epic count and requirement count. Output: one of `Accept`, `Regroup epics`, `Redraft requirements`. `Regroup epics` returns to step 11 with the epic block cleared. `Redraft requirements` returns to step 9 for the REQ IDs the user names in the same reply, leaving every other record as written; a reply that names no `REQ-nnn` re-asks step 13 unchanged. Both non-accept answers return to step 13 afterward. Text in `templates/questions.md`.

14. **Freeze the set.** Actor: CLI. Call: `devforgeai doc accept requirements --id <IDEA-nnn>`. Output: `accepted_by: user`, `accepted_at` in RFC 3339 UTC, document `status: accepted`, and every requirement at `draft` or `reopened` moved to `accepted`. Failure path: exit 1 ends the run with the binary's stderr on the `Blocked` line.

15. **Stop.** Actor: Stop hook. The hook runs `gate check --phase discover --id <IDEA-nnn>` and then `handoff`, which prints the block in `## Handoff`.

### Entry point C, before step 2

C1. **Load the citing report.** Actor: CLI. Call: `devforgeai report show <IDEA-nnn> <phase>` where `<phase>` is `plan`, `constitute`, or `design`, taken from the `--remedy` handoff the user pasted. Output: the defect line per cited ID.

C2. **Re-open.** Actor: CLI. Call: `devforgeai doc reopen requirements --id <IDEA-nnn> --ids <ID,ID> --from <phase>`. Output: `revision` raised by 1; a `revision_log` entry appended; every cited `REQ-nnn` moved to `status: reopened`, each staying in the epic that already groups it; one new `REQ-nnn` allocated per cited `UI-nnn` with `source: user`, `traces_to: [<UI-nnn>]`, empty `statement`, `rationale`, and `acceptance_signal`, and grouped by no epic until step 11 places it; every byte of every other record left as it was. `accepted_by` and `accepted_at` return to `null` and the document `status` becomes `reopened`. Failure path: an ID that resolves to no record and matches neither `^REQ-\d{3}$` nor `^UI-\d{3}$` produces exit 3 and ends the run with the binary's stderr on the `Blocked` line.

The run then joins the numbered steps at step 5.

## Subagents

### persona-mapper

- **name**: `persona-mapper`
- **derives_from**: `C:\Users\bryan\.claude\agents\stakeholder-analyst.md` — adapted. The influence tiers and the conflict matrix are dropped, and the AskUserQuestion interviews are not dropped so much as unavailable — Claude Code strips `AskUserQuestion` from every subagent whatever its `tools` list holds, so the tool could not have been kept: the question belongs to the invoking skill, which here is steps 6 to 8 and 13. A persona in `requirements.yaml` carries a role, a description, and a goal, with no power ranking. Prose output is replaced by the JSON below.
- **purpose**: Turn flow statements or a typed description into candidate persona records.
- **tools**: `Read`
- **model**: `sonnet` — extraction from supplied text with a fixed output shape.
- **input**: the `## Target user` lines plus the `Actor` cell of every `## Core flows` row at entry point A, the description string at entry point B, the existing `personas[]` block at entry point C.
- **output**:

```json
{ "type": "object", "required": ["personas"], "additionalProperties": false,
  "properties": {
    "personas": { "type": "array", "minItems": 1, "items": { "type": "object",
      "required": ["key","name","description","goal","from"],
      "additionalProperties": false,
      "properties": {
        "key": { "type": "string", "pattern": "^[a-z0-9]+(-[a-z0-9]+)*$" },
        "name": { "type": "string", "minLength": 1, "maxLength": 40 },
        "description": { "type": "string", "minLength": 1, "maxLength": 200 },
        "goal": { "type": "string", "minLength": 1, "maxLength": 200 },
        "from": { "type": "array", "items": { "type": "string",
          "pattern": "^(FLOW-[0-9]{3}|user)$" } } } } } } }
```

- **invoked_at**: step 5, alone.
- **registered_verifier**: `no`

### requirement-drafter

- **name**: `requirement-drafter`
- **derives_from**: `C:\Users\bryan\.claude\agents\requirements-analyst.md` — adapted, and split with `epic-grouper`. Dropped: story files, Given/When/Then criteria, story points, INVEST, API contracts, data models, and the `Write` and `Edit` tools, because stories belong to Plan (§5) and file writes belong to the skill. `AskUserQuestion` is not dropped either: Claude Code strips `AskUserQuestion` from every subagent whatever its `tools` list holds, so the tool could not have been kept: the question belongs to the invoking skill. Kept: decomposition of a described capability into single-sentence, independently valuable units. Added: the `priority` and `source` fields and the JSON output.
- **purpose**: Turn confirmed personas, outcomes, exclusions, and source text into requirement records.
- **tools**: `Read`
- **model**: `opus` — decomposition boundaries and one-sentence compression carry the judgment this phase exists for.
- **input**: the confirmed persona keys, the outcome set, the exclusion set, and either the flow list, the description, or the reopened records plus the citing report defect lines; at entry point C, the list of `REQ-nnn` to redraft and the instruction that every other record stays as written.
- **output**:

```json
{ "type": "object", "required": ["requirements"], "additionalProperties": false,
  "properties": {
    "requirements": { "type": "array", "minItems": 1, "items": { "type": "object",
      "required": ["key","actor_key","statement","rationale","acceptance_signal",
                   "priority","source","open_questions"],
      "additionalProperties": false,
      "properties": {
        "key": { "type": "string", "pattern": "^([a-z0-9]+(-[a-z0-9]+)*|REQ-[0-9]{3})$" },
        "actor_key": { "type": "string",
          "pattern": "^([a-z0-9]+(-[a-z0-9]+)*|PERSONA-[0-9]{3})$" },
        "statement": { "type": "string", "minLength": 1, "maxLength": 200 },
        "rationale": { "type": "string", "minLength": 1, "maxLength": 200 },
        "acceptance_signal": { "type": "string", "minLength": 1, "maxLength": 200 },
        "priority": { "enum": ["must","should","could","wont"] },
        "source": { "type": "string", "pattern": "^(FLOW-[0-9]{3}|user)$" },
        "open_questions": { "type": "array",
          "items": { "type": "string", "maxLength": 200 } } } } } } }
```

- **invoked_at**: step 9, alone; re-entered from step 13 when the user answers `Redraft requirements`.
- **registered_verifier**: `no`

### epic-grouper

- **name**: `epic-grouper`
- **derives_from**: `C:\Users\bryan\.claude\agents\requirements-analyst.md` — adapted, the second half of the split. Kept: grouping features into epics with a stated boundary. Dropped: every story-level field and every write tool. Added: `out_of_scope`, `success_metric`, and the rule that each requirement ID appears in exactly one epic.
- **purpose**: Partition the requirement set into one epic per selected outcome, each with a scope, an exclusion list, and that outcome as its success metric.
- **tools**: `Read`
- **model**: `sonnet` — partitioning a supplied list against a supplied outcome set.
- **input**: the requirement records with allocated `REQ-nnn` IDs, the outcome set from step 7, and the exclusion set from step 8. The epic count equals the outcome count; the nth epic's `success_metric` is the nth selected outcome. Each selected exclusion is placed on exactly one epic, and an epic drawing none takes the single entry `Nothing was named out of scope at discovery.`
- **output**:

```json
{ "type": "object", "required": ["epics"], "additionalProperties": false,
  "properties": {
    "epics": { "type": "array", "minItems": 1, "items": { "type": "object",
      "required": ["key","title","scope","out_of_scope","success_metric","requirement_ids"],
      "additionalProperties": false,
      "properties": {
        "key": { "type": "string", "pattern": "^([a-z0-9]+(-[a-z0-9]+)*|EPIC-[0-9]{3})$" },
        "title": { "type": "string", "minLength": 1, "maxLength": 60 },
        "scope": { "type": "string", "minLength": 1, "maxLength": 200 },
        "out_of_scope": { "type": "array", "minItems": 1,
          "items": { "type": "string", "maxLength": 200 } },
        "success_metric": { "type": "string", "minLength": 1, "maxLength": 200 },
        "requirement_ids": { "type": "array", "minItems": 1,
          "items": { "type": "string", "pattern": "^REQ-[0-9]{3}$" } } } } } } }
```

- **invoked_at**: step 11, alone, entry points A and B; re-entered from step 13 when the user answers `Regroup epics`. Entry point C invokes it in placement mode only when `revision_log[].added` is non-empty; in that mode it returns one existing `EPIC-nnn` per added `REQ-nnn` and emits no other field.
- **registered_verifier**: `no`

### flow-integrity-auditor

- **name**: `flow-integrity-auditor`
- **derives_from**: `C:\Users\bryan\.claude\agents\context-preservation-validator.md` — adapted. Dropped: the provenance-chain walk and the `<provenance>` tag presence check, both of which `devforgeai doc validate` performs as cross-reference checking (§4), and the strict/non-blocking mode switch, which §6 replaces with SEND BACK. Kept: the read-only posture and the finding-with-evidence output. Added: the two judgments over flow prose that no field check reaches — a statement that names no actor, and a pair of statements that cannot both hold.
- **purpose**: Report `## Core flows` rows whose `Actor` cell names no persona and pairs of rows whose `Trigger`, `Steps`, or `Outcome` cells cannot both hold.
- **tools**: `Read`
- **model**: `opus` — deciding that two prose sentences cannot both hold is the judgment this subagent exists for.
- **input**: every `## Core flows` row as `{id, actor, trigger, steps, outcome}`, the brief path `.devforgeai/explore/brief.md`, and, on the resume form, the `remedied_flows` list from `decision.yaml`.
- **output**:

One JSON object on stdout and nothing else: the `devforgeai/verifier/1` envelope, with this agent's own fields under `payload`.

```json
{ "type": "object",
  "required": ["schema","subagent","id","passed","total","unit","findings","payload"],
  "properties": {
    "schema":   { "const": "devforgeai/verifier/1" },
    "subagent": { "const": "flow-integrity-auditor" },
    "id":       { "type": "string", "pattern": "^IDEA-[0-9]{3}$" },
    "passed":   { "type": "integer", "minimum": 0 },
    "total":    { "type": "integer", "minimum": 0 },
    "unit":     { "const": "flows" },
    "findings": { "type": "array", "maxItems": 0 },
    "payload": { "type": "object",
      "required": ["flows_checked","flows_clean","actorless","contradictions"],
      "additionalProperties": false,
      "properties": {
        "flows_checked": { "type": "integer", "minimum": 0 },
        "flows_clean": { "type": "integer", "minimum": 0 },
        "actorless": { "type": "array", "items": { "type": "object",
          "required": ["flow_id","reason","confidence","evidence"], "additionalProperties": false,
          "properties": {
            "flow_id": { "type": "string", "pattern": "^FLOW-[0-9]{3}$" },
            "reason": { "enum": ["empty","not_a_persona","unlisted_role"] },
            "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
            "evidence": { "type": "string", "maxLength": 200 } } } },
        "contradictions": { "type": "array", "items": { "type": "object",
          "required": ["flow_ids","cells","confidence","evidence"], "additionalProperties": false,
          "properties": {
            "flow_ids": { "type": "array", "minItems": 2, "maxItems": 2,
              "items": { "type": "string", "pattern": "^FLOW-[0-9]{3}$" } },
            "cells": { "type": "array", "minItems": 1,
              "items": { "enum": ["trigger","steps","outcome"] } },
            "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
            "evidence": { "type": "string", "maxLength": 200 } } } } } } } }
```

`total` is `payload.flows_checked` and `passed` is `payload.flows_clean`, both at the top of the envelope where `verifier_pass` reads them. `findings` is `[]`: this phase allocates no `FIND-nnn`, and the two arrays under `payload` carry what the agent found, each entry with a `confidence` from `0.0` to `1.0`. Anything other than one object of this shape is `DFA-E410`, which writes the block at `status: unparsed` and blocks the `SubagentStop` so the envelope is asked for again.

- **invoked_at**: step 4, alone, entry point A and the resume form.
- **registered_verifier**: `yes` — `SubagentStop` ingests it from `last_assistant_message`, and `payload.flows_clean`/`payload.flows_checked` fills the `Verified` line of the handoff.

## Command

The entry point is the skill itself: `skills/discovering-requirements/SKILL.md`, installed to `.claude/skills/discover/`, whose frontmatter `name` is the slash command. There is no command file. This is the frontmatter and the preamble, verbatim, with one preamble line.

```markdown
---
name: discover
description: Phase 1 of DevForgeAI. Turns a promoted Explore brief or a one-line description into .devforgeai/requirements.yaml - personas, epics and requirements, frozen by an explicit user acceptance - through a flow audit, three elicitation rounds and a fourth acceptance round. Reach for it whenever /discover runs, whenever someone asks who the users are, what the requirements are, or what sits in and out of scope for a product about to be planned, and whenever a re-open arrives from Plan, Constitute or Design as /discover IDEA-nnn --remedy REQ-nnn. It owns the artifact too - requirements.yaml with its PERSONA-nnn, EPIC-nnn and REQ-nnn ids, its revision log and its acceptance fields - so read it before touching any of them.
argument-hint: '[IDEA-nnn | "<description>"] [--remedy REQ-nnn,REQ-nnn] [--resume]'
allowed-tools: Bash(devforgeai:*), PowerShell(devforgeai:*), Read, Write, Edit, Agent, AskUserQuestion
disable-model-invocation: true
---

!`devforgeai doc load discover-entry "$ARGUMENTS[0]"`
```

The one preamble line runs `doc load discover-entry "$ARGUMENTS[0]"`, which exits 0 in every case, so what it printed selects the entry point: the brief and the decision record for A, the prior `requirements.yaml` for C, nothing for B. It carries no `gate require` line because the subject is not known until the body decides the entry point, and the predecessor gate is enforced by the `UserPromptExpansion` hook. It allocates no id — entry point B's `IDEA-nnn` is allocated at step 2.

## CLI calls

| Step | Call | Exit codes handled |
|---|---|---|
| command preamble | `devforgeai doc load discover-entry "$ARGUMENTS[0]"` | exit 0 in every case; the printed document, or the absence of one, selects the entry point |
| 2 (entry B) | `devforgeai doc validate --allocate IDEA` | 0 prints `IDEA-nnn`; non-zero ends the run |
| 3 | `devforgeai phase set discover --id <IDEA-nnn>` | 0 continues; 3 ends the run |
| 10 | `devforgeai doc validate --allocate PERSONA` | 0 prints `PERSONA-nnn`; non-zero ends the run |
| 10 | `devforgeai doc validate --allocate REQ` | 0 prints `REQ-nnn`; non-zero ends the run |
| 11 | `devforgeai doc validate --allocate EPIC` | 0 prints `EPIC-nnn`; non-zero ends the run |
| 14 | `devforgeai doc accept requirements --id <IDEA-nnn>` | 0 continues; 1 ends the run |
| C1 | `devforgeai report show <IDEA-nnn> plan` \| `... constitute` \| `... design` | 0 prints the report; 1 ends the run |
| C2 | `devforgeai doc reopen requirements --id <IDEA-nnn> --ids <ID,ID> --from <plan\|constitute\|design>` | 0 continues; 3 ends the run |

`doc accept`, `doc reopen`, and `doc load discover-entry` are accepted §4 additions, with their grammar in `## Decisions`. Discover calls neither `gate require`, `gate check`, `doc validate <path>`, nor `handoff`: the command preamble, the PostToolUse hook, and the Stop hook own those (§4, §7).

## Gate

Entry in `.devforgeai/gates.toml`, verbatim, in the container shape 01-cli.md `## Gate` defines:

```toml
[[gate]]
phase = "discover"
requires = "explore"
on_fail = "fail"
send_back_to = "explore"
document = "requirements.yaml"
description = "Every requirement is complete, every actor resolves, every requirement sits in an epic, the user accepted."

  [[gate.check]]
  kind = "fields_present"
  id = "req-fields-present"
  collection = "requirements"
  fields = ["id", "actor", "statement", "rationale", "acceptance_signal", "priority", "source", "status"]
  message = "requirement {value} is missing a required field"

  [[gate.check]]
  kind = "ids_resolve"
  id = "actor-resolves"
  from = "requirements[].actor"
  to = "personas[].id"
  message = "actor {value} names no persona"

  [[gate.check]]
  kind = "length_between"
  id = "epic-not-empty"
  field = "epics[].requirements"
  min = 1
  exclude_status = ["withdrawn"]
  message = "epic {value} holds no live requirement"

  [[gate.check]]
  kind = "set_cover"
  id = "no-ungrouped-req"
  cover = "epics[].requirements"
  universe = "requirements[].id"
  exclude_status = ["withdrawn"]
  message = "requirement {value} belongs to no epic"

  [[gate.check]]
  kind = "fields_present"
  id = "accepted"
  path = "accepted_by"
  message = "requirements.yaml is not accepted"
```

Gate-level keys are the six 01-cli.md defines: `phase`, `requires`, `on_fail`, `send_back_to`, `document`, `description`. `document` supplies the path every check resolves against, so no check repeats it. No gate-level `report` or `path` key exists; the report path comes from `gate check`.

What each check reads, per the kind table in 01-cli.md `## Outputs`. `fields_present` in its `collection` form passes when every entry of `requirements` carries every name in `fields` with a non-empty value, and in its `path` form, used by `accepted`, passes when the value at `accepted_by` is non-null. `ids_resolve` in its `from`/`to` form passes when every value at `requirements[].actor` equals some value at `personas[].id`. `length_between` passes when every list at `epics[].requirements` holds at least `min` entries after dropping entries whose `status` is in `exclude_status`. `set_cover` passes when the multiset at `epics[].requirements` equals the set at `requirements[].id` minus withdrawn records, with no duplicate.

All five checks take the gate-level `on_fail = "fail"`; none sets a per-check `severity` or `on_fail`, so no check in this gate produces `send_back`. `send_back_to = "explore"` records where a send-back goes when one happens, and the send-back itself is decided at workflow step 4, before this document exists. The `accepted` check tests presence only: it reads whether `accepted_by` holds a value, not who wrote it.

## Send-back

Discover sends back to Explore only, and only from entry point A, and only at step 4.

| Condition | Detected by | IDs cited | Handoff `Next` |
|---|---|---|---|
| A row pair's `Trigger`, `Steps`, or `Outcome` cells cannot both hold | `flow-integrity-auditor`, `contradictions` non-empty | both `FLOW-nnn` of each pair | `/explore <IDEA-nnn> --remedy <FLOW ids>` |
| A row's `Actor` cell is empty, names no persona, or names a role the brief does not mention | `flow-integrity-auditor`, `actorless` non-empty with `reason` of `empty`, `not_a_persona`, or `unlisted_role` | the `FLOW-nnn` | `/explore <IDEA-nnn> --remedy <FLOW ids>` |
| `decision.yaml` holds `kill` or `park` | the `decision` field printed by `doc load discover-entry` | `IDEA-nnn` | none; the `Blocked` line carries the question |

On either auditor condition the skill writes no `requirements.yaml`, and the document at the prior revision, when one exists, keeps every byte. The user returns with `/discover <IDEA-nnn> --resume`, which re-enters at step 4. `gate check --phase discover` exits 2, the Stop hook prints the SEND BACK block, and `state.toml` keeps `[current].phase = "discover"` while `[last_gate]` records `result = "SEND_BACK"`, `send_back_to = "explore"`, and the cited ids in `failed_checks`.

## Integration

| Skill | Consumes (doc, IDs) | Produces for (doc, IDs) | Sends back to (condition) | Receives send-back from (condition) | Shared subagents | `state.toml` read / written |
|---|---|---|---|---|---|---|
| Explore | `explore/brief.md` sections `Core flows` (`FLOW-nnn`), `Problem statement`, `Target user`, `Success signal`, `Non-goals`; `explore/decision.yaml` fields `decision` and `remedied_flows`; id `IDEA-nnn` | none — Explore runs before Discover and reads no Discover output | yes: a row pair contradicts on `Trigger`, `Steps`, or `Outcome`, or a row's `Actor` cell names no persona; cites `FLOW-nnn`; the return trip is `/discover IDEA-nnn --resume` | none — Explore is upstream and cites no Discover ID | none | reads `[current].id` |
| Discover | self | self | self | self | self | reads `[current].phase`, `[current].id`; `phase set discover` writes `[current]` and `[active].discover` |
| Constitute | none — Constitute runs after Discover and emits nothing Discover reads | `requirements.yaml`: `REQ-nnn`, `EPIC-nnn`, `PERSONA-nnn` | none — a defect in `context/*.md` or an ADR is a Constitute-side edit, and Discover reads neither | yes: a `CON-nnn` constraint makes a `REQ-nnn` unachievable; arrives as `/discover IDEA-nnn --remedy REQ-nnn,...`, re-opened with `--from constitute` | none | reads `[current].id`; `revision` written in `requirements.yaml`, not in `state.toml` |
| Plan | `reports/<ID>-plan.yaml` defect lines, entry point C only | `requirements.yaml`: `REQ-nnn`, `EPIC-nnn`, `PERSONA-nnn` | none — Discover does not read stories | yes: a `REQ-nnn` cannot be split into a testable `STORY-nnn`; arrives as `/discover IDEA-nnn --remedy REQ-nnn,...`, re-opened with `--from plan` | none | reads `[current].id` |
| Build | none — Build reads stories and context, not `requirements.yaml` | none — Build reads `STORY-nnn`, which Plan derives from `REQ-nnn` | none — Discover reads no build report | none — Build sends back to Plan (§5) | none | none |
| Verify | none — Verify reads stories and code | none — Verify reads `AC-nnn` on stories | none — Discover reads no QA report | none — Verify sends back to Build and Plan (§5) | none | none |
| Release | none — Release reads QA reports and version data | none | none | none — Release sends back to Verify (§5) | none | none |
| Design | `reports/<ID>-design.yaml` defect lines and `UI-nnn`, entry point C only | `requirements.yaml`: `REQ-nnn` that a `UI-nnn` realizes | none — Discover names no interface element | yes: a `UI-nnn` flow has no `REQ-nnn`; arrives as `/discover IDEA-nnn --remedy UI-nnn,...`, re-opened with `--from design`; step C2 allocates one `REQ-nnn` per cited `UI-nnn` with `traces_to: [UI-nnn]` and `source: user`, step 9 drafts its text, and step 11 places it in an existing epic | none | reads `[current].id` |
| Reflect | none — Discover reads no reflect report | `reports/<ID>-discover.yaml`, written by `gate check`; `OBS-nnn` and `REC-nnn` reference `REQ-nnn` and `revision` | none — §5 records Reflect output as recommendations, not as gates | none — Reflect emits recommendations, not send-backs | none | none |
| CLI | `config.toml`, the `gates.toml` `[[gate]]` whose `phase` is `discover`, `state.toml` | `requirements.yaml` for `doc validate`, `gate check`, `doc load`; `state.toml` through `phase set` | not applicable | not applicable | not applicable | reads `[current].phase`, `[current].id`; `phase set discover` writes `[current].phase`, `[current].id`, `[active].discover`; `gate check` writes `[last_gate]` |

## Handoff

PASS:

```
Phase     1 · Discover        IDEA-004 · payment-reconciliation
Done      3 personas, 4 epics, 17 requirements, revision 1
Gate      PASS  17/17 fields · 17/17 grouped · accepted_by set
Verified  flow-integrity-auditor · 9/9 flows

Next      /constitute IDEA-004
Then      /plan EPIC-001
Blocked   none

Full report: .devforgeai/reports/IDEA-004-discover.yaml
```

SEND BACK:

```
Phase     1 · Discover        IDEA-004 · payment-reconciliation
Done      9 flows read, 0 requirements written
Gate      SEND BACK to Explore  FLOW-003 FLOW-007 FLOW-009
Verified  flow-integrity-auditor · 6/9 flows
Found     FLOW-003 and FLOW-009 disagree on who approves a refund
Found     FLOW-007 names no actor

Next      /explore IDEA-004 --remedy FLOW-003,FLOW-007,FLOW-009
Then      /discover IDEA-004 --resume
Blocked   none

Full report: .devforgeai/reports/IDEA-004-discover.yaml
```

## Templates

### `skills/discovering-requirements/templates/requirements.yaml`

```yaml
schema: devforgeai/requirements/1
id: IDEA-000
phase: discover
status: drafting
produced_by: discovering-requirements
consumes: []
open_questions: []
revision: 1
revision_log: []
accepted_by: null
accepted_at: null

personas:
  - id: PERSONA-000
    name: <role label, 1-40 characters>
    description: <one sentence naming what this role does>
    goal: <one sentence naming what this role wants from the product>

epics:
  - id: EPIC-000
    title: <1-60 characters>
    scope: <one sentence naming what this epic covers>
    out_of_scope:
      - <one sentence naming something this epic leaves out>
    success_metric: <one sentence with one number and one unit>
    requirements:
      - REQ-000

requirements:
  - id: REQ-000
    actor: PERSONA-000
    statement: <one sentence, 1-200 characters>
    rationale: <one sentence naming why the actor wants it>
    acceptance_signal: <one sentence naming one observable result>
    priority: must
    source: user
    status: draft
    open_questions: []
    traces_to: []
```

### `skills/discovering-requirements/templates/questions.md`

```markdown
# Round 1 — actors (workflow step 6)

question: "Which of these roles uses or is affected by <slug>?"
header: "Actors"
multiSelect: true
options, one per candidate from persona-mapper, up to four:
  label: <persona name>
  description: <persona description> + " Wants: " + <persona goal>

Routing: the selected candidates become `personas[]`. A candidate left
unselected is dropped from the run. A reply naming a role outside the options
adds one persona record with that role as `name`, and the model writes its
`description` and `goal` from the same reply.
An empty selection re-asks this question with the same options.

# Round 2 — outcomes (workflow step 7)

question: "Which of these has to be true for <slug> to be worth building?"
header: "Outcomes"
multiSelect: true
options, three to four drafted from the flow statements or the description:
  label: <outcome, up to 40 characters>
  description: <the same outcome as one sentence with one number and one unit>

Routing: step 11 emits one epic per selected outcome, and that outcome is the
epic's `success_metric`. An empty selection re-asks this question with the same
options.

# Round 3 — boundaries (workflow step 8)

question: "Which of these is outside <slug> for this round?"
header: "Boundaries"
multiSelect: true
options, three to four drafted from the flow statements or the description:
  label: <exclusion, up to 40 characters>
  description: <the same exclusion as one sentence>

Routing: step 11 places each selected exclusion on exactly one epic as an
`out_of_scope` entry. An epic drawing no exclusion, and every epic when the
selection is empty, takes the single entry
"Nothing was named out of scope at discovery."

Rounds 1, 2, and 3 run at entry points A and B. Entry point C skips all three.

# Round 4 — acceptance (workflow step 13)

question: "Accept these <n> epics and <m> requirements as the requirement set for <slug>?"
header: "Accept epics"
multiSelect: false
options:
  label: "Accept"
  description: "Freeze this set. Constitute and Plan read it next."
  label: "Regroup epics"
  description: "Keep every requirement and its text. Rebuild only the epic grouping."
  label: "Redraft requirements"
  description: "Name the REQ ids to rewrite in your reply. Every other id keeps its text."

Routing:
  "Accept" -> workflow step 14.
  "Regroup epics" -> workflow step 11 with the epic block cleared, then this
    question again.
  "Redraft requirements" -> workflow step 9 for the REQ ids in the reply, then
    this question again. A reply carrying no REQ id re-asks this question
    unchanged.
```

## Evals

### `skills/discovering-requirements/evals/evals.json`

```json
{
  "skill": "discovering-requirements",
  "evals": [
    {
      "prompt": "/discover IDEA-004",
      "expected_output": ".devforgeai/requirements.yaml holding 2 personas, 2 epics, 5 requirements, accepted_by user",
      "expectations": [
        "requirements.yaml has top-level keys schema, id, phase, status, produced_by, consumes, open_questions, revision, revision_log, accepted_by, accepted_at, personas, epics, requirements",
        "every requirements[].actor matches a personas[].id",
        "every requirements[].source is a FLOW id listed in consumes",
        "every requirements[].id appears in exactly one epics[].requirements list",
        "revision is 1 and revision_log is empty"
      ]
    },
    {
      "prompt": "/discover \"a tool that reconciles card settlements against the ledger each night\"",
      "expected_output": ".devforgeai/requirements.yaml with consumes [] and every source set to user",
      "expectations": [
        "the document id matches ^IDEA-\\d{3}$",
        "consumes is the empty list",
        "every requirements[].source equals user",
        "every epics[].out_of_scope holds at least one entry",
        "no .devforgeai/explore file is written"
      ]
    },
    {
      "prompt": "/discover IDEA-011",
      "expected_output": "SEND BACK to Explore citing the contradicting FLOW ids",
      "expectations": [
        "no .devforgeai/requirements.yaml is written",
        "the handoff Gate line reads SEND BACK to Explore",
        "the handoff Next line is /explore IDEA-011 --remedy with both FLOW ids",
        "the Found lines name both FLOW ids of the contradicting pair"
      ]
    },
    {
      "prompt": "/discover IDEA-012",
      "expected_output": "SEND BACK to Explore citing the FLOW id that names no actor",
      "expectations": [
        "no .devforgeai/requirements.yaml is written",
        "the handoff Gate line reads SEND BACK to Explore",
        "the Found line names the actorless FLOW id",
        "the Verified line names flow-integrity-auditor"
      ]
    },
    {
      "prompt": "/discover IDEA-004 --remedy REQ-007,REQ-011",
      "expected_output": "revision 2, REQ-007 and REQ-011 rewritten, every other record byte-identical",
      "expectations": [
        "revision is 2 and revision_log holds one entry with from plan",
        "REQ-007 and REQ-011 carry text different from the prior revision",
        "every requirement other than REQ-007 and REQ-011 is byte-identical to the prior revision",
        "no REQ id present before the remedy is absent after it"
      ]
    },
    {
      "prompt": "/discover IDEA-004 --remedy UI-005",
      "expected_output": "one new REQ allocated for UI-005 with traces_to [UI-005]",
      "expectations": [
        "revision is 2 and revision_log holds one entry with from design",
        "one requirement carries traces_to equal to [UI-005]",
        "that requirement has source user and a non-empty statement",
        "that requirement id appears in the requirements list of exactly one epic"
      ]
    }
  ]
}
```

### `skills/discovering-requirements/evals/cases.jsonl`

```jsonl
{"id": "disc-brief-happy", "prompt": "/discover IDEA-004", "answers": {"Actors": ["@first"], "Outcomes": ["@all"], "Boundaries": ["@first"], "Accept epics": "Accept"}, "setup": {"files": {".devforgeai/explore/brief.md": "---\nschema: devforgeai/explore-brief/1\nid: IDEA-004\nphase: explore\nstatus: decided\nproduced_by: exploring-ideas\nconsumes: []\nopen_questions: []\n---\n\n## Problem statement\n\nA settlement clerk loses two hours a night matching card settlements to the ledger by hand.\n\n## Target user\n\nBack-office settlement clerks at mid-size acquirers.\n- Reconciles 400 to 900 settlement lines a night.\n- Works from a bank export and an accounting export.\n\n## Core flows\n\n| ID | Actor | Trigger | Steps | Outcome |\n|---|---|---|---|---|\n| FLOW-001 | Settlement clerk | The nightly settlement file lands | Open the file -> upload it -> read the match count | Every settlement line is matched or flagged |\n| FLOW-002 | Settlement clerk | The match run finishes | Open the exception list -> sort by amount -> pick a line | The clerk sees which lines have no ledger match |\n| FLOW-003 | Finance lead | A line stays unmatched for two nights | Open the line -> read the history -> approve a write-off | The line leaves the exception list |\n\n## Non-goals\n\nThis product excludes chargeback dispute handling.\nThis product excludes ledger posting.\n\n## Success signal\n\nUnmatched settlement lines at 09:00 | below 5 | one week\n", ".devforgeai/explore/decision.yaml": "schema: devforgeai/explore-decision/1\nid: IDEA-004\nphase: explore\nstatus: recorded\nproduced_by: exploring-ideas\nconsumes: []\nopen_questions: []\ndecision: promote\ndecided_on: 2026-09-08\nreason: >-\n  Four of five clerks shown the FLOW-001 mockup asked for a pilot,\n  and both competitors found charge per seat for the manual version.\nrevisit_on: null\nelapsed_days: 4\nremedied_flows: []\ncarry_forward: []\n", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml"}}, "expect": {"grader": "requirements_document_wellformed", "args": {"min_personas": 1, "min_epics": 1, "min_requirements": 3, "expect_consumes": ["IDEA-004", "FLOW-001", "FLOW-002", "FLOW-003"]}}}
{"id": "disc-description-happy", "prompt": "/discover \"a tool that reconciles card settlements against the ledger each night\"", "answers": {"Actors": ["@first"], "Outcomes": ["@all"], "Boundaries": ["@first"], "Accept epics": "Accept"}, "setup": {"files": {".devforgeai/config.toml": "schema = \"devforgeai/config/1\"\ngenerated_at = \"2026-09-10T09:00:00Z\"\ncli_version = \"1.0.0\"\ndegraded = false\n[project]\nname = \"recon\"\n", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml"}}, "expect": {"grader": "requirements_document_wellformed", "args": {"min_personas": 1, "min_epics": 1, "min_requirements": 3, "expect_consumes": [], "expect_all_sources": "user"}}}
{"id": "disc-sendback-contradiction", "prompt": "/discover IDEA-011", "answers": {"Actors": ["@first"], "Outcomes": ["@all"], "Boundaries": ["@first"], "Accept epics": "Accept"}, "setup": {"files": {".devforgeai/explore/brief.md": "---\nschema: devforgeai/explore-brief/1\nid: IDEA-011\nphase: explore\nstatus: decided\nproduced_by: exploring-ideas\nconsumes: []\nopen_questions: []\n---\n\n## Problem statement\n\nA settlement clerk loses two hours a night matching card settlements to the ledger by hand.\n\n## Target user\n\nBack-office settlement clerks at mid-size acquirers.\n- Reconciles 400 to 900 settlement lines a night.\n- Works from a bank export and an accounting export.\n\n## Core flows\n\n| ID | Actor | Trigger | Steps | Outcome |\n|---|---|---|---|---|\n| FLOW-001 | Settlement clerk | The nightly settlement file lands | Open the file -> upload it | The file is loaded |\n| FLOW-003 | Finance lead | A refund is requested | Open the refund -> approve it | Only a finance lead approves a refund |\n| FLOW-009 | Settlement clerk | A refund is requested | Open the refund -> approve it | A clerk approves a refund with no finance lead |\n\n## Non-goals\n\nThis product excludes chargeback dispute handling.\nThis product excludes ledger posting.\n\n## Success signal\n\nUnmatched settlement lines at 09:00 | below 5 | one week\n", ".devforgeai/explore/decision.yaml": "schema: devforgeai/explore-decision/1\nid: IDEA-011\nphase: explore\nstatus: recorded\nproduced_by: exploring-ideas\nconsumes: []\nopen_questions: []\ndecision: promote\ndecided_on: 2026-09-08\nreason: >-\n  Four of five clerks shown the FLOW-001 mockup asked for a pilot,\n  and both competitors found charge per seat for the manual version.\nrevisit_on: null\nelapsed_days: 4\nremedied_flows: []\ncarry_forward: []\n", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml"}}, "expect": {"grader": "sendback_to_explore", "args": {"cited": ["FLOW-003", "FLOW-009"], "absent_path": ".devforgeai/requirements.yaml"}}}
{"id": "disc-sendback-actorless", "prompt": "/discover IDEA-012", "answers": {"Actors": ["@first"], "Outcomes": ["@all"], "Boundaries": ["@first"], "Accept epics": "Accept"}, "setup": {"files": {".devforgeai/explore/brief.md": "---\nschema: devforgeai/explore-brief/1\nid: IDEA-012\nphase: explore\nstatus: decided\nproduced_by: exploring-ideas\nconsumes: []\nopen_questions: []\n---\n\n## Problem statement\n\nA settlement clerk loses two hours a night matching card settlements to the ledger by hand.\n\n## Target user\n\nBack-office settlement clerks at mid-size acquirers.\n- Reconciles 400 to 900 settlement lines a night.\n- Works from a bank export and an accounting export.\n\n## Core flows\n\n| ID | Actor | Trigger | Steps | Outcome |\n|---|---|---|---|---|\n| FLOW-001 | Settlement clerk | The nightly settlement file lands | Open the file -> upload it | The file is loaded |\n| FLOW-004 |  | The match run finishes | Build the receipt -> send it | A receipt is emailed |\n| FLOW-005 | Settlement clerk | The match run finishes | Open the exception list -> pick a line | The clerk sees unmatched lines |\n\n## Non-goals\n\nThis product excludes chargeback dispute handling.\nThis product excludes ledger posting.\n\n## Success signal\n\nUnmatched settlement lines at 09:00 | below 5 | one week\n", ".devforgeai/explore/decision.yaml": "schema: devforgeai/explore-decision/1\nid: IDEA-012\nphase: explore\nstatus: recorded\nproduced_by: exploring-ideas\nconsumes: []\nopen_questions: []\ndecision: promote\ndecided_on: 2026-09-08\nreason: >-\n  Four of five clerks shown the FLOW-001 mockup asked for a pilot,\n  and both competitors found charge per seat for the manual version.\nrevisit_on: null\nelapsed_days: 4\nremedied_flows: []\ncarry_forward: []\n", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml"}}, "expect": {"grader": "sendback_to_explore", "args": {"cited": ["FLOW-004"], "absent_path": ".devforgeai/requirements.yaml"}}}
{"id": "disc-remedy-plan", "prompt": "/discover IDEA-004 --remedy REQ-007,REQ-011", "answers": {"Accept epics": "Accept", "*": "@first"}, "setup": {"files": {".devforgeai/requirements.yaml": "schema: devforgeai/requirements/1\nid: IDEA-004\nphase: discover\nstatus: accepted\nproduced_by: discovering-requirements\nconsumes: [IDEA-004]\nopen_questions: []\nrevision: 1\nrevision_log: []\naccepted_by: user\naccepted_at: 2026-09-01T09:00:00Z\npersonas:\n  - id: PERSONA-001\n    name: Settlement clerk\n    description: Loads the nightly settlement file and clears exceptions.\n    goal: Close every settlement night with no unmatched line.\nepics:\n  - id: EPIC-002\n    title: Exception clearing\n    scope: Everything a clerk does with an unmatched settlement line.\n    out_of_scope:\n      - Chargeback disputes are handled outside this product.\n    success_metric: Unmatched lines fall below 5 per night.\n    requirements: [REQ-007, REQ-011, REQ-014]\nrequirements:\n  - id: REQ-007\n    actor: PERSONA-001\n    statement: The clerk clears an unmatched line.\n    rationale: Unmatched lines block the nightly close.\n    acceptance_signal: The line leaves the exception list.\n    priority: must\n    source: user\n    status: accepted\n    open_questions: []\n    traces_to: []\n  - id: REQ-011\n    actor: PERSONA-001\n    statement: The clerk splits a settlement line.\n    rationale: One settlement line can cover two ledger entries.\n    acceptance_signal: Two lines appear where one was.\n    priority: should\n    source: user\n    status: accepted\n    open_questions: []\n    traces_to: []\n  - id: REQ-014\n    actor: PERSONA-001\n    statement: The clerk exports the exception list.\n    rationale: Finance reviews exceptions outside the product.\n    acceptance_signal: A file holding every open exception is produced.\n    priority: could\n    source: user\n    status: accepted\n    open_questions: []\n    traces_to: []\n", ".devforgeai/reports/IDEA-004-plan.yaml": "schema: devforgeai/report/1\nid: IDEA-004\nphase: plan\nfindings:\n  - id: REQ-007\n    defect: clears is not one testable action\n  - id: REQ-011\n    defect: splits names no rule for the split amounts\n", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml"}}, "expect": {"grader": "remedy_preserved_other_requirements", "args": {"reopened": ["REQ-007", "REQ-011"], "untouched": ["REQ-014"], "expect_revision": 2, "expect_from": "plan"}}}
{"id": "disc-remedy-design", "prompt": "/discover IDEA-004 --remedy UI-005", "answers": {"Accept epics": "Accept", "*": "@first"}, "setup": {"files": {".devforgeai/requirements.yaml": "schema: devforgeai/requirements/1\nid: IDEA-004\nphase: discover\nstatus: accepted\nproduced_by: discovering-requirements\nconsumes: [IDEA-004]\nopen_questions: []\nrevision: 1\nrevision_log: []\naccepted_by: user\naccepted_at: 2026-09-01T09:00:00Z\npersonas:\n  - id: PERSONA-001\n    name: Settlement clerk\n    description: Loads the nightly settlement file and clears exceptions.\n    goal: Close every settlement night with no unmatched line.\nepics:\n  - id: EPIC-002\n    title: Exception clearing\n    scope: Everything a clerk does with an unmatched settlement line.\n    out_of_scope:\n      - Chargeback disputes are handled outside this product.\n    success_metric: Unmatched lines fall below 5 per night.\n    requirements: [REQ-007]\nrequirements:\n  - id: REQ-007\n    actor: PERSONA-001\n    statement: The clerk marks an unmatched line as written off.\n    rationale: Unmatched lines block the nightly close.\n    acceptance_signal: The line leaves the exception list.\n    priority: must\n    source: user\n    status: accepted\n    open_questions: []\n    traces_to: []\n", ".devforgeai/ui-specs/UI-005.md": "---\nschema: devforgeai/ui-spec/1\nid: UI-005\nphase: design\nstatus: drafted\nproduced_by: designing-interfaces\nconsumes: []\nopen_questions: []\n---\n\n# Bulk write-off drawer\n\nA clerk selects many exception lines and writes them off in one action.\n", ".devforgeai/reports/IDEA-004-design.yaml": "schema: devforgeai/report/1\nid: IDEA-004\nphase: design\nfindings:\n  - id: UI-005\n    defect: bulk write-off drawer maps to no requirement\n", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml"}}, "expect": {"grader": "design_remedy_created_requirement", "args": {"ui_id": "UI-005", "expect_revision": 2, "untouched": ["REQ-007"]}}}
```

### `skills/discovering-requirements/evals/graders.py`

```python
def requirements_document_wellformed(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    """Load .devforgeai/requirements.yaml. Pass when: the thirteen top-level keys
    are present in template order; personas/epics/requirements meet args'
    min_personas/min_epics/min_requirements; every requirements[].actor is a
    personas[].id; every requirements[].id appears in exactly one
    epics[].requirements; every epics[] entry has a non-empty out_of_scope and a
    success_metric containing a digit; consumes equals args['expect_consumes'];
    and when args carries 'expect_all_sources', every requirements[].source
    equals it. Evidence: the first failing rule with the offending id, or the
    counts on pass."""


def sendback_to_explore(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    """Pass when args['absent_path'] does not exist in the workspace, the
    transcript holds a line starting 'Gate' whose text contains 'SEND BACK to
    Explore', a line starting 'Next' containing '--remedy', and every id in
    args['cited'] appears on a line starting 'Found' or on the Next line.
    Evidence: the Gate, Found, and Next lines, or the missing element."""


def remedy_preserved_other_requirements(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    """Read the pre-run copy of .devforgeai/requirements.yaml recorded by the
    runner and the post-run file. Serialize each requirements[] entry to a
    canonical string and hash it, and do the same for every personas[] and
    epics[] entry. Pass when: post revision equals args['expect_revision'];
    revision_log holds exactly one new entry whose 'from' equals
    args['expect_from'] and whose 'reopened' equals args['reopened']; every id in
    args['untouched'] has an identical hash before and after; every id in
    args['reopened'] has a different hash; every personas[] entry and every
    epics[] entry hashes identically before and after; the id set is unchanged;
    and accepted_by is non-null. Evidence: the id-to-hash diff."""


def design_remedy_created_requirement(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    """Read the pre-run and post-run .devforgeai/requirements.yaml. Pass when:
    post revision equals args['expect_revision']; revision_log's new entry has
    from 'design' and a one-element 'added' list; exactly one requirements[]
    entry has traces_to equal to [args['ui_id']]; that entry has source 'user',
    a non-empty statement, rationale, and acceptance_signal, and its id appears
    in exactly one epics[].requirements list; every id in args['untouched']
    hashes identically before and after; every personas[] entry hashes
    identically; and every epics[] field other than the one 'requirements' list
    that gained the new id hashes identically. Evidence: the new id, the epic
    that took it, and its fields."""


def personas_resolve(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    """Pass when every requirements[].actor resolves to a personas[].id and every
    personas[].id is referenced by at least one requirement. Evidence: the
    unresolved actor ids and the unreferenced persona ids."""


def acceptance_recorded(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    """Pass when accepted_by equals 'user', accepted_at matches
    ^\\d{4}-\\d{2}-\\d{2}T\\d{2}:\\d{2}:\\d{2}Z$, document status equals
    'accepted', and every requirements[].status is 'accepted' or 'withdrawn'.
    Evidence: the four field values."""
```

## Decisions

1. **Document id.** `requirements.yaml` carries `id: IDEA-nnn`. At entry point A it is the brief's id. At entry point B `devforgeai doc validate --allocate IDEA` prints a fresh one and no `.devforgeai/explore/` file is created; the allocated id threads through `state.toml`, `gate check --id`, and `reports/<ID>-discover.yaml`.
2. **Remedy and resume key on the phase id.** Per §4c the forms are `/discover IDEA-nnn --remedy REQ-014,REQ-019` and `/discover IDEA-nnn --resume`. A cited `REQ-nnn` already identifies its epic through `epics[].requirements`, so no epic argument is taken and the report path and the `state.toml` active id stay constant across revisions. A new `REQ-nnn` created for a cited `UI-nnn` belongs to no epic until step 11 places it, and the gate's `no-ungrouped-req` check is what makes that placement non-optional.
3. **`gate require` omitted.** Discover is an entry phase with no required predecessor, so `/discover` calls neither `gate require` nor `gate check`. Requested clarification to §4: `phase set <phase> --id <id>` refuses only when a predecessor gate exists and failed, so `phase set discover` succeeds at entry point B where no Explore gate exists. Written as a clarification, not a redefinition of `gate require`.
4. **Two accepted CLI additions, per §4.** Both are accepted for 01-cli.md, with this argument grammar verbatim.

   `devforgeai doc accept requirements --id <IDEA-nnn>` writes `accepted_by: user`, `accepted_at` in RFC 3339 UTC, the document `status: accepted`, and moves every requirement at `draft` or `reopened` to `accepted`. Exit 0 on success; exit 1 when `epics[]` or `requirements[]` is empty.

   `devforgeai doc reopen requirements --id <IDEA-nnn> --ids <ID,ID> --from <plan|constitute|design>` raises `revision` by 1, appends the `revision_log` entry, moves each cited `REQ-nnn` to `reopened` in place, allocates one `REQ-nnn` per cited `UI-nnn` with `source: user` and `traces_to: [<UI-nnn>]` and no epic, returns `accepted_by` and `accepted_at` to null, sets the document `status: reopened`, and leaves every other byte untouched. Exit 0 on success; exit 3 when a cited id matches neither `^REQ-\d{3}$` nor `^UI-\d{3}$`.

   Both exist so that the timestamp and the byte-identity guarantee come from the binary rather than from model editing (§1 rule 1).

5. **One accepted CLI addition: `doc load discover-entry <arg>`.** The command preamble runs unconditionally, so a preamble that calls `doc load explore` fails visibly at entry point B, where `$1` is a quoted description rather than an id. `devforgeai doc load discover-entry <arg>` exits 0 in every case: it prints `.devforgeai/explore/brief.md` and `.devforgeai/explore/decision.yaml` when `<arg>` is an `IDEA-nnn` with a brief on disk, prints `.devforgeai/requirements.yaml` when `<arg>` is an `IDEA-nnn` with requirements on disk and no brief, prints both when both exist, and prints nothing when `<arg>` matches no id. This keeps the failure silent for the model, which is what makes a one-line preamble safe across three entry points.

6. **`--resume` re-enters at step 4.** §4c fixes the flag name. Step 4 is the only step a run stops at without writing a document, so `--resume` needs no stored cursor: it re-runs the flow audit for `$1` against the brief Explore rewrote, and `decision.yaml` `remedied_flows` names the rows that changed.

7. **Priority enum is MoSCoW: `must`, `should`, `could`, `wont`.** The rename to `required | expected | optional | excluded` was made to dodge the §2 ceremony pattern, and that was a lint defect rather than a design judgment: under the scoped rule a YAML scalar is not instruction prose and matches nothing. MoSCoW is the vocabulary a stakeholder reading `requirements.yaml` already knows, and the rename lost the distinction its third and fourth levels carry — `could` is a nice-to-have while `excluded` reads as a prohibition rather than as out-of-scope-for-now.
8. **`source` enum left as given.** `source` holds `user` or a `FLOW-nnn`. A requirement created from a Design send-back records `source: user` and carries the interface reference in the separate `traces_to` list, so nothing downstream that reads `source` as a flow reference has to learn a third form.
9. **Send-back detection is one subagent.** `flow-integrity-auditor` owns both Explore-facing conditions, because both are judgments over prose: whether a sentence names an actor, and whether two sentences can both hold. A single verifier also keeps the handoff to one `Verified` line, as §6 shows.
10. **`accepted_by` is a one-member enum.** The only acceptance channel is the round 4 AskUserQuestion, so the only value is `user`. The gate tests presence, not authorship.
11. **Withdrawn requirements stay in the file.** A requirement the user drops keeps its id and text at `status: withdrawn` and stays in its epic's list. The `set_cover` and `length_between` checks carry `exclude_status = ["withdrawn"]` so a fully withdrawn set does not pass the grouping checks on paper.
12. **Elicitation stops at three rounds.** Rounds 1 to 3 run at entry points A and B alike, seeded by `persona-mapper` and by the model's reading of the flows or the description. Entry point C skips all three, and skips `epic-grouper`, so a re-open changes only the cited records and the named epic's `requirements` list. Round 4 is acceptance at every entry point. An unknown surviving round 3 becomes a `requirements[].open_questions` entry. The bound is fixed rather than judgment-based so that a run's question count is predictable.

13. **The outcome set is the epic partition key.** `epic-grouper` emits exactly one epic per outcome selected in round 2, and the nth epic's `success_metric` is the nth selected outcome. This is what keeps a required field from having no producer when the user selects fewer outcomes than the grouper would otherwise have emitted epics. Exclusions from round 3 are placed one per epic, and an epic drawing none takes the fixed entry `Nothing was named out of scope at discovery.`

14. **Top-level `open_questions` is derived, not authored.** Step 12 writes it as the ascending-ID list of every `REQ-nnn` whose own `open_questions` is non-empty. §5 fixes the key's presence and leaves its contents to this spec.

15. **The Explore shape is read from 02-explore.md, not assumed.** `## Inputs` names the six parts of `brief.md` Discover reads and the two fields of `decision.yaml`. `## Core flows` carries an explicit `Actor` column, so an empty cell is a field-level defect that `devforgeai doc validate` catches on the Explore side; `flow-integrity-auditor` reports it as `reason: empty` when it survives that far, and owns the judgment case, `reason: not_a_persona`, where the cell holds text that names no role. Contradictions compare the `Trigger`, `Steps`, and `Outcome` cells of a row pair, and the auditor names which of the three in its `cells` field. Review criterion (h) is satisfied against 02-explore.md `## Outputs`.
16. **The gate entry is 01-cli.md's, verbatim.** `## Gate` reproduces the `[[gate]]` whose `phase` is `discover` from 01-cli.md `## Gate`, with the container keys and the four kinds that spec defines: `fields_present` in both its `collection` and `path` forms, `ids_resolve` in its `from`/`to` form, `length_between`, and `set_cover`. The five check ids are unchanged, so nothing in `## Evals` moves.
17. **The `state.toml` keys are 01-cli.md's.** Discover reads `[current].phase` and `[current].id`. `phase set discover --id <IDEA-nnn>` writes `[current].phase`, `[current].id`, and `[active].discover`. `gate check` writes `[last_gate]`, whose `result` enum carries `SEND_BACK` for the Explore send-back and whose `failed_checks` holds the failing check ids. Discover writes no phase-scoped table: 01-cli.md reserves `[discover]` as a table name and gives it no key in v1.
18. **Existing agents not carried into this phase.**

| Agent | Disposition | Reason |
|---|---|---|
| `requirements-analyst` | adapted, split | Becomes `requirement-drafter` and `epic-grouper`. Its story files, Given/When/Then criteria, story points, INVEST pass, API contracts, and data models belong to Plan (§5). Its `Write` and `Edit` tools are removed, because the skill writes the file. `AskUserQuestion` is removed because it could not have been kept: Claude Code strips `AskUserQuestion` from every subagent whatever its `tools` list holds, so the tool could not have been kept: the question belongs to the invoking skill. |
| `stakeholder-analyst` | adapted | Becomes `persona-mapper`. Influence tiers and the conflict matrix are dropped, and its own AskUserQuestion interviews could not have been kept: Claude Code strips `AskUserQuestion` from every subagent whatever its `tools` list holds, so the tool could not have been kept: the question belongs to the invoking skill. A persona record carries a role, a description, and a goal. |
| `context-preservation-validator` | adapted | Becomes `flow-integrity-auditor`. Its provenance-chain walk and tag-presence check are what `devforgeai doc validate` does as cross-reference checking (§4 and §1 rule 1); its strict/non-blocking switch is replaced by SEND BACK (§6). |
| `ideation-result-interpreter` | replaced | Its box-drawing summary and next-step table are the §6 handoff, which `devforgeai handoff` prints (§1 rule 3). No subagent composes it. |
| `internet-sleuth` | unused | Market research, competitive analysis, and technology evaluation are Explore's work, and this phase names no technology (§1 rule 2). Its research reports are read by no Discover step. |
| `business-coach` | unused | Coaching produces no typed document and no field of `requirements.yaml` derives from it. |
| `entrepreneur-assessor` | unused | A self-reported work-style profile is an input to no field of `requirements.yaml`. |

19. **Blockers.** None. `doc accept`, `doc reopen`, and `doc load discover-entry` are accepted §4 additions with the grammar in decisions 4 and 5. The Explore document shape is fixed by 02-explore.md `## Outputs`, and the gate and state keys are fixed by 01-cli.md.
