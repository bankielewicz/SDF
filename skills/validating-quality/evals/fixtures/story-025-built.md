---
schema: devforgeai/story/1
id: STORY-025
phase: plan
status: built
produced_by: planning-work
consumes: [REQ-005, CON-005]
open_questions: []
---

# Order retry

## Story

As a buyer
I want order retry
So that the order reaches the store once

## Requirements

| REQ | Statement | Covered by |
|---|---|---|
| REQ-005 | A failed order submission is retried within a budget. | AC-031 AC-032 |

## Acceptance Criteria

- AC-031: Given a failed submission When the retry runs Then the order is submitted once more.
- AC-032: Given a retried submission When it fails again Then the failure is returned.

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
