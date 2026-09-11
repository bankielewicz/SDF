# The decision record

Read at step 10, while writing `.devforgeai/explore/decision.yaml` from `templates/decision.yaml`.

## Fields

```yaml
schema: devforgeai/explore-decision/1
id: IDEA-001
phase: explore
status: recorded
produced_by: exploring-ideas
consumes: []
open_questions: []
decision: promote
decided_on: 2026-09-10
reason: >-
  Three of four people shown the FLOW-001 mockup asked when they could pay for it,
  and the two competitors found charge per seat for the manual version of the same flow.
revisit_on: null
elapsed_days: 4
remedied_flows: []
carry_forward: []
```

| Field | Value |
|---|---|
| `status` | `recorded`, its one value. The file is written once per run and overwritten on a remedy run. |
| `decision` | one of `kill`, `park`, `promote`, from the step 9 answer |
| `decided_on` | `YYYY-MM-DD`, the local date the user answered step 9 |
| `reason` | one sentence, 15 to 40 words, naming the evidence that produced the decision |
| `revisit_on` | `YYYY-MM-DD` when `decision` is `park`, later than `decided_on`; `null` on `kill` and `promote` |
| `elapsed_days` | integer, whole days from `[explore].started_at` in `state.toml` to `decided_on` |
| `remedied_flows` | the `FLOW-nnn` ids the latest remedy run rewrote; `[]` on a fresh run |
| `carry_forward` | `[]` on `kill` and `park`; the seven entries below on `promote` |

`consumes` is `[]` here as it is in the brief: this phase reads no document another phase produced, and the Discover report a remedy run reads is keyed by the same `IDEA-nnn`, so listing it would be a self-reference.

## `carry_forward` on a promote

Exactly these seven entries, in this order:

```yaml
carry_forward:
  - path: .devforgeai/explore/brief.md
    sections: [Problem statement, Target user]
    becomes: discovery brief and first persona
    consumer: discovering-requirements
  - path: .devforgeai/explore/brief.md
    sections: [Core flows]
    becomes: epic candidates with draft acceptance criteria
    consumer: discovering-requirements
  - path: .devforgeai/explore/brief.md
    sections: [Non-goals]
    becomes: architecture-constraints.md and anti-patterns.md entries
    consumer: establishing-context
  - path: .devforgeai/explore/brief.md
    sections: [Competitor scan, Technology scan]
    becomes: tech stack candidates and ADR context
    consumer: establishing-context
  - path: .devforgeai/explore/mockups/
    sections: []
    becomes: UI specifications attached to stories
    consumer: designing-interfaces
  - path: .devforgeai/explore/seed-data.json
    sections: []
    becomes: test fixtures and example rows
    consumer: implementing-stories
  - path: .devforgeai/explore/decision.yaml
    sections: []
    becomes: ADR-000, the reason this project exists
    consumer: establishing-context
```

## What each answer leaves behind

| Answer | On disk afterwards | `.explore-prototype/` |
|---|---|---|
| `kill` | the brief at `status: decided`, the decision record, the seed data, the mockups | deleted by step 11 |
| `park` | the same, plus `revisit_on` and the open questions | left in place |
| `promote` | the same, plus the seven `carry_forward` entries Discover and Constitute read | deleted by step 11 |

## After the write

Move the brief frontmatter to `status: decided`, then run step 11. The PostToolUse hook runs `doc validate` on the write and its result reaches the model afterwards as `hookSpecificOutput.additionalContext`, naming the key that did not parse; rewrite that key. The brief and the decision record share the id `IDEA-nnn` under two different `schema` values, which `doc validate` treats as two documents rather than a duplicate id.
