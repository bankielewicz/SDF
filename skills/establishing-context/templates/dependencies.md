---
schema: devforgeai/context-dependencies/1
id: dependencies
phase: constitute
status: draft
produced_by: establishing-context
consumes: []
open_questions: []
---

# Dependencies

## Approved dependencies

| Name | Scope | Purpose | Layer | Admitted by |
|---|---|---|---|---|
| <identifier> | <runtime \| dev \| test \| build> | <one sentence> | <layer name or `all`> | ADR-nnn |

## Forbidden dependencies

| Name | Reason | Replacement | Recorded in |
|---|---|---|---|
| <identifier> | <one sentence> | <identifier or `none`> | <AP-nnn or CON-nnn> |

## Version policy

| Key | Value | Source |
|---|---|---|
| dep.<name>.version | <version constraint> | ADR-nnn |
| dep.<name>.scope | <runtime \| dev \| test \| build> | ADR-nnn |

## License policy

| License identifier | Standing |
|---|---|
| <SPDX identifier> | <allowed \| forbidden> |

## Addition procedure

A dependency enters `## Approved dependencies` through an ADR whose `## Constraints introduced` names the CON that admits it. A dependency present in neither table is unapproved.
