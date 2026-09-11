# Constraints and anti-patterns

Read at workflow steps 7 and 9, which write `architecture-constraints.md` and
`anti-patterns.md`.

A CON states a bound the project holds itself to. An AP states a pattern that
observes a CON being broken, in text a tool can match. The pairing is what turns
a sentence in a document into something Build's hooks and Verify's
`anti-pattern-scanner` act on: a CON with no AP is a rule nobody sees violated,
which is what `alignment-auditor` reports as `unobservable`.

## Step 7 · where a CON comes from

Three sources, each with its own `source` cell. IDs come from
`devforgeai doc validate --allocate CON`, one call per constraint.

| Source | `kind` | `source` cell | What supplies the statement |
|---|---|---|---|
| a `requirements[]` record whose `statement` or `acceptance_signal` names a bound rather than a behavior | the bound's kind: `performance`, `security`, `data`, `boundary`, `layering`, `dependency`, or `process` | that `REQ-nnn` | the `acceptance_signal`, whose number goes into the CON statement |
| an unnumbered line under the brief's `## Non-goals` | `non-goal` | `explore/brief.md ## Non-goals` | the line itself, in the present tense |
| an `epics[].out_of_scope` entry | `non-goal` | that `EPIC-nnn` | the entry itself |

The brief's non-goals carry no ID of their own — that section of
`.devforgeai/explore/brief.md` is a list of unnumbered lines — so the `source`
cell names the path and the heading instead. An `epics[].out_of_scope` list is required and holds at least
one entry, so a project that skipped Explore still has a non-goal set.

Distinguishing a bound from a behavior: `A signed-in shopper completes checkout
on one page` is a behavior, and Plan turns it into a story. `Every catalog query
returns within 50 milliseconds at p99` is a bound, and this phase turns it into
a `performance` CON whose statement carries the 50 and the p99. A record can
yield both; the behavior stays in `requirements.yaml` for Plan to read.

A `process` CON names its actor from `personas[]`: the `name` and `goal` fields
say who the process binds.

## The `### CON-nnn` block

Six fields, each from a closed set where the spec closes one.

| Field | Values |
|---|---|
| `kind` | `boundary`, `dependency`, `layering`, `non-goal`, `performance`, `security`, `data`, `process` |
| `status` | `active`, `retired` |
| `statement` | one declarative present-tense sentence |
| `source` | a `REQ-nnn`, `explore/brief.md ## Non-goals`, `init --analyze`, or an `EPIC-nnn` |
| `introduced_by` | the `ADR-nnn` that introduced it, or `none` |
| `enforced_by` | an `AP-nnn`, `devforgeai context audit`, `devforgeai design lint`, `devforgeai doc validate`, or `none` |

Every block also gets a row in `## Constraint index`, and the index row is where
`status` lives for the audit and for every downstream reader. A constraint whose
index row says `active` resolves in one of two ways: it appears in some ADR's
`## Constraints introduced` table, or its `source` cell holds a `REQ-nnn` that
`requirements.yaml` defines. The step 10 ADRs list the step 7 CON ids for
exactly this reason, so a non-goal CON drawn from the brief — whose `source` is
a path, not a REQ — reaches an ADR table.

`## Layer dependency rules` is one row per layer: what it depends on, what it
does not depend on, and the CON that says so. `nothing` is the value for a layer
with no dependencies. Build's layering check reads this table together with the
`## Constraint index`.

## Step 9 · where an AP comes from

One `### AP-nnn` block per CON whose statement a text or path pattern observes.
IDs come from `devforgeai doc validate --allocate AP`. The AP's `source` is the
CON id, and the CON's `enforced_by` field and index cell are set to the AP id in
the same pass — the two point at each other, and `alignment-auditor` reads the
pair.

| Field | Values |
|---|---|
| `category` | `library`, `structure`, `layer`, `smell`, `security`, `style` |
| `severity` | `blocker`, `high`, `medium`, `low` |
| `scope` | the glob the detector runs over |
| `detector_kind` | `literal`, `regex`, `glob` |
| `detector` | a literal string, a regular expression, or a glob |
| `remediation` | one declarative present-tense sentence |
| `source` | the `CON-nnn` this observes |

Which CONs yield an AP: a `dependency` CON forbidding a library yields a
`literal` detector on the import text; a `layering` CON yields a `glob` detector
on the importing path plus a `regex` on the imported name; a `non-goal` CON
whose excluded capability has a name in code yields a `literal` on that name; a
`security` CON naming a forbidden call yields a `regex`. A `performance` CON
yields none — a latency bound is measured, not matched — and keeps
`enforced_by: none`.

`severity` is the vocabulary `blocker | high | medium | low` across every
template and every subagent schema in this phase.

A CON that no pattern observes keeps `enforced_by: none` and appears in the
`alignment-auditor` findings at step 11 with `kind: unobservable`. That finding
is information for the user at step 12, which is the point: the phase records
which rules have teeth and which are prose.

`## Anti-pattern index` carries one row per block with the detector and its kind,
because `anti-pattern-scanner` in Verify reads the index rather than the blocks.
An index row whose `source` CON is `retired` is what `alignment-auditor` reports
as `orphaned`; the remedy path at R4a repoints or removes those rows.
