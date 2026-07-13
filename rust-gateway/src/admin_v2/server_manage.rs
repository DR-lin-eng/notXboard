use super::super::*;
use super::server_manage_protocol::normalize_server_save_input;

#[derive(Clone, sqlx::FromRow)]
struct LegacyServerRow {
    id: u64,
    server_type: String,
    code: Option<String>,
    parent_id: Option<i64>,
    group_ids: Option<SqlxJson<Value>>,
    route_ids: Option<SqlxJson<Value>>,
    name: String,
    rate: String,
    rate_time_enable: bool,
    rate_time_ranges: Option<SqlxJson<Value>>,
    tags: Option<SqlxJson<Value>>,
    host: String,
    port: String,
    server_port: i64,
    protocol_settings: Option<SqlxJson<Value>>,
    show: bool,
    sort: Option<i64>,
    created_at: Option<chrono::DateTime<Utc>>,
    updated_at: Option<chrono::DateTime<Utc>>,
}

pub async fn get_nodes(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_nodes_response(&state, headers, uri).await {
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

pub async fn copy(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_copy_response(&state, headers, uri, body).await {
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

async fn build_get_nodes_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let rows = load_legacy_servers(state).await.map_err(internal_error)?;
    let group_names = load_server_group_name_map(state).await.map_err(internal_error)?;
    let route_names = load_server_route_name_map(state).await.map_err(internal_error)?;
    let server_token = get_setting_string(state, "server_token", "").await;

    Ok(json_value_response(success_response_payload(Value::Array(
        rows.iter()
            .map(|row| serialize_legacy_server(row, &group_names, &route_names, &server_token))
            .collect::<Vec<_>>()
    ))))
}

async fn build_sort_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let items = payload
        .as_array()
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    for item in items {
        let row = item
            .as_object()
            .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
        let id = row
            .get("id")
            .and_then(parse_i64_value)
            .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
        let order = row
            .get("order")
            .and_then(parse_i64_value)
            .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

        sqlx::query("UPDATE v2_server SET sort = ? WHERE id = ?")
            .bind(order)
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|_| json_status_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"message":"保存失败"})))?;
    }
    tx.commit().await.map_err(|_| json_status_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"message":"保存失败"})))?;
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
    let obj = payload
        .as_object()
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    let id = obj
        .get("id")
        .and_then(parse_i64_value)
        .filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    let show = obj
        .get("show")
        .and_then(parse_i64_value)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let updated = sqlx::query("UPDATE v2_server SET `show` = ? WHERE id = ?")
        .bind(show)
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|_| json_status_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"message":"保存失败"})))?;
    if updated.rows_affected() == 0 {
        return Ok(json_status_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"message":"保存失败"})));
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
    let id = payload
        .get("id")
        .and_then(parse_i64_value)
        .filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let deleted = sqlx::query("DELETE FROM v2_server WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|_| json_status_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"message":"删除失败"})))?;
    if deleted.rows_affected() == 0 {
        return Ok(json_status_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"message":"删除失败"})));
    }
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_copy_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let id = payload
        .get("id")
        .and_then(parse_i64_value)
        .filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let row = load_legacy_server_by_id(state, id).await.map_err(internal_error)?;
    let Some(row) = row else {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"服务器不存在"})));
    };

    sqlx::query(
        "INSERT INTO v2_server
            (`type`, code, parent_id, group_ids, route_ids, name, rate, rate_time_enable, rate_time_ranges, tags, host, port, server_port, protocol_settings, `show`, sort, created_at, updated_at)
         VALUES (?, NULL, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?, ?)"
    )
    .bind(&row.server_type)
    .bind(row.parent_id)
    .bind(row.group_ids.as_ref().map(|v| v.0.to_string()))
    .bind(row.route_ids.as_ref().map(|v| v.0.to_string()))
    .bind(&row.name)
    .bind(&row.rate)
    .bind(row.rate_time_enable)
    .bind(row.rate_time_ranges.as_ref().map(|v| v.0.to_string()))
    .bind(row.tags.as_ref().map(|v| v.0.to_string()))
    .bind(&row.host)
    .bind(&row.port)
    .bind(row.server_port)
    .bind(row.protocol_settings.as_ref().map(|v| v.0.to_string()))
    .bind(row.sort)
    .bind(Utc::now())
    .bind(Utc::now())
    .execute(&state.db)
    .await
    .map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
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
    let normalized = normalize_server_save_input(obj)?;
    let now = Utc::now();

    if let Some(id) = obj.get("id").and_then(parse_i64_value).filter(|value| *value > 0) {
        let updated = sqlx::query(
            "UPDATE v2_server
             SET `type` = ?, code = ?, parent_id = ?, group_ids = ?, route_ids = ?, name = ?, rate = ?,
                 rate_time_enable = ?, rate_time_ranges = ?, tags = ?, host = ?, port = ?, server_port = ?,
                 protocol_settings = ?, `show` = ?, updated_at = ?
             WHERE id = ?"
        )
        .bind(&normalized.server_type)
        .bind(normalized.code.clone())
        .bind(normalized.parent_id)
        .bind(&normalized.group_ids_json)
        .bind(&normalized.route_ids_json)
        .bind(&normalized.name)
        .bind(&normalized.rate)
        .bind(normalized.rate_time_enable)
        .bind(&normalized.rate_time_ranges_json)
        .bind(&normalized.tags_json)
        .bind(&normalized.host)
        .bind(&normalized.port)
        .bind(normalized.server_port)
        .bind(&normalized.protocol_settings_json)
        .bind(normalized.show)
        .bind(now)
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|_| json_status_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"message":"保存失败"})))?;
        if updated.rows_affected() == 0 {
            return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"服务器不存在"})));
        }
        return Ok(json_value_response(success_response_payload(Value::Bool(true))));
    }

    sqlx::query(
        "INSERT INTO v2_server
            (`type`, code, parent_id, group_ids, route_ids, name, rate, rate_time_enable, rate_time_ranges, tags, host, port, server_port, protocol_settings, `show`, sort, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?)"
    )
    .bind(&normalized.server_type)
    .bind(normalized.code.clone())
    .bind(normalized.parent_id)
    .bind(&normalized.group_ids_json)
    .bind(&normalized.route_ids_json)
    .bind(&normalized.name)
    .bind(&normalized.rate)
    .bind(normalized.rate_time_enable)
    .bind(&normalized.rate_time_ranges_json)
    .bind(&normalized.tags_json)
    .bind(&normalized.host)
    .bind(&normalized.port)
    .bind(normalized.server_port)
    .bind(&normalized.protocol_settings_json)
    .bind(normalized.show)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|_| json_status_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"message":"创建失败"})))?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn load_legacy_servers(state: &AppState) -> Result<Vec<LegacyServerRow>, sqlx::Error> {
    sqlx::query_as::<_, LegacyServerRow>(
        "SELECT
            id,
            `type` AS server_type,
            code,
            parent_id,
            group_ids,
            route_ids,
            name,
            CAST(rate AS CHAR) AS rate,
            rate_time_enable,
            rate_time_ranges,
            tags,
            host,
            port,
            server_port,
            protocol_settings,
            `show`,
            CAST(sort AS SIGNED) AS sort,
            created_at,
            updated_at
         FROM v2_server
         ORDER BY sort ASC, id ASC"
    )
    .fetch_all(&state.db)
    .await
}

async fn load_legacy_server_by_id(
    state: &AppState,
    server_id: i64,
) -> Result<Option<LegacyServerRow>, sqlx::Error> {
    sqlx::query_as::<_, LegacyServerRow>(
        "SELECT
            id,
            `type` AS server_type,
            code,
            parent_id,
            group_ids,
            route_ids,
            name,
            CAST(rate AS CHAR) AS rate,
            rate_time_enable,
            rate_time_ranges,
            tags,
            host,
            port,
            server_port,
            protocol_settings,
            `show`,
            CAST(sort AS SIGNED) AS sort,
            created_at,
            updated_at
         FROM v2_server
         WHERE id = ?
         LIMIT 1"
    )
    .bind(server_id)
    .fetch_optional(&state.db)
    .await
}

async fn load_server_group_name_map(
    state: &AppState,
) -> Result<HashMap<i64, String>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, String)>(
        "SELECT id, name FROM v2_server_group"
    )
    .fetch_all(&state.db)
    .await?;
    Ok(rows.into_iter().collect())
}

async fn load_server_route_name_map(
    state: &AppState,
) -> Result<HashMap<i64, String>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, String)>(
        "SELECT id, remarks FROM v2_server_route"
    )
    .fetch_all(&state.db)
    .await?;
    Ok(rows.into_iter().collect())
}

fn serialize_legacy_server(
    row: &LegacyServerRow,
    group_names: &HashMap<i64, String>,
    route_names: &HashMap<i64, String>,
    master_server_token: &str,
) -> Value {
    let group_ids = parse_json_i64_array(row.group_ids.as_ref().map(|v| &v.0));
    let route_ids = parse_json_i64_array(row.route_ids.as_ref().map(|v| &v.0));
    let protocol_settings = row
        .protocol_settings
        .as_ref()
        .map(|value| value.0.clone())
        .unwrap_or(Value::Object(Map::new()));
    let rate_time_ranges = row
        .rate_time_ranges
        .as_ref()
        .map(|value| value.0.clone())
        .unwrap_or(Value::Array(Vec::new()));
    let tags = row
        .tags
        .as_ref()
        .map(|value| value.0.clone())
        .unwrap_or(Value::Array(Vec::new()));
    let groups = group_ids
        .iter()
        .filter_map(|id| group_names.get(id).map(|name| json!({"id": id, "name": name})))
        .collect::<Vec<_>>();
    let routes = route_ids
        .iter()
        .filter_map(|id| route_names.get(id).map(|name| json!({"id": id, "remarks": name})))
        .collect::<Vec<_>>();
    let updated_at_text = format_optional_naive_datetime(row.updated_at).unwrap_or_default();
    let created_at_text = format_optional_naive_datetime(row.created_at);

    json!({
        "id": row.id,
        "type": row.server_type,
        "code": row.code,
        "parent_id": row.parent_id,
        "parent": Value::Null,
        "group_ids": group_ids,
        "route_ids": route_ids,
        "groups": groups,
        "routes": routes,
        "name": row.name,
        "rate": row.rate.parse::<f64>().unwrap_or(1.0),
        "rate_time_enable": row.rate_time_enable,
        "rate_time_ranges": rate_time_ranges,
        "tags": tags,
        "host": row.host,
        "port": row.port,
        "server_port": row.server_port,
        "node_token": crate::legacy_server_v1::derive_legacy_server_token(
            master_server_token,
            &row.server_type,
            row.id,
        ),
        "protocol_settings": protocol_settings,
        "show": row.show,
        "sort": row.sort,
        "online": 0,
        "is_online": false,
        "available_status": 0,
        "cache_key": format!("{}-{}-{}-{}", row.server_type, row.id, updated_at_text, 0),
        "load_status": Value::Null,
        "last_check_at": format_optional_naive_datetime(row.updated_at),
        "last_push_at": format_optional_naive_datetime(row.updated_at),
        "created_at": created_at_text,
        "updated_at": format_optional_naive_datetime(row.updated_at),
    })
}

fn parse_json_i64_array(value: Option<&Value>) -> Vec<i64> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(parse_i64_value)
            .filter(|item| *item > 0)
            .collect(),
        _ => Vec::new(),
    }
}
