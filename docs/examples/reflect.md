# `/reflect` — a worked walkthrough

Cross-cutting. Reflect reads what the framework already wrote about itself and turns it into two numbered lists: `OBS-nnn` observations of what happened in a window, and `REC-nnn` recommendations each naming one framework file and one change. Plus a technical-debt section listing the deferred Definition-of-Done items with their ages.

It applies no change and edits no upstream document. A `REC-nnn` is prose until you run a command against the file it names.

---

## Entry

| Form | Mode | Window |
|---|---|---|
| `/reflect IDEA-nnn` | id | every report whose `id` is that idea |
| `/reflect EPIC-nnn` | id | every report for a `STORY-nnn` whose story frontmatter `consumes` holds that epic |
| `/reflect STORY-nnn` | id | every report whose `id` is that story |
| `/reflect vX.Y.Z` | id | every report for a `STORY-nnn` listed in `.devforgeai/releases/vX.Y.Z.yaml` |
| `/reflect --since YYYY-MM-DD` | since | every report whose `finished_at` is at or after that date at 00:00:00Z |

One preamble line, and it is the whole window:

```
!`devforgeai report aggregate $ARGUMENTS --json`
```

`$ARGUMENTS` here is the whole string, not `$ARGUMENTS[0]`, because `--since 2026-09-01` is two tokens.

**Every count and every timestamp in the report comes from that aggregate.** Counting reports, differencing timestamps, and tallying check ids is arithmetic the binary does, which is why this skill reads the aggregate and re-derives nothing from the report files.

It prints one `devforgeai/cli-json/1` envelope whose `data` object is the `devforgeai/aggregate/1` window with eleven keys: `window`, `reports[]`, `phase_time[]`, `gate_failures[]`, `send_backs[]`, `verifier_failures[]`, `deferrals[]`, `sessions`, `state`, `floors`, `counts`. Every key is present on every run; an empty result is an empty array or a zero.

```
$ devforgeai report aggregate --help
Collect a window of reports

Usage: devforgeai.exe report aggregate [OPTIONS] [ID]

Arguments:
  [ID]  The window subject

Options:
      --json                 One JSON envelope on stdout, no human text on stdout
      --since <YYYY-MM-DD>   The window start
      --project <path>       Project root override; `init` defaults to the working directory
      --session-root <path>  The session root override
      --quiet                Suppress human stdout; stderr is unaffected
  -h, --help                 Print help
```

Reproduced.

### The two ways the preamble stops the run

```
> /reflect lastweek
```

exits 3 on `DFA-E013`, whose stderr line reads `'lastweek' is not an ID`.

```
> /reflect
> /reflect STORY-001 --since 2026-09-01
```

both exit 3 on `DFA-E430`:

```
pass one id (IDEA-nnn, EPIC-nnn, STORY-nnn, vX.Y.Z) or --since <YYYY-MM-DD>
```

which is the neither-or-both rule `report aggregate` holds. Either way the body does not load, and the stderr line is what reaches you.

### Why there is no `gate require` and no `doc load`

There is no `gate require` line, because the reflect gate's `requires` is `""` and a call with no predecessor to test tells the run nothing. There is no `doc load` line, because every document this run reads reaches it through the aggregate.

The line allocates no id: `OBS` and `REC` ids are allocated at steps 6 and 8, once the window has produced something to number, so a window that matched no report spends no id.

---

## The exchange

```
> /reflect v0.1.0
```

**Step 1 — establish the window.** The mode is `since` when `$ARGUMENTS` starts with `--since`, and `id` otherwise. Fill `window.mode`, `window.subject` (the `$1` id in id mode, `""` in since mode), `window.from` (the earliest report `finished_at` in the window), and `window.to` (the run date). `window.generated_at` and `window.cli_version` come from the aggregate's own `window` block.

The run date in UTC, `YYYY-MM-DD`, is the document's `id` and the `<date>` in its filename.

**Step 2 — read the aggregate.** Parse the preamble's stdout and take the `data` object's eleven keys. Fill `sources.reports[]` from `reports[]` in aggregate order, one entry per report with `path`, `id`, `phase`, `status`, `started_at`, `finished_at`; fill `sources.sessions` from `sessions`; fill `sources.state` from `state`. Fill `consumes` with every subject id `reports[]` named, ascending and de-duplicated.

`data.counts.reports` of `0` means the window matched no report. The document is written with `observations: []`, `recommendations: []`, `technical_debt.total: 0`, and one `open_questions` line reading:

```
The window 2026-09-01 to 2026-09-11 matched no report
```

then the run continues at step 9.

**Steps 3, 4 and 5 — three subagents, one message.**

**3. `observation-miner`** takes `phase_time[]`, `gate_failures[]`, `verifier_failures[]`, the `checks[]` and `findings[]` of each `reports[]` entry, and the window dates. It returns `observations[]` of kinds `gate_failure`, `phase_time`, and `verifier_unparsed`, each with a `ref` and its `sources[]`, plus `notes`.

**4. `session-pattern-reader`** takes `sessions.files[]`, `sessions.commands[]`, `sessions.repeats[]` with the `path` and `line` of each repeat so the agent reads the surrounding lines, `send_backs[]`, `state.current_phase`, and the window dates. It returns `observations[]` of kinds `repeated_send_back` and `friction`, each with a `ref`, a `count`, and `evidence[]` carrying a file path, a line number, and a timestamp.

A `sessions.status` of `absent`, `empty`, or `unreadable` **skips** this step: the agent is not invoked, the step returns `observations: []`, and `sources.sessions.reason` carries the resolved path. Where the session files live comes from:

```toml
[reflect]
session_root = "~/.claude/projects"
session_key = ""
window_days = 14
```

Reproduced from `.devforgeai/config.toml`.

**5. `debt-aggregator`** takes `deferrals[]` with the `CON-nnn` or `AP-nnn` each record cites, `window.to`, and the `## Constraint index` and `## Anti-pattern index` rows of the two context files. It returns `as_of`, `total`, `oldest_days`, and `groups[]`, each group carrying `constraint`, `kind`, `count`, `oldest_days`, and `items[]` with `story`, `dod_item`, `deferred_at`, `age_days`, `reason`, and `report`.

A record citing neither a `CON-nnn` nor an `AP-nnn` lands in the group whose `constraint` is `none` and whose `kind` is `none`.

**Step 6 — allocate the OBS ids.** Merge the observations of steps 3 and 4, order them by `severity` descending then by `kind` in the `references/observations.md` table order, then:

```
devforgeai doc validate --allocate OBS
```

once per observation, in that order. Keep a map from each agent's `ref` to its allocated id, which step 7 reads.

Each agent returns a local `ref` — `om-nnn`, `sp-nnn`, `rd-nnn` — and the run attaches the allocated id here. Two agents emitting observations in parallel would collide on a self-allocated id, which is why the ids come from the CLI.

An observation whose `sources[]` came back empty is dropped here and one line naming its `ref` goes into `open_questions`. Exit 1 on `DFA-E215` means the prefix is exhausted; the run stops with the stderr line in hand.

**Step 7 — draft the recommendations.** `recommendation-drafter`, alone, after steps 3 to 6 return. It takes the id-bearing observations, the aggregate's `floors` block, the six-row target table `templates/rec-targets.md`, and the paths `Glob` returns under `.claude/skills/`, `.claude/agents/`, and `.claude/settings.json`, plus `.devforgeai/gates.toml` and `.devforgeai/config.toml`.

Those are the files an installed project holds; the repository the framework is built in is not on disk here.

It returns `recommendations[]`, each with a `ref`, an `observations[]` list of `OBS-nnn`, a `target` object, a `change` string, `current_value`, `proposed_value`, `effort`, and `applies_to[]`, plus `dropped[]` and `notes`.

**The six target kinds are ordered by specificity, and the first row a target matches is the one it takes.** A recommendation whose `target.path` is `.devforgeai/gates.toml` or `.devforgeai/config.toml` takes `kind: gate_threshold` with the dotted key in `target.key`, whatever the change is. `framework_file` is the last row for that reason: it names a path the five rows above it leave, not any path under `.claude/` or `.devforgeai/` at large.

A drafted recommendation whose `target.kind` is `gate_threshold` and whose `proposed_value` sits below the matching entry of `floors` is dropped by the agent, and the drop arrives in `dropped[]` naming the key, the proposed value, and the floor. The binary rejects such a value:

```
$ devforgeai gate check --phase verify --id STORY-001
devforgeai: DFA-E303 gate check: gates.toml gate 'verify' sets verifier_pass.min_ratio to 0.5, below the compiled minimum 1
```

Reproduced, exit 1 — so a proposal below a floor carries a number nothing can apply.

**Step 8 — allocate the REC ids.**

```
devforgeai doc validate --allocate REC
```

once per recommendation, in step 7's order. Exit 1 on `DFA-E215` stops the run, as at step 6.

**Step 9 — write the report.** `.devforgeai/reports/reflect-2026-09-11.yaml`, twelve top-level keys in this order, `status: final`:

```yaml
schema: devforgeai/reflect-report/1
id: 2026-09-11
phase: reflect
status: final
produced_by: improving-framework
consumes: [STORY-001, v0.1.0]
open_questions: []
window:
  mode: id
  subject: v0.1.0
  from: 2026-09-01
  to: 2026-09-11
  generated_at: 2026-09-11T19:02:43Z
  cli_version: 1.0.0
sources:
  reports: [...]
  sessions: {...}
  state: {...}
observations: []
recommendations: []
technical_debt:
  as_of: 2026-09-11
  total: 0
  oldest_days: 0
  groups: []
```

Shaped from `templates/reflect-report.yaml`. The first seven keys are the conventions §5 set; the last five are the payload this doc type permits.

`id` is the `<date>` of the filename — **this is the one doc type whose id is a date** rather than an allocated `PREFIX-nnn`. `status` runs two values: `draft` when step 9 started and the run stopped before the file was complete, `final` when step 9 finished.

Each `dropped[]` entry of step 7 becomes one `open_questions` line:

```
REC dropped: verifier_pass.min_ratio proposed 0.5, floor 1.0
```

The `PostToolUse` hook runs `doc validate` on the written path and hands its result back as `additionalContext`, naming the key to rewrite.

**Step 10 — gate.**

```
devforgeai gate check --phase reflect --id 2026-09-11
```

Exit 0 writes `.devforgeai/reports/2026-09-11-reflect.yaml`, the CLI's gate report for the run, which is a **second** file beside the document step 9 wrote. Exit 1 names the failing check ids on stderr; rewrite the entries the check named and run step 10 once more.

`gate check --phase reflect` takes a `YYYY-MM-DD` `--id` and has no `[active]` key to default from, so omitting it is `DFA-E011`.

### The two files, and which is which

| Path | Written by | Holds |
|---|---|---|
| `reports/reflect-<date>.yaml` | this skill, step 9 | the observations, the recommendations, the debt |
| `reports/<date>-reflect.yaml` | `gate check`, step 10 | the gate result and its check entries |

They differ by which side of the date the word sits on. The handoff's `Full report:` line cites `reflect-<date>.yaml`, because the recommendations live there.

**Step 11 — record the cross-cutting run.**

```
devforgeai phase set reflect --id 2026-09-11
```

as the **last** step. It records `[last_cross]` in `state.toml` with `phase`, `id`, and the turn, and leaves `[current]` where it found it. That is the whole of this step: this skill runs no `devforgeai handoff` of its own.

---

## The gate

```toml
[[gate]]
phase = "reflect"
requires = ""
on_fail = "fail"
send_back_to = ""
description = "The reflect report exists and parses, every REC cites an OBS, every OBS cites a report path or a session id, and no REC lowers a compiled floor."

  [[gate.check]]
  kind = "file_exists"
  id = "reflect-report-exists"
  ...

  [[gate.check]]
  kind = "doc_valid"
  id = "reflect-docs"
  ...

  [[gate.check]]
  kind = "yaml_cites"
  ...
```

Reproduced from the file `init` wrote. The required check kinds for this phase are `file_exists`, `doc_valid`, `yaml_cites`, and `no_threshold_decrease`.

`yaml_cites` is what makes a `REC-nnn` without an `OBS-nnn`, or an `OBS-nnn` without a source, a gate failure:

| Code | Condition |
|---|---|
| `DFA-E346` | `<doc>: <from>[<n>] id '<id>' cites <count> of <min> required` |
| `DFA-E347` | `<doc>: <from>[<n>] id '<id>' cites '<value>', which <into> does not define` |

`no_threshold_decrease` is what catches a recommendation the drafter did not drop:

| Code | Condition |
|---|---|
| `DFA-E348` | `<doc>: <from>[<n>] id '<id>' proposes <key> = <value>, below the compiled floor <floor>` |

`send_back_to = ""` and `on_fail = "fail"` together mean `gate check --phase reflect` exits 0 or 1 and the exit-2 path is unreachable here.

---

## The handoff

A `/reflect` run leaves `current_phase` where it found it, holds no key in `state.toml` `[active]`, and is not a phase in the §5 sequence. The Stop hook reads `[last_cross]`, renders Reflect's block and the block for the phase you were in, joins the two, and clears `[last_cross]`.

On a PASS the block returns you to the phase you were in, holding a document of recommendations you may act on at any time:

```
Next      /build STORY-014
Then      /verify STORY-014
```

`Next` is the command for `state.toml` `current_phase` with `[active].<current_phase>` as its id, and `Then` the command for the phase after it in the §5 sequence, omitted when `current_phase` is `release`. With `[active].<current_phase>` empty, the id is dropped and `Next` is the bare command.

On a FAIL, `Next` re-runs the same window as `/reflect <the run's own arguments>` and `Then` carries the line above.

Two lines are omitted in both cases:

- **`Found`**, because §6 scopes it to SEND BACK.
- **`Verified`**, because none of the four subagents here is a registered verifier, so `SubagentStop` ingests none of their output.

The two-block rendering is the same as Design's. A worked example of it, reproduced:

```
$ devforgeai phase set design --id UI-001
Phase     design · UI-001  (cross-cutting)

$ echo '{"stop_hook_active":false,"session_id":"doc"}' | devforgeai hook run stop
```

whose `systemMessage`, unescaped, holds two blocks joined by a blank line — the cross-cutting one first, then the phase's own. A second Stop in the same session prints one block.

---

## Send-back

**Reflect emits none.** Conventions §5 gives its "May send back to" column as any phase, reached as a recommendation and not as a gate — and a recommendation is not a send-back. A send-back is a gate result that blocks a phase until the cited ids are re-opened, and a `REC-nnn` blocks nothing.

The one condition that would otherwise produce a send-back — a `REC-nnn` naming a defect in an upstream document — is out of scope: this skill's targets are the installed framework files under `.claude/` and the two `.devforgeai/` configuration files, not the `.devforgeai/` documents a phase produced.

**Reflect receives none.** No phase cites a `reflect-<date>.yaml` in a gate check, no phase's `requires` names `reflect`, and no `[[gate]]` other than this one carries `send_back_to = "reflect"`.

---

## No `--remedy`, no `--resume`

`/reflect` takes neither. Those two flags re-open cited ids in an upstream phase and continue a downstream phase where it stopped, and this skill sits in neither relationship: it cites no ids into another phase's document and holds no position in the phase sequence to resume from.

A defect in a reflect report is corrected by running `/reflect` again for the same window. The run rewrites `reflect-<date>.yaml` at the same path with freshly allocated `OBS-nnn` and `REC-nnn` ids from the global counter, and step 10 rewrites the gate report beside it. A run whose gate failed leaves the document on disk and the failing check ids in `reports/<date>-reflect.yaml`, so the second run has both in view.

The window arguments are re-typed exactly as the handoff printed them, which for a FAIL is the `Next` line of the block the run just produced.

A second `/reflect` on the same date rewrites `reflect-<date>.yaml` over the first. The `{id}` token of the gate entry resolves to one path, which is what one document per date buys.

---

## Files this phase owns

| Document | Path | Template |
|---|---|---|
| Reflect report | `.devforgeai/reports/reflect-<date>.yaml`, one per date | `templates/reflect-report.yaml` |
| Target table | passed to `recommendation-drafter` at step 7 | `templates/rec-targets.md` |

Doc type `reflect-report`, schema `devforgeai/reflect-report/1`. `phase` is `reflect` and `produced_by` is `improving-framework`. `consumes` holds every subject id the aggregate's `reports[]` named, ascending and de-duplicated. The ids inside the document are `OBS-nnn` and `REC-nnn`.
