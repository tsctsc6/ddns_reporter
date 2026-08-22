#!/usr/bin/env bash

set -euo pipefail

SERVICE_USER="ddns_reporter"
SERVICE_GROUP="ddns_reporter"
APP_DIR="/opt/ddns_reporter"
EXE_DIR="/usr/local/bin"
BINARY_SRC="./ddns_reporter"
BINARY_NAME="ddns_reporter"

if [[ $EUID -ne 0 ]]; then
   echo "root permissions required" >&2
   exit 1
fi

if [[ ! -f "$BINARY_SRC" ]]; then
    echo "$BINARY_SRC not found" >&2
    exit 1
fi

echo "[1/4] Creating user..."
if ! getent group "$SERVICE_GROUP" >/dev/null 2>&1; then
    groupadd --system "$SERVICE_GROUP"
fi

if ! id -u "$SERVICE_USER" >/dev/null 2>&1; then
    useradd \
        --system \
        --gid "$SERVICE_GROUP" \
        --home-dir "$APP_DIR" \
        --no-create-home \
        --shell /usr/sbin/nologin \
        --comment "ddns_reporter Daemon User" \
        "$SERVICE_USER"
fi

echo "[2/4] Initialize directory..."
mkdir -p "${APP_DIR}"

echo "[3/4] Move executable..."
install -m 755 "$BINARY_SRC" "${EXE_DIR}/${BINARY_NAME}"

echo "[4/4] Set directory permission"
chown -R "${SERVICE_USER}:${SERVICE_GROUP}" "$APP_DIR"
chmod 600 "$APP_DIR"

echo "Deploy done! ${APP_DIR}/bin/${BINARY_NAME}"
