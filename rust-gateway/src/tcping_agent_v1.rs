use crate::*;

pub async fn config(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_config_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn heartbeat(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_heartbeat_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn samples(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_samples_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_config_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let agent = resolve_agent_by_token(state, &headers, &uri, body).await?;
    let targets = load_monitorable_targets_for_agent(state, &agent).await.map_err(internal_error)?;

    Ok(json_value_response(json!({
        "success": true,
        "data": {
            "agent": {
                "id": agent.id,
                "name": agent.name,
                "server_time": Utc::now().timestamp(),
                "pull_interval_seconds": 60,
            },
            "targets": targets,
        }
    })))
}

async fn build_heartbeat_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let agent = resolve_agent_by_token(state, &headers, &uri, body).await?;
    sqlx::query("UPDATE tcping_agents SET last_heartbeat_at = ?, updated_at = ? WHERE id = ?")
        .bind(Utc::now().timestamp())
        .bind(Utc::now().timestamp())
        .bind(agent.id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    Ok(json_value_response(json!({
        "success": true,
        "data": true,
    })))
}

async fn build_samples_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let agent = resolve_agent_by_token(state, &headers, &uri, Body::empty()).await?;
    let payload = parse_json_body(body).await?;
    let samples = payload
        .get("samples")
        .and_then(Value::as_array)
        .cloned()
        .or_else(|| payload.as_array().cloned())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Invalid samples payload"))?;

    let monitorable_nodes = load_monitorable_node_rows_for_agent(state, &agent).await.map_err(internal_error)?;
    let node_map = monitorable_nodes
        .into_iter()
        .map(|node| (node.id, node))
        .collect::<HashMap<_, _>>();

    let mut accepted = 0_i64;
    let now = Utc::now().timestamp();
    let mut tx = state.db.begin().await.map_err(internal_error)?;

    for item in samples {
        let Some(object) = item.as_object() else {
            continue;
        };
        let Some(node_id_i64) = object.get("node_id").and_then(parse_i64_value) else {
            continue;
        };
        if node_id_i64 <= 0 {
            continue;
        }
        let node_id = node_id_i64 as u64;
        let Some(node) = node_map.get(&node_id) else {
            continue;
        };

        let mut sampled_at = object.get("sampled_at").and_then(parse_i64_value).unwrap_or(now);
        if sampled_at <= 0 {
            sampled_at = now;
        }
        let is_reachable = object.get("is_reachable").and_then(Value::as_bool).unwrap_or(false);
        let latency_ms = if is_reachable {
            object.get("latency_ms").and_then(parse_i64_value).map(|value| value.max(0))
        } else {
            None
        };
        let is_timeout = object.get("is_timeout").and_then(Value::as_bool).unwrap_or(false);
        let error_message = object
            .get("error_message")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);

        sqlx::query(
            "INSERT INTO tcping_samples
                (node_id, agent_id, is_reachable, latency_ms, is_timeout, error_message, sampled_at, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(node_id)
        .bind(agent.id)
        .bind(is_reachable)
        .bind(latency_ms)
        .bind(is_timeout)
        .bind(error_message.as_deref())
        .bind(sampled_at)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;

        update_tcping_node_state_and_alerts_tx(
            &mut tx,
            node,
            sampled_at,
            is_reachable,
            latency_ms,
            error_message.as_deref().unwrap_or(""),
            is_timeout,
            now,
        )
        .await
        .map_err(internal_error)?;

        accepted += 1;
    }

    sqlx::query("UPDATE tcping_agents SET last_sync_at = ?, updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(now)
        .bind(agent.id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;

    tx.commit().await.map_err(internal_error)?;

    Ok(json_value_response(json!({
        "success": true,
        "data": {
            "accepted": accepted,
        },
    })))
}

async fn resolve_agent_by_token(
    state: &AppState,
    headers: &HeaderMap,
    uri: &Uri,
    body: Body,
) -> Result<TcpingAgentRow, Response<Body>> {
    let token = resolve_agent_token(headers, uri, body).await?;
    let agent = sqlx::query_as::<_, TcpingAgentRow>(
        "SELECT id, user_id, name, location_code, location_name, location_province, token,
                is_enabled, last_heartbeat_at, last_sync_at, created_at, updated_at
         FROM tcping_agents
         WHERE token = ?
         LIMIT 1"
    )
    .bind(&token)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?;

    let Some(agent) = agent else {
        return Err(json_status_response(
            StatusCode::UNAUTHORIZED,
            json!({"success": false, "error": "Invalid TCPing agent token"}),
        ));
    };

    Ok(agent)
}

async fn resolve_agent_token(
    headers: &HeaderMap,
    uri: &Uri,
    body: Body,
) -> Result<String, Response<Body>> {
    if let Some(token) = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Ok(token.to_string());
    }

    let params = parse_query(uri);
    if let Some(token) = params.get("token").map(|value| value.trim()).filter(|value| !value.is_empty()) {
        return Ok(token.to_string());
    }

    let payload = parse_json_body(body).await?;
    let token = payload
        .get("token")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            json_status_response(
                StatusCode::UNAUTHORIZED,
                json!({"success": false, "error": "Invalid TCPing agent token"}),
            )
        })?;
    Ok(token.to_string())
}

async fn load_monitorable_targets_for_agent(
    state: &AppState,
    agent: &TcpingAgentRow,
) -> Result<Vec<Value>, sqlx::Error> {
    let nodes = load_monitorable_node_rows_for_agent(state, agent).await?;
    Ok(nodes
        .into_iter()
        .map(|node| {
            json!({
                "node_id": node.id,
                "name": node.name,
                "host": node.tcping_host.clone().filter(|value| !value.trim().is_empty()).unwrap_or_else(|| node.host.clone()),
                "port": node.tcping_port.unwrap_or(node.port),
                "interval_seconds": node.tcping_interval_seconds.max(15),
                "timeout_ms": node.tcping_timeout_ms.max(500),
                "alert_after_seconds": node.tcping_alert_after_seconds.max(60),
                "recover_after_seconds": node.tcping_recover_after_seconds.max(30),
            })
        })
        .collect())
}

async fn load_monitorable_node_rows_for_agent(
    state: &AppState,
    agent: &TcpingAgentRow,
) -> Result<Vec<TcpingNodeOverviewRow>, sqlx::Error> {
    let user = load_bearer_user_by_id(state, agent.user_id).await?;
    let Some(user) = user else {
        return Ok(Vec::new());
    };

    let mut nodes = if user.is_super_admin == 1 {
        sqlx::query_as::<_, TcpingNodeOverviewRow>(
            "SELECT id, user_id, name, host, port, protocol, location_name, status,
                    tcping_enabled, tcping_host, tcping_port, tcping_interval_seconds,
                    tcping_timeout_ms, tcping_alert_after_seconds, tcping_recover_after_seconds,
                    tcping_last_status, tcping_last_latency_ms, tcping_last_error, tcping_last_sampled_at
             FROM server_nodes
             ORDER BY id DESC"
        )
        .fetch_all(&state.db)
        .await?
    } else {
        let mut owned = load_owned_server_nodes(state, user.id).await?
            .into_iter()
            .map(|node| TcpingNodeOverviewRow {
                id: node.id,
                user_id: node.user_id,
                name: node.name,
                host: node.host,
                port: node.port,
                protocol: node.protocol,
                location_name: node.location_name,
                status: node.status,
                tcping_enabled: node.tcping_enabled,
                tcping_host: node.tcping_host,
                tcping_port: node.tcping_port,
                tcping_interval_seconds: node.tcping_interval_seconds,
                tcping_timeout_ms: node.tcping_timeout_ms,
                tcping_alert_after_seconds: node.tcping_alert_after_seconds,
                tcping_recover_after_seconds: node.tcping_recover_after_seconds,
                tcping_last_status: node.tcping_last_status,
                tcping_last_latency_ms: node.tcping_last_latency_ms,
                tcping_last_error: node.tcping_last_error,
                tcping_last_sampled_at: node.tcping_last_sampled_at,
            })
            .collect::<Vec<_>>();

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
        let accessible = load_accessible_nodes_for_user_rows(state, &user_row).await?;
        let accessible_ids = accessible.into_iter().map(|node| node.id).collect::<Vec<_>>();
        let mut extra = Vec::new();
        for node_id in accessible_ids {
            if owned.iter().any(|node| node.id == node_id) {
                continue;
            }
            if let Some(node) = load_tcping_node_overview_row(state, node_id).await? {
                extra.push(node);
            }
        }
        owned.extend(extra);
        owned
    };

    nodes.sort_by(|a, b| b.id.cmp(&a.id));
    nodes.retain(|node| tcping_is_monitorable(node));
    nodes.dedup_by(|a, b| a.id == b.id);
    Ok(nodes)
}

async fn update_tcping_node_state_and_alerts_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    node: &TcpingNodeOverviewRow,
    sampled_at: i64,
    is_reachable: bool,
    latency_ms: Option<i64>,
    error_message: &str,
    is_timeout: bool,
    now: i64,
) -> Result<(), sqlx::Error> {
    let active_alert = sqlx::query_as::<_, TcpingAlertRow>(
        "SELECT id, node_id, user_id, status, started_at, triggered_at, recovered_at, latest_error
         FROM tcping_alerts
         WHERE node_id = ? AND status = 'active'
         ORDER BY triggered_at DESC
         LIMIT 1"
    )
    .bind(node.id)
    .fetch_optional(&mut **tx)
    .await?;

    if is_reachable {
        sqlx::query(
            "UPDATE server_nodes
             SET tcping_last_status = 'online',
                 tcping_last_latency_ms = ?,
                 tcping_last_error = NULL,
                 tcping_last_sampled_at = ?,
                 tcping_recovered_since = COALESCE(tcping_recovered_since, ?),
                 tcping_outage_since = NULL,
                 updated_at = ?
             WHERE id = ?"
        )
        .bind(latency_ms)
        .bind(sampled_at)
        .bind(sampled_at)
        .bind(now)
        .bind(node.id)
        .execute(&mut **tx)
        .await?;

        if let Some(alert) = active_alert {
            let recovered_since: Option<i64> = sqlx::query_scalar(
                "SELECT tcping_recovered_since FROM server_nodes WHERE id = ? LIMIT 1"
            )
            .bind(node.id)
            .fetch_optional(&mut **tx)
            .await?
            .flatten();
            let recover_after = node.tcping_recover_after_seconds.max(30);
            if let Some(recovered_since) = recovered_since {
                if sampled_at - recovered_since >= recover_after {
                    sqlx::query(
                        "UPDATE tcping_alerts
                         SET status = 'resolved', recovered_at = ?, latest_error = NULL, updated_at = ?
                         WHERE id = ?"
                    )
                    .bind(sampled_at)
                    .bind(now)
                    .bind(alert.id)
                    .execute(&mut **tx)
                    .await?;
                }
            }
        }
    } else {
        let last_error = if !error_message.trim().is_empty() {
            error_message.trim().to_string()
        } else if is_timeout {
            "timeout".to_string()
        } else {
            "unreachable".to_string()
        };

        sqlx::query(
            "UPDATE server_nodes
             SET tcping_last_status = 'offline',
                 tcping_last_latency_ms = NULL,
                 tcping_last_error = ?,
                 tcping_last_sampled_at = ?,
                 tcping_recovered_since = NULL,
                 tcping_outage_since = COALESCE(tcping_outage_since, ?),
                 updated_at = ?
             WHERE id = ?"
        )
        .bind(&last_error)
        .bind(sampled_at)
        .bind(sampled_at)
        .bind(now)
        .bind(node.id)
        .execute(&mut **tx)
        .await?;

        if let Some(alert) = active_alert {
            sqlx::query("UPDATE tcping_alerts SET latest_error = ?, updated_at = ? WHERE id = ?")
                .bind(&last_error)
                .bind(now)
                .bind(alert.id)
                .execute(&mut **tx)
                .await?;
        } else {
            let outage_since: Option<i64> = sqlx::query_scalar(
                "SELECT tcping_outage_since FROM server_nodes WHERE id = ? LIMIT 1"
            )
            .bind(node.id)
            .fetch_optional(&mut **tx)
            .await?
            .flatten();
            let alert_after = node.tcping_alert_after_seconds.max(60);
            if let Some(outage_since) = outage_since {
                if sampled_at - outage_since >= alert_after {
                    sqlx::query(
                        "INSERT INTO tcping_alerts
                            (node_id, user_id, status, started_at, triggered_at, latest_error, created_at, updated_at)
                         VALUES (?, ?, 'active', ?, ?, ?, ?, ?)"
                    )
                    .bind(node.id)
                    .bind(node.user_id)
                    .bind(outage_since)
                    .bind(sampled_at)
                    .bind(&last_error)
                    .bind(now)
                    .bind(now)
                    .execute(&mut **tx)
                    .await?;
                }
            }
        }
    }

    Ok(())
}
