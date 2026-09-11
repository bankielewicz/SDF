---
schema: devforgeai/context-tech-stack/1
id: tech-stack
phase: constitute
status: accepted
produced_by: establishing-context
consumes: []
open_questions: []
---

# Tech stack

## Languages

| Key | Value | Source |
|---|---|---|
| language.primary | primary | ADR-001 |

## Runtimes

| Key | Value | Source |
|---|---|---|
| runtime.name | primary-runtime | ADR-001 |

## Frameworks

| Key | Value | Source |
|---|---|---|
| framework.test | primary-test | ADR-002 |

## Data stores

| Key | Value | Source |
|---|---|---|
| datastore.primary | primary-store | ADR-003 |

## Tooling

| Key | Value | Source |
|---|---|---|
| framework.lint | primary-lint | ADR-002 |

## Excluded technologies

| Name | Reason | Source |
|---|---|---|
| second-store | One store keeps the write path single. | CON-001 |
