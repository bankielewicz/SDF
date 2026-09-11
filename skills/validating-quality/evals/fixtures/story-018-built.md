---
schema: devforgeai/story/1
id: STORY-018
phase: plan
status: built
produced_by: planning-work
consumes: [REQ-007, CON-003, AP-002]
open_questions: []
---

# Checkout use case

## Story

As a buyer
I want checkout use case
So that the order reaches the store once

## Requirements

| REQ | Statement | Covered by |
|---|---|---|
| REQ-007 | A buyer submits an order and the store records it once. | AC-011 AC-012 |

## Acceptance Criteria

- AC-011: Given a submitted order When checkout runs Then the order is written once.
- AC-012: Given a rejected order When checkout runs Then no write reaches the store.

## Constraints

| CON | Statement | Binds |
|---|---|---|
| CON-003 | One write path to the order store: the order store is written through one path in the infrastructure layer. | src/application/checkout.ext |

## Anti-patterns

| AP | Severity | Scope |
|---|---|---|
| AP-002 | blocker | src/application/** |

## Interface

none

## Layer

application

## Files

| Path | Kind | Layer |
|---|---|---|
| src/application/checkout.ext | source | application |
| tests/checkout_test.ext | test | application |

## Dependencies

none

## Out of scope

The payment capture, which STORY-015 carries.
