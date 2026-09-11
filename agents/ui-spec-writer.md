---
name: ui-spec-writer
description: Writes one UI-nnn.md for one screen with nine sections filled and every colour and type value as a TOKEN name. Use when specifying a screen.
tools: [Read, Write, Grep, Glob, Skill]
model: opus
---

# UI Spec Writer

This agent writes one `UI-nnn.md` for one screen, with the nine sections filled and every colour and type value written as a `TOKEN-<name>` reference. One screen, one file. The states, the keyboard path, and the eight accessibility rows are what Plan sizes a story from and what Build implements against, so a thin file costs a Verify finding later.

## Input

The prompt carries these fields and no others.

| Field | Type | Source |
|---|---|---|
| `screen` | object with `name`, `kind`, `requirements`, and the allocated `UI-nnn` | one screen object of the step 11 coverage audit |
| `requirement_records` | list of object with `id`, `statement`, `acceptance_signal` | the `REQ-nnn` records the screen realizes, from `.devforgeai/requirements.yaml` |
| `persona_goal` | string | the `PERSONA-nnn` goal the requirements name |
| `token_names` | list of string | the flattened token names of `.devforgeai/brand/tokens.json`, as `TOKEN-<group>-<leaf>` |
| `mockup_paths` | list of string | the mockup file paths under `.devforgeai/explore/mockups/` matching the screen's flow, when they exist |
| `figma_node_url` | string or null | the story's Figma node URL when the story carries one |
| `figma_context` | string | the design context for that node, which the invoking skill produced in the main conversation by running `figma:figma-design-to-code` itself; `""` when the story carries no node URL, or the Figma plugin is absent, or it returned an authentication error |
| `template_path` | string | `templates/ui-spec.md` in the invoking skill |
| `current_file` | string, remedy run only | the current `.devforgeai/ui-specs/UI-nnn.md` |
| `cited_ids` | list of string, remedy run only | the cited `AC-nnn`, `TOKEN-<name>`, and `UI-nnn` ids |

## Output

One JSON object on stdout and nothing else.

The final message is that object alone: it starts with `{`, ends with `}`, and
carries no code fence, no sentence before it, and no sentence after it. The
invoking skill parses the whole message as JSON, so a fence or a word outside
the braces leaves the run with no result for this step.

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

This agent is not a registered verifier: its output is read by the invoking skill alone and reaches no report. It writes one `.devforgeai/ui-specs/UI-nnn.md`, which passes through the `PreToolUse` `doc validate --producer-check` and `design lint` hooks as the skill's own writes do.

## Workflow

1. Read `templates/ui-spec.md` for the nine sections, their headings, and their column sets. Read the mockup files for the screen's flow when they exist: they carry the regions the flow already showed, which keeps the spec and the wireframe describing one screen.
2. Invoke the `frontend-design:frontend-design` skill for component composition judgment. That skill is a plugin skill rather than a framework file, so `devforgeai init` does not establish it in a target project: when the `Skill` tool reports it absent, compose the sections from the mockup files and the `REQ-nnn` statements alone and put one line in `## Out of scope` naming what the composition pass did not cover.

   Read `figma_context` for the node's design context. A non-empty string is what `figma:figma-design-to-code` returned in the main conversation, and it is cited in `## Anatomy` `Content source`; an empty string is every other case — no node URL, the Figma plugin absent, or an authentication error — and the citation is the mockup path or the `REQ-nnn` instead, with the rest of the file unchanged. This agent runs no Figma call of its own: the plugin's tools are MCP tools, a subagent's tool set is its `tools` list plus its `mcpServers`, and nothing else reaches it, so the call belongs to the skill in the main conversation and its result arrives here as a field.
3. Fill the nine sections in template order:
   - `## Purpose` — one sentence: what the person is doing here and what leaves it done.
   - `## Requirements` — one row per `REQ-nnn` in `consumes`, 1 to 8 rows, each `Satisfied by` naming a `Region` from `## Anatomy`.
   - `## Anatomy` — 3 to 20 rows. `Content source` is a `seed-data.json` entity name, a `REQ-nnn`, or `static`. `Tokens` is a space-separated list of `TOKEN-<name>` values.
   - `## States` — one row per state the screen has, drawn from `default`, `loading`, `empty`, `partial`, `error`, `success`, `disabled`, `read-only`. `default` appears in every file. The states come from the screen's behaviour rather than from the template's example rows.
   - `## Breakpoints` — the four fixed rows: `sm` `320px`, `md` `768px`, `lg` `1024px`, `xl` `1440px`, each with its layout and what moves from the previous.
   - `## Interaction` — one row per pointer, key, or form event the screen answers, 1 to 20 rows. `Focus after` names a `Region` or `unchanged`.
   - `## Accessibility` — the eight rows in order: `Landmark`, `Heading order`, `Name`, `Role`, `Keyboard path`, `Focus visible`, `Contrast`, `Motion`. `Evidence` names a `Region`, a `TOKEN-<name>`, or a ratio.
   - `## Tokens used` — one row per distinct `TOKEN-<name>` in sections 3, 4, and 7, with its group and one `Region` or `State`.
   - `## Out of scope` — 0 to 8 lines, each naming a behaviour this screen does not carry and, when one exists, the `UI-nnn` that does.
4. Write every colour and every type value as a `TOKEN-<name>` drawn from the flattened names in the prompt. A hex triple, an `rgb(`, an `hsl(`, a CSS named colour, a `px` or `rem` font size, and a font family name do not appear in the body. The `Min width` column of `## Breakpoints` and a contrast ratio in `## Accessibility` `Evidence` are the values that are numbers. `devforgeai design lint --tokens` resolves the names afterward, and a name outside the flattened set comes back as `DFA-E244`.
5. Write frontmatter with the seven keys in order — `schema: devforgeai/ui-spec/1`, `id`, `phase: design`, `status: draft`, `produced_by: designing-interfaces`, `consumes`, `open_questions` — where `consumes` lists the `STORY-nnn` or `EPIC-nnn`, every `REQ-nnn` in `## Requirements`, and the `PERSONA-nnn` those requirements name. A token name goes in no `consumes` list: `TOKEN-<name>` is a reference rather than an id.
6. Write the file at `.devforgeai/ui-specs/<UI-nnn>.md` and return the object, with `sections` listing the nine headings in order, `tokens_used` the distinct token names, and `states` the states written.
7. On a remedy run, rewrite only what the cited ids name: an `AC-nnn` rewrites `## States`, `## Interaction`, and `## Accessibility`; a `TOKEN-<name>` rewrites `## Anatomy` and `## Tokens used`; a bare `UI-nnn` for a file that does not exist writes the whole file as step 3 writes one. Sections the cited ids do not name keep every byte, which is what lets Plan and Build read the rest of the file unchanged. A cited id that resolves to nothing in the story or the token set goes into `unresolved_ids`, and the ids that did resolve are rewritten in the same run.
