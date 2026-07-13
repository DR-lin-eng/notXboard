#!/usr/bin/env bash
set -euo pipefail
umask 077

SCRIPT_VERSION="2026-01-28"

PANEL_URL=""
NODE_ID=""
NODE_TYPE=""
TOKEN=""
BOOTSTRAP_TOKEN=""
BOOTSTRAP_QUERY=""
CORE="sing"
LISTEN_IP="::"
SEND_IP="0.0.0.0"
DEVICE_ONLINE_MIN_TRAFFIC="200"
MIN_REPORT_TRAFFIC="0"
TCP_FAST_OPEN="false"
SNIFF_ENABLED="true"
CERT_MODE=""
CERT_DOMAIN=""
CERT_FILE="/etc/V2bX/fullchain.cer"
KEY_FILE="/etc/V2bX/cert.key"
CERT_PROVIDER="cloudflare"
CERT_EMAIL="v2bx@github.com"
CERT_REJECT_UNKNOWN_SNI="false"
CONFIG_DIR="/etc/V2bX"
INSTALL_URL="https://raw.githubusercontent.com/wyx2685/V2bX-script/c532ec57a67d7544c700f3f438c09dffcd0b1313/install.sh"
INSTALL_SHA256="b265d55aff142a490fc5c935aa34293cc466642d6f142678f0983ad1c087c7ac"
SKIP_INSTALL=0

usage() {
  cat <<'EOF'
Usage:
  v2bx-install.sh --panel <https://panel.example.com> --node-id <id> --node-type <type> (--bootstrap-token <ticket> --bootstrap-query <query> | --token <token>) [options]

Required:
  --panel        Panel base URL with scheme
  --node-id      Node ID
  --node-type    vmess|vless|trojan|shadowsocks|hysteria|hysteria2|tuic|anytls
  --bootstrap-token Short-lived installer ticket
  --bootstrap-query Signed installer query returned by the panel
  --token        V2bX token for an offline/manual install

Options:
  --core         Core type: sing|xray (default: sing; xray supports vmess|vless|trojan|shadowsocks)
  --listen-ip    Listen IP for node (default: ::)
  --send-ip      Send IP for node (default: 0.0.0.0)
  --cert-mode    Certificate mode: none|self|file|dns|http (default: auto)
  --cert-domain  Certificate domain (default: node host or panel host)
  --cert-file    Certificate file path (default: /etc/V2bX/fullchain.cer)
  --key-file     Certificate key path (default: /etc/V2bX/cert.key)
  --cert-provider ACME DNS provider (default: cloudflare)
  --cert-email   ACME email (default: v2bx@github.com)
  --config-dir   Config directory (default: /etc/V2bX)
  --install-url  V2bX install script URL
  --install-sha256 SHA-256 of the V2bX install script
  --skip-install Skip V2bX installation
  -h, --help     Show this help
EOF
}

die() {
  echo "[v2bx-install] $*" >&2
  exit 1
}

to_bool() {
  local raw="${1:-}"
  local lower
  lower="$(printf '%s' "$raw" | tr 'A-Z' 'a-z')"
  case "$lower" in
    1|true|yes|on) echo "true" ;;
    0|false|no|off) echo "false" ;;
    *)
      die "Invalid boolean value: $raw"
      ;;
  esac
}

has_unsafe_text() {
  local value="$1"
  [[ "$value" =~ [[:cntrl:]] || "$value" == *\"* || "$value" == *"'"* || "$value" == *\\* ]]
}

validate_http_url() {
  local name="$1"
  local value="$2"
  [[ "$value" == http://* || "$value" == https://* ]] || die "$name must include http:// or https://"
  if [[ "$value" =~ [[:space:]] ]] || has_unsafe_text "$value"; then
    die "$name contains unsafe characters"
  fi
  local authority="${value#*://}"
  authority="${authority%%/*}"
  [[ "$authority" != *"@"* && "$authority" != *"?"* && "$authority" != *"#"* ]] \
    || die "$name must not contain credentials, query, or fragment"
  if [[ "$value" == http://* ]]; then
    [[ "$authority" =~ ^(localhost|127\.0\.0\.1)(:[0-9]+)?$ || "$authority" =~ ^\[::1\](:[0-9]+)?$ ]] \
      || die "$name must use https except for loopback development"
  fi
}

validate_sha256() {
  local name="$1"
  local value="$2"
  [[ "$value" =~ ^[A-Fa-f0-9]{64}$ ]] || die "$name must be a 64-character SHA-256"
}

validate_uint() {
  local name="$1"
  local value="$2"
  [[ "$value" =~ ^[0-9]+$ ]] || die "$name must be an unsigned integer"
}

validate_token() {
  local name="$1"
  local value="$2"
  [[ "$value" =~ ^[A-Za-z0-9._:-]{8,512}$ ]] || die "$name contains unsupported characters"
}

validate_path() {
  local name="$1"
  local value="$2"
  [[ "$value" =~ ^/[A-Za-z0-9._@%+=,/:+-]+$ ]] || die "$name must be an absolute path without spaces or shell metacharacters"
  [[ "$value" != *"//"* && "$value" != */ ]] || die "$name must not contain duplicate or trailing slashes"
  local component
  local -a parts=()
  IFS='/' read -r -a parts <<< "${value#/}"
  for component in "${parts[@]}"; do
    [[ "$component" != "." && "$component" != ".." ]] || die "$name must not contain . or .. path components"
  done
}

reject_symlink_components() {
  local name="$1"
  local value="$2"
  local current="/"
  local component
  local -a parts=()
  validate_path "$name" "$value"
  IFS='/' read -r -a parts <<< "${value#/}"
  for component in "${parts[@]}"; do
    [[ -n "$component" ]] || continue
    current="${current%/}/${component}"
    [[ ! -L "$current" ]] || die "$name must not contain symbolic links: $current"
  done
}

atomic_write_file() {
  local target="$1"
  local mode="$2"
  local dir
  local base
  local tmp
  validate_path "target path" "$target"
  reject_symlink_components "target path" "$target"
  if [[ -e "$target" && ! -f "$target" ]]; then
    die "target path must be a regular file: $target"
  fi
  dir="$(dirname "$target")"
  base="$(basename "$target")"
  mkdir -p "$dir"
  reject_symlink_components "target path" "$target"
  tmp="$(mktemp "${dir}/.${base}.tmp.XXXXXX")"
  if ! cat > "$tmp"; then
    rm -f -- "$tmp"
    die "failed to write temporary file for $target"
  fi
  if ! chmod "$mode" "$tmp" || ! mv -f -- "$tmp" "$target"; then
    rm -f -- "$tmp"
    die "failed to replace $target"
  fi
}

validate_json_value() {
  local name="$1"
  local value="$2"
  if has_unsafe_text "$value"; then
    die "$name contains unsafe JSON characters"
  fi
}

validate_bootstrap_query() {
  local value="$1"
  [[ ${#value} -le 512 ]] || die "--bootstrap-query is too long"
  [[ "$value" =~ ^mb_v=1\&mb_kind=v2bx\&mb_exp=[0-9]{10,12}\&mb_asset=[A-Fa-f0-9]{64}\&mb_sig=[A-Fa-f0-9]{64}$ ]] \
    || die "--bootstrap-query is invalid"
}

exchange_bootstrap_credential() {
  command -v curl >/dev/null 2>&1 || die "curl is required to exchange the installer ticket"
  local credential
  local protocol="=https"
  [[ "$PANEL_URL" == http://* ]] && protocol="=http"
  credential="$(curl -fsS --proto "$protocol" --proto-redir "$protocol" -X POST \
    -H "Authorization: Bearer ${BOOTSTRAP_TOKEN}" \
    "${PANEL_URL}/api/v1/agent/bootstrap/v2bx?${BOOTSTRAP_QUERY}")" \
    || die "failed to exchange installer ticket; generate a new install command"
  validate_token "exchanged V2bX token" "$credential"
  TOKEN="$credential"
  BOOTSTRAP_TOKEN=""
  BOOTSTRAP_QUERY=""
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --panel)
      PANEL_URL="$2"; shift 2;;
    --node-id)
      NODE_ID="$2"; shift 2;;
    --node-type)
      NODE_TYPE="$2"; shift 2;;
    --token)
      TOKEN="$2"; shift 2;;
    --bootstrap-token)
      BOOTSTRAP_TOKEN="$2"; shift 2;;
    --bootstrap-query)
      BOOTSTRAP_QUERY="$2"; shift 2;;
    --core)
      CORE="$2"; shift 2;;
    --listen-ip)
      LISTEN_IP="$2"; shift 2;;
    --send-ip)
      SEND_IP="$2"; shift 2;;
    --cert-mode)
      CERT_MODE="$2"; shift 2;;
    --cert-domain)
      CERT_DOMAIN="$2"; shift 2;;
    --cert-file)
      CERT_FILE="$2"; shift 2;;
    --key-file)
      KEY_FILE="$2"; shift 2;;
    --cert-provider)
      CERT_PROVIDER="$2"; shift 2;;
    --cert-email)
      CERT_EMAIL="$2"; shift 2;;
    --config-dir)
      CONFIG_DIR="$2"; shift 2;;
    --install-url)
      INSTALL_URL="$2"; shift 2;;
    --install-sha256)
      INSTALL_SHA256="$2"; shift 2;;
    --skip-install)
      SKIP_INSTALL=1; shift;;
    -h|--help)
      usage; exit 0;;
    *)
      die "Unknown argument: $1";;
  esac
done

[[ -n "$PANEL_URL" ]] || die "Missing --panel"
[[ -n "$NODE_ID" ]] || die "Missing --node-id"
[[ -n "$NODE_TYPE" ]] || die "Missing --node-type"
if [[ -n "$TOKEN" && ( -n "$BOOTSTRAP_TOKEN" || -n "$BOOTSTRAP_QUERY" ) ]]; then
  die "Use either --token or the bootstrap options, not both"
fi
if [[ -z "$TOKEN" && ( -z "$BOOTSTRAP_TOKEN" || -z "$BOOTSTRAP_QUERY" ) ]]; then
  die "Missing --bootstrap-token/--bootstrap-query"
fi

validate_http_url "--panel" "$PANEL_URL"
validate_http_url "--install-url" "$INSTALL_URL"
validate_sha256 "--install-sha256" "$INSTALL_SHA256"
validate_uint "--node-id" "$NODE_ID"
validate_uint "--device-online-min-traffic" "$DEVICE_ONLINE_MIN_TRAFFIC"
validate_uint "--min-report-traffic" "$MIN_REPORT_TRAFFIC"
if [[ -n "$TOKEN" ]]; then
  validate_token "--token" "$TOKEN"
else
  [[ "$BOOTSTRAP_TOKEN" =~ ^[A-Fa-f0-9]{64}$ ]] || die "--bootstrap-token is invalid"
  validate_bootstrap_query "$BOOTSTRAP_QUERY"
fi
validate_json_value "--listen-ip" "$LISTEN_IP"
validate_json_value "--send-ip" "$SEND_IP"
validate_json_value "--cert-domain" "$CERT_DOMAIN"
validate_json_value "--cert-email" "$CERT_EMAIL"
[[ "$CERT_PROVIDER" =~ ^[A-Za-z0-9_-]+$ ]] || die "--cert-provider contains unsupported characters"
validate_path "--cert-file" "$CERT_FILE"
validate_path "--key-file" "$KEY_FILE"
validate_path "--config-dir" "$CONFIG_DIR"
[[ "$CONFIG_DIR" == "/etc/V2bX" || "$CONFIG_DIR" == /etc/V2bX/* ]] \
  || die "--config-dir must stay within /etc/V2bX"

NODE_TYPE="$(printf '%s' "$NODE_TYPE" | tr 'A-Z' 'a-z')"
CORE="$(printf '%s' "$CORE" | tr 'A-Z' 'a-z')"
TCP_FAST_OPEN="$(to_bool "$TCP_FAST_OPEN")"
SNIFF_ENABLED="$(to_bool "$SNIFF_ENABLED")"
CERT_REJECT_UNKNOWN_SNI="$(to_bool "$CERT_REJECT_UNKNOWN_SNI")"
case "$NODE_TYPE" in
  vmess|vless|trojan|shadowsocks|hysteria|hysteria2|tuic|anytls)
    ;;
  *)
    die "Unsupported --node-type: $NODE_TYPE";;
esac

case "$CORE" in
  sing|xray)
    ;;
  *)
    die "Unsupported --core: $CORE (supported: sing, xray)";;
esac

if [[ "$CORE" == "xray" ]]; then
  case "$NODE_TYPE" in
    vmess|vless|trojan|shadowsocks)
      ;;
    *)
      die "Core xray does not support node type: $NODE_TYPE (supported with xray: vmess, vless, trojan, shadowsocks)"
      ;;
  esac
fi

PANEL_HOST="$(printf '%s' "$PANEL_URL" | sed -E 's#^https?://##; s#/.*$##; s#:[0-9]+$##')"

if [[ -z "$CERT_MODE" ]]; then
  case "$NODE_TYPE" in
    shadowsocks)
      CERT_MODE="none"
      ;;
    vmess|vless|trojan|tuic|anytls|hysteria|hysteria2)
      CERT_MODE="self"
      ;;
    *)
      CERT_MODE="none"
      ;;
  esac
fi
CERT_MODE="$(printf '%s' "$CERT_MODE" | tr 'A-Z' 'a-z')"
case "$CERT_MODE" in
  none|self|file|dns|http)
    ;;
  *)
    die "Unsupported --cert-mode: $CERT_MODE (supported: none, self, file, dns, http)"
    ;;
esac

case "$NODE_TYPE" in
  trojan|tuic|anytls|hysteria|hysteria2)
    if [[ "$CERT_MODE" == "none" ]]; then
      echo "[v2bx-install] Notice: $NODE_TYPE requires TLS, auto switch --cert-mode to self"
      CERT_MODE="self"
    fi
    ;;
esac

if [[ -z "$CERT_DOMAIN" ]]; then
  CERT_DOMAIN="$PANEL_HOST"
fi
if [[ -z "$CERT_DOMAIN" ]]; then
  CERT_DOMAIN="localhost"
fi

if [[ $EUID -ne 0 ]]; then
  die "Please run as root"
fi

PANEL_URL="${PANEL_URL%/}"
if [[ -n "$BOOTSTRAP_TOKEN" ]]; then
  exchange_bootstrap_credential
fi

if [[ $SKIP_INSTALL -eq 0 ]]; then
  if ! command -v V2bX >/dev/null 2>&1; then
    echo "[v2bx-install] V2bX not found, installing..."
    command -v curl >/dev/null 2>&1 || die "curl is required to install V2bX"
    command -v sha256sum >/dev/null 2>&1 || die "sha256sum is required to verify V2bX"
    INSTALL_TMP_DIR="$(mktemp -d)"
    INSTALL_SCRIPT="${INSTALL_TMP_DIR}/install.sh"
    INSTALL_PROTOCOL="=https"
    [[ "$INSTALL_URL" == http://* ]] && INSTALL_PROTOCOL="=http"
    if ! curl -fsSL --proto "$INSTALL_PROTOCOL" --proto-redir "$INSTALL_PROTOCOL" \
      --retry 3 --connect-timeout 15 "$INSTALL_URL" -o "$INSTALL_SCRIPT"; then
      rm -rf -- "$INSTALL_TMP_DIR"
      die "failed to download V2bX installer"
    fi
    ACTUAL_INSTALL_SHA256="$(sha256sum "$INSTALL_SCRIPT" | awk '{print $1}')"
    if [[ "${ACTUAL_INSTALL_SHA256,,}" != "${INSTALL_SHA256,,}" ]]; then
      rm -rf -- "$INSTALL_TMP_DIR"
      die "V2bX installer SHA-256 mismatch"
    fi
    chmod 0700 "$INSTALL_SCRIPT"
    if ! bash "$INSTALL_SCRIPT"; then
      rm -rf -- "$INSTALL_TMP_DIR"
      die "V2bX installer failed"
    fi
    rm -rf -- "$INSTALL_TMP_DIR"
  fi
fi

V2BX_BIN="$(command -v V2bX || true)"
[[ -n "$V2BX_BIN" ]] || die "V2bX binary not found after install"
validate_path "V2bX binary" "$V2BX_BIN"

reject_symlink_components "--config-dir" "$CONFIG_DIR"
reject_symlink_components "--cert-file" "$CERT_FILE"
reject_symlink_components "--key-file" "$KEY_FILE"
mkdir -p "$CONFIG_DIR"
reject_symlink_components "--config-dir" "$CONFIG_DIR"
chmod 0700 "$CONFIG_DIR"

ORIGIN_PATH="$CONFIG_DIR/${CORE}_origin.json"
CONFIG_PATH="$CONFIG_DIR/config.json"
validate_path "origin config path" "$ORIGIN_PATH"
validate_path "main config path" "$CONFIG_PATH"

ensure_origin_config() {
  local core="$1"
  local path="$2"
  reject_symlink_components "origin config path" "$path"
  if [[ -e "$path" && ! -f "$path" ]]; then
    die "origin config path must be a regular file: $path"
  fi
  if [[ -s "$path" ]]; then
    chmod 0600 "$path"
    return 0
  fi

  case "$core" in
    sing)
      atomic_write_file "$path" 0600 <<'EOF'
{
  "log": {},
  "dns": {},
  "inbounds": [],
  "outbounds": [
    { "type": "direct", "tag": "direct" },
    { "type": "block", "tag": "block" },
    { "type": "dns", "tag": "dns-out" }
  ],
  "route": {
    "rules": [
      { "protocol": "dns", "outbound": "dns-out" }
    ],
    "final": "direct"
  }
}
EOF
      ;;
    xray)
      atomic_write_file "$path" 0600 <<'EOF'
{
  "log": {
    "loglevel": "warning"
  },
  "inbounds": [],
  "outbounds": [
    { "protocol": "freedom", "tag": "direct" },
    { "protocol": "blackhole", "tag": "block" }
  ],
  "routing": {
    "domainStrategy": "AsIs",
    "rules": []
  }
}
EOF
      ;;
  esac
  echo "[v2bx-install] Origin config initialized: $path"
}

ensure_origin_config "$CORE" "$ORIGIN_PATH"

atomic_write_file "$CONFIG_PATH" 0600 <<EOF
{
  "Log": {
    "Level": "info",
    "Output": ""
  },
  "Cores": [
    {
      "Type": "$CORE",
      "Log": { "Level": "info", "Timestamp": true },
      "NTP": { "Enable": false, "Server": "time.apple.com", "ServerPort": 0 },
      "OriginalPath": "$ORIGIN_PATH"
    }
  ],
  "Nodes": [
    {
      "ApiHost": "$PANEL_URL",
      "ApiKey": "$TOKEN",
      "NodeID": $NODE_ID,
      "NodeType": "$NODE_TYPE",
      "Timeout": 30,
      "Core": "$CORE",
      "ListenIP": "$LISTEN_IP",
      "SendIP": "$SEND_IP",
      "DeviceOnlineMinTraffic": $DEVICE_ONLINE_MIN_TRAFFIC,
      "MinReportTraffic": $MIN_REPORT_TRAFFIC,
      "ReportMinTraffic": $MIN_REPORT_TRAFFIC,
      "TCPFastOpen": $TCP_FAST_OPEN,
      "EnableTFO": $TCP_FAST_OPEN,
      "SniffEnabled": $SNIFF_ENABLED,
      "EnableSniff": $SNIFF_ENABLED,
      "CertConfig": {
        "CertMode": "$CERT_MODE",
        "RejectUnknownSni": $CERT_REJECT_UNKNOWN_SNI,
        "CertDomain": "$CERT_DOMAIN",
        "CertFile": "$CERT_FILE",
        "KeyFile": "$KEY_FILE",
        "Email": "$CERT_EMAIL",
        "Provider": "$CERT_PROVIDER",
        "DNSEnv": {
          "EnvName": "env1"
        }
      }
    }
  ]
}
EOF

echo "[v2bx-install] Config written: $CONFIG_PATH"

if command -v systemctl >/dev/null 2>&1; then
  SERVICE_FILE="/etc/systemd/system/V2bX.service"
  validate_path "service file" "$SERVICE_FILE"
  reject_symlink_components "service file" "$SERVICE_FILE"
  if [[ -e "$SERVICE_FILE" && ! -f "$SERVICE_FILE" ]]; then
    die "service file path must be a regular file: $SERVICE_FILE"
  fi
  if [[ ! -e "$SERVICE_FILE" ]]; then
    atomic_write_file "$SERVICE_FILE" 0644 <<EOF
[Unit]
Description=V2bX Service
After=network.target

[Service]
Type=simple
ExecStart=$V2BX_BIN server --config $CONFIG_DIR/config.json
Restart=on-failure
RestartSec=3s

[Install]
WantedBy=multi-user.target
EOF
    systemctl daemon-reload
  else
    chmod 0644 "$SERVICE_FILE"
  fi
  systemctl enable --now V2bX
  systemctl restart V2bX
  echo "[v2bx-install] V2bX started. Check: systemctl status V2bX -l"
else
  echo "[v2bx-install] systemctl not found. Start manually:"
  echo "  $V2BX_BIN server --config $CONFIG_DIR/config.json"
fi
