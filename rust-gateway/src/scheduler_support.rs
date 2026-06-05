use crate::*;

#[derive(Clone)]
pub(crate) struct SchedulerRunStats {
    pub(crate) database_backup: crate::backup_support::DatabaseBackupStats,
    pub(crate) expired_online_users_reset: u64,
    pub(crate) expired_node_sessions_deleted: u64,
    pub(crate) traffic_reset_times_initialized: u64,
    pub(crate) traffic_resets_performed: u64,
    pub(crate) offline_server_node_alerts_marked: u64,
    pub(crate) offline_server_node_alerts_sent: u64,
    pub(crate) subscription_credentials_rotated: u64,
    pub(crate) pending_orders_cancelled: u64,
    pub(crate) processing_orders_completed: u64,
    pub(crate) stale_tickets_closed: u64,
    pub(crate) commissions_auto_checked: u64,
    pub(crate) commissions_paid: u64,
    pub(crate) expired_refund_votings_finalized: u64,
    pub(crate) daily_log_cleanup: DailyLogCleanupStats,
    pub(crate) daily_statistics: crate::stats_support::DailyStatisticsRunStats,
    pub(crate) horizon_metrics_snapshot: crate::horizon_metrics_support::HorizonMetricsSnapshotStats,
    pub(crate) linux_do_user_sync: crate::oauth_sync_support::LinuxDoUserSyncStats,
    pub(crate) risk_review: crate::risk_review_support::RiskReviewRunStats,
    pub(crate) send_remind_mail: crate::mail_reminder_support::SendRemindMailStats,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct DailyLogCleanupStats {
    pub(crate) stat_user_deleted: u64,
    pub(crate) stat_server_deleted: u64,
    pub(crate) app_log_deleted: u64,
    pub(crate) node_traffic_records_deleted: u64,
    pub(crate) user_traffic_usage_logs_deleted: u64,
    pub(crate) tcping_samples_deleted: u64,
    pub(crate) tcping_alerts_deleted: u64,
    pub(crate) audit_logs_deleted: u64,
}

pub(crate) fn spawn_background_scheduler(state: AppState) {
    tokio::spawn(async move {
        run_background_scheduler_loop(state).await;
    });
}

async fn run_background_scheduler_loop(state: AppState) {
    const SCHEDULER_LOCK_KEY: &str = "rust:scheduler:minute-lock";
    const SCHEDULER_HEARTBEAT_KEY: &str = "SCHEDULE_LAST_CHECK_AT";
    let instance_id = random_uuid_string();
    let mut ticker = tokio::time::interval(Duration::from_secs(60));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    // Run once shortly after boot so command-center can observe Rust-owned heartbeat
    tokio::time::sleep(Duration::from_secs(2)).await;
    if let Err(err) = run_background_scheduler_tick(
        &state,
        &instance_id,
        SCHEDULER_LOCK_KEY,
        SCHEDULER_HEARTBEAT_KEY,
    )
    .await
    {
        warn!("background scheduler bootstrap tick failed: {}", err);
    }

    loop {
        ticker.tick().await;
        if let Err(err) = run_background_scheduler_tick(
            &state,
            &instance_id,
            SCHEDULER_LOCK_KEY,
            SCHEDULER_HEARTBEAT_KEY,
        )
        .await
        {
            warn!("background scheduler tick failed: {}", err);
        }
    }
}

async fn run_background_scheduler_tick(
    state: &AppState,
    instance_id: &str,
    lock_key: &str,
    heartbeat_key: &str,
) -> Result<(), String> {
    if !bootstrap_support::business_tables_ready(state)
        .await
        .map_err(|err| format!("check business tables failed: {err}"))?
    {
        info!("background scheduler skipped because business tables are not ready yet");
        return Ok(());
    }

    let lock_full_key = format!("{}{}{}", state.redis_prefix, state.cache_prefix, lock_key);
    let acquired = redis_set_nx_ex_raw(state, &lock_full_key, 55, instance_id).await?;
    if !acquired {
        return Ok(());
    }

    let now_ts = Utc::now().timestamp();
    redis_setex_string(state, heartbeat_key, 900, &now_ts.to_string()).await?;

    let stats = run_minutely_scheduler_jobs(state)
        .await
        .map_err(|err| format!("scheduler jobs failed: {}", err))?;
    info!(
        expired_online_users_reset = stats.expired_online_users_reset,
        database_backup_ran = stats.database_backup.ran,
        database_backup_uploaded = stats.database_backup.uploaded,
        database_backup_retained_local_copy = stats.database_backup.retained_local_copy,
        database_backup_compressed_bytes = stats.database_backup.compressed_bytes,
        database_backup_artifact_path = stats.database_backup.artifact_path.as_deref().unwrap_or(""),
        database_backup_uploaded_object = stats.database_backup.uploaded_object.as_deref().unwrap_or(""),
        expired_node_sessions_deleted = stats.expired_node_sessions_deleted,
        traffic_reset_times_initialized = stats.traffic_reset_times_initialized,
        traffic_resets_performed = stats.traffic_resets_performed,
        offline_server_node_alerts_marked = stats.offline_server_node_alerts_marked,
        offline_server_node_alerts_sent = stats.offline_server_node_alerts_sent,
        subscription_credentials_rotated = stats.subscription_credentials_rotated,
        pending_orders_cancelled = stats.pending_orders_cancelled,
        processing_orders_completed = stats.processing_orders_completed,
        stale_tickets_closed = stats.stale_tickets_closed,
        commissions_auto_checked = stats.commissions_auto_checked,
        commissions_paid = stats.commissions_paid,
        expired_refund_votings_finalized = stats.expired_refund_votings_finalized,
        daily_log_cleanup_stat_user_deleted = stats.daily_log_cleanup.stat_user_deleted,
        daily_log_cleanup_stat_server_deleted = stats.daily_log_cleanup.stat_server_deleted,
        daily_log_cleanup_app_log_deleted = stats.daily_log_cleanup.app_log_deleted,
        daily_log_cleanup_node_traffic_records_deleted = stats.daily_log_cleanup.node_traffic_records_deleted,
        daily_log_cleanup_user_traffic_usage_logs_deleted = stats.daily_log_cleanup.user_traffic_usage_logs_deleted,
        daily_log_cleanup_tcping_samples_deleted = stats.daily_log_cleanup.tcping_samples_deleted,
        daily_log_cleanup_tcping_alerts_deleted = stats.daily_log_cleanup.tcping_alerts_deleted,
        daily_log_cleanup_audit_logs_deleted = stats.daily_log_cleanup.audit_logs_deleted,
        daily_statistics_written = stats.daily_statistics.written,
        daily_statistics_record_at = stats.daily_statistics.record_at,
        daily_statistics_order_count = stats.daily_statistics.order_count,
        daily_statistics_order_total = stats.daily_statistics.order_total,
        daily_statistics_paid_count = stats.daily_statistics.paid_count,
        daily_statistics_paid_total = stats.daily_statistics.paid_total,
        daily_statistics_commission_count = stats.daily_statistics.commission_count,
        daily_statistics_commission_total = stats.daily_statistics.commission_total,
        daily_statistics_register_count = stats.daily_statistics.register_count,
        daily_statistics_invite_count = stats.daily_statistics.invite_count,
        daily_statistics_transfer_used_total = stats.daily_statistics.transfer_used_total,
        horizon_metrics_snapshot_ran = stats.horizon_metrics_snapshot.ran,
        horizon_metrics_snapshot_measured_jobs = stats.horizon_metrics_snapshot.measured_jobs,
        horizon_metrics_snapshot_measured_queues = stats.horizon_metrics_snapshot.measured_queues,
        horizon_metrics_snapshot_time = stats.horizon_metrics_snapshot.snapshot_time,
        linux_do_user_sync_ran = stats.linux_do_user_sync.ran,
        linux_do_user_sync_total = stats.linux_do_user_sync.total,
        linux_do_user_sync_synced = stats.linux_do_user_sync.synced,
        linux_do_user_sync_failed = stats.linux_do_user_sync.failed,
        risk_review_ran = stats.risk_review.ran,
        risk_review_groups_scanned = stats.risk_review.groups_scanned,
        risk_review_reviews_created = stats.risk_review.reviews_created,
        risk_review_users_skipped = stats.risk_review.users_skipped,
        risk_review_telegram_notifications = stats.risk_review.telegram_notifications,
        send_remind_mail_ran = stats.send_remind_mail.ran,
        send_remind_mail_processed_users = stats.send_remind_mail.processed_users,
        send_remind_mail_expire_emails = stats.send_remind_mail.expire_emails,
        send_remind_mail_traffic_emails = stats.send_remind_mail.traffic_emails,
        send_remind_mail_errors = stats.send_remind_mail.errors,
        send_remind_mail_skipped = stats.send_remind_mail.skipped,
        "background scheduler tick completed"
    );
    Ok(())
}

async fn run_minutely_scheduler_jobs(state: &AppState) -> Result<SchedulerRunStats, sqlx::Error> {
    let now_ts = Utc::now().timestamp();
    let database_backup = crate::backup_support::run_scheduled_database_backup(state, now_ts)
        .await
        .map_err(|err| sqlx::Error::Protocol(format!("database_backup failed: {err}")))?;
    let expired_online_users_reset = cleanup_expired_online_status(state, 5).await?;
    let expired_node_sessions_deleted = cleanup_expired_node_sessions(state, 10).await?;
    let traffic_reset_times_initialized = initialize_missing_user_reset_times(state, 500).await?;
    let traffic_resets_performed = reset_due_user_traffic(state, 500).await?;
    let (offline_server_node_alerts_marked, offline_server_node_alerts_sent) =
        check_server_nodes_offline(state).await?;
    let subscription_credentials_rotated =
        rotate_subscription_credentials_daily(state, Utc::now().timestamp()).await?;
    let (pending_orders_cancelled, processing_orders_completed) = check_orders(state, 500).await?;
    let stale_tickets_closed = check_tickets(state, 500).await?;
    let commissions_auto_checked = auto_check_commissions(state).await?;
    let commissions_paid = auto_pay_commissions(state, 500).await?;
    let expired_refund_votings_finalized = finalize_expired_refund_votings(state, 200).await?;
    let daily_log_cleanup = run_daily_log_cleanup(state, now_ts).await?;
    let daily_statistics = crate::stats_support::run_daily_statistics(state, now_ts).await?;
    let horizon_metrics_snapshot =
        crate::horizon_metrics_support::run_horizon_metrics_snapshot(state, now_ts)
            .await
            .map_err(|err| {
                sqlx::Error::Protocol(format!("horizon_metrics_snapshot failed: {err}"))
            })?;
    let linux_do_enabled = get_setting_bool(state, "oauth_linux_do_enable", false).await;
    let linux_do_user_sync = if linux_do_enabled {
        crate::oauth_sync_support::run_linux_do_user_sync(state, now_ts)
            .await
            .map_err(|err| sqlx::Error::Protocol(format!("linux_do_user_sync failed: {err}")))?
    } else {
        crate::oauth_sync_support::LinuxDoUserSyncStats::default()
    };
    let risk_review = crate::risk_review_support::run_scheduled_risk_review(state, now_ts)
        .await
        .map_err(|err| sqlx::Error::Protocol(format!("risk_review failed: {err}")))?;
    let send_remind_mail = crate::mail_reminder_support::run_send_remind_mail(state, now_ts)
        .await
        .map_err(|err| sqlx::Error::Protocol(format!("send_remind_mail failed: {err}")))?;
    Ok(SchedulerRunStats {
        database_backup,
        expired_online_users_reset,
        expired_node_sessions_deleted,
        traffic_reset_times_initialized,
        traffic_resets_performed,
        offline_server_node_alerts_marked,
        offline_server_node_alerts_sent,
        subscription_credentials_rotated,
        pending_orders_cancelled,
        processing_orders_completed,
        stale_tickets_closed,
        commissions_auto_checked,
        commissions_paid,
        expired_refund_votings_finalized,
        daily_log_cleanup,
        daily_statistics,
        horizon_metrics_snapshot,
        linux_do_user_sync,
        risk_review,
        send_remind_mail,
    })
}

pub(crate) async fn run_daily_log_cleanup(
    state: &AppState,
    now_ts: i64,
) -> Result<DailyLogCleanupStats, sqlx::Error> {
    let tz = chrono::FixedOffset::east_opt(8 * 3600)
        .ok_or_else(|| sqlx::Error::Protocol("invalid timezone offset".to_string()))?;
    let now = chrono::DateTime::from_timestamp(now_ts, 0)
        .ok_or_else(|| sqlx::Error::Protocol("invalid current timestamp".to_string()))?
        .with_timezone(&tz);
    let scheduled = tz
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 0, 0, 0)
        .single()
        .ok_or_else(|| sqlx::Error::Protocol("invalid log cleanup schedule".to_string()))?;
    if now < scheduled {
        return Ok(DailyLogCleanupStats::default());
    }

    let cache_key = "scheduler:reset-log:last-run-at";
    let already_ran = redis_get_string(state, cache_key)
        .await
        .map_err(sqlx::Error::Protocol)?;
    if already_ran
        .as_deref()
        .and_then(|value| value.parse::<i64>().ok())
        .map(|value| value >= scheduled.timestamp())
        .unwrap_or(false)
    {
        return Ok(DailyLogCleanupStats::default());
    }

    let stats = execute_daily_log_cleanup(state, now_ts).await?;
    redis_setex_string(state, cache_key, 172_800, &scheduled.timestamp().to_string())
        .await
        .map_err(sqlx::Error::Protocol)?;
    Ok(stats)
}

async fn execute_daily_log_cleanup(
    state: &AppState,
    now_ts: i64,
) -> Result<DailyLogCleanupStats, sqlx::Error> {
    let stat_user_deleted = delete_stat_user_before(state, now_ts - 60 * 60 * 24 * 60).await?;
    let stat_server_deleted = delete_stat_server_before(state, now_ts - 60 * 60 * 24 * 60).await?;
    let app_log_deleted = delete_app_logs_before(state, now_ts - 60 * 60 * 24 * 30).await?;

    let node_traffic_retention_days = get_setting_int(state, "node_traffic_records_retention_days", 7)
        .await
        .max(1);
    let user_traffic_usage_retention_days = get_setting_int(state, "user_traffic_usage_logs_retention_days", 7)
        .await
        .max(1);
    let tcping_samples_retention_days = get_setting_int(state, "tcping_samples_retention_days", 7)
        .await
        .max(1);
    let tcping_alerts_retention_days = get_setting_int(state, "tcping_alerts_retention_days", 7)
        .await
        .max(1);
    let audit_logs_retention_days = get_setting_int(state, "audit_logs_retention_days", 7)
        .await
        .max(1);

    let node_traffic_records_deleted =
        delete_node_traffic_records_before(state, days_ago_date_string(node_traffic_retention_days, now_ts)?).await?;
    let user_traffic_usage_logs_deleted = delete_user_traffic_usage_logs_before(
        state,
        now_ts - 60 * 60 * 24 * user_traffic_usage_retention_days,
    )
    .await?;
    let tcping_samples_deleted =
        delete_tcping_samples_before(state, now_ts - 60 * 60 * 24 * tcping_samples_retention_days).await?;
    let tcping_alerts_deleted =
        delete_tcping_alerts_before(state, now_ts - 60 * 60 * 24 * tcping_alerts_retention_days).await?;
    let audit_logs_deleted =
        delete_audit_logs_before(state, now_ts - 60 * 60 * 24 * audit_logs_retention_days).await?;

    Ok(DailyLogCleanupStats {
        stat_user_deleted,
        stat_server_deleted,
        app_log_deleted,
        node_traffic_records_deleted,
        user_traffic_usage_logs_deleted,
        tcping_samples_deleted,
        tcping_alerts_deleted,
        audit_logs_deleted,
    })
}

fn days_ago_date_string(days: i64, now_ts: i64) -> Result<String, sqlx::Error> {
    let tz = chrono::FixedOffset::east_opt(8 * 3600)
        .ok_or_else(|| sqlx::Error::Protocol("invalid timezone offset".to_string()))?;
    let now = chrono::DateTime::from_timestamp(now_ts, 0)
        .ok_or_else(|| sqlx::Error::Protocol("invalid current timestamp".to_string()))?
        .with_timezone(&tz);
    Ok((now - chrono::Duration::days(days))
        .date_naive()
        .format("%Y-%m-%d")
        .to_string())
}

async fn delete_stat_user_before(state: &AppState, cutoff_ts: i64) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM v2_stat_user WHERE record_at < ?")
        .bind(cutoff_ts)
        .execute(&state.db)
        .await?;
    Ok(result.rows_affected())
}

async fn delete_stat_server_before(state: &AppState, cutoff_ts: i64) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM v2_stat_server WHERE record_at < ?")
        .bind(cutoff_ts)
        .execute(&state.db)
        .await?;
    Ok(result.rows_affected())
}

async fn delete_app_logs_before(state: &AppState, cutoff_ts: i64) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM v2_log WHERE created_at < ?")
        .bind(cutoff_ts)
        .execute(&state.db)
        .await?;
    Ok(result.rows_affected())
}

async fn delete_node_traffic_records_before(
    state: &AppState,
    cutoff_date: String,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM node_traffic_records WHERE record_date < ?")
        .bind(cutoff_date)
        .execute(&state.db)
        .await?;
    Ok(result.rows_affected())
}

async fn delete_user_traffic_usage_logs_before(
    state: &AppState,
    cutoff_ts: i64,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM user_traffic_usage_logs WHERE recorded_at < ?")
        .bind(cutoff_ts)
        .execute(&state.db)
        .await?;
    Ok(result.rows_affected())
}

async fn delete_tcping_samples_before(state: &AppState, cutoff_ts: i64) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM tcping_samples WHERE sampled_at < ?")
        .bind(cutoff_ts)
        .execute(&state.db)
        .await?;
    Ok(result.rows_affected())
}

async fn delete_tcping_alerts_before(state: &AppState, cutoff_ts: i64) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM tcping_alerts WHERE triggered_at < ?")
        .bind(cutoff_ts)
        .execute(&state.db)
        .await?;
    Ok(result.rows_affected())
}

async fn delete_audit_logs_before(state: &AppState, cutoff_ts: i64) -> Result<u64, sqlx::Error> {
    let cutoff = chrono::DateTime::from_timestamp(cutoff_ts, 0)
        .ok_or_else(|| sqlx::Error::Protocol("invalid audit log cleanup cutoff".to_string()))?
        .naive_utc();
    let result = sqlx::query("DELETE FROM audit_logs WHERE created_at < ?")
        .bind(cutoff)
        .execute(&state.db)
        .await?;
    Ok(result.rows_affected())
}
