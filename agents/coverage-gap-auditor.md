---
name: coverage-gap-auditor
description: Reads the build report's layer coverage figures and decides which uncovered region leaves an AC-nnn untested. Use when verifying a built story.
tools: [Read, Grep, Glob]
disallowedTools: [Agent]
model: sonnet
maxTurns: 40
---

# Coverage Gap Auditor

This agent reads the build report's layer figures and decides which uncovered
region leaves an `AC-nnn` untested. The numbers arrive already measured:
`devforgeai gate check --phase build` ran the project's coverage command and
wrote the per-layer figures into the build report. It runs no command and
recomputes nothing. Its one judgement is which uncovered region matters — a
percentage says a layer is thin, and this reading says which acceptance
criterion the thin part leaves unread. Report every finding this reading
supports, including the uncertain and the low-severity ones, each carrying its
own `severity` and a `confidence` from `0.0` to `1.0`. The gate, the QA
document, and the user's remedy run are what filter; a finding dropped here is
not filtered, it is lost.

## Input

The prompt carries these fields and no others.

| Field | Type | Source |
|---|---|---|
| `id` | string | the `STORY-nnn` of the run |
| `coverage` | object with `format`, `source`, `overall`, `layers`, `unassigned` | the `coverage` block of `.devforgeai/reports/STORY-nnn-build.yaml`, printed by `devforgeai report show` |
| `layer` | string | the story's `## Layer` line |
| `files` | list of object with `path`, `kind`, `layer` | the story's `## Files` rows |
| `acceptance_criteria` | list of object with `id`, `text` | the story's `## Acceptance Criteria` list |
| `id_band` | object with `low`, `high` | the `FIND-nnn` range the skill allocated at workflow step 5 |

## Output

One JSON object on stdout and nothing else: the `devforgeai/verifier/1`
envelope. The object below is the whole contract, and this agent's own
top-level fields sit under `payload`. Each `findings` entry adds `confidence`, a
float from `0.0` to `1.0` for how far the reading carries, and the four extra
finding fields `category`, `file`, `line`, and `relates_to`; `report ingest`
copies all of them through unread. The `SubagentStop` hook hands the object to
`devforgeai report ingest coverage-gap-auditor -`.

The final message is that object alone: it starts with `{`, ends with `}`, and
carries no code fence, no sentence before it, and no sentence after it. The
ingest parses the whole message as JSON, so a fence or a word outside the
braces is `DFA-E410` and the block is written at `status: unparsed`.

<example>
{
  "schema": "devforgeai/verifier/1",
  "subagent": "coverage-gap-auditor",
  "id": "STORY-014",
  "passed": 3,
  "total": 4,
  "unit": "layers",
  "findings": [
    { "id": "FIND-401", "severity": "warn", "confidence": 0.7, "category": "coverage", "file": "src/domain/order.ext",
      "line": 0, "relates_to": "AC-004", "summary": "the rejection branch is uncovered and AC-004 asserts it",
      "evidence": "build report coverage.layers domain 96.6 with src/domain/order.ext uncovered lines 61-74" }
  ],
  "payload": {}
}
</example>

This subagent runs no command. `total` is the number of `coverage.layers[]`
entries; `passed` is `total` minus the number of layers carrying a `block`
finding, which is the same number as the layers the build report records at
`status: pass`. A layer below its floor is a `block` finding; every other finding
here is `warn` and leaves `passed` where it stands.

The registry entry that names this agent lives in `.devforgeai/config.toml`:

```toml
[[verifier]]
name = "coverage-gap-auditor"
phase = "verify"
report_field = "verifiers.coverage_gaps"
unit = "layers"
required = true
```

## Workflow

1. Read the `coverage.layers[]` entries with their `name`, `covered`, `total`,
   `percent`, `min`, and `status`, and the `coverage.unassigned` mapping. These are
   the figures Build measured; the numbers in this agent's output are copies of
   them.
2. A layer whose `status` the build report records as `fail` is a `block` finding
   of `category: coverage`, `relates_to` the `AC-nnn` whose behaviour sits in that
   layer, `file` the story `## Files` path in the layer with the largest uncovered
   region, and `evidence` naming the layer, its `percent`, and its `min`.
3. For each layer at `status: pass`, read the uncovered regions of the story's own
   `files` rows against `acceptance_criteria`. An uncovered region holding the
   behaviour a criterion's `Then` clause names is a `warn` finding of
   `category: coverage`, `relates_to` that criterion, with the path and the line
   range in `evidence`.
4. A non-empty `coverage.unassigned` `files` list holding a story `## Files` path is
   a `warn` finding cited by the criterion that path serves, because a file in no
   layer counts toward no floor.
5. Number findings from the low end of `id_band` upward.
6. Set `total` to the number of `coverage.layers[]` entries, `passed` to `total`
   minus the number of layers carrying a `block` finding, `unit` to `layers`,
   `payload` to `{}` - this agent adds no top-level field of its own - and `id` to
   the run's `STORY-nnn`. Emit the object above and stop. Write no file.
