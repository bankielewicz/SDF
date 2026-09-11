# Subagents · Validating Quality

Ten agents, every one a registered verifier, every one read-only. Two hold `Bash`
for one purpose each — running the optional `config.toml` command they are told
about — and none holds `Write` or `Edit`, so no subagent of this phase changes a
byte of the project or of a document under `.devforgeai/`. The two `Bash` holders
are scoped by a `PreToolUse` handler rather than by a prefix rule over the tool
name: the handler admits the `[verify].metrics_command` and
`[verify].call_graph_command` strings of `config.toml` and denies every other
command. A `Bash(devforgeai:*)` scope was wrong in both directions — it denied the
project command each agent exists to run, and it admitted every state-changing
subcommand of the framework binary, including `phase set`, `report ingest` and
`trust pin` — and a `PreToolUse` deny cannot be bypassed by permission mode, which
is where the framework places enforcement.

## Invocation order

| # | Workflow step | Agent | Batch | Registered verifier |
|---|---|---|---|---|
| 1 | 6 | `ac-compliance-verifier` | batch 1 of 7 | yes · `verifiers.ac_compliance` |
| 2 | 6 | `standards-reviewer` | batch 1 of 7 | yes · `verifiers.standards` |
| 3 | 6 | `anti-pattern-scanner` | batch 1 of 7 | yes · `verifiers.anti_patterns` |
| 4 | 6 | `constraint-auditor` | batch 1 of 7 | yes · `verifiers.constraints` |
| 5 | 6 | `coverage-gap-auditor` | batch 1 of 7 | yes · `verifiers.coverage_gaps` |
| 6 | 6 | `dead-code-detector` | batch 1 of 7 | yes · `verifiers.dead_code` |
| 7 | 6 | `deferral-validator` | batch 1 of 7; over the deferrals already on disk | yes · `verifiers.deferrals` |
| 8 | 7 | `security-auditor` | batch 2 of 3; deep mode alone | yes · `verifiers.security` |
| 9 | 7 | `code-quality-auditor` | batch 2 of 3; deep mode alone | yes · `verifiers.quality` |
| 10 | 7 | `adr-conformance-reviewer` | batch 2 of 3; deep mode alone | yes · `verifiers.adr_conformance` |
| 11 | 9 | `deferral-validator` | alone; second pass over the deferrals this run drafts | yes · `verifiers.deferrals` |
| 12 | R3 | the agents whose name appears in the cited findings' `found_by` | batch 3; remedy runs only | yes · as above |

A row whose Batch cell names a batch is invoked in the same message as every other
row naming that batch. A row with a condition carries it in the Batch cell after a
semicolon. The seven of batch 1 read the same story from seven angles and none
reads another's output, which is why they go out together; the three of batch 2 add
the readings a release needs and share that independence.

A subagent whose JSON does not parse is invoked once more with the parse error
appended to the prompt. `devforgeai report ingest` writes the unparsable block with
`status: unparsed`, and a block still unparsed after the second attempt fails the
`verifier_pass` check naming it, with `DFA-E316`.

## Contracts

| Agent | Model | Tools | Input fields | Output schema |
|---|---|---|---|---|
| `ac-compliance-verifier` | opus | `Read`, `Grep`, `Glob` | `story_path`, `id`, `acceptance_criteria`, `files`, `out_of_scope`, `id_band` | `agents/ac-compliance-verifier.md` `## Output` |
| `standards-reviewer` | opus | `Read`, `Grep`, `Glob` | `id`, `files`, `standards`, `constraints`, `accessibility`, `id_band` | `agents/standards-reviewer.md` `## Output` |
| `anti-pattern-scanner` | sonnet | `Read`, `Grep`, `Glob` | `id`, `anti_patterns`, `files`, `id_band` | `agents/anti-pattern-scanner.md` `## Output` |
| `constraint-auditor` | opus | `Read`, `Grep`, `Glob` | `id`, `constraints`, `constraint_blocks`, `layer_rule`, `files`, `acceptance_criteria`, `id_band` | `agents/constraint-auditor.md` `## Output` |
| `coverage-gap-auditor` | sonnet | `Read`, `Grep`, `Glob` | `id`, `coverage`, `layer`, `files`, `acceptance_criteria`, `id_band` | `agents/coverage-gap-auditor.md` `## Output` |
| `dead-code-detector` | sonnet | `Read`, `Grep`, `Glob`, `Bash` | `id`, `files`, `roots`, `excluded`, `call_graph_command`, `id_band` | `agents/dead-code-detector.md` `## Output` |
| `deferral-validator` | opus | `Read`, `Glob`, `Grep` | `id`, `deferrals`, `on_disk`, `sprint_stories`, `adr_status`, `out_of_scope`, `id_band` | `agents/deferral-validator.md` `## Output` |
| `security-auditor` | opus | `Read`, `Grep`, `Glob` | `id`, `files`, `security_constraints`, `security_anti_patterns`, `forbidden_dependencies`, `id_band` | `agents/security-auditor.md` `## Output` |
| `code-quality-auditor` | sonnet | `Read`, `Grep`, `Glob`, `Bash` | `id`, `files`, `complexity_max`, `duplication_max_percent`, `duplication_min_lines`, `metrics_command`, `constraints`, `id_band` | `agents/code-quality-auditor.md` `## Output` |
| `adr-conformance-reviewer` | opus | `Read`, `Grep`, `Glob` | `id`, `adrs`, `files`, `layer`, `constraints`, `layer_rules`, `id_band` | `agents/adr-conformance-reviewer.md` `## Output` |

Every one of the ten prints exactly one JSON object on stdout: the
`devforgeai/verifier/1` envelope. Its keys are `schema`, holding the constant
string `devforgeai/verifier/1`; `subagent`, the agent's own name; `id`, the run's
`STORY-nnn`; `passed` and `total`, two integers; `unit`, the word the registry row
below carries; `findings`, an array whose entries carry `id`, `severity` from the
closed enum `block | warn | info`, `summary` and `evidence`; and `payload`, an
object holding every top-level field the agent adds of its own. Anything other than
one object of that shape is `DFA-E410`, which writes the block at
`status: unparsed` and fails the `verifier_pass` check naming it with `DFA-E316`.
Two concatenated objects are the commonest way to produce that error, so each of
the ten prints one.

Each `findings` entry of this phase adds five fields to the contract four:
`confidence`, a float from `0.0` to `1.0`, and `category`, `file`, `line`, and
`relates_to`. `code-quality-auditor` adds `measured` and `limit`, and
`security-auditor` adds `owasp`. `report ingest` copies every one of them through
unread. `payload` is `{}` for eight of the ten; `dead-code-detector` fills it with
`method` and `dead`, and `code-quality-auditor` with `method` and `over_ceiling`.

Each agent maps its finding severity onto the envelope enum by one rule: `blocker`
emits `block`, and `high`, `medium`, and `low` emit `warn`. `info` carries a
reading the agent made and deliberately did not gate on — an out-of-scope
behaviour, an accepted ADR consequence, a deferral cycle among other stories.

`passed` is `total` minus the count of units carrying a `block` finding, and
nothing else moves it: **a `warn` or `info` finding lands in the report and leaves
`passed` where it stands.** A lowered `passed` fails `verifier_pass` at the
compiled floor `min_ratio = 1.0`, which is how a blocker-severity match reaches the
gate, and it is why the verify phase's send-back rule that a `warn` sets no
gate has to hold inside the arithmetic and not only in the prose. Two agents take
the exception that keeps both rules true at once: `dead-code-detector` and
`code-quality-auditor` raise `warn` findings only, so `passed` equals `total` on
every run and the counts they would otherwise have gated on ride in `payload.dead`
and `payload.over_ceiling`, where a project that wants them to gate adds its own
`report_metric` check. One agent takes the exception in the other direction:
`standards-reviewer` emits `block` for a departure from a rule a `CON-nnn` in
`constraints` binds, so its ratio can fail at all.

`id_band` is the `FIND-nnn` range workflow step 5 allocated: the agent at position
`k` of this table, counting from zero, takes `FIND-(base + 100k)` through
`FIND-(base + 100k + 99)` and numbers its findings from the low end. The band is
100 wide rather than 20 so that it fixes ids and bounds nothing else: an agent
reports every finding its reading supports, including the uncertain and the
low-severity ones, and the gate, the QA document and the user's remedy run are what
filter. A hundred findings from one agent on one story is past any real ceiling, so
no agent truncates its list and no agent carries a `truncated` flag.

## Registered verifiers

| Agent | phase | report_field | unit | required |
|---|---|---|---|---|
| `ac-compliance-verifier` | verify | `verifiers.ac_compliance` | ACs | true |
| `standards-reviewer` | verify | `verifiers.standards` | files | true |
| `anti-pattern-scanner` | verify | `verifiers.anti_patterns` | anti-patterns | true |
| `constraint-auditor` | verify | `verifiers.constraints` | constraints | true |
| `coverage-gap-auditor` | verify | `verifiers.coverage_gaps` | layers | true |
| `dead-code-detector` | verify | `verifiers.dead_code` | symbols | true |
| `deferral-validator` | verify | `verifiers.deferrals` | deferrals | true |
| `security-auditor` | verify | `verifiers.security` | OWASP categories | true |
| `code-quality-auditor` | verify | `verifiers.quality` | files | true |
| `adr-conformance-reviewer` | verify | `verifiers.adr_conformance` | ADRs | true |

The SubagentStop hook runs `devforgeai report ingest <name> -` for each row above.
A name absent from `config.toml` `[[verifier]]` is a no-op with exit 0 and
`DFA-W411`.

The blocks land in `.devforgeai/reports/STORY-nnn-verify.yaml`, the CLI's gate
report, which is a different file from `.devforgeai/reports/STORY-nnn-qa.yaml`, the
skill's own document. `required = true` means a `verifier_pass` check names the
agent; light mode reaches the gate through `verify-deep`'s `skip_when` condition
rather than through a lowered flag, so the three deep-mode rows carry the same
value as the seven light-mode rows.

## Shared lineage

| Agent | derives_from | Other skills deriving from the same file |
|---|---|---|
| `ac-compliance-verifier` | `C:\Users\bryan\.claude\agents\ac-compliance-verifier.md` — adapted | none |
| `standards-reviewer` | `C:\Users\bryan\.claude\agents\code-reviewer.md` — adapted | none |
| `anti-pattern-scanner` | `C:\Users\bryan\.claude\agents\anti-pattern-scanner.md` — adapted | none |
| `constraint-auditor` | `C:\Users\bryan\.claude\agents\context-validator.md` — adapted | `implementing-stories` · `context-validator` |
| `coverage-gap-auditor` | `C:\Users\bryan\.claude\agents\coverage-analyzer.md` — adapted | none |
| `dead-code-detector` | `C:\Users\bryan\.claude\agents\dead-code-detector.md` — adapted | none |
| `deferral-validator` | `C:\Users\bryan\.claude\agents\deferral-validator.md` — adapted | `releasing-software` · `deferral-auditor` |
| `security-auditor` | `C:\Users\bryan\.claude\agents\security-auditor.md` — adapted | none |
| `code-quality-auditor` | `C:\Users\bryan\.claude\agents\code-quality-auditor.md` — adapted | none |
| `adr-conformance-reviewer` | `C:\Users\bryan\.claude\agents\architect-reviewer.md` — adapted | `establishing-context` · `architecture-reviewer` |

Three names were resolved against other phases. Constitute registers
`architecture-reviewer`, so the ADR reviewer here is `adr-conformance-reviewer`.
Release registers `deferral-auditor`, so the deferral checker here keeps the
existing name at `phase = "verify"`; the two write into different report files and
the registry keys on `name`. Build's context checking and this phase's
`constraint-auditor` share the `context-validator` lineage and differ in name,
phase, and tool list.
