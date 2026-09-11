---
schema: devforgeai-spec/1
doc: verify
status: draft
produced_by: verify-spec-author
consumes: [00-conventions]
open_questions: []
---

# Verify · validating-quality

## Scope

`validating-quality` is Phase 5. It reviews what Build produced for one story, from a context that did not write the code, and records what it found. It runs on one slash command, `/verify STORY-nnn`, in light mode by default and in deep mode with `--deep` before a release. Light mode is seven judgements: acceptance-criteria compliance, code review against `context/coding-standards.md`, an anti-pattern scan against the `AP-nnn` rows of `context/anti-patterns.md`, constraint validation against the `CON-nnn` rows of `context/architecture-constraints.md`, a coverage review of the layer figures the CLI already measured during Build, dead-code detection over the story's declared file set, and validation of every Definition-of-Done item the story pushes to a later story. Deep mode adds three: a security audit over the ten OWASP Top 10 categories, complexity and duplication metrics against the thresholds in `config.toml`, and an architecture review against the accepted ADRs. Its one typed document is `reports/STORY-nnn-qa.yaml`, holding `FIND-nnn` findings and a `deferrals[]` sequence, read by Release and by Reflect.

This component writes no production code, no test, no story, no ADR, and no context file. It runs no test command, no coverage command, and no lint command: `devforgeai gate check` ran those during Build and runs the coverage and lint checks again at this gate, and the skill reads the numbers from `reports/STORY-nnn-build.yaml`. It decides no threshold: coverage floors live in `config.toml` `[[layer]].coverage_min`, complexity and duplication ceilings in `config.toml` `[verify]`, and the gate criteria in `gates.toml`. It names no language, package manager, test runner, linter, or scanner; the two optional commands it can call come from `config.toml` `[verify].metrics_command` and `[verify].call_graph_command`, each with a stated fallback that uses Grep alone. A defect it finds in the implementation leaves as a send-back to Build; a defect it finds in an acceptance criterion leaves as a send-back to Plan; it edits neither the code nor the story.

## Inputs

| Document | Path | IDs read | Produced by | Absent behaviour |
|---|---|---|---|---|
| Story | `.devforgeai/stories/STORY-nnn.md` | `STORY-nnn`, `AC-nnn`, and the `CON-nnn`, `AP-nnn`, `UI-nnn`, `REQ-nnn` of `consumes` | `planning-work` | `doc load story` exits 1 on `DFA-E200`; the command body does not run |
| Build report | `.devforgeai/reports/STORY-nnn-build.yaml` | none | `devforgeai-cli` | `report show` exits 1 on `DFA-E400`; the run stops with one `Blocked` line naming the path |
| Sprint | `.devforgeai/stories/sprint.yaml` | `SPRINT-nnn`, `STORY-nnn` | `planning-work` | the handoff `Next` line falls back to `/release vX.Y.Z` by the rule in `## Handoff` |
| Coding standards | `.devforgeai/context/coding-standards.md` | none | `establishing-context` | `doc load context all` exits 1 on `DFA-E200`; the command body does not run |
| Anti-patterns | `.devforgeai/context/anti-patterns.md` | `AP-nnn` | `establishing-context` | same |
| Architecture constraints | `.devforgeai/context/architecture-constraints.md` | `CON-nnn` | `establishing-context` | same |
| Source tree | `.devforgeai/context/source-tree.md` | none | `establishing-context` | same |
| Tech stack | `.devforgeai/context/tech-stack.md` | none | `establishing-context` | same |
| Dependencies | `.devforgeai/context/dependencies.md` | none | `establishing-context` | same |
| ADRs | `.devforgeai/adr/ADR-nnn.md` | `ADR-nnn` | `establishing-context` | deep mode only; an empty `adr/` directory makes `adr-conformance-reviewer` report `total: 0`, `passed: 0` |
| UI spec | `.devforgeai/ui-specs/UI-nnn.md` | `UI-nnn` | `designing-interfaces` | read only for a `UI-nnn` in the story's `## Interface`; an absent file is a `spec-gap` finding citing that id |
| Configuration | `.devforgeai/config.toml` | none | `devforgeai stack detect`, human | `DFA-E101`, exit 1 |
| Release gate report | `.devforgeai/reports/vX.Y.Z-release.yaml` | `FIND-nnn`, `STORY-nnn` | `devforgeai-cli` | read on the received-remedy path only; absent means the cited ids come from `$ARGUMENTS` alone |

The story sections this phase locates by heading text are `## Acceptance Criteria` (the `AC-nnn` list), `## Constraints` (the `CON-nnn` table), `## Anti-patterns` (the `AP-nnn` table), `## Interface` (the `UI-nnn` table), `## Layer` (the one layer name), `## Files` (the `Path | Kind | Layer` table that bounds every scan), and `## Out of scope` (the behaviours a finding does not name).

From the build report this phase reads only keys the `specs/01-cli.md` report schema defines: `gate.result`, `gate.checks[].id`, `gate.checks[].status`, `coverage.format`, `coverage.source`, `coverage.overall`, `coverage.layers[]` of `name`, `covered`, `total`, `percent`, `min`, `status`, and `coverage.unassigned` of `files` and `percent`. The Build spec adds no key to that file, because the CLI writes it.

From a UI spec this phase reads `## Accessibility`, whose eight rows are `Landmark`, `Heading order`, `Name`, `Role`, `Keyboard path`, `Focus visible`, `Contrast`, `Motion`, and whose `Evidence` column names a `Region`, a `TOKEN-<name>`, or a contrast ratio. Token conformance in the changed files is `devforgeai design lint`, which the PreToolUse hook already ran during Build, so this phase raises no token finding.

## Outputs

One typed document. `<ID>` for this phase is the `STORY-nnn` the command took as `$1`.

### `.devforgeai/reports/STORY-nnn-qa.yaml`

Doc type `qa-report` in the `specs/01-cli.md` doc-type table: schema `devforgeai/qa-report/1`, `id` grammar `^STORY-[0-9]{3}$`, `status` enum `pass`, `fail`, `send_back`, ID prefix `FIND`, extra top-level keys permitted. Sixteen top-level keys, in this order:

```yaml
schema: devforgeai/qa-report/1
id: STORY-014
phase: verify
status: pass
produced_by: validating-quality
consumes: [STORY-014, AC-001, AC-002, CON-003, AP-002, ADR-002, UI-004]
open_questions: []
mode: light
verified_on: 2026-09-11
build_report: .devforgeai/reports/STORY-014-build.yaml
coverage:
  source: .devforgeai/reports/STORY-014-build.yaml
  overall: 87.4
  layers:
    - name: domain
      percent: 96.6
      min: 95.0
      status: pass
findings:
  - id: FIND-021
    category: anti-pattern
    severity: high
    file: src/application/checkout.ext
    line: 118
    relates_to: AP-002
    disposition: fix
    summary: The handler formats a store row instead of returning the domain type.
    evidence: src/application/checkout.ext:118 returns a map literal built from the row.
    found_by: anti-pattern-scanner
blockers: []
deferrals:
  - id: FIND-024
    story: STORY-014
    dod_item: CON-005 the retry budget is enforced at the boundary
    target: STORY-031
    reason: dependency_missing
    opened_on: 2026-09-11
    con_or_ap: CON-005
summary:
  checks: 6
  findings: 2
  blockers: 0
  high: 1
  medium: 1
  low: 0
  deferrals: 1
```

| Key | Type | Required | Default | Constraint |
|---|---|---|---|---|
| `schema` | string | yes | none | constant `devforgeai/qa-report/1` |
| `id` | string | yes | none | `^STORY-\d{3}$`, the story reviewed |
| `phase` | string | yes | none | constant `verify` |
| `status` | string | yes | none | enum `pass`, `fail`, `send_back`; `pass` when `blockers` is empty and every check ran, `send_back` when the run leaves as `## Send-back`, `fail` otherwise |
| `produced_by` | string | yes | none | constant `validating-quality` |
| `consumes` | list of string | yes | none | the `STORY-nnn`, every `AC-nnn` reviewed, and every `CON-nnn`, `AP-nnn`, `ADR-nnn`, `UI-nnn` a finding or deferral cites |
| `open_questions` | list of string | yes | `[]` | each one sentence, 1 to 200 characters |
| `mode` | string | yes | none | enum `light`, `deep` |
| `verified_on` | string | yes | none | `YYYY-MM-DD`, the date the run wrote the file |
| `build_report` | string | yes | none | the path of the build report step 3 read |
| `coverage` | object | yes | none | schema below |
| `checks` | list of object | yes | none | 7 entries in light mode, 10 in deep; entry schema below |
| `findings` | list of object | yes | `[]` | entry schema below |
| `blockers` | list of string | yes | `[]` | every `findings[].id` whose `severity` is `blocker`, in `findings[]` order; the unqualified dotted path `blockers` that the `verify-blockers` check reads resolves to this top-level sequence, and the integer count is `summary.blockers` |
| `deferrals` | list of object | yes | `[]` | entry schema below |
| `summary` | object | yes | none | schema below |

`coverage` is copied from the build report and recomputed by nothing:

| Field | Type | Required | Constraint |
|---|---|---|---|
| `source` | string | yes | the build report path, equal to `build_report` |
| `overall` | float | yes | the build report `coverage.overall` value |
| `layers` | list of object | yes | one entry per build report `coverage.layers[]` entry, with `name`, `percent`, `min`, `status` copied |

`checks[]`:

| Field | Type | Required | Constraint |
|---|---|---|---|
| `name` | string | yes | the check-name enum below |
| `verifier` | string | yes | the subagent name from `## Subagents` |
| `examined` | integer | yes | the unit count the subagent reported as `total` |
| `passed` | integer | yes | the count the subagent reported as `passed` |
| `result` | string | yes | enum `pass` (`passed == examined`), `fail` (`passed < examined`), `skip` (the subagent reported `total: 0`) |

Check-name enum, a closed list of ten. The first seven run in both modes; the last three run in deep mode alone.

| `name` | Verifier | Mode |
|---|---|---|
| `ac-compliance` | `ac-compliance-verifier` | light, deep |
| `code-review` | `standards-reviewer` | light, deep |
| `anti-pattern-scan` | `anti-pattern-scanner` | light, deep |
| `constraint-validation` | `constraint-auditor` | light, deep |
| `coverage-review` | `coverage-gap-auditor` | light, deep |
| `dead-code` | `dead-code-detector` | light, deep |
| `deferral-validation` | `deferral-validator` | light, deep |
| `security-audit` | `security-auditor` | deep |
| `quality-metrics` | `code-quality-auditor` | deep |
| `architecture-review` | `adr-conformance-reviewer` | deep |

Light mode writes seven `checks[]` entries and deep mode ten. The `summary.checks` count is the length of `checks[]`.

`findings[]`:

| Field | Type | Required | Constraint |
|---|---|---|---|
| `id` | string | yes | `^FIND-\d{3}$`, from the band rule in workflow step 5 |
| `category` | string | yes | the category enum below |
| `severity` | string | yes | enum `blocker`, `high`, `medium`, `low`, the `## Anti-pattern index` `Severity` enum of `specs/04-constitute.md` |
| `file` | string | yes | a `Path` value of the story's `## Files` table, repo-relative with forward slashes, or `""` when the finding names no file |
| `line` | integer | yes | 1-based; `0` when `file` is `""` or the finding names no line |
| `relates_to` | string | yes | one id matching `^(AC\|CON\|AP\|ADR)-\d{3}$` |
| `disposition` | string | yes | enum `fix`, `defer`, `accept`; a finding of `severity: blocker` takes `fix` and no other value |
| `summary` | string | yes | one line, 1 to 120 characters |
| `evidence` | string | yes | one line, 1 to 200 characters, naming the path, the line, and what was read there |
| `found_by` | string | yes | the subagent name that emitted it |

Category enum, a closed list of eleven, one per source of judgement:

| `category` | Raised by | Cites |
|---|---|---|
| `standards` | `standards-reviewer` | a `## Formatting`, `## Naming`, `## Error handling`, `## Logging`, or `## Documentation` rule of `coding-standards.md`, through the `CON-nnn` that governs it |
| `anti-pattern` | `anti-pattern-scanner` | an `AP-nnn` |
| `constraint` | `constraint-auditor` | a `CON-nnn` |
| `coverage` | `coverage-gap-auditor` | an `AC-nnn` |
| `dead-code` | `dead-code-detector` | a `CON-nnn` or `AP-nnn` |
| `deferral` | `deferral-validator` | a `CON-nnn` or `AP-nnn` |
| `ac-compliance` | `ac-compliance-verifier` | an `AC-nnn` |
| `spec-gap` | `ac-compliance-verifier`, `constraint-auditor` | an `AC-nnn`; the code meets the criterion and the criterion misses the requirement |
| `security` | `security-auditor` | a `CON-nnn` of `kind: security`, or the `AP-nnn` of `category: security` |
| `complexity` | `code-quality-auditor` | a `CON-nnn` or `AP-nnn` |
| `duplication` | `code-quality-auditor` | a `CON-nnn` or `AP-nnn` |

`deferrals[]`. A deferral is one Definition-of-Done item pushed to a later story. A Definition-of-Done item is one of four things, and the `dod_item` string opens with its id: an `AC-nnn` of the story's `## Acceptance Criteria`, the layer coverage floor of the story's `## Layer`, a `CON-nnn` of its `## Constraints`, or an `AP-nnn` of its `## Anti-patterns`.

| Field | Type | Required | Constraint |
|---|---|---|---|
| `id` | string | yes | `^FIND-\d{3}$`, equal to the `findings[]` entry whose `disposition` is `defer` |
| `story` | string | yes | `^STORY-\d{3}$`, equal to the document `id` |
| `dod_item` | string | yes | one line, 1 to 120 characters, opening with the `AC-nnn`, `CON-nnn`, or `AP-nnn` it names, or with the layer name for a coverage floor |
| `target` | string | yes | `^(STORY\|ADR)-\d{3}$`, resolving in the ID index |
| `reason` | string | yes | the reason enum below |
| `opened_on` | string | yes | `YYYY-MM-DD`, equal to `verified_on` |
| `con_or_ap` | string | yes | `^(CON\|AP)-\d{3}$`, or `""` when the item is an `AC-nnn` or a coverage floor |

Deferral reason enum, a closed list of five:

| `reason` | Means |
|---|---|
| `dependency_missing` | the item needs behaviour a `STORY-nnn` that is not at `status: released` carries |
| `scope_boundary` | the item names a behaviour this story's `## Out of scope` excludes and another story carries |
| `decision_pending` | an `ADR-nnn` that would decide the item is at `status: proposed` |
| `external_blocker` | a system or party outside the project controls the item |
| `tooling_absent` | `config.toml` names no command that measures the item, and `[verify].metrics_command` or `[verify].call_graph_command` is `""` |

`summary`, seven integer fields: `checks` (the length of `checks[]`), `findings`, `blockers`, `high`, `medium`, `low`, `deferrals`. `blockers` is the length of the top-level `blockers` list. The `blockers` list and the `summary` counts are derived from `findings[]` by the model; no mechanical check compares the derivation against `findings[]`, and the gate reads the derived list. That limit is recorded in `## Decisions`.

### Severity mapping to the verifier contract

The `devforgeai/verifier/1` stdout contract of `specs/01-cli.md` `## Subagents` fixes a three-value severity enum, `block | warn | info`, which this spec does not change. Every subagent maps its finding severity onto it by one rule, stated again in each output schema: `blocker` emits `block`, and `high`, `medium`, and `low` emit `warn`. `info` is emitted by no subagent of this phase. A `block` finding lowers the subagent's `passed` below its `total`, which fails the `verifier_pass` check at the compiled floor `min_ratio = 1.0`, which is how a blocker-severity match fails the gate. The `findings[].severity` value in `reports/STORY-nnn-qa.yaml` is the four-value form; the value in `reports/STORY-nnn-verify.yaml`, which the CLI writes from the ingested JSON, is the three-value form.

### What this phase does not write

`.devforgeai/reports/STORY-nnn-verify.yaml` is the CLI's gate report, written by `gate check --phase verify` and by `report ingest`, per `specs/01-cli.md` Decision 46. `.devforgeai/stories/STORY-nnn.md` is Plan's document; `devforgeai phase set verify` writes its `status` to `built` and the PreToolUse producer check blocks a write to it from this phase. No file under the project's source roots is opened for writing by this skill or by any of its subagents.

## Workflow

Steps 1 to 11 are the full run. Deep mode adds step 7. The remedy and resume workflows follow.

**1. Establish the run — model.** Input: `$ARGUMENTS`, the stdout of the four preamble lines. The run kind is `remedy` when `$ARGUMENTS` holds `--remedy`, `resume` when it holds `--resume`, and `full` otherwise. The mode is `deep` when `$ARGUMENTS` holds `--deep`, and `config.toml` `[verify].mode` otherwise. Output: a run kind, a mode, and the `STORY-nnn` of `$1`. Failure path: `$1` carries a prefix other than `STORY`; the run stops with one `Blocked` line naming the accepted prefix.

**2. Read the story — model.** Input: the `doc load story` stdout. Output: the `AC-nnn` list from `## Acceptance Criteria`, the `Path | Kind | Layer` rows from `## Files`, the layer name from `## Layer`, the `CON-nnn` rows from `## Constraints`, the `AP-nnn` rows from `## Anti-patterns`, the `UI-nnn` rows from `## Interface`, and the lines of `## Out of scope`. Failure path: `## Files` holds no data row; the run stops with one `Blocked` line naming the story and that heading, because the file set bounds every scan.

**3. Read the build report — model, CLI.** Actor: the model runs `devforgeai report show $1 build`. Input: `.devforgeai/reports/STORY-nnn-build.yaml`. Output: `gate.result`, the `gate.checks[]` list, and the `coverage` block. Failure path: exit 1 on `DFA-E400` or `DFA-E401`; the run stops with one `Blocked` line naming the path, because the phase measures no coverage of its own.

**4. Open the phase — model, CLI.** Actor: the model runs `devforgeai phase set verify --id <STORY-nnn>`. Output: `state.toml` `[current]` of `verify` and that id, `[active].verify` of that id, and the story's frontmatter `status` set to `built`. Failure path: exit 1 on `DFA-E320` means the build gate is not `PASS`; the run stops and the Stop hook prints the gate result.

**5. Allocate the finding band — model, CLI.** Actor: the model runs `devforgeai doc validate --allocate FIND` once. Input: the ID index. Output: a `FIND-nnn` taken as `base`. The subagent at position `k` of the `## Subagents` order, counting from zero, receives the id band `FIND-(base + 100k)` through `FIND-(base + 100k + 99)` and numbers its findings from the low end. The band is 100 wide and no subagent truncates: a review agent reports every finding it has, including the uncertain and the low-severity, and the gate and this phase's own dispositions are what filter. An agent told to report only what matters under-reports, and a gap that reaches no report reaches no reader. Failure path: exit 1 on `DFA-E215` means the prefix is exhausted; the run stops with one `Blocked` line naming it.

**6. Review — seven subagents, one parallel batch.** Actor: `ac-compliance-verifier`, `standards-reviewer`, `anti-pattern-scanner`, `constraint-auditor`, `coverage-gap-auditor`, `dead-code-detector`, and `deferral-validator` are invoked in one message. Input: per `## Subagents`. Output: one `devforgeai/verifier/1` JSON object per subagent on its stdout; the `SubagentStop` hook runs `devforgeai report ingest <name> -`, which writes `verifiers.<report_field>` into `.devforgeai/reports/STORY-nnn-verify.yaml` and appends the findings. Failure path: the JSON does not parse; `report ingest` writes the block with `status: unparsed`, and the model invokes that one subagent once more with the parse error appended. A second failure leaves the block unparsed, and the `verifier_pass` check fails with `DFA-E316`.

`deferral-validator` runs in this batch on the deferrals already on disk: the entries of every `reports/STORY-*-qa.yaml` whose `target` is this story, plus, on a resume or remedy run, the `deferrals[]` of this story's own prior report. The deferrals this run opens are validated at step 9.

**7. Deep review — three subagents, one parallel batch, deep mode only.** Actor: `security-auditor`, `code-quality-auditor`, and `adr-conformance-reviewer` are invoked in one message. Input: per `## Subagents`. Output and failure path: as step 6. In light mode this step does not run, and the `verify-deep` gate check records `skip` with `reason: condition`.

**8. Disposition — model.** Input: every ingested finding. Output: one `disposition` per finding from the enum `fix`, `defer`, `accept`. A finding of `severity: blocker` takes `fix`. A finding of `category: spec-gap` takes `fix` and routes to Plan by `## Send-back`. A finding whose remedy is one Definition-of-Done item carried by another story takes `defer`. A finding the story's `## Out of scope` already excludes takes `accept`. Failure path: a finding cites a `file` value absent from `## Files`; the model drops the finding and records its `evidence` line under `open_questions`.

**9. Draft and validate the deferrals — model, `deferral-validator`.** Input: every finding of `disposition: defer`, the `sprint.yaml` `stories[]` list, the `adr/` directory listing. Output: one `deferrals[]` entry per such finding, then one further invocation of `deferral-validator` over the drafted entries. A drafted entry whose `target` resolves to no document, whose `reason` the evidence does not support, or whose `target` story already defers back to this story comes back as a `block` finding in that subagent's second block, which fails `verifier_pass`. Failure path: the second invocation returns unparsed JSON; the block carries `status: unparsed` and the gate fails with `DFA-E316`.

**10. Write the report — model.** Actor: the model writes `.devforgeai/reports/STORY-nnn-qa.yaml` from `templates/qa-report.yaml`. Input: the ingested findings, the dispositions, the deferral entries, the `coverage` block of step 3, the `checks[]` rows built from each subagent's `passed` and `total`. Output: the file, with `blockers` holding every `findings[].id` of `severity: blocker` and `summary` holding the seven counts. Failure path: the `PostToolUse` hook returns the `doc validate` diagnostic in `hookSpecificOutput.additionalContext`; the model rewrites the key it names.

**11. Close — CLI, Stop hook.** Actor: the Stop hook runs `devforgeai gate check --phase verify`, then `devforgeai handoff`. Output: `.devforgeai/reports/STORY-nnn-verify.yaml` and the §6 block. Failure path: a FAIL exits 2 with `decision: "block"` and the failing checks in `reason`, under a budget of three blocks per `session_id`; at the last block of that budget the hook exits 0 and the FAIL block renders in `systemMessage`, per §7. A SEND BACK exits 0 with the SEND BACK block in `systemMessage` and blocks nothing.

### Remedy workflow, `/verify STORY-nnn --remedy FIND-nnn,...`

This form arrives from Release, when a deferral blocks deployment on the target platform (`specs/09-release.md` `## Send-back`, check `release-deferrals`), or when a story has no PASS verify report (check `release-stories`).

**R1. Locate the cited findings — model, CLI.** Input: the id list after `--remedy`, `.devforgeai/reports/STORY-nnn-qa.yaml` read from disk, and `devforgeai report show <vX.Y.Z> release` for the release gate report's `findings[]`. Output: one `(FIND-nnn, deferrals[] entry, release finding summary)` triple per cited id. A cited id that is a `STORY-nnn` rather than a `FIND-nnn` opens the full run from step 2 for that story. Failure path: a cited `FIND-nnn` appears in no `findings[]` entry of this story's report; the run stops with one `Blocked` line naming the id.

**R2. Re-dispose — model.** Input: the triples from R1. Output: the cited finding's `disposition` changes from `defer` to `fix`, its `deferrals[]` entry is removed, and `blockers` and `summary` are recomputed. A cited finding whose evidence shows the deferral stands takes `disposition: defer` with a new `target` and a `reason` of `external_blocker`, which Release re-reads on its `--resume` run. Failure path: none; both outcomes are recorded in the same file.

**R3. Re-review the cited paths — subagents.** Input: the `file` values of the cited findings, restricted to rows of the story's `## Files`. Actor: the subagents whose `found_by` value appears among the cited findings, invoked in one batch. Output: fresh verifier blocks replacing the prior ones for those subagents. Failure path: as step 6.

**R4. Close — model, CLI.** Actor: the model rewrites `.devforgeai/reports/STORY-nnn-qa.yaml` with the re-disposed findings and runs `devforgeai phase set verify --id <STORY-nnn>`. Output: the Stop hook block, whose `Next` is `/build STORY-nnn --remedy FIND-nnn,...` when a re-disposed finding is now `fix`, and `/release vX.Y.Z --resume` when every cited deferral was closed without code work.

### Resume workflow, `/verify STORY-nnn --resume`

Re-enters at step 3 and re-reads `reports/STORY-nnn-build.yaml`, so a rebuilt story's fresh coverage and gate results are the numbers the run works from. Step 5 allocates a new `base`, because the prior run's ids are in the index and the allocator returns the next free one. Every `findings[]` entry of the prior report whose `disposition` was `defer` or `accept` is carried into the new report unchanged, including its `deferrals[]` entry and its original `opened_on`; every entry whose `disposition` was `fix` is dropped and re-raised by the step 6 batch when the defect stands. `state.toml` `[active].verify` already holds the story, so step 4 re-runs `phase set` and changes nothing.

## Subagents

Ten subagents, each a registered verifier, each read-only, each emitting exactly one `devforgeai/verifier/1` object and never two. The envelope keys sit at the top — `schema`, `subagent`, `id`, `passed`, `total`, `unit`, `findings` — and every field an agent adds of its own sits under `payload`. The `findings[]` entries carry the contract's four fields, `id`, `severity`, `summary`, `evidence`, plus `confidence`, a float from `0.0` to `1.0` for how far the reading carries, and the four this phase adds, `category`, `file`, `line`, `relates_to`, which `report ingest` copies through unread.

Three rules hold across all ten. A finding at `severity: warn` never lowers `passed`: the counts measure the units the agent was asked to judge, and a warning is a remark about one of them. Every finding is reported, uncertain and low-severity included, with no cap and no self-filtering; a low-confidence finding comes back at a low `confidence` rather than withheld. And no agent applies a threshold: an agent that runs a measuring command copies the number into its finding and its `payload` verbatim, and the comparison against a ceiling or a floor is a `gate check` check kind reading `config.toml` and `gates.toml`.

### ac-compliance-verifier

- **name**: `ac-compliance-verifier`
- **derives_from**: `C:\Users\bryan\.claude\agents\ac-compliance-verifier.md`
- **purpose**: Decide, without having written the code, whether each `AC-nnn` of the story is met by a test that reads the outcome the criterion names.
- **tools**: Read, Grep, Glob
- **model**: `opus` — the judgement is whether a test asserts the named outcome, which is a reading of two texts against each other.
- **input**: the story path, the `AC-nnn` list with full text, the `## Files` rows of `Kind` `source` and `test`, the `## Out of scope` lines, its id band.
- **output**:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "ac-compliance-verifier",
  "id": "STORY-014",
  "passed": 7,
  "total": 7,
  "unit": "ACs",
  "findings": [
    { "id": "FIND-021", "severity": "block", "category": "ac-compliance", "file": "tests/checkout.ext",
      "line": 0, "relates_to": "AC-007", "summary": "no test reads the rejected-payment outcome",
      "evidence": "tests/checkout.ext holds no case asserting the rejected state named by AC-007" }
  ]
}
```

`total` is the number of `AC-nnn` in the story; `passed` is the number with no `block` finding. `category` is `ac-compliance`, or `spec-gap` when the code meets the criterion and the criterion names less than its `REQ-nnn` states.
- **invoked_at**: workflow step 6, in parallel with six others.
- **registered_verifier**: yes

### standards-reviewer

- **name**: `standards-reviewer`
- **derives_from**: `C:\Users\bryan\.claude\agents\code-reviewer.md`
- **purpose**: Review the story's changed source files against the rules of `context/coding-standards.md`.
- **tools**: Read, Grep, Glob
- **model**: `opus` — the judgement is whether a rule written in prose is met by code, which no pattern decides.
- **input**: the `## Files` rows of `Kind` `source`, the seven `coding-standards.md` sections, the `CON-nnn` rows that `enforced_by` names for each, its id band.
- **output**:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "standards-reviewer",
  "id": "STORY-014",
  "passed": 5,
  "total": 6,
  "unit": "files",
  "findings": [
    { "id": "FIND-041", "severity": "warn", "category": "standards", "file": "src/application/checkout.ext",
      "line": 44, "relates_to": "CON-009", "summary": "the error path returns a bare value",
      "evidence": "src/application/checkout.ext:44 returns on failure without the error type coding-standards.md names" }
  ]
}
```

`total` is the number of `Kind` `source` rows; `passed` is the number with no `block` finding.
- **invoked_at**: workflow step 6, in parallel.
- **registered_verifier**: yes

### anti-pattern-scanner

- **name**: `anti-pattern-scanner`
- **derives_from**: `C:\Users\bryan\.claude\agents\anti-pattern-scanner.md`
- **purpose**: Match each `AP-nnn` detector against the story's file set and report every hit with its severity.
- **tools**: Read, Grep, Glob
- **model**: `sonnet` — the detector and its kind come from the index, so the work is matching and citing rather than deciding.
- **input**: the `## Anti-pattern index` rows of `AP`, `Category`, `Severity`, `Scope`, `Detector kind`, `Detector`, `Source`; the `## Files` rows whose `Path` the `Scope` glob matches; its id band.
- **output**:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "anti-pattern-scanner",
  "id": "STORY-014",
  "passed": 4,
  "total": 6,
  "unit": "anti-patterns",
  "findings": [
    { "id": "FIND-061", "severity": "block", "category": "anti-pattern", "file": "src/application/checkout.ext",
      "line": 118, "relates_to": "AP-002", "summary": "the application layer opens the order store directly",
      "evidence": "src/application/checkout.ext:118 matches the AP-002 regex detector" }
  ]
}
```

`severity` is `block` when the `## Anti-pattern index` `Severity` cell reads `blocker`, and `warn` for `high`, `medium`, and `low`. `total` is the number of `AP-nnn` rows in scope; `passed` is the number with no hit of `blocker` severity.
- **invoked_at**: workflow step 6, in parallel.
- **registered_verifier**: yes

### constraint-auditor

- **name**: `constraint-auditor`
- **derives_from**: `C:\Users\bryan\.claude\agents\context-validator.md`
- **purpose**: Decide whether the story's file set satisfies each `CON-nnn` its `## Constraints` table binds, and whether the constraint set decides the case at all.
- **tools**: Read, Grep, Glob
- **model**: `opus` — a constraint is a sentence, and whether code satisfies it is a reading.
- **input**: the story's `## Constraints` rows, the matching `### CON-nnn` blocks with `kind`, `status`, `statement`, `source`, `introduced_by`, `enforced_by`; the `## Layer dependency rules` row for the story's `## Layer`; the `## Files` rows; its id band.
- **output**:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "constraint-auditor",
  "id": "STORY-014",
  "passed": 3,
  "total": 4,
  "unit": "constraints",
  "findings": [
    { "id": "FIND-081", "severity": "warn", "category": "constraint", "file": "src/application/checkout.ext",
      "line": 91, "relates_to": "CON-003", "summary": "a second write path reaches the order store",
      "evidence": "src/application/checkout.ext:91 writes the store outside the single path CON-003 states" }
  ]
}
```

`category` is `constraint`, or `spec-gap` when the constraint holds and an `AC-nnn` asserts less than the constraint requires. `total` is the number of `CON-nnn` rows; `passed` is the number with no `block` finding.
- **invoked_at**: workflow step 6, in parallel.
- **registered_verifier**: yes

### coverage-gap-auditor

- **name**: `coverage-gap-auditor`
- **derives_from**: `C:\Users\bryan\.claude\agents\coverage-analyzer.md`
- **purpose**: Read the build report's layer figures and decide which uncovered region leaves an `AC-nnn` untested.
- **tools**: Read, Grep, Glob
- **model**: `sonnet` — the numbers arrive measured, and the judgement is which uncovered file relates to which criterion.
- **input**: the build report `coverage` block, the story's `## Layer`, the `## Files` rows, the `AC-nnn` list, its id band.
- **output**:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "coverage-gap-auditor",
  "id": "STORY-014",
  "passed": 3,
  "total": 4,
  "unit": "layers",
  "findings": [
    { "id": "FIND-101", "severity": "warn", "category": "coverage", "file": "src/domain/order.ext",
      "line": 0, "relates_to": "AC-004", "summary": "the rejection branch is uncovered and AC-004 asserts it",
      "evidence": "build report coverage.layers domain 96.6 with src/domain/order.ext uncovered lines 61-74" }
  ]
}
```

This subagent runs no command. `total` is the number of `coverage.layers[]` entries; `passed` is the number whose `status` the build report records as `pass`. A layer below its floor is a `block` finding.
- **invoked_at**: workflow step 6, in parallel.
- **registered_verifier**: yes

### dead-code-detector

- **name**: `dead-code-detector`
- **derives_from**: `C:\Users\bryan\.claude\agents\dead-code-detector.md`
- **purpose**: Report symbols defined in the story's file set that no other file in the project references.
- **tools**: Read, Grep, Glob, Bash — an unscoped `Bash`, admitted by the `PreToolUse` metrics arm alone. That handler reads the payload's `agent_type` and permits exactly the `[verify].call_graph_command` string of `config.toml`, denying every other command with the key to edit. A `Bash(<tool>:*)` scope would name a tool in an agent file, which §1 rule 2 forbids; the permission belongs to the project's configuration.
- **model**: `sonnet` — reference counting with a confidence judgement on dynamic dispatch.
- **input**: the `## Files` rows of `Kind` `source`, the `## Roots` and `## Generated and excluded paths` entries of `source-tree.md`, the value of `config.toml` `[verify].call_graph_command`, its id band.
- **output**:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "dead-code-detector",
  "id": "STORY-014",
  "passed": 11,
  "total": 12,
  "unit": "symbols",
  "findings": [
    { "id": "FIND-121", "severity": "warn", "confidence": 0.6, "category": "dead-code", "file": "src/domain/order.ext",
      "line": 203, "relates_to": "AP-004",
      "summary": "the symbol is defined and referenced by no file outside its own",
      "evidence": "grep over source roots returns one hit, the definition at src/domain/order.ext:203" }
  ],
  "payload": { "method": "grep", "dead": 1 }
}
```

`payload.method` is the closed enum `command` and `grep`. With `[verify].call_graph_command` non-empty the subagent runs it through `Bash` and sets `payload.method: command`; with the key `""` it resolves each symbol with Grep over the `## Roots` paths and sets `payload.method: grep`. `confidence` is a float from `0.0` to `1.0` and is present on every finding: `1.0` when the method is `command`, `0.6` when it is `grep` and the definition is exported — a caller outside the source roots or reached by name at run time leaves no text reference — and `0.9` when it is not exported, where the file set bounds every caller.

Every finding here is `warn`, so `passed` equals `total`: both are the number of symbols examined, and a `warn` never lowers `passed`. The count of dead symbols rides in `payload.dead` instead, where `gates.toml` can read it as `verifiers.dead_code.payload.dead` if the project wants a ceiling. The two rules would otherwise point opposite ways — `warn` sets no gate while a ratio does — and blocking a story on a grep-resolved symbol at `confidence: 0.6` is the worse trade.
- **invoked_at**: workflow step 6, in parallel.
- **registered_verifier**: yes

### deferral-validator

- **name**: `deferral-validator`
- **derives_from**: `C:\Users\bryan\.claude\agents\deferral-validator.md`
- **purpose**: Decide whether each deferral names a real target, a reason its evidence supports, and a chain that does not return to this story.
- **tools**: Read, Glob, Grep
- **model**: `opus` — whether a stated reason matches the evidence is a reading, not a lookup.
- **input**: the drafted `deferrals[]` entries, the `deferrals[]` entries of every `reports/STORY-*-qa.yaml` on disk, the `sprint.yaml` `stories[]` list with `status`, the `adr/ADR-nnn.md` frontmatter `status` values, the story's `## Out of scope` lines, its id band.
- **output**:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "deferral-validator",
  "id": "STORY-014",
  "passed": 1,
  "total": 2,
  "unit": "deferrals",
  "findings": [
    { "id": "FIND-141", "severity": "block", "category": "deferral", "file": "", "line": 0,
      "relates_to": "CON-005", "summary": "STORY-031 defers the same item back to STORY-014",
      "evidence": "reports/STORY-031-qa.yaml deferrals[0].target is STORY-014 for CON-005" }
  ]
}
```

`total` is the number of deferral entries examined; `passed` is the number with a resolving target, a supported reason, and no return edge. A circular chain, an unresolved target, and a reason the evidence contradicts are each `block`.
- **invoked_at**: workflow step 6 over the deferrals on disk, and again at step 9 over the deferrals this run drafts.
- **registered_verifier**: yes

### security-auditor

- **name**: `security-auditor`
- **derives_from**: `C:\Users\bryan\.claude\agents\security-auditor.md`
- **purpose**: Examine the story's file set against the ten OWASP Top 10 categories and report each hit against the constraint that governs it.
- **tools**: Read, Grep, Glob
- **model**: `opus` — the judgement is whether a code path is reachable and exploitable, which a pattern alone does not decide.
- **input**: the `## Files` rows of `Kind` `source` and `config`, the `CON-nnn` rows of `kind: security`, the `AP-nnn` rows of `Category` `security`, the `## Forbidden dependencies` rows of `dependencies.md`, its id band.
- **output**:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "security-auditor",
  "id": "STORY-014",
  "passed": 9,
  "total": 10,
  "unit": "OWASP categories",
  "findings": [
    { "id": "FIND-161", "severity": "block", "confidence": 0.9, "category": "security", "file": "src/api/orders.ext",
      "line": 57, "relates_to": "CON-011", "owasp": "A01",
      "summary": "the handler reads the order id from the request and applies no ownership check",
      "evidence": "src/api/orders.ext:57 loads by id with no comparison against the session subject" }
  ],
  "payload": {}
}
```

`payload` is `{}`: this agent adds no field of its own, and the empty object is present rather than omitted so the envelope has one shape across all ten verifiers. `owasp` rides on the finding, where the id it qualifies is. `owasp` is a closed enum of ten values: `A01` Broken Access Control, `A02` Cryptographic Failures, `A03` Injection, `A04` Insecure Design, `A05` Security Misconfiguration, `A06` Vulnerable and Outdated Components, `A07` Identification and Authentication Failures, `A08` Software and Data Integrity Failures, `A09` Security Logging and Monitoring Failures, `A10` Server-Side Request Forgery. `total` is `10`; `passed` is the number of categories with no `block` finding, so a `warn` finding leaves `passed` where it was. A reachability the agent is unsure of is still a finding, at `severity: warn` and a `confidence` that says how far the reading carries, rather than one it drops. This subagent runs no command and reads no network.
- **invoked_at**: workflow step 7, deep mode, in parallel with two others.
- **registered_verifier**: yes

### code-quality-auditor

- **name**: `code-quality-auditor`
- **derives_from**: `C:\Users\bryan\.claude\agents\code-quality-auditor.md`
- **purpose**: Report functions above the complexity ceiling and duplicated runs above the duplication ceiling, both from `config.toml`.
- **tools**: Read, Grep, Glob, Bash — an unscoped `Bash`, admitted by the `PreToolUse` metrics arm alone, which permits exactly the `[verify].metrics_command` string of `config.toml` for this `agent_type` and denies every other command.
- **model**: `sonnet` — the numbers are measured here and compared by the gate.
- **input**: the `## Files` rows of `Kind` `source`, `config.toml` `[verify].complexity_max`, `[verify].duplication_max_percent`, `[verify].duplication_min_lines`, `[verify].metrics_command`, its id band.
- **output**:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "code-quality-auditor",
  "id": "STORY-014",
  "passed": 5,
  "total": 6,
  "unit": "files",
  "findings": [
    { "id": "FIND-181", "severity": "warn", "confidence": 0.7, "category": "complexity", "file": "src/application/checkout.ext",
      "line": 22, "relates_to": "CON-009", "measured": 14, "limit": 10,
      "summary": "one function branches 14 ways against a ceiling of 10",
      "evidence": "src/application/checkout.ext:22-118 holds 13 branch keywords on one function body" }
  ],
  "payload": { "method": "grep", "over_ceiling": 1 }
}
```

`payload.method` is the closed enum `command` and `grep`. With `[verify].metrics_command` non-empty the subagent runs it through `Bash` and reads its `devforgeai-metrics/1` JSON, setting `payload.method: command`; with the key `""` it counts branch keywords and repeated line runs with Grep and sets `payload.method: grep`. `measured` and `limit` ride on the finding, where the file and the line they describe are: `measured` is the figure read and `limit` is `complexity_max` on a `complexity` finding and `duplication_max_percent` on a `duplication` one. Copying `limit` in is transcription, not judgment — the comparison that decides a gate is the `complexity_clean` check reading `config.toml`.

Both findings are `warn` and `passed` equals `total`, both the number of `Kind` `source` rows read: every file examined counts as passed, and the count of files over either ceiling rides in `payload.over_ceiling`, which `gates.toml` can read as `verifiers.code_quality.payload.over_ceiling`.
- **invoked_at**: workflow step 7, deep mode, in parallel.
- **registered_verifier**: yes

### adr-conformance-reviewer

- **name**: `adr-conformance-reviewer`
- **derives_from**: `C:\Users\bryan\.claude\agents\architect-reviewer.md`
- **purpose**: Decide whether the story's implementation conforms to each accepted ADR whose constraints its file set touches.
- **tools**: Read, Grep, Glob
- **model**: `opus` — a decision record is prose and conformance to it is a reading.
- **input**: every `adr/ADR-nnn.md` at `status: accepted` with `## Decision`, `## Consequences`, `## Constraints introduced`; the story's `## Files`, `## Layer`, `## Constraints`; the `## Layer dependency rules` table; its id band.
- **output**:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "adr-conformance-reviewer",
  "id": "STORY-014",
  "passed": 2,
  "total": 3,
  "unit": "ADRs",
  "findings": [
    { "id": "FIND-201", "severity": "warn", "category": "constraint", "file": "src/infrastructure/store.ext",
      "line": 12, "relates_to": "ADR-002", "summary": "the adapter carries the decision logic ADR-002 places in the application layer",
      "evidence": "src/infrastructure/store.ext:12-40 branches on order state, which ADR-002 assigns to application" }
  ]
}
```

`total` is the number of accepted ADRs whose `## Constraints introduced` names a `CON-nnn` the story's `## Constraints` also names; `passed` is the number with no `block` finding. `WebFetch` leaves the tool list, because a registered verifier running unattended under `SubagentStop` reads no network. `AskUserQuestion` leaves it because it could not have stayed: Claude Code strips `AskUserQuestion` from every subagent whatever its `tools` list holds, so the tool could not have been kept: the question belongs to the invoking skill.
- **invoked_at**: workflow step 7, deep mode, in parallel.
- **registered_verifier**: yes

### What each adaptation changes

| Existing agent | New name | What changes, and why |
|---|---|---|
| `ac-compliance-verifier.md` | `ac-compliance-verifier` | Kept by name, because `specs/01-cli.md` registers it at `phase = "verify"` and the default gate names it. Its output becomes the `devforgeai/verifier/1` object, so `SubagentStop` can ingest it; its id band replaces any id it chose for itself. |
| `code-reviewer.md` | `standards-reviewer` | The name changes because the review is against one named document rather than general seniority. `Write` and `Bash(git:*)` leave the tool list: the diff is the story's `## Files` table, and the observation write to `devforgeai/feedback/ai-analysis/` leaves entirely, because the report is the record. The seven review categories become the seven headings of `coding-standards.md`, which the project wrote. |
| `anti-pattern-scanner.md` | `anti-pattern-scanner` | Kept by name. The six built-in violation categories and the eleven built-in smell types leave, replaced by the `## Anti-pattern index` rows, which carry their own `Category`, `Severity`, `Scope`, `Detector kind`, and `Detector`; the language list and the external-tool commands leave with them, per §1.2. Its four-value upper-case severity enum becomes the index's `blocker`, `high`, `medium`, `low`. Its JSON keys `status`, `violations`, `blocks_qa`, `blocking_reasons`, `recommendations` become the `devforgeai/verifier/1` keys, because `blocks_qa` is a gate decision and §1.1 places that in the CLI. |
| `context-validator.md` | `constraint-auditor` | The name changes because Build's context checking derives from the same file and runs before a commit. The scope narrows from all six context files to `architecture-constraints.md` alone; the other five are read by `standards-reviewer`, `anti-pattern-scanner`, and `dead-code-detector`. The `spec-gap` outcome is added, because this phase reports a constraint that decides nothing as a criterion gap rather than a code defect. |
| `coverage-analyzer.md` | `coverage-gap-auditor` | Every `Bash` scope leaves the tool list, so the subagent runs no coverage command; the numbers arrive in the build report, which `devforgeai gate check --phase build` produced. The three built-in thresholds and the `business_logic`/`application`/`infrastructure`/`unknown` layer set leave, replaced by the four `config.toml` `[[layer]]` names and their `coverage_min` values. What remains is the one judgement: which uncovered region leaves an `AC-nnn` untested. |
| `dead-code-detector.md` | `dead-code-detector` | Kept by name. `Bash(treelint:*)` becomes `Bash(devforgeai:*)` plus the `config.toml` command of Decision 26, so the tool is a project setting rather than a name in a subagent. The framework-specific entry-point markers leave, replaced by the `## Roots` and `## Generated and excluded paths` entries of `source-tree.md`. `model` moves from `inherit` to `sonnet`, so the judgement does not change with the caller. The `confidence` field and the suppression rule stay, which is the part that carries the fallback. |
| `deferral-validator.md` | `deferral-validator` | Kept by name, because Release registers `deferral-auditor` for a different question. `AskUserQuestion` leaves the tool list because it could not have stayed: Claude Code strips `AskUserQuestion` from every subagent whatever its `tools` list holds, so the tool could not have been kept: the question belongs to the invoking skill. A registered verifier runs unattended under `SubagentStop` in any case, so an ambiguous deferral becomes a `block` finding instead of a question. The four free-text reason formats become the five-value `reason` enum of `## Outputs`, so the reason is machine-readable for Reflect. The four-level severity ladder collapses to the `block` and `warn` the contract carries. |
| `security-auditor.md` | `security-auditor` | Kept by name. Every `Bash` scope leaves the tool list, which removes the three named package-manager audit commands, per §1.2; dependency risk reaches this phase through the `## Forbidden dependencies` rows of `dependencies.md` instead. The Markdown report becomes the `devforgeai/verifier/1` object with the `owasp` field. The security score out of 100 leaves, because a number no gate reads is ceremony. |
| `code-quality-auditor.md` | `code-quality-auditor` | Kept by name. The seven named metric binaries leave the tool list, replaced by `Bash(devforgeai:*)` and `[verify].metrics_command`, with the Grep fallback of Decision 26; `Grep` and `Glob` join the list, which the original lacked and the fallback needs. The three built-in thresholds become `[verify].complexity_max`, `[verify].duplication_max_percent`, and `[verify].duplication_min_lines`. `blocks_qa` and `blocking_reasons` leave, because §1.1 places the gate decision in the CLI. |
| `architect-reviewer.md` | `adr-conformance-reviewer` | The name changes because Constitute registers `architecture-reviewer`. `WebFetch` and `AskUserQuestion` leave the tool list: a registered verifier reads no network and asks nothing. The built-in principle set and the named anti-patterns leave, replaced by the accepted `ADR-nnn` documents, which are the project's own decisions. The Markdown report becomes the `devforgeai/verifier/1` object. |

### Agents read and replaced

| Existing file | Disposition | Reason |
|---|---|---|
| `C:\Users\bryan\.claude\agents\security-auditor\references\owasp-patterns.md` | replaced | It is a pattern library keyed to three named languages, which §1.2 excludes. The ten categories are the closed `owasp` enum in `security-auditor`'s schema, and the patterns are the `Detector` cells of `anti-patterns.md` rows whose `Category` is `security`, which the project itself wrote. |
| `C:\Users\bryan\.claude\agents\qa-result-interpreter.md` | replaced | It composes the user-facing result block and picks one of eight display templates. `devforgeai handoff` renders that block from `state.toml` and the report, per §3 and §6, and the model composes none of it. Its `mode` enum `light | deep` is kept as the QA report's `mode` field. |
| `C:\Users\bryan\.claude\agents\security-auditor\references\treelint-security-patterns.md`, `coverage-analyzer\references\treelint-patterns.md`, `refactoring-specialist\references\treelint-refactoring-patterns.md`, `code-reviewer\references\treelint-review-patterns.md`, `references\treelint-search-patterns.md`, `anti-pattern-scanner\references\phase5-treelint-detection.md` | replaced | Each names one external tool and pins its version, which §1.2 excludes from a skill, command, subagent, or hook. The capability the six describe, an AST-aware call graph and an AST-aware metric, becomes `config.toml` `[verify].call_graph_command` and `[verify].metrics_command`, each defaulting to `""` with the Grep fallback stated in the two subagent schemas that read them. A project that has such a tool sets the key; a project that does not gets the fallback and a `method: grep` field that says so. |

## Command

The entry point is the skill itself: `skills/validating-quality/SKILL.md`, installed to `.claude/skills/verify/`, whose frontmatter `name` is the slash command. There is no command file. This is the frontmatter and the preamble, verbatim, with four preamble lines.

```markdown
---
name: verify
description: Phase 5 of DevForgeAI, run by /verify. Reviews one built story from a context that did not write the code and records what it found in .devforgeai/reports/STORY-nnn-qa.yaml, holding FIND-nnn findings, their dispositions, and the deferrals[] sequence of Definition-of-Done items pushed to a later story. Light mode is seven judgements - acceptance-criteria compliance, a code review against coding-standards.md, an anti-pattern scan against the AP-nnn index, constraint validation against the CON-nnn index, a coverage review of the figures Build measured, dead-code detection, and deferral validation - and --deep adds a security audit, complexity and duplication metrics, and an ADR conformance review. Reach for it whenever /verify is typed, whenever a built story is being reviewed, whenever a QA report, a FIND-nnn, a deferral, a blocker finding, or a spec gap is in play, and whenever a Release send-back arrives as /verify STORY-nnn --remedy FIND-nnn.
argument-hint: <STORY-nnn> [--deep] [--remedy FIND-nnn,FIND-nnn] [--resume]
allowed-tools: Bash(devforgeai:*), PowerShell(devforgeai:*), Read, Write, Glob, Grep, Agent
disable-model-invocation: true
---

!`devforgeai gate require verify $ARGUMENTS[0]`
!`devforgeai doc load story $ARGUMENTS[0]`
!`devforgeai doc load context all`
!`devforgeai report show $ARGUMENTS[0] build`
```

The `gate require` line leads, so a failing predecessor gate aborts the invocation before any document loads.

## CLI calls

| Call, exact arguments | Caller | When | Exit handling |
|---|---|---|---|
| `devforgeai gate require verify $ARGUMENTS[0]` | command `!` preamble | step 1 | 0 continues; 1 names the missing build gate and stops the command body |
| `devforgeai doc load story $ARGUMENTS[0]` | command `!` preamble | step 2 | 0 prints the story; 1 on `DFA-E200` stops the command body |
| `devforgeai doc load context all` | command `!` preamble | step 2 | 0 prints the six files; 1 on `DFA-E200` stops the command body |
| `devforgeai report show $ARGUMENTS[0] build` | command `!` preamble, and the model at step 3 on a resume run | steps 1, 3 | 0 prints the build report; 1 on `DFA-E400` or `DFA-E401` stops the run |
| `devforgeai phase set verify --id <STORY-nnn>` | model, Bash | steps 4, R4 | 0 sets the phase and the story `status` to `built`; 1 on `DFA-E320` stops the run |
| `devforgeai doc validate --allocate FIND` | model, Bash | step 5, once per run | 0 prints `FIND-nnn`; 1 on `DFA-E215` stops the run |
| `devforgeai report ingest <subagent> -` | `SubagentStop` hook | steps 6, 7, 9 | 0 in every case; an unparsed block carries `status: unparsed` |
| `devforgeai report show <vX.Y.Z> release` | model, Bash | step R1 | 0 prints the release gate report; 1 means the cited ids come from `$ARGUMENTS` alone |
| `devforgeai gate check --phase verify` | `Stop` hook | step 11 | 0 PASS, 1 FAIL, 2 SEND BACK |
| `devforgeai handoff` | `Stop` hook | step 11 | 0 in every case |
| `devforgeai doc validate <path>` | `PostToolUse` hook on `Write\|Edit\|NotebookEdit` under `.devforgeai/` | step 10 | returns the diagnostic in `hookSpecificOutput.additionalContext`; the model rewrites the key it names |
| `devforgeai doc validate --producer-check <path>` | `PreToolUse` hook on Write under `.devforgeai/` | step 10 | 1 maps to hook exit 2 and blocks a write to a document this phase does not produce |

Every call uses a conventions §4 name or a `specs/01-cli.md` addition already recorded there (`report ingest`, `doc validate --allocate`, `--producer-check`). The additions this spec proposes are the `no_cycle` check kind and the two `config.toml` tables below, both in `## Decisions`.

The `config.toml` tables this phase adds:

```toml
[verify]
mode = "light"                     # string; enum light | deep; default "light"; the mode a run with no --deep flag takes
complexity_max = 10                # integer; default 10; the per-function branch ceiling above which a complexity finding is raised
duplication_max_percent = 5.0      # float; default 5.0; the share of a file's lines that may repeat elsewhere in the file set
duplication_min_lines = 20         # integer; default 20; the shortest repeated run counted as duplication
metrics_command = ""               # string; default ""; prints one devforgeai-metrics/1 JSON object on stdout; "" selects the Grep fallback
call_graph_command = ""            # string; default ""; prints one devforgeai-callgraph/1 JSON object on stdout; "" selects the Grep fallback

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
```

`ac-compliance-verifier` already has its `[[verifier]]` table in the `specs/01-cli.md` `config.toml` default, at `phase = "verify"` with `report_field = "verifiers.ac_compliance"` and `unit = "ACs"`, and this spec leaves it as written. All ten entries carry `required = true`, which `config.toml` defines as "a `verifier_pass` check names it by `name`", and `verify-deep` names the three deep-mode entries. The flag does not vary with the mode: light mode reaches the same gate through `skip_when`, which records the check as `skip` with `reason: condition`, and a deep run with a missing block fails with `DFA-E316`.

## Gate

The `.devforgeai/gates.toml` entry for this phase. Checks `verify-docs`, `verify-acs`, `verify-ids`, and `verify-story-status` are `specs/01-cli.md` `## Gate` verbatim, with `verify-ids` gaining three prefixes; `verify-light`, `verify-deep`, `verify-blockers`, `verify-deferral-cycle`, `verify-coverage`, and `verify-lint` are added by this spec and proposed as an amendment to the default file in `## Decisions`. Every kind but one is an existing name reused; `no_cycle` is the one proposed kind.

```toml
[[gate]]
phase = "verify"
requires = "build"
on_fail = "fail"
send_back_to = "build"
description = "The QA report parses, every registered verifier passed, no blocker finding stands, every deferral resolves without a cycle, coverage by layer holds, lint is clean."

  [[gate.check]]
  kind = "doc_valid"
  id = "verify-docs"
  docs = ["reports/{id}-qa.yaml"]

  [[gate.check]]
  kind = "verifier_pass"
  id = "verify-acs"
  verifiers = ["ac-compliance-verifier"]
  min_ratio = 1.0
  on_fail = "send_back"

  [[gate.check]]
  kind = "verifier_pass"
  id = "verify-light"
  verifiers = ["standards-reviewer", "anti-pattern-scanner", "constraint-auditor",
               "coverage-gap-auditor", "dead-code-detector", "deferral-validator"]
  min_ratio = 1.0
  on_fail = "send_back"
  message = "a light-mode verifier reported a blocking finding for {id}"

  [[gate.check]]
  kind = "verifier_pass"
  id = "verify-deep"
  verifiers = ["security-auditor", "code-quality-auditor", "adr-conformance-reviewer"]
  min_ratio = 1.0
  on_fail = "send_back"
  skip_when = { path = "reports/{id}-qa.yaml", field = "mode", equals = "light" }
  message = "a deep-mode verifier reported a blocking finding for {id}"

  [[gate.check]]
  kind = "length_between"
  id = "verify-blockers"
  path = "reports/{id}-qa.yaml"
  field = "blockers"
  min = 0
  max = 0
  on_fail = "send_back"
  message = "{value} blocker findings stand for {id}"

  [[gate.check]]
  kind = "ids_resolve"
  id = "verify-ids"
  prefixes = ["FIND", "AC", "STORY", "ADR", "CON", "AP"]

  [[gate.check]]
  kind = "no_cycle"
  id = "verify-deferral-cycle"
  docs = ["reports/STORY-*-qa.yaml"]
  root = "{id}"
  from = "id"
  to = "deferrals[].target"
  prefix = "STORY"
  on_fail = "send_back"
  message = "a deferral chain returns to {id}"

  [[gate.check]]
  kind = "coverage_min"
  id = "verify-coverage"
  layers = ["domain", "application", "infrastructure", "interface"]
  overall = true
  source = "read"

  [[gate.check]]
  kind = "lint_clean"
  id = "verify-lint"
  stacks = []

  [[gate.check]]
  kind = "field_in_enum"
  id = "verify-story-status"
  path = "stories/{id}.md"
  field = "status"
  values = ["built", "verified"]
```

The gate's `send_back_to` is `build`, and Verify has two upstream targets. `gate check` resolves the report's `gate.send_back_to` from the blocking findings the same way the plan gate does: a blocking finding of `category` `spec-gap` resolves `plan`, every other blocking finding resolves `build`, and a mixed set resolves `build`, because a defect in the code is repaired before the criterion that failed to catch it. This resolution rule is proposed in `## Decisions`.

### The mechanical and judgement split

| Property | Decided by | Rule |
|---|---|---|
| The QA report parses, carries the §5 keys in order, and its ids are unique | `devforgeai gate check`, check `verify-docs` | `DFA-E20x` |
| Every `FIND-nnn`, `AC-nnn`, `STORY-nnn`, `ADR-nnn`, `CON-nnn`, and `AP-nnn` the report cites resolves | `devforgeai gate check`, check `verify-ids` | `DFA-E325` |
| Every layer's covered-line share is at or above its `config.toml` floor, and the overall figure is at or above `[coverage].overall_min` | `devforgeai gate check`, check `verify-coverage` | `DFA-E313`, from the coverage artifact Build produced, with no command run |
| The project's lint command exits 0 | `devforgeai gate check`, check `verify-lint` | `DFA-E315`; `skip` with `reason: no_lint_command` when the stack has none |
| No blocker finding stands | `devforgeai gate check`, check `verify-blockers` | `DFA-E331` on a non-empty `blockers` list |
| Every deferral target resolves to a document | `devforgeai gate check`, check `verify-ids` | `DFA-E325` |
| No deferral chain returns to the story that opened it | `devforgeai gate check`, check `verify-deferral-cycle` | `DFA-E340` |
| Every registered verifier ingested a block and passed every unit | `devforgeai gate check`, checks `verify-acs`, `verify-light`, `verify-deep` | `DFA-E316` on an absent block, `DFA-E317` below `min_ratio` |
| The story is at `status: built` | `devforgeai gate check`, check `verify-story-status` | `DFA-E322` |
| Whether a test reads the outcome an `AC-nnn` names | `ac-compliance-verifier` | `block` when no test reads it, cited by `AC-nnn` |
| Whether code satisfies a prose rule of `coding-standards.md` | `standards-reviewer` | `warn`, cited by `CON-nnn` |
| Whether an `AP-nnn` detector hit is a true match in context | `anti-pattern-scanner` | `block` at `Severity` `blocker`, `warn` otherwise, cited by `AP-nnn` |
| Whether the file set satisfies a `CON-nnn` statement | `constraint-auditor` | `block` on a violation of a `CON-nnn` the story binds, cited by `CON-nnn` |
| Whether the constraint set is silent on a case the story meets | `constraint-auditor` | `warn`, `category: spec-gap`, cited by `AC-nnn` |
| Which uncovered region leaves an `AC-nnn` untested | `coverage-gap-auditor` | `warn`, cited by `AC-nnn`; a layer below its floor is `block` |
| Whether an unreferenced symbol is reachable by dynamic dispatch | `dead-code-detector` | `warn` with a `confidence` value, cited by `CON-nnn` or `AP-nnn` |
| Whether a deferral's reason matches its evidence and its chain terminates | `deferral-validator` | `block` on an unsupported reason, an unresolved target, or a return edge |
| Whether an OWASP category is reachable and exploitable in this file set | `security-auditor` | `block`, cited by `CON-nnn` of `kind: security` |
| Whether a measured complexity or duplication figure warrants a rewrite | `code-quality-auditor` | `warn` above the `config.toml` ceiling |
| Whether the implementation conforms to an accepted ADR | `adr-conformance-reviewer` | `warn`, cited by `ADR-nnn` |

A `warn` finding lands in the report, sets no gate, and is carried forward by its `disposition`. A `block` finding lowers `passed` below `total`, fails the `verifier_pass` check that names its subagent, and produces the SEND BACK of the next section.

## Send-back

### To Build, exit 2

| Condition | Detected by | IDs cited | Handoff `Next` |
|---|---|---|---|
| SB-1 · An `AP-nnn` of `Severity` `blocker` matches a file of the story's `## Files` | `anti-pattern-scanner` finding, `severity: block` | the `FIND-nnn` of each such finding | `/build <STORY-nnn> --remedy <FIND ids>` |
| SB-2 · A `CON-nnn` the story binds is violated by its file set | `constraint-auditor` finding, `severity: block` | the `FIND-nnn` of each such finding | `/build <STORY-nnn> --remedy <FIND ids>` |
| SB-3 · No test reads the outcome an `AC-nnn` names | `ac-compliance-verifier` finding, `severity: block` | the `FIND-nnn` of each such finding | `/build <STORY-nnn> --remedy <FIND ids>` |
| SB-4 · A layer's covered-line share is below its `config.toml` floor | `coverage-gap-auditor` finding, `severity: block`, and check `verify-coverage` | the `FIND-nnn` of each such finding | `/build <STORY-nnn> --remedy <FIND ids>` |
| SB-5 · An OWASP category is reachable and exploitable in the file set | `security-auditor` finding, `severity: block`, deep mode | the `FIND-nnn` of each such finding | `/build <STORY-nnn> --remedy <FIND ids>` |
| SB-6 · A deferral names an unresolved target, an unsupported reason, or a chain that returns to this story | `deferral-validator` finding, `severity: block`, and checks `verify-ids`, `verify-deferral-cycle` | the `FIND-nnn` of each such finding | `/build <STORY-nnn> --remedy <FIND ids>` |

The return trip is `/verify <STORY-nnn> --resume`, on the `Then` line. Build consumes `--remedy FIND-nnn,...` and re-opens the cited findings alone; `specs/06-build.md` is written in parallel and this dependency is recorded in `## Decisions`. The story keeps `status: built` on this path, the code keeps every byte until Build edits it (§1.6), and `state.toml` keeps `[current].phase` of `verify`.

### To Plan, exit 2

| Condition | Detected by | IDs cited | Handoff `Next` |
|---|---|---|---|
| SB-7 · The code meets the `AC-nnn` as written and the criterion asserts less than its `REQ-nnn` states | `ac-compliance-verifier` finding, `category: spec-gap` | the `AC-nnn` of each such finding | `/plan <EPIC-nnn> --remedy <AC ids>` |
| SB-8 · A `CON-nnn` the story binds holds, and no `AC-nnn` of the story asserts the outcome the constraint requires | `constraint-auditor` finding, `category: spec-gap` | the `AC-nnn` the constraint's `Binds` cell names | `/plan <EPIC-nnn> --remedy <AC ids>` |

`<EPIC-nnn>` is the `epic` key of `.devforgeai/stories/sprint.yaml`. Plan receives this form at its Remedy workflow step R1, triages each criterion with `spec-gap-triager`, and returns `/build <STORY-nnn> --resume` when the criterion was rewritten or the send-back form of its own `## Send-back` when the gap is further upstream (`specs/05-plan.md` `## Send-back`, Received, row Verify). The return trip to this phase is `/verify <STORY-nnn> --resume` on the `Then` line. This phase cites the `AC-nnn`, not the `FIND-nnn`, because Plan re-opens criteria and the `FIND-nnn` stays in this report as the evidence for the rewrite.

### Received

| From | Command | Effect |
|---|---|---|
| Release | `/verify <STORY-nnn> --remedy FIND-nnn,...` | A deferral in this story's report blocks deployment on the release platform (`specs/09-release.md` check `release-deferrals`). The remedy workflow re-disposes the cited findings at R2, re-reviews their paths at R3, and closes at R4 with `/build <STORY-nnn> --remedy <FIND ids>` when code work remains and `/release vX.Y.Z --resume` when it does not |
| Release | `/verify <STORY-nnn> --remedy STORY-nnn` | The story is at `built` with no PASS verify report (`specs/09-release.md` check `release-stories`). R1 opens the full run from step 2 for that story, and the close is the ordinary PASS or SEND BACK block |
| Build | `/verify <STORY-nnn> --resume` | Build repaired the cited findings; the resume workflow re-enters at step 3 and re-reads the fresh build report |
| Plan | `/verify <STORY-nnn> --resume` | Plan rewrote a cited `AC-nnn`; the resume workflow re-enters at step 3 and step 6 re-reads the story through `doc load story` |

## Integration

| Skill | Consumes (doc, IDs) | Produces for (doc, IDs) | Sends back to (condition) | Receives send-back from (condition) | Shared subagents | `state.toml` fields read / written |
|---|---|---|---|---|---|---|
| 0 Explore · `exploring-ideas` | none — Explore's `brief.md` and `decision.yaml` reach this phase only as the ancestors of a `REQ-nnn`, and a finding cites the `AC-nnn` or `CON-nnn` in front of it, not the idea | none — Explore runs before any story exists and reads no QA report | none — §5 gives Verify two send-back targets, Build and Plan, and a defect in an idea is four phases upstream | none — §5 gives Explore no downstream send-back | none | none |
| 1 Discover · `discovering-requirements` | none directly — a `REQ-nnn` reaches this phase through the story's `## Requirements` table, which Plan copied byte for byte; this phase reads the story, not `requirements.yaml` | `.devforgeai/reports/STORY-nnn-qa.yaml` `findings[]` of `category: spec-gap`, which Plan turns into a `/discover <IDEA-nnn> --remedy REQ-nnn,...` when its own triage places the gap in the requirement | none — a requirement gap leaves as a `spec-gap` finding cited to Plan, which owns the criterion and decides whether the gap is further upstream | none — Discover emits no send-back downstream of Plan | none | none |
| 2 Constitute · `establishing-context` | `.devforgeai/context/coding-standards.md` all seven sections; `anti-patterns.md` `## Anti-pattern index` (`AP-nnn`) with `Category`, `Severity`, `Scope`, `Detector kind`, `Detector`, `Source`; `architecture-constraints.md` `## Constraints`, `## Layer dependency rules`, `## Constraint index` (`CON-nnn`); `source-tree.md` `## Roots`, `## Layers`, `## Generated and excluded paths`; `tech-stack.md`; `dependencies.md` `## Forbidden dependencies`; `adr/ADR-nnn.md` at `status: accepted` (`ADR-nnn`) in deep mode | `.devforgeai/reports/STORY-nnn-qa.yaml` `findings[]` whose `relates_to` is a `CON-nnn`, `AP-nnn`, or `ADR-nnn`, which is the record of which rule a project actually breaks; Constitute reads it through Plan at its remedy step R2 | none — §5 gives Verify's send-back targets as Build and Plan; a constraint that decides nothing leaves as a `spec-gap` finding to Plan, which forwards to Constitute as SB-3 | none — Constitute runs before any story exists | none — Constitute owns `alignment-auditor` and `architecture-reviewer`; this phase's `adr-conformance-reviewer` and `constraint-auditor` are separate registrations with different inputs and a different question | none |
| 3 Plan · `planning-work` | `.devforgeai/stories/STORY-nnn.md`: the ten headings, `## Acceptance Criteria` (`AC-nnn`), `## Files`, `## Layer`, `## Constraints`, `## Anti-patterns`, `## Interface`, `## Out of scope`, and frontmatter `status` and `consumes`; `.devforgeai/stories/sprint.yaml` `epic`, `stories[]` of `id`, `status`, `order` | `.devforgeai/reports/STORY-nnn-qa.yaml` `findings[]` of `category: spec-gap`, cited by `AC-nnn`, which Plan re-opens at its Remedy step R1 through `report show <STORY-nnn> verify` | yes: SB-7 the code meets the `AC-nnn` and the criterion asserts less than its `REQ-nnn`, SB-8 a `CON-nnn` holds and no criterion asserts it; leaves as `/plan <EPIC-nnn> --remedy AC-nnn,...` | yes: Plan rewrote a cited criterion; arrives as `/verify <STORY-nnn> --resume` and the resume workflow re-enters at step 3 | none — Plan owns `story-decomposer`, `story-file-set-planner`, `sprint-sequencer`, `story-invest-auditor`, and `spec-gap-triager`, which judge specifications, and this phase invokes none of them | reads `[active].plan` for the `EPIC-nnn` of an SB-7 or SB-8 `Next` line |
| 4 Build · `implementing-stories` | `.devforgeai/reports/STORY-nnn-build.yaml`: `gate.result`, `gate.checks[]` of `id` and `status`, `coverage.format`, `coverage.source`, `coverage.overall`, `coverage.layers[]` of `name`, `covered`, `total`, `percent`, `min`, `status`, `coverage.unassigned` of `files` and `percent`; the file is CLI-written, so Build adds no key to it; the source and test files Build wrote, read through the story's `## Files` rows | `.devforgeai/reports/STORY-nnn-qa.yaml` `findings[]` (`FIND-nnn`) with `file`, `line`, `severity`, `disposition`, and `evidence`, which is the remedy list `/build <STORY-nnn> --remedy FIND-nnn,...` re-opens | yes: SB-1 through SB-6, every blocking finding whose remedy is code; leaves as `/build <STORY-nnn> --remedy FIND-nnn,...` with `/verify <STORY-nnn> --resume` on the `Then` line | yes: Build repaired the cited findings; arrives as `/verify <STORY-nnn> --resume` | `constraint-auditor` shares the `context-validator` lineage that Build's context checking also derives from; the two are distinct registrations, Build's running before a commit and this one running with no write tool, and neither invokes the other | reads `[active].build` through `gate require verify`; writes `[active].verify` and `[current]` through `phase set verify` |
| 5 Verify · `validating-quality` | self | self | self | self | self | reads `[current].phase`, `[active].build`, `[active].verify`; `phase set verify` writes `[current]` and `[active].verify`; `gate check --phase verify` writes `[last_gate]` |
| 6 Release · `releasing-software` | `.devforgeai/reports/vX.Y.Z-release.yaml` `findings[]` on the received-remedy path alone, for the summary of what blocked deployment | `.devforgeai/reports/STORY-nnn-qa.yaml`: `findings[]` entries of `id`, `severity`, `summary`, and the deferral marker, which is `disposition: defer` plus the `deferrals[]` entry of the same `id` carrying `reason`; `.devforgeai/reports/STORY-nnn-verify.yaml` `gate.result`, which Release's `release_stories` check requires to be `PASS` | none — §5 gives Verify's send-back targets as Build and Plan, and Release runs downstream | yes: a deferral blocks deployment (`release-deferrals`) or a story has no PASS verify report (`release-stories`); arrives as `/verify <STORY-nnn> --remedy FIND-nnn,...` or `--remedy STORY-nnn`, with `/release vX.Y.Z --resume` on the `Then` line | `deferral-validator` here and `deferral-auditor` in Release both derive from the same existing agent; they are distinct registrations at different phases asking different questions, and neither invokes the other | reads `[active].verify`; `[active].release` is written by Release |
| Design · `designing-interfaces` | `.devforgeai/ui-specs/UI-nnn.md` `## Accessibility`, whose eight rows `Landmark`, `Heading order`, `Name`, `Role`, `Keyboard path`, `Focus visible`, `Contrast`, `Motion` are the checks `standards-reviewer` applies to a file of the story's `## Files` whose `Layer` is `interface`; `## States` and `## Interaction`, for the outcomes an `AC-nnn` asserts | `.devforgeai/reports/STORY-nnn-qa.yaml` `findings[]` whose `relates_to` is an `AC-nnn` covering a `UI-nnn` state, which Design reads through Plan when Plan calls `/design UI-nnn --remedy AC-nnn,...` | none — a screen spec gap leaves as a `spec-gap` finding cited by `AC-nnn` to Plan, which calls Design inside its own run | none — Design holds no gate and emits no send-back to this phase | none — Design owns `mockup-designer`, `brand-designer`, and `requirement-coverage-auditor`, and this phase invokes none of them | none |
| Reflect · `improving-framework` | none — Reflect returns recommendations, which §5 excludes from gates | `.devforgeai/reports/STORY-nnn-qa.yaml` `findings[]` (`FIND-nnn`) and the top-level `deferrals[]` sequence, from which Reflect's `technical_debt` section is built; `.devforgeai/reports/STORY-nnn-verify.yaml` `checks[]`, `verifiers[]`, and `findings[]`, which its aggregate reads through `report show` | none — a framework observation is not a defect in a story | none — Reflect emits recommendations, not send-backs | none — Reflect owns `debt-aggregator`, which counts deferrals long after this phase validated them | none |
| CLI · `devforgeai` | `.devforgeai/config.toml` keys `[verify].*`, `[[layer]].coverage_min`, `[coverage].overall_min`, `[[verifier]]`, `lint_command`, `coverage_paths`, `coverage_format`; `.devforgeai/gates.toml` `[[gate]]` with `phase = "verify"`; `.devforgeai/state.toml` `[current]`, `[active].build`, `[active].verify` | `.devforgeai/reports/STORY-nnn-qa.yaml` for `doc validate` and the `verify-docs`, `verify-blockers`, `verify-ids`, and `verify-deferral-cycle` checks; the subagent stdout `report ingest` parses into `reports/STORY-nnn-verify.yaml`; `state.toml` through `phase set verify` | not applicable | not applicable | none — the CLI invokes no subagent and ingests the output of registered ones | reads `[current].phase`, `[active].build`; `phase set verify` writes `[current]` and `[active].verify`; `gate check` writes `[last_gate]`; `handoff` writes `[last_handoff]` |

## Handoff

Printed by `devforgeai handoff`. The phase document for the `Full report:` line is the CLI's gate report, per `specs/01-cli.md` Decision 46. The `Phase` line slug is the first H1 of `stories/STORY-nnn.md`. The `Done` line for this phase is `<checks> checks · <findings> findings · <deferrals> deferrals`, read from the QA report's `summary`.

PASS, a light run on a clean story with one deferral, ten lines:

```
Phase     5 · Verify          STORY-014 · order-checkout
Done      7 checks · 2 findings · 1 deferral
Gate      PASS  10 checks
Verified  ac-compliance-verifier · 7/7 ACs

Next      /build STORY-015
Then      /verify STORY-015
Blocked   none

Full report: .devforgeai/reports/STORY-014-verify.yaml
```

`Next` names the `sprint.yaml` `stories[]` entry with the lowest `order` whose `status` is `ready`. When no entry is at `ready`, `Next` is `/release vX.Y.Z`, where `X.Y.Z` is the highest version among the `.devforgeai/releases/v*.yaml` filenames with the minor incremented and the patch set to `0`, and `v0.1.0` when that directory holds no file. `Then` is `/verify <the same STORY-nnn>` on the first form and omitted on the second, because Release's own `Then` is `/reflect vX.Y.Z`.

SEND BACK to Build, a light run with two blocking findings, twelve lines:

```
Phase     5 · Verify          STORY-014 · order-checkout
Done      7 checks · 5 findings · 0 deferrals
Gate      SEND BACK to Build  verify-blockers 2, verify-light 4/6
Verified  anti-pattern-scanner · 4/6 anti-patterns
Found     FIND-061 AP-002 blocker: the application layer opens the order store directly
Found     FIND-021 AC-007 has no test reading the rejected-payment outcome

Next      /build STORY-014 --remedy FIND-061,FIND-021
Then      /verify STORY-014 --resume
Blocked   none

Full report: .devforgeai/reports/STORY-014-verify.yaml
```

Twelve lines, the §6 cap. The `Found` labels and the `--remedy` list are the `id` values of the blocking `findings[]` entries in the order `report ingest` stored them. A SEND BACK to Plan renders the same shape with `Gate      SEND BACK to Plan`, `Found` lines labelled by `AC-nnn`, `Next      /plan EPIC-004 --remedy AC-007`, and `Then      /verify STORY-014 --resume`.

## Templates

### templates/qa-report.yaml

    schema: devforgeai/qa-report/1
    id: STORY-nnn
    phase: verify
    status: pass
    produced_by: validating-quality
    consumes: []
    open_questions: []
    mode: light
    verified_on: YYYY-MM-DD
    build_report: .devforgeai/reports/STORY-nnn-build.yaml
    coverage:
      source: .devforgeai/reports/STORY-nnn-build.yaml
      overall: 0.0
      layers:
        - name: <domain | application | infrastructure | interface>
          percent: 0.0
          min: 0.0
          status: <pass | fail>
    checks:
      - name: <ac-compliance | code-review | anti-pattern-scan | constraint-validation | coverage-review | dead-code | deferral-validation | security-audit | quality-metrics | architecture-review>
        verifier: <subagent name>
        examined: 0
        passed: 0
        result: <pass | fail | skip>
    findings:
      - id: FIND-nnn
        category: <standards | anti-pattern | constraint | coverage | dead-code | deferral | ac-compliance | spec-gap | security | complexity | duplication>
        severity: <blocker | high | medium | low>
        file: <a Path value of the story ## Files table, or "">
        line: 0
        relates_to: <AC-nnn | CON-nnn | AP-nnn | ADR-nnn>
        disposition: <fix | defer | accept>
        summary: <one line, 1 to 120 characters>
        evidence: <one line, 1 to 200 characters, naming the path, the line, and what was read there>
        found_by: <subagent name>
    blockers: []
    deferrals:
      - id: FIND-nnn
        story: STORY-nnn
        dod_item: <AC-nnn, CON-nnn, AP-nnn, or layer name, then its one-line text>
        target: <STORY-nnn | ADR-nnn>
        reason: <dependency_missing | scope_boundary | decision_pending | external_blocker | tooling_absent>
        opened_on: YYYY-MM-DD
        con_or_ap: <CON-nnn | AP-nnn | "">
    summary:
      checks: 0
      findings: 0
      blockers: 0
      high: 0
      medium: 0
      low: 0
      deferrals: 0

## Evals

Three artifacts under `skills/validating-quality/evals/`, per §9.

### evals/evals.json

```json
{
  "skill": "validating-quality",
  "evals": [
    {
      "prompt": "/verify STORY-014",
      "expected_output": "One .devforgeai/reports/STORY-014-qa.yaml holding seven checks and a PASS handoff naming the next ready story.",
      "expectations": [
        "The file .devforgeai/reports/STORY-014-qa.yaml carries the sixteen top-level keys in the order schema, id, phase, status, produced_by, consumes, open_questions, mode, verified_on, build_report, coverage, checks, findings, blockers, deferrals, summary.",
        "The mode value is 'light' and the checks list holds seven entries whose name values are ac-compliance, code-review, anti-pattern-scan, constraint-validation, coverage-review, dead-code, deferral-validation.",
        "Every findings entry carries a category from the eleven-value enum, a severity from blocker, high, medium, low, and a disposition from fix, defer, accept.",
        "The coverage.overall value equals the coverage.overall value of .devforgeai/reports/STORY-014-build.yaml.",
        "No file under the project source roots differs from the bytes the case set up."
      ]
    },
    {
      "prompt": "/verify STORY-014 --deep",
      "expected_output": "A ten-entry checks list adding security-audit, quality-metrics, and architecture-review, with the OWASP categories as a closed set.",
      "expectations": [
        "The mode value is 'deep' and the checks list holds ten entries.",
        "Every security finding carries an owasp field whose value is one of A01 through A10.",
        "Every complexity and duplication finding carries a measured value and a limit value, and the limit equals the config.toml [verify] key for that kind.",
        "The transcript holds one invocation each of security-auditor, code-quality-auditor, and adr-conformance-reviewer."
      ]
    },
    {
      "prompt": "/verify STORY-018",
      "expected_output": "A blocker-severity anti-pattern match produces a SEND BACK to Build citing the FIND ids.",
      "expectations": [
        "The handoff Gate line reads 'SEND BACK to Build'.",
        "The handoff Next line begins '/build STORY-018 --remedy FIND-'.",
        "The handoff Then line is '/verify STORY-018 --resume'.",
        "The blockers list of .devforgeai/reports/STORY-018-qa.yaml holds every findings id whose severity is blocker, and every one of those findings carries disposition 'fix'."
      ]
    },
    {
      "prompt": "/verify STORY-022",
      "expected_output": "A criterion the code satisfies while its requirement asks for more produces a SEND BACK to Plan citing the AC id.",
      "expectations": [
        "The handoff Gate line reads 'SEND BACK to Plan'.",
        "The handoff Next line begins '/plan EPIC-' and holds '--remedy AC-'.",
        "The finding that produced it carries category 'spec-gap' and a relates_to value matching AC-nnn.",
        ".devforgeai/stories/STORY-022.md is byte-identical to the file the case set up."
      ]
    },
    {
      "prompt": "/verify STORY-025",
      "expected_output": "A deferred Definition-of-Done item is recorded with a target, a reason, and the constraint it cites.",
      "expectations": [
        "The deferrals list holds one entry whose id equals a findings entry with disposition 'defer'.",
        "That entry carries the keys id, story, dod_item, target, reason, opened_on, con_or_ap in that order.",
        "Its reason is one of dependency_missing, scope_boundary, decision_pending, external_blocker, tooling_absent.",
        "Its target matches STORY-nnn or ADR-nnn and a document with that id exists.",
        "Its opened_on equals the report's verified_on value."
      ]
    },
    {
      "prompt": "/verify STORY-027",
      "expected_output": "A deferral whose target defers the same item back is reported as a blocking finding and the gate sends back.",
      "expectations": [
        "The handoff Gate line reads 'SEND BACK to Build' and names verify-deferral-cycle or verify-light.",
        "One findings entry carries category 'deferral' and severity 'blocker'.",
        "Its evidence line names the path of the other story's QA report.",
        "The deferrals list of .devforgeai/reports/STORY-027-qa.yaml holds no entry whose target is STORY-031."
      ]
    }
  ]
}
```

### evals/cases.jsonl

Six cases, two of them on the SEND BACK path (`vq-03`, `vq-04`) and a third on the received-remedy path (`vq-06`). `FIXTURE:<name>` resolves to `skills/validating-quality/evals/fixtures/<name>`, which the runner copies into the workspace. Prior state travels in `expect.args` and in no other place.

```
{"id":"vq-01-light-pass","prompt":"/verify STORY-014","setup":{"files":{".devforgeai/config.toml":"FIXTURE:config-verify.toml",".devforgeai/gates.toml":"FIXTURE:gates-verify.toml",".devforgeai/state.toml":"FIXTURE:state-build-pass.toml",".devforgeai/stories/STORY-014.md":"FIXTURE:story-014-built.md",".devforgeai/stories/STORY-015.md":"FIXTURE:story-015-ready.md",".devforgeai/stories/sprint.yaml":"FIXTURE:sprint-active.yaml",".devforgeai/reports/STORY-014-build.yaml":"FIXTURE:build-pass-014.yaml",".devforgeai/context/coding-standards.md":"FIXTURE:coding-standards.md",".devforgeai/context/anti-patterns.md":"FIXTURE:anti-patterns-clean.md",".devforgeai/context/architecture-constraints.md":"FIXTURE:architecture-constraints.md",".devforgeai/context/source-tree.md":"FIXTURE:source-tree.md",".devforgeai/context/tech-stack.md":"FIXTURE:tech-stack.md",".devforgeai/context/dependencies.md":"FIXTURE:dependencies.md","src/domain/order.ext":"FIXTURE:order-clean.ext","tests/order_test.ext":"FIXTURE:order-test.ext"}},"expect":{"grader":"qa_report_shape","args":{"report":".devforgeai/reports/STORY-014-qa.yaml","mode":"light","check_names":["ac-compliance","code-review","anti-pattern-scan","constraint-validation","coverage-review","dead-code","deferral-validation"],"top_keys":["schema","id","phase","status","produced_by","consumes","open_questions","mode","verified_on","build_report","coverage","checks","findings","blockers","deferrals","summary"],"build_overall":87.4,"next_line":"/build STORY-015","source_sha256":{"src/domain/order.ext":"800031666769e0ce3d926f6f00bae3d2df5f6eb2d4ef71daaa21088105f1a7e5","tests/order_test.ext":"a3a4ba3449c8fb57bc7028039450abf7b4ff21d5a13653a4a5cd42060ca949d6"}}}}
{"id":"vq-02-deep-owasp","prompt":"/verify STORY-014 --deep","setup":{"files":{".devforgeai/config.toml":"FIXTURE:config-verify-thresholds.toml",".devforgeai/gates.toml":"FIXTURE:gates-verify.toml",".devforgeai/state.toml":"FIXTURE:state-build-pass.toml",".devforgeai/stories/STORY-014.md":"FIXTURE:story-014-built.md",".devforgeai/stories/sprint.yaml":"FIXTURE:sprint-active.yaml",".devforgeai/reports/STORY-014-build.yaml":"FIXTURE:build-pass-014.yaml",".devforgeai/adr/ADR-002.md":"FIXTURE:adr-002-accepted.md",".devforgeai/context/coding-standards.md":"FIXTURE:coding-standards.md",".devforgeai/context/anti-patterns.md":"FIXTURE:anti-patterns-clean.md",".devforgeai/context/architecture-constraints.md":"FIXTURE:architecture-constraints.md",".devforgeai/context/source-tree.md":"FIXTURE:source-tree.md",".devforgeai/context/tech-stack.md":"FIXTURE:tech-stack.md",".devforgeai/context/dependencies.md":"FIXTURE:dependencies.md","src/api/orders.ext":"FIXTURE:orders-complex.ext"}},"expect":{"grader":"deep_mode_checks","args":{"report":".devforgeai/reports/STORY-014-qa.yaml","check_names":["ac-compliance","code-review","anti-pattern-scan","constraint-validation","coverage-review","dead-code","deferral-validation","security-audit","quality-metrics","architecture-review"],"owasp_enum":["A01","A02","A03","A04","A05","A06","A07","A08","A09","A10"],"complexity_max":10,"duplication_max_percent":5.0,"subagents":["security-auditor","code-quality-auditor","adr-conformance-reviewer"]}}}
{"id":"vq-03-blocker-sendback","prompt":"/verify STORY-018","setup":{"files":{".devforgeai/config.toml":"FIXTURE:config-verify.toml",".devforgeai/gates.toml":"FIXTURE:gates-verify.toml",".devforgeai/state.toml":"FIXTURE:state-build-pass-018.toml",".devforgeai/stories/STORY-018.md":"FIXTURE:story-018-built.md",".devforgeai/stories/sprint.yaml":"FIXTURE:sprint-active-018.yaml",".devforgeai/reports/STORY-018-build.yaml":"FIXTURE:build-pass-018.yaml",".devforgeai/context/coding-standards.md":"FIXTURE:coding-standards.md",".devforgeai/context/anti-patterns.md":"FIXTURE:anti-patterns-ap002-blocker.md",".devforgeai/context/architecture-constraints.md":"FIXTURE:architecture-constraints.md",".devforgeai/context/source-tree.md":"FIXTURE:source-tree.md",".devforgeai/context/tech-stack.md":"FIXTURE:tech-stack.md",".devforgeai/context/dependencies.md":"FIXTURE:dependencies.md","src/application/checkout.ext":"FIXTURE:checkout-direct-store.ext"}},"expect":{"grader":"sendback_block","args":{"to":"Build","next_prefix":"/build STORY-018 --remedy FIND-","then_line":"/verify STORY-018 --resume","report":".devforgeai/reports/STORY-018-qa.yaml","blocker_relates_to":"AP-002","blocker_disposition":"fix","forbidden_substrings":["/plan ","--deep"],"source_sha256":{"src/application/checkout.ext":"fee8e7ebb3fd6df9c00a5ca2b49882d0633fe41d07fca351aa0d27518373109c"}}}}
{"id":"vq-04-spec-gap-sendback","prompt":"/verify STORY-022","setup":{"files":{".devforgeai/config.toml":"FIXTURE:config-verify.toml",".devforgeai/gates.toml":"FIXTURE:gates-verify.toml",".devforgeai/state.toml":"FIXTURE:state-build-pass-022.toml",".devforgeai/stories/STORY-022.md":"FIXTURE:story-022-thin-ac.md",".devforgeai/stories/sprint.yaml":"FIXTURE:sprint-active-022.yaml",".devforgeai/reports/STORY-022-build.yaml":"FIXTURE:build-pass-022.yaml",".devforgeai/context/coding-standards.md":"FIXTURE:coding-standards.md",".devforgeai/context/anti-patterns.md":"FIXTURE:anti-patterns-clean.md",".devforgeai/context/architecture-constraints.md":"FIXTURE:architecture-constraints-con011.md",".devforgeai/context/source-tree.md":"FIXTURE:source-tree.md",".devforgeai/context/tech-stack.md":"FIXTURE:tech-stack.md",".devforgeai/context/dependencies.md":"FIXTURE:dependencies.md","src/domain/limit.ext":"FIXTURE:limit-correct.ext"}},"expect":{"grader":"sendback_block","args":{"to":"Plan","next_prefix":"/plan EPIC-004 --remedy AC-","then_line":"/verify STORY-022 --resume","report":".devforgeai/reports/STORY-022-qa.yaml","require_category":"spec-gap","story_sha256":{".devforgeai/stories/STORY-022.md":"ffa8e9e4f8a59bea2b7912aacdb451582c6357c903dd0a71f5afcaf4fa479ae8"},"forbidden_substrings":["/build STORY-022 --remedy"]}}}
{"id":"vq-05-deferral-recorded","prompt":"/verify STORY-025","setup":{"files":{".devforgeai/config.toml":"FIXTURE:config-verify.toml",".devforgeai/gates.toml":"FIXTURE:gates-verify.toml",".devforgeai/state.toml":"FIXTURE:state-build-pass-025.toml",".devforgeai/stories/STORY-025.md":"FIXTURE:story-025-built.md",".devforgeai/stories/STORY-031.md":"FIXTURE:story-031-ready.md",".devforgeai/stories/sprint.yaml":"FIXTURE:sprint-active-025.yaml",".devforgeai/reports/STORY-025-build.yaml":"FIXTURE:build-pass-025.yaml",".devforgeai/context/coding-standards.md":"FIXTURE:coding-standards.md",".devforgeai/context/anti-patterns.md":"FIXTURE:anti-patterns-clean.md",".devforgeai/context/architecture-constraints.md":"FIXTURE:architecture-constraints-con005.md",".devforgeai/context/source-tree.md":"FIXTURE:source-tree.md",".devforgeai/context/tech-stack.md":"FIXTURE:tech-stack.md",".devforgeai/context/dependencies.md":"FIXTURE:dependencies.md","src/application/retry.ext":"FIXTURE:retry-partial.ext"}},"expect":{"grader":"deferral_recorded","args":{"report":".devforgeai/reports/STORY-025-qa.yaml","entry_keys":["id","story","dod_item","target","reason","opened_on","con_or_ap"],"reason_enum":["dependency_missing","scope_boundary","decision_pending","external_blocker","tooling_absent"],"target_must_exist":true,"con_or_ap":"CON-005","story":"STORY-025"}}}
{"id":"vq-06-release-remedy-cycle","prompt":"/verify STORY-027 --remedy FIND-024","setup":{"files":{".devforgeai/config.toml":"FIXTURE:config-verify.toml",".devforgeai/gates.toml":"FIXTURE:gates-verify.toml",".devforgeai/state.toml":"FIXTURE:state-release-sendback.toml",".devforgeai/stories/STORY-027.md":"FIXTURE:story-027-built.md",".devforgeai/stories/STORY-031.md":"FIXTURE:story-031-ready.md",".devforgeai/stories/sprint.yaml":"FIXTURE:sprint-active-027.yaml",".devforgeai/reports/STORY-027-build.yaml":"FIXTURE:build-pass-027.yaml",".devforgeai/reports/STORY-027-qa.yaml":"FIXTURE:qa-027-defer-031.yaml",".devforgeai/reports/STORY-031-qa.yaml":"FIXTURE:qa-031-defer-027.yaml",".devforgeai/reports/v0.3.0-release.yaml":"FIXTURE:release-sendback-030.yaml",".devforgeai/context/coding-standards.md":"FIXTURE:coding-standards.md",".devforgeai/context/anti-patterns.md":"FIXTURE:anti-patterns-clean.md",".devforgeai/context/architecture-constraints.md":"FIXTURE:architecture-constraints-con005.md",".devforgeai/context/source-tree.md":"FIXTURE:source-tree.md",".devforgeai/context/tech-stack.md":"FIXTURE:tech-stack.md",".devforgeai/context/dependencies.md":"FIXTURE:dependencies.md","src/application/retry.ext":"FIXTURE:retry-partial.ext"}},"expect":{"grader":"cycle_detected","args":{"report":".devforgeai/reports/STORY-027-qa.yaml","cited":"FIND-024","other_report":".devforgeai/reports/STORY-031-qa.yaml","forbidden_target":"STORY-031","gate_names":["verify-deferral-cycle","verify-light"],"next_prefix":"/build STORY-027 --remedy FIND-","prior_disposition":"defer"}}}
```

### evals/graders.py

Five pure functions, each `def <grader>(workspace: str, transcript: str, args: dict) -> tuple[bool, str]`, each returning `(passed, evidence)`. No network, no subprocess, no randomness. Each reads YAML with a minimal indentation-aware parser held in the file, because the runner installs no package.

| Function | Logic |
|---|---|
| `qa_report_shape(workspace, transcript, args)` | Load `args["report"]`. Fail unless its top-level keys equal `args["top_keys"]` in order. Fail unless `mode` equals `args["mode"]` and the `checks[].name` multiset equals `args["check_names"]`. Fail unless `coverage.overall` equals `args["build_overall"]`. Fail unless every `findings[]` entry's `category`, `severity`, and `disposition` lie in the three enums held in the file. For each path and digest in `args["source_sha256"]`, fail when the SHA-256 of the workspace file differs. Fail unless the transcript holds a line equal to `Next      ` plus `args["next_line"]`. Evidence names the first key, enum value, or digest that differed. |
| `deep_mode_checks(workspace, transcript, args)` | Load `args["report"]`. Fail unless `mode` is `deep` and the `checks[].name` multiset equals `args["check_names"]`. Fail unless every `findings[]` entry of `category: security` carries an `owasp` value in `args["owasp_enum"]`. Fail unless every entry of `category` `complexity` carries `limit == args["complexity_max"]` and every entry of `category` `duplication` carries `limit == args["duplication_max_percent"]`. Fail unless the transcript names each string of `args["subagents"]`. Evidence names the first entry that failed. |
| `sendback_block(workspace, transcript, args)` | Locate the last handoff block in the transcript, the twelve lines from a line starting `Phase     `. Fail unless its `Gate` line holds `SEND BACK to ` plus `args["to"]`, its `Next` line starts with `args["next_prefix"]`, and its `Then` line equals `args["then_line"]`. Fail when any string of `args["forbidden_substrings"]` appears in the block. Load `args["report"]`; with `args["blocker_relates_to"]` set, fail unless a `findings[]` entry of `severity: blocker` carries that `relates_to` and `disposition` equal to `args["blocker_disposition"]`, and unless `blockers` holds exactly the ids of the `severity: blocker` entries; with `args["require_category"]` set, fail unless a `findings[]` entry carries that `category` and a `relates_to` matching `AC-\d{3}`. For each path and digest in `args["story_sha256"]` or `args["source_sha256"]`, fail when the workspace file's SHA-256 differs. Evidence names the failing line or id. |
| `deferral_recorded(workspace, transcript, args)` | Load `args["report"]`. Fail unless `deferrals` holds at least one entry and every entry's key order equals `args["entry_keys"]`. Fail unless each entry's `reason` lies in `args["reason_enum"]`, its `story` equals `args["story"]`, its `opened_on` equals the report's `verified_on`, its `con_or_ap` equals `args["con_or_ap"]`, and its `id` equals a `findings[]` id whose `disposition` is `defer`. With `args["target_must_exist"]`, fail unless a file `stories/<target>.md` or `adr/<target>.md` exists under `.devforgeai/`. Evidence names the first entry that failed. |
| `cycle_detected(workspace, transcript, args)` | Load `args["report"]` and `args["other_report"]`. Fail unless a `findings[]` entry of `args["report"]` carries `category: deferral` and `severity: blocker`, and unless its `evidence` string holds the basename of `args["other_report"]`. Fail when any `deferrals[]` entry of `args["report"]` carries `target` equal to `args["forbidden_target"]`. Locate the last handoff block; fail unless its `Gate` line names one string of `args["gate_names"]` and its `Next` line starts with `args["next_prefix"]`. Fail unless the finding whose id is `args["cited"]` carries a `disposition` other than `args["prior_disposition"]`. Evidence names the surviving edge or the failing line. |

## Decisions

Every choice this spec made where conventions §1 through §10 were silent, every proposed addition to `specs/01-cli.md`, and every dependency on a spec written in parallel.

### Proposed additions

1. **One new gate check kind, `no_cycle`.** `specs/01-cli.md` Decision 53 closes the kind enum at twenty-one and none of the twenty-one detects a cycle across documents. `story validate` detects a story dependency cycle inside `sprint.yaml` (`DFA-E232`) and reads no QA report. Grammar, with every key typed:

```toml
[[gate.check]]
kind = "no_cycle"                   # string
id = "verify-deferral-cycle"        # string; unique within the gate
docs = ["reports/STORY-*-qa.yaml"]  # array[string]; required; globs resolved under .devforgeai/
root = "{id}"                       # string; default ""; with a value, only a cycle passing through that node fails the check
from = "id"                         # string; required; dotted path to the edge source in each matched document
to = "deferrals[].target"           # string; required; dotted path to the edge targets; [] iterates a sequence
prefix = "STORY"                    # string; default ""; only a target matching <prefix>-nnn forms an edge
```

   Passes when the directed graph whose nodes are the `from` values of the matched documents and whose edges are their `to` values holds no cycle through `root`, and, with `root` empty, when it holds no cycle at all. A `to` value naming a document outside `docs` is a leaf and forms no edge. Failure is `DFA-E340`, exit 1, message `deferral cycle: <A> -> <B> -> <A>`, the cycle through `root` with the fewest edges. `root` is set to `{id}` in this gate so that a cycle between two other stories does not fail the story under review, whose `/build <STORY-nnn> --remedy` send-back could not repair it. `DFA-E340` is free across `specs/01-cli.md`, `02` through `05`, `08`, `09`, and `10`.

2. **The `config.toml` `[verify]` table and nine `[[verifier]]` tables**, listed verbatim in `## CLI calls`. `[[verifier]]` is the register `specs/01-cli.md` Decision 11 defines, and `init` writes these nine beside the `ac-compliance-verifier` entry it already writes. `[verify]` is a phase-scoped table in the same shape as `[explore]` and `[plan]`.

3. **Six checks added to the default verify gate in `gates.toml`**: `verify-light`, `verify-deep`, `verify-blockers`, `verify-deferral-cycle`, `verify-coverage`, and `verify-lint`, and three prefixes added to `verify-ids`. Five of the six reuse an existing kind name. The four checks `specs/01-cli.md` already writes stand verbatim.

4. **The `Done` line for the verify phase** is `<checks> checks · <findings> findings · <deferrals> deferrals`, read from the QA report `summary`. `specs/01-cli.md` Decision 28 makes the per-phase `Done` counts CLI-owned and lists no value for verify.

5. **The `Next` and `Then` transition for the verify phase.** `Next` is `/build <STORY-nnn>` for the `sprint.yaml` `stories[]` entry with the lowest `order` whose `status` is `ready`, and `Then` is `/verify <that same STORY-nnn>`. When no entry is at `ready`, `Next` is `/release vX.Y.Z` where `X.Y.Z` is the highest version among the `.devforgeai/releases/v*.yaml` filenames with the minor incremented and the patch set to `0`, `v0.1.0` when that directory holds no file, and `Then` is omitted. The version is a proposal the user may replace by typing another, because `/release` takes the version as its argument; `releasing-software` chooses nothing about the number. This extends the transition table of `specs/01-cli.md` Decision 28.

6. **`verifier_pass` reads a `total` of `0` as a ratio of `1.0`, proposed as a clarification to `specs/01-cli.md`.** That spec defines the check as `passed / total` at or above `min_ratio` and leaves `0 / 0` undefined. This phase produces that pair routinely and correctly: `deferral-validator` on a first run with no deferral on disk, `adr-conformance-reviewer` with an empty `adr/` directory, `anti-pattern-scanner` when no `AP-nnn` `Scope` glob matches a `## Files` path, and `constraint-auditor` on a story whose `## Constraints` reads `none`. The compiled floor holds `min_ratio` at `1.0`, so lowering it is unavailable, and the alternative of making every verifier invent a synthetic unit would put a number in the report that measured nothing. The clarification is one line of `gate.rs`: a `total` of `0` yields a ratio of `1.0` and the check passes. The QA document records the same fact as `checks[].result: skip`.

7. **The send-back destination resolution for this gate.** The gate's static `send_back_to` is `build`. `gate check` resolves the report's `gate.send_back_to` from the blocking findings: a blocking finding of `category: spec-gap` resolves `plan`, every other blocking finding resolves `build`, and a set holding both resolves `build`, because a defect in the code is repaired before the criterion that failed to catch it. This mirrors the resolution rule `specs/05-plan.md` Decision 8 proposes for the plan gate, keyed on `category` rather than on an id prefix, because every finding of this phase carries a `FIND-nnn` and the prefix distinguishes nothing.

### Silences filled

8. **The finding category enum is closed at eleven**, one per source of judgement, listed in `## Outputs`. §5 gives Verify the `FIND-nnn` prefix and no field list.

9. **The finding severity enum is `blocker`, `high`, `medium`, `low`**, the `## Anti-pattern index` `Severity` enum of `specs/04-constitute.md`, so a finding raised from an `AP-nnn` copies that row's cell with no translation.

10. **The verifier stdout severity is the three-value `block | warn | info` of `specs/01-cli.md`, which this spec does not change.** The mapping is `blocker` to `block` and `high`, `medium`, and `low` to `warn`; `info` is emitted by no subagent of this phase. The four-value form lives in `reports/STORY-nnn-qa.yaml` and the three-value form in `reports/STORY-nnn-verify.yaml`. This is the mechanism by which a blocker-severity match fails the gate: a `block` finding lowers `passed` below `total` and `verifier_pass` runs at the compiled floor `min_ratio = 1.0`.

11. **The disposition enum is closed at three**, `fix`, `defer`, `accept`. A finding of `severity: blocker` takes `fix` and no other value, so a blocker cannot be deferred or accepted past the gate.

12. **A Definition-of-Done item is one of four things**: an `AC-nnn` of the story's `## Acceptance Criteria`, the layer coverage floor of its `## Layer`, a `CON-nnn` of its `## Constraints`, or an `AP-nnn` of its `## Anti-patterns`. The `dod_item` string opens with that id, or with the layer name for a coverage floor. §5 and §10 define neither the term nor its members.

13. **The deferral reason enum is closed at five**, listed in `## Outputs`. `capacity` is absent on purpose: `specs/05-plan.md` uses `capacity` for `sprint.yaml` `deferred[].reason`, which is a story left out of a sprint, and a Definition-of-Done item pushed to a later story is a different fact on a different document. Reusing the word across the two would make a Reflect aggregate that reads both ambiguous.

14. **`deferrals[]` is a top-level sequence of the QA document, not a sub-key of a finding.** `specs/10-reflect.md` Decision 29 reads it there, and a top-level sequence lets `report aggregate` walk every QA report without descending into `findings[]`.

15. **The divergence from `specs/10-reflect.md` Decision 29, stated exactly.** That decision assumes each `deferrals[]` item holds `dod_item` (string), `deferred_at` (`YYYY-MM-DD`), `reason` (string), and `constraint` (`CON-nnn`, `AP-nnn`, or absent). This spec emits seven fields: `id`, `story`, `dod_item`, `target`, `reason`, `opened_on`, `con_or_ap`. Three of the four assumed names land unchanged in substance: `dod_item` is byte-identical, `reason` is present and now drawn from a closed enum, and the sequence is at the top level of `reports/STORY-nnn-qa.yaml` as assumed. Two names differ: `deferred_at` is `opened_on`, and `constraint` is `con_or_ap`, which is present in every entry and holds `""` rather than being absent when the item is an `AC-nnn` or a coverage floor. Three fields are added: `id`, which ties the deferral to the `FIND-nnn` it came from; `story`, which lets an aggregate group without reading the filename; and `target`, which the deferral cycle check reads. By that decision's own terms the change lands in two places, `report aggregate`'s deferral reader and the `debt-aggregator` input list, and nothing else in `specs/10-reflect.md` moves.

16. **The finding shape `specs/09-release.md` Decision 30 assumes lands unchanged.** It reads `id`, `severity`, `summary`, and a deferral marker with the stated reason. The marker is `findings[].disposition` equal to `defer`, and the reason is the `reason` of the `deferrals[]` entry whose `id` equals that finding's `id`. `deferral-auditor` reads those four facts from two places in one file.

17. **`FIND-nnn` ids are allocated once per run in bands, not once per finding.** The subagents are read-only and hold no `Bash` tool for `doc validate --allocate`, and the number of findings is unknown before they run. The model allocates one `base` at step 5 and gives subagent `k` the band `FIND-(base + 100k)` through `FIND-(base + 100k + 99)`. The band is 100 wide and nothing truncates: a review agent reports every finding it has, and a cap at twenty would have made the tenth finding of a thorough pass invisible to the gate that exists to catch it. Gaps in the sequence cost nothing, because `doc validate --allocate` returns the next free id from the index rather than the next integer.

18. **The gate re-reads the coverage artifact rather than the build report YAML.** `coverage_min` takes `source = "read"`, the enum value `specs/01-cli.md` defines and the build gate does not use, which parses the most recently modified match of `config.toml` `coverage_paths` with no command run. The artifact is Build's output. When it is absent at verify time the check fails with `DFA-E312`, which is a gate failure about a missing artifact rather than about quality; this spec accepts that outcome rather than skipping the check, because a silent skip would let a story reach Release with no coverage evidence at all. The skill separately reads the `coverage:` block of `reports/STORY-nnn-build.yaml` through `report show` for the per-layer numbers it copies into the QA report and hands to `coverage-gap-auditor`; the skill runs no coverage command in either path.

19. **The `blockers` list and the `summary` counts are derived by the model, and no mechanical check compares the derivation against `findings[]`.** `doc validate` checks frontmatter, headings, ids, and cross-references, not arithmetic over a payload sequence. The gate reads the derived `blockers` list through `length_between`. The backstop is the verifier side: a blocker finding reaches the report only by way of a subagent whose `block` severity already lowered its `passed` below its `total`, which fails `verifier_pass` independently of the list. A model that omits an id from `blockers` therefore still fails the gate.

20. **`length_between` decides the blocker count, in preference to `report_metric`.** `length_between` already takes `path` and `field`, so the check needs no new key and no new error code; `report_metric` roots its metric at the gate's own report and would need a `path` key added.

21. **The deep-mode verifier check skips by condition rather than by a second gate.** `verify-deep` carries `skip_when = { path = "reports/{id}-qa.yaml", field = "mode", equals = "light" }` at `severity = "block"`. A required check kind marked `warn` is treated as absent with `DFA-E303`, so lowering the severity is not an option. When the QA report is absent the `verify-docs` check fails first; the condition then reads a field of a missing document, which is `DFA-E339`, and both failures name the same missing file.

22. **The story status enum value `verified` is written by nothing.** `specs/05-plan.md` `## Outputs` states that `verified` is written by the Verify skill. Three facts stand against it: `devforgeai phase set verify` writes `built` and `phase set release` writes `released` (`specs/01-cli.md` §`phase set`), the PreToolUse producer check blocks a write from this phase to `stories/STORY-nnn.md`, which is Plan's document, and §2 forbids a status-transition instruction in skill prose. The resolution is that a story moves `ready` to `building` to `built` to `released`, and no project writes `verified`. The `verify-story-status` check keeps the CLI's `values = ["built", "verified"]` verbatim, because an unreachable value constrains nothing and changing the default file for it would be churn. That sentence of `specs/05-plan.md` is declined.

23. **Deep mode is a flag on `/verify`, not a second command.** §4b allows a second entry point as a flag on the one command and forbids a second command.

24. **`--deep` defaults from `config.toml` `[verify].mode`.** A project that reviews deeply on every story sets `mode = "deep"` once rather than typing the flag, and the handoff `Next` line stays typable verbatim per §1.7, since the flag is absent from it in both modes.

25. **The skill invokes `ac-compliance-verifier` although the task list for light mode does not name it.** `specs/01-cli.md` already registers that subagent at `phase = "verify"` and its default verify gate names it in the `verify-acs` check at `min_ratio = 1.0`. A run that did not invoke it would fail its own gate with `DFA-E316`.

26. **Ten subagents, all registered verifiers, all read-only.** Two hold `Bash(devforgeai:*)` for one purpose each, running the optional `config.toml` command they are told about; neither holds `Write` or `Edit`, so no subagent of this phase can change a byte of the project.

27. **Treelint stays out of the framework and its capability becomes two optional `config.toml` commands.** Six reference files under `C:\Users\bryan\.claude\agents\*\references\` describe AST-aware call-graph and metric patterns keyed to that one tool, each pinning a version in its frontmatter. §1.2 forbids a skill, command, subagent, or hook from naming a tool. Three of the agents that load them, `anti-pattern-scanner`, `security-auditor`, and `coverage-analyzer`, instruct that tool's commands while granting no `Bash` scope that could run them, so those three already run the Grep path in every project; the two that do hold the scope, `code-quality-auditor` and `dead-code-detector`, are the two whose adapted forms read the optional `config.toml` command here. The capability survives as `[verify].call_graph_command` and `[verify].metrics_command`, each defaulting to `""`. With a key set, the subagent runs the command and reports `payload.method: command`; with the key `""`, `dead-code-detector` resolves symbols with Grep over the `## Roots` paths and reports `payload.method: grep` with a `confidence` of `0.6` for an exported definition and `0.9` otherwise, and `code-quality-auditor` counts branch keywords and repeated line runs with Grep and reports `payload.method: grep`. Both fallbacks run with the built-in tools alone, so a project with no such tool loses precision and loses no check.

**The mechanism that lets those two run a command is a hook, not a tool grant.** Each holds `Bash` in its `tools` list and holds no permission to use it: the `PreToolUse` shell arm reads the payload's `agent_type`, and for `code-quality-auditor` and `dead-code-detector` alone it compares the trimmed command against `[verify].metrics_command` and `[verify].call_graph_command`. An exact match returns `permissionDecision: "allow"`; anything else returns `deny` naming the `config.toml` key to edit. The project decides what those two agents may run, which is why the permission lives in `config.toml` rather than in a `Bash(tool:*)` scope that names a tool and breaks §1 rule 2.

**And neither agent applies the threshold it reads.** An agent may read a tool's numeric output only to copy it into its finding and its `payload` verbatim — `measured` and `limit` on a `code-quality-auditor` finding, `payload.dead` on a `dead-code-detector` block. Comparing that number against a ceiling is a `gate check` check kind reading `config.toml` and `gates.toml`. This is what conventions §2 means when it forbids an agent to run tests, coverage, or linters and interpret the numbers: the reading is the agent's and the judgment is the gate's.

28. **Two of the files read for this phase are replaced rather than adapted**, with the reasons in `## Subagents`: `security-auditor\references\owasp-patterns.md`, which is a reference file rather than an agent and is keyed to three named languages, and `qa-result-interpreter.md`, which composes a result block that `devforgeai handoff` renders. Eight of the ten agents read are adapted, each change recorded in the table in `## Subagents`; the remaining two subagents of this phase, `constraint-auditor` and `ac-compliance-verifier`, adapt `context-validator.md` and `ac-compliance-verifier.md`.

29. **Name collisions with other phases are resolved by renaming this phase's agent, in three places.** Constitute registers `architecture-reviewer`, so the ADR reviewer here is `adr-conformance-reviewer`. Release registers `deferral-auditor` at `phase = "release"`, so the deferral checker here keeps the existing name `deferral-validator` at `phase = "verify"`, which is the collision `specs/09-release.md` Decision 31 anticipated and which `specs/11-subagent-catalog.md` records as two registrations of one lineage. Build's context checking and this phase's `constraint-auditor` share the `context-validator` lineage and differ in name, phase, and tool list.

30. **A `findings[].id` entry defines its `FIND-nnn` in the ID index; it does not reference one.** `specs/01-cli.md` Decision 27 gives the three defining forms as a frontmatter `id`, a heading that starts with the id, and a YAML item key, and its doc-type table gives `qa-report` the ID prefix `FIND`. A `findings[]` entry is the same shape as a `requirements[]` entry of `requirements.yaml`, whose `id` key defines a `REQ-nnn`. The `verify-ids` check depends on that reading: without it every `FIND-nnn` this phase writes would be an unresolved reference and the check would fail on a clean story. A CLI that reads it otherwise makes `verify-ids` fail on the first run, which is a visible failure rather than a silent pass.

31. **`FIXTURE-SHA:<name>` in `expect.args` was proposed as an addition to the shared eval runner and is rejected in favour of inlined digests.** The proposal was that the runner resolve the form at case-expansion time to the lowercase hex SHA-256 of the bytes of `evals/fixtures/<name>`, alongside the `FIXTURE:<name>` form that `setup.files` already carries. `evals/runner/run_jsonl.py` resolves `FIXTURE:<name>` in `setup.files` and nothing else, so an unresolved `FIXTURE-SHA:` string reaches the grader as a literal and `_digests_hold` compares it against a hex digest, which no file matches. The three cases `vq-01`, `vq-03`, and `vq-04` therefore carry the literal digest of each fixture in `expect.args`, which is the form `specs/04-constitute.md` uses. The digest is the SHA-256 of the fixture as the runner materializes it — read as UTF-8 text and written with `\n` line endings — so a fixture edit changes the digest and the case fails until the digest is recomputed, which is the cost this decision accepts. `specs/01-cli.md` Decision 62 stands unchanged: the grader receives the workspace, the transcript, and `args`, and `args` holds literal strings.


### Dependency on a spec written in parallel

32. **`specs/06-build.md`.** This spec's send-back to Build is `/build <STORY-nnn> --remedy FIND-nnn,...`, which is the §4c form with the ids this phase allocates. Build is assumed to re-open the cited `FIND-nnn` ids alone, to read them from `.devforgeai/reports/STORY-nnn-qa.yaml` `findings[]` by `id`, `file`, `line`, `severity`, `relates_to`, and `evidence`, and to return `/verify <STORY-nnn> --resume`. This spec reads `.devforgeai/reports/STORY-nnn-build.yaml` through the conventions §5 contract and the `specs/01-cli.md` report schema alone, listed field by field in `## Inputs`; Build adds no key to that file, because the CLI writes it. If Build re-opens a different id kind, the change lands in this spec's `## Send-back` `Next` column and in the `sendback_block` grader's `next_prefix` argument, and nothing else moves.

### Blockers

33. **None.** Every step of this phase runs inside a Claude Code terminal session with the §2 primitives: the four preamble lines are `Bash` invocations of the binary, the ten subagents are `Agent` invocations reading with Read, Grep, and Glob, the one document is a `Write`, the two optional commands are `Bash` invocations of a string from `config.toml`, and the gate and the handoff are hook-run subcommands. The two things this spec asks of the CLI that the CLI does not yet do are the `no_cycle` check kind of Decision 1, which is a graph walk over files the binary already parses, and the `total: 0` clarification of Decision 6, which is one branch in the `verifier_pass` evaluator. Neither needs a primitive outside the §2 list.
