# The session history

Read this before workflow step 4, and read it whenever `sources.sessions` needs a value written. It carries the project-key derivation, where the files sit, how a JSON Lines session file reads, and what each of the four `sessions.status` values means for the run.

The CLI resolves all of this and puts the result in the aggregate's `sessions` block. This document says how that block was produced, so the run can say in the report where the window was read from and why a session was left out.

## The project key

`<project-key>` is the absolute path of the project root with every character outside `[A-Za-z0-9]` replaced by `-`, one replacement character per source character. The path is not shortened, lower-cased, or collapsed: two adjacent separators become two dashes.

| Project root | `<project-key>` |
|---|---|
| `C:\Projects\DevForgeAI` | `C--Projects-DevForgeAI` |
| `C:\Projects\New folder` | `C--Projects-New-folder` |
| `\\wsl$\Ubuntu\home\bryan\Projects\DevForge` | `--wsl--Ubuntu-home-bryan-Projects-DevForge` |

`.devforgeai/config.toml` `[reflect].session_key` overrides the derived value when it is non-empty. The override exists for two cases: an eval workspace, whose temp-directory path is not knowable when the case file is written, and a future Claude Code release that names its project directories differently. Claude Code owns this naming, and a changed scheme leaves `sessions.status` at `absent` rather than producing a wrong reading.

## Where the files sit

`<session_root>/<project-key>/*.jsonl`, one file per session, named for the session id.

`<session_root>` is `.devforgeai/config.toml` `[reflect].session_root`, default `~/.claude/projects`, with `~` expanding to the user's home directory. `devforgeai report aggregate --session-root <path>` overrides it for one call. A root resolving outside the user's home directory is `DFA-E421`: the aggregate comes back with `sessions.status: unreadable` and every other block filled, and the run continues.

## Reading a session file

Each line of a `*.jsonl` file is one JSON object and the file carries no enclosing array, so a line reads on its own and a malformed line costs that line alone. The fields this skill reads:

| Field | Type | Meaning |
|---|---|---|
| `type` | string | `user` marks a line the user typed |
| `message.content` | string | the typed text, which for a slash command is the command and its arguments |
| `isMeta` | bool | `true` marks a line Claude Code inserted rather than the user; absent means false |
| `timestamp` | string | RFC 3339 UTC; the `at` value of an evidence entry |
| `sessionId` | string | the session id, equal to the file stem |

A slash command is a line whose `type` is `user`, whose `isMeta` is absent or false, and whose `message.content` matches `^/(explore|discover|constitute|plan|build|verify|release|design|reflect)\b`. Those lines are the aggregate's `sessions.commands[]`, each carrying its `command`, its `args`, its `at`, its `session_id`, its 1-based `line`, the `remedy_ids` its `--remedy` flag named, and whether `--resume` was present.

`sessions.repeats[]` groups those commands by `command` and by `key`, where `key` is the first argument joined to the sorted `--remedy` ids by `|`, and holds only the groups whose `count` is two or more. `session-pattern-reader` reads the lines each group points at, a few either side, because the repetition's meaning is in the turns between the typings rather than in the typings themselves.

Line numbers are 1-based and count every line of the file, malformed lines included, so a line number in `evidence[]` opens the same line the aggregate counted.

## The four status values

| `sessions.status` | Condition | `files`, `commands`, `repeats` | `reason` |
|---|---|---|---|
| `present` | the directory holds at least one readable `*.jsonl` | filled | `""` |
| `absent` | `<session_root>/<project-key>/` does not exist | `[]` | the resolved path |
| `empty` | the directory exists and holds no `*.jsonl` file | `[]` | the resolved path |
| `unreadable` | the directory exists and cannot be read, or `DFA-E421` resolved the root outside the user's home | `[]` | the resolved path |

Only `present` invokes `session-pattern-reader`. On the other three, workflow step 4 returns `observations: []`, `sources.sessions.reason` carries the resolved path, and every `OBS-nnn` in the run cites a report path.

An absent session directory leaves the gate result unchanged. The `reflect-obs-cites-source` check accepts a report path or a session id, and a report-only run supplies the first for every observation, so a window read from reports alone passes the gate and says so in `sources.sessions`.
