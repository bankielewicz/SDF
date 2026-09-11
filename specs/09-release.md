---
schema: devforgeai-spec/1
doc: release
status: draft
produced_by: spec-author-release
consumes: [00-conventions]
open_questions: []
---

# Phase 6 · Release · `releasing-software`

## Scope

This component turns a set of stories that Verify passed into one named version. It reads every `stories/STORY-nnn.md` at frontmatter `status: built`, resolves a deployment platform from a closed enum, writes the deployment manifests for that platform, writes the three documentation sets that describe what ships, writes `.devforgeai/releases/vX.Y.Z.yaml`, and runs the release gate. Its one slash command is `/release vX.Y.Z`. Its one typed document is `releases/vX.Y.Z.yaml`, read by Reflect. Its forward handoff is `/reflect vX.Y.Z`.

This component does not apply a deployment, connect to a cluster, a registry, or a host, hold a credential, or run the project's build. It writes manifests and the CI workflow that runs the gate; a push of the tag the release names is what the platform reacts to. It does not choose what to release: the set is every story at `built`, because `devforgeai phase set release` is the one writer that moves a story to `released`, so `built` is exactly the set Verify passed and no prior release consumed. It does not edit a story, a QA report, an ADR, or a context file; a defect in any of those leaves as a SEND BACK to Verify. It names no language, package manager, test runner, or build tool: the build and package commands come from `.devforgeai/config.toml` keys `[release].build_command` and `[release].package_command`, and the public-API symbol list comes from `[release].api_symbols_command`.

## Inputs

| Document | Path | IDs read | Producer | Used for |
|---|---|---|---|---|
| story | `.devforgeai/stories/STORY-nnn.md` | `STORY-nnn`, `AC-nnn`, `REQ-nnn` from `consumes` | `planning-work` | the release set, the release-note entry, the user-guide steps |
| sprint | `.devforgeai/stories/sprint.yaml` | `SPRINT-nnn`, `stories[].id`, `stories[].status` | `planning-work` | cross-check of the story set against the sprint's own list |
| QA report | `.devforgeai/reports/STORY-nnn-qa.yaml` | `FIND-nnn` | `validating-quality` | deferral audit, the `deferrals` field of each story entry |
| verify gate report | `.devforgeai/reports/STORY-nnn-verify.yaml` | `STORY-nnn` | `devforgeai` CLI | `gate.result`, which has to be `PASS` for every story in the set |
| ADR | `.devforgeai/adr/ADR-nnn.md` | `ADR-nnn`, `CON-nnn` | `establishing-context` | the architecture note, one H2 per ADR at `status: accepted` |
| tech stack | `.devforgeai/context/tech-stack.md` | `CON-nnn` | `establishing-context` | H2 `## Languages`, `## Runtimes`, `## Frameworks`, `## Data stores` fill the architecture note's stack table |
| dependencies | `.devforgeai/context/dependencies.md` | `CON-nnn` | `establishing-context` | H2 `## Approved dependencies` and `## License policy` fill the architecture note's dependency table |
| source tree | `.devforgeai/context/source-tree.md` | `CON-nnn` | `establishing-context` | H2 `## Layers` fills the architecture note's layer table |
| brand tokens | `.devforgeai/brand/tokens.json` | `TOKEN-<group>-<leaf>` | `designing-interfaces` | shipped asset, copied to `<docs_root>/brand/tokens.json` |
| brand logo | `.devforgeai/brand/logo.svg` | none | `designing-interfaces` | shipped asset, copied to `<docs_root>/brand/logo.svg`, referenced by `<docs_root>/README.md` |
| config | `.devforgeai/config.toml` | none | `devforgeai stack detect` | `[[stack]]`, `[release]`, `[[verifier]]` |
| state | `.devforgeai/state.toml` | none | `devforgeai phase set` | `current_phase`, `[active].verify`, `[active].release` |
| prior release | `.devforgeai/releases/v*.yaml` | none | this component | `previous_version`, and the monotonic version check |
| release gate report | `.devforgeai/reports/vX.Y.Z-release.yaml` | `STORY-nnn`, `FIND-nnn` | `devforgeai` CLI | `--resume` reads the `findings[]` of the last run at `status: send_back` |

`.devforgeai/requirements.yaml` is read for one field only: `requirements[].statement`, keyed by the `REQ-nnn` ids a story's `consumes` carries, which becomes the release-note line for that requirement. No epic, persona, or flow is read.

## Outputs

### 1. `.devforgeai/releases/vX.Y.Z.yaml`

The one typed document of this phase. Doc type `release` in `specs/01-cli.md` `doc validate`; the path form is `releases/v<X>.<Y>.<Z>.yaml`; extra top-level keys are permitted for this doc type. Sixteen top-level keys, in this order, and no others.

```yaml
schema: devforgeai/release/1        # string; fixed
id: v0.3.0                          # string; matches ^v[0-9]+\.[0-9]+\.[0-9]+$
phase: release                      # string; fixed
status: draft                       # string; enum: draft | released
produced_by: releasing-software     # string; fixed
consumes: [STORY-014, STORY-017]    # array[string]; every STORY-nnn in the release set, ascending
open_questions: []                  # array[string]; [] on a PASS run

previous_version: v0.2.1            # string; the highest v<X>.<Y>.<Z> already under releases/, or "" for a first release
released_at: 2026-09-11T09:14:02Z   # string; RFC 3339 UTC; the moment the file moved to status: released, or "" while draft

stories:                            # array; one entry per story in the release set, ascending by id; at least 1
  - id: STORY-014                   # string; STORY-nnn
    title: Order checkout           # string; the story's first H1 with the leading id and separator removed
    status: built                   # string; the story frontmatter status at write time; enum: built | released
    qa_report: .devforgeai/reports/STORY-014-qa.yaml       # string; project-relative path
    verify_report: .devforgeai/reports/STORY-014-verify.yaml  # string; project-relative path
    verify_result: PASS             # string; enum: PASS | FAIL | SEND_BACK; copied from the verify report gate.result
    requirements: [REQ-007, REQ-011]  # array[string]; the REQ-nnn entries of the story's consumes, ascending; [] when none
    deferrals: [FIND-003]           # array[string]; the FIND-nnn entries the QA report marks deferred; [] when none

artifacts:                          # array; zero or more; the files the release ships
  - path: dist/app.tar.gz           # string; project-relative path; the file exists at write time
    kind: package                   # string; enum: build | package | brand | docs | manifest
    bytes: 8412160                  # integer; the byte length of the file

platform:
  target: kubernetes                # string; the platform enum below
  source: config                    # string; enum: config | detected | asked
  marker: ""                        # string; the repo-relative path that matched when source is detected, else ""

deploy:
  root: deploy                      # string; config.toml [release].deploy_root
  manifests:                        # array; the files written for platform.target; [] when target is none
    - path: deploy/kubernetes/deployment.yaml   # string; project-relative
      kind: workload                # string; enum: workload | network | config | script | unit | workflow | overlay
  ci_workflow: .github/workflows/devforgeai-release.yml   # string; "" when config.toml [release].ci is none
  ci_check_name: devforgeai-release-gate                  # string; "" when ci_workflow is ""
  rollback: deploy/kubernetes/ROLLBACK.md                 # string; the rollback note path; "" when target is none

docs:
  root: docs                        # string; config.toml [release].docs_root
  index: docs/README.md             # string
  api:                              # array[string]; the API pages, index first
    - docs/api/index.md
    - docs/api/rust-src.md
  guide: docs/guide/index.md        # string
  architecture: docs/architecture/index.md   # string
  brand: [docs/brand/tokens.json, docs/brand/logo.svg]   # array[string]; [] when .devforgeai/brand/ holds neither file
  api_symbols: 142                  # integer; the line count of the [release].api_symbols_command output; 0 when the command is ""
  api_documented: 142               # integer; the symbols that appear as an H3 in an api page

notes:
  summary: Checkout and refunds     # string; at most 72 characters; the release line
  entries:                          # array; one per story, in stories order
    - story: STORY-014              # string; STORY-nnn
      title: Order checkout         # string; copied from stories[].title
      kind: feature                 # string; enum: feature | fix | internal
      requirements: [REQ-007, REQ-011]   # array[string]
  requirements:                     # array; the union of every entry's requirements, ascending, de-duplicated
    - id: REQ-007                   # string
      statement: A buyer completes a purchase without an account   # string; requirements.yaml requirements[].statement
  deferred:                         # array; one per FIND-nnn any story in the set defers; [] when none
    - find: FIND-003                # string; FIND-nnn
      story: STORY-014              # string; STORY-nnn
      summary: Empty-cart path has no test   # string; at most 90 characters
      blocks_deployment: false      # bool; the deferral-auditor verdict

signoff:
  gate: PASS                        # string; enum: PASS | FAIL | SEND_BACK | NOT_RUN
  report: .devforgeai/reports/v0.3.0-release.yaml   # string
  checks_passed: 10                 # integer
  checks_total: 10                  # integer
  at: 2026-09-11T09:14:02Z          # string; RFC 3339 UTC or ""
```

`status` enum, in progression order: `draft` (workflow step 11 wrote the file), `released` (workflow step 14 moved it after the gate returned PASS). A `--resume` run moves a file at `draft` forward again; there is no third value.

`notes.entries[].kind` is derived, not asked: `fix` when the story's `consumes` carries a `FIND-nnn`; `internal` when the story's `consumes` carries no `REQ-nnn`; `feature` in every other case.

### 2. The platform enum

Closed at five values. `platform.target` and `config.toml` `[release].platform` take one of these and nothing else.

| Value | Means | Manifests written under `[release].deploy_root` |
|---|---|---|
| `kubernetes` | a cluster applies declarative workload manifests | `kubernetes/deployment.yaml`, `kubernetes/service.yaml`, `kubernetes/ingress.yaml`, `kubernetes/kustomization.yaml`, `kubernetes/ROLLBACK.md` |
| `compose` | a single host runs a multi-container definition | `compose/docker-compose.yaml`, `compose/env.example`, `compose/ROLLBACK.md` |
| `github-actions` | the deployment itself is a workflow the repository host runs | `.github/workflows/deploy.yml`, and `<deploy_root>/ROLLBACK.md` |
| `vps` | a host runs a script and a service unit | `vps/deploy.sh`, `vps/app.service`, `vps/ROLLBACK.md` |
| `none` | nothing is deployed; the release is consumed as a library or a source distribution | none; `deploy.manifests` is `[]` and `deploy.rollback` is `""` |

### 3. The documentation layout

Rooted at `config.toml` `[release].docs_root`, default `docs`. Every path below is relative to the project root with `docs` substituted for the configured root.

```
docs/
├── README.md                       index: version, date, story count, links to the three sets, the logo
├── api/
│   ├── index.md                    table of every symbol, its kind, its stack, and the page it is on
│   └── <stack-id>-<root-slug>.md   one page per (stack, source root) pair; one H3 per symbol
├── guide/
│   └── index.md                    one H2 per story in the release, its ACs rewritten as steps
├── architecture/
│   └── index.md                    one H2 per accepted ADR, plus the stack, layer, and dependency tables
└── brand/
    ├── tokens.json                 byte copy of .devforgeai/brand/tokens.json
    └── logo.svg                    byte copy of .devforgeai/brand/logo.svg
```

`<stack-id>` is the `[[stack]].id` value. `<root-slug>` is that stack's `source_roots` entry with `/` and `\` replaced by `-`, lowercased, leading and trailing `-` removed; an entry of `.` gives `root`. A stack with three source roots gives three pages. The pair is used rather than the root alone because two `[[stack]]` tables can carry the same root string.

Section lists are fixed. `docs_cover` locates content by these strings.

| File | H2 sections, in order |
|---|---|
| `README.md` | `## What shipped`, `## Documentation`, `## Install`, `## Brand` |
| `api/index.md` | `## Symbols`, `## Pages` |
| `api/<page>.md` | `## Summary`, `## Symbols` (each symbol is an H3 below it, heading text exactly the symbol name) |
| `guide/index.md` | `## Before you start`, `## Tasks` (each story is an H3 below it, heading text exactly `<STORY-nnn> <title>`), `## Where to go next` |
| `architecture/index.md` | `## Stack`, `## Layers`, `## Dependencies`, `## Decisions` (each ADR is an H3 with the text `<ADR-nnn> <title>`), `## Constraints` |

### 4. The CI workflow

Always written when `config.toml` `[release].ci` is `github-actions`, for every value of `platform.target` including `none`. Path `.github/workflows/devforgeai-release.yml`. Its one job is named `devforgeai-release-gate`, which is the string a repository administrator adds as a required status check. Content in `## Templates`. It is a different file from the `github-actions` platform target's `.github/workflows/deploy.yml`: the first runs the gate and writes nothing; the second is a deployment and runs only when the gate job succeeded.

## Workflow

Sixteen steps. A `--resume` run enters at step 1 and narrows steps 4 through 10 as step 3 defines.

**1. Establish the run — model.** Input: `$ARGUMENTS`, `.devforgeai/state.toml`. Output: the version string `$1`, and a run kind of `fresh` or `resume` (`resume` when `$ARGUMENTS` contains `--resume`). Failure path: `$1` does not match `^v[0-9]+\.[0-9]+\.[0-9]+$`; the run stops with one `Blocked` line giving the accepted form `vX.Y.Z`.

**2. Activate the version — CLI.** Input: `devforgeai phase set release --id $1`. Output: `state.toml` `current_phase` at `release` and `[active].release` at `$1`, which is what `report ingest` resolves its target report from at step 8. The release file does not exist yet, so the story-status half of `phase set` writes nothing and returns `DFA-W210`, exit 0. Failure path: exit 1 on `DFA-E300`, meaning `gates.toml` holds no `release` gate; the run stops and the stderr line reaches the model. Exit 1 on `DFA-E320` means a story in the set has no verify gate at `PASS`; the run stops with one `Blocked` line naming `/verify STORY-nnn` for the story the stderr names, and no release file is written.

**3. Resume — read the last verdict — model.** Runs on a `resume` run only. Input: `.devforgeai/reports/$1-release.yaml`. Output: the `STORY-nnn` and `FIND-nnn` ids of its `findings[]`, which narrow step 4's set to those stories and step 10 to the pages those stories touch. Every file already written under `[release].deploy_root`, `[release].docs_root`, and `.github/workflows/` is left byte-identical. Failure path: the report is absent, or its `status` is not `send_back`; the run proceeds as `fresh`.

**4. Assemble the release set — CLI, model.** Input: the preamble's stdout, which is `devforgeai story list --status built --json`. The model reads each listed `stories/STORY-nnn.md` for its first H1, its frontmatter `consumes`, and its `status`, then reads `stories/sprint.yaml` `stories[]` and records any `{id, status}` pair whose status is `built` and whose id the CLI list omits. Output: the release set, ascending by id, each with `title`, `requirements`, and the two report paths. The entry shape is the one `templates/release.yaml` carries: `id`, `title`, `status`, `qa_report`, `verify_report`, `verify_result`, `requirements`, `deferrals`. Failure path: the set is empty; the run stops with `Blocked   you: no story is at status built; run /verify <STORY-nnn> first` and writes no release file.

**5. Read the verify verdicts — CLI.** Input: `devforgeai report show <STORY-nnn> verify` once per story in the set. Output: `gate.result` per story, written to `stories[].verify_result`. Failure path: exit 1 on `DFA-E400`, the report is absent; `verify_result` is recorded as `FAIL` and the story is carried into step 6, which is what produces the send-back.

**6. Audit the deferrals — subagent `deferral-auditor`.** Input: the `FIND-nnn` entries each `reports/STORY-nnn-qa.yaml` marks deferred, the story's ACs, `.devforgeai/context/dependencies.md` `## Approved dependencies`, and the release's `platform.target` once step 7 has run for a `resume` run, or the config value when one is set. Output: the JSON schema in `## Subagents`, ingested by `SubagentStop` into `.devforgeai/reports/$1-release.yaml` at `verifiers.deferrals`. Failure path: a story whose QA report is absent contributes one finding of `kind: missing_report` at `severity: block`, and the audit continues over the rest.

**7. Resolve the platform — model, CLI, user.** Input: `.devforgeai/config.toml` `[release].platform`. When it is present and not `""`, that value is `platform.target` with `source: config` and `marker: ""`. When it is absent or `""`, the model tests the marker list below in order and takes the first match, with `source: detected` and `marker` set to the matched path. When no marker matches, one `AskUserQuestion` call with the header `Platform` and the five options `Kubernetes`, `Compose`, `GitHub Actions`, `VPS`, `None` supplies the value with `source: asked`, and the model writes that value to `[release].platform` in `.devforgeai/config.toml` so the next release reads it from the file. Output: `platform.target`, `platform.source`, `platform.marker`. Failure path: a free-text answer matching no label re-asks once, then the run takes `none` and adds one `open_questions` line naming the header.

| Order | Marker glob | Gives |
|---|---|---|
| 1 | `**/kustomization.yaml`, `**/kustomization.yml`, `**/Chart.yaml` | `kubernetes` |
| 2 | `docker-compose.yaml`, `docker-compose.yml`, `compose.yaml`, `compose.yml` at the project root | `compose` |
| 3 | `.github/workflows/deploy.yml`, `.github/workflows/deploy.yaml` | `github-actions` |
| 4 | `<deploy_root>/vps/deploy.sh` | `vps` |

**8. Write the deployment manifests — subagent `deploy-manifest-writer`.** Input: `platform.target`, `config.toml` `[release].image_name`, `[release].deploy_root`, `[release].build_command`, `[release].package_command`, `[[stack]].source_roots`, the `## Constraints` rows of `.devforgeai/context/architecture-constraints.md`, and the templates named in `## Templates`. Output: the files the platform row of `## Outputs` lists, the JSON schema in `## Subagents`, and `deploy.manifests[]`. On `platform.target` of `none` the agent is not invoked and `deploy.manifests` is `[]`. Failure path: the agent returns `written: []` with a `reason` string; the model writes the platform's template files verbatim with `@@IMAGE@@`, `@@PORT@@`, and `@@NAME@@` substituted, and records the reason in `open_questions`.

**9. Write the CI workflow — model.** Runs when `config.toml` `[release].ci` is `github-actions`. Input: `templates/ci/devforgeai-release.yml`. Output: `.github/workflows/devforgeai-release.yml` with `@@VERSION@@` replaced by `$1`, and `deploy.ci_workflow` and `deploy.ci_check_name` filled. Failure path: `[release].ci` is `none`; the file is not written and both keys are `""`.

**10. Write the documentation — subagents `api-doc-writer` and `guide-writer`, in parallel.** Input to `api-doc-writer`: the output lines of `config.toml` `[release].api_symbols_command`, the `[[stack]]` tables, and `[release].docs_root`. Input to `guide-writer`: the release set with each story's ACs, every `adr/ADR-nnn.md` at `status: accepted`, the four context H2 sections `## Languages`, `## Layers`, `## Approved dependencies`, `## License policy`, and `[release].docs_root`. Output: the files of `## Outputs` section 3, and `docs.api`, `docs.guide`, `docs.architecture`, `docs.api_symbols`, `docs.api_documented`. The model then copies `.devforgeai/brand/tokens.json` and `.devforgeai/brand/logo.svg` to `<docs_root>/brand/` and fills `docs.brand`. Failure path: `[release].api_symbols_command` is `""`; `api-doc-writer` writes `docs/api/index.md` with an empty `## Symbols` table, `docs.api_symbols` and `docs.api_documented` are both `0`, and the `docs_cover` check evaluates to `SKIP` with `reason: no_api_symbols_command`.

**11. Write the release file — model.** Input: everything steps 4 through 10 produced, plus `previous_version` taken as the highest `v<X>.<Y>.<Z>` among the `.devforgeai/releases/v*.yaml` names other than `$1.yaml`, or `""` when the directory holds no other release. Output: `.devforgeai/releases/$1.yaml` with `status: draft` and `released_at: ""`, all sixteen keys in the `## Outputs` order. Failure path: the `PostToolUse` hook returns the `doc validate` diagnostic in `hookSpecificOutput.additionalContext`; the model rewrites the key it names and writes the file again.

**12. Move to released — model.** Input: the file from step 11. Output: the same file with `status: released` and `released_at` at the current UTC instant in RFC 3339. Failure path: none; the gate at step 13 is what decides whether the version stands, and a FAIL leaves the file at `released` with `signoff.gate` carrying `FAIL`, which step 13 writes from the report.

**13. Run the gate, then write the signoff — CLI, model.** Input: `devforgeai gate check --phase release --id $ARGUMENTS[0]`. Output: `.devforgeai/reports/$ARGUMENTS[0]-release.yaml` with its ten check entries and the ingested `verifiers.deferrals` block. The CLI writes nothing into the release file, so after `gate check` returns the model edits that file's `signoff` block from the report: `gate` takes the report's `result` (`PASS`, `FAIL`, or `SEND_BACK`), `report` takes that report's path, `checks_passed` and `checks_total` take the counts of `checks[]` at `pass` and in total, and `at` takes the report's `at`. This edit happens on every exit code, before the run stops or continues — a release file left at the template's `gate: NOT_RUN` with zero counts is one where the edit did not run. Then exit 0 goes to step 14, exit 2 goes to `## Send-back`, and exit 1 leaves the file at `status: released` with the `signoff` block just written and the run stops. Failure path: exit 1 on `DFA-E303` means `gates.toml` sits below a compiled floor; no report is written, `signoff` keeps `gate: NOT_RUN`, and the run stops with the stderr line reaching the model.

**14. Mark the stories released — CLI.** Input: `devforgeai phase set release --id $1`, run a second time. Output: every `STORY-nnn` under `stories` in `.devforgeai/releases/$1.yaml` moved to frontmatter `status: released`, which is why the `release_stories` check accepts `released` as well as `built`. Failure path: `DFA-W210`, exit 0, on a story file the release names and the directory does not hold; the `ids_resolve` check has already reported that id.

**15. Validate the release file — CLI.** Input: `devforgeai doc validate .devforgeai/releases/$1.yaml`. Output: exit 0. Failure path: exit 1 on `DFA-E217 (shape) or DFA-E218 (not greater than the previous version)`, the version is not greater than `previous_version`; the run stops with one `Blocked` line naming both versions.

**16. Handoff — CLI, Stop hook.** `devforgeai gate check --phase release --id $1` then `devforgeai handoff --phase release --id $1`, both run by the Stop hook, which emits the block in `systemMessage`. The Stop hook renders the closing block; this skill writes no part of it.

## Subagents

### deferral-auditor

- **name**: `deferral-auditor`
- **derives_from**: `C:\Users\bryan\.claude\agents\deferral-validator.md`
- **purpose**: Decide, per deferred `FIND-nnn`, whether the deferral blocks this release's deployment.
- **tools**: `Read`, `Glob`, `Grep`
- **model**: `opus` — the judgment is whether a stated reason survives contact with the target platform, which is the one place this phase reasons rather than assembles.
- **input**: the release version; `platform.target`; one record per story in the set holding `story` (`STORY-nnn`), `qa_report` (path), the `FIND-nnn` ids that report marks deferred with their stated reason, and the story's AC texts; the `## Approved dependencies` rows of `.devforgeai/context/dependencies.md`.
- **output**:

```json
{
  "schema": "devforgeai/verifier/1",
  "subagent": "deferral-auditor",
  "unit": "deferrals",
  "passed": 4,
  "total": 5,
  "findings": [
    {
      "id": "FIND-009",
      "severity": "block",
      "confidence": 0.85,
      "story": "STORY-017",
      "kind": "blocks_deployment",
      "summary": "Refund webhook signature check deferred with no replacement control",
      "reason": "Deferred to STORY-031",
      "evidence": ".devforgeai/reports/STORY-017-qa.yaml:41"
    }
  ],
  "payload": {}
}
```

  One object, the `devforgeai/verifier/1` envelope. `payload` is `{}`: this agent adds no field of its own, and the empty object is present rather than omitted so the envelope has one shape across every verifier.

  `kind` is a closed enum: `blocks_deployment`, `unjustified`, `circular`, `missing_report`, `accepted`. `severity` is `block` for the first four and `info` for `accepted`, and that pairing is the whole verdict — there is no separate `blocks_deployment` boolean on the finding. A boolean beside a `kind` that already says the same thing is two places to read one fact, and the two disagree the first time one is written and the other is not. `confidence` rides on every entry, `0.0` to `1.0`. `passed` counts the `accepted` entries; `total` counts every deferred `FIND-nnn` across the set. A story with no deferral contributes nothing to either count.
- **invoked_at**: workflow step 6, once for the whole release set, before the platform is resolved on a `fresh` run.
- **registered_verifier**: `yes` — `config.toml` carries the `[[verifier]]` table in `## Gate`, and `SubagentStop` ingests the block into `.devforgeai/reports/vX.Y.Z-release.yaml` at `verifiers.deferrals`.

Adapted from `deferral-validator` in four ways. The severity ladder of four levels collapses to the two the report schema carries, `block` and `info`, because `verifier_pass` reads `passed / total` and nothing else. `AskUserQuestion` leaves the tool list because it could not have been kept: Claude Code strips `AskUserQuestion` from every subagent whatever its `tools` list holds, so the tool could not have been kept: the question belongs to the invoking skill. `SubagentStop` ingests this agent unattended in any case. The question changes from "is this deferral justified" to "does this deferral block deployment on this platform", which is the only question Release is entitled to ask about work Verify already passed. The observation-capture write and the `devforgeai/feedback/` path leave entirely; the report is the record.

### deploy-manifest-writer

- **name**: `deploy-manifest-writer`
- **derives_from**: `C:\Users\bryan\.claude\agents\deployment-engineer.md`
- **purpose**: Write the manifest set for one platform target from the release's own values.
- **tools**: `Read`, `Write`, `Glob`
- **model**: `sonnet` — the shape is fixed by the templates and the work is substitution plus the constraint rows.
- **input**: `platform.target`; `[release].image_name`, `[release].deploy_root`, `[release].build_command`, `[release].package_command`, `[release].service_port`; the `[[stack]].id` and `source_roots` values; the `## Constraints` rows of `.devforgeai/context/architecture-constraints.md`; the version string; the template paths for the target.
- **output**:

```json
{
  "schema": "devforgeai/deploy-manifest/1",
  "target": "kubernetes",
  "written": [
    { "path": "deploy/kubernetes/deployment.yaml", "kind": "workload" },
    { "path": "deploy/kubernetes/service.yaml", "kind": "network" },
    { "path": "deploy/kubernetes/ingress.yaml", "kind": "network" },
    { "path": "deploy/kubernetes/kustomization.yaml", "kind": "overlay" },
    { "path": "deploy/kubernetes/ROLLBACK.md", "kind": "script" }
  ],
  "secrets_referenced": ["APP_DATABASE_URL", "APP_SIGNING_KEY"],
  "constraints_applied": ["CON-012"],
  "reason": ""
}
```

  `kind` is the closed enum of `deploy.manifests[].kind` in `## Outputs`. `secrets_referenced` lists the environment variable names the manifests read; values are excluded from the list. `reason` is `""` on success and a one-sentence string when `written` is `[]`.
- **invoked_at**: workflow step 8, once, after the platform is resolved and before the docs are written; not invoked when `platform.target` is `none`.
- **registered_verifier**: `no` — it writes files and reports no pass ratio; the `deploy_manifest` gate check reads the files it wrote.

Adapted from `deployment-engineer` in five ways. The platform list narrows from Kubernetes, Docker, Terraform, Ansible, Helm, AWS, Azure, and GCP to the five-value enum, because each extra platform is a manifest set no gate check validates. Every `Bash` entry leaves the tool list, so the agent writes manifests and runs none of them, which is what keeps credentials out of the phase. The infrastructure-as-code, monitoring, and runbook steps leave; a rollback note stays as one file per platform. The prose output becomes the JSON schema above. The `devforgeai/specs/` and `devforgeai/deployment/` paths become `.devforgeai/context/` and `[release].deploy_root`. `deployment-engineer-platform-patterns`, which lives at `C:\Users\bryan\.claude\agents\deployment-engineer\references\platform-patterns.md`, is replaced rather than adapted: it is a reference document, not an agent, it has no output to give a schema to per §10, and its content moves into this skill's `templates/` where the gate's `deploy_manifest` check can be written against fixed text.

### api-doc-writer

- **name**: `api-doc-writer`
- **derives_from**: `C:\Users\bryan\.claude\agents\documentation-writer.md`
- **purpose**: Write one API page per stack and source root, with one H3 per public symbol the CLI enumerated.
- **tools**: `Read`, `Write`, `Glob`, `Grep`
- **model**: `sonnet` — the symbol list arrives fixed and the work is locating each symbol and describing it.
- **input**: the symbol lines of `[release].api_symbols_command`, each `<kind>\t<symbol>\t<path>`; the `[[stack]].id` and `source_roots` pairs; `[release].docs_root`; the version string.
- **output**:

```json
{
  "schema": "devforgeai/api-docs/1",
  "pages": [
    { "path": "docs/api/index.md", "stack": "", "root": "", "symbols": 0 },
    { "path": "docs/api/rust-src.md", "stack": "rust", "root": "src", "symbols": 142 }
  ],
  "symbols_total": 142,
  "symbols_documented": 142,
  "symbols_missing": [],
  "reason": ""
}
```

  `symbols_missing` lists the symbol names the agent found no source for and therefore wrote no H3 for. `reason` is `""` on success.
- **invoked_at**: workflow step 10, in parallel with `guide-writer`.
- **registered_verifier**: `no` — the `docs_cover` gate check recomputes the ratio from the files rather than trusting the agent's count.

Adapted from `documentation-writer` in four ways. The scope narrows from five output kinds to one, the API pages, so a second agent can run beside it. OpenAPI, Swagger, C4, and the inline-docstring work leave: the page format is the fixed H2 and H3 shape of `## Outputs` section 3, which `docs_cover` reads by heading text. The "coverage below 80%" trigger leaves; the threshold lives in the gate's `docs_cover.min_ratio`, per §1. `Edit` leaves the tool list, because every page is written whole on each release.

### guide-writer

- **name**: `guide-writer`
- **derives_from**: `C:\Users\bryan\.claude\agents\documentation-writer.md`
- **purpose**: Write the user guide from the release's ACs and the architecture note from the accepted ADRs and the context files.
- **tools**: `Read`, `Write`, `Glob`
- **model**: `opus` — turning an acceptance criterion into a task a reader follows, and an ADR into a paragraph, is the judgment this phase keeps in a subagent.
- **input**: the release set with each story's `STORY-nnn`, title, and AC texts; every `adr/ADR-nnn.md` at `status: accepted` with its `## Context` and `## Decision` sections; the H2 sections `## Languages` and `## Runtimes` of `context/tech-stack.md`, `## Layers` of `context/source-tree.md`, `## Approved dependencies` and `## License policy` of `context/dependencies.md`, `## Constraint index` of `context/architecture-constraints.md`; `[release].docs_root`; the version string.
- **output**:

```json
{
  "schema": "devforgeai/guide-docs/1",
  "guide": { "path": "docs/guide/index.md", "stories": 4, "tasks": 11 },
  "architecture": { "path": "docs/architecture/index.md", "decisions": 7, "constraints": 12 },
  "stories_without_task": [],
  "reason": ""
}
```

  `stories_without_task` lists the `STORY-nnn` ids whose ACs produced no H3 in the guide. `reason` is `""` on success.
- **invoked_at**: workflow step 10, in parallel with `api-doc-writer`.
- **registered_verifier**: `no` — the guide has no pass ratio; the `file_exists` check confirms both files and `docs_cover` reads the API pages alone.

Adapted from `documentation-writer` in three ways. The scope narrows to the guide and the architecture note, the two output kinds `api-doc-writer` does not take. The README, inline-comment, and diagram work leave: the release index is a four-section file the model writes from a template at step 10, and no diagram format is fixed anywhere in this framework. The `devforgeai/specs/context/` paths become `.devforgeai/context/`.

## Command

The entry point is the skill itself: `skills/releasing-software/SKILL.md`, installed to `.claude/skills/release/`, whose frontmatter `name` is the slash command. There is no command file. This is the frontmatter and the preamble, verbatim, with one preamble line.

```markdown
---
name: release
description: Phase 6 of DevForgeAI, run by /release. Turns every story at status built into one named version - it resolves a deployment platform from a five-value enum, writes that platform's manifests under the configured deploy root, writes the API reference, the user guide, and the architecture note under the configured docs root, copies the brand assets, and writes .devforgeai/releases/vX.Y.Z.yaml for Reflect to read. Reach for it whenever /release is typed, whenever a version of the form vX.Y.Z is being cut, whenever deploy manifests, a rollback note, a release gate workflow, release notes, or a published documentation set are being written for this framework, and whenever releases/vX.Y.Z.yaml, a deferral that might block deployment, or a Release send-back to Verify appears in a report or a handoff.
argument-hint: vX.Y.Z [--resume]
allowed-tools: Bash(devforgeai:*), PowerShell(devforgeai:*), Read, Write, Edit, Glob, Grep, Agent, AskUserQuestion
disable-model-invocation: true
---

!`devforgeai story list --status built --json`
```

The one preamble line prints the release set. It carries no `gate require` line because Release's predecessor is checked per story rather than per subject, and the `UserPromptExpansion` hook runs `gate require release` on the version before the skill body enters context. It allocates no id: the version is `$1` and every other id it names was allocated upstream.

## CLI calls

| Subcommand with exact arguments | Called from | Exit handling |
|---|---|---|
| `devforgeai story list --status built --json` | command `!` preamble, and workflow step 4 | stdout is the release set; 1 on `DFA-E230` when `.devforgeai/stories/` is absent, and the run stops |
| `devforgeai phase set release --id <vX.Y.Z>` | workflow step 2, and again at step 14 | 0 sets `[active].release`; at step 2 the release file is absent and the story writes return `DFA-W210`, exit 0; 1 on `DFA-E300` stops the run |
| `devforgeai report show <STORY-nnn> verify` | workflow step 5, once per story | 0 prints the report; 1 on `DFA-E400` records `verify_result: FAIL` for that story |
| `devforgeai doc load qa-report <STORY-nnn>` | workflow step 6, once per story with a deferral | 1 on `DFA-E200` contributes a `missing_report` finding and the audit continues |
| `devforgeai doc load story <STORY-nnn>` | workflow steps 4 and 10, once per story | 1 on `DFA-E200` drops that story from the set and adds one `open_questions` line naming the id |
| `devforgeai doc load adr all` | workflow step 10 | 1 on `DFA-E200` means no ADR exists; `## Decisions` in the architecture note is written with the single line `No accepted decision record.` |
| `devforgeai doc load requirements -` | workflow step 11 | 1 on `DFA-E200` leaves every `notes.requirements[].statement` as `""` |
| `devforgeai report ingest deferral-auditor -` | `SubagentStop` hook, after step 6 | 0 in every case; a miss on the `[[verifier]]` lookup is `DFA-W411` and `verifier_pass` fails later with `DFA-E316` |
| `devforgeai gate check --phase release --id <vX.Y.Z>` | workflow step 13, and the CI job `devforgeai-release-gate` | 0 goes to step 14; 2 goes to `## Send-back`; 1 stops the run. On every exit code the model writes the `signoff` block from the report the call produced; `DFA-E303` writes no report and leaves `signoff.gate: NOT_RUN` |
| `devforgeai doc validate .devforgeai/releases/<vX.Y.Z>.yaml` | workflow step 15, and the PostToolUse hook on Write | 1 on `DFA-E217 (shape) or DFA-E218 (not greater than the previous version)` names the version that is not greater than `previous_version` |
| `devforgeai handoff --phase release --id <vX.Y.Z>` | workflow step 16 | prints the §6 block from `.devforgeai/reports/<vX.Y.Z>-release.yaml` |
| `devforgeai trust verify` | the CI job, as its first step | 4 fails the job before the gate runs |
| `devforgeai report show <vX.Y.Z> release` | user, `improving-framework` | prints `.devforgeai/reports/<vX.Y.Z>-release.yaml` |

`story list` is a proposed addition to the §4 surface and is recorded in `## Decisions`. Every other subcommand is a §4 name, with `report ingest` already carried as a proposed addition by `specs/01-cli.md` `## Decisions` entry 1.

### Local versus CI

| Step | Runs locally in `/release` | Runs in the CI job `devforgeai-release-gate` |
|---|---|---|
| `trust verify` | through every hook | yes, as the first job step |
| assemble the release set | yes | no |
| audit the deferrals | yes, through `deferral-auditor` | no; the ingested block is read from the committed report |
| resolve the platform | yes | no |
| write the manifests, the CI workflow, the docs | yes | no |
| write and move `releases/vX.Y.Z.yaml` | yes | no |
| `doc validate --all` | through the PostToolUse and pre-commit hooks | yes |
| `gate check --phase release --id <vX.Y.Z>` | yes, at step 13 | yes, and its exit code is the required status check |
| `phase set release` | yes, at steps 2 and 14 | no; CI writes nothing under `.devforgeai/` |
| apply a deployment | no | no; the tag push is what the platform reacts to |

The CI job is a second evaluation of the same gate over the committed tree, which is why `release_stories.require_status` accepts `released` as well as `built`: by the time the commit reaches CI, step 14 has already moved every story in the set to `released`, and a check that accepted `built` alone would turn the required status check red on a release that passed locally.

## Gate

`.devforgeai/gates.toml`, the `release` entry, verbatim. It replaces the default entry of `specs/01-cli.md` `## Gate`. It keeps the compiled-in required kinds for this phase, `doc_valid` and `file_exists`, both at `severity = "block"`.

```toml
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
```

Ten checks. The compiled-in required kinds for this phase, `doc_valid` and `file_exists`, are both present at the default `severity = "block"`.

`no_open_questions` is absent on purpose: a release that carries an unanswered platform question is a release `deploy_manifest` already fails, and a duplicate check would report the same fact twice in the handoff `Gate` line.

`release-manifest` carries no condition key and runs for all five platform values, including `none`, because the `none` row of the platform table below is itself a rule: `deploy.manifests[]` is `[]` and `deploy.rollback` is `""`. The check dispatches on `platform.target` internally, so the `none` case is validated rather than skipped and no condition key is needed.

Four of the ten checks reuse kinds by name from the `specs/01-cli.md` enum: `field_in_enum` for the release status, `length_between` for a non-empty story set, `fields_present` for the shape of each `stories[]` entry, and `ids_resolve` for the references. Three kinds are proposed additions, each justified in `## Decisions` against the closest existing kind.

`release-entries` and `release-stories` are not redundant, and the split is deliberate. `release-entries` checks shape: every `stories[]` entry carries the six fields with a non-empty value. Those values are what the model wrote at workflow step 11, so a release claiming `verify_result: PASS` for a story whose report says `FAIL` passes it. `release-stories` checks truth: it opens `stories/<id>.md` and `reports/<id>-verify.yaml` and compares. Shape alone gates nothing, which is why the `release_stories` proposal stands.

`config.toml` gains the `[[verifier]]` table that makes `release-deferrals` resolvable. `devforgeai init` writes it beside the Verify entry.

```toml
[[verifier]]
name = "deferral-auditor"
phase = "release"
report_field = "verifiers.deferrals"
unit = "deferrals"
required = true
```

`config.toml` also gains the `[release]` table this component reads. `devforgeai init` writes it with these defaults and `stack detect` leaves it untouched, in the way it already leaves `[[layer]]`, `[coverage]`, and `[[verifier]]`.

```toml
[release]
platform = ""                # string; enum: kubernetes | compose | github-actions | vps | none; "" means resolve at run time
ci = "github-actions"        # string; enum: github-actions | none; default "github-actions"
deploy_root = "deploy"       # string; default "deploy"; project-relative
docs_root = "docs"           # string; default "docs"; project-relative
build_command = ""           # string; default ""; the project's own release build
package_command = ""         # string; default ""; the project's own packaging step
artifact_paths = []          # array[string]; glob patterns; default []; the files listed under artifacts
api_symbols_command = ""     # string; default ""; prints one line per public symbol as <kind>\t<symbol>\t<path>
image_name = ""              # string; default ""; the container image reference for kubernetes, compose, github-actions
service_port = 8080          # integer; default 8080; the port the manifests expose
```

### Proposed check kinds

Three additions to the closed twenty-one-value enum of `specs/01-cli.md` `## Outputs`, taking it to twenty-four. Each is recorded in `## Decisions` with the existing kind it was weighed against. Every key below may carry the `{id}` and `{phase}` tokens, as the existing kinds do, and every one accepts the four condition keys and `message`.

| `kind` | Keys, types, defaults | Passes when |
|---|---|---|
| `release_stories` | `path` string, default `releases/{id}.yaml`; `require_status` array[string], default `["built", "released"]`; `require_report` string, default `verify`; `require_result` string, default `PASS` | every `stories[].id` in `path` names a file `stories/<id>.md` whose frontmatter `status` is in `require_status`, and a report `reports/<id>-<require_report>.yaml` whose `gate.result` equals `require_result` |
| `deploy_manifest` | `path` string, default `releases/{id}.yaml`; `platform` string, default `""` meaning read `config.toml` `[release].platform` and fall back to `path`'s `platform.target` | the rules of the platform table below hold for every entry of `path`'s `deploy.manifests[]` |
| `docs_cover` | `path` string, default `releases/{id}.yaml`; `min_ratio` float, default `1.0`; `command` string, default `""` meaning `config.toml` `[release].api_symbols_command` | the symbol names `command` prints each appear as an H3 heading in one of `path`'s `docs.api[]` files, at a ratio of at least `min_ratio`; `SKIP` with `reason: no_api_symbols_command` when the command is `""` |

`deploy_manifest` rules, by platform. Every rule is a parse or a pattern, never a network call and never an invocation of a platform tool.

| `platform` | Passes when |
|---|---|
| `kubernetes` | every listed path exists; every `.yaml` path parses as YAML; each of `deployment.yaml`, `service.yaml`, `ingress.yaml`, `kustomization.yaml` appears once in `deploy.manifests[]`; `deployment.yaml` carries `spec.template.spec.containers[0].image`, `.readinessProbe`, `.livenessProbe`, and `.resources.limits`; no file matches the secret pattern below |
| `compose` | `docker-compose.yaml` exists and parses as YAML; it carries a `services` mapping with at least one entry; every entry carries `image` or `build`; `env.example` exists; every `${NAME}` reference in the compose file has a `NAME=` line in `env.example`; no file matches the secret pattern |
| `github-actions` | `.github/workflows/deploy.yml` exists and parses as YAML; it carries `on`, `jobs`, and a job whose `needs` names `devforgeai-release-gate` when `deploy.ci_workflow` is non-empty; no file matches the secret pattern |
| `vps` | `vps/deploy.sh` exists, is non-empty, and its first line is `#!/bin/sh` or `#!/usr/bin/env sh`; `vps/app.service` exists and carries `ExecStart=`; no file matches the secret pattern |
| `none` | `deploy.manifests[]` is `[]` and `deploy.rollback` is `""` |

The secret pattern, applied line by line to every listed manifest: a line matching `(?i)(password|secret|token|api[_-]?key|private[_-]?key)\s*[:=]\s*["']?[A-Za-z0-9+/=_-]{12,}` that is not a `${...}` reference, a `secretKeyRef`, a `valueFrom`, or a `$(...)` substitution fails the check with `DFA-E343`.

## Send-back

Release sends back to Verify and to no other phase. A verify failure surfaces at one of two points, and the earlier one is not a send-back at all.

**Before the run does any work, at step 2.** `devforgeai phase set release --id $ARGUMENTS[0]` refuses with `DFA-E320` when a story in the set has no verify gate at `PASS`. The run stops there with one `Blocked` line naming `/verify STORY-nnn` for the story the stderr names, no release file is written, no manifest and no documentation page is touched, and the Stop hook renders the block for the phase `[current]` still names. This is the common shape: a story short of a verify PASS is caught before the version is assembled.

**After the gate runs, at step 13.** The destination is the gate's `send_back_to`, and three of the ten checks resolve `on_fail = "send_back"`. Those produce the send-back below.

| Producing check | Condition | IDs cited | Handoff `Next` |
|---|---|---|---|
| `release-deferrals` | `deferral-auditor` returned a finding at `severity: block`, so `passed < total` | the `FIND-nnn` and, on the same line, the `STORY-nnn` the finding names | `/verify <STORY-nnn> --remedy <FIND-nnn>,...` |
| `release-stories` | a story in the release is not at `built` or `released`, or its `reports/STORY-nnn-verify.yaml` is absent or its `gate.result` is not `PASS` | the `STORY-nnn` ids, ascending | `/verify <STORY-nnn> --remedy <STORY-nnn>` |
| `release-ids` | a `STORY`, `REQ`, `ADR`, or `FIND` id the release file references has no definition | the unresolved ids | `/verify <STORY-nnn> --remedy <unresolved ids>` |

On any of the three the release file keeps `status: released` and carries `signoff.gate: SEND_BACK`, which step 13 wrote from the report's `result` along with the rest of the `signoff` block, the manifests and the documentation stay on disk byte-identical, `state.toml` `[active].release` keeps the version, and step 14 does not run, so no story moves to `released`. The returning command is `/release vX.Y.Z --resume`, printed on the `Then` line, and workflow step 3 narrows the rerun to the cited ids.

A deferral that the auditor marks `kind: accepted` at `severity: info` is not a send-back: it appears in `notes.deferred[]` with `blocks_deployment: false`, counts toward `passed`, and ships in the release notes so a reader sees what was left out. The release file's boolean is the skill's own record of the verdict, written from the finding's `kind`; the finding itself carries the `kind` alone.

Release receives a send-back from no phase. §5 gives Reflect as the only phase downstream of Release, and Reflect emits `REC-nnn` recommendations that set no gate, so nothing arrives at `/release vX.Y.Z --remedy <ids>` and the command's `argument-hint` carries no `--remedy` form. A recommendation that concerns this phase reaches it as a human edit to `.devforgeai/config.toml` `[release]`, to `gates.toml`, or to this spec, and the next `/release` run reads the edited file.

## Integration

| Skill | Consumes (doc, IDs) | Produces for (doc, IDs) | Sends back to (condition) | Receives send-back from (condition) | Shared subagents | `state.toml` fields read/written |
|---|---|---|---|---|---|---|
| 0 Explore · `exploring-ideas` | none — Release reads no brief; an `IDEA-nnn` reaches it only as the ancestor of a `REQ-nnn`, and the release notes cite the requirement, not the idea | none — Explore runs before any story exists and reads no release | none — a defect this phase finds is in a story, a QA report, or a manifest, and §5 routes Release to Verify alone | none — §5 gives Explore no downstream send-back and Explore cites no version | none | none |
| 1 Discover · `discovering-requirements` | `.devforgeai/requirements.yaml`: `requirements[].statement`, keyed by the `REQ-nnn` ids each story's `consumes` carries, one line per entry of `notes.requirements[]` | `.devforgeai/releases/vX.Y.Z.yaml` `notes.requirements[]`, which names the `REQ-nnn` set a version delivered; Discover reads no release file and the relationship is one-way | none — an unresolved `REQ-nnn` is `release-ids`, and §5 routes it to Verify, which owns the story's references | none — Discover emits no send-back downstream of Plan | none | none |
| 2 Constitute · `establishing-context` | `.devforgeai/context/tech-stack.md` `## Languages` and `## Runtimes`; `context/source-tree.md` `## Layers`; `context/dependencies.md` `## Approved dependencies` and `## License policy`; `context/architecture-constraints.md` `## Constraints` and `## Constraint index` (`CON-nnn`); `adr/ADR-nnn.md` `## Context` and `## Decision` at `status: accepted` (`ADR-nnn`) | `<docs_root>/architecture/index.md`, which is the published form of the accepted ADRs and the four context tables; Constitute reads none of it back | none — a contradiction between an ADR and a manifest is a `CON-nnn` the `deploy_manifest` check reports, and the fix is a Constitute-side edit a human makes, not a §6 send-back, because §5 gives Release one target | none — §5 gives Constitute's send-back target as Discover | none | none |
| 3 Plan · `planning-work` | `.devforgeai/stories/STORY-nnn.md`: frontmatter `status`, `consumes` (`REQ-nnn`, `FIND-nnn`), the first H1 for `stories[].title`, and the AC texts the user guide turns into steps; `stories/sprint.yaml` `stories[]` of `{id, status}` for the cross-check at step 4 | `.devforgeai/stories/STORY-nnn.md` frontmatter `status: released`, written by `devforgeai phase set release` at step 14 and by no other actor; `releases/vX.Y.Z.yaml` `stories[]`, which names every story a version carried | none — a story that is not implementable is Plan's to rewrite and Verify's to cite; §5 gives Release one send-back target | none — §5 gives Plan's send-back targets as Discover and Constitute | none | reads `[active].verify` at step 4 to name the last story Verify closed; writes `[active].release` and `current_phase` through `phase set` |
| 4 Build · `implementing-stories` | `.devforgeai/reports/STORY-nnn-build.yaml`: nothing is read from it, so this cell is none — the build verdict reaches Release through the verify gate report, which Verify's gate already required to pass | none — Build reads no release file; the artifacts a release lists are the output of `[release].build_command`, which the project owns and Build does not write | none — an implementation defect leaves as a `FIND-nnn` citation to Verify, which is where a `FIND-nnn` is defined, and Verify forwards to Build | none — Build emits no send-back downstream of Verify | none | none |
| 5 Verify · `validating-quality` | `.devforgeai/reports/STORY-nnn-qa.yaml`: `findings[]` (`FIND-nnn`) with the deferral flag and the stated reason, which is the whole input of `deferral-auditor`; `.devforgeai/reports/STORY-nnn-verify.yaml` `gate.result`, which `release_stories` requires to be `PASS` | `.devforgeai/releases/vX.Y.Z.yaml` `stories[].deferrals` and `notes.deferred[]`, which is the record of which `FIND-nnn` shipped unaddressed; `.devforgeai/reports/vX.Y.Z-release.yaml` `findings[]`, which the send-back cites | yes: a deferral blocks deployment (`release-deferrals`), a story is not at `built` or `released` or has no PASS verify report (`release-stories`), or a referenced id does not resolve (`release-ids`); all three leave as `/verify <STORY-nnn> --remedy <FIND-nnn or STORY-nnn>,...` with `/release vX.Y.Z --resume` on the `Then` line | none — §5 gives Verify's send-back targets as Build and Plan, and Verify runs upstream of this phase | `deferral-auditor` derives from the same `deferral-validator` agent that Verify's deferral handling derives from; the two are distinct registrations with different phases and different questions, and neither invokes the other | reads `[active].verify` |
| 6 Release · `releasing-software` | self | self | self | self | self | reads `current_phase`, `[active].verify`, `[active].release`; writes `current_phase` and `[active].release` through two `phase set release` calls |
| Design · `designing-interfaces` | `.devforgeai/brand/tokens.json` and `.devforgeai/brand/logo.svg`, copied whole to `<docs_root>/brand/` and listed in `docs.brand[]` and in `artifacts[]` at `kind: brand`; no `TOKEN-<name>` value is read, and `ui-specs/UI-nnn.md` is not read | `<docs_root>/README.md` `## Brand`, which references the copied logo by its published path; Design reads no release file | none — a missing token or a wrong logo is a Design-side edit, and §5 gives Release one send-back target | none — Design emits no send-back downstream of Build | none; Design owns `mockup-designer`, `brand-designer`, `requirement-coverage-auditor`, and `ui-spec-writer`, and this phase invokes none of them | none |
| Reflect · `improving-framework` | none — Reflect runs after this phase and its `REC-nnn` output is advisory per §5, so nothing it writes is an input to a release | `.devforgeai/releases/vX.Y.Z.yaml`, the §5 document Reflect reads for the version window; `.devforgeai/reports/vX.Y.Z-release.yaml`, the gate report its `OBS-nnn` extraction reads | none — Reflect owns no gate to fail | none — Reflect emits recommendations, not send-backs, so this phase has no receiving side to specify | none | none |
| CLI · `devforgeai` | `.devforgeai/config.toml` keys `[release].*`, `[[stack]].id`, `[[stack]].source_roots`, `[[verifier]]`; `.devforgeai/gates.toml` the `release` entry; `.devforgeai/state.toml` `current_phase`, `[active].verify`, `[active].release` | `.devforgeai/releases/vX.Y.Z.yaml` for `doc validate`, `doc load release`, and the ten release checks; `.devforgeai/reports/vX.Y.Z-release.yaml` for `report ingest`, `report show`, and `handoff` | none — the CLI produces a send-back result and does not request one | none — a send-back names a phase and the CLI owns none | none — the CLI invokes no subagent and ingests `deferral-auditor` through `SubagentStop` | reads and writes `current_phase` and `[active].release` |

## Handoff

Both blocks are rendered by `devforgeai handoff --phase release --id vX.Y.Z`. The slug is `-` in both: the algorithm of `specs/01-cli.md` takes it from the document's first H1, and a YAML release file carries none.

`Done` content is `<n> stories · <version>`, the release row of the `Done` counts table of `specs/01-cli.md`. `Next` on PASS is `/reflect <vX.Y.Z>` and `Then` is omitted, which is that spec's transition table for this phase.

PASS, nine lines:

```
Phase     6 · Release        v0.3.0 · -
Done      4 stories · v0.3.0
Gate      PASS  10 checks
Verified  deferral-auditor · 5/5 deferrals

Next      /reflect v0.3.0
Blocked   none

Full report: .devforgeai/reports/v0.3.0-release.yaml
```

SEND BACK to Verify, two blocking deferrals and one story with no PASS verify report, twelve lines, which is the §6 cap exactly. The fixed lines number ten with `Verified` and `Then` present, so the `Found` budget is two, and three findings render as one Found line plus the `+2 more` line, by the truncation rule of `specs/01-cli.md`.

```
Phase     6 · Release        v0.3.0 · -
Done      4 stories · v0.3.0
Gate      SEND BACK to Verify  release-deferrals 3/5, release-stories 3/4
Verified  deferral-auditor · 3/5 deferrals
Found     FIND-009 STORY-017 refund webhook signature check has no replacement
Found     +2 more in report

Next      /verify STORY-017 --remedy FIND-009,FIND-012
Then      /release v0.3.0 --resume
Blocked   none

Full report: .devforgeai/reports/v0.3.0-release.yaml
```

## Templates

Shipped at `skills/releasing-software/templates/`. Every `@@NAME@@` token is replaced at write time: `@@VERSION@@` with `$1`, `@@IMAGE@@` with `[release].image_name`, `@@PORT@@` with `[release].service_port`, `@@APP@@` with the project directory name, `@@DOCS@@` with `[release].docs_root`, `@@DEPLOY@@` with `[release].deploy_root`. Shell files are written with LF line endings.

### `templates/release.yaml`

```yaml
schema: devforgeai/release/1
id: "@@VERSION@@"
phase: release
status: draft
produced_by: releasing-software
consumes: []
open_questions: []
previous_version: ""
released_at: ""
stories:
  - id: STORY-000
    title: <the story's first H1 with the leading id and separator removed>
    status: built
    qa_report: .devforgeai/reports/STORY-000-qa.yaml
    verify_report: .devforgeai/reports/STORY-000-verify.yaml
    verify_result: PASS
    requirements: []
    deferrals: []
artifacts: []
platform:
  target: none
  source: detected
  marker: ""
deploy:
  root: "@@DEPLOY@@"
  manifests: []
  ci_workflow: ""
  ci_check_name: ""
  rollback: ""
docs:
  root: "@@DOCS@@"
  index: ""
  api: []
  guide: ""
  architecture: ""
  brand: []
  api_symbols: 0
  api_documented: 0
notes:
  summary: ""
  entries: []
  requirements: []
  deferred: []
signoff:
  gate: NOT_RUN
  report: ""
  checks_passed: 0
  checks_total: 0
  at: ""
```
### `templates/ci/devforgeai-release.yml`

```yaml
name: devforgeai release gate
on:
  push:
    tags:
      - "v*.*.*"
  pull_request:
    paths:
      - ".devforgeai/**"
      - "@@DEPLOY@@/**"
      - "@@DOCS@@/**"
jobs:
  devforgeai-release-gate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: resolve version
        id: v
        run: |
          v="${GITHUB_REF_NAME}"
          case "$v" in
            v*.*.*) ;;
            *) v="@@VERSION@@" ;;
          esac
          echo "version=$v" >> "$GITHUB_OUTPUT"
      - name: verify the binary
        run: devforgeai trust verify
      - name: validate the documents
        run: devforgeai doc validate --all
      - name: check the release gate
        run: devforgeai gate check --phase release --id "${{ steps.v.outputs.version }}"
```

The job name `devforgeai-release-gate` is the string added as a required status check. The job writes no file under `.devforgeai/` other than the gate report, which `gate check` produces on its own.

### `templates/kubernetes/deployment.yaml`

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: "@@APP@@"
  labels:
    app: "@@APP@@"
    version: "@@VERSION@@"
spec:
  replicas: 2
  selector:
    matchLabels:
      app: "@@APP@@"
  template:
    metadata:
      labels:
        app: "@@APP@@"
        version: "@@VERSION@@"
    spec:
      containers:
        - name: "@@APP@@"
          image: "@@IMAGE@@:@@VERSION@@"
          ports:
            - containerPort: @@PORT@@
          env:
            - name: APP_PORT
              value: "@@PORT@@"
          envFrom:
            - secretRef:
                name: "@@APP@@-secrets"
          readinessProbe:
            httpGet:
              path: /healthz
              port: @@PORT@@
            initialDelaySeconds: 5
            periodSeconds: 10
          livenessProbe:
            httpGet:
              path: /healthz
              port: @@PORT@@
            initialDelaySeconds: 20
            periodSeconds: 20
          resources:
            requests:
              cpu: 100m
              memory: 128Mi
            limits:
              cpu: 500m
              memory: 512Mi
```

### `templates/kubernetes/service.yaml`

```yaml
apiVersion: v1
kind: Service
metadata:
  name: "@@APP@@"
  labels:
    app: "@@APP@@"
spec:
  type: ClusterIP
  selector:
    app: "@@APP@@"
  ports:
    - name: http
      port: 80
      targetPort: @@PORT@@
```

### `templates/kubernetes/ingress.yaml`

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: "@@APP@@"
  labels:
    app: "@@APP@@"
spec:
  rules:
    - host: "@@APP@@.example.invalid"
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: "@@APP@@"
                port:
                  number: 80
```

### `templates/kubernetes/kustomization.yaml`

```yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
resources:
  - deployment.yaml
  - service.yaml
  - ingress.yaml
images:
  - name: "@@IMAGE@@"
    newTag: "@@VERSION@@"
```

### `templates/compose/docker-compose.yaml`

```yaml
services:
  app:
    image: "@@IMAGE@@:@@VERSION@@"
    restart: unless-stopped
    ports:
      - "${APP_PORT}:@@PORT@@"
    environment:
      APP_PORT: "@@PORT@@"
      APP_DATABASE_URL: "${APP_DATABASE_URL}"
      APP_SIGNING_KEY: "${APP_SIGNING_KEY}"
    healthcheck:
      test: ["CMD", "wget", "-qO-", "http://127.0.0.1:@@PORT@@/healthz"]
      interval: 20s
      timeout: 5s
      retries: 3
```

### `templates/compose/env.example`

```
APP_PORT=@@PORT@@
APP_DATABASE_URL=
APP_SIGNING_KEY=
```

Every `${NAME}` reference in `docker-compose.yaml` has a `NAME=` line here; the `deploy_manifest` check compares the two sets.

### `templates/github-actions/deploy.yml`

```yaml
name: deploy @@VERSION@@
on:
  push:
    tags:
      - "v*.*.*"
jobs:
  deploy:
    needs: devforgeai-release-gate
    runs-on: ubuntu-latest
    environment: production
    steps:
      - uses: actions/checkout@v4
      - name: deploy
        env:
          APP_IMAGE: "@@IMAGE@@:@@VERSION@@"
          APP_DEPLOY_TOKEN: "${{ secrets.APP_DEPLOY_TOKEN }}"
        run: ./@@DEPLOY@@/vps/deploy.sh
```

`needs: devforgeai-release-gate` is the line the `deploy_manifest` check reads for this platform, so a deployment cannot start on a release whose gate job did not pass.

### `templates/vps/deploy.sh`

```sh
#!/bin/sh
set -eu

APP_NAME="@@APP@@"
APP_VERSION="@@VERSION@@"
APP_PORT="@@PORT@@"
APP_ROOT="${APP_ROOT:-/srv/${APP_NAME}}"
APP_RELEASE="${APP_ROOT}/releases/${APP_VERSION}"

mkdir -p "${APP_RELEASE}"
tar -xzf "${1:?release archive path}" -C "${APP_RELEASE}"
ln -sfn "${APP_RELEASE}" "${APP_ROOT}/current"
systemctl restart "${APP_NAME}"

i=0
while [ "${i}" -lt 30 ]; do
  if wget -qO- "http://127.0.0.1:${APP_PORT}/healthz" >/dev/null 2>&1; then
    echo "up ${APP_NAME} ${APP_VERSION}"
    exit 0
  fi
  i=$((i + 1))
  sleep 2
done

echo "health check failed for ${APP_NAME} ${APP_VERSION}" >&2
exit 1
```

### `templates/vps/app.service`

```
[Unit]
Description=@@APP@@
After=network-online.target

[Service]
Type=simple
WorkingDirectory=/srv/@@APP@@/current
EnvironmentFile=/srv/@@APP@@/env
ExecStart=/srv/@@APP@@/current/bin/@@APP@@
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

### `templates/ROLLBACK.md`

```markdown
# Rollback @@APP@@

Previous version: `@@PREVIOUS@@`
Current version: `@@VERSION@@`

## Kubernetes

    kubectl rollout undo deployment/@@APP@@

## Compose

    APP_IMAGE=@@IMAGE@@:@@PREVIOUS@@ docker compose up -d

## VPS

    ln -sfn /srv/@@APP@@/releases/@@PREVIOUS@@ /srv/@@APP@@/current
    systemctl restart @@APP@@

## After a rollback

The release file `.devforgeai/releases/@@VERSION@@.yaml` stays on disk at `status: released`.
The next release reads it as `previous_version`, so the version after a rollback is still the next one forward.
```

Only the section matching `platform.target` is kept; the writer deletes the other two H2 blocks.

### `templates/docs/README.md`

```markdown
# @@APP@@ @@VERSION@@

## What shipped

<!-- one row per story: | STORY-nnn | title | kind | REQ ids | -->

| Story | Title | Kind | Requirements |
|---|---|---|---|

## Documentation

- [API reference](api/index.md)
- [User guide](guide/index.md)
- [Architecture](architecture/index.md)

## Install

<!-- the [release].package_command output and where it lands; one paragraph -->

## Brand

![@@APP@@](brand/logo.svg)

Design tokens: [`brand/tokens.json`](brand/tokens.json)
```

### `templates/docs/api-index.md`

```markdown
# API reference @@VERSION@@

## Symbols

| Symbol | Kind | Stack | Page |
|---|---|---|---|

## Pages

<!-- one link per <stack-id>-<root-slug>.md page -->
```

### `templates/docs/api-page.md`

```markdown
# @@STACK@@ · @@ROOT@@

## Summary

<!-- one paragraph: what this source root holds and what a caller reaches first -->

## Symbols

### <symbol name, verbatim from the api_symbols_command line>

<!-- one paragraph, then the signature in a fenced block, then parameters and returns as a table -->
```

### `templates/docs/guide-index.md`

```markdown
# @@APP@@ user guide @@VERSION@@

## Before you start

<!-- one paragraph: what the reader has installed and configured before task one -->

## Tasks

### <STORY-nnn> <title>

<!-- one numbered list, one step per acceptance criterion, in the story's AC order -->

## Where to go next

- [API reference](../api/index.md)
- [Architecture](../architecture/index.md)
```

### `templates/docs/architecture-index.md`

```markdown
# @@APP@@ architecture @@VERSION@@

## Stack

| Layer of concern | Choice | Version |
|---|---|---|

## Layers

| Layer | Path | Depends on |
|---|---|---|

## Dependencies

| Name | Version | Scope | License |
|---|---|---|---|

## Decisions

### <ADR-nnn> <title>

<!-- one paragraph from ## Context, one from ## Decision -->

## Constraints

| CON | Kind | Statement | Enforced by |
|---|---|---|---|
```

## Evals

Shipped at `skills/releasing-software/evals/`. Fixtures named `FIXTURE:<name>` live at `skills/releasing-software/evals/fixtures/<name>` and are copied by the shared runner before the case runs.

### `evals/evals.json` — 9 entries, skill-creator format

| # | `prompt` | `expected_output` | `expectations[]` |
|---|---|---|---|
| 1 | `/release v0.3.0` with three stories at `built`, each with a PASS verify report and no deferral, and `[release].platform = "kubernetes"` | `.devforgeai/releases/v0.3.0.yaml` at `status: released` and four Kubernetes manifests | the sixteen top-level keys appear in the `## Outputs` order; `stories[]` holds three entries ascending by id; `platform.target` is `kubernetes` and `platform.source` is `config`; `deploy.manifests[]` holds `deployment.yaml`, `service.yaml`, `ingress.yaml`, `kustomization.yaml`; `signoff.gate` is `PASS` |
| 2 | `/release v0.3.0` with `[release].platform = ""` and a `docker-compose.yaml` at the project root | the release file with `platform.source: detected` | `platform.target` is `compose`; `platform.marker` is `docker-compose.yaml`; `deploy/compose/docker-compose.yaml` and `deploy/compose/env.example` both exist; every `${NAME}` in the compose file has a `NAME=` line in `env.example` |
| 3 | `/release v0.3.0` with `[release].platform = "none"` on a library project | the release file with no manifest | `deploy.manifests` is `[]`; `deploy.rollback` is `""`; no file exists under `deploy/`; `docs/api/index.md`, `docs/guide/index.md`, and `docs/architecture/index.md` all exist; the run printed a PASS handoff |
| 4 | `/release v0.3.0` where one story's QA report defers `FIND-009` with the reason `Deferred to STORY-031` on an authentication control | a SEND BACK handoff to Verify | the `Gate` line reads `SEND BACK to Verify`; a `Found` line names `FIND-009` and `STORY-017`; the `Next` line is `/verify STORY-017 --remedy FIND-009`; the `Then` line is `/release v0.3.0 --resume`; no story file moved to `status: released` |
| 5 | `/release v0.3.0` where `STORY-021` is at `built` and `reports/STORY-021-verify.yaml` carries `gate.result: FAIL` | a SEND BACK handoff to Verify citing the story | the `Gate` line names `release-stories`; the `Next` line cites `STORY-021`; `releases/v0.3.0.yaml` carries `signoff.gate: SEND_BACK`; `stories[]` still lists `STORY-021` with `verify_result: FAIL` |
| 6 | `/release v0.3.0 --resume` after case 4, with `FIND-009` now closed in the QA report | a PASS handoff with the manifests untouched | the `Gate` line reads `PASS`; every file under `deploy/` has the byte content it had before the resume; `notes.deferred[]` holds no entry with `blocks_deployment: true`; `Next` is `/reflect v0.3.0` |
| 7 | `/release v0.2.0` where `.devforgeai/releases/v0.3.0.yaml` already exists | the run stops before writing a release file | the run reports the version is not greater than `v0.3.0`; `.devforgeai/releases/v0.2.0.yaml` does not exist; no file was written under `deploy/` or `docs/` |
| 8 | `/release v0.3.0` with `[release].api_symbols_command` printing 12 symbols and `docs/api/` written for one stack | the API pages covering every symbol | `docs/api/index.md` `## Symbols` holds 12 data rows; `docs/api/<stack>-<root>.md` holds 12 H3 headings whose text equals the symbol names; `docs.api_symbols` and `docs.api_documented` are both `12`; `docs.brand` lists the two copied brand files |
| 9 | `/release v0.3.0` over three stories: one whose `consumes` holds `FIND-004`, one whose `consumes` holds no `REQ-nnn`, one holding two `REQ-nnn` | the release notes with the three derived kinds | `notes.entries` holds three entries in `stories[]` order; the kinds are `fix`, `internal`, and `feature` in the order the stories sort; `notes.requirements` is the ascending de-duplicated union of the entries' requirement ids; `notes.summary` is at most 72 characters |

### `evals/cases.jsonl` — 10 lines

Every case carries its prior state in `expect.args`. No grader reads a file the runner mirrored from a previous case, and no grader reads outside the case workspace.

```json
{"id": "rl-01-k8s-pass", "prompt": "/release v0.3.0", "timeout": 1500, "preflight": ["phase set release --id v0.3.0"], "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-k8s-api.toml", ".devforgeai/gates.toml": "FIXTURE:gates-release.toml", ".devforgeai/state.toml": "FIXTURE:state-verify.toml", ".devforgeai/stories/STORY-014.md": "FIXTURE:story-014-built.md", ".devforgeai/stories/STORY-017.md": "FIXTURE:story-017-built.md", ".devforgeai/stories/STORY-021.md": "FIXTURE:story-021-built.md", ".devforgeai/stories/sprint.yaml": "FIXTURE:sprint-active.yaml", ".devforgeai/reports/STORY-014-verify.yaml": "FIXTURE:verify-pass-014.yaml", ".devforgeai/reports/STORY-017-verify.yaml": "FIXTURE:verify-pass-017.yaml", ".devforgeai/reports/STORY-021-verify.yaml": "FIXTURE:verify-pass-021.yaml", ".devforgeai/reports/STORY-014-qa.yaml": "FIXTURE:qa-clean-014.yaml", ".devforgeai/reports/STORY-017-qa.yaml": "FIXTURE:qa-clean-017.yaml", ".devforgeai/reports/STORY-021-qa.yaml": "FIXTURE:qa-clean-021.yaml", ".devforgeai/requirements.yaml": "FIXTURE:requirements-8-reqs.yaml", ".devforgeai/context/tech-stack.md": "FIXTURE:tech-stack.md", ".devforgeai/context/source-tree.md": "FIXTURE:source-tree.md", ".devforgeai/context/dependencies.md": "FIXTURE:dependencies.md", ".devforgeai/context/architecture-constraints.md": "FIXTURE:architecture-constraints.md", ".devforgeai/adr/ADR-003.md": "FIXTURE:adr-003-accepted.md", ".devforgeai/brand/tokens.json": "FIXTURE:tokens.json", ".devforgeai/brand/logo.svg": "FIXTURE:logo.svg", "ci/api-symbols": "FIXTURE:ci-api-symbols.sh"}}, "expect": {"grader": "release_shape", "args": {"path": ".devforgeai/releases/v0.3.0.yaml", "keys": ["schema", "id", "phase", "status", "produced_by", "consumes", "open_questions", "previous_version", "released_at", "stories", "artifacts", "platform", "deploy", "docs", "notes", "signoff"], "status": "released", "stories": ["STORY-014", "STORY-017", "STORY-021"], "platform_target": "kubernetes", "platform_source": "config", "manifest_basenames": ["deployment.yaml", "service.yaml", "ingress.yaml", "kustomization.yaml"], "signoff_gate": "PASS"}}}
{"id": "rl-02-compose-detect", "prompt": "/release v0.3.0", "timeout": 1500, "preflight": ["phase set release --id v0.3.0"], "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-no-platform.toml", ".devforgeai/gates.toml": "FIXTURE:gates-release.toml", ".devforgeai/state.toml": "FIXTURE:state-verify.toml", ".devforgeai/stories/STORY-014.md": "FIXTURE:story-014-built.md", ".devforgeai/stories/sprint.yaml": "FIXTURE:sprint-active-014.yaml", ".devforgeai/reports/STORY-014-verify.yaml": "FIXTURE:verify-pass-014.yaml", ".devforgeai/reports/STORY-014-qa.yaml": "FIXTURE:qa-clean-014.yaml", ".devforgeai/requirements.yaml": "FIXTURE:requirements-8-reqs.yaml", ".devforgeai/context/tech-stack.md": "FIXTURE:tech-stack.md", ".devforgeai/context/source-tree.md": "FIXTURE:source-tree.md", ".devforgeai/context/dependencies.md": "FIXTURE:dependencies.md", "docker-compose.yaml": "services:\n  db:\n    image: postgres:16\n"}}, "expect": {"grader": "platform_resolution", "args": {"path": ".devforgeai/releases/v0.3.0.yaml", "target": "compose", "source": "detected", "marker": "docker-compose.yaml", "required_files": ["deploy/compose/docker-compose.yaml", "deploy/compose/env.example"], "env_pairs": {"compose": "deploy/compose/docker-compose.yaml", "env": "deploy/compose/env.example"}}}}
{"id": "rl-03-none-library", "prompt": "/release v0.3.0", "timeout": 1500, "preflight": ["phase set release --id v0.3.0"], "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-none.toml", ".devforgeai/gates.toml": "FIXTURE:gates-release.toml", ".devforgeai/state.toml": "FIXTURE:state-verify.toml", ".devforgeai/stories/STORY-014.md": "FIXTURE:story-014-built.md", ".devforgeai/stories/sprint.yaml": "FIXTURE:sprint-active-014.yaml", ".devforgeai/reports/STORY-014-verify.yaml": "FIXTURE:verify-pass-014.yaml", ".devforgeai/reports/STORY-014-qa.yaml": "FIXTURE:qa-clean-014.yaml", ".devforgeai/requirements.yaml": "FIXTURE:requirements-8-reqs.yaml", ".devforgeai/context/tech-stack.md": "FIXTURE:tech-stack.md", ".devforgeai/context/source-tree.md": "FIXTURE:source-tree.md", ".devforgeai/context/dependencies.md": "FIXTURE:dependencies.md", ".devforgeai/adr/ADR-003.md": "FIXTURE:adr-003-accepted.md"}}, "expect": {"grader": "docs_layout", "args": {"path": ".devforgeai/releases/v0.3.0.yaml", "docs_root": "docs", "required": ["docs/README.md", "docs/api/index.md", "docs/guide/index.md", "docs/architecture/index.md"], "absent_dir": "deploy", "empty_keys": ["deploy.manifests", "docs.brand"], "empty_strings": ["deploy.rollback"], "headings": {"docs/README.md": ["What shipped", "Documentation", "Install", "Brand"], "docs/architecture/index.md": ["Stack", "Layers", "Dependencies", "Decisions", "Constraints"]}, "transcript_gate": "PASS"}}}
{"id": "rl-04-deferral-sendback", "prompt": "/release v0.3.0", "timeout": 1500, "preflight": ["phase set release --id v0.3.0"], "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-k8s.toml", ".devforgeai/gates.toml": "FIXTURE:gates-release.toml", ".devforgeai/state.toml": "FIXTURE:state-verify.toml", ".devforgeai/stories/STORY-014.md": "FIXTURE:story-014-built.md", ".devforgeai/stories/STORY-017.md": "FIXTURE:story-017-built.md", ".devforgeai/stories/sprint.yaml": "FIXTURE:sprint-active-014-017.yaml", ".devforgeai/reports/STORY-014-verify.yaml": "FIXTURE:verify-pass-014.yaml", ".devforgeai/reports/STORY-017-verify.yaml": "FIXTURE:verify-pass-017.yaml", ".devforgeai/reports/STORY-014-qa.yaml": "FIXTURE:qa-clean-014.yaml", ".devforgeai/reports/STORY-017-qa.yaml": "FIXTURE:qa-defer-auth-017.yaml", ".devforgeai/requirements.yaml": "FIXTURE:requirements-8-reqs.yaml", ".devforgeai/context/tech-stack.md": "FIXTURE:tech-stack.md", ".devforgeai/context/dependencies.md": "FIXTURE:dependencies.md"}}, "expect": {"grader": "sendback_block", "args": {"to": "Verify", "next_prefix": "/verify STORY-017 --remedy ", "must_cite": ["FIND-009"], "forbidden_substrings": ["--resume", "STORY-014"], "then_line": "/release v0.3.0 --resume", "found_must_name": [["FIND-009", "STORY-017"]], "stories_not_released": [".devforgeai/stories/STORY-014.md", ".devforgeai/stories/STORY-017.md"]}}}
{"id": "rl-05-blocked-verify-fail", "prompt": "/release v0.3.0", "timeout": 1500, "preflight": [{"command": "phase set release --id v0.3.0", "exit": 1, "code": "DFA-E320"}], "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-k8s.toml", ".devforgeai/gates.toml": "FIXTURE:gates-release.toml", ".devforgeai/state.toml": "FIXTURE:state-verify.toml", ".devforgeai/stories/STORY-014.md": "FIXTURE:story-014-built.md", ".devforgeai/stories/STORY-021.md": "FIXTURE:story-021-built.md", ".devforgeai/stories/sprint.yaml": "FIXTURE:sprint-active-014-021.yaml", ".devforgeai/reports/STORY-014-verify.yaml": "FIXTURE:verify-pass-014.yaml", ".devforgeai/reports/STORY-021-verify.yaml": "FIXTURE:verify-fail-021.yaml", ".devforgeai/reports/STORY-014-qa.yaml": "FIXTURE:qa-clean-014.yaml", ".devforgeai/reports/STORY-021-qa.yaml": "FIXTURE:qa-clean-021.yaml", ".devforgeai/requirements.yaml": "FIXTURE:requirements-8-reqs.yaml", ".devforgeai/context/tech-stack.md": "FIXTURE:tech-stack.md", ".devforgeai/context/dependencies.md": "FIXTURE:dependencies.md"}}, "expect": {"grader": "blocked_at_activation", "args": {"code": "DFA-E320", "must_name": ["STORY-021"], "blocked_names": "/verify STORY-021", "absent": [".devforgeai/releases/v0.3.0.yaml", ".devforgeai/reports/v0.3.0-release.yaml"], "stories_not_released": [".devforgeai/stories/STORY-014.md", ".devforgeai/stories/STORY-021.md"], "report_phase": "release"}}}
{"id": "rl-06-resume-untouched", "prompt": "/release v0.3.0 --resume", "timeout": 1500, "preflight": ["phase set release --id v0.3.0"], "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-k8s.toml", ".devforgeai/gates.toml": "FIXTURE:gates-release.toml", ".devforgeai/state.toml": "FIXTURE:state-release-active.toml", ".devforgeai/stories/STORY-014.md": "FIXTURE:story-014-built.md", ".devforgeai/stories/STORY-017.md": "FIXTURE:story-017-built.md", ".devforgeai/stories/sprint.yaml": "FIXTURE:sprint-active.yaml", ".devforgeai/reports/STORY-014-verify.yaml": "FIXTURE:verify-pass-014.yaml", ".devforgeai/reports/STORY-017-verify.yaml": "FIXTURE:verify-pass-017.yaml", ".devforgeai/reports/STORY-014-qa.yaml": "FIXTURE:qa-clean-014.yaml", ".devforgeai/reports/STORY-017-qa.yaml": "FIXTURE:qa-clean-017.yaml", ".devforgeai/reports/v0.3.0-release.yaml": "FIXTURE:release-report-sendback.yaml", ".devforgeai/releases/v0.3.0.yaml": "FIXTURE:release-v030-draft.yaml", ".devforgeai/requirements.yaml": "FIXTURE:requirements-8-reqs.yaml", ".devforgeai/context/tech-stack.md": "FIXTURE:tech-stack.md", ".devforgeai/context/dependencies.md": "FIXTURE:dependencies.md", "deploy/kubernetes/deployment.yaml": "FIXTURE:k8s-deployment.yaml", "deploy/kubernetes/service.yaml": "FIXTURE:k8s-service.yaml", "deploy/kubernetes/ingress.yaml": "FIXTURE:k8s-ingress.yaml", "deploy/kubernetes/kustomization.yaml": "FIXTURE:k8s-kustomization.yaml"}}, "expect": {"grader": "resume_untouched", "args": {"path": ".devforgeai/releases/v0.3.0.yaml", "baseline_sha256": {"deploy/kubernetes/deployment.yaml": "da9771a95e225e389c05dc4a440dd51a1cf58bbb1331fa248bb1a7d54bb2c269", "deploy/kubernetes/service.yaml": "7ee97186bbf0d7fc487a39ec2a188b1d6e4c36b39c6205ccca00c31f1b0a2cc8", "deploy/kubernetes/ingress.yaml": "37948e8b1ac0d6c46f79b9209cf178de2a883e42b0786712e0c1090671eaca3a", "deploy/kubernetes/kustomization.yaml": "d10ef6921b3afe0cc101b02812ce4afb568727336694ee78cc39609c2355ae1d"}, "transcript_gate": "PASS", "next_line": "/reflect v0.3.0", "no_blocking_deferral": true}}}
{"id": "rl-07-version-not-ahead", "prompt": "/release v0.2.0", "timeout": 1500, "preflight": ["phase set release --id v0.2.0"], "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-k8s.toml", ".devforgeai/gates.toml": "FIXTURE:gates-release.toml", ".devforgeai/state.toml": "FIXTURE:state-verify.toml", ".devforgeai/stories/STORY-014.md": "FIXTURE:story-014-built.md", ".devforgeai/stories/sprint.yaml": "FIXTURE:sprint-active-014.yaml", ".devforgeai/reports/STORY-014-verify.yaml": "FIXTURE:verify-pass-014.yaml", ".devforgeai/reports/STORY-014-qa.yaml": "FIXTURE:qa-clean-014.yaml", ".devforgeai/releases/v0.3.0.yaml": "FIXTURE:release-v030-released.yaml", ".devforgeai/requirements.yaml": "FIXTURE:requirements-8-reqs.yaml", ".devforgeai/context/tech-stack.md": "FIXTURE:tech-stack.md", ".devforgeai/context/dependencies.md": "FIXTURE:dependencies.md"}}, "expect": {"grader": "version_refused", "args": {"attempted": "v0.2.0", "existing": "v0.3.0", "absent": [".devforgeai/releases/v0.2.0.yaml"], "absent_dirs": ["deploy", "docs"], "transcript_must_contain": ["v0.2.0", "v0.3.0"]}}}
{"id": "rl-08-api-coverage", "prompt": "/release v0.3.0", "timeout": 1500, "preflight": ["phase set release --id v0.3.0"], "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-api-symbols.toml", ".devforgeai/gates.toml": "FIXTURE:gates-release.toml", ".devforgeai/state.toml": "FIXTURE:state-verify.toml", ".devforgeai/stories/STORY-014.md": "FIXTURE:story-014-built.md", ".devforgeai/stories/sprint.yaml": "FIXTURE:sprint-active-014.yaml", ".devforgeai/reports/STORY-014-verify.yaml": "FIXTURE:verify-pass-014.yaml", ".devforgeai/reports/STORY-014-qa.yaml": "FIXTURE:qa-clean-014.yaml", ".devforgeai/requirements.yaml": "FIXTURE:requirements-8-reqs.yaml", ".devforgeai/context/tech-stack.md": "FIXTURE:tech-stack.md", ".devforgeai/context/source-tree.md": "FIXTURE:source-tree.md", ".devforgeai/context/dependencies.md": "FIXTURE:dependencies.md", ".devforgeai/brand/tokens.json": "FIXTURE:tokens.json", ".devforgeai/brand/logo.svg": "FIXTURE:logo.svg", "tools/api-symbols.sh": "FIXTURE:api-symbols-12.sh"}}, "expect": {"grader": "api_coverage", "args": {"path": ".devforgeai/releases/v0.3.0.yaml", "index": "docs/api/index.md", "symbols": ["open_project", "load_config", "Gate", "GateResult", "check_gate", "render_handoff", "Report", "ReportKind", "ingest", "allocate_id", "validate_doc", "resolve_platform"], "expect_symbols": 12, "expect_documented": 12, "brand": ["docs/brand/tokens.json", "docs/brand/logo.svg"]}}}
{"id": "rl-09-release-notes", "prompt": "/release v0.3.0", "timeout": 1500, "preflight": ["phase set release --id v0.3.0"], "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-none.toml", ".devforgeai/gates.toml": "FIXTURE:gates-release.toml", ".devforgeai/state.toml": "FIXTURE:state-verify.toml", ".devforgeai/stories/STORY-014.md": "FIXTURE:story-014-fix.md", ".devforgeai/stories/STORY-017.md": "FIXTURE:story-017-internal.md", ".devforgeai/stories/STORY-021.md": "FIXTURE:story-021-feature.md", ".devforgeai/stories/sprint.yaml": "FIXTURE:sprint-active.yaml", ".devforgeai/reports/STORY-014-verify.yaml": "FIXTURE:verify-pass-014.yaml", ".devforgeai/reports/STORY-017-verify.yaml": "FIXTURE:verify-pass-017.yaml", ".devforgeai/reports/STORY-021-verify.yaml": "FIXTURE:verify-pass-021.yaml", ".devforgeai/reports/STORY-014-qa.yaml": "FIXTURE:qa-clean-014.yaml", ".devforgeai/reports/STORY-017-qa.yaml": "FIXTURE:qa-clean-017.yaml", ".devforgeai/reports/STORY-021-qa.yaml": "FIXTURE:qa-clean-021.yaml", ".devforgeai/reports/FIND-004.md": "FIXTURE:find-004-stub.md", ".devforgeai/requirements.yaml": "FIXTURE:requirements-8-reqs.yaml", ".devforgeai/context/tech-stack.md": "FIXTURE:tech-stack.md", ".devforgeai/context/dependencies.md": "FIXTURE:dependencies.md"}}, "expect": {"grader": "release_notes", "args": {"path": ".devforgeai/releases/v0.3.0.yaml", "expect_kinds": {"STORY-014": "fix", "STORY-017": "internal", "STORY-021": "feature"}, "expect_requirements": ["REQ-002", "REQ-007", "REQ-011"], "summary_max": 72}}}
{"id": "rl-10-sendback-story-not-built", "prompt": "/release v0.3.0 --resume", "timeout": 1500, "preflight": ["phase set release --id v0.3.0"], "setup": {"files": {".devforgeai/config.toml": "FIXTURE:config-k8s.toml", ".devforgeai/gates.toml": "FIXTURE:gates-release.toml", ".devforgeai/state.toml": "FIXTURE:state-release-active.toml", ".devforgeai/stories/STORY-014.md": "FIXTURE:story-014-built.md", ".devforgeai/stories/STORY-017.md": "FIXTURE:story-017-ready.md", ".devforgeai/stories/sprint.yaml": "FIXTURE:sprint-active-014-017.yaml", ".devforgeai/reports/STORY-014-verify.yaml": "FIXTURE:verify-pass-014.yaml", ".devforgeai/reports/STORY-017-verify.yaml": "FIXTURE:verify-pass-017.yaml", ".devforgeai/reports/STORY-014-qa.yaml": "FIXTURE:qa-clean-014.yaml", ".devforgeai/reports/STORY-017-qa.yaml": "FIXTURE:qa-clean-017.yaml", ".devforgeai/releases/v0.3.0.yaml": "FIXTURE:release-v030-story-ready.yaml", ".devforgeai/requirements.yaml": "FIXTURE:requirements-8-reqs.yaml", ".devforgeai/context/tech-stack.md": "FIXTURE:tech-stack.md", ".devforgeai/context/dependencies.md": "FIXTURE:dependencies.md", "deploy/kubernetes/deployment.yaml": "FIXTURE:k8s-deployment.yaml", "deploy/kubernetes/service.yaml": "FIXTURE:k8s-service.yaml", "deploy/kubernetes/ingress.yaml": "FIXTURE:k8s-ingress.yaml", "deploy/kubernetes/kustomization.yaml": "FIXTURE:k8s-kustomization.yaml"}}, "expect": {"grader": "sendback_block", "args": {"to": "Verify", "gate_must_name": ["release-stories"], "next_prefix": "/verify STORY-017", "must_cite": ["STORY-017"], "forbidden_substrings": ["STORY-014"], "then_line": "/release v0.3.0 --resume", "found_must_name": [["STORY-017"]], "release_file": ".devforgeai/releases/v0.3.0.yaml", "release_signoff": "SEND_BACK", "stories_not_released": [".devforgeai/stories/STORY-014.md", ".devforgeai/stories/STORY-017.md"], "report_phase": ""}}}
```
### `evals/graders.py` — signatures and logic

Every function has the §9 signature `def <name>(workspace: str, transcript: str, args: dict) -> tuple[bool, str]`, reads only files under `workspace` and the `transcript` string, and returns `(passed, evidence)`. No function opens a network connection, starts a subprocess, calls a model, or uses a random source. Every prior-state value a grader compares against arrives in `args`; nothing is carried between cases by the runner.

- `release_shape(workspace, transcript, args)` — parse `args["path"]` as YAML with the standard-library-free minimal reader the module ships (`_yaml_load`, a mapping and sequence parser over the subset these files use). Compare `list(obj.keys())` to `args["keys"]` for equality including order. Check `obj["status"] == args["status"]`, `obj["id"]` equals the file stem, and `[s["id"] for s in obj["stories"]] == args["stories"]`. Check `obj["platform"]["target"] == args["platform_target"]` and `["source"] == args["platform_source"]`. Collect `os.path.basename(m["path"])` over `deploy.manifests` and compare the set to `args["manifest_basenames"]`; require each path to exist under `workspace` and be non-empty. Check `obj["signoff"]["gate"] == args["signoff_gate"]` and `checks_passed == checks_total`. Evidence: the key order, the story ids, and the manifest basenames found.
- `platform_resolution(workspace, transcript, args)` — parse `args["path"]`. Check `platform.target`, `platform.source`, and `platform.marker` against the three `args` values. Require every path in `args["required_files"]` to exist and be non-empty. When `args["env_pairs"]` is present, read the compose file, collect every `${NAME}` by the regex `\$\{([A-Z0-9_]+)\}`, read the env file, collect every leading `NAME=` by `^([A-Z0-9_]+)=`, and require the first set to be a subset of the second. Evidence: the three platform values and the two variable sets.
- `docs_layout(workspace, transcript, args)` — require every path in `args["required"]` to exist and be non-empty. Require `args["absent_dir"]` to hold no file. Parse `args["path"]`; for each dotted key in `args["empty_keys"]` require the resolved value to be `[]`, and for each in `args["empty_strings"]` require `""`. For each file and heading list in `args["headings"]`, split the file on `^## ` and compare the heading sequence for equality including order. Require `transcript` to hold one line beginning `Gate      ` whose next token is `args["transcript_gate"]`. Evidence: the paths checked, the empty keys, and the heading sequences.
- `sendback_block(workspace, transcript, args)` — require a `transcript` line beginning `Gate      SEND BACK to ` followed by `args["to"]`. When `args["gate_must_name"]` is present, require every string in it to appear on that line. Require a line beginning `Next      ` whose remainder starts with `args["next_prefix"]`, holds every string in `args["must_cite"]`, and holds no string in `args["forbidden_substrings"]`. Require a line beginning `Then      ` whose remainder equals `args["then_line"]`. When `args["found_must_name"]` is present, require, for each group in it, one `Found     ` line holding every string in the group. When `args["release_file"]` is present, parse it and require `signoff.gate == args["release_signoff"]`. For each path in `args["stories_not_released"]`, read the frontmatter and require `status` to be anything other than `released`. Evidence: the Gate, Next, Then, and Found lines, and the story statuses.
- `resume_untouched(workspace, transcript, args)` — the prior state arrives in `args["baseline_sha256"]`, a mapping from a workspace-relative path to the SHA-256 hex digest of that file's bytes before the run; the grader recomputes each digest with `hashlib.sha256` over the file bytes and requires equality, which is how an untouched-file assertion is made without a runner mirror. Parse `args["path"]` and require no entry of `notes.deferred` to carry `blocks_deployment: true` when `args["no_blocking_deferral"]` is true. Require a `transcript` line beginning `Gate      ` whose next token is `args["transcript_gate"]`, and a line beginning `Next      ` whose remainder equals `args["next_line"]`. Evidence: the per-path digest comparison and the two handoff lines.
- `version_refused(workspace, transcript, args)` — require every path in `args["absent"]` not to exist under `workspace`. Require every directory in `args["absent_dirs"]` to hold no file. Require every string in `args["transcript_must_contain"]` to appear in `transcript`. Require `transcript` to hold no line beginning `Gate      PASS`. Evidence: the paths checked and the matching transcript line.
- `api_coverage(workspace, transcript, args)` — parse `args["path"]`. Require `docs.api_symbols == args["expect_symbols"]` and `docs.api_documented == args["expect_documented"]`. Read `args["index"]`, split on `^## `, count the data rows of the table under `Symbols`, and require the count to equal `args["expect_symbols"]`. Collect every `^### ` heading text across the files listed in `docs.api` other than the index, and require the set to be a superset of `args["symbols"]`. Require every path in `args["brand"]` to exist and be non-empty. Evidence: the two counts, the row count, and the symbol names not found.
- `release_notes(workspace, transcript, args)` — parse `args["path"]`. Require `len(notes.entries) == len(stories)` and each entry's `story` to match the story at the same index. Require every `notes.entries[].kind` to be in `{"feature", "fix", "internal"}` and to agree with the rule: `fix` when the story's `requirements` is non-empty and the story file's `consumes` holds a `FIND-nnn`, `internal` when `requirements` is `[]`, `feature` otherwise. Require `notes.requirements` to be the ascending de-duplicated union of every entry's `requirements`. Require `notes.summary` to be at most 72 characters. Evidence: the entry kinds and the union comparison.

Every grader named above is called by exactly one of the nine cases, except `sendback_block`, which cases 4 and 5 both call with different `args`. `python -c "import graders"` needs only `hashlib`, `os`, and `re` from the standard library.

The four hex values in case 6's `baseline_sha256` are placeholders in this spec. The suite build step computes each one as the SHA-256 of the matching file under `evals/fixtures/` and writes it into `cases.jsonl` before the runner starts, so the digests always describe the fixtures the case actually copies. No digest is read from a previous case's workspace.

## Decisions

Each entry is a choice this spec made where conventions §1 through §10 were silent, a proposed addition to the §4 or `specs/01-cli.md` surface, a dependency on a spec being written in parallel, or a blocker.

### Proposed additions

1. **`devforgeai story list` is a proposed addition to the §4 subcommand table.** Exact grammar: `devforgeai story list [--status <s>[,<s>...]] [--sprint <SPRINT-nnn>] [--json] [--project <path>]`. It walks `.devforgeai/stories/STORY-*.md`, reads each frontmatter `id`, `status`, and `consumes`, and the first H1 as the title. `--status` filters to the listed values, each a member of the `story` doc-type status enum; an unknown value is `DFA-E230`, exit 3. `--sprint` filters to the ids under `stories[]` of `stories/sprint.yaml`. Human output is one line per story, `STORY-014  built  Order checkout`. `--json` `data`: `{"count":3,"stories":[{"id":"STORY-014","status":"built","title":"Order checkout","path":".devforgeai/stories/STORY-014.md","consumes":["REQ-007","REQ-011"]}]}`. Exit codes: 0 when the walk succeeded, including a count of zero; 1 on `DFA-E231` when `.devforgeai/stories/` is absent; 3 on `DFA-E230`; 5 on `DFA-E9xx`. Reason: the release set is every story at `built`, which is a walk of thirteen to hundreds of frontmatter blocks, and §1 rule 1 puts enumeration the CLI can perform outside the skill.
2. **Check kind `release_stories` is a proposed addition to the twenty-one-value enum of `specs/01-cli.md` `## Outputs`.** Keys, types, defaults, and pass condition are in `## Gate`. Weighed against `ids_resolve` with `from` and `to`, which tests set membership between two locations and reads neither a story's frontmatter `status` nor a second document's `gate.result`; against `set_cover`, whose `universe` would force the release to name every story rather than the subset at `built`; and against `fields_present` with `collection`, which confirms each `stories[]` entry is filled and takes the release file's own word for what it says. The gap is a per-entry lookup into two other documents, which no kind performs.
3. **Check kind `deploy_manifest` is a proposed addition to the same enum.** Keys, defaults, and the five per-platform rule sets are in `## Gate`. Weighed against `file_exists` plus `fields_present` under `required_when`, which covers the path and probe rules and leaves two gaps: the secret pattern is a regex over arbitrary lines of a non-document file, which `column_matches` performs over markdown table cells alone, and the compose `${NAME}`-to-`env.example` pairing compares two sets neither of which is a document location. The manifests also live at the project root rather than under `.devforgeai/`, which every `paths` and `path` key resolves against. Every rule is a parse or a pattern, so the check runs with no platform tool installed and no network.
4. **Check kind `docs_cover` is a proposed addition to the same enum.** Keys and defaults are in `## Gate`. Weighed against `set_cover`, whose `universe` names a location inside a document while the symbol universe is the stdout of `[release].api_symbols_command`; against `row_count_between` and `column_matches`, which read a count and a cell pattern and not a cross-file cover; and against `report_metric`, which reads the gate report the same `gate check` run is writing. The ratio is recomputed from the files on disk so that no subagent's own count decides the gate.
5. **The enum closes at twenty-four values.** Entries 2, 3, and 4 take it from twenty-one to twenty-four. A fourth candidate, a `version_ahead` kind, was folded into `doc validate` instead, per entry 6.
6. **The `release` row of the `specs/01-cli.md` `doc validate` doc-type table gains two content rules.** `id` matches `^v[0-9]+\.[0-9]+\.[0-9]+$`, which the row already states, and additionally the numeric triple compares strictly greater than the highest triple among the other `releases/v*.yaml` file names; a violation is `DFA-E217 (shape) or DFA-E218 (not greater than the previous version)`, exit 1, naming both versions. `previous_version` is either `""` or that highest triple. Reason: `doc_valid` already runs on `releases/{id}.yaml`, so the semver and monotonic rules cost one amendment rather than a fourth check kind, and the task's instruction to reuse kinds by name points the same way.
7. **`config.toml` gains a `[release]` table.** The ten keys with their types and defaults are in `## Gate`. `devforgeai init` writes it; `stack detect` leaves it untouched, in the way it already leaves `[[layer]]`, `[coverage]`, and `[[verifier]]`. Reason: conventions §1 rule 2 forbids naming a build or packaging tool anywhere in a skill, command, or subagent, so the build command, the package command, the symbol-enumeration command, the image reference, and the port are all config values.
8. **`config.toml` gains a `[[verifier]]` table for `deferral-auditor` at `phase = "release"`, `report_field = "verifiers.deferrals"`, `unit = "deferrals"`, `required = true`.** Verbatim in `## Gate`. Reason: `verifier_pass` resolves its named verifier through that registry, and without the entry `report ingest` returns `DFA-W411` and the gate fails with `DFA-E316`.

### Amendments to `specs/01-cli.md`

9. **The default `release` gate entry changes `requires = "verify"` to `requires = ""`.** `gate require <phase> <id>` tests the predecessor gate for one id, and the release id is a version while every verify report is per-story, so `reports/vX.Y.Z-verify.yaml` never exists and the preamble would exit 1 on every run. `phase set release --id vX.Y.Z` runs the same logic and would fail the same way. With `requires = ""` both pass, and the per-story verify verdict moves to `release_stories`, which is where a fan-out belongs. The `/release` command file carries no `gate require` line as a consequence.
10. **`produced_by` for this doc type is `releasing-software`, per conventions §4b.** No amendment is needed: the `release` row of the `doc validate` doc-type table in `specs/01-cli.md` already reads `releasing-software`, and `doc validate --producer-check` reads that row. Entry 21 of that spec's `## Decisions`, which gives the skill names as `devforgeai-<phase>`, is the outlier against its own table and against `specs/04-constitute.md` and `specs/08-design.md`.
11. **`release_stories.require_status` accepts `released` as well as `built`.** `phase set release` moves every story the release names to `released` at workflow step 14, and the CI job re-evaluates the same gate over the committed tree afterwards. A check that accepted `built` alone would turn the required status check red on a release that passed locally.
11b. **The release row of the `Next` and `Then` transition table gains a `Then` value on SEND BACK.** `specs/01-cli.md` gives that row as `Next /verify <STORY-nnn>` with `Then` omitted. Conventions §4c fixes the returning form as `/<downstream> <ID> --resume`, and a send-back with no printed return path leaves the user to reconstruct it, which §1 rule 7 forbids. The row becomes `Next /verify <STORY-nnn> --remedy <ids>`, `Then /release <vX.Y.Z> --resume`. The PASS row is unchanged and is what `## Handoff` shows: `Next /reflect <vX.Y.Z>`, `Then` omitted.
11c. **This spec follows the twenty-one-value check-kind enum table of `specs/01-cli.md` `## Outputs`, not the default `gates.toml` body of its `## Gate`.** The two disagree at the time of writing: the enum table drops `status_is` in favour of `field_in_enum` and the compiled-minimums table names `field_in_enum` for explore, while the default file body still writes `status_is` in seven places, including the `release` entry this spec replaces. The gate in `## Gate` uses `field_in_enum`. If the reconciliation restores `status_is`, the `release-status` check reverts to `kind = "status_is"`, `doc = "releases/{id}.yaml"`, `values = ["released"]`, and nothing else in this spec moves.
11d. **`fields_present` is read as taking `path` for the document and `collection` for the dotted location inside it.** `specs/01-cli.md` states that a `path` key names a document and that `field`, `from`, `to`, `cover`, and `universe` name locations, and lists `collection` in neither sentence. The `release-entries` and `release-no-manifest` checks are written on that reading. If `collection` turns out to carry the document name as well, both checks drop their `path` line and prefix the collection with `releases/{id}.yaml`.

### Silences filled

12. **The platform enum is closed at five values: `kubernetes`, `compose`, `github-actions`, `vps`, `none`.** Conventions §2 forbids an open list, and each value has a manifest set and a `deploy_manifest` rule set behind it. A sixth platform is a spec amendment that adds both.
13. **`github-actions` as a platform value and `.github/workflows/devforgeai-release.yml` are two different things.** The platform value means the deployment itself is a workflow, written to `.github/workflows/deploy.yml`. The gate workflow is written for every platform value, `none` included, whenever `[release].ci` is `github-actions`, and it runs `devforgeai gate check --phase release` and writes nothing. Their job names differ, `deploy` and `devforgeai-release-gate`, and the deploy job's `needs` names the gate job.
14. **The release set is every story at frontmatter `status: built`, with no date or tag comparison.** `phase set release` is the one writer that moves a story to `released`, so `built` is exactly the set Verify passed and no prior release consumed. "Since the last release" needs no computation.
15. **`devforgeai phase set release --id vX.Y.Z` runs twice, at workflow steps 2 and 14.** The first call sets `[active].release`, which is what `report ingest` resolves the target report from when `SubagentStop` fires for `deferral-auditor`; the release file does not exist yet, so the story-status half returns `DFA-W210` and writes nothing. The second call runs after the gate passed and is what moves the stories to `released`. Both calls are idempotent for `state.toml`.
16. **When `[release].platform` is absent or `""`, detection runs the four-marker ordered list of workflow step 7, and an unmatched project asks one `AskUserQuestion` with the five options.** The answer is written back to `[release].platform` in `.devforgeai/config.toml`, so a later release reads it from the file rather than asking again. `config.toml` is not in the `doc validate` doc-type table, so the PreToolUse producer check skips it, and `stack detect` preserves tables it does not own.
17. **The documentation layout is four directories and a fixed section list per file, in `## Outputs` section 3.** `docs_cover` reads the API pages by H3 heading text, so the shape is a contract rather than a suggestion. One page per `(stack id, source root)` pair rather than per root, because two `[[stack]]` tables can carry the same root string.
18. **The user guide is one file with one H3 per story, not one file per story.** A release is bounded by the stories Verify passed, so the file is bounded, and a reader following a release looks at one page.
19. **`artifacts[]` carries `path`, `kind`, and `bytes`, and no digest.** No §4 subcommand computes a file digest for a project artifact, and §2 forbids naming a hashing tool in skill prose. `cli/DIGEST` covers the framework binary, not a project's output. A digest field is an amendment that adds a subcommand first.
20. **`notes.entries[].kind` is derived by rule, not asked.** `fix` when the story's `consumes` carries a `FIND-nnn`, `internal` when it carries no `REQ-nnn`, `feature` otherwise. Story frontmatter has no kind key and §5 fixes the key list, so the rule reads what is already there.
21. **The handoff slug renders as `-` for this phase.** The rendering algorithm of `specs/01-cli.md` takes the slug from the phase document's first H1 and gives `-` when there is none; a YAML release file carries no H1. Both examples in `## Handoff` show it.
22. **This phase applies no deployment.** It writes manifests and the gate workflow. Applying one needs a credential inside a Claude session, which §2's aspiration rule excludes, and the tag push is what the platform reacts to. `deploy_manifest` validates by parsing and pattern, never by invoking a platform tool.
23. **`no_open_questions` is absent from the release gate.** A release carrying an unanswered platform question is one `deploy_manifest` already fails, and a second check would report the same fact twice in the handoff's `Gate` line.
24. **Release receives a send-back from no phase.** §5 puts Reflect alone downstream, and Reflect's `REC-nnn` output is advisory and sets no gate. The command's `argument-hint` carries `--resume` and no `--remedy` form as a result. A Reflect recommendation that concerns this phase lands as a human edit to `[release]`, to `gates.toml`, or to this spec.
25. **`deferral-auditor` asks a narrower question than `deferral-validator` did.** Verify already decided whether a deferral was justified. This phase decides only whether a deferral blocks deployment on `platform.target`, which is the one fact a release adds. The four-level severity ladder collapses to `block` and `info`, because `verifier_pass` reads `passed / total`.
26. **`deployment-engineer-platform-patterns` is replaced, not adapted.** It is a reference document at `C:\Users\bryan\.claude\agents\deployment-engineer\references\platform-patterns.md`, not an agent: §10 requires a JSON output schema and a reference has no output. Its content becomes this skill's `templates/`, where the `deploy_manifest` check can be written against fixed text.
27. **`documentation-writer` is split into two agents rather than adapted into one.** `api-doc-writer` and `guide-writer` run in parallel at workflow step 10 and read disjoint inputs, which one agent covering five output kinds could not do.

### Dependencies on specs being written in parallel

Each of these uses the conventions §5 contract and is listed so the orchestrator can reconcile it against the spec that lands.

28. **Plan (`specs/05-plan.md`).** This spec assumes `stories/STORY-nnn.md` carries §5 frontmatter with `status` in the doc-type enum `draft`, `ready`, `building`, `built`, `verified`, `released`, a `consumes` array holding `REQ-nnn` and, on a remediation story, `FIND-nnn`, a first H1 whose text becomes `stories[].title`, and acceptance criteria the `guide-writer` agent can read as ordered steps. It assumes `stories/sprint.yaml` carries `stories[]` of `{id, status}` in that key order. It reads no other section of a story and no other key of the sprint file. If Plan fixes a different H1 form, `stories[].title` and the guide's H3 text follow it.
29. **Build (`specs/06-build.md`).** This spec reads nothing Build writes. `reports/STORY-nnn-build.yaml` is not an input: the build verdict reaches this phase through the verify gate report, which the verify gate already required to pass. If Build's report gains a field this phase should carry into `artifacts[]`, that is an amendment to `## Outputs`.
30. **Verify (`specs/07-verify.md`).** This spec assumes `reports/STORY-nnn-qa.yaml` carries a `findings[]` array whose entries hold `id` as a `FIND-nnn`, a severity, a one-line summary, and a marker that the finding was deferred together with the stated reason. `deferral-auditor` reads exactly those four fields. It assumes the CLI's `reports/STORY-nnn-verify.yaml` carries `gate.result`, which `specs/01-cli.md` already fixes. If Verify names the deferral marker differently, `deferral-auditor`'s input list and the `stories[].deferrals` derivation follow it; nothing else in this spec changes.
31. **Verify's own deferral handling.** Both phases derive an agent from `deferral-validator`. This spec registers `deferral-auditor` at `phase = "release"` with `report_field = "verifiers.deferrals"`. If Verify registers a verifier under the same name, the catalog spec `specs/11-subagent-catalog.md` resolves the collision; the two are distinct registrations and neither invokes the other.
32. **Discover (`specs/03-discover.md`).** This spec reads one field, `requirements[].statement`, keyed by `REQ-nnn`. `specs/08-design.md` reads `requirements[]`, `epics[]`, and `personas[]` from the same file, so the array name is settled; the field name is the assumption to reconcile.

### Blockers

33. None. Every step of this phase is a file read, a file write, a `devforgeai` invocation, an `Agent` call, or one `AskUserQuestion`, all inside the §2 primitive list. The CI job runs the same binary the local run does, with no capability the terminal session lacks.

34. **Platform names in prose.** The five platform targets, and the manifest file names each one fixes, are named in this spec, in `skills/releasing-software/SKILL.md`, and in its four platform reference files. Conventions §1 rule 2 sanctions them: a deployment platform is not a thing `stack detect` detects, it does not vary with the project's language, and a manifest for a named platform is written by naming it. The rule still holds for everything else — no language, package manager, test runner, linter, or build tool appears here, and every command this phase runs comes from `config.toml` `[release]`.
