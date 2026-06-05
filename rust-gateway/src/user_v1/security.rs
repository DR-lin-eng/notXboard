use crate::*;

pub async fn get_active_session(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_active_session_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_quick_login_url(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_get_quick_login_url_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn remove_active_session(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_remove_active_session_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn reset_security(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_reset_security_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn change_password(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_change_password_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_get_active_session_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let cache_key = build_user_cache_key(&uri, user.id);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }
    let rows = sqlx::query(
        "SELECT id, tokenable_type, tokenable_id, name, abilities, last_used_at, expires_at, created_at, updated_at
         FROM personal_access_tokens
         WHERE tokenable_id = ?
         ORDER BY id DESC"
    )
    .bind(user.id as u64)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    let sessions = rows
        .into_iter()
        .map(|row| {
            let abilities = row
                .try_get::<String, _>("abilities")
                .ok()
                .and_then(|value| serde_json::from_str::<Value>(&value).ok())
                .unwrap_or(Value::Array(vec![]));
            json!({
                "id": row.try_get::<u64, _>("id").unwrap_or_default(),
                "tokenable_type": row.try_get::<String, _>("tokenable_type").unwrap_or_default(),
                "tokenable_id": row.try_get::<u64, _>("tokenable_id").unwrap_or_default(),
                "name": row.try_get::<String, _>("name").unwrap_or_default(),
                "abilities": abilities,
                "last_used_at": row.try_get::<Option<chrono::NaiveDateTime>, _>("last_used_at").ok().flatten().map(|v| v.format("%Y-%m-%dT%H:%M:%S.000000Z").to_string()),
                "expires_at": row.try_get::<Option<chrono::NaiveDateTime>, _>("expires_at").ok().flatten().map(|v| v.format("%Y-%m-%dT%H:%M:%S.000000Z").to_string()),
                "created_at": row.try_get::<Option<chrono::NaiveDateTime>, _>("created_at").ok().flatten().map(|v| v.format("%Y-%m-%dT%H:%M:%S.000000Z").to_string()),
                "updated_at": row.try_get::<Option<chrono::NaiveDateTime>, _>("updated_at").ok().flatten().map(|v| v.format("%Y-%m-%dT%H:%M:%S.000000Z").to_string()),
            })
        })
        .collect::<Vec<_>>();

    Ok(success_cached_response(state, cache_key, Value::Array(sessions), Duration::from_secs(5)))
}

async fn build_get_quick_login_url_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let redirect = payload.get("redirect").and_then(|v| v.as_str()).map(|v| v.trim()).filter(|v| !v.is_empty()).unwrap_or("dashboard");
    let code = Uuid::new_v4().simple().to_string();
    let cache_key = format!("TEMP_TOKEN_{}", code);
    redis_setex_string(state, &cache_key, 60, &user.id.to_string())
        .await
        .map_err(|err| {
            error!("user getQuickLoginUrl cache write failed: {}", err);
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "user getQuickLoginUrl cache write failed")
        })?;
    let app_url = get_setting_string(state, "app_url", "").await;
    let login_redirect = format!("/app/#/login?verify={}&redirect={}", code, urlencoding::encode(redirect));
    let url = if !app_url.trim().is_empty() {
        format!("{}{}", app_url.trim_end_matches('/'), login_redirect)
    } else {
        login_redirect
    };
    Ok(json_value_response(success_response_payload(Value::String(url))))
}

async fn build_remove_active_session_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let session_id = payload.get("session_id").and_then(|v| v.as_i64()).unwrap_or_default();
    sqlx::query("DELETE FROM personal_access_tokens WHERE id = ? AND tokenable_id = ?")
        .bind(session_id as u64)
        .bind(user.id as u64)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_reset_security_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    reset_single_user_security(state, user.id)
        .await
        .map_err(internal_error)?;

    let updated_user = load_bearer_user_by_id(state, user.id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::BAD_REQUEST, "The user does not exist"))?;
    let subscribe_url = build_user_subscribe_url(state, &updated_user).await.unwrap_or_default();
    Ok(json_value_response(success_response_payload(Value::String(subscribe_url))))
}

async fn build_change_password_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let bearer_user = authenticate_bearer_user(state, &headers).await?;
    let login_user = load_login_user_by_id(state, bearer_user.id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::BAD_REQUEST, "The user does not exist"))?;

    let payload = parse_json_body(body).await?;
    let old_password = payload.get("old_password").and_then(|v| v.as_str()).map(|v| v.to_string()).unwrap_or_default();
    let new_password = payload.get("new_password").and_then(|v| v.as_str()).map(|v| v.to_string()).unwrap_or_default();

    if old_password.is_empty() {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Old password cannot be empty"));
    }
    if new_password.is_empty() {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "New password cannot be empty"));
    }
    if new_password.chars().count() < 8 {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Password must be greater than 8 digits"));
    }
    if !verify_login_password(&login_user, &old_password) {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "The old password is wrong"));
    }

    update_user_password(state, login_user.id, &new_password).await.map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}
