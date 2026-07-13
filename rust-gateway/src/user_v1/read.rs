use crate::*;

pub async fn info(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_info_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn me(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_me_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn comm_config(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_comm_config_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn comm_get_stripe_public_key(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_comm_get_stripe_public_key_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn telegram_get_bot_info(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_telegram_get_bot_info_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn knowledge_fetch(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_knowledge_fetch_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn knowledge_get_category(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_knowledge_get_category_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn stat_get_traffic_log(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_stat_get_traffic_log_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_subscribe(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_subscribe_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn check_login(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_check_login_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_stat(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_stat_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn plan_fetch(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_plan_fetch_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn plan_quota(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_plan_quota_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_info_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let cache_key = build_user_cache_key(&uri, user.id);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }
    let avatar_url = format!("https://cdn.v2ex.com/gravatar/{:x}?s=64&d=identicon", md5::compute(user.email.as_bytes()));
    Ok(success_cached_response(state, cache_key, json!({
        "email": user.email,
        "transfer_enable": user.transfer_enable,
        "last_login_at": user.last_login_at,
        "created_at": user.created_at,
        "banned": user.banned != 0,
        "ban_reason": user.ban_reason,
        "remind_expire": user.remind_expire != 0,
        "remind_traffic": user.remind_traffic != 0,
        "expired_at": user.expired_at,
        "balance": user.balance,
        "commission_balance": user.commission_balance,
        "plan_id": user.plan_id,
        "discount": user.discount,
        "commission_rate": user.commission_rate,
        "telegram_id": user.telegram_id,
        "uuid": user.uuid,
        "avatar_url": avatar_url,
    }), Duration::from_secs(5)))
}

async fn build_me_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let cache_key = build_user_cache_key(&uri, user.id);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }
    Ok(success_cached_response(state, cache_key, json!({
        "id": user.id,
        "email": user.email,
        "is_admin": account_has_role(user.is_admin, user.is_super_admin, RequiredAccountRole::Admin),
        "is_super_admin": user.is_super_admin != 0,
        "trust_level": user.trust_level,
        "is_silenced": user.is_silenced != 0,
        "is_linux_do_user": user.linux_do_id.is_some(),
        "linux_do_username": user.linux_do_username,
        "linux_do_name": user.linux_do_name,
        "linux_do_avatar": user.linux_do_avatar,
        "api_key": user.api_key,
        "concurrent_ip_limit": if user.concurrent_ip_limit > 0 { user.concurrent_ip_limit } else { 3 },
        "refund_dispute_enable": get_setting_bool(state, "refund_dispute_enable", false).await,
        "secure_path": if account_has_role(user.is_admin, user.is_super_admin, RequiredAccountRole::Admin) {
            Some(get_setting_string(state, "secure_path", &get_setting_string(state, "frontend_admin_path", "").await).await)
        } else {
            None
        },
    }), Duration::from_secs(5)))
}

async fn build_comm_config_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _user = authenticate_bearer_user(state, &headers).await?;
    let cache_key = build_cache_key(&uri);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }
    let snapshot = load_user_comm_config_snapshot(state).await;
    Ok(success_cached_response(state, cache_key, json!({
        "is_telegram": if snapshot.telegram_enabled { 1 } else { 0 },
        "telegram_discuss_link": snapshot.telegram_discuss_link,
        "stripe_pk": snapshot.stripe_pk,
        "withdraw_methods": snapshot.withdraw_methods,
        "withdraw_close": if snapshot.withdraw_close { 1 } else { 0 },
        "currency": snapshot.currency,
        "currency_symbol": snapshot.currency_symbol,
        "commission_distribution_enable": if snapshot.commission_distribution_enable { 1 } else { 0 },
        "commission_distribution_l1": snapshot.commission_distribution_l1,
        "commission_distribution_l2": snapshot.commission_distribution_l2,
        "commission_distribution_l3": snapshot.commission_distribution_l3,
    }), Duration::from_secs(15)))
}

async fn build_comm_get_stripe_public_key_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _user = authenticate_bearer_user(state, &headers).await?;
    let mut payment_id = parse_query(&uri)
        .get("id")
        .and_then(|value| value.parse::<i64>().ok());
    if payment_id.is_none() {
        let payload = parse_json_body(body).await?;
        payment_id = payload.get("id").and_then(parse_i64_value);
    }
    let Some(payment_id) = payment_id else {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "payment is not found"));
    };

    let row = sqlx::query(
        "SELECT config
         FROM v2_payment
         WHERE id = ? AND payment = 'StripeCredit'
         LIMIT 1"
    )
    .bind(payment_id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?;
    let Some(row) = row else {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "payment is not found"));
    };
    let raw = row.try_get::<String, _>("config").unwrap_or_default();
    let config: Value = serde_json::from_str(&raw).unwrap_or(Value::Null);
    let stripe_pk = config
        .get("stripe_pk_live")
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.to_string());
    let Some(stripe_pk) = stripe_pk else {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "payment is not found"));
    };

    Ok(json_value_response(success_response_payload(Value::String(stripe_pk))))
}

async fn build_telegram_get_bot_info_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let configured_bot_token = get_setting_string(state, "telegram_bot_token", "").await;
    let bot_token = if configured_bot_token.trim().is_empty() {
        env::var("TELEGRAM_BOT_TOKEN")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_default()
    } else {
        configured_bot_token
    };
    if bot_token.trim().is_empty() {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "telegram bot is not configured"));
    }

    let api_base = env::var("TELEGRAM_API_BASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "https://api.telegram.org/bot".to_string());
    let target = format!("{api_base}{bot_token}/getMe");
    let req_uri: Uri = target
        .parse()
        .map_err(|err| fail_json_response(StatusCode::BAD_GATEWAY, &format!("invalid telegram uri: {err}")))?;
    let request = Request::builder()
        .method(Method::GET)
        .uri(req_uri)
        .body(Body::empty())
        .map_err(|err| fail_json_response(StatusCode::BAD_GATEWAY, &format!("build request failed: {err}")))?;

    let response = state
        .backend_client
        .request(request)
        .await
        .map_err(|err| {
            error!("telegram request failed: {err}");
            fail_json_response(StatusCode::BAD_GATEWAY, "telegram unavailable")
        })?;
    let bytes = match response.into_body().collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Ok(fail_json_response(StatusCode::BAD_GATEWAY, &format!("read telegram body failed: {err}")));
        }
    };
    let value: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    let username = value
        .get("result")
        .and_then(|result| result.get("username"))
        .and_then(|value| value.as_str())
        .map(|value| value.to_string())
        .ok_or_else(|| fail_json_response(StatusCode::BAD_GATEWAY, "telegram response invalid"))?;

    let bind_code = Uuid::new_v4().simple().to_string();
    let cache_key = format!("TELEGRAM_BIND_{}", sha256_hex(&bind_code));
    redis_setex_string(state, &cache_key, 600, &user.id.to_string())
        .await
        .map_err(|err| {
            error!("telegram bind code cache write failed: {err}");
            json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "telegram bind code unavailable",
            )
        })?;

    Ok(json_value_response(success_response_payload(json!({
        "username": username,
        "bind_code": bind_code,
        "bind_command": format!("/bind {bind_code}"),
        "bind_expires_in": 600,
    }))))
}

async fn build_knowledge_fetch_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let params = parse_query(&uri);
    let subscribe_url = build_user_subscribe_url(state, &user).await.unwrap_or_default();
    let app_name = get_setting_string(state, "app_name", "Portal").await;
    let user_available = user_is_available(&user);

    if let Some(id) = params.get("id").and_then(|value| value.parse::<i64>().ok()).filter(|value| *value > 0) {
        let row = sqlx::query_as::<_, KnowledgeRow>(
            "SELECT id, language, category, title, body, sort, `show`, created_at, updated_at
             FROM v2_knowledge
             WHERE `show` = 1 AND id = ?
             LIMIT 1"
        )
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?;

        let Some(row) = row else {
            return Ok(fail_json_response(StatusCode::NOT_FOUND, "Article does not exist"));
        };

        return Ok(json_value_response(success_response_payload(render_knowledge_row(
            &row,
            user_available,
            &subscribe_url,
            &app_name,
        ))));
    }

    let language = params
        .get("language")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let keyword = params
        .get("keyword")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    if language.is_none() {
        return Ok(json_value_response(success_response_payload(Value::Object(Map::new()))));
    }

    let mut sql = String::from(
        "SELECT id, language, category, title, body, sort, `show`, created_at, updated_at
         FROM v2_knowledge
         WHERE `show` = 1"
    );
    if language.is_some() {
        sql.push_str(" AND language = ?");
    }
    if keyword.is_some() {
        sql.push_str(" AND (title LIKE ? OR body LIKE ?)");
    }
    sql.push_str(" ORDER BY sort ASC, id ASC");

    let mut query = sqlx::query_as::<_, KnowledgeRow>(&sql);
    if let Some(language) = &language {
        query = query.bind(language);
    }
    if let Some(keyword) = &keyword {
        let pattern = format!("%{}%", keyword);
        query = query.bind(pattern.clone()).bind(pattern);
    }

    let rows = query.fetch_all(&state.db).await.map_err(internal_error)?;
    let mut grouped = Map::new();
    for row in rows {
        let item = render_knowledge_row(&row, user_available, &subscribe_url, &app_name);
        grouped
            .entry(row.category.clone())
            .or_insert_with(|| Value::Array(Vec::new()))
            .as_array_mut()
            .unwrap()
            .push(item);
    }

    Ok(json_value_response(success_response_payload(Value::Object(grouped))))
}

async fn build_knowledge_get_category_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _user = authenticate_bearer_user(state, &headers).await?;
    let rows = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT category
         FROM v2_knowledge
         WHERE `show` = 1
         ORDER BY category ASC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(json_value_response(success_response_payload(json!(rows))))
}

async fn build_stat_get_traffic_log_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let _params = parse_query(&uri);
    let start_at = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
    let rows = sqlx::query_as::<_, StatTrafficLogRow>(
        "SELECT user_id, u, d, record_at, CAST(server_rate AS DOUBLE) AS server_rate
         FROM v2_stat_user
         WHERE user_id = ?
           AND record_at >= ?
         ORDER BY record_at DESC"
    )
    .bind(user.id)
    .bind(start_at)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    let include_user_id = !std::env::var("ENABLE_EXPOSED_USER_COUNT_FIX")
        .ok()
        .map(|value| value == "3f06f182")
        .unwrap_or(false);

    let data = rows
        .iter()
        .map(|row| {
            let mut item = json!({
                "d": row.d,
                "u": row.u,
                "record_at": row.record_at,
                "server_rate": row.server_rate,
            });
            if include_user_id {
                if let Some(object) = item.as_object_mut() {
                    object.insert("user_id".to_string(), Value::from(row.user_id));
                }
            }
            item
        })
        .collect::<Vec<_>>();

    Ok(json_value_response(success_response_payload(json!(data))))
}

async fn build_get_subscribe_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let cache_key = build_user_cache_key(&uri, user.id);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }
    let subscribe_url = build_user_subscribe_url(state, &user).await;
    let reset_day = resolve_user_reset_day(&user);
    Ok(success_cached_response(state, cache_key, json!({
        "plan_id": user.plan_id,
        "token": user.token,
        "expired_at": user.expired_at,
        "u": user.u,
        "d": user.d,
        "transfer_enable": user.transfer_enable,
        "email": user.email,
        "uuid": user.uuid,
        "device_limit": user.device_limit,
        "speed_limit": user.speed_limit,
        "next_reset_at": user.next_reset_at,
        "subscribe_path": user.subscribe_path,
        "subscribe_key": user.subscribe_key,
        "subscribe_salt": user.subscribe_salt,
        "plan": Value::Null,
        "subscribe_url": subscribe_url,
        "reset_day": reset_day,
    }), Duration::from_secs(5)))
}

async fn build_check_login_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let cache_key = build_user_cache_key(&uri, user.id);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }
    let mut data = json!({ "is_login": true });
    if account_has_role(user.is_admin, user.is_super_admin, RequiredAccountRole::Admin) {
        data["is_admin"] = Value::Bool(true);
    }
    Ok(success_cached_response(state, cache_key, data, Duration::from_secs(5)))
}

async fn build_get_stat_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let cache_key = build_user_cache_key(&uri, user.id);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }
    let stat = load_user_panel_stat_row(state, user.id)
        .await
        .map_err(internal_error)?;

    Ok(success_cached_response(
        state,
        cache_key,
        json!([stat.pending_orders, stat.open_tickets, stat.invitees]),
        Duration::from_secs(5),
    ))
}

async fn build_plan_fetch_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let params = parse_query(&uri);
    let share_base = get_setting_string(state, "app_url", "").await;
    let system_reset_method = get_setting_int(state, "reset_traffic_method", 2).await;

    if let Some(token) = params.get("token").map(|value| value.trim()).filter(|value| !value.is_empty()) {
        let plan = load_user_visible_plan_by_share_token(state, token)
            .await
            .map_err(internal_error)?;
        let Some(plan) = plan else {
            return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Subscription plan does not exist"));
        };
        if !plan_available_for_user(&plan, &user, Some(token), state).await.map_err(internal_error)? {
            return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Subscription plan does not exist"));
        }
        return Ok(json_value_response(success_response_payload(
            serialize_guest_plan(&plan, share_base.trim_end_matches('/'), system_reset_method),
        )));
    }

    if let Some(plan_id) = params.get("id").and_then(|value| value.parse::<i64>().ok()) {
        let plan = load_user_visible_plan_by_id(state, plan_id)
            .await
            .map_err(internal_error)?;
        let Some(plan) = plan else {
            return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Subscription plan does not exist"));
        };
        if !plan_available_for_user(&plan, &user, None, state).await.map_err(internal_error)? {
            return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Subscription plan does not exist"));
        }
        return Ok(json_value_response(success_response_payload(
            serialize_guest_plan(&plan, share_base.trim_end_matches('/'), system_reset_method),
        )));
    }

    let plans = load_user_public_sellable_plans(state)
        .await
        .map_err(internal_error)?;
    let items = filter_available_plans_for_user(
        state,
        &user,
        plans,
        share_base.trim_end_matches('/'),
        system_reset_method,
    )
    .await
    .map_err(internal_error)?;

    Ok(json_value_response(success_response_payload(Value::Array(items))))
}

async fn build_plan_quota_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let cache_key = build_user_cache_key(&uri, user.id);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }
    let subscriptions = load_active_user_plan_subscriptions(state, user.id).await.map_err(internal_error)?;
    let items = subscriptions
        .iter()
        .map(|subscription| {
            let allowance = subscription.traffic_allowance_kb.max(0);
            let used = subscription.used_traffic_kb.max(0);
            let is_unlimited = allowance >= 8_000_000_000_000_000_i64;
            let remaining = if is_unlimited {
                allowance
            } else {
                (allowance - used).max(0)
            };
            json!({
                "subscription_id": subscription.id,
                "order_id": subscription.order_id,
                "plan_id": subscription.plan_id,
                "plan_name": subscription.plan_name,
                "scope": subscription.plan_scope,
                "period": subscription.period,
                "started_at": subscription.started_at,
                "expired_at": subscription.expired_at,
                "traffic_allowance_kb": allowance,
                "used_traffic_kb": used,
                "remaining_traffic_kb": remaining,
                "usage_percent": if is_unlimited { Value::Null } else if allowance > 0 { Value::from(((used as f64 / allowance as f64) * 100.0).min(100.0).round() / 1.0) } else { Value::from(0) },
                "is_unlimited_traffic": is_unlimited,
                "node_ids": subscription.node_ids.as_ref().map(|v| v.0.clone()).unwrap_or(Value::Array(vec![])),
            })
        })
        .collect::<Vec<_>>();
    let has_unlimited = items.iter().any(|item| item.get("is_unlimited_traffic").and_then(|v| v.as_bool()).unwrap_or(false));
    let total_allowance = if has_unlimited {
        8_000_000_000_000_000_i64
    } else {
        items.iter().map(|item| item.get("traffic_allowance_kb").and_then(|v| v.as_i64()).unwrap_or(0)).sum()
    };
    let total_used: i64 = items.iter().map(|item| item.get("used_traffic_kb").and_then(|v| v.as_i64()).unwrap_or(0)).sum();
    let total_remaining = if has_unlimited { 8_000_000_000_000_000_i64 } else { (total_allowance - total_used).max(0) };

    Ok(success_cached_response(state, cache_key, json!({
        "items": items,
        "total_allowance_kb": total_allowance,
        "total_used_kb": total_used,
        "total_remaining_kb": total_remaining,
        "has_unlimited_traffic": has_unlimited,
    }), Duration::from_secs(5)))
}

async fn filter_available_plans_for_user(
    state: &AppState,
    user: &BearerUserRow,
    plans: Vec<PlanRow>,
    share_base: &str,
    system_reset_method: i64,
) -> Result<Vec<Value>, sqlx::Error> {
    if plans.is_empty() {
        return Ok(Vec::new());
    }

    let now = Utc::now().timestamp();
    let active_plan_ids = load_user_active_subscription_plan_ids(state, user.id, now).await?;
    let limited_plan_ids = plans
        .iter()
        .filter_map(|plan| match plan.capacity_limit {
            Some(limit) if limit > 0 => Some(plan.id),
            _ => None,
        })
        .collect::<Vec<_>>();
    let active_counts = load_plan_active_subscription_counts(state, &limited_plan_ids, now).await?;

    let mut items = Vec::new();
    for plan in plans {
        if !plan_available_for_user_from_prefetch(&plan, user, &active_plan_ids, &active_counts) {
            continue;
        }
        items.push(serialize_guest_plan(
            &plan,
            share_base,
            system_reset_method,
        ));
    }

    Ok(items)
}

fn plan_available_for_user_from_prefetch(
    plan: &PlanRow,
    user: &BearerUserRow,
    active_plan_ids: &HashSet<i64>,
    active_counts: &HashMap<i64, i64>,
) -> bool {
    if let Some(min_trust_level) = plan.min_trust_level {
        if (user.trust_level.max(0) as u64) < min_trust_level {
            return false;
        }
    }

    if active_plan_ids.contains(&plan.id) {
        return plan.renew;
    }

    if !plan.sell || !plan_has_remaining_capacity(plan.capacity_limit, active_counts.get(&plan.id).copied().unwrap_or(0)) {
        return false;
    }

    let scope = normalize_visibility_scope(plan.visibility_scope.as_str());
    if scope == "assigned_only" {
        return plan
            .access_user_ids
            .as_ref()
            .and_then(|json| json.0.as_array().cloned())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| parse_i64_value(&value))
            .any(|id| id == user.id);
    }

    scope != "link_only" && plan.show
}
