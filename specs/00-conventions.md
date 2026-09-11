---
schema: devforgeai-spec/1
doc: conventions
status: authoritative
produced_by: orchestrator
consumed_by: every spec author, every skill builder, the CLI team
---

# DevForgeAI spec conventions

This document is the contract all other specs conform to. Where a spec disagrees with this document, the spec is wrong. Where this document is silent, the spec must decide and state the decision under `## Decisions`.

## 1. The eight rules

1. Skills contain judgment, workflow, and templates only. Enforcement lives in Claude Code hooks, git hooks, and the `devforgeai` Rust CLI. No skill, command, or subagent may contain a validation step that the CLI can perform.
2. The framework installs into any language project, greenfield or brownfield. No skill, subagent, or hook may name a language, package manager, test runner, linter, or build tool. Those come from `.devforgeai/config.toml`, written by `devforgeai stack detect`.
   Two classes of proper noun are sanctioned, because neither is a thing `stack detect` can detect and neither varies with the project's language. The first is a **deployment platform**: `releasing-software` resolves one target from a five-value enum, and a manifest for a named platform is written by naming it. The second is a **first-party Claude Code capability**: the `figma:*` and `frontend-design:*` skill names, and the built-in tool names, are how the harness addresses a capability, and a skill that means to reach one has no other spelling for it. Both classes are named in prose and neither reaches `config.toml`; every other proper noun stays out.
3. Every phase ends with the handoff block defined in §6. The Stop hook renders it and emits it in the hook's JSON; the model composes no part of it and writes no part of it.
4. One skill per phase (Explore, Discover, Constitute, Plan, Build, Verify, Release) plus two cross-cutting skills (Design, Reflect). Each skill is its own entry point: its frontmatter `name` is the slash command, and the `!` preamble lines sit at the top of its `SKILL.md`. There is no `commands/` directory.
5. Each phase emits exactly one typed document (§5) that the next phase reads.
6. A phase that finds a defect in an upstream document does not edit that document. It emits a SEND BACK handoff (§6) citing IDs.
7. The user passes slash-command parameters exactly as the previous handoff printed them.
8. No silos. Every skill has explicit relationships to other skills: the documents it consumes from them, the documents it produces for them, the phases it can send back to, the subagents it shares, and the state in `state.toml` it reads or advances. A spec that describes its phase without naming these relationships fails review.

## 2. Forbidden content

A spec, skill, command, or subagent that contains any of the following fails review.

**Ceremony.** Any sentence in a SKILL.md, a skill preamble, or a subagent file whose function is to make the model check itself. Detection pattern, in two halves:

```
\b(MUST|ALWAYS|NEVER|SHALL|CRITICAL|IMPORTANT|MANDATORY)\b
(?i)\b(verify that|ensure that|confirm that|do not skip|before proceeding|self-check|checklist)\b
```

Both halves are matched case-insensitively. The capitalised form is what the rule is aimed at — current models over-trigger on it — and the ordinary lower-case English uses of the same words are released by the scoping clauses below rather than by case, which is the sturdier of the two mechanisms: a backticked span, an enum cell, or a YAML scalar is out of scope whichever case it carries.

**Scope of the pattern.** It applies to instruction prose in a SKILL.md, a skill preamble, a subagent file, or a spec's workflow and template sections — text whose reader is the model executing it. A line is instruction prose when none of the following holds, tested in this order:

1. The line is inside a fenced block, at any fence depth.
2. The line is inside YAML frontmatter, or is a YAML or JSON scalar, key, or enum value.
3. The token that matched lies inside a backticked span.
4. The token that matched lies inside a table cell whose column holds enum values, identifiers, paths, or file names, as declared by that table's header row.
5. The token that matched is part of a path segment or a file name.
6. The line is a heading.

The pattern does not apply to a document the framework produces about a target project — the six context files, an ADR, a requirements record, a story, a UI spec, a report. Those state the project's own rules, and an absolute rule is what a project is entitled to write. A consequence of clause 2: the `requirements.yaml` `priority` enum is the MoSCoW set `must | should | could | wont`, because a YAML scalar is not instruction prose and matches nothing.

The pattern and the six clauses are implemented once, by `scripts/ceremony_scan.py`. That script is the only place the pattern is written out as code; `specs/BUILD-BRIEF.md` and any grader that needs the check call it rather than restating the regex.

Also forbidden: numbered validation checklists, "review your output for", status-transition instructions ("mark the story as"), file-existence checks ("confirm the file exists"), and any instruction to run tests, coverage, or linters and interpret the numbers. Each of these is a CLI call made by a hook or by the skill's `!` preamble.

**Ambiguity.** Any of: "as appropriate", "as needed", "if applicable", "etc.", "and so on", "or similar", "such as" without a closed list, "should" where "does" is meant, an unspecified default, an unspecified error path, an artifact without a path, a parameter without a type.

**Aspiration.** Any capability not implementable today inside a Claude Code terminal session with these primitives: the built-in tools (Read, Write, Edit, NotebookEdit, Bash, PowerShell, Grep, Glob, Agent, AskUserQuestion, Skill, ToolSearch, TodoWrite, WebFetch, WebSearch), hooks configured in `.claude/settings.json`, skills in `.claude/skills/*/SKILL.md` (which carry the slash commands as their frontmatter `name`), subagents in `.claude/agents/*.md`, and the `devforgeai` binary. `AskUserQuestion` is available to the main session alone: Claude Code removes it from every subagent whatever that agent's `tools` list holds, so a question belongs to a skill and never to an agent file. On Windows the PowerShell tool is the primary shell, so a rule or a matcher that names `Bash` names `Bash|PowerShell`. If a spec needs something outside this list, it says so under `## Decisions` as a blocker, not as a feature.

## 3. Repository layout

```
C:\Projects\DevForgeAI\
├── specs\                     spec documents, one per component (this wave's output)
├── cli\                       Rust crate `devforgeai`, TDD, tests first
├── skills\<skill-name>\       shipped into target .claude/skills/
│   ├── SKILL.md
│   ├── references\
│   ├── templates\
│   ├── agents.md              which subagents, invocation order, contracts
│   └── evals\
│       ├── evals.json         skill-creator format (references/schemas.md)
│       ├── cases.jsonl        deterministic cases, one JSON object per line
│       └── graders.py         pure functions, no network, no LLM
├── agents\<agent-name>.md     shipped into target .claude/agents/
├── hooks\settings.hooks.json  merged into target .claude/settings.json by `devforgeai init`
├── evals\runner\             run_jsonl.py, answer_hook.py, permission_host.py
├── scripts\ceremony_scan.py   the §2 ceremony rule, implemented once
└── README.md
```

Installed into a target project by `devforgeai init`:

```
<target>\
├── .claude\{skills,agents}\            copied from the framework, minus every evals\ subtree
├── .claude\settings.json               hooks merged in
├── CLAUDE.md                           a devforgeai section between markers, written by `init`
├── .devforgeai\
│   ├── config.toml                     from `stack detect`; thresholds; paths
│   ├── gates.toml                      gate definitions; the only place pass criteria live
│   ├── state.toml                      current phase, active ids, last gate result
│   ├── explore\brief.md, decision.yaml
│   ├── requirements.yaml
│   ├── context\{tech-stack,source-tree,dependencies,coding-standards,architecture-constraints,anti-patterns}.md
│   ├── adr\ADR-nnn.md
│   ├── stories\STORY-nnn.md, sprint.yaml
│   ├── ui-specs\UI-nnn.md, brand\tokens.json, brand\logo.svg
│   ├── reports\<ID>-<phase>.yaml
│   └── releases\vX.Y.Z.yaml
└── .git\hooks\{pre-commit,commit-msg,pre-push}   installed by `devforgeai hook install`
```

## 4. CLI surface (fixed)

Binary name `devforgeai`. Every subcommand accepts `--json` (machine output on stdout) and `--project <path>` (default: nearest ancestor containing `.devforgeai/`). Exit codes: `0` pass, `1` fail (gate not met, validation error), `2` send-back required, `3` usage error, `4` trust violation, `5` internal error. Human-readable output goes to stdout; diagnostics to stderr.

| Subcommand | Purpose | Called by |
|---|---|---|
| `init [--analyze]` | Create `.devforgeai/`, copy `skills` and `agents` (excluding every `evals/` subtree), merge hooks, write the CLAUDE.md section between markers, run `stack detect`. `--analyze` runs brownfield analysis to draft the six context files. | user |
| `stack detect` | Write `config.toml` with detected languages, test/coverage/lint commands, source roots. Replaces only the `[[stack]]` entries it wrote (`source = "detected"`), keeps a hand-written one (`source = "manual"`, the reading when the key is absent), appends a newly detected stack whose `id` is new, and writes the file only when the merged result differs ignoring `generated_at`. | `init`, SessionStart hook |
| `gate require <phase> <id>` | Exit 0 if predecessor gate of `<phase>` passed for `<id>`, else exit 1 with the missing gate named. | command `!` preamble |
| `gate check --phase <phase> [--id <id>]` | Evaluate the gate in `gates.toml`. Writes `reports/<id>-<phase>.yaml`. Exit 0/1/2. | Stop hook, pre-push, CI |
| `doc validate <path>` | Frontmatter keys, ID uniqueness, cross-references, status enum, producer match. Exit 0/1. | PostToolUse on Write/Edit, pre-commit |
| `doc load <name> <id>` | Print the named upstream document for `<id>` to stdout. | command `!` preamble |
| `handoff [--phase <phase>] [--id <id>]` | Print the §6 block from `state.toml` and the latest report. | Stop hook |
| `context audit` | Six context files present, pairwise contradiction check against ADRs. | Stop hook (Constitute), pre-commit |
| `story validate [<id>]` | ACs testable, REQ references resolve, no dependency cycles. | Stop hook (Plan) |
| `design lint [<paths>]` | Colors and type in changed frontend files resolve to `brand/tokens.json`. | PreToolUse on Write/Edit of files matching `config.toml` frontend globs |
| `hook install` | Write git hooks that call this binary. | `init` |
| `trust pin` / `trust verify` | Pin the binary's SHA-256 into `~/.devforgeai/trust.toml`; verify current binary against the pin. `pin` refuses to run when `CLAUDECODE` or `CLAUDE_CODE_ENTRYPOINT` is set in the environment. | human / every hook |
| `phase set <phase> --id <id>` | Update `state.toml`. Refuses unless `gate require` would pass. | workflow step; the predecessor gate is enforced by the `UserPromptExpansion` hook and by the preamble's `gate require` |
| `report show <id> <phase>` | Print a report. | user, Reflect skill |

The CLI spec (`specs/01-cli.md`) elaborates arguments, `gates.toml` schema, `config.toml` schema, `state.toml` schema, and the trust model. Skill specs reference subcommands by these names and do not invent others. A skill that needs a subcommand not listed here states it under `## Decisions` as a proposed addition.

### 4b. Slash-command names (fixed)

One entry point per skill, named after the phase, and the name is the skill's own frontmatter `name`. `/explore` → exploring-ideas, `/discover` → discovering-requirements, `/constitute` → establishing-context, `/plan` → planning-work, `/build` → implementing-stories, `/verify` → validating-quality, `/release` → releasing-software, `/design` → designing-interfaces, `/reflect` → improving-framework. Handoff `Next` and `Then` lines use these names only. A skill needing a second entry point (e.g. Design in sketch mode) takes a flag on its one command, not a second command.

**Entry-point shape.** Commands are merged into skills: a `SKILL.md` and a command file of the same name both produce `/name`, the skill wins, and the framework ships no command file. Each `SKILL.md` carries these frontmatter keys and no others:

| Key | Value |
|---|---|
| `name` | the slash name from the list above |
| `description` | what the skill does and when it is reached |
| `argument-hint` | the argument forms the `## Entry` table enumerates |
| `allowed-tools` | a closed list opening with `Bash(devforgeai:*), PowerShell(devforgeai:*)`, so the preamble and every mid-workflow `devforgeai` call runs under either shell tool |
| `disable-model-invocation` | `true` on eight skills; absent on `designing-interfaces` |

`disable-model-invocation: true` makes a skill user-only and keeps its description out of context until it is invoked. It is set on the seven phase skills and on `improving-framework`, each of which has side effects the model should not start on its own judgment. `designing-interfaces` omits it, because `exploring-ideas` and `planning-work` reach sketch mode through the Skill tool, and a skill the model cannot invoke cannot be reached that way.

Preamble lines sit directly under the frontmatter and use Claude Code's inline-bash syntax, a `!` followed by a backtick-wrapped command: `` !`devforgeai gate require plan $ARGUMENTS[0]` ``. The first argument is written `$ARGUMENTS[0]` and never `$1`: measured, a `!` command containing `$1` is refused before it runs, with `Shell command permission check failed ... Contains simple_expansion`, which aborts the whole invocation on a run that should have proceeded. `$ARGUMENTS` is the whole string, and the frontmatter `arguments:` named-positional form is not used. They run before the body loads and before the model sees any of it, and a non-zero exit aborts the whole invocation, which is the gate behaviour `gate require` is written for. Where a preamble opens a phase whose predecessor gate is keyed on a single subject, `gate require` is the first line — `establishing-context`, `planning-work`, `implementing-stories`, `validating-quality`. `discovering-requirements`, `releasing-software`, and `improving-framework` carry a preamble that loads the entry document instead, because their subject is not known until the body decides the entry point; their predecessor gate is enforced by the `UserPromptExpansion` hook (§7). `exploring-ideas` and `designing-interfaces` carry no preamble at all: both allocate their ids inside a workflow step once the run kind is known, and a preamble that allocated one would abort a run that needs none.

No preamble allocates an id. `doc validate --allocate` runs from the workflow step that needs the id.

**Directory name versus `name`.** Claude Code derives a project skill's slash command from the skill's directory name, and the frontmatter `name` is a display label. The framework's source tree keeps the long directory names, because `produced_by: implementing-stories` in every template resolves against them and the producer check reads that value; `devforgeai init` installs each skill at `.claude/skills/<frontmatter name>/`, so `skills/implementing-stories/` becomes `.claude/skills/build/` in the target and `/build` is the command there. The eval runner builds its workspaces the same way.

### 4c. Send-back and resume flags (fixed)

The upstream command re-opens only the cited IDs: `/<upstream> <ID> --remedy <ID>,<ID>`. The returning command continues where it stopped: `/<downstream> <ID> --resume`. No other flag names are used for these two purposes.

## 5. Document contract (fixed)

| Phase | Emits | Format | ID prefix | Read by | May send back to |
|---|---|---|---|---|---|
| 0 Explore | `explore/brief.md`, `explore/decision.yaml` | MD+YAML | `IDEA-nnn`, flows `FLOW-nnn` | Discover | none |
| 1 Discover | `requirements.yaml` | YAML | `REQ-nnn`, `EPIC-nnn`, `PERSONA-nnn` | Constitute, Plan | Explore |
| 2 Constitute | `context/*.md` (six files), `adr/ADR-nnn.md` | MD + frontmatter | `ADR-nnn`, `CON-nnn` (constraint) | Plan, Build, Verify | Discover |
| 3 Plan | `stories/STORY-nnn.md`, `stories/sprint.yaml` | MD + frontmatter | `STORY-nnn`, `AC-nnn`, `SPRINT-nnn` | Build, Verify | Discover, Constitute |
| 4 Build | `reports/STORY-nnn-build.yaml` | YAML, CLI-written | none | Verify | Plan |
| 5 Verify | `reports/STORY-nnn-qa.yaml` | YAML | `FIND-nnn` | Release, Reflect | Build, Plan |
| 6 Release | `releases/vX.Y.Z.yaml` | YAML | none | Reflect | Verify |
| Design | `brand/tokens.json`, `ui-specs/UI-nnn.md` | JSON + MD | `UI-nnn`, `TOKEN-<name>` | Plan, Build | Discover |
| Reflect | `reports/reflect-<date>.yaml` | YAML | `OBS-nnn`, `REC-nnn` | user | any (as recommendations, never as gates) |

Every MD document has this frontmatter, in this order, no other keys at the top level:

```yaml
---
schema: devforgeai/<doc-type>/1
id: <ID>
phase: <phase-name>
status: <enum defined per doc-type in its spec>
produced_by: <skill-name>
consumes: [<ID>, ...]
open_questions: []
---
```

YAML documents put the same keys at the top level. The envelope sits at the top level of every document the framework writes, MD and YAML alike, with one documented exception: `.devforgeai/brand/tokens.json` carries it under `meta`, because the rest of that file is a token tree a design tool reads and a sibling key at the root would be read as a token group. IDs are zero-padded three digits, allocated by `devforgeai doc validate --allocate <prefix>` (returns next free ID). Sections in MD documents appear in the order the template defines and use the template's heading text verbatim.

## 6. Handoff format (fixed)

Printed by `devforgeai handoff`. Twelve lines maximum, including blanks. Column two starts at character 11.

```
Phase     <n> · <Name>        <ID> · <slug>
Done      <counts measured by the CLI>
Gate      PASS | FAIL | SEND BACK [to <Phase>]  <numbers or IDs that produced it>
Verified  <subagent> · <n>/<m> <unit>            (omit line if no verifier ran)
Found     <ID> <one-line defect>                 (SEND BACK only; one line per ID, max 3, then "+n more in report")

Next      /<command> <args exactly as the user types them>
Then      /<command> <args>                      (omit if none)
Blocked   none | you: <one question>

Full report: .devforgeai/reports/<ID>-<phase>.yaml
```

The Stop hook emits this block; the model composes no part of it and writes nothing at the close of a phase. On a PASS the block travels in the `systemMessage` field of the hook's JSON, which the harness renders to the user and which Claude also sees. On a FAIL the hook returns a blocking decision whose `reason` names the failing checks, and the same block travels in `systemMessage` beside it. On a SEND BACK the block travels in `systemMessage` with no blocking decision, because the next step is a different command the user types. The model's final message is its own work; the block appears after it.

## 7. Hooks (fixed)

All hooks call the binary. All hooks run `devforgeai trust verify` first. Exit 2 is what blocks, on the events that can block: `PreToolUse`, `UserPromptSubmit`, `UserPromptExpansion`, `Stop`, `SubagentStop`. On every other event no exit code blocks, and the framework's channel to a reader is the hook's JSON: `systemMessage` reaches the user, `hookSpecificOutput.additionalContext` reaches Claude. Exit 4 marks a trust failure for the debug log and for the `--json` envelope; on a blocking event the dispatcher exits 2 instead. Exit 1 blocks nothing anywhere.

`hooks/settings.hooks.json` registers six events in exec form — `command` plus `args`, with the pinned binary path substituted for `@@DEVFORGEAI@@` — and dispatches seven arms of `devforgeai hook run <arm>`.

| Event | Matcher | Arm | Calls | Channel to a reader | Blocks |
|---|---|---|---|---|---|
| SessionStart | `startup\|resume\|clear\|fork` | `session-start` | `stack detect`, `handoff` | plain stdout, which this event adds to Claude's context | no channel; a trust failure emits `systemMessage`. `detect` here is idempotent: it preserves every `manual` stack and writes nothing when the merged result is unchanged, so opening a session does not rewrite a hand-tuned `config.toml` |
| UserPromptExpansion | the nine skill names | `prompt-expansion` | `trust verify`, `gate require <phase> <id>` | `reason` of the blocking decision | trust failure; predecessor gate not passed |
| PreToolUse | `Write\|Edit\|NotebookEdit` | `pre-tool-use` | `doc validate --producer-check`, `story files --check`, `design lint` | `hookSpecificOutput.permissionDecisionReason` | producer mismatch; undeclared file; token violation; a write to the trust store or the pinned framework tree; an internal error the hook cannot evaluate past |
| PreToolUse | `Bash\|PowerShell` | `pre-tool-use` | `doc validate --producer-check` on every path the command appears to write; the metrics-command decision for the two analysis agents | same | a shell write under `.devforgeai/` from a phase that does not own the document; an analysis agent asking to run a command `config.toml` does not name |
| PreToolUse | `Write\|Edit\|NotebookEdit\|Bash\|PowerShell\|Agent` | `trust-check` | `trust verify` and nothing else | same | trust failure |
| PostToolUse | `Write\|Edit\|NotebookEdit`, path under `.devforgeai/` | `post-tool-use` | `doc validate` | `hookSpecificOutput.additionalContext` | no |
| PostToolUse | `Bash\|PowerShell`, command matching a `config.toml` test command | `post-tool-use`, async | `gate check --phase build --partial` | `additionalContext` on the next turn | no |
| Stop | none | `stop` | `gate check --phase <current>`, `handoff` | `systemMessage`, plus `reason` on a FAIL | gate FAIL, under the block budget below |
| SubagentStop | the registered verifier names | `subagent-stop` | `report ingest <agent_type> <last_assistant_message>` | `reason` of the blocking decision | envelope parse failure; trust failure |
| pre-commit (git) | | | `doc validate` on staged `.devforgeai/` files, `context audit` | the hook's stderr | any failure |
| commit-msg (git) | | | a `STORY-nnn` or `ADR-nnn` token in the message | the hook's stderr | missing |
| pre-push (git) | | | `gate check --phase build` for every story in `sprint.yaml` at status `building` | the hook's stderr | any FAIL |

**The four Stop cases.** The dispatcher reads `stop_hook_active` and `session_id` from the payload, scans the documents written since the last Stop and runs the producer check on each, runs the gate, and writes one JSON object to stdout and nothing else. A scan refusal makes the result FAIL whatever the gate said and appears in the `reason` as `DFA-E212 <path>: <message>`: the `PreToolUse` write-tool arm never sees a shell redirection, so the scan is where a document written from the wrong phase through `cat >` is caught. `state.toml` `[stop_hook].blocked_phase` and `blocked_id` are a record of what last blocked and nothing reads them; `blocked_session` alone keys the budget.

*Case A — gate PASS.* Exit 0, `{"systemMessage": <block>}`.

*Case B — gate FAIL, inside the block budget.* Exit 2, one object carrying both keys: `decision: "block"` with a `reason` naming the failing checks and addressed to Claude, and `systemMessage` carrying the twelve-line block addressed to the user. The same `reason` text is copied to stderr, so a schema change upstream degrades to the stderr path rather than to silence.

*Case C — gate FAIL, budget spent.* Exit 0 with the FAIL block in `systemMessage`. The budget is three consecutive blocks, keyed on `session_id` and on nothing else, and it is one counter shared with the trust-failure branch of §8 rather than a counter per branch — a turn that blocks twice on a FAIL and once on a trust failure has spent it: a changed session is a different chain, and a changed phase or subject is not. Keying it on the subject as well is what the harness's own loop protection exists to prevent — Claude continues, the turn runs `phase set` and moves to the next story, the pair changes, the budget resets, and the chain never ends. The subject is exactly the thing a continuation can change, so it cannot be part of the guard, and `stop_hook_active` is not consulted here for the same reason. The harness's own ceiling is eight consecutive blocks, at which it overrides the hook and shows nothing, so the framework's budget ends the chain first and the last Stop inside it is the one that renders the handoff. The gate runs on the continuation too: Claude has been working on the failing checks since the last block, so re-running is the only way to notice that the work now passes.

*Case D — gate SEND BACK.* Exit 0, `{"systemMessage": <block>}` and no blocking decision. The next step is a command the user types, and holding the model in the turn cannot produce it.

**Cross-cutting skills.** Design and Reflect leave `[current].phase` where they found it, so one Stop has two blocks to render. The cross-cutting skill's own `handoff` call site writes `[last_cross]` in `state.toml` (`phase`, `id`, `turn`); the Stop renders `handoff --phase <last_cross.phase> --id <last_cross.id>` and then `handoff --phase <current>`, joins the two with a blank line into one `systemMessage`, and clears `[last_cross]`, so a second Stop in the same session prints one block. The joined string is capped at 10,000 characters.

**SessionStart.** Plain stdout, exit 0, carrying the last handoff: this event adds stdout to Claude's context, which is the audience for a session-opening block.

## 8. Trust model (fixed)

- The binary's SHA-256 is pinned in `~/.devforgeai/trust.toml` by a human running `devforgeai trust pin --framework <framework root>` in a terminal outside Claude Code. `trust pin` refuses to run when `CLAUDECODE` or `CLAUDE_CODE_ENTRYPOINT` is set in the environment, so a session cannot pin the binary it is running. `--framework` is required: it is the path `init --from` defaults to.
- Every hook runs `devforgeai trust verify` first. A trust failure blocks on `PreToolUse`, `Stop`, `SubagentStop`, and `UserPromptExpansion`, through exit 2 and a decision object; the `Stop` branch spends the same three-block budget §7 gives a gate FAIL, because two branches each holding three blocks is six, and the harness overrides at eight; on `SessionStart` and `PostToolUse` no exit code blocks and the hook emits `systemMessage` so the failure is visible. `[last_gate].result` takes `TRUST_FAIL`, which is a record rather than the channel the refusal depends on — the binary under suspicion is the one that renders the handoff.
- `trust pin` is a human action taken outside Claude Code, and two mechanisms hold it there: the binary refuses when `CLAUDECODE` or `CLAUDE_CODE_ENTRYPOINT` is set, and the `PreToolUse` shell arm denies any `Bash` or `PowerShell` command whose text contains `trust pin` while a session is active. The environment guard alone is bypassable from inside a shell by stripping the variable; a `PreToolUse` deny fires before the process starts and in every permission mode.
- The fail-closed set is three surfaces, one message: the `PreToolUse` `trust-check` arm on `Write|Edit|NotebookEdit|Bash|PowerShell|Agent` returns `permissionDecision: deny`, which fires in every permission mode including `bypassPermissions` and which no other hook's allow can override; the Stop hook blocks with the pin command and runs no gate; and `UserPromptExpansion` on the nine skill names blocks before the skill body enters context. An unpinned session can read and reason and can change nothing.
- `cli/` source in the framework repo carries `cli/REVISION` (git SHA or content digest) and `cli/DIGEST` (SHA-256 of the release binary). `trust pin` records both. `trust verify` compares the running binary to `DIGEST` and refuses if `cli/` source digest differs from `REVISION` while a Claude session is active.
- No subcommand modifies `gates.toml` thresholds downward. `gates.toml` is validated against a schema whose minimums are compiled into the binary.

## 9. Python evaluation (mandatory per skill)

Each skill ships three eval artifacts. A skill without all three does not build.

1. `evals/evals.json` in skill-creator format (`references/schemas.md` in the skill-creator plugin): at least 6 evals, each with `prompt`, `expected_output`, `expectations[]` (3 to 6 verifiable statements each). Used by skill-creator's trigger tests and grader agent.
2. `evals/cases.jsonl`: one JSON object per line, `{ "id": str, "prompt": str, "timeout": int, "setup": {"files": {path: content}, "git": bool | {"worktrees": [{"path": str, "branch": str}]}}, "answers": {question header: label | [label]}, "preflight": [str | {"command": str, "exit": int | "any", "code": str, "forbid_code": str}], "expect": {"grader": str, "args": {}} }`. At least 6 cases.

   `answers` pre-seeds the skill's questions, keyed on a question's fixed header. `setup.git` makes the workspace a git work tree for a case whose subject needs one, either `true` for a plain tree with one commit or an object registering worktrees up front. `timeout` is seconds and overrides `--timeout` for that case; the phases that write six or more documents need it, and Constitute carries 1500. A `setup.files` value of `FIXTURE:<name>` is read from `<skill>/evals/fixtures/<name>`, which is the one place a case reads a file the runner does not copy into the workspace — the fixtures directory stays outside every workspace so the model under test cannot read the expected output.

   `preflight` names the CLI calls the skill makes before it writes anything, each run against a throwaway copy of the materialised workspace. A bare string is a command required to exit 0. The object form says what else the entry is for: `exit` is the code required, or `"any"` when the exit is not the point; `code` is a `DFA-` code the call has to raise, which is how a case asserts the refusal it exists to catch; `forbid_code` is a code the call must not raise, which is how a case asserts that a path is open without pinning the exit. `exit` defaults to 0 when the object omits it.

   At least 2 cases exercise the SEND BACK path, except for a phase that has no such path. Explore is phase 0 and has nothing upstream to send back to; Reflect emits no send-back and receives none, since no gate carries `send_back_to = "reflect"`. Each of those two carries one documented stop-path case instead — `ex-09-blocked-prefix-exhausted` and `rf-08-blocked-bad-window` — so the floor is a case that measures the run stopping for a stated reason, rather than a send-back case written against a path the phase does not have.
3. `evals/graders.py`: pure functions `def <grader>(workspace: str, transcript: str, args: dict) -> tuple[bool, str]`. No network, no subprocess to an LLM, no randomness. Each returns `(passed, evidence)`.

Shared runner `evals/runner/run_jsonl.py` (framework-level, specified in `specs/01-cli.md` `## Evals`) invokes `claude -p` per case in a temp workspace, then calls the grader, then appends one `devforgeai/eval-result/2` line to `results.jsonl`. Graders decide pass/fail. Nothing in Rust is evaluated by Python; nothing in a phase gate is decided by Python.

Three properties of the runner that a skill author writes cases against. The prompt travels on stdin and the output format is `stream-json --verbose`, so a grader sees tool calls, tool inputs and tool results rather than the final string alone. The workspace carries the framework's own hook block by default, so the producer check, the gates and the Stop block behave as they do in a target project; that depends on a real `devforgeai trust pin` made by a human outside Claude Code, because a release binary honours no `DEVFORGEAI_HOME` redirect, and the runner falls back to `--no-hooks` and prints the pin command when `trust verify` fails. A case's `answers` map is pre-seeded through a `PreToolUse` handler on `AskUserQuestion` that returns `allow` with an `updatedInput` carrying the answers. The tool exists under `claude -p` only when a permission host does: measured on `claude 2.1.268`, a run with no host lists 33 tools and no `AskUserQuestion`, and the same run with `--mcp-config <workspace>/.claude/mcp-permissions.json --permission-prompt-tool mcp__dfa-permissions__approve` lists 36 including it. The runner therefore ships `evals/runner/permission_host.py`, a stdio MCP server whose one tool allows every request unchanged, and names it for any case that seeds answers; a case that seeds none runs under `--permission-prompts none` instead, so a question it was never meant to reach is denied rather than left hanging.

What a case seeds, and what it inherits. `.devforgeai/state.toml`, `config.toml`, and `gates.toml` come from `cli_defaults()`, which runs `devforgeai init` once per process in a throwaway directory outside every workspace and keeps the three files it wrote; a case seeds only what it cares about and inherits the rest, so a fixture cannot drift from what an install produces. A seeded file whose content opens with `#!` is written executable, because such a file is a command the run is expected to execute — `config.toml` names `./ci/test`, and Build's red-green loop turns on its exit code.

The `preflight` key and the `--preflight` mode. A case may name, under `preflight`, the CLI calls its skill makes before it writes anything: the phase's `phase set`, the id allocation before it, the `worktree ensure` a Build run opens with. `--preflight` runs each against a throwaway copy of the materialised workspace — throwaway because those calls mutate it — and requires exit 0 from every one. A non-zero exit there is the skill stopping on a defect the case shipped rather than on the behaviour the case exists to measure, and it costs nothing to find out. A TOML parse is not enough on its own: the binary refuses a `config.toml` with no `generated_at` that parses perfectly, and this is where that refusal surfaces.

`--permission-mode` is `bypassPermissions`. Headless has no approval surface, so anything the mode does not cover is denied with no one to ask, and `acceptEdits` covers Write and Edit alone: a compound shell command matches no `Bash(devforgeai *)` rule, and neither does the project's own test command. The workspace is a throwaway the runner creates and deletes, the child sees no user configuration, and the framework's hooks still enforce inside it when a pin exists — a `PreToolUse` deny fires in every permission mode. Only the prompt is gone.

Timeouts and the run statuses. `--timeout` defaults to 900 seconds, above the 828 a measured full-phase case took; a case may carry its own `timeout` key, which overrides it, and the Constitute cases carry 1500. A run that times out records no cost, because the result event that carries `total_cost_usd` never arrives. The status set is `pass`, `fail`, `error`, `timeout`, and `limit`: `limit` is a run the account's usage limit stopped before it finished, recognised from the error result's text, and it is the sixth column of the summary line beside the other four counts and the cost. `error`, `timeout`, and `limit` each exit the runner 2, because none of them is a measurement.

## 10. Subagent design requirements

Each skill spec includes `## Subagents`, one subsection per subagent the skill invokes, with exactly these fields:

- **name**: kebab-case, unique across the framework.
- **derives_from**: path under `C:\Users\bryan\.claude\agents\` if adapted from an existing agent, else `new`.
- **purpose**: one sentence.
- **tools**: closed list, minimal. Read-only unless the spec justifies a write.
- **model**: `sonnet`, `opus`, or `inherit`, with a one-line reason.
- **input**: what the skill passes in the prompt (fields, and which document IDs).
- **output**: a JSON schema. The skill and the CLI parse this; prose output is a defect.
- **invoked_at**: which workflow step, and whether in parallel with others.
- **registered_verifier**: `yes` if `SubagentStop` ingests its output into a report, else `no`.

Existing agents live at `C:\Users\bryan\.claude\agents\*.md`. Spec authors read the ones relevant to their phase and either adapt (state what changes and why) or replace (state why). The catalog spec (`specs/11-subagent-catalog.md`) consolidates all subagents across skills, resolves name collisions, and lists agents from the existing set that no skill uses.

Rules that hold for every agent in the framework:

1. **No agent asks the user.** Claude Code removes `AskUserQuestion` from every subagent whatever its `tools` list holds. A question belongs to the invoking skill, which asks it in the main conversation and passes the answer in the agent's prompt. No agent file lists the tool and the catalog's permitted-tool list does not name it.
2. **One JSON object per verifier.** A registered verifier prints exactly one object: the `devforgeai/verifier/1` envelope, with the agent's own top-level fields nested under a `payload` key. Never two objects, and never an object beside prose.
3. **`warn` never lowers `passed`.** A finding at `severity: warn` is recorded and does not change the envelope's `passed`.
4. **Review agents report everything.** An agent that reviews reports every finding, including uncertain and low-severity ones, with no cap and no self-filtering. The CLI and the Verify phase filter; an agent told to report only what matters under-reports.
5. **A `description` is a delegation trigger.** One or two sentences: what the agent does and when the owning skill delegates to it. The reasoning and the method live in the body, which loads only when the agent runs.
6. **An agent applies no threshold.** An agent may read a tool's numeric output only to copy it into `payload` verbatim — a complexity figure, a coverage percentage, a count. Comparing that number against a ceiling or a floor is a `gate check` check kind reading `config.toml` and `gates.toml`, never an agent's judgment. This is what rule 1 of §1 means for an agent that runs a measuring command.
7. **Model choice follows the prompting guidance.** An `opus` agent carries no "verify your work" or "double-check" instruction, because the model already does that unprompted and over-verifies when told. A `sonnet` agent states its scope explicitly, because it follows instructions literally.

## 11. Spec document template

Every spec is `specs/<nn>-<name>.md` with this frontmatter and these H2 sections in this order. No other H2s.

```yaml
---
schema: devforgeai-spec/1
doc: <name>
status: draft
produced_by: <agent label>
consumes: [00-conventions]
open_questions: []
---
```

```
## Scope            what this component is and is not; one paragraph each
## Inputs           documents read, with IDs and paths from §5
## Outputs          documents written, with full frontmatter and section list
## Workflow         numbered steps; each step names the actor (model, subagent, CLI, user), the input, the output, and the failure path
## Subagents        per §10
## Command          the skill's frontmatter block and its `!` preamble lines, verbatim from SKILL.md
## CLI calls        every subcommand this component invokes, with exact arguments
## Gate             the gates.toml entry for this phase, verbatim
## Send-back        conditions that produce SEND BACK, the upstream phase, the IDs cited
## Integration      a table with one row per other skill: consumes (doc, IDs), produces for (doc, IDs), sends back to (condition), receives send-back from (condition), shared subagents, state.toml fields read/written. Every one of the nine skills plus the CLI appears as a row, with "none" where there is no relationship and a one-line reason why none is correct.
## Handoff          a filled example of §6 for the PASS case and the SEND BACK case
## Templates        full content of every file under templates/
## Evals            the 6+ evals.json entries, the 6+ cases.jsonl lines, the grader function signatures and logic
## Decisions        every choice this spec made where §1 through §10 were silent; every proposed CLI addition; every blocker
```

Review acceptance criteria applied by the orchestrator: (a) zero matches for the §2 ceremony pattern, scoped by the six clauses and run through `scripts/ceremony_scan.py`, in `## Command`, `## Templates`, and workflow prose; (b) zero ambiguity terms from §2; (c) every artifact has a path; (d) every subagent output has a schema; (e) every CLI call uses a §4 name or is listed in Decisions; (f) both handoff examples fit twelve lines; (g) evals present and the send-back cases exist; (h) the Integration table has a row for every other skill and the CLI, and every consumed document matches what its producer's spec emits.
