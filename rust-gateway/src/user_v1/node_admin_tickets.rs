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
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_detail_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn reply(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_reply_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn close(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_close_response(&state, headers, uri, body).await {
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
    let params = parse_query(&uri);
    let cache_key = build_user_cache_key(&uri, user.id);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }

    let status_filter = params.get("status").map(|value| value.trim()).filter(|value| !value.is_empty());
    let status_filter = match status_filter {
        Some("0") => Some(0_i64),
        Some("1") => Some(1_i64),
        Some(_) => return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Invalid parameter")),
        None => None,
    };

    let tickets = load_assigned_admin_tickets(state, user.id, status_filter)
        .await
        .map_err(internal_error)?;
    let data = tickets
        .iter()
        .map(|ticket| ticket_to_value(ticket, None))
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
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let params = parse_query(&uri);
    let cache_key = build_user_cache_key(&uri, user.id);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }

    let ticket_id = parse_optional_positive_i64(params.get("id"))
        .ok_or_else(|| fail_json_response(StatusCode::BAD_REQUEST, "Invalid parameter"))?
        .ok_or_else(|| fail_json_response(StatusCode::BAD_REQUEST, "Invalid parameter"))?;

    let ticket = load_assigned_admin_ticket_by_id(state, user.id, ticket_id)
        .await
        .map_err(internal_error)?;
    let Some(ticket) = ticket else {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Ticket does not exist"));
    };
    let messages = load_ticket_messages(state, ticket.id)
        .await
        .map_err(internal_error)?;
    Ok(success_cached_response(
        state,
        cache_key,
        assigned_admin_ticket_to_value(&ticket, Some(&messages)),
        Duration::from_secs(5),
    ))
}

async fn build_reply_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let ticket_id = payload
        .get("id")
        .and_then(parse_i64_value)
        .filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::BAD_REQUEST, "Invalid parameter"))?;
    let message = payload
        .get("message")
        .and_then(|value| value.as_str())
        .map(|value| value.trim())
        .unwrap_or("");
    if message.is_empty() {
        return Ok(fail_json_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "The message field is required.",
        ));
    }

    let ticket = load_assigned_admin_ticket_by_id(state, user.id, ticket_id)
        .await
        .map_err(internal_error)?;
    let Some(ticket) = ticket else {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Ticket does not exist"));
    };
    if ticket.status != 0 {
        return Ok(fail_json_response(
            StatusCode::BAD_REQUEST,
            "The ticket is closed and cannot be replied",
        ));
    }

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let locked_ticket = load_assigned_admin_ticket_by_id_for_update(&mut tx, user.id, ticket_id)
        .await
        .map_err(internal_error)?;
    let Some(locked_ticket) = locked_ticket else {
        tx.rollback().await.ok();
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Ticket does not exist"));
    };
    if locked_ticket.status != 0 {
        tx.rollback().await.ok();
        return Ok(fail_json_response(
            StatusCode::BAD_REQUEST,
            "The ticket is closed and cannot be replied",
        ));
    }

    let now = Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO v2_ticket_message
            (user_id, ticket_id, message, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(user.id)
    .bind(ticket_id)
    .bind(message)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;

    sqlx::query(
        "UPDATE v2_ticket
         SET status = 0, reply_status = 0, last_reply_user_id = ?, updated_at = ?
         WHERE id = ? AND assigned_admin_user_id = ?"
    )
    .bind(user.id)
    .bind(now)
    .bind(ticket_id)
    .bind(user.id)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;

    tx.commit().await.map_err(internal_error)?;
    let _ = notify_ticket_reply_to_user(
        state,
        ticket.user_id,
        &ticket.subject,
        message,
    )
    .await;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_close_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let ticket_id = payload
        .get("id")
        .and_then(parse_i64_value)
        .filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::BAD_REQUEST, "Invalid parameter"))?;

    let ticket = load_assigned_admin_ticket_by_id(state, user.id, ticket_id)
        .await
        .map_err(internal_error)?;
    let Some(_) = ticket else {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Ticket does not exist"));
    };

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let locked_ticket = load_assigned_admin_ticket_by_id_for_update(&mut tx, user.id, ticket_id)
        .await
        .map_err(internal_error)?;
    let Some(locked_ticket) = locked_ticket else {
        tx.rollback().await.ok();
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Ticket does not exist"));
    };
    if locked_ticket.status != 1 {
        sqlx::query("UPDATE v2_ticket SET status = 1, updated_at = ? WHERE id = ? AND assigned_admin_user_id = ?")
            .bind(Utc::now().timestamp())
            .bind(ticket_id)
            .bind(user.id)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }

    tx.commit().await.map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}
