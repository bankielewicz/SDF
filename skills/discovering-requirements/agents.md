# Subagents · Discovering Requirements

## Invocation order

| # | Workflow step | Agent | Batch | Registered verifier |
|---|---|---|---|---|
| 1 | 4 | `flow-integrity-auditor` | alone; entry point A and the resume form, skipped at B and C | yes · `verifiers.flow_integrity` |
| 2 | 5 | `persona-mapper` | alone; every entry point, skipped at C when the re-open cites no new actor | no |
| 3 | 9 | `requirement-drafter` | alone; every entry point, re-entered from step 13 on `Redraft requirements` | no |
| 4 | 11 | `epic-grouper` | alone; entry points A and B, placement mode at C when `revision_log[].added` is non-empty, re-entered from step 13 on `Regroup epics` | no |

A row whose Batch cell names a batch is invoked in the same message as every other row naming that batch. No row here names one: step 9 consumes the personas step 5 returned and the answers rounds 1 to 3 gave, and step 11 consumes the ids step 10 allocated for step 9's records. Step 4 gates the other three at entry point A, since a brief that fails the audit produces no draft. A row with a condition carries it in the Batch cell after a semicolon.

A subagent whose JSON does not parse against its schema is invoked once more with the parse error appended. A second failure ends the run with one `Blocked` line naming the agent.

Each of the four returns one JSON object and writes no file. The skill writes `.devforgeai/requirements.yaml`, so the `PreToolUse` producer check sees one producer, and the skill asks every user question, so no subagent carries `AskUserQuestion`. Every tool list is `Read` alone: the phase reads text it was handed and returns records.

## Contracts

| Agent | Model | Tools | Input fields | Output schema |
|---|---|---|---|---|
| `flow-integrity-auditor` | opus | `Read` | `flows`, `brief_path`, `remedied_flows` | `agents/flow-integrity-auditor.md` `## Output` |
| `persona-mapper` | sonnet | `Read` | `entry_point`, `target_user`, `flow_actors`, `description`, `personas`, `cited_actor` | `agents/persona-mapper.md` `## Output` |
| `requirement-drafter` | opus | `Read` | `personas`, `outcomes`, `exclusions`, `problem_statement`, `source_text`, `redraft_ids` | `agents/requirement-drafter.md` `## Output` |
| `epic-grouper` | sonnet | `Read` | `mode`, `requirements`, `outcomes`, `exclusions`, `added`, `epics` | `agents/epic-grouper.md` `## Output` |

`flow-integrity-auditor` returns both arrays empty on a clean brief and the run continues at step 5. `epic-grouper` gives an epic that drew no exclusion from round 3 the single `out_of_scope` entry `Nothing was named out of scope at discovery.`, which is also what every epic takes when the user selected no exclusion at all.

## Registered verifiers

| Agent | phase | report_field | unit | required |
|---|---|---|---|---|
| `flow-integrity-auditor` | discover | `verifiers.flow_integrity` | flows | true |

The SubagentStop hook runs `devforgeai report ingest <name> -` for each row above.
A name absent from `config.toml` `[[verifier]]` is a no-op with exit 0 and `DFA-W411`.

The other three return run-local JSON the skill consumes, so SubagentStop ignores them. `required` is `true` because the discover gate carries a `verifier_pass` check naming this agent, at `min_ratio = 1.0` with `on_fail = "send_back"` routing to Explore. The block lands at `verifiers.flow_integrity` of `.devforgeai/reports/IDEA-nnn-discover.yaml`, where the handoff `Verified` line reads `payload.flows_clean`/`payload.flows_checked` in `flows`, and Reflect reads it afterwards.

At that floor the ratio is the send-back: one row in `payload.actorless` or `payload.contradictions` leaves `passed` below `total`, the check fails, and the idea returns to Explore citing the `FLOW-nnn` ids those two arrays carry. A brief whose every row is clean reports `passed` equal to `total` and the run continues at step 5. The arithmetic is the whole gate here, because this agent allocates no finding ids and `findings` is `[]` on every run: there is no severity for the check to read, so an unreported row is a row the gate cannot see.

`flow-integrity-auditor` prints exactly one JSON object on stdout: the `devforgeai/verifier/1` envelope, whose keys are `schema` (the constant `devforgeai/verifier/1`), `subagent`, `id`, `passed`, `total`, `unit`, `findings[]` of `id`, `severity` from the closed enum `block | warn | info`, `summary` and `evidence`, and `payload`, which holds every top-level field the agent adds of its own. Here `findings` is `[]` — this phase allocates no finding ids — and `payload` carries `flows_checked`, `flows_clean`, `actorless`, and `contradictions`, with a `confidence` from `0.0` to `1.0` on every entry of the last two. `total` is `payload.flows_checked` and `passed` is `payload.flows_clean`. Anything other than one object of that shape is `DFA-E410`, which writes the block at `status: unparsed`.

The `actorless` `reason` enum is `empty`, `not_a_persona`, and `unlisted_role`. The third value exists so that an `Actor` cell naming a role the brief does not mention is reported rather than passed over: that is the mid-confidence case the agent exists to catch, and a rule that reported it only when nothing a person did was named at all suppressed exactly it.

## Shared lineage

| Agent | derives_from | Other skills deriving from the same file |
|---|---|---|
| `flow-integrity-auditor` | `C:\Users\bryan\.claude\agents\context-preservation-validator.md` — adapted | none |
| `persona-mapper` | `C:\Users\bryan\.claude\agents\stakeholder-analyst.md` — adapted | none |
| `requirement-drafter` | `C:\Users\bryan\.claude\agents\requirements-analyst.md` — adapted | `exploring-ideas` · `flow-drafter`; `planning-work` · `story-invest-auditor` |
| `epic-grouper` | `C:\Users\bryan\.claude\agents\requirements-analyst.md` — adapted | `exploring-ideas` · `flow-drafter`; `planning-work` · `story-invest-auditor` |

The four names are discover-scoped rather than the source agents' names, so the catalog keeps every adaptation of `requirements-analyst.md`, `stakeholder-analyst.md` and `context-preservation-validator.md` without a collision. `internet-sleuth.md`, `business-coach.md`, `entrepreneur-assessor.md` and `ideation-result-interpreter.md` reach no step of this phase: market research and technology evaluation are Explore's work, a work-style profile feeds no field of `requirements.yaml`, and the box-drawing summary is the handoff block, which `devforgeai handoff` prints.
