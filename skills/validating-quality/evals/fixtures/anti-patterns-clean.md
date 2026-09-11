---
schema: devforgeai/context-anti-patterns/1
id: anti-patterns
phase: constitute
status: accepted
produced_by: establishing-context
consumes: [IDEA-002]
open_questions: []
---

# Anti-patterns

## Anti-patterns

### AP-001 A left marker

| Field | Value |
|---|---|
| category | smell |
| severity | low |
| scope | src/** |
| detector_kind | literal |
| detector | PENDING-MARKER |
| remediation | The marker is removed with the work it named. |
| source | CON-009 |

### AP-002 A driver type at the boundary

| Field | Value |
|---|---|
| category | layering |
| severity | high |
| scope | src/application/** |
| detector_kind | regex |
| detector | driver_error |
| remediation | The adapter wraps the failure and the use case reads the wrapped type. |
| source | CON-003 |

### AP-004 An unreferenced helper

| Field | Value |
|---|---|
| category | smell |
| severity | medium |
| scope | src/** |
| detector_kind | regex |
| detector | unused_ |
| remediation | The helper is removed or reached from the use case that needs it. |
| source | CON-009 |

## Anti-pattern index

| AP | Category | Severity | Scope | Detector kind | Detector | Source |
|---|---|---|---|---|---|---|
| AP-001 | smell | low | src/** | literal | PENDING-MARKER | CON-009 |
| AP-002 | layering | high | src/application/** | regex | driver_error | CON-003 |
| AP-004 | smell | medium | src/** | regex | unused_ | CON-009 |
