---
schema: devforgeai-spec/1
doc: subagent-catalog
status: draft
produced_by: orchestrator
consumes: [00-conventions]
open_questions: []
---

# Subagent catalog

## Scope

This document is the single source of truth for every subagent the framework ships under `agents/`. It holds one entry per subagent, consolidated across the nine skill specs, carrying the conventions §10 fields exactly as the owning skill spec wrote them, plus two fields §10 leaves to this document: `owned_by`, the one skill whose workflow invokes the agent, and `report_field`, the `config.toml` `[[verifier]]` key for a registered verifier. It fixes the file format every `agents/<name>.md` follows, resolves every name collision across the nine specs, records the disposition of all 117 files under `C:\Users\bryan\.claude\agents\`, gives the per-skill invocation map that each skill's `agents.md` carries, and states the model budget and the rule that produced it.

This document defines no new subagent, changes no contract a skill spec fixed, and owns no phase. Where a contract has to change for the catalog to be consistent, the change is written under `## Decisions` as an amendment against the named spec, and the entry in `## Subagents` carries the contract as that spec wrote it until the amendment lands. This document has no slash command, no gate, no send-back, no handoff, and no evals, because it is a registry rather than a phase; those five sections read `none` with the reason. Agent file content, eval cases, and gate thresholds stay in the owning skill specs.

## Inputs

| Input | Path | What this document reads from it |
|---|---|---|
| Conventions | `specs/00-conventions.md` | §2 forbidden content, §3 repository layout, §7 SubagentStop row, §10 subagent fields, §11 template |
| CLI spec | `specs/01-cli.md` | `## Subagents` (`devforgeai/verifier/1` stdout contract, the `[[verifier]]` register), `## Outputs` (`config.toml` `[[verifier]]` table schema, the report `verifiers` block), `## Gate` (the v1 registry contents), `## Decisions` 11, 12, 46 |
| Explore spec | `specs/02-explore.md` | `## Subagents` (5 entries), `## Decisions` 19, 20, 21, 22, 23 |
| Discover spec | `specs/03-discover.md` | `## Subagents` (4 entries), `## Decisions` 9, 18 |
| Constitute spec | `specs/04-constitute.md` | `## Subagents` (3 entries plus the seven-row disposition table), `## Decisions` 10, 15, 16, 17, 18, 20, 23, 26 |
| Plan spec | `specs/05-plan.md` | `## Subagents` (5 entries plus the five-row disposition table), `## CLI calls` (`[[verifier]]`), `## Decisions` 27, 28 |
| Build spec | `specs/06-build.md` | `## Subagents` (7 entries plus the thirteen-row disposition table), `## Templates` (`[[verifier]]`), `## Decisions` 18, 19, 20, 21, 23, 25, 31, 40 |
| Verify spec | `specs/07-verify.md` | `## Subagents` (10 entries plus the ten-row adaptation table and the three-row replacement table), `## CLI calls` (`[[verifier]]`), `## Decisions` 6, 10, 25, 26, 27, 28, 29 |
| Design spec | `specs/08-design.md` | `## Subagents` (4 entries), `## Decisions` 17, 18, 20, 21 |
| Release spec | `specs/09-release.md` | `## Subagents` (4 entries), `## Gate` (`[[verifier]]`), `## Decisions` 8, 25, 26, 27, 31 |
| Reflect spec | `specs/10-reflect.md` | `## Subagents` (4 entries plus the seven-row disposition table) |
| Existing agent set | `C:\Users\bryan\.claude\agents\*.md` (47 files) and `C:\Users\bryan\.claude\agents\*\references\*.md` plus `C:\Users\bryan\.claude\agents\references\*.md` (70 files) | frontmatter `name`, `description`, `tools`, `model` of each, for the disposition ledger |

This document reads no conventions §5 phase document. It has no `consumes` ID list beyond `00-conventions`, because a catalog of subagents is a framework artifact and not a phase artifact.

## Outputs

| Output | Path | Written by | Content fixed by |
|---|---|---|---|
| 46 agent files | `agents/<name>.md` | the framework build | `## Templates` agent-file template; the §10 fields of `## Subagents` |
| 9 invocation maps | `skills/<skill-name>/agents.md` | the framework build | `## Templates` agents.md template; the per-skill lists of `## Integration` |
| `[[verifier]]` registry, 20 tables | `.devforgeai/config.toml`, written by `devforgeai init` | the CLI | `## Decisions` 1, which carries the array verbatim and amends `specs/01-cli.md` from 6 entries to 20 |
| Installed copies | `<target>\.claude\agents\<name>.md` | `devforgeai init` | conventions §3 |

The 46 agent files carry no frontmatter key beyond the four of `## Templates`, and no body section beyond the three of `## Templates`. The nine `agents.md` files carry no content beyond the four sections of the agents.md template. This document writes no file under `.devforgeai/`.

## Workflow

Each step names the actor, the input, the output, and the failure path. Steps 1 to 3 run once per framework release; steps 4 to 7 run once per skill invocation in a target project.

1. **Catalog entry — spec author.** Input: a skill spec's `## Subagents` subsection with the nine §10 fields. Output: one row of the `## Subagents` table and one subsection of this document, plus `owned_by` set to that skill and `report_field` set from the spec's `[[verifier]]` table. Failure path: the subsection omits a §10 field; the entry is written with the field absent and the gap is recorded under `## Decisions` as an amendment against that spec.
2. **Name resolution — this document.** Input: the 46 names across nine specs. Output: a name set with no duplicate. Failure path: two specs name one agent for two contracts; `## Decisions` states which spec renames its agent, and the entry carries the surviving contract.
3. **File generation — framework build.** Input: the §10 fields of one `## Subagents` subsection plus the `## Templates` agent-file template. Output: `agents/<name>.md` with the four frontmatter keys and the three body sections. Failure path: the `tools` list holds a tool the purpose does not use; the file is not generated and the spec entry is corrected first.
4. **Install — CLI, `devforgeai init`.** Input: `agents/*.md` in the framework repo and the `[[verifier]]` rows of `## Integration`. Output: `<target>\.claude\agents\*.md` and the `[[verifier]]` array in `<target>\.devforgeai\config.toml`. Failure path: `config.toml` exists; `init` leaves `[[verifier]]` at its previous value, per `specs/01-cli.md` `## Outputs`.
5. **Invocation — skill.** Input: the skill's `agents.md` order and the prompt fields the agent's `input` field names. Output: one `Agent` call per agent, batched into one message where `## Integration` marks the agents parallel. Failure path: an agent returns output that does not parse against its schema; the skill re-invokes that one agent once with the parse error appended, and a second failure follows the owning spec's stated path.
6. **Ingest — CLI, `report ingest`, fired by the SubagentStop hook.** Input: the agent's stdout and its `config.toml` `[[verifier]]` row. Output: the block at `verifiers.<report_field>` of `.devforgeai/reports/<ID>-<phase>.yaml`, with the agent's `findings` appended to the report `findings` list. Failure path: the name is absent from the registry, which is `DFA-W411`, exit 0, nothing written; the stdout is not `devforgeai/verifier/1`, which is `DFA-E410`, exit 0, and the block is written with `status: unparsed`.
7. **Gate read — CLI, `gate check`.** Input: the ingested block and the phase's `verifier_pass` or `report_metric` checks. Output: the check result in `reports/<ID>-<phase>.yaml`. Failure path: an absent block is `DFA-E316`; a ratio below `min_ratio` is `DFA-E317`.

## Subagents

Forty-six subagents across nine skills. Twenty are registered verifiers. Every entry traces to one skill spec's `## Subagents` subsection, and the fields below are that spec's, carried unchanged. `owned_by` is the one skill whose workflow invokes the agent; every agent has exactly one, and `## Integration` states why the "shared subagents" column is `none` on every row.

| # | name | owned_by | derives_from | model | registered_verifier | report_field |
|---|---|---|---|---|---|---|
| 1 | `idea-interrogator` | Explore | `business-coach.md` | opus | no | — |
| 2 | `landscape-scanner` | Explore | `internet-sleuth.md` | sonnet | no | — |
| 3 | `flow-drafter` | Explore | `requirements-analyst.md` | opus | no | — |
| 4 | `prototype-builder` | Explore | `frontend-developer.md` | sonnet | no | — |
| 5 | `kill-case-builder` | Explore | `new` | opus | yes | `verifiers.kill_case` |
| 6 | `persona-mapper` | Discover | `stakeholder-analyst.md` | sonnet | no | — |
| 7 | `requirement-drafter` | Discover | `requirements-analyst.md` | opus | no | — |
| 8 | `epic-grouper` | Discover | `requirements-analyst.md` | sonnet | no | — |
| 9 | `flow-integrity-auditor` | Discover | `context-preservation-validator.md` | opus | yes | `verifiers.flow_integrity` |
| 10 | `architecture-reviewer` | Constitute | `architect-reviewer.md` | opus | yes | `verifiers.architecture_reviewer` |
| 11 | `alignment-auditor` | Constitute | `alignment-auditor.md` | sonnet | yes | `verifiers.alignment_auditor` |
| 12 | `source-tree-mapper` | Constitute | `code-analyzer.md` | sonnet | no | — |
| 13 | `story-decomposer` | Plan | `story-requirements-analyst.md` | opus | no | — |
| 14 | `story-file-set-planner` | Plan | `api-designer.md` | sonnet | no | — |
| 15 | `story-invest-auditor` | Plan | `requirements-analyst.md` | opus | yes | `verifiers.story_invest` |
| 16 | `sprint-sequencer` | Plan | `sprint-planner.md` | sonnet | no | — |
| 17 | `spec-gap-triager` | Plan | `new` | opus | no | — |
| 18 | `ac-test-writer` | Build | `test-automator.md` | opus | yes | `verifiers.ac_testable` |
| 19 | `backend-implementer` | Build | `backend-architect.md` | opus | no | — |
| 20 | `frontend-implementer` | Build | `frontend-developer.md` | sonnet | no | — |
| 21 | `refactor-surgeon` | Build | `refactoring-specialist.md` | sonnet | no | — |
| 22 | `integration-test-writer` | Build | `integration-tester.md` | sonnet | no | — |
| 23 | `story-ac-verifier` | Build | `ac-compliance-verifier.md` | opus | yes | `verifiers.story_ac` |
| 24 | `context-validator` | Build | `context-validator.md` | opus | yes | `verifiers.context` |
| 25 | `ac-compliance-verifier` | Verify | `ac-compliance-verifier.md` | opus | yes | `verifiers.ac_compliance` |
| 26 | `standards-reviewer` | Verify | `code-reviewer.md` | opus | yes | `verifiers.standards` |
| 27 | `anti-pattern-scanner` | Verify | `anti-pattern-scanner.md` | sonnet | yes | `verifiers.anti_patterns` |
| 28 | `constraint-auditor` | Verify | `context-validator.md` | opus | yes | `verifiers.constraints` |
| 29 | `coverage-gap-auditor` | Verify | `coverage-analyzer.md` | sonnet | yes | `verifiers.coverage_gaps` |
| 30 | `dead-code-detector` | Verify | `dead-code-detector.md` | sonnet | yes | `verifiers.dead_code` |
| 31 | `deferral-validator` | Verify | `deferral-validator.md` | opus | yes | `verifiers.deferrals` |
| 32 | `security-auditor` | Verify | `security-auditor.md` | opus | yes | `verifiers.security` |
| 33 | `code-quality-auditor` | Verify | `code-quality-auditor.md` | sonnet | yes | `verifiers.quality` |
| 34 | `adr-conformance-reviewer` | Verify | `architect-reviewer.md` | opus | yes | `verifiers.adr_conformance` |
| 35 | `mockup-designer` | Design | `frontend-developer.md` | sonnet | no | — |
| 36 | `brand-designer` | Design | `new` | opus | no | — |
| 37 | `requirement-coverage-auditor` | Design | `new` | sonnet | yes | `verifiers.requirement_coverage` |
| 38 | `ui-spec-writer` | Design | `ui-spec-formatter.md` | opus | no | — |
| 39 | `deferral-auditor` | Release | `deferral-validator.md` | opus | yes | `verifiers.deferrals` |
| 40 | `deploy-manifest-writer` | Release | `deployment-engineer.md` | sonnet | no | — |
| 41 | `api-doc-writer` | Release | `documentation-writer.md` | sonnet | no | — |
| 42 | `guide-writer` | Release | `documentation-writer.md` | opus | no | — |
| 43 | `observation-miner` | Reflect | `observation-extractor.md` | sonnet | no | — |
| 44 | `session-pattern-reader` | Reflect | `session-miner.md` | opus | no | — |
| 45 | `debt-aggregator` | Reflect | `technical-debt-analyzer.md` | sonnet | no | — |
| 46 | `recommendation-drafter` | Reflect | `framework-analyst.md` | opus | no | — |

`derives_from` is a relative basename above; the full value in each subsection is the absolute path under `C:\Users\bryan\.claude\agents\` that the owning spec wrote, or `new`. The four registry fields of rows 9, 10, 11, and 37 are assigned by `## Decisions` 3, because those owning specs declared `registered_verifier: yes` without writing a `[[verifier]]` table. The whole 20-entry array, in the order `devforgeai init` writes it, is in `## Decisions` 1. Rows 31 and 39 share a `report_field` string and do not collide: `specs/01-cli.md` `## Outputs` makes `name` unique across the registry and leaves `report_field` free, and the two write into `reports/STORY-nnn-verify.yaml` and `reports/vX.Y.Z-release.yaml` respectively.

### 1. idea-interrogator

- **name**: `idea-interrogator`
- **owned_by**: Explore · `exploring-ideas`
- **derives_from**: `C:\Users\bryan\.claude\agents\business-coach.md`
- **purpose**: Probe one raw idea until the problem, the people who hold it, their current workaround, and the reason the timing is now are each stated as a fact rather than a hope.
- **tools**: `Read`
- **model**: `opus` — separating a real problem from an enthusiasm is the judgment this whole phase rests on.
- **input**: `idea_line` from `$ARGUMENTS`; `idea_id`, the run's `IDEA-nnn`; `brief_path`, `.devforgeai/explore/brief.md` when the run is a resume and null otherwise; and, on the second call alone, `selected_segments`, the labels the user picked at the skill's workflow step 2b.
- **output**: One JSON object on stdout and nothing else. `candidate_segments` is what the first call drafts and the skill turns into the options of the question it asks; `holders` is `[]` on that call and filled on the second, one entry per selected label. The schema the agent file fixes:

```json
{ "type": "object", "required": ["idea_id","problem_statement","candidate_segments","holders","today","why_now","weak_signals","open_questions"],
  "properties": {
    "idea_id": { "type": "string", "pattern": "^IDEA-[0-9]{3}$" },
    "problem_statement": { "type": "string", "maxLength": 220 },
    "candidate_segments": { "type": "array", "minItems": 2, "maxItems": 4, "items": { "type": "object",
      "required": ["label","description","confidence"], "properties": {
        "label": { "type": "string", "maxLength": 60 },
        "description": { "type": "string", "maxLength": 200 },
        "confidence": { "type": "number", "minimum": 0, "maximum": 1 } } } },
    "holders": { "type": "array", "minItems": 0, "maxItems": 4, "items": { "type": "object",
      "required": ["segment","attributes","frequency"], "properties": {
        "segment": { "type": "string" },
        "attributes": { "type": "array", "minItems": 2, "maxItems": 4, "items": { "type": "string" } },
        "frequency": { "type": "string", "enum": ["daily","weekly","monthly","rarely"] } } } },
    "today": { "type": "array", "minItems": 1, "maxItems": 4, "items": { "type": "object",
      "required": ["approach","cost","breaks"], "properties": {
        "approach": { "type": "string" }, "cost": { "type": "string" }, "breaks": { "type": "string" } } } },
    "why_now": { "type": "array", "minItems": 0, "maxItems": 3, "items": { "type": "object",
      "required": ["month","change"], "properties": {
        "month": { "type": "string", "pattern": "^[0-9]{4}-[0-9]{2}$" }, "change": { "type": "string" } } } },
    "weak_signals": { "type": "array", "maxItems": 5, "items": { "type": "string" } },
    "open_questions": { "type": "array", "maxItems": 5, "items": { "type": "string" } } } }
```

- **invoked_at**: workflow steps 2a and 2c, alone each time; skipped on a remedy run.
- **registered_verifier**: `no`
- **report_field**: none

### 2. landscape-scanner

- **name**: `landscape-scanner`
- **owned_by**: Explore · `exploring-ideas`
- **derives_from**: `C:\Users\bryan\.claude\agents\internet-sleuth.md`
- **purpose**: Find who already solves this problem, how they charge for it, and which technologies a solution would rest on, in one pass with no file writes.
- **tools**: `WebSearch`, `WebFetch`, `Read`
- **model**: `sonnet` — retrieval and tabulation against a fixed schema, with the judgment left to `kill-case-builder`.
- **input**: `problem_statement` and `holders[].segment` from step 2; the run's `IDEA-nnn`.
- **output**: One JSON object on stdout and nothing else. The schema the agent file fixes:

```json
{ "type": "object", "required": ["idea_id","competitors","technologies","closest_match","sources"],
  "properties": {
    "idea_id": { "type": "string", "pattern": "^IDEA-[0-9]{3}$" },
    "competitors": { "type": "array", "maxItems": 8, "items": { "type": "object",
      "required": ["name","url","approach","price","gap"], "properties": {
        "name": { "type": "string" }, "url": { "type": "string" }, "approach": { "type": "string" },
        "price": { "type": "string" }, "gap": { "type": "string" } } } },
    "technologies": { "type": "array", "maxItems": 8, "items": { "type": "object",
      "required": ["capability","candidate","maturity","license","source"], "properties": {
        "capability": { "type": "string" }, "candidate": { "type": "string" },
        "maturity": { "type": "string", "enum": ["established","emerging","experimental"] },
        "license": { "type": "string" }, "source": { "type": "string" } } } },
    "closest_match": { "type": ["string","null"], "description": "name of the competitor nearest the problem statement, or null" },
    "sources": { "type": "array", "items": { "type": "string" } } } }
```

- **invoked_at**: workflow step 3, alone.
- **registered_verifier**: `no`
- **report_field**: none

### 3. flow-drafter

- **name**: `flow-drafter`
- **owned_by**: Explore · `exploring-ideas`
- **derives_from**: `C:\Users\bryan\.claude\agents\requirements-analyst.md`
- **purpose**: Turn the brainstorm and the scan into 3 to 5 named flows with IDs, the non-goals that bound them, one success signal, and the fake rows a mockup needs.
- **tools**: `Read`
- **model**: `opus` — choosing which three flows carry the idea, and which capabilities to exclude, decides what Discover inherits.
- **input**: the full JSON from steps 2 and 3; on a remedy run, the current `## Core flows` rows, `## Non-goals` lines, `## Success signal` line, and the `findings[]` entries of `.devforgeai/reports/IDEA-nnn-discover.yaml` whose ids appear in `explore.remedy_flows`.
- **output**: One JSON object on stdout and nothing else. The schema the agent file fixes:

```json
{ "type": "object", "required": ["idea_id","flows","non_goals","success_signal","seed_data","unresolved_flow_ids"],
  "properties": {
    "idea_id": { "type": "string", "pattern": "^IDEA-[0-9]{3}$" },
    "flows": { "type": "array", "minItems": 1, "maxItems": 5, "items": { "type": "object",
      "required": ["flow_id","name","actor","trigger","steps","outcome"], "properties": {
        "flow_id": { "type": "string", "pattern": "^FLOW-[0-9]{3}$" },
        "name": { "type": "string" }, "actor": { "type": "string" }, "trigger": { "type": "string" },
        "steps": { "type": "array", "minItems": 2, "maxItems": 7, "items": { "type": "string" } },
        "outcome": { "type": "string" } } } },
    "non_goals": { "type": "array", "minItems": 1, "maxItems": 10, "items": { "type": "string" } },
    "success_signal": { "type": "object", "required": ["metric","threshold","window"],
      "properties": { "metric": { "type": "string" }, "threshold": { "type": "string" }, "window": { "type": "string" } } },
    "seed_data": { "type": "object", "required": ["entities"], "properties": {
      "entities": { "type": "array", "minItems": 1, "items": { "type": "object",
        "required": ["name","fields","rows"], "properties": {
          "name": { "type": "string" },
          "fields": { "type": "array", "minItems": 2, "items": { "type": "string" } },
          "rows": { "type": "array", "minItems": 5, "maxItems": 20, "items": { "type": "object" } } } } } } },
    "unresolved_flow_ids": { "type": "array", "items": { "type": "string", "pattern": "^FLOW-[0-9]{3}$" } } } }
```

- **invoked_at**: workflow step 4, alone.
- **registered_verifier**: `no`
- **report_field**: none

### 4. prototype-builder

- **name**: `prototype-builder`
- **owned_by**: Explore · `exploring-ideas`
- **derives_from**: `C:\Users\bryan\.claude\agents\frontend-developer.md`
- **purpose**: Turn the sketch screens into a clickable static directory seeded with the fake rows, so the flows can be walked before the decision.
- **tools**: `Read`, `Write`, `Glob`
- **model**: `sonnet` — the output is deleted at step 11, so throughput beats polish.
- **input**: `screens[]` from the sketch-mode output; `.devforgeai/explore/seed-data.json`; the `FLOW-nnn` ids to make clickable; the output root `.explore-prototype/`.
- **output**: One JSON object on stdout and nothing else. The schema the agent file fixes:

```json
{ "type": "object", "required": ["idea_id","built","entry","files","flows_covered","reason"],
  "properties": {
    "idea_id": { "type": "string", "pattern": "^IDEA-[0-9]{3}$" },
    "built": { "type": "boolean" },
    "entry": { "type": ["string","null"], "description": ".explore-prototype/index.html, or null when built is false" },
    "files": { "type": "array", "items": { "type": "string" } },
    "flows_covered": { "type": "array", "items": { "type": "string", "pattern": "^FLOW-[0-9]{3}$" } },
    "reason": { "type": ["string","null"], "description": "one line when built is false, null when built is true" } } }
```

- **invoked_at**: workflow step 7, alone, and only when the user answered `Yes` to the prototype offer.
- **registered_verifier**: `no`
- **report_field**: none

### 5. kill-case-builder

- **name**: `kill-case-builder`
- **owned_by**: Explore · `exploring-ideas`
- **derives_from**: `new`
- **purpose**: State the strongest available case for killing the idea, from the brief and the scan, so the user answers step 9 against the evidence rather than against the effort already spent.
- **tools**: `Read`
- **model**: `opus` — arguing against work the same session just produced is the one place in this phase where a weak model produces agreement instead of analysis.
- **input**: `.devforgeai/explore/brief.md`; the step 3 JSON; the `## Prototype` table row `Built`.
- **output**: One JSON object on stdout and nothing else. The schema the agent file fixes:

```json
{ "type": "object",
  "required": ["schema","subagent","id","passed","total","unit","findings","payload"],
  "properties": {
    "schema":   { "const": "devforgeai/verifier/1" },
    "subagent": { "const": "kill-case-builder" },
    "id":       { "type": "string", "pattern": "^IDEA-[0-9]{3}$" },
    "passed":   { "type": "integer", "minimum": 0 },
    "total":    { "type": "integer", "minimum": 0 },
    "unit":     { "const": "objections" },
    "findings": { "type": "array", "maxItems": 0 },
    "payload": { "type": "object",
      "required": ["idea_id","kill_case","strongest_objection","evidence","recommended_decision","confidence"],
      "properties": {
        "idea_id": { "type": "string", "pattern": "^IDEA-[0-9]{3}$" },
        "kill_case": { "type": "array", "minItems": 1, "maxItems": 5, "items": { "type": "string" } },
        "strongest_objection": { "type": "string", "maxLength": 200 },
        "evidence": { "type": "array", "items": { "type": "object", "required": ["claim","source","confidence"],
          "properties": { "claim": { "type": "string" },
            "source": { "type": "string", "description": "a brief section heading, a FLOW-nnn id, a URL from the scan, or the literal unsourced" },
            "confidence": { "type": "number", "minimum": 0, "maximum": 1 } } } },
        "recommended_decision": { "type": "string", "enum": ["kill","park","promote","unknown"] },
        "confidence": { "type": "number", "minimum": 0, "maximum": 1 } } } } }
```

- **invoked_at**: workflow step 8, alone, after the mockups and before the question.
- **registered_verifier**: `yes` — SubagentStop ingests it into `.devforgeai/reports/IDEA-nnn-explore.yaml`, which supplies the handoff `Verified` line.
- **report_field**: `verifiers.kill_case`; `phase = "explore"`, `unit = "objections"`, `required = true`

### 6. persona-mapper

- **name**: `persona-mapper`
- **owned_by**: Discover · `discovering-requirements`
- **derives_from**: `C:\Users\bryan\.claude\agents\stakeholder-analyst.md` — adapted. The influence tiers, the conflict matrix, and the interviews are dropped. The interviews could not have been kept: Claude Code removes `AskUserQuestion` from every subagent whatever its `tools` list holds, so a question belongs to the invoking skill, which asks it at steps 6 to 8 and 13. A persona in `requirements.yaml` carries a role, a description, and a goal, with no power ranking. Prose output is replaced by the JSON below.
- **purpose**: Turn flow statements or a typed description into candidate persona records.
- **tools**: `Read`
- **model**: `sonnet` — extraction from supplied text with a fixed output shape.
- **input**: the `## Target user` lines plus the `Actor` cell of every `## Core flows` row at entry point A, the description string at entry point B, the existing `personas[]` block at entry point C.
- **output**: One JSON object on stdout and nothing else. The schema the agent file fixes:

```json
{ "type": "object", "required": ["personas"], "additionalProperties": false,
  "properties": {
    "personas": { "type": "array", "minItems": 1, "items": { "type": "object",
      "required": ["key","name","description","goal","from"],
      "additionalProperties": false,
      "properties": {
        "key": { "type": "string", "pattern": "^[a-z0-9]+(-[a-z0-9]+)*$" },
        "name": { "type": "string", "minLength": 1, "maxLength": 40 },
        "description": { "type": "string", "minLength": 1, "maxLength": 200 },
        "goal": { "type": "string", "minLength": 1, "maxLength": 200 },
        "from": { "type": "array", "items": { "type": "string",
          "pattern": "^(FLOW-[0-9]{3}|user)$" } } } } } } }
```

- **invoked_at**: step 5, alone.
- **registered_verifier**: `no`
- **report_field**: none

### 7. requirement-drafter

- **name**: `requirement-drafter`
- **owned_by**: Discover · `discovering-requirements`
- **derives_from**: `C:\Users\bryan\.claude\agents\requirements-analyst.md` — adapted, and split with `epic-grouper`. Dropped: story files, Given/When/Then criteria, story points, INVEST, API contracts, data models, and the `Write` and `Edit` tools. The source agent's `AskUserQuestion` is not dropped so much as unavailable — no subagent has it — because stories belong to Plan (§5) and file writes belong to the skill. Kept: decomposition of a described capability into single-sentence, independently valuable units. Added: the `priority` and `source` fields and the JSON output.
- **purpose**: Turn confirmed personas, outcomes, exclusions, and source text into requirement records.
- **tools**: `Read`
- **model**: `opus` — decomposition boundaries and one-sentence compression carry the judgment this phase exists for.
- **input**: the confirmed persona keys, the outcome set, the exclusion set, and either the flow list, the description, or the reopened records plus the citing report defect lines; at entry point C, the list of `REQ-nnn` to redraft and the instruction that every other record stays as written.
- **output**: One JSON object on stdout and nothing else. The schema the agent file fixes:

```json
{ "type": "object", "required": ["requirements"], "additionalProperties": false,
  "properties": {
    "requirements": { "type": "array", "minItems": 1, "items": { "type": "object",
      "required": ["key","actor_key","statement","rationale","acceptance_signal",
                   "priority","source","open_questions"],
      "additionalProperties": false,
      "properties": {
        "key": { "type": "string", "pattern": "^([a-z0-9]+(-[a-z0-9]+)*|REQ-[0-9]{3})$" },
        "actor_key": { "type": "string",
          "pattern": "^([a-z0-9]+(-[a-z0-9]+)*|PERSONA-[0-9]{3})$" },
        "statement": { "type": "string", "minLength": 1, "maxLength": 200 },
        "rationale": { "type": "string", "minLength": 1, "maxLength": 200 },
        "acceptance_signal": { "type": "string", "minLength": 1, "maxLength": 200 },
        "priority": { "enum": ["must","should","could","wont"] },
        "source": { "type": "string", "pattern": "^(FLOW-[0-9]{3}|user)$" },
        "open_questions": { "type": "array",
          "items": { "type": "string", "maxLength": 200 } } } } } } }
```

- **invoked_at**: step 9, alone; re-entered from step 13 when the user answers `Redraft requirements`.
- **registered_verifier**: `no`
- **report_field**: none

### 8. epic-grouper

- **name**: `epic-grouper`
- **owned_by**: Discover · `discovering-requirements`
- **derives_from**: `C:\Users\bryan\.claude\agents\requirements-analyst.md` — adapted, the second half of the split. Kept: grouping features into epics with a stated boundary. Dropped: every story-level field and every write tool. Added: `out_of_scope`, `success_metric`, and the rule that each requirement ID appears in exactly one epic.
- **purpose**: Partition the requirement set into one epic per selected outcome, each with a scope, an exclusion list, and that outcome as its success metric.
- **tools**: `Read`
- **model**: `sonnet` — partitioning a supplied list against a supplied outcome set.
- **input**: the requirement records with allocated `REQ-nnn` IDs, the outcome set from step 7, and the exclusion set from step 8. The epic count equals the outcome count; the nth epic's `success_metric` is the nth selected outcome. Each selected exclusion is placed on exactly one epic, and an epic drawing none takes the single entry `Nothing was named out of scope at discovery.`
- **output**: One JSON object on stdout and nothing else. The schema the agent file fixes:

```json
{ "type": "object", "required": ["epics"], "additionalProperties": false,
  "properties": {
    "epics": { "type": "array", "minItems": 1, "items": { "type": "object",
      "required": ["key","title","scope","out_of_scope","success_metric","requirement_ids"],
      "additionalProperties": false,
      "properties": {
        "key": { "type": "string", "pattern": "^([a-z0-9]+(-[a-z0-9]+)*|EPIC-[0-9]{3})$" },
        "title": { "type": "string", "minLength": 1, "maxLength": 60 },
        "scope": { "type": "string", "minLength": 1, "maxLength": 200 },
        "out_of_scope": { "type": "array", "minItems": 1,
          "items": { "type": "string", "maxLength": 200 } },
        "success_metric": { "type": "string", "minLength": 1, "maxLength": 200 },
        "requirement_ids": { "type": "array", "minItems": 1,
          "items": { "type": "string", "pattern": "^REQ-[0-9]{3}$" } } } } } } }
```

- **invoked_at**: step 11, alone, entry points A and B; re-entered from step 13 when the user answers `Regroup epics`. Entry point C invokes it in placement mode only when `revision_log[].added` is non-empty; in that mode it returns one existing `EPIC-nnn` per added `REQ-nnn` and emits no other field.
- **registered_verifier**: `no`
- **report_field**: none

### 9. flow-integrity-auditor

- **name**: `flow-integrity-auditor`
- **owned_by**: Discover · `discovering-requirements`
- **derives_from**: `C:\Users\bryan\.claude\agents\context-preservation-validator.md` — adapted. Dropped: the provenance-chain walk and the `<provenance>` tag presence check, both of which `devforgeai doc validate` performs as cross-reference checking (§4), and the strict/non-blocking mode switch, which §6 replaces with SEND BACK. Kept: the read-only posture and the finding-with-evidence output. Added: the two judgments over flow prose that no field check reaches — a statement that names no actor, and a pair of statements that cannot both hold.
- **purpose**: Report `## Core flows` rows whose `Actor` cell names no persona and pairs of rows whose `Trigger`, `Steps`, or `Outcome` cells cannot both hold.
- **tools**: `Read`
- **model**: `opus` — deciding that two prose sentences cannot both hold is the judgment this subagent exists for.
- **input**: every `## Core flows` row as `{id, actor, trigger, steps, outcome}`, the brief path `.devforgeai/explore/brief.md`, and, on the resume form, the `remedied_flows` list from `decision.yaml`.
- **output**: One JSON object on stdout and nothing else. The schema the agent file fixes:

```json
{ "type": "object",
  "required": ["schema","subagent","id","passed","total","unit","findings","payload"],
  "properties": {
    "schema":   { "const": "devforgeai/verifier/1" },
    "subagent": { "const": "flow-integrity-auditor" },
    "id":       { "type": "string", "pattern": "^IDEA-[0-9]{3}$" },
    "passed":   { "type": "integer", "minimum": 0 },
    "total":    { "type": "integer", "minimum": 0 },
    "unit":     { "const": "flows" },
    "findings": { "type": "array", "maxItems": 0 },
    "payload": { "type": "object",
      "required": ["flows_checked","flows_clean","actorless","contradictions"],
      "additionalProperties": false,
      "properties": {
        "flows_checked": { "type": "integer", "minimum": 0 },
        "flows_clean": { "type": "integer", "minimum": 0 },
        "actorless": { "type": "array", "items": { "type": "object",
          "required": ["flow_id","reason","confidence","evidence"], "additionalProperties": false,
          "properties": {
            "flow_id": { "type": "string", "pattern": "^FLOW-[0-9]{3}$" },
            "reason": { "enum": ["empty","not_a_persona","unlisted_role"] },
            "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
            "evidence": { "type": "string", "maxLength": 200 } } } },
        "contradictions": { "type": "array", "items": { "type": "object",
          "required": ["flow_ids","cells","confidence","evidence"], "additionalProperties": false,
          "properties": {
            "flow_ids": { "type": "array", "minItems": 2, "maxItems": 2,
              "items": { "type": "string", "pattern": "^FLOW-[0-9]{3}$" } },
            "cells": { "type": "array", "minItems": 1,
              "items": { "enum": ["trigger","steps","outcome"] } },
            "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
            "evidence": { "type": "string", "maxLength": 200 } } } } } } } }
```

- **invoked_at**: step 4, alone, entry point A and the resume form.
- **registered_verifier**: `yes` — `SubagentStop` ingests it, and `flows_clean`/`flows_checked` fills the `Verified` line of the handoff.
- **report_field**: `verifiers.flow_integrity`; `phase = "discover"`, `unit = "flows"`, `required = false`. Assigned by `## Decisions` 3, because `specs/03-discover.md` declares the registration without writing the `[[verifier]]` table.

### 10. architecture-reviewer

- **name**: architecture-reviewer
- **owned_by**: Constitute · `establishing-context`
- **derives_from**: `C:\Users\bryan\.claude\agents\architect-reviewer.md`
- **purpose**: Decide whether the accepted stack and constraint set can realize every REQ, and whether any two REQs force opposed constraints.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `opus` — the finding turns on reasoning across a requirement set and a constraint set at once.
- **input**: the six context file paths, the ADR directory path, the `requirements[]` records as `{id, actor, statement, rationale, acceptance_signal, priority, status}` with `status: withdrawn` records omitted, the `epics[]` records as `{id, scope, out_of_scope, success_metric, requirements}`, and on the remedy path a single CON id to scope to.
- **output**: One JSON object on stdout and nothing else. The agent file fixes the shape by worked example; this is the first of them:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "architecture-reviewer",
  "id": "IDEA-004",
  "passed": 22,
  "total": 23,
  "unit": "requirements",
  "findings": [
    { "id": "REQ-014", "severity": "block", "confidence": 0.85,
      "kind": "infeasible",
      "requirements": ["REQ-014"],
      "constraints": ["CON-002"],
      "adrs": [],
      "statement": "no architecture under the accepted stack meets 50ms p99 while CON-002 holds",
      "summary": "no architecture meets 50ms p99 under CON-002",
      "evidence": ".devforgeai/context/tech-stack.md:31" }
  ],
  "payload": {
    "requirements_reviewed": 23,
    "send_back_requirements": ["REQ-014"],
    "blocking_findings": 1
  }
}
```

  `kind` values: `infeasible` — no architecture under the accepted stack realizes the named REQ; `contradiction` — the named REQs force opposed constraints; `unsupported` — an ADR decision no REQ motivates; `overbuilt` — a CON stricter than any REQ asks. `payload.send_back_requirements` holds the union of `requirements` over findings of kind `infeasible` and `contradiction`. `payload.blocking_findings` counts findings at `severity: block`. `report ingest architecture-reviewer -` files this block under `verifiers.architecture_reviewer` in `.devforgeai/reports/<IDEA-nnn>-constitute.yaml`, so the gate reads `verifiers.architecture_reviewer.payload.blocking_findings` and `verifiers.architecture_reviewer.payload.send_back_requirements.length`.
- **invoked_at**: step 11, in parallel with `alignment-auditor`; R3 and R5 on the remedy path.
- **registered_verifier**: yes
- **report_field**: `verifiers.architecture_reviewer`; `phase = "constitute"`, `unit = "requirements"`, `required = false`. Assigned by `## Decisions` 3. The Constitute gate reads the block through two `report_metric` checks and no `verifier_pass` check names it, which is the condition `config.toml` attaches to `required = true`.

### 11. alignment-auditor

- **name**: alignment-auditor
- **owned_by**: Constitute · `establishing-context`
- **derives_from**: `C:\Users\bryan\.claude\agents\alignment-auditor.md`
- **purpose**: Find semantic disagreement across the six files and the ADR log that exact matching cannot see.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `sonnet` — paraphrase comparison over a bounded file set.
- **input**: the six context file paths, the ADR directory path, the `## Constraint index` rows, the `## Anti-pattern index` rows.
- **output**: One JSON object on stdout and nothing else. The agent file fixes the shape by worked example; this is the first of them:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "alignment-auditor",
  "id": "IDEA-004",
  "passed": 31,
  "total": 31,
  "unit": "checks",
  "findings": [
    { "id": "CON-007", "severity": "warn", "confidence": 0.8,
      "kind": "restated",
      "left": {"file": ".devforgeai/context/source-tree.md", "line": 44, "text": "quoted line"},
      "right": {"file": ".devforgeai/context/coding-standards.md", "line": 12, "text": "quoted line"},
      "ids": ["CON-007"],
      "resolution": "source-tree.md ## Naming conventions keeps the rule",
      "summary": "the layering rule is worded twice and the wordings differ",
      "evidence": ".devforgeai/context/source-tree.md:44" }
  ],
  "payload": { "checks_run": 31, "blocking_findings": 0 }
}
```

  `kind` values: `restated` — one rule worded two ways in two files, where a future edit to one leaves the other stale; `contradicted` — two files assert opposed rules that share no key and so escape CA-5; `unobservable` — a CON whose `enforced_by` names an AP whose detector does not observe the CON's statement, or whose `enforced_by` is `none`; `unpropagated` — an accepted ADR decision absent from the file its `## Constraints introduced` row points at; `orphaned` — an AP whose `source` CON has `status: retired`. `resolution` names a mutable target: a context file section or a new ADR, and not an existing ADR body.
- **invoked_at**: step 11, in parallel with `architecture-reviewer`; R5 on the remedy path.
- **registered_verifier**: yes
- **report_field**: `verifiers.alignment_auditor`; `phase = "constitute"`, `unit = "checks"`, `required = false`. Assigned by `## Decisions` 3. The Constitute gate reads the block through one `report_metric` check and no `verifier_pass` check names it.

### 12. source-tree-mapper

- **name**: source-tree-mapper
- **owned_by**: Constitute · `establishing-context`
- **derives_from**: `C:\Users\bryan\.claude\agents\code-analyzer.md`
- **purpose**: Read an existing codebase and report the roots, layers, entry points, internal edges, and declared dependencies that the brownfield drafts leave open.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `sonnet` — file-structure enumeration with fixed output shape.
- **input**: `source.root` from the draft `source-tree.md`, the layer names the draft proposes, the manifest paths from the draft `dependencies.md`.
- **output**: One JSON object on stdout and nothing else. The agent file fixes the shape by worked example; this is the first of them:

```json
{
  "schema": "devforgeai/source-tree-mapper/1",
  "roots": {"source": "path", "test": "path", "build_output": "path"},
  "layers": [{"name": "identifier", "path_glob": "glob", "file_count": 0, "purpose": "one sentence"}],
  "entry_points": ["path"],
  "internal_edges": [{"from": "identifier", "to": "identifier", "count": 0, "examples": ["path:line"]}],
  "external_dependencies": [{"name": "identifier", "version": "string", "scope": "runtime | dev | test | build", "declared_in": "path"}],
  "generated_paths": ["glob"],
  "unmapped_paths": ["glob"]
}
```

- **invoked_at**: B5, alone.
- **registered_verifier**: no
- **report_field**: none

### 13. story-decomposer

- **name**: `story-decomposer`
- **owned_by**: Plan · `planning-work`
- **derives_from**: `C:\Users\bryan\.claude\agents\story-requirements-analyst.md`
- **purpose**: Split one epic's accepted requirements into story drafts, each with acceptance criteria in Given/When/Then form, a layer, a constraint set, and a dependency list.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `opus` — the split decides how much a single Build run carries, and a wrong split propagates through every later phase.
- **input**: the `epics[]` entry (`EPIC-nnn`, `title`, `scope`, `out_of_scope`, `success_metric`); the `requirements[]` records it names (`REQ-nnn`, `actor`, `statement`, `rationale`, `acceptance_signal`, `priority`, `traces_to`); the `Layer` value set from `source-tree.md` `## Layers`; the active rows of `architecture-constraints.md` `## Constraint index` (`CON-nnn`, `Kind`, `Title`); the `## Layer dependency rules` rows; the rows of `anti-patterns.md` `## Anti-pattern index` (`AP-nnn`, `Severity`, `Scope`); the `[plan].story_points` set.
- **output**: One JSON object on stdout and nothing else. The agent file fixes the shape by worked example; this is the first of them:

```json
{
  "schema": "devforgeai/story-decomposer/1",
  "epic": "EPIC-001",
  "drafts": [
    {
      "key": "d1",
      "req_ids": ["REQ-007"],
      "story_lines": { "as_a": "Shopper", "i_want": "…", "so_that": "…" },
      "acceptance_criteria": [
        { "key": "a1", "req_id": "REQ-007", "given": "…", "when": "…", "then": "…" }
      ],
      "layer": "application",
      "con_ids": ["CON-003"],
      "ap_ids": ["AP-002"],
      "ui_ids": [],
      "needs_screen": false,
      "depends_on": ["d0"],
      "points": 3,
      "out_of_scope": ["…"]
    }
  ],
  "uncovered_reqs": ["REQ-012"],
  "notes": []
}
```

  `key` is a run-local handle the skill maps to an allocated `STORY-nnn` at step 7; `depends_on` holds `key` values, not ids, because the ids are unallocated when the subagent runs. `points` is a member of `[plan].story_points`. `needs_screen` is `true` when `layer` is `interface` and every `req_ids` entry carries an empty `traces_to`.
- **invoked_at**: workflow step 5, alone.
- **registered_verifier**: no.
- **report_field**: none

### 14. story-file-set-planner

- **name**: `story-file-set-planner`
- **owned_by**: Plan · `planning-work`
- **derives_from**: `C:\Users\bryan\.claude\agents\api-designer.md` — replaced, not adapted. The existing agent designs REST, GraphQL, and gRPC contracts and writes structured YAML API components; that output is subsumed by the story's `## Files` table and by the `Then` clause of an acceptance criterion, and Build owns the implementation. This subagent keeps the "fix the contract before code is written" idea and applies it to file paths.
- **purpose**: Assign each story draft a disjoint set of repo-relative paths, drawn from the source tree's placement rules, that the Build phase writes.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `sonnet` — the task is pattern application against two tables, and the overlap result is checked mechanically by `devforgeai story validate`.
- **input**: the `drafts[]` array from `story-decomposer`; `source-tree.md` `## Roots`, `## Directory map`, `## File placement rules` (`Artifact kind`, `Path pattern`, `Example`), `## Naming conventions`, `## Generated and excluded paths`; `coding-standards.md` `## Testing standards` rows (`Rule`, `Scope`); the existing repo tree under the `source.root` and `test.root` values.
- **output**: One JSON object on stdout and nothing else. The agent file fixes the shape by worked example; this is the first of them:

```json
{
  "schema": "devforgeai/story-file-set-planner/1",
  "sets": [
    { "key": "d1",
      "files": [ { "path": "src/application/checkout/place_order.ext", "kind": "source", "layer": "application" },
                 { "path": "tests/application/checkout/place_order_test.ext", "kind": "test", "layer": "application" } ] }
  ],
  "overlaps": [ { "path": "src/shared/clock.ext", "keys": ["d1", "d4"] } ],
  "unplaceable": [ { "key": "d7", "reason": "no File placement rules row matches an interface asset" } ]
}
```

  `kind` is one of `source`, `test`, `config`, `migration`, `asset`. A non-empty `overlaps` or `unplaceable` list sends the draft set back through one further invocation with those entries appended to the prompt.
- **invoked_at**: workflow step 7, alone, after step 5.
- **registered_verifier**: no.
- **report_field**: none

### 15. story-invest-auditor

- **name**: `story-invest-auditor`
- **owned_by**: Plan · `planning-work`
- **derives_from**: `C:\Users\bryan\.claude\agents\requirements-analyst.md` — adapted. The existing agent both writes stories and judges them, and holds Write and Edit. This one judges only and is read-only, because `story-decomposer` writes and §1.1 keeps the mechanical half of INVEST in `devforgeai story validate`. The existing agent's INVEST list is narrowed to the three judgments no parser makes: independence, size, and the wording of a `Then` clause.
- **purpose**: Judge each written story on independence, size, and the measurability of every `Then` clause, and report a requirement that admits two readings or a constraint the context set does not state.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `opus` — independence and measurability are judgments about meaning, and the finding routes the phase to a send-back.
- **input**: the path of every `.devforgeai/stories/STORY-nnn.md` written this run; the `epics[]` entry; the `requirements[]` records (`REQ-nnn`, `statement`, `acceptance_signal`); the active rows of `architecture-constraints.md` `## Constraint index` and the `## Layer dependency rules` rows.
- **output**: One JSON object on stdout and nothing else: the `devforgeai/verifier/1` envelope, with this agent's own top-level fields under `payload`. It adds none, so `payload` is `{}`. The agent file fixes the shape by worked example; this is the first of them:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "story-invest-auditor",
  "id": "SPRINT-001",
  "passed": 6,
  "total": 8,
  "unit": "stories",
  "findings": [
    { "id": "REQ-011", "severity": "block", "confidence": 0.85,
      "stands_against": ["STORY-104", "STORY-109"],
      "summary": "\"recent orders\" fixes no window and no count",
      "evidence": "requirements.yaml REQ-011 statement; STORY-104 AC-019 and AC-020 read it two ways" },
    { "id": "STORY-107", "severity": "warn", "confidence": 0.7,
      "stands_against": ["STORY-107"],
      "summary": "8 points and two layers in one story",
      "evidence": "STORY-107 ## Files rows 1-4 layer domain, rows 5-9 layer interface" }
  ],
  "payload": {}
}
```

  `severity` is the closed enum `block`, `warn`, `info` that `specs/01-cli.md` fixes, and `confidence` rides on every finding. A finding's `id` is the upstream id it cites — a `REQ-nnn` for an ambiguous requirement, a `CON-nnn` for a missing or contradictory constraint — so `devforgeai handoff` renders the `Found` line and composes the `--remedy` list from `findings[].id` without a further field. A judgment about a story carries that story's `STORY-nnn` as its `id` and `severity` of `warn`. `passed` counts stories against which no `block` finding stands; `total` is the story count.
- **invoked_at**: workflow step 9, and once more at step 10 over the edited set.
- **registered_verifier**: yes. The `config.toml` entry is in `specs/05-plan.md` `## CLI calls`.
- **report_field**: `verifiers.story_invest`; `phase = "plan"`, `unit = "stories"`, `required = true`

### 16. sprint-sequencer

- **name**: `sprint-sequencer`
- **owned_by**: Plan · `planning-work`
- **derives_from**: `C:\Users\bryan\.claude\agents\sprint-planner.md` — adapted. The existing agent writes the sprint file, edits story frontmatter, and appends workflow history; those writes move to workflow step 11, because a subagent that writes documents defeats the PostToolUse validation the skill's own writes pass through. Its capacity arithmetic and story selection are kept, its point thresholds move to `config.toml` `[plan].sprint_capacity_points`, and its dependency handling absorbs the topological ordering that `dependency-graph-analyzer` performed.
- **purpose**: Order the epic's stories by dependency and fill one sprint to the configured point capacity.
- **tools**: `Read`, `Glob`, `Grep`
- **model**: `sonnet` — the work is a topological sort and a running sum against a stated bound.
- **input**: for each story, its `STORY-nnn`, its `points`, and its `## Dependencies` id list; `points_max` and whether it came from `config.toml` or the default.
- **output**: One JSON object on stdout and nothing else. The agent file fixes the shape by worked example; this is the first of them:

```json
{
  "schema": "devforgeai/sprint-sequencer/1",
  "epic": "EPIC-001",
  "capacity": { "points_max": 20, "points_planned": 17, "source": "default" },
  "stories": [ { "id": "STORY-101", "order": 1, "points": 3 } ],
  "deferred": [ { "id": "STORY-109", "reason": "capacity" } ],
  "longest_chain": ["STORY-101", "STORY-103", "STORY-107"],
  "cycle": []
}
```

  `reason` is the closed enum `capacity`, `dependency`. `longest_chain` is the longest dependency chain in the sprint, reported so the skill can report it and used by no check. A non-empty `cycle` holds the id path of one cycle and stops workflow step 10.
- **invoked_at**: workflow step 11, alone.
- **registered_verifier**: no.
- **report_field**: none

### 17. spec-gap-triager

- **name**: `spec-gap-triager`
- **owned_by**: Plan · `planning-work`
- **derives_from**: `new`
- **purpose**: Decide, for one acceptance criterion a downstream phase sent back, whether the criterion is rewritten here, or the defect belongs to Discover, Constitute, or Design.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `opus` — the decision routes the send-back and a wrong route costs a full phase.
- **input**: one triple per cited id — `story_id`, `ac_id`, and the finding `summary` and `evidence` from the build or QA report; the `REQ-nnn` whose `Covered by` cell holds the criterion, with its `statement` and `acceptance_signal`; the active `CON-nnn` rows whose `Binds` value matches the story's `## Layer` or a `Path` in its `## Files`; the `UI-nnn` ids in the story's `## Interface`.
- **output**: One JSON object on stdout and nothing else. The agent file fixes the shape by worked example; this is the first of them:

```json
{
  "schema": "devforgeai/spec-gap-triager/1",
  "decisions": [
    { "story_id": "STORY-104", "ac_id": "AC-019",
      "disposition": "rewrite_ac",
      "cited_id": "AC-019",
      "rationale": "The Then clause names a state and no value a test reads.",
      "replacement": "Given a shopper with 3 saved orders When the list loads Then the response holds 3 rows ordered by placed_at descending." }
  ]
}
```

  `disposition` is the closed enum `rewrite_ac`, `send_back_discover`, `send_back_constitute`, `send_to_design`. `cited_id` is the `AC-nnn` for `rewrite_ac`, the `REQ-nnn` for `send_back_discover`, the `CON-nnn` for `send_back_constitute`, and the `UI-nnn` for `send_to_design`. `replacement` is the rewritten criterion text for `rewrite_ac` and `""` for the other three.
- **invoked_at**: remedy workflow step R2, alone. It runs in no `full` and no `resume` run.
- **registered_verifier**: no. Its output routes the run; `story-invest-auditor` supplies the report block.
- **report_field**: none

### 18. ac-test-writer

- **name**: `ac-test-writer`
- **owned_by**: Build · `implementing-stories`
- **derives_from**: `C:\Users\bryan\.claude\agents\test-automator.md`
- **purpose**: Turn one acceptance criterion into one failing test, or report that the criterion cannot be turned into one.
- **tools**: `Read`, `Write`, `Edit`, `Grep`, `Glob`
- **model**: `opus` — its `testable` and `conflicts_with` verdicts produce the send-back to Plan, and a wrong verdict either hides a specification defect or returns a good story.
- **input**: the `AC-nnn` id and its full `Given … When … Then …` line; the other `AC-nnn` lines of the story; the `## Files` rows of `Kind` `test` as `{path, layer}`; the `## Testing standards` rows of `coding-standards.md`; the `## Naming conventions` rows of `source-tree.md`; the current content of each declared test file; on a remedy run the `FIND-nnn` summary and evidence.
- **output**: One JSON object on stdout and nothing else. The schema the agent file fixes:

```json
{
  "type": "object",
  "required": ["schema", "subagent", "id", "passed", "total", "unit", "findings", "payload"],
  "properties": {
    "schema":   { "const": "devforgeai/verifier/1" },
    "subagent": { "const": "ac-test-writer" },
    "id":       { "type": "string", "pattern": "^STORY-[0-9]{3}$" },
    "passed":   { "type": "integer", "enum": [0, 1] },
    "total":    { "type": "integer", "const": 1 },
    "unit":     { "const": "AC" },
    "findings": { "type": "array", "items": {
      "type": "object",
      "required": ["id", "severity", "confidence", "summary", "evidence"],
      "properties": {
        "id":       { "type": "string", "pattern": "^AC-[0-9]{3}$" },
        "severity": { "enum": ["block", "warn", "info"] },
        "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
        "reason":   { "enum": ["then_names_no_readable_outcome", "when_names_no_action",
                               "given_names_no_reachable_state", "contradicts_other_ac",
                               "outcome_outside_declared_files"] },
        "summary":  { "type": "string", "maxLength": 120 },
        "evidence": { "type": "string", "maxLength": 300 }
      } } },
    "payload": {
      "type": "object",
      "required": ["testable", "ac", "test_paths"],
      "properties": {
        "testable": { "type": "boolean" },
        "ac":       { "type": "string", "pattern": "^AC-[0-9]{3}$" },
        "test_paths": { "type": "array", "items": { "type": "string" } },
        "assertion":  { "type": "string", "maxLength": 200 },
        "conflicts_with": { "type": "array", "items": { "type": "string", "pattern": "^AC-[0-9]{3}$" } }
      } }
  }
}
```

  `passed` is 1 with `testable` true and 0 otherwise. `findings` is `[]` when `testable` is true. A `contradicts_other_ac` reason emits one finding per id in `conflicts_with` plus one for `ac`.
- **invoked_at**: workflow step 7.1, once per criterion, serially. Nothing runs in parallel with it.
- **registered_verifier**: yes, `verifiers.ac_testable`.
- **report_field**: `verifiers.ac_testable`; `phase = "build"`, `unit = "AC"`, `required = true`

### 19. backend-implementer

- **name**: `backend-implementer`
- **owned_by**: Build · `implementing-stories`
- **derives_from**: `C:\Users\bryan\.claude\agents\backend-architect.md`
- **purpose**: Write the smallest code inside the declared file set that makes one named failing test pass without breaking the tests already green.
- **tools**: `Read`, `Write`, `Edit`, `Grep`, `Glob`
- **model**: `opus` — it places code into a layer under `## Layer dependency rules` and the `CON-nnn` set, and a layering error costs the gate two checks and a rewrite.
- **input**: the `AC-nnn` line; the failing test paths and their content; the failing test output; the `## Files` rows of `Kind` `source`, `config`, `migration`, and `asset` as `{path, layer}`; the `## Layer` value; the `CON-nnn` rows of `## Constraints` as `{id, statement, binds}`; the `AP-nnn` rows of `## Anti-patterns` as `{id, severity, scope}`; the `## Approved dependencies` and `## Forbidden dependencies` tables; the `## Error handling`, `## Logging`, and `## Formatting` sections; the source paths earlier cycles wrote.
- **output**: One JSON object on stdout and nothing else. The schema the agent file fixes:

```json
{
  "type": "object",
  "required": ["subagent", "ac", "status", "source_paths", "notes"],
  "properties": {
    "subagent": { "const": "backend-implementer" },
    "ac":       { "type": "string", "pattern": "^AC-[0-9]{3}$" },
    "status":   { "enum": ["implemented", "blocked"] },
    "source_paths": { "type": "array", "items": { "type": "string" }, "minItems": 0 },
    "blocked":  { "type": "object",
      "required": ["reason", "detail"],
      "properties": {
        "reason": { "enum": ["no_declared_path_fits", "constraint_forbids_the_only_shape",
                             "dependency_not_approved", "test_asserts_outside_declared_files"] },
        "detail": { "type": "string", "maxLength": 300 },
        "ids":    { "type": "array", "items": { "type": "string", "pattern": "^(CON|AP)-[0-9]{3}$" } }
      } },
    "notes": { "type": "array", "items": { "type": "string", "maxLength": 160 }, "maxItems": 5 }
  }
}
```

  `blocked` is present when `status` is `blocked` and absent otherwise.
- **invoked_at**: workflow step 7.4, once per criterion, serially, and once more at step 7.5 on a failing rerun.
- **registered_verifier**: no.
- **report_field**: none

### 20. frontend-implementer

- **name**: `frontend-implementer`
- **owned_by**: Build · `implementing-stories`
- **derives_from**: `C:\Users\bryan\.claude\agents\frontend-developer.md`
- **purpose**: Write the component code inside the declared file set that makes one named failing test pass and renders the screen the `UI-nnn` spec describes.
- **tools**: `Read`, `Write`, `Edit`, `Grep`, `Glob`
- **model**: `sonnet` — the `UI-nnn` tables fix the anatomy, the states, the four breakpoints, the interactions, and the eight accessibility rows, `tokens.json` fixes every colour and type value, and the `PreToolUse` `design lint` hook blocks a literal that resolves to no token, so the remaining choice is transcription.
- **input**: the `AC-nnn` line; the failing test paths, their content, and the failing output; the `## Files` rows of `Kind` `source` and `asset`; the `UI-nnn` `## Anatomy`, `## States`, `## Breakpoints`, `## Interaction`, `## Accessibility`, and `## Tokens used` tables; the `tokens.json` leaf names per group; the `## Formatting` and `## Naming` sections.
- **output**: One JSON object on stdout and nothing else. This agent is not a registered verifier, so the object carries no `devforgeai/verifier/1` envelope and no `SubagentStop` ingest reads it. The schema the agent file fixes — `backend-implementer`'s, with `subagent` set to this name, two further required properties, and a `blocked.reason` enum widened by two values:

```json
{
  "type": "object",
  "required": ["subagent", "ac", "status", "source_paths", "notes", "ui", "tokens_used"],
  "properties": {
    "subagent": { "const": "frontend-implementer" },
    "ac":       { "type": "string", "pattern": "^AC-[0-9]{3}$" },
    "status":   { "enum": ["implemented", "blocked"] },
    "source_paths": { "type": "array", "items": { "type": "string" }, "minItems": 0 },
    "ui": { "type": "string", "pattern": "^UI-[0-9]{3}$|^$" },
    "tokens_used": { "type": "array", "items": { "type": "string", "pattern": "^TOKEN-[a-z]+-[a-z0-9-]+$" } },
    "blocked":  { "type": "object",
      "required": ["reason", "detail"],
      "properties": {
        "reason": { "enum": ["no_declared_path_fits", "constraint_forbids_the_only_shape",
                             "dependency_not_approved", "test_asserts_outside_declared_files",
                             "token_missing_for_required_value", "ui_spec_omits_a_state_the_test_asserts"] },
        "detail": { "type": "string", "maxLength": 300 },
        "ids":    { "type": "array", "items": { "type": "string", "pattern": "^(CON|AP)-[0-9]{3}$" } }
      } },
    "notes": { "type": "array", "items": { "type": "string", "maxLength": 160 }, "maxItems": 5 }
  }
}
```

  `blocked` is present when `status` is `blocked` and absent otherwise. `ui` carries the screen id, or `""` for a criterion the story cites no screen for. The entry carries the whole object rather than the two-property fragment it once did: a fragment plus a sentence pointing at another entry is a schema a generator cannot read.
- **invoked_at**: workflow step 7.4, once per criterion, serially, in place of `backend-implementer`.
- **registered_verifier**: no.
- **report_field**: none

### 21. refactor-surgeon

- **name**: `refactor-surgeon`
- **owned_by**: Build · `implementing-stories`
- **derives_from**: `C:\Users\bryan\.claude\agents\refactoring-specialist.md`
- **purpose**: Rewrite the paths a failing lint, complexity, or anti-pattern result names, leaving every test green.
- **tools**: `Read`, `Write`, `Edit`, `Grep`, `Glob`
- **model**: `sonnet` — the trigger, the target paths, and the two numbers come from the CLI, so the subagent applies a rewrite to a named location rather than deciding whether one is warranted.
- **input**: the failing check ids from the closed set `build-lint`, `build-complexity`, `build-antipatterns`; each check's `reason` string; the `AP-nnn` match list of `devforgeai antipattern scan --json` when the trigger is `build-antipatterns`; the paths the cycle wrote; `[build].complexity_max` and `[build].duplication_max_percent`; the `## Formatting` and `## Naming` sections.
- **output**: One JSON object on stdout and nothing else. The schema the agent file fixes:

```json
{
  "type": "object",
  "required": ["subagent", "trigger", "status", "paths", "changes"],
  "properties": {
    "subagent": { "const": "refactor-surgeon" },
    "trigger":  { "enum": ["build-lint", "build-complexity", "build-antipatterns"] },
    "status":   { "enum": ["rewritten", "declined"] },
    "paths":    { "type": "array", "items": { "type": "string" } },
    "changes":  { "type": "array", "items": {
      "type": "object",
      "required": ["path", "pattern", "before", "after"],
      "properties": {
        "path":    { "type": "string" },
        "pattern": { "enum": ["extract-function", "extract-type", "inline", "rename",
                              "replace-conditional", "move-to-layer", "remove-duplicate"] },
        "before":  { "type": "string", "maxLength": 160 },
        "after":   { "type": "string", "maxLength": 160 }
      } } },
    "declined": { "type": "object",
      "required": ["reason", "detail"],
      "properties": {
        "reason": { "enum": ["rewrite_leaves_declared_file_set", "constraint_forbids_the_rewrite",
                             "threshold_breach_is_in_a_file_the_story_does_not_declare"] },
        "detail": { "type": "string", "maxLength": 300 } } }
  }
}
```

- **invoked_at**: workflow step 7.8, zero or one time per criterion, and workflow step 10 on an anti-pattern match. Serially.
- **registered_verifier**: no.
- **report_field**: none

### 22. integration-test-writer

- **name**: `integration-test-writer`
- **owned_by**: Build · `implementing-stories`
- **derives_from**: `C:\Users\bryan\.claude\agents\integration-tester.md`
- **purpose**: Write tests that exercise the story's criteria across the layer boundary the story crosses, rather than one unit at a time.
- **tools**: `Read`, `Write`, `Edit`, `Grep`, `Glob`
- **model**: `sonnet` — the contract is already written down in the `UI-nnn` `## Interaction` table and the `## Layer dependency rules` rows, and the unit tests of step 7 fix the shapes it composes.
- **input**: every `AC-nnn` line; the `## Files` rows of `Kind` `test`; the source paths of every cycle and their content; the `UI-nnn` `## Interaction` and `## States` tables when the story cites one; the `## Layer dependency rules` rows; the `## Testing standards` rows.
- **output**: One JSON object on stdout and nothing else. The schema the agent file fixes:

```json
{
  "type": "object",
  "required": ["subagent", "status", "test_paths", "scenarios"],
  "properties": {
    "subagent": { "const": "integration-test-writer" },
    "status":   { "enum": ["written", "not-applicable"] },
    "test_paths": { "type": "array", "items": { "type": "string" } },
    "scenarios": { "type": "array", "items": {
      "type": "object",
      "required": ["name", "acs", "boundary"],
      "properties": {
        "name": { "type": "string", "maxLength": 120 },
        "acs":  { "type": "array", "items": { "type": "string", "pattern": "^AC-[0-9]{3}$" }, "minItems": 1 },
        "boundary": { "enum": ["interface-application", "application-infrastructure",
                               "application-domain", "interface-external"] }
      } } },
    "not_applicable_reason": { "enum": ["story_crosses_no_layer_boundary", "no_declared_test_path_for_the_boundary"] }
  }
}
```

- **invoked_at**: workflow step 8, zero or one time, after the loop of step 7 ends.
- **registered_verifier**: no.
- **report_field**: none

### 23. story-ac-verifier

- **name**: `story-ac-verifier`
- **owned_by**: Build · `implementing-stories`
- **derives_from**: `C:\Users\bryan\.claude\agents\ac-compliance-verifier.md`
- **purpose**: Decide each `AC-nnn` of the story from the story text, the diff, and the test output alone.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `opus` — its ratio is the `build-acs` gate check, and a false pass ships an unimplemented criterion into Verify.
- **input**: three prompt fields and no others — the absolute path of `.devforgeai/stories/STORY-nnn.md`, the `data` object of `devforgeai story files --diff --id <STORY-nnn> --json` holding one entry per changed path, and the merged stdout and stderr of the last test command run. The prompt carries no part of the conversation, no subagent output from steps 7.1 through 10, and no summary of them.
- **output**: One JSON object on stdout and nothing else. The schema the agent file fixes:

```json
{
  "type": "object",
  "required": ["schema", "subagent", "id", "passed", "total", "unit", "findings", "payload"],
  "properties": {
    "schema":   { "const": "devforgeai/verifier/1" },
    "subagent": { "const": "story-ac-verifier" },
    "id":       { "type": "string", "pattern": "^STORY-[0-9]{3}$" },
    "passed":   { "type": "integer", "minimum": 0 },
    "total":    { "type": "integer", "minimum": 1 },
    "unit":     { "const": "ACs" },
    "findings": { "type": "array", "items": {
      "type": "object",
      "required": ["id", "severity", "confidence", "summary", "evidence"],
      "properties": {
        "id":       { "type": "string", "pattern": "^AC-[0-9]{3}$" },
        "severity": { "enum": ["block", "warn", "info"] },
        "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
        "reason":   { "enum": ["no_test_asserts_the_then_clause", "test_asserts_a_weaker_outcome",
                               "no_diff_hunk_implements_it", "test_output_shows_it_skipped"] },
        "summary":  { "type": "string", "maxLength": 120 },
        "evidence": { "type": "string", "maxLength": 300 }
      } } },
    "payload": {
      "type": "object",
      "required": ["checks"],
      "properties": {
        "checks": { "type": "array", "items": {
          "type": "object",
          "required": ["ac", "verdict", "confidence", "test_path", "source_path", "evidence"],
          "properties": {
            "ac":      { "type": "string", "pattern": "^AC-[0-9]{3}$" },
            "verdict": { "enum": ["met", "unmet"] },
            "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
            "test_path":   { "type": "string" },
            "source_path": { "type": "string" },
            "evidence":    { "type": "string", "maxLength": 300 }
          } } } } }
  }
}
```

  `total` equals the number of `AC-nnn` lines in the story, `checks` holds one entry per id, and `passed` equals the count of `verdict: PASS`. Each `FAIL` verdict emits one finding with the same `ac` as its `id`.
- **invoked_at**: workflow step 11, once, after steps 8, 9, and 10 have ended.
- **registered_verifier**: yes, `verifiers.story_ac`.
- **report_field**: `verifiers.story_ac`; `phase = "build"`, `unit = "ACs"`, `required = true`

### 24. context-validator

- **name**: `context-validator`
- **owned_by**: Build · `implementing-stories`
- **derives_from**: `C:\Users\bryan\.claude\agents\context-validator.md`
- **purpose**: Decide whether each changed file obeys the six context files on the points no CLI check covers.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `opus` — layer-boundary and coding-standard judgments over unfamiliar code are its whole output, and the ratio is the `build-context` gate check.
- **input**: the `data` object of `devforgeai story files --diff --id <STORY-nnn> --json`; the six context file paths; the `CON-nnn` rows of the story's `## Constraints`; the story's `## Layer` value.
- **output**: One JSON object on stdout and nothing else. The schema the agent file fixes:

```json
{
  "type": "object",
  "required": ["schema", "subagent", "id", "passed", "total", "unit", "findings", "payload"],
  "properties": {
    "schema":   { "const": "devforgeai/verifier/1" },
    "subagent": { "const": "context-validator" },
    "id":       { "type": "string", "pattern": "^STORY-[0-9]{3}$" },
    "passed":   { "type": "integer", "minimum": 0 },
    "total":    { "type": "integer", "minimum": 0 },
    "unit":     { "const": "files" },
    "payload":  { "type": "object" },
    "findings": { "type": "array", "items": {
      "type": "object",
      "required": ["id", "severity", "confidence", "summary", "evidence"],
      "properties": {
        "id":       { "type": "string", "pattern": "^(CON|AP)-[0-9]{3}$" },
        "severity": { "enum": ["block", "warn", "info"] },
        "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
        "kind":     { "enum": ["layer_boundary_crossed", "library_substituted", "dependency_unapproved",
                               "naming_convention_broken", "error_handling_shape_broken",
                               "logging_shape_broken", "file_outside_placement_rule"] },
        "path":     { "type": "string" },
        "line":     { "type": "integer", "minimum": 1 },
        "summary":  { "type": "string", "maxLength": 120 },
        "evidence": { "type": "string", "maxLength": 300 }
      } } }
  }
}
```

  `total` equals the number of changed paths and `passed` the number carrying no `severity: block` finding.
- **invoked_at**: workflow step 9, once, after step 8 ends and before step 11.
- **registered_verifier**: yes, `verifiers.context`.
- **report_field**: `verifiers.context`; `phase = "build"`, `unit = "files"`, `required = true`

Verify's ten entries share one preamble, carried from `specs/07-verify.md` `## Subagents`: each is a registered verifier, each read-only, each emitting one `devforgeai/verifier/1` object. The `findings[]` entries of that object carry the contract's four fields, `id`, `severity`, `summary`, `evidence`, plus the four this phase adds, `category`, `file`, `line`, `relates_to`, which `report ingest` copies through unread.

### 25. ac-compliance-verifier

- **name**: `ac-compliance-verifier`
- **owned_by**: Verify · `validating-quality`
- **derives_from**: `C:\Users\bryan\.claude\agents\ac-compliance-verifier.md`
- **purpose**: Decide, without having written the code, whether each `AC-nnn` of the story is met by a test that reads the outcome the criterion names.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `opus` — the judgement is whether a test asserts the named outcome, which is a reading of two texts against each other.
- **input**: the story path, the `AC-nnn` list with full text, the `## Files` rows of `Kind` `source` and `test`, the `## Out of scope` lines, its id band.
- **output**: One JSON object on stdout and nothing else. The agent file fixes the shape by worked example; this is the first of them:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "ac-compliance-verifier",
  "id": "STORY-014",
  "passed": 7,
  "total": 7,
  "unit": "ACs",
  "findings": [
    { "id": "FIND-001", "severity": "block", "confidence": 0.9, "category": "ac-compliance", "file": "tests/checkout.ext",
      "line": 0, "relates_to": "AC-007", "summary": "no test reads the rejected-payment outcome",
      "evidence": "tests/checkout.ext holds no case asserting the rejected state named by AC-007" }
  ],
  "payload": {}
}
```

`total` is the number of `AC-nnn` in the story; `passed` is the number with no `block` finding. `category` is `ac-compliance`, or `spec-gap` when the code meets the criterion and the criterion names less than its `REQ-nnn` states.
- **invoked_at**: workflow step 6, in parallel with six others.
- **registered_verifier**: yes
- **report_field**: `verifiers.ac_compliance`; `phase = "verify"`, `unit = "ACs"`, `required = true`. This is the one entry `specs/01-cli.md` `## Outputs` already carries in the default `config.toml`.

### 26. standards-reviewer

- **name**: `standards-reviewer`
- **owned_by**: Verify · `validating-quality`
- **derives_from**: `C:\Users\bryan\.claude\agents\code-reviewer.md`
- **purpose**: Review the story's changed source files against the rules of `context/coding-standards.md`.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `opus` — the judgement is whether a rule written in prose is met by code, which no pattern decides.
- **input**: the `## Files` rows of `Kind` `source`, the seven `coding-standards.md` sections, the `CON-nnn` rows that `enforced_by` names for each, its id band.
- **output**: One JSON object on stdout and nothing else. The agent file fixes the shape by worked example; this is the first of them:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "standards-reviewer",
  "id": "STORY-014",
  "passed": 5,
  "total": 6,
  "unit": "files",
  "findings": [
    { "id": "FIND-101", "severity": "block", "confidence": 0.9, "category": "standards", "file": "src/application/checkout.ext",
      "line": 44, "relates_to": "CON-009", "summary": "the error path returns a bare value",
      "evidence": "src/application/checkout.ext:44 returns on failure without the error type coding-standards.md names" }
  ],
  "payload": {}
}
```

`total` is the number of `Kind` `source` rows; `passed` is the number with no `block` finding.
- **invoked_at**: workflow step 6, in parallel.
- **registered_verifier**: yes
- **report_field**: `verifiers.standards`; `phase = "verify"`, `unit = "files"`, `required = true`

### 27. anti-pattern-scanner

- **name**: `anti-pattern-scanner`
- **owned_by**: Verify · `validating-quality`
- **derives_from**: `C:\Users\bryan\.claude\agents\anti-pattern-scanner.md`
- **purpose**: Match each `AP-nnn` detector against the story's file set and report every hit with its severity.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `sonnet` — the detector and its kind come from the index, so the work is matching and citing rather than deciding.
- **input**: the `## Anti-pattern index` rows of `AP`, `Category`, `Severity`, `Scope`, `Detector kind`, `Detector`, `Source`; the `## Files` rows whose `Path` the `Scope` glob matches; its id band.
- **output**: One JSON object on stdout and nothing else. The agent file fixes the shape by worked example; this is the first of them:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "anti-pattern-scanner",
  "id": "STORY-014",
  "passed": 4,
  "total": 6,
  "unit": "anti-patterns",
  "findings": [
    { "id": "FIND-201", "severity": "block", "confidence": 0.9, "category": "anti-pattern", "file": "src/application/checkout.ext",
      "line": 118, "relates_to": "AP-002", "summary": "the application layer opens the order store directly",
      "evidence": "src/application/checkout.ext:118 matches the AP-002 regex detector" }
  ],
  "payload": {}
}
```

`severity` is `block` when the `## Anti-pattern index` `Severity` cell reads `blocker`, and `warn` for `high`, `medium`, and `low`. `total` is the number of `AP-nnn` rows in scope; `passed` is the number with no hit of `blocker` severity.
- **invoked_at**: workflow step 6, in parallel.
- **registered_verifier**: yes
- **report_field**: `verifiers.anti_patterns`; `phase = "verify"`, `unit = "anti-patterns"`, `required = true`

### 28. constraint-auditor

- **name**: `constraint-auditor`
- **owned_by**: Verify · `validating-quality`
- **derives_from**: `C:\Users\bryan\.claude\agents\context-validator.md`
- **purpose**: Decide whether the story's file set satisfies each `CON-nnn` its `## Constraints` table binds, and whether the constraint set decides the case at all.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `opus` — a constraint is a sentence, and whether code satisfies it is a reading.
- **input**: the story's `## Constraints` rows, the matching `### CON-nnn` blocks with `kind`, `status`, `statement`, `source`, `introduced_by`, `enforced_by`; the `## Layer dependency rules` row for the story's `## Layer`; the `## Files` rows; its id band.
- **output**: One JSON object on stdout and nothing else. The agent file fixes the shape by worked example; this is the first of them:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "constraint-auditor",
  "id": "STORY-014",
  "passed": 3,
  "total": 4,
  "unit": "constraints",
  "findings": [
    { "id": "FIND-301", "severity": "warn", "confidence": 0.7, "category": "constraint", "file": "src/application/checkout.ext",
      "line": 91, "relates_to": "CON-003", "summary": "a second write path reaches the order store",
      "evidence": "src/application/checkout.ext:91 writes the store outside the single path CON-003 states" }
  ],
  "payload": {}
}
```

`category` is `constraint`, or `spec-gap` when the constraint holds and an `AC-nnn` asserts less than the constraint requires. `total` is the number of `CON-nnn` rows; `passed` is the number with no `block` finding.
- **invoked_at**: workflow step 6, in parallel.
- **registered_verifier**: yes
- **report_field**: `verifiers.constraints`; `phase = "verify"`, `unit = "constraints"`, `required = true`

### 29. coverage-gap-auditor

- **name**: `coverage-gap-auditor`
- **owned_by**: Verify · `validating-quality`
- **derives_from**: `C:\Users\bryan\.claude\agents\coverage-analyzer.md`
- **purpose**: Read the build report's layer figures and decide which uncovered region leaves an `AC-nnn` untested.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `sonnet` — the numbers arrive measured, and the judgement is which uncovered file relates to which criterion.
- **input**: the build report `coverage` block, the story's `## Layer`, the `## Files` rows, the `AC-nnn` list, its id band.
- **output**: One JSON object on stdout and nothing else. The agent file fixes the shape by worked example; this is the first of them:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "coverage-gap-auditor",
  "id": "STORY-014",
  "passed": 3,
  "total": 4,
  "unit": "layers",
  "findings": [
    { "id": "FIND-401", "severity": "warn", "confidence": 0.7, "category": "coverage", "file": "src/domain/order.ext",
      "line": 0, "relates_to": "AC-004", "summary": "the rejection branch is uncovered and AC-004 asserts it",
      "evidence": "build report coverage.layers domain 96.6 with src/domain/order.ext uncovered lines 61-74" }
  ],
  "payload": {}
}
```

This subagent runs no command. `total` is the number of `coverage.layers[]` entries; `passed` is the number whose `status` the build report records as `pass`. A layer below its floor is a `block` finding.
- **invoked_at**: workflow step 6, in parallel.
- **registered_verifier**: yes
- **report_field**: `verifiers.coverage_gaps`; `phase = "verify"`, `unit = "layers"`, `required = true`

### 30. dead-code-detector

- **name**: `dead-code-detector`
- **owned_by**: Verify · `validating-quality`
- **derives_from**: `C:\Users\bryan\.claude\agents\dead-code-detector.md`
- **purpose**: Report symbols defined in the story's file set that no other file in the project references.
- **tools**: `Read`, `Grep`, `Glob`, `Bash`
- **model**: `sonnet` — reference counting with a confidence judgement on dynamic dispatch.
- **input**: the `## Files` rows of `Kind` `source`, the `## Roots` and `## Generated and excluded paths` entries of `source-tree.md`, the value of `config.toml` `[verify].call_graph_command`, its id band.
- **output**: One JSON object on stdout and nothing else. The agent file fixes the shape by worked example; this is the first of them:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "dead-code-detector",
  "id": "STORY-014",
  "passed": 12,
  "total": 12,
  "unit": "symbols",
  "findings": [
    { "id": "FIND-501", "severity": "warn", "confidence": 0.6, "category": "dead-code", "file": "src/domain/order.ext",
      "line": 203, "relates_to": "AP-004",
      "summary": "the symbol is defined and referenced by no file outside its own",
      "evidence": "grep over source roots returns one hit, the definition at src/domain/order.ext:203" }
  ],
  "payload": { "method": "grep", "dead": 1 }
}
```

`method` is the closed enum `command` and `grep`. With `[verify].call_graph_command` non-empty the subagent runs it through `Bash` and sets `method: command`; with the key `""` it resolves each symbol with Grep over the `## Roots` paths and sets `method: grep`. `confidence` is a float from `0.0` to `1.0` and is present on every finding: `1.0` when `method` is `command`, and `0.6` when `method` is `grep` and the definition is exported, `0.9` when it is not. A dead-code finding is `warn` and no higher.
- **invoked_at**: workflow step 6, in parallel.
- **registered_verifier**: yes
- **report_field**: `verifiers.dead_code`; `phase = "verify"`, `unit = "symbols"`, `required = true`

### 31. deferral-validator

- **name**: `deferral-validator`
- **owned_by**: Verify · `validating-quality`
- **derives_from**: `C:\Users\bryan\.claude\agents\deferral-validator.md`
- **purpose**: Decide whether each deferral names a real target, a reason its evidence supports, and a chain that does not return to this story.
- **tools**: `Read`, `Glob`, `Grep`
- **model**: `opus` — whether a stated reason matches the evidence is a reading, not a lookup.
- **input**: the drafted `deferrals[]` entries, the `deferrals[]` entries of every `reports/STORY-*-qa.yaml` on disk, the `sprint.yaml` `stories[]` list with `status`, the `adr/ADR-nnn.md` frontmatter `status` values, the story's `## Out of scope` lines, its id band.
- **output**: One JSON object on stdout and nothing else. The agent file fixes the shape by worked example; this is the first of them:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "deferral-validator",
  "id": "STORY-014",
  "passed": 1,
  "total": 2,
  "unit": "deferrals",
  "findings": [
    { "id": "FIND-601", "severity": "block", "confidence": 0.9, "category": "deferral", "file": "", "line": 0,
      "relates_to": "CON-005", "summary": "STORY-031 defers the same item back to STORY-014",
      "evidence": "reports/STORY-031-qa.yaml deferrals[0].target is STORY-014 for CON-005" }
  ],
  "payload": {}
}
```

`total` is the number of deferral entries examined; `passed` is the number with a resolving target, a supported reason, and no return edge. A circular chain, an unresolved target, and a reason the evidence contradicts are each `block`.
- **invoked_at**: workflow step 6 over the deferrals on disk, and again at step 9 over the deferrals this run drafts.
- **registered_verifier**: yes
- **report_field**: `verifiers.deferrals`; `phase = "verify"`, `unit = "deferrals"`, `required = true`. It shares this string with `deferral-auditor` at `phase = "release"`; the two write into different report files and the registry keys on `name`.

### 32. security-auditor

- **name**: `security-auditor`
- **owned_by**: Verify · `validating-quality`
- **derives_from**: `C:\Users\bryan\.claude\agents\security-auditor.md`
- **purpose**: Examine the story's file set against the ten OWASP Top 10 categories and report each hit against the constraint that governs it.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `opus` — the judgement is whether a code path is reachable and exploitable, which a pattern alone does not decide.
- **input**: the `## Files` rows of `Kind` `source` and `config`, the `CON-nnn` rows of `kind: security`, the `AP-nnn` rows of `Category` `security`, the `## Forbidden dependencies` rows of `dependencies.md`, its id band.
- **output**: One JSON object on stdout and nothing else. The agent file fixes the shape by worked example; this is the first of them:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "security-auditor",
  "id": "STORY-014",
  "passed": 9,
  "total": 10,
  "unit": "OWASP categories",
  "findings": [
    { "id": "FIND-701", "severity": "block", "confidence": 0.9, "category": "security", "file": "src/api/orders.ext",
      "line": 57, "relates_to": "CON-011", "owasp": "A01",
      "summary": "the handler reads the order id from the request and applies no ownership check",
      "evidence": "src/api/orders.ext:57 loads by id with no comparison against the session subject" }
  ],
  "payload": {}
}
```

`owasp` is a closed enum of ten values: `A01` Broken Access Control, `A02` Cryptographic Failures, `A03` Injection, `A04` Insecure Design, `A05` Security Misconfiguration, `A06` Vulnerable and Outdated Components, `A07` Identification and Authentication Failures, `A08` Software and Data Integrity Failures, `A09` Security Logging and Monitoring Failures, `A10` Server-Side Request Forgery. `total` is `10`; `passed` is the number of categories with no `block` finding. This subagent runs no command and reads no network.
- **invoked_at**: workflow step 7, deep mode, in parallel with two others.
- **registered_verifier**: yes
- **report_field**: `verifiers.security`; `phase = "verify"`, `unit = "OWASP categories"`, `required = true`

### 33. code-quality-auditor

- **name**: `code-quality-auditor`
- **owned_by**: Verify · `validating-quality`
- **derives_from**: `C:\Users\bryan\.claude\agents\code-quality-auditor.md`
- **purpose**: Report functions above the complexity ceiling and duplicated runs above the duplication ceiling, both from `config.toml`.
- **tools**: `Read`, `Grep`, `Glob`, `Bash`
- **model**: `sonnet` — the thresholds are numbers and the work is measuring against them.
- **input**: the `## Files` rows of `Kind` `source`, `config.toml` `[verify].complexity_max`, `[verify].duplication_max_percent`, `[verify].duplication_min_lines`, `[verify].metrics_command`, its id band.
- **output**: One JSON object on stdout and nothing else. The agent file fixes the shape by worked example; this is the first of them:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "code-quality-auditor",
  "id": "STORY-014",
  "passed": 6,
  "total": 6,
  "unit": "files",
  "findings": [
    { "id": "FIND-801", "severity": "warn", "confidence": 0.7, "category": "complexity", "file": "src/application/checkout.ext",
      "line": 22, "relates_to": "CON-009", "measured": 14, "limit": 10,
      "summary": "one function branches 14 ways against a ceiling of 10",
      "evidence": "src/application/checkout.ext:22-118 holds 13 branch keywords on one function body" }
  ],
  "payload": { "method": "grep", "over_ceiling": 1 }
}
```

`method` is the closed enum `command` and `grep`. With `[verify].metrics_command` non-empty the subagent runs it through `Bash` and reads its `devforgeai-metrics/1` JSON, setting `method: command`; with the key `""` it counts branch keywords and repeated line runs with Grep and sets `method: grep`. `measured` and `limit` are numbers present on every `complexity` and `duplication` finding. `total` is the number of `Kind` `source` rows; `passed` is the number under both ceilings.
- **invoked_at**: workflow step 7, deep mode, in parallel.
- **registered_verifier**: yes
- **report_field**: `verifiers.quality`; `phase = "verify"`, `unit = "files"`, `required = true`

### 34. adr-conformance-reviewer

- **name**: `adr-conformance-reviewer`
- **owned_by**: Verify · `validating-quality`
- **derives_from**: `C:\Users\bryan\.claude\agents\architect-reviewer.md`
- **purpose**: Decide whether the story's implementation conforms to each accepted ADR whose constraints its file set touches.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `opus` — a decision record is prose and conformance to it is a reading.
- **input**: every `adr/ADR-nnn.md` at `status: accepted` with `## Decision`, `## Consequences`, `## Constraints introduced`; the story's `## Files`, `## Layer`, `## Constraints`; the `## Layer dependency rules` table; its id band.
- **output**: One JSON object on stdout and nothing else. The agent file fixes the shape by worked example; this is the first of them:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "adr-conformance-reviewer",
  "id": "STORY-014",
  "passed": 2,
  "total": 3,
  "unit": "ADRs",
  "findings": [
    { "id": "FIND-901", "severity": "warn", "confidence": 0.7, "category": "constraint", "file": "src/infrastructure/store.ext",
      "line": 12, "relates_to": "ADR-002", "summary": "the adapter carries the decision logic ADR-002 places in the application layer",
      "evidence": "src/infrastructure/store.ext:12-40 branches on order state, which ADR-002 assigns to application" }
  ],
  "payload": {}
}
```

`total` is the number of accepted ADRs whose `## Constraints introduced` names a `CON-nnn` the story's `## Constraints` also names; `passed` is the number with no `block` finding. `WebFetch` leaves the tool list, because a registered verifier runs unattended under `SubagentStop` and reads no network.
- **invoked_at**: workflow step 7, deep mode, in parallel.
- **registered_verifier**: yes
- **report_field**: `verifiers.adr_conformance`; `phase = "verify"`, `unit = "ADRs"`, `required = true`

### 35. mockup-designer

- **name**: `mockup-designer`
- **owned_by**: Design · `designing-interfaces`
- **derives_from**: `C:\Users\bryan\.claude\agents\frontend-developer.md`
- **purpose**: Draw one wireframe screen per step of a flow, as a static HTML file, carrying the seed rows the flow moves.
- **tools**: `Read`, `Write`, `Glob`, `Skill`
- **model**: `sonnet` — layout from a named flow and a fixed screen budget, with the built-in `design` skill supplying the visual judgment.
- **input**: `flows[]`, `out_dir`, `seed_data_path`, `constraints`, and `brand` from `.devforgeai/explore/sketch-request.json`; the run's `IDEA-nnn`.
- **output**: One JSON object on stdout and nothing else. The schema the agent file fixes:

```json
{ "type": "object", "required": ["idea_id","screens","brand_sketch","uncovered_flows","notes"],
  "properties": {
    "idea_id": { "type": "string", "pattern": "^IDEA-[0-9]{3}$" },
    "screens": { "type": "array", "items": { "type": "object",
      "required": ["flow_id","screen","path","title","state"], "properties": {
        "flow_id": { "type": "string", "pattern": "^FLOW-[0-9]{3}$" },
        "screen": { "type": "string", "pattern": "^FLOW-[0-9]{3}-[0-9]{2}$" },
        "path": { "type": "string" },
        "title": { "type": "string", "maxLength": 60 },
        "state": { "type": "string", "enum": ["default","empty","error"] } } } },
    "brand_sketch": { "type": ["object","null"], "required": ["path","name","palette","type_pair"],
      "properties": {
        "path": { "type": "string" },
        "name": { "type": "string", "maxLength": 40 },
        "palette": { "type": "array", "minItems": 3, "maxItems": 6,
          "items": { "type": "string", "pattern": "^#[0-9a-f]{6}$" } },
        "type_pair": { "type": "array", "minItems": 2, "maxItems": 2, "items": { "type": "string" } } } },
    "uncovered_flows": { "type": "array", "items": { "type": "string", "pattern": "^FLOW-[0-9]{3}$" } },
    "notes": { "type": "array", "maxItems": 5, "items": { "type": "string", "maxLength": 160 } } } }
```

- **invoked_at**: workflow steps 3 and 4, alone.
- **registered_verifier**: `no`
- **report_field**: none

### 36. brand-designer

- **name**: `brand-designer`
- **owned_by**: Design · `designing-interfaces`
- **derives_from**: `new`
- **purpose**: Turn five answers and a candidate palette into a complete token set whose text and border contrast ratios clear WCAG AA in both themes, and one SVG mark.
- **tools**: `Read`, `Write`, `Skill`
- **model**: `opus` — a palette that reads as one brand in two themes and still clears contrast in both is the judgment this mode rests on.
- **input**: the five answers of step 6; the candidate name, palette, and type pair of step 5; the `personas[].goal` strings of `requirements.yaml`; the answer-to-leaf mapping table.
- **output**: One JSON object on stdout and nothing else. The schema the agent file fixes:

```json
{ "type": "object", "required": ["brand_name","tokens_written","logo_written","contrast","adjustments","reason"],
  "properties": {
    "brand_name": { "type": "string", "maxLength": 40 },
    "tokens_written": { "type": "boolean" },
    "logo_written": { "type": "boolean" },
    "contrast": { "type": "array", "items": { "type": "object",
      "required": ["token","theme","ratio"], "properties": {
        "token": { "type": "string", "pattern": "^TOKEN-color-[a-z][a-z0-9-]*$" },
        "theme": { "type": "string", "enum": ["light","dark"] },
        "ratio": { "type": "number", "minimum": 1.0, "maximum": 21.0 } } } },
    "adjustments": { "type": "array", "maxItems": 6, "items": { "type": "object",
      "required": ["token","from","to","reason"], "properties": {
        "token": { "type": "string", "pattern": "^TOKEN-color-[a-z][a-z0-9-]*$" },
        "from": { "type": "string" }, "to": { "type": "string" },
        "reason": { "type": "string", "maxLength": 160 } } } },
    "reason": { "type": ["string","null"], "maxLength": 200 } } }
```

- **invoked_at**: workflow steps 7 and 8, in that order, alone.
- **registered_verifier**: `no`
- **report_field**: none

### 37. requirement-coverage-auditor

- **name**: `requirement-coverage-auditor`
- **owned_by**: Design · `designing-interfaces`
- **derives_from**: `new`
- **purpose**: Name every screen the story asks for that realizes no requirement, and every flow in the brief that no requirement sources, so the send-back cites ids rather than impressions.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `sonnet` — reference resolution across two documents against a stated rule, with no interview and no writes.
- **input**: the screen list of step 10, the `requirements[]` and `epics[]` arrays of `requirements.yaml`, the `## Core flows` table of `.devforgeai/explore/brief.md` when present, and the run's `STORY-nnn` or `EPIC-nnn`. A screen is covered when at least one `REQ-nnn` in the story's `consumes` names it in `acceptance_signal` or in an `AC-nnn` row. A flow is covered when its `FLOW-nnn` equals some `requirements[].source`.
- **output**: One JSON object on stdout and nothing else. The schema the agent file fixes:

```json
{ "type": "object",
  "required": ["schema","subagent","id","passed","total","unit","findings","payload"],
  "properties": {
    "schema":   { "const": "devforgeai/verifier/1" },
    "subagent": { "const": "requirement-coverage-auditor" },
    "id":       { "type": "string", "pattern": "^(STORY|EPIC)-[0-9]{3}$" },
    "passed":   { "type": "integer", "minimum": 0 },
    "total":    { "type": "integer", "minimum": 0 },
    "unit":     { "const": "screens" },
    "findings": { "type": "array", "items": { "type": "object",
      "required": ["id","severity","confidence","summary","evidence"], "properties": {
        "id": { "type": "string", "maxLength": 60 },
        "severity": { "enum": ["block","warn","info"] },
        "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
        "summary": { "type": "string", "maxLength": 120 },
        "evidence": { "type": "string", "maxLength": 200 } } } },
    "payload": { "type": "object",
      "required": ["subject_id","screens","screens_without_req","flows_without_req","covered"],
      "properties": {
        "subject_id": { "type": "string", "pattern": "^(STORY|EPIC)-[0-9]{3}$" },
        "screens": { "type": "array", "minItems": 0, "items": { "type": "object",
          "required": ["name","kind","requirements"], "properties": {
            "name": { "type": "string", "maxLength": 60 },
            "kind": { "type": "string", "enum": ["screen","component"] },
            "requirements": { "type": "array", "items": { "type": "string", "pattern": "^REQ-[0-9]{3}$" } } } } },
        "screens_without_req": { "type": "array", "items": { "type": "object",
          "required": ["name","evidence"], "properties": {
            "name": { "type": "string", "maxLength": 60 },
            "evidence": { "type": "string", "maxLength": 200 } } } },
        "flows_without_req": { "type": "array", "items": { "type": "object",
          "required": ["flow_id","evidence"], "properties": {
            "flow_id": { "type": "string", "pattern": "^FLOW-[0-9]{3}$" },
            "evidence": { "type": "string", "maxLength": 200 } } } },
        "covered": { "type": "integer", "minimum": 0 } } } } }
```

- **invoked_at**: workflow step 11, alone, before any `UI-nnn` is allocated.
- **registered_verifier**: `yes` — `SubagentStop` ingests it, and `covered`/`total` fills the `Verified` line of the handoff.
- **report_field**: `verifiers.requirement_coverage`; `phase = "design"`, `unit = "screens"`, `required = false`. Assigned by `## Decisions` 3. `required` is `false` because `specs/08-design.md` Decision 3 leaves `gate check --phase design` undefined, so no `verifier_pass` check names it; the block is read by `handoff --phase design` and by Reflect.

### 38. ui-spec-writer

- **name**: `ui-spec-writer`
- **owned_by**: Design · `designing-interfaces`
- **derives_from**: `C:\Users\bryan\.claude\agents\ui-spec-formatter.md`
- **purpose**: Write one `UI-nnn.md` for one screen, with the nine sections filled and every colour and type value written as a `TOKEN-<name>`.
- **tools**: `Read`, `Write`, `Grep`, `Glob`, `Skill`
- **model**: `opus` — the states, the keyboard path, and the eight accessibility rows are the content a story is built from, and a thin one costs a Verify finding.
- **input**: one screen object from step 11 with its allocated `UI-nnn`; the `REQ-nnn` records it realizes; the `PERSONA-nnn` goal; the flattened token names of `.devforgeai/brand/tokens.json`; the mockup file paths under `.devforgeai/explore/mockups/` matching the screen's flow; the story's Figma node URL when the story carries one; on a remedy run, the current file and the cited `AC-nnn`, `TOKEN-<name>`, and `UI-nnn` ids.
- **output**: One JSON object on stdout and nothing else. The schema the agent file fixes:

```json
{ "type": "object", "required": ["ui_id","path","sections","tokens_used","states","unresolved_ids"],
  "properties": {
    "ui_id": { "type": "string", "pattern": "^UI-[0-9]{3}$" },
    "path": { "type": "string", "pattern": "^\\.devforgeai/ui-specs/UI-[0-9]{3}\\.md$" },
    "sections": { "type": "array", "minItems": 9, "maxItems": 9, "items": { "type": "string" } },
    "tokens_used": { "type": "array", "items": { "type": "string", "pattern": "^TOKEN-[a-z][a-z0-9-]*-[a-z][a-z0-9-]*$" } },
    "states": { "type": "array", "minItems": 1, "items": { "type": "string",
      "enum": ["default","loading","empty","partial","error","success","disabled","read-only"] } },
    "unresolved_ids": { "type": "array", "items": { "type": "string" } } } }
```

- **invoked_at**: workflow step 13, one invocation per screen, in parallel across screens; workflow step 15, one invocation for the cited `UI-nnn`.
- **registered_verifier**: `no`
- **report_field**: none

### 39. deferral-auditor

- **name**: `deferral-auditor`
- **owned_by**: Release · `releasing-software`
- **derives_from**: `C:\Users\bryan\.claude\agents\deferral-validator.md`
- **purpose**: Decide, per deferred `FIND-nnn`, whether the deferral blocks this release's deployment.
- **tools**: `Read`, `Glob`, `Grep`
- **model**: `opus` — the judgment is whether a stated reason survives contact with the target platform, which is the one place this phase reasons rather than assembles.
- **input**: the release version; `platform.target`; one record per story in the set holding `story` (`STORY-nnn`), `qa_report` (path), the `FIND-nnn` ids that report marks deferred with their stated reason, and the story's AC texts; the `## Approved dependencies` rows of `.devforgeai/context/dependencies.md`.
- **output**: One JSON object on stdout and nothing else. The agent file fixes the shape by worked example; this is the first of them:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "deferral-auditor",
  "unit": "deferrals",
  "passed": 4,
  "total": 5,
  "findings": [
    {
      "id": "FIND-009",
      "severity": "block",
      "confidence": 0.85,
      "story": "STORY-017",
      "kind": "blocks_deployment",
      "summary": "Refund webhook signature check deferred with no replacement control",
      "reason": "Deferred to STORY-031",
      "evidence": ".devforgeai/reports/STORY-017-qa.yaml:41"
    }
  ],
  "payload": {}
}
```

  `kind` is a closed enum: `blocks_deployment`, `unjustified`, `circular`, `missing_report`, `accepted`. `severity` is `block` for the first four and `info` for `accepted`. `passed` counts the `accepted` entries; `total` counts every deferred `FIND-nnn` across the set. A story with no deferral contributes nothing to either count.
- **invoked_at**: workflow step 6, once for the whole release set, before the platform is resolved on a `fresh` run.
- **registered_verifier**: `yes` — `config.toml` carries the `[[verifier]]` table in `specs/09-release.md` `## Gate`, and `SubagentStop` ingests the block into `.devforgeai/reports/vX.Y.Z-release.yaml` at `verifiers.deferrals`.
- **report_field**: `verifiers.deferrals`; `phase = "release"`, `unit = "deferrals"`, `required = true`

### 40. deploy-manifest-writer

- **name**: `deploy-manifest-writer`
- **owned_by**: Release · `releasing-software`
- **derives_from**: `C:\Users\bryan\.claude\agents\deployment-engineer.md`
- **purpose**: Write the manifest set for one platform target from the release's own values.
- **tools**: `Read`, `Write`, `Glob`
- **model**: `sonnet` — the shape is fixed by the templates and the work is substitution plus the constraint rows.
- **input**: `platform.target`; `[release].image_name`, `[release].deploy_root`, `[release].build_command`, `[release].package_command`, `[release].service_port`; the `[[stack]].id` and `source_roots` values; the `## Constraints` rows of `.devforgeai/context/architecture-constraints.md`; the version string; the template paths for the target.
- **output**: One JSON object on stdout and nothing else. The agent file fixes the shape by worked example; this is the first of them:

```json
{
  "schema": "devforgeai/deploy-manifest/1",
  "target": "kubernetes",
  "written": [
    { "path": "deploy/kubernetes/deployment.yaml", "kind": "workload" },
    { "path": "deploy/kubernetes/service.yaml", "kind": "network" },
    { "path": "deploy/kubernetes/ingress.yaml", "kind": "network" },
    { "path": "deploy/kubernetes/kustomization.yaml", "kind": "overlay" },
    { "path": "deploy/kubernetes/ROLLBACK.md", "kind": "script" }
  ],
  "secrets_referenced": ["APP_DATABASE_URL", "APP_SIGNING_KEY"],
  "constraints_applied": ["CON-012"],
  "reason": ""
}
```

  `kind` is the closed enum of `deploy.manifests[].kind` in `specs/09-release.md` `## Outputs`. `secrets_referenced` lists the environment variable names the manifests read; values are excluded from the list. `reason` is `""` on success and a one-sentence string when `written` is `[]`.
- **invoked_at**: workflow step 8, once, after the platform is resolved and before the docs are written; not invoked when `platform.target` is `none`.
- **registered_verifier**: `no` — it writes files and reports no pass ratio; the `deploy_manifest` gate check reads the files it wrote.
- **report_field**: none

### 41. api-doc-writer

- **name**: `api-doc-writer`
- **owned_by**: Release · `releasing-software`
- **derives_from**: `C:\Users\bryan\.claude\agents\documentation-writer.md`
- **purpose**: Write one API page per stack and source root, with one H3 per public symbol the CLI enumerated.
- **tools**: `Read`, `Write`, `Glob`, `Grep`
- **model**: `sonnet` — the symbol list arrives fixed and the work is locating each symbol and describing it.
- **input**: the symbol lines of `[release].api_symbols_command`, each `<kind>\t<symbol>\t<path>`; the `[[stack]].id` and `source_roots` pairs; `[release].docs_root`; the version string.
- **output**: One JSON object on stdout and nothing else. The agent file fixes the shape by worked example; this is the first of them:

```json
{
  "schema": "devforgeai/api-docs/1",
  "pages": [
    { "path": "docs/api/index.md", "stack": "", "root": "", "symbols": 0 },
    { "path": "docs/api/primary-src.md", "stack": "primary", "root": "src", "symbols": 142 }
  ],
  "symbols_total": 142,
  "symbols_documented": 142,
  "symbols_missing": [],
  "reason": ""
}
```

  `symbols_missing` lists the symbol names the agent found no source for and therefore wrote no H3 for. `reason` is `""` on success.
- **invoked_at**: workflow step 10, in parallel with `guide-writer`.
- **registered_verifier**: `no` — the `docs_cover` gate check recomputes the ratio from the files rather than trusting the agent's count.
- **report_field**: none

### 42. guide-writer

- **name**: `guide-writer`
- **owned_by**: Release · `releasing-software`
- **derives_from**: `C:\Users\bryan\.claude\agents\documentation-writer.md`
- **purpose**: Write the user guide from the release's ACs and the architecture note from the accepted ADRs and the context files.
- **tools**: `Read`, `Write`, `Glob`
- **model**: `opus` — turning an acceptance criterion into a task a reader follows, and an ADR into a paragraph, is the judgment this phase keeps in a subagent.
- **input**: the release set with each story's `STORY-nnn`, title, and AC texts; every `adr/ADR-nnn.md` at `status: accepted` with its `## Context` and `## Decision` sections; the H2 sections `## Languages` and `## Runtimes` of `context/tech-stack.md`, `## Layers` of `context/source-tree.md`, `## Approved dependencies` and `## License policy` of `context/dependencies.md`, `## Constraint index` of `context/architecture-constraints.md`; `[release].docs_root`; the version string.
- **output**: One JSON object on stdout and nothing else. The agent file fixes the shape by worked example; this is the first of them:

```json
{
  "schema": "devforgeai/guide-docs/1",
  "guide": { "path": "docs/guide/index.md", "stories": 4, "tasks": 11 },
  "architecture": { "path": "docs/architecture/index.md", "decisions": 7, "constraints": 12 },
  "stories_without_task": [],
  "reason": ""
}
```

  `stories_without_task` lists the `STORY-nnn` ids whose ACs produced no H3 in the guide. `reason` is `""` on success.
- **invoked_at**: workflow step 10, in parallel with `api-doc-writer`.
- **registered_verifier**: `no` — the guide has no pass ratio; the `file_exists` check confirms both files and `docs_cover` reads the API pages alone.
- **report_field**: none

### 43. observation-miner

- **name**: `observation-miner`
- **owned_by**: Reflect · `improving-framework`
- **derives_from**: `C:\Users\bryan\.claude\agents\observation-extractor.md`
- **purpose**: Name the gate failures, phase durations, and unparsed verifier blocks in one window aggregate as observations, each tied to the report paths that carry it.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `sonnet` — the counts arrive already computed; the judgment is which of them is worth a line and how to phrase it.
- **input**: the aggregate's `phase_time[]`, `gate_failures[]`, `verifier_failures[]`, and, per `reports[]` entry, `path`, `id`, `phase`, `status`, `checks[]`, and `findings[]`; `window.from` and `window.to`.
- **output**: One JSON object on stdout and nothing else. The schema the agent file fixes:

```json
{ "type": "object", "required": ["observations","notes"],
  "properties": {
    "observations": { "type": "array", "items": { "type": "object",
      "required": ["ref","kind","severity","phase","summary","detail","count","metric","sources","evidence"],
      "properties": {
        "ref": { "type": "string", "pattern": "^om-[0-9]{3}$" },
        "kind": { "type": "string", "enum": ["gate_failure","phase_time","verifier_unparsed"] },
        "severity": { "type": "string", "enum": ["low","medium","high"] },
        "phase": { "type": "string", "enum": ["explore","discover","constitute","plan","build","verify","release",""] },
        "summary": { "type": "string", "minLength": 1, "maxLength": 120 },
        "detail": { "type": "string", "minLength": 1, "maxLength": 400 },
        "count": { "type": "integer", "minimum": 1 },
        "metric": { "type": "string", "maxLength": 20 },
        "sources": { "type": "array", "minItems": 1, "items": { "type": "string" } },
        "evidence": { "type": "array", "maxItems": 5, "items": { "type": "object",
          "required": ["path","line","at"], "properties": {
            "path": { "type": "string" },
            "line": { "type": "integer", "minimum": 0 },
            "at": { "type": "string" } } } } } } },
    "notes": { "type": "array", "maxItems": 5, "items": { "type": "string", "maxLength": 160 } } } }
```

- **invoked_at**: workflow step 3, in parallel with `session-pattern-reader` and `debt-aggregator`.
- **registered_verifier**: `no`
- **report_field**: none

### 44. session-pattern-reader

- **name**: `session-pattern-reader`
- **owned_by**: Reflect · `improving-framework`
- **derives_from**: `C:\Users\bryan\.claude\agents\session-miner.md`
- **purpose**: Name the repeated send-backs and re-typed commands in a project's session history as observations, each tied to a session id and a line number.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `opus` — reading a sequence of commands and saying which repetition is one pattern rather than three incidents is the judgment this skill exists for.
- **input**: the aggregate's `sessions.files[]`, `sessions.commands[]`, and `sessions.repeats[]`; `window.from` and `window.to`; the phase names of `state.current_phase` and each repeat's `from` and `to`.
- **output**: One JSON object on stdout and nothing else. The schema the agent file fixes:

```json
{ "type": "object", "required": ["observations","sessions_read","notes"],
  "properties": {
    "observations": { "type": "array", "items": { "type": "object",
      "required": ["ref","kind","severity","phase","summary","detail","count","sources","evidence"],
      "properties": {
        "ref": { "type": "string", "pattern": "^sp-[0-9]{3}$" },
        "kind": { "type": "string", "enum": ["repeated_send_back","friction"] },
        "severity": { "type": "string", "enum": ["low","medium","high"] },
        "phase": { "type": "string", "enum": ["explore","discover","constitute","plan","build","verify","release",""] },
        "summary": { "type": "string", "minLength": 1, "maxLength": 120 },
        "detail": { "type": "string", "minLength": 1, "maxLength": 400 },
        "count": { "type": "integer", "minimum": 2 },
        "sources": { "type": "array", "minItems": 1,
          "items": { "type": "string", "pattern": "^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$" } },
        "evidence": { "type": "array", "minItems": 1, "maxItems": 5, "items": { "type": "object",
          "required": ["path","line","at","command"], "properties": {
            "path": { "type": "string" },
            "line": { "type": "integer", "minimum": 1 },
            "at": { "type": "string" },
            "command": { "type": "string", "maxLength": 120 } } } } } } },
    "sessions_read": { "type": "integer", "minimum": 0 },
    "notes": { "type": "array", "maxItems": 5, "items": { "type": "string", "maxLength": 160 } } } }
```

- **invoked_at**: workflow step 4, in parallel with `observation-miner` and `debt-aggregator`; skipped when `sessions.status` is not `present`.
- **registered_verifier**: `no`
- **report_field**: none

### 45. debt-aggregator

- **name**: `debt-aggregator`
- **owned_by**: Reflect · `improving-framework`
- **derives_from**: `C:\Users\bryan\.claude\agents\technical-debt-analyzer.md`
- **purpose**: Group every deferred Definition-of-Done item in the window by the constraint or anti-pattern it cites, with each item's age in calendar days.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `sonnet` — the grouping key and the age arrive in the aggregate; the judgment is the group title and which reason text belongs on each row.
- **input**: the aggregate's `deferrals[]` (`story`, `dod_item`, `deferred_at`, `age_days`, `constraint`, `reason`, `report`), `window.to`, and the `## Constraint index` and `## Anti-pattern index` rows of `.devforgeai/context/architecture-constraints.md` and `.devforgeai/context/anti-patterns.md`.
- **output**: One JSON object on stdout and nothing else. The schema the agent file fixes:

```json
{ "type": "object", "required": ["as_of","total","oldest_days","groups"],
  "properties": {
    "as_of": { "type": "string", "pattern": "^[0-9]{4}-[0-9]{2}-[0-9]{2}$" },
    "total": { "type": "integer", "minimum": 0 },
    "oldest_days": { "type": "integer", "minimum": 0 },
    "groups": { "type": "array", "items": { "type": "object",
      "required": ["constraint","kind","count","oldest_days","items"],
      "properties": {
        "constraint": { "type": "string", "pattern": "^(CON-[0-9]{3}|AP-[0-9]{3}|none)$" },
        "kind": { "type": "string", "enum": ["constraint","anti_pattern","none"] },
        "count": { "type": "integer", "minimum": 1 },
        "oldest_days": { "type": "integer", "minimum": 0 },
        "items": { "type": "array", "minItems": 1, "items": { "type": "object",
          "required": ["story","dod_item","deferred_at","age_days","reason","report"],
          "properties": {
            "story": { "type": "string", "pattern": "^STORY-[0-9]{3}$" },
            "dod_item": { "type": "string", "minLength": 1, "maxLength": 120 },
            "deferred_at": { "type": "string", "pattern": "^[0-9]{4}-[0-9]{2}-[0-9]{2}$" },
            "age_days": { "type": "integer", "minimum": 0 },
            "reason": { "type": "string", "minLength": 1, "maxLength": 200 },
            "report": { "type": "string" } } } } } } } } }
```

- **invoked_at**: workflow step 5, in parallel with `observation-miner` and `session-pattern-reader`.
- **registered_verifier**: `no`
- **report_field**: none

### 46. recommendation-drafter

- **name**: `recommendation-drafter`
- **owned_by**: Reflect · `improving-framework`
- **derives_from**: `C:\Users\bryan\.claude\agents\framework-analyst.md`
- **purpose**: Turn a set of id-bearing observations into recommendations, one target file and one change apiece, each citing the observations behind it.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `opus` — naming which one file a friction point lives in, and what change to it removes the friction, is the judgment this skill exists for.
- **input**: the `observations[]` of step 6 with their `OBS-nnn` ids; the aggregate's `floors` block; the target-kind table of `specs/10-reflect.md` `## Outputs`; `templates/rec-targets.md`; the paths under `.claude/skills/`, `.claude/agents/`, and `.devforgeai/` that `Glob` returns.
- **output**: One JSON object on stdout and nothing else. The schema the agent file fixes:

```json
{ "type": "object", "required": ["recommendations","dropped","notes"],
  "properties": {
    "recommendations": { "type": "array", "items": { "type": "object",
      "required": ["ref","observations","target","change","current_value","proposed_value","effort","applies_to"],
      "properties": {
        "ref": { "type": "string", "pattern": "^rd-[0-9]{3}$" },
        "observations": { "type": "array", "minItems": 1,
          "items": { "type": "string", "pattern": "^OBS-[0-9]{3}$" } },
        "target": { "type": "object", "required": ["kind","path","key"], "properties": {
          "kind": { "type": "string",
            "enum": ["skill","subagent","template","hook","gate_threshold","framework_file"] },
          "path": { "type": "string", "minLength": 1 },
          "key": { "type": "string" } } },
        "change": { "type": "string", "minLength": 1, "maxLength": 300 },
        "current_value": { "type": "string", "maxLength": 40 },
        "proposed_value": { "type": "string", "maxLength": 40 },
        "effort": { "type": "string", "enum": ["small","medium","large"] },
        "applies_to": { "type": "array", "items": { "type": "string",
          "enum": ["explore","discover","constitute","plan","build","verify","release"] } } } } },
    "dropped": { "type": "array", "items": { "type": "object",
      "required": ["key","proposed_value","floor","reason"], "properties": {
        "key": { "type": "string" },
        "proposed_value": { "type": "string" },
        "floor": { "type": "string" },
        "reason": { "type": "string", "maxLength": 160 } } } },
    "notes": { "type": "array", "maxItems": 5, "items": { "type": "string", "maxLength": 160 } } } }
```

- **invoked_at**: workflow step 7, alone, after steps 3 to 6 return.
- **registered_verifier**: `no`
- **report_field**: none

### Model budget

| Model | Count | Agents |
|---|---|---|
| `opus` | 25 | `idea-interrogator`, `flow-drafter`, `kill-case-builder`, `requirement-drafter`, `flow-integrity-auditor`, `architecture-reviewer`, `story-decomposer`, `story-invest-auditor`, `spec-gap-triager`, `ac-test-writer`, `backend-implementer`, `story-ac-verifier`, `context-validator`, `ac-compliance-verifier`, `standards-reviewer`, `constraint-auditor`, `deferral-validator`, `security-auditor`, `adr-conformance-reviewer`, `brand-designer`, `ui-spec-writer`, `deferral-auditor`, `guide-writer`, `session-pattern-reader`, `recommendation-drafter` |
| `sonnet` | 21 | `landscape-scanner`, `prototype-builder`, `persona-mapper`, `epic-grouper`, `alignment-auditor`, `source-tree-mapper`, `story-file-set-planner`, `sprint-sequencer`, `frontend-implementer`, `refactor-surgeon`, `integration-test-writer`, `anti-pattern-scanner`, `coverage-gap-auditor`, `dead-code-detector`, `code-quality-auditor`, `mockup-designer`, `requirement-coverage-auditor`, `deploy-manifest-writer`, `api-doc-writer`, `observation-miner`, `debt-aggregator` |
| `inherit` | 0 | none |

The rule that produced the split, stated once and applied by every skill spec:

1. **`opus`** where the agent's output decides a gate outcome or routes a send-back, or where the agent argues against work the same session produced. Fourteen of the 25 are registered verifiers whose ratio a `verifier_pass` or `report_metric` check reads. The other eleven write text a later phase is built from (`backend-implementer`, `ui-spec-writer`, `guide-writer`, `story-decomposer`), route a send-back (`spec-gap-triager`), or make a judgment no fixed schema constrains (`idea-interrogator`, `flow-drafter`, `requirement-drafter`, `brand-designer`, `session-pattern-reader`, `recommendation-drafter`).
2. **`sonnet`** where the input arrives measured, enumerated, or tabulated and the output shape is fixed by a schema, an index, or a template. Six of the 21 are registered verifiers, each reading numbers or rows the CLI or the context set already produced: `alignment-auditor` compares a bounded file set, `anti-pattern-scanner` matches the `## Anti-pattern index` detectors, `coverage-gap-auditor` and `code-quality-auditor` read figures against `config.toml` thresholds, `dead-code-detector` counts references, and `requirement-coverage-auditor` resolves ids across two documents.
3. **`inherit`** only where the owning spec justified it, and no spec did. `specs/07-verify.md` `## Subagents` moved `dead-code-detector` from the existing agent's `inherit` to `sonnet` with the reason "so the judgement does not change with the caller", which is the reason the value is unused across all 46. The enum stays three-valued because §10 fixes it; the framework's v1 population is two-valued.

### Disposition of the existing agent set

`C:\Users\bryan\.claude\agents\` holds 117 files: 47 at the top level (46 `.md` plus one `.md.bak`) and 70 under companion `references\` directories. Every file appears exactly once across the two tables below.

The disposition enum is closed at four values:

- **`adapted`** — a subagent in this catalog derives from the file, and the owning spec's `derives_from` names it. The row gives the new name and the owning skill.
- **`replaced`** — a subagent in this catalog occupies the file's slot without carrying its method forward. The row gives the new name.
- **`retired`** — no subagent in this catalog does the file's work; the work moved to the `devforgeai` binary, to a hook, to `devforgeai handoff`, or nowhere. The row gives the reason.
- **`reference-only`** — the file is not an agent file. Conventions §3 lays the framework's agent directory out as `agents\<agent-name>.md`, a flat file with no companion directory, so a reference file becomes `references/` content of a skill or is dropped. The row says which.

`derives_from` and `disposition` are independent fields. `derives_from` is a §10 field the owning skill spec sets and this catalog carries unchanged; it records which file the spec author read. `disposition` is this catalog's judgement about the old file. Two rows differ between them on purpose: `api-designer.md` is `derives_from` for `story-file-set-planner` and `replaced` here, because `specs/05-plan.md` `## Subagents` writes "replaced, not adapted"; `ui-spec-formatter.md` is `derives_from` for `ui-spec-writer` and `replaced` here, because `specs/08-design.md` `## Decisions` 21 writes "replaced". The owning spec's word settles `adapted` against `replaced`; this catalog decides only where no spec gave a word, or where the word named no successor.

**Table A — the 47 top-level files.**

| File | Disposition | Becomes / reason | Decided by |
|---|---|---|---|
| `ac-compliance-verifier.md` | adapted | `ac-compliance-verifier` · Verify. Also the `derives_from` of `story-ac-verifier` · Build | `specs/07-verify.md` §Subagents; `specs/06-build.md` §Subagents |
| `agent-generator.md` | retired | It writes `.claude/agents/*.md`. No skill in the framework generates an agent file at run time; the 46 files are built from `## Templates` at release time | `specs/10-reflect.md` §Subagents (labelled "replaced" with no successor named); this catalog, `## Decisions` 6 |
| `alignment-auditor.md` | adapted | `alignment-auditor` · Constitute, with the method inverted: exact-text matching moves to `context audit` CA-1 through CA-8 and the agent takes the semantic comparison | `specs/04-constitute.md` §Subagents, §Decisions 15 |
| `anti-pattern-scanner.md` | adapted | `anti-pattern-scanner` · Verify, name kept | `specs/07-verify.md` §Subagents |
| `api-designer.md` | replaced | `story-file-set-planner` · Plan | `specs/05-plan.md` §Subagents; `specs/04-constitute.md` §Decisions 17 named Plan its owner |
| `architect-reviewer.md` | adapted | `architecture-reviewer` · Constitute. Also the `derives_from` of `adr-conformance-reviewer` · Verify | `specs/04-constitute.md` §Subagents; `specs/07-verify.md` §Subagents |
| `backend-architect.md` | adapted | `backend-implementer` · Build | `specs/06-build.md` §Subagents |
| `business-coach.md` | adapted | `idea-interrogator` · Explore | `specs/02-explore.md` §Subagents |
| `code-analyzer.md` | adapted | `source-tree-mapper` · Constitute | `specs/04-constitute.md` §Subagents |
| `code-quality-auditor.md` | adapted | `code-quality-auditor` · Verify, name kept | `specs/07-verify.md` §Subagents |
| `code-reviewer.md` | adapted | `standards-reviewer` · Verify | `specs/07-verify.md` §Subagents |
| `context-preservation-validator.md` | adapted | `flow-integrity-auditor` · Discover | `specs/03-discover.md` §Subagents, §Decisions 18 |
| `context-preservation-validator.md.bak` | retired | A backup copy of the adapted file. It is not loaded as an agent and ships nowhere | this catalog, `## Decisions` 7 |
| `context-validator.md` | adapted | `context-validator` · Build, name kept. Also the `derives_from` of `constraint-auditor` · Verify | `specs/06-build.md` §Subagents; `specs/07-verify.md` §Subagents, §Decisions 29 |
| `coverage-analyzer.md` | adapted | `coverage-gap-auditor` · Verify | `specs/07-verify.md` §Subagents |
| `dead-code-detector.md` | adapted | `dead-code-detector` · Verify, name kept, `model` moved from `inherit` to `sonnet` | `specs/07-verify.md` §Subagents |
| `deferral-validator.md` | adapted | `deferral-validator` · Verify, name kept. Also the `derives_from` of `deferral-auditor` · Release | `specs/07-verify.md` §Subagents, §Decisions 29; `specs/09-release.md` §Subagents, §Decisions 25, 31 |
| `dependency-graph-analyzer.md` | retired | Cycle detection is `devforgeai story validate` check 5, `DFA-E232`; transitive resolution and ordering are `sprint-sequencer`; its status vocabulary is replaced by the six-value enum `phase set` owns | `specs/05-plan.md` §Subagents |
| `deployment-engineer.md` | adapted | `deploy-manifest-writer` · Release | `specs/09-release.md` §Subagents |
| `dev-result-interpreter.md` | retired | The end-of-phase block is `devforgeai handoff`, which renders it from the report (§1 rule 3) | `specs/06-build.md` §Subagents |
| `diagnostic-analyst.md` | replaced | `alignment-auditor` · Constitute and `anti-pattern-scanner` · Verify between them carry the spec-drift reading; a second pass would produce project findings where Reflect emits framework recommendations | `specs/10-reflect.md` §Subagents; `specs/04-constitute.md` §Subagents ("not invoked here") |
| `documentation-writer.md` | adapted | `api-doc-writer` and `guide-writer`, both · Release; split rather than adapted into one, because the two read disjoint inputs and run in parallel | `specs/09-release.md` §Subagents, §Decisions 27 |
| `entrepreneur-assessor.md` | replaced | `kill-case-builder` · Explore | `specs/02-explore.md` §Decisions 20; `specs/03-discover.md` §Decisions 18 ("unused") |
| `epic-coverage-result-interpreter.md` | retired | Coverage counting is `devforgeai story validate` check `DFA-E235`; its four display templates are `devforgeai handoff` | `specs/05-plan.md` §Subagents |
| `file-overlap-detector.md` | retired | Pre-flight overlap is `story validate --scope sprint` check 9, `DFA-E237`; concurrent overlap is the `DFA-E272` refusal of `worktree ensure`; post-flight drift is `story files --diff` | `specs/06-build.md` §Subagents |
| `framework-analyst.md` | adapted | `recommendation-drafter` · Reflect | `specs/10-reflect.md` §Subagents |
| `frontend-developer.md` | adapted | `frontend-implementer` · Build. Also the `derives_from` of `mockup-designer` · Design and `prototype-builder` · Explore | `specs/06-build.md` §Subagents; `specs/08-design.md` §Subagents, §Decisions 20; `specs/02-explore.md` §Subagents |
| `git-validator.md` | retired | `devforgeai worktree ensure` reports `DFA-E271` when the project root is not a git work tree, and the §7 git hooks carry the rest | `specs/06-build.md` §Subagents, §Decisions 25 |
| `git-worktree-manager.md` | retired | Worktree creation, idle detection, and limit enforcement are `devforgeai worktree ensure`, `list`, and `remove`; they are file and process operations with no judgment in them | `specs/06-build.md` §Subagents, §Decisions 25 |
| `humanizer-grader.md` | retired | It grades prose for LLM authorship and writes an HTML report. No conventions §5 document is graded that way, no gate reads such a score, and its `Write` of a self-chosen path is outside the §3 layout | this catalog, `## Decisions` 8 |
| `ideation-result-interpreter.md` | retired | Its summary block and next-step table are the §6 handoff, which `devforgeai handoff` prints | `specs/03-discover.md` §Decisions 18 |
| `integration-tester.md` | adapted | `integration-test-writer` · Build | `specs/06-build.md` §Subagents |
| `internet-sleuth.md` | adapted | `landscape-scanner` · Explore | `specs/02-explore.md` §Subagents; `specs/03-discover.md` §Decisions 18 ("unused") |
| `observation-extractor.md` | adapted | `observation-miner` · Reflect | `specs/10-reflect.md` §Subagents |
| `pattern-compliance-auditor.md` | replaced | `recommendation-drafter` · Reflect, which fills the `command` row of the target-kind table. Its protocol file is outside the §3 layout and its effort-hour roadmap is an applier's output | `specs/10-reflect.md` §Subagents |
| `qa-result-interpreter.md` | retired | It composes the user-facing result block and picks one of eight display templates; `devforgeai handoff` renders that block from `state.toml` and the report. Its `mode` enum `light \| deep` survives as the QA report's `mode` field | `specs/07-verify.md` §Subagents, §Decisions 28 |
| `refactoring-specialist.md` | adapted | `refactor-surgeon` · Build | `specs/06-build.md` §Subagents |
| `requirements-analyst.md` | adapted | `requirement-drafter` and `epic-grouper`, both · Discover. Also the `derives_from` of `flow-drafter` · Explore and `story-invest-auditor` · Plan | `specs/03-discover.md` §Subagents, §Decisions 18; `specs/02-explore.md` §Subagents; `specs/05-plan.md` §Subagents |
| `security-auditor.md` | adapted | `security-auditor` · Verify, name kept | `specs/07-verify.md` §Subagents |
| `session-miner.md` | adapted | `session-pattern-reader` · Reflect | `specs/10-reflect.md` §Subagents |
| `sprint-planner.md` | adapted | `sprint-sequencer` · Plan | `specs/05-plan.md` §Subagents |
| `stakeholder-analyst.md` | adapted | `persona-mapper` · Discover | `specs/03-discover.md` §Subagents, §Decisions 18; `specs/02-explore.md` §Decisions 19 ("not invoked") |
| `story-requirements-analyst.md` | adapted | `story-decomposer` · Plan | `specs/05-plan.md` §Subagents |
| `tech-stack-detector.md` | retired | Detection is `devforgeai stack detect`, which writes `config.toml`; validation of detected values against `tech-stack.md` is `context audit` CA-5 | `specs/04-constitute.md` §Subagents, §Decisions 16; `specs/06-build.md` §Subagents |
| `technical-debt-analyzer.md` | adapted | `debt-aggregator` · Reflect | `specs/10-reflect.md` §Subagents |
| `test-automator.md` | adapted | `ac-test-writer` · Build | `specs/06-build.md` §Subagents |
| `ui-spec-formatter.md` | replaced | `ui-spec-writer` · Design, which writes the document instead of formatting a finished one for display | `specs/08-design.md` §Decisions 21; `specs/05-plan.md` §Subagents ("retired"); `specs/02-explore.md` §Decisions 21 ("not invoked") |

Table A counts: **adapted 30, replaced 5, retired 12, total 47.**

**Table B — the 70 companion reference files.** Every row is `reference-only`. Paths are relative to `C:\Users\bryan\.claude\agents\`.

| File | Target | Reason |
|---|---|---|
| `ac-compliance-verifier\references\report-generation.md` | dropped | Report rendering is `report ingest` plus `devforgeai handoff` |
| `ac-compliance-verifier\references\scoring-methodology.md` | dropped | The score is `passed` over `total` in the `devforgeai/verifier/1` envelope |
| `ac-compliance-verifier\references\verification-workflow.md` | `skills/validating-quality/references/ac-verification.md` | Tool-neutral reading of a criterion against a test, which the adapted contract keeps |
| `ac-compliance-verifier\references\xml-parsing-protocol.md` | dropped | A Plan criterion is the Markdown list item `- AC-nnn: Given … When … Then …`; the XML parser has no input |
| `agent-generator\references\canonical-agent-template.md` | dropped | Superseded by the agent-file template in `## Templates` |
| `agent-generator\references\command-refactoring-patterns.md` | dropped | Parent retired |
| `agent-generator\references\error-handling.md` | dropped | Parent retired |
| `agent-generator\references\frontmatter-specification.md` | dropped | Superseded by the four frontmatter keys in `## Templates` |
| `agent-generator\references\output-formats.md` | dropped | Superseded by the `devforgeai/verifier/1` envelope and the per-agent schemas |
| `agent-generator\references\reference-file-templates.md` | dropped | §3 gives an agent one flat file and no `references/` directory |
| `agent-generator\references\template-compliance-validation.md` | dropped | Parent retired |
| `agent-generator\references\template-patterns.md` | dropped | Parent retired |
| `agent-generator\references\tool-restrictions.md` | dropped | Superseded by the closed-tools rule in `## Templates` |
| `agent-generator\references\validation-workflow.md` | dropped | Parent retired |
| `alignment-auditor\references\validation-matrix.md` | dropped | Its exact-match rows become `context audit` checks CA-1 through CA-8 |
| `anti-pattern-scanner\references\code-smell-catalog.md` | dropped | Replaced by the project's own `## Anti-pattern index` rows |
| `anti-pattern-scanner\references\integration-testing-guide.md` | dropped | Integration tests are Build's `integration-test-writer` |
| `anti-pattern-scanner\references\metrics-reference.md` | dropped | The thresholds are `[verify].complexity_max`, `[verify].duplication_max_percent`, `[verify].duplication_min_lines` |
| `anti-pattern-scanner\references\output-contract.md` | dropped | Replaced by the `devforgeai/verifier/1` envelope |
| `anti-pattern-scanner\references\phase1-context-loading.md` | dropped | The inputs are the prompt fields of the agent's `input` field |
| `anti-pattern-scanner\references\phase2-library-detection.md` | dropped | Library substitution is a `Detector` cell of an `AP-nnn` row |
| `anti-pattern-scanner\references\phase3-structure-detection.md` | dropped | Structure violation is a `Detector` cell of an `AP-nnn` row |
| `anti-pattern-scanner\references\phase4-layer-detection.md` | dropped | Layer violation is `## Layer dependency rules` plus `constraint-auditor` |
| `anti-pattern-scanner\references\phase5-code-smells.md` | dropped | Replaced by the project's own `## Anti-pattern index` rows |
| `anti-pattern-scanner\references\phase5-treelint-detection.md` | dropped | Names one external tool and pins its version, which §1 rule 2 excludes; the capability is `[verify].call_graph_command` |
| `anti-pattern-scanner\references\phase6-security-scanning.md` | dropped | Security is `security-auditor` against the ten-value `owasp` enum |
| `anti-pattern-scanner\references\phase7-style-checks.md` | dropped | Style is `coding-standards.md` read by `standards-reviewer` |
| `anti-pattern-scanner\references\two-stage-filter-patterns.md` | `skills/validating-quality/references/detector-filtering.md` | Match-then-confirm over a supplied detector names no tool and the adapted agent applies it |
| `api-designer\references\openapi-specification.md` | dropped | Parent replaced; the contract is the story's `## Files` table and the `Then` clause of a criterion |
| `api-designer\references\rest-design-patterns.md` | dropped | Parent replaced; §1 rule 2 keeps a protocol style out of a subagent |
| `backend-architect\references\framework-patterns.md` | dropped | Names frameworks, which §1 rule 2 excludes from a subagent |
| `backend-architect\references\implementation-patterns.md` | `skills/implementing-stories/references/layering.md` | Layer placement under `## Layer dependency rules` names no tool |
| `backend-architect\references\treelint-patterns.md` | dropped | Names one external tool; the capability is `[verify].call_graph_command` |
| `code-analyzer\references\analysis-patterns.md` | `skills/establishing-context/references/tree-reading.md` | Layer, root, and entry-point discovery, which `source-tree-mapper` keeps |
| `code-quality-auditor\references\analysis-workflow.md` | dropped | The workflow is the `method` enum `command` and `grep` in the agent's own schema |
| `code-reviewer\references\anti-gaming-validation.md` | dropped | `story-ac-verifier` reads the diff and the test output in a fresh context, which replaces the step |
| `code-reviewer\references\dependency-impact-analysis.md` | dropped | Dependency scope is `## Approved dependencies` and `## Forbidden dependencies` |
| `code-reviewer\references\review-checklist.md` | dropped | A numbered self-validation list of the form §2 forbids in a subagent; the review targets are the seven `coding-standards.md` sections |
| `code-reviewer\references\treelint-review-patterns.md` | dropped | Names one external tool; the capability is `[verify].metrics_command` |
| `coverage-analyzer\references\semantic-test-coverage-mapping.md` | `skills/validating-quality/references/coverage-mapping.md` | Which uncovered region relates to which criterion is the one judgement `coverage-gap-auditor` keeps |
| `coverage-analyzer\references\treelint-patterns.md` | dropped | Names one external tool; the capability is `[verify].call_graph_command` |
| `dead-code-detector\references\entry-point-patterns.md` | dropped | Entry points are the `## Roots` and `## Generated and excluded paths` entries of `source-tree.md` |
| `deployment-engineer\references\platform-patterns.md` | `skills/releasing-software/templates/` | Its content becomes the release skill's manifest templates, where the `deploy_manifest` check reads fixed text |
| `frontend-developer\references\accessibility-patterns.md` | `skills/designing-interfaces/references/accessibility.md` | The eight accessibility rows of a `UI-nnn` document are written from it |
| `frontend-developer\references\framework-patterns.md` | dropped | Names three component frameworks, which §1 rule 2 excludes |
| `frontend-developer\references\performance-patterns.md` | dropped | No gate check in the framework reads a performance number |
| `integration-tester\references\anti-gaming-validation.md` | dropped | Duplicate of the `code-reviewer` file and dropped for the same reason |
| `integration-tester\references\test-patterns.md` | `skills/implementing-stories/references/boundary-tests.md` | The four `boundary` enum values of `integration-test-writer` are written from it |
| `refactoring-specialist\references\refactoring-catalog.md` | `skills/implementing-stories/references/rewrites.md` | The seven-value `pattern` enum of `refactor-surgeon` is written from it |
| `refactoring-specialist\references\treelint-refactoring-patterns.md` | dropped | Names one external tool; the capability is `[verify].metrics_command` |
| `references\treelint-search-patterns.md` | dropped | The shared external-tool reference; §1 rule 2 excludes it |
| `requirements-analyst\references\common-story-patterns.md` | `skills/planning-work/references/story-patterns.md` | Decomposition shapes `story-decomposer` applies |
| `requirements-analyst\references\edge-cases.md` | `skills/planning-work/references/edge-cases.md` | Criterion coverage of the negative path, which `ac-test-writer` then turns into a test |
| `requirements-analyst\references\nfr-templates.md` | dropped | A non-functional requirement is a `CON-nnn` written by Constitute |
| `requirements-analyst\references\story-format-template.md` | dropped | Superseded by `specs/05-plan.md` `## Templates` |
| `requirements-analyst\references\story-splitting-techniques.md` | `skills/planning-work/references/splitting.md` | The split judgement `story-decomposer` and `story-invest-auditor` share |
| `security-auditor\references\owasp-patterns.md` | dropped | A pattern library keyed to three named languages; the ten categories are the closed `owasp` enum and the patterns are `Detector` cells |
| `security-auditor\references\treelint-security-patterns.md` | dropped | Names one external tool; the capability is `[verify].call_graph_command` |
| `session-miner\references\anti-pattern-mining.md` | dropped | Framework friction is `session-pattern-reader`'s two-value `kind` enum |
| `session-miner\references\error-handling.md` | `skills/improving-framework/references/jsonl-reading.md` | Error-tolerant JSON Lines reading, which `session-pattern-reader` keeps |
| `session-miner\references\output-formats.md` | dropped | Superseded by the agent's own JSON schema |
| `session-miner\references\parsing-workflow.md` | dropped | The chunking is the CLI aggregate's; the agent reads the line ranges it points at |
| `session-miner\references\query-patterns.md` | dropped | The queries are the aggregate's `sessions.commands[]` and `sessions.repeats[]` |
| `session-miner\references\session-analysis.md` | dropped | Superseded by the aggregate the CLI computes |
| `test-automator\references\common-patterns.md` | `skills/implementing-stories/references/test-shapes.md` | Arrange-act-assert shaping of one criterion, which `ac-test-writer` keeps |
| `test-automator\references\coverage-optimization.md` | dropped | Coverage floors are `[[layer]].coverage_min` and `[coverage].overall_min`, read by the gate |
| `test-automator\references\exception-path-coverage.md` | `skills/implementing-stories/references/negative-paths.md` | Turning a failure criterion into an assertion names no runner |
| `test-automator\references\framework-patterns.md` | dropped | Names test frameworks, which §1 rule 2 excludes |
| `test-automator\references\remediation-mode.md` | dropped | Remediation is the `--remedy` flag of §4c plus the `FIND-nnn` the report carries |
| `test-automator\references\technical-specification.md` | dropped | Superseded by the agent's own JSON schema |

Table B counts: **reference-only 70**, of which 16 become skill `references/` or `templates/` content and 54 are dropped.

Whole-directory counts: **adapted 30, replaced 5, retired 12, reference-only 70, total 117.**

## Command

None. This document specifies no phase and no entry point, and conventions §4b fixes one slash command per skill with no tenth name available. The nine entry points are the nine skills' own frontmatter.

## CLI calls

This document invokes no subcommand itself. The table records the three subcommands that read or write the catalog's outputs, with the exact arguments, so the registry contract has one place to be checked against.

| Subcommand | Exact arguments | Caller | What it does with a catalog output |
|---|---|---|---|
| `devforgeai init` | `init [--analyze]` | user | Copies `agents/*.md` into `<target>\.claude\agents\` and writes the 20 `[[verifier]]` tables of `## Decisions` 1 into `<target>\.devforgeai\config.toml` when that file is absent |
| `devforgeai report ingest` | `report ingest <subagent> -` | SubagentStop hook | Looks `<subagent>` up in `config.toml` `[[verifier]]`, parses the stdout as `devforgeai/verifier/1`, writes the block at `verifiers.<report_field>` of `.devforgeai/reports/<id>-<phase>.yaml`, appends `findings` de-duplicated by `id` |
| `devforgeai gate check` | `gate check --phase <phase> [--id <id>]` | Stop hook, pre-push, CI | Reads `verifiers.<report_field>` for each `verifier_pass` check and each `report_metric` check the phase gate names |

`report ingest <subagent> <source>` is an addition to the §4 table, accepted in `specs/01-cli.md` `## Decisions` 1. No other subcommand in this document is outside the §4 list.

## Gate

None. A gate is a `[[gate]]` entry in `gates.toml` keyed by phase, and this document specifies no phase. The gate entries that read a registered verifier's block live in the owning skill specs: Explore `kill-case-answered`, Discover's flow checks, Constitute's three `report_metric` checks, Plan's `verifier_pass`, Build's `build-testable`, `build-acs`, `build-context`, Verify's `verify-acs`, `verify-light`, `verify-deep`, and Release's deferral check.

## Send-back

None. A send-back is a phase returning a document to its producer with cited IDs, and this document produces no phase document. A defect in a catalog entry is a spec amendment recorded under `## Decisions`, not a SEND BACK handoff.

## Integration

Every one of the 46 subagents has exactly one `owned_by` skill, so the "shared subagents" column reads `none` on all ten rows. That is the correct value rather than a gap in the analysis, for two reasons stated once here and not repeated per row. First, sharing in this framework happens at lineage level, not at invocation level: `requirements-analyst.md` is the `derives_from` of four agents across three skills, `frontend-developer.md` of three across three, `architect-reviewer.md`, `context-validator.md`, `deferral-validator.md`, `ac-compliance-verifier.md`, and `documentation-writer.md` of two each — and each descendant is a separate file with its own name, tools, model, and schema, because conventions §10 requires a name unique across the framework and `specs/01-cli.md` binds one registry `name` to one `phase`. Second, where one skill needs another skill's agent it invokes the skill, not the agent: Explore reaches `mockup-designer` by calling `designing-interfaces` in sketch mode at its workflow step 6, and `prototype-builder` consumes that mode's `screens[]` output. The `shares (lineage)` column below records the first kind; the `shares (through a skill)` column records the second.

| Skill | Owns | shares (lineage) | shares (through a skill) | `[[verifier]]` entries it adds |
|---|---|---|---|---|
| Explore · `exploring-ideas` | `idea-interrogator`, `landscape-scanner`, `flow-drafter`, `prototype-builder`, `kill-case-builder` | `requirements-analyst.md` with Discover and Plan; `frontend-developer.md` with Build and Design | `mockup-designer` through `designing-interfaces` sketch mode at step 6 | `kill-case-builder` |
| Discover · `discovering-requirements` | `persona-mapper`, `requirement-drafter`, `epic-grouper`, `flow-integrity-auditor` | `requirements-analyst.md` with Explore and Plan | none — Discover invokes no other skill | `flow-integrity-auditor` |
| Constitute · `establishing-context` | `architecture-reviewer`, `alignment-auditor`, `source-tree-mapper` | `architect-reviewer.md` with Verify | none — Constitute invokes no other skill | `architecture-reviewer`, `alignment-auditor` |
| Plan · `planning-work` | `story-decomposer`, `story-file-set-planner`, `story-invest-auditor`, `sprint-sequencer`, `spec-gap-triager` | `requirements-analyst.md` with Explore and Discover | none — Plan sends back to Design rather than invoking it | `story-invest-auditor` |
| Build · `implementing-stories` | `ac-test-writer`, `backend-implementer`, `frontend-implementer`, `refactor-surgeon`, `integration-test-writer`, `story-ac-verifier`, `context-validator` | `frontend-developer.md` with Explore and Design; `ac-compliance-verifier.md` with Verify; `context-validator.md` with Verify | none — Build reads `UI-nnn.md` and `tokens.json` as documents | `ac-test-writer`, `story-ac-verifier`, `context-validator` |
| Verify · `validating-quality` | `ac-compliance-verifier`, `standards-reviewer`, `anti-pattern-scanner`, `constraint-auditor`, `coverage-gap-auditor`, `dead-code-detector`, `deferral-validator`, `security-auditor`, `code-quality-auditor`, `adr-conformance-reviewer` | `architect-reviewer.md` with Constitute; `context-validator.md` with Build; `ac-compliance-verifier.md` with Build; `deferral-validator.md` with Release | none — Verify invokes no other skill | all ten |
| Design · `designing-interfaces` | `mockup-designer`, `brand-designer`, `requirement-coverage-auditor`, `ui-spec-writer` | `frontend-developer.md` with Explore and Build | none — Design is invoked by Explore, and invokes no skill in turn | `requirement-coverage-auditor` |
| Release · `releasing-software` | `deferral-auditor`, `deploy-manifest-writer`, `api-doc-writer`, `guide-writer` | `deferral-validator.md` with Verify; `documentation-writer.md` is split within this skill alone | none — Release invokes no other skill | `deferral-auditor` |
| Reflect · `improving-framework` | `observation-miner`, `session-pattern-reader`, `debt-aggregator`, `recommendation-drafter` | none — its four lineages are used by no other skill | none — Reflect reads reports and invokes no skill | none |
| CLI · `devforgeai` | none — it has no Agent tool, no model access, and no prompt | none | none | it writes the whole registry at `init` and reads it at `report ingest` and `gate check` |

### Invocation map

The ordered agent list per skill, with parallelism and registered-verifier status. This is the content each skill's `agents.md` carries, and the template for that file is in `## Templates`.

| Skill | Order | Step | Agent | Batch | Verifier |
|---|---|---|---|---|---|
| Explore | 1 | 2 | `idea-interrogator` | alone | no |
| Explore | 2 | 3 | `landscape-scanner` | alone | no |
| Explore | 3 | 4 | `flow-drafter` | alone | no |
| Explore | 4 | 7 | `prototype-builder` | alone; skipped when the user declines the prototype offer and on a remedy run | no |
| Explore | 5 | 8 | `kill-case-builder` | alone | yes · `verifiers.kill_case` |
| Discover | 1 | 4 | `flow-integrity-auditor` | alone; entry point A and the resume form only | yes · `verifiers.flow_integrity` |
| Discover | 2 | 5 | `persona-mapper` | alone | no |
| Discover | 3 | 9 | `requirement-drafter` | alone; re-entered from step 13 | no |
| Discover | 4 | 11 | `epic-grouper` | alone; re-entered from step 13; placement mode at entry point C | no |
| Constitute | 1 | B5 | `source-tree-mapper` | alone; brownfield path only | no |
| Constitute | 2 | 11 | `architecture-reviewer` | batch 1 of 2 | yes · `verifiers.architecture_reviewer` |
| Constitute | 3 | 11 | `alignment-auditor` | batch 1 of 2 | yes · `verifiers.alignment_auditor` |
| Constitute | 4 | R3, R5 | `architecture-reviewer`, then both at R5 | R5 is one batch of 2 | yes |
| Plan | 1 | 5 | `story-decomposer` | alone | no |
| Plan | 2 | 7 | `story-file-set-planner` | alone | no |
| Plan | 3 | 9 | `story-invest-auditor` | alone | yes · `verifiers.story_invest` |
| Plan | 4 | 10 | `story-invest-auditor` | alone; second pass over the edited set | yes |
| Plan | 5 | 11 | `sprint-sequencer` | alone | no |
| Plan | 6 | R2 | `spec-gap-triager` | alone; remedy runs only | no |
| Build | 1 | 7.1 | `ac-test-writer` | serial, once per criterion | yes · `verifiers.ac_testable` |
| Build | 2 | 7.4, 7.5 | `backend-implementer` or `frontend-implementer` | serial, once per criterion; one of the two per story `## Layer` | no |
| Build | 3 | 7.8 | `refactor-surgeon` | serial, zero or one per criterion | no |
| Build | 4 | 8 | `integration-test-writer` | serial, zero or one per story | no |
| Build | 5 | 9 | `context-validator` | serial, once | yes · `verifiers.context` |
| Build | 6 | 10 | `refactor-surgeon` | serial, on an anti-pattern match | no |
| Build | 7 | 11 | `story-ac-verifier` | serial, once, after 8, 9, and 10 | yes · `verifiers.story_ac` |
| Verify | 1 | 6 | `ac-compliance-verifier` | batch 1 of 7 | yes · `verifiers.ac_compliance` |
| Verify | 2 | 6 | `standards-reviewer` | batch 1 of 7 | yes · `verifiers.standards` |
| Verify | 3 | 6 | `anti-pattern-scanner` | batch 1 of 7 | yes · `verifiers.anti_patterns` |
| Verify | 4 | 6 | `constraint-auditor` | batch 1 of 7 | yes · `verifiers.constraints` |
| Verify | 5 | 6 | `coverage-gap-auditor` | batch 1 of 7 | yes · `verifiers.coverage_gaps` |
| Verify | 6 | 6 | `dead-code-detector` | batch 1 of 7 | yes · `verifiers.dead_code` |
| Verify | 7 | 6 | `deferral-validator` | batch 1 of 7 | yes · `verifiers.deferrals` |
| Verify | 8 | 7 | `security-auditor` | batch 2 of 3; deep mode only | yes · `verifiers.security` |
| Verify | 9 | 7 | `code-quality-auditor` | batch 2 of 3; deep mode only | yes · `verifiers.quality` |
| Verify | 10 | 7 | `adr-conformance-reviewer` | batch 2 of 3; deep mode only | yes · `verifiers.adr_conformance` |
| Verify | 11 | 9 | `deferral-validator` | alone; second pass over the deferrals this run drafts | yes |
| Design | 1 | 3, 4 | `mockup-designer` | alone | no |
| Design | 2 | 7, 8 | `brand-designer` | alone, in that step order | no |
| Design | 3 | 11 | `requirement-coverage-auditor` | alone | yes · `verifiers.requirement_coverage` |
| Design | 4 | 13 | `ui-spec-writer` | one batch, one invocation per screen | no |
| Design | 5 | 15 | `ui-spec-writer` | alone; one invocation for the cited `UI-nnn` | no |
| Release | 1 | 6 | `deferral-auditor` | alone | yes · `verifiers.deferrals` |
| Release | 2 | 8 | `deploy-manifest-writer` | alone; skipped when `platform.target` is `none` | no |
| Release | 3 | 10 | `api-doc-writer` | batch of 2 | no |
| Release | 4 | 10 | `guide-writer` | batch of 2 | no |
| Reflect | 1 | 3 | `observation-miner` | batch of 3 | no |
| Reflect | 2 | 4 | `session-pattern-reader` | batch of 3; skipped when `sessions.status` is not `present` | no |
| Reflect | 3 | 5 | `debt-aggregator` | batch of 3 | no |
| Reflect | 4 | 7 | `recommendation-drafter` | alone, after the batch returns | no |

Six batches run in parallel across the framework: Constitute step 11 and remedy step R5 (2 agents each), Verify step 6 (7 agents), Verify step 7 (3 agents, deep mode only), Design step 13 (one agent per screen), Release step 10 (2 agents), and Reflect steps 3 to 5 (3 agents). Every other invocation is one agent at a time, and Build's seven are serial throughout.

## Handoff

None. Conventions §6 fixes the handoff as a phase's closing block printed by `devforgeai handoff` from `state.toml` and the latest report. This document has no phase, writes no `state.toml` key, and produces no report, so it has no PASS block and no SEND BACK block to fill.

## Templates

Two files under `templates/`. The first is the shape of every `agents/<name>.md`; the second is the shape of every `skills/<skill-name>/agents.md`. Both are given whole, fenced, so the headings inside them stay inside them.

### templates/agent-file.md

An agent file is one flat Markdown file at `agents/<name>.md`, with no companion directory. Its frontmatter carries four required keys, in the order below, plus the optional keys rule 5 admits. Its body carries three H2 sections and no others, in this order.

````markdown
---
name: <kebab-case, unique across the framework, equal to the file stem>
description: <one sentence, at most 160 characters: what this agent decides or produces, then
              the condition under which it is delegated to. No workflow step number, no batch
              position, no slash command, no verifier registration - those live in
              skills/<skill>/agents.md, which is what the skill reads.>
tools: [<closed, minimal list>]
disallowedTools: [<optional; see rule 5>]
model: <opus | sonnet | inherit>
maxTurns: <optional; see rule 5>
memory: <optional; see rule 5>
effort: <optional; see rule 5>
---

# <Title Case name>

<One paragraph opening with a role sentence - "This agent decides ...", "This agent turns ..." -
which carries the purpose the description no longer states, then the reason behind the rules the
workflow applies. A review or audit agent closes the paragraph with the recall sentence of rule 6.
No second paragraph.>

## Input

The prompt carries these fields and no others.

| Field | Type | Source |
|---|---|---|
| <field> | <type> | <the document, ID, or CLI output the skill reads it from> |

## Output

One JSON object on stdout and nothing else.

```json
<the output schema of this agent's catalog entry, verbatim>
```

<example>
<one instance of that schema, tagged; a registered verifier shows two or three, covering a
finding that lowers `passed`, a finding that does not, and the clean case>
</example>

## Workflow

1. <step: read what, from where>
2. <step: decide what, against what>
3. <step: emit the object above>
````

Six rules bind every agent file.

1. **Zero ceremony.** Conventions §2 applies to an agent file as it applies to a SKILL.md and a command. No sentence whose function is to make the model check itself, no numbered validation list, no "review your output for", no status-transition instruction, no file-existence check, and no instruction to run tests, coverage, or linters and interpret the numbers. Each of those is a CLI call made by a hook or by a command preamble. The `## Workflow` section names reads and decisions; it names no step whose only effect is the model inspecting its own output.
2. **Closed, minimal tools, and no tool that cannot reach the agent.** The `tools` value is a list, not a wildcard, and it holds exactly the tools the `## Workflow` steps use. Read-only by default: 34 of the 46 hold `Read` with some of `Grep`, `Glob`, `WebSearch`, `WebFetch`, and `Skill`, and no write tool. **`AskUserQuestion` appears in no agent file.** A subagent has no `AskUserQuestion` tool whatever its `tools` list holds — the harness strips it, along with `Agent` at the depth limit, `EndConversation`, `EnterPlanMode`, `ExitPlanMode`, `ScheduleWakeup`, `TaskOutput`, `WaitForMcpServers`, and `Workflow` — so a question belongs in the owning skill, which asks it in the main conversation and passes the answer to the agent as an input field. An agent that needs an answer returns the candidate options and is invoked a second time with the selection. The same rule covers MCP tools: a subagent's tool set is its `tools` list plus its `mcpServers`, so an agent cannot reach a plugin's MCP tools by loading that plugin's prerequisite skill, and a capability of that shape belongs to the skill with its result passed in as a field.

   The 12 that hold `Write` or `Edit` are the ones whose output is a file rather than a report: `prototype-builder`, `ac-test-writer`, `backend-implementer`, `frontend-implementer`, `refactor-surgeon`, `integration-test-writer`, `mockup-designer`, `brand-designer`, `ui-spec-writer`, `deploy-manifest-writer`, `api-doc-writer`, `guide-writer`. Two of the twelve write a conventions §5 document under `.devforgeai/` — `brand-designer` writes `brand/tokens.json` and `ui-spec-writer` writes `ui-specs/UI-nnn.md`, both named as Design's outputs in §5 — and both pass through the `PreToolUse` `doc validate --producer-check` and `design lint` hooks of §7 exactly as the skill's own writes do. The other ten write source, test, config, manifest, documentation, or throwaway-prototype files outside `.devforgeai/`.

   Two hold `Bash`: `dead-code-detector` and `code-quality-auditor`, each to run one optional `config.toml` command. The scope is a `PreToolUse` handler rather than a prefix over the tool name — the handler admits the `[verify].call_graph_command` and `[verify].metrics_command` strings and denies every other command — because a `Bash(devforgeai:*)` scope is wrong in both directions: it denies the project command each agent exists to run, and it admits every state-changing subcommand of the framework binary. `specs/00-conventions.md` §1 rule 1 puts enforcement in the hook layer, and a `PreToolUse` deny cannot be bypassed by permission mode. No other agent file holds `Bash`, and no agent file names a language, package manager, test runner, or build tool, per §1 rule 2.
3. **A registered verifier emits one envelope with its own fields under `payload`.** An agent whose catalog entry reads `registered_verifier: yes` prints **one** JSON object, not two, matching the `devforgeai/verifier/1` contract of `specs/01-cli.md` `## Subagents`: `schema`, `subagent`, `id`, `passed`, `total`, `unit`, and `findings[]` whose entries carry `id`, `severity` from the closed enum `block | warn | info`, `summary`, and `evidence`. Two concatenated objects are `DFA-E410`, which writes the block at `status: unparsed` and fails every gate check reading it with `DFA-E316`, so an agent file shows one fenced object and its `## Workflow` closes with "print the object above and stop".

   Every top-level field the agent adds of its own goes under a `payload` key, and `payload` is present on every registered verifier, `{}` when the agent adds nothing. A gate reading such a field reads it at `verifiers.<field>.payload.<name>`. Findings may carry further keys beside the contract four, which `report ingest` copies through unread: `confidence` on every review and audit agent, Verify's `category`, `file`, `line`, `relates_to`, `measured`, `limit` and `owasp`, Build's `reason`, `kind`, `path` and `line`, and Release's `kind`, `story` and `reason`. No agent carries a `truncated` flag and no agent carries a `blocks_deployment` boolean: the first was a recall cap nothing downstream read, and the second was a gate decision, which belongs to the CLI.

   `total` and `unit` come from the agent's own counting rule, stated in its `## Output` prose. **`passed` is `total` minus the count of units carrying a `block` finding, and nothing else moves it: a `warn` or an `info` finding lands in the report and leaves `passed` where it stands.** That is `specs/07-verify.md` `## Send-back`'s rule held inside the arithmetic rather than only in the prose. Two consequences follow, and an agent that meets either states it: an agent whose findings are all `warn` reports `passed` equal to `total` and moves its count into `payload`, where a project that wants it to gate adds a `report_metric` check; an agent registered against a `verifier_pass` check needs at least one `block` case, or the check cannot fail under any input and the gate slot it occupies gates nothing.

   An agent whose entry reads `registered_verifier: no` prints the schema of its own catalog entry with no envelope and no `payload`, because no `SubagentStop` ingest reads it.
4. **One file, no references.** Conventions §3 lays the directory out as `agents\<agent-name>.md`. Content an agent would otherwise load from a sibling directory lives either inside the one file or in the owning skill's `references/`, which the skill puts into the prompt.
5. **Optional frontmatter keys, each with a stated reason.** The four required keys are joined by four optional ones, in the order the template gives. `disallowedTools: [Agent]` on the 34 read-only agents: omitting `Agent` from an allowlist is the current defence and depends on the allowlist staying an allowlist, while a top-level subagent is not at the nesting depth limit and so does not have `Agent` stripped for it. `maxTurns: 40` on the 20 registered verifiers: `/verify` sends seven in one message and each walks a file set, so the batch needs a ceiling. `memory: project` on the four `improving-framework` agents: the phase exists to accumulate readings across windows, and without it each `/reflect` forgets the last one. `effort: low` on `debt-aggregator`, `sprint-sequencer`, `observation-miner`, `api-doc-writer`, and `epic-grouper`: each groups, orders, or tabulates figures the CLI already computed.

   `isolation: worktree` appears on no agent file, and the reason is recorded here so nobody adds it later: the target project's working tree is the framework's own model — `.devforgeai/` state, `config.toml`, `gates.toml`, `state.toml`, the six context files and the story file set all live there, and `doc validate --producer-check` and `story files --check` fire against it — so a worktree-isolated agent would write into a branch the hooks, the gate and the next phase do not read, and the two `Bash` holders' commands would be refused outright.
6. **A review or audit agent reports for recall, not for precision.** The fourteen agents whose output is a judgment about someone else's work — `ac-compliance-verifier`, `standards-reviewer`, `anti-pattern-scanner`, `constraint-auditor`, `coverage-gap-auditor`, `dead-code-detector`, `deferral-validator`, `security-auditor`, `code-quality-auditor`, `adr-conformance-reviewer`, `architecture-reviewer`, `alignment-auditor`, `story-invest-auditor`, `kill-case-builder` — close their opening paragraph with one sentence to that effect: report every finding the reading supports, including the uncertain and the low-severity ones, each carrying its own `severity` and a `confidence` from `0.0` to `1.0`, because the gate, the QA document and the user's remedy run are what filter and a finding dropped in the agent is not filtered but lost. `specs/ANTHROPIC-GUIDANCE.md` §5 records that this outperforms asking for high-severity findings only, on both models these files use.

   The rule has a negative half. No agent file caps its finding count, and none carries a flag saying it stopped early. No agent file drops a candidate before it reaches `findings[]`: a reading that would once have "raised no finding" is reported at `warn` or `info` with the `confidence` it deserves and the reason for the discount in `evidence`. An agent whose id band would otherwise bind its report takes a wider band instead, which is why `skills/validating-quality/agents.md` allocates 100 ids per agent rather than 20.

### templates/agents.md

One file per skill at `skills/<skill-name>/agents.md`. It carries four H2 sections and no others.

````markdown
# Subagents · <Skill Name>

## Invocation order

| # | Workflow step | Agent | Batch | Registered verifier |
|---|---|---|---|---|
| 1 | <step id> | `<name>` | alone \| batch <k> of <n> \| serial, once per <unit> | no \| yes · `verifiers.<field>` |

A row whose Batch cell names a batch is invoked in the same message as every other row naming that batch.
A row with a condition carries it in the Batch cell after a semicolon.

## Contracts

| Agent | Model | Tools | Input fields | Output schema |
|---|---|---|---|---|
| `<name>` | <opus \| sonnet \| inherit> | `<tool>`, `<tool>` | <the field names the prompt carries> | `agents/<name>.md` `## Output` |

## Registered verifiers

| Agent | phase | report_field | unit | required |
|---|---|---|---|---|
| `<name>` | <phase> | `verifiers.<field>` | <unit word> | true \| false |

The SubagentStop hook runs `devforgeai report ingest <name> -` for each row above.
A name absent from `config.toml` `[[verifier]]` is a no-op with exit 0 and `DFA-W411`.

## Shared lineage

| Agent | derives_from | Other skills deriving from the same file |
|---|---|---|
| `<name>` | `<existing file>` | `<skill>` · `<their agent name>` |
````

## Evals

None. Conventions §9 requires the three eval artifacts of every skill, and this document is not a skill: it ships no `SKILL.md`, no slash command, and no prompt for `claude -p` to run. The behaviour a grader would check here — that every registered verifier's stdout parses as `devforgeai/verifier/1` and lands at its `report_field` — is covered by the `report ingest` integration tests of `specs/01-cli.md` `## Evals` (`ingest_writes_verifier_block`, `ingest_replaces_previous_block_same_subagent`, `ingest_merges_findings_by_id`, `ingest_unknown_subagent_gives_w411_exit_zero`, `ingest_unparsable_gives_e410_and_status_unparsed`) and by each skill's own `cases.jsonl`.

## Decisions

1. **The `[[verifier]]` registry holds 20 entries in v1, not 6. Amendment to `specs/01-cli.md` `## Gate`, closing paragraph.** That paragraph reads "holds six entries in v1, written by `init` from the shipped skill specs" and names `kill-case-builder`, `flow-integrity-auditor`, `architecture-reviewer`, `alignment-auditor`, `requirement-coverage-auditor`, and `ac-compliance-verifier`. Fourteen further entries were declared by skill specs written after it, each with `registered_verifier: yes` and each with a `[[verifier]]` table written verbatim in its own spec: `story-invest-auditor` (plan, `specs/05-plan.md` §CLI calls); `ac-test-writer`, `story-ac-verifier`, `context-validator` (build, `specs/06-build.md` §Templates); `standards-reviewer`, `anti-pattern-scanner`, `constraint-auditor`, `coverage-gap-auditor`, `dead-code-detector`, `deferral-validator`, `security-auditor`, `code-quality-auditor`, `adr-conformance-reviewer` (verify, `specs/07-verify.md` §CLI calls); `deferral-auditor` (release, `specs/09-release.md` §Gate). The amendment replaces "six" with "twenty" and the six-name list with the array below. Without it, `init` writes six tables, fourteen `SubagentStop` firings return `DFA-W411` with nothing written, and every gate check naming one of them fails with `DFA-E316`.

   The array `devforgeai init` writes into `<target>\.devforgeai\config.toml`, in this order, which is the order the handoff tie-break rule of `specs/06-build.md` §Decisions 40 reads:

```toml
[[verifier]]
name = "kill-case-builder"
phase = "explore"
report_field = "verifiers.kill_case"
unit = "objections"
required = true

[[verifier]]
name = "flow-integrity-auditor"
phase = "discover"
report_field = "verifiers.flow_integrity"
unit = "flows"
required = false

[[verifier]]
name = "architecture-reviewer"
phase = "constitute"
report_field = "verifiers.architecture_reviewer"
unit = "requirements"
required = false

[[verifier]]
name = "alignment-auditor"
phase = "constitute"
report_field = "verifiers.alignment_auditor"
unit = "checks"
required = false

[[verifier]]
name = "story-invest-auditor"
phase = "plan"
report_field = "verifiers.story_invest"
unit = "stories"
required = true

[[verifier]]
name = "ac-test-writer"
phase = "build"
report_field = "verifiers.ac_testable"
unit = "AC"
required = true

[[verifier]]
name = "story-ac-verifier"
phase = "build"
report_field = "verifiers.story_ac"
unit = "ACs"
required = true

[[verifier]]
name = "context-validator"
phase = "build"
report_field = "verifiers.context"
unit = "files"
required = true

[[verifier]]
name = "ac-compliance-verifier"
phase = "verify"
report_field = "verifiers.ac_compliance"
unit = "ACs"
required = true

[[verifier]]
name = "standards-reviewer"
phase = "verify"
report_field = "verifiers.standards"
unit = "files"
required = true

[[verifier]]
name = "anti-pattern-scanner"
phase = "verify"
report_field = "verifiers.anti_patterns"
unit = "anti-patterns"
required = true

[[verifier]]
name = "constraint-auditor"
phase = "verify"
report_field = "verifiers.constraints"
unit = "constraints"
required = true

[[verifier]]
name = "coverage-gap-auditor"
phase = "verify"
report_field = "verifiers.coverage_gaps"
unit = "layers"
required = true

[[verifier]]
name = "dead-code-detector"
phase = "verify"
report_field = "verifiers.dead_code"
unit = "symbols"
required = true

[[verifier]]
name = "deferral-validator"
phase = "verify"
report_field = "verifiers.deferrals"
unit = "deferrals"
required = true

[[verifier]]
name = "security-auditor"
phase = "verify"
report_field = "verifiers.security"
unit = "OWASP categories"
required = true

[[verifier]]
name = "code-quality-auditor"
phase = "verify"
report_field = "verifiers.quality"
unit = "files"
required = true

[[verifier]]
name = "adr-conformance-reviewer"
phase = "verify"
report_field = "verifiers.adr_conformance"
unit = "ADRs"
required = true

[[verifier]]
name = "requirement-coverage-auditor"
phase = "design"
report_field = "verifiers.requirement_coverage"
unit = "screens"
required = false

[[verifier]]
name = "deferral-auditor"
phase = "release"
report_field = "verifiers.deferrals"
unit = "deferrals"
required = true
```

   Every `name`, `phase`, `report_field`, and `unit` above is unchanged by the envelope rewrite of `## Templates` rule 3, checked entry by entry: the twenty agents count the same units they counted before — checks, flows, requirements, screens, symbols, files, ACs, constraints, layers, deferrals, objections, stories, ADRs, anti-patterns, OWASP categories — and no `report_field` string moves. What the rewrite changes is one level below the registry: an agent's own top-level fields now sit under a `payload` key of the envelope, so a `report_metric` gate check that read `verifiers.<field>.<name>` reads `verifiers.<field>.payload.<name>`. Three checks are affected, all in the constitute gate: `verifiers.architecture_reviewer.payload.blocking_findings`, `verifiers.architecture_reviewer.payload.send_back_requirements.length`, and `verifiers.alignment_auditor.payload.blocking_findings`. `verifier_pass` checks are unaffected, because `passed`, `total`, and `unit` stay at the top level of the envelope where the CLI reads them.

   Sixteen entries carry `required = true` and four carry `required = false`. The flag's meaning is the one `specs/01-cli.md` `## Outputs` gives it: "a `verifier_pass` check names it by `name`". The four at `false` are the four no `verifier_pass` check names — Constitute's two, which its gate reads through three `report_metric` checks; Discover's one, whose gate carries the four kinds `specs/03-discover.md` §Decisions 16 lists and no `verifier_pass`; and Design's one, whose phase has no gate at all per `specs/08-design.md` §Decisions 3. All four blocks are still ingested, still fill the handoff `Verified` line, and are still read by Reflect.
2. **`report_field` is an authored string per registry entry, not a value derived from the agent name. Amendment to `specs/04-constitute.md` `## Gate`.** That section reads "a subagent block ingested by `report ingest <subagent> <source>` is read under `verifiers.<subagent name with hyphens as underscores>`". The authoritative shape is `specs/01-cli.md` `## Outputs`, where `[[verifier]].report_field` is a declared string and the shipped example maps `ac-compliance-verifier` to `verifiers.ac_compliance` — a value the derivation rule does not produce. Sixteen of the eighteen values the skill specs wrote are likewise shorter than the agent name: `kill_case`, `story_invest`, `ac_testable`, `story_ac`, `context`, `standards`, `anti_patterns`, `constraints`, `coverage_gaps`, `dead_code`, `deferrals`, `security`, `quality`, `adr_conformance`. Constitute's own two values, `architecture_reviewer` and `alignment_auditor`, satisfy both readings, so no existing value changes; the amendment replaces the derivation sentence with a reference to the entry's declared `report_field`.
3. **Four registry entries were left without a `[[verifier]]` table by their owning spec, and are assigned here.** `specs/03-discover.md`, `specs/04-constitute.md`, and `specs/08-design.md` each declare `registered_verifier: yes` without writing the table. Constitute fixes its two `report_field` values indirectly, through the metric paths of its three `report_metric` gate checks, and fixes neither `unit` nor `required`; Discover and Design fix none of the four fields. The assignments, all four carried into the array of Decision 1:

| Agent | phase | report_field | unit | required | Reason for the assigned values |
|---|---|---|---|---|---|
| `flow-integrity-auditor` | `discover` | `verifiers.flow_integrity` | `flows` | `false` | `report_field` follows the shortened-name style of the sixteen declared values. `unit` is `flows` because `specs/03-discover.md` `## Handoff` renders `Verified  flow-integrity-auditor · 9/9 flows` verbatim. `required` is `false` because that spec's §Decisions 16 lists the discover gate's four check kinds and `verifier_pass` is not among them |
| `architecture-reviewer` | `constitute` | `verifiers.architecture_reviewer` | `requirements` | `false` | `report_field` is fixed by the gate metric path `verifiers.architecture_reviewer.payload.blocking_findings`. `unit` is `requirements` because the agent's own `payload.requirements_reviewed` field is the count its ratio is drawn from. `required` is `false` because the gate reads it through `report_metric` and no `verifier_pass` names it |
| `alignment-auditor` | `constitute` | `verifiers.alignment_auditor` | `checks` | `false` | `report_field` is fixed by the gate metric path `verifiers.alignment_auditor.payload.blocking_findings`. `unit` is `checks`, the `config.toml` default, because the agent's own count field is `payload.checks_run`. `required` is `false` for the same reason as the row above |
| `requirement-coverage-auditor` | `design` | `verifiers.requirement_coverage` | `screens` | `false` | `report_field` follows the shortened-name style. `unit` is `screens` because `specs/08-design.md` `## Handoff` renders `Verified  requirement-coverage-auditor · 3/3 screens` verbatim. `required` is `false` because that spec's §Decisions 3 leaves `gate check --phase design` undefined, so no check of any kind names it |
4. **`derives_from` and `disposition` are independent fields and disagree on two rows by design.** `derives_from` is a conventions §10 field the owning skill spec sets; it records which existing file the spec author read, and this catalog carries it unchanged. `disposition` is this catalog's four-value enum for what became of the existing file. `api-designer.md` is `derives_from` for `story-file-set-planner` and `replaced` here, on `specs/05-plan.md`'s own words "replaced, not adapted". `ui-spec-formatter.md` is `derives_from` for `ui-spec-writer` and `replaced` here, on `specs/08-design.md` `## Decisions` 21. Where an owning spec gave a word, that word sets `disposition`; where no spec gave a word, or the word named no successor, this catalog decides and says so in a numbered decision below.
5. **Zero of the 46 names collide, and that is the resolution of all five collisions the catalog was asked to check.** Each pair was diverged by the later spec at the point it was written, and the shared thing in every case is lineage, not name. One row per pair:

| Pair | Status | Resolution and the spec that made it |
|---|---|---|
| `context-validator` (Build) vs Verify's constraint agent | different agents, no collision | Verify named its agent `constraint-auditor`; Build kept `context-validator`. Both carry `derives_from: context-validator.md`. Build narrows to the six context files on the points no CLI check covers and runs pre-commit; Verify narrows to `architecture-constraints.md` alone and adds the `spec-gap` outcome. `specs/07-verify.md` §Decisions 29, §Subagents. No spec changes |
| `deferral-validator` (Verify) vs `deferral-auditor` (Release) | different agents, no collision | Two registrations of one lineage, anticipated by `specs/09-release.md` §Decisions 31 and settled by `specs/07-verify.md` §Decisions 29. Verify asks whether a deferral is justified; Release asks whether it blocks deployment on `platform.target`. They share the `report_field` string `verifiers.deferrals` legitimately: `specs/01-cli.md` `## Outputs` makes `name` unique and leaves `report_field` free, and the two write into `reports/STORY-nnn-verify.yaml` and `reports/vX.Y.Z-release.yaml`. No spec changes |
| `ac-compliance-verifier` (Verify) vs `story-ac-verifier` (Build) | different agents, no collision | `specs/06-build.md` §Decisions 20 renamed Build's agent, because a `[[verifier]]` table binds one `name` to one `phase` and `SubagentStop` calls `report ingest <subagent> -` with no `--phase`. Build reads three prompt fields in a fresh context and gates `build-acs`; Verify reads the story and the file set and gates `verify-acs`. No spec changes |
| `architecture-reviewer` (Constitute) vs `adr-conformance-reviewer` (Verify) | different agents, no collision | `specs/07-verify.md` §Decisions 29 renamed Verify's agent. Both carry `derives_from: architect-reviewer.md`. Constitute asks whether the stack and constraint set can realize every REQ; Verify asks whether one story's code conforms to the accepted ADRs. Verify's drops `WebFetch` and `AskUserQuestion`, because a registered verifier runs unattended. No spec changes |
| Any agent named by both Explore and Discover | none exists | Explore owns `idea-interrogator`, `landscape-scanner`, `flow-drafter`, `prototype-builder`, `kill-case-builder`; Discover owns `persona-mapper`, `requirement-drafter`, `epic-grouper`, `flow-integrity-auditor`. The two sets are disjoint. `specs/02-explore.md` §Decisions 22 chose explore-scoped names for exactly this reason. The shared item is the `derives_from` file `requirements-analyst.md`, which fans out to `flow-drafter` (Explore), `requirement-drafter` and `epic-grouper` (Discover), and `story-invest-auditor` (Plan) — four agents, three skills, four contracts. No spec changes |

6. **`agent-generator.md` is `retired`, not `replaced`. Cosmetic amendment to `specs/10-reflect.md` `## Subagents`.** That table labels it "replaced" and then states that no step in the skill runs it, naming no successor agent. The closed enum here reserves `replaced` for a file whose slot a catalog agent occupies. Nothing in the framework generates an agent file at run time: the 46 files are built from the `## Templates` agent-file template at release time, and a Reflect recommendation to change a subagent is a `subagent` row naming the file and the change, applied by the user.
7. **`context-preservation-validator.md.bak` is `retired`. No spec mentioned it.** It is a byte-for-byte older copy of `context-preservation-validator.md` with the same `name:` value in its frontmatter. Claude Code loads `*.md` and not `*.md.bak`, so it is not an agent today; it ships nowhere and nothing derives from it.
8. **`humanizer-grader.md` is `retired`. No spec mentioned it.** It grades a passage for human against LLM authorship on a ten-metric rubric and writes a self-contained HTML report to a path it chooses. No conventions §5 document is graded that way, no `gates.toml` check kind reads such a score, and its `Write` to a self-chosen path lies outside the §3 layout.
9. **Three files carried contradicting dispositions across specs; the owning spec wins in each case.**
   - `context-preservation-validator.md`: `specs/03-discover.md` §Decisions 18 adapts it into `flow-integrity-auditor`; `specs/05-plan.md` §Subagents lists it "retired". Discover owns it, and the catalog records `adapted`. Amendment to `specs/05-plan.md`: its row reads "not invoked here", the phrasing `specs/04-constitute.md` used for the same situation, with the same reason text.
   - `ui-spec-formatter.md`: `specs/08-design.md` §Decisions 21 replaces it with `ui-spec-writer`; `specs/05-plan.md` lists it "retired" and `specs/02-explore.md` §Decisions 21 lists it "not invoked". Design owns it, and the catalog records `replaced`. Amendment to `specs/05-plan.md`: "not invoked here".
   - `diagnostic-analyst.md`: `specs/04-constitute.md` §Decisions 17 lists it "not invoked here"; `specs/10-reflect.md` §Subagents lists it "replaced" and names Constitute's `alignment-auditor` and Verify's `anti-pattern-scanner` as the two agents that carry the work. The catalog records `replaced` with both names, which is the only spec statement that named a successor.
10. **`specs/02-explore.md` §Decisions 22 carries a stale reason. Cosmetic amendment.** It justifies explore-scoped names by writing that "Discover invokes `requirements-analyst`, `business-coach`, and `internet-sleuth` with different contracts". `specs/03-discover.md` §Decisions 18, written later, lists `business-coach` and `internet-sleuth` as unused by Discover. The conclusion stands unchanged — the five Explore agents keep explore-scoped names, and the catalog has no collision to resolve — and the reason text narrows to `requirements-analyst` alone. No contract moves.
11. **All 70 companion reference files are `reference-only`, and 54 of the 70 are dropped.** Conventions §3 lays the framework's agent directory out as `agents\<agent-name>.md`: a flat file with no companion directory. A reference file therefore has two possible futures, and each row of Table B names one. The specs named nine of the 70 individually — six external-tool pattern files in `specs/07-verify.md` §Decisions 27 and `specs/06-build.md` §Decisions 23, `security-auditor\references\owasp-patterns.md` in `specs/07-verify.md` §Subagents, `deployment-engineer\references\platform-patterns.md` in `specs/09-release.md` §Decisions 26, and `references\treelint-search-patterns.md` with the other five. The other 61 are assigned here by three tests applied in order: a file that names a language, package manager, test runner, build tool, or external binary is dropped under §1 rule 2; a file whose parent is `retired` or `replaced` is dropped with its parent; a file whose content is tool-neutral judgment the adapted contract still reads becomes `references/` content of the owning skill at the path the row gives. Sixteen files pass the third test.
12. **Each entry's `output` block is the agent file's own, and the entry says which of the two notations it is.** Twenty-four agent files fix their output as a JSON Schema with `type`, `required`, and `properties`; twenty-two fix it by worked example, one instance per case with the field names and sample values. Both satisfy conventions §10, which asks for "a JSON schema" and fixes no notation, and both are read by the skill and by `report ingest` against the same key set. The catalog copies whichever the agent file carries and labels it, rather than paraphrasing one into the other: a paraphrase is a second place the contract lives, and the two drift the first time one is edited.

   The `tools` field uses one notation across all 46 entries: a comma-separated list of backticked tool names, in the order the agent file's frontmatter list gives them. Where an agent's shell access is bounded, the bound is stated in prose and is a `PreToolUse` handler rather than a `Bash(prefix:*)` spelling, because the permitted command is a `config.toml` value the project owns.
13. **`inherit` is used by no agent, and the enum stays three-valued.** Conventions §10 fixes `model` as `sonnet | opus | inherit` with a one-line reason. No spec chose `inherit`: `specs/07-verify.md` §Subagents moved `dead-code-detector` from the existing agent's `inherit` to `sonnet` with the reason "so the judgement does not change with the caller", which is the argument that holds for all 46 — an agent whose model varies with its caller has a contract that varies with its caller, and every one of these contracts is read by a gate, a skill step, or `report ingest`. The v1 population is 25 `opus` and 21 `sonnet`. A future entry that justifies `inherit` states the justification in its own skill spec, per §10.
14. **Two line-level scan artifacts, both now out of scope by rule rather than by argument.** First, one Table B row carries the existing path `code-reviewer\references\review-checklist.md`, whose last segment a naive scan matches. It appears verbatim because a disposition ledger that renamed a path would not resolve against the directory it describes. Under the scoped §2 rule the token is exempt twice over — clause 5, a path segment, and clause 4, a table cell in a path column — and `scripts/ceremony_scan.py` reports no hit on it, so the paragraph that used to explain the hit is a record of why the scoping rule was written. That file is dropped, so the path survives into neither `agents/` nor `skills/`. Second, the two templates in `## Templates` are fenced with four backticks, and the H2 headings inside them — `## Input`, `## Output`, `## Workflow` in the agent-file template, and `## Invocation order`, `## Contracts`, `## Registered verifiers`, `## Shared lineage` in the agents.md template — are template content rather than sections of this spec. A Markdown parser reads them as code and this document keeps the fourteen H2s conventions §11 fixes; a line-oriented grep over the raw file reports seven more.
15. **Blockers: none.** Every agent in the catalog runs on the §2 primitive list — `Read`, `Write`, `Edit`, `NotebookEdit`, `Bash`, `PowerShell`, `Grep`, `Glob`, `Agent`, `Skill`, `WebFetch`, `WebSearch` — inside a Claude Code terminal session, with the `SubagentStop` hook of §7 and the `devforgeai` binary. `AskUserQuestion` is absent from that list on purpose: it is available to the main session and to no subagent, so it is a primitive the nine skills use and the 46 agents cannot. The framework asks for no capability outside that list. The three amendments this document raises, Decisions 1, 2, and 9, are edits to spec prose and to one `config.toml` default array, and none of them changes a subagent contract.
