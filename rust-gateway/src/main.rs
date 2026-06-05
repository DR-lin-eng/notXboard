use axum::{
    body::{Body, HttpBody},
    extract::State,
    http::{HeaderMap, HeaderValue, Method, Request, Response, StatusCode, Uri},
};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
pub(crate) use chrono::{Datelike, TimeZone, Utc};
use http_body_util::BodyExt;
use hyper_util::client::legacy::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use sha1::{Digest as Sha1Digest, Sha1};
use sqlx::{MySqlPool, Row};
use sqlx::types::Json as SqlxJson;
use std::{
    collections::{HashMap, HashSet},
    env,
    io::Read,
    sync::{Arc, OnceLock},
    time::{Duration, Instant},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tracing::{error, info, warn};
use uuid::Uuid;

mod limits_support;
mod plan_capacity_support;
mod node_plans_support;
mod commission_support;
mod public_dashboard_support;
mod access_control_support;
mod api_key_support;
mod archive_limit_support;
mod db_retry_support;
mod epay_render_support;
mod legacy_traffic_support;
mod command_center_support;
mod backup_support;
mod horizon_metrics_support;
mod fallback_support;
mod http_support;
mod laravel_crypto_support;
mod refunds_support;
mod oauth_sync_support;
mod order_checkout_support;
mod order_epay_checkout_support;
mod order_scheduler_support;
mod order_creation_support;
mod payment_notify_support;
mod refund_notify_support;
mod user_read_support;
mod user_comm_config_support;
mod mail_support;
mod mass_mail_support;
mod mail_reminder_support;
mod notice_notify_support;
mod ops_alert_support;
mod telegram_notify_support;
mod telegram_security_support;
mod ban_support;
mod url_security_support;
mod sponsor_support;
mod tickets_support;
mod bootstrap_support;
mod builtin_plugin_support;
mod builtin_theme_support;
mod subscription_security_support;
mod secure_compare_support;
mod traffic_reset_support;
mod traffic_query_support;
mod admin_v2_stat_rank_support;
mod admin_v2_user_generate_support;
mod user_generation_support;
mod risk_review_support;
mod payment_support;
mod proxy_support;
mod router_support;
mod runtime;
mod runtime_paths;
mod scheduler_support;
mod settings_support;
mod stats_support;
mod subscribe_formats;
mod subscribe_support;
mod ticket_notify_support;
mod traffic_ingest_support;
mod uniproxy_support;
mod uniproxy_user_support;
mod static_files;
mod app_state;
mod theme_support;
mod theme_page_render;
mod tcping_agent_v1;
mod web_pages;
mod client_v1;
mod legacy_server_v1;
mod admin_v1;
mod admin_v2;
mod guest_v1;
mod oauth_v1;
mod passport_v1;
mod user_v1;

pub(crate) use limits_support::{
    AdminGroupLimitRow, EffectiveLimits, UserLimitProfile, admin_group_limit_defaults,
    admin_trust_level_display_name, load_effective_limits, load_effective_limits_by_profiles,
    load_user_individual_limit_map, serialize_admin_group_limit_row,
    validate_admin_group_limit_values,
};
pub(crate) use plan_capacity_support::{
    load_plan_active_subscription_counts, load_user_active_subscription_plan_ids,
    plan_has_remaining_capacity,
};
pub(crate) use commission_support::{
    load_commission_payout_config, load_existing_commission_inviter_ids_with_tx,
    load_inviter_chain_with_tx, CommissionPayoutConfig,
};
pub(crate) use public_dashboard_support::{
    load_public_average_bandwidth, load_public_overview_metrics, load_public_top_nodes,
    load_public_top_users,
};
pub(crate) use node_plans_support::{
    load_admin_node_options, load_all_node_plans, load_latest_owned_node_plan,
    load_node_plan_by_id_any, load_owned_node_plan_by_id, normalize_visibility_scope,
    parse_node_plan_mutation_input, resolve_node_plan_share_token,
};
pub(crate) use api_key_support::{
    api_key_has_valid_format, api_key_prefix, ensure_user_api_key,
    fill_missing_api_keys_for_users, load_user_api_key, replace_api_keys_for_users,
    reset_user_api_key,
};
pub(crate) use access_control_support::{
    access_node_last_report_at, clear_accessible_user_ids_cache,
    distinct_positive_user_ids, revoke_individual_node_access,
    get_accessible_user_ids_for_node, get_accessible_user_ids_for_owner_node,
    load_access_stats_node, load_accessible_nodes_for_user_payload,
    load_accessible_nodes_for_user_rows, load_authorized_user_ids_for_node,
    load_node_blacklisted_user_ids, replace_individual_node_access_with_tx,
    upsert_individual_node_access, user_can_access_tcping_node,
};
pub(crate) use refunds_support::{
    approve_refund_request_as_admin, calculate_refund_cents, calculate_refund_usage_kb,
    deny_refund_request_as_admin, finalize_single_expired_refund_voting,
    load_all_refund_requests, load_assigned_admin_refund_request_detail,
    load_assigned_admin_refund_request_detail_for_update, load_assigned_admin_refund_requests,
    load_expired_voting_refund_requests, load_refund_create_order_by_trade_no,
    load_refund_create_plan_by_id, load_refund_evidences, load_refund_request_detail_any,
    load_refund_votes, load_user_refund_request_detail, load_user_refund_requests,
    load_voting_refund_requests, serialize_refund_request_detail,
    serialize_refund_request_summary,
};
pub(crate) use order_scheduler_support::{
    load_schedulable_orders, SchedulableOrderRow,
};
pub(crate) use order_checkout_support::{
    prepare_user_checkout, PreparedUserCheckout, UserCheckoutSession,
};
pub(crate) use order_epay_checkout_support::resolve_checkout_epay_config;
pub(crate) use order_creation_support::{
    create_user_order, load_order_plan_by_id, CreateUserOrderInput, OrderPlanRow,
};
pub(crate) use user_read_support::{
    has_plan_capacity, load_active_user_plan_subscriptions, load_guest_available_plans,
    load_owned_node_plans, load_user_panel_stat_row, load_user_public_sellable_plans,
    load_user_visible_plan_by_id, load_user_visible_plan_by_share_token, plan_available_for_user,
};
pub(crate) use user_comm_config_support::{
    load_user_comm_config_snapshot,
};
pub(crate) use tickets_support::{
    assigned_admin_ticket_to_value, load_admin_ticket_by_id, load_admin_ticket_by_id_for_update,
    load_admin_tickets, load_assigned_admin_ticket_by_id,
    load_assigned_admin_ticket_by_id_for_update, load_assigned_admin_tickets,
    load_last_ticket_message, load_last_ticket_message_with_tx, load_ticket_messages,
    load_ticket_node_by_id, load_user_ticket_by_id, load_user_ticket_by_id_for_update,
    load_user_tickets, parse_csv_i64_list, serialize_admin_ticket_detail,
    serialize_admin_ticket_list_item, ticket_to_value, user_can_access_ticket_node,
    TicketRow,
};
pub(crate) use sponsor_support::{
    build_sponsor_epay_checkout_payload, load_pending_sponsor_donation_by_trade_no,
    load_sponsor_donation_by_trade_no, load_sponsor_epay_profile, load_sponsor_methods,
    mark_sponsor_donation_paid, serialize_sponsor_donation, serialize_sponsor_method,
};
pub(crate) use ban_support::{batch_ban_users, BannableUserRow};
pub(crate) use payment_support::{
    build_epay_checkout_payload, complete_processing_order_by_id, CheckoutOrderRow,
    compute_handling_amount, convert_period_to_legacy_field, epay_config_from_payment_method,
    find_user_checkout_order, load_payment_config, load_payment_method_by_id,
    load_order_epay_config_snapshot, load_payment_notify_method_by_uuid,
    load_user_epay_profile, mark_order_paid_processing, order_epay_snapshot_columns_ready,
    PaymentMethodRow,
    normalize_coupon_period, normalize_order_period, plan_price_for_period,
    resolve_order_type, validate_order_plan_for_user, verify_epay_notify_signature, EpayConfig,
};
pub(crate) use http_support::{
    build_cache_key, cached_plain_response, cached_plain_response_with_headers,
    fail_json_response, internal_error, is_hop_header, json_cached_response, json_error,
    json_status_response, json_value_response, parse_json_body, parse_query,
    success_cached_response, success_response_payload, try_cached_response,
};
pub(crate) use http::header::{AUTHORIZATION, CONTENT_TYPE, ETAG};
pub(crate) use proxy_support::{
    map_proxy_response, response_body_bytes,
};
pub(crate) use subscribe_support::{
    apply_protocol_prefixes_to_subscribe_servers, load_subscribe_servers_for_user,
    prepend_subscribe_info_nodes, subscribe_shadowsocks_password, traffic_convert,
    subscribe_user_is_available,
};
pub(crate) use subscribe_formats::{
    build_loon_payload, build_shadowsocks_sip008_payload, build_stash_yaml_payload,
    build_surge_config_payload, build_surfboard_config_payload,
};
pub(crate) use command_center_support::{
    build_command_center_audit_stream, build_command_center_node_snapshots,
    build_command_center_open_tickets, build_command_center_pending_refunds,
    build_command_center_protocol_distribution, build_command_center_region_distribution,
    build_command_center_system_status, build_command_center_tcping_agents,
    build_command_center_tcping_alert_stream, build_command_center_top_users,
    build_command_center_traffic_trend, build_command_center_watchlist,
    load_command_center_active_tcping_alerts, load_command_center_nodes,
    load_command_center_online_sessions, load_command_center_today_traffic,
    load_command_center_watch_samples, load_command_center_weekly_node_traffic,
    resolve_command_center_throughput,
};
pub(crate) use mass_mail_support::send_mass_mail;
pub(crate) use laravel_crypto_support::{decrypt_laravel_string, encrypt_laravel_string};
pub(crate) use payment_notify_support::notify_payment_success_by_order_id;
pub(crate) use refund_notify_support::{
    notify_refund_status_changed, notify_refund_vote_cast, notify_refund_vote_started,
};
pub(crate) use app_state::{
    AppState, AsyncQueueMetrics, CachedIdList, CachedLegacyAvailability, CachedResponse,
    CachedSetting, CountryHit, TrafficSnapshot,
};
pub(crate) use notice_notify_support::notify_notice_published;
pub(crate) use telegram_notify_support::{
    send_telegram_text_to_admins, send_telegram_text_to_chat_ids, send_telegram_text_to_super_admins,
};
pub(crate) use ticket_notify_support::{
    notify_ticket_created_to_assigned_admin, notify_ticket_reply_to_user,
};
pub(crate) use settings_support::{
    clear_settings_cache, get_setting_bool, get_setting_f64, get_setting_int, get_setting_string,
    get_setting_value, resolve_oauth_linux_do_available, resolve_pow_effective_difficulty,
    resolve_register_mode, setting_json_or_csv_array, upsert_setting_string,
    upsert_setting_string_with_metadata,
};
pub(crate) use stats_support::{
    load_admin_dashboard_summary, percentage_growth,
};
pub(crate) use subscription_security_support::{
    reset_many_user_security, reset_single_user_security,
};
pub(crate) use traffic_reset_support::{
    calculate_next_reset_at_for_user_plan, initialize_missing_user_reset_times,
    load_plan_reset_info, reset_due_user_traffic, reset_single_user_traffic,
};
pub(crate) use traffic_query_support::{
    load_node_traffic_stats_summary, load_node_traffic_user_profiles,
    load_node_users_traffic_rows, load_online_user_count_for_node, load_owned_server_node,
    load_owned_server_nodes, load_tcping_agent_for_user, load_tcping_agents_by_ids,
    load_tcping_agents_for_user, load_tcping_alerts_for_node, load_tcping_node_overview_row,
    load_tcping_samples_for_node_since, load_user_today_node_traffic_records,
    load_user_traffic_usage_logs,
};
pub(crate) use admin_v2_stat_rank_support::load_traffic_rank_names;
pub(crate) use admin_v2_user_generate_support::build_admin_user_generate_response;
pub(crate) use user_generation_support::{
    batch_insert_generated_users, create_generated_user, load_generated_user_defaults,
    load_generated_user_plan_context, prepare_random_generated_users, prepared_user_to_result,
};
pub(crate) use scheduler_support::spawn_background_scheduler;
pub(crate) use traffic_ingest_support::{
    batch_insert_node_traffic_records, batch_insert_user_traffic_usage_logs,
    batch_upsert_legacy_stat_user,
    batch_update_subscription_usage, batch_update_user_traffic_totals,
    load_primary_subscription_ids_for_node_users, AggregatedTrafficRow,
};
pub(crate) use uniproxy_support::{
    uniproxy_alive, uniproxy_alivelist, uniproxy_audit, uniproxy_config, uniproxy_push,
    uniproxy_status, uniproxy_user,
};

use crate::guest_v1::public::serialize_guest_plan;

#[derive(Clone, sqlx::FromRow)]
struct ServerNodeRow {
    id: u64,
    user_id: i64,
    name: String,
    host: String,
    port: i64,
    service_port: Option<i64>,
    protocol: String,
    settings: Option<SqlxJson<Value>>,
    access_control: Option<SqlxJson<Value>>,
    device_limit: i64,
    connection_limit: i64,
    speed_limit_down: i64,
    created_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Clone, sqlx::FromRow)]
struct LegacySubscribeServerRow {
    id: i64,
    server_type: String,
    parent_id: Option<i64>,
    group_ids: Option<SqlxJson<Value>>,
    name: String,
    host: String,
    port: String,
    server_port: i64,
    protocol_settings: Option<SqlxJson<Value>>,
    created_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Clone, sqlx::FromRow)]
struct UserRow {
    id: i64,
    token: Option<String>,
    group_id: Option<i64>,
    subscribe_key: Option<String>,
    subscribe_salt: Option<String>,
    uuid: Option<String>,
    u: Option<i64>,
    d: Option<i64>,
    transfer_enable: Option<i64>,
    expired_at: Option<i64>,
    trust_level: Option<i64>,
    banned: Option<i8>,
    is_super_admin: Option<i8>,
    is_silenced: Option<i8>,
    subscription_credential_version: Option<i64>,
}

#[derive(Clone, sqlx::FromRow)]
struct AuditRuleRow {
    id: u64,
    rule_type: String,
    rule_pattern: String,
    action: String,
    is_active: i8,
}

#[derive(Clone, sqlx::FromRow)]
struct PlanRow {
    id: i64,
    scope: String,
    owner_user_id: Option<u64>,
    min_trust_level: Option<u64>,
    free_quota_gb_by_trust_level: Option<SqlxJson<Value>>,
    node_ids: Option<SqlxJson<Value>>,
    group_id: Option<u64>,
    transfer_enable: Option<u64>,
    is_unlimited_traffic: bool,
    name: String,
    speed_limit: Option<u64>,
    show: bool,
    visibility_scope: String,
    access_user_ids: Option<SqlxJson<Value>>,
    share_token: Option<String>,
    sort: Option<i64>,
    renew: bool,
    content: Option<String>,
    prices: Option<SqlxJson<Value>>,
    reset_traffic_method: Option<i64>,
    capacity_limit: Option<u64>,
    sell: bool,
    device_limit: Option<u64>,
    tags: Option<SqlxJson<Value>>,
    created_at: i64,
    updated_at: i64,
    owner_email: Option<String>,
    owner_linux_do_username: Option<String>,
    owner_linux_do_name: Option<String>,
}

#[derive(Clone, sqlx::FromRow)]
struct PublicTopUserRow {
    id: i64,
    email: String,
    u: i64,
    d: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct PublicNodeTotalRow {
    server_id: i64,
    total: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct InviteCodeRow {
    id: i64,
    user_id: i64,
    code: String,
    status: i8,
}

#[derive(Clone, sqlx::FromRow)]
struct CouponRow {
    id: i64,
    code: String,
    name: String,
    owner_user_id: Option<i64>,
    source_plan_id: Option<i64>,
    type_field: i64,
    value: i64,
    show: bool,
    limit_use: Option<i64>,
    limit_use_with_user: Option<i64>,
    limit_plan_ids: Option<String>,
    limit_period: Option<String>,
    started_at: i64,
    ended_at: i64,
    created_at: i64,
    updated_at: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct NoticeRow {
    id: i64,
    sort: Option<i64>,
    title: String,
    content: String,
    show: i8,
    popup: i8,
    author_user_id: Option<i64>,
    scope_type: String,
    target_plan_ids: Option<String>,
    img_url: Option<String>,
    tags: Option<String>,
    created_at: i64,
    updated_at: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct LoginUserRow {
    id: i64,
    email: String,
    password: String,
    password_algo: Option<String>,
    password_salt: Option<String>,
    banned: i8,
    ban_reason: Option<String>,
    token: String,
    is_admin: i8,
    is_super_admin: i8,
    last_login_at: Option<i64>,
}

#[derive(Clone, sqlx::FromRow)]
struct BearerUserRow {
    id: i64,
    invite_user_id: Option<i64>,
    email: String,
    transfer_enable: i64,
    last_login_at: Option<i64>,
    created_at: i64,
    banned: i8,
    ban_reason: Option<String>,
    remind_expire: i8,
    remind_traffic: i8,
    expired_at: Option<i64>,
    balance: i64,
    commission_balance: i64,
    plan_id: Option<i64>,
    group_id: Option<i64>,
    discount: Option<i64>,
    commission_rate: Option<i64>,
    telegram_id: Option<i64>,
    uuid: String,
    is_admin: i8,
    is_super_admin: i8,
    trust_level: i64,
    is_silenced: i8,
    linux_do_id: Option<String>,
    linux_do_username: Option<String>,
    linux_do_name: Option<String>,
    linux_do_avatar: Option<String>,
    api_key: Option<String>,
    concurrent_ip_limit: u64,
    token: String,
    subscribe_path: Option<String>,
    subscribe_key: Option<String>,
    subscribe_salt: Option<String>,
    u: i64,
    d: i64,
    device_limit: Option<i64>,
    speed_limit: Option<i64>,
    next_reset_at: Option<i64>,
}

#[derive(Clone, sqlx::FromRow)]
struct UserQuotaRow {
    id: i64,
    order_id: i64,
    plan_id: i64,
    period: String,
    traffic_allowance_kb: i64,
    used_traffic_kb: i64,
    started_at: i64,
    expired_at: Option<i64>,
    plan_name: Option<String>,
    plan_scope: Option<String>,
    node_ids: Option<SqlxJson<Value>>,
}

#[derive(Clone, sqlx::FromRow)]
struct UserOrderRow {
    id: i64,
    user_id: i64,
    plan_id: i64,
    payment_id: Option<i64>,
    period: String,
    trade_no: String,
    total_amount: i64,
    handling_amount: Option<i64>,
    balance_amount: Option<i64>,
    refund_amount: Option<i64>,
    surplus_amount: Option<i64>,
    discount_amount: Option<i64>,
    type_field: i64,
    status: i64,
    surplus_order_ids: Option<String>,
    coupon_id: Option<i64>,
    created_at: i64,
    updated_at: i64,
    commission_status: i64,
    invite_user_id: Option<i64>,
    actual_commission_balance: Option<i64>,
    commission_balance: i64,
    paid_at: Option<i64>,
    callback_no: Option<String>,
    plan_name: Option<String>,
    plan_scope: Option<String>,
}

#[derive(Clone, sqlx::FromRow)]
struct RefundRequestRow {
    id: u64,
    order_id: u64,
    trade_no: String,
    user_id: u64,
    plan_id: u64,
    assigned_admin_user_id: Option<u64>,
    status: String,
    reason: Option<String>,
    gateway_amount: i64,
    gateway_trade_no: Option<String>,
    epay_pid: Option<String>,
    epay_url: Option<String>,
    epay_key_encrypted: Option<String>,
    used_kb: Option<i64>,
    allowance_kb: Option<i64>,
    refund_amount: Option<i64>,
    charged_amount: Option<i64>,
    balance_refunded_amount: i64,
    gateway_refunded_amount: i64,
    site_balance_fallback_amount: i64,
    gateway_refund_pending_amount: i64,
    gateway_refund_started_at: Option<chrono::DateTime<Utc>>,
    voting_ends_at: Option<chrono::DateTime<Utc>>,
    refunded_at: Option<chrono::DateTime<Utc>>,
    resolved_at: Option<chrono::DateTime<Utc>>,
    resolved_by_user_id: Option<u64>,
    decision: Option<String>,
    created_at: Option<chrono::DateTime<Utc>>,
    updated_at: Option<chrono::DateTime<Utc>>,
    plan_name: Option<String>,
    plan_scope: Option<String>,
    user_email: Option<String>,
}

#[derive(Clone, sqlx::FromRow)]
struct RefundEvidenceRow {
    id: u64,
    refund_request_id: u64,
    user_id: u64,
    role: String,
    content: String,
    created_at: Option<chrono::DateTime<Utc>>,
    updated_at: Option<chrono::DateTime<Utc>>,
    user_email: Option<String>,
}

#[derive(Clone, sqlx::FromRow)]
struct RefundVoteRow {
    id: u64,
    refund_request_id: u64,
    user_id: u64,
    vote: String,
    created_at: Option<chrono::DateTime<Utc>>,
    updated_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Clone, sqlx::FromRow)]
struct RefundCreateOrderRow {
    id: i64,
    trade_no: String,
    user_id: i64,
    plan_id: i64,
    status: i64,
    total_amount: i64,
    handling_amount: Option<i64>,
    balance_amount: Option<i64>,
    callback_no: Option<String>,
    epay_pid: Option<String>,
    epay_url: Option<String>,
    epay_key_encrypted: Option<String>,
    paid_at: Option<i64>,
    created_at: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct RefundCreatePlanRow {
    id: i64,
    scope: String,
    owner_user_id: Option<u64>,
    transfer_enable: Option<u64>,
    node_ids: Option<SqlxJson<Value>>,
}

#[derive(Clone, sqlx::FromRow)]
struct RefundExecutionOrderRow {
    id: i64,
    trade_no: String,
    user_id: i64,
    plan_id: i64,
    status: i64,
    total_amount: i64,
    handling_amount: Option<i64>,
    balance_amount: Option<i64>,
    callback_no: Option<String>,
    epay_pid: Option<String>,
    epay_url: Option<String>,
    epay_key_encrypted: Option<String>,
    paid_at: Option<i64>,
    created_at: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct RefundExecutionUserRow {
    id: i64,
    balance: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct InviteCodeFetchRow {
    user_id: i64,
    code: String,
    pv: i64,
    status: i8,
    assigned_plan_id: Option<i64>,
    assigned_period: Option<String>,
    assigned_plan_name: Option<String>,
    created_at: i64,
    updated_at: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct AvailableInvitePlanRow {
    id: i64,
    name: String,
    prices: Option<SqlxJson<Value>>,
}

#[derive(Clone, sqlx::FromRow)]
struct CommissionLogRow {
    id: i64,
    order_amount: i64,
    trade_no: String,
    get_amount: i64,
    created_at: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct TrafficResetLogAdminRow {
    id: u64,
    user_id: i64,
    user_email: Option<String>,
    reset_type: String,
    reset_time: chrono::DateTime<Utc>,
    old_upload: i64,
    old_download: i64,
    old_total: i64,
    new_upload: i64,
    new_download: i64,
    new_total: i64,
    trigger_source: String,
    metadata: Option<SqlxJson<Value>>,
    created_at: chrono::DateTime<Utc>,
}

#[derive(Clone, sqlx::FromRow)]
struct UserEpayProfileRow {
    pid: Option<String>,
    key_encrypted: Option<String>,
    url: Option<String>,
    submit_path: Option<String>,
    use_post: bool,
    sitename: Option<String>,
    device: Option<String>,
}

#[derive(Clone, sqlx::FromRow)]
struct GiftCardCodeLookupRow {
    id: u64,
    template_id: i64,
    code: String,
    batch_id: Option<String>,
    status: i64,
    user_id: Option<i64>,
    used_at: Option<i64>,
    expires_at: Option<i64>,
    actual_rewards: Option<SqlxJson<Value>>,
    usage_count: i64,
    max_usage: i64,
    metadata: Option<SqlxJson<Value>>,
    created_at: i64,
    updated_at: i64,
    template_name: String,
    template_description: Option<String>,
    template_type: i64,
    template_status: i8,
    template_conditions: Option<SqlxJson<Value>>,
    template_rewards: SqlxJson<Value>,
    template_limits: Option<SqlxJson<Value>>,
    template_special_config: Option<SqlxJson<Value>>,
    template_icon: Option<String>,
    template_background_image: Option<String>,
    template_theme_color: String,
    template_sort: i64,
    template_admin_id: i64,
    template_created_at: i64,
    template_updated_at: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct GiftCardUsageListRow {
    id: u64,
    created_at: i64,
    rewards_given: SqlxJson<Value>,
    invite_rewards: Option<SqlxJson<Value>>,
    multiplier_applied: String,
    code: Option<String>,
    template_name: Option<String>,
    template_type: Option<i64>,
    template_icon: Option<String>,
    template_theme_color: Option<String>,
}

#[derive(Clone, sqlx::FromRow)]
struct GiftCardUsageDetailRow {
    id: u64,
    code_id: i64,
    template_id: i64,
    user_id: i64,
    invite_user_id: Option<i64>,
    rewards_given: SqlxJson<Value>,
    invite_rewards: Option<SqlxJson<Value>>,
    user_level_at_use: Option<i64>,
    plan_id_at_use: Option<i64>,
    multiplier_applied: String,
    user_agent: Option<String>,
    notes: Option<String>,
    created_at: i64,
    code: Option<String>,
    template_name: Option<String>,
    template_description: Option<String>,
    template_type: Option<i64>,
    template_icon: Option<String>,
    template_theme_color: Option<String>,
    invite_user_id_ref: Option<i64>,
    invite_user_email: Option<String>,
}

#[derive(Clone, sqlx::FromRow)]
struct GiftCardPlanInfoRow {
    id: i64,
    scope: String,
    owner_user_id: Option<u64>,
    min_trust_level: Option<u64>,
    free_quota_gb_by_trust_level: Option<SqlxJson<Value>>,
    node_ids: Option<SqlxJson<Value>>,
    group_id: Option<u64>,
    transfer_enable: Option<u64>,
    is_unlimited_traffic: bool,
    name: String,
    speed_limit: Option<u64>,
    show: bool,
    visibility_scope: String,
    access_user_ids: Option<SqlxJson<Value>>,
    share_token: Option<String>,
    sort: Option<i64>,
    renew: bool,
    content: Option<String>,
    prices: Option<SqlxJson<Value>>,
    reset_traffic_method: Option<i64>,
    capacity_limit: Option<u64>,
    sell: bool,
    device_limit: Option<u64>,
    tags: Option<SqlxJson<Value>>,
    created_at: i64,
    updated_at: i64,
    owner_email: Option<String>,
    owner_linux_do_username: Option<String>,
    owner_linux_do_name: Option<String>,
}

#[derive(Clone)]
struct TrafficRow {
    user_id: i64,
    upload: i64,
    download: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct NodeAdminNodeRow {
    id: u64,
    user_id: i64,
    name: String,
}

#[derive(Clone, sqlx::FromRow)]
struct NodeUserTrafficRow {
    user_id: i64,
    upload: i64,
    download: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct NodeTrafficUserProfileRow {
    id: i64,
    email: String,
    linux_do_name: Option<String>,
    trust_level: i64,
    is_silenced: i8,
    banned: i8,
}

#[derive(Clone, sqlx::FromRow)]
struct AuditRuleOwnerRow {
    id: u64,
    node_id: u64,
    rule_type: String,
    rule_pattern: String,
    action: String,
    is_active: i8,
    created_at: Option<chrono::DateTime<Utc>>,
    updated_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Clone, sqlx::FromRow)]
struct AuditLogUserRow {
    id: u64,
    user_id: i64,
    node_id: u64,
    rule_id: Option<u64>,
    ip_address: String,
    target_domain: Option<String>,
    target_protocol: Option<String>,
    action_taken: String,
    created_at: Option<chrono::DateTime<Utc>>,
    updated_at: Option<chrono::DateTime<Utc>>,
    node_name: Option<String>,
    rule_type: Option<String>,
    rule_pattern: Option<String>,
    rule_action: Option<String>,
}

#[derive(Clone, sqlx::FromRow)]
struct TcpingAgentRow {
    id: u64,
    user_id: i64,
    name: String,
    location_code: Option<String>,
    location_name: Option<String>,
    location_province: Option<String>,
    token: String,
    is_enabled: bool,
    last_heartbeat_at: Option<i64>,
    last_sync_at: Option<i64>,
    created_at: i64,
    updated_at: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct TcpingSampleRow {
    id: u64,
    node_id: u64,
    agent_id: Option<u64>,
    is_reachable: bool,
    latency_ms: Option<i64>,
    is_timeout: bool,
    error_message: Option<String>,
    sampled_at: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct TcpingAlertRow {
    id: u64,
    node_id: u64,
    user_id: i64,
    status: String,
    started_at: i64,
    triggered_at: i64,
    recovered_at: Option<i64>,
    latest_error: Option<String>,
}

#[derive(Clone, sqlx::FromRow)]
struct ServerNodeMonitorRow {
    id: u64,
    name: String,
    host: String,
    protocol: String,
    created_at: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct TcpingNodeOverviewRow {
    id: u64,
    user_id: i64,
    name: String,
    host: String,
    port: i64,
    protocol: String,
    location_name: Option<String>,
    status: String,
    tcping_enabled: bool,
    tcping_host: Option<String>,
    tcping_port: Option<i64>,
    tcping_interval_seconds: i64,
    tcping_timeout_ms: i64,
    tcping_alert_after_seconds: i64,
    tcping_recover_after_seconds: i64,
    tcping_last_status: Option<String>,
    tcping_last_latency_ms: Option<i64>,
    tcping_last_error: Option<String>,
    tcping_last_sampled_at: Option<i64>,
}

#[derive(Clone, sqlx::FromRow)]
struct ServerNodeOwnerRow {
    id: u64,
    user_id: i64,
    name: String,
    host: String,
    port: i64,
    service_port: Option<i64>,
    protocol: String,
    location_code: Option<String>,
    location_name: Option<String>,
    settings: Option<SqlxJson<Value>>,
    traffic_limit: u64,
    traffic_used: u64,
    traffic_multiplier: String,
    access_control: Option<SqlxJson<Value>>,
    status: String,
    v2bx_node_id: Option<i64>,
    v2bx_config: Option<SqlxJson<Value>>,
    v2bx_token: Option<String>,
    device_limit: i64,
    connection_limit: i64,
    speed_limit_up: i64,
    speed_limit_down: i64,
    cross_node_ip_limit: i64,
    concurrent_ip_limit: i64,
    tcping_enabled: bool,
    tcping_host: Option<String>,
    tcping_port: Option<i64>,
    tcping_interval_seconds: i64,
    tcping_timeout_ms: i64,
    tcping_alert_after_seconds: i64,
    tcping_recover_after_seconds: i64,
    tcping_last_status: Option<String>,
    tcping_last_latency_ms: Option<i64>,
    tcping_last_error: Option<String>,
    tcping_last_sampled_at: Option<i64>,
    created_at: Option<chrono::DateTime<Utc>>,
    updated_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Clone, Default)]
struct ServerNodeMutationInput {
    user_id: Option<i64>,
    name: Option<String>,
    host: Option<String>,
    port: Option<i64>,
    service_port: Option<i64>,
    protocol: Option<String>,
    location_code: Option<String>,
    location_name: Option<String>,
    settings: Option<Map<String, Value>>,
    traffic_limit: Option<i64>,
    traffic_used: Option<i64>,
    traffic_multiplier: Option<f64>,
    access_control: Option<Map<String, Value>>,
    status: Option<String>,
    device_limit: Option<i64>,
    connection_limit: Option<i64>,
    speed_limit_up: Option<i64>,
    speed_limit_down: Option<i64>,
    cross_node_ip_limit: Option<i64>,
    concurrent_ip_limit: Option<i64>,
    tcping_host: Option<String>,
    tcping_port: Option<i64>,
    tcping_interval_seconds: Option<i64>,
    tcping_timeout_ms: Option<i64>,
    tcping_alert_after_seconds: Option<i64>,
    tcping_recover_after_seconds: Option<i64>,
}

#[derive(Clone, sqlx::FromRow)]
struct UserTrafficUsageLogRow {
    id: u64,
    user_id: i64,
    node_id: u64,
    raw_traffic_kb: i64,
    billed_traffic_kb: i64,
    multiplier_snapshot: String,
    source: String,
    recorded_at: i64,
    node_name: Option<String>,
    node_protocol: Option<String>,
    node_location_name: Option<String>,
}

#[derive(Clone, sqlx::FromRow)]
struct KnowledgeRow {
    id: i64,
    language: String,
    category: String,
    title: String,
    body: String,
    sort: Option<i64>,
    show: bool,
    created_at: i64,
    updated_at: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct StatTrafficLogRow {
    user_id: i64,
    u: i64,
    d: i64,
    record_at: i64,
    server_rate: f64,
}

struct PowChallengeCacheEntry {
    seed: String,
    base: String,
    difficulty: i64,
    issued_at: i64,
    expires_at: i64,
    token: String,
    ja3_hash: Option<String>,
    ip: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let runtime = runtime::build_runtime_config().await;
    let alive_session_rx = runtime.alive_session_rx;
    let legacy_submit_rx = runtime.legacy_submit_rx;
    let port = runtime.port;
    let push_traffic_rx = runtime.push_traffic_rx;
    let state = runtime.state;
    if env_bool("RUST_GATEWAY_OWNS_SCHEDULER", true) {
        spawn_background_scheduler(state.clone());
    } else {
        info!("rust background scheduler disabled by RUST_GATEWAY_OWNS_SCHEDULER");
    }
    crate::uniproxy_support::spawn_alive_session_worker(state.clone(), alive_session_rx);
    crate::legacy_traffic_support::spawn_legacy_submit_worker(state.clone(), legacy_submit_rx);
    crate::uniproxy_support::spawn_push_traffic_worker(state.clone(), push_traffic_rx);
    let app = router_support::build_router(state);
    let listener = TcpListener::bind(("0.0.0.0", port)).await.expect("bind listener");
    info!("notxboard-gateway listening on 0.0.0.0:{}", port);
    axum::serve(listener, app).await.expect("server error");
}

async fn healthz(
    State(state): State<Arc<AppState>>,
) -> Response<Body> {
    let query = "
        SELECT
            EXISTS(
                SELECT 1
                FROM information_schema.tables
                WHERE table_schema = DATABASE()
                  AND table_name = ?
                  AND table_type IN ('BASE TABLE', 'SYSTEM VERSIONED')
            ) AS table_exists
    ";
    for table_name in ["migrations", "v2_settings"] {
        let exists = sqlx::query_scalar::<_, i64>(query)
            .bind(table_name)
            .fetch_one(&state.db)
            .await;
        match exists {
            Ok(1) => {}
            Ok(_) | Err(_) => {
                return Response::builder()
                    .status(StatusCode::SERVICE_UNAVAILABLE)
                    .header("Cache-Control", "no-store")
                    .body(Body::from("not ready"))
                    .unwrap();
            }
        }
    }

    Response::builder()
        .status(StatusCode::OK)
        .header("Cache-Control", "no-store")
        .body(Body::from("ok"))
        .unwrap()
}

async fn subscribe_entry(
    State(state): State<Arc<AppState>>,
    axum::extract::Path((subscribe_key, token_or_path)): axum::extract::Path<(String, String)>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_subscribe_entry_response(&state, subscribe_key, token_or_path, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_subscribe_entry_response(
    state: &AppState,
    subscribe_key: String,
    token_or_path: String,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let configured_subscribe_path = get_setting_string(state, "subscribe_path", "s").await;
    if subscribe_key != configured_subscribe_path || token_or_path.is_empty() {
        return Err(json_error(StatusCode::NOT_FOUND, "Not found"));
    }

    let cache_key = format!("subscribe:{}?{}", token_or_path, uri.query().unwrap_or_default());
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }

    let params = parse_query(&uri);
    let is_token_path = token_or_path.len() == 32 && token_or_path.chars().all(|c| c.is_ascii_hexdigit());
    let user = if is_token_path {
        load_user_by_token(state, &token_or_path)
            .await
            .map_err(internal_error)?
            .ok_or_else(|| subscribe_auth_error(state, headers.get("x-forwarded-for").or_else(|| headers.get("x-real-ip")).and_then(|v| v.to_str().ok()), "token is error"))?
    } else {
        authenticate_subscribe_obfuscated(state, &token_or_path, &params, headers.get("x-forwarded-for").or_else(|| headers.get("x-real-ip")).and_then(|v| v.to_str().ok()))
            .await?
    };

    if !subscribe_user_is_available(state, &user).await.map_err(internal_error)? {
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header(CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(Body::from(""))
            .unwrap());
    }

    let mode = rust_subscribe_mode(&params, &headers).unwrap_or(RustSubscribeMode::General);
    let payload = build_rust_subscribe_payload(state, &user, &params, mode).await?;
    let response = cached_plain_response(state, cache_key, payload, Duration::from_secs(15), "text/plain; charset=utf-8");
    Ok(response)
}

async fn load_user_by_token(state: &AppState, token: &str) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        "SELECT id, token, group_id, subscribe_path, subscribe_key, subscribe_salt, uuid, u, d, transfer_enable, expired_at, trust_level, banned, is_super_admin, is_silenced, subscription_credential_version
         FROM v2_user
         WHERE token = ?
         LIMIT 1"
    )
    .bind(token)
    .fetch_optional(&state.db)
    .await
}

async fn load_user_by_subscribe_path(state: &AppState, subscribe_path: &str) -> Result<Option<UserRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRow>(
        "SELECT id, token, group_id, subscribe_path, subscribe_key, subscribe_salt, uuid, u, d, transfer_enable, expired_at, trust_level, banned, is_super_admin, is_silenced, subscription_credential_version
         FROM v2_user
         WHERE subscribe_path = ?
         LIMIT 1"
    )
    .bind(subscribe_path)
    .fetch_optional(&state.db)
    .await
}

async fn load_available_server_nodes_for_user(
    state: &AppState,
    user_id: i64,
    trust_level: i64,
    is_super_admin: bool,
) -> Result<Vec<ServerNodeRow>, sqlx::Error> {
    if is_super_admin {
        return sqlx::query_as::<_, ServerNodeRow>(
            "SELECT DISTINCT sn.id, sn.user_id, sn.name, sn.host, sn.port, sn.service_port, sn.protocol, sn.settings, sn.access_control, sn.device_limit, sn.connection_limit, sn.speed_limit_down, sn.created_at
             FROM server_nodes sn
             WHERE sn.status = 'active'
             ORDER BY sn.id"
        )
        .fetch_all(&state.db)
        .await;
    }

    let now = Utc::now().timestamp();
    let unlimited_allowance = 8_000_000_000_000_000i64;

    let rows = sqlx::query_as::<_, ServerNodeRow>(
        "SELECT DISTINCT sn.id, sn.user_id, sn.name, sn.host, sn.port, sn.service_port, sn.protocol, sn.settings, sn.access_control, sn.device_limit, sn.connection_limit, sn.speed_limit_down, sn.created_at
         FROM server_nodes sn
         WHERE sn.status = 'active'
           AND (
             sn.user_id = ?
             OR EXISTS (
               SELECT 1 FROM user_node_access ua
               WHERE ua.node_id = sn.id AND ua.user_id = ?
             )
             OR EXISTS (
               SELECT 1
               FROM user_node_plan_access upa
               JOIN user_plan_subscriptions ups
                 ON ups.user_id = upa.user_id
                AND ups.plan_id = upa.plan_id
               WHERE upa.user_id = ?
                 AND upa.node_id = sn.id
                 AND ups.status = 1
                 AND (ups.expired_at IS NULL OR ups.expired_at > ?)
                 AND (
                   ups.traffic_allowance_kb >= ?
                   OR ups.traffic_allowance_kb > ups.used_traffic_kb
                 )
             )
             OR (
               JSON_EXTRACT(sn.access_control, '$.min_trust_level') IS NOT NULL
               AND CAST(JSON_UNQUOTE(JSON_EXTRACT(sn.access_control, '$.min_trust_level')) AS SIGNED) <= ?
             )
           )
           AND NOT EXISTS (
             SELECT 1 FROM user_node_blacklist ub
             WHERE ub.node_id = sn.id AND ub.user_id = ?
           )
         ORDER BY sn.id"
    )
    .bind(user_id)
    .bind(user_id)
    .bind(user_id)
    .bind(now)
    .bind(unlimited_allowance)
    .bind(trust_level)
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    Ok(rows)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RustSubscribeMode {
    General,
    Shadowsocks,
    Shadowrocket,
    QuantumultX,
    Loon,
    Surge,
    Surfboard,
    Stash,
    Clash,
    ClashMeta,
    SingBox,
}

fn rust_subscribe_mode(query: &HashMap<String, String>, headers: &HeaderMap) -> Option<RustSubscribeMode> {
    let flag = query
        .get("flag")
        .cloned()
        .or_else(|| headers.get("user-agent").and_then(|v| v.to_str().ok()).map(|v| v.to_string()))
        .unwrap_or_default()
        .to_lowercase();

    match flag.as_str() {
        "" | "general" | "v2rayn" | "v2rayng" | "passwall" | "ssrplus" | "sagernet" => Some(RustSubscribeMode::General),
        "shadowsocks" => Some(RustSubscribeMode::Shadowsocks),
        "shadowrocket" => Some(RustSubscribeMode::Shadowrocket),
        "quantumult%20x" | "quantumult-x" => Some(RustSubscribeMode::QuantumultX),
        "loon" => Some(RustSubscribeMode::Loon),
        "surge" => Some(RustSubscribeMode::Surge),
        "surfboard" => Some(RustSubscribeMode::Surfboard),
        "stash" => Some(RustSubscribeMode::Stash),
        "clash" => Some(RustSubscribeMode::Clash),
        "clashmeta" | "clash-meta" | "meta" | "verge" | "flclash" | "nekobox" | "clashmetaforandroid" => Some(RustSubscribeMode::ClashMeta),
        "sing-box" | "hiddify" | "sfm" => Some(RustSubscribeMode::SingBox),
        _ => Some(RustSubscribeMode::General),
    }
}

async fn build_rust_subscribe_payload(
    state: &AppState,
    user: &UserRow,
    query: &HashMap<String, String>,
    mode: RustSubscribeMode,
) -> Result<bytes::Bytes, Response<Body>> {
    let mut servers = load_subscribe_servers_for_user(state, user)
        .await
        .map_err(internal_error)?;
    if servers.is_empty() {
        return Ok(bytes::Bytes::from_static(b""));
    }

    let requested_types = parse_requested_types(query.get("types").map(|s| s.as_str()));
    let filter_keywords = parse_filter_keywords(query.get("filter").map(|s| s.as_str()));
    let rotate_credentials = get_setting_bool(state, "rotate_subscription_credentials_daily", false).await;
    let uuid = effective_uuid(
        user.uuid.as_deref().unwrap_or_default(),
        user.subscription_credential_version.unwrap_or(0),
        rotate_credentials,
    );
    let status_header = match mode {
        RustSubscribeMode::Shadowrocket => Some(format!(
            "STATUS=🚀↑:{}GB,↓:{}GB,TOT:{}GB💡Expires:{}\r\n",
            traffic_to_gb(user.u.unwrap_or(0)),
            traffic_to_gb(user.d.unwrap_or(0)),
            traffic_to_gb(user.transfer_enable.unwrap_or(0)),
            user.expired_at
                .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "长期有效".to_string())
        )),
        _ => None,
    };

    let original_server_count = servers.len();
    servers.retain(|server| {
        let normalized_type = normalize_type(&server.protocol).unwrap_or_default();
        if !requested_types.is_empty() && !requested_types.contains(&normalized_type) {
            return false;
        }
        if filter_keywords.is_empty() {
            return true;
        }
        filter_keywords.iter().any(|keyword| {
            server
                .host
                .to_lowercase()
                .contains(keyword)
                || server
                    .name
                    .to_lowercase()
                    .contains(keyword)
                || normalize_type(&server.protocol)
                    .unwrap_or_default()
                    .contains(keyword)
        })
    });

    let filtered_out_count = (original_server_count as i64 - servers.len() as i64).max(0);
    if get_setting_bool(state, "show_protocol_to_server_enable", false).await {
        apply_protocol_prefixes_to_subscribe_servers(&mut servers);
    }
    let show_info_to_server = get_setting_bool(state, "show_info_to_server_enable", false).await;
    prepend_subscribe_info_nodes(&mut servers, user, filtered_out_count, state, show_info_to_server).await;

    let mut buffer = String::new();
    if let Some(header) = status_header {
        buffer.push_str(&header);
    }
    for server in &servers {
        let normalized_type = normalize_type(&server.protocol).unwrap_or_default();
        let name = server.name.clone();
        let settings = normalized_protocol_settings(
            &normalized_type,
            server
                .settings
                .as_ref()
                .and_then(|json| json.0.as_object().cloned())
                .unwrap_or_default(),
        );

        let line = match mode {
            RustSubscribeMode::General => match normalized_type.as_str() {
                "vmess" => build_general_vmess(&uuid, &server, &name, &settings),
                "vless" => build_general_vless(&uuid, &server, &name, &settings),
                "shadowsocks" => build_general_shadowsocks(&uuid, &server, &name, &settings, &state.app_key),
                "trojan" => build_general_trojan(&uuid, &server, &name, &settings),
                "hysteria" => build_general_hysteria(&uuid, &server, &name, &settings),
                "socks" => build_general_socks(&uuid, &server, &name),
                _ => String::new(),
            },
            RustSubscribeMode::Shadowsocks => String::new(),
            RustSubscribeMode::Shadowrocket => match normalized_type.as_str() {
                "vmess" => build_shadowrocket_vmess(&uuid, &server, &name, &settings),
                "vless" => build_shadowrocket_vless(&uuid, &server, &name, &settings),
                "shadowsocks" => build_shadowrocket_shadowsocks(&uuid, &server, &name, &settings, &state.app_key),
                "trojan" => build_shadowrocket_trojan(&uuid, &server, &name, &settings),
                "hysteria" => build_general_hysteria(&uuid, &server, &name, &settings),
                "socks" => build_general_socks(&uuid, &server, &name),
                _ => String::new(),
            },
            RustSubscribeMode::QuantumultX => match normalized_type.as_str() {
                "vmess" => build_quantumultx_vmess(&uuid, &server, &name, &settings),
                "shadowsocks" => build_quantumultx_shadowsocks(&uuid, &server, &name, &settings, &state.app_key),
                "trojan" => build_quantumultx_trojan(&uuid, &server, &name, &settings),
                _ => String::new(),
            },
            RustSubscribeMode::Loon => String::new(),
            RustSubscribeMode::Surge => String::new(),
            RustSubscribeMode::Surfboard => String::new(),
            RustSubscribeMode::Stash => String::new(),
            RustSubscribeMode::Clash => String::new(),
            RustSubscribeMode::ClashMeta => String::new(),
            RustSubscribeMode::SingBox => String::new(),
        };

        if !line.is_empty() {
            buffer.push_str(&line);
        }
    }

    let final_payload = match mode {
        RustSubscribeMode::General => BASE64_STANDARD.encode(buffer),
        RustSubscribeMode::Shadowsocks => build_shadowsocks_sip008_payload(state, user, &servers, &uuid),
        RustSubscribeMode::Shadowrocket => BASE64_STANDARD.encode(buffer),
        RustSubscribeMode::QuantumultX => BASE64_STANDARD.encode(buffer),
        RustSubscribeMode::Loon => BASE64_STANDARD.encode(build_loon_payload(state, user, &servers, &uuid)),
        RustSubscribeMode::Surge => build_surge_config_payload(
            state,
            user,
            &servers,
            &uuid,
        ).await?,
        RustSubscribeMode::Surfboard => build_surfboard_config_payload(
            state,
            user,
            &servers,
            &uuid,
        ).await?,
        RustSubscribeMode::Stash => build_stash_yaml_payload(
            state,
            user,
            &servers,
            &uuid,
        ).await?,
        RustSubscribeMode::Clash => build_clash_yaml_payload(
            state,
            user,
            &servers,
            &uuid,
            query,
        ).await?,
        RustSubscribeMode::ClashMeta => build_clash_yaml_payload(
            state,
            user,
            &servers,
            &uuid,
            query,
        ).await?,
        RustSubscribeMode::SingBox => build_singbox_json_payload(
            state,
            user,
            &servers,
            &uuid,
        ).await?,
    };

    Ok(bytes::Bytes::from(final_payload))
}

fn parse_requested_types(value: Option<&str>) -> HashSet<String> {
    let value = value.unwrap_or("").trim();
    if value.is_empty() || value.eq_ignore_ascii_case("all") {
        return HashSet::new();
    }

    value
        .split(|c| c == ',' || c == '|' || c == '｜')
        .map(|item| normalize_type(item).unwrap_or_default())
        .filter(|item| !item.is_empty())
        .collect()
}

fn parse_filter_keywords(value: Option<&str>) -> Vec<String> {
    let value = value.unwrap_or("").trim();
    if value.is_empty() || value.chars().count() > 20 {
        return Vec::new();
    }

    value
        .split(|c| c == ',' || c == '|' || c == '｜')
        .map(|item| item.trim().to_lowercase())
        .filter(|item| !item.is_empty())
        .collect()
}

async fn build_clash_yaml_payload(
    state: &AppState,
    user: &UserRow,
    servers: &[ServerNodeRow],
    uuid: &str,
    query: &HashMap<String, String>,
) -> Result<String, Response<Body>> {
    let app_name = get_setting_string(state, "app_name", "Portal").await;
    let mut config = serde_yaml::from_str::<serde_yaml::Value>(CLASH_DEFAULT_TEMPLATE)
    .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "invalid clash template"))?;

    let proxies = build_clash_proxies(servers, uuid, state).await;
    let proxy_names = proxies
        .iter()
        .filter_map(|p| p.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .collect::<Vec<_>>();

    if let Some(map) = config.as_mapping_mut() {
        map.insert(
            serde_yaml::Value::String("proxies".into()),
            serde_yaml::to_value(proxies).unwrap_or_default(),
        );

        if let Some(groups) = map.get_mut(serde_yaml::Value::String("proxy-groups".into()))
            .and_then(|v| v.as_sequence_mut())
        {
            for group in groups.iter_mut() {
                if let Some(gmap) = group.as_mapping_mut() {
                    if let Some(proxies_field) = gmap.get_mut(serde_yaml::Value::String("proxies".into())) {
                        if let Some(seq) = proxies_field.as_sequence_mut() {
                            seq.clear();
                            seq.extend(proxy_names.iter().cloned().map(serde_yaml::Value::String));
                        }
                    }
                }
            }
        }

        if let Some(rules) = map.get_mut(serde_yaml::Value::String("rules".into()))
            .and_then(|v| v.as_sequence_mut())
        {
            if let Some(host) = query.get("host").cloned() {
                rules.insert(0, serde_yaml::Value::String(format!("DOMAIN,{},DIRECT", host)));
            }
        }
    }

    let mut out = serde_yaml::to_string(&config)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "yaml serialize failed"))?;
    out = out.replace("$app_name", &app_name);
    if let Some(header) = build_userinfo_header(user) {
        out = format!("{}\n{}", header, out);
    }
    Ok(out)
}

const CLASH_DEFAULT_TEMPLATE: &str = r#"mixed-port: 7890
allow-lan: true
bind-address: "*"
mode: rule
log-level: info
external-controller: 127.0.0.1:9090

dns:
  enable: true
  ipv6: false
  default-nameserver:
    - 223.5.5.5
    - 119.29.29.29
  enhanced-mode: fake-ip
  fake-ip-range: 198.18.0.1/16
  use-hosts: true
  nameserver:
    - https://doh.pub/dns-query
    - https://dns.alidns.com/dns-query
  fallback:
    - https://doh-pure.onedns.net/dns-query
    - https://ada.openbld.net/dns-query
    - https://223.5.5.5/dns-query
    - https://223.6.6.6/dns-query
  fallback-filter:
    geoip: true
    ipcidr:
      - 240.0.0.0/4
      - 0.0.0.0/32

proxies: []

proxy-groups:
  - { name: "$app_name", type: select, proxies: ["自动选择", "故障转移"] }
  - { name: "自动选择", type: url-test, proxies: [], url: "http://www.gstatic.com/generate_204", interval: 86400 }
  - { name: "故障转移", type: fallback, proxies: [], url: "http://www.gstatic.com/generate_204", interval: 7200 }

rules:
  - DOMAIN-SUFFIX,services.googleapis.cn,$app_name
  - DOMAIN-SUFFIX,xn--ngstr-lra8j.com,$app_name
  - DOMAIN,safebrowsing.urlsec.qq.com,DIRECT
  - DOMAIN,safebrowsing.googleapis.com,DIRECT
  - DOMAIN,developer.apple.com,$app_name
  - DOMAIN-SUFFIX,digicert.com,$app_name
  - DOMAIN,ocsp.apple.com,$app_name
  - DOMAIN-SUFFIX,apple-dns.net,$app_name
  - DOMAIN,itunes.apple.com,$app_name
  - DOMAIN-SUFFIX,apps.apple.com,$app_name
  - DOMAIN-SUFFIX,icloud.com,DIRECT
  - DOMAIN-SUFFIX,me.com,DIRECT
  - DOMAIN-SUFFIX,apple.com,DIRECT
  - DOMAIN-KEYWORD,google,$app_name
  - DOMAIN-KEYWORD,youtube,$app_name
  - DOMAIN-KEYWORD,facebook,$app_name
  - MATCH,$app_name
"#;

const SINGBOX_DEFAULT_TEMPLATE: &str = r#"{
  "dns": {
    "rules": [
      { "outbound": ["any"], "server": "local" },
      { "clash_mode": "global", "server": "remote" },
      { "clash_mode": "direct", "server": "local" },
      { "rule_set": ["geosite-cn"], "server": "local" }
    ],
    "servers": [
      { "address": "https://1.1.1.1/dns-query", "detour": "节点选择", "tag": "remote" },
      { "address": "https://223.5.5.5/dns-query", "detour": "direct", "tag": "local" },
      { "address": "rcode://success", "tag": "block" }
    ],
    "strategy": "prefer_ipv4"
  },
  "experimental": {
    "cache_file": {
      "enabled": true,
      "path": "cache.db",
      "cache_id": "cache_db",
      "store_fakeip": true
    }
  },
  "inbounds": [
    {
      "auto_route": true,
      "domain_strategy": "prefer_ipv4",
      "endpoint_independent_nat": true,
      "address": ["172.19.0.1/30", "2001:0470:f9da:fdfa::1/64"],
      "mtu": 9000,
      "sniff": true,
      "sniff_override_destination": true,
      "stack": "system",
      "strict_route": true,
      "type": "tun"
    },
    {
      "domain_strategy": "prefer_ipv4",
      "listen": "127.0.0.1",
      "listen_port": 2333,
      "sniff": true,
      "sniff_override_destination": true,
      "tag": "socks-in",
      "type": "socks",
      "users": []
    },
    {
      "domain_strategy": "prefer_ipv4",
      "listen": "127.0.0.1",
      "listen_port": 2334,
      "sniff": true,
      "sniff_override_destination": true,
      "tag": "mixed-in",
      "type": "mixed",
      "users": []
    }
  ],
  "outbounds": [
    { "tag": "节点选择", "type": "selector", "default": "自动选择", "outbounds": ["自动选择"] },
    { "tag": "direct", "type": "direct" },
    { "tag": "block", "type": "block" },
    { "tag": "dns-out", "type": "dns" },
    { "tag": "自动选择", "type": "urltest", "outbounds": [] }
  ],
  "route": {
    "auto_detect_interface": true,
    "rules": [
      { "outbound": "dns-out", "protocol": "dns" },
      { "clash_mode": "direct", "outbound": "direct" },
      { "clash_mode": "global", "outbound": "节点选择" },
      { "ip_is_private": true, "outbound": "direct" },
      { "rule_set": ["geosite-cn", "geoip-cn"], "outbound": "direct" }
    ],
    "rule_set": [
      { "tag": "geosite-cn", "type": "remote", "format": "binary", "url": "https://raw.githubusercontent.com/SagerNet/sing-geosite/rule-set/geosite-cn.srs", "download_detour": "自动选择" },
      { "tag": "geoip-cn", "type": "remote", "format": "binary", "url": "https://raw.githubusercontent.com/SagerNet/sing-geoip/rule-set/geoip-cn.srs", "download_detour": "自动选择" }
    ]
  }
}"#;

async fn load_login_user_by_email(state: &AppState, email: &str) -> Result<Option<LoginUserRow>, sqlx::Error> {
    sqlx::query_as::<_, LoginUserRow>(
        "SELECT id, email, password, password_algo, password_salt, banned, ban_reason, token, is_admin, is_super_admin, last_login_at
         FROM v2_user
         WHERE email = ?
         LIMIT 1"
    )
    .bind(email)
    .fetch_optional(&state.db)
    .await
}

async fn load_login_user_by_id(state: &AppState, user_id: i64) -> Result<Option<LoginUserRow>, sqlx::Error> {
    sqlx::query_as::<_, LoginUserRow>(
        "SELECT id, email, password, password_algo, password_salt, banned, ban_reason, token, is_admin, is_super_admin, last_login_at
         FROM v2_user
         WHERE id = ?
         LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
}

async fn load_bearer_user_by_id(state: &AppState, user_id: i64) -> Result<Option<BearerUserRow>, sqlx::Error> {
    sqlx::query_as::<_, BearerUserRow>(
        "SELECT id, invite_user_id, email, transfer_enable, last_login_at, created_at, banned, ban_reason,
                remind_expire, remind_traffic, expired_at, balance, commission_balance, plan_id, group_id,
                discount, commission_rate, telegram_id, uuid, is_admin, is_super_admin, trust_level,
                is_silenced, linux_do_id, linux_do_username, linux_do_name, linux_do_avatar, api_key,
                concurrent_ip_limit, token, subscribe_path, subscribe_key, subscribe_salt, u, d,
                device_limit, speed_limit, next_reset_at
         FROM v2_user WHERE id = ? LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
}

async fn load_bearer_user_by_token(state: &AppState, token: &str) -> Result<Option<BearerUserRow>, sqlx::Error> {
    sqlx::query_as::<_, BearerUserRow>(
        "SELECT id, invite_user_id, email, transfer_enable, last_login_at, created_at, banned, ban_reason,
                remind_expire, remind_traffic, expired_at, balance, commission_balance, plan_id, group_id,
                discount, commission_rate, telegram_id, uuid, is_admin, is_super_admin, trust_level,
                is_silenced, linux_do_id, linux_do_username, linux_do_name, linux_do_avatar, api_key,
                concurrent_ip_limit, token, subscribe_path, subscribe_key, subscribe_salt, u, d,
                device_limit, speed_limit, next_reset_at
         FROM v2_user WHERE token = ? LIMIT 1"
    )
    .bind(token)
    .fetch_optional(&state.db)
    .await
}

async fn load_coupon_by_code(
    state: &AppState,
    code: &str,
) -> Result<Option<CouponRow>, sqlx::Error> {
    sqlx::query_as::<_, CouponRow>(
        "SELECT
            id, code, name, owner_user_id, source_plan_id, type AS type_field, value, `show`,
            limit_use, limit_use_with_user, limit_plan_ids, limit_period,
            started_at, ended_at, created_at, updated_at
         FROM v2_coupon
         WHERE code = ?
         LIMIT 1"
    )
    .bind(code)
    .fetch_optional(&state.db)
    .await
}

async fn load_coupon_by_code_with_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    code: &str,
) -> Result<Option<CouponRow>, sqlx::Error> {
    sqlx::query_as::<_, CouponRow>(
        "SELECT
            id, code, name, owner_user_id, source_plan_id, type AS type_field, value, `show`,
            limit_use, limit_use_with_user, limit_plan_ids, limit_period,
            started_at, ended_at, created_at, updated_at
         FROM v2_coupon
         WHERE code = ?
         LIMIT 1
         FOR UPDATE"
    )
    .bind(code)
    .fetch_optional(&mut **tx)
    .await
}

async fn validate_coupon_for_user(
    state: &AppState,
    coupon: &CouponRow,
    user_id: i64,
    plan_id: Option<i64>,
    period: Option<&str>,
) -> Result<(), Response<Body>> {
    if !coupon.show {
        return Err(fail_json_response(StatusCode::BAD_REQUEST, "Invalid coupon"));
    }
    if let Some(limit_use) = coupon.limit_use {
        if limit_use <= 0 {
            return Err(fail_json_response(StatusCode::BAD_REQUEST, "This coupon is no longer available"));
        }
    }

    let now = Utc::now().timestamp();
    if now < coupon.started_at {
        return Err(fail_json_response(StatusCode::BAD_REQUEST, "This coupon has not yet started"));
    }
    if now > coupon.ended_at {
        return Err(fail_json_response(StatusCode::BAD_REQUEST, "This coupon has expired"));
    }

    if let Some(plan_id) = plan_id {
        let limit_plan_ids = parse_coupon_limit_plan_ids(coupon.limit_plan_ids.as_deref());
        if !limit_plan_ids.is_empty() && !limit_plan_ids.contains(&plan_id) {
            return Err(fail_json_response(StatusCode::BAD_REQUEST, "The coupon code cannot be used for this subscription"));
        }
    }

    if let Some(period) = period {
        let limit_periods = parse_coupon_limit_periods(coupon.limit_period.as_deref());
        if !limit_periods.is_empty() && !limit_periods.iter().any(|item| item == period) {
            return Err(fail_json_response(StatusCode::BAD_REQUEST, "The coupon code cannot be used for this period"));
        }
    }

    if let Some(limit_per_user) = coupon.limit_use_with_user {
        let used_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM v2_order
             WHERE coupon_id = ?
               AND user_id = ?
               AND status NOT IN (0, 2)"
        )
        .bind(coupon.id)
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
        if used_count >= limit_per_user {
            let message = format!("The coupon can only be used {} per person", limit_per_user);
            return Err(fail_json_response(StatusCode::BAD_REQUEST, &message));
        }
    }

    Ok(())
}

fn compute_coupon_discount_amount(total_amount: i64, coupon: &CouponRow) -> i64 {
    match coupon.type_field {
        1 => coupon.value.max(0),
        2 => ((total_amount as f64) * ((coupon.value as f64) / 100.0)).round() as i64,
        _ => 0,
    }
}

async fn load_user_orders(state: &AppState, user_id: i64, status_filter: Option<i64>) -> Result<Vec<UserOrderRow>, sqlx::Error> {
    let mut sql = String::from(
        "SELECT o.id, o.user_id, o.plan_id, o.payment_id, o.period, o.trade_no, o.total_amount, o.handling_amount,
                o.balance_amount, o.refund_amount, o.surplus_amount, o.discount_amount, o.type AS type_field,
                o.status, o.surplus_order_ids, o.coupon_id, o.created_at, o.updated_at, o.commission_status,
                o.invite_user_id, o.actual_commission_balance, o.commission_balance, o.paid_at, o.callback_no,
                p.name AS plan_name, p.scope AS plan_scope
         FROM v2_order o
         LEFT JOIN v2_plan p ON p.id = o.plan_id
         WHERE o.user_id = ?"
    );
    if status_filter.is_some() {
        sql.push_str(" AND o.status = ?");
    }
    sql.push_str(" ORDER BY o.created_at DESC");

    let mut query = sqlx::query_as::<_, UserOrderRow>(&sql).bind(user_id);
    if let Some(status) = status_filter {
        query = query.bind(status);
    }
    query.fetch_all(&state.db).await
}

async fn find_user_order_by_trade_no(state: &AppState, user_id: i64, trade_no: &str) -> Result<Option<UserOrderRow>, sqlx::Error> {
    if trade_no.is_empty() {
        return Ok(None);
    }
    sqlx::query_as::<_, UserOrderRow>(
        "SELECT o.id, o.user_id, o.plan_id, o.payment_id, o.period, o.trade_no, o.total_amount, o.handling_amount,
                o.balance_amount, o.refund_amount, o.surplus_amount, o.discount_amount, o.type AS type_field,
                o.status, o.surplus_order_ids, o.coupon_id, o.created_at, o.updated_at, o.commission_status,
                o.invite_user_id, o.actual_commission_balance, o.commission_balance, o.paid_at, o.callback_no,
                p.name AS plan_name, p.scope AS plan_scope
         FROM v2_order o
         LEFT JOIN v2_plan p ON p.id = o.plan_id
         WHERE o.user_id = ? AND o.trade_no = ?
         LIMIT 1"
    )
    .bind(user_id)
    .bind(trade_no)
    .fetch_optional(&state.db)
    .await
}

async fn load_orders_by_ids(state: &AppState, ids: &[i64]) -> Result<Vec<UserOrderRow>, sqlx::Error> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = vec!["?"; ids.len()].join(",");
    let sql = format!(
        "SELECT o.id, o.user_id, o.plan_id, o.payment_id, o.period, o.trade_no, o.total_amount, o.handling_amount,
                o.balance_amount, o.refund_amount, o.surplus_amount, o.discount_amount, o.type AS type_field,
                o.status, o.surplus_order_ids, o.coupon_id, o.created_at, o.updated_at, o.commission_status,
                o.invite_user_id, o.actual_commission_balance, o.commission_balance, o.paid_at, o.callback_no,
                p.name AS plan_name, p.scope AS plan_scope
         FROM v2_order o
         LEFT JOIN v2_plan p ON p.id = o.plan_id
         WHERE o.id IN ({})",
        placeholders
    );
    let mut query = sqlx::query_as::<_, UserOrderRow>(&sql);
    for id in ids {
        query = query.bind(*id);
    }
    query.fetch_all(&state.db).await
}

async fn load_owned_node_admin_node(
    state: &AppState,
    node_id: u64,
    owner_user_id: i64,
) -> Result<Option<NodeAdminNodeRow>, sqlx::Error> {
    sqlx::query_as::<_, NodeAdminNodeRow>(
        "SELECT id, user_id, name
         FROM server_nodes
         WHERE id = ? AND user_id = ?
         LIMIT 1"
    )
    .bind(node_id)
    .bind(owner_user_id)
    .fetch_optional(&state.db)
    .await
}

async fn apply_server_node_access_control_with_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    node_id: u64,
    access: &Map<String, Value>,
) -> Result<(), sqlx::Error> {
    if let Some(authorized_users) = access.get("authorized_users").and_then(Value::as_array) {
        let user_ids = distinct_positive_user_ids(
            authorized_users
                .iter()
                .filter_map(parse_i64_value),
        );
        replace_individual_node_access_with_tx(tx, node_id, &user_ids).await?;
    }
    Ok(())
}

fn payload_string_field(payload: &Map<String, Value>, key: &str) -> Option<String> {
    payload.get(key).and_then(Value::as_str).map(str::trim).filter(|value| !value.is_empty()).map(ToString::to_string)
}

fn payload_optional_i64_field(payload: &Map<String, Value>, key: &str) -> Option<Option<i64>> {
    payload.get(key).map(|value| {
        if value.is_null() {
            None
        } else {
            parse_i64_value(value)
        }
    })
}

fn payload_object_field(payload: &Map<String, Value>, key: &str) -> Option<Map<String, Value>> {
    payload.get(key).and_then(Value::as_object).cloned()
}

fn server_node_validate_protocol(protocol: &str) -> bool {
    server_node_supported_protocols().contains(&protocol)
}

fn parse_server_node_mutation_input(
    payload: &Value,
    existing: Option<&ServerNodeOwnerRow>,
    allow_concurrent_ip_limit: bool,
) -> Result<ServerNodeMutationInput, Response<Body>> {
    let payload = payload.as_object().ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    let mut input = ServerNodeMutationInput::default();

    input.name = payload_string_field(payload, "name");
    input.host = payload_string_field(payload, "host");
    input.protocol = payload_string_field(payload, "protocol").map(|value| value.to_ascii_lowercase());
    input.location_code = payload_string_field(payload, "location_code");
    input.location_name = payload_string_field(payload, "location_name");
    input.settings = payload_object_field(payload, "settings");
    input.access_control = payload_object_field(payload, "access_control");
    input.traffic_limit = payload.get("traffic_limit").and_then(parse_i64_value);
    input.traffic_multiplier = payload.get("traffic_multiplier").and_then(Value::as_f64);
    input.device_limit = payload.get("device_limit").and_then(parse_i64_value);
    input.connection_limit = payload.get("connection_limit").and_then(parse_i64_value);
    input.speed_limit_up = payload.get("speed_limit_up").and_then(parse_i64_value);
    input.speed_limit_down = payload.get("speed_limit_down").and_then(parse_i64_value);
    input.cross_node_ip_limit = payload.get("cross_node_ip_limit").and_then(parse_i64_value);
    input.concurrent_ip_limit = if allow_concurrent_ip_limit {
        payload.get("concurrent_ip_limit").and_then(parse_i64_value)
    } else {
        None
    };
    input.tcping_host = payload_string_field(payload, "tcping_host");
    input.tcping_interval_seconds = payload.get("tcping_interval_seconds").and_then(parse_i64_value);
    input.tcping_timeout_ms = payload.get("tcping_timeout_ms").and_then(parse_i64_value);
    input.tcping_alert_after_seconds = payload.get("tcping_alert_after_seconds").and_then(parse_i64_value);
    input.tcping_recover_after_seconds = payload.get("tcping_recover_after_seconds").and_then(parse_i64_value);
    input.port = payload.get("port").and_then(parse_i64_value);

    if let Some(service_port) = payload_optional_i64_field(payload, "service_port") {
        input.service_port = service_port;
    }
    if let Some(tcping_port) = payload_optional_i64_field(payload, "tcping_port") {
        input.tcping_port = tcping_port;
    }

    if existing.is_none() {
        if input.name.is_none() || input.host.is_none() || input.port.is_none() || input.protocol.is_none() || input.location_code.is_none() || input.location_name.is_none() {
            return Err(json_status_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({"message": "Validation failed"}),
            ));
        }
    }

    if let Some(name) = &input.name {
        if name.chars().count() > 255 {
            return Err(json_status_response(StatusCode::UNPROCESSABLE_ENTITY, json!({"message":"Validation failed","errors":{"name":["The name field must not be greater than 255 characters."]}})));
        }
    }
    if let Some(host) = &input.host {
        if host.chars().count() > 255 {
            return Err(json_status_response(StatusCode::UNPROCESSABLE_ENTITY, json!({"message":"Validation failed","errors":{"host":["The host field must not be greater than 255 characters."]}})));
        }
    }
    if let Some(location_code) = &input.location_code {
        if location_code.chars().count() > 16 {
            return Err(json_status_response(StatusCode::UNPROCESSABLE_ENTITY, json!({"message":"Validation failed","errors":{"location_code":["The location_code field must not be greater than 16 characters."]}})));
        }
    }
    if let Some(location_name) = &input.location_name {
        if location_name.chars().count() > 128 {
            return Err(json_status_response(StatusCode::UNPROCESSABLE_ENTITY, json!({"message":"Validation failed","errors":{"location_name":["The location_name field must not be greater than 128 characters."]}})));
        }
    }
    if let Some(protocol) = &input.protocol {
        if !server_node_validate_protocol(protocol) {
            return Err(json_status_response(StatusCode::UNPROCESSABLE_ENTITY, json!({"message":"Validation failed","errors":{"protocol":["The selected protocol is invalid."]}})));
        }
    }
    if let Some(port) = input.port {
        if !(1..=65535).contains(&port) {
            return Err(json_status_response(StatusCode::UNPROCESSABLE_ENTITY, json!({"message":"Validation failed","errors":{"port":["The port must be between 1 and 65535."]}})));
        }
    }
    if let Some(service_port) = input.service_port {
        if !(1..=65535).contains(&service_port) {
            return Err(json_status_response(StatusCode::UNPROCESSABLE_ENTITY, json!({"message":"Validation failed","errors":{"service_port":["The service_port must be between 1 and 65535."]}})));
        }
    }
    if let Some(tcping_port) = input.tcping_port {
        if !(1..=65535).contains(&tcping_port) {
            return Err(json_status_response(StatusCode::UNPROCESSABLE_ENTITY, json!({"message":"Validation failed","errors":{"tcping_port":["The tcping_port must be between 1 and 65535."]}})));
        }
    }
    if let Some(traffic_limit) = input.traffic_limit {
        if traffic_limit < 0 {
            return Err(json_status_response(StatusCode::UNPROCESSABLE_ENTITY, json!({"message":"Validation failed","errors":{"traffic_limit":["The traffic_limit must be at least 0."]}})));
        }
    }
    if let Some(multiplier) = input.traffic_multiplier {
        if !(0.1..=100.0).contains(&multiplier) {
            return Err(json_status_response(StatusCode::UNPROCESSABLE_ENTITY, json!({"message":"Validation failed","errors":{"traffic_multiplier":["The traffic_multiplier must be between 0.1 and 100."]}})));
        }
    }

    let resolved_protocol = input
        .protocol
        .clone()
        .or_else(|| existing.map(|node| node.protocol.clone()))
        .unwrap_or_else(|| "vmess".to_string());
    let resolved_host = input
        .host
        .clone()
        .or_else(|| existing.map(|node| node.host.clone()))
        .unwrap_or_default();
    let resolved_port = input.port.or(existing.map(|node| node.port)).unwrap_or(443);
    let is_udp_protocol = tcping_is_udp_protocol(&resolved_protocol);

    if input.traffic_multiplier.is_none() && existing.is_none() {
        input.traffic_multiplier = Some(1.0);
    } else if let Some(multiplier) = input.traffic_multiplier {
        input.traffic_multiplier = Some(multiplier.max(0.1));
    }

    input.tcping_host = Some(
        input
            .tcping_host
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| {
                if let Some(existing) = existing {
                    if input.host.is_none() && existing.tcping_host.as_deref().unwrap_or("") != existing.host.as_str() {
                        return existing.tcping_host.clone().unwrap_or_else(|| resolved_host.clone());
                    }
                }
                resolved_host.clone()
            }),
    );
    input.tcping_interval_seconds = Some(input.tcping_interval_seconds.unwrap_or(existing.map(|node| node.tcping_interval_seconds).unwrap_or(60)).max(15));
    input.tcping_timeout_ms = Some(input.tcping_timeout_ms.unwrap_or(existing.map(|node| node.tcping_timeout_ms).unwrap_or(3000)).max(500));
    input.tcping_alert_after_seconds = Some(input.tcping_alert_after_seconds.unwrap_or(existing.map(|node| node.tcping_alert_after_seconds).unwrap_or(300)).max(60));
    input.tcping_recover_after_seconds = Some(input.tcping_recover_after_seconds.unwrap_or(existing.map(|node| node.tcping_recover_after_seconds).unwrap_or(120)).max(30));

    input.tcping_port = Some(match input.tcping_port {
        Some(value) => value,
        None => {
            if is_udp_protocol {
                existing.and_then(|node| node.tcping_port).unwrap_or_default()
            } else {
                resolved_port
            }
        }
    }).and_then(|value| if is_udp_protocol && value == 0 { None } else { Some(value) });

    Ok(input)
}

fn trust_level_group_name(trust_level: i64) -> &'static str {
    match trust_level {
        0 => "new_user",
        1 => "basic_user",
        2 => "member",
        3 => "regular",
        4 => "leader",
        _ => "unknown",
    }
}

fn bearer_headers_for_user(user: &BearerUserRow) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if !user.token.trim().is_empty() {
        let value = format!("Bearer {}", user.token);
        if let Ok(header_value) = HeaderValue::from_str(&value) {
            headers.insert(AUTHORIZATION, header_value);
        }
    }
    headers
}

fn format_tcping_agent_location_display(agent: &TcpingAgentRow) -> String {
    let code = agent
        .location_code
        .as_deref()
        .unwrap_or_default()
        .trim()
        .to_uppercase();
    let name = agent.location_name.as_deref().unwrap_or_default().trim();
    let province = agent
        .location_province
        .as_deref()
        .unwrap_or_default()
        .trim();

    if code == "CN" {
        if !province.is_empty() {
            return format!("中国大陆 · {}", province);
        }
        if !name.is_empty() {
            return name.to_string();
        }
        return "中国大陆".to_string();
    }

    if !name.is_empty() {
        return name.to_string();
    }
    if !code.is_empty() {
        return code;
    }
    "未设置".to_string()
}

fn serialize_tcping_agent(agent: &TcpingAgentRow) -> Value {
    json!({
        "id": agent.id,
        "name": agent.name,
        "is_enabled": true,
        "token": agent.token,
        "location_code": agent.location_code,
        "location_name": agent.location_name,
        "location_province": agent.location_province,
        "location_display": format_tcping_agent_location_display(agent),
        "last_heartbeat_at": agent.last_heartbeat_at,
        "last_sync_at": agent.last_sync_at,
    })
}

fn serialize_tcping_agent_location(agent: &TcpingAgentRow) -> Value {
    json!({
        "id": agent.id,
        "location_code": agent.location_code,
        "location_name": agent.location_name,
        "location_province": agent.location_province,
        "location_display": format_tcping_agent_location_display(agent),
    })
}

fn serialize_tcping_sample(sample: &TcpingSampleRow) -> Value {
    json!({
        "id": sample.id,
        "agent_id": sample.agent_id,
        "is_reachable": sample.is_reachable,
        "latency_ms": sample.latency_ms,
        "is_timeout": sample.is_timeout,
        "error_message": sample.error_message,
        "sampled_at": sample.sampled_at,
    })
}

fn serialize_tcping_alert(alert: &TcpingAlertRow) -> Value {
    json!({
        "id": alert.id,
        "status": alert.status,
        "started_at": alert.started_at,
        "triggered_at": alert.triggered_at,
        "recovered_at": alert.recovered_at,
        "latest_error": alert.latest_error,
    })
}

fn server_node_protocol_labels() -> HashMap<&'static str, &'static str> {
    HashMap::from([
        ("vmess", "VMess"),
        ("vless", "VLESS"),
        ("trojan", "Trojan"),
        ("shadowsocks", "Shadowsocks"),
        ("hysteria", "Hysteria"),
        ("hysteria2", "Hysteria2"),
        ("tuic", "TUIC"),
        ("anytls", "AnyTLS"),
        ("socks", "SOCKS"),
        ("http", "HTTP"),
        ("naive", "Naive"),
        ("mieru", "Mieru"),
    ])
}

fn server_node_supported_protocols() -> Vec<&'static str> {
    vec![
        "vmess",
        "vless",
        "trojan",
        "shadowsocks",
        "hysteria",
        "hysteria2",
        "tuic",
        "anytls",
        "socks",
        "http",
        "naive",
        "mieru",
    ]
}

fn server_node_v2bx_supported_protocols() -> Vec<&'static str> {
    vec!["vmess", "vless", "trojan", "shadowsocks", "hysteria", "hysteria2", "tuic", "anytls"]
}

fn server_node_supports_v2bx_deploy(protocol: &str) -> bool {
    server_node_v2bx_supported_protocols().contains(&protocol)
}

fn server_node_prefers_auto_cert(protocol: &str) -> bool {
    matches!(protocol, "vmess" | "vless" | "trojan" | "hysteria" | "hysteria2" | "tuic" | "anytls")
}

fn server_node_protocol_items() -> Vec<Value> {
    let labels = server_node_protocol_labels();
    server_node_supported_protocols()
        .into_iter()
        .map(|protocol| {
            let normalized = if protocol == "hysteria2" { "hysteria" } else { protocol };
            let mut template = Value::Object(protocol_setting_template(normalized));
            if protocol == "hysteria2" {
                if let Some(object) = template.as_object_mut() {
                    object.insert("version".to_string(), Value::from(2));
                }
            }
            json!({
                "value": protocol,
                "label": labels.get(protocol).copied().unwrap_or(protocol.to_ascii_uppercase().leak()),
                "template": template,
            })
        })
        .collect()
}

fn server_node_monitorable(protocol: &str, tcping_port: Option<i64>) -> bool {
    if !tcping_is_udp_protocol(protocol) {
        return true;
    }
    matches!(tcping_port, Some(port) if port > 0)
}

fn server_node_last_report_at(status_cache: &HashMap<u64, Value>, node_id: u64) -> Option<i64> {
    access_node_last_report_at(status_cache, node_id)
}

fn server_node_is_online(status_cache: &HashMap<u64, Value>, node_id: u64) -> bool {
    server_node_last_report_at(status_cache, node_id)
        .map(|value| (Utc::now().timestamp() - value) <= 300)
        .unwrap_or(false)
}

fn server_node_online_status(status_cache: &HashMap<u64, Value>, node_id: u64) -> &'static str {
    if server_node_is_online(status_cache, node_id) { "online" } else { "offline" }
}

fn server_node_traffic_usage_percentage(node: &ServerNodeOwnerRow) -> f64 {
    if node.traffic_limit == 0 {
        0.0
    } else {
        ((node.traffic_used as f64 / node.traffic_limit as f64) * 100.0).min(100.0)
    }
}

fn server_node_remaining_traffic(node: &ServerNodeOwnerRow) -> Value {
    if node.traffic_limit == 0 {
        Value::from(i64::MAX)
    } else {
        Value::from((node.traffic_limit.saturating_sub(node.traffic_used)) as i64)
    }
}

fn server_node_is_traffic_exceeded(node: &ServerNodeOwnerRow) -> bool {
    node.traffic_limit > 0 && node.traffic_used >= node.traffic_limit
}

fn serialize_server_node_index_item(
    status_cache: &HashMap<u64, Value>,
    node: &ServerNodeOwnerRow,
) -> Value {
    let tcping_monitorable = server_node_monitorable(&node.protocol, node.tcping_port);
    json!({
        "id": node.id,
        "name": node.name,
        "host": node.host,
        "port": node.port,
        "service_port": node.service_port,
        "effective_service_port": node.service_port.unwrap_or(node.port),
        "protocol": node.protocol,
        "location_code": node.location_code,
        "location_name": node.location_name,
        "status": node.status,
        "online_status": server_node_online_status(status_cache, node.id),
        "is_online": server_node_is_online(status_cache, node.id),
        "last_report_at": server_node_last_report_at(status_cache, node.id),
        "traffic_limit": node.traffic_limit,
        "traffic_used": node.traffic_used,
        "traffic_usage_percentage": server_node_traffic_usage_percentage(node),
        "remaining_traffic": server_node_remaining_traffic(node),
        "traffic_multiplier": node.traffic_multiplier.parse::<f64>().unwrap_or(1.0),
        "online_users": 0,
        "device_limit": node.device_limit,
        "connection_limit": node.connection_limit,
        "speed_limit_up": node.speed_limit_up,
        "speed_limit_down": node.speed_limit_down,
        "cross_node_ip_limit": node.cross_node_ip_limit,
        "concurrent_ip_limit": node.concurrent_ip_limit,
        "tcping_enabled": tcping_monitorable,
        "tcping_host": node.tcping_host,
        "tcping_port": node.tcping_port,
        "tcping_interval_seconds": node.tcping_interval_seconds,
        "tcping_timeout_ms": node.tcping_timeout_ms,
        "tcping_alert_after_seconds": node.tcping_alert_after_seconds,
        "tcping_recover_after_seconds": node.tcping_recover_after_seconds,
        "tcping_status": if tcping_monitorable { node.tcping_last_status.clone().unwrap_or_else(|| "unknown".to_string()) } else { "unsupported".to_string() },
        "tcping_last_latency_ms": if tcping_monitorable { node.tcping_last_latency_ms } else { None },
        "tcping_last_sampled_at": if tcping_monitorable { node.tcping_last_sampled_at } else { None },
        "is_active": node.status == "active",
        "is_traffic_exceeded": server_node_is_traffic_exceeded(node),
        "created_at": format_optional_naive_datetime(node.created_at),
        "updated_at": format_optional_naive_datetime(node.updated_at),
    })
}

fn serialize_server_node_detail(
    status_cache: &HashMap<u64, Value>,
    node: &ServerNodeOwnerRow,
) -> Value {
    let tcping_monitorable = server_node_monitorable(&node.protocol, node.tcping_port);
    let mut object = Map::new();
    object.insert("id".to_string(), Value::from(node.id));
    object.insert("name".to_string(), Value::String(node.name.clone()));
    object.insert("host".to_string(), Value::String(node.host.clone()));
    object.insert("port".to_string(), Value::from(node.port));
    object.insert("service_port".to_string(), node.service_port.map(Value::from).unwrap_or(Value::Null));
    object.insert("effective_service_port".to_string(), Value::from(node.service_port.unwrap_or(node.port)));
    object.insert("protocol".to_string(), Value::String(node.protocol.clone()));
    object.insert("location_code".to_string(), node.location_code.clone().map(Value::String).unwrap_or(Value::Null));
    object.insert("location_name".to_string(), node.location_name.clone().map(Value::String).unwrap_or(Value::Null));
    object.insert("settings".to_string(), node.settings.as_ref().map(|value| value.0.clone()).unwrap_or(Value::Null));
    object.insert("status".to_string(), Value::String(node.status.clone()));
    object.insert("online_status".to_string(), Value::String(server_node_online_status(status_cache, node.id).to_string()));
    object.insert("is_online".to_string(), Value::Bool(server_node_is_online(status_cache, node.id)));
    object.insert("last_report_at".to_string(), server_node_last_report_at(status_cache, node.id).map(Value::from).unwrap_or(Value::Null));
    object.insert("traffic_limit".to_string(), Value::from(node.traffic_limit));
    object.insert("traffic_used".to_string(), Value::from(node.traffic_used));
    object.insert("traffic_usage_percentage".to_string(), Value::from(server_node_traffic_usage_percentage(node)));
    object.insert("remaining_traffic".to_string(), server_node_remaining_traffic(node));
    object.insert("traffic_multiplier".to_string(), Value::from(node.traffic_multiplier.parse::<f64>().unwrap_or(1.0)));
    object.insert("access_control".to_string(), node.access_control.as_ref().map(|value| value.0.clone()).unwrap_or(Value::Null));
    object.insert("device_limit".to_string(), Value::from(node.device_limit));
    object.insert("connection_limit".to_string(), Value::from(node.connection_limit));
    object.insert("speed_limit_up".to_string(), Value::from(node.speed_limit_up));
    object.insert("speed_limit_down".to_string(), Value::from(node.speed_limit_down));
    object.insert("cross_node_ip_limit".to_string(), Value::from(node.cross_node_ip_limit));
    object.insert("concurrent_ip_limit".to_string(), Value::from(node.concurrent_ip_limit));
    object.insert("tcping_enabled".to_string(), Value::Bool(tcping_monitorable));
    object.insert("tcping_host".to_string(), node.tcping_host.clone().map(Value::String).unwrap_or(Value::Null));
    object.insert("tcping_port".to_string(), node.tcping_port.map(Value::from).unwrap_or(Value::Null));
    object.insert("tcping_interval_seconds".to_string(), Value::from(node.tcping_interval_seconds));
    object.insert("tcping_timeout_ms".to_string(), Value::from(node.tcping_timeout_ms));
    object.insert("tcping_alert_after_seconds".to_string(), Value::from(node.tcping_alert_after_seconds));
    object.insert("tcping_recover_after_seconds".to_string(), Value::from(node.tcping_recover_after_seconds));
    object.insert(
        "tcping_status".to_string(),
        Value::String(if tcping_monitorable {
            node.tcping_last_status.clone().unwrap_or_else(|| "unknown".to_string())
        } else {
            "unsupported".to_string()
        }),
    );
    object.insert("tcping_last_latency_ms".to_string(), if tcping_monitorable { node.tcping_last_latency_ms.map(Value::from).unwrap_or(Value::Null) } else { Value::Null });
    object.insert("tcping_last_error".to_string(), if tcping_monitorable { node.tcping_last_error.clone().map(Value::String).unwrap_or(Value::Null) } else { Value::Null });
    object.insert("tcping_last_sampled_at".to_string(), if tcping_monitorable { node.tcping_last_sampled_at.map(Value::from).unwrap_or(Value::Null) } else { Value::Null });
    object.insert("v2bx_node_id".to_string(), node.v2bx_node_id.map(Value::from).unwrap_or(Value::Null));
    object.insert("v2bx_config".to_string(), node.v2bx_config.as_ref().map(|value| value.0.clone()).unwrap_or(Value::Null));
    object.insert("online_users".to_string(), Value::from(0));
    object.insert("audit_rules".to_string(), Value::Array(Vec::new()));
    object.insert("authorized_users".to_string(), Value::Array(Vec::new()));
    object.insert("is_active".to_string(), Value::Bool(node.status == "active"));
    object.insert("is_traffic_exceeded".to_string(), Value::Bool(server_node_is_traffic_exceeded(node)));
    object.insert("created_at".to_string(), format_optional_naive_datetime(node.created_at).map(Value::String).unwrap_or(Value::Null));
    object.insert("updated_at".to_string(), format_optional_naive_datetime(node.updated_at).map(Value::String).unwrap_or(Value::Null));
    Value::Object(object)
}

fn serialize_authorized_user_profile(user: &NodeTrafficUserProfileRow) -> Value {
    json!({
        "id": user.id,
        "email": user.email,
        "linux_do_name": user.linux_do_name,
        "trust_level": user.trust_level,
        "is_silenced": user.is_silenced != 0,
        "banned": user.banned != 0,
    })
}

fn render_knowledge_row(
    row: &KnowledgeRow,
    user_available: bool,
    subscribe_url: &str,
    app_name: &str,
) -> Value {
    let body = render_knowledge_body(&row.body, user_available, subscribe_url, app_name);
    json!({
        "id": row.id,
        "category": row.category,
        "title": row.title,
        "body": body,
        "updated_at": row.updated_at,
    })
}

fn render_knowledge_body(
    body: &str,
    user_available: bool,
    subscribe_url: &str,
    app_name: &str,
) -> String {
    let mut rendered = body.to_string();
    if !user_available {
        rendered = replace_knowledge_access_blocks(&rendered);
    }

    let encoded_subscribe_url = urlencoding::encode(subscribe_url).into_owned();
    let safe_base64_subscribe_url = url_safe_base64(subscribe_url.as_bytes());

    rendered = rendered.replace("{{siteName}}", app_name);
    rendered = rendered.replace("{{subscribeUrl}}", subscribe_url);
    rendered = rendered.replace("{{urlEncodeSubscribeUrl}}", &encoded_subscribe_url);
    rendered = rendered.replace("{{safeBase64SubscribeUrl}}", &safe_base64_subscribe_url);
    rendered
}

fn replace_knowledge_access_blocks(body: &str) -> String {
    let start_marker = "<!--access start-->";
    let end_marker = "<!--access end-->";
    let replacement = "<div class=\"no-access\">You must have a valid subscription to view content in this area</div>";

    let mut rendered = body.to_string();
    loop {
        let Some(start) = rendered.find(start_marker) else {
            break;
        };
        let search_start = start + start_marker.len();
        let Some(relative_end) = rendered[search_start..].find(end_marker) else {
            break;
        };
        let end = search_start + relative_end + end_marker.len();
        rendered.replace_range(start..end, replacement);
    }

    rendered
}

fn serialize_user_traffic_usage_log(log: &UserTrafficUsageLogRow) -> Value {
    json!({
        "id": log.id,
        "node_id": log.node_id,
        "node_name": log.node_name,
        "protocol": log.node_protocol,
        "location_name": log.node_location_name,
        "raw_traffic_kb": log.raw_traffic_kb,
        "billed_traffic_kb": log.billed_traffic_kb,
        "multiplier_snapshot": log.multiplier_snapshot.parse::<f64>().unwrap_or(1.0),
        "recorded_at": log.recorded_at,
        "source": log.source,
    })
}

fn serialize_server_node_status(
    status_cache: &HashMap<u64, Value>,
    node: &ServerNodeOwnerRow,
) -> Value {
    json!({
        "id": node.id,
        "name": node.name,
        "status": node.status,
        "online_status": server_node_online_status(status_cache, node.id),
        "is_online": server_node_is_online(status_cache, node.id),
        "last_report_at": server_node_last_report_at(status_cache, node.id),
        "is_active": node.status == "active",
        "online_users": 0,
        "traffic_usage_percentage": server_node_traffic_usage_percentage(node),
        "is_traffic_exceeded": server_node_is_traffic_exceeded(node),
        "tcping_status": node.tcping_last_status.clone().unwrap_or_else(|| "unknown".to_string()),
        "tcping_last_latency_ms": node.tcping_last_latency_ms,
        "tcping_last_error": node.tcping_last_error,
        "tcping_last_sampled_at": node.tcping_last_sampled_at,
        "last_check_at": format_optional_naive_datetime(node.updated_at),
    })
}

fn serialize_user_server_fetch_node_item(
    status_cache: &HashMap<u64, Value>,
    node: &ServerNodeRow,
) -> Option<Value> {
    if !server_node_is_online(status_cache, node.id) {
        return None;
    }

    let normalized_type = normalize_type(&node.protocol).unwrap_or_else(|| node.protocol.clone());
    let last_check_at = server_node_last_report_at(status_cache, node.id)
        .or_else(|| node.created_at.map(|dt| dt.timestamp()))
        .unwrap_or_else(|| Utc::now().timestamp());

    Some(json!({
        "id": node.id,
        "type": normalized_type.clone(),
        "version": Value::Null,
        "name": node.name.clone(),
        "rate": 1,
        "tags": ["server-node"],
        "is_online": true,
        "cache_key": format!("server_node:{}:{}", node.id, normalized_type),
        "last_check_at": last_check_at,
    }))
}

fn serialize_server_node_owner_model(node: &ServerNodeOwnerRow) -> Value {
    json!({
        "id": node.id,
        "user_id": node.user_id,
        "name": node.name,
        "host": node.host,
        "port": node.port,
        "service_port": node.service_port,
        "protocol": node.protocol,
        "location_code": node.location_code,
        "location_name": node.location_name,
        "settings": node.settings,
        "traffic_limit": node.traffic_limit,
        "traffic_used": node.traffic_used,
        "traffic_multiplier": node.traffic_multiplier.parse::<f64>().unwrap_or(1.0),
        "access_control": node.access_control,
        "status": node.status,
        "v2bx_node_id": node.v2bx_node_id,
        "v2bx_config": node.v2bx_config,
        "v2bx_token": node.v2bx_token,
        "device_limit": node.device_limit,
        "connection_limit": node.connection_limit,
        "speed_limit_up": node.speed_limit_up,
        "speed_limit_down": node.speed_limit_down,
        "cross_node_ip_limit": node.cross_node_ip_limit,
        "concurrent_ip_limit": node.concurrent_ip_limit,
        "tcping_enabled": node.tcping_enabled,
        "tcping_host": node.tcping_host,
        "tcping_port": node.tcping_port,
        "tcping_interval_seconds": node.tcping_interval_seconds,
        "tcping_timeout_ms": node.tcping_timeout_ms,
        "tcping_alert_after_seconds": node.tcping_alert_after_seconds,
        "tcping_recover_after_seconds": node.tcping_recover_after_seconds,
        "tcping_last_status": node.tcping_last_status,
        "tcping_last_latency_ms": node.tcping_last_latency_ms,
        "tcping_last_error": node.tcping_last_error,
        "tcping_last_sampled_at": node.tcping_last_sampled_at,
        "created_at": format_optional_naive_datetime(node.created_at),
        "updated_at": format_optional_naive_datetime(node.updated_at),
    })
}

fn tcping_is_udp_protocol(protocol: &str) -> bool {
    matches!(protocol.to_ascii_lowercase().as_str(), "hysteria" | "hysteria2" | "tuic")
}

fn tcping_is_monitorable(node: &TcpingNodeOverviewRow) -> bool {
    if !tcping_is_udp_protocol(&node.protocol) {
        return true;
    }
    matches!(node.tcping_port, Some(port) if port > 0)
}

fn serialize_tcping_node_summary(
    state: &AppState,
    node: &TcpingNodeOverviewRow,
    alerts: &[TcpingAlertRow],
) -> Value {
    let active_alert = alerts.iter().find(|alert| alert.status == "active");
    json!({
        "id": node.id,
        "name": node.name,
        "protocol": node.protocol,
        "location_name": node.location_name,
        "tcping_enabled": tcping_is_monitorable(node),
        "tcping_status": if tcping_is_monitorable(node) {
            node.tcping_last_status.clone().unwrap_or_else(|| "unknown".to_string())
        } else {
            "unsupported".to_string()
        },
        "tcping_last_latency_ms": if tcping_is_monitorable(node) { node.tcping_last_latency_ms } else { None },
        "tcping_last_error": if tcping_is_monitorable(node) { node.tcping_last_error.clone() } else { None },
        "tcping_last_sampled_at": if tcping_is_monitorable(node) { node.tcping_last_sampled_at } else { None },
        "tcping_alert_active": active_alert.is_some(),
        "tcping_active_alert": active_alert.map(|alert| json!({
            "id": alert.id,
            "started_at": alert.started_at,
            "triggered_at": alert.triggered_at,
            "latest_error": alert.latest_error,
        })).unwrap_or(Value::Null),
        "online_status": access_node_last_report_at(&state.load_status_cache.read(), node.id)
            .map(|last_report_at| if (Utc::now().timestamp() - last_report_at) <= 300 { "online" } else { "offline" })
            .unwrap_or("offline"),
    })
}

async fn resolve_panel_base_url(state: &AppState) -> String {
    let configured_url = get_setting_string(state, "app_url", "").await;
    let configured_url = configured_url.trim();
    if configured_url.is_empty() {
        return env::var("APP_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:18094".to_string())
            .trim_end_matches('/')
            .to_string();
    }
    if let Some(scheme_pos) = configured_url.find("://") {
        let scheme_end = scheme_pos + 3;
        let rest = &configured_url[scheme_end..];
        let split_pos = rest.find('/').unwrap_or(rest.len());
        let authority = &rest[..split_pos];
        let path = &rest[split_pos..];
        let mut base = format!("{}{}", &configured_url[..scheme_end], authority);
        let trimmed_path = path.trim_end_matches('/');
        if !trimmed_path.is_empty() && trimmed_path != "/" {
            base.push_str(trimmed_path);
        }
        return base.trim_end_matches('/').to_string();
    }
    configured_url.trim_end_matches('/').to_string()
}

async fn load_user_audit_logs(
    state: &AppState,
    user_id: i64,
    node_filter: Option<u64>,
    limit: i64,
) -> Result<Vec<AuditLogUserRow>, sqlx::Error> {
    let mut sql = String::from(
        "SELECT l.id, l.user_id, l.node_id, l.rule_id, l.ip_address, l.target_domain, l.target_protocol,
                l.action_taken, l.created_at, l.updated_at,
                n.name AS node_name, r.rule_type, r.rule_pattern, r.action AS rule_action
         FROM audit_logs l
         LEFT JOIN server_nodes n ON n.id = l.node_id
         LEFT JOIN audit_rules r ON r.id = l.rule_id
         WHERE l.user_id = ?"
    );
    if node_filter.is_some() {
        sql.push_str(" AND l.node_id = ?");
    }
    sql.push_str(" ORDER BY l.id DESC LIMIT ?");
    let mut query = sqlx::query_as::<_, AuditLogUserRow>(&sql).bind(user_id);
    if let Some(node_id) = node_filter {
        query = query.bind(node_id);
    }
    query.bind(limit).fetch_all(&state.db).await
}

async fn load_node_owner_audit_logs(
    state: &AppState,
    node_id: u64,
    limit: i64,
) -> Result<Vec<AuditLogUserRow>, sqlx::Error> {
    sqlx::query_as::<_, AuditLogUserRow>(
        "SELECT l.id, l.user_id, l.node_id, l.rule_id, l.ip_address, l.target_domain, l.target_protocol,
                l.action_taken, l.created_at, l.updated_at,
                n.name AS node_name, r.rule_type, r.rule_pattern, r.action AS rule_action
         FROM audit_logs l
         LEFT JOIN server_nodes n ON n.id = l.node_id
         LEFT JOIN audit_rules r ON r.id = l.rule_id
         WHERE l.node_id = ?
         ORDER BY l.id DESC
         LIMIT ?"
    )
    .bind(node_id)
    .bind(limit)
    .fetch_all(&state.db)
    .await
}

async fn load_node_owner_audit_rules(
    state: &AppState,
    node_id: u64,
) -> Result<Vec<AuditRuleOwnerRow>, sqlx::Error> {
    sqlx::query_as::<_, AuditRuleOwnerRow>(
        "SELECT id, node_id, rule_type, rule_pattern, action, is_active, created_at, updated_at
         FROM audit_rules
         WHERE node_id = ?
         ORDER BY id DESC"
    )
    .bind(node_id)
    .fetch_all(&state.db)
    .await
}

async fn load_latest_node_owner_audit_rule(
    state: &AppState,
    node_id: u64,
) -> Result<Option<AuditRuleOwnerRow>, sqlx::Error> {
    sqlx::query_as::<_, AuditRuleOwnerRow>(
        "SELECT id, node_id, rule_type, rule_pattern, action, is_active, created_at, updated_at
         FROM audit_rules
         WHERE node_id = ?
         ORDER BY id DESC
         LIMIT 1"
    )
    .bind(node_id)
    .fetch_optional(&state.db)
    .await
}

async fn load_node_owner_audit_rule_by_id(
    state: &AppState,
    node_id: u64,
    rule_id: u64,
) -> Result<Option<AuditRuleOwnerRow>, sqlx::Error> {
    sqlx::query_as::<_, AuditRuleOwnerRow>(
        "SELECT id, node_id, rule_type, rule_pattern, action, is_active, created_at, updated_at
         FROM audit_rules
         WHERE id = ? AND node_id = ?
         LIMIT 1"
    )
    .bind(rule_id)
    .bind(node_id)
    .fetch_optional(&state.db)
    .await
}

async fn load_unused_invite_codes(
    state: &AppState,
    user_id: i64,
) -> Result<Vec<InviteCodeFetchRow>, sqlx::Error> {
    sqlx::query_as::<_, InviteCodeFetchRow>(
        "SELECT ic.user_id, ic.code, ic.pv, ic.status, ic.assigned_plan_id, ic.assigned_period,
                p.name AS assigned_plan_name, ic.created_at, ic.updated_at
         FROM v2_invite_code ic
         LEFT JOIN v2_plan p ON p.id = ic.assigned_plan_id
         WHERE ic.user_id = ? AND ic.status = 0
         ORDER BY ic.id DESC"
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
}

async fn load_assignable_invite_plans(
    state: &AppState,
    owner_user_id: i64,
) -> Result<Vec<AvailableInvitePlanRow>, sqlx::Error> {
    sqlx::query_as::<_, AvailableInvitePlanRow>(
        "SELECT id, name, prices
         FROM v2_plan
         WHERE scope = 'node' AND owner_user_id = ?
         ORDER BY id DESC"
    )
    .bind(owner_user_id)
    .fetch_all(&state.db)
    .await
}

async fn validate_assignable_invite_plan(
    state: &AppState,
    owner_user_id: i64,
    plan_id: i64,
    period: &str,
) -> Result<(), Response<Body>> {
    let normalized_period = period.trim();
    let valid_period = matches!(
        normalized_period,
        "monthly" | "quarterly" | "half_yearly" | "yearly" | "two_yearly" | "three_yearly" | "onetime"
    );
    if !valid_period {
        return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "邀请套餐周期无效"));
    }

    let row = sqlx::query_as::<_, AvailableInvitePlanRow>(
        "SELECT id, name, prices
         FROM v2_plan
         WHERE id = ? AND scope = 'node' AND owner_user_id = ?
         LIMIT 1"
    )
    .bind(plan_id)
    .bind(owner_user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?;

    let Some(plan) = row else {
        return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "只能选择你自己创建的节点套餐"));
    };

    let prices = plan
        .prices
        .as_ref()
        .and_then(|value| value.0.as_object().cloned())
        .unwrap_or_default();
    if !prices.contains_key(normalized_period) {
        return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "所选套餐未启用该周期"));
    }

    Ok(())
}

async fn load_invite_commission_logs(
    state: &AppState,
    invite_user_id: i64,
    offset: i64,
    limit: i64,
) -> Result<Vec<CommissionLogRow>, sqlx::Error> {
    sqlx::query_as::<_, CommissionLogRow>(
        "SELECT id, order_amount, trade_no, get_amount, created_at
         FROM v2_commission_log
         WHERE invite_user_id = ? AND get_amount > 0
         ORDER BY created_at DESC
         LIMIT ? OFFSET ?"
    )
    .bind(invite_user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
}

async fn load_admin_traffic_reset_logs(
    state: &AppState,
    user_id: Option<i64>,
    user_email: Option<&str>,
    reset_type: Option<&str>,
    trigger_source: Option<&str>,
    start_date: Option<&str>,
    end_date: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<(Vec<TrafficResetLogAdminRow>, i64), sqlx::Error> {
    let mut where_clauses = Vec::new();
    if user_id.is_some() {
        where_clauses.push("l.user_id = ?");
    }
    if user_email.is_some() {
        where_clauses.push("u.email LIKE ?");
    }
    if reset_type.is_some() {
        where_clauses.push("l.reset_type = ?");
    }
    if trigger_source.is_some() {
        where_clauses.push("l.trigger_source = ?");
    }
    if start_date.is_some() {
        where_clauses.push("l.reset_time >= ?");
    }
    if end_date.is_some() {
        where_clauses.push("l.reset_time <= ?");
    }

    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", where_clauses.join(" AND "))
    };

    let base_from = format!(
        " FROM v2_traffic_reset_logs l
          LEFT JOIN v2_user u ON u.id = l.user_id{}",
        where_sql
    );
    let select_sql = format!(
        "SELECT l.id, l.user_id, u.email AS user_email, l.reset_type, l.reset_time,
                l.old_upload, l.old_download, l.old_total, l.new_upload, l.new_download, l.new_total,
                l.trigger_source, l.metadata, l.created_at
         {}
         ORDER BY l.reset_time DESC
         LIMIT ? OFFSET ?",
        base_from
    );
    let count_sql = format!("SELECT COUNT(*) {}", base_from);

    let mut select = sqlx::query_as::<_, TrafficResetLogAdminRow>(&select_sql);
    let mut count = sqlx::query_scalar::<_, i64>(&count_sql);

    if let Some(value) = user_id {
        select = select.bind(value);
        count = count.bind(value);
    }
    if let Some(value) = user_email {
        let pattern = format!("%{}%", value);
        select = select.bind(pattern.clone());
        count = count.bind(pattern);
    }
    if let Some(value) = reset_type {
        select = select.bind(value);
        count = count.bind(value);
    }
    if let Some(value) = trigger_source {
        select = select.bind(value);
        count = count.bind(value);
    }
    if let Some(value) = start_date {
        select = select.bind(value);
        count = count.bind(value);
    }
    if let Some(value) = end_date {
        let end = format!("{} 23:59:59", value);
        select = select.bind(end.clone());
        count = count.bind(end);
    }

    let total = count.fetch_one(&state.db).await?;
    let rows = select.bind(limit).bind(offset).fetch_all(&state.db).await?;
    Ok((rows, total))
}

async fn load_user_traffic_reset_history(
    state: &AppState,
    user_id: i64,
    limit: i64,
) -> Result<Vec<TrafficResetLogAdminRow>, sqlx::Error> {
    sqlx::query_as::<_, TrafficResetLogAdminRow>(
        "SELECT l.id, l.user_id, u.email AS user_email, l.reset_type, l.reset_time,
                l.old_upload, l.old_download, l.old_total, l.new_upload, l.new_download, l.new_total,
                l.trigger_source, l.metadata, l.created_at
         FROM v2_traffic_reset_logs l
         LEFT JOIN v2_user u ON u.id = l.user_id
         WHERE l.user_id = ?
         ORDER BY l.reset_time DESC
         LIMIT ?"
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(&state.db)
    .await
}

async fn load_bearer_user_row_by_id(
    state: &AppState,
    user_id: i64,
) -> Result<Option<BearerUserRow>, sqlx::Error> {
    sqlx::query_as::<_, BearerUserRow>(
        "SELECT id, invite_user_id, email, transfer_enable, last_login_at, created_at, banned, ban_reason,
                remind_expire, remind_traffic, expired_at, balance, commission_balance, plan_id,
                discount, commission_rate, telegram_id, uuid, is_admin, is_super_admin, trust_level,
                is_silenced, linux_do_id, linux_do_username, linux_do_name, linux_do_avatar,
                api_key, concurrent_ip_limit, token, subscribe_path, subscribe_key, subscribe_salt,
                u, d, device_limit, speed_limit, next_reset_at
         FROM v2_user
         WHERE id = ?
         LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
}

async fn user_reset_count(
    state: &AppState,
    user_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT reset_count FROM v2_user WHERE id = ? LIMIT 1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
}

async fn user_last_reset_at(
    state: &AppState,
    user_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT last_reset_at FROM v2_user WHERE id = ? LIMIT 1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
}

async fn load_gift_card_code_lookup(
    state: &AppState,
    code: &str,
) -> Result<Option<GiftCardCodeLookupRow>, sqlx::Error> {
    sqlx::query_as::<_, GiftCardCodeLookupRow>(
        "SELECT c.id, c.template_id, c.code, c.batch_id, c.status, c.user_id, c.used_at, c.expires_at,
                c.actual_rewards, c.usage_count, c.max_usage, c.metadata, c.created_at, c.updated_at,
                t.name AS template_name, t.description AS template_description, t.type AS template_type,
                t.status AS template_status, t.conditions AS template_conditions, t.rewards AS template_rewards,
                t.limits AS template_limits, t.special_config AS template_special_config, t.icon AS template_icon,
                t.background_image AS template_background_image, t.theme_color AS template_theme_color,
                t.sort AS template_sort, t.admin_id AS template_admin_id, t.created_at AS template_created_at,
                t.updated_at AS template_updated_at
         FROM v2_gift_card_code c
         JOIN v2_gift_card_template t ON t.id = c.template_id
         WHERE c.code = ?
         LIMIT 1"
    )
    .bind(code)
    .fetch_optional(&state.db)
    .await
}

async fn generate_unique_gift_card_code(
    state: &AppState,
    prefix: &str,
    mut tx: Option<&mut sqlx::Transaction<'_, sqlx::MySql>>,
) -> Result<String, sqlx::Error> {
    loop {
        let candidate = format!("{}{}", prefix, random_alnum(12).to_uppercase());
        let exists = if let Some(current_tx) = tx.as_deref_mut() {
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM v2_gift_card_code WHERE code = ?")
                .bind(&candidate)
                .fetch_one(&mut **current_tx)
                .await?
        } else {
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM v2_gift_card_code WHERE code = ?")
                .bind(&candidate)
                .fetch_one(&state.db)
                .await?
        };
        if exists == 0 {
            return Ok(candidate);
        }
    }
}

async fn load_gift_card_code_lookup_for_update(
    state: &AppState,
    code: &str,
) -> Result<Option<GiftCardCodeLookupRow>, sqlx::Error> {
    sqlx::query_as::<_, GiftCardCodeLookupRow>(
        "SELECT c.id, c.template_id, c.code, c.batch_id, c.status, c.user_id, c.used_at, c.expires_at,
                c.actual_rewards, c.usage_count, c.max_usage, c.metadata, c.created_at, c.updated_at,
                t.name AS template_name, t.description AS template_description, t.type AS template_type,
                t.status AS template_status, t.conditions AS template_conditions, t.rewards AS template_rewards,
                t.limits AS template_limits, t.special_config AS template_special_config, t.icon AS template_icon,
                t.background_image AS template_background_image, t.theme_color AS template_theme_color,
                t.sort AS template_sort, t.admin_id AS template_admin_id, t.created_at AS template_created_at,
                t.updated_at AS template_updated_at
         FROM v2_gift_card_code c
         JOIN v2_gift_card_template t ON t.id = c.template_id
         WHERE c.code = ?
         LIMIT 1
         FOR UPDATE"
    )
    .bind(code)
    .fetch_optional(&state.db)
    .await
}

async fn load_gift_card_usage_history(
    state: &AppState,
    user_id: i64,
    offset: i64,
    limit: i64,
) -> Result<Vec<GiftCardUsageListRow>, sqlx::Error> {
    sqlx::query_as::<_, GiftCardUsageListRow>(
        "SELECT u.id, u.created_at, u.rewards_given, u.invite_rewards,
                CAST(u.multiplier_applied AS CHAR) AS multiplier_applied,
                c.code, t.name AS template_name, t.type AS template_type, t.icon AS template_icon, t.theme_color AS template_theme_color
         FROM v2_gift_card_usage u
         LEFT JOIN v2_gift_card_code c ON c.id = u.code_id
         LEFT JOIN v2_gift_card_template t ON t.id = u.template_id
         WHERE u.user_id = ?
         ORDER BY u.created_at DESC
         LIMIT ? OFFSET ?"
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await
}

async fn load_gift_card_usage_detail(
    state: &AppState,
    usage_id: i64,
    user_id: i64,
) -> Result<Option<GiftCardUsageDetailRow>, sqlx::Error> {
    sqlx::query_as::<_, GiftCardUsageDetailRow>(
        "SELECT u.id, u.code_id, u.template_id, u.user_id, u.invite_user_id, u.rewards_given, u.invite_rewards,
                u.user_level_at_use, u.plan_id_at_use, CAST(u.multiplier_applied AS CHAR) AS multiplier_applied,
                u.user_agent, u.notes, u.created_at, c.code, t.name AS template_name, t.description AS template_description,
                t.type AS template_type, t.icon AS template_icon, t.theme_color AS template_theme_color,
                u.invite_user_id AS invite_user_id_ref, iu.email AS invite_user_email
         FROM v2_gift_card_usage u
         LEFT JOIN v2_gift_card_code c ON c.id = u.code_id
         LEFT JOIN v2_gift_card_template t ON t.id = u.template_id
         LEFT JOIN v2_user iu ON iu.id = u.invite_user_id
         WHERE u.id = ? AND u.user_id = ?
         LIMIT 1"
    )
    .bind(usage_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
}

async fn load_gift_card_plan_info(
    state: &AppState,
    plan_id: i64,
) -> Result<Option<GiftCardPlanInfoRow>, sqlx::Error> {
    sqlx::query_as::<_, GiftCardPlanInfoRow>(
        "SELECT
            p.id,
            p.scope,
            p.owner_user_id,
            p.min_trust_level,
            p.free_quota_gb_by_trust_level,
            p.node_ids,
            p.group_id,
            p.transfer_enable,
            p.is_unlimited_traffic,
            p.name,
            p.speed_limit,
            p.show,
            p.visibility_scope,
            p.access_user_ids,
            p.share_token,
            p.sort,
            p.renew,
            p.content,
            p.prices,
            p.reset_traffic_method,
            p.capacity_limit,
            p.sell,
            p.device_limit,
            p.tags,
            p.created_at,
            p.updated_at,
            owner.email AS owner_email,
            owner.linux_do_username AS owner_linux_do_username,
            owner.linux_do_name AS owner_linux_do_name
         FROM v2_plan p
         LEFT JOIN v2_user owner ON owner.id = p.owner_user_id
         WHERE p.id = ?
         LIMIT 1"
    )
    .bind(plan_id)
    .fetch_optional(&state.db)
    .await
}

async fn load_user_group_id_with_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    let result = sqlx::query_scalar::<_, Option<u64>>("SELECT group_id FROM v2_user WHERE id = ? LIMIT 1")
        .bind(user_id)
        .fetch_optional(&mut **tx)
        .await?;
    Ok(result.flatten().map(|value| value.min(i64::MAX as u64) as i64))
}

async fn load_plan_group_id(
    state: &AppState,
    plan_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    let result = sqlx::query_scalar::<_, Option<u64>>("SELECT group_id FROM v2_plan WHERE id = ? LIMIT 1")
        .bind(plan_id)
        .fetch_optional(&state.db)
        .await?;
    Ok(result.flatten().map(|value| value.min(i64::MAX as u64) as i64))
}

async fn load_effective_commission_rate(
    state: &AppState,
    user_id: i64,
) -> Result<i64, sqlx::Error> {
    let default_rate = get_setting_int(state, "invite_commission", 10).await;
    let row = sqlx::query("SELECT commission_rate FROM v2_user WHERE id = ? LIMIT 1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await?;
    let user_rate = row.try_get::<Option<i64>, _>("commission_rate").unwrap_or(None);
    Ok(user_rate.unwrap_or(default_rate))
}

async fn load_unchecked_commission_balance(
    state: &AppState,
    user_id: i64,
) -> Result<i64, sqlx::Error> {
    let mut sum: i64 = sqlx::query_scalar(
        "SELECT CAST(COALESCE(SUM(commission_balance), 0) AS SIGNED)
         FROM v2_order
         WHERE status = 3 AND commission_status = 0 AND invite_user_id = ?"
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await?;
    if get_setting_bool(state, "commission_distribution_enable", false).await {
        let ratio = get_setting_int(state, "commission_distribution_l1", 0).await;
        sum = ((sum as f64) * ((ratio as f64) / 100.0)).round() as i64;
    }
    Ok(sum)
}

fn generate_order_trade_no() -> String {
    let now = chrono::Local::now();
    format!(
        "{}{}{}",
        now.format("%Y%m%d%H%M%S"),
        format!("{:06}", now.timestamp_subsec_micros()),
        random_digits(5)
    )
}

async fn resolve_invite_code_for_registration(state: &AppState, invite_code: &str) -> Result<Option<InviteCodeRow>, sqlx::Error> {
    let trimmed = invite_code.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    sqlx::query_as::<_, InviteCodeRow>(
        "SELECT id, user_id, code, status
         FROM v2_invite_code
         WHERE code = ? AND status = 0
         LIMIT 1"
    )
    .bind(trimmed)
    .fetch_optional(&state.db)
    .await
}

async fn consume_invite_code(state: &AppState, invite: Option<&InviteCodeRow>) -> Result<(), sqlx::Error> {
    let Some(invite) = invite else {
        return Ok(());
    };
    if get_setting_bool(state, "invite_never_expire", false).await {
        return Ok(());
    }
    sqlx::query("UPDATE v2_invite_code SET status = 1, updated_at = ? WHERE id = ?")
        .bind(Utc::now().timestamp())
        .bind(invite.id)
        .execute(&state.db)
        .await
        .map(|_| ())
}

async fn create_registered_user(
    state: &AppState,
    email: &str,
    password: &str,
    invite_user_id: Option<i64>,
) -> Result<LoginUserRow, sqlx::Error> {
    let hashed = bcrypt::hash(password, 12).map_err(|_| sqlx::Error::Protocol("bcrypt hash failed".into()))?;
    let now = Utc::now().timestamp();
    let token = random_hex(32);
    let uuid = random_uuid_string();
    let subscribe_path = random_letters(10);
    let subscribe_key = random_letters(8);
    let mut subscribe_salt = random_letters(6);
    if subscribe_salt == subscribe_key {
        subscribe_salt = random_letters(6);
    }
    let remind_expire = get_setting_int(state, "default_remind_expire", 1).await;
    let remind_traffic = get_setting_int(state, "default_remind_traffic", 1).await;

    let result = sqlx::query(
        "INSERT INTO v2_user (
            invite_user_id, email, password, uuid, token, subscribe_path, subscribe_key, subscribe_salt,
            remind_expire, remind_traffic, expired_at, concurrent_ip_limit, created_at, updated_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 3, ?, ?)"
    )
    .bind(invite_user_id)
    .bind(email)
    .bind(hashed)
    .bind(uuid)
    .bind(token)
    .bind(subscribe_path)
    .bind(subscribe_key)
    .bind(subscribe_salt)
    .bind(remind_expire)
    .bind(remind_traffic)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await?;

    let user_id = result.last_insert_id() as i64;
    sqlx::query("UPDATE v2_user SET last_login_at = ? WHERE id = ?")
        .bind(now)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    load_login_user_by_id(state, user_id)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)
}

async fn update_login_timestamp(state: &AppState, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE v2_user SET last_login_at = ?, updated_at = ? WHERE id = ?")
        .bind(Utc::now().timestamp())
        .bind(Utc::now().timestamp())
        .bind(user_id)
        .execute(&state.db)
        .await
        .map(|_| ())
}

async fn update_user_password(state: &AppState, user_id: i64, password: &str) -> Result<(), sqlx::Error> {
    let hashed = bcrypt::hash(password, 12).map_err(|_| sqlx::Error::Protocol("bcrypt hash failed".into()))?;
    let now = Utc::now().timestamp();
    sqlx::query(
        "UPDATE v2_user
         SET password = ?, password_algo = NULL, password_salt = NULL, updated_at = ?
         WHERE id = ?"
    )
    .bind(hashed)
    .bind(now)
    .bind(user_id)
    .execute(&state.db)
    .await
    .map(|_| ())
}

async fn can_use_email_login(state: &AppState, is_super_admin: bool) -> bool {
    if !get_setting_bool(state, "force_oauth2_login", false).await {
        return true;
    }
    if !resolve_oauth_linux_do_available(state).await {
        return true;
    }
    is_super_admin
}

fn verify_login_password(user: &LoginUserRow, password: &str) -> bool {
    match user.password_algo.as_deref().unwrap_or("") {
        "md5" => user.password == format!("{:x}", md5::compute(password.as_bytes())),
        "sha256" => sha256_hex(password) == user.password,
        "md5salt" => {
            let input = format!("{}{}", password, user.password_salt.as_deref().unwrap_or(""));
            user.password == format!("{:x}", md5::compute(input.as_bytes()))
        }
        _ => verify_bcrypt_password(password, &user.password),
    }
}

fn verify_bcrypt_password(password: &str, hash: &str) -> bool {
    if !hash.starts_with("$2y$") && !hash.starts_with("$2a$") && !hash.starts_with("$2b$") {
        return false;
    }
    let normalized = if hash.starts_with("$2y$") {
        format!("$2b${}", &hash[4..])
    } else {
        hash.to_string()
    };
    bcrypt::verify(password, &normalized).unwrap_or(false)
}

fn user_suspension_message(user: &LoginUserRow) -> String {
    let base = "Your account has been suspended";
    let reason = user.ban_reason.as_deref().unwrap_or("").trim();
    if reason.is_empty() {
        base.to_string()
    } else {
        format!("{}: {}", base, reason)
    }
}

fn gift_card_type_name(type_id: i64) -> &'static str {
    match type_id {
        1 => "通用礼品卡",
        2 => "套餐礼品卡",
        3 => "盲盒礼品卡",
        _ => "未知类型",
    }
}

fn gift_card_status_name(status: i64, expired: bool) -> &'static str {
    if expired {
        return "已过期";
    }
    match status {
        0 => "未使用",
        1 => "已使用",
        2 => "已过期",
        3 => "已禁用",
        _ => "未知状态",
    }
}

fn gift_card_expired(expires_at: Option<i64>) -> bool {
    expires_at.map(|value| value < Utc::now().timestamp()).unwrap_or(false)
}

fn validate_gift_card_is_active(code: &GiftCardCodeLookupRow) -> Result<(), Response<Body>> {
    if code.template_status == 0 {
        return Err(fail_json_response(StatusCode::BAD_REQUEST, "该礼品卡类型已停用"));
    }
    let expired = gift_card_expired(code.expires_at);
    if code.status == 2 || expired {
        return Err(fail_json_response(StatusCode::BAD_REQUEST, "兑换码不可用：已过期"));
    }
    if code.status == 3 {
        return Err(fail_json_response(StatusCode::BAD_REQUEST, "兑换码不可用：已禁用"));
    }
    if code.usage_count >= code.max_usage {
        return Err(fail_json_response(StatusCode::BAD_REQUEST, "兑换码不可用：已使用"));
    }
    Ok(())
}

async fn check_gift_card_user_eligibility(
    state: &AppState,
    code: &GiftCardCodeLookupRow,
    user: &BearerUserRow,
) -> Result<(bool, Option<String>), Response<Body>> {
    if !gift_card_check_user_conditions(state, code, user).await.map_err(internal_error)? {
        return Ok((false, Some("您不满足此礼品卡的使用条件".to_string())));
    }
    if !gift_card_check_usage_limit(state, code, user.id).await.map_err(internal_error)? {
        return Ok((false, Some("您已达到此礼品卡的使用限制".to_string())));
    }
    Ok((true, None))
}

async fn gift_card_check_user_conditions(
    state: &AppState,
    code: &GiftCardCodeLookupRow,
    user: &BearerUserRow,
) -> Result<bool, sqlx::Error> {
    let rewards = code.template_rewards.0.as_object().cloned().unwrap_or_default();
    match code.template_type {
        1 => {
            if rewards.contains_key("transfer_enable") || rewards.contains_key("expire_days") || rewards.contains_key("reset_package") {
                if user.plan_id.is_none() {
                    return Ok(false);
                }
            }
        }
        2 => {
            if bearer_user_is_active(user) {
                return Ok(false);
            }
        }
        _ => {}
    }

    let conditions = code
        .template_conditions
        .as_ref()
        .map(|value| value.0.as_object().cloned().unwrap_or_default())
        .unwrap_or_default();

    if conditions.get("new_user_only").and_then(|v| v.as_bool()).unwrap_or(false) {
        let max_days = conditions.get("new_user_max_days").and_then(parse_i64_value).unwrap_or(7);
        if user.created_at < (Utc::now().timestamp() - max_days * 86_400) {
            return Ok(false);
        }
    }

    if conditions.get("paid_user_only").and_then(|v| v.as_bool()).unwrap_or(false) {
        let paid_orders: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_order WHERE user_id = ? AND status = 3")
            .bind(user.id)
            .fetch_one(&state.db)
            .await?;
        if paid_orders <= 0 {
            return Ok(false);
        }
    }

    if let Some(Value::Array(items)) = conditions.get("allowed_plans") {
        if let Some(plan_id) = user.plan_id {
            let allowed = items.iter().filter_map(parse_i64_value).any(|value| value == plan_id);
            if !allowed {
                return Ok(false);
            }
        }
    }

    if conditions.get("require_invite").and_then(|v| v.as_bool()).unwrap_or(false) && user.invite_user_id.is_none() {
        return Ok(false);
    }

    Ok(true)
}

async fn gift_card_check_usage_limit(
    state: &AppState,
    code: &GiftCardCodeLookupRow,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    let limits = code
        .template_limits
        .as_ref()
        .map(|value| value.0.as_object().cloned().unwrap_or_default())
        .unwrap_or_default();

    if let Some(max_use_per_user) = limits.get("max_use_per_user").and_then(parse_i64_value) {
        let used_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM v2_gift_card_usage WHERE template_id = ? AND user_id = ?"
        )
        .bind(code.template_id)
        .bind(user_id)
        .fetch_one(&state.db)
        .await?;
        if used_count >= max_use_per_user {
            return Ok(false);
        }
    }

    if let Some(cooldown_hours) = limits.get("cooldown_hours").and_then(parse_i64_value) {
        let last_usage = sqlx::query_scalar::<_, i64>(
            "SELECT created_at FROM v2_gift_card_usage WHERE template_id = ? AND user_id = ? ORDER BY created_at DESC LIMIT 1"
        )
        .bind(code.template_id)
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?;
        if let Some(last_usage) = last_usage {
            if Utc::now().timestamp() < last_usage + cooldown_hours * 3600 {
                return Ok(false);
            }
        }
    }

    Ok(true)
}

fn calculate_gift_card_actual_rewards(code: &GiftCardCodeLookupRow) -> Value {
    let mut actual = code.template_rewards.0.clone();
    if code.template_type == 3 {
        if let Some(random_rewards) = actual.get("random_rewards").and_then(|v| v.as_array()).cloned() {
            let total_weight: i64 = random_rewards
                .iter()
                .map(|item| item.get("weight").and_then(parse_i64_value).unwrap_or(0).max(0))
                .sum();
            if total_weight > 0 {
                let now_seed = Utc::now().timestamp_nanos_opt().unwrap_or_default().unsigned_abs() as i64;
                let mut picked = (now_seed % total_weight).max(0) + 1;
                for reward in random_rewards {
                    let weight = reward.get("weight").and_then(parse_i64_value).unwrap_or(0).max(0);
                    if weight <= 0 {
                        continue;
                    }
                    picked -= weight;
                    if picked <= 0 {
                        if let (Value::Object(mut base), Value::Object(extra)) = (actual.clone(), reward) {
                            for (key, value) in extra {
                                if key != "weight" {
                                    base.insert(key, value);
                                }
                            }
                            base.remove("random_rewards");
                            actual = Value::Object(base);
                        }
                        break;
                    }
                }
            }
        }
    }

    let bonus = gift_card_festival_bonus(code);
    if bonus > 1.0 {
        if let Some(object) = actual.as_object_mut() {
            for value in object.values_mut() {
                if let Some(number) = value.as_i64() {
                    *value = Value::from(((number as f64) * bonus).floor() as i64);
                }
            }
        }
    }
    actual
}

fn gift_card_festival_bonus(code: &GiftCardCodeLookupRow) -> f64 {
    let config = code
        .template_special_config
        .as_ref()
        .map(|value| value.0.as_object().cloned().unwrap_or_default())
        .unwrap_or_default();
    let start_time = config.get("start_time").and_then(parse_i64_value);
    let end_time = config.get("end_time").and_then(parse_i64_value);
    let bonus = config.get("festival_bonus").and_then(parse_f64_value).unwrap_or(1.0);
    let now = Utc::now().timestamp();
    if let (Some(start_time), Some(end_time)) = (start_time, end_time) {
        if now >= start_time && now <= end_time && bonus > 1.0 {
            return bonus;
        }
    }
    1.0
}

fn calculate_gift_card_multiplier(code: &GiftCardCodeLookupRow) -> f64 {
    gift_card_festival_bonus(code)
}

fn bearer_user_is_active(user: &BearerUserRow) -> bool {
    user.banned == 0
        && user.plan_id.is_some()
        && user.expired_at.map(|value| value > Utc::now().timestamp()).unwrap_or(true)
}

fn gift_card_reward_i64(rewards: &Value, key: &str) -> i64 {
    rewards
        .get(key)
        .and_then(parse_i64_value)
        .unwrap_or(0)
}

async fn build_gift_card_code_info(
    state: &AppState,
    code: &GiftCardCodeLookupRow,
) -> Result<Value, sqlx::Error> {
    let mut value = json!({
        "code": code.code,
        "template": {
            "name": code.template_name,
            "description": code.template_description,
            "type": code.template_type,
            "type_name": gift_card_type_name(code.template_type),
            "icon": code.template_icon,
            "background_image": code.template_background_image,
            "theme_color": code.template_theme_color,
        },
        "status": code.status,
        "status_name": gift_card_status_name(code.status, gift_card_expired(code.expires_at)),
        "expires_at": code.expires_at,
        "usage_count": code.usage_count,
        "max_usage": code.max_usage
    });

    if code.template_type == 2 {
        let rewards = code.template_rewards.0.as_object().cloned().unwrap_or_default();
        if let Some(plan_id) = rewards.get("plan_id").and_then(parse_i64_value) {
            if let Some(plan) = load_gift_card_plan_info(state, plan_id).await? {
                let share_base = get_setting_string(state, "app_url", "").await;
                let system_reset_method = get_setting_int(state, "reset_traffic_method", 1).await;
                if let Some(object) = value.as_object_mut() {
                    object.insert(
                        "plan_info".to_string(),
                        serialize_gift_card_plan(&plan, share_base.trim_end_matches('/'), system_reset_method),
                    );
                }
            }
        }
    }
    Ok(value)
}

async fn apply_gift_card_rewards_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    state: &AppState,
    user_id: i64,
    code: &GiftCardCodeLookupRow,
    actual_rewards: &Value,
    now: i64,
) -> Result<BearerUserRow, Response<Body>> {
    let current = sqlx::query_as::<_, BearerUserRow>(
        "SELECT id, invite_user_id, email, transfer_enable, last_login_at, created_at, banned, ban_reason,
                remind_expire, remind_traffic, expired_at, balance, commission_balance, plan_id,
                discount, commission_rate, telegram_id, uuid, is_admin, is_super_admin, trust_level,
                is_silenced, linux_do_id, linux_do_username, linux_do_name, linux_do_avatar, api_key,
                concurrent_ip_limit, token, subscribe_path, subscribe_key, subscribe_salt, u, d,
                device_limit, speed_limit, next_reset_at
         FROM v2_user WHERE id = ? LIMIT 1 FOR UPDATE"
    )
    .bind(user_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(internal_error)?;
    let Some(current) = current else {
        return Err(fail_json_response(StatusCode::BAD_REQUEST, "The user does not exist"));
    };
    let mut plan_id = current.plan_id;
    let mut group_id = load_user_group_id_with_tx(tx, user_id).await.map_err(internal_error)?;
    let mut transfer_enable = current.transfer_enable;
    let mut device_limit = current.device_limit.unwrap_or(0);
    let mut speed_limit = current.speed_limit;
    let mut expired_at = current.expired_at;
    let mut balance = current.balance;

    let balance_reward = gift_card_reward_i64(actual_rewards, "balance");
    if balance_reward > 0 {
        balance += balance_reward;
    }

    let transfer_reward = gift_card_reward_i64(actual_rewards, "transfer_enable");
    if transfer_reward > 0 {
        transfer_enable += transfer_reward;
    }

    let device_reward = gift_card_reward_i64(actual_rewards, "device_limit");
    if device_reward > 0 {
        device_limit += device_reward;
    }

    let plan_reward_id = actual_rewards.get("plan_id").and_then(parse_i64_value);
    if let Some(plan_reward_id) = plan_reward_id {
        let plan = load_gift_card_plan_info(state, plan_reward_id).await.map_err(internal_error)?;
        let Some(plan) = plan else {
            return Err(fail_json_response(StatusCode::BAD_REQUEST, "Subscription plan does not exist"));
        };
        let plan_id_i64 = plan.id;
        plan_id = Some(plan_id_i64);
        group_id = load_plan_group_id(state, plan_id_i64).await.map_err(internal_error)?;
        transfer_enable = plan.transfer_enable.unwrap_or(0).min(i64::MAX as u64) as i64;
        speed_limit = plan.speed_limit.map(|value| value.min(i64::MAX as u64) as i64);
        device_limit = plan.device_limit.map(|value| value.min(i64::MAX as u64) as i64).unwrap_or(device_limit);
        let validity_days = actual_rewards.get("plan_validity_days").and_then(parse_i64_value).unwrap_or(0);
        if validity_days > 0 {
            let current_expired = expired_at.unwrap_or(Utc::now().timestamp());
            expired_at = Some(current_expired.max(Utc::now().timestamp()) + validity_days * 86_400);
        }
    } else {
        let expire_days = gift_card_reward_i64(actual_rewards, "expire_days");
        if expire_days > 0 {
            let current_expired = expired_at.unwrap_or(Utc::now().timestamp());
            expired_at = Some(current_expired.max(Utc::now().timestamp()) + expire_days * 86_400);
        }
    }

    sqlx::query(
        "UPDATE v2_user
         SET balance = ?, transfer_enable = ?, device_limit = ?, plan_id = ?, group_id = ?,
             speed_limit = ?, expired_at = ?, updated_at = ?
         WHERE id = ?"
    )
    .bind(balance)
    .bind(transfer_enable)
    .bind(device_limit)
    .bind(plan_id)
    .bind(group_id)
    .bind(speed_limit)
    .bind(expired_at)
    .bind(now)
    .bind(user_id)
    .execute(&mut **tx)
    .await
    .map_err(internal_error)?;

    if actual_rewards.get("reset_package").and_then(|v| v.as_bool()).unwrap_or(false) && current.plan_id.is_some() {
        apply_gift_card_traffic_reset_tx(tx, &current, now).await.map_err(internal_error)?;
    }

    sqlx::query_as::<_, BearerUserRow>(
        "SELECT id, invite_user_id, email, transfer_enable, last_login_at, created_at, banned, ban_reason,
                remind_expire, remind_traffic, expired_at, balance, commission_balance, plan_id,
                discount, commission_rate, telegram_id, uuid, is_admin, is_super_admin, trust_level,
                is_silenced, linux_do_id, linux_do_username, linux_do_name, linux_do_avatar, api_key,
                concurrent_ip_limit, token, subscribe_path, subscribe_key, subscribe_salt, u, d,
                device_limit, speed_limit, next_reset_at
         FROM v2_user WHERE id = ? LIMIT 1"
    )
        .bind(user_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::BAD_REQUEST, "The user does not exist"))
}

async fn apply_gift_card_invite_rewards_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    _state: &AppState,
    user: &BearerUserRow,
    actual_rewards: &Value,
    now: i64,
) -> Result<Option<Value>, Response<Body>> {
    let Some(invite_user_id) = user.invite_user_id else {
        return Ok(None);
    };
    let rate = actual_rewards
        .get("invite_reward_rate")
        .and_then(parse_f64_value)
        .unwrap_or(0.0);
    if rate <= 0.0 {
        return Ok(None);
    }
    let invite_user = sqlx::query_as::<_, BearerUserRow>(
        "SELECT id, invite_user_id, email, transfer_enable, last_login_at, created_at, banned, ban_reason,
                remind_expire, remind_traffic, expired_at, balance, commission_balance, plan_id,
                discount, commission_rate, telegram_id, uuid, is_admin, is_super_admin, trust_level,
                is_silenced, linux_do_id, linux_do_username, linux_do_name, linux_do_avatar, api_key,
                concurrent_ip_limit, token, subscribe_path, subscribe_key, subscribe_salt, u, d,
                device_limit, speed_limit, next_reset_at
         FROM v2_user WHERE id = ? LIMIT 1 FOR UPDATE"
    )
    .bind(invite_user_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(internal_error)?;
    let Some(invite_user) = invite_user else {
        return Ok(None);
    };

    let mut payload = Map::new();
    let balance_reward = gift_card_reward_i64(actual_rewards, "balance");
    let transfer_reward = gift_card_reward_i64(actual_rewards, "transfer_enable");
    let invite_balance = ((balance_reward as f64) * rate).floor() as i64;
    let invite_transfer = ((transfer_reward as f64) * rate).floor() as i64;

    if invite_balance > 0 || invite_transfer > 0 {
        sqlx::query(
            "UPDATE v2_user
             SET balance = ?, transfer_enable = ?, updated_at = ?
             WHERE id = ?"
        )
        .bind(invite_user.balance + invite_balance)
        .bind(invite_user.transfer_enable + invite_transfer)
        .bind(now)
        .bind(invite_user_id)
        .execute(&mut **tx)
        .await
        .map_err(internal_error)?;
    }

    if invite_balance > 0 {
        payload.insert("balance".to_string(), Value::from(invite_balance));
    }
    if invite_transfer > 0 {
        payload.insert("transfer_enable".to_string(), Value::from(invite_transfer));
    }
    if payload.is_empty() {
        return Ok(None);
    }
    Ok(Some(Value::Object(payload)))
}

async fn apply_gift_card_traffic_reset_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user: &BearerUserRow,
    now: i64,
) -> Result<(), sqlx::Error> {
    let old_upload = user.u.max(0);
    let old_download = user.d.max(0);
    let old_total = old_upload + old_download;
    let next_reset_at = user.expired_at;
    sqlx::query(
        "UPDATE v2_user
         SET u = 0, d = 0, last_reset_at = ?, reset_count = COALESCE(reset_count, 0) + 1, next_reset_at = ?, updated_at = ?
         WHERE id = ?"
    )
    .bind(now)
    .bind(next_reset_at)
    .bind(now)
    .bind(user.id)
    .execute(&mut **tx)
    .await?;

    sqlx::query(
        "INSERT INTO v2_traffic_reset_logs
            (user_id, reset_type, reset_time, old_upload, old_download, old_total, new_upload, new_download, new_total, trigger_source, metadata, created_at, updated_at)
         VALUES (?, ?, NOW(), ?, ?, ?, 0, 0, 0, ?, NULL, NOW(), NOW())"
    )
    .bind(user.id)
    .bind("manual")
    .bind(old_upload)
    .bind(old_download)
    .bind(old_total)
    .bind("gift_card")
    .execute(&mut **tx)
    .await?;

    Ok(())
}

fn serialize_gift_card_history_item(row: &GiftCardUsageListRow) -> Value {
    let masked_code = row
        .code
        .as_deref()
        .map(mask_gift_card_code)
        .unwrap_or_default();
    json!({
        "id": row.id,
        "code": masked_code,
        "template_name": row.template_name,
        "template_type": row.template_type,
        "template_type_name": row.template_type.map(gift_card_type_name),
        "rewards_given": row.rewards_given.0.clone(),
        "invite_rewards": row.invite_rewards.as_ref().map(|value| value.0.clone()),
        "multiplier_applied": row.multiplier_applied.parse::<f64>().unwrap_or(1.0),
        "created_at": row.created_at
    })
}

fn serialize_gift_card_usage_detail(row: &GiftCardUsageDetailRow) -> Value {
    json!({
        "id": row.id,
        "code": row.code,
        "template": {
            "name": row.template_name,
            "description": row.template_description,
            "type": row.template_type,
            "type_name": row.template_type.map(gift_card_type_name),
            "icon": row.template_icon,
            "theme_color": row.template_theme_color,
        },
        "rewards_given": row.rewards_given.0.clone(),
        "invite_rewards": row.invite_rewards.as_ref().map(|value| value.0.clone()),
        "invite_user": match (row.invite_user_id_ref, row.invite_user_email.as_deref()) {
            (Some(id), Some(email)) => Some(mask_gift_card_invite_user(id, email)),
            _ => None,
        },
        "user_level_at_use": row.user_level_at_use,
        "plan_id_at_use": row.plan_id_at_use,
        "multiplier_applied": row.multiplier_applied.parse::<f64>().unwrap_or(1.0),
        "notes": row.notes,
        "created_at": row.created_at
    })
}

fn serialize_gift_card_plan(plan: &GiftCardPlanInfoRow, share_base: &str, system_reset_method: i64) -> Value {
    let proxy = PlanRow {
        id: plan.id,
        scope: plan.scope.clone(),
        owner_user_id: plan.owner_user_id,
        min_trust_level: plan.min_trust_level,
        free_quota_gb_by_trust_level: plan.free_quota_gb_by_trust_level.clone(),
        node_ids: plan.node_ids.clone(),
        group_id: plan.group_id,
        transfer_enable: plan.transfer_enable,
        is_unlimited_traffic: plan.is_unlimited_traffic,
        name: plan.name.clone(),
        speed_limit: plan.speed_limit,
        show: plan.show,
        visibility_scope: plan.visibility_scope.clone(),
        access_user_ids: plan.access_user_ids.clone(),
        share_token: plan.share_token.clone(),
        sort: plan.sort,
        renew: plan.renew,
        content: plan.content.clone(),
        prices: plan.prices.clone(),
        reset_traffic_method: plan.reset_traffic_method,
        capacity_limit: plan.capacity_limit,
        sell: plan.sell,
        device_limit: plan.device_limit,
        tags: plan.tags.clone(),
        created_at: plan.created_at,
        updated_at: plan.updated_at,
        owner_email: plan.owner_email.clone(),
        owner_linux_do_username: plan.owner_linux_do_username.clone(),
        owner_linux_do_name: plan.owner_linux_do_name.clone(),
    };
    serialize_guest_plan(&proxy, share_base, system_reset_method)
}

fn mask_gift_card_code(code: &str) -> String {
    let prefix: String = code.chars().take(8).collect();
    format!("{}****", prefix)
}

fn mask_email_short(email: &str) -> String {
    let prefix: String = email.chars().take(3).collect();
    format!("{}***@***", prefix)
}

fn is_valid_hex_color(value: &str) -> bool {
    value.len() == 7 && value.starts_with('#') && value.chars().skip(1).all(|ch| ch.is_ascii_hexdigit())
}

fn parse_optional_string_field(value: Option<&Value>) -> Result<Option<String>, Response<Body>> {
    match value {
        Some(Value::Null) => Ok(None),
        Some(Value::String(text)) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(trimmed.to_string()))
            }
        }
        Some(_) => Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed")),
        None => Ok(None),
    }
}

fn normalize_optional_json_object(value: Option<&Value>) -> Result<Option<Value>, Response<Body>> {
    match value {
        Some(Value::Null) => Ok(None),
        Some(Value::Object(_)) => Ok(value.cloned()),
        Some(_) => Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed")),
        None => Ok(None),
    }
}

fn merge_optional_json_object(value: Option<&Value>, fallback: Option<Value>) -> Result<Option<Value>, Response<Body>> {
    match value {
        Some(Value::Null) => Ok(None),
        Some(Value::Object(_)) => Ok(value.cloned()),
        Some(_) => Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed")),
        None => Ok(fallback),
    }
}

fn mask_gift_card_invite_user(id: i64, email: &str) -> Value {
    let prefix: String = email.chars().take(3).collect();
    json!({
        "id": id,
        "email": format!("{}***@***", prefix)
    })
}

fn serialize_audit_rule(rule: &AuditRuleOwnerRow) -> Value {
    json!({
        "id": rule.id,
        "node_id": rule.node_id,
        "rule_type": rule.rule_type,
        "rule_pattern": rule.rule_pattern,
        "action": rule.action,
        "is_active": rule.is_active != 0,
        "created_at": format_optional_naive_datetime(rule.created_at),
        "updated_at": format_optional_naive_datetime(rule.updated_at),
    })
}

fn serialize_audit_log(log: &AuditLogUserRow) -> Value {
    json!({
        "id": log.id,
        "user_id": log.user_id,
        "node_id": log.node_id,
        "rule_id": log.rule_id,
        "ip_address": log.ip_address,
        "target_domain": log.target_domain,
        "target_protocol": log.target_protocol,
        "action_taken": log.action_taken,
        "created_at": format_optional_naive_datetime(log.created_at),
        "updated_at": format_optional_naive_datetime(log.updated_at),
        "node": log.node_name.as_ref().map(|name| json!({
            "id": log.node_id,
            "name": name,
        })),
        "rule": log.rule_id.map(|rule_id| json!({
            "id": rule_id,
            "rule_type": log.rule_type,
            "rule_pattern": log.rule_pattern,
            "action": log.rule_action,
        })),
    })
}

async fn verify_captcha_disabled_or_supported(state: &AppState) -> bool {
    !get_setting_bool(state, "captcha_enable", false).await
}

async fn verify_pow_payload(state: &AppState, headers: &HeaderMap, payload: &Value) -> Result<bool, Response<Body>> {
    if !get_setting_bool(state, "pow_enable", false).await {
        return Ok(true);
    }
    let challenge_id = payload.get("pow_id").and_then(|v| v.as_str()).unwrap_or("").trim();
    let nonce = payload.get("pow_nonce").and_then(|v| v.as_str()).unwrap_or("").trim();
    let client_token = payload.get("pow_token").and_then(|v| v.as_str()).unwrap_or("").trim();
    if challenge_id.is_empty() || nonce.is_empty() || client_token.is_empty() {
        return Ok(false);
    }

    let key = format!("{}{}pow_challenge:{}", state.redis_prefix, state.cache_prefix, challenge_id);
    let raw = match redis_get_string_raw(state, &key).await.map_err(|err| {
        error!("pow cache read failed: {}", err);
        json_error(StatusCode::INTERNAL_SERVER_ERROR, "pow cache read failed")
    })? {
        Some(value) => value,
        None => return Ok(false),
    };
    let challenge = parse_php_serialized_pow_challenge(&raw).ok_or_else(|| json_error(StatusCode::INTERNAL_SERVER_ERROR, "invalid pow cache format"))?;
    let _ = redis_del_key(state, &key).await;

    if Utc::now().timestamp() > challenge.expires_at {
        return Ok(false);
    }
    let ja3 = extract_ja3(headers);
    if get_setting_bool(state, "pow_require_ja3", true).await {
        let Some(ja3_value) = ja3.as_deref() else {
            return Ok(false);
        };
        if challenge.ja3_hash.as_deref() != Some(&sha256_hex(ja3_value)) {
            return Ok(false);
        }
    }
    let expected_token = build_pow_token(&state.app_key, ja3.as_deref().unwrap_or(""), challenge.issued_at, &challenge.seed);
    if expected_token != client_token {
        return Ok(false);
    }
    let hash_input = format!("{}|{}|{}|{}", challenge.seed, challenge.base, expected_token, nonce);
    let hash = sha256_hex(&hash_input);
    let prefix = "0".repeat(challenge.difficulty.max(0) as usize);
    Ok(hash.starts_with(&prefix))
}

fn parse_php_serialized_pow_challenge(raw: &str) -> Option<PowChallengeCacheEntry> {
    let seed = php_serialized_find_string(raw, "seed")?;
    let base = php_serialized_find_string(raw, "base")?;
    let difficulty = php_serialized_find_int(raw, "difficulty")?;
    let issued_at = php_serialized_find_int(raw, "issued_at")?;
    let expires_at = php_serialized_find_int(raw, "expires_at")?;
    let token = php_serialized_find_string(raw, "token")?;
    let ip = php_serialized_find_string(raw, "ip")?;
    let ja3_hash = php_serialized_find_optional_string(raw, "ja3_hash");
    Some(PowChallengeCacheEntry {
        seed,
        base,
        difficulty,
        issued_at,
        expires_at,
        token,
        ja3_hash,
        ip,
    })
}

fn php_serialized_find_string(raw: &str, key: &str) -> Option<String> {
    let marker = format!("s:{}:\"{}\";s:", key.len(), key);
    let start = raw.find(&marker)? + marker.len();
    let rest = &raw[start..];
    let len_end = rest.find(':')?;
    let value_start = start + len_end + 2;
    let tail = &raw[value_start..];
    let value_end = tail.find("\";")?;
    Some(tail[..value_end].to_string())
}

fn php_serialized_find_optional_string(raw: &str, key: &str) -> Option<String> {
    let null_marker = format!("s:{}:\"{}\";N;", key.len(), key);
    if raw.contains(&null_marker) {
        return None;
    }
    php_serialized_find_string(raw, key)
}

fn php_serialized_find_int(raw: &str, key: &str) -> Option<i64> {
    let marker = format!("s:{}:\"{}\";i:", key.len(), key);
    let start = raw.find(&marker)? + marker.len();
    let rest = &raw[start..];
    let end = rest.find(';')?;
    rest[..end].parse::<i64>().ok()
}

async fn issue_personal_access_token(state: &AppState, user: &LoginUserRow) -> Result<String, sqlx::Error> {
    let expire_days = get_setting_int(state, "login_token_expire_days", 365).await.clamp(0, 3650);
    let expires_at = if expire_days > 0 {
        Some((Utc::now() + chrono::Duration::days(expire_days.max(1))).naive_utc())
    } else {
        None
    };
    let name = random_alnum(20);
    let plain = sanctum_plaintext_token();
    let hashed = sha256_hex(&plain);
    let abilities = "[\"*\"]";

    let result = sqlx::query(
        "INSERT INTO personal_access_tokens (tokenable_type, tokenable_id, name, token, abilities, expires_at, created_at, updated_at)
         VALUES ('App\\\\Models\\\\User', ?, ?, ?, ?, ?, NOW(), NOW())"
    )
    .bind(user.id as u64)
    .bind(&name)
    .bind(&hashed)
    .bind(abilities)
    .bind(expires_at)
    .execute(&state.db)
    .await?;

    Ok(format!("Bearer {}", plain))
}

async fn find_user_id_by_bearer_token(state: &AppState, authorization: &str) -> Result<Option<i64>, sqlx::Error> {
    let token = authorization.trim().strip_prefix("Bearer ").unwrap_or(authorization.trim()).trim();
    if token.is_empty() {
        return Ok(None);
    }
    let hashed = sha256_hex(token);
    let row = sqlx::query(
        "SELECT tokenable_id, expires_at
         FROM personal_access_tokens
         WHERE token = ? AND tokenable_type = 'App\\\\Models\\\\User'
         LIMIT 1"
    )
        .bind(hashed)
        .fetch_optional(&state.db)
        .await?;
    let Some(row) = row else {
        return Ok(None);
    };

    let expires_at = row.try_get::<Option<chrono::NaiveDateTime>, _>("expires_at")?;
    if expires_at
        .map(|value| value <= Utc::now().naive_utc())
        .unwrap_or(false)
    {
        return Ok(None);
    }

    let user_id = row.try_get::<u64, _>("tokenable_id")?;
    Ok(Some(user_id as i64))
}

fn user_is_available(user: &BearerUserRow) -> bool {
    user.banned == 0
        && user.transfer_enable > 0
        && user
            .expired_at
            .map(|expired_at| expired_at > Utc::now().timestamp())
            .unwrap_or(true)
}

async fn authenticate_bearer_user(state: &AppState, headers: &HeaderMap) -> Result<BearerUserRow, Response<Body>> {
    let authorization = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| json_error(StatusCode::FORBIDDEN, "未登录或登陆已过期"))?;

    let user_id = find_user_id_by_bearer_token(state, &authorization)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| json_error(StatusCode::FORBIDDEN, "未登录或登陆已过期"))?;

    let user = load_bearer_user_by_id(state, user_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::BAD_REQUEST, "The user does not exist"))?;

    if user.banned != 0 {
        return Err(json_error(StatusCode::FORBIDDEN, &user_suspension_message(&LoginUserRow {
            id: user.id,
            email: user.email.clone(),
            password: String::new(),
            password_algo: None,
            password_salt: None,
            banned: user.banned,
            ban_reason: user.ban_reason.clone(),
            token: user.token.clone(),
            is_admin: user.is_admin,
            is_super_admin: user.is_super_admin,
            last_login_at: user.last_login_at,
        })));
    }

    Ok(user)
}

async fn authenticate_super_admin_user(state: &AppState, headers: &HeaderMap) -> Result<BearerUserRow, Response<Body>> {
    let user = authenticate_bearer_user(state, headers).await?;
    if user.is_super_admin == 0 {
        return Err(json_error(StatusCode::FORBIDDEN, "Super administrator privileges required"));
    }
    Ok(user)
}

async fn authenticate_admin_user(state: &AppState, headers: &HeaderMap) -> Result<BearerUserRow, Response<Body>> {
    let user = authenticate_bearer_user(state, headers).await?;
    if user.is_admin == 0 && user.is_super_admin == 0 {
        return Err(json_error(StatusCode::FORBIDDEN, "Administrator privileges required"));
    }
    Ok(user)
}

async fn build_user_subscribe_url(state: &AppState, user: &BearerUserRow) -> Option<String> {
    let path = user.subscribe_path.as_deref()?;
    let key = user.subscribe_key.as_deref()?;
    let salt = user.subscribe_salt.as_deref()?;
    let subscribe_path = get_setting_string(state, "subscribe_path", "s").await;
    let app_url = first_non_empty(&[
        get_setting_string(state, "app_url", &std::env::var("APP_URL").unwrap_or_default()).await,
        String::new(),
    ]);
    let base = if app_url.is_empty() {
        return Some(format!("/{}/{}?{}={}&{}=1", subscribe_path, path, key, user.token, salt));
    } else {
        app_url.trim_end_matches('/').to_string()
    };
    Some(format!("{}/{}/{}?{}={}&{}=1", base, subscribe_path, path, key, user.token, salt))
}

fn build_user_cache_key(uri: &Uri, user_id: i64) -> String {
    format!("user:{}:{}?{}", user_id, uri.path(), uri.query().unwrap_or_default())
}

fn resolve_user_reset_day(user: &BearerUserRow) -> Option<i64> {
    let next_reset_at = user.next_reset_at?;
    let now = Utc::now().timestamp();
    if next_reset_at <= now {
        return Some(0);
    }
    Some(((next_reset_at - now) as f64 / 86400.0).ceil() as i64)
}

fn parse_optional_positive_u64(value: Option<&String>) -> Option<Option<u64>> {
    match value.map(|value| value.trim()).filter(|value| !value.is_empty()) {
        Some(value) => value.parse::<u64>().ok().filter(|value| *value > 0).map(Some),
        None => Some(None),
    }
}

fn parse_optional_positive_i64(value: Option<&String>) -> Option<Option<i64>> {
    match value.map(|value| value.trim()).filter(|value| !value.is_empty()) {
        Some(value) => value.parse::<i64>().ok().filter(|value| *value > 0).map(Some),
        None => Some(None),
    }
}

fn parse_i64_value(value: &Value) -> Option<i64> {
    value
        .as_i64()
        .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
        .or_else(|| value.as_str().and_then(|value| value.trim().parse::<i64>().ok()))
}

fn parse_u64_value(value: &Value) -> Result<u64, ()> {
    value
        .as_u64()
        .or_else(|| value.as_i64().and_then(|value| if value > 0 { Some(value as u64) } else { None }))
        .or_else(|| value.as_str().and_then(|value| value.trim().parse::<u64>().ok()))
        .filter(|value| *value > 0)
        .ok_or(())
}

fn request_optional_string_field(payload: &Value, key: &str) -> Option<Option<String>> {
    let object = payload.as_object()?;
    if !object.contains_key(key) {
        return None;
    }
    let value = object.get(key)?;
    Some(value.as_str().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
}

fn request_optional_bool_field(payload: &Value, key: &str) -> Option<bool> {
    payload.get(key).and_then(|value| value.as_bool())
}

fn request_optional_i64_field(payload: &Value, key: &str) -> Option<Option<i64>> {
    let object = payload.as_object()?;
    if !object.contains_key(key) {
        return None;
    }
    let value = object.get(key)?;
    if value.is_null() {
        return Some(None);
    }
    Some(parse_i64_value(value))
}

fn request_required_bool_field(payload: &Value, key: &str) -> Result<bool, Response<Body>> {
    payload
        .get(key)
        .and_then(|value| value.as_bool())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))
}

fn request_required_string_field(payload: &Value, key: &str) -> Result<String, Response<Body>> {
    payload
        .get(key)
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))
}

fn request_optional_i64_array_field(payload: &Value, key: &str) -> Result<Option<Vec<i64>>, Response<Body>> {
    let Some(object) = payload.as_object() else {
        return Ok(None);
    };
    if !object.contains_key(key) {
        return Ok(None);
    }
    let Some(value) = object.get(key) else {
        return Ok(Some(Vec::new()));
    };
    let Some(array) = value.as_array() else {
        return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    };
    let items = array
        .iter()
        .filter_map(parse_i64_value)
        .filter(|value| *value > 0)
        .collect::<Vec<_>>();
    Ok(Some(items))
}

fn request_optional_free_quota_field(payload: &Value, key: &str) -> Result<Option<Option<Value>>, Response<Body>> {
    let Some(object) = payload.as_object() else {
        return Ok(None);
    };
    if !object.contains_key(key) {
        return Ok(None);
    }
    let Some(value) = object.get(key) else {
        return Ok(Some(None));
    };
    if value.is_null() {
        return Ok(Some(None));
    }
    let Some(array) = value.as_object() else {
        return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    };
    let mut result = Map::new();
    let mut has_positive = false;
    for (level, raw) in array {
        let Ok(level_num) = level.parse::<i64>() else {
            continue;
        };
        if !(0..=4).contains(&level_num) {
            continue;
        }
        let quota = parse_f64_value(raw).unwrap_or(0.0).max(0.0);
        if quota > 0.0 {
            has_positive = true;
        }
        result.insert(level_num.to_string(), Value::from(quota));
    }
    if has_positive {
        Ok(Some(Some(Value::Object(result))))
    } else {
        Ok(Some(None))
    }
}

fn format_optional_naive_datetime(value: Option<chrono::DateTime<Utc>>) -> Option<String> {
    value.map(|v| v.format("%Y-%m-%dT%H:%M:%S.000000Z").to_string())
}

fn serialize_invite_code(code: &InviteCodeFetchRow) -> Value {
    json!({
        "user_id": code.user_id,
        "code": code.code,
        "pv": code.pv,
        "status": code.status != 0,
        "assigned_plan_id": code.assigned_plan_id,
        "assigned_period": code.assigned_period,
        "assigned_plan_name": code.assigned_plan_name,
        "created_at": code.created_at,
        "updated_at": code.updated_at,
    })
}

fn serialize_available_invite_plan(plan: &AvailableInvitePlanRow) -> Value {
    let periods = plan
        .prices
        .as_ref()
        .and_then(|prices| prices.0.as_object().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(key, value)| {
            if key == "reset_traffic" {
                return None;
            }
            let label = match key.as_str() {
                "monthly" => "月付",
                "quarterly" => "季付",
                "half_yearly" => "半年付",
                "yearly" => "年付",
                "two_yearly" => "两年付",
                "three_yearly" => "三年付",
                "onetime" => "一次性",
                _ => return None,
            };
            let numeric = value.as_f64().or_else(|| value.as_i64().map(|v| v as f64))?;
            if numeric < 0.0 {
                return None;
            }
            Some(json!({
                "value": key,
                "label": label,
            }))
        })
        .collect::<Vec<_>>();

    json!({
        "id": plan.id,
        "name": plan.name,
        "periods": periods,
    })
}

fn serialize_commission_log(log: &CommissionLogRow) -> Value {
    json!({
        "id": log.id,
        "order_amount": log.order_amount,
        "trade_no": log.trade_no,
        "get_amount": log.get_amount,
        "created_at": log.created_at,
    })
}

fn format_traffic_amount(bytes: i64) -> String {
    let bytes = bytes.max(0) as f64;
    let units = ["B", "KB", "MB", "GB", "TB"];
    if bytes <= 0.0 {
        return "0 B".to_string();
    }
    let mut value = bytes;
    let mut unit_idx = 0_usize;
    while value >= 1024.0 && unit_idx < units.len() - 1 {
        value /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.2} {}", value, units[unit_idx])
}

fn serialize_admin_traffic_reset_log(log: &TrafficResetLogAdminRow) -> Value {
    json!({
        "id": log.id,
        "user_id": log.user_id,
        "user_email": log.user_email.clone().unwrap_or_else(|| "N/A".to_string()),
        "reset_type": log.reset_type,
        "reset_type_name": log.reset_type,
        "reset_time": log.reset_time.timestamp(),
        "old_traffic": {
            "upload": log.old_upload,
            "download": log.old_download,
            "total": log.old_total,
            "formatted": format_traffic_amount(log.old_total),
        },
        "new_traffic": {
            "upload": log.new_upload,
            "download": log.new_download,
            "total": log.new_total,
            "formatted": format_traffic_amount(log.new_total),
        },
        "trigger_source": log.trigger_source,
        "trigger_source_name": log.trigger_source,
        "metadata": log.metadata.as_ref().map(|value| value.0.clone()).unwrap_or(Value::Null),
        "created_at": log.created_at.timestamp(),
    })
}

fn serialize_admin_traffic_reset_history_item(log: &TrafficResetLogAdminRow) -> Value {
    json!({
        "id": log.id,
        "reset_type": log.reset_type,
        "reset_type_name": log.reset_type,
        "reset_time": log.reset_time.timestamp(),
        "old_traffic": {
            "upload": log.old_upload,
            "download": log.old_download,
            "total": log.old_total,
            "formatted": format_traffic_amount(log.old_total),
        },
        "trigger_source": log.trigger_source,
        "trigger_source_name": log.trigger_source,
        "metadata": log.metadata.as_ref().map(|value| value.0.clone()).unwrap_or(Value::Null),
    })
}

fn user_is_active_for_traffic_reset(user: &BearerUserRow) -> bool {
    user.banned == 0
        && user.plan_id.is_some()
        && user.expired_at.map(|value| value > Utc::now().timestamp()).unwrap_or(true)
}

fn is_duplicate_sqlx_error(err: &sqlx::Error) -> bool {
    if let sqlx::Error::Database(db_err) = err {
        if db_err.code().as_deref() == Some("23000") {
            return true;
        }
        if db_err.code().as_deref() == Some("19") {
            return true;
        }
        let msg = db_err.message().to_ascii_lowercase();
        return msg.contains("duplicate") || msg.contains("unique");
    }
    false
}

fn parse_f64_value(value: &Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_i64().map(|v| v as f64))
        .or_else(|| value.as_str().and_then(|v| v.trim().parse::<f64>().ok()))
}

fn order_to_value(order: UserOrderRow) -> Value {
    json!({
        "id": order.id,
        "user_id": order.user_id,
        "plan_id": order.plan_id,
        "payment_id": order.payment_id,
        "period": legacy_period(&order.period),
        "trade_no": order.trade_no,
        "total_amount": order.total_amount,
        "handling_amount": order.handling_amount,
        "balance_amount": order.balance_amount,
        "refund_amount": order.refund_amount,
        "surplus_amount": order.surplus_amount,
        "discount_amount": order.discount_amount,
        "type": order.type_field,
        "status": order.status,
        "surplus_order_ids": order.surplus_order_ids,
        "coupon_id": order.coupon_id,
        "created_at": order.created_at,
        "updated_at": order.updated_at,
        "commission_status": order.commission_status,
        "invite_user_id": order.invite_user_id,
        "actual_commission_balance": order.actual_commission_balance,
        "commission_balance": order.commission_balance,
        "paid_at": order.paid_at,
        "callback_no": order.callback_no,
        "plan": order.plan_name.as_ref().map(|name| json!({
            "id": order.plan_id,
            "name": name,
            "scope": order.plan_scope.clone().unwrap_or_else(|| "legacy".to_string()),
        })).unwrap_or(Value::Null),
    })
}

fn legacy_period(period: &str) -> String {
    match period {
        "monthly" => "month_price",
        "quarterly" => "quarter_price",
        "half_yearly" => "half_year_price",
        "yearly" => "year_price",
        "two_yearly" => "two_year_price",
        "three_yearly" => "three_year_price",
        "onetime" => "onetime_price",
        "reset_traffic" => "reset_price",
        _ => period,
    }
    .to_string()
}

fn parse_order_id_list(raw: &str) -> Option<Vec<i64>> {
    if raw.trim().is_empty() {
        return None;
    }
    serde_json::from_str::<Vec<i64>>(raw).ok()
}

fn sanctum_plaintext_token() -> String {
    let entropy = random_alnum(40);
    let crc = format!("{:x}", crc32fast::hash(entropy.as_bytes()));
    format!("{}{}", entropy, crc)
}

fn random_alnum(len: usize) -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut bytes = vec![0_u8; len];
    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        let _ = file.read_exact(&mut bytes);
    }
    bytes
        .into_iter()
        .map(|byte| CHARS[(byte as usize) % CHARS.len()] as char)
        .collect()
}

fn random_hex(len: usize) -> String {
    let bytes_len = len.div_ceil(2);
    let mut bytes = vec![0_u8; bytes_len];
    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        let _ = file.read_exact(&mut bytes);
    }
    let mut out = hex_encode(&bytes);
    out.truncate(len);
    out
}

fn random_letters(len: usize) -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut bytes = vec![0_u8; len];
    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        let _ = file.read_exact(&mut bytes);
    }
    bytes
        .into_iter()
        .map(|byte| CHARS[(byte as usize) % CHARS.len()] as char)
        .collect()
}

pub(crate) fn random_uuid_string() -> String {
    Uuid::new_v4().to_string()
}

fn random_digits(len: usize) -> String {
    let mut bytes = vec![0_u8; len];
    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        let _ = file.read_exact(&mut bytes);
    }
    bytes
        .into_iter()
        .map(|byte| char::from(b'0' + (byte % 10)))
        .collect()
}

fn extract_ja3(headers: &HeaderMap) -> Option<String> {
    for key in ["x-ja3-fingerprint", "x-ja3", "x-ssl-ja3", "x-ja3-hash", "cf-ja3-hash"] {
        if let Some(value) = headers.get(key).and_then(|value| value.to_str().ok()) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn request_ip(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn random_seed_hex(seed_salt: &str) -> Result<String, Response<Body>> {
    let mut bytes = [0_u8; 32];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut file| file.read_exact(&mut bytes))
        .map_err(|err| {
            error!("random seed read failed: {}", err);
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "random source unavailable")
        })?;
    Ok(sha256_hex_bytes(&[&bytes[..], seed_salt.as_bytes()].concat()))
}

fn build_pow_token(app_key: &str, ja3: &str, issued_at: i64, seed: &str) -> String {
    let message = format!("{}|{}|{}", ja3, issued_at, seed);
    let mac = hmac_sha256::HMAC::mac(message.as_bytes(), &app_key_bytes(app_key));
    hex_encode(&mac)
}

fn sha256_hex(value: &str) -> String {
    sha256_hex_bytes(value.as_bytes())
}

fn sha256_hex_bytes(value: &[u8]) -> String {
    let digest = sha2::Sha256::digest(value);
    hex_encode(&digest)
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{:02x}", byte)).collect()
}

async fn write_pow_challenge_cache(
    state: &AppState,
    challenge_id: &str,
    payload: &PowChallengeCacheEntry,
    ttl_seconds: i64,
) -> Result<(), String> {
    let key = format!("{}{}pow_challenge:{}", state.redis_prefix, state.cache_prefix, challenge_id);
    let value = php_serialize_pow_challenge(payload);
    redis_setex(state, &key, ttl_seconds, &value).await
}

fn php_serialize_pow_challenge(payload: &PowChallengeCacheEntry) -> String {
    let ja3_hash = match &payload.ja3_hash {
        Some(value) => php_serialize_string(value),
        None => "N;".to_string(),
    };
    format!(
        "a:8:{{s:4:\"seed\";{}s:4:\"base\";{}s:10:\"difficulty\";i:{};s:9:\"issued_at\";i:{};s:10:\"expires_at\";i:{};s:5:\"token\";{}s:8:\"ja3_hash\";{}s:2:\"ip\";{}}}",
        php_serialize_string(&payload.seed),
        php_serialize_string(&payload.base),
        payload.difficulty,
        payload.issued_at,
        payload.expires_at,
        php_serialize_string(&payload.token),
        ja3_hash,
        php_serialize_string(&payload.ip),
    )
}

fn php_serialize_string(value: &str) -> String {
    format!("s:{}:\"{}\";", value.len(), value)
}

async fn redis_setex(state: &AppState, key: &str, ttl_seconds: i64, value: &str) -> Result<(), String> {
    let addr = format!("{}:{}", state.redis_host, state.redis_port);
    let mut stream = TcpStream::connect(&addr)
        .await
        .map_err(|err| format!("connect redis failed: {}", err))?;

    if let Some(password) = &state.redis_password {
        let auth = redis_resp_array(&[b"AUTH".as_slice(), password.as_bytes()]);
        stream.write_all(auth.as_bytes()).await.map_err(|err| format!("redis AUTH write failed: {}", err))?;
        let reply = redis_read_reply(&mut stream).await?;
        if !reply.starts_with("+OK") {
            return Err(format!("redis AUTH failed: {}", reply.trim()));
        }
    }

    let select = redis_resp_array(&[b"SELECT".as_slice(), state.redis_cache_db.to_string().as_bytes()]);
    stream.write_all(select.as_bytes()).await.map_err(|err| format!("redis SELECT write failed: {}", err))?;
    let reply = redis_read_reply(&mut stream).await?;
    if !reply.starts_with("+OK") {
        return Err(format!("redis SELECT failed: {}", reply.trim()));
    }

    let setex = redis_resp_array(&[
        b"SETEX".as_slice(),
        key.as_bytes(),
        ttl_seconds.to_string().as_bytes(),
        value.as_bytes(),
    ]);
    stream.write_all(setex.as_bytes()).await.map_err(|err| format!("redis SETEX write failed: {}", err))?;
    let reply = redis_read_reply(&mut stream).await?;
    if !reply.starts_with("+OK") {
        return Err(format!("redis SETEX failed: {}", reply.trim()));
    }

    Ok(())
}

pub(crate) async fn redis_setex_string(state: &AppState, key: &str, ttl_seconds: i64, value: &str) -> Result<(), String> {
    let key = format!("{}{}{}", state.redis_prefix, state.cache_prefix, key);
    redis_setex(state, &key, ttl_seconds, value).await
}

pub(crate) async fn redis_get_string(state: &AppState, key: &str) -> Result<Option<String>, String> {
    let key = format!("{}{}{}", state.redis_prefix, state.cache_prefix, key);
    redis_get_string_raw(state, &key).await
}

async fn redis_get_string_raw(state: &AppState, key: &str) -> Result<Option<String>, String> {
    let addr = format!("{}:{}", state.redis_host, state.redis_port);
    let mut stream = TcpStream::connect(&addr)
        .await
        .map_err(|err| format!("connect redis failed: {}", err))?;

    if let Some(password) = &state.redis_password {
        let auth = redis_resp_array(&[b"AUTH".as_slice(), password.as_bytes()]);
        stream.write_all(auth.as_bytes()).await.map_err(|err| format!("redis AUTH write failed: {}", err))?;
        let reply = redis_read_reply(&mut stream).await?;
        if !reply.starts_with("+OK") {
            return Err(format!("redis AUTH failed: {}", reply.trim()));
        }
    }

    let select = redis_resp_array(&[b"SELECT".as_slice(), state.redis_cache_db.to_string().as_bytes()]);
    stream.write_all(select.as_bytes()).await.map_err(|err| format!("redis SELECT write failed: {}", err))?;
    let reply = redis_read_reply(&mut stream).await?;
    if !reply.starts_with("+OK") {
        return Err(format!("redis SELECT failed: {}", reply.trim()));
    }

    let get = redis_resp_array(&[b"GET".as_slice(), key.as_bytes()]);
    stream.write_all(get.as_bytes()).await.map_err(|err| format!("redis GET write failed: {}", err))?;
    let reply = redis_read_reply(&mut stream).await?;
    if reply.starts_with("$-1") {
        return Ok(None);
    }
    if !reply.starts_with('$') {
        return Err(format!("redis GET failed: {}", reply.trim()));
    }
    let mut lines = reply.splitn(3, "\r\n");
    let _bulk = lines.next();
    let data = lines.next().unwrap_or_default().to_string();
    Ok(Some(data))
}

async fn redis_del_key(state: &AppState, key: &str) -> Result<(), String> {
    let addr = format!("{}:{}", state.redis_host, state.redis_port);
    let mut stream = TcpStream::connect(&addr)
        .await
        .map_err(|err| format!("connect redis failed: {}", err))?;

    if let Some(password) = &state.redis_password {
        let auth = redis_resp_array(&[b"AUTH".as_slice(), password.as_bytes()]);
        stream.write_all(auth.as_bytes()).await.map_err(|err| format!("redis AUTH write failed: {}", err))?;
        let reply = redis_read_reply(&mut stream).await?;
        if !reply.starts_with("+OK") {
            return Err(format!("redis AUTH failed: {}", reply.trim()));
        }
    }

    let select = redis_resp_array(&[b"SELECT".as_slice(), state.redis_cache_db.to_string().as_bytes()]);
    stream.write_all(select.as_bytes()).await.map_err(|err| format!("redis SELECT write failed: {}", err))?;
    let reply = redis_read_reply(&mut stream).await?;
    if !reply.starts_with("+OK") {
        return Err(format!("redis SELECT failed: {}", reply.trim()));
    }

    let del = redis_resp_array(&[b"DEL".as_slice(), key.as_bytes()]);
    stream.write_all(del.as_bytes()).await.map_err(|err| format!("redis DEL write failed: {}", err))?;
    let reply = redis_read_reply(&mut stream).await?;
    if !reply.starts_with(':') {
        return Err(format!("redis DEL failed: {}", reply.trim()));
    }
    Ok(())
}

pub(crate) async fn redis_set_nx_ex_raw(
    state: &AppState,
    key: &str,
    ttl_seconds: i64,
    value: &str,
) -> Result<bool, String> {
    let addr = format!("{}:{}", state.redis_host, state.redis_port);
    let mut stream = TcpStream::connect(&addr)
        .await
        .map_err(|err| format!("connect redis failed: {}", err))?;

    if let Some(password) = &state.redis_password {
        let auth = redis_resp_array(&[b"AUTH".as_slice(), password.as_bytes()]);
        stream.write_all(auth.as_bytes()).await.map_err(|err| format!("redis AUTH write failed: {}", err))?;
        let reply = redis_read_reply(&mut stream).await?;
        if !reply.starts_with("+OK") {
            return Err(format!("redis AUTH failed: {}", reply.trim()));
        }
    }

    let select = redis_resp_array(&[b"SELECT".as_slice(), state.redis_cache_db.to_string().as_bytes()]);
    stream.write_all(select.as_bytes()).await.map_err(|err| format!("redis SELECT write failed: {}", err))?;
    let reply = redis_read_reply(&mut stream).await?;
    if !reply.starts_with("+OK") {
        return Err(format!("redis SELECT failed: {}", reply.trim()));
    }

    let set = redis_resp_array(&[
        b"SET".as_slice(),
        key.as_bytes(),
        value.as_bytes(),
        b"NX".as_slice(),
        b"EX".as_slice(),
        ttl_seconds.to_string().as_bytes(),
    ]);
    stream.write_all(set.as_bytes()).await.map_err(|err| format!("redis SET NX EX write failed: {}", err))?;
    let reply = redis_read_reply(&mut stream).await?;
    if reply.starts_with("+OK") {
        return Ok(true);
    }
    if reply.starts_with("$-1") || reply.trim() == "$-1" || reply.trim().is_empty() {
        return Ok(false);
    }
    if reply.trim() == "$-1\r\n" {
        return Ok(false);
    }
    if reply.trim() == "(nil)" {
        return Ok(false);
    }
    if reply.starts_with('*') || reply.starts_with('-') {
        return Err(format!("redis SET NX EX failed: {}", reply.trim()));
    }
    Ok(false)
}

pub(crate) async fn cleanup_expired_online_status(
    state: &AppState,
    stale_minutes: i64,
) -> Result<u64, sqlx::Error> {
    let threshold = stale_minutes.max(1);
    let result = sqlx::query(
        "UPDATE v2_user
         SET online_count = 0
         WHERE online_count > 0
           AND last_online_at IS NOT NULL
           AND last_online_at < DATE_SUB(NOW(), INTERVAL ? MINUTE)"
    )
    .bind(threshold)
    .execute(&state.db)
    .await?;
    Ok(result.rows_affected())
}

pub(crate) async fn cleanup_expired_node_sessions(
    state: &AppState,
    minutes: i64,
) -> Result<u64, sqlx::Error> {
    let threshold = minutes.max(1);
    let result = sqlx::query(
        "DELETE FROM user_online_sessions
         WHERE last_activity < DATE_SUB(NOW(), INTERVAL ? MINUTE)"
    )
    .bind(threshold)
    .execute(&state.db)
    .await?;
    Ok(result.rows_affected())
}

pub(crate) async fn check_server_nodes_offline(
    state: &AppState,
) -> Result<(u64, u64), sqlx::Error> {
    let nodes = load_active_server_node_monitor_rows(state).await?;
    let push_interval = get_setting_int(state, "server_push_interval", 60).await.max(60);
    let status_cache = state.load_status_cache.read().clone();
    let now = Utc::now().timestamp();
    let mut marked = 0_u64;
    let mut sent = 0_u64;

    for node in nodes {
        let identifier = format!("server-node:{}", node.id);
        let cache_key = offline_alert_cache_key(&identifier);
        let last_report_at = access_node_last_report_at(&status_cache, node.id);
        let is_online = last_report_at
            .map(|value| (now - value) <= push_interval * 3)
            .unwrap_or(false);

        if is_online {
            let _ = redis_del_key(state, &format!("{}{}{}", state.redis_prefix, state.cache_prefix, cache_key)).await;
            continue;
        }

        if last_report_at.is_none() && (now - node.created_at) < push_interval * 3 {
            continue;
        }

        let full_key = format!("{}{}{}", state.redis_prefix, state.cache_prefix, cache_key);
        if redis_set_nx_ex_raw(state, &full_key, 3600, &now.to_string()).await.map_err(to_sqlx_redis_error)? {
            marked += 1;
            if send_server_node_offline_ops_alert(state, &node, last_report_at, now).await.unwrap_or(false) {
                sent += 1;
            }
        }
    }

    Ok((marked, sent))
}

pub(crate) async fn rotate_subscription_credentials_daily(
    state: &AppState,
    now_ts: i64,
) -> Result<u64, sqlx::Error> {
    if !get_setting_bool(state, "rotate_subscription_credentials_daily", false).await {
        return Ok(0);
    }

    let tz = chrono::FixedOffset::east_opt(8 * 3600).ok_or_else(|| sqlx::Error::Protocol("invalid timezone offset".to_string()))?;
    let now = chrono::DateTime::from_timestamp(now_ts, 0)
        .ok_or_else(|| sqlx::Error::Protocol("invalid current timestamp".to_string()))?
        .with_timezone(&tz);
    let scheduled = tz
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 1, 10, 0)
        .single()
        .ok_or_else(|| sqlx::Error::Protocol("invalid rotation schedule".to_string()))?;
    if now < scheduled {
        return Ok(0);
    }

    let already_rotated_today: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM v2_user
         WHERE last_subscription_credential_rotation_at IS NOT NULL
           AND last_subscription_credential_rotation_at >= ?"
    )
    .bind(scheduled.timestamp())
    .fetch_one(&state.db)
    .await?;
    if already_rotated_today > 0 {
        return Ok(0);
    }

    let result = sqlx::query(
        "UPDATE v2_user
         SET subscription_credential_version = GREATEST(COALESCE(subscription_credential_version, 0), 0) + 1,
             last_subscription_credential_rotation_at = ?,
             updated_at = ?
         WHERE id > 0"
    )
    .bind(now_ts)
    .bind(now_ts)
    .execute(&state.db)
    .await?;
    Ok(result.rows_affected())
}

async fn load_active_server_node_monitor_rows(
    state: &AppState,
) -> Result<Vec<ServerNodeMonitorRow>, sqlx::Error> {
    sqlx::query_as::<_, ServerNodeMonitorRow>(
        "SELECT id,
                name,
                host,
                protocol,
                CAST(UNIX_TIMESTAMP(created_at) AS SIGNED) AS created_at
         FROM server_nodes
         WHERE status = 'active'
         ORDER BY id ASC"
    )
    .fetch_all(&state.db)
    .await
}

fn offline_alert_cache_key(identifier: &str) -> String {
    format!("ops:server-offline-alert:{:x}", md5::compute(identifier))
}

fn to_sqlx_redis_error(err: String) -> sqlx::Error {
    sqlx::Error::Protocol(err)
}

async fn send_server_node_offline_ops_alert(
    state: &AppState,
    node: &ServerNodeMonitorRow,
    last_report_at: Option<i64>,
    now_ts: i64,
) -> Result<bool, String> {
    let text = format!(
        "V2bX 节点离线\n节点名称：{}\n节点地址：{}\n协议：{}\n最后上报：{}\n时间：{}",
        node.name,
        node.host,
        node.protocol,
        last_report_at
            .and_then(|value| chrono::DateTime::from_timestamp(value, 0))
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| "never".to_string()),
        chrono::DateTime::from_timestamp(now_ts, 0)
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| now_ts.to_string())
    );
    crate::ops_alert_support::send_ops_alert(state, &text).await
}

pub(crate) async fn check_orders(
    state: &AppState,
    limit: i64,
) -> Result<(u64, u64), sqlx::Error> {
    let rows = load_schedulable_orders(state, limit).await?;

    let mut cancelled = 0_u64;
    let mut completed = 0_u64;
    for row in rows {
        if row.trade_no.is_empty() {
            continue;
        }
        let action = handle_single_order_check(state, &row).await?;
        match action.as_deref() {
            Some("cancel") => cancelled += 1,
            Some("open") => completed += 1,
            _ => {}
        }
    }
    Ok((cancelled, completed))
}

pub(crate) async fn check_tickets(
    state: &AppState,
    limit: i64,
) -> Result<u64, sqlx::Error> {
    let now = Utc::now().timestamp();
    let result = sqlx::query(
        "UPDATE v2_ticket
         SET status = 1,
             updated_at = ?
         WHERE status = 0
           AND reply_status = 0
           AND updated_at <= ?
           AND (last_reply_user_id IS NULL OR user_id <> last_reply_user_id)
         LIMIT ?"
    )
    .bind(now)
    .bind(now - 86_400)
    .bind(limit.max(1))
    .execute(&state.db)
    .await?;
    Ok(result.rows_affected())
}

pub(crate) async fn auto_check_commissions(
    state: &AppState,
) -> Result<u64, sqlx::Error> {
    if !get_setting_bool(state, "commission_auto_check_enable", true).await {
        return Ok(0);
    }
    let result = sqlx::query(
        "UPDATE v2_order
         SET commission_status = 1
         WHERE commission_status = 0
           AND invite_user_id IS NOT NULL
           AND status = 3
           AND updated_at <= ?"
    )
    .bind(Utc::now().timestamp() - 259_200)
    .execute(&state.db)
    .await?;
    Ok(result.rows_affected())
}

pub(crate) async fn auto_pay_commissions(
    state: &AppState,
    limit: i64,
) -> Result<u64, sqlx::Error> {
    let config = load_commission_payout_config(state).await;
    let order_ids = sqlx::query_scalar::<_, i64>(
        "SELECT id
         FROM v2_order
         WHERE commission_status = 1
           AND invite_user_id IS NOT NULL
         ORDER BY id ASC
         LIMIT ?"
    )
    .bind(limit.max(1))
    .fetch_all(&state.db)
    .await?;

    let mut paid = 0_u64;
    for order_id in order_ids {
        if pay_single_order_commission(state, order_id, &config).await? {
            paid += 1;
        }
    }
    Ok(paid)
}

async fn pay_single_order_commission(
    state: &AppState,
    order_id: i64,
    config: &CommissionPayoutConfig,
) -> Result<bool, sqlx::Error> {
    let mut tx = state.db.begin().await?;
    let order = sqlx::query(
        "SELECT id, invite_user_id, user_id, trade_no, total_amount, commission_balance, actual_commission_balance, commission_status
         FROM v2_order
         WHERE id = ?
         LIMIT 1
         FOR UPDATE"
    )
    .bind(order_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(order) = order else {
        tx.commit().await?;
        return Ok(false);
    };

    let commission_status = order.try_get::<i64, _>("commission_status").unwrap_or_default();
    let invite_user_id = order.try_get::<Option<i64>, _>("invite_user_id").unwrap_or(None);
    if commission_status != 1 || invite_user_id.is_none() {
        tx.commit().await?;
        return Ok(false);
    }

    let trade_no = order.try_get::<String, _>("trade_no").unwrap_or_default();
    let user_id = order.try_get::<i64, _>("user_id").unwrap_or_default();
    let order_amount = order.try_get::<i64, _>("total_amount").unwrap_or_default();
    let order_commission_balance = order.try_get::<i64, _>("commission_balance").unwrap_or_default();
    let mut actual_commission_balance = order.try_get::<Option<i64>, _>("actual_commission_balance").unwrap_or(None).unwrap_or(0);
    let inviter_chain = load_inviter_chain_with_tx(
        &mut tx,
        invite_user_id.unwrap_or_default(),
        config.distribution_levels.len(),
    )
    .await?;
    let existing_inviter_ids = load_existing_commission_inviter_ids_with_tx(
        &mut tx,
        &inviter_chain,
        user_id,
        &trade_no,
    )
    .await?;

    for (index, ratio) in config.distribution_levels.iter().enumerate() {
        let Some(current_invite_user_id) = inviter_chain.get(index).copied() else {
            break;
        };
        if *ratio <= 0 || order_commission_balance <= 0 {
            continue;
        }

        let commission_get_amount = ((order_commission_balance as f64) * ((*ratio as f64) / 100.0)).round() as i64;
        if commission_get_amount <= 0 {
            continue;
        }

        if existing_inviter_ids.contains(&current_invite_user_id) {
            continue;
        }

        if config.withdraw_close_enable {
            sqlx::query(
                "UPDATE v2_user
                 SET balance = balance + ?,
                     updated_at = ?
                 WHERE id = ?"
            )
            .bind(commission_get_amount)
            .bind(Utc::now().timestamp())
            .bind(current_invite_user_id)
            .execute(&mut *tx)
            .await?;
        } else {
            sqlx::query(
                "UPDATE v2_user
                 SET commission_balance = commission_balance + ?,
                     updated_at = ?
                 WHERE id = ?"
            )
            .bind(commission_get_amount)
            .bind(Utc::now().timestamp())
            .bind(current_invite_user_id)
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query(
            "INSERT INTO v2_commission_log
                (invite_user_id, user_id, trade_no, order_amount, get_amount, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(current_invite_user_id)
        .bind(user_id)
        .bind(&trade_no)
        .bind(order_amount)
        .bind(commission_get_amount)
        .bind(Utc::now().timestamp())
        .bind(Utc::now().timestamp())
        .execute(&mut *tx)
        .await?;

        actual_commission_balance += commission_get_amount;
    }

    sqlx::query(
        "UPDATE v2_order
         SET commission_status = 2,
             actual_commission_balance = ?,
             updated_at = ?
         WHERE id = ? AND commission_status = 1"
    )
    .bind(actual_commission_balance)
    .bind(Utc::now().timestamp())
    .bind(order_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(true)
}

async fn handle_single_order_check(
    state: &AppState,
    order: &SchedulableOrderRow,
) -> Result<Option<&'static str>, sqlx::Error> {
    if order.trade_no.is_empty() {
        return Ok(None);
    }

    let status = order.status;
    if status == 0 {
        let created_at = order.created_at;
        if created_at <= Utc::now().timestamp() - 7200 {
            if cancel_pending_order_for_scheduler(state, order).await? {
                return Ok(Some("cancel"));
            }
        }
        return Ok(None);
    }

    if status == 1 {
        complete_processing_order_by_id(state, order.id).await?;
        return Ok(Some("open"));
    }

    Ok(None)
}

async fn cancel_pending_order_for_scheduler(
    state: &AppState,
    order: &SchedulableOrderRow,
) -> Result<bool, sqlx::Error> {
    let mut tx = state.db.begin().await?;
    let locked = sqlx::query(
        "SELECT user_id, status, balance_amount
         FROM v2_order
         WHERE id = ?
         LIMIT 1
         FOR UPDATE"
    )
    .bind(order.id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(locked) = locked else {
        tx.commit().await?;
        return Ok(false);
    };

    let status = locked.try_get::<i64, _>("status").unwrap_or_default();
    if status != 0 {
        tx.commit().await?;
        return Ok(false);
    }

    let order_id = order.id;
    let user_id = locked.try_get::<i64, _>("user_id").unwrap_or(order.user_id);
    let balance_amount = locked
        .try_get::<Option<i64>, _>("balance_amount")
        .unwrap_or(order.balance_amount)
        .unwrap_or(0);
    if balance_amount > 0 {
        sqlx::query(
            "UPDATE v2_user
             SET balance = balance + ?, updated_at = ?
             WHERE id = ?"
        )
        .bind(balance_amount)
        .bind(Utc::now().timestamp())
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    }

    let updated = sqlx::query(
        "UPDATE v2_order
         SET status = 2, updated_at = ?
         WHERE id = ? AND status = 0"
    )
    .bind(Utc::now().timestamp())
    .bind(order_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(updated.rows_affected() == 1)
}

pub(crate) async fn finalize_expired_refund_votings(
    state: &AppState,
    limit: i64,
) -> Result<u64, sqlx::Error> {
    let now = Utc::now();
    let requests = load_expired_voting_refund_requests(state, now, limit.max(1))
        .await?;
    let mut processed = 0_u64;
    for req in requests {
        finalize_single_expired_refund_voting(state, req.id, now).await?;
        processed += 1;
    }
    Ok(processed)
}

async fn redis_rpush_with_notify(
    state: &AppState,
    queue_key: &str,
    notify_key: &str,
    payload: &str,
) -> Result<(), String> {
    let addr = format!("{}:{}", state.redis_host, state.redis_port);
    let mut stream = TcpStream::connect(&addr)
        .await
        .map_err(|err| format!("connect redis failed: {}", err))?;

    if let Some(password) = &state.redis_password {
        let auth = redis_resp_array(&[b"AUTH".as_slice(), password.as_bytes()]);
        stream.write_all(auth.as_bytes()).await.map_err(|err| format!("redis AUTH write failed: {}", err))?;
        let reply = redis_read_reply(&mut stream).await?;
        if !reply.starts_with("+OK") {
            return Err(format!("redis AUTH failed: {}", reply.trim()));
        }
    }

    let select = redis_resp_array(&[b"SELECT".as_slice(), b"0"]);
    stream.write_all(select.as_bytes()).await.map_err(|err| format!("redis SELECT write failed: {}", err))?;
    let reply = redis_read_reply(&mut stream).await?;
    if !reply.starts_with("+OK") {
        return Err(format!("redis SELECT failed: {}", reply.trim()));
    }

    let queue_full = format!("{}{}", state.redis_prefix, queue_key);
    let notify_full = format!("{}{}", state.redis_prefix, notify_key);
    let rpush_job = redis_resp_array(&[b"RPUSH".as_slice(), queue_full.as_bytes(), payload.as_bytes()]);
    stream.write_all(rpush_job.as_bytes()).await.map_err(|err| format!("redis RPUSH job write failed: {}", err))?;
    let reply = redis_read_reply(&mut stream).await?;
    if !reply.starts_with(':') {
        return Err(format!("redis RPUSH job failed: {}", reply.trim()));
    }

    let rpush_notify = redis_resp_array(&[b"RPUSH".as_slice(), notify_full.as_bytes(), b"1"]);
    stream.write_all(rpush_notify.as_bytes()).await.map_err(|err| format!("redis RPUSH notify write failed: {}", err))?;
    let reply = redis_read_reply(&mut stream).await?;
    if !reply.starts_with(':') {
        return Err(format!("redis RPUSH notify failed: {}", reply.trim()));
    }

    Ok(())
}

fn redis_resp_array(parts: &[&[u8]]) -> String {
    let mut out = format!("*{}\r\n", parts.len());
    for part in parts {
        out.push_str(&format!("${}\r\n", part.len()));
        out.push_str(&String::from_utf8_lossy(part));
        out.push_str("\r\n");
    }
    out
}

async fn redis_read_reply(stream: &mut TcpStream) -> Result<String, String> {
    let mut buffer = vec![0_u8; 1024];
    let read = stream.read(&mut buffer).await.map_err(|err| format!("redis read failed: {}", err))?;
    if read == 0 {
        return Err("redis closed connection".to_string());
    }
    Ok(String::from_utf8_lossy(&buffer[..read]).to_string())
}

async fn build_singbox_json_payload(
    state: &AppState,
    _user: &UserRow,
    servers: &[ServerNodeRow],
    uuid: &str,
) -> Result<String, Response<Body>> {
    let mut config: Value = serde_json::from_str(SINGBOX_DEFAULT_TEMPLATE)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "invalid sing-box template"))?;

    let outbounds = config
        .get_mut("outbounds")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| json_error(StatusCode::INTERNAL_SERVER_ERROR, "invalid sing-box outbounds"))?;

    let mut tags = Vec::new();
    for server in servers {
        let normalized_type = normalize_type(&server.protocol).unwrap_or_default();
        let settings = normalized_protocol_settings(
            &normalized_type,
            server
                .settings
                .as_ref()
                .and_then(|json| json.0.as_object().cloned())
                .unwrap_or_default(),
        );

        let outbound = match normalized_type.as_str() {
            "shadowsocks" => Some(build_singbox_shadowsocks(uuid, server, &settings, &state.app_key)),
            "trojan" => Some(build_singbox_trojan(uuid, server, &settings)),
            "vmess" => Some(build_singbox_vmess(uuid, server, &settings)),
            "vless" => Some(build_singbox_vless(uuid, server, &settings)),
            "hysteria" => Some(build_singbox_hysteria(uuid, server, &settings)),
            "tuic" => Some(build_singbox_tuic(uuid, server, &settings)),
            "anytls" => Some(build_singbox_anytls(uuid, server, &settings)),
            "socks" => Some(build_singbox_socks(uuid, server, &settings)),
            "http" => Some(build_singbox_http(uuid, server, &settings)),
            _ => None,
        };

        if let Some(outbound) = outbound {
            if let Some(tag) = outbound.get("tag").and_then(|v| v.as_str()) {
                tags.push(tag.to_string());
            }
            outbounds.push(outbound);
        }
    }

    for outbound in outbounds.iter_mut() {
        if let Some(kind) = outbound.get("type").and_then(|v| v.as_str()) {
            if matches!(kind, "urltest" | "selector") {
                if let Some(list) = outbound.get_mut("outbounds").and_then(|v| v.as_array_mut()) {
                    list.extend(tags.iter().cloned().map(Value::String));
                }
            }
        }
    }

    serde_json::to_string_pretty(&config)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "sing-box serialize failed"))
}

async fn build_clash_proxies(
    servers: &[ServerNodeRow],
    uuid: &str,
    state: &AppState,
) -> Vec<serde_yaml::Value> {
    let mut result = Vec::new();
    for server in servers {
        let normalized_type = normalize_type(&server.protocol).unwrap_or_default();
        let settings = normalized_protocol_settings(
            &normalized_type,
            server
                .settings
                .as_ref()
                .and_then(|json| json.0.as_object().cloned())
                .unwrap_or_default(),
        );
        let proxy = match normalized_type.as_str() {
            "shadowsocks" => Some(build_clash_shadowsocks(uuid, server, &settings, &state.app_key)),
            "vmess" => Some(build_clash_vmess(uuid, server, &settings)),
            "trojan" => Some(build_clash_trojan(uuid, server, &settings)),
            "socks" => Some(build_clash_socks5(uuid, server)),
            "http" => Some(build_clash_http(uuid, server)),
            _ => None,
        };
        if let Some(proxy) = proxy {
            result.push(proxy);
        }
    }
    result
}

fn build_userinfo_header(user: &UserRow) -> Option<String> {
    Some(format!(
        "subscription-userinfo: upload={}; download={}; total={}; expire={}",
        user.u.unwrap_or(0),
        user.d.unwrap_or(0),
        user.transfer_enable.unwrap_or(0),
        user.expired_at.unwrap_or(0)
    ))
}

fn build_general_shadowsocks(
    uuid: &str,
    server: &ServerNodeRow,
    name: &str,
    settings: &Map<String, Value>,
    app_key: &str,
) -> String {
    let cipher = json_string(settings, "cipher", "aes-128-gcm");
    let password = subscribe_shadowsocks_password(uuid, server, settings, app_key);
    let user_info = format!("{}:{}", cipher, password);
    let encoded = url_safe_base64(user_info.as_bytes());
    let host = wrap_ipv6(&server.host);
    format!("ss://{}@{}:{}#{}\r\n", encoded, host, server.port, urlencoding::encode(name))
}

fn build_general_vmess(uuid: &str, server: &ServerNodeRow, name: &str, settings: &Map<String, Value>) -> String {
    let mut config = json!({
        "v": "2",
        "ps": name,
        "add": server.host,
        "port": server.port.to_string(),
        "id": uuid,
        "aid": "0",
        "net": json_string(settings, "network", "tcp"),
        "type": "none",
        "host": "",
        "path": "",
        "tls": if json_int(settings, "tls") > 0 { "tls" } else { "" },
    });

    if let Some(server_name) = settings.get("tls_settings").and_then(|v| v.get("server_name")).and_then(|v| v.as_str()) {
        config["sni"] = Value::String(server_name.to_string());
    }

    match json_string(settings, "network", "tcp").as_str() {
        "ws" => {
            config["type"] = Value::String("ws".to_string());
            if let Some(path) = settings.get("network_settings").and_then(|v| v.get("path")).and_then(|v| v.as_str()) {
                config["path"] = Value::String(path.to_string());
            }
            if let Some(host) = settings.get("network_settings").and_then(|v| v.get("headers")).and_then(|v| v.get("Host")).and_then(|v| v.as_str()) {
                config["host"] = Value::String(host.to_string());
            }
        }
        "grpc" => {
            config["type"] = Value::String("grpc".to_string());
            if let Some(service_name) = settings.get("network_settings").and_then(|v| v.get("serviceName")).and_then(|v| v.as_str()) {
                config["path"] = Value::String(service_name.to_string());
            }
        }
        _ => {}
    }

    format!("vmess://{}\r\n", BASE64_STANDARD.encode(config.to_string()))
}

fn build_general_vless(uuid: &str, server: &ServerNodeRow, name: &str, settings: &Map<String, Value>) -> String {
    let mut params = vec![
        ("encryption".to_string(), "none".to_string()),
        ("type".to_string(), json_string(settings, "network", "tcp")),
    ];
    let tls = json_int(settings, "tls");
    if tls == 1 {
        params.push(("security".to_string(), "tls".to_string()));
        if let Some(server_name) = settings.get("tls_settings").and_then(|v| v.get("server_name")).and_then(|v| v.as_str()) {
            params.push(("sni".to_string(), server_name.to_string()));
        }
    } else if tls == 2 {
        params.push(("security".to_string(), "reality".to_string()));
        if let Some(value) = settings.get("reality_settings").and_then(|v| v.get("public_key")).and_then(|v| v.as_str()) {
            params.push(("pbk".to_string(), value.to_string()));
        }
        if let Some(value) = settings.get("reality_settings").and_then(|v| v.get("short_id")).and_then(|v| v.as_str()) {
            params.push(("sid".to_string(), value.to_string()));
        }
        if let Some(value) = settings.get("reality_settings").and_then(|v| v.get("server_name")).and_then(|v| v.as_str()) {
            params.push(("sni".to_string(), value.to_string()));
            params.push(("servername".to_string(), value.to_string()));
        }
        params.push(("spx".to_string(), "/".to_string()));
        params.push(("fp".to_string(), "chrome".to_string()));
    }

    if let Some(flow) = settings.get("flow").and_then(|v| v.as_str()) {
        if !flow.is_empty() {
            params.push(("flow".to_string(), flow.to_string()));
        }
    }
    if let Some(path) = settings.get("network_settings").and_then(|v| v.get("path")).and_then(|v| v.as_str()) {
        params.push(("path".to_string(), path.to_string()));
    }
    if let Some(host) = settings.get("network_settings").and_then(|v| v.get("headers")).and_then(|v| v.get("Host")).and_then(|v| v.as_str()) {
        params.push(("host".to_string(), host.to_string()));
    }

    let query = serde_urlencoded::to_string(params).unwrap_or_default();
    format!(
        "vless://{}@{}:{}?{}#{}\r\n",
        uuid,
        wrap_ipv6(&server.host),
        server.port,
        query,
        urlencoding::encode(name)
    )
}

fn build_general_trojan(uuid: &str, server: &ServerNodeRow, name: &str, settings: &Map<String, Value>) -> String {
    let mut params = vec![(
        "allowInsecure".to_string(),
        settings.get("allow_insecure").and_then(|v| v.as_bool()).unwrap_or(false).to_string(),
    )];
    if let Some(server_name) = settings.get("server_name").and_then(|v| v.as_str()) {
        params.push(("peer".to_string(), server_name.to_string()));
        params.push(("sni".to_string(), server_name.to_string()));
    }
    if json_string(settings, "network", "tcp") == "ws" {
        params.push(("type".to_string(), "ws".to_string()));
        if let Some(path) = settings.get("network_settings").and_then(|v| v.get("path")).and_then(|v| v.as_str()) {
            params.push(("path".to_string(), path.to_string()));
        }
        if let Some(host) = settings.get("network_settings").and_then(|v| v.get("headers")).and_then(|v| v.get("Host")).and_then(|v| v.as_str()) {
            params.push(("host".to_string(), host.to_string()));
        }
    }
    let query = serde_urlencoded::to_string(params).unwrap_or_default();
    format!(
        "trojan://{}@{}:{}?{}#{}\r\n",
        uuid,
        wrap_ipv6(&server.host),
        server.port,
        query,
        urlencoding::encode(name)
    )
}

fn build_general_hysteria(uuid: &str, server: &ServerNodeRow, name: &str, settings: &Map<String, Value>) -> String {
    if json_int(settings, "version") != 2 {
        return String::new();
    }
    let mut params = Vec::new();
    if let Some(server_name) = settings.get("tls").and_then(|v| v.get("server_name")).and_then(|v| v.as_str()) {
        params.push(("sni".to_string(), server_name.to_string()));
        params.push(("security".to_string(), "tls".to_string()));
    }
    if settings.get("obfs").and_then(|v| v.get("open")).and_then(|v| v.as_bool()).unwrap_or(false) {
        params.push(("obfs".to_string(), "salamander".to_string()));
        if let Some(password) = settings.get("obfs").and_then(|v| v.get("password")).and_then(|v| v.as_str()) {
            params.push(("obfs-password".to_string(), password.to_string()));
        }
    }
    params.push((
        "insecure".to_string(),
        settings.get("tls").and_then(|v| v.get("allow_insecure")).and_then(|v| v.as_bool()).unwrap_or(false).to_string(),
    ));
    let query = serde_urlencoded::to_string(params).unwrap_or_default();
    format!(
        "hysteria2://{}@{}:{}?{}#{}\r\n",
        uuid,
        wrap_ipv6(&server.host),
        server.port,
        query,
        urlencoding::encode(name)
    )
}

fn build_general_socks(uuid: &str, server: &ServerNodeRow, name: &str) -> String {
    format!(
        "socks://{}@{}:{}#{}\r\n",
        BASE64_STANDARD.encode(format!("{}:{}", uuid, uuid)),
        server.host,
        server.port,
        urlencoding::encode(name)
    )
}

fn build_shadowrocket_shadowsocks(
    uuid: &str,
    server: &ServerNodeRow,
    name: &str,
    settings: &Map<String, Value>,
    app_key: &str,
) -> String {
    build_general_shadowsocks(uuid, server, name, settings, app_key)
}

fn build_shadowrocket_vmess(uuid: &str, server: &ServerNodeRow, name: &str, settings: &Map<String, Value>) -> String {
    let userinfo = BASE64_STANDARD.encode(format!("auto:{}@{}:{}", uuid, wrap_ipv6(&server.host), server.port));
    let mut params = vec![
        ("tfo".to_string(), "1".to_string()),
        ("remark".to_string(), name.to_string()),
        ("alterId".to_string(), "0".to_string()),
    ];

    if json_int(settings, "tls") > 0 {
        params.push(("tls".to_string(), "1".to_string()));
        if let Some(allow_insecure) = settings.get("tls_settings").and_then(|v| v.get("allow_insecure")).and_then(|v| v.as_bool()) {
            params.push(("allowInsecure".to_string(), (allow_insecure as i32).to_string()));
        }
        if let Some(server_name) = settings.get("tls_settings").and_then(|v| v.get("server_name")).and_then(|v| v.as_str()) {
            params.push(("peer".to_string(), server_name.to_string()));
        }
    }

    match json_string(settings, "network", "tcp").as_str() {
        "ws" => {
            params.push(("obfs".to_string(), "websocket".to_string()));
            if let Some(path) = settings.get("network_settings").and_then(|v| v.get("path")).and_then(|v| v.as_str()) {
                params.push(("path".to_string(), path.to_string()));
            }
            if let Some(host) = settings.get("network_settings").and_then(|v| v.get("headers")).and_then(|v| v.get("Host")).and_then(|v| v.as_str()) {
                params.push(("obfsParam".to_string(), host.to_string()));
            }
        }
        "grpc" => {
            params.push(("obfs".to_string(), "grpc".to_string()));
            if let Some(service_name) = settings.get("network_settings").and_then(|v| v.get("serviceName")).and_then(|v| v.as_str()) {
                params.push(("path".to_string(), service_name.to_string()));
            }
        }
        _ => {}
    }

    format!("vmess://{}?{}\r\n", userinfo, serde_urlencoded::to_string(params).unwrap_or_default())
}

fn build_shadowrocket_vless(uuid: &str, server: &ServerNodeRow, name: &str, settings: &Map<String, Value>) -> String {
    let userinfo = BASE64_STANDARD.encode(format!("auto:{}@{}:{}", uuid, wrap_ipv6(&server.host), server.port));
    let mut params = vec![
        ("tfo".to_string(), "1".to_string()),
        ("remark".to_string(), name.to_string()),
        ("alterId".to_string(), "0".to_string()),
    ];

    let tls = json_int(settings, "tls");
    if tls == 1 {
        params.push(("tls".to_string(), "1".to_string()));
        if let Some(server_name) = settings.get("tls_settings").and_then(|v| v.get("server_name")).and_then(|v| v.as_str()) {
            params.push(("peer".to_string(), server_name.to_string()));
        }
        if let Some(allow_insecure) = settings.get("tls_settings").and_then(|v| v.get("allow_insecure")).and_then(|v| v.as_bool()) {
            params.push(("allowInsecure".to_string(), (allow_insecure as i32).to_string()));
        }
    } else if tls == 2 {
        params.push(("tls".to_string(), "1".to_string()));
        if let Some(server_name) = settings.get("reality_settings").and_then(|v| v.get("server_name")).and_then(|v| v.as_str()) {
            params.push(("sni".to_string(), server_name.to_string()));
        }
        if let Some(value) = settings.get("reality_settings").and_then(|v| v.get("public_key")).and_then(|v| v.as_str()) {
            params.push(("pbk".to_string(), value.to_string()));
        }
        if let Some(value) = settings.get("reality_settings").and_then(|v| v.get("short_id")).and_then(|v| v.as_str()) {
            params.push(("sid".to_string(), value.to_string()));
        }
    }

    match json_string(settings, "network", "tcp").as_str() {
        "ws" => {
            params.push(("obfs".to_string(), "websocket".to_string()));
            if let Some(path) = settings.get("network_settings").and_then(|v| v.get("path")).and_then(|v| v.as_str()) {
                params.push(("path".to_string(), path.to_string()));
            }
            if let Some(host) = settings.get("network_settings").and_then(|v| v.get("headers")).and_then(|v| v.get("Host")).and_then(|v| v.as_str()) {
                params.push(("obfsParam".to_string(), host.to_string()));
            }
        }
        "grpc" => {
            params.push(("obfs".to_string(), "grpc".to_string()));
            if let Some(service_name) = settings.get("network_settings").and_then(|v| v.get("serviceName")).and_then(|v| v.as_str()) {
                params.push(("path".to_string(), service_name.to_string()));
            }
        }
        _ => {}
    }

    format!("vless://{}?{}\r\n", userinfo, serde_urlencoded::to_string(params).unwrap_or_default())
}

fn build_shadowrocket_trojan(uuid: &str, server: &ServerNodeRow, name: &str, settings: &Map<String, Value>) -> String {
    let mut params = vec![
        (
            "allowInsecure".to_string(),
            settings.get("allow_insecure").and_then(|v| v.as_bool()).unwrap_or(false).to_string(),
        ),
        ("tfo".to_string(), "1".to_string()),
    ];
    if let Some(server_name) = settings.get("server_name").and_then(|v| v.as_str()) {
        params.push(("peer".to_string(), server_name.to_string()));
    }
    if json_string(settings, "network", "tcp") == "grpc" {
        params.push(("obfs".to_string(), "grpc".to_string()));
        if let Some(service_name) = settings.get("network_settings").and_then(|v| v.get("serviceName")).and_then(|v| v.as_str()) {
            params.push(("path".to_string(), service_name.to_string()));
        }
    }
    if json_string(settings, "network", "tcp") == "ws" {
        if let Some(host) = settings.get("network_settings").and_then(|v| v.get("headers")).and_then(|v| v.get("Host")).and_then(|v| v.as_str()) {
            let path = settings.get("network_settings").and_then(|v| v.get("path")).and_then(|v| v.as_str()).unwrap_or("/");
            params.push(("plugin".to_string(), format!("obfs-local;obfs=websocket;obfs-host={};obfs-uri={}", host, path)));
        }
    }
    format!(
        "trojan://{}@{}:{}?{}#{}\r\n",
        uuid,
        wrap_ipv6(&server.host),
        server.port,
        serde_urlencoded::to_string(params).unwrap_or_default(),
        urlencoding::encode(name)
    )
}

fn build_quantumultx_shadowsocks(
    uuid: &str,
    server: &ServerNodeRow,
    name: &str,
    settings: &Map<String, Value>,
    app_key: &str,
) -> String {
    let cipher = json_string(settings, "cipher", "aes-128-gcm");
    let password = subscribe_shadowsocks_password(uuid, server, settings, app_key);

    let mut parts = vec![
        format!("shadowsocks={}:{}", server.host, server.port),
        format!("method={}", cipher),
        format!("password={}", password),
        "fast-open=true".to_string(),
        "udp-relay=true".to_string(),
        format!("tag={}", name),
    ];

    if let Some(plugin) = settings.get("plugin").and_then(|v| v.as_str()) {
        if plugin == "obfs" {
            if let Some(opts) = settings.get("plugin_opts").and_then(|v| v.as_str()) {
                for item in opts.split(';') {
                    if let Some((k, v)) = item.split_once('=') {
                        if k == "obfs" {
                            parts.push(format!("obfs={}", v));
                        } else if k == "obfs-host" {
                            parts.push(format!("obfs-host={}", v));
                        } else if k == "path" {
                            parts.push(format!("obfs-uri={}", v));
                        }
                    }
                }
            }
        }
    }

    format!("{}\r\n", parts.join(","))
}

fn build_quantumultx_vmess(uuid: &str, server: &ServerNodeRow, name: &str, settings: &Map<String, Value>) -> String {
    let mut parts = vec![
        format!("vmess={}:{}", server.host, server.port),
        "method=chacha20-poly1305".to_string(),
        format!("password={}", uuid),
        "fast-open=true".to_string(),
        "udp-relay=true".to_string(),
        format!("tag={}", name),
    ];

    if json_int(settings, "tls") > 0 {
        if json_string(settings, "network", "tcp") == "tcp" {
            parts.push("obfs=over-tls".to_string());
        }
        if let Some(allow_insecure) = settings.get("tls_settings").and_then(|v| v.get("allow_insecure")).and_then(|v| v.as_bool()) {
            parts.push(format!("tls-verification={}", (!allow_insecure).to_string()));
        }
        if let Some(server_name) = settings.get("tls_settings").and_then(|v| v.get("server_name")).and_then(|v| v.as_str()) {
            parts.push(format!("obfs-host={}", server_name));
        }
    }

    if json_string(settings, "network", "tcp") == "ws" {
        parts.push(if json_int(settings, "tls") > 0 { "obfs=wss" } else { "obfs=ws" }.to_string());
        if let Some(path) = settings.get("network_settings").and_then(|v| v.get("path")).and_then(|v| v.as_str()) {
            parts.push(format!("obfs-uri={}", path));
        }
        if let Some(host) = settings.get("network_settings").and_then(|v| v.get("headers")).and_then(|v| v.get("Host")).and_then(|v| v.as_str()) {
            parts.push(format!("obfs-host={}", host));
        }
    }

    format!("{}\r\n", parts.join(","))
}

fn build_quantumultx_trojan(uuid: &str, server: &ServerNodeRow, name: &str, settings: &Map<String, Value>) -> String {
    let mut parts = vec![
        format!("trojan={}:{}", server.host, server.port),
        format!("password={}", uuid),
        "over-tls=true".to_string(),
        format!("tls-verification={}", (!settings.get("allow_insecure").and_then(|v| v.as_bool()).unwrap_or(false)).to_string()),
        "fast-open=true".to_string(),
        "udp-relay=true".to_string(),
        format!("tag={}", name),
    ];
    if let Some(server_name) = settings.get("server_name").and_then(|v| v.as_str()) {
        parts.push(format!("tls-host={}", server_name));
    }
    format!("{}\r\n", parts.join(","))
}

fn build_clash_shadowsocks(
    uuid: &str,
    server: &ServerNodeRow,
    settings: &Map<String, Value>,
    app_key: &str,
) -> serde_yaml::Value {
    let cipher = json_string(settings, "cipher", "aes-128-gcm");
    let password = subscribe_shadowsocks_password(uuid, server, settings, app_key);
    let mut map = serde_yaml::Mapping::new();
    map.insert(vs("name"), vs(&server.name));
    map.insert(vs("type"), vs("ss"));
    map.insert(vs("server"), vs(&server.host));
    map.insert(vs("port"), serde_yaml::Value::Number(server.port.into()));
    map.insert(vs("cipher"), vs(&cipher));
    map.insert(vs("password"), vs(&password));
    map.insert(vs("udp"), serde_yaml::Value::Bool(true));
    serde_yaml::Value::Mapping(map)
}

fn build_clash_vmess(
    uuid: &str,
    server: &ServerNodeRow,
    settings: &Map<String, Value>,
) -> serde_yaml::Value {
    let mut map = serde_yaml::Mapping::new();
    map.insert(vs("name"), vs(&server.name));
    map.insert(vs("type"), vs("vmess"));
    map.insert(vs("server"), vs(&server.host));
    map.insert(vs("port"), serde_yaml::Value::Number(server.port.into()));
    map.insert(vs("uuid"), vs(uuid));
    map.insert(vs("alterId"), serde_yaml::Value::Number(0.into()));
    map.insert(vs("cipher"), vs("auto"));
    map.insert(vs("udp"), serde_yaml::Value::Bool(true));
    if json_int(settings, "tls") > 0 {
        map.insert(vs("tls"), serde_yaml::Value::Bool(true));
        map.insert(vs("skip-cert-verify"), serde_yaml::Value::Bool(
            settings.get("tls_settings").and_then(|v| v.get("allow_insecure")).and_then(|v| v.as_bool()).unwrap_or(false)
        ));
        if let Some(server_name) = settings.get("tls_settings").and_then(|v| v.get("server_name")).and_then(|v| v.as_str()) {
            map.insert(vs("servername"), vs(server_name));
        }
    }
    serde_yaml::Value::Mapping(map)
}

fn build_clash_trojan(
    uuid: &str,
    server: &ServerNodeRow,
    settings: &Map<String, Value>,
) -> serde_yaml::Value {
    let mut map = serde_yaml::Mapping::new();
    map.insert(vs("name"), vs(&server.name));
    map.insert(vs("type"), vs("trojan"));
    map.insert(vs("server"), vs(&server.host));
    map.insert(vs("port"), serde_yaml::Value::Number(server.port.into()));
    map.insert(vs("password"), vs(uuid));
    map.insert(vs("udp"), serde_yaml::Value::Bool(true));
    if let Some(server_name) = settings.get("server_name").and_then(|v| v.as_str()) {
        map.insert(vs("sni"), vs(server_name));
    }
    serde_yaml::Value::Mapping(map)
}

fn build_clash_socks5(uuid: &str, server: &ServerNodeRow) -> serde_yaml::Value {
    let mut map = serde_yaml::Mapping::new();
    map.insert(vs("name"), vs(&server.name));
    map.insert(vs("type"), vs("socks5"));
    map.insert(vs("server"), vs(&server.host));
    map.insert(vs("port"), serde_yaml::Value::Number(server.port.into()));
    map.insert(vs("username"), vs(uuid));
    map.insert(vs("password"), vs(uuid));
    map.insert(vs("udp"), serde_yaml::Value::Bool(true));
    serde_yaml::Value::Mapping(map)
}

fn build_clash_http(uuid: &str, server: &ServerNodeRow) -> serde_yaml::Value {
    let mut map = serde_yaml::Mapping::new();
    map.insert(vs("name"), vs(&server.name));
    map.insert(vs("type"), vs("http"));
    map.insert(vs("server"), vs(&server.host));
    map.insert(vs("port"), serde_yaml::Value::Number(server.port.into()));
    map.insert(vs("username"), vs(uuid));
    map.insert(vs("password"), vs(uuid));
    serde_yaml::Value::Mapping(map)
}

fn vs(value: &str) -> serde_yaml::Value {
    serde_yaml::Value::String(value.to_string())
}

fn build_singbox_shadowsocks(
    uuid: &str,
    server: &ServerNodeRow,
    settings: &Map<String, Value>,
    app_key: &str,
) -> Value {
    let cipher = json_string(settings, "cipher", "aes-128-gcm");
    let password = subscribe_shadowsocks_password(uuid, server, settings, app_key);
    json!({
        "tag": server.name,
        "type": "shadowsocks",
        "server": server.host,
        "server_port": server.port,
        "method": cipher,
        "password": password
    })
}

fn build_singbox_trojan(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut outbound = json!({
        "tag": server.name,
        "type": "trojan",
        "server": server.host,
        "server_port": server.port,
        "password": uuid,
        "tls": {
            "enabled": true,
            "insecure": settings.get("allow_insecure").and_then(|v| v.as_bool()).unwrap_or(false)
        }
    });
    if let Some(server_name) = settings.get("server_name").and_then(|v| v.as_str()) {
        outbound["tls"]["server_name"] = Value::String(server_name.to_string());
    }
    outbound
}

fn build_singbox_vmess(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut outbound = json!({
        "tag": server.name,
        "type": "vmess",
        "server": server.host,
        "server_port": server.port,
        "uuid": uuid,
        "security": "auto",
        "alter_id": 0
    });
    if json_int(settings, "tls") > 0 {
        outbound["tls"] = json!({
            "enabled": true,
            "insecure": settings.get("tls_settings").and_then(|v| v.get("allow_insecure")).and_then(|v| v.as_bool()).unwrap_or(false)
        });
        if let Some(server_name) = settings.get("tls_settings").and_then(|v| v.get("server_name")).and_then(|v| v.as_str()) {
            outbound["tls"]["server_name"] = Value::String(server_name.to_string());
        }
    }
    outbound
}

fn build_singbox_vless(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut outbound = json!({
        "tag": server.name,
        "type": "vless",
        "server": server.host,
        "server_port": server.port,
        "uuid": uuid,
        "packet_encoding": "xudp",
        "flow": settings.get("flow").and_then(|v| v.as_str()).unwrap_or("")
    });
    if json_int(settings, "tls") > 0 {
        outbound["tls"] = json!({
            "enabled": true,
            "insecure": settings.get("tls_settings").and_then(|v| v.get("allow_insecure")).and_then(|v| v.as_bool()).unwrap_or(false)
        });
    }
    outbound
}

fn build_singbox_hysteria(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    if json_int(settings, "version") == 2 {
        json!({
            "tag": server.name,
            "type": "hysteria2",
            "server": server.host,
            "server_port": server.port,
            "password": uuid,
            "tls": {
                "enabled": true,
                "server_name": settings.get("tls").and_then(|v| v.get("server_name")).and_then(|v| v.as_str()).unwrap_or(&server.host),
                "insecure": settings.get("tls").and_then(|v| v.get("allow_insecure")).and_then(|v| v.as_bool()).unwrap_or(false)
            }
        })
    } else {
        json!({
            "tag": server.name,
            "type": "hysteria",
            "server": server.host,
            "server_port": server.port,
            "auth_str": uuid,
            "up_mbps": settings.get("bandwidth").and_then(|v| v.get("up")).and_then(|v| v.as_i64()).unwrap_or(0),
            "down_mbps": settings.get("bandwidth").and_then(|v| v.get("down")).and_then(|v| v.as_i64()).unwrap_or(0)
        })
    }
}

fn build_singbox_tuic(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    json!({
        "tag": server.name,
        "type": "tuic",
        "server": server.host,
        "server_port": server.port,
        "uuid": uuid,
        "password": uuid,
        "congestion_control": settings.get("congestion_control").and_then(|v| v.as_str()).unwrap_or("cubic"),
        "udp_relay_mode": settings.get("udp_relay_mode").and_then(|v| v.as_str()).unwrap_or("native"),
        "tls": {
            "enabled": true,
            "server_name": settings.get("tls").and_then(|v| v.get("server_name")).and_then(|v| v.as_str()).unwrap_or(&server.host),
            "insecure": settings.get("tls").and_then(|v| v.get("allow_insecure")).and_then(|v| v.as_bool()).unwrap_or(false)
        }
    })
}

fn build_singbox_anytls(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    json!({
        "tag": server.name,
        "type": "anytls",
        "server": server.host,
        "server_port": server.port,
        "password": uuid,
        "tls": {
            "enabled": true,
            "server_name": settings.get("tls").and_then(|v| v.get("server_name")).and_then(|v| v.as_str()).unwrap_or(&server.host),
            "insecure": settings.get("tls").and_then(|v| v.get("allow_insecure")).and_then(|v| v.as_bool()).unwrap_or(false)
        }
    })
}

fn build_singbox_socks(uuid: &str, server: &ServerNodeRow, _settings: &Map<String, Value>) -> Value {
    json!({
        "tag": server.name,
        "type": "socks",
        "server": server.host,
        "server_port": server.port,
        "version": "5",
        "username": uuid,
        "password": uuid
    })
}

fn build_singbox_http(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut outbound = json!({
        "tag": server.name,
        "type": "http",
        "server": server.host,
        "server_port": server.port,
        "username": uuid,
        "password": uuid
    });
    if json_int(settings, "tls") > 0 {
        outbound["tls"] = json!({
            "enabled": true,
            "insecure": settings.get("tls_settings").and_then(|v| v.get("allow_insecure")).and_then(|v| v.as_bool()).unwrap_or(false)
        });
        if let Some(server_name) = settings.get("tls_settings").and_then(|v| v.get("server_name")).and_then(|v| v.as_str()) {
            outbound["tls"]["server_name"] = Value::String(server_name.to_string());
        }
    }
    outbound
}

fn wrap_ipv6(host: &str) -> String {
    if host.parse::<std::net::Ipv6Addr>().is_ok() {
        format!("[{}]", host)
    } else {
        host.to_string()
    }
}

fn url_safe_base64(input: &[u8]) -> String {
    BASE64_STANDARD
        .encode(input)
        .replace('+', "-")
        .replace('/', "_")
        .replace('=', "")
}

fn traffic_to_gb(value: i64) -> String {
    format!("{:.2}", (value as f64) / (1024_f64 * 1024_f64 * 1024_f64))
}

fn mask_identifier(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        return "anonymous".to_string();
    }
    if let Some((local, domain)) = value.split_once('@') {
        let local_masked = mask_text(local, 2, 1);
        let mut domain_parts = domain.split('.');
        let head = domain_parts.next().unwrap_or(domain);
        let tail = domain_parts.collect::<Vec<_>>();
        let domain_masked = mask_text(head, 1, 0);
        let suffix = if tail.is_empty() {
            String::new()
        } else {
            format!(".{}", tail.join("."))
        };
        return format!("{}@{}{}", local_masked, domain_masked, suffix);
    }
    mask_text(value, 2, 1)
}

fn mask_text(value: &str, keep_start: usize, keep_end: usize) -> String {
    let chars = value.chars().collect::<Vec<_>>();
    let len = chars.len();
    if len <= keep_start + keep_end {
        return "*".repeat(len.max(3));
    }
    let start = chars.iter().take(keep_start).collect::<String>();
    let end = if keep_end > 0 {
        chars.iter().skip(len - keep_end).collect::<String>()
    } else {
        String::new()
    };
    format!("{}{}{}", start, "*".repeat((len - keep_start - keep_end).max(3)), end)
}

async fn authenticate_subscribe_obfuscated(
    state: &AppState,
    path_key: &str,
    query: &HashMap<String, String>,
    ip_hint: Option<&str>,
) -> Result<UserRow, Response<Body>> {
    let user = load_user_by_subscribe_path(state, path_key)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| subscribe_auth_error(state, ip_hint, "token is error"))?;

    let query_key = user
        .subscribe_key
        .clone()
        .ok_or_else(|| subscribe_auth_error(state, ip_hint, "token is error"))?;
    let query_salt = user
        .subscribe_salt
        .clone()
        .ok_or_else(|| subscribe_auth_error(state, ip_hint, "token is error"))?;
    let token = user
        .token
        .clone()
        .ok_or_else(|| subscribe_auth_error(state, ip_hint, "token is error"))?;

    let query_value = query
        .get(&query_key)
        .cloned()
        .ok_or_else(|| subscribe_auth_error(state, ip_hint, "token is error"))?;
    if !query_value.ends_with(&token) {
        return Err(subscribe_auth_error(state, ip_hint, "token is error"));
    }
    if query.get(&query_salt).map(|v| v.as_str()) != Some("1") {
        return Err(subscribe_auth_error(state, ip_hint, "token is error"));
    }

    Ok(user)
}

fn subscribe_auth_error(state: &AppState, ip_hint: Option<&str>, message: &str) -> Response<Body> {
    if let Some(ip) = ip_hint {
        let key = format!("subscribe:invalid:{}", ip);
        let mut cache = state.response_cache.write();
        let attempts = cache
            .get(&key)
            .and_then(|cached| std::str::from_utf8(&cached.body).ok())
            .and_then(|body| body.parse::<i64>().ok())
            .unwrap_or(0)
            + 1;
        cache.insert(
            key,
            CachedResponse {
                status: StatusCode::OK,
                headers: vec![],
                body: bytes::Bytes::from(attempts.to_string()),
                expires_at: Instant::now() + Duration::from_secs(600),
            },
        );
        if attempts > 30 {
            return json_error(StatusCode::TOO_MANY_REQUESTS, "Too many invalid token requests");
        }
    }

    json_error(StatusCode::FORBIDDEN, message)
}

async fn insert_audit_log(
    state: &AppState,
    user_id: i64,
    node_id: u64,
    rule_id: Option<u64>,
    ip_address: &str,
    target_domain: Option<&str>,
    target_protocol: Option<&str>,
    action_taken: &str,
 ) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO audit_logs (user_id, node_id, rule_id, ip_address, target_domain, target_protocol, action_taken, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, NOW(), NOW())"
    )
    .bind(user_id)
    .bind(node_id)
    .bind(rule_id)
    .bind(ip_address)
    .bind(target_domain)
    .bind(target_protocol)
    .bind(action_taken)
    .execute(&state.db)
    .await?;

    Ok(result.last_insert_id())
}

async fn evaluate_and_insert_audit_log(
    state: &AppState,
    node_id: u64,
    user_id: i64,
    ip_address: &str,
    target_domain: Option<&str>,
    target_protocol: Option<&str>,
) -> Result<Option<u64>, sqlx::Error> {
    let rules = sqlx::query_as::<_, AuditRuleRow>(
        "SELECT id, rule_type, rule_pattern, action, is_active
         FROM audit_rules
         WHERE node_id = ? AND is_active = 1
         ORDER BY id"
    )
    .bind(node_id)
    .fetch_all(&state.db)
    .await?;

    for rule in rules {
        let matched = match rule.rule_type.as_str() {
            "domain" => target_domain.map(|target| matches_rule_pattern(&rule, target)).unwrap_or(false),
            "protocol" => target_protocol.map(|target| matches_rule_pattern(&rule, target)).unwrap_or(false),
            "ip" => matches_rule_pattern(&rule, ip_address),
            _ => false,
        };

        if !matched {
            continue;
        }

        let action_taken = match rule.action.as_str() {
            "block" => "blocked",
            "allow" => "allowed",
            "log" => "logged",
            _ => "logged",
        };

        let log_id = insert_audit_log(
            state,
            user_id,
            node_id,
            Some(rule.id),
            ip_address,
            target_domain,
            target_protocol,
            action_taken,
        )
        .await?;

        return Ok(Some(log_id));
    }

    Ok(None)
}

fn matches_rule_pattern(rule: &AuditRuleRow, target: &str) -> bool {
    if rule.is_active == 0 {
        return false;
    }

    match rule.rule_type.as_str() {
        "protocol" => rule.rule_pattern.eq_ignore_ascii_case(target),
        "ip" => ip_matches_pattern(&rule.rule_pattern, target),
        _ => wildcard_match(&rule.rule_pattern, target, true),
    }
}

fn wildcard_match(pattern: &str, target: &str, case_insensitive: bool) -> bool {
    let pattern = if case_insensitive { pattern.to_lowercase() } else { pattern.to_string() };
    let target = if case_insensitive { target.to_lowercase() } else { target.to_string() };
    wildcard_match_inner(pattern.as_bytes(), target.as_bytes())
}

fn wildcard_match_inner(pattern: &[u8], text: &[u8]) -> bool {
    let (mut p, mut t, mut star_idx, mut match_idx) = (0usize, 0usize, None, 0usize);
    while t < text.len() {
        if p < pattern.len() && (pattern[p] == text[t]) {
            p += 1;
            t += 1;
        } else if p < pattern.len() && pattern[p] == b'*' {
            star_idx = Some(p);
            match_idx = t;
            p += 1;
        } else if let Some(star) = star_idx {
            p = star + 1;
            match_idx += 1;
            t = match_idx;
        } else {
            return false;
        }
    }
    while p < pattern.len() && pattern[p] == b'*' {
        p += 1;
    }
    p == pattern.len()
}

fn ip_matches_pattern(pattern: &str, ip: &str) -> bool {
    if let Some((subnet, mask)) = pattern.split_once('/') {
        return cidr_match(subnet.trim(), mask.trim(), ip);
    }
    wildcard_match(pattern, ip, false)
}

fn cidr_match(subnet: &str, mask: &str, ip: &str) -> bool {
    let mask = match mask.parse::<u8>() {
        Ok(mask) => mask,
        Err(_) => return false,
    };

    if let (Ok(ipv4), Ok(subnetv4)) = (ip.parse::<std::net::Ipv4Addr>(), subnet.parse::<std::net::Ipv4Addr>()) {
        let ip_value = u32::from(ipv4);
        let subnet_value = u32::from(subnetv4);
        let mask_bits = if mask == 0 { 0 } else { u32::MAX << (32 - mask) };
        return (ip_value & mask_bits) == (subnet_value & mask_bits);
    }

    if let (Ok(ipv6), Ok(subnetv6)) = (ip.parse::<std::net::Ipv6Addr>(), subnet.parse::<std::net::Ipv6Addr>()) {
        let ip_bytes = ipv6.octets();
        let subnet_bytes = subnetv6.octets();
        let full_bytes = (mask / 8) as usize;
        let remaining_bits = mask % 8;
        if ip_bytes[..full_bytes] != subnet_bytes[..full_bytes] {
            return false;
        }
        if remaining_bits == 0 || full_bytes >= 16 {
            return true;
        }
        let mask_byte = u8::MAX << (8 - remaining_bits);
        (ip_bytes[full_bytes] & mask_byte) == (subnet_bytes[full_bytes] & mask_byte)
    } else {
        false
    }
}

fn nested_i64(object: &Map<String, Value>, path: &[&str]) -> Option<i64> {
    let first = path.first()?;
    let mut current = object.get(*first)?;
    for key in &path[1..] {
        current = current.get(*key)?;
    }
    current.as_i64()
}

fn normalize_type(value: &str) -> Option<String> {
    let lowered = value.trim().to_lowercase();
    let normalized = match lowered.as_str() {
        "v2ray" => "vmess",
        "hysteria2" => "hysteria",
        "" => return None,
        other => other,
    };
    Some(normalized.to_string())
}

async fn legacy_server_is_available(state: &AppState, server: &LegacySubscribeServerRow) -> bool {
    let now = Utc::now().timestamp();
    let parent_or_self = server.parent_id.unwrap_or(server.id);
    let server_type_upper = server.server_type.to_ascii_uppercase();
    let cache_key = format!("{}:{}", server_type_upper, parent_or_self);
    let instant_now = Instant::now();
    {
        let cache = state.legacy_server_availability_cache.read();
        if let Some(cached) = cache.get(&cache_key) {
            if cached.expires_at > instant_now {
                return cached.available;
            }
        }
    }
    state.legacy_server_availability_cache.write().remove(&cache_key);

    let key_check = format!("SERVER_{}_LAST_CHECK_AT_{}", server_type_upper, parent_or_self);
    let key_push = format!("SERVER_{}_LAST_PUSH_AT_{}", server_type_upper, parent_or_self);

    let last_check_at = redis_get_string(state, &key_check)
        .await
        .ok()
        .flatten()
        .and_then(|value| value.parse::<i64>().ok());
    let last_push_at = redis_get_string(state, &key_push)
        .await
        .ok()
        .flatten()
        .and_then(|value| value.parse::<i64>().ok());
    let check_ok = last_check_at.map(|ts| (now - ts) < 300).unwrap_or(false);
    let push_ok = last_push_at.map(|ts| (now - ts) < 300).unwrap_or(false);
    let available = check_ok && push_ok;

    let mut cache = state.legacy_server_availability_cache.write();
    if cache.len() >= 1024 {
        cache.retain(|_, entry| entry.expires_at > instant_now);
        if cache.len() >= 2048 {
            cache.clear();
        }
    }
    cache.insert(
        cache_key,
        CachedLegacyAvailability {
            available,
            expires_at: instant_now + Duration::from_secs(3),
        },
    );

    available
}

fn parse_coupon_limit_plan_ids(value: Option<&str>) -> Vec<i64> {
    let Some(raw) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Vec::new();
    };

    if let Ok(parsed) = serde_json::from_str::<Value>(raw) {
        if let Some(items) = parsed.as_array() {
            return items.iter().filter_map(parse_i64_value).collect();
        }
    }

    raw.split(|c| c == ',' || c == '/' || c == '|' || c == '｜')
        .filter_map(|item| item.trim().parse::<i64>().ok())
        .collect()
}

fn parse_coupon_limit_periods(value: Option<&str>) -> Vec<String> {
    let Some(raw) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Vec::new();
    };

    if let Ok(parsed) = serde_json::from_str::<Value>(raw) {
        if let Some(items) = parsed.as_array() {
            return items
                .iter()
                .filter_map(|item| item.as_str())
                .filter_map(normalize_coupon_period)
                .map(str::to_string)
                .collect();
        }
    }

    raw.split(|c| c == ',' || c == '/' || c == '|' || c == '｜')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .filter_map(normalize_coupon_period)
        .map(str::to_string)
        .collect()
}

fn parse_notice_target_plan_ids(raw: &str) -> Vec<i64> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    if let Ok(parsed) = serde_json::from_str::<Value>(trimmed) {
        if let Some(items) = parsed.as_array() {
            return items.iter().filter_map(parse_i64_value).filter(|v| *v > 0).collect();
        }
    }
    trimmed
        .split(|c| c == ',' || c == '/' || c == '|' || c == '｜')
        .filter_map(|item| item.trim().parse::<i64>().ok())
        .filter(|v| *v > 0)
        .collect()
}

fn parse_notice_tags(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    if let Ok(parsed) = serde_json::from_str::<Value>(trimmed) {
        if let Some(items) = parsed.as_array() {
            return items
                .iter()
                .filter_map(|item| item.as_str())
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect();
        }
    }
    trimmed
        .split(|c| c == ',' || c == '/' || c == '|' || c == '｜')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}

fn parse_optional_i64_field(value: Option<&Value>) -> Result<Option<i64>, Response<Body>> {
    match value {
        Some(Value::Null) | None => Ok(None),
        Some(value) => parse_i64_value(value)
            .map(Some)
            .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed")),
    }
}

fn serialize_coupon(coupon: &CouponRow) -> Value {
    let mut map = Map::new();
    map.insert("id".to_string(), Value::from(coupon.id));
    map.insert("code".to_string(), Value::from(coupon.code.clone()));
    map.insert("name".to_string(), Value::from(coupon.name.clone()));
    map.insert(
        "owner_user_id".to_string(),
        coupon.owner_user_id.map(Value::from).unwrap_or(Value::Null),
    );
    map.insert(
        "source_plan_id".to_string(),
        coupon.source_plan_id.map(Value::from).unwrap_or(Value::Null),
    );
    map.insert("type".to_string(), Value::from(coupon.type_field));
    map.insert("value".to_string(), Value::from(coupon.value));
    map.insert("show".to_string(), Value::Bool(coupon.show));
    map.insert(
        "limit_use".to_string(),
        coupon.limit_use.map(Value::from).unwrap_or(Value::Null),
    );
    map.insert(
        "limit_use_with_user".to_string(),
        coupon.limit_use_with_user.map(Value::from).unwrap_or(Value::Null),
    );

    let limit_plan_ids = parse_coupon_limit_plan_ids(coupon.limit_plan_ids.as_deref());
    map.insert(
        "limit_plan_ids".to_string(),
        if limit_plan_ids.is_empty() {
            Value::Null
        } else {
            Value::Array(limit_plan_ids.into_iter().map(|id| Value::String(id.to_string())).collect())
        },
    );

    let limit_periods = parse_coupon_limit_periods(coupon.limit_period.as_deref());
    map.insert(
        "limit_period".to_string(),
        if limit_periods.is_empty() {
            Value::Null
        } else {
            Value::Array(
                limit_periods
                    .into_iter()
                    .map(|period| Value::String(convert_period_to_legacy_field(&period).to_string()))
                    .collect(),
            )
        },
    );

    map.insert("started_at".to_string(), Value::from(coupon.started_at));
    map.insert("ended_at".to_string(), Value::from(coupon.ended_at));
    map.insert("created_at".to_string(), Value::from(coupon.created_at));
    map.insert("updated_at".to_string(), Value::from(coupon.updated_at));
    Value::Object(map)
}

fn env_bool(name: &str, default: bool) -> bool {
    env::var(name)
        .ok()
        .map(|value| matches!(value.trim(), "1" | "true" | "TRUE" | "on" | "yes"))
        .unwrap_or(default)
}

fn first_non_empty(values: &[String]) -> String {
    values
        .iter()
        .find_map(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .unwrap_or_default()
}

fn slug_for_prefix(value: &str) -> String {
    let mut out = String::new();
    let mut last_was_sep = false;
    for ch in value.chars() {
        let mapped = if ch.is_ascii_alphanumeric() {
            ch.to_ascii_lowercase()
        } else {
            '_'
        };
        if mapped == '_' {
            if !last_was_sep && !out.is_empty() {
                out.push('_');
            }
            last_was_sep = true;
        } else {
            out.push(mapped);
            last_was_sep = false;
        }
    }
    out.trim_matches('_').to_string().chars().collect::<String>()
}

fn plan_owner_display_name(plan: &PlanRow) -> String {
    let linux_do_name = plan.owner_linux_do_name.as_deref().unwrap_or("").trim();
    if !linux_do_name.is_empty() {
        return linux_do_name.to_string();
    }
    let linux_do_username = plan.owner_linux_do_username.as_deref().unwrap_or("").trim();
    if !linux_do_username.is_empty() {
        return linux_do_username.to_string();
    }
    let email = plan.owner_email.as_deref().unwrap_or("").trim();
    if !email.is_empty() {
        return email.to_string();
    }
    if let Some(owner_user_id) = plan.owner_user_id {
        return format!("用户#{}", owner_user_id);
    }
    "未知".to_string()
}

fn serialize_plan_owner(plan: &PlanRow) -> Value {
    match plan.owner_user_id {
        Some(owner_user_id) => json!({
            "id": owner_user_id,
            "email": plan.owner_email.clone().unwrap_or_default(),
            "linux_do_username": plan.owner_linux_do_username.clone().unwrap_or_default(),
            "linux_do_name": plan.owner_linux_do_name.clone().unwrap_or_default(),
            "display_name": plan_owner_display_name(plan),
        }),
        None => Value::Null,
    }
}

fn plan_has_trial_quota(value: &Value) -> bool {
    value
        .as_object()
        .map(|object| object.values().any(|item| item.as_f64().map(|value| value > 0.0).unwrap_or(false)))
        .unwrap_or(false)
}

fn price_to_legacy_number(value: Option<&Value>) -> Value {
    value
        .and_then(|item| item.as_f64())
        .map(|price| Value::from(price * 100.0))
        .unwrap_or(Value::Null)
}

fn format_capacity_limit(capacity_limit: Option<u64>) -> Value {
    match capacity_limit {
        None => Value::Null,
        Some(limit) if limit <= 0 => Value::String("Sold out".to_string()),
        Some(limit) => Value::from(limit),
    }
}

fn format_plan_content(
    raw_content: &str,
    transfer_enable: u64,
    speed_limit: Option<u64>,
    device_limit: Option<u64>,
    reset_traffic_method: i64,
    system_reset_method: i64,
) -> String {
    let resolved_reset_method = if reset_traffic_method == 0 {
        system_reset_method
    } else {
        reset_traffic_method
    };
    raw_content
        .replace("{{transfer}}", &transfer_enable.to_string())
        .replace("{{speed}}", &speed_limit.map(|value| value.to_string()).unwrap_or_else(|| "No Limit".to_string()))
        .replace("{{devices}}", &device_limit.map(|value| value.to_string()).unwrap_or_else(|| "No Limit".to_string()))
        .replace("{{reset_method}}", reset_method_text(resolved_reset_method))
}

fn reset_method_text(method: i64) -> &'static str {
    match method {
        0 => "First Day of Month",
        1 => "Monthly",
        2 => "Never",
        3 => "First Day of Year",
        4 => "Yearly",
        _ => "Monthly",
    }
}

fn normalized_protocol_settings(protocol: &str, raw_settings: Map<String, Value>) -> Map<String, Value> {
    let defaults = protocol_setting_template(protocol);
    merge_maps(defaults, raw_settings)
}

fn protocol_setting_template(protocol: &str) -> Map<String, Value> {
    let value = match protocol {
        "trojan" => json!({
            "allow_insecure": false,
            "server_name": null,
            "network": null,
            "network_settings": null
        }),
        "vmess" => json!({
            "tls": 0,
            "network": null,
            "rules": null,
            "network_settings": null,
            "tls_settings": null
        }),
        "vless" => json!({
            "tls": 0,
            "tls_settings": null,
            "flow": null,
            "network": null,
            "network_settings": null,
            "reality_settings": {
                "allow_insecure": false,
                "server_port": null,
                "server_name": null,
                "public_key": null,
                "private_key": null,
                "short_id": null
            }
        }),
        "shadowsocks" => json!({
            "cipher": null,
            "obfs": null,
            "obfs_settings": null,
            "plugin": null,
            "plugin_opts": null
        }),
        "hysteria" => json!({
            "version": 1,
            "bandwidth": { "up": null, "down": null },
            "obfs": { "open": false, "type": "salamander", "password": null },
            "tls": { "server_name": null, "allow_insecure": false },
            "hop_interval": null
        }),
        "tuic" => json!({
            "version": 5,
            "congestion_control": "cubic",
            "alpn": ["h3"],
            "udp_relay_mode": "native",
            "zero_rtt_handshake": false,
            "heartbeat": "10s",
            "tls": { "server_name": null, "allow_insecure": false }
        }),
        "anytls" => json!({
            "padding_scheme": [
                "stop=8",
                "0=30-30",
                "1=100-400",
                "2=400-500,c,500-1000,c,500-1000,c,500-1000,c,500-1000",
                "3=9-9,500-1000",
                "4=500-1000",
                "5=500-1000",
                "6=500-1000",
                "7=500-1000"
            ],
            "tls": { "server_name": null, "allow_insecure": false }
        }),
        "socks" => json!({
            "tls": 0,
            "tls_settings": { "allow_insecure": false, "server_name": null },
            "udp_over_tcp": false
        }),
        "naive" => json!({
            "tls": 0,
            "tls_settings": { "allow_insecure": false, "server_name": null }
        }),
        "http" => json!({
            "tls": 0,
            "tls_settings": { "allow_insecure": false, "server_name": null },
            "path": null,
            "headers": null
        }),
        "mieru" => json!({
            "protocol": 0,
            "transport": "tcp",
            "multiplexing": "MULTIPLEXING_LOW"
        }),
        _ => json!({}),
    };
    parse_json_object(Some(&value.to_string())).unwrap_or_default()
}

fn merge_maps(base: Map<String, Value>, overlay: Map<String, Value>) -> Map<String, Value> {
    let mut result = base;
    for (key, value) in overlay {
        match (result.get(&key).cloned(), value) {
            (Some(Value::Object(existing)), Value::Object(overlay_obj)) => {
                result.insert(key, Value::Object(merge_maps(existing, overlay_obj)));
            }
            (_, overlay_value) => {
                result.insert(key, overlay_value);
            }
        }
    }
    result
}

fn resolve_panel_server_name(settings: &Map<String, Value>, fallback_host: &str) -> String {
    let candidates = [
        settings.get("server_name"),
        settings.get("tls").and_then(|v| v.get("server_name")),
        settings.get("tls_settings").and_then(|v| v.get("server_name")),
        settings.get("reality_settings").and_then(|v| v.get("server_name")),
    ];

    for candidate in candidates {
        if let Some(value) = candidate.and_then(|v| v.as_str()) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }

    fallback_host.to_string()
}

fn json_int(settings: &Map<String, Value>, key: &str) -> i64 {
    settings.get(key).and_then(|v| v.as_i64()).unwrap_or(0)
}

fn json_bool(settings: &Map<String, Value>, key: &str) -> bool {
    settings.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
}

fn json_string(settings: &Map<String, Value>, key: &str, default: &str) -> String {
    settings.get(key).and_then(|v| v.as_str()).unwrap_or(default).to_string()
}

fn json_nested_int(settings: &Map<String, Value>, path: &[&str]) -> i64 {
    get_nested_value(settings, path).and_then(|v| v.as_i64()).unwrap_or(0)
}

fn json_nested_bool(settings: &Map<String, Value>, path: &[&str]) -> bool {
    get_nested_value(settings, path).and_then(|v| v.as_bool()).unwrap_or(false)
}

fn json_nested_string(settings: &Map<String, Value>, path: &[&str], default: &str) -> String {
    get_nested_value(settings, path)
        .and_then(|v| v.as_str())
        .unwrap_or(default)
        .to_string()
}

fn get_nested_value<'a>(settings: &'a Map<String, Value>, path: &[&str]) -> Option<&'a Value> {
    let mut current: Option<&Value> = None;
    for (index, key) in path.iter().enumerate() {
        if index == 0 {
            current = settings.get(*key);
        } else {
            current = current?.get(*key);
        }
    }
    current
}

fn normalize_object_value(value: Option<&Value>, default_empty_object: bool) -> Value {
    match value {
        Some(Value::Object(map)) => Value::Object(map.clone()),
        Some(Value::Array(array)) => {
            if array.is_empty() {
                if default_empty_object {
                    json!({})
                } else {
                    Value::Array(array.clone())
                }
            } else if default_empty_object {
                json!({})
            } else {
                Value::Array(array.clone())
            }
        }
        Some(Value::String(raw)) => {
            if let Ok(parsed) = serde_json::from_str::<Value>(raw) {
                normalize_object_value(Some(&parsed), default_empty_object)
            } else if default_empty_object {
                json!({})
            } else {
                Value::Null
            }
        }
        _ => {
            if default_empty_object {
                json!({})
            } else {
                Value::Null
            }
        }
    }
}

fn merge_json_object(base: impl Into<Value>, overlay: Value) -> Value {
    let mut result = match base.into() {
        Value::Object(object) => object,
        _ => Map::new(),
    };
    if let Value::Object(object) = overlay {
        for (key, value) in object {
            result.insert(key, value);
        }
    }
    Value::Object(result)
}

fn parse_json_object(raw: Option<&str>) -> Option<Map<String, Value>> {
    let raw = raw?.trim();
    if raw.is_empty() {
        return Some(Map::new());
    }
    serde_json::from_str::<Value>(raw)
        .ok()
        .and_then(|value| value.as_object().cloned())
}

fn get_server_key(app_key: &str, created_at: Option<chrono::DateTime<Utc>>, length: usize) -> String {
    let key_bytes = app_key_bytes(app_key);
    let timestamp = created_at
        .map(|dt| dt.timestamp())
        .unwrap_or_else(|| Utc::now().timestamp())
        .to_string();
    let mut hmac = hmac_sha256::HMAC::mac(timestamp.as_bytes(), &key_bytes);
    hmac.truncate(length);
    BASE64_STANDARD.encode(hmac)
}

fn app_key_bytes(app_key: &str) -> Vec<u8> {
    if let Some(encoded) = app_key.strip_prefix("base64:") {
        BASE64_STANDARD.decode(encoded).unwrap_or_else(|_| encoded.as_bytes().to_vec())
    } else {
        app_key.as_bytes().to_vec()
    }
}

fn effective_uuid(base_uuid: &str, version: i64, rotate_enabled: bool) -> String {
    if base_uuid.is_empty() || !rotate_enabled || version <= 0 {
        return base_uuid.to_string();
    }

    let mut sha = Sha1::new();
    sha.update(format!("{}|{}", base_uuid, version).as_bytes());
    let hash = sha.finalize();
    let mut bytes = hash[..16].to_vec();
    bytes[6] = (bytes[6] & 0x0f) | 0x50;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    )
}

mod hmac_sha256 {
    use sha2::{Digest, Sha256};

    pub struct HMAC;

    impl HMAC {
        pub fn mac(data: &[u8], key: &[u8]) -> Vec<u8> {
            const BLOCK_SIZE: usize = 64;
            let mut key_block = vec![0u8; BLOCK_SIZE];

            if key.len() > BLOCK_SIZE {
                let digest = Sha256::digest(key);
                key_block[..digest.len()].copy_from_slice(&digest);
            } else {
                key_block[..key.len()].copy_from_slice(key);
            }

            let mut o_key_pad = vec![0x5c; BLOCK_SIZE];
            let mut i_key_pad = vec![0x36; BLOCK_SIZE];
            for i in 0..BLOCK_SIZE {
                o_key_pad[i] ^= key_block[i];
                i_key_pad[i] ^= key_block[i];
            }

            let mut inner = Sha256::new();
            inner.update(&i_key_pad);
            inner.update(data);
            let inner_hash = inner.finalize();

            let mut outer = Sha256::new();
            outer.update(&o_key_pad);
            outer.update(inner_hash);
            outer.finalize().to_vec()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_ipv6_hosts() {
        assert_eq!(wrap_ipv6("2001:db8::1"), "[2001:db8::1]");
        assert_eq!(wrap_ipv6("1.2.3.4"), "1.2.3.4");
    }

    #[test]
    fn wildcard_matching_works() {
        assert!(wildcard_match("*.example.com", "api.example.com", true));
        assert!(!wildcard_match("*.example.com", "example.net", true));
    }

    #[test]
    fn cidr_matching_works() {
        assert!(cidr_match("10.0.0.0", "24", "10.0.0.8"));
        assert!(!cidr_match("10.0.0.0", "24", "10.0.1.8"));
    }

    #[test]
    fn rust_subscribe_mode_detects_general_and_shadowrocket() {
        let mut query = HashMap::new();
        let headers = HeaderMap::new();

        query.insert("flag".to_string(), "general".to_string());
        assert!(matches!(rust_subscribe_mode(&query, &headers), Some(RustSubscribeMode::General)));

        query.insert("flag".to_string(), "shadowsocks".to_string());
        assert!(matches!(rust_subscribe_mode(&query, &headers), Some(RustSubscribeMode::Shadowsocks)));

        query.insert("flag".to_string(), "shadowrocket".to_string());
        assert!(matches!(rust_subscribe_mode(&query, &headers), Some(RustSubscribeMode::Shadowrocket)));

        query.insert("flag".to_string(), "quantumult-x".to_string());
        assert!(matches!(rust_subscribe_mode(&query, &headers), Some(RustSubscribeMode::QuantumultX)));

        query.insert("flag".to_string(), "loon".to_string());
        assert!(matches!(rust_subscribe_mode(&query, &headers), Some(RustSubscribeMode::Loon)));

        query.insert("flag".to_string(), "surge".to_string());
        assert!(matches!(rust_subscribe_mode(&query, &headers), Some(RustSubscribeMode::Surge)));

        query.insert("flag".to_string(), "surfboard".to_string());
        assert!(matches!(rust_subscribe_mode(&query, &headers), Some(RustSubscribeMode::Surfboard)));

        query.insert("flag".to_string(), "stash".to_string());
        assert!(matches!(rust_subscribe_mode(&query, &headers), Some(RustSubscribeMode::Stash)));

        query.insert("flag".to_string(), "clashmeta".to_string());
        assert!(matches!(rust_subscribe_mode(&query, &headers), Some(RustSubscribeMode::ClashMeta)));

        query.insert("flag".to_string(), "unknown-client".to_string());
        assert!(matches!(rust_subscribe_mode(&query, &headers), Some(RustSubscribeMode::General)));
    }

    #[test]
    fn general_socks_format_contains_expected_scheme() {
        let out = build_general_socks("abc", &ServerNodeRow {
            id: 1,
            user_id: 1,
            name: "node-1".to_string(),
            host: "1.2.3.4".to_string(),
            port: 1080,
            service_port: None,
            protocol: "socks".to_string(),
            settings: None,
            access_control: None,
            device_limit: 0,
            connection_limit: 0,
            speed_limit_down: 0,
            created_at: Some(chrono::DateTime::UNIX_EPOCH),
        }, "node-1");

        assert!(out.starts_with("socks://"));
        assert!(out.contains("#node-1"));
    }

    #[test]
    fn shadowrocket_trojan_format_contains_expected_fields() {
        let settings = parse_json_object(Some(r#"{"allow_insecure":false,"server_name":"edge.example.com","network":"tcp","network_settings":{}}"#)).unwrap();
        let out = build_shadowrocket_trojan(
            "secret",
            &ServerNodeRow {
                id: 1,
                user_id: 1,
                name: "edge".to_string(),
                host: "1.2.3.4".to_string(),
                port: 443,
                service_port: None,
                protocol: "trojan".to_string(),
                settings: None,
                access_control: None,
                device_limit: 0,
                connection_limit: 0,
                speed_limit_down: 0,
                created_at: Some(chrono::DateTime::UNIX_EPOCH),
            },
            "edge",
            &settings,
        );

        assert!(out.starts_with("trojan://secret@1.2.3.4:443?"));
        assert!(out.contains("peer=edge.example.com"));
    }

    #[test]
    fn quantumultx_vmess_format_contains_expected_prefix() {
        let settings = parse_json_object(Some(r#"{"tls":1,"network":"ws","network_settings":{"path":"/ws","headers":{"Host":"cdn.example.com"}},"tls_settings":{"allow_insecure":false,"server_name":"cdn.example.com"}}"#)).unwrap();
        let out = build_quantumultx_vmess(
            "uuid-1",
            &ServerNodeRow {
                id: 1,
                user_id: 1,
                name: "edge".to_string(),
                host: "1.2.3.4".to_string(),
                port: 443,
                service_port: None,
                protocol: "vmess".to_string(),
                settings: None,
                access_control: None,
                device_limit: 0,
                connection_limit: 0,
                speed_limit_down: 0,
                created_at: Some(chrono::DateTime::UNIX_EPOCH),
            },
            "edge",
            &settings,
        );

        assert!(out.starts_with("vmess=1.2.3.4:443,"));
        assert!(out.contains("obfs=wss"));
        assert!(out.contains("obfs-uri=/ws"));
    }
}
