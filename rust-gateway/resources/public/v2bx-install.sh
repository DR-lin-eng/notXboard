#!/usr/bin/env bash
set -euo pipefail

SCRIPT_VERSION="2026-01-28"

PANEL_URL=""
NODE_ID=""
NODE_TYPE=""
TOKEN=""
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
INSTALL_URL="https://raw.githubusercontent.com/wyx2685/V2bX-script/master/install.sh"
SKIP_INSTALL=0

usage() {
  cat <<'EOF'
Usage:
  v2bx-install.sh --panel <https://panel.example.com> --node-id <id> --node-type <type> --token <token> [options]

Required:
  --panel        Panel base URL with scheme
  --node-id      Node ID
  --node-type    vmess|vless|trojan|shadowsocks|hysteria|hysteria2|tuic|anytls
  --token        V2bX token

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
}

validate_json_value() {
  local name="$1"
  local value="$2"
  if has_unsafe_text "$value"; then
    die "$name contains unsafe JSON characters"
  fi
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
[[ -n "$TOKEN" ]] || die "Missing --token"

validate_http_url "--panel" "$PANEL_URL"
validate_http_url "--install-url" "$INSTALL_URL"
validate_uint "--node-id" "$NODE_ID"
validate_uint "--device-online-min-traffic" "$DEVICE_ONLINE_MIN_TRAFFIC"
validate_uint "--min-report-traffic" "$MIN_REPORT_TRAFFIC"
validate_token "--token" "$TOKEN"
validate_json_value "--listen-ip" "$LISTEN_IP"
validate_json_value "--send-ip" "$SEND_IP"
validate_json_value "--cert-domain" "$CERT_DOMAIN"
validate_json_value "--cert-email" "$CERT_EMAIL"
[[ "$CERT_PROVIDER" =~ ^[A-Za-z0-9_-]+$ ]] || die "--cert-provider contains unsupported characters"
validate_path "--cert-file" "$CERT_FILE"
validate_path "--key-file" "$KEY_FILE"
validate_path "--config-dir" "$CONFIG_DIR"

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

if [[ $SKIP_INSTALL -eq 0 ]]; then
  if ! command -v V2bX >/dev/null 2>&1; then
    echo "[v2bx-install] V2bX not found, installing..."
    if command -v curl >/dev/null 2>&1; then
      bash <(curl -fsSL "$INSTALL_URL")
    elif command -v wget >/dev/null 2>&1; then
      wget -qO- "$INSTALL_URL" | bash
    else
      die "curl or wget is required to install V2bX"
    fi
  fi
fi

V2BX_BIN="$(command -v V2bX || true)"
[[ -n "$V2BX_BIN" ]] || die "V2bX binary not found after install"

mkdir -p "$CONFIG_DIR"

ORIGIN_PATH="$CONFIG_DIR/${CORE}_origin.json"

ensure_origin_config() {
  local core="$1"
  local path="$2"
  if [[ -s "$path" ]]; then
    return 0
  fi

  case "$core" in
    sing)
      cat > "$path" <<'EOF'
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
      cat > "$path" <<'EOF'
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

cat > "$CONFIG_DIR/config.json" <<EOF
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

echo "[v2bx-install] Config written: $CONFIG_DIR/config.json"

if command -v systemctl >/dev/null 2>&1; then
  SERVICE_FILE="/etc/systemd/system/V2bX.service"
  if [[ ! -f "$SERVICE_FILE" ]]; then
    cat > "$SERVICE_FILE" <<EOF
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
  fi
  systemctl enable --now V2bX
  systemctl restart V2bX
  echo "[v2bx-install] V2bX started. Check: systemctl status V2bX -l"
else
  echo "[v2bx-install] systemctl not found. Start manually:"
  echo "  $V2BX_BIN server --config $CONFIG_DIR/config.json"
fi
