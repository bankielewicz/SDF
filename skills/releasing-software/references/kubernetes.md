# Platform `kubernetes`

Read this at workflow step 8 when `platform.target` is `kubernetes`. A cluster applies declarative workload manifests, so the release writes the workload, its network surface, the overlay that pins the image tag, and one rollback note.

## Files written

Rooted at `config.toml` `[release].deploy_root`, `deploy` by default. Each row is one entry of `deploy.manifests[]` in the release file, in this order.

| Path | `kind` | Template | Holds |
|---|---|---|---|
| `<deploy_root>/kubernetes/deployment.yaml` | `workload` | `templates/kubernetes/deployment.yaml` | two replicas, the container image at `@@IMAGE@@:@@VERSION@@`, the container port, the environment from a secret reference, a readiness probe, a liveness probe, and resource requests and limits |
| `<deploy_root>/kubernetes/service.yaml` | `network` | `templates/kubernetes/service.yaml` | a `ClusterIP` service selecting the workload's labels, port 80 onto the container port |
| `<deploy_root>/kubernetes/ingress.yaml` | `network` | `templates/kubernetes/ingress.yaml` | one host rule with a prefix path onto the service's port 80 |
| `<deploy_root>/kubernetes/kustomization.yaml` | `overlay` | `templates/kubernetes/kustomization.yaml` | the three resources above and an `images` entry pinning `@@IMAGE@@` to `@@VERSION@@` |
| `<deploy_root>/kubernetes/ROLLBACK.md` | `script` | `templates/ROLLBACK.md` | the `## Kubernetes` block and the `## After a rollback` block; the `## Compose` and `## VPS` blocks are deleted |

`deploy.rollback` in the release file is the path of the fifth row.

## Token substitution

`@@APP@@` is the project directory name and is the `metadata.name` and the `app` label of every file. `@@VERSION@@` is `$1` and carries into the `version` label, the image tag, and the `newTag` of the overlay. `@@IMAGE@@` is `config.toml` `[release].image_name`. `@@PORT@@` is `[release].service_port` and appears unquoted at `containerPort`, at both probe ports, and at `targetPort`, and quoted as the `APP_PORT` environment value. `@@PREVIOUS@@` in the rollback note is the release file's `previous_version`.

## What the gate check reads

`deploy_manifest` for this platform is a parse and a pattern, run with no cluster reachable:

- every listed path exists, and every `.yaml` path parses as YAML;
- `deployment.yaml`, `service.yaml`, `ingress.yaml`, and `kustomization.yaml` each appear once in `deploy.manifests[]`;
- `deployment.yaml` carries `spec.template.spec.containers[0].image`, `.readinessProbe`, `.livenessProbe`, and `.resources.limits`;
- no line of a listed file matches the secret pattern `(?i)(password|secret|token|api[_-]?key|private[_-]?key)\s*[:=]\s*["']?[A-Za-z0-9+/=_-]{12,}` other than as a `${...}` reference, a `secretKeyRef`, a `valueFrom`, or a `$(...)` substitution. A literal match is `DFA-E343`.

The template reaches its secrets through `envFrom.secretRef`, which is the form the pattern exempts. A value read from `.devforgeai/` or from the environment goes in the same way; a value typed into the file fails the check.

## The architecture constraints

The `## Constraints` rows of `.devforgeai/context/architecture-constraints.md` reach `deploy-manifest-writer` as input. A row that binds the deployed shape — a replica floor, a resource ceiling, a required probe path, a forbidden host — is applied to the file it governs and its `CON-nnn` is listed in `constraints_applied`. A row that binds source code and not the manifest set is left out of that list.
