# Spec mode

Read before workflow step 10. Spec mode turns one `STORY-nnn` or `EPIC-nnn` and the `REQ-nnn` records it cites into one `.devforgeai/ui-specs/UI-nnn.md` per screen or component the story needs.

## The coverage rule

`requirement-coverage-auditor` at step 11 applies two rules, and the send-back cites ids rather than impressions because of them:

- A screen is covered when at least one `REQ-nnn` in the story's `consumes` names it in `acceptance_signal` or in an `AC-nnn` row.
- A flow is covered when its `FLOW-nnn` equals some `requirements[].source`.

The audit returns the `devforgeai/verifier/1` envelope. `passed` and `total` sit at the top level; everything else sits under `payload`. `payload.screens_without_req[]` carries one `{name, evidence}` per uncovered screen; `payload.flows_without_req[]` carries one `{flow_id, evidence}` per uncovered flow. `payload.covered` and `total` fill the handoff's `Verified` line.

`.devforgeai/explore/brief.md` absent is the shape Discover entry point B leaves — `requirements.yaml` carries `consumes: []` and no brief exists. The audit then returns `payload.flows_without_req` of `[]` and runs on screens alone.

## The nine sections

`templates/ui-spec.md` is the shape. In this order, with these headings verbatim:

| # | Heading | Content |
|---|---|---|
| 1 | `## Purpose` | One sentence. What the person on this screen is doing and what leaves it done. |
| 2 | `## Requirements` | Table `REQ \| Statement \| Satisfied by`. One row per `REQ-nnn` in `consumes`, 1 to 8 rows. `Satisfied by` names a `Region` value from `## Anatomy`. |
| 3 | `## Anatomy` | Table `Region \| Element \| Content source \| Tokens`. 3 to 20 rows. `Content source` is a `seed-data.json` entity name, a `REQ-nnn`, or `static`. `Tokens` is a space-separated list of `TOKEN-<name>` values. |
| 4 | `## States` | Table `State \| Trigger \| Visible change \| Tokens`. One row per state the screen has, drawn from the closed set `default`, `loading`, `empty`, `partial`, `error`, `success`, `disabled`, `read-only`. `default` is present in every file. |
| 5 | `## Breakpoints` | Table `Name \| Min width \| Layout \| Changes from previous`. Exactly four rows: `sm` `320px`, `md` `768px`, `lg` `1024px`, `xl` `1440px`. |
| 6 | `## Interaction` | Table `Trigger \| Response \| Focus after`. One row per pointer, key, or form event the screen answers, 1 to 20 rows. `Focus after` names a `Region` value or `unchanged`. |
| 7 | `## Accessibility` | Table `Check \| Requirement \| Evidence`. Exactly eight rows, in this order: `Landmark`, `Heading order`, `Name`, `Role`, `Keyboard path`, `Focus visible`, `Contrast`, `Motion`. `Evidence` names a `Region`, a `TOKEN-<name>`, or a ratio. |
| 8 | `## Tokens used` | Table `Token \| Group \| Where`. One row per distinct `TOKEN-<name>` appearing in sections 3, 4, and 7. `Where` names one `Region` or `State`. |
| 9 | `## Out of scope` | Unnumbered lines, 0 to 8, each one sentence naming a behaviour this screen does not carry and, when one exists, the `UI-nnn` that does. |

The four breakpoint widths are fixed by the template rather than drawn from tokens: `design lint` inspects colour and type properties, not media queries, so a breakpoint in `tokens.json` would be a value nothing resolves against. They are also the set Build's `frontend-developer` implements against, which keeps Build reading one list.

The states, the keyboard path, and the eight accessibility rows are the content Plan sizes a story from and Build implements against, and a thin one costs a Verify finding. `## States` is drawn from the screen's real behaviour, not from the template's four example rows.

## The token-only rule

Every colour and every type value in the body is a `TOKEN-<name>` reference. A hex triple, an `rgb(`, an `hsl(`, a CSS named colour, a `px` or `rem` font size, and a font family name do not appear in the body. The `Min width` column of `## Breakpoints` and a contrast ratio in `## Accessibility` `Evidence` are the values that are numbers rather than tokens.

Every `TOKEN-<name>` in the file flattens to a leaf `tokens.json` defines. `devforgeai design lint --tokens` at step 14 is what resolves them: exit 1 on `DFA-E244` names a token that `tokens.json` does not define, and the row it names is rewritten with a defined token.

Frontmatter carries the seven document keys in document order, with `consumes` listing the `STORY-nnn` or `EPIC-nnn`, every `REQ-nnn` in `## Requirements`, and the `PERSONA-nnn` those requirements name. A token name goes in no `consumes` list: `TOKEN-<name>` is a reference, not an id.

## Writing the screens

Step 13 runs one `ui-spec-writer` invocation per screen, in parallel across screens, because each file is written from one screen's own inputs and shares no state with the others.

Each invocation receives the screen with its allocated `UI-nnn`, the `REQ-nnn` records it realizes, the `PERSONA-nnn` goal, the flattened token names of `tokens.json`, the matching mockup files under `.devforgeai/explore/mockups/`, and the story's Figma node URL when the story carries one.

The agent invokes the built-in `frontend-design:frontend-design` skill for component composition judgment. It invokes no Figma skill: the skill runs `figma:figma-design-to-code` itself at step 13 and passes what it returns as the `figma_context` field, because a skill cannot be invoked from inside a subagent's turn. A non-empty `figma_context` cites the node in `## Anatomy` `Content source`; a `figma_context` of `""` — no node URL, the plugin absent, or an authentication error — cites the mockup path or the `REQ-nnn` instead, and the file is otherwise unchanged.

Files land at `status: draft` and move to `approved` after step 14 exits 0.

## Remedy

`/design UI-nnn --remedy <ids>` arrives from Plan as `/plan <SPRINT-nnn>`'s send-back and from Build as `/build <STORY-nnn>`'s. The ids come from the `findings[]` of `.devforgeai/reports/SPRINT-nnn-plan.yaml` or `.devforgeai/reports/STORY-nnn-build.yaml`, and each kind names its own rewrite:

| Cited id | What it names | What is rewritten |
|---|---|---|
| `AC-nnn` | a criterion the spec does not support | `## States`, `## Interaction`, `## Accessibility` |
| `TOKEN-<name>` | a token the spec references and `tokens.json` does not define | `## Anatomy`, `## Tokens used` |
| `UI-nnn` | a spec a story references and `.devforgeai/ui-specs/` does not hold | the whole file, written as step 13 writes one |

Sections the cited ids do not name keep every byte, which is what lets Plan and Build read the rest of the file unchanged across a remedy. The file goes to `status: draft` while it is rewritten and back to `approved` after step 14.

A cited `AC-nnn` the story does not hold adds one `open_questions` line naming the id, and the ids that did resolve are rewritten in the same run.

## The blocked path

`.devforgeai/brand/tokens.json` absent stops the mode at step 10: no `UI-nnn.md` is written, and the handoff carries `Blocked   you: run /design --brand <EPIC-nnn> first`. A spec written before the brand kit would reference names nothing defines and would fail `design lint --tokens` at step 14.
