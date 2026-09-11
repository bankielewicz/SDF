---
schema: devforgeai/context-coding-standards/1
id: coding-standards
phase: constitute
status: draft
produced_by: establishing-context
consumes: []
open_questions: []
---

# Coding standards

## Formatting

| Key | Value | Source |
|---|---|---|
| style.indent | <integer> | config.toml |
| style.line.max | <integer> | config.toml |
| style.quote | <single \| double> | config.toml |

## Naming

| Key | Value | Source |
|---|---|---|
| naming.type | <camel \| pascal \| snake \| kebab \| upper-snake> | ADR-nnn |
| naming.function | <camel \| pascal \| snake \| kebab \| upper-snake> | ADR-nnn |
| naming.variable | <camel \| pascal \| snake \| kebab \| upper-snake> | ADR-nnn |
| naming.constant | <camel \| pascal \| snake \| kebab \| upper-snake> | ADR-nnn |

## Error handling

| Rule | Scope | Source |
|---|---|---|
| <declarative sentence> | <glob> | <CON-nnn or ADR-nnn> |

## Logging

| Rule | Scope | Source |
|---|---|---|
| <declarative sentence> | <glob> | <CON-nnn or ADR-nnn> |

## Testing standards

| Rule | Scope | Source |
|---|---|---|
| <declarative sentence> | <glob> | <CON-nnn or ADR-nnn> |

## Documentation

| Rule | Scope | Source |
|---|---|---|
| <declarative sentence> | <glob> | <CON-nnn or ADR-nnn> |

## Design tokens

| Key | Value | Source |
|---|---|---|
| tokens.path | .devforgeai/brand/tokens.json | Design |

A color or type literal in a frontend file resolves to a token name declared in the file at `tokens.path`. `devforgeai design lint` reads `tokens.path` from this section and the frontend globs from `.devforgeai/config.toml`.
