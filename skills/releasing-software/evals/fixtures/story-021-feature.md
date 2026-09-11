---
schema: devforgeai/story/1
id: STORY-021
phase: plan
status: built
produced_by: planning-work
consumes: [REQ-007, REQ-011, CON-003]
open_questions: []
---

# STORY-021: Saved addresses

## Story

As a buyer
I want to reuse an address I entered before
So that I do not retype it on every purchase

## Requirements

| REQ | Statement | Covered by |
|---|---|---|
| REQ-007 | A buyer completes a purchase without an account. | AC-003 |
| REQ-011 | A completed purchase produces one order record. | AC-004 |

## Acceptance Criteria

- AC-003: Given one saved address When the buyer opens checkout Then that address is preselected.
- AC-004: Given a checkout completed with a new address When the order is written Then the address is stored against the order.

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
| src/interface/addresses.ext | source | interface |
| tests/interface/addresses_test.ext | test | interface |

## Dependencies

none

## Out of scope

This story carries no reporting surface over the data it writes.
