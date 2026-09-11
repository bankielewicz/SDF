# Remedy runs and resumes

Read at step 1 when `$ARGUMENTS` holds `--remedy`, or when a brief is already on disk.

## Where a remedy run comes from

Discover emits a send-back when a `FLOW-nnn` row contradicts itself, names no actor, or duplicates another row. Its handoff prints the command the user types:

```
Next      /explore IDEA-001 --remedy FLOW-002,FLOW-004
Then      /discover IDEA-001 --resume
```

The cited ids arrive in `$ARGUMENTS` and again in the `findings[]` of `.devforgeai/reports/IDEA-001-discover.yaml`, which step 1 reads and step 4 passes to `flow-drafter`.

## Fresh run beside remedy run

| Aspect | Fresh run | Remedy run |
|---|---|---|
| ID | allocated at step 1 by `doc validate --allocate IDEA` | taken from `$ARGUMENTS`; step 1 allocates nothing |
| Step 1 extra read | none | `.devforgeai/reports/IDEA-001-discover.yaml`, and `.devforgeai/explore/brief.md` |
| `phase set` arguments | `explore --id IDEA-001` | `explore --id IDEA-001 --remedy FLOW-002,FLOW-004` |
| `state.toml` writes | `[current].phase`, `[current].id`, `[active].explore`, `[explore].idea_id`, `[explore].started_at`, `[explore].timebox_days`, `[explore].remedy_flows = []` | `[explore].remedy_started_at`, `[explore].remedy_timebox_days`, `[explore].remedy_flows = ["FLOW-002","FLOW-004"]`; `[explore].started_at` and `[current]` unchanged |
| Time box | `explore.timebox_days` from `explore.started_at` | `explore.remedy_timebox_days` from `explore.remedy_started_at` |
| Step 2 brainstorm | runs | skipped |
| Step 3 scan | runs | skipped |
| Step 4 drafting | all flows, non-goals, success signal, seed data | the cited `FLOW-nnn` rows only; non-goals, success signal and seed data returned unchanged |
| Step 5 brief writes | all twelve sections | the cited rows of `## Core flows`, frontmatter `open_questions`, and `status` |
| Step 6 mockups | every flow in `flows` | the cited ids only; the other rows of `## Mockups` are left as they are |
| Step 7 prototype | offered | skipped; `.explore-prototype/` is left as it is |
| Step 8 kill case | runs | runs |
| Step 9 question | asked | asked, with the same three options |
| Step 10 decision | `remedied_flows: []` | `remedied_flows: ["FLOW-002","FLOW-004"]` |
| Handoff `Next` | `/discover IDEA-001` | `/discover IDEA-001 --resume` |

A remedy run rewrites named rows inside a one-day box, which is why the prototype is skipped: rebuilding a directory that step 11 deletes would spend that box on work the decision does not use.

## A cited id that is absent

`flow-drafter` returns it in `unresolved_flow_ids` rather than inventing a row. Write one line per id into the brief's frontmatter `open_questions`, in this form:

```
FLOW-009 cited by the Discover send-back is absent from Core flows
```

Rewrite the ids that did resolve. The gate's remedy check reports the absent id on the next Stop, so the handoff carries it back to the user.

## Resume runs

A resume is a brief on disk whose `status` is below `decided`, with no `--remedy` in `$ARGUMENTS`. Read the brief and continue at the step after its `status`:

| `status` | Continue at |
|---|---|
| `drafting` | step 3, the scan |
| `scanned` | step 4, the one-page spec |
| `specified` | step 6, the mockups |
| `mocked` | step 8, the kill case |

The ids already in the brief stay as they are, and `[explore].started_at` keeps the original time box running: a resume continues the same box rather than opening a new one.
