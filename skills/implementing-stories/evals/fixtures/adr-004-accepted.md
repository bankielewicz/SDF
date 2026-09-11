---
schema: devforgeai/adr/1
id: ADR-004
phase: constitute
status: accepted
produced_by: establishing-context
consumes: [REQ-007]
open_questions: []
---

# ADR-004: Design tokens are the only source of visual values

## Context

Two screens carried different hex values for the same surface, and neither named where the value came from.

## Decision

Every visual value is read from `.devforgeai/brand/tokens.json`; a literal colour, radius or spacing in a stylesheet is an anti-pattern.

## Consequences

| Consequence | Direction | Affected |
|---|---|---|
| A new visual value is added to the token file first | cost | REQ-007 |

## Constraints introduced

| CON | Kind | Statement |
|---|---|---|
| none | none | This decision introduces no constraint. |

## Supersedes

| Supersedes ADR | Retires CON |
|---|---|
| none | none |
