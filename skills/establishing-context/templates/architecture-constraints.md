---
schema: devforgeai/context-architecture-constraints/1
id: architecture-constraints
phase: constitute
status: draft
produced_by: establishing-context
consumes: []
open_questions: []
---

# Architecture constraints

## Constraints

### CON-nnn <title>

| Field | Value |
|---|---|
| kind | <boundary \| dependency \| layering \| non-goal \| performance \| security \| data \| process> |
| status | <active \| retired> |
| statement | <one declarative present-tense sentence> |
| source | <REQ-nnn \| explore/brief.md ## Non-goals \| init --analyze> |
| introduced_by | <ADR-nnn or `none`> |
| enforced_by | <AP-nnn \| devforgeai context audit \| devforgeai design lint \| devforgeai doc validate \| none> |

## Layer dependency rules

| Layer | Depends on | Does not depend on | Constraint |
|---|---|---|---|
| <name> | <comma-separated layer names or `nothing`> | <comma-separated layer names> | CON-nnn |

## Constraint index

| CON | Kind | Status | Title | Source | Introduced by | Enforced by |
|---|---|---|---|---|---|---|
| CON-nnn | <kind> | <status> | <title> | <source> | <ADR-nnn or `none`> | <enforced_by> |
