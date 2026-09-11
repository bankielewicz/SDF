---
schema: devforgeai/context-tech-stack/1
id: tech-stack
phase: constitute
status: accepted
produced_by: establishing-context
consumes: [IDEA-003]
open_questions: []
---

# Tech stack

## Languages

| Key | Value | Source |
|---|---|---|
| language.primary | primary | config.toml |
| language.primary.version | 1.4 | config.toml |

## Runtimes

| Key | Value | Source |
|---|---|---|
| runtime.name | primary-runtime | config.toml |
| runtime.version | 1.4 | config.toml |

## Frameworks

| Key | Value | Source |
|---|---|---|
| framework.web | web-kit | ADR-003 |
| framework.web.version | 2.0 | ADR-003 |

## Data stores

| Key | Value | Source |
|---|---|---|
| datastore.primary | ledger-store | ADR-003 |
| datastore.primary.version | 16 | ADR-003 |

## Tooling

| Key | Value | Source |
|---|---|---|
| framework.test | test-kit | config.toml |

## Excluded technologies

- A second data store is added only under an accepted ADR.
- No message broker is introduced for the checkout path.
