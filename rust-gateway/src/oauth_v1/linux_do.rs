use crate::*;
use hyper_util::rt::TokioExecutor;

#[derive(Deserialize)]
pub(crate) struct LinuxDoTokenResponse {
    pub(crate) access_token: String,
    #[serde(default)]
    pub(crate) refresh_token: Option<String>,
    #[serde(default = "default_oauth_expire")]
    pub(crate) expires_in: i64,
    #[serde(default)]
    pub(crate) token_type: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct LinuxDoUserInfo {
    pub(crate) id: i64,
    pub(crate) username: String,
    pub(crate) name: String,
    pub(crate) avatar_template: String,
    pub(crate) active: bool,
    pub(crate) trust_level: i64,
    #[serde(default)]
    pub(crate) silenced: bool,
    #[serde(default)]
    pub(crate) external_ids: Value,
}

pub async fn redirect(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_redirect_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn callback(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_callback_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn refresh(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
) -> Response<Body> {
    match build_refresh_response(&state, headers).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn sync(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
) -> Response<Body> {
    match build_sync_response(&state, headers).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_redirect_response(
    state: &AppState,
    _headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let oauth = resolve_linux_do_oauth_config(state).await?;
    let redirect_uri = oauth_redirect_uri(state).await;
    let state_value = random_alnum(32);
    let invite_code = parse_query(&uri)
        .get("invite_code")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let payload = json!({
        "valid": true,
        "invite_code": invite_code,
    });
    redis_setex_string(
        state,
        &oauth_state_cache_key(&state_value),
        600,
        &payload.to_string(),
    )
    .await
    .map_err(|err| json_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("oauth state cache failed: {err}")))?;

    let location = format!(
        "{}?{}",
        env::var("LINUX_DO_AUTHORIZE_URL")
            .unwrap_or_else(|_| "https://connect.linux.do/oauth2/authorize".to_string()),
        serde_urlencoded::to_string([
            ("client_id", oauth.client_id.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("response_type", "code"),
            ("scope", "read"),
            ("state", state_value.as_str()),
        ])
        .unwrap_or_default()
    );

    Ok(Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", location)
        .body(Body::empty())
        .unwrap())
}

async fn build_callback_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let params = parse_query(&uri);
    let code = params
        .get("code")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| oauth_json_or_html_error(&headers, StatusCode::BAD_REQUEST, "Missing code parameter"))?;
    let state_value = params
        .get("state")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| oauth_json_or_html_error(&headers, StatusCode::BAD_REQUEST, "Missing state parameter"))?;

    let raw_state = redis_get_string(state, &oauth_state_cache_key(&state_value))
        .await
        .map_err(|err| json_error(StatusCode::INTERNAL_SERVER_ERROR, &format!("oauth state read failed: {err}")))?;
    let _ = redis_del_key(state, &format!("{}{}{}", state.redis_prefix, state.cache_prefix, oauth_state_cache_key(&state_value))).await;
    let state_payload = raw_state
        .and_then(|value| serde_json::from_str::<Value>(&value).ok())
        .and_then(|value| value.as_object().cloned())
        .ok_or_else(|| oauth_json_or_html_error(&headers, StatusCode::UNAUTHORIZED, "Invalid state parameter"))?;
    if !state_payload.get("valid").and_then(Value::as_bool).unwrap_or(false) {
        return Err(oauth_json_or_html_error(&headers, StatusCode::UNAUTHORIZED, "Invalid state parameter"));
    }

    let oauth = resolve_linux_do_oauth_config(state).await?;
    let redirect_uri = oauth_redirect_uri(state).await;
    let token_data = exchange_linux_do_code(&oauth, &code, &redirect_uri).await?;
    let user_info = fetch_linux_do_user_info(&token_data.access_token).await?;
    let invite_code = state_payload
        .get("invite_code")
        .and_then(Value::as_str)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let login_user = sync_linux_do_user(state, &user_info, &token_data, invite_code.as_deref()).await?;
    if login_user.banned != 0 {
        return Err(oauth_json_or_html_error(&headers, StatusCode::FORBIDDEN, &user_suspension_message(&login_user)));
    }

    let auth_data = issue_personal_access_token(state, &login_user)
        .await
        .map_err(|err| {
            error!("oauth issue auth failed: {}", err);
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "oauth issue auth failed")
        })?;
    update_login_timestamp(state, login_user.id)
        .await
        .map_err(internal_error)?;

    let user = load_bearer_user_by_id(state, login_user.id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "The user does not exist"))?;

    let payload = json!({
        "success": true,
        "message": "Login successful",
        "data": {
            "user": {
                "id": user.id,
                "email": user.email,
                "linux_do_username": user.linux_do_username,
                "linux_do_name": user.linux_do_name,
                "linux_do_avatar": user.linux_do_avatar,
                "trust_level": user.trust_level,
                "is_admin": user.is_admin != 0,
                "is_super_admin": user.is_super_admin != 0,
                "api_key": user.api_key,
            },
            "auth": {
                "token": user.token,
                "auth_data": auth_data,
                "is_admin": user.is_admin != 0,
            },
            "provider": "linux_do"
        }
    });

    let accepts_html = headers
        .get("accept")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.contains("text/html") && !value.contains("application/json"))
        .unwrap_or(false);
    if accepts_html {
        return Ok(oauth_callback_html_response(&payload));
    }
    Ok(json_status_response(StatusCode::OK, payload))
}

pub(crate) struct LinuxDoOauthConfig {
    pub(crate) client_id: String,
    pub(crate) client_secret: String,
}

pub(crate) async fn resolve_linux_do_oauth_config(
    state: &AppState,
) -> Result<LinuxDoOauthConfig, Response<Body>> {
    if !resolve_oauth_linux_do_available(state).await {
        return Err(json_error(StatusCode::INTERNAL_SERVER_ERROR, "Linux DO 登录未启用或未完成配置"));
    }
    let client_id = first_non_empty(&[
        get_setting_string(state, "oauth_linux_do_client_id", "").await,
        env::var("LINUX_DO_CLIENT_ID").unwrap_or_default(),
    ]);
    let client_secret = first_non_empty(&[
        get_setting_string(state, "oauth_linux_do_client_secret", "").await,
        env::var("LINUX_DO_CLIENT_SECRET").unwrap_or_default(),
    ]);
    Ok(LinuxDoOauthConfig {
        client_id,
        client_secret,
    })
}

async fn oauth_redirect_uri(state: &AppState) -> String {
    let configured = get_setting_string(state, "oauth_linux_do_redirect_uri", "").await;
    if !configured.trim().is_empty() {
        configured
    } else {
        let app_url = get_setting_string(state, "app_url", &env::var("APP_URL").unwrap_or_default()).await;
        format!("{}/api/v1/passport/oauth2/linux-do/callback", app_url.trim_end_matches('/'))
    }
}

fn oauth_state_cache_key(state: &str) -> String {
    format!("oauth:linux_do:state:{}", state)
}

async fn exchange_linux_do_code(
    oauth: &LinuxDoOauthConfig,
    code: &str,
    redirect_uri: &str,
) -> Result<LinuxDoTokenResponse, Response<Body>> {
    let token_url = env::var("LINUX_DO_TOKEN_URL")
        .unwrap_or_else(|_| "https://connect.linux.do/oauth2/token".to_string());
    let uri: Uri = token_url
        .parse()
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("invalid oauth token uri: {err}")))?;
    let request = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(
            serde_urlencoded::to_string([
                ("client_id", oauth.client_id.as_str()),
                ("client_secret", oauth.client_secret.as_str()),
                ("code", code),
                ("grant_type", "authorization_code"),
                ("redirect_uri", redirect_uri),
            ])
            .unwrap_or_default(),
        ))
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("build oauth token request failed: {err}")))?;
    let client = Client::builder(TokioExecutor::new()).build_http();
    let response = client
        .request(request)
        .await
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("oauth token request failed: {err}")))?;
    let status = response.status();
    let bytes = response_body_bytes(map_proxy_response(response).await).await;
    if !status.is_success() {
        return Err(json_status_response(
            StatusCode::BAD_GATEWAY,
            json!({"success": false, "error": format!("Token exchange failed: HTTP {}", status.as_u16())}),
        ));
    }
    serde_json::from_slice::<LinuxDoTokenResponse>(&bytes)
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("invalid oauth token response: {err}")))
}

pub(crate) async fn fetch_linux_do_user_info(
    access_token: &str,
) -> Result<LinuxDoUserInfo, Response<Body>> {
    let user_url = env::var("LINUX_DO_USER_INFO_URL")
        .unwrap_or_else(|_| "https://connect.linux.do/api/user".to_string());
    let uri: Uri = user_url
        .parse()
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("invalid oauth user uri: {err}")))?;
    let request = Request::builder()
        .method(Method::GET)
        .uri(uri)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Accept", "application/json")
        .body(Body::empty())
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("build oauth user request failed: {err}")))?;
    let client = Client::builder(TokioExecutor::new()).build_http();
    let response = client
        .request(request)
        .await
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("oauth user request failed: {err}")))?;
    let status = response.status();
    let bytes = response_body_bytes(map_proxy_response(response).await).await;
    if !status.is_success() {
        return Err(json_status_response(
            StatusCode::BAD_GATEWAY,
            json!({"success": false, "error": format!("Failed to get user info: HTTP {}", status.as_u16())}),
        ));
    }
    serde_json::from_slice::<LinuxDoUserInfo>(&bytes)
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("invalid oauth user response: {err}")))
}

async fn sync_linux_do_user(
    state: &AppState,
    user_info: &LinuxDoUserInfo,
    token_data: &LinuxDoTokenResponse,
    invite_code: Option<&str>,
) -> Result<LoginUserRow, Response<Body>> {
    let existing = load_login_user_by_linux_do_id(state, user_info.id).await.map_err(internal_error)?;
    if existing.is_none() && !allows_oauth_registration(state).await {
        return Err(json_status_response(
            StatusCode::FORBIDDEN,
            json!({"success": false, "error": "OAuth2 registration is disabled", "message": "OAuth2 registration is disabled"}),
        ));
    }
    if existing.is_none() && get_setting_bool(state, "invite_force", false).await && invite_code.unwrap_or("").trim().is_empty() {
        return Err(json_status_response(
            StatusCode::FORBIDDEN,
            json!({"success": false, "error": "You must use the invitation code to register", "message": "You must use the invitation code to register"}),
        ));
    }

    let invite = resolve_invite_code_for_registration(state, invite_code.unwrap_or("")).await.map_err(internal_error)?;
    let invite_user_id = invite.as_ref().map(|value| value.user_id);

    let user_id = if let Some(existing) = existing {
        update_linux_do_existing_user(state, existing.id, user_info, token_data).await.map_err(internal_error)?;
        existing.id
    } else {
        let email = unique_linux_do_email(state, &user_info.username).await.map_err(internal_error)?;
        let user_id = create_linux_do_user(state, &email, user_info, token_data, invite_user_id)
            .await
            .map_err(internal_error)?;
        consume_invite_code(state, invite.as_ref()).await.map_err(internal_error)?;
        user_id
    };

    load_login_user_by_id(state, user_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "The user does not exist"))
}

async fn allows_oauth_registration(state: &AppState) -> bool {
    if !resolve_oauth_linux_do_available(state).await {
        return false;
    }
    let mode = resolve_register_mode(state).await;
    mode != "closed" && matches!(mode.as_str(), "all" | "oauth_only")
}

async fn load_login_user_by_linux_do_id(
    state: &AppState,
    linux_do_id: i64,
) -> Result<Option<LoginUserRow>, sqlx::Error> {
    sqlx::query_as::<_, LoginUserRow>(
        "SELECT id, email, password, password_algo, password_salt, banned, ban_reason, token, is_admin, is_super_admin, last_login_at
         FROM v2_user
         WHERE linux_do_id = ?
         LIMIT 1",
    )
    .bind(linux_do_id.to_string())
    .fetch_optional(&state.db)
    .await
}

async fn unique_linux_do_email(state: &AppState, username: &str) -> Result<String, sqlx::Error> {
    let base = format!("{}@linux.do", username);
    let mut email = base.clone();
    let mut counter = 1;
    while load_login_user_by_email(state, &email).await?.is_some() {
        email = format!("{}+{}@linux.do", username, counter);
        counter += 1;
    }
    Ok(email)
}

async fn create_linux_do_user(
    state: &AppState,
    email: &str,
    user_info: &LinuxDoUserInfo,
    token_data: &LinuxDoTokenResponse,
    invite_user_id: Option<i64>,
) -> Result<i64, sqlx::Error> {
    let hashed = bcrypt::hash(&random_alnum(32), 12).map_err(|_| sqlx::Error::Protocol("bcrypt hash failed".into()))?;
    let now = Utc::now().timestamp();
    let token = random_hex(32);
    let uuid = random_uuid_string();
    let subscribe_path = random_letters(10);
    let subscribe_key = random_letters(8);
    let mut subscribe_salt = random_letters(6);
    if subscribe_salt == subscribe_key {
        subscribe_salt = random_letters(6);
    }
    let remind_expire = get_setting_int(state, "default_remind_expire", 1).await;
    let remind_traffic = get_setting_int(state, "default_remind_traffic", 1).await;
    let device_limit = default_linux_do_device_limit(user_info.trust_level);
    let external_ids = serde_json::to_string(&user_info.external_ids).unwrap_or_else(|_| "null".to_string());
    let oauth_expires_at = chrono::DateTime::<Utc>::from_timestamp(now + token_data.expires_in.max(60), 0)
        .map(|dt| dt.naive_utc());

    let result = sqlx::query(
        "INSERT INTO v2_user (
            invite_user_id, email, password, uuid, token, subscribe_path, subscribe_key, subscribe_salt,
            remind_expire, remind_traffic, expired_at, concurrent_ip_limit, last_login_at,
            linux_do_id, linux_do_username, linux_do_name, linux_do_avatar, trust_level, is_silenced, external_ids,
            oauth_provider, oauth_access_token, oauth_refresh_token, oauth_expires_at, banned, commission_rate, commission_type,
            device_limit, created_at, updated_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 3, ?, ?, ?, ?, ?, ?, ?, ?, 'linux_do', ?, ?, ?, ?, 0.1, 0, ?, ?, ?)",
    )
    .bind(invite_user_id)
    .bind(email)
    .bind(hashed)
    .bind(uuid)
    .bind(token)
    .bind(subscribe_path)
    .bind(subscribe_key)
    .bind(subscribe_salt)
    .bind(remind_expire)
    .bind(remind_traffic)
    .bind(now)
    .bind(user_info.id.to_string())
    .bind(&user_info.username)
    .bind(&user_info.name)
    .bind(&user_info.avatar_template)
    .bind(user_info.trust_level)
    .bind(if user_info.silenced { 1 } else { 0 })
    .bind(external_ids)
    .bind(&token_data.access_token)
    .bind(token_data.refresh_token.clone())
    .bind(oauth_expires_at)
    .bind(if user_info.active { 0 } else { 1 })
    .bind(device_limit)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await?;

    Ok(result.last_insert_id() as i64)
}

async fn update_linux_do_existing_user(
    state: &AppState,
    user_id: i64,
    user_info: &LinuxDoUserInfo,
    token_data: &LinuxDoTokenResponse,
) -> Result<(), sqlx::Error> {
    let now = Utc::now().timestamp();
    let external_ids = serde_json::to_string(&user_info.external_ids).unwrap_or_else(|_| "null".to_string());
    let oauth_expires_at = chrono::DateTime::<Utc>::from_timestamp(now + token_data.expires_in.max(60), 0)
        .map(|dt| dt.naive_utc());

    sqlx::query(
        "UPDATE v2_user
         SET linux_do_username = ?, linux_do_name = ?, linux_do_avatar = ?, trust_level = ?, is_silenced = ?, external_ids = ?,
             oauth_provider = 'linux_do', oauth_access_token = ?, oauth_refresh_token = ?, oauth_expires_at = ?, banned = ?, last_login_at = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(&user_info.username)
    .bind(&user_info.name)
    .bind(&user_info.avatar_template)
    .bind(user_info.trust_level)
    .bind(if user_info.silenced { 1 } else { 0 })
    .bind(external_ids)
    .bind(&token_data.access_token)
    .bind(token_data.refresh_token.clone())
    .bind(oauth_expires_at)
    .bind(if user_info.active { 0 } else { 1 })
    .bind(now)
    .bind(now)
    .bind(user_id)
    .execute(&state.db)
    .await?;
    Ok(())
}

fn default_linux_do_device_limit(trust_level: i64) -> i64 {
    match trust_level {
        0 => 2,
        1 => 3,
        2 => 5,
        3 => 8,
        4 => 10,
        _ => 2,
    }
}

fn oauth_json_or_html_error(headers: &HeaderMap, status: StatusCode, message: &str) -> Response<Body> {
    let payload = json!({
        "success": false,
        "error": message,
        "message": message,
    });
    let accepts_html = headers
        .get("accept")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.contains("text/html") && !value.contains("application/json"))
        .unwrap_or(false);
    if accepts_html {
        oauth_callback_html_response(&payload)
    } else {
        json_status_response(status, payload)
    }
}

fn oauth_callback_html_response(payload: &Value) -> Response<Body> {
    let html_payload = payload.to_string().replace("</script>", "<\\/script>");
    let html = format!(
        r#"<!doctype html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Linux DO 登录回调</title>
  <style>
    body {{ font-family: system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, sans-serif; padding: 48px; }}
    .card {{ max-width: 520px; margin: 0 auto; border: 1px solid #e5e7eb; border-radius: 12px; padding: 24px; }}
    .muted {{ color: #6b7280; font-size: 14px; margin-top: 12px; }}
    .ok {{ color: #059669; }}
    .bad {{ color: #dc2626; }}
  </style>
</head>
<body>
  <div class="card">
    <h1>Linux DO 登录回调</h1>
    <div id="msg" class="muted">正在写入登录态并跳转...</div>
    <div class="muted">如果没有自动跳转，请点击：<a id="go" href="/">返回首页</a></div>
  </div>
  <script>
    (function () {{
      const payload = {html_payload};
      const msg = document.getElementById('msg');
      try {{
        if (!payload || payload.success === false) {{
          msg.className = 'muted bad';
          msg.textContent = payload?.error || payload?.message || '登录回调失败，请返回重试。';
          document.getElementById('go').href = '/app/#/login';
          return;
        }}
        if (!payload.data || !payload.data.auth || !payload.data.auth.auth_data) {{
          msg.className = 'muted bad';
          msg.textContent = '登录回调数据不完整，请重试。';
          return;
        }}
        localStorage.setItem('auth_data', payload.data.auth.auth_data);
        localStorage.setItem('token', payload.data.auth.token || '');
        localStorage.setItem('me', JSON.stringify({{
          id: payload.data.user?.id,
          email: payload.data.user?.email,
          is_admin: !!payload.data.user?.is_admin,
          is_super_admin: !!payload.data.user?.is_super_admin,
          trust_level: payload.data.user?.trust_level ?? 0,
          is_linux_do_user: true,
          linux_do_username: payload.data.user?.linux_do_username ?? null,
          linux_do_name: payload.data.user?.linux_do_name ?? null,
          linux_do_avatar: payload.data.user?.linux_do_avatar ?? null,
          api_key: payload.data.user?.api_key ?? null
        }}));
        msg.className = 'muted ok';
        msg.textContent = '登录成功，正在跳转...';
        window.location.href = '/app/#/dashboard';
      }} catch (e) {{
        msg.className = 'muted bad';
        msg.textContent = '写入登录态失败：' + (e?.message || e);
      }}
    }})();
  </script>
</body>
</html>"#
    );

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(html))
        .unwrap()
}

fn default_oauth_expire() -> i64 {
    3600
}

async fn build_refresh_response(
    state: &AppState,
    headers: HeaderMap,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let oauth_user = load_oauth_linux_do_user_by_id(state, user.id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| json_status_response(
            StatusCode::UNAUTHORIZED,
            json!({"success": false, "error": "User not authenticated with Linux DO"})
        ))?;
    if oauth_user.oauth_provider.as_deref() != Some("linux_do") || oauth_user.linux_do_id.is_none() {
        return Err(json_status_response(
            StatusCode::UNAUTHORIZED,
            json!({"success": false, "error": "User not authenticated with Linux DO"}),
        ));
    }
    let refresh_token = oauth_user
        .oauth_refresh_token
        .clone()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| json_status_response(
            StatusCode::UNAUTHORIZED,
            json!({"success": false, "error": "No refresh token available"}),
        ))?;

    let oauth = resolve_linux_do_oauth_config(state).await?;
    let token_data = refresh_linux_do_token(&oauth, &refresh_token).await?;
    persist_linux_do_tokens(state, oauth_user.id, &token_data)
        .await
        .map_err(internal_error)?;

    let login_user = load_login_user_by_id(state, oauth_user.id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "The user does not exist"))?;
    let auth_data = issue_personal_access_token(state, &login_user)
        .await
        .map_err(|err| {
            error!("oauth refresh issue auth failed: {}", err);
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "oauth refresh issue auth failed")
        })?;
    let refreshed = load_oauth_linux_do_user_by_id(state, oauth_user.id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "The user does not exist"))?;
    let expires_at = refreshed
        .oauth_expires_at
        .map(|value| value.format("%Y-%m-%dT%H:%M:%SZ").to_string());

    Ok(json_status_response(StatusCode::OK, json!({
        "success": true,
        "message": "Token refreshed successfully",
        "data": {
            "auth": {
                "token": refreshed.token,
                "auth_data": auth_data,
                "is_admin": refreshed.is_admin != 0,
            },
            "expires_at": expires_at,
        }
    })))
}

async fn build_sync_response(
    state: &AppState,
    headers: HeaderMap,
) -> Result<Response<Body>, Response<Body>> {
    let user = authenticate_bearer_user(state, &headers).await?;
    let oauth_user = load_oauth_linux_do_user_by_id(state, user.id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| json_status_response(
            StatusCode::UNAUTHORIZED,
            json!({"success": false, "error": "User not authenticated with Linux DO"})
        ))?;
    if oauth_user.oauth_provider.as_deref() != Some("linux_do") || oauth_user.linux_do_id.is_none() {
        return Err(json_status_response(
            StatusCode::UNAUTHORIZED,
            json!({"success": false, "error": "User not authenticated with Linux DO"}),
        ));
    }

    let oauth = resolve_linux_do_oauth_config(state).await?;
    let mut access_token = oauth_user.oauth_access_token.clone().unwrap_or_default();
    if access_token.trim().is_empty() {
        return Err(json_status_response(
            StatusCode::UNAUTHORIZED,
            json!({"success": false, "error": "Failed to sync user info"}),
        ));
    }

    let now = Utc::now();
    if oauth_user
        .oauth_expires_at
        .map(|value| value <= now)
        .unwrap_or(false)
    {
        let refresh_token = oauth_user
            .oauth_refresh_token
            .clone()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| json_status_response(
                StatusCode::UNAUTHORIZED,
                json!({"success": false, "error": "Failed to refresh token"}),
            ))?;
        let token_data = refresh_linux_do_token(&oauth, &refresh_token).await?;
        persist_linux_do_tokens(state, oauth_user.id, &token_data)
            .await
            .map_err(internal_error)?;
        access_token = token_data.access_token.clone();
    }

    let user_info = fetch_linux_do_user_info(&access_token).await?;
    update_linux_do_existing_user(
        state,
        oauth_user.id,
        &user_info,
        &LinuxDoTokenResponse {
            access_token,
            refresh_token: oauth_user.oauth_refresh_token.clone(),
            expires_in: oauth_user
                .oauth_expires_at
                .map(|value| (value.timestamp() - Utc::now().timestamp()).max(60))
                .unwrap_or(3600),
            token_type: Some("Bearer".to_string()),
        },
    )
    .await
    .map_err(internal_error)?;

    let refreshed = load_oauth_linux_do_user_by_id(state, oauth_user.id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "The user does not exist"))?;
    let updated_at = refreshed
        .updated_at
        .and_then(|value| chrono::DateTime::from_timestamp(value, 0))
        .map(|value| value.format("%Y-%m-%dT%H:%M:%SZ").to_string());

    Ok(json_status_response(StatusCode::OK, json!({
        "success": true,
        "message": "User info synced successfully",
        "data": {
            "user": {
                "id": refreshed.id,
                "linux_do_username": refreshed.linux_do_username,
                "linux_do_name": refreshed.linux_do_name,
                "linux_do_avatar": refreshed.linux_do_avatar,
                "trust_level": refreshed.trust_level,
                "is_silenced": refreshed.is_silenced != 0,
                "updated_at": updated_at,
            }
        }
    })))
}

#[derive(Clone, sqlx::FromRow)]
struct LinuxDoOauthUserRow {
    id: i64,
    email: String,
    token: String,
    is_admin: i8,
    is_super_admin: i8,
    trust_level: i64,
    is_silenced: i8,
    linux_do_id: Option<String>,
    linux_do_username: Option<String>,
    linux_do_name: Option<String>,
    linux_do_avatar: Option<String>,
    oauth_provider: Option<String>,
    oauth_access_token: Option<String>,
    oauth_refresh_token: Option<String>,
    oauth_expires_at: Option<chrono::DateTime<Utc>>,
    updated_at: Option<i64>,
}

async fn load_oauth_linux_do_user_by_id(
    state: &AppState,
    user_id: i64,
) -> Result<Option<LinuxDoOauthUserRow>, sqlx::Error> {
    sqlx::query_as::<_, LinuxDoOauthUserRow>(
        "SELECT id, email, token, is_admin, is_super_admin, trust_level, is_silenced,
                linux_do_id, linux_do_username, linux_do_name, linux_do_avatar,
                oauth_provider, oauth_access_token, oauth_refresh_token, oauth_expires_at, updated_at
         FROM v2_user WHERE id = ? LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn refresh_linux_do_token(
    oauth: &LinuxDoOauthConfig,
    refresh_token: &str,
) -> Result<LinuxDoTokenResponse, Response<Body>> {
    let token_url = env::var("LINUX_DO_TOKEN_URL")
        .unwrap_or_else(|_| "https://connect.linux.do/oauth2/token".to_string());
    let uri: Uri = token_url
        .parse()
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("invalid oauth token uri: {err}")))?;
    let request = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(
            serde_urlencoded::to_string([
                ("client_id", oauth.client_id.as_str()),
                ("client_secret", oauth.client_secret.as_str()),
                ("refresh_token", refresh_token),
                ("grant_type", "refresh_token"),
            ])
            .unwrap_or_default(),
        ))
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("build oauth refresh request failed: {err}")))?;
    let client = Client::builder(TokioExecutor::new()).build_http();
    let response = client
        .request(request)
        .await
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("oauth refresh request failed: {err}")))?;
    let status = response.status();
    let bytes = response_body_bytes(map_proxy_response(response).await).await;
    if !status.is_success() {
        return Err(json_status_response(
            StatusCode::UNAUTHORIZED,
            json!({"success": false, "error": format!("Failed to refresh token: HTTP {}", status.as_u16())}),
        ));
    }
    serde_json::from_slice::<LinuxDoTokenResponse>(&bytes)
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("invalid oauth refresh response: {err}")))
}

async fn persist_linux_do_tokens(
    state: &AppState,
    user_id: i64,
    token_data: &LinuxDoTokenResponse,
) -> Result<(), sqlx::Error> {
    let now = Utc::now().timestamp();
    let oauth_expires_at = chrono::DateTime::<Utc>::from_timestamp(now + token_data.expires_in.max(60), 0)
        .map(|dt| dt.naive_utc());
    sqlx::query(
        "UPDATE v2_user
         SET oauth_access_token = ?, oauth_refresh_token = ?, oauth_expires_at = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(&token_data.access_token)
    .bind(token_data.refresh_token.clone())
    .bind(oauth_expires_at)
    .bind(now)
    .bind(user_id)
    .execute(&state.db)
    .await?;
    Ok(())
}
