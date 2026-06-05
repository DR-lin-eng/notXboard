# notXboard 部署指南（非 Docker，PHP 兼容路径）

本指南只保留为兼容路径，默认推荐部署方式已经切到 Rust-first Docker。

优先参考：

- `docs/deployment-docker.md`
- `README.md`

如果你明确需要保留 PHP Octane / Horizon / `schedule:run` 这套旧运行面，再继续阅读本文。

本指南面向 Linux 服务器上的兼容部署，使用 Nginx + PHP 8.2 + MySQL + Redis。

## 1. 环境要求

- Linux 服务器
- PHP 8.2（建议扩展：swoole、redis、bcmath、pcntl、zip、fileinfo、mbstring、pdo_mysql、openssl）
- MySQL 5.7+ / 8.x
- Redis 6+
- Nginx（或 Caddy/Apache）
- Supervisor/systemd（用于托管常驻进程）

## 2. 获取代码

```bash
git clone <your-repo-url> notXboard
cd notXboard
```

## 3. 安装依赖与初始化（兼容模式）

```bash
sh init.sh --php-compat
```

默认 `sh init.sh` 已改为 Rust-first Docker bootstrap。
只有显式传 `--php-compat` 时，才会执行这里的旧 PHP 安装流程。

## 4. 配置 `.env`

安装完成后，确认并补充以下生产配置：

- `APP_URL`（必须）
- `APP_ENV=production`
- `APP_DEBUG=false`
- `APP_KEY`（必须，不能为空）
- `DB_HOST` / `DB_PORT` / `DB_DATABASE` / `DB_USERNAME` / `DB_PASSWORD`
- `REDIS_HOST` / `REDIS_PORT` / `REDIS_PASSWORD`
- `QUEUE_CONNECTION=redis`
- `CACHE_DRIVER=redis`
- `SESSION_DRIVER=redis`
- `MAIL_DRIVER=log`（仅使用 Telegram 机器人通知时推荐）
- `LINUX_DO_CLIENT_ID` / `LINUX_DO_CLIENT_SECRET`（如需 Linux DO 登录）

## 5. 配置 Nginx（Octane 反向代理）

示例配置（按需修改域名与路径）：

```nginx
server {
    listen 80;
    server_name example.com;
    root /path/to/notXboard/public;

    location / {
        try_files $uri @octane;
    }

    location @octane {
        proxy_http_version 1.1;
        proxy_set_header Connection "";
        proxy_set_header Host $host;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_pass http://127.0.0.1:8000;
    }
}
```

如果不使用反向代理，请确保对外暴露 `8000` 端口。

## 6. 运行常驻进程

至少需要启动以下进程（建议使用 Supervisor/systemd 管理）：

```bash
# Octane
php artisan octane:start --host=127.0.0.1 --port=8000

# Horizon
php artisan horizon
```

## 7. Scheduler（必须）

Laravel Scheduler 需要每分钟执行一次：

```bash
* * * * * cd /path/to/notXboard && php artisan schedule:run >> /var/log/notxboard-scheduler.log 2>&1
```

## 8. 更新说明

本项目不提供自动升级流程。如需更新，请自行评估风险并手动处理代码与依赖变更。
