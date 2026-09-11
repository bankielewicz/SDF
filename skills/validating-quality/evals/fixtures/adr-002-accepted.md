---
schema: devforgeai/adr/1
id: ADR-002
phase: constitute
status: accepted
produced_by: establishing-context
consumes: [REQ-007]
open_questions: []
---

# ADR-002: The order store is reached through one adapter

## Context

Two call sites wrote the order store during the first cycle, and the two
disagreed about which state transitions were legal.

## Decision

The order store is reached through one adapter in the infrastructure layer, and
the decision about which transition is legal sits in the application layer.

## Consequences

A use case that needs a new write extends the adapter rather than opening the
store. A read path that needs a projection accepts one more adapter method.

## Constraints introduced

| CON | Statement |
|---|---|
| CON-003 | The order store is written through one path in the infrastructure layer. |

## Supersedes

| ADR | Retired |
|---|---|
| none | none |
