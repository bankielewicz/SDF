---
schema: devforgeai/context-dependencies/1
id: dependencies
phase: constitute
status: accepted
produced_by: establishing-context
consumes: [IDEA-002]
open_questions: []
---

# Dependencies

## Approved dependencies

| Key | Value | Source |
|---|---|---|
| dep.orderstore.version | 2.x | dependencies.md |
| dep.orderstore.scope | runtime | dependencies.md |

## Forbidden dependencies

| Name | Reason |
|---|---|
| remote-eval | It executes text received from a request. |
| legacy-crypto | It carries a key derivation the project retired. |

## Version policy

A runtime dependency moves by minor version inside one release.

## License policy

A dependency whose license bars redistribution is out.

## Addition procedure

A new dependency arrives with an ADR naming what it replaces.
