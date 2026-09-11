---
schema: devforgeai/adr/1
id: ADR-002
phase: constitute
status: accepted
produced_by: establishing-context
consumes: [REQ-007]
open_questions: []
---

# ADR-002: One formatting and naming rule set

## Context

Three files in the first cycle disagreed about indent width, quote style and
how a constant was spelled, and each review spent time on the disagreement
rather than on the change.

## Decision

`coding-standards.md` carries one value per formatting and naming key — indent,
line length, quote style, and the four naming cases — and every file follows it.

## Consequences

| Consequence | Direction | Affected |
|---|---|---|
| A file that predates the rule is reformatted when it is next touched | cost | REQ-007 |

## Constraints introduced

| CON | Kind | Statement |
|---|---|---|
| none | none | This decision introduces no constraint. |

## Supersedes

| Supersedes ADR | Retires CON |
|---|---|
| none | none |
