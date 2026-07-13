use crate::*;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use flate2::write::GzEncoder;
use flate2::Compression;
use rsa::pkcs1v15::SigningKey;
use rsa::pkcs8::DecodePrivateKey;
use rsa::RsaPrivateKey;
use signature::{SignatureEncoding, Signer};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::process::Stdio;
#[cfg(unix)]
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;

#[derive(Clone, Debug, Default)]
pub(crate) struct DatabaseBackupStats {
    pub(crate) ran: bool,
    pub(crate) uploaded: bool,
    pub(crate) retained_local_copy: bool,
    pub(crate) compressed_bytes: u64,
    pub(crate) artifact_path: Option<String>,
    pub(crate) uploaded_object: Option<String>,
}

#[derive(Deserialize)]
struct GoogleServiceAccountKey {
    client_email: String,
    private_key: String,
    #[serde(default)]
    token_uri: Option<String>,
}

#[derive(Deserialize)]
struct GoogleAccessTokenResponse {
    access_token: String,
}

pub(crate) async fn run_scheduled_database_backup(
    state: &AppState,
    now_ts: i64,
) -> Result<DatabaseBackupStats, String> {
    if !env_bool("ENABLE_AUTO_BACKUP_AND_UPDATE", false) {
        return Ok(DatabaseBackupStats::default());
    }

    let tz = chrono::FixedOffset::east_opt(8 * 3600)
        .ok_or_else(|| "invalid timezone offset".to_string())?;
    let now = chrono::DateTime::from_timestamp(now_ts, 0)
        .ok_or_else(|| "invalid current timestamp".to_string())?
        .with_timezone(&tz);
    let (scheduled_hour, scheduled_minute) = parse_backup_schedule_at(
        &env::var("BACKUP_SCHEDULE_AT").unwrap_or_else(|_| "03:30".to_string()),
    );
    let scheduled = tz
        .with_ymd_and_hms(
            now.year(),
            now.month(),
            now.day(),
            scheduled_hour,
            scheduled_minute,
            0,
        )
        .single()
        .ok_or_else(|| "invalid backup schedule".to_string())?;
    if now < scheduled {
        return Ok(DatabaseBackupStats::default());
    }

    let marker_key = "scheduler:backup-database:last-run-at";
    let last_run_at = redis_get_string(state, marker_key)
        .await
        .ok()
        .flatten()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(0);
    if last_run_at >= scheduled.timestamp() {
        return Ok(DatabaseBackupStats::default());
    }

    match execute_database_backup(state, now_ts, true).await {
        Ok(stats) => {
            let _ = redis_setex_string(state, marker_key, 172_800, &scheduled.timestamp().to_string()).await;
            Ok(stats)
        }
        Err(err) => {
            let message = format!(
                "V2bX 数据库备份失败\n事件：backup:database:failed\n时间：{}\n\n错误：{}",
                now.format("%Y-%m-%d %H:%M:%S"),
                err
            );
            let _ = crate::ops_alert_support::send_ops_alert(state, &message).await;
            Err(err)
        }
    }
}

pub(crate) async fn run_database_backup_now(
    state: &AppState,
    upload_requested: bool,
) -> Result<DatabaseBackupStats, String> {
    execute_database_backup(state, Utc::now().timestamp(), upload_requested).await
}

async fn execute_database_backup(
    state: &AppState,
    now_ts: i64,
    upload_requested: bool,
) -> Result<DatabaseBackupStats, String> {
    let db_connection = env::var("DB_CONNECTION").unwrap_or_else(|_| "mysql".to_string());
    if !db_connection.eq_ignore_ascii_case("mysql") {
        return Err(format!(
            "rust backup currently supports mysql runtime only, got DB_CONNECTION={db_connection}"
        ));
    }

    let db_host = env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let db_port = env::var("DB_PORT").unwrap_or_else(|_| "3306".to_string());
    let db_name = env::var("DB_DATABASE").unwrap_or_else(|_| "xboard".to_string());
    let db_user = env::var("DB_USERNAME").unwrap_or_else(|_| "xboard".to_string());
    let db_password = env::var("DB_PASSWORD").unwrap_or_default();

    let tz = chrono::FixedOffset::east_opt(8 * 3600)
        .ok_or_else(|| "invalid timezone offset".to_string())?;
    let now = chrono::DateTime::from_timestamp(now_ts, 0)
        .ok_or_else(|| "invalid current timestamp".to_string())?
        .with_timezone(&tz);
    let stamp = now.format("%Y-%m-%d_%H-%M-%S").to_string();

    let backup_dir = crate::runtime_paths::state_path("backup");
    fs::create_dir_all(&backup_dir)
        .await
        .map_err(|err| format!("create backup directory failed: {err}"))?;
    #[cfg(unix)]
    fs::set_permissions(&backup_dir, std::fs::Permissions::from_mode(0o700))
        .await
        .map_err(|err| format!("secure backup directory failed: {err}"))?;
    let sql_path = backup_dir.join(format!("{stamp}_{db_name}_database_backup.sql"));
    let compressed_path = backup_dir.join(format!("{stamp}_{db_name}_database_backup.sql.gz"));

    let backup_result = async {
        dump_mysql_database_to_file(
            &sql_path,
            &db_host,
            &db_port,
            &db_name,
            &db_user,
            &db_password,
        )
        .await?;

        let compressed_bytes = compress_backup_file(&sql_path, &compressed_path).await?;

        let mut stats = DatabaseBackupStats {
            ran: true,
            uploaded: false,
            retained_local_copy: true,
            compressed_bytes,
            artifact_path: Some(compressed_path.display().to_string()),
            uploaded_object: None,
        };

        if upload_requested {
            let object_name = format!("backup/{stamp}_{db_name}_database_backup.sql.gz");
            upload_backup_to_google_cloud(state, &compressed_path, &object_name, now_ts).await?;
            let _ = fs::remove_file(&compressed_path).await;
            stats.uploaded = true;
            stats.retained_local_copy = false;
            stats.artifact_path = None;
            stats.uploaded_object = Some(object_name);
        }

        Ok::<DatabaseBackupStats, String>(stats)
    }
    .await;

    if backup_result.is_err() {
        let _ = fs::remove_file(&sql_path).await;
        let _ = fs::remove_file(&compressed_path).await;
    }

    backup_result
}

async fn dump_mysql_database_to_file(
    output_path: &Path,
    db_host: &str,
    db_port: &str,
    db_name: &str,
    db_user: &str,
    db_password: &str,
) -> Result<(), String> {
    let mut command = Command::new("mysqldump");
    command
        .arg("--single-transaction")
        .arg("--quick")
        .arg("--no-tablespaces")
        .arg("--skip-lock-tables")
        .arg("--host")
        .arg(db_host)
        .arg("--port")
        .arg(db_port)
        .arg("--user")
        .arg(db_user)
        .arg(db_name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if !db_password.is_empty() {
        command.env("MYSQL_PWD", db_password);
    }

    let mut child = command
        .spawn()
        .map_err(|err| format!("spawn mysqldump failed: {err}"))?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| "mysqldump stdout missing".to_string())?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| "mysqldump stderr missing".to_string())?;
    let stderr_task = tokio::spawn(async move {
        let mut buffer = Vec::new();
        let _ = stderr.read_to_end(&mut buffer).await;
        String::from_utf8_lossy(&buffer).trim().to_string()
    });

    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut output = options.open(output_path)
        .await
        .map_err(|err| format!("create dump file failed: {err}"))?;
    tokio::io::copy(&mut stdout, &mut output)
        .await
        .map_err(|err| format!("write dump file failed: {err}"))?;
    output
        .flush()
        .await
        .map_err(|err| format!("flush dump file failed: {err}"))?;

    let status = child
        .wait()
        .await
        .map_err(|err| format!("wait mysqldump failed: {err}"))?;
    let stderr_text = stderr_task
        .await
        .map_err(|err| format!("join mysqldump stderr failed: {err}"))?;
    if !status.success() {
        return Err(format!(
            "mysqldump exited with status {}{}",
            status.code().unwrap_or(-1),
            if stderr_text.is_empty() {
                String::new()
            } else {
                format!(": {stderr_text}")
            }
        ));
    }
    Ok(())
}

async fn compress_backup_file(source: &Path, target: &Path) -> Result<u64, String> {
    let source = source.to_path_buf();
    let target = target.to_path_buf();
    tokio::task::spawn_blocking(move || compress_backup_file_blocking(&source, &target))
        .await
        .map_err(|err| format!("join gzip compression failed: {err}"))?
}

fn compress_backup_file_blocking(source: &Path, target: &Path) -> Result<u64, String> {
    let input = std::fs::File::open(source)
        .map_err(|err| format!("open dump file failed: {err}"))?;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let output = options.open(target)
        .map_err(|err| format!("create gzip file failed: {err}"))?;
    let mut reader = BufReader::new(input);
    let writer = BufWriter::new(output);
    let mut encoder = GzEncoder::new(writer, Compression::default());
    std::io::copy(&mut reader, &mut encoder)
        .map_err(|err| format!("compress dump file failed: {err}"))?;
    let writer = encoder
        .finish()
        .map_err(|err| format!("finish gzip file failed: {err}"))?;
    writer
        .into_inner()
        .map_err(|err| format!("flush gzip writer failed: {err}"))?;
    std::fs::remove_file(source).map_err(|err| format!("cleanup dump file failed: {err}"))?;
    target
        .metadata()
        .map_err(|err| format!("stat gzip file failed: {err}"))
        .map(|meta| meta.len())
}

async fn upload_backup_to_google_cloud(
    state: &AppState,
    compressed_path: &Path,
    object_name: &str,
    now_ts: i64,
) -> Result<(), String> {
    let bucket = env::var("GOOGLE_CLOUD_STORAGE_BUCKET")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "GOOGLE_CLOUD_STORAGE_BUCKET is empty".to_string())?;
    let key_path = resolve_google_cloud_key_file_path(
        &env::var("GOOGLE_CLOUD_KEY_FILE").unwrap_or_default(),
    )?;
    let key_raw = fs::read_to_string(&key_path)
        .await
        .map_err(|err| format!("read Google Cloud key file failed: {}: {err}", key_path.display()))?;
    let service_account: GoogleServiceAccountKey = serde_json::from_str(&key_raw)
        .map_err(|err| format!("parse Google Cloud key file failed: {err}"))?;
    let access_token = request_google_access_token(state, &service_account, now_ts).await?;
    let payload = fs::read(compressed_path)
        .await
        .map_err(|err| format!("read compressed backup failed: {err}"))?;
    let target = format!(
        "https://storage.googleapis.com/upload/storage/v1/b/{}/o?uploadType=media&name={}",
        urlencoding::encode(&bucket),
        urlencoding::encode(object_name),
    );
    let req_uri: Uri = target
        .parse()
        .map_err(|err| format!("invalid Google Cloud upload uri: {err}"))?;
    let request = Request::builder()
        .method(Method::POST)
        .uri(req_uri)
        .header(CONTENT_TYPE, "application/gzip")
        .header(AUTHORIZATION, format!("Bearer {}", access_token))
        .body(Body::from(payload))
        .map_err(|err| format!("build Google Cloud upload request failed: {err}"))?;
    let response = state
        .backend_client
        .request(request)
        .await
        .map_err(|err| format!("Google Cloud upload request failed: {err}"))?;
    let status = response.status();
    let body = response_body_bytes(map_proxy_response(response).await).await;
    if !status.is_success() {
        return Err(format!(
            "Google Cloud upload failed: HTTP {} {}",
            status.as_u16(),
            String::from_utf8_lossy(&body).trim()
        ));
    }
    Ok(())
}

async fn request_google_access_token(
    state: &AppState,
    service_account: &GoogleServiceAccountKey,
    now_ts: i64,
) -> Result<String, String> {
    let token_uri = service_account
        .token_uri
        .clone()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "https://oauth2.googleapis.com/token".to_string());
    let jwt = build_google_service_account_jwt(service_account, now_ts, &token_uri)?;
    let body = serde_urlencoded::to_string([
        ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
        ("assertion", jwt.as_str()),
    ])
    .map_err(|err| format!("encode Google OAuth form failed: {err}"))?;
    let req_uri: Uri = token_uri
        .parse()
        .map_err(|err| format!("invalid Google OAuth token uri: {err}"))?;
    let request = Request::builder()
        .method(Method::POST)
        .uri(req_uri)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(body))
        .map_err(|err| format!("build Google OAuth request failed: {err}"))?;
    let response = state
        .backend_client
        .request(request)
        .await
        .map_err(|err| format!("Google OAuth request failed: {err}"))?;
    let status = response.status();
    let body = response_body_bytes(map_proxy_response(response).await).await;
    if !status.is_success() {
        return Err(format!(
            "Google OAuth token request failed: HTTP {} {}",
            status.as_u16(),
            String::from_utf8_lossy(&body).trim()
        ));
    }
    let token: GoogleAccessTokenResponse = serde_json::from_slice(&body)
        .map_err(|err| format!("parse Google OAuth token response failed: {err}"))?;
    if token.access_token.trim().is_empty() {
        return Err("Google OAuth access token is empty".to_string());
    }
    Ok(token.access_token)
}

fn build_google_service_account_jwt(
    service_account: &GoogleServiceAccountKey,
    now_ts: i64,
    token_uri: &str,
) -> Result<String, String> {
    let header = serde_json::to_vec(&json!({
        "alg": "RS256",
        "typ": "JWT",
    }))
    .map_err(|err| format!("serialize JWT header failed: {err}"))?;
    let claims = serde_json::to_vec(&json!({
        "iss": service_account.client_email,
        "scope": "https://www.googleapis.com/auth/devstorage.read_write",
        "aud": token_uri,
        "iat": now_ts,
        "exp": now_ts + 3600,
    }))
    .map_err(|err| format!("serialize JWT claims failed: {err}"))?;
    let signing_input = format!(
        "{}.{}",
        URL_SAFE_NO_PAD.encode(header),
        URL_SAFE_NO_PAD.encode(claims)
    );
    let key = RsaPrivateKey::from_pkcs8_pem(&service_account.private_key)
        .map_err(|err| format!("parse Google service account private key failed: {err}"))?;
    let signing_key = SigningKey::<sha2::Sha256>::new(key);
    let signature = signing_key.sign(signing_input.as_bytes());
    Ok(format!(
        "{}.{}",
        signing_input,
        URL_SAFE_NO_PAD.encode(signature.to_vec())
    ))
}

fn resolve_google_cloud_key_file_path(raw: &str) -> Result<PathBuf, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("GOOGLE_CLOUD_KEY_FILE is empty".to_string());
    }
    let candidate = PathBuf::from(raw);
    if candidate.is_absolute() {
        return Ok(candidate);
    }
    Ok(crate::runtime_paths::runtime_path(candidate))
}

fn parse_backup_schedule_at(raw: &str) -> (u32, u32) {
    let mut parts = raw.trim().split(':');
    let hour = parts
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|value| *value <= 23)
        .unwrap_or(3);
    let minute = parts
        .next()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|value| *value <= 59)
        .unwrap_or(30);
    (hour, minute)
}
