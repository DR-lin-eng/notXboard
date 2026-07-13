use crate::*;
use crate::legacy_traffic_support::enqueue_legacy_submit_traffic;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;
const MAX_LEGACY_TRAFFIC_DELTA_KB: i64 = 100_000_000;
const MAX_LEGACY_TRAFFIC_BATCH_KB: i64 = 1_000_000_000;

#[derive(Clone, sqlx::FromRow)]
struct LegacyServerCompatRow {
    id: u64,
    server_type: String,
    parent_id: Option<u64>,
    group_ids: Option<SqlxJson<Value>>,
    rate: String,
    host: String,
    server_port: i64,
    protocol_settings: Option<SqlxJson<Value>>,
    created_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Clone, sqlx::FromRow)]
struct LegacyAvailableUserRow {
    id: i64,
    uuid: String,
    subscription_credential_version: i64,
    speed_limit: i64,
    device_limit: i64,
}

pub async fn shadowsocks_user(
    State(state): State<Arc<AppState>>,
    uri: Uri,
    headers: HeaderMap,
) -> Response<Body> {
    match build_shadowsocks_user_response(&state, uri, headers).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn shadowsocks_submit(
    State(state): State<Arc<AppState>>,
    uri: Uri,
    headers: HeaderMap,
    body: Body,
) -> Response<Body> {
    match build_legacy_submit_response(&state, uri, headers, body, "shadowsocks").await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn trojan_config(
    State(state): State<Arc<AppState>>,
    uri: Uri,
    headers: HeaderMap,
) -> Response<Body> {
    match build_trojan_config_response(&state, uri, headers).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn trojan_user(
    State(state): State<Arc<AppState>>,
    uri: Uri,
    headers: HeaderMap,
) -> Response<Body> {
    match build_trojan_user_response(&state, uri, headers).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn trojan_submit(
    State(state): State<Arc<AppState>>,
    uri: Uri,
    headers: HeaderMap,
    body: Body,
) -> Response<Body> {
    match build_legacy_submit_response(&state, uri, headers, body, "trojan").await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_shadowsocks_user_response(
    state: &AppState,
    uri: Uri,
    headers: HeaderMap,
) -> Result<Response<Body>, Response<Body>> {
    let server = authenticate_legacy_server(state, &uri, "shadowsocks").await?;
    touch_legacy_server_cache(
        state,
        "SHADOWSOCKS",
        server.parent_id.unwrap_or(server.id) as i64,
        "LAST_CHECK_AT",
        Utc::now().timestamp(),
    )
    .await;

    let users = load_legacy_available_users(state, &server).await.map_err(internal_error)?;
    let rotate_credentials =
        get_setting_bool(state, "rotate_subscription_credentials_daily", false).await;
    let result = users
        .into_iter()
        .map(|user| {
            json!({
                "id": user.id,
                "port": server.server_port,
                "cipher": normalized_legacy_protocol_settings(&server)
                    .get("cipher")
                    .and_then(Value::as_str)
                    .unwrap_or("aes-128-gcm"),
                "secret": build_legacy_server_password(
                    &server,
                    &user,
                    &state.app_key,
                    rotate_credentials,
                ),
            })
        })
        .collect::<Vec<_>>();

    let payload = json!({ "data": result });
    let etag = format!("{:x}", Sha1::digest(payload.to_string().as_bytes()));
    if headers
        .get("if-none-match")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.contains(&etag))
        .unwrap_or(false)
    {
        return Ok(Response::builder().status(StatusCode::NOT_MODIFIED).body(Body::empty()).unwrap());
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/json")
        .header(ETAG, format!("\"{}\"", etag))
        .body(Body::from(payload.to_string()))
        .unwrap())
}

async fn build_trojan_user_response(
    state: &AppState,
    uri: Uri,
    headers: HeaderMap,
) -> Result<Response<Body>, Response<Body>> {
    let server = authenticate_legacy_server(state, &uri, "trojan").await?;
    touch_legacy_server_cache(
        state,
        "TROJAN",
        server.parent_id.unwrap_or(server.id) as i64,
        "LAST_CHECK_AT",
        Utc::now().timestamp(),
    )
    .await;

    let users = load_legacy_available_users(state, &server).await.map_err(internal_error)?;
    let rotate_credentials =
        get_setting_bool(state, "rotate_subscription_credentials_daily", false).await;
    let result = users
        .into_iter()
        .map(|user| {
            json!({
                "id": user.id,
                "speed_limit": user.speed_limit,
                "device_limit": user.device_limit,
                "trojan_user": {
                    "password": build_legacy_server_password(
                        &server,
                        &user,
                        &state.app_key,
                        rotate_credentials,
                    ),
                }
            })
        })
        .collect::<Vec<_>>();

    let payload = json!({
        "msg": "ok",
        "data": result,
    });
    let etag = format!("{:x}", Sha1::digest(payload.to_string().as_bytes()));
    if headers
        .get("if-none-match")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.contains(&etag))
        .unwrap_or(false)
    {
        return Ok(Response::builder().status(StatusCode::NOT_MODIFIED).body(Body::empty()).unwrap());
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/json")
        .header(ETAG, format!("\"{}\"", etag))
        .body(Body::from(payload.to_string()))
        .unwrap())
}

async fn build_trojan_config_response(
    state: &AppState,
    uri: Uri,
    _headers: HeaderMap,
) -> Result<Response<Body>, Response<Body>> {
    let server = authenticate_legacy_server(state, &uri, "trojan").await?;
    let params = parse_query(&uri);
    let local_port = params
        .get("local_port")
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::BAD_REQUEST, "本地端口不能为空"))?;

    let settings = normalized_legacy_protocol_settings(&server);
    let server_name = settings
        .get("server_name")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(server.host.as_str());

    let payload = json!({
        "run_type": "server",
        "local_addr": "0.0.0.0",
        "local_port": server.server_port,
        "remote_addr": "www.taobao.com",
        "remote_port": 80,
        "password": [],
        "ssl": {
            "cert": "/root/.cert/server.crt",
            "key": "/root/.cert/server.key",
            "sni": server_name,
        },
        "api": {
            "enabled": true,
            "api_addr": "127.0.0.1",
            "api_port": local_port,
        }
    });

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap())
}

async fn build_legacy_submit_response(
    state: &AppState,
    uri: Uri,
    _headers: HeaderMap,
    body: Body,
    protocol: &str,
) -> Result<Response<Body>, Response<Body>> {
    let server = authenticate_legacy_server(state, &uri, protocol).await?;
    let payload = parse_json_body(body).await?;
    let mut rows = normalize_legacy_submit_payload(&payload)
        .filter(|rows| rows.len() <= 20_000)
        .ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "Invalid data format"))?;
    let allowed_user_ids = load_legacy_available_users(state, &server)
        .await
        .map_err(internal_error)?
        .into_iter()
        .map(|user| user.id)
        .collect::<HashSet<_>>();
    let reported_count = rows.len();
    rows.retain(|row| allowed_user_ids.contains(&row.user_id));
    if rows.len() != reported_count {
        warn!(
            server_id = server.id,
            dropped_rows = reported_count - rows.len(),
            "legacy traffic report contained users outside the server authorization scope",
        );
    }
    let mut replay_rows = rows
        .iter()
        .map(|row| (row.user_id, row.upload, row.download))
        .collect::<Vec<_>>();
    replay_rows.sort_unstable();
    let replay_fingerprint = sha256_hex(
        &serde_json::to_string(&replay_rows).unwrap_or_default(),
    );
    let replay_key = format!(
        "{}{}machine-replay:legacy:{}:{}:{}",
        state.redis_prefix,
        state.cache_prefix,
        server.id,
        protocol,
        replay_fingerprint,
    );
    match redis_set_nx_ex_raw(state, &replay_key, 10, "1").await {
        Ok(false) => {
            return Ok(json_value_response(json!({
                "ret": 1,
                "data": true,
            })))
        }
        Err(err) => warn!(server_id = server.id, error = %err, "legacy traffic replay guard unavailable"),
        Ok(true) => {}
    }
    let server_type_upper = protocol.to_ascii_uppercase();
    let server_id = server.parent_id.unwrap_or(server.id) as i64;
    touch_legacy_server_cache(state, &server_type_upper, server_id, "ONLINE_USER", rows.len() as i64).await;
    touch_legacy_server_cache(state, &server_type_upper, server_id, "LAST_PUSH_AT", Utc::now().timestamp()).await;
    if !rows.is_empty() {
        enqueue_legacy_submit_traffic(
            state,
            server.id,
            protocol,
            &server.rate,
            &rows,
        )
        .await
        .map_err(internal_error)?;
    }

    Ok(json_value_response(json!({
        "ret": 1,
        "msg": "ok",
    })))
}

async fn authenticate_legacy_server(
    state: &AppState,
    uri: &Uri,
    expected_type: &str,
) -> Result<LegacyServerCompatRow, Response<Body>> {
    let params = parse_query(uri);
    let token = params.get("token").cloned().unwrap_or_default();
    let node_id = params.get("node_id").cloned().unwrap_or_default();

    if token.trim().is_empty() || node_id.trim().is_empty() {
        return Err(json_error(StatusCode::UNPROCESSABLE_ENTITY, "Missing token or node_id"));
    }

    let configured = get_setting_string(state, "server_token", "").await;
    if configured.trim().len() < 24 {
        return Err(json_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "Legacy server authentication is not securely configured",
        ));
    }

    let query = String::from(
        "SELECT
            id, `type` AS server_type, parent_id, group_ids,
            CAST(rate AS CHAR) AS rate, host, server_port, protocol_settings, created_at
         FROM v2_server
         WHERE (id = ? OR code = ?) AND `type` = ? AND `show` = 1
         LIMIT 1"
    );
    let row = sqlx::query_as::<_, LegacyServerCompatRow>(&query)
        .bind(node_id.parse::<i64>().unwrap_or_default())
        .bind(&node_id)
        .bind(expected_type)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?;

    let row = row.ok_or_else(|| json_error(StatusCode::UNAUTHORIZED, "Invalid server token"))?;
    let expected = derive_legacy_server_token(&configured, &row.server_type, row.id)
        .ok_or_else(|| json_error(StatusCode::SERVICE_UNAVAILABLE, "Legacy server authentication is unavailable"))?;
    if !crate::secure_compare_support::constant_time_eq_str(&expected, token.trim()) {
        return Err(json_error(StatusCode::UNAUTHORIZED, "Invalid server token"));
    }
    Ok(row)
}

pub(crate) fn derive_legacy_server_token(
    master_token: &str,
    server_type: &str,
    server_id: u64,
) -> Option<String> {
    let master_token = master_token.trim();
    if master_token.len() < 24 || server_type.trim().is_empty() || server_id == 0 {
        return None;
    }
    let mut mac = HmacSha256::new_from_slice(master_token.as_bytes()).ok()?;
    mac.update(b"notxboard:legacy-server:v1\0");
    mac.update(server_type.trim().to_ascii_lowercase().as_bytes());
    mac.update(b"\0");
    mac.update(server_id.to_string().as_bytes());
    Some(
        mac.finalize()
            .into_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    )
}

async fn load_legacy_available_users(
    state: &AppState,
    server: &LegacyServerCompatRow,
) -> Result<Vec<LegacyAvailableUserRow>, sqlx::Error> {
    let group_ids = parse_json_i64_array(server.group_ids.as_ref().map(|value| &value.0));
    if group_ids.is_empty() {
        return Ok(Vec::new());
    }
    let now = Utc::now().timestamp();
    let placeholders = vec!["?"; group_ids.len()].join(",");
    let sql = format!(
        "SELECT id, uuid, COALESCE(subscription_credential_version, 0) AS subscription_credential_version,
                COALESCE(speed_limit, 0) AS speed_limit, COALESCE(device_limit, 0) AS device_limit
         FROM v2_user
         WHERE group_id IN ({})
           AND COALESCE(transfer_enable, 0) > COALESCE(u, 0) + COALESCE(d, 0)
           AND (expired_at IS NULL OR expired_at >= ?)
           AND banned = 0",
        placeholders
    );
    let mut query = sqlx::query_as::<_, LegacyAvailableUserRow>(&sql);
    for group_id in &group_ids {
        query = query.bind(*group_id);
    }
    query = query.bind(now);
    query.fetch_all(&state.db).await
}

fn normalized_legacy_protocol_settings(server: &LegacyServerCompatRow) -> Map<String, Value> {
    let raw_settings = server
        .protocol_settings
        .as_ref()
        .and_then(|json| json.0.as_object().cloned())
        .unwrap_or_default();
    normalized_protocol_settings(&server.server_type, raw_settings)
}

fn build_legacy_server_password(
    server: &LegacyServerCompatRow,
    user: &LegacyAvailableUserRow,
    app_key: &str,
    rotate_credentials: bool,
) -> String {
    let effective = effective_uuid(
        &user.uuid,
        user.subscription_credential_version,
        rotate_credentials,
    );
    if server.server_type != "shadowsocks" {
        return effective;
    }
    let settings = normalized_legacy_protocol_settings(server);
    let cipher = settings.get("cipher").and_then(Value::as_str).unwrap_or_default();
    match cipher {
        "2022-blake3-aes-128-gcm" => {
            let server_key = get_server_key(app_key, server.created_at, 16);
            let user_key = BASE64_STANDARD.encode(effective.as_bytes());
            format!("{}:{}", server_key, user_key)
        }
        "2022-blake3-aes-256-gcm" | "2022-blake3-chacha20-poly1305" => {
            let server_key = get_server_key(app_key, server.created_at, 32);
            let user_key = BASE64_STANDARD.encode(effective.as_bytes());
            format!("{}:{}", server_key, user_key)
        }
        _ => effective,
    }
}

fn normalize_legacy_submit_payload(value: &Value) -> Option<Vec<TrafficRow>> {
    let object = value.as_object()?;
    let mut rows = Vec::new();
    let mut batch_total = 0_i64;
    for (user_id, row) in object {
        let user_id = user_id.parse::<i64>().ok()?;
        let row_obj = row.as_object()?;
        let upload = row_obj.get("u").and_then(parse_i64_value).unwrap_or(0);
        let download = row_obj.get("d").and_then(parse_i64_value).unwrap_or(0);
        let total = upload.checked_add(download)?;
        if upload < 0
            || download < 0
            || upload > MAX_LEGACY_TRAFFIC_DELTA_KB
            || download > MAX_LEGACY_TRAFFIC_DELTA_KB
            || total <= 0
        {
            return None;
        }
        batch_total = batch_total.checked_add(total)?;
        if batch_total > MAX_LEGACY_TRAFFIC_BATCH_KB {
            return None;
        }
        rows.push(TrafficRow {
            user_id,
            upload,
            download,
        });
    }
    Some(rows)
}

async fn touch_legacy_server_cache(
    state: &AppState,
    server_type_upper: &str,
    server_id: i64,
    suffix: &str,
    value: i64,
) {
    let key = format!("SERVER_{}_{}_{}", server_type_upper, suffix, server_id);
    let _ = redis_setex_string(state, &key, 3600, &value.to_string()).await;
}

fn parse_json_i64_array(value: Option<&Value>) -> Vec<i64> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items.iter()
                .filter_map(parse_i64_value)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod authorization_tests {
    use super::derive_legacy_server_token;

    #[test]
    fn legacy_tokens_are_bound_to_one_server_subject() {
        let master = "0123456789abcdef0123456789abcdef";
        let first = derive_legacy_server_token(master, "trojan", 1).unwrap();
        let same = derive_legacy_server_token(master, "TROJAN", 1).unwrap();
        let other_id = derive_legacy_server_token(master, "trojan", 2).unwrap();
        let other_type = derive_legacy_server_token(master, "shadowsocks", 1).unwrap();

        assert_eq!(first, same);
        assert_ne!(first, other_id);
        assert_ne!(first, other_type);
        assert_eq!(first.len(), 64);
    }

    #[test]
    fn legacy_token_derivation_rejects_weak_or_ambiguous_subjects() {
        assert!(derive_legacy_server_token("short", "trojan", 1).is_none());
        assert!(derive_legacy_server_token("0123456789abcdef01234567", "", 1).is_none());
        assert!(derive_legacy_server_token("0123456789abcdef01234567", "trojan", 0).is_none());
    }
}
