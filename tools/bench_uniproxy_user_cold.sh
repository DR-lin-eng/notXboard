#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

MYSQL_CONTAINER="${MYSQL_CONTAINER:-notxboard-mysql-1}"
TEST_DB="${TEST_DB:-notxboard_rust_default_stack}"
HOST_PORT="${HOST_PORT:-18001}"
GATEWAY_IMAGE="${GATEWAY_IMAGE:-notxboard-gateway:prebuilt-local}"
DB_ROOT_USERNAME="${DB_ROOT_USERNAME:-root}"
DB_ROOT_PASSWORD="${DB_ROOT_PASSWORD:-change-me-root}"

BENCH_USER_COUNT="${BENCH_USER_COUNT:-2000}"
COLD_REQUESTS="${COLD_REQUESTS:-60}"
COLD_CONCURRENCY="${COLD_CONCURRENCY:-10}"
SKIP_VERIFY="${SKIP_VERIFY:-0}"

NODE_TOKEN="${NODE_TOKEN:-bench-uniproxy-token}"
NODE_V2BX_NODE_ID="${NODE_V2BX_NODE_ID:-9001}"
NODE_NAME="${NODE_NAME:-bench-uniproxy-node}"
NODE_HOST="${NODE_HOST:-bench.example.com}"

if ! command -v curl >/dev/null 2>&1; then
  echo "curl is required on the host" >&2
  exit 1
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 is required on the host" >&2
  exit 1
fi

if [ "${SKIP_VERIFY}" != "1" ]; then
  GATEWAY_IMAGE="${GATEWAY_IMAGE}" bash "${ROOT_DIR}/tools/verify_rust_default_stack.sh"
fi

echo "[1/4] seed benchmark users and node"
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

user_url="http://127.0.0.1:${HOST_PORT}/api/v1/server/UniProxy/user?token=${NODE_TOKEN}&node_id=${NODE_ID}"
results_file="$(mktemp)"
trap 'rm -f "${results_file}"' EXIT

echo "[2/4] run cold-path curl benchmark"
seq "${COLD_REQUESTS}" | xargs -I{} -P "${COLD_CONCURRENCY}" sh -c '
  curl -fsS -H "Cache-Control: no-cache" -o /dev/null -w "%{http_code} %{time_total}\n" "$0&nonce=$1"
' "${user_url}" {} > "${results_file}"

echo "[3/4] summarize"
python3 - <<'PY' "${results_file}"
import json
import math
import statistics
import sys
from pathlib import Path

rows = []
for line in Path(sys.argv[1]).read_text().splitlines():
    if not line.strip():
        continue
    code_s, time_s = line.split()
    rows.append((int(code_s), float(time_s)))

codes = [code for code, _ in rows]
times_ms = sorted(round(t * 1000, 3) for _, t in rows)
assert rows, "no benchmark rows captured"
assert all(code == 200 for code in codes), {"codes": codes}

def percentile(values, pct):
    if len(values) == 1:
        return values[0]
    index = math.ceil((pct / 100) * len(values)) - 1
    index = max(0, min(index, len(values) - 1))
    return values[index]

summary = {
    "requests": len(rows),
    "p50_ms": percentile(times_ms, 50),
    "p95_ms": percentile(times_ms, 95),
    "p99_ms": percentile(times_ms, 99),
    "mean_ms": round(statistics.mean(times_ms), 3),
    "max_ms": max(times_ms),
    "min_ms": min(times_ms),
}
print(json.dumps(summary, ensure_ascii=False))
PY

echo "[4/4] sample response size"
curl -fsS -H "Cache-Control: no-cache" "$user_url&nonce=sample" -o /tmp/notxboard-uniproxy-user-cold-sample.json
wc -c /tmp/notxboard-uniproxy-user-cold-sample.json
