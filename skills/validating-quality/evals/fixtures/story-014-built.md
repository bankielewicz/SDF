---
schema: devforgeai/story/1
id: STORY-014
phase: plan
status: built
produced_by: planning-work
consumes: [REQ-007, CON-003]
open_questions: []
---

# Order checkout

## Story

As a buyer
I want order checkout
So that the order reaches the store once

## Requirements

| REQ | Statement | Covered by |
|---|---|---|
| REQ-007 | A buyer submits an order and the store records it once. | AC-001 AC-002 AC-003 AC-004 |

## Acceptance Criteria

- AC-001: Given a submitted order When the total is positive Then the order is accepted.
- AC-002: Given a submitted order When the total is zero Then the order is rejected.
- AC-003: Given an accepted order When it is read back Then its state reads accepted.
- AC-004: Given a rejected order When it is read back Then its state reads rejected.

## Constraints

| CON | Statement | Binds |
|---|---|---|
| CON-003 | One write path to the order store: the order store is written through one path in the infrastructure layer. | src/domain/order.ext |

## Anti-patterns

none

## Interface

none

## Layer

domain

## Files

| Path | Kind | Layer |
|---|---|---|
| src/domain/order.ext | source | domain |
| tests/order_test.ext | test | domain |

## Dependencies

none

## Out of scope

The payment capture, which STORY-015 carries.
