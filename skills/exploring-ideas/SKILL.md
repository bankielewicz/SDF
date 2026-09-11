---
name: explore
description: Phase 0 of DevForgeAI. Carries one half-formed idea from a sentence to a recorded kill, park or promote decision through six steps - brainstorm, competitor and technology scan, a one-page brief, wireframe mockups, an optional clickable prototype, and the decision - writing .devforgeai/explore/brief.md and .devforgeai/explore/decision.yaml. Reach for it whenever /explore runs, whenever someone brings a raw product idea, a "is this worth building" question, or a time-boxed spike that precedes requirements, and whenever a Discover send-back arrives as /explore IDEA-nnn --remedy FLOW-nnn. It also owns the artifacts - the explore brief, its FLOW-nnn core flows, the decision record, seed-data.json, sketch-request.json, the mockups directory, and the throwaway .explore-prototype directory - so read it before touching any of them.
argument-hint: '"<idea in one line>" | IDEA-nnn [--remedy FLOW-nnn,...]'
allowed-tools: Bash(devforgeai:*), PowerShell(devforgeai:*), Read, Write, Edit, Glob, Grep, Agent, AskUserQuestion, Skill, WebSearch, WebFetch
disable-model-invocation: true
---

# Exploring ideas

Phase 0 is a time-boxed loop over one idea. It ends with a decision on disk, not with code: everything under `.explore-prototype/` is thrown away, and what survives is the brief, the decision record, the seed data, and the mockups.

## Entry

`/explore` invokes this skill. Arguments:

| Form | Run kind | Meaning |
|---|---|---|
| `/explore "<idea in one line>"` | fresh | a new idea, free text |
| `/explore IDEA-nnn --remedy FLOW-nnn,FLOW-nnn` | remedy | Discover cited those flows and sent the idea back |
| `/explore IDEA-nnn` with a brief on disk below `status: decided` | resume | continue where the last run stopped |

This skill runs no preamble command. The `IDEA-nnn` is allocated at step 1, on a fresh run alone, once the run kind is known: a remedy run takes its id from `$ARGUMENTS` and a resume run takes it from the brief, so neither spends an id and neither aborts on an exhausted `IDEA` prefix. Allocation in a preamble runs before the run kind is known, which is why it sits in the step that needs the id.

Read from disk as the run needs them: `.devforgeai/state.toml` (`[current].phase`, `[current].id`, `[active].explore`, `[explore].*`), `.devforgeai/config.toml` (`[explore] timebox_days`, `[explore] remedy_timebox_days`), and on a remedy run `.devforgeai/explore/brief.md`, `.devforgeai/explore/decision.yaml`, and `.devforgeai/reports/IDEA-nnn-discover.yaml`.

## Workflow

**1. Establish the run.** The run is a remedy run when `$ARGUMENTS` holds `--remedy`, a resume when `.devforgeai/explore/brief.md` exists with a `status` other than `decided`, and a fresh run otherwise.

- Fresh run: run `devforgeai doc validate --allocate IDEA` and take the printed id, then run `devforgeai phase set explore --id IDEA-nnn`. Exit 1 on `DFA-E215` means the prefix is exhausted; the run stops with that stderr on the `Blocked` line.
- Remedy run: take the id from `$ARGUMENTS`, read the Discover report and the brief, run `devforgeai phase set explore --id IDEA-nnn --remedy FLOW-nnn,FLOW-nnn` with the ids from `$ARGUMENTS`, then continue at step 4.
- Resume: read the brief and continue at the step after its `status`.

`phase set` writes `[current].phase`, `[current].id`, `[active].explore`, and the `[explore]` table. A non-zero exit stops the run with its stderr in context, and the Stop hook renders the closing block. `references/remedy-run.md` carries what else a remedy run does differently.

**2a. Draft the candidate segments.** Invoke the `idea-interrogator` subagent with the idea line, the run's `IDEA-nnn`, and the brief when resuming. It returns `problem_statement`, `candidate_segments` (2 to 4 entries, each a `label` and a `description`), `today`, `why_now`, `weak_signals`, `open_questions`, and an empty `holders`. The agent drafts candidates and asks nothing: a subagent has no `AskUserQuestion` tool, so the question below is this skill's.

**2b. Ask who holds the problem — user, AskUserQuestion.** Input: `candidate_segments` from step 2a. The question and its options are fixed, and `templates/questions.md` carries the same block; `<label>` and `<description>` are taken from the entries in the order returned:

```
AskUserQuestion(questions=[{
  "question": "<IDEA-nnn> · <slug>: who holds this problem today?",
  "header": "Holders",
  "multiSelect": true,
  "options": [
    { "label": "<candidate_segments[0].label>", "description": "<candidate_segments[0].description>" },
    { "label": "<candidate_segments[1].label>", "description": "<candidate_segments[1].description>" },
    { "label": "<candidate_segments[2].label>", "description": "<candidate_segments[2].description>" },
    { "label": "Someone else",                  "description": "A segment none of the options names; type it in Other" }
  ]
}])
```

One round. The phase holds a five-day box and a second interview spends it. Output: the selected labels, plus any free text the user typed under `Someone else`.

**2c. Attribute the selected segments.** Invoke `idea-interrogator` a second time with the selected labels and the step 2a return. It fills `holders[]` — for each selected label, 2 to 4 `attributes` an observer could check from outside and a `frequency` — and returns `today`, `why_now`, `weak_signals`, `open_questions` unchanged. A user who selected nothing leaves `holders` empty: write the returned `open_questions` into the brief frontmatter and go to step 8 with `## Core flows` empty. Steps 2a to 2c are skipped on a remedy run.

**3. Scan.** Invoke the `landscape-scanner` subagent with `problem_statement` and the `holders[].segment` values of step 2c. It returns `competitors`, `technologies`, `closest_match`, `sources`. Zero search results come back as `competitors: []` and `sources: []`; write one `open_questions` line naming the terms that were searched and carry on. Skipped on a remedy run.

**4. Draft the one-page spec.** Invoke the `flow-drafter` subagent with the step 2c JSON — the confirmed `holders[]` the user selected at 2b, with `problem_statement`, `today`, `why_now`, `weak_signals` — and the step 3 JSON. The flows are drawn from the holders the user confirmed rather than from candidates. It returns `flows` (3 to 5), `non_goals`, `success_signal`, `seed_data`, `unresolved_flow_ids`. On a remedy run, pass the current `## Core flows` rows, `## Non-goals` lines, `## Success signal` line, and the cited `findings[]` of the Discover report; the agent returns only the cited rows in `flows` and every other field unchanged. Ids in `unresolved_flow_ids` were cited but are absent from the brief: write one `open_questions` line per id in the form `FLOW-nnn cited by the Discover send-back is absent from Core flows`, and rewrite the ids that did resolve.

**5. Write the brief and the seed data.** Write `.devforgeai/explore/brief.md` from `templates/brief.md` with all twelve sections filled and `status: specified`. Then read `templates/seed-data.json` and write `.devforgeai/explore/seed-data.json` as that template with its placeholders filled from the `seed_data` field step 4 returned: `id` takes the run's `IDEA-nnn`, and `entities[]` takes one object per entity with its `name`, its `fields` list, and 5 to 20 `rows` keyed by those fields. The written file holds the template's eight keys at the top level in template order and no others — the seven envelope keys `schema`, `id`, `phase`, `status`, `produced_by`, `consumes`, `open_questions` sit at the top level beside `entities`, in one flat object. `.devforgeai/brand/tokens.json` is the one JSON document in the framework that carries the envelope under a `meta` object; the seed data is not that shape, and a `meta` wrapper here leaves `doc validate` reading a document with no `schema`. `references/brief-sections.md` gives the content rule for each brief section and the per-entity rules. On a remedy run only the cited rows of `## Core flows`, the frontmatter `open_questions`, and `status` change. The PostToolUse hook runs `devforgeai doc validate` on the written path and its result reaches the model after the write as `hookSpecificOutput.additionalContext`; rewrite the section it names.

**6. Mockups.** Read `templates/sketch-request.json`, then write `.devforgeai/explore/sketch-request.json` as that template with its placeholders filled: `id` and `idea_id` take the run's `IDEA-nnn`, `flows[]` takes one object per `## Core flows` row in row order, `brand.palette_hint` and `brand.type_hint` come from `## Target user`, and `seed_data_path` is the seed data path. The written file holds the template's sixteen keys at the top level in template order and no others — the seven envelope keys `schema`, `id`, `phase`, `status`, `produced_by`, `consumes`, `open_questions` sit at the top level beside `mode`, `idea_id`, `brief_path`, `out_dir`, `seed_data_path`, `fidelity`, `flows`, `brand`, `constraints`, in one flat object. `.devforgeai/brand/tokens.json` is the one JSON document in the framework that carries the envelope under a `meta` object; this request is not that shape, and a `meta` wrapper here leaves `doc validate` reading a document with no `schema`. Then, through the Skill tool, invoke the `design` skill in sketch mode with that object. Fill `## Mockups` from the returned `screens[]` and move the brief to `status: mocked`. A non-empty `uncovered_flows` adds one `open_questions` line per id, and step 7 follows either way. `references/sketch-mode.md` holds both sides of the contract.

**7. Prototype.** Ask the question in `templates/prototype-offer.md`. On `Yes`, invoke the `prototype-builder` subagent with the mockup paths, the seed data path, the flows to make clickable, and the output root `.explore-prototype/`; fill `## Prototype` from what it returns. On `No`, write `Built | no` and the three `-` rows. `built: false` comes back with a `reason` string, which is the same table row and the same continuation to step 8. Skipped on a remedy run. Detail in `references/prototype.md`.

**8. Build the case against the idea.** Invoke the `kill-case-builder` subagent with `.devforgeai/explore/brief.md`, the step 3 JSON, and the `Built` row of `## Prototype`. It returns the `devforgeai/verifier/1` envelope with `passed` and `total` at the top level and `payload.idea_id`, `payload.kill_case`, `payload.strongest_objection`, `payload.evidence`, `payload.recommended_decision`, and `payload.confidence` under it. Each `payload.evidence` entry carries its own `confidence` and a `source`, which is the literal `unsourced` when the claim rests on nothing the brief or the scan holds. The SubagentStop hook ingests its stdout into `.devforgeai/reports/IDEA-nnn-explore.yaml`. A `payload.recommended_decision` of `unknown` with `payload.confidence` of `0` changes nothing about step 9.

**9. Ask the decision.** Write `payload.strongest_objection` and `payload.recommended_decision` as one line of prose, then ask the fixed question with `AskUserQuestion`:

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

`Park` is followed by a second fixed question, with `<d+30>`, `<d+90>`, `<d+180>` replaced by those dates computed from today in `YYYY-MM-DD`:

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

The answer fills `decision.yaml` `revisit_on`. An answer that parses as none of the three labels re-asks the same question once; a second unparsable answer stops the run with the brief at `status: mocked`.

**10. Write the decision record.** Write `.devforgeai/explore/decision.yaml` from `templates/decision.yaml` with the step 9 answer, `elapsed_days` counted from `[explore].started_at` to the answer date, and `remedied_flows` from `[explore].remedy_flows`. Move the brief frontmatter to `status: decided`. `references/decision-record.md` gives each field and the seven `carry_forward` entries a `promote` carries. The PostToolUse `doc validate` result reaches the model after the write as `additionalContext`; rewrite the key it names.

**11. Prune the prototype.** Run `devforgeai explore prune --id IDEA-nnn`. The CLI removes `.explore-prototype/` on `kill` and on `promote`, leaves it on `park`, and exits 0 when the directory is absent. Exit 1 carries `DFA-E200`, `DFA-E401`, or `DFA-E262` on stderr, and the gate's `prototype-pruned` check fails on the next Stop, which is what turns the exit code into the FAIL the user sees.

**12. Close.** The Stop hook runs `devforgeai gate check --phase explore --id IDEA-nnn` and `devforgeai handoff --phase explore --id IDEA-nnn`. The Stop hook renders the closing block; this skill writes no part of it. On a gate FAIL the Stop hook holds the turn with the failing checks as its `reason`, and the block appears when the turn ends.

## Subagents

| Subagent | Invoked at | Passed | Returns |
|---|---|---|---|
| `idea-interrogator` | step 2a, alone; skipped on a remedy run | the idea line, `IDEA-nnn`, the brief when resuming | `problem_statement`, `candidate_segments`, `today`, `why_now`, `weak_signals`, `open_questions`, `holders` empty |
| `idea-interrogator` | step 2c, alone, after the user answers; skipped on a remedy run | the labels selected at 2b as `selected_segments`, and the step 2a return | `holders[]` with `attributes` and `frequency` per selected label, the other fields unchanged |
| `landscape-scanner` | step 3, alone; skipped on a remedy run | `problem_statement`, `holders[].segment`, `IDEA-nnn` | `competitors`, `technologies`, `closest_match`, `sources` |
| `flow-drafter` | step 4, alone | the step 2c JSON with the confirmed `holders[]`, and the step 3 JSON; on a remedy run the current flows, non-goals, success signal, and the cited findings | `flows`, `non_goals`, `success_signal`, `seed_data`, `unresolved_flow_ids` |
| `prototype-builder` | step 7, alone, on a `Yes` answer | `screens[]`, the seed data path, the `FLOW-nnn` ids to make clickable, the output root | `built`, `entry`, `files`, `flows_covered`, `reason` |
| `kill-case-builder` | step 8, alone, after the mockups | the brief, the step 3 JSON, the `Built` row | `devforgeai/verifier/1`: `passed`, `total`, `unit: objections`, `findings[]` empty, and `payload` holding `idea_id`, `kill_case`, `strongest_objection`, `evidence`, `recommended_decision`, `confidence` |

Each agent returns one JSON object, writes no phase document, and asks the user nothing: this skill asks every user question, because a subagent has no `AskUserQuestion` tool whatever its `tools` list holds. `agents.md` carries the full contracts, the tool lists, and the invocation order. `kill-case-builder` is the phase's registered verifier, so its stdout also travels through SubagentStop into the report.

## Documents

| Document | Path | Template |
|---|---|---|
| Brief | `.devforgeai/explore/brief.md` | `templates/brief.md` |
| Decision record | `.devforgeai/explore/decision.yaml` | `templates/decision.yaml` |
| Seed data | `.devforgeai/explore/seed-data.json` | `templates/seed-data.json` |
| Sketch request | `.devforgeai/explore/sketch-request.json` | `templates/sketch-request.json` |
| Mockups | `.devforgeai/explore/mockups/` | written by the `design` skill in sketch mode |
| Prototype | `.explore-prototype/`, entry `index.html` | written by `prototype-builder`, deleted by the CLI |

Every file this skill writes under `.devforgeai/` carries the same seven top-level keys, in this order and with no other top-level key: `schema`, `id`, `phase`, `status`, `produced_by`, `consumes`, `open_questions`. The two JSON documents, `seed-data.json` and `sketch-request.json`, carry them as the first seven keys of one flat object — each is written from its own template in `templates/` — the way `brief.md` and `decision.yaml` carry them as their first seven YAML keys. `id` is the run's `IDEA-nnn`, `phase` is `explore`, `produced_by` is `exploring-ideas`, and `consumes` is `[]` on every run, including a remedy run, because this phase reads no document another phase produced. `FLOW-nnn` ids are zero-padded three digits, allocated in row order from `FLOW-001`. Sections appear in the template order with the template's heading text. Open questions live in the frontmatter `open_questions` list and in no body section.

The prototype sits outside `.devforgeai/` so the PreToolUse producer check leaves its files alone, and `devforgeai init` adds `.explore-prototype/` to the target project's `.gitignore`.

## Send-back

This phase sends nothing back. It is Phase 0: there is no upstream document to cite, its gate result is `PASS` or `FAIL`, and the handoff carries `Next /discover IDEA-nnn`.

It is the receiving end of one send-back. Discover emits it when a `FLOW-nnn` row contradicts itself, names no actor, or duplicates another row, and Discover's handoff prints the two lines the user types:

```
Next      /explore IDEA-001 --remedy FLOW-002,FLOW-004
Then      /discover IDEA-001 --resume
```

The `devforgeai handoff` call in the Stop hook renders every closing block, Explore's included; this skill writes no part of it.

## Remedy and resume

A remedy run reopens the cited `FLOW-nnn` rows and leaves the rest of the brief alone: steps 2a to 2c, step 3, and step 7 are skipped, step 4 rewrites the cited rows only, step 6 re-sketches the cited ids only, and step 10 records `remedied_flows: ["FLOW-002","FLOW-004"]`. Its time box is `[explore].remedy_timebox_days` counted from `[explore].remedy_started_at`, while `[explore].started_at` stays where the fresh run put it. The handoff `Next` line becomes `/discover IDEA-nnn --resume`. A cited id that is absent from `## Core flows` comes back in `unresolved_flow_ids` and becomes an `open_questions` line rather than a new row.

A resume run reads the brief, takes the step after its `status` (`drafting` resumes at step 3, `scanned` at step 4, `specified` at step 6, `mocked` at step 8), and keeps the ids already allocated.

`references/remedy-run.md` holds the fresh-run and remedy-run comparison in full.

## References

- `references/brief-sections.md` — read at step 5: the content rule for each of the twelve brief sections, and the `seed-data.json` shape.
- `references/sketch-mode.md` — read at step 6: the sketch-mode input and output objects, and how the `## Mockups` table is filled from `screens[]`.
- `references/prototype.md` — read at step 7: what goes in `.explore-prototype/`, what stays out, and the `## Prototype` table.
- `references/decision-record.md` — read at step 10: every `decision.yaml` field and the seven `carry_forward` entries of a `promote`.
- `references/remedy-run.md` — read at step 1 on a remedy run, and again before steps 4, 5, 6 and 10: how a remedy run differs from a fresh run, field by field.
