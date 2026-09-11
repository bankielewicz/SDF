---
name: constitute
description: Phase 2 of DevForgeAI. Turns an accepted requirements.yaml into the project's constitution - the six context files under .devforgeai/context/ (tech-stack, source-tree, dependencies, coding-standards, architecture-constraints, anti-patterns) and the append-only decision log under .devforgeai/adr/ - allocating CON-nnn constraints, AP-nnn anti-patterns and ADR-nnn decisions, and completing the drafts devforgeai init --analyze leaves on a brownfield project. Reach for it whenever /constitute runs, whenever someone asks where a technology choice, layering rule, dependency policy, coding standard, forbidden pattern or architecture decision record should live, and whenever a Plan send-back arrives as /constitute IDEA-nnn --remedy CON-nnn. It owns those artifacts and their IDs, so read it before touching anything under .devforgeai/context/ or .devforgeai/adr/.
argument-hint: <IDEA-nnn> [--remedy CON-nnn]
allowed-tools: Bash(devforgeai:*), PowerShell(devforgeai:*), Read, Write, Edit, Glob, Grep, Agent, AskUserQuestion, Skill
disable-model-invocation: true
---

!`devforgeai gate require constitute $ARGUMENTS[0]`
!`devforgeai doc load requirements $ARGUMENTS[0]`

# Establishing context

Phase 2 writes the rules the rest of the project is measured against. Every
later phase reads these files: Plan scopes stories inside the constraints, Build
enforces them through hooks, Verify scans against the anti-pattern index. The
six files are project singletons — one run writes all six for the whole
requirements document, and a second cycle amends them through new ADRs and new
CON rows rather than rewriting them.

## Entry

`/constitute` invokes this skill. Arguments:

| Form | Run kind | Meaning |
|---|---|---|
| `/constitute IDEA-nnn` | fresh | write the full context set for that requirements document |
| `/constitute IDEA-nnn --remedy CON-nnn` | remedy | Plan cited that one constraint and sent it back |

`<IDEA-nnn>` is the top-level `id` of `.devforgeai/requirements.yaml`. Constitute
runs once per requirements document, which carries every epic, so the command
takes no epic argument.

The two preamble lines at the head of this file run before the body loads:

- `devforgeai gate require constitute $ARGUMENTS[0]` — exit 0 means the Discover gate passed
  for that id. Exit 1 names the missing gate on stderr and the body does not
  load, which is the gate.
- `devforgeai doc load requirements $ARGUMENTS[0]` — `.devforgeai/requirements.yaml` on
  stdout.

Neither allocates an id. `CON`, `AP` and `ADR` ids are allocated at steps 7, 9
and 10, after the run kind has decided which of them the run writes, so a remedy
run that reopens one constraint cannot abort on a prefix it does not spend.

Seven of its fourteen top-level keys are read here: `id` is the phase instance,
`status` is `accepted`, `revision` travels into the report so Reflect can pair a
context set with the requirement revision it came from, `personas[]` supplies
the actor of a `process` constraint, `epics[]` supplies scope, `success_metric`
and `out_of_scope`, and `requirements[]` supplies `statement`,
`acceptance_signal`, `priority`, `source` and `status`. A requirement record
carries `statement`, not `text`. A record with `status: withdrawn` is skipped.

Read from disk at step 3: `.devforgeai/config.toml`,
`.devforgeai/explore/brief.md`, `.devforgeai/explore/decision.yaml`, and on the
brownfield branch the six drafts under `.devforgeai/context/`.

## Workflow

**1. Gate.** The CLI ran `devforgeai gate require constitute <IDEA-nnn>` in the
preamble against `state.toml` and `gates.toml`.

**2. Load.** The CLI ran `devforgeai doc load requirements <IDEA-nnn>` in the
preamble. Exit 1 names the path that did not resolve and the command stops.

**2b. Claim the phase.** Run `devforgeai phase set constitute --id <IDEA-nnn>`
through Bash. It writes `[current].phase`, `[current].id`,
`[active].constitute`, and resets `[stop_hook].block_count`. `DFA-E320` exit 1
repeats the missing-predecessor text from step 1 and the workflow stops.

**3. Read the inputs.** Read `.devforgeai/config.toml` for the detected stack,
`.devforgeai/explore/brief.md` for the unnumbered lines under `## Non-goals` and
the rows of `## Competitor scan` and `## Technology scan`, and
`.devforgeai/explore/decision.yaml` for the decision record.

- `config.toml` absent halts the step with the CLI's stderr; the SessionStart
  hook re-runs `stack detect`.
- `brief.md` absent leaves the brief non-goal list and the technology candidate
  set empty, and step 4 works from `config.toml` and the requirement records
  alone. Explore is optional ahead of Discover.
- `decision.yaml` absent skips ADR-000 at step 10.
- `decision.yaml` carrying `decision: kill` or `decision: park` halts the
  workflow and prints that value: Explore promoted nothing.

Then branch. `.devforgeai/context/tech-stack.md` present with `status: draft`
means `devforgeai init --analyze` already drafted the six files from the existing
code; continue at B4 in `references/brownfield.md`, which returns to step 11.
Otherwise continue at step 4.

Steps 4 to 10 each write one file from the template named in the step, filled in
place: the template's headings and columns in template order, and the seven
envelope keys as its YAML frontmatter, at the top level of the frontmatter block
with no other top-level key and no wrapper.

**4. tech-stack.md.** Write `.devforgeai/context/tech-stack.md` from
`templates/tech-stack.md` with `status: draft`, one `| Key | Value | Source |` row per detected key with
`Source` = `config.toml`. A `## Technology scan` row whose `Maturity` is
`established` supplies a candidate for a key `config.toml` left unset, with
`Source` = `explore/brief.md ## Technology scan`; `emerging` and `experimental`
rows supply ADR `## Context` text alone. A key still unset becomes one
`AskUserQuestion` whose closed option list is the `## Technology scan`
candidates for that key. An unanswered question stays in the frontmatter
`open_questions` list, where `context audit` check CA-2 reports it.
`references/key-namespace.md` carries the closed key set and the `Source`
values.

**5. source-tree.md.** Write `.devforgeai/context/source-tree.md` from
`templates/source-tree.md` with `status: draft`. `source.root`, `test.root` and `build.output.root` come from
`config.toml`; the layers and their path globs come from the `epics[].scope` and
`requirements[].statement` text. Unset keys follow step 4's path.

**6. dependencies.md.** Write `.devforgeai/context/dependencies.md` from
`templates/dependencies.md` with `status: draft`, from the manifests `config.toml` names and the
`## Technology scan` rows. Unset keys follow step 4's path.

**7. architecture-constraints.md.** Write
`.devforgeai/context/architecture-constraints.md` from
`templates/architecture-constraints.md` with `status: draft` and a populated
`## Constraint index`, one `### CON-nnn` block per entry of three
sources: a `requirements[]` record whose `statement` or `acceptance_signal`
names a bound rather than a behavior, with `source` = that `REQ-nnn` and the
number from its `acceptance_signal` in the CON statement; an unnumbered line
under the brief's `## Non-goals`, with `kind: non-goal` and `source` =
`explore/brief.md ## Non-goals`; an `epics[].out_of_scope` entry, with
`kind: non-goal` and `source` = that `EPIC-nnn`. IDs come from
`devforgeai doc validate --allocate CON`, whose exit 1 halts the step with its
stderr. `references/constraints.md` carries the field values and how a bound is
told from a behavior.

**8. coding-standards.md.** Write `.devforgeai/context/coding-standards.md` from
`templates/coding-standards.md` with `status: draft`, filled from steps 2, 4 and 7. `## Design tokens` carries
`tokens.path` = `.devforgeai/brand/tokens.json`, which is where
`devforgeai design lint` reads it.

**9. anti-patterns.md.** Write `.devforgeai/context/anti-patterns.md` from
`templates/anti-patterns.md` with `status: draft` and a populated
`## Anti-pattern index`. Each CON whose
statement a text or path pattern observes becomes one `### AP-nnn` block with
`source` = that CON id, and that CON's `enforced_by` field and index cell take
the AP id. IDs come from `devforgeai doc validate --allocate AP`. A CON no
pattern observes keeps `enforced_by: none`, and `alignment-auditor` reports it
at step 11 as `unobservable`. `references/constraints.md` says which CON kinds
yield a detector.

**10. The ADR log.** Write `.devforgeai/adr/ADR-000.md` from `templates/ADR.md`
filled from `decision.yaml` when that file exists, then one
`.devforgeai/adr/ADR-nnn.md` from the same template per decision that closed a
choice among alternatives, each `status: proposed`, `consumes` listing
the motivating REQ ids in `priority` order, and `## Constraints introduced`
listing the CON ids from step 7. IDs above 000 come from
`devforgeai doc validate --allocate ADR`; ADR-000 is reserved. Every CON id from
step 7 appears in the `## Constraints introduced` table of one of these ADRs: a
non-goal CON drawn from `## Non-goals` or from `epics[].out_of_scope` carries a
path or an `EPIC-nnn` in `source` rather than a REQ, so the ADR that records the
scope boundary it states is what carries it. A CON with no introducing ADR and
no REQ in `source` is reported by `context audit` check CA-4 at step 13.
`references/adr-log.md` carries ADR-000's six sections and the supersession
shape.

**11. Review.** Invoke `architecture-reviewer` and `alignment-auditor` in
parallel with the paths from steps 4 through 10 and the `requirements[]`
records. Each returns one JSON object on stdout; the SubagentStop hook runs
`devforgeai report ingest <subagent> <source>`, filing each block under
`verifiers.<subagent>` in `.devforgeai/reports/<IDEA-nnn>-constitute.yaml`. A
subagent whose output does not parse against its schema is re-invoked once with
the parse error appended to the prompt; a second parse failure leaves the gate
metric absent and `gate check` exits 1.

**12. Acceptance.** Render the two subagents' findings as one `AskUserQuestion`
with the options `accept`, `amend`, `send back`.

- `accept` — edit `status: accepted` into the six frontmatters and into every
  ADR frontmatter.
- `amend` — return to the step that wrote the file the finding names.
- `send back` — go to `## Send-back`.

`verifiers.architecture_reviewer.payload.send_back_requirements` non-empty routes to
`## Send-back` without asking the question.

**13. Close.** The Stop hook runs
`devforgeai gate check --phase constitute --id <IDEA-nnn>` and then
`devforgeai handoff --phase constitute --id <IDEA-nnn>`, which writes the report
and renders the closing block. The Stop hook renders that block; this skill
writes no part of it. On a gate FAIL the Stop hook holds the turn with the
failing checks as its `reason`, and the block appears when the turn ends.

## Subagents

| Subagent | Invoked at | Passed | Returns |
|---|---|---|---|
| `architecture-reviewer` | step 11, in parallel with `alignment-auditor`; R3 and R5 on the remedy path | the six context file paths, the ADR directory path, the `requirements[]` records, the `epics[]` records, and on the remedy path one CON id to scope to | `devforgeai/verifier/1`: `passed`, `total`, `unit: requirements`, `findings[]`, and `payload` holding `requirements_reviewed`, `send_back_requirements`, `blocking_findings` |
| `alignment-auditor` | step 11, in parallel with `architecture-reviewer`; R5 on the remedy path | the six context file paths, the ADR directory path, the `## Constraint index` rows, the `## Anti-pattern index` rows | `devforgeai/verifier/1`: `passed`, `total`, `unit: checks`, `findings[]`, and `payload` holding `checks_run`, `blocking_findings` |
| `source-tree-mapper` | B5, alone, on the brownfield branch | `source.root` from the draft, the layer names the draft proposes, the manifest paths from the draft `dependencies.md` | `devforgeai/source-tree-mapper/1`: `roots`, `layers`, `entry_points`, `internal_edges`, `external_dependencies`, `generated_paths`, `unmapped_paths` |

`architecture-reviewer` and `alignment-auditor` are this phase's registered
verifiers: each also prints the `devforgeai/verifier/1` envelope, which
SubagentStop hands to `report ingest`, and the gate reads
`verifiers.architecture_reviewer.payload.blocking_findings`,
`verifiers.alignment_auditor.payload.blocking_findings`, and
`verifiers.architecture_reviewer.payload.send_back_requirements.length` from the
result. The `payload.` segment is part of each path, because every field an agent
adds of its own sits under `payload` of the one envelope.
`source-tree-mapper` is not registered; its output travels in context alone.
`agents.md` carries the full contracts, the tool lists, and the invocation
order.

## Documents

| Document | Path | Template |
|---|---|---|
| Tech stack | `.devforgeai/context/tech-stack.md` | `templates/tech-stack.md` |
| Source tree | `.devforgeai/context/source-tree.md` | `templates/source-tree.md` |
| Dependencies | `.devforgeai/context/dependencies.md` | `templates/dependencies.md` |
| Coding standards | `.devforgeai/context/coding-standards.md` | `templates/coding-standards.md` |
| Architecture constraints | `.devforgeai/context/architecture-constraints.md` | `templates/architecture-constraints.md` |
| Anti-patterns | `.devforgeai/context/anti-patterns.md` | `templates/anti-patterns.md` |
| Decision record | `.devforgeai/adr/ADR-nnn.md` | `templates/ADR.md` |

Every one carries the same seven top-level frontmatter keys, in this order and
with no other top-level key: `schema`, `id`, `phase`, `status`, `produced_by`,
`consumes`, `open_questions`. `phase` is `constitute` and `produced_by` is
`establishing-context` throughout. A context file's `schema` is
`devforgeai/context-<stem>/1` and its `id` is the stem — the six are singletons
and take no allocated number. An ADR's `schema` is `devforgeai/adr/1` and its
`id` is its `ADR-nnn`.

`status` for a context file is `draft` or `accepted`; for an ADR it is
`proposed`, `accepted`, `rejected`, or `superseded`. An ADR's `consumes` carries
the REQ ids that motivated the decision and is the only place they appear; a
brownfield ADR recording a decision no requirement motivates carries
`consumes: []` and names its evidence path in `## Context`. Open questions live
in the frontmatter `open_questions` list and in no body section. Sections appear
in the template order with the template's heading text: `context audit`, Build's
hooks, Verify's `anti-pattern-scanner` and `design lint` locate content by those
strings.

The PreToolUse hook runs `doc validate --producer-check` on every write under
`.devforgeai/`, and the PostToolUse hook runs `doc validate` and returns its
result to the model after the write as `hookSpecificOutput.additionalContext`,
naming the key or section to rewrite.

## Send-back

Two upstream targets, each exit 2 from `gate check`.

**To Discover.** Two conditions, both detected by `architecture-reviewer` at
step 11:

| Condition | Finding | IDs cited |
|---|---|---|
| SB-1 · a requirement has no feasible architecture under the accepted stack | `kind: infeasible` | the `REQ-nnn` in that finding's `requirements` |
| SB-2 · two requirements force opposed constraints | `kind: contradiction` | both `REQ-nnn` in that finding's `requirements`, plus the `CON-nnn` in `constraints` |

The six context files and the ADRs written before the finding stay on disk at
`status: draft` and `status: proposed`. `requirements.yaml` is left as it
arrived; a defect upstream is cited, not edited. The handoff `Next` line the
Stop hook prints:

```
Next      /discover IDEA-004 --remedy REQ-014,REQ-019,REQ-031
```

The remedy list carries every cited REQ. Discover re-opens by REQ id alone, so
no `EPIC-nnn` appears on this line.

**To Plan.** One condition, on the remedy path:

| Condition | Detected by | IDs cited |
|---|---|---|
| SB-3 · a constraint Plan sent back stands | `architecture-reviewer` returning an empty finding list at R3 | the `CON-nnn` from `--remedy`, and the `STORY-nnn` from the Plan report |

No file changes on this path. The handoff `Next` line:

```
Next      /plan EPIC-001 --resume
```

The Stop hook's `devforgeai handoff` call composes both blocks from `state.toml`
and the latest report; this skill writes no part of them.

## Remedy and resume

`/constitute` takes one flag, `--remedy CON-nnn`, and no `--resume`: the
receiving side of a send-back from Plan, re-opening exactly one constraint. A
list holding more than one CON id is `DFA-E011`, exit 3.

**R1.** The two preamble lines run `devforgeai gate require constitute <IDEA-nnn>`
and `devforgeai doc load requirements <IDEA-nnn>` as at steps 1 and 2. Then run
`devforgeai phase set constitute --id <IDEA-nnn> --remedy CON-nnn` through Bash,
which writes `[constitute].remedy_con`.

**R2.** Read the `### CON-nnn` block, its `## Constraint index` row, the ADR its
`introduced_by` field names, and the Plan report
`.devforgeai/reports/<STORY-nnn>-plan.yaml` the send-back cited, whose
`send_back` mapping carries `to`, `constraint` and `reason`. A `CON-nnn` absent
from the index halts with that id on stderr.

**R3.** Invoke `architecture-reviewer` scoped to that one CON and the REQ ids in
its `source` field. A finding of kind `overbuilt` or `contradiction` naming the
CON supports replacement, at R4a. An empty finding list upholds the constraint,
at R4b.

**R4a, replacement.** Write a new ADR with `status: proposed`, its
`## Supersedes` row naming the introducing ADR and listing `CON-nnn` as retired,
and its `## Constraints introduced` listing the replacement CON from
`devforgeai doc validate --allocate CON`. Edit the superseded ADR frontmatter to
`status: superseded` and the `CON-nnn` index row to `status: retired`. Every
other CON row is left as it stands. An AP whose `source` is `CON-nnn` gets its
`source` repointed to the replacement CON, or its block and index row removed
when the replacement CON carries no pattern. No ADR body is edited in place.
`references/adr-log.md` carries the four supersession steps and what CA-8 reads
from them.

**R4b, upheld.** No ADR is written and no file changes. Continue at R6 with the
upheld CON id.

**R5.** Invoke `architecture-reviewer` and `alignment-auditor` over the whole
set, as at step 11. A changed CON reaches every file, so both run again.

**R6.** The Stop hook, as at step 13. On the upheld path the gate result is
`SEND BACK to Plan` and the handoff `Found` line carries `CON-nnn` with the
reviewer's one-sentence statement.

A second cycle over a project that already has the six files is a fresh run, not
a resume: the files are singletons, and the amendments arrive as new ADRs and
new CON rows. A run that finds the six files at `status: draft` continues into
the brownfield branch, which is where an interrupted first pass resumes.

## References

- `references/brownfield.md` — step 3 onward when the drafts exist: steps B4
  through B8, the draft-heading to template-heading map, and what
  `source-tree-mapper` supplies.
- `references/key-namespace.md` — steps 4, 5, 6, 8: the closed key set, how a
  concrete key matches a wildcard row, and what goes in the `Source` column.
- `references/constraints.md` — steps 7 and 9: where each CON comes from, the
  field values, and which CON kinds yield an anti-pattern detector.
- `references/adr-log.md` — step 10, B8, R4a: ADR-000 from `decision.yaml`, one
  ADR per closed choice, and the four steps of a supersession.
