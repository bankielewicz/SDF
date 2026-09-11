# Platform `github-actions`

Read this at workflow step 8 when `platform.target` is `github-actions`. Here the deployment itself is a workflow the repository host runs, so the release writes that workflow and one rollback note, and writes no manifest under a platform directory.

## Files written

| Path | `kind` | Template | Holds |
|---|---|---|---|
| `.github/workflows/deploy.yml` | `workflow` | `templates/github-actions/deploy.yml` | a tag trigger on `v*.*.*`, one job named `deploy` whose `needs` names `devforgeai-release-gate`, the production environment, the image reference, and the deploy token read from the repository's secret store |
| `<deploy_root>/ROLLBACK.md` | `script` | `templates/ROLLBACK.md` | the block matching where the workflow lands, and the `## After a rollback` block |

The rollback note sits at the root of `config.toml` `[release].deploy_root` for this platform rather than under a platform directory, because the workflow lives under `.github/workflows/` and the deploy root holds no sibling for it. `deploy.rollback` in the release file is that path.

## Two workflows, one repository

This platform's `deploy.yml` and the gate workflow of workflow step 9 are different files with different jobs.

| | `.github/workflows/deploy.yml` | `.github/workflows/devforgeai-release.yml` |
|---|---|---|
| Written when | `platform.target` is `github-actions` | `config.toml` `[release].ci` is `github-actions`, for every platform value including `none` |
| Job name | `deploy` | `devforgeai-release-gate` |
| What it does | deploys the version the tag names | runs `devforgeai gate check --phase release` and writes nothing |
| Template | `templates/github-actions/deploy.yml` | `templates/ci/devforgeai-release.yml` |

The `needs: devforgeai-release-gate` line of the deploy job is what ties the two together: a deployment starts on a release whose gate job succeeded and on no other. `deploy.ci_workflow` and `deploy.ci_check_name` in the release file name the second file and its job, which is the string a repository administrator adds as a required status check.

## Token substitution

`@@VERSION@@` is `$1` and appears in the workflow name and in the image reference. `@@IMAGE@@` is `config.toml` `[release].image_name`. `@@DEPLOY@@` is `[release].deploy_root` and forms the path the deploy step runs. `@@APP@@` and `@@PREVIOUS@@` appear in the rollback note alone.

## What the gate check reads

`deploy_manifest` for this platform is a parse and a lookup, run with no repository host reachable:

- `.github/workflows/deploy.yml` exists and parses as YAML;
- it carries `on` and `jobs`, and a job whose `needs` names `devforgeai-release-gate` while `deploy.ci_workflow` is non-empty;
- no line of a listed file matches the secret pattern `(?i)(password|secret|token|api[_-]?key|private[_-]?key)\s*[:=]\s*["']?[A-Za-z0-9+/=_-]{12,}` other than as a `${...}` reference, a `secretKeyRef`, a `valueFrom`, or a `$(...)` substitution. A literal match is `DFA-E343`.

The template reaches its token through `${{ secrets.APP_DEPLOY_TOKEN }}`, which the `${...}` exemption covers. A token pasted into the file fails the check.

## The architecture constraints

The `## Constraints` rows of `.devforgeai/context/architecture-constraints.md` reach `deploy-manifest-writer` as input. A row binding the deployment trigger, the environment name, or the permitted image source is applied to the workflow and its `CON-nnn` is listed in `constraints_applied`. A row binding source code alone is left out of that list.
