use crate::{
    admin_v1, admin_v2, authenticate_route_account, bootstrap_support, client_v1, exposure_control,
    fallback_support, first_non_empty, get_setting_string, guest_v1, healthz, json_error,
    passport_v1, secure_admin_path_matches, static_files, subscribe_entry, tcping_agent_v1,
    uniproxy_alive, uniproxy_alivelist, uniproxy_audit, uniproxy_config, uniproxy_push,
    uniproxy_status, uniproxy_user, user_v1, web_pages, AppState, RequiredAccountRole,
};
use axum::{
    extract::{Request, State},
    middleware::{self, Next},
    response::Response,
    routing::{any, get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::compression::CompressionLayer;

pub(crate) fn build_router(state: AppState) -> Router {
    let state = Arc::new(state);
    Router::new()
        .merge(web_routes(state.clone()))
        .merge(guest_routes(state.clone()))
        .merge(passport_routes())
        .merge(monitor_routes(state.clone()))
        .merge(payment_notify_routes())
        .merge(user_routes(state.clone()))
        .merge(admin_routes(state.clone()))
        .merge(client_routes())
        .merge(server_routes())
        .merge(agent_routes())
        .fallback(fallback_support::rust_fallback)
        .with_state(state)
        .layer(middleware::from_fn(exposure_control::add_privacy_headers))
        .layer(CompressionLayer::new())
}

fn web_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let browser_routes = Router::new()
        .route("/", get(web_pages::public_dashboard_page))
        .route("/app", get(web_pages::app_page))
        .route("/app/", get(web_pages::app_page))
        .route("/login/linux-do", get(web_pages::login_linux_do_page))
        .route("/assets/{*path}", get(static_files::assets_file))
        .route("/theme/{*path}", get(static_files::theme_file))
        .route(
            "/{admin_path}/assets/admin-console.css",
            get(web_pages::admin_console_stylesheet),
        )
        .route(
            "/{admin_path}/assets/admin-console.js",
            get(web_pages::admin_console_script),
        )
        .route("/{admin_path}/command-center", get(web_pages::admin_command_center_page))
        .route("/{admin_path}/leaderboards", get(web_pages::admin_leaderboards_redirect))
        .route("/{admin_path}", get(web_pages::admin_page))
        .route_layer(middleware::from_fn_with_state(
            state,
            exposure_control::require_web_access,
        ));
    let subscribe_routes = Router::new()
        .route("/{subscribe_key}/{token_or_path}", get(subscribe_entry))
        .route_layer(middleware::from_fn(exposure_control::mark_private_response));

    Router::new()
        .route("/healthz", get(healthz))
        .route("/robots.txt", get(web_pages::robots_txt))
        .route("/v2bx-install.sh", get(static_files::installer_file))
        .route("/tcping-agent-install.sh", get(static_files::installer_file))
        .route("/tcping-agent-src/go.mod", get(static_files::installer_file))
        .route("/tcping-agent-src/main.go", get(static_files::installer_file))
        .route("/bootstrap/status", get(bootstrap_support::bootstrap_status))
        .route("/bootstrap/minimal", post(bootstrap_support::bootstrap_minimal))
        .route("/bootstrap/full", post(bootstrap_support::bootstrap_full))
        .merge(browser_routes)
        .merge(subscribe_routes)
}

fn guest_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let public_content_routes = Router::new()
        .route("/api/v1/guest/plan/fetch", get(guest_v1::public::plan_fetch))
        .route("/api/v1/guest/public/overview", get(guest_v1::public::overview))
        .route("/api/v1/guest/public/leaderboards", get(guest_v1::public::leaderboards))
        .route("/api/v1/guest/public/geo", get(guest_v1::public::geo))
        .route_layer(middleware::from_fn_with_state(
            state,
            exposure_control::require_web_access,
        ));

    Router::new()
        .route("/api/v1/guest/comm/config", get(guest_v1::public::comm_config))
        .route("/api/v1/guest/telegram/webhook", post(guest_v1::telegram::webhook))
        .merge(public_content_routes)
}

fn passport_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/passport/auth/pow-challenge", get(passport_v1::auth::pow_challenge))
        .route("/api/v2/passport/auth/pow-challenge", get(passport_v1::auth::pow_challenge))
        .route("/api/v1/passport/auth/login", post(passport_v1::auth::login))
        .route("/api/v2/passport/auth/login", post(passport_v1::auth::login))
        .route("/api/v1/passport/auth/getQuickLoginUrl", post(passport_v1::auth::get_quick_login_url))
        .route("/api/v2/passport/auth/getQuickLoginUrl", post(passport_v1::auth::get_quick_login_url))
        .route("/api/v1/passport/auth/token2Login", get(passport_v1::auth::token2_login))
        .route("/api/v2/passport/auth/token2Login", get(passport_v1::auth::token2_login))
        .route("/api/v1/passport/auth/loginWithMailLink", post(passport_v1::auth::login_with_mail_link))
        .route("/api/v2/passport/auth/loginWithMailLink", post(passport_v1::auth::login_with_mail_link))
        .route("/api/v1/passport/auth/forget", post(passport_v1::auth::forget))
        .route("/api/v2/passport/auth/forget", post(passport_v1::auth::forget))
        .route("/api/v1/passport/auth/register", post(passport_v1::auth::register))
        .route("/api/v2/passport/auth/register", post(passport_v1::auth::register))
        .route("/api/v1/passport/oauth2/linux-do/redirect", get(crate::oauth_v1::linux_do::redirect))
        .route("/api/v1/passport/oauth2/linux-do/callback", get(crate::oauth_v1::linux_do::callback))
        .route("/api/v1/passport/oauth2/refresh", post(crate::oauth_v1::linux_do::refresh))
        .route("/api/v1/passport/oauth2/sync", post(crate::oauth_v1::linux_do::sync))
        .route("/api/v1/passport/comm/sendEmailVerify", post(passport_v1::comm::send_email_verify))
        .route("/api/v2/passport/comm/sendEmailVerify", post(passport_v1::comm::send_email_verify))
        .route("/api/v1/passport/comm/pv", post(passport_v1::comm::pv))
        .route("/api/v2/passport/comm/pv", post(passport_v1::comm::pv))
}

fn monitor_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/monitor/api/stats", get(admin_v2::system::monitor_stats))
        .route("/monitor/api/workload", get(admin_v2::system::monitor_workload))
        .route("/monitor/api/masters", get(admin_v2::system::monitor_masters))
        .route("/monitor/api/metrics/jobs", get(admin_v2::system::monitor_job_metrics))
        .route("/monitor/api/metrics/jobs/{id}", get(admin_v2::system::monitor_job_metric_detail))
        .route("/monitor/api/metrics/queues", get(admin_v2::system::monitor_queue_metrics))
        .route("/monitor/api/metrics/queues/{id}", get(admin_v2::system::monitor_queue_metric_detail))
        .route("/monitor/api/jobs/pending", get(admin_v2::system::monitor_jobs_pending))
        .route("/monitor/api/jobs/completed", get(admin_v2::system::monitor_jobs_completed))
        .route("/monitor/api/jobs/silenced", get(admin_v2::system::monitor_jobs_silenced))
        .route("/monitor/api/jobs/failed", get(admin_v2::system::monitor_jobs_failed))
        .route("/monitor/api/jobs/failed/{id}", get(admin_v2::system::monitor_failed_job_detail))
        .route("/monitor/api/jobs/{id}", get(admin_v2::system::monitor_job_detail))
        .route("/monitor/api/batches", get(admin_v2::system::monitor_batches))
        .route("/monitor/api/batches/{id}", get(admin_v2::system::monitor_batch_detail))
        .route("/monitor/api/batches/retry/{id}", post(admin_v2::system::monitor_batch_retry))
        .route("/monitor/api/monitoring", get(admin_v2::system::monitor_monitoring))
        .route("/monitor/api/monitoring/{tag}", get(admin_v2::system::monitor_monitoring_tag))
        .route("/monitor/api/jobs/retry/{id}", post(admin_v2::system::monitor_job_retry))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_super_admin_api_access,
        ))
        .route_layer(middleware::from_fn_with_state(
            state,
            require_fixed_admin_path,
        ))
}

fn payment_notify_routes() -> Router<Arc<AppState>> {
    Router::new().route("/api/v1/guest/payment/notify/{method}/{uuid}", any(guest_v1::payment::notify))
}

fn user_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/user/info", get(user_v1::read::info))
        .route("/api/v1/user/me", get(user_v1::read::me))
        .route("/api/v1/user/comm/config", get(user_v1::read::comm_config))
        .route("/api/v1/user/telegram/getBotInfo", get(user_v1::read::telegram_get_bot_info))
        .route("/api/v1/user/knowledge/fetch", get(user_v1::read::knowledge_fetch))
        .route("/api/v1/user/knowledge/getCategory", get(user_v1::read::knowledge_get_category))
        .route("/api/v1/user/comm/getStripePublicKey", post(user_v1::read::comm_get_stripe_public_key))
        .route("/api/v1/user/stat/getTrafficLog", get(user_v1::read::stat_get_traffic_log))
        .route("/api/v1/user/getSubscribe", get(user_v1::read::get_subscribe))
        .route("/api/v1/user/checkLogin", get(user_v1::read::check_login))
        .route("/api/v1/user/getStat", get(user_v1::read::get_stat))
        .route("/api/v1/user/plan/quota", get(user_v1::read::plan_quota))
        .route("/api/v1/user/plan/fetch", get(user_v1::read::plan_fetch))
        .route("/api/v1/user/node-plans", get(user_v1::node_plans::index).post(user_v1::node_plans::store))
        .route("/api/v1/user/node-plans/{id}", put(user_v1::node_plans::update).delete(user_v1::node_plans::destroy))
        .route("/api/v1/user/getActiveSession", get(user_v1::security::get_active_session))
        .route("/api/v1/user/getQuickLoginUrl", post(user_v1::security::get_quick_login_url))
        .route("/api/v1/user/removeActiveSession", post(user_v1::security::remove_active_session))
        .route("/api/v1/user/resetSecurity", get(user_v1::security::reset_security))
        .route("/api/v1/user/changePassword", post(user_v1::security::change_password))
        .route("/api/v1/user/transfer", post(user_v1::account::transfer))
        .route("/api/v1/user/coupon/check", post(user_v1::account::coupon_check))
        .route("/api/v1/user/payment-profiles/epay", get(user_v1::account::payment_profile_show_epay).put(user_v1::account::payment_profile_upsert_epay))
        .route("/api/v1/user/api-key", get(user_v1::account::api_key_show))
        .route("/api/v1/user/api-key/generate", post(user_v1::account::api_key_generate))
        .route("/api/v1/user/api-key/reset", post(user_v1::account::api_key_reset))
        .route("/api/v1/user/api-key/validate", post(user_v1::account::api_key_validate))
        .route("/api/v1/user/limits", get(admin_v1::users_limits::self_limits_show))
        .route("/api/v1/user/notice/fetch", get(user_v1::notices::fetch))
        .route("/api/v1/user/notice", post(user_v1::notices::save))
        .route("/api/v1/user/notice/{id}/toggle", post(user_v1::notices::toggle))
        .route("/api/v1/user/notice/{id}", any(user_v1::notices::drop_notice))
        .route("/api/v1/user/ticket/fetch", get(user_v1::tickets::fetch))
        .route("/api/v1/user/ticket/save", post(user_v1::tickets::save))
        .route("/api/v1/user/ticket/reply", post(user_v1::tickets::reply))
        .route("/api/v1/user/ticket/close", post(user_v1::tickets::close))
        .route("/api/v1/user/ticket/withdraw", post(user_v1::tickets::withdraw))
        .route("/api/v1/user/node-admin/server-nodes/{id}/users-traffic", get(user_v1::node_admin_nodes::users_traffic))
        .route("/api/v1/user/node-admin/server-nodes/{id}/blacklist", post(user_v1::node_admin_nodes::blacklist_user))
        .route("/api/v1/user/node-admin/server-nodes/{id}/unblacklist", post(user_v1::node_admin_nodes::unblacklist_user))
        .route("/api/v1/user/audit-logs", get(user_v1::audit::logs))
        .route("/api/v1/user/access/stats", get(user_v1::access::stats))
        .route("/api/v1/user/accessible-nodes", get(user_v1::access::accessible_nodes))
        .route("/api/v1/user/tcping/agents", get(user_v1::tcping::agents).post(user_v1::tcping::create_agent))
        .route("/api/v1/user/tcping/agents/{id}/rotate-token", post(user_v1::tcping::rotate_agent_token))
        .route("/api/v1/user/tcping/agents/{id}/toggle", post(user_v1::tcping::toggle_agent))
        .route("/api/v1/user/tcping/agents/{id}/install-command", get(user_v1::tcping::agent_install_command))
        .route("/api/v1/user/server-nodes/protocols", get(user_v1::server_nodes::protocols))
        .route("/api/v1/user/server/fetch", get(user_v1::server_nodes::fetch))
        .route("/api/v1/user/server-nodes", get(user_v1::server_nodes::index).post(user_v1::server_nodes::store))
        .route("/api/v1/user/server-nodes/{id}", get(user_v1::server_nodes::show).put(user_v1::server_nodes::update).delete(user_v1::server_nodes::destroy))
        .route("/api/v1/user/server-nodes/{id}/access/stats", get(user_v1::access::node_stats))
        .route("/api/v1/user/server-nodes/{id}/traffic", get(user_v1::traffic::node_stats))
        .route("/api/v1/user/node-traffic", get(user_v1::traffic::node_traffic))
        .route("/api/v1/user/traffic-usage-logs", get(user_v1::traffic::usage_logs))
        .route("/api/v1/user/server-nodes/{id}/status", get(user_v1::server_nodes::status))
        .route("/api/v1/user/server-nodes/{id}/deploy", post(user_v1::server_nodes::deploy))
        .route("/api/v1/user/server-nodes/{id}/deploy-command", post(user_v1::server_nodes::deploy_command))
        .route("/api/v1/user/server-nodes/{id}/deploy-command/rotate-token", post(user_v1::server_nodes::rotate_deploy_token))
        .route("/api/v1/user/server-nodes/{id}/access", post(user_v1::access::configure_node))
        .route("/api/v1/user/server-nodes/{id}/tcping", get(user_v1::tcping::node_overview))
        .route("/api/v1/user/server-nodes/{id}/share/user", post(user_v1::access::share_with_user))
        .route("/api/v1/user/server-nodes/{id}/share/group", post(user_v1::access::share_with_group))
        .route("/api/v1/user/server-nodes/{id}/share/revoke", post(user_v1::access::share_revoke))
        .route("/api/v1/user/server-nodes/{id}/audit-logs", get(user_v1::audit::node_logs))
        .route("/api/v1/user/server-nodes/{id}/audit-rules", get(user_v1::audit::node_rules).post(user_v1::audit::store_node_rule))
        .route("/api/v1/user/server-nodes/{id}/audit-rules/{ruleId}", any(user_v1::audit::update_or_destroy_node_rule))
        .route("/api/v1/user/node-admin/tickets", get(user_v1::node_admin_tickets::index))
        .route("/api/v1/user/node-admin/ticket/detail", get(user_v1::node_admin_tickets::detail))
        .route("/api/v1/user/node-admin/ticket/reply", post(user_v1::node_admin_tickets::reply))
        .route("/api/v1/user/node-admin/ticket/close", post(user_v1::node_admin_tickets::close))
        .route("/api/v1/user/refunds", get(user_v1::refunds::index).post(user_v1::refunds::create))
        .route("/api/v1/user/refunds/{id}", get(user_v1::refunds::detail))
        .route("/api/v1/user/refunds/{id}/evidence", post(user_v1::refunds::evidence))
        .route("/api/v1/user/node-admin/refunds", get(user_v1::node_admin_refunds::index))
        .route("/api/v1/user/node-admin/refunds/{id}", get(user_v1::node_admin_refunds::detail))
        .route("/api/v1/user/node-admin/refunds/{id}/approve", post(user_v1::node_admin_refunds::approve))
        .route("/api/v1/user/node-admin/refunds/{id}/deny", post(user_v1::node_admin_refunds::deny))
        .route("/api/v1/user/node-admin/refunds/{id}/dispute", post(user_v1::node_admin_refunds::dispute))
        .route("/api/v1/user/node-admin/refunds/{id}/evidence", post(user_v1::node_admin_refunds::evidence))
        .route("/api/v1/user/refund-votes", get(user_v1::refund_votes::index))
        .route("/api/v1/user/refund-votes/{id}", get(user_v1::refund_votes::detail).post(user_v1::refund_votes::cast))
        .route("/api/v1/user/refund-votes/{id}/evidence", post(user_v1::refund_votes::evidence))
        .route("/api/v1/user/order/fetch", get(user_v1::orders::fetch))
        .route("/api/v1/user/order/check", get(user_v1::orders::check))
        .route("/api/v1/user/order/detail", get(user_v1::orders::detail))
        .route("/api/v1/user/order/cancel", post(user_v1::orders::cancel))
        .route("/api/v1/user/order/getPaymentMethod", get(user_v1::orders::get_payment_method))
        .route("/api/v1/user/order/save", post(user_v1::orders::save))
        .route("/api/v1/user/order/checkout", post(user_v1::orders::checkout))
        .route("/api/v1/user/update", post(user_v1::account::update))
        .route("/api/v2/user/info", get(user_v1::read::info))
        .route("/api/v2/user/resetSecurity", get(user_v1::security::reset_security))
        .route("/api/v1/user/invite/save", post(user_v1::invites::save))
        .route("/api/v1/user/invite/fetch", get(user_v1::invites::fetch))
        .route("/api/v1/user/invite/details", get(user_v1::invites::details))
        .route("/api/v1/user/gift-card/check", post(user_v1::gift_cards::check))
        .route("/api/v1/user/gift-card/redeem", post(user_v1::gift_cards::redeem))
        .route("/api/v1/user/gift-card/history", get(user_v1::gift_cards::history))
        .route("/api/v1/user/gift-card/detail", get(user_v1::gift_cards::detail))
        .route("/api/v1/user/gift-card/types", get(user_v1::gift_cards::types))
        .route("/api/v1/user/sponsor/methods", get(user_v1::sponsors::methods))
        .route("/api/v1/user/sponsor", post(user_v1::sponsors::create))
        .route("/api/v1/user/sponsor/{tradeNo}", get(user_v1::sponsors::detail))
        .route("/api/v1/user/sponsor/checkout", post(user_v1::sponsors::checkout))
        .route_layer(middleware::from_fn_with_state(
            state,
            require_user_api_access,
        ))
}

fn admin_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    let secure_v2_state = state.clone();
    Router::new()
        .route("/api/v1/admin/refunds/finalize", post(admin_v1::refunds::finalize))
        .route("/api/v1/admin/refunds", get(admin_v1::refunds::index))
        .route("/api/v1/admin/refunds/{id}", get(admin_v1::refunds::detail))
        .route("/api/v1/admin/refunds/{id}/approve", post(admin_v1::refunds::approve))
        .route("/api/v1/admin/refunds/{id}/deny", post(admin_v1::refunds::deny))
        .route("/api/v1/admin/command-center", get(admin_v1::command_center::show))
        .route("/api/v1/admin/node-plans", get(admin_v1::node_plans::index).post(admin_v1::node_plans::store))
        .route("/api/v1/admin/node-plans/node-options", get(admin_v1::node_plans::node_options))
        .route("/api/v1/admin/node-plans/{id}", put(admin_v1::node_plans::update).delete(admin_v1::node_plans::destroy))
        .route("/api/v1/admin/api-key", get(admin_v1::api_keys::show))
        .route("/api/v1/admin/api-key/generate", post(admin_v1::api_keys::generate))
        .route("/api/v1/admin/api-key/reset", post(admin_v1::api_keys::reset))
        .route("/api/v1/admin/api-key/validate", post(admin_v1::api_keys::validate))
        .route("/api/v1/admin/api-keys/stats", get(admin_v1::api_keys::stats))
        .route("/api/v1/admin/api-keys/search", get(admin_v1::api_keys::search))
        .route("/api/v1/admin/api-keys/batch-generate", post(admin_v1::api_keys::batch_generate))
        .route("/api/v1/admin/api-keys/cleanup", post(admin_v1::api_keys::cleanup))
        .route("/api/v1/admin/group-limits", get(admin_v1::group_limits::index).post(admin_v1::group_limits::store))
        .route("/api/v1/admin/group-limits/batch", put(admin_v1::group_limits::batch_update))
        .route("/api/v1/admin/group-limits/defaults/template", get(admin_v1::group_limits::defaults_template))
        .route("/api/v1/admin/group-limits/defaults/apply", post(admin_v1::group_limits::defaults_apply))
        .route("/api/v1/admin/group-limits/{trustLevel}", get(admin_v1::group_limits::show).delete(admin_v1::group_limits::destroy))
        .route("/api/v1/admin/sponsor-epay", get(admin_v1::sponsor_epay::show).put(admin_v1::sponsor_epay::upsert))
        .route("/api/v1/admin/users", get(admin_v1::users_limits::index))
        .route("/api/v1/admin/users/limits/batch", put(admin_v1::users_limits::batch_update))
        .route("/api/v1/admin/users/{userId}/concurrent-ip-limit", get(admin_v1::users_limits::concurrent_ip_limit_show).put(admin_v1::users_limits::concurrent_ip_limit_update))
        .route("/api/v1/admin/users/{userId}/limits", get(admin_v1::users_limits::user_limits_show).post(admin_v1::users_limits::user_limits_store).delete(admin_v1::users_limits::user_limits_destroy))
        .route("/api/v1/admin/users/{userId}/api-key", get(admin_v1::api_keys::user_show))
        .route("/api/v1/admin/users/{userId}/api-key/generate", post(admin_v1::api_keys::user_generate))
        .route("/api/v1/admin/users/{userId}/api-key/reset", post(admin_v1::api_keys::user_reset))
        .nest("/api/v2/admin", admin_v2::router::routes())
        .nest("/api/v2/{admin_path}", admin_v2::router::secure_routes(secure_v2_state))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_admin_api_access,
        ))
        .route_layer(middleware::from_fn_with_state(
            state,
            require_admin_api_path,
        ))
}

async fn require_user_api_access(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    require_account_role(&state, request, next, RequiredAccountRole::User).await
}

async fn require_admin_api_access(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    require_account_role(&state, request, next, RequiredAccountRole::Admin).await
}

async fn require_super_admin_api_access(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    require_account_role(
        &state,
        request,
        next,
        RequiredAccountRole::SuperAdmin,
    )
    .await
}

async fn require_admin_api_path(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let candidate = request
        .uri()
        .path()
        .strip_prefix("/api/v2/")
        .and_then(|tail| tail.split('/').next())
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .or_else(|| admin_path_header(request.headers()).map(str::to_owned));
    require_configured_admin_path(&state, candidate.as_deref(), request, next).await
}

async fn require_fixed_admin_path(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let candidate = admin_path_header(request.headers()).map(str::to_owned);
    require_configured_admin_path(&state, candidate.as_deref(), request, next).await
}

fn admin_path_header(headers: &axum::http::HeaderMap) -> Option<&str> {
    headers
        .get("x-admin-path")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
}

async fn require_configured_admin_path(
    state: &AppState,
    candidate: Option<&str>,
    request: Request,
    next: Next,
) -> Response {
    let expected = first_non_empty(&[
        get_setting_string(state, "secure_path", "").await,
        get_setting_string(state, "frontend_admin_path", "").await,
        String::new(),
    ]);
    if candidate.is_some_and(|candidate| secure_admin_path_matches(&expected, candidate)) {
        next.run(request).await
    } else {
        json_error(axum::http::StatusCode::NOT_FOUND, "Not found")
    }
}

async fn require_account_role(
    state: &AppState,
    mut request: Request,
    next: Next,
    required: RequiredAccountRole,
) -> Response {
    match authenticate_route_account(state, request.headers(), required).await {
        Ok(account) => {
            request.extensions_mut().insert(account);
            let mut response = next.run(request).await;
            exposure_control::insert_authenticated_cache_headers(response.headers_mut());
            response
        }
        Err(response) => response,
    }
}

fn client_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/client/app/getConfig", get(client_v1::app_get_config))
        .route("/api/v1/client/app/getVersion", get(client_v1::app_get_version))
        .route("/api/v1/client/{path}", get(client_v1::subscribe_legacy))
        .route_layer(middleware::from_fn(exposure_control::mark_private_response))
}

fn server_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/server/ShadowsocksTidalab/user", get(crate::legacy_server_v1::shadowsocks_user))
        .route("/api/v1/server/ShadowsocksTidalab/submit", post(crate::legacy_server_v1::shadowsocks_submit))
        .route("/api/v1/server/TrojanTidalab/config", get(crate::legacy_server_v1::trojan_config))
        .route("/api/v1/server/TrojanTlab/user", get(crate::legacy_server_v1::trojan_user))
        .route("/api/v1/server/TrojanTlab/submit", post(crate::legacy_server_v1::trojan_submit))
        .route("/api/v1/server/TrojanTidalab/user", get(crate::legacy_server_v1::trojan_user))
        .route("/api/v1/server/TrojanTidalab/submit", post(crate::legacy_server_v1::trojan_submit))
        .route("/api/v1/server/UniProxy/config", any(uniproxy_config))
        .route("/api/v1/server/UniProxy/user", any(uniproxy_user))
        .route("/api/v1/server/UniProxy/alivelist", any(uniproxy_alivelist))
        .route("/api/v2/server/config", any(uniproxy_config))
        .route("/api/v2/server/user", any(uniproxy_user))
        .route("/api/v2/server/alivelist", any(uniproxy_alivelist))
        .route("/api/v1/server/UniProxy/push", post(uniproxy_push))
        .route("/api/v1/server/UniProxy/alive", post(uniproxy_alive))
        .route("/api/v1/server/UniProxy/status", post(uniproxy_status))
        .route("/api/v1/server/UniProxy/audit", post(uniproxy_audit))
        .route("/api/v2/server/push", post(uniproxy_push))
        .route("/api/v2/server/alive", post(uniproxy_alive))
        .route("/api/v2/server/status", post(uniproxy_status))
        .route("/api/v2/server/audit", post(uniproxy_audit))
        .route_layer(middleware::from_fn(exposure_control::mark_private_response))
}

fn agent_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/v1/agent/bootstrap/{kind}", post(crate::machine_bootstrap_support::exchange))
        .route("/api/v1/tcping/agent/config", get(tcping_agent_v1::config).post(tcping_agent_v1::config))
        .route("/api/v1/tcping/agent/heartbeat", post(tcping_agent_v1::heartbeat))
        .route("/api/v1/tcping/agent/samples", post(tcping_agent_v1::samples))
        .route_layer(middleware::from_fn(exposure_control::mark_private_response))
}
