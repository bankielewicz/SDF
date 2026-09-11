---
name: design
description: Draws wireframe screens, builds the brand token set, and writes UI specifications. The DevForgeAI design capability, run by /design in three modes. --sketch draws wireframe HTML screens for FLOW-nnn flows and returns the sketch-mode JSON that Explore step 6 consumes. --brand turns a brand sketch, the PERSONA-nnn records, and five fixed questions into brand/tokens.json, brand/logo.svg, and brand/brand-kit.md. --spec turns a STORY-nnn or EPIC-nnn into one ui-specs/UI-nnn.md per screen, each carrying states, breakpoints, eight accessibility rows, and every colour and type value as a TOKEN-name reference. Reach for it whenever /design is typed, whenever a wireframe, mockup, brand kit, design token, colour palette, type scale, logo mark, UI specification, screen anatomy, breakpoint, or accessibility row is being produced for this framework, and whenever UI-nnn, TOKEN-name, brand/tokens.json, or ui-specs/ appears in a story, a report, or a send-back.
argument-hint: '--sketch IDEA-nnn | --brand EPIC-nnn | --spec STORY-nnn | UI-nnn --remedy AC-nnn,...'
allowed-tools: Bash(devforgeai:*), PowerShell(devforgeai:*), Read, Write, Edit, Glob, Grep, Agent, AskUserQuestion, Skill
---

# designing-interfaces

Design is cross-cutting, not a phase. It has no number in the phase sequence, no gate in `gates.toml`, and no key in `state.toml` `[active]`. A run leaves `[current].phase` and `[current].id` as it found them.

## Entry

`/design`, one command with three modes.

| Argument form | Mode | `$1` |
|---|---|---|
| `--sketch IDEA-nnn`, or a Skill invocation carrying `"mode": "sketch"` | sketch | `IDEA-nnn` |
| `--brand EPIC-nnn` | brand | `EPIC-nnn` |
| `--spec STORY-nnn`, `--spec EPIC-nnn`, or a bare `STORY-nnn` / `EPIC-nnn` | spec | `STORY-nnn` or `EPIC-nnn` |
| `UI-nnn --remedy AC-nnn,TOKEN-name,UI-nnn` | spec, remedy run | `UI-nnn` |
| any of the above plus `--resume` | same mode, resume run | as above |

Spec mode is the default: any argument string holding none of `--sketch` and `--brand` is spec mode.

This skill runs no preamble command. Spec mode allocates its ids at step 12, one per screen that carries no `UI-nnn`. Sketch mode and brand mode allocate none. Allocation in a preamble runs before the mode is known, so a `--sketch` or `--brand` run would abort on an exhausted `UI` prefix it does not touch; step 12 is the point at which the run knows it needs an id.

Explore step 6 reaches sketch mode through the Skill tool with the request object rather than through `/design`, so that path carries no `$ARGUMENTS`. Sketch mode allocates no id in either case, and reads the request from `.devforgeai/explore/sketch-request.json`, which Explore wrote before the invocation.

Inputs each mode reads, by path:

| Input | Path | Mode |
|---|---|---|
| Sketch request | `.devforgeai/explore/sketch-request.json` | sketch |
| Explore brief | `.devforgeai/explore/brief.md` | all three |
| Explore seed data | `.devforgeai/explore/seed-data.json` | sketch, spec |
| Explore brand sketch | `.devforgeai/explore/mockups/brand-sketch.json` | brand |
| Requirements | `.devforgeai/requirements.yaml`, printed by `devforgeai doc load requirements -` | brand, spec |
| Story | `.devforgeai/stories/STORY-nnn.md`, printed by `devforgeai doc load story <STORY-nnn>` | spec |
| Frontend globs and token path | `.devforgeai/config.toml`, keys `[frontend].globs`, `[frontend].exclude`, `[frontend].tokens_path` | brand, spec |
| Brand tokens | `.devforgeai/brand/tokens.json` | spec |
| Prior UI spec | `.devforgeai/ui-specs/UI-nnn.md`, printed by `devforgeai doc load ui-spec <UI-nnn>` | spec remedy |
| Send-back report from Plan | `.devforgeai/reports/SPRINT-nnn-plan.yaml`, `findings[]` | spec remedy |
| Send-back report from Build | `.devforgeai/reports/STORY-nnn-build.yaml`, `findings[]` | spec remedy |
| Prior design report | `.devforgeai/reports/UI-nnn-design.yaml` at `status: send_back`, most recently modified | spec resume |
| Phase state | `.devforgeai/state.toml`, keys `[current].phase` and `[current].id` | all three |

Sketch mode reads no document produced by Discover, Constitute, or Plan: those phases have not run when Explore step 6 calls it. Spec mode reads `explore/brief.md` for the `## Core flows` table alone, and treats the file as absent when Discover ran from entry point B, where `requirements.yaml` carries `consumes: []` and no brief exists.

## Workflow

### 1. Establish the run — model

Read `$ARGUMENTS` and `.devforgeai/state.toml`. Settle three things: the mode from the table in `## Entry`, the run kind (fresh, remedy when `$ARGUMENTS` holds `--remedy`, resume when it holds `--resume`), and the subject id.

An id in `$ARGUMENTS` whose prefix is none of `IDEA`, `EPIC`, `STORY`, `UI` stops the run with one `Blocked` line naming those three accepted prefixes.

Then go to the mode's steps: sketch at step 2, brand at step 5, spec at step 10, a remedy run at step 15.

### Sketch mode — steps 2 to 4

Read `references/sketch.md` before this mode runs. It carries the request and return schemas, the screen naming rule, and the `out_dir` exemption.

**2. Read the request — model.** Read `.devforgeai/explore/sketch-request.json` and the `brief_path` it names. Take `flows[]`, `out_dir`, `seed_data_path`, `brand`, and `constraints` from it. When the file is absent, return `screens: []`, `uncovered_flows` holding every `flow_id` the invocation passed, and one `notes` line naming the missing path.

**3. Draw the screens — subagent `mockup-designer`.** Pass `flows[]`, `out_dir`, the rows of `.devforgeai/explore/seed-data.json`, `constraints.screens_per_flow_max`, and the `brand` object. The agent invokes the built-in `design` skill for layout and visual judgment and writes one HTML file per screen into `out_dir`. It returns `screens[]`, `brand_sketch`, `uncovered_flows`, `notes`. A flow whose steps produce no screen comes back in `uncovered_flows`, and the remaining flows keep their screens.

**4. Write the brand sketch — subagent `mockup-designer`.** Pass the `brand` object of the request. The agent writes `.devforgeai/explore/mockups/brand-sketch.json` holding exactly three keys at the top level of one flat object — `name`, `palette` (3 to 6 hex values), and `type_pair` (two family strings) — and the `brand_sketch` field of the return value points at it. This file has no template and carries no envelope: everything under `.devforgeai/explore/mockups/` sits outside `.devforgeai/` document validation, which `## Documents` records. A request carrying `brand: null` writes no file and leaves `brand_sketch` as `null`.

The mode's result is the sketch-mode return object: `mode`, `idea_id`, `screens`, `brand_sketch`, `uncovered_flows`, `notes`. It writes no `.devforgeai/brand/tokens.json` and no `.devforgeai/ui-specs/UI-nnn.md`, and it emits no send-back — an uncovered flow travels back as `uncovered_flows`, which Explore turns into one `open_questions` line per id.

### Brand mode — steps 5 to 9

Read `references/brand.md` before this mode runs. It carries the five questions, the answer-to-leaf mapping, the contrast rule, and the Figma fallback.

**5. Gather the input — model.** Read `.devforgeai/explore/mockups/brand-sketch.json` when it exists, the `personas[]` array of `requirements.yaml`, and the `epics[]` entry named by `$1`. Hold a candidate name, a candidate palette, a candidate type pair, and the persona goals the brand answers to. When `brand-sketch.json` is absent, the candidate name comes from the `epics[].title` of `$1`, and the candidate palette and type pair are the defaults in `templates/tokens.json`.

**6. Ask the five questions — user, AskUserQuestion.** Show the step 5 candidates as one line of prose above the first call. Two calls, with the questions, headers, and option labels fixed in `templates/brand-questions.md`: four questions in call 1 (`Tone`, `Color`, `Type`, `Density`), one in call 2 (`Logo`). A free-text answer matching no label re-asks that one question once, then the run takes the first option of that question and adds one `open_questions` line to `brand-kit.md` naming the header.

**7. Write the tokens — subagent `brand-designer`.** Pass the five answers, the step 5 candidates, and the answer-to-leaf mapping table in `templates/brand-questions.md`. The agent invokes the built-in `frontend-design:frontend-design` skill for palette and type judgment and writes `.devforgeai/brand/tokens.json` with `meta.status: draft`, every leaf of the six groups filled, each `color` leaf carrying `light` and `dark`, every text leaf at contrast 4.5 or above on `bg` in both themes, and the border leaf at 3.0 or above. A returned palette whose `text` on `bg` ratio falls below 4.5 in either theme comes back with the `text` value darkened or lightened until it clears, in the `adjustments[]` array; write that adjustment into `brand-kit.md` `## Do not`.

**8. Draw the logo — subagent `brand-designer`.** Pass the `Logo` answer, the brand name, and the `color` group. The agent writes `.devforgeai/brand/logo.svg` in the shape `## Documents` fixes. On `logo_written: false` with a `reason` string, write a one-line wordmark SVG from `templates/logo.svg` with the brand name and the `TOKEN-color-text` light value, and continue.

**9. Write the brand kit and mirror it — model, `figma:figma-use` and `figma:figma-generate-library` skills.** From `tokens.json`, `logo.svg`, and the five answers, write `.devforgeai/brand/brand-kit.md` with all nine sections from `templates/brand-kit.md`, and move `tokens.json` `meta.status` to `approved`.

When the Figma plugin's tools are present in the session and authenticated, invoke the `figma:figma-use` and `figma:figma-generate-library` skills to write the six groups as a Figma variable collection named `<brand name> tokens` with a light and a dark mode, and write `## Figma` with `Status: mirrored` and the collection name. When the plugin's tools are absent, or a call returns an authentication error, write `## Figma` with `Status: not mirrored` and `Reason` set to `figma plugin not present` or `figma plugin not authenticated`; every other output of this step is unchanged and the run continues.

The PostToolUse hook runs `devforgeai doc validate` on the write and its result reaches the model afterwards as `hookSpecificOutput.additionalContext`, naming a section; rewrite that section.

### Spec mode — steps 10 to 14

Read `references/spec.md` before this mode runs. It carries the nine sections, the token-only rule, the coverage rule, and the parallel write.

**10. Load the subject — model, CLI.** The CLI prints the subject: `devforgeai doc load story <STORY-nnn>` when `$1` is a story, and `devforgeai doc load requirements -` in both cases; when `$1` is an epic, take the `epics[].requirements` list. Hold the `AC-nnn` rows, the `REQ-nnn` records they cite, the `PERSONA-nnn` records those requirements name, and the `UI-nnn` ids the story already references.

When `.devforgeai/brand/tokens.json` is absent, the mode writes no `UI-nnn.md` and the handoff carries `Blocked   you: run /design --brand <EPIC-nnn> first`. Every colour and type value in a `UI-nnn.md` is a `TOKEN-<name>`, so a spec written before the brand kit would reference names nothing defines. Brand mode before spec mode is the one ordering this skill imposes on itself.

**11. Audit coverage — subagent `requirement-coverage-auditor`.** On a `--resume` run, first read the most recently modified `.devforgeai/reports/UI-*-design.yaml` whose `status` is `send_back`, take the `UI-nnn` and `FLOW-nnn` ids from its `findings[]`, and narrow this step and step 13 to the screen behind those ids; every `UI-nnn.md` at `status: approved` keeps every byte.

Pass the screens the story names, the `REQ-nnn` records of step 10, the `## Core flows` table of `.devforgeai/explore/brief.md` when that file exists, and the `requirements[].source` values of `requirements.yaml`. The agent returns the `devforgeai/verifier/1` envelope: `passed` and `total` at the top level, and `payload.subject_id`, `payload.screens[]`, `payload.screens_without_req[]`, `payload.flows_without_req[]`, and `payload.covered` under it. The SubagentStop hook ingests that output and writes `.devforgeai/reports/UI-nnn-design.yaml` from it; the report is the hook's to write, not this skill's. An absent `brief.md` is the shape Discover entry point B leaves: `payload.flows_without_req` comes back `[]` and the audit runs on screens alone.

`payload.screens_without_req` or `payload.flows_without_req` non-empty is the send-back condition; see `## Send-back`.

**12. Allocate the ids — CLI.** The CLI runs `devforgeai doc validate --allocate UI` once per screen from step 11 that carries no existing `UI-nnn`, and its stdout gives one `UI-nnn` per screen, in `payload.screens[]` order. Exit 1 on `DFA-E215` means the prefix is exhausted; the run stops and the stderr line reaches the model.

**13. Write the UI specs — model, subagent `ui-spec-writer`, one invocation per screen, in parallel across screens.** When the story carries a Figma node URL and the Figma plugin's tools are present and authenticated, run the `figma:figma-design-to-code` skill in this conversation first, once per node, and hold what it returns as `figma_context`. A story with no node URL, an absent plugin, or an authentication error leaves `figma_context` as `""`. A skill cannot be invoked from inside a subagent's turn, which is why the design context is produced here and passed in.

Then pass one screen with its allocated `UI-nnn`, the `REQ-nnn` records it realizes, the `PERSONA-nnn` goal, the flattened token names of `.devforgeai/brand/tokens.json`, the matching mockup files under `.devforgeai/explore/mockups/` when they exist, the story's `figma_node_url` when the story carries one, and `figma_context`. The agent also takes `templates/ui-spec.md` as its `template_path`. It invokes the built-in `frontend-design:frontend-design` skill for component composition judgment and writes one `.devforgeai/ui-specs/UI-nnn.md` per screen from that template at `status: draft`, with all nine sections and the seven envelope keys as YAML frontmatter.

With `figma_context` empty, `## Anatomy` `Content source` cites the mockup path or the `REQ-nnn` in place of the node, and the file is otherwise unchanged.

**14. Resolve the tokens — CLI.** The CLI runs `devforgeai design lint --tokens`. On exit 0, move every file written at step 13 to `status: approved`. Exit 1 on `DFA-E244` names a `TOKEN-<name>` in a `## Tokens used` row that `tokens.json` does not define; rewrite that row with a defined token, and the CLI reruns.

### 15. Remedy — model, subagent `ui-spec-writer`

Read `references/spec.md` `## Remedy` for the per-id rewrite rule.

Take the `UI-nnn` of `$1`, the ids after `--remedy`, the current file from `devforgeai doc load ui-spec <UI-nnn>`, and the `findings[]` of `.devforgeai/reports/SPRINT-nnn-plan.yaml` or `.devforgeai/reports/STORY-nnn-build.yaml`. Each cited id names its own rewrite:

| Cited id | What it names | Sections rewritten |
|---|---|---|
| `AC-nnn` | a criterion the spec does not support | `## States`, `## Interaction`, `## Accessibility` |
| `TOKEN-<name>` | a token the spec references and `tokens.json` does not define | `## Anatomy`, `## Tokens used` |
| `UI-nnn` | a spec a story references and `.devforgeai/ui-specs/` does not hold | the whole file, written from step 13 |

Sections the cited ids do not name keep every byte. The file goes to `status: draft`, then to `status: approved` after step 14. A cited `AC-nnn` absent from the story adds one `open_questions` line naming the id, and the ids that did resolve are rewritten.

`devforgeai doc load ui-spec <UI-nnn>` exiting 1 on `DFA-E200` means the spec is absent, and step 15 writes it from step 13.

### 16. Close the run — CLI

Run `devforgeai phase set design --id <the run's subject id>` — the `UI-nnn` in spec mode, the `EPIC-nnn` in brand mode, the `IDEA-nnn` in sketch mode — which records `[last_cross]` in `state.toml` with `phase`, `id`, and the turn. That is the whole of this step: this skill runs no `devforgeai handoff` of its own. The Stop hook reads `[last_cross]`, renders Design's block and the block for the phase `[current].phase` still names — Design left it where it found it — joins the two, and clears `[last_cross]`. The Stop hook renders both blocks; this skill writes no part of either.

## Subagents

Contracts, tools, and models are in `agents.md`.

| Subagent | Invoked at | What to pass | What comes back |
|---|---|---|---|
| `mockup-designer` | steps 3 and 4, sketch mode, alone | `flows[]`, `out_dir`, `seed_data_path`, `constraints`, `brand`, the run's `IDEA-nnn` | the sketch return object: `idea_id`, `screens`, `brand_sketch`, `uncovered_flows`, `notes` |
| `brand-designer` | steps 7 and 8, brand mode, in that order, alone | the five answers, the step 5 candidates, the `personas[].goal` strings, the answer-to-leaf mapping table | `brand_name`, `tokens_written`, `logo_written`, `contrast`, `adjustments`, `reason` |
| `requirement-coverage-auditor` | step 11, spec mode, alone, before any `UI-nnn` is allocated | the screen list of step 10, `requirements[]` and `epics[]`, the `## Core flows` table when present, the run's `STORY-nnn` or `EPIC-nnn` | `devforgeai/verifier/1`: `passed`, `total`, `unit: screens`, `findings[]`, and `payload` holding `subject_id`, `screens`, `screens_without_req`, `flows_without_req`, `covered`; a registered verifier, ingested by SubagentStop |
| `ui-spec-writer` | step 13, one per screen in parallel; step 15, one for the cited `UI-nnn` | one screen with its `UI-nnn`, its `REQ-nnn` records, the `PERSONA-nnn` goal, the flattened token names, mockup paths, `figma_node_url` when the story carries one, `figma_context` from the step 13 run of `figma:figma-design-to-code` or `""`, and on remedy the current file and the cited ids | `ui_id`, `path`, `sections`, `tokens_used`, `states`, `unresolved_ids` |

## Documents

| Document | Path | Written by | Template |
|---|---|---|---|
| Brand tokens | `.devforgeai/brand/tokens.json` | brand, step 7, `meta.status` to `approved` at step 9 | `templates/tokens.json` |
| Logo | `.devforgeai/brand/logo.svg` | brand, step 8 | `templates/logo.svg` |
| Brand kit | `.devforgeai/brand/brand-kit.md` | brand, step 9 | `templates/brand-kit.md` |
| UI spec | `.devforgeai/ui-specs/UI-nnn.md`, one per screen | spec, step 13, and step 15 on remedy | `templates/ui-spec.md` |
| Wireframe screen | `.devforgeai/explore/mockups/<FLOW-nnn-nn>.html` | sketch, step 3 | `templates/sketch-screen.html` |
| Brand sketch | `.devforgeai/explore/mockups/brand-sketch.json` | sketch, step 4 | none; `name`, `palette`, `type_pair` |

Frontmatter, one rule for every document under `.devforgeai/`: the seven keys `schema`, `id`, `phase`, `status`, `produced_by`, `consumes`, `open_questions`, in that order, with no other top-level key. `phase` is `design` and `produced_by` is `designing-interfaces` in all three. `tokens.json` carries those seven keys under its `meta` object rather than as frontmatter, `brand-kit.md` and `UI-nnn.md` carry them as YAML frontmatter, and the mockup HTML files carry none — they live outside `.devforgeai/` document validation and under the `design lint` exemption.

Status values:

- `tokens.json` `meta.status`: `draft` at step 7, `approved` at step 9.
- `brand-kit.md`: `approved`, the only value; the file is rewritten on the next brand run.
- `UI-nnn.md`: `draft` at step 13, `approved` at step 14. A remedy run moves the cited file back to `draft` and forward to `approved` again.

Token shape, which `design lint` resolves against: top-level keys `meta`, `color`, `type`, `spacing`, `radius`, `elevation`, `motion`, in that order. A leaf of `color` is an object with exactly the keys `light` and `dark`; a leaf of every other group is a string. Leaf names match `^[a-z][a-z0-9-]*$`. The flattened token name is `TOKEN-<group>-<leaf>` and the CSS custom property is `--<group>-<leaf>`. The leaf counts are 12 colour, 12 type, 6 spacing, 4 radius, 3 elevation, 4 motion, and brand mode writes no others.

Every colour and every type value in a `UI-nnn.md` is a `TOKEN-<name>` reference. A hex triple, an `rgb(`, an `hsl(`, a CSS named colour, a `px` or `rem` font size, and a font family name do not appear in the body. The four `## Breakpoints` widths — `sm` 320px, `md` 768px, `lg` 1024px, `xl` 1440px — are fixed by the template and are not tokens.

## Send-back

Design sends back to Discover only, from spec mode only, at step 11 only, before any `UI-nnn.md` is written.

| Condition | Detected by | IDs cited |
|---|---|---|
| A screen the story asks for realizes no requirement | `requirement-coverage-auditor`, `payload.screens_without_req` non-empty | the `UI-nnn` allocated for that screen at step 12 |
| A flow in `explore/brief.md` `## Core flows` equals no `requirements[].source` | `requirement-coverage-auditor`, `payload.flows_without_req` non-empty | the `FLOW-nnn` |

On either condition, write no `UI-nnn.md` for the cited screens and write the covered screens as step 13 defines. The SubagentStop hook writes `.devforgeai/reports/UI-nnn-design.yaml` at `status: send_back` from the audit output. `state.toml` `[current]` is unchanged, because Design advances no phase.

The Stop hook renders the closing block from that report; this skill writes no part of it. Its `Next` line reads:

```
Next      /discover <IDEA-nnn> --remedy UI-nnn,FLOW-nnn
```

The `<IDEA-nnn>` is the top-level `id` key of `.devforgeai/requirements.yaml`, which step 10 already loaded through `devforgeai doc load requirements -`. It is the document's own id, not the `EPIC-nnn` the story belongs to: an epic is a grouping inside that document, and Discover's remedy form re-opens the document by its own id. The send-back grammar is `/<upstream> <ID> --remedy <ID>,<ID>` and carries no `--from` flag; the `UI-` and `FLOW-` prefixes identify Design as the source. The `Then` line reads `/design <STORY-nnn> --resume`.

Sketch mode emits no send-back: it runs inside Explore's Phase 0, where `requirements.yaml` does not exist, and an uncovered flow travels back as `uncovered_flows`. Brand mode emits no send-back: it reads `personas[]` and writes tokens, and a thin persona is a Discover-side edit that no token depends on.

Design receives a send-back from Plan and from Build as `/design UI-nnn --remedy <ids>`, handled at step 15. The returning command on the `Then` line is `/plan <SPRINT-nnn> --resume` or `/build <STORY-nnn> --resume`.

## Remedy and resume

**`--remedy`** reaches spec mode alone, as `/design UI-nnn --remedy <ids>` from Plan or from Build. It runs step 15 and then step 14, and skips steps 10 to 13 except where a cited bare `UI-nnn` calls for a file step 13 writes. Sections the cited ids do not name keep every byte.

**`--resume`** applies to brand mode, to a partial spec run, and to the return from a send-back. Resume state lives in the documents and the report, not in `state.toml`, which Design leaves untouched:

- `--brand --resume` continues at the first step whose output is absent or whose `meta.status` is still `draft`.
- `--spec --resume` writes the screens that have no `UI-nnn.md` on disk, and reads the most recently modified `.devforgeai/reports/UI-*-design.yaml` at `status: send_back` to find the ids the send-back cited. Step 11 re-runs the coverage audit for those screens alone, against the `REQ-nnn` Discover allocated, and step 13 writes their files.
- Every `UI-nnn.md` already at `status: approved` keeps every byte in both cases.

## References

- `references/sketch.md` — read before step 2: the sketch request and return schemas, the `FLOW-nnn-nn` screen naming, the wireframe HTML shape, and why `out_dir` is exempt from `design lint`.
- `references/brand.md` — read before step 5: the five questions, the answer-to-leaf mapping, the contrast floors, the logo constraints, the nine `brand-kit.md` sections, and the Figma mirror and its fallback.
- `references/spec.md` — read before step 10: the nine `UI-nnn.md` sections with their column sets and row rules, the coverage rule the audit applies, the token-only rule, and the per-id remedy rewrite.
