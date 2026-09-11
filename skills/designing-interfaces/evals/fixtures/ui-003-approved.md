---
schema: devforgeai/ui-spec/1
id: UI-003
phase: design
status: approved
produced_by: designing-interfaces
consumes: [STORY-014, REQ-007, REQ-012, PERSONA-001]
open_questions: []
---

# UI-003 reconciliation review

## Purpose

The practice manager works down the week's unmatched payments and leaves when every one is matched to an invoice or carries a reason.

## Requirements

| REQ | Statement | Satisfied by |
|---|---|---|
| REQ-007 | The practice manager reviews every unmatched payment against open invoices before closing the week. | Unmatched list |
| REQ-012 | The practice manager sees the running total of unreconciled payments for the current week. | Totals bar |

## Anatomy

| Region | Element | Content source | Tokens |
|---|---|---|---|
| Page header | header with h1 and week label | static | TOKEN-color-text TOKEN-type-size-xl TOKEN-spacing-lg |
| Week selector | button group of the four most recent weeks | static | TOKEN-color-surface TOKEN-type-size-sm TOKEN-spacing-md |
| Unmatched list | table of payments, one row per payment | payment | TOKEN-color-surface TOKEN-color-border TOKEN-type-size-base TOKEN-spacing-md |
| Candidate invoices | list inside the expanded payment row | invoice | TOKEN-color-surface-raised TOKEN-type-family-mono TOKEN-spacing-sm |
| Totals bar | region showing the unreconciled total for the week | REQ-012 | TOKEN-color-text-muted TOKEN-type-size-sm TOKEN-spacing-md |
| Accept control | button inside the expanded payment row | static | TOKEN-color-primary TOKEN-color-on-primary TOKEN-radius-md TOKEN-spacing-sm |

## States

| State | Trigger | Visible change | Tokens |
|---|---|---|---|
| default | The week holds unmatched payments | Unmatched list shows one row per payment, Totals bar shows the week total | TOKEN-color-bg |
| loading | A week is chosen and its payments are being read | Unmatched list holds skeleton rows, Week selector stays usable | TOKEN-motion-duration-base |
| empty | The week holds no unmatched payment | Unmatched list is replaced by a line saying the week is closed | TOKEN-color-text-muted |
| error | The week could not be read | Unmatched list is replaced by the failure and a retry control | TOKEN-color-danger |

## Breakpoints

| Name | Min width | Layout | Changes from previous |
|---|---|---|---|
| sm | 320px | One column, candidate invoices open below the payment row | - |
| md | 768px | One column, Totals bar moves beside the Week selector | Totals bar leaves the foot of the page |
| lg | 1024px | Two columns, candidate invoices open beside the payment row | Candidate invoices leave the row flow |
| xl | 1440px | Two columns, list column capped and centred | The list stops growing and the gutters take the width |

## Interaction

| Trigger | Response | Focus after |
|---|---|---|
| Click a payment row | The row expands and Candidate invoices loads for that payment | Candidate invoices |
| Enter or Space on a focused payment row | Same as clicking the row | Candidate invoices |
| Click Accept on a candidate invoice | The payment leaves the Unmatched list and the Totals bar falls by its amount | Unmatched list |
| Escape inside an expanded row | The row collapses | Unmatched list |
| Choose a week in the Week selector | Unmatched list and Totals bar reload for that week | Unmatched list |

## Accessibility

| Check | Requirement | Evidence |
|---|---|---|
| Landmark | Banner wraps Page header, main wraps the list and the totals | Page header |
| Heading order | One h1 in Page header, h2 on Unmatched list and Totals bar, no level skipped | Unmatched list |
| Name | Every row control names the payer and the amount, every accept control names the invoice | Accept control |
| Role | The expanded row is a disclosure, the candidate list is a list, neither invents a role | Candidate invoices |
| Keyboard path | Week selector, then Unmatched list rows in date order, then Accept control inside the open row | Unmatched list |
| Focus visible | A two-sided outline in the accent colour on every focusable element | TOKEN-color-accent |
| Contrast | Body text on the page background, in both themes | 15.1 |
| Motion | The row expansion collapses to an instant change under reduced motion | TOKEN-motion-duration-fast |

## Tokens used

| Token | Group | Where |
|---|---|---|
| TOKEN-color-text | color | Page header |
| TOKEN-color-surface | color | Unmatched list |
| TOKEN-color-surface-raised | color | Candidate invoices |
| TOKEN-color-border | color | Unmatched list |
| TOKEN-color-text-muted | color | Totals bar |
| TOKEN-color-primary | color | Accept control |
| TOKEN-color-on-primary | color | Accept control |
| TOKEN-color-bg | color | default |
| TOKEN-color-danger | color | error |
| TOKEN-color-accent | color | Accept control |
| TOKEN-type-size-xl | type | Page header |
| TOKEN-type-size-sm | type | Totals bar |
| TOKEN-type-size-base | type | Unmatched list |
| TOKEN-type-family-mono | type | Candidate invoices |
| TOKEN-spacing-lg | spacing | Page header |
| TOKEN-spacing-md | spacing | Unmatched list |
| TOKEN-spacing-sm | spacing | Candidate invoices |
| TOKEN-radius-md | radius | Accept control |
| TOKEN-motion-duration-base | motion | loading |
| TOKEN-motion-duration-fast | motion | default |

## Out of scope

Importing the week from the bank export, which UI-002 carries.
Confirming a match and recording who confirmed it, which UI-004 carries.
