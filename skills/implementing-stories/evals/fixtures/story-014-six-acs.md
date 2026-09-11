---
schema: devforgeai/story/1
id: STORY-014
phase: plan
status: building
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
| REQ-007 | A signed-in shopper places the order held in the cart and receives its identifier. | AC-001 AC-002 AC-003 AC-004 AC-005 AC-006 |

## Acceptance Criteria

- AC-001: Given a signed-in shopper with one cart line When the shopper places the order Then the response holds status 201 and one order identifier.
- AC-002: Given a shopper with an empty cart When the shopper places the order Then the response holds status 422 and the message cart is empty.
- AC-003: Given a cart holding two lines priced 40 and 60 When the shopper places the order Then the stored order total is 100.
- AC-004: Given a cart line whose quantity is zero When the shopper places the order Then the response holds status 422 and the message quantity is zero.
- AC-005: Given a signed-out visitor When the visitor places the order Then the response holds status 401.
- AC-006: Given a placed order When the shopper places the same cart again Then the response holds status 409 and the first order identifier.

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
