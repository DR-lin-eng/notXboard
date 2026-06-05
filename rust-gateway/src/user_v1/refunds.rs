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

pub async fn create(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_create_response(&state, headers, uri, body).await {
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

    let items = load_user_refund_requests(state, user.id)
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

async fn build_create_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let trade_no = payload
        .get("trade_no")
        .and_then(|value| value.as_str())
        .map(|value| value.trim())
        .unwrap_or("");
    if trade_no.is_empty() {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "The trade no field is required."));
    }
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

    let order = load_refund_create_order_by_trade_no(state, user.id, trade_no)
        .await
        .map_err(internal_error)?;
    let Some(order) = order else {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Order not found"));
    };
    let plan = load_refund_create_plan_by_id(state, order.plan_id)
        .await
        .map_err(internal_error)?;
    let Some(plan) = plan else {
        return Ok(fail_json_response(StatusCode::INTERNAL_SERVER_ERROR, "Plan not found"));
    };
    if plan.scope.trim() != "node" {
        return Ok(fail_json_response(
            StatusCode::BAD_REQUEST,
            "Refunds are only supported for node plans currently",
        ));
    }
    if order.status != 3 {
        return Ok(fail_json_response(
            StatusCode::BAD_REQUEST,
            "Only completed orders can be refunded",
        ));
    }

    let (used_kb, allowance_kb) = calculate_refund_usage_kb(state, &order, &plan)
        .await
        .map_err(internal_error)?;
    let (refund_cents, charged_cents) =
        calculate_refund_cents(&order, used_kb, allowance_kb);
    let gateway_amount = order.total_amount + order.handling_amount.unwrap_or(0);
    let assigned_admin = plan.owner_user_id;

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let existing = sqlx::query_scalar::<_, u64>(
        "SELECT id FROM order_refund_requests WHERE order_id = ? LIMIT 1 FOR UPDATE"
    )
    .bind(order.id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal_error)?;

    let refund_id = if let Some(existing_id) = existing {
        existing_id
    } else {
        let result = sqlx::query(
            "INSERT INTO order_refund_requests
                (order_id, trade_no, user_id, plan_id, assigned_admin_user_id, status, reason, gateway_amount,
                 gateway_trade_no, epay_pid, epay_url, epay_key_encrypted, used_kb, allowance_kb, refund_amount,
                 charged_amount, balance_refunded_amount, gateway_refunded_amount, site_balance_fallback_amount,
                 gateway_refund_pending_amount, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, 'pending', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0, 0, 0, NOW(), NOW())"
        )
        .bind(order.id)
        .bind(&order.trade_no)
        .bind(order.user_id)
        .bind(order.plan_id)
        .bind(assigned_admin)
        .bind(reason.clone())
        .bind(gateway_amount)
        .bind(order.callback_no.clone())
        .bind(order.epay_pid.clone())
        .bind(order.epay_url.clone())
        .bind(order.epay_key_encrypted.clone())
        .bind(used_kb)
        .bind(allowance_kb)
        .bind(refund_cents)
        .bind(charged_cents)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
        let new_id = result.last_insert_id();
        if let Some(evidence) = &evidence {
            sqlx::query(
                "INSERT INTO order_refund_evidences
                    (refund_request_id, user_id, role, content, created_at, updated_at)
                 VALUES (?, ?, 'user', ?, NOW(), NOW())"
            )
            .bind(new_id)
            .bind(user.id as u64)
            .bind(evidence)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
        }
        new_id
    };

    tx.commit().await.map_err(internal_error)?;
    let req = load_user_refund_request_detail(state, refund_id, user.id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::INTERNAL_SERVER_ERROR, "Refund request not found"))?;
    Ok(json_value_response(success_response_payload(
        serialize_refund_request_summary(&req),
    )))
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

    let req = load_user_refund_request_detail(state, refund_id, user.id)
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

    let req = load_user_refund_request_detail(state, refund_id, user.id)
        .await
        .map_err(internal_error)?;
    let Some(_) = req else {
        return Ok(fail_json_response(StatusCode::NOT_FOUND, "Refund request not found"));
    };

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
