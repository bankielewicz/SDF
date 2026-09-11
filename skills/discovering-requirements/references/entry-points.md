# Entry points

Read at step 1, when `$ARGUMENTS` and the preamble output decide which run this is.

## Telling the four forms apart

The preamble runs `devforgeai doc load discover-entry "$1"` before the entry point is known. That call exits 0 in every case, so its output is a signal rather than an error: it prints `explore/brief.md` and `explore/decision.yaml` when `$1` is an `IDEA-nnn` with a brief on disk, prints `requirements.yaml` when requirements exist and no brief does, prints both when both exist, and prints nothing when `$1` matches no id or is a quoted description.

| Signal in `$ARGUMENTS` and the preamble output | Run | `$1` holds |
|---|---|---|
| `--remedy` present | C, a re-open | the `IDEA-nnn` |
| `--resume` present | resume, re-entering at step 4 | the `IDEA-nnn` |
| a brief printed, neither flag | A, a promoted Explore brief | the `IDEA-nnn` |
| nothing printed, neither flag | B, a typed description | `$ARGUMENTS` is the description |
| `$ARGUMENTS` empty | none | the run ends with `Blocked   you: name an IDEA id, a description, or an IDEA id with --remedy` |

At entry point A the `decision` field of the printed `decision.yaml` gates the run. `promote` is the one value the run proceeds on. `kill` or `park` ends the run with `Blocked   you: decision.yaml for IDEA-nnn holds <value>, not promote`, because an idea Explore did not promote has no requirement set to draft.

## What each form reads from the brief

Discover reads six parts of `.devforgeai/explore/brief.md` and two fields of `.devforgeai/explore/decision.yaml`. It reads no other section.

| Brief part | Shape | Becomes |
|---|---|---|
| frontmatter `id` | `IDEA-nnn` | the `id` of `requirements.yaml` |
| `## Core flows` | table `ID \| Actor \| Trigger \| Steps \| Outcome`, 3 to 5 rows, `Steps` joined with ` -> ` | the step 4 audit input, the step 9 drafts, and each requirement's `source` |
| `## Problem statement` | one sentence | the rationale seed for the drafted requirements |
| `## Target user` | one segment line plus 2 to 4 attribute lines | the first persona candidate at step 5 |
| `## Success signal` | one line, `<metric> \| <threshold> \| <observation window>` | the first round 2 outcome candidate |
| `## Non-goals` | 1 to 10 present-tense lines | the round 3 exclusion candidates |

`## Success signal` and `## Non-goals` are candidate text for rounds 2 and 3 only; neither becomes a requirement, and `## Non-goals` travels on to `establishing-context` through its own carry-forward entry. From `decision.yaml`: `decision` gates the run, and `remedied_flows` names the `FLOW-nnn` ids Explore rewrote on its latest remedy run, which the resume form names in the step 4 prompt.

## The step matrix

| Step | A, brief | B, description | C, re-open | Resume |
|---|---|---|---|---|
| C1 load the citing report | — | — | runs | — |
| C2 `doc reopen` | — | — | runs | — |
| 2 fix the id | the brief's `id` | `doc validate --allocate IDEA` | the loaded document's `id` | the loaded document's `id` |
| 3 `phase set discover` | runs | runs | runs | runs |
| 4 flow audit | runs | skipped | skipped | runs, every row again |
| 5 personas | runs | runs | skipped when no new actor is cited | runs |
| 6 round 1 actors | runs | runs | skipped | runs |
| 7 round 2 outcomes | runs | runs | skipped | runs |
| 8 round 3 boundaries | runs | runs | skipped | runs |
| 9 requirement drafting | every flow | the description | the cited records only | every flow |
| 10 id allocation | new records | new records | `revision_log[].added` only | new records |
| 11 epic grouping | one epic per outcome | one epic per outcome | placement mode, or skipped | one epic per outcome |
| 12 write the document | runs | runs | runs | runs |
| 13 round 4 acceptance | runs | runs | runs | runs |
| 14 `doc accept` | runs | runs | runs | runs |

`consumes` follows the entry point: the `IDEA-nnn` plus the `FLOW-nnn` ids the requirements draw from at A, and `[]` at B, where no Explore document exists and none is created. A resume that clears the audit continues through the same steps a fresh A run takes.
