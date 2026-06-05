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

pub async fn cast(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_cast_response(&state, id, headers, uri, body).await {
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
    ensure_refund_dispute_enabled(state).await?;
    let cache_key = build_user_cache_key(&uri, user.id);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }

    let items = load_voting_refund_requests(state).await.map_err(internal_error)?;
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
    ensure_refund_dispute_enabled(state).await?;
    let cache_key = build_user_cache_key(&uri, user.id);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }

    let req = load_refund_request_detail_any(state, refund_id)
        .await
        .map_err(internal_error)?;
    let Some(req) = req else {
        return Ok(fail_json_response(StatusCode::NOT_FOUND, "Refund request not found"));
    };

    let evidences = load_refund_evidences(state, refund_id).await.map_err(internal_error)?;
    let votes = load_refund_votes(state, refund_id).await.map_err(internal_error)?;
    let data = serialize_refund_request_detail(&req, &evidences, &votes, Some(user.id as u64));
    Ok(success_cached_response(state, cache_key, data, Duration::from_secs(5)))
}

async fn build_cast_response(
    state: &AppState,
    refund_id: u64,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    ensure_refund_dispute_enabled(state).await?;
    let payload = parse_json_body(body).await?;
    let vote = payload
        .get("vote")
        .and_then(|value| value.as_str())
        .map(|value| value.trim())
        .unwrap_or("");
    if !matches!(vote, "approve" | "deny") {
        return Ok(fail_json_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "The vote field must be either approve or deny.",
        ));
    }

    let req = load_refund_request_detail_any(state, refund_id)
        .await
        .map_err(internal_error)?;
    let Some(req) = req else {
        return Ok(fail_json_response(StatusCode::NOT_FOUND, "Refund request not found"));
    };
    if req.status != "voting" {
        return Ok(fail_json_response(
            StatusCode::BAD_REQUEST,
            "Refund request is not open for voting",
        ));
    }
    if let Some(voting_ends_at) = req.voting_ends_at {
        if chrono::Utc::now() > voting_ends_at {
            return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Voting has ended"));
        }
    }
    if user.banned != 0 {
        return Ok(fail_json_response(StatusCode::FORBIDDEN, "Permission denied"));
    }

    sqlx::query(
        "INSERT INTO order_refund_votes (refund_request_id, user_id, vote, created_at, updated_at)
         VALUES (?, ?, ?, NOW(), NOW())
         ON DUPLICATE KEY UPDATE vote = VALUES(vote), updated_at = VALUES(updated_at)"
    )
    .bind(refund_id)
    .bind(user.id as u64)
    .bind(vote)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    if let Err(err) = notify_refund_vote_cast(state, refund_id, user.id as u64, vote).await {
        tracing::warn!(
            refund_id,
            voter_user_id = user.id,
            vote,
            error = %err,
            "refund vote cast telegram notify failed after vote persisted"
        );
    }

    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_evidence_response(
    state: &AppState,
    refund_id: u64,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    ensure_refund_dispute_enabled(state).await?;
    let payload = parse_json_body(body).await?;
    let content = payload
        .get("content")
        .and_then(|value| value.as_str())
        .map(|value| value.trim())
        .unwrap_or("");
    if content.is_empty() {
        return Ok(fail_json_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "The content field is required.",
        ));
    }
    if content.chars().count() > 5000 {
        return Ok(fail_json_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "The content field must not be greater than 5000 characters.",
        ));
    }

    let req = load_refund_request_detail_any(state, refund_id)
        .await
        .map_err(internal_error)?;
    let Some(req) = req else {
        return Ok(fail_json_response(StatusCode::NOT_FOUND, "Refund request not found"));
    };
    if req.status != "voting" {
        return Ok(fail_json_response(
            StatusCode::BAD_REQUEST,
            "Refund request is not open for voting",
        ));
    }
    if let Some(voting_ends_at) = req.voting_ends_at {
        if chrono::Utc::now() > voting_ends_at {
            return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Voting has ended"));
        }
    }
    if user.banned != 0 {
        return Ok(fail_json_response(StatusCode::FORBIDDEN, "Permission denied"));
    }

    sqlx::query(
        "INSERT INTO order_refund_evidences
            (refund_request_id, user_id, role, content, created_at, updated_at)
         VALUES (?, ?, 'user', ?, NOW(), NOW())"
    )
    .bind(refund_id)
    .bind(user.id as u64)
    .bind(content)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn ensure_refund_dispute_enabled(state: &AppState) -> Result<(), Response<Body>> {
    if !get_setting_bool(state, "refund_dispute_enable", false).await {
        return Err(fail_json_response(StatusCode::FORBIDDEN, "争议退款功能已关闭"));
    }
    Ok(())
}
