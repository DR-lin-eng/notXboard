#!/usr/bin/env sh
set -eu

ROOT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"

export RUST_RUNTIME_ROOT="${RUST_RUNTIME_ROOT:-${ROOT_DIR}/runtime}"
export RUST_STATE_ROOT="${RUST_STATE_ROOT:-${ROOT_DIR}/state}"

mkdir -p "${RUST_STATE_ROOT}/theme" "${RUST_STATE_ROOT}/plugins"
exec "${ROOT_DIR}/notxboard-gateway" "$@"
