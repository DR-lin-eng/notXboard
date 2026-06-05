#!/usr/bin/env bash
set -euo pipefail

MYSQL_CONTAINER="${MYSQL_CONTAINER:-notxboard-mysql-1}"
TEST_DB="${TEST_DB:-notxboard_rust_default_stack}"
HOST_PORT="${HOST_PORT:-18001}"
GATEWAY_IMAGE="${GATEWAY_IMAGE:-notxboard-gateway:prebuilt-local}"
DB_ROOT_USERNAME="${DB_ROOT_USERNAME:-root}"
DB_ROOT_PASSWORD="${DB_ROOT_PASSWORD:-change-me-root}"

BENCH_USER_COUNT="${BENCH_USER_COUNT:-2000}"
REQUESTS="${REQUESTS:-40}"
CONCURRENCY="${CONCURRENCY:-8}"
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
  GATEWAY_IMAGE="${GATEWAY_IMAGE}" bash tools/verify_rust_default_stack.sh
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
echo "bench node id: ${NODE_ID}"

BASE_URL="http://127.0.0.1:${HOST_PORT}/api/v1/server/UniProxy/user?token=${NODE_TOKEN}&node_id=${NODE_ID}"

run_case() {
  local label="$1"
  local response_format="$2"
  local cache_mode="$3"
  local body_out
  local results_out
  body_out="$(mktemp)"
  results_out="$(mktemp)"

  local curl_headers=()
  if [ "$response_format" = "msgpack" ]; then
    curl_headers+=(-H "X-Response-Format: msgpack")
  fi
  if [ "$cache_mode" = "cold" ]; then
    curl_headers+=(-H "Cache-Control: no-cache")
  fi

  echo "[2/4] ${label}: sample response"
  if [ "${#curl_headers[@]}" -gt 0 ]; then
    curl -fsS "${curl_headers[@]}" "$BASE_URL&nonce=${label}-sample" -o "${body_out}"
  else
    curl -fsS "$BASE_URL&nonce=${label}-sample" -o "${body_out}"
  fi
  local body_size
  body_size="$(wc -c < "${body_out}" | tr -d ' ')"

  echo "[3/4] ${label}: latency run"
  export BENCH_BASE_URL="${BASE_URL}"
  export BENCH_HEADERS_JSON="$(python3 - <<'PY' "${response_format}" "${cache_mode}"
import json, sys
response_format, cache_mode = sys.argv[1], sys.argv[2]
headers = {}
if response_format == "msgpack":
    headers["X-Response-Format"] = "msgpack"
if cache_mode == "cold":
    headers["Cache-Control"] = "no-cache"
print(json.dumps(headers))
PY
)"
  python3 - <<'PY' "${BENCH_BASE_URL}" "${BENCH_HEADERS_JSON}" "${REQUESTS}" "${CONCURRENCY}" "${results_out}"
import concurrent.futures
import json
import sys
import time
import urllib.request

base_url, headers_json, request_count, concurrency, output_path = sys.argv[1:]
headers = json.loads(headers_json)
request_count = int(request_count)
concurrency = int(concurrency)

def do_request(i: int):
    req = urllib.request.Request(f"{base_url}&nonce={i}")
    for k, v in headers.items():
        req.add_header(k, v)
    start = time.perf_counter()
    with urllib.request.urlopen(req, timeout=30) as resp:
        resp.read()
        return resp.status, time.perf_counter() - start

with concurrent.futures.ThreadPoolExecutor(max_workers=concurrency) as pool:
    results = list(pool.map(do_request, range(1, request_count + 1)))

with open(output_path, "w", encoding="utf-8") as f:
    for status, elapsed in results:
        f.write(f"{status} {elapsed}\n")
PY

  python3 - <<'PY' "${label}" "${results_out}" "${body_size}"
import json, math, statistics, sys
from pathlib import Path
label, result_path, body_size = sys.argv[1], sys.argv[2], int(sys.argv[3])
rows = []
for line in Path(result_path).read_text().splitlines():
    if not line.strip():
        continue
    code_s, time_s = line.split()
    rows.append((int(code_s), float(time_s)))
assert rows, "no benchmark rows captured"
assert all(code == 200 for code, _ in rows), rows
times_ms = sorted(round(t * 1000, 3) for _, t in rows)
def percentile(values, pct):
    index = math.ceil((pct / 100) * len(values)) - 1
    index = max(0, min(index, len(values) - 1))
    return values[index]
print(json.dumps({
    "label": label,
    "requests": len(rows),
    "body_bytes": body_size,
    "p50_ms": percentile(times_ms, 50),
    "p95_ms": percentile(times_ms, 95),
    "p99_ms": percentile(times_ms, 99),
    "mean_ms": round(statistics.mean(times_ms), 3),
}, ensure_ascii=False))
PY

  rm -f "${body_out}" "${results_out}"
}

run_case "json-warm" "json" "warm"
run_case "msgpack-warm" "msgpack" "warm"
run_case "json-cold" "json" "cold"
run_case "msgpack-cold" "msgpack" "cold"
