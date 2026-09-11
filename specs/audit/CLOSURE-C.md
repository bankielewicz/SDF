---
schema: devforgeai-audit/1
doc: closure
verifier: C
scope: AUDIT-6-framework (blocker, high, medium) + FIX-PLAN decisions 1-12 + measured-behaviour and product-doc sweeps
produced_by: closure-verifier-C
---

# Closure C · AUDIT-6 framework

Every blocker, high and medium row of `specs/audit/AUDIT-6-framework.md`, decided against the files
rather than against a fixer's report. Lows are out of scope for this verifier except where a low row
is the other half of an item below. AUDIT-5 is not verified here (F5 verified separately).

Counts: blocker 5 closed / 0 open. High 14: 11 closed, 3 open. Medium 16: 13 closed, 3 open.
Total 35: 29 closed, 6 open.

## AUDIT-6 · blockers

| ID | severity | status | evidence |
|---|---|---|---|
| FWK-001 | blocker | closed (as amended by decision 1) | `specs/00-conventions.md:198` replaces the §6 closing paragraph with the `systemMessage` delivery text; `:202` and the eleven-row §7 table at `:206-219` carry a `Channel to a reader` column; `:221-231` state the four Stop cases and the cross-cutting join. Case C is the **amended** decision-1 text (re-run the gate, block again, three-block budget keyed on `session_id`), not AUDIT-6's original "no gate re-run": `specs/00-conventions.md:227`, `specs/01-cli.md:387` and `:1569`, and `cli/src/hooks/run.rs:1327-1356`. Tests: `stop_active_flag_blocks_again_below_cap`, `stop_active_flag_exits_zero_at_cap_with_fail_handoff`, `stop_active_flag_reruns_gate_and_passes_when_fixed`, all `ok` at `scratchpad/seal-output.txt:771-773`. The pre-amendment test name `stop_active_flag_emits_handoff_and_runs_no_gate` no longer exists in the suite |
| FWK-010 | blocker | closed | `hooks/settings.hooks.json` third `PreToolUse` group: `"matcher": "Write\|Edit\|NotebookEdit\|Bash\|PowerShell\|Agent"` → `args: ["hook","run","trust-check"]`. `specs/00-conventions.md:212` carries the same row; `:240` states the three-surface fail-closed set. `cli/src/hooks/run.rs:288` maps `"pre-tool-use" \| "trust-check"` to `deny(&reason)` |
| FWK-020 | blocker | closed | `agents/idea-interrogator.md:4` is `tools: [Read]`; `:57` states the tool is stripped from every subagent. `specs/11-subagent-catalog.md:2130` rule 2: "**`AskUserQuestion` appears in no agent file.**" `grep -rl AskUserQuestion agents/` returns only `idea-interrogator.md`, and there only in explanatory prose, not in `tools` |
| FWK-050 | blocker | closed | `cli/src/cmd/init.rs:27-28` markers, `:36` `CLAUDE_MD_BLOCK`, `:475` `fn append_claude_md`, called after `append_gitignore`. Diffed the const against `AUDIT-6-framework.md` lines 346-376: **IDENTICAL**, 31 lines both sides. Tests present at `cli/tests/init.rs:460,484,504,536,566` (`init_creates_claude_md_when_absent`, `init_appends_block_to_existing_claude_md`, `init_twice_replaces_between_markers`, `init_claude_md_block_names_nine_commands`, `init_json_envelope_lists_claude_md`) |
| FWK-070 | blocker | closed | `skills/improving-framework/SKILL.md:37` now reads the installed tree only — `.claude/skills/<name>/SKILL.md`, `.claude/skills/<name>/templates/<file>`, `.claude/agents/<name>.md`, `.devforgeai/gates.toml`, `.devforgeai/config.toml`, the `hooks` block of `.claude/settings.json` — and says "the repository the framework is built in is not on disk here". `:57` and `:82` pass `recommendation-drafter` the `Glob` results under `.claude/`. `:59` describes a **six**-kind target table; the `command` row is gone |

## AUDIT-6 · highs

| ID | severity | status | evidence |
|---|---|---|---|
| FWK-002 | high | closed | `specs/00-conventions.md:202` ends "Exit 1 blocks nothing anywhere"; §8 bullet 2 at `:238` is the four-event list ("blocks on `PreToolUse`, `Stop`, `SubagentStop`, and `UserPromptExpansion`, through exit 2 and a decision object") |
| FWK-003 | high | closed | All seven repair-loop steps now name the channel: `specs/02-explore.md:205` and `:248`, `specs/05-plan.md:183`, `specs/07-verify.md:240`, `specs/08-design.md:203`, `specs/09-release.md:204`, `specs/10-reflect.md:188`, each "returns the `doc validate` diagnostic in `hookSpecificOutput.additionalContext`" |
| FWK-004 | high | **open** | See `## Open items` |
| FWK-011 | high | closed | `cli/src/cmd/init.rs:240-249`: `trust::verify(None)` at the end of `run`; `Ok` pushes `Next       /explore`, the error arm pushes `Next       devforgeai trust pin --framework <root>, in a terminal outside Claude Code`. `README.md:47` states the unpinned posture in the install section |
| FWK-012 | high | closed | `hooks/settings.hooks.json` `UserPromptExpansion` group, matcher `explore\|discover\|constitute\|plan\|build\|verify\|release\|design\|reflect`, arm `prompt-expansion`, timeout 30. `specs/00-conventions.md:209` carries the row; `specs/01-cli.md:1529` lists the event in the `init` `--json` `events` array |
| FWK-022 | high | closed | Beyond the seven workflow steps of FWK-003, the `## CLI calls` rows also name the channel: `specs/02-explore.md:453`, `specs/05-plan.md:405`, `specs/07-verify.md:634`, `specs/08-design.md:393`. `specs/00-conventions.md:213-214` are the two PostToolUse table rows |
| FWK-023 | high | closed | The model-run handoff is gone from both cross-cutting skills. `skills/designing-interfaces/SKILL.md:132` and `skills/improving-framework/SKILL.md:69` run `phase set <design\|reflect>` which records `[last_cross]`, and each says "this skill runs no `devforgeai handoff` of its own". `specs/08-design.md:217` and `specs/10-reflect.md:192` match. `specs/00-conventions.md:231` specifies the join and the clear; `cli/src/cmd/handoff.rs` carries `last_cross` |
| FWK-027 | high | **open** | See `## Open items` |
| FWK-040 | high | closed | `ls commands` → "No such file or directory". `python scratchpad/f6b_cmdcheck.py` exits 0: all nine spec `## Command` fences are byte-equal to their SKILL.md frontmatter-plus-preamble head |
| FWK-041 | high | closed | `grep -l "disable-model-invocation: true" skills/*/SKILL.md` returns exactly eight — the seven phases plus `improving-framework`. `skills/designing-interfaces/SKILL.md` contains the string zero times. `specs/00-conventions.md:136` and `specs/BUILD-BRIEF.md:44` state the eight/one split |
| FWK-071 | high | **open** | See `## Open items` |
| FWK-072 | high | closed | `cli/src/cmd/init.rs:20` `const EXCLUDED_SUBTREES: &[&str] = &["evals"];`, applied inside the `COPIED` walk at `:108`. `README.md:49` and `specs/BUILD-BRIEF.md:79` both say every `evals/` subtree is left behind |
| FWK-080 | high | closed | `grep -n "no_ceremony\|CEREMONY" skills/establishing-context/evals/graders.py` returns nothing: the grader and the constant are deleted. `specs/00-conventions.md:38-48` carries the six-clause scope and the "document the framework produces about a target project" exemption; `scripts/ceremony_scan.py` implements it. `python scripts/ceremony_scan.py` over the default 94-file set exits 0, "no ceremony in 94 file(s)" |
| FWK-081 | high | closed | `skills/discovering-requirements/templates/requirements.yaml:35` is `priority: must`. `specs/03-discover.md:89`, `:134`, `:252` all carry `must \| should \| could \| wont`; `:786` is the recorded decision. `grep -rn "priority: required\|expected\|excluded" skills/` returns nothing |

## AUDIT-6 · mediums

| ID | severity | status | evidence |
|---|---|---|---|
| FWK-005 | medium | closed | `specs/00-conventions.md:213` and `:214` are the two PostToolUse rows, each naming `hookSpecificOutput.additionalContext` in the channel column |
| FWK-006 | medium | closed | `grep -rn "one sentence before\|may write one sentence\|nothing after it" skills/ specs/ README.md` returns no live text — only the audit files and `specs/01-cli.md:3010`, which records that the rule is deleted. The replacement sentence is present, e.g. `specs/08-design.md:217`, `specs/10-reflect.md:192` |
| FWK-013 | medium | closed | The refusal no longer depends on the suspect binary's own handoff rendering: `specs/00-conventions.md:240` states the three surfaces (PreToolUse deny, Stop block, UserPromptExpansion block) and explicitly demotes `[last_gate].result = TRUST_FAIL` to "a record rather than the channel the refusal depends on" (`:238`). `README.md:47` states the same to the user |
| FWK-021 | medium | **open** | See `## Open items` |
| FWK-024 | medium | closed | `specs/04-constitute.md:159` now reads "`SubagentStop` runs `devforgeai report ingest <agent_type> -` on the payload's `last_assistant_message`". `specs/02-explore.md:471` uses the `-` stdin form. No `<stdout>` or `<source>` argument survives in either spec |
| FWK-025 | medium | closed | Each `skills/*/agents.md` re-invocation paragraph now names the ingest consequence rather than implying the retry covers it, e.g. `skills/establishing-context/agents.md:16` ("a second failure leaves the verifier's block absent from the report, which `gate check --phase constitute` reports as exit 1"), `skills/implementing-stories/agents.md:19` (`status: unparsed` → failed `verifier_pass`). The ingest path itself is now blocking: `specs/00-conventions.md:216` SubagentStop blocks on envelope parse failure, and `specs/04-constitute.md:159` says the `reason` reaches the subagent as its next instruction |
| FWK-026 | medium | **open** | See `## Open items` |
| FWK-042 | medium | closed | `cli/src/cmd/init.rs:15` `const COPIED: &[&str] = &["skills", "agents"];`. `grep -n SKILL_COMMANDS evals/runner/run_jsonl.py` returns nothing; `:558` records that `commands/` no longer exists. `cli/tests/init.rs` carries `init_installs_each_skill_under_its_slash_name` (`:721`), `init_removes_a_skill_installed_under_its_old_directory_name` (`:751`), `init_keeps_a_foreign_skill_that_shares_a_directory_name` (`:770`), `init_falls_back_to_the_directory_name_when_the_frontmatter_has_none` (`:792`) |
| FWK-043 | medium | closed | All six contract-document loci rewritten: `README.md:28`, `specs/00-conventions.md:19` (rule 4), `:128` (§4b entry-point shape), `specs/BUILD-BRIEF.md:32`. The layout blocks in `README.md:14-27` and `specs/00-conventions.md:57-100` hold no `commands\` line; `README.md:52` installed-tree block is `.claude\{skills,agents}\` |
| FWK-052 | medium | closed | `cli/src/cmd/init.rs:242-249`, the conditional `Next` line, same evidence as FWK-011. The CLAUDE.md block is the second place the posture is stated (FWK-050) |
| FWK-073 | medium | closed | `README.md:39` install sequence is `devforgeai trust pin --framework C:\Projects\DevForgeAI`, and `:45` states "`--framework` is required: `init --from` defaults to the path the pin recorded, and without it `init` exits `DFA-E111` asking for `--from`" |
| FWK-074 | medium | closed | `README.md:32` "Build the binary first", followed by the `cargo build --release --manifest-path cli\Cargo.toml` line at `:36` and the `PATH` sentence |
| FWK-075 | medium | closed | `README.md:49` lists the `.gitignore` append (`.explore-prototype/`, `.devforgeai/state.toml`) and the CLAUDE.md section write between markers, alongside the other five actions |
| FWK-076 | medium | closed | `README.md:83`: "Evals run from this repository, not from a target project: `--skill skills/<name>` is a path here, and a target holds `.claude/skills/<name>/` with no `evals/` in it and no runner" |
| FWK-082 | medium | closed | `specs/00-conventions.md:38-48` states the six clauses in order and `:49` names `scripts/ceremony_scan.py` as the single implementation. `specs/BUILD-BRIEF.md:89` calls the script rather than an inline grep; `:93` explains the replacement. (One residual contradiction in the *case* half of the same text is recorded under FWK-083 and Regressions) |
| FWK-083 | medium | **open** | See `## Open items` |

## Mechanical checks

**8. `python <scratchpad>/review_spec.py specs/02..specs/10`** — exit 1.

```
== specs/02-explore.md: 1 FAIL   ceremony term 'never' in ## Subagents prose
== specs/03-discover.md: 1 FAIL  ceremony term 'must' in ## Templates
== specs/04-constitute.md: 2 FAIL ceremony term 'never' in ## Subagents prose (x2)
== specs/05-plan.md: 1 FAIL     ceremony term 'always' in ## Workflow prose
== specs/06-build.md: 1 FAIL    ceremony term 'never' in ## Subagents prose
== specs/07-verify.md: 3 FAIL   ceremony term 'never' in ## Subagents prose (x3)
== specs/08-design.md: 0 FAIL / == specs/09-release.md: 0 FAIL (1 INFO: proposed CLI addition `devforgeai release gate`) / == specs/10-reflect.md: 0 FAIL
```

Disposition. `review_spec.py` (written 02:13, before FIX-PLAN decision 9) applies the old unscoped
case-insensitive regex to whole sections. Under `specs/00-conventions.md:36` the rule reaches
"a spec's workflow and template sections", so the seven `## Subagents` hits are out of scope by
definition. The two in-scope hits were re-tested with the canonical scanner:

- `python scripts/ceremony_scan.py specs/02..07` → exit 1, 8 hits, none in `## Templates`.
  `03-discover ## Templates 'must'` is released by clause 2 (a `priority: must` YAML scalar inside
  the requirements template) — the canonical scanner does not flag it.
- `05-plan.md:195` (`## Workflow`) **is** flagged by both scanners. The text is
  "A remedy run always finds `sprint.yaml` on disk" — ordinary lower-case English, released by no
  clause, in scope by section. It is a hit only because the case rule was reverted; see FWK-083.

No structural failure: every spec carries the §11 H2 set, and no spec is missing a `## Command`,
`## Gate`, `## Integration`, `## Handoff` or `## Decisions` section.

**8a. `python <scratchpad>/f6b_cmdcheck.py`** — exit 0.

```
OK 02-explore.md == skills/exploring-ideas/SKILL.md
OK 03-discover.md == skills/discovering-requirements/SKILL.md
OK 04-constitute.md == skills/establishing-context/SKILL.md
OK 05-plan.md == skills/planning-work/SKILL.md
OK 06-build.md == skills/implementing-stories/SKILL.md
OK 07-verify.md == skills/validating-quality/SKILL.md
OK 08-design.md == skills/designing-interfaces/SKILL.md
OK 09-release.md == skills/releasing-software/SKILL.md
OK 10-reflect.md == skills/improving-framework/SKILL.md
```

All nine spec `## Command` fences are byte-equal to the shipped frontmatter plus preamble.

**8b. `python <scratchpad>/f6b_schemacheck.py`** — exit 1, `differing fences: 6`.

```
DIFF epic-grouper                 03-discover.md (near line 276)
DIFF persona-mapper               03-discover.md (near line 197)
DIFF requirement-drafter          03-discover.md (near line 235)
DIFF requirement-coverage-auditor 08-design.md   (near line 301)
DIFF story-ac-verifier            06-build.md    (near line 355)
MISSING fence: frontend-implementer in 06-build.md
```

Disposition: five of the six are **formatting-only** — the phase spec pretty-prints the same schema
the agent file writes compactly (`\d{3}` vs `[0-9]{3}`, brace rewrapping, one key per line). No
`required` list, `enum`, `pattern` semantic, `minLength`/`maxLength` bound or `additionalProperties`
value differs. Every agent fence matches its `specs/11-subagent-catalog.md` entry (the checker
reports zero catalog diffs), which is the entry `report ingest` is built against, so no contract is
at risk. The sixth, `frontend-implementer`, has no output fence in `06-build.md`; it is not a
registered verifier and `specs/11-subagent-catalog.md` carries its schema, so the gap is
documentary. Recorded under Regressions as cosmetic drift, not a blocker.

**(b) FIX-PLAN decisions 1-12** — code, skills, agents, specs and evals each checked.

| # | Decision | Verdict | Evidence per layer |
|---|---|---|---|
| 1 | Handoff delivery (amended) | all agree | code `cli/src/hooks/run.rs:1327-1356` (budget, per-session key, gate re-run) · skills `skills/designing-interfaces/SKILL.md:132`, `skills/improving-framework/SKILL.md:69` (`[last_cross]`, no own handoff) · agents n/a · specs `specs/00-conventions.md:221-231`, `specs/01-cli.md:387`, `:1569` · evals `scratchpad/seal-output.txt:771-773` (three amended Stop tests `ok`) |
| 2 | Fail closed under the real contract | all agree | code `run.rs:288` deny arm, `:296` `stop_hook_active` trust branch · skills n/a · agents n/a · specs `00-conventions.md:202`, `:238`, `:240`; `01-cli.md:1646-1647` · evals hooks-on workspaces carry the same block (`run_jsonl.py:594-607`) |
| 3 | Every gate check kind fails closed | all agree | code `cli/src/gate.rs` carries 102 `DFA-E3*` sites · skills n/a · agents n/a · specs `01-cli.md:318` ("It never reports `pass` and never reports `skip`", the three `skip` producers, `exits 5` on a stub) · evals gates fixtures ship `FIXTURE:gates-default.toml` |
| 4 | One JSON object per verifier | all agree | code `report ingest` parses the envelope (`01-cli.md:416` metric rooting) · skills `skills/*/agents.md` re-invocation paragraphs name `status: unparsed` · agents all 46 `agents/*.md` carry "One JSON object on stdout and nothing else" · specs `04-constitute.md:239`, `07-verify.md:262`, `:264` ("exactly one ... and never two"; "a `warn` finding never lowers `passed`"; "every finding is reported") · evals graders read the ingested block |
| 5 | No subagent asks the user | all agree | code n/a · skills `skills/exploring-ideas` step 2b owns the question (`specs/02-explore.md:197`) · agents `tools` lists hold no `AskUserQuestion` in any of 46 · specs `00-conventions.md:55`, `11-subagent-catalog.md:2130`, `BUILD-BRIEF.md:70` · evals runner pre-seeds through the skill's own tool call |
| 6 | Commands collapse into skills | all agree | code `init.rs:15` `COPIED = ["skills","agents"]`, `run_jsonl.py` has no `SKILL_COMMANDS` · skills nine SKILL.md carry `name`/`argument-hint`/`allowed-tools`/preamble; no `commands/` on disk · agents `SLASH_COMMANDS` unchanged and still valid (FWK-044) · specs `00-conventions.md:19`, `:128`, `BUILD-BRIEF.md:32`, `README.md:28` · evals cmdcheck 9/9 byte-equal |
| 7 | Shell matchers are `Bash\|PowerShell` | **partial** | code `hooks/settings.hooks.json` (both `PowerShell` PostToolUse entries, `trust-check` six-tool matcher) · skills `allowed-tools` opens `Bash(devforgeai:*), PowerShell(devforgeai:*)` in all nine · agents n/a · specs `00-conventions.md:55`, `:135`, `:211-214` · evals `run_jsonl.py` writes the same block. **One spec row missed:** `specs/06-build.md:521` — see FWK-027 |
| 8 | Agent descriptions are delegation triggers | all agree | 46/46 `description` lines measured: length 104-154 characters, one to two sentences, **zero** name a slash command. Purpose prose moved to the body · specs `BUILD-BRIEF.md:70` states the rule |
| 9 | MoSCoW restored, ceremony rule scoped to one script | **partial** | code `scripts/ceremony_scan.py` exists and is the only regex · skills template `priority: must` · agents n/a · specs `00-conventions.md:30-49`, `03-discover.md:786` · evals `no_ceremony` grader deleted. **Departure:** the case half was reverted to case-insensitive and two documents still assert case-sensitivity — see FWK-083 and Regressions R1/R2 |
| 10 | `init` writes the CLAUDE.md block | all agree | code `init.rs:27,36,475`; block diffed byte-identical to the audit text · skills n/a · agents n/a · specs `01-cli.md` `## Outputs` CLAUDE.md row, `README.md:49` · evals five `cli/tests/init.rs` tests |
| 11 | Eval runner pre-seeds AskUserQuestion answers | all agree | code `evals/runner/answer_hook.py` and `permission_host.py` exist; `run_jsonl.py:599-607` passes `--output-format stream-json --verbose`, `--allowedTools`, and `--permission-prompt-tool mcp__dfa-permissions__approve` · specs `00-conventions.md:254`, `ANTHROPIC-GUIDANCE.md:70`, `:112`, `BUILD-BRIEF.md:79` · (F5's case-level landing is verified separately) |
| 12 | Spec truth follows code truth | **partial** | Held in 30 of 35 rows. The five open rows below are all "the code and the skills landed, a spec locus did not" |

**(c) The three measured Claude Code behaviours, across the five named documents.**

| Behaviour | 00-conventions | 01-cli | ANTHROPIC-GUIDANCE | README | BUILD-BRIEF |
|---|---|---|---|---|---|
| A `!` preamble with `$1` is refused; `$ARGUMENTS[0]` everywhere | `:140` states it with the measured error string | `:583` states it with the same string | **`:58` lists `$N` as an available substitution with no caveat** | silent (no preamble text) | `:46` requires the preamble byte-identical to the spec, silent on the variable |
| Slash names come from the installed directory; `init` installs under the frontmatter `name:` | `:144` "**Directory name versus `name`**" states both halves | `:583` fixes the nine names by §4b | `:108` "the skill directory name already makes `/explore`" | `:49`, `:85` "`.claude/skills/<its slash name>/`" | `:48` states it and gives the `produced_by` reason |
| `AskUserQuestion` needs a permission host headless | `:254` states the 33-vs-36 tool-count measurement | `## Evals` runner contract carries it | `:70` and `:112` state it with the `claude 2.1.268` measurement | `:23` lists `permission_host.py`; the eval section does not state the requirement | `:79` states it |

Behaviours 2 and 3 are consistent everywhere they appear; only README's eval section under-states
behaviour 3, which is an omission rather than a contradiction. Behaviour 1 has a live contradiction
in `ANTHROPIC-GUIDANCE.md:58` — see Regressions R4. Verified empirically that no shipped preamble
uses `$1`: the nine preambles (dumped in full) use `$ARGUMENTS[0]` or `$ARGUMENTS` only.

**(d) README.md and BUILD-BRIEF.md describe the built system.**

- No `commands/`: `README.md:28` and `BUILD-BRIEF.md:32` both state the absence and the reason.
  Neither layout block holds a `commands\` line.
- Exec-form hooks: `README.md:49` describes the merge of the hook block plus the two
  `permissions.allow` rules; the template it refers to is exec form throughout (`command` + `args`,
  no `type: "command"` shell string), verified against `hooks/settings.hooks.json`.
- `trust pin` outside Claude Code: `README.md:32-45` gives the full sequence with `--framework`, the
  `DFA-E111` consequence, the environment refusal and the `PreToolUse` refusal of a `trust pin`
  command inside a session; `:45-47` states the unpinned posture.
- Evals mandatory with the runner's real flags: `README.md:95` "a skill without `evals.json`,
  `cases.jsonl`, and `graders.py` does not build"; `BUILD-BRIEF.md:72-77` §5. Every flag README lists
  exists in `evals/runner/run_jsonl.py:971-993` with the stated default (`--model sonnet`,
  `--timeout 900`, `--jobs`, `--filter`, `--case` repeatable, `--claude-config isolate|inherit`,
  `--hooks`/`--no-hooks`, `--dry-run`). One flag is undocumented — `--preflight` (`:993`); see R6.

## Open items

**FWK-004 · high · the deleted exit-1 Stop semantics survive in five spec loci.**
`specs/00-conventions.md:202` now ends "Exit 1 blocks nothing anywhere", and §7's Stop row blocks on
exit 2 under the three-block budget. Five spec passages still assert the old claim **and cite §7 as
their authority**, so a reader following the citation lands on text that contradicts them:

- `specs/04-constitute.md:161` — "Failure: exit 1 blocks the first Stop and returns stderr to the model; a second Stop prints the FAIL handoff."
- `specs/05-plan.md:185` — "Failure path: exit 1 blocks the stop once and the second Stop prints the FAIL block, per §7."
- `specs/06-build.md:140` — same sentence, plus "Exit 2 prints the SEND BACK block".
- `specs/07-verify.md:242` — same sentence.
- `specs/02-explore.md:941` (Decision 9) — "`gate check --phase explore` exits 1 and ... §7 gives the Stop hook's behavior: a FAIL blocks the stop once, and the second Stop passes with the FAIL handoff."

What remains: replace each with the amended decision-1 wording — the Stop hook exits 2 with
`decision: block` under a three-block per-session budget, and the FAIL handoff renders in
`systemMessage` at the last block of the budget. The four shipped `SKILL.md` loci the audit named
**are** fixed (no hit in any of them), so this is the spec half of a fix whose skill half landed.
Reachable path: a builder regenerating a phase spec's `## Workflow` close reintroduces the
exit-1 premise, and the four-spec agreement makes it look authoritative.

**FWK-027 · high · one `Bash`-alone matcher survives, at the locus the audit named.**
Two of the three loci are fixed: `specs/00-conventions.md:211-214` are all `Bash\|PowerShell`, and
`specs/01-cli.md:2518` describes the three PreToolUse groups and both shell tools. The shipped
template is correct — `hooks/settings.hooks.json` registers both `Bash(@@TEST_COMMAND@@)` and
`PowerShell(@@TEST_COMMAND@@)` in the PostToolUse shell group. The third locus is not:

- `specs/06-build.md:521` — "| §7 PostToolUse | **Bash**, command matching `[[stack]].test_command` | `devforgeai gate check --phase build --partial` |"

It is the only `Bash`-alone matcher left in any spec (`grep` over `specs/0*.md specs/1*.md` returns
this line and nothing else), and the row cites §7, which says `Bash\|PowerShell`. This is exactly
FWK-027's stated failure: "the three spec loci are where a regenerated template would pick the defect
back up." On a Windows target the model's test command resolves to the PowerShell tool, so a hook
regenerated from this row would not fire the partial build gate at all.

Secondary, same table, same class: `specs/06-build.md:518-520` give the write-tool matcher as
"Write, Edit" where §7's rows are `Write\|Edit\|NotebookEdit`. Not named by FWK-027, and worth the
same one-line fix while the table is open.
What remains: `Bash\|PowerShell` at `:521`, and `NotebookEdit` added at `:518-520`. Owner F6.

**FWK-071 · high · shipped, installed files still cite specs that are not installed.**
The three named skill loci are fixed — `skills/validating-quality/agents.md:48-56` now points at
`agents/<name>.md ## Output` rather than at `specs/01-cli.md`, and
`skills/establishing-context/references/constraints.md:18-28` carries the contract inline. The
`evals/` loci (`graders.py:17`, two `digests.txt`, `baselines.txt`) are moot because FWK-072 stops
`evals/` reaching a target. But the failure mode the finding describes is still reachable at scale in
files `init` **does** copy:

- All 46 `agents/*.md` cite `specs/01-cli.md ## Subagents` as the authority for the verifier
  envelope — e.g. `agents/ac-test-writer.md:32`, `agents/context-validator.md:28`,
  `agents/alignment-auditor.md:28`. These land at `.claude/agents/<name>.md`. This is FWK-071's own
  "worst case" wording: the envelope contract is the one thing an agent author needs and the one
  thing the target does not hold.
- `skills/validating-quality/agents.md:14` and `:87`, and `skills/implementing-stories/agents.md:50`,
  cite `specs/00-conventions.md` §1/§6 and `specs/07-verify.md` `## Send-back`. `agents.md` is copied
  by `init`: `cli/src/cmd/init.rs` `copy_skills` calls `copy_tree` on each whole skill directory
  (`:420`), and `EXCLUDED_SUBTREES` (`:20`) holds `evals` alone, so every other file in the skill
  directory — `agents.md`, `references/`, `templates/` — lands in the target.

What remains: state the envelope contract in the shipped file, or cite the installed
`.claude/agents/<name>.md`. Owner is F3 for `agents/*.md`, F4 for the two `agents.md` files.

**FWK-021 · medium · the tool-availability correction reached the catalog and not the phase specs.**
`specs/11-subagent-catalog.md:294` and `:324` now give the correct reason ("could not have been
kept", "not dropped so much as unavailable — no subagent has it"). Four spec passages still frame the
removal as a policy choice, which is the reading the audit says produced `idea-interrogator`:

- `specs/03-discover.md:189` — "the AskUserQuestion interviews are dropped: the skill owns every user question"
- `specs/03-discover.md:227` and `:804` — "its `Write`, `Edit`, and `AskUserQuestion` tools are removed: the skill writes the file and asks the questions"
- `specs/07-verify.md:572`, `:586`, `:589` — "`AskUserQuestion` leaves the tool list, because a registered verifier runs unattended"
- `specs/09-release.md:257` — "because a verifier that asks a question is not deterministic"

What remains: one clause per locus, matching the catalog's wording. Low risk, but the audit's point
is that a policy reason reads as a reversible choice.

**FWK-026 · medium · `phase set`'s caller column still names the `!` preamble.**
`specs/00-conventions.md:119` lists the caller of `phase set <phase> --id <id>` as "command `!`
preamble". Empirically no shipped preamble runs it: the nine preambles (dumped in full above) run
`gate require`, `doc load`, `report aggregate` and `story list` only, and `specs/00-conventions.md:142`
now states "No preamble allocates an id." All nine skills run `phase set` from a workflow step
(`skills/exploring-ideas/SKILL.md:31`, `discovering-requirements:36`, `establishing-context:66`,
`planning-work:48`, `implementing-stories:53`, `validating-quality:99`, `releasing-software:38`,
`designing-interfaces:132`, `improving-framework:69`), which is the weaker caller FWK-026 named.

Mitigated but not closed: each skill now states the failure path explicitly ("Exit 1 on `DFA-E320`
means the plan gate is not PASS; the run stops and the Stop hook prints the gate result"), so the
model is told what a refusal means. The reachable path that remains is the conventions row, which
tells a future author that the enforcement point is the preamble when it is not.
What remains: change the caller column to "workflow step; the predecessor gate is enforced by the
`UserPromptExpansion` hook and by the preamble's `gate require`".

**FWK-083 · medium · the case rule was reverted and two documents still assert the old one.**
The fix wave chose the opposite of FWK-083's corrected statement: `specs/00-conventions.md:36` and
`scripts/ceremony_scan.py:22-27, 63-69` both match **case-insensitively** and rely on the six scoping
clauses instead, with the rationale written out ("a case rule lets `You must never skip this`
through"). That is a defensible documented departure and the default scan is clean
(`python scripts/ceremony_scan.py` → exit 0, 94 files). It is recorded as open rather than as a
departure because two documents were left asserting the rule that was not adopted:

- `specs/BUILD-BRIEF.md:66` — "The single-word alternatives match case-sensitively, so the ordinary lower-case English uses of those words are prose, not ceremony."
- `specs/04-constitute.md:1071` — "which keeps both clear of the §2 pattern's single-word half under the case-sensitive rule."

Reachable path: a builder following the brief's §3 writes lower-case `always`/`never` freely, then
runs the brief's own §6 command (`python scripts/ceremony_scan.py`) and gets hits the brief told them
could not occur, with no rule for dispositioning them — which is the exact cost FWK-082 was raised to
remove. Demonstrated: `specs/05-plan.md:195` is a live in-scope hit on ordinary lower-case "always".
What remains: either restore case-sensitivity in the script (matching the audit) or correct
`BUILD-BRIEF.md:66` and `04-constitute.md:1071` to the case-insensitive rule. One of the two.

## Regressions

**R1. Two authorities disagree on the ceremony pattern's case rule.**
`specs/BUILD-BRIEF.md:66` says case-sensitive; `specs/00-conventions.md:36` and
`scripts/ceremony_scan.py:63-69` (`re.IGNORECASE` on both patterns) say case-insensitive. FWK-084's
stated goal was one definition referenced by the other two; the regex is now written once, but the
*rule about the regex* is stated twice and inconsistently. Same defect at
`specs/04-constitute.md:1071`. (This is the FWK-083 open row; listed here because it is a
contradiction introduced by the fix, not a pre-existing one.)

**R2. `specs/ANTHROPIC-GUIDANCE.md:58` contradicts the measured `$1` refusal.**
The guidance's substitution list reads "`$ARGUMENTS`, `$ARGUMENTS[N]`, `$N`, `$name`, ..." with no
caveat, while `specs/00-conventions.md:140` and `specs/01-cli.md:583` both record that a `!` command
containing `$1` is refused before it runs ("Contains simple_expansion") and aborts the invocation.
The guidance's own preamble says "Where our framework contradicts a statement, the statement wins",
so the file that is designated as the tie-breaker is the one carrying the stale claim. Add the
measured caveat to `:58`, or cross-reference §4b.

**R3. Five spec loci contradict §7 while citing it.** FWK-004 above. This is the sharpest form of
the regression class: `specs/05-plan.md:185`, `specs/06-build.md:140` and `specs/07-verify.md:242`
end with "per §7", and §7 now says the opposite.

**R4. `specs/00-conventions.md:119` names a caller that does not exist.** FWK-026 above.

**R5. Six agent output fences drift from their phase spec (cosmetic).**
`f6b_schemacheck.py` reports five formatting-only diffs (`epic-grouper`, `persona-mapper`,
`requirement-drafter`, `requirement-coverage-auditor`, `story-ac-verifier`) and one missing fence
(`frontend-implementer` in `06-build.md`). No semantic difference in any of the five: the checker
reports zero diffs against `specs/11-subagent-catalog.md`, which is the entry `report ingest` is
built against. Recorded so a future byte-identity check is not read as a contract break.

**R6. `README.md` omits one runner flag.** `--preflight` exists at
`evals/runner/run_jsonl.py:993` and appears in neither README's "Other arguments" list nor
`specs/BUILD-BRIEF.md`. One-word fix.

## Not verified here

`cargo test` / `cargo clippy` totals, `ceremony_scan` as mechanical check 2, the 46-agent frontmatter
parse, the nine-SKILL.md frontmatter sweep, the `settings.hooks.json` template diff, the runner unit
tests, and the `devforgeai init` smoke run (mechanical checks 1-7 and 9) are other verifiers' scope;
the seal at `scratchpad/seal-output.txt` is green at 934 tests, 0 failures, which this report relies
on only for the three amended Stop-hook test names. AUDIT-5 findings are out of scope.
