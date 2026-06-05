use crate::*;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct DailyStatisticsRunStats {
    pub(crate) written: bool,
    pub(crate) record_at: i64,
    pub(crate) order_count: i64,
    pub(crate) order_total: i64,
    pub(crate) paid_count: i64,
    pub(crate) paid_total: i64,
    pub(crate) commission_count: i64,
    pub(crate) commission_total: i64,
    pub(crate) register_count: i64,
    pub(crate) invite_count: i64,
    pub(crate) transfer_used_total: i64,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct TrafficTotals {
    pub(crate) upload: i64,
    pub(crate) download: i64,
    pub(crate) total: i64,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct AdminDashboardSummary {
    pub(crate) online_nodes: i64,
    pub(crate) online_devices: i64,
    pub(crate) online_users: i64,
    pub(crate) current_month_register_total: i64,
    pub(crate) last_month_register_total: i64,
    pub(crate) ticket_pending_total: i64,
    pub(crate) commission_pending_total: i64,
    pub(crate) today_income: i64,
    pub(crate) yesterday_income: i64,
    pub(crate) current_month_income: i64,
    pub(crate) last_month_income: i64,
    pub(crate) two_months_ago_income: i64,
    pub(crate) commission_month_payout: i64,
    pub(crate) commission_last_month_payout: i64,
    pub(crate) today_traffic: TrafficTotals,
    pub(crate) month_traffic: TrafficTotals,
    pub(crate) total_traffic: TrafficTotals,
}

#[derive(Clone, Copy, Debug)]
struct DashboardWindows {
    now: i64,
    today_start: i64,
    yesterday_start: i64,
    current_month_start: i64,
    last_month_start: i64,
    two_months_ago_start: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct AdminDashboardUserSnapshotRow {
    online_nodes: i64,
    online_devices: i64,
    online_users: i64,
    current_month_register_total: i64,
    last_month_register_total: i64,
    ticket_pending_total: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct AdminDashboardOrderSnapshotRow {
    today_income: i64,
    yesterday_income: i64,
    current_month_income: i64,
    last_month_income: i64,
    two_months_ago_income: i64,
    commission_pending_total: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct AdminDashboardCommissionSnapshotRow {
    commission_month_payout: i64,
    commission_last_month_payout: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct AdminDashboardTrafficSnapshotRow {
    today_upload: i64,
    today_download: i64,
    today_total: i64,
    month_upload: i64,
    month_download: i64,
    month_total: i64,
    total_upload: i64,
    total_download: i64,
    total_total: i64,
}

pub(crate) async fn run_daily_statistics(
    state: &AppState,
    now_ts: i64,
) -> Result<DailyStatisticsRunStats, sqlx::Error> {
    let tz = chrono::FixedOffset::east_opt(8 * 3600)
        .ok_or_else(|| sqlx::Error::Protocol("invalid timezone offset".to_string()))?;
    let now = chrono::DateTime::from_timestamp(now_ts, 0)
        .ok_or_else(|| sqlx::Error::Protocol("invalid current timestamp".to_string()))?
        .with_timezone(&tz);
    let today_start = tz
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .single()
        .ok_or_else(|| sqlx::Error::Protocol("invalid statistics schedule".to_string()))?;
    let scheduled = tz
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 10, 0)
        .single()
        .ok_or_else(|| sqlx::Error::Protocol("invalid statistics schedule".to_string()))?;
    if now < scheduled {
        return Ok(DailyStatisticsRunStats::default());
    }

    let target_start = today_start - chrono::Duration::days(1);
    let target_end = today_start;
    let record_at = target_start.timestamp();

    let exists: Option<i64> = sqlx::query_scalar(
        "SELECT id
         FROM v2_stat
         WHERE record_at = ? AND record_type = 'd'
         LIMIT 1",
    )
    .bind(record_at)
    .fetch_optional(&state.db)
    .await?;
    if exists.is_some() {
        return Ok(DailyStatisticsRunStats {
            written: false,
            record_at,
            ..Default::default()
        });
    }

    let stats = build_daily_statistics(state, target_start.timestamp(), target_end.timestamp(), now_ts).await?;
    upsert_daily_statistics(state, &stats, now_ts).await?;
    Ok(stats)
}

pub(crate) async fn load_admin_dashboard_summary(
    state: &AppState,
    now: i64,
) -> Result<AdminDashboardSummary, sqlx::Error> {
    let windows = dashboard_windows(now);
    let (users, orders, commissions, traffic) = tokio::try_join!(
        load_admin_dashboard_user_snapshot(state, windows),
        load_admin_dashboard_order_snapshot(state, windows),
        load_admin_dashboard_commission_snapshot(state, windows),
        load_admin_dashboard_traffic_snapshot(state, windows),
    )?;

    Ok(AdminDashboardSummary {
        online_nodes: users.online_nodes,
        online_devices: users.online_devices,
        online_users: users.online_users,
        current_month_register_total: users.current_month_register_total,
        last_month_register_total: users.last_month_register_total,
        ticket_pending_total: users.ticket_pending_total,
        commission_pending_total: orders.commission_pending_total,
        today_income: orders.today_income,
        yesterday_income: orders.yesterday_income,
        current_month_income: orders.current_month_income,
        last_month_income: orders.last_month_income,
        two_months_ago_income: orders.two_months_ago_income,
        commission_month_payout: commissions.commission_month_payout,
        commission_last_month_payout: commissions.commission_last_month_payout,
        today_traffic: TrafficTotals {
            upload: traffic.today_upload,
            download: traffic.today_download,
            total: traffic.today_total,
        },
        month_traffic: TrafficTotals {
            upload: traffic.month_upload,
            download: traffic.month_download,
            total: traffic.month_total,
        },
        total_traffic: TrafficTotals {
            upload: traffic.total_upload,
            download: traffic.total_download,
            total: traffic.total_total,
        },
    })
}

pub(crate) fn percentage_growth(current: i64, previous: i64) -> f64 {
    if previous == 0 {
        if current == 0 {
            0.0
        } else {
            100.0
        }
    } else {
        (((current - previous) as f64) / (previous as f64) * 100.0 * 100.0).round() / 100.0
    }
}

async fn build_daily_statistics(
    state: &AppState,
    start_at: i64,
    end_at: i64,
    _now_ts: i64,
) -> Result<DailyStatisticsRunStats, sqlx::Error> {
    let order_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM v2_order
         WHERE created_at >= ? AND created_at < ?",
    )
    .bind(start_at)
    .bind(end_at)
    .fetch_one(&state.db)
    .await?;

    let order_total: i64 = sqlx::query_scalar(
        "SELECT CAST(COALESCE(SUM(total_amount), 0) AS SIGNED)
         FROM v2_order
         WHERE created_at >= ? AND created_at < ?",
    )
    .bind(start_at)
    .bind(end_at)
    .fetch_one(&state.db)
    .await?;

    let paid_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM v2_order
         WHERE paid_at >= ? AND paid_at < ?
           AND status NOT IN (0, 2)",
    )
    .bind(start_at)
    .bind(end_at)
    .fetch_one(&state.db)
    .await?;

    let paid_total: i64 = sqlx::query_scalar(
        "SELECT CAST(COALESCE(SUM(total_amount), 0) AS SIGNED)
         FROM v2_order
         WHERE paid_at >= ? AND paid_at < ?
           AND status NOT IN (0, 2)",
    )
    .bind(start_at)
    .bind(end_at)
    .fetch_one(&state.db)
    .await?;

    let commission_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM v2_commission_log
         WHERE created_at >= ? AND created_at < ?",
    )
    .bind(start_at)
    .bind(end_at)
    .fetch_one(&state.db)
    .await?;

    let commission_total: i64 = sqlx::query_scalar(
        "SELECT CAST(COALESCE(SUM(get_amount), 0) AS SIGNED)
         FROM v2_commission_log
         WHERE created_at >= ? AND created_at < ?",
    )
    .bind(start_at)
    .bind(end_at)
    .fetch_one(&state.db)
    .await?;

    let register_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM v2_user
         WHERE created_at >= ? AND created_at < ?",
    )
    .bind(start_at)
    .bind(end_at)
    .fetch_one(&state.db)
    .await?;

    let invite_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM v2_user
         WHERE created_at >= ? AND created_at < ?
           AND invite_user_id IS NOT NULL",
    )
    .bind(start_at)
    .bind(end_at)
    .fetch_one(&state.db)
    .await?;

    let transfer_used_total: i64 = sqlx::query_scalar(
        "SELECT CAST(COALESCE(SUM(u) + SUM(d), 0) AS SIGNED)
         FROM v2_stat_server
         WHERE created_at >= ? AND created_at < ?",
    )
    .bind(start_at)
    .bind(end_at)
    .fetch_one(&state.db)
    .await?;

    Ok(DailyStatisticsRunStats {
        written: true,
        record_at: start_at,
        order_count,
        order_total,
        paid_count,
        paid_total,
        commission_count,
        commission_total,
        register_count,
        invite_count,
        transfer_used_total,
    })
}

async fn upsert_daily_statistics(
    state: &AppState,
    stats: &DailyStatisticsRunStats,
    now_ts: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO v2_stat
            (record_at, record_type, order_count, order_total, commission_count, commission_total, paid_count, paid_total, register_count, invite_count, transfer_used_total, created_at, updated_at)
         VALUES (?, 'd', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON DUPLICATE KEY UPDATE
            order_count = VALUES(order_count),
            order_total = VALUES(order_total),
            commission_count = VALUES(commission_count),
            commission_total = VALUES(commission_total),
            paid_count = VALUES(paid_count),
            paid_total = VALUES(paid_total),
            register_count = VALUES(register_count),
            invite_count = VALUES(invite_count),
            transfer_used_total = VALUES(transfer_used_total),
            updated_at = VALUES(updated_at)",
    )
    .bind(stats.record_at)
    .bind(stats.order_count)
    .bind(stats.order_total)
    .bind(stats.commission_count)
    .bind(stats.commission_total)
    .bind(stats.paid_count)
    .bind(stats.paid_total)
    .bind(stats.register_count)
    .bind(stats.invite_count)
    .bind(stats.transfer_used_total.to_string())
    .bind(now_ts)
    .bind(now_ts)
    .execute(&state.db)
    .await?;
    Ok(())
}

fn dashboard_windows(now: i64) -> DashboardWindows {
    let now_dt = chrono::DateTime::<Utc>::from_timestamp(now, 0).unwrap_or_else(Utc::now);
    let today_start = now_dt
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .map(|dt| dt.and_utc().timestamp())
        .unwrap_or(now - now.rem_euclid(86_400));
    let current_month_start = now_dt
        .date_naive()
        .with_day(1)
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .map(|dt| dt.and_utc().timestamp())
        .unwrap_or(today_start);
    let last_month_start = now_dt
        .date_naive()
        .with_day(1)
        .and_then(|date| date.pred_opt())
        .and_then(|date| date.with_day(1))
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .map(|dt| dt.and_utc().timestamp())
        .unwrap_or(current_month_start - 2_592_000);

    DashboardWindows {
        now,
        today_start,
        yesterday_start: today_start - 86_400,
        current_month_start,
        last_month_start,
        two_months_ago_start: last_month_start - 2_592_000,
    }
}

async fn load_admin_dashboard_user_snapshot(
    state: &AppState,
    windows: DashboardWindows,
) -> Result<AdminDashboardUserSnapshotRow, sqlx::Error> {
    sqlx::query_as::<_, AdminDashboardUserSnapshotRow>(
        "SELECT
            (SELECT COUNT(*) FROM v2_server WHERE `show` = 1) AS online_nodes,
            CAST(COALESCE(SUM(CASE WHEN t >= ? THEN online_count ELSE 0 END),0) AS SIGNED) AS online_devices,
            CAST(COALESCE(SUM(CASE WHEN t >= ? THEN 1 ELSE 0 END),0) AS SIGNED) AS online_users,
            CAST(COALESCE(SUM(CASE WHEN created_at >= ? AND created_at < ? THEN 1 ELSE 0 END),0) AS SIGNED) AS current_month_register_total,
            CAST(COALESCE(SUM(CASE WHEN created_at >= ? AND created_at < ? THEN 1 ELSE 0 END),0) AS SIGNED) AS last_month_register_total,
            (SELECT COUNT(*) FROM v2_ticket WHERE status = 0) AS ticket_pending_total
         FROM v2_user"
    )
    .bind(windows.now - 600)
    .bind(windows.now - 600)
    .bind(windows.current_month_start)
    .bind(windows.now)
    .bind(windows.last_month_start)
    .bind(windows.current_month_start)
    .fetch_one(&state.db)
    .await
}

async fn load_admin_dashboard_order_snapshot(
    state: &AppState,
    windows: DashboardWindows,
) -> Result<AdminDashboardOrderSnapshotRow, sqlx::Error> {
    sqlx::query_as::<_, AdminDashboardOrderSnapshotRow>(
        "SELECT
            CAST(COALESCE(SUM(CASE WHEN created_at >= ? AND created_at < ? AND status NOT IN (0, 2) THEN total_amount ELSE 0 END),0) AS SIGNED) AS today_income,
            CAST(COALESCE(SUM(CASE WHEN created_at >= ? AND created_at < ? AND status NOT IN (0, 2) THEN total_amount ELSE 0 END),0) AS SIGNED) AS yesterday_income,
            CAST(COALESCE(SUM(CASE WHEN created_at >= ? AND created_at < ? AND status NOT IN (0, 2) THEN total_amount ELSE 0 END),0) AS SIGNED) AS current_month_income,
            CAST(COALESCE(SUM(CASE WHEN created_at >= ? AND created_at < ? AND status NOT IN (0, 2) THEN total_amount ELSE 0 END),0) AS SIGNED) AS last_month_income,
            CAST(COALESCE(SUM(CASE WHEN created_at >= ? AND created_at < ? AND status NOT IN (0, 2) THEN total_amount ELSE 0 END),0) AS SIGNED) AS two_months_ago_income,
            CAST(COALESCE(SUM(CASE WHEN commission_status = 0 AND invite_user_id IS NOT NULL AND status NOT IN (0, 2) AND commission_balance > 0 THEN 1 ELSE 0 END),0) AS SIGNED) AS commission_pending_total
         FROM v2_order"
    )
    .bind(windows.today_start)
    .bind(windows.now)
    .bind(windows.yesterday_start)
    .bind(windows.today_start)
    .bind(windows.current_month_start)
    .bind(windows.now)
    .bind(windows.last_month_start)
    .bind(windows.current_month_start)
    .bind(windows.two_months_ago_start)
    .bind(windows.last_month_start)
    .fetch_one(&state.db)
    .await
}

async fn load_admin_dashboard_commission_snapshot(
    state: &AppState,
    windows: DashboardWindows,
) -> Result<AdminDashboardCommissionSnapshotRow, sqlx::Error> {
    sqlx::query_as::<_, AdminDashboardCommissionSnapshotRow>(
        "SELECT
            CAST(COALESCE(SUM(CASE WHEN created_at >= ? AND created_at < ? THEN get_amount ELSE 0 END),0) AS SIGNED) AS commission_month_payout,
            CAST(COALESCE(SUM(CASE WHEN created_at >= ? AND created_at < ? THEN get_amount ELSE 0 END),0) AS SIGNED) AS commission_last_month_payout
         FROM v2_commission_log"
    )
    .bind(windows.current_month_start)
    .bind(windows.now)
    .bind(windows.last_month_start)
    .bind(windows.current_month_start)
    .fetch_one(&state.db)
    .await
}

async fn load_admin_dashboard_traffic_snapshot(
    state: &AppState,
    windows: DashboardWindows,
) -> Result<AdminDashboardTrafficSnapshotRow, sqlx::Error> {
    sqlx::query_as::<_, AdminDashboardTrafficSnapshotRow>(
        "SELECT
            CAST(COALESCE(SUM(CASE WHEN record_at >= ? AND record_at < ? THEN u ELSE 0 END),0) AS SIGNED) AS today_upload,
            CAST(COALESCE(SUM(CASE WHEN record_at >= ? AND record_at < ? THEN d ELSE 0 END),0) AS SIGNED) AS today_download,
            CAST(COALESCE(SUM(CASE WHEN record_at >= ? AND record_at < ? THEN u + d ELSE 0 END),0) AS SIGNED) AS today_total,
            CAST(COALESCE(SUM(CASE WHEN record_at >= ? AND record_at < ? THEN u ELSE 0 END),0) AS SIGNED) AS month_upload,
            CAST(COALESCE(SUM(CASE WHEN record_at >= ? AND record_at < ? THEN d ELSE 0 END),0) AS SIGNED) AS month_download,
            CAST(COALESCE(SUM(CASE WHEN record_at >= ? AND record_at < ? THEN u + d ELSE 0 END),0) AS SIGNED) AS month_total,
            CAST(COALESCE(SUM(u),0) AS SIGNED) AS total_upload,
            CAST(COALESCE(SUM(d),0) AS SIGNED) AS total_download,
            CAST(COALESCE(SUM(u + d),0) AS SIGNED) AS total_total
         FROM v2_stat_server"
    )
    .bind(windows.today_start)
    .bind(windows.now)
    .bind(windows.today_start)
    .bind(windows.now)
    .bind(windows.today_start)
    .bind(windows.now)
    .bind(windows.current_month_start)
    .bind(windows.now)
    .bind(windows.current_month_start)
    .bind(windows.now)
    .bind(windows.current_month_start)
    .bind(windows.now)
    .fetch_one(&state.db)
    .await
}
