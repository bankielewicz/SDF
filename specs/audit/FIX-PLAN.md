---
schema: devforgeai-audit/1
doc: fix-plan
status: authoritative
produced_by: orchestrator
consumes: [AUDIT-1-cli, AUDIT-2-hooks, AUDIT-3-skills, AUDIT-4-agents, AUDIT-5-evals, AUDIT-6-framework]
---

# Fix plan

Six audit reports under `specs/audit/` carry the findings and, under each report's `## Fix specifications`, the exact change per blocker and high finding. This plan assigns every file to exactly one fixer, states the decisions that cut across reports so no fixer re-derives them, and orders the work. A fixer reads `specs/ANTHROPIC-GUIDANCE.md` and `specs/00-conventions.md` first, then its assigned audit reports in full, then the files it owns.

## 1. Decisions that bind every fixer

1. **Handoff delivery.** The Stop hook emits one JSON object on stdout. PASS: `{"systemMessage": <12-line block>}`, exit 0. FAIL with `stop_hook_active` false: `{"decision":"block","reason":<checks and fix>,"systemMessage":<block>}`, exit 2. `stop_hook_active` true (amended 2026-09-11, orchestrator, on advisor review): the hook re-runs the gate, because the continuation turn may have repaired the failing check; a FAIL blocks again with the same object and exit 2, under a budget of three blocks per `session_id` (not per subject: a changed phase or id does not reset it), after which the hook exits 0 with the FAIL handoff in `systemMessage` alone. This amends the sentence it replaces, which said no gate run; `specs/ANTHROPIC-GUIDANCE.md` §2's "exit 0 when set" is satisfied by the session-wide budget, which is below the harness's own eight-block cap. SEND BACK: `{"systemMessage": <block>}`, exit 0, no block. Cross-cutting skills (Design, Reflect) record `[last_cross]` in state.toml and the Stop joins two blocks. SessionStart prints the last handoff as plain stdout. The model prints no sentence before and no block after; every "one sentence before the block" instruction is deleted. Full text: AUDIT-6 FWK-001.
2. **Fail closed under the real contract.** Exit 2 blocks only on PreToolUse, Stop, SubagentStop, UserPromptSubmit, UserPromptExpansion. Everywhere else the channel is JSON: `systemMessage` for the user, `hookSpecificOutput.additionalContext` for Claude. Exit 4 and exit 1 never block; they are recorded and converted to exit 2 on blocking events. Trust failure: PreToolUse `trust-check` arm on `Write|Edit|NotebookEdit|Bash|PowerShell|Agent` returning `permissionDecision: deny`; Stop blocks with the pin command; UserPromptExpansion on the nine skill names blocks. Full text: AUDIT-6 FWK-010.
3. **Every gate check kind fails closed.** A check whose document, field, path, pattern, `verifiers`, `stories`, or state field is absent, null, unparsable, or of the wrong type reports `fail` with a named error code, never `pass` and never `skip`. `skip` is produced only by an explicit `skip_when` that evaluates true against a loadable document, or by `degraded`/`no_run` on the four command-running kinds. Compiled minimums are enforced at load. Full text: AUDIT-1 CLI-100 to CLI-153 and CLI-200 to CLI-210.
4. **One JSON object per verifier.** A registered verifier prints exactly one object: the `devforgeai/verifier/1` envelope with the agent's own fields merged in under a `payload` key. Never two objects. `warn` findings never lower `passed`. Review agents report every finding, including uncertain and low severity, with no cap and no self-filtering; the CLI and Verify filter. Full text: AUDIT-4 AGT-014 to AGT-021 and AGT-032, AGT-033.
5. **No subagent asks the user.** `AskUserQuestion` is removed from every agent file and from the catalog's permitted list. `idea-interrogator` returns candidate holders; `exploring-ideas` SKILL.md step 2 asks the user with the spec's question text and passes the answers on. Full text: AUDIT-4 AGT-001, AGT-002.
6. **Commands collapse into skills.** `commands/` is deleted. Each SKILL.md gains `name: <slash name>`, `argument-hint`, `allowed-tools` including `Bash(devforgeai:*)` and `PowerShell(devforgeai:*)`, and the `!` preamble lines at the top, `gate require` first. Eight skills take `disable-model-invocation: true` (the seven phases plus improving-framework); designing-interfaces stays model-invocable. Preambles that allocate an id run only when the run needs one (AUDIT-3 SKL-027): allocation moves into the workflow step, not the preamble. Full text: AUDIT-6 FWK-040, AUDIT-3 SKL-024.
7. **Shell matchers are `Bash|PowerShell`.** Everywhere: settings template, dispatcher, specs.
8. **Agent descriptions are delegation triggers.** One or two sentences: what it does and when the owning skill delegates to it. Purpose paragraphs move to the body. Full text: AUDIT-4 AGT-009, AGT-010.
9. **MoSCoW restored.** Priority enum returns to `must | should | could | wont`. The ceremony rule is scoped to instruction prose only (AUDIT-6 FWK-080): fenced blocks, YAML scalars, backticked spans, enum cells, paths, and headings are exempt; documents the framework produces about a target project are exempt. One script, `scripts/ceremony_scan.py`, implements the rule; specs and the build brief reference it.
10. **`init` writes the CLAUDE.md block** between markers, idempotently, 33 lines, text verbatim from AUDIT-6 FWK-050; copies `skills` and `agents` only, excluding every `evals/` subtree; prints the pin command as `Next` when trust verify fails.
11. **The eval runner pre-seeds AskUserQuestion answers** through a PreToolUse hook written into the workspace settings, per AUDIT-5 EVL findings; it passes `--allowedTools "Bash(devforgeai *)"` and captures `stream-json` so graders can see tool calls.
12. **Spec truth follows code truth.** Fixers of code and skills implement the audit fix specs; the specs fixer updates every spec, the conventions, the brief, the README, and the questions file to state what now exists. No fixer edits a file another fixer owns; a needed change outside one's files is written as a one-line note at the end of the fixer's report, and the orchestrator routes it.

## 2. Ownership

| Fixer | Reads | Owns (may edit) | Does not touch |
|---|---|---|---|
| F1 gate engine | AUDIT-1 (§2, §5 gate require rows, §6 gate tests, §7), decision 3 | `cli/src/gate.rs`, `cli/src/gates.rs`, `cli/src/report.rs`, `cli/tests/gate_check.rs`, `cli/tests/gate_require.rs`, `cli/tests/default_gates.rs`; error rows appended at the end of `cli/src/errors_table.rs` under a `// gate engine` comment | `cli/src/hooks/*`, `trust.rs`, `docops.rs`, `doc/*`, `cli.rs`, `cmd/init.rs` |
| F2 dispatcher, trust, docs, envelope | AUDIT-2 (all), AUDIT-1 (§1, §3, §4, §5 subcommand rows, CLI-205 to CLI-210), AUDIT-6 FWK-001, FWK-010, FWK-050, FWK-072, decisions 1, 2, 7, 10 | `cli/src/hooks/*`, `cli/src/trust.rs`, `cli/src/docops.rs`, `cli/src/doc/*`, `cli/src/cli.rs`, `cli/src/main.rs`, `cli/src/lib.rs`, `cli/src/cmd/*`, `cli/src/state.rs`, `cli/src/handoff.rs`, `cli/src/aggregate.rs`, `hooks/settings.hooks.json`, `cli/REVISION`, `cli/DIGEST`, all `cli/tests/*` except the three F1 owns; error rows appended under `// hooks and docs` | `cli/src/gate.rs`, `cli/src/gates.rs` (request a change through the report) |
| F3 agents | AUDIT-4 (all), decisions 4, 5, 8 | `agents/*.md`, every `skills/*/agents.md`, `specs/11-subagent-catalog.md` `## Templates` and Decision 1 registry | any `SKILL.md`, references, templates |
| F4 skills | AUDIT-3 (all), AUDIT-6 FWK-040, FWK-006, FWK-070, FWK-071, decisions 1 (the deleted sentence), 5 (the SKILL.md side), 6, 9 (Discover template) | `skills/*/SKILL.md`, `skills/*/references/*`, `skills/*/templates/*`, deletion of `commands/` | `agents.md` files, `agents/*.md`, `evals/*`, specs |
| F5 evals | AUDIT-5 (all), AUDIT-6 FWK-080 grader part, decisions 9, 11 | `evals/runner/*`, every `skills/*/evals/*` | skill bodies, templates, CLI |
| F6 specs and docs | AUDIT-6 (all), the `## Fix specifications` of every other report for the spec edits they name, decisions 1 to 12 | `specs/00-conventions.md`, `specs/01-cli.md` through `specs/10-reflect.md`, `specs/BUILD-BRIEF.md`, `specs/questions.md`, `README.md`, `scripts/ceremony_scan.py` (new) | code, skills, agents, evals |

## 3. Sequence

Wave A runs F1, F2, F3, F4 in parallel; they share no files. Wave B runs F5 and F6 when slots free; F5 needs F4's collapsed skill names for the runner's copy logic (read `skills/*/SKILL.md` frontmatter `name:` at that point), and F6 documents the end state of everything.

Within F1 and F2, every change is test-first: the audit's `Test first` list names the tests. Each fixer runs `cargo test --no-fail-fast`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt` before reporting, and F2 rebuilds the release binary and regenerates `cli/REVISION` and `cli/DIGEST` with the corrected walk (CLI-030: the walk excludes `cli/REVISION` as well as `cli/DIGEST`, since a file cannot contain its own digest).

## 4. Report format for every fixer

Reply with: findings addressed (IDs) and findings deliberately left (IDs, one-line reason each); files changed with line counts; test summary lines; the verification commands run and their output; and a `## Routed changes` list of one-line notes for files outside your ownership that need a change, naming the owner. Nothing else.
