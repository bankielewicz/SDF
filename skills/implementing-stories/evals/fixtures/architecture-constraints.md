---
schema: devforgeai/context-architecture-constraints/1
id: architecture-constraints
phase: constitute
status: accepted
produced_by: establishing-context
consumes: [IDEA-004]
open_questions: []
---

# Architecture constraints

## Constraints

### CON-001 Single write path

| Field | Value |
|---|---|
| kind | layering |
| status | active |
| statement | The application layer holds the only write path to the order store. |
| enforced_by | AP-001 |
| source | ADR-003 |

## Layer dependency rules

| Layer | May call | Constraint |
|---|---|---|
| domain | nothing | CON-001 |
| application | domain | CON-001 |
| infrastructure | domain | CON-001 |
| interface | application | CON-001 |

## Constraint index

| CON | Title | Kind | Status | Enforced by | Source |
|---|---|---|---|---|---|
| CON-001 | Single write path | layering | active | AP-001 | ADR-003 |
