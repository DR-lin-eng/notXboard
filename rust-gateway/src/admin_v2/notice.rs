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

pub async fn update(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    save(State(state), headers, uri, body).await
}

pub async fn show(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_show_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn drop(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_drop_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn sort(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_sort_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_fetch_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let rows = load_admin_notices(state).await.map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Array(
        rows.iter().map(serialize_admin_notice).collect::<Vec<_>>()
    ))))
}

async fn build_save_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let obj = payload.as_object().ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let title = obj.get("title").and_then(|value| value.as_str()).map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "标题不能为空"))?;
    let content = obj.get("content").and_then(|value| value.as_str()).map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "内容不能为空"))?;
    let img_url = parse_optional_string_field(obj.get("img_url"))?;
    let tags = normalize_notice_tags(obj.get("tags"))?;
    let show = obj.get("show").and_then(|value| value.as_bool()).unwrap_or(false);
    let popup = obj.get("popup").and_then(|value| value.as_bool()).unwrap_or(false);
    let scope_type = obj.get("scope_type").and_then(|value| value.as_str()).map(|value| value.trim().to_string()).filter(|value| !value.is_empty()).unwrap_or_else(|| "global".to_string());
    if scope_type != "global" && scope_type != "plan_subscribers" {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "公告范围类型不正确"));
    }

    let (normalized_scope_type, normalized_targets) = if scope_type == "plan_subscribers" {
        let target_ids = normalize_notice_target_plan_ids(obj.get("target_plan_ids"))?;
        if target_ids.is_empty() {
            return Ok(json_status_response(StatusCode::UNPROCESSABLE_ENTITY, json!({"message":"请选择至少一个目标套餐"})));
        }
        ensure_notice_target_plans_exist(state, &target_ids).await?;
        ("plan_subscribers".to_string(), Some(serde_json::to_string(&target_ids).unwrap_or_else(|_| "[]".to_string())))
    } else {
        ("global".to_string(), Some("[]".to_string()))
    };

    let serialized_tags = tags.as_ref().map(|items| serde_json::to_string(items).unwrap_or_else(|_| "[]".to_string()));
    let now = Utc::now().timestamp();

    if let Some(id) = obj.get("id").and_then(parse_i64_value).filter(|value| *value > 0) {
        let existing = load_admin_notice_by_id(state, id).await.map_err(internal_error)?;
        let Some(existing) = existing else {
            return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"公告不存在"})));
        };
        sqlx::query(
            "UPDATE v2_notice
             SET title = ?, content = ?, img_url = ?, tags = ?, `show` = ?, popup = ?, scope_type = ?, target_plan_ids = ?, updated_at = ?
             WHERE id = ?"
        )
        .bind(&title)
        .bind(&content)
        .bind(img_url.clone())
        .bind(serialized_tags.clone())
        .bind(if show { 1 } else { 0 })
        .bind(if popup { 1 } else { 0 })
        .bind(&normalized_scope_type)
        .bind(normalized_targets.clone())
        .bind(now)
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

        if show && existing.show == 0 {
            info!("notice published by admin {} notice_id={}", admin.id, id);
            let _ = notify_notice_published(state, id).await;
        }
        return Ok(json_value_response(success_response_payload(Value::Bool(true))));
    }

    let inserted = sqlx::query(
        "INSERT INTO v2_notice
            (sort, title, content, `show`, popup, author_user_id, scope_type, target_plan_ids, img_url, tags, created_at, updated_at)
         VALUES (0, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&title)
    .bind(&content)
    .bind(if show { 1 } else { 0 })
    .bind(if popup { 1 } else { 0 })
    .bind(admin.id)
    .bind(&normalized_scope_type)
    .bind(normalized_targets.clone())
    .bind(img_url.clone())
    .bind(serialized_tags.clone())
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    if show {
        info!("notice published by admin {} notice_id={}", admin.id, inserted.last_insert_id());
        let _ = notify_notice_published(state, inserted.last_insert_id() as i64).await;
    }
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_show_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let id = payload.get("id").and_then(parse_i64_value).filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "公告ID不能为空"))?;

    let notice = load_admin_notice_by_id(state, id).await.map_err(internal_error)?;
    let Some(notice) = notice else {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"公告不存在"})));
    };
    let next_show = if notice.show == 0 { 1 } else { 0 };
    sqlx::query("UPDATE v2_notice SET `show` = ?, updated_at = ? WHERE id = ?")
        .bind(next_show)
        .bind(Utc::now().timestamp())
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    if next_show == 1 {
        info!("notice published by admin {} notice_id={}", admin.id, id);
        let _ = notify_notice_published(state, id).await;
    }
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_drop_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let id = payload.get("id").and_then(parse_i64_value).filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "公告ID不能为空"))?;

    let deleted = sqlx::query("DELETE FROM v2_notice WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    if deleted.rows_affected() == 0 {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"公告不存在"})));
    }
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_sort_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let ids = payload
        .get("ids")
        .and_then(|value| value.as_array())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "参数有误"))?;
    if ids.is_empty() {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "参数有误"));
    }

    let parsed_ids = ids
        .iter()
        .filter_map(parse_i64_value)
        .filter(|value| *value > 0)
        .collect::<Vec<_>>();
    if parsed_ids.len() != ids.len() {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "参数有误"));
    }

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    for (index, notice_id) in parsed_ids.iter().enumerate() {
        let updated = sqlx::query("UPDATE v2_notice SET sort = ?, updated_at = ? WHERE id = ?")
            .bind((index + 1) as i64)
            .bind(Utc::now().timestamp())
            .bind(*notice_id)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
        if updated.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"公告不存在"})));
        }
    }
    tx.commit().await.map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn load_admin_notices(state: &AppState) -> Result<Vec<NoticeRow>, sqlx::Error> {
    sqlx::query_as::<_, NoticeRow>(
        "SELECT id, sort, title, content, `show`, popup, author_user_id, scope_type, target_plan_ids, img_url, tags, created_at, updated_at
         FROM v2_notice
         ORDER BY sort ASC, id DESC"
    )
    .fetch_all(&state.db)
    .await
}

async fn load_admin_notice_by_id(
    state: &AppState,
    notice_id: i64,
) -> Result<Option<NoticeRow>, sqlx::Error> {
    sqlx::query_as::<_, NoticeRow>(
        "SELECT id, sort, title, content, `show`, popup, author_user_id, scope_type, target_plan_ids, img_url, tags, created_at, updated_at
         FROM v2_notice
         WHERE id = ?
         LIMIT 1"
    )
    .bind(notice_id)
    .fetch_optional(&state.db)
    .await
}

async fn ensure_notice_target_plans_exist(
    state: &AppState,
    plan_ids: &[i64],
) -> Result<(), Response<Body>> {
    if plan_ids.is_empty() {
        return Ok(());
    }
    let placeholders = vec!["?"; plan_ids.len()].join(",");
    let sql = format!("SELECT COUNT(*) FROM v2_plan WHERE id IN ({})", placeholders);
    let mut query = sqlx::query_scalar::<_, i64>(&sql);
    for plan_id in plan_ids {
        query = query.bind(*plan_id);
    }
    let count = query.fetch_one(&state.db).await.map_err(internal_error)?;
    if count != plan_ids.len() as i64 {
        return Err(json_status_response(StatusCode::UNPROCESSABLE_ENTITY, json!({"message":"目标套餐不存在"})));
    }
    Ok(())
}

fn serialize_admin_notice(row: &NoticeRow) -> Value {
    json!({
        "id": row.id,
        "sort": row.sort,
        "title": row.title,
        "content": row.content,
        "show": row.show != 0,
        "popup": row.popup != 0,
        "author_user_id": row.author_user_id,
        "scope_type": row.scope_type,
        "target_plan_ids": row.target_plan_ids.as_deref().map(parse_notice_target_plan_ids).unwrap_or_default(),
        "img_url": row.img_url,
        "tags": row.tags.as_deref().map(parse_notice_tags).unwrap_or_default(),
        "created_at": row.created_at,
        "updated_at": row.updated_at,
    })
}

fn normalize_notice_tags(value: Option<&Value>) -> Result<Option<Vec<String>>, Response<Body>> {
    match value {
        Some(Value::Null) | None => Ok(None),
        Some(Value::Array(items)) => Ok(Some(
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect(),
        )),
        Some(_) => Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "标签格式不正确")),
    }
}

fn normalize_notice_target_plan_ids(value: Option<&Value>) -> Result<Vec<i64>, Response<Body>> {
    match value {
        Some(Value::Null) | None => Ok(Vec::new()),
        Some(Value::Array(items)) => Ok(
            items
                .iter()
                .filter_map(parse_i64_value)
                .filter(|item| *item > 0)
                .collect::<Vec<_>>()
        ),
        Some(_) => Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "目标套餐格式不正确")),
    }
}
