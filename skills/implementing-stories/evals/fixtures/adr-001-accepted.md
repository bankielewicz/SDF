---
schema: devforgeai/adr/1
id: ADR-001
phase: constitute
status: accepted
produced_by: establishing-context
consumes: [REQ-007]
open_questions: []
---

# ADR-001: Source and tests sit in two roots

## Context

The first cycle mixed tests beside the code they exercised, and a coverage run could not tell the two apart.

## Decision

Source lives under `src` and tests under `tests`, mirroring each other path for path. `source-tree.md` carries the two roots and the naming rules that follow from them.

## Consequences

| Consequence | Direction | Affected |
|---|---|---|
| A new layer appears in both roots at once | cost | REQ-007 |

## Constraints introduced

| CON | Kind | Statement |
|---|---|---|
| none | none | This decision introduces no constraint. |

## Supersedes

| Supersedes ADR | Retires CON |
|---|---|
| none | none |
