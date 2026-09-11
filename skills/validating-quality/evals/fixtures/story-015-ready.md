---
schema: devforgeai/story/1
id: STORY-015
phase: plan
status: ready
produced_by: planning-work
consumes: [REQ-008, CON-003]
open_questions: []
---

# Payment capture

## Story

As a buyer
I want payment capture
So that the order reaches the store once

## Requirements

| REQ | Statement | Covered by |
|---|---|---|
| REQ-008 | A buyer pays for an accepted order. | AC-005 |

## Acceptance Criteria

- AC-005: Given an accepted order When capture is requested Then the capture is recorded.

## Constraints

| CON | Statement | Binds |
|---|---|---|
| CON-003 | One write path to the order store: the order store is written through one path in the infrastructure layer. | src/application/capture.ext |

## Anti-patterns

none

## Interface

none

## Layer

application

## Files

| Path | Kind | Layer |
|---|---|---|
| src/application/capture.ext | source | application |
| tests/capture_test.ext | test | application |

## Dependencies

- STORY-014: the accepted order state it captures against.

## Out of scope

The refund path, which no story carries yet.
