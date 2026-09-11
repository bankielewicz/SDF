---
schema: devforgeai/story/1
id: STORY-022
phase: plan
status: built
produced_by: planning-work
consumes: [REQ-011, CON-011]
open_questions: []
---

# Order read limit

## Story

As a buyer
I want order read limit
So that the order reaches the store once

## Requirements

| REQ | Statement | Covered by |
|---|---|---|
| REQ-011 | A buyer reads the orders that belong to them, at most fifty at a time. | AC-021 |

## Acceptance Criteria

- AC-021: Given a read request When the limit is applied Then at most fifty orders come back.

## Constraints

| CON | Statement | Binds |
|---|---|---|
| CON-011 | An order read compares the requested id against the session subject: an order is read only when the requested id belongs to the session subject. | src/domain/limit.ext |

## Anti-patterns

none

## Interface

none

## Layer

domain

## Files

| Path | Kind | Layer |
|---|---|---|
| src/domain/limit.ext | source | domain |
| tests/limit_test.ext | test | domain |

## Dependencies

none

## Out of scope

The pagination cursor, which no story carries yet.
