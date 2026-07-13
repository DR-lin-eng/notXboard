use crate::*;
use crate::mail_support::{send_platform_mail, MailSendRequest};

pub async fn pow_challenge(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_pow_challenge_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_login_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_quick_login_url(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_get_quick_login_url_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn token2_login(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_token2_login_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn login_with_mail_link(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_login_with_mail_link_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn forget(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_forget_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_register_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_pow_challenge_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    if !get_setting_bool(state, "pow_enable", false).await {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Pow disabled"));
    }

    let ja3 = extract_ja3(&headers);
    if get_setting_bool(state, "pow_require_ja3", true).await && ja3.is_none() {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "JA3 fingerprint missing"));
    }

    let issued_at = Utc::now().timestamp();
    let ttl = get_setting_int(state, "pow_ttl", 120).await.clamp(30, 600);
    let expires_at = issued_at + ttl;
    let configured_difficulty = get_setting_int(state, "pow_difficulty", 4).await.clamp(1, 8);
    let difficulty = resolve_pow_effective_difficulty(state, configured_difficulty).await;
    let seed_salt = get_setting_string(state, "pow_seed_salt", "").await;
    let seed = random_seed_hex(&seed_salt)?;
    let base = get_setting_string(state, "pow_base_value", "portal").await;
    let challenge_id = Uuid::new_v4().to_string();
    let token = build_pow_token(&state.app_key, ja3.as_deref().unwrap_or(""), issued_at, &seed);
    let ja3_hash = ja3.as_deref().map(sha256_hex);
    let ip = request_ip(&headers).unwrap_or_else(|| "127.0.0.1".to_string());

    write_pow_challenge_cache(
        state,
        &challenge_id,
        &PowChallengeCacheEntry {
            seed: seed.clone(),
            base: base.clone(),
            difficulty,
            issued_at,
            expires_at,
            token: token.clone(),
            ja3_hash,
            ip: ip.clone(),
        },
        ttl,
    )
    .await
    .map_err(|err| {
        error!("pow challenge cache write failed: {}", err);
        json_error(StatusCode::INTERNAL_SERVER_ERROR, "pow cache write failed")
    })?;

    Ok(json_value_response(success_response_payload(json!({
        "challenge_id": challenge_id,
        "seed": seed,
        "base": base,
        "difficulty": difficulty,
        "issued_at": issued_at,
        "expires_at": expires_at,
        "token": token,
        "algo": "sha256-prefix-zeros",
    }))))
}

async fn build_login_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let payload = parse_json_body(body).await?;
    let email = payload
        .get("email")
        .and_then(|v| v.as_str())
        .map(|v| v.trim().to_ascii_lowercase())
        .unwrap_or_default();
    let password = payload.get("password").and_then(|v| v.as_str()).map(|v| v.to_string()).unwrap_or_default();

    if email.is_empty() {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Email can not be empty"));
    }
    if !email.contains('@') {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Email format is incorrect"));
    }
    if password.is_empty() {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Password can not be empty"));
    }
    if password.chars().count() < 8 {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Password must be greater than 8 digits"));
    }

    if !verify_captcha_disabled_or_supported(state).await {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Invalid captcha type"));
    }

    let pow_valid = verify_pow_payload(state, &headers, &payload).await?;
    if !pow_valid {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Pow verification failed"));
    }

    let password_limit_enable = get_setting_bool(state, "password_limit_enable", true).await;
    let password_limit_count = get_setting_int(state, "password_limit_count", 5).await.max(1);
    let password_limit_expire = get_setting_int(state, "password_limit_expire", 60).await.max(1);
    let password_limit_key = format!("PASSWORD_ERROR_LIMIT_{}", email);

    if password_limit_enable {
        let attempts = redis_get_string(state, &password_limit_key).await.ok().flatten().and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
        if attempts >= password_limit_count {
            return Ok(fail_json_response(
                StatusCode::TOO_MANY_REQUESTS,
                &format!("There are too many password errors, please try again after {} minutes.", password_limit_expire),
            ));
        }
    }

    let user = load_login_user_by_email(state, &email).await.map_err(internal_error)?;
    let user = match user {
        Some(user) => user,
        None => return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Incorrect email or password")),
    };

    if !can_use_email_login(state, user.is_super_admin != 0).await {
        return Ok(fail_json_response(StatusCode::FORBIDDEN, "Email login is disabled. Please use OAuth2 login"));
    }

    let password_ok = verify_login_password(&user, &password);
    if !password_ok {
        if password_limit_enable {
            let attempts = redis_get_string(state, &password_limit_key).await.ok().flatten().and_then(|v| v.parse::<i64>().ok()).unwrap_or(0) + 1;
            let _ = redis_setex_string(state, &password_limit_key, password_limit_expire * 60, &attempts.to_string()).await;
        }
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Incorrect email or password"));
    }

    if password_limit_enable {
        let _ = redis_del_key(state, &password_limit_key).await;
    }

    if user.banned != 0 {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, &user_suspension_message(&user)));
    }

    let auth_data = issue_personal_access_token(state, &user).await.map_err(|err| {
        error!("issue personal access token failed: {}", err);
        json_error(StatusCode::INTERNAL_SERVER_ERROR, "token issue failed")
    })?;
    update_login_timestamp(state, user.id).await.map_err(internal_error)?;

    Ok(json_value_response(success_response_payload(json!({
        "token": user.token,
        "auth_data": auth_data,
        "is_admin": user.is_admin != 0,
    }))))
}

async fn build_get_quick_login_url_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let payload = parse_json_body(body).await?;
    let authorization = payload
        .get("auth_data")
        .and_then(|v| v.as_str())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .or_else(|| {
            headers
                .get("authorization")
                .and_then(|value| value.to_str().ok())
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        });

    let Some(authorization) = authorization else {
        return Ok(json_status_response(
            StatusCode::UNAUTHORIZED,
            json!({ "message": [401001, "授权失败，请先登录"] }),
        ));
    };

    let user_id = match find_user_id_by_bearer_token(state, &authorization).await.map_err(internal_error)? {
        Some(user_id) => user_id,
        None => {
            return Ok(json_status_response(
                StatusCode::UNAUTHORIZED,
                json!({ "message": [401200, "账号信息已过期，请重新登录"] }),
            ));
        }
    };

    let redirect = payload
        .get("redirect")
        .and_then(|v| v.as_str())
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .unwrap_or("dashboard");
    let code = Uuid::new_v4().simple().to_string();
    let cache_key = format!("TEMP_TOKEN_{}", code);
    redis_setex_string(state, &cache_key, 60, &user_id.to_string())
        .await
        .map_err(|err| {
            error!("quick login cache write failed: {}", err);
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "quick login cache write failed")
        })?;
    let login_redirect = format!("/app/#/login?verify={}&redirect={}", code, urlencoding::encode(redirect));
    let app_url = get_setting_string(state, "app_url", "").await;
    let url = if !app_url.trim().is_empty() {
        format!("{}{}", app_url.trim_end_matches('/'), login_redirect)
    } else {
        login_redirect
    };

    Ok(json_value_response(success_response_payload(Value::String(url))))
}

async fn build_token2_login_response(
    state: &AppState,
    _headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let params = parse_query(&uri);

    if let Some(token) = params.get("token").map(|v| v.trim()).filter(|v| !v.is_empty()) {
        let redirect = params.get("redirect").map(|v| v.trim()).filter(|v| !v.is_empty()).unwrap_or("dashboard");
        let login_redirect = format!("/app/#/login?verify={}&redirect={}", token, redirect);
        let app_url = get_setting_string(state, "app_url", "").await;
        let final_url = if !app_url.trim().is_empty() {
            format!("{}{}", app_url.trim_end_matches('/'), login_redirect)
        } else {
            login_redirect
        };

        return Ok(Response::builder()
            .status(StatusCode::FOUND)
            .header("location", final_url)
            .body(Body::empty())
            .unwrap());
    }

    if let Some(verify) = params.get("verify").map(|v| v.trim()).filter(|v| !v.is_empty()) {
        let cache_key = format!("TEMP_TOKEN_{}", verify);
        let user_id = redis_getdel_string(state, &cache_key)
            .await
            .map_err(|err| {
                error!("token2Login cache read failed: {}", err);
                json_error(StatusCode::INTERNAL_SERVER_ERROR, "token2Login cache read failed")
            })?
            .and_then(|value| value.parse::<i64>().ok());

        let Some(user_id) = user_id else {
            return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({ "message": "Token error" })));
        };

        let user = load_login_user_by_id(state, user_id).await.map_err(internal_error)?;
        let Some(user) = user else {
            return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({ "message": "User not found" })));
        };
        if user.banned != 0 {
            return Ok(json_status_response(StatusCode::FORBIDDEN, json!({ "message": user_suspension_message(&user) })));
        }
        if !can_use_email_login(state, user.is_super_admin != 0).await {
            return Ok(json_status_response(StatusCode::FORBIDDEN, json!({ "message": "Email login is disabled. Please use OAuth2 login" })));
        }

        let auth_data = issue_personal_access_token(state, &user).await.map_err(|err| {
            error!("token2Login issue auth failed: {}", err);
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "token2Login issue auth failed")
        })?;
        update_login_timestamp(state, user.id).await.map_err(internal_error)?;

        return Ok(json_value_response(json!({
            "data": {
                "token": user.token,
                "auth_data": auth_data,
                "is_admin": user.is_admin != 0,
            }
        })));
    }

    Ok(json_status_response(StatusCode::BAD_REQUEST, json!({ "message": "Invalid request" })))
}

async fn build_login_with_mail_link_response(
    state: &AppState,
    _headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    if env_bool("TELEGRAM_ONLY_MODE", false) {
        return Ok(fail_json_response(StatusCode::FORBIDDEN, "Email login is disabled in Telegram-only mode"));
    }
    if !get_setting_bool(state, "login_with_mail_link_enable", false).await {
        return Ok(json_status_response(StatusCode::NOT_FOUND, json!({ "message": null })));
    }

    let payload = parse_json_body(body).await?;
    let email = payload
        .get("email")
        .and_then(|v| v.as_str())
        .map(|v| v.trim().to_ascii_lowercase())
        .unwrap_or_default();
    if email.is_empty() {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "The email field is required."));
    }
    if !email.contains('@') {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "The email must be a valid email address."));
    }
    let redirect = payload.get("redirect").and_then(|v| v.as_str()).map(|v| v.trim()).filter(|v| !v.is_empty()).unwrap_or("dashboard");

    let rate_key = format!("LAST_SEND_LOGIN_WITH_MAIL_LINK_TIMESTAMP_{}", email);
    if redis_get_string(state, &rate_key).await.ok().flatten().is_some() {
        return Ok(fail_json_response(StatusCode::TOO_MANY_REQUESTS, "Sending frequently, please try again later"));
    }
    redis_setex_string(state, &rate_key, 60, &Utc::now().timestamp().to_string())
        .await
        .map_err(|err| {
            error!("loginWithMailLink rate key write failed: {}", err);
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "mail link rate cache write failed")
        })?;

    let user = load_login_user_by_email(state, &email).await.map_err(internal_error)?;
    let Some(user) = user else {
        return Ok(json_value_response(success_response_payload(Value::Bool(true))));
    };

    if user.banned != 0
        || user.is_super_admin != 0
        || !can_use_email_login(state, false).await
    {
        return Ok(json_value_response(success_response_payload(Value::Bool(true))));
    }

    let code = Uuid::new_v4().simple().to_string();
    let temp_token_key = format!("TEMP_TOKEN_{}", code);
    redis_setex_string(state, &temp_token_key, 300, &user.id.to_string())
        .await
        .map_err(|err| {
            error!("loginWithMailLink TEMP_TOKEN write failed: {}", err);
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "mail link cache write failed")
        })?;
    let redirect_url = format!(
        "/app/#/login?verify={}&redirect={}",
        code,
        urlencoding::encode(redirect),
    );
    let app_url = get_setting_string(state, "app_url", "").await;
    let link = if !app_url.trim().is_empty() {
        format!("{}{}", app_url.trim_end_matches('/'), redirect_url)
    } else {
        redirect_url
    };

    let subject = format!("Login to {}", get_setting_string(state, "app_name", "Portal").await);
    let result = send_platform_mail(
        state,
        MailSendRequest {
            email: &email,
            subject: &subject,
            template_name: "login",
            template_value: &json!({
                "name": get_setting_string(state, "app_name", "Portal").await,
                "link": link,
                "url": app_url,
            }),
        },
    )
    .await;
    let send_failed = match &result {
        Ok(result) => result.error.is_some(),
        Err(_) => true,
    };
    if send_failed {
        let _ = redis_del_key(
            state,
            &format!("{}{}{}", state.redis_prefix, state.cache_prefix, temp_token_key),
        )
        .await;
        match result {
            Ok(result) => error!("loginWithMailLink send failed: {:?}", result.error),
            Err(err) => error!("loginWithMailLink send failed: {}", err),
        }
    }

    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_forget_response(
    state: &AppState,
    _headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    if env_bool("TELEGRAM_ONLY_MODE", false) {
        return Ok(fail_json_response(StatusCode::FORBIDDEN, "Password reset by email is disabled in Telegram-only mode"));
    }

    let payload = parse_json_body(body).await?;
    let email = payload.get("email").and_then(|v| v.as_str()).map(|v| v.trim().to_string()).unwrap_or_default();
    let password = payload.get("password").and_then(|v| v.as_str()).map(|v| v.to_string()).unwrap_or_default();
    let email_code = payload.get("email_code").and_then(|v| v.as_str()).map(|v| v.trim().to_string()).unwrap_or_default();

    if email.is_empty() {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Email can not be empty"));
    }
    if !email.contains('@') {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Email format is incorrect"));
    }
    if password.is_empty() {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Password can not be empty"));
    }
    if password.chars().count() < 8 {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Password must be greater than 8 digits"));
    }
    if email_code.is_empty() {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Email verification code cannot be empty"));
    }

    let user = load_login_user_by_email(state, &email).await.map_err(internal_error)?;
    let Some(user) = user else {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "This email is not registered in the system"));
    };
    if !can_use_email_login(state, user.is_super_admin != 0).await {
        return Ok(fail_json_response(StatusCode::FORBIDDEN, "Email login is disabled. Please use OAuth2 login"));
    }

    let forget_limit_key = format!("FORGET_REQUEST_LIMIT_{}", email);
    let forget_limit = redis_get_string(state, &forget_limit_key).await.ok().flatten().and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
    if forget_limit >= 3 {
        return Ok(fail_json_response(StatusCode::TOO_MANY_REQUESTS, "Reset failed, Please try again later"));
    }

    let verify_key = format!("EMAIL_VERIFY_CODE_{}", email);
    let cached_code = redis_get_string(state, &verify_key).await.ok().flatten().unwrap_or_default();
    if cached_code != email_code {
        let next = if forget_limit > 0 { forget_limit + 1 } else { 1 };
        let _ = redis_setex_string(state, &forget_limit_key, 300, &next.to_string()).await;
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Incorrect email verification code"));
    }

    update_user_password(state, user.id, &password).await.map_err(internal_error)?;
    let _ = redis_del_key(state, &format!("{}{}{}", state.redis_prefix, state.cache_prefix, verify_key)).await;
    let _ = redis_del_key(state, &format!("{}{}{}", state.redis_prefix, state.cache_prefix, forget_limit_key)).await;

    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_register_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let payload = parse_json_body(body).await?;
    let email = payload.get("email").and_then(|v| v.as_str()).map(|v| v.trim().to_string()).unwrap_or_default();
    let password = payload.get("password").and_then(|v| v.as_str()).map(|v| v.to_string()).unwrap_or_default();
    let invite_code = payload.get("invite_code").and_then(|v| v.as_str()).map(|v| v.trim().to_string()).unwrap_or_default();
    let email_code = payload.get("email_code").and_then(|v| v.as_str()).map(|v| v.trim().to_string()).unwrap_or_default();

    if email.is_empty() {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Email can not be empty"));
    }
    if !email.contains('@') {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Email format is incorrect"));
    }
    if password.is_empty() {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Password can not be empty"));
    }
    if password.chars().count() < 8 {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Password must be greater than 8 digits"));
    }

    let register_mode = resolve_register_mode(state).await;
    if !matches!(register_mode.as_str(), "all" | "email_only") {
        return Ok(fail_json_response(StatusCode::FORBIDDEN, "Registration by email is disabled"));
    }

    if get_setting_bool(state, "register_limit_by_ip_enable", false).await {
        let ip = request_ip(&headers).unwrap_or_else(|| "127.0.0.1".to_string());
        let rate_key = format!("REGISTER_IP_RATE_LIMIT_{}", ip);
        let count = redis_get_string(state, &rate_key).await.ok().flatten().and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
        let limit = get_setting_int(state, "register_limit_count", 3).await.max(1);
        if count >= limit {
            let expire = get_setting_int(state, "register_limit_expire", 60).await.max(1);
            return Ok(fail_json_response(
                StatusCode::TOO_MANY_REQUESTS,
                &format!("Register frequently, please try again after {} minute", expire),
            ));
        }
    }

    if !verify_captcha_disabled_or_supported(state).await {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Invalid captcha type"));
    }
    let pow_valid = verify_pow_payload(state, &headers, &payload).await?;
    if !pow_valid {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Pow verification failed"));
    }

    if get_setting_bool(state, "email_whitelist_enable", false).await {
        let whitelist = setting_json_or_csv_array(state, "email_whitelist_suffix", &[
            "gmail.com", "qq.com", "163.com", "yahoo.com", "sina.com", "126.com", "outlook.com", "yeah.net", "foxmail.com",
        ]).await;
        let suffix_ok = whitelist
            .as_array()
            .map(|items| items.iter().filter_map(|v| v.as_str()).any(|suffix| email.ends_with(&format!("@{}", suffix))))
            .unwrap_or(false);
        if !suffix_ok {
            return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Email suffix is not in the Whitelist"));
        }
    }

    if get_setting_bool(state, "email_gmail_limit_enable", false).await {
        if let Some(prefix) = email.split('@').next() {
            if prefix.contains('.') || prefix.contains('+') {
                return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Gmail alias is not supported"));
            }
        }
    }

    if get_setting_bool(state, "invite_force", false).await && invite_code.is_empty() {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "You must use the invitation code to register"));
    }

    if get_setting_bool(state, "email_verify", false).await {
        if email_code.is_empty() {
            return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Email verification code cannot be empty"));
        }
        let verify_key = format!("EMAIL_VERIFY_CODE_{}", email);
        let cached_code = redis_get_string(state, &verify_key).await.ok().flatten().unwrap_or_default();
        if cached_code != email_code {
            return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Incorrect email verification code"));
        }
    }

    if load_login_user_by_email(state, &email).await.map_err(internal_error)?.is_some() {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Email already exists"));
    }

    let invite = resolve_invite_code_for_registration(state, &invite_code).await.map_err(internal_error)?;
    let invite_user_id = invite.as_ref().map(|value| value.user_id);
    let user = create_registered_user(state, &email, &password, invite_user_id).await.map_err(internal_error)?;
    consume_invite_code(state, invite.as_ref()).await.map_err(internal_error)?;

    if get_setting_bool(state, "email_verify", false).await {
        let _ = redis_del_key(state, &format!("{}{}EMAIL_VERIFY_CODE_{}", state.redis_prefix, state.cache_prefix, email)).await;
    }

    if get_setting_bool(state, "register_limit_by_ip_enable", false).await {
        let ip = request_ip(&headers).unwrap_or_else(|| "127.0.0.1".to_string());
        let rate_key = format!("REGISTER_IP_RATE_LIMIT_{}", ip);
        let count = redis_get_string(state, &rate_key).await.ok().flatten().and_then(|v| v.parse::<i64>().ok()).unwrap_or(0) + 1;
        let expire = get_setting_int(state, "register_limit_expire", 60).await.max(1);
        let _ = redis_setex_string(state, &rate_key, expire * 60, &count.to_string()).await;
    }

    let auth_data = issue_personal_access_token(state, &user).await.map_err(|err| {
        error!("register issue auth failed: {}", err);
        json_error(StatusCode::INTERNAL_SERVER_ERROR, "register issue auth failed")
    })?;

    Ok(json_value_response(success_response_payload(json!({
        "token": user.token,
        "auth_data": auth_data,
        "is_admin": user.is_admin != 0,
    }))))
}
