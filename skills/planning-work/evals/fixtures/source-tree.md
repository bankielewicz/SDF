---
schema: devforgeai/context-source-tree/1
id: source-tree
phase: constitute
status: accepted
produced_by: establishing-context
consumes: []
open_questions: []
---

# Source tree

## Roots

| Key | Value | Source |
|---|---|---|
| source.root | src | config.toml |
| test.root | tests | config.toml |
| build.output.root | out | config.toml |

## Layers

| Key | Value | Source |
|---|---|---|
| layer.domain.path | src/domain/** | ADR-001 |
| layer.application.path | src/application/** | ADR-001 |

| Layer | Purpose |
|---|---|
| domain | Holds entities and rules with no outward dependency. |
| application | Holds use cases that orchestrate the domain. |

## Directory map

```
src/domain/        entities
src/application/   use cases
```

## File placement rules

| Artifact kind | Path pattern | Example |
|---|---|---|
| entity or rule | src/domain/**/*.ext | src/domain/checkout/order.ext |
| use case | src/application/**/*.ext | src/application/checkout/place_order.ext |
| unit test | tests/**/*_test.ext | tests/application/checkout/place_order_test.ext |

## Naming conventions

| Key | Value | Source |
|---|---|---|
| naming.file | snake | ADR-001 |
| naming.directory | snake | ADR-001 |
| naming.test-file | snake | ADR-001 |

## Generated and excluded paths

| Glob | Origin |
|---|---|
| out/** | build |
