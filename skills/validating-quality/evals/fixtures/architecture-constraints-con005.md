---
schema: devforgeai/context-architecture-constraints/1
id: architecture-constraints
phase: constitute
status: accepted
produced_by: establishing-context
consumes: [IDEA-002]
open_questions: []
---

# Architecture constraints

## Constraints

### CON-003 One write path to the order store

| Field | Value |
|---|---|
| kind | boundary |
| status | active |
| statement | The order store is written through one path in the infrastructure layer. |
| source | REQ-007 |
| introduced_by | ADR-002 |
| enforced_by | AP-002 |

### CON-009 Failures carry the error type

| Field | Value |
|---|---|
| kind | process |
| status | active |
| statement | A failure path returns the project error type rather than a bare value. |
| source | REQ-009 |
| introduced_by | ADR-002 |
| enforced_by | none |

### CON-005 The retry budget is enforced at the boundary

| Field | Value |
|---|---|
| kind | process |
| status | active |
| statement | The retry budget for an order submission is enforced at the boundary rather than inside the use case. |
| source | REQ-005 |
| introduced_by | ADR-002 |
| enforced_by | none |

## Layer dependency rules

| Layer | Depends on | Does not depend on | Constraint |
|---|---|---|---|
| domain | nothing | application, infrastructure, interface | CON-003 |
| application | domain | infrastructure, interface | CON-003 |
| infrastructure | domain | application, interface | CON-003 |
| interface | application, domain | infrastructure | CON-003 |

## Constraint index

| CON | Kind | Status | Title | Source | Introduced by | Enforced by |
|---|---|---|---|---|---|---|
| CON-003 | boundary | active | One write path to the order store | REQ-007 | ADR-002 | AP-002 |
| CON-009 | process | active | Failures carry the error type | REQ-009 | ADR-002 | none |
| CON-005 | process | active | The retry budget is enforced at the boundary | REQ-005 | ADR-002 | none |
