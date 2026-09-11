---
schema: devforgeai/context-dependencies/1
id: dependencies
phase: constitute
status: accepted
produced_by: establishing-context
consumes: []
open_questions: []
---

# Dependencies

## Approved dependencies

| Name | Scope | Version | Used for |
|---|---|---|---|
| store-client | runtime | 2.x | the order store adapter |
| spec-harness | test | 5.x | the test harness every spec file loads |

## Forbidden dependencies

| Name | Reason |
|---|---|
| direct-store-driver | It opens a second write path around the application layer. |

## Version policy

| Key | Value | Source |
|---|---|---|
| dep.store-client.version | 2.x | ADR-003 |
| dep.store-client.scope | runtime | ADR-003 |

## License policy

Permissive licences are approved. A copyleft licence is an ADR.

## Addition procedure

A new dependency arrives as an ADR naming the requirement behind it, and the approved table gains its row when that ADR reaches `accepted`.
