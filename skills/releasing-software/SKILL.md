---
name: release
description: Phase 6 of DevForgeAI, run by /release. Turns every story at status built into one named version - it resolves a deployment platform from a five-value enum, writes that platform's manifests under the configured deploy root, writes the API reference, the user guide, and the architecture note under the configured docs root, copies the brand assets, and writes .devforgeai/releases/vX.Y.Z.yaml for Reflect to read. Reach for it whenever /release is typed, whenever a version of the form vX.Y.Z is being cut, whenever deploy manifests, a rollback note, a release gate workflow, release notes, or a published documentation set are being written for this framework, and whenever releases/vX.Y.Z.yaml, a deferral that might block deployment, or a Release send-back to Verify appears in a report or a handoff.
argument-hint: vX.Y.Z [--resume]
allowed-tools: Bash(devforgeai:*), PowerShell(devforgeai:*), Read, Write, Edit, Glob, Grep, Agent, AskUserQuestion
disable-model-invocation: true
---

!`devforgeai story list --status built --json`

# releasing-software

Release turns the set of stories Verify passed into one named version. It writes manifests, documentation, and one release file; it applies no deployment, holds no credential, and runs no build of its own. Every command it would otherwise name comes from `.devforgeai/config.toml` `[release]`. It edits no story, no QA report, no ADR, and no context file — a defect in any of those leaves as a SEND BACK to Verify citing ids.

## Entry

`/release` invokes this skill. Arguments:

| Form | Run kind | Meaning |
|---|---|---|
| `/release vX.Y.Z` | fresh | assemble every story at `status: built` into that version |
| `/release vX.Y.Z --resume` | resume | continue the run the last send-back stopped, narrowed to the ids that report cites |

`$1` is the version. A `$1` that does not match `^v[0-9]+\.[0-9]+\.[0-9]+$` stops the run with one `Blocked` line giving the accepted form `vX.Y.Z`. There is no `--remedy` form: Reflect is the only phase downstream and its `REC-nnn` output sets no gate, so nothing arrives here as a send-back.

The preamble line at the head of this file runs before the body loads:

- `devforgeai story list --status built --json` — `{"count":n,"stories":[{"id","status","title","path","consumes"}]}`. That list is the release set; the call exits 0 at a count of zero, which step 4 handles. Exit 1 on `DFA-E231` means `.devforgeai/stories/` is absent and the body does not load. The line allocates no id — this phase allocates none, because the version is `$1` and every other id it names was allocated upstream.

Read from disk as the run needs them: `.devforgeai/config.toml` (`[release]` ten keys, `[[stack]].id` and `[[stack]].source_roots`, `[[verifier]]`), `.devforgeai/state.toml` (`current_phase`, `[active].verify`, `[active].release`), `.devforgeai/stories/STORY-nnn.md` and `stories/sprint.yaml`, `.devforgeai/reports/STORY-nnn-qa.yaml` and `reports/STORY-nnn-verify.yaml`, `.devforgeai/context/tech-stack.md`, `source-tree.md`, `dependencies.md`, `architecture-constraints.md`, `.devforgeai/adr/ADR-nnn.md`, `.devforgeai/brand/tokens.json` and `brand/logo.svg`, `.devforgeai/requirements.yaml` for `requirements[].statement`, and the `.devforgeai/releases/v*.yaml` names for `previous_version`.

## Workflow

Sixteen steps. A `--resume` run enters at step 1, and step 3 narrows steps 4 through 10.

**1. Establish the run — model.** Input: `$ARGUMENTS`, `.devforgeai/state.toml`. Output: the version `$1` and a run kind of `fresh`, or `resume` when `$ARGUMENTS` holds `--resume`. Failure path: `$1` fails the version pattern; the run stops with one `Blocked` line naming the form.

**2. Activate the version — CLI.** Run `devforgeai phase set release --id $1`. That sets `state.toml` `current_phase` to `release` and `[active].release` to `$1`, which is where `report ingest` resolves its target report at step 6. The release file does not exist yet, so the story-status half of the call writes nothing and returns `DFA-W210` at exit 0. Failure path: exit 1 on `DFA-E300` means `gates.toml` holds no `release` gate; the run stops and the stderr line reaches the model. Exit 1 on `DFA-E320` means a story in the set has no verify gate at `PASS`; the run stops with one `Blocked` line naming `/verify STORY-nnn` for the story the stderr names, and no release file is written.

**3. Read the last verdict — model.** Runs on a `resume` run alone. Read `.devforgeai/reports/$1-release.yaml` and take the `STORY-nnn` and `FIND-nnn` ids of its `findings[]`. Those ids narrow step 4's set to those stories and step 10 to the pages those stories touch. Leave every file already written under `[release].deploy_root`, `[release].docs_root`, and `.github/workflows/` byte-identical. Failure path: the report is absent or its `status` is not `send_back`; the run proceeds as `fresh`.

**4. Assemble the release set — CLI, model.** Input: the preamble's `story list` stdout. Read each listed `stories/STORY-nnn.md` for its first H1, its frontmatter `consumes`, and its `status`. Then read `stories/sprint.yaml` `stories[]` and record any `{id, status}` pair whose status is `built` and whose id the CLI list omits — the sprint's own list is the cross-check on the walk. Output: the release set ascending by id, each with `title`, `requirements` (the `REQ-nnn` of `consumes`, ascending), and the two report paths. Failure path: the set is empty; the run stops with `Blocked   you: no story is at status built; run /verify STORY-nnn first` and writes no release file. A story whose `devforgeai doc load story STORY-nnn` returns `DFA-E200` leaves the set and adds one `open_questions` line naming the id.

**5. Read the verify verdicts — CLI.** Run `devforgeai report show STORY-nnn verify` once per story and take `gate.result` into `stories[].verify_result`. Failure path: exit 1 on `DFA-E400`, the report is absent; record `verify_result: FAIL` and carry the story into step 6, which is what produces the send-back.

**6. Audit the deferrals — subagent `deferral-auditor`.** Pass the version, the `FIND-nnn` entries each `reports/STORY-nnn-qa.yaml` marks deferred with their stated reason, each story's AC texts, the `## Approved dependencies` rows of `context/dependencies.md`, and `platform.target` when a `resume` run or a config value already fixes it. The agent returns the `devforgeai/verifier/1` envelope; the SubagentStop hook runs `devforgeai report ingest deferral-auditor -`, which writes `verifiers.deferrals` into `.devforgeai/reports/$1-release.yaml`. Failure path: a story whose QA report is absent contributes one finding of `kind: missing_report` at `severity: block`, and the audit continues over the rest.

**7. Resolve the platform — model, CLI, user.** Read `config.toml` `[release].platform`. A value other than `""` is `platform.target` with `source: config` and `marker: ""`. When the key is absent or `""`, test the marker list below in order and take the first match, with `source: detected` and `marker` set to the matched path. When no marker matches, one `AskUserQuestion` call with the header `Platform` and the five options `Kubernetes`, `Compose`, `GitHub Actions`, `VPS`, `None` supplies the value with `source: asked`; write that value back to `[release].platform` in `.devforgeai/config.toml` so the next release reads it from the file. Failure path: a free-text answer matching no label re-asks once, then the run takes `none` and adds one `open_questions` line naming the header. The PreToolUse producer check covers every path under `.devforgeai/`, `config.toml` included; a refusal leaves `platform.target` at the answered value for this run alone and adds one `open_questions` line naming the key that was not written.

| Order | Marker glob | Gives |
|---|---|---|
| 1 | `**/kustomization.yaml`, `**/kustomization.yml`, `**/Chart.yaml` | `kubernetes` |
| 2 | `docker-compose.yaml`, `docker-compose.yml`, `compose.yaml`, `compose.yml` at the project root | `compose` |
| 3 | `.github/workflows/deploy.yml`, `.github/workflows/deploy.yaml` | `github-actions` |
| 4 | `<deploy_root>/vps/deploy.sh` | `vps` |

**8. Write the deployment manifests — subagent `deploy-manifest-writer`.** Pass `platform.target`, `[release].image_name`, `[release].deploy_root`, `[release].build_command`, `[release].package_command`, `[release].service_port`, the `[[stack]].id` and `source_roots` values, the `## Constraints` rows of `context/architecture-constraints.md`, the version, and the template paths for the target. It returns `devforgeai/deploy-manifest/1` and writes the files the platform's reference file lists, which fill `deploy.manifests[]`. On `platform.target` of `none` the agent is not invoked, `deploy.manifests` is `[]`, and `deploy.rollback` is `""`. Failure path: the agent returns `written: []` with a `reason`; write the platform's template files with `@@IMAGE@@`, `@@PORT@@`, and `@@NAME@@` substituted, and record the reason in `open_questions`.

**9. Write the gate workflow — model.** Runs when `config.toml` `[release].ci` is `github-actions`, for every value of `platform.target` including `none`. Write `templates/ci/devforgeai-release.yml` to `.github/workflows/devforgeai-release.yml` with `@@VERSION@@` replaced by `$1`, `@@DEPLOY@@` by `[release].deploy_root`, and `@@DOCS@@` by `[release].docs_root`, then fill `deploy.ci_workflow` and `deploy.ci_check_name` with that path and the job name `devforgeai-release-gate`. That job name is the string a repository administrator adds as a required status check. Failure path: `[release].ci` is `none`; the file is not written and both keys are `""`.

**10. Write the documentation — subagents `api-doc-writer` and `guide-writer`, in parallel.** Pass `api-doc-writer` the output lines of `[release].api_symbols_command`, each `<kind>\t<symbol>\t<path>`, the `[[stack]]` tables, `[release].docs_root`, and the version. Pass `guide-writer` the release set with each story's ACs, every `adr/ADR-nnn.md` at `status: accepted`, the four context H2 sections `## Languages` of `tech-stack.md`, `## Layers` of `source-tree.md`, and `## Approved dependencies` and `## License policy` of `dependencies.md`, `[release].docs_root`, and the version. They fill `docs.api`, `docs.guide`, `docs.architecture`, `docs.api_symbols`, and `docs.api_documented`. Then write `<docs_root>/README.md` from `templates/docs/README.md`, copy `.devforgeai/brand/tokens.json` and `brand/logo.svg` to `<docs_root>/brand/`, and fill `docs.index` and `docs.brand`; `docs.brand` is `[]` when `.devforgeai/brand/` holds neither file. `references/docs-layout.md` carries the four directories and the fixed H2 list per file. Failure path: `[release].api_symbols_command` is `""`; `api-doc-writer` writes `docs/api/index.md` with an empty `## Symbols` table, `docs.api_symbols` and `docs.api_documented` are both `0`, and the gate's `docs_cover` check records `SKIP` with `reason: no_api_symbols_command`.

**11. Write the release file — model.** Input: everything steps 4 through 10 produced, plus `previous_version` taken as the highest `v<X>.<Y>.<Z>` among the `.devforgeai/releases/v*.yaml` names other than `$1.yaml`, or `""` when the directory holds no other release. Write `.devforgeai/releases/$1.yaml` from `templates/release.yaml` with `status: draft` and `released_at: ""`, all sixteen top-level keys in the `## Documents` order. `stories[]` takes the entry shape the template carries — `id`, `title`, `status`, `qa_report`, `verify_report`, `verify_result`, `requirements`, `deferrals`, all eight keys per entry — with one entry per story in the step 4 release set, ascending by id; the template's `STORY-000` entry is the shape rather than a row to keep. The gate's `fields_present` check reads those keys, and an entry short one of them is `DFA-E332` naming the story and the key. Failure path: the PostToolUse `doc validate` result reaches the model after the write as `hookSpecificOutput.additionalContext`, naming a key; rewrite that key and write the file again.

**12. Move to released — model.** Edit the file from step 11 to carry `status: released` and `released_at` at the current UTC instant in RFC 3339. Failure path: none; the gate at step 13 decides whether the version stands, and a FAIL leaves the file at `released` with `signoff.gate` carrying the verdict.

**13. Run the gate — CLI.** `devforgeai gate check --phase release --id $1` writes `.devforgeai/reports/$1-release.yaml` with its ten check entries and the ingested `verifiers.deferrals` block, and `signoff` is filled from it. Exit 0 goes to step 14. Exit 2 goes to `## Send-back`. Exit 1 leaves the file at `status: released` with `signoff.gate: FAIL` and the run stops. Failure path: exit 1 on `DFA-E303` means `gates.toml` sits below a compiled floor; the run stops and the stderr line reaches the model.

**14. Move the stories — CLI.** Run `devforgeai phase set release --id $1` a second time. That call moves every `STORY-nnn` under `stories` in `.devforgeai/releases/$1.yaml` to frontmatter `status: released`, which is why the `release_stories` check accepts `released` beside `built`. Failure path: `DFA-W210` at exit 0 on a story the release names and the directory does not hold; the `ids_resolve` check has already reported that id.

**15. Run `doc validate` on the release file — CLI.** `devforgeai doc validate .devforgeai/releases/$1.yaml`. Failure path: exit 1 on `DFA-E217` (shape) or `DFA-E218` (not greater than the previous version); the run stops with one `Blocked` line naming both versions.

**16. Close — CLI, Stop hook.** The Stop hook runs `gate check` and then `devforgeai handoff --phase release --id $1`. The Stop hook renders the closing block; this skill writes no part of it. On a gate FAIL the Stop hook holds the turn with the failing checks as its `reason`, and the block appears when the turn ends; on a SEND BACK it renders the block with no block on the turn, because the next step is a command the user types.

## Subagents

Contracts, tools, models, and invocation order are in `agents.md`. Full schemas are in each `agents/<name>.md` `## Output`.

| Subagent | Invoked at | What to pass | What comes back |
|---|---|---|---|
| `deferral-auditor` | step 6, alone, once for the whole set | the version, `platform.target` when known, one record per story with `story`, `qa_report`, the deferred `FIND-nnn` with their reasons, and the AC texts; the `## Approved dependencies` rows | `devforgeai/verifier/1`: `passed`, `total`, `unit: deferrals`, `findings[]` each with `confidence` and a `kind` from the closed five `blocks_deployment`, `unjustified`, `circular`, `missing_report`, `accepted`; `payload` is `{}`; a registered verifier, ingested by SubagentStop into `verifiers.deferrals` |
| `deploy-manifest-writer` | step 8, alone; not invoked when `platform.target` is `none` | `platform.target`, the six `[release]` values, the `[[stack]]` ids and source roots, the `## Constraints` rows, the version, the template paths | `devforgeai/deploy-manifest/1`: `written[]`, `secrets_referenced`, `constraints_applied`, `reason` |
| `api-doc-writer` | step 10, batch 1 of 2 | the `api_symbols_command` lines, the `[[stack]].id` and `source_roots` pairs, `[release].docs_root`, the version | `devforgeai/api-docs/1`: `pages[]`, `symbols_total`, `symbols_documented`, `symbols_missing`, `reason` |
| `guide-writer` | step 10, batch 1 of 2 | the release set with AC texts, the accepted ADRs, the four context H2 sections of step 10, `[release].docs_root`, the version | `devforgeai/guide-docs/1`: `guide`, `architecture`, `stories_without_task`, `reason` |

## Documents

| Document | Path | Template |
|---|---|---|
| Release | `.devforgeai/releases/vX.Y.Z.yaml`, one per version | `templates/release.yaml` |
| Deploy manifests | under `config.toml` `[release].deploy_root`, one set per platform | `templates/kubernetes/*`, `templates/compose/*`, `templates/github-actions/deploy.yml`, `templates/vps/*` |
| Rollback note | `<deploy_root>/<platform>/ROLLBACK.md`, and `<deploy_root>/ROLLBACK.md` for `github-actions` | `templates/ROLLBACK.md` |
| Gate workflow | `.github/workflows/devforgeai-release.yml` | `templates/ci/devforgeai-release.yml` |
| Documentation set | under `config.toml` `[release].docs_root` | `templates/docs/README.md`, `api-index.md`, `api-page.md`, `guide-index.md`, `architecture-index.md` |

The release file is the one typed document of this phase. It carries the seven §5 keys as its first seven top-level keys, in this order and with no other key between them: `schema` (`devforgeai/release/1`), `id` (the version, matching `^v[0-9]+\.[0-9]+\.[0-9]+$`), `phase` (`release`), `status`, `produced_by` (`releasing-software`), `consumes` (every `STORY-nnn` in the set, ascending), `open_questions`. Nine further keys follow, in this order: `previous_version`, `released_at`, `stories`, `artifacts`, `platform`, `deploy`, `docs`, `notes`, `signoff`. Sixteen keys, no others.

`status` runs `draft`, written at step 11, then `released`, written at step 12 once the file is complete. A `--resume` run moves a file at `draft` forward again; there is no third value.

`notes.entries[].kind` is derived rather than asked, because story frontmatter carries no kind key: `fix` when the story's `consumes` holds a `FIND-nnn`, `internal` when its `consumes` holds no `REQ-nnn`, `feature` in every other case. `notes.requirements[]` is the ascending de-duplicated union of every entry's `requirements`, each with its `requirements[].statement` from `.devforgeai/requirements.yaml`. `notes.summary` is at most 72 characters and `notes.deferred[].summary` at most 90.

`artifacts[]` lists the files the release ships, each with `path`, `kind` from the enum `build | package | brand | docs | manifest`, and `bytes`. The glob patterns of `[release].artifact_paths` say which files those are.

Every `@@NAME@@` token in a template is replaced at write time: `@@VERSION@@` with `$1`, `@@PREVIOUS@@` with `previous_version`, `@@IMAGE@@` with `[release].image_name`, `@@PORT@@` with `[release].service_port`, `@@APP@@` with the project directory name, `@@DOCS@@` with `[release].docs_root`, `@@DEPLOY@@` with `[release].deploy_root`, and in an API page `@@STACK@@` with the `[[stack]].id` and `@@ROOT@@` with the source root. Shell files are written with LF line endings. `templates/ROLLBACK.md` keeps the one H2 block matching `platform.target` and the `## After a rollback` block; the writer deletes the other two platform blocks.

## Send-back

Release sends back to Verify and to no other phase. A verify failure surfaces at one of two points, and the earlier one is not a send-back at all.

**Before the run does any work, at step 2.** `devforgeai phase set release --id $1` refuses with `DFA-E320` when a story in the set has no verify gate at `PASS`. The run stops there with one `Blocked` line naming `/verify STORY-nnn` for the story the stderr names, no release file is written, no manifest and no documentation page is touched, and the Stop hook renders the block for the phase `[current]` still names. This is the common shape: a story short of a verify PASS is caught before the version is assembled.

**After the gate runs, at step 13.** Three of the ten gate checks resolve `on_fail = "send_back"`, and those produce the send-back below.

| Producing check | Condition | IDs cited |
|---|---|---|
| `release-deferrals` | `deferral-auditor` returned a finding at `severity: block`, so `passed < total` | the `FIND-nnn`, and on the same line the `STORY-nnn` the finding names |
| `release-stories` | a story in the release is at neither `built` nor `released`, or its `reports/STORY-nnn-verify.yaml` is absent or its `gate.result` is not `PASS` | the `STORY-nnn` ids, ascending |
| `release-ids` | a `STORY`, `REQ`, `ADR`, or `FIND` id the release file references resolves to no definition | the unresolved ids |

On any of the three the release file keeps `status: released` and `signoff.gate: SEND_BACK`, the manifests and the documentation stay on disk byte-identical, `state.toml` `[active].release` keeps the version, and step 14 does not run, so no story moves to `released`.

The Stop hook renders the closing block from the report; this skill writes no part of it. The two lines it carries:

```
Next      /verify STORY-nnn --remedy FIND-nnn,FIND-nnn
Then      /release vX.Y.Z --resume
```

A deferral the auditor marks `kind: accepted` is not a send-back. That finding carries `severity: info`, lands in `notes.deferred[]`, counts toward `passed`, and ships in the release notes so a reader sees what was left out. There is no `blocks_deployment` boolean on a finding: `kind: blocks_deployment` at `severity: block` says it once, and the routing is the gate's.

## Remedy and resume

**`--resume`** is the one flag this command takes. It re-enters at step 1 and runs step 3, which reads `.devforgeai/reports/$1-release.yaml` `findings[]` and narrows the rest of the run to the `STORY-nnn` and `FIND-nnn` those findings cite: step 4 assembles only those stories, step 6 re-audits only their deferrals, and step 10 rewrites only the pages they touch. Every other file under `[release].deploy_root`, `[release].docs_root`, and `.github/workflows/` keeps the bytes it had, because a send-back names a defect upstream of the manifests and the manifests did not change. A release file already on disk at `status: draft` moves forward; one at `released` is rewritten in place with the repaired values. Step 2 runs again and is idempotent for `state.toml`.

**`--remedy`** is not accepted. Reflect is the only phase downstream, its `REC-nnn` output is advisory and sets no gate, and a recommendation that concerns this phase arrives as a human edit to `.devforgeai/config.toml` `[release]`, to `gates.toml`, or to the spec, which the next `/release` run reads from the file.

## References

- `references/docs-layout.md` — read before step 10: the four documentation directories, the fixed H2 list per file, the `<stack-id>-<root-slug>` page-naming rule, and what `docs_cover` reads.
- `references/kubernetes.md` — read at step 8 when `platform.target` is `kubernetes`: the five files, their `kind` values, the fields the gate check reads, and the rollback note.
- `references/compose.md` — read at step 8 when `platform.target` is `compose`: the three files, the `${NAME}` to `env.example` pairing the gate check compares, and the rollback note.
- `references/github-actions.md` — read at step 8 when `platform.target` is `github-actions`: the deploy workflow, how it differs from the gate workflow of step 9, and the rollback note.
- `references/vps.md` — read at step 8 when `platform.target` is `vps`: the three files, the first-line and `ExecStart=` rules the gate check reads, and the rollback note.
