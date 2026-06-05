use crate::*;

pub async fn app_page(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response<Body> {
    match build_app_page_response(&state, &headers).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn login_linux_do_page(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response<Body> {
    match build_login_linux_do_page_response(&state, &headers).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn public_dashboard_page(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response<Body> {
    match build_public_dashboard_page_response(&state, &headers).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn admin_page(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(admin_path): axum::extract::Path<String>,
    headers: HeaderMap,
) -> Response<Body> {
    match build_admin_page_response(&state, &headers, &admin_path).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn admin_command_center_page(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(admin_path): axum::extract::Path<String>,
    headers: HeaderMap,
) -> Response<Body> {
    match build_admin_command_center_page_response(&state, &headers, &admin_path).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn admin_leaderboards_redirect(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(admin_path): axum::extract::Path<String>,
    headers: HeaderMap,
) -> Response<Body> {
    match build_admin_leaderboards_redirect_response(&state, &headers, &admin_path).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_app_page_response(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Response<Body>, Response<Body>> {
    enforce_safe_mode_host(state, headers).await?;

    let theme_name = theme_support::resolve_runtime_theme_name(state).await;
    let title = read_setting_from_db(state, "app_name", "Portal").await;
    let version = read_env_fallback("APP_VERSION", "dev");
    let app_url = first_non_empty(&[
        read_setting_from_db(state, "app_url", "").await,
        std::env::var("APP_URL").unwrap_or_default(),
        String::new(),
    ]);
    let asset_version = theme_support::theme_asset_version(&theme_name, &version);
    let theme = theme_support::find_theme(&theme_name)
        .ok_or_else(|| json_error(StatusCode::INTERNAL_SERVER_ERROR, "load app page failed"))?;
    let theme_config = theme_support::load_theme_config(state, &theme).await;

    let template = theme_support::load_theme_template(&theme_name)
        .ok_or_else(|| json_error(StatusCode::INTERNAL_SERVER_ERROR, "load app page failed"))?;

    let rendered = if theme_name.eq_ignore_ascii_case(theme_support::PORTAL_THEME_NAME) {
        theme_page_render::render_portal_theme(
            &template,
            &theme_name,
            &title,
            &version,
            &asset_version,
            &read_setting_from_db(state, "app_description", "Secure access portal").await,
            &read_setting_from_db(state, "logo", "").await,
            &theme_config,
        )
    } else {
        theme_page_render::render_maintainable_theme(
            &template,
            &theme_name,
            &title,
            &version,
            &app_url,
            &asset_version,
        )
    };

    Ok(html_response(rendered))
}

async fn build_login_linux_do_page_response(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Response<Body>, Response<Body>> {
    enforce_safe_mode_host(state, headers).await?;

    let template = std::fs::read_to_string(crate::runtime_paths::resources_path("views/login-linux-do.blade.php"))
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "load login page failed"))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(template))
        .unwrap())
}

async fn build_public_dashboard_page_response(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Response<Body>, Response<Body>> {
    enforce_safe_mode_host(state, headers).await?;

    let title = read_setting_from_db(state, "app_name", "Portal").await;
    let version = read_env_fallback("APP_VERSION", "1.0.0");
    let description = read_setting_from_db(state, "app_description", "Secure access portal").await;

    let template = std::fs::read_to_string(crate::runtime_paths::resources_path("views/public-dashboard.blade.php"))
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "load public dashboard failed"))?;

    let rendered = template
        .replace("{{ $title }}", &escape_html(&title))
        .replace("{{ $description }}", &escape_html(&description))
        .replace("{{ $version }}", &escape_html(&version));

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(rendered))
        .unwrap())
}

async fn build_admin_page_response(
    state: &AppState,
    headers: &HeaderMap,
    admin_path: &str,
) -> Result<Response<Body>, Response<Body>> {
    validate_secure_path(state, headers, admin_path).await?;
    let title = read_setting_from_db(state, "app_name", "Portal").await;
    let version = read_env_fallback("APP_VERSION", "1.0.0");
    let logo = read_setting_from_db(state, "logo", "").await;
    let template = std::fs::read_to_string(crate::runtime_paths::resources_path("views/admin.blade.php"))
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "load admin page failed"))?;
    let rendered = template
        .replace("{{ $title }}", &escape_html(&title))
        .replace("{{ $version }}", &escape_html(&version))
        .replace("{{ $logo }}", &escape_html(&logo))
        .replace("{{ $secure_path }}", &escape_html(admin_path))
        .replace("{{ !empty($command_center_only) ? 'true' : 'false' }}", "false")
        .replace("{{ !empty($command_center_only) ? 'command-center-only' : '' }}", "")
        .replace("{{ !empty($command_center_only) ? 'hidden' : '' }}", "")
        .replace("{{ $logo ?: '' }}", &escape_html(&logo));

    Ok(html_response(rendered))
}

async fn build_admin_command_center_page_response(
    state: &AppState,
    headers: &HeaderMap,
    admin_path: &str,
) -> Result<Response<Body>, Response<Body>> {
    validate_secure_path(state, headers, admin_path).await?;
    let title = read_setting_from_db(state, "app_name", "Portal").await;
    let version = read_env_fallback("APP_VERSION", "1.0.0");
    let logo = read_setting_from_db(state, "logo", "").await;
    let description = read_setting_from_db(state, "app_description", "Super admin command center").await;
    let template = std::fs::read_to_string(crate::runtime_paths::resources_path("views/admin-command-center.blade.php"))
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "load command center page failed"))?;
    let rendered = template
        .replace("{{ $title }}", &escape_html(&title))
        .replace("{{ $version }}", &escape_html(&version))
        .replace("{{ $logo }}", &escape_html(&logo))
        .replace("{{ $description }}", &escape_html(&description))
        .replace("{{ $secure_path }}", &escape_html(admin_path));

    Ok(html_response(rendered))
}

async fn build_admin_leaderboards_redirect_response(
    state: &AppState,
    headers: &HeaderMap,
    admin_path: &str,
) -> Result<Response<Body>, Response<Body>> {
    validate_secure_path(state, headers, admin_path).await?;
    Ok(Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", format!("/{}/command-center", admin_path))
        .body(Body::empty())
        .unwrap())
}

async fn read_setting_from_db(state: &AppState, name: &str, default: &str) -> String {
    load_setting_value(state, name)
        .await
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default.to_string())
}

async fn load_setting_value(state: &AppState, name: &str) -> Option<String> {
    sqlx::query_scalar::<_, Option<String>>(
        "SELECT value FROM v2_settings WHERE name = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(name.to_ascii_lowercase())
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .flatten()
}

fn read_env_fallback(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

async fn enforce_safe_mode_host(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(), Response<Body>> {
    let safe_mode = read_setting_from_db(state, "safe_mode_enable", "0").await;
    if !matches!(safe_mode.trim(), "1" | "true" | "TRUE" | "on" | "yes") {
        return Ok(());
    }
    let app_url = read_setting_from_db(state, "app_url", "").await;
    if app_url.trim().is_empty() {
        return Ok(());
    }
    let expected_host = url::Url::parse(&app_url)
        .ok()
        .and_then(|url| url.host_str().map(|value| value.to_string()))
        .unwrap_or_default();
    if expected_host.is_empty() {
        return Ok(());
    }
    let request_host = headers
        .get("host")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if request_host != expected_host {
        return Err(json_error(StatusCode::FORBIDDEN, "Forbidden"));
    }
    Ok(())
}

async fn validate_secure_path(
    state: &AppState,
    headers: &HeaderMap,
    value: &str,
) -> Result<(), Response<Body>> {
    enforce_safe_mode_host(state, headers).await?;
    let secure_path = read_setting_from_db(state, "secure_path", "").await;
    let frontend_admin_path = read_setting_from_db(state, "frontend_admin_path", "").await;
    let expected = first_non_empty(&[
        secure_path,
        frontend_admin_path,
        String::new(),
    ]);
    if expected.trim().is_empty() || expected.trim() != value.trim() {
        return Err(json_error(StatusCode::NOT_FOUND, "Not found"));
    }
    Ok(())
}

fn html_response(rendered: String) -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(rendered))
        .unwrap()
}

pub async fn try_render_page(
    state: &AppState,
    headers: &HeaderMap,
    uri: &Uri,
) -> Option<Response<Body>> {
    let path = uri.path().trim_matches('/');
    if path.is_empty() {
        return None;
    }

    let secure_path = first_non_empty(&[
        read_setting_from_db(state, "secure_path", "").await,
        read_setting_from_db(state, "frontend_admin_path", "").await,
        String::new(),
    ]);
    if secure_path.is_empty() {
        return None;
    }
    if path == secure_path {
        return Some(
            build_admin_page_response(state, headers, &secure_path)
                .await
                .unwrap_or_else(|response| response),
        );
    }
    if path == format!("{}/command-center", secure_path) {
        return Some(
            build_admin_command_center_page_response(state, headers, &secure_path)
                .await
                .unwrap_or_else(|response| response),
        );
    }
    if path == format!("{}/leaderboards", secure_path) {
        return Some(
            build_admin_leaderboards_redirect_response(state, headers, &secure_path)
                .await
                .unwrap_or_else(|response| response),
        );
    }
    None
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;")
        .replace('\'', "&#39;")
}
