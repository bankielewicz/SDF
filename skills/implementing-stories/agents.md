# Subagents · Implementing Stories

## Invocation order

| # | Workflow step | Agent | Batch | Registered verifier |
|---|---|---|---|---|
| 1 | 7.1 | `ac-test-writer` | serial, once per criterion | yes · `verifiers.ac_testable` |
| 2 | 7.4 | `backend-implementer` | serial, once per criterion; when `## Layer` is not `interface`, or `## Interface` holds no row | no |
| 3 | 7.4 | `frontend-implementer` | serial, once per criterion; when `## Layer` is `interface` and `## Interface` holds a row | no |
| 4 | 7.5 | `backend-implementer` or `frontend-implementer` | serial; one retry per criterion, on a non-zero test exit | no |
| 5 | 7.8 | `refactor-surgeon` | serial, zero or one per criterion; when `build-lint` or `build-complexity` reads `fail` | no |
| 6 | 8 | `integration-test-writer` | alone, zero or one per run; when `## Layer` is `interface` or `## Interface` holds a row | no |
| 7 | 9 | `context-validator` | alone, once per run, after step 8 | yes · `verifiers.context` |
| 8 | 10 | `refactor-surgeon` | alone, zero or one per run; on a `DFA-E270` match | no |
| 9 | 11 | `story-ac-verifier` | alone, once per run, after steps 8, 9, and 10 | yes · `verifiers.story_ac` |

A row whose Batch cell names a batch is invoked in the same message as every other row naming that batch. No row here names one: every step of the cycle consumes the step before it, step 8 reads the source paths of every cycle, step 9 reads the diff step 8 finished, and step 11 reads the diff and the test output steps 8 through 10 left behind. A row with a condition carries it in the Batch cell after a semicolon.

A subagent whose JSON does not parse against its schema is invoked once more with the parse error appended. A second failure leaves a registered verifier's block at `status: unparsed`, which `devforgeai gate check --phase build` reports as a failed `verifier_pass` check, and stops the run for the other four.

## Contracts

| Agent | Model | Tools | Input fields | Output schema |
|---|---|---|---|---|
| `ac-test-writer` | opus | `Read`, `Write`, `Edit`, `Grep`, `Glob` | `id`, `ac`, `ac_line`, `other_acs`, `test_files`, `testing_standards`, `naming_conventions`, `current_test_content`, `finding_summary`, `finding_evidence` | `agents/ac-test-writer.md` `## Output` |
| `backend-implementer` | opus | `Read`, `Write`, `Edit`, `Grep`, `Glob` | `ac`, `ac_line`, `failing_tests`, `failing_output`, `source_files`, `layer`, `constraints`, `anti_patterns`, `approved_dependencies`, `forbidden_dependencies`, `error_handling`, `logging`, `formatting`, `earlier_source_paths` | `agents/backend-implementer.md` `## Output` |
| `frontend-implementer` | sonnet | `Read`, `Write`, `Edit`, `Grep`, `Glob` | `ac`, `ac_line`, `failing_tests`, `failing_output`, `source_files`, `ui`, `anatomy`, `states`, `breakpoints`, `interaction`, `accessibility`, `tokens_declared`, `token_leaves`, `formatting`, `naming` | `agents/frontend-implementer.md` `## Output` |
| `refactor-surgeon` | sonnet | `Read`, `Write`, `Edit`, `Grep`, `Glob` | `trigger`, `reason`, `matches`, `paths`, `complexity_max`, `duplication_max_percent`, `formatting`, `naming` | `agents/refactor-surgeon.md` `## Output` |
| `integration-test-writer` | sonnet | `Read`, `Write`, `Edit`, `Grep`, `Glob` | `acs`, `test_files`, `source_paths`, `source_content`, `ui_interaction`, `ui_states`, `layer_dependency_rules`, `testing_standards` | `agents/integration-test-writer.md` `## Output` |
| `story-ac-verifier` | opus | `Read`, `Grep`, `Glob` | `story_path`, `diff`, `test_output` | `agents/story-ac-verifier.md` `## Output` |
| `context-validator` | opus | `Read`, `Grep`, `Glob` | `id`, `diff`, `context_paths`, `constraints`, `layer` | `agents/context-validator.md` `## Output` |

`story-ac-verifier` takes its three fields and nothing else: no part of the conversation, no subagent output from steps 7.1 through 10, and no summary of them. Its freshness is a property of what the prompt carries. The skill wraps the three in XML tags — `<story_path>`, `<diff>`, `<test_output>` — so that the agent can tell input from instruction: a diff hunk or a test log can hold a comment, a fixture string, or a commit message shaped like a directive, and the tags are what say that everything inside them is data.

Three of the seven hold `Write` and `Edit` because their output is a file — a test, an implementation, a rewrite. Each writes inside the story's `## Files` set, which the `PreToolUse` `story files --check` hook enforces against a subagent's writes as it does against the model's. None of the seven writes under `.devforgeai/`. None holds `Bash`: the test command is the model's step, and every git call is a CLI call.

## Registered verifiers

| Agent | phase | report_field | unit | required |
|---|---|---|---|---|
| `ac-test-writer` | build | `verifiers.ac_testable` | AC | true |
| `story-ac-verifier` | build | `verifiers.story_ac` | ACs | true |
| `context-validator` | build | `verifiers.context` | files | true |

The SubagentStop hook runs `devforgeai report ingest <name> -` for each row above.
A name absent from `config.toml` `[[verifier]]` is a no-op with exit 0 and `DFA-W411`.

Each of the three prints exactly one JSON object on stdout: the `devforgeai/verifier/1` envelope, whose keys are `schema` (the constant `devforgeai/verifier/1`), `subagent`, `id`, `passed`, `total`, `unit`, `findings[]` of `id`, `severity` from the closed enum `block | warn | info`, `confidence` from `0.0` to `1.0`, `summary` and `evidence`, and `payload`, which holds every top-level field the agent adds of its own. `ac-test-writer` fills `payload` with `testable`, `ac`, `test_paths`, `assertion`, and `conflicts_with`; `story-ac-verifier` with `checks`; `context-validator` adds nothing, so its `payload` is `{}`. Findings add `reason` for the first two and `kind`, `path`, `line` for the third, which `report ingest` copies through unread. Anything other than one object of that shape is `DFA-E410`, which writes the block at `status: unparsed` and fails the `verifier_pass` check naming it with `DFA-E316`.

`passed` is `total` minus the count of units carrying a `block` finding, and nothing else moves it: a `warn` or `info` finding lands in the report and leaves `passed` where it stands. `story-ac-verifier`'s per-criterion `verdict` reads `met` or `unmet` rather than `PASS` or `FAIL`, because the framework reserves those two words for the handoff `Gate` line the CLI prints and a report showing `FAIL` beside a criterion under a `Gate PASS` line for the same run reads as a contradiction it is not.

The other four return run-local JSON the skill consumes, so SubagentStop ignores them. Each of the three rows above is its own `verifier_pass` check — `build-testable`, `build-acs`, `build-context` — rather than one ratio over all three, so a passing verifier does not compensate for a failing one. `report ingest` replaces a subagent's previous block and appends its findings de-duplicated by `id`, so after a run the `ac_testable` block holds the last invocation's `passed`/`total` while `findings[]` holds every criterion any invocation flagged.

## Shared lineage

| Agent | derives_from | Other skills deriving from the same file |
|---|---|---|
| `ac-test-writer` | `C:\Users\bryan\.claude\agents\test-automator.md` — adapted | none |
| `backend-implementer` | `C:\Users\bryan\.claude\agents\backend-architect.md` — adapted | none |
| `frontend-implementer` | `C:\Users\bryan\.claude\agents\frontend-developer.md` — adapted | `designing-interfaces` · `mockup-designer`; `exploring-ideas` · `prototype-builder` |
| `refactor-surgeon` | `C:\Users\bryan\.claude\agents\refactoring-specialist.md` — adapted | none |
| `integration-test-writer` | `C:\Users\bryan\.claude\agents\integration-tester.md` — adapted | none |
| `story-ac-verifier` | `C:\Users\bryan\.claude\agents\ac-compliance-verifier.md` — adapted | `validating-quality` · `ac-compliance-verifier` |
| `context-validator` | `C:\Users\bryan\.claude\agents\context-validator.md` — adapted, name kept | `validating-quality` · `constraint-auditor` |

`story-ac-verifier` carries a new name rather than a second registration of `ac-compliance-verifier`, because a `[[verifier]]` table binds one name to one phase and `SubagentStop` calls `report ingest <subagent> -` with no `--phase`. `ac-compliance-verifier` stays Verify's.
