---
schema: devforgeai/adr/1
id: ADR-003
phase: constitute
status: accepted
produced_by: establishing-context
consumes: [REQ-007, REQ-011]
open_questions: []
---

# ADR-003: Four layers with one direction of dependency

## Context

Checkout, refunds, and order history each read the same order record and each reached the store directly in the first draft. Three readers of one row shape left every change to that shape touching three call sites, and a change to the payment provider reached the page that renders a confirmation.

## Decision

The source tree carries four layers — domain, application, infrastructure, interface — and a dependency runs one way, from the outside in. A store row becomes a domain value at the infrastructure boundary and travels no further in its stored shape.

## Consequences

A new reader of the order record adds a use case rather than a second query. The boundary conversion is one more file per adapter, which is the cost this decision accepts.

## Constraints introduced

| CON | Statement |
|---|---|
| CON-003 | A store row leaves the infrastructure layer as a domain value. |

## Supersedes

none
