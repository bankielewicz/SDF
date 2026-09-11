---
schema: devforgeai/context-tech-stack/1
id: tech-stack
phase: constitute
status: draft
produced_by: establishing-context
consumes: []
open_questions: []
---

# Tech stack

## Languages

| Key | Value | Source |
|---|---|---|
| language.primary | <identifier> | config.toml |
| language.primary.version | <version> | config.toml |

## Runtimes

| Key | Value | Source |
|---|---|---|
| runtime.name | <identifier> | config.toml |
| runtime.version | <version> | config.toml |

## Frameworks

| Key | Value | Source |
|---|---|---|
| framework.<role> | <identifier> | ADR-nnn |
| framework.<role>.version | <version> | ADR-nnn |

## Data stores

| Key | Value | Source |
|---|---|---|
| datastore.<role> | <identifier> | ADR-nnn |
| datastore.<role>.version | <version> | ADR-nnn |

## Tooling

| Key | Value | Source |
|---|---|---|
| framework.test | <identifier> | config.toml |
| framework.build | <identifier> | config.toml |
| framework.lint | <identifier> | config.toml |
| framework.format | <identifier> | config.toml |
| framework.package | <identifier> | config.toml |

## Excluded technologies

| Technology | Reason | Recorded in |
|---|---|---|
| <identifier> | <one sentence> | ADR-nnn |
