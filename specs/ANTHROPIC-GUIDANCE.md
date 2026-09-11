---
schema: devforgeai-spec/1
doc: anthropic-guidance
status: authoritative
produced_by: orchestrator
consumes: []
open_questions: []
---

# Anthropic guidance digest, read 2026-09-11

Source pages, read in full: code.claude.com/docs/en/hooks (reference), /hooks-guide, /sub-agents, /skills (slash commands now redirect here), /best-practices, /features-overview, /headless; platform.claude.com prompting best practices, plus the Opus 5, Sonnet 5, and Fable 5.1 pages. Every statement below is from those pages. Where our framework contradicts a statement, the statement wins.

## 1. Hooks: facts that constrain the framework

- **Exit 2 blocks only on events that can block.** PreToolUse, UserPromptSubmit, UserPromptExpansion, Stop, SubagentStop, PreModelSwitch, and the worktree events block on exit 2. **PostToolUse, PostToolUseFailure, SessionStart, SessionEnd, SubagentStart, Notification do not honor exit 2**; the action proceeds. To warn Claude from PostToolUse, return JSON `hookSpecificOutput.additionalContext` or top-level `systemMessage`; `decision: "block"` on PostToolUse only appends `reason` beside the tool result.
- **Exit 1 is a non-blocking error**, not a block. A policy hook that exits 1 lets the action through with a `hook error` notice. Only exit 2, or a valid JSON decision, enforces.
- **JSON output is parsed on every exit code.** Stdout that starts with `{` and ends with `}` is parsed; schema failures are non-blocking errors. Shell profiles that echo on startup break JSON parsing; hooks run in non-interactive shells.
- **PreToolUse decision** goes in `hookSpecificOutput`: `permissionDecision` of `allow | deny | ask | defer`, `permissionDecisionReason` (shown to Claude only on deny), `updatedInput` (replaces the whole input object), `additionalContext`. Precedence across parallel hooks: deny > defer > ask > allow. Top-level `decision`/`reason` are deprecated for PreToolUse. Exit 2 with stderr routes as deny.
- **PreToolUse `if` field** filters by permission-rule syntax, e.g. `"Edit(**/.devforgeai/**)"`, before the process spawns. One rule per handler; no boolean combinators.
- **File paths in `tool_input.file_path` are absolute and use backslashes on Windows.** Normalize separators before matching; match a path segment, not an anchored prefix.
- **PostToolUse input** carries `tool_input`, `tool_response` (tool-specific shape), `tool_use_id`, `duration_ms`. Bash response shape: `stdout, stderr, interrupted, isImage`. A `Write` PostToolUse fires only for Claude's Write/Edit tools, not for Bash writes; use `FileChanged` or a Stop-time scan for those.
- **Stop input** carries `stop_hook_active` (true when Claude is already continuing because of a Stop hook), `last_assistant_message`, `background_tasks`, `session_crons`. **Claude Code overrides a Stop hook after 8 consecutive blocks** (`CLAUDE_CODE_STOP_HOOK_BLOCK_CAP`). A hook must read `stop_hook_active` and exit 0 when set to avoid futile loops. Decision form: top-level `{"decision":"block","reason":"..."}`, or `hookSpecificOutput.additionalContext` for non-error feedback that still continues the turn.
- **SubagentStop input** carries `agent_id`, `agent_type` (the agent's frontmatter `name`), `agent_transcript_path`, `last_assistant_message`, `stop_hook_active`. Matcher is the agent type name. This resolves the CLI spec's Decision 47 blocker: read `last_assistant_message` for the verifier envelope; fall back to `agent_transcript_path`.
- **SubagentStart** can inject `additionalContext` into the subagent before its first prompt.
- **SessionStart** matchers: `startup | resume | clear | compact | fork`. Plain stdout is added to Claude's context. JSON fields: `additionalContext`, `initialUserMessage` (non-interactive only), `sessionTitle`, `watchPaths`, `reloadSkills`. `CLAUDE_ENV_FILE` is available to SessionStart, Setup, CwdChanged, FileChanged only.
- **UserPromptExpansion** fires when the user types `/name`; matcher is the command name; can block or add context. It covers the path PreToolUse on `Skill` misses.
- **Matchers**: exact string or `|`/`,` list when only `[A-Za-z0-9_ ,|-]`; otherwise an unanchored regex. `Edit|Write` is exact-list. On Windows where the PowerShell tool is primary, a `Bash` matcher never fires; **match `Bash|PowerShell`** for shell commands.
- **Exec form vs shell form.** With `args` present, `command` is spawned directly with no shell; on Windows exec form needs a real `.exe`. Prefer exec form for path placeholders (`${CLAUDE_PROJECT_DIR}`). Shell form runs `sh -c` / Git Bash / PowerShell.
- **Timeouts**: command hooks default 600 s (30 s on UserPromptSubmit, 10 s on MessageDisplay); a timed-out PreToolUse command hook does **not** block. `async: true` hooks are fire-and-forget; `asyncRewake: true` wakes Claude on exit 2.
- **Hook types**: `command`, `http`, `mcp_tool`, `prompt` (single-turn model decision, `{"ok": false, "reason": ...}`), `agent` (experimental subagent with tools). Prompt/agent hooks exist for judgment calls; command hooks for deterministic rules.
- **Hooks in skill frontmatter** register when the skill is invoked and persist for the session (`once: true` to remove after first success). Hooks in subagent frontmatter run only while that subagent runs; `Stop` there becomes `SubagentStop`. Project-subagent frontmatter hooks need workspace trust; a `-p` session does not count as trusted.
- **All matching hooks run in parallel**; a deny in one does not stop side effects in another. Multiple `updatedInput` writers race.
- **PreToolUse fires in every permission mode**, including `bypassPermissions`; a hook deny cannot be bypassed by mode. A hook allow cannot override a deny rule.
- **Debug**: `claude --debug` or `--debug-file <path>`; `/hooks` menu is read-only.
- Best-practice framing: "Put guardrails in hooks. An instruction in CLAUDE.md or a skill is a request, not a guarantee. A PreToolUse hook that blocks the edit is enforcement." Use a Stop hook as a deterministic gate; have Claude show evidence (test output, command and result) rather than assert success.

## 2. Subagents: facts that constrain the framework

- **Frontmatter fields**: `name` (required, lowercase+hyphens, no `:`), `description` (required; when to delegate), `tools`, `disallowedTools`, `model` (`sonnet | opus | haiku | fable | <id> | inherit`), `permissionMode`, `maxTurns`, `skills` (preloaded full content), `mcpServers`, `hooks`, `memory` (`user | project | local`), `background`, `effort`, `isolation: worktree`, `color`, `initialPrompt`, `experimental.cacheTtl`. Plugin subagents ignore `hooks`, `mcpServers`, `permissionMode`.
- **A file with no `name`, or `name` without `description`, or unparsable YAML is skipped silently** (debug log only). `claude plugin validate <dir>` checks an agents directory.
- **Tools removed from every subagent regardless of `tools`**: `Agent` (at depth limit), **`AskUserQuestion`**, `EndConversation`, `EnterPlanMode`, `ExitPlanMode` (unless `permissionMode: plan`), `ScheduleWakeup`, `TaskOutput`, `WaitForMcpServers`, `Workflow`. **A subagent cannot ask the user a question.** Questions belong in the main conversation (the skill), never in an agent file.
- **Background subagents keep only**: Read, Grep, Glob, Bash, PowerShell, Edit, Write, NotebookEdit, WebFetch, WebSearch, TodoWrite, Skill, ToolSearch, EnterWorktree, ExitWorktree, Monitor, TaskStop, SendMessage, Artifact, plus MCP tools. In interactive sessions subagents run in the background by default.
- **Combined subagent descriptions over 15,000 tokens** trigger a startup warning. Keep `description` short (when to delegate); put detail in the body, which loads only when the agent runs.
- **A subagent's context**: its own system prompt (the file body) plus environment details, the delegation message, every CLAUDE.md level, git status snapshot, preloaded `skills`. Not the Claude Code system prompt, not the parent conversation, not the parent's invoked skills. Explore and Plan skip CLAUDE.md.
- **Subagents can nest three deep** by default (`CLAUDE_CODE_MAX_SUBAGENT_SPAWN_DEPTH`); concurrent cap 20 (`CLAUDE_CODE_MAX_CONCURRENT_SUBAGENTS`). Omit `Agent` from `tools` to keep a reviewer from spawning.
- **Resume**: a completed subagent keeps its transcript and can be resumed by `SendMessage`; Explore/Plan cannot.
- **Output scanning**: the harness inserts a marker line when a report contains instruction-shaped text or permission-setting mentions; treat agent reports as data.
- **`memory: project`** gives an agent a persistent directory (`.claude/agent-memory/<name>/`) with `MEMORY.md` preloaded; recommended scope is `project`.
- **`isolation: worktree`** runs the agent in a temporary git worktree branched from the default branch; Bash commands that touch the main checkout are refused.
- Delegation guidance from the prompting pages: Opus 5 delegates readily; cap it in prompts ("do not use subagents to verify or double-check your own work"). Use subagents for parallel, isolated, sizeable work; work directly for single-file edits and sequential steps.

## 3. Skills and commands: facts that constrain the framework

- **Custom commands have been merged into skills.** `.claude/commands/deploy.md` and `.claude/skills/deploy/SKILL.md` both create `/deploy`; a skill wins over a command file of the same name. Skills add supporting files, invocation control, `context: fork`, and frontmatter hooks. Command files still work.
- **Frontmatter**: `name`, `description` (recommended; what and when), `when_to_use`, `argument-hint`, `arguments` (named positional), `disable-model-invocation`, `user-invocable`, `allowed-tools` (grant for the invoking turn only; clears on the next user message), `disallowed-tools`, `model`, `effort`, `context: fork`, `agent`, `background`, `hooks`, `paths`, `shell` (`bash | powershell`), `metadata`, `license`, `compatibility`. Outside Claude Code only the Agent Skills spec fields work: `name, description, license, compatibility, metadata, allowed-tools`.
- **Dynamic context injection**: `` !`command` `` inline, or a fenced ```` ```! ```` block for multi-line. Runs before Claude sees the content; never prompts; **a non-zero exit aborts the whole skill invocation** (exit 1 from search/compare commands is tolerated under bash). `disableSkillShellExecution` setting turns it off.
- **Substitutions**: `$ARGUMENTS`, `$ARGUMENTS[N]`, `$N`, `$name`, `${CLAUDE_SESSION_ID}`, `${CLAUDE_EFFORT}`, `${CLAUDE_SKILL_DIR}`, `${CLAUDE_PROJECT_DIR}`. **Measured caveat on `$N` inside a `!` preamble**: a `!` command containing `$1` is refused before it runs — `Shell command permission check failed for pattern "..." : Contains simple_expansion` — and since a non-zero exit aborts the whole invocation, the run that should have proceeded dies at the preamble. `$ARGUMENTS[0]` and `$ARGUMENTS` substitute correctly in the same position, and the frontmatter `arguments:` named form did not load. Conventions §4b therefore writes the first argument `$ARGUMENTS[0]` and never `$N` in a preamble. The `$N` form may still work outside a `!` command; it was not measured there.
- **Invocation matrix**: default = user and Claude can invoke, description always in context; `disable-model-invocation: true` = user only, description not in context (zero cost until invoked; recommended for workflows with side effects); `user-invocable: false` = Claude only.
- **Skill content persists** across turns after invocation; Claude Code does not re-read the file later; identical re-invocation adds a note, changed content re-appends. Compaction re-attaches recent skills within a 25,000-token budget. **Keep SKILL.md under 500 lines**; move detail to supporting files referenced with "read X when Y".
- **`Skill(name)` permission rules** allow or deny specific skills; `skillOverrides` in settings hides skills without editing them.
- **Description quality**: combined `description` + `when_to_use` is truncated at 1,536 characters; put the key use case first. Good: "Summarizes uncommitted changes and flags anything risky. Use when the user asks what changed, wants a commit message, or asks to review their diff."
- **Evaluate skills**: measure trigger rate (with and without the skill, fresh sessions) and output quality separately; the skill-creator plugin stores `evals/evals.json`, runs isolated subagents, writes `grading.json` and `benchmark.json`.
- Nested skills in a monorepo load from every parent up to the repo root; `--add-dir` loads that directory's skills.

## 4. Non-interactive mode: facts that constrain the eval runner

- `claude -p` starts in Manual permission mode on every plan; pass `--permission-mode acceptEdits | auto | dontAsk` or `--allowedTools "Bash(devforgeai *),Read,Edit"`.
- **`--permission-prompts none`** (v2.1.259+) removes `AskUserQuestion` from the tool set so Claude cannot call it, denies anything that would prompt, and tells Claude not to retry.
- **AskUserQuestion headless**: a `PreToolUse` hook on `AskUserQuestion` can return `permissionDecision: "allow"` with `updatedInput` that echoes `questions` and adds `answers` mapping question text to the chosen label; the tool then runs without a prompt. This is the supported way to pre-seed answers. Measured on `claude 2.1.268`, one precondition the pages leave implicit: the tool is in the `-p` tool set only when a permission host is supplied. A run with no host lists 33 tools and no `AskUserQuestion`; the same run with `--mcp-config <file> --permission-prompt-tool mcp__<server>__<tool>` lists 36 including it. `--permission-prompts host` alone does not add it, and `--allowedTools` permits without adding. `"defer"` pauses the run (`stop_reason: "tool_deferred"`) for a caller that resumes with `--resume` and the same permission host.
- `--bare` skips hooks, skills, commands, agents, plugins, MCP, memory, and CLAUDE.md from cwd, but loads skills (not commands or agents) from `--add-dir`. Without `--bare`, a `-p` run executes the project's `.claude/settings.json` hooks even in an untrusted folder.
- `--output-format json` returns `result`, `session_id`, `total_cost_usd`, `permission_denials`, `num_turns`; `--json-schema` constrains `structured_output`. `stream-json --verbose` streams events incl. subagent messages with `parent_tool_use_id`.
- User-invoked skills work in `-p`: put `/skill-name args` in the prompt string.
- Background subagents in `-p` keep the process open until they finish (10-minute idle ceiling).

## 5. Prompting: facts that shape skill bodies and agent prompts

- **Be clear and direct; explain why.** Give the motivation behind an instruction; Claude generalizes from the reason. "Tell Claude what to do instead of what not to do."
- **Dial back emphasis.** Current models over-trigger on "CRITICAL: You MUST"; use "Use this tool when...". This matches our ceremony rule and gives it a second justification: it is not only ignored, it over-triggers.
- **Examples** (3 to 5) in `<example>` tags, relevant and diverse, are the most reliable format steer. **XML tags** separate instructions, context, input. **A role sentence** in the system prompt focuses behavior. **Long inputs at the top**, query at the end.
- **Opus 5** (our opus agents): verifies its own work unprompted; **remove explicit verification and double-check instructions** or it over-verifies. It expands scope; constrain with "deliver what was asked, at the scope intended". It delegates readily; cap subagent use. Written files run long; add "match the length to what the task needs". For code review: "report every issue, including uncertain and low-severity; a separate step filters" beats "only report high-severity".
- **Sonnet 5** (our sonnet agents): follows instructions literally, especially at low effort; state scope explicitly ("apply to every section, not only the first"). Under-thinks on hard problems at low effort: raise effort rather than prompt around it. Same review-harness guidance: coverage first, filter later.
- **Fable 5.1** (orchestrator sessions): finish-the-whole-task and delivering-work blocks; "you are operating autonomously... asking 'Shall I' blocks the work"; keep changes and tests to what the task asks; prefer targeted edits over whole-file rewrites; give progress updates explicitly.
- **Agentic state**: JSON for structured state, free text for progress notes, git for checkpoints; "it is unacceptable to remove or edit tests"; a first-context-window prompt sets up tests and scripts, later windows iterate.
- **Avoid hardcoding to pass tests**: "Implement a solution that works for all valid inputs, not just the test cases... If the task is infeasible or a test is incorrect, inform me rather than working around it."
- **Overengineering**: no extra files, abstractions, defensive code for impossible cases, or docstrings on untouched code.
- **Investigate before answering**: read the file before making claims about it.

## 6. Claude Code best practices: facts that shape the framework's shape

- Give Claude a check it can run: tests, a build exit code, a diff against a fixture. A Stop hook can gate the turn on it; a verification subagent in fresh context can refute it; `/goal` re-checks after every turn.
- Explore, plan, implement, commit. Plan mode for multi-file or unfamiliar changes; skip it when the diff fits one sentence.
- **CLAUDE.md**: under 200 lines, only what Claude cannot infer from code; bash commands, style deltas, test runner, repo etiquette, gotchas. Reference skills for on-demand material. `/init` generates a starter; `/doctor` proposes cuts.
- **Feature choice**: CLAUDE.md for always-on rules; skill for on-demand knowledge or a `/workflow`; hook for what must happen every time; subagent for isolation; MCP for external services; plugin to package. "Put guardrails in hooks."
- Adversarial review: a reviewer in a fresh subagent sees only the diff and criteria; tell it to flag only gaps that affect correctness or stated requirements, else it over-reports.
- Fan-out with `claude -p` in a loop, `--allowedTools` scoped; `/batch` for 5 to 30 subagents in worktrees.
- Failure patterns: kitchen-sink sessions, correcting more than twice, over-long CLAUDE.md, trust-then-verify gap, infinite exploration.

## 7. Implications the auditors test against the framework

1. Any agent file listing `AskUserQuestion` in `tools` is broken by construction; the question moves to the skill.
2. Any hook that relies on exit 2 on PostToolUse, SessionStart, or SubagentStart is a no-op; the design must use JSON output or move the check to PreToolUse or Stop.
3. Any hook that exits 1 to block is a silent pass.
4. The Stop hook must honor `stop_hook_active` and the 8-block cap; `[stop_hook].block_count` in state.toml is our own bookkeeping, not the harness's.
5. `SubagentStop` ingest reads `last_assistant_message` and `agent_type`; the CLI spec's Decision 47 fallback chain is obsolete.
6. Shell-command matchers are `Bash|PowerShell`.
7. `hooks/settings.hooks.json` should use exec form (`command` + `args`) with the pinned binary path, since the binary is a real `.exe`.
8. Command files are legacy; the skill directory name already makes `/explore`. The thin command's preamble (`!`devforgeai gate require ...``) works identically inside SKILL.md, and a failing preamble aborts the invocation, which is the gate behavior we want. Phase skills that only the user should start get `disable-model-invocation: true`; skills another skill invokes (designing-interfaces from exploring-ideas) stay model-invocable.
9. Skill-frontmatter `hooks` can register phase-scoped hooks (for example Build's file-set guard) when `/build` is invoked, instead of every hook being global.
10. Agent descriptions must be short (delegation trigger only); the 46-agent total must stay well under 15,000 tokens.
11. Verifier agents on opus must not carry "verify your work" or "double-check" instructions; review agents must be told to report everything and let the CLI or a later step filter.
12. The eval runner pre-seeds AskUserQuestion answers through a `PreToolUse` hook returning `allow` + `updatedInput.answers`, and supplies a stdio permission host with `--permission-prompt-tool` for any such case, because without a host the `-p` tool set holds no `AskUserQuestion` for the hook to intercept. A case that should never reach a question runs with `--permission-prompts none` instead.
13. `devforgeai init` should write a CLAUDE.md section for the target project under 40 lines: the nine commands, where the handoff comes from, and that gates are hooks.
14. Skills must say when to read each supporting file; SKILL.md stays under 500 lines and is the standing instruction for the whole phase, since it is not re-read.
