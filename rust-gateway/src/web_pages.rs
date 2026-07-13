use crate::*;

const LOGIN_LINUX_DO_TEMPLATE: &str = include_str!("../resources/views/login-linux-do.html");
const PRIVATE_ENTRY_TEMPLATE: &str = include_str!("../resources/views/private-entry.html");
const PUBLIC_DASHBOARD_TEMPLATE: &str = include_str!("../resources/views/public-dashboard.html");
const ADMIN_TEMPLATE: &str = include_str!("../resources/views/admin.html");
const ADMIN_CONSOLE_STYLESHEET: &str = include_str!("../resources/admin/admin-console.css");
const ADMIN_CONSOLE_SCRIPT: &str = include_str!("../resources/admin/admin-console.js");
const ADMIN_COMMAND_CENTER_TEMPLATE: &str =
    include_str!("../resources/views/admin-command-center.html");

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
    let response = match state.exposure.public_site_mode() {
        PublicSiteMode::Disabled => return private_not_found_response(),
        PublicSiteMode::Minimal => build_private_entry_page_response(&state, &headers).await,
        PublicSiteMode::Dashboard => build_public_dashboard_page_response(&state, &headers).await,
    };
    match response {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn robots_txt() -> Response<Body> {
    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/plain; charset=utf-8")
        .header("Cache-Control", "public, max-age=86400")
        .body(Body::from("User-agent: *\nDisallow: /\n"))
        .unwrap();
    crate::exposure_control::insert_privacy_headers(response.headers_mut());
    response
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

pub async fn admin_console_stylesheet(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(admin_path): axum::extract::Path<String>,
    headers: HeaderMap,
) -> Response<Body> {
    build_admin_console_asset_response(
        &state,
        &headers,
        &admin_path,
        "text/css; charset=utf-8",
        ADMIN_CONSOLE_STYLESHEET,
    )
    .await
    .unwrap_or_else(|response| response)
}

pub async fn admin_console_script(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(admin_path): axum::extract::Path<String>,
    headers: HeaderMap,
) -> Response<Body> {
    build_admin_console_asset_response(
        &state,
        &headers,
        &admin_path,
        "application/javascript; charset=utf-8",
        ADMIN_CONSOLE_SCRIPT,
    )
    .await
    .unwrap_or_else(|response| response)
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
    let _ = warm_setting_cache(
        state,
        &[
            "safe_mode_enable",
            "app_url",
            "frontend_theme",
            "current_theme",
            "app_name",
            "app_description",
            "logo",
        ],
    )
    .await;
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
    let _ = warm_setting_cache(state, &["safe_mode_enable", "app_url"]).await;
    enforce_safe_mode_host(state, headers).await?;

    Ok(html_response(LOGIN_LINUX_DO_TEMPLATE.to_string()))
}

async fn build_private_entry_page_response(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Response<Body>, Response<Body>> {
    let _ = warm_setting_cache(state, &["safe_mode_enable", "app_url", "app_name"]).await;
    enforce_safe_mode_host(state, headers).await?;

    let default_title = read_env_fallback("APP_NAME", "Workspace");
    let title = read_setting_from_db(state, "app_name", &default_title).await;
    let rendered = PRIVATE_ENTRY_TEMPLATE.replace("{{ $title }}", &escape_html(&title));
    Ok(html_response(rendered))
}

async fn build_public_dashboard_page_response(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Response<Body>, Response<Body>> {
    let _ = warm_setting_cache(
        state,
        &[
            "safe_mode_enable",
            "app_url",
            "app_name",
            "app_description",
        ],
    )
    .await;
    enforce_safe_mode_host(state, headers).await?;

    let title = read_setting_from_db(state, "app_name", "Portal").await;
    let version = read_env_fallback("APP_VERSION", "1.0.0");
    let description = read_setting_from_db(state, "app_description", "Secure access portal").await;

    let rendered = PUBLIC_DASHBOARD_TEMPLATE
        .replace("{{ $title }}", &escape_html(&title))
        .replace("{{ $description }}", &escape_html(&description))
        .replace("{{ $version }}", &escape_html(&version));

    Ok(html_response(rendered))
}

async fn build_admin_page_response(
    state: &AppState,
    headers: &HeaderMap,
    admin_path: &str,
) -> Result<Response<Body>, Response<Body>> {
    let _ = warm_setting_cache(
        state,
        &[
            "safe_mode_enable",
            "app_url",
            "secure_path",
            "frontend_admin_path",
            "app_name",
            "logo",
        ],
    )
    .await;
    validate_secure_path(state, headers, admin_path).await?;
    let title = read_setting_from_db(state, "app_name", "Portal").await;
    let version = read_env_fallback("APP_VERSION", "1.0.0");
    let logo = read_setting_from_db(state, "logo", "").await;
    let rendered = ADMIN_TEMPLATE
        .replace("{{ $title_json }}", &json_script_string(&title))
        .replace("{{ $version_json }}", &json_script_string(&version))
        .replace("{{ $logo_json }}", &json_script_string(&logo))
        .replace(
            "{{ $secure_path_json }}",
            &json_script_string(admin_path),
        )
        .replace("{{ $title }}", &escape_html(&title))
        .replace("{{ $version }}", &escape_html(&version))
        .replace("{{ $logo }}", &escape_html(&logo))
        .replace("{{ $secure_path }}", &escape_html(admin_path))
        .replace("{{ $logo ?: '' }}", &escape_html(&logo));

    Ok(html_response(rendered))
}

async fn build_admin_console_asset_response(
    state: &AppState,
    headers: &HeaderMap,
    admin_path: &str,
    content_type: &'static str,
    source: &'static str,
) -> Result<Response<Body>, Response<Body>> {
    let _ = warm_setting_cache(
        state,
        &[
            "safe_mode_enable",
            "app_url",
            "secure_path",
            "frontend_admin_path",
        ],
    )
    .await;
    validate_secure_path(state, headers, admin_path).await?;
    Ok(admin_console_asset_response(content_type, source))
}

async fn build_admin_command_center_page_response(
    state: &AppState,
    headers: &HeaderMap,
    admin_path: &str,
) -> Result<Response<Body>, Response<Body>> {
    let _ = warm_setting_cache(
        state,
        &[
            "safe_mode_enable",
            "app_url",
            "secure_path",
            "frontend_admin_path",
            "app_name",
            "app_description",
            "logo",
        ],
    )
    .await;
    validate_secure_path(state, headers, admin_path).await?;
    let title = read_setting_from_db(state, "app_name", "Portal").await;
    let version = read_env_fallback("APP_VERSION", "1.0.0");
    let logo = read_setting_from_db(state, "logo", "").await;
    let description = read_setting_from_db(state, "app_description", "Super admin command center").await;
    let rendered = ADMIN_COMMAND_CENTER_TEMPLATE
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
    let _ = warm_setting_cache(
        state,
        &[
            "safe_mode_enable",
            "app_url",
            "secure_path",
            "frontend_admin_path",
        ],
    )
    .await;
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
    load_cached_setting_value(state, name)
    .await
    .ok()
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
    if !secure_admin_path_matches(&expected, value) {
        return Err(json_error(StatusCode::NOT_FOUND, "Not found"));
    }
    Ok(())
}

fn html_response(rendered: String) -> Response<Body> {
    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/html; charset=utf-8")
        .header("Cache-Control", "no-store, max-age=0")
        .body(Body::from(rendered))
        .unwrap();
    crate::exposure_control::insert_privacy_headers(response.headers_mut());
    response
}

fn admin_console_asset_response(content_type: &'static str, source: &'static str) -> Response<Body> {
    let mut response = Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, content_type)
        .header("Cache-Control", "private, no-store")
        .header("X-Content-Type-Options", "nosniff")
        .body(Body::from(source))
        .unwrap();
    crate::exposure_control::insert_privacy_headers(response.headers_mut());
    response
}

fn private_not_found_response() -> Response<Body> {
    let mut response = Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header(CONTENT_TYPE, "text/plain; charset=utf-8")
        .header("Cache-Control", "no-store")
        .body(Body::from("Not Found"))
        .unwrap();
    crate::exposure_control::insert_privacy_headers(response.headers_mut());
    response
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
    if !is_valid_secure_admin_path(&secure_path) {
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

fn json_script_string(input: &str) -> String {
    serde_json::to_string(input)
        .unwrap_or_else(|_| "\"\"".to_string())
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_entry_avoids_operational_product_terms() {
        for term in [
            "机场",
            "流量",
            "带宽",
            "节点",
            "订阅",
            "bandwidth",
            "proxy",
            "subscription",
            "traffic",
            "vpn",
        ] {
            assert!(!PRIVATE_ENTRY_TEMPLATE.to_ascii_lowercase().contains(term));
        }
    }

    #[test]
    fn html_pages_are_not_indexable_or_cacheable() {
        let response = html_response("<main>Private</main>".to_string());

        assert_eq!(
            response.headers().get("cache-control").unwrap(),
            "no-store, max-age=0"
        );
        assert!(response
            .headers()
            .get("x-robots-tag")
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.contains("noindex")));
    }

    #[test]
    fn admin_template_scopes_console_assets_to_secure_path() {
        assert!(ADMIN_TEMPLATE.contains(
            r#"href="/{{ $secure_path }}/assets/admin-console.css?v={{ $version }}""#
        ));
        assert!(ADMIN_TEMPLATE.contains(
            r#"src="/{{ $secure_path }}/assets/admin-console.js?v={{ $version }}""#
        ));
        assert!(!ADMIN_TEMPLATE.contains(r#"href="/assets/admin-console.css"#));
        assert!(!ADMIN_TEMPLATE.contains(r#"src="/assets/admin-console.js"#));
    }

    #[test]
    fn admin_settings_use_safe_json_strings() {
        let encoded = json_script_string("A&B \"console\" </script>\u{2028}");
        assert_eq!(
            encoded,
            r#""A\u0026B \"console\" \u003c/script\u003e\u2028""#
        );
        assert!(!encoded.contains("</script>"));
        assert!(ADMIN_TEMPLATE.contains("title: {{ $title_json }}"));
        assert!(!ADMIN_TEMPLATE.contains(r#"title: "{{ $title }}""#));
    }

    #[test]
    fn admin_console_assets_are_private_and_typed() {
        for (content_type, source) in [
            ("text/css; charset=utf-8", ADMIN_CONSOLE_STYLESHEET),
            ("application/javascript; charset=utf-8", ADMIN_CONSOLE_SCRIPT),
        ] {
            let response = admin_console_asset_response(content_type, source);
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), content_type);
            assert_eq!(
                response.headers().get("cache-control").unwrap(),
                "private, no-store"
            );
            assert_eq!(
                response.headers().get("x-content-type-options").unwrap(),
                "nosniff"
            );
            assert!(response
                .headers()
                .get("x-robots-tag")
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value.contains("noindex")));
        }
    }
}
