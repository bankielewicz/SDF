---
schema: devforgeai/story/1
id: STORY-021
phase: plan
status: ready
produced_by: planning-work
consumes: [EPIC-002, REQ-030, PERSONA-001]
open_questions: []
---

# STORY-021 refund-request

## Story

As the practice manager, a settled payment can be refunded from the screen that shows it, so a patient asking where their money went gets an answer without a second system.

## Screens

| Screen | Kind | UI spec |
|---|---|---|
| refund request form | screen | - |
| refund reason picker | component | - |

## Acceptance criteria

| AC | REQ | Criterion |
|---|---|---|
| AC-021 | REQ-030 | Given a settled payment, when the practice manager raises a refund, then the refund request form takes the payment, the amount, and the date the refund leaves, and the refund appears as awaiting the bank. |
| AC-022 | - | Given a refund being raised, when the practice manager opens the refund reason picker, then a reason is chosen from the practice's list and shown on the refund. |

## Dependencies

STORY-014

## Notes

The reason picker came out of the support inbox rather than the requirements: no requirement in EPIC-002 names it, and a refund can be raised without a reason today.
