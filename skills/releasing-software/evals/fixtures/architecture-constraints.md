---
schema: devforgeai/context-architecture-constraints/1
id: architecture-constraints
phase: constitute
status: accepted
produced_by: establishing-context
consumes: [IDEA-003]
open_questions: []
---

# Architecture constraints

## Constraints

| CON | Kind | Statement |
|---|---|---|
| CON-003 | boundary | A store row leaves the infrastructure layer as a domain value. |
| CON-005 | security | Every inbound callback carries a signature the receiver compares before reading the payload. |
| CON-012 | process | A deployed workload declares a readiness probe and a resource ceiling. |

## Layer dependency rules

| Layer | Depends on | Does not depend on | Constraint |
|---|---|---|---|
| domain | none | application, infrastructure, interface | CON-003 |
| application | domain | infrastructure, interface | CON-003 |
| infrastructure | domain | application, interface | CON-003 |
| interface | application | infrastructure | CON-003 |


### CON-003 A rule CON-003 names

| Field | Value |
|---|---|
| kind | layering |
| status | active |
| statement | A rule CON-003 names. |
| source | REQ-007 |
| introduced_by | ADR-003 |
| enforced_by | none |


### CON-005 A rule CON-005 names

| Field | Value |
|---|---|
| kind | layering |
| status | active |
| statement | A rule CON-005 names. |
| source | REQ-007 |
| introduced_by | ADR-003 |
| enforced_by | none |


### CON-012 A rule CON-012 names

| Field | Value |
|---|---|
| kind | layering |
| status | active |
| statement | A rule CON-012 names. |
| source | REQ-007 |
| introduced_by | ADR-003 |
| enforced_by | none |

## Constraint index

| CON | Title | Kind | Status | Enforced by |
|---|---|---|---|---|
| CON-003 | Boundary types | boundary | active | devforgeai context audit |
| CON-005 | Callback signatures | security | active | AP-004 |
| CON-012 | Deployed workload shape | process | active | devforgeai doc validate |
| CON-003 | layering | active | A rule CON-003 names | REQ-007 | ADR-003 | none |
| CON-005 | layering | active | A rule CON-005 names | REQ-007 | ADR-003 | none |
| CON-012 | layering | active | A rule CON-012 names | REQ-007 | ADR-003 | none |

