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

pub async fn withdraw(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_withdraw_response(&state, headers, uri, body).await {
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
    let params = parse_query(&uri);
    let cache_key = build_user_cache_key(&uri, user.id);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }

    let node_id = parse_optional_positive_u64(params.get("node_id"))
        .ok_or_else(|| fail_json_response(StatusCode::BAD_REQUEST, "Invalid parameter"))?;
    if let Some(ticket_id) = parse_optional_positive_i64(params.get("id"))
        .ok_or_else(|| fail_json_response(StatusCode::BAD_REQUEST, "Invalid parameter"))?
    {
        let ticket = load_user_ticket_by_id(state, user.id, ticket_id)
            .await
            .map_err(internal_error)?;
        let Some(ticket) = ticket else {
            return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Ticket does not exist"));
        };
        let messages = load_ticket_messages(state, ticket.id)
            .await
            .map_err(internal_error)?;
        return Ok(success_cached_response(
            state,
            cache_key,
            ticket_to_value(&ticket, Some(&messages)),
            Duration::from_secs(5),
        ));
    }

    let tickets = load_user_tickets(state, user.id, node_id)
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

async fn build_save_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;

    let subject = payload
        .get("subject")
        .and_then(|value| value.as_str())
        .map(|value| value.trim())
        .unwrap_or("");
    if subject.is_empty() {
        return Ok(fail_json_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Ticket subject cannot be empty",
        ));
    }

    let level = payload.get("level").and_then(parse_i64_value).ok_or_else(|| {
        fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Ticket level cannot be empty")
    })?;
    if !matches!(level, 0..=2) {
        return Ok(fail_json_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Incorrect ticket level format",
        ));
    }

    let message = payload
        .get("message")
        .and_then(|value| value.as_str())
        .map(|value| value.trim())
        .unwrap_or("");
    if message.is_empty() {
        return Ok(fail_json_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Message cannot be empty",
        ));
    }

    let node_id = payload
        .get("node_id")
        .map(parse_u64_value)
        .transpose()
        .map_err(|_| fail_json_response(StatusCode::BAD_REQUEST, "Invalid parameter"))?;

    let assigned_admin_user_id = if let Some(node_id) = node_id {
        let node = load_ticket_node_by_id(state, node_id)
            .await
            .map_err(internal_error)?;
        let Some(node) = node else {
            return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Invalid parameter"));
        };
        let can_access = user_can_access_ticket_node(state, &user, &node)
            .await
            .map_err(internal_error)?;
        if !can_access {
            return Ok(fail_json_response(
                StatusCode::FORBIDDEN,
                "No permission to create ticket for this node",
            ));
        }
        Some(node.user_id)
    } else {
        None
    };

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT id
         FROM v2_ticket
         WHERE status = 0
           AND user_id = ?
           AND ((? IS NULL AND node_id IS NULL) OR node_id = ?)
         LIMIT 1"
    )
    .bind(user.id)
    .bind(node_id)
    .bind(node_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal_error)?;
    if existing.is_some() {
        tx.rollback().await.ok();
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "存在未关闭的工单"));
    }

    let now = Utc::now().timestamp();
    let result = sqlx::query(
        "INSERT INTO v2_ticket
            (user_id, node_id, assigned_admin_user_id, subject, level, status, reply_status, last_reply_user_id, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, 0, 1, ?, ?, ?)"
    )
    .bind(user.id)
    .bind(node_id)
    .bind(assigned_admin_user_id)
    .bind(subject)
    .bind(level)
    .bind(user.id)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;

    let ticket_id = result.last_insert_id() as i64;
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

    tx.commit().await.map_err(internal_error)?;
    let _ = notify_ticket_created_to_assigned_admin(
        state,
        assigned_admin_user_id,
        node_id,
        subject,
        message,
    )
    .await;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
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
            StatusCode::BAD_REQUEST,
            "Message cannot be empty",
        ));
    }

    let ticket = load_user_ticket_by_id(state, user.id, ticket_id)
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

    let last_message = load_last_ticket_message(state, ticket.id, false)
        .await
        .map_err(internal_error)?;
    if let Some(last_message) = last_message {
        if last_message.user_id == user.id {
            return Ok(fail_json_response(
                StatusCode::BAD_REQUEST,
                "Please wait for the technical enginneer to reply",
            ));
        }
    }

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let locked_ticket = load_user_ticket_by_id_for_update(&mut tx, user.id, ticket_id)
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
    let last_message = load_last_ticket_message_with_tx(&mut tx, ticket_id)
        .await
        .map_err(internal_error)?;
    if let Some(last_message) = last_message {
        if last_message.user_id == user.id {
            tx.rollback().await.ok();
            return Ok(fail_json_response(
                StatusCode::BAD_REQUEST,
                "Please wait for the technical enginneer to reply",
            ));
        }
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
         SET status = 0, reply_status = 1, last_reply_user_id = ?, updated_at = ?
         WHERE id = ? AND user_id = ?"
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
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Invalid parameter"))?;

    let ticket = load_user_ticket_by_id(state, user.id, ticket_id)
        .await
        .map_err(internal_error)?;
    let Some(_) = ticket else {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Ticket does not exist"));
    };

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let locked_ticket = load_user_ticket_by_id_for_update(&mut tx, user.id, ticket_id)
        .await
        .map_err(internal_error)?;
    let Some(locked_ticket) = locked_ticket else {
        tx.rollback().await.ok();
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Ticket does not exist"));
    };

    if locked_ticket.status != 1 {
        sqlx::query("UPDATE v2_ticket SET status = 1, updated_at = ? WHERE id = ? AND user_id = ?")
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

async fn build_withdraw_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    if get_setting_bool(state, "withdraw_close_enable", false).await {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Unsupported withdraw"));
    }

    let payload = parse_json_body(body).await?;
    let withdraw_method = payload
        .get("withdraw_method")
        .and_then(|value| value.as_str())
        .map(|value| value.trim())
        .unwrap_or("");
    if withdraw_method.is_empty() {
        return Ok(fail_json_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Withdrawal method cannot be empty",
        ));
    }

    let account = payload
        .get("account")
        .and_then(|value| value.as_str())
        .map(|value| value.trim())
        .unwrap_or("");
    if account.is_empty() {
        return Ok(fail_json_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Withdrawal account cannot be empty",
        ));
    }

    let withdraw_amount = payload
        .get("withdraw_amount")
        .and_then(parse_i64_value)
        .ok_or_else(|| {
            fail_json_response(
                StatusCode::UNPROCESSABLE_ENTITY,
                "The withdrawal amount cannot be empty",
            )
        })?;
    if withdraw_amount <= 0 {
        return Ok(fail_json_response(
            StatusCode::BAD_REQUEST,
            "Incorrect withdrawal amount",
        ));
    }

    if user.commission_balance < withdraw_amount {
        return Ok(fail_json_response(
            StatusCode::BAD_REQUEST,
            "Insufficient commission balance",
        ));
    }

    let ticket_subject = format!("Commission withdraw - {}", withdraw_method);
    let ticket_message = format!(
        "Withdraw method: {}\nAccount: {}\nAmount: {}",
        withdraw_method, account, withdraw_amount
    );
    let now = Utc::now().timestamp();
    let mut tx = state.db.begin().await.map_err(internal_error)?;

    let exists = sqlx::query_scalar::<_, i64>(
        "SELECT id
         FROM v2_ticket
         WHERE user_id = ?
           AND status = 0
           AND subject = ?
         LIMIT 1"
    )
    .bind(user.id)
    .bind(&ticket_subject)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal_error)?;
    if exists.is_some() {
        tx.rollback().await.ok();
        return Ok(fail_json_response(
            StatusCode::BAD_REQUEST,
            "There is already an open withdrawal ticket",
        ));
    }

    let result = sqlx::query(
        "INSERT INTO v2_ticket
            (user_id, node_id, assigned_admin_user_id, subject, level, status, reply_status, last_reply_user_id, created_at, updated_at)
         VALUES (?, NULL, NULL, ?, 0, 0, 1, ?, ?, ?)"
    )
    .bind(user.id)
    .bind(&ticket_subject)
    .bind(user.id)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;
    let ticket_id = result.last_insert_id() as i64;

    sqlx::query(
        "INSERT INTO v2_ticket_message
            (user_id, ticket_id, message, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(user.id)
    .bind(ticket_id)
    .bind(ticket_message)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;

    tx.commit().await.map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}
