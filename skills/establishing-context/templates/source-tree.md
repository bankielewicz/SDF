---
schema: devforgeai/context-source-tree/1
id: source-tree
phase: constitute
status: draft
produced_by: establishing-context
consumes: []
open_questions: []
---

# Source tree

## Roots

| Key | Value | Source |
|---|---|---|
| source.root | <path> | config.toml |
| test.root | <path> | config.toml |
| build.output.root | <path> | config.toml |

## Layers

| Key | Value | Source |
|---|---|---|
| layer.<name>.path | <glob> | ADR-nnn |

| Layer | Purpose |
|---|---|
| <name> | <one sentence> |

## Directory map

```
<path>/            <one phrase>
<path>/<sub>/      <one phrase>
```

## File placement rules

| Artifact kind | Path pattern | Example |
|---|---|---|
| <kind> | <glob> | <path> |

## Naming conventions

| Key | Value | Source |
|---|---|---|
| naming.file | <camel \| pascal \| snake \| kebab \| upper-snake> | ADR-nnn |
| naming.directory | <camel \| pascal \| snake \| kebab \| upper-snake> | ADR-nnn |
| naming.test-file | <camel \| pascal \| snake \| kebab \| upper-snake> | ADR-nnn |

## Generated and excluded paths

| Glob | Origin |
|---|---|
| <glob> | <generator name or `vendored`> |
