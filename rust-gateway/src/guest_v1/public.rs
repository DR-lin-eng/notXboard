use crate::*;

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

pub async fn overview(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_overview_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn leaderboards(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_leaderboards_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn geo(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_geo_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_comm_config_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let cache_key = build_cache_key(&uri);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }

    let telegram_only_mode = env_bool("TELEGRAM_ONLY_MODE", false);
    let register_mode = resolve_register_mode(state).await;
    let oauth_linux_do_available = resolve_oauth_linux_do_available(state).await;
    let allow_email_register = matches!(register_mode.as_str(), "all" | "email_only");
    let allow_oauth_register = oauth_linux_do_available && matches!(register_mode.as_str(), "all" | "oauth_only");
    let email_whitelist_enable = get_setting_bool(state, "email_whitelist_enable", false).await;
    let captcha_enable = get_setting_bool(state, "captcha_enable", false).await;
    let pow_difficulty = get_setting_int(state, "pow_difficulty", 4).await.clamp(1, 8);

    let expose_public_metadata = state.exposure.public_metadata_enabled();
    let (app_description, app_url, logo, windows_version, windows_download_url, macos_version, macos_download_url, android_version, android_download_url) =
        if expose_public_metadata {
            (
                get_setting_value(state, "app_description").await,
                get_setting_value(state, "app_url").await,
                get_setting_value(state, "logo").await,
                get_setting_value(state, "windows_version").await,
                get_setting_value(state, "windows_download_url").await,
                get_setting_value(state, "macos_version").await,
                get_setting_value(state, "macos_download_url").await,
                get_setting_value(state, "android_version").await,
                get_setting_value(state, "android_download_url").await,
            )
        } else {
            (
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
            )
        };

    let data = json!({
        "tos_url": get_setting_value(state, "tos_url").await,
        "is_email_verify": if telegram_only_mode { 0 } else if get_setting_bool(state, "email_verify", false).await { 1 } else { 0 },
        "is_invite_force": if get_setting_bool(state, "invite_force", false).await { 1 } else { 0 },
        "register_mode": register_mode,
        "allow_email_register": if allow_email_register { 1 } else { 0 },
        "allow_oauth_register": if allow_oauth_register { 1 } else { 0 },
        "email_whitelist_suffix": if email_whitelist_enable {
            setting_json_or_csv_array(state, "email_whitelist_suffix", &[
                "gmail.com", "qq.com", "163.com", "yahoo.com", "sina.com", "126.com", "outlook.com", "yeah.net", "foxmail.com",
            ]).await
        } else {
            Value::from(0)
        },
        "is_captcha": if captcha_enable { 1 } else { 0 },
        "captcha_type": get_setting_string(state, "captcha_type", "recaptcha").await,
        "recaptcha_site_key": get_setting_value(state, "recaptcha_site_key").await,
        "recaptcha_v3_site_key": get_setting_value(state, "recaptcha_v3_site_key").await,
        "recaptcha_v3_score_threshold": get_setting_f64(state, "recaptcha_v3_score_threshold", 0.5).await,
        "turnstile_site_key": get_setting_value(state, "turnstile_site_key").await,
        "pow_enable": if get_setting_bool(state, "pow_enable", false).await { 1 } else { 0 },
        "pow_difficulty": pow_difficulty,
        "pow_effective_difficulty": resolve_pow_effective_difficulty(state, pow_difficulty).await,
        "pow_ttl": get_setting_int(state, "pow_ttl", 120).await.clamp(30, 600),
        "pow_algo": "sha256-prefix-zeros",
        "app_description": app_description,
        "app_url": app_url,
        "logo": logo,
        "windows_version": windows_version,
        "windows_download_url": windows_download_url,
        "macos_version": macos_version,
        "macos_download_url": macos_download_url,
        "android_version": android_version,
        "android_download_url": android_download_url,
        "force_oauth2_login": if get_setting_bool(state, "force_oauth2_login", false).await { 1 } else { 0 },
        "oauth_linux_do_enable": if oauth_linux_do_available { 1 } else { 0 },
        "is_recaptcha": if captcha_enable { 1 } else { 0 },
    });

    Ok(success_cached_response(state, cache_key, data, Duration::from_secs(15)))
}

async fn build_plan_fetch_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    if !state.exposure.public_catalog_enabled() {
        return Err(json_error(StatusCode::NOT_FOUND, "Not found"));
    }
    let cache_key = build_cache_key(&uri);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }

    let plans = load_guest_available_plans(state).await.map_err(internal_error)?;
    let share_base = get_setting_string(state, "app_url", "").await;
    let system_reset_method = get_setting_int(state, "reset_traffic_method", 1).await;
    let data = plans
        .iter()
        .map(|plan| serialize_guest_plan(plan, share_base.trim_end_matches('/'), system_reset_method))
        .collect::<Vec<_>>();

    Ok(success_cached_response(state, cache_key, Value::Array(data), Duration::from_secs(15)))
}

async fn build_overview_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    if !state.exposure.public_dashboard_enabled() {
        return Err(json_error(StatusCode::NOT_FOUND, "Not found"));
    }
    let cache_key = build_cache_key(&uri);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }

    record_public_country_hit(state, &headers);
    let metrics = load_public_overview_metrics(state).await.map_err(internal_error)?;
    let avg_bandwidth = load_public_average_bandwidth(state).await.map_err(internal_error)?;

    Ok(json_cached_response(
        state,
        cache_key,
        json!({
            "code": 0,
            "message": "success",
            "data": {
                "active_users": metrics.active_users,
                "registered_users": metrics.registered_users,
                "node_count": metrics.node_count,
                "avg_bandwidth": avg_bandwidth,
            }
        }),
        Duration::from_secs(15),
    ))
}

async fn build_leaderboards_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    if !state.exposure.public_dashboard_enabled() {
        return Err(json_error(StatusCode::NOT_FOUND, "Not found"));
    }
    let cache_key = build_cache_key(&uri);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }

    record_public_country_hit(state, &headers);
    let top_users = load_public_top_users(state).await.map_err(internal_error)?;
    let top_nodes = load_public_top_nodes(state).await.map_err(internal_error)?;
    let regions = public_country_counts(state);

    Ok(json_cached_response(
        state,
        cache_key,
        json!({
            "code": 0,
            "message": "success",
            "data": {
                "top_users": top_users,
                "top_nodes": top_nodes,
                "regions": regions,
            }
        }),
        Duration::from_secs(15),
    ))
}

async fn build_geo_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    if !state.exposure.public_dashboard_enabled() {
        return Err(json_error(StatusCode::NOT_FOUND, "Not found"));
    }
    let cache_key = build_cache_key(&uri);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }

    record_public_country_hit(state, &headers);
    let regions = public_country_counts(state);

    Ok(json_cached_response(
        state,
        cache_key,
        json!({
            "code": 0,
            "message": "success",
            "data": {
                "regions": regions,
            }
        }),
        Duration::from_secs(15),
    ))
}

pub fn record_public_country_hit(state: &AppState, headers: &HeaderMap) {
    let ip = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    let country = resolve_country_code(headers);
    let mut cache = state.public_geo_cache.write();
    let now = Instant::now();
    cache.retain(|_, hit| hit.expires_at > now);
    cache.entry(ip).or_insert(CountryHit {
        country,
        expires_at: now + Duration::from_secs(86_400),
    });
}

fn resolve_country_code(headers: &HeaderMap) -> String {
    for key in ["cf-ipcountry", "x-country-code", "x-geo-country", "x-country"] {
        if let Some(value) = headers.get(key).and_then(|value| value.to_str().ok()) {
            let code = value.trim().to_uppercase();
            if !code.is_empty() && code != "XX" {
                return code;
            }
        }
    }
    "ZZ".to_string()
}

pub fn public_country_counts(state: &AppState) -> Vec<Value> {
    let mut counts = HashMap::<String, i64>::new();
    let now = Instant::now();
    let mut cache = state.public_geo_cache.write();
    cache.retain(|_, hit| hit.expires_at > now);
    for hit in cache.values() {
        *counts.entry(hit.country.clone()).or_insert(0) += 1;
    }
    let mut items = counts
        .into_iter()
        .map(|(code, count)| json!({ "code": code, "count": count }))
        .collect::<Vec<_>>();
    items.sort_by(|a, b| {
        b.get("count")
            .and_then(|v| v.as_i64())
            .unwrap_or_default()
            .cmp(&a.get("count").and_then(|v| v.as_i64()).unwrap_or_default())
    });
    items
}

pub fn serialize_guest_plan(plan: &PlanRow, share_base: &str, system_reset_method: i64) -> Value {
    let visibility_scope = normalize_visibility_scope(plan.visibility_scope.as_str());
    let share_token = plan.share_token.clone().unwrap_or_default();
    let owner_display_name = plan_owner_display_name(plan);
    let share_purchase_link = if !share_token.is_empty() && !share_base.is_empty() {
        Some(format!("{}/app/#/plan-link/{}", share_base, urlencoding::encode(&share_token)))
    } else {
        None
    };
    let content = crate::html_safety_support::sanitize_rich_html(&format_plan_content(
        plan.content.as_deref().unwrap_or(""),
        plan.transfer_enable.unwrap_or(0),
        plan.speed_limit,
        plan.device_limit,
        plan.reset_traffic_method.unwrap_or(system_reset_method),
        system_reset_method,
    ));
    let prices = plan.prices.as_ref().map(|json| json.0.clone()).unwrap_or(Value::Null);
    let price_map = prices.as_object().cloned().unwrap_or_default();
    let free_quota = plan
        .free_quota_gb_by_trust_level
        .as_ref()
        .map(|json| json.0.clone())
        .unwrap_or(Value::Null);
    let node_ids = plan.node_ids.as_ref().map(|json| json.0.clone()).unwrap_or(Value::Null);
    let access_user_ids = plan
        .access_user_ids
        .as_ref()
        .map(|json| json.0.clone())
        .unwrap_or_else(|| json!([]));
    let tags = plan.tags.as_ref().map(|json| json.0.clone()).unwrap_or(Value::Null);

    json!({
        "id": plan.id,
        "scope": if plan.scope.trim().is_empty() { "legacy" } else { plan.scope.as_str() },
        "owner_user_id": plan.owner_user_id,
        "owner_display_name": owner_display_name,
        "owner": serialize_plan_owner(plan),
        "min_trust_level": plan.min_trust_level,
        "allow_trial": plan_has_trial_quota(&free_quota),
        "free_quota_gb_by_trust_level": free_quota,
        "paid_quota_gb": plan.transfer_enable.unwrap_or(0),
        "is_unlimited_traffic": plan.is_unlimited_traffic,
        "node_ids": node_ids,
        "visibility_scope": visibility_scope,
        "access_user_ids": access_user_ids,
        "share_token": if share_token.is_empty() { Value::Null } else { Value::String(share_token.clone()) },
        "share_purchase_link": share_purchase_link,
        "group_id": plan.group_id,
        "name": plan.name,
        "tags": tags,
        "content": content,
        "month_price": price_to_legacy_number(price_map.get("monthly")),
        "quarter_price": price_to_legacy_number(price_map.get("quarterly")),
        "half_year_price": price_to_legacy_number(price_map.get("half_yearly")),
        "year_price": price_to_legacy_number(price_map.get("yearly")),
        "two_year_price": price_to_legacy_number(price_map.get("two_yearly")),
        "three_year_price": price_to_legacy_number(price_map.get("three_yearly")),
        "onetime_price": price_to_legacy_number(price_map.get("onetime")),
        "reset_price": price_to_legacy_number(price_map.get("reset_traffic")),
        "capacity_limit": format_capacity_limit(plan.capacity_limit),
        "transfer_enable": plan.transfer_enable.unwrap_or(0),
        "speed_limit": plan.speed_limit,
        "device_limit": plan.device_limit,
        "show": plan.show,
        "sell": plan.sell,
        "renew": plan.renew,
        "reset_traffic_method": plan.reset_traffic_method.unwrap_or(system_reset_method),
        "sort": plan.sort,
        "created_at": plan.created_at,
        "updated_at": plan.updated_at
    })
}
