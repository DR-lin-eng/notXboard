# Rust Schema Baseline

本文件记录当前用于推进 Rust 接管初始化链的数据库基线证据。

## 生成时间

- 日期：2026-05-26
- 方式：在 Docker 中使用临时数据库 `notxboard_schema_snapshot` 执行完整 Laravel migration

## 验证命令

```bash
docker exec notxboard-mysql-1 mysql -uroot -pchange-me-root -e "DROP DATABASE IF EXISTS notxboard_schema_snapshot; CREATE DATABASE notxboard_schema_snapshot CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci; GRANT ALL PRIVILEGES ON notxboard_schema_snapshot.* TO 'notxboard'@'%'; FLUSH PRIVILEGES;"
APP_PORT=18000 docker compose run --rm -e DB_DATABASE=notxboard_schema_snapshot php php artisan migrate --force
docker exec notxboard-mysql-1 sh -lc "exec mysqldump -uroot -pchange-me-root --no-data --skip-comments --skip-set-charset --no-tablespaces --routines notxboard_schema_snapshot" > /private/tmp/notxboard-schema-snapshot.sql
docker exec notxboard-mysql-1 mysql -N -uroot -pchange-me-root -D notxboard_schema_snapshot -e "SHOW TABLES"
```

## 结果

- 完整 Laravel migration 已在临时数据库中执行成功
- 导出的 schema SQL 位于 `/private/tmp/notxboard-schema-snapshot.sql`
- 当前导出的 schema 文件约 `709` 行

## 基线表清单

```text
audit_logs
audit_rules
failed_jobs
migrations
node_traffic_records
order_refund_evidences
order_refund_requests
order_refund_votes
personal_access_tokens
server_nodes
sponsor_donations
tcping_agents
tcping_alerts
tcping_samples
user_ban_records
user_group_limits
user_individual_limits
user_node_access
user_node_blacklist
user_node_plan_access
user_online_sessions
user_payment_profiles
user_plan_subscriptions
user_risk_reviews
user_traffic_usage_logs
v2_commission_log
v2_coupon
v2_gift_card_code
v2_gift_card_template
v2_gift_card_usage
v2_invite_code
v2_knowledge
v2_log
v2_mail_log
v2_notice
v2_order
v2_payment
v2_plan
v2_plugins
v2_server
v2_server_group
v2_server_route
v2_settings
v2_stat
v2_stat_server
v2_stat_user
v2_ticket
v2_ticket_message
v2_traffic_reset_logs
v2_user
```

## 当前 Rust 侧状态

- Rust 网关已提供最小初始化接口：
  - `GET /bootstrap/status`
  - `POST /bootstrap/minimal`
- Rust 网关已提供完整 schema baseline 初始化接口：
  - `POST /bootstrap/full`
- 当前最小初始化仅覆盖：
  - `migrations`
  - `v2_settings`
  - `v2_user`
  - 最基础设置项和管理员账号
- `POST /bootstrap/full` 已在隔离数据库 `notxboard_rust_full_snapshot` 上验证通过：
  - 关键业务表已创建：`v2_plan`、`v2_order`、`server_nodes`、`v2_ticket`、`v2_settings`、`v2_user`
  - `migrations` 表记录数为 `76`
  - `bootstrap_mode` 已写为 `rust-full-schema`
- 当前剩余缺口是 seeder、默认插件安装和更高层初始化语义；后续 Rust 接管完整初始化时，应以本基线继续扩展，而不是继续长期依赖 PHP 运行时执行全部 migration
