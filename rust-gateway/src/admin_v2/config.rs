use super::super::*;

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

pub async fn get_email_template(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_email_template_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn test_send_mail(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_test_send_mail_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn set_telegram_webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_set_telegram_webhook_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_fetch_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let key = params.get("key").map(|value| value.trim()).unwrap_or("");
    let mappings = load_config_mappings(state).await;
    if !key.is_empty() {
        if let Some(value) = mappings.get(key) {
            return Ok(json_value_response(success_response_payload(json!({
                key: value
            }))));
        }
    }
    Ok(json_value_response(success_response_payload(Value::Object(mappings))))
}

async fn build_get_email_template_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let files = vec!["classic".to_string(), "default".to_string()];
    Ok(json_value_response(success_response_payload(Value::Array(
        files.into_iter().map(Value::from).collect(),
    ))))
}

async fn build_test_send_mail_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    _body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let admin = authenticate_super_admin_user(state, &headers).await?;
    let app_name = config_value_string(state, "app_name", "Portal").await;
    let app_url = config_value_string(state, "app_url", &std::env::var("APP_URL").unwrap_or_default()).await;
    let subject = "This is a test email".to_string();
    let template_name = "mail.default.notify".to_string();
    let error = None::<String>;
    let now = Utc::now().timestamp();

    let inserted = sqlx::query(
        "INSERT INTO v2_mail_log (email, subject, template_name, error, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind(&admin.email)
    .bind(&subject)
    .bind(&template_name)
    .bind(error.clone())
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(json_value_response(json!({
        "data": {
            "id": inserted.last_insert_id(),
            "email": admin.email,
            "subject": subject,
            "template_name": template_name,
            "error": error,
            "config": {
                "app_name": app_name,
                "app_url": app_url,
                "driver": std::env::var("MAIL_DRIVER").unwrap_or_else(|_| "log".to_string()),
            }
        }
    })))
}

async fn build_set_telegram_webhook_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await.unwrap_or(Value::Null);

    let app_url = config_value_string(state, "app_url", "").await;
    if app_url.trim().is_empty() {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "请先设置站点网址"));
    }

    let token_override = payload
        .get("telegram_bot_token")
        .and_then(Value::as_str)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let token = if let Some(token) = token_override.clone() {
        token
    } else {
        let configured = config_value_string(state, "telegram_bot_token", "").await;
        if configured.trim().is_empty() {
            std::env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default()
        } else {
            configured
        }
    };
    if token.trim().is_empty() {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "请先设置 Telegram Bot Token"));
    }

    let secret_token = crate::telegram_security_support::telegram_webhook_secret_token(
        &token,
        &std::env::var("APP_KEY").unwrap_or_default(),
    );
    let hook_url = format!(
        "{}/api/v1/guest/telegram/webhook",
        app_url.trim_end_matches('/')
    );

    telegram_api_request(
        state,
        &token,
        "getMe",
        None,
        false,
    )
    .await?;
    telegram_api_request(
        state,
        &token,
        "setWebhook",
        Some(vec![
            ("url".to_string(), hook_url.clone()),
            ("secret_token".to_string(), secret_token.clone()),
        ]),
        false,
    )
    .await?;

    let _ = telegram_api_request(
        state,
        &token,
        "setMyCommands",
        Some(vec![
            (
                "commands".to_string(),
                serde_json::to_string(&vec![
                    json!({"command":"/start","description":"开始使用"}),
                    json!({"command":"/bind","description":"绑定账号"}),
                    json!({"command":"/traffic","description":"查看流量"}),
                    json!({"command":"/getlatesturl","description":"获取订阅链接"}),
                    json!({"command":"/tickets","description":"查看工单"}),
                    json!({"command":"/ticket","description":"查看工单详情"}),
                    json!({"command":"/close","description":"关闭工单"}),
                    json!({"command":"/unbind","description":"解绑账号"}),
                ])
                .unwrap_or_else(|_| "[]".to_string()),
            ),
            (
                "scope".to_string(),
                serde_json::to_string(&json!({"type":"default"})).unwrap_or_else(|_| "{\"type\":\"default\"}".to_string()),
            ),
        ]),
        true,
    )
    .await;

    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_save_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let obj = payload
        .as_object()
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let allowed_keys = allowed_config_keys();
    let mut updates = Vec::<(String, String)>::new();

    for (key, value) in obj {
        if key == "frontend_theme" {
            continue;
        }
        if !allowed_keys.contains(key.as_str()) {
            continue;
        }
        if key == "register_mode" {
            let mode = value.as_str().unwrap_or("").trim();
            if !matches!(mode, "all" | "email_only" | "oauth_only" | "closed") {
                return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
            }
        }
        if key == "secure_path" {
            let path = value.as_str().unwrap_or_default();
            if !is_valid_secure_admin_path(path) {
                return Ok(fail_json_response(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "secure_path must be an 8-64 character URL-safe segment",
                ));
            }
        }
        let serialized = serialize_setting_value(value);
        updates.push((key.clone(), serialized));
    }

    if let Some(path) = obj.get("secure_path").and_then(Value::as_str) {
        updates.push((
            "frontend_admin_path".to_string(),
            serialize_setting_value(&Value::String(path.to_string())),
        ));
    }

    if !obj.contains_key("register_mode") && obj.contains_key("stop_register") {
        let stop_register = obj.get("stop_register").and_then(Value::as_bool).unwrap_or(false);
        updates.push((
            "register_mode".to_string(),
            if stop_register {
                "\"closed\"".to_string()
            } else {
                "\"all\"".to_string()
            },
        ));
    }
    if obj.contains_key("register_mode") {
        let mode = obj.get("register_mode").and_then(Value::as_str).unwrap_or("all");
        updates.push((
            "stop_register".to_string(),
            if mode == "closed" { "1".to_string() } else { "0".to_string() },
        ));
    }

    updates.push((
        "frontend_theme".to_string(),
        format!("\"{}\"", theme_support::DEFAULT_THEME_NAME),
    ));
    updates.push((
        "current_theme".to_string(),
        format!("\"{}\"", theme_support::DEFAULT_THEME_NAME),
    ));

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    for (name, value) in updates {
        let affected = sqlx::query("UPDATE v2_settings SET value = ?, updated_at = NOW() WHERE name = ?")
            .bind(&value)
            .bind(&name)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?
            .rows_affected();
        if affected == 0 {
            sqlx::query(
                "INSERT INTO v2_settings (name, value, created_at, updated_at)
                 VALUES (?, ?, NOW(), NOW())"
            )
            .bind(&name)
            .bind(&value)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
        }
    }
    tx.commit().await.map_err(internal_error)?;
    clear_settings_cache(state);
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn load_config_mappings(state: &AppState) -> Map<String, Value> {
    let _ = warm_all_settings_cache(state).await;
    let register_mode = config_value_string(state, "register_mode", "email_password").await;
    let app_url = first_non_empty(&[
        config_value_string(state, "app_url", &std::env::var("APP_URL").unwrap_or_default()).await,
        std::env::var("APP_URL").unwrap_or_default(),
    ]);
    let app_url = app_url.trim_end_matches('/').to_string();
    let default_oauth_redirect = format!("{}/api/v1/passport/oauth2/linux-do/callback", app_url);

    let mut map = Map::new();
    map.insert("invite".to_string(), json!({
        "invite_force": config_bool(state, "invite_force", false).await,
        "invite_commission": config_i64(state, "invite_commission", 10).await,
        "invite_gen_limit": config_i64(state, "invite_gen_limit", 5).await,
        "invite_never_expire": config_bool(state, "invite_never_expire", false).await,
        "commission_first_time_enable": config_bool(state, "commission_first_time_enable", true).await,
        "commission_auto_check_enable": config_bool(state, "commission_auto_check_enable", true).await,
        "commission_withdraw_limit": config_i64(state, "commission_withdraw_limit", 100).await,
        "commission_withdraw_method": config_value_string(state, "commission_withdraw_method", "telegram,bank").await,
        "withdraw_close_enable": config_bool(state, "withdraw_close_enable", false).await,
        "commission_distribution_enable": config_bool(state, "commission_distribution_enable", false).await,
        "commission_distribution_l1": config_optional_value(state, "commission_distribution_l1").await,
        "commission_distribution_l2": config_optional_value(state, "commission_distribution_l2").await,
        "commission_distribution_l3": config_optional_value(state, "commission_distribution_l3").await,
    }));
    map.insert("site".to_string(), json!({
        "logo": config_optional_value(state, "logo").await,
        "force_https": config_i64(state, "force_https", 0).await,
        "stop_register": config_i64(state, "stop_register", 0).await,
        "register_mode": register_mode,
        "app_name": config_value_string(state, "app_name", "Portal").await,
        "app_description": config_value_string(state, "app_description", "Secure access portal").await,
        "app_url": config_optional_value(state, "app_url").await,
        "subscribe_url": config_optional_value(state, "subscribe_url").await,
        "subscribe_root_domains": config_value_string(state, "subscribe_root_domains", "").await,
        "try_out_plan_id": config_i64(state, "try_out_plan_id", 0).await,
        "try_out_hour": config_i64(state, "try_out_hour", 1).await,
        "tos_url": config_optional_value(state, "tos_url").await,
        "currency": config_value_string(state, "currency", "CNY").await,
        "currency_symbol": config_value_string(state, "currency_symbol", "¥").await,
    }));
    map.insert("subscribe".to_string(), json!({
        "plan_change_enable": config_bool(state, "plan_change_enable", true).await,
        "reset_traffic_method": config_i64(state, "reset_traffic_method", 0).await,
        "surplus_enable": config_bool(state, "surplus_enable", true).await,
        "new_order_event_id": config_i64(state, "new_order_event_id", 0).await,
        "renew_order_event_id": config_i64(state, "renew_order_event_id", 0).await,
        "change_order_event_id": config_i64(state, "change_order_event_id", 0).await,
        "show_info_to_server_enable": config_bool(state, "show_info_to_server_enable", false).await,
        "show_protocol_to_server_enable": config_bool(state, "show_protocol_to_server_enable", false).await,
        "default_remind_expire": config_bool(state, "default_remind_expire", true).await,
        "default_remind_traffic": config_bool(state, "default_remind_traffic", true).await,
        "subscribe_path": config_value_string(state, "subscribe_path", "s").await,
    }));
    map.insert("frontend".to_string(), json!({
        "frontend_theme": theme_support::DEFAULT_THEME_NAME,
        "frontend_theme_sidebar": config_value_string(state, "frontend_theme_sidebar", "light").await,
        "frontend_theme_header": config_value_string(state, "frontend_theme_header", "dark").await,
        "frontend_theme_color": config_value_string(state, "frontend_theme_color", "default").await,
        "frontend_background_url": config_optional_value(state, "frontend_background_url").await,
    }));
    map.insert("server".to_string(), json!({
        "server_token": config_optional_value(state, "server_token").await,
        "server_pull_interval": config_i64(state, "server_pull_interval", 60).await,
        "server_push_interval": config_i64(state, "server_push_interval", 60).await,
        "device_limit_mode": config_i64(state, "device_limit_mode", 0).await,
    }));
    map.insert("email".to_string(), json!({
        "email_template": config_value_string(state, "email_template", "default").await,
        "email_host": config_optional_value(state, "email_host").await,
        "email_port": config_optional_value(state, "email_port").await,
        "email_username": config_optional_value(state, "email_username").await,
        "email_password": config_optional_value(state, "email_password").await,
        "email_encryption": config_optional_value(state, "email_encryption").await,
        "email_from_address": config_optional_value(state, "email_from_address").await,
        "remind_mail_enable": config_bool(state, "remind_mail_enable", false).await,
    }));
    map.insert("telegram".to_string(), json!({
        "telegram_bot_enable": config_bool(state, "telegram_bot_enable", false).await,
        "telegram_bot_token": config_optional_value(state, "telegram_bot_token").await,
        "telegram_discuss_link": config_optional_value(state, "telegram_discuss_link").await,
        "telegram_user_ticket_enable": config_bool(state, "telegram_user_ticket_enable", true).await,
        "telegram_notify_ops_alert": config_bool(state, "telegram_notify_ops_alert", true).await,
        "telegram_notify_ticket_created": config_bool(state, "telegram_notify_ticket_created", true).await,
        "telegram_notify_ticket_replied": config_bool(state, "telegram_notify_ticket_replied", true).await,
        "telegram_notify_ticket_closed": config_bool(state, "telegram_notify_ticket_closed", true).await,
        "telegram_notify_payment_success": config_bool(state, "telegram_notify_payment_success", true).await,
        "telegram_notify_notice_published": config_bool(state, "telegram_notify_notice_published", true).await,
        "telegram_notify_tcping_alert": config_bool(state, "telegram_notify_tcping_alert", true).await,
        "telegram_notify_tcping_recover": config_bool(state, "telegram_notify_tcping_recover", true).await,
        "telegram_notify_refund_vote": config_bool(state, "telegram_notify_refund_vote", true).await,
        "telegram_notify_refund_status": config_bool(state, "telegram_notify_refund_status", true).await,
        "telegram_notify_user_risk_detected": config_bool(state, "telegram_notify_user_risk_detected", true).await,
        "telegram_notify_user_banned": config_bool(state, "telegram_notify_user_banned", true).await,
    }));
    map.insert("oauth".to_string(), json!({
        "oauth_linux_do_enable": config_bool(state, "oauth_linux_do_enable", true).await,
        "oauth_linux_do_client_id": config_value_string(state, "oauth_linux_do_client_id", "").await,
        "oauth_linux_do_client_secret": config_value_string(state, "oauth_linux_do_client_secret", "").await,
        "oauth_linux_do_redirect_uri": config_value_string(state, "oauth_linux_do_redirect_uri", &default_oauth_redirect).await,
    }));
    map.insert("app".to_string(), json!({
        "windows_version": config_value_string(state, "windows_version", "").await,
        "windows_download_url": config_value_string(state, "windows_download_url", "").await,
        "macos_version": config_value_string(state, "macos_version", "").await,
        "macos_download_url": config_value_string(state, "macos_download_url", "").await,
        "android_version": config_value_string(state, "android_version", "").await,
        "android_download_url": config_value_string(state, "android_download_url", "").await,
    }));
    map.insert("safe".to_string(), json!({
        "email_verify": config_bool(state, "email_verify", false).await,
        "safe_mode_enable": config_bool(state, "safe_mode_enable", false).await,
        "secure_path": config_value_string(state, "secure_path", "").await,
        "force_oauth2_login": config_bool(state, "force_oauth2_login", false).await,
        "login_token_expire_days": config_i64(state, "login_token_expire_days", 365).await,
        "email_whitelist_enable": config_bool(state, "email_whitelist_enable", false).await,
        "email_whitelist_suffix": config_value_string(state, "email_whitelist_suffix", "gmail.com,outlook.com").await,
        "email_gmail_limit_enable": config_bool(state, "email_gmail_limit_enable", false).await,
        "captcha_enable": config_bool(state, "captcha_enable", false).await,
        "captcha_type": config_value_string(state, "captcha_type", "recaptcha").await,
        "recaptcha_key": config_value_string(state, "recaptcha_key", "").await,
        "recaptcha_site_key": config_value_string(state, "recaptcha_site_key", "").await,
        "recaptcha_v3_secret_key": config_value_string(state, "recaptcha_v3_secret_key", "").await,
        "recaptcha_v3_site_key": config_value_string(state, "recaptcha_v3_site_key", "").await,
        "recaptcha_v3_score_threshold": config_f64(state, "recaptcha_v3_score_threshold", 0.5).await,
        "turnstile_secret_key": config_value_string(state, "turnstile_secret_key", "").await,
        "turnstile_site_key": config_value_string(state, "turnstile_site_key", "").await,
        "pow_enable": config_bool(state, "pow_enable", false).await,
        "pow_difficulty": config_i64(state, "pow_difficulty", 4).await,
        "pow_effective_difficulty": config_i64(state, "pow_difficulty", 4).await,
        "pow_ttl": config_i64(state, "pow_ttl", 120).await,
        "pow_seed_salt": config_value_string(state, "pow_seed_salt", "").await,
        "pow_base_value": config_value_string(state, "pow_base_value", "portal").await,
        "pow_require_ja3": config_bool(state, "pow_require_ja3", true).await,
        "pow_auto_scale_enable": config_bool(state, "pow_auto_scale_enable", true).await,
        "pow_auto_max_difficulty": config_i64(state, "pow_auto_max_difficulty", 7).await,
        "register_mode": register_mode,
        "register_limit_by_ip_enable": config_bool(state, "register_limit_by_ip_enable", false).await,
        "register_limit_count": config_i64(state, "register_limit_count", 3).await,
        "register_limit_expire": config_i64(state, "register_limit_expire", 60).await,
        "password_limit_enable": config_bool(state, "password_limit_enable", true).await,
        "password_limit_count": config_i64(state, "password_limit_count", 5).await,
        "password_limit_expire": config_i64(state, "password_limit_expire", 60).await,
        "recaptcha_enable": config_bool(state, "captcha_enable", false).await,
    }));
    map.insert("subscribe_template".to_string(), json!({
        "subscribe_template_singbox": config_value_string(state, "subscribe_template_singbox", "").await,
        "subscribe_template_clash": config_value_string(state, "subscribe_template_clash", "").await,
        "subscribe_template_clashmeta": config_value_string(state, "subscribe_template_clashmeta", "").await,
        "subscribe_template_stash": config_value_string(state, "subscribe_template_stash", "").await,
        "subscribe_template_surge": config_value_string(state, "subscribe_template_surge", "").await,
        "subscribe_template_surfboard": config_value_string(state, "subscribe_template_surfboard", "").await,
    }));
    map.insert("system".to_string(), json!({
        "rotate_subscription_credentials_daily": config_bool(state, "rotate_subscription_credentials_daily", false).await,
        "refund_dispute_enable": config_bool(state, "refund_dispute_enable", false).await,
        "node_traffic_records_retention_days": config_i64(state, "node_traffic_records_retention_days", 7).await,
        "user_traffic_usage_logs_retention_days": config_i64(state, "user_traffic_usage_logs_retention_days", 7).await,
        "tcping_samples_retention_days": config_i64(state, "tcping_samples_retention_days", 7).await,
        "tcping_alerts_retention_days": config_i64(state, "tcping_alerts_retention_days", 7).await,
        "audit_logs_retention_days": config_i64(state, "audit_logs_retention_days", 7).await,
    }));
    map
}

async fn config_optional_value(state: &AppState, key: &str) -> Value {
    load_cached_setting_value(state, key)
        .await
        .ok()
        .flatten()
        .and_then(|value| parse_setting_json_value(&value))
        .unwrap_or(Value::Null)
}

async fn config_value_string(state: &AppState, key: &str, default: &str) -> String {
    match config_optional_value(state, key).await {
        Value::String(value) => value,
        Value::Number(value) => value.to_string(),
        Value::Bool(value) => if value { "1".to_string() } else { "0".to_string() },
        Value::Null => default.to_string(),
        other => other.to_string(),
    }
}

async fn config_i64(state: &AppState, key: &str, default: i64) -> i64 {
    match config_optional_value(state, key).await {
        Value::Number(value) => value.as_i64().unwrap_or(default),
        Value::String(value) => value.parse::<i64>().unwrap_or(default),
        Value::Bool(value) => if value { 1 } else { 0 },
        _ => default,
    }
}

async fn config_f64(state: &AppState, key: &str, default: f64) -> f64 {
    match config_optional_value(state, key).await {
        Value::Number(value) => value.as_f64().unwrap_or(default),
        Value::String(value) => value.parse::<f64>().unwrap_or(default),
        _ => default,
    }
}

async fn config_bool(state: &AppState, key: &str, default: bool) -> bool {
    match config_optional_value(state, key).await {
        Value::Bool(value) => value,
        Value::Number(value) => value.as_i64().map(|v| v != 0).unwrap_or(default),
        Value::String(value) => matches!(value.as_str(), "1" | "true" | "TRUE"),
        _ => default,
    }
}

fn parse_setting_json_value(raw: &str) -> Option<Value> {
    if let Ok(value) = serde_json::from_str::<Value>(raw) {
        return Some(value);
    }
    Some(Value::String(raw.to_string()))
}

fn serialize_setting_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Null => String::new(),
        Value::Bool(flag) => {
            if *flag { "1".to_string() } else { "0".to_string() }
        }
        Value::Number(number) => number.to_string(),
        Value::Array(_) | Value::Object(_) => serde_json::to_string(value).unwrap_or_else(|_| String::new()),
    }
}

fn allowed_config_keys() -> HashSet<&'static str> {
    [
        "invite_force",
        "invite_commission",
        "invite_gen_limit",
        "invite_never_expire",
        "commission_first_time_enable",
        "commission_auto_check_enable",
        "commission_withdraw_limit",
        "commission_withdraw_method",
        "withdraw_close_enable",
        "commission_distribution_enable",
        "commission_distribution_l1",
        "commission_distribution_l2",
        "commission_distribution_l3",
        "logo",
        "force_https",
        "stop_register",
        "register_mode",
        "app_name",
        "app_description",
        "app_url",
        "subscribe_url",
        "subscribe_root_domains",
        "try_out_plan_id",
        "try_out_hour",
        "tos_url",
        "currency",
        "currency_symbol",
        "plan_change_enable",
        "reset_traffic_method",
        "surplus_enable",
        "new_order_event_id",
        "renew_order_event_id",
        "change_order_event_id",
        "show_info_to_server_enable",
        "show_protocol_to_server_enable",
        "subscribe_path",
        "frontend_theme_sidebar",
        "frontend_theme_header",
        "frontend_theme_color",
        "frontend_background_url",
        "server_token",
        "server_pull_interval",
        "server_push_interval",
        "device_limit_mode",
        "email_template",
        "email_host",
        "email_port",
        "email_username",
        "email_password",
        "email_encryption",
        "email_from_address",
        "remind_mail_enable",
        "windows_version",
        "windows_download_url",
        "macos_version",
        "macos_download_url",
        "android_version",
        "android_download_url",
        "email_verify",
        "safe_mode_enable",
        "secure_path",
        "force_oauth2_login",
        "login_token_expire_days",
        "email_whitelist_enable",
        "email_whitelist_suffix",
        "email_gmail_limit_enable",
        "captcha_enable",
        "captcha_type",
        "recaptcha_key",
        "recaptcha_site_key",
        "recaptcha_v3_secret_key",
        "recaptcha_v3_site_key",
        "recaptcha_v3_score_threshold",
        "turnstile_secret_key",
        "turnstile_site_key",
        "pow_enable",
        "pow_difficulty",
        "pow_ttl",
        "pow_seed_salt",
        "pow_base_value",
        "pow_require_ja3",
        "pow_auto_scale_enable",
        "pow_auto_max_difficulty",
        "register_limit_by_ip_enable",
        "register_limit_count",
        "register_limit_expire",
        "password_limit_enable",
        "password_limit_count",
        "password_limit_expire",
        "oauth_linux_do_enable",
        "oauth_linux_do_client_id",
        "oauth_linux_do_client_secret",
        "oauth_linux_do_redirect_uri",
        "default_remind_expire",
        "default_remind_traffic",
        "subscribe_template_singbox",
        "subscribe_template_clash",
        "subscribe_template_clashmeta",
        "subscribe_template_stash",
        "subscribe_template_surge",
        "subscribe_template_surfboard",
        "rotate_subscription_credentials_daily",
        "refund_dispute_enable",
        "node_traffic_records_retention_days",
        "user_traffic_usage_logs_retention_days",
        "tcping_samples_retention_days",
        "tcping_alerts_retention_days",
        "audit_logs_retention_days",
    ]
    .into_iter()
    .collect()
}

async fn telegram_api_request(
    state: &AppState,
    token: &str,
    method: &str,
    form_fields: Option<Vec<(String, String)>>,
    allow_failure: bool,
) -> Result<Value, Response<Body>> {
    let api_base = std::env::var("TELEGRAM_API_BASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "https://api.telegram.org/bot".to_string());
    let target = format!("{api_base}{token}/{method}");
    let uri: Uri = target
        .parse()
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("invalid telegram uri: {err}")))?;

    let mut request = Request::builder().uri(uri);
    let body = if let Some(form_fields) = form_fields {
        request = request.method(Method::POST).header(CONTENT_TYPE, "application/x-www-form-urlencoded");
        let payload = form_fields
            .into_iter()
            .map(|(key, value)| format!("{}={}", urlencoding::encode(&key), urlencoding::encode(&value)))
            .collect::<Vec<_>>()
            .join("&");
        Body::from(payload)
    } else {
        request = request.method(Method::GET);
        Body::empty()
    };
    let request = request
        .body(body)
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("build telegram request failed: {err}")))?;

    let response = state
        .backend_client
        .request(request)
        .await
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("telegram request failed: {err}")))?;
    let status = response.status();
    let bytes = response_body_bytes(map_proxy_response(response).await).await;
    let payload: Value = serde_json::from_slice(&bytes)
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("invalid telegram response: {err}")))?;

    if !status.is_success() {
        if allow_failure {
            return Ok(payload);
        }
        return Err(json_status_response(
            StatusCode::BAD_GATEWAY,
            json!({"message": format!("Telegram API 请求失败: HTTP {}", status.as_u16())}),
        ));
    }
    if payload.get("ok").and_then(Value::as_bool) == Some(false) && !allow_failure {
        let description = payload
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("未知错误");
        return Err(json_status_response(
            StatusCode::BAD_GATEWAY,
            json!({"message": format!("Telegram API 错误: {}", description)}),
        ));
    }
    Ok(payload)
}
