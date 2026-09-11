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

### AP-002 The application layer opens the order store

| Field | Value |
|---|---|
| category | layer |
| severity | blocker |
| scope | src/application/** |
| detector_kind | regex |
| detector | open_store |
| remediation | The use case calls the infrastructure adapter that owns the write path. |
| source | CON-003 |

## Anti-pattern index

| AP | Category | Severity | Scope | Detector kind | Detector | Source |
|---|---|---|---|---|---|---|
| AP-001 | smell | low | src/** | literal | PENDING-MARKER | CON-009 |
| AP-004 | smell | medium | src/** | regex | unused_ | CON-009 |
| AP-002 | layer | blocker | src/application/** | regex | open_store | CON-003 |
