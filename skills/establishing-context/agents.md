# Subagents · Establishing Context

## Invocation order

| # | Workflow step | Agent | Batch | Registered verifier |
|---|---|---|---|---|
| 1 | B5 | `source-tree-mapper` | alone; the brownfield branch only, after the six drafts are read | no |
| 2 | 11 | `architecture-reviewer` | batch 1 of 2 | yes · `verifiers.architecture_reviewer` |
| 3 | 11 | `alignment-auditor` | batch 1 of 2 | yes · `verifiers.alignment_auditor` |
| 4 | R3 | `architecture-reviewer` | alone; the remedy path, scoped to one `CON-nnn` | yes · `verifiers.architecture_reviewer` |
| 5 | R5 | `architecture-reviewer` | batch 2 of 2; the remedy path, after a replacement, over the whole set | yes · `verifiers.architecture_reviewer` |
| 6 | R5 | `alignment-auditor` | batch 2 of 2; the remedy path, after a replacement, over the whole set | yes · `verifiers.alignment_auditor` |

A row whose Batch cell names a batch is invoked in the same message as every other row naming that batch. Rows 2 and 3 name batch 1 and rows 5 and 6 name batch 2: neither agent reads the other's output — one holds the requirement set against the constraint set, the other holds the files against each other. `source-tree-mapper` runs alone, before either, because the files they read do not exist until B7 completes. A row with a condition carries it in the Batch cell after a semicolon.

A subagent whose JSON does not parse against its schema is invoked once more with the parse error appended. A second failure leaves the verifier's block absent from the report, which `devforgeai gate check --phase constitute` reports as exit 1 on the missing metric; for `source-tree-mapper` it continues B7 with the draft values and turns every unfilled field into an `open_questions` entry.

Each of the three is read-only and returns one JSON object, writing no phase document. The skill writes the six context files and every ADR, so the `PreToolUse` producer check sees one producer for everything under `.devforgeai/context/` and `.devforgeai/adr/`.

## Contracts

| Agent | Model | Tools | Input fields | Output schema |
|---|---|---|---|---|
| `architecture-reviewer` | opus | `Read`, `Grep`, `Glob` | `id`, `context_paths`, `adr_dir`, `requirements`, `epics`, `scoped_constraint` | `agents/architecture-reviewer.md` `## Output` |
| `alignment-auditor` | sonnet | `Read`, `Grep`, `Glob` | `id`, `context_paths`, `adr_dir`, `constraint_index`, `anti_pattern_index` | `agents/alignment-auditor.md` `## Output` |
| `source-tree-mapper` | sonnet | `Read`, `Grep`, `Glob` | `source_root`, `layer_names`, `manifest_paths` | `agents/source-tree-mapper.md` `## Output` |

`architecture-reviewer` returns an empty `findings` array when nothing holds; on the remedy path that empty array is the answer that upholds the constraint and the skill routes the send-back to Plan on it. None of the five `alignment-auditor` finding kinds reduces to string equality, which is what separates them from `context audit` checks CA-1 through CA-8.

## Registered verifiers

| Agent | phase | report_field | unit | required |
|---|---|---|---|---|
| `architecture-reviewer` | constitute | `verifiers.architecture_reviewer` | requirements | false |
| `alignment-auditor` | constitute | `verifiers.alignment_auditor` | checks | false |

The SubagentStop hook runs `devforgeai report ingest <name> -` for each row above.
A name absent from `config.toml` `[[verifier]]` is a no-op with exit 0 and `DFA-W411`.

`source-tree-mapper` returns run-local JSON the skill consumes, so SubagentStop ignores it. Both blocks land in `.devforgeai/reports/IDEA-nnn-constitute.yaml`. The gate reads `verifiers.architecture_reviewer.payload.blocking_findings` and `verifiers.architecture_reviewer.payload.send_back_requirements.length` — the second carrying `on_fail = "send_back"`, which routes to Discover — and `verifiers.alignment_auditor.payload.blocking_findings`. All three are `report_metric` checks and no `verifier_pass` check names either agent, which is why both rows carry `required = false`.

The `payload.` segment is part of all three paths. Each of the two verifiers prints exactly one JSON object on stdout: the `devforgeai/verifier/1` envelope, whose keys are `schema` (the constant `devforgeai/verifier/1`), `subagent`, `id`, `passed`, `total`, `unit`, `findings[]` of `id`, `severity` from the closed enum `block | warn | info`, `summary` and `evidence`, and `payload`, which holds every top-level field the agent adds of its own. `architecture-reviewer` fills `payload` with `requirements_reviewed`, `send_back_requirements` and `blocking_findings`; `alignment-auditor` with `checks_run` and `blocking_findings`. Each finding adds `confidence`, a float from `0.0` to `1.0`, and the phase fields the agent's own `## Output` names, which `report ingest` copies through unread. Anything other than one object of that shape is `DFA-E410`, which writes the block at `status: unparsed` and fails every gate check reading it with `DFA-E316`.

`passed` is `total` minus the count of units carrying a `block` finding and nothing else moves it: a `warn` or `info` finding lands in the report and leaves `passed` where it stands. For `architecture-reviewer` the unit is a requirement and `passed` counts distinct REQ ids; for `alignment-auditor` it is a check pair, so two `block` findings over one pair take `passed` down by one.

## Shared lineage

| Agent | derives_from | Other skills deriving from the same file |
|---|---|---|
| `architecture-reviewer` | `C:\Users\bryan\.claude\agents\architect-reviewer.md` — adapted, with the Markdown severity report replaced by JSON, the SOLID and pattern catalogs dropped as generic, and the scope narrowed to REQ-against-architecture feasibility and CON contradiction | `validating-quality` · `adr-conformance-reviewer` |
| `alignment-auditor` | `C:\Users\bryan\.claude\agents\alignment-auditor.md` — adapted, name kept, with the exact-text matching moved to `context audit` CA-1 through CA-8 and the semantic comparison kept | none |
| `source-tree-mapper` | `C:\Users\bryan\.claude\agents\code-analyzer.md` — adapted, with the documentation-coverage and public-API extraction dropped and the layer, root, entry-point and internal-edge discovery retained | none |

Four existing agents reach no step of this phase. `tech-stack-detector.md` is retired: detection is `devforgeai stack detect`, which writes `config.toml`, and validation of detected values against `tech-stack.md` is `context audit` check CA-5. `api-designer.md` is not invoked here: an API style choice is an ADR this skill writes, and endpoint contracts belong to Plan, which owns the agent. `context-validator.md` is not invoked here: it validates source code against the six files, which is Build's hook path and Verify's `anti-pattern-scanner`. `diagnostic-analyst.md` is not invoked here: it investigates a failure that already happened, which is Build and Verify territory.
