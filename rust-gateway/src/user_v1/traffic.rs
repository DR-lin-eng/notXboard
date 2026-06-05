use crate::*;

pub async fn node_stats(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_node_stats_response(&state, id, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn node_traffic(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_node_traffic_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn usage_logs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_usage_logs_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_node_stats_response(
    state: &AppState,
    node_id: u64,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let node = load_owned_server_node(state, node_id, user.id)
        .await
        .map_err(internal_error)?;
    let Some(node) = node else {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({"success": false, "error": "Server node not found"})));
    };

    let params = parse_query(&uri);
    let days = params
        .get("days")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(30)
        .max(1);
    let end = Utc::now().date_naive();
    let start = end - chrono::Days::new((days - 1) as u64);
    let stats = load_node_traffic_stats_summary(state, node.id, &start.to_string(), &end.to_string())
        .await
        .map_err(internal_error)?;
    let status_cache = state.load_status_cache.read().clone();

    Ok(json_value_response(json!({
        "success": true,
        "data": {
            "node": {
                "id": node.id,
                "name": node.name,
                "protocol": node.protocol,
                "location_name": node.location_name,
                "status": node.status,
                "online_status": server_node_online_status(&status_cache, node.id),
                "is_online": server_node_is_online(&status_cache, node.id),
                "last_report_at": server_node_last_report_at(&status_cache, node.id),
                "traffic_limit": node.traffic_limit,
                "traffic_used": node.traffic_used,
                "traffic_multiplier": node.traffic_multiplier.parse::<f64>().unwrap_or(1.0),
                "tcping_enabled": server_node_monitorable(&node.protocol, node.tcping_port),
                "tcping_status": if server_node_monitorable(&node.protocol, node.tcping_port) {
                    node.tcping_last_status.clone().unwrap_or_else(|| "unknown".to_string())
                } else {
                    "unsupported".to_string()
                },
                "tcping_last_latency_ms": if server_node_monitorable(&node.protocol, node.tcping_port) { node.tcping_last_latency_ms } else { None },
                "tcping_last_sampled_at": if server_node_monitorable(&node.protocol, node.tcping_port) { node.tcping_last_sampled_at } else { None },
            },
            "stats": stats
        }
    })))
}

async fn build_node_traffic_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let user_row = UserRow {
        id: user.id,
        token: None,
        group_id: user.group_id,
        subscribe_key: None,
        subscribe_salt: None,
        uuid: None,
        u: None,
        d: None,
        transfer_enable: None,
        expired_at: user.expired_at,
        trust_level: Some(user.trust_level),
        banned: Some(user.banned),
        is_super_admin: Some(user.is_super_admin),
        is_silenced: Some(user.is_silenced),
        subscription_credential_version: None,
    };
    let nodes = load_accessible_nodes_for_user_rows(state, &user_row)
        .await
        .map_err(internal_error)?;
    if nodes.is_empty() {
        return Ok(json_value_response(json!({"success": true, "data": []})));
    }
    let today = Utc::now().date_naive().to_string();
    let node_ids = nodes.iter().map(|node| node.id).collect::<Vec<_>>();
    let record_map = load_user_today_node_traffic_records(state, user.id, &node_ids, &today)
        .await
        .map_err(internal_error)?;
    let status_cache = state.load_status_cache.read().clone();

    let data = nodes
        .iter()
        .map(|node| {
            let key = node.id;
            let (upload, download) = record_map.get(&key).copied().unwrap_or((0_i64, 0_i64));
            json!({
                "node_id": node.id,
                "node_name": node.name,
                "protocol": node.protocol,
                "location_name": node.location_name,
                "status": node.status,
                "online_status": server_node_online_status(&status_cache, node.id),
                "is_online": server_node_is_online(&status_cache, node.id),
                "last_report_at": server_node_last_report_at(&status_cache, node.id),
                "traffic_limit": node.traffic_limit,
                "traffic_used": node.traffic_used,
                "traffic_multiplier": node.traffic_multiplier.parse::<f64>().unwrap_or(1.0),
                "node_remaining_traffic": if node.traffic_limit == 0 {
                    Value::from(i64::MAX)
                } else {
                    Value::from((node.traffic_limit.saturating_sub(node.traffic_used)) as i64)
                },
                "traffic_usage_percentage": if node.traffic_limit == 0 {
                    0.0
                } else {
                    ((node.traffic_used as f64 / node.traffic_limit as f64) * 100.0).min(100.0)
                },
                "tcping_enabled": node.tcping_enabled,
                "tcping_status": if node.tcping_enabled {
                    node.tcping_last_status.clone().unwrap_or_else(|| "unknown".to_string())
                } else {
                    "unsupported".to_string()
                },
                "tcping_last_latency_ms": if node.tcping_enabled { node.tcping_last_latency_ms } else { None },
                "tcping_last_sampled_at": if node.tcping_enabled { node.tcping_last_sampled_at } else { None },
                "today": {
                    "upload": upload,
                    "download": download,
                }
            })
        })
        .collect::<Vec<_>>();

    Ok(json_value_response(json!({
        "success": true,
        "data": data
    })))
}

async fn build_usage_logs_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let params = parse_query(&uri);
    let days = params
        .get("days")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(7)
        .clamp(1, 30);
    let limit = params
        .get("limit")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(200)
        .clamp(1, 500);
    let start_at = Utc::now().timestamp() - days * 86_400;
    let logs = load_user_traffic_usage_logs(state, user.id, start_at, limit)
        .await
        .map_err(internal_error)?;
    Ok(json_value_response(json!({
        "success": true,
        "data": logs.iter().map(serialize_user_traffic_usage_log).collect::<Vec<_>>()
    })))
}
