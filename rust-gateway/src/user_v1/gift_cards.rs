use crate::*;

pub async fn check(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_check_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn redeem(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_redeem_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn history(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_history_response(&state, headers, uri).await {
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

pub async fn types(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_types_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_check_response(
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
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "请输入兑换码"));
    }

    let code_row = load_gift_card_code_lookup(state, code).await.map_err(internal_error)?;
    let Some(code_row) = code_row else {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "兑换码不存在"));
    };
    validate_gift_card_is_active(&code_row)?;

    let eligibility = check_gift_card_user_eligibility(state, &code_row, &user).await?;
    let code_info = build_gift_card_code_info(state, &code_row).await.map_err(internal_error)?;
    let reward_preview = calculate_gift_card_actual_rewards(&code_row);

    Ok(json_value_response(success_response_payload(json!({
        "code_info": code_info,
        "reward_preview": reward_preview,
        "can_redeem": eligibility.0,
        "reason": eligibility.1
    }))))
}

async fn build_redeem_response(
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
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "请输入兑换码"));
    }
    if code.chars().count() < 8 {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "兑换码长度不能少于8位"));
    }
    if code.chars().count() > 32 {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "兑换码长度不能超过32位"));
    }

    let locked = load_gift_card_code_lookup_for_update(state, code).await.map_err(internal_error)?;
    let Some(code_row) = locked else {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "兑换码不存在"));
    };
    validate_gift_card_is_active(&code_row)?;
    let eligibility = check_gift_card_user_eligibility(state, &code_row, &user).await?;
    if !eligibility.0 {
        return Ok(fail_json_response(
            StatusCode::BAD_REQUEST,
            eligibility.1.as_deref().unwrap_or("您不满足此礼品卡的使用条件"),
        ));
    }

    let actual_rewards = calculate_gift_card_actual_rewards(&code_row);
    let multiplier = calculate_gift_card_multiplier(&code_row);
    let user_agent = headers
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string());
    let now = Utc::now().timestamp();
    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let updated_user = apply_gift_card_rewards_tx(&mut tx, state, user.id, &code_row, &actual_rewards, now).await?;
    let invite_rewards = apply_gift_card_invite_rewards_tx(&mut tx, state, &updated_user, &actual_rewards, now).await?;

    sqlx::query(
        "UPDATE v2_gift_card_code
         SET status = 1, user_id = ?, used_at = ?, usage_count = usage_count + 1, actual_rewards = ?, updated_at = ?
         WHERE id = ?"
    )
    .bind(user.id)
    .bind(now)
    .bind(actual_rewards.to_string())
    .bind(now)
    .bind(code_row.id)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;

    sqlx::query(
        "INSERT INTO v2_gift_card_usage
            (code_id, template_id, user_id, invite_user_id, rewards_given, invite_rewards, user_level_at_use,
             plan_id_at_use, multiplier_applied, user_agent, notes, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(code_row.id)
    .bind(code_row.template_id)
    .bind(user.id)
    .bind(updated_user.invite_user_id)
    .bind(actual_rewards.to_string())
    .bind(invite_rewards.as_ref().map(|value| value.to_string()))
    .bind(updated_user.plan_id)
    .bind(updated_user.plan_id)
    .bind(format!("{:.2}", multiplier))
    .bind(user_agent)
    .bind(None::<String>)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;
    tx.commit().await.map_err(internal_error)?;

    Ok(json_value_response(success_response_payload(json!({
        "message": "兑换成功！",
        "rewards": actual_rewards,
        "invite_rewards": invite_rewards,
        "template_name": code_row.template_name
    }))))
}

async fn build_history_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let params = parse_query(&uri);
    let per_page = params
        .get("per_page")
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0 && *value <= 100)
        .unwrap_or(15);
    let page = params
        .get("page")
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(1);
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_gift_card_usage WHERE user_id = ?")
        .bind(user.id)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    let offset = (page - 1) * per_page;
    let rows = load_gift_card_usage_history(state, user.id, offset, per_page)
        .await
        .map_err(internal_error)?;
    let last_page = if total <= 0 { 1 } else { ((total + per_page - 1) / per_page).max(1) };
    Ok(json_value_response(json!({
        "data": rows.iter().map(serialize_gift_card_history_item).collect::<Vec<_>>(),
        "pagination": {
            "current_page": page,
            "last_page": last_page,
            "per_page": per_page,
            "total": total
        }
    })))
}

async fn build_detail_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let params = parse_query(&uri);
    let usage_id = params
        .get("id")
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Invalid parameter"))?;

    let usage = load_gift_card_usage_detail(state, usage_id, user.id)
        .await
        .map_err(internal_error)?;
    let Some(usage) = usage else {
        return Ok(fail_json_response(StatusCode::NOT_FOUND, "记录不存在"));
    };

    Ok(json_value_response(success_response_payload(serialize_gift_card_usage_detail(&usage))))
}

async fn build_types_response(
    _state: &AppState,
    _headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    Ok(json_value_response(success_response_payload(json!({
        "types": {
            "1": "通用礼品卡",
            "2": "套餐礼品卡",
            "3": "盲盒礼品卡"
        }
    }))))
}
