# `/design` — a worked walkthrough

Cross-cutting, not a phase. Design has no number in the phase sequence, no gate in `gates.toml`, and no key in `state.toml` `[active]`. A run leaves `[current].phase` and `[current].id` as it found them.

```
$ devforgeai gate check --phase design --id UI-001
devforgeai: DFA-E300 gate check: gates.toml has no gate for phase 'design'
```

That is by design, not a defect: §5 gives one gate per phase, and Design is not a phase.

---

## Entry — one command, three modes

| Argument form | Mode | `$1` |
|---|---|---|
| `--sketch IDEA-nnn`, or a Skill invocation carrying `"mode": "sketch"` | sketch | `IDEA-nnn` |
| `--brand EPIC-nnn` | brand | `EPIC-nnn` |
| `--spec STORY-nnn`, `--spec EPIC-nnn`, or a bare `STORY-nnn` / `EPIC-nnn` | spec | `STORY-nnn` or `EPIC-nnn` |
| `UI-nnn --remedy AC-nnn,TOKEN-name,UI-nnn` | spec, remedy run | `UI-nnn` |
| any of the above plus `--resume` | same mode, resume run | as above |

**Spec mode is the default**: any argument string holding neither `--sketch` nor `--brand` is spec mode.

An id whose prefix is none of `IDEA`, `EPIC`, `STORY`, `UI` stops the run with one `Blocked` line naming the accepted prefixes.

This skill runs **no** preamble command. Spec mode allocates its ids at step 12, one per screen that carries no `UI-nnn`; sketch mode and brand mode allocate none. Allocation in a preamble runs before the mode is known, so a `--sketch` or `--brand` run would abort on an exhausted `UI` prefix it does not touch.

`designing-interfaces` is the one skill of the nine without `disable-model-invocation: true` in its frontmatter, because `exploring-ideas` step 6 and `planning-work` step 6 reach it through the Skill tool, and a skill the model cannot invoke cannot be reached that way.

### What each mode reads

| Input | Path | Mode |
|---|---|---|
| Sketch request | `.devforgeai/explore/sketch-request.json` | sketch |
| Explore brief | `.devforgeai/explore/brief.md` | all three |
| Explore seed data | `.devforgeai/explore/seed-data.json` | sketch, spec |
| Explore brand sketch | `.devforgeai/explore/mockups/brand-sketch.json` | brand |
| Requirements | `.devforgeai/requirements.yaml`, via `devforgeai doc load requirements -` | brand, spec |
| Story | `.devforgeai/stories/STORY-nnn.md`, via `devforgeai doc load story <STORY-nnn>` | spec |
| Frontend globs and token path | `.devforgeai/config.toml` `[frontend]` | brand, spec |
| Brand tokens | `.devforgeai/brand/tokens.json` | spec |
| Prior UI spec | `.devforgeai/ui-specs/UI-nnn.md`, via `devforgeai doc load ui-spec <UI-nnn>` | spec remedy |
| Send-back report from Plan | `.devforgeai/reports/SPRINT-nnn-plan.yaml` `findings[]` | spec remedy |
| Send-back report from Build | `.devforgeai/reports/STORY-nnn-build.yaml` `findings[]` | spec remedy |
| Prior design report | `.devforgeai/reports/UI-nnn-design.yaml` at `status: send_back`, most recent | spec resume |
| Phase state | `.devforgeai/state.toml` `[current].phase`, `[current].id` | all three |

Sketch mode reads no document produced by Discover, Constitute, or Plan: those phases have not run when Explore step 6 calls it. Spec mode reads `explore/brief.md` for the `## Core flows` table alone, and treats the file as absent when Discover ran from entry point B.

---

## Sketch mode — steps 2 to 4

Reached from Explore step 6 through the Skill tool with the request object rather than through `/design`, so that path carries no `$ARGUMENTS`. It reads the request from `.devforgeai/explore/sketch-request.json`, which Explore wrote before the invocation.

**2. Read the request.** Take `flows[]`, `out_dir`, `seed_data_path`, `brand`, and `constraints`. When the file is absent, return `screens: []`, `uncovered_flows` holding every `flow_id` the invocation passed, and one `notes` line naming the missing path.

**3. Draw the screens.** `mockup-designer` writes one HTML file per screen into `out_dir`, named `<FLOW-nnn-nn>.html`. It invokes the built-in `design` skill for layout and visual judgment. A flow whose steps produce no screen comes back in `uncovered_flows`, and the remaining flows keep their screens.

**4. Write the brand sketch.** `.devforgeai/explore/mockups/brand-sketch.json`, exactly three keys at the top level of one flat object:

```json
{
  "name": "Reconcile",
  "palette": ["#0F172A", "#2563EB", "#F8FAFC"],
  "type_pair": ["Inter", "IBM Plex Mono"]
}
```

3 to 6 hex values in `palette`, two family strings in `type_pair`. This file has no template and carries **no envelope**: everything under `.devforgeai/explore/mockups/` sits outside `.devforgeai/` document validation. A request carrying `brand: null` writes no file and leaves `brand_sketch` as `null`.

The mode returns `mode`, `idea_id`, `screens`, `brand_sketch`, `uncovered_flows`, `notes`. It writes no `brand/tokens.json` and no `ui-specs/UI-nnn.md`, and it emits no send-back — an uncovered flow travels back as `uncovered_flows`, which Explore turns into one `open_questions` line per id.

---

## Brand mode — steps 5 to 9

```
> /design --brand EPIC-001
```

**5. Gather the input.** Read `brand-sketch.json` when it exists, the `personas[]` array of `requirements.yaml`, and the `epics[]` entry named by `$1`. Hold a candidate name, a candidate palette, a candidate type pair, and the persona goals the brand answers to. With `brand-sketch.json` absent, the candidate name comes from `epics[].title` and the candidate palette and type pair are the defaults in `templates/tokens.json`.

**6. Ask the five questions.** The step 5 candidates are shown as one line of prose above the first call. Two `AskUserQuestion` calls, with the questions, headers, and option labels fixed in `templates/brand-questions.md`:

| Call | Headers |
|---|---|
| 1 | `Tone`, `Color`, `Type`, `Density` |
| 2 | `Logo` |

A free-text answer matching no label re-asks that one question once, then the run takes the first option of that question and adds one `open_questions` line to `brand-kit.md` naming the header.

**7. Write the tokens.** `brand-designer` invokes the built-in `frontend-design:frontend-design` skill for palette and type judgment and writes `.devforgeai/brand/tokens.json` with `meta.status: draft`.

Token shape, which `design lint` resolves against:

| Group | Leaves | Leaf type |
|---|---|---|
| `meta` | the seven envelope keys | — |
| `color` | 12 | an object with exactly `light` and `dark` |
| `type` | 12 | string |
| `spacing` | 6 | string |
| `radius` | 4 | string |
| `elevation` | 3 | string |
| `motion` | 4 | string |

Top-level keys in that order. Leaf names match `^[a-z][a-z0-9-]*$`. The flattened token name is `TOKEN-<group>-<leaf>` and the CSS custom property is `--<group>-<leaf>`. Brand mode writes no other leaves.

Contrast floors: every text leaf at 4.5 or above on `bg` in **both** themes, and the border leaf at 3.0 or above. A returned palette whose `text` on `bg` ratio falls below 4.5 in either theme comes back with the `text` value darkened or lightened until it clears, in the `adjustments[]` array; that adjustment is written into `brand-kit.md` `## Do not`.

`tokens.json` is the one JSON document in the framework that carries the seven envelope keys under a `meta` object rather than as siblings at the root, because the rest of the file is a token tree a design tool reads and a sibling key at the root would be read as a token group.

**8. Draw the logo.** `brand-designer` writes `.devforgeai/brand/logo.svg`. On `logo_written: false` with a `reason` string, write a one-line wordmark SVG from `templates/logo.svg` with the brand name and the `TOKEN-color-text` light value, and continue.

**9. Write the brand kit and mirror it.** `.devforgeai/brand/brand-kit.md` with all nine sections from `templates/brand-kit.md`, and `tokens.json` `meta.status` moved to `approved`.

When the Figma plugin's tools are present in the session and authenticated, the `figma:figma-use` and `figma:figma-generate-library` skills write the six groups as a Figma variable collection named `<brand name> tokens` with a light and a dark mode, and `## Figma` carries `Status: mirrored` and the collection name. When the plugin's tools are absent, or a call returns an authentication error, `## Figma` carries `Status: not mirrored` and `Reason` set to `figma plugin not present` or `figma plugin not authenticated`; every other output of this step is unchanged and the run continues.

---

## Spec mode — steps 10 to 14

```
> /design --spec STORY-001
```

**10. Load the subject.**

```
devforgeai doc load story STORY-001
devforgeai doc load requirements -
```

Hold the `AC-nnn` rows, the `REQ-nnn` records they cite, the `PERSONA-nnn` records those requirements name, and the `UI-nnn` ids the story already references. When `$1` is an epic, take the `epics[].requirements` list.

**With `.devforgeai/brand/tokens.json` absent, this mode writes no `UI-nnn.md`** and the handoff carries:

```
Blocked   you: run /design --brand <EPIC-nnn> first
```

Every colour and type value in a `UI-nnn.md` is a `TOKEN-<name>`, so a spec written before the brand kit would reference names nothing defines. Brand mode before spec mode is the one ordering this skill imposes on itself.

**11. Audit coverage.** `requirement-coverage-auditor` — this mode's registered verifier — takes the screens the story names, the `REQ-nnn` records of step 10, the `## Core flows` table of `explore/brief.md` when that file exists, and the `requirements[].source` values. It returns the envelope with `payload.subject_id`, `payload.screens[]`, `payload.screens_without_req[]`, `payload.flows_without_req[]`, and `payload.covered`.

The `SubagentStop` hook ingests that output and writes `.devforgeai/reports/UI-nnn-design.yaml` from it; **the report is the hook's to write, not this skill's.**

An absent `brief.md` is the shape Discover entry point B leaves: `payload.flows_without_req` comes back `[]` and the audit runs on screens alone.

A non-empty `payload.screens_without_req` or `payload.flows_without_req` is the send-back condition.

On a `--resume` run this step first reads the most recently modified `reports/UI-*-design.yaml` whose `status` is `send_back`, takes the `UI-nnn` and `FLOW-nnn` ids from its `findings[]`, and narrows itself and step 13 to the screen behind those ids. Every `UI-nnn.md` at `status: approved` keeps every byte.

**12. Allocate the ids.**

```
devforgeai doc validate --allocate UI
```

once per screen from step 11 that carries no existing `UI-nnn`, giving one `UI-nnn` per screen in `payload.screens[]` order. Exit 1 on `DFA-E215` means the prefix is exhausted; the run stops and the stderr line reaches the model.

**13. Write the UI specs.** One `ui-spec-writer` invocation per screen, in parallel across screens.

When the story carries a Figma node URL and the plugin's tools are present and authenticated, the `figma:figma-design-to-code` skill runs **in this conversation** first, once per node, and what it returns is held as `figma_context`. A story with no node URL, an absent plugin, or an authentication error leaves `figma_context` as `""`. A skill cannot be invoked from inside a subagent's turn, which is why the design context is produced here and passed in.

`ui-spec-writer` takes one screen with its allocated `UI-nnn`, the `REQ-nnn` records it realizes, the `PERSONA-nnn` goal, the flattened token names, the matching mockup files under `explore/mockups/` when they exist, the story's `figma_node_url` when it carries one, `figma_context`, and `templates/ui-spec.md` as its `template_path`. It invokes the built-in `frontend-design:frontend-design` skill for component composition judgment and writes one `.devforgeai/ui-specs/UI-nnn.md` per screen at `status: draft`, with all nine sections:

```markdown
---
schema: devforgeai/ui-spec/1
id: UI-004
phase: design
status: draft
produced_by: designing-interfaces
consumes: [REQ-001, PERSONA-001]
open_questions: []
---

# UI-004 Statement staging list

## Purpose
## Requirements
## Anatomy
## States
## Breakpoints
## Interaction
## Accessibility
## Tokens used
## Out of scope
```

Section names reproduced from `skills/designing-interfaces/templates/ui-spec.md`.

With `figma_context` empty, `## Anatomy` `Content source` cites the mockup path or the `REQ-nnn` in place of the node, and the file is otherwise unchanged.

The `## Accessibility` table holds eight rows, and they are fixed: `Landmark`, `Heading order`, `Name`, `Role`, `Keyboard path`, `Focus visible`, `Contrast`, `Motion`. Verify reads them.

The four `## Breakpoints` widths — `sm` 320px, `md` 768px, `lg` 1024px, `xl` 1440px — are fixed by the template and are **not** tokens.

**14. Resolve the tokens.**

```
devforgeai design lint --tokens
```

On exit 0, move every file written at step 13 to `status: approved`. Exit 1 on `DFA-E244` names a `TOKEN-<name>` in a `## Tokens used` row that `tokens.json` does not define:

```
<path>:<line> references token '<name>', which brand/tokens.json does not define
```

Rewrite that row with a defined token and the CLI reruns.

### The token rule, enforced at the moment of the write

The same `design lint` runs from the `PreToolUse` hook on every write to a path matching `[frontend].globs` minus `[frontend].exclude`:

```toml
[frontend]
globs = [
    "**/*.css", "**/*.scss", "**/*.sass", "**/*.less",
    "**/*.vue", "**/*.svelte", "**/*.jsx", "**/*.tsx",
    "**/*.html", "**/*.styles.ts", "**/*.styles.js",
]
tokens_path = ".devforgeai/brand/tokens.json"
exclude = [
    "**/node_modules/**", "**/dist/**", "**/build/**", "**/vendor/**",
    ".explore-prototype/**", ".devforgeai/explore/mockups/**",
]
```

Reproduced from `.devforgeai/config.toml`. The two `exclude` entries at the end are why a wireframe screen and a throwaway prototype may hold literal colours: they sit under the `design lint` exemption.

A literal colour in a linted file is `DFA-E240`:

```
<path>:<line> uses the literal colour '<value>'; brand/tokens.json defines <nearest-token>
```

and a literal type value is `DFA-E241`, with the same shape. A reference to an undefined token is `DFA-E242`. Each denies the write.

A hex triple, an `rgb(`, an `hsl(`, a CSS named colour, a `px` or `rem` font size, and a font family name do not appear in a `UI-nnn.md` body either.

---

## Step 15 — the remedy run

`/design UI-004 --remedy AC-007,TOKEN-color-accent` arrives from Plan or from Build. It runs step 15 and then step 14, and skips steps 10 to 13 except where a cited bare `UI-nnn` calls for a file step 13 writes.

Take the `UI-nnn` of `$1`, the ids after `--remedy`, the current file from `devforgeai doc load ui-spec UI-004`, and the `findings[]` of `reports/SPRINT-nnn-plan.yaml` or `reports/STORY-nnn-build.yaml`. Each cited id names its own rewrite:

| Cited id | What it names | Sections rewritten |
|---|---|---|
| `AC-nnn` | a criterion the spec does not support | `## States`, `## Interaction`, `## Accessibility` |
| `TOKEN-<name>` | a token the spec references and `tokens.json` does not define | `## Anatomy`, `## Tokens used` |
| `UI-nnn` | a spec a story references and `.devforgeai/ui-specs/` does not hold | the whole file, written from step 13 |

**Sections the cited ids do not name keep every byte.** The file goes to `status: draft`, then to `status: approved` after step 14. A cited `AC-nnn` absent from the story adds one `open_questions` line naming the id, and the ids that did resolve are rewritten.

`devforgeai doc load ui-spec UI-004` exiting 1 on `DFA-E200` means the spec is absent, and step 15 writes it from step 13.

---

## Step 16 — close the run

```
$ devforgeai phase set design --id UI-001
Phase     design · UI-001  (cross-cutting)
```

Reproduced, exit 0. The subject is the `UI-nnn` in spec mode, the `EPIC-nnn` in brand mode, the `IDEA-nnn` in sketch mode.

That call records `[last_cross]` in `state.toml` with `phase`, `id`, and the turn, and leaves `[current]` where it found it. **That is the whole of this step**: this skill runs no `devforgeai handoff` of its own.

The Stop hook then reads `[last_cross]`, renders Design's block and the block for the phase `[current].phase` still names, joins the two with a blank line, and clears `[last_cross]`:

```
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

Next      /explore IDEA-001
Blocked   none

Full report: .devforgeai/reports/IDEA-001-explore.yaml
```

Reproduced.

Three things it shows.

- **The em dash** in column one is what a phase outside the §5 numbered sequence renders.
- **`Done UI-001`.** Design owns no count of its own, so the `Done` line carries the subject.
- **Design's `Next` line names the row the current phase holds**, not a row of its own, because Design advances nothing. Here `[current].phase` is `explore`, so both blocks say `/explore IDEA-001`.

A second Stop in the same session prints one block: `[last_cross]` was cleared. The joined string is capped at 10,000 characters.

---

## Send-back

Design sends back to Discover only, from spec mode only, at step 11 only, **before any `UI-nnn.md` is written**.

| Condition | Detected by | Ids cited |
|---|---|---|
| A screen the story asks for realizes no requirement | `payload.screens_without_req` non-empty | the `UI-nnn` allocated for that screen at step 12 |
| A flow in `explore/brief.md` `## Core flows` equals no `requirements[].source` | `payload.flows_without_req` non-empty | the `FLOW-nnn` |

On either condition, write no `UI-nnn.md` for the cited screens and write the covered screens as step 13 defines. `SubagentStop` writes `.devforgeai/reports/UI-nnn-design.yaml` at `status: send_back` from the audit output. `state.toml` `[current]` is unchanged, because Design advances no phase.

```
Next      /discover IDEA-001 --remedy UI-004,FLOW-003
Then      /design STORY-001 --resume
```

The `<IDEA-nnn>` is the top-level `id` key of `requirements.yaml`, which step 10 already loaded — the document's own id, **not** the `EPIC-nnn` the story belongs to. An epic is a grouping inside that document, and Discover's remedy form re-opens the document by its own id. The send-back grammar is `/<upstream> <ID> --remedy <ID>,<ID>` and carries no `--from` flag; the `UI-` and `FLOW-` prefixes identify Design as the source.

Sketch mode emits no send-back: it runs inside Explore's Phase 0, where `requirements.yaml` does not exist. Brand mode emits none either: it reads `personas[]` and writes tokens, and a thin persona is a Discover-side edit that no token depends on.

Design **receives** a send-back from Plan and from Build as `/design UI-nnn --remedy <ids>`, handled at step 15. The returning command on the `Then` line is `/plan <SPRINT-nnn> --resume` or `/build <STORY-nnn> --resume`.

---

## `--resume`

Resume state lives in the documents and the report, not in `state.toml`, which Design leaves untouched.

| Form | Continues at |
|---|---|
| `--brand --resume` | the first step whose output is absent or whose `meta.status` is still `draft` |
| `--spec --resume` | the screens with no `UI-nnn.md` on disk; step 11 re-runs the coverage audit for the ids the most recent `send_back` report cited, and step 13 writes their files |

Every `UI-nnn.md` already at `status: approved` keeps every byte in both cases.

---

## Files this mode owns

| Document | Path | Written by | Template |
|---|---|---|---|
| Brand tokens | `.devforgeai/brand/tokens.json` | brand, step 7; `meta.status` to `approved` at step 9 | `templates/tokens.json` |
| Logo | `.devforgeai/brand/logo.svg` | brand, step 8 | `templates/logo.svg` |
| Brand kit | `.devforgeai/brand/brand-kit.md` | brand, step 9 | `templates/brand-kit.md` |
| UI spec | `.devforgeai/ui-specs/UI-nnn.md`, one per screen | spec, step 13, and step 15 on remedy | `templates/ui-spec.md` |
| Wireframe screen | `.devforgeai/explore/mockups/<FLOW-nnn-nn>.html` | sketch, step 3 | `templates/sketch-screen.html` |
| Brand sketch | `.devforgeai/explore/mockups/brand-sketch.json` | sketch, step 4 | none; `name`, `palette`, `type_pair` |

`phase` is `design` and `produced_by` is `designing-interfaces` in all three modes. `tokens.json` carries the envelope under `meta`; `brand-kit.md` and `UI-nnn.md` carry it as YAML frontmatter; the mockup HTML files carry none.

Status values:

- `tokens.json` `meta.status`: `draft` at step 7, `approved` at step 9.
- `brand-kit.md`: `approved`, the only value; the file is rewritten on the next brand run.
- `UI-nnn.md`: `draft` at step 13, `approved` at step 14. A remedy run moves the cited file back to `draft` and forward to `approved` again.
