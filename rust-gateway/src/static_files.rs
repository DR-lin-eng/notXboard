use crate::*;
use tokio_util::io::ReaderStream;

pub async fn assets_file(
    headers: HeaderMap,
    uri: Uri,
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Response<Body> {
    if is_private_admin_asset_path(&path) {
        return json_error(StatusCode::NOT_FOUND, "Not found");
    }
    serve_static_under(
        crate::runtime_paths::public_path("assets"),
        &path,
        &headers,
        &uri,
    )
    .await
}

fn is_private_admin_asset_path(raw_path: &str) -> bool {
    normalize_relative_path(raw_path)
        .and_then(|path| {
            path.components().find_map(|component| match component {
                std::path::Component::Normal(value) => Some(
                    value
                        .to_str()
                        .is_some_and(|segment| segment.eq_ignore_ascii_case("admin")),
                ),
                std::path::Component::CurDir => None,
                _ => Some(false),
            })
        })
        .unwrap_or(false)
}

pub async fn public_file(
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    let path = uri.path().trim_start_matches('/');
    serve_static_under(
        crate::runtime_paths::public_path(""),
        path,
        &headers,
        &uri,
    )
    .await
}

pub async fn installer_file(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    if !crate::machine_bootstrap_support::installer_asset_authorized(&state, &uri).await {
        return crate::machine_bootstrap_support::no_store_error(
            StatusCode::NOT_FOUND,
            "Not found",
        );
    }
    let mut response = public_file(headers, uri).await;
    response
        .headers_mut()
        .insert("cache-control", HeaderValue::from_static("private, no-store"));
    response
        .headers_mut()
        .insert("pragma", HeaderValue::from_static("no-cache"));
    response
}

pub async fn theme_file(
    headers: HeaderMap,
    uri: Uri,
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Response<Body> {
    serve_theme_asset(&path, &headers, &uri).await
}

async fn serve_static_under(
    base_dir: std::path::PathBuf,
    raw_path: &str,
    headers: &HeaderMap,
    uri: &Uri,
) -> Response<Body> {
    let Some(safe_path) = normalize_relative_path(raw_path) else {
        return json_error(StatusCode::NOT_FOUND, "Not found");
    };
    let Some(file_path) = canonical_file_under(&base_dir, &safe_path).await else {
        return json_error(StatusCode::NOT_FOUND, "Not found");
    };
    let metadata = match tokio::fs::metadata(&file_path).await {
        Ok(metadata) => metadata,
        Err(_) => return json_error(StatusCode::NOT_FOUND, "Not found"),
    };
    if !metadata.is_file() {
        return json_error(StatusCode::NOT_FOUND, "Not found");
    }

    serve_file(file_path, metadata, headers, uri).await
}

async fn serve_theme_asset(raw_path: &str, headers: &HeaderMap, uri: &Uri) -> Response<Body> {
    let Some(file_path) = theme_support::resolve_theme_asset_path(raw_path) else {
        return json_error(StatusCode::NOT_FOUND, "Not found");
    };
    let metadata = match tokio::fs::metadata(&file_path).await {
        Ok(metadata) => metadata,
        Err(_) => return json_error(StatusCode::NOT_FOUND, "Not found"),
    };
    if !metadata.is_file() {
        return json_error(StatusCode::NOT_FOUND, "Not found");
    }

    serve_file(file_path, metadata, headers, uri).await
}

async fn serve_file(
    file_path: std::path::PathBuf,
    metadata: std::fs::Metadata,
    headers: &HeaderMap,
    uri: &Uri,
) -> Response<Body> {
    let etag = metadata_etag(&metadata);
    let cache_control = cache_control_for_uri(uri);
    if request_etag_matches(headers, &etag) {
        return Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .header(ETAG, etag)
            .header("Cache-Control", cache_control)
            .body(Body::empty())
            .unwrap_or_else(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "internal error"));
    }

    let file = match tokio::fs::File::open(&file_path).await {
        Ok(file) => file,
        Err(_) => return json_error(StatusCode::INTERNAL_SERVER_ERROR, "open static file failed"),
    };
    let stream = ReaderStream::new(file);

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, content_type_for_path(&file_path))
        .header("Content-Length", metadata.len())
        .header("Cache-Control", cache_control)
        .header("X-Content-Type-Options", "nosniff")
        .header(ETAG, etag)
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "internal error"))
}

fn metadata_etag(metadata: &std::fs::Metadata) -> String {
    let modified = metadata
        .modified()
        .ok()
        .and_then(|value| value.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    format!("W/\"{:x}-{:x}\"", metadata.len(), modified)
}

fn request_etag_matches(headers: &HeaderMap, etag: &str) -> bool {
    headers
        .get("if-none-match")
        .and_then(|value| value.to_str().ok())
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .any(|candidate| {
                    candidate == "*" || weak_etag_value(candidate) == weak_etag_value(etag)
                })
        })
        .unwrap_or(false)
}

fn weak_etag_value(etag: &str) -> &str {
    etag.strip_prefix("W/").unwrap_or(etag)
}

fn cache_control_for_uri(uri: &Uri) -> &'static str {
    let versioned = uri.query().is_some_and(|query| {
        query.split('&').any(|pair| {
            pair.split_once('=')
                .map(|(key, value)| key == "v" && !value.is_empty())
                .unwrap_or(false)
        })
    });
    if versioned {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=3600, stale-while-revalidate=86400"
    }
}

fn normalize_relative_path(raw_path: &str) -> Option<std::path::PathBuf> {
    let trimmed = raw_path.trim_start_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    let path = std::path::Path::new(trimmed);
    if path
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir | std::path::Component::RootDir | std::path::Component::Prefix(_)))
    {
        return None;
    }
    Some(path.to_path_buf())
}

async fn canonical_file_under(base_dir: &std::path::Path, relative_path: &std::path::Path) -> Option<std::path::PathBuf> {
    let base = tokio::fs::canonicalize(base_dir).await.ok()?;
    let file = tokio::fs::canonicalize(base_dir.join(relative_path)).await.ok()?;
    file.starts_with(&base).then_some(file)
}

fn content_type_for_path(path: &std::path::Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()).unwrap_or_default().to_ascii_lowercase().as_str() {
        "sh" => "text/x-shellscript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "go" => "text/plain; charset=utf-8",
        "html" => "text/html; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        "map" => "application/json; charset=utf-8",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versioned_assets_use_immutable_cache_control() {
        let uri = "/assets/app.js?theme=dark&v=20260711"
            .parse::<Uri>()
            .expect("valid URI");

        assert_eq!(
            cache_control_for_uri(&uri),
            "public, max-age=31536000, immutable"
        );
    }

    #[test]
    fn unversioned_assets_use_bounded_cache_control() {
        for raw_uri in [
            "/assets/app.js",
            "/assets/app.js?theme=dark",
            "/assets/app.js?v=",
            "/assets/app.js?version=20260711",
        ] {
            let uri = raw_uri.parse::<Uri>().expect("valid URI");
            assert_eq!(
                cache_control_for_uri(&uri),
                "public, max-age=3600, stale-while-revalidate=86400"
            );
        }
    }

    #[test]
    fn if_none_match_supports_lists_wildcards_and_weak_comparison() {
        let etag = r#"W/"a-123""#;
        for header_value in [r#""other", W/"a-123""#, r#""a-123""#, "*"] {
            let mut headers = HeaderMap::new();
            headers.insert(
                "if-none-match",
                header_value.parse().expect("valid header value"),
            );
            assert!(request_etag_matches(&headers, etag));
        }

        let mut headers = HeaderMap::new();
        headers.insert(
            "if-none-match",
            r#"W/"different""#.parse().expect("valid header value"),
        );
        assert!(!request_etag_matches(&headers, etag));
    }

    #[test]
    fn public_assets_reject_private_admin_prefix() {
        for path in [
            "admin/index.html",
            "admin/assets/index.js",
            "/ADMIN/assets/vendor.js",
            "./admin/index.html",
            "AdMiN/locale/zh.json",
        ] {
            assert!(
                is_private_admin_asset_path(path),
                "path should be private: {path}"
            );
        }

        for path in ["administrator/app.js", "admin-console.js", "theme/admin.css"] {
            assert!(
                !is_private_admin_asset_path(path),
                "path should remain public: {path}"
            );
        }
    }
}
