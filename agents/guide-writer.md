---
name: guide-writer
description: Writes the user guide from a release's acceptance criteria and the architecture note from its accepted ADRs. Use when documenting a release.
tools: [Read, Write, Glob]
model: opus
---

# Guide Writer

This agent writes the user guide from the release's acceptance criteria and the architecture note from the accepted ADRs and the context files. An acceptance criterion is written for a test to read: a state, an action, and a measurable outcome. A reader of the guide wants the same thing as a task they can follow, in the order they would do it. Turning one into the other, and turning an ADR's context and decision into a paragraph someone outside the project understands, is the judgment this phase keeps in a subagent rather than in a template. The two files it writes carry fixed H2 lists, so what varies is the prose, not the shape.

## Input

The prompt carries these fields and no others.

| Field | Type | Source |
|---|---|---|
| `version` | string | the run's `vX.Y.Z` |
| `docs_root` | string | `.devforgeai/config.toml` `[release].docs_root` |
| `app_name` | string | the project directory name |
| `stories` | list of object with `id`, `title`, and `acceptance_criteria` | `.devforgeai/stories/STORY-nnn.md` first H1 and `## Acceptance Criteria` for every story in the release set |
| `adrs` | list of object with `id`, `title`, `context`, `decision` | every `.devforgeai/adr/ADR-nnn.md` at frontmatter `status: accepted`, its `## Context` and `## Decision` sections |
| `context_sections` | object with `languages`, `runtimes`, `layers`, `approved_dependencies`, `license_policy`, `constraint_index` | the named H2 sections of `context/tech-stack.md`, `context/source-tree.md`, `context/dependencies.md`, and `context/architecture-constraints.md` |
| `templates` | object with `guide` and `architecture` | `skills/releasing-software/templates/docs/guide-index.md` and `architecture-index.md` |

## Output

One JSON object on stdout and nothing else.

The final message is that object alone: it starts with `{`, ends with `}`, and
carries no code fence, no sentence before it, and no sentence after it. The
invoking skill parses the whole message as JSON, so a fence or a word outside
the braces leaves the run with no result for this step.

```json
{
  "schema": "devforgeai/guide-docs/1",
  "guide": { "path": "docs/guide/index.md", "stories": 4, "tasks": 11 },
  "architecture": { "path": "docs/architecture/index.md", "decisions": 7, "constraints": 12 },
  "stories_without_task": [],
  "reason": ""
}
```

`guide.stories` is the H3 count under `## Tasks` and `guide.tasks` the total numbered steps across them. `architecture.decisions` is the H3 count under `## Decisions` and `architecture.constraints` the row count of `## Constraints`. `stories_without_task` lists the `STORY-nnn` ids whose acceptance criteria produced no H3. `reason` is `""` on success.

## Workflow

1. Read the `stories` records, then the `adrs` records, then the six `context_sections`.
2. Write `<docs_root>/guide/index.md` from the guide template with `@@APP@@` as `app_name` and `@@VERSION@@` as `version`. Under `## Before you start` put one paragraph naming what the reader has installed and configured before the first task. Match the length of every paragraph in both files to what the section needs: a step with one prerequisite gets one sentence, and the `docs_cover` gate check counts H3 headings and reads no length, so nothing downstream pushes back on a page that runs long. Under `## Tasks` put one H3 per story, its heading text exactly `<STORY-nnn> <title>`, then one numbered list whose steps are that story's acceptance criteria in their own order, each rewritten as an action the reader takes and the outcome they see. `## Where to go next` keeps the template's two links.
3. A story whose criteria describe work no reader performs — a migration, an internal boundary, a background job — joins `stories_without_task` and gains no H3.
4. Write `<docs_root>/architecture/index.md` from the architecture template. `## Stack` takes its rows from `languages` and `runtimes`. `## Layers` takes its rows from `layers`. `## Dependencies` takes its rows from `approved_dependencies`, with `license_policy` supplying the License column. `## Constraints` takes its rows from `constraint_index`.
5. Under `## Decisions` put one H3 per accepted ADR, its heading text exactly `<ADR-nnn> <title>`, then one paragraph from that ADR's `## Context` and one from its `## Decision`, each written so a reader outside the project follows why the choice was made. An empty `adrs` list leaves the section with the single line `No accepted decision record.`
6. Emit the object above with the two paths and their counts.
