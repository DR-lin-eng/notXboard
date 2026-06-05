use super::super::*;

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

pub async fn fetch_post(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_fetch_response_from_body(&state, headers, uri, body).await {
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

async fn build_fetch_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    build_fetch_response_with_params(state, params).await
}

async fn build_fetch_response_from_body(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let params = value_object_to_string_map(payload.as_object());
    build_fetch_response_with_params(state, params).await
}

async fn build_fetch_response_with_params(
    state: &AppState,
    params: HashMap<String, String>,
) -> Result<Response<Body>, Response<Body>> {
    if let Some(ticket_id) = parse_optional_positive_i64(params.get("id"))
        .ok_or_else(|| fail_json_response(StatusCode::BAD_REQUEST, "Invalid parameter"))?
    {
        let ticket = load_admin_ticket_by_id(state, ticket_id)
            .await
            .map_err(internal_error)?;
        let Some(ticket) = ticket else {
            return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"工单不存在"})));
        };
        let messages = load_ticket_messages(state, ticket.id)
            .await
            .map_err(internal_error)?;
        let user = load_bearer_user_by_id(state, ticket.user_id)
            .await
            .map_err(internal_error)?;
        return Ok(json_value_response(success_response_payload(serialize_admin_ticket_detail(
            state,
            &ticket,
            &messages,
            user.as_ref(),
        ).await)));
    }

    let current = params.get("current").and_then(|value| value.parse::<i64>().ok()).unwrap_or(1).max(1);
    let page_size = params.get("pageSize").and_then(|value| value.parse::<i64>().ok()).unwrap_or(10).clamp(1, 100);
    let offset = (current - 1) * page_size;
    let status_filter = params.get("status").and_then(|value| value.parse::<i64>().ok());
    let reply_status_filter = parse_csv_i64_list(params.get("reply_status"));
    let email_filter = params.get("email").map(|value| value.trim().to_string()).filter(|value| !value.is_empty());

    let (rows, total) = load_admin_tickets(
        state,
        status_filter,
        if reply_status_filter.is_empty() { None } else { Some(reply_status_filter.as_slice()) },
        email_filter.as_deref(),
        offset,
        page_size,
    )
    .await
    .map_err(internal_error)?;

    Ok(json_value_response(success_response_payload(json!({
        "data": rows.iter().map(serialize_admin_ticket_list_item).collect::<Vec<_>>(),
        "total": total,
        "current": current,
        "pageSize": page_size,
    }))))
}

async fn build_reply_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let ticket_id = payload
        .get("id")
        .and_then(parse_i64_value)
        .filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "工单ID不能为空"))?;
    let message = payload
        .get("message")
        .and_then(|value| value.as_str())
        .map(|value| value.trim())
        .unwrap_or("");
    if message.is_empty() {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "消息不能为空"));
    }

    let ticket = load_admin_ticket_by_id(state, ticket_id)
        .await
        .map_err(internal_error)?;
    let Some(ticket) = ticket else {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"工单不存在"})));
    };
    if ticket.status != 0 {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"工单已关闭，无法继续回复"})));
    }

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let locked_ticket = load_admin_ticket_by_id_for_update(&mut tx, ticket_id)
        .await
        .map_err(internal_error)?;
    let Some(locked_ticket) = locked_ticket else {
        tx.rollback().await.ok();
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"工单不存在"})));
    };
    if locked_ticket.status != 0 {
        tx.rollback().await.ok();
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"工单已关闭，无法继续回复"})));
    }

    let now = Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO v2_ticket_message
            (user_id, ticket_id, message, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?)"
    )
    .bind(admin.id)
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
         WHERE id = ?"
    )
    .bind(admin.id)
    .bind(now)
    .bind(ticket_id)
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
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let ticket_id = payload
        .get("id")
        .and_then(parse_i64_value)
        .filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "工单ID不能为空"))?;

    let ticket = load_admin_ticket_by_id(state, ticket_id)
        .await
        .map_err(internal_error)?;
    let Some(_) = ticket else {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"工单不存在"})));
    };

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let locked_ticket = load_admin_ticket_by_id_for_update(&mut tx, ticket_id)
        .await
        .map_err(internal_error)?;
    let Some(locked_ticket) = locked_ticket else {
        tx.rollback().await.ok();
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"工单不存在"})));
    };

    if locked_ticket.status != 1 {
        sqlx::query("UPDATE v2_ticket SET status = 1, updated_at = ? WHERE id = ?")
            .bind(Utc::now().timestamp())
            .bind(ticket_id)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }

    tx.commit().await.map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

fn value_object_to_string_map(object: Option<&serde_json::Map<String, Value>>) -> HashMap<String, String> {
    let mut params = HashMap::new();
    let Some(object) = object else {
        return params;
    };
    for (key, value) in object {
        match value {
            Value::Null => {}
            Value::String(text) => {
                params.insert(key.clone(), text.clone());
            }
            Value::Number(number) => {
                params.insert(key.clone(), number.to_string());
            }
            Value::Bool(flag) => {
                params.insert(key.clone(), if *flag { "1".to_string() } else { "0".to_string() });
            }
            Value::Array(items) => {
                let joined = items
                    .iter()
                    .filter_map(|item| match item {
                        Value::String(text) => Some(text.clone()),
                        Value::Number(number) => Some(number.to_string()),
                        Value::Bool(flag) => Some(if *flag { "1".to_string() } else { "0".to_string() }),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                if !joined.is_empty() {
                    params.insert(key.clone(), joined);
                }
            }
            Value::Object(_) => {}
        }
    }
    params
}
