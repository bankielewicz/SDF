---
schema: devforgeai/context-coding-standards/1
id: coding-standards
phase: constitute
status: accepted
produced_by: establishing-context
consumes: [IDEA-002]
open_questions: []
---

# Coding standards

## Formatting

| Key | Value | Source |
|---|---|---|
| style.indent | 4 | coding-standards.md |
| style.line.max | 100 | coding-standards.md |
| style.quote | double | coding-standards.md |

## Naming

| Key | Value | Source |
|---|---|---|
| naming.type | pascal | coding-standards.md |
| naming.function | snake | coding-standards.md |
| naming.variable | snake | coding-standards.md |
| naming.constant | upper-snake | coding-standards.md |

## Error handling

A failure path returns the project's error type and carries the failing id.
A bare value on a failure path leaves the caller with nothing to branch on.
Enforced by CON-009.

## Logging

A log line at the boundary carries the request id and no order payload.

## Testing standards

One test file per source file, named after it, holding one case per acceptance
criterion.

## Documentation

A public declaration carries one sentence naming what it returns.

## Design tokens

| Key | Value | Source |
|---|---|---|
| tokens.path | .devforgeai/brand/tokens.json | coding-standards.md |
