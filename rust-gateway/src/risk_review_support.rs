use crate::*;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct RiskReviewRunStats {
    pub(crate) ran: bool,
    pub(crate) groups_scanned: i64,
    pub(crate) reviews_created: i64,
    pub(crate) users_skipped: i64,
    pub(crate) telegram_notifications: i64,
}

pub(crate) async fn run_scheduled_risk_review(
    state: &AppState,
    now_ts: i64,
) -> Result<RiskReviewRunStats, sqlx::Error> {
    if !get_setting_bool(state, "user_risk_review_enable", false).await {
        return Ok(RiskReviewRunStats::default());
    }

    let interval_minutes = get_setting_int(state, "user_risk_review_schedule_minutes", 30)
        .await
        .max(5);
    let marker_key = "user_risk_review:last_run_at";
    let last_run_at = redis_get_string(state, marker_key)
        .await
        .ok()
        .flatten()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(0);
    if last_run_at > 0 && (now_ts - last_run_at) < interval_minutes * 60 {
        return Ok(RiskReviewRunStats::default());
    }

    let limit = get_setting_int(state, "user_risk_review_scan_limit", 20)
        .await
        .max(1)
        .min(200);
    let summary = crate::admin_v2::risk_review::run_risk_review_scan(state, limit).await?;

    let _ = redis_setex_string(state, marker_key, 86_400, &now_ts.to_string()).await;

    Ok(RiskReviewRunStats {
        ran: true,
        groups_scanned: summary.get("groups_scanned").and_then(Value::as_i64).unwrap_or(0),
        reviews_created: summary.get("reviews_created").and_then(Value::as_i64).unwrap_or(0),
        users_skipped: summary.get("users_skipped").and_then(Value::as_i64).unwrap_or(0),
        telegram_notifications: summary.get("telegram_notifications").and_then(Value::as_i64).unwrap_or(0),
    })
}
