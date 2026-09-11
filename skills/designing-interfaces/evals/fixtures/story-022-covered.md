---
schema: devforgeai/story/1
id: STORY-022
phase: plan
status: ready
produced_by: planning-work
consumes: [EPIC-002, REQ-030, REQ-032, PERSONA-001]
open_questions: []
---

# STORY-022 reconciliation-review

## Story

As the practice manager, the week's unmatched payments can be reviewed against open invoices from one screen, so Friday's close is one pass rather than two systems.

## Screens

| Screen | Kind | UI spec |
|---|---|---|
| reconciliation review | screen | - |
| refund request form | screen | - |

## Acceptance criteria

| AC | REQ | Criterion |
|---|---|---|
| AC-030 | REQ-032 | Given a week of imported payments, when the practice manager opens the reconciliation review, then every unmatched payment is listed beside the open invoices it could settle. |
| AC-031 | REQ-030 | Given a settled payment, when the practice manager raises a refund, then the refund request form takes the payment, the amount, and the date the refund leaves. |

## Dependencies

STORY-014

## Notes

Every screen this story asks for realizes a requirement: the review screen is REQ-032 and the refund form is REQ-030. What no requirement carries is FLOW-004, the quarterly export the brief lists as a core flow, which is why this run's audit reports a flow rather than a screen.
