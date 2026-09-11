# Subagents · Improving Framework

## Invocation order

| # | Workflow step | Agent | Batch | Registered verifier |
|---|---|---|---|---|
| 1 | 3 | `observation-miner` | batch 1 of 3 | no |
| 2 | 4 | `session-pattern-reader` | batch 1 of 3; skipped when `sessions.status` is not `present` | no |
| 3 | 5 | `debt-aggregator` | batch 1 of 3 | no |
| 4 | 7 | `recommendation-drafter` | alone; after steps 3 to 6 return | no |

A row whose Batch cell names a batch is invoked in the same message as every other row naming that batch. The three rows of batch 1 read three disjoint parts of one aggregate — the report tallies, the session history, and the deferral records — so none waits on another. `recommendation-drafter` runs alone because its input is the merged observations of steps 3 and 4 with the `OBS-nnn` ids step 6 allocated.

A row with a condition carries it in the Batch cell after a semicolon. `session-pattern-reader` carries the one condition in this skill: an absent, empty, or unreadable session directory leaves step 4 returning `observations: []` with the agent not invoked, and the run continues with every observation citing a report path.

A subagent whose JSON does not parse against its schema is invoked once more with the parse error appended. A second failure leaves that step's observations out of the document, and one `notes` line naming the agent goes into `open_questions`.

## Contracts

| Agent | Model | Tools | Input fields | Output schema |
|---|---|---|---|---|
| `observation-miner` | sonnet | `Read`, `Grep`, `Glob` | `phase_time`, `gate_failures`, `verifier_failures`, `reports`, `window_from`, `window_to` | `agents/observation-miner.md` `## Output` |
| `session-pattern-reader` | opus | `Read`, `Grep`, `Glob` | `session_files`, `session_commands`, `session_repeats`, `send_backs`, `current_phase`, `window_from`, `window_to` | `agents/session-pattern-reader.md` `## Output` |
| `debt-aggregator` | sonnet | `Read`, `Grep`, `Glob` | `deferrals`, `as_of`, `constraint_index`, `anti_pattern_index` | `agents/debt-aggregator.md` `## Output` |
| `recommendation-drafter` | opus | `Read`, `Grep`, `Glob` | `observations`, `floors`, `target_kinds`, `repo_paths` | `agents/recommendation-drafter.md` `## Output` |

`recommendation-drafter`'s `repo_paths` holds installed paths, not repository paths: the paths `Glob` returns under `.claude/skills/` and `.claude/agents/`, plus `.claude/settings.json`, `.devforgeai/gates.toml`, and `.devforgeai/config.toml`. Workflow step 7 of the skill passes exactly that set. The repository the framework is built from is not on disk in a target project, so a recommendation naming a path in that repository would name a file the user cannot open, and the six rows of `templates/rec-targets.md` are written against the installed layout for the same reason.

All four carry `memory: project`, which gives each one `.claude/agent-memory/<name>/` with `MEMORY.md` preloaded. That is what lets a window's reading build on the last one: without it `/reflect` runs over a window and forgets the previous window, `session-pattern-reader` cannot tell a loop it named last month from a new one, and `recommendation-drafter` re-proposes a change the user declined. Accumulating observations across windows is the point of the phase, so the memory scope is `project` rather than `user` or `local`.

Each agent returns a local `ref` — `om-nnn`, `sp-nnn`, `rd-nnn` — and the skill attaches the allocated `OBS-nnn` or `REC-nnn`. Two agents emitting observations in parallel would collide on a self-allocated id, which is why the ids come from `devforgeai doc validate --allocate` at steps 6 and 8 and not from the agents.

## Registered verifiers

| Agent | phase | report_field | unit | required |
|---|---|---|---|---|

No row. Every gate check of this phase is a structural property of a written YAML document, which the CLI reads directly; a verifier block would restate what `yaml_cites` already decides. So `.devforgeai/config.toml` gains no `[[verifier]]` table for this skill, the SubagentStop hook ingests nothing here, and the handoff omits its `Verified` line, which conventions §6 makes optional.

## Shared lineage

| Agent | derives_from | Other skills deriving from the same file |
|---|---|---|
| `observation-miner` | `C:\Users\bryan\.claude\agents\observation-extractor.md` — adapted | none |
| `session-pattern-reader` | `C:\Users\bryan\.claude\agents\session-miner.md` — adapted | none |
| `debt-aggregator` | `C:\Users\bryan\.claude\agents\technical-debt-analyzer.md` — adapted | none |
| `recommendation-drafter` | `C:\Users\bryan\.claude\agents\framework-analyst.md` — adapted | none |

All four are this skill's alone. `pattern-compliance-auditor.md`, `agent-generator.md`, and `diagnostic-analyst.md` are replaced rather than adapted: the first two produce an applier's output, and the third re-reads the context files whose findings reach this skill through the reports the aggregate already carries.
