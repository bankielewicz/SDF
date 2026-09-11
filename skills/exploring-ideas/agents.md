# Subagents · Exploring Ideas

## Invocation order

| # | Workflow step | Agent | Batch | Registered verifier |
|---|---|---|---|---|
| 1 | 2a | `idea-interrogator` | alone; fresh run and resume, skipped on a remedy run | no |
| 2 | 2c | `idea-interrogator` | alone; fresh run and resume, after the step 2b question, skipped on a remedy run | no |
| 3 | 3 | `landscape-scanner` | alone; fresh run and resume, skipped on a remedy run | no |
| 4 | 4 | `flow-drafter` | alone; every run | no |
| 5 | 7 | `prototype-builder` | alone; when the user answered `Yes` to the prototype offer, skipped on a remedy run | no |
| 6 | 8 | `kill-case-builder` | alone; every run, after the mockups and before the decision question | yes · `verifiers.kill_case` |

A row whose Batch cell names a batch is invoked in the same message as every other row naming that batch. No row here names one: step 2c consumes the answers step 2b collected, step 3 consumes step 2c's output, step 4 consumes both, step 7 consumes the sketch-mode screens, and step 8 reads the brief the earlier steps produced. A row with a condition carries it in the Batch cell after a semicolon.

Rows 1 and 2 are the two calls of one agent around one question. `idea-interrogator` drafts `candidate_segments` at 2a; the skill asks the user which of them hold the problem at 2b, with `AskUserQuestion` in the main conversation; the agent attributes the selected labels at 2c. The skill asks every user question in this phase, so no subagent carries `AskUserQuestion` — the tool is stripped from every subagent whatever its `tools` list holds, so an agent file that listed it would be broken by construction.

A subagent whose JSON does not parse against its schema is invoked once more with the parse error appended. A second failure leaves the gate metric absent, which `devforgeai gate check --phase explore` reports as exit 1.

Each of the five returns one JSON object and writes no phase document. The skill writes `brief.md`, `decision.yaml`, `seed-data.json`, and `sketch-request.json`, so the `PreToolUse` producer check sees one producer. `prototype-builder` holds the only `Write` tool, scoped to `.explore-prototype/`, which the CLI deletes at step 11.

## Contracts

| Agent | Model | Tools | Input fields | Output schema |
|---|---|---|---|---|
| `idea-interrogator` | opus | `Read` | `idea_line`, `idea_id`, `brief_path`, `selected_segments` | `agents/idea-interrogator.md` `## Output` |
| `landscape-scanner` | sonnet | `WebSearch`, `WebFetch`, `Read` | `idea_id`, `problem_statement`, `segments` | `agents/landscape-scanner.md` `## Output` |
| `flow-drafter` | opus | `Read` | `idea_id`, `brainstorm`, `scan`, `current_flows`, `current_non_goals`, `current_success_signal`, `findings` | `agents/flow-drafter.md` `## Output` |
| `prototype-builder` | sonnet | `Read`, `Write`, `Glob` | `idea_id`, `screens`, `seed_data_path`, `flow_ids`, `out_dir` | `agents/prototype-builder.md` `## Output` |
| `kill-case-builder` | opus | `Read` | `idea_id`, `brief_path`, `scan`, `prototype_built` | `agents/kill-case-builder.md` `## Output` |

Four of the five return a result the skill reads without treating it as a failure: `idea-interrogator` returns `holders: []` at 2a on every call and at 2c when the user selected no segment, and the skill writes the returned `open_questions` into the brief frontmatter and goes to step 8 with `## Core flows` empty; `landscape-scanner` returns `competitors: []` and `sources: []` on zero search results, and the skill writes one `open_questions` line naming the searched terms; `flow-drafter` returns a cited `FLOW-nnn` in `unresolved_flow_ids` when no current row carries it, and the skill writes one `open_questions` line per id and rewrites the ids that did resolve; `prototype-builder` returns `built: false` with a one-line `reason`, and the skill writes `Built | no` into the `## Prototype` table and continues to step 8. `kill-case-builder` returns `recommended_decision: "unknown"` with `confidence: 0` when the brief is too thin to argue against, and step 9 proceeds with its three options unchanged.

## Registered verifiers

| Agent | phase | report_field | unit | required |
|---|---|---|---|---|
| `kill-case-builder` | explore | `verifiers.kill_case` | objections | true |

The SubagentStop hook runs `devforgeai report ingest <name> -` for each row above.
A name absent from `config.toml` `[[verifier]]` is a no-op with exit 0 and `DFA-W411`.

The other four return run-local JSON the skill consumes, so SubagentStop ignores them. The ingested block lands at `verifiers.kill_case` of `.devforgeai/reports/IDEA-nnn-explore.yaml`, where the handoff `Verified` line reads `passed`/`total` in `objections`.

`kill-case-builder` prints one JSON object and no second one: the `devforgeai/verifier/1` envelope, whose keys are `schema` (the constant `devforgeai/verifier/1`), `subagent`, `id`, `passed`, `total`, `unit`, `findings[]` of `id`, `severity` from the closed enum `block | warn | info`, `summary` and `evidence`, and `payload`, which holds every field the agent adds of its own. Here `findings` is `[]` and `payload` carries `idea_id`, `kill_case`, `strongest_objection`, `evidence`, `recommended_decision`, and `confidence`. `total` is the length of `payload.kill_case` and `passed` the count of its entries the brief already answers, so no `warn` finding exists that could lower the ratio. Anything other than one object of that shape is `DFA-E410`, which writes the block at `status: unparsed`.

## Shared lineage

| Agent | derives_from | Other skills deriving from the same file |
|---|---|---|
| `idea-interrogator` | `C:\Users\bryan\.claude\agents\business-coach.md` — adapted | none |
| `landscape-scanner` | `C:\Users\bryan\.claude\agents\internet-sleuth.md` — adapted | none |
| `flow-drafter` | `C:\Users\bryan\.claude\agents\requirements-analyst.md` — adapted | `discovering-requirements` · `requirement-drafter`, `epic-grouper`; `planning-work` · `story-invest-auditor` |
| `prototype-builder` | `C:\Users\bryan\.claude\agents\frontend-developer.md` — adapted | `implementing-stories` · `frontend-implementer`; `designing-interfaces` · `mockup-designer` |
| `kill-case-builder` | `new` | none |

The five names are explore-scoped rather than the source agents' names, so the catalog keeps both adaptations of `requirements-analyst.md` and `frontend-developer.md` without a collision: the other skills invoke their own adaptations under different contracts.
