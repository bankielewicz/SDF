# Explore questions

The user questions this phase asks, with their text, headers, and option labels fixed. The skill asks
every one of them; no subagent does, because a subagent has no `AskUserQuestion` tool.

## Workflow step 2b — holders

`<label>` and `<description>` come from `candidate_segments` in the order `idea-interrogator` returned
them at step 2a.

```
AskUserQuestion(questions=[{
  "question": "<IDEA-nnn> · <slug>: who holds this problem today?",
  "header": "Holders",
  "multiSelect": true,
  "options": [
    { "label": "<candidate_segments[0].label>", "description": "<candidate_segments[0].description>" },
    { "label": "<candidate_segments[1].label>", "description": "<candidate_segments[1].description>" },
    { "label": "<candidate_segments[2].label>", "description": "<candidate_segments[2].description>" },
    { "label": "Someone else",                  "description": "A segment none of the options names; type it in Other" }
  ]
}])
```

One round. The selected labels travel to `idea-interrogator` at step 2c as `selected_segments`, and the
`holders[]` it returns is what step 3 searches on and step 4 draws flows from.
