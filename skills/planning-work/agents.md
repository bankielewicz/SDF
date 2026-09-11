# Subagents · Planning Work

## Invocation order

| # | Workflow step | Agent | Batch | Registered verifier |
|---|---|---|---|---|
| 1 | 5 | `story-decomposer` | alone | no |
| 2 | 7 | `story-file-set-planner` | alone | no |
| 3 | 9 | `story-invest-auditor` | alone | yes · `verifiers.story_invest` |
| 4 | 10 | `story-invest-auditor` | alone; second pass over the edited set | yes · `verifiers.story_invest` |
| 5 | 11 | `sprint-sequencer` | alone | no |
| 6 | R2 | `spec-gap-triager` | alone; remedy runs only | no |

A row whose Batch cell names a batch is invoked in the same message as every other row naming that batch. No row here names one: step 7 consumes step 5's drafts, step 9 reads the files step 8 wrote from both, step 10 re-reads what step 9's findings changed, and step 11 reads the dependency lists step 8 recorded. A row with a condition carries it in the Batch cell after a semicolon.

A subagent whose JSON does not parse against its schema is invoked once more with the parse error appended. A second failure leaves the gate metric absent, which `devforgeai gate check --phase plan` reports as exit 1.

## Contracts

| Agent | Model | Tools | Input fields | Output schema |
|---|---|---|---|---|
| `story-decomposer` | opus | `Read`, `Grep`, `Glob` | `epic`, `requirements`, `layers`, `constraints`, `layer_rules`, `anti_patterns`, `story_points` | `agents/story-decomposer.md` `## Output` |
| `story-file-set-planner` | sonnet | `Read`, `Grep`, `Glob` | `drafts`, `roots`, `directory_map`, `placement_rules`, `naming`, `excluded_globs`, `testing_standards`, `repo_tree` | `agents/story-file-set-planner.md` `## Output` |
| `story-invest-auditor` | opus | `Read`, `Grep`, `Glob` | `sprint_id`, `story_paths`, `epic`, `requirements`, `constraints`, `layer_rules` | `agents/story-invest-auditor.md` `## Output` |
| `sprint-sequencer` | sonnet | `Read`, `Glob`, `Grep` | `epic`, `stories`, `points_max`, `points_max_source` | `agents/sprint-sequencer.md` `## Output` |
| `spec-gap-triager` | opus | `Read`, `Grep`, `Glob` | `triples`, `criterion_text`, `requirement`, `constraints`, `ui_ids` | `agents/spec-gap-triager.md` `## Output` |

## Registered verifiers

| Agent | phase | report_field | unit | required |
|---|---|---|---|---|
| `story-invest-auditor` | plan | `verifiers.story_invest` | stories | true |

The SubagentStop hook runs `devforgeai report ingest <name> -` for each row above.
A name absent from `config.toml` `[[verifier]]` is a no-op with exit 0 and `DFA-W411`.

The other four return run-local JSON the skill consumes, so SubagentStop ignores them.

`story-invest-auditor` prints exactly one JSON object on stdout: the `devforgeai/verifier/1` envelope, whose keys are `schema` (the constant `devforgeai/verifier/1`), `subagent`, `id`, `passed`, `total`, `unit`, `findings[]` of `id`, `severity` from the closed enum `block | warn | info`, `confidence` from `0.0` to `1.0`, `stands_against`, `summary` and `evidence`, and `payload`, which holds every top-level field the agent adds of its own. This agent adds none, so `payload` is `{}`. Anything other than one object of that shape is `DFA-E410`, which writes the block at `status: unparsed`.

`total` is the story count and `passed` is `total` minus the number of stories a `block` finding stands against; a `warn` or `info` finding leaves `passed` where it stands. `stands_against` is the field that resolves the mapping: a `block` finding cited by a `REQ-nnn` or a `CON-nnn` stands against every story whose `## Acceptance Criteria` cover that id, and a finding cited by a `STORY-nnn` stands against that story alone. The agent writes the list rather than leaving the CLI to derive it, because the two available readings give different ratios against this phase's `verifier_pass` at `min_ratio = 1.0`.

## Shared lineage

| Agent | derives_from | Other skills deriving from the same file |
|---|---|---|
| `story-decomposer` | `C:\Users\bryan\.claude\agents\story-requirements-analyst.md` | none |
| `story-file-set-planner` | `C:\Users\bryan\.claude\agents\api-designer.md` — replaced, not adapted | none |
| `story-invest-auditor` | `C:\Users\bryan\.claude\agents\requirements-analyst.md` — adapted | `exploring-ideas` · `flow-drafter`; `discovering-requirements` · `requirement-drafter`, `epic-grouper` |
| `sprint-sequencer` | `C:\Users\bryan\.claude\agents\sprint-planner.md` — adapted | none |
| `spec-gap-triager` | `new` | none |
