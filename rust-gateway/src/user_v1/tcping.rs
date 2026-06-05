use crate::*;

pub async fn agents(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_agents_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn create_agent(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_create_agent_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn rotate_agent_token(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_rotate_agent_token_response(&state, id, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn toggle_agent(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_toggle_agent_response(&state, id, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn agent_install_command(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_agent_install_command_response(&state, id, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn node_overview(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_node_overview_response(&state, id, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_agents_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let agents = load_tcping_agents_for_user(state, user.id)
        .await
        .map_err(internal_error)?;
    Ok(json_value_response(json!({
        "success": true,
        "data": agents.iter().map(serialize_tcping_agent).collect::<Vec<_>>(),
    })))
}

async fn build_create_agent_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let name = payload
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "The name field is required."))?;
    if name.chars().count() > 128 {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "The name field must not be greater than 128 characters."));
    }
    let location_code = payload
        .get("location_code")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "The location_code field is required."))?
        .to_uppercase();
    if location_code.chars().count() > 16 {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "The location_code field must not be greater than 16 characters."));
    }
    let location_name = payload
        .get("location_name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "The location_name field is required."))?;
    if location_name.chars().count() > 128 {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "The location_name field must not be greater than 128 characters."));
    }
    let location_province = payload
        .get("location_province")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    if location_code == "CN" && location_province.is_none() {
        return Ok(fail_json_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "中国大陆地区的探针必须填写省份归属",
        ));
    }
    if location_province.as_ref().map(|value| value.chars().count() > 64).unwrap_or(false) {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "The location_province field must not be greater than 64 characters."));
    }

    let token = random_alnum(48);
    let now = Utc::now().timestamp();
    let agent_id = sqlx::query(
        "INSERT INTO tcping_agents (
            user_id, name, location_code, location_name, location_province, token,
            is_enabled, last_heartbeat_at, last_sync_at, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, ?, 1, NULL, NULL, ?, ?)"
    )
    .bind(user.id)
    .bind(name)
    .bind(&location_code)
    .bind(location_name)
    .bind(location_province.as_deref())
    .bind(&token)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(internal_error)?
    .last_insert_id();

    let agent = load_tcping_agent_for_user(state, agent_id, user.id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::INTERNAL_SERVER_ERROR, "TCPing agent not found"))?;

    Ok(json_value_response(json!({
        "success": true,
        "data": serialize_tcping_agent(&agent),
    })))
}

async fn build_rotate_agent_token_response(
    state: &AppState,
    agent_id: u64,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let Some(_agent) = load_tcping_agent_for_user(state, agent_id, user.id)
        .await
        .map_err(internal_error)? else {
        return Ok(json_status_response(
            StatusCode::NOT_FOUND,
            json!({"success": false, "error": "TCPing agent not found"}),
        ));
    };

    let token = random_alnum(48);
    let now = Utc::now().timestamp();
    sqlx::query("UPDATE tcping_agents SET token = ?, updated_at = ? WHERE id = ? AND user_id = ?")
        .bind(&token)
        .bind(now)
        .bind(agent_id)
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    let agent = load_tcping_agent_for_user(state, agent_id, user.id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::INTERNAL_SERVER_ERROR, "TCPing agent not found"))?;

    Ok(json_value_response(json!({
        "success": true,
        "data": serialize_tcping_agent(&agent),
    })))
}

async fn build_toggle_agent_response(
    state: &AppState,
    agent_id: u64,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let Some(_agent) = load_tcping_agent_for_user(state, agent_id, user.id)
        .await
        .map_err(internal_error)? else {
        return Ok(json_status_response(
            StatusCode::NOT_FOUND,
            json!({"success": false, "error": "TCPing agent not found"}),
        ));
    };

    Ok(json_status_response(
        StatusCode::FORBIDDEN,
        json!({
            "success": false,
            "error": "TCPing Agent 监控由系统统一调度，当前不允许手动停用/启用",
        }),
    ))
}

async fn build_agent_install_command_response(
    state: &AppState,
    agent_id: u64,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let Some(agent) = load_tcping_agent_for_user(state, agent_id, user.id)
        .await
        .map_err(internal_error)? else {
        return Ok(json_status_response(
            StatusCode::NOT_FOUND,
            json!({"success": false, "error": "TCPing agent not found"}),
        ));
    };

    let panel_url = resolve_panel_base_url(state).await;
    let script_url = format!("{}/tcping-agent-install.sh?v=rust", panel_url.trim_end_matches('/'));
    let command = format!(
        "curl -fsSL '{}' | bash -s -- --panel '{}' --token '{}'",
        script_url, panel_url, agent.token
    );

    Ok(json_value_response(json!({
        "success": true,
        "data": {
            "panel_url": panel_url,
            "script_url": script_url,
            "token": agent.token,
            "command": command,
        },
    })))
}

async fn build_node_overview_response(
    state: &AppState,
    node_id: u64,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let node = load_tcping_node_overview_row(state, node_id)
        .await
        .map_err(internal_error)?;
    let Some(node) = node else {
        return Ok(json_status_response(
            StatusCode::NOT_FOUND,
            json!({"success": false, "error": "Server node not found"}),
        ));
    };

    if !user_can_access_tcping_node(state, &user, &node).await.map_err(internal_error)? {
        return Ok(json_status_response(
            StatusCode::NOT_FOUND,
            json!({"success": false, "error": "Server node not found"}),
        ));
    }

    let params = parse_query(&uri);
    let hours = params
        .get("hours")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(24)
        .clamp(1, 24 * 30);
    let start_at = Utc::now().timestamp() - hours * 3600;

    let samples = load_tcping_samples_for_node_since(state, node.id, start_at)
        .await
        .map_err(internal_error)?;
    let alerts = load_tcping_alerts_for_node(state, node.id)
        .await
        .map_err(internal_error)?;
    let reachable_count = samples.iter().filter(|sample| sample.is_reachable).count();
    let total_count = samples.len();

    let mut agent_ids = Vec::new();
    for sample in &samples {
        if let Some(agent_id) = sample.agent_id {
            if !agent_ids.contains(&agent_id) {
                agent_ids.push(agent_id);
            }
        }
    }
    let agents = load_tcping_agents_by_ids(state, &agent_ids)
        .await
        .map_err(internal_error)?;

    Ok(json_value_response(json!({
        "success": true,
        "data": {
            "node": serialize_tcping_node_summary(state, &node, &alerts),
            "range_hours": hours,
            "stats": {
                "total_samples": total_count,
                "reachable_samples": reachable_count,
                "reachability_rate": if total_count > 0 {
                    Value::from(((reachable_count as f64 / total_count as f64) * 100.0 * 100.0).round() / 100.0)
                } else {
                    Value::Null
                }
            },
            "agents": agents.iter().map(serialize_tcping_agent_location).collect::<Vec<_>>(),
            "samples": samples.iter().map(serialize_tcping_sample).collect::<Vec<_>>(),
            "alerts": alerts.iter().map(serialize_tcping_alert).collect::<Vec<_>>(),
            "can_manage": user.is_super_admin == 1 || node.user_id == user.id,
        }
    })))
}
