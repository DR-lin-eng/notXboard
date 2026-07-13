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

pub async fn detail(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_detail_response(&state, id, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn approve(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_approve_response(&state, id, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn deny(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_deny_response(&state, id, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn dispute(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_dispute_response(&state, id, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn evidence(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_evidence_response(&state, id, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_index_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let cache_key = build_user_cache_key(&uri, user.id);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }

    let items = load_assigned_admin_refund_requests(state, user.id as u64)
        .await
        .map_err(internal_error)?;
    let data = items
        .iter()
        .map(serialize_refund_request_summary)
        .collect::<Vec<_>>();
    Ok(success_cached_response(
        state,
        cache_key,
        Value::Array(data),
        Duration::from_secs(5),
    ))
}

async fn build_detail_response(
    state: &AppState,
    refund_id: u64,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let cache_key = build_user_cache_key(&uri, user.id);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }

    let req = load_assigned_admin_refund_request_detail(state, refund_id, user.id as u64)
        .await
        .map_err(internal_error)?;
    let Some(req) = req else {
        return Ok(fail_json_response(StatusCode::NOT_FOUND, "Refund request not found"));
    };
    let evidences = load_refund_evidences(state, refund_id).await.map_err(internal_error)?;
    let votes = load_refund_votes(state, refund_id).await.map_err(internal_error)?;
    let data = serialize_refund_request_detail(&req, &evidences, &votes, None);
    Ok(success_cached_response(state, cache_key, data, Duration::from_secs(5)))
}

async fn build_deny_response(
    state: &AppState,
    refund_id: u64,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let reason = payload
        .get("reason")
        .and_then(|value| value.as_str())
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());
    if let Some(reason) = &reason {
        if reason.chars().count() > 2000 {
            return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "The reason field must not be greater than 2000 characters."));
        }
    }

    let req = load_assigned_admin_refund_request_detail(state, refund_id, user.id as u64)
        .await
        .map_err(internal_error)?;
    let Some(_) = req else {
        return Ok(fail_json_response(StatusCode::NOT_FOUND, "Refund request not found"));
    };

    deny_refund_request_as_admin(
        state,
        refund_id,
        user.id as u64,
        reason.clone(),
        Some(user.id as u64),
    )
    .await?;
    let _ = notify_refund_status_changed(state, refund_id).await;
    let req = load_assigned_admin_refund_request_detail(state, refund_id, user.id as u64)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::NOT_FOUND, "Refund request not found"))?;
    Ok(json_value_response(success_response_payload(
        serialize_refund_request_summary(&req),
    )))
}

async fn build_approve_response(
    state: &AppState,
    refund_id: u64,
    headers: HeaderMap,
    _uri: Uri,
    _body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let req = load_assigned_admin_refund_request_detail(state, refund_id, user.id as u64)
        .await
        .map_err(internal_error)?;
    let Some(_) = req else {
        return Ok(fail_json_response(StatusCode::NOT_FOUND, "Refund request not found"));
    };

    approve_refund_request_as_admin(
        state,
        refund_id,
        user.id as u64,
        Some(user.id as u64),
    )
    .await?;
    let _ = notify_refund_status_changed(state, refund_id).await;
    let req = load_assigned_admin_refund_request_detail(state, refund_id, user.id as u64)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::NOT_FOUND, "Refund request not found"))?;
    Ok(json_value_response(success_response_payload(
        serialize_refund_request_summary(&req),
    )))
}

async fn build_dispute_response(
    state: &AppState,
    refund_id: u64,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    if !get_setting_bool(state, "refund_dispute_enable", false).await {
        return Ok(fail_json_response(StatusCode::FORBIDDEN, "争议退款功能已关闭"));
    }

    let payload = parse_json_body(body).await?;
    let minutes = payload
        .get("minutes")
        .and_then(parse_i64_value)
        .unwrap_or(1440);
    if !(10..=10080).contains(&minutes) {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "The minutes field must be between 10 and 10080."));
    }
    let evidence = payload
        .get("evidence")
        .and_then(|value| value.as_str())
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string());
    if let Some(evidence) = &evidence {
        if evidence.chars().count() > 5000 {
            return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "The evidence field must not be greater than 5000 characters."));
        }
    }

    let req = load_assigned_admin_refund_request_detail(state, refund_id, user.id as u64)
        .await
        .map_err(internal_error)?;
    let Some(_) = req else {
        return Ok(fail_json_response(StatusCode::NOT_FOUND, "Refund request not found"));
    };

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let locked = load_assigned_admin_refund_request_detail_for_update(&mut tx, refund_id, user.id as u64)
        .await
        .map_err(internal_error)?;
    let Some(locked) = locked else {
        tx.rollback().await.ok();
        return Ok(fail_json_response(StatusCode::NOT_FOUND, "Refund request not found"));
    };
    if locked.status != "pending" {
        tx.rollback().await.ok();
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Only pending requests can enter voting"));
    }

    if let Some(evidence) = &evidence {
        sqlx::query(
            "INSERT INTO order_refund_evidences
                (refund_request_id, user_id, role, content, created_at, updated_at)
             VALUES (?, ?, 'admin', ?, NOW(), NOW())"
        )
        .bind(refund_id)
        .bind(user.id as u64)
        .bind(evidence)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    }

    let updated = sqlx::query(
        "UPDATE order_refund_requests
         SET status = 'voting',
             voting_ends_at = DATE_ADD(NOW(), INTERVAL ? MINUTE),
             updated_at = NOW()
         WHERE id = ? AND assigned_admin_user_id = ? AND status = 'pending'"
    )
    .bind(minutes)
    .bind(refund_id)
    .bind(user.id as u64)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;
    if updated.rows_affected() != 1 {
        tx.rollback().await.ok();
        return Ok(fail_json_response(StatusCode::CONFLICT, "Refund assignment changed"));
    }

    tx.commit().await.map_err(internal_error)?;
    let _ = notify_refund_vote_started(state, refund_id).await;
    let req = load_assigned_admin_refund_request_detail(state, refund_id, user.id as u64)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::NOT_FOUND, "Refund request not found"))?;
    Ok(json_value_response(success_response_payload(
        serialize_refund_request_summary(&req),
    )))
}

async fn build_evidence_response(
    state: &AppState,
    refund_id: u64,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let content = payload
        .get("content")
        .and_then(|value| value.as_str())
        .map(|value| value.trim())
        .unwrap_or("");
    if content.is_empty() {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "The content field is required."));
    }
    if content.chars().count() > 5000 {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "The content field must not be greater than 5000 characters."));
    }

    let req = load_assigned_admin_refund_request_detail(state, refund_id, user.id as u64)
        .await
        .map_err(internal_error)?;
    let Some(_) = req else {
        return Ok(fail_json_response(StatusCode::NOT_FOUND, "Refund request not found"));
    };

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let locked = load_assigned_admin_refund_request_detail_for_update(
        &mut tx,
        refund_id,
        user.id as u64,
    )
    .await
    .map_err(internal_error)?;
    if locked.is_none() {
        tx.rollback().await.ok();
        return Ok(fail_json_response(StatusCode::NOT_FOUND, "Refund request not found"));
    }

    sqlx::query(
        "INSERT INTO order_refund_evidences
            (refund_request_id, user_id, role, content, created_at, updated_at)
         VALUES (?, ?, 'admin', ?, NOW(), NOW())"
    )
    .bind(refund_id)
    .bind(user.id as u64)
    .bind(content)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;
    tx.commit().await.map_err(internal_error)?;

    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}
