use crate::*;

pub async fn stats(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_stats_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn accessible_nodes(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_accessible_nodes_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

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

pub async fn configure_node(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_configure_node_response(&state, id, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn share_with_user(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_share_with_user_response(&state, id, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn share_with_group(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_share_with_group_response(&state, id, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn share_revoke(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_share_revoke_response(&state, id, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_stats_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let owned_nodes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM server_nodes WHERE user_id = ?")
        .bind(user.id)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    let accessible_nodes = load_accessible_nodes_for_user_payload(state, user.id).await.map_err(internal_error)?;
    let accessible_count = accessible_nodes.len() as i64;
    let shared_nodes = (accessible_count - owned_nodes).max(0);
    Ok(json_value_response(json!({
        "success": true,
        "data": {
            "owned_nodes": owned_nodes,
            "accessible_nodes": accessible_count,
            "shared_nodes": shared_nodes,
            "trust_level": user.trust_level,
            "user_group": trust_level_group_name(user.trust_level),
        }
    })))
}

async fn build_accessible_nodes_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let data = load_accessible_nodes_for_user_payload(state, user.id)
        .await
        .map_err(internal_error)?;
    Ok(json_value_response(json!({
        "success": true,
        "data": data,
    })))
}

async fn build_node_stats_response(
    state: &AppState,
    node_id: u64,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let node = load_owned_node_admin_node(state, node_id, user.id).await.map_err(internal_error)?;
    let Some(node) = node else {
        return Ok(json_status_response(
            StatusCode::NOT_FOUND,
            json!({
                "success": false,
                "error": "Server node not found"
            }),
        ));
    };

    let stats_row = load_access_stats_node(state, node.id).await.map_err(internal_error)?;
    let accessible_user_ids = get_accessible_user_ids_for_owner_node(state, node.id).await.map_err(internal_error)?;
    let individually_authorized_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_node_access WHERE node_id = ?")
        .bind(node.id)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    let min_trust_level = stats_row
        .as_ref()
        .and_then(|row| row.access_control.as_ref())
        .and_then(|json| json.0.get("min_trust_level"))
        .and_then(|value| value.as_i64());
    let has_individual = individually_authorized_users > 0;
    let access_type = if min_trust_level.is_some() && has_individual {
        "mixed"
    } else if min_trust_level.is_some() {
        "group_based"
    } else if has_individual {
        "individual_based"
    } else {
        "owner_only"
    };

    Ok(json_value_response(json!({
        "success": true,
        "data": {
            "total_accessible_users": accessible_user_ids.len(),
            "individually_authorized_users": individually_authorized_users,
            "min_trust_level": min_trust_level,
            "access_type": access_type,
        }
    })))
}

async fn build_configure_node_response(
    state: &AppState,
    node_id: u64,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let node = load_owned_server_node(state, node_id, user.id)
        .await
        .map_err(internal_error)?;
    let Some(node) = node else {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({ "message": "Server node not found" })));
    };

    let payload = parse_json_body(body).await?;
    let min_trust_level = payload.get("min_trust_level").and_then(parse_i64_value);
    if min_trust_level.map(|value| !(0..=4).contains(&value)).unwrap_or(false) {
        return Ok(json_status_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({"message": "Validation failed", "errors": {"min_trust_level": ["The min_trust_level must be between 0 and 4."]}}),
        ));
    }

    let authorized_users = payload
        .get("authorized_users")
        .and_then(Value::as_array)
        .map(|values| distinct_positive_user_ids(values.iter().filter_map(parse_i64_value)));

    let mut access_control = payload.as_object().cloned().unwrap_or_default();
    if let Some(level) = min_trust_level {
        access_control.insert("min_trust_level".to_string(), Value::from(level));
    }
    if let Some(user_ids) = &authorized_users {
        access_control.insert(
            "authorized_users".to_string(),
            Value::Array(user_ids.iter().copied().map(Value::from).collect()),
        );
    }
    let access_control_json = Value::Object(access_control.clone());

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    sqlx::query("UPDATE server_nodes SET access_control = ?, updated_at = NOW() WHERE id = ? AND user_id = ?")
        .bind(access_control_json.to_string())
        .bind(node.id)
        .bind(user.id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    if let Some(user_ids) = authorized_users {
        replace_individual_node_access_with_tx(&mut tx, node.id, &user_ids)
            .await
            .map_err(internal_error)?;
    }
    tx.commit().await.map_err(internal_error)?;
    clear_accessible_user_ids_cache(state, node.id);

    let refreshed = load_owned_server_node(state, node.id, user.id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::INTERNAL_SERVER_ERROR, "Server node not found"))?;
    let status_cache = state.load_status_cache.read().clone();
    let audit_rules = load_node_owner_audit_rules(state, refreshed.id)
        .await
        .map_err(internal_error)?;
    let authorized_user_ids = load_authorized_user_ids_for_node(state, refreshed.id)
        .await
        .map_err(internal_error)?;
    let authorized_users = load_node_traffic_user_profiles(state, &authorized_user_ids)
        .await
        .map_err(internal_error)?;
    let mut payload = serialize_server_node_detail(&status_cache, &refreshed);
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
        "message": "Access control configured successfully",
        "data": payload
    })))
}

async fn build_share_with_user_response(
    state: &AppState,
    node_id: u64,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let node = load_owned_node_admin_node(state, node_id, user.id).await.map_err(internal_error)?;
    let Some(node) = node else {
        return Ok(json_status_response(
            StatusCode::NOT_FOUND,
            json!({
                "success": false,
                "error": "Server node not found"
            }),
        ));
    };
    let payload = parse_json_body(body).await?;
    let target_user_id = payload
        .get("user_id")
        .and_then(parse_i64_value)
        .filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "The user_id field is required."))?;
    let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_user WHERE id = ?")
        .bind(target_user_id)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    if exists <= 0 {
        return Ok(json_value_response(json!({"success": false, "error": "User not found"})));
    }
    upsert_individual_node_access(state, node.id, target_user_id)
        .await
        .map_err(internal_error)?;
    clear_accessible_user_ids_cache(state, node.id);
    Ok(json_value_response(json!({ "success": true })))
}

async fn build_share_with_group_response(
    state: &AppState,
    node_id: u64,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let node = load_owned_node_admin_node(state, node_id, user.id).await.map_err(internal_error)?;
    let Some(node) = node else {
        return Ok(json_status_response(
            StatusCode::NOT_FOUND,
            json!({
                "success": false,
                "error": "Server node not found"
            }),
        ));
    };
    let payload = parse_json_body(body).await?;
    let min_trust_level = payload
        .get("min_trust_level")
        .and_then(parse_i64_value)
        .filter(|value| (0..=4).contains(value))
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "The min_trust_level field is required."))?;
    let current = load_access_stats_node(state, node.id).await.map_err(internal_error)?;
    let mut access_control = current
        .and_then(|row| row.access_control.map(|value| value.0))
        .unwrap_or_else(|| json!({}));
    if !access_control.is_object() {
        access_control = json!({});
    }
    access_control["min_trust_level"] = Value::from(min_trust_level);
    sqlx::query("UPDATE server_nodes SET access_control = ?, updated_at = NOW() WHERE id = ?")
    .bind(access_control.to_string())
    .bind(node.id)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;
    clear_accessible_user_ids_cache(state, node.id);
    Ok(json_value_response(json!({ "success": true })))
}

async fn build_share_revoke_response(
    state: &AppState,
    node_id: u64,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let node = load_owned_node_admin_node(state, node_id, user.id).await.map_err(internal_error)?;
    let Some(node) = node else {
        return Ok(json_status_response(
            StatusCode::NOT_FOUND,
            json!({
                "success": false,
                "error": "Server node not found"
            }),
        ));
    };
    let payload = parse_json_body(body).await?;
    let target_user_id = payload
        .get("user_id")
        .and_then(parse_i64_value)
        .filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "The user_id field is required."))?;
    revoke_individual_node_access(state, node.id, target_user_id)
        .await
        .map_err(internal_error)?;
    clear_accessible_user_ids_cache(state, node.id);
    Ok(json_value_response(json!({ "success": true })))
}
