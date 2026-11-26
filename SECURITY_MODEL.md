# notXboard threat model

Scope: `DR-lin-eng/notXboard`. Built from the code, `README.md`, `docs/deployment-docker.md`, `docs/security-permission-audit.md` (audit items P-01..P-19, R-01, R-02) and the user's answers (marked **[user]**). No separate infrastructure repository; deployment facts come from `Dockerfile`, `deploy/compose.ghcr.yml` and the docs.

## System overview and trust boundaries

A single Rust binary (`rust-gateway`, axum 0.8 + sqlx/MySQL 8 + Redis) serves every surface; the Vue 3 SPA (`web/src`, built into `rust-gateway/resources/public/app`) and legacy themes are static assets. Router: `rust-gateway/src/router_support.rs`. Root-level PHP-era directories (`plugins/`, `resources/`, `theme/`, `storage/`) are not copied into the gateway image.

Deployment **[user]**: the gateway sits behind a reverse proxy/CDN (nginx/Cloudflare) that sets `X-Forwarded-For`; port 8000 is not public. MySQL and Redis are on the private compose network `notxboard`. Optional HTTP Basic gate for pages (`WEB_ACCESS_USERNAME/PASSWORD`, `exposure_control.rs`).

Trust boundaries: internet -> proxy -> gateway; user <-> other users; node owner <-> other node owners; Admin <-> SuperAdmin; node machine <-> gateway; gateway -> payment gateways / Telegram / Linux DO / LLM risk-review endpoint.

## Components

| Component | Code |
| --- | --- |
| Auth | `passport_v1/`, `oauth_v1/linux_do.rs`, `authorization_support.rs`, `authenticate_route_account` in `main.rs`, `api_key_support.rs`, `guest_v1/telegram.rs`, `telegram_security_support.rs`, `secure_compare_support.rs` |
| User API | `user_v1/` (account, orders, tickets, refunds, refund_votes, invites, gift_cards, notices, security, sponsors, traffic, read), `tickets_support.rs`, `user_read_support.rs` |
| Node Owner | `user_v1/server_nodes.rs`, `user_v1/node_admin_*.rs`, `user_v1/node_plans.rs`, `user_v1/access.rs`, `user_v1/tcping.rs`, `user_v1/audit.rs`, `access_control_support.rs`, `node_plans_support.rs`, `node_free_quota_support.rs` |
| Admin API | `admin_v1/`, `admin_v2/` (incl. plugin/, theme/, system/, config.rs, risk_review.rs), `command_center_support.rs`, `backup_support.rs` |
| Payments | `payment_support.rs`, `payment_notify_support.rs`, `order_*`, `guest_v1/payment.rs`, `refunds_support.rs`, `refund_notify_support.rs`, `commission_support.rs`, `sponsor_support.rs`, `epay_render_support.rs`, `url_security_support.rs` |
| Machine Protocol | `uniproxy_support.rs`, `uniproxy_user_support.rs`, `legacy_server_v1.rs`, `tcping_agent_v1.rs`, `machine_bootstrap_support.rs`, `traffic_ingest_support.rs`, `public/*-install.sh`, `public/tcping-agent-src/` |
| Subscription | `subscribe_entry` in `main.rs`, `subscribe_support.rs`, `subscribe_formats.rs`, `subscribe_protocols_v2.rs`, `client_v1.rs`, `subscription_security_support.rs` |
| Web Frontend | `web/src`, `rust-gateway/resources/public/` (admin console, Maintainable/portal themes) |
| Deployment | `Dockerfile`, `rust-gateway/Dockerfile`, `deploy/`, `bootstrap_support.rs`, `.github/workflows/`, `.env.example` |

## Entry points and who can reach them

- Guest: `/healthz`, public dashboard/catalog (opt-in via `PUBLIC_SITE_MODE`/`PUBLIC_CATALOG_ENABLED`), installers, `/bootstrap/*` (requires `X-Bootstrap-Token`, >= 24 chars), passport register/login/forgot, Linux DO OAuth callback, payment notify routes, Telegram webhook (`x-telegram-bot-api-secret-token`).
- Subscription: `/{SUBSCRIBE_PATH}/{token}` and client v1 - bearer of the subscription token.
- User (`RequiredAccountRole::User`): `/api/v1/user/*`, including node-owner functions.
- Admin: `/api/v2/{admin_path}/*` and fixed v1 admin routes; hidden `secure_path` plus `RequiredAccountRole::Admin`; SuperAdmin-only handlers re-check `SuperAdmin`.
- Machines: UniProxy/Xboard-Node (`/api/v1/server/UniProxy/*`, WebSocket), legacy server protocol, TCPing agent - per-node token + `node_id`.

## Attacker profiles that matter

- **[user]** Anyone on the internet can create an account (public sign-up / Linux DO). Unauthenticated and authenticated-user attackers are in scope at full severity.
- **[user]** Any registered user can become a node owner (personal admin). Node owners are untrusted toward other owners' nodes, plans, tickets, refunds and users, and toward global admin.
- Node machines run on node owners' servers and are as trusted as their owner, scoped to their own node.
- **[user]** Admin is semi-trusted: Admin -> SuperAdmin escalation or Admin -> code execution / server compromise is a real issue.
- **[user]** SuperAdmin is fully trusted; issues that require SuperAdmin are low priority.
- Holders of a leaked subscription link (it ends up in clients, logs, shared chats).

## Assets **[user: all of these matter]**

- Payments, orders, balance, commission balance, gift cards, coupons, refunds.
- User accounts, sessions (`personal_access_tokens`, sha256-hashed), API keys.
- Subscription tokens, node-level user UUIDs, per-node machine tokens, legacy HMAC node tokens.
- Admin/SuperAdmin control and the server: `APP_KEY` (Laravel crypto, UUID derivation, HMAC), DB/Redis credentials, backups (`mysqldump`, GCS), plugin/theme state under `/app/state`.
- Personal data: emails, tickets, refund evidence, Telegram IDs.

## Existing controls

- Route-role middleware (`require_user_api_access`/`require_admin_api_access`/`require_super_admin_api_access`), PATs must be `tokenable_type='App\\Models\\User'`, unexpired, abilities `"*"`, user not banned.
- Owner invariants re-carried in final SQL (`user_id`, `owner_user_id`, `assigned_admin_user_id`), `FOR UPDATE` + `rows_affected` checks (P-11).
- Node-plan entitlement computed live from `user_plan_subscriptions` + `v2_plan.node_ids` with `plan.owner_user_id = server_nodes.user_id` (P-10); direct sharing / trust-wide publish only for SuperAdmin owners (P-09).
- Per-node 256-bit machine tokens bound to `node_id`; legacy tokens HMAC(type+id) (P-07); node-level UUID derivation from `APP_KEY`.
- EPay notify accepts only success statuses and binds `payment_id`, route UUID, `pid`, exact amount (P-01).
- `ammonia` sanitization for plans/notices (`html_safety_support.rs`), text rendering for low-privilege content (P-12).
- Payment URL validation (`url_security_support.rs`, P-13); archive extraction checks (`archive_limit_support.rs`, P-08).
- One-time installer tickets via Redis `GETDEL` (P-14), Telegram bind codes (P-06), OAuth state nonce cookie, ban not liftable by Linux DO sync (P-15).

## Security Invariants

- **Authentication (CWE-287/613)**: every non-guest route sits behind `authenticate_route_account`; tokens are looked up by `sha256_hex` and must carry full abilities. Bans apply on every path (OAuth, Telegram, API keys). Bootstrap endpoints require the bootstrap token and refuse once installed. Gap to watch: any handler that parses `authorization` itself instead of the middleware.
- **Authorization / IDOR / tenant isolation (CWE-639/862/863)**: every dynamic ID from a user or node owner is resolved with the caller's `user_id`/`owner_user_id` in the final `UPDATE`/`DELETE`/`SELECT`, not only a prior check. Node owners never reach another owner's `server_nodes`, plans, tickets, refunds, audit rules, tokens or user lists. Admin handlers for users/limits/API keys/system/plugins/themes/backups re-check `SuperAdmin`.
- **Machine scope (CWE-284)**: a machine token reaches only its own `node_id` and only users entitled to that node; traffic for users not entitled to the node is dropped. Machine traffic misreporting for its own entitled users, including replay after the 10 s Redis dedup window (R-01), is accepted risk **[user]**.
- **Payment integrity (CWE-345/362)**: an order becomes paid only through a verified gateway notify bound to that order's payment method and exact amount; balance/commission/gift-card/coupon changes are atomic and idempotent under concurrent requests.
- **Injection (CWE-89)**: sqlx with bound parameters; dynamic SQL (e.g. `admin_v2/user.rs`, `tickets_support.rs`, `traffic_query_support.rs`) may only interpolate allow-listed column names and sort orders, never user strings. `backup_support.rs` runs `mysqldump` via `Command::new` with arguments, never a shell string.
- **XSS (CWE-79)**: content from users or node owners (tickets, node names, plan descriptions, refund evidence, error text) must not reach `v-html`/`innerHTML`/`dangerouslySetInnerHTML` in admin or other users' sessions unless passed through `ammonia`; the shipped admin/theme bundles in `rust-gateway/resources/public` are part of this surface.
- **SSRF and egress (CWE-918)**: user-supplied URLs are only returned to the browser (R-02); the gateway's outbound `backend_client` targets admin-configured endpoints (Telegram API base, LLM risk-review endpoint). Configuring those is a SuperAdmin action **[user: trusted]**; any path where a user or node owner controls an outbound URL is a gap.
- **Path traversal / file handling (CWE-22/434)**: static and theme files are canonicalized under their base (`static_files.rs`); plugin/theme uploads follow P-08 rules; staging stays in `/app/state/.plugin-*`/`.theme-*`.
- **Secrets and crypto (CWE-200/330/208)**: tokens use CSPRNG; comparisons use `secure_compare_support.rs`; credentials responses are `private, no-store`; subscription and installer responses are marked private. `APP_KEY` never leaves the server.
- **Client IP trust (CWE-348)**: `X-Forwarded-For`/`X-Real-IP` are used for subscribe and register rate limits; the proxy sets them **[user]** and port 8000 is not public, so spoofing is limited to what the proxy forwards.
- **Webhook verification (CWE-347)**: payment notify routes verify the gateway signature; Telegram webhook verifies the secret token header.

## Out of scope / accepted risk **[user]**

- Node machines misreporting traffic for their own entitled users (R-01).
- Issues that need a SuperAdmin account.
- Direct access to port 8000 (not public).

## Open questions

- Which payment plugins are enabled in production (EPay only, or also AlipayF2f/Btcpay/CoinPayments/Coinbase/Mgate)?
- Does the reverse proxy overwrite or append `X-Forwarded-For`?
