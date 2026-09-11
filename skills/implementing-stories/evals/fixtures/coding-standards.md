---
schema: devforgeai/context-coding-standards/1
id: coding-standards
phase: constitute
status: accepted
produced_by: establishing-context
consumes: [IDEA-004]
open_questions: []
---

# Coding standards

## Formatting

| Key | Value | Source |
|---|---|---|
| style.indent | 2 | ADR-002 |
| style.line.max | 100 | ADR-002 |
| style.quote | double | ADR-002 |

## Naming

| Key | Value | Source |
|---|---|---|
| naming.type | pascal | ADR-002 |
| naming.function | snake | ADR-002 |
| naming.variable | snake | ADR-002 |
| naming.constant | upper-snake | ADR-002 |

## Error handling

| Rule | Applies to |
|---|---|
| A failure leaves the use case as a typed result rather than as a raised value. | application |
| A store failure is wrapped at the adapter boundary and never surfaces its driver type. | infrastructure |

## Logging

| Rule | Applies to |
|---|---|
| One log line per use case entry, carrying the use case name and the subject id. | application |
| No log line carries a cart line price or a shopper identifier. | every layer |

## Testing standards

| Rule | Applies to |
|---|---|
| One spec file mirrors one source file and appends `_spec` to the stem. | every layer |
| A test names the criterion it asserts in its first line. | every layer |
| A test arranges, acts, and asserts in that order, with one assertion block. | every layer |
| An integration spec exercises the real seam between two layers rather than a stand-in. | application, interface |

## Documentation

Each public use case carries one sentence naming what it does and what it returns.

## Design tokens

| Key | Value | Source |
|---|---|---|
| tokens.path | .devforgeai/brand/tokens.json | ADR-004 |
