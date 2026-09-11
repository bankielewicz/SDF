---
schema: devforgeai-audit/1
area: hooks
produced_by: audit-hooks
---

# Hook layer audit against the Anthropic hooks reference

Sources: `specs/ANTHROPIC-GUIDANCE.md` §1-§4, §7; `specs/00-conventions.md` §7, §8; `hooks/settings.hooks.json` (byte-identical to `cli/templates/settings.hooks.json`, compiled in at `cli/src/hooks/settings.rs:9`); `specs/01-cli.md` §`hook install` (1463-1480), §`hook run <event>` (1481-1506), §Templates (2271-2401); `cli/src/hooks/run.rs`, `cli/src/hooks/githooks.rs`, `cli/src/hooks/mod.rs`, `cli/tests/hook_run.rs`, `cli/tests/hook_install.rs`. Reference line numbers below are into `hooks-raw.md`.

## 1. Event and matcher correctness

| ID | Severity | Location | Reference says | What exists | Failure scenario |
|---|---|---|---|---|---|
| HOOK-001 | high | `hooks/settings.hooks.json:27`; `cli/src/hooks/run.rs:293` | "On Windows, wherever the PowerShell tool is enabled, Claude treats PowerShell as the primary shell... A hook that matches only `Bash` never fires there." PowerShell `tool_input.command` has the same shape as Bash (raw 1629-1648). Guidance §1 "match `Bash|PowerShell`" | Matcher is `"Bash"`; the dispatcher branches on `tool == "Bash"` and reads `tool_input.command` only in that arm | On Windows without Git Bash the Bash tool is not registered. The developer runs the test suite through PowerShell; the `PostToolUse` handler never fires, `gate check --phase build --partial` never runs, and the build report carries no partial result for the whole phase. Even if the matcher were fixed, the `tool == "Bash"` branch drops the PowerShell payload |
| HOOK-002 | medium | `hooks/settings.hooks.json:26-31`, `:19-25` | `if` filters by permission-rule syntax before the process spawns, on tool events only; one rule per handler (raw 429, 447) | No `if` on any handler | Every Bash call in the session spawns `devforgeai hook run post-tool-use`, which hashes the binary for `trust verify`, loads `config.toml`, compares the command to `test_command`, and exits 0. `ls` pays the full startup cost. The same applies to the two `Write|Edit` handlers, which spawn on every edit anywhere in the tree |
| HOOK-003 | medium | `hooks/settings.hooks.json:43` | `SubagentStop` matcher filters on agent type; exact-string or `\|`-list (raw 288-291, 2360) | `"*"` | The framework ships 46 agents plus the built-ins. Every `Explore`, `Plan`, and `general-purpose` return spawns the dispatcher, which calls `report ingest` and gets `DFA-W411 unregistered` (`cli/src/cmd/report.rs:105-113`). The warning stream fills with noise that hides a real ingest failure, and every subagent return pays a trust-verify hash |
| HOOK-004 | low | `hooks/settings.hooks.json:34` | `Stop` has no matcher support; "If you add a `matcher` field to an event without matcher support, it is silently ignored" (raw 314, 320) | `"matcher": "*"` on `Stop` | No behavioural defect. The field reads as intentional filtering to anyone editing the file and invites a future edit that expects a matcher to work |
| HOOK-005 | low | `hooks/settings.hooks.json:5` | `SessionStart` matchers are `startup \| resume \| clear \| compact \| fork`; `"*"` matches all (raw 286, 1128-1135) | `"*"` | `stack detect` plus `handoff` re-run on every auto-compaction inside a long Build turn. `stack detect` rewrites `config.toml`, so a compaction mid-turn can change detected commands under a gate that is already evaluating |
| HOOK-006 | high | `hooks/settings.hooks.json:12-18`; `cli/src/hooks/run.rs:152-245` | "Claude Code doesn't run a `PostToolUse` hook matching `Edit\|Write` when a `Bash` command or a process outside Claude Code rewrites the same file" (raw 1962). `FileChanged` fires whatever wrote the file (raw 2828) | The producer check, the declared-set check, and the token check all hang off `PreToolUse` matching `Write\|Edit` | `cat > .devforgeai/stories/STORY-007.md <<'EOF' ... EOF` through Bash or PowerShell writes a story with `produced_by: implementing-stories`. No `PreToolUse` hook fires (the matcher is Write/Edit), no `PostToolUse` hook fires (reference, above), so `doc validate --producer-check` never runs. The producer gate, which conventions §7 lists as the blocking rule for `PreToolUse`, is bypassed by one heredoc |
| HOOK-007 | low | `hooks/settings.hooks.json:13`, `:21` | Matcher set `[A-Za-z0-9_ ,\|-]` is exact-list; anything else is an unanchored regex (raw 288-291) | `"Write\|Edit"` is on the exact-match path, correctly | No mismatch. Recorded because the audit asks. Note that `NotebookEdit` is a separate tool name and is not covered by either handler, so a notebook write under `.devforgeai/` skips the producer check |

## 2. Exit-code semantics per event

Dispatcher behaviour read from `cli/src/hooks/run.rs`; process mapping from `cli/src/main.rs:34-35` (`Outcome.exit` becomes the process exit code verbatim).

| Event | Our intent | Our exit | What the harness does (raw 876-917) |
|---|---|---|---|
| SessionStart | never blocks, prints handoff | 0 | stdout added to Claude's context (raw 812); the intent holds for context but see HOOK-021 for the handoff's audience |
| SessionStart, trust failure | fail closed (conventions §8) | 4 | `Can block? No` — "Shows stderr to user only"; exit 4 with plain/empty stdout is a non-blocking error notice. The session proceeds untrusted |
| PreToolUse | block producer mismatch / token violation / undeclared file | 2 | Blocks the tool call, stderr is the denial reason. Holds |
| PreToolUse, input or state error | (unstated) | 3 | Non-blocking error; the write proceeds |
| PreToolUse, trust failure | block | 2 | Blocks. Holds |
| PreToolUse, timeout at 30 s | block | cancelled | "A timed-out `command`... hook doesn't block the tool call... don't count on a stalled hook to act as a gate" (raw 871) |
| PostToolUse | annotate | 0 | stdout to the debug log, stderr to the debug log. Claude sees nothing |
| PostToolUse, trust failure | fail closed | 4 | `Can block? No`. Non-blocking error notice; Claude sees the notice text, not our diagnostic |
| Stop, gate FAIL, first time | block once with the reason | 2 | Blocks. The reason is "the reason from your JSON's blocking decision when it makes one, and your stderr text otherwise" (raw 826). Our reason is on stdout |
| Stop, gate FAIL, second time | print FAIL handoff | 0 | stdout to the debug log. The handoff is not shown |
| Stop, internal error | (unstated) | 5 | Non-blocking error; the turn ends with no gate evaluated |
| SubagentStop | never blocks | 0 | Correct as an intent, but SubagentStop `Can block? Yes`, so a malformed verifier envelope cannot be pushed back to the subagent |
| SubagentStop, trust failure | fail closed | 4 | Non-blocking error; the subagent stops normally |

| ID | Severity | Location | Reference says | What exists | Failure scenario |
|---|---|---|---|---|---|
| HOOK-010 | blocker | `cli/src/hooks/run.rs:263-329`; conventions §7 row 3 and row 4 | `PostToolUse` `Can block? No` on exit 2; to reach Claude, return `hookSpecificOutput.additionalContext`, top-level `decision`/`reason`, or `systemMessage` (raw 900, 1994-2010). Exit 0 stdout and stderr both go to the debug log only (raw 812, 822) | Exit 0, `doc validate` diagnostics into `Outcome.warnings` (stderr), `gate check --partial` lines into `Outcome.human` (stdout) | A story document written with a broken `consumes:` reference gets `DFA-Exxx` from `doc validate`. It is printed to stderr, exit 0, and lands in the debug log. Claude never learns the document is invalid and proceeds to the next phase. Conventions §7 calls this row "never blocks (annotates)"; there is no annotation |
| HOOK-011 | blocker | `cli/src/hooks/run.rs:353-365, 400-412`; `cli/src/cmd/gate.rs:99-148` | On Stop, exit 2 "Prevents Claude from stopping"; the message Claude receives is the JSON `reason` or "your stderr text otherwise" (raw 826, 2597) | `gate::check` returns its `Gate`/`Result`/`Checks` lines in `human` (stdout) and no `warnings`. A blocked Stop returns exit 2 with `human` populated and `warnings` empty, so stderr is empty | The build gate fails on coverage. The Stop hook exits 2. Claude is told to continue with an empty reason, so it has no information about which check failed or which story it belongs to, and guesses. The `Verified`/`Gate` numbers that conventions §6 puts in front of the model exist only in the debug log |
| HOOK-012 | high | `cli/src/hooks/run.rs:102` (`Payload::parse`), `:153-159`, `:195` (`ctx.state()?`), `:222` (`ctx.config()?` via `matches_frontend`) | "Without valid JSON on stdout, Claude Code treats exit code 1 as a non-blocking error and proceeds with the action" — the same holds for 3 and 5; only exit 2 or a valid decision object enforces (raw 796, 848-866). Guidance §7.3 | Every error path out of `pre_tool_use` is a `CliError`: `DFA-E021` (stdin not JSON) exit 3, `DFA-E020` (no `tool_name` / no `tool_input.file_path`) exit 3, `DFA-E030` (no `.devforgeai/` above cwd) exit 3, a `config.toml` or `state.toml` parse failure exit 1 or 5. Tests `hook_stdin_not_json_gives_e021` and `hook_missing_key_gives_e020` (`cli/tests/hook_run.rs:747-765`) assert exit 3 | An agent edits `.devforgeai/config.toml` and leaves it unparsable. Every subsequent `PreToolUse` run exits 1 from `ctx.config()?` inside `matches_frontend`. The harness reports a non-blocking hook error and lets every write through, including writes to `.devforgeai/`. Corrupting one config file disables the whole PreToolUse gate |
| HOOK-013 | medium | `cli/src/hooks/run.rs:77-99`; conventions §8 | `SessionStart` exit 2 (and by extension any non-zero): "Shows stderr to user only"; Claude does not see it and the session proceeds (raw 915) | Trust failure on `session-start` returns exit 4 with the diag on stderr | A tampered binary is detected at session start. The user sees a hook-error notice, the session starts anyway, and every later hook in that session runs the same tampered binary. Conventions §8's "Mismatch: exit 4, the hook blocks" is not achievable on this event by any exit code |
| HOOK-014 | medium | `cli/src/hooks/run.rs:415-462` | `SubagentStop` `Can block? Yes`; `decision: "block"` with `reason` "keeps the subagent running and delivers `reason` to the subagent as its next instruction" (raw 2387) | Always exit 0, even when `report ingest` fails to parse the verifier envelope (`DFA-W411` or the ingest error becomes a warning) | A verifier agent returns prose instead of the `devforgeai/verifier/1` envelope. Ingest fails, exit 0, the subagent stops, and the phase report carries no verifier block. The gate then evaluates `verifier_pass` against an absent block. The one event that could ask the agent to re-emit the envelope is used as a logging sink |
| HOOK-015 | low | `cli/src/hooks/run.rs:331-413`; `cli/src/lib.rs` error arm | Exit codes other than 2 are non-blocking on Stop (raw 848) | A `DFA-E9xx` from `ctx.store_state()?` at `run.rs:392` propagates as exit 5 before the handoff runs | A read-only `state.toml` turns every Stop into a silent non-blocking error. The gate result is computed and thrown away |

## 3. JSON output

| ID | Severity | Location | Reference says | What exists | Failure scenario |
|---|---|---|---|---|---|
| HOOK-020 | blocker | `cli/src/hooks/run.rs` (whole file); `cli/src/lib.rs:165-200` | Decision and context travel in `hookSpecificOutput` with a `hookEventName` matching the event: `PreToolUse` `permissionDecision`/`permissionDecisionReason`, `PostToolUse` `additionalContext`/`decision`+`reason`, `Stop` `decision`+`reason` or `additionalContext`, `SessionStart` `additionalContext` (raw 1037-1060, 1771-1790, 1994-2010, 2571-2590, 1177-1200) | The string `hookSpecificOutput` does not appear anywhere in `cli/`. No event emits a decision object. Every event communicates through exit code, plain stdout, and stderr only | Only the two exit-2 paths (PreToolUse, first Stop) carry any signal at all, and each carries at most a stderr string. Everything the framework wants to say on the other events, and every reason a `PreToolUse` deny could carry as structured text, is dropped. Guidance §7.2 names this as the design's failure mode |
| HOOK-021 | blocker | `cli/src/hooks/run.rs:394-398`; conventions §6 "The Stop hook prints this block"; §1 rule 3 | "For most events, Claude Code writes stdout to the debug log and doesn't show it in the transcript. The exceptions are `UserPromptSubmit`, `UserPromptExpansion`, `SessionStart`, and `PostModelSwitch`" (raw 812). `Stop` is not an exception | The non-blocking Stop path calls `cmd::handoff::run` and pushes its lines into `Outcome.human`, which `lib.rs` writes to stdout | The user finishes a Build turn. The handoff block with `Next /verify STORY-003` is written to stdout, the harness files it in the debug log, and neither the user nor Claude ever sees it. The framework's single hand-off mechanism, which conventions §1 rule 3 makes the end of every phase, produces no visible output. Every skill's documented "the Stop hook prints this block" is inert |
| HOOK-022 | medium | `cli/src/lib.rs:167-178`; `specs/01-cli.md:1503` | "Starts with `{` and ends with `}`: Claude Code parses it as JSON"; an object that fails schema validation is a non-blocking error, and on the context events "Claude Code doesn't add the text" (raw 814-820) | `hook run --json` prints the DevForgeAI envelope, an object starting with `{`. The spec documents a `--json` `data` shape for `hook run` | Anyone who adds `--json` to the settings command string (the spec invites it by documenting the shape) turns every hook into a schema-validation failure: `PreToolUse` blocks stop working, `SessionStart` context stops being added, and the transcript shows a hook-error notice instead of our message |
| HOOK-023 | low | `cli/src/hooks/run.rs:124-150` | Stderr from a hook that exits 0 "goes to the debug log only, never the transcript, and Claude never sees it" (raw 822) | `session_start` collects `stack detect` and `handoff` failures into `warnings` (stderr) and still exits 0 | A project with no `.devforgeai/` or a broken `config.toml` starts a session where detection silently failed. The context Claude gets is the handoff's fallback text with no indication that detection did not run |
| HOOK-024 | low | `cli/src/hooks/run.rs:145-149` | Output strings are capped at 10,000 characters; longer output is written to a file and replaced with a preview (raw 939) | `session-start` concatenates `stack detect` plus the full handoff with no length budget | A brownfield project with many detected stacks pushes the SessionStart context over the cap and Claude receives a file path and a preview instead of the handoff |

## 4. Stop loop protection

| ID | Severity | Location | Reference says | What exists | Failure scenario |
|---|---|---|---|---|---|
| HOOK-030 | blocker | `cli/src/hooks/run.rs:332` | "`stop_hook_active` is `true` when Claude Code is already continuing as a result of a stop hook. Check this value or process the transcript to avoid blocking on a condition that will never resolve. Claude Code overrides the hook and ends the turn after 8 consecutive blocks" (raw 2519). Guidance §1, §7.4 | `let _ = p.bool_key("stop_hook_active");` — the value is read and discarded. Nothing else in `run.rs` references it | The value is parsed only so the field exists in the payload; no code path changes on it. Test `stop_different_id_resets_counter` (`cli/tests/hook_run.rs:665-687`) passes `stop_hook_active: true` and asserts exit 2, so the loop behaviour is encoded in the suite. Concretely: Build is active on STORY-003 and the gate fails. Stop 1 blocks. Claude continues, moves to STORY-004 (`phase set` runs from the `/build` preamble), stops. The pair changed, so `blocked_phase`/`blocked_id` reset and Stop 2 blocks. Alternating or advancing IDs produce an unbounded chain of blocks, each of which re-runs the full gate under the 900 s timeout, until the harness's 8-block cap ends the turn with no handoff shown |
| HOOK-031 | high | `cli/src/hooks/run.rs:367-392`; `specs/01-cli.md:1500` ("Stop counter"); `state.toml` `[stop_hook]` | `stop_hook_active` is per-continuation state owned by the harness. The 8-block cap is per-turn | `block_count`, `blocked_phase`, `blocked_id` are persisted to `state.toml` and survive the session | The developer blocks once on a failing explore gate, quits, and starts a new session the next morning. The first Stop of that session finds the same pair with `block_count >= 1` and exits 0. The gate never blocks again for that ID in any future session, so the FAIL is announced once and then permanently tolerated. This is the inverse of HOOK-030 with the same root cause: our counter is not the harness's continuation state |
| HOOK-032 | medium | `specs/00-conventions.md` §7 Stop row; `specs/01-cli.md:1500` | The harness's loop protection is `stop_hook_active` plus the 8-block cap | Both specs express "second Stop passes" against `[stop_hook].block_count` | The spec's rule is stated in terms of a counter the harness does not read, so an implementation that satisfies the spec exactly (which this one does) still loops. The rule should read: exit 0 when `stop_hook_active` is true; block at most once per turn otherwise |
| HOOK-033 | low | `cli/src/hooks/run.rs:353-365` | Stop hooks run on every turn end | A blocked Stop re-runs `gate check` with no `--no-run`, so the project's test suite runs again on the continuation turn | Two full test-suite runs per blocked turn, under a 900 s timeout each. On a slow suite the turn stalls for the user with no spinner text (`statusMessage` is not set) |

## 5. SubagentStop ingest

| ID | Severity | Location | Reference says | What exists | Failure scenario |
|---|---|---|---|---|---|
| HOOK-040 | blocker | `cli/src/hooks/run.rs:415-424` | SubagentStop input carries `agent_id`, `agent_type`, `agent_transcript_path`, `last_assistant_message`; "The `last_assistant_message` field contains the text content of the subagent's final response, so hooks can access it without parsing the transcript file" (raw 2367-2370). `transcript_path` "is the main session's transcript" | `p.opt_str(&["tool_response", "content"]).or_else(|| p.opt_str(&["transcript_path"]).and_then(last_assistant))`. `last_assistant_message` is never read; `agent_transcript_path` is never read | `tool_response` is a `PostToolUse` field and is absent from every SubagentStop payload, so the first arm always misses. The second arm opens the **parent session** transcript and returns its last assistant message, which is the orchestrator's text, not the verifier's. `report ingest` then receives the wrong document, fails to parse it as `devforgeai/verifier/1`, and the phase report gets no verifier block. Every `verifier_pass` gate check evaluates against a missing block. Test `subagent_stop_registered_ingests` (`cli/tests/hook_run.rs:694-724`) supplies `tool_response.content` by hand, so the suite never exercises a real payload |
| HOOK-041 | high | `cli/src/hooks/run.rs:421-424`, `:465-488` | `agent_transcript_path` "is the subagent's own transcript stored in a nested `subagents/` folder" (raw 2369) | The fallback reads `transcript_path` | Even after `last_assistant_message` is read first, the fallback still points at the wrong file. A verifier whose final message was truncated has no correct recovery path |
| HOOK-042 | medium | `specs/01-cli.md:52`, `:1497`, `:2764` | Guidance §1 (line 24): "This resolves the CLI spec's Decision 47 blocker: read `last_assistant_message` for the verifier envelope; fall back to `agent_transcript_path`." Guidance §7.5 (line 105) marks the fallback chain obsolete | Line 52 lists the stdin keys as `tool_response` (object, PostToolUse **and SubagentStop**) and `transcript_path` (string, **SubagentStop fallback**). Line 1497 is the `hook run` table row: "`agent_type`, `tool_response.content`". Line 2764 is the `### Blockers` item 80 that fixes the fallback chain: "Where it carries only `transcript_path`, the dispatcher reads that JSONL file and takes the content of the last assistant message" | The implementation matches the spec exactly; the spec is what is wrong. All three lines have to change together, and `last_assistant_message` and `agent_transcript_path` appear in none of them. Fixing only the code leaves the next implementer reading three obsolete statements |
| HOOK-043 | low | `cli/src/hooks/run.rs:465-488` | Transcript entries are JSONL message records | `last_assistant` accepts `content` only when it is a string, at `v.content` or `v.message.content` | Assistant messages in a Claude Code transcript carry `content` as an array of typed blocks. The function returns `None` for every such line, so the fallback yields nothing even when pointed at the right file |

## 6. Trust gating

| ID | Severity | Location | Reference says | What exists | Failure scenario |
|---|---|---|---|---|---|
| HOOK-050 | high | `cli/src/hooks/run.rs:21-23, 77-99`; conventions §8; `specs/01-cli.md:1489` | Exit 2 blocks on `PreToolUse`, `Stop`, `SubagentStop` (and `UserPromptSubmit`, `UserPromptExpansion`, `PreCompact`, `TaskCreated`, `ConfigChange`). It does not block on `PostToolUse`, `SessionStart`, `SubagentStart`, `SessionEnd`, `Notification` (raw 880-914) | `blocks_on_trust_failure` returns true for `pre-tool-use` and `stop` only; the other three exit 4. Test `hook_trust_failure_exits_four_on_post_tool_use` (`cli/tests/hook_run.rs:790-812`) asserts this | Three of the five events fail open on a tampered binary: `session-start` (HOOK-013), `post-tool-use`, and `subagent-stop`. `subagent-stop` is the avoidable one — the event can block on exit 2, so a trust failure there could and should stop the subagent. As written, a tampered binary is reported once as a hook-error notice at session start and then silently used for the rest of the session |
| HOOK-051 | medium | `specs/00-conventions.md` §8 bullet 2 | The per-event table (raw 876-917) | "Every hook runs `devforgeai trust verify` first. Mismatch: exit 4, the hook blocks, the handoff shows `Gate TRUST FAIL`." | Exit 4 blocks on no event. The sentence is false for all five events the framework registers: on `pre-tool-use` and `stop` the dispatcher has to translate to 2 (and does), and on the other three nothing blocks at any exit code. The conventions text should name the two blocking events and state that the other three degrade to a user-visible notice |
| HOOK-052 | low | `cli/src/hooks/githooks.rs:22, 47, 62` | Git treats any non-zero hook exit as a failure | `"$DFA" trust verify || exit 4` in all three scripts; a missing binary gives 127, which also takes the `||` branch | No mismatch. Recorded because the audit asks: the git hooks do fail closed on a missing or untrusted binary |
| HOOK-053 | low | `cli/src/hooks/run.rs:77` | `PreToolUse` command hooks share the write path's latency budget | `trust verify` hashes the whole binary on every hook invocation, including every `Write` and `Edit` | On a large release binary over a network drive, the SHA-256 read is charged to every keystroke-latency write under a 30 s timeout, and a slow read pushes toward the cancellation that HOOK-060 describes |

## 7. Exec form versus shell form, and timeouts

| ID | Severity | Location | Reference says | What exists | Failure scenario |
|---|---|---|---|---|---|
| HOOK-060 | high | `hooks/settings.hooks.json:15` | "A timed-out `command`, `http`, or `mcp_tool` hook doesn't block the tool call. The call continues through the normal permission flow, so don't count on a stalled hook to act as a gate" (raw 871). Command default is 600 s (raw 458) | `"timeout": 30` on the `PreToolUse` handler, justified in `specs/01-cli.md:2333` as "30 for the one that blocks a keystroke-latency write" | `design lint` on a frontend file with a large `brand/tokens.json`, on a cold binary, on a project whose `.devforgeai/` sits on a network share, plus the trust hash of HOOK-053, exceeds 30 s. The hook is cancelled, its output discarded, and the write proceeds. The token gate is disabled by being slow rather than by failing — and it fails open exactly under the load where a violation is most likely |
| HOOK-061 | medium | `hooks/settings.hooks.json` (all five handlers); `cli/src/hooks/githooks.rs:101-117` | "Set `args` whenever the hook references a path placeholder, since each element is passed as one argument with no quoting"; on Windows exec form needs a real executable such as a `.exe` (raw 470-495). Guidance §1, §7.7 | Shell form everywhere: `"command": "@@DEVFORGEAI@@ hook run stop"`. `token_value()` substitutes an absolute forward-slash path when `devforgeai` is not on `PATH`, and the path is interpolated into the command string unquoted | A binary installed at `C:/Program Files/devforgeai/devforgeai.exe` produces `C:/Program Files/devforgeai/devforgeai.exe hook run stop`. Git Bash splits at the space and runs `C:/Program`, which does not exist. Exit 127, non-blocking notice, every hook silently disabled for that install. Our binary is a real `.exe`, so exec form is available and removes the class |
| HOOK-062 | low | `hooks/settings.hooks.json:7, 23, 29, 37, 45` | Defaults: 600 s for command hooks, lowered to 30 on `UserPromptSubmit`, `PreModelSwitch`, `PostModelSwitch`, 10 on `MessageDisplay` (raw 458) | 120 / 60 / 900 / 900 / 60 | No mismatch on the raised values: 900 on Stop and on the Bash `PostToolUse` handler is under nothing, since neither event lowers the default and there is no ceiling. Only the 30 in HOOK-060 is a defect |
| HOOK-063 | low | `hooks/settings.hooks.json` | `statusMessage` sets the spinner text while a hook runs (raw 461) | Not set on any handler | A 900 s Stop hook running the test suite shows the generic spinner. The user cannot tell the session from a hang |

## 8. Path normalization

| ID | Severity | Location | Reference says | What exists | Failure scenario |
|---|---|---|---|---|---|
| HOOK-070 | high | `cli/src/project.rs:rel_display`; `cli/src/hooks/run.rs:162, 222, 249-261` | "On Windows, the path arrives with backslash separators... A comparison written with forward slashes, such as a `/src/` check, never matches a backslash path, and the tool call proceeds as if the hook had nothing to block. Normalize separators before comparing... then match a path segment such as `/src/` rather than anchoring with `^`" (raw 1594-1600) | `rel_display` tries `path.strip_prefix(root)` and, on failure, silently returns the whole absolute path with slashes flipped. `matches_frontend` then runs project-relative globs (`src/**/*.tsx`) against that absolute string, and `story files --check` receives it too | `Path::strip_prefix` is a case-sensitive component comparison, while the root is `fs::canonicalize`d to the on-disk case. Claude writes to `c:\Projects\DevForgeAI\src\Button.tsx` (lower-case drive letter, which Windows accepts and Claude emits freely). `strip_prefix` fails, `rel` becomes `c:/Projects/DevForgeAI/src/Button.tsx`, the glob `src/**/*.tsx` does not match, `design lint` never runs, and a hard-coded `#ff0000` lands in a component. The same input skips the declared-set check. Short names (`C:/PROJEC~1/...`), a junction or `subst` drive, and a UNC spelling of the same path each reproduce it |
| HOOK-071 | high | `cli/src/hooks/run.rs:167, 194, 273` | Match a path segment, normalized, rather than trusting an exact spelling (raw 1600) | `rel.starts_with(".devforgeai/") \|\| rel.contains("/.devforgeai/")` — case-sensitive on a case-insensitive filesystem | `C:\Projects\DevForgeAI\.DevForgeAI\stories\STORY-003.md` names the same file on Windows. Neither test matches, so the path is classified as "outside `.devforgeai/`": the producer check is skipped entirely, and instead the declared-set check runs and rejects it (or passes it in a non-build phase). A story can be written with any `produced_by` value by capitalizing one directory segment. Note the `contains` arm does rescue the HOOK-070 absolute-path fallback for this one check, which is why the bypass needs the case trick |
| HOOK-072 | medium | `cli/src/hooks/run.rs:154-159` | `tool_input.file_path` is present for `Write`, `Edit`, `Read` (raw 1590, 1648-1667) | A missing `file_path` is `DFA-E020`, exit 3 | Any future addition to the `PreToolUse` matcher whose input shape differs (`NotebookEdit` uses `notebook_path`) turns every such call into a non-blocking hook error rather than a pass, which is HOOK-012's class |
| HOOK-073 | medium | `cli/src/hooks/run.rs:152-245`; cross-reference HOOK-006 | `FileChanged` "runs the hook no matter what changed the file" and is the recommended answer for Bash writes (raw 1962, 2828) | No `FileChanged` registration anywhere | The path checks of this section are only reachable through the `Write`/`Edit` tools. Normalizing them perfectly still leaves the Bash write path with no check at all |

## 9. Git hooks

| ID | Severity | Location | Reference says | What exists | Failure scenario |
|---|---|---|---|---|---|
| HOOK-080 | medium | `cli/src/hooks/githooks.rs:19-40` (`PRE_COMMIT`) | A pre-commit hook enforces by exiting non-zero; conventions §7 pre-commit row blocks on "any failure" | No `set -e`. `git_dir=$(git rev-parse --git-dir)` is unchecked, and the redirect `> "$list"` is unchecked. `status` is initialized to 0 | In a worktree, `git rev-parse --git-dir` prints the per-worktree git dir, which exists, so the common case works. But if the command fails or the directory is not writable, `> "$list"` fails, the `while` loop reads an absent or empty file, the `context audit` arm is skipped when `.devforgeai/context` is relative to a cwd that is not the repo top-level, and `exit 0` lets a commit through with zero documents validated. Failure is indistinguishable from "nothing staged" |
| HOOK-081 | medium | `cli/src/hooks/githooks.rs:44-55` (`COMMIT_MSG`); conventions §7 commit-msg row | The hook rejects a message with no STORY or ADR id | `grep -Eq '(STORY\|ADR)-[0-9][0-9][0-9]' "$1"` over the raw message file, unanchored, with no comment stripping | Answering the audit question directly: yes, a message with no id is rejected (exit 1 aborts the commit). But the check is satisfiable without a real reference: `git commit -v` writes the full diff into the message file, so any diff hunk containing `STORY-001` passes; a `#`-commented line such as the `# On branch story/STORY-004` that git itself may insert passes; and `ADR-000` passes. The id is never checked against `.devforgeai/stories/` or `.devforgeai/adr/` |
| HOOK-082 | medium | `cli/src/hooks/mod.rs:120-161, 185-214` | Git runs hooks from `core.hooksPath` when it is set, not from `<git-common-dir>/hooks` | `git_common_dir` asks only for `--git-common-dir` and joins `hooks` | A repository that sets `core.hooksPath` (husky, lefthook, pre-commit, or a corporate template) gets three scripts written to a directory git never consults. `hook install` reports success, the `--json` `git_hooks` array lists all three, and none of them ever runs. The commit gate is silently absent |
| HOOK-083 | low | `cli/src/hooks/githooks.rs:59-78` (`PRE_PUSH`) | Conventions §7 pre-push row: gate check for every story in `sprint.yaml` with status `building` | An awk program taking `$3` of an `- id:` line and requiring `status:` to follow `id:` within the item | `- id: "STORY-004"` yields `"STORY-004"` with quotes; `- {id: STORY-004, status: building}` yields nothing; a sprint file where `status` precedes `id` attributes the status to the previous story. The spec acknowledges the coupling to `templates/sprint.min.yaml`, so any hand edit of the sprint file can silently empty the push gate (`ids` empty, `status` 0, push proceeds) |
| HOOK-084 | low | `cli/src/hooks/githooks.rs:19, 44, 59`; `cli/src/hooks/mod.rs:165-176` | Git for Windows executes hooks through its bundled `sh` | `#!/bin/sh`, no arrays, no `[[`, no bashisms; `make_executable` is a no-op off Unix | No mismatch. The three scripts are POSIX sh and Git Bash compatible, and each calls `trust verify` on its first executable line |
| HOOK-085 | low | `cli/src/hooks/githooks.rs:35` | — | `[ -d .devforgeai/context ]` and `-- .devforgeai` are relative to the hook's cwd, which git sets to the repository top-level | A project installed into a subdirectory of a larger repository (`monorepo/app/.devforgeai/`) stages documents whose paths git reports relative to the repo root; `-- .devforgeai` matches nothing and `context audit` never runs. The pre-commit gate is inert for every non-root install |

## 10. Missed opportunities from the reference

| ID | Severity | Location | Reference says | What exists | Failure scenario |
|---|---|---|---|---|---|
| HOOK-090 | low | `hooks/settings.hooks.json` | `UserPromptExpansion` fires when a user types `/name`, matches on `command_name`, and `decision: "block"` prevents the expansion. "This event covers the path `PreToolUse` doesn't: a `PreToolUse` hook matching the `Skill` tool fires only when Claude calls the tool, but typing `/skillname` directly bypasses `PreToolUse`" (raw 1391-1397) | Gate enforcement for `/build` lives in the command's `!`-preamble (`devforgeai gate require build $1`), which runs after expansion and inside the model's turn | A `UserPromptExpansion` handler matching `build\|verify\|release\|plan` could run `gate require` and block the expansion before the skill body enters context, saving the whole SKILL.md token load on a gate that will fail anyway, and putting the refusal in front of the user rather than inside the model's turn |
| HOOK-091 | low | `hooks/settings.hooks.json` | `SubagentStart` injects `hookSpecificOutput.additionalContext` into the subagent before its first prompt; it cannot block (raw 2322-2356). Guidance §1 | Not registered | The story's declared `## Files` set, the six context file paths, and the active gate thresholds could be injected into every implementer agent at spawn, instead of relying on the delegation message to carry them. The re-injection rule (only when the copy is absent) keeps the subagent prompt cache intact |
| HOOK-092 | low | `skills/*/SKILL.md` frontmatter | "Hooks in skill frontmatter register when the skill is invoked and persist for the session"; `once: true` removes after the first success (raw 461, 688). Guidance §3, §7.9 | Every hook is global in `settings.hooks.json` | The declared-set guard (`story files --check`) is meaningful only during Build, yet it is evaluated on every `Write` in every phase, gated at runtime by `ctx.state()?.current.phase == "build"` (`run.rs:195`). Moving it to `implementing-stories` frontmatter removes both the spawn and the state read from every other phase |
| HOOK-093 | low | `cli/src/hooks/run.rs:124-150` | `SessionStart` accepts `additionalContext`, `sessionTitle`, `watchPaths`, `reloadSkills`; "Since plain stdout already reaches Claude for this event, a hook that only loads context can print to stdout directly... Use the JSON form when you need to combine context with other fields" (raw 1177-1200) | Plain stdout | Plain stdout is correct for context alone and is not a defect. The JSON form becomes worth it when combined with `sessionTitle` (`<phase> · <ID>`, checked against the incoming `session_title` so a user-set name is not overwritten) and with `watchPaths`, which HOOK-095 needs |
| HOOK-094 | low | `hooks/settings.hooks.json:26-31` | `async: true` runs a command hook in the background; its `additionalContext` and `systemMessage` are delivered on the next turn; `timeout` is not enforced once it is running (raw 3619-3660) | The Bash `PostToolUse` handler runs `gate check --phase build --partial` synchronously with a 900 s timeout | The partial gate runs the project's test suite inline, so the agent loop stalls behind it. As `async: true` it never blocks, and its `additionalContext` (once HOOK-010 is fixed) arrives on the next turn, which is when the agent can act on it anyway. Note `classifierContext` is ignored on async hooks |
| HOOK-095 | low | `hooks/settings.hooks.json` | `FileChanged` fires whatever wrote the file. Its matcher "is split on `\|` and each segment is registered as a literal filename in the working directory"; to watch paths you cannot name up front, return `watchPaths` (absolute) from `SessionStart` and give the handling group an **omitted** matcher — `"*"` is registered as a literal file named `*` (raw 2828-2870) | Not registered | A `FileChanged` group watching `.devforgeai/gates.toml`, `config.toml`, and `state.toml` detects an agent lowering a threshold or rewriting the phase, which conventions §8 forbids but nothing currently observes. The matcher cannot express the `.devforgeai/` prefix, so this needs `SessionStart` `watchPaths` with absolute paths plus a matcher-less `FileChanged` group |
| HOOK-096 | low | `hooks/settings.hooks.json` | `ConfigChange` matches on `user_settings`, `project_settings`, `local_settings`, `policy_settings`, `skills`; it blocks on exit 2 or `decision: "block"`, and the change is not applied to the running session (raw 2676-2746) | Not registered | An agent editing `.claude/skills/implementing-stories/SKILL.md` mid-session, or removing our block from `.claude/settings.json`, is currently undetectable. A `ConfigChange` handler on `skills\|project_settings` is the only event that sees it, and it can refuse |
| HOOK-097 | low | `hooks/settings.hooks.json:41-48` | "To inject context into the parent session after a subagent returns, use a `PostToolUse` hook on the `Agent` tool instead" (raw 2387) | Ingest happens on `SubagentStop`, which can only speak back to the subagent | A `PostToolUse` handler with matcher `Agent` is the supported way to put the ingested verifier counts in front of the orchestrator, which is what the `Verified` line of the §6 handoff needs |
| HOOK-098 | low | `hooks/settings.hooks.json` | `PostToolUseFailure` carries the same matchers and shows stderr to Claude on exit 2 (raw 2061-2122) | Not registered | A failed `Write` to a read-only `.devforgeai/` file, or a test command that exits non-zero, produces no framework signal at all. This is the one event where exit 2 does put text in front of Claude on an already-completed action |
| HOOK-099 | low | `hooks/settings.hooks.json`; guidance §4, §7.12 | A `PreToolUse` hook on `AskUserQuestion` returning `permissionDecision: "allow"` with `updatedInput` that echoes `questions` and adds `answers` is "the supported way to pre-seed answers" in `-p` runs (raw 1810-1852) | The eval runner has no such handler; the hooks block registers nothing for `AskUserQuestion` | Every eval case whose skill reaches a question either stalls or is denied. The runner cannot exercise the interactive paths of Explore, Discover, or Plan |
| HOOK-100 | low | `hooks/settings.hooks.json` | Hook types include `prompt` (single-turn model decision returning `{"ok": false, "reason": ...}`) and `agent` (experimental, tool-using). "Prompt/agent hooks exist for judgment calls; command hooks for deterministic rules" (guidance §1; raw 608-616) | Only `command` hooks | Conventions §1 rule 1 puts judgment in skills and enforcement in the CLI, which leaves judgment unenforceable. A `prompt` hook on Stop is the reference's answer for the checks `gates.toml` cannot express as a number |

## Fix specifications

### HOOK-010 — PostToolUse annotates through JSON

**File** `cli/src/hooks/run.rs`, `fn post_tool_use`.

**New stdin fields read**: none beyond the current set; add `tool_name` acceptance for `PowerShell` (see HOOK-001).

**New stdout JSON**, emitted on exit 0 when any diagnostic or gate line was produced, as the only thing on stdout:

```json
{
  "hookSpecificOutput": {
    "hookEventName": "PostToolUse",
    "additionalContext": "devforgeai doc validate .devforgeai/stories/STORY-003.md: DFA-E2xx <message>"
  }
}
```

Build `additionalContext` by joining the rendered diagnostics and the gate's human lines with `\n`, capped at 10,000 characters (truncate with a trailing `… [+N chars]`). Emit nothing when there is nothing to say — an empty object is still a parsed object and costs a notice on no-op calls. Where the intent is a warning rather than a note, use top-level `{"decision":"block","reason":"..."}`, which on `PostToolUse` appends the reason beside the tool result without undoing the write. That option is open to the synchronous `Write`/`Edit`/`NotebookEdit` handlers only: the two shell handlers carry `async: true` in the replacement template, and an async hook's `decision` has no effect because the action it would control has already completed (raw 3621). An async handler's only channels are `additionalContext` and `systemMessage`, and `classifierContext` is dropped on it.

**New exit code**: unchanged, 0. Exit 2 is available on this event only to force stderr in front of Claude; JSON is the better channel and does not produce a hook-error notice.

**Plumbing**: `Outcome` needs a `hook_json: Option<serde_json::Value>` field written to stdout ahead of `human` in `cli/src/lib.rs`, and `human` is suppressed whenever `hook_json` is set, because stdout "must contain only the JSON object".

**Test first** (`cli/tests/hook_run.rs`): `post_tool_use_invalid_document_emits_additional_context` — write an invalid document under `.devforgeai/`, dispatch `post-tool-use` with a `Write` payload, assert the outcome's hook JSON parses, `hookSpecificOutput.hookEventName == "PostToolUse"`, and `additionalContext` contains the `DFA-` code.

### HOOK-011 — a blocked Stop carries its reason on stderr or in `reason`

**File** `cli/src/hooks/run.rs`, `fn stop`.

**Change**: when `blocked` is true, emit the decision object instead of relying on stderr:

```json
{
  "decision": "block",
  "reason": "Gate      build · STORY-003\nResult    FAIL\nChecks    coverage_min 71% < 80%; tests_pass 3 failing\nRun /verify STORY-003 after the failing checks pass."
}
```

Build `reason` from the `gate check` human lines already collected in `human`, plus the failed check IDs from the gate's `data` object. Keep exit 2 (belt and braces: exit 2 blocks whether or not the JSON parses, and the reference routes stderr as the reason when the JSON has none), and additionally copy the same text into `Outcome.stderr` so a schema change upstream degrades to the stderr path rather than to silence.

**New exit code**: unchanged, 2.

**Test first**: `stop_block_reason_names_the_failing_check` — a failing explore gate, dispatch `stop`, assert exit 2 and that the hook JSON's `reason` (or `Outcome.stderr` joined) contains the gate ID and the word `FAIL`.

### HOOK-020 — emit `hookSpecificOutput` on every event that has a decision channel

**Files** `cli/src/lib.rs` (`Outcome`, the stdout arm of `run`), `cli/src/hooks/run.rs` (all five handlers).

**Change**: add `pub hook_json: Option<serde_json::Value>` to `Outcome`. In `lib.rs`, when `args.json` is false and `hook_json` is `Some`, write that object to stdout and write nothing else there; warnings continue to stderr. Per event:

- `pre_tool_use`, on block: `{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"<the diagnostics, joined>"}}`, exit 2 retained. On pass: no JSON.
- `post_tool_use`: HOOK-010.
- `stop`: HOOK-011 on block; on the pass path, `{"hookSpecificOutput":{"hookEventName":"Stop","additionalContext":"<handoff>"}}` only when the framework wants Claude to act on it — otherwise HOOK-021's `systemMessage`.
- `subagent_stop`: on an ingest parse failure, `{"decision":"block","reason":"<the fields the envelope is missing>"}` with exit 2 (HOOK-014).
- `session_start`: HOOK-021.

**Test first**: one test per event asserting `hookEventName` equals the event's PascalCase name, plus `pre_tool_use_deny_carries_permission_decision`.

### HOOK-021 — the Stop handoff travels on a channel someone reads

**File** `cli/src/hooks/run.rs`, `fn stop` (non-blocking path) and `fn session_start`.

**Reference constraint**: Stop stdout goes to the debug log. The two channels that reach a reader are `systemMessage` (shown to the user) and `hookSpecificOutput.additionalContext` (added to Claude's context at the end of the turn, and the conversation continues so Claude can act on it).

**New stdout JSON** on the non-blocking Stop path:

```json
{
  "systemMessage": "Phase     4 · Build        STORY-003 · checkout-flow\nDone      3/3 ACs, 41 tests\nGate      PASS  coverage 86%\n\nNext      /verify STORY-003\nThen      /release\nBlocked   none\n\nFull report: .devforgeai/reports/STORY-003-build.yaml"
}
```

The handoff is addressed to the user (conventions §6: the `Next` line is what the user types), so `systemMessage` is the correct field and `additionalContext` is not a substitute. Keep `human` populated for the `--json`-free CLI invocation of `devforgeai handoff`, but suppress it from stdout whenever `hook_json` is set.

For `session_start`, keep plain stdout (the reference endorses it) unless `sessionTitle` or `watchPaths` are also being set, in which case switch to `{"hookSpecificOutput":{"hookEventName":"SessionStart","additionalContext":"...","sessionTitle":"..."}}`.

**Test first**: `stop_pass_emits_handoff_as_system_message` — a passing gate, dispatch `stop`, assert exit 0 and that the hook JSON's `systemMessage` contains `Next      /` and `Full report:`.

### HOOK-030 — honour `stop_hook_active`

**File** `cli/src/hooks/run.rs`, `fn stop`, first statement.

**New stdin fields read**: `stop_hook_active` (bool, already parsed at line 332 and discarded).

**Change**:

```rust
if p.bool_key("stop_hook_active") {
    // Claude Code is already continuing because of this hook. Blocking again
    // cannot change the outcome and burns one of the harness's 8 continuations.
    let mut out = outcome("stop", &[], 0, false);
    out.project = ctx.root.display().to_string();
    return Ok(out);
}
```

Place it before the `gate check` call so the continuation turn does not re-run the test suite (HOOK-033). Emit the handoff (HOOK-021) on this path so the FAIL handoff is what the user sees when the continuation ends.

**New exit code**: 0 whenever `stop_hook_active` is true, at any gate result.

**Test first**: `stop_active_flag_never_blocks` — a failing explore gate, dispatch `stop` with `{"stop_hook_active": true}` on a fresh `state.toml`, assert exit 0, `blocked == false`, and that `actions` does not contain `gate check`. Then amend `stop_different_id_resets_counter` (`cli/tests/hook_run.rs:665-687`), which currently asserts exit 2 on `stop_hook_active: true` and encodes the bug.

### HOOK-031 — the block counter is turn-scoped

**File** `cli/src/hooks/run.rs`, `fn stop`, block-counter section; `cli/src/state.rs` `[stop_hook]`.

**Change**: with HOOK-030 in place, `stop_hook_active` is the loop guard and the persisted counter becomes redundant. Keep `[stop_hook]` as reporting state only — record `blocked_phase`, `blocked_id`, `block_count`, and a new `blocked_session` (from the payload's `session_id`) — and make the block decision from `stop_hook_active` alone. When `blocked_session` differs from the incoming `session_id`, reset the counter before recording.

**New stdin fields read**: `session_id`.

**Test first**: `stop_blocks_again_in_a_new_session` — block once, rewrite `state.toml`'s `blocked_session` to a different value (or dispatch with a new `session_id`), assert the next Stop with `stop_hook_active: false` exits 2.

### HOOK-040 — read `last_assistant_message` first

**File** `cli/src/hooks/run.rs`, `fn subagent_stop` and `fn last_assistant`.

**New stdin fields read**: `last_assistant_message` (string), `agent_transcript_path` (string), `agent_id` (string, for the diagnostic text). `tool_response.content` is removed.

**Change**:

```rust
let content = p
    .opt_str(&["last_assistant_message"])
    .or_else(|| p.opt_str(&["agent_transcript_path"]).and_then(|t| last_assistant(&t)));
```

`transcript_path` does not appear in this function: it names the parent session's transcript.

**New exit code**: unchanged 0 on success; see HOOK-014 for the parse-failure path.

**Test first**: `subagent_stop_reads_last_assistant_message` — dispatch with a payload carrying `agent_type`, `last_assistant_message` holding the verifier envelope, and a `transcript_path` pointing at a decoy file whose last assistant message is different. Assert the report gets the envelope's counts and not the decoy's. Then rewrite `subagent_stop_registered_ingests` (`cli/tests/hook_run.rs:694-724`), which supplies `tool_response.content` and would otherwise keep passing against the removed path.

**Spec follow-up** (HOOK-042): amend `specs/01-cli.md:52` (drop `tool_response` from the SubagentStop key list, replace the `transcript_path` fallback with `agent_transcript_path`), `:1497` (the `subagent-stop` table row becomes `agent_type`, `last_assistant_message`, `agent_transcript_path`), and `:2764` (blocker item 80, which records the retired fallback chain as an open question the reference has since answered).

### HOOK-041 — fall back to the subagent's own transcript, parsed correctly

**File** `cli/src/hooks/run.rs`, `fn last_assistant`.

**Change**: accept `content` as either a string or an array of blocks, concatenating the `text` field of every block whose `type` is `"text"`. Keep the existing `v.content` and `v.message.content` shapes. Point the caller at `agent_transcript_path` (HOOK-040).

**Test first**: `last_assistant_reads_block_array_content` — a JSONL fixture whose final assistant line carries `{"type":"assistant","message":{"content":[{"type":"text","text":"{...envelope...}"}]}}`; assert the envelope is returned.

### HOOK-001 — match and handle PowerShell

**Files** `cli/templates/settings.hooks.json` and `hooks/settings.hooks.json` (keep byte-identical); `cli/src/hooks/run.rs:293`.

**Change**: matcher `"Bash|PowerShell"`; the dispatcher branch becomes `} else if tool == "Bash" || tool == "PowerShell" {`. The `tool_input.command` field name is identical for both tools (raw 1629-1648), so nothing else in the branch changes.

**Test first**: `post_tool_use_powershell_matching_test_command_runs_partial` — a clone of `post_tool_use_bash_matching_test_command_runs_partial` (`cli/tests/hook_run.rs:523`) with `"tool_name": "PowerShell"`, asserting `actions` contains `gate check --phase build --partial`. Add a settings-template test asserting the `PostToolUse` matcher string equals `Bash|PowerShell`.

### HOOK-006 — close the Bash-write hole

**Files** `cli/src/hooks/run.rs` (new `file-changed` event), `cli/src/hooks/settings.rs` (`EVENTS`), the settings template.

**Change**: add `file-changed` to `EVENTS` in `run.rs`, reading `file_path` from stdin and running `doc validate <path>` plus the producer check. Register it as a `FileChanged` group with an **omitted** matcher (a `"*"` matcher registers a literal file named `*`), and seed the watch list from `session_start` by returning `watchPaths` with the absolute paths of `.devforgeai/gates.toml`, `.devforgeai/config.toml`, `.devforgeai/state.toml`, and every file under `.devforgeai/stories/`, `adr/`, and `context/`. `FileChanged` cannot block (it fires after the change), so the output is `systemMessage` plus `additionalContext`; the enforcement it buys is detection, not prevention.

The prevention half is a permission deny rule, not a hook: add `"deny": ["Bash(*>*.devforgeai/*)"]`-class rules only if the project already ships a permissions block — otherwise document the residual hole in conventions §7.

**New exit code**: 0 always (`FileChanged` has no decision control).

**Test first**: `file_changed_validates_a_document_written_outside_the_tools` — dispatch `file-changed` with a `file_path` naming an invalid document, assert the hook JSON carries the `DFA-` code in `systemMessage`.

### HOOK-012 — PreToolUse fails closed on its own errors

**File** `cli/src/hooks/run.rs`, `fn dispatch` and `fn pre_tool_use`.

**Change**: wrap the `pre-tool-use` arm so that every `CliError` becomes a deny rather than a propagated exit 3/1/5:

```rust
"pre-tool-use" => pre_tool_use(ctx, &p).or_else(|e| {
    let reason = format!("devforgeai could not evaluate this write: {}", e.diag().map(|d| d.to_string()).unwrap_or_default());
    let mut out = outcome("pre-tool-use", &["fail-closed"], 2, true);
    out.stderr = vec![reason.clone()];
    out.hook_json = Some(deny(reason));
    Ok(out)
}),
```

Apply the same treatment to `stop` (the other blocking event), where an internal error currently exits 5 and skips the gate. Leave the non-blocking events propagating, since no exit code helps there.

**New exit code**: 2 in place of 1, 3, and 5 on `pre-tool-use` and `stop`.

**Test first**: `pre_tool_use_unparsable_config_denies` — corrupt `.devforgeai/config.toml`, dispatch a `Write` payload for a frontend path, assert exit 2 and a `permissionDecision` of `deny`. Then amend `hook_stdin_not_json_gives_e021` and `hook_missing_key_gives_e020` (`cli/tests/hook_run.rs:747-765`) for the two blocking events.

### HOOK-050 — trust blocks wherever blocking exists

**File** `cli/src/hooks/run.rs`, `fn blocks_on_trust_failure`.

**Change**: `matches!(event, "pre-tool-use" | "stop" | "subagent-stop")`. `SubagentStop` blocks on exit 2, so a tampered binary can stop the subagent instead of being ignored. On `session-start` and `post-tool-use`, no exit code blocks; emit `{"systemMessage": "devforgeai trust verify failed: <code>. Hooks are running an unverified binary."}` alongside exit 4 so the failure is at least visible to the user, and keep writing `TRUST_FAIL` into `[last_gate].result` so the next Stop's handoff carries it.

**Spec follow-up** (HOOK-051): amend conventions §8 to name the three blocking events and state that `SessionStart` and `PostToolUse` degrade to a user-visible notice.

**Test first**: `hook_trust_failure_blocks_on_subagent_stop` (exit 2) and `hook_trust_failure_warns_on_session_start` (exit 4 plus a `systemMessage` naming the code).

### HOOK-060 — raise the PreToolUse timeout

**File** the settings template.

**Change**: `"timeout": 600` (the reference default for command hooks; `PreToolUse` is not one of the events that lowers it). The spec's keystroke-latency argument is inverted by the reference: a cancelled `PreToolUse` hook does not block, so a short timeout buys latency at the cost of the gate. Latency is better addressed by the `if` filter in the template below, which stops the process from spawning on writes the hook would pass anyway, and by caching the trust hash (HOOK-053).

**Test first**: a settings-template test asserting `PreToolUse[0].hooks[0].timeout == 600`.

### HOOK-070 — a relative path that cannot be resolved denies instead of skipping

**Files** `cli/src/project.rs`, `cli/src/hooks/run.rs` (`fn pre_tool_use`, `fn matches_frontend`).

**Change**: add to `project.rs` a predicate that reports failure instead of falling back to the absolute path:

```rust
/// The project-relative form of `p`, or `None` when `p` is not under `root`.
/// Case-folded on Windows, so `c:\projects\…` resolves against a
/// canonicalised `C:\Projects\…`, and short names are resolved first.
pub fn rel_under(root: &Path, p: &Path) -> Option<String> { /* … */ }
```

`rel_display` stays the display helper it is named for and stops being used as a predicate; its silent `Err` fallback to the whole absolute path is the defect. In `pre_tool_use`, the glob arm and the `story files --check` arm both take `rel_under(&ctx.root, path)`. When it returns `None` — a path genuinely outside the project, a short name, a junction, a UNC spelling — deny with `permissionDecisionReason: "path does not resolve under the project root"` and exit 2. Skipping the check is the fail-open behaviour the finding describes.

**New exit code**: 2 where the current code silently passes.

**Test first**: `pre_tool_use_lowercase_drive_letter_still_lints` — a token violation at `c:\…\src\Button.tsx` (lower-case drive letter against a canonicalised root) asserts exit 2 and a `design lint` entry in `actions`. Then `pre_tool_use_unresolvable_path_denies` for the `None` arm.

### HOOK-071 — the `.devforgeai/` test is case-folded and segment-anchored

**File** `cli/src/hooks/run.rs:167, 194, 273`.

**Change**: add to `project.rs`:

```rust
/// A hook path normalized for segment matching: forward slashes, lower-cased
/// on Windows, with a leading and trailing slash so a segment test cannot
/// match a partial name.
pub fn hook_path_key(p: &Path) -> String { /* … */ }
```

Every "under `.devforgeai/`" test in `pre_tool_use` and `post_tool_use` becomes `hook_path_key(path).contains("/.devforgeai/")`. The three current spellings (`rel.starts_with(".devforgeai/") || rel.contains("/.devforgeai/")`) collapse into that one call, and the `starts_with` arm disappears with them, since the key always carries a leading slash.

**New exit code**: unchanged; the change is which paths reach the producer check.

**Test first**: `pre_tool_use_mixed_case_dot_dir_hits_the_producer_check` — dispatch a `Write` payload for `…\.DevForgeAI\stories\STORY-001.md` carrying a producer mismatch, assert exit 2 and `doc validate --producer-check` in `actions`. Add `hook_path_key_does_not_match_a_partial_segment` as a unit test (`my.devforgeai.bak/` does not match).


### Replacement `hooks/settings.hooks.json`

Exec form throughout (`command` + `args`), so the absolute-path token is one argument and cannot be split at a space. `hook install` substitutes `@@DEVFORGEAI@@` with the absolute `.exe` path in forward-slash form and stops preferring the bare `PATH` name, because exec form resolves `command` as an executable and the absolute path removes the ambiguity. The `if` rules cut the spawn on writes no arm would examine; the third `PreToolUse` handler carries no `if` because the declared-set and token arms need every path. `hook install` writes the `Bash(...)`/`PowerShell(...)` rules of the `PostToolUse` shell handlers from `config.toml`'s `test_command` values, and the `SubagentStop` matcher from the `[[verifier]]` names — both are per-project and cannot be a static template, so the block below shows the shape with `@@TEST_COMMAND@@` and `@@VERIFIERS@@` as the two additional substitution tokens.

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup|resume|clear|fork",
        "hooks": [
          {
            "type": "command",
            "command": "@@DEVFORGEAI@@",
            "args": ["hook", "run", "session-start"],
            "timeout": 120,
            "statusMessage": "devforgeai: detecting stack"
          }
        ]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "Write|Edit|NotebookEdit",
        "hooks": [
          {
            "type": "command",
            "command": "@@DEVFORGEAI@@",
            "args": ["hook", "run", "pre-tool-use"],
            "timeout": 600,
            "statusMessage": "devforgeai: checking the write"
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Write|Edit|NotebookEdit",
        "hooks": [
          {
            "type": "command",
            "command": "@@DEVFORGEAI@@",
            "args": ["hook", "run", "post-tool-use"],
            "if": "Edit(**/.devforgeai/**)",
            "timeout": 60
          },
          {
            "type": "command",
            "command": "@@DEVFORGEAI@@",
            "args": ["hook", "run", "post-tool-use"],
            "if": "Write(**/.devforgeai/**)",
            "timeout": 60
          },
          {
            "type": "command",
            "command": "@@DEVFORGEAI@@",
            "args": ["hook", "run", "post-tool-use"],
            "if": "NotebookEdit(**/.devforgeai/**)",
            "timeout": 60
          }
        ]
      },
      {
        "matcher": "Bash|PowerShell",
        "hooks": [
          {
            "type": "command",
            "command": "@@DEVFORGEAI@@",
            "args": ["hook", "run", "post-tool-use"],
            "if": "Bash(@@TEST_COMMAND@@)",
            "async": true,
            "timeout": 900,
            "statusMessage": "devforgeai: partial build gate"
          },
          {
            "type": "command",
            "command": "@@DEVFORGEAI@@",
            "args": ["hook", "run", "post-tool-use"],
            "if": "PowerShell(@@TEST_COMMAND@@)",
            "async": true,
            "timeout": 900,
            "statusMessage": "devforgeai: partial build gate"
          }
        ]
      },
      {
        "matcher": "Agent",
        "hooks": [
          {
            "type": "command",
            "command": "@@DEVFORGEAI@@",
            "args": ["hook", "run", "agent-result"],
            "timeout": 60
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "@@DEVFORGEAI@@",
            "args": ["hook", "run", "stop"],
            "timeout": 900,
            "statusMessage": "devforgeai: phase gate"
          }
        ]
      }
    ],
    "SubagentStop": [
      {
        "matcher": "@@VERIFIERS@@",
        "hooks": [
          {
            "type": "command",
            "command": "@@DEVFORGEAI@@",
            "args": ["hook", "run", "subagent-stop"],
            "timeout": 60
          }
        ]
      }
    ],
    "FileChanged": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "@@DEVFORGEAI@@",
            "args": ["hook", "run", "file-changed"],
            "timeout": 60
          }
        ]
      }
    ]
  }
}
```

Notes on the block, each tied to a finding above: the `Stop` and `FileChanged` groups omit `matcher` (HOOK-004; a `"*"` on `FileChanged` would register a literal file named `*`, HOOK-095); the `SessionStart` matcher drops `compact` (HOOK-005), and the narrower alternative, which keeps the handoff re-injected at the moment context was discarded, is to register all five sources and have `session_start` read `source` and skip the `stack detect` call when it is `compact` — pick one, since the fix already reads `source` for `sessionTitle`; the `Agent` `PostToolUse` group is the parent-session half of ingest (HOOK-097) and needs a new `agent-result` event in the `EVENTS` enum; `FileChanged` needs `session_start` to return `watchPaths` before the watcher starts (HOOK-095, HOOK-006). Adding `NotebookEdit` to the two file matchers closes HOOK-007, and requires `pre_tool_use` to read `tool_input.notebook_path` when `file_path` is absent rather than raising `DFA-E020` (HOOK-072).

One caution on the `if` rules: `"Bash(@@TEST_COMMAND@@)"` reintroduces HOOK-001's failure shape at the config layer. A `test_command` that does not render as valid permission-rule syntax, or that differs from what Claude actually types (`npm test` against `npm run test`), leaves a handler that never fires and a partial gate that dies without a notice. The dispatcher already compares the command to `config.toml` internally (`run.rs:294-303`), so these two `if` rules are pure spawn-cost optimization with a silent-death downside: have `hook install` validate the rendered rule and omit the `if` when it does not parse.
