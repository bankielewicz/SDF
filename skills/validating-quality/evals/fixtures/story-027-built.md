---
schema: devforgeai/story/1
id: STORY-027
phase: plan
status: built
produced_by: planning-work
consumes: [REQ-005, CON-005]
open_questions: []
---

# Order retry backoff

## Story

As a buyer
I want order retry backoff
So that the order reaches the store once

## Requirements

| REQ | Statement | Covered by |
|---|---|---|
| REQ-005 | A failed order submission is retried within a budget. | AC-041 |

## Acceptance Criteria

- AC-041: Given a failed submission When the retry runs Then the wait doubles between attempts.

## Constraints

| CON | Statement | Binds |
|---|---|---|
| CON-005 | The retry budget is enforced at the boundary: the retry budget for an order submission is enforced at the boundary rather than inside the use case. | src/application/retry.ext |

## Anti-patterns

none

## Interface

none

## Layer

application

## Files

| Path | Kind | Layer |
|---|---|---|
| src/application/retry.ext | source | application |
| tests/retry_test.ext | test | application |

## Dependencies

none

## Out of scope

The boundary retry budget, which STORY-031 carries.
