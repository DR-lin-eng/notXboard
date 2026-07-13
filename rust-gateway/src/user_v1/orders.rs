use crate::*;

pub async fn fetch(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_fetch_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn check(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_check_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn detail(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_detail_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn cancel(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_cancel_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_payment_method(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_payment_method_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn save(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_save_response(&state, headers, uri, body).await {
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

async fn build_fetch_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let cache_key = build_user_cache_key(&uri, user.id);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }
    let params = parse_query(&uri);
    let status_filter = params.get("status").and_then(|v| v.parse::<i64>().ok());
    let orders = load_user_orders(state, user.id, status_filter).await.map_err(internal_error)?;
    let data = orders.into_iter().map(order_to_value).collect::<Vec<_>>();
    Ok(success_cached_response(state, cache_key, Value::Array(data), Duration::from_secs(5)))
}

async fn build_check_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let params = parse_query(&uri);
    let trade_no = params.get("trade_no").map(|v| v.trim()).filter(|v| !v.is_empty()).unwrap_or("");
    let order = find_user_order_by_trade_no(state, user.id, trade_no).await.map_err(internal_error)?;
    let Some(order) = order else {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Order does not exist"));
    };
    Ok(json_value_response(success_response_payload(Value::from(order.status))))
}

async fn build_detail_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let params = parse_query(&uri);
    let trade_no = params.get("trade_no").map(|v| v.trim()).filter(|v| !v.is_empty()).unwrap_or("");
    let order = find_user_order_by_trade_no(state, user.id, trade_no).await.map_err(internal_error)?;
    let Some(order) = order else {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Order does not exist or has been paid"));
    };
    if order.plan_name.is_none() {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Subscription plan does not exist"));
    }
    let mut value = order_to_value(order.clone());
    value["try_out_plan_id"] = Value::from(get_setting_int(state, "try_out_plan_id", 0).await);
    if let Some(ids) = order.surplus_order_ids.as_deref().and_then(parse_order_id_list) {
        let surplus_orders = load_orders_by_ids(state, user.id, &ids).await.map_err(internal_error)?;
        value["surplus_orders"] = Value::Array(surplus_orders.into_iter().map(order_to_value).collect());
    }
    Ok(json_value_response(success_response_payload(value)))
}

async fn build_cancel_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let trade_no = payload.get("trade_no").and_then(|v| v.as_str()).map(|v| v.trim()).unwrap_or("");
    if trade_no.is_empty() {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Invalid parameter"));
    }

    let order = find_user_order_by_trade_no(state, user.id, trade_no).await.map_err(internal_error)?;
    let Some(order) = order else {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Order does not exist"));
    };
    if order.status != 0 {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "You can only cancel pending orders"));
    }

    sqlx::query("UPDATE v2_order SET status = 2, updated_at = ? WHERE id = ? AND user_id = ? AND status = 0")
        .bind(Utc::now().timestamp())
        .bind(order.id)
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_get_payment_method_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _user = authenticate_bearer_user(state, &headers).await?;
    let cache_key = build_cache_key(&uri);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }
    let rows = sqlx::query(
        "SELECT id, name, payment, icon, handling_fee_fixed,
                CAST(handling_fee_percent AS CHAR) AS handling_fee_percent
         FROM v2_payment
         WHERE enable = 1
         ORDER BY sort ASC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;
    let data = rows
        .into_iter()
        .map(|row| {
            json!({
                "id": row.try_get::<i64, _>("id").unwrap_or_default(),
                "name": row.try_get::<String, _>("name").unwrap_or_default(),
                "payment": row.try_get::<String, _>("payment").unwrap_or_default(),
                "icon": row.try_get::<Option<String>, _>("icon").unwrap_or(None),
                "handling_fee_fixed": row.try_get::<Option<i64>, _>("handling_fee_fixed").unwrap_or(None),
                "handling_fee_percent": row.try_get::<Option<String>, _>("handling_fee_percent").unwrap_or(None),
            })
        })
        .collect::<Vec<_>>();
    Ok(success_cached_response(state, cache_key, Value::Array(data), Duration::from_secs(15)))
}

async fn build_save_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let plan_id = payload.get("plan_id").and_then(|v| v.as_i64());
    let purchase_token = payload
        .get("purchase_token")
        .and_then(|v| v.as_str())
        .map(|v| v.trim())
        .filter(|v| !v.is_empty());
    let coupon_code = payload
        .get("coupon_code")
        .and_then(|v| v.as_str())
        .map(|v| v.trim())
        .filter(|v| !v.is_empty());
    let period = payload
        .get("period")
        .and_then(|v| v.as_str())
        .map(|v| v.trim())
        .unwrap_or("");
    let trade_no = create_user_order(
        state,
        &user,
        CreateUserOrderInput {
            plan_id,
            purchase_token,
            coupon_code,
            period,
        },
    )
    .await?;

    Ok(json_value_response(success_response_payload(Value::String(trade_no))))
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
        .and_then(|v| v.as_str())
        .map(|v| v.trim())
        .unwrap_or("");
    let method = payload.get("method").and_then(|v| v.as_i64()).unwrap_or(0);
    let pay_to = payload
        .get("pay_to")
        .and_then(|v| v.as_str())
        .map(|value| value.trim())
        .filter(|value| !value.is_empty());

    let payment = if method > 0 {
        load_payment_method_by_id(state, method).await.map_err(internal_error)?
    } else {
        None
    };

    match prepare_user_checkout(state, user.id, trade_no, payment).await? {
        PreparedUserCheckout::NoPaymentRequired => {
            Ok(json_value_response(json!({ "type": -1, "data": true })))
        }
        PreparedUserCheckout::Pending(session) => {
            let UserCheckoutSession {
                order,
                payment,
                amount,
            } = session;
            if payment.payment == "EPay" {
                let config = resolve_checkout_epay_config(state, &order, &payment, pay_to).await?;
                let data = build_epay_checkout_payload(&order, amount, &config, &payment.uuid)?;
                return Ok(json_value_response(json!({
                    "type": 1,
                    "data": data
                })));
            }

            Ok(json_value_response(json!({
                "type": 1,
                "data": {
                    "trade_no": order.trade_no,
                    "payment": payment.payment,
                    "payment_id": payment.id,
                    "amount": amount,
                    "handling_amount": order.handling_amount.unwrap_or(0),
                }
            })))
        }
    }
}
