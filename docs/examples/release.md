# `/release` — a worked walkthrough

Phase 6. Every story at `status: built` becomes one named version: deployment manifests under the configured deploy root, an API reference, a user guide and an architecture note under the configured docs root, the brand assets, and `.devforgeai/releases/vX.Y.Z.yaml` for Reflect to read.

Release applies no deployment, holds no credential, and runs no build of its own. Every command it would otherwise name comes from `.devforgeai/config.toml` `[release]`. It edits no story, no QA report, no ADR, and no context file — a defect in any of those leaves as a SEND BACK to Verify citing ids.

---

## Entry

| Form | Run kind |
|---|---|
| `/release vX.Y.Z` | fresh — assemble every story at `status: built` into that version |
| `/release vX.Y.Z --resume` | resume — continue the run the last send-back stopped, narrowed to the ids that report cites |

`$1` is the version. A `$1` that does not match `^v[0-9]+\.[0-9]+\.[0-9]+$` stops the run with one `Blocked` line giving the accepted form `vX.Y.Z`.

There is **no `--remedy` form.** Reflect is the only phase downstream, its `REC-nnn` output sets no gate, and nothing arrives here as a send-back. A recommendation that concerns this phase arrives as a human edit to `config.toml` `[release]`, to `gates.toml`, or to the spec, which the next `/release` run reads from the file.

One preamble line, and it is the release set:

```
!`devforgeai story list --status built --json`
```

It prints `{"count":n,"stories":[{"id","status","title","path","consumes"}]}` and exits 0 at a count of zero, which step 4 handles. Exit 1 on `DFA-E231` means `.devforgeai/stories/` is absent and the body does not load. The line allocates no id — this phase allocates none, because the version is `$1` and every other id it names was allocated upstream.

```
$ devforgeai story list --status built
```

Reproduced on a project with no built story: no output, exit 0.

---

## The `[release]` table

Ten keys, and everything the phase would otherwise have to name:

```toml
[release]
platform = ""
ci = "github-actions"
deploy_root = "deploy"
docs_root = "docs"
build_command = ""
package_command = ""
artifact_paths = []
api_symbols_command = ""
image_name = ""
service_port = 8080
```

Reproduced from `.devforgeai/config.toml`.

---

## The exchange

```
> /release v0.1.0
```

**Step 1 — establish the run.** The version `$1` and a run kind of `fresh`, or `resume` when `$ARGUMENTS` holds `--resume`.

**Step 2 — activate the version.**

```
devforgeai phase set release --id v0.1.0
```

Sets `state.toml` `current_phase` to `release` and `[active].release` to `$1`, which is where `report ingest` resolves its target report at step 6. The release file does not exist yet, so the story-status half of the call writes nothing and returns `DFA-W210` at exit 0.

| Exit | Code | Means |
|---|---|---|
| 1 | `DFA-E300` | `gates.toml` holds no `release` gate; the run stops |
| 1 | `DFA-E320` | a story in the set has no verify gate at PASS; the run stops with one `Blocked` line naming `/verify STORY-nnn` for the story the stderr names, and no release file is written |

That second refusal is the common shape of a release failure, and it happens **before** the version is assembled.

**Step 3 — read the last verdict.** Resume runs only. Read `.devforgeai/reports/v0.1.0-release.yaml` and take the `STORY-nnn` and `FIND-nnn` ids of its `findings[]`. Those ids narrow step 4's set to those stories and step 10 to the pages those stories touch. Every file already written under `[release].deploy_root`, `[release].docs_root`, and `.github/workflows/` is left byte-identical. A report that is absent or whose `status` is not `send_back` makes the run proceed as `fresh`.

**Step 4 — assemble the release set.** Read each listed `stories/STORY-nnn.md` for its first H1, its frontmatter `consumes`, and its `status`. Then read `stories/sprint.yaml` `stories[]` and record any `{id, status}` pair whose status is `built` and whose id the CLI list omits — the sprint's own list is the cross-check on the walk.

Output: the release set ascending by id, each with `title`, `requirements` (the `REQ-nnn` of `consumes`, ascending), and the two report paths.

An empty set stops the run:

```
Blocked   you: no story is at status built; run /verify STORY-nnn first
```

and writes no release file. A story whose `devforgeai doc load story STORY-nnn` returns `DFA-E200` leaves the set and adds one `open_questions` line naming the id.

**Step 5 — read the verify verdicts.**

```
devforgeai report show STORY-001 verify
```

once per story, taking `gate.result` into `stories[].verify_result`. Exit 1 on `DFA-E400` records `verify_result: FAIL` and carries the story into step 6, which is what produces the send-back.

**Step 6 — audit the deferrals.** `deferral-auditor`, this phase's registered verifier, takes the version, the `FIND-nnn` entries each `reports/STORY-nnn-qa.yaml` marks deferred with their stated reason, each story's AC texts, the `## Approved dependencies` rows of `context/dependencies.md`, and `platform.target` when a resume run or a config value already fixes it.

Its findings each carry a `confidence` and a `kind` from a closed five: `blocks_deployment`, `unjustified`, `circular`, `missing_report`, `accepted`. `SubagentStop` runs `devforgeai report ingest deferral-auditor -`, which writes `verifiers.deferrals` into `.devforgeai/reports/v0.1.0-release.yaml`.

A story whose QA report is absent contributes one finding of `kind: missing_report` at `severity: block`, and the audit continues over the rest.

**Step 7 — resolve the platform.** `config.toml` `[release].platform` wins when it is not `""`, giving `source: config` and `marker: ""`. When the key is absent or `""`, the marker list is tested in order and the first match is taken, with `source: detected`:

| Order | Marker glob | Gives |
|---|---|---|
| 1 | `**/kustomization.yaml`, `**/kustomization.yml`, `**/Chart.yaml` | `kubernetes` |
| 2 | `docker-compose.yaml`, `docker-compose.yml`, `compose.yaml`, `compose.yml` at the project root | `compose` |
| 3 | `.github/workflows/deploy.yml`, `.github/workflows/deploy.yaml` | `github-actions` |
| 4 | `<deploy_root>/vps/deploy.sh` | `vps` |

When no marker matches, one `AskUserQuestion` with the header `Platform` and the five options `Kubernetes`, `Compose`, `GitHub Actions`, `VPS`, `None` supplies the value with `source: asked`. That value is then written back to `[release].platform` in `config.toml`, so the next release reads it from the file.

A free-text answer matching no label re-asks once, then the run takes `none` and adds one `open_questions` line naming the header. The `PreToolUse` producer check covers every path under `.devforgeai/`, `config.toml` included; a refusal leaves `platform.target` at the answered value for this run alone and adds one `open_questions` line naming the key that was not written.

**Step 8 — write the deployment manifests.** `deploy-manifest-writer` takes `platform.target`, six `[release]` values, the `[[stack]].id` and `source_roots` values, the `## Constraints` rows of `context/architecture-constraints.md`, the version, and the template paths for the target. It returns `devforgeai/deploy-manifest/1` and writes the files the platform's reference file lists, which fill `deploy.manifests[]`.

On `platform.target` of `none` the agent is not invoked, `deploy.manifests` is `[]`, and `deploy.rollback` is `""`. An agent returning `written: []` with a `reason` falls back to the platform's template files with `@@IMAGE@@`, `@@PORT@@`, and `@@NAME@@` substituted, and the reason recorded in `open_questions`.

**Step 9 — write the gate workflow.** Runs when `[release].ci` is `github-actions`, for **every** value of `platform.target` including `none`. Write `templates/ci/devforgeai-release.yml` to `.github/workflows/devforgeai-release.yml` with `@@VERSION@@` replaced by `$1`, `@@DEPLOY@@` by `[release].deploy_root`, and `@@DOCS@@` by `[release].docs_root`, then fill `deploy.ci_workflow` and `deploy.ci_check_name` with that path and the job name `devforgeai-release-gate`.

That job name is the string a repository administrator adds as a required status check. With `[release].ci` at `none` the file is not written and both keys are `""`.

The gate workflow is separate from a `github-actions` deploy target: the first runs `gate check --phase release`, the second deploys.

**Step 10 — write the documentation.** `api-doc-writer` and `guide-writer`, in parallel.

`api-doc-writer` gets the output lines of `[release].api_symbols_command`, each `<kind>\t<symbol>\t<path>`, the `[[stack]]` tables, `[release].docs_root`, and the version. With that command at `""`, it writes `docs/api/index.md` with an empty `## Symbols` table, `docs.api_symbols` and `docs.api_documented` are both `0`, and the gate's `docs_cover` check records `SKIP` with `reason: no_api_symbols_command`.

`guide-writer` gets the release set with each story's ACs, every `adr/ADR-nnn.md` at `status: accepted`, and four context H2 sections: `## Languages` of `tech-stack.md`, `## Layers` of `source-tree.md`, and `## Approved dependencies` and `## License policy` of `dependencies.md`.

Then `<docs_root>/README.md` from `templates/docs/README.md`, and `.devforgeai/brand/tokens.json` and `brand/logo.svg` copied to `<docs_root>/brand/`. `docs.brand` is `[]` when `.devforgeai/brand/` holds neither file.

**Step 11 — write the release file.** `previous_version` is the highest `v<X>.<Y>.<Z>` among the `.devforgeai/releases/v*.yaml` names other than `$1.yaml`, or `""` when the directory holds no other release.

```yaml
schema: devforgeai/release/1
id: v0.1.0
phase: release
status: draft
produced_by: releasing-software
consumes: [STORY-001]
open_questions: []
previous_version: ""
released_at: ""
stories:
  - id: STORY-001
    title: Match a statement line to a ledger entry
    status: built
    qa_report: .devforgeai/reports/STORY-001-qa.yaml
    verify_report: .devforgeai/reports/STORY-001-verify.yaml
    verify_result: PASS
    requirements: [REQ-001, REQ-002]
    deferrals: []
artifacts: []
platform:
  target: none
  source: detected
  marker: ""
deploy:
  manifests: []
  rollback: ""
  ci_workflow: .github/workflows/devforgeai-release.yml
  ci_check_name: devforgeai-release-gate
docs:
  ...
notes:
  ...
signoff:
  gate: NOT_RUN
  report: ""
  checks_passed: 0
  checks_total: 0
  at: ""
```

Shaped from `templates/release.yaml`. Sixteen top-level keys, no others: the seven envelope keys, then `previous_version`, `released_at`, `stories`, `artifacts`, `platform`, `deploy`, `docs`, `notes`, `signoff`.

Each `stories[]` entry carries all eight keys. The gate's `fields_present` check reads them, and an entry short one is `DFA-E332` naming the story and the key. The template's `STORY-000` entry is the shape rather than a row to keep.

**Step 12 — move to released.** Edit the file to carry `status: released` and `released_at` at the current UTC instant in RFC 3339. `status` runs `draft` then `released`; there is no third value.

**Step 13 — run the gate, then write the signoff.**

```
devforgeai gate check --phase release --id v0.1.0
```

writes `.devforgeai/reports/v0.1.0-release.yaml` with its ten check entries and the ingested `verifiers.deferrals` block. The CLI writes nothing into the release file, so after `gate check` returns the model edits that file's `signoff` block from the report: `gate` takes the report's `result`, `report` takes its path, `checks_passed` and `checks_total` take the counts of `checks[]` at `pass` and in total, and `at` takes the report's `at`.

**This edit happens on every exit code**, before the run stops or continues. A release file left at the template's `gate: NOT_RUN` with zero counts is one where the edit did not run.

Then exit 0 goes to step 14, exit 2 goes to the send-back, and exit 1 leaves the file at `status: released` with the `signoff` block just written and the run stops. Exit 1 on `DFA-E303` means `gates.toml` sits below a compiled floor; no report is written and `signoff` keeps `gate: NOT_RUN`.

**Step 14 — move the stories.**

```
devforgeai phase set release --id v0.1.0
```

a second time. That call moves every `STORY-nnn` under `stories` in the release file to frontmatter `status: released`, which is why the `release_stories` check accepts `released` beside `built`. `DFA-W210` at exit 0 on a story the release names and the directory does not hold; the `ids_resolve` check has already reported that id.

**Step 15 — validate the release file.**

```
devforgeai doc validate .devforgeai/releases/v0.1.0.yaml
```

Exit 1 on `DFA-E217` (`<path> id '<value>' is not vX.Y.Z`) or `DFA-E218` (`<path> version <value> is not greater than <previous>`) stops the run with one `Blocked` line naming both versions.

---

## The gate

Ten checks. Their ids, from `.devforgeai/gates.toml`:

| Check id | Kind | `on_fail` |
|---|---|---|
| `release-docs` | `doc_valid` | fail |
| `release-file` | `file_exists` | fail |
| `release-status` | `field_in_enum` | fail |
| `release-set` | `length_between` | fail |
| `release-entries` | `fields_present` | fail |
| `release-ids` | `ids_resolve` | **send_back** |
| `release-stories` | `release_stories` | **send_back** |
| `release-deferrals` | `verifier_pass` | **send_back** |
| `release-manifest` | `deploy_manifest` | fail |
| `release-docs-cover` | `docs_cover` | fail |

Reproduced from the file `init` wrote. The gate block carries `requires = ""` and `send_back_to = "verify"`.

`requires = ""` is why `releasing-software` carries no `gate require` preamble: this gate has no single predecessor subject to test, and every story's verify gate is checked by `phase set` at step 2 instead.

Two further checks worth knowing: a manifest holding a literal secret is `DFA-E343` (`<path>:<line> holds a literal secret`), and a `deploy_manifest` failure is `DFA-E342` (`<path> <is absent | does not parse | has no <key>> for platform '<platform>'`).

---

## The handoff

```
Phase     6 · Release         v0.1.0 · -
Done      1 stories · v0.1.0
Gate      PASS  10 checks
Verified  deferral-auditor · 3/3 deferrals

Next      /reflect
Blocked   none

Full report: .devforgeai/reports/v0.1.0-release.yaml
```

The `Done` line counts the entries under `stories` in the release file and names the version. `Next /reflect` carries no argument on a PASS, because Reflect's window is yours to choose.

---

## Send-back

Release sends back to Verify and to no other phase. Three of the ten gate checks resolve `on_fail = "send_back"`:

| Producing check | Condition | Ids cited |
|---|---|---|
| `release-deferrals` | `deferral-auditor` returned a finding at `severity: block`, so `passed < total` | the `FIND-nnn`, and on the same line the `STORY-nnn` the finding names |
| `release-stories` | a story in the release is at neither `built` nor `released`, or its `reports/STORY-nnn-verify.yaml` is absent or its `gate.result` is not `PASS` | the `STORY-nnn` ids, ascending |
| `release-ids` | a `STORY`, `REQ`, `ADR`, or `FIND` id the release file references resolves to no definition | the unresolved ids |

On any of the three the release file keeps `status: released` and carries `signoff.gate: SEND_BACK`, the manifests and the documentation stay on disk byte-identical, `[active].release` keeps the version, and step 14 does not run — so no story moves to `released`.

```
Next      /verify STORY-nnn --remedy FIND-nnn,FIND-nnn
Then      /release v0.1.0 --resume
```

A deferral the auditor marks `kind: accepted` is **not** a send-back. That finding carries `severity: info`, lands in `notes.deferred[]`, counts toward `passed`, and ships in the release notes so a reader sees what was left out. There is no `blocks_deployment` boolean on a finding: `kind: blocks_deployment` at `severity: block` says it once, and the routing is the gate's.

---

## `--resume`

Re-enters at step 1 and runs step 3, which reads `reports/v0.1.0-release.yaml` `findings[]` and narrows the rest of the run to the `STORY-nnn` and `FIND-nnn` those findings cite: step 4 assembles only those stories, step 6 re-audits only their deferrals, and step 10 rewrites only the pages they touch.

Every other file under `[release].deploy_root`, `[release].docs_root`, and `.github/workflows/` keeps the bytes it had, because a send-back names a defect upstream of the manifests and the manifests did not change. A release file at `status: draft` moves forward; one at `released` is rewritten in place with the repaired values. Step 2 runs again and is idempotent for `state.toml`.

---

## Files this phase owns

| Document | Path | Template |
|---|---|---|
| Release | `.devforgeai/releases/vX.Y.Z.yaml`, one per version | `templates/release.yaml` |
| Deploy manifests | under `[release].deploy_root`, one set per platform | `templates/kubernetes/*`, `templates/compose/*`, `templates/github-actions/deploy.yml`, `templates/vps/*` |
| Rollback note | `<deploy_root>/<platform>/ROLLBACK.md`, and `<deploy_root>/ROLLBACK.md` for `github-actions` | `templates/ROLLBACK.md` |
| Gate workflow | `.github/workflows/devforgeai-release.yml` | `templates/ci/devforgeai-release.yml` |
| Documentation set | under `[release].docs_root` | `templates/docs/README.md`, `api-index.md`, `api-page.md`, `guide-index.md`, `architecture-index.md` |

Every `@@NAME@@` token in a template is replaced at write time: `@@VERSION@@` with `$1`, `@@PREVIOUS@@` with `previous_version`, `@@IMAGE@@` with `[release].image_name`, `@@PORT@@` with `[release].service_port`, `@@APP@@` with the project directory name, `@@DOCS@@` with `[release].docs_root`, `@@DEPLOY@@` with `[release].deploy_root`, and in an API page `@@STACK@@` with the `[[stack]].id` and `@@ROOT@@` with the source root. Shell files are written with LF line endings.

`notes.entries[].kind` is derived rather than asked, because story frontmatter carries no kind key: `fix` when the story's `consumes` holds a `FIND-nnn`, `internal` when its `consumes` holds no `REQ-nnn`, `feature` in every other case. `notes.summary` is at most 72 characters and `notes.deferred[].summary` at most 90.

`artifacts[]` lists the files the release ships, each with `path`, `kind` from `build | package | brand | docs | manifest`, and `bytes`. The glob patterns of `[release].artifact_paths` say which files those are.
