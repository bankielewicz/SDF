---
name: plan
description: Phase 3 of DevForgeAI, run by /plan. Turns one epic's accepted requirements into one .devforgeai/stories/STORY-nnn.md per unit of work and one stories/sprint.yaml naming the subset that fits the configured point capacity, in dependency order. Each story carries AC-nnn acceptance criteria in Given/When/Then form, the REQ-nnn it implements, the CON-nnn and AP-nnn it is bound by, the UI-nnn it renders, its layer, the file paths Build writes, and its dependencies on other stories. Reach for it whenever /plan is typed, whenever an epic is being decomposed into stories, whenever an acceptance criterion, story point, sprint capacity, dependency order, or declared file set is being written for this framework, and whenever STORY-nnn, SPRINT-nnn, AC-nnn, stories/sprint.yaml, or a Plan send-back to Discover or Constitute appears in a story, a report, or a handoff.
argument-hint: <EPIC-nnn | SPRINT-nnn> [--remedy AC-nnn,AC-nnn] [--resume]
allowed-tools: Bash(devforgeai:*), PowerShell(devforgeai:*), Read, Write, Edit, Glob, Grep, Agent, AskUserQuestion, Skill
disable-model-invocation: true
---

!`devforgeai gate require plan $ARGUMENTS[0]`
!`devforgeai doc load requirements $ARGUMENTS[0]`
!`devforgeai doc load context all`

# planning-work

Plan takes one epic and turns its requirements into buildable units. It writes two document types and nothing else: one `STORY-nnn.md` per unit of work, and one `sprint.yaml` naming the subset that fits capacity. It edits no upstream document — a defect in `requirements.yaml`, in a context file, or in an ADR leaves as a SEND BACK citing ids.

## Entry

`/plan` invokes this skill. Arguments:

| Form | Run kind | Meaning |
|---|---|---|
| `/plan EPIC-nnn` | full | decompose that epic |
| `/plan SPRINT-nnn` | full | the epic is the `epic` key of `.devforgeai/stories/sprint.yaml` |
| `/plan EPIC-nnn --remedy AC-nnn,AC-nnn` | remedy | Build or Verify cited those criteria and sent them back |
| `/plan EPIC-nnn --resume` | resume | re-enter at workflow step 2 after a send-back returned |

An id in `$1` whose prefix is neither `EPIC` nor `SPRINT` stops the run with one `Blocked` line naming those two accepted prefixes.

The three preamble lines at the head of this file run before the body loads:

- `devforgeai gate require plan $ARGUMENTS[0]` — exit 1 names the missing predecessor gate and the body does not load, which is the gate.
- `devforgeai doc load requirements $ARGUMENTS[0]` — the whole `.devforgeai/requirements.yaml`. The subcommand ignores its `<id>` argument, so either prefix reaches the same file.
- `devforgeai doc load context all` — the six context files.

None of the three allocates an id. `SPRINT`, `STORY` and `AC` ids are allocated at steps 4 and 8, once the run kind is known, so a remedy run that allocates nothing cannot abort on an exhausted prefix.

Read from disk as the run needs them: `.devforgeai/config.toml` (`[plan].sprint_capacity_points`, `[plan].story_points`, `[[layer]].name`), `.devforgeai/state.toml` (`current_phase`, `[active].constitute`, `[active].plan`), `.devforgeai/stories/sprint.yaml` when it exists, and on a remedy run `.devforgeai/reports/STORY-nnn-build.yaml` and `.devforgeai/reports/STORY-nnn-qa.yaml` through `devforgeai report show`.

## Workflow

**1. Establish the run — model.** The run kind is `remedy` when `$ARGUMENTS` holds `--remedy`, `resume` when it holds `--resume`, and `full` otherwise. The epic is `$1` for an `EPIC` prefix, and the `epic` key of `.devforgeai/stories/sprint.yaml` for a `SPRINT` prefix. A remedy run goes to `## Remedy and resume` and runs steps R1 to R4 in place of steps 2 to 13.

**2. Read the epic — model.** From the `requirements.yaml` text on the preamble's stdout, take the `epics[]` entry whose `id` is the run's epic, then the `requirements[]` records its `requirements` list names whose `status` is `accepted`. A record at `status: withdrawn`, or at `priority: wont`, is dropped here and named in step 8's `## Out of scope`. An epic id resolving to no `epics[]` entry stops the run with one `Blocked` line naming the id and the epic ids the file does hold.

**3. Read the context set — model.** From the six files on the preamble's stdout, take: the layer name set from `source-tree.md` `## Layers` second table; the `## File placement rules`, `## Naming conventions`, and `## Generated and excluded paths` rows; the `architecture-constraints.md` `## Constraint index` rows at `Status: active` and the `## Layer dependency rules` rows; the `anti-patterns.md` `## Anti-pattern index` rows; the `coding-standards.md` `## Testing standards` rows; the `tech-stack.md` `## Excluded technologies` and `dependencies.md` `## Forbidden dependencies` lists, which keep an excluded or forbidden name out of an acceptance criterion. A `## Layers` table with no data row stops the run with one `Blocked` line naming `source-tree.md ## Layers`.

**4. Open the sprint — model, CLI.** When `.devforgeai/stories/sprint.yaml` exists and its `epic` key equals the run's epic, reuse that file's `id` and allocate nothing. Otherwise run `devforgeai doc validate --allocate SPRINT`, which returns the next free id. Then run `devforgeai phase set plan --id SPRINT-nnn --epic <the run's epic>`. `--epic` carries the `EPIC-nnn` step 1 settled. The call needs it whenever `sprint.yaml` is absent, which is every full run — the file is written at step 12 — and every resume that follows a send-back, where step 12 did not run. With the file present the flag repeats the file's own `epic` key, and a value differing from it is `DFA-E013`. That call sets `state.toml` `current_phase` to `plan` and `[active].plan` to this id, which is where `report ingest` resolves its target report at step 9. Exit 1 on `DFA-E320` means the constitute gate is not PASS; the run stops and the Stop hook prints the gate result.

**5. Decompose — subagent `story-decomposer`, alone.** Pass the epic entry, the requirement records with their `traces_to` lists, the layer set, the active constraint rows, the layer dependency rules, the anti-pattern rows, and the `[plan].story_points` set. It returns one draft per story with `req_ids`, `story_lines`, `acceptance_criteria`, `layer`, `con_ids`, `ap_ids`, `ui_ids`, `needs_screen`, `depends_on`, `points`, and `out_of_scope`, plus `uncovered_reqs` and `notes`. `depends_on` holds run-local `key` values, because no `STORY-nnn` exists yet. Failure path: output that does not parse against the schema in `agents/story-decomposer.md` `## Output` re-invokes the subagent once with the parse error appended; a second failure leaves the gate metric absent, which `gate check` reports as exit 1.

**6. Specify screens — Design skill, spec mode, zero or more invocations.** A draft whose `needs_screen` is `true` — its `layer` is `interface` and every `REQ-nnn` in its `req_ids` carries an empty `traces_to` — has no screen specification yet. Through the Skill tool, invoke the `design` skill with `STORY-nnn --spec` once per such draft, using the `STORY-nnn` step 8 allocated for it, then read each returned id back with `devforgeai doc load ui-spec UI-nnn`. Four sections of that document feed the story: `## Requirements` fixes the `REQ-nnn` set the screen realizes, `## States` fixes the state list section 6 records and the size the story carries, `## Breakpoints` fixes the four layouts one `AC-nnn` covers, and `## Out of scope` fixes what section 10 records. This step and step 8 interleave: the allocation comes from step 8 and the ids come back into step 8's `## Interface` table and frontmatter `consumes`. When Design returns a SEND BACK to Discover, write the stories held so far at `status: draft`, write no `sprint.yaml`, and let the handoff carry Design's `Next` line.

**7. Plan the file sets — subagent `story-file-set-planner`, alone, after step 5.** Pass every draft, the `## File placement rules` rows, the `## Naming conventions` rows, the `## Generated and excluded paths` globs, and the `## Testing standards` rows. It returns one `files[]` table per draft — a `path`, a `kind`, and a `layer` per row — plus `overlaps` and `unplaceable`. A non-empty `overlaps` or `unplaceable` list goes back through one further invocation with those entries appended to the prompt. Failure path: output that does not parse against the schema in `agents/story-file-set-planner.md` `## Output` re-invokes the subagent once with the parse error appended; a second failure leaves the gate metric absent, which `gate check` reports as exit 1.

**8. Write the stories — model, CLI.** Run `devforgeai doc validate --allocate STORY` once per draft and `devforgeai doc validate --allocate AC` once per criterion, then write `.devforgeai/stories/STORY-nnn.md` from `templates/story.md` with the frontmatter `status` value `draft`. The ten sections, their columns, and their row rules are in `references/story-sections.md`; read it before the first write. Exit 1 on `DFA-E215` from `--allocate` stops the run with one `Blocked` line naming the exhausted prefix. The PostToolUse hook runs `devforgeai doc validate` on each written path and its result reaches the model after the write as `hookSpecificOutput.additionalContext`; rewrite the key it names.

**9. Audit — subagent `story-invest-auditor`, alone.** Pass every written story path, the run's `SPRINT-nnn`, the epic entry, the requirement records, and the active constraint rows with the layer dependency rules. It returns the `devforgeai/verifier/1` envelope with `passed`, `total`, `unit: stories`, and `findings[]`. The SubagentStop hook runs `devforgeai report ingest story-invest-auditor -`, which writes `verifiers.story_invest` into `.devforgeai/reports/SPRINT-nnn-plan.yaml` and creates that file when it is absent. Failure path: output that does not parse against the `devforgeai/verifier/1` envelope re-invokes the subagent once with the parse error appended; a second failure leaves the gate metric absent, which `gate check` reports as exit 1.

**10. Rewrite on judgment findings — model, subagent `story-invest-auditor`.** Each `findings[]` entry at `severity: warn` cites a `STORY-nnn` and names an independence, size, or wording problem in it. Edit those stories, then invoke the auditor once more over the edited set. The loop runs once: a `warn` finding surviving the second pass stays in the report, annotates the gate, and changes no gate result. A finding at `severity: block` sets no edit here — it is the send-back of `## Send-back`.

**11. Sequence and fill — subagent `sprint-sequencer`, alone.** Pass every story id with its `points` estimate from step 5 and its `## Dependencies` id list, plus `points_max` from `[plan].sprint_capacity_points` or the default `20`, and which of the two it came from. It returns the `stories[]` list in topological order with `order` and `points`, the `deferred[]` list with a `reason` of `capacity` or `dependency`, the `capacity` object, `longest_chain`, and `cycle`. A non-empty `cycle` stops the run with one `Blocked` line naming the cycle path; `devforgeai story validate` reports the same condition as `DFA-E232`. Failure path: output that does not parse against the schema in `agents/sprint-sequencer.md` `## Output` re-invokes the subagent once with the parse error appended; a second failure leaves the gate metric absent, which `gate check` reports as exit 1. The capacity arithmetic is in `references/sprint-file.md`.

**12. Write the sprint — model.** Write `.devforgeai/stories/sprint.yaml` from `templates/sprint.yaml` with the step 4 `SPRINT-nnn` and the `status` value `active`, then edit each story named in `stories[]` and in `deferred[]` to carry the frontmatter `status` value `ready`. `ready` is Plan's value to write: it is absent from `devforgeai phase set`'s transition table, which owns `building`, `built`, and `released`. The PostToolUse `doc validate` result reaches the model after the write as `additionalContext`, naming a key; rewrite that key.

**13. Close — CLI, Stop hook.** The Stop hook runs `devforgeai gate check --phase plan`, which runs `devforgeai story validate --scope sprint` as its `plan-stories` check and writes `.devforgeai/reports/SPRINT-nnn-plan.yaml`, then `devforgeai handoff`. The Stop hook renders the closing block; this skill writes no part of it. On a gate FAIL the Stop hook holds the turn with the failing checks as its `reason`, and the block appears when the turn ends.

## Subagents

Contracts, tools, models, and invocation order are in `agents.md`. Full schemas are in each `agents/<name>.md` `## Output`.

| Subagent | Invoked at | What to pass | What comes back |
|---|---|---|---|
| `story-decomposer` | step 5, alone | the epic entry, the requirement records, the layer set, the active constraints, the layer dependency rules, the anti-pattern rows, `[plan].story_points` | `devforgeai/story-decomposer/1`: `drafts[]`, `uncovered_reqs`, `notes` |
| `story-file-set-planner` | step 7, alone, after step 5 | the drafts, the placement rules, the naming conventions, the excluded globs, the testing standards, the repo tree under the source and test roots | `devforgeai/story-file-set-planner/1`: `sets[]`, `overlaps`, `unplaceable` |
| `story-invest-auditor` | step 9, alone; again at step 10 over the edited set | the written story paths, the `SPRINT-nnn`, the epic entry, the requirement records, the active constraints and layer rules | `devforgeai/verifier/1`: `passed`, `total`, `unit`, `findings[]`; a registered verifier, ingested by SubagentStop |
| `sprint-sequencer` | step 11, alone | each story's id, points, and dependency list; `points_max` and its source | `devforgeai/sprint-sequencer/1`: `capacity`, `stories[]`, `deferred[]`, `longest_chain`, `cycle` |
| `spec-gap-triager` | step R2, alone, remedy runs only | one triple per cited id, the criterion text, the requirement behind it, the binding constraints, the cited `UI-nnn` ids | `devforgeai/spec-gap-triager/1`: `decisions[]` with a `disposition` from the closed four |

## Documents

| Document | Path | Template |
|---|---|---|
| Story | `.devforgeai/stories/STORY-nnn.md`, one per unit of work | `templates/story.md` |
| Sprint | `.devforgeai/stories/sprint.yaml`, one per project | `templates/sprint.yaml` |
| Config fragment | merged into `.devforgeai/config.toml` by `devforgeai init` | `templates/plan-config.toml` |

Both documents carry the same seven top-level keys, in this order, with no other top-level key: `schema`, `id`, `phase`, `status`, `produced_by`, `consumes`, `open_questions`. The story carries them as YAML frontmatter; the sprint carries them as its first seven keys, followed by `epic`, `capacity`, `stories`, and `deferred`. `phase` is `plan` and `produced_by` is `planning-work` in both. A story's `consumes` holds at least one `REQ-nnn` and each entry matches `^(REQ|CON|AP|UI)-\d{3}$`; the sprint's `consumes` holds exactly one entry, the epic. Ids are zero-padded three digits, allocated by `devforgeai doc validate --allocate <prefix>`. Sections appear in the template order with the template's heading text.

Story `status` runs `draft`, `ready`, `building`, `built`, `released`. Plan writes `draft` at step 8 and `ready` at step 12; `devforgeai phase set` writes the last three. Sprint `status` runs `planned`, `active`, `closed`. Plan writes `active` at step 12, because the gate's `plan-sprint-status` check reads that value; Release writes `closed`.

`references/story-sections.md` carries the ten story sections and `references/sprint-file.md` the sprint keys and the capacity arithmetic.

## Send-back

Plan has two upstream targets, both raised by a `story-invest-auditor` finding at `severity: block`. The blocking finding's `id` prefix routes the trip: `REQ` to Discover, `CON` and `AP` to Constitute, `UI` to Design. Blocking findings of two prefixes route to Constitute, because a missing constraint is repaired before the requirement it would have decided.

| Id | Condition | IDs cited | Target |
|---|---|---|---|
| SB-1 | A requirement's `statement` admits two incompatible readings, each yielding a different `Then` clause | the `REQ-nnn` of each such finding | Discover |
| SB-2 | A requirement's `acceptance_signal` names no outcome a test reads | the `REQ-nnn` of each such finding | Discover |
| SB-3 | The constraint set decides no case the story meets, in a layer the `## Layer dependency rules` table covers | the `CON-nnn` in that layer row's `Constraint` column | Constitute |
| SB-4 | Two active constraints bind one path set in opposed directions | both `CON-nnn` | Constitute |

On either target the stories written so far stay on disk at `status: draft`, `sprint.yaml` is not written, `requirements.yaml` and the six context files keep every byte, and `state.toml` keeps `current_phase` of `plan`.

The Stop hook renders the closing block from the report; this skill writes no part of it. The lines it carries on a send-back to Discover:

```
Next      /discover IDEA-nnn --remedy REQ-nnn,REQ-nnn
Then      /plan EPIC-nnn --resume
```

`IDEA-nnn` is the top-level `id` of `.devforgeai/requirements.yaml`, which the preamble printed — the document's own id, not the epic the run took as `$1`. A send-back to Constitute prints `/constitute IDEA-nnn --remedy CON-nnn` on the `Next` line and the same `Then` line.

A missing or thin `UI-nnn` is not a send-back. It is repaired inside the run by invoking the `design` skill with `UI-nnn --remedy AC-nnn,...`, because Design is cross-cutting and holds no gate. `references/send-back.md` carries the INVEST split — which property the CLI decides and which the auditor decides — and the table of what arrives here from Build, Verify, Design, Discover, and Constitute.

## Remedy and resume

**`--remedy AC-nnn,...`** arrives from Build or from Verify, and runs four steps in place of steps 2 to 13. Read `references/remedy-run.md` before R1.

**R1. Locate the criteria — model, CLI.** For each cited id, run `devforgeai report show STORY-nnn build` and `devforgeai report show STORY-nnn verify` and pair the id with its `findings[]` entry, producing one `(STORY-nnn, AC-nnn, finding summary)` triple. A cited `AC-nnn` appearing in no story file stops the run with one `Blocked` line naming the id.

**R2. Triage — subagent `spec-gap-triager`, alone.** Pass the triples, the criterion text, the `REQ-nnn` whose `Covered by` cell holds each criterion with its `statement` and `acceptance_signal`, the active constraints binding the story's layer or file paths, and the `UI-nnn` ids in its `## Interface`. It returns one `disposition` per triple from `rewrite_ac`, `send_back_discover`, `send_back_constitute`, `send_to_design`.

**R3. Act — model, Design skill.** `rewrite_ac` edits that one criterion line in place in its `STORY-nnn.md` and leaves every other byte of the file unchanged. `send_to_design` invokes the `design` skill through the Skill tool with `UI-nnn --remedy AC-nnn,...` and rewrites section 6 from the returned spec. `send_back_discover` and `send_back_constitute` write no file. An edit that breaks the `Given … When … Then …` grammar reaches the model after the write as `additionalContext` from the PostToolUse `doc validate`, carrying `DFA-E230`; rewrite the line.

**R4. Reset and close — model, CLI.** Write the frontmatter `status` value `ready` into each story edited at R3 and into that story's entry in `sprint.yaml` `stories[]`, changing no other key of `sprint.yaml`, then run `devforgeai phase set plan --id SPRINT-nnn --epic <EPIC-nnn>` with the id and the `epic` key the file on disk carries. A remedy run reaches R4 with `sprint.yaml` on disk, so both values are read from it. The handoff `Next` line reads `/build STORY-nnn --resume` when every disposition was `rewrite_ac` or `send_to_design`, and the send-back form above otherwise.

**`--resume`** re-enters at step 2 and re-reads `requirements.yaml` and the six context files from the preamble's stdout, so a repaired `REQ-nnn` or a new `CON-nnn` is the text the run works from. Step 4 reuses the `SPRINT-nnn` that `state.toml` `[active].plan` holds and allocates none. Every `STORY-nnn.md` already on disk at `status: draft` is kept and re-audited at step 9, and the ids it holds are not re-allocated. `sprint.yaml`, absent after a send-back, is written at step 12.

## References

- `references/story-sections.md` — read before step 8: the ten story sections with their columns, row counts, and the `none` rule; the frontmatter keys; the three properties that make an `AC-nnn` testable.
- `references/sprint-file.md` — read before step 11: the eleven `sprint.yaml` keys, the `capacity`, `stories[]`, and `deferred[]` schemas, and the capacity arithmetic including the story larger than the whole sprint.
- `references/remedy-run.md` — read before R1: the four remedy steps field by field, what each of the four dispositions edits, and what a remedy run leaves untouched.
- `references/send-back.md` — read at step 10 when a finding carries `severity: block`: the INVEST split between `devforgeai story validate` and `story-invest-auditor`, the four send-back conditions, and the table of send-backs this phase receives.
