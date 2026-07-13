#!/usr/bin/env bash

set -euo pipefail
umask 077

usage() {
  cat <<'EOF'
Rust-first installer for notXboard.

Default behavior:
  1. Start the default Docker base stack: mysql + redis + gateway
  2. Call Rust /bootstrap/full to initialize schema, settings, and admin

Usage:
  sh init.sh --admin-email admin@example.com --admin-password 'ChangeMe123!'

Optional:
  --app-name NAME
  --app-url URL
  --app-port PORT
  --env-file PATH
  --project-name NAME

Examples:
  sh init.sh --admin-email admin@example.com --admin-password 'ChangeMe123!'
  sh init.sh --app-url https://example.com --admin-email admin@example.com --admin-password 'ChangeMe123!'
  sh init.sh --project-name notxboard-init-dev --admin-email admin@example.com --admin-password 'ChangeMe123!'
EOF
}

ADMIN_EMAIL=""
ADMIN_PASSWORD=""
APP_NAME="notXboard"
APP_URL=""
APP_PORT="${APP_PORT:-8000}"
ENV_FILE=".env"
PROJECT_NAME="${COMPOSE_PROJECT_NAME:-}"

while [ $# -gt 0 ]; do
  case "$1" in
    --admin-email)
      ADMIN_EMAIL="${2:-}"
      shift 2
      ;;
    --admin-password)
      ADMIN_PASSWORD="${2:-}"
      shift 2
      ;;
    --app-name)
      APP_NAME="${2:-}"
      shift 2
      ;;
    --app-url)
      APP_URL="${2:-}"
      shift 2
      ;;
    --app-port)
      APP_PORT="${2:-}"
      shift 2
      ;;
    --env-file)
      ENV_FILE="${2:-}"
      shift 2
      ;;
    --project-name)
      PROJECT_NAME="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      echo
      usage
      exit 1
      ;;
  esac
done

if [ ! -f "docker-compose.yml" ]; then
  echo "Please run this script from the project root."
  exit 1
fi

if ! command -v docker >/dev/null 2>&1; then
  echo "Docker is required for the default Rust-first installer."
  exit 1
fi

if ! docker compose version >/dev/null 2>&1; then
  echo "Docker Compose v2 is required."
  exit 1
fi

if [ -z "$ADMIN_EMAIL" ] || [ -z "$ADMIN_PASSWORD" ]; then
  echo "--admin-email and --admin-password are required for Rust-first install."
  echo
  usage
  exit 1
fi

if [ "${#ADMIN_PASSWORD}" -lt 8 ]; then
  echo "admin password must be at least 8 chars."
  exit 1
fi

if [ -L "$ENV_FILE" ]; then
  echo "env file must not be a symbolic link: $ENV_FILE"
  exit 1
fi
if [ -e "$ENV_FILE" ] && [ ! -f "$ENV_FILE" ]; then
  echo "env file path must be a regular file: $ENV_FILE"
  exit 1
fi
if [ ! -f "$ENV_FILE" ] && [ -f ".env.example" ]; then
  cp -- .env.example "$ENV_FILE"
fi
chmod 0600 "$ENV_FILE"

if [ -z "$APP_URL" ]; then
  APP_URL="http://127.0.0.1:${APP_PORT}"
fi

echo "Rust-first install flow"
echo "ENV file: ${ENV_FILE}"
echo "APP URL:  ${APP_URL}"
echo "APP PORT: ${APP_PORT}"
if [ -n "$PROJECT_NAME" ]; then
  echo "PROJECT:  ${PROJECT_NAME}"
fi
echo
echo "[1/2] starting default Docker stack"
COMPOSE_ARGS=()
if [ -n "$PROJECT_NAME" ]; then
  COMPOSE_ARGS+=(-p "$PROJECT_NAME")
fi
APP_ENV_FILE="$ENV_FILE" APP_PORT="$APP_PORT" docker compose "${COMPOSE_ARGS[@]}" up -d mysql redis gateway

echo
echo "[2/2] waiting for /bootstrap/status"
BOOTSTRAP_READY=0
for _ in $(seq 1 30); do
  if curl -fsS "${APP_URL}/bootstrap/status" >/dev/null 2>&1; then
    BOOTSTRAP_READY=1
    break
  fi
  sleep 1
done

if [ "$BOOTSTRAP_READY" != "1" ]; then
  echo "Rust gateway bootstrap endpoint is not ready: ${APP_URL}/bootstrap/status"
  exit 1
fi

BOOTSTRAP_RESPONSE="$(curl -fsS -X POST "${APP_URL}/bootstrap/full" \
  -H 'Content-Type: application/json' \
  --data "{\"app_name\":\"${APP_NAME}\",\"app_url\":\"${APP_URL}\",\"admin_email\":\"${ADMIN_EMAIL}\",\"admin_password\":\"${ADMIN_PASSWORD}\"}")"

echo
echo "Bootstrap response:"
echo "${BOOTSTRAP_RESPONSE}"

SECURE_PATH="$(printf '%s' "${BOOTSTRAP_RESPONSE}" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("secure_path",""))' 2>/dev/null || true)"
if [ -n "$SECURE_PATH" ]; then
  echo
  echo "Admin panel: ${APP_URL}/${SECURE_PATH}"
fi
