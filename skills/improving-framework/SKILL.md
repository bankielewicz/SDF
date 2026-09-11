---
name: reflect
description: Cross-cutting Reflect phase of DevForgeAI, run by /reflect. Reads the window aggregate the CLI computes over .devforgeai/reports/, the project's Claude Code session history, and the deferral records inside Verify's QA reports, then writes one .devforgeai/reports/reflect-DATE.yaml holding OBS-nnn observations, REC-nnn recommendations that each name one framework file and one change, and a technical-debt section grouped by CON-nnn or AP-nnn with ages in days. Reach for it whenever /reflect is typed, whenever a retrospective is being run over a window of stories or a release, whenever friction points, repeated send-backs, gate failures by check name, time spent per phase, unparsed subagent output, or accumulated deferred Definition-of-Done items are being looked at, and whenever OBS-nnn, REC-nnn, report aggregate, or a reflect report appears in a document, a report, or a handoff.
argument-hint: IDEA-nnn | EPIC-nnn | STORY-nnn | vX.Y.Z | --since YYYY-MM-DD
allowed-tools: Bash(devforgeai:*), PowerShell(devforgeai:*), Read, Write, Glob, Grep, Agent
disable-model-invocation: true
---

!`devforgeai report aggregate $ARGUMENTS --json`

# improving-framework

Reflect reads what the framework already wrote about itself and turns it into two numbered lists. It writes one document: observations of what happened in a window, recommendations naming one framework file and one change apiece, and a technical-debt section listing the deferred Definition-of-Done items with their ages. It applies no change and edits no upstream document — a `REC-nnn` is prose until the user runs a command against the file it names.

Every count and every timestamp in that document comes from `devforgeai report aggregate --json`. Counting reports, differencing timestamps, and tallying check ids is arithmetic the binary does, which is why this skill reads the aggregate and re-derives nothing from the report files.

## Entry

`/reflect` invokes this skill. Arguments, in two forms:

| Form | Mode | Window |
|---|---|---|
| `/reflect IDEA-nnn` | id | every report whose `id` is that idea |
| `/reflect EPIC-nnn` | id | every report for a `STORY-nnn` whose story frontmatter `consumes` holds that epic |
| `/reflect STORY-nnn` | id | every report whose `id` is that story |
| `/reflect vX.Y.Z` | id | every report for a `STORY-nnn` listed in `.devforgeai/releases/vX.Y.Z.yaml` |
| `/reflect --since YYYY-MM-DD` | since | every report whose `finished_at` is at or after that date at 00:00:00Z |

`$ARGUMENTS` holding neither a recognised id prefix nor `--since` stops the run in the preamble: `devforgeai report aggregate` exits 3 on `DFA-E430` and its stderr line names the four accepted prefixes and the `--since` form.

The preamble line at the head of this file runs before the body loads:

- `devforgeai report aggregate $ARGUMENTS --json` — one `devforgeai/cli-json/1` envelope whose `data` object is the `devforgeai/aggregate/1` window: `window`, `reports[]`, `phase_time[]`, `gate_failures[]`, `send_backs[]`, `verifier_failures[]`, `deferrals[]`, `sessions`, `state`, `floors`, `counts`. Every key is present on every run; an empty result is an empty array or a zero.

There is no `gate require` line, because the reflect gate's `requires` is `""` and a call with no predecessor to test tells the run nothing. There is no `doc load` line, because every document this run reads reaches it through the aggregate. The line allocates no id: `OBS` and `REC` ids are allocated at steps 6 and 8, once the window has produced something to number, so a window that matched no report spends no id.

Read from disk as the run needs them: `.devforgeai/context/architecture-constraints.md` and `.devforgeai/context/anti-patterns.md` for the index rows `debt-aggregator` titles its groups from, and the installed framework files a recommendation can target: `.claude/skills/<name>/SKILL.md`, `.claude/skills/<name>/templates/<file>`, `.claude/agents/<name>.md`, `.devforgeai/gates.toml`, `.devforgeai/config.toml`, and the `hooks` block of `.claude/settings.json`. Those six paths are what an installed project holds; the repository the framework is built in is not on disk here.

## Workflow

**1. Establish the window — model.** From `$ARGUMENTS` and the aggregate on the preamble's stdout: the mode is `since` when `$ARGUMENTS` starts with `--since`, and `id` otherwise. Fill `window.mode`, `window.subject` (the `$1` id in id mode, `""` in since mode), `window.from` (the earliest report `finished_at` in the window), and `window.to` (the run date). `window.generated_at` and `window.cli_version` come from the aggregate's `window` block. The run date in UTC, `YYYY-MM-DD`, is the document's `id` and the `<date>` in its filename.

**2. Read the aggregate — model.** Parse the preamble's stdout as the JSON envelope and take the `data` object's eleven keys. Fill `sources.reports[]` from `reports[]` in aggregate order, one entry per report with `path`, `id`, `phase`, `status`, `started_at`, `finished_at`; fill `sources.sessions` from `sessions`; fill `sources.state` from `state`. Fill `consumes` with every subject id `reports[]` named, ascending and de-duplicated.

`data.counts.reports` of `0` means the window matched no report. Write the document with `observations: []`, `recommendations: []`, `technical_debt.total: 0`, and one `open_questions` line reading `The window <from> to <to> matched no report`, then continue at step 9.

**3. Mine the reports — subagent `observation-miner`.** Pass `phase_time[]`, `gate_failures[]`, `verifier_failures[]`, the `checks[]` and `findings[]` of each `reports[]` entry, and `window.from` and `window.to`. It returns `observations[]` of kinds `gate_failure`, `phase_time`, and `verifier_unparsed`, each with a `ref` and its `sources[]`, plus `notes`. Invoke it in the same message as steps 4 and 5.

**4. Read the session history — subagent `session-pattern-reader`.** Pass `sessions.files[]`, `sessions.commands[]`, `sessions.repeats[]` with the `path` and `line` of each repeat so the agent reads the surrounding lines, `send_backs[]`, `state.current_phase`, and the window dates. It returns `observations[]` of kinds `repeated_send_back` and `friction`, each with a `ref`, a `count`, and `evidence[]` carrying a file path, a line number, and a timestamp. Invoke it in the same message as steps 3 and 5.

A `sessions.status` of `absent`, `empty`, or `unreadable` skips this step: the agent is not invoked, the step returns `observations: []`, and `sources.sessions.reason` carries the resolved path. `references/session-history.md` carries the project-key derivation, the JSONL line shape, and what each of the four status values means.

**5. Collect the deferrals — subagent `debt-aggregator`.** Pass `deferrals[]` with the `CON-nnn` or `AP-nnn` each record cites, `window.to`, and the `## Constraint index` and `## Anti-pattern index` rows of the two context files. It returns `as_of`, `total`, `oldest_days`, and `groups[]`, each group carrying `constraint`, `kind`, `count`, `oldest_days`, and `items[]` with `story`, `dod_item`, `deferred_at`, `age_days`, `reason`, and `report`. Invoke it in the same message as steps 3 and 4. A record citing neither a `CON-nnn` nor an `AP-nnn` lands in the group whose `constraint` is `none` and whose `kind` is `none`.

**6. Allocate the OBS ids — model, CLI.** Merge the `observations[]` of steps 3 and 4, order them by `severity` descending then by `kind` in the `references/observations.md` table order, and run `devforgeai doc validate --allocate OBS` once per observation, in that order. Keep a map from each agent's `ref` to its allocated id, which step 7 reads. An observation whose `sources[]` came back empty is dropped here and one line naming its `ref` goes into `open_questions`. Exit 1 on `DFA-E215` means the prefix is exhausted; the run stops with the stderr line in hand.

**7. Draft the recommendations — subagent `recommendation-drafter`.** Pass the id-bearing observations, the aggregate's `floors` block, the six-row target table `templates/rec-targets.md`, and the paths `Glob` returns under `.claude/skills/`, `.claude/agents/`, and `.claude/settings.json`, plus `.devforgeai/gates.toml` and `.devforgeai/config.toml`. It returns `recommendations[]`, each with a `ref`, an `observations[]` list of `OBS-nnn`, a `target` object, a `change` string, `current_value`, `proposed_value`, `effort`, and `applies_to[]`, plus `dropped[]` and `notes`. Invoke it alone, after steps 3 to 6 return.

The six kinds are ordered by specificity, and the first row a target matches is the one it takes. A recommendation whose `target.path` is `.devforgeai/gates.toml` or `.devforgeai/config.toml` takes `kind: gate_threshold` with the dotted key in `target.key`, whatever the change is; `framework_file` names neither of those two files. `framework_file` is the last row for that reason: it names a path the five rows above it leave, not any path under `.claude/` or `.devforgeai/` at large.

A drafted recommendation whose `target.kind` is `gate_threshold` and whose `proposed_value` sits below the matching entry of `floors` is dropped by the agent, and the drop arrives in `dropped[]` naming the key, the proposed value, and the floor. The binary rejects such a value, so a proposal below a floor carries a number nothing can apply.

**8. Allocate the REC ids — model, CLI.** Run `devforgeai doc validate --allocate REC` once per recommendation, in step 7's order. Exit 1 on `DFA-E215` stops the run, as at step 6.

**9. Write the report — model.** Write `.devforgeai/reports/reflect-<date>.yaml` from `templates/reflect-report.yaml`, with the twelve top-level keys in `## Documents` order and `status: final`. Each `dropped[]` entry of step 7 becomes one `open_questions` line in the form `REC dropped: <key> proposed <value>, floor <value>`. The PostToolUse hook runs `devforgeai doc validate` on the written path and its result reaches the model after the write as `hookSpecificOutput.additionalContext`; rewrite the key it names and write again.

**10. Gate — CLI.** Run `devforgeai gate check --phase reflect --id <date>`. Exit 0 writes `.devforgeai/reports/<date>-reflect.yaml`, the CLI's gate report for the run, which is a second file from the document step 9 wrote. Exit 1 names the failing check ids on stderr; rewrite the entries the check named and run step 10 once more.

**11. Record the cross-cutting run — CLI.** Run `devforgeai phase set reflect --id <date>` as the last step, which records `[last_cross]` in `state.toml` with `phase`, `id`, and the turn and leaves `[current]` where it found it. That is the whole of this step: this skill runs no `devforgeai handoff` of its own. A non-zero exit stops the run with its stderr in context.

A `/reflect` run leaves `current_phase` where it found it: the step 11 call writes `[last_cross]` alone, holds no key in `state.toml` `[active]`, and this skill is not a phase in the §5 sequence. The Stop hook reads `[last_cross]`, renders Reflect's block and the block for the phase the user was in, joins the two, and clears `[last_cross]`. The Stop hook renders both blocks; this skill writes no part of either.

## Subagents

Contracts, tools, models, and invocation order are in `agents.md`. Full schemas are in each `agents/<name>.md` `## Output`. None of the four is a registered verifier, so SubagentStop ingests none of their output and the handoff omits its `Verified` line.

| Subagent | Invoked at | What to pass | What comes back |
|---|---|---|---|
| `observation-miner` | step 3, with steps 4 and 5 | `phase_time[]`, `gate_failures[]`, `verifier_failures[]`, each report's `path`, `id`, `phase`, `status`, `checks[]`, `findings[]`, the window dates | `observations[]` of three kinds with `ref` and `sources[]`, `notes` |
| `session-pattern-reader` | step 4, with steps 3 and 5; skipped unless `sessions.status` is `present` | `sessions.files[]`, `sessions.commands[]`, `sessions.repeats[]`, `send_backs[]`, `state.current_phase`, the window dates | `observations[]` of two kinds with `ref`, `count`, `evidence[]`, plus `sessions_read`, `notes` |
| `debt-aggregator` | step 5, with steps 3 and 4 | `deferrals[]`, `window.to`, the two context index row sets | `as_of`, `total`, `oldest_days`, `groups[]` with `items[]` |
| `recommendation-drafter` | step 7, alone, after steps 3 to 6 | the id-bearing observations, `floors`, the six-row target table `templates/rec-targets.md`, the installed paths `Glob` returns under `.claude/` and the two `.devforgeai/` configuration files | `recommendations[]` with `ref` and `target`, `dropped[]`, `notes` |

Each agent returns a local `ref` — `om-nnn`, `sp-nnn`, `rd-nnn` — and the run attaches the allocated id at step 6 or step 8. Two agents emitting observations in parallel would collide on a self-allocated id, which is why the ids come from the CLI.

## Documents

| Document | Path | Template |
|---|---|---|
| Reflect report | `.devforgeai/reports/reflect-<date>.yaml`, one per date | `templates/reflect-report.yaml` |
| Target table | passed to `recommendation-drafter` at step 7 | `templates/rec-targets.md` |

`<date>` is the UTC date of the run in `YYYY-MM-DD`. Doc type `reflect-report`, schema `devforgeai/reflect-report/1`. Twelve top-level keys, in this order and with no other top-level key: `schema`, `id`, `phase`, `status`, `produced_by`, `consumes`, `open_questions`, `window`, `sources`, `observations`, `recommendations`, `technical_debt`. The first seven are the conventions §5 keys in §5 order; the last five are the payload this doc type permits. `phase` is `reflect` and `produced_by` is `improving-framework`.

`id` is the `<date>` of the filename — this is the one doc type whose id is a date rather than an allocated `PREFIX-nnn`. `status` runs two values: `draft` when step 9 started and the run stopped before the file was complete, `final` when step 9 finished. `consumes` holds every subject id the aggregate's `reports[]` named, ascending and de-duplicated. The ids inside the document are `OBS-nnn` and `REC-nnn`, allocated by `devforgeai doc validate --allocate`.

A second `/reflect` on the same date rewrites `reflect-<date>.yaml` over the first, with newly allocated ids from the global counter. The `{id}` token of the gate entry resolves to one path, which is what one document per date buys.

`.devforgeai/reports/<date>-reflect.yaml` is the CLI's gate report for the run, written by `gate check` at step 10. The two files differ by which side of the date the word sits on. The handoff's `Full report:` line cites `reflect-<date>.yaml`, because the recommendations live there.

`references/observations.md` carries the five observation kinds and the entry shape; `references/recommendations.md` carries the six target kinds, the floors, and the entry shape.

## Send-back

Reflect emits no SEND BACK. Conventions §5 gives its "May send back to" column as any phase, reached as a recommendation and not as a gate, and a recommendation is not a send-back: a send-back is a gate result that blocks a phase until the cited IDs are re-opened, and a `REC-nnn` blocks nothing. The reflect gate's `send_back_to` is `""` and its `on_fail` is `fail`, so `gate check --phase reflect` exits 0 or 1 and the exit-2 path is unreachable here. The one condition that would otherwise produce a send-back — a `REC-nnn` naming a defect in an upstream document — is out of scope: this skill's targets are the installed framework files under `.claude/` and the two `.devforgeai/` configuration files, not the `.devforgeai/` documents a phase produced.

Reflect receives no SEND BACK. No phase cites a `reflect-<date>.yaml` in a gate check, no phase's `requires` names `reflect`, and no `[[gate]]` other than this one carries `send_back_to = "reflect"`.

The Stop hook renders the closing block from the report; this skill writes no part of it. On a PASS the block returns the user to the phase they were in, holding a document of recommendations they may act on at any time:

```
Next      /build STORY-014
Then      /verify STORY-014
```

`Next` is the command for `state.toml` `current_phase` with `[active].<current_phase>` as its id, and `Then` the command for the phase after it in the §5 sequence, omitted when `current_phase` is `release`. With `[active].<current_phase>` empty, the id is dropped and `Next` is the bare command. On a FAIL, `Next` re-runs the same window as `/reflect <the run's own arguments>` and `Then` carries the line above. The `Found` line is omitted in both, because §6 scopes it to SEND BACK, and the `Verified` line is omitted because no subagent here is a registered verifier.

## Remedy and resume

`/reflect` takes neither `--remedy` nor `--resume`. Those two flags re-open cited IDs in an upstream phase and continue a downstream phase where it stopped, and this skill sits in neither relationship: it cites no ids into another phase's document and holds no position in the phase sequence to resume from.

A defect in a reflect report is corrected by running `/reflect` again for the same window. The run rewrites `reflect-<date>.yaml` at the same path with freshly allocated `OBS-nnn` and `REC-nnn` ids, and step 10 rewrites the gate report beside it. A run whose gate failed leaves the document on disk and the failing check ids in `.devforgeai/reports/<date>-reflect.yaml`, so the second run has both in view.

The window arguments are re-typed exactly as the handoff printed them, which for a FAIL is the `Next` line of the block the run just produced.

## References

- `references/session-history.md` — read before step 4: the project-key derivation with its three worked paths, where the session files sit, the JSONL fields a slash-command line carries, how `sessions.commands[]` and `sessions.repeats[]` are built from them, and what each of the four `sessions.status` values does to the run.
- `references/observations.md` — read before step 6: the five observation kinds with the part of the aggregate each is detected from, what `count` and `metric` mean per kind, the severity ranking, the ten fields of an entry, and the ordering the ids are allocated in.
- `references/recommendations.md` — read before step 7 and again at step 9: the six target kinds with their path shapes, the six compiled floors and the `open_questions` form a dropped proposal takes, the ten fields of an entry, and which files a recommendation may name.
