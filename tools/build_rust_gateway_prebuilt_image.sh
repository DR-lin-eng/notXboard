#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
IMAGE_TAG="${IMAGE_TAG:-notxboard-gateway:prebuilt-local}"
BIN_PATH="${BIN_PATH:-${ROOT_DIR}/rust-gateway/target/release/notxboard-gateway}"

if [ ! -x "${BIN_PATH}" ]; then
  echo "prebuilt binary missing or not executable: ${BIN_PATH}" >&2
  exit 1
fi

BUILD_DIR="$(mktemp -d /private/tmp/notxboard-gateway-prebuilt.XXXXXX)"
cleanup() {
  rm -rf "${BUILD_DIR}"
}
trap cleanup EXIT

cp "${BIN_PATH}" "${BUILD_DIR}/notxboard-gateway"
cp -R "${ROOT_DIR}/resources" "${BUILD_DIR}/resources"
cp -R "${ROOT_DIR}/public" "${BUILD_DIR}/public"
mkdir -p "${BUILD_DIR}/theme/portal"
cp -R "${ROOT_DIR}/theme/portal/assets" "${BUILD_DIR}/theme/portal/assets"

cat > "${BUILD_DIR}/Dockerfile" <<'EOF'
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates default-mysql-client tzdata wget \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY notxboard-gateway /usr/local/bin/notxboard-gateway
COPY resources /app/runtime/resources
COPY public /app/runtime/public
COPY theme/portal/assets /app/runtime/public/theme/portal/assets
RUN rm -f /app/runtime/public/theme/Maintainable/dashboard.blade.php /app/runtime/public/theme/Maintainable/config.json \
    && mkdir -p /app/state/theme /app/state/plugins
ENV RUST_LOG=info
ENV RUST_RUNTIME_ROOT=/app/runtime
ENV RUST_STATE_ROOT=/app/state
EXPOSE 8000
ENTRYPOINT ["/usr/local/bin/notxboard-gateway"]
EOF

DOCKER_BUILDKIT=0 docker build -t "${IMAGE_TAG}" "${BUILD_DIR}"
