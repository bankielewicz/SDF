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

### CON-003 Single write path

| Field | Value |
|---|---|
| kind | layering |
| status | active |
| statement | The application layer holds the only write path to the order store. |
| source | REQ-001 |
| introduced_by | ADR-001 |
| enforced_by | none |

## Layer dependency rules

| Layer | Depends on | Does not depend on | Constraint |
|---|---|---|---|
| domain | nothing | application | CON-003 |
| application | domain | nothing | CON-003 |

## Constraint index

| CON | Kind | Status | Title | Source | Introduced by | Enforced by |
|---|---|---|---|---|---|---|
| CON-003 | layering | active | Single write path | REQ-001 | ADR-001 | none |

### CON-001 A rule CON-001 names

| Field | Value |
|---|---|
| kind | layering |
| status | active |
| statement | A rule CON-001 names. |
| source | REQ-007 |
| introduced_by | ADR-003 |
| enforced_by | none |
| CON-001 | layering | active | A rule CON-001 names | REQ-007 | ADR-003 | none |

