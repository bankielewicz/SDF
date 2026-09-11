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

### AP-002 Direct store write

| Field | Value |
|---|---|
| category | layer |
| severity | high |
| scope | src/domain/** |
| detector_kind | regex |
| detector | store\\.write\\( |
| remediation | The call routes through the application layer. |
| source | CON-003 |

## Anti-pattern index

| AP | Category | Severity | Scope | Detector kind | Detector | Source |
|---|---|---|---|---|---|---|
| AP-002 | layer | high | src/domain/** | regex | store\\.write\\( | CON-003 |
