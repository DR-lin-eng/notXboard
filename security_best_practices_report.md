# 安全漏洞审计报告

日期：2026-05-31

## 摘要

本轮审计覆盖 PHP/Laravel 业务层、Rust gateway、前端 Blade 页面、公开安装脚本和 `V2bX-dev_new` Go 节点组件。已修复安装引导越权、Bearer token 过期绕过、上传 ZIP 炸弹/路径风险、前端/邮件 XSS、支付回调与网关 URL 校验不足、Telegram webhook 弱校验、静态文件 symlink 越界读取、用户态缓存串号、签名比较时序侧信道、管理员封禁绕过、API Key 有效性探测、本地安装命令注入和 root 安装脚本参数注入等问题。

仍保留 2 个需要上游/架构配合的依赖风险：Go `github.com/quic-go/quic-go` 的 HTTP/3 QPACK DoS 只能升到 `v0.56.0`，继续升到修复版 `v0.57.0` 会破坏当前 `wyx2685/sing-box_mod` 动态用户接口；Rust `rsa 0.9.10` 存在 `RUSTSEC-2023-0071`，当前无直接修复版本。

## 高危/中危已修复

1. 安装引导接口安装后仍可匿名访问。
   影响：已安装站点可被匿名读取/修改初始化配置或暴露管理路径。
   修复：Rust bootstrap 响应统一先调用安装状态检查，已安装或已有管理员时返回冲突错误。
   位置：`rust-gateway/src/bootstrap_support.rs:72`, `rust-gateway/src/bootstrap_support.rs:179`, `rust-gateway/src/bootstrap_support.rs:328`

2. Bearer token 未校验过期时间或 token 类型。
   影响：过期 token 或非用户 token 仍可能访问用户接口。
   修复：PHP/Rust 两侧都校验 `expires_at`，Rust 查询限定 `tokenable_type = App\\Models\\User`。
   位置：`app/Services/AuthService.php:68`, `rust-gateway/src/main.rs:4221`, `rust-gateway/src/main.rs:4230`, `rust-gateway/src/main.rs:4240`

3. 主题/插件 ZIP 上传可触发资源耗尽或危险解压。
   影响：恶意压缩包可能造成磁盘/内存 DoS，或通过异常条目绕过预期路径处理。
   修复：PHP/Rust 上传链路加入压缩包大小、条目数、总解压量、单文件大小限制，并改为流式受限写入。
   位置：`app/Services/ThemeService.php:21`, `app/Services/ThemeService.php:53`, `app/Services/Plugin/PluginManager.php:20`, `app/Services/Plugin/PluginManager.php:44`, `rust-gateway/src/archive_limit_support.rs:59`

4. 前端排行榜渲染使用 `innerHTML`。
   影响：接口返回值若含 HTML/脚本片段，可能形成存储型或反射型 XSS。
   修复：改为 `createElement`、`textContent`、`replaceChildren` 构造 DOM。
   位置：`resources/views/public-dashboard.blade.php:385`, `resources/views/public-dashboard.blade.php:390`, `resources/views/admin-leaderboards.blade.php:472`, `resources/views/admin-leaderboards.blade.php:477`

5. EPay 网关地址和自动提交表单缺少严格转义/校验。
   影响：恶意网关配置可能形成表单 action 注入、HTML 属性注入或非 HTTP(S) 跳转。
   修复：限制网关 URL scheme 为 `http/https`，Rust/PHP 表单 action 与 input 属性做转义。
   位置：`rust-gateway/src/epay_render_support.rs:3`, `rust-gateway/src/epay_render_support.rs:18`, `rust-gateway/src/payment_support.rs:615`, `rust-gateway/src/sponsor_support.rs:238`, `plugins/Epay/Plugin.php:110`, `plugins/Epay/Plugin.php:117`

6. Telegram webhook 使用 URL query token。
   影响：token 容易进入日志、代理、浏览器历史，且旧 MD5 校验较弱。
   修复：设置 webhook 时使用 Telegram 官方 `secret_token`，回调只接受 `X-Telegram-Bot-Api-Secret-Token`，密钥派生改为 HMAC-SHA256。
   位置：`app/Services/TelegramService.php:104`, `app/Services/TelegramService.php:302`, `app/Services/TelegramService.php:311`, `app/Http/Controllers/V1/Guest/TelegramController.php:31`, `rust-gateway/src/telegram_security_support.rs:6`, `rust-gateway/src/admin_v2/config.rs:169`

7. 静态资源可能通过 symlink 越界读取。
   影响：公开 assets/theme 路由可能读取根目录外文件。
   修复：读取前 canonicalize 基准目录和目标文件，要求目标仍在基准目录内。
   位置：`rust-gateway/src/static_files.rs:84`, `rust-gateway/src/static_files.rs:86`, `rust-gateway/src/theme_support.rs:523`, `rust-gateway/src/theme_support.rs:525`

8. 用户态接口缓存键未按用户隔离。
   影响：某些已登录用户接口可能从 URI-only 缓存读到其他用户数据。
   修复：私有响应先鉴权，再使用 `build_user_cache_key(&uri, user.id)`；仅公共登录后配置继续使用全局缓存。
   位置：`rust-gateway/src/user_v1/read.rs:152`, `rust-gateway/src/user_v1/read.rs:153`, `rust-gateway/src/user_v1/orders.rs:88`, `rust-gateway/src/user_v1/orders.rs:89`, `rust-gateway/src/user_v1/security.rs:66`, `rust-gateway/src/user_v1/server_nodes.rs:137`

9. 签名/密钥比较使用普通字符串比较。
   影响：理论上可能泄露签名匹配进度，增加侧信道攻击面。
   修复：PHP 使用 `hash_equals`，Rust 使用集中封装的常量时间比较。
   位置：`plugins/Epay/Plugin.php:160`, `rust-gateway/src/secure_compare_support.rs:1`, `rust-gateway/src/payment_support.rs:195`

10. 本地安装命令拼接文件路径。
    影响：若安装流程处理异常路径，可能扩大本地 CLI 注入面。
    修复：`git checkout HEAD --` 的路径参数改为 `escapeshellarg`。
    位置：`app/Console/Commands/XboardInstall.php:403`

11. 邮件通知模板对正文使用 raw HTML 输出。
    影响：后台或自动任务写入的邮件内容若含 HTML，可能在收件人邮箱中触发 HTML/脚本型内容注入。
    修复：Blade 模板改为 `nl2br(e($content))`，Rust 邮件渲染器继续用转义后的 HTML 替换新旧模板标记。
    位置：`resources/views/mail/default/notify.blade.php:22`, `resources/views/mail/classic/notify.blade.php:128`, `rust-gateway/src/mail_support.rs:177`, `rust-gateway/src/mail_support.rs:178`

12. BTCPay/Coinbase 插件外部请求缺少统一 URL 和超时约束。
    影响：异常网关配置或恶意回调数据可能扩大 SSRF、长时间阻塞或路径拼接风险。
    修复：限制 HTTP(S) 协议、设置连接/请求超时、编码 store/invoice ID、改用请求头读取签名并校验响应结构。
    位置：`plugins/Btcpay/Plugin.php:11`, `plugins/Btcpay/Plugin.php:87`, `plugins/Btcpay/Plugin.php:115`, `plugins/Btcpay/Plugin.php:122`, `plugins/Btcpay/Plugin.php:132`, `plugins/Btcpay/Plugin.php:145`, `plugins/Coinbase/Plugin.php:11`, `plugins/Coinbase/Plugin.php:87`, `plugins/Coinbase/Plugin.php:113`, `plugins/Coinbase/Plugin.php:137`

13. 管理员/超级管理员中间件未复用封禁态拦截。
    影响：被封禁但仍保留管理员标记的账号可能继续访问后台接口。
    修复：新增 `BannedUserGuard`，用户、管理员、超级管理员中间件统一拒绝封禁用户并清理会话。
    位置：`app/Services/Auth/BannedUserGuard.php:11`, `app/Http/Middleware/User.php:26`, `app/Http/Middleware/Admin.php:29`, `app/Http/Middleware/SuperAdminMiddleware.php:36`

14. API Key 校验接口可作为其他用户密钥有效性探测器。
    影响：已登录用户可提交任意 API Key，接口会泄露该 Key 是否存在且是否属于当前用户。
    修复：校验范围收敛到当前登录用户，使用常量时间比较，不再按提交的 Key 查询其他用户。
    位置：`app/Services/ApiKeyService.php:95`, `app/Http/Controllers/V1/User/ApiKeyController.php:159`, `rust-gateway/src/user_v1/account.rs:382`

15. 个人/赞助 EPay 配置缺少集中 URL、路径和私网目标限制。
    影响：用户配置的个人收款网关进入退款或支付链路时，可能造成非 HTTP(S) 跳转、路径注入或对内网地址的服务端请求。
    修复：新增 PHP `UrlSecurity` 与 Rust `url_security_support`，个人 EPay URL 阻止私有/保留 IP，提交路径限制为相对路径；赞助配置和渲染链路统一校验 HTTP(S)。
    位置：`app/Support/UrlSecurity.php:9`, `app/Support/UrlSecurity.php:32`, `app/Http/Controllers/V1/User/PaymentProfileController.php:55`, `app/Services/PaymentProfileService.php:32`, `app/Services/EpayApiService.php:13`, `rust-gateway/src/url_security_support.rs:4`, `rust-gateway/src/user_v1/account.rs:199`, `rust-gateway/src/payment_support.rs:443`, `rust-gateway/src/sponsor_support.rs:102`, `rust-gateway/src/admin_v1/sponsor_epay.rs:67`

16. V2bX/TCPing root 安装脚本参数可写入 JSON、env 或 systemd 文件。
    影响：复制执行的一键安装命令若被传入换行、引号、路径或非 HTTP 安装源，可能造成配置注入、env 注入或执行非预期下载源。
    修复：两个脚本增加 HTTP URL、token、绝对路径、service name、Go version 和数值参数校验，拒绝控制字符、引号、反斜杠和非 HTTP(S) 安装源。
    位置：`public/v2bx-install.sh:79`, `public/v2bx-install.sh:160`, `public/v2bx-install.sh:161`, `public/v2bx-install.sh:165`, `public/tcping-agent-install.sh:54`, `public/tcping-agent-install.sh:63`, `public/tcping-agent-install.sh:279`, `public/tcping-agent-install.sh:280`

17. 面板 URL 和汇率请求缺少集中校验/超时。
    影响：部署命令可能继承非 HTTP(S) 的面板地址；汇率请求可被异常币种参数拼接查询串且没有请求超时。
    修复：部署命令 URL 统一经 `PanelUrlResolver` 和 `UrlSecurity` 解析；汇率请求改用 Laravel HTTP client、币种白名单和连接/请求超时。
    位置：`app/Support/PanelUrlResolver.php:8`, `app/Support/PanelUrlResolver.php:19`, `app/Http/Controllers/V1/User/ServerNodeController.php:594`, `app/Http/Controllers/V1/User/TcpingController.php:192`, `app/Utils/Helper.php:39`

18. LLM/OAuth 外联缺少完整超时与敏感日志收敛。
    影响：管理员误配置 LLM Base URL 可能形成非预期请求目标；OAuth 失败日志可能记录授权码或过长响应体，并且外联没有统一连接超时。
    修复：LLM Base URL 统一限制为 HTTP(S)、拒绝 userinfo/query/fragment 并设置连接超时；Linux DO OAuth 统一 HTTP client 超时，日志只记录响应摘要和授权码长度。
    位置：`app/Services/Llm/OpenAiCompatibleClient.php:40`, `app/Services/Llm/OpenAiCompatibleClient.php:148`, `app/Services/Llm/OpenAiCompatibleClient.php:152`, `app/Services/OAuth/LinuxDoOAuthService.php:86`, `app/Services/OAuth/LinuxDoOAuthService.php:119`, `app/Services/OAuth/LinuxDoOAuthService.php:248`, `app/Services/OAuth/LinuxDoOAuthService.php:253`

## 依赖审计

1. Go：已升级 `golang.org/x/net` 到 `v0.55.0`、`golang.org/x/crypto` 到 `v0.52.0`、`github.com/go-jose/go-jose/v4` 到 `v4.1.4`、`google.golang.org/grpc` 到 `v1.79.3`、`github.com/cloudflare/circl` 到 `v1.6.3`、`github.com/refraction-networking/utls` 到 `v1.8.2`、`github.com/quic-go/quic-go` 到 `v0.56.0`。

2. Go 残余：`govulncheck` 仍报告 `GO-2025-4233`，模块 `github.com/quic-go/quic-go@v0.56.0`，修复版为 `v0.57.0`。当前不能直接升到 `v0.57.0`，原因是 `qpack v0.6.0` 会破坏 `github.com/apernet/quic-go`、`github.com/sagernet/quic-go` 0.52 系列；联动升级 `sing-box` 又会丢失 `wyx2685/sing-box_mod` 提供的 `AddUsers/DelUsers` 动态用户接口。

3. Rust 残余：`cargo-audit` 已知报告 `RUSTSEC-2023-0071`，`rsa 0.9.10` Marvin Attack，无修复版本。当前直接用法位于 Google service account JWT 签名路径，不是 RSA 解密路径，但仍应跟踪上游修复。

## 验证

已通过：

- `docker run --rm -v .../rust-gateway:/app -w /app rust:latest cargo check`
- `docker run --rm -v .../notXboard:/app -w /app php:8.3-cli sh -c 'for f in ...; do php -l "$f" || exit 1; done'`
- `docker run --rm -v .../notXboard:/app -w /app php:8.3-cli sh -c 'for f in app/Services/Llm/OpenAiCompatibleClient.php app/Services/OAuth/LinuxDoOAuthService.php; do php -l "$f" || exit 1; done'`
- `bash -n public/v2bx-install.sh`
- `bash -n public/tcping-agent-install.sh`
- `bash public/v2bx-install.sh --panel https://example.com --node-id 1 --node-type vmess --token ... --install-url file:///tmp/install --skip-install` 按预期拒绝非 HTTP(S) 安装源
- `bash public/tcping-agent-install.sh --panel https://example.com --token $'validtoken012345\nEVIL=1' --skip-start` 按预期拒绝换行 token
- `docker run --rm -v .../V2bX-dev_new:/app ... golang:1.25 go test ./... -run '^$' -count=0`
- `docker run --rm -v .../V2bX-dev_new:/app ... golang:1.25 sh -c 'go install golang.org/x/vuln/cmd/govulncheck@latest && /go/bin/govulncheck ./...'`
- `rg -n "access_token=|webhook\\?access_token|extract_access_token_from_query|telegram_webhook_secret_md5" ...` 无命中
- `rg -n "stream_get_contents\\(|extractTo\\(|std::io::copy\\(&mut file|li\\.innerHTML" ...` 无命中
- `rg -n "file_get_contents\\('https://api\\.exchangerate|Http::asForm\\(\\)->timeout\\(20\\)->post\\(\\$url|findUserByApiKey\\(\\$apiKey\\)|SELECT id, banned FROM v2_user WHERE api_key|is_documentation\\(" app rust-gateway plugins public` 无命中

未通过/未完成：

- `cargo fmt -- --check` 未执行成功，原因是 `rust:latest` 容器缺少 `cargo-fmt`/`rustfmt`。
- `go test ./...` 全量运行阶段出现测试进程阻塞，已改用 `go test ./... -run '^$' -count=0` 验证所有包可编译。
