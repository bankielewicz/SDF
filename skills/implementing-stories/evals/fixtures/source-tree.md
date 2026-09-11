---
schema: devforgeai/context-source-tree/1
id: source-tree
phase: constitute
status: accepted
produced_by: establishing-context
consumes: [IDEA-004]
open_questions: []
---

# Source tree

## Roots

| Key | Value | Source |
|---|---|---|
| source.root | src | ADR-001 |
| test.root | tests | ADR-001 |
| build.output.root | out | ADR-001 |

## Layers

| Key | Value | Source |
|---|---|---|
| layer.domain.path | src/domain/** | ADR-001 |
| layer.application.path | src/application/** | ADR-001 |
| layer.infrastructure.path | src/infrastructure/** | ADR-001 |
| layer.interface.path | src/api/** | ADR-001 |

| Layer | Holds | Depends on |
|---|---|---|
| domain | entities and their rules | none |
| application | use cases and the write path | domain |
| infrastructure | stores and external calls | domain |
| interface | request handling and rendering | application |

## Directory map

| Path | Holds |
|---|---|
| src/domain | entities |
| src/application | use cases |
| src/infrastructure | adapters |
| src/api | handlers |
| tests | one mirror directory per source layer |

## File placement rules

| Path pattern | Kind | Layer |
|---|---|---|
| src/domain/**/*.ext | source | domain |
| src/application/**/*.ext | source | application |
| src/infrastructure/**/*.ext | source | infrastructure |
| src/api/**/*.ext | source | interface |
| tests/**/*_spec.ext | test | the layer the mirrored path names |

## Naming conventions

| Key | Value | Source |
|---|---|---|
| naming.file | snake | ADR-001 |
| naming.directory | snake | ADR-001 |
| naming.test-file | snake | ADR-001 |

A test file mirrors its source path and appends `_spec` to the stem.

## Generated and excluded paths

| Glob | Reason |
|---|---|
| out/** | build output |
| coverage/** | coverage output |
