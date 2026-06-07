# Rust Compatibility Audit

本文件用于量化 notXboard 从 PHP 底层向 Rust 底层迁移后的剩余兼容面。

## 1. HTTP Route Coverage

审计脚本：

```bash
python3 tools/route_coverage_audit.py
```

当前结果：

- PHP 路由声明：`320`
- Rust 路由注册：`512`
- 已覆盖 PHP 路由：`320 / 320`
- HTTP 路由缺口：`0`

结论：

- 当前 legacy HTTP 路由基线已固化到 `tools/compat_baselines/php_routes.json.gz.b64`，并已全部映射到 Rust 网关。
- Rust 网关已不再依赖 PHP 路由 fallback 才能提供现有 HTTP 接口。
- Rust router fallback 已拆到 `rust-gateway/src/fallback_support.rs`，默认只处理 Rust 动态页面补偿或返回 Rust 404，不再保留隐式 `proxy_to_php` 语义。

## 2. Core Scheduler Coverage

审计脚本：

```bash
python3 tools/scheduler_coverage_audit.py
```

当前 scheduler 基线来源：

- `tools/compat_baselines/scheduler_commands.json`

当前结果：

- legacy 核心调度基线命令：`16`
- Rust 已覆盖：`16`
- 核心调度缺口：`0`

说明：

- 该审计优先读取 `tools/compat_baselines/scheduler_commands.json` 中固化的核心调度基线。
- `PluginManager::registerPluginSchedules($schedule)` 注册的插件调度仍属于单独兼容面，未在此统计中展开。
- 新增的 Rust 原生数据库备份链位于 `rust-gateway/src/backup_support.rs`。
- 手动触发入口位于 Rust admin system API：
  - `POST /api/v2/{admin_path}/system/runDatabaseBackup`
- 当前已完成：
  - Rust 网关镜像编译通过
  - 镜像内 `mysqldump` 可执行性取证
  - 基于 `--no-tablespaces` 的最小 MySQL schema dump 运行取证
  - 已提供 `tools/verify_rust_default_stack.sh`，用于串联 bootstrap、admin 登录、手动备份入口与 scheduler 日志检查
  - 已提供 `tools/verify_rust_default_stack_isolated.sh`，用于在全新 Docker network/container 中验证默认 `gateway + mysql + redis` 栈
  - 2026-05-31 隔离运行证据：
    - `RUN_ID=0531h HOST_PORT=18088 CLEANUP=1 CLEANUP_IMAGE=1 bash tools/verify_rust_default_stack_isolated.sh`
    - 默认 Rust gateway 镜像内不存在 `/app/runtime/app`、`/app/runtime/config`、`/app/runtime/plugins`、`/app/runtime/theme`
    - 默认 Rust gateway 镜像内不存在任何 `/app/runtime/**/*.php` 或 `/app/runtime/**/*.blade.php` 文件
    - Rust 内置主题 metadata/template 由 `rust-gateway/resources/themes/*` 编译进网关，默认只保留运行所需 public assets
    - `/bootstrap/full` 返回 `mode=rust-full-schema`
    - Rust passport 登录成功
    - Rust `listHooks` 在无 Laravel `app/plugins` runtime 下返回核心静态 hook 与插件 hook
    - Rust `/` 公共概览页与 `/app` 内置 `Maintainable` 主题页均可在无 theme source runtime 下渲染
    - Rust 手动备份返回 `/app/state/backup/2026-05-31_14-31-47_xboard_database_backup.sql.gz`
    - Redis heartbeat `notxboard_database_notxboard_cacheSCHEDULE_LAST_CHECK_AT=1780209166`
    - Rust gateway 日志出现连续 `background scheduler tick completed`
- 仍待补强：
  - GCS 上传链路的运行态验证

## 3. Current Default Stack Status

结合以上两项审计，当前状态可以概括为：

- 默认 HTTP 运行面：Rust
- 默认数据库初始化：Rust `/bootstrap/full`
- 默认 Docker 基础栈：`gateway + mysql + redis`
- 默认 `docker build .` 入口：Rust 网关镜像
- 默认 `docker compose` 的 `gateway` 构建上下文：`rust-gateway/`
- 默认 Rust gateway 镜像不再复制 Laravel `app/`、`config/`、内置 PHP `plugins/`、theme source、仓库根级 `resources/` 与 `public/`
- 默认内置主题 catalog/template 与运行时静态资源都由 `rust-gateway/resources/**` 提供，运行时只保留 `/app/runtime/**` 与 `/app/state/**`
- 内置插件调度审计：
  - `python3 tools/plugin_scheduler_audit.py`
  - 当前结果：`7` 个内置插件中，`0` 个覆写 `schedule()`，兼容面状态为 `dormant`
- PHP 兼容 profile 主要仍用于：
  - 少量尚未迁移的非 HTTP / 非核心 scheduler 兼容面
  - 将来可能新增的插件自定义调度

## 4. PHP CLI Surface

审计脚本：

```bash
python3 tools/cli_surface_audit.py
```

当前 CLI 基线来源：

- `tools/compat_baselines/cli_commands.json`

当前结果：

- PHP artisan 命令总数：`22`
- 已有 Rust 等效能力：`22`
- 仍然 PHP-only 的命令：`0`

当前仍为 PHP-only 的手工命令：无

结论：

- 当前 PHP artisan 命令面已全部具备 Rust runtime/API/scheduler 等效能力。
- 审计优先读取 `tools/compat_baselines/cli_commands.json` 中固化的 CLI 基线。
- 默认在线请求链路、默认数据库初始化链路、核心定时任务链路都已不再依赖 PHP artisan 命令。
- 已新增的 Rust maintenance 入口包括：
  - `GET /api/v2/{admin_path}/system/exportLogsCsv`
  - `GET /api/v2/{admin_path}/system/listHooks`
  - `POST /api/v2/{admin_path}/system/cleanupDormantUsers`
  - `POST /api/v2/{admin_path}/system/resetAllUserSecurity`
  - `POST /api/v2/{admin_path}/system/resetUserPassword`

## 5. Next Migration Priority

按当前审计结果，下一阶段最高优先级是：

1. 把已 service 化的 legacy traffic / alive / order 路径继续推进到 Rust 等效实现
   - `LegacyTrafficDispatchService`
   - `OrderService::handleTradeNo(...)`
2. 明确默认 legacy server 流量入口只应走 Rust router
   - `ShadowsocksTidalab`
   - `TrojanTidalab`
   - `UniProxy`
3. 量化并迁移插件调度兼容面

## 6. Current Queue Compatibility Strategy

当前默认 Rust-first 部署口径已经进一步收紧为：

- `.env.example`
  - 不再暴露默认 PHP compat 路由、页面和同步执行开关
- `docker-compose.yml`
  - 默认仅保留 `gateway + mysql + redis`
  - `gateway` 构建上下文已收缩到 `rust-gateway/`
- Rust 默认镜像
  - 仅复制 `rust-gateway/resources/**`
  - 不再复制仓库根级 `resources/`、`public/`、`theme/portal/assets`

含义：

- 默认栈下，PHP 兼容部署层已经不再作为受支持运行方式提供
- 默认栈下，PHP API / Web compat 开关也不再作为默认部署入口暴露

当前默认路径已经进一步收口：

- 订单链：
  - `OrderService::handleTradeNo(...)` 已成为默认执行入口
  - `OrderHandleJob` 仅保留为 compat 包装
- legacy submit / stat / alive：
  - 已抽离到 `LegacyTrafficDispatchService`
  - 默认路径会优先直接调用 service
  - `TrafficFetchJob` / `StatUserJob` / `StatServerJob` / `UpdateAliveDataJob` 仅在 compat 异步模式下继续承担包装角色

- 通知 / 批量发信：
  - `config/ops.php` 已固定为默认同步发送
  - 不再通过环境变量暴露 PHP compat 发送模式切换
  - Rust admin `POST /api/v2/{admin_path}/user/sendMail` 已直接承接后台批量发信
  - Rust 公共 Telegram 广播能力已抽到共享模块：
    - `rust-gateway/src/telegram_notify_support.rs`
  - Rust 工单邮件通知能力已抽到共享模块：
    - `rust-gateway/src/ticket_notify_support.rs`
  - 当前已切到 Rust 广播链的 Telegram 通知包括：
    - 运维告警 `ops_alert`
    - 风险审查提醒 `risk_review`
    - Rust 批量封禁后的 super-admin 通知
    - Rust 后台公告发布 `notice.published`
    - Rust 支付成功通知 `payment.success`
  - Rust 退款链通知：
      - `refund.vote.started`
      - `refund.vote.cast`
      - `refund.status.changed`
  - 当前已切到 Rust 邮件链的工单通知包括：
    - 用户创建节点工单后通知对应管理员
    - 管理员 / 节点管理员回复工单后通知用户
  - `SendEmailJob` / `SendTelegramJob` 仅保留为历史兼容实现，不属于默认部署依赖
- 定时任务兼容层：
  - 核心 scheduler 命令已由 Rust `16 / 16` 覆盖
  - 插件调度审计结果为 `dormant`
  - 因此 `scheduler` 服务当前更接近“极少数自定义 PHP 定时任务兼容层”

这意味着当前剩余 PHP 底层面，已经从“默认直接依赖 Laravel queue job”
收缩为“默认仍通过少量 PHP service 执行 legacy 业务逻辑”。

同时，默认 Docker / Rust router 口径下：

- `docker compose config --services`
  - 仅包含 `gateway + mysql + redis`
- Rust router 已直接注册：
  - `/api/v1/server/ShadowsocksTidalab/*`
  - `/api/v1/server/TrojanTidalab/*`
  - `/api/v1/server/UniProxy/*`
  - `/`
  - `/app`
  - `/login/linux-do`
  - `/{subscribe_path}/{token_or_path}`
- Laravel 默认应用注册已不再包含 `RouteServiceProvider`
- 默认 Rust 运行面已不再保留 `routes/web.php`

因此默认部署下，legacy server 流量入口、页面入口和订阅入口都不再需要 PHP runtime 承载。

当前保留下来的 PHP route 文件主要只承担两类作用：

- 作为 `tools/compat_baselines/php_routes.json.gz.b64` 的历史语义来源
- 作为后续彻底删除 PHP 代码前的对照来源

最新运行证据：

- `bash tools/bench_legacy_submit.sh`
  - 命中 Rust `ShadowsocksTidalab/submit` 入口
  - `40` 个请求全部成功
  - `Requests per second: 1620.09`
  - `P95=7ms`
  - `v2_stat_user` 行数：`500`
  - `v2_stat_server` 行数：`1`
  - 用户/节点统计累计值均符合预期

这说明默认部署下，legacy submit 兼容入口已经不需要 PHP controller / PHP queue / PHP service 参与。

- `bash tools/bench_uniproxy_hotpaths.sh`
  - `POST /api/v1/server/UniProxy/alive`
    - `Requests per second: 4670.71`
    - `P95=1ms`
  - `user_online_sessions` 行数：`100`
  - `v2_user.online_count > 0` 行数：`100`

这说明默认部署下，`alive` 写路径已经完全由 Rust 网关承载，
且与旧 PHP 兼容的 `v2_user.online_count / last_online_at` 读模型也已由 Rust 同步维护。

- Rust 后台批量发信运行验证：
  - 当前工作树二进制：`rust-gateway/target/release/notxboard-gateway`
  - 测试 SMTP 捕获器：`axllent/mailpit`
  - `POST /api/v2/{admin_path}/user/sendMail`
    - 返回：`{"concurrency":4,"failed":0,"sent":4,"success":true,"total":4}`
  - `mailpit /api/v1/messages`
    - 收件数：`4`

这说明后台群发邮件默认也已经可以由 Rust 运行面直接承载，不再要求 PHP Horizon 作为默认底层依赖。

- Rust Telegram 通知链编译验证：
  - `docker run --rm -v "$PWD":/app -w /app/rust-gateway rust:1.89-bookworm sh -lc '/usr/local/cargo/bin/cargo check'`
  - 结果：通过

这说明 Rust 侧公共 Telegram 广播模块已经接入现有运行面，`ops_alert` / `risk_review` / Rust 批量封禁通知不再需要 PHP `TelegramService` 作为底层发送实现。

- Rust 公告 Telegram 通知运行验证：
  - 当前工作树二进制：`rust-gateway/target/release/notxboard-gateway`
  - 本地 fake Telegram API：`http://host.docker.internal:18090/bot`
  - Rust 管理接口：`POST /api/v2/{admin_path}/notice/save`
  - 捕获到的请求：
    - 路径：`/bottest-token/sendMessage`
    - 请求体包含：
      - `chat_id=123456789`
      - `text=公告发布 ... 标题：Rust Notice 4 ...`

这说明 `notice.published` 这条 Telegram 通知能力也已经由 Rust 运行面直接承接，不再依赖 PHP Telegram 插件 hook 作为默认底层路径。

- Rust 支付成功 Telegram 通知运行验证：
  - 当前工作树二进制：`rust-gateway/target/release/notxboard-gateway`
  - 本地 fake Telegram API：`http://host.docker.internal:18090/bot`
  - Rust 回调接口：`POST /api/v1/guest/payment/notify/EPay/notify-test-uuid`
  - EPay 签名参数：
    - `out_trade_no=notify-order-1`
    - `trade_no=gw-pay-runtime-1`
    - `trade_status=TRADE_SUCCESS`
  - Rust 响应：
    - `success`
  - 数据库结果：
    - `v2_order.id=9301` 已更新为 `status=3`
    - `callback_no=gw-pay-runtime-1`
    - `user_plan_subscriptions.order_id=9301` 已写入启用订阅
    - `v2_user.id=1` 已更新到套餐 `7301`
  - 捕获到的请求：
    - 路径：`/bottest-token/sendMessage`
    - 用户通知：
      - `chat_id=444444444`
      - 文本包含 `支付成功`、`订单号：notify-order-1`、`金额：10.00 元`、`套餐：Notify Plan`、`支付渠道：Notify Test`
    - 管理员通知：
      - `chat_id=444444444`
      - 文本包含 `用户支付成功`、`用户：fulladmin@example.com`

这说明支付成功后的 Telegram 用户通知和管理员通知已经由 Rust 支付回调链端到端触发，不再需要 PHP `payment.notify.success` hook 作为默认通知底层。

- Rust EPay 订单级凭据快照验签运行验证：
  - 当前工作树二进制：`rust-gateway/target/release/notxboard-gateway`
  - Rust 回调接口：`POST /api/v1/guest/payment/notify/EPay/notify-test-uuid`
  - 新增 Rust 兼容模块：
    - `rust-gateway/src/laravel_crypto_support.rs`
  - 快照场景：
    - `v2_order.id=9301`
    - 订单快照：`epay_pid=snapshotpid`
    - 订单快照 key：Laravel `Crypt::encryptString('snapshotsecret')` 兼容密文
    - 支付方式默认 key 仍为：`secret123`
    - 回调签名使用：`snapshotsecret`
    - Rust 响应：`success`
    - Rust debug 验签输出：
      - `expected=0f34e5a823f25bda1bdedb55996c7c05`
      - `sign=0f34e5a823f25bda1bdedb55996c7c05`
    - 数据库结果：
      - `v2_order.id=9301` 已更新为 `status=3`
      - `callback_no=gw-snapshot-runtime-1`
  - 回落场景：
    - 清空订单 `epay_pid / epay_url / epay_key_encrypted`
    - 回调签名使用支付方式默认 key：`secret123`
    - Rust 响应：`success`
    - Rust debug 验签输出：
      - `expected=d569de53531a9993fe4b0c9c3debd18f`
      - `sign=d569de53531a9993fe4b0c9c3debd18f`
    - 数据库结果：
      - `callback_no=gw-fallback-runtime-1`

这说明 PHP 回调中原有的订单级 EPay 凭据快照验签能力已经迁入 Rust；存在订单快照时优先按快照 key 验签，快照缺失时仍回落到 `v2_payment.config`。

- Rust EPay checkout 凭据快照写入运行验证：
  - 当前工作树二进制：`rust-gateway/target/release/notxboard-gateway`
  - 新增 Rust 兼容模块：
    - `rust-gateway/src/order_epay_checkout_support.rs`
    - `rust-gateway/src/laravel_crypto_support.rs`
  - 编译验证：
    - `docker run --rm -v "$PWD":/app -w /app/rust-gateway rust:1.89-bookworm /usr/local/cargo/bin/cargo check`
    - `docker run --rm -v "$PWD":/app -w /app/rust-gateway rust:1.89-bookworm /usr/local/cargo/bin/cargo build --release -j 1`
  - 节点主收款场景：
    - Rust checkout 接口：`POST /api/v1/user/order/checkout`
    - 订单：`trade_no=checkout-owner-order-1`
    - 套餐：`v2_plan.id=7302, scope=node, owner_user_id=2`
    - 节点主 EPay profile：`pid=ownerpid`, key 为 Laravel `Crypt::encryptString('snapshotsecret')` 兼容密文
    - checkout 返回：
      - 表单 `action=https://owner-pay.example/pay/submit.php`
      - `pid=ownerpid`
      - `notify_url=http://127.0.0.1:18087/api/v1/guest/payment/notify/EPay/epaycheckoutnotify00000000000001`
    - 数据库结果：
      - `v2_order.id=9401`
      - `payment_id=8802`
      - `handling_amount=0`
      - `epay_pid=ownerpid`
      - `epay_url=https://owner-pay.example`
      - `epay_key_encrypted IS NOT NULL`
    - 回调签名使用：`snapshotsecret`
    - Rust 回调响应：`success`
    - 回调后数据库结果：
      - `v2_order.status=3`
      - `callback_no=gw-owner-cb-1`
      - `user_plan_subscriptions.order_id=9401` 已写入启用订阅
  - 默认支付配置回落场景：
    - 订单：`trade_no=checkout-default-order-1`
    - 套餐：`v2_plan.id=7303, scope=legacy`
    - checkout 返回：
      - 表单 `action=https://default-pay.example/pay/submit.php`
      - `pid=defaultpid`
    - 数据库结果：
      - `v2_order.id=9402`
      - `epay_pid=defaultpid`
      - `epay_url=https://default-pay.example`
      - `epay_key_encrypted IS NOT NULL`
    - 回调签名使用：`secret123`
    - Rust 回调响应：`success`
    - 回调后数据库结果：
      - `v2_order.status=3`
      - `callback_no=gw-default-cb-1`
      - `user_plan_subscriptions.order_id=9402` 已写入启用订阅
  - 快照损坏场景：
    - 订单：`trade_no=checkout-invalid-snapshot-1`
    - 已存在 `epay_pid / epay_url / epay_key_encrypted`，但密文不是 Laravel payload
    - Rust checkout 响应：`HTTP/1.1 400 Bad Request`
    - 响应 message：`Payment snapshot is invalid, please contact support`

这说明 PHP `OrderController::checkout` 中 EPay 凭据选择、订单级快照冻结、快照复用和损坏快照拒绝能力已经迁入 Rust checkout 路径；节点套餐默认按 owner profile 收款，普通套餐仍可回落到默认支付配置，后续 EPay 回调通过订单快照完成验签。

- Rust 退款 Telegram 通知编译验证：
  - `docker run --rm -v "$PWD":/app -w /app/rust-gateway rust:1.89-bookworm sh -lc '/usr/local/cargo/bin/cargo check'`
  - 结果：通过

这说明退款链的关键 Telegram 通知触发点已经接入 Rust 运行面，不再全部依赖 PHP `OrderRefundService` + Telegram 插件 hook 才能产生通知。

- Rust 退款状态 Telegram 通知运行验证：
  - 当前工作树二进制：`rust-gateway/target/release/notxboard-gateway`
  - 本地 fake Telegram API：`http://host.docker.internal:18090/bot`
  - Rust 接口：`POST /api/v1/admin/refunds/2/deny`
  - 捕获到的请求：
    - 路径：`/bottest-token/sendMessage`
    - `chat_id=222222222`
      - 对应用户：`refunduser@example.com`
    - `chat_id=333333333`
      - 对应 assigned admin：`assignedadmin@example.com`
    - 文本包含：
      - `退款状态更新`
      - `申请单：#2`
      - `状态：已拒绝`
      - `说明：deny with new binary`

这说明 `refund.status.changed` 这条 Telegram 通知能力已经由 Rust 运行面完成端到端承接。

- Rust 退款投票开启 Telegram 通知运行验证：
  - 当前工作树二进制：`rust-gateway/target/release/notxboard-gateway`
  - 本地 fake Telegram API：`http://host.docker.internal:18090/bot`
  - Rust 接口：`POST /api/v1/user/node-admin/refunds/2/dispute`
  - 捕获到的请求：
    - 路径：`/bottest-token/sendMessage`
    - `chat_id=222222222`
      - 对应用户：`refunduser@example.com`
    - `chat_id=333333333`
      - 对应 assigned admin：`assignedadmin@example.com`
    - 文本包含：
      - `退款争议投票已开启`
      - `申请单：#2`
      - `套餐：Refund Plan`
      - `截止时间：2026-05-26T17:33:07.000000Z`

这说明 `refund.vote.started` 这条 Telegram 通知能力也已经由 Rust 运行面完成端到端承接。

- Rust 退款投票提交 Telegram 通知运行验证：
  - 当前工作树二进制：`rust-gateway/target/release/notxboard-gateway`
  - 本地 fake Telegram API：`http://host.docker.internal:18090/bot`
  - Rust 接口：`POST /api/v1/user/refund-votes/2`
  - Rust 响应保持 PHP 兼容形态：
    - `{"data":true,"error":null,"message":"操作成功","status":"success"}`
  - 数据库结果：
    - `order_refund_votes` 已新增 `refund_request_id=2,user_id=2,vote=approve`
  - 容器日志：
    - `refund vote cast telegram notify resolved admin recipients`
    - `refund_id=2`
    - `vote="approve"`
    - `recipient_count=1`
  - 捕获到的请求：
    - 路径：`/bottest-token/sendMessage`
    - `chat_id=333333333`
      - 对应 assigned admin：`assignedadmin@example.com`
    - 文本包含：
      - `退款争议有新投票`
      - `申请单：#2`
      - `用户：refunduser@example.com`
      - `投票人：refunduser@example.com`
      - `结果：支持退款`

这说明 `refund.vote.cast` 这条 Telegram 通知能力已经由 Rust 运行面完成端到端承接，并且用户 API 响应仍保持旧 PHP `success(true)` 形态。

- Rust 工单通知链编译验证：
  - `docker run --rm -v "$PWD":/app -w /app/rust-gateway rust:1.89-bookworm sh -lc '/usr/local/cargo/bin/cargo check'`
  - 结果：通过

这说明工单创建/回复后的核心邮件通知副作用已经开始由 Rust 运行面直接承担，不再完全依赖 PHP `TicketService` 的邮件分发逻辑。

- Rust 工单通知运行验证：
  - 当前工作树二进制：`rust-gateway/target/release/notxboard-gateway`
  - 测试 SMTP 捕获器：`axllent/mailpit`
  - 用户建单 `POST /api/v1/user/ticket/save`
    - `mailpit /api/v1/messages`
      - 收到管理员邮件：`nodeadmin@example.com`
      - 主题：`节点工单通知 - notXboard`
  - 节点管理员回复 `POST /api/v1/user/node-admin/ticket/reply`
    - `mailpit /api/v1/messages`
      - 收到用户邮件：`ticketuser@example.com`
      - 主题：`您在notXboard的工单得到了回复`

这说明工单创建通知管理员、管理员回复通知用户这两条默认工单通知副作用已经由 Rust 运行面完成端到端承接。
