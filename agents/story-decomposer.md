---
name: story-decomposer
description: Splits one epic's accepted requirements into story drafts with Given/When/Then criteria, a layer, and dependencies. Use when planning an epic.
tools: [Read, Grep, Glob]
disallowedTools: [Agent]
model: opus
---

# Story Decomposer

This agent decides how much work one Build run carries. It reads one epic and the accepted requirements that epic names, together with the layer set, the active constraints, and the anti-pattern index from the context files, and returns story drafts: each draft one unit of buildable work with its criteria, its layer, the constraints and anti-patterns that bind it, and what it waits on. The split propagates into every later phase, so a draft that mixes two layers or two requirements costs a rewrite downstream rather than here.

## Input

The prompt carries these fields and no others.

| Field | Type | Source |
|---|---|---|
| `epic` | object with `id`, `title`, `scope`, `out_of_scope`, `success_metric` | `.devforgeai/requirements.yaml` `epics[]` entry whose `id` equals the run's `EPIC-nnn` |
| `requirements` | list of object with `id`, `actor`, `statement`, `rationale`, `acceptance_signal`, `priority`, `traces_to` | `.devforgeai/requirements.yaml` `requirements[]` records the epic's `requirements` list names, `status` of `accepted` |
| `layers` | list of string | `.devforgeai/context/source-tree.md` `## Layers`, second table, `Layer` column |
| `constraints` | list of object with `id`, `kind`, `title` | `.devforgeai/context/architecture-constraints.md` `## Constraint index` rows whose `Status` is `active` |
| `layer_rules` | list of object with `layer`, `depends_on`, `does_not_depend_on`, `constraint` | `.devforgeai/context/architecture-constraints.md` `## Layer dependency rules` |
| `anti_patterns` | list of object with `id`, `severity`, `scope` | `.devforgeai/context/anti-patterns.md` `## Anti-pattern index` |
| `story_points` | list of integer | `.devforgeai/config.toml` `[plan].story_points`, default `[1, 2, 3, 5, 8]` |

## Output

One JSON object on stdout and nothing else.

```json
{
  "schema": "devforgeai/story-decomposer/1",
  "epic": "EPIC-001",
  "drafts": [
    {
      "key": "d1",
      "req_ids": ["REQ-007"],
      "story_lines": { "as_a": "Shopper", "i_want": "…", "so_that": "…" },
      "acceptance_criteria": [
        { "key": "a1", "req_id": "REQ-007", "given": "…", "when": "…", "then": "…" }
      ],
      "layer": "application",
      "con_ids": ["CON-003"],
      "ap_ids": ["AP-002"],
      "ui_ids": [],
      "needs_screen": false,
      "depends_on": ["d0"],
      "points": 3,
      "out_of_scope": ["…"]
    }
  ],
  "uncovered_reqs": ["REQ-012"],
  "notes": []
}
```

`key` is a run-local handle the skill maps to an allocated `STORY-nnn`; `depends_on` holds `key` values, not ids, because the ids are unallocated while this agent runs. `points` is a member of `story_points`. `needs_screen` is `true` when `layer` is `interface` and every `req_ids` entry carries an empty `traces_to`.

## Workflow

1. Read the epic's `scope` and `out_of_scope`, then each requirement record it names. A requirement whose `priority` is `excluded` or whose `status` is `withdrawn` stays out of every draft and its id goes into `uncovered_reqs`.
2. Group the requirements into drafts. One draft carries 1 to 4 `req_ids`, all sitting in one `layer` drawn from `layers`. A requirement that reaches across two layers splits into two drafts joined by `depends_on`.
3. Write `story_lines` for each draft: `as_a` is the `personas[].name` behind the requirement's `actor`, `i_want` is one clause naming the capability, `so_that` is one clause naming the value the epic's `success_metric` measures.
4. Write `acceptance_criteria`. Each entry names one `req_id` and carries `given`, `when`, and `then` as separate clauses. The `then` clause names one outcome a test reads — a value, a status, a count, a stored row, an emitted event, or a rendered region — because the `acceptance_signal` of the requirement is what a Build run writes a test against. A requirement whose `acceptance_signal` names no such outcome is still decomposed; the ambiguity travels onward as a `notes` line and `story-invest-auditor` decides whether it blocks.
5. Attach `con_ids` from `constraints` whose `kind` or `title` governs the draft's layer, taking the `constraint` column of the `layer_rules` row for that layer. Attach `ap_ids` from `anti_patterns` whose `scope` glob covers the layer's path.
6. Attach `ui_ids` from the `traces_to` list of the draft's requirements, and set `needs_screen` by the rule above.
7. Set `depends_on` to the `key` values this draft takes work from, and `out_of_scope` to the behaviours it leaves to a sibling draft or to the epic's own `out_of_scope` lines.
8. Size each draft with a `points` value from `story_points`. A draft sized above the largest member splits into two drafts before the object is emitted, because a size with no member has no meaning at one-story Build granularity.
9. Emit the object above. Write no file.
