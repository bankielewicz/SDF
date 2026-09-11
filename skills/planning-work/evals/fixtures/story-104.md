---
schema: devforgeai/story/1
id: STORY-104
phase: plan
status: building
produced_by: planning-work
consumes: [REQ-001]
open_questions: []
---

# STORY-104: Place the order

## Story

As a Shopper
I want to place an order from one page
So that I finish without a page change

## Requirements

| REQ | Statement | Covered by |
|---|---|---|
| REQ-001 | A signed-in shopper completes checkout on one page. | AC-018 AC-019 |

## Acceptance Criteria

- AC-018: Given a signed-in shopper with one cart item When the shopper submits the form Then the response holds status 201 and one order id.
- AC-019: Given a signed-in shopper When the shopper submits the form Then the page looks right.

## Constraints

| CON | Statement | Binds |
|---|---|---|
| CON-003 | Single write path: The application layer holds the only write path to the order store. | application |

## Anti-patterns

none

## Interface

none

## Layer

application

## Files

| Path | Kind | Layer |
|---|---|---|
| src/application/checkout/place_order.ext | source | application |
| tests/application/checkout/place_order_test.ext | test | application |

## Dependencies

none

## Out of scope

