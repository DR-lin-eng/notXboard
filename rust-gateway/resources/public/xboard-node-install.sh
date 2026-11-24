#!/usr/bin/env bash
set -euo pipefail
umask 077

SCRIPT_VERSION="2026-11-23"

PANEL_URL=""
NODE_ID=""
MACHINE_ID=""
NODE_TYPE=""
TOKEN=""
BOOTSTRAP_TOKEN=""
BOOTSTRAP_QUERY=""
KERNEL="singbox"
RELEASE_VERSION="v1.13"
INSTALL_URL="https://raw.githubusercontent.com/cedar2025/Xboard-Node/0a29338e1f102a462363ce3527417029f89bab28/install.sh"
INSTALL_SHA256="5473e11af436e0aa3651dd5d465dbdf52eedaf5a1ec6ef90ca3c74016ef26423"

usage() {
  cat <<'HELP'
Usage:
  xboard-node-install.sh --panel <https://panel.example.com> (--node-id <id> | --machine-id <id>) (--bootstrap-token <ticket> --bootstrap-query <query> | --token <token>) [options]

Installs cedar2025/Xboard-Node in node or machine mode and connects it to this panel.

Required:
  --panel           Panel base URL with scheme
  --node-id         Node ID (node mode)
  --machine-id      Machine ID (machine mode, serves every node bound to the machine)
  --bootstrap-token Short-lived installer ticket
  --bootstrap-query Signed installer query returned by the panel
  --token           Node or machine token for an offline/manual install

Options:
  --node-type       Node protocol (informational, validated only)
  --kernel          singbox|xray (default: singbox)
  --version         Xboard-Node release (default: v1.13, or "latest")
  --install-url     Xboard-Node install.sh URL
  --install-sha256  SHA-256 of the Xboard-Node install.sh
  -h, --help        Show this help
HELP
}

die() {
  echo "[xboard-node-install] $*" >&2
  exit 1
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

validate_token() {
  local name="$1"
  local value="$2"
  [[ "$value" =~ ^[A-Za-z0-9._:-]{8,512}$ ]] || die "$name contains unsupported characters"
}

validate_bootstrap_query() {
  local value="$1"
  [[ ${#value} -le 512 ]] || die "--bootstrap-query is too long"
  [[ "$value" =~ ^mb_v=1\&mb_kind=${BOOTSTRAP_KIND}\&mb_exp=[0-9]{10,12}\&mb_asset=[A-Fa-f0-9]{64}\&mb_sig=[A-Fa-f0-9]{64}$ ]] \
    || die "--bootstrap-query is invalid"
}

curl_protocol() {
  if [[ "$1" == http://* ]]; then echo "=http"; else echo "=https"; fi
}

exchange_bootstrap_credential() {
  local credential
  local protocol
  protocol="$(curl_protocol "$PANEL_URL")"
  credential="$(curl -fsS --proto "$protocol" --proto-redir "$protocol" -X POST \
    -H "Authorization: Bearer ${BOOTSTRAP_TOKEN}" \
    "${PANEL_URL}/api/v1/agent/bootstrap/${BOOTSTRAP_KIND}?${BOOTSTRAP_QUERY}")" \
    || die "failed to exchange installer ticket; generate a new install command"
  validate_token "exchanged credential" "$credential"
  TOKEN="$credential"
  BOOTSTRAP_TOKEN=""
  BOOTSTRAP_QUERY=""
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --panel) PANEL_URL="$2"; shift 2;;
    --node-id) NODE_ID="$2"; shift 2;;
    --machine-id) MACHINE_ID="$2"; shift 2;;
    --node-type) NODE_TYPE="$2"; shift 2;;
    --token) TOKEN="$2"; shift 2;;
    --bootstrap-token) BOOTSTRAP_TOKEN="$2"; shift 2;;
    --bootstrap-query) BOOTSTRAP_QUERY="$2"; shift 2;;
    --kernel|--core) KERNEL="$2"; shift 2;;
    --version) RELEASE_VERSION="$2"; shift 2;;
    --install-url) INSTALL_URL="$2"; shift 2;;
    --install-sha256) INSTALL_SHA256="$2"; shift 2;;
    -h|--help) usage; exit 0;;
    *) die "Unknown option: $1";;
  esac
done

[[ "$(id -u)" -eq 0 ]] || die "Please run as root"
command -v curl >/dev/null 2>&1 || die "curl is required"
command -v sha256sum >/dev/null 2>&1 || die "sha256sum is required"
[[ -n "$PANEL_URL" ]] || { usage >&2; die "Missing --panel"; }
if [[ -n "$NODE_ID" && -n "$MACHINE_ID" ]]; then
  die "Use either --node-id or --machine-id, not both"
fi
if [[ -n "$MACHINE_ID" ]]; then
  MODE="machine"
  BOOTSTRAP_KIND="xboard-machine"
  [[ "$MACHINE_ID" =~ ^[0-9]+$ ]] || die "--machine-id must be an unsigned integer"
else
  [[ -n "$NODE_ID" ]] || { usage >&2; die "Missing --node-id or --machine-id"; }
  MODE="node"
  BOOTSTRAP_KIND="xboard-node"
  [[ "$NODE_ID" =~ ^[0-9]+$ ]] || die "--node-id must be an unsigned integer"
fi
if [[ -n "$TOKEN" && ( -n "$BOOTSTRAP_TOKEN" || -n "$BOOTSTRAP_QUERY" ) ]]; then
  die "Use either --token or the bootstrap options, not both"
fi
if [[ -z "$TOKEN" && ( -z "$BOOTSTRAP_TOKEN" || -z "$BOOTSTRAP_QUERY" ) ]]; then
  die "Missing --bootstrap-token/--bootstrap-query"
fi

PANEL_URL="${PANEL_URL%/}"
validate_http_url "--panel" "$PANEL_URL"
validate_http_url "--install-url" "$INSTALL_URL"
[[ "$INSTALL_SHA256" =~ ^[A-Fa-f0-9]{64}$ ]] || die "--install-sha256 must be a 64-character SHA-256"
[[ "$RELEASE_VERSION" =~ ^(latest|v[0-9][0-9A-Za-z._-]*)$ ]] || die "--version is invalid"
KERNEL="$(printf '%s' "$KERNEL" | tr 'A-Z' 'a-z')"
case "$KERNEL" in
  sing|singbox|sing-box) KERNEL="singbox";;
  xray) ;;
  *) die "Unsupported --kernel: $KERNEL";;
esac
NODE_TYPE="$(printf '%s' "$NODE_TYPE" | tr 'A-Z' 'a-z')"
if [[ -n "$NODE_TYPE" ]]; then
  case "$NODE_TYPE" in
    vmess|vless|trojan|shadowsocks|hysteria|hysteria2|tuic|anytls|socks|naive|http|mieru) ;;
    *) die "Unsupported --node-type: $NODE_TYPE";;
  esac
fi
if [[ -n "$TOKEN" ]]; then
  validate_token "--token" "$TOKEN"
else
  [[ "$BOOTSTRAP_TOKEN" =~ ^[A-Fa-f0-9]{64}$ ]] || die "--bootstrap-token is invalid"
  validate_bootstrap_query "$BOOTSTRAP_QUERY"
  exchange_bootstrap_credential
fi

WORK_DIR="$(mktemp -d)"
trap 'rm -rf -- "$WORK_DIR"' EXIT
INSTALLER="$WORK_DIR/install.sh"
INSTALL_PROTOCOL="$(curl_protocol "$INSTALL_URL")"
curl -fsSL --proto "$INSTALL_PROTOCOL" --proto-redir "$INSTALL_PROTOCOL" "$INSTALL_URL" -o "$INSTALLER" \
  || die "failed to download Xboard-Node installer"
echo "${INSTALL_SHA256}  ${INSTALLER}" | sha256sum -c --status - \
  || die "Xboard-Node installer checksum mismatch"

if [[ "$MODE" = "machine" ]]; then
  SUBJECT_ARGS=(--machine-id "$MACHINE_ID")
  SUBJECT_LABEL="machine ${MACHINE_ID}"
else
  SUBJECT_ARGS=(--node-id "$NODE_ID")
  SUBJECT_LABEL="node ${NODE_ID}"
fi
if command -v systemctl >/dev/null 2>&1 && systemctl is-active --quiet V2bX.service; then
  echo "[xboard-node-install] V2bX.service is active. If this is an in-place migration, stop it to release any ports Xboard-Node will reuse." >&2
fi
echo "[xboard-node-install] ${SCRIPT_VERSION}: installing Xboard-Node ${RELEASE_VERSION} (${KERNEL}) for ${SUBJECT_LABEL}"
bash "$INSTALLER" install \
  --mode "$MODE" \
  --panel "$PANEL_URL" \
  --token "$TOKEN" \
  "${SUBJECT_ARGS[@]}" \
  --kernel "$KERNEL" \
  --version "$RELEASE_VERSION" \
  --force-reconfigure \
  --yes
