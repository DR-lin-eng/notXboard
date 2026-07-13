use crate::AppState;
use axum::{
    body::Body,
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, Response, StatusCode},
    middleware::Next,
};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use std::{env, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PublicSiteMode {
    Minimal,
    Dashboard,
    Disabled,
}

impl PublicSiteMode {
    fn parse(value: Option<&str>) -> Result<Self, String> {
        let value = value.map(str::trim).filter(|value| !value.is_empty());
        let normalized = value.map(str::to_ascii_lowercase);
        match normalized.as_deref() {
            None | Some("minimal") => Ok(Self::Minimal),
            Some("dashboard") => Ok(Self::Dashboard),
            Some("disabled") | Some("off") => Ok(Self::Disabled),
            Some(_) => Err(format!(
                "PUBLIC_SITE_MODE must be minimal, dashboard, or disabled; got {}",
                value.unwrap_or_default()
            )),
        }
    }
}

#[derive(Clone)]
struct WebAccessCredentials {
    username: String,
    password: String,
}

#[derive(Clone)]
pub(crate) struct ExposureConfig {
    public_site_mode: PublicSiteMode,
    public_catalog_enabled: bool,
    web_access: Option<WebAccessCredentials>,
}

impl ExposureConfig {
    pub(crate) fn from_env() -> Result<Self, String> {
        let public_site_mode = env::var("PUBLIC_SITE_MODE").ok();
        let public_catalog_enabled = env::var("PUBLIC_CATALOG_ENABLED").ok();
        let web_access_username = env::var("WEB_ACCESS_USERNAME").ok();
        let web_access_password = env::var("WEB_ACCESS_PASSWORD").ok();

        Self::from_values(
            public_site_mode.as_deref(),
            public_catalog_enabled.as_deref(),
            web_access_username.as_deref(),
            web_access_password.as_deref(),
        )
    }

    fn from_values(
        public_site_mode: Option<&str>,
        public_catalog_enabled: Option<&str>,
        web_access_username: Option<&str>,
        web_access_password: Option<&str>,
    ) -> Result<Self, String> {
        let username = web_access_username.unwrap_or_default().trim();
        let password = web_access_password.unwrap_or_default().trim();
        let web_access = match (username.is_empty(), password.is_empty()) {
            (true, true) => None,
            (false, false) => {
                if username.contains(':') {
                    return Err("WEB_ACCESS_USERNAME cannot contain ':'".to_string());
                }
                Some(WebAccessCredentials {
                    username: username.to_string(),
                    password: password.to_string(),
                })
            }
            _ => {
                return Err(
                    "WEB_ACCESS_USERNAME and WEB_ACCESS_PASSWORD must be configured together"
                        .to_string(),
                );
            }
        };

        Ok(Self {
            public_site_mode: PublicSiteMode::parse(public_site_mode)?,
            public_catalog_enabled: parse_env_bool(
                "PUBLIC_CATALOG_ENABLED",
                public_catalog_enabled,
                false,
            )?,
            web_access,
        })
    }

    pub(crate) fn public_site_mode(&self) -> PublicSiteMode {
        self.public_site_mode
    }

    pub(crate) fn public_dashboard_enabled(&self) -> bool {
        self.public_site_mode == PublicSiteMode::Dashboard
    }

    pub(crate) fn public_catalog_enabled(&self) -> bool {
        self.public_catalog_enabled
    }

    pub(crate) fn public_metadata_enabled(&self) -> bool {
        self.public_dashboard_enabled() || self.public_catalog_enabled
    }

    pub(crate) fn web_access_enabled(&self) -> bool {
        self.web_access.is_some()
    }
}

fn parse_env_bool(name: &str, value: Option<&str>, default: bool) -> Result<bool, String> {
    match value.map(str::trim).filter(|value| !value.is_empty()) {
        None => Ok(default),
        Some(value) if matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on") => Ok(true),
        Some(value) if matches!(value.to_ascii_lowercase().as_str(), "0" | "false" | "no" | "off") => Ok(false),
        Some(value) => Err(format!("{name} must be a boolean value; got {value}")),
    }
}

fn basic_authorization_matches(headers: &HeaderMap, credentials: &WebAccessCredentials) -> bool {
    let parsed = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().split_once(' '))
        .filter(|(scheme, encoded)| scheme.eq_ignore_ascii_case("basic") && !encoded.trim().is_empty())
        .and_then(|(_, encoded)| BASE64_STANDARD.decode(encoded.trim()).ok())
        .and_then(|decoded| String::from_utf8(decoded).ok())
        .and_then(|decoded| {
            let (username, password) = decoded.split_once(':')?;
            Some((username.to_string(), password.to_string()))
        });

    let (username, password) = parsed.unwrap_or_default();
    let username_matches = crate::secure_compare_support::constant_time_eq_str(
        &username,
        &credentials.username,
    );
    let password_matches = crate::secure_compare_support::constant_time_eq_str(
        &password,
        &credentials.password,
    );
    username_matches & password_matches
}

pub(crate) async fn require_web_access(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response<Body> {
    let Some(credentials) = state.exposure.web_access.as_ref() else {
        return next.run(request).await;
    };
    if basic_authorization_matches(request.headers(), credentials) {
        let mut response = next.run(request).await;
        insert_authenticated_cache_headers(response.headers_mut());
        return response;
    }

    web_access_challenge()
}

pub(crate) fn protect_fallback_web_response(
    state: &AppState,
    headers: &HeaderMap,
    mut response: Response<Body>,
) -> Response<Body> {
    let Some(credentials) = state.exposure.web_access.as_ref() else {
        return response;
    };
    if !basic_authorization_matches(headers, credentials) {
        return web_access_challenge();
    }
    insert_authenticated_cache_headers(response.headers_mut());
    response
}

fn web_access_challenge() -> Response<Body> {
    let mut response = Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("Content-Type", "text/plain; charset=utf-8")
        .header("Cache-Control", "no-store")
        .header(
            "WWW-Authenticate",
            "Basic realm=\"Private workspace\", charset=\"UTF-8\"",
        )
        .body(Body::from("Authentication required"))
        .unwrap();
    insert_privacy_headers(response.headers_mut());
    response
}

pub(crate) fn insert_authenticated_cache_headers(headers: &mut HeaderMap) {
    headers.insert(
        "cache-control",
        HeaderValue::from_static("private, no-store"),
    );
    headers.append("vary", HeaderValue::from_static("Authorization"));
}

pub(crate) async fn add_privacy_headers(request: Request, next: Next) -> Response<Body> {
    let mut response = next.run(request).await;
    insert_privacy_headers(response.headers_mut());
    response
}

pub(crate) async fn mark_private_response(request: Request, next: Next) -> Response<Body> {
    let mut response = next.run(request).await;
    insert_authenticated_cache_headers(response.headers_mut());
    response
}

pub(crate) fn insert_privacy_headers(headers: &mut HeaderMap) {
    headers.insert(
        "x-robots-tag",
        HeaderValue::from_static("noindex, nofollow, noarchive, nosnippet, noimageindex"),
    );
    headers.insert(
        "referrer-policy",
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        "permissions-policy",
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        "content-security-policy",
        HeaderValue::from_static(
            "script-src-attr 'none'; object-src 'none'; base-uri 'self'; frame-ancestors 'self'",
        ),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposure_defaults_are_low_visibility() {
        let config = ExposureConfig::from_values(None, None, None, None).expect("valid config");

        assert_eq!(config.public_site_mode(), PublicSiteMode::Minimal);
        assert!(!config.public_dashboard_enabled());
        assert!(!config.public_catalog_enabled());
        assert!(!config.web_access_enabled());
    }

    #[test]
    fn incomplete_web_access_credentials_are_rejected() {
        assert!(ExposureConfig::from_values(None, None, Some("member"), None).is_err());
        assert!(ExposureConfig::from_values(None, None, None, Some("password")).is_err());
    }

    #[test]
    fn explicit_public_modes_and_boolean_values_are_parsed() {
        let dashboard = ExposureConfig::from_values(
            Some("DASHBOARD"),
            Some("yes"),
            Some("member"),
            Some("password"),
        )
        .expect("valid dashboard config");
        assert_eq!(dashboard.public_site_mode(), PublicSiteMode::Dashboard);
        assert!(dashboard.public_dashboard_enabled());
        assert!(dashboard.public_catalog_enabled());
        assert!(dashboard.web_access_enabled());

        let disabled = ExposureConfig::from_values(Some("off"), Some("0"), None, None)
            .expect("valid disabled config");
        assert_eq!(disabled.public_site_mode(), PublicSiteMode::Disabled);
        assert!(!disabled.public_catalog_enabled());
    }

    #[test]
    fn basic_authorization_requires_both_exact_credentials() {
        let credentials = WebAccessCredentials {
            username: "member".to_string(),
            password: "long-test-password".to_string(),
        };
        let mut valid = HeaderMap::new();
        valid.insert(
            "authorization",
            HeaderValue::from_static("Basic bWVtYmVyOmxvbmctdGVzdC1wYXNzd29yZA=="),
        );
        assert!(basic_authorization_matches(&valid, &credentials));

        let mut wrong = HeaderMap::new();
        wrong.insert(
            "authorization",
            HeaderValue::from_static("Basic bWVtYmVyOndyb25n"),
        );
        assert!(!basic_authorization_matches(&wrong, &credentials));
        assert!(!basic_authorization_matches(&HeaderMap::new(), &credentials));
    }

    #[test]
    fn privacy_headers_prevent_indexing_and_referrer_leaks() {
        let mut headers = HeaderMap::new();
        insert_privacy_headers(&mut headers);

        assert_eq!(
            headers.get("x-robots-tag").and_then(|value| value.to_str().ok()),
            Some("noindex, nofollow, noarchive, nosnippet, noimageindex")
        );
        assert_eq!(
            headers.get("referrer-policy").and_then(|value| value.to_str().ok()),
            Some("no-referrer")
        );
    }

    #[test]
    fn authenticated_responses_cannot_enter_shared_caches() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "cache-control",
            HeaderValue::from_static("public, max-age=31536000, immutable"),
        );

        insert_authenticated_cache_headers(&mut headers);

        assert_eq!(
            headers.get("cache-control").and_then(|value| value.to_str().ok()),
            Some("private, no-store")
        );
        assert_eq!(
            headers.get("vary").and_then(|value| value.to_str().ok()),
            Some("Authorization")
        );
    }
}
