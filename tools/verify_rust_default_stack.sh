#!/usr/bin/env bash
set -euo pipefail

MYSQL_CONTAINER="${MYSQL_CONTAINER:-notxboard-mysql-1}"
NETWORK_NAME="${NETWORK_NAME:-notxboard_notxboard}"
GATEWAY_IMAGE="${GATEWAY_IMAGE:-notxboard-gateway:prebuilt-local}"
TEST_DB="${TEST_DB:-notxboard_rust_default_stack}"
TEST_CONTAINER="${TEST_CONTAINER:-notxboard-rust-full-test}"
HOST_PORT="${HOST_PORT:-18001}"

DB_PORT="${DB_PORT:-3306}"
DB_USERNAME="${DB_USERNAME:-notxboard}"
DB_PASSWORD="${DB_PASSWORD:-t26fFEJCWD33iaTk}"
DB_ROOT_USERNAME="${DB_ROOT_USERNAME:-root}"
DB_ROOT_PASSWORD="${DB_ROOT_PASSWORD:-change-me-root}"
REDIS_CONTAINER="${REDIS_CONTAINER:-notxboard-redis-1}"
APP_KEY="${APP_KEY:-base64:h8KOzHFYUR2mToeLkkAAqw2/Oaibg+YEzOVW0gfAzNo=}"
BOOTSTRAP_TOKEN="${BOOTSTRAP_TOKEN:-notxboard-test-bootstrap-token-2026}"
APP_NAME="${APP_NAME:-notXboard}"
ADMIN_EMAIL="${ADMIN_EMAIL:-fulladmin@example.com}"
ADMIN_PASSWORD="${ADMIN_PASSWORD:-Passw0rd!2026}"
REDIS_PREFIX="${REDIS_PREFIX:-notxboard_database_}"
CACHE_PREFIX="${CACHE_PREFIX:-notxboard_cache}"
SCHEDULER_HEARTBEAT_KEY="${REDIS_PREFIX}${CACHE_PREFIX}SCHEDULE_LAST_CHECK_AT"

echo "[1/6] recreate test database: ${TEST_DB}"
docker exec "${MYSQL_CONTAINER}" mysql -u"${DB_ROOT_USERNAME}" -p"${DB_ROOT_PASSWORD}" \
  -e "DROP DATABASE IF EXISTS \`${TEST_DB}\`;
      CREATE DATABASE \`${TEST_DB}\` CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
      GRANT ALL PRIVILEGES ON \`${TEST_DB}\`.* TO '${DB_USERNAME}'@'%';
      FLUSH PRIVILEGES;"

echo "[2/6] remove old test container if present"
docker rm -f "${TEST_CONTAINER}" >/dev/null 2>&1 || true
docker exec "${REDIS_CONTAINER}" redis-cli -n 1 DEL "${SCHEDULER_HEARTBEAT_KEY}" >/dev/null

echo "[3/6] start Rust-only test container"
echo "gateway image: ${GATEWAY_IMAGE}"
docker run --name "${TEST_CONTAINER}" \
  --network "${NETWORK_NAME}" \
  -p "${HOST_PORT}:8000" \
  -e PORT=8000 \
  -e DB_HOST=mysql \
  -e DB_PORT="${DB_PORT}" \
  -e DB_DATABASE="${TEST_DB}" \
  -e DB_USERNAME="${DB_USERNAME}" \
  -e DB_PASSWORD="${DB_PASSWORD}" \
  -e REDIS_HOST=redis \
  -e REDIS_PORT=6379 \
  -e APP_KEY="${APP_KEY}" \
  -e BOOTSTRAP_TOKEN="${BOOTSTRAP_TOKEN}" \
  -e APP_URL="http://127.0.0.1:${HOST_PORT}" \
  -e APP_NAME="${APP_NAME}" \
  -e RUST_RUNTIME_ROOT=/app/runtime \
  -e RUST_STATE_ROOT=/app/state \
  -d "${GATEWAY_IMAGE}" >/dev/null

echo "container image: $(docker inspect -f '{{.Config.Image}}' "${TEST_CONTAINER}")"

echo "[4/6] wait for bootstrap endpoint"
for _ in $(seq 1 30); do
  if curl -fsS "http://127.0.0.1:${HOST_PORT}/bootstrap/status" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

echo "[5/6] run /bootstrap/full"
curl -fsS -X POST "http://127.0.0.1:${HOST_PORT}/bootstrap/full" \
  -H 'Content-Type: application/json' \
  -H "X-Bootstrap-Token: ${BOOTSTRAP_TOKEN}" \
  --data "{\"app_name\":\"${APP_NAME}\",\"app_url\":\"http://127.0.0.1:${HOST_PORT}\",\"admin_email\":\"${ADMIN_EMAIL}\",\"admin_password\":\"${ADMIN_PASSWORD}\"}"
echo

echo "[6/6] verify database state"
docker exec "${MYSQL_CONTAINER}" mysql -u"${DB_USERNAME}" -p"${DB_PASSWORD}" -D "${TEST_DB}" -e "
SELECT name, value
FROM v2_settings
WHERE name IN ('oauth_linux_do_enable','bootstrap_mode','app_name','app_url','secure_path','frontend_admin_path')
ORDER BY name;
SELECT COUNT(*) AS migrations_count FROM migrations;
SELECT COUNT(*) AS group_limit_count FROM user_group_limits;
"

SECURE_PATH="$(docker exec "${MYSQL_CONTAINER}" mysql -N -s -u"${DB_USERNAME}" -p"${DB_PASSWORD}" -D "${TEST_DB}" -e "SELECT value FROM v2_settings WHERE name='secure_path' ORDER BY id DESC LIMIT 1;")"
echo
echo "Resolved secure_path: ${SECURE_PATH}"

echo "Login as super-admin through Rust API"
LOGIN_RESPONSE="$(curl -fsS -X POST "http://127.0.0.1:${HOST_PORT}/api/v1/passport/auth/login" \
  -H 'Content-Type: application/json' \
  --data "{\"email\":\"${ADMIN_EMAIL}\",\"password\":\"${ADMIN_PASSWORD}\"}")"
echo "${LOGIN_RESPONSE}"

AUTH_DATA="$(printf '%s' "${LOGIN_RESPONSE}" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("data", {}).get("auth_data", ""))')"
if [ -z "${AUTH_DATA}" ]; then
  echo "auth_data missing from login response" >&2
  exit 1
fi

echo
echo "Trigger Rust-native manual backup endpoint"
BACKUP_RESPONSE="$(curl -fsS -X POST "http://127.0.0.1:${HOST_PORT}/api/v2/${SECURE_PATH}/system/runDatabaseBackup" \
  -H "Authorization: ${AUTH_DATA}" \
  -H 'Content-Type: application/json' \
  --data '{"upload":false}')"
echo "${BACKUP_RESPONSE}"

ARTIFACT_PATH="$(printf '%s' "${BACKUP_RESPONSE}" | python3 -c 'import json,sys; print((json.load(sys.stdin).get("data") or {}).get("artifact_path") or "")')"
if [ -z "${ARTIFACT_PATH}" ]; then
  echo "artifact_path missing from backup response" >&2
  exit 1
fi

echo
echo "Verify backup artifact exists inside container: ${ARTIFACT_PATH}"
docker exec "${TEST_CONTAINER}" test -f "${ARTIFACT_PATH}"

echo
echo "Waiting 65 seconds for the next scheduler tick..."
sleep 65

echo
echo "Verify scheduler heartbeat key was written to Redis"
SCHEDULER_HEARTBEAT="$(docker exec "${REDIS_CONTAINER}" redis-cli -n 1 GET "${SCHEDULER_HEARTBEAT_KEY}" | tr -d '\r')"
if [ -z "${SCHEDULER_HEARTBEAT}" ] || [ "${SCHEDULER_HEARTBEAT}" = "(nil)" ]; then
  echo "scheduler heartbeat key missing from Redis" >&2
  exit 1
fi
NOW_TS="$(date +%s)"
if ! [ "${SCHEDULER_HEARTBEAT}" -gt $((NOW_TS - 180)) ]; then
  echo "scheduler heartbeat key is stale: ${SCHEDULER_HEARTBEAT}" >&2
  exit 1
fi
echo "${SCHEDULER_HEARTBEAT_KEY}=${SCHEDULER_HEARTBEAT}"

echo "Recent container logs:"
docker logs --tail 120 "${TEST_CONTAINER}"
