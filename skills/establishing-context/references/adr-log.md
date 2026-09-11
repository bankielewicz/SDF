# The ADR log

Read at workflow step 10, at B8 on the brownfield branch, and at R4a on the
remedy path.

`.devforgeai/adr/` is append-only. A decision that stops holding is not edited
out; a later ADR supersedes it, and both stay on disk. That is what makes the
directory a log rather than a document: Reflect reads it as `OBS-nnn` evidence,
the commit-msg hook resolves `ADR-nnn` tokens against it, and a reader a year
later can see which choice was made when and what replaced it.

## ADR-000, the reason the project exists

Written first at step 10, from `.devforgeai/explore/decision.yaml`, and only
when that file exists. It is the one ADR whose `consumes` holds an `IDEA-nnn`
rather than `REQ-nnn` ids, and the one allocated by reservation rather than by
`devforgeai doc validate --allocate ADR`.

| Section | Content |
|---|---|
| frontmatter `consumes` | `[IDEA-nnn]`, the id from `decision.yaml` |
| `## Context` | the `reason` field of `decision.yaml`, quoted |
| `## Decision` | `decision: promote`, with the `decided_on` date |
| `## Consequences` | one row per `carry_forward` entry whose `consumer` is `establishing-context` |
| `## Constraints introduced` | the header row alone |
| `## Supersedes` | `none` and `none` |

Two `carry_forward` entries name this phase: the brief's `## Non-goals`, which
becomes constraint and anti-pattern entries, and the brief's `## Competitor scan`
and `## Technology scan`, which become stack candidates and ADR context. Those
two are the `## Consequences` rows.

A `decision.yaml` absent from disk skips ADR-000 and step 10 continues with the
remaining decisions. A `decision.yaml` carrying `decision: kill` or
`decision: park` stops the workflow at step 3: Explore promoted nothing, so
there is nothing to constitute.

## ADR-nnn, one per closed choice

After ADR-000, one ADR per decision that closed a choice among alternatives —
which data store, which layering, which dependency, which naming, which version
pin. A choice with one candidate is a detected fact and belongs in a
`| Key | Value | Source |` row with `Source` = `config.toml`, not in an ADR.

One ADR in the set records the scope boundary: what this project does and what
it leaves to something else. Its `## Constraints introduced` table carries the
`non-goal` CON ids, whose `source` cells hold `explore/brief.md ## Non-goals` or
an `EPIC-nnn` rather than a `REQ-nnn`. Without it those constraints resolve
through neither half of CA-4, and a project whose only non-goals came from
`epics[].out_of_scope` would audit dirty on its first run.

| Field | Content |
|---|---|
| `id` | from `devforgeai doc validate --allocate ADR` |
| `status` | `proposed` until step 12, then `accepted` |
| `consumes` | the motivating REQ ids, in `priority` order |
| `## Context` | the forces, present tense |
| `## Decision` | one sentence naming the chosen option, then the options rejected |
| `## Consequences` | one row per gain and per cost, each naming a file path, a layer name, or a `REQ-nnn` |
| `## Constraints introduced` | the CON ids step 7 allocated for this decision |
| `## Supersedes` | `none` and `none` on a fresh ADR |

`consumes` is the only place the motivating REQ ids appear; the body carries no
duplicate list. `context audit` CA-7 resolves every id in `consumes` against
`requirements.yaml` and every CON in `## Constraints introduced` against the
`## Constraint index`, so an ADR that names an unallocated CON fails the audit
rather than sitting unnoticed.

`## Consequences` rows carry `gain` or `cost` in the `Direction` cell. Recording
the cost is what lets Plan read the ADR and know what a story built under it
pays.

## Supersession

The shape of every change to a decision, on the remedy path at R4a and on a
second cycle alike.

1. The new ADR is written with `status: proposed`, its `## Supersedes` table
   naming the superseded `ADR-nnn` and the `CON-nnn` that row retires, and its
   `## Constraints introduced` listing the replacement CON from
   `devforgeai doc validate --allocate CON`.
2. The superseded ADR's frontmatter moves to `status: superseded`. Its body
   stays as written.
3. The retired CON's `## Constraint index` row moves to `status: retired`. Every
   other CON row stays as it stands.
4. An AP whose `source` is the retired CON has its `source` repointed to the
   replacement CON, or its block and index row removed when the replacement CON
   carries no pattern.

`context audit` CA-8 reads step 1 against steps 2 and 3: an ADR named in a
`## Supersedes` row carries `status: superseded`, and a CON that row retires
carries `status: retired`. The two halves stay in step, which is what keeps the
log readable after a dozen supersessions.

A second cycle over the same project — `/constitute EPIC-008` after the first
run — amends the six files through new ADRs and new CON rows by this same path.
The six files are project singletons and are not rewritten from scratch.

## Brownfield ADRs

Written at B8, one per decision the existing code already embodies.
`## Context` names the evidence path from `source-tree-mapper` rather than the
forces a greenfield decision weighed, and `consumes` is `[]` when no requirement
motivated the decision — CA-7 resolves an empty list trivially. Everything else
is the same shape.
