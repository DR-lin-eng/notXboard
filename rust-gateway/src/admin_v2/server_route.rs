use super::super::*;

#[derive(Clone, sqlx::FromRow)]
struct ServerRouteRow {
    id: i64,
    remarks: String,
    action: String,
    action_value: Option<String>,
    route_match: String,
    created_at: i64,
    updated_at: i64,
}

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

async fn build_fetch_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let rows = sqlx::query_as::<_, ServerRouteRow>(
        "SELECT id, remarks, action, action_value, `match` AS route_match, created_at, updated_at
         FROM v2_server_route
         ORDER BY id DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(json_value_response(success_response_payload(Value::Array(
        rows.into_iter()
            .map(serialize_server_route)
            .collect::<Vec<_>>()
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
    let obj = payload
        .as_object()
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let remarks = obj
        .get("remarks")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "备注不能为空"))?;
    let action = obj
        .get("action")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "动作类型不能为空"))?;
    if action != "block" && action != "dns" {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "动作类型参数有误"));
    }

    let matches = normalize_route_match_list(obj.get("match"))?;
    let action_value = parse_optional_string_field(obj.get("action_value"))?;
    let match_text = serde_json::to_string(&matches).unwrap_or_else(|_| "[]".to_string());
    let now = Utc::now().timestamp();

    if let Some(id) = obj.get("id").and_then(parse_i64_value).filter(|value| *value > 0) {
        let updated = sqlx::query(
            "UPDATE v2_server_route
             SET remarks = ?, `match` = ?, action = ?, action_value = ?, updated_at = ?
             WHERE id = ?"
        )
        .bind(&remarks)
        .bind(&match_text)
        .bind(&action)
        .bind(action_value.clone())
        .bind(now)
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
        if updated.rows_affected() == 0 {
            return Ok(json_status_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"message":"保存失败"})));
        }
        return Ok(json_value_response(success_response_payload(Value::Bool(true))));
    }

    sqlx::query(
        "INSERT INTO v2_server_route
            (remarks, `match`, action, action_value, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&remarks)
    .bind(&match_text)
    .bind(&action)
    .bind(action_value.clone())
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|_| json_status_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"message":"创建失败"})))?;
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
    let route_id = payload
        .get("id")
        .and_then(parse_i64_value)
        .filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let deleted = sqlx::query("DELETE FROM v2_server_route WHERE id = ?")
        .bind(route_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    if deleted.rows_affected() == 0 {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"路由不存在"})));
    }
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

fn normalize_route_match_list(value: Option<&Value>) -> Result<Vec<String>, Response<Body>> {
    let items = value
        .and_then(|raw| raw.as_array())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "匹配值不能为空"))?;
    Ok(items
        .iter()
        .filter_map(|item| item.as_str())
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>())
}

fn serialize_server_route(row: ServerRouteRow) -> Value {
    json!({
        "id": row.id,
        "remarks": row.remarks,
        "action": row.action,
        "action_value": row.action_value,
        "match": serde_json::from_str::<Value>(&row.route_match).unwrap_or(Value::Array(Vec::new())),
        "created_at": row.created_at,
        "updated_at": row.updated_at,
    })
}
