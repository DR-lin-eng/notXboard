use crate::*;

pub async fn show(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_show_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn upsert(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_upsert_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_show_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let url = get_setting_string(state, "sponsor_epay_url", "").await;
    let pid = get_setting_string(state, "sponsor_epay_pid", "").await;
    let key = get_setting_string(state, "sponsor_epay_key", "").await;
    let submit_path = get_setting_string(state, "sponsor_epay_submit_path", "/pay/submit.php").await;
    let use_post = get_setting_bool(state, "sponsor_epay_use_post", true).await;
    let sitename = get_setting_string(state, "sponsor_epay_sitename", "").await;
    let device = get_setting_string(state, "sponsor_epay_device", "").await;

    Ok(json_value_response(json!({
        "success": true,
        "data": {
            "url": url,
            "pid": pid,
            "submit_path": submit_path,
            "use_post": use_post,
            "sitename": sitename,
            "device": device,
            "has_key": !key.trim().is_empty(),
        }
    })))
}

async fn build_upsert_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let obj = payload.as_object().ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let url = obj.get("url").and_then(|value| value.as_str()).map(|v| v.trim()).filter(|v| !v.is_empty()).ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    let pid = obj.get("pid").and_then(|value| value.as_str()).map(|v| v.trim()).filter(|v| !v.is_empty()).ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    let key = obj.get("key").and_then(|value| value.as_str()).map(|v| v.trim()).filter(|v| !v.is_empty()).ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    let url = crate::url_security_support::normalize_http_url(url, false)
        .map_err(|message| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, &message))?;
    let submit_path = crate::url_security_support::normalize_relative_path(
        obj.get("submit_path").and_then(|value| value.as_str()),
        "/pay/submit.php",
    )
    .map_err(|message| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, &message))?;
    let use_post = obj.get("use_post").and_then(|value| value.as_bool()).unwrap_or(true);
    let sitename = obj.get("sitename").and_then(|value| value.as_str()).map(|v| v.trim()).unwrap_or("");
    let device = obj.get("device").and_then(|value| value.as_str()).map(|v| v.trim()).unwrap_or("");

    upsert_setting_string(state, "sponsor_epay_url", &url).await.map_err(internal_error)?;
    upsert_setting_string(state, "sponsor_epay_pid", pid).await.map_err(internal_error)?;
    upsert_setting_string(state, "sponsor_epay_key", key).await.map_err(internal_error)?;
    upsert_setting_string(state, "sponsor_epay_submit_path", &submit_path).await.map_err(internal_error)?;
    upsert_setting_string(state, "sponsor_epay_use_post", if use_post { "1" } else { "0" }).await.map_err(internal_error)?;
    upsert_setting_string(state, "sponsor_epay_sitename", sitename).await.map_err(internal_error)?;
    upsert_setting_string(state, "sponsor_epay_device", device).await.map_err(internal_error)?;

    Ok(json_value_response(json!({
        "success": true
    })))
}
