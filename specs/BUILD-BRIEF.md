---
schema: devforgeai-spec/1
doc: build-brief
status: authoritative
produced_by: orchestrator
consumes: [00-conventions, 11-subagent-catalog]
open_questions: []
---

# Build brief for skill and agent builders

Read `specs/00-conventions.md` first. Then read the spec you are building from, in full. This brief tells you where files go, what each file contains, and how you verify your output before reporting. It does not repeat the spec; the spec is the source of every piece of content.

## 1. What a skill build produces

For skill `<name>` from `specs/<nn>-<phase>.md`:

```
skills/<name>/
├── SKILL.md
├── references/            one file per topic the spec's ## Workflow defers detail to
├── templates/             every file under the spec's ## Templates, byte-identical
├── agents.md              from the spec's ## Subagents and the catalog's agents.md template
└── evals/
    ├── evals.json         the spec's ## Evals entries, skill-creator schema
    ├── cases.jsonl        the spec's cases, one JSON object per line
    ├── graders.py         the spec's grader functions, implemented
    └── fixtures/          every FIXTURE:<name> the cases reference, with the exact bytes the spec's digests were computed over
agents/<agent>.md          one file per subagent the spec owns (owned_by in the catalog), in the catalog's agent file format
```

There is no `commands/` directory. A skill and a command file of the same name both produce `/name` and the skill wins, so the entry point is the skill's own frontmatter and its `!` preamble lines.

## 2. SKILL.md

Frontmatter, and no key outside this list:

| Key | Value |
|---|---|
| `name` | the slash name — `explore`, `discover`, `constitute`, `plan`, `build`, `verify`, `release`, `design`, `reflect` |
| `description` | one paragraph: what the skill does, then the phrases and contexts that trigger it, written so the model reaches for it whenever the phase's command or its artifacts are in play; specific, not pushy with capitals |
| `argument-hint` | the argument forms the spec's `## Entry` table enumerates |
| `allowed-tools` | a closed list opening with `Bash(devforgeai:*), PowerShell(devforgeai:*)` |
| `disable-model-invocation` | `true` on eight skills; absent on `designing-interfaces` |

Directly under the frontmatter, the `!` preamble lines from the spec's `## Command`, byte-identical, `gate require` first where the spec has one. They run before the body loads, and a non-zero exit aborts the whole invocation, which is the gate behaviour. No preamble allocates an id.

The directory name stays the long one — `skills/implementing-stories/` — because `produced_by` and the producer check resolve against it; `devforgeai init` installs the skill at `.claude/skills/<frontmatter name>/`, which is what makes `/build` the command in a target project.

Body, H2 sections in this order, headings verbatim:

1. `## Entry` — the command that invokes this skill, the arguments it accepts, and what the preamble already loaded into context. From the spec's ## Command and ## Inputs.
2. `## Workflow` — the spec's ## Workflow steps, rewritten in the imperative for the model executing them. Each step keeps its actor, input, output, and failure path. Steps performed by the CLI or a hook are stated as "the CLI does X" so the model does not attempt them.
3. `## Subagents` — a table: name, when invoked, what to pass, what comes back (schema name). Point to `agents.md` for contracts.
4. `## Documents` — the documents this skill writes, their paths, and a pointer to the template for each. Frontmatter rules from conventions §5 stated once.
5. `## Send-back` — the conditions from the spec's ## Send-back, and the exact command the handoff will print. The model composes no part of the handoff and writes nothing at the close of a phase; state that the Stop hook emits the block.
6. `## Remedy and resume` — what changes on `--remedy` and `--resume`, from the spec.
7. `## References` — one line per file under `references/` saying when to read it.

Under 500 lines. No section titled anything else. Explain why a step matters where the spec gives the reason; do not add reasons the spec does not give.

## 3. What does not go in a SKILL.md or an agent file

Conventions §2 in full, under the six-clause scoping rule that `scripts/ceremony_scan.py` implements. Concretely: no sentence matching the ceremony pattern in instruction prose; no numbered list the model walks over its own output; no instruction to run tests or read coverage numbers (the CLI and hooks do that); no status transitions (`phase set` does that); no handoff text (the Stop hook emits it, and no skill writes any part of the close of a phase); no language, test runner, package manager, linter, or scanner name.

What the scoping rule releases, so a builder does not contort prose to dodge a hit that is not one: a fenced block, a YAML frontmatter or scalar value, a backticked span, a table cell in an enum or path column, a path segment, and a heading. Both halves of the pattern match case-insensitively, so the case a word carries releases nothing: an ordinary lower-case `always` in instruction prose is a hit, and what releases the ordinary English uses of those words is the scoping above — a backticked span, an enum cell, or a YAML scalar is out of scope whichever case it carries. Conventions §2 states the rule; this brief restates nothing of it. Templates are copied from the spec byte-identical, so the spec author already cleared them, and a document the framework produces about a target project — a context file, an ADR, a requirements record, a story, a UI spec, a report — is outside the rule entirely.

## 4. Agent files

Format from `specs/11-subagent-catalog.md` `## Templates`, and the six rules that follow it. In short: frontmatter `name`, `description` (one or two sentences — what it does and when the owning skill delegates to it, nothing more), `tools` (the spec's closed list), `model`; body H2s `## Input`, `## Output`, `## Workflow`. `AskUserQuestion` appears in no agent file, because Claude Code removes it from every subagent whatever the list holds; a question belongs to the owning skill, which asks it and passes the answer in as a field. A registered verifier's `## Output` shows exactly one JSON object, the `devforgeai/verifier/1` envelope with the agent's own fields under `payload`. A `warn` finding never lowers `passed`, and a review agent reports every finding it has rather than filtering for the caller. The catalog reconciles names; if the catalog renames an agent you built, the rename is a file move plus a find-and-replace, which the orchestrator does.

## 5. Evals

- `evals.json`: copy the spec's entries into the skill-creator schema (`skill_name`, `evals[].id`, `prompt`, `expected_output`, `files`, `expectations`).
- `cases.jsonl`: copy the spec's lines. Every `FIXTURE:<name>` must exist under `evals/fixtures/`.
- `graders.py`: implement every function the spec names with the exact signature `def <name>(workspace: str, transcript: str, args: dict) -> tuple[bool, str]`. Standard library only. No network, no subprocess, no randomness. Parse YAML frontmatter and simple YAML documents with a small in-module reader (the runner environment has PyYAML available, but graders do not import it, so a missing package cannot fail a grade).
- Do not run the evals. They need `claude -p` and the compiled CLI; the orchestrator runs them after the CLI build.

What the runner does with what you wrote, so a case is written against the real thing. It installs the skill at `<workspace>/.claude/skills/<frontmatter name>/` minus `evals/` and `agents/`, copies the agents the skill's `agents.md` names, and puts the release binary on `PATH`, because every preamble opens with `` !`devforgeai …` `` and a non-zero exit there aborts the invocation. The prompt travels on stdin and the output format is `stream-json --verbose`, so a grader sees tool calls and tool inputs rather than the final string alone. The workspace carries the framework's own hook block by default, which needs a real `devforgeai trust pin` made by a human outside Claude Code; without one the runner falls back to `--no-hooks` and prints the pin command. A case may carry `answers`, pre-seeded through a `PreToolUse` handler on `AskUserQuestion` with a stdio permission host supplied so the tool is in the `-p` tool set at all, and `setup.git`, which makes the workspace a git work tree and can register worktrees up front. A grader may read its own skill's `evals/fixtures/`, which sits outside the workspace.

## 6. Verification before you report

Run each of these and include the output lines in your report.

```
python -c "import ast,sys; ast.parse(open('skills/<name>/evals/graders.py').read())"
python -c "import json; json.load(open('skills/<name>/evals/evals.json'))"
python -c "import json; [json.loads(l) for l in open('skills/<name>/evals/cases.jsonl') if l.strip()]"
python scripts/ceremony_scan.py skills/<name> agents/<each agent>.md
python <skill-creator>/scripts/quick_validate.py skills/<name>
```

`ceremony_scan.py` is the one place the §2 pattern and its six clauses are written; it exits 0 on a clean scan and 1 with `file:line: token` on each hit. It replaces the inline grep this brief used to carry, because a line-oriented grep cannot express the fenced-block and YAML-scalar exclusions and produced hits a builder had no rule for dispositioning.

`quick_validate.py` lives at `C:\Users\bryan\.claude\plugins\marketplaces\claude-plugins-official\plugins\skill-creator\skills\skill-creator\scripts\quick_validate.py`. It validates against the portable Agent Skills field set, which does not include `argument-hint` or `disable-model-invocation`, so it rejects a conformant DevForgeAI SKILL.md on those two keys alone. Run it over a copy with those two lines stripped; a pass there plus a clean `ceremony_scan.py` is the check. Fix anything else it reports.

Diff every template against the spec block and confirm zero differences, and diff the SKILL.md frontmatter and preamble against the spec's `## Command` block the same way.

## 7. Report format

Reply with: the list of files written with line counts; the verification output (grep result, validator result, parse results); every place the spec was ambiguous or contradictory and what you did; and nothing else.
