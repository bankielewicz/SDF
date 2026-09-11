---
schema: devforgeai/context-anti-patterns/1
id: anti-patterns
phase: constitute
status: accepted
produced_by: establishing-context
consumes: [IDEA-004]
open_questions: []
---

# Anti-patterns

## Anti-patterns

### AP-001 Store write outside the application layer

| Field | Value |
|---|---|
| category | layer |
| severity | high |
| scope | src/application/** |
| detector_kind | regex |
| detector | driver\\.execute\\( |
| remediation | The write routes through the order store adapter the application layer owns. |
| source | CON-001 |

## Anti-pattern index

| AP | Category | Severity | Scope | Detector kind | Detector | Source |
|---|---|---|---|---|---|---|
| AP-001 | layer | high | src/application/** | regex | driver\\.execute\\( | CON-001 |
