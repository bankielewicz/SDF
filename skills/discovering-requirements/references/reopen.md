# Re-opening an accepted set

Read at steps C1, C2, 9, 10 and 11, when `--remedy` is in `$ARGUMENTS`.

## Where a re-open comes from

Three phases downstream cite ids in an accepted requirement set, and each arrives as one line the user types from the handoff that sent it:

| From | Condition | Arrives as | `--from` |
|---|---|---|---|
| Plan | a `REQ-nnn` cannot be split into a testable `STORY-nnn` | `/discover IDEA-004 --remedy REQ-007,REQ-011` | `plan` |
| Constitute | a `CON-nnn` constraint makes a `REQ-nnn` unachievable | `/discover IDEA-004 --remedy REQ-014` | `constitute` |
| Design | a `UI-nnn` flow maps to no `REQ-nnn` | `/discover IDEA-004 --remedy UI-005` | `design` |

The phase name in `--from` comes from the handoff the user pasted, and it selects the report C1 loads: `.devforgeai/reports/IDEA-nnn-plan.yaml`, `-constitute.yaml` or `-design.yaml`. Each report's `findings[]` carry one defect line per cited id, which is the text step 9 drafts against.

## What `doc reopen` writes

```
devforgeai doc reopen requirements --id IDEA-nnn --ids ID,ID --from <plan|constitute|design>
```

The binary, not the model, makes every one of these changes:

- `revision` rises by exactly 1.
- One `revision_log` entry is appended, carrying the new `revision`, an RFC 3339 UTC `at`, the `from` phase, the `reopened` list and the `added` list.
- Each cited `REQ-nnn` moves to `status: reopened` in place and stays in the epic that already groups it.
- One `REQ-nnn` is allocated per cited `UI-nnn`, with `source: user`, `traces_to: [UI-nnn]`, empty `statement`, `rationale` and `acceptance_signal`, and no epic.
- `accepted_by` and `accepted_at` return to `null` and the document `status` becomes `reopened`.
- Every other byte of every other record is left as it was.

Exit 3 with `DFA-E261` says a cited id matches neither `^REQ-\d{3}$` nor `^UI-\d{3}$`; the run ends with that message on the `Blocked` line. Exit 1 says the document is absent, or a cited `REQ-nnn` is in no record.

The byte-identity guarantee is the reason this is a CLI call: an untouched record keeps the text the user accepted, and the graders that compare a pre-run document to a post-run one read exactly that.

## What the run does after C2

Steps 2 and 3 run, then the run joins the numbered steps at step 5.

| Step | At entry point C |
|---|---|
| 4 flow audit | skipped; the brief is not what was cited |
| 5 personas | skipped when the re-open cites no new actor; otherwise the existing `personas[]` block goes to `persona-mapper` as its input |
| 6, 7, 8 | skipped; elicitation ran when the set was first drafted |
| 9 drafting | `requirement-drafter` receives the reopened records, the citing report's defect lines, the `REQ-nnn` ids to redraft, and the statement that every other record stays as written |
| 10 allocation | runs for the records in `revision_log[].added` only |
| 11 grouping | skipped when `added` is empty; placement mode when it is not |
| 12 write | the reopened and added records take their new text; every other record keeps its bytes; the derived top-level `open_questions` list is rebuilt |
| 13 round 4 | asked with the new counts |
| 14 `doc accept` | moves every requirement at `draft` or `reopened` back to `accepted` and writes a fresh `accepted_at` |

## Placement mode at step 11

A `REQ-nnn` the CLI allocated for a cited `UI-nnn` belongs to no epic, and the gate's `no-ungrouped-req` check is what makes placing it non-optional. In placement mode `epic-grouper` receives the added records and the existing `epics[]` block, returns one existing `EPIC-nnn` per added `REQ-nnn`, and allocates no epic. Step 11 then appends the id to the named epic's `requirements` list and changes nothing else in `epics[]`.

A re-open from Plan or Constitute cites `REQ-nnn` ids that already sit in epics, so `added` is empty and `epics[]` keeps every byte.

## The revision number

`revision` counts re-open passes, not runs: a fresh A or B run writes `revision: 1` with an empty `revision_log`, and each later pass adds one entry and raises the number by one. The number lives in `requirements.yaml` rather than in `state.toml`, and the report path and the `state.toml` active id stay constant across revisions, because a cited `REQ-nnn` already identifies its epic through `epics[].requirements`.
