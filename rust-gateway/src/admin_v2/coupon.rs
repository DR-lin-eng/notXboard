use super::super::*;

pub async fn fetch(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_fetch_response_from_query(&state, headers, uri).await {
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

pub async fn generate(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_generate_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
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

pub async fn update(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_update_response(&state, headers, uri, body).await {
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

async fn build_fetch_response_from_query(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    build_fetch_response(state, params).await
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
    build_fetch_response(state, params).await
}

async fn build_fetch_response(
    state: &AppState,
    params: HashMap<String, String>,
) -> Result<Response<Body>, Response<Body>> {
    let current = params.get("current").and_then(|value| value.parse::<i64>().ok()).unwrap_or(1).max(1);
    let page_size = params.get("pageSize").and_then(|value| value.parse::<i64>().ok()).unwrap_or(10).clamp(1, 100);
    let offset = (current - 1) * page_size;

    let (rows, total) = load_admin_coupons(state, offset, page_size).await.map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(json!({
        "data": rows.iter().map(serialize_coupon).collect::<Vec<_>>(),
        "total": total,
        "current": current,
        "pageSize": page_size,
    }))))
}

async fn build_generate_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let obj = payload.as_object().ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let generate_count = obj.get("generate_count").and_then(parse_i64_value).unwrap_or(0);
    if generate_count < 0 || generate_count > 500 {
        return Ok(json_status_response(StatusCode::UNPROCESSABLE_ENTITY, json!({"message":"生成数量最大为500个"})));
    }
    let name = obj.get("name").and_then(|value| value.as_str()).map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "名称不能为空"))?;
    let type_field = obj.get("type").and_then(|value| value.as_str()).and_then(|value| value.trim().parse::<i64>().ok())
        .or_else(|| obj.get("type").and_then(parse_i64_value))
        .filter(|value| matches!(*value, 1 | 2))
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "类型格式有误"))?;
    let value = obj.get("value").and_then(parse_i64_value).filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "金额或比例不能为空"))?;
    let started_at = obj.get("started_at").and_then(parse_i64_value).filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "开始时间不能为空"))?;
    let ended_at = obj.get("ended_at").and_then(parse_i64_value).filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "结束时间不能为空"))?;
    let limit_use = parse_optional_i64_field(obj.get("limit_use"))?;
    let limit_use_with_user = parse_optional_i64_field(obj.get("limit_use_with_user"))?;
    let limit_plan_ids = normalize_coupon_plan_ids_from_value(obj.get("limit_plan_ids"))?;
    let limit_periods = normalize_coupon_periods_from_value(obj.get("limit_period"))?;
    let fixed_code = obj.get("code").and_then(|value| value.as_str()).map(|value| value.trim().to_string()).filter(|value| !value.is_empty());
    let now = Utc::now().timestamp();

    if generate_count > 0 {
        let mut tx = state.db.begin().await.map_err(internal_error)?;
        let mut generated_codes = Vec::new();
        for index in 0..generate_count {
            let code = if index == 0 {
                if let Some(code) = fixed_code.clone() {
                    code
                } else {
                    generate_unique_coupon_code(state, Some(&mut tx)).await.map_err(internal_error)?
                }
            } else {
                generate_unique_coupon_code(state, Some(&mut tx)).await.map_err(internal_error)?
            };
            insert_coupon_row(
                &mut tx,
                &name,
                &code,
                type_field,
                value,
                started_at,
                ended_at,
                limit_use,
                limit_use_with_user,
                limit_plan_ids.as_deref(),
                limit_periods.as_deref(),
                now,
            )
            .await
            .map_err(internal_error)?;
            generated_codes.push(code);
        }
        tx.commit().await.map_err(internal_error)?;
        return Ok(json_value_response(success_response_payload(json!({
            "generated_count": generate_count,
            "codes": generated_codes,
        }))));
    }

    let code = fixed_code.unwrap_or_else(|| random_alnum(8).to_uppercase());
    insert_coupon_direct(
        state,
        &name,
        &code,
        type_field,
        value,
        started_at,
        ended_at,
        limit_use,
        limit_use_with_user,
        limit_plan_ids.as_deref(),
        limit_periods.as_deref(),
        now,
    )
    .await
    .map_err(internal_error)?;

    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_show_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let id = payload.get("id").and_then(parse_i64_value).filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "优惠券ID不能为空"))?;

    let coupon = load_admin_coupon_by_id(state, id).await.map_err(internal_error)?;
    let Some(coupon) = coupon else {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"优惠券不存在"})));
    };
    sqlx::query("UPDATE v2_coupon SET `show` = ?, updated_at = ? WHERE id = ?")
        .bind(if coupon.show { 0 } else { 1 })
        .bind(Utc::now().timestamp())
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_update_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let obj = payload.as_object().ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    let id = obj.get("id").and_then(parse_i64_value).filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "优惠券ID不能为空"))?;

    let coupon = load_admin_coupon_by_id(state, id).await.map_err(internal_error)?;
    let Some(_) = coupon else {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"优惠券不存在"})));
    };

    let show = match obj.get("show") {
        Some(Value::Bool(flag)) => Some(*flag),
        Some(Value::Number(number)) => match number.as_i64() {
            Some(0) => Some(false),
            Some(1) => Some(true),
            _ => return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed")),
        },
        Some(Value::String(text)) => match text.trim() {
            "0" => Some(false),
            "1" => Some(true),
            "false" | "FALSE" => Some(false),
            "true" | "TRUE" => Some(true),
            _ => return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed")),
        },
        Some(Value::Null) | None => None,
        Some(_) => return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed")),
    };

    let Some(show) = show else {
        return Ok(json_value_response(success_response_payload(Value::Bool(true))));
    };

    sqlx::query("UPDATE v2_coupon SET `show` = ?, updated_at = ? WHERE id = ?")
        .bind(show)
        .bind(Utc::now().timestamp())
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
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
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "优惠券ID不能为空"))?;

    let deleted = sqlx::query("DELETE FROM v2_coupon WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    if deleted.rows_affected() == 0 {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"优惠券不存在"})));
    }
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn load_admin_coupons(
    state: &AppState,
    offset: i64,
    limit: i64,
) -> Result<(Vec<CouponRow>, i64), sqlx::Error> {
    let rows = sqlx::query_as::<_, CouponRow>(
        "SELECT id, code, name, owner_user_id, source_plan_id, type AS type_field, value, `show`, limit_use, limit_use_with_user,
                limit_plan_ids, limit_period, started_at, ended_at, created_at, updated_at
         FROM v2_coupon
         ORDER BY created_at DESC
         LIMIT ? OFFSET ?"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM v2_coupon")
        .fetch_one(&state.db)
        .await?;
    Ok((rows, total))
}

async fn load_admin_coupon_by_id(
    state: &AppState,
    coupon_id: i64,
) -> Result<Option<CouponRow>, sqlx::Error> {
    sqlx::query_as::<_, CouponRow>(
        "SELECT id, code, name, owner_user_id, source_plan_id, type AS type_field, value, `show`, limit_use, limit_use_with_user,
                limit_plan_ids, limit_period, started_at, ended_at, created_at, updated_at
         FROM v2_coupon
         WHERE id = ?
         LIMIT 1"
    )
    .bind(coupon_id)
    .fetch_optional(&state.db)
    .await
}

async fn generate_unique_coupon_code(
    state: &AppState,
    mut tx: Option<&mut sqlx::Transaction<'_, sqlx::MySql>>,
) -> Result<String, sqlx::Error> {
    loop {
        let candidate = random_alnum(8).to_uppercase();
        let exists = if let Some(current_tx) = tx.as_deref_mut() {
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM v2_coupon WHERE code = ?")
                .bind(&candidate)
                .fetch_one(&mut **current_tx)
                .await?
        } else {
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM v2_coupon WHERE code = ?")
                .bind(&candidate)
                .fetch_one(&state.db)
                .await?
        };
        if exists == 0 {
            return Ok(candidate);
        }
    }
}

async fn insert_coupon_row(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    name: &str,
    code: &str,
    type_field: i64,
    value: i64,
    started_at: i64,
    ended_at: i64,
    limit_use: Option<i64>,
    limit_use_with_user: Option<i64>,
    limit_plan_ids: Option<&str>,
    limit_period: Option<&str>,
    now: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO v2_coupon
            (code, name, type, value, `show`, limit_use, limit_use_with_user, limit_plan_ids, limit_period, started_at, ended_at, created_at, updated_at)
         VALUES (?, ?, ?, ?, 1, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(code)
    .bind(name)
    .bind(type_field)
    .bind(value)
    .bind(limit_use)
    .bind(limit_use_with_user)
    .bind(limit_plan_ids)
    .bind(limit_period)
    .bind(started_at)
    .bind(ended_at)
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn insert_coupon_direct(
    state: &AppState,
    name: &str,
    code: &str,
    type_field: i64,
    value: i64,
    started_at: i64,
    ended_at: i64,
    limit_use: Option<i64>,
    limit_use_with_user: Option<i64>,
    limit_plan_ids: Option<&str>,
    limit_period: Option<&str>,
    now: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO v2_coupon
            (code, name, type, value, `show`, limit_use, limit_use_with_user, limit_plan_ids, limit_period, started_at, ended_at, created_at, updated_at)
         VALUES (?, ?, ?, ?, 1, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(code)
    .bind(name)
    .bind(type_field)
    .bind(value)
    .bind(limit_use)
    .bind(limit_use_with_user)
    .bind(limit_plan_ids)
    .bind(limit_period)
    .bind(started_at)
    .bind(ended_at)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await?;
    Ok(())
}

fn normalize_coupon_plan_ids_from_value(value: Option<&Value>) -> Result<Option<String>, Response<Body>> {
    match value {
        Some(Value::Null) | None => Ok(None),
        Some(Value::Array(items)) => {
            let ids = items
                .iter()
                .filter_map(parse_i64_value)
                .filter(|id| *id > 0)
                .collect::<Vec<_>>();
            if ids.is_empty() {
                Ok(None)
            } else {
                Ok(Some(serde_json::to_string(&ids).unwrap_or_else(|_| "[]".to_string())))
            }
        }
        Some(_) => Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "指定订阅格式有误")),
    }
}

fn normalize_coupon_periods_from_value(value: Option<&Value>) -> Result<Option<String>, Response<Body>> {
    match value {
        Some(Value::Null) | None => Ok(None),
        Some(Value::Array(items)) => {
            let periods = items
                .iter()
                .filter_map(|item| item.as_str())
                .filter_map(normalize_coupon_period)
                .map(str::to_string)
                .collect::<Vec<_>>();
            if periods.is_empty() {
                Ok(None)
            } else {
                Ok(Some(serde_json::to_string(&periods).unwrap_or_else(|_| "[]".to_string())))
            }
        }
        Some(_) => Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "指定周期格式有误")),
    }
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
