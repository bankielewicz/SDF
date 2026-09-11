# Subagents · Releasing Software

## Invocation order

| # | Workflow step | Agent | Batch | Registered verifier |
|---|---|---|---|---|
| 1 | 6 | `deferral-auditor` | alone | yes · `verifiers.deferrals` |
| 2 | 8 | `deploy-manifest-writer` | alone; skipped when `platform.target` is `none` | no |
| 3 | 10 | `api-doc-writer` | batch 1 of 2 | no |
| 4 | 10 | `guide-writer` | batch 1 of 2 | no |

A row whose Batch cell names a batch is invoked in the same message as every other row naming that batch. Rows 3 and 4 name batch 1: they read disjoint inputs — one takes the symbol lines and the stack tables, the other takes the release set, the accepted ADRs, and the context sections — and write to different directories under `[release].docs_root`.

Rows 1 and 2 stand alone. Row 2 consumes the platform value row 1's finding set is read against, and row 2's `secrets_referenced` is what the gate's `deploy_manifest` check reads back from the files. A row with a condition carries it in the Batch cell after a semicolon.

A subagent whose JSON does not parse against its schema is invoked once more with the parse error appended. A second failure leaves the gate metric absent, which `devforgeai gate check --phase release` reports as exit 1.

## Contracts

| Agent | Model | Tools | Input fields | Output schema |
|---|---|---|---|---|
| `deferral-auditor` | opus | `Read`, `Glob`, `Grep` | `version`, `platform_target`, `stories`, `approved_dependencies` | `agents/deferral-auditor.md` `## Output` |
| `deploy-manifest-writer` | sonnet | `Read`, `Write`, `Glob` | `target`, `version`, `image_name`, `deploy_root`, `build_command`, `package_command`, `service_port`, `stacks`, `constraints`, `templates`, `previous_version`, `app_name` | `agents/deploy-manifest-writer.md` `## Output` |
| `api-doc-writer` | sonnet | `Read`, `Write`, `Glob`, `Grep` | `symbols`, `stacks`, `docs_root`, `version`, `templates` | `agents/api-doc-writer.md` `## Output` |
| `guide-writer` | opus | `Read`, `Write`, `Glob` | `version`, `docs_root`, `app_name`, `stories`, `adrs`, `context_sections`, `templates` | `agents/guide-writer.md` `## Output` |

## Registered verifiers

| Agent | phase | report_field | unit | required |
|---|---|---|---|---|
| `deferral-auditor` | release | `verifiers.deferrals` | deferrals | true |

The SubagentStop hook runs `devforgeai report ingest <name> -` for each row above.
A name absent from `config.toml` `[[verifier]]` is a no-op with exit 0 and `DFA-W411`.

`deferral-auditor` prints exactly one JSON object on stdout: the `devforgeai/verifier/1` envelope, whose keys are `schema` (the constant `devforgeai/verifier/1`), `subagent`, `passed`, `total`, `unit`, `findings[]` of `id`, `severity` from the closed enum `block | warn | info`, `confidence` from `0.0` to `1.0`, `summary` and `evidence`, and `payload`, which holds every top-level field the agent adds of its own. This agent adds none, so `payload` is `{}`. The envelope's `id` key is omitted on purpose: `report ingest` falls back to `state.toml` `[active].release`, which the run set to the release version. Findings add `story`, `kind`, and `reason`, which `report ingest` copies through unread. Anything other than one object of that shape is `DFA-E410`, which writes the block at `status: unparsed`.

`total` counts every deferred `FIND-nnn` across the release set and `passed` is `total` minus the entries carrying a `block` finding, which is the same number as the `accepted` entries. Findings carry no `blocks_deployment` boolean: a gate decision belongs to the CLI, `kind: blocks_deployment` at `severity: block` already says what the boolean said, and the release gate is what turns a lowered ratio into a send-back.

The other three return run-local JSON the skill consumes, so SubagentStop ignores them. `deploy-manifest-writer`'s files are read back by the gate's `deploy_manifest` check and `api-doc-writer`'s pages by `docs_cover`, each recomputing from disk rather than from the agent's own count.

## Shared lineage

| Agent | derives_from | Other skills deriving from the same file |
|---|---|---|
| `deferral-auditor` | `C:\Users\bryan\.claude\agents\deferral-validator.md` — adapted | `validating-quality` · `deferral-validator` |
| `deploy-manifest-writer` | `C:\Users\bryan\.claude\agents\deployment-engineer.md` — adapted | none |
| `api-doc-writer` | `C:\Users\bryan\.claude\agents\documentation-writer.md` — adapted | `releasing-software` · `guide-writer` |
| `guide-writer` | `C:\Users\bryan\.claude\agents\documentation-writer.md` — adapted | `releasing-software` · `api-doc-writer` |

`deferral-auditor` and Verify's `deferral-validator` are two registrations of the same lineage at different phases with different questions: Verify decides whether a deferral is justified, Release decides whether it blocks deployment on `platform.target`. Neither invokes the other.
