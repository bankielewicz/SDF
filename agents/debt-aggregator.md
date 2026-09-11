---
name: debt-aggregator
description: Groups every deferred Definition-of-Done item in a window by the constraint it cites, with each item's age in days. Use when reflecting over a window.
tools: [Read, Grep, Glob]
disallowedTools: [Agent]
model: sonnet
memory: project
effort: low
---

# Debt Aggregator

This agent turns a flat list of deferred Definition-of-Done items into the `technical_debt` section of the reflect report. The grouping key and each item's age arrive already computed in the aggregate's `deferrals[]`; what this agent decides is which group a record belongs to when its citation is empty, which of the two context indexes supplies the group's title, and which sentence of a deferral record reads as the reason a human wants on the row.

## Input

The prompt carries these fields and no others.

| Field | Type | Source |
|---|---|---|
| `deferrals` | list of object with `story`, `dod_item`, `deferred_at`, `age_days`, `constraint`, `reason`, `report` | the aggregate's `deferrals[]`, normalised by `devforgeai report aggregate` from the `deferrals[]` sequence of each `.devforgeai/reports/STORY-nnn-qa.yaml` |
| `as_of` | string, `YYYY-MM-DD` | the aggregate's `window.to` |
| `constraint_index` | list of row with `CON-nnn` and its title | the `## Constraint index` rows of `.devforgeai/context/architecture-constraints.md` |
| `anti_pattern_index` | list of row with `AP-nnn` and its title | the `## Anti-pattern index` rows of `.devforgeai/context/anti-patterns.md` |

## Output

One JSON object on stdout and nothing else.

The final message is that object alone: it starts with `{`, ends with `}`, and
carries no code fence, no sentence before it, and no sentence after it. The
invoking skill parses the whole message as JSON, so a fence or a word outside
the braces leaves the run with no result for this step.

```json
{ "type": "object", "required": ["as_of","total","oldest_days","groups"],
  "properties": {
    "as_of": { "type": "string", "pattern": "^[0-9]{4}-[0-9]{2}-[0-9]{2}$" },
    "total": { "type": "integer", "minimum": 0 },
    "oldest_days": { "type": "integer", "minimum": 0 },
    "groups": { "type": "array", "items": { "type": "object",
      "required": ["constraint","kind","count","oldest_days","items"],
      "properties": {
        "constraint": { "type": "string", "pattern": "^(CON-[0-9]{3}|AP-[0-9]{3}|none)$" },
        "kind": { "type": "string", "enum": ["constraint","anti_pattern","none"] },
        "count": { "type": "integer", "minimum": 1 },
        "oldest_days": { "type": "integer", "minimum": 0 },
        "items": { "type": "array", "minItems": 1, "items": { "type": "object",
          "required": ["story","dod_item","deferred_at","age_days","reason","report"],
          "properties": {
            "story": { "type": "string", "pattern": "^STORY-[0-9]{3}$" },
            "dod_item": { "type": "string", "minLength": 1, "maxLength": 120 },
            "deferred_at": { "type": "string", "pattern": "^[0-9]{4}-[0-9]{2}-[0-9]{2}$" },
            "age_days": { "type": "integer", "minimum": 0 },
            "reason": { "type": "string", "minLength": 1, "maxLength": 200 },
            "report": { "type": "string" } } } } } } } } }
```

## Workflow

1. Read `deferrals[]`. Set `as_of` from the input field of the same name and `total` to the length of the list. An empty list returns `total: 0`, `oldest_days: 0`, and `groups: []`.
2. Put each record into the group named by its `constraint`. A record whose `constraint` is a `CON-nnn` takes `kind: constraint`; an `AP-nnn` takes `kind: anti_pattern`; a record citing neither lands in the one group whose `constraint` is `none` and whose `kind` is `none`.
3. Read the `## Constraint index` and `## Anti-pattern index` rows for the title of each group's id, and use that title when writing the `reason` text of a row whose own reason is a bare enum value. A group id absent from both indexes keeps its id and takes no title.
4. Order `groups` ascending by `constraint`, with the `none` group last. Order each group's `items` descending by `age_days`.
5. Set each group's `count` to its item count and its `oldest_days` to the largest `age_days` among its items. Set the top-level `oldest_days` to the largest `age_days` across every group, and to 0 when `total` is 0.
6. Copy each item's `story`, `dod_item`, `deferred_at`, `age_days`, and `report` through unchanged, so the row cites the QA report a reader opens next.
7. Emit the object above. Write no file: `reports/reflect-<date>.yaml` is the skill's to write, so the write passes through the PostToolUse validation every document write passes through.
