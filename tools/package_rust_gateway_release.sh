#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_PATH="${BIN_PATH:-${ROOT_DIR}/rust-gateway/target/release/notxboard-gateway}"
OUTPUT_DIR="${OUTPUT_DIR:-${ROOT_DIR}/dist}"
VERSION="${VERSION:-$(git -C "${ROOT_DIR}" describe --tags --always)}"
ARCH="${ARCH:-amd64}"
SAFE_VERSION="$(printf '%s' "${VERSION}" | tr -cs 'A-Za-z0-9._-' '-')"
PACKAGE_NAME="notxboard-${SAFE_VERSION}-linux-${ARCH}"
BUILD_DIR="$(mktemp -d "${TMPDIR:-/tmp}/notxboard-release.XXXXXX")"

cleanup() {
  rm -rf "${BUILD_DIR}"
}
trap cleanup EXIT

if [ ! -x "${BIN_PATH}" ]; then
  echo "release binary is missing or not executable: ${BIN_PATH}" >&2
  exit 1
fi

mkdir -p "${BUILD_DIR}/${PACKAGE_NAME}/runtime" "${BUILD_DIR}/${PACKAGE_NAME}/state" "${OUTPUT_DIR}"
cp "${BIN_PATH}" "${BUILD_DIR}/${PACKAGE_NAME}/notxboard-gateway"
cp "${ROOT_DIR}/tools/run-notxboard-gateway.sh" "${BUILD_DIR}/${PACKAGE_NAME}/run.sh"
cp "${ROOT_DIR}/.env.example" "${BUILD_DIR}/${PACKAGE_NAME}/.env.example"
cp "${ROOT_DIR}/LICENSE" "${BUILD_DIR}/${PACKAGE_NAME}/LICENSE"
cp -R "${ROOT_DIR}/rust-gateway/resources" "${BUILD_DIR}/${PACKAGE_NAME}/runtime/resources"
cp -R "${ROOT_DIR}/rust-gateway/resources/public" "${BUILD_DIR}/${PACKAGE_NAME}/runtime/public"
chmod 0755 "${BUILD_DIR}/${PACKAGE_NAME}/notxboard-gateway" "${BUILD_DIR}/${PACKAGE_NAME}/run.sh"

ARCHIVE_PATH="${OUTPUT_DIR}/${PACKAGE_NAME}.tar.gz"
tar -C "${BUILD_DIR}" -czf "${ARCHIVE_PATH}" "${PACKAGE_NAME}"

if command -v sha256sum >/dev/null 2>&1; then
  (cd "${OUTPUT_DIR}" && sha256sum "$(basename "${ARCHIVE_PATH}")") > "${ARCHIVE_PATH}.sha256"
else
  (cd "${OUTPUT_DIR}" && shasum -a 256 "$(basename "${ARCHIVE_PATH}")") > "${ARCHIVE_PATH}.sha256"
fi

echo "Created ${ARCHIVE_PATH}"
echo "Created ${ARCHIVE_PATH}.sha256"
