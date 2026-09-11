---
schema: devforgeai-spec/1
doc: design
status: draft
produced_by: design-spec-author
consumes: [00-conventions]
open_questions: []
---

# Cross-cutting · Design · `designing-interfaces`

## Scope

`designing-interfaces` is the framework's own design capability. It runs in three modes on one slash command, `/design`. `--sketch` draws wireframe screens for a set of `FLOW-nnn` flows under the contract `specs/02-explore.md` `## Outputs` defines, and returns JSON; it is the mode Explore step 6 calls. `--brand` turns the Explore brand sketch, the `PERSONA-nnn` records of `requirements.yaml`, and the answers to five fixed questions into `.devforgeai/brand/tokens.json`, `.devforgeai/brand/logo.svg`, and `.devforgeai/brand/brand-kit.md`. `--spec` turns one `STORY-nnn` or `EPIC-nnn` and the `REQ-nnn` records it cites into one `.devforgeai/ui-specs/UI-nnn.md` per screen or component the story needs, each carrying states, breakpoints, accessibility rows, and every colour and type value written as a `TOKEN-<name>` reference. It sends back to Discover, and it receives a send-back from Plan and from Build as `/design UI-nnn --remedy <ids>`.

`designing-interfaces` is not a phase. It has no number in §5's phase sequence, no `[[gate]]` in `gates.toml`, no key in `state.toml` `[active]`, and it advances no phase: a run leaves `[current].phase` and `[current].id` as it found them. It writes no requirement, no context file, no ADR, no story, and no production code. It does not implement the interfaces it specifies; `implementing-stories` does that through `frontend-developer`, reading the documents this skill wrote. It does not decide whether a colour literal in a source file is acceptable; `devforgeai design lint` decides that, on PreToolUse, against `tokens.json`. It does not edit `requirements.yaml` when a screen has no requirement behind it; it emits a SEND BACK to Discover citing `UI-nnn` and `FLOW-nnn` ids.

## Inputs

| Input | Path | IDs | Read by | Mode |
|---|---|---|---|---|
| Sketch request | `.devforgeai/explore/sketch-request.json` | `IDEA-nnn`, `FLOW-nnn` | step 2 | `--sketch` |
| Explore brief | `.devforgeai/explore/brief.md` | `IDEA-nnn`, `FLOW-nnn` | steps 2, 5, 11 | all three |
| Explore seed data | `.devforgeai/explore/seed-data.json` | `IDEA-nnn` | steps 3, 13 | `--sketch`, `--spec` |
| Explore brand sketch | `.devforgeai/explore/mockups/brand-sketch.json` | `IDEA-nnn` | step 5 | `--brand` |
| Requirements | `.devforgeai/requirements.yaml`, printed by `devforgeai doc load requirements -` | `REQ-nnn`, `EPIC-nnn`, `PERSONA-nnn` | steps 5, 10, 11 | `--brand`, `--spec` |
| Story | `.devforgeai/stories/STORY-nnn.md`, printed by `devforgeai doc load story <STORY-nnn>` | `STORY-nnn`, `AC-nnn`, `REQ-nnn`, `UI-nnn` | step 10 | `--spec` |
| Frontend globs and token path | `.devforgeai/config.toml`, keys `[frontend].globs`, `[frontend].exclude`, `[frontend].tokens_path` | none | step 14 | `--brand`, `--spec` |
| Brand tokens | `.devforgeai/brand/tokens.json` | `TOKEN-<name>` | steps 13, 15 | `--spec` |
| Prior UI spec | `.devforgeai/ui-specs/UI-nnn.md`, printed by `devforgeai doc load ui-spec <UI-nnn>` | `UI-nnn` | step 15 | `--spec` remedy |
| Send-back report from Plan | `.devforgeai/reports/SPRINT-nnn-plan.yaml`, `findings[]` | `UI-nnn`, `AC-nnn` | step 15 | `--spec` remedy |
| Send-back report from Build | `.devforgeai/reports/STORY-nnn-build.yaml`, `findings[]` | `UI-nnn`, `AC-nnn`, `TOKEN-<name>` | step 15 | `--spec` remedy |
| Prior design report | `.devforgeai/reports/UI-nnn-design.yaml`, the most recently modified one at `status: send_back`, `findings[]` | `UI-nnn`, `FLOW-nnn` | step 11 | `--spec --resume` |
| Next free UI id | stdout of `devforgeai doc validate --allocate UI` | `UI-nnn` | step 12 | `--spec` |
| Phase state | `.devforgeai/state.toml`, keys `[current].phase` and `[current].id` | none | step 16 | all three |

`--sketch` reads no document produced by Discover, Constitute, or Plan: those phases have not run when Explore step 6 calls it. `--spec` reads `explore/brief.md` for the `## Core flows` table alone, and treats the file as absent when Discover ran from entry point B, where `requirements.yaml` carries `consumes: []` and no brief exists.

## Outputs

### 1. `.devforgeai/brand/tokens.json`

Written by `--brand`. Top-level keys, in this order, with no other top-level key: `meta`, `color`, `type`, `spacing`, `radius`, `elevation`, `motion`. `meta` carries the seven §5 keys, in §5 order. The other six are token groups. A group's keys are leaf names matching `^[a-z][a-z0-9-]*$`. A leaf of `color` is an object with exactly the keys `light` and `dark`. A leaf of every other group is a string. The flattened token name is `TOKEN-<group>-<leaf>`; the CSS custom property a frontend file references is `--<group>-<leaf>`.

```json
{
  "meta": {
    "schema": "devforgeai/tokens/1",
    "id": "TOKEN-001",
    "phase": "design",
    "status": "approved",
    "produced_by": "designing-interfaces",
    "consumes": ["PERSONA-001", "EPIC-001"],
    "open_questions": []
  },
  "color": {
    "bg":             { "light": "#fbfaf8", "dark": "#141413" },
    "surface":        { "light": "#ffffff", "dark": "#1d1d1b" },
    "surface-raised": { "light": "#f3f1ed", "dark": "#262623" },
    "border":         { "light": "#ddd9d2", "dark": "#3a3a36" },
    "text":           { "light": "#141413", "dark": "#f5f4f1" },
    "text-muted":     { "light": "#5f5c55", "dark": "#a8a49c" },
    "primary":        { "light": "#1a5e4a", "dark": "#4bbf97" },
    "on-primary":     { "light": "#ffffff", "dark": "#0d2a21" },
    "accent":         { "light": "#b4530a", "dark": "#e8933f" },
    "success":        { "light": "#1c6b3c", "dark": "#54c07d" },
    "warning":        { "light": "#8a5a00", "dark": "#d9a021" },
    "danger":         { "light": "#a32020", "dark": "#e86464" }
  },
  "type": {
    "family-sans":    "Inter, system-ui, sans-serif",
    "family-mono":    "JetBrains Mono, ui-monospace, monospace",
    "size-xs":        "0.75rem",
    "size-sm":        "0.875rem",
    "size-base":      "1rem",
    "size-lg":        "1.25rem",
    "size-xl":        "1.75rem",
    "size-xxl":       "2.5rem",
    "line-tight":     "1.2",
    "line-base":      "1.55",
    "weight-regular": "400",
    "weight-bold":    "650"
  },
  "spacing": {
    "xs": "0.25rem", "sm": "0.5rem", "md": "1rem",
    "lg": "1.5rem",  "xl": "2.5rem", "xxl": "4rem"
  },
  "radius": {
    "none": "0", "sm": "0.25rem", "md": "0.5rem", "full": "9999px"
  },
  "elevation": {
    "flat":    "none",
    "raised":  "0 1px 2px rgba(20, 20, 19, 0.10)",
    "overlay": "0 8px 24px rgba(20, 20, 19, 0.18)"
  },
  "motion": {
    "duration-fast": "120ms",
    "duration-base": "200ms",
    "duration-slow": "360ms",
    "ease-standard": "cubic-bezier(0.2, 0, 0, 1)"
  }
}
```

`meta.status` enum: `draft` (step 7 wrote the file), `approved` (step 9 wrote `brand-kit.md` and the file passed `devforgeai design lint --tokens`). The leaf sets above are the complete leaf sets: `--brand` writes these 12 colour leaves, 12 type leaves, 6 spacing leaves, 4 radius leaves, 3 elevation leaves, and 4 motion leaves, and writes no others. The values above are the template defaults; step 7 replaces every value.

### 2. `.devforgeai/brand/logo.svg`

Written by `--brand`, step 8. One `<svg>` element with a `viewBox` of `0 0 256 64`, `role="img"`, and one `<title>` holding the brand name. Every `fill` and `stroke` attribute is one of `currentColor`, `none`, or a hex value equal to a `color` leaf's `light` value. No `<image>`, no `<script>`, no external reference, no embedded font: text in the mark is drawn as `<path>` data. Byte size at most 32768.

### 3. `.devforgeai/brand/brand-kit.md`

Written by `--brand`, step 9. Frontmatter, in this order, no other top-level keys:

```yaml
---
schema: devforgeai/brand-kit/1
id: TOKEN-001
phase: design
status: approved
produced_by: designing-interfaces
consumes: [PERSONA-001, EPIC-001]
open_questions: []
---
```

`status` enum: `approved` is the only value; the file is written once per `--brand` run and overwritten on the next one. Sections, in this order, with these headings verbatim:

| # | Heading | Content |
|---|---|---|
| 1 | `## Brand name` | One line, the name. Then one line, the one-sentence descriptor used under the logo. |
| 2 | `## Voice` | Table, columns `Trait \| Sounds like \| Does not sound like`. Exactly 3 rows, one per trait chosen at step 6. |
| 3 | `## Logo` | Table, columns `Field \| Value`, with exactly these four rows: `Form` (the step 6 answer), `Path` (`.devforgeai/brand/logo.svg`), `Clear space` (a `TOKEN-spacing-<leaf>` name), `Smallest size` (a px value). |
| 4 | `## Color` | Table, columns `Token \| Light \| Dark \| Used for \| Contrast on bg`. One row per `color` leaf, 12 rows, in the `tokens.json` key order. `Contrast on bg` is the WCAG contrast ratio against `TOKEN-color-bg` in the same theme, to one decimal, lower of the two themes. |
| 5 | `## Type` | Table, columns `Token \| Value \| Used for`. One row per `type` leaf, 12 rows, in `tokens.json` key order. |
| 6 | `## Spacing and shape` | Table, columns `Token \| Value \| Used for`. One row per `spacing` and `radius` leaf, 10 rows. |
| 7 | `## Motion` | Table, columns `Token \| Value \| Used for`. One row per `motion` leaf, 4 rows. Then one line stating what `prefers-reduced-motion: reduce` changes. |
| 8 | `## Figma` | Table, columns `Field \| Value`, with exactly these three rows: `Status` (`mirrored` or `not mirrored`), `Collection` (the variable collection name, or `-`), `Reason` (`-`, or `figma plugin not authenticated`, or `figma plugin not present`). |
| 9 | `## Do not` | Unnumbered lines, 3 to 8, each one sentence in the present tense naming one thing the brand excludes. |

The `Smallest size` value in `## Logo` is a px measurement of the printed mark, and it is the one numeric literal in this file. `devforgeai design lint` reads `.devforgeai/brand/tokens.json` and files matching `[frontend].globs`, and reads neither this file nor a `UI-nnn.md` except under `--tokens`, so the value resolves to no token and needs none.

### 4. `.devforgeai/ui-specs/UI-nnn.md`

Written by `--spec`, one file per screen or component the story needs. Frontmatter, in this order, no other top-level keys:

```yaml
---
schema: devforgeai/ui-spec/1
id: UI-003
phase: design
status: draft
produced_by: designing-interfaces
consumes: [STORY-014, REQ-007, REQ-011, PERSONA-001]
open_questions: []
---
```

`status` enum, in progression order: `draft` (step 13 wrote the file), `approved` (step 14 ran `devforgeai design lint --tokens` with exit 0 and `devforgeai doc validate` with exit 0). A remedy run moves the cited file back to `draft` and forward to `approved` again; there is no third value.

Sections, in this order, with these headings verbatim:

| # | Heading | Content |
|---|---|---|
| 1 | `## Purpose` | One sentence. What the person on this screen is doing and what leaves it done. |
| 2 | `## Requirements` | Table, columns `REQ \| Statement \| Satisfied by`. One row per `REQ-nnn` in `consumes`, 1 to 8 rows. `Satisfied by` names a `Region` value from `## Anatomy`. |
| 3 | `## Anatomy` | Table, columns `Region \| Element \| Content source \| Tokens`. 3 to 20 rows. `Content source` is a `seed-data.json` entity name, a `REQ-nnn`, or `static`. `Tokens` is a space-separated list of `TOKEN-<name>` values. |
| 4 | `## States` | Table, columns `State \| Trigger \| Visible change \| Tokens`. One row per state the screen has, drawn from the closed set `default`, `loading`, `empty`, `partial`, `error`, `success`, `disabled`, `read-only`. `default` is present in every file. |
| 5 | `## Breakpoints` | Table, columns `Name \| Min width \| Layout \| Changes from previous`. Exactly four rows: `sm` `320px`, `md` `768px`, `lg` `1024px`, `xl` `1440px`. |
| 6 | `## Interaction` | Table, columns `Trigger \| Response \| Focus after`. One row per pointer, key, or form event the screen answers, 1 to 20 rows. `Focus after` names a `Region` value or `unchanged`. |
| 7 | `## Accessibility` | Table, columns `Check \| Requirement \| Evidence`. Exactly eight rows, in this order: `Landmark`, `Heading order`, `Name`, `Role`, `Keyboard path`, `Focus visible`, `Contrast`, `Motion`. `Evidence` names a `Region`, a `TOKEN-<name>`, or a ratio. |
| 8 | `## Tokens used` | Table, columns `Token \| Group \| Where`. One row per distinct `TOKEN-<name>` appearing in sections 3, 4, and 7. `Where` names one `Region` or `State`. |
| 9 | `## Out of scope` | Unnumbered lines, 0 to 8, each one sentence naming a behaviour this screen does not carry and, when one exists, the `UI-nnn` that does. |

Every colour and every type value in this document is a `TOKEN-<name>` reference. A hex triple, an `rgb(`, an `hsl(`, a CSS named colour, a `px` or `rem` font size, and a font family name do not appear in the body.

### 5. `.devforgeai/explore/mockups/`

Written by `--sketch`. One file per screen, named `<screen>.html`, where `<screen>` is `FLOW-nnn-nn` as `specs/02-explore.md` `## Outputs` fixes it, plus `brand-sketch.json` when the sketch request carried a `brand` object. Static HTML with a `<style>` element and no script, no build step, and no dependency manifest. The directory is Explore's `out_dir` and is deleted by `devforgeai explore prune` on `kill` and on `promote`. `.devforgeai/explore/mockups/**` is one of the two sketch entries in the default `config.toml` `[frontend].exclude` list, alongside `.explore-prototype/**`, so `design lint` skips both: a wireframe drawn before a brand kit exists has no token to resolve against.

### 6. The sketch-mode return value

`--sketch` returns the JSON object `specs/02-explore.md` `## Outputs` defines, with the fields `mode`, `idea_id`, `screens`, `brand_sketch`, `uncovered_flows`, and `notes`, and writes no `.devforgeai/brand/tokens.json` and no `.devforgeai/ui-specs/UI-nnn.md`.

## Workflow

**1. Establish the run — model, CLI.** Input: `$ARGUMENTS`, the allocated id on the preamble's stdout, `.devforgeai/state.toml`. The mode is `--sketch` when `$ARGUMENTS` contains `--sketch` or the skill was invoked with a `mode` field of `"sketch"`; `--brand` when `$ARGUMENTS` contains `--brand`; and `--spec` in every other case, which covers `--spec`, a bare `STORY-nnn` or `EPIC-nnn`, and the remedy form `UI-nnn --remedy <ids>`. The run is a remedy run when `$ARGUMENTS` contains `--remedy`, and a resume when it contains `--resume`. Output: a mode, a run kind, and the subject id. Failure path: `$ARGUMENTS` holds an id whose prefix is none of `IDEA`, `EPIC`, `STORY`, `UI`; the run stops with one `Blocked` line naming the three accepted prefixes.

**2. Sketch — read the request — model.** Input: `.devforgeai/explore/sketch-request.json`, and `brief_path` from it. Output: the `flows[]` array, `out_dir`, `seed_data_path`, `brand`, and `constraints` in hand. Failure path: the file is absent; the mode returns `screens: []`, `uncovered_flows` holding every `flow_id` the invocation passed, and one `notes` line naming the missing path.

**3. Sketch — draw the screens — subagent `mockup-designer`.** Input: `flows[]`, `out_dir`, the rows of `.devforgeai/explore/seed-data.json`, `constraints.screens_per_flow_max`, and the `brand` object. The agent invokes the built-in `design` skill for layout and visual judgment and writes one HTML file per screen into `out_dir`. Output: JSON with `screens[]`, `brand_sketch`, `uncovered_flows`, `notes`. Failure path: a flow whose steps produce no screen comes back in `uncovered_flows`, and the remaining flows keep their screens.

**4. Sketch — write the brand sketch — subagent `mockup-designer`.** Input: the `brand` object of the request. Output: `.devforgeai/explore/mockups/brand-sketch.json` holding `name`, `palette` (3 to 6 hex values), and `type_pair` (two family strings), and the `brand_sketch` field of the return value pointing at it. Failure path: the request carried `brand: null`; the file is not written and `brand_sketch` is `null`. The file holds exactly those three keys at the top level of one flat object. It has no template and carries no envelope: everything under `.devforgeai/explore/mockups/` sits outside `.devforgeai/` document validation, which `## Documents` records.

**5. Brand — gather the input — model.** Input: `.devforgeai/explore/mockups/brand-sketch.json` when it exists, the `personas[]` array of `requirements.yaml`, and the `epics[]` entry named by `$1`. Output: a candidate name, a candidate palette, a candidate type pair, and the persona goals the brand answers to. Failure path: `brand-sketch.json` is absent; the candidate name comes from the `epics[].title` of `$1` and the candidate palette and type pair are the template defaults of `## Outputs`.

**6. Brand — ask the five questions — user, AskUserQuestion.** Input: the step 5 candidates, shown as one line of prose above the first call. Two calls, with the questions, headers, and option labels fixed in `templates/brand-questions.md`. Output: five answers, one per header: `Tone`, `Color`, `Type`, `Density`, `Logo`. Failure path: a free-text answer matching no label re-asks that one question once, then the run takes the first option of that question and adds one `open_questions` line to `brand-kit.md` naming the header.

**7. Brand — write the tokens — subagent `brand-designer`.** Input: the five answers, the step 5 candidates, and the answer-to-leaf mapping table in `templates/brand-questions.md`. The agent invokes the built-in `frontend-design:frontend-design` skill for palette and type judgment. Output: `.devforgeai/brand/tokens.json` with `meta.status: draft` and every leaf of the six groups filled, each `color` leaf carrying `light` and `dark`, and every `Contrast on bg` ratio at 4.5 or above for the text leaves and 3.0 or above for the border leaf. Failure path: a returned palette whose `text` on `bg` ratio falls below 4.5 in either theme; the agent darkens or lightens the `text` value until it clears and records the adjustment in a `notes` field the model writes into `brand-kit.md` `## Do not`.

**8. Brand — draw the logo — subagent `brand-designer`.** Input: the `Logo` answer, the brand name, and the `color` group. Output: `.devforgeai/brand/logo.svg` as `## Outputs` fixes it. Failure path: the agent returns `logo_written: false` with a `reason` string; the model writes a one-line wordmark SVG from `templates/logo.svg` with the brand name and the `TOKEN-color-text` light value, and continues.

**9. Brand — write the brand kit and mirror it — model, `figma:figma-use` and `figma:figma-generate-library` skills.** Input: `tokens.json`, `logo.svg`, and the five answers. Output: `.devforgeai/brand/brand-kit.md` with all nine sections, and `tokens.json` `meta.status` moved to `approved`. When the Figma plugin's tools are present in the session and authenticated, the two Figma skills write the six groups as a Figma variable collection named `<brand name> tokens` with a light and a dark mode, and `## Figma` carries `Status: mirrored` with the collection name. When the plugin's tools are absent, or a call returns an authentication error, `## Figma` carries `Status: not mirrored` with `Reason` set to `figma plugin not present` or `figma plugin not authenticated`, every other output of this step is unchanged, and the run continues. Failure path: `brand-kit.md` fails the `PostToolUse` `doc validate`, whose diagnostic comes back in `hookSpecificOutput.additionalContext`; the model rewrites the section it names.

**10. Spec — load the subject — model, CLI.** Input: `devforgeai doc load story <STORY-nnn>` when `$1` is a story, `devforgeai doc load requirements -` in both cases, and the `epics[].requirements` list when `$1` is an epic. Output: the `AC-nnn` rows, the `REQ-nnn` records they cite, the `PERSONA-nnn` records those requirements name, and the `UI-nnn` ids the story already references. Failure path: `.devforgeai/brand/tokens.json` is absent; the mode writes no `UI-nnn.md`, and the handoff carries `Blocked   you: run /design --brand <EPIC-nnn> first`.

**11. Spec — audit coverage — subagent `requirement-coverage-auditor`.** On a `--resume` run the model first reads the most recently modified `.devforgeai/reports/UI-*-design.yaml` whose `status` is `send_back`, takes the `UI-nnn` and `FLOW-nnn` ids from its `findings[]`, and narrows this step and step 13 to the screen behind those ids; every `UI-nnn.md` at `status: approved` is left as it is. Input: the screens the story names, the `REQ-nnn` records of step 10, the `## Core flows` table of `.devforgeai/explore/brief.md` when that file exists, and the `requirements[].source` values of `requirements.yaml`. Output: one `devforgeai/verifier/1` object with `total` and `passed` at the top and `screens[]`, `screens_without_req[]`, `flows_without_req[]`, `covered`, and `subject_id` under `payload`. The `SubagentStop` hook ingests it into `verifiers.requirement_coverage` of `.devforgeai/reports/UI-nnn-design.yaml`. Failure path: `brief.md` is absent, which is the shape Discover entry point B leaves; `payload.flows_without_req` is `[]` and the audit runs on screens alone.

**12. Spec — allocate the ids — model, CLI.** Input: `screens[]` from step 11 and `devforgeai doc validate --allocate UI`, run once per screen that carries no existing `UI-nnn`. Output: one `UI-nnn` per screen, in `screens[]` order. Failure path: `--allocate` exits 1 on `DFA-E215` with the prefix exhausted; the run stops and the stderr line reaches the model.

**13. Spec — write the UI specs — model, subagent `ui-spec-writer`.** Before the agent is invoked, and in the main conversation, the model runs `figma:figma-design-to-code` for the story's Figma node URL when the story carries one and the Figma plugin's tools are present and authenticated, and takes what it returns as `figma_context`; in every other case `figma_context` is `""`. Input to the agent: one screen from step 11 with its allocated `UI-nnn`, the `REQ-nnn` records it realizes, the `PERSONA-nnn` goal, the flattened token names of `.devforgeai/brand/tokens.json`, the matching mockup files under `.devforgeai/explore/mockups/` when they exist, and `figma_context`. The agent invokes the built-in `frontend-design:frontend-design` skill for component composition judgment and makes no Figma call: MCP tools do not reach a subagent, so that call is the skill's. Output: one `.devforgeai/ui-specs/UI-nnn.md` per screen, with `status: draft` and all nine sections. One invocation per screen, run in parallel across screens. Failure path: `figma_context` is `""`; `## Anatomy` `Content source` cites the mockup path or the `REQ-nnn` in place of the node, and the file is otherwise unchanged.

**14. Spec — resolve the tokens — CLI.** Input: `devforgeai design lint --tokens`. Output: exit 0, and the model moves every file written at step 13 to `status: approved`. Failure path: exit 1 with `DFA-E244` naming a `TOKEN-<name>` in a `## Tokens used` row that `tokens.json` does not define; the model rewrites that row with a defined token and reruns.

**15. Remedy — rewrite the cited ids — model, subagent `ui-spec-writer`.** Input: the `UI-nnn` of `$1`, the ids after `--remedy`, `devforgeai doc load ui-spec <UI-nnn>`, and the `findings[]` of `.devforgeai/reports/SPRINT-nnn-plan.yaml` or `.devforgeai/reports/STORY-nnn-build.yaml`. An `AC-nnn` in `--remedy` names a criterion the spec does not support and rewrites `## States`, `## Interaction`, and `## Accessibility`. A `TOKEN-<name>` in `--remedy` names a token the spec references and `tokens.json` does not define, and rewrites `## Anatomy` and `## Tokens used`. A bare `UI-nnn` in `--remedy` names a spec a story references and this directory does not hold, and writes it from step 13. Output: the cited file at `status: draft`, then `status: approved` after step 14. Sections the cited ids do not name keep every byte. Failure path: a cited `AC-nnn` absent from the story; the model adds one `open_questions` line naming the id and rewrites the ids that did resolve.

**16. Record the cross-cutting run — CLI.** `devforgeai handoff --phase design --id UI-nnn`, run as the last step, which writes `state.toml` `[last_cross]` with `phase: design`, the `UI-nnn`, and the turn's timestamp. The Stop hook then renders that block and the block for `[current].phase`, which Design left as it found it, joins the two with a blank line into one `systemMessage`, and clears `[last_cross]`. The Stop hook renders the closing block; this skill writes no part of it.

## Subagents

### mockup-designer

- **name**: `mockup-designer`
- **derives_from**: `C:\Users\bryan\.claude\agents\frontend-developer.md`
- **purpose**: Draw one wireframe screen per step of a flow, as a static HTML file, carrying the seed rows the flow moves.
- **tools**: `Read`, `Write`, `Glob`, `Skill`
- **model**: `sonnet` — layout from a named flow and a fixed screen budget, with the built-in `design` skill supplying the visual judgment.
- **input**: `flows[]`, `out_dir`, `seed_data_path`, `constraints`, and `brand` from `.devforgeai/explore/sketch-request.json`; the run's `IDEA-nnn`.
- **output**:

```json
{ "type": "object", "required": ["idea_id","screens","brand_sketch","uncovered_flows","notes"],
  "properties": {
    "idea_id": { "type": "string", "pattern": "^IDEA-[0-9]{3}$" },
    "screens": { "type": "array", "items": { "type": "object",
      "required": ["flow_id","screen","path","title","state"], "properties": {
        "flow_id": { "type": "string", "pattern": "^FLOW-[0-9]{3}$" },
        "screen": { "type": "string", "pattern": "^FLOW-[0-9]{3}-[0-9]{2}$" },
        "path": { "type": "string" },
        "title": { "type": "string", "maxLength": 60 },
        "state": { "type": "string", "enum": ["default","empty","error"] } } } },
    "brand_sketch": { "type": ["object","null"], "required": ["path","name","palette","type_pair"],
      "properties": {
        "path": { "type": "string" },
        "name": { "type": "string", "maxLength": 40 },
        "palette": { "type": "array", "minItems": 3, "maxItems": 6,
          "items": { "type": "string", "pattern": "^#[0-9a-f]{6}$" } },
        "type_pair": { "type": "array", "minItems": 2, "maxItems": 2, "items": { "type": "string" } } } },
    "uncovered_flows": { "type": "array", "items": { "type": "string", "pattern": "^FLOW-[0-9]{3}$" } },
    "notes": { "type": "array", "maxItems": 5, "items": { "type": "string", "maxLength": 160 } } } }
```

- **invoked_at**: workflow steps 3 and 4, alone.
- **registered_verifier**: `no`

### brand-designer

- **name**: `brand-designer`
- **derives_from**: `new`
- **purpose**: Turn five answers and a candidate palette into a complete token set whose text and border contrast ratios clear WCAG AA in both themes, and one SVG mark.
- **tools**: `Read`, `Write`, `Skill`
- **model**: `opus` — a palette that reads as one brand in two themes and still clears contrast in both is the judgment this mode rests on.
- **input**: the five answers of step 6; the candidate name, palette, and type pair of step 5; the `personas[].goal` strings of `requirements.yaml`; the answer-to-leaf mapping table.
- **output**:

```json
{ "type": "object", "required": ["brand_name","tokens_written","logo_written","contrast","adjustments","reason"],
  "properties": {
    "brand_name": { "type": "string", "maxLength": 40 },
    "tokens_written": { "type": "boolean" },
    "logo_written": { "type": "boolean" },
    "contrast": { "type": "array", "items": { "type": "object",
      "required": ["token","theme","ratio"], "properties": {
        "token": { "type": "string", "pattern": "^TOKEN-color-[a-z][a-z0-9-]*$" },
        "theme": { "type": "string", "enum": ["light","dark"] },
        "ratio": { "type": "number", "minimum": 1.0, "maximum": 21.0 } } } },
    "adjustments": { "type": "array", "maxItems": 6, "items": { "type": "object",
      "required": ["token","from","to","reason"], "properties": {
        "token": { "type": "string", "pattern": "^TOKEN-color-[a-z][a-z0-9-]*$" },
        "from": { "type": "string" }, "to": { "type": "string" },
        "reason": { "type": "string", "maxLength": 160 } } } },
    "reason": { "type": ["string","null"], "maxLength": 200 } } }
```

- **invoked_at**: workflow steps 7 and 8, in that order, alone.
- **registered_verifier**: `no`

### requirement-coverage-auditor

- **name**: `requirement-coverage-auditor`
- **derives_from**: `new`
- **purpose**: Name every screen the story asks for that realizes no requirement, and every flow in the brief that no requirement sources, so the send-back cites ids rather than impressions.
- **tools**: `Read`, `Grep`, `Glob`
- **model**: `sonnet` — reference resolution across two documents against a stated rule, with no interview and no writes.
- **input**: the screen list of step 10, the `requirements[]` and `epics[]` arrays of `requirements.yaml`, the `## Core flows` table of `.devforgeai/explore/brief.md` when present, and the run's `STORY-nnn` or `EPIC-nnn`. A screen is covered when at least one `REQ-nnn` in the story's `consumes` names it in `acceptance_signal` or in an `AC-nnn` row. A flow is covered when its `FLOW-nnn` equals some `requirements[].source`.
- **output**:

One JSON object on stdout and nothing else: the `devforgeai/verifier/1` envelope, with the audit's own fields under `payload`.

```json
{ "type": "object",
  "required": ["schema","subagent","id","passed","total","unit","findings","payload"],
  "properties": {
    "schema":   { "const": "devforgeai/verifier/1" },
    "subagent": { "const": "requirement-coverage-auditor" },
    "id":       { "type": "string", "pattern": "^(STORY|EPIC)-[0-9]{3}$" },
    "passed":   { "type": "integer", "minimum": 0 },
    "total":    { "type": "integer", "minimum": 0 },
    "unit":     { "const": "screens" },
    "findings": { "type": "array", "items": { "type": "object",
      "required": ["id","severity","confidence","summary","evidence"], "properties": {
        "id": { "type": "string", "maxLength": 60 },
        "severity": { "enum": ["block","warn","info"] },
        "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
        "summary": { "type": "string", "maxLength": 120 },
        "evidence": { "type": "string", "maxLength": 200 } } } },
    "payload": { "type": "object",
      "required": ["subject_id","screens","screens_without_req","flows_without_req","covered"],
      "properties": {
        "subject_id": { "type": "string", "pattern": "^(STORY|EPIC)-[0-9]{3}$" },
        "screens": { "type": "array", "minItems": 0, "items": { "type": "object",
          "required": ["name","kind","requirements"], "properties": {
            "name": { "type": "string", "maxLength": 60 },
            "kind": { "type": "string", "enum": ["screen","component"] },
            "requirements": { "type": "array", "items": { "type": "string", "pattern": "^REQ-[0-9]{3}$" } } } } },
        "screens_without_req": { "type": "array", "items": { "type": "object",
          "required": ["name","evidence"], "properties": {
            "name": { "type": "string", "maxLength": 60 },
            "evidence": { "type": "string", "maxLength": 200 } } } },
        "flows_without_req": { "type": "array", "items": { "type": "object",
          "required": ["flow_id","evidence"], "properties": {
            "flow_id": { "type": "string", "pattern": "^FLOW-[0-9]{3}$" },
            "evidence": { "type": "string", "maxLength": 200 } } } },
        "covered": { "type": "integer", "minimum": 0 } } } } }
```

`id` is `payload.subject_id` and `total` is the number of screens in the input list. `passed` is `total` minus the count of screens carrying a `block` finding, which equals `payload.covered`. Each uncovered screen is one `findings[]` entry carrying the screen name as `id` at `severity: block`; each uncovered flow is one entry carrying the `FLOW-nnn` as `id` at `severity: warn`, which lands in the report and leaves `passed` where it stands, because the unit this ratio measures is screens. Every entry carries `confidence`, the rule that failed as `summary`, and the audit's evidence string as `evidence`.

- **invoked_at**: workflow step 11, alone, before any `UI-nnn` is allocated.
- **registered_verifier**: `yes` — `SubagentStop` ingests it from `last_assistant_message` into `verifiers.requirement_coverage` of `.devforgeai/reports/UI-nnn-design.yaml`, and `payload.covered`/`total` fills the `Verified` line of the handoff.

### ui-spec-writer

- **name**: `ui-spec-writer`
- **derives_from**: `C:\Users\bryan\.claude\agents\ui-spec-formatter.md`
- **purpose**: Write one `UI-nnn.md` for one screen, with the nine sections filled and every colour and type value written as a `TOKEN-<name>`.
- **tools**: `Read`, `Write`, `Grep`, `Glob`, `Skill`
- **model**: `opus` — the states, the keyboard path, and the eight accessibility rows are the content a story is built from, and a thin one costs a Verify finding.
- **input**: one screen object from step 11 with its allocated `UI-nnn`; the `REQ-nnn` records it realizes; the `PERSONA-nnn` goal; the flattened token names of `.devforgeai/brand/tokens.json`; the mockup file paths under `.devforgeai/explore/mockups/` matching the screen's flow; `figma_context`, a string holding the design context the skill obtained by running `figma:figma-design-to-code` itself in the main conversation, and `""` when the story carries no node URL, the Figma plugin is absent, or the call returned an authentication error; on a remedy run, the current file and the cited `AC-nnn`, `TOKEN-<name>`, and `UI-nnn` ids.

  This agent makes no Figma call of its own. The plugin's tools are MCP tools, and a subagent's tool set is its `tools` list plus its own `mcpServers` and nothing else, so a `Skill` grant would load the skill and still find no tools for it to drive. The call belongs to the skill in the main conversation and its result arrives here as a field.
- **output**:

```json
{ "type": "object", "required": ["ui_id","path","sections","tokens_used","states","unresolved_ids"],
  "properties": {
    "ui_id": { "type": "string", "pattern": "^UI-[0-9]{3}$" },
    "path": { "type": "string", "pattern": "^\\.devforgeai/ui-specs/UI-[0-9]{3}\\.md$" },
    "sections": { "type": "array", "minItems": 9, "maxItems": 9, "items": { "type": "string" } },
    "tokens_used": { "type": "array", "items": { "type": "string", "pattern": "^TOKEN-[a-z][a-z0-9-]*-[a-z][a-z0-9-]*$" } },
    "states": { "type": "array", "minItems": 1, "items": { "type": "string",
      "enum": ["default","loading","empty","partial","error","success","disabled","read-only"] } },
    "unresolved_ids": { "type": "array", "items": { "type": "string" } } } }
```

- **invoked_at**: workflow step 13, one invocation per screen, in parallel across screens; workflow step 15, one invocation for the cited `UI-nnn`.
- **registered_verifier**: `no`

## Command

The entry point is the skill itself: `skills/designing-interfaces/SKILL.md`, installed to `.claude/skills/design/`, whose frontmatter `name` is the slash command. There is no command file. This is the frontmatter and the preamble, verbatim, with no preamble line.

```markdown
---
name: design
description: Draws wireframe screens, builds the brand token set, and writes UI specifications. The DevForgeAI design capability, run by /design in three modes. --sketch draws wireframe HTML screens for FLOW-nnn flows and returns the sketch-mode JSON that Explore step 6 consumes. --brand turns a brand sketch, the PERSONA-nnn records, and five fixed questions into brand/tokens.json, brand/logo.svg, and brand/brand-kit.md. --spec turns a STORY-nnn or EPIC-nnn into one ui-specs/UI-nnn.md per screen, each carrying states, breakpoints, eight accessibility rows, and every colour and type value as a TOKEN-name reference. Reach for it whenever /design is typed, whenever a wireframe, mockup, brand kit, design token, colour palette, type scale, logo mark, UI specification, screen anatomy, breakpoint, or accessibility row is being produced for this framework, and whenever UI-nnn, TOKEN-name, brand/tokens.json, or ui-specs/ appears in a story, a report, or a send-back.
argument-hint: '--sketch IDEA-nnn | --brand EPIC-nnn | --spec STORY-nnn | UI-nnn --remedy AC-nnn,...'
allowed-tools: Bash(devforgeai:*), PowerShell(devforgeai:*), Read, Write, Edit, Glob, Grep, Agent, AskUserQuestion, Skill
---
```

This skill carries no preamble and no `disable-model-invocation` flag. It has no gate, so there is nothing for `gate require` to read; and spec mode allocates its `UI-nnn` ids at workflow step 12, one per screen that carries none, while sketch mode and brand mode allocate none — a preamble allocation would run before the mode is known and abort a `--sketch` or `--brand` run on an exhausted `UI` prefix it does not touch. The flag is omitted because `exploring-ideas` and `planning-work` reach sketch mode through the Skill tool, and a skill the model cannot invoke cannot be reached that way.

## CLI calls

| Subcommand with exact arguments | Called from | Exit handling |
|---|---|---|
| `devforgeai doc validate --allocate UI` | workflow step 12, once per uncovered screen; never in a preamble, because a `--sketch` or `--brand` run allocates none | stdout is the next free `UI-nnn`, reserved on disk under `.devforgeai/.allocated/` before it is printed; 1 on `DFA-E215` stops the run |
| `devforgeai doc load requirements -` | workflow steps 5 and 10 | 1 on `DFA-E200` with the path on stderr; brand mode falls back to the epic title, spec mode stops |
| `devforgeai doc load story <STORY-nnn>` | workflow step 10, when `$1` is a story | 1 on `DFA-E200` stops the run |
| `devforgeai doc load ui-spec <UI-nnn>` | workflow step 15 | 1 on `DFA-E200` means the spec is absent and step 15 writes it from step 13 |
| `devforgeai design lint --tokens` | workflow step 14 | 0 moves every written file to `approved`; 1 on `DFA-E244` names the row to rewrite |
| `devforgeai design lint <path>` | PreToolUse hook on Write and Edit of a file matching `[frontend].globs` | 1 maps to hook exit 2 and blocks the write; the stderr line names the path, the line, and the nearest token |
| `devforgeai doc validate <path>` | `PostToolUse` hook on `Write\|Edit\|NotebookEdit` under `.devforgeai/` | returns the diagnostic in `hookSpecificOutput.additionalContext`; steps 9, 13, and 15 rewrite the named key or section |
| `devforgeai handoff --phase design --id UI-nnn` | workflow step 16 | prints the §6 block from `.devforgeai/reports/UI-nnn-design.yaml` |
| `devforgeai report show UI-nnn design` | user, `improving-framework` | prints `.devforgeai/reports/UI-nnn-design.yaml` |

`design lint --tokens` and the `design` value of the `--phase` argument of `handoff`, `report show`, and `report ingest` are elaborations of §4 names that `specs/01-cli.md` carries, and are recorded as accepted in `## Decisions`. Every subcommand above is a §4 name.

## Gate

`.devforgeai/gates.toml` holds no entry for this skill. §5 gives one gate per phase, Design is not a phase, and `gate check --phase design` is undefined. Two enforcement points decide whether Design's outputs hold, and they differ in strength.

**PreToolUse, blocking.** §7 fixes the hook: a Write or Edit whose path matches `.devforgeai/config.toml` `[frontend].globs` minus `[frontend].exclude` runs `devforgeai design lint <path>`. `design lint` reads the token file from `.devforgeai/config.toml` `[frontend].tokens_path`, default `.devforgeai/brand/tokens.json`, and flattens it to `TOKEN-<group>-<leaf>`. `config.toml` is the one source of that path. `.devforgeai/context/coding-standards.md` `## Design tokens` carries the same path as prose for a person reading the standards, written by Constitute from the template in `specs/04-constitute.md`; no CLI command reads that section. The sketch and prototype exemptions are the two entries `.explore-prototype/**` and `.devforgeai/explore/mockups/**` in the default `[frontend].exclude` list, so a wireframe drawn before a brand kit exists resolves against nothing and is skipped. A literal colour (`#[0-9a-fA-F]{3,8}\b`, `rgba?(`, `hsla?(`, or one of the 148 CSS named colours), a literal `font-size`, a literal `font-family`, a literal `line-height`, or a `var(--name)` whose name resolves to no token is a violation: `design lint` exits 1 with `DFA-E240`, `DFA-E241`, or `DFA-E242`, and the hook wrapper maps exit 1 to hook exit 2, which blocks the write and returns stderr to the model. `var(--<token>)`, `token(<name>)`, `transparent`, `currentColor`, `inherit`, `none`, and `normal` pass.

**The build gate, non-blocking.** `.devforgeai/gates.toml`, verbatim, the one check in the file that reads Design's output:

```toml
  [[gate.check]]
  kind = "design_tokens"
  id = "build-design"
  paths = []
  severity = "warn"
```

`severity = "warn"` leaves the gate result unchanged, so a token violation that reached the tree some other way annotates the build report and does not fail the build gate. The blocking point is PreToolUse, which stops the write before the file exists.

## Send-back

Design sends back to Discover only, and only from `--spec` mode, and only at step 11, before any `UI-nnn.md` is written.

| Condition | Detected by | IDs cited | Handoff `Next` |
|---|---|---|---|
| A screen the story asks for realizes no requirement | `requirement-coverage-auditor`, `payload.screens_without_req` non-empty | the `UI-nnn` allocated for that screen at step 12 | `/discover <IDEA-nnn> --remedy <UI ids>` |
| A flow in `explore/brief.md` `## Core flows` equals no `requirements[].source` | `requirement-coverage-auditor`, `flows_without_req` non-empty | the `FLOW-nnn` | `/discover <IDEA-nnn> --remedy <FLOW ids>` |

The `IDEA-nnn` on the `Next` line is the top-level `id` key of `.devforgeai/requirements.yaml`, which step 10 loads through `devforgeai doc load requirements -`. It is the project's phase id, one per file, and it is not the `EPIC-nnn` the story belongs to: an epic is a grouping inside that document, and Discover's remedy form re-opens the document by its own id.

On either condition the skill writes no `UI-nnn.md` for the cited screens, writes the covered screens as step 13 defines, and writes `.devforgeai/reports/UI-nnn-design.yaml` with `status: send_back` through the `SubagentStop` ingest. `state.toml` `[current]` is unchanged, because Design advances no phase. `--sketch` emits no send-back: Explore is Phase 0, no requirement exists, and `uncovered_flows` in the sketch return value carries the same fact back to Explore. `--brand` emits no send-back: it reads `personas[]` and writes tokens, and a thin persona is a Discover-side edit that no token depends on.

Design receives a send-back from Plan and from Build as `/design UI-nnn --remedy <ids>`, handled at step 15. `--remedy` carries `AC-nnn` ids (a criterion the spec does not support), `TOKEN-<name>` names (a token the spec references and `tokens.json` does not define), and `UI-nnn` ids (a spec a story references and `.devforgeai/ui-specs/` does not hold). The returning command is `/plan <SPRINT-nnn> --resume` or `/build <STORY-nnn> --resume`, printed on the `Then` line.

## Integration

| Skill | Consumes (doc, IDs) | Produces for (doc, IDs) | Sends back to (condition) | Receives send-back from (condition) | Shared subagents | `state.toml` fields read/written |
|---|---|---|---|---|---|---|
| 0 Explore · `exploring-ideas` | `.devforgeai/explore/sketch-request.json`: `mode`, `idea_id`, `brief_path`, `out_dir`, `seed_data_path`, `fidelity`, `flows[]` (`FLOW-nnn`), `brand`, `constraints`; `.devforgeai/explore/brief.md` `## Core flows` (`FLOW-nnn`) and `## Target user`; `.devforgeai/explore/seed-data.json` `entities[].rows` | the sketch-mode return value: `mode`, `idea_id`, `screens[]` keyed `FLOW-nnn-nn`, `brand_sketch`, `uncovered_flows`, `notes`; the files under `.devforgeai/explore/mockups/`, which fill the brief's `## Mockups` table and, on `promote`, carry forward as the source of `ui-specs/UI-nnn.md` | none — §5 gives Explore's "May send back to" column as `none`, and an uncovered flow travels back as `uncovered_flows`, not as a SEND BACK | none — Explore calls this skill and reads no `UI-nnn` | none; Design owns `mockup-designer` and `brand-designer`, Explore owns `prototype-builder`, and each is invoked only by its owner | none |
| 1 Discover · `discovering-requirements` | `.devforgeai/requirements.yaml`: `personas[]` (`PERSONA-nnn`) for brand mode, `requirements[]` (`REQ-nnn`) and `epics[]` (`EPIC-nnn`) for spec mode, `requirements[].source` for the flow-coverage rule | `.devforgeai/ui-specs/UI-nnn.md` (`UI-nnn`), which Discover records in `requirements[].traces_to` when a re-open from Design allocates a requirement for it | yes: a screen realizes no `REQ-nnn`, cited by `UI-nnn`; a `FLOW-nnn` equals no `requirements[].source`, cited by `FLOW-nnn`; both leave as `/discover <IDEA-nnn> --remedy <ids>`, with the `IDEA-nnn` taken from the top-level `id` key of `requirements.yaml` | none — Discover runs before this skill and cites no `UI-nnn` back to it | none | reads `[current].phase` |
| 2 Constitute · `establishing-context` | none — Design resolves the token file through `config.toml` `[frontend].tokens_path`, and reads no `context/*.md` | `.devforgeai/brand/tokens.json`, named by path and not by value in the `## Design tokens` section Constitute writes, which restates `config.toml` `[frontend].tokens_path` as prose for a reader of the standards and which no CLI command reads | none — a wrong `CON-nnn` is a Constitute-side edit | none — §5 gives Constitute's send-back target as Discover | none | reads `[current].phase` |
| 3 Plan · `planning-work` | `.devforgeai/reports/SPRINT-nnn-plan.yaml` `findings[]` (`UI-nnn`, `AC-nnn`), remedy runs only | `.devforgeai/ui-specs/UI-nnn.md` (`UI-nnn`): Plan attaches the id to a story's frontmatter `consumes` and reads `## Requirements` to confirm the story's `REQ-nnn` set matches, `## States` to size the story, `## Breakpoints` to know the four layouts an `AC-nnn` covers, and `## Out of scope` to keep a second story's work out of this one | none — Design cites `REQ-nnn` gaps to Discover, and a story is Plan's to rewrite | yes: a story references a `UI-nnn` that `.devforgeai/ui-specs/` does not hold, or a `UI-nnn` does not support an `AC-nnn`; arrives as `/design UI-nnn --remedy AC-nnn,...` | none | reads `[current].phase` |
| 4 Build · `implementing-stories` | `.devforgeai/reports/STORY-nnn-build.yaml` `findings[]` (`UI-nnn`, `AC-nnn`, `TOKEN-<name>`), remedy runs only | `.devforgeai/ui-specs/UI-nnn.md` and `.devforgeai/brand/tokens.json`: Build's `frontend-developer` reads `## Anatomy`, `## States`, `## Breakpoints`, `## Interaction`, and `## Accessibility` as the component contract, and the flattened `TOKEN-<group>-<leaf>` names as the only colour and type values it writes; `devforgeai design lint` gates every write it makes to a file matching `[frontend].globs` | none — an implementation defect is Build's to fix | yes: a `TOKEN-<name>` the spec references is undefined, or a state the implementation needs is absent; arrives as `/design UI-nnn --remedy TOKEN-<name>,AC-nnn` | none; `frontend-developer` stays Build's implementer and is not adapted here | reads `[current].phase` |
| 5 Verify · `validating-quality` | none — Design reads no QA report; a Verify finding reaches this skill through Build's `--remedy` | `.devforgeai/ui-specs/UI-nnn.md` (`UI-nnn`): a `FIND-nnn` of kind accessibility names the `UI-nnn` and the `## Accessibility` row it failed, and `.devforgeai/reports/STORY-nnn-qa.yaml` `consumes` carries the `UI-nnn` | none — Design cites requirement gaps, and a failing check is Build's to fix | none — §5 gives Verify's send-back targets as Build and Plan | none | reads `[current].phase` |
| 6 Release · `releasing-software` | none — Design reads no release manifest | `.devforgeai/brand/logo.svg` and `.devforgeai/brand/tokens.json` as shipped assets, and `.devforgeai/brand/brand-kit.md` as the document naming their use; the manifest entry that lists them is `releases/vX.Y.Z.yaml`, whose shape `specs/09-release.md` fixes | none — Design cites requirement gaps, which are upstream of a release | none — §5 gives Release's send-back target as Verify | none | reads `[current].phase` |
| Design · `designing-interfaces` | self | self | self | self | self | reads `[current].phase`; writes none |
| Reflect · `improving-framework` | none — Reflect returns recommendations, which §5 excludes from gates | `.devforgeai/reports/UI-nnn-design.yaml`, written by the `SubagentStop` ingest of `requirement-coverage-auditor`, as one input to `OBS-nnn` extraction | none — Design cites requirement gaps, not framework observations | none — Reflect emits recommendations, not send-backs | none | none |
| CLI · `devforgeai` | `.devforgeai/config.toml` keys `[frontend].globs`, `[frontend].exclude`, `[frontend].tokens_path`; `.devforgeai/state.toml` keys `[current].phase` and `[current].id` | `.devforgeai/brand/tokens.json` for `design lint`; `.devforgeai/ui-specs/UI-nnn.md` for `doc validate` and `doc load ui-spec`; `.devforgeai/reports/UI-nnn-design.yaml` for `report ingest`, `report show`, and `handoff` | none | none | none | reads `[current].phase` and `[current].id`; writes none, because no `phase set` runs in any mode and `[active]` has no design key |

## Handoff

PASS, spec mode, three screens specified:

```
Phase     — · Design        UI-003 · checkout-review
Done      3 UI specs, 41 tokens, 8 accessibility rows each
Gate      PASS  3/3 screens carry a REQ · design lint 0 violations
Verified  requirement-coverage-auditor · 3/3 screens

Next      /build STORY-014
Then      /verify STORY-014
Blocked   none

Full report: .devforgeai/reports/UI-003-design.yaml
```

SEND BACK to Discover, one screen and one flow with no requirement behind them:

```
Phase     — · Design        UI-005 · refund-request
Done      2 of 3 screens specified, 1 held back
Gate      SEND BACK to Discover  UI-005 FLOW-004
Found     UI-005 refund reason picker realizes no REQ
          FLOW-004 is the source of no REQ in EPIC-002

Next      /discover IDEA-004 --remedy UI-005,FLOW-004
Then      /design STORY-021 --resume
Blocked   you: is a refund reason required at submit?

Full report: .devforgeai/reports/UI-005-design.yaml
```

## Templates

### `templates/tokens.json`

```json
{
  "meta": {
    "schema": "devforgeai/tokens/1",
    "id": "TOKEN-001",
    "phase": "design",
    "status": "draft",
    "produced_by": "designing-interfaces",
    "consumes": [],
    "open_questions": []
  },
  "color": {
    "bg":             { "light": "#ffffff", "dark": "#111111" },
    "surface":        { "light": "#ffffff", "dark": "#1a1a1a" },
    "surface-raised": { "light": "#f4f4f4", "dark": "#242424" },
    "border":         { "light": "#d8d8d8", "dark": "#3a3a3a" },
    "text":           { "light": "#111111", "dark": "#f2f2f2" },
    "text-muted":     { "light": "#5a5a5a", "dark": "#a6a6a6" },
    "primary":        { "light": "#1f5f4f", "dark": "#4fbf9f" },
    "on-primary":     { "light": "#ffffff", "dark": "#0d2a23" },
    "accent":         { "light": "#b0520a", "dark": "#e4913f" },
    "success":        { "light": "#1c6b3c", "dark": "#54c07d" },
    "warning":        { "light": "#8a5a00", "dark": "#d9a021" },
    "danger":         { "light": "#a32020", "dark": "#e86464" }
  },
  "type": {
    "family-sans":    "system-ui, sans-serif",
    "family-mono":    "ui-monospace, monospace",
    "size-xs":        "0.75rem",
    "size-sm":        "0.875rem",
    "size-base":      "1rem",
    "size-lg":        "1.25rem",
    "size-xl":        "1.75rem",
    "size-xxl":       "2.5rem",
    "line-tight":     "1.2",
    "line-base":      "1.55",
    "weight-regular": "400",
    "weight-bold":    "650"
  },
  "spacing": {
    "xs": "0.25rem", "sm": "0.5rem", "md": "1rem",
    "lg": "1.5rem",  "xl": "2.5rem", "xxl": "4rem"
  },
  "radius": {
    "none": "0", "sm": "0.25rem", "md": "0.5rem", "full": "9999px"
  },
  "elevation": {
    "flat":    "none",
    "raised":  "0 1px 2px rgba(17, 17, 17, 0.10)",
    "overlay": "0 8px 24px rgba(17, 17, 17, 0.18)"
  },
  "motion": {
    "duration-fast": "120ms",
    "duration-base": "200ms",
    "duration-slow": "360ms",
    "ease-standard": "cubic-bezier(0.2, 0, 0, 1)"
  }
}
```

### `templates/ui-spec.md`

```markdown
---
schema: devforgeai/ui-spec/1
id: UI-nnn
phase: design
status: draft
produced_by: designing-interfaces
consumes: []
open_questions: []
---

# UI-nnn <screen name>

## Purpose

<one sentence: what the person is doing here and what leaves it done>

## Requirements

| REQ | Statement | Satisfied by |
|---|---|---|
| REQ-nnn | <the requirement statement, verbatim> | <Region name from ## Anatomy> |

## Anatomy

| Region | Element | Content source | Tokens |
|---|---|---|---|
| <region name> | <semantic element> | <entity name \| REQ-nnn \| static> | TOKEN-color-<leaf> TOKEN-type-<leaf> TOKEN-spacing-<leaf> |

## States

| State | Trigger | Visible change | Tokens |
|---|---|---|---|
| default | <what puts the screen here> | <what the person sees> | TOKEN-color-<leaf> |
| loading | <what puts the screen here> | <what the person sees> | TOKEN-motion-<leaf> |
| empty | <what puts the screen here> | <what the person sees> | TOKEN-color-text-muted |
| error | <what puts the screen here> | <what the person sees> | TOKEN-color-danger |

## Breakpoints

| Name | Min width | Layout | Changes from previous |
|---|---|---|---|
| sm | 320px | <column count and order> | - |
| md | 768px | <column count and order> | <what moves> |
| lg | 1024px | <column count and order> | <what moves> |
| xl | 1440px | <column count and order> | <what moves> |

## Interaction

| Trigger | Response | Focus after |
|---|---|---|
| <pointer, key, or form event> | <what the screen does> | <Region name \| unchanged> |

## Accessibility

| Check | Requirement | Evidence |
|---|---|---|
| Landmark | <the landmark role wrapping each region> | <Region name> |
| Heading order | <the heading levels in document order> | <Region name> |
| Name | <the accessible name of each control> | <Region name> |
| Role | <the role of each non-native control> | <Region name> |
| Keyboard path | <the tab order, region to region> | <Region name> |
| Focus visible | <the focus indicator> | TOKEN-color-<leaf> |
| Contrast | <the pair measured> | <ratio to one decimal> |
| Motion | <what prefers-reduced-motion: reduce changes> | TOKEN-motion-<leaf> |

## Tokens used

| Token | Group | Where |
|---|---|---|
| TOKEN-<group>-<leaf> | <color \| type \| spacing \| radius \| elevation \| motion> | <Region or State> |

## Out of scope

<one sentence naming a behaviour this screen does not carry, and the UI-nnn that does>
```

### `templates/brand-kit.md`

```markdown
---
schema: devforgeai/brand-kit/1
id: TOKEN-001
phase: design
status: approved
produced_by: designing-interfaces
consumes: []
open_questions: []
---

# <brand name>

## Brand name

<the name>
<the one-sentence descriptor used under the logo>

## Voice

| Trait | Sounds like | Does not sound like |
|---|---|---|
| <trait from the Tone answer> | <one phrase> | <one phrase> |

## Logo

| Field | Value |
|---|---|
| Form | <Wordmark \| Monogram \| Geometric mark \| Wordmark and mark> |
| Path | .devforgeai/brand/logo.svg |
| Clear space | TOKEN-spacing-<leaf> |
| Smallest size | <n>px wide |

## Color

| Token | Light | Dark | Used for | Contrast on bg |
|---|---|---|---|---|
| TOKEN-color-<leaf> | #<hex> | #<hex> | <one phrase> | <ratio to one decimal> |

## Type

| Token | Value | Used for |
|---|---|---|
| TOKEN-type-<leaf> | <value> | <one phrase> |

## Spacing and shape

| Token | Value | Used for |
|---|---|---|
| TOKEN-spacing-<leaf> | <value> | <one phrase> |
| TOKEN-radius-<leaf> | <value> | <one phrase> |

## Motion

| Token | Value | Used for |
|---|---|---|
| TOKEN-motion-<leaf> | <value> | <one phrase> |

<one line: what prefers-reduced-motion: reduce changes>

## Figma

| Field | Value |
|---|---|
| Status | <mirrored \| not mirrored> |
| Collection | <collection name \| -> |
| Reason | <- \| figma plugin not authenticated \| figma plugin not present> |

## Do not

<one sentence in the present tense naming one thing the brand excludes>
```

### `templates/brand-questions.md`

```markdown
Call 1, four questions in one AskUserQuestion invocation:

AskUserQuestion(questions=[
  { "question": "<brand name>: what should it feel like to use?",
    "header": "Tone", "multiSelect": false,
    "options": [
      { "label": "Plain",    "description": "Nothing decorative. Short labels, one accent colour, generous white space." },
      { "label": "Warm",     "description": "Rounded corners, warm neutrals, sentence-case labels that sound like a person." },
      { "label": "Precise",  "description": "Tight grid, cool neutrals, a mono face for numbers and identifiers." },
      { "label": "Bold",     "description": "High contrast, large display sizes, one saturated colour carrying the brand." } ] },
  { "question": "<brand name>: which colour direction?",
    "header": "Color", "multiSelect": false,
    "options": [
      { "label": "Cool neutral",  "description": "Grey-blue surfaces, one green or blue primary. Reads as software." },
      { "label": "Warm neutral",  "description": "Sand and stone surfaces, one terracotta or olive primary. Reads as paper." },
      { "label": "Mono",          "description": "Black, white, and one accent used only on the primary action." },
      { "label": "From sketch",   "description": "The palette in .devforgeai/explore/mockups/brand-sketch.json, adjusted to clear contrast." } ] },
  { "question": "<brand name>: which type pairing?",
    "header": "Type", "multiSelect": false,
    "options": [
      { "label": "One sans",       "description": "A single sans face at six sizes and two weights." },
      { "label": "Sans and mono",  "description": "A sans for prose, a mono for numbers, ids, and code." },
      { "label": "Sans and serif", "description": "A serif for headings, a sans for everything else." },
      { "label": "Humanist sans",  "description": "One humanist sans with wide apertures, at six sizes and two weights." } ] },
  { "question": "<brand name>: how much room between things?",
    "header": "Density", "multiSelect": false,
    "options": [
      { "label": "Compact",     "description": "Dense tables and lists. More rows on screen, smaller targets." },
      { "label": "Comfortable", "description": "The middle. 44px targets, one line of air between rows." },
      { "label": "Spacious",    "description": "Wide gutters and large targets. Fewer rows, easier to scan." } ] }
])

Call 2, one question:

AskUserQuestion(questions=[
  { "question": "<brand name>: what shape is the mark?",
    "header": "Logo", "multiSelect": false,
    "options": [
      { "label": "Wordmark",          "description": "The name drawn as paths, no symbol." },
      { "label": "Monogram",          "description": "The first letter or two in a shape, usable alone at 24px." },
      { "label": "Geometric mark",    "description": "An abstract shape, usable alone at 24px, name set beside it." },
      { "label": "Wordmark and mark", "description": "A symbol and the name, locked together with fixed clear space." } ] }
])

Answer-to-leaf mapping:

| Header | Label | Leaves it sets |
|---|---|---|
| Tone | Plain | radius.sm 0.125rem, radius.md 0.25rem, elevation.raised none, type.weight-bold 600 |
| Tone | Warm | radius.sm 0.375rem, radius.md 0.75rem, elevation.raised a 2px soft shadow, type.weight-bold 600 |
| Tone | Precise | radius.sm 0.125rem, radius.md 0.25rem, elevation.raised a 1px hairline shadow, type.weight-bold 700 |
| Tone | Bold | radius.sm 0.25rem, radius.md 0.5rem, elevation.raised a 4px shadow, type.weight-bold 800 |
| Color | Cool neutral | the 12 color leaves, bg and surface at hue 210 to 230, primary at hue 150 to 210 |
| Color | Warm neutral | the 12 color leaves, bg and surface at hue 30 to 50, primary at hue 10 to 30 or 60 to 90 |
| Color | Mono | the 12 color leaves, bg, surface, border, text and text-muted at chroma 0, primary the one saturated value |
| Color | From sketch | the 12 color leaves, seeded from brand-sketch.json palette, each adjusted until its row in ## Color clears its ratio |
| Type | One sans | family-sans set, family-mono set to the platform mono stack |
| Type | Sans and mono | family-sans set, family-mono set to a named mono face |
| Type | Sans and serif | family-sans set, family-mono set to the platform mono stack, size-xl and size-xxl carrying the serif face |
| Type | Humanist sans | family-sans set to a humanist face, family-mono set to the platform mono stack |
| Density | Compact | spacing.xs 0.125rem, sm 0.25rem, md 0.5rem, lg 1rem, xl 1.5rem, xxl 2.5rem |
| Density | Comfortable | spacing.xs 0.25rem, sm 0.5rem, md 1rem, lg 1.5rem, xl 2.5rem, xxl 4rem |
| Density | Spacious | spacing.xs 0.375rem, sm 0.75rem, md 1.5rem, lg 2.5rem, xl 4rem, xxl 6rem |
| Logo | any of the four | the viewBox content of .devforgeai/brand/logo.svg |
```

### `templates/logo.svg`

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 64" role="img" width="256" height="64">
  <title><brand name></title>
  <path d="" fill="currentColor"/>
</svg>
```

### `templates/sketch-screen.html`

```html
<!doctype html>
<meta charset="utf-8">
<title><screen title></title>
<style>
  :root { --ink: #1a1a1a; --line: #c9c9c9; --paper: #ffffff; --muted: #6b6b6b; }
  body { margin: 0; font: 16px/1.5 system-ui, sans-serif; color: var(--ink); background: var(--paper); }
  header, main, footer { max-width: 960px; margin: 0 auto; padding: 24px; }
  .box { border: 1px solid var(--line); border-radius: 4px; padding: 16px; margin: 0 0 16px; }
  .label { color: var(--muted); font-size: 12px; text-transform: uppercase; letter-spacing: .06em; }
  table { border-collapse: collapse; width: 100%; } td, th { border: 1px solid var(--line); padding: 8px; text-align: left; }
</style>
<header><p class="label"><flow id> · screen <nn> · <state></p><h1><screen title></h1></header>
<main><section class="box"><p class="label"><region name></p><!-- seed rows --></section></main>
<footer><p class="label"><the next screen, or the outcome the flow names></p></footer>
```

## Evals

Shipped at `skills/designing-interfaces/evals/`.

### `evals/evals.json` — 8 entries, skill-creator format

| # | `prompt` | `expected_output` | `expectations[]` |
|---|---|---|---|
| 1 | `/design --brand EPIC-002` with `requirements.yaml` holding two personas and the four answers `Plain`, `Cool neutral`, `One sans`, `Comfortable`, then `Wordmark` | `.devforgeai/brand/tokens.json` at `meta.status: approved`, `.devforgeai/brand/logo.svg`, `.devforgeai/brand/brand-kit.md` | the seven top-level keys appear in the order `meta`, `color`, `type`, `spacing`, `radius`, `elevation`, `motion`; every `color` leaf is an object with exactly `light` and `dark`; every other group's leaves are strings; the leaf counts are 12, 12, 6, 4, 3, 4; `meta` holds the seven §5 keys in §5 order |
| 2 | `/design --brand EPIC-002` where `requirements.yaml` holds one persona and no brand sketch exists | `brand-kit.md` with all nine headings | the nine headings appear in template order; `## Color` holds 12 rows; `## Type` holds 12 rows; `## Spacing and shape` holds 10 rows; `## Motion` holds 4 rows; `## Figma` holds exactly the rows `Status`, `Collection`, `Reason` |
| 3 | `/design --brand EPIC-002` in a session with no Figma plugin tools | `brand-kit.md` with `Status: not mirrored` | `## Figma` `Status` is `not mirrored`; `Reason` is `figma plugin not present`; `tokens.json` exists at `meta.status: approved`; `logo.svg` exists; the run printed a PASS handoff |
| 4 | `/design --spec STORY-014` with `tokens.json` on disk and a story citing `REQ-007` and `REQ-011` | two files under `.devforgeai/ui-specs/` at `status: approved` | each file holds the nine headings in template order; `## Breakpoints` holds exactly the four rows `sm`, `md`, `lg`, `xl` with `320px`, `768px`, `1024px`, `1440px`; `## Accessibility` holds exactly the eight named checks in order; `## States` holds a `default` row |
| 5 | `/design --spec STORY-014` with `tokens.json` on disk | `UI-nnn.md` bodies carrying no colour or type literal | no body line matches `#[0-9a-fA-F]{3,8}\b`, `rgba?\(`, or `hsla?\(`; no body line matches `[0-9.]+(px\|rem\|em\|pt)` outside the `## Breakpoints` `Min width` column; every token string matches `^TOKEN-[a-z][a-z0-9-]*-[a-z][a-z0-9-]*$`; every `## Tokens used` row names a token that `tokens.json` defines |
| 6 | `/design --spec STORY-021` where one screen the story names matches no `REQ-nnn` and one `FLOW-nnn` in the brief sources no requirement | a SEND BACK handoff to Discover, no `UI-nnn.md` for the uncovered screen | the `Gate` line reads `SEND BACK to Discover`; the `Next` line is `/discover IDEA-nnn --remedy UI-nnn,FLOW-nnn` with no other flag, and its `IDEA-nnn` equals the `id` key of `requirements.yaml`; the `Then` line is `/design STORY-021 --resume`; `.devforgeai/ui-specs/` holds no file for the uncovered screen; the covered screens were written |
| 7 | `/design UI-003 --remedy AC-009` with `UI-003.md` on disk and a Build report citing `AC-009` | `UI-003.md` with `## States`, `## Interaction`, and `## Accessibility` changed and every other section byte-identical | `## Purpose`, `## Requirements`, `## Anatomy`, `## Breakpoints`, `## Tokens used`, and `## Out of scope` are byte-identical to the prior file; at least one of the three named sections differs; `status` ends at `approved`; the `Then` line is `/build STORY-nnn --resume` |
| 8 | `/design --sketch IDEA-001` with `sketch-request.json` holding four flows | the sketch-mode return value and files under `.devforgeai/explore/mockups/` | `mode` is `sketch` and `idea_id` echoes the request; every `screens[].screen` matches `^FLOW-[0-9]{3}-[0-9]{2}$` and its `flow_id` is one of the request's flows; `.devforgeai/brand/tokens.json` was not written; `.devforgeai/ui-specs/` holds no new file; `uncovered_flows` holds only ids from the request |

Entries 6, 7, and 8 exercise the send-back path: 6 emits the send-back to Discover, 7 is the receiving side of a send-back from Build, and 8 confirms that sketch mode emits `uncovered_flows` in place of one.

### `evals/cases.jsonl` — 8 lines

```json
{"id": "dz-01-brand-tokens", "prompt": "/design --brand EPIC-002", "answers": {"Tone": "Plain", "Color": "Cool neutral", "Type": "One sans", "Density": "Comfortable", "Logo": "Wordmark"}, "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-node.toml", ".devforgeai/state.toml": "schema = \"devforgeai/state/1\"\nupdated_at = \"2026-09-10T09:00:00Z\"\n[current]\nphase = \"constitute\"\nid = \"IDEA-004\"\n", ".devforgeai/requirements.yaml": "FIXTURE:requirements-2-personas.yaml", ".devforgeai/context/coding-standards.md": "FIXTURE:coding-standards.md", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml"}}, "expect": {"grader": "tokens_shape", "args": {"path": ".devforgeai/brand/tokens.json", "groups": ["meta", "color", "type", "spacing", "radius", "elevation", "motion"], "leaf_counts": {"color": 12, "type": 12, "spacing": 6, "radius": 4, "elevation": 3, "motion": 4}, "color_keys": ["light", "dark"], "status": "approved"}}}
{"id": "dz-02-brand-kit-sections", "prompt": "/design --brand EPIC-002", "answers": {"Tone": "Warm", "Color": "Warm neutral", "Type": "Sans and mono", "Density": "Spacious", "Logo": "Monogram"}, "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-node.toml", ".devforgeai/state.toml": "schema = \"devforgeai/state/1\"\nupdated_at = \"2026-09-10T09:00:00Z\"\n[current]\nphase = \"constitute\"\nid = \"IDEA-004\"\n", ".devforgeai/requirements.yaml": "FIXTURE:requirements-1-persona.yaml", ".devforgeai/context/coding-standards.md": "FIXTURE:coding-standards.md", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml"}}, "expect": {"grader": "brand_kit_shape", "args": {"path": ".devforgeai/brand/brand-kit.md", "headings": ["Brand name", "Voice", "Logo", "Color", "Type", "Spacing and shape", "Motion", "Figma", "Do not"], "row_counts": {"Color": 12, "Type": 12, "Spacing and shape": 10, "Motion": 4}, "figma_rows": ["Status", "Collection", "Reason"]}}}
{"id": "dz-03-figma-absent", "prompt": "/design --brand EPIC-002", "answers": {"Tone": "Precise", "Color": "Mono", "Type": "Humanist sans", "Density": "Compact", "Logo": "Geometric mark"}, "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-node.toml", ".devforgeai/state.toml": "schema = \"devforgeai/state/1\"\nupdated_at = \"2026-09-10T09:00:00Z\"\n[current]\nphase = \"constitute\"\nid = \"IDEA-004\"\n", ".devforgeai/requirements.yaml": "FIXTURE:requirements-1-persona.yaml", ".devforgeai/context/coding-standards.md": "FIXTURE:coding-standards.md", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml"}}, "expect": {"grader": "figma_fallback", "args": {"path": ".devforgeai/brand/brand-kit.md", "status": "not mirrored", "reasons": ["figma plugin not present", "figma plugin not authenticated"], "present": [".devforgeai/brand/tokens.json", ".devforgeai/brand/logo.svg"]}}}
{"id": "dz-04-spec-sections", "prompt": "/design --spec STORY-014", "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-node.toml", ".devforgeai/state.toml": "schema = \"devforgeai/state/1\"\nupdated_at = \"2026-09-10T09:00:00Z\"\n[current]\nphase = \"plan\"\nid = \"EPIC-002\"\n", ".devforgeai/requirements.yaml": "FIXTURE:requirements-2-personas.yaml", ".devforgeai/stories/STORY-014.md": "FIXTURE:story-014.md", ".devforgeai/brand/tokens.json": "FIXTURE:tokens-approved.json", ".devforgeai/context/coding-standards.md": "FIXTURE:coding-standards.md", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml"}}, "expect": {"grader": "ui_spec_shape", "args": {"dir": ".devforgeai/ui-specs", "headings": ["Purpose", "Requirements", "Anatomy", "States", "Breakpoints", "Interaction", "Accessibility", "Tokens used", "Out of scope"], "breakpoints": [["sm", "320px"], ["md", "768px"], ["lg", "1024px"], ["xl", "1440px"]], "a11y_checks": ["Landmark", "Heading order", "Name", "Role", "Keyboard path", "Focus visible", "Contrast", "Motion"], "min_files": 1}}}
{"id": "dz-05-token-only-values", "prompt": "/design --spec STORY-014", "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-node.toml", ".devforgeai/state.toml": "schema = \"devforgeai/state/1\"\nupdated_at = \"2026-09-10T09:00:00Z\"\n[current]\nphase = \"plan\"\nid = \"EPIC-002\"\n", ".devforgeai/requirements.yaml": "FIXTURE:requirements-2-personas.yaml", ".devforgeai/stories/STORY-014.md": "FIXTURE:story-014.md", ".devforgeai/brand/tokens.json": "FIXTURE:tokens-approved.json", ".devforgeai/context/coding-standards.md": "FIXTURE:coding-standards.md", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml"}}, "expect": {"grader": "tokens_only", "args": {"dir": ".devforgeai/ui-specs", "tokens": ".devforgeai/brand/tokens.json", "exempt_columns": {"Breakpoints": ["Min width"], "Accessibility": ["Evidence"]}}}}
{"id": "dz-06-sendback-discover", "prompt": "/design --spec STORY-021", "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-node.toml", ".devforgeai/state.toml": "schema = \"devforgeai/state/1\"\nupdated_at = \"2026-09-10T09:00:00Z\"\n[current]\nphase = \"plan\"\nid = \"EPIC-002\"\n", ".devforgeai/requirements.yaml": "FIXTURE:requirements-gap.yaml", ".devforgeai/stories/STORY-021.md": "FIXTURE:story-021-uncovered.md", ".devforgeai/explore/brief.md": "FIXTURE:brief-4-flows.md", ".devforgeai/brand/tokens.json": "FIXTURE:tokens-approved.json", ".devforgeai/context/coding-standards.md": "FIXTURE:coding-standards.md", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml"}}, "expect": {"grader": "sendback_block", "args": {"to": "Discover", "next_prefix": "/discover IDEA-004 --remedy", "must_cite": ["UI-", "FLOW-004"], "forbidden_substrings": ["--from", "EPIC-"], "then_line": "/design STORY-021 --resume", "dir": ".devforgeai/ui-specs", "max_files": 1, "report_phase": "design", "absent": [".devforgeai/ui-specs/UI-004.md"]}}}
{"id": "dz-07-remedy-touches-cited", "prompt": "/design UI-003 --remedy AC-009", "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-node.toml", ".devforgeai/state.toml": "schema = \"devforgeai/state/1\"\nupdated_at = \"2026-09-10T09:00:00Z\"\n[current]\nphase = \"build\"\nid = \"STORY-014\"\n", ".devforgeai/requirements.yaml": "FIXTURE:requirements-2-personas.yaml", ".devforgeai/stories/STORY-014.md": "FIXTURE:story-014.md", ".devforgeai/ui-specs/UI-003.md": "FIXTURE:ui-003-approved.md", ".devforgeai/reports/STORY-014-build.yaml": "FIXTURE:build-sendback.yaml", ".devforgeai/brand/tokens.json": "FIXTURE:tokens-approved.json", ".devforgeai/context/coding-standards.md": "FIXTURE:coding-standards.md", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml"}}, "expect": {"grader": "remedy_touched_only", "args": {"path": ".devforgeai/ui-specs/UI-003.md", "changed_any_of": ["States", "Interaction", "Accessibility"], "unchanged_sections": ["Purpose", "Requirements", "Anatomy", "Breakpoints", "Tokens used", "Out of scope"], "end_status": "approved", "baseline_section_sha256": {"Purpose": "ec9b0fe71e4bbaa25177e9877540fef5a9d0c55025c517e85f695c7c6d45c9e9", "Requirements": "f52362cc2b8f24d9573ca6b7974761c75ce2b4e91bb4048061d6c02648d787b4", "Anatomy": "7abcd58b364ae3c0756e2bf01ee9c30cfabe45858b47eca7e6452fb656c49523", "States": "bbfae8562ab471981d2e5fe318edc9758b041ce6322e816840921629ef092314", "Breakpoints": "3588a71de3eba3d5ae5713f73e2b2afa254ef7a6979a126f5e302326df00f00e", "Interaction": "c95e2570eba949c1b9371a3ea768da6e962dd6405e2fff3f5a0aa022326163d0", "Accessibility": "becb5c7c6385288fa83b44ee1b33ec34a459513a71a5d473832913aa09c68198", "Tokens used": "db7bc14ef21f0aff36404c5026c64afef33c8fe00b7d5a98f40e9ec4481f1013", "Out of scope": "c626bbd74d51090742f7fde8a59044f93c56639554d729beb9bef68b04722071"}}}}
{"id": "dz-08-sketch-contract", "prompt": "/design --sketch IDEA-001", "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-node.toml", ".devforgeai/state.toml": "schema = \"devforgeai/state/1\"\nupdated_at = \"2026-09-10T09:00:00Z\"\n[current]\nphase = \"explore\"\nid = \"IDEA-001\"\n", ".devforgeai/explore/brief.md": "FIXTURE:brief-4-flows.md", ".devforgeai/explore/seed-data.json": "FIXTURE:seed-data.json", ".devforgeai/explore/sketch-request.json": "FIXTURE:sketch-request-4-flows.json", ".devforgeai/gates.toml": "FIXTURE:gates-default.toml"}}, "expect": {"grader": "sketch_contract", "args": {"request": ".devforgeai/explore/sketch-request.json", "out_dir": ".devforgeai/explore/mockups", "absent": [".devforgeai/brand/tokens.json"], "empty_dirs": [".devforgeai/ui-specs"]}}}
```

Fixture files named `FIXTURE:<name>` live at `skills/designing-interfaces/evals/fixtures/<name>`, and the shared runner copies them in as it copies any `setup.files` entry. The digests in `baseline_section_sha256` are the SHA-256 of the named sections of `fixtures/ui-003-approved.md`, recorded in the case line so no grader depends on a runner behaviour beyond the §9 contract; `skills/designing-interfaces/evals/fixtures/digests.txt` records how each was produced.

### `evals/graders.py` — signatures and logic

Every function has the §9 signature `def <name>(workspace: str, transcript: str, args: dict) -> tuple[bool, str]`, reads only files under `workspace` and the `transcript` string, and returns `(passed, evidence)`.

- `tokens_shape(workspace, transcript, args)` — parse `args["path"]` as JSON with `json.load`. Compare `list(obj.keys())` to `args["groups"]` for equality including order. Compare `obj["meta"]` key order to the seven §5 keys. Check `obj["meta"]["status"] == args["status"]`. For each group and count in `args["leaf_counts"]`, check the leaf count and that every leaf name matches `^[a-z][a-z0-9-]*$`. For `color`, check every leaf is a dict whose key set equals `set(args["color_keys"])` and whose values match `^#[0-9a-f]{6}$`; for every other group, check every leaf is a `str`. Evidence: the group order found and the leaf counts found.
- `brand_kit_shape(workspace, transcript, args)` — parse `args["path"]`. Split the body on `^## ` and compare the heading sequence to `args["headings"]`. For each heading and count in `args["row_counts"]`, count markdown table data rows under that heading and compare. Under `Figma`, read the first column of each data row and compare to `args["figma_rows"]` for equality including order. Evidence: the heading sequence and the per-heading row counts.
- `figma_fallback(workspace, transcript, args)` — parse `args["path"]`, read the `## Figma` table into a dict. Check `Status == args["status"]` and `Reason in args["reasons"]`. For each path in `args["present"]`, require it exists under `workspace` and is non-empty. Scan `transcript` for the substring `Gate      PASS` and require one match. Evidence: the three Figma rows and the paths checked.
- `ui_spec_shape(workspace, transcript, args)` — glob `args["dir"]` for `UI-[0-9][0-9][0-9].md`, require at least `args["min_files"]`. For each file: compare the frontmatter key order to the seven §5 keys; compare the `^## ` heading sequence to `args["headings"]`; read the `Breakpoints` table's first two columns and compare to `args["breakpoints"]` for equality including order; read the `Accessibility` table's first column and compare to `args["a11y_checks"]` for equality including order; require a `States` row whose first cell is `default`. Evidence: the file names and each file's heading sequence.
- `tokens_only(workspace, transcript, args)` — flatten `args["tokens"]` to the name set `{"TOKEN-" + group + "-" + leaf}` for every group other than `meta`. For each `UI-*.md` under `args["dir"]`, take the body below the frontmatter, drop the table columns named in `args["exempt_columns"]` by header position, then require no remaining line to match `#[0-9a-fA-F]{3,8}\b`, `rgba?\(`, `hsla?\(`, or `[0-9.]+(px|rem|em|pt)`. Require every substring matching `TOKEN-[a-z0-9-]+` to be in the flattened name set. Evidence: the offending line numbers, or the count of tokens resolved.
- `sendback_block(workspace, transcript, args)` — scan `transcript` for a line starting `Gate      SEND BACK to ` followed by `args["to"]`. Require a line starting `Next      ` whose text starts with `args["next_prefix"]`. Require every string in `args["must_cite"]` to appear on that `Next` line. Require no string in `args["forbidden_substrings"]` to appear on it. Require a line starting `Then      ` whose text equals `args["then_line"]` after stripping trailing spaces. Count files matching `UI-[0-9][0-9][0-9].md` under `args["dir"]` and require at most `args["max_files"]`. Evidence: the `Gate` and `Next` lines, and the file names found.
- `remedy_touched_only(workspace, transcript, args)` — split `args["path"]` on `^## ` into `{heading: body}`. The prior state arrives in `args`, not from the workspace: `args["baseline_section_sha256"]` maps a heading to the SHA-256 hex digest of its body before the run, computed over the section text with trailing whitespace stripped per line. Require every heading in `args["unchanged_sections"]` to hash to its recorded digest. Require at least one heading in `args["changed_any_of"]` to hash to something other than its recorded digest, when one is recorded, and otherwise to be non-empty. Parse the frontmatter and require `status == args["end_status"]`. Evidence: the headings whose digest matched and those that did not.
- `sketch_contract(workspace, transcript, args)` — parse `args["request"]` as JSON for its `flows[].flow_id` list. Scan `transcript` for the last JSON object holding the key `"screens"`, parse it, and require `mode == "sketch"`, `idea_id` equal to the request's `idea_id`, every `screens[].screen` matching `^FLOW-[0-9]{3}-[0-9]{2}$` with its `flow_id` in the request list, every `screens[].path` existing under `workspace` beneath `args["out_dir"]`, and every entry of `uncovered_flows` in the request list. For each path in `args["absent"]` require it does not exist; for each directory in `args["empty_dirs"]` require it is absent or holds no file. Evidence: the screen ids and the paths checked.

No grader opens a network connection, starts a subprocess, calls a model, or uses a random source, and none depends on a runner behaviour beyond the §9 contract: the runner copies `setup.files` into a temp workspace, invokes `claude -p`, and calls the grader with `workspace`, `transcript`, and `args`. Prior-state comparisons travel in `args`. `python -c "import graders"` needs only the standard library.

## Decisions

1. **Accepted. `tokens.json` leaves of `color` are objects with the keys `light` and `dark`; every other group's leaves are strings.** `specs/01-cli.md` `### design lint` now carries this grammar and the union rule for `DFA-E240`, so this is settled rather than proposed. The grammar, exactly: a group's leaf is a string in `type`, `spacing`, `radius`, `elevation`, and `motion`; a leaf of `color` is an object with exactly the keys `light` and `dark`, each a string. The flattened token name stays `TOKEN-<group>-<leaf>` in every group — one name, and for `color` two values — and the CSS custom property stays `--<group>-<leaf>`. `DFA-E240`'s nearest token is the token with the smallest Euclidean distance in sRGB to any value in the union of the `light` and `dark` values, and the error names the token, not the theme. The alternative, two groups `color-light` and `color-dark`, needs no CLI change and costs every frontend file a theme-qualified reference like `var(--color-light-primary)`, which puts theme switching in every call site instead of in one token.
2. **Accepted. `design lint --tokens` is part of the CLI surface.** `specs/01-cli.md` `### design lint` carries the grammar `devforgeai design lint [<paths>...] [--tokens] [--json] [--project <path>]`, the two error codes, and `DFA-E010` for `--tokens` given a path argument. With `--tokens` the command reads no frontend file. It checks that the token file at `config.toml` `[frontend].tokens_path` parses, that its top-level keys are `meta` plus the six groups, that `meta` holds the seven §5 keys, that every leaf name matches `^[a-z][a-z0-9-]*$`, that `color` leaves are `{light, dark}` objects and other groups' leaves are strings, and that every substring matching `TOKEN-[a-z0-9-]+` in `.devforgeai/ui-specs/UI-*.md` flattens to a defined token. Error codes: `DFA-E243` (tokens group or leaf shape) and `DFA-E244` (undefined token in a UI spec). Exit 0 with no problem, 1 on `DFA-E120`, `DFA-E121`, `DFA-E243`, `DFA-E244`, 3 on `DFA-E010`. §1 rule 1 puts this check in the CLI: it is a structural validation, and a skill that performed it would be the ceremony §2 forbids.
3. **Accepted. `design` is a `--phase` value for `handoff`, `report ingest`, and `report show`, and for the report path `reports/<ID>-design.yaml`, and for no gate command.** `specs/01-cli.md` `### handoff` accepts the seven phase names and `design`, with `--id` taken from the argument for `design`, which has no `[active]` key; `### report ingest` writes a `design` verifier's block into `reports/<UI-nnn>-design.yaml`; the transition table gives the `design` row. `gate check --phase design` and `gate require design <id>` stay undefined and exit 1 with `DFA-E300`, because `gates.toml` holds one gate per phase and Design is not a phase. No hook calls either one with `design`: §7 gives the Stop hook `gate check --phase <[current].phase>`, and Design leaves `[current]` untouched.
4. **Accepted. A `/design` run prints two handoff blocks: Design's, then the current phase's.** `specs/01-cli.md` `### handoff` states the same: the skill's last workflow step runs `handoff --phase design --id <UI-nnn>`, and the Stop hook then runs `gate check --phase <[current].phase>` and `handoff` for the phase the user is in. Both come from the CLI; the model composes neither. The alternative, suppressing the Stop hook's block, would amend §7, which is labelled fixed, for a cosmetic gain.
5. **Accepted. The `Phase` line renders Design's `<n>` as an em dash.** §6 fixes the line as `Phase     <n> · <Name>        <ID> · <slug>`, and §5 gives Design no number in the phase sequence. `Phase     — · Design` keeps the separator, the column-11 start, and the twelve-line cap.
6. **The send-back to Discover prints `/discover <IDEA-nnn> --remedy <ids>` with no `--from` flag.** §4c fixes the send-back grammar as `/<upstream> <ID> --remedy <ID>,<ID>` and states that no other flag names are used for it. The `<ID>` is Discover's own phase id: `.devforgeai/requirements.yaml` carries one top-level `id: IDEA-nnn` for the whole document, and step 10 already has it from `devforgeai doc load requirements -`. An `EPIC-nnn` names a grouping inside that document, not the document, so it is not the id the remedy form re-opens. `specs/03-discover.md` `## Integration` describes the arrival as carrying `--from design` and as keyed on `EPIC-nnn`; its `## Command` `argument-hint` carries neither `--from` nor a document-level meaning for `EPIC-nnn`. The `UI-` and `FLOW-` prefixes in `--remedy` identify Design as the source without a flag. Both corrections belong in the Discover Integration row.
7. **The send-back cites `UI-nnn` and `FLOW-nnn`, and Discover's entry point C handles both.** `specs/03-discover.md` `## Integration` names `UI-nnn` alone and allocates one `REQ-nnn` per cited id with `traces_to: [UI-nnn]` and `source: user`. A cited `FLOW-nnn` takes the same path with `source` set to that `FLOW-nnn`, which is already a member of the `requirement source` enum, and `traces_to: []`. Collapsing the two into one by allocating a `UI-nnn` stub for an uncovered flow would put a screen id on something no screen exists for.
8. **`TOKEN-<name>` is a token reference, not an ID-index entry.** §5 lists `TOKEN-<name>` beside `UI-nnn` as a Design ID prefix, and `specs/01-cli.md` builds the ID index from `\b[A-Z]+-[0-9]{3}\b`, which no lowercase, non-numeric token name matches. So `doc validate` neither defines nor resolves a token name, `consumes` holds none, and `design lint --tokens` is what resolves them. This is the reason Decision 2 exists rather than folding the check into `doc validate`.
9. **Token leaf names are lowercase and begin with a letter.** The leaf pattern is `^[a-z][a-z0-9-]*$`, which is why the spacing and type scales read `xxl` rather than `2xl`. A leaf like `2xl` would be legal JSON and would put a digit where the ID index and the CSS custom-property grammar both read a name.
10. **`produced_by` is `designing-interfaces` in `tokens.json`, `brand-kit.md`, and every `UI-nnn.md`.** §5 gives the frontmatter key as `<skill-name>` and §4b fixes the skill name. `specs/01-cli.md` `## Document registry` gives the producer of `ui-spec` and `tokens` as `devforgeai-design`, and gives every other row a `devforgeai-<phase>` name that disagrees with §4b in the same way. The correction applies to all fourteen rows, not to Design's two.
11. **`UI-nnn.md` has a two-value `status` enum, `draft` and `approved`, and a remedy run reuses `draft`.** `specs/01-cli.md` gives the `ui-spec` enum as those two values. A third value, `reopened`, would carry one fact that the run's own `--remedy` argument and the send-back report already carry, at the cost of a fourth amendment to the CLI spec.
12. **Breakpoints are fixed at `sm` 320px, `md` 768px, `lg` 1024px, `xl` 1440px, in the `## Breakpoints` template, and are not tokens.** The task fixes the six token groups, and `design lint` inspects colour and type properties, not media queries, so a breakpoint in `tokens.json` would be a value nothing resolves against. The four widths are the set `frontend-developer` already implements against, which keeps Build reading one list.
13. **`brand-kit.md` carries `id: TOKEN-001`, the same id as `tokens.json`.** `specs/01-cli.md` scopes ID uniqueness per `schema` value, and the two files carry `devforgeai/brand-kit/1` and `devforgeai/tokens/1`. The kit is the prose face of the token file and shares its subject; a second id would invent a prefix §5 does not list.
14. **Brand mode asks exactly five questions in two `AskUserQuestion` calls.** The tool takes at most four questions per call, and the fifth question, `Logo`, depends on none of the first four, so the split costs the user nothing and keeps the set fixed rather than adaptive. Every answer maps to named leaves through the table in `templates/brand-questions.md`, so the same five answers produce the same token set.
15. **Spec mode stops when `.devforgeai/brand/tokens.json` is absent.** Every colour and type value in a `UI-nnn.md` is a `TOKEN-<name>`, so a spec written before the brand kit would reference names nothing defines and fail `design lint --tokens` at step 14. Step 10 writes no file and the handoff carries `Blocked   you: run /design --brand <EPIC-nnn> first`. Brand mode before spec mode is the one ordering this skill imposes on itself.
16. **Sketch mode emits no SEND BACK.** It runs inside Explore's Phase 0, where `requirements.yaml` does not exist, so no screen can be tested against a `REQ-nnn`. A flow that produced no screen travels back as `uncovered_flows`, which `specs/02-explore.md` step 6 turns into one `open_questions` line per id.
17. **The built-in `design` skill is used by `mockup-designer` for layout and visual judgment, and no Design output depends on an Artifact.** §2 lists the primitives a spec may rely on, and the `Artifact` tool is not among them. The agent holds `Write` and writes the HTML files into `out_dir` itself, so the mode completes whether or not the skill publishes anything.
18. **The built-in `frontend-design:frontend-design` skill is used by `brand-designer` at step 7 and by `ui-spec-writer` at step 13, for palette, type, and composition judgment. It writes no file this skill's outputs depend on.** Its role is the aesthetic direction the two agents then express as token values and `## Anatomy` rows.
19. **Three Figma skills are invoked, and the Figma path is optional in every mode.** `figma:figma-use` and `figma:figma-generate-library` at step 9 mirror the six token groups into a variable collection with a light and a dark mode; `figma:figma-design-to-code` at step 13 reads a node when the story cites a node URL. Every other Figma skill in the plugin — `figma:figma-code-connect`, `figma:figma-create-new-file`, `figma:figma-generate-diagram`, `figma:figma-generative-plugins`, `figma:figma-implement-motion`, `figma:figma-shaders`, `figma:figma-swiftui`, `figma:figma-use-figjam`, `figma:figma-use-motion`, `figma:figma-use-slides` — is not invoked by this skill. Figma tools reach the session through an MCP plugin, which §2's primitive list does not name, so the fallback is the normal path: the plugin absent or a call returning an authentication error leaves `brand-kit.md` `## Figma` at `Status: not mirrored` with the reason recorded, leaves `## Anatomy` citing the mockup path or the `REQ-nnn` in place of the node, and changes no other output.
20. **`frontend-developer` is not adapted here.** It writes production components, which is Build's work; §5 gives Design's outputs as `brand/tokens.json` and `ui-specs/UI-nnn.md` and no source file. `mockup-designer` derives from it, keeping its semantic-element and breakpoint judgment and dropping the framework patterns, the `Bash(npm:*)` tool, the test step, and the observation file. Build's own `frontend-developer` gains two inputs this spec fixes: `UI-nnn.md` as the component contract and `tokens.json` as the only source of colour and type values, with `devforgeai design lint` gating each write.
21. **`ui-spec-formatter` is replaced by `ui-spec-writer`.** It reads a finished spec and produces a display template for a command's output, which §2 places outside a skill: the handoff is printed by `devforgeai handoff`, not composed from a subagent's JSON. Its slot is taken by an agent that writes the document instead of formatting it.
22. **`stakeholder-analyst` is not invoked.** Personas arrive at this skill already written, as `PERSONA-nnn` records with `name`, `description`, and `goal` in `requirements.yaml`. Design reads them; Discover discovers them. A second interview would ask the user for facts one document already holds.
23. **Design writes no `state.toml` field and runs no `phase set`.** §5 gives Design no phase number; `specs/01-cli.md` `[active]` has one key per phase and none for Design, `[current].phase` takes the phase enum which excludes `design`, and the phase-scoped tables are `[explore]` and `[constitute]` alone. A `phase set` would move the user out of the phase they were in. Design reads `[current].phase` and `[current].id` and writes neither. Resume state lives in the documents and the report: `meta.status` in `tokens.json`, `status` in each `UI-nnn.md`, and the latest `reports/UI-nnn-design.yaml` at `status: send_back` per Decision 26.
24. **This skill carries no preamble line at all.** A `!` line runs unconditionally and before the mode is known, and every candidate fails in some mode: `gate require` has no gate to test, `doc load requirements -` fails in sketch mode where `requirements.yaml` does not exist, and `design lint --tokens` fails in brand mode before `tokens.json` is written. The allocation was the last candidate standing, and it fails too — a `--sketch` or `--brand` run allocates no `UI-nnn`, so a preamble allocation would spend an id those modes discard and would abort them outright on an exhausted `UI` prefix they never touch. `doc validate --allocate UI` therefore runs at workflow step 12, once per screen that carries no id, which is the point at which the run knows it needs one. `specs/02-explore.md` Decision 26 moves Explore's allocation for the same reason.
25. **The send-back reports Design reads are CLI-written, and `findings[]` is the CLI's key.** `specs/01-cli.md` `.devforgeai/reports/<ID>-<phase>.yaml` carries a top-level `findings: []`, described there as the merged findings across verifiers and the source of the handoff's `Found` lines. Step 15 reads that key of `.devforgeai/reports/SPRINT-nnn-plan.yaml` and `.devforgeai/reports/STORY-nnn-build.yaml` and takes the `UI-nnn`, `AC-nnn`, and `TOKEN-<name>` ids from its entries. `specs/05-plan.md` and `specs/06-build.md` are unwritten; whichever entry shape they specify, the key Design reads is the CLI's, and the ids that reach step 15 are the ids on the `--remedy` argument, which §4c fixes.
26. **`--resume` applies to `--brand`, to a partial `--spec` run, and to the return from a send-back, per §4c.** Resume state lives in two places, neither of them `state.toml`, which Decision 23 leaves untouched. The first is document `status`: `--brand --resume` continues at the first step whose output is absent or at `meta.status: draft`, and `--spec --resume` writes the screens that have no `UI-nnn.md` on disk. The second is the report: `.devforgeai/reports/<UI-nnn>-design.yaml` with `status: send_back` holds the `UI-nnn` and `FLOW-nnn` ids the send-back cited, in its `findings[]`, and `--resume` reads the most recently modified `reports/UI-*-design.yaml` whose `status` is `send_back` to find the blocked id. Step 11 then re-runs the coverage audit for that screen alone against the `REQ-nnn` Discover allocated, and step 13 writes its `UI-nnn.md`. Every `UI-nnn.md` already at `status: approved` keeps every byte. The `Then` line after a send-back therefore reads `/design <STORY-nnn> --resume`.
27. **Blockers: none, and no open CLI ask.** Every step runs on the §2 primitive list: Read, Write, Edit, Bash, PowerShell, Grep, Glob, Agent, `AskUserQuestion` in the main session, Skill, the hooks of §7, one skill that is its own entry point, four subagents, and the `devforgeai` binary.

28. **The `figma:*` and `frontend-design:*` skill names are sanctioned proper nouns.** Conventions §1 rule 2 names the second sanctioned class: a first-party Claude Code capability is addressed by the name the harness gives it, and a skill that means to reach one has no other spelling. `HTML` and `SVG` are output formats of a wireframe and a logo rather than a project's toolchain, and neither reaches `config.toml`. No language, package manager, test runner, or build tool is named in this spec or in the skill it specifies.

29. **The Figma call belongs to the skill, not to `ui-spec-writer`.** A subagent's tool set is its `tools` list plus its own `mcpServers`, so an MCP tool the parent session holds does not reach it and a `Skill` grant would load `figma:figma-design-to-code` into an agent with nothing for it to drive. Workflow step 13 runs the call in the main conversation and passes what it returns as the `figma_context` field; `""` covers every case where no node URL exists, the plugin is absent, or the call returned an authentication error. The three additions this spec asked for are accepted in `specs/01-cli.md` and recorded as such in Decisions 1, 2, and 3. The one check kind this spec names, `design_tokens`, is a member of the CLI's closed twenty-one-kind enum. The Figma path of Decision 19 is the one capability outside the primitive list, and no output depends on it.
