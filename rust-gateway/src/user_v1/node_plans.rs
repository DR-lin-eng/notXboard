use crate::*;

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

pub async fn store(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_store_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_update_response(&state, id, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn destroy(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_destroy_response(&state, id, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_index_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let plans = load_owned_node_plans(state, user.id).await.map_err(internal_error)?;
    let share_base = get_setting_string(state, "app_url", "").await;
    let system_reset_method = get_setting_int(state, "reset_traffic_method", 2).await;
    let data = plans
        .iter()
        .map(|plan| serialize_guest_plan(plan, share_base.trim_end_matches('/'), system_reset_method))
        .collect::<Vec<_>>();
    Ok(json_value_response(success_response_payload(Value::Array(data))))
}

async fn build_store_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let parsed = parse_node_plan_mutation_input(&payload, true)?;

    let Some(name) = parsed.name else {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    };
    let Some(prices) = parsed.prices else {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    };
    let Some(node_ids) = parsed.node_ids else {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    };
    if node_ids.is_empty() {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    }
    let Some(sell) = parsed.sell else {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    };
    let Some(show) = parsed.show else {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    };
    let Some(renew) = parsed.renew else {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    };
    let visibility_scope = parsed.visibility_scope.unwrap_or_else(|| "public".to_string());
    let access_user_ids = if visibility_scope == "assigned_only" {
        let ids = parsed.access_user_ids.unwrap_or_default();
        if ids.is_empty() {
            return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
        }
        ids
    } else {
        Vec::new()
    };

    let placeholders = vec!["?"; node_ids.len()].join(",");
    let sql = format!(
        "SELECT COUNT(*) FROM server_nodes WHERE user_id = ? AND id IN ({})",
        placeholders
    );
    let mut query = sqlx::query_scalar::<_, i64>(&sql).bind(user.id);
    for node_id in &node_ids {
        query = query.bind(*node_id);
    }
    let owned_count = query.fetch_one(&state.db).await.map_err(internal_error)?;
    if owned_count != node_ids.len() as i64 {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "node_ids contains nodes you do not own"));
    }

    let free_quota = if parsed.allow_trial.unwrap_or(true) {
        match parsed.free_quota_gb_by_trust_level {
            Some(Some(value)) => Some(value),
            Some(None) => None,
            None => None,
        }
    } else {
        None
    };
    let share_token = resolve_node_plan_share_token(
        state,
        &visibility_scope,
        parsed.share_token.as_ref().and_then(|value| value.as_deref()),
        None,
    )
    .await
    .map_err(internal_error)?;

    let now = Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO v2_plan (
            scope, owner_user_id, min_trust_level, free_quota_gb_by_trust_level, node_ids, group_id,
            transfer_enable, is_unlimited_traffic, name, speed_limit, `show`, visibility_scope,
            access_user_ids, share_token, sort, renew, content, prices, capacity_limit, sell, device_limit, tags,
            created_at, updated_at
         ) VALUES (?, ?, ?, ?, ?, NULL, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind("node")
    .bind(user.id)
    .bind(parsed.min_trust_level.flatten())
    .bind(free_quota.as_ref().map(|v| v.to_string()))
    .bind(Value::Array(node_ids.into_iter().map(Value::from).collect::<Vec<_>>()).to_string())
    .bind(parsed.transfer_enable.flatten().unwrap_or(0))
    .bind(parsed.is_unlimited_traffic.unwrap_or(false))
    .bind(name)
    .bind(parsed.speed_limit.flatten())
    .bind(if visibility_scope == "public" { show } else { false })
    .bind(&visibility_scope)
    .bind(Value::Array(access_user_ids.into_iter().map(Value::from).collect::<Vec<_>>()).to_string())
    .bind(share_token)
    .bind(parsed.sort.unwrap_or(0))
    .bind(renew)
    .bind(parsed.content.flatten())
    .bind(prices.to_string())
    .bind(parsed.capacity_limit.flatten())
    .bind(sell)
    .bind(parsed.device_limit.flatten())
    .bind("[]")
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    let plan = load_latest_owned_node_plan(state, user.id).await.map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::INTERNAL_SERVER_ERROR, "Subscription plan does not exist"))?;
    let share_base = get_setting_string(state, "app_url", "").await;
    let system_reset_method = get_setting_int(state, "reset_traffic_method", 2).await;
    Ok(json_value_response(success_response_payload(
        serialize_guest_plan(&plan, share_base.trim_end_matches('/'), system_reset_method),
    )))
}

async fn build_update_response(
    state: &AppState,
    plan_id: i64,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let existing = load_owned_node_plan_by_id(state, user.id, plan_id).await.map_err(internal_error)?;
    let Some(existing) = existing else {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Subscription plan does not exist"));
    };

    let payload = parse_json_body(body).await?;
    let parsed = parse_node_plan_mutation_input(&payload, false)?;
    let visibility_scope = parsed
        .visibility_scope
        .unwrap_or_else(|| normalize_visibility_scope(existing.visibility_scope.as_str()).to_string());

    let node_ids = if let Some(ids) = parsed.node_ids {
        if ids.is_empty() {
            return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
        }
        let placeholders = vec!["?"; ids.len()].join(",");
        let sql = format!(
            "SELECT COUNT(*) FROM server_nodes WHERE user_id = ? AND id IN ({})",
            placeholders
        );
        let mut query = sqlx::query_scalar::<_, i64>(&sql).bind(user.id);
        for node_id in &ids {
            query = query.bind(*node_id);
        }
        let owned_count = query.fetch_one(&state.db).await.map_err(internal_error)?;
        if owned_count != ids.len() as i64 {
            return Ok(fail_json_response(StatusCode::BAD_REQUEST, "node_ids contains nodes you do not own"));
        }
        Some(Value::Array(ids.into_iter().map(Value::from).collect::<Vec<_>>()).to_string())
    } else {
        None
    };

    let access_user_ids = if visibility_scope == "assigned_only" {
        let ids = if let Some(ids) = parsed.access_user_ids {
            ids
        } else {
            existing
                .access_user_ids
                .as_ref()
                .and_then(|json| json.0.as_array().cloned())
                .unwrap_or_default()
                .iter()
                .filter_map(parse_i64_value)
                .filter(|v| *v > 0)
                .collect::<Vec<_>>()
        };
        if ids.is_empty() {
            return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
        }
        Some(Value::Array(ids.into_iter().map(Value::from).collect::<Vec<_>>()).to_string())
    } else {
        Some("[]".to_string())
    };

    let free_quota = if matches!(parsed.allow_trial, Some(false)) {
        None
    } else {
        match parsed.free_quota_gb_by_trust_level {
            Some(Some(value)) => Some(value.to_string()),
            Some(None) => None,
            None => existing.free_quota_gb_by_trust_level.as_ref().map(|v| v.0.to_string()),
        }
    };
    let requested_share_token = match &parsed.share_token {
        None => existing.share_token.as_deref(),
        Some(Some(token)) => Some(token.as_str()),
        Some(None) => None,
    };
    let share_token = resolve_node_plan_share_token(
        state,
        &visibility_scope,
        requested_share_token,
        Some(plan_id),
    )
    .await
    .map_err(internal_error)?;

    let show = if visibility_scope == "public" {
        parsed.show.unwrap_or(existing.show)
    } else {
        false
    };
    let content: Option<String> = match parsed.content {
        None => existing.content.clone(),
        Some(Some(value)) => Some(value),
        Some(None) => None,
    };
    let now = Utc::now().timestamp();
    sqlx::query(
        "UPDATE v2_plan
         SET min_trust_level = ?, free_quota_gb_by_trust_level = ?, node_ids = ?, transfer_enable = ?, is_unlimited_traffic = ?,
             name = ?, speed_limit = ?, `show` = ?, visibility_scope = ?, access_user_ids = ?, share_token = ?, sort = ?, renew = ?,
             content = ?, prices = ?, capacity_limit = ?, sell = ?, device_limit = ?, updated_at = ?
         WHERE id = ? AND owner_user_id = ? AND scope = 'node'"
    )
    .bind(parsed.min_trust_level.flatten().or(existing.min_trust_level.map(|v| v as i64)))
    .bind(free_quota)
    .bind(node_ids.unwrap_or_else(|| existing.node_ids.as_ref().map(|v| v.0.to_string()).unwrap_or_else(|| "[]".to_string())))
    .bind(parsed.transfer_enable.flatten().unwrap_or(existing.transfer_enable.unwrap_or(0) as i64))
    .bind(parsed.is_unlimited_traffic.unwrap_or(existing.is_unlimited_traffic))
    .bind(parsed.name.unwrap_or(existing.name))
    .bind(parsed.speed_limit.flatten().or(existing.speed_limit.map(|v| v as i64)))
    .bind(show)
    .bind(&visibility_scope)
    .bind(access_user_ids)
    .bind(share_token)
    .bind(parsed.sort.unwrap_or(existing.sort.unwrap_or(0)))
    .bind(parsed.renew.unwrap_or(existing.renew))
    .bind(content)
    .bind(parsed.prices.map(|v| v.to_string()).unwrap_or_else(|| existing.prices.as_ref().map(|v| v.0.to_string()).unwrap_or_else(|| "{}".to_string())))
    .bind(parsed.capacity_limit.flatten().or(existing.capacity_limit.map(|v| v as i64)))
    .bind(parsed.sell.unwrap_or(existing.sell))
    .bind(parsed.device_limit.flatten().or(existing.device_limit.map(|v| v as i64)))
    .bind(now)
    .bind(plan_id)
    .bind(user.id)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    let refreshed = load_owned_node_plan_by_id(state, user.id, plan_id).await.map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::BAD_REQUEST, "Subscription plan does not exist"))?;
    let share_base = get_setting_string(state, "app_url", "").await;
    let system_reset_method = get_setting_int(state, "reset_traffic_method", 2).await;
    Ok(json_value_response(success_response_payload(
        serialize_guest_plan(&refreshed, share_base.trim_end_matches('/'), system_reset_method),
    )))
}

async fn build_destroy_response(
    state: &AppState,
    plan_id: i64,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let deleted = sqlx::query("DELETE FROM v2_plan WHERE id = ? AND owner_user_id = ? AND scope = 'node'")
        .bind(plan_id)
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    if deleted.rows_affected() == 0 {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Subscription plan does not exist"));
    }
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}
