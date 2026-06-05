use super::super::super::*;
use super::models::SystemLogRow;
use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::response::Response;
use http_body_util::BodyExt;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::io::Read;

const CORE_HOOKS_CATALOG: &str = include_str!("../../../resources/hooks/core_hooks.txt");

#[derive(Deserialize, Default)]
struct CleanupDormantUsersRequest {
    #[serde(default)]
    dry_run: Option<bool>,
}

#[derive(Deserialize, Default)]
struct ResetAllUserSecurityRequest {
    #[serde(default)]
    dry_run: Option<bool>,
}

#[derive(Deserialize, Default)]
struct ResetUserPasswordRequest {
    email: Option<String>,
    #[serde(default)]
    password: Option<String>,
}

pub async fn cleanup_dormant_users(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_cleanup_dormant_users_response(&state, headers, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn reset_all_user_security(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_reset_all_user_security_response(&state, headers, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn reset_user_password(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_reset_user_password_response(&state, headers, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn export_logs_csv(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_export_logs_csv_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn list_hooks(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
) -> Response<Body> {
    match build_list_hooks_response(&state, headers).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_cleanup_dormant_users_response(
    state: &AppState,
    headers: HeaderMap,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let request: CleanupDormantUsersRequest = parse_optional_json_body(body).await?;
    let dry_run = request.dry_run.unwrap_or(false);

    let dormant_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)
         FROM v2_user
         WHERE plan_id IS NULL
           AND transfer_enable = 0
           AND (expired_at = 0 OR expired_at IS NULL)
           AND last_login_at IS NULL",
    )
    .fetch_one(&state.db)
    .await
    .map_err(internal_error)?;

    if dry_run {
        return Ok(json_value_response(success_response_payload(json!({
            "dry_run": true,
            "matched_count": dormant_count,
            "deleted_count": 0,
        }))));
    }

    let result = sqlx::query(
        "DELETE FROM v2_user
         WHERE plan_id IS NULL
           AND transfer_enable = 0
           AND (expired_at = 0 OR expired_at IS NULL)
           AND last_login_at IS NULL",
    )
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(json_value_response(success_response_payload(json!({
        "dry_run": false,
        "matched_count": dormant_count,
        "deleted_count": result.rows_affected(),
    }))))
}

async fn build_reset_all_user_security_response(
    state: &AppState,
    headers: HeaderMap,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let request: ResetAllUserSecurityRequest = parse_optional_json_body(body).await?;
    let dry_run = request.dry_run.unwrap_or(false);

    let user_ids = sqlx::query_scalar::<_, i64>("SELECT id FROM v2_user ORDER BY id ASC")
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;
    if dry_run {
        return Ok(json_value_response(success_response_payload(json!({
            "dry_run": true,
            "matched_count": user_ids.len(),
            "updated_count": 0,
        }))));
    }

    let updated = reset_many_user_security(state, &user_ids)
        .await
        .map_err(internal_error)?;

    Ok(json_value_response(success_response_payload(json!({
        "dry_run": false,
        "matched_count": updated,
        "updated_count": updated,
    }))))
}

async fn build_reset_user_password_response(
    state: &AppState,
    headers: HeaderMap,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let request: ResetUserPasswordRequest = parse_required_json_body(body).await?;
    let email = request
        .email
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "邮箱不能为空"))?;
    let password = request
        .password
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(random_guid_hex_local);
    if password.chars().count() < 8 {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "密码长度最小8位"));
    }

    let exists = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM v2_user WHERE email = ?")
        .bind(email)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    if exists == 0 {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "邮箱不存在"));
    }

    let hashed = bcrypt::hash(&password, 12)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "重置失败"))?;
    sqlx::query(
        "UPDATE v2_user
         SET password = ?, password_algo = NULL, password_salt = NULL, updated_at = ?
         WHERE email = ?",
    )
    .bind(hashed)
    .bind(Utc::now().timestamp())
    .bind(email)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(json_value_response(success_response_payload(json!({
        "email": email,
        "password": password,
    }))))
}

async fn build_export_logs_csv_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let days = params
        .get("days")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(1)
        .clamp(1, 365);
    let cutoff = {
        let now = chrono::Local::now();
        let start = now
            .checked_sub_signed(chrono::Duration::days(days))
            .unwrap_or(now)
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| json_error(StatusCode::INTERNAL_SERVER_ERROR, "构建日志时间范围失败"))?;
        start.and_utc().timestamp()
    };

    let rows = sqlx::query_as::<_, SystemLogRow>(
        "SELECT id, title, level, uri, data, context, created_at
         FROM v2_log
         WHERE created_at >= ?
         ORDER BY created_at DESC",
    )
    .bind(cutoff)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    let mut csv = String::from("\u{feff}Level,ID,Title,URI,Data,Context,Created At\n");
    for row in rows {
        let created_at = chrono::DateTime::from_timestamp(row.created_at, 0)
            .map(|value| value.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| row.created_at.to_string());
        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
            sanitize_csv_field(row.level.as_deref().unwrap_or("")),
            sanitize_csv_field(&row.id.to_string()),
            sanitize_csv_field(&row.title),
            sanitize_csv_field(&row.uri),
            sanitize_csv_field(row.data.as_deref().unwrap_or("")),
            sanitize_csv_field(row.context.as_deref().unwrap_or("")),
            sanitize_csv_field(&created_at),
        ));
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/csv; charset=utf-8")
        .header("Content-Disposition", "attachment; filename=\"v2_logs.csv\"")
        .body(Body::from(csv))
        .unwrap())
}

async fn build_list_hooks_response(
    state: &AppState,
    headers: HeaderMap,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let hook_pattern = regex::Regex::new(
        r#"HookManager::(?:call|filter|register|registerFilter)\(['"]([a-zA-Z0-9_.-]+)['"]"#,
    )
    .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "构建 hook 扫描器失败"))?;
    let mut hooks = BTreeSet::new();
    add_core_hooks(&mut hooks);
    hooks.extend(crate::builtin_plugin_support::builtin_plugin_hooks());

    for root in [
        crate::runtime_paths::state_plugins_path(""),
        crate::runtime_paths::plugins_path(""),
    ] {
        collect_php_hooks_from_root(&root, &hook_pattern, &mut hooks);
    }

    Ok(json_value_response(success_response_payload(Value::Array(
        hooks.into_iter().map(Value::String).collect(),
    ))))
}

fn add_core_hooks(hooks: &mut BTreeSet<String>) {
    hooks.extend(
        CORE_HOOKS_CATALOG
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned),
    );
}

fn collect_php_hooks_from_root(
    root: &std::path::Path,
    hook_pattern: &regex::Regex,
    hooks: &mut BTreeSet<String>,
) {
    if !root.exists() {
        return;
    }
    for entry in walk_php_files(root) {
        let path = match entry {
            Ok(path) => path,
            Err(_) => continue,
        };
        let content = match std::fs::read_to_string(&path) {
            Ok(content) => content,
            Err(_) => continue,
        };
        for capture in hook_pattern.captures_iter(&content) {
            if let Some(value) = capture.get(1) {
                hooks.insert(value.as_str().to_string());
            }
        }
    }
}

fn walk_php_files(root: &std::path::Path) -> Vec<Result<std::path::PathBuf, std::io::Error>> {
    let mut out = Vec::new();
    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(err) => {
            out.push(Err(err));
            return out;
        }
    };
    for entry in entries {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_dir() {
                    out.extend(walk_php_files(&path));
                } else if path.extension().and_then(|value| value.to_str()) == Some("php") {
                    out.push(Ok(path));
                }
            }
            Err(err) => out.push(Err(err)),
        }
    }
    out
}

fn sanitize_csv_field(value: &str) -> String {
    let value = if value.is_empty() {
        String::new()
    } else {
        let trimmed = value.trim_start();
        if matches!(trimmed.chars().next(), Some('=' | '+' | '-' | '@' | '\t')) {
            format!("'{}", value)
        } else {
            value.to_string()
        }
    };
    value.replace('"', "\"\"")
}

fn random_alnum_local(len: usize) -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut bytes = vec![0_u8; len];
    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        let _ = file.read_exact(&mut bytes);
    }
    bytes
        .into_iter()
        .map(|byte| CHARS[(byte as usize) % CHARS.len()] as char)
        .collect()
}

fn random_letters_local(len: usize) -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut bytes = vec![0_u8; len];
    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        let _ = file.read_exact(&mut bytes);
    }
    bytes
        .into_iter()
        .map(|byte| CHARS[(byte as usize) % CHARS.len()] as char)
        .collect()
}

fn random_hex_local(len: usize) -> String {
    let bytes_len = len.div_ceil(2);
    let mut bytes = vec![0_u8; bytes_len];
    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        let _ = file.read_exact(&mut bytes);
    }
    let mut out = bytes.iter().map(|value| format!("{:02x}", value)).collect::<String>();
    out.truncate(len);
    out
}

fn random_guid_hex_local() -> String {
    let bytes = random_guid_bytes_local();
    bytes.iter().map(|value| format!("{:02x}", value)).collect()
}

fn random_guid_hyphenated_local() -> String {
    let data = random_guid_hex_local();
    format!(
        "{}-{}-{}-{}-{}",
        &data[0..8],
        &data[8..12],
        &data[12..16],
        &data[16..20],
        &data[20..32],
    )
}

fn random_guid_bytes_local() -> [u8; 16] {
    let mut data = [0_u8; 16];
    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        let _ = file.read_exact(&mut data);
    }
    data[6] = (data[6] & 0x0f) | 0x40;
    data[8] = (data[8] & 0x3f) | 0x80;
    data
}

async fn parse_optional_json_body<T: serde::de::DeserializeOwned + Default>(
    body: Body,
) -> Result<T, Response<Body>> {
    let bytes = body
        .collect()
        .await
        .map_err(|err| json_error(StatusCode::BAD_REQUEST, &format!("read request body failed: {err}")))?
        .to_bytes();
    if bytes.is_empty() {
        return Ok(T::default());
    }
    serde_json::from_slice::<T>(&bytes)
        .map_err(|_| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))
}

async fn parse_required_json_body<T: serde::de::DeserializeOwned>(
    body: Body,
) -> Result<T, Response<Body>> {
    let bytes = body
        .collect()
        .await
        .map_err(|err| json_error(StatusCode::BAD_REQUEST, &format!("read request body failed: {err}")))?
        .to_bytes();
    if bytes.is_empty() {
        return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    }
    serde_json::from_slice::<T>(&bytes)
        .map_err(|_| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))
}
