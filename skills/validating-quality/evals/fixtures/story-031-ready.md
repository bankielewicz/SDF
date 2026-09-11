---
schema: devforgeai/story/1
id: STORY-031
phase: plan
status: ready
produced_by: planning-work
consumes: [REQ-005, CON-005]
open_questions: []
---

# Boundary retry budget

## Story

As a buyer
I want boundary retry budget
So that the order reaches the store once

## Requirements

| REQ | Statement | Covered by |
|---|---|---|
| REQ-005 | A failed order submission is retried within a budget. | AC-051 |

## Acceptance Criteria

- AC-051: Given a submission at the boundary When the budget is spent Then the request is refused.

## Constraints

| CON | Statement | Binds |
|---|---|---|
| CON-005 | The retry budget is enforced at the boundary: the retry budget for an order submission is enforced at the boundary rather than inside the use case. | src/api/orders.ext |

## Anti-patterns

none

## Interface

none

## Layer

interface

## Files

| Path | Kind | Layer |
|---|---|---|
| src/api/orders.ext | source | interface |
| tests/orders_test.ext | test | interface |

## Dependencies

none

## Out of scope

The backoff curve, which STORY-027 carries.
