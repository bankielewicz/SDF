# The documentation layout

Read this before workflow step 10. It carries the four directories, the page-naming rule, the fixed H2 list per file, and what the gate's `docs_cover` check reads. `templates/docs/` holds the shape of each file.

Rooted at `config.toml` `[release].docs_root`, `docs` by default. Every path below is relative to the project root with the configured root substituted for `docs`.

```
docs/
├── README.md                       index: version, date, story count, links to the three sets, the logo
├── api/
│   ├── index.md                    table of every symbol, its kind, its stack, and the page it is on
│   └── <stack-id>-<root-slug>.md   one page per (stack, source root) pair; one H3 per symbol
├── guide/
│   └── index.md                    one H2 per story in the release, its ACs rewritten as steps
├── architecture/
│   └── index.md                    one H2 per accepted ADR, plus the stack, layer, and dependency tables
└── brand/
    ├── tokens.json                 byte copy of .devforgeai/brand/tokens.json
    └── logo.svg                    byte copy of .devforgeai/brand/logo.svg
```

## The API page names

`<stack-id>` is the `[[stack]].id` value from `.devforgeai/config.toml`. `<root-slug>` is that stack's `source_roots` entry with `/` and `\` replaced by `-`, lowercased, with leading and trailing `-` removed; an entry of `.` gives `root`. A stack with three source roots gives three pages.

The pair is what names a page rather than the root alone, because two `[[stack]]` tables can carry the same root string and one page for both would lose which symbol came from which.

| `[[stack]].id` | `source_roots` entry | Page |
|---|---|---|
| `alpha` | `src` | `docs/api/alpha-src.md` |
| `alpha` | `src/cli` | `docs/api/alpha-src-cli.md` |
| `beta` | `.` | `docs/api/beta-root.md` |

`docs.api[]` in the release file lists `index.md` first and then one entry per page, in the `[[stack]]` order and within a stack the `source_roots` order.

## The section lists

Headings are fixed. `docs_cover` locates content by these strings, so the shape is a contract rather than a suggestion.

| File | H2 sections, in order |
|---|---|
| `README.md` | `## What shipped`, `## Documentation`, `## Install`, `## Brand` |
| `api/index.md` | `## Symbols`, `## Pages` |
| `api/<page>.md` | `## Summary`, `## Symbols`, each symbol an H3 below it whose heading text is exactly the symbol name |
| `guide/index.md` | `## Before you start`, `## Tasks`, each story an H3 below it whose heading text is exactly `<STORY-nnn> <title>`, `## Where to go next` |
| `architecture/index.md` | `## Stack`, `## Layers`, `## Dependencies`, `## Decisions`, each accepted ADR an H3 with the text `<ADR-nnn> <title>`, `## Constraints` |

The guide is one file with one H3 per story rather than one file per story: a release is bounded by the stories Verify passed, so the file is bounded, and a reader following a version reads one page.

## Where each table's rows come from

| Table | Rows |
|---|---|
| `README.md` `## What shipped` | one per `stories[]` entry: the id, the title, the derived `notes.entries[].kind`, and the `REQ-nnn` list |
| `README.md` `## Install` | one paragraph naming what `config.toml` `[release].package_command` produces and where it lands |
| `api/index.md` `## Symbols` | one per line of `[release].api_symbols_command`, whose columns are the symbol, its kind, its stack, and the page holding its H3 |
| `api/index.md` `## Pages` | one link per page file |
| `architecture/index.md` `## Stack` | the `## Languages`, `## Runtimes`, `## Frameworks`, and `## Data stores` entries of `context/tech-stack.md` |
| `architecture/index.md` `## Layers` | the `## Layers` rows of `context/source-tree.md` |
| `architecture/index.md` `## Dependencies` | the `## Approved dependencies` rows of `context/dependencies.md`, with the `## License policy` supplying the License column |
| `architecture/index.md` `## Decisions` | one H3 per `adr/ADR-nnn.md` at `status: accepted`, a paragraph from its `## Context` and a paragraph from its `## Decision` |
| `architecture/index.md` `## Constraints` | the `## Constraint index` rows of `context/architecture-constraints.md` |
| `guide/index.md` `## Tasks` | one H3 per story, then one numbered list whose steps are that story's acceptance criteria in their own order |

`devforgeai doc load adr all` returning `DFA-E200` means no ADR exists; `## Decisions` is then written with the single line `No accepted decision record.` and the section stays in place.

## What `docs_cover` reads

The check runs `config.toml` `[release].api_symbols_command`, takes the symbol name from each line, and looks for an H3 whose heading text equals that name across the files listed in `docs.api[]` other than the index. It passes at a ratio of `min_ratio`, `1.0` in the shipped gate. A symbol with no H3 is counted against the ratio and named by `DFA-E344`.

An `api_symbols_command` of `""` makes the check record `SKIP` with `reason: no_api_symbols_command`. `api/index.md` is written with an empty `## Symbols` table, no page file is written, and `docs.api_symbols` and `docs.api_documented` are both `0`.

The ratio is recomputed from the files on disk rather than read from `api-doc-writer`'s own `symbols_documented`, so the agent's count reports and decides nothing.

## The brand copies

`.devforgeai/brand/tokens.json` and `.devforgeai/brand/logo.svg` are copied byte for byte to `<docs_root>/brand/` and listed in `docs.brand[]` and in `artifacts[]` at `kind: brand`. `README.md` `## Brand` references the copied logo by its published path, `brand/logo.svg`. A project where `.devforgeai/brand/` holds neither file leaves `docs.brand` as `[]`.
