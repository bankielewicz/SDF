# The throwaway prototype

Read at step 7, the one optional step of the phase.

## The offer

Ask the question in `templates/prototype-offer.md`, with `IDEA-nnn` and the flow count filled in. `Yes` runs `prototype-builder`; `No` skips it. A remedy run skips the offer entirely and leaves `.explore-prototype/` as it stands.

## What `prototype-builder` receives

- `screens[]` from the sketch-mode output, with the paths under `.devforgeai/explore/mockups/`
- `.devforgeai/explore/seed-data.json`, the source of every row the pages display
- the `FLOW-nnn` ids to make clickable, which are the ids in `## Core flows` that have at least one screen
- the output root, `.explore-prototype/`

## What comes back

```json
{ "idea_id": "IDEA-001", "built": true, "entry": ".explore-prototype/index.html",
  "files": [".explore-prototype/index.html", ".explore-prototype/FLOW-001-01.html"],
  "flows_covered": ["FLOW-001", "FLOW-002"], "reason": null }
```

`built: false` carries a one-line `reason` and a `null` entry. Either way the run continues to step 8; the difference shows up only in the table.

## The `## Prototype` table

Four rows, in this order:

| Field | Built, from the agent | Skipped or `built: false` |
|---|---|---|
| `Built` | `yes` | `no` |
| `Path` | `.explore-prototype/` | `-` |
| `Entry` | `.explore-prototype/index.html` | `-` |
| `Flows covered` | `flows_covered` joined with `, ` | `-` |

## What the directory holds

Static files at the project root, sibling to `.devforgeai/`, opened by the operating system's file handler: an `index.html` entry that links to one page per screen, pages that link onward along each flow, and the seed rows rendered inline. No build step, no dependency manifest, no test files, no context files, and nothing named after a language or a toolchain.

The directory sits outside `.devforgeai/`, so the PreToolUse producer check does not apply to its files, and `.explore-prototype/**` is one of the two paths `devforgeai design lint` skips: a wireframe drawn before a brand kit exists has no token to resolve against.

## What happens to it

Step 11 runs `devforgeai explore prune --id IDEA-nnn`. The CLI deletes the directory on `kill` and on `promote`, leaves it on `park`, and exits 0 when it is already gone. Nothing under `.explore-prototype/` is carried into a later phase; a promoted idea has its code written again in Phase 4, from stories. `devforgeai init` has already added `.explore-prototype/` to the target project's `.gitignore`, since a directory the CLI deletes without asking does not belong in version control.
