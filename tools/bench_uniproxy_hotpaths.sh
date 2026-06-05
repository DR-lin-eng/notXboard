#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

MYSQL_CONTAINER="${MYSQL_CONTAINER:-notxboard-mysql-1}"
TEST_DB="${TEST_DB:-notxboard_rust_default_stack}"
HOST_PORT="${HOST_PORT:-18001}"
GATEWAY_IMAGE="${GATEWAY_IMAGE:-notxboard-gateway:prebuilt-local}"
DB_ROOT_USERNAME="${DB_ROOT_USERNAME:-root}"
DB_ROOT_PASSWORD="${DB_ROOT_PASSWORD:-change-me-root}"

BENCH_USER_COUNT="${BENCH_USER_COUNT:-5000}"
BENCH_ACTIVE_USERS="${BENCH_ACTIVE_USERS:-1000}"
AB_REQUESTS="${AB_REQUESTS:-300}"
AB_CONCURRENCY="${AB_CONCURRENCY:-30}"
AB_KEEPALIVE="${AB_KEEPALIVE:-1}"
SKIP_VERIFY="${SKIP_VERIFY:-0}"
BENCH_SETTLE_SECS="${BENCH_SETTLE_SECS:-1}"

NODE_TOKEN="${NODE_TOKEN:-bench-uniproxy-token}"
NODE_V2BX_NODE_ID="${NODE_V2BX_NODE_ID:-9001}"
NODE_NAME="${NODE_NAME:-bench-uniproxy-node}"
NODE_HOST="${NODE_HOST:-bench.example.com}"

if ! command -v ab >/dev/null 2>&1; then
  echo "apachebench 'ab' is required on the host" >&2
  exit 1
fi

if [ "${SKIP_VERIFY}" != "1" ]; then
  GATEWAY_IMAGE="${GATEWAY_IMAGE}" bash "${ROOT_DIR}/tools/verify_rust_default_stack.sh"
fi

echo "[1/5] seed benchmark users and node"
docker exec "${MYSQL_CONTAINER}" mysql -u"${DB_ROOT_USERNAME}" -p"${DB_ROOT_PASSWORD}" -D "${TEST_DB}" -e "
SET SESSION cte_max_recursion_depth = 100000;
DELETE FROM user_online_sessions WHERE node_id IN (SELECT id FROM server_nodes WHERE v2bx_token = '${NODE_TOKEN}');
DELETE FROM node_traffic_records WHERE node_id IN (SELECT id FROM server_nodes WHERE v2bx_token = '${NODE_TOKEN}');
DELETE FROM user_traffic_usage_logs WHERE node_id IN (SELECT id FROM server_nodes WHERE v2bx_token = '${NODE_TOKEN}');
DELETE FROM user_node_access WHERE node_id IN (SELECT id FROM server_nodes WHERE v2bx_token = '${NODE_TOKEN}');
DELETE FROM server_nodes WHERE v2bx_token = '${NODE_TOKEN}';
DELETE FROM v2_user WHERE email LIKE 'bench-user-%@example.com';
INSERT INTO server_nodes (
  user_id, name, host, port, service_port, protocol, location_code, location_name, settings,
  traffic_limit, traffic_used, traffic_multiplier, tcping_enabled, access_control, status,
  v2bx_node_id, v2bx_config, v2bx_token, device_limit, connection_limit,
  speed_limit_up, speed_limit_down, cross_node_ip_limit, concurrent_ip_limit, created_at, updated_at
) VALUES (
  1, '${NODE_NAME}', '${NODE_HOST}', 443, NULL, 'vmess', 'HK', 'Bench Node', JSON_OBJECT(),
  0, 0, 1.00, 0, JSON_OBJECT('min_trust_level', 0), 'active',
  ${NODE_V2BX_NODE_ID}, JSON_OBJECT(), '${NODE_TOKEN}', 0, 0,
  0, 0, 0, 0, NOW(), NOW()
);
INSERT INTO v2_user (
  email, password, balance, commission_type, commission_balance, t, u, d, transfer_enable,
  banned, is_admin, is_super_admin, is_staff, uuid, trust_level, is_silenced, token,
  subscription_credential_version, expired_at, created_at, updated_at
)
WITH RECURSIVE seq AS (
  SELECT 1 AS n
  UNION ALL
  SELECT n + 1 FROM seq WHERE n < ${BENCH_USER_COUNT}
)
SELECT
  CONCAT('bench-user-', n, '@example.com'),
  RPAD('bench-password', 64, 'x'),
  0, 0, 0, 0, 0, 0, 1099511627776,
  0, 0, 0, 0,
  LOWER(CONCAT(
    SUBSTR(MD5(CONCAT('bench-uuid-', n)), 1, 8), '-',
    SUBSTR(MD5(CONCAT('bench-uuid-', n)), 9, 4), '-',
    SUBSTR(MD5(CONCAT('bench-uuid-', n)), 13, 4), '-',
    SUBSTR(MD5(CONCAT('bench-uuid-', n)), 17, 4), '-',
    SUBSTR(MD5(CONCAT('bench-uuid-', n)), 21, 12)
  )),
  0, 0,
  LPAD(LOWER(HEX(n + 65536)), 32, '0'),
  0,
  UNIX_TIMESTAMP() + 31536000,
  UNIX_TIMESTAMP(), UNIX_TIMESTAMP()
FROM seq;
" >/dev/null

NODE_ID="$(docker exec "${MYSQL_CONTAINER}" mysql -N -s -u"${DB_ROOT_USERNAME}" -p"${DB_ROOT_PASSWORD}" -D "${TEST_DB}" -e "SELECT id FROM server_nodes WHERE v2bx_token='${NODE_TOKEN}' ORDER BY id DESC LIMIT 1;")"
USER_COUNT="$(docker exec "${MYSQL_CONTAINER}" mysql -N -s -u"${DB_ROOT_USERNAME}" -p"${DB_ROOT_PASSWORD}" -D "${TEST_DB}" -e "SELECT COUNT(*) FROM v2_user WHERE email LIKE 'bench-user-%@example.com';")"
echo "bench node id: ${NODE_ID}"
echo "bench users: ${USER_COUNT}"

alive_payload="$(mktemp)"
push_payload="$(mktemp)"
trap 'rm -f "${alive_payload}" "${push_payload}"' EXIT

echo "[2/5] generate request payloads"
{
  printf '{'
  i=2
  end_user=$((BENCH_ACTIVE_USERS + 1))
  while [ "${i}" -le "${end_user}" ]; do
    if [ "${i}" -gt 2 ]; then
      printf ','
    fi
    printf '"%s":["10.%d.%d.%d"]' "${i}" $(((i / 60000) % 250 + 1)) $(((i / 250) % 250 + 1)) $((i % 250 + 1))
    i=$((i + 1))
  done
  printf '}'
} > "${alive_payload}"

{
  printf '{'
  i=2
  end_user=$((BENCH_ACTIVE_USERS + 1))
  while [ "${i}" -le "${end_user}" ]; do
    if [ "${i}" -gt 2 ]; then
      printf ','
    fi
    printf '"%s":[1024,2048]' "${i}"
    i=$((i + 1))
  done
  printf '}'
} > "${push_payload}"

base_url="http://127.0.0.1:${HOST_PORT}"
user_url="${base_url}/api/v1/server/UniProxy/user?token=${NODE_TOKEN}&node_id=${NODE_ID}"
alive_url="${base_url}/api/v1/server/UniProxy/alive?token=${NODE_TOKEN}&node_id=${NODE_ID}"
push_url="${base_url}/api/v1/server/UniProxy/push?token=${NODE_TOKEN}&node_id=${NODE_ID}"

echo "[3/5] warm endpoints"
curl -fsS "${user_url}" >/dev/null
curl -fsS -X POST "${alive_url}" -H 'Content-Type: application/json' --data-binary @"${alive_payload}" >/dev/null
curl -fsS -X POST "${push_url}" -H 'Content-Type: application/json' --data-binary @"${push_payload}" >/dev/null

ab_keepalive_args=()
if [ "${AB_KEEPALIVE}" = "1" ]; then
  ab_keepalive_args+=("-k")
fi

run_ab() {
  local name="$1"
  shift
  local output
  output="$(mktemp)"
  ab -q "${ab_keepalive_args[@]}" -n "${AB_REQUESTS}" -c "${AB_CONCURRENCY}" "$@" | tee "${output}"
  echo
  echo "--- ${name} summary ---"
  grep -E "Complete requests|Failed requests|Requests per second|Time per request" "${output}" || true
  awk '
    /Percentage of the requests served within a certain time/ {capture=1; next}
    capture && $1 ~ /^(50|66|75|80|90|95|98|99|100)%$/ {print}
  ' "${output}" || true
  echo
  rm -f "${output}"
}

echo "[4/5] run ApacheBench"
run_ab "UniProxy user" "${user_url}"
run_ab "UniProxy alive" -p "${alive_payload}" -T "application/json" "${alive_url}"
run_ab "UniProxy push" -p "${push_payload}" -T "application/json" "${push_url}"

echo "[5/5] current online session rows"
sleep "${BENCH_SETTLE_SECS}"
docker exec "${MYSQL_CONTAINER}" mysql -N -s -u"${DB_ROOT_USERNAME}" -p"${DB_ROOT_PASSWORD}" -D "${TEST_DB}" -e "
SELECT COUNT(*) FROM user_online_sessions WHERE node_id = ${NODE_ID};
" || true
