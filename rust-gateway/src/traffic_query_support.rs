use crate::*;
use sqlx::{MySql, QueryBuilder, Row};
use std::collections::HashMap;

pub(crate) async fn load_user_today_node_traffic_records(
    state: &AppState,
    user_id: i64,
    node_ids: &[u64],
    record_date: &str,
) -> Result<HashMap<u64, (i64, i64)>, sqlx::Error> {
    if node_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut builder = QueryBuilder::<MySql>::new(
        "SELECT node_id,
                CAST(upload_traffic AS SIGNED) AS upload_traffic,
                CAST(download_traffic AS SIGNED) AS download_traffic
         FROM node_traffic_records
         WHERE user_id = ",
    );
    builder
        .push_bind(user_id)
        .push(" AND node_id IN (");
    {
        let mut separated = builder.separated(", ");
        for node_id in node_ids {
            separated.push_bind(*node_id);
        }
    }
    builder.push(") AND record_date = ").push_bind(record_date);

    let rows = builder.build().fetch_all(&state.db).await?;
    let mut map = HashMap::new();
    for row in rows {
        let node_id: u64 = row.try_get("node_id").unwrap_or(0);
        let upload: i64 = row.try_get("upload_traffic").unwrap_or(0);
        let download: i64 = row.try_get("download_traffic").unwrap_or(0);
        map.insert(node_id, (upload, download));
    }
    Ok(map)
}

pub(crate) async fn load_node_traffic_user_profiles(
    state: &AppState,
    user_ids: &[i64],
) -> Result<Vec<NodeTrafficUserProfileRow>, sqlx::Error> {
    if user_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut builder = QueryBuilder::<MySql>::new(
        "SELECT id, email, linux_do_name, trust_level, is_silenced, banned
         FROM v2_user
         WHERE id IN (",
    );
    {
        let mut separated = builder.separated(", ");
        for user_id in user_ids {
            separated.push_bind(*user_id);
        }
    }
    builder.push(")");
    builder.build_query_as::<NodeTrafficUserProfileRow>().fetch_all(&state.db).await
}

pub(crate) async fn load_tcping_agents_for_user(
    state: &AppState,
    user_id: i64,
) -> Result<Vec<TcpingAgentRow>, sqlx::Error> {
    sqlx::query_as::<_, TcpingAgentRow>(
        "SELECT id, user_id, name, location_code, location_name, location_province, token,
                is_enabled, last_heartbeat_at, last_sync_at, created_at, updated_at
         FROM tcping_agents
         WHERE user_id = ?
         ORDER BY id DESC",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
}

pub(crate) async fn load_tcping_agent_for_user(
    state: &AppState,
    agent_id: u64,
    user_id: i64,
) -> Result<Option<TcpingAgentRow>, sqlx::Error> {
    sqlx::query_as::<_, TcpingAgentRow>(
        "SELECT id, user_id, name, location_code, location_name, location_province, token,
                is_enabled, last_heartbeat_at, last_sync_at, created_at, updated_at
         FROM tcping_agents
         WHERE id = ? AND user_id = ?
         LIMIT 1",
    )
    .bind(agent_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn load_tcping_agents_by_ids(
    state: &AppState,
    agent_ids: &[u64],
) -> Result<Vec<TcpingAgentRow>, sqlx::Error> {
    if agent_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut builder = QueryBuilder::<MySql>::new(
        "SELECT id, user_id, name, location_code, location_name, location_province, token,
                is_enabled, last_heartbeat_at, last_sync_at, created_at, updated_at
         FROM tcping_agents
         WHERE id IN (",
    );
    {
        let mut separated = builder.separated(", ");
        for agent_id in agent_ids {
            separated.push_bind(*agent_id);
        }
    }
    builder.push(") ORDER BY id");
    builder.build_query_as::<TcpingAgentRow>().fetch_all(&state.db).await
}

pub(crate) async fn load_node_users_traffic_rows(
    state: &AppState,
    node_id: u64,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<NodeUserTrafficRow>, sqlx::Error> {
    sqlx::query_as::<_, NodeUserTrafficRow>(
        "SELECT user_id,
                CAST(COALESCE(SUM(upload_traffic), 0) AS SIGNED) AS upload,
                CAST(COALESCE(SUM(download_traffic), 0) AS SIGNED) AS download
         FROM node_traffic_records
         WHERE node_id = ?
           AND record_date BETWEEN ? AND ?
         GROUP BY user_id
         ORDER BY (SUM(upload_traffic) + SUM(download_traffic)) DESC
         LIMIT 5000",
    )
    .bind(node_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(&state.db)
    .await
}

pub(crate) async fn load_node_traffic_stats_summary(
    state: &AppState,
    node_id: u64,
    start_date: &str,
    end_date: &str,
) -> Result<Value, sqlx::Error> {
    let row = sqlx::query(
        "SELECT
            CAST(COALESCE(SUM(upload_traffic), 0) AS SIGNED) AS total_upload,
            CAST(COALESCE(SUM(download_traffic), 0) AS SIGNED) AS total_download,
            COUNT(DISTINCT user_id) AS unique_users,
            COUNT(*) AS active_days
         FROM node_traffic_records
         WHERE node_id = ?
           AND record_date BETWEEN ? AND ?",
    )
    .bind(node_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_one(&state.db)
    .await?;

    let total_upload: i64 = row.try_get("total_upload").unwrap_or(0);
    let total_download: i64 = row.try_get("total_download").unwrap_or(0);
    let unique_users: i64 = row.try_get("unique_users").unwrap_or(0);
    let active_days: i64 = row.try_get("active_days").unwrap_or(0);

    Ok(json!({
        "total_upload": total_upload,
        "total_download": total_download,
        "total_traffic": total_upload + total_download,
        "unique_users": unique_users,
        "active_days": active_days,
        "range": {
            "start": start_date,
            "end": end_date,
        }
    }))
}

pub(crate) async fn load_user_traffic_usage_logs(
    state: &AppState,
    user_id: i64,
    start_at: i64,
    limit: i64,
) -> Result<Vec<UserTrafficUsageLogRow>, sqlx::Error> {
    sqlx::query_as::<_, UserTrafficUsageLogRow>(
        "SELECT l.id, l.user_id, l.node_id, CAST(l.raw_traffic_kb AS SIGNED) AS raw_traffic_kb,
                CAST(l.billed_traffic_kb AS SIGNED) AS billed_traffic_kb,
                CAST(l.multiplier_snapshot AS CHAR) AS multiplier_snapshot,
                l.source, l.recorded_at,
                n.name AS node_name, n.protocol AS node_protocol, n.location_name AS node_location_name
         FROM user_traffic_usage_logs l
         LEFT JOIN server_nodes n ON n.id = l.node_id
         WHERE l.user_id = ?
           AND l.recorded_at >= ?
         ORDER BY l.recorded_at DESC
         LIMIT ?",
    )
    .bind(user_id)
    .bind(start_at)
    .bind(limit)
    .fetch_all(&state.db)
    .await
}

pub(crate) async fn load_tcping_node_overview_row(
    state: &AppState,
    node_id: u64,
) -> Result<Option<TcpingNodeOverviewRow>, sqlx::Error> {
    sqlx::query_as::<_, TcpingNodeOverviewRow>(
        "SELECT id, user_id, name, host, port, protocol, location_name, status,
                tcping_enabled, tcping_host, tcping_port, tcping_interval_seconds,
                tcping_timeout_ms, tcping_alert_after_seconds, tcping_recover_after_seconds,
                tcping_last_status, tcping_last_latency_ms, tcping_last_error, tcping_last_sampled_at
         FROM server_nodes
         WHERE id = ?
         LIMIT 1",
    )
    .bind(node_id)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn load_tcping_samples_for_node_since(
    state: &AppState,
    node_id: u64,
    start_at: i64,
) -> Result<Vec<TcpingSampleRow>, sqlx::Error> {
    sqlx::query_as::<_, TcpingSampleRow>(
        "SELECT id, node_id, agent_id, is_reachable, latency_ms, is_timeout, error_message, sampled_at
         FROM tcping_samples
         WHERE node_id = ? AND sampled_at >= ?
         ORDER BY sampled_at",
    )
    .bind(node_id)
    .bind(start_at)
    .fetch_all(&state.db)
    .await
}

pub(crate) async fn load_tcping_alerts_for_node(
    state: &AppState,
    node_id: u64,
) -> Result<Vec<TcpingAlertRow>, sqlx::Error> {
    sqlx::query_as::<_, TcpingAlertRow>(
        "SELECT id, node_id, user_id, status, started_at, triggered_at, recovered_at, latest_error
         FROM tcping_alerts
         WHERE node_id = ?
         ORDER BY triggered_at DESC
         LIMIT 20",
    )
    .bind(node_id)
    .fetch_all(&state.db)
    .await
}

pub(crate) async fn load_owned_server_nodes(
    state: &AppState,
    user_id: i64,
) -> Result<Vec<ServerNodeOwnerRow>, sqlx::Error> {
    sqlx::query_as::<_, ServerNodeOwnerRow>(
        "SELECT id, user_id, name, host, port, service_port, protocol, location_code, location_name,
                settings, traffic_limit, traffic_used, CAST(traffic_multiplier AS CHAR) AS traffic_multiplier,
                access_control, status, v2bx_node_id, v2bx_config, v2bx_token, device_limit,
                connection_limit, speed_limit_up, speed_limit_down, cross_node_ip_limit, concurrent_ip_limit,
                tcping_enabled, tcping_host, tcping_port, tcping_interval_seconds, tcping_timeout_ms,
                tcping_alert_after_seconds, tcping_recover_after_seconds, tcping_last_status,
                tcping_last_latency_ms, tcping_last_error, tcping_last_sampled_at, created_at, updated_at
         FROM server_nodes
         WHERE user_id = ?
         ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await
}

pub(crate) async fn load_owned_server_node(
    state: &AppState,
    node_id: u64,
    user_id: i64,
) -> Result<Option<ServerNodeOwnerRow>, sqlx::Error> {
    sqlx::query_as::<_, ServerNodeOwnerRow>(
        "SELECT id, user_id, name, host, port, service_port, protocol, location_code, location_name,
                settings, traffic_limit, traffic_used, CAST(traffic_multiplier AS CHAR) AS traffic_multiplier,
                access_control, status, v2bx_node_id, v2bx_config, v2bx_token, device_limit,
                connection_limit, speed_limit_up, speed_limit_down, cross_node_ip_limit, concurrent_ip_limit,
                tcping_enabled, tcping_host, tcping_port, tcping_interval_seconds, tcping_timeout_ms,
                tcping_alert_after_seconds, tcping_recover_after_seconds, tcping_last_status,
                tcping_last_latency_ms, tcping_last_error, tcping_last_sampled_at, created_at, updated_at
         FROM server_nodes
         WHERE id = ? AND user_id = ?
         LIMIT 1",
    )
    .bind(node_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn load_online_user_count_for_node(
    state: &AppState,
    node_id: u64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT COUNT(DISTINCT user_id)
         FROM user_online_sessions
         WHERE node_id = ? AND last_activity >= DATE_SUB(NOW(), INTERVAL 5 MINUTE)",
    )
    .bind(node_id)
    .fetch_one(&state.db)
    .await
}
