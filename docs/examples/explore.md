# `/explore` — a worked walkthrough

Phase 0. One half-formed idea, carried to a recorded kill, park or promote decision. Nothing here is code: everything under `.explore-prototype/` is thrown away, and what survives is the brief, the decision record, the seed data, and the mockups.

Entry forms:

| Form | Run kind |
|---|---|
| `/explore "<idea in one line>"` | fresh |
| `/explore IDEA-nnn --remedy FLOW-nnn,FLOW-nnn` | remedy — Discover cited those flows |
| `/explore IDEA-nnn` with a brief on disk below `status: decided` | resume |

`exploring-ideas` runs **no** preamble command. Both other entry-point-allocating skills do, and this one does not, because a fresh run allocates its `IDEA-nnn` inside step 1 once the run kind is known: a remedy run takes its id from `$ARGUMENTS` and a resume run takes it from the brief, so neither spends an id and neither aborts on an exhausted `IDEA` prefix.

---

## The exchange

```
> /explore "bookkeepers spend two hours a night matching bank lines to ledger entries by hand"
```

**Step 1 — establish the run.** No `--remedy` in `$ARGUMENTS` and no brief on disk, so this is fresh:

```
$ devforgeai doc validate --allocate IDEA
IDEA-001

$ devforgeai phase set explore --id IDEA-001
Phase     0 · Explore      IDEA-001
```

Both reproduced, exit 0. `phase set` writes `[current].phase`, `[current].id`, `[active].explore`, and the whole `[explore]` table:

```toml
[current]
phase = "explore"
id = "IDEA-001"

[active]
explore = "IDEA-001"

[explore]
idea_id = "IDEA-001"
started_at = "2026-09-11T19:00:32Z"
timebox_days = 5
remedy_started_at = ""
remedy_timebox_days = 1
remedy_flows = []
```

Reproduced from `.devforgeai/state.toml`. `started_at` is what the `time-box` gate check measures against `timebox_days`.

**Step 2a — draft the candidate segments.** The `idea-interrogator` subagent returns `problem_statement`, `candidate_segments` (2 to 4, each a `label` and a `description`), `today`, `why_now`, `weak_signals`, `open_questions`, and an empty `holders`. It asks nothing: a subagent has no `AskUserQuestion` tool whatever its `tools` list holds.

**Step 2b — the skill asks you.** Fixed question, fixed header, one round:

```
AskUserQuestion(questions=[{
  "question": "IDEA-001 · nightly-bank-reconciliation: who holds this problem today?",
  "header": "Holders",
  "multiSelect": true,
  "options": [
    { "label": "Solo bookkeeper",   "description": "Keeps the books for under twenty clients, closes after 19:00" },
    { "label": "Small-firm partner","description": "Reviews the close, does not do the matching" },
    { "label": "In-house finance",  "description": "One of several duties, monthly rather than nightly" },
    { "label": "Someone else",      "description": "A segment none of the options names; type it in Other" }
  ]
}])
```

One round only. The phase holds a five-day box and a second interview spends it.

**Step 2c — attribute the selection.** `idea-interrogator` runs a second time with the labels you chose and fills `holders[]` — per label, 2 to 4 `attributes` an observer could check from outside, and a `frequency`. A selection of nothing leaves `holders` empty, writes the returned `open_questions` into the brief, and jumps to step 8 with `## Core flows` empty.

**Step 3 — scan.** `landscape-scanner` returns `competitors`, `technologies`, `closest_match`, `sources`. Zero search results come back as empty arrays plus one `open_questions` line naming the terms searched.

**Step 4 — draft the one-page spec.** `flow-drafter` takes the confirmed `holders[]` — the ones you selected at 2b, not the candidates — and returns `flows` (3 to 5), `non_goals`, `success_signal`, `seed_data`, `unresolved_flow_ids`.

**Step 5 — write the brief and the seed data.**

`.devforgeai/explore/brief.md`, twelve sections in template order:

```markdown
---
schema: devforgeai/explore-brief/1
id: IDEA-001
phase: explore
status: decided
produced_by: exploring-ideas
consumes: []
open_questions: []
---

# Nightly bank reconciliation

## Problem statement

A bookkeeper loses two hours every night matching bank lines to ledger entries by hand.

## Target user

Solo bookkeeper at a firm of under twenty clients
- keeps a spreadsheet of unmatched lines
- closes the books after 19:00

## What they do today

| Current approach | Cost | Where it breaks |
|---|---|---|
| Manual spreadsheet match | 2 hours a night | A split payment across two invoices |

## Why now

2026-03 — every bank in the region now exposes a statement API

## Competitor scan

| Name | URL | Approach | Price | Gap |
|---|---|---|---|---|
| Ledgerly | https://example.com | Rule engine | $40/mo | No split handling |

## Technology scan

| Capability | Candidate | Maturity | License | Source |
|---|---|---|---|---|
| Statement import | OFX parser | established | MIT | https://example.com |

## Core flows

| ID | Actor | Trigger | Steps | Outcome |
|---|---|---|---|---|
| FLOW-001 | Bookkeeper | Statement file arrives | import -> normalise -> stage | Lines are staged for matching |
| FLOW-002 | Bookkeeper | Staged lines exist | suggest -> confirm -> post | Matched lines are posted to the ledger |
| FLOW-003 | Bookkeeper | A line matches nothing | flag -> annotate -> defer | The unmatched line carries a reason |

## Non-goals

Tax filing

## Success signal

Unmatched lines | below 5 per night | first 30 nights

## Mockups

| Flow | Screen | Path | State |
|---|---|---|---|
| FLOW-001 | FLOW-001-01 | .devforgeai/explore/mockups/FLOW-001-01.html | default |

## Seed data

| Entity | Rows | Field count |
|---|---|---|
| statement_line | 40 | 6 |

## Prototype

| Field | Value |
|---|---|
| Built | no |
| Path | - |
| Entry | - |
| Flows covered | - |
```

This exact file was written to disk and run through the gate for the outputs below.

`consumes` is `[]` on every run, including a remedy run, because this phase reads no document another phase produced. `FLOW-nnn` ids are allocated in row order from `FLOW-001`.

`.devforgeai/explore/seed-data.json` carries the same seven envelope keys as the **first seven keys of one flat object**, beside `entities`. Not under a `meta` wrapper: `brand/tokens.json` is the only JSON document in the framework shaped that way, and a `meta` wrapper here would leave `doc validate` reading a document with no `schema`.

The `PostToolUse` hook runs `doc validate` on each write and hands its result back as `additionalContext`:

```json
{"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":"devforgeai doc validate: DFA-W202 .devforgeai/explore/brief.md cites FLOW-001, which consumes does not list\ndevforgeai doc validate: DFA-E210 .devforgeai/explore/brief.md references FLOW-001, which no document defines\n..."}}
```

Reproduced, exit 0. The write stands; the message names what to rewrite.

**Step 6 — mockups.** The skill writes `.devforgeai/explore/sketch-request.json` from its template — sixteen keys at the top level in template order, the seven envelope keys beside `mode`, `idea_id`, `brief_path`, `out_dir`, `seed_data_path`, `fidelity`, `flows`, `brand`, `constraints` — then invokes the `design` skill in sketch mode through the Skill tool with that object. `## Mockups` is filled from the returned `screens[]` and the brief moves to `status: mocked`. A non-empty `uncovered_flows` adds one `open_questions` line per id and step 7 follows either way.

**Step 7 — prototype.** One question from `templates/prototype-offer.md`. On `Yes`, `prototype-builder` writes into `.explore-prototype/`, which sits outside `.devforgeai/` so the producer check leaves its files alone; `init` added that directory to `.gitignore`. On `No`, `Built | no` and three `-` rows.

**Step 8 — the case against the idea.** `kill-case-builder` is this phase's registered verifier. Its final message is one `devforgeai/verifier/1` envelope, and `SubagentStop` ingests it:

```
$ devforgeai report ingest kill-case-builder - --id IDEA-001 --phase explore
Ingested  kill-case-builder · 3/3 objections -> .devforgeai/reports/IDEA-001-explore.yaml
```

Reproduced, exit 0, from this envelope on stdin:

```json
{"schema":"devforgeai/verifier/1","subagent":"kill-case-builder","passed":3,"total":3,
 "unit":"objections","findings":[],
 "payload":{"idea_id":"IDEA-001",
   "kill_case":"The bookkeeper may absorb the two hours rather than pay.",
   "strongest_objection":"Ledgerly already covers the simple case at a price below the switching cost.",
   "evidence":[{"claim":"No scanned competitor handles a split payment","confidence":0.6,"source":"unsourced"}],
   "recommended_decision":"promote","confidence":0.6}}
```

Each `payload.evidence` entry carries its own `confidence` and a `source`, which is the literal `unsourced` when the claim rests on nothing the brief or the scan holds.

**Step 9 — the decision.** The skill writes `payload.strongest_objection` and `payload.recommended_decision` as one line of prose, then asks:

```
AskUserQuestion(questions=[{
  "question": "IDEA-001 · nightly-bank-reconciliation: what happens to this idea?",
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

A `Park` answer is followed by one more fixed question, header `Revisit`, whose three labels are today plus 30, 90 and 180 days in `YYYY-MM-DD`. The answer fills `decision.yaml` `revisit_on`. An answer that parses as none of the three labels re-asks once; a second unparsable answer stops the run with the brief at `status: mocked`.

A `payload.recommended_decision` of `unknown` with `payload.confidence` of `0` changes nothing about this step. The agent recommends; you decide.

**Step 10 — the decision record.** `.devforgeai/explore/decision.yaml`:

```yaml
schema: devforgeai/explore-decision/1
id: IDEA-001
phase: explore
status: recorded
produced_by: exploring-ideas
consumes: []
open_questions: []
decision: promote
decided_on: 2026-09-11
reason: >-
  The nightly two-hour manual match is observable in the bookkeeper's own spreadsheet and no
  scanned competitor handles a payment split across two invoices.
revisit_on: null
elapsed_days: 1
remedied_flows: []
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

This exact file was written and gated. A `promote` carries exactly seven `carry_forward` entries in this order; the `promote-carries-forward` gate check is a `length_between` with `min = 7` and `max = 7`. A `kill` or `park` leaves the list empty, and the same check carries `empty_when` for those two values.

`elapsed_days` is counted from `[explore].started_at` to the answer date. `remedied_flows` comes from `[explore].remedy_flows`.

**Step 11 — prune the prototype.**

```
devforgeai explore prune --id IDEA-001
```

The CLI removes `.explore-prototype/` on `kill` and on `promote`, leaves it on `park`, and exits 0 when the directory is absent. Exit 1 carries `DFA-E200`, `DFA-E401`, or `DFA-E262` on stderr, and the gate's `prototype-pruned` check fails on the next Stop — which is what turns the exit code into the FAIL you see.

---

## The gate

Twelve checks, from `.devforgeai/gates.toml`:

```
$ devforgeai gate check --phase explore --id IDEA-001
Gate      explore · IDEA-001
  pass    decision-exists   file_exists
  pass    decision-enum     field_in_enum
  pass    decision-dated    field_is_date
  skip    park-has-revisit  field_is_date   not_required
  pass    promote-carries-forwardlength_between
  pass    flow-count        row_count_between
  pass    flow-id-shape     column_matches
  pass    one-success-signalrow_count_between
  pass    kill-case-answeredverifier_pass
  skip    remedy-flows-presentcolumn_contains_allcondition
  skip    time-box          elapsed_days_at_mostcondition
  pass    prototype-pruned  file_exists
Result    PASS
Report    .devforgeai/reports/IDEA-001-explore.yaml
Stack undetected; command checks skipped.
```

Reproduced, exit 0.

| Check | Kind | What it reads |
|---|---|---|
| `decision-exists` | `file_exists` | `.devforgeai/explore/decision.yaml` is there |
| `decision-enum` | `field_in_enum` | `decision` is one of `kill`, `park`, `promote` |
| `decision-dated` | `field_is_date` | `decided_on` is `%Y-%m-%d` |
| `park-has-revisit` | `field_is_date` | `revisit_on` later than `decided_on`; required only on `park`, null on `kill`/`promote` |
| `promote-carries-forward` | `length_between` | `carry_forward` holds exactly 7; empty on `kill`/`park` |
| `flow-count` | `row_count_between` | brief `## Core flows` holds 3 to 5 rows |
| `flow-id-shape` | `column_matches` | each `ID` cell matches `^FLOW-[0-9]{3}$`, unique |
| `one-success-signal` | `row_count_between` | brief `## Success signal` holds exactly 1 |
| `kill-case-answered` | `verifier_pass` | the report carries a `kill-case-builder` block; `min_ratio = 0.0` |
| `remedy-flows-present` | `column_contains_all` | every id in `[explore].remedy_flows` appears in `## Core flows`; skipped when that list is empty |
| `time-box` | `elapsed_days_at_most` | `[explore].started_at` against `timebox_days`; skipped once `decision.yaml` exists |
| `prototype-pruned` | `file_exists` with `absent = true` | `.explore-prototype` is gone; required only on `kill`/`promote` |

Before the verifier block is ingested, one check fails:

```
  fail    kill-case-answeredverifier_pass   DFA-E316 report .devforgeai/reports/IDEA-001-explore.yaml has no verifiers block for 'kill-case-builder'
Result    FAIL
```

Reproduced, exit 1.

---

## The handoff

```
$ devforgeai handoff
Phase     0 · Explore         IDEA-001 · nightly-bank-reconciliat
Done      1 ideas · 0 flows
Gate      PASS  12 checks
Verified  kill-case-builder · 3/3 objections

Next      /discover IDEA-001
Then      /constitute IDEA-001
Blocked   none

Full report: .devforgeai/reports/IDEA-001-explore.yaml
```

Reproduced, exit 0. The slug is the brief's first H1, lowercased with non-alphanumerics collapsed to hyphens and cut to 24 characters.

The Stop hook wraps it in one JSON object and nothing else:

```json
{"systemMessage":"Phase     0 · Explore         IDEA-001 · nightly-bank-reconciliat\nDone      1 ideas · 0 flows\nGate      PASS  12 checks\nVerified  kill-case-builder · 3/3 objections\n\nNext      /discover IDEA-001\nThen      /constitute IDEA-001\nBlocked   none\n\nFull report: .devforgeai/reports/IDEA-001-explore.yaml"}
```

Reproduced, exit 0.

### What a FAIL looks like

With the brief and the decision record absent, the same Stop produces case B — one object carrying both keys, exit 2:

```json
{"decision":"block","reason":"Gate      explore · IDEA-001\nResult    FAIL\nChecks    decision-exists DFA-E323 found 0 of 1 paths; missing .devforgeai/explore/decision.yaml; decision-enum DFA-E200 ...\\explore/decision.yaml not found; ... kill-case-answered DFA-E316 report .devforgeai/reports/IDEA-001-explore.yaml has no verifiers block for 'kill-case-builder'; prototype-pruned DFA-E200 ...\nFix the failing checks and stop again; the handoff prints when the turn ends.","systemMessage":"Phase     0 · Explore         IDEA-001 · -\nDone      0 ideas · 0 flows\nGate      FAIL  decision-exists DFA-E323 found 0 of 1 paths; missing...\n\nNext      /explore IDEA-001\nBlocked   none\n\nFull report: .devforgeai/reports/IDEA-001-explore.yaml"}
```

Reproduced, exit 2. `reason` is addressed to Claude and names every failing check; `systemMessage` is addressed to you and is the twelve-line block, whose `Gate` line carries only the first failing check, cut at the width rule.

Three consecutive blocks per session, then the hook lets the turn go with the FAIL block rendered.

---

## Receiving a send-back

Explore sends nothing back. It is phase 0: there is no upstream document to cite, its gate result is PASS or FAIL, and the handoff carries `Next /discover IDEA-nnn`.

It is the receiving end of one send-back, from Discover, when a `FLOW-nnn` row contradicts itself, names no actor, or duplicates another row. Discover's handoff prints the two lines you type:

```
Next      /explore IDEA-001 --remedy FLOW-002,FLOW-004
Then      /discover IDEA-001 --resume
```

A remedy run reopens the cited rows and leaves the rest of the brief alone: steps 2a to 2c, step 3, and step 7 are skipped, step 4 rewrites the cited rows only, step 6 re-sketches the cited ids only, and step 10 records `remedied_flows: ["FLOW-002","FLOW-004"]`. Its time box is `[explore].remedy_timebox_days` from `[explore].remedy_started_at`, while `[explore].started_at` stays where the fresh run put it. The handoff `Next` line becomes `/discover IDEA-001 --resume`.

A cited id absent from `## Core flows` comes back in `unresolved_flow_ids` and becomes one `open_questions` line in the form `FLOW-nnn cited by the Discover send-back is absent from Core flows`, rather than a new row.

A resume run reads the brief and takes the step after its `status`: `drafting` resumes at step 3, `scanned` at step 4, `specified` at step 6, `mocked` at step 8. Ids already allocated are kept.

---

## Files this phase owns

| Path | Template |
|---|---|
| `.devforgeai/explore/brief.md` | `templates/brief.md` |
| `.devforgeai/explore/decision.yaml` | `templates/decision.yaml` |
| `.devforgeai/explore/seed-data.json` | shape in `references/brief-sections.md` |
| `.devforgeai/explore/sketch-request.json` | `templates/sketch-request.json` |
| `.devforgeai/explore/mockups/` | written by the `design` skill in sketch mode |
| `.explore-prototype/` | written by `prototype-builder`, deleted by `explore prune` |
