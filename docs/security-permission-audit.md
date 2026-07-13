# 权限与路径安全审计

审计日期：2026-07-12

## 执行摘要

本轮审计覆盖 HTTP 路由角色、动态资源 ID、节点发布与套餐授权、机器凭据、授权缓存、OAuth/Telegram 旁路、支付与富文本前端、插件/主题目录以及安装脚本落盘路径。已修复可直接越权的自助限额写入、退款投票 IDOR、PAT 多态类型混淆、支付签名重放，并收口了跨 owner 最终写入、共享节点升级为机器写权、跨节点凭据复用、本地封禁被外部同步复活、旧物化授权残留、缓存延迟撤权和上传目录路径别名等组合链。

当前授权判定必须同时满足：路由角色允许、当前主体拥有或被明确授予资源、最终 SQL/文件操作再次携带主体约束。节点套餐权利在请求时直接根据有效订阅和套餐 `node_ids` 计算，并要求 `plan.owner_user_id = server_nodes.user_id`，不再信任旧的 `user_node_plan_access` 物化结果；`user_node_access` 只表示 SuperAdmin owner 的显式直接共享。

仍有一个明确的协议边界：机器流量增量上报没有单调 `batch_id`。当前仅用 Redis 对相同载荷做 10 秒去重，不能提供跨时间窗口的严格幂等，详见“剩余边界”。

## 权限矩阵

| 主体 | 允许范围 | 明确禁止 |
| --- | --- | --- |
| 访客 | 健康检查、受公开模式控制的页面/目录、已验签支付回调、Telegram webhook | 用户数据、后台、监控、安装资源 |
| 普通用户 | 本人账户、会话、订单、退款、工单、邀请、礼品卡、赞助 | 他人对象、自行提高或删除限额、后台 API |
| 资源所有者 | 自有 `server_nodes`、自有节点套餐、节点配置、通知与审计规则、自有 TCPing agent | 共享节点写入；直接向用户分享或按信任等级发布节点，除非同时是 SuperAdmin |
| 套餐订阅者 | 有效节点套餐中由同一 owner 持有且列入 `node_ids` 的节点使用权 | 跨 owner、套餐外、过期/停用/流量耗尽后的继续使用、节点管理权 |
| 直接共享用户 | 仅 SuperAdmin owner 明确授予节点的读取和使用权 | 节点状态、告警、配置、共享关系和机器报告写入 |
| 被指派运营者 | `assigned_admin_user_id = 当前用户` 的节点工单/退款 | 未指派或指派已变更的对象 |
| 管理员 | 全局订单、支付、套餐、节点、公告、优惠券、礼品卡、工单、统计、系统日志 | 超管专属系统操作；用户/组限额处理器仍会再次要求 SuperAdmin；封禁后所有旁路失效 |
| SuperAdmin | 管理员全部能力，加用户/限额/API key/系统配置/插件/主题/备份/维护/监控；可发布自己拥有的节点 | 仍须通过隐藏后台路径；发布和写入仍须通过对象 owner 检查 |
| `is_staff` | 展示和通知属性 | 不构成 HTTP 鉴权角色 |
| V2bX 节点 | 256-bit 随机 token 加 `node_id` 绑定的单节点 | 其他节点、已封禁 owner、弱 token、用户 API key、重复 token |
| 旧节点协议 | HMAC 派生的 `type + server_id` 单节点 token | 直接使用全局 `server_token` |
| TCPing agent | 本 owner 的自有节点，即使 owner 是 SuperAdmin 也没有全局 agent 范围 | 共享节点、其他 owner 节点、禁用 agent、已封禁 owner |
| 安装 bootstrap | HMAC 签名、10 分钟、限 kind/owner/subject/asset 的固定资源包 | 其他文件、跨 kind、转移/撤销后兑换、二次兑换 |

## 强制不变式

1. `/api/v1/user/*`、后台 API 和 monitor API 分别经过 User/Admin/SuperAdmin 路由中间件；敏感处理器保留二次角色检查。实现见 `router_support.rs:113-140`、`router_support.rs:255-303` 和 `authorization_support.rs:10-19`。
2. 后台动态路径、固定 v1 后台 API 和 monitor API 都校验当前 `secure_path`，不匹配返回 404；路径必须是 8 到 64 字节的单个非保留安全段并使用常量时间比较。见 `router_support.rs:336-383` 和 `authorization_support.rs:31-64`。
3. Bearer token 必须是 `App\\Models\\User`、未过期且 abilities 包含 `"*"`；角色鉴权先于处理器和机器读缓存。见 `main.rs:4336-4407`、`uniproxy_support.rs:521-523` 和 `uniproxy_support.rs:554-556`。
4. 用户可控动态 ID 的最终写操作携带 `user_id`、`owner_user_id`、`author_user_id`、`assigned_admin_user_id` 或状态条件；需要保持归属稳定的流程在事务内执行 `FOR UPDATE`，并检查 `rows_affected`。
5. 直接用户分享与按信任等级发布仅允许 SuperAdmin owner。历史 `user_node_access` 和 trust-wide 配置只在当前 owner 仍是未封禁 SuperAdmin 时生效。见 `user_v1/access.rs:182-225`、`user_v1/access.rs:332-446` 和 `access_control_support.rs:244-270`。
6. 节点套餐授权由 `v2_plan.scope='node'`、`plan.owner_user_id = node.user_id`、`plan.node_ids`、有效 `user_plan_subscriptions`、有效期和剩余流量动态计算；节点列表、实际订阅输出、反向 UniProxy 用户、工单节点权限和流量计费订阅选择使用同一 owner 不变式。见 `access_control_support.rs`、`main.rs`、`tickets_support.rs` 和 `traffic_ingest_support.rs`。
7. 用户 UUID 在订阅输出和 V2bX 用户列表中通过 `APP_KEY + base_uuid + owner_user_id + node_id` 派生为节点级 UUID，避免一个节点泄漏的凭据直接复用于其他节点。见 `main.rs:1428-1464`、`main.rs:7409-7439` 和 `uniproxy_user_support.rs:235-255`。
8. TCPing agent 配置仅下发 owner 节点；样本写入在事务中锁定节点并复核 `current_owner == agent.user_id`。见 `tcping_agent_v1.rs:85-215` 和 `tcping_agent_v1.rs:295-332`。
9. 节点共享、套餐、用户/组限额、购买发放、退款撤销订阅、封禁和安全凭据变更在成功提交后清理节点授权、UniProxy 用户快照和响应缓存。见 `access_control_support.rs:458-467`、`admin_v1/group_limits.rs:244-268`、`admin_v1/users_limits.rs:346-448`、`payment_support.rs:387-437`、`refunds_support.rs:699-700` 和 `refunds_support.rs:1009-1011`。
10. 上传包目标必须是规范名、根目录直属子目录、非 symlink 且无大小写/Studly 别名冲突。解压临时目录为 0700 并自动清理；staging/backup 位于 `/app/state/.plugin-*` 或 `/app/state/.theme-*`，不进入可扫描插件/主题根目录。见 `archive_limit_support.rs:27-62`、`admin_v2/plugin/lifecycle.rs:570-619` 和 `admin_v2/theme/mod.rs:414-463`。
11. 安装资源先验签 query 再查询 Redis，ticket 用 `GETDEL` 一次兑换，兑换时再次复核 owner、subject 和当前机器凭据；所有凭据响应为 `private, no-store`。见 `machine_bootstrap_support.rs:6-105` 和 `machine_bootstrap_support.rs:120-219`。
12. Linux DO callback、refresh、手动同步和定时同步均不能解除本地封禁；用户行在资料写入和 PAT 签发前使用 `FOR UPDATE` 复核，外部 inactive 只能执行 `0 -> 1` 封禁。OAuth `state` 还必须匹配 callback-path、HttpOnly、SameSite=Lax Cookie 中的浏览器 nonce HMAC，且在换 code 前校验、在所有结果中清除。
13. Telegram 所有绑定后入口统一只加载 `telegram_id = ? AND banned = 0` 的用户；历史管理员角色、已有绑定和 callback 都不能绕过封禁。
14. `link_only` 分享 token 必须为 32 到 64 字节的 URL-safe 强 token；读取、展示入口和最终下单再次校验。全局公告只信任当前未封禁的 Admin/SuperAdmin 作者，普通或封禁作者的历史 global 数据不会传播给其他用户；隐藏知识分类不进入用户分类列表。

## 已修复问题

### P-01 Critical - EPay checkout 签名可重放为支付回调

- 影响：用户可不付款直接把 checkout 参数提交到 notify，使订单进入已支付处理。
- 修复：仅接受 `TRADE_SUCCESS/TRADE_FINISHED`，并绑定订单 `payment_id`、route UUID、`pid` 和精确分值金额。见 `payment_support.rs:182-290` 和 `guest_v1/payment.rs:47-129`。

### P-02 High - 普通用户可写入或删除个人限额

- 影响：个人限额优先于组限额，用户可通过大数值绕过管理策略。
- 修复：`/api/v1/user/limits` 只保留 GET；个人和组限额写入处理器再次要求 SuperAdmin，并在成功后失效全部授权缓存。见 `router_support.rs:177`、`admin_v1/users_limits.rs:346-448` 和 `admin_v1/group_limits.rs:159-315`。

### P-03 High - 退款投票详情跨用户 IDOR

- 影响：任意登录用户可枚举已关闭/待处理退款并读取邮箱、订单号、网关流水和投票人。
- 修复：只加载投票中且未过期记录，并使用最小公开 DTO 脱敏。见 `refunds_support.rs:173` 和 `user_v1/refund_votes.rs:90`。

### P-04 High - PAT 缺少多态类型约束

- 影响：其他 tokenable 类型 ID 与用户 ID 相同时，会话列表、删除和封禁可能跨类型操作 token。
- 修复：用户、管理员、路由鉴权和批量封禁统一限制 `tokenable_type='App\\Models\\User'`。见 `main.rs:4336-4407`、`user_v1/security.rs:74`、`admin_v2/user.rs:636` 和 `ban_support.rs:121`。

### P-05 High - 共享节点读权可升级为 TCPing 写权

- 影响：共享用户或旧版 SuperAdmin agent 可改写不属于自己的节点在线状态和告警。
- 修复：所有 TCPing agent 一律只下发 owner 节点；样本写入前锁行复核 owner，节点状态最终 UPDATE 同时携带 `id + user_id`。见 `tcping_agent_v1.rs:85-215` 和 `tcping_agent_v1.rs:295-332`。

### P-06 High - 订阅凭据可绑定 Telegram 账户

- 影响：订阅链接常进入客户端、日志或共享渠道，持有者可将站点账户绑定到自己的 Telegram。
- 修复：登录态生成 10 分钟一次性绑定码，Telegram 端使用 Redis `GETDEL`，并复核封禁、已绑定账户和 Telegram ID 冲突。见 `user_v1/read.rs:338` 和 `guest_v1/telegram.rs:169`。

### P-07 High - 用户级或全局节点凭据可跨节点冒用

- 影响：V2bX 曾把用户 API key 写入多个节点；旧协议曾使用单个全局 `server_token`；用户协议 UUID 也可跨节点复用。
- 修复：V2bX 使用每节点强随机 token 并同时绑定 `node_id`，拒绝弱值、用户 API key 和重复 token；旧协议 token 用 HMAC 绑定 `type + id`；用户 UUID 再派生为节点级 UUID。见 `user_v1/server_nodes.rs:632-701`、`uniproxy_support.rs:782-834`、`legacy_server_v1.rs:304` 和 `main.rs:7409-7439`。

### P-08 High - 插件 code 别名可覆盖或删除插件根目录

- 影响：`_`/`__` 可映射为空 Studly 目录；`foo_bar`/`foo__bar` 等可映射同一目录，后上传者可覆盖或递归删除错误目标。
- 修复：规范 code、直属子目录校验、根目录保护、原 config code 校验、symlink/大小写/Studly 冲突阻断；插件与主题 staging/backup 移至私有 state 根并用 0700 RAII 临时目录清理异常路径。见 `admin_v2/plugin/lifecycle.rs:293-485`、`admin_v2/plugin/lifecycle.rs:570-619` 和 `admin_v2/theme/mod.rs:414-463`。

### P-09 High - 普通 owner 可把节点发布给任意用户或信任等级

- 影响：低权限 owner 可创建广域节点授权，历史共享记录在 owner 权限下降或封禁后仍可能继续生效。
- 修复：直接用户分享和 trust-wide 发布仅允许 SuperAdmin 且仍要求节点 owner；读取历史共享时同时复核 owner 当前未封禁且仍为 SuperAdmin。节点套餐购买是独立的显式授权，不等同于直接发布。见 `user_v1/access.rs:182-225`、`user_v1/access.rs:332-446` 和 `access_control_support.rs:155-218`。

### P-10 High - 节点套餐授权依赖过期物化表

- 影响：套餐删除节点、订阅过期、流量耗尽或退款后，旧 `user_node_plan_access` 行可能继续授权；正向列表与节点反向用户列表可能不一致。
- 修复：两向查询都直接联结 `user_plan_subscriptions + v2_plan.node_ids` 并检查状态、有效期和流量；套餐、购买、退款撤销订阅和限额变更提交后统一清理授权相关缓存。见 `access_control_support.rs:155-218`、`access_control_support.rs:238-365`、`access_control_support.rs:458-467`、`refunds_support.rs:699-700` 和 `refunds_support.rs:1009-1011`。

### P-11 Medium - 最终写入未重复携带 owner 条件

- 影响：前置归属查询与最终写入之间发生资源转移或改派时，可改写新 owner 的对象。
- 修复：节点访问、节点管理、通知/审计、部署 token 和运营退款在事务内锁定 owner/assignment，并在最终 SQL 加入主体与状态条件及影响行数检查。

### P-12 Medium - 富文本、错误信息和 EPay 表单形成存储型/DOM XSS 链

- 影响：套餐、公告、节点、工单、退款证据、错误文本、支付 URL 或返回 HTML 可进入 `dangerouslySetInnerHTML`/`innerHTML`，低权限内容可在用户或后台会话中执行脚本。
- 修复：套餐与公告富文本使用 `ammonia` allowlist 清洗；低权限 Portal 套餐/公告改为文本渲染；Maintainable 对动态文本和 URL 使用安全 DOM/转义，EPay POST 只在脱离文档的 `DOMParser` 中读取 HTTP(S) action 和 hidden inputs，再创建新表单并调用原型 `submit`。见 `html_safety_support.rs:1-18`、`node_plans_support.rs:71-73`、`user_v1/notices.rs:120-175`、`rust-gateway/resources/public/theme/Maintainable/app.js:3188-3220` 和 `tools/verify_theme_xss_boundaries.sh`。

### P-13 Medium - 支付 URL 可跨源、逃逸目录或指向显式内网地址

- 影响：普通用户的 EPay profile 可诱导浏览器访问本地/保留地址，或用绝对路径、`../`、query/fragment 改写支付目标。
- 修复：普通用户 profile 仅接受 HTTP(S)、无 userinfo、非显式私网/保留 IP；提交路径必须是受限相对路径，join 后保持同源且位于配置的 base 目录。受信任管理员的 sponsor EPay 仍允许内网网关，但同样受 scheme、路径和同源约束。见 `url_security_support.rs:4-125`、`user_v1/account.rs:185-205` 和 `payment_support.rs:510-549`。

### P-14 Medium - 安装脚本长期 token 进入命令行且落盘权限过宽

- 影响：token 可进入剪贴板、shell history、进程参数和默认 0644 文件；symlink 可改写任意目标。
- 修复：命令只包含 10 分钟签名 ticket，脚本内一次兑换长期 token；加入 `umask 077`、逐级 symlink 检查、同目录原子替换和 `0700/0600` 权限。V2bX 自签证书生成也会截断旧文件、使用正确 RSA PEM 类型，并强制私钥为 `0600`。见 `machine_bootstrap_support.rs:6-81`、`public/v2bx-install.sh:3`、`public/tcping-agent-install.sh:3` 和 `V2bX-dev_new/node/cert.go`。

### P-15 High - Linux DO 同步可复活本地封禁账号

- 影响：管理员封禁后，Linux DO `active=true` 的 callback、refresh 或定时同步可写回 `banned=0` 并签发新 PAT；与保留的管理员角色组合后可恢复后台权限。
- 修复：所有 OAuth 写入锁定当前用户并拒绝任何已封禁状态；外部 inactive 使用独立来源记录且只能新增封禁，PAT 在同一用户行锁事务中签发，token 持久化也要求 `banned=0`。见 `oauth_v1/linux_do.rs`、`oauth_sync_support.rs` 和 `ban_support.rs`。

### P-16 Medium - Telegram 历史绑定绕过封禁

- 影响：封禁不清除 `telegram_id` 和角色，旧 loader 仍允许获取订阅 URL、操作工单；历史 Admin 还可列出全站工单。
- 修复：绑定用户统一查询加入 `banned=0`，覆盖命令、回复、callback、入群检查和管理员工单列表；`is_silenced` 保持原有非封禁语义。见 `guest_v1/telegram.rs`。

### P-17 Medium - OAuth state 未绑定浏览器导致 login CSRF

- 影响：攻击者可把自己的未兑换 callback URL 发给受害者，使 callback 页面把攻击者 PAT 写入受害者浏览器，造成账号混淆。
- 修复：redirect 生成独立 256-bit 浏览器 nonce，Redis state 保存 `HMAC(state, nonce)`；callback Cookie 限专属路径并设置 HttpOnly、SameSite=Lax，HTTPS 自动 Secure。绑定在 code exchange 前常量时间校验，成功或失败均清 Cookie。见 `oauth_v1/linux_do.rs`。

### P-18 Medium - 脏套餐数据可形成跨 owner 节点授权

- 影响：若套餐换 owner 后保留原 `node_ids`，仅凭有效订阅可在正向节点列表、实际订阅、反向节点用户或工单/计费路径获得其他 owner 节点能力。
- 修复：所有节点套餐授权路径同时绑定套餐 owner 与节点 owner；Admin 创建、更新和仅修改 owner 时也验证全部有效节点属于最终 owner。见 `access_control_support.rs`、`main.rs`、`tickets_support.rs`、`traffic_ingest_support.rs` 和 `admin_v1/node_plans.rs`。

### P-19 Low - 弱分享 token 与历史内容可绕过新发布策略

- 影响：可猜测的 link-only token 能被直接下单；旧普通用户 global 公告、封禁发布者公告和隐藏知识分类仍可能被其他用户看到。
- 修复：分享 token 在解析、查询、可见性与下单处统一要求 32 到 64 字节 URL-safe 值；全局公告复核作者当前未封禁且仍为 Admin/SuperAdmin，隐藏知识分类查询加入 `show=1`。见 `node_plans_support.rs`、`user_read_support.rs`、`order_creation_support.rs`、`payment_support.rs`、`user_v1/notices.rs` 和 `user_v1/read.rs`。

## 升级与运维注意

1. 逐个 V2bX 节点重新生成部署命令。旧 token 只要过短、在多节点复用或等于 owner 的用户 API key，新鉴权就会失败关闭；重新生成命令会在节点 owner 事务锁内轮换为每节点强随机 token。不要把旧用户 API key 手工填回节点配置。
2. 旧协议节点不再直接接受全局 `server_token`。管理员节点列表返回派生的 `node_token`，需更新对应节点配置。
3. 节点级 UUID 会改变各节点看到的用户协议凭据。升级后应让节点刷新用户列表，并让客户端刷新订阅；不要在节点端继续缓存旧的全局 UUID。
4. 旧的普通 owner 直接共享或 trust-wide 配置现在会被忽略。需要持续授权时应改用有效节点套餐；需要直接发布时，节点必须由未封禁 SuperAdmin 持有并重新授权。
5. 现有数据库在增加唯一索引前，先检查 `v2bx_token` 和非空 `telegram_id` 重复。新完整建库 schema 已将二者设为唯一。
6. 安装 ticket 有效期 10 分钟并使用 `GETDEL`。如兑换成功后客户端在接收响应前断线，必须重新生成安装命令，不会重放返回长期凭据。
7. `is_staff` 仍不是 HTTP 角色。以后若要授予客服权限，必须新增显式 capability，不应把 staff 隐式并入 admin。
8. 反向代理访问日志不得记录 installer query。使用 `$uri`，不要记录 `$request`、`$request_uri` 或 `$args`，避免签名 query 泄漏；见 `docs/deployment-docker.md`。
9. 检查现有 `scope='node'` 套餐：`owner_user_id` 必须非空，且每个 `node_ids` 节点都由该 owner 持有。错配套餐升级后会安全失效；先修正 owner/节点列表，再恢复销售。旧 `link_only` token 少于 32 字节或含非 URL-safe 字符时同样失效，保存套餐时会生成或要求新的强 token。
10. Linux DO 上游 inactive 导致的封禁不会在上游恢复 active 后自动解除；确认账号状态和风险后由管理员显式解封，避免外部身份源覆盖本地处置。
11. 历史 `scope_type='global'` 公告仅在作者仍为未封禁 Admin/SuperAdmin 时向其他用户展示；NULL 作者或普通用户作者的旧记录应由管理员迁移、重新发布或删除。

## 剩余边界

### R-01 - 机器增量流量报告尚无严格幂等标识

UniProxy 和旧协议会将规范化后的 `user_id/upload/download` 集合哈希，并以 `node_id + fingerprint` 在 Redis 执行 10 秒 `SET NX EX`。这能吸收短时间内的相同重试，但不等价于单调批次号：10 秒后重放、改变拆包方式或 Redis 不可用时仍可能重复计费；两个内容恰好相同的合法批次在 10 秒内也可能被合并。见 `uniproxy_support.rs:571-624` 和 `legacy_server_v1.rs:251-315`。

当前服务端会过滤无权用户、限制单行和整批增量，并把 token 绑定到单节点，但机器凭据仍代表“该节点可为授权用户报告流量”的固有信任。后续协议应由 agent 持久化并发送每节点单调 `batch_id`，服务端在同一计费事务中持久化唯一 `(node_id, batch_id)`，只有成功提交后 agent 才清除本地批次。

### R-02 - 私网 URL 检查不是通用 SSRF 解析器

普通用户 EPay profile 会拒绝 `localhost` 和显式私网/保留 IP，但不会解析普通域名。当前支付目标只返回给浏览器跳转，后端不会主动抓取该 URL；如果未来增加服务端探测、退款请求或 webhook fetch，必须在连接和每次重定向前进行 DNS 解析后的 IP 范围复核，并设置超时、响应上限和重定向策略。

## 验证基线

- Rust：`cargo test --release --locked -j 1` 全量 73 项通过，release 二进制构建通过；覆盖缓存撤权、OAuth/Telegram 封禁、OAuth 浏览器绑定、套餐 owner、路径、真实 MySQL 类型和富文本边界。
- Go：V2bX 与两份 TCPing agent `go test ./...` 通过，发布副本与源副本使用 `cmp` 校验一致。V2bX 文件监听和真实 ACME 用例默认不阻塞、不联网；ACME 仅在显式设置 `V2BX_LIVE_ACME_TEST=1` 及真实凭据时运行，并使用测试临时目录。
- 前端：5 份修改 bundle `node --check` 通过，`tools/verify_theme_xss_boundaries.sh` 通过。
- 脚本：4 份安装脚本及权限/XSS verifier 通过 `bash -n`；安装脚本覆盖权限、原子写入和 symlink 拒绝。
- MySQL 8：动态节点套餐授权验证购买立即授权、移除节点立即撤权、旧物化行无效、过期/耗尽拒绝、无限流量允许，并验证反向节点用户查询结果。
- 集成测试：最新镜像在全新 MySQL/Redis 上通过 `tools/verify_rust_default_stack_isolated.sh` 的 12 步新装、低暴露首页、bootstrap、备份和调度器验证。`tools/verify_permission_boundaries.sh` 的 A/B 用户、Admin、SuperAdmin 和两节点机器 token 10 步用例在同一数据库连续两次通过；覆盖跨对象 GET/PUT/DELETE、traffic/status/TCPing、access/blacklist/audit/deploy、TCPing owner、bootstrap 资源、跨节点 token、脏套餐正反向授权、实际 Clash 订阅、弱 token 下单、owner-only 更新、隐藏分类与历史公告，并查库确认失败写入未改变状态。
