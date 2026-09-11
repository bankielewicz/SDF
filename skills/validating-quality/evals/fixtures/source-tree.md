---
schema: devforgeai/context-source-tree/1
id: source-tree
phase: constitute
status: accepted
produced_by: establishing-context
consumes: [IDEA-002]
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

| Layer | Purpose |
|---|---|
| domain | The order rules, with no reference to any adapter. |
| application | The use cases that sequence the domain rules. |
| infrastructure | The adapters that reach the order store. |
| interface | The request handlers and the rendered screens. |

| Key | Value | Source |
|---|---|---|
| layer.domain.path | src/domain/** | source-tree.md |
| layer.application.path | src/application/** | source-tree.md |
| layer.infrastructure.path | src/infrastructure/** | source-tree.md |
| layer.interface.path | src/api/** | source-tree.md |

## Directory map

| Path | Holds |
|---|---|
| src/domain | domain types and rules |
| src/application | use cases |
| src/infrastructure | store adapters |
| src/api | request handlers |
| tests | every test file |

## File placement rules

| Path pattern | Kind | Layer |
|---|---|---|
| src/domain/** | source | domain |
| src/application/** | source | application |
| src/infrastructure/** | source | infrastructure |
| src/api/** | source | interface |
| tests/** | test | the layer of the file under test |

## Naming conventions

| Key | Value | Source |
|---|---|---|
| naming.file | snake | source-tree.md |
| naming.directory | snake | source-tree.md |
| naming.test-file | snake | source-tree.md |

## Generated and excluded paths

- out/**
- tests/fixtures/**
