use crate::*;
use crate::mail_support::{send_platform_mail, MailSendRequest};

pub async fn send_email_verify(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_send_email_verify_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn pv(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_pv_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_send_email_verify_response(
    state: &AppState,
    _headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    if env_bool("TELEGRAM_ONLY_MODE", false) {
        return Ok(fail_json_response(StatusCode::FORBIDDEN, "Email verification is disabled in Telegram-only mode"));
    }

    let payload = parse_json_body(body).await?;
    let email = payload.get("email").and_then(|v| v.as_str()).map(|v| v.trim().to_string()).unwrap_or_default();
    if email.is_empty() {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Email can not be empty"));
    }
    if !email.contains('@') {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Email format is incorrect"));
    }

    if !verify_captcha_disabled_or_supported(state).await {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Invalid captcha type"));
    }

    if get_setting_bool(state, "email_whitelist_enable", false).await {
        let is_registered = load_login_user_by_email(state, &email).await.map_err(internal_error)?.is_some();
        if !is_registered {
            let whitelist = setting_json_or_csv_array(state, "email_whitelist_suffix", &[
                "gmail.com", "qq.com", "163.com", "yahoo.com", "sina.com", "126.com", "outlook.com", "yeah.net", "foxmail.com",
            ]).await;
            let suffix_ok = whitelist
                .as_array()
                .map(|items| items.iter().filter_map(|v| v.as_str()).any(|suffix| email.ends_with(&format!("@{}", suffix))))
                .unwrap_or(false);
            if !suffix_ok {
                return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Email suffix is not in whitelist"));
            }
        }
    }

    let cooldown_key = format!("LAST_SEND_EMAIL_VERIFY_TIMESTAMP_{}", email);
    if redis_get_string(state, &cooldown_key).await.ok().flatten().is_some() {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "Email verification code has been sent, please request again later"));
    }

    let code = random_digits(6);
    let app_name = get_setting_string(state, "app_name", "Portal").await;
    let app_url = get_setting_string(state, "app_url", "").await;
    let subject = format!("{}Email verification code", app_name);
    let result = send_platform_mail(
        state,
        MailSendRequest {
            email: &email,
            subject: &subject,
            template_name: "verify",
            template_value: &json!({
                "name": app_name,
                "url": app_url,
                "code": code,
            }),
        },
    )
    .await
    .map_err(|err| {
        error!("sendEmailVerify send failed: {}", err);
        json_error(StatusCode::INTERNAL_SERVER_ERROR, "sendEmailVerify send failed")
    })?;
    if result.error.is_some() {
        return Ok(fail_json_response(StatusCode::INTERNAL_SERVER_ERROR, "Email send failed"));
    }

    redis_setex_string(state, &format!("EMAIL_VERIFY_CODE_{}", email), 300, &code)
        .await
        .map_err(|err| {
            error!("EMAIL_VERIFY_CODE write failed: {}", err);
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "EMAIL_VERIFY_CODE write failed")
        })?;
    redis_setex_string(state, &cooldown_key, 60, &Utc::now().timestamp().to_string())
        .await
        .map_err(|err| {
            error!("LAST_SEND_EMAIL_VERIFY_TIMESTAMP write failed: {}", err);
            json_error(StatusCode::INTERNAL_SERVER_ERROR, "LAST_SEND_EMAIL_VERIFY_TIMESTAMP write failed")
        })?;

    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_pv_response(
    state: &AppState,
    _headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let mut invite_code = parse_query(&uri)
        .get("invite_code")
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    if invite_code.is_none() {
        let payload = parse_json_body(body).await?;
        invite_code = payload
            .get("invite_code")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToString::to_string);
    }

    if let Some(invite_code) = invite_code {
        sqlx::query("UPDATE v2_invite_code SET pv = COALESCE(pv, 0) + 1, updated_at = ? WHERE code = ?")
            .bind(Utc::now().timestamp())
            .bind(invite_code)
            .execute(&state.db)
            .await
            .map_err(internal_error)?;
    }

    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}
