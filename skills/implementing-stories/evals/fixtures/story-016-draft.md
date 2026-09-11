---
schema: devforgeai/story/1
id: STORY-016
phase: plan
status: draft
produced_by: planning-work
consumes: [REQ-011]
open_questions: []
---

# STORY-016: Capture the payment

## Story

As a Shopper
I want the payment for my order captured
So that the goods are released once the money has moved

## Requirements

| REQ | Statement | Covered by |
|---|---|---|
| REQ-011 | A shopper sees the order total before the order is placed. | AC-010 |

## Acceptance Criteria

- AC-010: Given an order awaiting payment When the capture runs Then the stored order carries the captured amount.

## Files

| Path | Kind | Layer |
|---|---|---|
| src/application/checkout/capture_payment.ext | source | application |
| tests/application/checkout/capture_payment_spec.ext | test | application |

## Dependencies

none

## Notes

This story exists so STORY-014's `## Out of scope` line resolves. No case builds
it, and nothing in the corpus asserts on its content.
