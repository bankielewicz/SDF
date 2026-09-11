# Platform `compose`

Read this at workflow step 8 when `platform.target` is `compose`. A single host runs a multi-container definition, so the release writes the definition, the environment file that names every variable it reads, and one rollback note.

## Files written

Rooted at `config.toml` `[release].deploy_root`, `deploy` by default. Each row is one entry of `deploy.manifests[]` in the release file, in this order.

| Path | `kind` | Template | Holds |
|---|---|---|---|
| `<deploy_root>/compose/docker-compose.yaml` | `config` | `templates/compose/docker-compose.yaml` | one `services` entry at `image: @@IMAGE@@:@@VERSION@@`, a restart policy, the published port, the environment block, and a health check against the service port |
| `<deploy_root>/compose/env.example` | `config` | `templates/compose/env.example` | one `NAME=` line per `${NAME}` the definition reads, with the port filled and the two secret values left empty |
| `<deploy_root>/compose/ROLLBACK.md` | `script` | `templates/ROLLBACK.md` | the `## Compose` block and the `## After a rollback` block; the `## Kubernetes` and `## VPS` blocks are deleted |

`deploy.rollback` in the release file is the path of the third row.

## The variable pairing

The definition reads `${APP_PORT}`, `${APP_DATABASE_URL}`, and `${APP_SIGNING_KEY}`, and `env.example` carries a line for each. A service added to the definition brings its variables with it, and each one gains a `NAME=` line in the same write. The two sets are what the gate check compares, so an added variable with no line is the one way these two files drift.

`secrets_referenced` in the agent's output lists those variable names. The values stay out of both files and out of that list: `env.example` is the shape of the environment, and the host supplies the contents.

## Token substitution

`@@IMAGE@@` is `config.toml` `[release].image_name` and `@@VERSION@@` is `$1`, which together form the image reference. `@@PORT@@` is `[release].service_port` and appears as the container side of the port mapping, as the `APP_PORT` environment value, inside the health check URL, and as the value of the `APP_PORT=` line in `env.example`. `@@APP@@` and `@@PREVIOUS@@` appear in the rollback note alone.

## What the gate check reads

`deploy_manifest` for this platform is a parse and a set comparison, run with no host reachable:

- `docker-compose.yaml` exists and parses as YAML;
- it carries a `services` mapping holding at least one entry, and every entry carries `image` or `build`;
- `env.example` exists;
- every `${NAME}` reference in the definition has a `NAME=` line in `env.example`;
- no line of a listed file matches the secret pattern `(?i)(password|secret|token|api[_-]?key|private[_-]?key)\s*[:=]\s*["']?[A-Za-z0-9+/=_-]{12,}` other than as a `${...}` reference, a `secretKeyRef`, a `valueFrom`, or a `$(...)` substitution. A literal match is `DFA-E343`.

The `${NAME}` form is the one the pattern exempts, which is why every secret in the definition is a reference and its value lives on the host.

## The architecture constraints

The `## Constraints` rows of `.devforgeai/context/architecture-constraints.md` reach `deploy-manifest-writer` as input. A row binding the running shape — a restart policy, a health-check interval, a published port range, a forbidden image source — is applied to the definition and its `CON-nnn` is listed in `constraints_applied`. A row binding source code alone is left out of that list.
