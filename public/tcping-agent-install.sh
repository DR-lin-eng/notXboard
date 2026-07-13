#!/usr/bin/env bash
set -euo pipefail
umask 077

SCRIPT_VERSION="2026-03-11"
GO_MIN_VERSION="1.22.0"
GO_FALLBACK_VERSION="1.24.1"
GO_INSTALL_VERSION=""
GO_INSTALL_SHA256=""
GO_INSTALL_ROOT="/usr/local"
GO_PROFILE_PATH="/etc/profile.d/notx-go.sh"

PANEL_URL=""
TOKEN=""
BOOTSTRAP_TOKEN=""
BOOTSTRAP_QUERY=""
INSTALL_DIR="/opt/notx-tcping-agent"
SRC_DIR=""
BINARY_PATH="/usr/local/bin/notx-tcping-agent"
ENV_FILE="/etc/notx-tcping-agent.env"
SERVICE_NAME="notx-tcping-agent"
SKIP_START=0

usage() {
  cat <<'EOF'
Usage:
  tcping-agent-install.sh --panel <https://panel.example.com> (--bootstrap-token <ticket> --bootstrap-query <query> | --token <agent-token>) [options]

Required:
  --panel        Panel base URL with http(s) scheme
  --bootstrap-token Short-lived installer ticket
  --bootstrap-query Signed installer query returned by the panel
  --token        TCPing agent token for an offline/manual install

Options:
  --install-dir  Working directory (default: /opt/notx-tcping-agent)
  --binary       Binary install path (default: /usr/local/bin/notx-tcping-agent)
  --env-file     Environment file (default: /etc/notx-tcping-agent.env)
  --service-name systemd service name (default: notx-tcping-agent)
  --go-version   Force a specific Go version for auto-install, e.g. 1.24.1
  --go-sha256    Expected SHA-256 for a custom Go archive version
  --skip-start   Only install files, do not start service
  -h, --help     Show this help
EOF
}

die() {
  echo "[tcping-agent-install] $*" >&2
  exit 1
}

log() {
  echo "[tcping-agent-install] $*"
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
  [[ "$value" =~ ^[A-Za-z0-9._:-]{16,512}$ ]] || die "$name contains unsupported characters"
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

validate_service_name() {
  local value="$1"
  [[ "$value" =~ ^[A-Za-z0-9_.@-]{1,128}$ ]] || die "--service-name contains unsupported characters"
}

validate_go_version() {
  local value="$1"
  [[ -z "$value" || "$value" =~ ^[0-9]+(\.[0-9]+){1,2}$ ]] || die "--go-version must look like 1.24.1"
}

validate_optional_sha256() {
  local value="$1"
  [[ -z "$value" || "$value" =~ ^[A-Fa-f0-9]{64}$ ]] \
    || die "--go-sha256 must be a 64-character SHA-256"
}

download_file() {
  local remote="$1"
  local local_path="$2"
  local dir
  local base
  local tmp
  local display_remote
  validate_path "download target" "$local_path"
  reject_symlink_components "download target" "$local_path"
  if [[ -e "$local_path" && ! -f "$local_path" ]]; then
    die "download target must be a regular file: $local_path"
  fi
  dir="$(dirname "$local_path")"
  base="$(basename "$local_path")"
  mkdir -p "$dir"
  reject_symlink_components "download target" "$local_path"
  tmp="$(mktemp "${dir}/.${base}.download.XXXXXX")"
  display_remote="${remote%%\?*}"
  local protocol="=https"
  if [[ "$remote" == http://* ]]; then
    validate_http_url "download URL" "$display_remote"
    protocol="=http"
  elif [[ "$remote" != https://* ]]; then
    die "download URL must use http(s)"
  fi
  log "Downloading ${display_remote}"
  if ! curl -fsSL --proto "$protocol" --proto-redir "$protocol" \
    --retry 3 --retry-delay 1 --connect-timeout 15 "${remote}" -o "${tmp}"; then
    rm -f -- "$tmp"
    die "download failed: $display_remote"
  fi
  if ! chmod 0600 "$tmp" || ! mv -f -- "$tmp" "$local_path"; then
    rm -f -- "$tmp"
    die "failed to replace download target: $local_path"
  fi
}

validate_bootstrap_query() {
  local value="$1"
  [[ ${#value} -le 512 ]] || die "--bootstrap-query is too long"
  [[ "$value" =~ ^mb_v=1\&mb_kind=tcping\&mb_exp=[0-9]{10,12}\&mb_asset=[A-Fa-f0-9]{64}\&mb_sig=[A-Fa-f0-9]{64}$ ]] \
    || die "--bootstrap-query is invalid"
}

exchange_bootstrap_credential() {
  local credential
  local protocol="=https"
  [[ "$PANEL_URL" == http://* ]] && protocol="=http"
  credential="$(curl -fsS --proto "$protocol" --proto-redir "$protocol" -X POST \
    -H "Authorization: Bearer ${BOOTSTRAP_TOKEN}" \
    "${PANEL_URL}/api/v1/agent/bootstrap/tcping?${BOOTSTRAP_QUERY}")" \
    || die "failed to exchange installer ticket; generate a new install command"
  validate_token "exchanged TCPing token" "$credential"
  TOKEN="$credential"
  BOOTSTRAP_TOKEN=""
  BOOTSTRAP_QUERY=""
}

normalize_version() {
  local raw="${1#go}"
  raw="${raw%%[^0-9.]*}"
  printf '%s\n' "${raw}"
}

version_to_parts() {
  local normalized
  normalized="$(normalize_version "$1")"
  local major="0"
  local minor="0"
  local patch="0"
  IFS='.' read -r major minor patch <<<"${normalized}"
  printf '%s %s %s\n' "${major:-0}" "${minor:-0}" "${patch:-0}"
}

version_ge() {
  local left_major left_minor left_patch
  local right_major right_minor right_patch
  read -r left_major left_minor left_patch <<<"$(version_to_parts "$1")"
  read -r right_major right_minor right_patch <<<"$(version_to_parts "$2")"

  if (( left_major != right_major )); then
    (( left_major > right_major ))
    return
  fi
  if (( left_minor != right_minor )); then
    (( left_minor > right_minor ))
    return
  fi
  (( left_patch >= right_patch ))
}

current_go_version() {
  if ! command -v go >/dev/null 2>&1; then
    return 1
  fi

  local version_text=""
  version_text="$(go env GOVERSION 2>/dev/null || true)"
  if [[ -z "${version_text}" ]]; then
    version_text="$(go version 2>/dev/null | awk '{print $3}')"
  fi
  version_text="$(normalize_version "${version_text}")"
  [[ -n "${version_text}" ]] || return 1
  printf '%s\n' "${version_text}"
}

detect_go_arch() {
  case "$(uname -m)" in
    x86_64|amd64)
      printf 'amd64\n'
      ;;
    aarch64|arm64)
      printf 'arm64\n'
      ;;
    armv6l|armv7l)
      printf 'armv6l\n'
      ;;
    ppc64le)
      printf 'ppc64le\n'
      ;;
    s390x)
      printf 's390x\n'
      ;;
    *)
      die "Unsupported CPU architecture for automatic Go install: $(uname -m)"
      ;;
  esac
}

discover_go_version() {
  local latest=""
  local payload=""
  payload="$(curl -fsSL --proto '=https' --proto-redir '=https' --retry 2 --connect-timeout 10 'https://go.dev/dl/?mode=json' 2>/dev/null || true)"
  if [[ -z "${payload}" ]]; then
    return 0
  fi

  if command -v python3 >/dev/null 2>&1; then
    latest="$(
      printf '%s' "${payload}" | python3 -c 'import json, sys
data = json.load(sys.stdin)
print((data[0].get("version", "").removeprefix("go")) if data else "", end="")' 2>/dev/null || true
    )"
  fi

  if [[ -z "${latest}" ]]; then
    latest="$(
      printf '%s' "${payload}" | tr -d '\n' | sed -n 's/.*"version":"go\([0-9][^"]*\)".*/\1/p' | head -n 1
    )"
  fi

  printf '%s\n' "$(normalize_version "${latest}")"
}

resolve_go_archive_sha256() {
  local archive_name="$1"
  if [[ -n "$GO_INSTALL_SHA256" ]]; then
    printf '%s\n' "$GO_INSTALL_SHA256"
    return 0
  fi
  command -v python3 >/dev/null 2>&1 \
    || die "python3 or --go-sha256 is required for secure Go auto-install"
  local payload
  payload="$(curl -fsSL --proto '=https' --proto-redir '=https' --retry 2 --connect-timeout 10 'https://go.dev/dl/?mode=json&include=all')" \
    || die "failed to load the official Go checksum manifest"
  printf '%s' "$payload" | python3 -c 'import json,sys
name=sys.argv[1]
for release in json.load(sys.stdin):
    for item in release.get("files", []):
        if item.get("filename") == name:
            print(item.get("sha256", ""), end="")
            raise SystemExit(0)
raise SystemExit(1)' "$archive_name" \
    || die "official Go checksum not found for $archive_name"
}

install_go() {
  [[ "$(uname -s)" == "Linux" ]] || die "Automatic Go install currently supports Linux only"
  command -v tar >/dev/null 2>&1 || die "tar is required for automatic Go installation"

  local go_arch
  local go_version
  local archive_name
  local download_url
  local tmp_dir
  local archive_path
  local installed_version
  local expected_sha256
  local actual_sha256

  go_arch="$(detect_go_arch)"
  go_version="$(normalize_version "${GO_INSTALL_VERSION}")"
  if [[ -z "${go_version}" ]]; then
    go_version="$(discover_go_version)"
  fi
  if [[ -z "${go_version}" ]]; then
    go_version="${GO_FALLBACK_VERSION}"
    log "Unable to detect latest stable Go release automatically, falling back to Go ${go_version}"
  fi

  archive_name="go${go_version}.linux-${go_arch}.tar.gz"
  download_url="https://go.dev/dl/${archive_name}"
  tmp_dir="$(mktemp -d)"
  archive_path="${tmp_dir}/${archive_name}"

  log "Installing Go ${go_version} for linux-${go_arch}"
  download_file "${download_url}" "${archive_path}"
  command -v sha256sum >/dev/null 2>&1 || die "sha256sum is required to verify Go"
  expected_sha256="$(resolve_go_archive_sha256 "$archive_name")"
  validate_optional_sha256 "$expected_sha256"
  actual_sha256="$(sha256sum "$archive_path" | awk '{print $1}')"
  [[ "${actual_sha256,,}" == "${expected_sha256,,}" ]] \
    || die "Go archive SHA-256 mismatch"
  if ! tar -tzf "$archive_path" | while IFS= read -r entry; do
    [[ "$entry" == go || "$entry" == go/* ]] || exit 1
    [[ "$entry" != *"/../"* && "$entry" != ../* && "$entry" != /* ]] || exit 1
  done; then
    die "Go archive contains unsafe paths"
  fi

  reject_symlink_components "Go installation path" "${GO_INSTALL_ROOT}/go"
  if [[ -e "${GO_INSTALL_ROOT}/go" && ! -d "${GO_INSTALL_ROOT}/go" ]]; then
    die "Go installation path must be a directory"
  fi
  rm -rf "${GO_INSTALL_ROOT}/go"
  tar -C "${GO_INSTALL_ROOT}" -xzf "${archive_path}"
  ln -sf "${GO_INSTALL_ROOT}/go/bin/go" /usr/local/bin/go
  ln -sf "${GO_INSTALL_ROOT}/go/bin/gofmt" /usr/local/bin/gofmt

  atomic_write_file "${GO_PROFILE_PATH}" 0644 <<EOF
export PATH=${GO_INSTALL_ROOT}/go/bin:\$PATH
EOF
  export PATH="${GO_INSTALL_ROOT}/go/bin:${PATH}"

  installed_version="$(current_go_version || true)"
  rm -rf "${tmp_dir}"

  [[ -n "${installed_version}" ]] || die "Go installation completed, but go command is still unavailable"
  version_ge "${installed_version}" "${GO_MIN_VERSION}" || die "Installed Go ${installed_version} is lower than required ${GO_MIN_VERSION}"
  log "Go ${installed_version} is ready"
}

ensure_go() {
  local installed_version=""
  if installed_version="$(current_go_version)"; then
    if version_ge "${installed_version}" "${GO_MIN_VERSION}"; then
      log "Detected Go ${installed_version}"
      return 0
    fi
    log "Detected Go ${installed_version}, but Go ${GO_MIN_VERSION}+ is required. Upgrading automatically."
  else
    log "Go not found. Installing Go ${GO_MIN_VERSION}+ automatically."
  fi

  install_go
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --panel)
      PANEL_URL="$2"; shift 2;;
    --token)
      TOKEN="$2"; shift 2;;
    --bootstrap-token)
      BOOTSTRAP_TOKEN="$2"; shift 2;;
    --bootstrap-query)
      BOOTSTRAP_QUERY="$2"; shift 2;;
    --install-dir)
      INSTALL_DIR="$2"; shift 2;;
    --binary)
      BINARY_PATH="$2"; shift 2;;
    --env-file)
      ENV_FILE="$2"; shift 2;;
    --service-name)
      SERVICE_NAME="$2"; shift 2;;
    --go-version)
      GO_INSTALL_VERSION="$2"; shift 2;;
    --go-sha256)
      GO_INSTALL_SHA256="$2"; shift 2;;
    --skip-start)
      SKIP_START=1; shift;;
    -h|--help)
      usage; exit 0;;
    *)
      die "Unknown argument: $1";;
  esac
done

[[ -n "$PANEL_URL" ]] || die "Missing --panel"
if [[ -n "$TOKEN" && ( -n "$BOOTSTRAP_TOKEN" || -n "$BOOTSTRAP_QUERY" ) ]]; then
  die "Use either --token or the bootstrap options, not both"
fi
if [[ -z "$TOKEN" && ( -z "$BOOTSTRAP_TOKEN" || -z "$BOOTSTRAP_QUERY" ) ]]; then
  die "Missing --bootstrap-token/--bootstrap-query"
fi
validate_http_url "--panel" "$PANEL_URL"
if [[ -n "$TOKEN" ]]; then
  validate_token "--token" "$TOKEN"
else
  [[ "$BOOTSTRAP_TOKEN" =~ ^[A-Fa-f0-9]{64}$ ]] || die "--bootstrap-token is invalid"
  validate_bootstrap_query "$BOOTSTRAP_QUERY"
fi
validate_path "--install-dir" "$INSTALL_DIR"
validate_path "--binary" "$BINARY_PATH"
validate_path "--env-file" "$ENV_FILE"
validate_path "Go installation root" "$GO_INSTALL_ROOT"
validate_path "Go profile path" "$GO_PROFILE_PATH"
validate_service_name "$SERVICE_NAME"
validate_go_version "$GO_INSTALL_VERSION"
validate_optional_sha256 "$GO_INSTALL_SHA256"
[[ "$INSTALL_DIR" == "/opt/notx-tcping-agent" || "$INSTALL_DIR" == /opt/notx-tcping-agent/* ]] \
  || die "--install-dir must stay within /opt/notx-tcping-agent"
[[ "$BINARY_PATH" == "/usr/local/bin/notx-tcping-agent" ]] \
  || die "--binary must be /usr/local/bin/notx-tcping-agent"
[[ "$ENV_FILE" == "/etc/notx-tcping-agent.env" ]] \
  || die "--env-file must be /etc/notx-tcping-agent.env"
[[ "$SERVICE_NAME" =~ ^notx-tcping-agent([@._-][A-Za-z0-9_.@-]+)?$ ]] \
  || die "--service-name must use the notx-tcping-agent prefix"

if [[ $EUID -ne 0 ]]; then
  die "Please run as root"
fi

if ! command -v curl >/dev/null 2>&1; then
  die "curl is required"
fi

PANEL_URL="${PANEL_URL%/}"
SRC_DIR="${INSTALL_DIR}/src"
BUILD_DIR="${INSTALL_DIR}/build"
validate_path "source directory" "$SRC_DIR"
validate_path "build directory" "$BUILD_DIR"

log "Version: ${SCRIPT_VERSION}"
log "Using panel: ${PANEL_URL}"

reject_symlink_components "--install-dir" "$INSTALL_DIR"
reject_symlink_components "source directory" "$SRC_DIR"
reject_symlink_components "build directory" "$BUILD_DIR"
reject_symlink_components "--binary" "$BINARY_PATH"
reject_symlink_components "--env-file" "$ENV_FILE"
reject_symlink_components "Go installation root" "$GO_INSTALL_ROOT"
reject_symlink_components "Go profile path" "$GO_PROFILE_PATH"
mkdir -p "$SRC_DIR" "$BUILD_DIR" "$(dirname "$BINARY_PATH")" "$(dirname "$ENV_FILE")"
reject_symlink_components "--install-dir" "$INSTALL_DIR"
reject_symlink_components "source directory" "$SRC_DIR"
reject_symlink_components "build directory" "$BUILD_DIR"
reject_symlink_components "--binary" "$BINARY_PATH"
reject_symlink_components "--env-file" "$ENV_FILE"
chmod 0700 "$INSTALL_DIR" "$SRC_DIR" "$BUILD_DIR"

SOURCE_QUERY=""
if [[ -n "$BOOTSTRAP_QUERY" ]]; then
  SOURCE_QUERY="?${BOOTSTRAP_QUERY}"
fi
download_file "${PANEL_URL}/tcping-agent-src/go.mod${SOURCE_QUERY}" "${SRC_DIR}/go.mod"
download_file "${PANEL_URL}/tcping-agent-src/main.go${SOURCE_QUERY}" "${SRC_DIR}/main.go"
if [[ -n "$BOOTSTRAP_TOKEN" ]]; then
  exchange_bootstrap_credential
fi
ensure_go

pushd "$SRC_DIR" >/dev/null
BINARY_DIR="$(dirname "$BINARY_PATH")"
BINARY_BASE="$(basename "$BINARY_PATH")"
BINARY_TMP="$(mktemp "${BINARY_DIR}/.${BINARY_BASE}.tmp.XXXXXX")"
if ! CGO_ENABLED=0 go build -trimpath -ldflags "-s -w -X main.agentVersion=${SCRIPT_VERSION}" -o "$BINARY_TMP" .; then
  rm -f -- "$BINARY_TMP"
  popd >/dev/null
  die "failed to build TCPing agent"
fi
popd >/dev/null
chmod 0755 "$BINARY_TMP"
reject_symlink_components "--binary" "$BINARY_PATH"
if [[ -e "$BINARY_PATH" && ! -f "$BINARY_PATH" ]]; then
  rm -f -- "$BINARY_TMP"
  die "binary target must be a regular file: $BINARY_PATH"
fi
mv -f -- "$BINARY_TMP" "$BINARY_PATH"

atomic_write_file "$ENV_FILE" 0600 <<EOF
PANEL_URL=${PANEL_URL}
TCPING_AGENT_TOKEN=${TOKEN}
EOF

SERVICE_FILE="/etc/systemd/system/${SERVICE_NAME}.service"
validate_path "service file" "$SERVICE_FILE"
reject_symlink_components "service file" "$SERVICE_FILE"
atomic_write_file "$SERVICE_FILE" 0644 <<EOF
[Unit]
Description=notXboard TCPing Agent
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
EnvironmentFile=${ENV_FILE}
ExecStart=${BINARY_PATH}
Restart=always
RestartSec=5
User=root
WorkingDirectory=${INSTALL_DIR}

[Install]
WantedBy=multi-user.target
EOF

echo "[tcping-agent-install] Binary installed at ${BINARY_PATH}"
echo "[tcping-agent-install] Environment file written to ${ENV_FILE}"
echo "[tcping-agent-install] Service file written to ${SERVICE_FILE}"

if [[ $SKIP_START -eq 1 ]]; then
  echo "[tcping-agent-install] Installation completed. Start later with:"
  echo "  systemctl daemon-reload && systemctl enable --now ${SERVICE_NAME}"
  exit 0
fi

if command -v systemctl >/dev/null 2>&1 && [[ -d /run/systemd/system ]]; then
  systemctl daemon-reload
  systemctl enable --now "${SERVICE_NAME}"
  systemctl --no-pager --full status "${SERVICE_NAME}" || true
  echo "[tcping-agent-install] Service started: ${SERVICE_NAME}"
else
  echo "[tcping-agent-install] systemd not found. Run manually:"
  echo "  Load PANEL_URL and TCPING_AGENT_TOKEN from ${ENV_FILE}, then run ${BINARY_PATH}"
fi
