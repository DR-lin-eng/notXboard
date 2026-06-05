use crate::*;

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

pub async fn details(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_details_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_save_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let assigned_plan_id = payload.get("assigned_plan_id").and_then(parse_i64_value).unwrap_or(0);
    let assigned_period = payload
        .get("assigned_period")
        .and_then(|value| value.as_str())
        .map(|value| value.trim())
        .unwrap_or("");

    if assigned_plan_id > 0 || !assigned_period.is_empty() {
        if assigned_plan_id <= 0 || assigned_period.is_empty() {
            return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "邀请套餐和周期必须同时指定"));
        }
        validate_assignable_invite_plan(state, user.id, assigned_plan_id, assigned_period)
            .await?;
    }

    let limit = get_setting_int(state, "invite_gen_limit", 5).await.max(1);
    let mut saved = false;
    let mut last_error = None;

    for _attempt in 0..6 {
        let mut tx = state.db.begin().await.map_err(internal_error)?;
        let active_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM v2_invite_code WHERE user_id = ? AND status = 0 FOR UPDATE"
        )
        .bind(user.id)
        .fetch_one(&mut *tx)
        .await
        .map_err(internal_error)?;
        if active_count >= limit {
            tx.rollback().await.ok();
            return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "The maximum number of creations has been reached"));
        }

        let code = random_letters(8);
        let now = Utc::now().timestamp();
        let result = sqlx::query(
            "INSERT INTO v2_invite_code
                (user_id, assigned_plan_id, assigned_period, code, status, pv, created_at, updated_at)
             VALUES (?, ?, ?, ?, 0, 0, ?, ?)"
        )
        .bind(user.id)
        .bind(if assigned_plan_id > 0 { Some(assigned_plan_id) } else { None::<i64> })
        .bind(if assigned_period.is_empty() { None::<String> } else { Some(assigned_period.to_string()) })
        .bind(&code)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await;

        match result {
            Ok(_) => {
                tx.commit().await.map_err(internal_error)?;
                saved = true;
                break;
            }
            Err(err) => {
                last_error = Some(err);
                tx.rollback().await.ok();
                if !is_duplicate_sqlx_error(last_error.as_ref().unwrap()) {
                    return Err(internal_error(last_error.unwrap()));
                }
            }
        }
    }

    if !saved {
        if let Some(err) = last_error {
            return Err(internal_error(err));
        }
        return Ok(fail_json_response(StatusCode::INTERNAL_SERVER_ERROR, "邀请码生成失败，请重试"));
    }

    Ok(json_value_response(success_response_payload(Value::Bool(true))))
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

    let commission_rate = load_effective_commission_rate(state, user.id).await.map_err(internal_error)?;
    let uncheck_commission_balance = load_unchecked_commission_balance(state, user.id).await.map_err(internal_error)?;
    let invite_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_user WHERE invite_user_id = ?")
        .bind(user.id)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    let effective_commission: i64 = sqlx::query_scalar("SELECT CAST(COALESCE(SUM(get_amount), 0) AS SIGNED) FROM v2_commission_log WHERE invite_user_id = ?")
        .bind(user.id)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    let available_commission: i64 = sqlx::query_scalar("SELECT commission_balance FROM v2_user WHERE id = ? LIMIT 1")
        .bind(user.id)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;

    let codes = load_unused_invite_codes(state, user.id).await.map_err(internal_error)?;
    let available_plans = load_assignable_invite_plans(state, user.id).await.map_err(internal_error)?;

    let data = json!({
        "codes": codes.iter().map(serialize_invite_code).collect::<Vec<_>>(),
        "available_plans": available_plans.iter().map(serialize_available_invite_plan).collect::<Vec<_>>(),
        "stat": [
            invite_count,
            effective_commission,
            uncheck_commission_balance,
            commission_rate,
            available_commission
        ]
    });
    Ok(success_cached_response(state, cache_key, data, Duration::from_secs(5)))
}

async fn build_details_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let params = parse_query(&uri);
    let current = params.get("current").and_then(|value| value.parse::<i64>().ok()).filter(|value| *value > 0).unwrap_or(1);
    let requested_page_size = params.get("page_size").and_then(|value| value.parse::<i64>().ok()).unwrap_or(10);
    let page_size = if requested_page_size >= 10 { requested_page_size } else { 10 };

    let total: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM v2_commission_log WHERE invite_user_id = ? AND get_amount > 0"
    )
    .bind(user.id)
    .fetch_one(&state.db)
    .await
    .map_err(internal_error)?;

    let offset = (current - 1) * page_size;
    let rows = load_invite_commission_logs(state, user.id, offset, page_size)
        .await
        .map_err(internal_error)?;
    Ok(json_value_response(json!({
        "data": rows.iter().map(serialize_commission_log).collect::<Vec<_>>(),
        "total": total
    })))
}
