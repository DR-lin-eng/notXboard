use super::super::*;

#[derive(Clone, sqlx::FromRow)]
struct ServerGroupRow {
    id: i64,
    name: String,
    created_at: i64,
    updated_at: i64,
    users_count: i64,
    server_count: i64,
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
    let rows = sqlx::query_as::<_, ServerGroupRow>(
        "SELECT
            g.id,
            g.name,
            g.created_at,
            g.updated_at,
            (SELECT COUNT(*) FROM v2_user u WHERE u.group_id = g.id) AS users_count,
            (
                SELECT COUNT(*)
                FROM v2_server s
                WHERE JSON_CONTAINS(COALESCE(s.group_ids, JSON_ARRAY()), JSON_QUOTE(CAST(g.id AS CHAR)))
                   OR JSON_CONTAINS(COALESCE(s.group_ids, JSON_ARRAY()), CAST(g.id AS JSON))
            ) AS server_count
         FROM v2_server_group
         AS g
         ORDER BY id DESC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(json_value_response(success_response_payload(Value::Array(
        rows.into_iter()
            .map(|row| json!({
                "id": row.id,
                "name": row.name,
                "users_count": row.users_count,
                "server_count": row.server_count,
                "created_at": row.created_at,
                "updated_at": row.updated_at,
            }))
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

    let name = obj
        .get("name")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "组名不能为空"))?;
    let now = Utc::now().timestamp();

    if let Some(id) = obj.get("id").and_then(parse_i64_value).filter(|value| *value > 0) {
        let updated = sqlx::query(
            "UPDATE v2_server_group
             SET name = ?, updated_at = ?
             WHERE id = ?"
        )
        .bind(&name)
        .bind(now)
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
        if updated.rows_affected() == 0 {
            return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"组不存在"})));
        }
        return Ok(json_value_response(success_response_payload(Value::Bool(true))));
    }

    sqlx::query(
        "INSERT INTO v2_server_group (name, created_at, updated_at)
         VALUES (?, ?, ?)"
    )
    .bind(&name)
    .bind(now)
    .bind(now)
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
    let group_id = payload
        .get("id")
        .and_then(parse_i64_value)
        .filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "组ID不能为空"))?;

    let group_exists: Option<i64> = sqlx::query_scalar(
        "SELECT id
         FROM v2_server_group
         WHERE id = ?
         LIMIT 1"
    )
    .bind(group_id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?;
    if group_exists.is_none() {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"组不存在"})));
    }

    let server_in_use: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM v2_server
         WHERE JSON_CONTAINS(COALESCE(group_ids, JSON_ARRAY()), JSON_QUOTE(CAST(? AS CHAR)))
            OR JSON_CONTAINS(COALESCE(group_ids, JSON_ARRAY()), CAST(? AS JSON))"
    )
    .bind(group_id)
    .bind(group_id)
    .fetch_one(&state.db)
    .await
    .map_err(internal_error)?;
    if server_in_use > 0 {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"该组已被节点所使用，无法删除"})));
    }

    let plan_in_use: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_plan WHERE group_id = ?")
        .bind(group_id)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    if plan_in_use > 0 {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"该组已被订阅所使用，无法删除"})));
    }

    let user_in_use: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_user WHERE group_id = ?")
        .bind(group_id)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    if user_in_use > 0 {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"该组已被用户所使用，无法删除"})));
    }

    let deleted = sqlx::query("DELETE FROM v2_server_group WHERE id = ?")
        .bind(group_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Bool(
        deleted.rows_affected() > 0,
    ))))
}
