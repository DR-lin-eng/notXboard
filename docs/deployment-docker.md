# notXboard Docker 部署指南

本指南对应仓库内的 `docker-compose.yml`，当前生产环境默认采用纯 Rust 主运行面：

- 默认 `bridge` 网络（`notxboard`），不使用 `host network`
- 基础服务：`gateway`、`mysql`、`redis`
- 仓库根级 `Dockerfile` 默认构建 Rust 网关镜像
- PHP 兼容镜像使用 `Dockerfile.php-compat`
- 兼容服务已拆成多个 overlay：
  - `docker-compose.compat.yml`：基础 PHP 兼容层
  - `docker-compose.queue.yml`：异步通知 / 批量发信
  - `docker-compose.scheduler.yml`：极少数自定义 PHP 定时任务
- 兼容层按职责拆分：
  - `php` 负责旧页面/旧入口
  - `queue` 负责 Horizon worker
  - `scheduler` 负责 PHP `schedule:work`
- 默认在线流量与默认初始化都不再依赖 PHP
- 默认 Rust gateway 镜像不复制 Laravel `app/`、`config/`、内置 PHP `plugins/` 与 theme source
- 内置 hook、插件 catalog、主题 catalog/template 都由 Rust resources 提供；运行时只保留 public assets 与 `gateway_state`
- 通过健康检查 + `depends_on.condition=service_healthy` 控制启动顺序
- 使用命名卷持久化数据，避免 `storage`/`bootstrap/cache` bind mount 权限陷阱
- `gateway_state` 持久化 Rust 运行时的主题/插件等可写状态
- `php` / `queue` / `scheduler` 仍可作为兼容层运行，但不属于默认拓扑

当前兼容面量化结果见：

- `docs/rust-compat-audit.md`
- `docs/rust-performance-notes.md`
- `docs/rust-verification-checklist.md`
- `python3 tools/route_coverage_audit.py`
- `python3 tools/scheduler_coverage_audit.py`
- `python3 tools/plugin_scheduler_audit.py`
- `python3 tools/cli_surface_audit.py`
- `bash tools/verify_rust_default_stack_isolated.sh`
- `bash tools/verify_rust_default_stack.sh`

Rust 原生数据库备份链还提供 super-admin 手动触发入口：

- `POST /api/v2/{admin_path}/system/runDatabaseBackup`

默认 Rust 栈的 fresh-container 验证优先使用：

```bash
bash tools/verify_rust_default_stack_isolated.sh
```

该脚本会构建 Rust gateway 镜像，并在隔离 Docker network 中创建全新的 MySQL、Redis、gateway 容器，串联验证 `/bootstrap/full`、Rust passport 登录、手动数据库备份和 scheduler heartbeat。
脚本还会确认默认 gateway runtime 中不存在 `/app/runtime/app`、`/app/runtime/config`、`/app/runtime/plugins`、`/app/runtime/theme`，验证 `listHooks` 可在无 Laravel `app/plugins` runtime 下返回核心静态 hook 与插件 hook，并验证 `/app` 可在无 theme source runtime 下渲染内置 `Maintainable` 主题。

## 1. 环境要求

- Docker 20+
- Docker Compose v2+

## 2. 准备 `.env`

```bash
cp .env.example .env
chmod 600 .env
```

至少确认以下生产配置：

```env
APP_ENV=production
APP_DEBUG=false
APP_URL=https://your-domain.com

DB_CONNECTION=mysql
DB_HOST=mysql
DB_PORT=3306
DB_DATABASE=xboard
DB_USERNAME=xboard
DB_PASSWORD=change-me
DB_ROOT_PASSWORD=change-me-root

REDIS_HOST=redis
REDIS_PORT=6379
REDIS_PASSWORD=

QUEUE_CONNECTION=sync
CORE_JOB_SYNC_EXECUTION=true
PHP_HTTP_API_COMPAT=false
PHP_SERVER_INGRESS_COMPAT=false
PHP_WEB_COMPAT=false
CACHE_DRIVER=redis
SESSION_DRIVER=redis
MAIL_DRIVER=log
RUST_GATEWAY_OWNS_SCHEDULER=true
MAIL_SYNC_SEND=true
TELEGRAM_SYNC_SEND=true

# Rust gateway DB pool tuning
DB_MAX_CONNECTIONS=128
DB_MIN_CONNECTIONS=16
DB_ACQUIRE_TIMEOUT_SECS=5
DB_IDLE_TIMEOUT_SECS=300
SETTING_CACHE_TTL_SECS=3
```

`APP_KEY` 需要在首次部署前写入 `.env`。格式建议与 Laravel 保持一致，例如：

```env
APP_KEY=base64:h8KOzHFYUR2mToeLkkAAqw2/Oaibg+YEzOVW0gfAzNo=
```

可使用任意 32 字节随机值再做 base64 编码生成该字段。
如果仅使用 Telegram 机器人通知，建议保持 `MAIL_DRIVER=log`，避免邮件通道配置缺失导致告警任务报错。
上述开关建议保持默认：
- `RUST_GATEWAY_OWNS_SCHEDULER=true`：核心 scheduler 任务由 Rust 网关接管
- `RUST_GATEWAY_OWNS_SCHEDULER=false`：仅用于独立测试、只读网关或显式由其他进程接管 scheduler 的兼容场景；设置后 Rust 网关不会启动后台 scheduler tick
- `MAIL_SYNC_SEND=true`：PHP 旧邮件路径走同步直发，不强依赖 Horizon
- `TELEGRAM_SYNC_SEND=true`：PHP 旧 Telegram 路径走同步直发，不强依赖 Horizon
- `QUEUE_CONNECTION=sync`：默认 Rust 栈下不要求 PHP queue worker；只有叠加 compat compose 时才建议切回 `redis`
- `CORE_JOB_SYNC_EXECUTION=true`：legacy submit / alive / order handle 这类残留 PHP job 在默认栈下同步执行，不默认依赖 Horizon
- `PHP_HTTP_API_COMPAT=false`：默认不加载 PHP API 路由，避免 PHP API 与 Rust API 并存
- `PHP_SERVER_INGRESS_COMPAT=false`：默认不加载 PHP `ServerRoute`，避免兼容 server ingress 与 Rust ingress 并存
- `PHP_WEB_COMPAT=false`：默认不加载 PHP 页面路由，避免 PHP 页面入口与 Rust 页面入口并存

Rust 网关的未命中路由 fallback 默认也不会代理到 PHP；它只会尝试 Rust 侧动态页面补偿，否则返回 Rust 404。需要旧 PHP 入口时必须显式叠加 compat compose。

## 3. 启动默认基础组件

```bash
docker compose up -d mysql redis gateway
docker compose ps
```

此时先确认 `mysql`、`redis` 为 `healthy`。  
`gateway` 在执行 Rust bootstrap 之前会因为缺少基础表而处于 `unhealthy`，这是预期行为。

## 4. 初始化应用

```bash
APP_PORT="${APP_PORT:-8000}"
curl -X POST "http://127.0.0.1:${APP_PORT}/bootstrap/full" \
  -H 'Content-Type: application/json' \
  --data "{\"app_name\":\"notXboard\",\"app_url\":\"http://127.0.0.1:${APP_PORT}\",\"admin_email\":\"admin@example.com\",\"admin_password\":\"ChangeMe123!\"}"
```

返回体会包含 `mode: rust-full-schema` 和生成后的 `secure_path`。管理员账号就是你提交的 `admin_email` / `admin_password`。

注意：

- `gateway` 与 `php` 的健康检查都要求数据库里至少已有 `migrations` 和 `v2_settings` 表。
- 因此首次部署时，必须先完成 Rust bootstrap，不能直接对未初始化数据库做流量切换。
- Rust 网关提供的安装接口：
  - `GET /bootstrap/status`
  - `POST /bootstrap/minimal`
  - `POST /bootstrap/full`
- 当前默认安装路径应使用 `POST /bootstrap/full`。它会应用完整 schema baseline、写入 migration 标记、初始化基础设置、创建管理员账号、写入默认用户组限制、启用保护插件并补齐缺失 API key。
- 如果宿主机 `8000` 端口已被占用，可在启动前改用其他端口，例如：

```bash
APP_PORT=18000 docker compose up -d gateway
```

## 5. 兼容模式（仅在确有需要时启用）

如果你仍需要保留 Laravel Octane、旧插件链，或确实要启用异步通知/批量发信，再按需叠加对应 overlay：

```bash
docker compose -f docker-compose.yml -f docker-compose.compat.yml up -d php
docker compose -f docker-compose.yml -f docker-compose.compat.yml -f docker-compose.queue.yml up -d queue
docker compose -f docker-compose.yml -f docker-compose.compat.yml -f docker-compose.scheduler.yml up -d scheduler
```

更细的启用建议：

- 只需要旧 PHP 页面或个别兼容 API：`php`
- 需要异步通知 / 批量发信：`queue`
- 需要保留极少数自定义 PHP 定时任务：`scheduler`
- `queue` / `scheduler` 仅依赖 `mysql + redis + 代码卷`，不再要求 `php` Web 容器先启动

如果你确实需要恢复旧 PHP API/ingress，再显式设置：

```env
PHP_HTTP_API_COMPAT=true
PHP_SERVER_INGRESS_COMPAT=true
PHP_WEB_COMPAT=true
```

当前建议把 `php`、`queue`、`scheduler` 全部视为兼容服务，而不是基础运行面：

- 默认在线流量入口使用 `gateway + mysql + redis`
- `RUST_GATEWAY_OWNS_SCHEDULER=true` 时，核心数据库备份与核心定时任务已由 Rust 接管
- `MAIL_SYNC_SEND=true`、`TELEGRAM_SYNC_SEND=true` 时，旧 PHP 邮件/Telegram 路径不再强依赖 Horizon
- `CORE_JOB_SYNC_EXECUTION=true` 时，`traffic_fetch` / `stat` / `online_sync` / `order_handle` 这类残留 PHP job 会同步执行
- 当前剩余明确会异步投递的默认面主要是：
  - `SendEmailJob`
  - `SendTelegramJob`
- 因此 `queue` 服务当前更接近“异步通知与批量发信兼容层”，而不是默认业务链的基础组件
 - Rust admin `POST /api/v2/{admin_path}/user/sendMail` 已支持直接批量发信，默认不再要求 PHP queue worker 承接后台群发
 - 因此 `queue` 服务当前更接近“异步通知兼容层”，而不是默认业务链的基础组件
- 当前内置插件调度审计结果为 `dormant`，即没有插件覆写 `schedule()`
- `scheduler` 服务当前更接近“极少数自定义 PHP 定时任务兼容层”，而不是默认核心调度组件
- 若未使用 PHP Horizon 队列链或少量剩余兼容任务，不要叠加 `docker-compose.compat.yml`

## 6. 反向代理（Nginx）

推荐由 Nginx/Caddy 代理到宿主机 `127.0.0.1:8000`：

```nginx
location / {
    proxy_http_version 1.1;
    proxy_set_header Connection "";
    proxy_set_header Host $host;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
    proxy_pass http://127.0.0.1:8000;
}
```

如需改端口，请修改 `.env` 中 `APP_PORT` 并重启 Compose。

## 7. 使用预构建镜像（可选）

如果镜像已发布到仓库（例如 GHCR），可直接拉取：

```bash
DOCKER_IMAGE=ghcr.io/<owner>/<repo>:latest \
docker compose up -d --pull always --no-build
```

如需本地单独构建 PHP 兼容镜像：

```bash
docker build -f Dockerfile.php-compat -t notxboard-php:local .
```

## 8. 更新

- 预构建镜像：

```bash
docker compose pull
docker compose up -d
```

- 本地构建：

```bash
docker compose build --no-cache
docker compose up -d
```

## 常见问题

- `502/504`：检查 `docker compose ps` 中 `gateway` 是否 `healthy`，以及宿主机 `APP_PORT` 是否被占用。
- `queue/scheduler` 未启动：只有在叠加了 `docker-compose.compat.yml` 时这两个服务才会出现；若已叠加，再确认 `mysql/redis/php` 健康检查通过并查看 `docker compose logs queue scheduler`。
- `queue/scheduler` 未启动：只有在叠加了对应 overlay 时这两个服务才会出现；当前它们只依赖 `mysql/redis`，所以优先查看 `docker compose logs queue scheduler mysql redis`。
- 权限报错：当前模型使用命名卷并在启动时自动修复 `storage`/`bootstrap/cache` 权限，通常无需手动 `chown`。
