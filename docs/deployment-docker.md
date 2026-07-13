# notXboard Docker 部署指南

本指南对应仓库内的 `docker-compose.yml`，当前生产环境默认采用纯 Rust 主运行面：

- 默认 `bridge` 网络（`notxboard`），不使用 `host network`
- 基础服务：`gateway`、`mysql`、`redis`
- 默认 `gateway` 构建上下文为 `rust-gateway/`
- 仓库根级 `Dockerfile` 也可单独构建 Rust 网关镜像
- 默认在线流量与默认初始化都不再依赖 PHP
- 默认 Rust gateway 镜像不复制 Laravel `app/`、`config/`、内置 PHP `plugins/`、theme source、仓库根级 `resources/` 与 `public/`
- 内置 hook、插件 catalog、主题 catalog/template 都由 Rust resources 提供；运行时只保留 public assets 与 `gateway_state`
- 通过健康检查 + `depends_on.condition=service_healthy` 控制启动顺序
- 使用命名卷持久化数据，避免 `storage`/`bootstrap/cache` bind mount 权限陷阱
- `gateway_state` 持久化 Rust 运行时的主题/插件等可写状态

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

CACHE_DRIVER=redis
SESSION_DRIVER=redis
MAIL_DRIVER=log
RUST_GATEWAY_OWNS_SCHEDULER=true

# Rust gateway DB pool tuning
DB_MAX_CONNECTIONS=128
DB_MIN_CONNECTIONS=16
DB_ACQUIRE_TIMEOUT_SECS=5
DB_IDLE_TIMEOUT_SECS=300
SETTING_CACHE_TTL_SECS=3

# Public exposure controls
PUBLIC_SITE_MODE=minimal
PUBLIC_CATALOG_ENABLED=false
WEB_ACCESS_USERNAME=
WEB_ACCESS_PASSWORD=
```

### 降低公开暴露面

默认配置不会公开运营仪表盘或套餐目录：

- `PUBLIC_SITE_MODE=minimal`：根路径只显示通用授权入口，不加载统计、排行或地图。
- `PUBLIC_SITE_MODE=disabled`：根路径直接返回通用 `404`。
- `PUBLIC_SITE_MODE=dashboard`：显式恢复公开运营仪表盘及其三个公开数据接口。
- `PUBLIC_CATALOG_ENABLED=false`：公开套餐目录返回 `404`；确需访客浏览套餐时再设为 `true`。

生产环境建议同时配置页面级访问认证：

```env
WEB_ACCESS_USERNAME=your-private-user
WEB_ACCESS_PASSWORD=use-a-unique-random-password
```

两个变量必须同时设置。启用后，首页、用户门户、管理页面、主题资源、公开统计和公开套餐目录都需要标准 HTTP Basic 认证；认证后的响应强制使用 `private, no-store`，不会进入共享缓存。Bearer 业务 API、订阅链接、健康检查以及安装器资源不受 Basic 认证影响；安装器资源使用绑定 owner、subject 和资源 kind 的短期签名 query 单独保护。Basic 凭据和安装 query 都必须只通过 HTTPS 使用，建议使用至少 20 位的唯一随机密码，也可以在反向代理层改用 SSO、mTLS 或受管访问策略。

`robots.txt` 与 `X-Robots-Tag` 只负责阻止正常搜索引擎收录，不是访问控制。任何配置都无法保证域名不会被第三方分类或封禁；可靠做法是减少未认证内容、使用真实访问认证，并避免依赖文案替换、代码混淆或路径轮换。

`APP_KEY` 需要在首次部署前写入 `.env`。格式建议与 Laravel 保持一致，例如：

```env
APP_KEY=base64:h8KOzHFYUR2mToeLkkAAqw2/Oaibg+YEzOVW0gfAzNo=
```

可使用任意 32 字节随机值再做 base64 编码生成该字段。
如果仅使用 Telegram 机器人通知，建议保持 `MAIL_DRIVER=log`，避免邮件通道配置缺失导致告警任务报错。
上述开关建议保持默认：
- `RUST_GATEWAY_OWNS_SCHEDULER=true`：核心 scheduler 任务由 Rust 网关接管
- `RUST_GATEWAY_OWNS_SCHEDULER=false`：仅用于独立测试、只读网关或显式由其他进程接管 scheduler 的兼容场景；设置后 Rust 网关不会启动后台 scheduler tick

Rust 网关的未命中路由 fallback 默认不会代理到 PHP；它只会尝试 Rust 侧动态页面补偿，否则返回 Rust 404。

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
: "${BOOTSTRAP_TOKEN:?set BOOTSTRAP_TOKEN in .env first}"
curl -X POST "http://127.0.0.1:${APP_PORT}/bootstrap/full" \
  -H 'Content-Type: application/json' \
  -H "X-Bootstrap-Token: ${BOOTSTRAP_TOKEN}" \
  --data "{\"app_name\":\"notXboard\",\"app_url\":\"http://127.0.0.1:${APP_PORT}\",\"admin_email\":\"admin@example.com\",\"admin_password\":\"ChangeMe123!\"}"
```

返回体会包含 `mode: rust-full-schema` 和生成后的 `secure_path`。管理员账号就是你提交的 `admin_email` / `admin_password`。

注意：

- `gateway` 的健康检查要求数据库里至少已有 `migrations` 和 `v2_settings` 表。
- 因此首次部署时，必须先完成 Rust bootstrap，不能直接对未初始化数据库做流量切换。
- Rust 网关提供的安装接口：
  - `GET /bootstrap/status`
  - `POST /bootstrap/minimal`
  - `POST /bootstrap/full`
- 当前默认安装路径应使用 `POST /bootstrap/full`。它会应用完整 schema baseline、写入 migration 标记、初始化基础设置、创建管理员账号、写入默认用户组限制、启用保护插件并补齐缺失 API key。
- 两个写入型安装接口都要求 `X-Bootstrap-Token` 与环境变量 `BOOTSTRAP_TOKEN` 恒定时间匹配；令牌至少 24 字符，建议使用 `openssl rand -hex 32` 生成。数据库行锁保证并发初始化最多只有一个请求进入创建管理员的临界区。
- 如果宿主机 `8000` 端口已被占用，可在启动前改用其他端口，例如：

```bash
APP_PORT=18000 docker compose up -d gateway
```

## 5. 反向代理（Nginx）

推荐由 Nginx/Caddy 代理到宿主机 `127.0.0.1:8000`。先在 `nginx.conf` 的 `http {}` 中定义不带 query 的日志格式：

```nginx
log_format notxboard '$remote_addr - $remote_user [$time_local] '
                     '"$request_method $uri $server_protocol" $status $body_bytes_sent '
                     '"$http_referer" "$http_user_agent"';
```

再在站点的 `server {}` 中使用该格式并配置代理：

```nginx
access_log /var/log/nginx/notxboard.access.log notxboard;

location / {
    proxy_http_version 1.1;
    proxy_set_header Connection "";
    proxy_set_header Host $host;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
    proxy_pass http://127.0.0.1:8000;
}
```

不要在 Nginx、CDN、WAF、APM 或错误追踪中记录 `$request`、`$request_uri`、`$args` 或完整 URL。为兼容 UniProxy 和旧节点协议，部分机器凭据仍位于 query string；记录完整请求会把这些凭据持久化到日志系统。上述格式只记录 `$uri`，不记录 query。如果上游产品无法关闭完整 URL 采集，必须先配置 query 脱敏或对这些机器路径禁用采集。

如需改端口，请修改 `.env` 中 `APP_PORT` 并重启 Compose。

## 6. 使用预构建镜像（可选）

仓库的 `Build and Publish` Action 会在推送到默认分支、推送 `v*` 标签或手动触发时发布
`linux/amd64` 与 `linux/arm64` 镜像到 GHCR。可直接切换到预构建镜像：

```bash
GATEWAY_IMAGE=ghcr.io/<owner>/<repo>:latest \
docker compose -f deploy/compose.ghcr.yml up -d mysql redis gateway
```

`deploy/compose.ghcr.yml` 只使用预构建镜像，不包含本地 `build` 配置；默认镜像为
`ghcr.io/dr-lin-eng/notxboard:latest`。Fork 仓库使用时通过 `GATEWAY_IMAGE` 替换镜像地址。
GHCR 包首次发布后通常是私有状态：公开部署时需在 GitHub Packages 设置中将该包改为 Public；
保持私有时，部署主机必须先使用具备 `read:packages` 权限的令牌执行 `docker login ghcr.io`。

Action 还会生成 `notxboard-<version>-linux-amd64.tar.gz`。该压缩包包含网关二进制、运行资源、
启动脚本和环境变量模板，适合不使用 Docker 的 Linux AMD64 主机。解压后配置环境变量并运行：

```bash
./run.sh
```

## 7. 更新

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
- 权限报错：当前模型使用命名卷并在启动时自动修复 `storage`/`bootstrap/cache` 权限，通常无需手动 `chown`。
