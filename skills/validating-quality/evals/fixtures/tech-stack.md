---
schema: devforgeai/context-tech-stack/1
id: tech-stack
phase: constitute
status: accepted
produced_by: establishing-context
consumes: [IDEA-002]
open_questions: []
---

# Tech stack

## Languages

| Key | Value | Source |
|---|---|---|
| language.primary | primary | config.toml |
| language.primary.version | 1.0 | config.toml |

## Runtimes

| Key | Value | Source |
|---|---|---|
| runtime.name | runtime | config.toml |
| runtime.version | 1.0 | config.toml |

## Frameworks

| Key | Value | Source |
|---|---|---|
| framework.test | harness | config.toml |

## Data stores

| Key | Value | Source |
|---|---|---|
| datastore.primary | orderstore | config.toml |

## Tooling

| Key | Value | Source |
|---|---|---|
| framework.build | builder | config.toml |

## Excluded technologies

- A second data store for order state.
