use crate::*;

pub async fn users_traffic(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_users_traffic_response(&state, id, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn blacklist_user(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_blacklist_user_response(&state, id, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn unblacklist_user(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<u64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_unblacklist_user_response(&state, id, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_users_traffic_response(
    state: &AppState,
    node_id: u64,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let node = load_owned_node_admin_node(state, node_id, user.id).await.map_err(internal_error)?;
    let Some(node) = node else {
        return Ok(json_value_response(json!({
            "success": false,
            "error": "Server node not found"
        })));
    };

    let params = parse_query(&uri);
    let days = params
        .get("days")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(30)
        .max(1);
    let start = chrono::Local::now().date_naive() - chrono::Days::new((days - 1) as u64);
    let end = chrono::Local::now().date_naive();

    let rows = load_node_users_traffic_rows(state, node.id, &start.to_string(), &end.to_string())
        .await
        .map_err(internal_error)?;
    let user_ids = rows.iter().map(|row| row.user_id).collect::<Vec<_>>();
    let users = load_node_traffic_user_profiles(state, &user_ids).await.map_err(internal_error)?;
    let user_map = users.into_iter().map(|row| (row.id, row)).collect::<HashMap<_, _>>();
    let blacklisted = load_node_blacklisted_user_ids(state, node.id).await.map_err(internal_error)?;
    let blacklisted_set = blacklisted.into_iter().collect::<HashSet<_>>();

    let data = rows
        .into_iter()
        .map(|row| {
            let profile = user_map.get(&row.user_id);
            let upload = row.upload.max(0);
            let download = row.download.max(0);
            json!({
                "user_id": row.user_id,
                "email": profile.map(|item| item.email.clone()),
                "name": profile.and_then(|item| item.linux_do_name.clone()),
                "trust_level": profile.map(|item| item.trust_level).unwrap_or(0),
                "is_silenced": profile.map(|item| item.is_silenced != 0).unwrap_or(false),
                "banned": profile.map(|item| item.banned != 0).unwrap_or(false),
                "upload": upload,
                "download": download,
                "total": upload + download,
                "is_blacklisted": blacklisted_set.contains(&row.user_id),
            })
        })
        .collect::<Vec<_>>();

    Ok(json_value_response(json!({
        "success": true,
        "data": {
            "node": {
                "id": node.id,
                "name": node.name,
            },
            "days": days,
            "users": data,
        }
    })))
}

async fn build_blacklist_user_response(
    state: &AppState,
    node_id: u64,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let node = load_owned_node_admin_node(state, node_id, user.id).await.map_err(internal_error)?;
    let Some(node) = node else {
        return Ok(json_value_response(json!({
            "success": false,
            "error": "Server node not found"
        })));
    };

    let payload = parse_json_body(body).await?;
    let target_user_id = payload
        .get("user_id")
        .and_then(parse_i64_value)
        .filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "The user_id field is required."))?;
    let reason = payload
        .get("reason")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_user WHERE id = ?")
        .bind(target_user_id)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    if exists <= 0 {
        return Ok(json_value_response(json!({
            "success": false,
            "error": "User not found"
        })));
    }

    sqlx::query(
        "INSERT INTO user_node_blacklist (node_id, user_id, reason, created_at)
         VALUES (?, ?, ?, NOW())
         ON DUPLICATE KEY UPDATE reason = VALUES(reason)"
    )
    .bind(node.id)
    .bind(target_user_id)
    .bind(reason)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;
    clear_accessible_user_ids_cache(state, node.id);

    Ok(json_value_response(json!({ "success": true })))
}

async fn build_unblacklist_user_response(
    state: &AppState,
    node_id: u64,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let node = load_owned_node_admin_node(state, node_id, user.id).await.map_err(internal_error)?;
    let Some(node) = node else {
        return Ok(json_value_response(json!({
            "success": false,
            "error": "Server node not found"
        })));
    };

    let payload = parse_json_body(body).await?;
    let target_user_id = payload
        .get("user_id")
        .and_then(parse_i64_value)
        .filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "The user_id field is required."))?;

    sqlx::query("DELETE FROM user_node_blacklist WHERE node_id = ? AND user_id = ?")
        .bind(node.id)
    .bind(target_user_id)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;
    clear_accessible_user_ids_cache(state, node.id);

    Ok(json_value_response(json!({ "success": true })))
}
