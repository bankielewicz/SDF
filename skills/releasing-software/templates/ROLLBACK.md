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
