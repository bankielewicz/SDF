# The shape of `requirements.yaml`

Read at step 12, when the persona, epic and requirement records go to disk. `templates/requirements.yaml` carries the skeleton; this file carries the types and the limits.

## Top-level keys, in this order

| Key | Type | Default | Constraint |
|---|---|---|---|
| `schema` | string | none | constant `devforgeai/requirements/1` |
| `id` | string | none | `^IDEA-\d{3}$` |
| `phase` | string | none | constant `discover` |
| `status` | string | `drafting` | enum below |
| `produced_by` | string | none | constant `discovering-requirements` |
| `consumes` | list of string | `[]` | each matches `^(IDEA\|FLOW)-\d{3}$`; `[]` at entry point B |
| `open_questions` | list of string | `[]` | each matches `^REQ-\d{3}$`; holds every `REQ-nnn` whose own `open_questions` is non-empty |
| `revision` | integer | `1` | `>= 1`; raised by exactly 1 per re-open pass |
| `revision_log` | list of object | `[]` | entry shape below |
| `accepted_by` | string or null | `null` | enum `user`, or null |
| `accepted_at` | string or null | `null` | RFC 3339 UTC, `^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$`, or null |
| `personas` | list of object | `[]` | shape below; length `>= 1` |
| `epics` | list of object | `[]` | shape below; length `>= 1` |
| `requirements` | list of object | `[]` | shape below; length `>= 1` |

The top-level `open_questions` list is derived rather than authored: step 12 writes it as the ascending-id list of every `REQ-nnn` carrying a non-empty `open_questions` of its own. An unknown that survives round 3 lands in the owning requirement's list and reaches the top level through that derivation.

`status` runs `drafting` while the run is drafting, `awaiting_acceptance` at step 12, `accepted` after `doc accept` at step 14, and `reopened` after `doc reopen` at C2. `accepted_by` and `accepted_at` are written by the binary, not by editing, so the timestamp comes from one place.

## `revision_log[]`

| Field | Type | Default | Constraint |
|---|---|---|---|
| `revision` | integer | none | the value of `revision` after the raise |
| `at` | string | none | RFC 3339 UTC |
| `from` | string | none | enum `plan`, `constitute`, `design` |
| `reopened` | list of string | `[]` | each `^REQ-\d{3}$`, moved to `status: reopened` by this pass |
| `added` | list of string | `[]` | each `^REQ-\d{3}$`, allocated by this pass |

## `personas[]`

| Field | Type | Constraint |
|---|---|---|
| `id` | string | `^PERSONA-\d{3}$`, unique in the file |
| `name` | string | 1 to 40 characters, a role label |
| `description` | string | one sentence, 1 to 200 characters |
| `goal` | string | one sentence, 1 to 200 characters |

## `epics[]`

| Field | Type | Constraint |
|---|---|---|
| `id` | string | `^EPIC-\d{3}$`, unique in the file |
| `title` | string | 1 to 60 characters |
| `scope` | string | one sentence, 1 to 200 characters |
| `out_of_scope` | list of string | length `>= 1`, each one sentence, 1 to 200 characters |
| `success_metric` | string | one sentence carrying one measurable quantity, its unit or count, and a comparison; at least one digit |
| `requirements` | list of string | each `^REQ-\d{3}$` resolving to a `requirements[].id`, with no duplicate across all epics |

`success_metric` in the shape the field takes: `Unmatched lines fall below 5 per night.` — the quantity `5`, the unit `lines per night`, the comparison `fall below`. `Every clerk closes the night.` carries no quantity, `Unmatched lines drop` carries a comparison with nothing to compare, and `Reconciliation improves` carries neither; each one leaves the field failing the shape rule and `doc validate` reporting it on the write. The sentence is drafted once, at round 2, and travels to this field word for word.

The epic count equals the count of outcomes the user selected at round 2, and the nth epic's `success_metric` is the nth selected outcome. That is what keeps `success_metric` from having no producer when the user selects fewer outcomes than the grouping would otherwise have produced epics. Each exclusion selected at round 3 lands on exactly one epic; an epic drawing none takes the single entry `Nothing was named out of scope at discovery.`

## `requirements[]`

| Field | Type | Default | Constraint |
|---|---|---|---|
| `id` | string | none | `^REQ-\d{3}$`, unique in the file |
| `actor` | string | none | `^PERSONA-\d{3}$`, resolving to a `personas[].id` |
| `statement` | string | none | one sentence, 1 to 200 characters |
| `rationale` | string | none | one sentence, 1 to 200 characters |
| `acceptance_signal` | string | none | one sentence naming one observable result, 1 to 200 characters |
| `priority` | string | none | enum `must`, `should`, `could`, `wont` |
| `source` | string | none | `user`, or a `^FLOW-\d{3}$` present in `consumes` |
| `status` | string | `draft` | enum `draft`, `accepted`, `reopened`, `withdrawn` |
| `open_questions` | list of string | `[]` | each one sentence, 1 to 200 characters |
| `traces_to` | list of string | `[]` | each `^UI-\d{3}$`; filled by a re-open from Design |

The four `priority` ranks are the four MoSCoW ranks. They are enum values in a YAML scalar, not instruction prose, so the conventions §2 ceremony rule does not reach them. `source` holds `user` or a flow id and nothing else: a requirement created from a Design re-open records `source: user` and carries its interface reference in `traces_to`, so nothing downstream that reads `source` as a flow reference learns a third form.

## Ids live for the life of the project

An id that once appears in `personas[]`, `epics[]` or `requirements[]` stays in the file. A requirement the user drops moves to `status: withdrawn`, keeps its `id`, its `statement` and every other field as written, and stays in the epic that already groups it. The gate's `length_between` and `set_cover` checks carry `exclude_status = ["withdrawn"]`, so a fully withdrawn set does not pass the grouping checks on paper.

## What the gate reads

`gate check --phase discover` evaluates five checks against this document: every requirement carries `id`, `actor`, `statement`, `rationale`, `acceptance_signal`, `priority`, `source` and `status` with a non-empty value; every `requirements[].actor` equals some `personas[].id`; every `epics[].requirements` list holds at least one live requirement; the union of those lists equals the live `requirements[].id` set with no duplicate; and `accepted_by` holds a value. The `accepted` check tests presence, not authorship.
