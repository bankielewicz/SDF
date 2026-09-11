# The twelve brief sections and the seed data

Read at step 5, while writing `.devforgeai/explore/brief.md` from `templates/brief.md` and `.devforgeai/explore/seed-data.json` from `templates/seed-data.json`, filled from the `seed_data` field `flow-drafter` returned.

## Frontmatter

```yaml
---
schema: devforgeai/explore-brief/1
id: IDEA-001
phase: explore
status: specified
produced_by: exploring-ideas
consumes: []
open_questions: []
---
```

`status` moves through `drafting` (step 2c done), `scanned` (step 3 done), `specified` (steps 4 and 5 done), `mocked` (step 6 done), `decided` (step 10 done). Step 5 writes `specified`. `consumes` stays `[]` on every run. Open questions live here and in no body section.

## Sections, in this order, with these headings

| # | Heading | Content |
|---|---|---|
| 1 | `## Problem statement` | One sentence. Who loses what, and how often. |
| 2 | `## Target user` | One line naming the segment, then 2 to 4 lines, one attribute per line, each an observable fact. |
| 3 | `## What they do today` | Table, columns `Current approach`, `Cost`, `Where it breaks`. One row per approach, 1 to 4 rows. |
| 4 | `## Why now` | 1 to 3 lines. Each line is a dated change in the world, formatted `YYYY-MM — <change>`. |
| 5 | `## Competitor scan` | Table, columns `Name`, `URL`, `Approach`, `Price`, `Gap`. 0 to 8 rows. `Gap` is the part of the problem statement that competitor leaves unsolved. |
| 6 | `## Technology scan` | Table, columns `Capability`, `Candidate`, `Maturity`, `License`, `Source`. 0 to 8 rows. `Maturity` is one of `established`, `emerging`, `experimental`. `Source` is a URL. |
| 7 | `## Core flows` | Table, columns `ID`, `Actor`, `Trigger`, `Steps`, `Outcome`. 3 to 5 rows. `ID` is `FLOW-nnn`, zero-padded, allocated in row order starting at `FLOW-001`. `Steps` is one line, steps joined with ` -> `. |
| 8 | `## Non-goals` | Unnumbered lines, one per line, 1 to 10 lines. Each states a capability this idea excludes, in the present tense. |
| 9 | `## Success signal` | One line, format `<metric> \| <threshold> \| <observation window>`. Exactly one line. |
| 10 | `## Mockups` | Table, columns `Flow`, `Screen`, `Path`, `State`. One row per screen returned by the sketch-mode contract. Empty table with the header row when step 6 is skipped. |
| 11 | `## Seed data` | Table, columns `Entity`, `Rows`, `Field count`. One row per entity in `.devforgeai/explore/seed-data.json`. |
| 12 | `## Prototype` | Table, columns `Field`, `Value`, with exactly four rows: `Built` (`yes` or `no`), `Path` (`.explore-prototype/` or `-`), `Entry` (`.explore-prototype/index.html` or `-`), `Flows covered` (comma-separated `FLOW-nnn` list or `-`). |

Sections 1 to 4 come from `idea-interrogator`: `problem_statement` fills section 1, `holders[0].segment` and `holders[0].attributes` fill section 2, `today[]` fills section 3 one row per entry, and `why_now[]` fills section 4 as `<month> — <change>`.

Sections 5 and 6 come from `landscape-scanner`: `competitors[]` and `technologies[]`, one row each, in the order returned. Zero rows is a valid scan; the header row stays.

Sections 7, 8, 9 and 11 come from `flow-drafter`: `flows[]`, `non_goals[]`, `success_signal` written as `<metric> | <threshold> | <window>`, and one `## Seed data` row per `seed_data.entities[]` entry with `Rows` the length of `rows` and `Field count` the length of `fields`.

Sections 10 and 12 are filled at steps 6 and 7.

## `.devforgeai/explore/seed-data.json`

`templates/seed-data.json` is the file to fill. The seven envelope keys come first at the top level, then the payload, in one flat object with no wrapper, as the template carries them:

```json
{
  "schema": "devforgeai/explore-seed-data/1",
  "id": "IDEA-001",
  "phase": "explore",
  "status": "recorded",
  "produced_by": "exploring-ideas",
  "consumes": [],
  "open_questions": [],
  "entities": [
    { "name": "invoice", "fields": ["id", "customer", "amount", "due_on", "status"],
      "rows": [ { "id": "INV-1001", "customer": "Marsh Dental", "amount": 480.00, "due_on": "2026-09-30", "status": "sent" } ] }
  ]
}
```

`status` has one value, `recorded`. Each entity carries 5 to 20 rows, every value invented. The file lives outside `.explore-prototype/`, so deleting the prototype leaves it standing, and it survives `kill`, `park`, and `promote` alike.

## When step 2c returned no holders

An empty `holders` array means the idea has no one to sell to. Write the returned `open_questions` into the frontmatter, leave `## Core flows` with its header row only, and go to step 8. The gate's flow-count check reports the empty table on the next Stop.
