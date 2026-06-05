/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;
DROP TABLE IF EXISTS `audit_logs`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `audit_logs` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `user_id` int NOT NULL COMMENT '用户 ID',
  `node_id` bigint unsigned NOT NULL COMMENT '节点 ID',
  `rule_id` bigint unsigned DEFAULT NULL COMMENT '触发的规则 ID',
  `ip_address` varchar(45) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '源 IP 地址',
  `target_domain` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '目标域名',
  `target_protocol` varchar(50) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '目标协议',
  `action_taken` enum('blocked','allowed','logged') COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '执行的动作',
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `audit_logs_rule_id_foreign` (`rule_id`),
  KEY `audit_logs_user_id_index` (`user_id`),
  KEY `audit_logs_node_id_index` (`node_id`),
  KEY `audit_logs_created_at_index` (`created_at`),
  KEY `audit_logs_action_taken_index` (`action_taken`),
  CONSTRAINT `audit_logs_node_id_foreign` FOREIGN KEY (`node_id`) REFERENCES `server_nodes` (`id`) ON DELETE CASCADE,
  CONSTRAINT `audit_logs_rule_id_foreign` FOREIGN KEY (`rule_id`) REFERENCES `audit_rules` (`id`) ON DELETE SET NULL,
  CONSTRAINT `audit_logs_user_id_foreign` FOREIGN KEY (`user_id`) REFERENCES `v2_user` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `audit_rules`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `audit_rules` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `node_id` bigint unsigned NOT NULL COMMENT '关联节点 ID',
  `rule_type` enum('domain','protocol','ip') COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '规则类型',
  `rule_pattern` varchar(500) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '规则模式 (正则表达式或具体值)',
  `action` enum('block','allow','log') COLLATE utf8mb4_unicode_ci NOT NULL DEFAULT 'block' COMMENT '动作',
  `is_active` tinyint(1) NOT NULL DEFAULT '1' COMMENT '是否启用',
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `audit_rules_node_id_index` (`node_id`),
  KEY `audit_rules_rule_type_index` (`rule_type`),
  KEY `audit_rules_is_active_index` (`is_active`),
  CONSTRAINT `audit_rules_node_id_foreign` FOREIGN KEY (`node_id`) REFERENCES `server_nodes` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `failed_jobs`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `failed_jobs` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `connection` text COLLATE utf8mb4_unicode_ci NOT NULL,
  `queue` text COLLATE utf8mb4_unicode_ci NOT NULL,
  `payload` longtext COLLATE utf8mb4_unicode_ci NOT NULL,
  `exception` longtext COLLATE utf8mb4_unicode_ci NOT NULL,
  `failed_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `migrations`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `migrations` (
  `id` int unsigned NOT NULL AUTO_INCREMENT,
  `migration` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL,
  `batch` int NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=77 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `node_traffic_records`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `node_traffic_records` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `user_id` int NOT NULL COMMENT '用户 ID',
  `node_id` bigint unsigned NOT NULL COMMENT '节点 ID',
  `upload_traffic` bigint unsigned NOT NULL DEFAULT '0' COMMENT '上传流量 (KB)',
  `download_traffic` bigint unsigned NOT NULL DEFAULT '0' COMMENT '下载流量 (KB)',
  `record_date` date NOT NULL COMMENT '记录日期',
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `unique_user_node_date` (`user_id`,`node_id`,`record_date`),
  KEY `node_traffic_records_user_id_index` (`user_id`),
  KEY `node_traffic_records_node_id_index` (`node_id`),
  KEY `node_traffic_records_record_date_index` (`record_date`),
  KEY `idx_node_traffic_records_node_date` (`node_id`,`record_date`),
  KEY `idx_node_traffic_records_user_date` (`user_id`,`record_date`),
  CONSTRAINT `node_traffic_records_node_id_foreign` FOREIGN KEY (`node_id`) REFERENCES `server_nodes` (`id`) ON DELETE CASCADE,
  CONSTRAINT `node_traffic_records_user_id_foreign` FOREIGN KEY (`user_id`) REFERENCES `v2_user` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `order_refund_evidences`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `order_refund_evidences` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `refund_request_id` bigint unsigned NOT NULL,
  `user_id` bigint unsigned NOT NULL,
  `role` varchar(20) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT 'user|admin|member|super_admin',
  `content` text COLLATE utf8mb4_unicode_ci NOT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `order_refund_evidences_refund_request_id_index` (`refund_request_id`),
  CONSTRAINT `order_refund_evidences_refund_request_id_foreign` FOREIGN KEY (`refund_request_id`) REFERENCES `order_refund_requests` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `order_refund_requests`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `order_refund_requests` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `order_id` bigint unsigned NOT NULL,
  `trade_no` varchar(36) COLLATE utf8mb4_unicode_ci NOT NULL,
  `user_id` bigint unsigned NOT NULL,
  `plan_id` bigint unsigned NOT NULL,
  `assigned_admin_user_id` bigint unsigned DEFAULT NULL,
  `status` varchar(20) COLLATE utf8mb4_unicode_ci NOT NULL DEFAULT 'pending',
  `reason` text COLLATE utf8mb4_unicode_ci,
  `gateway_amount` int NOT NULL DEFAULT '0' COMMENT 'EPay original amount (cents)',
  `gateway_trade_no` varchar(64) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT 'EPay trade_no (gateway)',
  `epay_pid` varchar(64) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `epay_url` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `epay_key_encrypted` text COLLATE utf8mb4_unicode_ci,
  `used_kb` bigint DEFAULT NULL,
  `allowance_kb` bigint DEFAULT NULL,
  `refund_amount` int DEFAULT NULL COMMENT 'Refund to user (cents)',
  `charged_amount` int DEFAULT NULL COMMENT 'Fee charged for used traffic (cents)',
  `balance_refunded_amount` int NOT NULL DEFAULT '0',
  `gateway_refunded_amount` int NOT NULL DEFAULT '0',
  `site_balance_fallback_amount` int NOT NULL DEFAULT '0',
  `gateway_refund_pending_amount` int NOT NULL DEFAULT '0',
  `gateway_refund_started_at` timestamp NULL DEFAULT NULL,
  `voting_ends_at` timestamp NULL DEFAULT NULL,
  `refunded_at` timestamp NULL DEFAULT NULL,
  `resolved_at` timestamp NULL DEFAULT NULL,
  `resolved_by_user_id` bigint unsigned DEFAULT NULL,
  `decision` varchar(20) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uniq_order_refund` (`order_id`),
  KEY `order_refund_requests_user_id_status_index` (`user_id`,`status`),
  KEY `order_refund_requests_assigned_admin_user_id_status_index` (`assigned_admin_user_id`,`status`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `order_refund_votes`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `order_refund_votes` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `refund_request_id` bigint unsigned NOT NULL,
  `user_id` bigint unsigned NOT NULL,
  `vote` varchar(10) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT 'approve|deny',
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uniq_refund_vote` (`refund_request_id`,`user_id`),
  KEY `order_refund_votes_refund_request_id_index` (`refund_request_id`),
  CONSTRAINT `order_refund_votes_refund_request_id_foreign` FOREIGN KEY (`refund_request_id`) REFERENCES `order_refund_requests` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `personal_access_tokens`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `personal_access_tokens` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `tokenable_type` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL,
  `tokenable_id` bigint unsigned NOT NULL,
  `name` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL,
  `token` varchar(64) COLLATE utf8mb4_unicode_ci NOT NULL,
  `abilities` text COLLATE utf8mb4_unicode_ci,
  `last_used_at` timestamp NULL DEFAULT NULL,
  `expires_at` timestamp NULL DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `personal_access_tokens_token_unique` (`token`),
  KEY `personal_access_tokens_tokenable_type_tokenable_id_index` (`tokenable_type`,`tokenable_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `server_nodes`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `server_nodes` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `user_id` int NOT NULL COMMENT '服务器提供者 ID',
  `name` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '节点名称',
  `host` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '服务器地址',
  `port` int NOT NULL COMMENT '端口',
  `service_port` int DEFAULT NULL COMMENT '服务端口(下发给节点，空则等于访问端口)',
  `protocol` varchar(32) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '协议类型',
  `location_code` varchar(16) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `location_name` varchar(128) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `settings` json DEFAULT NULL COMMENT '节点配置',
  `traffic_limit` bigint unsigned NOT NULL DEFAULT '0' COMMENT '流量限制 (KB)',
  `traffic_used` bigint unsigned NOT NULL DEFAULT '0' COMMENT '已使用流量 (KB)',
  `traffic_multiplier` decimal(8,2) NOT NULL DEFAULT '1.00',
  `tcping_enabled` tinyint(1) NOT NULL DEFAULT '0',
  `tcping_host` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `tcping_port` int DEFAULT NULL,
  `tcping_interval_seconds` int NOT NULL DEFAULT '60',
  `tcping_timeout_ms` int NOT NULL DEFAULT '3000',
  `tcping_alert_after_seconds` int NOT NULL DEFAULT '300',
  `tcping_recover_after_seconds` int NOT NULL DEFAULT '120',
  `tcping_last_status` varchar(16) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `tcping_last_latency_ms` int DEFAULT NULL,
  `tcping_last_error` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `tcping_last_sampled_at` int DEFAULT NULL,
  `tcping_outage_since` int DEFAULT NULL,
  `tcping_recovered_since` int DEFAULT NULL,
  `access_control` json DEFAULT NULL COMMENT '访问控制配置',
  `status` enum('active','inactive','maintenance','deploying') COLLATE utf8mb4_unicode_ci NOT NULL DEFAULT 'inactive' COMMENT '节点状态',
  `v2bx_node_id` int DEFAULT NULL COMMENT 'V2bX 节点 ID',
  `v2bx_config` json DEFAULT NULL COMMENT 'V2bX 配置',
  `v2bx_token` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT 'V2bX 认证令牌',
  `device_limit` int NOT NULL DEFAULT '0' COMMENT '设备数限制',
  `connection_limit` int NOT NULL DEFAULT '0' COMMENT '连接数限制',
  `speed_limit_up` int NOT NULL DEFAULT '0' COMMENT '上传限速 (Mbps)',
  `speed_limit_down` int NOT NULL DEFAULT '0' COMMENT '下载限速 (Mbps)',
  `cross_node_ip_limit` int NOT NULL DEFAULT '0' COMMENT '跨节点IP限制',
  `concurrent_ip_limit` int NOT NULL DEFAULT '0' COMMENT '跨节点并发IP限制 (仅超管可配置)',
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `server_nodes_user_id_index` (`user_id`),
  KEY `server_nodes_status_index` (`status`),
  KEY `server_nodes_protocol_index` (`protocol`),
  KEY `server_nodes_v2bx_node_id_index` (`v2bx_node_id`),
  KEY `idx_server_nodes_v2bx_token` (`v2bx_token`),
  KEY `idx_server_nodes_v2bx_node_token` (`v2bx_node_id`,`v2bx_token`),
  KEY `idx_server_nodes_status_user` (`status`,`user_id`),
  CONSTRAINT `server_nodes_user_id_foreign` FOREIGN KEY (`user_id`) REFERENCES `v2_user` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `sponsor_donations`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `sponsor_donations` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `user_id` bigint unsigned DEFAULT NULL COMMENT '用户ID（可为空）',
  `trade_no` varchar(36) COLLATE utf8mb4_unicode_ci NOT NULL,
  `total_amount` int NOT NULL COMMENT '金额（分）',
  `payment_id` int DEFAULT NULL,
  `callback_no` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `status` int NOT NULL DEFAULT '0' COMMENT '0待支付 1已完成 2已取消',
  `paid_at` int DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `sponsor_donations_trade_no_unique` (`trade_no`),
  KEY `sponsor_donations_user_id_index` (`user_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `tcping_agents`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `tcping_agents` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `user_id` int NOT NULL,
  `name` varchar(128) COLLATE utf8mb4_unicode_ci NOT NULL,
  `location_code` varchar(16) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `location_name` varchar(128) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `location_province` varchar(64) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `token` varchar(96) COLLATE utf8mb4_unicode_ci NOT NULL,
  `is_enabled` tinyint(1) NOT NULL DEFAULT '1',
  `last_heartbeat_at` int DEFAULT NULL,
  `last_sync_at` int DEFAULT NULL,
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `tcping_agents_token_unique` (`token`),
  KEY `idx_tcping_agents_user_enabled` (`user_id`,`is_enabled`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `tcping_alerts`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `tcping_alerts` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `node_id` bigint unsigned NOT NULL,
  `user_id` int NOT NULL,
  `status` varchar(16) COLLATE utf8mb4_unicode_ci NOT NULL DEFAULT 'active',
  `started_at` int NOT NULL,
  `triggered_at` int NOT NULL,
  `recovered_at` int DEFAULT NULL,
  `latest_error` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`),
  KEY `idx_tcping_alerts_user_status` (`user_id`,`status`),
  KEY `idx_tcping_alerts_node_status` (`node_id`,`status`),
  KEY `idx_tcping_alerts_triggered` (`triggered_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `tcping_samples`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `tcping_samples` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `node_id` bigint unsigned NOT NULL,
  `agent_id` bigint unsigned DEFAULT NULL,
  `is_reachable` tinyint(1) NOT NULL DEFAULT '0',
  `latency_ms` int DEFAULT NULL,
  `is_timeout` tinyint(1) NOT NULL DEFAULT '0',
  `error_message` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `sampled_at` int NOT NULL,
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`),
  KEY `idx_tcping_samples_node_sampled` (`node_id`,`sampled_at`),
  KEY `idx_tcping_samples_agent_sampled` (`agent_id`,`sampled_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `user_ban_records`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `user_ban_records` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `user_id` int NOT NULL COMMENT '用户 ID',
  `admin_id` int DEFAULT NULL COMMENT '操作管理员 ID',
  `action` varchar(16) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT 'ban / unban',
  `reason` varchar(500) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '操作原因',
  `source` varchar(32) COLLATE utf8mb4_unicode_ci NOT NULL DEFAULT 'manual' COMMENT '来源',
  `context` text COLLATE utf8mb4_unicode_ci COMMENT '扩展上下文 JSON',
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`),
  KEY `idx_user_ban_records_user_created` (`user_id`,`created_at`),
  KEY `idx_user_ban_records_admin_created` (`admin_id`,`created_at`),
  KEY `idx_user_ban_records_action_created` (`action`,`created_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `user_group_limits`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `user_group_limits` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `trust_level` tinyint NOT NULL COMMENT '信任等级 (0-4)',
  `speed_limit_up` int NOT NULL DEFAULT '0' COMMENT '上传限速 (Mbps)',
  `speed_limit_down` int NOT NULL DEFAULT '0' COMMENT '下载限速 (Mbps)',
  `device_limit` int NOT NULL DEFAULT '0' COMMENT '设备数限制',
  `connection_limit` int NOT NULL DEFAULT '0' COMMENT '连接数限制',
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `user_group_limits_trust_level_unique` (`trust_level`),
  KEY `user_group_limits_trust_level_index` (`trust_level`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `user_individual_limits`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `user_individual_limits` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `user_id` int NOT NULL COMMENT '用户 ID',
  `speed_limit_up` int NOT NULL DEFAULT '0' COMMENT '上传限速 (Mbps, 0表示使用组配置)',
  `speed_limit_down` int NOT NULL DEFAULT '0' COMMENT '下载限速 (Mbps, 0表示使用组配置)',
  `device_limit` int NOT NULL DEFAULT '0' COMMENT '设备数限制 (0表示使用组配置)',
  `connection_limit` int NOT NULL DEFAULT '0' COMMENT '连接数限制 (0表示使用组配置)',
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `user_individual_limits_user_id_unique` (`user_id`),
  KEY `user_individual_limits_user_id_index` (`user_id`),
  CONSTRAINT `user_individual_limits_user_id_foreign` FOREIGN KEY (`user_id`) REFERENCES `v2_user` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `user_node_access`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `user_node_access` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `user_id` int NOT NULL COMMENT '用户 ID',
  `node_id` bigint unsigned NOT NULL COMMENT '节点 ID',
  `access_type` enum('individual','group') COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '访问类型',
  `granted_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '授权时间',
  PRIMARY KEY (`id`),
  KEY `idx_user_node_access_node_user` (`node_id`,`user_id`),
  UNIQUE KEY `unique_user_node` (`user_id`,`node_id`),
  KEY `user_node_access_node_id_foreign` (`node_id`),
  CONSTRAINT `user_node_access_node_id_foreign` FOREIGN KEY (`node_id`) REFERENCES `server_nodes` (`id`) ON DELETE CASCADE,
  CONSTRAINT `user_node_access_user_id_foreign` FOREIGN KEY (`user_id`) REFERENCES `v2_user` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `user_node_blacklist`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `user_node_blacklist` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `node_id` bigint unsigned NOT NULL COMMENT '节点 ID',
  `user_id` int NOT NULL COMMENT '用户 ID',
  `reason` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '拉黑原因',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uniq_node_user_blacklist` (`node_id`,`user_id`),
  KEY `user_node_blacklist_user_id_index` (`user_id`),
  CONSTRAINT `user_node_blacklist_node_id_foreign` FOREIGN KEY (`node_id`) REFERENCES `server_nodes` (`id`) ON DELETE CASCADE,
  CONSTRAINT `user_node_blacklist_user_id_foreign` FOREIGN KEY (`user_id`) REFERENCES `v2_user` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `user_node_plan_access`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `user_node_plan_access` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `user_id` int NOT NULL COMMENT '用户 ID',
  `node_id` bigint unsigned NOT NULL COMMENT '节点 ID',
  `plan_id` int NOT NULL COMMENT '套餐 ID',
  `granted_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '授权时间',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uniq_user_node_plan` (`user_id`,`node_id`,`plan_id`),
  KEY `user_node_plan_access_user_id_node_id_index` (`user_id`,`node_id`),
  KEY `user_node_plan_access_plan_id_index` (`plan_id`),
  KEY `idx_user_node_plan_access_user_plan_node` (`user_id`,`plan_id`,`node_id`),
  KEY `idx_user_node_plan_access_node_plan_user` (`node_id`,`plan_id`,`user_id`),
  CONSTRAINT `user_node_plan_access_node_id_foreign` FOREIGN KEY (`node_id`) REFERENCES `server_nodes` (`id`) ON DELETE CASCADE,
  CONSTRAINT `user_node_plan_access_plan_id_foreign` FOREIGN KEY (`plan_id`) REFERENCES `v2_plan` (`id`) ON DELETE CASCADE,
  CONSTRAINT `user_node_plan_access_user_id_foreign` FOREIGN KEY (`user_id`) REFERENCES `v2_user` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `user_online_sessions`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `user_online_sessions` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `user_id` int NOT NULL COMMENT '用户 ID',
  `node_id` bigint unsigned NOT NULL COMMENT '节点 ID',
  `ip_address` varchar(45) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT 'IP 地址',
  `user_agent` text COLLATE utf8mb4_unicode_ci COMMENT '用户代理',
  `connection_count` int NOT NULL DEFAULT '1' COMMENT '连接数',
  `upload_traffic` bigint unsigned NOT NULL DEFAULT '0' COMMENT '上传流量 (KB)',
  `download_traffic` bigint unsigned NOT NULL DEFAULT '0' COMMENT '下载流量 (KB)',
  `last_activity` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '最后活动时间',
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `unique_user_node_ip` (`user_id`,`node_id`,`ip_address`),
  KEY `user_online_sessions_user_id_index` (`user_id`),
  KEY `user_online_sessions_node_id_index` (`node_id`),
  KEY `user_online_sessions_ip_address_index` (`ip_address`),
  KEY `user_online_sessions_last_activity_index` (`last_activity`),
  KEY `idx_user_online_sessions_user_last_activity_ip` (`user_id`,`last_activity`,`ip_address`),
  CONSTRAINT `user_online_sessions_node_id_foreign` FOREIGN KEY (`node_id`) REFERENCES `server_nodes` (`id`) ON DELETE CASCADE,
  CONSTRAINT `user_online_sessions_user_id_foreign` FOREIGN KEY (`user_id`) REFERENCES `v2_user` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `user_payment_profiles`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `user_payment_profiles` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `user_id` int NOT NULL COMMENT '用户ID',
  `provider` varchar(32) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '支付协议/提供商，例如 epay',
  `pid` varchar(64) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT 'Client ID',
  `key_encrypted` text COLLATE utf8mb4_unicode_ci COMMENT 'Client Secret (encrypted)',
  `url` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT 'Gateway base url',
  `submit_path` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT 'Submit path',
  `use_post` tinyint(1) NOT NULL DEFAULT '1' COMMENT 'Use POST submit',
  `sitename` varchar(128) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT 'Optional site name',
  `device` varchar(128) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT 'Optional device tag',
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uniq_user_provider` (`user_id`,`provider`),
  KEY `user_payment_profiles_provider_index` (`provider`),
  CONSTRAINT `user_payment_profiles_user_id_foreign` FOREIGN KEY (`user_id`) REFERENCES `v2_user` (`id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `user_plan_subscriptions`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `user_plan_subscriptions` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `user_id` int NOT NULL COMMENT '用户 ID',
  `plan_id` int NOT NULL COMMENT '套餐 ID',
  `order_id` int NOT NULL COMMENT '来源订单 ID',
  `period` varchar(32) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '购买周期',
  `traffic_allowance_kb` bigint NOT NULL DEFAULT '0' COMMENT '该实例总流量额度(KB)',
  `used_traffic_kb` bigint NOT NULL DEFAULT '0' COMMENT '已使用流量(KB)',
  `started_at` int NOT NULL COMMENT '生效时间',
  `expired_at` int DEFAULT NULL COMMENT '到期时间，NULL 表示长期',
  `status` tinyint NOT NULL DEFAULT '1' COMMENT '1=active,2=expired,3=revoked',
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `user_plan_subscriptions_order_id_unique` (`order_id`),
  KEY `idx_user_status_expired` (`user_id`,`status`,`expired_at`),
  KEY `idx_plan_status_expired` (`plan_id`,`status`,`expired_at`),
  KEY `idx_user_plan_status` (`user_id`,`plan_id`,`status`),
  KEY `idx_ups_user_plan_status_expired` (`user_id`,`plan_id`,`status`,`expired_at`),
  KEY `idx_ups_plan_user_status_expired` (`plan_id`,`user_id`,`status`,`expired_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `user_risk_reviews`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `user_risk_reviews` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `user_id` int NOT NULL COMMENT '用户 ID',
  `source` varchar(32) COLLATE utf8mb4_unicode_ci NOT NULL DEFAULT 'shared_ip' COMMENT '风险来源',
  `shared_ip` varchar(45) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '命中的共享 IP',
  `matched_user_count` int NOT NULL DEFAULT '0' COMMENT '共享 IP 命中的用户数',
  `matched_user_ids` text COLLATE utf8mb4_unicode_ci COMMENT '命中的用户 ID 列表 JSON',
  `risk_level` varchar(16) COLLATE utf8mb4_unicode_ci NOT NULL DEFAULT 'medium' COMMENT '风险等级',
  `suspicion_score` int unsigned NOT NULL DEFAULT '0' COMMENT '疑似滥用评分',
  `llm_model` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '审查使用的 LLM 模型',
  `summary` text COLLATE utf8mb4_unicode_ci COMMENT '审查摘要',
  `recommendation` text COLLATE utf8mb4_unicode_ci COMMENT '处理建议',
  `raw_response` mediumtext COLLATE utf8mb4_unicode_ci COMMENT 'LLM 原始响应',
  `evidence` mediumtext COLLATE utf8mb4_unicode_ci COMMENT '证据 JSON',
  `reviewed_at` int NOT NULL COMMENT '审查时间',
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`),
  KEY `idx_user_risk_reviews_user_reviewed` (`user_id`,`reviewed_at`),
  KEY `idx_user_risk_reviews_ip_reviewed` (`shared_ip`,`reviewed_at`),
  KEY `idx_user_risk_reviews_level_reviewed` (`risk_level`,`reviewed_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `user_traffic_usage_logs`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `user_traffic_usage_logs` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `user_id` int NOT NULL,
  `node_id` bigint unsigned NOT NULL,
  `raw_traffic_kb` bigint unsigned NOT NULL DEFAULT '0',
  `billed_traffic_kb` bigint unsigned NOT NULL DEFAULT '0',
  `multiplier_snapshot` decimal(8,2) NOT NULL DEFAULT '1.00',
  `source` varchar(32) COLLATE utf8mb4_unicode_ci NOT NULL DEFAULT 'push',
  `recorded_at` int NOT NULL,
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`),
  KEY `idx_utul_user_recorded` (`user_id`,`recorded_at`),
  KEY `idx_utul_node_recorded` (`node_id`,`recorded_at`),
  KEY `idx_utul_recorded` (`recorded_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_commission_log`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_commission_log` (
  `id` int NOT NULL AUTO_INCREMENT,
  `invite_user_id` int NOT NULL,
  `user_id` int NOT NULL,
  `trade_no` char(36) COLLATE utf8mb4_unicode_ci NOT NULL,
  `order_amount` int NOT NULL,
  `get_amount` int NOT NULL,
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uniq_v2_commission_log_trade_inviter_user` (`invite_user_id`,`user_id`,`trade_no`),
  KEY `v2_commission_log_created_at_index` (`created_at`),
  KEY `v2_commission_log_get_amount_index` (`get_amount`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_coupon`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_coupon` (
  `id` int NOT NULL AUTO_INCREMENT,
  `code` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL,
  `name` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL,
  `owner_user_id` int DEFAULT NULL COMMENT 'coupon publisher user id',
  `source_plan_id` int DEFAULT NULL COMMENT 'plan id that owns this coupon',
  `type` int NOT NULL,
  `value` int NOT NULL,
  `show` tinyint(1) NOT NULL DEFAULT '0',
  `limit_use` int DEFAULT NULL,
  `limit_use_with_user` int DEFAULT NULL,
  `limit_plan_ids` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `limit_period` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `started_at` int NOT NULL,
  `ended_at` int NOT NULL,
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`),
  KEY `v2_coupon_owner_user_id_index` (`owner_user_id`),
  KEY `v2_coupon_source_plan_id_index` (`source_plan_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_gift_card_code`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_gift_card_code` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `template_id` int NOT NULL COMMENT '模板ID',
  `code` varchar(32) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '兑换码',
  `batch_id` varchar(32) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '批次ID',
  `status` tinyint NOT NULL DEFAULT '0' COMMENT '状态：0未使用 1已使用 2已过期 3已禁用',
  `user_id` int DEFAULT NULL COMMENT '使用用户ID',
  `used_at` int DEFAULT NULL COMMENT '使用时间',
  `expires_at` int DEFAULT NULL COMMENT '过期时间',
  `actual_rewards` json DEFAULT NULL COMMENT '实际获得的奖励(用于盲盒等)',
  `usage_count` int NOT NULL DEFAULT '0' COMMENT '使用次数(分享卡)',
  `max_usage` int NOT NULL DEFAULT '1' COMMENT '最大使用次数',
  `metadata` json DEFAULT NULL COMMENT '额外数据',
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `v2_gift_card_code_code_unique` (`code`),
  KEY `idx_gift_code_template_id` (`template_id`),
  KEY `idx_gift_code_status` (`status`),
  KEY `idx_gift_code_user_id` (`user_id`),
  KEY `idx_gift_code_batch_id` (`batch_id`),
  KEY `idx_gift_code_expires_at` (`expires_at`),
  KEY `idx_gift_code_lookup` (`code`,`status`,`expires_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_gift_card_template`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_gift_card_template` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '礼品卡名称',
  `description` text COLLATE utf8mb4_unicode_ci COMMENT '礼品卡描述',
  `type` tinyint NOT NULL COMMENT '卡片类型：1余额 2有效期 3流量 4重置包 5套餐 6组合 7盲盒 8任务 9等级 10节日',
  `status` tinyint NOT NULL DEFAULT '1' COMMENT '状态：0禁用 1启用',
  `conditions` json DEFAULT NULL COMMENT '使用条件配置',
  `rewards` json NOT NULL COMMENT '奖励配置',
  `limits` json DEFAULT NULL COMMENT '限制条件',
  `special_config` json DEFAULT NULL COMMENT '特殊配置(节日时间、等级倍率等)',
  `icon` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '卡片图标',
  `background_image` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '背景图片URL',
  `theme_color` varchar(7) COLLATE utf8mb4_unicode_ci NOT NULL DEFAULT '#1890ff' COMMENT '主题色',
  `sort` int NOT NULL DEFAULT '0' COMMENT '排序',
  `admin_id` int NOT NULL COMMENT '创建管理员ID',
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`),
  KEY `idx_gift_template_type_status` (`type`,`status`),
  KEY `idx_gift_template_created_at` (`created_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_gift_card_usage`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_gift_card_usage` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `code_id` int NOT NULL COMMENT '兑换码ID',
  `template_id` int NOT NULL COMMENT '模板ID',
  `user_id` int NOT NULL COMMENT '使用用户ID',
  `invite_user_id` int DEFAULT NULL COMMENT '邀请人ID',
  `rewards_given` json NOT NULL COMMENT '实际发放的奖励',
  `invite_rewards` json DEFAULT NULL COMMENT '邀请人获得的奖励',
  `user_level_at_use` int DEFAULT NULL COMMENT '使用时用户等级',
  `plan_id_at_use` int DEFAULT NULL COMMENT '使用时用户套餐ID',
  `multiplier_applied` decimal(3,2) NOT NULL DEFAULT '1.00' COMMENT '应用的倍率',
  `ip_address` varchar(45) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '使用IP地址',
  `user_agent` text COLLATE utf8mb4_unicode_ci COMMENT '用户代理',
  `notes` text COLLATE utf8mb4_unicode_ci COMMENT '备注',
  `created_at` int NOT NULL,
  PRIMARY KEY (`id`),
  KEY `idx_gift_usage_code_id` (`code_id`),
  KEY `idx_gift_usage_template_id` (`template_id`),
  KEY `idx_gift_usage_user_id` (`user_id`),
  KEY `idx_gift_usage_invite_user_id` (`invite_user_id`),
  KEY `idx_gift_usage_created_at` (`created_at`),
  KEY `idx_gift_usage_user_usage` (`user_id`,`created_at`),
  KEY `idx_gift_usage_template_stats` (`template_id`,`created_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_invite_code`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_invite_code` (
  `id` int NOT NULL AUTO_INCREMENT,
  `user_id` int NOT NULL,
  `assigned_plan_id` int DEFAULT NULL COMMENT '邀请注册赠送的套餐 ID',
  `assigned_period` varchar(32) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '邀请注册赠送的套餐周期',
  `code` char(32) COLLATE utf8mb4_unicode_ci NOT NULL,
  `status` tinyint(1) NOT NULL DEFAULT '0',
  `pv` int NOT NULL DEFAULT '0',
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uniq_v2_invite_code_code` (`code`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_knowledge`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_knowledge` (
  `id` int NOT NULL AUTO_INCREMENT,
  `language` char(5) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '語言',
  `category` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '分類名',
  `title` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '標題',
  `body` text COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '內容',
  `sort` int DEFAULT NULL COMMENT '排序',
  `show` tinyint(1) NOT NULL DEFAULT '0' COMMENT '顯示',
  `created_at` int NOT NULL COMMENT '創建時間',
  `updated_at` int NOT NULL COMMENT '更新時間',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_log`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_log` (
  `id` int NOT NULL AUTO_INCREMENT,
  `title` text COLLATE utf8mb4_unicode_ci NOT NULL,
  `level` varchar(11) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `host` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `uri` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL,
  `method` varchar(11) COLLATE utf8mb4_unicode_ci NOT NULL,
  `data` text COLLATE utf8mb4_unicode_ci,
  `ip` varchar(128) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `context` text COLLATE utf8mb4_unicode_ci,
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_mail_log`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_mail_log` (
  `id` int NOT NULL AUTO_INCREMENT,
  `email` varchar(64) COLLATE utf8mb4_unicode_ci NOT NULL,
  `subject` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL,
  `template_name` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL,
  `error` text COLLATE utf8mb4_unicode_ci,
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_notice`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_notice` (
  `id` int NOT NULL AUTO_INCREMENT,
  `sort` int DEFAULT NULL,
  `title` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL,
  `content` text COLLATE utf8mb4_unicode_ci NOT NULL,
  `show` tinyint(1) NOT NULL DEFAULT '0',
  `popup` tinyint(1) NOT NULL DEFAULT '0',
  `author_user_id` int DEFAULT NULL,
  `scope_type` varchar(32) COLLATE utf8mb4_unicode_ci NOT NULL DEFAULT 'global',
  `target_plan_ids` text COLLATE utf8mb4_unicode_ci,
  `img_url` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `tags` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`),
  KEY `v2_notice_sort_index` (`sort`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_order`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_order` (
  `id` int NOT NULL AUTO_INCREMENT,
  `invite_user_id` int DEFAULT NULL,
  `user_id` int NOT NULL,
  `plan_id` int NOT NULL,
  `coupon_id` int DEFAULT NULL,
  `payment_id` int DEFAULT NULL,
  `type` int NOT NULL COMMENT '1新购2续费3升级',
  `period` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL,
  `trade_no` varchar(36) COLLATE utf8mb4_unicode_ci NOT NULL,
  `callback_no` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `epay_pid` varchar(64) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT 'EPay pid snapshot',
  `epay_url` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT 'EPay gateway url snapshot',
  `epay_key_encrypted` text COLLATE utf8mb4_unicode_ci COMMENT 'EPay key snapshot (encrypted)',
  `total_amount` int NOT NULL,
  `handling_amount` int DEFAULT NULL,
  `discount_amount` int DEFAULT NULL,
  `surplus_amount` int DEFAULT NULL COMMENT '剩余价值',
  `refund_amount` int DEFAULT NULL COMMENT '退款金额',
  `balance_amount` int DEFAULT NULL COMMENT '使用余额',
  `surplus_order_ids` text COLLATE utf8mb4_unicode_ci COMMENT '折抵订单',
  `status` int NOT NULL DEFAULT '0' COMMENT '0待支付1开通中2已取消3已完成4已折抵',
  `commission_status` int NOT NULL DEFAULT '0' COMMENT '0待确认1发放中2有效3无效',
  `commission_balance` int NOT NULL DEFAULT '0',
  `actual_commission_balance` int DEFAULT NULL COMMENT '实际支付佣金',
  `paid_at` int DEFAULT NULL,
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `trade_no` (`trade_no`),
  KEY `v2_order_created_at_index` (`created_at`),
  KEY `v2_order_status_index` (`status`),
  KEY `v2_order_total_amount_index` (`total_amount`),
  KEY `v2_order_commission_status_index` (`commission_status`),
  KEY `v2_order_invite_user_id_index` (`invite_user_id`),
  KEY `v2_order_commission_balance_index` (`commission_balance`),
  KEY `v2_order_updated_at_index` (`updated_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_payment`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_payment` (
  `id` int NOT NULL AUTO_INCREMENT,
  `uuid` char(32) COLLATE utf8mb4_unicode_ci NOT NULL,
  `payment` varchar(16) COLLATE utf8mb4_unicode_ci NOT NULL,
  `name` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL,
  `icon` text COLLATE utf8mb4_unicode_ci,
  `config` text COLLATE utf8mb4_unicode_ci NOT NULL,
  `notify_domain` varchar(128) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `handling_fee_fixed` int DEFAULT NULL,
  `handling_fee_percent` decimal(5,2) DEFAULT NULL,
  `enable` tinyint(1) NOT NULL DEFAULT '0',
  `sort` int DEFAULT NULL,
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_plan`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_plan` (
  `id` int NOT NULL AUTO_INCREMENT,
  `group_id` int unsigned DEFAULT NULL,
  `transfer_enable` bigint unsigned DEFAULT NULL COMMENT 'Transfer limit in bytes',
  `is_unlimited_traffic` tinyint(1) NOT NULL DEFAULT '0' COMMENT '是否无限流量',
  `name` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL,
  `scope` varchar(20) COLLATE utf8mb4_unicode_ci NOT NULL DEFAULT 'legacy' COMMENT 'legacy|node',
  `owner_user_id` bigint unsigned DEFAULT NULL COMMENT 'node plan publisher user id',
  `min_trust_level` tinyint unsigned DEFAULT NULL COMMENT 'Minimum Linux DO trust_level required to view/buy',
  `free_quota_gb_by_trust_level` json DEFAULT NULL COMMENT 'Free quota per trust_level (GB/month)',
  `node_ids` json DEFAULT NULL COMMENT 'server_nodes ids for node plan',
  `prices` json DEFAULT NULL COMMENT 'Store different duration prices and reset traffic price',
  `sell` tinyint(1) NOT NULL DEFAULT '0' COMMENT 'is sell',
  `speed_limit` int unsigned DEFAULT NULL COMMENT 'Speed limit in Mbps, 0 for unlimited',
  `device_limit` int unsigned DEFAULT NULL,
  `show` tinyint(1) NOT NULL DEFAULT '0',
  `visibility_scope` varchar(24) COLLATE utf8mb4_unicode_ci NOT NULL DEFAULT 'public' COMMENT 'public|link_only|assigned_only',
  `access_user_ids` json DEFAULT NULL COMMENT 'assigned users for this plan',
  `share_token` varchar(64) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT 'dedicated share token',
  `share_discount_type` tinyint unsigned NOT NULL DEFAULT '0' COMMENT '0:none,1:fixed,2:percent',
  `share_discount_value` int unsigned NOT NULL DEFAULT '0' COMMENT 'fixed:fen,percent:1-100',
  `sort` int DEFAULT NULL,
  `renew` tinyint(1) NOT NULL DEFAULT '1',
  `content` text COLLATE utf8mb4_unicode_ci,
  `tags` json DEFAULT NULL,
  `reset_traffic_method` int DEFAULT '0' COMMENT '重置流量方式:null跟随系统设置、0每月1号、1按月重置、2不重置、3每年1月1日、4按年重置',
  `capacity_limit` int unsigned DEFAULT '0' COMMENT '0 for unlimited',
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `v2_plan_share_token_unique` (`share_token`),
  KEY `v2_plan_scope_index` (`scope`),
  KEY `v2_plan_owner_user_id_index` (`owner_user_id`),
  KEY `v2_plan_visibility_scope_index` (`visibility_scope`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_plugins`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_plugins` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `name` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL,
  `code` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL,
  `type` varchar(20) COLLATE utf8mb4_unicode_ci NOT NULL DEFAULT 'feature' COMMENT '插件类型：feature功能性，payment支付型',
  `version` varchar(50) COLLATE utf8mb4_unicode_ci NOT NULL,
  `is_enabled` tinyint(1) NOT NULL DEFAULT '0',
  `config` json DEFAULT NULL,
  `installed_at` timestamp NULL DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `v2_plugins_code_unique` (`code`),
  KEY `v2_plugins_type_is_enabled_index` (`type`,`is_enabled`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_server`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_server` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `type` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT 'Server Type',
  `code` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT 'Server Spectific Key',
  `parent_id` int unsigned DEFAULT NULL COMMENT 'Parent Server ID',
  `group_ids` json DEFAULT NULL COMMENT 'Group ID',
  `route_ids` json DEFAULT NULL COMMENT 'Route ID',
  `name` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT 'Server Name',
  `rate` decimal(8,2) NOT NULL COMMENT 'Traffic Rate',
  `rate_time_enable` tinyint(1) NOT NULL DEFAULT '0' COMMENT '是否启用动态倍率',
  `rate_time_ranges` json DEFAULT NULL COMMENT '动态倍率规则',
  `tags` json DEFAULT NULL COMMENT 'Server Tags',
  `host` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT 'Server Host',
  `port` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT 'Client Port',
  `server_port` int NOT NULL COMMENT 'Server Port',
  `protocol_settings` json DEFAULT NULL,
  `show` tinyint(1) NOT NULL DEFAULT '0' COMMENT 'Show in List',
  `sort` int unsigned DEFAULT NULL,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `v2_server_type_code_unique` (`type`,`code`),
  KEY `v2_server_sort_index` (`sort`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_server_group`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_server_group` (
  `id` int NOT NULL AUTO_INCREMENT,
  `name` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL,
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_server_route`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_server_route` (
  `id` int NOT NULL AUTO_INCREMENT,
  `remarks` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL,
  `match` text COLLATE utf8mb4_unicode_ci NOT NULL,
  `action` varchar(11) COLLATE utf8mb4_unicode_ci NOT NULL,
  `action_value` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_settings`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_settings` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `group` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '设置分组',
  `type` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '设置类型',
  `name` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '设置名称',
  `value` mediumtext COLLATE utf8mb4_unicode_ci,
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `idx_setting_name` (`name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_stat`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_stat` (
  `id` int NOT NULL AUTO_INCREMENT,
  `record_at` int NOT NULL,
  `record_type` char(1) COLLATE utf8mb4_unicode_ci NOT NULL,
  `order_count` int NOT NULL COMMENT '订单数量',
  `order_total` int NOT NULL COMMENT '订单合计',
  `commission_count` int NOT NULL,
  `commission_total` int NOT NULL COMMENT '佣金合计',
  `paid_count` int NOT NULL,
  `paid_total` int NOT NULL,
  `register_count` int NOT NULL,
  `invite_count` int NOT NULL,
  `transfer_used_total` varchar(32) COLLATE utf8mb4_unicode_ci NOT NULL,
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `v2_stat_record_at_unique` (`record_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_stat_server`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_stat_server` (
  `id` int NOT NULL AUTO_INCREMENT,
  `server_id` int NOT NULL COMMENT '节点id',
  `server_type` char(11) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '节点类型',
  `u` bigint NOT NULL,
  `d` bigint NOT NULL,
  `record_type` char(1) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT 'd day m month',
  `record_at` int NOT NULL COMMENT '记录时间',
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `server_id_server_type_record_at` (`server_id`,`server_type`,`record_at`),
  KEY `server_id` (`server_id`),
  KEY `record_at` (`record_at`),
  KEY `v2_stat_server_server_id_index` (`server_id`),
  KEY `v2_stat_server_record_at_index` (`record_at`),
  KEY `v2_stat_server_u_index` (`u`),
  KEY `v2_stat_server_d_index` (`d`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_stat_user`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_stat_user` (
  `id` int NOT NULL AUTO_INCREMENT,
  `user_id` int NOT NULL,
  `server_rate` decimal(10,2) NOT NULL,
  `u` bigint NOT NULL,
  `d` bigint NOT NULL,
  `record_type` char(2) COLLATE utf8mb4_unicode_ci NOT NULL,
  `record_at` int NOT NULL,
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `server_rate_user_id_record_at` (`server_rate`,`user_id`,`record_at`),
  KEY `v2_stat_user_user_id_server_rate_record_at_index` (`user_id`,`server_rate`,`record_at`),
  KEY `v2_stat_user_u_index` (`u`),
  KEY `v2_stat_user_d_index` (`d`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_ticket`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_ticket` (
  `id` int NOT NULL AUTO_INCREMENT,
  `user_id` int NOT NULL,
  `node_id` bigint unsigned DEFAULT NULL COMMENT 'server_nodes id',
  `assigned_admin_user_id` bigint unsigned DEFAULT NULL COMMENT 'responsible admin user id',
  `subject` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL,
  `level` int NOT NULL,
  `status` int NOT NULL DEFAULT '0' COMMENT '0:已开启 1:已关闭',
  `reply_status` int NOT NULL DEFAULT '1' COMMENT '0:待回复 1:已回复',
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  `last_reply_user_id` int DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `v2_ticket_status_index` (`status`),
  KEY `v2_ticket_created_at_index` (`created_at`),
  KEY `v2_ticket_node_id_index` (`node_id`),
  KEY `v2_ticket_assigned_admin_user_id_index` (`assigned_admin_user_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_ticket_message`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_ticket_message` (
  `id` int NOT NULL AUTO_INCREMENT,
  `user_id` int NOT NULL,
  `ticket_id` int NOT NULL,
  `message` text COLLATE utf8mb4_unicode_ci NOT NULL,
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_traffic_reset_logs`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_traffic_reset_logs` (
  `id` bigint unsigned NOT NULL AUTO_INCREMENT,
  `user_id` bigint NOT NULL COMMENT '用户ID',
  `reset_type` varchar(50) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '重置类型',
  `reset_time` timestamp NOT NULL COMMENT '重置时间',
  `old_upload` bigint NOT NULL DEFAULT '0' COMMENT '重置前上传流量',
  `old_download` bigint NOT NULL DEFAULT '0' COMMENT '重置前下载流量',
  `old_total` bigint NOT NULL DEFAULT '0' COMMENT '重置前总流量',
  `new_upload` bigint NOT NULL DEFAULT '0' COMMENT '重置后上传流量',
  `new_download` bigint NOT NULL DEFAULT '0' COMMENT '重置后下载流量',
  `new_total` bigint NOT NULL DEFAULT '0' COMMENT '重置后总流量',
  `trigger_source` varchar(50) COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '触发来源',
  `metadata` json DEFAULT NULL COMMENT '额外元数据',
  `created_at` timestamp NULL DEFAULT NULL,
  `updated_at` timestamp NULL DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `idx_user_id` (`user_id`),
  KEY `idx_reset_time` (`reset_time`),
  KEY `idx_user_reset_time` (`user_id`,`reset_time`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
DROP TABLE IF EXISTS `v2_user`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `v2_user` (
  `id` int NOT NULL AUTO_INCREMENT,
  `invite_user_id` int DEFAULT NULL,
  `telegram_id` bigint DEFAULT NULL,
  `email` varchar(64) COLLATE utf8mb4_unicode_ci NOT NULL,
  `password` varchar(64) COLLATE utf8mb4_unicode_ci NOT NULL,
  `password_algo` char(10) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `password_salt` char(10) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `balance` int NOT NULL DEFAULT '0',
  `discount` int DEFAULT NULL,
  `commission_type` tinyint NOT NULL DEFAULT '0' COMMENT '0: system 1: period 2: onetime',
  `commission_rate` int DEFAULT NULL,
  `commission_balance` int NOT NULL DEFAULT '0',
  `t` int NOT NULL DEFAULT '0',
  `u` bigint NOT NULL DEFAULT '0',
  `d` bigint NOT NULL DEFAULT '0',
  `transfer_enable` bigint NOT NULL DEFAULT '0',
  `banned` tinyint(1) NOT NULL DEFAULT '0',
  `ban_reason` varchar(500) COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '当前封禁原因',
  `banned_at` int DEFAULT NULL COMMENT '封禁时间',
  `banned_by_admin_id` int DEFAULT NULL COMMENT '封禁操作管理员 ID',
  `is_admin` tinyint(1) NOT NULL DEFAULT '0',
  `is_super_admin` tinyint(1) NOT NULL DEFAULT '0',
  `last_login_at` int DEFAULT NULL,
  `is_staff` tinyint(1) NOT NULL DEFAULT '0',
  `last_login_ip` int DEFAULT NULL,
  `uuid` varchar(36) COLLATE utf8mb4_unicode_ci NOT NULL,
  `linux_do_id` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `linux_do_username` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `linux_do_name` varchar(255) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `linux_do_avatar` text COLLATE utf8mb4_unicode_ci,
  `trust_level` tinyint NOT NULL DEFAULT '0',
  `is_silenced` tinyint(1) NOT NULL DEFAULT '0',
  `external_ids` json DEFAULT NULL,
  `api_key` varchar(64) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `oauth_provider` varchar(50) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `oauth_access_token` text COLLATE utf8mb4_unicode_ci,
  `oauth_refresh_token` text COLLATE utf8mb4_unicode_ci,
  `oauth_expires_at` timestamp NULL DEFAULT NULL,
  `group_id` int DEFAULT NULL,
  `plan_id` int DEFAULT NULL,
  `speed_limit` int DEFAULT NULL,
  `remind_expire` tinyint DEFAULT '1',
  `remind_traffic` tinyint DEFAULT '1',
  `token` char(32) COLLATE utf8mb4_unicode_ci NOT NULL,
  `subscribe_path` varchar(32) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `subscribe_key` varchar(32) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `subscribe_salt` varchar(32) COLLATE utf8mb4_unicode_ci DEFAULT NULL,
  `subscription_credential_version` int NOT NULL DEFAULT '0',
  `last_subscription_credential_rotation_at` int DEFAULT NULL,
  `expired_at` bigint DEFAULT '0',
  `next_reset_at` int DEFAULT NULL COMMENT '下次流量重置时间',
  `last_reset_at` int DEFAULT NULL COMMENT '上次流量重置时间',
  `reset_count` int NOT NULL DEFAULT '0' COMMENT '流量重置次数',
  `device_limit` int DEFAULT NULL,
  `concurrent_ip_limit` int unsigned NOT NULL DEFAULT '3' COMMENT '同用户多IP并发限制',
  `online_count` int DEFAULT NULL,
  `last_online_at` timestamp NULL DEFAULT NULL,
  `remarks` text COLLATE utf8mb4_unicode_ci,
  `created_at` int NOT NULL,
  `updated_at` int NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `email` (`email`),
  UNIQUE KEY `v2_user_linux_do_id_unique` (`linux_do_id`),
  UNIQUE KEY `v2_user_api_key_unique` (`api_key`),
  KEY `v2_user_u_d_expired_at_group_id_banned_transfer_enable_index` (`u`,`d`,`expired_at`,`group_id`,`banned`,`transfer_enable`),
  KEY `v2_user_t_index` (`t`),
  KEY `v2_user_online_count_index` (`online_count`),
  KEY `v2_user_created_at_index` (`created_at`),
  KEY `v2_user_linux_do_id_index` (`linux_do_id`),
  KEY `v2_user_trust_level_index` (`trust_level`),
  KEY `v2_user_api_key_index` (`api_key`),
  KEY `v2_user_is_super_admin_index` (`is_super_admin`),
  KEY `idx_next_reset_at` (`next_reset_at`),
  KEY `v2_user_concurrent_ip_limit_index` (`concurrent_ip_limit`),
  KEY `idx_v2_user_token` (`token`),
  KEY `idx_v2_user_subscribe_path` (`subscribe_path`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;
