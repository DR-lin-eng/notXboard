#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${BASE_URL:-http://127.0.0.1:18011}"
ADMIN_EMAIL="${ADMIN_EMAIL:-fulladmin@example.com}"
ADMIN_PASSWORD="${ADMIN_PASSWORD:-Passw0rd!2026}"
SECURE_PATH="${SECURE_PATH:?SECURE_PATH is required}"
MYSQL_CONTAINER="${MYSQL_CONTAINER:?MYSQL_CONTAINER is required}"
DB_DATABASE="${DB_DATABASE:-xboard}"
DB_USERNAME="${DB_USERNAME:-notxboard}"
DB_PASSWORD="${DB_PASSWORD:-notxboard-pass}"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

json_field() {
  local field="$1"
  FIELD="$field" python3 -c '
import json, os, sys
value = json.load(sys.stdin)
for part in os.environ["FIELD"].split("."):
    value = value.get(part) if isinstance(value, dict) else None
print("" if value is None else value)
'
}

json_data_has_id() {
  local expected_id="$1"
  EXPECTED_ID="$expected_id" python3 -c '
import json, os, sys
payload = json.load(sys.stdin)
items = payload.get("data") if isinstance(payload, dict) else None
if not isinstance(items, list):
    sys.exit(1)
expected = int(os.environ["EXPECTED_ID"])
if not any(isinstance(item, dict) and item.get("id") == expected for item in items):
    sys.exit(1)
'
}

json_users_has_id() {
  local expected_id="$1"
  EXPECTED_ID="$expected_id" python3 -c '
import json, os, sys
payload = json.load(sys.stdin)
items = payload.get("users") if isinstance(payload, dict) else None
if not isinstance(items, list):
    sys.exit(1)
expected = int(os.environ["EXPECTED_ID"])
if not any(isinstance(item, dict) and item.get("id") == expected for item in items):
    sys.exit(1)
'
}

bootstrap_token_from_command() {
  local expected_query="$1"
  EXPECTED_QUERY="$expected_query" python3 -c '
import json
import os
import re
import shlex
import sys

try:
    payload = json.load(sys.stdin)
    data = payload.get("data")
    command = data.get("command") if isinstance(data, dict) else None
    if not isinstance(command, str) or not command.strip():
        raise ValueError("data.command is missing")
    if "\n" in command or "\r" in command or "\x00" in command:
        raise ValueError("data.command must be one line")

    argv = shlex.split(command, comments=False, posix=True)
    pipes = [index for index, value in enumerate(argv) if value == "|"]
    if len(pipes) != 1 or pipes[0] + 1 >= len(argv) or argv[pipes[0] + 1] != "bash":
        raise ValueError("command must contain exactly one installer pipe to bash")
    positions = [index for index, value in enumerate(argv) if value == "--bootstrap-token"]
    if len(positions) != 1 or any(value.startswith("--bootstrap-token=") for value in argv):
        raise ValueError("command must contain exactly one standalone --bootstrap-token")
    position = positions[0]
    if position <= pipes[0] or position + 1 >= len(argv):
        raise ValueError("--bootstrap-token has no value")

    token = argv[position + 1]
    if re.fullmatch(r"[0-9a-f]{64}", token) is None:
        raise ValueError("bootstrap token must be 64 lowercase hexadecimal characters")

    query_positions = [index for index, value in enumerate(argv) if value == "--bootstrap-query"]
    if len(query_positions) != 1 or any(value.startswith("--bootstrap-query=") for value in argv):
        raise ValueError("command must contain exactly one standalone --bootstrap-query")
    query_position = query_positions[0]
    if query_position <= pipes[0] or query_position + 1 >= len(argv):
        raise ValueError("--bootstrap-query has no value")
    if argv[query_position + 1] != os.environ["EXPECTED_QUERY"]:
        raise ValueError("command bootstrap query does not match script_url")
    print(token)
except (AttributeError, TypeError, ValueError) as error:
    print(f"invalid bootstrap command: {error}", file=sys.stderr)
    sys.exit(1)
'
}

validate_signed_bootstrap_url() {
  local url="$1"
  local expected_kind="$2"
  local bootstrap_token="$3"
  BOOTSTRAP_URL="$url" EXPECTED_KIND="$expected_kind" BOOTSTRAP_TOKEN="$bootstrap_token" python3 -c '
import os
import re
import time
import urllib.parse

url = os.environ["BOOTSTRAP_URL"]
expected_kind = os.environ["EXPECTED_KIND"]
bootstrap_token = os.environ["BOOTSTRAP_TOKEN"]
parts = urllib.parse.urlsplit(url)
if parts.scheme not in {"http", "https"} or not parts.netloc or parts.fragment:
    raise SystemExit("bootstrap script_url must be an absolute HTTP(S) URL without a fragment")
if parts.username is not None or parts.password is not None:
    raise SystemExit("bootstrap script_url must not contain URL credentials")
if "ticket" in url.casefold() or "token" in url.casefold():
    raise SystemExit("bootstrap script_url exposed a ticket/token field")
if bootstrap_token in url:
    raise SystemExit("bootstrap script_url exposed the bootstrap token")

try:
    pairs = urllib.parse.parse_qsl(parts.query, keep_blank_values=True, strict_parsing=True)
except ValueError as error:
    raise SystemExit(f"bootstrap script_url query is malformed: {error}") from error
expected_keys = {"mb_v", "mb_kind", "mb_exp", "mb_asset", "mb_sig"}
keys = [key for key, _ in pairs]
if len(pairs) != 5 or len(set(keys)) != 5 or set(keys) != expected_keys:
    raise SystemExit("bootstrap query fields must be exactly mb_v/mb_kind/mb_exp/mb_asset/mb_sig")
values = dict(pairs)
if values["mb_v"] != "1" or values["mb_kind"] != expected_kind:
    raise SystemExit("bootstrap query version or kind is invalid")
if re.fullmatch(r"[1-9][0-9]*", values["mb_exp"]) is None:
    raise SystemExit("bootstrap query expiration is invalid")
remaining = int(values["mb_exp"]) - int(time.time())
if remaining <= 0 or remaining > 605:
    raise SystemExit("bootstrap query expiration is outside the 600-second TTL window")
if re.fullmatch(r"[0-9a-f]{64}", values["mb_asset"]) is None:
    raise SystemExit("bootstrap query asset id is invalid")
if re.fullmatch(r"[0-9a-f]{64}", values["mb_sig"]) is None:
    raise SystemExit("bootstrap query signature is invalid")
'
}

expect_status() {
  local expected="$1"
  shift
  local actual
  actual="$(curl -sS -o /dev/null -w '%{http_code}' "$@")"
  if [ "$actual" != "$expected" ]; then
    echo "expected HTTP ${expected}, got ${actual}: curl $*" >&2
    exit 1
  fi
}

mysql_scalar() {
  local sql="$1"
  docker exec "${MYSQL_CONTAINER}" mysql -N -s \
    -u"${DB_USERNAME}" -p"${DB_PASSWORD}" -D "${DB_DATABASE}" \
    -e "$sql" 2>/dev/null | tr -d '\r'
}

signed_asset_get() {
  local url="$1"
  local suffix="$2"
  local status
  status="$(curl -sS -D "${TMP_DIR}/${suffix}.headers" -o "${TMP_DIR}/${suffix}.body" -w '%{http_code}' "$url")"
  [ "$status" = "200" ] || { echo "signed asset returned HTTP ${status}" >&2; exit 1; }
  grep -qi '^cache-control: private, no-store' "${TMP_DIR}/${suffix}.headers"
}

echo "[1/10] authenticate and verify route/path gates"
LOGIN_RESPONSE="$(curl -fsS -X POST "${BASE_URL}/api/v1/passport/auth/login" \
  -H 'Content-Type: application/json' \
  --data "{\"email\":\"${ADMIN_EMAIL}\",\"password\":\"${ADMIN_PASSWORD}\"}")"
AUTH_DATA="$(printf '%s' "$LOGIN_RESPONSE" | json_field data.auth_data)"
[ -n "$AUTH_DATA" ] || { echo "login did not return auth_data" >&2; exit 1; }

expect_status 200 "${BASE_URL}/api/v2/${SECURE_PATH}/system/listHooks" -H "Authorization: ${AUTH_DATA}"
expect_status 404 "${BASE_URL}/api/v2/admin/system/listHooks" -H "Authorization: ${AUTH_DATA}" -H "X-Admin-Path: ${SECURE_PATH}"
expect_status 404 "${BASE_URL}/api/v1/admin/users" -H "Authorization: ${AUTH_DATA}"
expect_status 200 "${BASE_URL}/api/v1/admin/users" -H "Authorization: ${AUTH_DATA}" -H "X-Admin-Path: ${SECURE_PATH}"
expect_status 404 "${BASE_URL}/monitor/api/stats" -H "Authorization: ${AUTH_DATA}"
expect_status 200 "${BASE_URL}/monitor/api/stats" -H "Authorization: ${AUTH_DATA}" -H "X-Admin-Path: ${SECURE_PATH}"

echo "[2/10] verify user route method and authentication gates"
expect_status 403 "${BASE_URL}/api/v1/user/limits"
expect_status 405 -X POST "${BASE_URL}/api/v1/user/limits" -H "Authorization: ${AUTH_DATA}" -H 'Content-Type: application/json' --data '{}'
expect_status 200 "${BASE_URL}/api/v1/user/getActiveSession" -H "Authorization: ${AUTH_DATA}"

echo "[3/10] create an ordinary user and verify the admin role boundary"
BOUNDARY_EMAIL="permission-boundary-${RANDOM}-${RANDOM}@example.com"
mysql_scalar "INSERT INTO v2_user (email,password,uuid,token,subscribe_path,subscribe_key,subscribe_salt,transfer_enable,expired_at,created_at,updated_at) SELECT '${BOUNDARY_EMAIL}',password,UUID(),REPLACE(UUID(),'-',''),SUBSTRING(REPLACE(UUID(),'-',''),1,10),SUBSTRING(REPLACE(UUID(),'-',''),1,8),SUBSTRING(REPLACE(UUID(),'-',''),1,6),1000000000000,2000000000,UNIX_TIMESTAMP(),UNIX_TIMESTAMP() FROM v2_user WHERE email='${ADMIN_EMAIL}' LIMIT 1;"
BOUNDARY_ID="$(mysql_scalar "SELECT id FROM v2_user WHERE email='${BOUNDARY_EMAIL}' LIMIT 1;")"
[ -n "$BOUNDARY_ID" ] || { echo "ordinary boundary user missing" >&2; exit 1; }
BOUNDARY_LOGIN="$(curl -fsS -X POST "${BASE_URL}/api/v1/passport/auth/login" \
  -H 'Content-Type: application/json' \
  --data "{\"email\":\"${BOUNDARY_EMAIL}\",\"password\":\"${ADMIN_PASSWORD}\"}")"
BOUNDARY_AUTH="$(printf '%s' "$BOUNDARY_LOGIN" | json_field data.auth_data)"
[ -n "$BOUNDARY_AUTH" ] || { echo "ordinary boundary user login failed" >&2; exit 1; }
expect_status 403 "${BASE_URL}/api/v1/admin/users" -H "Authorization: ${BOUNDARY_AUTH}" -H "X-Admin-Path: ${SECURE_PATH}"
expect_status 404 "${BASE_URL}/api/v1/admin/users" -H "Authorization: ${BOUNDARY_AUTH}"

echo "[4/10] create TCPing agent and issue a short-lived installer bundle"
AGENT_RESPONSE="$(curl -fsS -X POST "${BASE_URL}/api/v1/user/tcping/agents" \
  -H "Authorization: ${AUTH_DATA}" -H 'Content-Type: application/json' \
  --data '{"name":"permission-boundary-agent","location_code":"US","location_name":"Integration"}')"
AGENT_ID="$(printf '%s' "$AGENT_RESPONSE" | json_field data.id)"
[ -n "$AGENT_ID" ] || { echo "agent creation did not return id" >&2; exit 1; }
AGENT_TOKEN="$(mysql_scalar "SELECT token FROM tcping_agents WHERE id=${AGENT_ID} LIMIT 1;")"
[ -n "$AGENT_TOKEN" ] || { echo "agent token missing in database" >&2; exit 1; }
case "$AGENT_RESPONSE" in *"$AGENT_TOKEN"*) echo "agent response leaked long credential" >&2; exit 1;; esac
expect_status 404 "${BASE_URL}/api/v1/user/tcping/agents/${AGENT_ID}" -H "Authorization: ${BOUNDARY_AUTH}"
expect_status 404 "${BASE_URL}/api/v1/user/tcping/agents/${AGENT_ID}/install-command" -H "Authorization: ${BOUNDARY_AUTH}"
expect_status 404 -X POST "${BASE_URL}/api/v1/user/tcping/agents/${AGENT_ID}/rotate-token" -H "Authorization: ${BOUNDARY_AUTH}"
expect_status 404 -X POST "${BASE_URL}/api/v1/user/tcping/agents/${AGENT_ID}/toggle" -H "Authorization: ${BOUNDARY_AUTH}"

INSTALL_RESPONSE="$(curl -fsS "${BASE_URL}/api/v1/user/tcping/agents/${AGENT_ID}/install-command" -H "Authorization: ${AUTH_DATA}")"
case "$INSTALL_RESPONSE" in *"$AGENT_TOKEN"*) echo "install response leaked long credential" >&2; exit 1;; esac
SCRIPT_URL="$(printf '%s' "$INSTALL_RESPONSE" | json_field data.script_url)"
[ -n "$SCRIPT_URL" ] || { echo "install response missing script_url" >&2; exit 1; }
BOOTSTRAP_TTL="$(printf '%s' "$INSTALL_RESPONSE" | json_field data.bootstrap_expires_in)"
[ "$BOOTSTRAP_TTL" = "600" ] || { echo "TCPing bootstrap TTL must be 600 seconds" >&2; exit 1; }
BOOTSTRAP_QUERY="${SCRIPT_URL#*\?}"
BOOTSTRAP_TICKET="$(printf '%s' "$INSTALL_RESPONSE" | bootstrap_token_from_command "$BOOTSTRAP_QUERY")"
validate_signed_bootstrap_url "$SCRIPT_URL" tcping "$BOOTSTRAP_TICKET"

echo "[5/10] verify installer resource allowlist, retry, and no-store"
expect_status 404 "${BASE_URL}/tcping-agent-install.sh"
signed_asset_get "$SCRIPT_URL" tcping-install-1
signed_asset_get "$SCRIPT_URL" tcping-install-2
signed_asset_get "${BASE_URL}/tcping-agent-src/go.mod?${BOOTSTRAP_QUERY}" tcping-go-mod
signed_asset_get "${BASE_URL}/tcping-agent-src/main.go?${BOOTSTRAP_QUERY}" tcping-main-go
expect_status 404 "${BASE_URL}/tcping-agent-src/extra.go?${BOOTSTRAP_QUERY}"

echo "[6/10] exchange TCPing ticket once and revalidate the current subject"
EXCHANGED_TOKEN="$(curl -fsS -X POST "${BASE_URL}/api/v1/agent/bootstrap/tcping?${BOOTSTRAP_QUERY}" \
  -H "Authorization: Bearer ${BOOTSTRAP_TICKET}")"
[ "$EXCHANGED_TOKEN" = "$AGENT_TOKEN" ] || { echo "TCPing exchange returned the wrong credential" >&2; exit 1; }
expect_status 404 -X POST "${BASE_URL}/api/v1/agent/bootstrap/tcping?${BOOTSTRAP_QUERY}" -H "Authorization: Bearer ${BOOTSTRAP_TICKET}"
expect_status 404 "$SCRIPT_URL"

echo "[7/10] verify cross-user object writes fail closed"
ADMIN_ID="$(mysql_scalar "SELECT id FROM v2_user WHERE email='${ADMIN_EMAIL}' LIMIT 1;")"
NODE_NAME="permission-boundary-node-${RANDOM}-${RANDOM}"
mysql_scalar "INSERT INTO server_nodes (user_id,name,host,port,protocol,status,created_at,updated_at) VALUES (${ADMIN_ID},'${NODE_NAME}','node.example.com',443,'vmess','active',NOW(),NOW());"
NODE_ID="$(mysql_scalar "SELECT id FROM server_nodes WHERE name='${NODE_NAME}' ORDER BY id DESC LIMIT 1;")"
expect_status 404 "${BASE_URL}/api/v1/user/server-nodes/${NODE_ID}" -H "Authorization: ${BOUNDARY_AUTH}"
expect_status 404 -X PUT "${BASE_URL}/api/v1/user/server-nodes/${NODE_ID}" -H "Authorization: ${BOUNDARY_AUTH}" -H 'Content-Type: application/json' --data '{"name":"cross-user-write"}'
expect_status 404 -X DELETE "${BASE_URL}/api/v1/user/server-nodes/${NODE_ID}" -H "Authorization: ${BOUNDARY_AUTH}"
expect_status 404 "${BASE_URL}/api/v1/user/server-nodes/${NODE_ID}/status" -H "Authorization: ${BOUNDARY_AUTH}"
expect_status 404 "${BASE_URL}/api/v1/user/server-nodes/${NODE_ID}/traffic" -H "Authorization: ${BOUNDARY_AUTH}"
expect_status 404 "${BASE_URL}/api/v1/user/server-nodes/${NODE_ID}/access/stats" -H "Authorization: ${BOUNDARY_AUTH}"
expect_status 404 "${BASE_URL}/api/v1/user/server-nodes/${NODE_ID}/tcping" -H "Authorization: ${BOUNDARY_AUTH}"
expect_status 404 "${BASE_URL}/api/v1/user/server-nodes/${NODE_ID}/audit-logs" -H "Authorization: ${BOUNDARY_AUTH}"
expect_status 404 "${BASE_URL}/api/v1/user/server-nodes/${NODE_ID}/audit-rules" -H "Authorization: ${BOUNDARY_AUTH}"
expect_status 404 "${BASE_URL}/api/v1/user/node-admin/server-nodes/${NODE_ID}/users-traffic" -H "Authorization: ${BOUNDARY_AUTH}"
expect_status 404 -X POST "${BASE_URL}/api/v1/user/server-nodes/${NODE_ID}/deploy" -H "Authorization: ${BOUNDARY_AUTH}"
expect_status 404 -X POST "${BASE_URL}/api/v1/user/server-nodes/${NODE_ID}/access" -H "Authorization: ${BOUNDARY_AUTH}" -H 'Content-Type: application/json' --data '{"authorized_users":[]}'
expect_status 404 -X POST "${BASE_URL}/api/v1/user/node-admin/server-nodes/${NODE_ID}/blacklist" -H "Authorization: ${BOUNDARY_AUTH}" -H 'Content-Type: application/json' --data "{\"user_id\":${ADMIN_ID},\"reason\":\"cross-user\"}"
expect_status 404 -X POST "${BASE_URL}/api/v1/user/node-admin/server-nodes/${NODE_ID}/unblacklist" -H "Authorization: ${BOUNDARY_AUTH}" -H 'Content-Type: application/json' --data "{\"user_id\":${ADMIN_ID}}"
expect_status 404 -X POST "${BASE_URL}/api/v1/user/server-nodes/${NODE_ID}/audit-rules" -H "Authorization: ${BOUNDARY_AUTH}" -H 'Content-Type: application/json' --data '{"rule_type":"domain","rule_pattern":"example.com","action":"block"}'
expect_status 404 -X POST "${BASE_URL}/api/v1/user/server-nodes/${NODE_ID}/deploy-command" -H "Authorization: ${BOUNDARY_AUTH}"
expect_status 404 -X POST "${BASE_URL}/api/v1/user/server-nodes/${NODE_ID}/deploy-command/rotate-token" -H "Authorization: ${BOUNDARY_AUTH}"
mysql_scalar "SELECT COUNT(*) FROM server_nodes WHERE id=${NODE_ID} AND user_id=${ADMIN_ID};" | grep -qx '1'
mysql_scalar "SELECT COUNT(*) FROM user_node_blacklist WHERE node_id=${NODE_ID};" | grep -qx '0'
mysql_scalar "SELECT COUNT(*) FROM audit_rules WHERE node_id=${NODE_ID};" | grep -qx '0'

mysql_scalar "INSERT INTO audit_rules (node_id,rule_type,rule_pattern,action,is_active,created_at,updated_at) VALUES (${NODE_ID},'domain','owner.example','block',1,NOW(),NOW());"
OWNER_RULE_ID="$(mysql_scalar "SELECT id FROM audit_rules WHERE node_id=${NODE_ID} ORDER BY id DESC LIMIT 1;")"
expect_status 404 -X PUT "${BASE_URL}/api/v1/user/server-nodes/${NODE_ID}/audit-rules/${OWNER_RULE_ID}" -H "Authorization: ${BOUNDARY_AUTH}" -H 'Content-Type: application/json' --data '{"rule_pattern":"cross-user.example"}'
expect_status 404 -X DELETE "${BASE_URL}/api/v1/user/server-nodes/${NODE_ID}/audit-rules/${OWNER_RULE_ID}" -H "Authorization: ${BOUNDARY_AUTH}"
mysql_scalar "SELECT rule_pattern FROM audit_rules WHERE id=${OWNER_RULE_ID} AND node_id=${NODE_ID};" | grep -qx 'owner.example'

BOUNDARY_NODE_NAME="permission-boundary-owned-${RANDOM}-${RANDOM}"
mysql_scalar "INSERT INTO server_nodes (user_id,name,host,port,protocol,status,created_at,updated_at) VALUES (${BOUNDARY_ID},'${BOUNDARY_NODE_NAME}','boundary.example.com',443,'vmess','active',NOW(),NOW());"
BOUNDARY_NODE_ID="$(mysql_scalar "SELECT id FROM server_nodes WHERE name='${BOUNDARY_NODE_NAME}' ORDER BY id DESC LIMIT 1;")"
expect_status 200 "${BASE_URL}/api/v1/user/server-nodes/${BOUNDARY_NODE_ID}" -H "Authorization: ${BOUNDARY_AUTH}"
expect_status 403 -X POST "${BASE_URL}/api/v1/user/server-nodes/${BOUNDARY_NODE_ID}/share/user" -H "Authorization: ${BOUNDARY_AUTH}" -H 'Content-Type: application/json' --data "{\"user_id\":${ADMIN_ID}}"
expect_status 403 -X POST "${BASE_URL}/api/v1/user/server-nodes/${BOUNDARY_NODE_ID}/access" -H "Authorization: ${BOUNDARY_AUTH}" -H 'Content-Type: application/json' --data '{"min_trust_level":0}'

echo "[8/10] verify V2bX credentials are bound to one node and hidden from the browser"
NODE_INSTALL_RESPONSE="$(curl -fsS -X POST "${BASE_URL}/api/v1/user/server-nodes/${NODE_ID}/deploy-command" -H "Authorization: ${AUTH_DATA}")"
NODE_TOKEN="$(mysql_scalar "SELECT v2bx_token FROM server_nodes WHERE id=${NODE_ID} LIMIT 1;")"
[ -n "$NODE_TOKEN" ] || { echo "V2bX token missing in database" >&2; exit 1; }
case "$NODE_INSTALL_RESPONSE" in *"$NODE_TOKEN"*) echo "V2bX response leaked long credential" >&2; exit 1;; esac
NODE_SCRIPT_URL="$(printf '%s' "$NODE_INSTALL_RESPONSE" | json_field data.script_url)"
[ -n "$NODE_SCRIPT_URL" ] || { echo "V2bX response missing script_url" >&2; exit 1; }
NODE_BOOTSTRAP_TTL="$(printf '%s' "$NODE_INSTALL_RESPONSE" | json_field data.bootstrap_expires_in)"
[ "$NODE_BOOTSTRAP_TTL" = "600" ] || { echo "V2bX bootstrap TTL must be 600 seconds" >&2; exit 1; }
NODE_QUERY="${NODE_SCRIPT_URL#*\?}"
NODE_TICKET="$(printf '%s' "$NODE_INSTALL_RESPONSE" | bootstrap_token_from_command "$NODE_QUERY")"
validate_signed_bootstrap_url "$NODE_SCRIPT_URL" v2bx "$NODE_TICKET"
signed_asset_get "$NODE_SCRIPT_URL" v2bx-install
NODE_EXCHANGED="$(curl -fsS -X POST "${BASE_URL}/api/v1/agent/bootstrap/v2bx?${NODE_QUERY}" -H "Authorization: Bearer ${NODE_TICKET}")"
[ "$NODE_EXCHANGED" = "$NODE_TOKEN" ] || { echo "V2bX exchange returned the wrong credential" >&2; exit 1; }
expect_status 404 -X POST "${BASE_URL}/api/v1/agent/bootstrap/v2bx?${NODE_QUERY}" -H "Authorization: Bearer ${NODE_TICKET}"

SECOND_NODE_NAME="permission-boundary-node-2-${RANDOM}-${RANDOM}"
SECOND_NODE_TOKEN="$(python3 -c 'import secrets; print(secrets.token_hex(32))')"
mysql_scalar "INSERT INTO server_nodes (user_id,name,host,port,protocol,status,v2bx_token,created_at,updated_at) VALUES (${ADMIN_ID},'${SECOND_NODE_NAME}','node2.example.com',443,'vmess','active','${SECOND_NODE_TOKEN}',NOW(),NOW());"
SECOND_NODE_ID="$(mysql_scalar "SELECT id FROM server_nodes WHERE name='${SECOND_NODE_NAME}' ORDER BY id DESC LIMIT 1;")"
expect_status 200 "${BASE_URL}/api/v1/server/UniProxy/config?token=${NODE_TOKEN}&node_id=${NODE_ID}"
expect_status 401 "${BASE_URL}/api/v1/server/UniProxy/config?token=${NODE_TOKEN}&node_id=${SECOND_NODE_ID}"

echo "[9/10] verify dirty-data authorization, weak links, notices, and hidden categories"
DIRTY_PLAN_NAME="permission-boundary-dirty-plan-${RANDOM}-${RANDOM}"
WEAK_SHARE_TOKEN="weak${RANDOM}${RANDOM}"
mysql_scalar "INSERT INTO v2_plan (name,scope,owner_user_id,node_ids,prices,sell,\`show\`,visibility_scope,share_token,created_at,updated_at) VALUES ('${DIRTY_PLAN_NAME}','node',${BOUNDARY_ID},JSON_ARRAY(${NODE_ID}),JSON_OBJECT('monthly',1),1,0,'link_only','${WEAK_SHARE_TOKEN}',UNIX_TIMESTAMP(),UNIX_TIMESTAMP());"
DIRTY_PLAN_ID="$(mysql_scalar "SELECT id FROM v2_plan WHERE name='${DIRTY_PLAN_NAME}' ORDER BY id DESC LIMIT 1;")"
mysql_scalar "SELECT COUNT(*) FROM v2_order WHERE user_id=${BOUNDARY_ID};" | grep -qx '0'
expect_status 400 "${BASE_URL}/api/v1/user/plan/fetch?token=${WEAK_SHARE_TOKEN}" -H "Authorization: ${BOUNDARY_AUTH}"
WEAK_ORDER_COUNT_BEFORE="$(mysql_scalar "SELECT COUNT(*) FROM v2_order WHERE user_id=${BOUNDARY_ID} AND plan_id=${DIRTY_PLAN_ID};")"
expect_status 400 -X POST "${BASE_URL}/api/v1/user/order/save" -H "Authorization: ${BOUNDARY_AUTH}" -H 'Content-Type: application/json' --data "{\"plan_id\":${DIRTY_PLAN_ID},\"purchase_token\":\"${WEAK_SHARE_TOKEN}\",\"period\":\"month_price\"}"
WEAK_ORDER_COUNT_AFTER="$(mysql_scalar "SELECT COUNT(*) FROM v2_order WHERE user_id=${BOUNDARY_ID} AND plan_id=${DIRTY_PLAN_ID};")"
[ "$WEAK_ORDER_COUNT_AFTER" = "$WEAK_ORDER_COUNT_BEFORE" ] || { echo "weak share token created an order" >&2; exit 1; }

DIRTY_ORDER_ID="$(mysql_scalar "SELECT COALESCE(MAX(order_id),0)+1 FROM user_plan_subscriptions;")"
mysql_scalar "INSERT INTO user_plan_subscriptions (user_id,plan_id,order_id,period,traffic_allowance_kb,used_traffic_kb,started_at,expired_at,status,created_at,updated_at) VALUES (${BOUNDARY_ID},${DIRTY_PLAN_ID},${DIRTY_ORDER_ID},'month_price',1000000,0,UNIX_TIMESTAMP(),NULL,1,UNIX_TIMESTAMP(),UNIX_TIMESTAMP());"
ACCESSIBLE_RESPONSE="$(curl -fsS "${BASE_URL}/api/v1/user/accessible-nodes" -H "Authorization: ${BOUNDARY_AUTH}")"
printf '%s' "$ACCESSIBLE_RESPONSE" | json_data_has_id "$BOUNDARY_NODE_ID" || { echo "owned node missing from accessible list" >&2; exit 1; }
if printf '%s' "$ACCESSIBLE_RESPONSE" | json_data_has_id "$NODE_ID"; then
  echo "cross-owner dirty plan granted node access" >&2
  exit 1
fi
UNIPROXY_USERS="$(curl -fsS "${BASE_URL}/api/v1/server/UniProxy/user?token=${NODE_TOKEN}&node_id=${NODE_ID}")"
printf '%s' "$UNIPROXY_USERS" | json_users_has_id "$ADMIN_ID" || { echo "node owner missing from UniProxy user list" >&2; exit 1; }
if printf '%s' "$UNIPROXY_USERS" | json_users_has_id "$BOUNDARY_ID"; then
  echo "cross-owner dirty plan granted reverse UniProxy access" >&2
  exit 1
fi
expect_status 403 -X POST "${BASE_URL}/api/v1/server/UniProxy/audit?token=${NODE_TOKEN}&node_id=${NODE_ID}" -H 'Content-Type: application/json' --data "{\"user_id\":${BOUNDARY_ID}}"
SUBSCRIBE_INFO="$(curl -fsS "${BASE_URL}/api/v1/user/getSubscribe" -H "Authorization: ${BOUNDARY_AUTH}")"
SUBSCRIBE_URL="$(printf '%s' "$SUBSCRIBE_INFO" | json_field data.subscribe_url)"
[ -n "$SUBSCRIBE_URL" ] || { echo "boundary user subscribe_url missing" >&2; exit 1; }
case "$SUBSCRIBE_URL" in
  *\?*) SUBSCRIBE_TEST_URL="${SUBSCRIBE_URL}&flag=clash" ;;
  *) SUBSCRIBE_TEST_URL="${SUBSCRIBE_URL}?flag=clash" ;;
esac
SUBSCRIBE_PAYLOAD="$(curl -fsS "$SUBSCRIBE_TEST_URL")"
case "$SUBSCRIBE_PAYLOAD" in *"boundary.example.com"*) ;; *) echo "owned node missing from subscription output" >&2; exit 1;; esac
case "$SUBSCRIBE_PAYLOAD" in *"node.example.com"*) echo "cross-owner dirty plan leaked into subscription output" >&2; exit 1;; esac

VALID_PLAN_NAME="permission-boundary-valid-plan-${RANDOM}-${RANDOM}"
mysql_scalar "INSERT INTO v2_plan (name,scope,owner_user_id,node_ids,prices,sell,\`show\`,visibility_scope,created_at,updated_at) VALUES ('${VALID_PLAN_NAME}','node',${ADMIN_ID},JSON_ARRAY(${NODE_ID}),JSON_OBJECT('monthly',1),1,0,'public',UNIX_TIMESTAMP(),UNIX_TIMESTAMP());"
VALID_PLAN_ID="$(mysql_scalar "SELECT id FROM v2_plan WHERE name='${VALID_PLAN_NAME}' ORDER BY id DESC LIMIT 1;")"
expect_status 422 -X PUT "${BASE_URL}/api/v1/admin/node-plans/${VALID_PLAN_ID}" -H "Authorization: ${AUTH_DATA}" -H "X-Admin-Path: ${SECURE_PATH}" -H 'Content-Type: application/json' --data "{\"owner_user_id\":${BOUNDARY_ID}}"
mysql_scalar "SELECT owner_user_id FROM v2_plan WHERE id=${VALID_PLAN_ID};" | grep -qx "$ADMIN_ID"

HIDDEN_CATEGORY="hidden-${RANDOM}-${RANDOM}"
VISIBLE_CATEGORY="visible-${RANDOM}-${RANDOM}"
mysql_scalar "INSERT INTO v2_knowledge (language,category,title,body,sort,\`show\`,created_at,updated_at) VALUES ('zh-CN','${HIDDEN_CATEGORY}','hidden','hidden',0,0,UNIX_TIMESTAMP(),UNIX_TIMESTAMP()),('zh-CN','${VISIBLE_CATEGORY}','visible','visible',0,1,UNIX_TIMESTAMP(),UNIX_TIMESTAMP());"
CATEGORY_RESPONSE="$(curl -fsS "${BASE_URL}/api/v1/user/knowledge/getCategory" -H "Authorization: ${BOUNDARY_AUTH}")"
case "$CATEGORY_RESPONSE" in *"$VISIBLE_CATEGORY"*) ;; *) echo "visible knowledge category missing" >&2; exit 1;; esac
case "$CATEGORY_RESPONSE" in *"$HIDDEN_CATEGORY"*) echo "hidden knowledge category leaked" >&2; exit 1;; esac

UNTRUSTED_NOTICE="untrusted-global-${RANDOM}-${RANDOM}"
TRUSTED_NOTICE="trusted-global-${RANDOM}-${RANDOM}"
BANNED_NOTICE="banned-global-${RANDOM}-${RANDOM}"
BANNED_PUBLISHER_EMAIL="banned-publisher-${RANDOM}-${RANDOM}@example.com"
NOTICE_SORT="$(mysql_scalar "SELECT COALESCE(MIN(sort),0)-1 FROM v2_notice;")"
mysql_scalar "INSERT INTO v2_user (email,password,uuid,token,subscribe_path,subscribe_key,subscribe_salt,transfer_enable,expired_at,is_admin,banned,created_at,updated_at) SELECT '${BANNED_PUBLISHER_EMAIL}',password,UUID(),REPLACE(UUID(),'-',''),SUBSTRING(REPLACE(UUID(),'-',''),1,10),SUBSTRING(REPLACE(UUID(),'-',''),1,8),SUBSTRING(REPLACE(UUID(),'-',''),1,6),1000000000000,2000000000,1,1,UNIX_TIMESTAMP(),UNIX_TIMESTAMP() FROM v2_user WHERE id=${ADMIN_ID} LIMIT 1;"
BANNED_PUBLISHER_ID="$(mysql_scalar "SELECT id FROM v2_user WHERE email='${BANNED_PUBLISHER_EMAIL}' LIMIT 1;")"
mysql_scalar "INSERT INTO v2_notice (sort,title,content,\`show\`,popup,author_user_id,scope_type,target_plan_ids,created_at,updated_at) VALUES (${NOTICE_SORT},'${UNTRUSTED_NOTICE}','private',1,0,${BOUNDARY_ID},'global','[]',UNIX_TIMESTAMP(),UNIX_TIMESTAMP()),(${NOTICE_SORT},'${TRUSTED_NOTICE}','trusted',1,0,${ADMIN_ID},'global','[]',UNIX_TIMESTAMP(),UNIX_TIMESTAMP()),(${NOTICE_SORT},'${BANNED_NOTICE}','banned',1,0,${BANNED_PUBLISHER_ID},'global','[]',UNIX_TIMESTAMP(),UNIX_TIMESTAMP());"
ADMIN_NOTICE_RESPONSE="$(curl -fsS "${BASE_URL}/api/v1/user/notice/fetch?page_size=20" -H "Authorization: ${AUTH_DATA}")"
case "$ADMIN_NOTICE_RESPONSE" in *"$UNTRUSTED_NOTICE"*) echo "ordinary-author historical global notice leaked" >&2; exit 1;; esac
case "$ADMIN_NOTICE_RESPONSE" in *"$BANNED_NOTICE"*) echo "banned publisher global notice leaked" >&2; exit 1;; esac
BOUNDARY_NOTICE_RESPONSE="$(curl -fsS "${BASE_URL}/api/v1/user/notice/fetch?page_size=20" -H "Authorization: ${BOUNDARY_AUTH}")"
case "$BOUNDARY_NOTICE_RESPONSE" in *"$TRUSTED_NOTICE"*) ;; *) echo "trusted global notice missing for another user" >&2; exit 1;; esac

echo "[10/10] verify database uniqueness for new installations when present"
mysql_scalar "SELECT COUNT(*) FROM tcping_agents WHERE token='${AGENT_TOKEN}';" | grep -qx '1'
mysql_scalar "SELECT COUNT(*) FROM server_nodes WHERE v2bx_token='${NODE_TOKEN}';" | grep -qx '1'

echo "Permission boundary verification passed."
