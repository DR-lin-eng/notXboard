# 易支付（EPay）兼容接口接入说明

本项目已内置 EasyPay / CodePay / VPay 兼容支付方式。默认 Rust gateway 镜像使用 Rust 内置插件目录，不再依赖运行时复制 `plugins/Epay/Plugin.php`。

对 `https://credit.linux.do/epay` 的兼容实现按你提供的协议说明实现（`type=epay`，`/pay/submit.php`，签名算法、回调验签与 `trade_status` 规则）。

## 配置位置

在后台“支付方式”中启用插件 `EPay/易支付`，按表单填写配置项。默认 Rust 栈的内置插件元数据来自 `rust-gateway/resources/plugins/builtin_plugins.json`；PHP `plugins/Epay` 仅作为兼容层源码保留。

## 参数说明（提交）

插件默认使用常见易支付接口参数：

- `pid`：商户 ID
- `key`：通信密钥（用于签名）
- `type`：固定 `epay`
- `out_trade_no`：订单号（系统 trade_no）
- `name`：商品名称（默认使用订单号）
- `money`：金额（元，系统内部为分）
- `notify_url`：异步回调地址（系统自动生成）
- `return_url`：同步跳转地址（系统自动生成，指向前端 `/#/order/{trade_no}`）
- `sign` / `sign_type`：签名（`MD5`）

签名算法（credit.linux.do 版本）：

1) 取所有非空字段（排除 `sign`、`sign_type`）
2) 按 ASCII 升序拼接为 `k1=v1&k2=v2`
3) 在末尾追加密钥 `...{secret}`
4) MD5 输出小写十六进制为 `sign`

## submit.php 路径与 POST/GET

不同服务商的提交接口路径可能不同，插件增加了可配置项：

- `submit_path`：默认 `/submit.php`
- `use_post`：默认关闭；开启后返回自动提交的 HTML `<form method="post">`（credit.linux.do 必须 POST）

## 回调（notify）

回调会验签，若包含 `trade_status` 字段，仅当状态为 `TRADE_SUCCESS`/`TRADE_FINISHED`/`SUCCESS` 才视为成功；回调需要返回 `success`（不区分大小写）才会停止重试。

系统回调路由（由支付方式 UUID 决定）：

- `POST/GET /api/v1/guest/payment/notify/{method}/{uuid}`

## credit.linux.do 推荐配置

- `url`: `https://credit.linux.do/epay`
- `submit_path`: `/pay/submit.php`
- `use_post`: `true`
