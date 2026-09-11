---
name: discover
description: Phase 1 of DevForgeAI. Turns a promoted Explore brief or a one-line description into .devforgeai/requirements.yaml - personas, epics and requirements, frozen by an explicit user acceptance - through a flow audit, three elicitation rounds and a fourth acceptance round. Reach for it whenever /discover runs, whenever someone asks who the users are, what the requirements are, or what sits in and out of scope for a product about to be planned, and whenever a re-open arrives from Plan, Constitute or Design as /discover IDEA-nnn --remedy REQ-nnn. It owns the artifact too - requirements.yaml with its PERSONA-nnn, EPIC-nnn and REQ-nnn ids, its revision log and its acceptance fields - so read it before touching any of them.
argument-hint: '[IDEA-nnn | "<description>"] [--remedy REQ-nnn,REQ-nnn] [--resume]'
allowed-tools: Bash(devforgeai:*), PowerShell(devforgeai:*), Read, Write, Edit, Agent, AskUserQuestion
disable-model-invocation: true
---

!`devforgeai doc load discover-entry "$ARGUMENTS[0]"`

# Discovering requirements

Phase 1 turns one input into one document. It names no technology, no repository layout and no interface element, writes no story and no Given/When/Then criterion, and runs no market research: those belong to Constitute, Plan, Design and Explore. What it produces is a requirement set the user has frozen by answering one question.

## Entry

`/discover` invokes this skill. Arguments:

| Form | Entry point | Meaning |
|---|---|---|
| `/discover IDEA-nnn` with a brief printed by the preamble | A | a promoted Explore brief |
| `/discover "<description in one line>"` | B | a typed description, no Explore phase ran |
| `/discover IDEA-nnn --remedy REQ-nnn,REQ-nnn` | C | Plan, Constitute or Design cited those ids and sent the set back |
| `/discover IDEA-nnn --resume` | resume | the flow audit sent a brief back to Explore and Explore rewrote it |

The preamble line at the head of this file runs `devforgeai doc load discover-entry "$ARGUMENTS[0]"` before the body loads. That call exits 0 in every case, so what it printed selects the entry point: the brief and the decision record for A, the prior `requirements.yaml` for C, nothing for B. It allocates no id — the `IDEA-nnn` of entry point B is allocated at step 2, once the entry point is known, so an exhausted prefix cannot abort an entry point that needs no new id.

Read from disk as the run needs them: `.devforgeai/config.toml`, `.devforgeai/state.toml` (`[current].phase`, `[current].id`), and at entry point C `.devforgeai/requirements.yaml` and the citing report under `.devforgeai/reports/`.

## Workflow

**1. Resolve the entry point.** Read `$ARGUMENTS` and the preamble output. `--remedy` selects C with the `IDEA-nnn` in `$1`; `--resume` re-enters at step 4 for the `IDEA-nnn` in `$1`; a printed brief selects A with the `IDEA-nnn` in `$1`; a preamble that printed nothing selects B with `$ARGUMENTS` as the description. An empty `$ARGUMENTS` ends the run with `Blocked   you: name an IDEA id, a description, or an IDEA id with --remedy`. At entry point A, read `decision` from the printed `decision.yaml` first: the one value the run proceeds on is `promote`, and `kill` or `park` ends the run with `Blocked   you: decision.yaml for IDEA-nnn holds <value>, not promote`. Entry point C runs the two steps under **Re-open** below before step 2. `references/entry-points.md` carries the per-step skip matrix.

**2. Fix the document id.** At A the id is the one in the brief's frontmatter. At B the CLI prints a fresh one: run `devforgeai doc validate --allocate IDEA`. At C the id is the `id` key of the loaded `requirements.yaml`. A non-zero exit from `--allocate` ends the run with the binary's stderr on the `Blocked` line. Every later step keys on this id: `state.toml`, `gate check --id`, and the report path.

**3. Set the phase.** Run `devforgeai phase set discover --id IDEA-nnn`. The CLI writes `[current].phase`, `[current].id` and `[active].discover`. Exit 3 ends the run with the binary's stderr on the `Blocked` line.

**4. Audit the flows.** Entry point A and the resume form only. Invoke the `flow-integrity-auditor` subagent with every `## Core flows` row as `{id, actor, trigger, steps, outcome}`, the brief path, and on a resume the `remedied_flows` list from `decision.yaml`. It returns the `devforgeai/verifier/1` envelope with `passed` and `total` at the top level and `payload.flows_checked`, `payload.flows_clean`, `payload.actorless[]`, and `payload.contradictions[]` under it; every entry of both arrays carries a `confidence` from `0.0` to `1.0`. A non-empty `payload.actorless` or `payload.contradictions` sends the idea back to Explore and the run stops here with no document written: see `## Send-back`. Entry points B and C skip this step. `references/flow-audit.md` carries the two judgments and the return trip.

**5. Draft the personas.** Invoke the `persona-mapper` subagent with the brief's `## Target user` lines plus the `Actor` cells of `## Core flows` (A), the description (B), or the existing `personas[]` block (C). It returns candidate persona records keyed by a kebab-case `key`. At entry point C a re-open citing no new actor reuses the existing personas and skips this step.

**6. Round 1, actors.** Ask the question in `templates/questions.md` with `AskUserQuestion`, one option per candidate from step 5, up to four. The selected candidates become `personas[]`; an unselected candidate is dropped from the run; a role named outside the options adds one persona whose `description` and `goal` come from the same reply. An empty selection re-asks the question with the same options. Entry point C skips this round.

**7. Round 2, outcomes.** Ask the second question with three to four candidate outcomes: the first drafted from the brief's `## Success signal` line and the rest from the `Outcome` cells of `## Core flows` (A), or all of them from the description (B). The selected outcomes decide the epic count at step 11, and the nth selected outcome becomes the nth epic's `success_metric` verbatim — so each candidate is drafted in the shape that field takes: one sentence carrying one measurable quantity, its unit or count, and a comparison, as in `Unmatched lines fall below 5 per night.` The measurable part is the number: `5`, the unit is `lines per night`, and `fall below` is the comparison. A candidate phrased as a direction with no quantity — `fewer unmatched lines`, `faster nightly close` — reaches `epics[].success_metric` word for word and leaves the field failing the shape rule `references/requirements-shape.md` carries, which `doc validate` reports at step 12. Entry point C skips this round.

**8. Round 3, boundaries.** Ask the third question with three to four candidate exclusions drafted from the brief's `## Non-goals` lines (A) or from the description (B). Step 11 places each selected exclusion on exactly one epic. Elicitation ends here: an unknown that survives this round becomes an entry in the owning requirement's `open_questions` rather than a fourth round, which is what keeps a run's question count predictable. Entry point C skips this round.

**9. Draft the requirements.** Invoke the `requirement-drafter` subagent with the personas round 1 kept, the outcome set, the exclusion set, the `## Problem statement` sentence, and either the `## Core flows` rows (A), the description (B), or the reopened records plus the citing report text (C). Output that does not parse as the schema in `agents/requirement-drafter.md` `## Output` goes back to the same subagent once with the parser error appended; a second unparseable output ends the run with `Blocked   you: requirement-drafter returned no parseable draft`.

**10. Allocate the ids.** Run `devforgeai doc validate --allocate PERSONA` and `devforgeai doc validate --allocate REQ`, once per new record, and keep the key-to-id map. A non-zero exit ends the run with the binary's stderr on the `Blocked` line. At entry point C the allocation covers only the records the CLI listed in `revision_log[].added`.

**11. Group the epics.** Entry points A and B: invoke the `epic-grouper` subagent with the requirement records carrying their allocated ids, the outcome set and the exclusion set. It returns one epic per selected outcome, each carrying the nth selected outcome as its `success_metric` unchanged, so the number step 7 put there is the number the field holds. Run `devforgeai doc validate --allocate EPIC` once per new epic key. At entry point C with an empty `revision_log[].added` the step is skipped and `epics[]` keeps every byte; with a non-empty one the subagent runs in placement mode and returns one existing `EPIC-nnn` per added `REQ-nnn`, so the only change to `epics[]` is the appended id in the named epic's `requirements` list.

**12. Write the document.** Write `.devforgeai/requirements.yaml` from `templates/requirements.yaml` with `status: awaiting_acceptance`, every requirement at `status: draft` (A, B) or at the status it already carries (C), and the top-level `open_questions` list holding, in ascending id order, every `REQ-nnn` whose own `open_questions` is non-empty. The PostToolUse hook runs `devforgeai doc validate` on the written path and its result reaches the model after the write as `hookSpecificOutput.additionalContext`; rewrite the field it names. `references/requirements-shape.md` gives every field, enum and constraint.

**13. Round 4, acceptance.** Ask the fourth question in `templates/questions.md` with the epic count and the requirement count. `Accept` goes to step 14. `Regroup epics` returns to step 11 with the epic block cleared. `Redraft requirements` returns to step 9 for the `REQ-nnn` ids the user names in the same reply, leaving every other record as written; a reply naming no `REQ-nnn` re-asks this question unchanged. Both non-accept answers come back to this question afterwards.

**14. Freeze the set.** Run `devforgeai doc accept requirements --id IDEA-nnn`. The CLI writes `accepted_by: user`, `accepted_at` in RFC 3339 UTC, document `status: accepted`, and moves every requirement at `draft` or `reopened` to `accepted`. Exit 1 ends the run with the binary's stderr on the `Blocked` line. The timestamp and the byte-identity of every untouched record come from the binary rather than from editing.

**15. Stop.** The Stop hook runs `devforgeai gate check --phase discover --id IDEA-nnn` and then `devforgeai handoff`. The Stop hook renders the closing block; this skill writes no part of it. On a gate FAIL the Stop hook holds the turn with the failing checks as its `reason`, and the block appears when the turn ends.

### Re-open, before step 2 at entry point C

**C1. Load the citing report.** Run `devforgeai report show IDEA-nnn <phase>`, where `<phase>` is `plan`, `constitute` or `design`, taken from the `--remedy` handoff the user pasted. Exit 1 ends the run with the binary's stderr on the `Blocked` line.

**C2. Re-open the cited ids.** Run `devforgeai doc reopen requirements --id IDEA-nnn --ids ID,ID --from <phase>`. The CLI raises `revision` by 1, appends the `revision_log` entry, moves each cited `REQ-nnn` to `reopened` in place, allocates one `REQ-nnn` per cited `UI-nnn`, returns `accepted_by` and `accepted_at` to null, sets document `status: reopened`, and leaves every other byte untouched. Exit 3 ends the run with the binary's stderr on the `Blocked` line.

The run then takes steps 2 and 3 and joins the numbered steps at step 5. `references/reopen.md` carries the field-by-field detail.

## Subagents

| Subagent | Invoked at | Passed | Returns |
|---|---|---|---|
| `flow-integrity-auditor` | step 4, alone; entry point A and the resume form | every `## Core flows` row as `{id, actor, trigger, steps, outcome}`, the brief path, `remedied_flows` on a resume | `devforgeai/verifier/1`: `passed`, `total`, `unit: flows`, `findings[]`, and `payload` holding `flows_checked`, `flows_clean`, `actorless`, `contradictions`, each array entry carrying `confidence` |
| `persona-mapper` | step 5, alone | the `## Target user` lines and the `Actor` cells (A), the description (B), the existing `personas[]` (C) | `personas[]` keyed by `key` |
| `requirement-drafter` | step 9, alone; re-entered from step 13 on `Redraft requirements` | the personas round 1 kept, the outcomes, the exclusions, the problem statement, and the flows, the description or the reopened records | `requirements[]` keyed by `key`, each with `actor_key`, `statement`, `rationale`, `acceptance_signal`, `priority`, `source`, `open_questions` |
| `epic-grouper` | step 11, alone, entry points A and B; placement mode at C; re-entered from step 13 on `Regroup epics` | the requirement records with allocated ids, the outcome set, the exclusion set | `epics[]` keyed by `key`, each with `title`, `scope`, `out_of_scope`, `success_metric`, `requirement_ids` |

Each subagent returns one JSON object and writes no document: this skill writes the file, so the PreToolUse producer check sees one producer, and this skill asks every user question. Contracts, tools, models, and invocation order are in `agents.md`. Full schemas are in each `agents/<name>.md` `## Output`. `flow-integrity-auditor` is the phase's registered verifier, so its stdout also travels through SubagentStop into the report, where the handoff `Verified` line reads it.

## Documents

| Document | Path | Template |
|---|---|---|
| Requirement set | `.devforgeai/requirements.yaml` | `templates/requirements.yaml` |

That is the only document this phase writes. It carries the seven top-level keys every `.devforgeai/` document carries, in this order and with no other key before them: `schema`, `id`, `phase`, `status`, `produced_by`, `consumes`, `open_questions`, then its own `revision`, `revision_log`, `accepted_by`, `accepted_at`, `personas`, `epics`, `requirements`. `id` is the run's `IDEA-nnn`, `phase` is `discover`, `produced_by` is `discovering-requirements`, and `consumes` holds the `IDEA-nnn` and the `FLOW-nnn` ids at entry point A and is `[]` at entry point B. `PERSONA-nnn`, `EPIC-nnn` and `REQ-nnn` ids are zero-padded three digits, allocated by the CLI.

An id that once appears in `personas[]`, `epics[]` or `requirements[]` stays in the file for the life of the project. A requirement the user drops moves to `status: withdrawn` and keeps its `id`, its `statement` and every other field as written, which is why the gate's grouping checks carry `exclude_status = ["withdrawn"]`. Field types, enums and length limits are in `references/requirements-shape.md`.

## Send-back

Discover sends back to Explore only, from entry point A and the resume form only, and at step 4 only.

| Condition | Detected by | Ids cited |
|---|---|---|
| A row pair's `Trigger`, `Steps` or `Outcome` cells cannot both hold | `payload.contradictions` non-empty | both `FLOW-nnn` of each pair |
| A row's `Actor` cell is empty, names no persona, or names a role `personas[]` does not list | `payload.actorless` non-empty | the `FLOW-nnn` |

On either condition the run writes no `requirements.yaml`, and a document at the prior revision keeps every byte. `gate check --phase discover` exits 2, `state.toml` `[last_gate]` records `result = "SEND_BACK"`, `send_back_to = "explore"` and the cited ids in `failed_checks`, and the Stop hook prints the SEND BACK block, whose two command lines the user types:

```
Next      /explore IDEA-004 --remedy FLOW-003,FLOW-007,FLOW-009
Then      /discover IDEA-004 --resume
```

The `devforgeai handoff` call in the Stop hook renders that block; this skill writes no part of it.

A third condition ends the run without a send-back: `decision.yaml` holding `kill` or `park` puts the question on the `Blocked` line at step 1, because an idea Explore did not promote has no requirement set to draft.

Discover receives send-backs from Plan, Constitute and Design. Each arrives as `/discover IDEA-nnn --remedy REQ-nnn,...` and enters at C.

## Remedy and resume

A remedy pass (`--remedy`, entry point C) changes the cited records and nothing else. Steps 4, 6, 7 and 8 are skipped; the CLI re-opens the cited ids at C2; step 9 redrafts them from the citing report's defect lines; step 11 runs only to place ids the CLI allocated for a cited `UI-nnn`; step 14 freezes the set again. `revision` rises by 1 per pass, and `revision_log` records the pass with its `from` phase, its `reopened` list and its `added` list.

A resume (`--resume`) re-enters at step 4 and audits every `## Core flows` row again, naming the `remedied_flows` ids from `decision.yaml` in the prompt as the rows Explore rewrote. Step 4 is the one step a run stops at without writing a document, which is why the resume form needs no stored cursor.

`references/reopen.md` covers the remedy pass field by field; `references/flow-audit.md` covers the resume.

## References

- `references/entry-points.md` — read at step 1: how A, B, C and the resume form are told apart, and which steps each one skips.
- `references/requirements-shape.md` — read before step 12: every top-level key, every field of `personas[]`, `epics[]` and `requirements[]`, the four enums, and the derived `open_questions` list.
- `references/reopen.md` — read before C1 at entry point C, and again at steps 9, 10 and 11: what `doc reopen` writes, what a cited `UI-nnn` allocates, and how placement mode changes step 11.
- `references/flow-audit.md` — read at step 4: the two judgments `flow-integrity-auditor` makes, what each finding cites, and the send-back and resume loop.
