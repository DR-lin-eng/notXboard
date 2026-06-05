use super::super::*;
use super::{
    config, coupon, gift_card, knowledge, notice, order, payment, plan, plugin,
    public_dashboard, risk_review, server_group, server_manage, server_route, stat, system,
    theme, ticket, traffic_reset, user,
};
use axum::{
    extract::{Path, Request, State},
    middleware::{self, Next},
    routing::{any, get, post},
    response::Response,
    Router,
};
use std::{collections::HashMap, sync::Arc};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/gift-card/templates", get(gift_card::templates).post(gift_card::templates_post))
        .route("/gift-card/create-template", post(gift_card::create_template))
        .route("/gift-card/update-template", post(gift_card::update_template))
        .route("/gift-card/delete-template", post(gift_card::delete_template))
        .route("/gift-card/generate-codes", post(gift_card::generate_codes))
        .route("/gift-card/types", get(gift_card::types))
        .route("/gift-card/statistics", get(gift_card::statistics).post(gift_card::statistics_post))
        .route("/gift-card/codes", get(gift_card::codes).post(gift_card::codes_post))
        .route("/gift-card/toggle-code", post(gift_card::toggle_code))
        .route("/gift-card/export-codes", get(gift_card::export_codes))
        .route("/gift-card/update-code", post(gift_card::update_code))
        .route("/gift-card/delete-code", post(gift_card::delete_code))
        .route("/gift-card/usages", get(gift_card::usages).post(gift_card::usages_post))
        .route("/notice/fetch", get(notice::fetch))
        .route("/notice/save", post(notice::save))
        .route("/notice/update", post(notice::update))
        .route("/notice/show", post(notice::show))
        .route("/notice/drop", post(notice::drop))
        .route("/notice/sort", post(notice::sort))
        .route("/knowledge/fetch", get(knowledge::fetch))
        .route("/knowledge/getCategory", get(knowledge::get_category))
        .route("/knowledge/save", post(knowledge::save))
        .route("/knowledge/show", post(knowledge::show))
        .route("/knowledge/drop", post(knowledge::drop))
        .route("/knowledge/sort", post(knowledge::sort))
        .route("/ticket/fetch", get(ticket::fetch).post(ticket::fetch_post))
        .route("/ticket/reply", post(ticket::reply))
        .route("/ticket/close", post(ticket::close))
        .route("/payment/fetch", get(payment::fetch))
        .route("/payment/getPaymentMethods", get(payment::get_payment_methods))
        .route("/payment/getPaymentForm", post(payment::get_payment_form))
        .route("/payment/save", post(payment::save))
        .route("/payment/show", post(payment::show))
        .route("/payment/drop", post(payment::drop))
        .route("/payment/sort", post(payment::sort))
        .route("/system/getSystemStatus", get(system::get_system_status))
        .route("/system/getQueueStats", get(system::get_queue_stats))
        .route("/system/getQueueWorkload", get(system::get_queue_workload))
        .route("/system/getQueueMasters", get(system::get_queue_masters))
        .route("/system/getSystemLog", get(system::get_system_log))
        .route("/system/getHorizonFailedJobs", get(system::get_horizon_failed_jobs))
        .route("/system/exportLogsCsv", get(system::maintenance::export_logs_csv))
        .route("/system/listHooks", get(system::maintenance::list_hooks))
        .route("/system/cleanupDormantUsers", post(system::maintenance::cleanup_dormant_users))
        .route("/system/resetAllUserSecurity", post(system::maintenance::reset_all_user_security))
        .route("/system/resetUserPassword", post(system::maintenance::reset_user_password))
        .route("/system/runDatabaseBackup", post(system::run_database_backup))
        .route("/system/clearSystemLog", post(system::clear_system_log))
        .route("/system/getLogClearStats", get(system::get_log_clear_stats))
        .route("/plan/fetch", get(plan::fetch))
        .route("/plan/save", post(plan::save))
        .route("/plan/update", post(plan::update))
        .route("/plan/drop", post(plan::drop))
        .route("/plan/sort", post(plan::sort))
        .route("/server/group/fetch", get(server_group::fetch))
        .route("/server/group/save", post(server_group::save))
        .route("/server/group/drop", post(server_group::drop))
        .route("/server/route/fetch", get(server_route::fetch))
        .route("/server/route/save", post(server_route::save))
        .route("/server/route/drop", post(server_route::drop))
        .route("/server/manage/getNodes", get(server_manage::get_nodes))
        .route("/server/manage/save", post(server_manage::save))
        .route("/server/manage/sort", post(server_manage::sort))
        .route("/server/manage/update", post(server_manage::update))
        .route("/server/manage/drop", post(server_manage::drop))
        .route("/server/manage/copy", post(server_manage::copy))
        .route("/order/fetch", any(order::fetch))
        .route("/order/detail", any(order::detail))
        .route("/order/paid", any(order::paid))
        .route("/order/cancel", any(order::cancel))
        .route("/order/update", any(order::update))
        .route("/order/assign", any(order::assign))
        .route("/user/fetch", any(user::fetch))
        .route("/user/update", post(user::update))
        .route("/user/getUserInfoById", get(user::get_user_info_by_id))
        .route("/user/resetSecret", post(user::reset_secret))
        .route("/user/setInviteUser", post(user::set_invite_user))
        .route("/user/generate", post(user::generate))
        .route("/user/destroy", post(user::destroy))
        .route("/user/ban", post(user::ban))
        .route("/user/ban-records", get(user::ban_records))
        .route("/user/sendMail", post(user::send_mail))
        .route("/user/dumpCSV", post(user::dump_csv))
        .route("/risk-review/fetch", get(risk_review::fetch))
        .route("/risk-review/run", post(risk_review::run))
        .route("/config/fetch", get(config::fetch))
        .route("/config/save", post(config::save))
        .route("/config/getEmailTemplate", get(config::get_email_template))
        .route("/config/setTelegramWebhook", post(config::set_telegram_webhook))
        .route("/config/telegram/setWebhook", post(config::set_telegram_webhook))
        .route("/config/testSendMail", post(config::test_send_mail))
        .route("/plugin/types", get(plugin::types))
        .route("/plugin/getPlugins", get(plugin::get_plugins))
        .route("/plugin/config", any(plugin::config))
        .route("/plugin/upload", post(plugin::upload))
        .route("/plugin/install", post(plugin::install))
        .route("/plugin/uninstall", post(plugin::uninstall))
        .route("/plugin/enable", post(plugin::enable))
        .route("/plugin/disable", post(plugin::disable))
        .route("/plugin/upgrade", post(plugin::upgrade))
        .route("/plugin/delete", post(plugin::delete))
        .route("/theme/getThemes", get(theme::get_themes))
        .route("/theme/getThemeConfig", post(theme::get_theme_config))
        .route("/theme/saveThemeConfig", post(theme::save_theme_config))
        .route("/theme/upload", post(theme::upload_theme))
        .route("/theme/delete", post(theme::delete_theme))
        .route("/stat/getOverride", get(stat::get_override))
        .route("/stat/getStats", get(stat::get_stats))
        .route("/stat/getOrder", get(stat::get_order))
        .route("/stat/getStatUser", any(stat::get_stat_user))
        .route("/stat/getTrafficRank", get(stat::get_traffic_rank))
        .route("/stat/getServerLastRank", get(stat::get_server_last_rank))
        .route("/stat/getServerYesterdayRank", get(stat::get_server_yesterday_rank))
        .route("/stat/getStatRecord", get(stat::get_stat_record))
        .route("/stat/getRanking", get(stat::get_ranking))
        .route("/public-dashboard/overview", get(public_dashboard::overview))
        .route("/public-dashboard/leaderboards", get(public_dashboard::leaderboards))
        .route("/public-dashboard/geo", get(public_dashboard::geo))
        .route("/coupon/fetch", get(coupon::fetch).post(coupon::fetch_post))
        .route("/coupon/generate", post(coupon::generate))
        .route("/coupon/show", post(coupon::show))
        .route("/coupon/update", post(coupon::update))
        .route("/coupon/drop", post(coupon::drop))
        .route("/traffic-reset/logs", get(traffic_reset::logs))
        .route("/traffic-reset/stats", get(traffic_reset::stats))
        .route("/traffic-reset/reset-user", post(traffic_reset::reset_user))
        .route("/traffic-reset/user/{userId}/history", get(traffic_reset::user_history))
}

pub fn secure_routes(state: Arc<AppState>) -> Router<Arc<AppState>> {
    routes().route_layer(middleware::from_fn_with_state(state, require_secure_path))
}

async fn require_secure_path(
    State(state): State<Arc<AppState>>,
    Path(params): Path<HashMap<String, String>>,
    request: Request,
    next: Next,
) -> Response {
    let admin_path = params
        .get("admin_path")
        .map(|value| value.trim().to_string())
        .unwrap_or_default();
    let expected = first_non_empty(&[
        get_setting_string(&state, "secure_path", "").await,
        get_setting_string(&state, "frontend_admin_path", "").await,
        String::new(),
    ]);
    if expected.trim().is_empty() || admin_path != expected.trim() {
        return json_error(StatusCode::NOT_FOUND, "Not found");
    }
    next.run(request).await
}
