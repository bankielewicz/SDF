# The flow audit and the trip back to Explore

Read at step 4, on entry point A and on the resume form.

## The two judgments

`devforgeai doc validate` already performs the field-level and cross-reference checks on the Explore side, so the audit covers the two things no field check reaches, both of them judgments over prose:

**A row that names no actor.** The `## Core flows` table carries an explicit `Actor` column, so an empty cell is a field-level defect the Explore-side validation catches first; when one survives that far the auditor reports it as `reason: empty`. Two judgment cases follow. `reason: not_a_persona`: the cell holds text, and that text names no role a person could hold — a system, a document, a time of day. `reason: unlisted_role`: the cell names a role a person could hold, and `personas[]` lists no persona for it, so the flow has an actor the requirement set cannot trace to. Each entry carries the `FLOW-nnn`, its `reason` from the closed three, a `confidence` from `0.0` to `1.0` for how far the reading carries, and an `evidence` string of at most 200 characters quoting the cell.

**A pair of rows that cannot both hold.** Two rows contradict when their `Trigger`, `Steps` or `Outcome` cells describe states that exclude each other: one row says only a finance lead approves a refund and another says a clerk approves one with no finance lead. Each entry carries both `FLOW-nnn` ids, the `cells` that disagree — one or more of `trigger`, `steps`, `outcome` — a `confidence` from `0.0` to `1.0`, and an `evidence` string naming the disagreement.

Both arrays and both counts sit under `payload`. `payload.flows_checked` is the row count, `payload.flows_clean` the count of rows in neither list. Those two numbers fill the handoff `Verified` line through the report.

## What a finding does to the run

A non-empty `payload.actorless` or `payload.contradictions` stops the run at step 4. No `requirements.yaml` is written, and a document at the prior revision keeps every byte. `gate check --phase discover` exits 2, `state.toml` `[last_gate]` records `result = "SEND_BACK"`, `send_back_to = "explore"` and the cited ids in `failed_checks`, and the Stop hook prints the block:

```
Phase     1 · Discover        IDEA-004 · payment-reconciliation
Done      9 flows read, 0 requirements written
Gate      SEND BACK to Explore  FLOW-003 FLOW-007 FLOW-009
Verified  flow-integrity-auditor · 6/9 flows
Found     FLOW-003 and FLOW-009 disagree on who approves a refund
Found     FLOW-007 names no actor

Next      /explore IDEA-004 --remedy FLOW-003,FLOW-007,FLOW-009
Then      /discover IDEA-004 --resume
Blocked   none

Full report: .devforgeai/reports/IDEA-004-discover.yaml
```

`devforgeai handoff` composes and prints that block from the report and `state.toml`. This skill writes one sentence above it and nothing below it.

The reason a defect in the brief goes back rather than getting fixed here: the brief belongs to Explore, and a phase that finds a defect upstream cites ids rather than editing the document it reads.

## The resume form

The user returns with `/discover IDEA-004 --resume`. Explore's remedy run rewrote the cited rows and recorded their ids in `decision.yaml` `remedied_flows`, so the resume prompt names that list to the auditor as the rows that changed, and the auditor audits every row again — a rewritten row can contradict a row nobody touched.

Step 4 is the one step a run stops at without writing a document, which is why `--resume` carries no stored cursor: re-entering at step 4 re-runs the audit against the brief as it now stands, and a clean audit continues through steps 5 to 15 exactly as a fresh entry-point-A run does.

## The registered verifier

`flow-integrity-auditor` prints the `devforgeai/verifier/1` envelope alongside its findings. SubagentStop hands that stdout to `devforgeai report ingest flow-integrity-auditor`, which writes it under `verifiers.flow_integrity` of `.devforgeai/reports/IDEA-nnn-discover.yaml`. `passed` is `payload.flows_clean`, `total` is `payload.flows_checked`, and `unit` is `flows`. The discover gate names no `verifier_pass` check, so that block feeds the handoff `Verified` line and nothing else.
