use crate::*;

pub async fn show(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_show_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_show_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let now = Utc::now().timestamp();
    let today = chrono::DateTime::from_timestamp(now, 0)
        .map(|dt| dt.date_naive().to_string())
        .unwrap_or_else(|| chrono::Utc::now().date_naive().to_string());
    let window_start = chrono::DateTime::from_timestamp(now, 0)
        .map(|dt| dt.date_naive() - chrono::Days::new(6))
        .map(|date| date.to_string())
        .unwrap_or_else(|| chrono::Utc::now().date_naive().to_string());

    let node_rows = load_command_center_nodes(state).await.map_err(internal_error)?;
    let status_cache = state.load_status_cache.read().clone();
    let online_sessions = load_command_center_online_sessions(state, now).await.map_err(internal_error)?;
    let weekly_traffic = load_command_center_weekly_node_traffic(state, &window_start, &today).await.map_err(internal_error)?;
    let active_alerts = load_command_center_active_tcping_alerts(state).await.map_err(internal_error)?;
    let alert_count_map = active_alerts
        .iter()
        .fold(HashMap::<u64, i64>::new(), |mut acc, row| {
            *acc.entry(row.node_id).or_insert(0) += 1;
            acc
        });
    let latest_alert_map = active_alerts
        .iter()
        .fold(HashMap::<u64, TcpingAlertRow>::new(), |mut acc, row| {
            acc.entry(row.node_id).or_insert_with(|| row.clone());
            acc
        });

    let node_snapshots = build_command_center_node_snapshots(
        &node_rows,
        &status_cache,
        &online_sessions,
        &weekly_traffic,
        &latest_alert_map,
        &alert_count_map,
        now,
    );
    let watch_node_ids = node_snapshots.iter().take(8).filter_map(|node| node.get("id").and_then(Value::as_u64)).collect::<Vec<_>>();
    let tcping_samples = load_command_center_watch_samples(state, &watch_node_ids, now - 86_400).await.map_err(internal_error)?;
    let node_watchlist = build_command_center_watchlist(&node_snapshots, &tcping_samples);

    let today_traffic = load_command_center_today_traffic(state, &today).await.map_err(internal_error)?;
    let active_subscriptions: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_plan_subscriptions WHERE status = 1 AND expired_at > ?"
    )
    .bind(now)
    .fetch_one(&state.db)
    .await
    .map_err(internal_error)?;
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_user")
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    let live_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_user WHERE t >= ?")
        .bind(now - 300)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    let completed_orders_24h: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_order WHERE status = 3 AND paid_at >= ?")
        .bind(now - 86_400)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    let revenue_24h_amount: i64 = sqlx::query_scalar("SELECT CAST(COALESCE(SUM(total_amount),0) AS SIGNED) FROM v2_order WHERE status = 3 AND paid_at >= ?")
        .bind(now - 86_400)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    let open_tickets_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_ticket WHERE status = 0")
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    let pending_refunds_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM order_refund_requests WHERE status IN ('pending','voting')")
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    let tcping_agents_total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tcping_agents")
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    let tcping_agents_online: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tcping_agents WHERE last_heartbeat_at IS NOT NULL AND last_heartbeat_at >= ?")
        .bind(now - 180)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;

    let tickets = build_command_center_open_tickets(state).await.map_err(internal_error)?;
    let refunds = build_command_center_pending_refunds(state).await.map_err(internal_error)?;
    let tcping_agents = build_command_center_tcping_agents(state, now).await.map_err(internal_error)?;
    let tcping_alerts = build_command_center_tcping_alert_stream(state, now).await.map_err(internal_error)?;
    let audit_stream = build_command_center_audit_stream(state).await.map_err(internal_error)?;
    let top_users = build_command_center_top_users(state, now, &window_start, &today).await.map_err(internal_error)?;
    let protocol_distribution = build_command_center_protocol_distribution(&node_snapshots);
    let region_distribution = build_command_center_region_distribution(&node_snapshots);
    let system = build_command_center_system_status(state, now).await.map_err(internal_error)?;
    let traffic_trend = build_command_center_traffic_trend(state, now).await.map_err(internal_error)?;
    let (throughput_upload_bps, throughput_download_bps) = resolve_command_center_throughput(state, now).await.map_err(internal_error)?;

    let overview = json!({
        "total_users": total_users,
        "live_users": live_users,
        "active_subscriptions": active_subscriptions,
        "total_nodes": node_rows.len(),
        "online_nodes": node_snapshots.iter().filter(|node| node.get("online_status").and_then(Value::as_str) == Some("online")).count(),
        "maintenance_nodes": node_snapshots.iter().filter(|node| node.get("status").and_then(Value::as_str) == Some("maintenance")).count(),
        "traffic_hot_nodes": node_snapshots.iter().filter(|node| node.get("traffic_usage_percentage").and_then(Value::as_f64).unwrap_or(0.0) >= 80.0).count(),
        "tcping_enabled_nodes": node_snapshots.iter().filter(|node| node.get("tcping_enabled").and_then(Value::as_bool) == Some(true)).count(),
        "tcping_alerts_active": active_alerts.len(),
        "tcping_agents_total": tcping_agents_total,
        "tcping_agents_online": tcping_agents_online,
        "open_tickets": open_tickets_count,
        "pending_refunds": pending_refunds_count,
        "completed_orders_24h": completed_orders_24h,
        "revenue_24h_amount": revenue_24h_amount,
        "traffic_today_kb": today_traffic.get("total_kb").and_then(Value::as_i64).unwrap_or(0),
        "traffic_today_unique_users": today_traffic.get("unique_users").and_then(Value::as_i64).unwrap_or(0),
        "throughput_upload_bps": throughput_upload_bps,
        "throughput_download_bps": throughput_download_bps
    });

    Ok(json_value_response(success_response_payload(json!({
        "generated_at": now,
        "refresh_interval_seconds": 20,
        "overview": overview,
        "system": system,
        "traffic_trend": traffic_trend,
        "node_watchlist": node_watchlist,
        "hot_nodes": node_watchlist,
        "protocol_distribution": protocol_distribution,
        "region_distribution": region_distribution,
        "tickets": tickets,
        "refunds": refunds,
        "tcping_agents": tcping_agents,
        "tcping_alerts": tcping_alerts,
        "audit_stream": audit_stream,
        "top_users": top_users
    }))))
}
