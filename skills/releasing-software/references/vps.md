# Platform `vps`

Read this at workflow step 8 when `platform.target` is `vps`. A host runs a script and a service unit, so the release writes the script, the unit, and one rollback note.

## Files written

Rooted at `config.toml` `[release].deploy_root`, `deploy` by default. Each row is one entry of `deploy.manifests[]` in the release file, in this order.

| Path | `kind` | Template | Holds |
|---|---|---|---|
| `<deploy_root>/vps/deploy.sh` | `script` | `templates/vps/deploy.sh` | the release directory under `${APP_ROOT}`, the archive unpack, the `current` symlink swap, the service restart, and a thirty-attempt health poll at two-second intervals that exits 1 when the service stays down |
| `<deploy_root>/vps/app.service` | `unit` | `templates/vps/app.service` | a simple service with the working directory at `current`, an environment file, `ExecStart=`, and restart on failure |
| `<deploy_root>/vps/ROLLBACK.md` | `script` | `templates/ROLLBACK.md` | the `## VPS` block and the `## After a rollback` block; the `## Kubernetes` and `## Compose` blocks are deleted |

`deploy.rollback` in the release file is the path of the third row. `deploy.sh` is written with LF line endings, because the host reads its first line as the interpreter line and a carriage return there leaves the script unrunnable.

## Token substitution

`@@APP@@` is the project directory name and appears as `APP_NAME` in the script, as the unit description, and in every `/srv/@@APP@@` path of both files. `@@VERSION@@` is `$1` and appears as `APP_VERSION`, which names the release directory the symlink points at. `@@PORT@@` is `config.toml` `[release].service_port` and is the port the health poll reads. `@@PREVIOUS@@` in the rollback note is the release file's `previous_version`, which names the directory the symlink swings back to.

The script takes the release archive path as its one positional argument, `${1:?release archive path}`, and that archive is what `[release].package_command` produces. The script names no packaging step of its own.

## What the gate check reads

`deploy_manifest` for this platform is a parse and a pattern, run with no host reachable:

- `vps/deploy.sh` exists, is non-empty, and its first line is `#!/bin/sh` or `#!/usr/bin/env sh`;
- `vps/app.service` exists and carries `ExecStart=`;
- no line of a listed file matches the secret pattern `(?i)(password|secret|token|api[_-]?key|private[_-]?key)\s*[:=]\s*["']?[A-Za-z0-9+/=_-]{12,}` other than as a `${...}` reference, a `secretKeyRef`, a `valueFrom`, or a `$(...)` substitution. A literal match is `DFA-E343`.

The unit reaches its configuration through `EnvironmentFile=`, so the file on the host holds the values and neither shipped file does. `secrets_referenced` in the agent's output lists the variable names those two files read.

## The architecture constraints

The `## Constraints` rows of `.devforgeai/context/architecture-constraints.md` reach `deploy-manifest-writer` as input. A row binding the host layout — the release root, the restart policy, the health path, the poll budget — is applied to the script or the unit and its `CON-nnn` is listed in `constraints_applied`. A row binding source code alone is left out of that list.
