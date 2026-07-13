use crate::*;

pub async fn finalize(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_finalize_response(&state, headers, uri, body).await {
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

pub async fn detail(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(refund_id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_detail_response(&state, refund_id, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn approve(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(refund_id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_approve_response(&state, refund_id, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn deny(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(refund_id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_deny_response(&state, refund_id, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_finalize_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let limit = payload
        .get("limit")
        .and_then(parse_i64_value)
        .unwrap_or(500)
        .max(1)
        .min(5000);

    let now = chrono::Utc::now();
    let requests = load_expired_voting_refund_requests(state, now, limit)
        .await
        .map_err(internal_error)?;

    let mut processed = 0_i64;
    for req in requests {
        finalize_single_expired_refund_voting(state, req.id, now)
            .await
            .map_err(internal_error)?;
        let _ = notify_refund_status_changed(state, req.id).await;
        processed += 1;
    }

    Ok(json_value_response(success_response_payload(json!({
        "processed": processed
    }))))
}

async fn build_index_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let items = load_all_refund_requests(state, 500).await.map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Array(
        items.iter().map(serialize_refund_request_summary).collect::<Vec<_>>()
    ))))
}

async fn build_detail_response(
    state: &AppState,
    refund_id: u64,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let req = load_refund_request_detail_any(state, refund_id)
        .await
        .map_err(internal_error)?;
    let Some(req) = req else {
        return Ok(fail_json_response(StatusCode::NOT_FOUND, "Refund request not found"));
    };
    let evidences = load_refund_evidences(state, refund_id).await.map_err(internal_error)?;
    let votes = load_refund_votes(state, refund_id).await.map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(
        serialize_refund_request_detail(&req, &evidences, &votes, None),
    )))
}

async fn build_approve_response(
    state: &AppState,
    refund_id: u64,
    headers: HeaderMap,
    _uri: Uri,
    _body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let admin = authenticate_super_admin_user(state, &headers).await?;
    let req = load_refund_request_detail_any(state, refund_id)
        .await
        .map_err(internal_error)?;
    let Some(_) = req else {
        return Ok(fail_json_response(StatusCode::NOT_FOUND, "Refund request not found"));
    };
    approve_refund_request_as_admin(state, refund_id, admin.id as u64, None).await?;
    let _ = notify_refund_status_changed(state, refund_id).await;
    let req = load_refund_request_detail_any(state, refund_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::NOT_FOUND, "Refund request not found"))?;
    Ok(json_value_response(success_response_payload(
        serialize_refund_request_summary(&req),
    )))
}

async fn build_deny_response(
    state: &AppState,
    refund_id: u64,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let admin = authenticate_super_admin_user(state, &headers).await?;
    let req = load_refund_request_detail_any(state, refund_id)
        .await
        .map_err(internal_error)?;
    let Some(_) = req else {
        return Ok(fail_json_response(StatusCode::NOT_FOUND, "Refund request not found"));
    };
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
    deny_refund_request_as_admin(state, refund_id, admin.id as u64, reason, None).await?;
    let _ = notify_refund_status_changed(state, refund_id).await;
    let req = load_refund_request_detail_any(state, refund_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::NOT_FOUND, "Refund request not found"))?;
    Ok(json_value_response(success_response_payload(
        serialize_refund_request_summary(&req),
    )))
}
