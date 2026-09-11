---
schema: devforgeai/context-architecture-constraints/1
id: architecture-constraints
phase: constitute
status: accepted
produced_by: establishing-context
consumes: [REQ-001]
open_questions: []
---

# Architecture constraints

## Constraints

### CON-002 Retry budget at the boundary

| Field | Value |
|---|---|
| kind | reliability |
| status | active |
| statement | A call that leaves the process carries a retry budget applied at the boundary. |
| source | REQ-001 |
| introduced_by | ADR-011 |
| enforced_by | none |

### CON-005 Single write path

| Field | Value |
|---|---|
| kind | layering |
| status | active |
| statement | The application layer holds the only write path to the order store. |
| source | REQ-002 |
| introduced_by | ADR-001 |
| enforced_by | none |

## Layer dependency rules

| Layer | Depends on | Does not depend on | Constraint |
|---|---|---|---|
| domain | nothing | application | CON-005 |
| application | domain | interface | CON-005 |
| infrastructure | domain | interface | CON-002 |
| interface | application | infrastructure | CON-002 |

## Constraint index

| CON | Kind | Status | Title | Source | Introduced by | Enforced by |
|---|---|---|---|---|---|---|
| CON-002 | reliability | active | Retry budget at the boundary | REQ-001 | ADR-011 | none |
| CON-005 | layering | active | Single write path | REQ-002 | ADR-001 | none |
