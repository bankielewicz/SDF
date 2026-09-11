---
schema: devforgeai/story/1
id: STORY-nnn
phase: plan
status: draft
produced_by: planning-work
consumes: []
open_questions: []
---

# STORY-nnn: <title, 1 to 60 characters>

## Story

As a <persona name>
I want <one clause>
So that <one clause>

## Requirements

| REQ | Statement | Covered by |
|---|---|---|
| REQ-nnn | <the requirements[].statement value, copied byte for byte> | AC-nnn AC-nnn |

## Acceptance Criteria

- AC-nnn: Given <state> When <action> Then <one outcome a test reads>.

## Constraints

| CON | Statement | Binds |
|---|---|---|
| CON-nnn | <index Title>: <constraint statement> | <a Path value from ## Files, or the ## Layer value> |

## Anti-patterns

| AP | Severity | Scope |
|---|---|---|
| AP-nnn | <blocker \| high \| medium \| low> | <glob> |

## Interface

| UI | Screen | States covered |
|---|---|---|
| UI-nnn | <the UI spec's ## Purpose first sentence> | default loading empty |

## Layer

<a Layer value from source-tree.md ## Layers>

## Files

| Path | Kind | Layer |
|---|---|---|
| <repo-relative path> | <source \| test \| config \| migration \| asset> | <a Layer value> |

## Dependencies

- STORY-nnn: <one sentence naming what this story takes from it>

## Out of scope

<one sentence naming a behaviour this story does not carry, and the STORY-nnn that does when one exists>
