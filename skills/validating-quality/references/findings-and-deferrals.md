# Findings, dispositions, and deferrals

Read before workflow step 8 and before step 10. This is the field-by-field shape of
the three payload sequences of `reports/STORY-nnn-qa.yaml`: `findings[]`,
`deferrals[]`, and the `blockers` list the gate reads.

## The finding band

Workflow step 5 runs `devforgeai doc validate --allocate FIND` once and takes the
returned id as `base`. The subagent at position `k` of the `agents.md`
`## Contracts` order, counting from zero, receives the band `FIND-(base + 100k)`
through `FIND-(base + 100k + 99)` and numbers its findings from the low end. The
band is 100 wide so that it fixes ids and bounds nothing else: every subagent
reports every item it has, at every severity, and no subagent truncates its list
or carries a `truncated` flag. Coverage first, and the dispositions of step 8 and
the gate do the filtering.

Ids are allocated once per run rather than once per finding because the subagents
are read-only and hold no Bash scope for the allocator, and because the number of
findings is unknown before they run. Gaps in the sequence cost nothing:
`doc validate --allocate` returns the next free id from the index, not the next
integer. On a resume run the allocator returns a fresh `base`, because the prior
run's ids are already in the index.

## `findings[]`

| Field | Type | Constraint |
|---|---|---|
| `id` | string | `^FIND-\d{3}$`, from the band rule above |
| `category` | string | the eleven-value enum below |
| `severity` | string | `blocker`, `high`, `medium`, `low` |
| `file` | string | a `Path` value of the story's `## Files` table, repo-relative with forward slashes, or `""` when the finding names no file |
| `line` | integer | 1-based; `0` when `file` is `""` or the finding names no line |
| `relates_to` | string | one id matching `^(AC\|CON\|AP\|ADR)-\d{3}$` |
| `disposition` | string | `fix`, `defer`, `accept` |
| `summary` | string | one line, 1 to 120 characters |
| `evidence` | string | one line, 1 to 200 characters, naming the path, the line, and what was read there |
| `found_by` | string | the subagent name that emitted it |

| `category` | Raised by | Cites |
|---|---|---|
| `standards` | `standards-reviewer` | a `## Formatting`, `## Naming`, `## Error handling`, `## Logging`, or `## Documentation` rule of `coding-standards.md`, through the `CON-nnn` that governs it |
| `anti-pattern` | `anti-pattern-scanner` | an `AP-nnn` |
| `constraint` | `constraint-auditor`, `adr-conformance-reviewer` | a `CON-nnn` or an `ADR-nnn` |
| `coverage` | `coverage-gap-auditor` | an `AC-nnn` |
| `dead-code` | `dead-code-detector` | a `CON-nnn` or `AP-nnn` |
| `deferral` | `deferral-validator` | a `CON-nnn` or `AP-nnn` |
| `ac-compliance` | `ac-compliance-verifier` | an `AC-nnn` |
| `spec-gap` | `ac-compliance-verifier`, `constraint-auditor` | an `AC-nnn`; the code meets the criterion and the criterion misses the requirement |
| `security` | `security-auditor` | a `CON-nnn` of `kind: security`, or the `AP-nnn` of `category: security` |
| `complexity` | `code-quality-auditor` | a `CON-nnn` or `AP-nnn` |
| `duplication` | `code-quality-auditor` | a `CON-nnn` or `AP-nnn` |

### Severity across the two files

The subagent envelope carries the three-value enum `block`, `warn`, `info` that
the `devforgeai/verifier/1` envelope fixes, as each
`.claude/agents/<name>.md` `## Output` states. The QA report carries the four-value form. The mapping is
one rule: `blocker` emits `block`, and `high`, `medium`, and `low` emit `warn`;
`info` comes from no subagent of this phase. A finding arriving as `block` is
recorded as `blocker`; a finding arriving as `warn` takes `high`, `medium`, or
`low` from what it costs — `high` when the behaviour a criterion names is affected,
`medium` when a rule of the context set is broken without a criterion moving, `low`
otherwise. An `AP-nnn` finding copies the `## Anti-pattern index` `Severity` cell
with no translation.

A `block` finding is also what reaches the gate: it lowers its subagent's `passed`
below its `total`, and `verifier_pass` runs at the compiled floor
`min_ratio = 1.0`.

## Disposition, at workflow step 8

Three values, one per finding:

| Value | When |
|---|---|
| `fix` | the remedy is code in this story; every finding of `severity: blocker` takes this value and no other, and every finding of `category: spec-gap` takes it and routes to Plan |
| `defer` | the remedy is one Definition-of-Done item another story carries; the finding also gets a `deferrals[]` entry |
| `accept` | the story's `## Out of scope` already excludes the behaviour the finding names |

A finding citing a `file` value absent from the story's `## Files` table is dropped
and its `evidence` line is recorded under the document's `open_questions`, because
the file set bounds every scan and a finding outside it names code this story does
not own.

## `deferrals[]`, at workflow step 9

A deferral is one Definition-of-Done item pushed to a later story. A
Definition-of-Done item is one of four things, and the `dod_item` string opens with
its id: an `AC-nnn` of the story's `## Acceptance Criteria`, the layer coverage
floor of its `## Layer`, a `CON-nnn` of its `## Constraints`, or an `AP-nnn` of its
`## Anti-patterns`.

| Field | Type | Constraint |
|---|---|---|
| `id` | string | `^FIND-\d{3}$`, equal to the `findings[]` entry whose `disposition` is `defer` |
| `story` | string | `^STORY-\d{3}$`, equal to the document `id` |
| `dod_item` | string | one line, 1 to 120 characters, opening with the `AC-nnn`, `CON-nnn`, or `AP-nnn` it names, or with the layer name for a coverage floor |
| `target` | string | `^(STORY\|ADR)-\d{3}$`, resolving in the ID index |
| `reason` | string | the five-value enum below |
| `opened_on` | string | `YYYY-MM-DD`, equal to `verified_on` |
| `con_or_ap` | string | `^(CON\|AP)-\d{3}$`, or `""` when the item is an `AC-nnn` or a coverage floor |

| `reason` | Means |
|---|---|
| `dependency_missing` | the item needs behaviour a `STORY-nnn` that is not at `status: released` carries |
| `scope_boundary` | the item names a behaviour this story's `## Out of scope` excludes and another story carries |
| `decision_pending` | an `ADR-nnn` that would decide the item is at `status: proposed` |
| `external_blocker` | a system or party outside the project controls the item |
| `tooling_absent` | `config.toml` names no command that measures the item, and `[verify].metrics_command` or `[verify].call_graph_command` is `""` |

`capacity` is absent from that list on purpose: `sprint.yaml` `deferred[].reason`
uses the word for a story left out of a sprint, which is a different fact on a
different document, and one word for both would make a Reflect aggregate that reads
both ambiguous.

The drafted entries go back through `deferral-validator` once more at step 9. An
entry whose `target` resolves to no document, whose `reason` the evidence does not
support, or whose `target` story already defers back to this story comes back as a
`block` finding in that subagent's second block, which fails `verifier_pass` and
routes the run to `## Send-back`.

## `blockers` and `summary`

`blockers` holds every `findings[].id` whose `severity` is `blocker`, in
`findings[]` order, and is a top-level list of strings. The gate reads it through
`length_between` at `min = 0`, `max = 0`, so a non-empty list is `DFA-E331` with
`on_fail = "send_back"`.

`summary` holds seven integers: `checks` (the length of `checks[]`, 7 in light mode
and 10 in deep), `findings`, `blockers` (the length of the top-level `blockers`
list), `high`, `medium`, `low`, and `deferrals`.

Both are derived from `findings[]` while writing the document, and no mechanical
check compares the derivation against the sequence — `doc validate` reads
frontmatter, headings, ids, and cross-references rather than arithmetic over a
payload. The backstop sits on the verifier side: a blocker finding reaches the
report only by way of a subagent whose `block` severity already lowered its
`passed`, so an id left out of `blockers` still fails `verifier_pass`.

## What the receiving phases read

Release reads four facts from this file: a finding's `id`, `severity`, and
`summary`, and the deferral marker, which is `disposition: defer` on the finding
plus the `deferrals[]` entry of the same `id` carrying its `reason`. Reflect reads
the top-level `deferrals[]` sequence for its technical-debt section, which is why
the sequence sits at the top level rather than inside a finding.
