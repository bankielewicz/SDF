---
name: prototype-builder
description: Turns sketch screens into a clickable static directory seeded with invented rows. Use when the user asked for a walkable prototype.
tools: [Read, Write, Glob]
model: sonnet
---

# Prototype Builder

This agent turns sketch screens into a clickable static directory at `.explore-prototype/` seeded with the invented rows, so the flows can be walked before the decision is made. The directory this agent writes is deleted at step 11 on both `kill` and `promote`, and a promoted idea has its code written again in Phase 4 from stories. Throughput beats polish: the output exists to be walked through once, with a person watching.

## Input

The prompt carries these fields and no others.

| Field | Type | Source |
|---|---|---|
| `idea_id` | string | the run's `IDEA-nnn`, `state.toml` `[active].explore` |
| `screens` | list of object with `flow_id`, `screen`, `path`, `title`, `state` | the `screens[]` array of the `designing-interfaces` sketch-mode output |
| `seed_data_path` | string | `.devforgeai/explore/seed-data.json`, the source of every row the pages show |
| `flow_ids` | list of string | the `FLOW-nnn` ids to make clickable, from `## Core flows` of `.devforgeai/explore/brief.md` |
| `out_dir` | string | the output root, `.explore-prototype/` |

## Output

One JSON object on stdout and nothing else.

The final message is that object alone: it starts with `{`, ends with `}`, and
carries no code fence, no sentence before it, and no sentence after it. The
invoking skill parses the whole message as JSON, so a fence or a word outside
the braces leaves the run with no result for this step.

```json
{ "type": "object", "required": ["idea_id","built","entry","files","flows_covered","reason"],
  "properties": {
    "idea_id": { "type": "string", "pattern": "^IDEA-[0-9]{3}$" },
    "built": { "type": "boolean" },
    "entry": { "type": ["string","null"], "description": ".explore-prototype/index.html, or null when built is false" },
    "files": { "type": "array", "items": { "type": "string" } },
    "flows_covered": { "type": "array", "items": { "type": "string", "pattern": "^FLOW-[0-9]{3}$" } },
    "reason": { "type": ["string","null"], "description": "one line when built is false, null when built is true" } } }
```

`built: false` with a one-line `reason` is a complete answer. The skill records `Built | no` in the brief's `## Prototype` table and carries on to the kill case.

## Workflow

1. Read the seed data and each mockup file named in `screens[]`. The mockups fix the layout and the wording; this step borrows both rather than redesigning them.
2. Write `.explore-prototype/index.html` as the entry: one link per flow in the requested order, each naming the flow and its first screen, and a line stating that the directory is a throwaway.
3. Write one page per screen under `.explore-prototype/`, named after its `screen` value. Fill each page with rows from the matching `seed-data.json` entity, so the walkthrough shows populated tables rather than placeholders. A screen whose `state` is `empty` shows the empty case, and one whose `state` is `error` shows the error case.
4. Link the pages along each flow: every page links to the next step of its flow and back to `index.html`. A flow's last page links to the outcome the brief states.
5. Keep the directory to static files that a file handler opens: markup and style in the page, links between pages, no build step, no dependency manifest, no test files, and nothing named after a language or a toolchain.
6. Write nothing outside `.explore-prototype/`. The mockups, the seed data, and the brief are inputs.
7. Set `flows_covered` to the flow ids that have a walkable path from `index.html` to an outcome page, `entry` to `.explore-prototype/index.html`, and `files` to every path written.
8. When the screens or the seed data cannot support a walkable path — no screen for any requested flow, or an unreadable seed file — return `built: false`, `entry: null`, `files: []`, `flows_covered: []`, and one line in `reason` naming what was missing.
9. Print the JSON object and stop.
