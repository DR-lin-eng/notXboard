use crate::*;

pub async fn protocols(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_protocols_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn fetch(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_fetch_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn index(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_index_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn store(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_store_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn show(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_show_response(&state, id, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_update_response(&state, id, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn destroy(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_destroy_response(&state, id, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn status(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_status_response(&state, id, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn deploy(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_deploy_response(&state, id, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn deploy_command(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_deploy_command_response(&state, id, headers, uri, false).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn rotate_deploy_token(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_deploy_command_response(&state, id, headers, uri, true).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_protocols_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _user = authenticate_bearer_user(state, &headers).await?;
    Ok(json_value_response(json!({
        "data": server_node_protocol_items()
    })))
}

async fn build_fetch_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let cache_key = build_user_cache_key(&uri, user.id);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }

    let mut data = Vec::new();
    if user_is_available(&user) {
        let status_cache = state.load_status_cache.read().clone();
        let nodes = load_available_server_nodes_for_user(
            state,
            user.id,
            user.trust_level as i64,
            user.is_super_admin != 0,
        )
            .await
            .map_err(internal_error)?;
        data = nodes
            .iter()
            .filter_map(|node| serialize_user_server_fetch_node_item(&status_cache, node))
            .collect::<Vec<_>>();
    }

    Ok(json_cached_response(
        state,
        cache_key,
        json!({ "data": data }),
        Duration::from_secs(15),
    ))
}

async fn build_index_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let nodes = load_owned_server_nodes(state, user.id)
        .await
        .map_err(internal_error)?;
    let status_cache = state.load_status_cache.read().clone();
    let data = nodes
        .iter()
        .map(|node| serialize_server_node_index_item(&status_cache, node))
        .collect::<Vec<_>>();
    Ok(json_value_response(json!({ "data": data })))
}

async fn build_store_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let mut input = parse_server_node_mutation_input(
        &payload,
        None,
        user.is_super_admin == 1,
        user.is_super_admin == 1,
    )?;
    input.user_id = Some(user.id);
    input.status = Some("inactive".to_string());
    input.traffic_used = Some(0);

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let access_control_json = input
        .access_control
        .as_ref()
        .map(|value| Value::Object(value.clone()))
        .unwrap_or(Value::Null);
    let settings_json = input
        .settings
        .as_ref()
        .map(|value| Value::Object(value.clone()))
        .unwrap_or(Value::Null);
    let v2bx_config_json = Value::Object(Map::new());

    let result = sqlx::query(
        "INSERT INTO server_nodes (
            user_id, name, host, port, service_port, protocol, location_code, location_name,
            settings, traffic_limit, traffic_used, traffic_multiplier, access_control, status,
            v2bx_config, device_limit, connection_limit, speed_limit_up, speed_limit_down,
            cross_node_ip_limit, concurrent_ip_limit, tcping_enabled, tcping_host, tcping_port,
            tcping_interval_seconds, tcping_timeout_ms, tcping_alert_after_seconds, tcping_recover_after_seconds,
            created_at, updated_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?, ?, ?, ?, ?, NOW(), NOW())"
    )
    .bind(input.user_id.unwrap_or(user.id))
    .bind(input.name.as_deref().unwrap_or(""))
    .bind(input.host.as_deref().unwrap_or(""))
    .bind(input.port.unwrap_or(0))
    .bind(input.service_port)
    .bind(input.protocol.as_deref().unwrap_or(""))
    .bind(input.location_code.as_deref())
    .bind(input.location_name.as_deref())
    .bind(settings_json.to_string())
    .bind(input.traffic_limit.unwrap_or(0))
    .bind(input.traffic_used.unwrap_or(0))
    .bind(input.traffic_multiplier.unwrap_or(1.0))
    .bind(access_control_json.to_string())
    .bind(input.status.as_deref().unwrap_or("inactive"))
    .bind(v2bx_config_json.to_string())
    .bind(input.device_limit.unwrap_or(0))
    .bind(input.connection_limit.unwrap_or(0))
    .bind(input.speed_limit_up.unwrap_or(0))
    .bind(input.speed_limit_down.unwrap_or(0))
    .bind(input.cross_node_ip_limit.unwrap_or(0))
    .bind(input.concurrent_ip_limit.unwrap_or(0))
    .bind(input.tcping_host.as_deref())
    .bind(input.tcping_port)
    .bind(input.tcping_interval_seconds.unwrap_or(60))
    .bind(input.tcping_timeout_ms.unwrap_or(3000))
    .bind(input.tcping_alert_after_seconds.unwrap_or(300))
    .bind(input.tcping_recover_after_seconds.unwrap_or(120))
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;

    let node_id = result.last_insert_id();
    if let Some(access) = input.access_control {
        if !apply_owned_server_node_access_control_with_tx(
            &mut tx,
            node_id,
            user.id,
            &access,
        )
        .await
        .map_err(internal_error)?
        {
            tx.rollback().await.ok();
            return Ok(json_status_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "message": "Server node could not be created" }),
            ));
        }
    }
    tx.commit().await.map_err(internal_error)?;
    clear_accessible_user_ids_cache(state, node_id);

    let node = load_owned_server_node(state, node_id, user.id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::INTERNAL_SERVER_ERROR, "Server node not found"))?;
    Ok(json_status_response(
        StatusCode::CREATED,
        json!({
            "message": "Server node created successfully",
            "data": serialize_server_node_owner_model(&node)
        }),
    ))
}

async fn build_show_response(
    state: &AppState,
    node_id: u64,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let node = load_owned_server_node(state, node_id, user.id)
        .await
        .map_err(internal_error)?;
    let Some(node) = node else {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({ "message": "Server node not found" })));
    };
    let status_cache = state.load_status_cache.read().clone();
    let audit_rules = load_node_owner_audit_rules(state, node.id)
        .await
        .map_err(internal_error)?;
    let authorized_user_ids = load_authorized_user_ids_for_node(state, node.id)
        .await
        .map_err(internal_error)?;
    let authorized_users = load_node_traffic_user_profiles(state, &authorized_user_ids)
        .await
        .map_err(internal_error)?;
    let mut payload = serialize_server_node_detail(&status_cache, &node);
    if let Some(object) = payload.as_object_mut() {
        object.insert(
            "audit_rules".to_string(),
            Value::Array(audit_rules.iter().map(serialize_audit_rule).collect::<Vec<_>>()),
        );
        object.insert(
            "authorized_users".to_string(),
            Value::Array(authorized_users.iter().map(serialize_authorized_user_profile).collect::<Vec<_>>()),
        );
    }
    Ok(json_value_response(json!({
        "data": payload
    })))
}

async fn build_update_response(
    state: &AppState,
    node_id: u64,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let existing = load_owned_server_node(state, node_id, user.id)
        .await
        .map_err(internal_error)?;
    let Some(existing) = existing else {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({ "message": "Server node not found" })));
    };

    let payload = parse_json_body(body).await?;
    let input = parse_server_node_mutation_input(
        &payload,
        Some(&existing),
        user.is_super_admin == 1,
        user.is_super_admin == 1,
    )?;

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    if !lock_owned_server_node_for_update(&mut tx, existing.id, user.id)
        .await
        .map_err(internal_error)?
    {
        tx.rollback().await.ok();
        return Ok(json_status_response(
            StatusCode::NOT_FOUND,
            json!({ "message": "Server node not found" }),
        ));
    }

    let updated = sqlx::query(
        "UPDATE server_nodes
         SET name = ?, host = ?, port = ?, service_port = ?, protocol = ?, location_code = ?, location_name = ?,
             settings = ?, traffic_limit = ?, traffic_multiplier = ?, device_limit = ?, connection_limit = ?,
             speed_limit_up = ?, speed_limit_down = ?, cross_node_ip_limit = ?, concurrent_ip_limit = ?,
             tcping_enabled = 1, tcping_host = ?, tcping_port = ?, tcping_interval_seconds = ?, tcping_timeout_ms = ?,
             tcping_alert_after_seconds = ?, tcping_recover_after_seconds = ?, updated_at = NOW()
         WHERE id = ? AND user_id = ?"
    )
    .bind(input.name.as_deref().unwrap_or(&existing.name))
    .bind(input.host.as_deref().unwrap_or(&existing.host))
    .bind(input.port.unwrap_or(existing.port))
    .bind(input.service_port.or(existing.service_port))
    .bind(input.protocol.as_deref().unwrap_or(&existing.protocol))
    .bind(input.location_code.as_deref().or(existing.location_code.as_deref()))
    .bind(input.location_name.as_deref().or(existing.location_name.as_deref()))
    .bind(input.settings.map(Value::Object).unwrap_or_else(|| existing.settings.as_ref().map(|v| v.0.clone()).unwrap_or(Value::Object(Map::new()))).to_string())
    .bind(input.traffic_limit.unwrap_or(existing.traffic_limit as i64))
    .bind(input.traffic_multiplier.unwrap_or(existing.traffic_multiplier.parse::<f64>().unwrap_or(1.0)))
    .bind(input.device_limit.unwrap_or(existing.device_limit))
    .bind(input.connection_limit.unwrap_or(existing.connection_limit))
    .bind(input.speed_limit_up.unwrap_or(existing.speed_limit_up))
    .bind(input.speed_limit_down.unwrap_or(existing.speed_limit_down))
    .bind(input.cross_node_ip_limit.unwrap_or(existing.cross_node_ip_limit))
    .bind(input.concurrent_ip_limit.unwrap_or(existing.concurrent_ip_limit))
    .bind(input.tcping_host.as_deref().or(existing.tcping_host.as_deref()))
    .bind(input.tcping_port.or(existing.tcping_port))
    .bind(input.tcping_interval_seconds.unwrap_or(existing.tcping_interval_seconds))
    .bind(input.tcping_timeout_ms.unwrap_or(existing.tcping_timeout_ms))
    .bind(input.tcping_alert_after_seconds.unwrap_or(existing.tcping_alert_after_seconds))
    .bind(input.tcping_recover_after_seconds.unwrap_or(existing.tcping_recover_after_seconds))
    .bind(existing.id)
    .bind(user.id)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;
    if !owner_scoped_server_node_write_matched(
        &mut tx,
        existing.id,
        user.id,
        updated.rows_affected(),
    )
    .await
    .map_err(internal_error)?
    {
        tx.rollback().await.ok();
        return Ok(json_status_response(
            StatusCode::NOT_FOUND,
            json!({ "message": "Server node not found" }),
        ));
    }

    if let Some(access) = input.access_control {
        let access_updated = sqlx::query(
            "UPDATE server_nodes
             SET access_control = ?, updated_at = NOW()
             WHERE id = ? AND user_id = ?",
        )
        .bind(Value::Object(access.clone()).to_string())
        .bind(existing.id)
        .bind(user.id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
        if !owner_scoped_server_node_write_matched(
            &mut tx,
            existing.id,
            user.id,
            access_updated.rows_affected(),
        )
        .await
        .map_err(internal_error)?
        {
            tx.rollback().await.ok();
            return Ok(json_status_response(
                StatusCode::NOT_FOUND,
                json!({ "message": "Server node not found" }),
            ));
        }
        if !apply_owned_server_node_access_control_with_tx(
            &mut tx,
            existing.id,
            user.id,
            &access,
        )
        .await
        .map_err(internal_error)?
        {
            tx.rollback().await.ok();
            return Ok(json_status_response(
                StatusCode::NOT_FOUND,
                json!({ "message": "Server node not found" }),
            ));
        }
    }

    tx.commit().await.map_err(internal_error)?;
    clear_accessible_user_ids_cache(state, existing.id);
    let refreshed = load_owned_server_node(state, existing.id, user.id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::INTERNAL_SERVER_ERROR, "Server node not found"))?;
    Ok(json_value_response(json!({
        "message": "Server node updated successfully",
        "data": serialize_server_node_owner_model(&refreshed)
    })))
}

async fn build_destroy_response(
    state: &AppState,
    node_id: u64,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let node = load_owned_server_node(state, node_id, user.id)
        .await
        .map_err(internal_error)?;
    let Some(node) = node else {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({ "message": "Server node not found" })));
    };

    let online_users = load_online_user_count_for_node(state, node.id)
        .await
        .map_err(internal_error)?;
    if online_users > 0 {
        return Ok(json_status_response(
            StatusCode::CONFLICT,
            json!({
                "message": "Cannot delete node with active users",
                "online_users": online_users,
            }),
        ));
    }

    sqlx::query("DELETE FROM server_nodes WHERE id = ? AND user_id = ?")
        .bind(node.id)
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    Ok(json_value_response(json!({
        "message": "Server node deleted successfully"
    })))
}

async fn build_status_response(
    state: &AppState,
    node_id: u64,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let node = load_owned_server_node(state, node_id, user.id)
        .await
        .map_err(internal_error)?;
    let Some(node) = node else {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({ "message": "Server node not found" })));
    };
    let status_cache = state.load_status_cache.read().clone();
    Ok(json_value_response(json!({
        "data": serialize_server_node_status(&status_cache, &node)
    })))
}

async fn build_deploy_response(
    state: &AppState,
    node_id: u64,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let node = load_owned_server_node(state, node_id, user.id)
        .await
        .map_err(internal_error)?;
    let Some(node) = node else {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({ "message": "Server node not found" })));
    };

    let node_type = node.protocol.to_ascii_lowercase();
    if !server_node_supports_v2bx_deploy(&node_type) {
        return Ok(json_status_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "message": "当前协议暂不支持 V2bX 一键部署",
                "data": {
                    "protocol": node_type,
                    "supported_protocols": server_node_v2bx_supported_protocols(),
                }
            }),
        ));
    }

    sqlx::query("UPDATE server_nodes SET status = 'deploying', updated_at = NOW() WHERE id = ? AND user_id = ?")
        .bind(node.id)
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    sqlx::query(
        "UPDATE server_nodes
         SET status = 'active',
             v2bx_node_id = COALESCE(v2bx_node_id, ?),
             updated_at = NOW()
         WHERE id = ? AND user_id = ?"
    )
    .bind(node.id as i64)
    .bind(node.id)
    .bind(user.id)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    let refreshed = load_owned_server_node(state, node.id, user.id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::INTERNAL_SERVER_ERROR, "Server node not found"))?;

    Ok(json_value_response(json!({
        "message": "Server node deployed successfully",
        "data": serialize_server_node_owner_model(&refreshed)
    })))
}

async fn build_deploy_command_response(
    state: &AppState,
    node_id: u64,
    headers: HeaderMap,
    _uri: Uri,
    rotate_token: bool,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let node = load_owned_server_node(state, node_id, user.id)
        .await
        .map_err(internal_error)?;
    let Some(node) = node else {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({ "message": "Server node not found" })));
    };

    let node_type = node.protocol.to_ascii_lowercase();
    if !server_node_supports_v2bx_deploy(&node_type) {
        return Ok(json_status_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "message": "当前协议暂不支持 V2bX 一键部署命令",
                "data": {
                    "protocol": node_type,
                    "supported_protocols": server_node_v2bx_supported_protocols(),
                }
            }),
        ));
    }

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    if !lock_owned_server_node_for_update(&mut tx, node.id, user.id)
        .await
        .map_err(internal_error)?
    {
        tx.rollback().await.ok();
        return Ok(json_status_response(
            StatusCode::NOT_FOUND,
            json!({ "message": "Server node not found" }),
        ));
    }
    let current_token = sqlx::query_scalar::<_, Option<String>>(
        "SELECT v2bx_token FROM server_nodes WHERE id = ? AND user_id = ? LIMIT 1",
    )
    .bind(node.id)
    .bind(user.id)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal_error)?;

    let token_is_reused = if let Some(existing_token) = current_token.as_deref() {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM server_nodes WHERE v2bx_token = ?",
        )
        .bind(existing_token)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal_error)?
            > 1
    } else {
        false
    };
    let user_api_key = user.api_key.as_deref().map(str::trim).filter(|value| !value.is_empty());
    let token_uses_user_credential = current_token
        .as_deref()
        .is_some_and(|token| user_api_key == Some(token));
    let token = if rotate_token
        || current_token.as_deref().map_or(true, str::is_empty)
        || !current_token
            .as_deref()
            .is_some_and(crate::machine_bootstrap_support::is_strong_machine_token)
        || token_is_reused
        || token_uses_user_credential
    {
        format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
    } else {
        current_token.clone().unwrap_or_default()
    };
    let token_changed = current_token.as_deref() != Some(token.as_str());
    if token_changed {
        let updated = sqlx::query(
            "UPDATE server_nodes
             SET v2bx_token = ?, updated_at = NOW()
             WHERE id = ? AND user_id = ?",
        )
            .bind(&token)
            .bind(node.id)
            .bind(user.id)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
        if updated.rows_affected() != 1 {
            tx.rollback().await.ok();
            return Ok(json_status_response(
                StatusCode::NOT_FOUND,
                json!({ "message": "Server node not found" }),
            ));
        }
    }
    tx.commit().await.map_err(internal_error)?;

    let bootstrap = crate::machine_bootstrap_support::issue_machine_bootstrap_ticket(
        state,
        "v2bx",
        user.id,
        node.id,
        &token,
    )
    .await
    .map_err(|err| {
        error!("v2bx bootstrap ticket creation failed: {err}");
        json_error(StatusCode::SERVICE_UNAVAILABLE, "Installer bootstrap unavailable")
    })?;
    let panel_url = resolve_installer_panel_base_url(state).await?;
    let script_url = format!(
        "{}/v2bx-install.sh?{}",
        panel_url.trim_end_matches('/'),
        bootstrap.query,
    );
    let node_id_value = node.v2bx_node_id.unwrap_or(node.id as i64);
    let cert_domain = node.host.trim().to_string();
    let auto_cert_mode = server_node_prefers_auto_cert(&node_type);
    let core = node
        .v2bx_config
        .as_ref()
        .and_then(|value| value.0.get("core").or_else(|| value.0.get("Core")))
        .and_then(Value::as_str)
        .map(ToString::to_string);

    let mut command = format!(
        "{} {} | bash -s -- --panel {} --node-id {} --node-type {} --bootstrap-token {} --bootstrap-query {}",
        installer_curl_command_prefix(&panel_url),
        posix_shell_arg(&script_url),
        posix_shell_arg(&panel_url),
        posix_shell_arg(&node_id_value.to_string()),
        posix_shell_arg(&node_type),
        posix_shell_arg(&bootstrap.ticket),
        posix_shell_arg(&bootstrap.query),
    );
    if let Some(core) = &core {
        command.push_str(&format!(" --core {}", posix_shell_arg(core)));
    }
    if auto_cert_mode {
        command.push_str(" --cert-mode self");
        if !cert_domain.is_empty() {
            command.push_str(&format!(" --cert-domain {}", posix_shell_arg(&cert_domain)));
        }
    }

    Ok(json_value_response(json!({
        "data": {
            "script_url": script_url,
            "command": command,
            "panel_url": panel_url,
            "node_id": node_id_value,
            "node_type": node_type,
            "bootstrap_expires_in": bootstrap.expires_in,
            "token_rotated": token_changed,
            "core": core,
            "cert_mode": if auto_cert_mode { Value::String("self".to_string()) } else { Value::Null },
            "cert_domain": if auto_cert_mode && !cert_domain.is_empty() { Value::String(cert_domain) } else { Value::Null },
        }
    })))
}

pub(crate) async fn lock_owned_server_node_for_update(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    node_id: u64,
    owner_user_id: i64,
) -> Result<bool, sqlx::Error> {
    let locked = sqlx::query_scalar::<_, u64>(
        "SELECT id
         FROM server_nodes
         WHERE id = ? AND user_id = ?
         LIMIT 1
         FOR UPDATE",
    )
    .bind(node_id)
    .bind(owner_user_id)
    .fetch_optional(&mut **tx)
    .await?;
    Ok(locked.is_some())
}

pub(crate) async fn owner_scoped_server_node_write_matched(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    node_id: u64,
    owner_user_id: i64,
    rows_affected: u64,
) -> Result<bool, sqlx::Error> {
    if rows_affected == 1 {
        return Ok(true);
    }
    if rows_affected > 1 {
        return Ok(false);
    }

    let matched = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)
         FROM server_nodes
         WHERE id = ? AND user_id = ?",
    )
    .bind(node_id)
    .bind(owner_user_id)
    .fetch_one(&mut **tx)
    .await?;
    Ok(matched == 1)
}

pub(crate) async fn replace_owned_individual_node_access_with_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    node_id: u64,
    owner_user_id: i64,
    user_ids: &[i64],
) -> Result<bool, sqlx::Error> {
    if !lock_owned_server_node_for_update(tx, node_id, owner_user_id).await? {
        return Ok(false);
    }

    sqlx::query(
        "DELETE grant_row
         FROM user_node_access grant_row
         JOIN server_nodes node_row ON node_row.id = grant_row.node_id
         WHERE grant_row.node_id = ? AND node_row.user_id = ?",
    )
    .bind(node_id)
    .bind(owner_user_id)
    .execute(&mut **tx)
    .await?;

    for user_id in distinct_positive_user_ids(user_ids.iter().copied()) {
        let inserted = sqlx::query(
            "INSERT INTO user_node_access (user_id, node_id, access_type, granted_at)
             SELECT ?, node_row.id, 'individual', CURRENT_TIMESTAMP
             FROM server_nodes node_row
             WHERE node_row.id = ? AND node_row.user_id = ?
             ON DUPLICATE KEY UPDATE
               access_type = VALUES(access_type),
               granted_at = CURRENT_TIMESTAMP",
        )
        .bind(user_id)
        .bind(node_id)
        .bind(owner_user_id)
        .execute(&mut **tx)
        .await?;
        // MySQL reports 2 for an existing row whose duplicate-key update changed data.
        if inserted.rows_affected() > 2 {
            return Ok(false);
        }
    }

    Ok(true)
}

async fn apply_owned_server_node_access_control_with_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    node_id: u64,
    owner_user_id: i64,
    access: &Map<String, Value>,
) -> Result<bool, sqlx::Error> {
    let Some(authorized_users) = access.get("authorized_users").and_then(Value::as_array) else {
        return Ok(true);
    };
    let user_ids = distinct_positive_user_ids(authorized_users.iter().filter_map(parse_i64_value));
    replace_owned_individual_node_access_with_tx(tx, node_id, owner_user_id, &user_ids).await
}

pub(crate) fn posix_shell_arg(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod deploy_command_tests {
    use super::posix_shell_arg;

    #[test]
    fn posix_shell_argument_keeps_single_quotes_inside_one_argument() {
        assert_eq!(posix_shell_arg("plain"), "'plain'");
        assert_eq!(
            posix_shell_arg("host'; touch /tmp/injected; echo '"),
            "'host'\"'\"'; touch /tmp/injected; echo '\"'\"''",
        );
    }
}
