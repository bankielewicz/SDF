---
schema: devforgeai/story/1
id: STORY-021
phase: plan
status: built
produced_by: planning-work
consumes: [REQ-002, CON-003]
open_questions: []
---

# STORY-021: Order history

## Story

As a buyer
I want to see what I bought before
So that I can find a past order without asking support

## Requirements

| REQ | Statement | Covered by |
|---|---|---|
| REQ-002 | A buyer reads the orders their address placed. | AC-006 AC-007 |

## Acceptance Criteria

- AC-006: Given three past orders When the buyer opens the history page Then three rows are rendered in descending date order.
- AC-007: Given no past order When the buyer opens the history page Then the empty region is rendered.

## Constraints

| CON | Statement | Binds |
|---|---|---|
| CON-003 | Boundary types: a store row leaves the infrastructure layer as a domain value. | interface |

## Anti-patterns

none

## Interface

none

## Layer

interface

## Files

| Path | Kind | Layer |
|---|---|---|
| src/interface/history.ext | source | interface |
| tests/interface/history_test.ext | test | interface |

## Dependencies

none

## Out of scope

This story carries no reporting surface over the data it writes.
