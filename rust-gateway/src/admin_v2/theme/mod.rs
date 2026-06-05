use super::super::*;
use crate::archive_limit_support::{
    copy_zip_entry_limited, validate_zip_metadata, THEME_ARCHIVE_LIMITS,
    THEME_ARCHIVE_MAX_BYTES,
};

pub async fn get_themes(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_themes_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_theme_config(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_get_theme_config_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn save_theme_config(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_save_theme_config_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn upload_theme(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    multipart: axum::extract::Multipart,
) -> Response<Body> {
    match build_upload_theme_response(&state, headers, multipart).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn delete_theme(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_delete_theme_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_get_themes_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let active = theme_support::load_active_theme_name(state).await;
    let current_theme = theme_support::load_current_theme_name(state).await;
    let themes = theme_support::build_theme_catalog(&current_theme);
    Ok(json_value_response(success_response_payload(json!({
        "themes": themes,
        "active": active,
    }))))
}

async fn build_get_theme_config_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let name = parse_theme_name(&payload)?;
    let theme = theme_support::find_theme(&name)
        .ok_or_else(|| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"Theme not found"})))?;
    let config = theme_support::load_theme_config(state, &theme).await;
    Ok(json_value_response(success_response_payload(Value::Object(config))))
}

async fn build_save_theme_config_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let name = parse_theme_name(&payload)?;
    let theme = theme_support::find_theme(&name)
        .ok_or_else(|| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"Theme not found"})))?;
    let config_payload = payload
        .get("config")
        .and_then(Value::as_object)
        .cloned()
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let saved = theme_support::save_theme_config(state, &theme, config_payload).await?;
    Ok(json_value_response(success_response_payload(Value::Object(saved))))
}

async fn build_upload_theme_response(
    state: &AppState,
    headers: HeaderMap,
    mut multipart: axum::extract::Multipart,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let mut file_name = None::<String>;
    let mut file_bytes = None::<bytes::Bytes>;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"Theme upload failed"})))?
    {
        if field.name().unwrap_or_default() != "file" {
            continue;
        }
        file_name = field.file_name().map(|value| value.to_string());
        file_bytes = Some(
            field
                .bytes()
                .await
                .map_err(|_| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"Theme upload failed"})))?,
        );
        break;
    }

    let upload_name = file_name.unwrap_or_default();
    if !upload_name.to_lowercase().ends_with(".zip") {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"Invalid theme package"})));
    }
    let file_bytes = file_bytes.ok_or_else(|| {
        json_status_response(StatusCode::UNPROCESSABLE_ENTITY, json!({"message":"Theme upload failed"}))
    })?;
    if file_bytes.len() > THEME_ARCHIVE_MAX_BYTES {
        return Ok(json_status_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({"message":"Theme package size cannot exceed 10MB"}),
        ));
    }

    let extracted = extract_theme_archive(&file_bytes)?;
    let target_root = crate::runtime_paths::state_path("theme");
    std::fs::create_dir_all(&target_root).map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Theme upload failed"))?;
    if theme_support::is_system_theme(&extracted.name) {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"Cannot upload theme with same name as system theme"})));
    }

    let target_path = target_root.join(&extracted.name);
    if target_path.exists() {
        let old_config_raw = std::fs::read_to_string(target_path.join("config.json"))
            .map_err(|_| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"Existing theme missing config file"})))?;
        let old_value = serde_json::from_str::<Value>(&old_config_raw)
            .map_err(|_| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"Existing theme missing config file"})))?;
        let old_version = old_value.get("version").and_then(Value::as_str).unwrap_or("0.0.0");
        if compare_semver(&extracted.version, old_version) <= 0 {
            return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"Theme exists and not a newer version"})));
        }
        std::fs::remove_dir_all(&target_path).map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Theme upload failed"))?;
    }

    std::fs::rename(&extracted.directory, &target_path).or_else(|_| {
        copy_dir_all(&extracted.directory, &target_path)?;
        std::fs::remove_dir_all(&extracted.directory)
    })
    .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Theme upload failed"))?;

    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_delete_theme_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let name = parse_theme_name(&payload)?;
    if theme_support::is_system_theme(&name) {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"System theme cannot be deleted"})));
    }
    let current_theme = theme_support::load_current_theme_name(state).await;
    if name.eq_ignore_ascii_case(&current_theme) {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"Current theme cannot be deleted"})));
    }

    let theme_path = crate::runtime_paths::state_path(std::path::Path::new("theme").join(&name));
    if !theme_path.exists() {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"Theme not found"})));
    }

    let public_theme_path = crate::runtime_paths::public_path(std::path::Path::new("theme").join(&name));
    if public_theme_path.exists() {
        let _ = std::fs::remove_dir_all(&public_theme_path);
    }
    std::fs::remove_dir_all(&theme_path)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Theme deletion failed"))?;
    let _ = theme_support::save_theme_config_value(state, &format!("theme_{}", name.to_ascii_lowercase()), Value::Null).await;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

fn parse_theme_name(payload: &Value) -> Result<String, Response<Body>> {
    payload
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))
}

struct ExtractedTheme {
    name: String,
    version: String,
    directory: std::path::PathBuf,
}

fn extract_theme_archive(bytes: &bytes::Bytes) -> Result<ExtractedTheme, Response<Body>> {
    let reader = std::io::Cursor::new(bytes.as_ref());
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|_| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"Invalid theme package"})))?;
    validate_zip_metadata(&mut archive, THEME_ARCHIVE_LIMITS, "Theme package is too large")?;
    let temp_root = std::env::temp_dir().join(format!("notxboard-theme-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_root)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Theme upload failed"))?;

    let mut total_written = 0_u64;
    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|_| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"Invalid theme package"})))?;
        let enclosed = file
            .enclosed_name()
            .map(|path| path.to_path_buf())
            .ok_or_else(|| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"Theme package contains unsafe paths"})))?;
        let out_path = temp_root.join(enclosed);
        if file.is_dir() {
            std::fs::create_dir_all(&out_path)
                .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Theme upload failed"))?;
            continue;
        }
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Theme upload failed"))?;
        }
        let mut out_file = std::fs::File::create(&out_path)
            .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "Theme upload failed"))?;
        copy_zip_entry_limited(
            &mut file,
            &mut out_file,
            &mut total_written,
            THEME_ARCHIVE_LIMITS,
            "Theme package is too large",
        )?;
    }

    let mut candidate_dirs = Vec::new();
    if temp_root.join("config.json").exists() {
        candidate_dirs.push(temp_root.clone());
    }
    if let Ok(entries) = std::fs::read_dir(&temp_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("config.json").exists() {
                candidate_dirs.push(path);
            }
        }
    }
    let theme_dir = candidate_dirs
        .into_iter()
        .next()
        .ok_or_else(|| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"Theme config file not found"})))?;

    if !theme_dir.join("dashboard.blade.php").exists() {
        return Err(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"Missing required theme file: dashboard.blade.php"})));
    }

    let raw = std::fs::read_to_string(theme_dir.join("config.json"))
        .map_err(|_| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"Theme config file not found"})))?;
    let value = serde_json::from_str::<Value>(&raw)
        .map_err(|_| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"Theme config file not found"})))?;
    let object = value
        .as_object()
        .ok_or_else(|| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"Theme config file not found"})))?;
    let name = object
        .get("name")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"Theme name not configured"})))?
        .to_string();
    validate_theme_name(&name)?;
    let version = object
        .get("version")
        .and_then(Value::as_str)
        .unwrap_or("0.0.0")
        .to_string();

    Ok(ExtractedTheme {
        name,
        version,
        directory: theme_dir,
    })
}

fn validate_theme_name(name: &str) -> Result<(), Response<Body>> {
    let valid = regex::Regex::new(r"^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$")
        .unwrap()
        .is_match(name)
        && !name.contains("..")
        && !name.contains('/')
        && !name.contains('\\');
    if !valid {
        return Err(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"Invalid theme name"})));
    }
    Ok(())
}

fn compare_semver(new_version: &str, old_version: &str) -> i32 {
    let parse = |value: &str| {
        value
            .split('.')
            .map(|part| part.parse::<i64>().unwrap_or(0))
            .collect::<Vec<_>>()
    };
    let left = parse(new_version);
    let right = parse(old_version);
    let max_len = left.len().max(right.len());
    for index in 0..max_len {
        let l = *left.get(index).unwrap_or(&0);
        let r = *right.get(index).unwrap_or(&0);
        if l > r {
            return 1;
        }
        if l < r {
            return -1;
        }
    }
    0
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}
