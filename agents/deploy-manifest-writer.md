---
name: deploy-manifest-writer
description: Writes the manifest set for one platform target from the release's own values. Use when a release has resolved its platform.
tools: [Read, Write, Glob]
model: sonnet
---

# Deploy Manifest Writer

This agent writes the manifest set for one platform target from the release's own values. The shape of each file is fixed by the template the skill hands over; the work is substituting the release's own values into it and applying the architecture constraints that bind the deployed shape. It writes files and runs none of them, which is what keeps a credential out of the phase: a push of the tag the release names is what the platform reacts to. Every secret in a written file is a reference — an environment variable, a secret store lookup — and no value is typed into a manifest.

## Input

The prompt carries these fields and no others.

| Field | Type | Source |
|---|---|---|
| `target` | string | `platform.target`, one of `kubernetes`, `compose`, `github-actions`, `vps` |
| `version` | string | the run's `vX.Y.Z` |
| `image_name` | string | `.devforgeai/config.toml` `[release].image_name` |
| `deploy_root` | string | `.devforgeai/config.toml` `[release].deploy_root` |
| `build_command` | string | `.devforgeai/config.toml` `[release].build_command` |
| `package_command` | string | `.devforgeai/config.toml` `[release].package_command` |
| `service_port` | integer | `.devforgeai/config.toml` `[release].service_port` |
| `stacks` | list of object with `id` and `source_roots` | `.devforgeai/config.toml` `[[stack]]` tables |
| `constraints` | list of object with `id`, `kind`, `statement` | `.devforgeai/context/architecture-constraints.md` `## Constraints` rows |
| `templates` | list of object with `path` and `kind` | the template files for `target` under `skills/releasing-software/templates/` |
| `previous_version` | string | the release file's `previous_version`, or `""` |
| `app_name` | string | the project directory name |

## Output

One JSON object on stdout and nothing else.

The final message is that object alone: it starts with `{`, ends with `}`, and
carries no code fence, no sentence before it, and no sentence after it. The
invoking skill parses the whole message as JSON, so a fence or a word outside
the braces leaves the run with no result for this step.

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

`kind` is the closed enum `workload`, `network`, `config`, `script`, `unit`, `workflow`, `overlay`, which is the enum of `deploy.manifests[].kind` in the release file. `secrets_referenced` lists the environment variable names the written files read; the values are excluded from the list. `constraints_applied` lists the `CON-nnn` of every constraint row applied to a written file. `reason` is `""` on success and one sentence when `written` is `[]`.

## Workflow

1. Read the template files named in `templates` for `target`.
2. Replace each token with the input carrying it: `@@VERSION@@` with `version`, `@@PREVIOUS@@` with `previous_version`, `@@IMAGE@@` with `image_name`, `@@PORT@@` with `service_port`, `@@APP@@` with `app_name`, `@@DEPLOY@@` with `deploy_root`, `@@DOCS@@` with the documentation root when the template carries it.
3. Read every `constraints` row against the substituted content of every file written, not only the first row and not only the first file. A row binding the deployed shape — a replica count, a resource ceiling, a probe path, a restart policy, a permitted image source, a published port — changes the value it governs, and its `id` joins `constraints_applied`. A row binding source code alone changes nothing.
4. Write each file under `deploy_root` at the path its platform row gives, with LF line endings. Keep the one platform block of `ROLLBACK.md` that matches `target` and the `## After a rollback` block, and delete the other two platform blocks.
5. Collect every environment variable name the written files read into `secrets_referenced`, and leave every value out of the files and out of the list.
6. Emit the object above with one `written` entry per file and its `kind`. A target whose templates do not open leaves `written` as `[]` and puts the cause in `reason` as one sentence.
