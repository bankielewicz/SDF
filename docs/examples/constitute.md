# `/constitute` — a worked walkthrough

Phase 2. The accepted `requirements.yaml` becomes the project's constitution: six context files under `.devforgeai/context/` and an append-only decision log under `.devforgeai/adr/`.

Every later phase reads these files. Plan scopes stories inside the constraints, Build enforces them through hooks, Verify scans against the anti-pattern index. The six files are project singletons — one run writes all six for the whole requirements document, and a second cycle amends them through new ADRs and new CON rows rather than rewriting them.

---

## Entry

| Form | Run kind |
|---|---|
| `/constitute IDEA-nnn` | fresh |
| `/constitute IDEA-nnn --remedy CON-nnn` | remedy — Plan cited that one constraint |

`<IDEA-nnn>` is the top-level `id` of `.devforgeai/requirements.yaml`. Constitute runs once per requirements document, which carries every epic, so the command takes no epic argument.

There is no `--resume`. A list holding more than one CON id is `DFA-E011`, exit 3.

Two preamble lines:

```
!`devforgeai gate require constitute $ARGUMENTS[0]`
!`devforgeai doc load requirements $ARGUMENTS[0]`
```

The first is the gate. A non-zero exit aborts the whole invocation, so the body does not load:

```
$ devforgeai gate require constitute IDEA-001
devforgeai: DFA-E321 gate require: constitute needs discover gate PASS for IDEA-001; no report at .devforgeai/reports/IDEA-001-discover.yaml; to repair, run /discover IDEA-001, then /constitute IDEA-001
```

Reproduced, exit 1. The same refusal reaches you a step earlier from `UserPromptExpansion`, before the skill is loaded at all:

```
$ echo '{"command_name":"constitute","command_args":"IDEA-001"}' | devforgeai hook run prompt-expansion
{"decision":"block","reason":"DFA-E321 constitute needs discover gate PASS for IDEA-001; no report at .devforgeai/reports/IDEA-001-discover.yaml; to repair, run /discover IDEA-001, then /constitute IDEA-001\nThe predecessor gate for /constitute IDEA-001 has not passed, so the skill was not loaded."}
```

Reproduced, exit 2.

Neither preamble line allocates an id. `CON`, `AP` and `ADR` ids are allocated at steps 7, 9 and 10, after the run kind has decided which of them the run writes, so a remedy run that reopens one constraint cannot abort on a prefix it does not spend.

---

## The exchange

```
> /constitute IDEA-001
```

**Steps 1 and 2** already ran in the preamble.

**Step 2b — claim the phase.**

```
devforgeai phase set constitute --id IDEA-001
```

Writes `[current].phase`, `[current].id`, `[active].constitute`, and resets `[stop_hook].block_count`. A `DFA-E320` exit 1 repeats the missing-predecessor text and the workflow stops.

**Step 3 — read the inputs, then branch.**

| Input | Absent means |
|---|---|
| `.devforgeai/config.toml` | halt with the CLI's stderr; the SessionStart hook re-runs `stack detect` |
| `.devforgeai/explore/brief.md` | the brief non-goal list and the technology candidate set are empty; step 4 works from `config.toml` and the requirement records alone. Explore is optional ahead of Discover |
| `.devforgeai/explore/decision.yaml` | ADR-000 is skipped at step 10 |

`decision.yaml` carrying `decision: kill` or `decision: park` halts the workflow and prints that value: Explore promoted nothing.

Then the branch. `.devforgeai/context/tech-stack.md` present with `status: draft` means `devforgeai init --analyze` already drafted the six files from the existing code; the run continues at B4 in `references/brownfield.md` and returns to step 11. Otherwise it continues at step 4.

Seven of the requirements document's fourteen top-level keys are read here: `id` is the phase instance, `status` is `accepted`, `revision` travels into the report so Reflect can pair a context set with the requirement revision it came from, `personas[]` supplies the actor of a `process` constraint, `epics[]` supplies scope, `success_metric` and `out_of_scope`, and `requirements[]` supplies `statement`, `acceptance_signal`, `priority`, `source` and `status`. A requirement record carries `statement`, not `text`. A record with `status: withdrawn` is skipped.

### Steps 4 to 10 — one file per step

Each writes one file from its named template, filled in place: the template's headings and columns in template order, and the seven envelope keys as its YAML frontmatter with no other top-level key and no wrapper.

**4. `context/tech-stack.md`** — one `| Key | Value | Source |` row per detected key with `Source` = `config.toml`. A `## Technology scan` row whose `Maturity` is `established` supplies a candidate for a key `config.toml` left unset, with `Source` = `explore/brief.md ## Technology scan`; `emerging` and `experimental` rows supply ADR `## Context` text alone. A key still unset becomes one `AskUserQuestion` whose closed option list is the `## Technology scan` candidates for that key. An unanswered question stays in the frontmatter `open_questions` list, where `context audit` check CA-2 reports it.

**5. `context/source-tree.md`** — `source.root`, `test.root` and `build.output.root` from `config.toml`; the layers and their path globs from the `epics[].scope` and `requirements[].statement` text.

**6. `context/dependencies.md`** — from the manifests `config.toml` names and the `## Technology scan` rows.

**7. `context/architecture-constraints.md`** — one `### CON-nnn` block per entry of three sources:

| Source | `kind` | `source` value |
|---|---|---|
| a `requirements[]` record whose `statement` or `acceptance_signal` names a bound rather than a behavior | — | that `REQ-nnn`, with the number from its `acceptance_signal` in the CON statement |
| an unnumbered line under the brief's `## Non-goals` | `non-goal` | `explore/brief.md ## Non-goals` |
| an `epics[].out_of_scope` entry | `non-goal` | that `EPIC-nnn` |

Ids come from `devforgeai doc validate --allocate CON`, whose exit 1 halts the step with its stderr.

**8. `context/coding-standards.md`** — filled from steps 2, 4 and 7. `## Design tokens` carries `tokens.path` = `.devforgeai/brand/tokens.json`, which is where `devforgeai design lint` reads it.

**9. `context/anti-patterns.md`** — each CON whose statement a text or path pattern observes becomes one `### AP-nnn` block with `source` = that CON id, and that CON's `enforced_by` field and index cell take the AP id. Ids from `devforgeai doc validate --allocate AP`. A CON no pattern observes keeps `enforced_by: none`, and `alignment-auditor` reports it at step 11 as `unobservable`.

**10. The ADR log** — `adr/ADR-000.md` from `decision.yaml` when that file exists, then one `adr/ADR-nnn.md` per decision that closed a choice among alternatives, each at `status: proposed`, `consumes` listing the motivating REQ ids in `priority` order, and `## Constraints introduced` listing the CON ids from step 7. Ids above 000 from `devforgeai doc validate --allocate ADR`; **ADR-000 is reserved**.

Every CON id from step 7 appears in the `## Constraints introduced` table of one of these ADRs. A non-goal CON drawn from `## Non-goals` or from `epics[].out_of_scope` carries a path or an `EPIC-nnn` in `source` rather than a REQ, so the ADR that records the scope boundary it states is what carries it. A CON with no introducing ADR and no REQ in `source` is reported by `context audit` check CA-4 at step 13.

A drafted `tech-stack.md` on a brownfield project, reproduced from `init --analyze`:

```markdown
---
schema: devforgeai/context-tech-stack/1
id: tech-stack
phase: constitute
status: draft
produced_by: establishing-context
consumes: []
open_questions: []
---

# tech-stack

## Languages

| Stack | Markers |
|---|---|
| node | package.json |

## Package managers

| Stack | Manager | Lockfile |
|---|---|---|
| node | npm | absent |

## Test tooling

| Stack | Test command | Coverage command | Format |
|---|---|---|---|
| node | npm run test | none detected | lcov |
```

A context file's `schema` is `devforgeai/context-<stem>/1` and its `id` is the stem — the six are singletons and take no allocated number. An ADR's `schema` is `devforgeai/adr/1` and its `id` is its `ADR-nnn`.

**Step 11 — review.** `architecture-reviewer` and `alignment-auditor` in parallel with the paths from steps 4 through 10 and the `requirements[]` records. Both are registered verifiers; `SubagentStop` runs `devforgeai report ingest <subagent> <source>` and files each block under `verifiers.<subagent>` in `.devforgeai/reports/IDEA-001-constitute.yaml`. A subagent whose output does not parse is re-invoked once with the parse error appended; a second parse failure leaves the gate metric absent and `gate check` exits 1.

`source-tree-mapper` runs on the brownfield branch at B5 and is not registered; its output travels in context alone.

**Step 12 — acceptance.** One `AskUserQuestion` rendering the two subagents' findings, with three options:

| Option | What it does |
|---|---|
| `accept` | edit `status: accepted` into the six frontmatters and into every ADR frontmatter |
| `amend` | return to the step that wrote the file the finding names |
| `send back` | go to the send-back below |

A non-empty `verifiers.architecture_reviewer.payload.send_back_requirements` routes to the send-back without asking the question.

---

## The gate

Until the six files are accepted with no open questions, `context audit` refuses:

```
$ devforgeai context audit
context audit  6/6 files · 0 constraints · 0 anti-patterns · CA-2, CA-3 failed
devforgeai: DFA-E223 context audit: .devforgeai/context/tech-stack.md is status 'draft' with 0 open questions; context audit needs accepted and []
  at .devforgeai/context/tech-stack.md:5
devforgeai: DFA-E223 context audit: .devforgeai/context/coding-standards.md is status 'draft' with 1 open questions; context audit needs accepted and []
  at .devforgeai/context/coding-standards.md:5
```

Reproduced on a freshly `--analyze`d project, exit 1.

The constitute gate's required check kind is `context_audit`. The eight CA checks it runs:

| Check | Code | What it catches |
|---|---|---|
| file present | `DFA-E220` | one of the six context files is absent |
| CA-2 | `DFA-E223` | a context file is not `accepted` with an empty `open_questions` |
| contradiction | `DFA-E221` | a context statement contradicts an ADR, naming both lines |
| CA-3 | `DFA-E222` | a `CON-nnn` is defined in two files |
| CA-4 | `DFA-E224` | an active CON is introduced by no ADR and its `source` resolves to no requirement |
| CA-6 | `DFA-E225` | an anti-pattern row has no detector, a bad `detector_kind`, or a `source` that is not an active CON |
| CA-7 | `DFA-E226` | an ADR's id duplicates another, it consumes a `REQ-nnn` `requirements.yaml` does not define, or it introduces a CON the constraint index omits |
| CA-8 | `DFA-E227` | a supersession is incomplete: a superseded ADR or a retired CON at the wrong status |

`context audit` also runs from the `pre-commit` git hook on every commit touching `.devforgeai/`.

---

## The handoff

```
Phase     2 · Constitute      IDEA-001 · tech-stack
Done      6/6 context · 4 ADR
Gate      PASS  3 checks
Verified  alignment-auditor · 12/12 checks +1 more

Next      /plan EPIC-001
Then      /build STORY-001
Blocked   none

Full report: .devforgeai/reports/IDEA-001-constitute.yaml
```

The slug for this phase is the first H1 of the **first context file that exists**, because Constitute writes six documents rather than one. The `Done` line counts context files holding more than whitespace, and `ADR-*.md` files in `adr/`. The `Verified` line shows the lower of the two verifier ratios, with `+1 more` for the other.

---

## Send-back

Two upstream targets, each exit 2 from `gate check`.

### To Discover

Two conditions, both detected by `architecture-reviewer` at step 11:

| Condition | Finding | Ids cited |
|---|---|---|
| SB-1 · a requirement has no feasible architecture under the accepted stack | `kind: infeasible` | the `REQ-nnn` in that finding's `requirements` |
| SB-2 · two requirements force opposed constraints | `kind: contradiction` | both `REQ-nnn` in that finding's `requirements`, plus the `CON-nnn` in `constraints` |

The six context files and the ADRs written before the finding stay on disk at `status: draft` and `status: proposed`. `requirements.yaml` is left as it arrived; a defect upstream is cited, not edited.

```
Next      /discover IDEA-001 --remedy REQ-014,REQ-019,REQ-031
```

The remedy list carries every cited REQ. Discover re-opens by REQ id alone, so no `EPIC-nnn` appears on this line.

### To Plan

One condition, on the remedy path:

| Condition | Detected by | Ids cited |
|---|---|---|
| SB-3 · a constraint Plan sent back stands | `architecture-reviewer` returning an empty finding list at R3 | the `CON-nnn` from `--remedy`, and the `STORY-nnn` from the Plan report |

No file changes on this path.

```
Next      /plan EPIC-001 --resume
```

---

## The remedy path, step by step

`/constitute IDEA-001 --remedy CON-004` arrives from Plan and re-opens exactly one constraint.

**R1.** The two preamble lines run as at steps 1 and 2. Then:

```
devforgeai phase set constitute --id IDEA-001 --remedy CON-004
```

which writes `[constitute].remedy_con`.

**R2.** Read the `### CON-004` block, its `## Constraint index` row, the ADR its `introduced_by` field names, and the Plan report `.devforgeai/reports/<STORY-nnn>-plan.yaml` the send-back cited, whose `send_back` mapping carries `to`, `constraint` and `reason`. A `CON-nnn` absent from the index halts with that id on stderr.

**R3.** `architecture-reviewer` scoped to that one CON and the REQ ids in its `source` field. A finding of kind `overbuilt` or `contradiction` naming the CON supports replacement, at R4a. An empty finding list upholds the constraint, at R4b.

**R4a, replacement.** Write a new ADR at `status: proposed`, its `## Supersedes` row naming the introducing ADR and listing `CON-004` as retired, and its `## Constraints introduced` listing the replacement CON from `devforgeai doc validate --allocate CON`. Edit the superseded ADR frontmatter to `status: superseded` and the `CON-004` index row to `status: retired`. Every other CON row is left as it stands. An AP whose `source` is `CON-004` gets its `source` repointed to the replacement CON, or its block and index row removed when the replacement CON carries no pattern. **No ADR body is edited in place** — the log is append-only.

**R4b, upheld.** No ADR is written and no file changes. Continue at R6 with the upheld CON id.

**R5.** `architecture-reviewer` and `alignment-auditor` over the whole set, as at step 11. A changed CON reaches every file, so both run again.

**R6.** The Stop hook, as at step 13. On the upheld path the gate result is `SEND BACK to Plan` and the handoff `Found` line carries `CON-004` with the reviewer's one-sentence statement.

A second cycle over a project that already has the six files is a **fresh** run, not a resume: the files are singletons, and the amendments arrive as new ADRs and new CON rows. A run that finds the six files at `status: draft` continues into the brownfield branch, which is where an interrupted first pass resumes.

---

## Files this phase owns

| Path | Template |
|---|---|
| `.devforgeai/context/tech-stack.md` | `templates/tech-stack.md` |
| `.devforgeai/context/source-tree.md` | `templates/source-tree.md` |
| `.devforgeai/context/dependencies.md` | `templates/dependencies.md` |
| `.devforgeai/context/coding-standards.md` | `templates/coding-standards.md` |
| `.devforgeai/context/architecture-constraints.md` | `templates/architecture-constraints.md` |
| `.devforgeai/context/anti-patterns.md` | `templates/anti-patterns.md` |
| `.devforgeai/adr/ADR-nnn.md` | `templates/ADR.md` |

`phase` is `constitute` and `produced_by` is `establishing-context` throughout. A context file's `status` is `draft` or `accepted`; an ADR's is `proposed`, `accepted`, `rejected`, or `superseded`.

An ADR's `consumes` carries the REQ ids that motivated the decision and is the only place they appear. A brownfield ADR recording a decision no requirement motivates carries `consumes: []` and names its evidence path in `## Context`.

Sections appear in template order with the template's heading text, because `context audit`, Build's hooks, Verify's `anti-pattern-scanner` and `design lint` locate content by those strings.
