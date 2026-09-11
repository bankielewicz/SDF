# DevForgeAI user guide

This guide is for someone using DevForgeAI from the Claude Code terminal in their own project. It covers installing the framework, the nine slash commands, the documents each one writes, how gates and hooks decide whether a phase advances, and what to do when something refuses.

Everything here was read from the built artefacts. Terminal blocks marked **reproduced** were produced by running `cli/target/release/devforgeai.exe` in a throwaway git repository and pasted verbatim; paths in them are that repository's paths.

---

## 1. What the framework is

DevForgeAI runs software delivery as seven numbered phases plus two cross-cutting ones. Each phase is a single Claude Code skill that is its own slash command, reads the typed document the previous phase wrote, and writes one typed document of its own under `.devforgeai/`. The skills hold judgment, workflow, and templates; nothing else. Enforcement lives outside them, in a Rust binary called `devforgeai`, in Claude Code hooks that call it on six events, and in three git hooks. A phase advances when the binary says its gate passed, not because a model said it did. The framework names no language, package manager, test runner, linter, or build tool: those are detected into `.devforgeai/config.toml` by `devforgeai stack detect`, and every skill reads them from there.

---

## 2. Prerequisites

| Requirement | Why | Check |
|---|---|---|
| git | `devforgeai` installs three git hooks and Build works inside git worktrees | `git --version` |
| Claude Code | the nine commands are Claude Code skills; the gates are Claude Code hooks | `claude --version` |
| A `devforgeai` binary | either a released binary or a Rust toolchain to build one | `devforgeai --version` |

Build from source if you do not have a released binary:

```
cd C:\Projects\DevForgeAI
cargo build --release --manifest-path cli\Cargo.toml
```

That writes `cli\target\release\devforgeai.exe`. Put that directory on `PATH`, or type the full path in place of `devforgeai` everywhere below.

```
$ devforgeai --version
devforgeai 1.0.0 (unversioned)
```

**reproduced.** The parenthesised value is `cli/REVISION` line 1. It reads `unversioned` when the framework repository has no commit the digest tool could name; after a commit, `devforgeai trust digest` rewrites it with the commit SHA.

Python 3 is needed only to run the framework's own eval suite (§16) and the ceremony scanner. A target project needs neither.

---

## 3. Install, step by step

### 3.1 Pin the binary — outside Claude Code

Open a plain terminal. Not Claude Code, not a Claude Code Bash tool call.

```
cd C:\Projects\DevForgeAI
cli\target\release\devforgeai.exe trust pin --framework C:\Projects\DevForgeAI
```

`--framework` is required. It records the path that `devforgeai init --from` defaults to.

Two separate mechanisms hold this step outside Claude Code, because each closes a hole the other leaves:

- The binary refuses `trust pin` when `CLAUDECODE` or `CLAUDE_CODE_ENTRYPOINT` is set in the environment (`DFA-E500`, exit 4). A child process that strips its own environment walks past that guard.
- The `PreToolUse` shell hook denies any `Bash` or `PowerShell` command whose text contains `trust pin`, before the process starts, in every permission mode.

```
$ echo '{"tool_name":"Bash","tool_input":{"command":"devforgeai trust pin --framework /c/Projects/DevForgeAI"}}' \
    | devforgeai hook run pre-tool-use
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"devforgeai: trust pin runs only in a terminal outside Claude Code"}}
```

**reproduced**, exit 2.

The pin writes `~/.devforgeai/trust.toml`, holding the binary's SHA-256, the digest of the `cli/` source tree, and the framework path. §13 covers what the pin protects and what happens after you rebuild.

Confirm it took:

```
$ devforgeai trust verify
trust verify  ok  sha256:865e5d...
```

**reproduced**, exit 0.

### 3.2 Initialise the target project

```
cd <your project>
devforgeai init --from C:\Projects\DevForgeAI
```

```
$ devforgeai init --from 'C:\Projects\DevForgeAI'
Initialised .devforgeai/ in C:\Users\bryan\AppData\Local\Temp\...\demo
Copied     102 skills, 46 agents
Commands   /build, /constitute, /design, /discover, /explore, /plan, /reflect, /release, /verify
Hooks      .claude/settings.json merged; git hooks pre-commit, commit-msg, pre-push
CLAUDE.md  devforgeai section written
Stack      undetected
Next       /explore
```

**reproduced**, exit 0. `--from` may be omitted once a pin exists: it then defaults to the `framework_path` the pin recorded. Without either, `init` exits `DFA-E111`.

`Stack undetected` is what an empty project produces. It sets `degraded = true` in `config.toml`, which makes the four command-running gate checks (`tests_pass`, `coverage_min`, `lint_clean`, `complexity_clean`) evaluate to `skip` and count as passing. A project with a manifest detects its stack and clears the flag (§12).

`Next /explore` is the line to follow. If `trust verify` fails at the end of `init`, that line carries the pin command instead.

### 3.3 What `init` writes

```
<target>\
├── .claude\skills\{explore,discover,constitute,plan,build,verify,release,design,reflect}\
├── .claude\agents\
├── .claude\settings.json          hooks + two permission rules merged in
├── CLAUDE.md                      a section between markers
├── .gitignore                     two lines appended
├── .devforgeai\
│   ├── config.toml                from stack detect
│   ├── gates.toml                 the only place pass criteria live
│   ├── state.toml                 current phase, active ids, last gate result
│   └── explore\ context\ adr\ stories\ ui-specs\ brand\ reports\ releases\
└── .git\hooks\{pre-commit,commit-msg,pre-push}
```

Each skill lands at `.claude/skills/<its frontmatter name>/`, not under the framework's own directory name. The framework tree keeps long names (`skills/exploring-ideas/`) because `produced_by: exploring-ideas` in every template resolves against them and the producer check reads that value; Claude Code derives a project skill's slash command from its **directory** name, so `init` installs `skills/exploring-ideas/` at `.claude/skills/explore/` and `/explore` is the command in your project. Every `evals/` subtree is left behind.

`.gitignore` gains two lines: `.explore-prototype/` and `.devforgeai/state.toml`.

`.claude/settings.json` gains the six hook events (§10) and two permission rules:

```json
"permissions": { "allow": ["Bash(devforgeai *)", "PowerShell(devforgeai *)"] }
```

**reproduced** from the merged file.

### 3.4 The CLAUDE.md block

`init` writes this between `<!-- devforgeai:begin -->` and `<!-- devforgeai:end -->`, creating `CLAUDE.md` if absent and replacing only the span on a re-run:

```markdown
<!-- devforgeai:begin -->
## DevForgeAI

Delivery runs as phases. Each phase is one skill with one slash command, writes one
document under `.devforgeai/`, and ends with a handoff block naming the next command.

| Command | Phase | Writes |
|---|---|---|
| `/explore "<idea>"` | 0 Explore | `explore/brief.md`, `explore/decision.yaml` |
| `/discover <IDEA-nnn>` | 1 Discover | `requirements.yaml` |
| `/constitute <IDEA-nnn>` | 2 Constitute | `context/*.md`, `adr/ADR-nnn.md` |
| `/plan <EPIC-nnn>` | 3 Plan | `stories/STORY-nnn.md`, `stories/sprint.yaml` |
| `/build <STORY-nnn>` | 4 Build | source and tests, `reports/STORY-nnn-build.yaml` |
| `/verify <STORY-nnn>` | 5 Verify | `reports/STORY-nnn-qa.yaml` |
| `/release <vX.Y.Z>` | 6 Release | `releases/vX.Y.Z.yaml` |
| `/design <id>` | any | `brand/tokens.json`, `ui-specs/UI-nnn.md` |
| `/reflect <window>` | any | `reports/reflect-<date>.yaml` |

The handoff comes from the Stop hook at the end of each phase. Its `Next` line is the
command to type next, copied as printed, arguments included.

Phase documents live under `.devforgeai/`. Source and tests live where
`.devforgeai/config.toml` records the project's roots.

Gates are the `devforgeai` binary and the Claude Code hooks that call it, not
instructions in this file: a write to `.devforgeai/` is checked as it happens, a phase
ends when its gate passes, and a failing gate holds the turn open.

A defect found in an upstream document is cited, not edited. The handoff prints the
send-back command that reopens the cited ids.
<!-- devforgeai:end -->
```

**reproduced** from the written file; identical to `CLAUDE_MD_BLOCK` in `cli/src/cmd/init.rs`.

### 3.5 Start a Claude Code session

Start Claude Code in the project. The `SessionStart` hook runs `stack detect` and `handoff` and adds their output to Claude's context:

```
$ echo '{"source":"startup"}' | devforgeai hook run session-start
degraded: true
Stack undetected; command checks skipped.
Phase     0 · Explore         IDEA-001 · nightly-bank-reconciliat
Done      1 ideas · 0 flows
Gate      PASS  12 checks
Verified  kill-case-builder · 3/3 objections

Next      /discover IDEA-001
Then      /constitute IDEA-001
Blocked   none

Full report: .devforgeai/reports/IDEA-001-explore.yaml
```

**reproduced**, exit 0. This event has no `systemMessage`: plain stdout is what Claude Code adds to context, and Claude is the audience for a session-opening block.

---

## 4. The pipeline

```
0 Explore ─→ 1 Discover ─→ 2 Constitute ─→ 3 Plan ─→ 4 Build ─→ 5 Verify ─→ 6 Release
                                                        ↑__________|
   ┌── /design ──────────────────────────────────────────────────────┐  cross-cutting,
   └── /reflect ─────────────────────────────────────────────────────┘  no phase number
```

| Order | Command | Skill directory | Phase |
|---|---|---|---|
| 0 | `/explore` | `exploring-ideas` | Explore |
| 1 | `/discover` | `discovering-requirements` | Discover |
| 2 | `/constitute` | `establishing-context` | Constitute |
| 3 | `/plan` | `planning-work` | Plan |
| 4 | `/build` | `implementing-stories` | Build |
| 5 | `/verify` | `validating-quality` | Verify |
| 6 | `/release` | `releasing-software` | Release |
| — | `/design` | `designing-interfaces` | cross-cutting |
| — | `/reflect` | `improving-framework` | cross-cutting |

Design and Reflect hold no number, no gate in `gates.toml`, and no key in `state.toml` `[active]`. A run of either leaves `[current].phase` and `[current].id` where it found them, and records `[last_cross]` instead. `gate check --phase design` exits 1 with `DFA-E300`.

Verify loops back to Build until a story passes; Release takes every story at `status: built`. Reflect reads a window of reports and produces recommendations that set no gate.

### 4.1 The ids

| Prefix | Allocated by | Lives in | Means |
|---|---|---|---|
| `IDEA-nnn` | Explore step 1, or Discover step 2 at entry point B | `explore/brief.md`, `requirements.yaml` | one product idea; the subject Explore, Discover and Constitute key on |
| `FLOW-nnn` | Explore step 4, in `## Core flows` row order | `explore/brief.md` | one core flow of the idea |
| `PERSONA-nnn` | Discover step 10 | `requirements.yaml` | one actor |
| `EPIC-nnn` | Discover step 11 | `requirements.yaml` | one outcome group; the subject Plan keys on |
| `REQ-nnn` | Discover step 10 | `requirements.yaml` | one requirement |
| `CON-nnn` | Constitute step 7 | `context/architecture-constraints.md` | one architectural constraint |
| `AP-nnn` | Constitute step 9 | `context/anti-patterns.md` | one anti-pattern with a detector |
| `ADR-nnn` | Constitute step 10; `ADR-000` reserved | `adr/ADR-nnn.md` | one closed architectural choice |
| `SPRINT-nnn` | Plan step 4 | `stories/sprint.yaml` | one capacity-bounded story set |
| `STORY-nnn` | Plan step 8 | `stories/STORY-nnn.md` | one unit of work; the subject Build and Verify key on |
| `AC-nnn` | Plan step 8 | `stories/STORY-nnn.md` | one acceptance criterion in Given/When/Then form |
| `UI-nnn` | Design step 12 | `ui-specs/UI-nnn.md` | one screen specification |
| `TOKEN-<name>` | Design step 7 | `brand/tokens.json` | one design token, `TOKEN-<group>-<leaf>` |
| `FIND-nnn` | Verify step 5, one band per subagent | `reports/STORY-nnn-qa.yaml` | one QA finding |
| `OBS-nnn` | Reflect step 6 | `reports/reflect-<date>.yaml` | one observation about a window |
| `REC-nnn` | Reflect step 8 | `reports/reflect-<date>.yaml` | one recommendation naming one file and one change |
| `vX.Y.Z` | you, on the `/release` line | `releases/vX.Y.Z.yaml` | one named version |

Every `PREFIX-nnn` is zero-padded to three digits and allocated by `devforgeai doc validate --allocate <prefix>`. Nothing else allocates one.

```
$ devforgeai doc validate --allocate IDEA
IDEA-001
```

**reproduced**, exit 0. An unknown prefix exits 3:

```
$ devforgeai doc validate --allocate WIDGET
devforgeai: DFA-E214 doc validate: 'WIDGET' is not an ID prefix; conventions section 5 defines IDEA, FLOW, PERSONA, REQ, EPIC, CON, AP, ADR, STORY, AC, SPRINT, UI, TOKEN, FIND, OBS, REC
```

**reproduced.**

### 4.2 What each phase reads and writes

| Phase | Reads | Writes | ID prefixes it allocates | May send back to |
|---|---|---|---|---|
| Explore | nothing upstream | `.devforgeai/explore/brief.md`, `explore/decision.yaml`, `explore/seed-data.json`, `explore/sketch-request.json`, `explore/mockups/` | `IDEA`, `FLOW` | none (phase 0) |
| Discover | `explore/brief.md`, `explore/decision.yaml` | `.devforgeai/requirements.yaml` | `IDEA` (entry B), `PERSONA`, `REQ`, `EPIC` | Explore |
| Constitute | `requirements.yaml`, `explore/brief.md`, `explore/decision.yaml`, `config.toml` | `.devforgeai/context/{tech-stack,source-tree,dependencies,coding-standards,architecture-constraints,anti-patterns}.md`, `.devforgeai/adr/ADR-nnn.md` | `CON`, `AP`, `ADR` | Discover, Plan |
| Plan | `requirements.yaml`, the six context files | `.devforgeai/stories/STORY-nnn.md`, `stories/sprint.yaml` | `SPRINT`, `STORY`, `AC` | Discover, Constitute, Design |
| Build | one `STORY-nnn.md`, the six context files, `sprint.yaml`, `ui-specs/UI-nnn.md`, `explore/seed-data.json` | source and test files inside the story's `## Files` set, `.devforgeai/build/STORY-nnn-note.yaml` | none | Plan |
| Verify | one `STORY-nnn.md`, the six context files, `reports/STORY-nnn-build.yaml` | `.devforgeai/reports/STORY-nnn-qa.yaml` | `FIND` | Build, Plan |
| Release | every story at `status: built`, their QA and verify reports, the context files, the ADRs, `brand/` | `.devforgeai/releases/vX.Y.Z.yaml`, deploy manifests, `<docs_root>/`, `.github/workflows/devforgeai-release.yml` | none | Verify |
| Design | `explore/brief.md`, `requirements.yaml`, one `STORY-nnn.md`, `config.toml` | `.devforgeai/brand/tokens.json`, `brand/logo.svg`, `brand/brand-kit.md`, `ui-specs/UI-nnn.md`, `explore/mockups/` | `UI` | Discover |
| Reflect | the `report aggregate` window, the two context index files, the installed `.claude/` tree | `.devforgeai/reports/reflect-<date>.yaml` | `OBS`, `REC` | none |

Reports under `.devforgeai/reports/` are written by the binary, not by a skill. `produced_by` in one is `devforgeai-cli`. The exception is `reports/STORY-nnn-qa.yaml`, which Verify writes itself — a different file from `reports/STORY-nnn-verify.yaml`, which `gate check --phase verify` writes and which the handoff's `Full report:` line cites.

Every document under `.devforgeai/` carries the same seven top-level keys, in this order, with no other top-level key:

```yaml
schema: devforgeai/<doc-type>/1
id: <ID>
phase: <phase-name>
status: <enum defined per doc-type>
produced_by: <skill-directory-name>
consumes: [<ID>, ...]
open_questions: []
```

Markdown documents carry them as YAML frontmatter; YAML documents carry them as their first seven top-level keys. `brand/tokens.json` is the one exception: it carries them under `meta`, because the rest of that file is a token tree and a sibling key at the root would read as a token group.

---

## 5. A complete worked example

A fresh project, from the first idea to the first release. Each phase has its own self-contained transcript in [`examples/`](examples/); this is the through-line.

### Explore

You type:

```
/explore "bookkeepers spend two hours a night matching bank lines to ledger entries by hand"
```

`exploring-ideas` runs no preamble command. It allocates the id inside the workflow once it knows this is a fresh run, then claims the phase:

```
$ devforgeai doc validate --allocate IDEA
IDEA-001

$ devforgeai phase set explore --id IDEA-001
Phase     0 · Explore      IDEA-001
```

**reproduced**, both exit 0.

It asks you two questions with `AskUserQuestion`, both with fixed headers. The first, header `Holders`, is multi-select over 2 to 4 candidate segments a subagent drafted plus a `Someone else` option. The second, header `Decision`, comes at the end of the run with three options: `Kill`, `Park`, `Promote`. A `Park` answer is followed by a third question, header `Revisit`, offering today plus 30, 90 and 180 days.

Between them: a competitor and technology scan, a one-page brief, wireframe mockups drawn by the `design` skill in sketch mode, an optional clickable prototype under `.explore-prototype/`, and a `kill-case-builder` subagent that argues against the idea.

Files that appear:

```
.devforgeai/explore/brief.md              twelve sections, status: decided
.devforgeai/explore/decision.yaml         the decision, its reason, and carry_forward
.devforgeai/explore/seed-data.json        example rows Build later uses as fixtures
.devforgeai/explore/sketch-request.json   the request Design's sketch mode read
.devforgeai/explore/mockups/              one HTML file per screen
```

The Stop hook then runs `gate check --phase explore --id IDEA-001` and `handoff`. The twelve explore checks:

```
$ devforgeai gate check --phase explore --id IDEA-001
Gate      explore · IDEA-001
  pass    decision-exists   file_exists
  pass    decision-enum     field_in_enum
  pass    decision-dated    field_is_date
  skip    park-has-revisit  field_is_date   not_required
  pass    promote-carries-forwardlength_between
  pass    flow-count        row_count_between
  pass    flow-id-shape     column_matches
  pass    one-success-signalrow_count_between
  pass    kill-case-answeredverifier_pass
  skip    remedy-flows-presentcolumn_contains_allcondition
  skip    time-box          elapsed_days_at_mostcondition
  pass    prototype-pruned  file_exists
Result    PASS
Report    .devforgeai/reports/IDEA-001-explore.yaml
Stack undetected; command checks skipped.
```

**reproduced**, exit 0. (The missing space between a long check id and its kind is a rendering defect in the human output, not in the report; §17 lists it.)

The Stop hook prints one JSON object and nothing else:

```json
{"systemMessage":"Phase     0 · Explore         IDEA-001 · nightly-bank-reconciliat\nDone      1 ideas · 0 flows\nGate      PASS  12 checks\nVerified  kill-case-builder · 3/3 objections\n\nNext      /discover IDEA-001\nThen      /constitute IDEA-001\nBlocked   none\n\nFull report: .devforgeai/reports/IDEA-001-explore.yaml"}
```

**reproduced**, exit 0. Claude Code renders the `systemMessage` to you as the block:

```
Phase     0 · Explore         IDEA-001 · nightly-bank-reconciliat
Done      1 ideas · 0 flows
Gate      PASS  12 checks
Verified  kill-case-builder · 3/3 objections

Next      /discover IDEA-001
Then      /constitute IDEA-001
Blocked   none

Full report: .devforgeai/reports/IDEA-001-explore.yaml
```

**reproduced.** Twelve lines maximum, including blanks. Column two starts at character 11. The model composes no part of this and writes no part of it at the close of a phase.

### Discover

Copy the `Next` line as printed:

```
/discover IDEA-001
```

The preamble line `` !`devforgeai doc load discover-entry "$ARGUMENTS[0]"` `` runs before the skill body loads. It exits 0 in every case; what it printed selects the entry point — the brief and decision record for a promoted idea, the prior `requirements.yaml` for a re-open, nothing for a typed description.

Four rounds of `AskUserQuestion`, headers fixed in `templates/questions.md`: actors, outcomes, boundaries, then acceptance. The outcome you select becomes an epic's `success_metric` verbatim, so a candidate carries one measurable quantity, its unit, and a comparison — `Unmatched lines fall below 5 per night.` reaches the field word for word.

One file appears, `.devforgeai/requirements.yaml`, holding `personas[]`, `epics[]`, and `requirements[]`. The `Accept` answer runs:

```
devforgeai doc accept requirements --id IDEA-001
```

The binary writes `accepted_by: user`, `accepted_at` in RFC 3339 UTC, and moves every requirement at `draft` or `reopened` to `accepted`. The timestamp comes from the binary, not from an edit.

Handoff:

```
Next      /constitute IDEA-001
Then      /plan EPIC-001
```

### Constitute

```
/constitute IDEA-001
```

Two preamble lines run before the body loads:

```
!`devforgeai gate require constitute $ARGUMENTS[0]`
!`devforgeai doc load requirements $ARGUMENTS[0]`
```

The first is the gate. A non-zero exit aborts the whole invocation, so the body does not load:

```
$ devforgeai gate require constitute IDEA-001
devforgeai: DFA-E321 gate require: constitute needs discover gate PASS for IDEA-001; no report at .devforgeai/reports/IDEA-001-discover.yaml; to repair, run /discover IDEA-001, then /constitute IDEA-001
```

**reproduced**, exit 1. The same refusal reaches you earlier, from the `UserPromptExpansion` hook, before the skill is even loaded (§10.2).

Six context files and a set of ADRs appear:

```
.devforgeai/context/tech-stack.md
.devforgeai/context/source-tree.md
.devforgeai/context/dependencies.md
.devforgeai/context/coding-standards.md
.devforgeai/context/architecture-constraints.md    the CON-nnn index
.devforgeai/context/anti-patterns.md               the AP-nnn index
.devforgeai/adr/ADR-000.md                          why this project exists
.devforgeai/adr/ADR-nnn.md                          one per closed choice
```

One `AskUserQuestion` at the end, three options: `accept`, `amend`, `send back`. `accept` edits `status: accepted` into all six frontmatters and every ADR.

Handoff:

```
Next      /plan EPIC-001
Then      /build STORY-001
```

### Plan

```
/plan EPIC-001
```

Three preamble lines:

```
!`devforgeai gate require plan $ARGUMENTS[0]`
!`devforgeai doc load requirements $ARGUMENTS[0]`
!`devforgeai doc load context all`
```

`phase set` for this phase takes an extra flag, because `sprint.yaml` does not exist yet:

```
devforgeai phase set plan --id SPRINT-001 --epic EPIC-001
```

One `STORY-nnn.md` per unit of work and one `sprint.yaml`. Each story carries ten sections in template order: `## Story`, `## Requirements`, `## Acceptance Criteria`, `## Constraints`, `## Anti-patterns`, `## Interface`, `## Layer`, `## Files`, `## Dependencies`, `## Out of scope`. The `## Files` table is the writable set Build is held to.

Story `status` runs `draft → ready → building → built → released`. Plan writes `draft` at step 8 and `ready` at step 12; `devforgeai phase set` writes the last three.

Handoff:

```
Next      /build STORY-001
Then      /verify STORY-001
```

### Build

```
/build STORY-001
```

Four preamble lines, one of them the gate:

```
!`devforgeai gate require build $ARGUMENTS[0]`
!`devforgeai doc load story $ARGUMENTS[0]`
!`devforgeai doc load context all`
!`devforgeai doc load sprint -`
```

Build opens a git worktree and works inside it:

```
devforgeai worktree ensure STORY-001
devforgeai phase set build --id STORY-001
devforgeai config get stack.test_command
```

Then one red-green cycle per `AC-nnn`, in `## Acceptance Criteria` order: a failing test, the test command, `devforgeai commit STORY-001 -m "AC-001 red"`, the smallest code that satisfies it, the test command again, `devforgeai commit STORY-001 -m "AC-001 green"`. Two different commit shas in one cycle are the record that the test existed and failed before the code existed. A refactor pass runs only when `build-lint` or `build-complexity` already carries `status: fail` — a rewrite is opened by a number, not by taste.

Build runs no git command of its own. Every worktree operation and every commit goes through `devforgeai`, so the hooks run on each one.

The `PreToolUse` hook checks each write against the story's `## Files` set while `[current].phase` is `build`. A path outside it is denied.

Handoff:

```
Next      /verify STORY-001
Then      /release v0.1.0
```

### Verify

```
/verify STORY-001
```

Seven read-only subagents in one batch in light mode, ten in `--deep`. Each returns a `devforgeai/verifier/1` envelope; the `SubagentStop` hook runs `devforgeai report ingest <name> -` and files each block into `.devforgeai/reports/STORY-001-verify.yaml`.

Verify writes `.devforgeai/reports/STORY-001-qa.yaml` — sixteen top-level keys, `findings[]` each with a `disposition` of `fix`, `defer` or `accept`, a `deferrals[]` sequence, a `blockers` list, and a `summary`. It touches no source file, no test, no story, and no context file.

### A send-back, and the round trip

Suppose two criteria are unmet. The gate result is not FAIL but SEND BACK, and the handoff carries two command lines:

```
$ devforgeai handoff
Phase     5 · Verify          STORY-001 · match-a-statement-line-t
Done      5/7 ACs · 6 findings
Gate      SEND BACK to Plan  verify-docs DFA-E200...
Verified  ac-compliance-verifier · 5/7 ACs
Found     AC-003 DFA-E325 2 references do not resolve: AC-003, AC-004
Found     +5 more in report

Next      /plan SPRINT-001 --remedy AC-003,AC-004
Then      /verify STORY-001 --resume
Blocked   none

Full report: .devforgeai/reports/STORY-001-verify.yaml
```

**reproduced**, exit 0.

Three things to notice.

1. The gate's static `send_back_to` for `verify` is `build`, but the block reads `SEND BACK to Plan`. `gate check` resolves the report's own target from the blocking findings: a blocking finding of `category: spec-gap` resolves `plan`, every other resolves `build`, and a set holding both resolves `build` — a defect in the code is repaired before the criterion that failed to catch it.
2. The `Found` lines are capped at three, and the third folds into `+n more in report` when the budget is spent. Severity order is `block`, `warn`, `info`, then ID ascending.
3. On a SEND BACK the Stop hook emits the block with **no** blocking decision and exits 0. The next step is a command you type, and holding the model in the turn cannot produce it.

The round trip is two commands, typed exactly as the block printed them:

```
/plan SPRINT-001 --remedy AC-003,AC-004
```

The remedy run replaces steps 2 to 13 with four steps: locate each cited criterion and its finding, triage each with `spec-gap-triager` into one of `rewrite_ac`, `send_back_discover`, `send_back_constitute`, `send_to_design`, act on it, then reset the edited stories to `status: ready`. A `rewrite_ac` edits that one criterion line in place and leaves every other byte of the file unchanged.

Then:

```
/verify STORY-001 --resume
```

The resume re-enters at step 3 and re-reads `reports/STORY-001-build.yaml`, so the rebuilt story's fresh coverage and gate results are the numbers the run works from. Every prior finding at `disposition: defer` or `accept` carries into the new report unchanged, including its original `opened_on`; every entry at `disposition: fix` is dropped and re-raised if the defect still stands.

`--remedy` and `--resume` are the only two flag names used for these purposes anywhere in the framework. The upstream command reopens only the cited ids; the returning command continues where it stopped.

### Release

```
/release v0.1.0
```

One preamble line, which is the release set:

```
!`devforgeai story list --status built --json`
```

Sixteen steps. The two worth knowing:

- **Step 2** runs `devforgeai phase set release --id v0.1.0`. It refuses with `DFA-E320` when a story in the set has no verify gate at PASS, and the run stops with one `Blocked` line naming `/verify STORY-nnn` — before a release file, a manifest, or a documentation page is touched. This is the common shape of a release failure, and it is not a send-back.
- **Step 7** resolves a deployment platform from a closed five: `kubernetes`, `compose`, `github-actions`, `vps`, `none`. `config.toml` `[release].platform` wins if set; otherwise marker files are tested in order (`kustomization.yaml`/`Chart.yaml`, then a root `docker-compose.yaml`, then `.github/workflows/deploy.yml`, then `<deploy_root>/vps/deploy.sh`); otherwise one `AskUserQuestion` with header `Platform`, and the answer is written back to `config.toml` so the next release reads it from the file.

`.devforgeai/releases/v0.1.0.yaml` carries sixteen top-level keys: the seven envelope keys, then `previous_version`, `released_at`, `stories`, `artifacts`, `platform`, `deploy`, `docs`, `notes`, `signoff`.

Handoff:

```
Next      /reflect
```

---

## 6. The two cross-cutting commands

### `/design`

One command, three modes. Spec mode is the default: any argument string holding neither `--sketch` nor `--brand` is spec mode.

| Form | Mode | Writes |
|---|---|---|
| `/design --sketch IDEA-nnn` | sketch | `explore/mockups/<FLOW-nnn-nn>.html`, `explore/mockups/brand-sketch.json` |
| `/design --brand EPIC-nnn` | brand | `brand/tokens.json`, `brand/logo.svg`, `brand/brand-kit.md` |
| `/design --spec STORY-nnn` or a bare `STORY-nnn` | spec | `ui-specs/UI-nnn.md`, one per screen |
| `/design UI-nnn --remedy AC-nnn,TOKEN-name,UI-nnn` | spec, remedy | the cited sections of that one file |
| any of the above plus `--resume` | same mode, resume | the outputs still absent |

`designing-interfaces` is the one skill without `disable-model-invocation: true`, because `exploring-ideas` and `planning-work` reach sketch and spec mode through the Skill tool, and a skill the model cannot invoke cannot be reached that way.

Brand mode before spec mode is the one ordering the skill imposes on itself: every colour and type value in a `UI-nnn.md` is a `TOKEN-<name>` reference, so a spec written before the brand kit would reference names nothing defines. With `brand/tokens.json` absent, spec mode writes no file and the handoff carries `Blocked   you: run /design --brand <EPIC-nnn> first`.

Brand mode asks five questions across two `AskUserQuestion` calls: `Tone`, `Color`, `Type`, `Density` in the first, `Logo` in the second. The token shape it writes: top-level keys `meta`, `color`, `type`, `spacing`, `radius`, `elevation`, `motion`, in that order; 12 colour, 12 type, 6 spacing, 4 radius, 3 elevation, 4 motion leaves. A `color` leaf is an object with exactly `light` and `dark`; every other leaf is a string. Flattened, a token is `TOKEN-<group>-<leaf>` and its CSS custom property is `--<group>-<leaf>`.

Design advances no phase. Its last step is:

```
devforgeai phase set design --id UI-004
```

which records `[last_cross]` in `state.toml` and leaves `[current]` alone.

### `/reflect`

```
/reflect IDEA-001 | EPIC-001 | STORY-001 | v0.1.0 | --since 2026-09-01
```

One preamble line, and it is the whole window:

```
!`devforgeai report aggregate $ARGUMENTS --json`
```

Every count and every timestamp in the report comes from that aggregate. The skill re-derives nothing from the report files.

A bare `/reflect`, or one carrying both an id and `--since`, exits 3 on `DFA-E430` in the preamble and the body does not load. A token that parses as no id exits 3 on `DFA-E013`.

It writes `.devforgeai/reports/reflect-<date>.yaml` with twelve top-level keys: the seven envelope keys, then `window`, `sources`, `observations`, `recommendations`, `technical_debt`. This is the one doc type whose `id` is a date rather than an allocated `PREFIX-nnn`.

A `REC-nnn` names one framework file and one change. It sets no gate and blocks nothing; it is prose until you run a command against the file it names. The six target kinds are ordered by specificity and a target takes the first row it matches — a change to `.devforgeai/gates.toml` or `config.toml` takes `kind: gate_threshold` whatever the change is, and `framework_file` names neither of those two files. A proposal below a compiled floor is dropped by the drafter and the drop appears in `open_questions` as `REC dropped: <key> proposed <value>, floor <value>`.

### One Stop, two blocks

Design and Reflect leave `[current].phase` where they found it, so the Stop that follows one of them has two blocks to render: the cross-cutting one first, then the phase's own, joined by a blank line, with `[last_cross]` cleared afterwards. A second Stop in the same session prints one block. The joined string is capped at 10,000 characters.

```
$ devforgeai phase set design --id UI-001
Phase     design · UI-001  (cross-cutting)

$ echo '{"stop_hook_active":false,"session_id":"doc"}' | devforgeai hook run stop
```

The `systemMessage`, unescaped:

```
Phase     — · Design          UI-001 · -
Done      UI-001
Gate      NOT RUN

Next      /explore IDEA-001
Blocked   none

Full report: none

Phase     0 · Explore         IDEA-001 · -
Done      0 ideas · 0 flows
Gate      FAIL  decision-exists DFA-E323 found 0 of 1 paths; missing...
...
```

**reproduced.** The em dash in column one is what a phase outside the numbered sequence renders. Design's `Next` line names the row the phase `[current]` still holds, because Design advances nothing of its own.

---

## 7. The handoff block

Printed by `devforgeai handoff`. Twelve lines maximum, including blanks. Column two starts at character 11; the `Full report:` line is exempt from the column rule and capped as a whole line at 100 characters.

```
Phase     <n> · <Name>        <ID> · <slug>
Done      <counts measured by the CLI>
Gate      PASS | FAIL | SEND BACK [to <Phase>]  <numbers or IDs that produced it>
Verified  <subagent> · <n>/<m> <unit>            (omitted when no verifier ran)
Found     <ID> <one-line defect>                 (SEND BACK only; max 3, then "+n more in report")

Next      /<command> <args exactly as you type them>
Then      /<command> <args>                      (omitted if none)
Blocked   none | you: <one question>

Full report: .devforgeai/reports/<ID>-<phase>.yaml
```

| Line | Where it comes from |
|---|---|
| `Phase` | `state.toml` `[current]`, and the first H1 of the phase's own document, lowercased and cut to 24 characters |
| `Done` | counts the binary measures on disk, one row per phase: ideas and flows; REQ, EPIC and personas; context files and ADRs; stories and ACs; tests and coverage; ACs and findings; stories and version |
| `Gate` | the report's `gate.result`, then the passing check count on PASS, or up to three failing `<id> <reason>` pairs otherwise |
| `Verified` | the report's `verifiers` block, showing the **lowest** `passed/total` ratio, with `+n more` when there are others |
| `Found` | the report's `findings[]`, severity `block` then `warn` then `info`, then ID ascending |
| `Next` / `Then` | a fixed transition table keyed on `(phase, gate result)` |
| `Blocked` | the first non-empty entry of the phase document's frontmatter `open_questions`, or `none` |

`Verified` picks the lowest ratio so one verifier's clean sheet cannot hide another's. A tie goes to the first `[[verifier]]` entry in `config.toml`, then to the order the report writes the blocks.

The `Next` line on a trust failure overrides every phase row and reads `devforgeai trust pin`.

---

## 8. Gates

A gate is an entry in `.devforgeai/gates.toml`. That file is the only place pass criteria live. Nothing in a skill decides whether a phase passed.

```toml
[[gate]]
phase = "explore"
requires = ""
on_fail = "fail"
send_back_to = ""
description = "A decision is recorded, the brief carries three to five flows, the time box holds."

  [[gate.check]]
  kind = "file_exists"
  id = "decision-exists"
  severity = "block"
  on_fail = "fail"
  paths = [".devforgeai/explore/decision.yaml"]
  min_count = 1
  message = "no decision recorded for {id}"
```

**reproduced** from the `gates.toml` `init` wrote.

| Key | Means |
|---|---|
| `phase` | which `gate check --phase` and `gate require` resolve to this block |
| `requires` | the predecessor phase whose gate has to be PASS before this one may start |
| `on_fail` | the gate-level result when a check fails: `fail` or `send_back` |
| `send_back_to` | the phase a SEND BACK names; empty when `on_fail` is `fail` |
| `[[gate.check]].id` | the string the handoff's `Gate` and the Stop `reason` name |
| `[[gate.check]].severity` | `block` lowers the result; `warn` records `DFA-W310` and changes nothing |
| `[[gate.check]].on_fail` | overrides the gate-level value for this check alone |
| `skip_when` / `required_when` / `null_when` / `empty_when` | conditions that turn a check into `skip` rather than `fail` |

### `gate require`

Runs from a skill's `!` preamble, before the body loads. Exit 0 means the predecessor gate passed for that id; a non-zero exit aborts the whole invocation, which is the gate.

```
$ devforgeai gate require constitute IDEA-001
devforgeai: DFA-E321 gate require: constitute needs discover gate PASS for IDEA-001; no report at .devforgeai/reports/IDEA-001-discover.yaml; to repair, run /discover IDEA-001, then /constitute IDEA-001
```

**reproduced**, exit 1.

Four skills open with it: `establishing-context`, `planning-work`, `implementing-stories`, `validating-quality`. Three carry a `doc load` preamble instead, because their subject is not known until the body decides the entry point (`discovering-requirements`, `releasing-software`, `improving-framework`); the `UserPromptExpansion` hook enforces their predecessor gate. Two carry no preamble at all (`exploring-ideas`, `designing-interfaces`): both allocate their ids inside a workflow step once the run kind is known, and a preamble that allocated one would abort a run that needs none.

### `gate check`

Evaluates every check in the phase's gate, writes `.devforgeai/reports/<id>-<phase>.yaml`, and updates `state.toml` `[last_gate]`.

```
$ devforgeai gate check --phase explore --id IDEA-001
```

Exit 0 is PASS, 1 is FAIL, 2 is SEND BACK.

The Stop hook runs it on every turn end, including a continuation after a block: Claude has been working on the failing checks since the last block, so re-running is the only way to notice that the work now passes.

### Three results

| Result | What produced it | What you do |
|---|---|---|
| **PASS** | every `block`-severity check passed or skipped | type the `Next` line |
| **FAIL** | a `block` check with `on_fail = "fail"` failed, or a document was written from the wrong phase | the defect is inside this phase; the Stop hook holds the turn open up to three times so Claude can repair it, then prints the block |
| **SEND BACK** | a `block` check with `on_fail = "send_back"` failed | the defect is in an upstream document; type the `Next` line, which reopens the cited ids with `--remedy`, then the `Then` line, which resumes with `--resume` |

A FAIL is repaired inside the turn. A SEND BACK cannot be, so the Stop hook does not block on one.

### Reading a report

`.devforgeai/reports/<ID>-<phase>.yaml`, written by the binary. Print it:

```
$ devforgeai report show IDEA-001 explore
Gate      explore · IDEA-001
  pass    decision-exists   file_exists
  ...
Result    PASS
Verified  kill-case-builder · 3/3 objections
```

**reproduced**, exit 0. One check entry at a time:

```
devforgeai report show STORY-001 build --check build-lint
```

Each check entry carries `id`, `kind`, `status` (`pass`, `fail`, `skip`), `severity`, `reason` (a `DFA-` code and message when it failed), and `evidence`. The report also carries `verifiers.<field>` blocks written by `report ingest` from `SubagentStop`, and `findings[]`.

`DFA-E400` means the report is not there yet — for Build's partial report, running the test command once more makes the `PostToolUse` hook write it.

---

## 9. What `devforgeai` does when you run it yourself

You will rarely type a `devforgeai` command by hand: the skills, the preambles and the hooks run them. Four are worth knowing.

```
$ devforgeai handoff
```
Reprints the last block for the current phase. Safe at any time.

```
$ devforgeai report show <ID> <phase>
```
The full gate report.

```
$ devforgeai story list --status built
```
What a `/release` would pick up.

```
$ devforgeai trust verify
```
Whether the hooks will work at all.

---

## 10. Hooks, and what you see when one blocks

`init` merges six events into `.claude/settings.json`, dispatching seven arms of `devforgeai hook run <arm>`. Every arm runs `devforgeai trust verify` first.

| Event | Matcher | Arm | Calls | Channel | Blocks on |
|---|---|---|---|---|---|
| SessionStart | `startup\|resume\|clear\|fork` | `session-start` | `stack detect`, `handoff` | plain stdout → Claude's context | nothing |
| UserPromptExpansion | the nine skill names | `prompt-expansion` | `trust verify`, `gate require` | `reason` of the blocking decision | trust failure; predecessor gate not passed |
| PreToolUse | `Write\|Edit\|NotebookEdit` | `pre-tool-use` | `doc validate --producer-check`, `story files --check`, `design lint` | `permissionDecisionReason` | producer mismatch; undeclared file; token violation |
| PreToolUse | `Bash\|PowerShell` | `pre-tool-use` | producer check on every path the command appears to write; the metrics-command decision | same | a shell write under `.devforgeai/` from the wrong phase; an analysis agent running an unconfigured command; any `trust pin` |
| PreToolUse | `Write\|Edit\|NotebookEdit\|Bash\|PowerShell\|Agent` | `trust-check` | `trust verify` and nothing else | same | trust failure |
| PostToolUse | the write tools, path under `.devforgeai/` | `post-tool-use` | `doc validate` | `additionalContext` → Claude | nothing |
| PostToolUse | `Bash\|PowerShell` matching a `config.toml` test command | `post-tool-use`, async | `gate check --phase build --partial` | `additionalContext` on the next turn | nothing |
| Stop | none | `stop` | document scan, `gate check`, `handoff` | `systemMessage` → you, plus `reason` → Claude on a FAIL | gate FAIL, under the block budget |
| SubagentStop | the registered verifier names | `subagent-stop` | `report ingest` | `reason` of the blocking decision | envelope parse failure; trust failure |

Exit 2 is what blocks, and only on `PreToolUse`, `UserPromptSubmit`, `UserPromptExpansion`, `Stop`, and `SubagentStop`. On every other event no exit code blocks and the hook's JSON is the only channel to a reader. Exit 1 blocks nothing anywhere.

### 10.1 PreToolUse: a deny

Writing a document your current phase does not own:

```
$ echo '{"tool_name":"Write","tool_input":{"file_path":".devforgeai/stories/STORY-001.md","content":"..."}}' \
    | devforgeai hook run pre-tool-use
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"DFA-E212 phase 'explore' does not write story; 'planning-work' does"}}
```

**reproduced**, exit 2. You see the `permissionDecisionReason` as the refusal text. A `permissionDecision: deny` fires in every permission mode, `bypassPermissions` included, and no other hook's allow can override it.

The same rule reaches a shell redirection, because a `PostToolUse` hook matching `Write|Edit` does not fire for a `cat >`:

```
$ echo '{"tool_name":"Bash","tool_input":{"command":"cat > .devforgeai/stories/STORY-002.md <<EOF"}}' \
    | devforgeai hook run pre-tool-use
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"devforgeai: this command writes under .devforgeai/ from a phase that does not own the document.\nDFA-E212 phase 'explore' does not write story; 'planning-work' does"}}
```

**reproduced**, exit 2. The recognised write forms are a `>` or `>>` redirection (a heredoc included) and the tokens `tee`, `Out-File`, `Set-Content`, `Add-Content`, `New-Item`, `Copy-Item`, `Move-Item`, `cp`, `mv`. A construct the scan does not recognise still lands — and is caught by the Stop-time document scan instead (§10.4).

Two analysis agents are held to one configured command each:

```
$ echo '{"tool_name":"Bash","agent_type":"code-quality-auditor","tool_input":{"command":"radon cc -s src"}}' \
    | devforgeai hook run pre-tool-use
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"devforgeai: code-quality-auditor may run only the command config.toml records at [verify].metrics_command. It asked to run 'radon cc -s src'. Set that key, or have the agent run what it names."}}
```

**reproduced**, exit 2. The refusal names the `config.toml` key to edit, so the person who owns the project's tooling can act on it. `dead-code-detector` is held to `[verify].call_graph_command` the same way. When the command matches, the hook returns `permissionDecision: allow` — these agents carry no other shell grant, so the allow is what lets the one command through.

Writes outside the project root are allowed with no output, with two exceptions: the trust store and the pinned framework tree are refused in every case, and during a Build run with an active story every path outside the project is outside the declared file set by construction.

### 10.2 UserPromptExpansion: a refusal

Typing a phase command whose predecessor gate has not passed:

```
$ echo '{"command_name":"constitute","command_args":"IDEA-001"}' | devforgeai hook run prompt-expansion
{"decision":"block","reason":"DFA-E321 constitute needs discover gate PASS for IDEA-001; no report at .devforgeai/reports/IDEA-001-discover.yaml; to repair, run /discover IDEA-001, then /constitute IDEA-001\nThe predecessor gate for /constitute IDEA-001 has not passed, so the skill was not loaded."}
```

**reproduced**, exit 2. The skill body does not load, so that `reason` is the whole of what you get — which is why it names the command that repairs the state, not only the diagnostic.

A pass emits nothing and the expansion proceeds untouched:

```
$ echo '{"command_name":"discover","command_args":"IDEA-001"}' | devforgeai hook run prompt-expansion
```

**reproduced**, no output, exit 0.

The arm stays quiet on invocations that carry no resolvable subject — `/explore "<idea>"`, `/design --sketch …`, `/reflect --since <date>`, a bare `/reflect`. Blocking there would refuse runs that legitimately carry no id, and the skill's own preamble raises the usage error where it belongs. `explore` is absent from the gate table because it has no predecessor; `design` and `reflect` because they are cross-cutting.

### 10.3 PostToolUse: an annotation

This event honours no exit code. Its stdout and stderr reach the debug log alone, so `additionalContext` is the only channel that puts a diagnostic in front of Claude:

```
$ echo '{"tool_name":"Write","tool_input":{"file_path":".devforgeai/explore/brief.md"}}' \
    | devforgeai hook run post-tool-use
{"hookSpecificOutput":{"hookEventName":"PostToolUse","additionalContext":"devforgeai doc validate: DFA-W202 .devforgeai/explore/brief.md cites FLOW-001, which consumes does not list\n..."}}
```

**reproduced**, exit 0. The write stands; the diagnostic names the key or section to rewrite, and Claude rewrites it on the next turn. Nothing to say emits nothing at all.

### 10.4 Stop: the four cases

The Stop dispatcher scans every document written since the last scan, runs the producer check on each, runs the gate, and writes one JSON object to stdout and nothing else.

The scan is the enforcing half of the shell-write guard: whatever wrote the file, the file is there and its producer is checked. A refusal makes the result FAIL whatever the gate said, and appears in the `reason` as `DFA-E212 <path>: <message>`.

**Case A — PASS.** Exit 0, one key:

```json
{"systemMessage":"Phase     0 · Explore         IDEA-001 · nightly-bank-reconciliat\nDone      1 ideas · 0 flows\nGate      PASS  12 checks\n..."}
```

**Case B — FAIL, inside the block budget.** Exit 2, one object carrying both keys. `reason` is addressed to Claude and names the checks; `systemMessage` is addressed to you and is the handoff block:

```json
{"decision":"block","reason":"Gate      explore · IDEA-001\nResult    FAIL\nChecks    decision-exists DFA-E323 found 0 of 1 paths; missing .devforgeai/explore/decision.yaml; ...\nFix the failing checks and stop again; the handoff prints when the turn ends.","systemMessage":"Phase     0 · Explore         IDEA-001 · -\nDone      0 ideas · 0 flows\nGate      FAIL  decision-exists DFA-E323 found 0 of 1 paths; missing...\n\nNext      /explore IDEA-001\nBlocked   none\n\nFull report: .devforgeai/reports/IDEA-001-explore.yaml"}
```

**reproduced**, exit 2. The same `reason` text also goes to stderr, so a schema change upstream degrades to the stderr path rather than to silence.

**Case C — FAIL, budget spent.** Exit 0 with the FAIL block in `systemMessage` and no blocking decision. The budget is **three consecutive blocks**, keyed on `session_id` alone. A changed session is a different chain; a changed phase or subject is not, because the subject is exactly the thing a continuation can change. Claude Code's own ceiling is eight consecutive blocks, at which it overrides the hook and shows nothing — so the framework's budget ends the chain first and the last Stop inside it is the one that renders the handoff.

**Case D — SEND BACK.** Exit 0, `systemMessage` alone, no blocking decision.

### 10.5 SubagentStop

A registered verifier's final message is read as one `devforgeai/verifier/1` envelope and ingested into the report. An envelope that does not parse blocks with:

```
devforgeai could not ingest your report: DFA-E410 ...
Re-emit your final message as one devforgeai/verifier/1 envelope and nothing else.
```

read from `cli/src/hooks/run.rs`. The `reason` reaches the subagent as its next instruction, so the envelope is asked for again rather than logged and lost — the gate would otherwise evaluate `verifier_pass` against a block of zeros.

---

## 11. Brownfield install

`devforgeai init --analyze` runs `stack detect`, then reads the existing tree and drafts the six context files at `status: draft` for `/constitute` to finish.

```
$ devforgeai init --analyze --from 'C:\Projects\DevForgeAI'
Initialised .devforgeai/ in C:\Users\bryan\AppData\Local\Temp\...\brown
Copied     102 skills, 46 agents
Commands   /build, /constitute, /design, /discover, /explore, /plan, /reflect, /release, /verify
Hooks      .claude/settings.json merged; git hooks pre-commit, commit-msg, pre-push
CLAUDE.md  devforgeai section written
Stack      node
Analysis   3 context files drafted, 3 stubbed, 145 files scanned
Next       /explore
```

**reproduced** in a repository holding a `package.json`, `src/domain/`, `src/api/`, and `tests/`. Exit 0.

`stack detect` wrote:

```toml
[[stack]]
id = "node"
markers = ["package.json"]
package_manager = "npm"
source_roots = ["src"]
test_command = "npm run test"
coverage_command = ""
coverage_format = "lcov"
coverage_paths = ["coverage/lcov.info"]
lint_command = "npm run lint"
complexity_command = ""
timeout_secs = 900
```

**reproduced**, and `degraded = false` — so the command-running gate checks now actually run, and the compiled coverage floors apply.

The drafted `tech-stack.md` opens:

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
```

**reproduced.**

Three files are *drafted* from what the analysis found; three are *stubbed*, because a constraint index and an anti-pattern index cannot be read off a source tree. `/constitute IDEA-nnn` finds `tech-stack.md` at `status: draft`, takes the brownfield branch, invokes `source-tree-mapper` over the real tree, and finishes all six.

Until they are accepted, `context audit` refuses:

```
$ devforgeai context audit
context audit  6/6 files · 0 constraints · 0 anti-patterns · CA-2, CA-3 failed
devforgeai: DFA-E223 context audit: .devforgeai/context/tech-stack.md is status 'draft' with 0 open questions; context audit needs accepted and []
```

**reproduced**, exit 1. The analysis has a 120-second cap; reaching it emits `DFA-E113` at exit 0 rather than failing the install.

`--analyze` on a project that already has `.devforgeai/` needs `--force`, which overwrites `.devforgeai/` and existing git hooks:

```
$ devforgeai init --from 'C:\Projects\DevForgeAI'
devforgeai: DFA-E110 init: project is already initialised; pass --force to overwrite .devforgeai/
```

**reproduced**, exit 1. The CLAUDE.md section is not covered by `--force`: it is replaced between its markers on every run, and nothing outside them can be lost.

---

## 12. Git hooks

`init` writes three, and `devforgeai hook install --git-only` rewrites them. Each begins with `trust verify` and exits 4 if it fails.

| Hook | Runs | Refuses when |
|---|---|---|
| `pre-commit` | `doc validate` on every staged file under `.devforgeai/`, then `context audit` when `.devforgeai/context/` exists | any of them fails |
| `commit-msg` | looks for a `STORY-nnn` or `ADR-nnn` token in the message | the token is absent |
| `pre-push` | `gate check --phase build` for every story in `sprint.yaml` at status `building` | any gate FAILs |

The `pre-commit` hook as written:

```sh
#!/bin/sh
# devforgeai-hook v1
DFA="C:/Projects/DevForgeAI/cli/target/release/devforgeai.exe"
"$DFA" trust verify || exit 4

git_dir=$(git rev-parse --git-dir) || exit 1
[ -n "$git_dir" ] || exit 1
list="$git_dir/devforgeai-staged"
git diff --cached --name-only --diff-filter=ACM -- .devforgeai > "$list" || exit 1

status=0
while IFS= read -r f; do
  [ -n "$f" ] || continue
  "$DFA" doc validate "$f" || status=1
done < "$list"
rm -f "$list"

if [ -d .devforgeai/context ]; then
  "$DFA" context audit || status=1
fi

exit $status
```

**reproduced** from `.git/hooks/pre-commit`. The binary path is the one the pin recorded.

A hook that exists and this binary did not write is left alone, with `DFA-E132` naming it, unless you pass `--force`. A project with no `.git/` gets `DFA-E130` at exit 0 — no git directory is not an install failure.

---

## 13. Trust

### What the pin protects

`~/.devforgeai/trust.toml` records the SHA-256 of the binary, the digest of the `cli/` source tree, and the framework path. `trust verify` compares the running binary against the pinned digest, and refuses when the `cli/` source digest differs from the pinned `REVISION` while a Claude session is active.

The point is that every gate, every producer check, and every handoff is computed by that binary. A session that could swap it could grant itself anything. So the pin is made by a human, in a terminal, outside Claude Code, and the two guards in §3.1 keep it there.

### Fail-closed: what you see without a pin

Three surfaces, one message. An unpinned session can read and reason and can change nothing.

```
$ devforgeai trust verify
devforgeai: DFA-E502 trust verify: no pin for C:\...\unpinned\devforgeai.exe in trust.toml
  at C:\Users\bryan\.devforgeai\trust.toml:0
```

**reproduced** by running a copy of the binary from an unpinned path, exit 4.

**PreToolUse `trust-check`** — denies every `Write`, `Edit`, `NotebookEdit`, `Bash`, `PowerShell`, and `Agent` call:

```json
{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"devforgeai trust verify failed: DFA-E502 no pin for C:\\...\\unpinned\\devforgeai.exe in trust.toml. In a terminal outside Claude Code, run: devforgeai trust pin --framework C:\\Projects\\DevForgeAI. Then start a new session."}}
```

**reproduced**, exit 2.

**UserPromptExpansion** — refuses each of the nine phase commands before the skill body enters context:

```json
{"decision":"block","reason":"devforgeai trust verify failed: DFA-E502 no pin for ... In a terminal outside Claude Code, run: devforgeai trust pin --framework C:\\Projects\\DevForgeAI. Then start a new session."}
```

**reproduced**, exit 2.

**Stop** — blocks and runs no gate:

```json
{"decision":"block","reason":"devforgeai trust verify failed: DFA-E502 ... Then start a new session. No gate ran for this turn.","systemMessage":"Gate      TRUST FAIL  DFA-E502\nBlocked   you: run devforgeai trust pin --framework C:\\Projects\\DevForgeAI outside Claude Code\n\nNo gate ran for this turn."}
```

**reproduced**, exit 2. The rendered `systemMessage`:

```
Gate      TRUST FAIL  DFA-E502
Blocked   you: run devforgeai trust pin --framework C:\Projects\DevForgeAI outside Claude Code

No gate ran for this turn.
```

On a continuation (`stop_hook_active: true`) the same hook exits 0 with `systemMessage` alone and no blocking decision — a continuation cannot repair a pin, so the turn ends holding the refusal rather than looping against it. **reproduced.**

`SessionStart` and `PostToolUse` honour no exit code, so a trust failure there reaches you through `systemMessage`:

```json
{"systemMessage":"devforgeai trust verify failed: DFA-E502 no pin for ... Then start a new session."}
```

**reproduced**, exit 4.

`state.toml` `[last_gate].result` takes `TRUST_FAIL`, but that is a record rather than the channel the refusal depends on — the binary under suspicion is the one that renders the handoff.

### After you rebuild the binary

A rebuild changes the binary's SHA-256, so the running binary no longer matches the pin and `trust verify` fails with `DFA-E503`:

```
binary digest <actual> does not match the pin <expected>
```

read from `cli/src/errors_table.rs`, exit 4. Editing `cli/` source during a session gives `DFA-E504` instead: `cli/ source digest <actual> differs from the pinned REVISION <expected> while a Claude session is active`.

The repair is the same in both cases, and every refusal above names it: close the session, open a plain terminal, re-run `trust pin --framework <framework root>`, start a new session.

### `trust digest`

A hidden build-time subcommand. It prints the three values a release writes into `cli/REVISION` (two lines) and `cli/DIGEST` (one):

```
$ devforgeai trust digest --framework 'C:\Projects\DevForgeAI'
REVISION  71243ef898c8932499ae2b07779889f34863d8ee
SOURCE    sha256:8430bacff6f7970352ab26fffdbe4bdf786849a2b891ff8b66582b31e8835a98
DIGEST    sha256:865e5d7cc15f63e13944b903337e5c29893c7d5786c81cbf420f96e3efe540cd
```

**reproduced**, exit 0. Use it rather than an external script: the tree walk it performs is the one `trust verify` reads the files back with, and a separate implementation drifts the first time an exclusion changes, surfacing only as `DFA-E504` refusing every write.

`trust verify --json` reports the comparison in full:

```json
{"schema":"devforgeai/cli-json/1","command":"trust verify","ok":true,"exit":0,"degraded":false,"project":"","at":"2026-09-11T19:02:43Z","data":{"binary":"C:\\Projects\\DevForgeAI\\cli\\target\\release\\devforgeai.exe","digest":"sha256:865e5d...","pinned":"sha256:865e5d...","match":true,"session_active":true,"source_digest_checked":true},"errors":[],"warnings":[]}
```

**reproduced**, exit 0.

---

## 14. Customising `gates.toml` and `config.toml`

Both files are yours to edit. The binary carries compiled floors that a project can raise and cannot lower. Values are rejected, not clamped, so an edited file fails loudly rather than behaving differently from what it says.

### The floors

| Phase | Required check kinds | Numeric floors |
|---|---|---|
| explore | `file_exists`, `field_in_enum` | — |
| discover | `fields_present`, `ids_resolve` | — |
| constitute | `context_audit` | — |
| plan | `doc_valid`, `story_valid`, `ids_resolve` | — |
| build | `doc_valid`, `tests_pass`, `coverage_min` | `config.toml`: domain ≥ 90.0, application ≥ 80.0, infrastructure ≥ 70.0, interface ≥ 60.0, `[coverage].overall_min` ≥ 75.0 |
| verify | `doc_valid`, `verifier_pass` | `gates.toml`: `verifier_pass.min_ratio` ≥ 1.0 |
| release | `doc_valid`, `file_exists` | — |
| reflect | `file_exists`, `doc_valid`, `yaml_cites`, `no_threshold_decrease` | — |

A required kind present with `severity = "warn"` is treated as absent. A `[[layer]]` table deleted from `config.toml` keeps its default threshold, so removing a layer lowers nothing.

Lowering the verify gate's `min_ratio`:

```
$ devforgeai gate check --phase verify --id STORY-001
devforgeai: DFA-E303 gate check: gates.toml gate 'verify' sets verifier_pass.min_ratio to 0.5, below the compiled minimum 1

$ devforgeai gate require verify STORY-001
devforgeai: DFA-E303 gate require: gates.toml gate 'verify' sets verifier_pass.min_ratio to 0.5, below the compiled minimum 1
```

**reproduced**, both exit 1. `gates.toml` is validated by `gate require`, `gate check`, and `phase set` before anything else runs.

Lowering a layer's coverage floor in `config.toml`:

```
$ devforgeai gate check --phase build --id STORY-001 --no-run
devforgeai: DFA-E303 gate check: config.toml sets layer.domain.coverage_min to 80, below the compiled minimum 90

$ devforgeai config get stack.test_command
devforgeai: DFA-E303 config get: config.toml sets layer.domain.coverage_min to 80, below the compiled minimum 90
```

**reproduced**, both exit 1. This check applies while `degraded = false`; with the flag set the command checks are skipped, so the numbers govern nothing and are left alone.

The default `gates.toml` satisfies the floors exactly, with no margin.

### What you can safely change

| File | Key | Effect |
|---|---|---|
| `config.toml` | `[explore] timebox_days`, `remedy_timebox_days` | the `time-box` gate check's limit; 5 and 1 by default |
| `config.toml` | `[plan] sprint_capacity_points`, `story_points` | sprint capacity and the point ladder; 20 and `[1,2,3,5,8]` |
| `config.toml` | `[build] worktree_root`, `branch_prefix`, `base_ref`, `complexity_max`, `duplication_max_percent` | where Build's worktrees live and what opens a refactor pass |
| `config.toml` | `[verify] mode` | `light` or `deep` on every `/verify` that carries no `--deep` |
| `config.toml` | `[verify] metrics_command`, `call_graph_command` | the one command each of the two analysis agents may run; empty means a Grep fallback |
| `config.toml` | `[release] platform`, `ci`, `deploy_root`, `docs_root`, `image_name`, `service_port`, `api_symbols_command`, `artifact_paths`, `build_command`, `package_command` | everything Release would otherwise have to name |
| `config.toml` | `[[layer]] globs` | which paths belong to which layer for coverage |
| `config.toml` | `[frontend] globs`, `exclude`, `tokens_path` | which files `design lint` checks against the token set |
| `config.toml` | `[reflect] session_root`, `session_key`, `window_days` | where Reflect reads session history |
| `gates.toml` | any `[[gate.check]]` you add, or any threshold you raise | a stricter gate |
| `gates.toml` | `cli_min_version` | `DFA-E304` when a binary older than the value tries to read the file |

Adding a check kind the binary does not know is `DFA-E301`, naming the closed list. Adding a key a kind does not define is `DFA-E302`. Defining a phase or a check id twice is `DFA-E305`.

### Degradation

`config.toml` carries one boolean, `degraded`, set by `stack detect` — `true` when `[[stack]]` is empty, `false` otherwise.

- `degraded = false`: a `tests_pass`, `coverage_min`, or `lint_clean` check whose command string is `""` fails with `DFA-E310`, `DFA-E312`, or `DFA-E314`.
- `degraded = true`: the four command-running kinds evaluate to `skip`, count as passing, and the build floors do not apply.

Every skip shows as `SKIP (degraded)` in the human output and `{"status":"skip","reason":"degraded"}` in `--json`.

---

## 15. Troubleshooting by error code

Every message is `devforgeai: <code> <subcommand>: <message>` on stderr, optionally followed by an indented `at <path>:<line>`. Exit codes: **0** pass, **1** fail, **2** send-back required, **3** usage error, **4** trust violation, **5** internal error.

### Getting started

| Code | Exit | Printed message | Do this |
|---|---|---|---|
| `DFA-E030` | 3 | `no .devforgeai/ directory in <cwd> or any ancestor; run 'devforgeai init'` | run from inside the project, or pass `--project <path>` |
| `DFA-E110` | 1 | `project is already initialised; pass --force to overwrite .devforgeai/` | the project is installed; pass `--force` only to start over |
| `DFA-E111` | 1 | `framework source not found; pass --from <path> to the DevForgeAI repo root` | pass `--from`, or pin the binary so `init` can default to the recorded path |
| `DFA-E101` | 1 | `.devforgeai/config.toml not found; run 'devforgeai stack detect'` | run `devforgeai stack detect` |
| `DFA-E102` / `DFA-E103` | 1 | `.devforgeai/gates.toml not found` / `state.toml not found; run 'devforgeai init'` | re-run `init --force`, or restore the file from git |
| `DFA-E130` | 0 | `no .git directory; git hooks not installed` | `git init`, then `devforgeai hook install --git-only` |

### Trust

| Code | Exit | Printed message | Do this |
|---|---|---|---|
| `DFA-E500` | 4 | `trust pin does not run inside a Claude session; <var> is set` | run the pin in a plain terminal |
| `DFA-E501` | 4 | `<home>/trust.toml not found; a human runs 'devforgeai trust pin' outside Claude` | make the first pin (§3.1) |
| `DFA-E502` | 4 | `no pin for <path> in trust.toml` | the binary you are running is not the one pinned; pin this path, or run the pinned one |
| `DFA-E503` | 4 | `binary digest <actual> does not match the pin <expected>` | the binary was rebuilt; re-pin outside Claude Code |
| `DFA-E504` | 4 | `cli/ source digest <actual> differs from the pinned REVISION <expected> while a Claude session is active` | `cli/` source changed; re-pin outside Claude Code |
| `DFA-E510` | 4 | `<path> is missing` / `<path> is malformed; release builds write it` | `cli/DIGEST` or `cli/REVISION` is absent; run `devforgeai trust digest --framework <root>` and write both files |

### Gates and phases

| Code | Exit | Printed message | Do this |
|---|---|---|---|
| `DFA-E321` | 1 | `<phase> needs <pred> gate PASS for <id>; no report at <path>` / `... the last result is <result> at <time>` | the message names the repair command; run it |
| `DFA-E320` | 1 | `phase '<phase>' needs gate '<pred>' PASS for <id>; the last result is <result>` | same; `phase set` refuses for the same reason `gate require` does |
| `DFA-E300` | 1 | `gates.toml has no gate for phase '<phase>'` | `design` legitimately has no gate; otherwise `gates.toml` lost a block |
| `DFA-E303` | 1 | `gates.toml gate '<phase>' sets <kind>.<key> to <value>, below the compiled minimum <floor>` / `config.toml sets <key> to <value>, below the compiled minimum <floor>` | raise the value back to the floor (§14) |
| `DFA-E304` | 1 | `gates.toml requires devforgeai <value> or newer; this binary is <version>` | build a newer binary, or lower `cli_min_version` |
| `DFA-E310` / `DFA-E314` | 1 | `stack '<id>' has no test_command` / `no lint_command; set one in config.toml` | set the command, or re-run `stack detect` |
| `DFA-E311` / `DFA-E315` | 1 | `test command '<cmd>' exited <code> for stack '<id>'` / `lint command ...` | the project's own tests or linter failed; fix the code |
| `DFA-E313` | 1 | `coverage for layer '<name>' is <actual>%, below the <min>% in config.toml` | add tests, or raise coverage in that layer |
| `DFA-E316` | 1 | `report <path> has no verifiers block for '<name>'` | that verifier did not run, or its envelope did not parse; re-run the phase |
| `DFA-E317` | 1 | `verifier '<name>' passed <p>/<t>, below min_ratio <r>` | the findings are in the report; repair them |
| `DFA-E318` | 1 | `command '<cmd>' exceeded <n>s and was terminated` | raise `[[stack]].timeout_secs` |

### Documents

| Code | Exit | Printed message | Do this |
|---|---|---|---|
| `DFA-E200` | 1 | `<path> not found` | the upstream document is not there; run the phase that writes it |
| `DFA-E201` | 1 | `<path> has no frontmatter; line 1 opens with three dashes` | the file lost its envelope |
| `DFA-E203` / `DFA-E204` / `DFA-E205` | 1 | `frontmatter has no key '<key>'` / `has key '<key>', which conventions section 5 does not define` / `key '<key>' is at position <n>; conventions section 5 places it at <m>` | the seven keys, in order, nothing else |
| `DFA-E208` | 1 | `<path> status is '<value>'; <doc-type> allows <a, b, c>` | use a value from the enum |
| `DFA-E209` | 1 | `<path> id '<value>' is malformed` / `duplicates <other-path>` | ids are `PREFIX-nnn`, three digits, allocated once |
| `DFA-E210` | 1 | `<path> references <ID>, which no document defines` | the cited id is not defined anywhere; allocate it, or cite one that exists |
| `DFA-E211` / `DFA-E212` | 1 | `<path> produced_by is '<value>'; <doc-type> is produced by '<skill>'` / `phase '<current>' does not write <doc-type>; '<skill>' does` | this is the producer check; switch phase, or write a document this phase owns |
| `DFA-E213` | 1 | `<path> heading '<text>' is absent` / `is at position <n>, expected <m>` | sections appear in template order with the template's heading text |
| `DFA-E214` | 3 | `'<prefix>' is not an ID prefix; conventions section 5 defines <list>` | use a prefix from §4.1 |
| `DFA-E215` | 1 | `prefix <prefix> has no free ID below 999` | the prefix is exhausted |
| `DFA-E013` | 3 | `'<value>' is not an ID; expected PREFIX-nnn with three digits, or it contradicts the id the document on disk carries` | check the id on the command line against the one in the file |
| `DFA-W201` / `DFA-W202` | 0 | `<path> consumes <ID>, which the body does not cite` / `cites <ID>, which consumes does not list` | warnings; the write stands |

### Stories, context, tokens

| Code | Exit | Printed message | Do this |
|---|---|---|---|
| `DFA-E223` | 1 | `<path> is status '<value>' with <n> open questions; context audit needs accepted and []` | finish `/constitute` and answer its open questions |
| `DFA-E230` | 1 | `<STORY-nnn> <AC-nnn> has no testable predicate; the grammar is Given/When/Then or a single assertion line` | rewrite the criterion |
| `DFA-E231` | 1 | `<STORY-nnn> consumes <REQ-nnn>, which requirements.yaml does not define` | the story cites a requirement that is not there |
| `DFA-E232` | 1 | `dependency cycle: <A> -> <B> -> <A>` | break the cycle in `## Dependencies` |
| `DFA-E237` | 1 | `<STORY-nnn> and <STORY-nnn> both declare <path>` | two stories claim one file; move it to one of them |
| `DFA-E239` | 1 | `<path> is outside the declared file set of <STORY-nnn>` | add the path to the story's `## Files` table, through `/plan --remedy` |
| `DFA-E240` / `DFA-E241` | 1 | `<path>:<line> uses the literal colour '<value>'; brand/tokens.json defines <nearest-token>` | use the token the message names |
| `DFA-E242` / `DFA-E244` | 1 | `<path>:<line> references token '<name>', which brand/tokens.json does not define` | the token does not exist; add it in `/design --brand`, or use one that does |
| `DFA-E272` | 1 | `<STORY-nnn> and <STORY-nnn> both declare <path>; one worktree at a time` | finish or remove the other story's worktree |
| `DFA-E273` | 1 | `<path> holds <n> uncommitted changes and <m> commits absent from <base_ref>; pass --force` | commit or merge first |

### Reports and hooks

| Code | Exit | Printed message | Do this |
|---|---|---|---|
| `DFA-E400` | 1 | `.devforgeai/reports/<id>-<phase>.yaml not found` | the phase has not run its gate yet |
| `DFA-E401` | 1 | `<path> is not valid YAML: <parser message>` | delete the report and re-run the gate |
| `DFA-E410` | 0 | `subagent '<name>' output is not devforgeai/verifier/1: <parser message>` | the SubagentStop hook asks the agent again |
| `DFA-E430` | 3 | `pass one id (IDEA-nnn, EPIC-nnn, STORY-nnn, vX.Y.Z) or --since <YYYY-MM-DD>` | `/reflect` takes exactly one of the two |
| `DFA-E020` / `DFA-E021` | 3 | `hook input has no key '<key>' for event <event>` / `stdin is not JSON: <parser message>` | you are running `hook run` by hand; give it the payload the event carries |
| `DFA-E902` | 5 | `check kind '<kind>' is specified and not implemented in this build` | the kind is in the spec and stubbed in this binary |

---

## 16. Running the eval suite

Evals run **from the framework repository**, not from a target project: `--skill skills/<name>` is a path there, and a target holds `.claude/skills/<name>/` with no `evals/` in it and no runner.

```
cd C:\Projects\DevForgeAI
python evals/runner/run_jsonl.py --skill skills/exploring-ideas --preflight
python evals/runner/run_jsonl.py --skill skills/exploring-ideas
python evals/runner/run_jsonl.py --skill skills/implementing-stories --case BLD-03 --keep
```

Run `--preflight` first. It runs each case's declared `preflight` CLI calls against a throwaway copy of its materialised workspace and requires exit 0 from every one. That catches a fixture missing a `state.toml` field, a missing predecessor report, or a missing `--epic` before a paid run — six of seven failures in the first end-to-end pass would have been caught by it. A `preflight` entry may be a bare string (exit 0 required) or an object carrying `exit`, `code` (a `DFA-` code the call has to raise), or `forbid_code`.

`--dry-run` materialises the workspaces, prints the invocation it would make, and keeps them for inspection.

The runner builds one temporary workspace per case, installs the skill under test at `.claude/skills/<its slash name>/` with the agents it names, puts the release binary on `PATH`, writes the framework's hook block into `<workspace>/.claude/settings.json`, runs `claude -p` there with the prompt on stdin and `stream-json --verbose` output, then calls the case's grader from the skill's `graders.py`.

| Argument | Default | Means |
|---|---|---|
| `--skill <path>` | — | the skill directory; `--cases`, `--graders` and `--out` default beside it |
| `--cases <path>` | `<skill>/evals/cases.jsonl` | the case file |
| `--graders <path>` | `graders.py` beside the cases | the grader module |
| `--out <path>` | `results.jsonl` beside the cases | appended one `devforgeai/eval-result/2` line per case |
| `--model` | `sonnet` | the model under test |
| `--timeout` | `900` | seconds; above the 828 a measured full-phase case took |
| `--jobs` | `1` | parallel cases |
| `--filter <glob>` | `*` | case id glob |
| `--case <id>` | — | repeatable, one case |
| `--workdir`, `--logdir`, `--keep` | — | where workspaces and logs go, and whether they survive |
| `--claude-bin`, `--devforgeai-bin`, `--framework-root` | — | path overrides |
| `--claude-config isolate\|inherit` | — | whether the child sees your Claude configuration |
| `--hooks` / `--no-hooks` | hooks on | whether the framework's hooks enforce inside the workspace |
| `--dry-run`, `--preflight` | off | the two cheap modes above |

**A case may carry its own `timeout` key, which overrides `--timeout` for that case.** The Constitute cases carry 1500, because that phase writes six or more documents.

**Hooks-on needs the pin.** A release binary reads `~/.devforgeai/trust.toml` and honours no `DEVFORGEAI_HOME` redirect, so nothing can stand in for a real `devforgeai trust pin` made by a human outside Claude Code. The runner checks `trust verify` once before a suite and falls back to `--no-hooks` with the pin command printed when it fails; passing `--hooks` explicitly makes it refuse to start instead. Without the pin, every eval measures a skill with the framework's hooks off — which means the producer check, the gates, and the Stop block are not part of the measurement.

A case that seeds `answers` runs with `evals/runner/permission_host.py` as a stdio MCP server and `--permission-prompt-tool mcp__dfa-permissions__approve`, because measured on `claude 2.1.268` the `AskUserQuestion` tool is in the tool set only when a permission host is supplied: no host lists 33 tools and no `AskUserQuestion`, the host lists 36 including it. A case that seeds no answers runs under `--permission-prompts none`, so a question the case was not written to reach is denied rather than left hanging. `--permission-mode` is `bypassPermissions` inside the throwaway workspace.

Statuses are `pass`, `fail`, `error`, `timeout`, and `limit` — `limit` being a run the account's usage limit stopped, which says nothing about the case. Exit codes: 0 every case passed, 1 a case failed, 2 a case errored, timed out, or hit the limit, 3 a usage error.

The runner's own tests use a fake `claude` on `PATH` and reach no network:

```
python -m unittest evals/runner/test_runner.py
```

---

## 17. Command reference

Global options on every subcommand: `--json` (one machine envelope on stdout, no human text), `--project <path>` (default: the nearest ancestor holding `.devforgeai/`; `init` defaults to the working directory), `--quiet` (suppress human stdout; stderr unaffected).

### Commands you run

| Command | Purpose |
|---|---|
| `devforgeai --version` | print `devforgeai <semver> (<revision>)` |
| `devforgeai init [--analyze] [--force] [--from <path>] [--no-hooks]` | create `.devforgeai/`, copy skills and agents, merge hooks, write the CLAUDE.md section, run `stack detect` |
| `devforgeai trust pin [--binary <path>] --framework <path>` | pin the binary's SHA-256; refuses inside a Claude session |
| `devforgeai trust verify [--binary <path>]` | verify the running binary against its pin |
| `devforgeai trust digest --framework <path> [--binary <path>]` | print the `REVISION`, `SOURCE` and `DIGEST` a release writes (hidden) |
| `devforgeai stack detect [--dry-run]` | write `config.toml` with the detected stacks |
| `devforgeai hook install [--force] [--claude-only] [--git-only]` | write the Claude hooks and the three git hooks |
| `devforgeai handoff [--phase <phase>] [--id <id>]` | print the handoff block |
| `devforgeai report show <id> <phase> [--check <check-id>]` | print a gate report, or one check entry |
| `devforgeai report aggregate [<ID>] [--since <YYYY-MM-DD>] [--session-root <path>]` | the Reflect window |
| `devforgeai story list [--status <s>] [--sprint <id>]` | one line per story |
| `devforgeai worktree list` | one line per story worktree |
| `devforgeai worktree remove <id> [--force]` | remove a story's worktree |
| `devforgeai config get <key> [--stack <id>]` | print one configuration value |
| `devforgeai context audit` | the eight CA checks over the six context files |
| `devforgeai doc validate <paths...> [--all]` | frontmatter, ids, cross-references, status enum, producer match |
| `devforgeai gate check --phase <phase> [--id <id>] [--partial] [--no-run]` | evaluate a gate and write its report |
| `devforgeai gate require <phase> <id>` | exit 0 when the predecessor gate passed |

### Commands the skills, preambles and hooks run

| Command | Called by |
|---|---|
| `devforgeai doc validate --allocate <prefix>` | the workflow step that needs the id |
| `devforgeai doc validate --producer-check [--stdin-content]` | `PreToolUse`, and the Stop-time scan |
| `devforgeai doc load <name> <id>` | a skill's `!` preamble, and mid-workflow reads |
| `devforgeai doc accept requirements --id <IDEA-nnn>` | Discover step 14 |
| `devforgeai doc reopen requirements --id <IDEA-nnn> --ids <ID,ID> --from <phase>` | Discover step C2 |
| `devforgeai phase set <phase> --id <id> [--remedy <ID,ID>] [--epic <EPIC-nnn>]` | every phase; refuses unless `gate require` would pass |
| `devforgeai story validate [<id>] [--scope active\|sprint\|all]` | the Plan gate's `plan-stories` check |
| `devforgeai story files [--check <path>] [--list] [--diff] [--id <id>] [--base <ref>]` | `PreToolUse` during Build, and Build steps 9 and 11 |
| `devforgeai worktree ensure <id>` | Build step 4 |
| `devforgeai commit <id> -m <message> [--paths <p,p>]` | Build steps 7.3, 7.6, 7.8, 8, 10 |
| `devforgeai antipattern scan [--id <id>] [--paths <p>] [--min-severity <s>]` | Build step 10 |
| `devforgeai design lint [<paths>] [--tokens]` | `PreToolUse` on frontend files; Design step 14 |
| `devforgeai explore prune --id <IDEA-nnn>` | Explore step 11 |
| `devforgeai report ingest <subagent> <source> [--id <id>] [--phase <phase>]` | `SubagentStop` |
| `devforgeai report note <id> <phase> --key <key> --file <path>` | Build step 13 |
| `devforgeai hook run <event>` | the six registered Claude Code hook events |

`hook run` accepts these seven arms: `session-start`, `prompt-expansion`, `pre-tool-use`, `trust-check`, `post-tool-use`, `stop`, `subagent-stop`.

### `--help`

```
$ devforgeai --help
The enforcement core of the DevForgeAI framework

Usage: devforgeai.exe [OPTIONS] [COMMAND]

Commands:
  init         Create `.devforgeai/`, copy skills, merge hooks, detect the stack
  stack        Stack detection
  gate         Gate evaluation
  doc          Document operations
  story        Story operations
  explore      Explore-phase operations
  design       Design-token linting
  context      The six context files
  hook         Hook installation and dispatch
  report       Report operations
  trust        The binary trust pin
  phase        Phase state
  worktree     Git worktrees for concurrent stories
  config       Configuration reads
  antipattern  Anti-pattern detection
  handoff      Print the conventions section 6 handoff block
  commit       Commit the declared file set of a story
  help         Print this message or the help of the given subcommand(s)

Options:
      --json            One JSON envelope on stdout, no human text on stdout
      --project <path>  Project root override; `init` defaults to the working directory
      --quiet           Suppress human stdout; stderr is unaffected
      --version         Print `devforgeai <semver> (<revision>)` and exit 0
  -h, --help            Print help
```

**reproduced.**

---

## 18. Worked walkthroughs

One file per command, each a self-contained transcript with the real files and blocks.

| Command | Walkthrough |
|---|---|
| `/explore` | [examples/explore.md](examples/explore.md) |
| `/discover` | [examples/discover.md](examples/discover.md) |
| `/constitute` | [examples/constitute.md](examples/constitute.md) |
| `/plan` | [examples/plan.md](examples/plan.md) |
| `/build` | [examples/build.md](examples/build.md) |
| `/verify` | [examples/verify.md](examples/verify.md) |
| `/release` | [examples/release.md](examples/release.md) |
| `/design` | [examples/design.md](examples/design.md) |
| `/reflect` | [examples/reflect.md](examples/reflect.md) |
