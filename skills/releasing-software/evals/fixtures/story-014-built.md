---
schema: devforgeai/story/1
id: STORY-014
phase: plan
status: built
produced_by: planning-work
consumes: [REQ-007, REQ-011, CON-003]
open_questions: []
---

# STORY-014: Order checkout

## Story

As a buyer
I want to pay for the basket I assembled
So that I receive what I chose without opening an account

## Requirements

| REQ | Statement | Covered by |
|---|---|---|
| REQ-007 | A buyer completes a purchase without an account. | AC-001 AC-002 |
| REQ-011 | A completed purchase produces one order record. | AC-003 |

## Acceptance Criteria

- AC-001: Given a basket holding one item When the buyer submits the checkout form Then the response carries a confirmation number.
- AC-002: Given a basket holding no item When the buyer opens checkout Then the response status is 400.
- AC-003: Given a confirmed checkout When the order store is read Then one order row carries the confirmation number.

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
