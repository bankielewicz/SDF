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

### AP-004 Store row as the return type

| Field | Value |
|---|---|
| category | layer |
| severity | high |
| scope | src/infrastructure/** |
| detector_kind | regex |
| detector | return row |
| remediation | The adapter returns the domain type the application layer reads. |
| source | CON-005 |

## Anti-pattern index

| AP | Category | Severity | Scope | Detector kind | Detector | Source |
|---|---|---|---|---|---|---|
| AP-004 | layer | high | src/infrastructure/** | regex | return row | CON-005 |
