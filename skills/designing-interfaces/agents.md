# Subagents · Designing Interfaces

## Invocation order

| # | Workflow step | Agent | Batch | Registered verifier |
|---|---|---|---|---|
| 1 | 3 | `mockup-designer` | alone; `--sketch` mode | no |
| 2 | 4 | `mockup-designer` | alone; `--sketch` mode, after step 3 | no |
| 3 | 7 | `brand-designer` | alone; `--brand` mode | no |
| 4 | 8 | `brand-designer` | alone; `--brand` mode, after step 7 | no |
| 5 | 11 | `requirement-coverage-auditor` | alone; `--spec` mode, before any `UI-nnn` is allocated | yes · `verifiers.requirement_coverage` |
| 6 | 13 | `ui-spec-writer` | batch 1; `--spec` mode, one invocation per screen | no |
| 7 | 15 | `ui-spec-writer` | alone; `--spec` remedy run, one invocation for the cited `UI-nnn` | no |

A row whose Batch cell names a batch is invoked in the same message as every other row naming that batch. Row 6 names batch 1: the screens are disjoint, each invocation writes its own `UI-nnn.md`, and none reads another's output. A row with a condition carries it in the Batch cell after a semicolon.

A subagent whose JSON does not parse against its schema is invoked once more with the parse error appended. A second failure leaves the block that step would have produced absent, and the run stops with one `Blocked` line naming the agent.

All four agents are owned by this skill and invoked by it alone. The agent files ship to `.claude/agents/`: `agents/mockup-designer.md`, `agents/brand-designer.md`, `agents/requirement-coverage-auditor.md`, `agents/ui-spec-writer.md`.

## Contracts

| Agent | Model | Tools | Input fields | Output schema |
|---|---|---|---|---|
| `mockup-designer` | sonnet | `Read`, `Write`, `Glob`, `Skill` | `idea_id`, `flows`, `out_dir`, `seed_data_path`, `constraints`, `brand`, `template_path` | `agents/mockup-designer.md` `## Output` |
| `brand-designer` | opus | `Read`, `Write`, `Skill` | `answers`, `candidate_name`, `candidate_palette`, `candidate_type_pair`, `persona_goals`, `mapping_table`, `template_paths`, `color_group` | `agents/brand-designer.md` `## Output` |
| `requirement-coverage-auditor` | sonnet | `Read`, `Grep`, `Glob` | `subject_id`, `screens`, `requirements`, `epics`, `consumes`, `acceptance_criteria`, `core_flows` | `agents/requirement-coverage-auditor.md` `## Output` |
| `ui-spec-writer` | opus | `Read`, `Write`, `Grep`, `Glob`, `Skill` | `screen`, `requirement_records`, `persona_goal`, `token_names`, `mockup_paths`, `figma_node_url`, `figma_context`, `template_path`, `current_file`, `cited_ids` | `agents/ui-spec-writer.md` `## Output` |

Three of the four hold `Write`. `mockup-designer` writes one HTML file per screen under `out_dir` and `brand-sketch.json` when the request carried a `brand` object; `brand-designer` writes `.devforgeai/brand/tokens.json` at step 7 and `.devforgeai/brand/logo.svg` at step 8; `ui-spec-writer` writes one `.devforgeai/ui-specs/UI-nnn.md`. The two that write under `.devforgeai/` pass through the `PreToolUse` `doc validate --producer-check` and `design lint` hooks exactly as the skill's own writes do. `requirement-coverage-auditor` writes nothing.

The coverage rules `requirement-coverage-auditor` applies: a screen is covered when at least one `REQ-nnn` in the story's `consumes` names it in `acceptance_signal` or in an `AC-nnn` row; a flow is covered when its `FLOW-nnn` equals some `requirements[].source`.

The skill produces `figma_context` itself, in the main conversation, by running `figma:figma-design-to-code` for the story's node and passing the result to `ui-spec-writer` as a string; it passes the empty string when the story carries no node URL, or the Figma plugin is absent, or the call returns an authentication error. The agent makes no Figma call of its own: a subagent's tool set is its `tools` list plus its `mcpServers` and nothing else reaches it, so an agent holding `Skill` could load the prerequisite skill and would still have no `mcp__plugin_figma_figma__*` tool to call.

`mockup-designer` and `brand-designer` invoke `frontend-design:frontend-design` and no other skill. `frontend-design`, `design`, and `figma:figma-design-to-code` are plugin or first-party skills whose presence `devforgeai init` does not establish in a target project, so each of the three agents holding `Skill` states in its own `## Workflow` what it does when the skill is absent. `mockup-designer` does not invoke `design`: that skill's output path publishes a canvas through the `Artifact` tool, which the agent does not hold and which would put a published page in the middle of a phase whose outputs are files under `.devforgeai/`.

## Registered verifiers

| Agent | phase | report_field | unit | required |
|---|---|---|---|---|
| `requirement-coverage-auditor` | design | `verifiers.requirement_coverage` | screens | false |

The SubagentStop hook runs `devforgeai report ingest <name> -` for each row above.
A name absent from `config.toml` `[[verifier]]` is a no-op with exit 0 and `DFA-W411`.

The other three return run-local JSON the skill consumes, so SubagentStop ignores them. `required` is `false` because the design phase has no gate, so no check of any kind names the agent; the block lands at `verifiers.requirement_coverage` of `.devforgeai/reports/UI-nnn-design.yaml`, where `passed`/`total` fills the handoff `Verified` line, and Reflect reads it afterwards.

`requirement-coverage-auditor` prints exactly one JSON object on stdout: the `devforgeai/verifier/1` envelope, whose keys are `schema` (the constant `devforgeai/verifier/1`), `subagent`, `id`, `passed`, `total`, `unit`, `findings[]` of `id`, `severity` from the closed enum `block | warn | info`, `confidence`, `summary` and `evidence`, and `payload`, which holds every top-level field the agent adds of its own — here `subject_id`, `screens`, `screens_without_req`, `flows_without_req`, and `covered`. `total` is the screen count and `passed` is `total` minus the screens carrying a `block` finding, which equals `payload.covered`. An uncovered screen is `block`; an uncovered flow is `warn` and leaves `passed` where it stands, because the unit this ratio measures is screens. Anything other than one object of that shape is `DFA-E410`, which writes the block at `status: unparsed`.

## Shared lineage

| Agent | derives_from | Other skills deriving from the same file |
|---|---|---|
| `mockup-designer` | `C:\Users\bryan\.claude\agents\frontend-developer.md` — adapted, keeping its semantic-element and breakpoint judgment and dropping the framework patterns, the package-manager tool, the test step, and the observation file | `implementing-stories` · `frontend-implementer`; `exploring-ideas` · `prototype-builder` |
| `brand-designer` | `new` | none |
| `requirement-coverage-auditor` | `new` | none |
| `ui-spec-writer` | `C:\Users\bryan\.claude\agents\ui-spec-formatter.md` — replaced, because the existing agent read a finished spec and produced a display template for a command's output, and the handoff is printed by `devforgeai handoff` | none |

Three existing agents reach no step of this skill. `frontend-developer.md` stays Build's lineage for the implementer and is not adapted a second time here: that agent writes production components, and this skill's outputs are `brand/tokens.json` and `ui-specs/UI-nnn.md` and no source file. `stakeholder-analyst.md` is not invoked: personas arrive already written, as `PERSONA-nnn` records with `name`, `description`, and `goal` in `requirements.yaml`, so Design reads them and Discover discovers them. `prototype-builder` is Explore's agent, invoked by its owner alone.
