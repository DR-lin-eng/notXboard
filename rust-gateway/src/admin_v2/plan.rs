use super::super::*;
use crate::guest_v1::public::serialize_guest_plan;

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
    let plans = load_legacy_plans(state).await.map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Array(
        plans.iter().map(serialize_legacy_plan).collect::<Vec<_>>()
    ))))
}

async fn build_save_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let obj = payload.as_object().ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let name = obj.get("name").and_then(|value| value.as_str()).map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "套餐名称不能为空"))?;
    let content = parse_optional_string_field(obj.get("content"))?;
    let reset_traffic_method = parse_optional_i64_field(obj.get("reset_traffic_method"))?;
    let transfer_enable_gb = obj.get("transfer_enable").and_then(parse_i64_value).filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "流量配额不能为空"))?;
    let transfer_enable = gb_to_bytes_u64(transfer_enable_gb)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "流量配额格式错误"))?;
    let prices = normalize_plan_prices(obj.get("prices"))?;
    let group_id = parse_optional_i64_field(obj.get("group_id"))?;
    let speed_limit = parse_optional_i64_field(obj.get("speed_limit"))?;
    let device_limit = parse_optional_i64_field(obj.get("device_limit"))?;
    let capacity_limit = parse_optional_i64_field(obj.get("capacity_limit"))?;
    let tags = normalize_string_array_field(obj.get("tags"))?;
    let now = Utc::now().timestamp();

    if let Some(id) = obj.get("id").and_then(parse_i64_value).filter(|value| *value > 0) {
        let existing = load_legacy_plan_by_id(state, id).await.map_err(internal_error)?;
        let Some(_) = existing else {
            return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"该订阅不存在"})));
        };
        sqlx::query(
            "UPDATE v2_plan
             SET name = ?, content = ?, reset_traffic_method = ?, transfer_enable = ?, prices = ?, group_id = ?,
                 speed_limit = ?, device_limit = ?, capacity_limit = ?, tags = ?, updated_at = ?
             WHERE id = ? AND scope = 'legacy'"
        )
        .bind(&name)
        .bind(content.clone())
        .bind(reset_traffic_method)
        .bind(transfer_enable)
        .bind(prices.to_string())
        .bind(group_id)
        .bind(speed_limit)
        .bind(device_limit)
        .bind(capacity_limit)
        .bind(tags.as_ref().map(|items| serde_json::to_string(items).unwrap_or_else(|_| "[]".to_string())))
        .bind(now)
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
        return Ok(json_value_response(success_response_payload(Value::Bool(true))));
    }

    sqlx::query(
        "INSERT INTO v2_plan
            (group_id, transfer_enable, is_unlimited_traffic, name, scope, speed_limit, `show`, visibility_scope, sort, renew, content, prices, reset_traffic_method, capacity_limit, sell, device_limit, tags, created_at, updated_at)
         VALUES (?, ?, 0, ?, 'legacy', ?, 1, 'public', 0, 1, ?, ?, ?, ?, 1, ?, ?, ?, ?)"
    )
    .bind(group_id)
    .bind(transfer_enable)
    .bind(&name)
    .bind(speed_limit)
    .bind(content.clone())
    .bind(prices.to_string())
    .bind(reset_traffic_method)
    .bind(capacity_limit)
    .bind(device_limit)
    .bind(tags.as_ref().map(|items| serde_json::to_string(items).unwrap_or_else(|_| "[]".to_string())))
    .bind(now)
    .bind(now)
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
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let existing = load_legacy_plan_by_id(state, id).await.map_err(internal_error)?;
    let Some(existing) = existing else {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"该订阅不存在"})));
    };

    let show = obj.get("show").and_then(|value| value.as_i64()).map(|value| value != 0).unwrap_or(existing.show);
    let sell = obj.get("sell").and_then(|value| value.as_i64()).map(|value| value != 0).unwrap_or(existing.sell);
    let renew = obj.get("renew").and_then(|value| value.as_i64()).map(|value| value != 0).unwrap_or(existing.renew);

    sqlx::query(
        "UPDATE v2_plan
         SET `show` = ?, sell = ?, renew = ?, updated_at = ?
         WHERE id = ? AND scope = 'legacy'"
    )
    .bind(show)
    .bind(sell)
    .bind(renew)
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
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let order_exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_order WHERE plan_id = ?")
        .bind(id)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    if order_exists > 0 {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"该订阅下存在订单无法删除"})));
    }
    let user_exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_user WHERE plan_id = ?")
        .bind(id)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    if user_exists > 0 {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"该订阅下存在用户无法删除"})));
    }
    let deleted = sqlx::query("DELETE FROM v2_plan WHERE id = ? AND scope = 'legacy'")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    if deleted.rows_affected() == 0 {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"该订阅不存在"})));
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
    for (index, plan_id) in parsed_ids.iter().enumerate() {
        let updated = sqlx::query(
            "UPDATE v2_plan SET sort = ?, updated_at = ? WHERE id = ? AND scope = 'legacy'"
        )
        .bind((index + 1) as i64)
        .bind(Utc::now().timestamp())
        .bind(*plan_id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
        if updated.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"该订阅不存在"})));
        }
    }
    tx.commit().await.map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn load_legacy_plans(
    state: &AppState,
) -> Result<Vec<PlanRow>, sqlx::Error> {
    sqlx::query_as::<_, PlanRow>(
        "SELECT
            p.id, p.scope, p.owner_user_id, p.min_trust_level, p.free_quota_gb_by_trust_level,
            p.node_ids, p.group_id, p.transfer_enable, COALESCE(p.is_unlimited_traffic, 0) AS is_unlimited_traffic,
            p.name, p.speed_limit, p.`show`, COALESCE(p.visibility_scope, 'public') AS visibility_scope,
            p.access_user_ids, p.share_token, p.sort, p.renew, p.content, p.prices,
            p.reset_traffic_method, p.capacity_limit, p.sell, p.device_limit, p.tags,
            p.created_at, p.updated_at,
            owner.email AS owner_email, owner.linux_do_username AS owner_linux_do_username, owner.linux_do_name AS owner_linux_do_name
         FROM v2_plan p
         LEFT JOIN v2_user owner ON owner.id = p.owner_user_id
         WHERE p.scope = 'legacy'
         ORDER BY p.sort ASC, p.id DESC"
    )
    .fetch_all(&state.db)
    .await
}

async fn load_legacy_plan_by_id(
    state: &AppState,
    plan_id: i64,
) -> Result<Option<PlanRow>, sqlx::Error> {
    sqlx::query_as::<_, PlanRow>(
        "SELECT
            p.id, p.scope, p.owner_user_id, p.min_trust_level, p.free_quota_gb_by_trust_level,
            p.node_ids, p.group_id, p.transfer_enable, COALESCE(p.is_unlimited_traffic, 0) AS is_unlimited_traffic,
            p.name, p.speed_limit, p.`show`, COALESCE(p.visibility_scope, 'public') AS visibility_scope,
            p.access_user_ids, p.share_token, p.sort, p.renew, p.content, p.prices,
            p.reset_traffic_method, p.capacity_limit, p.sell, p.device_limit, p.tags,
            p.created_at, p.updated_at,
            owner.email AS owner_email, owner.linux_do_username AS owner_linux_do_username, owner.linux_do_name AS owner_linux_do_name
         FROM v2_plan p
         LEFT JOIN v2_user owner ON owner.id = p.owner_user_id
         WHERE p.id = ? AND p.scope = 'legacy'
         LIMIT 1"
    )
    .bind(plan_id)
    .fetch_optional(&state.db)
    .await
}

fn serialize_legacy_plan(plan: &PlanRow) -> Value {
    let mut value = serialize_guest_plan(plan, "", 0);
    if let Some(object) = value.as_object_mut() {
        object.insert("prices".to_string(), plan.prices.as_ref().map(|value| value.0.clone()).unwrap_or(Value::Object(Map::new())));
        object.insert("group".to_string(), match plan.group_id {
            Some(group_id) => json!({"id": group_id, "name": format!("Group-{}", group_id)}),
            None => Value::Null,
        });
        object.insert("users_count".to_string(), Value::from(0));
        object.insert("active_users_count".to_string(), Value::from(0));
    }
    value
}

fn normalize_plan_prices(value: Option<&Value>) -> Result<Value, Response<Body>> {
    match value {
        Some(Value::Object(object)) => {
            let mut out = Map::new();
            for (key, raw) in object {
                if let Some(number) = raw.as_f64() {
                    if number > 0.0 {
                        out.insert(key.clone(), Value::from((number * 100.0).round() / 100.0));
                    }
                }
            }
            Ok(Value::Object(out))
        }
        Some(Value::Null) | None => Ok(Value::Object(Map::new())),
        Some(_) => Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "价格配置格式错误")),
    }
}

fn normalize_string_array_field(value: Option<&Value>) -> Result<Option<Vec<String>>, Response<Body>> {
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
        Some(_) => Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "标签格式必须是数组")),
    }
}

fn gb_to_bytes_u64(gb: i64) -> Option<u64> {
    if gb <= 0 {
        return None;
    }
    let bytes = (gb as i128).checked_mul(1024)?.checked_mul(1024)?.checked_mul(1024)?;
    u64::try_from(bytes).ok()
}
