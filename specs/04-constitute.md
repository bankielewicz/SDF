---
schema: devforgeai-spec/1
doc: constitute
status: draft
produced_by: constitute-spec-author
consumes: [00-conventions]
open_questions: []
---

# Phase 2 · Constitute · skill `establishing-context`

## Scope

Constitute turns `requirements.yaml` into the project's constitution: six context files under `.devforgeai/context/` and an append-only decision log under `.devforgeai/adr/`. It reads `requirements.yaml` from Discover, the non-goals in `.devforgeai/explore/brief.md` when Explore ran, and `.devforgeai/config.toml` written by `devforgeai stack detect`. For a brownfield project it completes the drafts that `devforgeai init --analyze` left on disk. It records every technology choice, boundary, dependency rule, code standard, and forbidden pattern that later phases read and that Build's hooks enforce. Constraints carry `CON-nnn` IDs, anti-patterns carry `AP-nnn` IDs, decisions carry `ADR-nnn` IDs. The six context files are project singletons: a second cycle run as `/constitute EPIC-008` amends them through new ADRs and new CON rows, and does not rewrite them from scratch.

Constitute does not write stories, acceptance criteria, sprints, UI specs, brand tokens, or code. It does not run tests, linters, or coverage. It does not decide pass criteria; those live in `gates.toml`. It does not detect the stack; `devforgeai stack detect` does that and Constitute records the result. It does not edit `requirements.yaml` or `explore/brief.md`; a defect upstream produces a SEND BACK to Discover citing REQ IDs. It does not scan source code for violations of its own files; `devforgeai context audit`, Build's hooks, and Verify's `anti-pattern-scanner` do that.

## Inputs

| Document | Path | IDs read | Producer | Absence |
|---|---|---|---|---|
| Requirements | `.devforgeai/requirements.yaml` | `IDEA-nnn` (top-level `id`), `EPIC-nnn` (`epics[].id`), `REQ-nnn` (`requirements[].id`), `PERSONA-nnn` (`personas[].id`) | Discover | `gate require constitute <IDEA-nnn>` exits 1 and names the missing Discover gate; the command stops |
| Explore brief | `.devforgeai/explore/brief.md` | `IDEA-nnn` (frontmatter `id`), `FLOW-nnn` (column `ID` of `## Core flows`); sections `## Non-goals`, `## Competitor scan`, `## Technology scan` | Explore | the non-goal CON set from the brief is empty, the technology candidate set is empty, and step 4 reads `config.toml` and the requirement records alone |
| Explore decision | `.devforgeai/explore/decision.yaml` | `IDEA-nnn` (`id`) | Explore | ADR-000 is not written and step 10 proceeds with the remaining decisions |
| Stack config | `.devforgeai/config.toml` | none | `devforgeai stack detect` | SessionStart hook re-runs `stack detect`; a file still absent halts step 2 with the CLI's stderr |
| Brownfield drafts | `.devforgeai/context/*.md` with `status: draft` | file stem IDs | `devforgeai init --analyze` | the greenfield workflow runs instead |
| Constraint under remedy | `.devforgeai/context/architecture-constraints.md`, row `CON-nnn` in `## Constraint index` | `CON-nnn` | this skill | `--remedy` with an unknown CON id exits 3 from `doc validate` and the command stops |

`requirements.yaml` top-level keys, from `specs/03-discover.md` §Outputs, in this order: `schema`, `id`, `phase`, `status`, `produced_by`, `consumes`, `open_questions`, `revision`, `revision_log`, `accepted_by`, `accepted_at`, `personas`, `epics`, `requirements`. This phase reads seven of them.

| Key read | Field | Use here |
|---|---|---|
| `id` | — | the phase instance `<ID>` |
| `status` | — | `gate require constitute` passes on `accepted` alone |
| `revision` | — | copied into the constitute report so Reflect can pair a context set with the requirement revision it was written from |
| `personas[]` | `id`, `name`, `goal` | the actor named in a CON whose `kind` is `process` |
| `epics[]` | `id`, `title`, `scope`, `success_metric`, `requirements` | epic scoping of the handoff `Next` line; `success_metric` supplies the number in a `performance` CON |
| `epics[]` | `out_of_scope` | one `non-goal` CON per entry, `source` = that `EPIC-nnn` |
| `requirements[]` | `id`, `actor`, `statement`, `rationale`, `acceptance_signal`, `priority`, `source`, `status` | `statement` and `acceptance_signal` are the text a CON constrains and an ADR cites; `priority` orders the ADR set; a record with `status: withdrawn` is skipped |

A requirement record carries `statement`, not `text`. Every REQ id this phase cites resolves to a `requirements[].id` whose `status` is `accepted` or `reopened`.

## Outputs

Seven document types, all Markdown with the §5 frontmatter. `<ID>` for this phase is the top-level `id` of `requirements.yaml`, an `IDEA-nnn` (`specs/03-discover.md` §Outputs, key `id`, pattern `^IDEA-\d{3}$`). Constitute runs once per requirements document, which holds every epic for the project, and the six context files are project singletons.

### Frontmatter, six context files

```yaml
---
schema: devforgeai/context-<stem>/1
id: <stem>
phase: constitute
status: draft | accepted
produced_by: establishing-context
consumes: [IDEA-nnn, REQ-nnn, ...]
open_questions: []
---
```

`<stem>` is one of `tech-stack`, `source-tree`, `dependencies`, `coding-standards`, `architecture-constraints`, `anti-patterns`. The `id` is the stem: these six files are singletons and take no allocated number.

### Frontmatter, ADR

```yaml
---
schema: devforgeai/adr/1
id: ADR-nnn
phase: constitute
status: proposed | accepted | rejected | superseded
produced_by: establishing-context
consumes: [REQ-nnn, ...]
open_questions: []
---
```

`consumes` carries the REQ IDs that motivated the decision and is the only place they appear. An ADR that records an observed brownfield decision no requirement motivates carries `consumes: []` and names its evidence path in `## Context`.

### Section lists

Headings are fixed. `devforgeai context audit`, Build's hooks, Verify's `anti-pattern-scanner`, and `devforgeai design lint` locate content by these strings.

| File | Path | H2 sections, in order |
|---|---|---|
| tech-stack.md | `.devforgeai/context/tech-stack.md` | `## Languages`, `## Runtimes`, `## Frameworks`, `## Data stores`, `## Tooling`, `## Excluded technologies` |
| source-tree.md | `.devforgeai/context/source-tree.md` | `## Roots`, `## Layers`, `## Directory map`, `## File placement rules`, `## Naming conventions`, `## Generated and excluded paths` |
| dependencies.md | `.devforgeai/context/dependencies.md` | `## Approved dependencies`, `## Forbidden dependencies`, `## Version policy`, `## License policy`, `## Addition procedure` |
| coding-standards.md | `.devforgeai/context/coding-standards.md` | `## Formatting`, `## Naming`, `## Error handling`, `## Logging`, `## Testing standards`, `## Documentation`, `## Design tokens` |
| architecture-constraints.md | `.devforgeai/context/architecture-constraints.md` | `## Constraints`, `## Layer dependency rules`, `## Constraint index` |
| anti-patterns.md | `.devforgeai/context/anti-patterns.md` | `## Anti-patterns`, `## Anti-pattern index` |
| ADR-nnn.md | `.devforgeai/adr/ADR-nnn.md` | `## Context`, `## Decision`, `## Consequences`, `## Constraints introduced`, `## Supersedes` |

### The key namespace

Any table whose header row is exactly `| Key | Value | Source |` contributes `(key, value, file, line)` triples to the cross-file contradiction check. Keys come from this closed set. `<role>`, `<name>`, `<entity>`, `<n>` are typed wildcards.

| Key | Value domain | Owning file |
|---|---|---|
| `language.primary` | identifier | tech-stack.md |
| `language.primary.version` | version string | tech-stack.md |
| `language.secondary.<n>` | identifier, `<n>` integer | tech-stack.md |
| `language.secondary.<n>.version` | version string | tech-stack.md |
| `runtime.name` | identifier | tech-stack.md |
| `runtime.version` | version string | tech-stack.md |
| `framework.<role>` | identifier; `<role>` in `web api ui orm test build lint format package` | tech-stack.md |
| `framework.<role>.version` | version string | tech-stack.md |
| `datastore.<role>` | identifier; `<role>` in `primary cache search queue blob` | tech-stack.md |
| `datastore.<role>.version` | version string | tech-stack.md |
| `source.root` | path relative to project root | source-tree.md |
| `test.root` | path relative to project root | source-tree.md |
| `build.output.root` | path relative to project root | source-tree.md |
| `layer.<name>.path` | glob | source-tree.md |
| `dep.<name>.version` | version constraint string | dependencies.md |
| `dep.<name>.scope` | one of `runtime dev test build` | dependencies.md |
| `style.indent` | integer | coding-standards.md |
| `style.line.max` | integer | coding-standards.md |
| `style.quote` | one of `single double` | coding-standards.md |
| `naming.<entity>` | one of `camel pascal snake kebab upper-snake`; `<entity>` in `type function variable constant` | coding-standards.md |
| `naming.<entity>` | one of `camel pascal snake kebab upper-snake`; `<entity>` in `file directory test-file` | source-tree.md |
| `tokens.path` | path relative to project root | coding-standards.md |

A key repeated in a second file with the same value passes. A key with two distinct values across the set fails `context audit` check CA-5.

A concrete key matches a wildcard row by segment: `<name>`, `<entity>`, and `<role>` each match exactly one dot-free segment drawn from the row's stated value set where the row states one; `<n>` matches one to three decimal digits. `layer.domain.path` matches `layer.<name>.path`; `dep.serde.version` matches `dep.<name>.version`; `layer.domain.core.path` matches nothing and fails CA-5. The literal wildcard strings appear in the templates as placeholder rows and are absent from a file with `status: accepted`, which CA-2 and CA-5 together enforce.

Constraint status and anti-pattern severity are not keys in this namespace. They live once each, in `## Constraint index` and `## Anti-pattern index`, and checks CA-4, CA-6, and CA-8 read them there.

### Closed enumerations

| Field | Values |
|---|---|
| context file `status` | `draft`, `accepted` |
| ADR `status` | `proposed`, `accepted`, `rejected`, `superseded` |
| CON `kind` | `boundary`, `dependency`, `layering`, `non-goal`, `performance`, `security`, `data`, `process` |
| CON `status` | `active`, `retired` |
| CON `enforced_by` | an `AP-nnn` id, `devforgeai context audit`, `devforgeai design lint`, `devforgeai doc validate`, `none` |
| AP `category` | `library`, `structure`, `layer`, `smell`, `security`, `style` |
| AP `severity` | `blocker`, `high`, `medium`, `low` |
| AP `detector_kind` | `literal`, `regex`, `glob` |
| dependency `scope` | `runtime`, `dev`, `test`, `build` |

## Workflow

Each of steps 4 through 10 reads its `templates/<file>` first and fills it: the section set, the heading text, and the row shapes are the template's, and the step supplies the values. Every one of the six context files and every ADR is one flat document — the seven envelope keys `schema`, `id`, `phase`, `status`, `produced_by`, `consumes`, `open_questions` sit at the top level of the frontmatter, with the body's H2 sections below. Nothing here nests the envelope under a wrapper key.

### Greenfield

1. **CLI** · `devforgeai gate require constitute <IDEA-nnn>` in the command preamble. Input: `state.toml`, `gates.toml`. Output: exit 0. Failure: exit 1 prints the missing Discover gate to stderr and the command stops.
2. **CLI** · `devforgeai doc load requirements <IDEA-nnn>` in the command preamble. Input: `.devforgeai/requirements.yaml`. Output: the document on stdout, in the model's context. Failure: exit 1 with the path that did not resolve; the command stops.
2b. **Model** · `devforgeai phase set constitute --id <IDEA-nnn>` through Bash. Input: `state.toml`. Output: `[current].phase = "constitute"`, `[current].id` and `[active].constitute` set to the IDEA id, `[stop_hook].block_count` reset. Failure: `DFA-E320` exit 1 repeats the missing-predecessor text from step 1 and the workflow stops.
3. **Model** · Read `.devforgeai/config.toml`, `.devforgeai/explore/brief.md`, and `.devforgeai/explore/decision.yaml`. Input: those three paths. Output: the detected stack values, the unnumbered lines under the brief's `## Non-goals`, the rows of `## Competitor scan` and `## Technology scan`, and the decision record, held in context. Failure: `config.toml` absent halts with the CLI's stderr; `brief.md` absent sets the brief non-goal list and the technology candidate set to empty; `decision.yaml` absent skips ADR-000 at step 10; `decision.yaml` carrying `decision: kill` or `decision: park` halts the workflow and prints that value, because Explore promoted nothing.
4. **Model** · Write `.devforgeai/context/tech-stack.md` from the `config.toml` values, one `| Key | Value | Source |` row per detected key with `Source` = `config.toml`. A `## Technology scan` row whose `Maturity` is `established` supplies a candidate for a key `config.toml` left unset, with `Source` = `explore/brief.md ## Technology scan`. Input: step 3. Output: the file with `status: draft`. Failure: a key still unset becomes one `AskUserQuestion` whose closed option list is the `## Technology scan` candidates for that key; an unanswered question stays in `open_questions` and `context audit` check CA-2 reports it.
5. **Model** · Write `.devforgeai/context/source-tree.md`: `source.root`, `test.root`, `build.output.root` from `config.toml`; layers and path globs from the `epics[].scope` and `requirements[].statement` text. Input: steps 2 and 3. Output: the file with `status: draft`. Failure: as step 4.
6. **Model** · Write `.devforgeai/context/dependencies.md` from the manifests `config.toml` names and the `## Technology scan` rows. Input: steps 2 and 3. Output: the file with `status: draft`. Failure: as step 4.
7. **Model** · Write `.devforgeai/context/architecture-constraints.md`, one `### CON-nnn` block per entry of three sources. A `requirements[]` record whose `statement` or `acceptance_signal` names a bound rather than a behavior takes `source` = that `REQ-nnn`, and its `acceptance_signal` supplies the number in the CON statement. An unnumbered line under the brief's `## Non-goals` takes `kind: non-goal` and `source` = `explore/brief.md ## Non-goals`. An `epics[].out_of_scope` entry takes `kind: non-goal` and `source` = that `EPIC-nnn`. IDs come from `devforgeai doc validate --allocate CON`. Input: steps 2 and 3. Output: the file with `status: draft` and a populated `## Constraint index`. Failure: `--allocate` exit 1 halts the step with its stderr.
8. **Model** · Write `.devforgeai/context/coding-standards.md`. Input: steps 2, 4, 7. Output: the file with `status: draft`; `## Design tokens` carries `tokens.path` = `.devforgeai/brand/tokens.json`. Failure: as step 4.
9. **Model** · Write `.devforgeai/context/anti-patterns.md`. Each CON whose statement a text or path pattern observes becomes one `### AP-nnn` block with `source` = that CON id, and the CON's `enforced_by` field is set to the AP id. IDs come from `devforgeai doc validate --allocate AP`. Input: step 7. Output: the file with `status: draft` and a populated `## Anti-pattern index`. Failure: a CON no pattern observes keeps `enforced_by: none` and the `alignment-auditor` reports it at step 11.
10. **Model** · Write `.devforgeai/adr/ADR-000.md` from `decision.yaml`: `consumes: [IDEA-nnn]`, `## Context` quoting the `reason` field, `## Decision` naming `decision: promote` with the `decided_on` date, `## Consequences` one row per `carry_forward` entry whose `consumer` is `establishing-context`, an empty `## Constraints introduced` table, and `## Supersedes` reading `none`. Then write one `.devforgeai/adr/ADR-nnn.md` per decision that closed a choice among alternatives, with `consumes` listing the motivating REQ ids in `priority` order and `## Constraints introduced` listing the CON ids from step 7. IDs above 000 come from `devforgeai doc validate --allocate ADR`. Input: steps 3 through 9. Output: the ADR files with `status: proposed`. Failure: a CON with no introducing ADR and no REQ in `source` is reported by `context audit` check CA-4 at step 13.
11. **Subagents** · `architecture-reviewer` and `alignment-auditor`, in parallel. Input: the paths from steps 4 through 10 and the `requirements[]` records. Output: one JSON object each on stdout; `SubagentStop` runs `devforgeai report ingest <agent_type> -` on the payload's `last_assistant_message`, filing each block under `verifiers.<report_field>` in `.devforgeai/reports/<IDEA-nnn>-constitute.yaml`, with the agent's own fields under `payload`. Failure: output the model can parse but `report ingest` cannot is `DFA-E410`; the `subagent-stop` arm exits 2 with a blocking decision, and its `reason` reaches the subagent as its next instruction, so the envelope is asked for again rather than logged and lost. A block written with `status: unparsed` leaves the gate metric absent and `gate check` exits 1.
12. **User** · Accept the set. Input: the `architecture-reviewer` and `alignment-auditor` findings rendered as one `AskUserQuestion` with options `accept`, `amend`, `send back`. Output: on `accept` the model edits `status: accepted` into the six frontmatters and `status: accepted` into every ADR frontmatter; on `amend` the workflow returns to the step that wrote the file the finding names; on `send back` the workflow goes to §Send-back. Failure: `verifiers.architecture_reviewer.payload.send_back_requirements` non-empty routes to §Send-back without the question.
13. **CLI** · Stop hook runs `devforgeai gate check --phase constitute --id <IDEA-nnn>` then `devforgeai handoff --phase constitute --id <IDEA-nnn>`. Input: the six files, the ADRs, the two ingested reports. Output: `.devforgeai/reports/<IDEA-nnn>-constitute.yaml` and the §6 block. Failure: a FAIL exits 2 with `decision: "block"` and the failing checks in `reason`, under a budget of three blocks per `session_id`; at the last block of that budget the hook exits 0 and the FAIL handoff renders in `systemMessage`. §7 gives the four cases.

### Brownfield

Runs when `.devforgeai/context/tech-stack.md` exists with `status: draft`. Steps 1 through 3 and 11 through 13 are identical. The drafts stay uncommitted across B4 through B8: pre-commit runs `context audit`, which exits 1 while any of the six carries `status: draft`, so the first commit that touches `.devforgeai/context/` follows the acceptance at step 12.

- **B4. Model** · Read all six drafts. Input: `.devforgeai/context/*.md`. Output: the draft values and each file's `open_questions` list held in context. Failure: a draft whose H2 list differs from §Outputs is rewritten to the template heading order with its content preserved under the matching heading; content matching no heading moves to `open_questions`.
- **B5. Subagent** · `source-tree-mapper`. Input: `source.root` from the draft, the layer names the draft proposes. Output: JSON with roots, layers, entry points, internal edges, external dependencies, generated paths, unmapped paths. Failure: a parse failure re-invokes once, then the workflow continues with the draft values and every unfilled field enters `open_questions`.
- **B6. User** · Resolve each `open_questions` entry. Input: one `AskUserQuestion` per entry, options drawn from the `source-tree-mapper` observations plus `leave open`. Output: the answers. Failure: `leave open` keeps the entry in `open_questions` and `context audit` check CA-2 reports it.
- **B7. Model** · Complete the six files from B5 and B6, writing `Source` = `init --analyze` on rows the drafts supplied and `Source` = `source-tree-mapper` on rows B5 supplied. Output: the six files, still `status: draft`. Failure: as step 4.
- **B8. Model** · Write one ADR per decision the existing code already embodies, `status: proposed`, `consumes` listing the REQ ids the decision now serves or `[]`, `## Context` naming the evidence path from B5. Output: the ADR files. Then proceed to step 11.

### Remedy

Runs on `/constitute <IDEA-nnn> --remedy CON-nnn`, the receiving side of a SEND BACK from Plan.

- **R1. CLI and model** · `devforgeai gate require constitute <IDEA-nnn>`, then `devforgeai doc load requirements <IDEA-nnn>`, then `devforgeai phase set constitute --id <IDEA-nnn> --remedy CON-nnn` through Bash, which writes `[constitute].remedy_con`. Failure: as step 1; a `--remedy` list holding more than one CON id is `DFA-E011`, exit 3.
- **R2. Model** · Read the `### CON-nnn` block, its `## Constraint index` row, the ADR its `introduced_by` field names, and the Plan report `.devforgeai/reports/<STORY-nnn>-plan.yaml` the send-back cited. Failure: `CON-nnn` absent from the index halts with that id on stderr.
- **R3. Subagent** · `architecture-reviewer`, scoped to that one CON and the REQ ids in its `source` field. Output: JSON; a finding of kind `overbuilt` or `contradiction` naming the CON supports replacement, an empty finding list upholds it.
- **R4a. Model, replacement path** · Write a new ADR with `status: proposed`, `## Supersedes` naming the introducing ADR and listing `CON-nnn` as retired, and `## Constraints introduced` listing the replacement CON allocated by `devforgeai doc validate --allocate CON`. Edit the superseded ADR frontmatter to `status: superseded`. Edit the `CON-nnn` index row to `status: retired`. Every other CON row is left as it stands. Any AP whose `source` is `CON-nnn` gets its `source` repointed to the replacement CON or its block removed when the replacement CON carries no pattern.
- **R4b. Model, upheld path** · No ADR is written and no file changes. The workflow goes to R6 with the upheld CON id.
- **R5. Subagents** · `architecture-reviewer` and `alignment-auditor` over the whole set, as step 11. A changed CON reaches every file, so both run again.
- **R6. CLI** · Stop hook, as step 13. On the upheld path the gate result is `SEND BACK to Plan` and the handoff `Found` line carries `CON-nnn` with the reviewer's one-sentence statement.

## Subagents

Disposition of the seven existing agents read for this spec.

| Existing agent | Disposition | Reason |
|---|---|---|
| `architect-reviewer.md` | adapted to `architecture-reviewer` | Markdown severity report replaced by JSON; SOLID and pattern catalogs dropped as generic; scope narrowed to REQ-against-architecture feasibility and CON contradiction, which is the only judgment this phase needs |
| `alignment-auditor.md` | adapted, method inverted | The existing agent's exact-text matching is work `devforgeai context audit` performs, so §1.1 moves it to the CLI; the adapted agent does the semantic work the existing one declines |
| `code-analyzer.md` | adapted to `source-tree-mapper` | Documentation-coverage and public-API extraction dropped; layer, root, entry-point, and internal-edge discovery retained because brownfield drafts need them |
| `tech-stack-detector.md` | retired | Detection is `devforgeai stack detect`, which writes `config.toml`; validation of detected values against `tech-stack.md` is `devforgeai context audit` check CA-5. §1.1 leaves the agent nothing to do |
| `api-designer.md` | not invoked here | An API style choice is an ADR this skill writes; endpoint contracts belong to Plan, which owns the agent |
| `context-validator.md` | not invoked here | It validates source code against the six files, which is Build's hook path and Verify's `anti-pattern-scanner` |
| `diagnostic-analyst.md` | not invoked here | It investigates a failure that already happened, which is Build and Verify territory |

### architecture-reviewer

- **name**: architecture-reviewer
- **derives_from**: `C:\Users\bryan\.claude\agents\architect-reviewer.md`
- **purpose**: Decide whether the accepted stack and constraint set can realize every REQ, and whether any two REQs force opposed constraints.
- **tools**: Read, Grep, Glob
- **model**: opus — the finding turns on reasoning across a requirement set and a constraint set at once.
- **input**: the six context file paths, the ADR directory path, the `requirements[]` records as `{id, actor, statement, rationale, acceptance_signal, priority, status}` with `status: withdrawn` records omitted, the `epics[]` records as `{id, scope, out_of_scope, success_metric, requirements}`, and on the remedy path a single CON id to scope to.
- **output**:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "architecture-reviewer",
  "id": "IDEA-nnn",
  "passed": 22,
  "total": 23,
  "unit": "requirements",
  "findings": [
    {
      "id": "REQ-nnn",
      "severity": "block | warn | info",
      "confidence": 0.85,
      "kind": "infeasible | contradiction | unsupported | overbuilt",
      "requirements": ["REQ-nnn"],
      "constraints": ["CON-nnn"],
      "adrs": ["ADR-nnn"],
      "statement": "one sentence",
      "summary": "one line",
      "evidence": "path:line"
    }
  ],
  "payload": {
    "requirements_reviewed": 23,
    "send_back_requirements": ["REQ-nnn"],
    "blocking_findings": 1
  }
}
```

  One object, never two: the `devforgeai/verifier/1` envelope with this agent's own fields under `payload`. `passed` and `total` count requirements — `total` is every requirement reviewed, `passed` is that count minus the requirements a `severity: block` finding names. A `warn` finding never lowers `passed`. `severity` is the envelope's closed enum `block | warn | info`, which is what `report ingest` parses; the four-value `blocker | high | medium | low` vocabulary of the context documents is a property of a CON row rather than of a finding. `confidence` is `0.0` to `1.0` and is reported on every finding, including a low-confidence one: the agent reports everything it has and the CLI and the Verify phase filter.

  `kind` values: `infeasible` — no architecture under the accepted stack realizes the named REQ; `contradiction` — the named REQs force opposed constraints; `unsupported` — an ADR decision no REQ motivates; `overbuilt` — a CON stricter than any REQ asks. `payload.send_back_requirements` holds the union of `requirements` over findings of kind `infeasible` and `contradiction`. `payload.blocking_findings` counts findings at `severity: block`. `report ingest architecture-reviewer -` files this block under `verifiers.architecture_reviewer` in `.devforgeai/reports/<IDEA-nnn>-constitute.yaml`, so the gate reads `verifiers.architecture_reviewer.payload.blocking_findings` and `verifiers.architecture_reviewer.payload.send_back_requirements.length`.
- **invoked_at**: step 11, in parallel with `alignment-auditor`; R3 and R5 on the remedy path.
- **registered_verifier**: yes

### alignment-auditor

- **name**: alignment-auditor
- **derives_from**: `C:\Users\bryan\.claude\agents\alignment-auditor.md`
- **purpose**: Find semantic disagreement across the six files and the ADR log that exact matching cannot see.
- **tools**: Read, Grep, Glob
- **model**: sonnet — paraphrase comparison over a bounded file set.
- **input**: the six context file paths, the ADR directory path, the `## Constraint index` rows, the `## Anti-pattern index` rows.
- **output**:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "alignment-auditor",
  "id": "IDEA-nnn",
  "passed": 30,
  "total": 31,
  "unit": "checks",
  "findings": [
    {
      "id": "CON-nnn",
      "severity": "block | warn | info",
      "confidence": 0.9,
      "kind": "restated | contradicted | unobservable | unpropagated | orphaned",
      "left": {"file": "path", "line": 0, "text": "quoted line"},
      "right": {"file": "path", "line": 0, "text": "quoted line"},
      "ids": ["CON-nnn"],
      "resolution": "one sentence naming the file and the H2 section that changes",
      "summary": "one line",
      "evidence": "path:line"
    }
  ],
  "payload": { "checks_run": 31, "blocking_findings": 1 }
}
```

  One object, the same envelope. `total` is the number of pairs compared and `passed` is that count minus the pairs a `severity: block` finding names, so a `restated` finding at `warn` leaves `passed` where it was. `confidence` rides on every finding. The gate reads `verifiers.alignment_auditor.payload.blocking_findings`.

  `kind` values: `restated` — one rule worded two ways in two files, where a future edit to one leaves the other stale; `contradicted` — two files assert opposed rules that share no key and so escape CA-5; `unobservable` — a CON whose `enforced_by` names an AP whose detector does not observe the CON's statement, or whose `enforced_by` is `none`; `unpropagated` — an accepted ADR decision absent from the file its `## Constraints introduced` row points at; `orphaned` — an AP whose `source` CON has `status: retired`. `resolution` names a mutable target: a context file section or a new ADR, and not an existing ADR body.
- **invoked_at**: step 11, in parallel with `architecture-reviewer`; R5 on the remedy path.
- **registered_verifier**: yes

### source-tree-mapper

- **name**: source-tree-mapper
- **derives_from**: `C:\Users\bryan\.claude\agents\code-analyzer.md`
- **purpose**: Read an existing codebase and report the roots, layers, entry points, internal edges, and declared dependencies that the brownfield drafts leave open.
- **tools**: Read, Grep, Glob
- **model**: sonnet — file-structure enumeration with fixed output shape.
- **input**: `source.root` from the draft `source-tree.md`, the layer names the draft proposes, the manifest paths from the draft `dependencies.md`.
- **output**:

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

## Command

The entry point is the skill itself: `skills/establishing-context/SKILL.md`, installed to `.claude/skills/constitute/`, whose frontmatter `name` is the slash command. There is no command file. This is the frontmatter and the preamble, verbatim, with two preamble lines.

```markdown
---
name: constitute
description: Phase 2 of DevForgeAI. Turns an accepted requirements.yaml into the project's constitution - the six context files under .devforgeai/context/ (tech-stack, source-tree, dependencies, coding-standards, architecture-constraints, anti-patterns) and the append-only decision log under .devforgeai/adr/ - allocating CON-nnn constraints, AP-nnn anti-patterns and ADR-nnn decisions, and completing the drafts devforgeai init --analyze leaves on a brownfield project. Reach for it whenever /constitute runs, whenever someone asks where a technology choice, layering rule, dependency policy, coding standard, forbidden pattern or architecture decision record should live, and whenever a Plan send-back arrives as /constitute IDEA-nnn --remedy CON-nnn. It owns those artifacts and their IDs, so read it before touching anything under .devforgeai/context/ or .devforgeai/adr/.
argument-hint: <IDEA-nnn> [--remedy CON-nnn]
allowed-tools: Bash(devforgeai:*), PowerShell(devforgeai:*), Read, Write, Edit, Glob, Grep, Agent, AskUserQuestion, Skill
disable-model-invocation: true
---

!`devforgeai gate require constitute $ARGUMENTS[0]`
!`devforgeai doc load requirements $ARGUMENTS[0]`
```

The `gate require` line leads, so a failing predecessor gate aborts the invocation before any document loads.

## CLI calls

| Call | Caller | When |
|---|---|---|
| `devforgeai gate require constitute <IDEA-nnn>` | command `!` preamble | steps 1, R1 |
| `devforgeai doc load requirements <IDEA-nnn>` | command `!` preamble | steps 2, R1 |
| `devforgeai phase set constitute --id <IDEA-nnn>` | model, Bash | step 2b |
| `devforgeai phase set constitute --id <IDEA-nnn> --remedy CON-nnn` | model, Bash | R1 |
| `devforgeai doc validate --allocate CON` | model, Bash | steps 7, R4a |
| `devforgeai doc validate --allocate AP` | model, Bash | step 9 |
| `devforgeai doc validate --allocate ADR` | model, Bash | steps 10, B8, R4a |
| `devforgeai doc validate --producer-check <path>` | PreToolUse hook on Write and Edit | every write under `.devforgeai/` |
| `devforgeai doc validate <path>` | PostToolUse hook on Write and Edit | every write under `.devforgeai/` |
| `devforgeai report ingest architecture-reviewer - --id <IDEA-nnn> --phase constitute` | SubagentStop hook | steps 11, R3, R5 |
| `devforgeai report ingest alignment-auditor - --id <IDEA-nnn> --phase constitute` | SubagentStop hook | steps 11, R5 |
| `devforgeai gate check --phase constitute --id <IDEA-nnn>` | Stop hook, pre-push | steps 13, R6 |
| `devforgeai handoff --phase constitute --id <IDEA-nnn>` | Stop hook | steps 13, R6 |
| `devforgeai context audit` | Stop hook (Constitute), pre-commit | steps 13, R6, every commit touching `.devforgeai/` |

### What `devforgeai context audit` checks

Mechanical, exit 0 or 1, one line of stderr per failing check naming the file, line, and id.

| Check | Rule |
|---|---|
| CA-1 | The six paths under `.devforgeai/context/` exist and parse as Markdown with §5 frontmatter in key order |
| CA-2 | Every context file has `status: accepted` and `open_questions: []` |
| CA-3 | Each file's H2 lines equal its §Outputs list, in that order, with no extra H2 |
| CA-4 | Every `CON-nnn` in `## Constraint index` with `status: active` appears in the `## Constraints introduced` table of at least one ADR, or its `source` field holds a `REQ-nnn` that resolves in `requirements.yaml` |
| CA-5 | Across every `\| Key \| Value \| Source \|` table under `.devforgeai/context/`, no key holds two distinct values; keys outside the closed namespace fail |
| CA-6 | Every `AP-nnn` in `## Anti-pattern index` has a non-empty `detector`, a `detector_kind` in the enum, and a `source` CON that exists with `status: active` |
| CA-7 | Every ADR id is unique; every id in an ADR's `consumes` resolves in `requirements.yaml`; every CON in an ADR's `## Constraints introduced` exists in `## Constraint index` |
| CA-8 | Every ADR named in a `## Supersedes` row has `status: superseded`, and every CON that row retires has `status: retired` |

CA-2 places acceptance before the first commit: pre-commit runs `context audit`, so a brownfield project commits its context files once the user has accepted them, and not while they carry `status: draft`.

### What the `alignment-auditor` subagent checks

Judgment, returned as JSON, ingested by `report ingest alignment-auditor -` under `verifiers.alignment_auditor`, read by the gate as `verifiers.alignment_auditor.payload.blocking_findings`. The five `kind` values in its output schema define the checks: `restated`, `contradicted`, `unobservable`, `unpropagated`, `orphaned`. None of the five reduces to string equality, which is what separates them from CA-1 through CA-8.

## Gate

The `[[gate]]` entry for `constitute` in `.devforgeai/gates.toml`, verbatim from `specs/01-cli.md` §Gate:

```toml
[[gate]]
phase = "constitute"
requires = "discover"
on_fail = "fail"
send_back_to = "discover"
description = "The six context files audit clean and both reviewers report no blocking finding."

  [[gate.check]]
  kind = "context_audit"
  id = "CTX-AUDIT"
  allow_warn = false
  message = "context audit failed: {value}"

  [[gate.check]]
  kind = "report_metric"
  id = "REVIEW"
  metric = "verifiers.architecture_reviewer.payload.blocking_findings"
  op = "eq"
  value = 0
  message = "architecture-reviewer reports {value} blocking findings"

  [[gate.check]]
  kind = "report_metric"
  id = "ALIGNMENT"
  metric = "verifiers.alignment_auditor.payload.blocking_findings"
  op = "eq"
  value = 0
  message = "alignment-auditor reports {value} blocking findings"

  [[gate.check]]
  kind = "report_metric"
  id = "SEND-BACK-REQUIREMENTS"
  metric = "verifiers.architecture_reviewer.payload.send_back_requirements.length"
  op = "eq"
  value = 0
  on_fail = "send_back"
  message = "architecture-reviewer sends {value} requirements back to Discover"
```

`context_audit` is the compiled-in required check kind for this phase, carried at the default `severity = "block"`. Metric paths are rooted at the report `specs/01-cli.md` §Outputs defines, so a subagent block ingested by `report ingest <agent_type> -` is read under the `report_field` that agent's `[[verifier]]` entry in `config.toml` names (`verifiers.architecture_reviewer` and `verifiers.alignment_auditor`, per `specs/11-subagent-catalog.md`). The agent's own fields sit under `payload`, so a metric into one of them carries that segment; `passed` and `total` stay at the top of the block, where `verifier_pass` reads them. A `.length` suffix reads a sequence's length. The `SEND-BACK-REQUIREMENTS` check carries `on_fail = "send_back"`, which makes `gate check` exit 2 and route to `send_back_to`; the other three fall to the gate's `on_fail = "fail"` and exit 1.

## Send-back

### To Discover, exit 2

| Condition | Detected by | IDs cited |
|---|---|---|
| SB-1 · A requirement has no feasible architecture under the accepted stack | `architecture-reviewer` finding, `kind: infeasible` | the `REQ-nnn` in that finding's `requirements` |
| SB-2 · Two requirements force opposed constraints | `architecture-reviewer` finding, `kind: contradiction` | both `REQ-nnn` in that finding's `requirements`, plus the `CON-nnn` in `constraints` |

The six context files and the ADRs written before the finding stay on disk with `status: draft` and `status: proposed`. `requirements.yaml` is left untouched, per §1.6. `Next` on the handoff is `/discover <IDEA-nnn> --remedy <comma-separated REQ ids>`, the §4c upstream form. `<IDEA-nnn>` is the top-level `id` of the `requirements.yaml` loaded at step 2, which `specs/03-discover.md` §Command makes `$1` at entry point C. The remedy list carries every cited REQ; `specs/03-discover.md` records them in `revision_log[].reopened` and scopes nothing by epic.

### To Plan, exit 2

| Condition | Detected by | IDs cited |
|---|---|---|
| SB-3 · A constraint sent back by Plan stands | `architecture-reviewer` returning an empty finding list at R3 | the `CON-nnn` from `--remedy`, and the `STORY-nnn` from the Plan report |

No file changes on this path. `Next` on the handoff is `/plan <EPIC-nnn> --resume`.

### Received

| From | Command | Effect |
|---|---|---|
| Plan | `/constitute <IDEA-nnn> --remedy CON-nnn` | The Remedy workflow re-opens exactly that constraint. Replacement proceeds by supersession: a new ADR, the introducing ADR set to `superseded`, the CON row set to `retired`, a replacement CON allocated. No ADR body is edited in place |

## Integration

| Skill | Consumes from it | Produces for it | Sends back to it | Receives send-back from it | Shared subagents | `state.toml` read / written |
|---|---|---|---|---|---|---|
| Explore | `explore/brief.md` sections `## Non-goals`, `## Competitor scan`, `## Technology scan`; `explore/decision.yaml` fields `decision`, `decided_on`, `reason`, `carry_forward`. `IDEA-nnn` is read from `decision.yaml` into ADR-000's `consumes`; brief non-goals are unnumbered lines and carry the path and heading in the CON `source` field | none — Explore is upstream and terminal in that direction | none — §5 gives Constitute one send-back target, Discover | none — Explore sends back nowhere | none | reads `[current].phase` |
| Discover | `requirements.yaml`: `IDEA-nnn` (`id`), `EPIC-nnn` (`epics[].id`), `REQ-nnn` (`requirements[].id`), `PERSONA-nnn` (`personas[].id`); fields `status`, `revision`, `epics[].out_of_scope`, `requirements[].{statement, acceptance_signal, priority, status}` | none — Discover is upstream | SB-1, SB-2, citing `REQ-nnn`, scoped by the `EPIC-nnn` holding the first cited REQ | none — Discover's only send-back target is Explore | none | reads `[current].id` and `[active].constitute` |
| Constitute | self | self | self | self | self | writes `[current].phase = "constitute"`, `[current].id`, `[active].constitute = IDEA-nnn`, and on the remedy path `[constitute].remedy_con`, all through `phase set` |
| Plan | `reports/<STORY-nnn>-plan.yaml` on the remedy path | `context/*.md` (all six), `adr/ADR-nnn.md`, IDs `CON-nnn`, `AP-nnn`, `ADR-nnn` | SB-3, citing `CON-nnn` and `STORY-nnn` | `/constitute <IDEA-nnn> --remedy CON-nnn` when a story cannot satisfy a constraint | none — Plan owns `api-designer` | reads `[current].id` and `[active].plan`; the CLI writes `[constitute].remedy_con` from `phase set constitute --id <IDEA-nnn> --remedy CON-nnn` |
| Build | none — Build emits CLI-written reports this phase does not read | `context/*.md` read by Build's hooks per the map below; `adr/ADR-nnn.md` read by the commit-msg hook for the id | none — Build defects route through Plan, per §5 | none — Build's send-back target is Plan | none | reads `[current].id` and `[active].constitute` |
| Verify | none — Verify's `FIND-nnn` reach Constitute through Plan | `anti-patterns.md` `## Anti-pattern index`, read by `anti-pattern-scanner` by `AP-nnn`; `architecture-constraints.md` `## Constraint index`, read for the source CON of each finding | none — §5 gives Constitute one send-back target | none — Verify sends back to Build and Plan | none | none — Verify runs on story ids |
| Release | none | none — release notes cite `STORY-nnn`, not `ADR-nnn` | none | none — Release sends back to Verify | none | none |
| Design | `brand/tokens.json` path only; the token values stay in Design's file | `coding-standards.md` `## Design tokens`, holding `tokens.path`; `devforgeai design lint` reads `tokens.path` from that section and the frontend globs from `config.toml` | none | none — Design sends back to Discover | none | none |
| Reflect | none — Reflect runs after Release | `adr/ADR-nnn.md` and `reports/<IDEA-nnn>-constitute.yaml`, read as `OBS-nnn` evidence | none | none — Reflect returns `REC-nnn` recommendations, not gates | none | none |
| CLI | `config.toml` from `stack detect`; `gates.toml`; `state.toml` | `context/*.md` parsed by `context audit` and `doc validate`; the two subagent JSON objects parsed by `report ingest` | exit 2 from `gate check --phase constitute` | none — the CLI issues send-backs, it does not receive them | none | reads `[current].phase`, `[current].id`, `[active].constitute`; writes all three plus `[constitute].remedy_con` through `phase set` |

### Build hook read map

| Hook | Call | File and H2 section read |
|---|---|---|
| pre-commit (git) | `context audit` | all six files: frontmatter, the H2 list, `## Constraint index`, `## Anti-pattern index` |
| pre-commit (git) | `doc validate` on staged `.devforgeai/` files | frontmatter of the staged file |
| PreToolUse Write/Edit | `doc validate --producer-check <path>` | frontmatter key `produced_by` of the target file |
| PreToolUse Write/Edit | `design lint <path>` | `coding-standards.md` `## Design tokens`, key `tokens.path` |
| PostToolUse Bash | `gate check --phase build --partial` | `tech-stack.md` `## Tooling`, key `framework.test` |
| commit-msg (git) | id presence | `adr/` directory listing, for `ADR-nnn` id resolution |
| Build's library-substitution check | `context audit` key set plus the scanner | `tech-stack.md` `## Languages`, `## Frameworks`, `## Data stores`; `dependencies.md` `## Approved dependencies`, `## Forbidden dependencies` |
| Build's placement check | `context audit` CA-3 plus the scanner | `source-tree.md` `## Roots`, `## Layers`, `## File placement rules`, `## Naming conventions` |
| Build's layering check | the scanner | `architecture-constraints.md` `## Layer dependency rules`, `## Constraint index` |
| Build's style check | the scanner | `coding-standards.md` `## Formatting`, `## Naming` |

## Handoff

PASS:

```
Phase     2 · Constitute      IDEA-004 · payment-reconciliation
Done      6 context files · 9 ADRs · 14 CON · 11 AP
Gate      PASS  context audit 0 · review 0 · alignment 0
Verified  alignment-auditor · 31/31 cross-file pairs

Next      /plan EPIC-001
Then      /design EPIC-001
Blocked   none

Full report: .devforgeai/reports/IDEA-004-constitute.yaml
```

`Next` and `Then` carry an `EPIC-nnn`, not the `IDEA-nnn` this phase ran on: Plan works one epic at a time while Constitute writes one context set for the whole requirements document. `devforgeai handoff` picks the first entry of `requirements.epics[]` in file order whose `requirements` list holds at least one `requirements[]` record with `status: accepted`.

SEND BACK, with the §6 maximum of three `Found` lines and the `Then` line omitted:

```
Phase     2 · Constitute      IDEA-004 · payment-reconciliation
Done      6 context files · 4 ADRs · 7 CON · 0 AP
Gate      SEND BACK to Discover  3 requirements unrealizable
Verified  architecture-reviewer · 23/23 requirements
Found     REQ-014 no architecture meets 50ms p99 under CON-002
Found     REQ-019 single-region storage contradicts REQ-014
Found     REQ-031 offline write contradicts REQ-019

Next      /discover IDEA-004 --remedy REQ-014,REQ-019,REQ-031
Blocked   none

Full report: .devforgeai/reports/IDEA-004-constitute.yaml
```

Twelve lines. `devforgeai handoff` prints `alignment-auditor` on the `Verified` line for a PASS and `architecture-reviewer` for a SEND BACK, because §6 allows one such line and the gate result names which verifier produced it.

## Templates

Files under `skills/establishing-context/templates/`. Each body below is indented four spaces so that this spec document carries only the fourteen §11 H2 headings; the shipped file holds the same bytes with the four-space indent removed from every line.

### templates/tech-stack.md


    ---
    schema: devforgeai/context-tech-stack/1
    id: tech-stack
    phase: constitute
    status: draft
    produced_by: establishing-context
    consumes: []
    open_questions: []
    ---

    # Tech stack

    ## Languages

    | Key | Value | Source |
    |---|---|---|
    | language.primary | <identifier> | config.toml |
    | language.primary.version | <version> | config.toml |

    ## Runtimes

    | Key | Value | Source |
    |---|---|---|
    | runtime.name | <identifier> | config.toml |
    | runtime.version | <version> | config.toml |

    ## Frameworks

    | Key | Value | Source |
    |---|---|---|
    | framework.<role> | <identifier> | ADR-nnn |
    | framework.<role>.version | <version> | ADR-nnn |

    ## Data stores

    | Key | Value | Source |
    |---|---|---|
    | datastore.<role> | <identifier> | ADR-nnn |
    | datastore.<role>.version | <version> | ADR-nnn |

    ## Tooling

    | Key | Value | Source |
    |---|---|---|
    | framework.test | <identifier> | config.toml |
    | framework.build | <identifier> | config.toml |
    | framework.lint | <identifier> | config.toml |
    | framework.format | <identifier> | config.toml |
    | framework.package | <identifier> | config.toml |

    ## Excluded technologies

    | Technology | Reason | Recorded in |
    |---|---|---|
    | <identifier> | <one sentence> | ADR-nnn |


### templates/source-tree.md


    ---
    schema: devforgeai/context-source-tree/1
    id: source-tree
    phase: constitute
    status: draft
    produced_by: establishing-context
    consumes: []
    open_questions: []
    ---

    # Source tree

    ## Roots

    | Key | Value | Source |
    |---|---|---|
    | source.root | <path> | config.toml |
    | test.root | <path> | config.toml |
    | build.output.root | <path> | config.toml |

    ## Layers

    | Key | Value | Source |
    |---|---|---|
    | layer.<name>.path | <glob> | ADR-nnn |

    | Layer | Purpose |
    |---|---|
    | <name> | <one sentence> |

    ## Directory map

    ```
    <path>/            <one phrase>
    <path>/<sub>/      <one phrase>
    ```

    ## File placement rules

    | Artifact kind | Path pattern | Example |
    |---|---|---|
    | <kind> | <glob> | <path> |

    ## Naming conventions

    | Key | Value | Source |
    |---|---|---|
    | naming.file | <camel \| pascal \| snake \| kebab \| upper-snake> | ADR-nnn |
    | naming.directory | <camel \| pascal \| snake \| kebab \| upper-snake> | ADR-nnn |
    | naming.test-file | <camel \| pascal \| snake \| kebab \| upper-snake> | ADR-nnn |

    ## Generated and excluded paths

    | Glob | Origin |
    |---|---|
    | <glob> | <generator name or `vendored`> |


### templates/dependencies.md


    ---
    schema: devforgeai/context-dependencies/1
    id: dependencies
    phase: constitute
    status: draft
    produced_by: establishing-context
    consumes: []
    open_questions: []
    ---

    # Dependencies

    ## Approved dependencies

    | Name | Scope | Purpose | Layer | Admitted by |
    |---|---|---|---|---|
    | <identifier> | <runtime \| dev \| test \| build> | <one sentence> | <layer name or `all`> | ADR-nnn |

    ## Forbidden dependencies

    | Name | Reason | Replacement | Recorded in |
    |---|---|---|---|
    | <identifier> | <one sentence> | <identifier or `none`> | <AP-nnn or CON-nnn> |

    ## Version policy

    | Key | Value | Source |
    |---|---|---|
    | dep.<name>.version | <version constraint> | ADR-nnn |
    | dep.<name>.scope | <runtime \| dev \| test \| build> | ADR-nnn |

    ## License policy

    | License identifier | Standing |
    |---|---|
    | <SPDX identifier> | <allowed \| forbidden> |

    ## Addition procedure

    A dependency enters `## Approved dependencies` through an ADR whose `## Constraints introduced` names the CON that admits it. A dependency present in neither table is unapproved.


### templates/coding-standards.md


    ---
    schema: devforgeai/context-coding-standards/1
    id: coding-standards
    phase: constitute
    status: draft
    produced_by: establishing-context
    consumes: []
    open_questions: []
    ---

    # Coding standards

    ## Formatting

    | Key | Value | Source |
    |---|---|---|
    | style.indent | <integer> | config.toml |
    | style.line.max | <integer> | config.toml |
    | style.quote | <single \| double> | config.toml |

    ## Naming

    | Key | Value | Source |
    |---|---|---|
    | naming.type | <camel \| pascal \| snake \| kebab \| upper-snake> | ADR-nnn |
    | naming.function | <camel \| pascal \| snake \| kebab \| upper-snake> | ADR-nnn |
    | naming.variable | <camel \| pascal \| snake \| kebab \| upper-snake> | ADR-nnn |
    | naming.constant | <camel \| pascal \| snake \| kebab \| upper-snake> | ADR-nnn |

    ## Error handling

    | Rule | Scope | Source |
    |---|---|---|
    | <declarative sentence> | <glob> | <CON-nnn or ADR-nnn> |

    ## Logging

    | Rule | Scope | Source |
    |---|---|---|
    | <declarative sentence> | <glob> | <CON-nnn or ADR-nnn> |

    ## Testing standards

    | Rule | Scope | Source |
    |---|---|---|
    | <declarative sentence> | <glob> | <CON-nnn or ADR-nnn> |

    ## Documentation

    | Rule | Scope | Source |
    |---|---|---|
    | <declarative sentence> | <glob> | <CON-nnn or ADR-nnn> |

    ## Design tokens

    | Key | Value | Source |
    |---|---|---|
    | tokens.path | .devforgeai/brand/tokens.json | Design |

    A color or type literal in a frontend file resolves to a token name declared in the file at `tokens.path`. `devforgeai design lint` reads `tokens.path` from this section and the frontend globs from `.devforgeai/config.toml`.


### templates/architecture-constraints.md


    ---
    schema: devforgeai/context-architecture-constraints/1
    id: architecture-constraints
    phase: constitute
    status: draft
    produced_by: establishing-context
    consumes: []
    open_questions: []
    ---

    # Architecture constraints

    ## Constraints

    ### CON-nnn <title>

    | Field | Value |
    |---|---|
    | kind | <boundary \| dependency \| layering \| non-goal \| performance \| security \| data \| process> |
    | status | <active \| retired> |
    | statement | <one declarative present-tense sentence> |
    | source | <REQ-nnn \| explore/brief.md ## Non-goals \| init --analyze> |
    | introduced_by | <ADR-nnn or `none`> |
    | enforced_by | <AP-nnn \| devforgeai context audit \| devforgeai design lint \| devforgeai doc validate \| none> |

    ## Layer dependency rules

    | Layer | Depends on | Does not depend on | Constraint |
    |---|---|---|---|
    | <name> | <comma-separated layer names or `nothing`> | <comma-separated layer names> | CON-nnn |

    ## Constraint index

    | CON | Kind | Status | Title | Source | Introduced by | Enforced by |
    |---|---|---|---|---|---|---|
    | CON-nnn | <kind> | <status> | <title> | <source> | <ADR-nnn or `none`> | <enforced_by> |


### templates/anti-patterns.md


    ---
    schema: devforgeai/context-anti-patterns/1
    id: anti-patterns
    phase: constitute
    status: draft
    produced_by: establishing-context
    consumes: []
    open_questions: []
    ---

    # Anti-patterns

    ## Anti-patterns

    ### AP-nnn <title>

    | Field | Value |
    |---|---|
    | category | <library \| structure \| layer \| smell \| security \| style> |
    | severity | <blocker \| high \| medium \| low> |
    | scope | <glob> |
    | detector_kind | <literal \| regex \| glob> |
    | detector | <literal string, regular expression, or glob> |
    | remediation | <one declarative present-tense sentence> |
    | source | CON-nnn |

    ## Anti-pattern index

    | AP | Category | Severity | Scope | Detector kind | Detector | Source |
    |---|---|---|---|---|---|---|
    | AP-nnn | <category> | <severity> | <glob> | <detector_kind> | <detector> | CON-nnn |


### templates/ADR.md


    ---
    schema: devforgeai/adr/1
    id: ADR-nnn
    phase: constitute
    status: proposed
    produced_by: establishing-context
    consumes: []
    open_questions: []
    ---

    # ADR-nnn: <title>

    ## Context

    <the forces, in present tense; for a brownfield decision, the evidence path and line>

    ## Decision

    <one declarative present-tense sentence naming the chosen option, then the options rejected>

    ## Consequences

    | Consequence | Direction | Affected |
    |---|---|---|
    | <one sentence> | <gain \| cost> | <file path, layer name, or REQ-nnn> |

    ## Constraints introduced

    | CON | Kind | Statement |
    |---|---|---|
    | CON-nnn | <kind> | <one declarative present-tense sentence> |

    ## Supersedes

    | Supersedes ADR | Retires CON |
    |---|---|
    | <ADR-nnn or `none`> | <CON-nnn or `none`> |


## Evals

Files under `skills/establishing-context/evals/`.

### evals/evals.json

```json
{
  "skill": "establishing-context",
  "evals": [
    {
      "prompt": "/constitute IDEA-004 in a greenfield project whose requirements.yaml carries id IDEA-004, one epic, and requirements REQ-001 through REQ-006, and whose config.toml names a primary language and a test tool.",
      "expected_output": "Six context files at .devforgeai/context/ and at least one ADR, followed by the PASS handoff block.",
      "expectations": [
        "Each of the six files exists at .devforgeai/context/<stem>.md",
        "Each file's H2 lines equal the fixed list for its stem, in order",
        "tech-stack.md holds a language.primary row whose Source column is config.toml",
        "Every CON-nnn in the constraint index appears in an ADR's Constraints introduced table or carries a REQ-nnn source",
        "The final block begins with 'Phase     2 · Constitute'"
      ]
    },
    {
      "prompt": "/constitute IDEA-012 in a brownfield project where init --analyze left six drafts with status: draft and three open_questions entries.",
      "expected_output": "The six files completed and carrying status: accepted after the user answers, plus ADRs recording the as-is decisions.",
      "expectations": [
        "No file retains status: draft",
        "open_questions is [] in every one of the six files",
        "At least one ADR carries consumes: [] and names an evidence path in its Context section",
        "source-tree.md Layers rows carry Source values of init --analyze or source-tree-mapper",
        "The final block begins with 'Phase     2 · Constitute'"
      ]
    },
    {
      "prompt": "/constitute IDEA-003 where the explore brief lists two unnumbered lines under ## Non-goals and the single epic lists one out_of_scope entry.",
      "expected_output": "Three CON blocks with kind: non-goal, two sourced to the brief heading and one to the epic id.",
      "expectations": [
        "architecture-constraints.md holds three CON blocks with kind: non-goal",
        "Two non-goal CON source fields read 'explore/brief.md ## Non-goals'",
        "One non-goal CON source field is the EPIC-nnn id that declared out_of_scope",
        "No non-goal CON cites an IDEA-nnn or FLOW-nnn id",
        "All three appear as rows in the Constraint index"
      ]
    },
    {
      "prompt": "/constitute IDEA-021 where REQ-014 asks for a 50ms p99 that no architecture over the accepted data store reaches.",
      "expected_output": "A SEND BACK to Discover citing REQ-014.",
      "expectations": [
        "The gate line reads 'SEND BACK to Discover'",
        "A Found line names REQ-014",
        "requirements.yaml is byte-identical to its input",
        "The Next line reads '/discover IDEA-021 --remedy REQ-014'",
        "The handoff block is at most twelve lines"
      ]
    },
    {
      "prompt": "/constitute IDEA-022 where REQ-019 pins single-region storage and REQ-031 requires offline multi-region writes.",
      "expected_output": "A SEND BACK to Discover citing both REQ ids.",
      "expectations": [
        "The gate line reads 'SEND BACK to Discover'",
        "Found lines name REQ-019 and REQ-031",
        "The Next line names IDEA-022 and lists both REQ ids comma-separated",
        "No ADR carries status: accepted",
        "The handoff block is at most twelve lines"
      ]
    },
    {
      "prompt": "/constitute IDEA-004 --remedy CON-003 after Plan reported that STORY-041 cannot satisfy CON-003.",
      "expected_output": "A new ADR superseding the introducing ADR, CON-003 retired, and a replacement CON allocated.",
      "expectations": [
        "The ADR that introduced CON-003 carries status: superseded",
        "A new ADR's Supersedes table names that ADR and retires CON-003",
        "The CON-003 row in the Constraint index reads status retired",
        "A replacement CON row exists with status active",
        "Every CON row other than CON-003 is unchanged"
      ]
    },
    {
      "prompt": "/constitute IDEA-009 where the brownfield draft tech-stack.md and draft dependencies.md name two different values for language.primary.version.",
      "expected_output": "One value across both files, and the discarded value recorded in an ADR.",
      "expectations": [
        "language.primary.version holds one value across the six files",
        "An ADR records the choice between the two values",
        "context audit check CA-5 is satisfiable against the written files"
      ]
    }
  ]
}
```

### evals/cases.jsonl

```jsonl
{"id": "CON-01-greenfield-headings", "prompt": "/constitute IDEA-004", "answers": {"*": "@first"}, "setup": {"files": {".devforgeai/requirements.yaml": "schema: devforgeai/requirements/1\nid: IDEA-004\nphase: discover\nstatus: accepted\nproduced_by: discovering-requirements\nconsumes: [IDEA-004]\nopen_questions: []\nrevision: 1\nrevision_log: []\naccepted_by: user\naccepted_at: 2026-09-10T14:22:05Z\npersonas:\n  - id: PERSONA-001\n    name: Shopper\n    description: A signed-in customer buying from the storefront.\n    goal: Complete a purchase without re-entering data.\nepics:\n  - id: EPIC-001\n    title: Checkout\n    scope: A signed-in shopper completes a purchase in one page.\n    out_of_scope:\n      - The system does not hold card numbers.\n    success_metric: Checkout completion reaches 80 percent.\n    requirements: [REQ-001, REQ-002]\nrequirements:\n  - id: REQ-001\n    actor: PERSONA-001\n    statement: A signed-in shopper completes checkout on one page.\n    rationale: A multi-page checkout loses shoppers at each step.\n    acceptance_signal: The shopper reaches the confirmation state without a page change.\n    priority: should\n    source: user\n    status: accepted\n    open_questions: []\n    traces_to: []\n  - id: REQ-002\n    actor: PERSONA-001\n    statement: Checkout state survives a browser refresh.\n    rationale: Shoppers refresh when a payment step stalls.\n    acceptance_signal: The cart and the entered address are present after a refresh.\n    priority: should\n    source: user\n    status: accepted\n    open_questions: []\n    traces_to: []\n", ".devforgeai/config.toml": "schema = \"devforgeai/config/1\"\ngenerated_at = \"2026-09-10T09:00:00Z\"\ncli_version = \"1.0.0\"\ndegraded = false\n[stack]\nprimary_language = \"lang-a\"\nprimary_language_version = \"9.9\"\ntest_tool = \"tool-t\"\n[paths]\nsource_root = \"src\"\ntest_root = \"tests\"\nbuild_output_root = \"out\"\n", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml", ".devforgeai/reports/IDEA-004-discover.yaml": "FIXTURE:report-IDEA-004-discover.yaml"}}, "expect": {"grader": "context_headings", "args": {"files": "all"}}}
{"id": "CON-02-con-adr-link", "prompt": "/constitute IDEA-005", "answers": {"*": "@first"}, "setup": {"files": {".devforgeai/requirements.yaml": "schema: devforgeai/requirements/1\nid: IDEA-005\nphase: discover\nstatus: accepted\nproduced_by: discovering-requirements\nconsumes: [IDEA-005]\nopen_questions: []\nrevision: 1\nrevision_log: []\naccepted_by: user\naccepted_at: 2026-09-10T14:22:05Z\npersonas:\n  - id: PERSONA-001\n    name: Shopper\n    description: A signed-in customer buying from the storefront.\n    goal: Complete a purchase without re-entering data.\nepics:\n  - id: EPIC-001\n    title: Checkout\n    scope: A signed-in shopper completes a purchase in one page.\n    out_of_scope:\n      - The system does not hold card numbers.\n    success_metric: Checkout completion reaches 80 percent.\n    requirements: [REQ-001, REQ-003]\nrequirements:\n  - id: REQ-001\n    actor: PERSONA-001\n    statement: A signed-in shopper completes checkout on one page.\n    rationale: A multi-page checkout loses shoppers at each step.\n    acceptance_signal: The shopper reaches the confirmation state without a page change.\n    priority: should\n    source: user\n    status: accepted\n    open_questions: []\n    traces_to: []\n  - id: REQ-003\n    actor: PERSONA-001\n    statement: Payment records stay readable for seven years.\n    rationale: An auditor asks for any prior year on request.\n    acceptance_signal: A record written seven years ago opens without a restore step.\n    priority: should\n    source: user\n    status: accepted\n    open_questions: []\n    traces_to: []\n", ".devforgeai/config.toml": "schema = \"devforgeai/config/1\"\ngenerated_at = \"2026-09-10T09:00:00Z\"\ncli_version = \"1.0.0\"\ndegraded = false\n[stack]\nprimary_language = \"lang-a\"\nprimary_language_version = \"9.9\"\ntest_tool = \"tool-t\"\n[paths]\nsource_root = \"src\"\ntest_root = \"tests\"\nbuild_output_root = \"out\"\n", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml", ".devforgeai/reports/IDEA-005-discover.yaml": "FIXTURE:report-IDEA-005-discover.yaml"}}, "expect": {"grader": "con_referenced", "args": {}}}
{"id": "CON-03-non-goal-to-con", "prompt": "/constitute IDEA-003", "answers": {"*": "@first"}, "setup": {"files": {".devforgeai/requirements.yaml": "schema: devforgeai/requirements/1\nid: IDEA-003\nphase: discover\nstatus: accepted\nproduced_by: discovering-requirements\nconsumes: [IDEA-003]\nopen_questions: []\nrevision: 1\nrevision_log: []\naccepted_by: user\naccepted_at: 2026-09-10T14:22:05Z\npersonas:\n  - id: PERSONA-001\n    name: Guest\n    description: A visitor with no stored account.\n    goal: Buy once without signing up.\nepics:\n  - id: EPIC-001\n    title: Guest checkout\n    scope: A guest completes a purchase without an account.\n    out_of_scope:\n      - The system does not import orders from the legacy store.\n    success_metric: Guest conversion reaches 40 percent.\n    requirements: [REQ-010]\nrequirements:\n  - id: REQ-010\n    actor: PERSONA-001\n    statement: A guest checks out without an account.\n    rationale: Account creation loses first-time buyers.\n    acceptance_signal: The guest reaches the confirmation state with no account row created.\n    priority: should\n    source: user\n    status: accepted\n    open_questions: []\n    traces_to: []\n", ".devforgeai/explore/brief.md": "---\nschema: devforgeai/explore-brief/1\nid: IDEA-003\nphase: explore\nstatus: decided\nproduced_by: exploring-ideas\nconsumes: []\nopen_questions: []\n---\n\n# Brief\n\n## Non-goals\n\nThe system does not hold card numbers.\nThe system does not ship a native mobile client.\n", ".devforgeai/config.toml": "schema = \"devforgeai/config/1\"\ngenerated_at = \"2026-09-10T09:00:00Z\"\ncli_version = \"1.0.0\"\ndegraded = false\n[stack]\nprimary_language = \"lang-a\"\nprimary_language_version = \"9.9\"\ntest_tool = \"tool-t\"\n[paths]\nsource_root = \"src\"\ntest_root = \"tests\"\nbuild_output_root = \"out\"\n", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml", ".devforgeai/reports/IDEA-003-discover.yaml": "FIXTURE:report-IDEA-003-discover.yaml"}}, "expect": {"grader": "non_goal_constraints", "args": {"brief": 2, "epic": 1, "brief_source": "explore/brief.md ## Non-goals", "epic_source": "EPIC-001"}}}
{"id": "CON-04-brownfield-completion", "prompt": "/constitute IDEA-012", "answers": {"*": "@first"}, "setup": {"files": {".devforgeai/requirements.yaml": "schema: devforgeai/requirements/1\nid: IDEA-012\nphase: discover\nstatus: accepted\nproduced_by: discovering-requirements\nconsumes: [IDEA-012]\nopen_questions: []\nrevision: 1\nrevision_log: []\naccepted_by: user\naccepted_at: 2026-09-10T14:22:05Z\npersonas:\n  - id: PERSONA-001\n    name: Operator\n    description: A support agent reading past orders.\n    goal: Open any order on request.\nepics:\n  - id: EPIC-001\n    title: Rewrite\n    scope: The order service moves to the new stack.\n    out_of_scope:\n      - The system does not change the public API shape.\n    success_metric: Unreadable orders after cutover stay at 0.\n    requirements: [REQ-020]\nrequirements:\n  - id: REQ-020\n    actor: PERSONA-001\n    statement: Existing orders stay readable after the rewrite.\n    rationale: Support reads old orders daily.\n    acceptance_signal: An order written before the rewrite opens unchanged.\n    priority: should\n    source: user\n    status: accepted\n    open_questions: []\n    traces_to: []\n", ".devforgeai/config.toml": "schema = \"devforgeai/config/1\"\ngenerated_at = \"2026-09-10T09:00:00Z\"\ncli_version = \"1.0.0\"\ndegraded = false\n[stack]\nprimary_language = \"lang-a\"\nprimary_language_version = \"9.9\"\ntest_tool = \"tool-t\"\n[paths]\nsource_root = \"src\"\ntest_root = \"tests\"\nbuild_output_root = \"out\"\n", ".devforgeai/context/tech-stack.md": "---\nschema: devforgeai/context-tech-stack/1\nid: tech-stack\nphase: constitute\nstatus: draft\nproduced_by: establishing-context\nconsumes: []\nopen_questions: [\"framework.orm unresolved\"]\n---\n\n# Tech stack\n\n## Languages\n\n| Key | Value | Source |\n|---|---|---|\n| language.primary | lang-a | init --analyze |\n\n## Runtimes\n\n## Frameworks\n\n## Data stores\n\n## Tooling\n\n## Excluded technologies\n", "src/domain/order.txt": "order aggregate\n", "src/infra/store.txt": "store adapter\n", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml", ".devforgeai/reports/IDEA-012-discover.yaml": "FIXTURE:report-IDEA-012-discover.yaml"}}, "expect": {"grader": "drafts_accepted", "args": {}}}
{"id": "CON-05-sendback-infeasible", "prompt": "/constitute IDEA-021", "answers": {"*": "@first"}, "setup": {"files": {".devforgeai/requirements.yaml": "schema: devforgeai/requirements/1\nid: IDEA-021\nphase: discover\nstatus: accepted\nproduced_by: discovering-requirements\nconsumes: [IDEA-021]\nopen_questions: []\nrevision: 1\nrevision_log: []\naccepted_by: user\naccepted_at: 2026-09-10T14:22:05Z\npersonas:\n  - id: PERSONA-001\n    name: Shopper\n    description: A signed-in customer buying from the storefront.\n    goal: Complete a purchase without re-entering data.\nepics:\n  - id: EPIC-001\n    title: Catalog\n    scope: A shopper browses the catalog.\n    out_of_scope:\n      - The system does not offer faceted search.\n    success_metric: Catalog p99 latency reaches 50 milliseconds.\n    requirements: [REQ-014]\nrequirements:\n  - id: REQ-014\n    actor: PERSONA-001\n    statement: Every catalog query returns within 50 milliseconds at p99.\n    rationale: Shoppers abandon a slow catalog.\n    acceptance_signal: The p99 query latency measured over one hour is at or below 50 milliseconds.\n    priority: should\n    source: user\n    status: accepted\n    open_questions: []\n    traces_to: []\n", ".devforgeai/config.toml": "schema = \"devforgeai/config/1\"\ngenerated_at = \"2026-09-10T09:00:00Z\"\ncli_version = \"1.0.0\"\ndegraded = false\n[stack]\nprimary_language = \"lang-a\"\nprimary_language_version = \"9.9\"\ntest_tool = \"tool-t\"\n[paths]\nsource_root = \"src\"\ntest_root = \"tests\"\nbuild_output_root = \"out\"\n[stack.datastore]\nprimary = \"remote-object-store\"\nprimary_latency_floor_ms = 400\n", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml", ".devforgeai/reports/IDEA-021-discover.yaml": "FIXTURE:report-IDEA-021-discover.yaml"}}, "expect": {"grader": "send_back", "args": {"to": "Discover", "ids": ["REQ-014"], "idea": "IDEA-021", "upstream_sha256": {".devforgeai/requirements.yaml": "b6decd07df6e189e6df7818f3fa296e038d1aec7da0e2e09b46601214516af58"}}}}
{"id": "CON-06-sendback-contradiction", "prompt": "/constitute IDEA-022", "answers": {"*": "@first"}, "setup": {"files": {".devforgeai/requirements.yaml": "schema: devforgeai/requirements/1\nid: IDEA-022\nphase: discover\nstatus: accepted\nproduced_by: discovering-requirements\nconsumes: [IDEA-022]\nopen_questions: []\nrevision: 1\nrevision_log: []\naccepted_by: user\naccepted_at: 2026-09-10T14:22:05Z\npersonas:\n  - id: PERSONA-001\n    name: Shopper\n    description: A signed-in customer buying from the storefront.\n    goal: Complete a purchase without re-entering data.\nepics:\n  - id: EPIC-001\n    title: Residency\n    scope: Order data stays in one region.\n    out_of_scope:\n      - The system does not support per-tenant regions.\n    success_metric: Rows outside the named region stay at 0.\n    requirements: [REQ-019]\n  - id: EPIC-002\n    title: Offline\n    scope: A shopper writes orders offline.\n    out_of_scope:\n      - The system does not sync media offline.\n    success_metric: Offline write durability reaches 100 percent.\n    requirements: [REQ-031]\nrequirements:\n  - id: REQ-019\n    actor: PERSONA-001\n    statement: All order data is stored in exactly one region and is not replicated out of it.\n    rationale: A data residency rule binds the operator.\n    acceptance_signal: An audit of the store finds order rows in one region only.\n    priority: should\n    source: user\n    status: accepted\n    open_questions: []\n    traces_to: []\n  - id: REQ-031\n    actor: PERSONA-001\n    statement: A shopper writes an order offline and the write is durable before any network round trip.\n    rationale: Field staff work without connectivity.\n    acceptance_signal: An order written with the network disabled survives a device restart.\n    priority: should\n    source: user\n    status: accepted\n    open_questions: []\n    traces_to: []\n", ".devforgeai/config.toml": "schema = \"devforgeai/config/1\"\ngenerated_at = \"2026-09-10T09:00:00Z\"\ncli_version = \"1.0.0\"\ndegraded = false\n[stack]\nprimary_language = \"lang-a\"\nprimary_language_version = \"9.9\"\ntest_tool = \"tool-t\"\n[paths]\nsource_root = \"src\"\ntest_root = \"tests\"\nbuild_output_root = \"out\"\n", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml", ".devforgeai/reports/IDEA-022-discover.yaml": "FIXTURE:report-IDEA-022-discover.yaml"}}, "expect": {"grader": "send_back", "args": {"to": "Discover", "ids": ["REQ-019", "REQ-031"], "idea": "IDEA-022", "upstream_sha256": {".devforgeai/requirements.yaml": "6c3c5df9211eb6943987974bb4785e2ad56a60815d1ac8bef7e7abf519605e7d"}}}}
{"id": "CON-07-remedy-supersession", "prompt": "/constitute IDEA-004 --remedy CON-003", "answers": {"*": "@first"}, "setup": {"files": {".devforgeai/requirements.yaml": "schema: devforgeai/requirements/1\nid: IDEA-004\nphase: discover\nstatus: accepted\nproduced_by: discovering-requirements\nconsumes: [IDEA-004]\nopen_questions: []\nrevision: 1\nrevision_log: []\naccepted_by: user\naccepted_at: 2026-09-10T14:22:05Z\npersonas:\n  - id: PERSONA-001\n    name: Shopper\n    description: A signed-in customer buying from the storefront.\n    goal: Complete a purchase without re-entering data.\nepics:\n  - id: EPIC-001\n    title: Checkout\n    scope: A signed-in shopper completes a purchase in one page.\n    out_of_scope:\n      - The system does not hold card numbers.\n    success_metric: Checkout completion reaches 80 percent.\n    requirements: [REQ-001]\nrequirements:\n  - id: REQ-001\n    actor: PERSONA-001\n    statement: A signed-in shopper completes checkout on one page.\n    rationale: A multi-page checkout loses shoppers at each step.\n    acceptance_signal: The shopper reaches the confirmation state without a page change.\n    priority: should\n    source: user\n    status: accepted\n    open_questions: []\n    traces_to: []\n", ".devforgeai/context/architecture-constraints.md": "---\nschema: devforgeai/context-architecture-constraints/1\nid: architecture-constraints\nphase: constitute\nstatus: accepted\nproduced_by: establishing-context\nconsumes: [IDEA-004]\nopen_questions: []\n---\n\n# Architecture constraints\n\n## Constraints\n\n### CON-003 Single write path\n\n| Field | Value |\n|---|---|\n| kind | layering |\n| status | active |\n| statement | The application layer holds the only write path to the order store. |\n| source | REQ-001 |\n| introduced_by | ADR-002 |\n| enforced_by | none |\n\n### CON-004 Page budget\n\n| Field | Value |\n|---|---|\n| kind | performance |\n| status | active |\n| statement | The checkout page loads within two seconds. |\n| source | REQ-001 |\n| introduced_by | ADR-002 |\n| enforced_by | none |\n\n## Layer dependency rules\n\n| Layer | Depends on | Does not depend on | Constraint |\n|---|---|---|---|\n| domain | nothing | application, infrastructure | CON-003 |\n\n## Constraint index\n\n| CON | Kind | Status | Title | Source | Introduced by | Enforced by |\n|---|---|---|---|---|---|---|\n| CON-003 | layering | active | Single write path | REQ-001 | ADR-002 | none |\n| CON-004 | performance | active | Page budget | REQ-001 | ADR-002 | none |\n", ".devforgeai/adr/ADR-002.md": "---\nschema: devforgeai/adr/1\nid: ADR-002\nphase: constitute\nstatus: accepted\nproduced_by: establishing-context\nconsumes: [REQ-001]\nopen_questions: []\n---\n\n# ADR-002: One write path through the application layer\n\n## Context\n\nCheckout writes arrive from two entry points.\n\n## Decision\n\nThe application layer owns the only write path to the order store.\n\n## Consequences\n\n| Consequence | Direction | Affected |\n|---|---|---|\n| Batch import routes through the application layer | cost | REQ-001 |\n\n## Constraints introduced\n\n| CON | Kind | Statement |\n|---|---|---|\n| CON-003 | layering | The application layer holds the only write path to the order store. |\n| CON-004 | performance | The checkout page loads within two seconds. |\n\n## Supersedes\n\n| Supersedes ADR | Retires CON |\n|---|---|\n| none | none |\n", ".devforgeai/reports/STORY-041-plan.yaml": "schema: devforgeai/report/1\nid: STORY-041\nphase: plan\nstatus: send_back\nproduced_by: planning-work\nconsumes: [CON-003]\nopen_questions: []\nsend_back:\n  to: constitute\n  constraint: CON-003\n  reason: The nightly import writes 400k rows and cannot route through the application layer inside the batch window.\n", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml", ".devforgeai/reports/IDEA-004-discover.yaml": "FIXTURE:report-IDEA-004-discover.yaml"}}, "expect": {"grader": "remedy_supersession", "args": {"con": "CON-003", "adr": "ADR-002", "untouched_con": ["CON-004"], "baseline_rows": {"CON-003": "| CON-003 | layering | active | Single write path | REQ-001 | ADR-002 | none |", "CON-004": "| CON-004 | performance | active | Page budget | REQ-001 | ADR-002 | none |"}}}}
{"id": "CON-08-key-contradiction", "prompt": "/constitute IDEA-009", "answers": {"*": "@first"}, "setup": {"files": {".devforgeai/requirements.yaml": "schema: devforgeai/requirements/1\nid: IDEA-009\nphase: discover\nstatus: accepted\nproduced_by: discovering-requirements\nconsumes: [IDEA-009]\nopen_questions: []\nrevision: 1\nrevision_log: []\naccepted_by: user\naccepted_at: 2026-09-10T14:22:05Z\npersonas:\n  - id: PERSONA-001\n    name: Shopper\n    description: A signed-in customer buying from the storefront.\n    goal: Complete a purchase without re-entering data.\nepics:\n  - id: EPIC-001\n    title: Reproducible build\n    scope: The service builds the same way on every machine.\n    out_of_scope:\n      - The system does not vendor the toolchain.\n    success_metric: Byte-identical rebuilds reach 100 percent.\n    requirements: [REQ-001]\nrequirements:\n  - id: REQ-001\n    actor: PERSONA-001\n    statement: The service builds reproducibly on one toolchain version.\n    rationale: Two toolchain versions produced different binaries last quarter.\n    acceptance_signal: Two builds of the same commit produce identical output.\n    priority: should\n    source: user\n    status: accepted\n    open_questions: []\n    traces_to: []\n", ".devforgeai/config.toml": "schema = \"devforgeai/config/1\"\ngenerated_at = \"2026-09-10T09:00:00Z\"\ncli_version = \"1.0.0\"\ndegraded = false\n[stack]\nprimary_language = \"lang-a\"\nprimary_language_version = \"9.9\"\n[paths]\nsource_root = \"src\"\ntest_root = \"tests\"\nbuild_output_root = \"out\"\n", ".devforgeai/context/tech-stack.md": "---\nschema: devforgeai/context-tech-stack/1\nid: tech-stack\nphase: constitute\nstatus: draft\nproduced_by: establishing-context\nconsumes: []\nopen_questions: []\n---\n\n# Tech stack\n\n## Languages\n\n| Key | Value | Source |\n|---|---|---|\n| language.primary | lang-a | init --analyze |\n| language.primary.version | 9.9 | init --analyze |\n\n## Runtimes\n\n## Frameworks\n\n## Data stores\n\n## Tooling\n\n## Excluded technologies\n", ".devforgeai/context/dependencies.md": "---\nschema: devforgeai/context-dependencies/1\nid: dependencies\nphase: constitute\nstatus: draft\nproduced_by: establishing-context\nconsumes: []\nopen_questions: []\n---\n\n# Dependencies\n\n## Approved dependencies\n\n## Forbidden dependencies\n\n## Version policy\n\n| Key | Value | Source |\n|---|---|---|\n| language.primary.version | 8.1 | init --analyze |\n\n## License policy\n\n## Addition procedure\n\nA dependency enters the approved table through an ADR.\n", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml", ".devforgeai/reports/IDEA-009-discover.yaml": "FIXTURE:report-IDEA-009-discover.yaml"}}, "expect": {"grader": "key_consistency", "args": {"key": "language.primary.version"}}}
```

Eight cases. `CON-05` and `CON-06` exercise the SEND BACK path.

Prior state travels in the case line rather than coming from the runner. `upstream_sha256` in `CON-05` and `CON-06` is the SHA-256 of the `setup.files` entry at that path, computed over the exact bytes written there, so the `send_back` grader detects an edit to `requirements.yaml` without reading a baseline copy. `baseline_rows` in `CON-07` holds the two `## Constraint index` rows as they stand in `setup.files` before the run. Both follow the §9 contract, which gives a grader the workspace, the transcript, and `args`.

### evals/graders.py

Pure functions over the workspace filesystem and the transcript string. No network, no subprocess, no randomness, no call to the `devforgeai` binary, which the temp workspace does not hold.

```python
CONTEXT_HEADINGS: dict[str, list[str]]
# stem -> the fixed H2 list from ## Outputs, for all six stems.

def _h2(path: str) -> list[str]: ...
# Return every line beginning "## " in file order, stripped.

def _frontmatter(path: str) -> dict: ...
# Return the key/value pairs between the first two "---" lines, values unparsed.

def _kv_rows(path: str) -> list[tuple[str, str, int]]:
    ...
# Return (key, value, line_no) for every row of every table whose header row is
# exactly "| Key | Value | Source |".

def _handoff_block(transcript: str) -> list[str]: ...
# Return the lines from the last line starting "Phase     " through the line
# starting "Full report:", inclusive.

def context_headings(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    ...
# For each stem in args["files"] ("all" expands to the six), compare _h2 against
# CONTEXT_HEADINGS[stem] for equality including order. Evidence names the first
# stem whose list differs and the differing index.

def con_referenced(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    ...
# Parse the "## Constraint index" table for CON ids with status "active".
# Parse every ADR's "## Constraints introduced" table for CON ids, and
# requirements.yaml for REQ ids. Pass when every active CON appears in some ADR
# table, or its index Source cell holds a REQ id present in requirements.yaml.
# Evidence lists unreferenced CON ids.

def non_goal_constraints(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    ...
# Count "### CON-" blocks in architecture-constraints.md whose kind field is
# "non-goal". Pass when args["brief"] of them carry source == args["brief_source"]
# and args["epic"] of them carry source == args["epic_source"], with no other
# non-goal block present. Evidence names offending CON ids and their source cells.

def drafts_accepted(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    ...
# Pass when all six files exist, each _frontmatter status is "accepted", each
# open_questions is "[]", and every "## " section in each file is non-empty
# (holds at least one non-heading, non-blank line). Evidence names the first
# file and field that fails.

def send_back(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    ...
# Read _handoff_block. Pass when: the Gate line contains
# "SEND BACK to " + args["to"]; the Next line contains args["idea"] followed by
# " --remedy " and every id in args["ids"]; one "Found " line names each id;
# the block is at most 12 lines; and for every path in args["upstream_sha256"],
# the SHA-256 hex digest of that file in the workspace equals the recorded
# digest, which is the prior state carried in the case line rather than read
# back from the runner. Evidence names the first condition that failed and the
# offending line.

def remedy_supersession(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    ...
# Pass when: _frontmatter of adr/<args["adr"]>.md has status "superseded";
# exactly one other ADR's "## Supersedes" row names args["adr"] and retires
# args["con"]; the args["con"] row in the Constraint index reads status
# "retired"; at least one CON row whose id is absent from args["baseline_rows"]
# exists with status "active"; the args["con"] row differs from its
# args["baseline_rows"] entry; and every id in args["untouched_con"] equals its
# args["baseline_rows"] entry after collapsing runs of spaces. The prior rows
# arrive in args rather than from the workspace. Evidence names the first
# failing condition.

def key_consistency(workspace: str, transcript: str, args: dict) -> tuple[bool, str]:
    ...
# Collect _kv_rows across the six files. Group by key. Pass when no key maps to
# more than one distinct value, and when args holds "key", that key is present.
# Evidence names the key and every (file, line, value) triple that disagrees.
```

## Decisions

1. Context file `id` is the filename stem, not an allocated three-digit id, because the six files are project singletons.
2. Context file `schema` is `devforgeai/context-<stem>/1`, distinct per file, so `doc validate` selects the heading list from the schema alone.
3. Accepted by `specs/01-cli.md`. Context file `status` enum is `draft | accepted`; §5 leaves the per-doc-type enum to this spec.
4. ADR `status` enum is `proposed | accepted | rejected | superseded`.
5. Accepted by `specs/01-cli.md`. `AP-nnn` is an ID prefix for this phase, allocated by `doc validate --allocate AP`; §5 lists only `ADR-nnn` and `CON-nnn`, and the amendment stands.
6. Motivating REQ IDs live in ADR frontmatter `consumes` alone, with no duplicate body section; `consumes: []` is valid for a brownfield ADR that records a decision the code already embodies.
7. The cross-file contradiction check runs over tables whose header row is exactly `| Key | Value | Source |`, against the closed key namespace in `## Outputs`; a key outside the namespace fails CA-5.
8. A non-goal CON drawn from the brief cites `explore/brief.md ## Non-goals` in its `source` field rather than an Explore ID, confirmed against `specs/02-explore.md` §Outputs section 8, which defines that section as "Unnumbered lines, one per line, 1 to 10 lines" carrying no ID. A non-goal CON drawn from `requirements.yaml` cites the `EPIC-nnn` whose `out_of_scope` list held it. Explore's `carry_forward` entry for `sections: [Non-goals]` names `establishing-context` as consumer and `architecture-constraints.md and anti-patterns.md entries` as the product, so a brief non-goal a pattern can observe also yields an AP at step 9.
9. The Constitute instance `<ID>` is the top-level `id` of `requirements.yaml`, an `IDEA-nnn` (`specs/03-discover.md` §Outputs, `^IDEA-\d{3}$`); reports land at `.devforgeai/reports/<IDEA-nnn>-constitute.yaml`. Constitute runs once per requirements document, which carries every epic, because the six context files are project singletons. `/constitute` takes no epic argument.
10. Accepted by `specs/01-cli.md`. The six context files carry their filename stem as `id` and are writable by `establishing-context` alone, enforced by the PreToolUse `doc validate --producer-check` hook, which the CLI applies to these singletons. This replaces the "context files are IMMUTABLE" rule in the existing `alignment-auditor`, which had no enforcement behind it.
11. `devforgeai init --analyze` stamps its drafts `produced_by: establishing-context` and `status: draft`, so the producer-check hook admits this skill's completing edits. Proposed behavior for `specs/01-cli.md`.
12. Accepted by `specs/01-cli.md`. `devforgeai context audit` exits 1 while any context file carries `status: draft`, which places user acceptance before the first commit that touches `.devforgeai/context/`.
13. `--remedy` resolves by supersession: a new ADR, the introducing ADR moved to `status: superseded`, the CON row moved to `status: retired`, and a replacement CON allocated. No ADR body is edited in place.
14. `--remedy` has an upheld outcome: when `architecture-reviewer` returns no finding against the constraint, no file changes and the gate emits SEND BACK to Plan citing the CON and the STORY.
15. `alignment-auditor` is adapted with its method inverted: the exact-text matching the existing agent performs moves into `context audit` checks CA-1 through CA-8, and the subagent takes the semantic comparison the existing agent declines.
16. `tech-stack-detector` is retired. Detection is `devforgeai stack detect`; validation of detected values against `tech-stack.md` is `context audit` CA-5. Recorded for `specs/11-subagent-catalog.md`.
17. `api-designer`, `context-validator`, and `diagnostic-analyst` are not invoked by this phase; their owners are Plan, Build, and Verify respectively. Recorded for `specs/11-subagent-catalog.md`.
18. Two severity vocabularies, each in its own place. A document row — an `AP-nnn` entry, a `CON-nnn` row — carries `blocker | high | medium | low`, because that is a property of the rule. A verifier finding carries the envelope's `block | warn | info`, because that is what `report ingest` parses and what decides whether `passed` moves. Neither vocabulary collides with the §2 pattern's single-word half, and not because of case — the pattern matches case-insensitively. Both are released by clause 2 and clause 4 of the scoping: each value is a YAML scalar or an enum table cell, which is not instruction prose.
19. The §2 ceremony rule is scoped by six clauses and run by `scripts/ceremony_scan.py`. A fenced block, a YAML scalar, a backticked span, an enum table cell, a path, and a heading are each out of scope, and the six context files this phase writes are out of scope entirely: they are documents about a target project, and an absolute rule is what a project is entitled to write. The `no_ceremony` grader that walked `.devforgeai/` is deleted with its whole subject; the case that called it takes a grader checking what the phase is accountable for — that each of the six files holds its declared H2 set and that each rule line carries a `CON-nnn`.
20. Settled by `specs/01-cli.md`, which specifies `report ingest <subagent> <source>` as an addition to §4 and records it under its own `## Decisions`. §7 names the call in the SubagentStop row without listing it in the §4 table. This phase invokes it for `architecture-reviewer` and `alignment-auditor`.
21. Settled by `specs/01-cli.md`. `## Gate` reproduces the `[[gate]]` entry for `constitute` verbatim from that spec's §Gate, in the `[[gate]]` / `[[gate.check]]` schema with check kinds `context_audit` and `report_metric`. `context_audit` is this phase's compiled-in required kind. The send-back is a check carrying `on_fail = "send_back"` rather than a gate-level table.
22. The `state.toml` fields this phase touches are `[current].phase`, `[current].id`, `[active].constitute` holding an `IDEA-nnn`, and `[constitute].remedy_con` holding one `CON-nnn`. All four are in the final schema in `specs/01-cli.md` §Outputs. `remedy_con` is written by `devforgeai phase set constitute --id <IDEA-nnn> --remedy CON-nnn`; `--remedy` is a general `phase set` flag that each receiving phase interprets, and for `constitute` it accepts exactly one CON id.
23. `devforgeai handoff` prints `alignment-auditor` on the `Verified` line for a PASS and `architecture-reviewer` for a SEND BACK; two verifiers run and §6 allows one such line.
24. The SEND BACK handoff layout omits the `Then` line, which §6 permits, so that the §6 maximum of three `Found` lines fits inside twelve lines.
25. An absent `.devforgeai/explore/brief.md` yields an empty non-goal CON set and the workflow continues; Explore is optional ahead of Discover.
26. A subagent whose JSON does not parse against its schema is re-invoked once with the parse error appended; a second failure leaves the gate metric absent, which `gate check` treats as exit 1.
27. `requirements.yaml` is read against the real shape in `specs/03-discover.md` §Outputs: fourteen top-level keys, `requirements[]` records keyed `id`, `actor`, `statement`, `rationale`, `acceptance_signal`, `priority`, `source`, `status`, `open_questions`, `traces_to`, and `epics[]` records keyed `id`, `title`, `scope`, `out_of_scope`, `success_metric`, `requirements`. The field is `statement`, not `text`. No interface is assumed.
29. The send-back `Next` line is `/discover <IDEA-nnn> --remedy <REQ ids>`, the §4c upstream form. `specs/03-discover.md` §Command states that `--remedy` selects entry point C and "`$1` is the IDEA id", and its §Send-back table prints the same shape for its own send-back to Explore. The `<IDEA-nnn>` is the top-level `id` of the `requirements.yaml` loaded at step 2. Discover re-opens by REQ id alone and its `revision_log[]` carries no epic field, so this phase cites no `EPIC-nnn` on the send-back path.
30. The forward `Next` and `Then` lines print `/plan <EPIC-nnn>` and `/design <EPIC-nnn>`, selected by `devforgeai handoff` as the first `epics[]` entry in file order whose `requirements` list holds a `requirements[]` record with `status: accepted`. `epics[]` carries no `status` field of its own in `specs/03-discover.md` §Outputs, so epic readiness is derived from its requirements. `specs/03-discover.md` §Handoff prints `Then /plan IDEA-004`; the Plan spec settles which of the two Plan accepts, and a change there alters these two lines alone.
31. `ADR-000` is reserved for the Explore decision, written at step 10 from `explore/decision.yaml`, per the `carry_forward` entry in `specs/02-explore.md` §Outputs that names `decision.yaml` becoming "ADR-000, the reason this project exists" with `consumer: establishing-context`. It is the one ADR whose `consumes` holds an `IDEA-nnn` rather than `REQ-nnn` ids, and the one allocated by reservation rather than by `doc validate --allocate ADR`. Absent `decision.yaml`, ADR-000 is not written.
32. `decision.yaml` carrying `decision: kill` or `decision: park` halts step 3, because Explore promoted nothing and there is nothing to constitute. The value is printed and the command stops.
33. The brief's `## Competitor scan` and `## Technology scan` are read at step 3, per Explore's `carry_forward` entry naming them "tech stack candidates and ADR context" for `establishing-context`. A `## Technology scan` row with `Maturity: established` supplies a candidate for an unset `tech-stack.md` key and a row in `dependencies.md`, with `Source` = `explore/brief.md ## Technology scan`; `Maturity` of `emerging` or `experimental` supplies ADR `## Context` text alone.
34. `epics[].out_of_scope` yields one `non-goal` CON per entry, with `source` = that `EPIC-nnn`. `specs/03-discover.md` §Outputs makes the field required with length `>= 1`, so every epic contributes at least one such CON and a project run with no Explore phase still has a non-goal set.
35. Dependency: eval case `CON-07` fixes the shape of Plan's send-back report at `.devforgeai/reports/<STORY-nnn>-plan.yaml` with a `send_back` mapping carrying `to`, `constraint`, and `reason`. §3 makes that file CLI-written and §5 does not give its schema. The Plan spec and `specs/01-cli.md` settle it; a different shape changes workflow step R2 and that one fixture.

Blockers: none.

29. **The `Skill` grant on `allowed-tools` is sanctioned and unused by the numbered workflow.** It is there for the send-back path: a Plan remedy that turns on a screen's shape reaches `designing-interfaces` through the Skill tool rather than asking the user to type `/design` mid-phase. No numbered step invokes it, and the grant costs nothing until it is used, so it stays rather than being removed and re-added the first time that path is walked.

30. **`AskUserQuestion` belongs to this skill and to no agent of this phase.** Step 12 asks the acceptance question in the main conversation. `architecture-reviewer` and `alignment-auditor` hold `Read`, `Grep`, and `Glob` and nothing else; Claude Code would strip the tool from them in any case.
