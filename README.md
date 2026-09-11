# DevForgeAI

DevForgeAI is a Claude Code framework that runs software delivery as seven phases, each one skill that is its own slash command and writes one typed document the next phase reads. Judgment lives in the skills; enforcement lives in Claude Code hooks, git hooks, and the `devforgeai` Rust CLI, so a phase advances when the CLI says its gate passed and not because a model said so. It installs into any language project, greenfield or brownfield, and reads the language, test runner, and build tool from `.devforgeai/config.toml` rather than naming them anywhere in the skills.

## Layout

```
DevForgeAI\
├── specs\                     one spec per component; the source of every other file here
├── cli\                       Rust crate `devforgeai`, written test-first
├── skills\<skill-name>\       shipped into the target's .claude/skills/
│   ├── SKILL.md
│   ├── references\
│   ├── templates\
│   ├── agents.md              which subagents, invocation order, contracts
│   └── evals\
│       ├── evals.json         skill-creator format
│       ├── cases.jsonl        deterministic cases, one JSON object per line
│       ├── graders.py         pure functions, no network, no LLM
│       └── fixtures\          bytes the cases reference as FIXTURE:<name>
├── agents\<agent-name>.md     shipped into the target's .claude/agents/
├── hooks\settings.hooks.json  merged into the target's .claude/settings.json
├── evals\runner\             run_jsonl.py, answer_hook.py, permission_host.py
├── scripts\ceremony_scan.py   the conventions section 2 rule, implemented once
└── README.md
```

There is no `commands\` directory. A skill and a command file of the same name both produce `/name` and the skill wins, so each entry point is one file: the skill's frontmatter carries the slash name and its `!` preamble lines carry the gate.

## Install into a project

Build the binary first. `cargo build --release` inside `cli\` writes `cli\target\release\devforgeai.exe`; put that directory on `PATH`, or pass the full path to each command below.

```
cd C:\Projects\DevForgeAI
cargo build --release --manifest-path cli\Cargo.toml

# once per machine, in a terminal outside Claude Code:
devforgeai trust pin --framework C:\Projects\DevForgeAI

cd <target project>
devforgeai init [--analyze]
```

`trust pin` records the binary's SHA-256 and the framework path in `~/.devforgeai/trust.toml`. It refuses to run inside Claude Code, and the `PreToolUse` shell hook refuses a command containing `trust pin` there as well, because the pin is what a session is measured against. `--framework` is required: `init --from` defaults to the path the pin recorded, and without it `init` exits `DFA-E111` asking for `--from`.

Until the pin exists every hook fails closed. A `PreToolUse` deny refuses every Write, Edit, NotebookEdit, Bash, PowerShell, and Agent call; `UserPromptExpansion` refuses each of the nine phase commands before the skill loads; and the Stop hook blocks with the pin command and runs no gate. The session can read and reason and can change nothing. `init` runs `trust verify` at the end and, on a failure, prints the pin command as its `Next` line instead of `/explore`.

`init` creates `.devforgeai/`; copies the skills and the agents into `.claude/`, each skill landing at `.claude/skills/<its slash name>/` and every `evals/` subtree left behind; merges the hook block into `.claude/settings.json` and adds `Bash(devforgeai *)` and `PowerShell(devforgeai *)` to `permissions.allow`; appends `.explore-prototype/` and `.devforgeai/state.toml` to the target's `.gitignore`; writes a DevForgeAI section into the target's `CLAUDE.md` between `<!-- devforgeai:begin -->` and `<!-- devforgeai:end -->`, replacing the span on a re-run and leaving every line outside it alone; installs the `pre-commit`, `commit-msg`, and `pre-push` git hooks; and runs `stack detect` to write `config.toml`. `--analyze` reads a brownfield tree and drafts the six context files for the Constitute phase to finish. The target ends up with:

```
<target>\
├── .claude\{skills,agents}\            copied from the framework, minus every evals\ subtree
├── .claude\settings.json               hooks and the two permission rules merged in
├── CLAUDE.md                           a DevForgeAI section between markers
├── .devforgeai\
│   ├── config.toml                     stack, thresholds, paths
│   ├── gates.toml                      the one place pass criteria live
│   ├── state.toml                      current phase, active ids, last gate result
│   ├── explore\, requirements.yaml, context\, adr\, stories\, ui-specs\
│   └── reports\, releases\
└── .git\hooks\{pre-commit,commit-msg,pre-push}
```

## Phase commands

Each command takes the arguments the previous handoff printed, and the Stop hook emits the next handoff in its JSON. The model writes no part of that block.

| Order | Command | Skill | Emits |
|---|---|---|---|
| 0 | `/explore <idea>` | exploring-ideas | `explore/brief.md`, `explore/decision.yaml` |
| 1 | `/discover <IDEA-nnn>` | discovering-requirements | `requirements.yaml` |
| 2 | `/constitute <IDEA-nnn>` | establishing-context | `context/*.md`, `adr/ADR-nnn.md` |
| 3 | `/plan <EPIC-nnn>` | planning-work | `stories/STORY-nnn.md`, `stories/sprint.yaml` |
| 4 | `/build <STORY-nnn>` | implementing-stories | `reports/STORY-nnn-build.yaml` |
| 5 | `/verify <STORY-nnn>` | validating-quality | `reports/STORY-nnn-qa.yaml` |
| 6 | `/release <vX.Y.Z>` | releasing-software | `releases/vX.Y.Z.yaml` |

Two commands cut across the sequence: `/design <REQ-nnn>` (designing-interfaces) writes `brand/tokens.json` and `ui-specs/UI-nnn.md` for Plan and Build, and `/reflect <YYYY-MM-DD>` (improving-framework) writes `reports/reflect-<date>.yaml` as recommendations. A phase that finds a defect upstream emits a SEND BACK handoff citing IDs; the upstream command reopens only those IDs with `--remedy <ID>,<ID>`, and the returning command continues with `--resume`.

## Run a skill's evals

Evals run from this repository, not from a target project: `--skill skills/<name>` is a path here, and a target holds `.claude/skills/<name>/` with no `evals/` in it and no runner.

`evals/runner/run_jsonl.py` builds one temporary workspace per case, installs the skill under test at `.claude/skills/<its slash name>/` with the agents it names, puts the release binary on `PATH`, writes the framework's own hook block into `<workspace>/.claude/settings.json`, runs `claude -p` there, then calls the case's grader from the skill's `graders.py`. Hooks-on needs a real `trust pin`: the runner checks once before the suite and falls back to `--no-hooks` with the pin command printed when the check fails.

```
python evals/runner/run_jsonl.py --skill skills/exploring-ideas --dry-run
python evals/runner/run_jsonl.py --skill skills/exploring-ideas
python evals/runner/run_jsonl.py --skill skills/implementing-stories --case BLD-03 --keep
```

`--cases` defaults to `<skill>/evals/cases.jsonl`, `--graders` to `graders.py` beside it, and `--out` to `results.jsonl` beside it, appended one line per case. Other arguments: `--model` (default `sonnet`), `--timeout` seconds (default 900, above the 828 seconds a measured full-phase case took), `--jobs`, `--filter <glob>`, `--case <id>` (repeatable), `--workdir`, `--logdir`, `--keep`, `--claude-bin`, `--devforgeai-bin`, `--framework-root`, `--claude-config isolate|inherit`, and `--hooks` / `--no-hooks`. `--dry-run` materialises the workspaces, prints the invocation it would make, and keeps them for inspection. `--preflight` runs each case's declared `preflight` CLI calls against its materialised workspace and requires exit 0 from every one, which is how a fixture missing a `state.toml` field, a predecessor report, or a `--epic` is caught before a paid run. A case may carry its own `timeout` key, which overrides `--timeout`. The summary line counts `pass`, `fail`, `error`, `timeout`, and `limit` — `limit` being a run the account's usage limit stopped, which says nothing about the case — and then the wall clock and the summed cost. Exit codes: 0 every case passed, 1 a case failed, 2 a case errored, timed out, or hit the limit, 3 a usage error. Graders decide pass and fail; no phase gate reads any of it.

Every skill ships six or more cases, at least two of them on the SEND BACK path, and a skill without `evals.json`, `cases.jsonl`, and `graders.py` does not build.

The runner's own tests use a fake `claude` on PATH and reach no network:

```
python -m unittest evals/runner/test_runner.py
```

## Specs

`specs/00-conventions.md` is the contract every other document conforms to: the eight rules, forbidden content, the repository layout, the CLI surface, the document contract, the handoff format, the hook table, the trust model, and the eval requirements. `specs/01-cli.md` covers the `devforgeai` binary, `config.toml`, `gates.toml`, `state.toml`, the hooks, and the eval runner contract. `specs/02-explore.md` through `specs/10-reflect.md` cover one phase each, `specs/11-subagent-catalog.md` consolidates the subagents, and `specs/BUILD-BRIEF.md` says what a skill build produces and how it is checked. `scripts/ceremony_scan.py` implements the conventions §2 rule and is what the brief's verification step runs.
