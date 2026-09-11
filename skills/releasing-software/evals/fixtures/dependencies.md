---
schema: devforgeai/context-dependencies/1
id: dependencies
phase: constitute
status: accepted
produced_by: establishing-context
consumes: [IDEA-003]
open_questions: []
---

# Dependencies

## Approved dependencies

| Name | Version | Scope | License |
|---|---|---|---|
| web-kit | 2.0 | runtime | MIT |
| ledger-client | 3.1 | runtime | Apache-2.0 |
| test-kit | 4.2 | test | MIT |

## Forbidden dependencies

| Name | Reason |
|---|---|
| crypto-lite | The signature primitive it ships is not reviewed. |

## Version policy

A runtime dependency moves by minor version inside a sprint and by major version under an accepted ADR.

## License policy

| Licence | Allowed | Reason |
|---|---|---|
| MIT | yes | Permissive with attribution. |
| Apache-2.0 | yes | Permissive with a patent grant. |
| AGPL-3.0 | no | The network clause reaches the hosted service. |

## Addition procedure

A new dependency arrives as a row of `## Approved dependencies` in the same commit as the ADR that admits it.
