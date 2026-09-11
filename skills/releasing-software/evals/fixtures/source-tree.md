---
schema: devforgeai/context-source-tree/1
id: source-tree
phase: constitute
status: accepted
produced_by: establishing-context
consumes: [IDEA-003]
open_questions: []
---

# Source tree

## Roots

| Key | Value | Source |
|---|---|---|
| source.root | src | config.toml |
| test.root | tests | config.toml |
| build.output.root | dist | config.toml |

## Layers

| Key | Value | Source |
|---|---|---|
| layer.domain.path | src/domain/** | ADR-003 |
| layer.application.path | src/application/** | ADR-003 |
| layer.infrastructure.path | src/infrastructure/** | ADR-003 |
| layer.interface.path | src/interface/** | ADR-003 |

| Layer | Path | Depends on |
|---|---|---|
| domain | src/domain/** | none |
| application | src/application/** | domain |
| infrastructure | src/infrastructure/** | domain |
| interface | src/interface/** | application |

## Directory map

| Path | Holds |
|---|---|
| src/domain | Entities and the rules over them. |
| src/application | Use cases that orchestrate domain values. |
| src/infrastructure | Adapters onto the store and the payment provider. |
| src/interface | The request surface and the rendered pages. |

## File placement rules

| Path pattern | Kind |
|---|---|
| src/**/*.ext | source |
| tests/**/*_test.ext | test |

## Naming conventions

| Key | Value | Source |
|---|---|---|
| naming.file | snake | ADR-003 |
| naming.directory | snake | ADR-003 |
| naming.test-file | snake | ADR-003 |

## Generated and excluded paths

- `dist/**`
- `.devforgeai/**`
