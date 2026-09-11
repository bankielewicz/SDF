---
schema: devforgeai/context-coding-standards/1
id: coding-standards
phase: constitute
status: accepted
produced_by: establishing-context
consumes: []
open_questions: []
---

# Coding standards

## Formatting

| Key | Value | Source |
|---|---|---|
| format.tool | the formatter configured in config.toml | stack detect |
| format.line_length | 100 | team |
| format.indent | 2 spaces | team |

## Naming

| Key | Value | Source |
|---|---|---|
| naming.file | kebab-case | team |
| naming.type | PascalCase | team |
| naming.function | camelCase | team |
| naming.constant | SCREAMING_SNAKE_CASE | team |

## Error handling

Errors carry the id of the record they concern. A caught error is either handled where it is caught or re-raised with the same id attached.

A user-visible failure names what did not happen and what to do next.

## Logging

| Key | Value | Source |
|---|---|---|
| log.format | structured, one object per line | team |
| log.levels | error, warn, info, debug | team |
| log.forbidden_fields | payer name, account number | team |

## Testing standards

| Key | Value | Source |
|---|---|---|
| test.naming | the AC id appears in the case name | team |
| test.isolation | one case touches one behaviour | team |

## Documentation

Every exported symbol carries one line saying what it is for. A module carries one paragraph saying what it owns.

## Design tokens

| Key | Value | Source |
|---|---|---|
| tokens.path | .devforgeai/brand/tokens.json | Design |

A color or type literal in a frontend file resolves to a token name declared in the file at `tokens.path`. `devforgeai design lint` reads `tokens.path` from this section and the frontend globs from `.devforgeai/config.toml`.
