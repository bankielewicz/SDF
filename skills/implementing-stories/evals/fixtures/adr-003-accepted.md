---
schema: devforgeai/adr/1
id: ADR-003
phase: constitute
status: accepted
produced_by: establishing-context
consumes: [REQ-007]
open_questions: []
---

# ADR-003: One write path to the order store

## Context

Two call sites wrote the order store and disagreed about which transitions were legal.

## Decision

The application layer holds the only write path to the order store, and the store client is pinned to the 2.x line at runtime scope.

## Consequences

| Consequence | Direction | Affected |
|---|---|---|
| A new write extends the use case rather than the adapter | cost | REQ-007 |

## Constraints introduced

| CON | Kind | Statement |
|---|---|---|
| CON-001 | layering | The application layer holds the only write path to the order store. |

## Supersedes

| Supersedes ADR | Retires CON |
|---|---|
| none | none |
