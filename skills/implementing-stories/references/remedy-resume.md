# The remedy run and the resume run

Read this at R1 and before a `--resume` run. Both are returning forms: the story already has a worktree, a branch, commits, and a build note, and both re-enter that work rather than starting it again. What separates them is where the criterion set comes from — a citing report for a remedy, the note's own cursor for a resume.

## `/build STORY-nnn --remedy FIND-nnn,...`

Arrives from Verify. The `--remedy` list holds `FIND-nnn` ids and no others, because Verify allocates the `FIND` prefix and its findings are what a Verify send-back cites.

### R1 · Resolve the findings

```
devforgeai doc load qa-report <STORY-nnn>
```

That prints `.devforgeai/reports/STORY-nnn-qa.yaml`. For each cited id, take the `findings[]` entry whose `id` matches and read four fields from it: the `FIND-nnn`, the `AC-nnn` it names, its `summary`, and its `evidence`. One quadruple per cited id.

A cited `FIND-nnn` appearing in no `findings[]` entry stops the run with one `Blocked` line naming the id. Exit 1 from `doc load` stops the run the same way, with the binary's stderr on that line.

The criterion set of workflow step 7 is the distinct `AC-nnn` values of those quadruples, in `## Acceptance Criteria` order. Two findings naming one criterion produce one cycle, not two.

### R2 · Re-enter

The run executes workflow steps 4 through 14 with that criterion set and `run: remedy`.

- Step 4 reuses the existing worktree: `worktree ensure` is idempotent and prints the path it already holds.
- Step 5 runs as usual; the story is already at `status: building`, and `phase set` leaves a document already carrying the target value untouched.
- Step 7.1 receives the criterion's `summary` and `evidence` as two further prompt fields, so the new test asserts the gap the finding named rather than re-asserting what the old test already covered.
- Steps 8 through 11 run over the whole story, not over the re-opened subset: `integration-test-writer` reads every criterion, and `context-validator` and `story-ac-verifier` read the full diff and decide every path and every criterion.

### R3 · Record

The note's `remedy` key carries the cited `FIND-nnn` list, and `cycles` holds one entry per criterion of the R1 set alone. `resumed_at` stays `""`.

### What a remedy run leaves untouched

- Criteria outside the R1 set keep their tests, their code, and their commits. No file they own is written, which `story files --diff` records as an unchanged path.
- `.devforgeai/stories/STORY-nnn.md`, `sprint.yaml`, the six context files, and every `UI-nnn.md` keep every byte. The producer check refuses a build-phase write to any of them.
- `.devforgeai/reports/STORY-nnn-qa.yaml` is read and not written; it is Verify's document.

### Where it goes next

`Next` is `/verify STORY-nnn` when the gate passes. When a re-opened criterion turns out to be untestable, the run takes the send-back of `## Send-back` instead and `Next` names Plan.

## `/build STORY-nnn --resume`

Arrives after a send-back that Plan repaired, or after a run that stopped part way.

The run re-enters at workflow step 2 and re-reads the story and the six context files from the preamble's stdout, so a criterion Plan rewrote is the text this run works from. Step 4 reuses the worktree. Step 5 is a no-op that leaves `status` at `building`.

### The cursor

Read `.devforgeai/build/STORY-nnn-note.yaml`. The criterion set begins at the first `AC-nnn` of `## Acceptance Criteria` that satisfies either condition:

- it has no entry in `cycles`, or
- its entry carries `green_commit` of `""`.

From that id the set runs to the end of `## Acceptance Criteria`. The note's `resumed_at` records that id, and `cycles` keeps every earlier entry byte for byte — the same `red_commit`, the same `green_commit`, the same paths.

An absent note file makes the criterion set the whole list, which is the `full` run. A resume after a crash before step 12 therefore loses no correctness and repeats work.

### The two shapes a send-back leaves

SB-1, SB-2, and SB-3 stop the run at step 7.1, so the criteria after the failing one have no `cycles` entry at all and the resume re-enters at the first absent one.

SB-4 arrives after step 11, so every criterion has an entry. The resume re-enters at the first entry whose `green_commit` was cleared when the criterion was re-opened for rewriting.

### What a resume run leaves untouched

- Every earlier `cycles` entry, byte for byte, including its commit shas.
- Every commit already on the branch.
- The `remedy` key, which stays `[]` on a resume.
