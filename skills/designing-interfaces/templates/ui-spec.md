---
schema: devforgeai/ui-spec/1
id: UI-nnn
phase: design
status: draft
produced_by: designing-interfaces
consumes: []
open_questions: []
---

# UI-nnn <screen name>

## Purpose

<one sentence: what the person is doing here and what leaves it done>

## Requirements

| REQ | Statement | Satisfied by |
|---|---|---|
| REQ-nnn | <the requirement statement, verbatim> | <Region name from ## Anatomy> |

## Anatomy

| Region | Element | Content source | Tokens |
|---|---|---|---|
| <region name> | <semantic element> | <entity name \| REQ-nnn \| static> | TOKEN-color-<leaf> TOKEN-type-<leaf> TOKEN-spacing-<leaf> |

## States

| State | Trigger | Visible change | Tokens |
|---|---|---|---|
| default | <what puts the screen here> | <what the person sees> | TOKEN-color-<leaf> |
| loading | <what puts the screen here> | <what the person sees> | TOKEN-motion-<leaf> |
| empty | <what puts the screen here> | <what the person sees> | TOKEN-color-text-muted |
| error | <what puts the screen here> | <what the person sees> | TOKEN-color-danger |

## Breakpoints

| Name | Min width | Layout | Changes from previous |
|---|---|---|---|
| sm | 320px | <column count and order> | - |
| md | 768px | <column count and order> | <what moves> |
| lg | 1024px | <column count and order> | <what moves> |
| xl | 1440px | <column count and order> | <what moves> |

## Interaction

| Trigger | Response | Focus after |
|---|---|---|
| <pointer, key, or form event> | <what the screen does> | <Region name \| unchanged> |

## Accessibility

| Check | Requirement | Evidence |
|---|---|---|
| Landmark | <the landmark role wrapping each region> | <Region name> |
| Heading order | <the heading levels in document order> | <Region name> |
| Name | <the accessible name of each control> | <Region name> |
| Role | <the role of each non-native control> | <Region name> |
| Keyboard path | <the tab order, region to region> | <Region name> |
| Focus visible | <the focus indicator> | TOKEN-color-<leaf> |
| Contrast | <the pair measured> | <ratio to one decimal> |
| Motion | <what prefers-reduced-motion: reduce changes> | TOKEN-motion-<leaf> |

## Tokens used

| Token | Group | Where |
|---|---|---|
| TOKEN-<group>-<leaf> | <color \| type \| spacing \| radius \| elevation \| motion> | <Region or State> |

## Out of scope

<one sentence naming a behaviour this screen does not carry, and the UI-nnn that does>
