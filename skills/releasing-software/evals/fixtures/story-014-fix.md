---
schema: devforgeai/story/1
id: STORY-014
phase: plan
status: built
produced_by: planning-work
consumes: [REQ-002, FIND-004, CON-003]
open_questions: []
---

# STORY-014: Checkout rounding repair

## Story

As a buyer
I want the total I am charged to equal the total I was shown
So that I am not billed a cent more than the basket said

## Requirements

| REQ | Statement | Covered by |
|---|---|---|
| REQ-002 | A buyer reads the orders their address placed. | AC-001 |

## Acceptance Criteria

- AC-001: Given a basket whose line totals carry a half cent When the checkout total is computed Then the charged amount equals the displayed amount.

## Constraints

| CON | Statement | Binds |
|---|---|---|
| CON-003 | Boundary types: a store row leaves the infrastructure layer as a domain value. | application |

## Anti-patterns

none

## Interface

none

## Layer

application

## Files

| Path | Kind | Layer |
|---|---|---|
| src/application/checkout.ext | source | application |
| tests/application/checkout_test.ext | test | application |

## Dependencies

none

## Out of scope

This story carries no reporting surface over the data it writes.
