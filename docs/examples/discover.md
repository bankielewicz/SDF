# `/discover` — a worked walkthrough

Phase 1. One input becomes one document: `.devforgeai/requirements.yaml`, frozen by an explicit user acceptance.

It names no technology, no repository layout and no interface element, writes no story and no Given/When/Then criterion, and runs no market research. Those belong to Constitute, Plan, Design and Explore.

---

## Entry

| Form | Entry point | Meaning |
|---|---|---|
| `/discover IDEA-nnn` with a brief printed by the preamble | A | a promoted Explore brief |
| `/discover "<description in one line>"` | B | a typed description, no Explore phase ran |
| `/discover IDEA-nnn --remedy REQ-nnn,REQ-nnn` | C | Plan, Constitute or Design cited those ids |
| `/discover IDEA-nnn --resume` | resume | the flow audit sent a brief back to Explore and Explore rewrote it |

One preamble line sits at the head of `SKILL.md`:

```
!`devforgeai doc load discover-entry "$ARGUMENTS[0]"`
```

It exits 0 in every case, so **what it printed selects the entry point**: the brief and the decision record for A, the prior `requirements.yaml` for C, nothing for B. It allocates no id — entry point B's `IDEA-nnn` comes at step 2, once the entry point is known, so an exhausted prefix cannot abort an entry point that needs no new id.

The first argument is written `$ARGUMENTS[0]` rather than `$1`. Measured: a `!` command containing `$1` is refused before it runs, with `Shell command permission check failed ... Contains simple_expansion`, which aborts the whole invocation on a run that should have proceeded.

---

## The exchange

```
> /discover IDEA-001
```

**Step 1 — resolve the entry point.** The preamble printed a brief, so this is A. At A the skill reads `decision` from the printed `decision.yaml` first. The one value the run proceeds on is `promote`:

```
Blocked   you: decision.yaml for IDEA-001 holds park, not promote
```

is what a `kill` or `park` produces, and the run ends there. An idea Explore did not promote has no requirement set to draft. An empty `$ARGUMENTS` ends the run with `Blocked   you: name an IDEA id, a description, or an IDEA id with --remedy`.

**Steps 2 and 3 — fix the id and claim the phase.**

```
devforgeai phase set discover --id IDEA-001
```

Writes `[current].phase`, `[current].id`, `[active].discover`. Exit 3 ends the run with the binary's stderr on the `Blocked` line.

At A the id is the brief's frontmatter `id`. At B the CLI prints a fresh one from `devforgeai doc validate --allocate IDEA`. At C it is the `id` key of the loaded `requirements.yaml`. Every later step keys on it: `state.toml`, `gate check --id`, and the report path.

**Step 4 — audit the flows.** Entry point A and the resume form only. `flow-integrity-auditor`, this phase's registered verifier, takes every `## Core flows` row as `{id, actor, trigger, steps, outcome}`. It returns the envelope with `payload.flows_checked`, `payload.flows_clean`, `payload.actorless[]`, and `payload.contradictions[]`; every entry of both arrays carries a `confidence` from `0.0` to `1.0`.

A non-empty `actorless` or `contradictions` sends the idea back to Explore and the run stops here **with no document written**. See the send-back below. Entry points B and C skip this step.

**Step 5 — draft the personas.** `persona-mapper` returns candidate persona records keyed by a kebab-case `key`. At C a re-open citing no new actor reuses the existing personas and skips this step.

**Steps 6, 7, 8 — three elicitation rounds.** Each is one `AskUserQuestion` with a fixed header from `templates/questions.md`. Entry point C skips all three.

*Round 1, actors.* One option per candidate from step 5, up to four. Selected candidates become `personas[]`; an unselected candidate is dropped from the run; a role named outside the options adds one persona whose `description` and `goal` come from the same reply. An empty selection re-asks with the same options.

*Round 2, outcomes.* Three to four candidates: the first drafted from the brief's `## Success signal` line and the rest from the `Outcome` cells of `## Core flows`. The nth selected outcome becomes the nth epic's `success_metric` **verbatim**, so each candidate is drafted in the shape that field takes — one sentence carrying one measurable quantity, its unit or count, and a comparison:

```
Unmatched lines fall below 5 per night.
```

The measurable part is `5`, the unit is `lines per night`, and `fall below` is the comparison. A candidate phrased as a direction with no quantity — `fewer unmatched lines`, `faster nightly close` — reaches `epics[].success_metric` word for word and leaves the field failing the shape rule, which `doc validate` reports at step 12.

*Round 3, boundaries.* Three to four candidate exclusions drafted from the brief's `## Non-goals` lines. Step 11 places each selected exclusion on exactly one epic. Elicitation ends here: an unknown that survives this round becomes an entry in the owning requirement's `open_questions` rather than a fourth round, which is what keeps a run's question count predictable.

**Step 9 — draft the requirements.** `requirement-drafter` takes the kept personas, the outcome set, the exclusion set, the `## Problem statement` sentence, and the flows. Output that does not parse goes back to the same subagent once with the parser error appended; a second unparseable output ends the run with `Blocked   you: requirement-drafter returned no parseable draft`.

**Step 10 — allocate the ids.**

```
devforgeai doc validate --allocate PERSONA
devforgeai doc validate --allocate REQ
```

once per new record. A non-zero exit ends the run with the binary's stderr on the `Blocked` line. At C the allocation covers only the records the CLI listed in `revision_log[].added`.

**Step 11 — group the epics.** `epic-grouper` returns one epic per selected outcome, each carrying the nth selected outcome as its `success_metric` unchanged, so the number round 2 put there is the number the field holds. Then `devforgeai doc validate --allocate EPIC` once per new epic key.

**Step 12 — write the document.**

```yaml
schema: devforgeai/requirements/1
id: IDEA-001
phase: discover
status: awaiting_acceptance
produced_by: discovering-requirements
consumes: [IDEA-001, FLOW-001, FLOW-002, FLOW-003]
open_questions: []
revision: 1
revision_log: []
accepted_by: null
accepted_at: null

personas:
  - id: PERSONA-001
    name: Solo bookkeeper
    description: Keeps the books for under twenty clients and closes them nightly.
    goal: Finish the nightly close without a manual spreadsheet match.

epics:
  - id: EPIC-001
    title: Nightly statement reconciliation
    scope: Import a bank statement, suggest matches, and post the confirmed ones.
    out_of_scope:
      - Tax filing
    success_metric: Unmatched lines fall below 5 per night.
    requirements:
      - REQ-001
      - REQ-002

requirements:
  - id: REQ-001
    actor: PERSONA-001
    statement: The system imports a bank statement file and stages every line for matching.
    rationale: The bookkeeper cannot match lines that are not in the system.
    acceptance_signal: Every line in the imported file appears in the staging list.
    priority: must
    source: FLOW-001
    status: draft
    open_questions: []
    traces_to: []
```

Shaped from `templates/requirements.yaml`. The top-level `open_questions` list holds, in ascending id order, every `REQ-nnn` whose own `open_questions` is non-empty.

`priority` is the MoSCoW set `must | should | could | wont`. A YAML scalar is not instruction prose, so the ceremony rule does not reach it.

The `PostToolUse` hook runs `doc validate` on the write and hands its result back as `additionalContext`, naming a field to rewrite.

**Step 13 — round 4, acceptance.** The fourth question, with the epic count and the requirement count:

| Answer | What happens |
|---|---|
| `Accept` | go to step 14 |
| `Regroup epics` | return to step 11 with the epic block cleared |
| `Redraft requirements` | return to step 9 for the `REQ-nnn` ids named in the same reply, leaving every other record as written |

A `Redraft` reply naming no `REQ-nnn` re-asks this question unchanged. Both non-accept answers come back to this question afterwards.

**Step 14 — freeze the set.**

```
devforgeai doc accept requirements --id IDEA-001
```

The CLI writes `accepted_by: user`, `accepted_at` in RFC 3339 UTC, document `status: accepted`, and moves every requirement at `draft` or `reopened` to `accepted`. Exit 1 ends the run with the binary's stderr on the `Blocked` line. The timestamp and the byte-identity of every untouched record come from the binary rather than from an edit.

---

## The gate

Five checks, from `.devforgeai/gates.toml`:

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
  field = "accepted_by"
  message = "requirements.yaml is not accepted"
```

Reproduced from the file `init` wrote.

`requires = "explore"` is what the `UserPromptExpansion` hook enforces before the skill loads, because this skill's preamble carries a `doc load` rather than a `gate require`:

```
$ echo '{"command_name":"discover","command_args":"IDEA-001"}' | devforgeai hook run prompt-expansion
```

Reproduced: no output, exit 0, because the explore gate had passed. Without it, the hook returns `{"decision":"block","reason":"DFA-E321 ..."}` at exit 2 and the body does not load.

`exclude_status = ["withdrawn"]` on two checks is why a requirement you drop keeps its id: it moves to `status: withdrawn` and keeps its `statement` and every other field as written. An id that once appears in `personas[]`, `epics[]` or `requirements[]` stays in the file for the life of the project.

---

## The handoff

```
Phase     1 · Discover        IDEA-001 · -
Done      2 REQ · 1 EPIC · 1 personas
Gate      PASS  5 checks
Verified  flow-integrity-auditor · 3/3 flows

Next      /constitute IDEA-001
Then      /plan EPIC-001
Blocked   none

Full report: .devforgeai/reports/IDEA-001-discover.yaml
```

The `Done` counts are measured by the binary from the id index over `requirements.yaml`; the `Verified` line reads the `flow-integrity-auditor` block `SubagentStop` ingested.

---

## The send-back to Explore

Discover sends back to Explore only, and there are two points it can happen from.

**At step 4, before any document exists.** Entry point A and the resume form run `flow-integrity-auditor` over the brief's `## Core flows`, and a non-empty `actorless` or `contradictions` stops the run there with no `requirements.yaml` written.

**At the gate.** The `discover-flows` check is a `verifier_pass` on the same auditor with `on_fail = "send_back"`, so a ratio below 1.0 in the ingested `verifiers.flow_integrity` block sends back from the gate as well. That closes a gap a live run found: the send-back used to be decided only at step 4, so a run that got past it and then produced a flow defect reached a PASS with nothing to catch it. Both paths name the same destination.

| Condition | Detected by | Ids cited |
|---|---|---|
| A row pair's `Trigger`, `Steps` or `Outcome` cells cannot both hold | `payload.contradictions` non-empty | both `FLOW-nnn` of each pair |
| A row's `Actor` cell is empty, names no persona, or names a role `personas[]` does not list | `payload.actorless` non-empty | the `FLOW-nnn` |

On either condition the run writes no `requirements.yaml`, and a document at the prior revision keeps every byte. `state.toml` `[last_gate]` records `result = "SEND_BACK"`, `send_back_to = "explore"`, and the cited ids in `failed_checks`. The Stop hook prints the block with **no** blocking decision, at exit 0, and the two command lines are yours to type:

```
Next      /explore IDEA-001 --remedy FLOW-002,FLOW-004
Then      /discover IDEA-001 --resume
```

A third condition ends the run without a send-back: `decision.yaml` holding `kill` or `park` puts the question on the `Blocked` line at step 1.

---

## Receiving a send-back — the remedy pass

Discover receives send-backs from Plan, Constitute and Design. Each arrives as `/discover IDEA-nnn --remedy REQ-nnn,...` and enters at C, which runs two steps before step 2.

**C1.** `devforgeai report show IDEA-001 <phase>`, where `<phase>` is `plan`, `constitute` or `design`, taken from the `--remedy` handoff you pasted. Exit 1 ends the run with the binary's stderr on the `Blocked` line.

**C2.**

```
devforgeai doc reopen requirements --id IDEA-001 --ids REQ-014,REQ-019 --from plan
```

The CLI raises `revision` by 1, appends the `revision_log` entry, moves each cited `REQ-nnn` to `reopened` in place, allocates one `REQ-nnn` per cited `UI-nnn`, returns `accepted_by` and `accepted_at` to null, sets document `status: reopened`, and leaves every other byte untouched. Exit 3 ends the run with the binary's stderr on the `Blocked` line.

The run then takes steps 2 and 3 and joins at step 5. Steps 4, 6, 7 and 8 are skipped. Step 9 redrafts the cited records from the citing report's defect lines; step 11 runs only to place ids the CLI allocated for a cited `UI-nnn`; step 14 freezes the set again.

`revision` rises by 1 per pass, and `revision_log` records the pass with its `from` phase, its `reopened` list and its `added` list.

A `--resume` re-enters at step 4 and audits every `## Core flows` row again, naming the `remedied_flows` ids from `decision.yaml` in the prompt as the rows Explore rewrote. Step 4 is the one step a run stops at without writing a document, which is why the resume form needs no stored cursor.

---

## Files this phase owns

| Path | Template |
|---|---|
| `.devforgeai/requirements.yaml` | `templates/requirements.yaml` |

One file. It carries the seven envelope keys in order, then its own `revision`, `revision_log`, `accepted_by`, `accepted_at`, `personas`, `epics`, `requirements`. `consumes` holds the `IDEA-nnn` and the `FLOW-nnn` ids at entry point A and is `[]` at entry point B.
