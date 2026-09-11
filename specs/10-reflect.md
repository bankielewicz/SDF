---
schema: devforgeai-spec/1
doc: reflect
status: draft
produced_by: reflect-spec-author
consumes: [00-conventions]
open_questions: []
---

# Cross-cutting · Reflect · `improving-framework`

## Scope

`improving-framework` reads what the framework already wrote about itself and turns it into two numbered lists. It runs on one slash command, `/reflect`, in two forms: `/reflect <ID>`, where `<ID>` is an `IDEA-nnn`, `EPIC-nnn`, `STORY-nnn`, or a `vX.Y.Z` version, and `/reflect --since <YYYY-MM-DD>`. Its three inputs are the CLI-written reports under `.devforgeai/reports/`, the Claude Code session history for the project under `~/.claude/projects/<project-key>/*.jsonl`, and the deferral records inside Verify's QA reports. Its one output is `.devforgeai/reports/reflect-<date>.yaml`, holding `OBS-nnn` observations (friction points, repeated send-backs, gate failures by check name, time spent per phase, subagent output that failed schema validation), `REC-nnn` recommendations (one target apiece, each citing the `OBS-nnn` behind it), and a technical-debt section listing every deferred Definition-of-Done item across stories with its age in days, grouped by the `CON-nnn` or `AP-nnn` it cites.

`improving-framework` is not a phase. It has no number in §5's phase sequence and no key in `state.toml` `[active]`, it runs no `phase set`, and a run leaves `current_phase` as it found it. It writes no requirement, no context file, no ADR, no story, no UI spec, and no production code. It changes no file under `.claude/skills/`, `.claude/agents/`, `.claude/settings.json`, `.devforgeai/gates.toml`, or `.devforgeai/config.toml`: a `REC-nnn` names the file and the change, and applying one is the user's next command against that file. It emits no SEND BACK and receives none. It reads the aggregate the CLI computes and does not re-derive counts from the report files itself. It owns one `[[gate]]` entry, which tests the shape of its own document and nothing about the project's code.

## Inputs

| Input | Path | IDs | Read by | Producer |
|---|---|---|---|---|
| Window aggregate | stdout of `devforgeai report aggregate $ARGUMENTS --json` | every ID in the window | steps 2 to 5, 7 | CLI, this spec's Decision 2 |
| Explore gate report | `.devforgeai/reports/IDEA-nnn-explore.yaml` | `IDEA-nnn`, `FLOW-nnn` | the aggregate | `gate check`, `report ingest` of `kill-case-builder` |
| Discover gate report | `.devforgeai/reports/IDEA-nnn-discover.yaml` | `IDEA-nnn`, `REQ-nnn`, `FLOW-nnn` | the aggregate | `gate check`, per `specs/03-discover.md` `## Integration` |
| Constitute gate report | `.devforgeai/reports/IDEA-nnn-constitute.yaml` | `IDEA-nnn`, `REQ-nnn`, `CON-nnn`, `ADR-nnn` | the aggregate | `gate check`, `report ingest` of `alignment-auditor` and `architecture-reviewer` |
| Plan gate report | `.devforgeai/reports/SPRINT-nnn-plan.yaml` | `SPRINT-nnn`, `STORY-nnn`, `UI-nnn`, `AC-nnn` | the aggregate | `gate check`, per `specs/05-plan.md` |
| Build gate report | `.devforgeai/reports/STORY-nnn-build.yaml` | `STORY-nnn`, `AC-nnn`, `TOKEN-<name>` | the aggregate | `gate check`, per `specs/06-build.md` |
| QA document | `.devforgeai/reports/STORY-nnn-qa.yaml` | `STORY-nnn`, `FIND-nnn`, `CON-nnn`, `AP-nnn` | the aggregate, step 5 | `validating-quality`, per `specs/07-verify.md` |
| Verify gate report | `.devforgeai/reports/STORY-nnn-verify.yaml` | `STORY-nnn`, `FIND-nnn`, `AC-nnn` | the aggregate | `gate check`, `report ingest` of `ac-compliance-verifier` |
| Release gate report | `.devforgeai/reports/vX.Y.Z-release.yaml` | `STORY-nnn` | the aggregate | `gate check`, per `specs/09-release.md` |
| Release manifest | `.devforgeai/releases/vX.Y.Z.yaml` | `STORY-nnn` | the aggregate | `releasing-software`, per `specs/09-release.md` |
| Design report | `.devforgeai/reports/UI-nnn-design.yaml` | `UI-nnn`, `FLOW-nnn`, `REQ-nnn` | the aggregate | `report ingest` of `requirement-coverage-auditor` |
| Session history | `<session_root>/<project-key>/*.jsonl` | none | the aggregate, step 4 | Claude Code |
| Phase state | `.devforgeai/state.toml`, keys `current_phase`, `[active]`, `[last_gate]`, `[last_handoff]` | the active IDs | the aggregate, step 11 | `phase set`, `gate check`, `handoff` |
| Session root and key | `.devforgeai/config.toml`, keys `[reflect].session_root`, `[reflect].session_key`, `[reflect].window_days` | none | the aggregate | `init`, this spec's Decision 3 |
| Compiled floors | `.devforgeai/config.toml` `[[layer]].coverage_min` and `[coverage].overall_min`; `.devforgeai/gates.toml` `verifier_pass.min_ratio` | none | the aggregate, step 7 | `init`, the binary's compiled table |
| Next free OBS id | stdout of `devforgeai doc validate --allocate OBS` | `OBS-nnn` | step 6 | CLI |
| Next free REC id | stdout of `devforgeai doc validate --allocate REC` | `REC-nnn` | step 8 | CLI |

**The project key.** `<project-key>` is the absolute path of the project root with every character outside `[A-Za-z0-9]` replaced by `-`, one replacement character per source character. `C:\Projects\DevForgeAI` gives `C--Projects-DevForgeAI`; `C:\Projects\New folder` gives `C--Projects-New-folder`; `\\wsl$\Ubuntu\home\bryan\Projects\DevForge` gives `--wsl--Ubuntu-home-bryan-Projects-DevForge`. `[reflect].session_key` overrides the derived value when it is non-empty.

**When the session directory is absent.** `<session_root>/<project-key>/` missing, present and holding no `*.jsonl` file, or present and unreadable each produce the same result: the aggregate's `sessions.status` is `absent`, `unreadable`, or `empty`, `sessions.files` is `[]`, `sessions.commands` is `[]`, `sessions.repeats` is `[]`, and `sessions.reason` carries the resolved path. Step 4 is skipped, `session-pattern-reader` is not invoked, and every `OBS-nnn` in the run cites a report path. The gate result is unchanged, because `reflect-obs-cites-source` accepts a report path or a session id and this path supplies the first.

## Outputs

One document, `.devforgeai/reports/reflect-<date>.yaml`, where `<date>` is the UTC date of the run in `YYYY-MM-DD`. Doc type `reflect-report`, schema `devforgeai/reflect-report/1`. Top-level keys, in this order, with no other top-level key: `schema`, `id`, `phase`, `status`, `produced_by`, `consumes`, `open_questions`, `window`, `sources`, `observations`, `recommendations`, `technical_debt`. The first seven are the §5 keys in §5 order; the last five are the payload `specs/01-cli.md` Decision 44 permits for this doc type.

`status` enum, closed at two: `draft` (step 9 started and the run stopped before the file was complete), `final` (step 9 finished). `id` is the `<date>` of the filename. `consumes` holds every subject ID the aggregate's `reports[]` named, ascending, de-duplicated.

```yaml
schema: devforgeai/reflect-report/1
id: 2026-09-11
phase: reflect
status: final
produced_by: improving-framework
consumes: [SPRINT-001, STORY-012, STORY-014]
open_questions: []

window:
  mode: id                          # string; enum: id | since
  subject: STORY-014                # string; the $1 id in id mode, "" in since mode
  from: 2026-08-28                  # string; YYYY-MM-DD; the earliest report finished_at in the window
  to: 2026-09-11                    # string; YYYY-MM-DD; the run date
  generated_at: 2026-09-11T09:14:02Z   # string; RFC 3339 UTC
  cli_version: 1.0.0                # string; copied from the aggregate

sources:
  reports:                          # array; one entry per report the aggregate read, in aggregate order
    - path: .devforgeai/reports/STORY-014-build.yaml   # string; project-relative
      id: STORY-014                 # string
      phase: build                  # string; phase enum
      status: pass                  # string; enum: pass | fail | send_back | skip
      started_at: 2026-09-03T10:58:11Z    # string; RFC 3339 UTC or ""
      finished_at: 2026-09-03T11:02:41Z   # string; RFC 3339 UTC or ""
  sessions:
    status: present                 # string; enum: present | absent | empty | unreadable
    root: C:\Users\bryan\.claude\projects        # string; the resolved session root
    key: C--Projects-DevForgeAI     # string; the project key used
    reason: ""                      # string; "" when status is present, the resolved path otherwise
    files:                          # array; one entry per *.jsonl read; [] unless status is present
      - session_id: ad1071cd-4d3c-4d0f-acb9-52f21ca81fba   # string
        path: C:\Users\bryan\.claude\projects\C--Projects-DevForgeAI\ad1071cd-4d3c-4d0f-acb9-52f21ca81fba.jsonl
        from: 2026-09-03T10:41:02Z  # string; RFC 3339 UTC; first line timestamp
        to: 2026-09-03T12:18:55Z    # string; RFC 3339 UTC; last line timestamp
        lines: 812                  # integer
  state:
    current_phase: build            # string; phase enum; copied from state.toml
    active_id: STORY-014            # string; state.toml [active].<current_phase>
    last_gate_result: PASS          # string; enum: PASS | FAIL | SEND_BACK | TRUST_FAIL | NOT_RUN

observations:                       # array; ascending by id
  - id: OBS-014                     # string; OBS-nnn
    kind: repeated_send_back        # string; enum of five, below
    severity: high                  # string; enum: low | medium | high
    phase: verify                   # string; phase enum, or "" when the observation spans phases
    summary: Verify sent STORY-014 back to Build three times on the same AC   # string; 1 to 120 chars
    detail: >                       # string; 1 to 400 chars
      verify-acs failed on AC-007 in three consecutive runs, each returning
      /build STORY-014 --resume, with no change to the AC text between them.
    count: 3                        # integer; occurrences in the window; 1 when the kind has no repeat notion
    metric: ""                      # string; "" unless kind is phase_time, then "<n>m <n>s"
    sources:                        # array[string]; 1 or more; report paths and session ids
      - .devforgeai/reports/STORY-014-verify.yaml
      - ad1071cd-4d3c-4d0f-acb9-52f21ca81fba
    evidence:                       # array; at most 5 entries, in time order
      - path: C:\Users\bryan\.claude\projects\C--Projects-DevForgeAI\ad1071cd-....jsonl
        line: 214                   # integer; 1-based line in the cited file; 0 for a report path
        at: 2026-09-03T11:41:09Z    # string; RFC 3339 UTC

recommendations:                    # array; ascending by id
  - id: REC-009                     # string; REC-nnn
    observations: [OBS-014, OBS-003]   # array[string]; 1 or more OBS-nnn defined in this document
    target:
      kind: subagent                # string; enum of six, below
      path: agents/ac-compliance-verifier.md   # string; repo-relative under §3, or .devforgeai-relative for gate_threshold
      key: ""                       # string; dotted key inside the target file; "" unless kind is gate_threshold
    change: >                       # string; 1 to 300 chars; one change, in the imperative
      Add the failing AC id to the findings entry so the send-back names the
      criterion rather than the story.
    current_value: ""               # string; the value at the key today; "" unless kind is gate_threshold
    proposed_value: ""              # string; the value asked for; "" unless kind is gate_threshold
    effort: small                   # string; enum: small | medium | large
    applies_to: [verify]            # array[string]; phase enum entries; [] when framework-wide

technical_debt:
  as_of: 2026-09-11                 # string; YYYY-MM-DD; equals window.to
  total: 6                          # integer; deferral records in the window
  oldest_days: 41                   # integer; 0 when total is 0
  groups:                           # array; ascending by constraint, with "none" last
    - constraint: CON-002           # string; CON-nnn, AP-nnn, or "none"
      kind: constraint              # string; enum: constraint | anti_pattern | none
      count: 2                      # integer
      oldest_days: 41               # integer
      items:                        # array; descending by age_days
        - story: STORY-009          # string; STORY-nnn
          dod_item: Integration test for the retry path   # string; 1 to 120 chars
          deferred_at: 2026-08-01   # string; YYYY-MM-DD
          age_days: 41              # integer; as_of minus deferred_at, in calendar days
          reason: Retry backoff is unspecified until ADR-011 is accepted   # string; 1 to 200 chars
          report: .devforgeai/reports/STORY-009-qa.yaml   # string
```

**`observations[].kind`, closed at five.**

| `kind` | Source in the aggregate | `count` means |
|---|---|---|
| `friction` | `sessions.commands[]`, a command re-typed inside one session with the same `$1` and no intervening gate `PASS` | re-typings |
| `repeated_send_back` | `send_backs[]`, two or more entries with the same `from`, `to`, and `id` | send-backs on that triple |
| `gate_failure` | `gate_failures[]`, one entry per failing `check_id` | failures of that check |
| `phase_time` | `phase_time[]`, one entry per phase in the window | runs of that phase; `metric` carries the total |
| `verifier_unparsed` | `verifier_failures[]`, a `status` of `unparsed` (`DFA-E410`) or an unregistered name (`DFA-W411`) | reports carrying that status |

**`recommendations[].target.kind`, closed at six.** Every path is one the installed project holds: `.claude/` carries the skills, the agents and the hook block, and `.devforgeai/` carries the two configuration files. The repository the framework is built from is not on disk in a target project, so no recommendation names a path under it, and there is no `command` kind, because there is no `commands/` directory.

| `kind` | `path` shape | `key` |
|---|---|---|
| `skill` | `.claude/skills/<skill-name>/SKILL.md` | `""` |
| `subagent` | `.claude/agents/<agent-name>.md` | `""` |
| `template` | `.claude/skills/<skill-name>/templates/<file>` | `""` |
| `hook` | `.claude/settings.json` | the hook event name, e.g. `Stop` |
| `gate_threshold` | `.devforgeai/gates.toml` or `.devforgeai/config.toml` | a dotted key, e.g. `layer.domain.coverage_min` |
| `framework_file` | an installed path none of the five rows above names, e.g. `.claude/skills/<name>/references/<file>` | `""` |

The rows are ordered by specificity and a target takes the first row it matches. A recommendation whose `target.path` is `.devforgeai/gates.toml` or `.devforgeai/config.toml` takes `kind: gate_threshold` with the dotted key in `target.key`, whatever the change is; `framework_file` names neither of those two files.

A `skill` or `template` path uses the skill's directory name — `implementing-stories`, not `build` — because that is what `produced_by` resolves against; the slash command is the frontmatter `name`.

The CLI's own gate report for the run is `.devforgeai/reports/<date>-reflect.yaml`, written by `gate check --phase reflect --id <date>` with the `specs/01-cli.md` report schema. It is a second file from the document above, on the precedent `specs/01-cli.md` Decision 46 set for Verify, and the handoff's `Full report:` line cites the document rather than the gate report, because the document is what the user reads.

## Workflow

**1. Establish the window — model.** Input: `$ARGUMENTS` and the aggregate JSON on the preamble's stdout. The mode is `since` when `$ARGUMENTS` starts with `--since`, and `id` otherwise. Output: `window.mode`, `window.subject`, `window.from`, `window.to`. Failure path: the preamble exited 3 with `DFA-E430`, meaning `$ARGUMENTS` held neither a recognised id prefix nor `--since`; the run stops and the stderr line naming the four accepted prefixes and the `--since` form reaches the model.

**2. Read the aggregate — model.** Input: the preamble's stdout, parsed as the §4 JSON envelope. Output: the `data` object's `reports[]`, `phase_time[]`, `gate_failures[]`, `send_backs[]`, `verifier_failures[]`, `deferrals[]`, `sessions`, `state`, and `floors` in hand, and `sources` filled from `reports[]`, `sessions`, and `state`. Failure path: `data.counts.reports` is `0`, meaning the window matched no report; the run writes the document with `observations: []`, `recommendations: []`, `technical_debt.total: 0`, and one `open_questions` line reading `The window <from> to <to> matched no report`, and continues at step 9.

**3. Mine the reports — subagent `observation-miner`.** Input: `phase_time[]`, `gate_failures[]`, `verifier_failures[]`, and the `checks[]` and `findings[]` of each `reports[]` entry. Output: JSON with `observations[]` of kinds `gate_failure`, `phase_time`, and `verifier_unparsed`, each carrying a `ref` and its `sources[]`. Invoked in parallel with steps 4 and 5. Failure path: a returned observation whose `sources[]` is empty is dropped by the model at step 6 and one line naming its `ref` goes into `open_questions`.

**4. Read the session history — subagent `session-pattern-reader`.** Input: `sessions.files[]`, `sessions.commands[]`, and `sessions.repeats[]` from the aggregate, plus the `path` and `line` of each repeat so the agent reads the surrounding lines. Output: JSON with `observations[]` of kinds `repeated_send_back` and `friction`, each carrying a `ref`, a `count`, and `evidence[]` with a file path, a line number, and a timestamp. Invoked in parallel with steps 3 and 5. Failure path: `sessions.status` is `absent`, `empty`, or `unreadable`; the agent is not invoked, the step returns `observations: []`, and the document's `sources.sessions.reason` carries the resolved path.

**5. Collect the deferrals — subagent `debt-aggregator`.** Input: `deferrals[]` from the aggregate, the `CON-nnn` and `AP-nnn` ids each record cites, and `window.to`. Output: JSON with `groups[]`, each carrying `constraint`, `kind`, `count`, `oldest_days`, and `items[]` with `story`, `dod_item`, `deferred_at`, `age_days`, `reason`, and `report`. Invoked in parallel with steps 3 and 4. Failure path: a deferral record citing neither a `CON-nnn` nor an `AP-nnn` lands in the group whose `constraint` is `none` and whose `kind` is `none`.

**6. Allocate the OBS ids — model, CLI.** Input: the merged `observations[]` of steps 3 and 4, ordered by `severity` descending then `kind` in the §Outputs table order, and `devforgeai doc validate --allocate OBS`, run once per observation. Output: one `OBS-nnn` per observation, and a map from each agent's `ref` to its allocated id. Failure path: `--allocate` exits 1 on `DFA-E215` with the prefix exhausted; the run stops and the stderr line reaches the model.

**7. Draft the recommendations — subagent `recommendation-drafter`.** Input: the id-bearing `observations[]` of step 6, the aggregate's `floors` block, the target-kind table of `## Outputs`, `templates/rec-targets.md`, and the paths `Glob` returns under `.claude/skills/`, `.claude/agents/`, and `.devforgeai/`. Output: JSON with `recommendations[]`, each carrying a `ref`, an `observations[]` list of `OBS-nnn` ids, a `target` object, a `change` string, `current_value`, `proposed_value`, `effort`, and `applies_to[]`. Failure path: a drafted recommendation whose `target.kind` is `gate_threshold` and whose `proposed_value` is below the matching entry of `floors` is dropped by the agent, and one entry is added to its `dropped[]` array naming the key, the proposed value, and the floor.

**8. Allocate the REC ids — model, CLI.** Input: `recommendations[]` from step 7 and `devforgeai doc validate --allocate REC`, run once per recommendation. Output: one `REC-nnn` per recommendation, in step 7's order. Failure path: as step 6.

**9. Write the report — model.** Input: steps 1 to 8. Output: `.devforgeai/reports/reflect-<date>.yaml` with the twelve top-level keys in `## Outputs` order, `status: final`, and the `dropped[]` entries of step 7 written as `open_questions` lines in the form `REC dropped: <key> proposed <value>, floor <value>`. Failure path: the `PostToolUse` hook returns the `doc validate` diagnostic in `hookSpecificOutput.additionalContext`; the model rewrites the key it names and writes again.

**10. Gate — CLI.** Input: `devforgeai gate check --phase reflect --id <date>`. Output: `.devforgeai/reports/<date>-reflect.yaml` and exit 0. Failure path: exit 1 with the failing check ids on stderr; the model rewrites the entries the check named and reruns step 10 once.

**11. Record the cross-cutting run — CLI.** Input: `devforgeai handoff --phase reflect --id <date>`, run as the last step. Output: the §6 block, with `Next` and `Then` taken from `state.toml` by the table in `## Handoff`, and `state.toml` `[last_cross]` written with `phase: reflect`, the date, and the turn's timestamp. The Stop hook then renders that block and the block for `[current].phase`, which Reflect left as it found it, joins the two into one `systemMessage`, and clears `[last_cross]`. The Stop hook renders the closing block; this skill writes no part of it.

## Subagents

### observation-miner

- **name**: `observation-miner`
- **derives_from**: `C:\Users\bryan\.claude\agents\observation-extractor.md`
- **purpose**: Name the gate failures, phase durations, and unparsed verifier blocks in one window aggregate as observations, each tied to the report paths that carry it.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `sonnet` — the counts arrive already computed; the judgment is which of them is worth a line and how to phrase it.
- **input**: the aggregate's `phase_time[]`, `gate_failures[]`, `verifier_failures[]`, and, per `reports[]` entry, `path`, `id`, `phase`, `status`, `checks[]`, and `findings[]`; `window.from` and `window.to`.
- **output**:

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

### session-pattern-reader

- **name**: `session-pattern-reader`
- **derives_from**: `C:\Users\bryan\.claude\agents\session-miner.md`
- **purpose**: Name the repeated send-backs and re-typed commands in a project's session history as observations, each tied to a session id and a line number.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `opus` — reading a sequence of commands and saying which repetition is one pattern rather than three incidents is the judgment this skill exists for.
- **input**: the aggregate's `sessions.files[]`, `sessions.commands[]`, and `sessions.repeats[]`; `window.from` and `window.to`; the phase names of `state.current_phase` and each repeat's `from` and `to`.
- **output**:

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

### debt-aggregator

- **name**: `debt-aggregator`
- **derives_from**: `C:\Users\bryan\.claude\agents\technical-debt-analyzer.md`
- **purpose**: Group every deferred Definition-of-Done item in the window by the constraint or anti-pattern it cites, with each item's age in calendar days.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `sonnet` — the grouping key and the age arrive in the aggregate; the judgment is the group title and which reason text belongs on each row.
- **input**: the aggregate's `deferrals[]` (`story`, `dod_item`, `deferred_at`, `age_days`, `constraint`, `reason`, `report`), `window.to`, and the `## Constraint index` and `## Anti-pattern index` rows of `.devforgeai/context/architecture-constraints.md` and `.devforgeai/context/anti-patterns.md`.
- **output**:

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

### recommendation-drafter

- **name**: `recommendation-drafter`
- **derives_from**: `C:\Users\bryan\.claude\agents\framework-analyst.md`
- **purpose**: Turn a set of id-bearing observations into recommendations, one target file and one change apiece, each citing the observations behind it.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `opus` — naming which one file a friction point lives in, and what change to it removes the friction, is the judgment this skill exists for.
- **input**: the `observations[]` of step 6 with their `OBS-nnn` ids; the aggregate's `floors` block; the target-kind table of `## Outputs`; `templates/rec-targets.md`; the paths under `.claude/skills/`, `.claude/agents/`, and `.devforgeai/` that `Glob` returns. Those are the directories an installed project holds; the framework repository is not on disk there, so a glob over `skills/`, `agents/`, `commands/`, `hooks/`, or `specs/` returns nothing and every recommendation it could produce would name a file outside the project.
- **output**:

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

### Existing agents this spec does not carry forward

| Agent | Disposition | Reason |
|---|---|---|
| `observation-extractor.md` | adapted into `observation-miner` | Kept: the seven-to-five category narrowing, the severity triple, the silent-skip rule for absent fields, the sensitive-field filter. Dropped: the `devforgeai/feedback/ai-analysis/` write path, the `phase-state.json` schema, the per-subagent source-field mapping table, the two-digit phase numbering. Its inputs are now the aggregate's already-counted arrays, so the extraction rules it carried are the CLI's work. |
| `session-miner.md` | adapted into `session-pattern-reader` | Kept: the error-tolerant JSON Lines reading and the normalised per-entry shape. Dropped: `~/.claude/history.jsonl` as the source, the pagination contract, the `STORY-2xx` trigger list, and the observation-file write. The source is `<session_root>/<project-key>/*.jsonl`, the chunking is the aggregate's, and the agent reads the line ranges the aggregate points at. |
| `technical-debt-analyzer.md` | adapted into `debt-aggregator` | Kept: the deferred-DoD inventory and the age-in-days grouping. Dropped: the `Write` tool and the `technical-debt-analysis-{date}.md` file, per §10's read-only default and §1 rule 4, which gives Reflect one document; its three uppercase severity strings, the first of which collides with the §2 ceremony pattern; the `technical-debt-register.md` and `*.story.md` paths, which are not in the §3 layout. |
| `framework-analyst.md` | adapted into `recommendation-drafter` | Kept: the specific-actionable-feasible-non-aspirational test on each recommendation and the duplicate check. Dropped: the `recommendations-queue.json` path, the `phase-state.json` input, the `dev`/`qa` workflow parameter, and the nine-field analysis object; the fields that survive are the eight of the `recommendations[]` schema above. |
| `pattern-compliance-auditor.md` | replaced | It audits commands against `devforgeai/protocols/lean-orchestration-pattern.md`, a file the §3 layout does not hold, and it produces a refactoring roadmap with effort hours, which is an applier's output. The part worth keeping — naming one file as the target of a change — is the target-kind table, and `recommendation-drafter` fills it. |
| `agent-generator.md` | replaced | It writes `.claude/agents/*.md`. This skill writes recommendations and applies none, so an agent that generates agent files has no step to run in. A recommendation to change a subagent is a `subagent` row naming the file and the change, and the user applies it. |
| `diagnostic-analyst.md` | replaced | It reads the six context files for spec drift against the codebase, which is Constitute's `alignment-auditor` and Verify's `anti-pattern-scanner`, both of whose findings reach this skill through the reports the aggregate already read. A second pass over the same files here would produce project findings, and this skill's output is framework recommendations. |

## Command

The entry point is the skill itself: `skills/improving-framework/SKILL.md`, installed to `.claude/skills/reflect/`, whose frontmatter `name` is the slash command. There is no command file. This is the frontmatter and the preamble, verbatim, with one preamble line.

```markdown
---
name: reflect
description: Cross-cutting Reflect phase of DevForgeAI, run by /reflect. Reads the window aggregate the CLI computes over .devforgeai/reports/, the project's Claude Code session history, and the deferral records inside Verify's QA reports, then writes one .devforgeai/reports/reflect-DATE.yaml holding OBS-nnn observations, REC-nnn recommendations that each name one framework file and one change, and a technical-debt section grouped by CON-nnn or AP-nnn with ages in days. Reach for it whenever /reflect is typed, whenever a retrospective is being run over a window of stories or a release, whenever friction points, repeated send-backs, gate failures by check name, time spent per phase, unparsed subagent output, or accumulated deferred Definition-of-Done items are being looked at, and whenever OBS-nnn, REC-nnn, report aggregate, or a reflect report appears in a document, a report, or a handoff.
argument-hint: IDEA-nnn | EPIC-nnn | STORY-nnn | vX.Y.Z | --since YYYY-MM-DD
allowed-tools: Bash(devforgeai:*), PowerShell(devforgeai:*), Read, Write, Glob, Grep, Agent
disable-model-invocation: true
---

!`devforgeai report aggregate $ARGUMENTS --json`
```

The one preamble line is the whole data source: every count and every timestamp in the Reflect document comes from `report aggregate --json`. It carries no `gate require` line because Reflect has no predecessor gate, and `$ARGUMENTS` holding neither a recognised id prefix nor `--since` stops the run there, `report aggregate` exiting 3 on `DFA-E430`.

## CLI calls

| Subcommand with exact arguments | Called from | Exit handling |
|---|---|---|
| `devforgeai report aggregate $ARGUMENTS --json` | command `!` preamble | 0 puts the envelope on stdout; 3 on `DFA-E430` stops the run with the accepted forms on stderr; 1 on `DFA-E421` means the session root resolved to a path outside the user's home and the run continues with `sessions.status: unreadable` |
| `devforgeai doc validate --allocate OBS` | workflow step 6, once per observation | stdout is the next free `OBS-nnn`; 1 on `DFA-E215` stops the run |
| `devforgeai doc validate --allocate REC` | workflow step 8, once per recommendation | stdout is the next free `REC-nnn`; 1 on `DFA-E215` stops the run |
| `devforgeai gate check --phase reflect --id <date>` | workflow step 10 | 0 writes `.devforgeai/reports/<date>-reflect.yaml`; 1 names the failing check ids on stderr and step 10 reruns once after the model rewrites them |
| `devforgeai handoff --phase reflect --id <date>` | workflow step 11 | 0 prints the §6 block; 1 on `DFA-E400` means step 10 wrote no report and the run stops |
| `devforgeai report show <id> <phase>` | the user, after reading a `sources.reports[].path` line | prints the named report |

`report aggregate` and the `reflect` value of `--phase` are proposed additions to §4 and are recorded in `## Decisions`. `doc validate --allocate`, `gate check`, `handoff`, and `report show` are §4 names.

## Gate

`.devforgeai/gates.toml` gains one entry. `{id}` resolves to the gate subject id, which for this phase is the run's `<date>`, so `reports/reflect-{id}.yaml` resolves to the document `## Outputs` defines. The entry is verbatim:

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
  paths = ["reports/reflect-{id}.yaml"]
  min_count = 1

  [[gate.check]]
  kind = "doc_valid"
  id = "reflect-docs"
  docs = ["reports/reflect-{id}.yaml"]

  [[gate.check]]
  kind = "yaml_cites"
  id = "reflect-rec-cites-obs"
  doc = "reports/reflect-{id}.yaml"
  from = "recommendations"
  field = "observations"
  into = ["observations[].id"]
  min = 1

  [[gate.check]]
  kind = "yaml_cites"
  id = "reflect-obs-cites-source"
  doc = "reports/reflect-{id}.yaml"
  from = "observations"
  field = "sources"
  into = ["sources.reports[].path", "sources.sessions.files[].session_id"]
  min = 1

  [[gate.check]]
  kind = "no_threshold_decrease"
  id = "reflect-no-lowered-floor"
  doc = "reports/reflect-{id}.yaml"
  from = "recommendations"
  target_field = "target.path"
  key_field = "target.key"
  value_field = "proposed_value"
  files = ["gates.toml", "config.toml"]
```

The compiled-minimums row for the phase, in the format of `specs/01-cli.md` `### Compiled-in minimums`:

| Phase | Required check kinds | Floors |
|---|---|---|
| reflect | `file_exists`, `doc_valid`, `yaml_cites`, `no_threshold_decrease` | — |

`reflect-no-lowered-floor` is why §8's sentence "No subcommand modifies `gates.toml` thresholds downward" holds for this skill: a recommendation is prose until a human applies it, and this check stops the prose from carrying a number the binary would reject anyway. A `REC-nnn` proposing a value at or above the floor passes, so raising a threshold is a recommendation this skill can make.

`gate check --phase reflect` is called at workflow step 10 with an explicit `--id`, and by nothing else. The Stop hook runs `gate check --phase <current>`, and a `/reflect` run leaves `current_phase` at whatever phase the user was in, so the Stop hook evaluates that phase's gate and never this one. `state.toml` `[active]` gains no `reflect` key for the same reason.

## Send-back

Reflect emits no SEND BACK. §5 gives its "May send back to" column as `any (as recommendations, never as gates)`, and a recommendation is not a send-back: a send-back is a gate result that blocks a phase until the cited IDs are re-opened, and a `REC-nnn` blocks nothing. The reflect gate's `send_back_to` is `""` and its `on_fail` is `fail`, so `gate check --phase reflect` exits 0 or 1 and the exit-2 path of `specs/01-cli.md` `## Send-back` is unreachable for this phase. The one condition that would otherwise produce a send-back — a `REC-nnn` naming a defect in an upstream document — is out of scope by `## Scope`: this skill's targets are framework files under the §3 repo layout, not the `.devforgeai/` documents a phase produced.

Reflect receives no SEND BACK. No phase cites a `reflect-<date>.yaml` in a gate check, no phase's `requires` names `reflect`, and no `[[gate]]` other than this one carries `send_back_to = "reflect"`. A defect in a reflect report is corrected by running `/reflect` again for the same window, which rewrites the file at the same path.

**How the handoff still fits §6.** §6 fixes nine labels and a twelve-line cap; it does not require a `SEND BACK` case. The `Gate` line takes `PASS` or `FAIL` with no `to <Phase>` suffix, the `Found` line is omitted because §6 scopes it to SEND BACK, and the `Verified` line is omitted because no subagent in this skill is a registered verifier. `Next` and `Then` come from `state.toml`, by this table, which `devforgeai handoff --phase reflect` applies:

| Gate | `Next` | `Then` |
|---|---|---|
| PASS | `/<command for state.toml current_phase> <state.toml [active].<current_phase>>` | the command for the phase after `current_phase` in the §5 sequence, with the same id; omitted when `current_phase` is `release` |
| FAIL | `/reflect <the run's own $ARGUMENTS>` | `/<command for current_phase> <[active].<current_phase>>` |

The phase-to-command mapping is §4b's: `explore`, `discover`, `constitute`, `plan`, `build`, `verify`, `release`. With `current_phase = "build"` and `[active].build = "STORY-014"`, the PASS block reads `Next      /build STORY-014` and `Then      /verify STORY-014`: the user goes back to the phase they were in, holding a document of recommendations they may act on at any time. With `[active].<current_phase>` empty, the id is dropped and `Next` is the bare command.

## Integration

Ten rows: the nine skills of §4b and the CLI.

| Skill | Report it gives Reflect | REC kinds Reflect can raise against it | Consumes (doc, IDs) | Produces for (doc, IDs) | Sends back to (condition) | Receives send-back from (condition) | Shared subagents | `state.toml` fields read / written |
|---|---|---|---|---|---|---|---|---|
| 0 Explore · `exploring-ideas` | `.devforgeai/reports/IDEA-nnn-explore.yaml`, written by `gate check` and by the `SubagentStop` ingest of `kill-case-builder`; carries the `time-box`, `decision-exists`, and `remedy-flows-present` check results and the kill-case verifier block | `skill` on `.claude/skills/exploring-ideas/SKILL.md`; `subagent` on `.claude/agents/{idea-interrogator,landscape-scanner,flow-drafter,prototype-builder,kill-case-builder}.md`; `template` on `.claude/skills/exploring-ideas/templates/{brief.md,decision.yaml,sketch-request.json,prototype-offer.md}`; `gate_threshold` on `config.toml` `explore.timebox_days` and `explore.remedy_timebox_days`, at or above the compiled floor | the explore report's `checks[]`, `findings[]`, `started_at`, `finished_at`; `IDEA-nnn`, `FLOW-nnn` | none — Reflect writes one document and Explore reads no report | none — a `REC-nnn` is not a send-back, per `## Send-back` | none — no gate carries `send_back_to = "reflect"` | none; `session-pattern-reader` reads the `/explore ... --remedy` lines Explore's remedy runs produced, which is session text rather than a shared agent | reads `current_phase`, `[active].explore`; writes none |
| 1 Discover · `discovering-requirements` | `.devforgeai/reports/IDEA-nnn-discover.yaml`, written by `gate check`, which is what `specs/03-discover.md` `## Integration` names; carries `discover-ids`, `discover-status`, `discover-questions` results and the `SEND_BACK` result that names Explore. A `verifiers.flow_integrity` block appears when Discover registers `flow-integrity-auditor` in `config.toml` `[[verifier]]`, which its handoff `Verified` line implies and its Integration row leaves unstated; the aggregate reads the block when present and omits it when absent | `skill` on `.claude/skills/discovering-requirements/SKILL.md`; `subagent` on `.claude/agents/{persona-mapper,requirement-drafter,epic-grouper,flow-integrity-auditor}.md`; `template` on `.claude/skills/discovering-requirements/templates/{requirements.yaml,questions.md}` | the discover report's `checks[]`, `findings[]`, `gate.send_back_to`; `REQ-nnn`, `EPIC-nnn`, `PERSONA-nnn`, `FLOW-nnn` | none | none | none | none | reads `current_phase`, `[active].discover`; writes none |
| 2 Constitute · `establishing-context` | `.devforgeai/reports/IDEA-nnn-constitute.yaml`, written by `gate check` and by the ingest of `alignment-auditor` and `architecture-reviewer`; carries `constitute-audit`, `constitute-ids` results and both verifier blocks | `skill` on `.claude/skills/establishing-context/SKILL.md`; `subagent` on `.claude/agents/{architecture-reviewer,alignment-auditor,source-tree-mapper}.md`; `template` on the seven files of `.claude/skills/establishing-context/templates/` | the constitute report's `checks[]`, `verifiers[]`, `findings[]`; `CON-nnn` and `AP-nnn` as the grouping keys of `technical_debt.groups[]`, and the `## Constraint index` and `## Anti-pattern index` rows `debt-aggregator` reads for a group title | none | none | none | none | reads `current_phase`, `[active].constitute`; writes none |
| 3 Plan · `planning-work` | `.devforgeai/reports/SPRINT-nnn-plan.yaml`, written by `gate check`; carries `plan-stories`, `plan-ids`, `plan-sprint-status` results and `findings[]` with `STORY-nnn` and `UI-nnn` ids | `skill` on `.claude/skills/planning-work/SKILL.md`; `subagent` on the agents `specs/05-plan.md` names; `template` on `.claude/skills/planning-work/templates/`; `gate_threshold` on `gates.toml` `plan-stories.scope` | the plan report's `checks[]`, `findings[]`, `started_at`, `finished_at`; `SPRINT-nnn`, `STORY-nnn`, `AC-nnn` | none | none | none | none | reads `current_phase`, `[active].plan`; writes none |
| 4 Build · `implementing-stories` | `.devforgeai/reports/STORY-nnn-build.yaml`, written by `gate check` including the `--partial` PostToolUse annotations; carries `build-tests`, `build-coverage`, `build-lint`, `build-design` results with `evidence.duration_ms`, `evidence.passed`, `evidence.failed`, and the `coverage` block | `skill` on `.claude/skills/implementing-stories/SKILL.md`; `subagent` on the agents `specs/06-build.md` names; `hook` on `.claude/settings.json` at `PostToolUse` and `Stop`; `gate_threshold` on `config.toml` `layer.<name>.coverage_min` and `coverage.overall_min`, at or above the compiled floors 90.0, 80.0, 70.0, 60.0, 75.0 | the build report's `checks[]`, `coverage`, `findings[]`, `started_at`, `finished_at`; `STORY-nnn` | none | none | none | none | reads `current_phase`, `[active].build`; writes none |
| 5 Verify · `validating-quality` | two files: `.devforgeai/reports/STORY-nnn-qa.yaml`, the §5 document holding `findings[]` (`FIND-nnn`) and the `deferrals[]` records this skill's `technical_debt` section is built from; and `.devforgeai/reports/STORY-nnn-verify.yaml`, the CLI gate report holding `verify-acs`, `verify-ids`, `verify-story-status` results and the `ac-compliance-verifier` block | `skill` on `.claude/skills/validating-quality/SKILL.md`; `subagent` on the agents `specs/07-verify.md` names, including `ac-compliance-verifier` and `deferral-validator`; `hook` on `.claude/settings.json` at `SubagentStop`; `gate_threshold` on `gates.toml` `verify-acs.min_ratio`, at or above the compiled floor 1.0 | both reports' `checks[]`, `verifiers[]`, `findings[]`; the QA document's `deferrals[]` entries with `dod_item`, `deferred_at`, `reason`, and the `CON-nnn` or `AP-nnn` each cites; `FIND-nnn`, `STORY-nnn`, `AC-nnn` | none | none | none | none | reads `current_phase`, `[active].verify`; writes none |
| 6 Release · `releasing-software` | `.devforgeai/reports/vX.Y.Z-release.yaml`, written by `gate check`; carries `release-docs`, `release-file`, `release-ids`, `release-status` results. The manifest `.devforgeai/releases/vX.Y.Z.yaml` supplies the `STORY-nnn` list that bounds an `id`-mode window on a version | `skill` on `.claude/skills/releasing-software/SKILL.md`; `subagent` on the agents `specs/09-release.md` names; `template` on `.claude/skills/releasing-software/templates/` | the release report's `checks[]`, `findings[]`; the manifest's `STORY-nnn` list | none | none | none | none | reads `current_phase`, `[active].release`; writes none |
| Design · `designing-interfaces` | `.devforgeai/reports/UI-nnn-design.yaml`, written by the ingest of `requirement-coverage-auditor`; carries `screens_without_req`, `flows_without_req`, `covered`, `total`, and a `status` of `send_back` when Design held a screen back | `skill` on `.claude/skills/designing-interfaces/SKILL.md`; `subagent` on `.claude/agents/{mockup-designer,brand-designer,requirement-coverage-auditor,ui-spec-writer}.md`; `template` on `.claude/skills/designing-interfaces/templates/`; `framework_file` on `specs/08-design.md` | the design report's `verifiers[]` and `findings[]`; `UI-nnn`, `FLOW-nnn` | none | none | none | none | reads `current_phase`; writes none |
| Reflect · `improving-framework` | self — `.devforgeai/reports/<date>-reflect.yaml` from `gate check`, alongside the document `reflect-<date>.yaml`. A later run reads an earlier `<date>-reflect.yaml` as one more window report | self — `skill`, `subagent`, `template`, and `framework_file` targets naming this skill's own four agents, `.claude/skills/improving-framework/SKILL.md`, and the files under `.claude/skills/improving-framework/templates/` | self | self | self | self | self — `observation-miner`, `session-pattern-reader`, `debt-aggregator`, `recommendation-drafter`, each invoked by this skill alone | reads `current_phase`, `[active]`, `[last_gate]`, `[last_handoff]`; writes none |
| CLI · `devforgeai` | every report above, plus `state.toml` `[last_gate]` and `[last_handoff]`, delivered as the one aggregate object `report aggregate` prints | `framework_file` on `specs/01-cli.md` and on paths under `cli/src/`; `hook` on `.claude/settings.json`; `gate_threshold` on `.devforgeai/gates.toml` and `.devforgeai/config.toml`, at or above the compiled floors. A `REC-nnn` against the CLI names a file and a change; the binary is rebuilt and re-pinned by a human, per §8 | `config.toml` keys `[reflect].session_root`, `[reflect].session_key`, `[reflect].window_days`, `[[layer]].coverage_min`, `[coverage].overall_min`; `gates.toml` `verifier_pass.min_ratio`; `state.toml` `current_phase`, `[active]`, `[last_gate]`, `[last_handoff]` | `.devforgeai/reports/reflect-<date>.yaml` for `doc validate`, `gate check --phase reflect`, and `handoff --phase reflect` | none — `gate check --phase reflect` exits 0 or 1, and this phase's `send_back_to` is `""` | none — the CLI issues send-backs and receives none | none — the CLI invokes no subagent | reads every key named above; writes `[last_gate]` and `[last_handoff]` through `gate check` and `handoff`, and no `[active]` key, because no `phase set` runs |

## Handoff

Both blocks are rendered by `devforgeai handoff --phase reflect --id <date>`. Labels occupy characters 1 to 10, content starts at character 11. The `Verified` line is omitted in both, per §6, because no subagent of this skill is a registered verifier. The `Found` line is omitted in both, per §6, which scopes it to SEND BACK, and this skill has no SEND BACK case.

PASS, nine lines:

```
Phase     — · Reflect       2026-09-11 · sprint-1-window
Done      14 OBS · 9 REC · 6 debt items · 11 reports · 3 sessions
Gate      PASS  9/9 REC cite an OBS · 14/14 OBS cite a source · 0 lowered floors

Next      /build STORY-014
Then      /verify STORY-014
Blocked   none

Full report: .devforgeai/reports/reflect-2026-09-11.yaml
```

FAIL, nine lines. This is the second example: §5 gives Reflect no send-back target, so `FAIL` is the only non-`PASS` result the gate produces, and `Next` re-runs the same window while `Then` returns the user to the phase they were in:

```
Phase     — · Reflect       2026-09-11 · sprint-1-window
Done      14 OBS · 9 REC · 6 debt items · 11 reports · 0 sessions
Gate      FAIL  reflect-rec-cites-obs REC-004 · reflect-no-lowered-floor REC-007

Next      /reflect --since 2026-08-28
Then      /build STORY-014
Blocked   you: is domain coverage 60 a deliberate ask?

Full report: .devforgeai/reports/reflect-2026-09-11.yaml
```

The `Phase` line renders `<n>` as an em dash, on the precedent `specs/08-design.md` Decision 5 set: §5 gives this skill no number in the phase sequence, and the em dash keeps the separator, the column-11 start, and the twelve-line cap. The slug is the `window.mode` and its subject: `sprint-1-window` in `since` mode from the `[active].plan` value covering the window, and the subject id's own slug in `id` mode.

## Templates

Two files under `skills/improving-framework/templates/`.

### `templates/reflect-report.yaml`

```yaml
schema: devforgeai/reflect-report/1
id: <YYYY-MM-DD>
phase: reflect
status: final
produced_by: improving-framework
consumes: []
open_questions: []
window:
  mode: <id|since>
  subject: <IDEA-nnn|EPIC-nnn|STORY-nnn|vX.Y.Z|>
  from: <YYYY-MM-DD>
  to: <YYYY-MM-DD>
  generated_at: <RFC 3339 UTC>
  cli_version: <semver>
sources:
  reports: []
  sessions:
    status: <present|absent|empty|unreadable>
    root: <path>
    key: <project-key>
    reason: ""
    files: []
  state:
    current_phase: <phase>
    active_id: <id>
    last_gate_result: <PASS|FAIL|SEND_BACK|TRUST_FAIL|NOT_RUN>
observations: []
recommendations: []
technical_debt:
  as_of: <YYYY-MM-DD>
  total: 0
  oldest_days: 0
  groups: []
```

One `observations[]` entry, one `recommendations[]` entry, and one `technical_debt.groups[]` entry, as the shapes to copy:

```yaml
observations:
  - id: OBS-000
    kind: <friction|repeated_send_back|gate_failure|phase_time|verifier_unparsed>
    severity: <low|medium|high>
    phase: <phase|>
    summary: <1 to 120 chars>
    detail: <1 to 400 chars>
    count: 1
    metric: ""
    sources: []
    evidence:
      - path: <path>
        line: 0
        at: <RFC 3339 UTC>
recommendations:
  - id: REC-000
    observations: [OBS-000]
    target:
      kind: <skill|subagent|command|template|hook|gate_threshold|framework_file>
      path: <path>
      key: ""
    change: <1 to 300 chars, imperative, one change>
    current_value: ""
    proposed_value: ""
    effort: <small|medium|large>
    applies_to: []
technical_debt:
  groups:
    - constraint: <CON-nnn|AP-nnn|none>
      kind: <constraint|anti_pattern|none>
      count: 1
      oldest_days: 0
      items:
        - story: STORY-000
          dod_item: <1 to 120 chars>
          deferred_at: <YYYY-MM-DD>
          age_days: 0
          reason: <1 to 200 chars>
          report: <path>
```

### `templates/rec-targets.md`

```markdown
# Recommendation targets

One recommendation names one target. The six kinds below are the whole set, and every
path is a path the installed project holds: `.claude/` carries the skills, the agents
and the hook block, `.devforgeai/` carries the two configuration files. The repository
the framework is built from is not on disk in a target project, so no recommendation
names a path under it.

| kind | path | key | What a change to it does |
|---|---|---|---|
| skill | `.claude/skills/<skill-name>/SKILL.md` | `""` | Changes the workflow, the judgment, or the wording one phase applies |
| subagent | `.claude/agents/<agent-name>.md` | `""` | Changes one agent's purpose, tools, model, input, or output schema |
| template | `.claude/skills/<skill-name>/templates/<file>` | `""` | Changes the document shape one phase writes |
| hook | `.claude/settings.json` | the hook event name | Changes when the CLI runs, or with which matcher |
| gate_threshold | `.devforgeai/gates.toml` or `.devforgeai/config.toml` | a dotted key | Changes a number a gate reads |
| framework_file | an installed path none of the five rows above names, e.g. `.claude/skills/<name>/references/<file>` | `""` | Changes a file none of the other five rows names |

The rows are ordered by specificity and a target takes the first row it matches. A recommendation whose `target.path` is `.devforgeai/gates.toml` or `.devforgeai/config.toml` takes `kind: gate_threshold` with the dotted key in `target.key`, whatever the change is; `framework_file` names neither of those two files.

## The floors

A `gate_threshold` recommendation carries `current_value` and `proposed_value` as strings. These keys have a compiled floor; a proposal below the floor is dropped before the report is written, and the drop is recorded in `open_questions`.

| File | Key | Floor |
|---|---|---|
| `config.toml` | `layer.domain.coverage_min` | 90.0 |
| `config.toml` | `layer.application.coverage_min` | 80.0 |
| `config.toml` | `layer.infrastructure.coverage_min` | 70.0 |
| `config.toml` | `layer.interface.coverage_min` | 60.0 |
| `config.toml` | `coverage.overall_min` | 75.0 |
| `gates.toml` | `verifier_pass.min_ratio` | 1.0 |

## The nine skill names in a path

`exploring-ideas`, `discovering-requirements`, `establishing-context`, `planning-work`, `implementing-stories`, `validating-quality`, `releasing-software`, `designing-interfaces`, `improving-framework`.

A skill's directory name is the one above; its slash command is its frontmatter `name`
(`explore`, `discover`, `constitute`, `plan`, `build`, `verify`, `release`, `design`,
`reflect`). A `skill` or `template` path uses the directory name.
```

## Evals

Shipped at `skills/improving-framework/evals/`.

### `evals/evals.json` — 7 entries, skill-creator format

| # | `prompt` | `expected_output` | `expectations[]` |
|---|---|---|---|
| 1 | `/reflect --since 2026-08-28` with eleven reports under `.devforgeai/reports/` and `[reflect].session_root` naming an absent directory | `.devforgeai/reports/reflect-<date>.yaml` at `status: final` | the twelve top-level keys appear in `## Outputs` order; `sources.sessions.status` is `absent` and `sources.sessions.files` is `[]`; every `observations[].sources` entry is a path that exists under the workspace; the `Gate` line reads `PASS` |
| 2 | `/reflect STORY-014` with a build report and a verify report for that story | a report whose every `REC-nnn` cites an `OBS-nnn` defined in the same file | every `recommendations[].observations` list has one or more entries; every entry matches an `observations[].id` in the same document; every `recommendations[].target.kind` is one of the six; no `target.path` is under `.devforgeai/` unless `target.kind` is `gate_threshold` |
| 3 | `/reflect --since 2026-08-20` with a session JSONL holding three `/explore IDEA-001 --remedy FLOW-002` lines across two sessions | an `OBS-nnn` of kind `repeated_send_back` naming the Explore remedy loop | one `observations[]` entry has `kind: repeated_send_back` and `count: 3`; its `summary` names `FLOW-002`; its `sources` holds both session ids; its `evidence` holds three entries whose `line` values are the three line numbers; `sources.sessions.status` is `present` |
| 4 | `/reflect STORY-014` with a session JSONL holding four `/build STORY-014 --resume` lines after three `verify-acs` failures | an `OBS-nnn` of kind `repeated_send_back` naming the Verify-to-Build loop, and a `REC-nnn` citing it | one `observations[]` entry has `kind: repeated_send_back`, `phase: verify`, and `count: 3`; its `summary` names `STORY-014`; at least one `recommendations[]` entry lists that `OBS-nnn`; that recommendation's `target.kind` is `subagent` or `skill` |
| 5 | `/reflect --since 2026-08-01` with three QA reports carrying five deferral records citing `CON-002`, `AP-004`, and none | a `technical_debt` section grouped by constraint with ages in days | `technical_debt.groups` holds three entries with `constraint` values `AP-004`, `CON-002`, `none` in that order; every `items[].age_days` equals `as_of` minus `deferred_at` in calendar days; `technical_debt.total` is 5; `oldest_days` equals the largest `items[].age_days` |
| 6 | `/reflect --since 2026-08-28` with a build report whose coverage failed at 61 percent on the domain layer | a report with no `REC-nnn` proposing a domain coverage below 90.0 | no `recommendations[]` entry has `target.key` of `layer.domain.coverage_min` with a `proposed_value` below 90.0; a `gate_threshold` recommendation, when present, has a `proposed_value` at or above its floor; the `Gate` line reads `PASS` |
| 7 | `/reflect --since 2026-08-28` on a project whose current phase is `build` and whose `[active].build` is `STORY-014` | a PASS handoff whose `Next` returns the user to Build | the `Gate` line reads `PASS`; the `Next` line is `/build STORY-014`; the `Then` line is `/verify STORY-014`; no line of the block starts with `Found` or `Verified`; the block is at most twelve lines |

§9 asks for at least two cases on the SEND BACK path. This skill emits and receives none, per `## Send-back`, so entries 3 and 4 take that slot: each feeds a session history holding a repeated send-back produced by another phase and asserts the `OBS-nnn` that names the pattern. The substitution is recorded in `## Decisions`.

### `evals/cases.jsonl` — 7 lines

```json
{"id": "rf-01-no-sessions", "prompt": "/reflect --since 2026-08-28", "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-rust-reflect.toml", ".devforgeai/gates.toml": "FIXTURE:gates-with-reflect.toml", ".devforgeai/state.toml": "FIXTURE:state-build.toml", ".devforgeai/reports/STORY-014-build.yaml": "FIXTURE:report-build-pass.yaml", ".devforgeai/reports/STORY-014-verify.yaml": "FIXTURE:report-verify-fail.yaml", ".devforgeai/reports/SPRINT-001-plan.yaml": "FIXTURE:report-plan-pass.yaml", ".sessions/.gitkeep": ""}}, "expect": {"grader": "reflect_report_shape", "args": {"dir": ".devforgeai/reports", "keys": ["schema", "id", "phase", "status", "produced_by", "consumes", "open_questions", "window", "sources", "observations", "recommendations", "technical_debt"], "status": "final", "sessions_status": "absent", "obs_kinds": ["friction", "repeated_send_back", "gate_failure", "phase_time", "verifier_unparsed"], "gate": "PASS"}}}
{"id": "rf-02-rec-cites-obs", "prompt": "/reflect STORY-014", "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-rust-reflect.toml", ".devforgeai/gates.toml": "FIXTURE:gates-with-reflect.toml", ".devforgeai/state.toml": "FIXTURE:state-build.toml", ".devforgeai/reports/STORY-014-build.yaml": "FIXTURE:report-build-pass.yaml", ".devforgeai/reports/STORY-014-verify.yaml": "FIXTURE:report-verify-fail.yaml", ".devforgeai/reports/STORY-014-qa.yaml": "FIXTURE:report-qa-deferrals.yaml", ".sessions/.gitkeep": ""}}, "expect": {"grader": "rec_cites_obs", "args": {"dir": ".devforgeai/reports", "target_kinds": ["skill", "subagent", "command", "template", "hook", "gate_threshold", "framework_file"], "threshold_prefixes": [".devforgeai/gates.toml", ".devforgeai/config.toml"], "min_obs_per_rec": 1}}}
{"id": "rf-03-repeat-explore-remedy", "prompt": "/reflect --since 2026-08-20", "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-rust-reflect-sessions.toml", ".devforgeai/gates.toml": "FIXTURE:gates-with-reflect.toml", ".devforgeai/state.toml": "FIXTURE:state-discover.toml", ".devforgeai/reports/IDEA-001-explore.yaml": "FIXTURE:report-explore-pass.yaml", ".devforgeai/reports/IDEA-001-discover.yaml": "FIXTURE:report-discover-sendback.yaml", ".sessions/fixture/11111111-1111-4111-8111-111111111111.jsonl": "FIXTURE:session-explore-remedy-a.jsonl", ".sessions/fixture/22222222-2222-4222-8222-222222222222.jsonl": "FIXTURE:session-explore-remedy-b.jsonl"}}, "expect": {"grader": "obs_names_pattern", "args": {"dir": ".devforgeai/reports", "kind": "repeated_send_back", "count": 3, "summary_contains": ["FLOW-002"], "session_ids": ["11111111-1111-4111-8111-111111111111", "22222222-2222-4222-8222-222222222222"], "evidence_lines": [7, 19, 31], "sessions_status": "present"}}}
{"id": "rf-04-repeat-verify-build", "prompt": "/reflect STORY-014", "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-rust-reflect-sessions.toml", ".devforgeai/gates.toml": "FIXTURE:gates-with-reflect.toml", ".devforgeai/state.toml": "FIXTURE:state-build.toml", ".devforgeai/reports/STORY-014-build.yaml": "FIXTURE:report-build-pass.yaml", ".devforgeai/reports/STORY-014-verify.yaml": "FIXTURE:report-verify-fail-x3.yaml", ".sessions/fixture/33333333-3333-4333-8333-333333333333.jsonl": "FIXTURE:session-verify-build-loop.jsonl"}}, "expect": {"grader": "obs_names_pattern", "args": {"dir": ".devforgeai/reports", "kind": "repeated_send_back", "count": 3, "phase": "verify", "summary_contains": ["STORY-014"], "session_ids": ["33333333-3333-4333-8333-333333333333"], "evidence_lines": [12, 28, 44], "sessions_status": "present", "cited_by_rec": true, "rec_target_kinds": ["subagent", "skill", "command"]}}}
{"id": "rf-05-debt-groups", "prompt": "/reflect --since 2026-08-01", "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-rust-reflect.toml", ".devforgeai/gates.toml": "FIXTURE:gates-with-reflect.toml", ".devforgeai/state.toml": "FIXTURE:state-verify.toml", ".devforgeai/context/architecture-constraints.md": "FIXTURE:architecture-constraints.md", ".devforgeai/context/anti-patterns.md": "FIXTURE:anti-patterns.md", ".devforgeai/reports/STORY-009-qa.yaml": "FIXTURE:report-qa-deferrals-con002.yaml", ".devforgeai/reports/STORY-011-qa.yaml": "FIXTURE:report-qa-deferrals-ap004.yaml", ".devforgeai/reports/STORY-014-qa.yaml": "FIXTURE:report-qa-deferrals-none.yaml", ".sessions/.gitkeep": ""}}, "expect": {"grader": "debt_groups", "args": {"dir": ".devforgeai/reports", "as_of": "2026-09-11", "expected_groups": [["AP-004", "anti_pattern", 1], ["CON-002", "constraint", 3], ["none", "none", 1]], "total": 5, "oldest_days": 41, "deferred_at": {"STORY-009": "2026-08-01", "STORY-011": "2026-08-14", "STORY-014": "2026-09-02"}}}}
{"id": "rf-06-no-lowered-floor", "prompt": "/reflect --since 2026-08-28", "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-rust-reflect.toml", ".devforgeai/gates.toml": "FIXTURE:gates-with-reflect.toml", ".devforgeai/state.toml": "FIXTURE:state-build.toml", ".devforgeai/reports/STORY-014-build.yaml": "FIXTURE:report-build-coverage-61.yaml", ".sessions/.gitkeep": ""}}, "expect": {"grader": "no_lowered_floor", "args": {"dir": ".devforgeai/reports", "floors": {"layer.domain.coverage_min": 90.0, "layer.application.coverage_min": 80.0, "layer.infrastructure.coverage_min": 70.0, "layer.interface.coverage_min": 60.0, "coverage.overall_min": 75.0, "verifier_pass.min_ratio": 1.0}, "gate": "PASS"}}}
{"id": "rf-07-handoff-returns", "prompt": "/reflect --since 2026-08-28", "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-rust-reflect.toml", ".devforgeai/gates.toml": "FIXTURE:gates-with-reflect.toml", ".devforgeai/state.toml": "FIXTURE:state-build.toml", ".devforgeai/reports/STORY-014-build.yaml": "FIXTURE:report-build-pass.yaml", ".devforgeai/reports/SPRINT-001-plan.yaml": "FIXTURE:report-plan-pass.yaml", ".sessions/.gitkeep": ""}}, "expect": {"grader": "handoff_returns", "args": {"gate": "PASS", "next": "/build STORY-014", "then": "/verify STORY-014", "absent_labels": ["Found", "Verified"], "max_lines": 12, "requires_reflect_document": true}}}
```

Fixture files named `FIXTURE:<name>` live at `skills/improving-framework/evals/fixtures/<name>`, and the shared runner copies them in as it copies any `setup.files` entry. `config-rust-reflect.toml` sets `[reflect] session_root = ".sessions"` and `session_key = "fixture"` with no `.sessions/` directory present, so `sessions.status` resolves to `absent`; `config-rust-reflect-sessions.toml` sets the same two keys with the fixture JSONL files under `.sessions/fixture/`. Prior state a grader needs — the line numbers in each fixture JSONL, the `deferred_at` date per story, the compiled floors, and the expected group order — travels in `expect.args` and is not read back from the workspace. `fixtures/digests.txt` records the line numbers each session fixture carries and how they were counted.

### `evals/graders.py` — signatures and logic

Every function has the §9 signature `def <name>(workspace: str, transcript: str, args: dict) -> tuple[bool, str]`, reads only files under `workspace` and the `transcript` string, and returns `(passed, evidence)`. A helper `_load(workspace, args)` globs `args["dir"]` for `reflect-[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9].yaml`, requires exactly one match, and parses it with the standard library's YAML-free reader described below; the fixtures and the written document use block mappings and block sequences only, so a 90-line recursive-descent reader in `graders.py` covers them and keeps the module free of a third-party import.

- `reflect_report_shape(workspace, transcript, args)` — load the document. Compare `list(obj.keys())` to `args["keys"]` for equality including order. Check `obj["status"] == args["status"]` and `obj["sources"]["sessions"]["status"] == args["sessions_status"]`. When that status is not `present`, require `sources.sessions.files == []`. Require every `observations[].kind` to be in `args["obs_kinds"]`, every `observations[].severity` to be in `{"low","medium","high"}`, and every `observations[].sources` entry that starts with `.devforgeai/` to exist under `workspace`. Scan `transcript` for a line starting `Gate      ` and require its next word to equal `args["gate"]`. Evidence: the key order found, the sessions status, and the observation kinds counted.
- `rec_cites_obs(workspace, transcript, args)` — load the document. Build `ids = {o["id"] for o in obj["observations"]}`. For each `recommendations[]` entry: require `len(entry["observations"]) >= args["min_obs_per_rec"]`, require every listed id to be in `ids`, require `entry["target"]["kind"]` to be in `args["target_kinds"]`, and require `entry["target"]["path"]` to start with one of `args["threshold_prefixes"]` when and only when the kind is `gate_threshold`. Require every `observations[].sources` list to be non-empty. Evidence: the REC-to-OBS map and any unresolved id.
- `obs_names_pattern(workspace, transcript, args)` — load the document. Select the `observations[]` entries whose `kind` equals `args["kind"]` and, when `args` carries `phase`, whose `phase` matches. Require at least one. Take the entry with the largest `count` and require `count == args["count"]`, require every string in `args["summary_contains"]` to appear in its `summary`, require `set(args["session_ids"]) <= set(entry["sources"])`, and require `sorted(e["line"] for e in entry["evidence"]) == sorted(args["evidence_lines"])`. Check `sources.sessions.status == args["sessions_status"]`. When `args.get("cited_by_rec")` is true, require at least one `recommendations[]` entry listing that observation's id and having a `target.kind` in `args["rec_target_kinds"]`. Evidence: the matched observation's id, count, summary, and evidence lines.
- `debt_groups(workspace, transcript, args)` — load the document, read `technical_debt`. Compare `[(g["constraint"], g["kind"], g["count"]) for g in groups]` to `args["expected_groups"]` for equality including order. Check `total == args["total"]` and `oldest_days == args["oldest_days"]`. For each `items[]` row, compute the calendar-day difference between `args["as_of"]` and `args["deferred_at"][row["story"]]` with `datetime.date.fromisoformat` and require it to equal `row["age_days"]`, and require `row["deferred_at"]` to equal the recorded date. Evidence: the group tuples found and each computed age beside the reported one.
- `no_lowered_floor(workspace, transcript, args)` — load the document. For each `recommendations[]` entry whose `target["kind"]` is `gate_threshold`: look `target["key"]` up in `args["floors"]`; when it is present, parse `proposed_value` as a float and require it to be at or above the floor. Require no `open_questions` line to name a key of `args["floors"]` with a value at or above its floor, since only a dropped proposal is written there. Scan `transcript` for a `Gate      ` line and require its next word to equal `args["gate"]`. Evidence: each threshold recommendation with its key, proposed value, and floor.
- `handoff_returns(workspace, transcript, args)` — find the last block in `transcript` whose first line starts `Phase     `. Require it to hold at most `args["max_lines"]` lines up to and including the `Full report:` line. Require the `Gate      ` line's next word to equal `args["gate"]`, the `Next      ` line's text to equal `args["next"]`, and the `Then      ` line's text to equal `args["then"]`, each after stripping trailing spaces. Require no line of the block to start with any label in `args["absent_labels"]`. Evidence: the block as found and its line count.

No grader opens a network connection, starts a subprocess, calls a model, or uses a random source, and none depends on a runner behaviour beyond the §9 contract: the runner copies `setup.files` into a temp workspace, invokes `claude -p`, and calls the grader with `workspace`, `transcript`, and `args`. `python -c "import graders"` needs only the standard library.

## Decisions

Each entry is a choice this spec made where §1 through §10 were silent, a proposed addition to the §4 surface, a dependency on a spec being written in parallel, or a blocker.

### Proposed additions to the CLI surface

1. **`reflect` is added to the `[[gate]].phase` enum, with a compiled-minimums row and no floors.** `specs/01-cli.md` fixes that enum at seven values and gives a required-kind row per phase. The task fixes a `gates.toml` entry for this skill, so the enum takes an eighth value. Exact effect: `gates.toml` may hold one `[[gate]]` with `phase = "reflect"`, `requires = ""`, `on_fail = "fail"`, `send_back_to = ""`; the compiled table gains the row `reflect | file_exists, doc_valid, yaml_cites, no_threshold_decrease | —`; `gate check --phase reflect --id <date>` and `gate require reflect <date>` both resolve, the second trivially, since an empty `requires` has nothing to test. `state.toml` `[active]` gains no `reflect` key, so `gate check --phase reflect` without `--id` is `DFA-E011`, exit 3, with the message `phase reflect has no active id; pass --id <YYYY-MM-DD>`. The code sits at E324 with the three of Decisions 5 and 6, above the E300 to E316 range `specs/01-cli.md` already uses. This is the branch `specs/08-design.md` Decision 3 declined for Design, and the reason the two differ is that Design's outputs are gated at PreToolUse by `design lint` while this skill's output is gated nowhere else.

2. **`report aggregate` is added.** Exact grammar:

   ```
   devforgeai report aggregate [<ID>] [--since <YYYY-MM-DD>] [--session-root <path>] [--json] [--project <path>] [--quiet]
   ```

   `<ID>` matches `^(IDEA|EPIC|STORY)-[0-9]{3}$` or `^v[0-9]+\.[0-9]+\.[0-9]+$`. Exactly one of `<ID>` and `--since` is given; neither or both is `DFA-E430`, exit 3, with the message `pass one id (IDEA-nnn, EPIC-nnn, STORY-nnn, vX.Y.Z) or --since <YYYY-MM-DD>`. With `<ID>` the window holds every report under `.devforgeai/reports/` whose `id` equals `<ID>`, plus, for a version, every report whose `id` is a `STORY-nnn` in that version's `.devforgeai/releases/<ID>.yaml`, plus, for an `EPIC-nnn`, every report whose `id` is a `STORY-nnn` whose story frontmatter `consumes` holds that epic. With `--since` the window holds every report whose `finished_at` is at or after the date at 00:00:00Z. `--session-root` overrides `[reflect].session_root` for one call. Exit codes: 0 with the envelope on stdout; 1 on `DFA-E420` (a report under `.devforgeai/reports/` does not parse; the entry is skipped and the code is a warning when at least one report parsed, an error when none did) and `DFA-E421` (the resolved session root is outside the user's home directory; `sessions.status` is `unreadable` and the rest of the aggregate is produced); 3 on `DFA-E430`, `DFA-E012`, `DFA-E013`; 5 on `DFA-E9xx`. The `--json` `data` object is the schema in Decision 4. Human output is one line per window report and one summary line. §1 rule 1 puts this in the CLI: counting reports, differencing timestamps, and tallying check ids is arithmetic, and a skill performing it would be the ceremony §2 forbids.

3. **`[reflect]` is added to `config.toml`, with three keys.** Exact content and defaults, in the key order `init` writes:

   ```toml
   [reflect]
   session_root = "~/.claude/projects"   # string; default as shown; ~ expands to the user's home directory
   session_key = ""                      # string; default ""; "" derives the key from the project path
   window_days = 14                      # integer; default 14; the --since default when the flag carries no date
   ```

   `session_root` is a path override that works with default cargo features, which matters because `DEVFORGEAI_HOME` sits behind the `test-home` feature that `specs/01-cli.md` Decision 38 deliberately excludes from the released binary's digest. `session_key` exists because an eval workspace is a temp directory whose derived key is not knowable when the case file is written. `stack detect` leaves all three at their previous values when `config.toml` exists, on the same rule that governs `[[layer]]`, `[coverage]`, and `[[verifier]]`.

4. **The `report aggregate --json` `data` schema.** Every key is present on every run; an empty result is an empty array or a zero, not an absent key.

   ```json
   { "schema": "devforgeai/aggregate/1",
     "window": { "mode": "id|since", "subject": "STORY-014", "from": "2026-08-28", "to": "2026-09-11",
                 "generated_at": "2026-09-11T09:14:02Z", "cli_version": "1.0.0", "degraded": false },
     "reports": [ { "path": ".devforgeai/reports/STORY-014-build.yaml", "id": "STORY-014", "phase": "build",
                    "status": "pass", "started_at": "2026-09-03T10:58:11Z", "finished_at": "2026-09-03T11:02:41Z",
                    "duration_ms": 270000,
                    "gate": { "result": "PASS", "send_back_to": "", "failed_checks": [] },
                    "checks": [ { "id": "build-tests", "kind": "tests_pass", "status": "pass",
                                  "severity": "block", "reason": "", "duration_ms": 41203 } ],
                    "verifiers": [ { "subagent": "ac-compliance-verifier", "status": "ingested",
                                     "passed": 7, "total": 7, "unit": "ACs", "code": "" } ],
                    "findings": [ { "id": "FIND-003", "severity": "block", "summary": "..." } ],
                    "coverage": { "overall": 87.4, "layers": [ { "name": "domain", "percent": 96.6,
                                                                "min": 95.0, "status": "pass" } ] } } ],
     "phase_time": [ { "phase": "build", "runs": 4, "total_ms": 412000, "median_ms": 98000,
                       "first_at": "2026-08-29T08:11:00Z", "last_at": "2026-09-03T11:02:41Z" } ],
     "gate_failures": [ { "check_id": "build-coverage", "kind": "coverage_min", "phase": "build",
                          "count": 3, "ids": ["STORY-014"], "reasons": ["DFA-E312 domain 61.0 < 95.0"] } ],
     "send_backs": [ { "from": "verify", "to": "build", "id": "STORY-014", "count": 3,
                       "at": ["2026-09-03T11:41:09Z"], "check_ids": ["verify-acs"],
                       "finding_ids": ["FIND-003"] } ],
     "verifier_failures": [ { "subagent": "anti-pattern-scanner", "status": "unparsed", "code": "DFA-E410",
                              "report": ".devforgeai/reports/STORY-012-verify.yaml", "count": 2 } ],
     "deferrals": [ { "story": "STORY-009", "dod_item": "Integration test for the retry path",
                      "deferred_at": "2026-08-01", "age_days": 41, "constraint": "CON-002",
                      "reason": "Retry backoff is unspecified until ADR-011 is accepted",
                      "report": ".devforgeai/reports/STORY-009-qa.yaml" } ],
     "sessions": { "status": "present|absent|empty|unreadable", "root": "...", "key": "C--Projects-DevForgeAI",
                   "reason": "",
                   "files": [ { "session_id": "...", "path": "...", "from": "...", "to": "...", "lines": 812 } ],
                   "commands": [ { "command": "/explore", "args": "IDEA-001 --remedy FLOW-002",
                                   "at": "2026-09-01T09:12:00Z", "session_id": "...", "line": 7,
                                   "remedy_ids": ["FLOW-002"], "resume": false } ],
                   "repeats": [ { "command": "/explore", "key": "IDEA-001|FLOW-002", "count": 3,
                                  "session_ids": ["...", "..."], "lines": [7, 19, 31] } ] },
     "state": { "current_phase": "build", "active": { "build": "STORY-014" },
                "last_gate": { "phase": "build", "id": "STORY-014", "result": "PASS",
                               "at": "2026-09-03T11:02:41Z", "failed_checks": [] },
                "last_handoff_at": "2026-09-03T11:02:42Z" },
     "floors": { "config.toml": { "layer.domain.coverage_min": 90.0, "layer.application.coverage_min": 80.0,
                                  "layer.infrastructure.coverage_min": 70.0, "layer.interface.coverage_min": 60.0,
                                  "coverage.overall_min": 75.0 },
                 "gates.toml": { "verifier_pass.min_ratio": 1.0 } },
     "counts": { "reports": 11, "sessions": 3, "gate_failures": 7, "send_backs": 4,
                 "deferrals": 6, "verifier_failures": 1 } }
   ```

   `sessions.commands[]` is built by matching each session line's `message.content`, when `type` is `user` and `isMeta` is absent or false, against `^/(explore|discover|constitute|plan|build|verify|release|design|reflect)\b`. `sessions.repeats[]` groups those by `command` and `key`, where `key` is the first argument joined to the sorted `--remedy` ids by `|`, and holds only groups whose `count` is two or more.

5. **`yaml_cites` is added to the check-kind enum, a thirteenth value.** Row, in the format of `specs/01-cli.md`'s kind table:

   | `kind` | Keys, types, defaults | Passes when |
   |---|---|---|
   | `yaml_cites` | `doc` string required, a §5 document name under `.devforgeai/`; `from` string required, a path to a sequence; `field` string required, a key inside each item of that sequence holding an array of strings; `into` array[string] required, each a path resolving to a set of strings; `min` integer, default 1 | every item of `from` has at least `min` entries in `field`, and every entry is a member of the union of the sets `into` resolves to |

   Path grammar for `from` and `into`: dot-separated segments, where `[]` after a segment means "each item of that sequence" and a trailing segment after `[]` names a key inside each item. `observations[].id` resolves to the set of `id` values of the `observations` sequence; `sources.reports[].path` to the set of `path` values under `sources.reports`. A path whose every segment exists in the document resolves, and an empty sequence at a `[]` segment resolves to the empty set, so `sources.sessions.files[].session_id` over `files: []` contributes nothing to the union and raises no error. A path one of whose segments is absent from the document is `DFA-E345`, exit 1, message `gates.toml check '<id>': path '<path>' does not resolve in <doc>`. A `from` path resolving to an empty sequence passes the check, because a document with no recommendations cites nothing and violates nothing. An item missing `field`, or holding fewer than `min` entries, is `DFA-E346`, exit 1, message `<doc>: <from>[<n>] id '<id>' cites <count> of <min> required`. An entry outside the union is `DFA-E347`, exit 1, message `<doc>: <from>[<n>] id '<id>' cites '<value>', which <into> does not define`.

6. **`no_threshold_decrease` is added to the check-kind enum, a fourteenth value.** Row:

   | `kind` | Keys, types, defaults | Passes when |
   |---|---|---|
   | `no_threshold_decrease` | `doc` string required; `from` string, default `recommendations`; `target_field` string, default `target.path`; `key_field` string, default `target.key`; `value_field` string, default `proposed_value`; `files` array[string], default `["gates.toml", "config.toml"]` | no item of `from` whose `target_field` ends with one of `files` and whose `key_field` names a key in the binary's compiled floor table carries a `value_field` that parses as a number below that floor |

   An item naming a floor key with a lower value is `DFA-E348`, exit 1, message `<doc>: <from>[<n>] id '<id>' proposes <key> = <value>, below the compiled floor <floor>`. An item whose `value_field` does not parse as a number, while its `key_field` names a floor key, is the same error with `<value>` printed verbatim. An item whose `target_field` names neither file, or whose `key_field` is `""`, passes without inspection. This is §8's "No subcommand modifies `gates.toml` thresholds downward" applied one step earlier, to the document that proposes the change.

7. **`reflect` is accepted as a `--phase` value by `handoff`, `report show`, `doc validate`, and the report path `reports/<date>-reflect.yaml`, and by `gate check` and `gate require` per Decision 1.** `handoff --phase reflect --id <date>` renders the block of `## Handoff`, with the `Next` and `Then` transition table of `## Send-back`. `report show <date> reflect` prints the CLI gate report.

8. **`reflect-report` is the doc type, `devforgeai/reflect-report/1` the schema, and its `status` enum is `draft | final`.** `specs/01-cli.md` Decision 44 already names `reflect-report` as one of four doc types permitted top-level keys beyond the §5 seven, which is what the five payload keys rely on. The status enum is two values: a third value for a report whose gate failed would carry one fact the gate report already carries.

9. **`doc validate` accepts a date-shaped `id` for the `reflect-report` doc type alone.** Every other doc type's `id` matches `^[A-Z]+-[0-9]{3}$`. This one matches `^[0-9]{4}-[0-9]{2}-[0-9]{2}$`, because the document is one per date rather than one per allocated ID, and §5 gives Reflect only `OBS-nnn` and `REC-nnn`, which are the ids inside the document. `DFA-E203` (bad id shape) applies the date pattern for this schema value and the prefix pattern for every other.

### Silences filled

10. **The project key is the absolute project root path with every character outside `[A-Za-z0-9]` replaced by `-`.** Verified against the directory listing of `~/.claude/projects`: `C:\Projects\DevForgeAI` gives `C--Projects-DevForgeAI`, `C:\Projects\New folder` gives `C--Projects-New-folder`, and a UNC path gives a name starting with two dashes. Claude Code owns this naming and may change it; `[reflect].session_key` is the override that keeps a run working when it does, and a changed scheme leaves `sessions.status: absent` rather than a wrong reading.

11. **An absent session directory is not a gate failure.** `reflect-obs-cites-source` accepts a report path or a session id, so every observation in a session-less run cites a report path and the gate passes. The fact is recorded in `sources.sessions.status` and `sources.sessions.reason`, so a reader knows the window was read from reports alone.

12. **Time per phase comes from each report's `started_at` and `finished_at`, not from `state.toml`.** `state.toml`'s schema in `specs/01-cli.md` carries `updated_at`, `[last_gate].at`, and `[last_handoff].rendered_at` — one snapshot each, not a series, so no per-phase duration can be differenced out of it. `specs/01-cli.md` Decision 41 guarantees a report for every phase, and its report schema carries both timestamps, which is the series. `state.toml` supplies `current_phase`, `[active]`, and `[last_gate]` to the aggregate's `state` block and the handoff's transition table.

13. **This spec reads `state.toml` by the `specs/01-cli.md` schema and by no sibling's.** `specs/02-explore.md` names `explore.started_at` and `explore.timebox_days`, `specs/03-discover.md` names `phase` and `active_id`, and `specs/04-constitute.md` names `[current] phase` and `[current] id`. `specs/01-cli.md` owns the file and gives `current_phase`, `[active].<phase>`, `[last_gate]`, `[last_handoff]`, `[stop_hook]`. The aggregate reads those, and the four divergent shapes are a reconciliation item for the CLI spec's author rather than a choice this spec makes.

14. **`OBS-nnn` and `REC-nnn` are allocated globally and monotonically across every reflect report, by `doc validate --allocate`.** `specs/01-cli.md`'s Reflect integration row already names both calls. Global numbering lets a `REC-nnn` in a later report cite an `OBS-nnn` an earlier one defined, and keeps two reports from holding two different `OBS-014`s. The `reflect-rec-cites-obs` check resolves within one document, so a cross-document citation is a reader's convenience and not a gate subject.

15. **Subagents return a local `ref`, and the model attaches the allocated id.** Two agents run in parallel and both emit observations; an agent that allocated its own id would collide with the other's. Each `ref` matches `^(om|sp|rd)-[0-9]{3}$`, unique within one agent's return, and steps 6 and 8 map it to the allocated id.

16. **`observations[].kind` is closed at five and `recommendations[].target.kind` at six.** The five are the five the task names. The six are the installed targets: `skill`, `subagent`, `template`, `hook`, `gate_threshold`, and `framework_file` for an installed path none of the other five names. The `command` kind is gone with `commands/`, and every path is one the target project holds, because the framework repository is not on disk there.

17. **Severity is `low | medium | high`.** `technical-debt-analyzer.md` carries an uppercase triple whose top value matches the §2 ceremony pattern, and Q-007 in `specs/questions.md` shows the orchestrator applies that pattern to enum data values. `observation-extractor.md`'s lowercase triple collides with nothing and covers the same range.

18. **No subagent of this skill is a registered verifier, so `config.toml` gains no `[[verifier]]` entry and the handoff omits its `Verified` line.** Every gate check here is a structural property of a written YAML document, which the CLI reads directly; a verifier block would restate a fact `yaml_cites` already decides. §6 makes the line optional, so both handoff examples fit without it.

19. **The reflect document and the reflect gate report are two files, on the precedent `specs/01-cli.md` Decision 46 set for Verify.** `.devforgeai/reports/reflect-<date>.yaml` is the §5 document this skill writes; `.devforgeai/reports/<date>-reflect.yaml` is the CLI's gate report for the run, written by `gate check --phase reflect --id <date>`. The handoff's `Full report:` line cites the first, because the recommendations live there.

20. **One document per date.** A second `/reflect` on the same date rewrites `reflect-<date>.yaml` over the first, with newly allocated `OBS-nnn` and `REC-nnn` ids from the global counter. This keeps the `{id}` token of the gate entry resolving to one path and matches §5's `reports/reflect-<date>.yaml` naming, which admits no per-run discriminator.

21. **The command preamble passes `$ARGUMENTS` through unchanged.** `report aggregate` accepts both the positional id and `--since <date>`, so one `!` line covers both forms of the command, and the shape check lives in the binary rather than in prose, per §1 rule 1.

22. **`/reflect` runs with no gate requirement.** The reflect gate's `requires` is `""`, so a user may run `/reflect` in a project whose first phase has not finished; the window then matches no report and step 2's failure path writes a document with empty lists and one `open_questions` line.

23. **A `/reflect` run prints two handoff blocks: Reflect's, then the current phase's**, on the precedent `specs/08-design.md` Decision 4 set. Step 11 runs `handoff --phase reflect`; the Stop hook then runs `gate check --phase <current>` and `handoff` for the phase the user was in. Both come from the CLI.

24. **§9's two SEND BACK cases are filled by the two session-history cases.** `## Send-back` gives the reason: this skill emits and receives none, so the path does not exist to exercise. Cases `rf-03` and `rf-04` each feed a session history holding another phase's repeated send-back and assert the `OBS-nnn` that names it, which is the nearest thing this skill has to the behaviour §9 is testing for. `specs/01-cli.md` Decision 5 and `specs/08-design.md` fill "none" sections the same way.

25. **`graders.py` parses the report with a reader written in the module rather than importing a YAML library.** §9 forbids network and subprocess calls and `specs/01-cli.md` Decision 42 gives the runner a bare workspace; a third-party import would add an install step to the eval harness. The document and the fixtures use block mappings, block sequences, plain scalars, and one folded scalar per `detail` and `change` field, which a small recursive-descent reader covers.

26. **`AP-nnn` is a Constitute id from `specs/04-constitute.md`, not from §5.** §5's Constitute row names `ADR-nnn` and `CON-nnn` only; `specs/04-constitute.md` `## Outputs` and its Integration row add `AP-nnn` for the `## Anti-pattern index` rows of `anti-patterns.md`. `technical_debt.groups[].constraint` accepts both prefixes because a deferred Definition-of-Done item cites whichever of the two the QA report recorded.

### Dependencies on specs being written in parallel

Each entry names the §5 contract this spec relies on and what changes if the parallel spec lands differently. In every case the key this spec reads is the CLI's, because `gate check` writes the report and `specs/01-cli.md` fixes its schema.

27. **`specs/05-plan.md`.** §5 gives Plan `stories/STORY-nnn.md` and `stories/sprint.yaml`, ids `STORY-nnn`, `AC-nnn`, `SPRINT-nnn`. This spec reads `.devforgeai/reports/SPRINT-nnn-plan.yaml`, which `specs/01-cli.md` Decision 41 guarantees `gate check` writes, and takes its `checks[]`, `findings[]`, `started_at`, and `finished_at`. A `REC-nnn` against Plan names `.claude/skills/planning-work/SKILL.md`, a file under `.claude/skills/planning-work/templates/`, or a subagent Plan's `## Subagents` section names. If Plan's report carries a differently named key, the aggregate's `reports[]` entry for that phase loses that field and the observation kinds that read it produce nothing for Plan; no other phase is affected.
28. **`specs/06-build.md`.** §5 gives Build `reports/STORY-nnn-build.yaml`, YAML, CLI-written, no ids of its own. This spec reads its `checks[]`, `coverage`, `findings[]`, and both timestamps. The `build-tests`, `build-coverage`, `build-lint`, and `build-design` check ids come from the default `gates.toml` in `specs/01-cli.md` `## Gate` and not from Build's spec, so a `gate_failure` observation naming one of them holds whatever Build decides about its workflow.
29. **`specs/07-verify.md`.** §5 gives Verify `reports/STORY-nnn-qa.yaml` with `FIND-nnn`. This spec's `technical_debt` section reads a `deferrals[]` sequence in that document, each item holding `dod_item` (string), `deferred_at` (`YYYY-MM-DD`), `reason` (string), and a `constraint` (`CON-nnn`, `AP-nnn`, or absent). That sequence is this spec's one assumption about a document another spec owns. If Verify names the sequence differently, or puts the deferral records in `reports/STORY-nnn-verify.yaml` instead, the change lands in `report aggregate`'s deferral reader and in the `debt-aggregator` input list, and nothing else in this spec moves. `deferral-validator.md` exists at `C:\Users\bryan\.claude\agents\` and belongs to Verify, which validates a deferral at the moment it is made; this skill counts deferrals long after, which is why `debt-aggregator` does not adapt it.
30. **`specs/09-release.md`.** §5 gives Release `releases/vX.Y.Z.yaml` with no ids of its own. This spec reads that manifest for the `STORY-nnn` list that bounds a version-mode window, and `.devforgeai/reports/vX.Y.Z-release.yaml` for its `checks[]`. The second path is inferred rather than read: `specs/01-cli.md` Decision 41 gives every phase a `reports/<id>-<phase>.yaml`, and its Integration row gives Release's gate subject as `vX.Y.Z`. Two fallbacks cover a different landing. If the manifest names its story list differently, a version-mode window falls back to the reports whose `id` equals the version string. If the release gate report lands at another path or under another subject id, the aggregate's `reports[]` holds no release entry, the `gate_failure` and `phase_time` observations produce nothing for Release, and every other phase in the window is unaffected. `--since` mode is unaffected by either.

### Blockers

31. **None.** Every step runs on the §2 primitive list: Read, Write, Glob, Grep, Agent, the hooks of §7, one slash command, one skill, four subagents, and the `devforgeai` binary. AskUserQuestion is not used: this skill reads and reports, and a question would ask the user for a fact the reports already carry. Seven CLI additions are asked for and named in Decisions 1 to 7, all of them arithmetic or schema validation that §1 rule 1 keeps out of skill prose. The one input outside the project root is `~/.claude/projects/<project-key>/*.jsonl`, which is a file the Read tool opens and which `[reflect].session_root` relocates when it is somewhere else; its absence is a named, handled path rather than a failure.
