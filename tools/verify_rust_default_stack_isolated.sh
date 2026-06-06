#!/usr/bin/env bash
set -euo pipefail

RUN_ID="${RUN_ID:-$(date +%Y%m%d%H%M%S)}"
PREFIX="${PREFIX:-notxboard-rust-stack-${RUN_ID}}"
NETWORK_NAME="${NETWORK_NAME:-${PREFIX}-net}"
MYSQL_CONTAINER="${MYSQL_CONTAINER:-${PREFIX}-mysql}"
REDIS_CONTAINER="${REDIS_CONTAINER:-${PREFIX}-redis}"
GATEWAY_CONTAINER="${GATEWAY_CONTAINER:-${PREFIX}-gateway}"
GATEWAY_IMAGE="${GATEWAY_IMAGE:-notxboard-gateway:rust-default-stack-${RUN_ID}}"
HOST_PORT="${HOST_PORT:-18001}"
BUILD_IMAGE="${BUILD_IMAGE:-1}"
CLEANUP="${CLEANUP:-1}"
CLEANUP_IMAGE="${CLEANUP_IMAGE:-0}"

DB_DATABASE="${DB_DATABASE:-xboard}"
DB_USERNAME="${DB_USERNAME:-notxboard}"
DB_PASSWORD="${DB_PASSWORD:-notxboard-pass}"
DB_ROOT_PASSWORD="${DB_ROOT_PASSWORD:-notxboard-root-pass}"
APP_KEY="${APP_KEY:-base64:h8KOzHFYUR2mToeLkkAAqw2/Oaibg+YEzOVW0gfAzNo=}"
APP_NAME="${APP_NAME:-notXboard}"
ADMIN_EMAIL="${ADMIN_EMAIL:-fulladmin@example.com}"
ADMIN_PASSWORD="${ADMIN_PASSWORD:-Passw0rd!2026}"
REDIS_PREFIX="${REDIS_PREFIX:-notxboard_database_}"
CACHE_PREFIX="${CACHE_PREFIX:-notxboard_cache}"
SCHEDULER_HEARTBEAT_KEY="${REDIS_PREFIX}${CACHE_PREFIX}SCHEDULE_LAST_CHECK_AT"

cleanup() {
  if [ "${CLEANUP}" = "1" ]; then
    docker rm -f "${GATEWAY_CONTAINER}" "${MYSQL_CONTAINER}" "${REDIS_CONTAINER}" >/dev/null 2>&1 || true
    docker network rm "${NETWORK_NAME}" >/dev/null 2>&1 || true
  fi
  if [ "${CLEANUP_IMAGE}" = "1" ]; then
    docker image rm "${GATEWAY_IMAGE}" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

wait_for_http() {
  local url="$1"
  local attempts="${2:-60}"
  for _ in $(seq 1 "${attempts}"); do
    if curl -fsS "${url}" >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done
  return 1
}

wait_for_mysql() {
  for _ in $(seq 1 90); do
    if docker exec "${MYSQL_CONTAINER}" mysqladmin ping -h 127.0.0.1 -uroot -p"${DB_ROOT_PASSWORD}" --silent >/dev/null 2>&1; then
      return 0
    fi
    sleep 1
  done
  return 1
}

json_field() {
  local field="$1"
  FIELD="${field}" python3 -c '
import json
import os
import sys

cur = json.load(sys.stdin)
for part in os.environ["FIELD"].split("."):
    cur = cur.get(part, {}) if isinstance(cur, dict) else {}
print(cur if isinstance(cur, str) else "")
'
}

json_array_contains() {
  local field="$1"
  local expected="$2"
  FIELD="${field}" EXPECTED="${expected}" python3 -c '
import json
import os
import sys

cur = json.load(sys.stdin)
for part in os.environ["FIELD"].split("."):
    cur = cur.get(part, {}) if isinstance(cur, dict) else {}
if not isinstance(cur, list) or os.environ["EXPECTED"] not in cur:
    sys.exit(1)
'
}

echo "[1/12] prepare isolated Docker network"
docker network create "${NETWORK_NAME}" >/dev/null

if [ "${BUILD_IMAGE}" = "1" ]; then
  echo "[2/12] build Rust gateway image: ${GATEWAY_IMAGE}"
  docker build -t "${GATEWAY_IMAGE}" rust-gateway
else
  echo "[2/12] reuse Rust gateway image: ${GATEWAY_IMAGE}"
fi

echo "[3/12] start isolated mysql and redis"
docker run -d --rm \
  --name "${MYSQL_CONTAINER}" \
  --network "${NETWORK_NAME}" \
  -e MYSQL_DATABASE="${DB_DATABASE}" \
  -e MYSQL_USER="${DB_USERNAME}" \
  -e MYSQL_PASSWORD="${DB_PASSWORD}" \
  -e MYSQL_ROOT_PASSWORD="${DB_ROOT_PASSWORD}" \
  mysql:8.0 \
  --character-set-server=utf8mb4 \
  --collation-server=utf8mb4_unicode_ci >/dev/null

docker run -d --rm \
  --name "${REDIS_CONTAINER}" \
  --network "${NETWORK_NAME}" \
  redis:7-alpine >/dev/null

wait_for_mysql

echo "[4/12] start Rust-only gateway"
docker run -d --rm \
  --name "${GATEWAY_CONTAINER}" \
  --network "${NETWORK_NAME}" \
  -p "${HOST_PORT}:8000" \
  -e PORT=8000 \
  -e DB_HOST="${MYSQL_CONTAINER}" \
  -e DB_PORT=3306 \
  -e DB_DATABASE="${DB_DATABASE}" \
  -e DB_USERNAME="${DB_USERNAME}" \
  -e DB_PASSWORD="${DB_PASSWORD}" \
  -e REDIS_HOST="${REDIS_CONTAINER}" \
  -e REDIS_PORT=6379 \
  -e REDIS_PREFIX="${REDIS_PREFIX}" \
  -e CACHE_PREFIX="${CACHE_PREFIX}" \
  -e APP_KEY="${APP_KEY}" \
  -e APP_URL="http://127.0.0.1:${HOST_PORT}" \
  -e APP_NAME="${APP_NAME}" \
  -e RUST_GATEWAY_OWNS_SCHEDULER=true \
  -e RUST_LOG=info \
  "${GATEWAY_IMAGE}" >/dev/null

wait_for_http "http://127.0.0.1:${HOST_PORT}/bootstrap/status" 90

echo "[5/12] verify Rust runtime excludes Laravel app/config/plugins/theme sources"
docker exec "${GATEWAY_CONTAINER}" sh -c 'test ! -e /app/runtime/app && test ! -e /app/runtime/config && test ! -e /app/runtime/plugins && test ! -e /app/runtime/theme && test ! -e /app/runtime/public/theme/Maintainable/dashboard.blade.php && test ! -e /app/runtime/public/theme/Maintainable/config.json && test -f /app/runtime/public/theme/portal/assets/umi.js'

echo "[6/12] bootstrap full schema through Rust"
curl -fsS -X POST "http://127.0.0.1:${HOST_PORT}/bootstrap/full" \
  -H 'Content-Type: application/json' \
  --data "{\"app_name\":\"${APP_NAME}\",\"app_url\":\"http://127.0.0.1:${HOST_PORT}\",\"admin_email\":\"${ADMIN_EMAIL}\",\"admin_password\":\"${ADMIN_PASSWORD}\"}"
echo

echo "[7/12] verify bootstrapped database state"
docker exec "${MYSQL_CONTAINER}" mysql -u"${DB_USERNAME}" -p"${DB_PASSWORD}" -D "${DB_DATABASE}" -e "
SELECT name, value
FROM v2_settings
WHERE name IN ('bootstrap_mode','app_name','app_url','secure_path','frontend_admin_path')
ORDER BY name;
SELECT COUNT(*) AS migrations_count FROM migrations;
SELECT COUNT(*) AS group_limit_count FROM user_group_limits;
"

SECURE_PATH="$(docker exec "${MYSQL_CONTAINER}" mysql -N -s -u"${DB_USERNAME}" -p"${DB_PASSWORD}" -D "${DB_DATABASE}" -e "SELECT value FROM v2_settings WHERE name='secure_path' ORDER BY id DESC LIMIT 1;" | tr -d '\r')"
if [ -z "${SECURE_PATH}" ]; then
  echo "secure_path missing after bootstrap" >&2
  exit 1
fi
echo "Resolved secure_path: ${SECURE_PATH}"

echo "[8/12] login through Rust passport API"
LOGIN_RESPONSE="$(curl -fsS -X POST "http://127.0.0.1:${HOST_PORT}/api/v1/passport/auth/login" \
  -H 'Content-Type: application/json' \
  --data "{\"email\":\"${ADMIN_EMAIL}\",\"password\":\"${ADMIN_PASSWORD}\"}")"
echo "${LOGIN_RESPONSE}"

AUTH_DATA="$(printf '%s' "${LOGIN_RESPONSE}" | json_field "data.auth_data")"
if [ -z "${AUTH_DATA}" ]; then
  echo "auth_data missing from login response" >&2
  exit 1
fi

echo "[9/12] verify Rust hook catalog without Laravel app/plugins runtime"
HOOKS_RESPONSE="$(curl -fsS "http://127.0.0.1:${HOST_PORT}/api/v2/${SECURE_PATH}/system/listHooks" \
  -H "Authorization: ${AUTH_DATA}")"
echo "${HOOKS_RESPONSE}"
printf '%s' "${HOOKS_RESPONSE}" | json_array_contains "data" "order.create.before"
printf '%s' "${HOOKS_RESPONSE}" | json_array_contains "data" "user.telegram.bind.after"

echo "[10/12] verify Rust built-in theme app without theme source runtime"
PUBLIC_PAGE="$(curl -fsS "http://127.0.0.1:${HOST_PORT}/")"
printf '%s' "${PUBLIC_PAGE}" | grep -q '公共概览'
APP_PAGE="$(curl -fsS "http://127.0.0.1:${HOST_PORT}/app")"
printf '%s' "${APP_PAGE}" | grep -q 'Maintainable'
printf '%s' "${APP_PAGE}" | grep -q '/theme/Maintainable/app.js'
if printf '%s' "${APP_PAGE}" | grep -q '{{ \$version'; then
  echo "unrendered version placeholder found in /app page" >&2
  exit 1
fi

echo "[11/12] trigger Rust-native manual database backup"
BACKUP_RESPONSE="$(curl -fsS -X POST "http://127.0.0.1:${HOST_PORT}/api/v2/${SECURE_PATH}/system/runDatabaseBackup" \
  -H "Authorization: ${AUTH_DATA}" \
  -H 'Content-Type: application/json' \
  --data '{"upload":false}')"
echo "${BACKUP_RESPONSE}"

ARTIFACT_PATH="$(printf '%s' "${BACKUP_RESPONSE}" | json_field "data.artifact_path")"
if [ -z "${ARTIFACT_PATH}" ]; then
  echo "artifact_path missing from backup response" >&2
  exit 1
fi
docker exec "${GATEWAY_CONTAINER}" test -f "${ARTIFACT_PATH}"
echo "Backup artifact exists: ${ARTIFACT_PATH}"

echo "[12/12] wait for Rust scheduler heartbeat"
sleep 70
SCHEDULER_HEARTBEAT="$(docker exec "${REDIS_CONTAINER}" redis-cli -n 1 GET "${SCHEDULER_HEARTBEAT_KEY}" | tr -d '\r')"
if [ -z "${SCHEDULER_HEARTBEAT}" ] || [ "${SCHEDULER_HEARTBEAT}" = "(nil)" ]; then
  echo "scheduler heartbeat key missing from Redis: ${SCHEDULER_HEARTBEAT_KEY}" >&2
  docker logs --tail 120 "${GATEWAY_CONTAINER}" >&2
  exit 1
fi
NOW_TS="$(date +%s)"
if ! [ "${SCHEDULER_HEARTBEAT}" -gt $((NOW_TS - 180)) ]; then
  echo "scheduler heartbeat key is stale: ${SCHEDULER_HEARTBEAT}" >&2
  docker logs --tail 120 "${GATEWAY_CONTAINER}" >&2
  exit 1
fi

echo "${SCHEDULER_HEARTBEAT_KEY}=${SCHEDULER_HEARTBEAT}"
echo "Recent Rust gateway logs:"
docker logs --tail 120 "${GATEWAY_CONTAINER}"
echo "Rust default stack verification passed."
