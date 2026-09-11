# The story document

Read this before workflow step 8. It carries the frontmatter keys, the ten sections in order with their columns and row rules, and the three properties that make an acceptance criterion testable. `templates/story.md` is the shape; this file is the content rule for each part of it.

## Frontmatter

Seven keys, in this order, with no other top-level key.

| Key | Type | Required | Default | Constraint |
|---|---|---|---|---|
| `schema` | string | yes | none | constant `devforgeai/story/1` |
| `id` | string | yes | none | `^STORY-\d{3}$`, allocated by `devforgeai doc validate --allocate STORY` |
| `phase` | string | yes | none | constant `plan` |
| `status` | string | yes | `draft` | `draft`, `ready`, `building`, `built`, `released`, in progression order |
| `produced_by` | string | yes | none | constant `planning-work` |
| `consumes` | list of string | yes | none | each matches `^(REQ\|CON\|AP\|UI)-\d{3}$`; at least one `REQ-nnn`; every `REQ-nnn` belongs to the epic in `sprint.yaml` `epic` |
| `open_questions` | list of string | yes | `[]` | each one sentence, 1 to 200 characters |

Plan writes `draft` at step 8 and `ready` at step 12. `devforgeai phase set` writes `building`, `built`, and `released`; `ready` is absent from its transition table, which is why Plan holds it.

The H1 below the frontmatter reads `# STORY-nnn: <title>`, the title 1 to 60 characters.

## The ten sections

Headings appear in this order with this text. `devforgeai doc validate` compares the H2 list against `templates/story.md`, and `devforgeai story validate` locates content by the strings `## Requirements`, `## Acceptance Criteria`, `## Interface`, `## Files`, and `## Dependencies`.

| # | Heading | Content |
|---|---|---|
| 1 | `## Story` | Exactly three lines: `As a <personas[].name>`, `I want <one clause>`, `So that <one clause>`. |
| 2 | `## Requirements` | Table, columns `REQ \| Statement \| Covered by`. One row per `REQ-nnn` in `consumes`, 1 to 4 rows. `Statement` is the `requirements[].statement` value copied byte for byte. `Covered by` is a space-separated list of `AC-nnn` ids from section 3, 1 to 8 entries. |
| 3 | `## Acceptance Criteria` | Unnumbered list, 1 to 12 items, each `- AC-nnn: Given <state> When <action> Then <measurable outcome>.` Every `AC-nnn` appears in exactly one `Covered by` cell of section 2. |
| 4 | `## Constraints` | Table, columns `CON \| Statement \| Binds`. One row per `CON-nnn` in `consumes`, 0 to 8 rows. `Statement` is the `## Constraint index` row's `Title` and the `## Constraints` block's `statement` value joined by a colon. `Binds` names a `Path` value from section 8 or the `## Layer` value from section 7. |
| 5 | `## Anti-patterns` | Table, columns `AP \| Severity \| Scope`. One row per `AP-nnn` in `consumes` whose `Scope` glob matches a `Path` value in section 8, 0 to 8 rows. `Severity` is the `## Anti-pattern index` value, one of `blocker`, `high`, `medium`, `low`. |
| 6 | `## Interface` | Table, columns `UI \| Screen \| States covered`. One row per `UI-nnn` in `consumes`, 0 to 4 rows. `Screen` is the `UI-nnn` document's `## Purpose` first sentence. `States covered` is a space-separated list drawn from that document's `## States` `State` column. |
| 7 | `## Layer` | One line, a `Layer` value from `.devforgeai/context/source-tree.md` `## Layers` second table. |
| 8 | `## Files` | Table, columns `Path \| Kind \| Layer`. 1 to 20 rows. `Path` is repo-relative and matches a `Path pattern` glob from `## File placement rules`. `Kind` is one of `source`, `test`, `config`, `migration`, `asset`. `Layer` is a `Layer` value from `## Layers`, equal to section 7 for every row of `Kind` `source`. |
| 9 | `## Dependencies` | Unnumbered list, 0 to 6 items, each `- STORY-nnn: <one sentence naming what this story takes from it>`. |
| 10 | `## Out of scope` | Unnumbered lines, 0 to 8, each one sentence naming a behaviour this story does not carry and, when one exists, the `STORY-nnn` that does. |

## The `none` rule

Sections 4, 5, 6, and 9 carry the single line `none` in place of their table or list when they have no row: a story bound by no constraint, matching no anti-pattern, rendering no screen, or waiting on no other story. Section 10 with nothing to record carries no line at all. The other five sections carry at least one row.

## What makes an acceptance criterion testable

Three properties, each decided by `devforgeai story validate` or by `story-invest-auditor`:

1. The text matches `Given .+ When .+ Then .+`. The CLI reads it as `DFA-E230`.
2. The `Then` clause names one outcome a test reads — a value, a status, a count, a stored row, an emitted event, or a rendered region. `story-invest-auditor` judges this one and raises a `warn` finding cited by the story's own id when the clause names a feeling, an appearance, or a state with no reader.
3. The id appears in exactly one `Covered by` cell of section 2, so the criterion references at most one requirement. None is `DFA-E236`; more than one is `DFA-E245`.

A `Then` clause draws its outcome from the requirement's `acceptance_signal`, which is where Discover recorded the observable result. A name from `tech-stack.md` `## Excluded technologies` or from `dependencies.md` `## Forbidden dependencies` stays out of every criterion, because Build would then be asked to write against something the context set already excluded.

## What section 6 takes from a UI spec

Four sections of `.devforgeai/ui-specs/UI-nnn.md`, read through `devforgeai doc load ui-spec UI-nnn`:

- `## Requirements` fixes the `REQ-nnn` set the screen realizes, which is the set section 2 carries for an interface story.
- `## States` fixes the `States covered` cell of section 6 and the size the story carries: a screen with seven states is more work than a screen with two.
- `## Breakpoints` fixes the four layouts one `AC-nnn` covers, so the four are one criterion rather than four.
- `## Out of scope` fixes what section 10 records for that screen.
