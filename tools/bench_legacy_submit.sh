#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

MYSQL_CONTAINER="${MYSQL_CONTAINER:-notxboard-mysql-1}"
TEST_DB="${TEST_DB:-notxboard_rust_default_stack}"
HOST_PORT="${HOST_PORT:-18001}"
DB_ROOT_USERNAME="${DB_ROOT_USERNAME:-root}"
DB_ROOT_PASSWORD="${DB_ROOT_PASSWORD:-change-me-root}"

LEGACY_PROTOCOL="${LEGACY_PROTOCOL:-shadowsocks}"
LEGACY_ROUTE_NAME="${LEGACY_ROUTE_NAME:-ShadowsocksTidalab}"
LEGACY_NODE_CODE="${LEGACY_NODE_CODE:-legacy-bench}"
LEGACY_SERVER_TOKEN="${LEGACY_SERVER_TOKEN:-legacy-token-bench}"
LEGACY_SERVER_RATE="${LEGACY_SERVER_RATE:-1.00}"
LEGACY_GROUP_ID="${LEGACY_GROUP_ID:-1}"

BENCH_USER_COUNT="${BENCH_USER_COUNT:-2000}"
LEGACY_ACTIVE_USERS="${LEGACY_ACTIVE_USERS:-500}"
AB_REQUESTS="${AB_REQUESTS:-120}"
AB_CONCURRENCY="${AB_CONCURRENCY:-20}"
AB_KEEPALIVE="${AB_KEEPALIVE:-1}"
SKIP_VERIFY="${SKIP_VERIFY:-0}"

if ! command -v ab >/dev/null 2>&1; then
  echo "apachebench 'ab' is required on the host" >&2
  exit 1
fi

if [ "${SKIP_VERIFY}" != "1" ]; then
  bash "${ROOT_DIR}/tools/verify_rust_default_stack.sh"
fi

echo "[1/5] seed legacy bench users and server"
docker exec "${MYSQL_CONTAINER}" mysql -u"${DB_ROOT_USERNAME}" -p"${DB_ROOT_PASSWORD}" -D "${TEST_DB}" -e "
SET SESSION cte_max_recursion_depth = 100000;
DELETE FROM v2_server WHERE type = '${LEGACY_PROTOCOL}' AND code = '${LEGACY_NODE_CODE}';
DELETE FROM v2_settings WHERE name = 'server_token';
DELETE FROM v2_stat_user;
DELETE FROM v2_stat_server;
DELETE FROM v2_user WHERE email LIKE 'legacy-bench-user-%@example.com';
INSERT INTO v2_settings (name, value, created_at, updated_at)
VALUES ('server_token', '${LEGACY_SERVER_TOKEN}', NOW(), NOW());
INSERT INTO v2_user (
  email, password, balance, commission_type, commission_balance, t, u, d, transfer_enable,
  banned, is_admin, is_super_admin, is_staff, uuid, trust_level, is_silenced, token,
  subscription_credential_version, expired_at, group_id, created_at, updated_at
)
WITH RECURSIVE seq AS (
  SELECT 1 AS n
  UNION ALL
  SELECT n + 1 FROM seq WHERE n < ${BENCH_USER_COUNT}
)
SELECT
  CONCAT('legacy-bench-user-', n, '@example.com'),
  RPAD('bench-password', 64, 'x'),
  0, 0, 0, 0, 0, 0, 1099511627776,
  0, 0, 0, 0,
  LOWER(CONCAT(
    SUBSTR(MD5(CONCAT('legacy-bench-uuid-', n)), 1, 8), '-',
    SUBSTR(MD5(CONCAT('legacy-bench-uuid-', n)), 9, 4), '-',
    SUBSTR(MD5(CONCAT('legacy-bench-uuid-', n)), 13, 4), '-',
    SUBSTR(MD5(CONCAT('legacy-bench-uuid-', n)), 17, 4), '-',
    SUBSTR(MD5(CONCAT('legacy-bench-uuid-', n)), 21, 12)
  )),
  0, 0,
  LPAD(LOWER(HEX(n + 131072)), 32, '0'),
  0,
  UNIX_TIMESTAMP() + 31536000,
  ${LEGACY_GROUP_ID},
  UNIX_TIMESTAMP(),
  UNIX_TIMESTAMP()
FROM seq;
INSERT INTO v2_server (
  type, code, parent_id, group_ids, route_ids, name, rate,
  rate_time_enable, rate_time_ranges, tags, host, port, server_port,
  protocol_settings, sort, created_at, updated_at
) VALUES (
  '${LEGACY_PROTOCOL}',
  '${LEGACY_NODE_CODE}',
  NULL,
  JSON_ARRAY(${LEGACY_GROUP_ID}),
  JSON_ARRAY(),
  'Legacy Bench',
  ${LEGACY_SERVER_RATE},
  0,
  JSON_ARRAY(),
  JSON_ARRAY(),
  'legacy.example.com',
  '443',
  443,
  JSON_OBJECT('cipher', 'aes-128-gcm'),
  1,
  NOW(),
  NOW()
);
" >/dev/null

payload_file="$(mktemp)"
trap 'rm -f "${payload_file}"' EXIT

echo "[2/5] build legacy submit payload"
{
  printf '{'
  i=1
  while [ "${i}" -le "${LEGACY_ACTIVE_USERS}" ]; do
    user_id=$((i + 1))
    if [ "${i}" -gt 1 ]; then
      printf ','
    fi
    printf '"%s":{"u":1024,"d":2048}' "${user_id}"
    i=$((i + 1))
  done
  printf '}'
} > "${payload_file}"

submit_url="http://127.0.0.1:${HOST_PORT}/api/v1/server/${LEGACY_ROUTE_NAME}/submit?node_id=${LEGACY_NODE_CODE}&token=${LEGACY_SERVER_TOKEN}"

echo "[3/5] warm endpoint"
curl -fsS -X POST "${submit_url}" -H 'Content-Type: application/json' --data-binary @"${payload_file}" >/dev/null

ab_keepalive_args=()
if [ "${AB_KEEPALIVE}" = "1" ]; then
  ab_keepalive_args+=("-k")
fi

echo "[4/5] run ApacheBench"
output_file="$(mktemp)"
trap 'rm -f "${payload_file}" "${output_file}"' EXIT
ab -q "${ab_keepalive_args[@]}" -n "${AB_REQUESTS}" -c "${AB_CONCURRENCY}" -p "${payload_file}" -T "application/json" "${submit_url}" | tee "${output_file}"
echo
echo "--- Legacy submit summary ---"
grep -E "Complete requests|Failed requests|Non-2xx responses|Requests per second|Time per request" "${output_file}" || true
awk '
  /Percentage of the requests served within a certain time/ {capture=1; next}
  capture && $1 ~ /^(50|66|75|80|90|95|98|99|100)%$/ {print}
' "${output_file}" || true

echo
echo "[5/5] verify stat tables"
docker exec "${MYSQL_CONTAINER}" mysql -N -s -u"${DB_ROOT_USERNAME}" -p"${DB_ROOT_PASSWORD}" -D "${TEST_DB}" -e "
SELECT COUNT(*) AS stat_user_rows FROM v2_stat_user;
SELECT COUNT(*) AS stat_server_rows FROM v2_stat_server;
SELECT COALESCE(SUM(u), 0) AS total_u, COALESCE(SUM(d), 0) AS total_d FROM v2_stat_user;
SELECT COALESCE(SUM(u), 0) AS total_u, COALESCE(SUM(d), 0) AS total_d FROM v2_stat_server;
" || true
