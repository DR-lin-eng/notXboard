use crate::*;

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

pub async fn show(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(trust_level): axum::extract::Path<i64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_show_response(&state, trust_level, headers, uri).await {
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

pub async fn batch_update(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_batch_update_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn destroy(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(trust_level): axum::extract::Path<i64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_destroy_response(&state, trust_level, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn defaults_template(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_defaults_template_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn defaults_apply(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_defaults_apply_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_index_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let rows = sqlx::query_as::<_, AdminGroupLimitRow>(
        "SELECT trust_level, speed_limit_up, speed_limit_down, device_limit, connection_limit, created_at, updated_at
         FROM user_group_limits
         ORDER BY trust_level ASC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(json_value_response(json!({
        "success": true,
        "data": rows.into_iter().map(serialize_admin_group_limit_row).collect::<Vec<_>>()
    })))
}

async fn build_show_response(
    state: &AppState,
    trust_level: i64,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    if !(0..=4).contains(&trust_level) {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({
            "success": false,
            "error": "Invalid trust level. Must be between 0 and 4."
        })));
    }

    let row = sqlx::query_as::<_, AdminGroupLimitRow>(
        "SELECT trust_level, speed_limit_up, speed_limit_down, device_limit, connection_limit, created_at, updated_at
         FROM user_group_limits
         WHERE trust_level = ?
         LIMIT 1"
    )
    .bind(trust_level)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?;

    let Some(row) = row else {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({
            "success": false,
            "error": "Group limit not found"
        })));
    };

    Ok(json_value_response(json!({
        "success": true,
        "data": serialize_admin_group_limit_row(row)
    })))
}

async fn build_store_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let obj = payload.as_object().ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let trust_level = obj.get("trust_level").and_then(parse_i64_value).ok_or_else(|| {
        json_status_response(StatusCode::UNPROCESSABLE_ENTITY, json!({"success": false, "error": "Validation failed"}))
    })?;
    let speed_limit_up = obj.get("speed_limit_up").and_then(parse_i64_value).unwrap_or(0);
    let speed_limit_down = obj.get("speed_limit_down").and_then(parse_i64_value).unwrap_or(0);
    let device_limit = obj.get("device_limit").and_then(parse_i64_value).unwrap_or(0);
    let connection_limit = obj.get("connection_limit").and_then(parse_i64_value).unwrap_or(0);

    validate_admin_group_limit_values(trust_level, speed_limit_up, speed_limit_down, device_limit, connection_limit)?;

    let now = Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO user_group_limits (trust_level, speed_limit_up, speed_limit_down, device_limit, connection_limit, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, FROM_UNIXTIME(?), FROM_UNIXTIME(?))
         ON DUPLICATE KEY UPDATE
           speed_limit_up = VALUES(speed_limit_up),
           speed_limit_down = VALUES(speed_limit_down),
           device_limit = VALUES(device_limit),
           connection_limit = VALUES(connection_limit),
           updated_at = VALUES(updated_at)"
    )
    .bind(trust_level)
    .bind(speed_limit_up)
    .bind(speed_limit_down)
    .bind(device_limit)
    .bind(connection_limit)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(json_value_response(json!({
        "success": true,
        "message": "Group limit updated successfully",
        "data": {
            "trust_level": trust_level,
            "trust_level_name": admin_trust_level_display_name(trust_level),
            "speed_limit_up": speed_limit_up,
            "speed_limit_down": speed_limit_down,
            "device_limit": device_limit,
            "connection_limit": connection_limit,
            "updated_at": now,
        }
    })))
}

async fn build_batch_update_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let obj = payload.as_object().ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    let items = obj.get("limits").and_then(|value| value.as_array()).ok_or_else(|| {
        json_status_response(StatusCode::UNPROCESSABLE_ENTITY, json!({"success": false, "error": "Validation failed"}))
    })?;

    let now = Utc::now().timestamp();
    let mut data = Vec::with_capacity(items.len());
    for item in items {
        let row = item.as_object().ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
        let trust_level = row.get("trust_level").and_then(parse_i64_value).ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
        let speed_limit_up = row.get("speed_limit_up").and_then(parse_i64_value).unwrap_or(0);
        let speed_limit_down = row.get("speed_limit_down").and_then(parse_i64_value).unwrap_or(0);
        let device_limit = row.get("device_limit").and_then(parse_i64_value).unwrap_or(0);
        let connection_limit = row.get("connection_limit").and_then(parse_i64_value).unwrap_or(0);
        validate_admin_group_limit_values(trust_level, speed_limit_up, speed_limit_down, device_limit, connection_limit)?;

        sqlx::query(
            "INSERT INTO user_group_limits (trust_level, speed_limit_up, speed_limit_down, device_limit, connection_limit, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, FROM_UNIXTIME(?), FROM_UNIXTIME(?))
             ON DUPLICATE KEY UPDATE
               speed_limit_up = VALUES(speed_limit_up),
               speed_limit_down = VALUES(speed_limit_down),
               device_limit = VALUES(device_limit),
               connection_limit = VALUES(connection_limit),
               updated_at = VALUES(updated_at)"
        )
        .bind(trust_level)
        .bind(speed_limit_up)
        .bind(speed_limit_down)
        .bind(device_limit)
        .bind(connection_limit)
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

        data.push(json!({
            "trust_level": trust_level,
            "trust_level_name": admin_trust_level_display_name(trust_level),
            "speed_limit_up": speed_limit_up,
            "speed_limit_down": speed_limit_down,
            "device_limit": device_limit,
            "connection_limit": connection_limit,
            "updated_at": now,
        }));
    }

    Ok(json_value_response(json!({
        "success": true,
        "message": "Group limits updated successfully",
        "data": data
    })))
}

async fn build_destroy_response(
    state: &AppState,
    trust_level: i64,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    if !(0..=4).contains(&trust_level) {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({
            "success": false,
            "error": "Invalid trust level. Must be between 0 and 4."
        })));
    }

    let deleted = sqlx::query("DELETE FROM user_group_limits WHERE trust_level = ?")
        .bind(trust_level)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    if deleted.rows_affected() == 0 {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({
            "success": false,
            "error": "Group limit not found"
        })));
    }

    Ok(json_value_response(json!({
        "success": true,
        "message": "Group limit deleted successfully"
    })))
}

async fn build_defaults_template_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    Ok(json_value_response(json!({
        "success": true,
        "data": admin_group_limit_defaults()
    })))
}

async fn build_defaults_apply_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    _body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let now = Utc::now().timestamp();
    let defaults = admin_group_limit_defaults();

    for row in &defaults {
        let trust_level = row.get("trust_level").and_then(Value::as_i64).unwrap_or(0);
        let speed_limit_up = row.get("speed_limit_up").and_then(Value::as_i64).unwrap_or(0);
        let speed_limit_down = row.get("speed_limit_down").and_then(Value::as_i64).unwrap_or(0);
        let device_limit = row.get("device_limit").and_then(Value::as_i64).unwrap_or(0);
        let connection_limit = row.get("connection_limit").and_then(Value::as_i64).unwrap_or(0);

        sqlx::query(
            "INSERT INTO user_group_limits (trust_level, speed_limit_up, speed_limit_down, device_limit, connection_limit, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, FROM_UNIXTIME(?), FROM_UNIXTIME(?))
             ON DUPLICATE KEY UPDATE
               speed_limit_up = VALUES(speed_limit_up),
               speed_limit_down = VALUES(speed_limit_down),
               device_limit = VALUES(device_limit),
               connection_limit = VALUES(connection_limit),
               updated_at = VALUES(updated_at)"
        )
        .bind(trust_level)
        .bind(speed_limit_up)
        .bind(speed_limit_down)
        .bind(device_limit)
        .bind(connection_limit)
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    }

    Ok(json_value_response(json!({
        "success": true,
        "message": "Default group limits applied successfully",
        "data": defaults.into_iter().map(|mut item| {
            if let Some(object) = item.as_object_mut() {
                object.insert("updated_at".to_string(), Value::from(now));
            }
            item
        }).collect::<Vec<_>>()
    })))
}
