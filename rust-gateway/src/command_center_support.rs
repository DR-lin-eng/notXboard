use crate::*;

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct CommandCenterNodeTrafficAggRow {
    pub(crate) node_id: u64,
    pub(crate) traffic_kb: i64,
    pub(crate) unique_users: i64,
}

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct CommandCenterOnlineSessionAggRow {
    pub(crate) node_id: u64,
    pub(crate) active_users: i64,
    pub(crate) active_connections: i64,
}

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct CommandCenterTopUserTrafficRow {
    pub(crate) user_id: i64,
    pub(crate) traffic_kb: i64,
}

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct CommandCenterUserIdentityRow {
    pub(crate) id: i64,
    pub(crate) email: String,
    pub(crate) linux_do_name: Option<String>,
    pub(crate) linux_do_username: Option<String>,
    pub(crate) trust_level: Option<i64>,
    pub(crate) last_login_at: Option<i64>,
}

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct CommandCenterSubscriptionInfoRow {
    pub(crate) user_id: i64,
    pub(crate) expired_at: i64,
    pub(crate) plan_name: Option<String>,
}

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct CommandCenterLogLevelAggRow {
    pub(crate) level: String,
    pub(crate) count: i64,
}

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct CommandCenterTrafficTrendRow {
    pub(crate) record_date: chrono::NaiveDate,
    pub(crate) upload_kb: i64,
    pub(crate) download_kb: i64,
    pub(crate) unique_users: i64,
}

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct CommandCenterNodeRow {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) host: String,
    pub(crate) port: i64,
    pub(crate) protocol: String,
    pub(crate) location_code: Option<String>,
    pub(crate) location_name: Option<String>,
    pub(crate) traffic_limit: u64,
    pub(crate) traffic_used: u64,
    pub(crate) traffic_multiplier: String,
    pub(crate) status: String,
    pub(crate) tcping_port: Option<i64>,
    pub(crate) tcping_last_status: Option<String>,
    pub(crate) tcping_last_latency_ms: Option<i64>,
    pub(crate) tcping_last_error: Option<String>,
    pub(crate) tcping_last_sampled_at: Option<i64>,
    pub(crate) owner_email: Option<String>,
    pub(crate) owner_linux_do_username: Option<String>,
    pub(crate) owner_linux_do_name: Option<String>,
}

fn command_center_node_traffic_usage_percentage(node: &CommandCenterNodeRow) -> f64 {
    if node.traffic_limit == 0 {
        0.0
    } else {
        ((node.traffic_used as f64 / node.traffic_limit as f64) * 100.0).min(100.0)
    }
}

pub(crate) async fn load_command_center_nodes(
    state: &AppState,
) -> Result<Vec<CommandCenterNodeRow>, sqlx::Error> {
    sqlx::query_as::<_, CommandCenterNodeRow>(
        "SELECT sn.id, sn.name, sn.host, sn.port, sn.protocol, sn.location_code, sn.location_name,
                sn.traffic_limit, sn.traffic_used, CAST(sn.traffic_multiplier AS CHAR) AS traffic_multiplier,
                sn.status, sn.tcping_port, sn.tcping_last_status, sn.tcping_last_latency_ms, sn.tcping_last_error,
                sn.tcping_last_sampled_at,
                owner.email AS owner_email, owner.linux_do_username AS owner_linux_do_username, owner.linux_do_name AS owner_linux_do_name
         FROM server_nodes sn
         LEFT JOIN v2_user owner ON owner.id = sn.user_id
         ORDER BY id"
    )
    .fetch_all(&state.db)
    .await
}

pub(crate) async fn load_command_center_online_sessions(
    state: &AppState,
    now_ts: i64,
) -> Result<HashMap<u64, CommandCenterOnlineSessionAggRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, CommandCenterOnlineSessionAggRow>(
        "SELECT node_id,
                COUNT(DISTINCT user_id) AS active_users,
                CAST(COALESCE(SUM(connection_count),0) AS SIGNED) AS active_connections
         FROM user_online_sessions
         WHERE last_activity >= ?
         GROUP BY node_id"
    )
    .bind(now_ts - 300)
    .fetch_all(&state.db)
    .await?;
    Ok(rows.into_iter().map(|row| (row.node_id, row)).collect())
}

pub(crate) async fn load_command_center_weekly_node_traffic(
    state: &AppState,
    window_start: &str,
    today: &str,
) -> Result<HashMap<u64, CommandCenterNodeTrafficAggRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, CommandCenterNodeTrafficAggRow>(
        "SELECT node_id,
                CAST(COALESCE(SUM(upload_traffic + download_traffic),0) AS SIGNED) AS traffic_kb,
                COUNT(DISTINCT user_id) AS unique_users
         FROM node_traffic_records
         WHERE record_date BETWEEN ? AND ?
         GROUP BY node_id"
    )
    .bind(window_start)
    .bind(today)
    .fetch_all(&state.db)
    .await?;
    Ok(rows.into_iter().map(|row| (row.node_id, row)).collect())
}

pub(crate) async fn load_command_center_today_traffic(
    state: &AppState,
    today: &str,
) -> Result<Value, sqlx::Error> {
    let row = sqlx::query(
        "SELECT CAST(COALESCE(SUM(upload_traffic),0) AS SIGNED) AS upload_kb,
                CAST(COALESCE(SUM(download_traffic),0) AS SIGNED) AS download_kb,
                CAST(COALESCE(SUM(upload_traffic + download_traffic),0) AS SIGNED) AS total_kb,
                COUNT(DISTINCT user_id) AS unique_users
         FROM node_traffic_records
         WHERE record_date = ?"
    )
    .bind(today)
    .fetch_one(&state.db)
    .await?;
    Ok(json!({
        "upload_kb": row.try_get::<i64, _>("upload_kb").unwrap_or(0),
        "download_kb": row.try_get::<i64, _>("download_kb").unwrap_or(0),
        "total_kb": row.try_get::<i64, _>("total_kb").unwrap_or(0),
        "unique_users": row.try_get::<i64, _>("unique_users").unwrap_or(0)
    }))
}

pub(crate) async fn load_command_center_active_tcping_alerts(
    state: &AppState,
) -> Result<Vec<TcpingAlertRow>, sqlx::Error> {
    sqlx::query_as::<_, TcpingAlertRow>(
        "SELECT id, node_id, user_id, status, started_at, triggered_at, recovered_at, latest_error
         FROM tcping_alerts
         WHERE status = 'active'
         ORDER BY triggered_at DESC"
    )
    .fetch_all(&state.db)
    .await
}

pub(crate) async fn load_command_center_watch_samples(
    state: &AppState,
    node_ids: &[u64],
    start_at: i64,
) -> Result<HashMap<u64, Vec<TcpingSampleRow>>, sqlx::Error> {
    if node_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let placeholders = vec!["?"; node_ids.len()].join(",");
    let sql = format!(
        "SELECT id, node_id, agent_id, is_reachable, latency_ms, is_timeout, error_message, sampled_at
         FROM tcping_samples
         WHERE node_id IN ({}) AND sampled_at >= ?
         ORDER BY sampled_at DESC",
        placeholders
    );
    let mut query = sqlx::query_as::<_, TcpingSampleRow>(&sql);
    for node_id in node_ids {
        query = query.bind(*node_id);
    }
    query = query.bind(start_at);
    let rows = query.fetch_all(&state.db).await?;
    let mut map = HashMap::<u64, Vec<TcpingSampleRow>>::new();
    for row in rows {
        map.entry(row.node_id).or_default().push(row);
    }
    for values in map.values_mut() {
        values.sort_by_key(|row| row.sampled_at);
        if values.len() > 18 {
            let drain_count = values.len() - 18;
            values.drain(0..drain_count);
        }
    }
    Ok(map)
}

pub(crate) async fn build_command_center_open_tickets(
    state: &AppState,
) -> Result<Vec<Value>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT CAST(t.id AS SIGNED) AS id, t.subject, t.level, t.status,
                t.created_at AS created_at, t.updated_at AS updated_at,
                u.email AS user_email, COALESCE(u.linux_do_name, u.linux_do_username, u.email) AS user_name,
                n.name AS node_name, a.email AS assigned_admin_email
         FROM v2_ticket t
         LEFT JOIN v2_user u ON u.id = t.user_id
         LEFT JOIN server_nodes n ON n.id = t.node_id
         LEFT JOIN v2_user a ON a.id = t.assigned_admin_user_id
         WHERE t.status = 0
         ORDER BY t.updated_at DESC
         LIMIT 6"
    )
    .fetch_all(&state.db)
    .await?;
    Ok(rows.into_iter().map(|row| json!({
        "id": row.try_get::<i64, _>("id").unwrap_or_default(),
        "subject": row.try_get::<Option<String>, _>("subject").ok().flatten().unwrap_or_else(|| "-".to_string()),
        "level": row.try_get::<Option<i64>, _>("level").ok().flatten().unwrap_or_default(),
        "status": row.try_get::<Option<i64>, _>("status").ok().flatten().unwrap_or_default(),
        "user_email": row.try_get::<Option<String>, _>("user_email").ok().flatten().unwrap_or_else(|| "-".to_string()),
        "user_name": row.try_get::<Option<String>, _>("user_name").ok().flatten().unwrap_or_else(|| "-".to_string()),
        "node_name": row.try_get::<Option<String>, _>("node_name").ok().flatten().unwrap_or_else(|| "-".to_string()),
        "assigned_admin_email": row.try_get::<Option<String>, _>("assigned_admin_email").ok().flatten().unwrap_or_else(|| "-".to_string()),
        "updated_at": row.try_get::<Option<i64>, _>("updated_at").ok().flatten(),
        "created_at": row.try_get::<Option<i64>, _>("created_at").ok().flatten()
    })).collect())
}

pub(crate) async fn build_command_center_pending_refunds(
    state: &AppState,
) -> Result<Vec<Value>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT CAST(rr.id AS SIGNED) AS id, rr.trade_no, rr.status, rr.gateway_amount, rr.refund_amount,
                UNIX_TIMESTAMP(rr.created_at) AS created_at, UNIX_TIMESTAMP(rr.updated_at) AS updated_at,
                u.email AS user_email, COALESCE(u.linux_do_name, u.linux_do_username, u.email) AS user_name,
                p.name AS plan_name, a.email AS assigned_admin_email
         FROM order_refund_requests rr
         LEFT JOIN v2_user u ON u.id = rr.user_id
         LEFT JOIN v2_plan p ON p.id = rr.plan_id
         LEFT JOIN v2_user a ON a.id = rr.assigned_admin_user_id
         WHERE rr.status IN ('pending', 'voting')
         ORDER BY rr.updated_at DESC
         LIMIT 6"
    )
    .fetch_all(&state.db)
    .await?;
    Ok(rows.into_iter().map(|row| json!({
        "id": row.try_get::<i64, _>("id").unwrap_or_default(),
        "trade_no": row.try_get::<Option<String>, _>("trade_no").ok().flatten().unwrap_or_else(|| "-".to_string()),
        "status": row.try_get::<Option<String>, _>("status").ok().flatten().unwrap_or_else(|| "pending".to_string()),
        "user_email": row.try_get::<Option<String>, _>("user_email").ok().flatten().unwrap_or_else(|| "-".to_string()),
        "user_name": row.try_get::<Option<String>, _>("user_name").ok().flatten().unwrap_or_else(|| "-".to_string()),
        "plan_name": row.try_get::<Option<String>, _>("plan_name").ok().flatten().unwrap_or_else(|| "-".to_string()),
        "assigned_admin_email": row.try_get::<Option<String>, _>("assigned_admin_email").ok().flatten().unwrap_or_else(|| "-".to_string()),
        "gateway_amount": row.try_get::<Option<i64>, _>("gateway_amount").ok().flatten().unwrap_or_default(),
        "refund_amount": row.try_get::<Option<i64>, _>("refund_amount").ok().flatten().unwrap_or_default(),
        "updated_at": row.try_get::<Option<i64>, _>("updated_at").ok().flatten(),
        "created_at": row.try_get::<Option<i64>, _>("created_at").ok().flatten()
    })).collect())
}

pub(crate) async fn build_command_center_tcping_agents(
    state: &AppState,
    now_ts: i64,
) -> Result<Vec<Value>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT CAST(a.id AS SIGNED) AS id, a.name, a.last_heartbeat_at, a.last_sync_at,
                u.email AS owner_email,
                COALESCE(u.linux_do_name, u.linux_do_username, u.email) AS owner_name
         FROM tcping_agents a
         LEFT JOIN v2_user u ON u.id = a.user_id
         ORDER BY COALESCE(a.last_heartbeat_at, 0) DESC
         LIMIT 6"
    )
    .fetch_all(&state.db)
    .await?;
    Ok(rows.into_iter().map(|row| {
        let last_heartbeat_at = row.try_get::<Option<i64>, _>("last_heartbeat_at").ok().flatten();
        json!({
            "id": row.try_get::<i64, _>("id").unwrap_or_default(),
            "name": row.try_get::<Option<String>, _>("name").ok().flatten().unwrap_or_else(|| "-".to_string()),
            "is_enabled": true,
            "is_online": last_heartbeat_at.map(|v| v >= now_ts - 180).unwrap_or(false),
            "owner_email": row.try_get::<Option<String>, _>("owner_email").ok().flatten().unwrap_or_else(|| "-".to_string()),
            "owner_name": row.try_get::<Option<String>, _>("owner_name").ok().flatten().unwrap_or_else(|| "-".to_string()),
            "last_heartbeat_at": last_heartbeat_at,
            "last_sync_at": row.try_get::<Option<i64>, _>("last_sync_at").ok().flatten()
        })
    }).collect())
}

pub(crate) async fn build_command_center_audit_stream(
    state: &AppState,
) -> Result<Vec<Value>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT CAST(l.id AS SIGNED) AS id, l.action_taken, l.ip_address, l.target_domain, l.target_protocol,
                UNIX_TIMESTAMP(l.created_at) AS created_at,
                u.email AS user_email, COALESCE(u.linux_do_name, u.linux_do_username, u.email) AS user_name,
                n.name AS node_name, n.protocol AS node_protocol, n.location_code AS node_location_code, n.location_name AS node_location_name
         FROM audit_logs l
         LEFT JOIN v2_user u ON u.id = l.user_id
         LEFT JOIN server_nodes n ON n.id = l.node_id
         ORDER BY l.created_at DESC
         LIMIT 12"
    )
    .fetch_all(&state.db)
    .await?;
    Ok(rows.into_iter().map(|row| json!({
        "id": row.try_get::<i64, _>("id").unwrap_or_default(),
        "action_taken": row.try_get::<Option<String>, _>("action_taken").ok().flatten().unwrap_or_else(|| "logged".to_string()),
        "user_email": row.try_get::<Option<String>, _>("user_email").ok().flatten().unwrap_or_else(|| "-".to_string()),
        "user_name": row.try_get::<Option<String>, _>("user_name").ok().flatten().unwrap_or_else(|| "-".to_string()),
        "node_name": row.try_get::<Option<String>, _>("node_name").ok().flatten().unwrap_or_else(|| "-".to_string()),
        "node_protocol": row.try_get::<Option<String>, _>("node_protocol").ok().flatten().unwrap_or_else(|| "-".to_string()),
        "node_location_code": row.try_get::<Option<String>, _>("node_location_code").ok().flatten().unwrap_or_default(),
        "node_location_name": row.try_get::<Option<String>, _>("node_location_name").ok().flatten().unwrap_or_else(|| "-".to_string()),
        "ip_address": row.try_get::<Option<String>, _>("ip_address").ok().flatten().unwrap_or_else(|| "-".to_string()),
        "target_domain": row.try_get::<Option<String>, _>("target_domain").ok().flatten(),
        "target_protocol": row.try_get::<Option<String>, _>("target_protocol").ok().flatten(),
        "created_at": row.try_get::<Option<i64>, _>("created_at").ok().flatten()
    })).collect())
}

pub(crate) async fn build_command_center_tcping_alert_stream(
    state: &AppState,
    now_ts: i64,
) -> Result<Vec<Value>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT CAST(a.id AS SIGNED) AS id, a.status, a.triggered_at, a.latest_error,
                u.email AS user_email, COALESCE(u.linux_do_name, u.linux_do_username, u.email) AS user_name,
                n.name AS node_name, n.protocol AS node_protocol, n.location_code AS node_location_code, n.location_name AS node_location_name
         FROM tcping_alerts a
         LEFT JOIN v2_user u ON u.id = a.user_id
         LEFT JOIN server_nodes n ON n.id = a.node_id
         WHERE a.status = 'active'
         ORDER BY a.triggered_at DESC
         LIMIT 8"
    )
    .fetch_all(&state.db)
    .await?;
    Ok(rows.into_iter().map(|row| {
        let triggered_at = row.try_get::<Option<i64>, _>("triggered_at").ok().flatten().unwrap_or(0);
        json!({
            "id": row.try_get::<i64, _>("id").unwrap_or_default(),
            "status": row.try_get::<Option<String>, _>("status").ok().flatten().unwrap_or_else(|| "active".to_string()),
            "user_email": row.try_get::<Option<String>, _>("user_email").ok().flatten().unwrap_or_else(|| "-".to_string()),
            "user_name": row.try_get::<Option<String>, _>("user_name").ok().flatten().unwrap_or_else(|| "-".to_string()),
            "node_name": row.try_get::<Option<String>, _>("node_name").ok().flatten().unwrap_or_else(|| "-".to_string()),
            "node_protocol": row.try_get::<Option<String>, _>("node_protocol").ok().flatten().unwrap_or_else(|| "-".to_string()),
            "node_location_code": row.try_get::<Option<String>, _>("node_location_code").ok().flatten().unwrap_or_default(),
            "node_location_name": row.try_get::<Option<String>, _>("node_location_name").ok().flatten().unwrap_or_else(|| "-".to_string()),
            "triggered_at": triggered_at,
            "duration_seconds": (now_ts - triggered_at).max(0),
            "latest_error": row.try_get::<Option<String>, _>("latest_error").ok().flatten().unwrap_or_default()
        })
    }).collect())
}

pub(crate) fn build_command_center_protocol_distribution(node_watchlist: &[Value]) -> Vec<Value> {
    let protocol_labels = server_node_protocol_labels();
    let mut map = HashMap::<String, (i64, i64, i64)>::new();
    for node in node_watchlist {
        let protocol = node.get("protocol").and_then(Value::as_str).unwrap_or("").to_string();
        let entry = map.entry(protocol).or_insert((0, 0, 0));
        entry.0 += 1;
        if node.get("online_status").and_then(Value::as_str) == Some("online") {
            entry.1 += 1;
        }
        if node.get("tcping_enabled").and_then(Value::as_bool) == Some(true) {
            entry.2 += 1;
        }
    }
    let mut rows = map.into_iter().map(|(protocol, (total, online, tcping_enabled))| {
        let normalized = protocol.to_ascii_lowercase();
        let label = protocol_labels
            .get(normalized.as_str())
            .copied()
            .map(str::to_string)
            .unwrap_or_else(|| protocol.to_ascii_uppercase());
        json!({
            "protocol": protocol.clone(),
            "label": label,
            "total": total,
            "online": online,
            "tcping_enabled": tcping_enabled
        })
    }).collect::<Vec<_>>();
    rows.sort_by(|a, b| b.get("total").and_then(Value::as_i64).unwrap_or(0).cmp(&a.get("total").and_then(Value::as_i64).unwrap_or(0)));
    rows
}

pub(crate) fn build_command_center_region_distribution(node_watchlist: &[Value]) -> Vec<Value> {
    let mut map = HashMap::<String, (String, i64, i64)>::new();
    for node in node_watchlist {
        let code = node.get("location_code").and_then(Value::as_str).unwrap_or("").to_string();
        let name = node.get("location_name").and_then(Value::as_str).unwrap_or("未标注地区").to_string();
        let key = if code.is_empty() { name.clone() } else { code.clone() };
        let entry = map.entry(key).or_insert((name.clone(), 0, 0));
        entry.1 += 1;
        if node.get("online_status").and_then(Value::as_str) == Some("online") {
            entry.2 += 1;
        }
    }
    let mut rows = map.into_iter().map(|(key, (name, total, online))| json!({
        "location_code": if name == "未标注地区" { "".to_string() } else { key.clone() },
        "location_name": name,
        "total": total,
        "online": online
    })).collect::<Vec<_>>();
    rows.sort_by(|a, b| b.get("total").and_then(Value::as_i64).unwrap_or(0).cmp(&a.get("total").and_then(Value::as_i64).unwrap_or(0)));
    rows.truncate(10);
    rows
}

pub(crate) async fn build_command_center_top_users(
    state: &AppState,
    now_ts: i64,
    window_start: &str,
    today: &str,
) -> Result<Vec<Value>, sqlx::Error> {
    let rows = sqlx::query_as::<_, CommandCenterTopUserTrafficRow>(
        "SELECT user_id, CAST(COALESCE(SUM(upload_traffic + download_traffic),0) AS SIGNED) AS traffic_kb
         FROM node_traffic_records
         WHERE record_date BETWEEN ? AND ?
         GROUP BY user_id
         ORDER BY traffic_kb DESC
         LIMIT 8"
    )
    .bind(window_start)
    .bind(today)
    .fetch_all(&state.db)
    .await?;

    let user_ids = rows.iter().map(|row| row.user_id).collect::<Vec<_>>();
    if user_ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = vec!["?"; user_ids.len()].join(",");
    let user_sql = format!(
        "SELECT id, email, linux_do_name, linux_do_username, trust_level, last_login_at
         FROM v2_user WHERE id IN ({})",
        placeholders
    );
    let mut user_query = sqlx::query_as::<_, CommandCenterUserIdentityRow>(&user_sql);
    for user_id in &user_ids {
        user_query = user_query.bind(*user_id);
    }
    let user_map = user_query
        .fetch_all(&state.db)
        .await?
        .into_iter()
        .map(|row| (row.id, row))
        .collect::<HashMap<_, _>>();

    let sub_sql = format!(
        "SELECT s.user_id, s.expired_at, p.name AS plan_name
         FROM user_plan_subscriptions s
         LEFT JOIN v2_plan p ON p.id = s.plan_id
         WHERE s.status = 1 AND s.expired_at > ? AND s.user_id IN ({})
         ORDER BY s.expired_at DESC",
        placeholders
    );
    let mut sub_query = sqlx::query_as::<_, CommandCenterSubscriptionInfoRow>(&sub_sql).bind(now_ts);
    for user_id in &user_ids {
        sub_query = sub_query.bind(*user_id);
    }
    let mut subscription_map = HashMap::new();
    for row in sub_query.fetch_all(&state.db).await? {
        subscription_map.entry(row.user_id).or_insert(row);
    }

    Ok(rows.into_iter().map(|row| {
        let user = user_map.get(&row.user_id);
        let subscription = subscription_map.get(&row.user_id);
        json!({
            "id": user.map(|v| v.id).unwrap_or(row.user_id),
            "email": user.map(|v| v.email.clone()).unwrap_or_else(|| format!("user-{}", row.user_id)),
            "display_name": user.map(|v| v.linux_do_name.clone().or(v.linux_do_username.clone()).unwrap_or_else(|| v.email.clone())).unwrap_or_else(|| format!("User {}", row.user_id)),
            "traffic_kb": row.traffic_kb,
            "trust_level": user.and_then(|v| v.trust_level).unwrap_or(0),
            "last_login_at": user.and_then(|v| v.last_login_at),
            "plan_name": subscription.and_then(|v| v.plan_name.clone()).unwrap_or_else(|| "无有效套餐".to_string()),
            "subscription_expired_at": subscription.map(|v| v.expired_at),
        })
    }).collect())
}

pub(crate) async fn build_command_center_system_status(
    state: &AppState,
    now_ts: i64,
) -> Result<Value, sqlx::Error> {
    let row = sqlx::query(
        "SELECT
            CAST(COALESCE(SUM(CASE WHEN level = 'INFO' THEN 1 ELSE 0 END),0) AS SIGNED) AS info_count,
            CAST(COALESCE(SUM(CASE WHEN level = 'WARNING' THEN 1 ELSE 0 END),0) AS SIGNED) AS warning_count,
            CAST(COALESCE(SUM(CASE WHEN level = 'ERROR' THEN 1 ELSE 0 END),0) AS SIGNED) AS error_count,
            CAST(COALESCE(COUNT(*),0) AS SIGNED) AS total_count,
            CAST(COALESCE(SUM(CASE WHEN level = 'ERROR' AND created_at >= ? THEN 1 ELSE 0 END),0) AS SIGNED) AS errors_last_24h,
            CAST(COALESCE(SUM(CASE WHEN level = 'WARNING' AND created_at >= ? THEN 1 ELSE 0 END),0) AS SIGNED) AS warnings_last_24h
         FROM v2_log"
    )
    .bind(now_ts - 86_400)
    .bind(now_ts - 86_400)
    .fetch_one(&state.db)
    .await?;

    let schedule_last_runtime = redis_get_string(state, "SCHEDULE_LAST_CHECK_AT")
        .await
        .ok()
        .flatten()
        .and_then(|value| value.parse::<i64>().ok());
    let schedule_ok = schedule_last_runtime.map(|value| (now_ts - value) < 120);
    let rust_async_queues =
        serde_json::to_value(state.async_queue_metrics.snapshot()).unwrap_or(Value::Null);

    Ok(json!({
        "schedule_ok": schedule_ok,
        "schedule_last_runtime": schedule_last_runtime,
        "horizon": {
            "available": false,
            "ok": Value::Null,
            "master_count": 0,
            "paused_masters": 0
        },
        "logs": {
            "info": row.try_get::<i64, _>("info_count").unwrap_or(0),
            "warning": row.try_get::<i64, _>("warning_count").unwrap_or(0),
            "error": row.try_get::<i64, _>("error_count").unwrap_or(0),
            "total": row.try_get::<i64, _>("total_count").unwrap_or(0),
            "errors_last_24h": row.try_get::<i64, _>("errors_last_24h").unwrap_or(0),
            "warnings_last_24h": row.try_get::<i64, _>("warnings_last_24h").unwrap_or(0)
        },
        "rust_async_queues": rust_async_queues
    }))
}

pub(crate) async fn build_command_center_traffic_trend(
    state: &AppState,
    now: i64,
) -> Result<Vec<Value>, sqlx::Error> {
    let start = chrono::DateTime::from_timestamp(now, 0)
        .map(|dt| (dt.date_naive() - chrono::Days::new(6)).to_string())
        .unwrap_or_else(|| chrono::Utc::now().date_naive().to_string());
    let end = chrono::DateTime::from_timestamp(now, 0)
        .map(|dt| dt.date_naive().to_string())
        .unwrap_or_else(|| chrono::Utc::now().date_naive().to_string());
    let rows = sqlx::query_as::<_, CommandCenterTrafficTrendRow>(
        "SELECT record_date,
                CAST(COALESCE(SUM(upload_traffic),0) AS SIGNED) AS upload_kb,
                CAST(COALESCE(SUM(download_traffic),0) AS SIGNED) AS download_kb,
                COUNT(DISTINCT user_id) AS unique_users
         FROM node_traffic_records
         WHERE record_date BETWEEN ? AND ?
         GROUP BY record_date
         ORDER BY record_date"
    )
    .bind(&start)
    .bind(&end)
    .fetch_all(&state.db)
    .await?;
    let row_map = rows
        .into_iter()
        .map(|row| (row.record_date.to_string(), row))
        .collect::<HashMap<_, _>>();

    let mut trend = Vec::new();
    for i in (0..=6).rev() {
        let date = chrono::DateTime::from_timestamp(now, 0)
            .map(|dt| (dt.date_naive() - chrono::Days::new(i)).to_string())
            .unwrap_or_else(|| chrono::Utc::now().date_naive().to_string());
        if let Some(row) = row_map.get(&date) {
            trend.push(json!({
                "date": date,
                "upload_kb": row.upload_kb,
                "download_kb": row.download_kb,
                "total_kb": row.upload_kb + row.download_kb,
                "unique_users": row.unique_users
            }));
        } else {
            trend.push(json!({
                "date": date,
                "upload_kb": 0,
                "download_kb": 0,
                "total_kb": 0,
                "unique_users": 0
            }));
        }
    }
    Ok(trend)
}

pub(crate) async fn resolve_command_center_throughput(
    state: &AppState,
    now_ts: i64,
) -> Result<(i64, i64), sqlx::Error> {
    const THROUGHPUT_SNAPSHOT_KEY: &str = "admin:command-center:throughput-snapshot";
    let row = sqlx::query(
        "SELECT CAST(COALESCE(SUM(u), 0) AS SIGNED) AS total_u,
                CAST(COALESCE(SUM(d), 0) AS SIGNED) AS total_d
         FROM v2_user"
    )
        .fetch_one(&state.db)
        .await?;
    let current_u = row.try_get::<i64, _>("total_u").unwrap_or(0);
    let current_d = row.try_get::<i64, _>("total_d").unwrap_or(0);
    let current_snapshot = TrafficSnapshot {
        ts: now_ts,
        u: current_u,
        d: current_d,
    };

    let redis_previous = redis_get_string(state, THROUGHPUT_SNAPSHOT_KEY)
        .await
        .ok()
        .flatten()
        .and_then(|value| serde_json::from_str::<TrafficSnapshot>(&value).ok());

    if let Ok(payload) = serde_json::to_string(&current_snapshot) {
        let _ = redis_setex_string(state, THROUGHPUT_SNAPSHOT_KEY, 120, &payload).await;
    }

    let mut previous = redis_previous;
    let mut snapshot = state.traffic_snapshot.write();
    if previous.is_none() {
        previous = *snapshot;
    }
    *snapshot = Some(current_snapshot);

    let Some(previous) = previous else {
        return Ok((0, 0));
    };
    let delta = now_ts - previous.ts;
    if !(10..=180).contains(&delta) {
        return Ok((0, 0));
    }
    Ok((
        ((current_u - previous.u) / delta).max(0),
        ((current_d - previous.d) / delta).max(0),
    ))
}

pub(crate) fn build_command_center_node_snapshots(
    nodes: &[CommandCenterNodeRow],
    status_cache: &HashMap<u64, Value>,
    online_sessions: &HashMap<u64, CommandCenterOnlineSessionAggRow>,
    weekly_traffic: &HashMap<u64, CommandCenterNodeTrafficAggRow>,
    latest_alert_map: &HashMap<u64, TcpingAlertRow>,
    alert_count_map: &HashMap<u64, i64>,
    now_ts: i64,
) -> Vec<Value> {
    let protocol_labels = server_node_protocol_labels();
    let mut rows = nodes
        .iter()
        .map(|node| {
            let session = online_sessions.get(&node.id);
            let traffic = weekly_traffic.get(&node.id);
            let active_alert = latest_alert_map.get(&node.id);
            let online_status = server_node_online_status(status_cache, node.id);
            let traffic_usage_percentage = command_center_node_traffic_usage_percentage(node);
            let tcping_monitorable = server_node_monitorable(&node.protocol, node.tcping_port);
            let tcping_status = if tcping_monitorable {
                node.tcping_last_status.clone().unwrap_or_else(|| {
                    if active_alert.is_some() { "offline".to_string() } else { "unknown".to_string() }
                })
            } else {
                "unsupported".to_string()
            };

            let mut watch_score = 0_i64;
            if online_status != "online" { watch_score += 140; }
            if tcping_monitorable && tcping_status == "offline" { watch_score += 90; }
            if active_alert.is_some() { watch_score += 110; }
            if node.status == "maintenance" { watch_score += 35; }
            if node.status == "deploying" { watch_score += 18; }
            watch_score += traffic_usage_percentage.round() as i64;
            watch_score += traffic.map(|v| (v.unique_users * 6).min(50)).unwrap_or(0);
            watch_score += session.map(|v| (v.active_users * 5).min(45)).unwrap_or(0);
            let normalized_protocol = node.protocol.to_ascii_lowercase();
            let protocol_label = protocol_labels
                .get(normalized_protocol.as_str())
                .copied()
                .map(str::to_string)
                .unwrap_or_else(|| node.protocol.to_ascii_uppercase());

            json!({
                "id": node.id,
                "name": node.name,
                "host": node.host,
                "port": node.port,
                "protocol": node.protocol,
                "protocol_label": protocol_label,
                "location_code": node.location_code.clone().unwrap_or_default(),
                "location_name": node.location_name.clone().unwrap_or_else(|| "未标注地区".to_string()),
                "status": node.status,
                "online_status": online_status,
                "traffic_limit_kb": node.traffic_limit,
                "traffic_used_kb": node.traffic_used,
                "traffic_usage_percentage": traffic_usage_percentage,
                "traffic_remaining_kb": if node.traffic_limit > 0 { Some((node.traffic_limit as i64 - node.traffic_used as i64).max(0)) } else { None },
                "traffic_multiplier": node.traffic_multiplier.parse::<f64>().unwrap_or(1.0),
                "weekly_traffic_kb": traffic.map(|v| v.traffic_kb).unwrap_or(0),
                "weekly_unique_users": traffic.map(|v| v.unique_users).unwrap_or(0),
                "online_users": session.map(|v| v.active_users).unwrap_or(0),
                "active_connections": session.map(|v| v.active_connections).unwrap_or(0),
                "owner_email": node.owner_email.clone().unwrap_or_else(|| "-".to_string()),
                "owner_name": node.owner_linux_do_name.clone().or(node.owner_linux_do_username.clone()).or(node.owner_email.clone()).unwrap_or_else(|| "-".to_string()),
                "tcping_enabled": tcping_monitorable,
                "tcping_status": tcping_status,
                "tcping_last_latency_ms": node.tcping_last_latency_ms,
                "tcping_last_sampled_at": node.tcping_last_sampled_at,
                "tcping_last_error": node.tcping_last_error.clone().or_else(|| active_alert.and_then(|v| v.latest_error.clone())).unwrap_or_default(),
                "active_alert": active_alert.map(|alert| json!({
                    "count": alert_count_map.get(&node.id).copied().unwrap_or(1),
                    "triggered_at": alert.triggered_at,
                    "duration_seconds": (now_ts - alert.triggered_at).max(0),
                    "latest_error": alert.latest_error.clone().unwrap_or_default(),
                })),
                "watch_score": watch_score,
            })
        })
        .collect::<Vec<_>>();

    rows.sort_by(|a, b| {
        b.get("watch_score")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            .cmp(&a.get("watch_score").and_then(Value::as_i64).unwrap_or(0))
    });
    rows
}

pub(crate) fn build_command_center_watchlist(
    node_snapshots: &[Value],
    tcping_samples: &HashMap<u64, Vec<TcpingSampleRow>>,
) -> Vec<Value> {
    node_snapshots
        .iter()
        .take(8)
        .map(|node| {
            let mut row = node.clone();
            if let Some(object) = row.as_object_mut() {
                let samples = node
                    .get("id")
                    .and_then(Value::as_u64)
                    .and_then(|node_id| tcping_samples.get(&node_id))
                    .map(|rows| rows.iter().map(serialize_tcping_sample).collect::<Vec<_>>())
                    .unwrap_or_default();
                object.insert("tcping_samples".to_string(), Value::Array(samples));
            }
            row
        })
        .collect()
}
