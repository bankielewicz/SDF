---
schema: devforgeai/story/1
id: STORY-014
phase: plan
status: ready
produced_by: planning-work
consumes: [REQ-007, CON-001, AP-001]
open_questions: []
---

# STORY-014: Place the order

## Story

As a Shopper
I want to place the order held in my cart
So that the goods are on their way before I close the page

## Requirements

| REQ | Statement | Covered by |
|---|---|---|
| REQ-007 | A signed-in shopper places the order held in the cart and receives its identifier. | AC-001 AC-002 AC-003 AC-007 |

## Acceptance Criteria

- AC-001: Given a signed-in shopper with one cart line When the shopper places the order Then the response holds status 201 and one order identifier.
- AC-002: Given a shopper with an empty cart When the shopper places the order Then the response holds status 422 and the message cart is empty.
- AC-003: Given a cart holding two lines priced 40 and 60 When the shopper places the order Then the stored order total is 100.
- AC-007: Given a cart When the buyer checks out Then the experience feels fast.

## Constraints

| CON | Statement | Binds |
|---|---|---|
| CON-001 | Single write path: the application layer holds the only write path to the order store. | application |

## Anti-patterns

| AP | Severity | Scope |
|---|---|---|
| AP-001 | high | src/application/** |

## Interface

none

## Layer

application

## Files

| Path | Kind | Layer |
|---|---|---|
| src/application/checkout/place_order.ext | source | application |
| tests/application/checkout/place_order_spec.ext | test | application |
| src/application/checkout/order_total.ext | source | application |

## Dependencies

none

## Out of scope

Payment capture, which STORY-016 carries.
