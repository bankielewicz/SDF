# Send-back, and the INVEST split

Read this at step 10 when a `story-invest-auditor` finding carries `severity: block`. It carries which INVEST property is decided where, the four conditions that leave this phase upstream, and the table of send-backs this phase receives.

## The INVEST split

Conventions §1 rule 1 keeps every mechanical check in the CLI. What is left for a judgment model is meaning: whether a story stands alone, whether its size matches one Build run, and whether a sentence says one thing or two.

| Property | Decided by | Rule |
|---|---|---|
| Every requirement of the epic is covered by at least one story | `devforgeai story validate --scope sprint` | `DFA-E235` |
| Every acceptance criterion references a requirement that exists | `devforgeai story validate` | `DFA-E236` when the id appears in no `Covered by` cell, `DFA-E231` when the cell's `REQ-nnn` is absent from `requirements.yaml` |
| Every acceptance criterion references at most one requirement | `devforgeai story validate` | `DFA-E245` |
| Every acceptance criterion has a testable predicate | `devforgeai story validate` | `DFA-E230`, the `Given … When … Then …` grammar |
| No dependency cycle | `devforgeai story validate` | `DFA-E232` |
| Declared file sets do not overlap inside `stories[]` | `devforgeai story validate --scope sprint` | `DFA-E237` |
| Every `UI-nnn` cited exists | `devforgeai story validate` | `DFA-E238` |
| Every story listed in the sprint has a file | `devforgeai story validate --scope sprint` | `DFA-E233` |
| Every story has at least one criterion | `devforgeai story validate` | `DFA-E234` |
| Independence: the story delivers value with its dependencies met and nothing further | `story-invest-auditor` | `severity: warn`, cited by `STORY-nnn` |
| Size: the story fits one layer and one Build run at its point value | `story-invest-auditor` | `severity: warn`, cited by `STORY-nnn` |
| Testability of wording: the `Then` clause names one outcome a test reads | `story-invest-auditor` | `severity: warn`, cited by `STORY-nnn` |
| The requirement admits two incompatible readings | `story-invest-auditor` | `severity: block`, cited by `REQ-nnn` |
| The constraint set is silent or self-opposed on a case the story meets | `story-invest-auditor` | `severity: block`, cited by `CON-nnn` |

A `warn` finding is repaired inside the run at step 10 and sets no gate. A `block` finding lowers `passed` below `total`, fails the gate's `plan-invest` check, and produces the send-back below.

## To Discover

| Condition | Detected by | IDs cited | Handoff `Next` |
|---|---|---|---|
| SB-1 · A requirement's `statement` admits two incompatible readings, each yielding a different `Then` clause | `story-invest-auditor` finding, `severity: block`, `id` prefix `REQ` | the `REQ-nnn` of each such finding | `/discover IDEA-nnn --remedy REQ-nnn,REQ-nnn` |
| SB-2 · A requirement's `acceptance_signal` names no outcome a test reads | `story-invest-auditor` finding, `severity: block`, `id` prefix `REQ` | the `REQ-nnn` of each such finding | `/discover IDEA-nnn --remedy REQ-nnn,REQ-nnn` |

`IDEA-nnn` is the top-level `id` of `.devforgeai/requirements.yaml`, which the preamble printed. It is not the `EPIC-nnn` the run took as `$1`: Discover re-opens the document by its own id and records the epic itself. The return trip is `/plan EPIC-nnn --resume`.

## To Constitute

| Condition | Detected by | IDs cited | Handoff `Next` |
|---|---|---|---|
| SB-3 · The constraint set decides no case the story meets, in a layer the `## Layer dependency rules` table covers | `story-invest-auditor` finding, `severity: block`, `id` prefix `CON` | the `CON-nnn` in the `Constraint` column of that layer's row | `/constitute IDEA-nnn --remedy CON-nnn` |
| SB-4 · Two active constraints bind one path set in opposed directions | `story-invest-auditor` finding, `severity: block`, `id` prefix `CON` | both `CON-nnn` | `/constitute IDEA-nnn --remedy CON-nnn,CON-nnn` |

SB-3 cites an existing id because `architecture-constraints.md` `## Layer dependency rules` carries one `CON-nnn` per layer row, so the constraint that governs the layer and decides no case the story meets already has a name. The return trip is `/plan EPIC-nnn --resume`.

## How the target is resolved

`devforgeai gate check --phase plan` reads the prefixes of the blocking `findings[].id` values through the fixed map `REQ` to `discover`, `CON` to `constitute`, `AP` to `constitute`, `UI` to `design`, and falls back to the gate's static `send_back_to` of `discover` when no blocking finding is present. Blocking findings of two prefixes resolve to `constitute`, because a missing constraint is repaired before the requirement it would have decided.

On either target the stories written so far stay on disk at `status: draft`, `sprint.yaml` is not written, `requirements.yaml` and the six context files keep every byte, and `state.toml` keeps `current_phase` of `plan`.

## To Design, which is not a send-back

A story citing a `UI-nnn` that `.devforgeai/ui-specs/` does not hold, or a `UI-nnn` whose `## States` list carries no state an `AC-nnn` asserts, is repaired inside the run by invoking the `design` skill through the Skill tool with `UI-nnn --remedy AC-nnn,...`. Design is cross-cutting and holds no gate, so the trip sets no phase and the `Then` line carries `/plan EPIC-nnn --resume`.

## Send-backs this phase receives

| From | Command | Effect |
|---|---|---|
| Build | `/plan EPIC-nnn --remedy AC-nnn,...` | The criterion is untestable as written. R2 triages it; `rewrite_ac` edits the one criterion line in its story and leaves every other byte of the file and all of `sprint.yaml` unchanged; the story returns to `status: ready`; `Next` is `/build STORY-nnn --resume` |
| Verify | `/plan EPIC-nnn --remedy AC-nnn,...` | A `FIND-nnn` names a gap between the criterion and what the story specifies. R2 triages it by the same four dispositions; `Next` is `/build STORY-nnn --resume` when the criterion was rewritten, and the send-back form above when the gap is upstream |
| Design | `/plan EPIC-nnn --resume` | Design's remedy run rewrote the cited `UI-nnn`; step 6 re-reads it and step 8 rewrites the story's `## Interface` |
| Discover | `/plan EPIC-nnn --resume` | A cited `REQ-nnn` was re-opened and its `statement` or `acceptance_signal` rewritten; the resume workflow re-enters at step 2 |
| Constitute | `/plan EPIC-nnn --resume` | A cited `CON-nnn` was superseded or a replacement allocated; the resume workflow re-enters at step 2. Constitute's outcome where the constraint stands arrives on the same line, and the run rewrites the stories that assumed otherwise |
