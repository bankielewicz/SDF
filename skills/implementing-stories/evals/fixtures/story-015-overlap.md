---
schema: devforgeai/story/1
id: STORY-015
phase: plan
status: ready
produced_by: planning-work
consumes: [REQ-011, CON-001]
open_questions: []
---

# STORY-015: Apply a discount code

## Story

As a Shopper
I want to apply a discount code to my cart
So that the order total reflects the code before I place it

## Requirements

| REQ | Statement | Covered by |
|---|---|---|
| REQ-011 | A shopper applies one discount code to the cart before placing the order. | AC-020 AC-021 |

## Acceptance Criteria

- AC-020: Given a cart totalling 100 and an active code worth 10 percent When the shopper applies the code Then the stored order total is 90.
- AC-021: Given an expired code When the shopper applies the code Then the response holds status 422 and the message code expired.

## Constraints

| CON | Statement | Binds |
|---|---|---|
| CON-001 | Single write path: the application layer holds the only write path to the order store. | application |

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
| src/application/checkout/discount.ext | source | application |
| tests/application/checkout/discount_spec.ext | test | application |

## Dependencies

- STORY-014: the order total this story discounts.

## Out of scope

Stacking two codes on one cart, which no story carries yet.
