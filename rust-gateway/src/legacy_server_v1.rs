use crate::*;
use crate::legacy_traffic_support::enqueue_legacy_submit_traffic;

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
                "secret": build_legacy_server_password(&server, &user, &state.app_key),
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
    let result = users
        .into_iter()
        .map(|user| {
            json!({
                "id": user.id,
                "speed_limit": user.speed_limit,
                "device_limit": user.device_limit,
                "trojan_user": {
                    "password": build_legacy_server_password(&server, &user, &state.app_key),
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
    let rows = normalize_legacy_submit_payload(&payload).ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "Invalid data format"))?;
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
    if configured.trim().is_empty() || configured != token {
        return Err(json_error(StatusCode::UNAUTHORIZED, "Invalid server token"));
    }

    let query = String::from(
        "SELECT
            id, `type` AS server_type, parent_id, group_ids,
            CAST(rate AS CHAR) AS rate, host, server_port, protocol_settings, created_at
         FROM v2_server
         WHERE (id = ? OR code = ?) AND `type` = ?
         LIMIT 1"
    );
    let row = sqlx::query_as::<_, LegacyServerCompatRow>(&query)
        .bind(node_id.parse::<i64>().unwrap_or_default())
        .bind(&node_id)
        .bind(expected_type)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?;

    row.ok_or_else(|| json_error(StatusCode::UNAUTHORIZED, "Invalid server token"))
}

async fn load_legacy_available_users(
    state: &AppState,
    server: &LegacyServerCompatRow,
) -> Result<Vec<BearerUserRow>, sqlx::Error> {
    let group_ids = parse_json_i64_array(server.group_ids.as_ref().map(|value| &value.0));
    if group_ids.is_empty() {
        return Ok(Vec::new());
    }
    let now = Utc::now().timestamp();
    let placeholders = vec!["?"; group_ids.len()].join(",");
    let sql = format!(
        "SELECT id, invite_user_id, email, transfer_enable, last_login_at, created_at, banned, ban_reason,
                remind_expire, remind_traffic, expired_at, balance, commission_balance, plan_id,
                discount, commission_rate, telegram_id, uuid, is_admin, is_super_admin, trust_level,
                is_silenced, linux_do_id, linux_do_username, linux_do_name, linux_do_avatar, api_key,
                concurrent_ip_limit, token, subscribe_path, subscribe_key, subscribe_salt, u, d,
                device_limit, speed_limit, next_reset_at
         FROM v2_user
         WHERE group_id IN ({})
           AND COALESCE(transfer_enable, 0) > COALESCE(u, 0) + COALESCE(d, 0)
           AND (expired_at IS NULL OR expired_at >= ?)
           AND banned = 0",
        placeholders
    );
    let mut query = sqlx::query_as::<_, BearerUserRow>(&sql);
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
    user: &BearerUserRow,
    app_key: &str,
) -> String {
    let rotate_credentials = false;
    let effective = effective_uuid(&user.uuid, 0, rotate_credentials);
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
    for (user_id, row) in object {
        let user_id = user_id.parse::<i64>().ok()?;
        let row_obj = row.as_object()?;
        let upload = row_obj.get("u").and_then(parse_i64_value).unwrap_or(0);
        let download = row_obj.get("d").and_then(parse_i64_value).unwrap_or(0);
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
