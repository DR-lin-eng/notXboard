use crate::*;

pub async fn logs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_logs_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn node_logs(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_node_logs_response(&state, id, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn node_rules(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_node_rules_response(&state, id, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn store_node_rule(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_store_node_rule_response(&state, id, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn update_or_destroy_node_rule(
    State(state): State<Arc<AppState>>,
    axum::extract::Path((node_id, rule_id)): axum::extract::Path<(u64, u64)>,
    headers: HeaderMap,
    uri: Uri,
    request: Request<Body>,
) -> Response<Body> {
    let method = request.method().clone();
    let body = request.into_body();
    let result = match method {
        Method::PUT => build_update_node_rule_response(&state, node_id, rule_id, headers, uri, body).await,
        Method::DELETE => build_destroy_node_rule_response(&state, node_id, rule_id, headers, uri).await,
        _ => Err(json_error(StatusCode::METHOD_NOT_ALLOWED, "method not allowed")),
    };
    match result {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_logs_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let params = parse_query(&uri);
    let limit = params
        .get("limit")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(100)
        .clamp(1, 500);
    let node_filter = params.get("node_id").and_then(|value| value.parse::<u64>().ok());

    let rows = load_user_audit_logs(state, user.id, node_filter, limit)
        .await
        .map_err(internal_error)?;
    Ok(json_value_response(json!({
        "success": true,
        "data": rows.iter().map(serialize_audit_log).collect::<Vec<_>>(),
    })))
}

async fn build_node_logs_response(
    state: &AppState,
    node_id: u64,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let node = load_owned_node_admin_node(state, node_id, user.id).await.map_err(internal_error)?;
    let Some(node) = node else {
        return Ok(json_value_response(json!({
            "success": false,
            "error": "Server node not found"
        })));
    };

    let params = parse_query(&uri);
    let limit = params
        .get("limit")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(100)
        .clamp(1, 500);
    let rows = load_node_owner_audit_logs(state, node.id, limit)
        .await
        .map_err(internal_error)?;

    Ok(json_value_response(json!({
        "success": true,
        "data": rows.iter().map(serialize_audit_log).collect::<Vec<_>>(),
    })))
}

async fn build_node_rules_response(
    state: &AppState,
    node_id: u64,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let node = load_owned_node_admin_node(state, node_id, user.id).await.map_err(internal_error)?;
    let Some(node) = node else {
        return Ok(json_value_response(json!({
            "success": false,
            "error": "Server node not found"
        })));
    };
    let rows = load_node_owner_audit_rules(state, node.id).await.map_err(internal_error)?;
    Ok(json_value_response(json!({
        "success": true,
        "data": rows.iter().map(serialize_audit_rule).collect::<Vec<_>>(),
    })))
}

async fn build_store_node_rule_response(
    state: &AppState,
    node_id: u64,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let node = load_owned_node_admin_node(state, node_id, user.id).await.map_err(internal_error)?;
    let Some(node) = node else {
        return Ok(json_value_response(json!({
            "success": false,
            "error": "Server node not found"
        })));
    };
    let payload = parse_json_body(body).await?;
    let rule_type = payload.get("rule_type").and_then(|value| value.as_str()).map(|value| value.trim()).unwrap_or("");
    let rule_pattern = payload.get("rule_pattern").and_then(|value| value.as_str()).map(|value| value.trim()).unwrap_or("");
    let action = payload.get("action").and_then(|value| value.as_str()).map(|value| value.trim()).unwrap_or("");
    let is_active = payload.get("is_active").and_then(|value| value.as_bool()).unwrap_or(true);
    if !matches!(rule_type, "domain" | "protocol" | "ip") {
        return Ok(json_value_response(json!({"success": false, "error": "Invalid rule type"})));
    }
    if rule_pattern.is_empty() || rule_pattern.chars().count() > 500 {
        return Ok(json_value_response(json!({"success": false, "error": "Invalid rule pattern"})));
    }
    if !matches!(action, "block" | "allow" | "log") {
        return Ok(json_value_response(json!({"success": false, "error": "Invalid action"})));
    }

    sqlx::query(
        "INSERT INTO audit_rules (node_id, rule_type, rule_pattern, action, is_active, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, NOW(), NOW())"
    )
    .bind(node.id)
    .bind(rule_type)
    .bind(rule_pattern)
    .bind(action)
    .bind(if is_active { 1 } else { 0 })
    .execute(&state.db)
    .await
    .map_err(internal_error)?;
    let row = load_latest_node_owner_audit_rule(state, node.id).await.map_err(internal_error)?;
    Ok(json_status_response(StatusCode::CREATED, json!({
        "success": true,
        "data": row.map(|value| serialize_audit_rule(&value)).unwrap_or(Value::Null)
    })))
}

async fn build_update_node_rule_response(
    state: &AppState,
    node_id: u64,
    rule_id: u64,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let node = load_owned_node_admin_node(state, node_id, user.id).await.map_err(internal_error)?;
    let Some(node) = node else {
        return Ok(json_value_response(json!({
            "success": false,
            "error": "Server node not found"
        })));
    };
    let existing = load_node_owner_audit_rule_by_id(state, node.id, rule_id).await.map_err(internal_error)?;
    let Some(existing) = existing else {
        return Ok(json_value_response(json!({
            "success": false,
            "error": "Audit rule not found"
        })));
    };

    let payload = parse_json_body(body).await?;
    let rule_type = payload.get("rule_type").and_then(|value| value.as_str()).map(|value| value.trim().to_string());
    let rule_pattern = payload.get("rule_pattern").and_then(|value| value.as_str()).map(|value| value.trim().to_string());
    let action = payload.get("action").and_then(|value| value.as_str()).map(|value| value.trim().to_string());
    let is_active = payload.get("is_active").and_then(|value| value.as_bool());

    if let Some(value) = rule_type.as_deref() {
        if !matches!(value, "domain" | "protocol" | "ip") {
            return Ok(json_value_response(json!({"success": false, "error": "Invalid rule type"})));
        }
    }
    if let Some(value) = rule_pattern.as_deref() {
        if value.is_empty() || value.chars().count() > 500 {
            return Ok(json_value_response(json!({"success": false, "error": "Invalid rule pattern"})));
        }
    }
    if let Some(value) = action.as_deref() {
        if !matches!(value, "block" | "allow" | "log") {
            return Ok(json_value_response(json!({"success": false, "error": "Invalid action"})));
        }
    }

    sqlx::query(
        "UPDATE audit_rules
         SET rule_type = COALESCE(?, rule_type),
             rule_pattern = COALESCE(?, rule_pattern),
             action = COALESCE(?, action),
             is_active = COALESCE(?, is_active),
             updated_at = NOW()
         WHERE id = ? AND node_id = ?"
    )
    .bind(rule_type)
    .bind(rule_pattern)
    .bind(action)
    .bind(is_active.map(|value| if value { 1 } else { 0 }))
    .bind(existing.id)
    .bind(node.id)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    let refreshed = load_node_owner_audit_rule_by_id(state, node.id, existing.id).await.map_err(internal_error)?;
    Ok(json_value_response(json!({
        "success": true,
        "data": refreshed.map(|value| serialize_audit_rule(&value)).unwrap_or(Value::Null)
    })))
}

async fn build_destroy_node_rule_response(
    state: &AppState,
    node_id: u64,
    rule_id: u64,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let node = load_owned_node_admin_node(state, node_id, user.id).await.map_err(internal_error)?;
    let Some(node) = node else {
        return Ok(json_value_response(json!({
            "success": false,
            "error": "Server node not found"
        })));
    };
    let existing = load_node_owner_audit_rule_by_id(state, node.id, rule_id).await.map_err(internal_error)?;
    if existing.is_none() {
        return Ok(json_value_response(json!({
            "success": false,
            "error": "Audit rule not found"
        })));
    }
    sqlx::query("DELETE FROM audit_rules WHERE id = ? AND node_id = ?")
        .bind(rule_id)
        .bind(node.id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(json_value_response(json!({ "success": true })))
}
