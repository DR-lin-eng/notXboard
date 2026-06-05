use crate::*;

pub async fn assets_file(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Response<Body> {
    serve_static_under(crate::runtime_paths::public_path("assets"), &path).await
}

pub async fn public_file(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Response<Body> {
    serve_static_under(crate::runtime_paths::public_path(""), &path).await
}

pub async fn theme_file(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Response<Body> {
    serve_theme_asset(&path).await
}

async fn serve_static_under(base_dir: std::path::PathBuf, raw_path: &str) -> Response<Body> {
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

    match tokio::fs::read(&file_path).await {
        Ok(bytes) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, content_type_for_path(&file_path))
            .body(Body::from(bytes))
            .unwrap_or_else(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "internal error")),
        Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "read static file failed"),
    }
}

async fn serve_theme_asset(raw_path: &str) -> Response<Body> {
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

    match tokio::fs::read(&file_path).await {
        Ok(bytes) => Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, content_type_for_path(&file_path))
            .body(Body::from(bytes))
            .unwrap_or_else(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "internal error")),
        Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "read static file failed"),
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
