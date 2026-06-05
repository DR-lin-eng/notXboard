use super::super::*;

pub async fn logs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_logs_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn stats(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_stats_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn reset_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_reset_user_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn user_history(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(user_id): axum::extract::Path<i64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_user_history_response(&state, user_id, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_logs_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let per_page = params.get("per_page").and_then(|value| value.parse::<i64>().ok()).unwrap_or(20).clamp(1, 10_000);
    let page = params.get("page").and_then(|value| value.parse::<i64>().ok()).unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;

    let user_id = params.get("user_id").and_then(|value| value.parse::<i64>().ok()).filter(|value| *value > 0);
    let user_email = params.get("user_email").map(|value| value.trim().to_string()).filter(|value| !value.is_empty());
    let reset_type = params.get("reset_type").map(|value| value.trim().to_string()).filter(|value| !value.is_empty());
    let trigger_source = params.get("trigger_source").map(|value| value.trim().to_string()).filter(|value| !value.is_empty());
    let start_date = params.get("start_date").map(|value| value.trim().to_string()).filter(|value| !value.is_empty());
    let end_date = params.get("end_date").map(|value| value.trim().to_string()).filter(|value| !value.is_empty());

    let (rows, total) = load_admin_traffic_reset_logs(
        state,
        user_id,
        user_email.as_deref(),
        reset_type.as_deref(),
        trigger_source.as_deref(),
        start_date.as_deref(),
        end_date.as_deref(),
        offset,
        per_page,
    )
    .await
    .map_err(internal_error)?;
    let last_page = if total <= 0 { 1 } else { ((total + per_page - 1) / per_page).max(1) };

    Ok(json_value_response(json!({
        "data": rows.iter().map(serialize_admin_traffic_reset_log).collect::<Vec<_>>(),
        "pagination": {
            "current_page": page,
            "last_page": last_page,
            "per_page": per_page,
            "total": total,
        }
    })))
}

async fn build_stats_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let days = params.get("days").and_then(|value| value.parse::<i64>().ok()).unwrap_or(30).clamp(1, 365);
    let start_ts = Utc::now().timestamp() - days * 86_400;

    let total_resets: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_traffic_reset_logs WHERE UNIX_TIMESTAMP(reset_time) >= ?")
        .bind(start_ts)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    let manual_resets: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_traffic_reset_logs WHERE UNIX_TIMESTAMP(reset_time) >= ? AND trigger_source = 'manual'")
        .bind(start_ts)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    let cron_resets: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_traffic_reset_logs WHERE UNIX_TIMESTAMP(reset_time) >= ? AND trigger_source = 'cron'")
        .bind(start_ts)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    let auto_resets: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_traffic_reset_logs WHERE UNIX_TIMESTAMP(reset_time) >= ? AND trigger_source = 'auto'")
        .bind(start_ts)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;

    Ok(json_value_response(json!({
        "data": {
            "total_resets": total_resets,
            "auto_resets": auto_resets,
            "manual_resets": manual_resets,
            "cron_resets": cron_resets,
        }
    })))
}

async fn build_reset_user_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let user_id = payload.get("user_id").and_then(parse_i64_value).filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let user = load_bearer_user_row_by_id(state, user_id).await.map_err(internal_error)?;
    let Some(user) = user else {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "The user does not exist"));
    };
    if !user_is_active_for_traffic_reset(&user) {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message": "traffic_reset.user_cannot_reset"})));
    }

    let success = reset_single_user_traffic(state, &user, "manual").await.map_err(internal_error)?;
    if !success {
        return Ok(json_status_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"message": "traffic_reset.reset_failed"})));
    }

    let refreshed = load_bearer_user_row_by_id(state, user_id).await.map_err(internal_error)?;
    let next_reset_at = refreshed.and_then(|value| value.next_reset_at);
    Ok(json_value_response(json!({
        "message": "traffic_reset.reset_success",
        "data": {
            "user_id": user.id,
            "email": user.email,
            "reset_time": Utc::now().timestamp(),
            "next_reset_at": next_reset_at,
        }
    })))
}

async fn build_user_history_response(
    state: &AppState,
    user_id: i64,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let limit = params.get("limit").and_then(|value| value.parse::<i64>().ok()).unwrap_or(10).clamp(1, 50);
    let user = load_bearer_user_row_by_id(state, user_id).await.map_err(internal_error)?;
    let Some(user) = user else {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "The user does not exist"));
    };
    let history = load_user_traffic_reset_history(state, user_id, limit).await.map_err(internal_error)?;
    Ok(json_value_response(json!({
        "data": {
            "user": {
                "id": user.id,
                "email": user.email,
                "reset_count": user_reset_count(state, user_id).await.map_err(internal_error)?,
                "last_reset_at": user_last_reset_at(state, user_id).await.map_err(internal_error)?,
                "next_reset_at": user.next_reset_at,
            },
            "history": history.iter().map(serialize_admin_traffic_reset_history_item).collect::<Vec<_>>()
        }
    })))
}
