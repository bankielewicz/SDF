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
