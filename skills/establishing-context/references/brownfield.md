# The brownfield branch

Read at workflow step 3 when `.devforgeai/context/tech-stack.md` exists with
`status: draft`. That one condition selects the whole branch: steps B4 through
B8 replace steps 4 through 10, and steps 1 through 3 and 11 through 13 run
unchanged.

The drafts come from `devforgeai init --analyze`, which walked the project root
before this session started. It read the code and the manifests, wrote three
files from what it found and three stubs, stamped every one
`produced_by: establishing-context` so the PreToolUse producer check admits the
completing edits, and left all six at `status: draft`. The `context audit` check
CA-2 exits 1 while any of the six is a draft, and `pre-commit` runs
`context audit`, so the drafts stay out of every commit until the acceptance at
step 12.

## B4 · Read the six drafts

Read all six and hold each file's values and its frontmatter `open_questions`
list. The stubs carry the body line `Drafted by the Constitute skill.` under
their empty headings and the entry `body drafted by the Constitute skill` in
`open_questions`; those are the places B7 fills.

The drafts do not carry the template heading lists. The CLI wrote the headings
that fit what it could read from code, and this phase rewrites them into the
template order, carrying each draft body under the heading that owns it. A
draft line matching no heading becomes an `open_questions` entry rather than a
seventh heading, because `context audit` CA-3 compares each file's H2 lines to
the fixed list with no extra H2 allowed.

| Draft file | Draft H2 | Template H2 it moves under |
|---|---|---|
| tech-stack.md | `## Languages` | `## Languages` |
| tech-stack.md | `## Runtime versions` | `## Runtimes` |
| tech-stack.md | `## Package managers` | `## Tooling`, key `framework.package` |
| tech-stack.md | `## Test tooling` | `## Tooling`, key `framework.test` |
| tech-stack.md | `## Lint tooling` | `## Tooling`, keys `framework.lint` and `framework.format` |
| tech-stack.md | `## Open` | frontmatter `open_questions`, one entry per bullet |
| source-tree.md | `## Tree` | `## Directory map` |
| source-tree.md | `## Layers` | `## Layers` |
| source-tree.md | `## Entry points` | `## File placement rules` |
| source-tree.md | `## Test roots` | `## Roots`, key `test.root` |
| dependencies.md | `## Direct dependencies` | `## Approved dependencies` |
| dependencies.md | `## Lockfile status` | `## Version policy` |
| dependencies.md | `## Unverified` | frontmatter `open_questions`, one entry per row |
| coding-standards.md | `## Detected configuration` | `## Formatting`, keys `style.indent`, `style.line.max`, `style.quote` |
| coding-standards.md | `## Naming` | `## Naming` |
| coding-standards.md | `## Formatting` | `## Formatting` |
| coding-standards.md | `## Error handling` | `## Error handling` |
| coding-standards.md | `## Testing` | `## Testing standards` |
| architecture-constraints.md | `## Layer map` | `## Layer dependency rules` |
| architecture-constraints.md | `## Module boundaries` | `## Layer dependency rules` |
| architecture-constraints.md | `## Constraints` | `## Constraints` |
| anti-patterns.md | `## Enabled lint rule sets` | `## Anti-patterns`, one `### AP-nnn` block per rule set a pattern observes |
| anti-patterns.md | `## Patterns` | `## Anti-patterns` |

The four template headings no draft supplies — `## Frameworks`, `## Data stores`
and `## Excluded technologies` in tech-stack.md, `## Naming conventions` and
`## Generated and excluded paths` in source-tree.md, `## Forbidden dependencies`,
`## License policy` and `## Addition procedure` in dependencies.md,
`## Logging`, `## Documentation` and `## Design tokens` in coding-standards.md,
`## Constraint index` and `## Anti-pattern index` — are written at B7 from B5,
B6, and the template.

## B5 · source-tree-mapper

One invocation, alone. Pass `source.root` from the draft `source-tree.md`, the
layer names the draft proposes, and the manifest paths from the draft
`dependencies.md`. It returns roots, layers with path globs and file counts,
entry points, internal edges, external dependencies, generated paths, and
unmapped paths.

Output that does not parse against the schema re-invokes the agent once with the
parse error appended to the prompt. A second parse failure continues the branch
with the draft values alone, and every field B5 would have supplied becomes an
`open_questions` entry answered at B6.

`internal_edges` is the evidence for `## Layer dependency rules`: an edge from
`interface` to `domain` that appears 40 times is a rule the code already
follows, and one that appears twice is a rule the code breaks twice. Both are
worth a row; the second is also worth an `AP-nnn`.

`unmapped_paths` are directories under `source.root` that fall under no layer
glob. Each becomes one B6 question, because a path with no layer has no
placement rule and no constraint reaches it.

## B6 · Answer the open questions

One `AskUserQuestion` per entry in the union of the six drafts'
`open_questions` lists and the entries B5 added. The options for each question
are the observations `source-tree-mapper` returned for that subject, plus
`leave open`.

`leave open` keeps the entry in `open_questions`, which `context audit` CA-2
reports at step 13 and which holds the gate at FAIL until a later run answers
it. That is the intended shape: an unanswered question about the existing code
is a fact about the project, and CA-2 makes it visible rather than letting the
draft ship as if it were decided.

## B7 · Complete the six files

Write all six from the template heading order, with the draft content under the
heading the table above names and the B5 and B6 answers filling the rest. Two
`Source` values distinguish provenance in every `| Key | Value | Source |` row:

| Row came from | `Source` cell |
|---|---|
| the draft `init --analyze` left | `init --analyze` |
| the B5 `source-tree-mapper` output | `source-tree-mapper` |
| a B6 answer that closed a choice | the `ADR-nnn` B8 writes for it |

The files stay at `status: draft` through B7. Step 12 is where the user moves
them to `accepted`.

Two drafts naming different values for one key is the case `context audit` CA-5
catches: a key holds one value across the whole set. The disagreement is a
decision, not a typo — the code carried both values — so B8 writes the ADR that
records which one the project keeps and why, and B7 writes the chosen value in
both files.

## B8 · ADRs for the decisions the code embodies

One ADR per decision already visible in the code: the layering the internal
edges show, the dependency choices the manifests hold, the naming the file
names follow, the version the pins fix.

These ADRs differ from the greenfield ones in two fields. `consumes` lists the
REQ ids the decision now serves, and is `[]` when the decision predates every
requirement — `context audit` CA-7 resolves each id in `consumes` against
`requirements.yaml`, and an empty list resolves trivially. `## Context` names
the evidence path from B5, which is what stands in for the forces a greenfield
ADR records: `src/infrastructure/` holds every database call and `src/domain/`
holds none is the force, stated as a path.

`status` is `proposed` until step 12. Then proceed to step 11, where
`architecture-reviewer` and `alignment-auditor` read the completed set the same
way they read a greenfield one.
