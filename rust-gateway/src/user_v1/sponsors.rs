use crate::*;

pub async fn methods(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_methods_response(&state, headers, uri).await {
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
    axum::extract::Path(trade_no): axum::extract::Path<String>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_detail_response(&state, trade_no, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn checkout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_checkout_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_methods_response(
    state: &AppState,
    _headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let methods = load_sponsor_methods(state).await.map_err(internal_error)?;
    let data = methods
        .iter()
        .filter(|method| method.payment == "EPay")
        .map(serialize_sponsor_method)
        .collect::<Vec<_>>();
    Ok(json_value_response(json!({
        "success": true,
        "data": data
    })))
}

async fn build_create_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let amount = payload.get("amount").and_then(parse_f64_value).ok_or_else(|| {
        fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "The amount field is required.")
    })?;
    if !(0.01..=100000.0).contains(&amount) {
        return Ok(json_value_response(json!({
            "success": false,
            "error": "The amount field must be between 0.01 and 100000."
        })));
    }

    let total_amount = (amount * 100.0).round() as i64;
    let trade_no = generate_order_trade_no();
    let result = sqlx::query(
        "INSERT INTO sponsor_donations (user_id, trade_no, total_amount, status, created_at, updated_at)
         VALUES (?, ?, ?, 0, NOW(), NOW())"
    )
    .bind(user.id as u64)
    .bind(&trade_no)
    .bind(total_amount)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    let data = json!({
        "trade_no": trade_no,
        "total_amount": total_amount,
        "status": 0,
        "id": result.last_insert_id()
    });
    Ok(json_value_response(json!({
        "success": true,
        "data": data
    })))
}

async fn build_detail_response(
    state: &AppState,
    trade_no: String,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let donation = load_sponsor_donation_by_trade_no(state, &trade_no, Some(user.id as u64))
        .await
        .map_err(internal_error)?;
    let Some(donation) = donation else {
        return Ok(json_value_response(json!({
            "success": false,
            "error": "Donation not found"
        })));
    };

    Ok(json_value_response(json!({
        "success": true,
        "data": serialize_sponsor_donation(&donation)
    })))
}

async fn build_checkout_response(
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
    let method = payload
        .get("method")
        .and_then(parse_i64_value)
        .unwrap_or_default();
    if trade_no.is_empty() || method <= 0 {
        return Ok(json_value_response(json!({
            "success": false,
            "error": "Validation failed"
        })));
    }

    let donation = load_pending_sponsor_donation_by_trade_no(state, trade_no, Some(user.id as u64))
        .await
        .map_err(internal_error)?;
    let Some(donation) = donation else {
        return Ok(json_value_response(json!({
            "success": false,
            "error": "Donation not found or already paid"
        })));
    };

    let payment = load_payment_method_by_id(state, method)
        .await
        .map_err(internal_error)?;
    let Some(payment) = payment else {
        return Ok(json_value_response(json!({
            "success": false,
            "error": "Payment method not available"
        })));
    };
    if !payment.enable || payment.payment != "EPay" {
        return Ok(json_value_response(json!({
            "success": false,
            "error": "Payment method not available"
        })));
    }

    let sponsor = load_sponsor_epay_profile(state).await?;
    let Some(sponsor) = sponsor else {
        return Ok(json_value_response(json!({
            "success": false,
            "error": "Sponsor payment profile not configured"
        })));
    };

    let updated = sqlx::query(
        "UPDATE sponsor_donations
         SET payment_id = ?, updated_at = NOW()
         WHERE id = ? AND user_id = ? AND status = 0",
    )
        .bind(payment.id)
        .bind(donation.id)
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    if updated.rows_affected() != 1 {
        return Ok(json_value_response(json!({
            "success": false,
            "error": "Donation not found or already paid"
        })));
    }

    let data = build_sponsor_epay_checkout_payload(
        &donation.trade_no,
        donation.total_amount,
        &sponsor,
        &get_setting_string(state, "app_url", "").await,
    )?;
    Ok(json_value_response(json!({
        "success": true,
        "type": 1,
        "data": data
    })))
}
