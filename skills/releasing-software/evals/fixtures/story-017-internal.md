---
schema: devforgeai/story/1
id: STORY-017
phase: plan
status: built
produced_by: planning-work
consumes: [CON-003]
open_questions: []
---

# STORY-017: Ledger index migration

## Story

As a operator
I want the ledger read path to stay under its latency budget
So that the daily reconciliation finishes inside its window

## Requirements

| REQ | Statement | Covered by |
|---|---|---|
| - | No requirement is delivered by this story. | - |

## Acceptance Criteria

- AC-002: Given the ledger table When the migration has run Then the reconciliation query plan reads one index.

## Constraints

| CON | Statement | Binds |
|---|---|---|
| CON-003 | Boundary types: a store row leaves the infrastructure layer as a domain value. | infrastructure |

## Anti-patterns

none

## Interface

none

## Layer

infrastructure

## Files

| Path | Kind | Layer |
|---|---|---|
| src/infrastructure/ledger_index.ext | migration | infrastructure |
| tests/infrastructure/ledger_index_test.ext | test | infrastructure |

## Dependencies

none

## Out of scope

This story carries no reporting surface over the data it writes.
