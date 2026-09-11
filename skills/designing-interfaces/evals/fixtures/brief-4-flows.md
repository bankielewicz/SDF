---
schema: devforgeai/explore-brief/1
id: IDEA-001
phase: explore
status: specified
produced_by: exploring-ideas
consumes: []
open_questions: []
---

# IDEA-001 payment-reconciliation

## Problem statement

A practice manager loses an afternoon a week matching bank payments to open invoices by eye, and one payment in twenty stays unmatched into the next month.

## Target user

Single-site clinics with one part-time administrator.

Books between 40 and 300 invoices a month.
Takes payment by bank transfer and by card terminal, in two different exports.
Uses a spreadsheet for the matching step today.

## What they do today

| Current approach | Cost | Where it breaks |
|---|---|---|
| Bank export beside the invoice list in a spreadsheet | Three to four hours a week | Two payments of the same amount on the same day |
| Asking the patient for the reference | One call per unmatched payment | The patient paid from a different account |
| Leaving it for the accountant at year end | One reconciliation fee | The trail is cold by then |

## Why now

2025-11 — Open banking exports became available to every small business account at the two banks these clinics use.
2026-03 — The card terminal provider started publishing settlement references in its daily export.

## Competitor scan

| Name | URL | Approach | Price | Gap |
|---|---|---|---|---|
| Ledgerly | https://example.com/ledgerly | Full bookkeeping suite with reconciliation as one module | 49 per seat per month | Priced and scoped for a bookkeeper, not for one administrator |
| MatchMate | https://example.com/matchmate | Bank feed with automatic matching rules | 19 per month | No partial matches, so a part-paid invoice stays unmatched |

## Technology scan

| Capability | Candidate | Maturity | License | Source |
|---|---|---|---|---|
| Bank export parsing | ofx and camt parsers | established | MIT | https://example.com/ofx |
| Fuzzy amount and date matching | record linkage libraries | established | BSD | https://example.com/linkage |

## Core flows

| ID | Actor | Trigger | Steps | Outcome |
|---|---|---|---|---|
| FLOW-001 | Practice manager | A patient asks for their money back | Open the settled payment -> raise a refund -> set amount and date -> submit | The refund is raised and awaiting the bank |
| FLOW-002 | Bookkeeper | The monthly close | Open the refund list -> filter to awaiting the bank -> mark the cleared ones | Every refund is either awaiting the bank or cleared |
| FLOW-003 | Practice manager | Friday afternoon close | Import the week -> open the review screen -> match each payment to an invoice -> accept | Every payment for the week is matched or has a reason |
| FLOW-004 | Practice manager | The accountant asks for the quarter | Choose the period -> export the reconciled rows -> send the file | The accountant has the quarter in one file |

## Non-goals

The product does not take payments.
The product does not replace the practice's invoicing system.
The product does not file anything with a tax authority.

## Success signal

unmatched payments at month end | below 1 percent of payments | first three months of use

## Mockups

| Flow | Screen | Path | State |
|---|---|---|---|

## Seed data

| Entity | Rows | Field count |
|---|---|---|
| payment | 6 | 5 |
| invoice | 6 | 5 |

## Prototype

| Field | Value |
|---|---|
| Built | no |
| Path | - |
| Entry | - |
| Flows covered | - |
