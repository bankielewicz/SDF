---
schema: devforgeai-spec/1
doc: cli
status: draft
produced_by: cli-spec-author
consumes: [00-conventions]
open_questions: []
---

# devforgeai CLI

## Scope

The `devforgeai` binary is the enforcement core of the framework. It detects the project stack, writes and reads the three project state files (`config.toml`, `gates.toml`, `state.toml`), validates every typed document defined in conventions §5, evaluates the phase gates defined in `gates.toml`, runs the project's own test, coverage, and lint commands, ingests verifier subagent output into reports, renders the conventions §6 handoff block, installs the Claude Code hooks and the three git hooks, and verifies its own binary against a human-pinned SHA-256. It is a single static executable with no network access, no LLM calls, and no interactive prompts. Every subcommand is deterministic given the file system, the environment, and the exit status of the commands named in `config.toml`.

The binary does not author content. It writes no prose, no requirement, no story, no ADR, and no code. It does not edit documents a skill produced, beyond allocating an ID on request and writing CLI-owned report files. It contains no language-specific logic beyond the marker-file table and the default command table in `## Outputs`; every command it executes comes from `config.toml`, which a human may edit. It renders no judgement about whether a document is good, only whether it parses, resolves, and satisfies the checks listed in `gates.toml`. No gate decision reads Python output, and no grader in `evals/` is read by any subcommand.

## Inputs

Files read, all paths relative to the project root, which is the nearest ancestor directory of the working directory containing a `.devforgeai/` directory unless `--project <path>` overrides it.

| Path | Format | Read by | Absent behaviour |
|---|---|---|---|
| `.devforgeai/config.toml` | TOML | every subcommand except `init`, `trust pin`, `trust verify` | error `DFA-E101`, exit 1 |
| `.devforgeai/gates.toml` | TOML | `gate require`, `gate check`, `phase set`, `hook run` | error `DFA-E102`, exit 1 |
| `.devforgeai/state.toml` | TOML | `gate require`, `gate check`, `handoff`, `phase set`, `doc validate --producer-check`, `report ingest`, `hook run` | error `DFA-E103`, exit 1 |
| `~/.devforgeai/trust.toml` | TOML | `trust verify`, `trust pin` | `trust verify` error `DFA-E501`, exit 4 |
| `.devforgeai/explore/brief.md` | MD + frontmatter | `doc validate`, `doc load explore-brief`, `gate check --phase explore` | per check kind |
| `.devforgeai/explore/decision.yaml` | YAML | `doc validate`, `doc load explore-decision`, `gate check --phase explore` | per check kind |
| `.devforgeai/requirements.yaml` | YAML | `doc validate`, `doc load requirements`, `gate check --phase discover`, `story validate` | per check kind |
| `.devforgeai/context/{tech-stack,source-tree,dependencies,coding-standards,architecture-constraints,anti-patterns}.md` | MD + frontmatter | `doc validate`, `doc load context`, `context audit`, `gate check --phase constitute` | per check kind |
| `.devforgeai/adr/ADR-nnn.md` | MD + frontmatter | `doc validate`, `doc load adr`, `context audit` | per check kind |
| `.devforgeai/stories/STORY-nnn.md` | MD + frontmatter | `doc validate`, `doc load story`, `story validate`, `gate check --phase plan\|build\|verify` | per check kind |
| `.devforgeai/stories/sprint.yaml` | YAML | `doc validate`, `story validate`, `gate check --phase plan`, git `pre-push` | per check kind |
| `.devforgeai/ui-specs/UI-nnn.md` | MD + frontmatter | `doc validate`, `doc load ui-spec` | per check kind |
| `.devforgeai/brand/tokens.json` | JSON | `design lint` | error `DFA-E120`, exit 1 |
| `.devforgeai/reports/<ID>-<phase>.yaml` | YAML | `handoff`, `report show`, `report ingest`, `gate check` (`verifier_pass`) | per check kind |
| `.devforgeai/releases/vX.Y.Z.yaml` | YAML | `doc validate`, `gate check --phase release` | per check kind |
| coverage report named by `config.toml` | lcov, cobertura XML, jacoco XML, go-cover, `devforgeai-json` | `gate check --phase build` | check `coverage_min` fails with `DFA-E312` |
| `cli/REVISION`, `cli/DIGEST` | text | `trust pin`, `trust verify` | `trust pin` error `DFA-E510`, exit 4 |
| `.claude/settings.json` | JSON | `init`, `hook install` | created |

Environment read:

| Variable | Type | Read by | Effect |
|---|---|---|---|
| `CLAUDECODE` | string | `trust pin`, `trust verify` | non-empty after trimming means a Claude session is active |
| `CLAUDE_CODE_ENTRYPOINT` | string | `trust pin`, `trust verify` | non-empty after trimming means a Claude session is active |
| `DEVFORGEAI_HOME` | path | `trust pin`, `trust verify` | replaces `~/.devforgeai`; read only when the crate is compiled with the `test-home` feature, ignored otherwise |
| `DEVFORGEAI_NOW` | RFC 3339 string | every subcommand that writes a timestamp | replaces the system clock; read only under the `test-home` feature |

`--project` defaults to the nearest ancestor of the working directory holding a `.devforgeai/` directory. One directory is skipped by that walk: a `.devforgeai/` that holds no `config.toml` and is the trust home `~/.devforgeai/`. The trust store and a project share a directory name, and a session whose working directory is the home directory would otherwise resolve the trust store as its project and read a `gates.toml` that is not there. A `~/.devforgeai/` that does hold a `config.toml` is a project like any other, so a user who initialises one in their home directory keeps it.

Standard input is read by `hook run <event>` only. The Claude Code hook JSON object arrives on stdin as a single UTF-8 document terminated by EOF; the dispatcher parses it with `serde_json` and reads the keys `hook_event_name` (string), `session_id` (string), `cwd` (string), `tool_name` (string, PreToolUse and PostToolUse), `tool_input` (object, PreToolUse and PostToolUse), `stop_hook_active` (bool, Stop), `agent_type` (string, SubagentStop and the PreToolUse metrics arm), `last_assistant_message` (string or content array, SubagentStop), and `agent_transcript_path` (string, SubagentStop fallback — the subagent's own transcript, not the parent session's `transcript_path`, whose last assistant message is the orchestrator's text). A missing key that the event needs is error `DFA-E020`, exit 3. Unknown keys are ignored.

## Outputs

### Runtime artifacts

| Path | Written by | Format |
|---|---|---|
| `.devforgeai/config.toml` | `init`, `stack detect` | TOML, schema below |
| `.devforgeai/gates.toml` | `init` | TOML, schema below, default content in `## Gate` |
| `.devforgeai/state.toml` | `init`, `phase set`, `gate check`, `handoff`, `hook run stop` | TOML, schema below |
| `~/.devforgeai/trust.toml` | `trust pin` | TOML, schema below |
| `.devforgeai/reports/<ID>-<phase>.yaml` | `gate check`, `report ingest` | YAML, schema below |
| `.devforgeai/context/*.md` | `init --analyze` | MD + §5 frontmatter |
| `.devforgeai/stories/STORY-<nnn>.md` | `phase set`, the frontmatter `status` value alone | MD + §5 frontmatter |
| `.claude/settings.json` | `init`, `hook install` | JSON, hooks block in `## Templates` |
| `.git/hooks/{pre-commit,commit-msg,pre-push}` | `init`, `hook install` | POSIX sh, content in `## Templates` |
| `.devforgeai/coverage.*` | the project's own coverage command | per `coverage_format` |

All writes are atomic: the binary writes `<path>.tmp-<pid>` in the destination directory, flushes, then renames over the destination. A rename failure is error `DFA-E901`, exit 5. TOML files the CLI rewrites are edited with `toml_edit` so human comments and key order survive; TOML files the CLI creates are serialised in the key order shown below.

### `.devforgeai/config.toml`

```toml
schema = "devforgeai/config/1"                 # string; fixed; written by `stack detect`
generated_at = "2026-09-10T14:02:11Z"          # string; RFC 3339 UTC; rewritten on every `stack detect`
cli_version = "1.0.0"                          # string; semver of the binary that wrote the file
degraded = false                               # bool; default false; true when `stack` is empty

[[stack]]                                      # zero or more tables; detection order is the table order
id = "rust"                                    # string; enum: rust | node | python | go | dotnet | jvm | ruby
markers = ["Cargo.toml"]                       # array[string]; default []; repo-relative paths that matched
package_manager = "cargo"                      # string; enum per ecosystem; "" when undetermined
source_roots = ["src"]                         # array[string]; default per ecosystem table
test_command = "cargo test --all-features"     # string; default per ecosystem table; "" when unavailable
coverage_command = "cargo llvm-cov --all-features --lcov --output-path target/devforgeai/lcov.info"
                                               # string; default per ecosystem table; "" when unavailable
coverage_format = "lcov"                       # string; enum: lcov | cobertura | jacoco | go-cover | devforgeai-json | none
coverage_paths = ["target/devforgeai/lcov.info"]  # array[string]; glob patterns; default per ecosystem table
lint_command = "cargo clippy --all-targets --all-features -- -D warnings"
                                               # string; default per ecosystem table; "" when unavailable
complexity_command = ""                        # string; default ""; exits non-zero above the project's own complexity ceiling
timeout_secs = 900                             # integer; default 900; per-command wall clock limit
env = {}                                       # table[string,string]; default {}; added to the command environment

[frontend]
globs = [                                      # array[string]; default as shown; matched against repo-relative paths
  "**/*.css", "**/*.scss", "**/*.sass", "**/*.less",
  "**/*.vue", "**/*.svelte", "**/*.jsx", "**/*.tsx",
  "**/*.html", "**/*.styles.ts", "**/*.styles.js"
]
tokens_path = ".devforgeai/brand/tokens.json"  # string; default as shown
exclude = ["**/node_modules/**", "**/dist/**", "**/build/**", "**/vendor/**",
           ".explore-prototype/**", ".devforgeai/explore/mockups/**"]
                                               # array[string]; default as shown; the last two are the sketch paths design lint skips

[[layer]]                                      # exactly four tables; names are a closed enum; order is match order
name = "domain"                                # string; enum: domain | application | infrastructure | interface
globs = ["**/domain/**", "**/core/**", "**/entities/**", "**/model/**", "**/models/**"]
coverage_min = 95.0                            # float; percent of covered lines; default 95.0

[[layer]]
name = "application"
globs = ["**/application/**", "**/services/**", "**/usecases/**", "**/use_cases/**", "**/handlers/**"]
coverage_min = 85.0                            # float; default 85.0

[[layer]]
name = "infrastructure"
globs = ["**/infrastructure/**", "**/infra/**", "**/adapters/**", "**/repositories/**", "**/persistence/**", "**/db/**"]
coverage_min = 80.0                            # float; default 80.0

[[layer]]
name = "interface"
globs = ["**/api/**", "**/controllers/**", "**/routes/**", "**/cli/**", "**/ui/**", "**/components/**", "**/pages/**"]
coverage_min = 70.0                            # float; default 70.0

[explore]                                      # phase-scoped configuration; read by `phase set explore` and the explore gate
timebox_days = 5                               # integer; default 5; calendar days from [explore].started_at
remedy_timebox_days = 1                        # integer; default 1; calendar days from [explore].remedy_started_at

[plan]
sprint_capacity_points = 20                    # integer; default 20; the maximum sum of stories[].points in one sprint
story_points = [1, 2, 3, 5, 8]                 # array[integer]; default as shown; the closed set stories[].points is drawn from

[build]
worktree_root = "../wt"                        # string; default "../wt"; resolved against the project root
branch_prefix = "story/"                       # string; default "story/"
base_ref = "HEAD"                              # string; default "HEAD"; the ref a story worktree branches from
complexity_max = 10                            # integer; default 10; the per-function branch ceiling refactor aims below
duplication_max_percent = 5.0                  # float; default 5.0; the share of a file's lines that may repeat

[verify]
mode = "light"                                 # string; enum light | deep; default "light"
complexity_max = 10                            # integer; default 10
duplication_max_percent = 5.0                  # float; default 5.0
duplication_min_lines = 20                     # integer; default 20
metrics_command = ""                           # string; default ""; prints one devforgeai-metrics/1 JSON object
call_graph_command = ""                        # string; default ""; prints one devforgeai-callgraph/1 JSON object

[release]
platform = ""                                  # string; enum kubernetes | compose | github-actions | vps | none; "" resolves at run time
ci = "github-actions"                          # string; enum github-actions | none; default "github-actions"
deploy_root = "deploy"                         # string; default "deploy"; project-relative
docs_root = "docs"                             # string; default "docs"; project-relative
build_command = ""                             # string; default ""; the project's own release build
package_command = ""                           # string; default ""; the project's own packaging step
artifact_paths = []                            # array[string]; glob patterns; default []
api_symbols_command = ""                       # string; default ""; prints one line per public symbol
image_name = ""                                # string; default ""; the container image reference
service_port = 8080                            # integer; default 8080

[reflect]
session_root = "~/.claude/projects"            # string; default as shown; ~ expands to the user's home directory
session_key = ""                               # string; default ""; "" derives the key from the project path
window_days = 14                               # integer; default 14; the --since default when the flag carries no date

[coverage]
overall_min = 80.0                             # float; default 80.0; percent over all files under source_roots
unassigned_policy = "report"                   # string; enum: report | fail; default "report"
exclude = ["**/tests/**", "**/test/**", "**/*_test.*", "**/*.test.*", "**/*.spec.*", "**/mocks/**"]
                                               # array[string]; default as shown; excluded from every layer and the overall figure

[[verifier]]                                   # the registry of subagents SubagentStop ingests; one table per subagent
name = "ac-compliance-verifier"                # string; kebab-case subagent name; unique
phase = "verify"                               # string; phase enum; the report this verifier writes into
report_field = "verifiers.ac_compliance"       # string; dotted path inside the report YAML
unit = "ACs"                                   # string; default "checks"; the unit word in the handoff Verified line
required = true                                # bool; default false; a `verifier_pass` check names it by `name`
```

`init` writes twenty `[[verifier]]` tables, the array `specs/11-subagent-catalog.md` Decision 1 fixes, in this order, which is the order the handoff tie-break of rendering rule 6 reads:

| Phase | `name` | `report_field` | `unit` | `required` |
|---|---|---|---|---|
| explore | `kill-case-builder` | `verifiers.kill_case` | `objections` | `true` |
| discover | `flow-integrity-auditor` | `verifiers.flow_integrity` | `flows` | `false` |
| constitute | `architecture-reviewer` | `verifiers.architecture_reviewer` | `requirements` | `false` |
| constitute | `alignment-auditor` | `verifiers.alignment_auditor` | `checks` | `false` |
| plan | `story-invest-auditor` | `verifiers.story_invest` | `stories` | `true` |
| build | `ac-test-writer` | `verifiers.ac_testable` | `AC` | `true` |
| build | `story-ac-verifier` | `verifiers.story_ac` | `ACs` | `true` |
| build | `context-validator` | `verifiers.context` | `files` | `true` |
| verify | `ac-compliance-verifier` | `verifiers.ac_compliance` | `ACs` | `true` |
| verify | `standards-reviewer` | `verifiers.standards` | `files` | `true` |
| verify | `anti-pattern-scanner` | `verifiers.anti_patterns` | `anti-patterns` | `true` |
| verify | `constraint-auditor` | `verifiers.constraints` | `constraints` | `true` |
| verify | `coverage-gap-auditor` | `verifiers.coverage_gaps` | `layers` | `true` |
| verify | `dead-code-detector` | `verifiers.dead_code` | `symbols` | `true` |
| verify | `deferral-validator` | `verifiers.deferrals` | `deferrals` | `true` |
| verify | `security-auditor` | `verifiers.security` | `OWASP categories` | `true` |
| verify | `code-quality-auditor` | `verifiers.quality` | `files` | `true` |
| verify | `adr-conformance-reviewer` | `verifiers.adr_conformance` | `ADRs` | `true` |
| design | `requirement-coverage-auditor` | `verifiers.requirement_coverage` | `screens` | `false` |
| release | `deferral-auditor` | `verifiers.deferrals` | `deferrals` | `true` |

Twenty registrations across seven phases. One `name` binds to one `phase`, which is why Build's `story-ac-verifier` and Verify's `ac-compliance-verifier` are separate registrations rather than one name in two phases. `required = false` marks a verifier no `verifier_pass` check names: `flow-integrity-auditor` reports into a document Discover writes, `architecture-reviewer` and `alignment-auditor` are read by `report_metric` checks, and `requirement-coverage-auditor` reports into a design report no gate reads.

`stack detect` writes `[[stack]]`, `[frontend].globs`, and `degraded`. It leaves `[[layer]]`, `[coverage]`, `[[verifier]]`, and the six phase tables `[explore]`, `[plan]`, `[build]`, `[verify]`, `[release]`, `[reflect]` at their previous values when the file exists, and writes the defaults above when it does not.

### `.devforgeai/gates.toml`

```toml
schema = "devforgeai/gates/1"                  # string; fixed
cli_min_version = "1.0.0"                      # string; semver; `gate check` exits 1 (DFA-E304) below it

[[gate]]
phase = "build"                                # string; enum: explore | discover | constitute | plan | build | verify | release | reflect; unique across gates
requires = "plan"                              # string; phase enum or ""; the predecessor `gate require` tests; default: the preceding phase
on_fail = "send_back"                          # string; enum: fail | send_back; default "fail"
send_back_to = "plan"                          # string; phase enum; required when on_fail = "send_back", else ""
document = ""                                  # string; default ""; the default `path` of every check in this gate
description = "Story implemented, tests green, coverage by layer met."   # string; default ""

  [[gate.check]]
  kind = "tests_pass"                          # string; closed enum, table below
  id = "build-tests"                           # string; unique within the gate; used in report and error text
  severity = "block"                           # string; enum: block | warn; default "block"; warn leaves the gate result unchanged
  on_fail = "fail"                             # string; enum: fail | send_back; default: the gate's on_fail
  message = "tests failed for {id}"            # string; default ""; printed in the report reason and the handoff evidence
  # kind-specific keys follow; unknown keys are error DFA-E302
```

Four optional condition keys are accepted on every check, each an inline table drawn from the closed key set `path`, `field`, `equals`, `in`, `state_field`, `is_empty`, `doc_exists`. A key outside that set is `DFA-E302`.

| Key | True means |
|---|---|
| `skip_when` | the check is recorded `skip` with `reason: condition` and does not affect the result |
| `required_when` | absent or false, the check is recorded `skip` with `reason: not_required` |
| `null_when` | the named field is expected to hold null or be absent; holding a value is a failure |
| `empty_when` | the named field is expected to hold an empty list; holding entries is a failure |

`message` accepts the placeholders `{id}`, `{phase}`, `{value}`, and `{limit}`, replaced with the gate subject ID, the gate phase, the value the check read, and the bound it compared against.

Check kinds, a closed enum of thirty. Any other value is error `DFA-E301`, exit 1. Every path or document name in a kind-specific key may contain the tokens `{id}` and `{phase}`, replaced with the gate subject ID and the gate phase before the path is resolved. A relative path in any key resolves against `.devforgeai/`, except one beginning with `.`, which resolves against the project root; `reports/{id}-qa.yaml` and `.devforgeai/stories/sprint.yaml` therefore name the same kind of location and `.explore-prototype` names a directory outside `.devforgeai/`. A check's `path` defaults to the gate's `document`. A `path` key always names a document and never a location inside one; a `field`, `from`, `to`, `cover`, or `universe` key names a location inside it, written in the dotted form `requirements[].actor`, where `[]` iterates a sequence. The two keys are never interchangeable: reading `path` as a field whenever the gate carried a `document` gave one key two meanings and made a check carrying neither key fail unconditionally.

| `kind` | Keys, types, defaults | Passes when |
|---|---|---|
| `doc_valid` | `docs` array[string], default `[]` | every matched file passes `doc validate` |
| `ids_resolve` | `prefixes` array[string], default `[]`; `from` string, default `""`; `to` string, default `""` | with `from` and `to`, every value at `from` equals some value at `to`; otherwise every reference of those prefixes resolves in the ID index |
| `file_exists` | `paths` array[string], required; `min_count` integer, default `paths.len()`; `absent` bool, default `false` | at least `min_count` paths exist and are non-empty, or with `absent = true`, none of them exists |
| `fields_present` | `collection` string, default `""`; `path` string, default the gate's `document`; `field` string, default `""`; `fields` array[string], default `[]` | with `collection`, every entry carries every name in `fields` with a non-empty value; with no `collection`, every value the `field` location names is non-null, and with no `field` either, the document itself is non-null |
| `field_in_enum` | `path` string, required; `field` string, required; `values` array[string], required | the value at `field` is a member of `values` |
| `field_is_date` | `path` string, required; `field` string, required; `format` string, default `"%Y-%m-%d"`; `after_field` string, default `""` | the value parses in `format` and, with `after_field` set, is later than that field's value |
| `length_between` | `path` string, default the gate's `document`; `field` string, required; `min` integer, default `0`; `max` integer, default `2147483647`; `exclude_status` array[string], default `[]` | every list the `field` location names holds between `min` and `max` entries after dropping entries whose `status` is in `exclude_status`; a null at the location is a list of zero, and a scalar or a mapping there is a failure |
| `no_open_questions` | `docs` array[string], default `[]` | `open_questions` is `[]` in every matched document |
| `set_cover` | `cover` string, required; `universe` string, required; `exclude_status` array[string], default `[]`; `status_field` string, default `"status"` | the multiset at `cover` equals the set at `universe` minus excluded records, with no duplicate. The identity of a universe record is the leaf segment of `universe` — `requirements[].key` keys on `key` — rather than a hardcoded `id`, and a `universe` with no leaf after `[]` keys on `id`. A record's status is read from `status_field` |
| `row_count_between` | `path` string, required; `section` string, required; `min` integer, default `0`; `max` integer, default `2147483647` | the named H2 section holds between `min` and `max` markdown table data rows, or that many non-blank body lines when the section holds no table |
| `column_matches` | `path` string, required; `section` string, required; `column` string, required; `pattern` string, required; `unique` bool, default `false` | every cell of the named column matches `pattern`, and with `unique = true` no value repeats |
| `column_contains_all` | `path` string, required; `section` string, required; `column` string, required; `state_field` string, required | every value of the `state.toml` array at `state_field` appears in the named column |
| `elapsed_days_at_most` | `started_field` string, required; `limit_field` string, required; `remedy_started_field` string, default `""`; `remedy_limit_field` string, default `""` | whole calendar days from the state timestamp to now are at or below the state integer limit; the `remedy_*` pair is used instead when both are set and the remedy start is non-empty |
| `context_audit` | `allow_warn` bool, default `false` | `context audit` exits 0, or exits 1 with warnings only and `allow_warn = true` |
| `story_valid` | `scope` string, enum `active` \| `sprint` \| `all`, default `active` | `story validate` exits 0 for the scope |
| `tests_pass` | `stacks` array[string], default `[]` meaning every stack; `allow_empty` bool, default `false` | every named stack's `test_command` exits 0; a `""` command passes when `degraded = true`, fails `DFA-E310` otherwise |
| `coverage_min` | `layers` array[string], default every layer name; `overall` bool, default `true`; `source` string, enum `run` \| `read`, default `run` | every named layer's covered-line percentage is at or above its `config.toml` `coverage_min`, and the overall figure is at or above `[coverage].overall_min` |
| `lint_clean` | `stacks` array[string], default `[]` meaning every stack | every named stack's `lint_command` exits 0; a `""` command is `skip` |
| `verifier_pass` | `verifiers` array[string], required; `min_ratio` float, default `1.0` | the named verifier's ingested block exists in the report and `passed / total` is at or above `min_ratio` |
| `report_metric` | `metric` string, required, a dotted path into the report with an optional `.length` suffix that reads a sequence's length; `op` string, enum `eq` \| `ne` \| `lt` \| `lte` \| `gt` \| `gte`, default `eq`; `value` float, default `0` | the metric compares true against `value` under `op`; an absent metric is a failure |
| `design_tokens` | `paths` array[string], default `[]` meaning the frontend globs | `design lint` exits 0 over the matched files |
| `no_cycle` | `docs` array[string], required; `root` string, default `""`; `from` string, required; `to` string, required; `prefix` string, default `""` | the directed graph whose nodes are the `from` values of the matched documents and whose edges are their `to` values holds no cycle through `root`, or no cycle at all when `root` is `""`; a `to` value naming a document outside `docs` is a leaf |
| `complexity_clean` | `stacks` array[string], default `[]` meaning every stack | every named stack's `complexity_command` exits 0; a `""` command is `skip` with `reason: no_complexity_command` |
| `antipattern_clean` | `min_severity` string, enum `blocker` \| `high` \| `medium` \| `low`, default `high`; `scope` string, enum `story` \| `project`, default `story` | `antipattern scan` exits 0 over the story's `## Files` set with `scope = "story"`, and over `[[stack]].source_roots` with `scope = "project"` |
| `files_declared` | `base` string, default `""` | `story files --diff` exits 0 for the gate subject; with `base` at `""` the check resolves the subject's worktree through `worktree list`, runs the diff inside it, and takes the base as the merge-base of `[build].base_ref` and that worktree's branch head, falling back to the project root against `[build].base_ref` when the subject has no worktree |
| `release_stories` | `path` string, default `releases/{id}.yaml`; `require_status` array[string], default `["built", "released"]`; `require_report` string, default `verify`; `require_result` string, default `PASS` | every `stories[].id` in `path` names a file `stories/<id>.md` whose frontmatter `status` is in `require_status`, and a report `reports/<id>-<require_report>.yaml` whose `gate.result` equals `require_result` |
| `deploy_manifest` | `path` string, default `releases/{id}.yaml`; `platform` string, default `""` meaning `config.toml` `[release].platform` and then `path`'s `platform.target` | the platform rules under `## Gate` hold for every entry of `path`'s `deploy.manifests[]` |
| `docs_cover` | `path` string, default `releases/{id}.yaml`; `min_ratio` float, default `1.0`; `command` string, default `""` meaning `config.toml` `[release].api_symbols_command` | the symbol names `command` prints each appear as an H3 heading in one of `path`'s `docs.api[]` files, at a ratio of at least `min_ratio`; `skip` with `reason: no_api_symbols_command` when the command is `""` |
| `yaml_cites` | `doc` string, required; `from` string, required, a path to a sequence; `field` string, required, a key inside each item holding an array of strings; `into` array[string], required, each a path resolving to a set of strings; `min` integer, default `1` | every item of `from` has at least `min` entries in `field`, and every entry is a member of the union of the sets `into` resolves to; a `from` resolving to an empty sequence passes |
| `no_threshold_decrease` | `doc` string, required; `from` string, default `recommendations`; `target_field` string, default `target.path`; `key_field` string, default `target.key`; `value_field` string, default `proposed_value`; `files` array[string], default `["gates.toml", "config.toml"]` | no item of `from` whose `target_field` ends with one of `files` and whose `key_field` names a key in the compiled floor table carries a `value_field` below that floor |

The path grammar of `from`, `to`, `into`, `cover`, `universe`, `metric`, and `field` is dot-separated segments where `[]` after a segment iterates a sequence and a trailing segment after `[]` names a key inside each item: `observations[].id` resolves to the set of `id` values of the `observations` sequence. An empty sequence at a `[]` segment resolves to the empty set. A path one of whose segments is absent from the document is `DFA-E345`.

`verifier_pass` reads a `total` of `0` as a ratio of `1.0`, so a verifier with no unit to count passes. `coverage_min` with `source = "read"` parses the most recently modified match of `config.toml` `coverage_paths` and runs no command; an absent artifact is `DFA-E312` rather than a skip, so a story cannot reach Release with no coverage evidence.

### Compiled-in minimums

The binary carries a table of required check kinds per phase and a table of numeric floors. Thresholds live in two files, so the floors are enforced in two places, both with error `DFA-E303`, exit 1, naming the file, the key, the offending value, and the floor. Values are rejected, not clamped, so an edited file fails loudly rather than behaving differently from what it says.

- `gates.toml` is validated by `gate require`, `gate check`, and `phase set` before anything else runs: the required check kinds are present with `severity = "block"`, and `verifier_pass.min_ratio` is at or above its floor.
- `config.toml` is validated by `config::load`, which every subcommand except `init`, `trust pin`, and `trust verify` calls: each `[[layer]].coverage_min` and `[coverage].overall_min` is at or above its floor. This check applies when `degraded = false`; with the flag set the command checks are skipped, so the numbers govern nothing and are left alone.

| Phase | Required check kinds | Floors |
|---|---|---|
| explore | `file_exists`, `field_in_enum` | — |
| discover | `fields_present`, `ids_resolve` | — |
| constitute | `context_audit` | — |
| plan | `doc_valid`, `story_valid`, `ids_resolve` | — |
| build | `doc_valid`, `tests_pass`, `coverage_min` | in `config.toml`: domain ≥ 90.0, application ≥ 80.0, infrastructure ≥ 70.0, interface ≥ 60.0, `overall_min` ≥ 75.0 |
| verify | `doc_valid`, `verifier_pass` | in `gates.toml`: `verifier_pass.min_ratio` ≥ 1.0 |
| release | `doc_valid`, `file_exists` | — |
| reflect | `file_exists`, `doc_valid`, `yaml_cites`, `no_threshold_decrease` | — |

A required check present with `severity = "warn"` is treated as absent: error `DFA-E303`. `doc_valid` is a required kind for the phases whose gate is the only place their document is checked; the three entry phases rely on the PostToolUse and `pre-commit` runs of `doc validate`, which reach every file under `.devforgeai/` whatever the gate holds. A gate missing from `gates.toml` entirely is error `DFA-E300`, exit 1. A `[[layer]]` table absent from `config.toml` is treated as present at its default threshold, so deleting a layer lowers nothing.

### Degradation rule

`config.toml` carries one boolean, `degraded`. `stack detect` sets it to `true` when the `[[stack]]` array is empty, and to `false` otherwise. This single flag governs every command-running check:

- `degraded = false`: a `tests_pass`, `coverage_min`, or `lint_clean` check whose command string is `""` fails with `DFA-E310`, `DFA-E312`, or `DFA-E314`.
- `degraded = true`: the four command-running kinds — `tests_pass`, `coverage_min`, `lint_clean`, `complexity_clean` — evaluate to `SKIP`, count as passing, and the compiled-in floors for `build` are not applied.
- Every `SKIP` appears in the report as `status: skip` with `reason: degraded`, in the human output as `SKIP (degraded)`, and in `--json` as `{"status":"skip","reason":"degraded"}`.
- A check kind this build stubs is a separate case with the same shape: the check is recorded `status: skip` with `reason: not_implemented`, `DFA-E902` goes to stderr, `gate check` exits 5, and a later `gate require` refuses the resulting report with `DFA-E321`. A stub is visible in three places rather than passing quietly.
- `gate check` prints `degraded: true` in its JSON envelope and the line `Stack undetected; command checks skipped.` on stderr whenever the flag is set.

A stack that is detected but whose individual `lint_command` is `""` (jvm, and ruby without `.rubocop.yml`) makes `lint_clean` evaluate to `SKIP` with `reason: no_lint_command`, independent of `degraded`. The same shape holds for `complexity_clean` with an empty `complexity_command` (`reason: no_complexity_command`) and for `docs_cover` with an empty `api_symbols_command` (`reason: no_api_symbols_command`).

**Every other way a check can end is a failure.** A check whose document, `path`, `field`, pattern, `verifiers` list, `stories` list, or `state.toml` field is absent, null, unparsable, or of the wrong type reports `fail` with a named error code. It never reports `pass` and never reports `skip`. `skip` is produced by exactly three things: an explicit `skip_when` or a `required_when` that evaluates false against a document that loaded; the `degraded` flag over the four command-running kinds; and `--no-run` over `tests_pass`, `lint_clean`, `complexity_clean`, and `docs_cover`. `coverage_min` is absent from the `--no-run` set deliberately: under `--no-run` it takes its `source = "read"` path, which runs no command and still judges the coverage artifact on disk, which is stronger than a skip that counts as passing. A check kind this build stubs is the fourth shape and is not a skip that passes: it is recorded `status: skip` with `reason: not_implemented`, `gate check` exits 5, and a later `gate require` refuses the resulting report with `DFA-E321` even when its `result` reads `PASS`, so a stub cannot carry a phase forward.

### `.devforgeai/state.toml`

```toml
schema = "devforgeai/state/1"                  # string; fixed
updated_at = "2026-09-10T14:02:11Z"            # string; RFC 3339 UTC

[current]
phase = "build"                                # string; phase enum; default "explore"
id = "STORY-014"                               # string; the active id of `phase`; mirrors [active].<phase>; default ""

[active]                                       # one key per phase; "" when the phase has no active work
explore = "IDEA-003"                           # string; IDEA-nnn or ""
discover = "IDEA-003"                          # string; IDEA-nnn or ""
constitute = "IDEA-003"                        # string; IDEA-nnn or ""
plan = "SPRINT-001"                            # string; SPRINT-nnn or ""; the plan report is named after it
build = "STORY-014"                            # string; STORY-nnn or ""
verify = "STORY-014"                           # string; STORY-nnn or ""
release = "v0.3.0"                             # string; vX.Y.Z or ""

[explore]                                      # phase-scoped fields; written by `phase set explore`
idea_id = "IDEA-003"                           # string; default ""
started_at = "2026-09-06T09:00:00Z"            # string; RFC 3339; default ""
timebox_days = 5                               # integer; default: config.toml [explore].timebox_days
remedy_started_at = ""                         # string; RFC 3339 or ""; default ""
remedy_timebox_days = 1                        # integer; default: config.toml [explore].remedy_timebox_days
remedy_flows = []                              # array[string]; FLOW-nnn ids; default []

[constitute]                                   # phase-scoped fields; written by `phase set constitute`
remedy_con = ""                                # string; CON-nnn or ""; default ""

[plan]                                         # phase-scoped fields; written by `phase set plan`
epic = "EPIC-002"                              # string; EPIC-nnn or ""; default ""

[last_gate]
phase = "build"                                # string; phase enum or ""; default ""
id = "STORY-014"                               # string; default ""
result = "PASS"                                # string; enum: PASS | FAIL | SEND_BACK | TRUST_FAIL | NOT_RUN; default "NOT_RUN"
at = "2026-09-10T14:02:11Z"                    # string; RFC 3339 UTC or ""
report = ".devforgeai/reports/STORY-014-build.yaml"   # string; default ""
send_back_to = ""                              # string; phase enum or ""; default ""
failed_checks = []                             # array[string]; check ids that failed; default []

[last_handoff]
rendered_at = "2026-09-10T14:02:12Z"           # string; RFC 3339 UTC or ""
phase = "build"                                # string; phase enum, "design", or ""
id = "STORY-014"                               # string; default ""
lines = []                                     # array[string]; the rendered block, at most 12 entries; default []

[stop_hook]
block_count = 0                                # integer; default 0; incremented when the Stop hook blocks
blocked_phase = ""                             # string; phase enum or ""; a record of what last blocked, read by nothing
blocked_id = ""                                # string; default ""; the same, and not part of the budget key
blocked_session = ""                           # string; the session_id of the payload that last blocked; default ""; this alone keys the budget
scanned_at = ""                                # string; RFC 3339 UTC of the last Stop-time document scan; default ""

[last_cross]                                   # written by a cross-cutting skill's handoff call site
phase = ""                                     # string; "design", "reflect", or ""; default ""
id = ""                                        # string; the subject of the cross-cutting run; default ""
turn = ""                                      # string; RFC 3339 UTC of the turn that recorded it, or ""
```

`[current].phase` and `[current].id` are the pair every skill reads. `[current].id` equals `[active].<current.phase>` in every valid file; `phase set` writes both, and a file where they disagree is `DFA-E106`. `[active]` keeps one id per phase so `gate require` can resolve a predecessor whose id differs from the current one.

`.devforgeai/state.toml` is the one path under `.devforgeai/` that git does not track; `init` adds it to `.gitignore`. Each Build worktree therefore carries its own copy with its own `[active].build`, two concurrent builds produce no merge conflict on it, and `worktree ensure` seeds a new worktree from the main checkout's file.

The phase-scoped tables are `[explore]` and `[constitute]`. The other five phase names are reserved as table names and hold no key in v1. A key in a phase table that this schema does not list is `DFA-E108`, so a typo fails rather than being silently unread.

`[stop_hook]` is the framework's own bookkeeping, not the harness's: the harness owns the loop guard through `stop_hook_active` and its eight-block ceiling. `block_count` resets to `0` on any of: a `gate check` whose result is `PASS`, a successful `phase set`, or a Stop whose `session_id` differs from `blocked_session`. The budget is keyed on the session alone and is three blocks. `blocked_phase` and `blocked_id` record which subject was blocked last and key nothing: the subject is exactly what a continuation can change, so including it in the key would let an advancing story reset the budget for ever.

`[last_cross]` exists because Design and Reflect leave `[current].phase` where they found it, so a Stop at the end of a cross-cutting run has two blocks to render. The cross-cutting skill's own `handoff` call site writes the table; the Stop that renders the pair clears it, so a second Stop in the same session prints one block.

### `~/.devforgeai/trust.toml`

```toml
schema = "devforgeai/trust/1"                  # string; fixed
updated_at = "2026-09-10T14:02:11Z"            # string; RFC 3339 UTC

[[pin]]                                        # one table per pinned binary path; unique by binary_path
binary_path = "C:\\Users\\bryan\\.cargo\\bin\\devforgeai.exe"   # string; absolute, canonicalised
digest = "sha256:7f0c…"                        # string; lowercase hex, `sha256:` prefixed; SHA-256 of the binary bytes
revision = "a1b2c3d4e5f60718…"                 # string; contents of cli/REVISION line 1 (git SHA-1 or "unversioned")
source_digest = "sha256:91ab…"                 # string; contents of cli/REVISION line 2
release_digest = "sha256:7f0c…"                # string; contents of cli/DIGEST
framework_path = "C:\\Projects\\DevForgeAI"    # string; absolute path of the framework repo, or "" when unknown
pinned_at = "2026-09-10T14:02:11Z"             # string; RFC 3339 UTC
pinned_by = "DESKTOP-9\\bryan"                 # string; host and user name at pin time
```

### `.devforgeai/reports/<ID>-<phase>.yaml`

```yaml
schema: devforgeai/report/1        # string; fixed
id: STORY-014                      # string; the gate subject ID
phase: build                       # string; phase enum
status: pass                       # string; enum: pass | fail | send_back | skip
produced_by: devforgeai-cli        # string; fixed
consumes: [STORY-014, SPRINT-001]  # array[string]; IDs the gate read
open_questions: []                 # array[string]; [] in every CLI-written report
started_at: 2026-09-10T14:01:03Z   # string; RFC 3339 UTC
finished_at: 2026-09-10T14:02:11Z  # string; RFC 3339 UTC
cli_version: 1.0.0                 # string
degraded: false                    # bool; copied from config.toml
gate:
  result: PASS                     # string; enum: PASS | FAIL | SEND_BACK
  send_back_to: ""                 # string; phase enum or ""
  checks:                          # array; one entry per gate.check, in gates.toml order
    - id: build-tests              # string
      kind: tests_pass             # string; check kind enum
      status: pass                 # string; enum: pass | fail | skip | warn
      severity: block              # string; enum: block | warn
      reason: ""                   # string; "" on pass; the DFA code and text on fail; "degraded" or "no_lint_command" on skip
      evidence:                    # mapping; kind-specific, keys listed under each subcommand
        command: cargo test --all-features
        exit_code: 0
        duration_ms: 41203
        passed: 214
        failed: 0
coverage:                          # present when a coverage_min check ran; absent otherwise
  format: lcov                     # string; coverage_format enum
  source: target/devforgeai/lcov.info   # string; the file parsed
  overall: 87.4                    # float; percent
  layers:                          # array; one entry per layer in config.toml order
    - name: domain
      covered: 512                 # integer; covered lines
      total: 530                   # integer; instrumented lines
      percent: 96.6                # float
      min: 95.0                    # float
      status: pass                 # string; enum: pass | fail
  unassigned:
    files: 3                       # integer
    percent: 61.0                  # float
verifiers:                         # written by `report ingest`; absent when no verifier ran
  ac_compliance:
    subagent: ac-compliance-verifier   # string
    ingested_at: 2026-09-10T14:00:02Z  # string; RFC 3339 UTC
    passed: 7                      # integer
    total: 7                       # integer
    unit: ACs                      # string
    findings: []                   # array; FIND-nnn entries, schema under `report ingest`
findings: []                       # array; merged findings across verifiers; the handoff Found lines read this
handoff: []                        # array[string]; the block rendered for this gate, at most 12 entries
```

### Crate layout under `cli/`

```
cli/
├── Cargo.toml
├── REVISION                     two lines: git SHA-1 of the framework repo, then sha256: of the cli/ source tree
├── DIGEST                       one line: sha256: of the released binary built with default features
├── src/
│   ├── main.rs                  argument dispatch; maps CliError to a process exit code
│   ├── lib.rs                   re-exports; `pub fn run(args: Cli, io: &mut Io) -> Result<Outcome, CliError>`
│   ├── cli.rs                   clap derive types: `Cli`, `Command`, one struct per subcommand
│   ├── project.rs               root discovery, atomic write, path constants
│   ├── json.rs                  `struct Envelope`, `fn emit(envelope: &Envelope, w: &mut dyn Write)`
│   ├── errors.rs                `enum CliError` with `fn code(&self) -> &'static str` and `fn exit(&self) -> i32`
│   ├── config.rs                `struct Config`, `fn load(root: &Path) -> Result<Config, CliError>`
│   ├── stack.rs                 marker table, detection, default command table
│   ├── gates.rs                 `struct Gates`, `struct Check`, `enum CheckKind`, `fn validate_minimums(&self) -> Result<(), CliError>`
│   ├── state.rs                 `struct State`, `fn load`, `fn store`, `fn advance`
│   ├── doc/
│   │   ├── mod.rs               `struct Document`, `fn validate(path: &Path, ctx: &Ctx) -> Vec<Diagnostic>`
│   │   ├── frontmatter.rs       `fn parse(src: &str) -> Result<Frontmatter, Diagnostic>`
│   │   ├── ids.rs               `struct IdIndex`, `fn build(root: &Path) -> IdIndex`, `fn allocate(&self, prefix: &str) -> Result<String, CliError>`
│   │   └── xref.rs              `fn resolve(doc: &Document, index: &IdIndex) -> Vec<Diagnostic>`
│   ├── exec.rs                  `fn run_command(cmd: &str, cwd: &Path, timeout: Duration, env: &Env) -> Result<CommandOutcome, CliError>`
│   ├── coverage/
│   │   ├── mod.rs               `enum CoverageFormat`, `fn parse(fmt: CoverageFormat, bytes: &[u8]) -> Result<FileCoverage, CliError>`, `fn by_layer(fc: &FileCoverage, cfg: &Config) -> LayerCoverage`
│   │   ├── lcov.rs              `fn parse_lcov(s: &str) -> Result<FileCoverage, CliError>`
│   │   ├── cobertura.rs         `fn parse_cobertura(b: &[u8]) -> Result<FileCoverage, CliError>`
│   │   ├── jacoco.rs            `fn parse_jacoco(b: &[u8]) -> Result<FileCoverage, CliError>`
│   │   ├── gocover.rs           `fn parse_gocover(s: &str) -> Result<FileCoverage, CliError>`
│   │   └── fallback.rs          `fn parse_devforgeai_json(b: &[u8]) -> Result<FileCoverage, CliError>`
│   ├── report.rs                `struct Report`, `fn write`, `fn ingest(subagent: &str, stdout: &str) -> Result<VerifierBlock, CliError>`
│   ├── handoff.rs               `fn render(state: &State, report: Option<&Report>, cfg: &Config) -> Vec<String>`
│   ├── gate.rs                  `fn check(phase: Phase, id: &str, ctx: &Ctx) -> Result<Report, CliError>`, `fn require(phase: Phase, id: &str, ctx: &Ctx) -> Result<(), CliError>`
│   ├── audit.rs                 `fn context_audit(ctx: &Ctx) -> Vec<Diagnostic>`
│   ├── story.rs                 `fn validate(scope: Scope, ctx: &Ctx) -> Vec<Diagnostic>`
│   ├── design.rs                `fn lint(paths: &[PathBuf], tokens: &Tokens) -> Vec<Diagnostic>`, `fn lint_tokens(tokens: &Tokens, specs: &[PathBuf]) -> Vec<Diagnostic>`
│   ├── docops.rs                `fn accept_requirements(ctx: &Ctx, id: &str) -> Result<Accepted, CliError>`, `fn reopen_requirements(ctx: &Ctx, id: &str, ids: &[String], from: Phase) -> Result<Reopened, CliError>`
│   ├── explore.rs               `fn prune(ctx: &Ctx, id: &str) -> Result<Pruned, CliError>`
│   ├── story.rs                 also `fn files_check`, `fn files_list`, `fn files_diff`, `fn list(filter: &Filter) -> Vec<StoryRow>`
│   ├── git.rs                   `fn worktree_ensure|list|remove`, `fn commit(id: &str, msg: &str, paths: &[PathBuf]) -> Result<CommitOutcome, CliError>`
│   ├── antipattern.rs           `fn scan(ctx: &Ctx, set: &Candidates, min: Severity) -> Vec<Match>`
│   ├── aggregate.rs             `fn aggregate(ctx: &Ctx, window: Window) -> Result<Aggregate, CliError>`, `fn sessions(root: &Path, key: &str) -> SessionBlock`
│   ├── analyze.rs               brownfield scan; `fn analyze(root: &Path) -> AnalysisResult`
│   ├── hooks/
│   │   ├── mod.rs               `fn install(root: &Path) -> Result<InstallReport, CliError>`
│   │   ├── settings.rs          `fn merge(existing: &mut Value, block: &Value) -> MergeReport`
│   │   ├── githooks.rs          the three script bodies as `const &str`
│   │   └── run.rs               `fn dispatch(event: Event, stdin: &str, ctx: &Ctx) -> Result<HookOutcome, CliError>`
│   └── trust.rs                 `fn pin(ctx: &Ctx) -> Result<Pin, CliError>`, `fn verify(ctx: &Ctx) -> Result<(), CliError>`, `fn digest_file(p: &Path) -> Result<String, CliError>`
└── tests/
    ├── fixtures/                see `## Evals`
    └── *.rs                     one integration test file per subcommand
```

Dependencies, pinned in `Cargo.toml`: `clap = { version = "4.5", features = ["derive", "env"] }` for argument parsing; `serde = { version = "1", features = ["derive"] }`; `toml = "0.8"` for reads and `toml_edit = "0.22"` for comment-preserving writes; `serde_json = "1"` for `--json`, `.claude/settings.json`, `brand/tokens.json`, and hook stdin; `serde_yaml_ng = "0.10"` for the YAML documents and reports; `sha2 = "0.10"`; `globset = "0.4"`; `walkdir = "2"`; `quick-xml = "0.36"` for cobertura and jacoco; `time = { version = "0.3", features = ["formatting", "parsing"] }`; `thiserror = "2"`. Dev-dependencies: `assert_cmd = "2"`, `predicates = "3"`, `tempfile = "3"`, `insta = "1"`. Features: `default = []`, `test-home = []`.

## Workflow

Steps 1 to 4 run once per project. Steps 5 to 12 repeat per phase.

1. **User** runs `devforgeai trust pin` in a terminal with no Claude session active. Input: the installed binary and `cli/REVISION`, `cli/DIGEST` from the framework repo. Output: a `[[pin]]` table in `~/.devforgeai/trust.toml`. Failure path: `CLAUDECODE` or `CLAUDE_CODE_ENTRYPOINT` non-empty gives `DFA-E500`, exit 4, nothing written; a missing `cli/DIGEST` gives `DFA-E510`, exit 4.
2. **User** runs `devforgeai init [--analyze]` in the target project root. Input: the target directory and the framework's `skills/` and `agents/`. Output: `.devforgeai/` with `config.toml`, `gates.toml`, `state.toml`; `.claude/skills` and `.claude/agents`, each minus every `evals/` subtree; a merged `.claude/settings.json` carrying the hook block and the two permission rules; the `CLAUDE.md` section between markers; the three git hooks; with `--analyze`, six draft context files. Failure path: an existing `.devforgeai/config.toml` without `--force` gives `DFA-E110`, exit 1, nothing written.
3. **CLI** runs `stack detect` as the last step of `init`. Input: marker files in the project. Output: `[[stack]]` entries and `degraded` in `config.toml`. Failure path: zero markers sets `degraded = true`, writes an empty `[[stack]]` array, exits 0, and prints `Stack undetected; command checks skipped.` on stderr.
4. **CLI** runs `hook install` as part of `init`. Input: the compiled-in settings block and the git directory. Output: merged hooks and three executable scripts. Failure path: no `.git/` directory writes the Claude hooks, skips the git hooks, prints `DFA-E130` on stderr, exits 0.
5. **User** invokes the phase's slash command. **CLI**, in the command's `!` preamble, runs `gate require <phase> <id>`. Input: `gates.toml`, `state.toml`, the predecessor report. Output: exit 0 and nothing on stdout. Failure path: exit 1 with the missing predecessor gate named; the command body does not run.
6. **CLI**, in the same preamble, runs `doc load <name> <id>` for each upstream document the phase reads. Input: the §5 path for `<name>`. Output: the file contents on stdout. Failure path: a missing file is `DFA-E200`, exit 1, and the preamble stops.
7. **Model**, inside the skill, writes the phase's document with Write or Edit. **CLI** runs `doc validate --producer-check <path>` on PreToolUse. Input: the pending `tool_input.file_path` and `state.toml`. Output: exit 0. Failure path: a producer mismatch is exit 1 from the CLI, which the hook maps to hook exit 2, blocking the write and returning the message to the model.
8. **CLI** runs `doc validate <path>` on PostToolUse. Input: the written file. Output: diagnostics on stderr. Failure path: diagnostics annotate; the write stands.
9. **Subagent** runs and prints JSON. **CLI** runs `report ingest <subagent> -` on SubagentStop when the subagent name appears in `config.toml` `[[verifier]]`. Input: the subagent's stdout. Output: a block under `verifiers.<key>` in `reports/<active-id>-<phase>.yaml`. Failure path: unparsable JSON is `DFA-E410` on stderr, exit 0 from the hook, and the block is written with `status: unparsed`.
10. **CLI** runs `gate check --phase <current>` on Stop. Input: `gates.toml`, `config.toml`, the documents, the project's test and coverage commands, the ingested verifier blocks. Output: `reports/<ID>-<phase>.yaml` and `state.toml` `[last_gate]`. Failure path: a FAIL inside the three-block session budget exits 2 with `decision: "block"` and the handoff in `systemMessage`, incrementing `[stop_hook].block_count`; a FAIL with the budget spent exits 0 and lets the FAIL handoff stand.
11. **CLI** runs `handoff` on Stop. Input: `state.toml` and the report named by `[last_gate].report`. Output: at most twelve lines, carried in the Stop hook's `systemMessage` and stored in `[last_handoff].lines`; when `[last_cross]` is populated the cross-cutting block is rendered first and the two are joined. Failure path: no report gives a handoff with `Gate      NOT RUN` and exit 0.
12. **User** runs the `Next` line verbatim. **CLI** runs `phase set <phase> --id <id>` from the workflow step of the skill that needs it. Input: `gates.toml`, `state.toml`. Output: `[current]`, `[active]`, and `stop_hook.block_count = 0`. Failure path: the predecessor gate did not pass, giving `DFA-E320`, exit 1, and `state.toml` unchanged.

Git enforcement runs outside this loop. `pre-commit` calls `doc validate` on staged `.devforgeai/` files and `context audit`. `commit-msg` requires a `STORY-nnn` or `ADR-nnn` token. `pre-push` calls `gate check --phase build --id <id>` for every story in `sprint.yaml` whose status is `building`. Each script begins with `devforgeai trust verify` and exits 4 on mismatch.

## Subagents

The CLI invokes no subagents. It has no Agent tool, no model access, and no prompt. Its relationship to subagents is one-directional: `report ingest` parses the stdout of a subagent that the SubagentStop hook hands it, and `gate check` reads the resulting block for a `verifier_pass` check.

The registry of which subagents are ingested lives in `config.toml` as `[[verifier]]` tables, because conventions §7 names the condition ("when the subagent is a registered verifier") without naming the register. Each table maps a subagent `name` to a `report_field`, a `phase`, a `unit` word for the handoff `Verified` line, and a `required` flag. `init` writes the array with the entries the shipped skill specs declare `registered_verifier: yes`; a skill spec that adds a verifier adds a `[[verifier]]` table. A SubagentStop for a name absent from the registry is a no-op with exit 0.

The subagent stdout contract the CLI parses is exactly one JSON object, never two and never an object beside prose. The envelope keys sit at the top; every field the agent defines for itself sits under `payload`:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "ac-compliance-verifier",
  "id": "STORY-014",
  "passed": 7,
  "total": 7,
  "unit": "ACs",
  "findings": [
    { "id": "FIND-001", "severity": "block", "summary": "AC-003 has no test", "evidence": "tests/: no case names AC-003" }
  ],
  "payload": {
    "checks": [
      { "ac": "AC-003", "verdict": "unmet", "evidence": "tests/: no case names AC-003" }
    ]
  }
}
```

`severity` is the closed enum `block | warn | info`. `findings` defaults to `[]`. `payload` defaults to `{}`. `id` absent falls back to `state.toml` `[active].<phase>`. Any other shape is `DFA-E410`.

The ingested block keeps the same split: `subagent`, `ingested_at`, `passed`, `total`, `unit`, `findings`, and the agent's own object at `verifiers.<report_field>.payload`. A `report_metric` check reading an agent field therefore carries the `payload` segment — `verifiers.architecture_reviewer.payload.blocking_findings` — while `verifier_pass` keeps reading `passed` and `total` at the top of the block. The nesting is what lets one envelope serve forty-six agents with forty-six different field sets and one parser.

A finding at `severity: warn` never lowers `passed`: the counts measure the units the agent was asked to judge, and a warning is a remark about one of them. A review agent reports every finding it has, including the uncertain and the low-severity, with no cap and no self-filtering; `gate check` and the Verify phase do the filtering, and an agent told to report only what matters under-reports.

## Command

The CLI ships no slash command. It is a binary, invoked by hooks, by git, and by the `!` preamble lines at the top of the nine `SKILL.md` files, each of which is specified in its own skill spec. Every skill that calls it mid-workflow reaches it through the `Bash(devforgeai:*)` and `PowerShell(devforgeai:*)` grants in that skill's `allowed-tools`, and through the two standing `permissions.allow` rules `hook install` merges.

The preamble idiom every command uses is §4b's inline-bash syntax, a `!` followed by a backtick-wrapped command:

```
!`devforgeai gate require plan $ARGUMENTS[0]`
!`devforgeai doc load requirements $ARGUMENTS[0]`
```

`$ARGUMENTS[0]` is the first argument and `$ARGUMENTS` the whole string. The first argument is never written `$1`: measured, a `!` command containing `$1` is refused before it runs, with `Shell command permission check failed ... Contains simple_expansion`, which aborts an invocation that should have proceeded. There is no separate command file: §4b merges the commands into the skills, so the frontmatter is the `SKILL.md`'s and carries `name`, `description`, `argument-hint`, `allowed-tools`, and `disable-model-invocation`, with `allowed-tools` listing both `Bash(devforgeai:*)` and `PowerShell(devforgeai:*)` so the preamble can run under either shell. A non-zero exit from a `!` line stops the command body, and stderr reaches the model. The nine command names are fixed by §4b: `/explore`, `/discover`, `/constitute`, `/plan`, `/build`, `/verify`, `/release`, `/design`, `/reflect`. The handoff `Next` and `Then` lines print these names and no other.


## CLI calls

### Global grammar

```
devforgeai [--json] [--project <path>] [--quiet] [--version] [--help] <subcommand> [<args>]
```

| Flag | Type | Default | Effect |
|---|---|---|---|
| `--json` | bool | false | one JSON envelope on stdout, no human text on stdout |
| `--project <path>` | path | nearest ancestor of the working directory containing `.devforgeai/` | project root override; `init` defaults to the working directory instead |
| `--quiet` | bool | false | suppresses human stdout; stderr is unaffected |
| `--version` | bool | false | prints `devforgeai <semver> (<REVISION line 1, first 12 chars>)`, exit 0 |
| `--help` | bool | false | prints usage for the binary or the subcommand, exit 0 |

Subcommand names, arguments, and exit codes come from conventions §4 and are repeated below with their full grammar. `--json` and `--project` are accepted by every subcommand. Human output goes to stdout, diagnostics to stderr, per §4.

### JSON envelope

Every `--json` invocation prints exactly one object, on failure as well as on success:

```json
{
  "schema": "devforgeai/cli-json/1",
  "command": "gate check",
  "ok": false,
  "exit": 1,
  "degraded": false,
  "project": "C:\\Projects\\acme",
  "at": "2026-09-10T14:02:11Z",
  "data": {},
  "errors": [ { "code": "DFA-E311", "message": "...", "path": ".devforgeai/gates.toml", "line": 22 } ],
  "warnings": [ { "code": "DFA-W201", "message": "...", "path": "...", "line": 0 } ]
}
```

`command` is the subcommand name with its space. `ok` is `exit == 0`. `path` is `""` and `line` is `0` when the diagnostic has no location. `data` is the per-subcommand object specified below.

### Error and warning table

Every stderr line has the form `devforgeai: <code> <subcommand>: <message>`, followed where a location exists by a second line `  at <path>:<line>`. Angle brackets mark a substitution.

| Code | Subcommand | Condition | Message text | Exit |
|---|---|---|---|---|
| DFA-E010 | any | unknown option or subcommand | `unknown option '<arg>'; run 'devforgeai --help'` | 3 |
| DFA-E011 | any | required argument absent | `'<subcommand>' requires <arg>` | 3 |
| DFA-E012 | any | value outside an enum | `'<value>' is not a <enum-name>; expected one of <a, b, c>` | 3 |
| DFA-E013 | any | malformed ID | `'<value>' is not an ID; expected PREFIX-nnn with three digits, or it contradicts the id the document on disk carries` | 3 |
| DFA-E020 | hook run | hook JSON lacks a key the event needs | `hook input has no key '<key>' for event <event>` | 3 |
| DFA-E021 | hook run | stdin is not JSON | `stdin is not JSON: <parser message>` | 3 |
| DFA-E030 | any | project root not found | `no .devforgeai/ directory in <cwd> or any ancestor; run 'devforgeai init'` | 3 |
| DFA-E031 | any | `--project` path absent | `--project path '<path>' does not exist` | 3 |
| DFA-E101 | any | `config.toml` absent | `.devforgeai/config.toml not found; run 'devforgeai stack detect'` | 1 |
| DFA-E102 | any | `gates.toml` absent | `.devforgeai/gates.toml not found; run 'devforgeai init'` | 1 |
| DFA-E103 | any | `state.toml` absent | `.devforgeai/state.toml not found; run 'devforgeai init'` | 1 |
| DFA-E104 | any | `config.toml` unparsable | `config.toml is not valid TOML: <parser message>` | 1 |
| DFA-E105 | any | `gates.toml` unparsable | `gates.toml is not valid TOML: <parser message>` | 1 |
| DFA-E106 | any | `state.toml` unparsable | `state.toml is not valid TOML: <parser message>` | 1 |
| DFA-E108 | any | a key in a `state.toml` phase table is not in the schema | `state.toml [<phase>] has key '<key>', which the state schema does not define` | 1 |
| DFA-E107 | any | `schema` key unknown | `<file>: schema '<value>' is unsupported; this binary reads <expected>` | 1 |
| DFA-E110 | init | `.devforgeai/config.toml` exists, no `--force` | `project is already initialised; pass --force to overwrite .devforgeai/` | 1 |
| DFA-E111 | init | framework source not resolvable | `framework source not found; pass --from <path> to the DevForgeAI repo root` | 1 |
| DFA-E112 | init | copy failed | `copying <src> to <dst> failed: <io message>` | 1 |
| DFA-E113 | init --analyze | analysis time cap reached | `analysis stopped after 120s; <n> files unread` | 0 |
| DFA-E120 | design lint | `brand/tokens.json` absent | `.devforgeai/brand/tokens.json not found; the Design skill produces it` | 1 |
| DFA-E121 | design lint | tokens JSON unparsable | `brand/tokens.json is not valid JSON: <parser message>` | 1 |
| DFA-E130 | hook install | no `.git/` directory | `no .git directory; git hooks not installed` | 0 |
| DFA-E131 | hook install | `.claude/settings.json` unparsable | `.claude/settings.json is not valid JSON: <parser message>; nothing merged` | 1 |
| DFA-E132 | hook install | a git hook exists that this binary did not write | `<hook> exists and is not a devforgeai hook; pass --force to replace it` | 1 |
| DFA-E200 | doc validate, doc load | file absent | `<path> not found` | 1 |
| DFA-E201 | doc validate | no frontmatter fence | `<path> has no frontmatter; line 1 opens with three dashes` | 1 |
| DFA-E202 | doc validate | frontmatter is not a mapping | `<path> frontmatter is not a YAML mapping` | 1 |
| DFA-E203 | doc validate | key absent | `<path> frontmatter has no key '<key>'` | 1 |
| DFA-E204 | doc validate | extra top-level key | `<path> frontmatter has key '<key>', which conventions section 5 does not define` | 1 |
| DFA-E205 | doc validate | keys out of order | `<path> frontmatter key '<key>' is at position <n>; conventions section 5 places it at <m>` | 1 |
| DFA-E206 | doc validate | wrong type | `<path> frontmatter key '<key>' is <actual>; expected <expected>` | 1 |
| DFA-E207 | doc validate | schema mismatch | `<path> schema is '<value>'; this document type is '<expected>'` | 1 |
| DFA-E208 | doc validate | status outside the enum | `<path> status is '<value>'; <doc-type> allows <a, b, c>` | 1 |
| DFA-E209 | doc validate | ID malformed or duplicated | `<path> id '<value>' is malformed` / `<path> id '<value>' duplicates <other-path>` | 1 |
| DFA-E210 | doc validate | reference unresolved | `<path> references <ID>, which no document defines` | 1 |
| DFA-E211 | doc validate | producer wrong for the doc type | `<path> produced_by is '<value>'; <doc-type> is produced by '<skill>'` | 1 |
| DFA-E212 | doc validate --producer-check | current phase does not write this doc type | `phase '<current>' does not write <doc-type>; '<skill>' does` | 1 |
| DFA-E213 | doc validate | heading absent or out of order | `<path> heading '<text>' is absent` / `<path> heading '<text>' is at position <n>, expected <m>` | 1 |
| DFA-E214 | doc validate --allocate | prefix outside §5 | `'<prefix>' is not an ID prefix; conventions section 5 defines <list>` | 3 |
| DFA-E215 | doc validate --allocate | prefix exhausted | `prefix <prefix> has no free ID below 999` | 1 |
| DFA-E216 | doc validate | YAML document lacks a §5 top-level key | `<path> has no top-level key '<key>'` | 1 |
| DFA-E217 | doc validate | a `release` document id is not a semver triple | `<path> id '<value>' is not vX.Y.Z` | 1 |
| DFA-E218 | doc validate | a `release` version is not greater than the highest existing one | `<path> version <value> is not greater than <previous>` | 1 |
| DFA-E220 | context audit | context file absent | `.devforgeai/context/<name>.md not found; conventions section 3 lists six context files` | 1 |
| DFA-E221 | context audit | context statement contradicts an ADR | `<file> line <n> states '<claim>'; <ADR-nnn> line <m> states '<counter>'` | 1 |
| DFA-E222 | context audit | constraint ID reused | `CON-<nnn> is defined in <file-a> and <file-b>` | 1 |
| DFA-E223 | context audit | CA-2, a context file is not accepted | `<path> is status '<value>' with <n> open questions; context audit needs accepted and []` | 1 |
| DFA-E224 | context audit | CA-4, an active CON is introduced by no ADR and traces to no REQ | `CON-<nnn> is introduced by no ADR and its source '<value>' resolves to no requirement` | 1 |
| DFA-E225 | context audit | CA-6, an anti-pattern row is incomplete | `AP-<nnn> has <no detector \| detector_kind '<value>' \| source '<value>' which is not an active CON>` | 1 |
| DFA-E226 | context audit | CA-7, an ADR reference does not resolve | `<path> <id duplicates <other> \| consumes <ID> which requirements.yaml does not define \| introduces CON-<nnn> which the constraint index omits>` | 1 |
| DFA-E227 | context audit | CA-8, a supersession is incomplete | `<path> supersedes <ADR-nnn>, which is status '<value>'` / `<path> retires CON-<nnn>, which is status '<value>'` | 1 |
| DFA-E230 | story validate | AC has no testable predicate | `<STORY-nnn> <AC-nnn> has no testable predicate; the grammar is Given/When/Then or a single assertion line` | 1 |
| DFA-E231 | story validate | REQ reference unresolved | `<STORY-nnn> consumes <REQ-nnn>, which requirements.yaml does not define` | 1 |
| DFA-E232 | story validate | dependency cycle | `dependency cycle: <A> -> <B> -> <A>` | 1 |
| DFA-E233 | story validate | sprint names an unknown story | `sprint.yaml lists <STORY-nnn>, which has no file` | 1 |
| DFA-E234 | story validate | story has no ACs | `<STORY-nnn> defines no acceptance criteria` | 1 |
| DFA-E235 | story validate | an epic requirement is covered by no story | `<EPIC-nnn> requirement <REQ-nnn> appears in the Requirements table of no story` | 1 |
| DFA-E236 | story validate | an AC appears in no Covered by cell | `<STORY-nnn> <AC-nnn> appears in no Covered by cell of ## Requirements` | 1 |
| DFA-E237 | story validate | declared file sets overlap | `<STORY-nnn> and <STORY-nnn> both declare <path>` | 1 |
| DFA-E238 | story validate | a cited UI spec is absent | `<STORY-nnn> cites <UI-nnn>, which .devforgeai/ui-specs/ does not hold` | 1 |
| DFA-E239 | story files, commit | a path is outside the declared set | `<path> is outside the declared file set of <STORY-nnn>` | 1 |
| DFA-E245 | story validate | an AC covers two requirements | `<STORY-nnn> <AC-nnn> appears in the Covered by cell of <REQ-nnn> and <REQ-nnn>` | 1 |
| DFA-E240 | design lint | literal colour | `<path>:<line> uses the literal colour '<value>'; brand/tokens.json defines <nearest-token>` | 1 |
| DFA-E241 | design lint | literal type value | `<path>:<line> uses the literal <property> '<value>'; brand/tokens.json defines <nearest-token>` | 1 |
| DFA-E242 | design lint | unknown token reference | `<path>:<line> references token '<name>', which brand/tokens.json does not define` | 1 |
| DFA-E243 | design lint --tokens | token group or leaf shape | `brand/tokens.json group '<group>' leaf '<leaf>' is <found>, expected <expected>` | 1 |
| DFA-E244 | design lint --tokens | undefined token in a UI spec | `<path>:<line> references token '<name>', which brand/tokens.json does not define` | 1 |
| DFA-E250 | doc load | unknown document name | `'<name>' is not a document; the names are <list>` | 3 |
| DFA-E270 | antipattern scan | an anti-pattern detector matched | `<AP-nnn> matched <path>:<line>: <text>` | 1 |
| DFA-E271 | worktree, commit | the project root is not a git work tree | `<path> is not a git work tree; run git init` | 1 |
| DFA-E272 | worktree ensure | two stories declare a shared path | `<STORY-nnn> and <STORY-nnn> both declare <path>; one worktree at a time` | 1 |
| DFA-E273 | worktree remove | the worktree has uncommitted or unmerged work | `<path> holds <n> uncommitted changes and <m> commits absent from <base_ref>; pass --force` | 1 |
| DFA-E260 | doc accept | the document has no epics or no requirements | `requirements.yaml has no <epics \| requirements>; nothing accepted` | 1 |
| DFA-E261 | doc reopen | a cited id is neither REQ nor UI | `'<id>' is neither a REQ nor a UI id` | 3 |
| DFA-E262 | explore prune | a path could not be removed | `removing <path> failed: <io message>` | 1 |
| DFA-E300 | gate check, gate require, phase set | no gate for the phase | `gates.toml has no gate for phase '<phase>'` | 1 |
| DFA-E301 | any reading gates | unknown check kind | `gates.toml gate '<phase>' check '<id>' has kind '<value>'; the kinds are <list>` | 1 |
| DFA-E302 | any reading gates | unknown key in a check | `gates.toml gate '<phase>' check '<id>' has key '<key>', which kind '<kind>' does not define` | 1 |
| DFA-E303 | any reading gates or config | compiled minimum violated | `gates.toml gate '<phase>' omits required check kind '<kind>'` / `gates.toml gate '<phase>' sets <kind>.<key> to <value>, below the compiled minimum <floor>` / `config.toml sets <key> to <value>, below the compiled minimum <floor>` | 1 |
| DFA-E304 | any reading gates | `cli_min_version` above the binary | `gates.toml requires devforgeai <value> or newer; this binary is <version>` | 1 |
| DFA-E305 | any reading gates | duplicate phase or check id | `gates.toml defines phase '<phase>' twice` / `gates.toml defines check id '<id>' twice in gate '<phase>'` | 1 |
| DFA-E306 | any reading gates | `send_back_to` absent or invalid | `gates.toml gate '<phase>' has on_fail = "send_back" and send_back_to = '<value>'` | 1 |
| DFA-E310 | gate check | `tests_pass` with an empty command, `degraded = false` | `stack '<id>' has no test_command; set one in config.toml` | 1 |
| DFA-E311 | gate check | test command exited non-zero | `test command '<cmd>' exited <code> for stack '<id>'` | 1 |
| DFA-E312 | gate check | coverage report absent or unparsable | `coverage report not found at <globs>` / `coverage report <path> is not <format>: <parser message>` | 1 |
| DFA-E313 | gate check | layer below threshold | `coverage for layer '<name>' is <actual>%, below the <min>% in config.toml` | 1 |
| DFA-E314 | gate check | `lint_clean` with an empty command, `degraded = false` | `stack '<id>' has no lint_command; set one in config.toml` | 1 |
| DFA-E315 | gate check | lint command exited non-zero | `lint command '<cmd>' exited <code> for stack '<id>'` | 1 |
| DFA-E316 | gate check | no ingested block for a named verifier | `report <path> has no verifiers block for '<name>'` | 1 |
| DFA-E317 | gate check | verifier ratio below minimum | `verifier '<name>' passed <p>/<t>, below min_ratio <r>` | 1 |
| DFA-E318 | gate check | command exceeded `timeout_secs` | `command '<cmd>' exceeded <n>s and was terminated` | 1 |
| DFA-E319 | gate check | command could not be spawned | `command '<cmd>' could not start: <io message>` | 1 |
| DFA-E320 | phase set | predecessor gate not passed | `phase '<phase>' needs gate '<pred>' PASS for <id>; the last result is <result>` | 1 |
| DFA-E321 | gate require | predecessor report absent or not PASS | `<phase> needs <pred> gate PASS for <id>; no report at <path>` / `<phase> needs <pred> gate PASS for <id>; the last result is <result> at <time>` | 1 |
| DFA-E322 | gate check | `field_in_enum` failed | `<path> field '<field>' is '<value>'; gate '<phase>' check '<id>' allows <list>` | 1 |
| DFA-E330 | gate check | `field_is_date` failed | `<path> field '<field>' is '<value>', expected <format>` / `<path> field '<field>' is not later than '<after_field>'` | 1 |
| DFA-E331 | gate check | `length_between` failed | `<path> <field> holds <value> entries, expected <min> to <max>` | 1 |
| DFA-E332 | gate check | `fields_present` failed | `<path> <collection> entry '<id>' has no '<field>'` / `<path> '<path-key>' is null` | 1 |
| DFA-E333 | gate check | `set_cover` failed | `<n> ids at <universe> are absent from <cover>: <first five>` / `<id> appears twice in <cover>` | 1 |
| DFA-E334 | gate check | `row_count_between` failed | `<path> section '<section>' holds <value> rows, expected <min> to <max>` | 1 |
| DFA-E335 | gate check | `column_matches` failed | `<path> section '<section>' column '<column>' value '<value>' is <malformed \| duplicated>` | 1 |
| DFA-E336 | gate check | `column_contains_all` failed | `<value> is in <state_field> and absent from <path> section '<section>' column '<column>'` | 1 |
| DFA-E337 | gate check | `elapsed_days_at_most` failed | `<value> days elapsed of <limit> allowed since <started_field>` | 1 |
| DFA-E338 | gate check | `report_metric` failed | `<metric> is <value>, expected <op> <limit>` / `report <path> has no metric '<metric>'` | 1 |
| DFA-E339 | gate check | a condition key names a field that does not exist | `gate '<phase>' check '<id>' condition reads '<field>', which <path> does not define` | 1 |
| DFA-E323 | gate check | `file_exists` failed | `gate '<phase>' check '<id>' found <n> of <min_count> paths; missing <list>` | 1 |
| DFA-E324 | gate check | `no_open_questions` failed | `<path> has <n> open questions` | 1 |
| DFA-E325 | gate check | `ids_resolve` failed | `<n> references do not resolve: <first five>` | 1 |
| DFA-E326 | gate check | `doc_valid` failed | `<path> failed doc validate: <first code and message>` | 1 |
| DFA-E327 | gate check | `context_audit` failed | `context audit found <n> problems; first: <message>` | 1 |
| DFA-E328 | gate check | `story_valid` failed | `story validate found <n> problems; first: <message>` | 1 |
| DFA-E329 | gate check | `design_tokens` failed | `design lint found <n> problems; first: <message>` | 1 |
| DFA-E341 | gate check | `release_stories` failed | `<STORY-nnn> is status '<value>' with verify result '<result>'; the release needs <require_status> and <require_result>` | 1 |
| DFA-E342 | gate check | `deploy_manifest` failed | `<path> <is absent \| does not parse \| has no <key>> for platform '<platform>'` | 1 |
| DFA-E343 | gate check | a manifest holds a literal secret | `<path>:<line> holds a literal secret` | 1 |
| DFA-E344 | gate check | `docs_cover` failed | `<n> of <m> public symbols have no H3 heading in docs.api[]: <first five>` | 1 |
| DFA-E345 | gate check | a document path in a check does not resolve | `gates.toml check '<id>': path '<path>' does not resolve in <doc>` | 1 |
| DFA-E346 | gate check | `yaml_cites` found too few citations | `<doc>: <from>[<n>] id '<id>' cites <count> of <min> required` | 1 |
| DFA-E347 | gate check | `yaml_cites` found an unknown citation | `<doc>: <from>[<n>] id '<id>' cites '<value>', which <into> does not define` | 1 |
| DFA-E348 | gate check | `no_threshold_decrease` found a lowered floor | `<doc>: <from>[<n>] id '<id>' proposes <key> = <value>, below the compiled floor <floor>` | 1 |
| DFA-E349 | gate check | `complexity_clean` failed or its stack selection names nothing | `complexity command '<command>' exited <code> for stack '<stack>'` | 1 |
| DFA-E350 | gate check | a failing check produced no diagnostic code | `gate '<phase>' check '<id>' failed and named no code` | 1 |
| DFA-E400 | report show, handoff | report absent | `.devforgeai/reports/<id>-<phase>.yaml not found` | 1 |
| DFA-E401 | report show, gate check, handoff, report aggregate | report unparsable | `<path> is not valid YAML: <parser message>` | 1 |
| DFA-E410 | report ingest | stdout is not the verifier JSON | `subagent '<name>' output is not devforgeai/verifier/1: <parser message>` | 0 |
| DFA-E412 | report ingest | no active ID for the verifier's phase | `state.toml has no active id for phase '<phase>'; nothing ingested` | 0 |
| DFA-E413 | report note | the note file fails the `devforgeai/build-note/1` schema | `<path> key '<key>' is <found>, expected <expected>; nothing written` | 1 |
| DFA-E421 | report aggregate | the session root resolves outside the user's home | `session root '<path>' is outside <home>; sessions unreadable` | 1 |
| DFA-E430 | report aggregate | neither or both of `<ID>` and `--since` | `pass one id (IDEA-nnn, EPIC-nnn, STORY-nnn, vX.Y.Z) or --since <YYYY-MM-DD>` | 3 |
| DFA-E500 | trust pin | a Claude session is active | `trust pin does not run inside a Claude session; <var> is set` | 4 |
| DFA-E501 | trust verify | `trust.toml` absent | `<home>/trust.toml not found; a human runs 'devforgeai trust pin' outside Claude` | 4 |
| DFA-E502 | trust verify | no pin for this binary path | `no pin for <path> in trust.toml` | 4 |
| DFA-E503 | trust verify | digest mismatch | `binary digest <actual> does not match the pin <expected>` | 4 |
| DFA-E504 | trust verify | source digest drift during a session | `cli/ source digest <actual> differs from the pinned REVISION <expected> while a Claude session is active` | 4 |
| DFA-E510 | trust pin | `cli/DIGEST` or `cli/REVISION` absent or malformed | `<path> is missing` / `<path> is malformed; release builds write it` | 4 |
| DFA-E511 | trust pin, trust verify | current executable path unavailable | `the running executable path is unavailable: <io message>` | 4 |
| DFA-E512 | trust pin, trust verify | `trust.toml` unparsable | `trust.toml is not valid TOML: <parser message>` | 4 |
| DFA-E900 | any | filesystem error | `<operation> on <path> failed: <io message>` | 5 |
| DFA-E902 | any | a subcommand or check kind is specified and stubbed in this build | `check kind '<kind>' is specified and not implemented in this build` | 5 |
| DFA-E901 | any | atomic rename failed | `replacing <path> failed: <io message>; the temporary file is <tmp>` | 5 |
| DFA-W130 | hook install | a Claude hook entry is already present | `<event> hook already present; left unchanged` | 0 |
| DFA-W201 | doc validate | `consumes` lists an ID the body does not cite | `<path> consumes <ID>, which the body does not cite` | 0 |
| DFA-W210 | phase set, gate require | the document whose status the phase advances is absent | `status not written; <path> not found` | 0 |
| DFA-W202 | doc validate | body cites an ID absent from `consumes` | `<path> cites <ID>, which consumes does not list` | 0 |
| DFA-E340 | gate check | `no_cycle` found a cycle | `deferral cycle: <A> -> <B> -> <A>` | 1 |
| DFA-W243 | commit | the stage set is empty | `no declared path changed; nothing committed` | 0 |
| DFA-W310 | gate check | a `warn` severity check failed | `check '<id>' failed with severity warn: <message>` | 0 |
| DFA-W320 | gate check | covered files match no layer glob, `unassigned_policy = "report"` | `<n> covered files match no layer glob` | 0 |
| DFA-W411 | report ingest | subagent absent from the registry | `subagent '<name>' is not in config.toml [[verifier]]; nothing ingested` | 0 |
| DFA-W420 | report aggregate | a report under reports/ does not parse | `<path> does not parse; it is excluded from the window` | 0 |
| DFA-W500 | trust verify | the pin predates the binary file time | `the pin predates the binary file time; digests still match` | 0 |

### `init`

```
devforgeai init [--analyze] [--force] [--from <path>] [--no-hooks] [--json] [--project <path>]
```

| Argument | Type | Default | Effect |
|---|---|---|---|
| `--analyze` | bool | false | runs the brownfield analysis after `stack detect` |
| `--force` | bool | false | overwrites an existing `.devforgeai/` and existing git hooks |
| `--from <path>` | path | `framework_path` of the matching `[[pin]]` in `trust.toml`; absent, error `DFA-E111` | the DevForgeAI repo root supplying `skills/` and `agents/` |
| `--no-hooks` | bool | false | skips both the settings merge and the git hooks |
| `--project <path>` | path | the working directory | the target project root |

Order of operations: create `.devforgeai/` and the subdirectories `explore`, `context`, `adr`, `stories`, `ui-specs`, `brand`, `reports`, `releases`; copy `<from>/skills` to `.claude/skills` and `<from>/agents` to `.claude/agents`, file by file, skipping every directory named `evals` below the top of each tree; write `.devforgeai/gates.toml` with the default content in `## Gate`; write `.devforgeai/state.toml` from `templates/state.initial.toml`, which sets `[current].phase = "explore"`, `[current].id = ""`, every `[active]` key `""`, `[last_gate].result = "NOT_RUN"`, and `[stop_hook].block_count = 0`; run `stack detect`; append `.explore-prototype/` and `.devforgeai/state.toml` to the target project's `.gitignore` when those lines are absent, creating the file when it does not exist; write the CLAUDE.md section; run `hook install` unless `--no-hooks`; run the analysis when `--analyze`.

**Where each skill lands.** Claude Code derives a project skill's slash command from its *directory* name; the frontmatter `name` is a display label. The framework's source tree keeps the long directory names — `skills/exploring-ideas/`, `skills/implementing-stories/` — because `produced_by: implementing-stories` in every template resolves against them and the producer check reads that value. `init` therefore installs each skill at `.claude/skills/<frontmatter name>/`: `skills/implementing-stories/` becomes `.claude/skills/build/`, which is what makes `/build` the command in the target project. The eval runner's workspace build does the same, so a case prompt of `/build STORY-014` resolves there too.

There is no `commands/` copy: a skill and a command file of the same name both produce `/name` and the skill wins, so each entry point carries its own frontmatter and preamble and there is no shadow file to edit by mistake. The `evals/` exclusion keeps the framework's graders, fixtures and expected outputs out of every installed project — a target runs no eval, and a model working in that project could otherwise read the expected outputs.

**The CLAUDE.md section.** `init` writes a block between `<!-- devforgeai:begin -->` and `<!-- devforgeai:end -->` into `CLAUDE.md` at the project root: 31 lines including the two markers, 29 between them. When the begin marker is present the span from it through the end marker inclusive is replaced; when it is absent the block is appended after a blank line; when the file does not exist it is created holding an `# <project directory name>` heading, a blank line, and the block. The write goes through the same atomic path as every other project file, and `--force` is not consulted, because replacing between markers touches no text the user wrote outside them. The block names the nine commands and what each writes, says that the handoff comes from the Stop hook and that its `Next` line is copied as printed, says where phase documents live, says that gates are the binary and the hooks rather than instructions in that file, and says that a defect in an upstream document is cited rather than edited. `created` gains `CLAUDE.md` on the run that created it.

`--json` `data`:

```json
{
  "created": [".devforgeai", ".devforgeai/reports"],
  "copied": { "skills": 9, "agents": 14 },
  "commands": ["explore", "discover", "constitute", "plan", "build", "verify", "release", "design", "reflect"],
  "hooks": { "settings": "merged", "git": ["pre-commit", "commit-msg", "pre-push"] },
  "stacks": ["rust"],
  "trusted": true,
  "degraded": false,
  "analyze": { "derived": ["tech-stack", "source-tree", "dependencies"],
               "stubs": ["coding-standards", "architecture-constraints", "anti-patterns"],
               "skipped": [], "files_scanned": 812, "truncated": false }
}
```

Human output:

```
Initialised .devforgeai/ in C:\Projects\acme
Copied     9 skills, 14 agents
Commands   /explore, /discover, /constitute, /plan, /build, /verify, /release, /design, /reflect
Hooks      .claude/settings.json merged; git hooks pre-commit, commit-msg, pre-push
CLAUDE.md  devforgeai section written
Stack      rust (cargo)
Analysis   3 context files drafted, 3 stubbed, 812 files scanned
Next       /explore
```

The `Commands` line names the slash commands this install produced, and `data.commands` carries the same list. A skill's slash command is its directory name under `.claude/skills/`, which `init` takes from the source skill's frontmatter `name` rather than from the framework's own long directory name, so the line says what the user can actually type. It is omitted when no skill was copied.

The `CLAUDE.md` line reads `written` on the run that created the file and `updated` on every later run. With no `.git` directory the human output's `Hooks` line reads `git hooks none` in place of the three names, `data.hooks.git` is `[]`, and `DFA-E130` goes to stderr with exit 0. With `--no-hooks` the `Hooks` line is omitted.

`init` runs `trust verify` at the end and reports the result in `data.trusted`. On a failure the `Next` line becomes `Next       devforgeai trust pin --framework <framework root>, in a terminal outside Claude Code`, and a warning repeats it with the diagnostic: an unpinned binary refuses every write, every phase command, and every turn end, so the next thing to do is the pin rather than `/explore`. This is the only place the message reaches an operator before the refusals start.

Exit codes: 0 on success, including the `DFA-E130` and `DFA-E113` cases; 1 on `DFA-E110`, `DFA-E111`, `DFA-E112`, `DFA-E131`, `DFA-E132`; 3 on `DFA-E010`, `DFA-E031`; 5 on `DFA-E900`, `DFA-E901`.

#### `init --analyze`, brownfield

The analysis walks the project root, excluding `.git`, `node_modules`, `target`, `dist`, `build`, `out`, `vendor`, `.venv`, `__pycache__`, `bin`, `obj`, and every path in `config.toml` `[coverage].exclude`. The directory walk stops at depth 6. Each file is read to a cap of 256 KiB. The whole analysis stops after 120 seconds, writes `DFA-E113` on stderr, and adds `analysis truncated after <n> files` to `open_questions` of every file it drafted.

Each of the six files gets the §5 frontmatter with `schema: devforgeai/context-<stem>/1`, `id: <stem>`, `phase: constitute`, `status: draft`, `produced_by: establishing-context`, `consumes: []`, and the `open_questions` list described below. The producer value is the skill's, not the CLI's, because `doc validate` reads the doc-type row and the PreToolUse producer check gives the six files to `establishing-context` alone. All six carry `status: draft`, so `context audit` CA-2 keeps them out of a commit until the user accepts them. Three are derived from code, three are stubs whose body the Constitute skill fills.

| File | Derived or stub | Content written from code alone |
|---|---|---|
| `context/tech-stack.md` | derived | `## Languages`, one row per `[[stack]]` with the marker files that matched; `## Package managers`, the detected manager and lockfile; `## Test tooling`, the test and coverage commands; `## Lint tooling`, the lint command or `none detected`; `## Runtime versions`, the pins found in `rust-toolchain.toml`, `.nvmrc`, `package.json` `engines`, `.python-version`, the `go` line of `go.mod`, `global.json`, `.ruby-version`, `pom.xml` `maven.compiler.release`; `## Open`, one bullet per unresolved item, mirrored into `open_questions` |
| `context/source-tree.md` | derived | `## Tree`, the directory tree to depth 3 with per-directory file counts; `## Layers`, each directory mapped to `domain`, `application`, `infrastructure`, `interface`, or `unassigned` by the `[[layer]]` globs; `## Entry points`, files named `main.*`, `index.*`, `Program.cs`, `app.*`, `cmd/*/main.go`; `## Test roots`, directories matching the `[coverage].exclude` test patterns |
| `context/dependencies.md` | derived | `## Direct dependencies`, one row per manifest entry with name, version constraint, and manifest path, read from `Cargo.toml`, `package.json`, `pyproject.toml`, `requirements.txt`, `go.mod`, `*.csproj`, `pom.xml`, `build.gradle*`, `Gemfile`; `## Lockfile status`, present or absent per ecosystem; `## Unverified`, entries whose version the manifest leaves open |
| `context/coding-standards.md` | stub | `## Detected configuration`, the names of the formatter and linter config files found among `.editorconfig`, `rustfmt.toml`, `.prettierrc*`, `.eslintrc*`, `ruff.toml`, `.rubocop.yml`, `checkstyle.xml`; the headings `## Naming`, `## Formatting`, `## Error handling`, `## Testing` present with the body line `Drafted by the Constitute skill.` |
| `context/architecture-constraints.md` | stub | `## Layer map`, the `[[layer]]` globs as detected; `## Module boundaries`, the top-level directories under each source root; `## Constraints` present with no `CON-nnn` entries and the body line `Drafted by the Constitute skill.` |
| `context/anti-patterns.md` | stub | `## Enabled lint rule sets`, the rule set names read from the linter config files; `## Patterns` present with no entries and the body line `Drafted by the Constitute skill.` |

The analysis allocates no ID and writes no ADR; a context file's `id` is its filename stem, which takes no allocation. A context file that already exists is left untouched unless `--force` is passed, and its name appears in `data.analyze.skipped`. `open_questions` of each stub contains `body drafted by the Constitute skill`.

### `stack detect`

```
devforgeai stack detect [--dry-run] [--json] [--project <path>]
```

`--dry-run` is a bool, default false; it prints the result and writes nothing.

Markers are searched at the project root and at directory depth 1 and 2 below it, excluding the directories listed under `init --analyze`. Detection order is the table order; every ecosystem whose markers match produces one `[[stack]]` table.

| `id` | Marker files | `package_manager` | `test_command` | `coverage_command` | `coverage_format`, `coverage_paths` | `lint_command` | `source_roots` |
|---|---|---|---|---|---|---|---|
| `rust` | `Cargo.toml` | `cargo` | `cargo test --all-features` | `cargo llvm-cov --all-features --lcov --output-path target/devforgeai/lcov.info` | `lcov`, `["target/devforgeai/lcov.info"]` | `cargo clippy --all-targets --all-features -- -D warnings` | `["src"]` |
| `node` | `package.json` | `bun` with `bun.lockb`, `pnpm` with `pnpm-lock.yaml`, `yarn` with `yarn.lock`, else `npm` | `<pm> run test` when `scripts.test` exists, else `""` | `<pm> run coverage` when `scripts.coverage` exists, else `""` | `lcov`, `["coverage/lcov.info"]` | `<pm> run lint` when `scripts.lint` exists, else `""` | `["src"]` when `src/` exists, else `["."]` |
| `python` | `pyproject.toml`, `setup.py`, `requirements.txt` | `uv` with `uv.lock`, `poetry` with `poetry.lock`, else `pip` | `python -m pytest -q` | `python -m pytest -q --cov --cov-report=xml:.devforgeai/coverage.xml` | `cobertura`, `[".devforgeai/coverage.xml"]` | `python -m ruff check .` when `ruff.toml`, `.ruff.toml`, or a `[tool.ruff]` table exists, else `""` | `["src"]` when `src/` exists, else the top-level packages holding `__init__.py` |
| `go` | `go.mod` | `go` | `go test ./...` | `go test ./... -coverprofile=.devforgeai/coverage.out` | `go-cover`, `[".devforgeai/coverage.out"]` | `go vet ./...` | `["."]` |
| `dotnet` | `*.sln`, `*.csproj`, `*.fsproj` | `dotnet` | `dotnet test` | `dotnet test --collect:"XPlat Code Coverage" --results-directory .devforgeai/coverage` | `cobertura`, `[".devforgeai/coverage/**/coverage.cobertura.xml"]` | `dotnet format --verify-no-changes` | the directories holding the project files |
| `jvm` | `pom.xml`, `build.gradle`, `build.gradle.kts` | `maven` with `pom.xml`, else `gradle` | maven `mvn -q -B test`; gradle `./gradlew test` | maven `mvn -q -B jacoco:report`; gradle `./gradlew jacocoTestReport` | `jacoco`, maven `["target/site/jacoco/jacoco.xml"]`, gradle `["build/reports/jacoco/**/*.xml"]` | `""` | `["src/main/java", "src/main/kotlin"]` filtered to those that exist |
| `ruby` | `Gemfile` | `bundler` | `bundle exec rspec` when `spec/` exists, else `bundle exec rake test` | the test command with `COVERAGE=1` added to `env` | `lcov`, `["coverage/lcov.info"]` | `bundle exec rubocop` when `.rubocop.yml` exists, else `""` | `["lib", "app"]` filtered to those that exist |

Zero marker matches writes `stack = []` and `degraded = true`, prints `Stack undetected; command checks skipped.` on stderr, and exits 0. In that state the `build` gate keeps its `doc_valid` check and its `tests_pass`, `coverage_min`, and `lint_clean` checks evaluate to `skip` with `reason: degraded`, per the degradation rule in `## Outputs`.

Detection does not assign the `devforgeai-json` coverage format. A human sets it, pointing `coverage_paths` at a file the project writes with this shape:

```json
{ "schema": "devforgeai/coverage/1",
  "files": [ { "path": "src/domain/order.rs", "covered": 120, "total": 130 } ] }
```

`[frontend].globs` is written as the default list in `## Outputs`, extended for a `node` stack with `**/*.ts` and `**/*.js` restricted to the `interface` layer globs. `[[layer]]`, `[coverage]`, and `[[verifier]]` keep their previous values when the file exists.

`--json` `data`: `{"stacks":[<the [[stack]] tables as objects>],"degraded":false,"written":true,"path":".devforgeai/config.toml"}`.

Human output, one line per stack, then the flag:

```
rust    cargo   test: cargo test --all-features   coverage: lcov   lint: cargo clippy --all-targets --all-features -- -D warnings
degraded: false
```

Exit codes: 0 in every case, including the undetected case; 1 on `DFA-E104`; 3 on `DFA-E030`, `DFA-E031`; 5 on `DFA-E900`.

### `gate require <phase> <id>`

```
devforgeai gate require <phase> <id> [--json] [--project <path>]
```

`<phase>` is the phase enum, required. `<id>` is required and matches `^[A-Za-z]+-[0-9]{3}$`, `^v[0-9]+\.[0-9]+\.[0-9]+$`, or a `YYYY-MM-DD` date.

**The per-arm ID shape is normative.** Each arm accepts one kind of subject, checked before the gate is loaded or any report path is built; a subject of the wrong kind is `DFA-E013`, exit 3, naming the shape the arm expects. `explore`, `discover`, and `constitute` take an `IDEA-nnn`; `plan` takes an `EPIC-nnn` or a `SPRINT-nnn`; `build` and `verify` take a `STORY-nnn`; `release` takes a version such as `v1.0.0`; `reflect` imposes no shape, because its subject is a date or a window rather than an id; `design` holds no gate, so `DFA-E300` refuses it before the shape is consulted. Without this check every prefix routed through every arm, and a caller asking for `gate require constitute STORY-001` got `DFA-E321` naming a report path that cannot exist rather than the argument error it made.

The predecessor is the `requires` key of the `<phase>` gate; an empty string exits 0 with no output. The phases key on different ID kinds, so the subject of the predecessor gate is resolved by the chain below before the report path is built. Nothing is written.

| Call | Predecessor | Subject resolution | Report read |
|---|---|---|---|
| `gate require explore <IDEA-nnn>` | none | — | none; exit 0 |
| `gate require discover <IDEA-nnn>` | `explore` when `.devforgeai/explore/decision.yaml` exists with that `id`, else none | the same `IDEA-nnn` | `reports/<IDEA-nnn>-explore.yaml`, or exit 0 when no Explore ran |
| `gate require constitute <IDEA-nnn>` | `discover` | the same `IDEA-nnn` | `reports/<IDEA-nnn>-discover.yaml` |
| `gate require plan <EPIC-nnn \| SPRINT-nnn>` | `constitute` | the `IDEA-nnn` at the top-level `id` of `requirements.yaml`, the document whose `epics[]` holds the epic; a `SPRINT-nnn` argument resolves to its epic through `sprint.yaml` `epic` first | `reports/<IDEA-nnn>-constitute.yaml` |
| `gate require build <STORY-nnn>` | `plan` | the `SPRINT-nnn` of the `stories/sprint.yaml` whose `stories[]` or `deferred[]` lists the story; a story listed in no sprint is `DFA-E013`, exit 3 | `reports/<SPRINT-nnn>-plan.yaml` |
| `gate require verify <STORY-nnn>` | `build` | the same `STORY-nnn` | `reports/<STORY-nnn>-build.yaml` |
| `gate require release <vX.Y.Z>` | `verify` | every `STORY-nnn` under `stories` in `releases/<vX.Y.Z>.yaml`, or, when that file is absent, every `stories[].id` of `stories/sprint.yaml` | one `reports/<STORY-nnn>-verify.yaml` per story; every one passes or the call fails |
| `gate require reflect <window>` | none | — | none; exit 0, since `requires` is `""`. The arm imposes no id shape: a Reflect window is a date, an `IDEA-nnn`, an `EPIC-nnn`, a `STORY-nnn`, or a version |
| `gate require design <id>` | — | — | exit 1 with `DFA-E300`; Design is not a phase and holds no gate, and `gate check --phase design` fails the same way |

**The first release.** `gate require release` runs before `releases/<vX.Y.Z>.yaml` exists, because the Release skill writes that file during the run the gate opens. An absent manifest is therefore `DFA-W210`, exit 0, and the story set is taken from `stories/sprint.yaml` `stories[].id` instead. That is a fallback in where the list comes from and not in what is checked: every story the sprint names still has its `reports/<STORY-nnn>-verify.yaml` read, and a story short of `PASS` still refuses the call with `DFA-E321`. With the sprint absent too the call exits 0 with the warning and no reports read, and the run stops one step later — `phase set` refuses to advance on a story set that read empty, so an empty read never stands in for a passing one.

A manifest that is present and will not parse is not this case: it stays `DFA-E200` or `DFA-E401`. The fallback answers a file that does not exist yet; a file that exists and is broken is a defect to repair, and reading the sprint instead would hide it.

Explore and Discover are entry phases. When the predecessor report named above is absent and the predecessor is `none`, the call exits 0; when the predecessor exists and its report is absent or not `PASS`, the call exits 1 with `DFA-E321`. The gate passes when every report read carries `gate.result: PASS`.

A `PASS` whose report holds a check recorded `status: skip` with `reason: not_implemented` does not satisfy the call: `gate require` exits 1 with `DFA-E321`, naming the check ids that were not implemented. A build that stubs a check kind therefore blocks the phase transition rather than letting an unevaluated gate stand as a pass.

`--json` `data`: `{"phase":"build","id":"STORY-014","requires":"plan","subject":"SPRINT-001","reports":[".devforgeai/reports/SPRINT-001-plan.yaml"],"result":"PASS"}`.

Human output on pass: nothing. On failure, one line: `gate require: build needs plan gate PASS for SPRINT-001; the last result is FAIL at 2026-09-10T13:41:02Z`.

Exit codes: 0 pass; 1 `DFA-E321`, `DFA-E300`, `DFA-E303`; 3 `DFA-E012`, `DFA-E013`.

### `gate check`

```
devforgeai gate check --phase <phase> [--id <id>] [--partial] [--no-run] [--json] [--project <path>]
```

| Argument | Type | Default | Effect |
|---|---|---|---|
| `--phase <phase>` | phase enum, plus `reflect` | required | the gate evaluated; `design` is `DFA-E300` |
| `--id <id>` | ID | `state.toml` `[active].<phase>`; an empty value is `DFA-E011` | the gate subject; `--phase reflect` has no `[active]` key and takes a `YYYY-MM-DD`, so omitting `--id` there is `DFA-E011` |
| `--partial` | bool | false | evaluates only `tests_pass`, `coverage_min`, `lint_clean`, `complexity_clean`; writes the report with `partial: true`; leaves `state.toml` untouched; exits 0 whatever the result |
| `--no-run` | bool | false | executes no command; `tests_pass`, `lint_clean`, `complexity_clean`, and `docs_cover` become `skip` with `reason: no_run`; `coverage_min` takes its `source = "read"` path and parses the coverage artifact on disk, which judges the evidence without running anything |

Algorithm, in order:

1. Load `gates.toml` and validate it against the compiled minimums; a violation exits 1 before any command runs.
2. Load `reports/<id>-<phase>.yaml` when it exists, keeping its `verifiers` block; a parse failure is `DFA-E401`.
3. Evaluate each `gate.check` in file order, recording one entry per check in the report.
4. The result is `PASS` when every `severity = "block"` check is `pass` or `skip`. Otherwise, when at least one failing block check resolves `on_fail = "send_back"`, the result is `SEND BACK`; with no such check the result is `FAIL`. A check's `on_fail` defaults to the gate's.
4a. On `SEND BACK` the report's `gate.send_back_to` is resolved from the blocking `findings[]`, not from the gate alone. The prefix of each blocking finding's `id` maps `REQ` to `discover`, `CON` and `AP` to `constitute`, `UI` to `design`, `AC` to `plan`, and `FIND` to `build`. Two prefixes that map to different phases resolve to the earlier phase in §5 order, so a missing constraint is repaired before the requirement it would have decided. A finding whose prefix the map does not name, and an empty blocking set, fall back to the gate's static `send_back_to`. The verify gate adds one rule of its own: a blocking finding carrying `category: spec-gap` maps to `plan`, and a set holding both that and another blocking finding resolves `build`.
5. Write the report atomically, then `state.toml` `[last_gate]` unless `--partial` is set.

Command execution: the command string goes to `cmd.exe /C <cmd>` on Windows and `/bin/sh -c <cmd>` elsewhere, with the working directory set to the project root, the environment extended by `[[stack]].env` plus `DEVFORGEAI=1` and `CI=1`, stdout and stderr captured, and a wall-clock limit of `timeout_secs`. On timeout the child process tree is terminated and the check fails with `DFA-E318`. The report records `command`, `exit_code`, `duration_ms`, and `output_tail`, the last 4096 bytes of the merged output.

Test counts are informational. `passed` and `failed` are filled when the merged output matches the pattern for the stack and are `null` otherwise. The check result comes from the exit code alone.

| Stack | Pattern |
|---|---|
| rust | `test result: ok\. (\d+) passed; (\d+) failed` |
| node | `Tests\s+(\d+) passed` and `(\d+) failed` |
| python | `(\d+) passed` and `(\d+) failed` |
| go | lines matching `^ok\s+\S+` counted as passed, lines matching `^FAIL` counted as failed |
| dotnet | `Failed:\s+(\d+),\s+Passed:\s+(\d+)` |
| jvm | `Tests run: (\d+), Failures: (\d+)` |
| ruby | `(\d+) examples?, (\d+) failures?` |

Coverage evaluation: with `source = "run"` the `coverage_command` of every stack runs first; with `source = "read"` no command runs. Each glob in `coverage_paths` is expanded and the most recently modified match is parsed with the parser for `coverage_format`. File paths are normalised to project-relative forward-slash form, joining the cobertura `sources` prefix and the jacoco `package` and `sourcefile` pair first. Files matching `[coverage].exclude` are dropped. Each remaining file takes the name of the first `[[layer]]` whose globs match in `config.toml` order, and `unassigned` when none match. A layer percentage is `covered * 100 / instrumented`, rounded to one decimal, half away from zero; a layer with zero instrumented lines scores `100.0` with `total: 0`. The overall figure is computed over every non-excluded file. With `unassigned_policy = "fail"`, a non-empty unassigned set fails the check with `DFA-E313` instead of warning with `DFA-W320`.

Verifier evaluation: a `verifier_pass` check reads `verifiers.<report_field>` from the report loaded in step 2, where `report_field` comes from the `[[verifier]]` table whose `name` matches. A `verifiers` list that is empty is `DFA-E316`, as is a name `config.toml` registers no `[[verifier]]` for, and an absent block. `passed` and `total` are read as present integers: a missing key, or one holding anything but an integer, is `DFA-E317`, because defaulting it to 0 would make a block that counted nothing read as the free pass that belongs to a verifier that genuinely counted zero units. A `passed` or `total` below zero, or a `passed` above `total`, is `DFA-E317`. `total == 0` is a ratio of `1.0` and passes, which is the one place a verifier with no unit to count is allowed through. A ratio below `min_ratio` is `DFA-E317`, and the check carries every `severity: block` finding of the report as its evidence.

`--json` `data`:

```json
{
  "phase": "build", "id": "STORY-014", "result": "FAIL", "send_back_to": "",
  "partial": false, "degraded": false,
  "report": ".devforgeai/reports/STORY-014-build.yaml",
  "checks": [ { "id": "build-tests", "kind": "tests_pass", "status": "pass",
                "severity": "block", "reason": "", "evidence": {} } ],
  "coverage": { "overall": 87.4,
                "layers": [ { "name": "domain", "percent": 96.6, "min": 95.0, "status": "pass" } ],
                "unassigned": { "files": 3, "percent": 61.0 } },
  "findings": [ { "id": "FIND-001", "severity": "block", "summary": "AC-003 has no test" } ]
}
```

Human output:

```
Gate      build · STORY-014
  pass    build-docs        doc_valid       4 documents
  pass    build-tests       tests_pass      214 passed, 0 failed in 41.2s
  fail    build-coverage    coverage_min    domain 88.1%, below 95.0%
  skip    build-lint        lint_clean      degraded
Result    FAIL
Report    .devforgeai/reports/STORY-014-build.yaml
```

Exit codes: 0 for `PASS`, and 0 for any result under `--partial`; 1 for `FAIL` and for every `DFA-E1xx` and `DFA-E3xx` load failure; 2 for `SEND BACK`; 3 for usage; 5 for `DFA-E9xx`.

### `doc validate`

```
devforgeai doc validate [<path>...] [--all] [--allocate <prefix>] [--producer-check] [--stdin-content] [--json] [--project <path>]
```

| Argument | Type | Default | Effect |
|---|---|---|---|
| `<path>...` | paths | none | the documents validated; relative paths resolve against the project root |
| `--all` | bool | false | validates every file under `.devforgeai/` whose path matches the doc-type table |
| `--allocate <prefix>` | string | none | prints the next free ID for the prefix and exits; incompatible with `<path>` and `--all` |
| `--producer-check` | bool | false | runs the frontmatter and producer checks only, over exactly one path; skips heading order and cross-references |
| `--stdin-content` | bool | false | reads the document body from stdin instead of the file, for a write that has not landed yet |

No path, no `--all`, and no `--allocate` is `DFA-E011`.

#### Doc-type table

The doc type comes from the path. Every other row value follows from it.

| Doc type | Path | Producer skill | Phase | `schema` | `id` grammar | `status` enum | ID prefixes | Extra top-level keys |
|---|---|---|---|---|---|---|---|---|
| `explore-brief` | `explore/brief.md` | `exploring-ideas` | explore | `devforgeai/explore-brief/1` | `^IDEA-[0-9]{3}$` | `drafting`, `scanned`, `specified`, `mocked`, `decided` | IDEA, FLOW | refused |
| `explore-decision` | `explore/decision.yaml` | `exploring-ideas` | explore | `devforgeai/explore-decision/1` | `^IDEA-[0-9]{3}$` | `recorded` | IDEA, FLOW | permitted |
| `explore-payload` | `explore/*.json` | `exploring-ideas` | explore | `^devforgeai/[a-z-]+/1$` | `^IDEA-[0-9]{3}$` | any non-empty string | IDEA, FLOW | permitted |
| `requirements` | `requirements.yaml` | `discovering-requirements` | discover | `devforgeai/requirements/1` | `^IDEA-[0-9]{3}$` | `drafting`, `awaiting_acceptance`, `accepted`, `reopened` | REQ, EPIC, PERSONA, IDEA, FLOW, UI | permitted |
| `context` | `context/<stem>.md`, stem one of `tech-stack`, `source-tree`, `dependencies`, `coding-standards`, `architecture-constraints`, `anti-patterns` | `establishing-context` | constitute | `devforgeai/context-<stem>/1` | the stem verbatim | `draft`, `accepted` | CON, AP | refused |
| `adr` | `adr/ADR-<nnn>.md` | `establishing-context` | constitute | `devforgeai/adr/1` | `^ADR-[0-9]{3}$` | `proposed`, `accepted`, `rejected`, `superseded` | ADR, CON, REQ | refused |
| `story` | `stories/STORY-<nnn>.md` | `planning-work` | plan | `devforgeai/story/1` | `^STORY-[0-9]{3}$` | `draft`, `ready`, `building`, `built`, `released` | STORY, AC | refused |
| `sprint` | `stories/sprint.yaml` | `planning-work` | plan | `devforgeai/sprint/1` | `^SPRINT-[0-9]{3}$` | `planned`, `active`, `closed` | SPRINT, STORY | permitted |
| `ui-spec` | `ui-specs/UI-<nnn>.md` | `designing-interfaces` | design | `devforgeai/ui-spec/1` | `^UI-[0-9]{3}$` | `draft`, `approved` | UI | refused |
| `tokens` | `brand/tokens.json` | `designing-interfaces` | design | `devforgeai/tokens/1` | `^TOKEN-[0-9]{3}$` | `draft`, `approved` | TOKEN | permitted |
| `brand-kit` | `brand/brand-kit.md` | `designing-interfaces` | design | `devforgeai/brand-kit/1` | `^TOKEN-[0-9]{3}$` | `draft`, `approved` | TOKEN | refused |
| `build-report` | `reports/STORY-<nnn>-build.yaml` | `devforgeai-cli` | build | `devforgeai/report/1` | `^STORY-[0-9]{3}$` | `pass`, `fail`, `send_back`, `skip` | none | permitted |
| `gate-report` | `reports/<ID>-<phase>.yaml` for the phases `explore`, `discover`, `constitute`, `plan`, `verify`, `release`, `design` | `devforgeai-cli` | the phase in the filename | `devforgeai/report/1` | the subject ID of that phase | `pass`, `fail`, `send_back`, `skip` | none | permitted |
| `qa-report` | `reports/STORY-<nnn>-qa.yaml` | `validating-quality` | verify | `devforgeai/qa-report/1` | `^STORY-[0-9]{3}$` | `pass`, `fail`, `send_back` | FIND | permitted |
| `release` | `releases/v<X>.<Y>.<Z>.yaml` | `releasing-software` | release | `devforgeai/release/1` | `^v[0-9]+\.[0-9]+\.[0-9]+$`, and the triple compares strictly greater than the highest triple among the other `releases/v*.yaml` names | `draft`, `released` | none | permitted |
| `reflect-report` | `reports/reflect-<YYYY-MM-DD>.yaml` | `improving-framework` | reflect | `devforgeai/reflect-report/1` | `^[0-9]{4}-[0-9]{2}-[0-9]{2}$`, the one doc type whose id is a date | `draft`, `final` | OBS, REC | permitted |

A path under `.devforgeai/` matching no row is skipped with no diagnostic, so a skill may keep scratch files there.

The `Extra top-level keys` column decides `DFA-E204`. A `permitted` row carries payload beyond the seven §5 keys, which is why the CLI's own reports, `requirements.yaml`, `explore/decision.yaml`, `sprint.yaml`, and `tokens.json` pass the `pre-commit` hook that validates staged `.devforgeai/` files. The seven §5 keys are checked in every row, first and in order, whatever the column says.

ID uniqueness is scoped per `schema` value. `explore/brief.md` and `explore/decision.yaml` both carry `id: IDEA-nnn` under different schemas and both are valid; the same `id` twice under one schema is `DFA-E209`.

The `release` row carries two content rules beyond the shape: a malformed id is `DFA-E217`, and a version that is not greater than the highest existing one is `DFA-E218`. `previous_version` inside the document is either `""` or that highest triple.

The `story` enum has five values. `verified` is dropped: `phase set verify` writes `built`, `phase set release` writes `built` to `released`, the producer check blocks a Verify-phase write to Plan's document, and §2 forbids a status-transition instruction in skill prose, so no actor wrote it. The `verify-story-status` check keeps `values = ["built", "verified"]` as `specs/07-verify.md` writes it; the second value is unreachable and constrains nothing.

`TOKEN-<name>` in a UI spec or a stylesheet is a token reference, not an ID-index entry: the index pattern is `[A-Z]+-[0-9]{3}`, which a lowercase leaf name does not match. `design lint --tokens` resolves those names.

#### Frontmatter grammar

For a Markdown document: line 1 is exactly `---`; the block ends at the next line that is exactly `---`; the text between is a YAML mapping. For a YAML document: the same keys are top-level keys of the document, and a leading `---` document marker is permitted and ignored. For `brand/tokens.json` the keys live under the top-level object key `meta`.

A YAML template in a spec fence or under a skill's `templates/` is one document and carries no `---` separator of its own. A file opening on `---` and closing on `---` parses as a document with empty content followed by the real one, and `doc validate` reads the first, which is `DFA-E202` — the frontmatter is not a mapping because there is no frontmatter. A YAML template therefore opens on its first key.

Keys, in this order, first in the document, and with no other top-level key in the nine doc types that refuse extras:

| Key | Type | Constraint |
|---|---|---|
| `schema` | string | equals the doc-type row's `schema` |
| `id` | string | matches the `id` grammar of the doc-type row, and where that grammar is an ID form, its prefix is in the row's ID-prefix list |
| `phase` | string | equals the row's phase, or any phase for `reflect-report` |
| `status` | string | member of the row's status enum |
| `produced_by` | string | matches `^[a-z][a-z0-9-]*$` and equals the row's producer skill |
| `consumes` | array of strings | each matches the ID form; `[]` is valid |
| `open_questions` | array of strings | `[]` is valid |

#### ID index, definitions, references

The index is built by walking `.devforgeai/`. An ID is *defined* by any of: the frontmatter `id` of a document; an ID at the start of a Markdown heading line, as in `### AC-003 …`; a Markdown list item whose text begins `<PREFIX>-<nnn>:` at the start of the line, which is the acceptance-criterion form `- AC-003: Given …`; a mapping key `id:` inside a YAML sequence item. Every other occurrence of the pattern `\b[A-Z]+-[0-9]{3}\b` is a *reference*. A reference with no definition is `DFA-E210`. A definition appearing twice is `DFA-E209`. An entry of `consumes` with no definition is `DFA-E210`. `DFA-W201` and `DFA-W202` report the two directions of drift between `consumes` and the body.

`--allocate <prefix>` reads the index, takes the highest numeric suffix for the prefix, adds one, and reserves the result on disk before printing it. An unused prefix returns `<prefix>-001`. A prefix whose highest number is `999` is `DFA-E215`. The prefix is one of the sixteen §5 prefixes: `IDEA`, `FLOW`, `PERSONA`, `REQ`, `EPIC`, `CON`, `AP`, `ADR`, `STORY`, `AC`, `SPRINT`, `UI`, `TOKEN`, `FIND`, `OBS`, `REC`; any other prefix is `DFA-E214`.

**The reservation.** The allocation writes one empty marker file, `.devforgeai/.allocated/<ID>`, created with `create_new` so the write is atomic. Reading the index alone would hand two worktrees sharing one `.devforgeai/` the same number, and the collision would surface only when the branches merge. The loser of the race gets `AlreadyExists` and takes the next number. The high-water mark the next allocation starts from is the union of the document definitions and the reservations, so an ID that is reserved and not yet written into a document is never handed out twice. `.devforgeai/.allocated/` matches no doc-type row, so the producer check skips it with no diagnostic, and `doc validate --all` does not walk it. The marker outlives the run: an allocation the caller then abandons leaves a number unused rather than reissued, which is the cheaper of the two failures.

The allocatable prefixes are the closed list `IDEA`, `FLOW`, `PERSONA`, `REQ`, `EPIC`, `CON`, `AP`, `ADR`, `STORY`, `AC`, `SPRINT`, `UI`, `TOKEN`, `FIND`, `OBS`, `REC`. Any other value is `DFA-E214`. `AP` is Constitute's anti-pattern prefix, which §5 does not list; it is recorded under `## Decisions`. The six context files take no allocation, since their `id` is the filename stem.

#### Heading order

For each Markdown doc type the template in the producing skill's spec fixes the H2 list. `doc validate` compares the H2 headings in the file against that list by verbatim text: an absent heading or a heading out of order is `DFA-E213`. An H2 the template does not define is `DFA-E213` with the position text `absent`. Headings below H2 are not checked.

#### Producer check

`--producer-check <path>` reads `state.toml` `[current].phase`, maps it to the producer skill through the doc-type table, and compares that skill to the doc type of `<path>`. Equal is exit 0. Unequal is `DFA-E212`, exit 1. The `design` and `reflect` doc types are allowed from every phase, since both skills are cross-cutting. When `--stdin-content` is passed, the frontmatter checks run against the stdin text and the file on disk is not read; the file being absent is not an error in that mode.

On PreToolUse the hook wrapper maps exit 1 from `--producer-check` to hook exit 2, which blocks the write and returns stderr to the model, per conventions §7.

`--json` `data`:

```json
{ "checked": 4, "failed": 1,
  "files": [ { "path": ".devforgeai/stories/STORY-014.md", "doc_type": "story", "id": "STORY-014",
               "status": "ready", "valid": false,
               "errors": [ { "code": "DFA-E210", "message": "...", "line": 7 } ], "warnings": [] } ] }
```

`--allocate` `data`: `{"prefix":"REQ","id":"REQ-014","scanned":13,"reservation":".devforgeai/.allocated/REQ-014"}`.
`--producer-check` `data`: `{"path":"...","doc_type":"story","expected_producer":"planning-work","current":{"phase":"plan","id":"STORY-014"},"allowed":true}`.

Human output: one line per file, `ok  .devforgeai/stories/STORY-014.md  story  STORY-014`, or `fail` with the diagnostics on stderr. `--allocate` prints the ID alone, with a trailing newline and nothing else.

Exit codes: 0 when every file is valid; 1 on any `DFA-E2xx`; 3 on `DFA-E011`, `DFA-E214`; 5 on `DFA-E9xx`.

### `doc load <name> <id>`

```
devforgeai doc load <name> <id> [--json] [--project <path>]
```

`<name>` is a doc type from the table above. `<id>` selects the file:

| `<name>` | `<id>` meaning |
|---|---|
| `explore-brief`, `explore-decision`, `requirements`, `sprint`, `tokens` | ignored; pass `-` |
| `context` | one of the six base names, or `all` for the six concatenated in the §3 order, separated by a line containing three dashes |
| `adr` | `ADR-<nnn>`, or `all` for every ADR ordered by number |
| `story`, `qa-report`, `build-report` | `STORY-<nnn>` |
| `ui-spec` | `UI-<nnn>` |
| `release` | `v<X>.<Y>.<Z>` |
| `reflect-report` | `<YYYY-MM-DD>`, or `latest` |
| `brand-kit` | ignored; pass `-` |
| `discover-entry` | a quoted string: an `IDEA-nnn`, or any other text |

`discover-entry` is the one name whose absence is not an error, because the `/discover` preamble runs before the entry point is known. With an `IDEA-nnn` argument it prints `explore/brief.md` and `explore/decision.yaml` when a brief exists, prints `requirements.yaml` when requirements exist and no brief does, prints both when both exist, and prints nothing when the id matches no document. With any other argument it prints nothing. It exits 0 in every one of those cases, and 1 only on `DFA-E900`.

The file content goes to stdout byte for byte, with no added header. `--json` `data`: `{"name":"story","id":"STORY-014","path":".devforgeai/stories/STORY-014.md","bytes":4211,"content":"..."}`.

Exit codes: 0 on success; 1 on `DFA-E200`; 3 on `DFA-E250`, `DFA-E011`; 5 on `DFA-E900`.

### `doc accept requirements --id <IDEA-nnn>`

```
devforgeai doc accept requirements --id <IDEA-nnn> [--json] [--project <path>]
```

`requirements` is the only document name this verb takes; any other is `DFA-E250`. The command writes `accepted_by: user`, `accepted_at` in RFC 3339 UTC, the document `status: accepted`, and moves every requirement at `draft` or `reopened` to `accepted`. Every other byte is unchanged.

Exit 0 on success. Exit 1 with `DFA-E260` when `epics[]` or `requirements[]` is empty, message `requirements.yaml has no <epics | requirements>; nothing accepted`. Exit 1 on `DFA-E200` when the file is absent.

`--json` `data`: `{"id":"IDEA-004","accepted_at":"2026-09-10T14:22:05Z","requirements_moved":7,"status":"accepted"}`.

Human output: `Accepted  IDEA-004 · 7 requirements · 2 epics`.

### `doc reopen requirements --id <IDEA-nnn>`

```
devforgeai doc reopen requirements --id <IDEA-nnn> --ids <ID,ID> --from <plan|constitute|design> [--json] [--project <path>]
```

`--ids` is a comma-separated list of `REQ-nnn` and `UI-nnn`, required. `--from` is the closed enum `plan`, `constitute`, `design`, required.

The command raises `revision` by 1, appends the `revision_log` entry, moves each cited `REQ-nnn` to `reopened` in place, allocates one `REQ-nnn` per cited `UI-nnn` with `source: user`, `traces_to: [<UI-nnn>]`, and no epic, sets `accepted_by` and `accepted_at` to null, sets the document `status: reopened`, and leaves every other byte untouched.

Exit 0 on success. Exit 3 with `DFA-E261` when a cited id matches neither `^REQ-[0-9]{3}$` nor `^UI-[0-9]{3}$`, message `'<id>' is neither a REQ nor a UI id`. Exit 1 on `DFA-E200`, and on `DFA-E210` when a cited `REQ-nnn` is not in the document.

`--json` `data`: `{"id":"IDEA-004","revision":2,"reopened":["REQ-014"],"allocated":["REQ-021"],"from":"design","status":"reopened"}`.

Human output: `Reopened  IDEA-004 · revision 2 · REQ-014 reopened · REQ-021 allocated`.

### `explore prune --id <IDEA-nnn>`

```
devforgeai explore prune --id <IDEA-nnn> [--json] [--project <path>]
```

The command reads `decision` from `.devforgeai/explore/decision.yaml` for that id. With `kill` or `promote` it removes the `.explore-prototype/` directory and its contents. With `park` it leaves the directory in place. An absent directory is success. An absent or unparsable `decision.yaml` is `DFA-E200` or `DFA-E401`, exit 1. A path inside the directory that cannot be removed is `DFA-E262`, exit 1, message `removing <path> failed: <io message>`.

`--json` `data`: `{"id":"IDEA-001","decision":"promote","removed":true,"path":".explore-prototype","files":14}`.

Human output: `Pruned    .explore-prototype (14 files) after a promote decision`.

### `story files`

```
devforgeai story files --check <path> [--id <STORY-nnn>] [--json] [--project <path>]
devforgeai story files --list [--id <STORY-nnn>] [--json] [--project <path>]
devforgeai story files --diff [--id <STORY-nnn>] [--base <ref>] [--json] [--project <path>]
```

Exactly one of `--check`, `--list`, and `--diff` is given; none or two is `DFA-E011`. `--id` defaults to `state.toml` `[active].build`.

`--check` exits 0 when `<path>`, made repo-relative, equals a `Path` value in that story's `## Files` table, and exits 1 with `DFA-E239` otherwise. `--list` prints one `Path`, `Kind`, `Layer` triple per line. `--diff` takes the union of the paths changed between `--base` and `HEAD` and the uncommitted changes of the work tree, tests each by the `--check` rule, and exits 1 with `DFA-E239` and one stderr line per undeclared path. `--base` defaults to the merge-base of `config.toml` `[build].base_ref` and the current work tree's branch head.

An absent story is `DFA-E200`, exit 1. An absent `[active].build` is `DFA-E412`, exit 0, so the PreToolUse hook does not block outside a Build run.

`--json` `data` for `--check`: `{"id":"STORY-104","path":"src/application/checkout/place_order.ext","allowed":true,"declared":2}`. For `--list`: `{"id":"STORY-104","files":[{"path":"src/…","kind":"source","layer":"application"}]}`. For `--diff`: `{"id":"STORY-014","base":"HEAD","paths":[{"path":"src/…","declared":true,"kind":"source","status":"modified"}],"undeclared":0}`.

Exit codes: 0 allowed, listed, or clean; 1 on `DFA-E239`, `DFA-E200`; 3 on `DFA-E011`, `DFA-E013`; 5 on `DFA-E9xx`.

### `story list`

```
devforgeai story list [--status <s>[,<s>...]] [--sprint <SPRINT-nnn>] [--json] [--project <path>]
```

The command walks `.devforgeai/stories/STORY-*.md`, reads each frontmatter `id`, `status`, and `consumes`, and the first H1 as the title. `--status` filters to the listed values, each a member of the `story` status enum; an unknown value is `DFA-E230`, exit 3. `--sprint` filters to the ids under `stories[]` of `stories/sprint.yaml`.

Human output is one line per story: `STORY-014  built  Order checkout`. `--json` `data`: `{"count":3,"stories":[{"id":"STORY-014","status":"built","title":"Order checkout","path":".devforgeai/stories/STORY-014.md","consumes":["REQ-007","REQ-011"]}]}`.

Exit codes: 0 when the walk succeeded, including a count of zero; 1 on `DFA-E231` when `.devforgeai/stories/` is absent; 3 on `DFA-E230`; 5 on `DFA-E9xx`.

### `report note <id> <phase> --key <key> --file <path>`

```
devforgeai report note <id> <phase> --key <key> --file <path> [--json] [--project <path>]
```

`<id>` and `<phase>` name the report, resolved as `report show` resolves them. `--key` is drawn from a closed per-phase set: the `build` phase accepts the single value `build`, and every other phase accepts none, so a `--key` outside the set is `DFA-E013`, exit 3. `--file` is a path to a YAML file holding one mapping.

The CLI parses that file, validates it against the `devforgeai/build-note/1` schema, drops its `schema` and `id` keys, and writes the remainder at the report's top-level `<key>`, creating the report from the `## Outputs` skeleton when it is absent. `produced_by` stays `devforgeai-cli`: the note lands under its own key and changes no §5 key of the report. A schema failure is `DFA-E413`, exit 1, naming the first offending key, and nothing is written. An absent `--file` path is `DFA-E400`, exit 1.

This is the one path by which a skill contributes to a CLI-owned report. The PreToolUse producer check refuses a phase's Write to `reports/<ID>-<phase>.yaml` with `DFA-E212`, which is why the skill writes its own note file and hands it over.

`--json` `data`: `{"id":"STORY-014","report":".devforgeai/reports/STORY-014-build.yaml","key":"build","keys_written":9}`. Human output: `Noted     build -> .devforgeai/reports/STORY-014-build.yaml`.

Exit codes: 0 written; 1 on `DFA-E400`, `DFA-E401`, `DFA-E413`; 3 on `DFA-E012`, `DFA-E013`; 5 on `DFA-E9xx`.

### `report aggregate`

```
devforgeai report aggregate [<ID>] [--since <YYYY-MM-DD>] [--session-root <path>] [--json] [--project <path>] [--quiet]
```

`<ID>` matches `^(IDEA|EPIC|STORY)-[0-9]{3}$` or `^v[0-9]+\.[0-9]+\.[0-9]+$`. Exactly one of `<ID>` and `--since` is given; neither or both is `DFA-E430`, exit 3, message `pass one id (IDEA-nnn, EPIC-nnn, STORY-nnn, vX.Y.Z) or --since <YYYY-MM-DD>`. A lone token that is not a recognised id satisfies the exactly-one rule, so `DFA-E430` does not apply to it: the token is then tested against the prefix set and fails with `DFA-E013`, exit 3, message `'<value>' is not an ID; expected PREFIX-nnn with three digits`. `DFA-E430` is reserved for the two cases it names — neither argument, or both.

Window rules. With `<ID>` the window holds every report under `.devforgeai/reports/` whose `id` equals `<ID>`, plus, for a version, every report whose `id` is a `STORY-nnn` listed in `.devforgeai/releases/<ID>.yaml`, plus, for an `EPIC-nnn`, every report whose `id` is a `STORY-nnn` whose story frontmatter `consumes` holds that epic. With `--since` the window holds every report whose `finished_at` is at or after that date at 00:00:00Z. `--since` with no date takes `config.toml` `[reflect].window_days` back from today.

`--session-root` overrides `config.toml` `[reflect].session_root`, default `~/.claude/projects`, where the project key is the absolute project root path with every character outside `[A-Za-z0-9]` replaced by `-`, overridable by `[reflect].session_key`. `sessions.status` is one of `present`, `absent`, `empty`, `unreadable`. A session root that canonicalises outside the user's home directory is `DFA-E421`, exit 1, with `sessions.status: unreadable`; that code is reserved for that case alone. A session root that does not exist — including a relative path that resolves to nothing — is `sessions.status: absent` and exit 0, since a window read from the reports alone is a complete answer and a machine that has never run a session is not a defect.

An unparsable report under `.devforgeai/reports/` is excluded from the window and named: `DFA-W420`, exit 0, the message giving the path and saying that the file does not parse. A warning is the right band because the aggregate is advisory — Reflect's output sets no gate — and refusing the whole window over one defective file would leave the user with no aggregate and no way to see which file to repair. The excluded paths appear in the warning list, one per file, so the count the aggregate reports and the files it could not read are both visible in one run.

`--json` `data` is the `devforgeai/aggregate/1` object: `window`, `reports[]`, `phase_time[]`, `gate_failures[]`, `send_backs[]`, `verifier_failures[]`, `deferrals[]`, `sessions`, `state`, `floors`, and `counts`, every key present on every run, an empty result being an empty array or a zero.

`deferrals[]` is read from the top-level `deferrals[]` sequence of each `reports/STORY-nnn-qa.yaml` in the window, which `specs/07-verify.md` writes with the fields `id`, `story`, `dod_item`, `target`, `reason`, `opened_on`, and `con_or_ap`. The aggregate renames three and adds two:

| Aggregate field | Source |
|---|---|
| `id` | the qa entry's `id`, carried through |
| `story` | the qa entry's `story` |
| `dod_item` | the qa entry's `dod_item` |
| `deferred_at` | the qa entry's `opened_on` |
| `age_days` | whole calendar days from `opened_on` to the run's `generated_at`, an integer |
| `constraint` | the qa entry's `con_or_ap`, `""` when the item is an `AC-nnn` or a coverage floor |
| `reason` | the qa entry's `reason` |
| `report` | the path of the qa report the entry was read from |

The qa entry's `target` is read by the verify gate's `no_cycle` check and is not carried into the aggregate. An entry missing `opened_on` takes `deferred_at` of `""` and `age_days` of `0`.

One `deferrals[]` entry of the aggregate:

```json
{ "id": "FIND-012", "story": "STORY-009", "dod_item": "Integration test for the retry path",
  "deferred_at": "2026-08-01", "age_days": 41, "constraint": "CON-002",
  "reason": "Retry backoff is unspecified until ADR-011 is accepted",
  "report": ".devforgeai/reports/STORY-009-qa.yaml" }
``` Per-phase durations come from each report's `started_at` and `finished_at`; `state` carries `[current]`, `[active]`, `[last_gate]`, and `[last_handoff].rendered_at`; `floors` carries the compiled minimums of `config.toml` and `gates.toml`.

Human output is one line per section with its count. Exit codes: 0 aggregated; 1 on `DFA-E421`, `DFA-E401`; 3 on `DFA-E430`, `DFA-E013`; 5 on `DFA-E9xx`.

### `worktree`

```
devforgeai worktree ensure <STORY-nnn> [--json] [--project <path>]
devforgeai worktree list [--json] [--project <path>]
devforgeai worktree remove <STORY-nnn> [--force] [--json] [--project <path>]
```

The worktree path is `<config.toml [build].worktree_root>/<STORY-nnn>`, resolved against the project root, default `../wt`. The branch is `<[build].branch_prefix><STORY-nnn>`, default prefix `story/`.

`ensure` is idempotent: a path that is already a worktree of this repository on the matching branch is printed and the command exits 0. Otherwise it reads `git worktree list`, takes the `STORY-nnn` encoded in each worktree directory name under `worktree_root`, runs the `story files --list` set for each, and exits 1 with `DFA-E272` naming both story ids and the first shared `Path` when the requested story's set intersects one of them. With no intersection it creates the worktree from `[build].base_ref` on a new branch, copies the main checkout's `.devforgeai/state.toml` into it, and prints the path on stdout. A project root that is not a git work tree is `DFA-E271`, exit 1. `ensure` does not accept `--force`.

`list` prints one line per worktree under `worktree_root`: `<STORY-nnn>  <path>  <branch>  <clean|dirty>  <n> ahead`.

`remove` exits 1 with `DFA-E273` when the worktree holds uncommitted changes or commits absent from `[build].base_ref`, unless `--force` is passed. On success it removes the worktree and deletes the branch when the branch is merged into `base_ref`.

`--json` `data` for `ensure`: `{"id":"STORY-014","path":"../wt/STORY-014","branch":"story/STORY-014","created":true,"state_seeded":true}`. For `list`: `{"worktrees":[{"id":"STORY-014","path":"../wt/STORY-014","branch":"story/STORY-014","dirty":false,"ahead":3}]}`. For `remove`: `{"id":"STORY-014","removed":true,"branch_deleted":true}`.

Exit codes: 0 success; 1 on `DFA-E271`, `DFA-E272`, `DFA-E273`; 3 on `DFA-E012`; 5 on `DFA-E9xx`.

### `commit <STORY-nnn> -m <message>`

```
devforgeai commit <STORY-nnn> -m <message> [--paths <path>,...] [--json] [--project <path>]
```

`--paths` defaults to every path git reports as changed in the current work tree. Each path is tested by the `story files --check` rule first; a path outside the story's `## Files` set is `DFA-E239`, exit 1, and nothing is staged.

The committed message is `<STORY-nnn>: <message>` when `<message>` does not already hold the story id, and `<message>` verbatim when it does, which satisfies the §7 `commit-msg` hook by construction rather than by an instruction to the model. The command stages the paths and commits, so the `pre-commit` and `commit-msg` hooks run unchanged; a non-zero hook exit becomes exit 1 with the hook's stderr on stderr and nothing committed. An empty stage set is `DFA-W243`, exit 0, nothing committed.

`--json` `data`: `{"id":"STORY-014","commit":"a0d4f19","staged":2,"message":"STORY-014: AC-003 green","hooks":["pre-commit","commit-msg"]}`. Human output: `Commit    a0d4f19  STORY-014: AC-003 green  2 files`.

Exit codes: 0 committed; 1 on `DFA-E239`, `DFA-E271`, a hook failure; 3 on `DFA-E011`, `DFA-E012`; 5 on `DFA-E9xx`.

### `config get <key>`

```
devforgeai config get <key> [--stack <id>] [--json] [--project <path>]
```

`<key>` is drawn from the closed list `stack.test_command`, `stack.coverage_command`, `stack.lint_command`, `stack.complexity_command`, `stack.source_roots`, `stack.package_manager`, `build.worktree_root`, `build.branch_prefix`, `build.base_ref`, `build.complexity_max`, `build.duplication_max_percent`, `coverage.overall_min`, `layer.<name>.coverage_min`, `frontend.tokens_path`, `frontend.globs`, `degraded`. Any other value is `DFA-E013`, exit 3.

A `stack.*` key with no `--stack` takes the first `[[stack]]` table; `--stack <id>` selects by `id`, and an unmatched id is `DFA-E013`. The value goes to stdout with a trailing newline; an array prints one element per line; an empty string prints an empty line and exits 0.

`--json` `data`: `{"key":"stack.test_command","stack":"rust","value":"cargo test --all-features","kind":"string"}`.

Exit codes: 0 printed; 1 on `DFA-E10x`; 3 on `DFA-E011`, `DFA-E013`; 5 on `DFA-E9xx`.

### `antipattern scan`

```
devforgeai antipattern scan [--id <STORY-nnn>] [--paths <path>,...] [--min-severity <blocker|high|medium|low>] [--json] [--project <path>]
```

The candidate set is `--paths` when given, the story's `## Files` `Path` values when `--id` is given, and every file under `[[stack]].source_roots` otherwise. For each row of `.devforgeai/context/anti-patterns.md` `## Anti-pattern index` whose `Severity` is at or above `--min-severity`, default `high`, the command intersects the candidate set with the row's `Scope` glob and applies the row's `Detector` under its `Detector kind`: `literal` is a substring match over the file bytes, `regex` is a match in the `regex` crate dialect, and `glob` matches the path and reads no file.

One match is `DFA-E270`, exit 1, with one stderr line per match carrying the `AP-nnn`, the path, the line, and the matched text. No match is exit 0.

`--json` `data`: `{"scanned":21,"rules":4,"matches":[{"id":"AP-002","severity":"high","path":"src/…","line":44,"text":"…"}]}`.

Exit codes: 0 clean; 1 on `DFA-E270`, `DFA-E200`; 3 on `DFA-E013`; 5 on `DFA-E9xx`.

### `handoff`

```
devforgeai handoff [--phase <phase>] [--id <id>] [--json] [--project <path>]
```

`--phase` accepts the seven phase names, `design`, and `reflect`. It defaults to `state.toml` `[current].phase`. `--id` defaults to `[active].<phase>`, and to the `--id` argument for `design`, which has no `[active]` key. The report read is `.devforgeai/reports/<id>-<phase>.yaml`; absent, the block renders with `Gate      NOT RUN` and `Full report: none`, exit 0.

#### Rendering algorithm

1. Every line except the last is `<label padded with spaces to width 10><content>`, so content starts at character 11, per §6. The last line is the literal `Full report: ` followed by the path, and is exempt from the column rule because §6 writes it that way.
2. Widths are counted in Unicode scalar values, so the `·` separator counts as one. Line width is capped at 100. Content longer than 89 is cut at the last space at or before scalar 86, then `...` is appended, giving a line of at most 100.
3. `Phase` content is `<index> · <Name>` padded with spaces to width 20, then `<ID> · <slug>`. `index` is 0 to 6 in the §5 order and `Name` is the §5 phase name. Design has no number in that sequence, so `--phase design` renders `— · Design`, an em dash in the number's place, which keeps the separator, the column-11 start, and the twelve-line cap. `slug` is the first H1 of the phase's document, lowercased, non-alphanumerics collapsed to single hyphens, cut to 24 characters; a document with no H1 gives `-`.
4. `Done` content comes from the table below, measured by the CLI.
5. `Gate` content is `PASS`, `FAIL`, `SEND BACK to <Name>`, `TRUST FAIL`, or `NOT RUN`, then two spaces, then the evidence: for `PASS` the text `<n> checks`, where `n` counts every `[[gate.check]]` entry the gate evaluated, at either severity and including entries recorded `skip`; for `FAIL` and `SEND BACK` the first three failing check ids with their short reason, comma separated; for `TRUST FAIL` the trust error code.
6. `Verified` is omitted when the report has no `verifiers` block. With one entry the content is `<subagent> · <passed>/<total> <unit>`. With more, the entry with the lowest `passed/total` is shown and `+<k> more` is appended; a tie at the lowest ratio is broken by `config.toml` `[[verifier]]` order, the first of the tied entries winning.
7. `Found` lines are rendered for `SEND BACK` only, from `findings`, ordered by severity `block`, `warn`, `info`, then by ID ascending. Content is `<ID> <summary>`.
8. A blank line follows.
9. `Next` and the optional `Then` come from the transition table below, with IDs taken from `[active]` of the target phase, falling back to the current ID.
10. `Blocked` is `none`, or `you: <first entry of open_questions>` of the phase's document when that list is non-empty.
11. A blank line follows.
12. The `Full report:` line ends the block.

#### The twelve-line cap and the Found truncation rule

Count the fixed lines first: `Phase`, `Done`, `Gate`, the optional `Verified`, the blank, `Next`, the optional `Then`, `Blocked`, the blank, and `Full report:`. That is 8 lines at minimum and 10 with both optional lines present. The Found budget is `12 - fixed`, capped at 3 by §6. Let `n` be the number of findings.

- `n <= budget`: render `n` Found lines.
- `n > budget` and `budget >= 1`: render `budget - 1` Found lines, then one line with the label `Found` and the content `+<n - (budget - 1)> more in report`.
- `budget == 0`: drop the `Then` line, which §6 marks optional, recompute the budget, and apply the rules above.

With `Verified` and `Then` present the budget is 2, so two findings render in full and three render as one Found line plus `+2 more in report`.

#### Done counts

| Phase | `Done` content |
|---|---|
| explore | `<n> ideas · <m> flows`, counted from the IDEA and FLOW definitions in `explore/` |
| discover | `<n> REQ · <m> EPIC · <p> personas`, counted from `requirements.yaml` |
| constitute | `<n>/6 context · <m> ADR`, counted from non-empty context files and ADR files |
| plan | `<n> stories · <m> ACs`, counted from `stories/` |
| build | `<n> tests · <c>% coverage`, read from the report `gate.checks` evidence and `coverage.overall`; `tests` is `-` when the pattern did not match |
| verify | `<p>/<t> ACs · <f> findings`, read from the report `verifiers` and `findings` |
| release | `<n> stories · <version>`, counted from the release file |

#### Next and Then

| Phase | Result | `Next` | `Then` |
|---|---|---|---|
| explore | PASS | `/discover <IDEA-nnn>` | `/constitute <IDEA-nnn>` |
| explore | FAIL | `/explore <IDEA-nnn>` | omitted |
| discover | PASS | `/constitute <IDEA-nnn>` | `/plan <EPIC-nnn>` |
| discover | FAIL | `/discover <IDEA-nnn>` | omitted |
| discover | SEND BACK | `/explore <IDEA-nnn> --remedy <FLOW ids>` | `/discover <IDEA-nnn> --resume` |
| constitute | PASS | `/plan <EPIC-nnn>` | `/build <STORY-nnn>` |
| constitute | FAIL | `/constitute <IDEA-nnn>` | omitted |
| constitute | SEND BACK | `/discover <IDEA-nnn> --remedy <REQ ids>` | `/constitute <IDEA-nnn> --resume` |
| plan | PASS | `/build <STORY-nnn>` | `/verify <STORY-nnn>` |
| plan | FAIL | `/plan <SPRINT-nnn>` | omitted |
| plan | SEND BACK | `/discover <IDEA-nnn> --remedy <REQ ids>` or `/constitute <IDEA-nnn> --remedy <CON-nnn>`, by `send_back_to` | `/plan <SPRINT-nnn> --resume` |
| build | PASS | `/verify <STORY-nnn>` | `/release <vX.Y.Z>`, omitted while `[active].release` is empty |
| build | FAIL | `/build <STORY-nnn>` | omitted |
| build | SEND BACK | `/plan <EPIC-nnn> --remedy <AC ids>` | `/build <STORY-nnn> --resume` |
| verify | PASS | `/build <STORY-nnn>` for the `sprint.yaml` `stories[]` entry with the lowest `order` whose `status` is `ready`, else `/release <vX.Y.Z>` at the highest existing version with the minor incremented and the patch zeroed, `v0.1.0` when `releases/` is empty | `/verify <that STORY-nnn>`, omitted when the `Next` line is the release |
| verify | FAIL | `/verify <STORY-nnn>` | omitted |
| verify | SEND BACK | `/build <STORY-nnn> --remedy <FIND ids>` or `/plan <SPRINT-nnn> --remedy <AC ids>`, by `send_back_to` | `/verify <STORY-nnn> --resume` |
| release | PASS | `/reflect` | omitted |
| release | FAIL | `/release <vX.Y.Z>` | omitted |
| release | SEND BACK | `/verify <STORY-nnn> --remedy <FIND ids>` | `/release <vX.Y.Z> --resume` |
| design | any | the `Next` line of the phase `[current].phase` names | omitted |
| reflect | PASS | `/<command of [current].phase> <[current].id>` | `/<command of the phase after it> <its id>` |
| reflect | FAIL | `/reflect --since <window start>` | `/<command of [current].phase> <[current].id>` |
| any | NOT RUN | the current phase's own command with the active ID | omitted |
| any | TRUST FAIL | `devforgeai trust pin`, shown as a shell line rather than a slash command | omitted |

The two forms on a send-back are §4c's and no other: `--remedy <ID>,<ID>` re-opens the cited ids upstream, and `--resume` returns downstream. The upstream `<ID>` is the id that phase keys on, which the chain under `gate require` fixes.

The rendered lines are written to `state.toml` `[last_handoff]` with `rendered_at`, `phase`, and `id`.

A `/design` run prints two blocks: the skill's last workflow step runs `handoff --phase design --id <UI-nnn>`, and the Stop hook then runs `gate check --phase <[current].phase>` and `handoff` for the phase the user is in. Design advances no phase, so the second block is the phase's own. Both come from the CLI. A `/reflect` run prints two blocks the same way, Reflect's first and the current phase's second; Reflect's own block carries the current phase's command on `Next` and the command of the phase after it on `Then`, so a reader who acts on the first block alone still moves forward.

`--json` `data`: `{"lines":["Phase     4 · Build ..."],"phase":"build","id":"STORY-014","result":"PASS","next":"/verify STORY-014","then":"/release v0.3.0","blocked":"none","report":".devforgeai/reports/STORY-014-build.yaml"}`.

Exit codes: 0 in every rendering case, including `NOT RUN`; 1 on `DFA-E103`, `DFA-E401`; 3 on `DFA-E012`; 5 on `DFA-E9xx`.

### `context audit`

```
devforgeai context audit [--json] [--project <path>]
```

Eight mechanical checks, each exit 0 or 1, with one stderr line per failure naming the file, the line, and the id. The list is `specs/04-constitute.md`'s, which is the phase that owns the six files.

| Check | Rule | Code on failure |
|---|---|---|
| CA-1 | The six paths under `.devforgeai/context/` exist and parse as Markdown with the §5 frontmatter in key order | `DFA-E220` |
| CA-2 | Every context file carries `status: accepted` and `open_questions: []` | `DFA-E223` |
| CA-3 | Each file's H2 lines equal its doc-type heading list, in that order, with no extra H2 | `DFA-E213` |
| CA-4 | Every `CON-nnn` in `## Constraint index` with `status: active` appears in the `## Constraints introduced` table of at least one ADR, or its `source` field holds a `REQ-nnn` that resolves in `requirements.yaml` | `DFA-E224` |
| CA-5 | Across every table under `.devforgeai/context/` whose header row is `\| Key \| Value \| Source \|`, no key holds two distinct values, and every key is inside the closed namespace `specs/04-constitute.md` `## Outputs` fixes | `DFA-E221` |
| CA-6 | Every `AP-nnn` in `## Anti-pattern index` has a non-empty `detector`, a `detector_kind` inside its enum, and a `source` CON that exists with `status: active` | `DFA-E225` |
| CA-7 | Every ADR id is unique; every id in an ADR's `consumes` resolves in `requirements.yaml`; every CON in an ADR's `## Constraints introduced` exists in `## Constraint index` | `DFA-E226` |
| CA-8 | Every ADR named in a `## Supersedes` row has `status: superseded`, and every CON that row retires has `status: retired` | `DFA-E227` |

CA-2 places user acceptance before the first commit: the `pre-commit` hook runs `context audit`, so a brownfield project commits its context files once they leave `status: draft`. CA-5 is the pairwise contradiction check §4 names. It is textual: keys and values are compared after trimming, collapsing runs of whitespace, and lowercasing, against accepted ADRs only. It detects no semantic contradiction, which is deliberate, since §1 keeps judgement in skills.

A duplicated `CON-nnn` across two files is `DFA-E222`.

`--json` `data`:

```json
{ "files": [ { "name": "tech-stack", "present": true, "valid": true, "status": "accepted", "keys": 14 } ],
  "constraints": 9, "anti_patterns": 4, "adrs": 3,
  "checks": [ { "id": "CA-5", "status": "fail", "count": 1 } ],
  "findings": [ { "check": "CA-5", "code": "DFA-E221", "key": "database",
                  "a": { "path": ".devforgeai/context/tech-stack.md", "line": 12, "value": "postgres 16" },
                  "b": { "path": ".devforgeai/adr/ADR-004.md", "line": 22, "value": "sqlite" } } ] }
```

Human output: `context audit  6/6 files · 9 constraints · 4 anti-patterns · CA-5 failed` then one line per finding on stderr.

Exit codes: 0 when all eight checks pass; 1 on any `DFA-E22x`, `DFA-E213`, or any `DFA-E2xx` from the frontmatter grammar; 5 on `DFA-E9xx`.

### `story validate`

```
devforgeai story validate [<id>] [--scope <active|sprint|all>] [--json] [--project <path>]
```

`<id>` is a `STORY-<nnn>`, optional; giving it restricts the run to that story and ignores `--scope`. `--scope` defaults to `active`, meaning the story in `state.toml` `[active].build`; `sprint` means every story listed in `stories/sprint.yaml`; `all` means every file matching `stories/STORY-<nnn>.md`.

Checks per story:

1. The frontmatter grammar for doc type `story`.
2. At least one AC, else `DFA-E234`. An AC is a list item under the H2 `## Acceptance Criteria` matching `^- (?P<id>AC-[0-9]{3}): (?P<text>.+)$`.
3. Each AC text is testable: it matches `Given .+ When .+ Then .+`, or it contains one of the verbs `returns`, `rejects`, `renders`, `persists`, `emits`, `exits`, `responds` with text on both sides. Otherwise `DFA-E230`.
4. Each `consumes` entry with the `REQ` prefix exists in `requirements.yaml`, else `DFA-E231`.
5. Dependencies are the `STORY-<nnn>` items listed under the H2 `## Dependencies`. A cycle over that graph is `DFA-E232`, reported as the cycle path.
6. For `--scope sprint`, every `stories[].id` in `sprint.yaml` has a file, else `DFA-E233`. `sprint.yaml` `stories` is a sequence of mappings with `id` (STORY form) and `status` (one of the story status enum values).

7. For `--scope sprint`, every `requirements[].id` the epic named by `sprint.yaml` `epic` lists appears in the first column of the `## Requirements` table of some story in the union of `stories[].id` and `deferred[].id`, else `DFA-E235`.
8. Each `AC-nnn` appears in exactly one `Covered by` cell of its own story's `## Requirements` table: none is `DFA-E236`, more than one is `DFA-E245`.
9. For `--scope sprint`, no `Path` value appears in the `## Files` table of two stories listed under `stories[]`, which is the concurrent set and excludes `deferred[]`, else `DFA-E237`.
10. Every `UI-nnn` in a story's `consumes` resolves to `.devforgeai/ui-specs/<UI-nnn>.md`, else `DFA-E238`.

`--json` `data`: `{"scope":"sprint","stories":[{"id":"STORY-014","valid":true,"acs":5,"deps":["STORY-011"],"errors":[]}],"checked":6,"failed":0}`.

Human output: one line per story, `ok  STORY-014  5 ACs  1 dependency`.

Exit codes: 0 when every story passes; 1 on any `DFA-E23x` or `DFA-E2xx`; 3 on `DFA-E012`, `DFA-E013`; 5 on `DFA-E9xx`.

### `design lint`

```
devforgeai design lint [<paths>...] [--tokens] [--json] [--project <path>]
```

With no path the file set is every file matching `config.toml` `[frontend].globs` minus `[frontend].exclude`. With paths, only those files, filtered by the same globs; a path outside the globs is skipped and counted in `data.skipped`. Two prefixes are skipped in every run and exit 0: `.explore-prototype/` and `.devforgeai/explore/mockups/`, which hold throwaway sketches drawn before a brand kit exists. Both are in the default `[frontend].exclude` list.

The token file is `config.toml` `[frontend].tokens_path`, default `.devforgeai/brand/tokens.json`. Its top-level keys are `meta` plus the six groups `color`, `type`, `spacing`, `radius`, `elevation`, and `motion`. `meta` carries the §5 frontmatter and is not a group. A leaf of `color` is an object with exactly the keys `light` and `dark`, each a string; a leaf of every other group is a string. The flattened token name is `TOKEN-<group>-<leaf>` in every group, one name for a colour with two values, and the CSS custom property is `--<group>-<leaf>`. Leaf names match `^[a-z][a-z0-9-]*$`.

A literal is resolved against the union of the `light` and `dark` values, and `DFA-E240` names the token rather than the theme.

`--tokens` reads no frontend file. It checks that the token file parses, that its top-level keys are `meta` plus the six groups, that `meta` holds the seven §5 keys, that every leaf name matches the pattern above, that `color` leaves are `{light, dark}` objects and other groups' leaves are strings, and that every substring matching `TOKEN-[a-z0-9-]+` in `.devforgeai/ui-specs/UI-*.md` flattens to a defined token. A group or leaf of the wrong shape is `DFA-E243`; an undefined token in a UI spec is `DFA-E244`. `--tokens` with a path argument is `DFA-E010`.

Violations:

| Rule | Pattern | Allowed instead |
|---|---|---|
| literal colour | `#[0-9a-fA-F]{3,8}\b`, `rgba?\(`, `hsla?\(`, or one of the 148 CSS named colours | `var(--<token>)`, `token(<name>)`, `transparent`, `currentColor`, `inherit`, `none` |
| literal font size | `font-size\s*:\s*[0-9.]+(px\|rem\|em\|pt)` | `var(--<token>)` |
| literal font family | `font-family\s*:\s*[^v]` | `var(--<token>)` |
| literal line height | `line-height\s*:\s*[0-9.]+` | `var(--<token>)`, `normal` |
| unknown token | `var\(--(?P<name>[a-z0-9-]+)\)` whose name has no token | a defined token |

The nearest token named in `DFA-E240` is the token with the smallest Euclidean distance in sRGB after converting both values to RGB; for `DFA-E241` it is the numerically nearest token value after converting to pixels at 16px per rem. A file with no token of the matching kind reports `no token defined` in place of the name.

`--json` `data`: `{"mode":"paths","files":12,"skipped":2,"violations":[{"path":"src/ui/Button.tsx","line":22,"code":"DFA-E240","value":"#3355ff","nearest":"TOKEN-color-primary"}]}`. With `--tokens`, `mode` is `tokens`, `files` counts the UI specs scanned, and `violations` carries `DFA-E243` and `DFA-E244` entries.

Human output: one line per violation, then `design lint  12 files · 1 violation`.

Exit codes: 0 with no violation; 1 on any violation, `DFA-E120`, `DFA-E121`, `DFA-E243`, `DFA-E244`; 3 on `DFA-E010`; 5 on `DFA-E9xx`. On PreToolUse the hook wrapper maps exit 1 to hook exit 2, blocking the write.

### `hook install`

```
devforgeai hook install [--force] [--claude-only] [--git-only] [--json] [--project <path>]
```

`--force` replaces a git hook this binary did not write. `--claude-only` and `--git-only` are mutually exclusive; giving both is `DFA-E010`.

Settings merge: read `.claude/settings.json`, creating `{}` when absent; parse as JSON, a failure being `DFA-E131`; resolve the two per-project tokens in the compiled-in block, `@@TEST_COMMAND@@` from the `test_command` of each `[[stack]]` and `@@VERIFIERS@@` from the `[[verifier]]` names, dropping a handler whose token resolves to nothing rather than writing a filter that never fires; for each event key in the block in `## Templates`, append each matcher entry to the existing array when no entry with an identical identity is present, the identity being the `command` followed by its `args` vector, leaving every other key and array element untouched; an entry already present is `DFA-W130`; then, inside each matcher entry, drop any handler byte-for-byte identical to an earlier one in the same `hooks` array, keeping the first; write the result with two-space indentation, preserving key order and appending new keys at the end; copy the previous file to `.claude/settings.json.bak-<RFC 3339 basic timestamp>` before writing.

The second de-duplication is not the same rule as the first. The first compares whole matcher entries across an event's array by their handler identity. The second is what makes dropping an unresolvable permission rule safe: two handlers that differed only by a `Bash(...)` and a `PowerShell(...)` rule become the same object once the rule is gone, and leaving both would spawn the process twice for one tool call. Whatever is still distinct after the drop stays.

Permission merge: the same write adds `Bash(devforgeai *)` and `PowerShell(devforgeai *)` to `permissions.allow`, creating `permissions` and `allow` when absent and leaving a rule already present alone. A skill's `allowed-tools` grant covers the invoking turn and clears on the next user message, while a phase spans many turns that each call the binary; without a standing rule the second turn of every phase stops for a permission prompt on a command the framework itself installed. Both spellings are registered because the PowerShell tool is primary on Windows and the Bash tool elsewhere. The rules added are reported in `data.permissions`.

Git hooks: the target is `<git-common-dir>/hooks/<name>`, read from `git rev-parse --git-common-dir` so worktrees resolve, falling back to `.git/hooks` when git is absent. Each script's first line after the shebang is the marker `# devforgeai-hook v1`; a file without that marker is `DFA-E132` unless `--force`. Files are written with LF endings and mode `0755` on Unix; on Windows the mode call is skipped, since Git Bash executes the hook through `sh`.

`--json` `data`: `{"settings":"merged","events":["SessionStart","UserPromptExpansion","PreToolUse","PostToolUse","Stop","SubagentStop"],"git_hooks":["pre-commit","commit-msg","pre-push"],"backup":".claude/settings.json.bak-20260910T140211Z","permissions":["Bash(devforgeai *)","PowerShell(devforgeai *)"]}`.

Human output: `Hooks  .claude/settings.json merged (6 events); git hooks pre-commit, commit-msg, pre-push`.

Exit codes: 0 on success and on `DFA-E130`; 1 on `DFA-E131`, `DFA-E132`; 3 on `DFA-E010`; 5 on `DFA-E9xx`.

### `hook run <event>`

```
devforgeai hook run <event> [--json] [--project <path>]
```

`<event>` is the closed enum `session-start`, `prompt-expansion`, `pre-tool-use`, `trust-check`, `post-tool-use`, `stop`, `subagent-stop`. The hook JSON arrives on stdin. This subcommand is an addition to conventions §4 and is recorded under `## Decisions`; it exists so one registration works identically under every shell, with no JSON parser in the shell. A settings command string never carries `--json`: the dispatcher chooses its own output shape per event, and a `--json` envelope on stdout would be read by the harness as a hook decision object.

Each event runs `trust verify` first, per §8. The events that can block are `prompt-expansion`, `pre-tool-use`, `trust-check`, `stop`, and `subagent-stop`; on those a trust failure exits 2 carrying a decision object, and on `session-start` and `post-tool-use` it exits 4 with a `systemMessage` carrying the `DFA-E5xx` text, because no exit code blocks there. The `devforgeai trust verify` subcommand itself exits 4 in every failing case; the mapping to 2 belongs to the dispatcher.

| `<event>` | Reads from stdin | Runs | Blocks |
|---|---|---|---|
| `session-start` | `cwd` | `stack detect`, then `handoff` | nothing; exit 0, the block on plain stdout, which this event adds to Claude's context |
| `prompt-expansion` | the matched command name, the prompt's first argument | `trust verify`, then `gate require <phase> <id>` with the phase taken from the command name and the id from the argument | exit 2 with `{"decision":"block","reason":...}` on a trust failure or a predecessor gate that has not passed; exit 0 and empty stdout otherwise |
| `pre-tool-use` | `tool_name`, `tool_input.file_path` or `tool_input.notebook_path`, `tool_input.content` for Write only, `tool_input.command` for the shell tools, `agent_type` | write tools: `doc validate --producer-check <path>` when the path is under `.devforgeai/`, with `--stdin-content` for Write and without it for Edit; `design lint <path>` when the path matches the frontend globs; `story files --check <path>` when the path is outside `.devforgeai/` and `[current].phase` is `build`. Shell tools: `doc validate --producer-check` on every path the command appears to write under `.devforgeai/`, and the metrics-command decision below | exit 2 with `hookSpecificOutput.permissionDecision` of `deny` on any of them failing |
| `trust-check` | nothing | `trust verify` and nothing else: no `config.toml` read, no `state.toml` read, no path resolution, so it adds one hash to the write path and can fail for no other reason | exit 2 with `permissionDecision: deny` naming the pin command on a trust failure; exit 0 and empty stdout otherwise |
| `post-tool-use` | `tool_name`, `tool_input.file_path` or `tool_input.notebook_path`, `tool_input.command` | `doc validate <path>` for the write tools under `.devforgeai/`; `gate check --phase build --partial` for the shell tools when the command string equals a `test_command` in `config.toml` after trimming | nothing; exit 0, every diagnostic in `hookSpecificOutput.additionalContext` |
| `stop` | `stop_hook_active`, `session_id` | `gate check --phase <[current].phase>`, then `handoff` | exit 2 on FAIL inside the block budget below, one object carrying `decision`, `reason` and `systemMessage` |
| `subagent-stop` | `agent_type`, `last_assistant_message`, `agent_transcript_path` | `report ingest <agent_type> -` when the name is in `config.toml` `[[verifier]]`, fed the `last_assistant_message` text and falling back to the last assistant message of `agent_transcript_path` | exit 2 with `{"decision":"block","reason":...}` when the envelope does not parse, so the subagent is asked for it again; exit 0 otherwise |

**The metrics-command arm.** Two read-only analysis agents run one configured command apiece and hold no other shell grant: `code-quality-auditor` runs `config.toml` `[verify].metrics_command`, `dead-code-detector` runs `[verify].call_graph_command`. When the payload's `agent_type` is one of those two, the dispatcher compares the trimmed `tool_input.command` with the trimmed configured value. Equal: `permissionDecision: "allow"` with a reason naming the key, which is what lets the one command through. Unequal or the key empty: `permissionDecision: "deny"` naming the `config.toml` key to edit, so the refusal is actionable by whoever owns the project's tooling. Any other `agent_type` falls through to the ordinary shell scan. The project decides what those agents may run, which is why the permission lives in `config.toml` rather than in the agent's tool grant.

**The `trust pin` refusal.** The shell arm denies any `Bash` or `PowerShell` command whose text contains `trust pin` while a Claude session is active. `trust pin` already refuses to run when `CLAUDECODE` or `CLAUDE_CODE_ENTRYPOINT` is set, and that guard is bypassable from inside a shell by stripping the variable before the call; a `PreToolUse` deny is not, because it fires before the process starts and in every permission mode. The pin is a human action taken in a terminal outside Claude Code, and this is the surface that makes it one.

**A path outside the project root.** A write whose path does not resolve under the root is refused in two cases and allowed otherwise: a path under the trust store (`~/.devforgeai/`) or under the `framework_path` of a pin in `trust.toml`, because that is how an unpinned binary would be made to look pinned; and any path during a Build run whose `[active].build` story declares a file set, because a path outside the project is outside the declared set by construction. Every other outside path is allowed with no output — refusing them all would refuse the session scratchpad, and no arm of this hook has a rule that speaks to them.

**The Stop-time document scan.** Before the gate runs, the dispatcher scans the documents written since the last scan and runs the producer check on each. The window opens at `state.toml` `[stop_hook].scanned_at`; with that empty it opens at `[last_gate].at`, the nearest turn boundary state carries; with neither, this Stop establishes the baseline and reads nothing, because every document in a freshly initialised project would otherwise look new. The window closes at the end of the recorded second, not at its start: an RFC 3339 stamp carries whole seconds while a filesystem modification time carries more, so a write in the same second as the last scan would otherwise be read twice — once by the scan that recorded the stamp and once by the next.

The scan walks `.devforgeai/explore`, `context`, `adr`, `stories`, `ui-specs`, `brand`, and `releases` for files with the extensions `md`, `yaml`, and `json` whose modification time is later than the window's start, and runs `doc validate --producer-check` on each. `reports/` is excluded because the CLI writes it during the very Stop that would then flag it; `state.toml`, `config.toml`, and `gates.toml` are excluded for the same reason; `.allocated/` holds reservation markers, which are not documents. What is left is exactly the set a skill authors and a producer owns.

Every refusal is folded into the FAIL reason as `DFA-E212 <path>: <message>` and carried in the envelope's `warnings[]`, so the `--json` reader sees it as a diagnostic rather than only as a string inside `reason`. The gate result becomes `FAIL` whatever the gate itself said, so a document written from the wrong phase is refused rather than argued with. `scanned_at` is then advanced to the run's timestamp, whether or not anything was found.

This is the enforcing half of the shell-write guard. The `PreToolUse` write-tool arm sees a `Write`, an `Edit`, and a `NotebookEdit`; it does not see a shell redirection, a heredoc, or a `cat >`, so the producer gate was one `>` away from being bypassed. The scan reaches every document in that set whatever wrote it, and the path is carried in the reason because the model has to know which file it wrote from the wrong phase rather than only that one exists.

**Stop block budget.** The budget is three blocks per session, keyed on `session_id` alone and recorded in `state.toml` `[stop_hook]`. A Stop whose `session_id` differs from `blocked_session` starts a fresh budget; nothing else resets it, and in particular a changed phase or id does not. That is the whole point of the rule: the subject is what a continuation turn can change, so a budget keyed on it lets Claude advance to the next story and block again without end, which is the loop the harness's eight-block ceiling exists to stop. `stop_hook_active` is not consulted for the budget either — a fresh turn that keeps failing is the same unresolved gate. A FAIL with `block_count` below the budget increments it, emits one object holding `decision: "block"`, a `reason` naming the failing checks, and `systemMessage` holding the handoff block, and exits 2; the same `reason` also goes to stderr, so a schema change upstream degrades to the stderr path rather than to silence. A FAIL with the budget spent exits 0 with the FAIL handoff in `systemMessage`, which is what puts the block in front of the user. A PASS resets `block_count` to 0. The gate runs on the continuation as well as on the first Stop: Claude has been working on the failing checks since the last block, and re-running is the only way to notice that the work now passes. The harness's own ceiling is eight consecutive blocks, at which it overrides the hook and shows nothing; the framework's budget ends the chain first.

**Cross-cutting blocks.** When `state.toml` `[last_cross]` holds a phase, the dispatcher runs `handoff --phase <last_cross.phase> --id <last_cross.id>` first, then `handoff --phase <[current].phase>`, joins the two with a blank line into one `systemMessage`, clears `[last_cross]`, and caps the joined string at 10,000 characters.

`--json` `data`: `{"event":"stop","actions":["gate check","handoff"],"blocked":true,"exit":2,"block_count":1}`.

Human output: the output of the called subcommands, in call order, unmodified. On `stop` the human output is suppressed whenever the JSON object is set, so stdout holds one object and nothing else.

Exit codes: 0 non-blocking; 2 blocking; 3 on `DFA-E020`, `DFA-E021`, `DFA-E012`; 4 trust failure on `session-start` or `post-tool-use`; 5 on `DFA-E9xx`.

### `trust pin`

```
devforgeai trust pin [--binary <path>] [--framework <path>] [--json]
```

`--binary` defaults to `std::env::current_exe()` canonicalised; an error there is `DFA-E511`. `--framework` defaults to the nearest ancestor of the current working directory containing a `cli/` directory with both `REVISION` and `DIGEST`; absent, `DFA-E510`. `--project` is accepted and ignored, since the pin is user-wide.

Flow:

1. Read `CLAUDECODE` and `CLAUDE_CODE_ENTRYPOINT` with `std::env::var_os`. If either is present and its value is non-empty after trimming ASCII whitespace, print `DFA-E500` naming the variable and exit 4, writing nothing. Presence with an empty or whitespace-only value is treated as absent.
2. Read `<framework>/cli/REVISION`. Line 1 is the git SHA-1 of the framework repo at release, or the literal `unversioned`. Line 2 is `sha256:` followed by 64 lowercase hex characters. Any other shape is `DFA-E510`.
3. Read `<framework>/cli/DIGEST`: one line, `sha256:` followed by 64 lowercase hex characters. Any other shape is `DFA-E510`.
4. Compute the SHA-256 of the bytes of `--binary`.
5. Compare it with `cli/DIGEST`. A mismatch is `DFA-E503`, exit 4, nothing written.
6. Load `~/.devforgeai/trust.toml`, creating the directory and an empty file when absent; a parse failure is `DFA-E512`.
7. Replace the `[[pin]]` whose `binary_path` equals the canonical binary path, or append a new one, filling every field in the `## Outputs` schema.
8. Write atomically and print the pin.

`~` resolves to `%USERPROFILE%` on Windows and `$HOME` elsewhere. Under the `test-home` cargo feature, `DEVFORGEAI_HOME` replaces that directory; with the feature off the variable is read by nothing, so a released binary cannot be redirected. `cli/DIGEST` is computed over the release binary built with default features, so a binary compiled with `test-home` has a different digest and fails `trust verify` by construction.

**Producing `cli/REVISION` and `cli/DIGEST` at release.** One hidden subcommand writes both, and no script reimplements the walk:

```
devforgeai trust digest --framework <root> [--binary <path>]
```

It prints three labelled lines — `REVISION`, `SOURCE`, `DIGEST` — which are `REVISION` line 1, `REVISION` line 2, and the whole of `DIGEST`. `--binary` defaults to the running executable. `--framework` defaults to the current working directory, and not to the nearest ancestor holding `cli/REVISION` and `cli/DIGEST` the way `trust pin` resolves the same flag: this subcommand writes those two files rather than reading them, so it cannot search for a directory by their presence. A release step therefore passes `--framework` or runs from the framework root. The subcommand is hidden because it is a build tool rather than part of the §4 surface, and it exists so that the two files are written by the same code path `trust verify` later reads them with: an external script reimplementing the walk drifts the first time an exclusion changes, and the drift surfaces only as `DFA-E504` refusing every write on a developer's machine.

`REVISION` line 1 is the output of `git rev-parse HEAD` in the framework root — forty lowercase hex characters — when that root is a git work tree, and the literal `unversioned` when it is not. Line 2 is the source digest: the SHA-256 over the concatenation, for every file under `cli/` in byte-wise ascending path order, of the path in forward-slash form, a `0x00` byte, the file length as 8 big-endian bytes, and the file bytes. The walk excludes the depth-1 directories `target` and `.git` and the two files `cli/REVISION` and `cli/DIGEST` — neither release file is part of the digest it records, because a file cannot contain its own digest. `DIGEST` is the SHA-256 of the binary produced by `cargo build --release --locked` with default features, taken from the artifact that ships. Both files are committed in the same release commit that builds the binary. The test `the_walk_digest_matches_the_shipped_revision` recomputes line 2 over the tree and compares it with the committed `cli/REVISION`, so a change to the walk that is not reflected in the shipped file fails the suite rather than every `trust verify` on a developer's machine.

`--json` `data`: `{"binary":"C:\\Users\\bryan\\.cargo\\bin\\devforgeai.exe","digest":"sha256:...","revision":"a1b2c3...","source_digest":"sha256:...","framework_path":"C:\\Projects\\DevForgeAI","trust_file":"C:\\Users\\bryan\\.devforgeai\\trust.toml","replaced":false}`.

Human output:

```
Pinned    C:\Users\bryan\.cargo\bin\devforgeai.exe
Digest    sha256:7f0c1a...
Revision  a1b2c3d4e5f6
Trust     C:\Users\bryan\.devforgeai\trust.toml
```

Exit codes: 0 pinned; 4 on `DFA-E500`, `DFA-E503`, `DFA-E510`, `DFA-E511`, `DFA-E512`; 3 on `DFA-E010`; 5 on `DFA-E9xx`.

### `trust verify`

```
devforgeai trust verify [--binary <path>] [--json]
```

Flow:

1. Resolve the binary path as in `trust pin`; failure is `DFA-E511`.
2. Load `~/.devforgeai/trust.toml`; absent is `DFA-E501`, unparsable is `DFA-E512`.
3. Select the `[[pin]]` whose `binary_path` equals the canonical path; none is `DFA-E502`.
4. Compute the SHA-256 of the binary and compare it with `pin.digest`; a mismatch is `DFA-E503`.
5. Read `CLAUDECODE` and `CLAUDE_CODE_ENTRYPOINT` by the same rule as `trust pin`. When a session is active and `pin.framework_path` is non-empty and that directory still holds `cli/`, recompute the source digest by the algorithm above and compare it with `pin.source_digest`; a difference is `DFA-E504`.
6. When no session is active, step 5 is skipped, so a developer may edit `cli/` freely outside Claude.
7. A pin whose `pinned_at` precedes the binary's modification time with matching digests is `DFA-W500`, exit 0.

The subcommand exits 0 or 4 and writes nothing. Behaviour per hook on failure:

| Hook | On trust failure |
|---|---|
| SessionStart | exit 4; the session starts; `stack detect` and `handoff` do not run; the code travels in `systemMessage`, because no exit code blocks here |
| UserPromptExpansion | exit 2 with `{"decision":"block","reason":...}`; the skill body never enters context, so a phase cannot start on an unpinned binary |
| PreToolUse, both the `pre-tool-use` and the `trust-check` arms | exit 2 with `hookSpecificOutput.permissionDecision` of `deny`; the deny fires in every permission mode including `bypassPermissions`, and no other hook's allow overrides it. The `trust-check` matcher covers `Write`, `Edit`, `NotebookEdit`, `Bash`, `PowerShell`, and `Agent`, so the session can read and reason and can change nothing |
| PostToolUse | exit 4; the tool result stands; the code travels in `systemMessage` |
| Stop | exit 2 with `decision: "block"` naming the pin command, and no gate runs for the turn; with `stop_hook_active` true it exits 0 carrying the `systemMessage` alone, so the harness's continuation ends and the user is left holding the refusal rather than a loop. `state.toml` `[last_gate].result` takes `TRUST_FAIL` in both branches, which is a record rather than the channel the refusal depends on — the binary under suspicion is the one that renders the handoff |
| SubagentStop | exit 2 with `{"decision":"block","reason":...}`; nothing is ingested |
| pre-commit, commit-msg, pre-push | exit 4; git aborts the operation |

The message is one text on all three blocking surfaces: `devforgeai trust verify failed: <code> <diagnostic>. In a terminal outside Claude Code, run: devforgeai trust pin --framework <framework root>. Then start a new session.`

`--json` `data`: `{"binary":"...","digest":"sha256:...","pinned":"sha256:...","match":true,"session_active":true,"source_digest_checked":true}`.

Human output on success with `--quiet` unset: `trust verify  ok  sha256:7f0c1a...`. On failure, nothing on stdout.

Exit codes: 0 verified; 4 on `DFA-E501` through `DFA-E512`; 5 on `DFA-E9xx`.

### `phase set <phase> --id <id>`

```
devforgeai phase set <phase> --id <id> [--epic <EPIC-nnn>] [--remedy <ID,ID>] [--json] [--project <path>]
```

`<phase>` and `--id` are required. `--remedy` takes a comma-separated ID list, default empty, and is the §4c send-back form; it writes the receiving phase's remedy fields:

| `<phase>` | `--remedy` accepts | Fields written |
|---|---|---|
| `explore` | `FLOW-nnn` ids | `[explore].remedy_flows`, `[explore].remedy_started_at` set to now, `[explore].remedy_timebox_days` from `config.toml`; `[explore].started_at` unchanged |
| `constitute` | one `CON-nnn` | `[constitute].remedy_con` |
| every other phase | nothing; a non-empty `--remedy` is `DFA-E011` | — |

A `phase set explore --id <IDEA-nnn>` with no `--remedy` writes `[explore].idea_id`, `[explore].started_at` set to now, `[explore].timebox_days` from `config.toml`, and `[explore].remedy_flows = []`.

`--epic` belongs to the `plan` arm alone and names the `EPIC-nnn` the sprint realizes. `[active].plan` still holds the `SPRINT-nnn`, which is the subject the phase advances; the epic goes to `[plan].epic`, because Plan is entered before its own document exists and the gate has to resolve against something in the meantime. The flag is required whenever `.devforgeai/stories/sprint.yaml` is absent, which is every full Plan run — the file is written at the end of the phase — and every resume that follows a send-back, since the sprint the run would resolve its epic from does not yet exist. With the file present the flag repeats the file's own `epic` key.

Two codes, dividing by what went wrong rather than by which check caught it. `DFA-E011`, the flag error, covers `--epic` absent with no `sprint.yaml` on disk and `--epic` given on any phase but `plan`. `DFA-E013`, the malformed-ID error, covers an epic that is not an `EPIC-nnn` and an epic that contradicts the `epic` key `sprint.yaml` already records: in both cases the value the caller passed is not the id the run is for. Nothing is written in either case.

The command runs the `gate require <phase> <id>` logic first. It refuses only when a predecessor gate exists for the phase and its result is not `PASS`: that is `DFA-E320`, exit 1, and nothing is written. An entry phase with no predecessor, which is `explore` in every project and `discover` when no Explore ran, sets the phase and exits 0. On success it sets `[current].phase` and `[current].id`, sets `[active].<phase>` to `<id>`, sets `updated_at`, sets `[stop_hook].block_count` to `0`, and clears `blocked_phase` and `blocked_id`. Other `[active]` keys keep their values.

`phase set` is also the one writer of a story's `status`. A story moves through `ready`, `building`, `built`, `released` inside phases that do not produce the story document, so no skill can advance it: the Plan skill owns `stories/STORY-nnn.md` and the producer check blocks a Build-phase write to it. The transition table:

| `phase set <phase>` | Document written | New `status` |
|---|---|---|
| `build` | `stories/<id>.md` | `building` |
| `verify` | `stories/<id>.md` | `built` |
| `release` | every `STORY-nnn` listed under `stories` in `releases/<id>.yaml` | `released` |
| `explore`, `discover`, `constitute`, `plan` | none | — |

The write replaces the `status` value in the frontmatter and touches no other byte of the file. A document already carrying the target value is left untouched. An absent document is `DFA-W210`, exit 0, with `state.toml` still advanced. The status values of every other document type are written by the skill that produces it, since each of those transitions happens inside the producing phase.

`--json` `data`: `{"from":"plan","to":"build","id":"STORY-014","required":"plan","required_result":"PASS","remedy":[],"status_writes":[{"path":".devforgeai/stories/STORY-014.md","status":"building"}]}`.

Human output: `Phase     4 · Build        STORY-014`.

Exit codes: 0 set; 1 on `DFA-E320`, `DFA-E300`, `DFA-E10x`; 3 on `DFA-E011`, `DFA-E012`, `DFA-E013`; 5 on `DFA-E9xx`.

### `report show <id> <phase>`

```
devforgeai report show <id> <phase> [--check <check-id>] [--json] [--project <path>]
```

Prints `.devforgeai/reports/<id>-<phase>.yaml`. `<phase>` accepts the seven phase names, `design`, and `reflect`. The reflect gate report is `reports/<YYYY-MM-DD>-reflect.yaml` and the reflect document is `reports/reflect-<YYYY-MM-DD>.yaml`; the two differ by which side of the date the word sits on. With `--check <check-id>` only that check's entry is printed. An absent file is `DFA-E400`; unparsable is `DFA-E401`.

`--json` `data` is the parsed report object, or the single check entry with `--check`.

Human output for the whole report is the `gate check` human block plus the `coverage` and `verifiers` summaries.

Exit codes: 0 printed; 1 on `DFA-E400`, `DFA-E401`; 3 on `DFA-E012`, `DFA-E013`.

### `report ingest <subagent> <source>`

```
devforgeai report ingest <subagent> <source> [--id <id>] [--phase <phase>] [--json] [--project <path>]
```

This subcommand is an addition to conventions §4 and is recorded under `## Decisions`; §7 names the call in the SubagentStop row without listing it in the §4 table.

`<subagent>` is the kebab-case agent name. `<source>` is a file path, or `-` for stdin. `--id` defaults to `state.toml` `[active].<phase>`, and `--phase` defaults to the `phase` of the matching `[[verifier]]` table. A `[[verifier]]` whose `phase` is `design` writes into `reports/<UI-nnn>-design.yaml`, which no gate reads and `handoff --phase design` prints.

Flow: look the name up in `config.toml` `[[verifier]]`; a miss is `DFA-W411`, exit 0, nothing written. Read the source and parse it as the one JSON object of `## Subagents`; a failure, a `schema` other than `devforgeai/verifier/1`, or a finding whose `severity` is outside `block | warn | info` is `DFA-E410`, and the block is written with `passed: 0`, `total: 0`, and `status: unparsed`. `report ingest` itself exits 0 in every case; the `subagent-stop` hook arm recognises the `DFA-E410` warning and exits 2 with a blocking decision, so the subagent is asked for the envelope again rather than the gate evaluating `verifier_pass` against a block of zeros. Resolve the target report `.devforgeai/reports/<id>-<phase>.yaml`, creating it with the `## Outputs` skeleton when absent. Write the parsed block at `verifiers.<report_field>`, replacing any previous block from the same subagent. Append the parsed `findings` to the report's `findings` list, de-duplicated by `id`, keeping the newest entry per ID. An absent active ID is `DFA-E412`, exit 0.

The Verify phase has two report files, and they are distinct on purpose. `reports/<id>-qa.yaml` is the Verify skill's own document, listed in conventions §5, holding its findings and prose. `reports/<id>-verify.yaml` is the CLI's gate report for the verify phase, which is where `report ingest` writes the verifier block, where `gate check --phase verify` records its checks, and which the handoff `Full report:` line cites. The verify gate validates the first with `doc_valid` and reads the second for `verifier_pass`.

`--json` `data`: `{"subagent":"ac-compliance-verifier","report":".devforgeai/reports/STORY-014-verify.yaml","field":"verifiers.ac_compliance","passed":7,"total":7,"findings":0,"status":"ingested"}`.

Human output: `Ingested  ac-compliance-verifier · 7/7 ACs -> .devforgeai/reports/STORY-014-verify.yaml`.

Exit codes: 0 in every case, so SubagentStop does not block, per §7; 3 on `DFA-E012`; 5 on `DFA-E9xx`.

## Gate

The CLI owns no phase, so it has no gate of its own. It owns the file every gate is read from. This is the default `.devforgeai/gates.toml` that `init` writes, verbatim. Each `[[gate]]` block is the one its owning skill spec publishes: `specs/02-explore.md`, `specs/03-discover.md`, `specs/04-constitute.md`, `specs/05-plan.md`, `specs/06-build.md`, `specs/07-verify.md`, `specs/09-release.md`, and `specs/10-reflect.md`. Design holds no entry: `specs/08-design.md` `## Gate` states it, §5 gives one gate per phase, and Design is not a phase, so `gate check --phase design` and `gate require design <id>` exit 1 with `DFA-E300`. The file satisfies the compiled minimums exactly, with no margin, so a project can raise a threshold and cannot lower one below these values.

```toml
schema = "devforgeai/gates/1"
cli_min_version = "1.0.0"

[[gate]]
phase = "explore"
requires = ""
on_fail = "fail"
send_back_to = ""
description = "A decision is recorded, the brief carries three to five flows, the time box holds."

  [[gate.check]]
  kind = "file_exists"
  id = "decision-exists"
  severity = "block"
  on_fail = "fail"
  paths = [".devforgeai/explore/decision.yaml"]
  min_count = 1
  message = "no decision recorded for {id}"

  [[gate.check]]
  kind = "field_in_enum"
  id = "decision-enum"
  severity = "block"
  on_fail = "fail"
  path = "explore/decision.yaml"
  field = "decision"
  values = ["kill", "park", "promote"]
  message = "decision is {value}, expected kill, park or promote"

  [[gate.check]]
  kind = "field_is_date"
  id = "decision-dated"
  severity = "block"
  on_fail = "fail"
  path = "explore/decision.yaml"
  field = "decided_on"
  format = "%Y-%m-%d"
  message = "decided_on is {value}, expected YYYY-MM-DD"

  [[gate.check]]
  kind = "field_is_date"
  id = "park-has-revisit"
  severity = "block"
  on_fail = "fail"
  path = "explore/decision.yaml"
  field = "revisit_on"
  format = "%Y-%m-%d"
  after_field = "decided_on"
  required_when = { path = "explore/decision.yaml", field = "decision", equals = "park" }
  null_when = { path = "explore/decision.yaml", field = "decision", in = ["kill", "promote"] }
  message = "park decision needs revisit_on later than decided_on"

  [[gate.check]]
  kind = "length_between"
  id = "promote-carries-forward"
  severity = "block"
  on_fail = "fail"
  path = "explore/decision.yaml"
  field = "carry_forward"
  min = 7
  max = 7
  required_when = { path = "explore/decision.yaml", field = "decision", equals = "promote" }
  empty_when = { path = "explore/decision.yaml", field = "decision", in = ["kill", "park"] }
  message = "promote decision carries {value} artifacts, expected 7"

  [[gate.check]]
  kind = "row_count_between"
  id = "flow-count"
  severity = "block"
  on_fail = "fail"
  path = "explore/brief.md"
  section = "Core flows"
  min = 3
  max = 5
  message = "{value} core flows, expected 3 to 5"

  [[gate.check]]
  kind = "column_matches"
  id = "flow-id-shape"
  severity = "block"
  on_fail = "fail"
  path = "explore/brief.md"
  section = "Core flows"
  column = "ID"
  pattern = "^FLOW-[0-9]{3}$"
  unique = true
  message = "core flow id {value} is malformed or duplicated"

  [[gate.check]]
  kind = "row_count_between"
  id = "one-success-signal"
  severity = "block"
  on_fail = "fail"
  path = "explore/brief.md"
  section = "Success signal"
  min = 1
  max = 1
  message = "{value} success signals, expected exactly 1"

  [[gate.check]]
  kind = "verifier_pass"
  id = "kill-case-answered"
  severity = "block"
  on_fail = "fail"
  verifiers = ["kill-case-builder"]
  min_ratio = 0.0
  message = "kill-case-builder result absent from the report for {id}"

  [[gate.check]]
  kind = "column_contains_all"
  id = "remedy-flows-present"
  severity = "block"
  on_fail = "fail"
  path = "explore/brief.md"
  section = "Core flows"
  column = "ID"
  state_field = "explore.remedy_flows"
  skip_when = { state_field = "explore.remedy_flows", is_empty = true }
  message = "remedy cited {value}, which is absent from Core flows"

  [[gate.check]]
  kind = "elapsed_days_at_most"
  id = "time-box"
  severity = "block"
  on_fail = "fail"
  started_field = "explore.started_at"
  limit_field = "explore.timebox_days"
  remedy_started_field = "explore.remedy_started_at"
  remedy_limit_field = "explore.remedy_timebox_days"
  skip_when = { doc_exists = "explore/decision.yaml" }
  message = "time box spent, {value} of {limit} days, decide kill, park or promote"

  [[gate.check]]
  kind = "file_exists"
  id = "prototype-pruned"
  severity = "block"
  on_fail = "fail"
  paths = [".explore-prototype"]
  absent = true
  required_when = { path = "explore/decision.yaml", field = "decision", in = ["kill", "promote"] }
  message = "prototype directory still present after a {value} decision"

[[gate]]
phase = "discover"
requires = "explore"
on_fail = "fail"
send_back_to = "explore"
document = "requirements.yaml"
description = "Every requirement is complete, every actor resolves, every requirement sits in an epic, the user accepted."

  [[gate.check]]
  kind = "fields_present"
  id = "req-fields-present"
  collection = "requirements"
  fields = ["id", "actor", "statement", "rationale", "acceptance_signal", "priority", "source", "status"]
  message = "requirement {value} is missing a required field"

  [[gate.check]]
  kind = "ids_resolve"
  id = "actor-resolves"
  from = "requirements[].actor"
  to = "personas[].id"
  message = "actor {value} names no persona"

  [[gate.check]]
  kind = "length_between"
  id = "epic-not-empty"
  field = "epics[].requirements"
  min = 1
  exclude_status = ["withdrawn"]
  message = "epic {value} holds no live requirement"

  [[gate.check]]
  kind = "set_cover"
  id = "no-ungrouped-req"
  cover = "epics[].requirements"
  universe = "requirements[].id"
  exclude_status = ["withdrawn"]
  message = "requirement {value} belongs to no epic"

  [[gate.check]]
  kind = "fields_present"
  id = "accepted"
  field = "accepted_by"
  message = "requirements.yaml is not accepted"

[[gate]]
phase = "constitute"
requires = "discover"
on_fail = "fail"
send_back_to = "discover"
description = "The six context files audit clean and both reviewers report no blocking finding."

  [[gate.check]]
  kind = "context_audit"
  id = "CTX-AUDIT"
  allow_warn = false
  message = "context audit failed: {value}"

  [[gate.check]]
  kind = "report_metric"
  id = "REVIEW"
  metric = "verifiers.architecture_reviewer.payload.blocking_findings"
  op = "eq"
  value = 0
  message = "architecture-reviewer reports {value} blocking findings"

  [[gate.check]]
  kind = "report_metric"
  id = "ALIGNMENT"
  metric = "verifiers.alignment_auditor.payload.blocking_findings"
  op = "eq"
  value = 0
  message = "alignment-auditor reports {value} blocking findings"

  [[gate.check]]
  kind = "report_metric"
  id = "SEND-BACK-REQUIREMENTS"
  metric = "verifiers.architecture_reviewer.payload.send_back_requirements.length"
  op = "eq"
  value = 0
  on_fail = "send_back"
  message = "architecture-reviewer sends {value} requirements back to Discover"

[[gate]]
phase = "plan"
requires = "constitute"
on_fail = "fail"
send_back_to = "discover"
description = "Stories and sprint parse, every AC is testable, references resolve, no dependency cycle."

  [[gate.check]]
  kind = "doc_valid"
  id = "plan-docs"
  docs = ["stories/STORY-*.md", "stories/sprint.yaml"]

  [[gate.check]]
  kind = "story_valid"
  id = "plan-stories"
  scope = "sprint"

  [[gate.check]]
  kind = "ids_resolve"
  id = "plan-ids"
  prefixes = ["STORY", "AC", "REQ", "SPRINT", "UI"]
  on_fail = "send_back"

  [[gate.check]]
  kind = "field_in_enum"
  id = "plan-sprint-status"
  path = ".devforgeai/stories/sprint.yaml"
  field = "status"
  values = ["active"]

  [[gate.check]]
  kind = "verifier_pass"
  id = "plan-invest"
  verifiers = ["story-invest-auditor"]
  min_ratio = 1.0
  on_fail = "send_back"

[[gate]]
phase = "build"
requires = "plan"
on_fail = "fail"
send_back_to = "plan"
description = "Story implemented, tests green, coverage by layer met, lint clean."

  [[gate.check]]
  kind = "doc_valid"
  id = "build-docs"
  docs = ["stories/{id}.md"]
  on_fail = "send_back"

  [[gate.check]]
  kind = "field_in_enum"
  id = "build-status"
  path = "stories/{id}.md"
  field = "status"
  values = ["building", "built"]

  [[gate.check]]
  kind = "tests_pass"
  id = "build-tests"
  stacks = []
  allow_empty = false

  [[gate.check]]
  kind = "coverage_min"
  id = "build-coverage"
  layers = ["domain", "application", "infrastructure", "interface"]
  overall = true
  source = "run"

  [[gate.check]]
  kind = "lint_clean"
  id = "build-lint"
  stacks = []

  [[gate.check]]
  kind = "design_tokens"
  id = "build-design"
  paths = []
  severity = "warn"

  [[gate.check]]
  kind = "complexity_clean"
  id = "build-complexity"
  stacks = []
  severity = "warn"

  [[gate.check]]
  kind = "files_declared"
  id = "build-files"
  message = "{value} is outside the declared file set of {id}"

  [[gate.check]]
  kind = "antipattern_clean"
  id = "build-antipatterns"
  min_severity = "high"
  scope = "story"

  [[gate.check]]
  kind = "verifier_pass"
  id = "build-testable"
  verifiers = ["ac-test-writer"]
  min_ratio = 1.0
  on_fail = "send_back"

  [[gate.check]]
  kind = "verifier_pass"
  id = "build-context"
  verifiers = ["context-validator"]
  min_ratio = 1.0

  [[gate.check]]
  kind = "verifier_pass"
  id = "build-acs"
  verifiers = ["story-ac-verifier"]
  min_ratio = 1.0
  on_fail = "send_back"

[[gate]]
phase = "verify"
requires = "build"
on_fail = "fail"
send_back_to = "build"
description = "The QA report parses, every registered verifier passed, no blocker finding stands, every deferral resolves without a cycle, coverage by layer holds, lint is clean."

  [[gate.check]]
  kind = "doc_valid"
  id = "verify-docs"
  docs = ["reports/{id}-qa.yaml"]

  [[gate.check]]
  kind = "verifier_pass"
  id = "verify-acs"
  verifiers = ["ac-compliance-verifier"]
  min_ratio = 1.0
  on_fail = "send_back"

  [[gate.check]]
  kind = "verifier_pass"
  id = "verify-light"
  verifiers = ["standards-reviewer", "anti-pattern-scanner", "constraint-auditor",
               "coverage-gap-auditor", "dead-code-detector", "deferral-validator"]
  min_ratio = 1.0
  on_fail = "send_back"
  message = "a light-mode verifier reported a blocking finding for {id}"

  [[gate.check]]
  kind = "verifier_pass"
  id = "verify-deep"
  verifiers = ["security-auditor", "code-quality-auditor", "adr-conformance-reviewer"]
  min_ratio = 1.0
  on_fail = "send_back"
  skip_when = { path = "reports/{id}-qa.yaml", field = "mode", equals = "light" }
  message = "a deep-mode verifier reported a blocking finding for {id}"

  [[gate.check]]
  kind = "length_between"
  id = "verify-blockers"
  path = "reports/{id}-qa.yaml"
  field = "blockers"
  min = 0
  max = 0
  on_fail = "send_back"
  message = "{value} blocker findings stand for {id}"

  [[gate.check]]
  kind = "ids_resolve"
  id = "verify-ids"
  prefixes = ["FIND", "AC", "STORY", "ADR", "CON", "AP"]

  [[gate.check]]
  kind = "no_cycle"
  id = "verify-deferral-cycle"
  docs = ["reports/STORY-*-qa.yaml"]
  root = "{id}"
  from = "id"
  to = "deferrals[].target"
  prefix = "STORY"
  on_fail = "send_back"
  message = "a deferral chain returns to {id}"

  [[gate.check]]
  kind = "coverage_min"
  id = "verify-coverage"
  layers = ["domain", "application", "infrastructure", "interface"]
  overall = true
  source = "read"

  [[gate.check]]
  kind = "lint_clean"
  id = "verify-lint"
  stacks = []

  [[gate.check]]
  kind = "field_in_enum"
  id = "verify-story-status"
  path = "stories/{id}.md"
  field = "status"
  values = ["built", "verified"]

[[gate]]
phase = "release"
requires = ""
on_fail = "fail"
send_back_to = "verify"
description = "Every story in the release is built and verified, no deferral blocks deployment, the manifests and the docs match what ships, and the version moves forward."

  [[gate.check]]
  kind = "doc_valid"
  id = "release-docs"
  docs = ["releases/{id}.yaml"]

  [[gate.check]]
  kind = "file_exists"
  id = "release-file"
  paths = ["releases/{id}.yaml"]
  min_count = 1

  [[gate.check]]
  kind = "field_in_enum"
  id = "release-status"
  path = "releases/{id}.yaml"
  field = "status"
  values = ["released"]
  message = "{id} is at {value}; the release file moves to released before the gate runs"

  [[gate.check]]
  kind = "length_between"
  id = "release-set"
  path = "releases/{id}.yaml"
  field = "stories[]"
  min = 1
  message = "{id} names {value} stories; a release carries at least {limit}"

  [[gate.check]]
  kind = "fields_present"
  id = "release-entries"
  path = "releases/{id}.yaml"
  collection = "stories[]"
  fields = ["id", "title", "status", "qa_report", "verify_report", "verify_result"]

  [[gate.check]]
  kind = "ids_resolve"
  id = "release-ids"
  prefixes = ["STORY", "REQ", "ADR", "FIND"]
  on_fail = "send_back"

  [[gate.check]]
  kind = "release_stories"
  id = "release-stories"
  path = "releases/{id}.yaml"
  require_status = ["built", "released"]
  require_report = "verify"
  require_result = "PASS"
  on_fail = "send_back"
  message = "{value} is not built with a PASS verify report"

  [[gate.check]]
  kind = "verifier_pass"
  id = "release-deferrals"
  verifiers = ["deferral-auditor"]
  min_ratio = 1.0
  on_fail = "send_back"

  [[gate.check]]
  kind = "deploy_manifest"
  id = "release-manifest"
  path = "releases/{id}.yaml"
  platform = ""

  [[gate.check]]
  kind = "docs_cover"
  id = "release-docs-cover"
  path = "releases/{id}.yaml"
  min_ratio = 1.0

[[gate]]
phase = "reflect"
requires = ""
on_fail = "fail"
send_back_to = ""
description = "The reflect report exists and parses, every REC cites an OBS, every OBS cites a report path or a session id, and no REC lowers a compiled floor."

  [[gate.check]]
  kind = "file_exists"
  id = "reflect-report-exists"
  paths = ["reports/reflect-{id}.yaml"]
  min_count = 1

  [[gate.check]]
  kind = "doc_valid"
  id = "reflect-docs"
  docs = ["reports/reflect-{id}.yaml"]

  [[gate.check]]
  kind = "yaml_cites"
  id = "reflect-rec-cites-obs"
  doc = "reports/reflect-{id}.yaml"
  from = "recommendations"
  field = "observations"
  into = ["observations[].id"]
  min = 1

  [[gate.check]]
  kind = "yaml_cites"
  id = "reflect-obs-cites-source"
  doc = "reports/reflect-{id}.yaml"
  from = "observations"
  field = "sources"
  into = ["sources.reports[].path", "sources.sessions.files[].session_id"]
  min = 1

  [[gate.check]]
  kind = "no_threshold_decrease"
  id = "reflect-no-lowered-floor"
  doc = "reports/reflect-{id}.yaml"
  from = "recommendations"
  target_field = "target.path"
  key_field = "target.key"
  value_field = "proposed_value"
  files = ["gates.toml", "config.toml"]
```

Eight gates, sixty-three checks. The `[[verifier]]` names the `verifier_pass` and `report_metric` checks read are the twenty registrations in `## Outputs`; a gate that names a subagent absent from that registry fails with `DFA-E316` rather than passing silently.

## Send-back

Exit code 2 is a result, not an error. It is produced by `gate check` alone, and by `hook run stop` passing that result through. Every other subcommand exits 0, 1, 3, 4, or 5.

A gate yields `SEND BACK` when at least one failing check with `severity = "block"` resolves `on_fail = "send_back"`. The destination is the gate's `send_back_to`. The IDs cited in the handoff `Found` lines come from the report `findings` list, which is the union of the verifier findings ingested by `report ingest` and the CLI-generated findings below.

| Producing check | Condition | Destination | IDs cited |
|---|---|---|---|
| `discover-ids` | a `REQ`, `EPIC`, `PERSONA`, or `IDEA` reference has no definition | explore | the unresolved IDs, up to five |
| `constitute-ids` | a `CON`, `ADR`, or `REQ` reference has no definition | discover | the unresolved IDs |
| `plan-ids` | a `STORY`, `AC`, `REQ`, `SPRINT`, or `UI` reference has no definition | discover | the unresolved IDs |
| `build-docs` | the story document fails `doc validate` | plan | the story ID and the failing diagnostic code |
| `verify-acs` | the registered verifier reports `passed < total` | build | the `FIND-nnn` IDs the verifier emitted |
| `release-ids` | a released story ID has no story file | verify | the unresolved `STORY-nnn` IDs |

For a CLI-generated send-back the report gains one `findings` entry per cited ID, with `id` set to the cited ID, `severity` set to `block`, and `summary` set to the error message for the failing check, cut to 90 characters. The CLI allocates no `FIND-nnn` of its own; `FIND` IDs come from the Verify skill and its verifier subagents.

A phase whose `send_back_to` is `""`, which is Explore alone, reports `FAIL` instead, per conventions §5, where Explore may send back to nothing.

## Integration

Each row is one skill. The columns describe that skill's relationship with the CLI and with the phase pipeline the CLI enforces.

| Skill | Consumes from the CLI | Produces for the CLI | Sends back to | Receives send-back from | Shared subagents | state.toml read / written | Subcommands it calls | Documents the CLI validates for it | Gate it owns |
|---|---|---|---|---|---|---|---|---|---|
| Explore | `doc load` output; `handoff` block; `gate check` result for explore | `explore/brief.md`, `explore/decision.yaml` (IDEA-nnn, FLOW-nnn) | none; §5 gives Explore no upstream phase | Discover, when a REQ cites a missing IDEA | none; the CLI shares no subagent with any skill | reads `[current]`, `[active].explore`, `[explore].*`; writes all of them through `phase set` | `doc validate --allocate IDEA`, `phase set explore --id <IDEA-nnn> [--remedy <FLOW ids>]`, `doc validate <path>`, `explore prune --id <IDEA-nnn>`, `gate check --phase explore --id <IDEA-nnn>`, `handoff --phase explore --id <IDEA-nnn>`, `report show <IDEA-nnn> explore` | `explore/brief.md`, `explore/decision.yaml` | explore |
| Discover | `doc load explore-brief -`, `doc load explore-decision -`; handoff | `requirements.yaml` (REQ-nnn, EPIC-nnn, PERSONA-nnn) | Explore, when an IDEA reference does not resolve | Constitute and Plan, when a REQ reference does not resolve | none | reads `[active].explore`; writes `[active].discover` | `doc load discover-entry "$ARGUMENTS[0]"`, `doc validate --allocate IDEA`, `phase set discover --id <IDEA-nnn>`, `doc validate --allocate PERSONA`, `--allocate REQ`, `--allocate EPIC`, `doc accept requirements --id <IDEA-nnn>`, `doc reopen requirements --id <IDEA-nnn> --ids <ID,ID> --from <plan|constitute|design>`, `report show <IDEA-nnn> <plan|constitute|design>` | `requirements.yaml` | discover |
| Constitute | `doc load requirements -`; `context audit` result; handoff | `context/*.md` (CON-nnn), `adr/ADR-nnn.md` (ADR-nnn) | Discover, when a CON or REQ reference does not resolve | Plan, when an ADR contradicts a story assumption | none | reads `[active].discover`; writes `[active].constitute` | `gate require constitute <IDEA-nnn>`, `doc load requirements <IDEA-nnn>`, `doc validate --allocate CON`, `--allocate AP`, `--allocate ADR`, `context audit`, `gate check --phase constitute --id <IDEA-nnn>`, `handoff --phase constitute --id <IDEA-nnn>`, `phase set constitute --id <IDEA-nnn> [--remedy CON-nnn]` | the six `context/*.md` files, every `adr/ADR-nnn.md` | constitute |
| Plan | `doc load requirements -`, `doc load context all`, `doc load ui-spec <UI-nnn>`; `story validate` result | `stories/STORY-nnn.md` (STORY-nnn, AC-nnn), `stories/sprint.yaml` (SPRINT-nnn) | Discover on an unresolved REQ; Constitute on a contradicted ADR | Build, when a story is not implementable as written | none | reads `[active].constitute`; writes `[active].plan` | `gate require plan $ARGUMENTS[0]`, `doc load requirements $ARGUMENTS[0]`, `doc load context all`, `doc load ui-spec <UI-nnn>`, `doc validate --allocate STORY`, `--allocate AC`, `--allocate SPRINT`, `story validate --scope sprint`, `report show <STORY-nnn> <build|verify>`, `phase set plan --id <SPRINT-nnn>` | every `stories/STORY-nnn.md`, `stories/sprint.yaml` | plan |
| Build | `doc load story <STORY-nnn>`, `doc load context all`; `gate check --phase build` result; `--partial` annotations on PostToolUse | code and tests in the project tree; `reports/STORY-nnn-build.yaml` is CLI-written | Plan, when the story document fails validation or an AC is untestable | Verify, when a finding names a defect in the implementation | none | reads `[active].plan`; writes `[active].build` | `gate require build $ARGUMENTS[0]`, `doc load story $ARGUMENTS[0]`, `doc load context all`, `doc load sprint -`, `doc load ui-spec <UI-nnn>`, `doc load qa-report <STORY-nnn>`, `config get <key>`, `worktree ensure|list|remove <STORY-nnn>`, `phase set build --id <STORY-nnn>`, `commit <STORY-nnn> -m <message>`, `story files --diff --id <STORY-nnn>`, `antipattern scan --id <STORY-nnn>`, `report show <STORY-nnn> build --check <check-id>`, `report note <STORY-nnn> build --key build --file <path>` | `stories/STORY-nnn.md`; the CLI writes, and then validates, the build report | build |
| Verify | `doc load story <STORY-nnn>`, `report show <STORY-nnn> build`; ingested verifier blocks | `reports/STORY-nnn-qa.yaml` (FIND-nnn) | Build on an implementation defect; Plan on an untestable AC | Release, when a released story has no passing QA report | none | reads `[active].build`; writes `[active].verify` | `gate require verify $ARGUMENTS[0]`, `doc load story $ARGUMENTS[0]`, `doc load context all`, `report show <STORY-nnn> build`, `report show <vX.Y.Z> release`, `doc validate --allocate FIND`, `phase set verify --id <STORY-nnn>`, `gate check --phase verify --id <STORY-nnn>`, `handoff` | `reports/STORY-nnn-qa.yaml` | verify |
| Release | `doc load qa-report <STORY-nnn>`, `report show <STORY-nnn> verify` | `releases/vX.Y.Z.yaml` | Verify, when a story in the release has no PASS verify gate | Reflect, as a recommendation only, which sets no gate | none | reads `[active].verify`; writes `[active].release` | `story list --status built --json`, `phase set release --id <vX.Y.Z>`, `report show <STORY-nnn> verify`, `doc load qa-report <STORY-nnn>`, `doc load story <STORY-nnn>`, `doc load adr all`, `doc load requirements -`, `gate check --phase release --id <vX.Y.Z>`, `doc validate .devforgeai/releases/<vX.Y.Z>.yaml`, `handoff --phase release --id <vX.Y.Z>` | `releases/vX.Y.Z.yaml` | release |
| Design | `doc load requirements -`; `design lint` result on PreToolUse | `brand/tokens.json` (TOKEN-name), `ui-specs/UI-nnn.md` (UI-nnn) | Discover, when a UI spec needs a requirement that does not exist | Plan and Build, when a story references a UI-nnn that does not exist | none | reads `[current].phase` only; writes none, since Design is cross-cutting and advances no phase | `doc validate --allocate UI`, `doc load requirements -`, `doc load story <STORY-nnn>`, `doc load ui-spec <UI-nnn>`, `design lint --tokens`, `design lint <paths>`, `handoff --phase design --id <UI-nnn>`, `report show <UI-nnn> design` | `ui-specs/UI-nnn.md`, `brand/tokens.json` | none; Design is cross-cutting and `gates.toml` has one gate per phase |
| Reflect | `report show <id> <phase>` for every report in the window; `handoff` history from `[last_handoff]` | `reports/reflect-<date>.yaml` (OBS-nnn, REC-nnn) | any phase, as a recommendation that sets no gate | none; a recommendation produces no send-back | none | reads `[last_gate]`, `[last_handoff]`, `[active]`; writes none | `report aggregate <ID|--since <date>> --json`, `doc validate --allocate OBS`, `--allocate REC`, `gate check --phase reflect --id <date>`, `handoff --phase reflect --id <date>`, `report show <id> <phase>` | `reports/reflect-<date>.yaml` | none; Reflect owns no phase and its output is advisory, per §5 |
| devforgeai CLI | itself; `hook run` calls the other subcommands in process | `config.toml`, `gates.toml`, `state.toml`, `reports/<ID>-<phase>.yaml`, the hook files, the handoff block | none; the CLI produces a send-back result and does not request one | none; a send-back names a phase, and the CLI owns no phase | none; the CLI invokes no subagent and ingests the output of registered ones | reads and writes every key of `state.toml` | every subcommand in `## CLI calls` | every document in the doc-type table | every gate in `gates.toml`, as the evaluator rather than the owner |

## Handoff

Both blocks are rendered by `devforgeai handoff` from `state.toml` and the report. Labels occupy characters 1 to 10, content starts at character 11, and the `Full report:` line is the §6 exception.

PASS, ten lines:

```
Phase     4 · Build           STORY-014 · order-checkout
Done      214 tests · 87.4% coverage
Gate      PASS  6 checks
Verified  ac-compliance-verifier · 7/7 ACs

Next      /verify STORY-014
Then      /release v0.3.0
Blocked   none

Full report: .devforgeai/reports/STORY-014-build.yaml
```

SEND BACK with five findings, twelve lines. `Verified` and `Then` are both present, so the fixed lines number ten and the Found budget is two: one finding renders and the rest fold into the `+n more` line.

```
Phase     5 · Verify          STORY-014 · order-checkout
Done      5/7 ACs · 5 findings
Gate      SEND BACK to Build  verify-acs 5/7
Verified  ac-compliance-verifier · 5/7 ACs
Found     FIND-001 AC-003 has no test covering the empty cart path
Found     +4 more in report

Next      /build STORY-014 --remedy FIND-001,FIND-002
Then      /verify STORY-014 --resume
Blocked   none

Full report: .devforgeai/reports/STORY-014-verify.yaml
```

## Templates

Five templates ship with the binary. `gates.default.toml` and `settings.hooks.json` are files under `cli/templates/`, compiled in with `include_str!`; the framework root also carries `hooks/settings.hooks.json` byte-identical to the second. The three git hook scripts and `state.initial.toml` are string constants in `cli/src/hooks/githooks.rs` and `cli/src/state.rs`, so a target install needs no framework path to write them. In every template the token `@@DEVFORGEAI@@` is replaced at install time: with the bare word `devforgeai` when that name resolves on `PATH` at install time, and with the absolute path of the running binary in forward-slash form when it does not. Git hook files are written with LF line endings.

### `hooks/settings.hooks.json`

Merged into `.claude/settings.json` by the algorithm in `hook install`. This is the whole file.

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
    "UserPromptExpansion": [
      {
        "matcher": "explore|discover|constitute|plan|build|verify|release|design|reflect",
        "hooks": [
          {
            "type": "command",
            "command": "@@DEVFORGEAI@@",
            "args": ["hook", "run", "prompt-expansion"],
            "timeout": 30,
            "statusMessage": "devforgeai: phase gate"
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
      },
      {
        "matcher": "Bash|PowerShell",
        "hooks": [
          {
            "type": "command",
            "command": "@@DEVFORGEAI@@",
            "args": ["hook", "run", "pre-tool-use"],
            "timeout": 600,
            "statusMessage": "devforgeai: checking the command"
          }
        ]
      },
      {
        "matcher": "Write|Edit|NotebookEdit|Bash|PowerShell|Agent",
        "hooks": [
          {
            "type": "command",
            "command": "@@DEVFORGEAI@@",
            "args": ["hook", "run", "trust-check"],
            "timeout": 600,
            "statusMessage": "devforgeai: trust"
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
            "if": "Write(**/.devforgeai/**)",
            "timeout": 60
          },
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
    ]
  }
}
```

The file registers six events and dispatches seven arms of `devforgeai hook run`. Every entry is exec form: `command` holds the binary path alone and `args` holds the argument vector, so no shell parses the line and `@@DEVFORGEAI@@` may expand to a path holding spaces. A settings command string never carries `--json`: the dispatcher decides its own output shape per event, and a `--json` envelope on stdout would be parsed by the harness as a hook decision object.

The three `PreToolUse` groups are separate registrations rather than one wide matcher, because they do different work: `pre-tool-use` on the write tools resolves a path and runs the producer, declared-set and token checks; `pre-tool-use` on the shell tools scans the command for write targets and decides the metrics-command case; `trust-check` runs `trust verify` and nothing else across all six tools, so the trust refusal cannot fail for any other reason. `@@TEST_COMMAND@@` in the `PostToolUse` shell group is replaced at install time with the `config.toml` test command, and `@@VERIFIERS@@` in the `SubagentStop` matcher with the pipe-joined names of the registered verifiers. Timeouts are seconds: 900 for the events that can run the project's test suite, 120 for `session-start`, 60 for the annotating and ingesting arms, 30 for `prompt-expansion`, which the harness caps at 30 in any case.

### `templates/pre-commit`

```sh
#!/bin/sh
# devforgeai-hook v1
DFA="@@DEVFORGEAI@@"
"$DFA" trust verify || exit 4

git_dir=$(git rev-parse --git-dir)
list="$git_dir/devforgeai-staged"
git diff --cached --name-only --diff-filter=ACM -- .devforgeai > "$list"

status=0
while IFS= read -r f; do
  [ -n "$f" ] || continue
  "$DFA" doc validate "$f" || status=1
done < "$list"
rm -f "$list"

if [ -d .devforgeai/context ]; then
  "$DFA" context audit || status=1
fi

exit $status
```

### `templates/commit-msg`

```sh
#!/bin/sh
# devforgeai-hook v1
DFA="@@DEVFORGEAI@@"
"$DFA" trust verify || exit 4

if grep -Eq '(STORY|ADR)-[0-9][0-9][0-9]' "$1"; then
  exit 0
fi

echo "devforgeai: commit message has no STORY-nnn or ADR-nnn token" >&2
exit 1
```

### `templates/pre-push`

```sh
#!/bin/sh
# devforgeai-hook v1
DFA="@@DEVFORGEAI@@"
"$DFA" trust verify || exit 4

sprint=".devforgeai/stories/sprint.yaml"
[ -f "$sprint" ] || exit 0

ids=$(awk '
  /^[[:space:]]*-[[:space:]]*id:/ { id = $3 }
  /^[[:space:]]*status:[[:space:]]*building[[:space:]]*$/ { if (id != "") print id }
' "$sprint")

status=0
for id in $ids; do
  "$DFA" gate check --phase build --id "$id" || status=1
done

exit $status
```

The awk program reads the two keys the CLI needs from each sequence item and relies on `id` preceding `status` inside the item, which `templates/sprint.min.yaml` fixes. Git for Windows runs these files through its bundled `sh`, so one copy serves Git Bash and every POSIX shell; the scripts use no `bash` extension, no array, and no `[[`.

### `templates/state.initial.toml`

Written by `init` before `stack detect` runs.

```toml
schema = "devforgeai/state/1"
updated_at = "@@NOW@@"

[current]
phase = "explore"
id = ""

[active]
explore = ""
discover = ""
constitute = ""
plan = ""
build = ""
verify = ""
release = ""

[explore]
idea_id = ""
started_at = ""
timebox_days = 5
remedy_started_at = ""
remedy_timebox_days = 1
remedy_flows = []

[constitute]
remedy_con = ""

[last_gate]
phase = ""
id = ""
result = "NOT_RUN"
at = ""
report = ""
send_back_to = ""
failed_checks = []

[last_handoff]
rendered_at = ""
phase = ""
id = ""
lines = []

[stop_hook]
block_count = 0
blocked_phase = ""
blocked_id = ""
```

`@@NOW@@` is replaced with the RFC 3339 UTC timestamp at write time. `[stop_hook].blocked_session`, `[stop_hook].scanned_at`, and the whole of `[plan]` and `[last_cross]` are absent from the initial file and take their empty defaults on load, so a state file written before those keys existed still parses.

### `templates/sprint.min.yaml`

The keys `story validate` and the `pre-push` hook read. The Plan skill spec defines the rest of the file.

```yaml
schema: devforgeai/sprint/1
id: SPRINT-001
phase: plan
status: active
produced_by: planning-work
consumes: []
open_questions: []
stories:
  - id: STORY-014
    status: building
  - id: STORY-015
    status: ready
```

### `templates/context-stub.md`

Written by `init --analyze` for the three stub files, with `@@NAME@@` replaced by the file's base name, `@@DETECTED@@` by the detected configuration list, and `@@SECTION@@` repeated once per heading from the brownfield table in `## CLI calls`. The `##` lines inside the fence below belong to the template file, not to this spec.

```markdown
---
schema: devforgeai/context-@@NAME@@/1
id: @@NAME@@
phase: constitute
status: draft
produced_by: establishing-context
consumes: []
open_questions:
  - body drafted by the Constitute skill
---

# @@NAME@@

## Detected configuration

@@DETECTED@@

## @@SECTION@@

Drafted by the Constitute skill.
```

## Evals

The CLI ships no `evals.json` and no `cases.jsonl`. Conventions §9 attaches those three artifacts to skills and states that nothing in Rust is evaluated by Python. The CLI's equivalent is the Rust test suite below, written before the implementation. This section also carries the contract for the shared Python runner, which §9 assigns to this spec.

### Test harness rules

Every integration test builds a project in a `tempfile::TempDir` and passes `--project <temp>`. No test reads or writes the real home directory: the `test-home` cargo feature makes `trust.rs` resolve the trust directory from `DEVFORGEAI_HOME`, and every trust test sets that variable to a temp path. With the feature off, the variable is read by no code path, so a released binary cannot be redirected. `DEVFORGEAI_NOW` under the same feature fixes timestamps so report and state files compare byte for byte. The suite runs as `cargo test --features test-home`. Two guard tests hold the line: `trust_home_env_ignored_without_test_feature` compiles the resolver with default features and asserts the variable is unread, and `no_test_touches_real_home` asserts that the resolved trust directory is inside the test's temp root.

### Fixture directories

Under `cli/tests/fixtures/`, each a complete project skeleton copied into the temp directory by `fn fixture(name: &str) -> TempDir`.

| Fixture | Contents |
|---|---|
| `empty/` | an empty directory, for root discovery and `init` |
| `rust-min/` | `Cargo.toml`, `src/domain/order.rs`, `tests/`, an lcov file under `target/devforgeai/` |
| `node-min/` | `package.json` with `test`, `coverage`, `lint` scripts, `pnpm-lock.yaml`, `coverage/lcov.info` |
| `python-min/` | `pyproject.toml` with a `[tool.ruff]` table, `.devforgeai/coverage.xml` in cobertura form |
| `go-min/` | `go.mod`, `.devforgeai/coverage.out` in go-cover form |
| `dotnet-min/` | `app.csproj`, a cobertura file under `.devforgeai/coverage/<guid>/` |
| `jvm-maven-min/` | `pom.xml`, `target/site/jacoco/jacoco.xml` |
| `ruby-min/` | `Gemfile`, `spec/`, `.rubocop.yml`, `coverage/lcov.info` |
| `polyglot/` | `Cargo.toml` and `package.json` at the root, for two `[[stack]]` tables |
| `unknown/` | source files with no marker, for the degraded path |
| `docs-valid/` | `.devforgeai/` with one valid document of every doc type |
| `docs-broken/` | one document per `DFA-E2xx` code, each named for the code it triggers |
| `gates-bad/` | `gates.toml` variants, one per `DFA-E30x` code |
| `reports/` | a PASS build report, a SEND BACK verify report with five findings, an unparsable report |
| `hooks-json/` | an existing `.claude/settings.json` with a foreign hook and a foreign top-level key |
| `brownfield/` | a layered source tree with manifests, linter configs, and version pins, for `init --analyze` |

### Order of implementation

Each wave is written test-first and lands green before the next starts.

1. `errors`, `json`, `project`: root discovery, the envelope, the exit-code mapping.
2. `config`, `stack`: parsing and detection.
3. `doc` and its three submodules, then `doc validate` and `doc load`.
4. `gates`, `state`, then `gate require` and `phase set`.
5. `exec`, `coverage`, then `gate check`.
6. `report`, then `report ingest`.
7. `handoff`.
8. `audit`, `story`, `design`.
9. `hooks`, then `hook install` and `hook run`.
10. `trust`.
11. `analyze`, then `init`, which composes every wave above.

### Test names

Unit tests live beside their module; integration tests live in `cli/tests/<subcommand>.rs` and drive the binary through `assert_cmd`.

**`project`, `json`, `errors`** — unit: `root_found_in_ancestor`, `root_missing_gives_e030`, `project_flag_overrides_discovery`, `project_flag_missing_gives_e031`, `atomic_write_replaces_file`, `atomic_write_leaves_tmp_on_rename_failure`, `envelope_serialises_every_field`, `envelope_emitted_on_failure`, `error_code_maps_to_exit`.

**`init`** — integration: `init_creates_directory_tree`, `init_writes_default_gates`, `init_writes_initial_state`, `init_runs_stack_detect`, `init_merges_settings_hooks`, `init_installs_three_git_hooks`, `init_without_git_warns_e130_and_exits_zero`, `init_twice_gives_e110`, `init_force_overwrites`, `init_missing_framework_gives_e111`, `init_no_hooks_skips_both`, `init_json_envelope_lists_counts`. Analyze: `analyze_drafts_tech_stack_from_manifests`, `analyze_drafts_source_tree_with_layers`, `analyze_drafts_dependencies_with_versions`, `analyze_stubs_coding_standards`, `analyze_stubs_architecture_constraints`, `analyze_stubs_anti_patterns`, `analyze_sets_status_draft_on_all_six`, `analyze_skips_existing_file_without_force`, `analyze_time_cap_adds_open_question`.

**`stack detect`** — unit: `markers_rust`, `markers_node_with_pnpm_lock`, `markers_python_with_uv_lock`, `markers_go`, `markers_dotnet_glob`, `markers_jvm_maven`, `markers_jvm_gradle_kts`, `markers_ruby_with_spec_dir`, `node_without_test_script_gives_empty_command`, `ruby_without_rubocop_gives_empty_lint`, `jvm_lint_is_empty`, `detection_order_is_table_order`, `depth_two_marker_found`, `ignored_directory_marker_skipped`. Integration: `detect_writes_config`, `detect_polyglot_writes_two_stacks`, `detect_unknown_sets_degraded`, `detect_preserves_layers_and_verifiers`, `detect_dry_run_writes_nothing`, `detect_json_shape`.

**`doc validate`** — unit: `frontmatter_parses_minimal`, `frontmatter_missing_fence_gives_e201`, `frontmatter_extra_key_gives_e204`, `frontmatter_order_gives_e205`, `frontmatter_type_gives_e206`, `schema_mismatch_gives_e207`, `status_outside_enum_gives_e208`, `id_malformed_gives_e209`, `id_duplicate_gives_e209`, `yaml_doc_reads_top_level_keys`, `tokens_json_reads_meta_key`, `index_defines_from_heading`, `index_defines_from_yaml_item`, `reference_without_definition_gives_e210`, `consumes_without_body_gives_w201`, `body_without_consumes_gives_w202`, `heading_out_of_order_gives_e213`, `allocate_first_id_is_001`, `allocate_next_after_gap`, `allocate_unknown_prefix_gives_e214`, `allocate_at_999_gives_e215`. Integration: `validate_all_walks_devforgeai`, `validate_unmatched_path_is_skipped`, `producer_check_matching_phase_exits_zero`, `producer_check_mismatch_gives_e212`, `producer_check_design_allowed_from_any_phase`, `producer_check_stdin_content_without_file`, `validate_json_lists_per_file_errors`, `allocate_prints_id_alone`.

**`doc accept`, `doc reopen`, `explore prune`** — integration: `accept_sets_accepted_by_and_status`, `accept_moves_draft_and_reopened_requirements`, `accept_empty_epics_gives_e260`, `accept_leaves_other_bytes`, `reopen_raises_revision`, `reopen_appends_revision_log`, `reopen_allocates_req_per_ui`, `reopen_nulls_acceptance`, `reopen_bad_id_gives_e261`, `reopen_leaves_other_bytes`, `prune_removes_on_kill`, `prune_removes_on_promote`, `prune_keeps_on_park`, `prune_absent_directory_exits_zero`, `prune_without_decision_gives_e200`.

**`doc load`** — integration: `load_story_prints_bytes`, `load_discover_entry_prints_brief_and_decision`, `load_discover_entry_prints_requirements_without_brief`, `load_discover_entry_unknown_arg_exits_zero_silent`, `load_context_all_concatenates_six`, `load_adr_all_orders_by_number`, `load_reflect_latest_picks_newest`, `load_missing_gives_e200`, `load_unknown_name_gives_e250`, `load_json_carries_content`.

**`gate require`** — integration: `require_plan_resolves_idea_through_requirements`, `require_build_resolves_epic_through_story_consumes`, `require_build_two_epics_gives_e013`, `require_verify_reads_build_report_same_story`, `require_release_checks_every_story_in_manifest`, `require_discover_without_explore_exits_zero`, `require_design_gives_e300`, `require_with_empty_predecessor_exits_zero`, `require_pass_report_exits_zero`, `require_fail_report_gives_e321`, `require_missing_report_gives_e321`, `require_falls_back_to_active_id`, `require_unknown_phase_gives_e012`, `require_writes_nothing`.

**new check kinds** — unit: `no_cycle_detects_cycle_through_root`, `no_cycle_ignores_cycle_off_root`, `no_cycle_leaf_outside_docs`, `complexity_clean_skips_empty_command`, `antipattern_clean_wraps_exit_code`, `files_declared_resolves_worktree`, `release_stories_checks_status_and_report`, `deploy_manifest_per_platform`, `deploy_manifest_literal_secret_gives_e343`, `docs_cover_skips_without_command`, `yaml_cites_resolves_union`, `yaml_cites_missing_path_gives_e345`, `yaml_cites_below_min_gives_e346`, `yaml_cites_unknown_value_gives_e347`, `no_threshold_decrease_gives_e348`, `verifier_pass_zero_total_is_one`, `coverage_min_source_read_runs_no_command`, `field_in_enum_passes_and_fails`, `field_is_date_parses_format`, `field_is_date_after_field`, `length_between_excludes_status`, `fields_present_collection_form`, `fields_present_path_form_is_field_set`, `set_cover_detects_uncovered_and_duplicate`, `row_count_between_counts_table_rows`, `row_count_between_counts_body_lines_without_table`, `column_matches_pattern_and_unique`, `column_contains_all_reads_state_array`, `elapsed_days_prefers_remedy_pair`, `elapsed_days_uses_calendar_days`, `report_metric_reads_length_suffix`, `report_metric_absent_metric_fails`, `ids_resolve_from_to_form`, `file_exists_absent_flag`, `skip_when_records_skip`, `required_when_false_records_skip`, `null_when_and_empty_when`, `message_placeholders_substituted`, `gate_document_supplies_default_path`.

**`gates` and minimums** — unit: `gates_parse_default_file`, `unknown_kind_gives_e301`, `unknown_key_gives_e302`, `missing_required_kind_gives_e303`, `required_kind_as_warn_gives_e303`, `coverage_floor_below_minimum_gives_e303`, `verifier_ratio_below_floor_gives_e303`, `cli_min_version_above_binary_gives_e304`, `duplicate_phase_gives_e305`, `duplicate_check_id_gives_e305`, `send_back_without_target_gives_e306`, `default_gates_satisfy_minimums`, `id_token_substituted_in_paths`.

**`exec` and `coverage`** — unit: `command_runs_with_shell_per_platform`, `command_timeout_terminates_and_gives_e318`, `command_spawn_failure_gives_e319`, `output_tail_capped_at_4096`, `lcov_parses_records`, `cobertura_parses_sources_prefix`, `jacoco_parses_package_and_sourcefile`, `gocover_parses_blocks`, `devforgeai_json_parses_files`, `unparsable_report_gives_e312`, `layer_assignment_is_first_match`, `excluded_files_dropped`, `unassigned_files_counted`, `percent_rounds_half_away_from_zero`, `empty_layer_scores_hundred`, `overall_spans_every_file`.

**`gate check`** — integration: `check_pass_writes_report_and_state`, `check_fail_names_failing_check`, `check_send_back_exits_two`, `check_warn_check_does_not_change_result`, `check_partial_runs_three_kinds_only`, `check_partial_exits_zero_on_failure`, `check_partial_leaves_state_untouched`, `check_no_run_skips_commands`, `check_degraded_skips_command_kinds`, `check_degraded_ignores_coverage_floor`, `check_empty_test_command_gives_e310`, `check_layer_below_threshold_gives_e313`, `check_missing_verifier_block_gives_e316`, `check_verifier_ratio_gives_e317`, `check_preserves_existing_verifiers_block`, `check_test_counts_parsed_per_stack`, `check_test_counts_null_without_match`, `check_json_shape`, `check_human_output_columns`.

**`report ingest`** — integration: `ingest_writes_verifier_block`, `ingest_from_stdin`, `ingest_replaces_previous_block_same_subagent`, `ingest_merges_findings_by_id`, `ingest_unknown_subagent_gives_w411_exit_zero`, `ingest_unparsable_gives_e410_and_status_unparsed`, `ingest_without_active_id_gives_e412`, `ingest_creates_report_when_absent`.

**`report show`** — integration: `show_prints_yaml`, `show_check_filters_one_entry`, `show_missing_gives_e400`, `show_unparsable_gives_e401`.

**`handoff`** — unit: `design_phase_renders_em_dash`, `send_back_prints_remedy_and_resume_lines`, `label_column_is_ten_wide`, `phase_column_padded_to_twenty`, `line_capped_at_hundred_with_ellipsis`, `pass_block_is_ten_lines`, `send_back_three_findings_fits_twelve`, `send_back_five_findings_truncates_to_plus_three`, `budget_zero_drops_then_line`, `verified_omitted_without_verifiers`, `verified_picks_lowest_ratio`, `found_ordered_by_severity_then_id`, `full_report_line_exempt_from_column_rule`, `done_counts_per_phase`, `next_then_table_per_result`, `trust_fail_renders_gate_line`. Integration: `handoff_without_report_renders_not_run`, `handoff_writes_last_handoff`, `handoff_json_carries_lines`.

**`context audit`** — integration, one per check: `ca1_missing_file_gives_e220`, `ca2_draft_status_gives_e223`, `ca2_open_question_gives_e223`, `ca3_heading_order_gives_e213`, `ca4_orphan_con_gives_e224`, `ca4_con_with_resolving_req_passes`, `ca5_key_two_values_gives_e221`, `ca5_key_outside_namespace_gives_e221`, `ca5_proposed_adr_ignored`, `ca6_incomplete_anti_pattern_gives_e225`, `ca7_adr_reference_gives_e226`, `ca8_supersession_gives_e227`, `audit_duplicate_constraint_gives_e222`, `audit_all_pass_exits_zero`, `audit_json_lists_checks`.

**`story validate`** — integration: `story_given_when_then_is_testable`, `story_verb_form_is_testable`, `story_prose_ac_gives_e230`, `story_without_ac_gives_e234`, `story_unresolved_req_gives_e231`, `story_cycle_gives_e232`, `sprint_unknown_story_gives_e233`, `scope_active_reads_state`, `scope_all_walks_directory`, `story_json_shape`.

**`design lint`** — integration: `lint_color_leaf_resolves_light_and_dark`, `lint_skips_explore_prototype`, `lint_skips_explore_mockups`, `tokens_mode_rejects_string_color_leaf_e243`, `tokens_mode_rejects_unknown_group_e243`, `tokens_mode_rejects_uppercase_leaf_e243`, `tokens_mode_undefined_token_in_ui_spec_e244`, `tokens_mode_with_path_gives_e010`, `lint_hex_literal_gives_e240`, `lint_named_colour_gives_e240`, `lint_font_size_literal_gives_e241`, `lint_unknown_token_gives_e242`, `lint_var_token_passes`, `lint_transparent_passes`, `lint_nearest_token_is_closest_rgb`, `lint_missing_tokens_gives_e120`, `lint_path_outside_globs_skipped`, `lint_default_paths_from_globs`.

**`hook install`** — integration: `install_creates_settings_when_absent`, `install_merges_into_existing_settings`, `install_preserves_foreign_hooks_and_keys`, `install_duplicate_command_gives_w130`, `install_backs_up_settings`, `install_unparsable_settings_gives_e131`, `install_writes_three_scripts_with_marker`, `install_substitutes_devforgeai_token`, `install_foreign_hook_gives_e132`, `install_force_replaces_foreign_hook`, `install_uses_git_common_dir`, `install_git_only_skips_settings`.

**`hook run`** — integration: `session_start_runs_detect_then_handoff`, `pre_tool_use_producer_mismatch_exits_two`, `pre_tool_use_design_violation_exits_two`, `pre_tool_use_path_outside_devforgeai_exits_zero`, `post_tool_use_write_annotates_only`, `post_tool_use_bash_matching_test_command_runs_partial`, `post_tool_use_bash_other_command_exits_zero`, `stop_first_fail_exits_two_and_counts`, `stop_second_fail_exits_zero_with_handoff`, `stop_pass_resets_counter`, `stop_different_id_resets_counter`, `subagent_stop_registered_ingests`, `subagent_stop_unregistered_exits_zero`, `hook_stdin_not_json_gives_e021`, `hook_missing_key_gives_e020`, `hook_trust_failure_blocks_on_pre_tool_use`, `hook_trust_failure_exits_four_on_post_tool_use`.

**`trust`** — integration: `pin_writes_entry`, `pin_refuses_when_claudecode_set`, `pin_refuses_when_entrypoint_set`, `pin_allows_empty_variable`, `pin_digest_mismatch_gives_e503`, `pin_malformed_digest_file_gives_e510`, `pin_replaces_entry_for_same_path`, `verify_matching_digest_exits_zero`, `verify_without_trust_file_gives_e501`, `verify_without_entry_gives_e502`, `verify_modified_binary_gives_e503`, `verify_source_drift_in_session_gives_e504`, `verify_source_drift_outside_session_exits_zero`, `verify_older_pin_gives_w500`, `source_digest_is_order_independent_of_walk`, `trust_home_env_ignored_without_test_feature`, `no_test_touches_real_home`.

**`story files`, `story list`, `worktree`, `commit`, `config get`, `antipattern scan`, `report note`, `report aggregate`** — integration: `files_check_declared_path_exits_zero`, `files_check_undeclared_gives_e239`, `files_check_without_active_build_exits_zero`, `files_list_prints_triples`, `files_diff_uses_merge_base`, `files_diff_undeclared_gives_e239`, `story_list_filters_by_status`, `story_list_unknown_status_gives_e230`, `story_list_filters_by_sprint`, `worktree_ensure_is_idempotent`, `worktree_ensure_seeds_state`, `worktree_ensure_overlap_gives_e272`, `worktree_ensure_outside_git_gives_e271`, `worktree_remove_dirty_gives_e273`, `worktree_remove_force_removes`, `commit_prefixes_story_id`, `commit_keeps_message_holding_id`, `commit_undeclared_path_gives_e239`, `commit_empty_stage_gives_w243`, `commit_hook_failure_exits_one`, `config_get_prints_value`, `config_get_unknown_key_gives_e013`, `config_get_stack_selects_by_id`, `config_get_array_prints_one_per_line`, `antipattern_literal_detector_matches`, `antipattern_regex_detector_matches`, `antipattern_glob_detector_reads_no_file`, `antipattern_min_severity_filters`, `antipattern_clean_exits_zero`, `report_note_writes_key`, `report_note_keeps_produced_by`, `report_note_bad_schema_gives_e413`, `aggregate_id_window_collects_reports`, `aggregate_version_window_walks_release`, `aggregate_since_window_filters_by_finished_at`, `aggregate_both_forms_gives_e430`, `aggregate_absent_sessions_is_absent_not_error`, `aggregate_session_root_outside_home_gives_e421`.

**`phase set`** — integration: `set_writes_current_and_active`, `set_explore_writes_timebox_from_config`, `set_explore_remedy_keeps_started_at`, `set_constitute_remedy_writes_remedy_con`, `set_remedy_on_other_phase_gives_e011`, `set_entry_phase_without_predecessor_exits_zero`, `set_advances_state`, `set_resets_stop_counter`, `set_without_predecessor_pass_gives_e320`, `set_keeps_other_active_ids`, `set_unknown_phase_gives_e012`, `set_json_shape`.

### Python, the shared eval runner

`evals/runner/run_jsonl.py` is framework-level and skill-agnostic. It uses the standard library alone, and the processes it spawns are `claude`, `devforgeai trust verify`, and `git` for a case that declares `setup.git`. The runner and the graders evaluate skills only. No gate reads their output, no `gates.toml` check kind names them, and no subcommand of the CLI invokes them.

Arguments:

| Argument | Type | Default | Effect |
|---|---|---|---|
| `--cases <path>` | path | `<skill>/evals/cases.jsonl` | the `cases.jsonl` file |
| `--skill <path>` | path | required with no `--cases` | the skill directory installed into each workspace, excluding its `evals/` and `agents/` subtrees so the model under test cannot read the graders or the fixtures |
| `--graders <path>` | path | `graders.py` beside `--cases` | the module holding the grader functions |
| `--out <path>` | path | `results.jsonl` beside `--cases` | appended, one line per case |
| `--model <name>` | string | `sonnet` | passed to `claude -p --model` |
| `--timeout <seconds>` | int | 900 | per-case wall clock, overridden by a case's own `timeout` key. A measured full-phase case ran 828 seconds at $2.85, so the default leaves headroom above the longest real case rather than above a guess. The Constitute cases carry `timeout: 1500`, because writing six context files and two ADRs is the longest phase in the set |
| `--jobs <n>` | int | 1 | cases run in parallel |
| `--filter <glob>` | string | `*` | case ids to run, matched with `fnmatch` |
| `--case <id>` | string, repeatable | none | run exactly these case ids |
| `--workdir <path>` | path | a `mkdtemp` directory named `dfa-evals-*` | where workspaces are created |
| `--logdir <path>` | path | `<workdir>/logs` | where the raw child streams are written |
| `--keep` | flag | false | keeps workspaces after the run |
| `--claude-bin <path>` | path | `claude` | the executable invoked |
| `--devforgeai-bin <path>` | path | `<framework>/cli/target/release/devforgeai[.exe]` | the binary put on the child's `PATH` |
| `--framework-root <path>` | path | the repository holding the runner | the root the binary and the hook template are resolved against |
| `--claude-config <mode>` | enum `isolate` \| `inherit` | `isolate` | `isolate` redirects `CLAUDE_CONFIG_DIR` into the workspace |
| `--hooks` / `--no-hooks` | flag | hooks on when `trust verify` passes | whether the workspace carries the framework hook block |
| `--dry-run` | flag | false | prints the argv and the workspace plan and runs no child |

**The verification gates the suite holds.** Four things are checked before any model is invoked, because a case that fails one of them measures the fixture rather than the skill, and it costs a paid run to find out:

1. **Dry-run materialisation** (`--dry-run`). Every case's workspace is built from its `setup` and its fixtures, the argv and the prompt are printed, and no child runs. A fixture path that does not resolve, a `setup.git` that cannot be created, or a prompt that does not survive the stdin path fails here.
2. **The preamble exits 0.** A skill's `!` preamble lines are CLI calls, and a non-zero exit there aborts the whole invocation before the body loads, so a case whose fixture omits the predecessor report or the entry document measures nothing. A case declares those calls under `preflight` and `--preflight` runs each against the materialised workspace, requiring exit 0.
3. **`config.toml` and `state.toml` are what the binary writes.** The three project files a case inherits — `state.toml`, `config.toml`, `gates.toml` — come from `cli_defaults()`, which runs `devforgeai init` once per process in a throwaway directory outside every workspace and keeps the three files it wrote. A case seeds only the keys it cares about and inherits the rest, so a fixture cannot drift from what an install produces. A TOML parse is not enough on its own: the binary refuses a `config.toml` with no `generated_at` that parses perfectly, and `--preflight` is where that refusal surfaces.
4. **`gates.toml` loads for the phase the case exercises.** Every `preflight` call goes through `config::load` and the gate loader, so a check kind, a key, or an enum value a fixture carries and the binary does not is caught at suite level rather than inside the run.

The `preflight` key is the whole mechanism for 2, 3 and 4. A case names under it the CLI calls its skill makes before it writes anything — the phase's `phase set`, the id allocation that precedes it, the `worktree ensure` a Build run opens with — and `--preflight` runs each against a throwaway copy of the materialised workspace, requiring exit 0 from every one. The copy is thrown away because those calls mutate it, so the same case can then be run for real against a clean workspace.

A `preflight` entry is a bare command string required to exit 0, or an object saying what else the entry is for: `{"command", "exit", "code", "forbid_code"}`. `exit` is the code required and defaults to 0; `"any"` says the exit is not the point. `code` names a `DFA-` code the call has to raise, which is how a case asserts the refusal it exists to catch — Explore's exhausted-prefix case runs `{"command": "doc validate --allocate IDEA", "exit": 1, "code": "DFA-E215"}`. `forbid_code` names one the call must not raise, which is how a case asserts a path is open without pinning the exit — Build's entry runs `{"command": "gate check --phase build --partial --id STORY-014 --quiet", "exit": "any", "forbid_code": "DFA-E311"}`, since a partial gate legitimately fails on work not yet done and the one thing it must not report is a missing test command. Release's cases are the plain form, `["phase set release --id v0.3.0"]`, with one exception asserting the refusal: `{"command": "phase set release --id v0.3.0", "exit": 1, "code": "DFA-E320"}` is the case whose fixture holds a verify gate short of PASS.

**Seeded files and the shebang rule.** A case's `setup.files` entries are written into the workspace as given, and an entry whose content opens with `#!` is written executable. Such a file is a command the run is expected to execute rather than a document: `config.toml` names `./ci/test`, and Build's red-green loop turns on its exit code. Without the mode bit the loop failed on a permission error that looked like a test failure.

**What the child sees.** The release binary's directory is prepended to `PATH`, because every skill preamble opens with `` !`devforgeai …` `` and a non-zero exit there aborts the whole invocation. The prompt travels on stdin rather than in argv: on Windows `claude` is an npm `.cmd` shim and an argument carrying newlines does not survive `cmd.exe`. The argv is

```
<claude-bin> -p --model <model> --output-format stream-json --verbose
  --include-partial-messages --permission-mode bypassPermissions
  --allowedTools "<the list below>" --add-dir <workspace>
  # a case that seeds answers:
  --mcp-config <workspace>/.claude/mcp-permissions.json
  --permission-prompt-tool mcp__dfa-permissions__approve
  # a case that seeds none:
  --permission-prompts none
```

`--permission-mode` is `bypassPermissions`, and the reason is the acceptance set rather than convenience. `acceptEdits` covers Write and Edit and nothing else, and `claude -p` has no approval surface, so every command the mode does not cover is denied outright with no one to ask. A compound shell command matches no `Bash(devforgeai *)` rule — `cd wt/STORY-014 && ./ci/test` is what a Build run issues — and neither does the project's own test command, so a run under `acceptEdits` failed on the shell rather than on the skill. What bypassing costs is bounded: the workspace is a throwaway the runner creates under `mkdtemp` and deletes, the child sees no user configuration, and the framework's own hooks still run inside it — the `PreToolUse` deny fires in every permission mode, `bypassPermissions` included, and no allow overrides it. So the gates still enforce and only the prompt is gone.

**The tamper guard.** Bypassing the prompt means a model in an eval can rewrite the files that decide whether it passed. `.devforgeai/gates.toml`, `.devforgeai/config.toml`, `.devforgeai/state.toml`, and the workspace `.claude/settings.json` are hashed before the run and after it, and any change marks the case `status: tampered`, with the changed path and its before and after digests in `tampered`, whatever the grader returned. `state.toml` is the exception to the equality check, because `phase set` rewrites it by design and that is the run doing its job; it is hashed anyway so the record carries both digests, and reported only when the file is absent afterwards, since a deletion is never the CLI's doing.

No grader reads those four files today, so tampering buys nothing. That is a property of the current graders rather than of the harness, and the guard makes it a property of the harness: a grader added later that does read one of them inherits the protection rather than having to ask for it.

`--allowedTools` is still passed, because naming a tool grants it and denies nothing and the list is what a case's transcript is read against: `Bash(devforgeai *)`, `PowerShell(devforgeai *)`, `Bash(devforgeai:*)`, `PowerShell(devforgeai:*)`, `Read`, `Write`, `Edit`, `Glob`, `Grep`, `Agent`, `Skill`, `AskUserQuestion`, `WebSearch`, `WebFetch`, `TodoWrite`. Both grant spellings appear because the frontmatter uses one and the permission-rule syntax the other. The last two lines of the argv are exclusive: a case that declares `answers` takes the permission host, and a case that declares none takes `--permission-prompts none`, so a question it was never meant to reach is denied rather than left hanging.

`stream-json --verbose` is what makes tool calls, tool inputs and tool results visible to a grader; `--output-format json` returns the final string alone. `--include-partial-messages` adds `stream_event` lines carrying text a later `assistant` event repeats, and the reader skips them so no text is counted twice. The `result` text is appended last, so a grader reading the closing handoff block still finds it at the end. The raw stdout and stderr are written to `<logdir>/<case id>.stdout.txt` and `<case id>.stderr.txt`, beside the workspaces and never inside one, so no grader's tree walk counts them as a write by the model.

The session variables `CLAUDECODE`, `CLAUDE_PID`, `AI_AGENT`, `CLAUDE_EFFORT`, and `ANTHROPIC_API_KEY` are stripped from the child environment, so a nested run is not mistaken for the parent session. Under `--claude-config isolate` the child's `CLAUDE_CONFIG_DIR` points inside the workspace. `HOME` and `USERPROFILE` are left alone on purpose: the release binary resolves `~/.devforgeai/trust.toml` from those, and a redirected home would make every `trust verify` fail.

**Hooks during an eval.** By default the workspace settings carry the framework's own hook block from `hooks/settings.hooks.json` with `@@DEVFORGEAI@@` resolved to the binary and the two per-project tokens resolved as `hook install` resolves them, so the producer check, the gates and the Stop block run exactly as in a target project. That depends on a trust pin a human made outside Claude Code: the release binary reads `~/.devforgeai/trust.toml` and honours `DEVFORGEAI_HOME` only under the `test-home` cargo feature, which a release build does not carry. The runner therefore runs `devforgeai trust verify` once before the suite; on a failure it falls back to `--no-hooks` and prints the pin command, unless `--hooks` was passed explicitly, in which case it refuses to start. `--no-hooks` leaves only the AskUserQuestion answer handler registered, for a case that measures skill behaviour in isolation.

**Pre-seeded answers, and why a permission host is part of the runner.** A case may carry an `answers` map from a question's fixed header to the label to choose. The runner writes the map into the workspace as `eval-answers.json`, copies `evals/runner/answer_hook.py` into `<workspace>/.claude/`, and registers it as a `PreToolUse` handler on `AskUserQuestion` ahead of every other handler. The handler returns `permissionDecision: "allow"` with an `updatedInput` that echoes `questions` and adds `answers`, so the tool runs with no prompt and the model takes the case's answer through the same code path it takes a user's. A value is a label string, or a list of labels for a multi-select; `@first`, `@last`, `@n`, and `@all` select by position, and the key `"*"` answers any question the map does not name.

That handler needs the tool to exist, and under `claude -p` the tool exists only when a permission host does. Measured on `claude 2.1.268`, the same argv both times apart from the host flags, reading `system.init.tools`: with no host, 33 tools and no `AskUserQuestion`, so the skill notices and asks in prose instead; with `--mcp-config <workspace>/.claude/mcp-permissions.json --permission-prompt-tool mcp__dfa-permissions__approve`, 36 tools including `AskUserQuestion`, and `mcp_servers` listing `dfa-permissions: connected`. `--permission-prompts host` on its own does not add the tool, and `--allowedTools` permits without adding. The host is `evals/runner/permission_host.py`, a stdio MCP server exposing one tool, `approve`, which allows every request unchanged; the runner copies it into the workspace with its `mcp-permissions.json` for any case that seeds answers. With it in place the pre-seed hook fires live, and a `@first` resolves against the labels the run produced.

Per case, in order:

1. Create `<workdir>/dfa-eval-<case id>-<8 hex>` and treat it as the workspace root.
2. Resolve `setup.extends`, an optional key holding the `id` of an earlier case in the same file: that case's `setup.files` map is written first and this case's map over it, so a path in both takes this case's content. A chain resolves outermost first. A cycle, or an id no earlier line defines, is a usage error, exit 3.
3. Write every entry of `setup.files` as a file at that relative path inside the workspace, creating parent directories, UTF-8, LF endings. A value of the form `FIXTURE:<name>` is replaced by the bytes of `<skill>/evals/fixtures/<name>`; an absent fixture is a usage error, exit 3. A path escaping the workspace stops the case with status `error`.
4. When `setup.git` is present, make the workspace a git work tree with one commit. `true` does that alone; an object of the form `{"worktrees": [{"path": "wt/STORY-014", "branch": "devforgeai/STORY-014"}]}` registers each worktree up front, for a case whose subject is a Build run that expects one to exist.
5. Install `--skill` at `<workspace>/.claude/skills/<frontmatter name>/`, which is what makes the case prompt `/build STORY-014` resolve, and copy the `agents/` directory the skill's `agents.md` names into `<workspace>/.claude/agents/`.
6. Write `<workspace>/.claude/settings.json`: the answer handler always, the framework hook block unless `--no-hooks`. For a case that seeds answers, copy `permission_host.py` in beside it and write `mcp-permissions.json` naming it.
7. Invoke the argv above with the working directory set to the workspace, the prompt on stdin, and the timeout applied. A timeout records status `timeout`.
8. Assemble the transcript from the `stream-json` events, and pull `session_id`, `total_cost_usd`, `num_turns`, `permission_denials`, and `is_error` from the terminal `result` event. Each tool call is rendered into the transcript with its input, so a grader can assert on the call and not only on the prose.
9. Import `--graders` through `importlib.util.spec_from_file_location` under the module name `dfa_graders_<stem of --cases>`, resolve `expect.grader` with `getattr`, and call it as `grader(workspace, transcript, expect.args)`. A grader may read its own skill's `evals/fixtures/`, which is outside the workspace and is the framework's own baseline. A missing attribute records status `error` with evidence `grader_missing: <name>`. A raised exception records status `error` with the last line of the traceback as evidence.
10. Append one line to `--out`.
11. Remove the workspace unless `--keep` is set.

`results.jsonl` line schema:

```json
{ "schema": "devforgeai/eval-result/2",
  "run_id": "20260910T140211Z-a1b2c3d4",
  "case_id": "send-back-missing-req",
  "skill": "discovering-requirements",
  "model": "sonnet",
  "grader": "asserts_send_back",
  "status": "pass",
  "passed": true,
  "evidence": "transcript contains 'SEND BACK to Explore' and cites IDEA-002",
  "started_at": "2026-09-10T14:02:11Z",
  "finished_at": "2026-09-10T14:03:02Z",
  "duration_ms": 51004,
  "exit_code": 0,
  "workspace": "/tmp/dfa-evals-7b1/dfa-eval-send-back-missing-req-9f2a1c04",
  "workspace_kept": false,
  "transcript_bytes": 8122,
  "hooks": true,
  "answers": ["Which segment holds the problem?"],
  "log_stdout": "/tmp/dfa-evals-7b1/logs/send-back-missing-req.stdout.txt",
  "log_stderr": "/tmp/dfa-evals-7b1/logs/send-back-missing-req.stderr.txt",
  "session_id": "6f1c…",
  "total_cost_usd": 0.0412,
  "num_turns": 9,
  "permission_denials": [],
  "is_error": false,
  "tool_calls": { "Bash": 4, "Write": 2, "Agent": 1 },
  "tampered": "" }
```

`FIXTURE-SHA:<name>`, which would substitute a fixture's SHA-256 into `expect.args`, is not implemented: a case needing a digest inlines the literal, which is what `specs/04-constitute.md` already does. `FIXTURE:<name>` in `setup.files` is the one fixture reference form.

`status` is the closed enum `pass`, `fail`, `error`, `timeout`, `limit`, `tampered`. `passed` is `status == "pass"`. `evidence` is the second element of the grader's return tuple, cut to 2000 characters.

`limit` is a run the account's usage limit stopped before it finished, recognised from the text of the error result rather than from an exit code, because the process exits the way any other error exits. It is its own status and not an `error`: an `error` says the case or the skill is wrong, and a `limit` says nothing about either — rerunning it later is the whole remedy, and folding it into `error` would have a suite report defects it did not find.

A run that times out records no `total_cost_usd`: the result event carrying it never arrives, so the field stays null and the summary's cost total omits that case. The cost of a timed-out run is real and unrecorded, which is worth knowing when a suite's total is read as what it spent.

The summary printed to stdout after the last case:

```
cases 6  pass 5  fail 1  error 0  timeout 0  limit 0  tampered 0  duration 142.3s  cost $0.24
FAIL  send-back-missing-req  asserts_send_back: transcript has no SEND BACK line
results: skills/discovering-requirements/evals/results.jsonl
```

Seven counts, one per status plus the case total, then the wall clock and the summed `total_cost_usd` of every case that reported one. One line per case that did not pass follows, then the results path. The runner exits 0 when every case passed, 1 when a case failed, 2 when any case errored, timed out, hit the usage limit, or tampered, and 3 on a usage error: a `fail` is a measurement and the other four are not. `tampered` overrides whatever the grader returned, so a case that tampered and would otherwise have passed is not counted as a pass.

Runner exit codes, read by a human and by CI: 0 when every case passes, 1 when a case fails, 2 when a case errors or times out, 3 on a usage error. No phase gate consumes them.

## Decisions

Each entry is a choice this spec made where conventions §1 through §10 were silent, a proposed addition to the §4 surface, or a conflict inside the conventions and its resolution.

### Proposed additions to the §4 surface

1. `report ingest <subagent> <source>` is added. §7 names the call in the SubagentStop row and §4 omits it from the table; the hook has no other way to write a verifier block.
2. `hook run <event>` is added. The registration has to work under every shell and has to read JSON from stdin; a dispatcher subcommand does that with no shell JSON parser. It calls only §4 subcommands internally. The enum has seven arms: `session-start`, `prompt-expansion`, `pre-tool-use`, `trust-check`, `post-tool-use`, `stop`, `subagent-stop`. A settings command string never carries `--json`, because the harness reads a JSON object on stdout as a hook decision.
3. Flags added to existing §4 subcommands: `gate check --partial` and `--no-run`; `doc validate --allocate <prefix>`, `--producer-check`, `--all`, `--stdin-content`; `init --force`, `--from`, `--no-hooks`; `stack detect --dry-run`; `hook install --force`, `--claude-only`, `--git-only`; `story validate --scope`; `report show --check`; the global `--quiet`. `--partial`, `--allocate`, and `--producer-check` appear in §5 and §7 already; the rest are new.

### Conventions conflicts and their resolution

4. §9 names a "§Python" section of this spec; §11 forbids any H2 beyond its list. The runner contract is an H3 inside `## Evals`.
5. §11's section list presupposes a skill. `## Subagents`, `## Command`, and the `evals.json` and `cases.jsonl` parts of `## Evals` are filled with an explicit "none" and the reason, since the CLI invokes no subagent, ships no slash command, and is not evaluated by Python per §9.
6. §4 gives `doc validate` exit codes 0 and 1; §7 requires the PreToolUse hook to block, which needs hook exit 2. The subcommand keeps 0 and 1; the `hook run pre-tool-use` dispatcher maps 1 to 2. The same mapping applies to `design lint`.
7. §7 states that every hook exits 4 on a trust failure, and separately that exit 2 is what blocks. A hook that exits 4 does not block the action, and neither does exit 1. Resolution: `devforgeai trust verify` exits 4 in every failing case, and the dispatcher exits 2 with a decision object on the five events that can block — `prompt-expansion`, `pre-tool-use`, `trust-check`, `stop`, `subagent-stop` — and exits 4 with a `systemMessage` on `session-start` and `post-tool-use`, where no exit code blocks and JSON is the only channel to a reader.
8. §8 requires the handoff to show `Gate      TRUST FAIL`. `state.toml` `[last_gate].result` therefore carries the extra enum value `TRUST_FAIL`, which §5 does not define.
9. §11 asks `## Gate` for "the gates.toml entry for this phase". The CLI owns no phase, so the section carries the default file for all seven phases, which is the artifact the CLI actually owns.
10. The full subcommand reference lives under `## CLI calls`, and `## Workflow` carries the install and phase lifecycle. §11 describes `## CLI calls` as the subcommands a component invokes; for the CLI those are its own.

### Silences filled

11. The registered-verifier register, which §7 and §10 reference without locating, is `[[verifier]]` in `config.toml`, mapping subagent name to report field, phase, unit word, and a required flag.
12. The subagent stdout contract the CLI parses is `devforgeai/verifier/1`, defined in `## Subagents`.
13. Compiled-in minimums have two halves: required check kinds per phase and numeric floors. A `gates.toml` that omits a required kind, marks it `warn`, or sets a value below a floor is rejected with `DFA-E303` rather than clamped, since §8 calls it schema validation.
14. Degradation is one flag, `degraded`, set by `stack detect`. With it set, the four command-running kinds — `tests_pass`, `coverage_min`, `lint_clean`, `complexity_clean` — evaluate to `skip` and the build floors do not apply. With it clear, an empty command string fails the check. `skip` has exactly four producers: an explicit `skip_when` or a false `required_when`, this flag, `--no-run`, and the `not_implemented` stub case, which is a skip that refuses the next `gate require` rather than one that passes. Every other absent, null, unparsable, or wrongly typed input is a `fail` with a named code.
15. The check-kind enum is closed and listed in `## Outputs`; Decision 63 takes it to thirty.
16. A check may override the gate's `on_fail`, so one failing check can produce `SEND BACK` inside a gate that otherwise reports `FAIL`.
17. Path and document values inside a check may carry the tokens `{id}` and `{phase}`.
18. The v1 ecosystem list is closed at seven: rust, node, python, go, dotnet, jvm, ruby, plus the degraded state for none. jvm ships an empty `lint_command`, because Maven and Gradle share no linter; node commands exist only when the matching `package.json` script exists; ruby lint exists only with `.rubocop.yml`.
19. Accepted coverage formats are lcov, cobertura, jacoco, go-cover, and the `devforgeai-json` fallback that a project writes itself. Detection assigns the first four; a human assigns the fallback.
20. Coverage layers are four, `domain`, `application`, `infrastructure`, `interface`, with default thresholds 95, 85, 80, 70 and the glob lists in `## Outputs`. Files matching no layer are `unassigned` and reported, not failed, unless `unassigned_policy = "fail"`.
21. Skill names used by `produced_by` and the producer check are `devforgeai-<phase>` for the seven phases, plus `designing-interfaces` and `improving-framework`. Each skill spec repeats its own name; a divergence is a defect in that spec, not in the CLI.
22. The status enum per doc type is fixed in the doc-type table. §5 defers it to each doc-type's spec; a skill spec may add a value by amending that table, and removal of a value is refused.
23. Story dependencies live in the body under the H2 `## Dependencies`, because §5 fixes the frontmatter keys and leaves no room for a `depends_on` key.
24. `sprint.yaml` carries `stories`, a sequence of mappings with `id` and `status`; the pre-push hook reads them with awk, relying on `id` preceding `status` in each item.
25. An AC is testable when it matches `Given … When … Then …` or contains one of seven verbs, listed under `story validate`. No other form passes.
26. The `context audit` contradiction check is textual: `subject: value` bullets compared pairwise after normalisation, against accepted ADRs only. It detects no semantic contradiction, and that limit is deliberate, since §1 keeps judgement in skills.
27. An ID is defined by a frontmatter `id`, a heading that starts with it, or a YAML item key; every other occurrence is a reference. §5 fixes the ID shape and not this distinction.
28. Handoff details §6 leaves open: the `Done` counts per phase, the slug taken from the document's first H1, the 100-column line cap, the ellipsis rule, the Found budget computed against the twelve-line cap, dropping the optional `Then` line when the budget reaches zero, the `Full report:` line exempt from the column rule because §6 writes it that way, the `Next` and `Then` transition table, and the `TRUST FAIL` rendering.
29. Error identity is `DFA-E<nnn>` and `DFA-W<nnn>`, banded by domain, with the verbatim text in the error table. §4 fixes only the exit codes.
30. `--json` output is one envelope with `schema`, `command`, `ok`, `exit`, `degraded`, `project`, `at`, `data`, `errors`, `warnings`, printed on failure as well as success.
31. Every file write is a temp-file rename in the destination directory; `.claude/settings.json` is copied to a timestamped backup first.
32. `init --from` defaults to the `framework_path` of the matching `[[pin]]` in `trust.toml`, which ties installation to the binary a human already pinned.
33. Brownfield analysis derives `tech-stack.md`, `source-tree.md`, and `dependencies.md` from manifests and the tree, and stubs `coding-standards.md`, `architecture-constraints.md`, and `anti-patterns.md`. All six carry `status: draft` and the placeholder `id: CON-000`; the Constitute skill fills the stubs and allocates real IDs.
34. Hook registrations are exec form: `command` carries the `@@DEVFORGEAI@@` token, replaced at install time with the bare name when it resolves on `PATH` and with the absolute path otherwise, and `args` carries the argument vector, so no shell parses the line and a path holding spaces survives. Two further tokens are resolved per project at install: `@@TEST_COMMAND@@` from each `[[stack]].test_command` and `@@VERIFIERS@@` from the `[[verifier]]` names. A handler whose token resolves to nothing is dropped rather than written as a filter that never fires.
35. Hook timeouts are 900 seconds for `stop` and the shell `post-tool-use` entry, 600 for the three `PreToolUse` groups, 120 for `session-start`, 60 for the write-tool `post-tool-use` entry and `subagent-stop`, and 30 for `prompt-expansion`, which the harness caps at 30 in any case. Every shell matcher is `Bash|PowerShell`, because on Windows the PowerShell tool is primary and a `Bash`-only matcher never fires.
36. For a PreToolUse on Edit the dispatcher sends no content, because `new_string` is a fragment rather than a document; the producer check then runs against the path and the file on disk. Write sends the whole `content` with `--stdin-content`.
37. The crate uses clap 4.5, serde 1, toml 0.8 with toml_edit 0.22, serde_json 1, serde_yaml_ng 0.10, sha2 0.10, globset 0.4, walkdir 2, quick-xml 0.36, time 0.3, and thiserror 2. `serde_yaml_ng` is chosen as a maintained fork of the deprecated `serde_yaml` with the same API; swapping it for another YAML crate changes no interface in this spec.
38. The test-only cargo feature is `test-home`. It enables `DEVFORGEAI_HOME` and `DEVFORGEAI_NOW`. With default features both variables are read by no code path, and `cli/DIGEST` is computed over a default-features release build, so a binary that honours the override has a different digest and fails `trust verify` by construction.
39. `cli/REVISION` line 1 is `git rev-parse HEAD` when the framework root is a git work tree and the literal `unversioned` when it is not; line 2 is a SHA-256 over every file under `cli/` except the depth-1 directories `target` and `.git` and the files `cli/REVISION` and `cli/DIGEST`, each contributed as path, a zero byte, an eight-byte big-endian length, then bytes, in ascending path order. `cli/DIGEST` is the SHA-256 of the `cargo build --release --locked` artifact that ships. Both are written by `devforgeai trust digest --framework <root>`, a hidden subcommand printing `REVISION`, `SOURCE`, and `DIGEST` lines. It is an addition to conventions §4 on the same ground as `hook run`: the files have to be produced by the same code path that reads them back, and a release script reimplementing the walk drifts silently the first time an exclusion changes.
40. `trust verify` compares the source digest only while a Claude session is active, so a developer may edit `cli/` freely outside a session, which is where `trust pin` runs.
41. `gate check` writes `reports/<id>-<phase>.yaml` for every phase, not for Build alone; §4 says so and §5 lists a report document for Build and Verify only, so the other five reports are CLI-owned artifacts that no skill consumes.
42. The Python runner writes `<workspace>/.claude/settings.json` carrying the `AskUserQuestion` answer-seed handler always, and the framework's own hook block by default, so the producer check, the gates and the Stop block run during an eval as they do in a target project. Hooks-on depends on a real `trust pin` made by a human outside Claude Code, because a release binary honours no `DEVFORGEAI_HOME` redirect; the runner runs `trust verify` once before the suite and falls back to `--no-hooks` with the pin command printed when it fails. `--no-hooks` leaves the answer handler alone, for a case that measures the skill in isolation.

43. The compiled floors bind numbers in two files: `config.toml` `[[layer]].coverage_min` and `[coverage].overall_min`, checked at config load while `degraded = false`, and `gates.toml` `verifier_pass.min_ratio`, checked at gate load. A `[[layer]]` table deleted from `config.toml` keeps its default threshold, so removal lowers nothing.
44. `build-report`, `qa-report`, `release`, and `reflect-report` permit top-level keys beyond the §5 seven, because the CLI's own reports carry gate, coverage, and verifier payload and the `pre-commit` hook validates them. The other nine doc types refuse extras with `DFA-E204`.
45. `phase set` writes the `status` frontmatter value of the story it activates, by the table in its section, because a story's `ready` to `building` to `built` to `released` transitions happen inside phases that do not own the story document and §2 keeps the transition out of skill prose. This extends §4's one-line description of `phase set`, which names `state.toml` alone.
46. The Verify phase has two report files: `reports/<id>-qa.yaml`, the Verify skill's §5 document, and `reports/<id>-verify.yaml`, the CLI's gate report that holds the ingested verifier block and that the handoff cites.

### Reconciliation pass against the skill specs

49. Skill names follow §4b and §3: `exploring-ideas`, `discovering-requirements`, `establishing-context`, `planning-work`, `implementing-stories`, `validating-quality`, `releasing-software`, `designing-interfaces`, `improving-framework`. This replaces the `devforgeai-<phase>` names of Decision 21, which `specs/08-design.md` Decision 10 flagged. `devforgeai-cli` stays as the `produced_by` of CLI-written reports, since the CLI is not a skill and §4b names no value for it. The coverage fallback format `devforgeai-json` and the `devforgeai/<doc-type>/1` schemas are unrelated strings and are unchanged.
50. The pipeline keys on four id kinds, not one: Explore, Discover, and Constitute on `IDEA-nnn`, Plan on `EPIC-nnn`, Build and Verify on `STORY-nnn`, Release on `vX.Y.Z`. `requirements.yaml` is one document per project carrying `id: IDEA-nnn` and holding every epic, and the six context files are project singletons. A project that skipped Explore gets its `IDEA-nnn` from `doc validate --allocate IDEA` inside Discover. The `gate require` table resolves each predecessor subject by that chain.
51. `phase set` refuses only when a predecessor gate exists and is not `PASS`, which is `specs/03-discover.md` Decision 3's requested clarification. `phase set explore` and `phase set discover` at an entry point therefore succeed with no report on disk.
52. Five subcommand additions from the skill specs are adopted with their argument grammar verbatim: `explore prune --id <IDEA-nnn>`, `phase set <phase> --id <id> --remedy <ID,ID>`, `doc accept requirements --id <IDEA-nnn>`, `doc reopen requirements --id <IDEA-nnn> --ids <ID,ID> --from <plan|constitute|design>`, and `design lint --tokens`. `--remedy` is generalised from Explore's flow list to the receiving phase's remedy field, which is how `[constitute].remedy_con` is written. Together with `hook run <event>` and `report ingest`, the §4 surface gains seven entries.
53. The check-kind enum is closed at twenty-one after folding in the skill-proposed kinds. `doc_exists` folds into `file_exists`; `path_absent` becomes `file_exists` with `absent = true`; `status_is` is dropped in favour of `field_in_enum` on the `status` field; `list_length_between` and `collection_min_len` become `length_between`; `reference_resolves` becomes the `from` and `to` form of `ids_resolve`; `field_set` becomes the `path` form of `fields_present`; Constitute's `run` plus `expect` form becomes the `context_audit` kind and its `metric` plus `expect` form becomes the new `report_metric` kind, whose metric path is rooted at the report, so `architecture_reviewer.blocking_findings` is written `verifiers.architecture_reviewer.payload.blocking_findings` — `verifiers.<report_field>` locates the block, and `payload` is where an agent's own fields sit inside it.
54. The `gates.toml` container shape stays this spec's, because the four skill specs propose three different shapes: `[gates.<phase>]` with `[[gates.<phase>.checks]]` and a `name` key, `[<phase>]` with `[[<phase>.check]]` and an `id` key, and `[gates.<phase>]` with `run`, `metric`, and `expect`. One shape is `[[gate]]` with `[[gate.check]]`, `id`, `kind`, `severity`, `on_fail`, and `message`. The checks themselves are unchanged, which is what `specs/03-discover.md` Decision 16 asked for.
55. A check gains four condition keys, `skip_when`, `required_when`, `null_when`, and `empty_when`, and a `message` with the placeholders `{id}`, `{phase}`, `{value}`, and `{limit}`. A gate gains `document`, which supplies the default `path` of its checks. Explore's gate needs all five.
56. The gate-level `verifier` key of `specs/02-explore.md` and the `id_prefix`, `requires_phase`, `predecessor`, and `send_back` table of the other two are dropped. Registration lives in `config.toml` `[[verifier]]`, the predecessor is `requires`, the report path is fixed by §3, and a send-back is a check with `on_fail = "send_back"`.
57. `state.toml` takes the `[current]` shape: `[current].phase` and `[current].id` are the pair every skill reads, `[active]` keeps one id per phase for predecessor resolution, and a phase-scoped table carries a phase's own fields. `[explore]` holds Explore's six fields and `[constitute]` holds `remedy_con`. Discover's `phase` and `active_id` are `[current].phase` and `[current].id`. An unknown key in a phase table is `DFA-E108`.
58. `tokens.json` colour leaves are `{light, dark}` objects and every other group's leaves are strings; `design lint` resolves a literal against the union of both themes and names the token, not the theme. `design` is a valid `--phase` value for `handoff`, `report ingest`, `report show`, and the report path, and for no gate command. A `/design` run prints Design's block and then the Stop hook's block for `[current].phase`. The `Phase` line renders Design's number as an em dash.
59. `design lint` exits 0 without reading a file under `.explore-prototype/` or `.devforgeai/explore/mockups/`, which `specs/02-explore.md` Decision 3 asks for; both prefixes are in the default `[frontend].exclude`. `init` appends `.explore-prototype/` to the target project's `.gitignore`, per that spec's Decision 4.
60. A context file's `id` is its filename stem and its `schema` is `devforgeai/context-<stem>/1`, so `doc validate` selects the heading list from the schema. `AP-nnn` joins the allocatable prefixes. The six files are writable by `establishing-context` alone through the producer check, which is why `init --analyze` writes that value rather than the CLI's. `context audit` is the eight checks CA-1 to CA-8, and CA-2 exits 1 while any of the six carries `status: draft`.
61. ID uniqueness is scoped per `schema` value, per `specs/02-explore.md` Decision 9, so `brief.md` and `decision.yaml` may share one `IDEA-nnn`. `TOKEN-<name>` is a token reference and not an ID-index entry, per `specs/08-design.md` Decision 8.
62. The eval runner redirects `CLAUDE_CONFIG_DIR` into each workspace and leaves `HOME` and `USERPROFILE` alone, because the release binary resolves `~/.devforgeai/trust.toml` from those and reads `DEVFORGEAI_HOME` only under the `test-home` feature. This replaces the `DEVFORGEAI_HOME` redirect this decision first proposed, which a release binary ignores. Prior state travels in `setup.files`, in the grader `args`, and in `setup.git` for a case whose subject needs a work tree.

### Second reconciliation pass

63. Nine check kinds join the enum, taking it to thirty: `no_cycle` (07), `complexity_clean`, `antipattern_clean`, `files_declared` (06), `release_stories`, `deploy_manifest`, `docs_cover` (09), `yaml_cites`, `no_threshold_decrease` (10). Each wraps a parse or an exit code and none parses a tool's output beyond the coverage formats already carried.
64. Eight subcommands join the surface, taking it to twenty-eight: `story files` with `--check`, `--list`, and `--diff`; `story list`; `report note`; `report aggregate`; `worktree` with `ensure`, `list`, and `remove`; `commit`; `config get`; `antipattern scan`. Every grammar is the owning spec's verbatim.
65. Path rooting inside `gates.toml` is one rule: a relative path resolves against `.devforgeai/`, except one beginning with `.`, which resolves against the project root. `explore/decision.yaml`, `.devforgeai/explore/decision.yaml`, and `.explore-prototype` are all correct as their specs write them, and the default file is consistent under one reading.
66. Error codes are allocated by this spec, so four skill-proposed numbers move to free ones: Build's `DFA-E226`, `E240`, `E241`, `E242` become `DFA-E270` (anti-pattern match), `E271` (not a git work tree), `E272` (worktree overlap), `E273` (worktree dirty), because the first four are already context audit CA-7 and the three `design lint` codes. Release's secret-pattern `DFA-E330` becomes `DFA-E343` and its version rules take `DFA-E217` and `DFA-E218` rather than `DFA-E220`, which is a context audit code. Reflect's `DFA-E320` through `E323` become `DFA-E345` through `E348`, since `E320` to `E329` are gate and phase-set codes. `DFA-E340`, `DFA-E413`, `DFA-E421`, `DFA-E430`, and `DFA-W243` are adopted as proposed. Every code in the table is unique.
67. `[active].plan` holds a `SPRINT-nnn` and the plan report is `reports/<SPRINT-nnn>-plan.yaml`, per `specs/05-plan.md` Decision 4. The predecessor chain follows: `gate require build <STORY-nnn>` resolves the sprint whose `stories[]` or `deferred[]` lists the story, and a story in no sprint is `DFA-E013`. `/plan` accepts an `EPIC-nnn` or a `SPRINT-nnn` as `$1`, and `gate require plan` resolves the epic through `sprint.yaml` `epic` when given a sprint.
68. The story status enum loses `verified`, leaving `draft`, `ready`, `building`, `built`, `released`. `phase set release` writes `built` to `released`. No actor wrote `verified`: `phase set verify` writes `built`, and the producer check blocks a Verify-phase write to Plan's document. The `verify-story-status` check keeps `values = ["built", "verified"]` byte-identical to `specs/07-verify.md`, where the second value is now unreachable. `specs/05-plan.md` Decision 2's sentence naming Verify as the writer is the one line this contradicts.
69. `gate.send_back_to` is resolved from the blocking findings' id prefixes for every gate, not for the plan gate alone: `REQ` to discover, `CON` and `AP` to constitute, `UI` to design, `AC` to plan, `FIND` to build, ties to the earlier phase in §5 order, and an empty or unmapped set to the gate's static value. The verify gate adds the `category: spec-gap` rule of `specs/07-verify.md` Decision 7.
70. Handoff rendering gains two clarifications from `specs/06-build.md`: `<n> checks` counts every evaluated `[[gate.check]]` entry at either severity, including skips; a tie at the lowest verifier ratio is broken by `[[verifier]]` order. The `build` PASS `Then` line is omitted while `[active].release` is empty, and the `verify` PASS `Next` line is the next `ready` story of the sprint before it is a release version.
71. `config.toml` gains six phase tables, `[explore]`, `[plan]`, `[build]`, `[verify]`, `[release]`, `[reflect]`, and one `[[stack]]` key, `complexity_command`. `stack detect` preserves all of them across runs, the rule that already governs `[[layer]]`, `[coverage]`, and `[[verifier]]`.
72. The `[[verifier]]` registry ships the twenty registrations of `specs/11-subagent-catalog.md` Decision 1, listed in `## Outputs`. One `name` binds to one `phase`, which is why `story-ac-verifier` (build) and `ac-compliance-verifier` (verify) are separate names for related work.
73. `.devforgeai/state.toml` is untracked and every other path under `.devforgeai/` is tracked, which is what lets two Build worktrees hold different `[active].build` values without a merge conflict. `init` writes both that line and `.explore-prototype/` into `.gitignore`.
74. `verifier_pass` reads `total: 0` as ratio `1.0`; `coverage_min` with `source = "read"` parses the coverage artifact named by `coverage_paths` and runs no command, and an absent artifact is `DFA-E312` rather than a skip. `gate check --partial` evaluates four kinds, the fourth being `complexity_clean`.
75. `reflect` joins the `[[gate]].phase` enum with a compiled-minimums row and no floors, and is accepted by `handoff`, `report show`, `doc validate`, `gate check`, and `gate require`. `gate check --phase reflect` takes a `YYYY-MM-DD` `--id` and has no `[active]` key to default from, so omitting it is `DFA-E011`. The `reflect-report` doc type is the one type whose `id` is a date, its schema is `devforgeai/reflect-report/1`, and its status enum is `draft` and `final`.
76. `report note` is the one path by which a skill contributes to a CLI-owned report. `produced_by` stays `devforgeai-cli` for `build-report`, `qa-report`'s CLI sibling, and every per-phase gate report; the note lands under its own top-level key and changes no §5 key.
77. `cases.jsonl` gains `setup.extends`, a plain merge of an earlier case's `setup.files` before this case's, resolved outermost first, with a cycle or an unknown id a runner usage error. `FIXTURE:<name>` in `setup.files` is substituted from `<skill>/evals/fixtures/<name>`. `FIXTURE-SHA:` is declined: a case needing a digest inlines the literal.
78. `story validate` runs ten checks, the four added ones being epic requirement coverage (`DFA-E235`), the one-`Covered by`-cell rule (`DFA-E236`, `DFA-E245`), file-set overlap inside a sprint (`DFA-E237`), and UI spec resolution (`DFA-E238`). A Markdown list item beginning `<PREFIX>-<nnn>:` defines that ID in the index, which is what makes `AC-nnn` resolvable.
79. The `pre-tool-use` arm calls `story files --check <path>` for a write outside `.devforgeai/` while `[current].phase` is `build`; exit 1 maps to hook exit 2, the mapping Decision 6 already fixes for `doc validate --producer-check` and `design lint`.

### Blockers

80. The SubagentStop payload is settled and is no longer a blocker. The event carries `agent_type`, `agent_id`, `last_assistant_message`, `agent_transcript_path`, and `stop_hook_active`. The dispatcher reads `agent_type` for the name and `last_assistant_message` for the envelope, falling back to the last assistant message of `agent_transcript_path` — the subagent's own transcript, never the parent session's `transcript_path`, whose last assistant message is the orchestrator's text. With neither present it exits 0 with `DFA-W411` and the verify gate fails later with `DFA-E316`, which is a visible failure rather than a silent pass.
81. No other requirement of this spec needs a primitive outside the §2 list. Every subcommand is a process invocation, every hook is a command string in `.claude/settings.json`, and every file is under the project root or the user's home directory.

### Third reconciliation pass, against the built system

82. The handoff is delivered in the Stop hook's `systemMessage` and never on plain stdout, because Claude Code writes hook stdout to the debug log on every event but `UserPromptSubmit`, `UserPromptExpansion`, `SessionStart`, and `PostModelSwitch`. The four Stop cases and the block budget are in `## CLI calls` under `hook run`. The rule that let the model write one sentence before the block is deleted: the model writes no part of the close of a phase.
83. The fail-closed trust set is three registrations rather than one: the `trust-check` `PreToolUse` arm over six tools, the Stop block, and `UserPromptExpansion` over the nine skill names. The previous set stopped writes through two tools and stopped nothing else, which left a session on an unpinned binary free to run the test suite, spawn every subagent, and commit through a shell.
84. `commands/` is deleted and each `SKILL.md` carries the frontmatter and the preamble. A skill and a command file of the same name both produce `/name` and the skill wins, so the two-file shape had one live file and one shadow file per entry point. The collapse buys `disable-model-invocation` and frontmatter `hooks`, neither of which a command file has, and it changes no user-facing string, because the slash name comes from the skill.
85. `init` writes a CLAUDE.md section between markers, 31 lines with the markers and 29 between them. A target project otherwise has nine skills whose descriptions are out of context until invoked, a hook layer that enforces silently, and no standing statement that any of it exists.
86. The §2 ceremony pattern is scoped by six clauses and implemented once, in `scripts/ceremony_scan.py`. Both halves are matched case-insensitively, and the scoping rather than the case is what releases the ordinary English uses of those words: a case rule would let `You must never skip this` through. A YAML scalar is not instruction prose, which restores the MoSCoW `priority` enum the rename had replaced.
87. `AskUserQuestion` is removed from every agent file and from the catalog's permitted-tool list, because Claude Code removes the tool from every subagent whatever its `tools` list holds. The questions move to the invoking skill. A registered verifier prints exactly one object, the envelope with the agent's own fields nested under `payload`; `warn` findings never lower `passed`; a review agent reports every finding and the CLI filters.
88. An agent may read a tool's numeric output only to copy it into `payload` verbatim. Every threshold is applied by a `gate check` check kind reading `config.toml` and `gates.toml`. This resolves the conflict between conventions §2, which forbids an agent interpreting tool numbers, and `specs/07-verify.md`, which placed that reading in two agents: the agents keep the reading and lose the judgment.
89. `doc validate --allocate` reserves the id on disk under `.devforgeai/.allocated/<ID>` before printing it, so two worktrees sharing one `.devforgeai/` cannot be handed the same number. The reservation directory matches no doc-type row, so the producer check skips it.
90. `gate require` enforces a per-arm id shape before it loads the gate, and `reflect` is the one arm that imposes none, because its subject is a window rather than an id.
