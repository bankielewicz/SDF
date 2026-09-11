# The seven light-mode checks

Read before workflow step 6. Every run writes these seven `checks[]` entries,
whatever the mode: light mode writes these alone, deep mode writes these plus the
three of `deep-checks.md`.

## What a check entry holds

One `checks[]` entry per subagent invoked, built from that subagent's envelope:

| Field | Value |
|---|---|
| `name` | the check name from the table below |
| `verifier` | the subagent name from the same row |
| `examined` | the subagent's `total` |
| `passed` | the subagent's `passed` |
| `result` | `pass` when `passed == examined`, `fail` when `passed < examined`, `skip` when `total` is `0` |

A `total` of `0` is a real outcome rather than a miss — a story bound by no
constraint, a first run with no deferral on disk, an anti-pattern index whose
`Scope` globs match no path in the file set. The gate reads `0 / 0` as a ratio of
`1.0`, and the QA report records `result: skip` for the same pair.

| `name` | Verifier | `unit` |
|---|---|---|
| `ac-compliance` | `ac-compliance-verifier` | ACs |
| `code-review` | `standards-reviewer` | files |
| `anti-pattern-scan` | `anti-pattern-scanner` | anti-patterns |
| `constraint-validation` | `constraint-auditor` | constraints |
| `coverage-review` | `coverage-gap-auditor` | layers |
| `dead-code` | `dead-code-detector` | symbols |
| `deferral-validation` | `deferral-validator` | deferrals |

## What each subagent receives

All seven go out in one message. Each one gets its id band from workflow step 5 and
the story's `STORY-nnn` as `id`; the rest differs per agent. Full field types are in
`agents.md` `## Contracts` and each `agents/<name>.md` `## Input`.

**`ac-compliance-verifier`** — the story path, the `AC-nnn` list with each line
whole, the `## Files` rows of `Kind` `source` and `test`, the `## Out of scope`
lines. It decides whether a test reads the outcome each criterion names, and it
runs in light mode as well as deep because the default verify gate names it in the
`verify-acs` check at `min_ratio = 1.0`; a run that skipped it would fail its own
gate with `DFA-E316`.

**`standards-reviewer`** — the `## Files` rows of `Kind` `source`, the seven H2
sections of `coding-standards.md`, the `## Constraint index` rows whose
`Enforced by` cell names a standards rule, and, for a file whose `Layer` is
`interface`, the eight `## Accessibility` rows of each `UI-nnn` the story's
`## Interface` names. Those rows are `Landmark`, `Heading order`, `Name`, `Role`,
`Keyboard path`, `Focus visible`, `Contrast`, and `Motion`. Token conformance in the
changed files is `devforgeai design lint`, which the PreToolUse hook already ran
during Build, so this run raises no token finding.

**`anti-pattern-scanner`** — the `## Anti-pattern index` rows of `AP`, `Category`,
`Severity`, `Scope`, `Detector kind`, `Detector`, and `Source`, together with the
`## Files` rows whose `Path` each row's `Scope` glob matches. The severity in the
finding is the index's own cell, so a `blocker` row produces a `block` envelope
finding with no translation.

**`constraint-auditor`** — the story's `## Constraints` rows, the matching
`### CON-nnn` blocks with `kind`, `status`, `statement`, `source`, `introduced_by`,
and `enforced_by`, the `## Layer dependency rules` row for the story's `## Layer`,
the `## Files` rows, and the `AC-nnn` list. It raises two categories: `constraint`
for code that breaks a constraint, and `spec-gap` for a constraint that holds while
no criterion asserts what it requires. The second routes to Plan.

**`coverage-gap-auditor`** — the `coverage` block of
`reports/STORY-nnn-build.yaml` as `devforgeai report show` printed it, the story's
`## Layer`, the `## Files` rows, and the `AC-nnn` list. It runs no command; the
figures were measured by `devforgeai gate check --phase build` and reach it as data.

**`dead-code-detector`** — the `## Files` rows of `Kind` `source`, the `## Roots`
and `## Generated and excluded paths` entries of `source-tree.md`, and the value of
`config.toml` `[verify].call_graph_command`. A non-empty value is a command it runs
through Bash, reporting `method: command`; `""` selects the Grep path over the
roots, reporting `method: grep` with a `confidence` of `0.6` for an exported
definition and `0.9` otherwise.

**`deferral-validator`** — at step 6 it sees the deferrals already on disk: the
entries of every `reports/STORY-*-qa.yaml` whose `target` is this story, plus, on a
resume or remedy run, the `deferrals[]` of this story's own prior report. It also
gets the `sprint.yaml` `stories[]` list with each `status`, the frontmatter `status`
of each `adr/ADR-nnn.md`, and the story's `## Out of scope` lines. The deferrals
this run opens are its second pass, at workflow step 9.

## When a block comes back unparsed

The `SubagentStop` hook runs `devforgeai report ingest <name> -` for each of the
seven, writing `verifiers.<report_field>` into
`.devforgeai/reports/STORY-nnn-verify.yaml` and appending the findings. JSON that
does not parse is written with `status: unparsed` and exit 0, which leaves the hook
non-blocking. Invoke that one subagent once more with the parse error appended to the
prompt. A block still unparsed after the second attempt leaves the
`verifier_pass` check that names it failing with `DFA-E316`, which the gate reports.
