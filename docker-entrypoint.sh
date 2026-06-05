#!/usr/bin/env bash
set -Eeuo pipefail

APP_DIR=/var/www/html
APP_USER=${APP_RUNTIME_USER:-www-data}
APP_GROUP=${APP_RUNTIME_GROUP:-www-data}

prepare_runtime_dirs() {
  mkdir -p \
    "${APP_DIR}/storage" \
    "${APP_DIR}/bootstrap/cache" \
    "${APP_DIR}/storage/app/public" \
    "${APP_DIR}/storage/framework/cache" \
    "${APP_DIR}/storage/framework/sessions" \
    "${APP_DIR}/storage/framework/views" \
    "${APP_DIR}/storage/logs"
}

clear_bootstrap_cache() {
  find "${APP_DIR}/bootstrap/cache" -maxdepth 1 -type f ! -name '.gitignore' -delete
}

fix_permissions_if_root() {
  if [ "$(id -u)" -ne 0 ]; then
    return 0
  fi

  chown -R "${APP_USER}:${APP_GROUP}" "${APP_DIR}/storage" "${APP_DIR}/bootstrap/cache"
  chmod -R ug+rwX "${APP_DIR}/storage" "${APP_DIR}/bootstrap/cache"
}

run_laravel_bootstrap() {
  if [ ! -f "${APP_DIR}/artisan" ]; then
    return 0
  fi

  if [ "$(id -u)" -eq 0 ]; then
    gosu "${APP_USER}:${APP_GROUP}" php artisan storage:link >/dev/null 2>&1 || true
    return 0
  fi

  php artisan storage:link >/dev/null 2>&1 || true
}

should_drop_privileges() {
  if [ "$(id -u)" -ne 0 ]; then
    return 1
  fi

  if [ "${APP_FORCE_ROOT:-0}" = "1" ]; then
    return 1
  fi

  if [ "$#" -lt 3 ]; then
    return 1
  fi

  if [ "$1" = "php" ] && [ "$2" = "artisan" ]; then
    case "$3" in
      octane:start|horizon|schedule:work)
        return 0
        ;;
    esac
  fi

  return 1
}

cd "${APP_DIR}"
prepare_runtime_dirs
fix_permissions_if_root
clear_bootstrap_cache
run_laravel_bootstrap

if should_drop_privileges "$@"; then
  exec gosu "${APP_USER}:${APP_GROUP}" docker-php-entrypoint "$@"
fi

exec docker-php-entrypoint "$@"
