# The three deep-mode checks

Read before workflow step 7. These three run when the mode is `deep` — the run
carried `--deep`, or `config.toml` `[verify].mode` reads `deep` and no flag
overrode it. They are invoked in one message, after the step 6 batch has returned,
and they add three `checks[]` entries to the seven of `light-checks.md`, so a deep
report holds ten and `summary.checks` reads `10`.

| `name` | Verifier | `unit` |
|---|---|---|
| `security-audit` | `security-auditor` | OWASP categories |
| `quality-metrics` | `code-quality-auditor` | files |
| `architecture-review` | `adr-conformance-reviewer` | ADRs |

In light mode this step does not run, the three entries are absent from `checks[]`,
and the `verify-deep` gate check records `skip` with `reason: condition` from its
`skip_when = { path = "reports/{id}-qa.yaml", field = "mode", equals = "light" }`.

## What each subagent receives

Each gets its id band from workflow step 5 and the story's `STORY-nnn` as `id`.
Full field types are in `agents.md` `## Contracts` and each `agents/<name>.md`
`## Input`.

**`security-auditor`** — the `## Files` rows of `Kind` `source` and `config`, the
`### CON-nnn` blocks at `kind: security`, the `## Anti-pattern index` rows whose
`Category` is `security`, and the `## Forbidden dependencies` rows of
`dependencies.md`. Its `total` is the constant `10`, one per OWASP Top 10 category,
and every finding carries an `owasp` value from the closed enum `A01` Broken Access
Control, `A02` Cryptographic Failures, `A03` Injection, `A04` Insecure Design,
`A05` Security Misconfiguration, `A06` Vulnerable and Outdated Components, `A07`
Identification and Authentication Failures, `A08` Software and Data Integrity
Failures, `A09` Security Logging and Monitoring Failures, `A10` Server-Side Request
Forgery. Dependency risk reaches it through the forbidden list rather than through
any audit command: it runs no command and reads no network.

**`code-quality-auditor`** — the `## Files` rows of `Kind` `source` and the four
`config.toml` `[verify]` values `complexity_max`, `duplication_max_percent`,
`duplication_min_lines`, and `metrics_command`. The two ceilings are the project's
numbers, set once in configuration; this phase decides neither. A non-empty
`metrics_command` is a command it runs through Bash, reading a
`devforgeai-metrics/1` JSON object from stdout and reporting `method: command`;
`""` selects the Grep fallback, counting branch keywords and repeated line runs and
reporting `method: grep`. Every `complexity` and `duplication` finding carries
`measured` and `limit`, and the `limit` is the `config.toml` value for that kind.

**`adr-conformance-reviewer`** — every `adr/ADR-nnn.md` at `status: accepted` with
its `## Decision`, `## Consequences`, and `## Constraints introduced`, plus the
story's `## Files`, `## Layer`, and `## Constraints`, and the `## Layer dependency
rules` table. Its `total` counts the accepted ADRs whose `## Constraints
introduced` names a `CON-nnn` the story also binds, which is why an empty `adr/`
directory produces `total: 0`, `passed: 0`, and a `checks[]` entry at
`result: skip`.

## What deep mode adds to the report

The three extra `checks[]` entries, the findings they raise, and nothing else.
`mode` reads `deep`, the sixteen top-level keys stay the same, and the `coverage`
block is still the copy of the build report's figures. The three categories that
arrive only from this step are `security`, `complexity`, and `duplication`; a
`adr-conformance-reviewer` finding lands under `constraint`, cited by its
`ADR-nnn`.

The gate check that reads them is `verify-deep`, a `verifier_pass` naming the three
at `min_ratio = 1.0` with `on_fail = "send_back"`. A `block` finding from any of
the three lowers that subagent's `passed` below its `total` and routes the run to
`## Send-back`; `SB-5` in the skill's send-back table is the security case, and a
deep-mode `warn` lands in the report and sets no gate.

## When a block comes back unparsed

The failure path is the one in `light-checks.md`: `report ingest` writes the block
with `status: unparsed` and exit 0, the subagent is invoked once more with the parse
error appended, and a block still unparsed leaves `verify-deep` failing with
`DFA-E316`.
