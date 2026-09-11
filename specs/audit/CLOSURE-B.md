---
schema: devforgeai-audit/1
doc: closure
scope: AUDIT-3-skills, AUDIT-4-agents
produced_by: closure-verifier-b
---

# Closure B — skills and agents

Scope: every **high** and **medium** finding of `AUDIT-3-skills.md` (it carries no blocker),
and every **blocker** and **high** finding of `AUDIT-4-agents.md`. Mechanical checks 2, 3 and 4
of the verifier brief; another verifier covers 1, 5, 6, 7, 8, 9.

This repository is not a git checkout, so "regression" below means **a file that contradicts a
sibling file, a spec, or the code that reads it** — not "differs from a previous revision". No
claim below rests on a fixer's report; each cites a path and a line, a command and its output, or
both.

## AUDIT-3 — skills

| ID | severity | status | evidence |
|---|---|---|---|
| SKL-001 | high | closed | `skills/improving-framework/templates/reflect-report.yaml` is 35 lines (`wc -l`), ending at the closing `---`; the illustrative second block is gone. The shapes landed where the fix spec put them: `skills/improving-framework/references/observations.md` and `references/recommendations.md` both carry `observations[]`/`recommendations[]`/`technical_debt.groups` shapes. `specs/10-reflect.md:522-572` still introduces its second block as "the shapes to copy", which is the spec text the audit said was correct |
| SKL-002 | medium | **open** | Unresolved and empirically reachable. `reflect-report.yaml` is the only one of the seven YAML templates that carries `---` fences (the other six — `requirements.yaml`, `decision.yaml`, `build-note.yaml`, `sprint.yaml`, `release.yaml`, `qa-report.yaml` — open on `schema:`). `specs/10-reflect.md:524` and `:559` still carry them. Reachable path measured, not argued: `cli/target/release/devforgeai.exe doc validate .devforgeai/reports/reflect-2026-09-11.yaml` on a byte copy of the shipped template exits 1 with `DFA-E202 … frontmatter is not a YAML mapping: deserializing from YAML containing more than one document is not supported`. `cli/src/doc/frontmatter.rs:126-139` strips a *leading* `---` only; the trailing one opens a second document that `mapping()` at `:197` rejects. Deleting the last line takes the same file past E202 (next diagnostic is the template's placeholder id, `DFA-E209`). So `improving-framework` step 9's "write from `templates/reflect-report.yaml`" produces a reflect report that fails its own `doc validate` |
| SKL-003 | medium | closed | `skills/exploring-ideas/SKILL.md:89-104` now carries the Park follow-up as a literal `AskUserQuestion` block with `"header": "Revisit"` and the three `<d+30>`/`<d+90>`/`<d+180>` options with their descriptions, plus the re-ask rule |
| SKL-004 | medium | closed | `skills/planning-work/SKILL.md:50`, `:54`, `:58`, `:62` — steps 5, 7, 9 and 11 each now end with the parse-failure path in the spec's words ("re-invokes the subagent once with the parse error appended; a second failure leaves the gate metric absent, which `gate check` reports as exit 1"). Step 7 keeps its distinct `overlaps`/`unplaceable` re-invocation alongside |
| SKL-008 | medium | closed | One count in both places: `skills/releasing-software/SKILL.md:61` step 10 passes "the four context H2 sections `## Languages` … `## Layers` … `## Approved dependencies` and `## License policy`", and `:84` Subagents table reads "the four context H2 sections of step 10". `## Runtimes` and `## Constraint index` are gone |
| SKL-018 | medium | superseded (decision 6 / SKL-017) | Eight of nine still open "Phase N of DevForgeAI", but all eight now carry `disable-model-invocation: true`, so the description leaves the request context — the condition the audit itself named ("Under `disable-model-invocation: true` this stops mattering"). The one skill that stays model-invocable, `designing-interfaces`, now opens with the action: `SKILL.md:3` "Draws wireframe screens, builds the brand token set, and writes UI specifications." |
| SKL-017 | high | closed | `disable-model-invocation: true` present in the frontmatter of all eight (`discovering-requirements`, `establishing-context`, `exploring-ideas`, `implementing-stories`, `improving-framework`, `planning-work`, `releasing-software`, `validating-quality`) and absent from `designing-interfaces`, which `exploring-ideas` step 6 and `planning-work` steps 6/R3 invoke through the Skill tool. Verified by parsing all nine frontmatter blocks |
| SKL-019 | medium | closed | The grant is now on the skill, not on a vanished command file. All nine `SKILL.md` carry `allowed-tools:` including both `Bash(devforgeai:*)` and `PowerShell(devforgeai:*)` — mechanical check 4 below |
| SKL-024 | high | closed | Collapse complete on every one of the eight breakage points. No `commands/` directory anywhere (`find . -type d -name commands` → empty). All nine `name:` values are the slash names (`explore discover constitute plan build verify release design reflect`). `cli/src/cmd/init.rs:15` `const COPIED: &[&str] = &["skills", "agents"];` with the reason in the doc comment at `:11-14`; `:179` prints `"Copied     {} skills, {} agents"`. `evals/runner/run_jsonl.py` has no `SKILL_COMMANDS` map (`:472` records the removal). `cli/tests/init.rs:733`, `:736`, `:764` assert `.claude/skills/exploring-ideas` and `.claude/skills/planning-work` do **not** exist, which is the short-name install the override asks for |
| SKL-025 | medium | closed | One name for one target on both paths: `skills/planning-work/SKILL.md:52` "invoke the `design` skill with `STORY-nnn --spec`", `:116` and `:126` "invokes the `design` skill … with `UI-nnn --remedy AC-nnn,...`"; `skills/exploring-ideas/SKILL.md:65` "invoke the `design` skill in sketch mode". `skills/designing-interfaces/SKILL.md:2` is `name: design`, which is what the Skill tool resolves, and the directory name is unchanged so `produced_by` values stay valid |
| SKL-028 | medium | closed | The unreachable recovery is gone *and* the stderr the user actually sees is actionable. `skills/improving-framework/SKILL.md:29` states the abort as the behaviour — "`$ARGUMENTS` holding neither a recognised id prefix nor `--since` **stops the run in the preamble**: `devforgeai report aggregate` exits 3 on `DFA-E430` and its stderr line names the four accepted prefixes and the `--since` form" — and the claim checks out against the code: `cli/src/errors_table.rs:915-919` gives `DFA-E430` the message `"pass one id (IDEA-nnn, EPIC-nnn, STORY-nnn, vX.Y.Z) or --since <YYYY-MM-DD>"`. The skill no longer documents a step-1 failure path the abort semantics prevent from running, and the one line the user gets names the repair |
| SKL-029 | medium | closed | The navigation the finding said was missing now rides on the diagnostic itself. `cli/src/cmd/gate.rs:67-86` `repair_hint()` rewrites every `DFA-E321` to `"<message>; to repair, <command>"`, per phase: `build` → "run /plan \<EPIC-nnn\>, then /build again", `verify` → "run /build {id}, then /verify {id}", and so on for discover/constitute/plan/release. `:93-96` records why (the preamble aborts, so "this diagnostic is all the user sees"). It reaches all four `gate require` preambles |
| SKL-030 | medium | closed | `skills/releasing-software/SKILL.md:28` states both halves: "the call exits 0 at a count of zero, which step 4 handles. Exit 1 on `DFA-E231` means `.devforgeai/stories/` is absent **and the body does not load**." The skill no longer promises a `Blocked` line on a path that cannot print one, and the aborting diagnostic carries its own repair: `cli/src/cmd/story.rs:443-446` emits `DFA-E231` as `".devforgeai/stories/ not found; run 'devforgeai init'"`. One CLI-side note for the AUDIT-1 verifier, not a skill defect: `cli/src/errors_table.rs:397-401` documents `DFA-E231` under `subcommand: "story validate"` with the REQ-unresolved condition only, so `story list`'s use of the same code is undocumented in the table that mechanical check 9 tests for bijection |
| SKL-035 | medium | closed | All three `AskUserQuestion` call sites carry a literal block. `skills/exploring-ideas/SKILL.md:42` (Holders), `:74` (decision) and `:92` (Revisit) each open `AskUserQuestion(questions=[{`; Design's five brand questions are fixed verbatim in `skills/designing-interfaces/templates/brand-questions.md` (2 literal calls), referenced by `SKILL.md:78` and `references/brand.md:7`. Note: no SKILL.md uses `<example>` tags — the agents do (see AGT-038 area) — but the gap the finding named was the three call sites, and it is shut |
| SKL-042 | medium | closed | `skills/releasing-software/SKILL.md:48` step 7 now carries the failure path the finding said was absent: "The PreToolUse producer check covers every path under `.devforgeai/`, `config.toml` included; a refusal leaves `platform.target` at the answered value for this run alone and adds one `open_questions` line naming the key that was not written." |
| SKL-053 | medium | **open** | Neither side moved. `skills/exploring-ideas/templates/decision.yaml:37-40` still advertises `path: .devforgeai/explore/seed-data.json` / `becomes: test fixtures and example rows` / `consumer: implementing-stories`, and a grep for `seed-data` or `seed_data` across `skills/implementing-stories/` (SKILL.md, all references, agents.md), `specs/06-build.md`, and `agents/ac-test-writer.md` returns **zero hits**. The two `## Integration` tables still contradict each other, and the seed rows the user shaped in Phase 0 still reach no test |
| SKL-056 | medium | closed | Closed by the AGT-001 fix. `agents/idea-interrogator.md:4` is `tools: [Read]`; the question moved to `skills/exploring-ideas/SKILL.md:39-55` step 2b. Detail under AGT-001 |

Counts: 4 high — 4 closed, 0 open. 14 medium — 11 closed, 1 superseded, 2 open.

## AUDIT-4 — agents

| ID | severity | status | evidence |
|---|---|---|---|
| AGT-001 | blocker | closed | All six pieces present. `agents/idea-interrogator.md:4` `tools: [Read]`; `:22` `## Input` gains `selected_segments`; `:29`/`:33` `## Output` requires `candidate_segments` (2-4, `label`+`description`+`confidence`); `:55` records `holders` empty on call 1; `:57` states the no-`AskUserQuestion` rule; `:63-64` workflow steps 3 and 4 split across the two calls. Skill side: `skills/exploring-ideas/SKILL.md:37` (2a), `:39-55` (2b, literal block, `"header": "Holders"`, `multiSelect: true`, the `Someone else` option), `:57` (2c, with the empty-`holders` branch to step 8). Downstream reads the confirmed holders, not the candidates: `:59` step 3 and `:61` step 4 both cite "step 2c". `skills/exploring-ideas/agents.md:7-8` splits the invocation rows at 2a and 2c, `:26` Contracts row reads Tools `Read` and inputs `idea_line, idea_id, brief_path, selected_segments`, `:16` carries the no-question paragraph. `skills/exploring-ideas/templates/questions.md` exists |
| AGT-002 | blocker | **open** | Half done. The allowlist is fixed: `specs/11-subagent-catalog.md:2087` `## Templates` rule 2 now reads "**`AskUserQuestion` appears in no agent file.**" with the strip rule and the return-candidates-then-reinvoke pattern spelled out, and no agent file lists the tool (mechanical check 3). But the catalog's own registry still ships the dead tool: `specs/11-subagent-catalog.md:121`, entry 1 `idea-interrogator`, reads ``- **tools**: `Read`, `AskUserQuestion` ``, and its `output` schema at `:128-135` lists `required: [idea_id, problem_statement, holders, today, why_now, weak_signals, open_questions]` with no `candidate_segments` and no `selected_segments` input. Reachable path: the catalog `## Workflow` step 3 generates the 46 files from these entries, so a regeneration reinstates `tools: [Read, AskUserQuestion]` on `idea-interrogator` and drops the 2a/2c contract the skill now depends on. The fix spec's item 5 named exactly this edit |
| AGT-014 | blocker | closed | One object. `agents/alignment-auditor.md:27` "One JSON object on stdout and nothing else: the `devforgeai/verifier/1` envelope …, with this agent's own top-level fields under `payload`"; three `<example>` instances at `:33`, `:59`, `:85` each showing a single object with `schema: devforgeai/verifier/1`, the envelope keys at top level and `payload: { checks_run, blocking_findings }`. The `schema`/`findings` collision is gone — `findings[]` carries `id`/`severity`/`summary`/`evidence` plus `kind`/`left`/`right`/`ids`/`resolution` inline. Gate path updated in lockstep on all three sides: `cli/templates/gates.default.toml:211`, `specs/04-constitute.md:399`, `skills/establishing-context/SKILL.md:191` and `agents.md:40` all read `verifiers.alignment_auditor.payload.blocking_findings`. AGT-031 rides along: `:118` "two `block` findings over one pair take `passed` down by one rather than by two". *One defect remains inside this file — see Regressions R1* |
| AGT-015 | blocker | closed | Identical treatment. `agents/architecture-reviewer.md:29` one object under the verifier schema, with `payload.requirements_reviewed`, `payload.send_back_requirements`, `payload.blocking_findings` (`:52-54`, `:82-84`, `:103-105`, `:116`). The two `report_metric` checks follow: `cli/templates/gates.default.toml:203` and `:219`, `specs/04-constitute.md:391`/`:407` and `:239`, `skills/establishing-context/SKILL.md:169`, `:190`, `:192` all carry `.payload.` |
| AGT-016 | blocker | closed | `grep -n additionalProperties agents/flow-integrity-auditor.md` returns `:41`, `:46`, `:53` — the two inner objects of `actorless[]`/`contradictions[]` keep theirs, and the one at `:41` is now inside the `payload` object, not at the top level. `:27` header and `:29-40` schema agree: `required: [schema, subagent, id, passed, total, unit, findings, payload]`, `unit` const `flows`, `findings` `maxItems: 0`. One schema, one object; the self-contradiction across three lines is gone |
| AGT-017 | blocker | closed | `agents/requirement-coverage-auditor.md:32` "One JSON object … with the fields of the audit under `payload`"; `:36` required list is the eight envelope keys; `subject_id` retained as a duplicate inside `payload` (`:52`, `:54`) rather than dropped. The consumer matches name for name: `skills/designing-interfaces/SKILL.md:100` reads `payload.subject_id`, `payload.screens[]`, `payload.screens_without_req[]`, `payload.flows_without_req[]`, `payload.covered`; `:142` Subagents row and `:174-175` Send-back rows use the same `payload.` paths |
| AGT-003 | high | closed | The narrower form of the fix spec was taken, and the hook exists. `agents/code-quality-auditor.md:4` `tools: [Read, Grep, Glob, Bash]`; `cli/src/hooks/run.rs:785-793` `METRICS_AGENTS` maps `code-quality-auditor` → `[verify].metrics_command`; `:895-945` `metrics_agent_decision()` matches on `agent_type`, returns `permissionDecision: "allow"` when the command equals the configured string and a deny naming the `config.toml` key otherwise; `:820` calls it from `pre_tool_use_shell` ahead of the producer scan. The `Bash|PowerShell` PreToolUse arm at `hooks/settings.hooks.json` routes it. The `method: command` branch is reachable for the first time |
| AGT-004 | high | closed | Same mechanism. `agents/dead-code-detector.md:4` `tools: [Read, Grep, Glob, Bash]`; `cli/src/hooks/run.rs:790-792` maps `dead-code-detector` → `[verify].call_graph_command` |
| AGT-054 (rides with 003/004) | medium | closed | The over-broad `Bash(devforgeai:*)` prefix grant is gone from both files, so `phase set`, `report ingest`, `hook install` and `trust pin` are no longer admitted to a read-only verifier. The residual `Bash` is narrowed at the permission layer by the deny arm above, which `ANTHROPIC-GUIDANCE.md` §1 makes unbypassable; `cli/src/hooks/run.rs:806-815` adds an independent `trust pin` refusal on every shell call |
| AGT-005 | high | closed | Option B, implemented on both sides. `agents/ui-spec-writer.md:24` `## Input` gains `figma_context`; `:50` step 2 invokes `frontend-design:frontend-design` with an absent-skill fallback; `:52` reads `figma_context` and states why the agent makes no Figma call of its own. The skill produces it: `skills/designing-interfaces/SKILL.md:106` step 13 runs `figma:figma-design-to-code` in the main conversation and holds the result as `figma_context`, `:108` passes it, `:110` gives the empty-string behaviour; `references/spec.md:50`, `references/brand.md:94` and `agents.md:28`/`:34` agree |
| AGT-006 | high | closed | The recommended resolution. `agents/mockup-designer.md:4` `tools: [Read, Write, Glob, Skill]` (no `Artifact`); `:57` step 2 invokes `frontend-design:frontend-design` and records why the `design` skill is not the one invoked ("its output path publishes a canvas through the `Artifact` tool, which this agent does not hold"), plus the absent-skill fallback. `skills/designing-interfaces/agents.md:36` carries the same reason |
| AGT-009 | high | closed | Measured, not inspected. Parsing all 46 frontmatter blocks: every `description` is **≤160 characters** (min 104, max 154), total **6,353 characters** across the 46 — down from the audit's 14,851, and below the ~5,800-plus estimate the fix spec projected. No description carries a workflow step number, batch position, slash command or verifier registration |
| AGT-010 | high | closed | `specs/11-subagent-catalog.md:2037` `## Templates` frontmatter block now reads `description: <one sentence, at most 160 characters: …>`, and `:2032` relaxes the four-key rule to "four required keys, in the order below, **plus the optional keys rule 5 admits**" — which is what lets the shipped files carry `disallowedTools`, `maxTurns`, `memory` and `effort` (mechanical check 3 finds seven distinct key sets across the 46). A regeneration from the template no longer reintroduces the purpose sentence |
| AGT-018 | high | closed | `agents/kill-case-builder.md:27` one object with the six kill-case fields under `payload`; `:31` the eight required envelope keys; `:112` fixes `total`/`passed`/`unit: objections`/`findings: []` and explains why no `warn` entry can lower `passed`. Workflow steps write `payload.kill_case` and `payload.strongest_objection` by name (`:126`, `:127`), so the two-block ambiguity is gone |
| AGT-019 | high | closed | The `block` case exists and a workflow step produces it, not just the Output prose. `agents/standards-reviewer.md:66-70` "`passed` is `total` minus the number of those files carrying a `block` finding … except a departure from a rule a `CON-nnn` in `constraints` binds, which is `block`. Without that exception every finding would be `warn`"; `:109` workflow "…is `block`; one that resolves to none of them stays `warn`"; `:112` sets `passed` from it. `verify-light` at `min_ratio = 1.0` can now fail on this agent |
| AGT-020 | high | closed | The coupling is broken in the direction the fix spec chose. `agents/dead-code-detector.md:74` "A dead-code finding is `warn` and no higher, and `passed` equals `total`"; the count moves to `payload.dead` (`:62` example, `:121` workflow step); `:82` names `verifiers.dead_code.payload.dead` as the `report_metric` a project adds if it wants dead code to gate. A `warn` no longer fires a send-back on a `confidence: 0.6` grep count |
| AGT-021 | high | closed | Same shape. `agents/code-quality-auditor.md:79` "Both findings are `warn`, and `passed` equals `total`"; `payload.over_ceiling` at `:66` and `:131`; `:86` names `verifiers.quality.payload.over_ceiling`. `skills/validating-quality/agents.md:91-92` carries both exceptions in the skill's own severity paragraph, which the fix spec required |
| AGT-032 | high | closed | All fourteen carry the recall sentence, verified one file at a time (line-wrapping defeats a single-line grep, so each was read). Exact wording at `ac-compliance-verifier:19-22`, `standards-reviewer:19-20`, `anti-pattern-scanner:18-21`, `constraint-auditor:20-23`, `coverage-gap-auditor`, `dead-code-detector`, `deferral-validator:20-23`, `security-auditor`, `code-quality-auditor`, `adr-conformance-reviewer`, `architecture-reviewer`, `alignment-auditor`, `story-invest-auditor`, and `kill-case-builder:12` (adapted to objections: "including the uncertain and the weak ones … an objection dropped here is not filtered, it is lost") |
| AGT-033 | high | closed | The band widened and the cap is gone. `skills/validating-quality/agents.md:97-99` `FIND-(base + 100k)` through `FIND-(base + 100k + 99)`; `:104` "no agent truncates its list and no agent carries a `truncated` flag". `skills/validating-quality/references/findings-and-deferrals.md:10-16` states the same, in the positive ("every subagent reports every item it has, at every severity"). `grep -rn "truncated\|first twenty\|more than twenty"` over all 46 agents, all nine SKILL.md, all references and all `agents.md` returns only those two negations — no agent schema carries `truncated` and no agent step applies the cap |

Counts: 6 blocker — 5 closed, 1 open. 11 high — 11 closed, 0 open.

## Mechanical checks

Checks 1, 5, 6, 7, 8 and 9 are another verifier's; not run here.

### Check 2 — ceremony scan

`scripts/ceremony_scan.py` exists (the docs fixer wrote it), so the brief's fallback was not needed.

```
$ python scripts/ceremony_scan.py
no ceremony in 94 file(s)
EXIT=0
```

The script's default set is `skills/**/SKILL.md`, `skills/**/references/*.md` and `agents/*.md` —
it does **not** include `skills/*/agents.md`, which the override's file list names. Re-run over the
full set:

```
$ python scripts/ceremony_scan.py --paths skills agents
no ceremony in 180 file(s)
EXIT=0
```

Zero hits over all nine `SKILL.md`, all 39 reference files, all nine `agents.md` and all 46
`agents/*.md`. One note for the specs owner: the script's docstring (`scripts/ceremony_scan.py:22-25`)
matches the single-word alternatives **case-sensitively** by deliberate choice, while `AUDIT-3`
SKL-013 and `AUDIT-4` §5 applied the §2 regex case-insensitively. The scan is therefore narrower
than the audits' scan; a lower-case `must`/`should` in instruction prose passes the script. Not a
defect in any shipped file — both scans return zero — but the script and the audits do not implement
the same rule, and `specs/00-conventions.md` names the script as the rule's one implementation.

### Check 3 — frontmatter of all 46 `agents/*.md`

Parsed programmatically; no exceptions.

```
files: 46
BAD: none                      (no missing name/description/tools; no AskUserQuestion in any tools;
                                every name equals its file stem)
desc len min/max/total: 104 154 6353
over 160 chars: 0
key sets: {name,description,tools,model,disallowedTools,maxTurns}      19
          {name,description,tools,model}                              10
          {name,description,tools,model,disallowedTools}                9
          {…,disallowedTools,effort,memory}                             2
          {…,disallowedTools,effort}                                    2
          {…,disallowedTools,memory}                                    2
          {name,description,tools,model,maxTurns}                       1
          {name,description,tools,model,effort}                         1
models: opus 25, sonnet 21
```

`name`, `description` and `tools` present on all 46. `AskUserQuestion` in none. Description lengths
104-154, total 6,353 characters (~1,590 tokens at chars/4, about 11 % of the §2 15,000-token
threshold). `model` split 25 opus / 21 sonnet, matching catalog Decision 13. The optional keys
(`disallowedTools`, `maxTurns`, `memory`, `effort`) are the AGT-046/048/050/051 enhancements, now
admitted by the amended template rule (AGT-010).

### Check 4 — all nine `skills/*/SKILL.md`

| skill directory | `name:` | argument-hint | `Bash(devforgeai:*)` | `PowerShell(devforgeai:*)` | `disable-model-invocation` | first `!` line |
|---|---|---|---|---|---|---|
| exploring-ideas | `explore` | yes | yes | yes | true | *(none — allocation moved to step 1)* |
| discovering-requirements | `discover` | yes | yes | yes | true | `doc load discover-entry "$1"` |
| establishing-context | `constitute` | yes | yes | yes | true | **`gate require constitute $1`** |
| planning-work | `plan` | yes | yes | yes | true | **`gate require plan $1`** |
| implementing-stories | `build` | yes | yes | yes | true | **`gate require build $1`** |
| validating-quality | `verify` | yes | yes | yes | true | **`gate require verify $1`** |
| releasing-software | `release` | yes | yes | yes | true | `story list --status built --json` |
| designing-interfaces | `design` | yes | yes | yes | *absent (correct)* | *(none — allocation moved to step 12)* |
| improving-framework | `reflect` | yes | yes | yes | true | `report aggregate $ARGUMENTS --json` |

All nine `name:` values equal the slash names. `gate require <phase> $1` leads the preamble of
constitute, plan, build and verify. `disable-model-invocation: true` on eight; `designing-interfaces`
omits it, which is what keeps the Explore and Plan Skill-tool calls working.

`find . -type d -name commands` → **no `commands/` directory anywhere** (checked repo-wide, not only
the root; `cli/templates/`, `evals/` and `skills/` are clean).

`exploring-ideas` and `designing-interfaces` carry no preamble at all. This is the FIX-PLAN decision 6
resolution of SKL-027 ("allocation moves into the workflow step, not the preamble"), which supersedes
the audit's `|| true` wording. Both skills state it and both allocate where the id is needed:
`skills/exploring-ideas/SKILL.md:23` and `:31` (step 1, fresh run only, with the `DFA-E215` branch),
`skills/designing-interfaces/SKILL.md:104` (step 12, once per screen carrying no `UI-nnn`). Neither
run can now abort on a prefix it does not spend.

### Cross-file consistency (the audits could not check these)

**Subagent return fields against the agent's output contract.** Every one of the 20 registered
verifiers carries a `payload` object in its `## Output`
(`grep -l '"payload"' agents/*.md` → 20, exactly the 20 registered verifiers; the other 26 do not,
correctly). Every skill-side read carries the `payload.` segment where the agent puts the field
there — checked pair by pair for `alignment-auditor`, `architecture-reviewer`,
`requirement-coverage-auditor`, `flow-integrity-auditor`, `kill-case-builder`, `dead-code-detector`,
`code-quality-auditor` and `standards-reviewer`. No skill reads a field its agent does not emit, and
no skill names a metric path missing the `payload.` segment. One exception, in eval fixtures rather
than in a skill body — Regression R2.

**Every agent a SKILL.md invokes exists and is listed.** Programmatic check over all nine skills:
`mentioned-not-listed` is empty for every skill, and every mentioned name resolves to a file in
`agents/`. (`agents.md` files legitimately name extra agents in cross-reference prose — e.g.
`exploring-ideas/agents.md` names five others in its `## Shared lineage` section — which is not a
mismatch.)

**Every `devforgeai` subcommand a SKILL.md runs exists in `cli/src/cli.rs`.** Extracted all 25
distinct `devforgeai <verb> <sub>` forms from the nine `SKILL.md`, the 39 references, the nine
`agents.md` and the 46 agent files, and matched each against the clap enums: `doc validate|load|accept|reopen`,
`report ingest|show|aggregate|note`, `gate check|require`, `story validate|files|list`, `phase set`,
`worktree ensure|list|remove`, `explore prune`, `context audit`, `antipattern scan`, `design lint`,
`config get`, `stack detect`, `handoff`, `commit`, `init`. **All 25 resolve** — no skill or agent
runs a subcommand the CLI does not define.

Argument values were checked too, not only the verbs. The nine distinct `doc load` kinds the corpus
uses — `discover-entry`, `requirements`, `context` (with `all`), `story`, `sprint` (with `-`),
`ui-spec`, `adr` (with `all`), `qa-report` — are all present in `DOC_NAMES`
(`cli/src/cmd/doc.rs:341-357`) and all have match arms (`:213`, `:220`, `:221`, `:224`, `:243`,
`:262`, `:263`, `:265`). The three `report show <id> <phase>` phases used (`build`, `verify`,
`release`) pass `check_phase` at `cli/src/cmd/report.rs:30`. No preamble aborts on an argument the
CLI rejects.

**Spec side of the collapse (SKL-017, SKL-024).** `specs/BUILD-BRIEF.md` §2 no longer says "No other
frontmatter keys": `:42` and `:44` now name `argument-hint` and `disable-model-invocation` with the
eight/one split, and `:95` records that the portable `quick_validate.py` rejects those two keys, so
the check runs over a stripped copy. A repo-wide grep for `commands\<name>.md` / `commands/<…>` over
`specs/00-conventions.md`, `specs/BUILD-BRIEF.md` and `specs/02-*.md` … `specs/10-*.md` returns
**zero hits**. Two stale *preamble* sentences survive inside two spec files — Regression R5.

**Every `report_field` the fixed agents cite exists in the registry.** `cli/src/config.rs:469-602`
holds the 20 defaults; the paths the AGT-014/017/018/020/021 rewrites now name all resolve —
`verifiers.kill_case` (`:469`), `verifiers.flow_integrity` (`:476`),
`verifiers.architecture_reviewer` (`:483`), `verifiers.alignment_auditor` (`:490`),
`verifiers.dead_code` (`:560`, for the new `payload.dead` metric),
`verifiers.quality` (`:581`, for `payload.over_ceiling`),
`verifiers.requirement_coverage` (`:595`). No agent's `[[verifier]]` block or recommended metric
path names a `report_field` the registry does not carry.

**Short-name install paths.** No file references `.claude/skills/<long-dir-name>/`. The only mentions
of the long names outside their own directories are `cli/tests/init.rs:727-764`, which *assert those
paths are absent*, and `README.md:72-74`, which passes `skills/exploring-ideas` as a **source**
directory to the eval runner (not an install path). Both correct.

## Open items

**SKL-002 — the Reflect template is a two-document YAML stream and fails its own `doc validate`.**
What remains: delete the leading and trailing `---` from
`skills/improving-framework/templates/reflect-report.yaml` (35 → 33 lines) and from the
`## Templates` fence of `specs/10-reflect.md:524` and `:559`, so the file opens on `schema:` like the
other six YAML templates. Owners: F4 (template), F6 (spec) — the two must move together, since the
template is held byte-identical to the spec fence. Reachable path, measured:
`devforgeai doc validate` on a byte copy of the shipped template exits 1 with `DFA-E202 …
deserializing from YAML containing more than one document is not supported`, because
`cli/src/doc/frontmatter.rs:126-139` strips only the *leading* marker and `mapping()` at `:197`
rejects the rest. `improving-framework` step 9 instructs writing the reflect report from this file.

**SKL-053 — Explore still advertises a consumer that does not read the file.** What remains: one of
two edits, not both. Either `skills/implementing-stories` gains `seed-data.json` in its
`## Entry` read-from-disk list and in `references/tdd-cycle.md` 7.1's prompt-field table (so
`ac-test-writer` receives the seed rows), or
`skills/exploring-ideas/templates/decision.yaml:37-40` drops `consumer: implementing-stories` from
`carry_forward` entry 6 and `specs/02-explore.md` `## Integration` drops its Build row. Owner: F4 for
both skill-side options, F6 for the spec table. Reachable path: a `promote` decision writes a
`carry_forward` entry naming a consumer that never opens the file; the tests `ac-test-writer` writes
use invented fixture rows. `grep -rn "seed-data\|seed_data"` over `skills/implementing-stories/`,
`specs/06-build.md` and `agents/ac-test-writer.md` returns zero.

**AGT-002 — the catalog registry still ships the dead tool on entry 1.** What remains: at
`specs/11-subagent-catalog.md:121`, ``- **tools**: `Read`, `AskUserQuestion` `` becomes
``- **tools**: `Read` ``; the entry's `input` gains the selected labels and its `output` schema at
`:128-135` gains `candidate_segments` (2-4 × `{label, description, confidence}`) and the note that
`holders` is `[]` on the first call. This is item 5 of the AGT-001/002 fix spec, and it is the only
one of the six not done. Owner: F3 owns the catalog's `## Templates` and Decision 1 registry; the
`## Subagents` entry falls between F3 and F6 and needs routing. Reachable path: the catalog's own
`## Workflow` step 3 generates the 46 agent files from these entries, so any regeneration restores
`tools: [Read, AskUserQuestion]` on `idea-interrogator` — the exact condition AGT-002 was raised to
prevent — and drops the `candidate_segments`/`selected_segments` contract that
`skills/exploring-ideas/SKILL.md` steps 2a-2c now depend on. Rule 2 of `## Templates` being fixed
does not cover this, because rule 2 constrains new files and the entry is what a regeneration reads.

## Regressions

**R1 — `agents/alignment-auditor.md:125` emits a severity outside the closed enum, which re-creates
the exact AGT-014 failure.** Workflow step 2 reads:

> One rule worded two ways is `restated`: … **Severity is `medium`.**

`medium` is not in the enum. The same file's `## Output` at `:103` says a `restated` finding is
`warn` or `info`, and every other workflow step of the file uses the enum correctly (`:126`
`severity `block``; `:127`, `:128`, `:129` `severity `warn``). A scan for `Severity is` across all 46
agents finds this is the only occurrence outside the enum
(`constraint-auditor:99` `block`, `security-auditor:114` `block`/`warn`).

Reachable path, in code: `cli/src/report.rs:352-357` rejects any finding whose `severity` is not
`block | warn | info` — `"subagent '{name}' output is not devforgeai/verifier/1: severity '{}' is
outside block, warn, info"` — and `cli/src/report.rs:581-582` tests exactly that rejection. A model
following step 2 emits `"severity": "medium"`, the whole block fails to parse, `report ingest` writes
`verifiers.alignment_auditor` at `status: unparsed`, and the constitute gate's `report_metric` on
`verifiers.alignment_auditor.payload.blocking_findings`
(`cli/templates/gates.default.toml:211`) reads an absent path and fails on every run. That is the
AGT-014 failure verbatim, reintroduced by one word in the step the fix rewrote. One-word fix:
`medium` → `warn`, matching `:103`. Owner: F3.

**R2 — the five `skills/*/evals/fixtures/gates-default.toml` are pre-`payload` and carry a wrong key.**
All five (`designing-interfaces`, `discovering-requirements`, `establishing-context`,
`exploring-ideas`, `planning-work`) diverge from `cli/templates/gates.default.toml` in four places:

```
184c184  <   path = "accepted_by"                                          (fixture)
         >   field = "accepted_by"                                         (template)
203c203  < metric = "verifiers.architecture_reviewer.blocking_findings"
         > metric = "verifiers.architecture_reviewer.payload.blocking_findings"
211c211  < metric = "verifiers.alignment_auditor.blocking_findings"
         > metric = "verifiers.alignment_auditor.payload.blocking_findings"
219c219  < metric = "verifiers.architecture_reviewer.send_back_requirements.length"
         > metric = "verifiers.architecture_reviewer.payload.send_back_requirements.length"
```

These fixtures are not decoration: every `cases.jsonl` entry in those five suites writes them into
the eval workspace as `.devforgeai/gates.toml` (`"FIXTURE:gates-default.toml"` appears on every case
of `skills/designing-interfaces/evals/cases.jsonl`, and the same in the other four). Two reachable
consequences under FIX-PLAN decision 3 ("every gate check kind fails closed"): the three
`report_metric` checks read paths that no longer exist after the payload split, and the
`fields_present` check named `accepted` sets `path` (which `cli/src/gates.rs:81` treats as the
*document* path) instead of `field`, so it looks for a document literally named `accepted_by`.
Every eval run over discover, constitute, plan and design now carries a gates file whose checks fail
for reasons unrelated to the skill under test. Owner: F5 (`skills/*/evals/*`), who needs F1/F2's
current `cli/templates/gates.default.toml` as the source.

**R3 — `agents/recommendation-drafter.md:23` still names a directory decision 6 deleted.** The
`repo_paths` input row reads "the paths under `skills/`, `agents/`, **`commands/`**, `hooks/`, and
`specs/` that `Glob` returns". `commands/` no longer exists (check 4). A `REC-nnn` naming a file
under `commands/` names nothing, and the agent is instructed to glob a path that returns empty on
every run. Low blast radius, one-word fix. Owner: F3.

**R4 — `specs/11-subagent-catalog.md:121` contradicts the file it generates.** Recorded as AGT-002
above rather than repeated here; listed in this section because it is a spec-versus-shipped-file
contradiction in the same class as R1-R3, and because the generation direction runs from the spec to
the file, which makes it the more durable of the two.

**R5 — two specs still say the allocation runs in a preamble that no longer exists, and each
contradicts itself.** SKL-027's resolution (FIX-PLAN decision 6) moved both allocations into workflow
steps and both preambles are gone from the shipped skills (check 4). The specs record the move in one
place and the old shape in another:

- `specs/02-explore.md:23` and `:945` (Decision 26) both put the allocation at step 1 and explain why
  it left the preamble — but `:435` `## CLI calls` still reads
  `` | `devforgeai doc validate --allocate IDEA` | command `!` preamble | … used by step 1 on a fresh
  run and ignored on a remedy run | ``. There is no command file and no preamble; the trigger column
  is wrong and the "ignored on a remedy run" clause describes the behaviour the fix removed.
- `specs/08-design.md:34`, `:209` and `:387` put it at step 12, with `:387` stating "**never in a
  preamble**, because a `--sketch` or `--brand` run allocates none" — but `:869` Decision 24 still
  opens "**The `/design` command carries one preamble line, `devforgeai doc validate --allocate
  UI`**" and argues for the discard the fix eliminated. It also cites
  `specs/02-explore.md` Decision 24 "for the same reason", while that file's live decision on this is
  now 26, which says the opposite.

Reachable path: these are the `## CLI calls` and `## Decisions` sections a future fixer or a
regeneration reads as authority, and both would restore an unconditional preamble allocation — the
SKL-027 failure. The two `## Command` sections are the other half of SKL-024 step 8. Owner: F6.
