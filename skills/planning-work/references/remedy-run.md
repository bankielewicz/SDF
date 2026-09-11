# The remedy run

Read this before step R1. `/plan EPIC-nnn --remedy AC-nnn,AC-nnn` arrives from Build or from Verify: a criterion this phase wrote could not be built or tested against. The run re-opens the cited criteria and writes no new story. Steps R1 to R4 replace steps 2 to 13.

The `--remedy` id set is closed at the `AC` prefix. Build cites an `AC-nnn` directly; Verify's `FIND-nnn` names the criterion, and the criterion is the id the remedy re-opens.

## R1 · Locate the criteria

For each id after `--remedy`, find the story whose `## Acceptance Criteria` list holds that line, then read the report that cited it:

- `devforgeai report show STORY-nnn build` for a Build send-back
- `devforgeai report show STORY-nnn verify` for a Verify send-back

Pair each id with its `findings[]` entry, producing one `(STORY-nnn, AC-nnn, finding summary)` triple. Exit 1 from `report show` means there is no report of that kind and the run stops. A cited `AC-nnn` appearing in no story file stops the run with one `Blocked` line naming the id.

## R2 · Triage

Invoke `spec-gap-triager` once with every triple. Alongside the triples it takes the criterion text, the `REQ-nnn` whose `Covered by` cell holds each criterion with its `statement` and `acceptance_signal`, the active `CON-nnn` rows whose `Binds` value matches the story's `## Layer` or a `Path` in its `## Files`, and the `UI-nnn` ids in the story's `## Interface`.

It returns one `disposition` per triple from a closed set of four, plus a `cited_id`, a one-sentence `rationale`, and for `rewrite_ac` a `replacement` line.

## R3 · Act on each disposition

| `disposition` | What the run does | What it leaves alone |
|---|---|---|
| `rewrite_ac` | Replaces that one criterion line in its `STORY-nnn.md` with the `replacement` text | Every other byte of the file, and all of `sprint.yaml` except the story's `status` |
| `send_to_design` | Invokes the `design` skill through the Skill tool with `UI-nnn --remedy AC-nnn,...` and rewrites section 6 from the returned spec | Sections the returned spec does not touch |
| `send_back_discover` | Writes no file; the `REQ-nnn` in `cited_id` goes on the handoff `Found` line | Every document |
| `send_back_constitute` | Writes no file; the `CON-nnn` in `cited_id` goes on the handoff `Found` line | Every document |

An edit that breaks the `Given .+ When .+ Then .+` grammar comes back from the PostToolUse `devforgeai doc validate` as `DFA-E230`; rewrite the line.

`send_to_design` is not a send-back. Design is cross-cutting, holds no gate, and returns on the `Then` line as `/plan EPIC-nnn --resume`.

## R4 · Reset and close

Write the frontmatter `status` value `ready` into each story edited at R3, and the same value into that story's entry in `sprint.yaml` `stories[]`. No other key of `sprint.yaml` changes — not `capacity`, not `order`, not `points`, not `deferred`. Then run `devforgeai phase set plan --id SPRINT-nnn --epic <EPIC-nnn>` with the `id` and the `epic` key the file on disk carries. `--epic` repeating the file's own value is what the call accepts; a differing value is `DFA-E013`.

The Stop hook prints the handoff. Its `Next` line reads `/build STORY-nnn --resume` when every disposition was `rewrite_ac` or `send_to_design`, and the send-back form of `references/send-back.md` when any disposition routed upstream.

## What a remedy run does not do

- It allocates no `STORY-nnn`, no `AC-nnn`, and no `SPRINT-nnn`.
- It invokes `story-decomposer`, `story-file-set-planner`, `story-invest-auditor`, and `sprint-sequencer` not at all; `spec-gap-triager` runs in no full and no resume run.
- It rewrites no requirement, no context file, and no ADR.
- It writes no new story file and adds no row to `sprint.yaml` `stories[]` or `deferred[]`.

## The resume run

`/plan EPIC-nnn --resume` is the other returning form, and it is a full run re-entered at step 2. It re-reads `requirements.yaml` and the six context files from the preamble's stdout, so a repaired `REQ-nnn` or a new `CON-nnn` is the text the run works from. Step 4 reuses the `SPRINT-nnn` in `state.toml` `[active].plan` and allocates none. Every `STORY-nnn.md` already on disk at `status: draft` is kept, keeps its allocated ids, and is re-audited at step 9. `sprint.yaml`, absent after a send-back, is written at step 12.
