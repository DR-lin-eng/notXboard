use super::super::*;

#[derive(Clone, sqlx::FromRow)]
struct AdminGiftCardTemplateRow {
    id: i64,
    name: String,
    description: Option<String>,
    type_field: i64,
    status: i8,
    conditions: Option<SqlxJson<Value>>,
    rewards: SqlxJson<Value>,
    limits: Option<SqlxJson<Value>>,
    special_config: Option<SqlxJson<Value>>,
    icon: Option<String>,
    background_image: Option<String>,
    theme_color: String,
    sort: i64,
    admin_id: i64,
    created_at: i64,
    updated_at: i64,
    codes_count: i64,
    used_count: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct AdminGiftCardCodeRow {
    id: i64,
    template_id: i64,
    template_name: Option<String>,
    code: String,
    batch_id: Option<String>,
    status: i64,
    user_id: Option<i64>,
    user_email: Option<String>,
    used_at: Option<i64>,
    expires_at: Option<i64>,
    usage_count: i64,
    max_usage: i64,
    created_at: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct AdminGiftCardUsageRow {
    id: i64,
    code: Option<String>,
    template_name: Option<String>,
    user_email: Option<String>,
    invite_user_email: Option<String>,
    rewards_given: SqlxJson<Value>,
    invite_rewards: Option<SqlxJson<Value>>,
    multiplier_applied: String,
    created_at: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct AdminGiftCardDailyUsageRow {
    date: String,
    count: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct AdminGiftCardTypeStatRow {
    template_name: Option<String>,
    type_field: Option<i64>,
    count: i64,
}

pub async fn templates(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_templates_response_from_query(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn templates_post(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_templates_response_from_body(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn create_template(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_create_template_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn update_template(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_update_template_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn delete_template(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_delete_template_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn generate_codes(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_generate_codes_response(&state, headers, uri, body).await {
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

pub async fn statistics(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_statistics_response_from_query(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn statistics_post(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_statistics_response_from_body(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn codes(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_codes_response_from_query(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn codes_post(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_codes_response_from_body(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn toggle_code(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_toggle_code_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn delete_code(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_delete_code_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn export_codes(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_export_codes_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn update_code(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_update_code_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn usages(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_usages_response_from_query(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn usages_post(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_usages_response_from_body(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_templates_response_from_query(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    build_templates_response(state, params).await
}

async fn build_templates_response_from_body(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let params = value_object_to_string_map(payload.as_object());
    build_templates_response(state, params).await
}

async fn build_templates_response(
    state: &AppState,
    params: HashMap<String, String>,
) -> Result<Response<Body>, Response<Body>> {
    let per_page = params.get("per_page").and_then(|value| value.parse::<i64>().ok()).unwrap_or(15).clamp(1, 1000);
    let page = params.get("page").and_then(|value| value.parse::<i64>().ok()).unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;
    let type_filter = params.get("type").and_then(|value| value.parse::<i64>().ok()).filter(|value| (1..=10).contains(value));
    let status_filter = params.get("status").and_then(|value| value.parse::<i64>().ok()).filter(|value| matches!(*value, 0 | 1));

    let (rows, total) = load_admin_gift_card_templates(state, type_filter, status_filter, offset, per_page)
        .await
        .map_err(internal_error)?;
    let last_page = if total <= 0 { 1 } else { ((total + per_page - 1) / per_page).max(1) };

    Ok(json_value_response(json!({
        "data": rows.iter().map(serialize_admin_gift_card_template).collect::<Vec<_>>(),
        "current_page": page,
        "last_page": last_page,
        "per_page": per_page,
        "total": total
    })))
}

async fn build_create_template_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let obj = payload.as_object().ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let name = obj.get("name").and_then(|value| value.as_str()).map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "礼品卡名称不能为空"))?;
    if name.chars().count() > 255 {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    }
    let description = obj.get("description").and_then(|value| value.as_str()).map(|value| value.to_string()).filter(|value| !value.is_empty());
    let type_field = obj.get("type").and_then(parse_i64_value).filter(|value| (1..=3).contains(value))
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "无效的礼品卡类型"))?;
    let status = obj.get("status").and_then(|value| value.as_bool()).unwrap_or(true);
    let rewards = obj.get("rewards").cloned().filter(|value| value.is_object())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "奖励配置不能为空"))?;
    let conditions = normalize_optional_json_object(obj.get("conditions"))?;
    let limits = normalize_optional_json_object(obj.get("limits"))?;
    let special_config = normalize_optional_json_object(obj.get("special_config"))?;
    let icon = obj.get("icon").and_then(|value| value.as_str()).map(|value| value.trim().to_string()).filter(|value| !value.is_empty());
    let background_image = obj.get("background_image").and_then(|value| value.as_str()).map(|value| value.trim().to_string()).filter(|value| !value.is_empty());
    let theme_color = obj.get("theme_color").and_then(|value| value.as_str()).map(|value| value.trim().to_string()).filter(|value| !value.is_empty()).unwrap_or_else(|| "#1890ff".to_string());
    if !is_valid_hex_color(&theme_color) {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "主题色格式不正确"));
    }
    let sort = obj.get("sort").and_then(parse_i64_value).unwrap_or(0);
    if sort < 0 {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    }

    let now = Utc::now().timestamp();
    let result = sqlx::query(
        "INSERT INTO v2_gift_card_template
            (name, description, type, status, conditions, rewards, limits, special_config, icon, background_image, theme_color, sort, admin_id, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&name)
    .bind(description.clone())
    .bind(type_field)
    .bind(if status { 1 } else { 0 })
    .bind(conditions.map(SqlxJson))
    .bind(SqlxJson(rewards.clone()))
    .bind(limits.map(SqlxJson))
    .bind(special_config.map(SqlxJson))
    .bind(icon.clone())
    .bind(background_image.clone())
    .bind(&theme_color)
    .bind(sort)
    .bind(admin.id)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    let inserted_id = i64::try_from(result.last_insert_id()).unwrap_or_default();
    let row = load_admin_gift_card_template_by_id(state, inserted_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::INTERNAL_SERVER_ERROR, "创建失败"))?;
    Ok(json_value_response(success_response_payload(serialize_admin_gift_card_template(&row))))
}

async fn build_update_template_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let obj = payload.as_object().ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    let id = obj.get("id").and_then(parse_i64_value).filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let existing = load_admin_gift_card_template_by_id(state, id).await.map_err(internal_error)?;
    let Some(existing) = existing else {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({"message":"模板不存在"})));
    };

    let name = match obj.get("name") {
        Some(value) => value.as_str().map(|v| v.trim().to_string()).filter(|v| !v.is_empty()).ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?,
        None => existing.name.clone(),
    };
    if name.chars().count() > 255 {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    }
    let description = parse_optional_string_field(obj.get("description"))?.or(existing.description.clone());
    let type_field = match obj.get("type") {
        Some(value) => parse_i64_value(value).filter(|value| (1..=3).contains(value)).ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?,
        None => existing.type_field,
    };
    let status = obj.get("status").and_then(|value| value.as_bool()).unwrap_or(existing.status != 0);
    let rewards = match obj.get("rewards") {
        Some(value) if value.is_object() => value.clone(),
        Some(_) => return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed")),
        None => existing.rewards.0.clone(),
    };
    let conditions = merge_optional_json_object(obj.get("conditions"), existing.conditions.as_ref().map(|value| value.0.clone()))?;
    let limits = merge_optional_json_object(obj.get("limits"), existing.limits.as_ref().map(|value| value.0.clone()))?;
    let special_config = merge_optional_json_object(obj.get("special_config"), existing.special_config.as_ref().map(|value| value.0.clone()))?;
    let icon = parse_optional_string_field(obj.get("icon"))?.or(existing.icon.clone());
    let background_image = parse_optional_string_field(obj.get("background_image"))?.or(existing.background_image.clone());
    let theme_color = match obj.get("theme_color") {
        Some(value) => value.as_str().map(|v| v.trim().to_string()).filter(|v| !v.is_empty()).ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?,
        None => existing.theme_color.clone(),
    };
    if !is_valid_hex_color(&theme_color) {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "主题色格式不正确"));
    }
    let sort = match obj.get("sort") {
        Some(value) => {
            let parsed = parse_i64_value(value).ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
            if parsed < 0 {
                return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
            }
            parsed
        }
        None => existing.sort,
    };

    sqlx::query(
        "UPDATE v2_gift_card_template
         SET name = ?, description = ?, type = ?, status = ?, conditions = ?, rewards = ?, limits = ?, special_config = ?,
             icon = ?, background_image = ?, theme_color = ?, sort = ?, updated_at = ?
         WHERE id = ?"
    )
    .bind(&name)
    .bind(description.clone())
    .bind(type_field)
    .bind(if status { 1 } else { 0 })
    .bind(conditions.clone().map(SqlxJson))
    .bind(SqlxJson(rewards.clone()))
    .bind(limits.clone().map(SqlxJson))
    .bind(special_config.clone().map(SqlxJson))
    .bind(icon.clone())
    .bind(background_image.clone())
    .bind(&theme_color)
    .bind(sort)
    .bind(Utc::now().timestamp())
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    let refreshed = load_admin_gift_card_template_by_id(state, id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::INTERNAL_SERVER_ERROR, "更新失败"))?;
    Ok(json_value_response(success_response_payload(serialize_admin_gift_card_template(&refreshed))))
}

async fn build_delete_template_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let id = payload.get("id").and_then(parse_i64_value).filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let template = load_admin_gift_card_template_by_id(state, id).await.map_err(internal_error)?;
    let Some(template) = template else {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({"message":"模板不存在"})));
    };
    if template.codes_count > 0 {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"该模板下存在兑换码，无法删除"})));
    }
    sqlx::query("DELETE FROM v2_gift_card_template WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_generate_codes_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let obj = payload.as_object().ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let template_id = obj.get("template_id").and_then(parse_i64_value).filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "请选择礼品卡模板"))?;
    let count = obj.get("count").and_then(parse_i64_value).filter(|value| (1..=10_000).contains(value))
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "请指定生成数量"))?;
    let prefix = obj.get("prefix").and_then(|value| value.as_str()).map(|value| value.trim().to_string()).filter(|value| !value.is_empty()).unwrap_or_else(|| "GC".to_string());
    if prefix.len() > 10 || !prefix.chars().all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit()) {
        return Ok(json_status_response(StatusCode::UNPROCESSABLE_ENTITY, json!({"message":"前缀只能包含大写字母和数字"})));
    }
    let max_usage = obj.get("max_usage").and_then(parse_i64_value).unwrap_or(1);
    if !(1..=1000).contains(&max_usage) {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    }
    let expires_hours = obj.get("expires_hours").and_then(parse_i64_value);
    if let Some(value) = expires_hours {
        if value < 1 {
            return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
        }
    }

    let template = load_admin_gift_card_template_by_id(state, template_id).await.map_err(internal_error)?;
    let Some(template) = template else {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({"message":"模板不存在"})));
    };
    if template.status == 0 {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"模板已被禁用"})));
    }

    let batch_id = format!("batch_{}", random_alnum(13).to_lowercase());
    let now = Utc::now().timestamp();
    let expires_at = expires_hours.map(|value| now + value * 3600);

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    for _ in 0..count {
        let code = generate_unique_gift_card_code(state, &prefix, Some(&mut tx)).await.map_err(internal_error)?;
        sqlx::query(
            "INSERT INTO v2_gift_card_code
                (template_id, code, batch_id, status, expires_at, max_usage, created_at, updated_at)
             VALUES (?, ?, ?, 0, ?, ?, ?, ?)"
        )
        .bind(template_id)
        .bind(code)
        .bind(&batch_id)
        .bind(expires_at)
        .bind(max_usage)
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    }
    tx.commit().await.map_err(internal_error)?;

    Ok(json_value_response(success_response_payload(json!({
        "batch_id": batch_id,
        "count": count,
        "message": "生成成功"
    }))))
}

async fn build_types_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    Ok(json_value_response(success_response_payload(json!({
        "1": gift_card_type_name(1),
        "2": gift_card_type_name(2),
        "3": gift_card_type_name(3)
    }))))
}

async fn build_statistics_response_from_query(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    build_statistics_response(state, params).await
}

async fn build_statistics_response_from_body(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let params = value_object_to_string_map(payload.as_object());
    build_statistics_response(state, params).await
}

async fn build_statistics_response(
    state: &AppState,
    params: HashMap<String, String>,
) -> Result<Response<Body>, Response<Body>> {
    let start_date = params
        .get("start_date")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| {
            let dt = (Utc::now() - chrono::Duration::days(30)).format("%Y-%m-%d").to_string();
            dt
        });
    let end_date = params
        .get("end_date")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());

    let templates_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_gift_card_template")
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    let active_templates_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_gift_card_template WHERE status = 1")
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    let codes_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_gift_card_code")
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    let used_codes_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_gift_card_code WHERE status = 1")
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    let usages_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_gift_card_usage")
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;

    let daily_usages = sqlx::query_as::<_, AdminGiftCardDailyUsageRow>(
        "SELECT CAST(daily_usage_date AS CHAR) AS date, COUNT(*) AS count
         FROM (
            SELECT DATE(FROM_UNIXTIME(created_at)) AS daily_usage_date
            FROM v2_gift_card_usage
            WHERE DATE(FROM_UNIXTIME(created_at)) BETWEEN ? AND ?
         ) usage_days
         GROUP BY daily_usage_date
         ORDER BY daily_usage_date ASC"
    )
    .bind(&start_date)
    .bind(&end_date)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    let type_stats = sqlx::query_as::<_, AdminGiftCardTypeStatRow>(
        "SELECT t.name AS template_name, t.type AS type_field, COUNT(*) AS count
         FROM v2_gift_card_usage u
         LEFT JOIN v2_gift_card_template t ON t.id = u.template_id
         GROUP BY u.template_id, t.name, t.type
         ORDER BY count DESC, u.template_id DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(json_value_response(success_response_payload(json!({
        "total_stats": {
            "templates_count": templates_count,
            "active_templates_count": active_templates_count,
            "codes_count": codes_count,
            "used_codes_count": used_codes_count,
            "usages_count": usages_count,
        },
        "daily_usages": daily_usages.iter().map(|row| json!({
            "date": row.date,
            "count": row.count,
        })).collect::<Vec<_>>(),
        "type_stats": type_stats.iter().map(|row| json!({
            "template_name": row.template_name,
            "type_name": row.type_field.map(gift_card_type_name),
            "count": row.count,
        })).collect::<Vec<_>>(),
    }))))
}

async fn build_codes_response_from_query(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    build_codes_response(state, params).await
}

async fn build_codes_response_from_body(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let params = value_object_to_string_map(payload.as_object());
    build_codes_response(state, params).await
}

async fn build_codes_response(
    state: &AppState,
    params: HashMap<String, String>,
) -> Result<Response<Body>, Response<Body>> {
    let per_page = params.get("per_page").and_then(|value| value.parse::<i64>().ok()).unwrap_or(15).clamp(1, 500);
    let page = params.get("page").and_then(|value| value.parse::<i64>().ok()).unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;
    let template_id = params.get("template_id").and_then(|value| value.parse::<i64>().ok()).filter(|value| *value > 0);
    let batch_id = params.get("batch_id").map(|value| value.trim().to_string()).filter(|value| !value.is_empty());
    let status_filter = params.get("status").and_then(|value| value.parse::<i64>().ok()).filter(|value| (0..=3).contains(value));

    let (rows, total) = load_admin_gift_card_codes(state, template_id, batch_id.as_deref(), status_filter, offset, per_page)
        .await
        .map_err(internal_error)?;
    let last_page = if total <= 0 { 1 } else { ((total + per_page - 1) / per_page).max(1) };

    Ok(json_value_response(json!({
        "data": rows.iter().map(serialize_admin_gift_card_code).collect::<Vec<_>>(),
        "current_page": page,
        "last_page": last_page,
        "per_page": per_page,
        "total": total
    })))
}

async fn build_usages_response_from_query(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    build_usages_response(state, params).await
}

async fn build_usages_response_from_body(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let params = value_object_to_string_map(payload.as_object());
    build_usages_response(state, params).await
}

async fn build_usages_response(
    state: &AppState,
    params: HashMap<String, String>,
) -> Result<Response<Body>, Response<Body>> {
    let per_page = params.get("per_page").and_then(|value| value.parse::<i64>().ok()).unwrap_or(15).clamp(1, 500);
    let page = params.get("page").and_then(|value| value.parse::<i64>().ok()).unwrap_or(1).max(1);
    let offset = (page - 1) * per_page;
    let template_id = params.get("template_id").and_then(|value| value.parse::<i64>().ok()).filter(|value| *value > 0);
    let user_id = params.get("user_id").and_then(|value| value.parse::<i64>().ok()).filter(|value| *value > 0);

    let (rows, total) = load_admin_gift_card_usages(state, template_id, user_id, offset, per_page)
        .await
        .map_err(internal_error)?;
    let last_page = if total <= 0 { 1 } else { ((total + per_page - 1) / per_page).max(1) };

    Ok(json_value_response(json!({
        "data": rows.iter().map(serialize_admin_gift_card_usage).collect::<Vec<_>>(),
        "current_page": page,
        "last_page": last_page,
        "per_page": per_page,
        "total": total
    })))
}

async fn build_toggle_code_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let obj = payload.as_object().ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    let id = obj.get("id").and_then(parse_i64_value).filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    let action = obj.get("action").and_then(|value| value.as_str()).map(|value| value.trim().to_string())
        .filter(|value| matches!(value.as_str(), "disable" | "enable"))
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let code = load_admin_gift_card_code_by_id(state, id).await.map_err(internal_error)?;
    let Some(code) = code else {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({"message":"兑换码不存在"})));
    };

    let next_status = if action == "disable" {
        3
    } else if code.status == 3 {
        0
    } else {
        code.status
    };

    sqlx::query("UPDATE v2_gift_card_code SET status = ?, updated_at = ? WHERE id = ?")
        .bind(next_status)
        .bind(Utc::now().timestamp())
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    Ok(json_value_response(success_response_payload(json!({
        "message": if action == "disable" { "已禁用" } else { "已启用" }
    }))))
}

async fn build_delete_code_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let id = payload.get("id").and_then(parse_i64_value).filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let code = load_admin_gift_card_code_by_id(state, id).await.map_err(internal_error)?;
    let Some(code) = code else {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({"message":"礼品卡不存在"})));
    };
    if code.status == 1 {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"该礼品卡已被使用，无法删除"})));
    }
    let usage_exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_gift_card_usage WHERE code_id = ?")
        .bind(id)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    if usage_exists > 0 {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"该礼品卡存在使用记录，无法删除"})));
    }
    sqlx::query("DELETE FROM v2_gift_card_code WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(json!({
        "message": "删除成功"
    }))))
}

async fn build_export_codes_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let batch_id = params.get("batch_id").map(|value| value.trim()).filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let codes = load_gift_card_codes_by_batch_id(state, batch_id).await.map_err(internal_error)?;
    if codes.is_empty() {
        return Ok(json_status_response(StatusCode::UNPROCESSABLE_ENTITY, json!({
            "status": "fail",
            "message": "The selected batch id is invalid.",
            "data": Value::Null,
            "error": Value::Null
        })));
    }

    let content = codes.join("\n");
    let filename = format!("attachment; filename=\"gift_cards_{}.txt\"", batch_id);
    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/plain; charset=utf-8")
        .header("Content-Disposition", filename)
        .body(Body::from(content))
        .unwrap())
}

async fn build_update_code_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let obj = payload.as_object().ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    let id = obj.get("id").and_then(parse_i64_value).filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let code = load_admin_gift_card_code_by_id(state, id).await.map_err(internal_error)?;
    let Some(code) = code else {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({"message":"礼品卡不存在"})));
    };

    let expires_at = match obj.get("expires_at") {
        Some(Value::Null) => Some(None),
        Some(value) => Some(
            Some(
                parse_i64_value(value)
                    .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?
            )
        ),
        None => None,
    };
    let max_usage = match obj.get("max_usage") {
        Some(value) => {
            let parsed = parse_i64_value(value).filter(|value| (1..=1000).contains(value))
                .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
            Some(parsed)
        }
        None => None,
    };
    let status = match obj.get("status") {
        Some(value) => {
            let parsed = parse_i64_value(value).filter(|value| matches!(*value, 0..=3))
                .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
            Some(parsed)
        }
        None => None,
    };

    if expires_at.is_none() && max_usage.is_none() && status.is_none() {
        return Ok(json_value_response(success_response_payload(serialize_admin_gift_card_code(&code))));
    }

    let next_expires_at = expires_at.unwrap_or(code.expires_at);
    let next_max_usage = max_usage.unwrap_or(code.max_usage);
    let next_status = status.unwrap_or(code.status);
    let now = Utc::now().timestamp();

    sqlx::query(
        "UPDATE v2_gift_card_code
         SET expires_at = ?, max_usage = ?, status = ?, updated_at = ?
         WHERE id = ?"
    )
    .bind(next_expires_at)
    .bind(next_max_usage)
    .bind(next_status)
    .bind(now)
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    let refreshed = load_admin_gift_card_code_by_id(state, id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::INTERNAL_SERVER_ERROR, "更新失败"))?;
    Ok(json_value_response(success_response_payload(serialize_admin_gift_card_code(&refreshed))))
}

async fn load_admin_gift_card_templates(
    state: &AppState,
    type_filter: Option<i64>,
    status_filter: Option<i64>,
    offset: i64,
    limit: i64,
) -> Result<(Vec<AdminGiftCardTemplateRow>, i64), sqlx::Error> {
    let rows = sqlx::query_as::<_, AdminGiftCardTemplateRow>(
        "SELECT CAST(t.id AS SIGNED) AS id, t.name, t.description, t.type AS type_field, t.status, t.conditions, t.rewards, t.limits,
                t.special_config, t.icon, t.background_image, t.theme_color, t.sort, t.admin_id, t.created_at,
                t.updated_at,
                (SELECT COUNT(*) FROM v2_gift_card_code c WHERE c.template_id = t.id) AS codes_count,
                (SELECT COUNT(*) FROM v2_gift_card_usage u WHERE u.template_id = t.id) AS used_count
         FROM v2_gift_card_template t
         WHERE (? IS NULL OR t.type = ?)
           AND (? IS NULL OR t.status = ?)
         ORDER BY t.sort ASC, t.created_at DESC
         LIMIT ? OFFSET ?"
    )
    .bind(type_filter)
    .bind(type_filter)
    .bind(status_filter)
    .bind(status_filter)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)
         FROM v2_gift_card_template t
         WHERE (? IS NULL OR t.type = ?)
           AND (? IS NULL OR t.status = ?)"
    )
    .bind(type_filter)
    .bind(type_filter)
    .bind(status_filter)
    .bind(status_filter)
    .fetch_one(&state.db)
    .await?;

    Ok((rows, total))
}

async fn load_admin_gift_card_template_by_id(
    state: &AppState,
    id: i64,
) -> Result<Option<AdminGiftCardTemplateRow>, sqlx::Error> {
    sqlx::query_as::<_, AdminGiftCardTemplateRow>(
        "SELECT CAST(t.id AS SIGNED) AS id, t.name, t.description, t.type AS type_field, t.status, t.conditions, t.rewards, t.limits,
                t.special_config, t.icon, t.background_image, t.theme_color, t.sort, t.admin_id, t.created_at,
                t.updated_at,
                (SELECT COUNT(*) FROM v2_gift_card_code c WHERE c.template_id = t.id) AS codes_count,
                (SELECT COUNT(*) FROM v2_gift_card_usage u WHERE u.template_id = t.id) AS used_count
         FROM v2_gift_card_template t
         WHERE t.id = ?
         LIMIT 1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
}

async fn load_admin_gift_card_codes(
    state: &AppState,
    template_id: Option<i64>,
    batch_id: Option<&str>,
    status_filter: Option<i64>,
    offset: i64,
    limit: i64,
) -> Result<(Vec<AdminGiftCardCodeRow>, i64), sqlx::Error> {
    let rows = sqlx::query_as::<_, AdminGiftCardCodeRow>(
        "SELECT CAST(c.id AS SIGNED) AS id, c.template_id, t.name AS template_name, c.code, c.batch_id, c.status, c.user_id, u.email AS user_email,
                c.used_at, c.expires_at, c.usage_count, c.max_usage, c.created_at
         FROM v2_gift_card_code c
         LEFT JOIN v2_gift_card_template t ON t.id = c.template_id
         LEFT JOIN v2_user u ON u.id = c.user_id
         WHERE (? IS NULL OR c.template_id = ?)
           AND (? IS NULL OR c.batch_id = ?)
           AND (? IS NULL OR c.status = ?)
         ORDER BY c.created_at DESC
         LIMIT ? OFFSET ?"
    )
    .bind(template_id)
    .bind(template_id)
    .bind(batch_id)
    .bind(batch_id)
    .bind(status_filter)
    .bind(status_filter)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)
         FROM v2_gift_card_code c
         WHERE (? IS NULL OR c.template_id = ?)
           AND (? IS NULL OR c.batch_id = ?)
           AND (? IS NULL OR c.status = ?)"
    )
    .bind(template_id)
    .bind(template_id)
    .bind(batch_id)
    .bind(batch_id)
    .bind(status_filter)
    .bind(status_filter)
    .fetch_one(&state.db)
    .await?;

    Ok((rows, total))
}

async fn load_admin_gift_card_code_by_id(
    state: &AppState,
    id: i64,
) -> Result<Option<AdminGiftCardCodeRow>, sqlx::Error> {
    sqlx::query_as::<_, AdminGiftCardCodeRow>(
        "SELECT CAST(c.id AS SIGNED) AS id, c.template_id, t.name AS template_name, c.code, c.batch_id, c.status, c.user_id, u.email AS user_email,
                c.used_at, c.expires_at, c.usage_count, c.max_usage, c.created_at
         FROM v2_gift_card_code c
         LEFT JOIN v2_gift_card_template t ON t.id = c.template_id
         LEFT JOIN v2_user u ON u.id = c.user_id
         WHERE c.id = ?
         LIMIT 1"
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
}

async fn load_admin_gift_card_usages(
    state: &AppState,
    template_id: Option<i64>,
    user_id: Option<i64>,
    offset: i64,
    limit: i64,
) -> Result<(Vec<AdminGiftCardUsageRow>, i64), sqlx::Error> {
    let rows = sqlx::query_as::<_, AdminGiftCardUsageRow>(
        "SELECT CAST(gu.id AS SIGNED) AS id, gc.code, gt.name AS template_name, uu.email AS user_email, iu.email AS invite_user_email,
                gu.rewards_given, gu.invite_rewards, CAST(gu.multiplier_applied AS CHAR) AS multiplier_applied, gu.created_at
         FROM v2_gift_card_usage gu
         LEFT JOIN v2_gift_card_code gc ON gc.id = gu.code_id
         LEFT JOIN v2_gift_card_template gt ON gt.id = gu.template_id
         LEFT JOIN v2_user uu ON uu.id = gu.user_id
         LEFT JOIN v2_user iu ON iu.id = gu.invite_user_id
         WHERE (? IS NULL OR gu.template_id = ?)
           AND (? IS NULL OR gu.user_id = ?)
         ORDER BY gu.created_at DESC
         LIMIT ? OFFSET ?"
    )
    .bind(template_id)
    .bind(template_id)
    .bind(user_id)
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let total = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)
         FROM v2_gift_card_usage gu
         WHERE (? IS NULL OR gu.template_id = ?)
           AND (? IS NULL OR gu.user_id = ?)"
    )
    .bind(template_id)
    .bind(template_id)
    .bind(user_id)
    .bind(user_id)
    .fetch_one(&state.db)
    .await?;

    Ok((rows, total))
}

async fn load_gift_card_codes_by_batch_id(
    state: &AppState,
    batch_id: &str,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>(
        "SELECT code
         FROM v2_gift_card_code
         WHERE batch_id = ?
         ORDER BY created_at ASC"
    )
    .bind(batch_id)
    .fetch_all(&state.db)
    .await
}

fn serialize_admin_gift_card_template(row: &AdminGiftCardTemplateRow) -> Value {
    json!({
        "id": row.id,
        "name": row.name,
        "description": row.description,
        "type": row.type_field,
        "type_name": gift_card_type_name(row.type_field),
        "status": row.status != 0,
        "conditions": row.conditions.as_ref().map(|value| value.0.clone()).unwrap_or(Value::Object(Map::new())),
        "rewards": row.rewards.0.clone(),
        "limits": row.limits.as_ref().map(|value| value.0.clone()).unwrap_or(Value::Object(Map::new())),
        "special_config": row.special_config.as_ref().map(|value| value.0.clone()),
        "icon": row.icon,
        "background_image": row.background_image,
        "theme_color": row.theme_color,
        "sort": row.sort,
        "admin_id": row.admin_id,
        "created_at": row.created_at,
        "updated_at": row.updated_at,
        "codes_count": row.codes_count,
        "used_count": row.used_count,
    })
}

fn serialize_admin_gift_card_code(row: &AdminGiftCardCodeRow) -> Value {
    let user_email = row.user_email.as_deref().map(mask_email_short);
    json!({
        "id": row.id,
        "template_id": row.template_id,
        "template_name": row.template_name,
        "code": row.code,
        "batch_id": row.batch_id,
        "status": row.status,
        "status_name": gift_card_status_name(row.status, gift_card_expired(row.expires_at)),
        "user_id": row.user_id,
        "user_email": user_email,
        "used_at": row.used_at,
        "expires_at": row.expires_at,
        "usage_count": row.usage_count,
        "max_usage": row.max_usage,
        "created_at": row.created_at,
    })
}

fn serialize_admin_gift_card_usage(row: &AdminGiftCardUsageRow) -> Value {
    json!({
        "id": row.id,
        "code": row.code,
        "template_name": row.template_name,
        "user_email": row.user_email,
        "invite_user_email": row.invite_user_email.as_deref().map(mask_email_short),
        "rewards_given": row.rewards_given.0.clone(),
        "invite_rewards": row.invite_rewards.as_ref().map(|value| value.0.clone()),
        "multiplier_applied": row.multiplier_applied.parse::<f64>().unwrap_or(1.0),
        "created_at": row.created_at,
    })
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
