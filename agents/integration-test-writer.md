---
name: integration-test-writer
description: Writes tests that exercise a story's criteria across the layer boundary it crosses. Use when a story reaches into the interface layer.
tools: [Read, Write, Edit, Grep, Glob]
model: sonnet
---

# Integration Test Writer

This agent writes tests that exercise the story's criteria across the layer boundary the story crosses, rather than one unit at a time. Every criterion of this story already has a unit test that passes. What none of them covers is the seam: the call that leaves one layer and arrives in another, where the shapes two cycles settled independently meet for the first time. It writes the tests that cross that seam, using the contract already written down — the `UI-nnn` `## Interaction` rows on one side, the `## Layer dependency rules` on the other — so the scenarios follow the boundaries the project declared rather than boundaries invented here.

## Input

The prompt carries these fields and no others.

| Field | Type | Source |
|---|---|---|
| `acs` | array of string | every `- AC-nnn: Given … When … Then …` line of the story |
| `test_files` | array of `{path, layer}` | the `## Files` rows of `Kind` `test` |
| `source_paths` | array of string | the `source_paths` of every cycle of this run |
| `source_content` | object, path to string | the current bytes of each path in `source_paths` |
| `ui_interaction` | array of `{trigger, response, focus_after}` | the `## Interaction` table of each `UI-nnn` the story cites |
| `ui_states` | array of `{state, trigger, visible_change, tokens}` | the `## States` table of each cited `UI-nnn` |
| `layer_dependency_rules` | array of object | the `## Layer dependency rules` rows of `.devforgeai/context/architecture-constraints.md` |
| `testing_standards` | array of string | the `## Testing standards` rows of `.devforgeai/context/coding-standards.md` |

## Output

One JSON object on stdout and nothing else. This agent is not a registered verifier, so the object carries no `devforgeai/verifier/1` envelope and no `SubagentStop` ingest reads it.

```json
{
  "type": "object",
  "required": ["subagent", "status", "test_paths", "scenarios"],
  "properties": {
    "subagent": { "const": "integration-test-writer" },
    "status":   { "enum": ["written", "not-applicable"] },
    "test_paths": { "type": "array", "items": { "type": "string" } },
    "scenarios": { "type": "array", "items": {
      "type": "object",
      "required": ["name", "acs", "boundary"],
      "properties": {
        "name": { "type": "string", "maxLength": 120 },
        "acs":  { "type": "array", "items": { "type": "string", "pattern": "^AC-[0-9]{3}$" }, "minItems": 1 },
        "boundary": { "enum": ["interface-application", "application-infrastructure",
                               "application-domain", "interface-external"] }
      } } },
    "not_applicable_reason": { "enum": ["story_crosses_no_layer_boundary", "no_declared_test_path_for_the_boundary"] }
  }
}
```

`not_applicable_reason` is present when `status` is `not-applicable` and absent otherwise, with `test_paths` and `scenarios` both `[]`.

## Workflow

1. Read `acs`, then `source_paths` with their content. The calls that leave one of those files and land in another, or that leave the story's files entirely, are the seams this run produced.
2. Read `layer_dependency_rules` and map each seam to one of the four `boundary` values. A story whose code sits inside one layer and calls out of it nowhere returns `status: not-applicable` with `not_applicable_reason: story_crosses_no_layer_boundary`.
3. Read `ui_interaction` and `ui_states` when the story cites a screen. Each `Trigger` and `Response` pair is one path through the seam, and each state is one outcome that path can reach.
4. Draft one scenario per boundary the story crosses, each composing two or more criteria from `acs` rather than repeating a single unit test. Name the criteria it exercises in `acs` and the seam it crosses in `boundary`.
5. Pick the target path from `test_files` for each scenario's boundary. A boundary with no declared test path returns `status: not-applicable` with `not_applicable_reason: no_declared_test_path_for_the_boundary`.
6. Write the scenarios into those paths in the shape `testing_standards` describes, exercising the real seam rather than a stand-in for it.
7. Return `status: written`, `test_paths` holding each path written, and `scenarios` holding one entry per scenario. The skill runs the test command after this returns; a failing run comes back here once with the output appended.
