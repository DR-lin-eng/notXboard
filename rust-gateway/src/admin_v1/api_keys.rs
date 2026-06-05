use crate::*;

pub async fn show(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    if let Err(response) = authenticate_super_admin_user(&state, &headers).await {
        return response;
    }
    user_v1::account::api_key_show(State(state), headers, uri).await
}

pub async fn generate(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    if let Err(response) = authenticate_super_admin_user(&state, &headers).await {
        return response;
    }
    user_v1::account::api_key_generate(State(state), headers, uri, body).await
}

pub async fn reset(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    if let Err(response) = authenticate_super_admin_user(&state, &headers).await {
        return response;
    }
    user_v1::account::api_key_reset(State(state), headers, uri, body).await
}

pub async fn validate(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    if let Err(response) = authenticate_super_admin_user(&state, &headers).await {
        return response;
    }
    user_v1::account::api_key_validate(State(state), headers, uri, body).await
}

pub async fn stats(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_stats_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn search(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_search_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn batch_generate(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_batch_generate_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn cleanup(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_cleanup_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn user_show(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(user_id): axum::extract::Path<i64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_user_show_response(&state, user_id, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn user_generate(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(user_id): axum::extract::Path<i64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_user_generate_response(&state, user_id, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn user_reset(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(user_id): axum::extract::Path<i64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_user_reset_response(&state, user_id, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_stats_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_user")
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    let users_with_api_keys: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM v2_user WHERE api_key IS NOT NULL AND api_key <> ''"
    )
    .fetch_one(&state.db)
    .await
    .map_err(internal_error)?;
    let users_without_api_keys = total_users - users_with_api_keys;
    let coverage = if total_users > 0 {
        ((users_with_api_keys as f64 / total_users as f64) * 100.0 * 100.0).round() / 100.0
    } else {
        0.0
    };

    Ok(json_value_response(json!({
        "success": true,
        "data": {
            "total_users": total_users,
            "users_with_api_keys": users_with_api_keys,
            "users_without_api_keys": users_without_api_keys,
            "api_key_coverage_percentage": coverage,
        }
    })))
}

async fn build_search_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let query = params
        .get("query")
        .map(|value| value.trim().to_string())
        .filter(|value| value.len() >= 3)
        .ok_or_else(|| json_status_response(StatusCode::UNPROCESSABLE_ENTITY, json!({
            "success": false,
            "error": "Validation failed"
        })))?;
    let per_page = params
        .get("per_page")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(20)
        .clamp(1, 100);

    let pattern = format!("%{}%", query);
    let rows = sqlx::query(
        "SELECT id, email, linux_do_username, api_key, trust_level, is_super_admin, created_at
         FROM v2_user
         WHERE email LIKE ?
            OR linux_do_username LIKE ?
            OR api_key LIKE ?
         ORDER BY id DESC
         LIMIT ?"
    )
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .bind(per_page)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    let data = rows
        .iter()
        .map(|row| {
            let api_key = row.try_get::<Option<String>, _>("api_key").ok().flatten();
            json!({
                "id": row.try_get::<i64, _>("id").unwrap_or_default(),
                "email": row.try_get::<String, _>("email").unwrap_or_default(),
                "linux_do_username": row.try_get::<Option<String>, _>("linux_do_username").ok().flatten(),
                "api_key": api_key,
                "api_key_prefix": api_key.as_ref().map(|value| format!("{}...", &value[..std::cmp::min(10, value.len())])),
                "has_api_key": api_key.as_ref().map(|value| !value.trim().is_empty()).unwrap_or(false),
                "trust_level": row.try_get::<Option<i64>, _>("trust_level").ok().flatten(),
                "is_super_admin": row.try_get::<Option<i8>, _>("is_super_admin").ok().flatten().unwrap_or(0) != 0,
                "created_at": row.try_get::<Option<i64>, _>("created_at").ok().flatten(),
            })
        })
        .collect::<Vec<_>>();

    Ok(json_value_response(json!({
        "success": true,
        "data": {
            "data": data,
            "per_page": per_page,
        }
    })))
}

async fn build_batch_generate_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    _body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let users = sqlx::query(
        "SELECT id, email
         FROM v2_user
         WHERE api_key IS NULL
         ORDER BY id ASC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    let user_rows = users
        .iter()
        .map(|row| {
            (
                row.try_get::<i64, _>("id").unwrap_or_default(),
                row.try_get::<String, _>("email").unwrap_or_default(),
            )
        })
        .collect::<Vec<_>>();
    let user_ids = user_rows.iter().map(|(user_id, _)| *user_id).collect::<Vec<_>>();
    let assignments = fill_missing_api_keys_for_users(state, &user_ids)
        .await
        .map_err(internal_error)?
        .into_iter()
        .map(|assignment| (assignment.user_id, assignment.api_key))
        .collect::<std::collections::HashMap<_, _>>();

    let mut results = Vec::with_capacity(user_rows.len());
    for (user_id, email) in user_rows {
        if let Some(api_key) = assignments.get(&user_id) {
            results.push(json!({
                "user_id": user_id,
                "email": email,
                "api_key_generated": true,
                "api_key_prefix": api_key_prefix(api_key),
            }));
        } else {
            results.push(json!({
                "user_id": user_id,
                "email": email,
                "api_key_generated": false,
                "error": "API key assignment missing after batch update",
            }));
        }
    }
    let successful = assignments.len();
    let failed = results.len().saturating_sub(successful);

    Ok(json_value_response(json!({
        "success": true,
        "message": "Batch API key generation completed",
        "data": {
            "results": results,
            "summary": {
                "total_processed": successful + failed,
                "successful": successful,
                "failed": failed,
            }
        }
    })))
}

async fn build_cleanup_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    _body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let users = sqlx::query(
        "SELECT id, email, api_key
         FROM v2_user
         WHERE api_key IS NOT NULL AND api_key <> ''
         ORDER BY id ASC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    let mut invalid_format_count = 0usize;
    let mut duplicate_count = 0usize;
    let mut pending_users = Vec::new();
    let mut seen_keys = std::collections::HashSet::new();

    for row in users {
        let user_id = row.try_get::<i64, _>("id").unwrap_or_default();
        let email = row.try_get::<String, _>("email").unwrap_or_default();
        let api_key = row.try_get::<Option<String>, _>("api_key").ok().flatten().unwrap_or_default();

        let invalid_format = !api_key_has_valid_format(&api_key);
        let duplicate = !invalid_format && !seen_keys.insert(api_key.clone());

        if !invalid_format && !duplicate {
            continue;
        }

        let reason = if invalid_format {
            invalid_format_count += 1;
            "invalid_format"
        } else {
            duplicate_count += 1;
            "duplicate"
        };

        pending_users.push((user_id, email, api_key, reason));
    }

    let pending_ids = pending_users.iter().map(|(user_id, _, _, _)| *user_id).collect::<Vec<_>>();
    let assignments = replace_api_keys_for_users(state, &pending_ids)
        .await
        .map_err(internal_error)?
        .into_iter()
        .map(|assignment| (assignment.user_id, assignment.api_key))
        .collect::<std::collections::HashMap<_, _>>();
    let mut cleaned_users = Vec::with_capacity(pending_users.len());
    for (user_id, email, old_api_key, reason) in pending_users {
        let new_api_key = assignments
            .get(&user_id)
            .ok_or_else(|| json_error(StatusCode::INTERNAL_SERVER_ERROR, "API key replacement missing after batch update"))?;
        cleaned_users.push(json!({
            "user_id": user_id,
            "email": email,
            "reason": reason,
            "old_key_prefix": api_key_prefix(&old_api_key),
            "new_key_prefix": api_key_prefix(new_api_key),
        }));
    }

    Ok(json_value_response(json!({
        "success": true,
        "message": "API key cleanup completed",
        "data": {
            "invalid_format_count": invalid_format_count,
            "duplicate_count": duplicate_count,
            "cleaned_users": cleaned_users,
        }
    })))
}

async fn build_user_show_response(
    state: &AppState,
    target_user_id: i64,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let user = load_bearer_user_by_id(state, target_user_id)
        .await
        .map_err(internal_error)?;
    let Some(user) = user else {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({
            "success": false,
            "error": "User not found"
        })));
    };

    Ok(json_value_response(json!({
        "success": true,
        "data": {
            "user_id": user.id,
            "email": user.email,
            "linux_do_username": user.linux_do_username,
            "api_key": user.api_key,
            "api_key_prefix": user.api_key.as_ref().map(|value| api_key_prefix(value)),
            "has_api_key": user.api_key.as_ref().map(|value| !value.trim().is_empty()).unwrap_or(false),
            "created_at": user.created_at,
            "last_login_at": user.last_login_at,
            "is_active": user.banned == 0,
        }
    })))
}

async fn build_user_generate_response(
    state: &AppState,
    target_user_id: i64,
    headers: HeaderMap,
    _uri: Uri,
    _body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let admin = authenticate_super_admin_user(state, &headers).await?;
    let user = load_bearer_user_by_id(state, target_user_id)
        .await
        .map_err(internal_error)?;
    let Some(user) = user else {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({
            "success": false,
            "error": "User not found"
        })));
    };
    let api_key = reset_user_api_key(state, target_user_id).await.map_err(internal_error)?;
    let _ = admin;
    Ok(json_value_response(json!({
        "success": true,
        "message": "API key generated successfully",
        "data": {
            "user_id": user.id,
            "email": user.email,
            "api_key": api_key,
            "api_key_prefix": api_key_prefix(&api_key),
            "generated_at": Utc::now().to_rfc3339(),
        }
    })))
}

async fn build_user_reset_response(
    state: &AppState,
    target_user_id: i64,
    headers: HeaderMap,
    _uri: Uri,
    _body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let admin = authenticate_super_admin_user(state, &headers).await?;
    let user = load_bearer_user_by_id(state, target_user_id)
        .await
        .map_err(internal_error)?;
    let Some(user) = user else {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({
            "success": false,
            "error": "User not found"
        })));
    };

    let old_key_prefix = user
        .api_key
        .as_ref()
        .filter(|value| !value.trim().is_empty())
        .map(|value| api_key_prefix(value));

    let new_api_key = reset_user_api_key(state, target_user_id).await.map_err(internal_error)?;

    let _ = admin;
    Ok(json_value_response(json!({
        "success": true,
        "message": if old_key_prefix.is_some() { "API key reset successfully" } else { "User had no API key. Generated a new one." },
        "data": {
            "user_id": user.id,
            "email": user.email,
            "api_key": new_api_key,
            "api_key_prefix": api_key_prefix(&new_api_key),
            "old_key_prefix": old_key_prefix,
            "action": if user.api_key.as_ref().filter(|value| !value.trim().is_empty()).is_some() { "reset" } else { "generated" },
            "reset_at": Utc::now().to_rfc3339(),
        }
    })))
}
