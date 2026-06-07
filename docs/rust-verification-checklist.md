# Rust Verification Checklist

本清单只记录当前最有价值、最缺证据的运行态验证项。

## 1. Fresh Container Default Stack

目标：

- 验证默认 `gateway + mysql + redis` 栈能完成 Rust-only 启动与初始化
- 验证最新 schema / bootstrap / scheduler / maintenance / backup 接口都能在 fresh container 中运行

执行脚本：

```bash
bash tools/verify_rust_default_stack_isolated.sh
```

应重点核对：

- `/bootstrap/full` 成功
- `bootstrap_mode=rust-full-schema`
- scheduler 下一次 tick 正常完成
- 手动备份入口 `POST /api/v2/{admin_path}/system/runDatabaseBackup` 成功
- 备份产物真实存在于容器内

当前已有证据：

- `RUN_ID=0531h HOST_PORT=18088 CLEANUP=1 CLEANUP_IMAGE=1 bash tools/verify_rust_default_stack_isolated.sh` 已通过
- 默认 Rust gateway 容器中不存在 `/app/runtime/app`、`/app/runtime/config`、`/app/runtime/plugins`、`/app/runtime/theme`
- 默认 Rust gateway 容器中不存在任何 `/app/runtime/**/*.php` 或 `/app/runtime/**/*.blade.php` 文件
- fresh MySQL/Redis/gateway 容器中 `/bootstrap/full` 返回 `mode=rust-full-schema`
- Rust passport 登录成功并返回 bearer auth
- Rust `listHooks` 在无 Laravel `app/plugins` runtime 下返回核心静态 hook 与插件 hook
- Rust `/` 公共概览页与 `/app` 内置 `Maintainable` 主题页均可渲染
- Rust 手动备份返回 `/app/state/backup/2026-05-31_14-31-47_xboard_database_backup.sql.gz`
- Redis scheduler heartbeat 写入 `notxboard_database_notxboard_cacheSCHEDULE_LAST_CHECK_AT`
- Rust gateway 日志出现连续 `background scheduler tick completed`

## 2. Node Hot Path Pressure Test

目标：

- 对比 Rust 侧当前热路径优化前后，验证数据库往返减少是否带来真实吞吐提升

重点接口：

- `authenticate_node()`
- `GET /api/v1/server/UniProxy/user`
- `GET /api/v1/server/UniProxy/alivelist`
- `POST /api/v1/server/UniProxy/push`
- `POST /api/v1/server/UniProxy/alive`

建议观察项：

- P50 / P95 / P99 latency
- RPS / QPS
- MySQL CPU 与慢查询
- Rust 进程 CPU / RSS
- 数据库连接池占用
- 单请求 SQL 次数

建议补一组协议对照：

- 全量 warm
- 全量 cold
- `msgpack` warm
- `msgpack` cold
- `delta` no-change
- `delta` with-change

建议脚本：

```bash
bash tools/bench_uniproxy_user_formats.sh
bash tools/bench_uniproxy_user_delta.sh
```

其中 `delta` 组应同时记录：

- 响应体字节数
- `304` 命中率
- `delta` 负载大小
- 用户变更数与 `upserts/removed_ids` 数量

## 3. Backup Runtime Validation

目标：

- 验证 Rust 原生数据库备份链不只编译通过，而且在容器里可真实跑通

当前已有证据：

- `sh init.sh --project-name notxboard-init-verify --admin-email rustinit@example.com --admin-password Passw0rd!2026` 已在隔离 Compose project 上成功拉起 `mysql + redis + gateway` 并返回 `mode=rust-full-schema`
- 镜像内 `mysqldump` 可执行
- `--no-tablespaces` 后最小 schema dump 成功
- `tools/verify_rust_default_stack_isolated.sh` 已在 fresh container 中成功触发 `runDatabaseBackup`
- 本地备份文件已验证存在于 gateway 容器内

仍需补强：

- 如果配置 GCS：
  - OAuth token 获取成功
  - 上传成功
  - `uploaded_object` 返回正确

## 4. Remaining PHP Surface Sanity Check

当前预期：

- HTTP route coverage: `320/320`
- core scheduler coverage: `16/16`
- plugin schedule overrides: `0/7`
- CLI covered: `22/22`
- only PHP-only artisan command left: none

执行：

```bash
python3 tools/route_coverage_audit.py
python3 tools/scheduler_coverage_audit.py
python3 tools/plugin_scheduler_audit.py
python3 tools/cli_surface_audit.py
```
