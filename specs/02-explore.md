---
schema: devforgeai-spec/1
doc: explore
status: draft
produced_by: explore-spec-author
consumes: [00-conventions]
open_questions: []
---

# Phase 0 · Explore · `exploring-ideas`

## Scope

`exploring-ideas` is a time-boxed loop that carries one half-formed idea from a sentence to a recorded `kill`, `park`, or `promote` decision. It runs six steps in a fixed order: brainstorm, competitor and technology scan, a one-page lightweight spec, mockups of the core flows, an optional clickable prototype, and the decision. It emits two documents, `.devforgeai/explore/brief.md` and `.devforgeai/explore/decision.yaml`, and one throwaway directory, `.explore-prototype/`. It is the entry point of the framework and the only optional phase: a project may start at Phase 1 with a brief the user wrote by hand.

`exploring-ideas` is not a requirements phase, an architecture phase, or a build phase. It writes no context files, no ADRs, no stories, and no tests. It produces no production code: everything under `.explore-prototype/` is deleted by the CLI on `kill` and on `promote`, and its source is not carried into any later phase. It does not draw mockups itself; it calls the `designing-interfaces` skill in sketch mode under the contract in `## Outputs`. It emits no SEND BACK, because §5 gives its "May send back to" column as `none`; it is the receiving end of a send-back from Discover.

## Inputs

| Input | Path | IDs | Read by | When |
|---|---|---|---|---|
| The user's idea, one line of free text | `$ARGUMENTS` of `/explore` | none | step 1 | fresh run |
| Next free idea ID | stdout of `devforgeai doc validate --allocate IDEA` | `IDEA-nnn` | step 1 | fresh run |
| Prior brief | `.devforgeai/explore/brief.md` | `IDEA-nnn`, `FLOW-nnn` | step 1 | remedy run and resume |
| Prior decision | `.devforgeai/explore/decision.yaml` | `IDEA-nnn` | step 1 | remedy run and resume |
| Discover send-back report | `.devforgeai/reports/IDEA-nnn-discover.yaml` | `FLOW-nnn` cited in its `findings[]` | step 1 | remedy run |
| Phase state | `.devforgeai/state.toml`, keys `[current].phase`, `[current].id`, `[active].explore`, `[explore].*` | `IDEA-nnn` | steps 1 and 10 | every run |
| Time box length | `.devforgeai/config.toml`, `[explore] timebox_days`, `[explore] remedy_timebox_days` | none | step 1 | every run |

Explore reads no document produced by another phase on a fresh run. On a remedy run it reads one: the Discover report named above, which is a CLI-written report, not a §5 phase document.

## Outputs

### 1. `.devforgeai/explore/brief.md`

Frontmatter, in this order, no other top-level keys:

```yaml
---
schema: devforgeai/explore-brief/1
id: IDEA-001
phase: explore
status: specified
produced_by: exploring-ideas
consumes: []
open_questions: []
---
```

`status` enum, in progression order: `drafting` (step 2 done), `scanned` (step 3 done), `specified` (steps 4 and 5 done), `mocked` (step 6 done), `decided` (step 10 done). `consumes` is `[]` on every run, including a remedy run, because Explore reads no §5 phase document.

Sections, in this order, with these headings verbatim:

| # | Heading | Content |
|---|---|---|
| 1 | `## Problem statement` | One sentence. Who loses what, and how often. |
| 2 | `## Target user` | One line naming the segment, then 2 to 4 lines, one attribute per line, each an observable fact. |
| 3 | `## What they do today` | Table, columns `Current approach \| Cost \| Where it breaks`. One row per approach, 1 to 4 rows. |
| 4 | `## Why now` | 1 to 3 lines. Each line is a dated change in the world, formatted `YYYY-MM — <change>`. |
| 5 | `## Competitor scan` | Table, columns `Name \| URL \| Approach \| Price \| Gap`. 0 to 8 rows. `Gap` is the part of the problem statement that competitor leaves unsolved. |
| 6 | `## Technology scan` | Table, columns `Capability \| Candidate \| Maturity \| License \| Source`. 0 to 8 rows. `Maturity` is one of `established`, `emerging`, `experimental`. `Source` is a URL. |
| 7 | `## Core flows` | Table, columns `ID \| Actor \| Trigger \| Steps \| Outcome`. 3 to 5 rows. `ID` is `FLOW-nnn`, zero-padded, allocated in row order starting at `FLOW-001`. `Steps` is one line, steps joined with ` -> `. |
| 8 | `## Non-goals` | Unnumbered lines, one per line, 1 to 10 lines. Each states a capability this idea excludes, in the present tense. |
| 9 | `## Success signal` | One line, format `<metric> \| <threshold> \| <observation window>`. Exactly one line. |
| 10 | `## Mockups` | Table, columns `Flow \| Screen \| Path \| State`. One row per screen returned by the sketch-mode contract. Empty table with header row when step 6 is skipped. |
| 11 | `## Seed data` | Table, columns `Entity \| Rows \| Field count`. One row per entity in `.devforgeai/explore/seed-data.json`. |
| 12 | `## Prototype` | Table, columns `Field \| Value`, with exactly these four rows: `Built` (`yes` or `no`), `Path` (`.explore-prototype/` or `-`), `Entry` (`.explore-prototype/index.html` or `-`), `Flows covered` (comma-separated `FLOW-nnn` list or `-`). |

Open questions live in the frontmatter `open_questions` list and nowhere else in the document.

### 2. `.devforgeai/explore/decision.yaml`

Exact content. `#` comments below are annotation for this spec and are absent from the written file.

```yaml
schema: devforgeai/explore-decision/1
id: IDEA-001
phase: explore
status: recorded
produced_by: exploring-ideas
consumes: []
open_questions: []
decision: promote                 # one of: kill, park, promote
decided_on: 2026-09-10            # YYYY-MM-DD, the local date the user answered step 9
reason: >-                        # one sentence, 15 to 40 words, names the evidence
  Three of four people shown the FLOW-001 mockup asked when they could pay for it,
  and the two competitors found charge per seat for the manual version of the same flow.
revisit_on: null                  # YYYY-MM-DD when decision is park; null when decision is kill or promote
elapsed_days: 4                   # integer, whole days from explore.started_at to decided_on
remedied_flows: []                # FLOW-nnn ids rewritten by the latest remedy run; [] on a fresh run
carry_forward: []                 # [] when decision is kill or park; the seven entries below when decision is promote
```

`status` enum: `recorded` is the only value. The document is written once per run and overwritten on a remedy run.

`carry_forward` when `decision: promote`, exactly these seven entries in this order:

```yaml
carry_forward:
  - path: .devforgeai/explore/brief.md
    sections: [Problem statement, Target user]
    becomes: discovery brief and first persona
    consumer: discovering-requirements
  - path: .devforgeai/explore/brief.md
    sections: [Core flows]
    becomes: epic candidates with draft acceptance criteria
    consumer: discovering-requirements
  - path: .devforgeai/explore/brief.md
    sections: [Non-goals]
    becomes: architecture-constraints.md and anti-patterns.md entries
    consumer: establishing-context
  - path: .devforgeai/explore/brief.md
    sections: [Competitor scan, Technology scan]
    becomes: tech stack candidates and ADR context
    consumer: establishing-context
  - path: .devforgeai/explore/mockups/
    sections: []
    becomes: UI specifications attached to stories
    consumer: designing-interfaces
  - path: .devforgeai/explore/seed-data.json
    sections: []
    becomes: test fixtures and example rows
    consumer: implementing-stories
  - path: .devforgeai/explore/decision.yaml
    sections: []
    becomes: ADR-000, the reason this project exists
    consumer: establishing-context
```

### 3. `.devforgeai/explore/seed-data.json`

Written by step 5 from the `seed_data` field of `flow-drafter`. It carries the seven §5 top-level keys, which §5 requires of a document under `.devforgeai/`, followed by its own payload:

```json
{
  "schema": "devforgeai/explore-seed-data/1",
  "id": "IDEA-001",
  "phase": "explore",
  "status": "recorded",
  "produced_by": "exploring-ideas",
  "consumes": [],
  "open_questions": [],
  "entities": [
    { "name": "invoice", "fields": ["id", "customer", "amount", "due_on", "status"],
      "rows": [ { "id": "INV-1001", "customer": "Marsh Dental", "amount": 480.00, "due_on": "2026-09-30", "status": "sent" } ] }
  ]
}
```

`status` enum: `recorded` is the only value. `entities[].rows` holds 5 to 20 rows per entity. Every value is invented. The file survives `kill`, `park`, and `promote`; it lives outside `.explore-prototype/` so that deleting the prototype does not delete it.

### 4. `.devforgeai/explore/sketch-request.json`

Written by step 6 before the `designing-interfaces` skill is invoked, so the request is on disk and readable independently of that skill's result. It carries the same seven §5 top-level keys, with `status` enum `recorded` as its only value, followed by the sketch-mode input fields defined below.

### 5. `.explore-prototype/`

The throwaway prototype directory, at the project root, sibling to `.devforgeai/`. Entry file `.explore-prototype/index.html`. Contents: static files only, opened by the operating system's file handler. No build step, no dependency manifest, no test files, no context files. Deleted by `devforgeai explore prune --id IDEA-nnn` when `decision` is `kill` or `promote`; left in place when `decision` is `park`. `devforgeai init` appends `.explore-prototype/` to the target project's `.gitignore`.

### The sketch-mode contract with `designing-interfaces`

Step 6 invokes the `designing-interfaces` skill with the JSON object below and consumes the JSON object below it. The Design spec author conforms to both.

**Input** — written to `.devforgeai/explore/sketch-request.json` and passed in the Skill invocation:

| Field | Type | Value |
|---|---|---|
| `mode` | string | `"sketch"` |
| `idea_id` | string | `IDEA-nnn` |
| `brief_path` | string | `".devforgeai/explore/brief.md"` |
| `out_dir` | string | `".devforgeai/explore/mockups"` |
| `seed_data_path` | string | `".devforgeai/explore/seed-data.json"` |
| `fidelity` | string | `"wireframe"` |
| `flows` | array | 1 to 5 objects: `{ "flow_id": "FLOW-nnn", "name": str, "actor": str, "steps": [str], "outcome": str }` |
| `brand` | object or null | `{ "name": str, "palette_hint": str, "type_hint": str }`, or `null` when the idea has no name yet |
| `constraints` | object | `{ "no_backend": true, "no_auth": true, "no_persistence": true, "screens_per_flow_max": 3 }` |

**Output** — returned as JSON by the Design skill, with the referenced files written to `out_dir`:

| Field | Type | Value |
|---|---|---|
| `mode` | string | `"sketch"` |
| `idea_id` | string | echo of the input `idea_id` |
| `screens` | array | one object per screen: `{ "flow_id": "FLOW-nnn", "screen": "FLOW-nnn-nn", "path": str, "title": str, "state": "default" \| "empty" \| "error" }` |
| `brand_sketch` | object or null | `{ "path": ".devforgeai/explore/mockups/brand-sketch.json", "name": str, "palette": [str], "type_pair": [str, str] }` |
| `uncovered_flows` | array | `FLOW-nnn` ids from the input that produced no screen; `[]` when all flows produced screens |
| `notes` | array | 0 to 5 strings, each one line, each an observation about a flow that the mockup exposed |

`screen` is keyed `<flow id>-<two-digit screen number>`; it allocates no new ID prefix. `brand_sketch` is a precursor to Design's §5 `brand/tokens.json`, not an instance of it: sketch mode writes no `brand/tokens.json` and no `ui-specs/UI-nnn.md`. `uncovered_flows` non-empty adds one line per id to the brief's `open_questions`.

## Workflow

**1. Establish the run — model, CLI.** Input: `$ARGUMENTS`, the allocated ID on the preamble's stdout, `.devforgeai/state.toml`. The run is a remedy run when `$ARGUMENTS` contains `--remedy`, a resume when `.devforgeai/explore/brief.md` exists with `status` other than `decided`, and a fresh run otherwise. Fresh run: take the ID from the preamble stdout, run `devforgeai phase set explore --id IDEA-nnn`. Remedy run: take the ID from `$ARGUMENTS`, read `.devforgeai/reports/IDEA-nnn-discover.yaml` and `.devforgeai/explore/brief.md`, run `devforgeai phase set explore --id IDEA-nnn --remedy FLOW-nnn,FLOW-nnn` with the ids from `$ARGUMENTS`, then continue at step 4. Resume: read the brief and continue at the step after its `status`. Output: an ID, a run kind, and `[current].phase = "explore"`, `[current].id`, `[active].explore`, and the `[explore]` table written in `state.toml` by `phase set`. Failure path: `phase set` exits non-zero, its stderr reaches the model, the run stops and the Stop hook prints the handoff.

**2a. Draft the candidate segments — subagent `idea-interrogator`.** Input: the idea line, the run's `IDEA-nnn`, and the brief when resuming. Output: JSON with `problem_statement`, `candidate_segments` (2 to 4 entries, each a `label` and a `description`), `today`, `why_now`, `weak_signals`, `open_questions`, and an empty `holders`. The agent drafts candidates and asks nothing: Claude Code removes `AskUserQuestion` from every subagent, so the question belongs to this skill. Failure path: `candidate_segments` empty means the idea names no one to sell to; the model writes the returned `open_questions` into the brief frontmatter and goes to step 8 with `## Core flows` empty. Skipped on a remedy run.

**2b. Ask who holds the problem — user, `AskUserQuestion`.** Input: `candidate_segments` from step 2a. The question text, header, and option shape are fixed, and `templates/questions.md` carries the same block; `<label>` and `<description>` come from the entries in the order returned, and a fourth option reads `Someone else` with the description `A segment none of the options names; type it in Other`. `multiSelect` is true and the header is `Holders`. One round: the phase holds a five-day box and a second interview spends it. Output: the selected labels, plus any free text typed under `Someone else`. Failure path: none — a user who selects nothing is the step 2c case below.

**2c. Attribute the selected segments — subagent `idea-interrogator`.** Input: the selected labels as `selected_segments`, and the step 2a return. Output: `holders[]` filled — per selected label, 2 to 4 `attributes` an observer could check from outside, and a `frequency` — with `today`, `why_now`, `weak_signals`, and `open_questions` unchanged. Failure path: the user selected nothing, so `holders` is empty; the model writes the returned `open_questions` into the brief frontmatter and goes to step 8 with `## Core flows` empty. Steps 2a to 2c are skipped on a remedy run.

**3. Scan — subagent `landscape-scanner`.** Input: `problem_statement` and the `holders[].segment` values from step 2c. Output: JSON with `competitors`, `technologies`, `closest_match`, `sources`. Runs in parallel with nothing. Failure path: zero search results returns `competitors: []` and `sources: []`; the model writes one `open_questions` line naming the searched terms and continues. Skipped on a remedy run.

**4. Draft the one-page spec — subagent `flow-drafter`.** Input: the step 2c JSON, which carries the confirmed `holders[]` the user selected at 2b alongside `problem_statement`, `today`, `why_now`, and `weak_signals`, and the step 3 JSON — the flows are drawn from the holders the user confirmed rather than from the candidates; on a remedy run, the current `## Core flows` rows, `## Non-goals` lines, and the cited findings from the Discover report. Output: JSON with `flows` (3 to 5), `non_goals`, `success_signal`, `seed_data`. On a remedy run the agent returns only the cited `FLOW-nnn` rows in `flows`, and every other field unchanged from the brief. Failure path: a cited `FLOW-nnn` absent from `## Core flows` comes back in `unresolved_flow_ids`; the model writes one `open_questions` line per id and rewrites the ids that did resolve.

**5. Write the brief and the seed data — model.** Input: the output of steps 2c, 3, and 4. Output: `.devforgeai/explore/brief.md` with `status: specified` and all twelve sections, written from `templates/brief.md`; and `.devforgeai/explore/seed-data.json`, written by reading `templates/seed-data.json` and filling its placeholders from the `seed_data` field step 4 returned — `id` takes the run's `IDEA-nnn`, and `entities[]` takes one object per entity with its `name`, its `fields` list, and 5 to 20 `rows` keyed by those fields. The written file holds the template's eight keys at the top level in template order and no others: the seven envelope keys sit beside `entities` in one flat object. `.devforgeai/brand/tokens.json` is the one JSON document in the framework that carries the envelope under a `meta` object; the seed data is not that shape, and a `meta` wrapper here leaves `doc validate` reading a document with no `schema`. On a remedy run only the cited rows in `## Core flows`, the frontmatter `open_questions`, and `status` change. Failure path: the `PostToolUse` hook runs `doc validate` and returns its diagnostic in `hookSpecificOutput.additionalContext`, which is the channel this event has; the model reads it and rewrites the section it names.

**6. Mockups — model, `designing-interfaces` skill.** Input: `templates/sketch-request.json` read first, then filled — `id` and `idea_id` take the run's `IDEA-nnn`, `flows[]` takes one object per `## Core flows` row in row order, `brand.palette_hint` and `brand.type_hint` come from `## Target user`, and `seed_data_path` is the seed data path. The written file holds the template's keys at the top level in template order and no others: the seven envelope keys `schema`, `id`, `phase`, `status`, `produced_by`, `consumes`, `open_questions` sit at the top level of one flat object beside the document's own keys. `.devforgeai/brand/tokens.json` is the single documented exception, carrying the envelope under `meta`; every other document, MD or YAML or JSON, is flat, and a `meta` wrapper elsewhere leaves `doc validate` reading a document with no `schema`. The filled object is passed to the `design` skill in sketch mode through the Skill tool. Output: `.devforgeai/explore/sketch-request.json`, the files under `.devforgeai/explore/mockups/`, the `## Mockups` table filled from `screens`, and `status: mocked`. On a remedy run `flows` holds only the cited ids and only those rows of `## Mockups` change. Failure path: the skill returns `uncovered_flows` non-empty, and the model adds one `open_questions` line per uncovered id and continues to step 7.

**7. Prototype — subagent `prototype-builder`, optional.** Input: the mockup file paths, the seed data path, and the flows to make clickable. The step runs when the user answers `Yes` to the AskUserQuestion in the template `prototype-offer`; it is skipped on a remedy run and skipped when the user answers `No`. Output: `.explore-prototype/` with `index.html` as the entry, and the `## Prototype` table. Failure path: the agent returns `built: false` with a `reason` string; the model writes `Built | no` into the table and continues to step 8.

**8. Build the case against the idea — subagent `kill-case-builder`.** Input: `.devforgeai/explore/brief.md` and the step 3 output. Output: JSON with `kill_case`, `strongest_objection`, `evidence`, `recommended_decision`, `confidence`. The SubagentStop hook ingests it into `.devforgeai/reports/IDEA-nnn-explore.yaml`. Failure path: the agent returns `recommended_decision: "unknown"` with `confidence: 0`, and step 9 proceeds with the three options unchanged.

**9. Ask the decision — user, AskUserQuestion.** Input: `strongest_objection` and `recommended_decision` from step 8, shown as one line of prose above the question. The question and its options are fixed:

```
AskUserQuestion(questions=[{
  "question": "IDEA-001 · <slug>: what happens to this idea?",
  "header": "Decision",
  "multiSelect": false,
  "options": [
    { "label": "Kill",
      "description": "The problem is weaker than it looked, or someone already solves it well enough. The brief and the decision record stay on disk. The prototype directory is deleted." },
    { "label": "Park",
      "description": "A real problem, with the vision unsharp or the timing wrong. The brief, the mockups and the open questions stay. You pick a revisit date. The prototype directory stays." },
    { "label": "Promote",
      "description": "Worth an MVP. The brief, flows, non-goals, mockups, competitor scan and seed data carry into Discover. The prototype directory is deleted and the code is rebuilt in Phase 4." }
  ]
}])
```

When the answer is `Park`, a second AskUserQuestion follows, with `<d+30>`, `<d+90>`, `<d+180>` replaced by those dates computed from today in `YYYY-MM-DD`:

```
AskUserQuestion(questions=[{
  "question": "IDEA-001: when do you look at this again?",
  "header": "Revisit",
  "multiSelect": false,
  "options": [
    { "label": "<d+30>",  "description": "Thirty days out." },
    { "label": "<d+90>",  "description": "Ninety days out, one quarter." },
    { "label": "<d+180>", "description": "Six months out." }
  ]
}])
```

Output: one of `kill`, `park`, `promote`, and a date when the answer is `park`. Failure path: a free-text answer that parses as none of the three labels re-asks the same question once, then the run stops with the brief at `status: mocked`.

**10. Write the decision record — model.** Input: the step 9 answer, `[explore].started_at` from `state.toml`, and `[explore].remedy_flows`. Output: `.devforgeai/explore/decision.yaml` as specified in `## Outputs`, and `.devforgeai/explore/brief.md` frontmatter moved to `status: decided`. Failure path: the `PostToolUse` hook returns the `doc validate` diagnostic in `hookSpecificOutput.additionalContext`; the model rewrites the key it names.

**11. Prune the prototype — CLI.** Input: `devforgeai explore prune --id IDEA-nnn`. Output: `.explore-prototype/` removed when `decision` is `kill` or `promote`; untouched when `decision` is `park`; exit 0 in all three cases, including when the directory is absent. Failure path: an absent or unparsable `decision.yaml` is `DFA-E200` or `DFA-E401` and a path that cannot be removed is `DFA-E262`, each exit 1, and the gate's `prototype-pruned` check fails on the next Stop.

**12. Handoff — CLI, Stop hook.** `devforgeai gate check --phase explore --id IDEA-nnn` then `devforgeai handoff --phase explore --id IDEA-nnn`, both run by the Stop hook, which emits the block in `systemMessage`. The Stop hook renders the closing block; this skill writes no part of it.

## Subagents

### idea-interrogator

- **name**: `idea-interrogator`
- **derives_from**: `C:\Users\bryan\.claude\agents\business-coach.md`
- **purpose**: Probe one raw idea until the problem, the people who hold it, their current workaround, and the reason the timing is now are each stated as a fact rather than a hope.
- **tools**: `Read`
- **model**: `opus` — separating a real problem from an enthusiasm is the judgment this whole phase rests on.
- **input**: `idea_line` from `$ARGUMENTS`; `idea_id`, the run's `IDEA-nnn`; `brief_path`, `.devforgeai/explore/brief.md` when the run is a resume and null otherwise; and, on the second call alone, `selected_segments`, the labels the user selected at workflow step 2b.
- **output**:

```json
{ "type": "object", "required": ["idea_id","problem_statement","candidate_segments","holders","today","why_now","weak_signals","open_questions"],
  "properties": {
    "idea_id": { "type": "string", "pattern": "^IDEA-[0-9]{3}$" },
    "problem_statement": { "type": "string", "maxLength": 220 },
    "candidate_segments": { "type": "array", "minItems": 2, "maxItems": 4, "items": { "type": "object",
      "required": ["label","description","confidence"], "properties": {
        "label": { "type": "string", "maxLength": 60 },
        "description": { "type": "string", "maxLength": 200 },
        "confidence": { "type": "number", "minimum": 0, "maximum": 1 } } } },
    "holders": { "type": "array", "minItems": 0, "maxItems": 4, "items": { "type": "object",
      "required": ["segment","attributes","frequency"], "properties": {
        "segment": { "type": "string" },
        "attributes": { "type": "array", "minItems": 2, "maxItems": 4, "items": { "type": "string" } },
        "frequency": { "type": "string", "enum": ["daily","weekly","monthly","rarely"] } } } },
    "today": { "type": "array", "minItems": 1, "maxItems": 4, "items": { "type": "object",
      "required": ["approach","cost","breaks"], "properties": {
        "approach": { "type": "string" }, "cost": { "type": "string" }, "breaks": { "type": "string" } } } },
    "why_now": { "type": "array", "minItems": 0, "maxItems": 3, "items": { "type": "object",
      "required": ["month","change"], "properties": {
        "month": { "type": "string", "pattern": "^[0-9]{4}-[0-9]{2}$" }, "change": { "type": "string" } } } },
    "weak_signals": { "type": "array", "maxItems": 5, "items": { "type": "string" } },
    "open_questions": { "type": "array", "maxItems": 5, "items": { "type": "string" } } } }
```

- **invoked_at**: workflow steps 2a and 2c, alone each time; skipped on a remedy run.
- **registered_verifier**: `no`

`candidate_segments` is what this agent drafts on the first call and what the skill turns into the options of the question it asks at step 2b. Each entry carries a `label` a person in that job would answer to, one `description` line an observer could check, and a `confidence` from `0.0` to `1.0` for how far the idea line supports the candidate. A weakly supported candidate comes back at a low confidence rather than withheld: the user is the filter, and a segment the question does not carry never reaches them.

`holders` is `[]` on the first call and filled on the second, one entry per label in `selected_segments`. It comes back empty on the second call too when the user selected nothing, which is a result rather than an error.

This agent holds no `AskUserQuestion` tool, and neither does any other: Claude Code strips the tool from every subagent whatever its `tools` list holds, so a question belongs in the owning skill. `exploring-ideas` asks it; this agent drafts the options and attributes the answers.

### landscape-scanner

- **name**: `landscape-scanner`
- **derives_from**: `C:\Users\bryan\.claude\agents\internet-sleuth.md`
- **purpose**: Find who already solves this problem, how they charge for it, and which technologies a solution would rest on, in one pass with no file writes.
- **tools**: `WebSearch`, `WebFetch`, `Read`
- **model**: `sonnet` — retrieval and tabulation against a fixed schema, with the judgment left to `kill-case-builder`.
- **input**: `problem_statement` and `holders[].segment` from step 2c; the run's `IDEA-nnn`.
- **output**:

```json
{ "type": "object", "required": ["idea_id","competitors","technologies","closest_match","sources"],
  "properties": {
    "idea_id": { "type": "string", "pattern": "^IDEA-[0-9]{3}$" },
    "competitors": { "type": "array", "maxItems": 8, "items": { "type": "object",
      "required": ["name","url","approach","price","gap"], "properties": {
        "name": { "type": "string" }, "url": { "type": "string" }, "approach": { "type": "string" },
        "price": { "type": "string" }, "gap": { "type": "string" } } } },
    "technologies": { "type": "array", "maxItems": 8, "items": { "type": "object",
      "required": ["capability","candidate","maturity","license","source"], "properties": {
        "capability": { "type": "string" }, "candidate": { "type": "string" },
        "maturity": { "type": "string", "enum": ["established","emerging","experimental"] },
        "license": { "type": "string" }, "source": { "type": "string" } } } },
    "closest_match": { "type": ["string","null"], "description": "name of the competitor nearest the problem statement, or null" },
    "sources": { "type": "array", "items": { "type": "string" } } } }
```

- **invoked_at**: workflow step 3, alone.
- **registered_verifier**: `no`

### flow-drafter

- **name**: `flow-drafter`
- **derives_from**: `C:\Users\bryan\.claude\agents\requirements-analyst.md`
- **purpose**: Turn the brainstorm and the scan into 3 to 5 named flows with IDs, the non-goals that bound them, one success signal, and the fake rows a mockup needs.
- **tools**: `Read`
- **model**: `opus` — choosing which three flows carry the idea, and which capabilities to exclude, decides what Discover inherits.
- **input**: the full JSON from steps 2 and 3; on a remedy run, the current `## Core flows` rows, `## Non-goals` lines, `## Success signal` line, and the `findings[]` entries of `.devforgeai/reports/IDEA-nnn-discover.yaml` whose ids appear in `explore.remedy_flows`.
- **output**:

```json
{ "type": "object", "required": ["idea_id","flows","non_goals","success_signal","seed_data","unresolved_flow_ids"],
  "properties": {
    "idea_id": { "type": "string", "pattern": "^IDEA-[0-9]{3}$" },
    "flows": { "type": "array", "minItems": 1, "maxItems": 5, "items": { "type": "object",
      "required": ["flow_id","name","actor","trigger","steps","outcome"], "properties": {
        "flow_id": { "type": "string", "pattern": "^FLOW-[0-9]{3}$" },
        "name": { "type": "string" }, "actor": { "type": "string" }, "trigger": { "type": "string" },
        "steps": { "type": "array", "minItems": 2, "maxItems": 7, "items": { "type": "string" } },
        "outcome": { "type": "string" } } } },
    "non_goals": { "type": "array", "minItems": 1, "maxItems": 10, "items": { "type": "string" } },
    "success_signal": { "type": "object", "required": ["metric","threshold","window"],
      "properties": { "metric": { "type": "string" }, "threshold": { "type": "string" }, "window": { "type": "string" } } },
    "seed_data": { "type": "object", "required": ["entities"], "properties": {
      "entities": { "type": "array", "minItems": 1, "items": { "type": "object",
        "required": ["name","fields","rows"], "properties": {
          "name": { "type": "string" },
          "fields": { "type": "array", "minItems": 2, "items": { "type": "string" } },
          "rows": { "type": "array", "minItems": 5, "maxItems": 20, "items": { "type": "object" } } } } } } },
    "unresolved_flow_ids": { "type": "array", "items": { "type": "string", "pattern": "^FLOW-[0-9]{3}$" } } } }
```

- **invoked_at**: workflow step 4, alone.
- **registered_verifier**: `no`

### prototype-builder

- **name**: `prototype-builder`
- **derives_from**: `C:\Users\bryan\.claude\agents\frontend-developer.md`
- **purpose**: Turn the sketch screens into a clickable static directory seeded with the fake rows, so the flows can be walked before the decision.
- **tools**: `Read`, `Write`, `Glob`
- **model**: `sonnet` — the output is deleted at step 11, so throughput beats polish.
- **input**: `screens[]` from the sketch-mode output; `.devforgeai/explore/seed-data.json`; the `FLOW-nnn` ids to make clickable; the output root `.explore-prototype/`.
- **output**:

```json
{ "type": "object", "required": ["idea_id","built","entry","files","flows_covered","reason"],
  "properties": {
    "idea_id": { "type": "string", "pattern": "^IDEA-[0-9]{3}$" },
    "built": { "type": "boolean" },
    "entry": { "type": ["string","null"], "description": ".explore-prototype/index.html, or null when built is false" },
    "files": { "type": "array", "items": { "type": "string" } },
    "flows_covered": { "type": "array", "items": { "type": "string", "pattern": "^FLOW-[0-9]{3}$" } },
    "reason": { "type": ["string","null"], "description": "one line when built is false, null when built is true" } } }
```

- **invoked_at**: workflow step 7, alone, and only when the user answered `Yes` to the prototype offer.
- **registered_verifier**: `no`

### kill-case-builder

- **name**: `kill-case-builder`
- **derives_from**: `new`
- **purpose**: State the strongest available case for killing the idea, from the brief and the scan, so the user answers step 9 against the evidence rather than against the effort already spent.
- **tools**: `Read`
- **model**: `opus` — arguing against work the same session just produced is the one place in this phase where a weak model produces agreement instead of analysis.
- **input**: `.devforgeai/explore/brief.md`; the step 3 JSON; the `## Prototype` table row `Built`.
- **output**:

```json
{ "type": "object",
  "required": ["schema","subagent","id","passed","total","unit","findings","payload"],
  "properties": {
    "schema":   { "const": "devforgeai/verifier/1" },
    "subagent": { "const": "kill-case-builder" },
    "id":       { "type": "string", "pattern": "^IDEA-[0-9]{3}$" },
    "passed":   { "type": "integer", "minimum": 0 },
    "total":    { "type": "integer", "minimum": 0 },
    "unit":     { "const": "objections" },
    "findings": { "type": "array", "maxItems": 0 },
    "payload": { "type": "object",
      "required": ["idea_id","kill_case","strongest_objection","evidence","recommended_decision","confidence"],
      "properties": {
        "idea_id": { "type": "string", "pattern": "^IDEA-[0-9]{3}$" },
        "kill_case": { "type": "array", "minItems": 1, "maxItems": 5, "items": { "type": "string" } },
        "strongest_objection": { "type": "string", "maxLength": 200 },
        "evidence": { "type": "array", "items": { "type": "object", "required": ["claim","source","confidence"],
          "properties": { "claim": { "type": "string" },
            "source": { "type": "string", "description": "a brief section heading, a FLOW-nnn id, a URL from the scan, or the literal unsourced" },
            "confidence": { "type": "number", "minimum": 0, "maximum": 1 } } } },
        "recommended_decision": { "type": "string", "enum": ["kill","park","promote","unknown"] },
        "confidence": { "type": "number", "minimum": 0, "maximum": 1 } } } } }
```

  One object, the `devforgeai/verifier/1` envelope with this agent's own fields under `payload`. `total` counts the objections raised and `passed` counts the ones the brief answers, so the handoff's `Verified` line reads as a ratio of objections survived. `findings` is empty by construction: a kill case is an argument the user rules on at step 9, not a defect the gate refuses.

- **invoked_at**: workflow step 8, alone, after the mockups and before the question.
- **registered_verifier**: `yes` — SubagentStop ingests it into `.devforgeai/reports/IDEA-nnn-explore.yaml`, which supplies the handoff `Verified` line.

## Command

The entry point is the skill itself: `skills/exploring-ideas/SKILL.md`, installed to `.claude/skills/explore/`, whose frontmatter `name` is the slash command. There is no command file. This is the frontmatter and the preamble, verbatim, with no preamble line.

```markdown
---
name: explore
description: Phase 0 of DevForgeAI. Carries one half-formed idea from a sentence to a recorded kill, park or promote decision through six steps - brainstorm, competitor and technology scan, a one-page brief, wireframe mockups, an optional clickable prototype, and the decision - writing .devforgeai/explore/brief.md and .devforgeai/explore/decision.yaml. Reach for it whenever /explore runs, whenever someone brings a raw product idea, a "is this worth building" question, or a time-boxed spike that precedes requirements, and whenever a Discover send-back arrives as /explore IDEA-nnn --remedy FLOW-nnn. It also owns the artifacts - the explore brief, its FLOW-nnn core flows, the decision record, seed-data.json, sketch-request.json, the mockups directory, and the throwaway .explore-prototype directory - so read it before touching any of them.
argument-hint: '"<idea in one line>" | IDEA-nnn [--remedy FLOW-nnn,...]'
allowed-tools: Bash(devforgeai:*), PowerShell(devforgeai:*), Read, Write, Edit, Glob, Grep, Agent, AskUserQuestion, Skill, WebSearch, WebFetch
disable-model-invocation: true
---
```

This skill carries no preamble. Explore has no predecessor phase, so there is no `gate require` line, and it reads no upstream document of conventions section 5, so there is no `doc load` line. The `IDEA-nnn` is allocated inside workflow step 1, once the run kind is known: a remedy run takes its id from `$ARGUMENTS` and a resume run takes it from the brief, so neither spends an id and neither aborts on an exhausted `IDEA` prefix. Allocation in a preamble runs before the run kind is known, which is why it sits in the step that needs the id.

## CLI calls

| Subcommand with exact arguments | Called from | Exit handling |
|---|---|---|
| `devforgeai doc validate --allocate IDEA` | workflow step 1, on a fresh run alone | stdout is the next free `IDEA-nnn`, used by step 1 on a fresh run and ignored on a remedy run |
| `devforgeai phase set explore --id IDEA-nnn` | workflow step 1, fresh run | non-zero stops the run; stderr reaches the model |
| `devforgeai phase set explore --id IDEA-nnn --remedy FLOW-nnn,FLOW-nnn` | workflow step 1, remedy run | non-zero stops the run; stderr reaches the model |
| `devforgeai explore prune --id IDEA-nnn` | workflow step 11 | 0 when the directory is absent, removed, or kept for `park`; 1 on `DFA-E200`, `DFA-E401`, or `DFA-E262` |
| `devforgeai doc validate <path>` | `PostToolUse` hook on `Write\|Edit\|NotebookEdit` under `.devforgeai/` | returns the diagnostic in `hookSpecificOutput.additionalContext`; step 5 and step 10 rewrite the named key or section |
| `devforgeai gate check --phase explore --id IDEA-nnn` | Stop hook | 0 PASS, 1 FAIL |
| `devforgeai handoff --phase explore --id IDEA-nnn` | Stop hook | prints the §6 block |
| `devforgeai report show IDEA-nnn explore` | user, `improving-framework` | prints `.devforgeai/reports/IDEA-nnn-explore.yaml` |

`explore prune` is accepted into the CLI surface by `specs/01-cli.md`. Every other subcommand is a §4 name.

`kill-case-builder` is a registered verifier, so the SubagentStop hook ingests it and the `kill-case-answered` gate check and the handoff `Verified` line have a source. Its `config.toml` registry entry, in the `[[verifier]]` schema `specs/01-cli.md` `## Outputs` fixes:

```toml
[[verifier]]
name = "kill-case-builder"
phase = "explore"
report_field = "verifiers.kill_case"
unit = "objections"
required = true
```

`report ingest kill-case-builder -` writes the agent's JSON under `verifiers.kill_case` of `.devforgeai/reports/IDEA-nnn-explore.yaml`, with `total` the length of `kill_case[]` and `passed` the count of its entries the brief answers.

## Gate

The `[[gate]]` entry for `explore` in `.devforgeai/gates.toml`, in the container shape `specs/01-cli.md` `## Outputs` fixes, verbatim. A `path` or `doc_exists` value is a document name and resolves against `.devforgeai/`; the `paths` of a `file_exists` check resolve against the project root, which is what lets `prototype-pruned` reach outside `.devforgeai/`.

```toml
[[gate]]
phase = "explore"
requires = ""
on_fail = "fail"
send_back_to = ""
description = "A decision is recorded, the brief carries three to five flows, the time box holds."

  [[gate.check]]
  kind = "file_exists"
  id = "decision-exists"
  severity = "block"
  on_fail = "fail"
  paths = [".devforgeai/explore/decision.yaml"]
  min_count = 1
  message = "no decision recorded for {id}"

  [[gate.check]]
  kind = "field_in_enum"
  id = "decision-enum"
  severity = "block"
  on_fail = "fail"
  path = "explore/decision.yaml"
  field = "decision"
  values = ["kill", "park", "promote"]
  message = "decision is {value}, expected kill, park or promote"

  [[gate.check]]
  kind = "field_is_date"
  id = "decision-dated"
  severity = "block"
  on_fail = "fail"
  path = "explore/decision.yaml"
  field = "decided_on"
  format = "%Y-%m-%d"
  message = "decided_on is {value}, expected YYYY-MM-DD"

  [[gate.check]]
  kind = "field_is_date"
  id = "park-has-revisit"
  severity = "block"
  on_fail = "fail"
  path = "explore/decision.yaml"
  field = "revisit_on"
  format = "%Y-%m-%d"
  after_field = "decided_on"
  required_when = { path = "explore/decision.yaml", field = "decision", equals = "park" }
  null_when = { path = "explore/decision.yaml", field = "decision", in = ["kill", "promote"] }
  message = "park decision needs revisit_on later than decided_on"

  [[gate.check]]
  kind = "length_between"
  id = "promote-carries-forward"
  severity = "block"
  on_fail = "fail"
  path = "explore/decision.yaml"
  field = "carry_forward"
  min = 7
  max = 7
  required_when = { path = "explore/decision.yaml", field = "decision", equals = "promote" }
  empty_when = { path = "explore/decision.yaml", field = "decision", in = ["kill", "park"] }
  message = "promote decision carries {value} artifacts, expected 7"

  [[gate.check]]
  kind = "row_count_between"
  id = "flow-count"
  severity = "block"
  on_fail = "fail"
  path = "explore/brief.md"
  section = "Core flows"
  min = 3
  max = 5
  message = "{value} core flows, expected 3 to 5"

  [[gate.check]]
  kind = "column_matches"
  id = "flow-id-shape"
  severity = "block"
  on_fail = "fail"
  path = "explore/brief.md"
  section = "Core flows"
  column = "ID"
  pattern = "^FLOW-[0-9]{3}$"
  unique = true
  message = "core flow id {value} is malformed or duplicated"

  [[gate.check]]
  kind = "row_count_between"
  id = "one-success-signal"
  severity = "block"
  on_fail = "fail"
  path = "explore/brief.md"
  section = "Success signal"
  min = 1
  max = 1
  message = "{value} success signals, expected exactly 1"

  [[gate.check]]
  kind = "verifier_pass"
  id = "kill-case-answered"
  severity = "block"
  on_fail = "fail"
  verifiers = ["kill-case-builder"]
  min_ratio = 0.0
  message = "kill-case-builder result absent from the report for {id}"

  [[gate.check]]
  kind = "column_contains_all"
  id = "remedy-flows-present"
  severity = "block"
  on_fail = "fail"
  path = "explore/brief.md"
  section = "Core flows"
  column = "ID"
  state_field = "explore.remedy_flows"
  skip_when = { state_field = "explore.remedy_flows", is_empty = true }
  message = "remedy cited {value}, which is absent from Core flows"

  [[gate.check]]
  kind = "elapsed_days_at_most"
  id = "time-box"
  severity = "block"
  on_fail = "fail"
  started_field = "explore.started_at"
  limit_field = "explore.timebox_days"
  remedy_started_field = "explore.remedy_started_at"
  remedy_limit_field = "explore.remedy_timebox_days"
  skip_when = { doc_exists = "explore/decision.yaml" }
  message = "time box spent, {value} of {limit} days, decide kill, park or promote"

  [[gate.check]]
  kind = "file_exists"
  id = "prototype-pruned"
  severity = "block"
  on_fail = "fail"
  paths = [".explore-prototype"]
  absent = true
  required_when = { path = "explore/decision.yaml", field = "decision", in = ["kill", "promote"] }
  message = "prototype directory still present after a {value} decision"
```

The gate holds `file_exists` and `field_in_enum` at `severity = "block"`, which is what the compiled-in minimums table requires of the explore phase. `kill-case-answered` carries `min_ratio = 0.0`, so the check passes on the presence of the ingested block alone: the floor of 1.0 applies to the verify phase, and a `kill` decision is the case where every objection stands unanswered on purpose. The pass criteria for this phase live here and are not restated in the skill, the command, or the subagents.

## Send-back

Explore emits no SEND BACK. §5 gives its "May send back to" column as `none`, because Phase 0 has no upstream phase whose document it could cite. Its gate result is `PASS` or `FAIL` only, and `send_back_to = []` in the gate entry above holds that.

Explore is the receiving end of exactly one send-back: Discover's, when `requirements.yaml` cannot be written because a flow in the brief contradicts itself, names no actor, or duplicates another flow. Discover's handoff prints `Next /explore IDEA-nnn --remedy FLOW-nnn,FLOW-nnn` and `Then /discover IDEA-nnn --resume`, and the user types the first line as printed.

What `/explore IDEA-001 --remedy FLOW-002,FLOW-004` does differently from a fresh run:

| Aspect | Fresh run | Remedy run |
|---|---|---|
| ID | allocated by the preamble | taken from `$ARGUMENTS`; the preamble's allocation is discarded |
| Step 1 extra read | none | `.devforgeai/reports/IDEA-001-discover.yaml`, and `.devforgeai/explore/brief.md` |
| `phase set` arguments | `explore --id IDEA-001` | `explore --id IDEA-001 --remedy FLOW-002,FLOW-004` |
| `state.toml` writes | `[current].phase`, `[current].id`, `[active].explore`, `[explore].idea_id`, `[explore].started_at`, `[explore].timebox_days`, `[explore].remedy_flows = []` | `[explore].remedy_started_at`, `[explore].remedy_timebox_days`, `[explore].remedy_flows = ["FLOW-002","FLOW-004"]`; `[explore].started_at` and `[current]` unchanged |
| Time box | `explore.timebox_days` from `explore.started_at` | `explore.remedy_timebox_days` from `explore.remedy_started_at` |
| Step 2 brainstorm | runs | skipped |
| Step 3 scan | runs | skipped |
| Step 4 drafting | all flows, non-goals, success signal, seed data | the cited `FLOW-nnn` rows only; non-goals, success signal and seed data returned unchanged |
| Step 5 brief writes | all twelve sections | the cited rows of `## Core flows`, frontmatter `open_questions`, and `status` |
| Step 6 mockups | every flow in `flows` | the cited ids only; the other rows of `## Mockups` are left as they are |
| Step 7 prototype | offered | skipped; `.explore-prototype/` is left as it is |
| Step 8 kill case | runs | runs |
| Step 9 question | asked | asked, with the same three options |
| Step 10 decision | `remedied_flows: []` | `remedied_flows: ["FLOW-002","FLOW-004"]` |
| Handoff `Next` | `/discover IDEA-001` | `/discover IDEA-001 --resume` |

A cited `FLOW-nnn` that is absent from `## Core flows` comes back from step 4 in `unresolved_flow_ids`; the model writes one line per id into the brief's `open_questions` in the form `FLOW-nnn cited by the Discover send-back is absent from Core flows`, and the gate's `remedy-flows-present` check fails, so the handoff carries the id.

## Integration

| Skill | Consumes (doc, IDs) | Produces for (doc, IDs) | Sends back to (condition) | Receives send-back from (condition) | Shared subagents | `state.toml` fields read/written |
|---|---|---|---|---|---|---|
| 1 Discover · `discovering-requirements` | none — Explore is Phase 0 and reads no requirements document | `.devforgeai/explore/brief.md` (`IDEA-nnn`, `FLOW-nnn`): `## Problem statement` and `## Target user` seed the discovery brief and first persona, `## Core flows` rows seed epic candidates; `.devforgeai/explore/decision.yaml` (`IDEA-nnn`) supplies the `promote` verdict that lets `/discover` start | none — §5 gives Explore's "May send back to" column as `none` | Discover, when a `FLOW-nnn` row contradicts itself, names no actor, or duplicates another row; arrives as `/explore IDEA-nnn --remedy FLOW-nnn,...` | `flow-drafter` derives from the same source agent Discover adapts (`requirements-analyst.md`); the two adaptations stay separate agents | reads `[current].phase`, `[active].explore`; writes `[active].explore`, `[explore].*` |
| 2 Constitute · `establishing-context` | none — context files and ADRs do not exist during Phase 0 | `.devforgeai/explore/brief.md`: `## Non-goals` lines, cited by verbatim text, become `architecture-constraints.md` and `anti-patterns.md` entries and Constitute allocates their `CON-nnn` ids itself; `## Competitor scan` and `## Technology scan` rows become tech stack candidates and ADR context; `.devforgeai/explore/decision.yaml` becomes `adr/ADR-000.md` | none | none — Constitute sends back to Discover, not past it | none | none |
| 3 Plan · `planning-work` | none — no stories exist during Phase 0 | none directly; `## Core flows` reaches Plan through Discover's `requirements.yaml` | none | none — Plan sends back to Discover and Constitute | none | none |
| 4 Build · `implementing-stories` | none — Phase 0 writes no production code | `.devforgeai/explore/seed-data.json`, named by `carry_forward` entry 6: `entities[].rows` become test fixtures and example rows. Build reads the file when it exists — it is in `implementing-stories`' entry list and in `ac-test-writer`'s prompt fields — and works from its own invented rows when it does not, so a project that skipped Explore loses nothing | none | none — Build sends back to Plan | none | none |
| 5 Verify · `validating-quality` | none — nothing built in Phase 0 is verified; `.explore-prototype/` carries no tests and is deleted at step 11 | none | none | none — Verify sends back to Build and Plan | none | none |
| 6 Release · `releasing-software` | none — Phase 0 ships nothing | none | none | none — Release sends back to Verify | none | none |
| Design · `designing-interfaces` | the sketch-mode output object defined in `## Outputs`: `screens[]`, `brand_sketch`, `uncovered_flows`, `notes`, plus the files under `.devforgeai/explore/mockups/`. Screens are keyed `FLOW-nnn-nn` and allocate no `UI-nnn` | the sketch-mode input object written to `.devforgeai/explore/sketch-request.json`: `mode`, `idea_id`, `brief_path`, `out_dir`, `seed_data_path`, `fidelity`, `flows[]` (`FLOW-nnn`), `brand`, `constraints`. On promote, `.devforgeai/explore/mockups/` carries forward as the source of `ui-specs/UI-nnn.md` | none | none — Design sends back to Discover, not to Explore | none; Design owns `mockup-designer` and `brand-designer`, Explore owns `prototype-builder`, and each is invoked only by its owner | none |
| Reflect · `improving-framework` | none — Reflect runs after a phase and returns recommendations, which §5 excludes from gates | `.devforgeai/reports/IDEA-nnn-explore.yaml`, written by the SubagentStop ingest of `kill-case-builder` and by `gate check`, as one input to `OBS-nnn` extraction | none | none — Reflect returns recommendations, not send-backs | none | reads `[current].*`, `[last_gate].*`, `[explore].*` |
| CLI · `devforgeai` | `.devforgeai/state.toml` keys `[current].phase`, `[current].id`, `[active].explore`, `[explore].*`; `.devforgeai/config.toml` keys `[explore].timebox_days`, `[explore].remedy_timebox_days`, and the `[[verifier]]` entry for `kill-case-builder`; `.devforgeai/gates.toml` entry `[[gate]]` with `phase = "explore"` | `.devforgeai/explore/brief.md` and `.devforgeai/explore/decision.yaml` for `doc validate` and `gate check`; `.devforgeai/reports/IDEA-nnn-explore.yaml` for `report show` and `handoff` | none | none | none | reads `[current].phase`, `[current].id`, `[active].explore`, and the six `[explore]` keys; writes `[current].phase`, `[current].id`, `[active].explore`, and the six `[explore]` keys via `phase set` |

## Handoff

PASS, fresh run, promote decision:

```
Phase     0 · Explore        IDEA-001 · invoice-chaser
Done      4 flows, 7 screens, 2 competitors, prototype built and pruned
Gate      PASS  decision promote · 4 of 5 days · 11 checks
Verified  kill-case-builder · 3/3 objections answered in the brief

Next      /discover IDEA-001
Then      /constitute IDEA-001
Blocked   none

Full report: .devforgeai/reports/IDEA-001-explore.yaml
```

FAIL, time box spent with no decision recorded:

```
Phase     0 · Explore        IDEA-002 · shift-swapper
Done      5 flows, 9 screens, 3 competitors, prototype built
Gate      FAIL  time-box 6 of 5 days · decision-exists missing
Verified  kill-case-builder · recommended park at confidence 0.4

Next      /explore IDEA-002
Then      /discover IDEA-002
Blocked   you: kill, park or promote?

Full report: .devforgeai/reports/IDEA-002-explore.yaml
```

SEND BACK, printed by Discover and not by Explore — §5 gives Explore's "May send back to" column as `none`, so this is the block the user acts on to start a remedy run:

```
Phase     1 · Discover       IDEA-001 · invoice-chaser
Done      5 of 7 epics drafted, 2 blocked on flows
Gate      SEND BACK  to Explore
Found     FLOW-002 names no actor
          FLOW-004 duplicates FLOW-001

Next      /explore IDEA-001 --remedy FLOW-002,FLOW-004
Then      /discover IDEA-001 --resume
Blocked   you: is FLOW-004 a separate flow?

Full report: .devforgeai/reports/IDEA-001-discover.yaml
```

## Templates

### `templates/brief.md`

```markdown
---
schema: devforgeai/explore-brief/1
id: IDEA-nnn
phase: explore
status: drafting
produced_by: exploring-ideas
consumes: []
open_questions: []
---

## Problem statement

<one sentence: who loses what, how often>

## Target user

<segment>
- <observable attribute>
- <observable attribute>

## What they do today

| Current approach | Cost | Where it breaks |
|---|---|---|
| <approach> | <time or money> | <the moment it fails> |

## Why now

YYYY-MM — <what changed>

## Competitor scan

| Name | URL | Approach | Price | Gap |
|---|---|---|---|---|
| <name> | <url> | <how they solve it> | <what they charge> | <what they leave unsolved> |

## Technology scan

| Capability | Candidate | Maturity | License | Source |
|---|---|---|---|---|
| <capability> | <name> | established\|emerging\|experimental | <license> | <url> |

## Core flows

| ID | Actor | Trigger | Steps | Outcome |
|---|---|---|---|---|
| FLOW-001 | <who> | <what starts it> | <step> -> <step> -> <step> | <what is true at the end> |

## Non-goals

<capability this idea excludes>

## Success signal

<metric> | <threshold> | <observation window>

## Mockups

| Flow | Screen | Path | State |
|---|---|---|---|
| FLOW-001 | FLOW-001-01 | .devforgeai/explore/mockups/FLOW-001-01.html | default |

## Seed data

| Entity | Rows | Field count |
|---|---|---|
| <entity> | <n> | <n> |

## Prototype

| Field | Value |
|---|---|
| Built | yes\|no |
| Path | .explore-prototype/ |
| Entry | .explore-prototype/index.html |
| Flows covered | FLOW-001, FLOW-002 |
```

### `templates/decision.yaml`

```yaml
schema: devforgeai/explore-decision/1
id: IDEA-nnn
phase: explore
status: recorded
produced_by: exploring-ideas
consumes: []
open_questions: []
decision: kill|park|promote
decided_on: YYYY-MM-DD
reason: >-
  <one sentence, 15 to 40 words, naming the evidence that produced this decision>
revisit_on: YYYY-MM-DD|null
elapsed_days: 0
remedied_flows: []
carry_forward: []
# carry_forward when decision is promote, these seven entries in this order:
#   - path: .devforgeai/explore/brief.md
#     sections: [Problem statement, Target user]
#     becomes: discovery brief and first persona
#     consumer: discovering-requirements
#   - path: .devforgeai/explore/brief.md
#     sections: [Core flows]
#     becomes: epic candidates with draft acceptance criteria
#     consumer: discovering-requirements
#   - path: .devforgeai/explore/brief.md
#     sections: [Non-goals]
#     becomes: architecture-constraints.md and anti-patterns.md entries
#     consumer: establishing-context
#   - path: .devforgeai/explore/brief.md
#     sections: [Competitor scan, Technology scan]
#     becomes: tech stack candidates and ADR context
#     consumer: establishing-context
#   - path: .devforgeai/explore/mockups/
#     sections: []
#     becomes: UI specifications attached to stories
#     consumer: designing-interfaces
#   - path: .devforgeai/explore/seed-data.json
#     sections: []
#     becomes: test fixtures and example rows
#     consumer: implementing-stories
#   - path: .devforgeai/explore/decision.yaml
#     sections: []
#     becomes: ADR-000, the reason this project exists
#     consumer: establishing-context
```

### `templates/seed-data.json`

```json
{
  "schema": "devforgeai/explore-seed-data/1",
  "id": "IDEA-nnn",
  "phase": "explore",
  "status": "recorded",
  "produced_by": "exploring-ideas",
  "consumes": [],
  "open_questions": [],
  "entities": [
    { "name": "<entity name>", "fields": ["<field-1>", "<field-2>"],
      "rows": [ { "<field-1>": "<value>", "<field-2>": "<value>" } ] }
  ]
}
```

Eight keys at the top level of one flat object, in this order: the seven envelope keys and `entities`. No `meta` wrapper — `brand/tokens.json` is the one JSON document in the framework shaped that way, and a wrapper here leaves `doc validate` reading a document with no `schema`. `id` takes the run's `IDEA-nnn`. Each `entities[]` object carries a `name`, a `fields` list, and `rows` keyed by those field names.

### `templates/sketch-request.json`

```json
{
  "schema": "devforgeai/explore-sketch-request/1",
  "id": "IDEA-nnn",
  "phase": "explore",
  "status": "recorded",
  "produced_by": "exploring-ideas",
  "consumes": [],
  "open_questions": [],
  "mode": "sketch",
  "idea_id": "IDEA-nnn",
  "brief_path": ".devforgeai/explore/brief.md",
  "out_dir": ".devforgeai/explore/mockups",
  "seed_data_path": ".devforgeai/explore/seed-data.json",
  "fidelity": "wireframe",
  "flows": [
    { "flow_id": "FLOW-001", "name": "<flow name>", "actor": "<who>",
      "steps": ["<step>", "<step>"], "outcome": "<what is true at the end>" }
  ],
  "brand": { "name": "<idea name>", "palette_hint": "<two or three words>", "type_hint": "<two or three words>" },
  "constraints": { "no_backend": true, "no_auth": true, "no_persistence": true, "screens_per_flow_max": 3 }
}
```

### `templates/prototype-offer.md`

```markdown
AskUserQuestion(questions=[{
  "question": "IDEA-nnn: build a clickable prototype of <n> flows before deciding?",
  "header": "Prototype",
  "multiSelect": false,
  "options": [
    { "label": "Yes", "description": "Static pages under .explore-prototype/, seeded with the fake rows, deleted on kill and on promote." },
    { "label": "No",  "description": "Decide from the mockups. The Prototype row of the brief reads no." }
  ]
}])
```

## Evals

Shipped at `skills/exploring-ideas/evals/`.

### `evals/evals.json` — 8 entries, skill-creator format

| # | `prompt` | `expected_output` | `expectations[]` |
|---|---|---|---|
| 1 | `/explore "a way for small dental practices to chase unpaid invoices"` | `.devforgeai/explore/brief.md` at `status: decided` and `.devforgeai/explore/decision.yaml` | brief has the twelve headings in template order; `## Core flows` holds 3 to 5 rows; every `ID` matches `^FLOW-[0-9]{3}$`; `## Success signal` holds exactly one line; frontmatter keys are the seven of §5 in order |
| 2 | `/explore "an app that tells you which houseplant is dying"` then the user answers `Kill` | `decision.yaml` with `decision: kill` | `decision` is `kill`; `revisit_on` is `null`; `carry_forward` is `[]`; `reason` is 15 to 40 words; `.explore-prototype/` is absent |
| 3 | `/explore "shift swapping for restaurant staff"` then the user answers `Park` and picks the 90-day option | `decision.yaml` with `decision: park` | `decision` is `park`; `revisit_on` parses as `YYYY-MM-DD` and is later than `decided_on`; `carry_forward` is `[]`; the brief is still on disk; `.explore-prototype/` is still on disk |
| 4 | `/explore "invoice chasing for dental practices"` then the user answers `Promote` | `decision.yaml` with `decision: promote` | `carry_forward` holds exactly seven entries in the order given in the template; every `path` in it exists; `.explore-prototype/` is absent; `consumer` values are the four skill names of the carry-forward table |
| 5 | `/explore IDEA-001 --remedy FLOW-002,FLOW-004` with a prior brief and a Discover report on disk | a brief whose `FLOW-002` and `FLOW-004` rows changed and whose other rows did not | `FLOW-001` and `FLOW-003` rows are byte-identical to the prior brief; `FLOW-002` and `FLOW-004` rows differ; `remedied_flows` is `["FLOW-002","FLOW-004"]`; no web search ran; `## Competitor scan` is unchanged |
| 6 | `/explore IDEA-001 --remedy FLOW-002,FLOW-009` where `FLOW-009` is absent from the brief | a brief with an `open_questions` entry naming `FLOW-009` | `open_questions` holds a line containing `FLOW-009`; the `FLOW-002` row differs from the prior brief; `## Core flows` still holds 3 to 5 rows; no row with id `FLOW-009` was invented |
| 7 | `/explore "a budgeting tool for freelancers"` where `state.toml` shows `[explore].started_at` six days ago | a brief at `status: mocked` and no `decision.yaml` | `.devforgeai/explore/decision.yaml` is absent; the run reached step 9 and asked the decision question; the three option labels are `Kill`, `Park`, `Promote` |
| 8 | `/explore "a tool for tracking what a book club has read"` | `.devforgeai/explore/sketch-request.json` matching the sketch-mode input contract | the file carries the seven §5 top-level keys and `mode: "sketch"`; `flows[]` holds one object per `## Core flows` row with the same `FLOW-nnn` ids in the same order; `out_dir` is `.devforgeai/explore/mockups` and `seed_data_path` is `.devforgeai/explore/seed-data.json`; `constraints` holds the four fixed keys with `no_backend`, `no_auth` and `no_persistence` true |

Entries 5 and 6 exercise the send-back path, per §9.

### `evals/cases.jsonl` — 8 lines

```json
{"id": "ex-01-fresh-brief", "prompt": "/explore \"a way for small dental practices to chase unpaid invoices\"", "answers": {"Holders": ["@first"], "Prototype": "No", "Decision": "Kill"}, "setup": {"files": {".devforgeai/config.toml": "schema = \"devforgeai/config/1\"\ngenerated_at = \"2026-09-10T09:00:00Z\"\ncli_version = \"1.0.0\"\ndegraded = false\n[explore]\ntimebox_days = 5\nremedy_timebox_days = 1\n", ".devforgeai/state.toml": "schema = \"devforgeai/state/1\"\nupdated_at = \"2026-09-10T09:00:00Z\"\n[current]\nphase = \"explore\"\nid = \"\"\n[active]\nexplore = \"\"\n[explore]\n", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml"}}, "expect": {"grader": "brief_shape", "args": {"path": ".devforgeai/explore/brief.md", "min_flows": 3, "max_flows": 5}}}
{"id": "ex-02-kill", "prompt": "/explore \"an app that tells you which houseplant is dying\"", "answers": {"Holders": ["@first"], "Prototype": "No", "Decision": "Kill"}, "setup": {"files": {".devforgeai/config.toml": "schema = \"devforgeai/config/1\"\ngenerated_at = \"2026-09-10T09:00:00Z\"\ncli_version = \"1.0.0\"\ndegraded = false\n[explore]\ntimebox_days = 5\nremedy_timebox_days = 1\n", ".devforgeai/state.toml": "schema = \"devforgeai/state/1\"\nupdated_at = \"2026-09-10T09:00:00Z\"\n[current]\nphase = \"explore\"\nid = \"\"\n[active]\nexplore = \"\"\n[explore]\n", ".explore-prototype/index.html": "<p>old</p>", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml"}}, "expect": {"grader": "decision_shape", "args": {"decision": "kill", "revisit": false, "carry_forward_len": 0, "absent": [".explore-prototype"]}}}
{"id": "ex-03-park", "prompt": "/explore \"shift swapping for restaurant staff\"", "answers": {"Holders": ["@first"], "Prototype": "No", "Decision": "Park", "Revisit": "@2"}, "setup": {"files": {".devforgeai/config.toml": "schema = \"devforgeai/config/1\"\ngenerated_at = \"2026-09-10T09:00:00Z\"\ncli_version = \"1.0.0\"\ndegraded = false\n[explore]\ntimebox_days = 5\nremedy_timebox_days = 1\n", ".devforgeai/state.toml": "schema = \"devforgeai/state/1\"\nupdated_at = \"2026-09-10T09:00:00Z\"\n[current]\nphase = \"explore\"\nid = \"\"\n[active]\nexplore = \"\"\n[explore]\n", ".explore-prototype/index.html": "<p>old</p>", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml"}}, "expect": {"grader": "decision_shape", "args": {"decision": "park", "revisit": true, "carry_forward_len": 0, "present": [".explore-prototype/index.html", ".devforgeai/explore/brief.md"]}}}
{"id": "ex-04-promote", "prompt": "/explore \"invoice chasing for dental practices\"", "answers": {"Holders": ["@first"], "Prototype": "No", "Decision": "Promote"}, "setup": {"files": {".devforgeai/config.toml": "schema = \"devforgeai/config/1\"\ngenerated_at = \"2026-09-10T09:00:00Z\"\ncli_version = \"1.0.0\"\ndegraded = false\n[explore]\ntimebox_days = 5\nremedy_timebox_days = 1\n", ".devforgeai/state.toml": "schema = \"devforgeai/state/1\"\nupdated_at = \"2026-09-10T09:00:00Z\"\n[current]\nphase = \"explore\"\nid = \"\"\n[active]\nexplore = \"\"\n[explore]\n", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml"}}, "expect": {"grader": "carry_forward_exact", "args": {"path": ".devforgeai/explore/decision.yaml", "consumers": ["discovering-requirements", "discovering-requirements", "establishing-context", "establishing-context", "designing-interfaces", "implementing-stories", "establishing-context"], "absent": [".explore-prototype"]}}}
{"id": "ex-05-remedy-two-flows", "prompt": "/explore IDEA-001 --remedy FLOW-002,FLOW-004", "setup": {"files": {".devforgeai/config.toml": "schema = \"devforgeai/config/1\"\ngenerated_at = \"2026-09-10T09:00:00Z\"\ncli_version = \"1.0.0\"\ndegraded = false\n[explore]\ntimebox_days = 5\nremedy_timebox_days = 1\n", ".devforgeai/state.toml": "schema = \"devforgeai/state/1\"\nupdated_at = \"2026-09-10T09:00:00Z\"\n[current]\nphase = \"explore\"\nid = \"IDEA-001\"\n[active]\nexplore = \"IDEA-001\"\n[explore]\nidea_id = \"IDEA-001\"\nstarted_at = \"2026-09-01T09:00:00Z\"\ntimebox_days = 5\n", ".devforgeai/explore/brief.md": "FIXTURE:brief-4-flows.md", ".devforgeai/reports/IDEA-001-discover.yaml": "FIXTURE:discover-sendback.yaml", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml"}}, "expect": {"grader": "remedy_touched_only", "args": {"path": ".devforgeai/explore/brief.md", "changed": ["FLOW-002", "FLOW-004"], "unchanged": ["FLOW-001", "FLOW-003"], "unchanged_sections": ["Competitor scan", "Technology scan", "Non-goals", "Success signal"], "remedied_flows": ["FLOW-002", "FLOW-004"], "baseline_rows": {"FLOW-001": "| FLOW-001 | club organiser | monthly meeting is set | pick a book -> post the date -> collect who is coming | every member knows the book and the date |", "FLOW-003": "| FLOW-003 | member | finished the book | mark it read -> leave a one-line note | the club sees who has finished |"}, "baseline_section_sha256": {"Competitor scan": "3a4020210a6f69e2a70ecaae74cf6c9c6514d97983323412cae844f29fb094a2", "Technology scan": "f1076d104e1f7257526ab4d3233651e764d71fbd8f1af929d066fe5268677355", "Non-goals": "ffa2f67848c1a7b76c3bc79d720d007d6e60f79843195172c6a958a792a74d2e", "Success signal": "54431fda8f53c08bcdefaeca08efa5923eb1d752b8f16877f915ca94e314f309"}}}}
{"id": "ex-06-remedy-unknown-flow", "prompt": "/explore IDEA-001 --remedy FLOW-002,FLOW-009", "setup": {"files": {".devforgeai/config.toml": "schema = \"devforgeai/config/1\"\ngenerated_at = \"2026-09-10T09:00:00Z\"\ncli_version = \"1.0.0\"\ndegraded = false\n[explore]\ntimebox_days = 5\nremedy_timebox_days = 1\n", ".devforgeai/state.toml": "schema = \"devforgeai/state/1\"\nupdated_at = \"2026-09-10T09:00:00Z\"\n[current]\nphase = \"explore\"\nid = \"IDEA-001\"\n[active]\nexplore = \"IDEA-001\"\n[explore]\nidea_id = \"IDEA-001\"\nstarted_at = \"2026-09-01T09:00:00Z\"\ntimebox_days = 5\n", ".devforgeai/explore/brief.md": "FIXTURE:brief-4-flows.md", ".devforgeai/reports/IDEA-001-discover.yaml": "FIXTURE:discover-sendback-unknown.yaml", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml"}}, "expect": {"grader": "open_questions_mentions", "args": {"path": ".devforgeai/explore/brief.md", "ids": ["FLOW-009"], "absent_rows": ["FLOW-009"], "changed": ["FLOW-002"], "baseline_rows": {"FLOW-002": "| FLOW-002 | member | the date is posted | open the invite -> answer yes or no | the organiser has a head count |"}}}}
{"id": "ex-07-timebox-spent", "prompt": "/explore \"a budgeting tool for freelancers\"", "answers": {"Holders": ["@first"], "Prototype": "No", "Decision": "Kill"}, "setup": {"files": {".devforgeai/config.toml": "schema = \"devforgeai/config/1\"\ngenerated_at = \"2026-09-10T09:00:00Z\"\ncli_version = \"1.0.0\"\ndegraded = false\n[explore]\ntimebox_days = 5\nremedy_timebox_days = 1\n", ".devforgeai/state.toml": "schema = \"devforgeai/state/1\"\nupdated_at = \"2026-09-10T09:00:00Z\"\n[current]\nphase = \"explore\"\nid = \"IDEA-003\"\n[active]\nexplore = \"IDEA-003\"\n[explore]\nidea_id = \"IDEA-003\"\nstarted_at = \"2026-09-01T09:00:00Z\"\ntimebox_days = 5\n", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml"}}, "expect": {"grader": "asked_decision", "args": {"labels": ["Kill", "Park", "Promote"], "header": "Decision", "answer": "Kill", "artifact": ".devforgeai/explore/brief.md"}}}
{"id": "ex-08-sketch-request", "prompt": "/explore \"a tool for tracking what a book club has read\"", "answers": {"Holders": ["@first"], "Prototype": "No", "Decision": "Kill"}, "setup": {"files": {".devforgeai/config.toml": "schema = \"devforgeai/config/1\"\ngenerated_at = \"2026-09-10T09:00:00Z\"\ncli_version = \"1.0.0\"\ndegraded = false\n[explore]\ntimebox_days = 5\nremedy_timebox_days = 1\n", ".devforgeai/state.toml": "schema = \"devforgeai/state/1\"\nupdated_at = \"2026-09-10T09:00:00Z\"\n[current]\nphase = \"explore\"\nid = \"\"\n[active]\nexplore = \"\"\n[explore]\n", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml"}}, "expect": {"grader": "sketch_request_contract", "args": {"path": ".devforgeai/explore/sketch-request.json", "brief": ".devforgeai/explore/brief.md", "out_dir": ".devforgeai/explore/mockups", "seed_data_path": ".devforgeai/explore/seed-data.json", "constraint_keys": ["no_backend", "no_auth", "no_persistence", "screens_per_flow_max"]}}}
```

Fixture files named `FIXTURE:<name>` live at `skills/exploring-ideas/evals/fixtures/<name>`, and the shared runner copies them in as it copies any `setup.files` entry. The digests in `baseline_section_sha256` are the SHA-256 of the named sections of `fixtures/brief-4-flows.md`, recorded in the case line so no grader depends on a runner behavior beyond the §9 contract; `skills/exploring-ideas/evals/fixtures/digests.txt` records how each was produced.

### `evals/graders.py` — signatures and logic

Every function has the §9 signature `def <name>(workspace: str, transcript: str, args: dict) -> tuple[bool, str]`, reads only files under `workspace` and the `transcript` string, and returns `(passed, evidence)`.

- `brief_shape(workspace, transcript, args)` — parse `args["path"]`. Split the frontmatter on the first two `---` lines and compare its key order to the seven §5 keys. Split the body on `^## ` and compare the heading sequence to the twelve template headings. Parse the `Core flows` table, count data rows, check `args["min_flows"] <= n <= args["max_flows"]`, match every `ID` cell against `^FLOW-[0-9]{3}$`, check uniqueness. Count `Success signal` non-blank body lines and require 1. Evidence: the heading sequence found and the flow ids found.
- `decision_shape(workspace, transcript, args)` — parse `.devforgeai/explore/decision.yaml`. Check `decision == args["decision"]`. When `args["revisit"]` is true, parse `revisit_on` as `%Y-%m-%d` and require it later than `decided_on`; when false, require `revisit_on is None`. Check `len(carry_forward) == args["carry_forward_len"]`. Check `15 <= len(reason.split()) <= 40`. For each path in `args.get("absent", [])` require it does not exist under `workspace`; for each in `args.get("present", [])` require it does. Evidence: the decision, dates, and the paths checked.
- `carry_forward_exact(workspace, transcript, args)` — parse `carry_forward` from `args["path"]`. Require exactly seven entries, every entry holding the keys `path`, `sections`, `becomes`, `consumer`, the `consumer` sequence equal to `args["consumers"]`, and every `path` value existing under `workspace`. Apply `args["absent"]` as in `decision_shape`. Evidence: the seven `path` and `consumer` pairs.
- `remedy_touched_only(workspace, transcript, args)` — parse the `Core flows` table of `args["path"]` into `{flow_id: row_text}`. The prior state arrives in `args`, not from the workspace: `args["baseline_rows"]` maps a flow id to its row text before the run, and `args["baseline_sections"]` maps a heading to its body text before the run. Require every id in `args["changed"]` to differ from its `baseline_rows` entry, every id in `args["unchanged"]` to equal its entry after collapsing runs of spaces, and the SHA-256 hex digest of every heading's body in `args["unchanged_sections"]` to equal its `args["baseline_section_sha256"]` entry, computed over the section text with trailing whitespace stripped per line. Parse `remedied_flows` from `decision.yaml` and compare to `args["remedied_flows"]` as a set. Evidence: the ids that differed and the ids that did not.
- `open_questions_mentions(workspace, transcript, args)` — parse the frontmatter `open_questions` list from `args["path"]`. Require every id in `args["ids"]` to appear in at least one entry. Require no `Core flows` row whose `ID` cell equals any id in `args["absent_rows"]`. Require every id in `args["changed"]` to differ from its `args["baseline_rows"]` entry. Evidence: the open-question lines and the flow ids present.
- `sketch_request_contract(workspace, transcript, args)` — parse `args["path"]` as JSON. Require the seven §5 top-level keys, `mode == "sketch"`, `out_dir == args["out_dir"]`, `seed_data_path == args["seed_data_path"]`, `fidelity == "wireframe"`, and `sorted(constraints.keys()) == sorted(args["constraint_keys"])` with the three boolean constraints true. Parse the `Core flows` `ID` column of `args["brief"]` and require `[f["flow_id"] for f in flows]` to equal it, in order. Require every `flows[]` object to hold `flow_id`, `name`, `actor`, `steps`, `outcome`, with `steps` a list of 2 to 7 strings. Evidence: the flow ids in the request beside the flow ids in the brief.
- `asked_decision(workspace, transcript, args)` — scan `transcript` for the substring `"header": "` + `args["header"]`, then require every label in `args["labels"]` to appear within the 2000 characters that follow it. Evidence: the matched slice, truncated to 400 characters.

No grader opens a network connection, starts a subprocess, calls a model, or uses a random source, and none depends on a runner behavior beyond the §9 contract: the runner copies `setup.files` into a temp workspace, invokes `claude -p`, and calls the grader with `workspace`, `transcript`, and `args`. Prior-state comparisons travel in `args`. `python -c "import graders"` needs only the standard library.

## Decisions

1. **`explore prune --id <IDEA-nnn>` is accepted into the CLI surface by `specs/01-cli.md`.** It reads `decision` from `.devforgeai/explore/decision.yaml`, removes `.explore-prototype/` when that value is `kill` or `promote`, leaves it when the value is `park`, and exits 0 when the directory is already absent. The deletion is a filesystem operation with a rule attached, which §1 rule 1 places in the CLI rather than in the skill.
2. **The prototype lives at `.explore-prototype/`, outside `.devforgeai/`.** §7 fixes PreToolUse on Write and Edit to run `doc validate --producer-check` for paths under `.devforgeai/`, which would block every prototype file write. Placing the directory outside that prefix keeps §7 unchanged. The alternative, a path-prefix exemption inside `.devforgeai/`, would amend a section labeled fixed for a throwaway artifact, which is the worse trade.
3. **`design lint` exits 0 without checking for paths under `.explore-prototype/`, and for paths under `.devforgeai/explore/mockups/`.** Accepted: both paths are the last two entries of `[frontend].exclude` in the `config.toml` defaults of `specs/01-cli.md`. It is the only CLI exemption this phase asks for. Sketch mockups and the prototype are drawn before any brand kit exists, so the §7 PreToolUse token check has nothing to resolve against. `context audit` needs no exemption: it reads only `context/` and `adr/`.
   **`doc validate` needs no exemption either, because every file this skill writes under `.devforgeai/` carries the seven §5 top-level keys.** §7 runs `doc validate --producer-check` on PreToolUse for any path under `.devforgeai/`, and that hook blocks on a producer mismatch, so `seed-data.json` and `sketch-request.json` carry `schema`, `id`, `phase`, `status`, `produced_by`, `consumes`, and `open_questions` ahead of their payload, exactly as §5 allows for a YAML or JSON document. The alternative, a CLI rule that treats `.json` under `.devforgeai/explore/` as data rather than as a document, would buy the same outcome at the price of a second exemption.
4. **`devforgeai init` appends `.explore-prototype/` to the target project's `.gitignore`.** A directory the CLI deletes without asking does not belong in version control.
5. **The time box is 5 calendar days, in `config.toml` as `[explore] timebox_days = 5`, with `[explore] remedy_timebox_days = 1`.** Days are calendar days from `explore.started_at`, not working days, because a calendar comparison needs no holiday table.
6. **The `state.toml` keys are accepted and final in `specs/01-cli.md`**: `[current].phase` and `[current].id`, `[active].explore`, and the six keys of the `[explore]` table — `idea_id` (string), `started_at` (RFC 3339), `timebox_days` (integer), `remedy_started_at` (RFC 3339 or `""`), `remedy_timebox_days` (integer), `remedy_flows` (array of `FLOW-nnn`). `phase set explore --id <ID>` writes `[current]`, `[active].explore`, `idea_id`, `started_at`, `timebox_days`, and sets `remedy_flows = []`; `phase set explore --id <ID> --remedy <ids>` writes `remedy_started_at`, `remedy_timebox_days`, `remedy_flows` and leaves `started_at` unchanged, so `elapsed_days` in `decision.yaml` stays true to the original box across a remedy run.
7. **Every gate check kind this phase asked for is accepted into the closed enum of `specs/01-cli.md`, two of them renamed.** `field_in_enum`, `field_is_date`, `row_count_between`, `column_matches`, `column_contains_all`, and `elapsed_days_at_most` keep the names this spec proposed. `doc_exists` became `file_exists` with `paths` and `min_count`; `path_absent` became `file_exists` with `absent = true`; `list_length_between` became `length_between`. The gate above uses the accepted names throughout. `verifier_pass` was already in the enum and now carries the `kill-case-builder` check.
8. **The `decision-exists` check writes `paths = [".devforgeai/explore/decision.yaml"]`, not `["explore/decision.yaml"]`.** `specs/01-cli.md` resolves a `file_exists` check's `paths` against the project root rather than against `.devforgeai/`, which is what lets `prototype-pruned` reach `.explore-prototype`. The default `gates.toml` in that spec writes the shorter form for this one check, which would resolve to `<root>/explore/decision.yaml`. The prefixed form here is the one that matches the stated resolution rule; the CLI author reconciles the default file to it.
9. **Time box enforcement is the `time-box` gate check, run by the Stop hook.** With no `decision.yaml` on disk and the elapsed days past the limit, `gate check --phase explore` exits 1 and the handoff carries `Blocked you: kill, park or promote?`. §7 gives the Stop hook's behaviour: a FAIL exits 2 with `decision: "block"` under a budget of three blocks per `session_id`, and at the last block of that budget the hook exits 0 with the FAIL handoff in `systemMessage`, which is what puts the question in front of the user. The run is not interrupted mid-step and no work is discarded.
10. **ID uniqueness is scoped per `schema` value, not per project.** `brief.md` and `decision.yaml` both carry `id: IDEA-nnn`, because §5 lists both under one phase with one ID prefix. `doc validate` treats a repeated id across two different `schema` values as valid and a repeated id within one `schema` value as a violation.
11. **`consumes` is `[]` in both Explore documents, on fresh and remedy runs alike.** `consumes` holds §5 document IDs, and Explore reads no §5 document. The Discover report a remedy run reads is a CLI-written report keyed by the same `IDEA-nnn`, so listing it would produce a self-reference.
12. **Non-goals carry no IDs.** §5 fixes Explore's ID prefixes as `IDEA-nnn` and `FLOW-nnn`. Constitute cites a non-goal by its verbatim line and allocates the `CON-nnn` id itself, which keeps ID allocation with the phase that owns the prefix.
13. **Sketch screens are keyed `FLOW-nnn-nn`, not with a new prefix.** A screen belongs to exactly one flow and has no life outside it, so the flow id plus a two-digit number identifies it without adding a prefix to §5. `UI-nnn` stays with the Design phase, where a UI spec outlives the flow that prompted it.
14. **Seed data is produced by `flow-drafter` and written by the skill, not by the Design skill.** Rows derive from the entities a flow names, which exist whether or not mockups are drawn, and step 6 is skippable. Sketch mode receives `seed_data_path` as an input and returns no seed data field.
15. **`brand_sketch` is a precursor to `brand/tokens.json`, not an instance of it.** Sketch mode writes no `brand/tokens.json` and no `ui-specs/UI-nnn.md`. Those are §5 Design outputs, and Phase 0 has no brand kit to validate them against.
16. **Open questions live only in the brief's frontmatter.** §5 puts `open_questions` in every document's frontmatter; a second body section would give the same facts two homes and two chances to disagree.
17. **The prototype is a directory of static files opened by the operating system's file handler.** No build step, no dependency manifest, no package manager. This satisfies §1 rule 2, which bars a skill from naming a language or toolchain, and it matches the throwaway rule: a directory with nothing to install is a directory with nothing to keep.
18. **The prototype step is skipped on a remedy run.** A remedy rewrites named flows inside a one-day box; rebuilding a directory that step 11 deletes would spend that box on work the decision does not use.
19. **`stakeholder-analyst` is not invoked, and the holders question belongs to the skill.** Its work in this phase is one field, `holders[]`, which `idea-interrogator` drafts as `candidate_segments` and attributes from `selected_segments`. Adding a second agent inside a five-day box would cost the user a second round of questions for one array. The deeper reason the question is the skill's rather than any agent's: Claude Code removes `AskUserQuestion` from every subagent whatever its `tools` list holds, so an agent that "asks" cannot, whichever agent it is. Step 2b is the skill asking, and `idea-interrogator` is invoked twice around it — once to draft the options, once to attribute the answers.

20. **`WebSearch` and `WebFetch` are sanctioned on this skill's `allowed-tools`.** The grant is the skill's and not the agent's: `landscape-scanner` carries the two tools and does the searching, and a subagent's grant does not come from its caller. The skill keeps them for the resume path, where the model re-reads a source the scan cited to check that it still says what the brief recorded, and for a remedy run that re-checks one competitor without spending an agent invocation. No other phase carries them: market and technology research is Phase 0's work, and `discovering-requirements` states so in its own scope.
21. **`entrepreneur-assessor` is replaced by `kill-case-builder`.** It normalizes a six-dimension work-style questionnaire into a user profile. No step of this phase collects that questionnaire, and a work-style profile does not bear on whether an idea survives. `kill-case-builder` occupies its slot at step 0.6 and does the work the decision actually needs.
22. **`ui-spec-formatter` is not invoked.** It formats `devforgeai-ui-generator` output for display, and Phase 0 produces no UI spec. Sketch-mode results reach the user through the `## Mockups` table and the handoff.
23. **The five subagents carry explore-scoped names rather than the source agents' names.** §10 requires names unique across the framework, and Discover invokes `requirements-analyst`, `business-coach`, and `internet-sleuth` with different contracts. Adapting under new names lets `specs/11-subagent-catalog.md` keep both without a collision to resolve.
24. **All five subagents return JSON and none writes a §5 document.** `prototype-builder` holds the only `Write` tool, scoped to `.explore-prototype/`, which the CLI deletes. The skill writes `brief.md`, `decision.yaml`, `seed-data.json`, and `sketch-request.json`, so the PreToolUse producer check sees one producer.
25. **The skill carries no preamble at all.** Explore has no predecessor gate, so there is nothing for `gate require` to read; the only document a run might load, the Discover report, exists on remedy runs alone, and a `!` preamble line runs unconditionally and would be handed a quoted idea phrase instead of an id on a fresh run. Step 1 reads the report, where the run kind is known.
26. **Allocation moved out of the preamble and into step 1.** A preamble `doc validate --allocate IDEA` ran before the run kind was known, so a remedy or resume run spent an id it then discarded, and an exhausted `IDEA` prefix aborted a run that needed no new id at all. Step 1 establishes the run kind and allocates only on a fresh run. The allocation reserves the id on disk under `.devforgeai/.allocated/`, so two runs cannot be handed the same number.
27. **The `## Handoff` section carries three blocks rather than two.** §11 asks for a PASS example and a SEND BACK example. §5 gives Explore no send-back target, so the SEND BACK block shown is Discover's, labeled as such, and a third block shows the FAIL that this phase can actually emit: the spent time box.
28. **Blockers: none.** Every step runs on the §2 primitive list: Read, Write, Edit, Bash, PowerShell, Grep, Glob, Agent, `AskUserQuestion` in the main session, Skill, WebFetch, WebSearch, the hooks of §7, one skill that is its own entry point, five subagents, and the `devforgeai` binary. `AskUserQuestion` is counted for the skill alone, which is where step 2b and step 9 ask; no agent of this phase holds it. The one dependency outside this spec is the `designing-interfaces` sketch mode defined in `## Outputs`, which `specs/08-design.md` conforms to.
