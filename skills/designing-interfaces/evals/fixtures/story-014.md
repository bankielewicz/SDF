---
schema: devforgeai/story/1
id: STORY-014
phase: plan
status: ready
produced_by: planning-work
consumes: [EPIC-002, REQ-007, REQ-011, REQ-012, PERSONA-001, PERSONA-002, UI-003, UI-004]
open_questions: []
---

# STORY-014 checkout-review

## Story

As the practice manager, the week closes with every unmatched payment either matched to an open invoice or left with a reason, so the bookkeeper starts the month with a trail rather than a mailbox.

## Screens

| Screen | Kind | UI spec |
|---|---|---|
| reconciliation review | screen | UI-003 |
| match confirmation panel | component | UI-004 |

## Acceptance criteria

| AC | REQ | Criterion |
|---|---|---|
| AC-007 | REQ-007, REQ-012 | Given unmatched payments exist for the selected week, when the practice manager opens the reconciliation review screen, then each unmatched payment is listed with the open invoices it could settle and the unreconciled total for that week. |
| AC-008 | REQ-011 | Given a payment and an invoice of the same amount, when the practice manager accepts the match in the match confirmation panel, then the match records the confirming user and the timestamp, and the payment leaves the unmatched list. |
| AC-009 | REQ-007 | Given a payment that settles part of an invoice, when the practice manager accepts it, then the reconciliation review screen shows the invoice as partly settled with the remaining balance, and focus stays on the payment row. |

## Dependencies

none

## Notes

The unreconciled total is read from the same query as the list, so a partial match changes both in one response.
