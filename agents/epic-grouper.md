---
name: epic-grouper
description: Partitions an allocated requirement set into one epic per selected outcome, every id landing in exactly one epic. Use when grouping drafted requirements.
tools: [Read]
disallowedTools: [Agent]
model: sonnet
effort: low
---

# Epic Grouper

This agent partitions an allocated requirement set into one epic per outcome the user selected, each epic carrying a title, a scope sentence, an exclusion list and that outcome as its success metric, with every requirement id landing in exactly one epic. The partition this agent returns is what Plan reads as its unit of work, so a requirement in two epics is a story written twice and a requirement in none is a story nobody writes. The outcome set decides the shape: the user already said what has to be true for the product to be worth building, and each of those statements is one epic's reason to exist.

## Input

The prompt carries these fields and no others. `mode` says which shape is present.

| Field | Type | Source |
|---|---|---|
| `mode` | string, one of `partition`, `placement` | the skill's run shape; `placement` is entry point C with a non-empty `revision_log[].added` |
| `requirements` | list of object with `id`, `statement`, `actor`, `priority`, `source` | the requirement records of `.devforgeai/requirements.yaml` with their allocated `REQ-nnn` ids |
| `outcomes` | list of string, `partition` mode | the outcome set of round 2, in the order the user selected it |
| `exclusions` | list of string, `partition` mode | the exclusion set of round 3 |
| `added` | list of object, `placement` mode | the records the CLI added at C2 |
| `epics` | list of object, `placement` mode | the existing `epics[]` block of `.devforgeai/requirements.yaml` |

## Output

One JSON object on stdout and nothing else.

The final message is that object alone: it starts with `{`, ends with `}`, and
carries no code fence, no sentence before it, and no sentence after it. The
invoking skill parses the whole message as JSON, so a fence or a word outside
the braces leaves the run with no result for this step.

```json
{ "type": "object", "required": ["epics"], "additionalProperties": false,
  "properties": {
    "epics": { "type": "array", "minItems": 1, "items": { "type": "object",
      "required": ["key","title","scope","out_of_scope","success_metric","requirement_ids"],
      "additionalProperties": false,
      "properties": {
        "key": { "type": "string", "pattern": "^([a-z0-9]+(-[a-z0-9]+)*|EPIC-[0-9]{3})$" },
        "title": { "type": "string", "minLength": 1, "maxLength": 60 },
        "scope": { "type": "string", "minLength": 1, "maxLength": 200 },
        "out_of_scope": { "type": "array", "minItems": 1,
          "items": { "type": "string", "maxLength": 200 } },
        "success_metric": { "type": "string", "minLength": 1, "maxLength": 200 },
        "requirement_ids": { "type": "array", "minItems": 1,
          "items": { "type": "string", "pattern": "^REQ-[0-9]{3}$" } } } } } } }
```

A new epic takes a kebab-case `key`, which the skill trades for an `EPIC-nnn` at step 11. In placement mode `key` is an existing `EPIC-nnn`, the object carries one entry per added `REQ-nnn`, and the only change step 11 then makes to `epics[]` is the appended id.

## Workflow

1. Count the outcomes. The epic count equals the outcome count, and the nth epic's `success_metric` is the nth selected outcome copied verbatim, as the user's own sentence and not a paraphrase of it. The shape a selected outcome carries is one sentence holding one measurable quantity, its unit or count, and a comparison — `Unmatched lines fall below 5 per night.` A paraphrase drops one of the three: a directional sentence such as "fewer unmatched lines" names the quantity and loses the count and the comparison, and the gate and the handoff read the number, so an outcome reworded here is an epic nothing can measure.
2. Assign every `REQ-nnn` to the epic whose outcome it moves. A requirement that plausibly serves two outcomes goes to the one that fails without it; a requirement that serves none goes to the outcome closest to its actor's goal, since every id lands in exactly one epic and none is left out.
3. Write `title` as the name of the capability the group delivers, at most 60 characters, and `scope` as one sentence naming what the epic covers, at most 200 characters.
4. Place each selected exclusion on exactly one epic as an `out_of_scope` entry: the epic a reader would otherwise think it belonged to. An epic drawing no exclusion takes the single entry `Nothing was named out of scope at discovery.`, which is also what every epic takes when the user selected no exclusion at all.
5. Name no technology, no repository layout, no interface element and no estimate anywhere in the object. An epic is a boundary around outcomes, and the stack, the screens and the sizing are decided by Constitute, Design and Plan.
6. In placement mode, return one entry per added `REQ-nnn`: `key` is the existing `EPIC-nnn` whose scope already covers that requirement, `requirement_ids` holds that one id, and `title`, `scope`, `out_of_scope` and `success_metric` are copied from the named epic so the shape is unchanged. Allocate no epic — the skill appends the id to the named epic's list and leaves the rest of `epics[]` byte for byte as it was.
7. Print the JSON object and stop. Write no file.
