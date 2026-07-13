use crate::*;
use sqlx::{MySql, QueryBuilder};

#[derive(Clone)]
struct BatchUserLimitUpdateItem {
    user_id: i64,
    speed_limit_up: i64,
    speed_limit_down: i64,
    device_limit: i64,
    connection_limit: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct BatchUserSummaryRow {
    id: i64,
    email: String,
}

pub async fn index(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_index_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn user_limits_show(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(user_id): axum::extract::Path<i64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_user_limits_show_response(&state, user_id, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn self_limits_show(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_self_limits_show_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn user_limits_store(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(user_id): axum::extract::Path<i64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_user_limits_store_response(&state, user_id, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn user_limits_destroy(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(user_id): axum::extract::Path<i64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_user_limits_destroy_response(&state, user_id, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn concurrent_ip_limit_show(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(user_id): axum::extract::Path<i64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_concurrent_ip_limit_show_response(&state, user_id, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn concurrent_ip_limit_update(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(user_id): axum::extract::Path<i64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_concurrent_ip_limit_update_response(&state, user_id, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn batch_update(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_batch_update_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_index_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let per_page = params
        .get("per_page")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(20)
        .clamp(1, 100);
    let search = params.get("search").map(|value| value.trim()).filter(|value| !value.is_empty());

    let page = params
        .get("page")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(1)
        .max(1);
    let offset = (page - 1) * per_page;

    let (count_sql, data_sql, bind_search) = if search.is_some() {
        (
            "SELECT COUNT(*) FROM v2_user WHERE email LIKE ? OR linux_do_username LIKE ?",
            "SELECT id, token, group_id, subscribe_key, subscribe_salt, uuid, u, d, transfer_enable, expired_at, trust_level, banned, is_super_admin, is_silenced, subscription_credential_version
             FROM v2_user
             WHERE email LIKE ? OR linux_do_username LIKE ?
             ORDER BY id DESC
             LIMIT ? OFFSET ?",
            true,
        )
    } else {
        (
            "SELECT COUNT(*) FROM v2_user",
            "SELECT id, token, group_id, subscribe_key, subscribe_salt, uuid, u, d, transfer_enable, expired_at, trust_level, banned, is_super_admin, is_silenced, subscription_credential_version
             FROM v2_user
             ORDER BY id DESC
             LIMIT ? OFFSET ?",
            false,
        )
    };

    let total = if let Some(search_value) = search {
        let pattern = format!("%{}%", search_value);
        let mut q = sqlx::query_scalar::<_, i64>(count_sql);
        q = q.bind(&pattern).bind(&pattern);
        q.fetch_one(&state.db).await.map_err(internal_error)?
    } else {
        sqlx::query_scalar::<_, i64>(count_sql)
            .fetch_one(&state.db)
            .await
            .map_err(internal_error)?
    };

    let users = if bind_search {
        let pattern = format!("%{}%", search.unwrap_or_default());
        sqlx::query_as::<_, UserRow>(data_sql)
            .bind(&pattern)
            .bind(&pattern)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&state.db)
            .await
            .map_err(internal_error)?
    } else {
        sqlx::query_as::<_, UserRow>(data_sql)
            .bind(per_page)
            .bind(offset)
            .fetch_all(&state.db)
            .await
            .map_err(internal_error)?
    };

    let user_ids = users.iter().map(|user| user.id).collect::<Vec<_>>();
    let mut detail_map = HashMap::new();
    if !user_ids.is_empty() {
        let placeholders = vec!["?"; user_ids.len()].join(",");
        let sql = format!(
            "SELECT id, email, linux_do_username, trust_level, is_super_admin, created_at
             FROM v2_user
             WHERE id IN ({})",
            placeholders
        );
        let mut query = sqlx::query(&sql);
        for user_id in &user_ids {
            query = query.bind(*user_id);
        }
        let rows = query.fetch_all(&state.db).await.map_err(internal_error)?;
        for row in rows {
            detail_map.insert(
                row.try_get::<i64, _>("id").unwrap_or_default(),
                json!({
                    "email": row.try_get::<String, _>("email").unwrap_or_default(),
                    "linux_do_username": row.try_get::<Option<String>, _>("linux_do_username").ok().flatten(),
                    "trust_level": row.try_get::<Option<i64>, _>("trust_level").ok().flatten().unwrap_or(0),
                    "is_super_admin": row.try_get::<Option<i8>, _>("is_super_admin").ok().flatten().unwrap_or(0) != 0,
                    "created_at": row.try_get::<Option<i64>, _>("created_at").ok().flatten(),
                }),
            );
        }
    }

    let limits = load_effective_limits(state, &users).await.map_err(internal_error)?;
    let individual_limit_map = load_user_individual_limit_map(state, &user_ids).await.map_err(internal_error)?;

    let data = users
        .into_iter()
        .map(|user| {
            let extra = detail_map.get(&user.id).cloned().unwrap_or_else(|| json!({}));
            let effective = limits.get(&user.id).cloned().unwrap_or(EffectiveLimits {
                speed_limit_down: 0,
                device_limit: 2,
                connection_limit: 10,
            });
            let object = extra.as_object().cloned().unwrap_or_default();
            json!({
                "id": user.id,
                "email": object.get("email").cloned().unwrap_or(Value::Null),
                "linux_do_username": object.get("linux_do_username").cloned().unwrap_or(Value::Null),
                "trust_level": object.get("trust_level").cloned().unwrap_or(Value::from(0)),
                "is_super_admin": object.get("is_super_admin").cloned().unwrap_or(Value::Bool(false)),
                "has_individual_limits": individual_limit_map.contains_key(&user.id),
                "effective_limits": {
                    "speed_limit_down": effective.speed_limit_down,
                    "device_limit": effective.device_limit,
                    "connection_limit": effective.connection_limit,
                },
                "created_at": object.get("created_at").cloned().unwrap_or(Value::Null),
            })
        })
        .collect::<Vec<_>>();

    Ok(json_value_response(json!({
        "success": true,
        "data": {
            "current_page": page,
            "data": data,
            "per_page": per_page,
            "total": total,
        }
    })))
}

async fn build_user_limits_show_response(
    state: &AppState,
    user_id: i64,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    build_owned_user_limits_show_response(state, user_id, uri).await
}

async fn build_owned_user_limits_show_response(
    state: &AppState,
    user_id: i64,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = load_bearer_user_by_id(state, user_id).await.map_err(internal_error)?;
    let Some(user) = user else {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({
            "success": false,
            "error": "User not found"
        })));
    };

    let extra = sqlx::query("SELECT email, trust_level FROM v2_user WHERE id = ? LIMIT 1")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?;

    let individual = load_user_individual_limit_map(state, &[user_id]).await.map_err(internal_error)?;
    let effective = load_effective_limits_by_profiles(
        state,
        &[UserLimitProfile {
            user_id,
            trust_level: user.trust_level,
        }],
    )
    .await
    .map_err(internal_error)?;

    let row = extra.unwrap();
    let limit = individual.get(&user_id);
    let effective_limit = effective.get(&user_id).cloned().unwrap_or(EffectiveLimits {
        speed_limit_down: 0,
        device_limit: 2,
        connection_limit: 10,
    });

    Ok(json_value_response(json!({
        "success": true,
        "data": {
            "user_id": user_id,
            "user_email": row.try_get::<String, _>("email").unwrap_or_default(),
            "trust_level": row.try_get::<Option<i64>, _>("trust_level").ok().flatten().unwrap_or(0),
            "individual_limits": limit.map(|value| json!({
                "speed_limit_up": value.speed_limit_up,
                "speed_limit_down": value.speed_limit_down,
                "device_limit": value.device_limit,
                "connection_limit": value.connection_limit,
                "created_at": value.created_at.map(|v| v.timestamp()),
                "updated_at": value.updated_at.map(|v| v.timestamp()),
            })),
            "effective_limits": {
                "speed_limit_down": effective_limit.speed_limit_down,
                "device_limit": effective_limit.device_limit,
                "connection_limit": effective_limit.connection_limit,
            },
            "has_individual_limits": limit.is_some(),
        }
    })))
}

async fn build_self_limits_show_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    build_owned_user_limits_show_response(state, user.id, uri).await
}

async fn build_user_limits_store_response(
    state: &AppState,
    user_id: i64,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let user = load_bearer_user_by_id(state, user_id).await.map_err(internal_error)?;
    let Some(user) = user else {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({
            "success": false,
            "error": "User not found"
        })));
    };

    let payload = parse_json_body(body).await?;
    let obj = payload.as_object().ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    let speed_limit_up = obj.get("speed_limit_up").and_then(parse_i64_value).unwrap_or(0);
    let speed_limit_down = obj.get("speed_limit_down").and_then(parse_i64_value).unwrap_or(0);
    let device_limit = obj.get("device_limit").and_then(parse_i64_value).unwrap_or(0);
    let connection_limit = obj.get("connection_limit").and_then(parse_i64_value).unwrap_or(0);
    validate_admin_group_limit_values(0, speed_limit_up, speed_limit_down, device_limit, connection_limit)?;

    let now = Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO user_individual_limits (user_id, speed_limit_up, speed_limit_down, device_limit, connection_limit, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, FROM_UNIXTIME(?), FROM_UNIXTIME(?))
         ON DUPLICATE KEY UPDATE
           speed_limit_up = VALUES(speed_limit_up),
           speed_limit_down = VALUES(speed_limit_down),
           device_limit = VALUES(device_limit),
           connection_limit = VALUES(connection_limit),
           updated_at = VALUES(updated_at)"
    )
    .bind(user_id)
    .bind(speed_limit_up)
    .bind(speed_limit_down)
    .bind(device_limit)
    .bind(connection_limit)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;
    clear_all_authorization_caches(state);

    let effective = load_effective_limits_by_profiles(
        state,
        &[UserLimitProfile {
            user_id,
            trust_level: user.trust_level,
        }],
    )
    .await
    .map_err(internal_error)?;
    let effective_limit = effective.get(&user_id).cloned().unwrap_or(EffectiveLimits {
        speed_limit_down: 0,
        device_limit: 2,
        connection_limit: 10,
    });

    Ok(json_value_response(json!({
        "success": true,
        "message": "User limits updated successfully",
        "data": {
            "user_id": user_id,
            "individual_limits": {
                "speed_limit_up": speed_limit_up,
                "speed_limit_down": speed_limit_down,
                "device_limit": device_limit,
                "connection_limit": connection_limit,
                "updated_at": now,
            },
            "effective_limits": {
                "speed_limit_down": effective_limit.speed_limit_down,
                "device_limit": effective_limit.device_limit,
                "connection_limit": effective_limit.connection_limit,
            }
        }
    })))
}

async fn build_user_limits_destroy_response(
    state: &AppState,
    user_id: i64,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let user = load_bearer_user_by_id(state, user_id).await.map_err(internal_error)?;
    let Some(user) = user else {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({
            "success": false,
            "error": "User not found"
        })));
    };

    let deleted = sqlx::query("DELETE FROM user_individual_limits WHERE user_id = ?")
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    if deleted.rows_affected() == 0 {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({
            "success": false,
            "error": "No individual limits found for this user"
        })));
    }
    clear_all_authorization_caches(state);

    let effective = load_effective_limits_by_profiles(
        state,
        &[UserLimitProfile {
            user_id,
            trust_level: user.trust_level,
        }],
    )
    .await
    .map_err(internal_error)?;
    let effective_limit = effective.get(&user_id).cloned().unwrap_or(EffectiveLimits {
        speed_limit_down: 0,
        device_limit: 2,
        connection_limit: 10,
    });

    Ok(json_value_response(json!({
        "success": true,
        "message": "Individual limits removed successfully. User will now use group limits.",
        "data": {
            "user_id": user_id,
            "effective_limits": {
                "speed_limit_down": effective_limit.speed_limit_down,
                "device_limit": effective_limit.device_limit,
                "connection_limit": effective_limit.connection_limit,
            }
        }
    })))
}

async fn build_concurrent_ip_limit_show_response(
    state: &AppState,
    user_id: i64,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let row = sqlx::query("SELECT id, concurrent_ip_limit FROM v2_user WHERE id = ? LIMIT 1")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?;
    let Some(row) = row else {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({
            "success": false,
            "error": "User not found"
        })));
    };

    Ok(json_value_response(json!({
        "success": true,
        "data": {
            "user_id": row.try_get::<i64, _>("id").unwrap_or_default(),
            "concurrent_ip_limit": row.try_get::<i64, _>("concurrent_ip_limit").unwrap_or(0),
        }
    })))
}

async fn build_concurrent_ip_limit_update_response(
    state: &AppState,
    user_id: i64,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let obj = payload.as_object().ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    let concurrent_ip_limit = obj
        .get("concurrent_ip_limit")
        .and_then(parse_i64_value)
        .filter(|value| (0..=1000).contains(value))
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let updated = sqlx::query("UPDATE v2_user SET concurrent_ip_limit = ?, updated_at = ? WHERE id = ?")
        .bind(concurrent_ip_limit)
        .bind(Utc::now().timestamp())
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    if updated.rows_affected() == 0 {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({
            "success": false,
            "error": "User not found"
        })));
    }
    clear_all_authorization_caches(state);

    Ok(json_value_response(json!({
        "success": true,
        "message": "Concurrent IP limit updated successfully",
        "data": {
            "user_id": user_id,
            "concurrent_ip_limit": concurrent_ip_limit,
        }
    })))
}

async fn build_batch_update_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let obj = payload.as_object().ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    let users = obj.get("users").and_then(|value| value.as_array()).ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let now = Utc::now().timestamp();
    let mut updates = Vec::with_capacity(users.len());
    for item in users {
        let row = item.as_object().ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
        let user_id = row.get("user_id").and_then(parse_i64_value).ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
        let speed_limit_up = row.get("speed_limit_up").and_then(parse_i64_value).unwrap_or(0);
        let speed_limit_down = row.get("speed_limit_down").and_then(parse_i64_value).unwrap_or(0);
        let device_limit = row.get("device_limit").and_then(parse_i64_value).unwrap_or(0);
        let connection_limit = row.get("connection_limit").and_then(parse_i64_value).unwrap_or(0);
        validate_admin_group_limit_values(0, speed_limit_up, speed_limit_down, device_limit, connection_limit)?;
        updates.push(BatchUserLimitUpdateItem {
            user_id,
            speed_limit_up,
            speed_limit_down,
            device_limit,
            connection_limit,
        });
    }

    let user_ids = updates.iter().map(|item| item.user_id).collect::<Vec<_>>();
    let existing_users = load_batch_user_summary_rows(state, &user_ids)
        .await
        .map_err(internal_error)?;
    if existing_users.len() != user_ids.len() {
        return Ok(json_status_response(StatusCode::UNPROCESSABLE_ENTITY, json!({
            "success": false,
            "error": "Validation failed"
        })));
    }

    batch_upsert_user_individual_limits(&updates, now)
        .build()
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    clear_all_authorization_caches(state);

    let results = existing_users
        .into_iter()
        .map(|user| json!({
            "user_id": user.id,
            "email": user.email,
            "limits_updated": true,
        }))
        .collect::<Vec<_>>();

    Ok(json_value_response(json!({
        "success": true,
        "message": "User limits updated successfully",
        "data": results
    })))
}

async fn load_batch_user_summary_rows(
    state: &AppState,
    user_ids: &[i64],
) -> Result<Vec<BatchUserSummaryRow>, sqlx::Error> {
    if user_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut builder = QueryBuilder::<MySql>::new(
        "SELECT id, email
         FROM v2_user
         WHERE id IN (",
    );
    {
        let mut separated = builder.separated(", ");
        for user_id in user_ids {
            separated.push_bind(user_id);
        }
    }
    builder.push(") ORDER BY id");
    builder.build_query_as::<BatchUserSummaryRow>().fetch_all(&state.db).await
}

fn batch_upsert_user_individual_limits(
    updates: &[BatchUserLimitUpdateItem],
    now: i64,
) -> QueryBuilder<'static, MySql> {
    let mut builder = QueryBuilder::<MySql>::new(
        "INSERT INTO user_individual_limits (user_id, speed_limit_up, speed_limit_down, device_limit, connection_limit, created_at, updated_at) ",
    );
    builder.push_values(updates, |mut row, item| {
        row.push_bind(item.user_id)
            .push_bind(item.speed_limit_up)
            .push_bind(item.speed_limit_down)
            .push_bind(item.device_limit)
            .push_bind(item.connection_limit)
            .push("FROM_UNIXTIME(")
            .push_bind(now)
            .push(")")
            .push("FROM_UNIXTIME(")
            .push_bind(now)
            .push(")");
    });
    builder.push(
        " ON DUPLICATE KEY UPDATE
            speed_limit_up = VALUES(speed_limit_up),
            speed_limit_down = VALUES(speed_limit_down),
            device_limit = VALUES(device_limit),
            connection_limit = VALUES(connection_limit),
            updated_at = VALUES(updated_at)",
    );
    builder
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_user_limit_mutation_invalidates_authorization_caches() {
        let source = include_str!("users_limits.rs");
        for (start, end) in [
            (
                "async fn build_user_limits_store_response",
                "async fn build_user_limits_destroy_response",
            ),
            (
                "async fn build_user_limits_destroy_response",
                "async fn build_concurrent_ip_limit_show_response",
            ),
            (
                "async fn build_concurrent_ip_limit_update_response",
                "async fn build_batch_update_response",
            ),
            (
                "async fn build_batch_update_response",
                "async fn load_batch_user_summary_rows",
            ),
        ] {
            let section = source
                .split_once(start)
                .and_then(|(_, tail)| tail.split_once(end).map(|(body, _)| body))
                .expect("mutation handler must remain present");
            assert!(
                section.contains("clear_all_authorization_caches(state);"),
                "{start} must invalidate authorization caches after a successful write"
            );
        }
    }
}
