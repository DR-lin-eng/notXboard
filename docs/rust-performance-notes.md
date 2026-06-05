# Rust Performance Notes

本文件只记录已经落地到 Rust 运行面的并发/吞吐优化点，便于后续压测与回归对照。

## 1. Runtime Defaults

Rust 网关数据库连接池已开放以下调优变量：

- `DB_MAX_CONNECTIONS`
- `DB_MIN_CONNECTIONS`
- `DB_ACQUIRE_TIMEOUT_SECS`
- `DB_IDLE_TIMEOUT_SECS`
- `DB_WRITE_RETRY_ATTEMPTS`
- `DB_WRITE_RETRY_BACKOFF_MS`

当前默认值：

- `DB_MAX_CONNECTIONS=128`
- `DB_MIN_CONNECTIONS=16`
- `DB_ACQUIRE_TIMEOUT_SECS=5`
- `DB_IDLE_TIMEOUT_SECS=300`
- `DB_WRITE_RETRY_ATTEMPTS=3`
- `DB_WRITE_RETRY_BACKOFF_MS=25`

这些变量已体现在：

- `rust-gateway/src/runtime.rs`
- `docs/deployment-docker.md`

### Setting cache

Rust 运行面已为 `v2_settings` 读取增加超短 TTL 的进程内缓存：

- 环境变量：`SETTING_CACHE_TTL_SECS`
- 默认值：`3`

主要收益：

- 降低 `UniProxy`、订阅、登录限流、scheduler 等路径上的重复配置查询
- 配置更新时会主动失效对应 cache key
- 内置了惰性过期清理与简单容量保护，避免长时间运行只增不减

### Node access cache

节点访问控制结果也已加入超短 TTL 的进程内缓存：

- 环境变量：`NODE_ACCESS_CACHE_TTL_SECS`
- 默认值：`3`

主要收益：

- 降低 `UniProxy push/alive`、用户访问面板、`tcping` 节点访问判定的重复 SQL
- 在授权用户、最小信任等级、黑名单变更后会主动失效对应节点 cache

## 2. UniProxy Hot Path Optimizations

### `authenticate_node()` shared path

节点侧 `config/user/alivelist/push/alive/status/audit` 入口都会先执行节点鉴权。

当前已将鉴权查询从 `v2bx_token = ? AND (id = ? OR v2bx_node_id = ?)` 的单条 `OR` 查询，
改成双分支 `UNION ALL` 形式，优先让：

- 主键 `id`
- `idx_server_nodes_v2bx_node_token (v2bx_node_id, v2bx_token)`

更稳定地参与执行计划。

同时已增加超短 TTL（3 秒）的进程内节点鉴权缓存：

- 命中场景下可直接跳过数据库
- 主要覆盖 `config/user/alivelist/push/alive/status/audit` 这类高频重复鉴权
- 同样带有惰性过期清理与容量保护，避免缓存无限增长

### `GET /api/v1/server/UniProxy/user`

已从多段式查询收敛为单次权限/限额聚合查询：

- 访问控制、用户基础字段、个人限额、组限额在 SQL 端直接汇总
- 候选用户集合改为 `UNION` 聚合，避免以 `v2_user` 为外层做大范围相关子查询扫描
- 节点级 `device_limit / connection_limit / speed_limit_down` 覆盖逻辑在 Rust 端做常量裁剪
- 避免原先：
  - `get_accessible_user_ids_for_node`
  - `load_users`
  - `load_effective_limits`
  三段式数据库往返

当前代码也已按职责拆分：

- `rust-gateway/src/uniproxy_support.rs`
  - 保留 UniProxy 通用入口、鉴权、alive/push 写路径
- `rust-gateway/src/uniproxy_user_support.rs`
  - 负责 `UniProxy/user` 的用户快照构建
  - 负责 `msgpack/json` 与 `gzip` 协商
  - 负责 `X-User-Snapshot-Version` / `X-User-Sync-Mode: delta` 协议
  - 负责 latest/version 双层快照缓存

在协议层，`UniProxy/user` 现在支持：

- 传统全量模式：
  - 返回 `{ "users": [...] }`
- 增量模式：
  - 请求头：`X-User-Sync-Mode: delta`
  - 请求头：`X-User-Snapshot-Version: <previous-version>`
  - 响应体：`{ "mode": "delta", "version": "...", "upserts": [...], "removed_ids": [...] }`
- 若快照版本未变化：
  - 返回 `304 Not Modified`

这个调整的目标不是继续压缩“全量大响应”的常数项，而是让已支持 `ETag`/`msgpack` 的 V2bX 客户端逐步切到真正的增量同步。

### `GET /api/v1/server/UniProxy/alivelist`

已改为单查询直接生成在线计数：

- 可访问用户筛选与设备数限制在同一条 SQL 内完成
- `user_online_sessions` 的 `DISTINCT ip_address` 统计下沉到 SQL
- 不再把会话全集拉回 Rust 用 `HashSet` 去重
- 设备数限制过滤也前移到查询路径
- 同一套 candidate-union 判定也服务于 `push/alive` 写路径的用户授权过滤

### `POST /api/v1/server/UniProxy/push`

已从同步重事务改成“请求线程入队，Rust 后台批量 flush”：

- 请求线程只负责：
  - payload 解析
  - 可访问用户过滤
  - 入队
- 默认参数：
  - `PUSH_TRAFFIC_QUEUE_CAPACITY=20000`
  - `PUSH_TRAFFIC_FLUSH_INTERVAL_MS=50`
  - `PUSH_TRAFFIC_MAX_BATCH_JOBS=512`
  - `PUSH_TRAFFIC_MAX_BATCH_ROWS=20000`

后台 flush 内部仍保留这些数据库侧优化：

- 同一批 payload 内按 `user_id` 先聚合上传/下载流量
- `node_traffic_records` 改为批量 `INSERT ... ON DUPLICATE KEY UPDATE`
- `user_traffic_usage_logs` 改为批量 `INSERT`
- `v2_user` 的 `u/d/t` 更新改为 `JSON_TABLE` 驱动的批量更新
- 活跃订阅实例选择改为整批查询，`used_traffic_kb` 回写改为整批 `CASE WHEN` 更新

当前效果：

- `push` 已从同步数据库写请求路径中移出
- 相同压测参数下，吞吐量已从几十 `req/s` 提升到数千 `req/s`

### `POST /api/v1/server/UniProxy/alive`

也已改成“请求线程入队，Rust 后台批量 flush”：

- 默认参数：
  - `ALIVE_SESSION_QUEUE_CAPACITY=20000`
  - `ALIVE_SESSION_FLUSH_INTERVAL_MS=50`
  - `ALIVE_SESSION_MAX_BATCH_JOBS=512`
  - `ALIVE_SESSION_MAX_BATCH_USERS=20000`

后台差异更新逻辑：

- 先整批查询当前节点下相关用户的现有在线会话
- 在 Rust 内存中按用户计算 stale IP 差异
- stale 会话改为整批 `DELETE`
- 新会话 / 已存在会话刷新改为整批 `INSERT ... ON DUPLICATE KEY UPDATE`

实现位置：

- `rust-gateway/src/uniproxy_support.rs`
- `rust-gateway/src/main.rs`

### Async queue metrics

异步 `push/alive` 队列已增加内建指标，并可通过现有 monitor stats 接口读取：

- `capacity`
- `queued_jobs`
- `queued_items`
- `enqueued_jobs_total`
- `enqueued_items_total`
- `flushed_jobs_total`
- `flushed_items_total`
- `flush_failures_total`
- `fallback_sync_total`
- `last_flush_at`

这些指标用于判断：

- 是否发生队列积压
- 是否回退到同步写路径
- flush 是否持续失败
- flush 是否长期没有消费

### Legacy submit queue

legacy 兼容 submit 流量链路现在也已接入异步批量 flush：

- `LEGACY_SUBMIT_QUEUE_CAPACITY=20000`
- `LEGACY_SUBMIT_FLUSH_INTERVAL_MS=50`
- `LEGACY_SUBMIT_MAX_BATCH_JOBS=512`
- `LEGACY_SUBMIT_MAX_BATCH_ROWS=20000`

它也会出现在 `rustAsyncQueues.legacy_submit` 中，便于统一观察：

- legacy 兼容协议是否积压
- 是否回退到同步写路径
- flush 是否失败

## 3. Indexes Added For Hot Queries

为在线会话热查询补充了复合索引：

- `idx_user_online_sessions_user_last_activity_ip (user_id, last_activity, ip_address)`

为订阅与 token 热查找补充了索引：

- `idx_v2_user_token (token)`
- `idx_v2_user_subscribe_path (subscribe_path)`

为节点授权存在性判断补充了 node-first 组合索引：

- `idx_user_node_access_node_user (node_id, user_id)`

同步位置：

- `database/migrations/2025_01_26_000004_create_user_online_sessions_table.php`
- `database/migrations/2026_03_11_000012_add_hot_path_user_lookup_indexes.php`
- `rust-gateway/resources/schema/notxboard.full.sql`
- `resources/schema/notxboard.full.sql`

## 4. Legacy Subscribe Filtering

旧 `v2_server` 订阅线路加载不再全表拉回后再按 `group_ids` 过滤：

- 组过滤已下沉到 SQL：`JSON_CONTAINS(...)`
- `UserRow` 已直接携带 `group_id`，订阅链不再为 legacy server 过滤额外回查一次 `v2_user`
- legacy server 在线/可用性判定已增加超短 TTL 的进程内缓存，避免每条线路都顺序命中 Redis `LAST_CHECK_AT / LAST_PUSH_AT`

实现位置：

- `rust-gateway/src/subscribe_support.rs`

## 5. Legacy Server Submit Write Path

`/api/v1/server/ShadowsocksTidalab/submit` 与 `/api/v1/server/TrojanTidalab/submit` 兼容接口
已从逐用户同步写入改成批量聚合写入：

- `v2_user` 流量累计改为批量更新
- `v2_stat_user` 改为批量 `INSERT ... ON DUPLICATE KEY UPDATE`
- `v2_stat_server` 保留单条汇总写入
- 同样接入 `DB_WRITE_RETRY_*` 并发重试

实现位置：

- `rust-gateway/src/legacy_server_v1.rs`
- `rust-gateway/src/traffic_ingest_support.rs`

2026-05-26 还补到一轮基于当前默认 Rust 镜像的容器压测：

- `bash tools/bench_legacy_submit.sh`
  - `Requests per second: 1620.09`
  - `P50=4ms`
  - `P95=7ms`
  - `P99=7ms`
  - `v2_stat_user` 写入行数：`500`
  - `v2_stat_server` 写入行数：`1`

这轮结果说明 legacy submit 兼容入口在默认部署下已经可以完全由 Rust 网关承载。

同日还补到一轮 `alive` 读模型对齐验证：

- `bash tools/bench_uniproxy_hotpaths.sh`
  - `POST /api/v1/server/UniProxy/alive`
    - `Requests per second: 4670.71`
    - `P95=1ms`
  - `user_online_sessions` 行数：`100`
  - `v2_user.online_count > 0` 行数：`100`

这说明 Rust `alive` 写链现在不仅写入 `user_online_sessions`，
也会同步维护旧兼容读模型 `v2_user.online_count / last_online_at`。

## 6. What Still Needs Real Measurement

当前这些优化大多已有源码级与结构级证据，但仍缺最新代码版本的容器级强验证：

1. 对 `UniProxy user/alivelist` 做更大样本的真实压测，对比优化前后的：
   - 数据库 query 次数
   - P95 / P99 latency
   - 连接池占用
   - CPU 使用率

### Latest Docker evidence

2026-05-31 已补默认 Rust 栈 fresh-container 隔离验证：

- `RUN_ID=0531h HOST_PORT=18088 CLEANUP=1 CLEANUP_IMAGE=1 bash tools/verify_rust_default_stack_isolated.sh`
  - Rust gateway image build 成功
  - 默认 Rust gateway 容器中不存在 `/app/runtime/app`、`/app/runtime/config`、`/app/runtime/plugins`、`/app/runtime/theme`
  - 默认 Rust gateway 容器中不存在 `/app/runtime/public/theme/Maintainable/dashboard.blade.php` 与 `/app/runtime/public/theme/Maintainable/config.json`
  - 全新 MySQL/Redis/gateway 容器启动成功
  - `/bootstrap/full` 返回 `mode=rust-full-schema`
  - Rust passport 登录成功
  - Rust `listHooks` 在无 Laravel `app/plugins` runtime 下返回核心静态 hook 与插件 hook
  - Rust `/` 公共概览页与 `/app` 内置 `Maintainable` 主题页均可在无 theme source runtime 下渲染
  - `POST /api/v2/{admin_path}/system/runDatabaseBackup` 成功
  - 备份文件存在于 `/app/state/backup/2026-05-31_14-31-47_xboard_database_backup.sql.gz`
  - Redis 写入 `notxboard_database_notxboard_cacheSCHEDULE_LAST_CHECK_AT`
  - Rust scheduler 连续输出 `background scheduler tick completed`

2026-05-26 已补到一轮缩小版容器验证：

- `bash tools/verify_rust_default_stack.sh`
  - `/bootstrap/full` 成功
  - `bootstrap_mode=rust-full-schema`
  - Rust admin 登录成功
  - Rust 手动备份接口成功
- `BENCH_USER_COUNT=2000 BENCH_ACTIVE_USERS=300 AB_REQUESTS=60 AB_CONCURRENCY=10 bash tools/bench_uniproxy_hotpaths.sh`
  - `GET /api/v1/server/UniProxy/user`
    - `763.08 req/s`
    - `P50=10ms`
    - `P95=37ms`
    - `P99=40ms`
  - `POST /api/v1/server/UniProxy/alive`
    - `7626.80 req/s`
    - `P50=1ms`
    - `P95=2ms`
    - `P99=3ms`
  - `POST /api/v1/server/UniProxy/push`
    - `9647.85 req/s`
    - `P50=1ms`
    - `P95=3ms`
    - `P99=3ms`
  - 最终 `user_online_sessions` 行数：`300`

- 2026-05-26 同口径复测（更新 `notxboard-gateway:prebuilt-local` 后再次执行相同 bench）
  - `GET /api/v1/server/UniProxy/user`
    - `715.09 req/s`
    - `P50=13ms`
    - `P95=25ms`
    - `P99=32ms`
  - `POST /api/v1/server/UniProxy/alive`
    - `4930.56 req/s`
    - `P50=1ms`
    - `P95=5ms`
    - `P99=6ms`
  - `POST /api/v1/server/UniProxy/push`
    - `6439.84 req/s`
    - `P50=1ms`
    - `P95=3ms`
    - `P99=4ms`

说明：

- `UniProxy/user` 在当前样本下仍明显慢于 `alive/push`，后续继续优化应优先看这条路径
- 两轮结果都证明 `alive/push` 已经稳定在低毫秒级
  - 当前这次改动更像“缓存命中顺序整理”，没有形成足够强的吞吐提升证据，后续应继续针对 `user` 路径做更深的 SQL / 序列化优化

- 2026-05-26 后续 warm-path 复测（typed serialization + 复用访问缓存链后）
  - `GET /api/v1/server/UniProxy/user`
    - `686.99 req/s`
    - `P50=14ms`
    - `P95=23ms`
    - `P99=35ms`
  - `POST /api/v1/server/UniProxy/alive`
    - `5095.54 req/s`
    - `P50=1ms`
    - `P95=4ms`
    - `P99=5ms`
  - `POST /api/v1/server/UniProxy/push`
    - `8087.34 req/s`
    - `P50=1ms`
    - `P95=3ms`
    - `P99=3ms`

- 2026-05-26 冷路径补测（`Cache-Control: no-cache`，避开 5 秒响应缓存）
  - `bash tools/bench_uniproxy_user_cold.sh`
  - `GET /api/v1/server/UniProxy/user`
    - `P50=34.913ms`
    - `P95=52.992ms`
    - `P99=56.233ms`
    - `mean=36.926ms`
    - sample body size: `300162 bytes`

- 2026-05-26 再次调整后复测（typed serialization + 访问缓存复用 + gzip 协商支持）
  - warm path
    - `GET /api/v1/server/UniProxy/user`
      - `727.23 req/s`
      - `P50=12ms`
      - `P95=28ms`
      - `P99=32ms`
  - cold path
    - `GET /api/v1/server/UniProxy/user`
      - `P50=29.936ms`
      - `P95=54.613ms`
      - `P99=58.271ms`
      - `mean=32.105ms`
      - sample body size: `300162 bytes`

- 2026-05-26 最后一次复测（继续复用访问缓存链，保持 msgpack/gzip 协商能力）
  - warm path
    - `GET /api/v1/server/UniProxy/user`
      - `978.31 req/s`
      - `P50=8ms`
      - `P95=18ms`
      - `P99=29ms`
  - cold path
    - `GET /api/v1/server/UniProxy/user`
      - `P50=26.475ms`
      - `P95=37.248ms`
      - `P99=43.471ms`
      - `mean=26.935ms`
      - sample body size: `300162 bytes`

- 2026-05-26 在统一镜像口径（`notxboard-gateway:prebuilt-local`）下重新验证 JSON vs msgpack
  - `bash tools/bench_uniproxy_user_formats.sh`
  - `json-warm`
    - `body_bytes=299058`
    - `P50=4.382ms`
    - `P95=12.964ms`
    - `P99=13.579ms`
    - `mean=5.864ms`
  - `msgpack-warm`
    - `body_bytes=241751`
    - `P50=3.860ms`
    - `P95=12.612ms`
    - `P99=12.660ms`
    - `mean=5.366ms`
  - `json-cold`
    - `body_bytes=299058`
    - `P50=19.746ms`
    - `P95=33.799ms`
    - `P99=34.890ms`
    - `mean=21.849ms`
  - `msgpack-cold`
    - `body_bytes=241751`
    - `P50=15.012ms`
    - `P95=25.051ms`
    - `P99=27.368ms`
    - `mean=17.018ms`

这说明当前 Rust 默认栈下：

- `user/alive/push` 三条热路径都已经具备稳定的容器级运行证据
- `alive/push` 的请求线程已经基本维持在单毫秒级
- `user` 的 warm-path 已受 5 秒短 TTL 响应缓存明显影响，冷路径成本仍然偏高
- `user` 接口当前约 `300KB` 的响应体本身就是性能瓶颈候选，后续优化不应只停留在查询微调
- 复用访问缓存链 + 强类型序列化后，`user` 的 warm/cold 两条路径都出现了更像样的改善
- `msgpack` 协商在统一镜像口径下已确认生效，响应体约下降 `19%`，冷路径 `P95` 进一步从 `33.799ms` 降到 `25.051ms`
- 仅靠 gzip/msgpack 协商仍然不能根治大响应体问题，但它已经提供了一个真实可测的协议级收益基线

## 7. Delta Sync Validation

2026-05-26 在基于当前源码重新构建的镜像
`notxboard-gateway:prebuilt-local`（`sha256:0a1c258a1e6a...`）上补到一轮 `UniProxy/user` 增量同步验证：

- `go test -run '^$' ./api/panel ./node`
  - `api/panel`
  - `node`
  - 均通过，说明 bundled V2bX 客户端的新同步模型至少可编译
- `bash tools/bench_uniproxy_user_formats.sh`
  - `json-warm`
    - `body_bytes=300162`
    - `P95=11.12ms`
  - `msgpack-warm`
    - `body_bytes=242131`
    - `P95=10.824ms`
  - `json-cold`
    - `body_bytes=300162`
    - `P95=17.703ms`
  - `msgpack-cold`
    - `body_bytes=242131`
    - `P95=16.967ms`
- `bash tools/bench_uniproxy_user_delta.sh`
  - `delta-no-change`
    - `sample_body_bytes=0`
    - `response_body_bytes=[0]`
    - 命中 `304`
  - `delta-with-change`
    - `sample_body_bytes=1276`
    - `response_body_bytes=[1276]`
    - 变更后返回 `200` 差量体，不再错误命中 `304`

这一轮还顺手暴露并修复了一个协议级 bug：

- 问题：
  - 服务端把 latest snapshot TTL 当成绝对正确
  - 当数据库已变更但 TTL 未过期时，`delta-with-change` 仍可能返回 `304`
- 修复：
  - 对“带旧版本号的 delta 请求”强制重建当前快照
  - 普通 full 请求与无版本 delta 请求仍复用 latest cache
- 后续若要继续显著改善 `user`，更值得考虑的是协议级改变：字段裁剪、增量同步、分页或版本协商
- 后续更值得补的是更大并发和更长持续时间下的 CPU / 连接池 / 队列回退观测
