---
schema: devforgeai/story/1
id: STORY-017
phase: plan
status: built
produced_by: planning-work
consumes: [REQ-009, CON-003]
open_questions: []
---

# STORY-017: Refund webhook

## Story

As a finance clerk
I want refunds to reach the ledger without a manual entry
So that the daily reconciliation matches without my typing it

## Requirements

| REQ | Statement | Covered by |
|---|---|---|
| REQ-009 | A refund raised by the payment provider reaches the ledger. | AC-004 AC-005 |

## Acceptance Criteria

- AC-004: Given a signed refund callback When the endpoint receives it Then one ledger row carries the refund amount.
- AC-005: Given a callback whose signature does not match When the endpoint receives it Then the response status is 401.

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
| src/interface/refund_hook.ext | source | interface |
| tests/interface/refund_hook_test.ext | test | interface |

## Dependencies

none

## Out of scope

This story carries no reporting surface over the data it writes.
