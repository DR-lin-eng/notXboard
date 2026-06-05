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

pub async fn toggle(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_toggle_response(&state, id, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn drop_notice(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_drop_response(&state, id, headers, uri, body).await {
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
    let current = params.get("current").and_then(|v| v.parse::<i64>().ok()).filter(|v| *v > 0).unwrap_or(1);
    let page_size = params.get("page_size").and_then(|v| v.parse::<i64>().ok()).map(|v| v.clamp(5, 20)).unwrap_or(10);
    let offset = (current - 1) * page_size;

    let active_plan_ids: Vec<i64> = sqlx::query_scalar(
        "SELECT DISTINCT ups.plan_id
         FROM user_plan_subscriptions ups
         WHERE ups.user_id = ?
           AND ups.status = 1
           AND (ups.expired_at IS NULL OR ups.expired_at > ?)
         ORDER BY ups.plan_id"
    )
    .bind(user.id)
    .bind(Utc::now().timestamp())
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    let notices = sqlx::query(
        "SELECT id, sort, title, content, show, popup, author_user_id, scope_type, target_plan_ids, img_url, tags, created_at, updated_at
         FROM v2_notice
         WHERE show = 1
         ORDER BY sort ASC, id DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    let items = notices
        .into_iter()
        .filter_map(|row| {
            let scope_type = row.try_get::<Option<String>, _>("scope_type").ok().flatten().unwrap_or_else(|| "global".to_string());
            let author_user_id = row.try_get::<Option<i64>, _>("author_user_id").ok().flatten().unwrap_or_default();
            let target_plan_ids = row.try_get::<Option<String>, _>("target_plan_ids").ok().flatten().unwrap_or_default();
            let target_ids = parse_notice_target_plan_ids(&target_plan_ids);
            let allowed = scope_type == "global"
                || author_user_id == user.id
                || (!target_ids.is_empty() && target_ids.iter().any(|id| active_plan_ids.contains(id)));
            if std::env::var("NOTICE_DEBUG").ok().as_deref() == Some("1") {
                eprintln!(
                    "NOTICE DEBUG user_id={} notice_id={} scope_type={} author_user_id={} target_ids={:?} active_plan_ids={:?} allowed={}",
                    user.id,
                    row.try_get::<i64, _>("id").unwrap_or_default(),
                    scope_type,
                    author_user_id,
                    target_ids,
                    active_plan_ids,
                    allowed
                );
            }
            if allowed {
                Some(json!({
                    "id": row.try_get::<i64, _>("id").unwrap_or_default(),
                    "sort": row.try_get::<Option<i64>, _>("sort").ok().flatten(),
                    "title": row.try_get::<String, _>("title").unwrap_or_default(),
                    "content": row.try_get::<String, _>("content").unwrap_or_default(),
                    "show": row.try_get::<Option<i8>, _>("show").ok().flatten().unwrap_or(0) != 0,
                    "popup": row.try_get::<Option<i8>, _>("popup").ok().flatten().unwrap_or(0) != 0,
                    "author_user_id": row.try_get::<Option<i64>, _>("author_user_id").ok().flatten(),
                    "scope_type": scope_type,
                    "target_plan_ids": target_ids.into_iter().map(|v| Value::from(v)).collect::<Vec<_>>(),
                    "img_url": row.try_get::<Option<String>, _>("img_url").ok().flatten(),
                    "tags": row.try_get::<Option<String>, _>("tags").ok().flatten(),
                    "created_at": row.try_get::<Option<i64>, _>("created_at").ok().flatten(),
                    "updated_at": row.try_get::<Option<i64>, _>("updated_at").ok().flatten(),
                }))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let total = items.len() as i64;
    let data = items
        .into_iter()
        .skip(offset as usize)
        .take(page_size as usize)
        .collect::<Vec<_>>();

    Ok(json_value_response(success_response_payload(json!({
        "data": data,
        "total": total,
        "can_publish": false,
        "plan_options": [],
        "manageable": [],
    }))))
}

async fn build_save_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let title = payload.get("title").and_then(|v| v.as_str()).map(|v| v.trim()).unwrap_or("");
    let content = payload.get("content").and_then(|v| v.as_str()).map(|v| v.trim()).unwrap_or("");
    if title.is_empty() || content.is_empty() {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    }

    let target_plan_ids = payload.get("target_plan_ids").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    if target_plan_ids.is_empty() {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    }

    let mut plan_ids = target_plan_ids
        .into_iter()
        .filter_map(|value| parse_i64_value(&value))
        .filter(|value| *value > 0)
        .collect::<Vec<_>>();
    plan_ids.sort_unstable();
    plan_ids.dedup();
    if plan_ids.is_empty() {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    }

    let placeholders = vec!["?"; plan_ids.len()].join(",");
    let sql = format!(
        "SELECT COUNT(*) FROM v2_plan WHERE scope = 'node' AND owner_user_id = ? AND id IN ({})",
        placeholders
    );
    let mut query = sqlx::query_scalar::<_, i64>(&sql).bind(user.id);
    for plan_id in &plan_ids {
        query = query.bind(*plan_id);
    }
    let owned_count = query.fetch_one(&state.db).await.map_err(internal_error)?;
    if owned_count != plan_ids.len() as i64 {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "target_plan_ids contains plans you do not own"));
    }

    let serialized_targets = serde_json::to_string(&plan_ids).unwrap_or_else(|_| "[]".to_string());
    let tags = payload.get("tags").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let serialized_tags = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());

    let inserted = sqlx::query(
        "INSERT INTO v2_notice (sort, title, content, `show`, popup, author_user_id, scope_type, target_plan_ids, img_url, tags, created_at, updated_at)
         VALUES (0, ?, ?, 1, 0, ?, 'plan_subscribers', ?, NULL, ?, UNIX_TIMESTAMP(), UNIX_TIMESTAMP())"
    )
    .bind(title)
    .bind(content)
    .bind(user.id)
    .bind(serialized_targets)
    .bind(serialized_tags)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(json_value_response(success_response_payload(json!({
        "id": inserted.last_insert_id(),
        "title": title,
        "content": content,
        "show": true,
        "popup": false,
        "author_user_id": user.id,
        "scope_type": "plan_subscribers",
        "target_plan_ids": plan_ids,
    }))))
}

async fn build_toggle_response(
    state: &AppState,
    notice_id: u64,
    headers: HeaderMap,
    _uri: Uri,
    _body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let row = sqlx::query("SELECT id, `show` FROM v2_notice WHERE id = ? AND author_user_id = ? AND scope_type = 'plan_subscribers' LIMIT 1")
        .bind(notice_id)
        .bind(user.id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?;
    let Some(row) = row else {
        return Ok(fail_json_response(StatusCode::NOT_FOUND, "Notice not found"));
    };
    let current_show = row.try_get::<i8, _>("show").unwrap_or(0) != 0;
    sqlx::query("UPDATE v2_notice SET `show` = ?, updated_at = UNIX_TIMESTAMP() WHERE id = ?")
        .bind(!current_show)
        .bind(notice_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_drop_response(
    state: &AppState,
    notice_id: u64,
    headers: HeaderMap,
    _uri: Uri,
    _body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let deleted = sqlx::query("DELETE FROM v2_notice WHERE id = ? AND author_user_id = ? AND scope_type = 'plan_subscribers'")
        .bind(notice_id)
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    if deleted.rows_affected() == 0 {
        return Ok(fail_json_response(StatusCode::NOT_FOUND, "Notice not found"));
    }
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}
