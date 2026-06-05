use crate::*;

pub async fn update(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_update_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn transfer(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_transfer_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn coupon_check(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_coupon_check_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn payment_profile_show_epay(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_payment_profile_show_epay_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn payment_profile_upsert_epay(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_payment_profile_upsert_epay_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn api_key_show(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_api_key_show_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn api_key_generate(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_api_key_generate_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn api_key_reset(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_api_key_reset_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn api_key_validate(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_api_key_validate_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_coupon_check_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let code = payload
        .get("code")
        .and_then(|value| value.as_str())
        .map(|value| value.trim())
        .unwrap_or("");
    if code.is_empty() {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Coupon cannot be empty"));
    }

    let plan_id = payload.get("plan_id").and_then(parse_i64_value);
    let period = payload
        .get("period")
        .and_then(|value| value.as_str())
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .and_then(normalize_order_period);

    let coupon = load_coupon_by_code(state, code).await.map_err(internal_error)?;
    let Some(coupon) = coupon else {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Invalid coupon"));
    };

    validate_coupon_for_user(state, &coupon, user.id, plan_id, period).await?;

    Ok(json_value_response(success_response_payload(serialize_coupon(&coupon))))
}

async fn build_payment_profile_show_epay_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let row = sqlx::query(
        "SELECT pid, url, submit_path, use_post, sitename, device, key_encrypted
         FROM user_payment_profiles
         WHERE user_id = ? AND provider = 'epay'
         LIMIT 1"
    )
    .bind(user.id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?;

    if let Some(row) = row {
        let has_key = row.try_get::<Option<String>, _>("key_encrypted").ok().flatten().map(|v| !v.trim().is_empty()).unwrap_or(false);
        return Ok(json_value_response(success_response_payload(json!({
            "pid": row.try_get::<Option<String>, _>("pid").ok().flatten(),
            "url": row.try_get::<Option<String>, _>("url").ok().flatten(),
            "submit_path": row.try_get::<Option<String>, _>("submit_path").ok().flatten(),
            "use_post": row.try_get::<Option<bool>, _>("use_post").ok().flatten().unwrap_or(true),
            "sitename": row.try_get::<Option<String>, _>("sitename").ok().flatten(),
            "device": row.try_get::<Option<String>, _>("device").ok().flatten(),
            "has_key": has_key,
        }))));
    }

    Ok(json_value_response(success_response_payload(json!({
        "pid": null,
        "url": null,
        "submit_path": null,
        "use_post": true,
        "sitename": null,
        "device": null,
        "has_key": false,
    }))))
}

async fn build_payment_profile_upsert_epay_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let pid = payload.get("pid").and_then(|v| v.as_str()).map(|v| v.trim()).filter(|v| !v.is_empty()).unwrap_or("");
    let _key = payload.get("key").and_then(|v| v.as_str()).map(|v| v.trim()).filter(|v| !v.is_empty()).unwrap_or("");
    let url = payload.get("url").and_then(|v| v.as_str()).map(|v| v.trim()).filter(|v| !v.is_empty()).unwrap_or("");
    if pid.is_empty() || url.is_empty() {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    }
    let url = crate::url_security_support::normalize_http_url(url, true)
        .map_err(|message| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, &message))?;
    let submit_path = crate::url_security_support::normalize_relative_path(
        payload.get("submit_path").and_then(|v| v.as_str()),
        "/pay/submit.php",
    )
    .map_err(|message| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, &message))?;
    let use_post = payload.get("use_post").and_then(|v| v.as_bool()).unwrap_or(true);
    let sitename = payload.get("sitename").and_then(|v| v.as_str()).map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
    let device = payload.get("device").and_then(|v| v.as_str()).map(|v| v.trim().to_string()).filter(|v| !v.is_empty());

    let existing = sqlx::query("SELECT id FROM user_payment_profiles WHERE user_id = ? AND provider = 'epay' LIMIT 1")
        .bind(user.id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?;

    if let Some(row) = existing {
        let id: u64 = row.try_get("id").unwrap_or_default();
        sqlx::query(
            "UPDATE user_payment_profiles
             SET pid = ?, url = ?, submit_path = ?, use_post = ?, sitename = ?, device = ?, updated_at = NOW()
             WHERE id = ?"
        )
        .bind(pid)
        .bind(&url)
        .bind(&submit_path)
        .bind(use_post)
        .bind(sitename.as_deref())
        .bind(device.as_deref())
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    } else {
        sqlx::query(
            "INSERT INTO user_payment_profiles (user_id, provider, pid, url, submit_path, use_post, sitename, device, created_at, updated_at)
             VALUES (?, 'epay', ?, ?, ?, ?, ?, ?, NOW(), NOW())"
        )
        .bind(user.id)
        .bind(pid)
        .bind(&url)
        .bind(&submit_path)
        .bind(use_post)
        .bind(sitename.as_deref())
        .bind(device.as_deref())
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    }

    Ok(json_value_response(success_response_payload(json!({
        "pid": pid,
        "url": url,
        "submit_path": submit_path,
        "use_post": use_post,
        "sitename": sitename,
        "device": device,
        "has_key": false,
    }))))
}

async fn build_api_key_show_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let row = sqlx::query(
        "SELECT api_key, created_at, last_login_at, banned
         FROM v2_user
         WHERE id = ?
         LIMIT 1"
    )
    .bind(user.id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?;

    let Some(row) = row else {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "The user does not exist"));
    };

    let api_key = row.try_get::<Option<String>, _>("api_key").ok().flatten();
    let data = json!({
        "api_key": api_key.clone(),
        "api_key_prefix": api_key.as_ref().map(|value| api_key_prefix(value)),
        "has_api_key": api_key.as_ref().map(|value| !value.is_empty()).unwrap_or(false),
        "created_at": row.try_get::<Option<i64>, _>("created_at").ok().flatten(),
        "last_login_at": row.try_get::<Option<i64>, _>("last_login_at").ok().flatten(),
        "is_active": row.try_get::<Option<i8>, _>("banned").ok().flatten().unwrap_or(0) == 0,
    });
    Ok(json_value_response(success_response_payload(data)))
}

async fn build_api_key_generate_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let payload = parse_json_body(body).await.unwrap_or(Value::Object(Map::new()));
    let force = payload.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
    let existing_key = load_user_api_key(state, user.id).await.map_err(internal_error)?;
    if existing_key.as_deref().map(|v| !v.is_empty()).unwrap_or(false) && !force {
        return Ok(json_value_response(json!({
            "success": false,
            "error": "API key already exists. Use reset endpoint to generate a new one.",
            "has_existing_key": true
        })));
    }
    let api_key = ensure_user_api_key(state, &user).await.map_err(internal_error)?;
    Ok(json_value_response(json!({
        "success": true,
        "message": "API key generated successfully",
        "data": {
            "api_key": api_key,
            "api_key_prefix": api_key_prefix(&api_key),
            "generated_at": Utc::now().to_rfc3339(),
        }
    })))
}

async fn build_api_key_reset_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    _body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let current_key = load_user_api_key(state, user.id).await.map_err(internal_error)?;
    let old_prefix = current_key
        .as_ref()
        .map(|value| api_key_prefix(value))
        .unwrap_or_else(|| "none".to_string());
    let new_api_key = reset_user_api_key(state, user.id).await.map_err(internal_error)?;
    Ok(json_value_response(json!({
        "success": true,
        "message": "API key reset successfully",
        "data": {
            "api_key": new_api_key,
            "api_key_prefix": api_key_prefix(&new_api_key),
            "old_key_prefix": old_prefix,
            "action": "reset",
            "reset_at": Utc::now().to_rfc3339(),
        }
    })))
}

async fn build_api_key_validate_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let api_key = payload.get("api_key").and_then(|v| v.as_str()).map(|v| v.trim()).unwrap_or("");
    if api_key.is_empty() {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    }
    if !api_key_has_valid_format(api_key) {
        return Ok(json_value_response(json!({
            "success": true,
            "valid": false,
            "message": "Invalid API key"
        })));
    }
    let row = sqlx::query("SELECT api_key, banned FROM v2_user WHERE id = ? LIMIT 1")
        .bind(user.id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?;
    let Some(row) = row else {
        return Ok(json_value_response(json!({
            "success": true,
            "valid": false,
            "message": "Invalid API key"
        })));
    };
    let stored_key = row.try_get::<Option<String>, _>("api_key").ok().flatten().unwrap_or_default();
    let is_active = row.try_get::<i8, _>("banned").unwrap_or(0) == 0;
    if stored_key.is_empty() || !is_active || !crate::secure_compare_support::constant_time_eq(stored_key.as_bytes(), api_key.as_bytes()) {
        return Ok(json_value_response(json!({
            "success": true,
            "valid": false,
            "message": "Invalid API key"
        })));
    }
    Ok(json_value_response(json!({
        "success": true,
        "valid": true,
        "data": {
            "belongs_to_current_user": true,
            "is_active": is_active,
        }
    })))
}

async fn build_update_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let remind_expire = payload.get("remind_expire").and_then(|v| v.as_i64());
    let remind_traffic = payload.get("remind_traffic").and_then(|v| v.as_i64());
    if let Some(value) = remind_expire {
        if value != 0 && value != 1 {
            return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Incorrect format of expiration reminder"));
        }
    }
    if let Some(value) = remind_traffic {
        if value != 0 && value != 1 {
            return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Incorrect traffic alert format"));
        }
    }
    sqlx::query("UPDATE v2_user SET remind_expire = COALESCE(?, remind_expire), remind_traffic = COALESCE(?, remind_traffic), updated_at = ? WHERE id = ?")
        .bind(remind_expire)
        .bind(remind_traffic)
        .bind(Utc::now().timestamp())
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_transfer_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let transfer_amount = payload
        .get("transfer_amount")
        .and_then(parse_i64_value)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "The transfer amount cannot be empty"))?;
    if transfer_amount < 1 {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "The transfer amount parameter is wrong"));
    }

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let row = sqlx::query("SELECT commission_balance, balance FROM v2_user WHERE id = ? LIMIT 1 FOR UPDATE")
        .bind(user.id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(internal_error)?;
    let Some(row) = row else {
        tx.rollback().await.ok();
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "The user does not exist"));
    };

    let commission_balance = row.try_get::<i64, _>("commission_balance").unwrap_or_default();
    let balance = row.try_get::<i64, _>("balance").unwrap_or_default();
    if commission_balance < transfer_amount {
        tx.rollback().await.ok();
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Insufficient commission balance"));
    }

    let updated = sqlx::query(
        "UPDATE v2_user
         SET commission_balance = ?, balance = ?, updated_at = ?
         WHERE id = ?"
    )
    .bind(commission_balance - transfer_amount)
    .bind(balance + transfer_amount)
    .bind(Utc::now().timestamp())
    .bind(user.id)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;

    if updated.rows_affected() == 0 {
        tx.rollback().await.ok();
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Transfer failed"));
    }

    tx.commit().await.map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}
