---
schema: devforgeai/context-anti-patterns/1
id: anti-patterns
phase: constitute
status: accepted
produced_by: establishing-context
consumes: []
open_questions: []
---

# Anti-patterns

## Anti-patterns

### AP-004 An unreferenced helper

| Field | Value |
|---|---|
| category | smell |
| severity | medium |
| scope | src/** |
| detector_kind | regex |
| detector | unused_ |
| remediation | The helper is removed or reached from the use case that needs it. |
| source | CON-003 |

## Anti-pattern index

| AP | Category | Severity | Scope | Detector kind | Detector | Source |
|---|---|---|---|---|---|---|
| AP-004 | smell | medium | src/** | regex | unused_ | CON-003 |
