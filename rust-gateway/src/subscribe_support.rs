use crate::*;

pub(crate) async fn load_subscribe_servers_for_user(
    state: &AppState,
    user: &UserRow,
) -> Result<Vec<ServerNodeRow>, sqlx::Error> {
    let mut merged = load_available_server_nodes_for_user(
        state,
        user.id,
        user.trust_level.unwrap_or(0),
        user.is_super_admin.unwrap_or(0) != 0,
    )
    .await?;
    let legacy = load_legacy_subscribe_servers_for_user(state, user).await?;
    let mut seen = merged
        .iter()
        .map(|server| format!("server_node:{}:{}", server.id, server.protocol))
        .collect::<HashSet<_>>();

    for server in legacy {
        let key = format!("legacy:{}:{}", server.id, server.protocol);
        if seen.insert(key) {
            merged.push(server);
        }
    }

    Ok(merged)
}

pub(crate) async fn subscribe_user_is_available(
    state: &AppState,
    user: &UserRow,
) -> Result<bool, sqlx::Error> {
    let active = user.banned.unwrap_or(0) == 0
        && user.transfer_enable.unwrap_or(0) > 0
        && user
            .expired_at
            .map(|expired_at| expired_at > Utc::now().timestamp())
            .unwrap_or(true);
    if active || user.is_super_admin.unwrap_or(0) != 0 {
        return Ok(true);
    }

    let accessible = load_subscribe_servers_for_user(state, user).await?;
    Ok(!accessible.is_empty())
}

pub(crate) fn apply_protocol_prefixes_to_subscribe_servers(servers: &mut [ServerNodeRow]) {
    for server in servers {
        let settings = server
            .settings
            .as_ref()
            .and_then(|json| json.0.as_object());
        if let Some(prefix) = subscribe_protocol_prefix(&server.protocol, settings) {
            server.name = format!("{}{}", prefix, server.name);
        }
    }
}

pub(crate) async fn prepend_subscribe_info_nodes(
    servers: &mut Vec<ServerNodeRow>,
    user: &UserRow,
    reject_server_count: i64,
    state: &AppState,
    show_info: bool,
) {
    if servers.is_empty() {
        return;
    }

    let template = servers[0].clone();

    if show_info {
        let mut expire = template.clone();
        expire.name = format!(
            "套餐到期：{}",
            user.expired_at
                .and_then(|ts| chrono::DateTime::from_timestamp(ts, 0))
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "长期有效".to_string())
        );
        servers.insert(0, expire);

        if let Some(reset_days) = subscribe_reset_days(state, user).await {
            if reset_days > 0 {
                let mut reset = template.clone();
                reset.name = format!("距离下次重置剩余：{} 天", reset_days);
                servers.insert(0, reset);
            }
        }

        let mut remaining = template.clone();
        let used = user.u.unwrap_or(0) + user.d.unwrap_or(0);
        let total = user.transfer_enable.unwrap_or(0);
        remaining.name = format!("剩余流量：{}", traffic_convert((total - used).max(0)));
        servers.insert(0, remaining);
    }

    if reject_server_count > 0 {
        let mut filtered = template;
        filtered.name = format!("过滤掉{}条线路", reject_server_count);
        servers.insert(0, filtered);
    }
}

pub(crate) fn subscribe_shadowsocks_password(
    uuid: &str,
    server: &ServerNodeRow,
    settings: &Map<String, Value>,
    app_key: &str,
) -> String {
    if let Some(password) = settings.get("legacy_password").and_then(Value::as_str) {
        return password.to_string();
    }

    let cipher = json_string(settings, "cipher", "aes-128-gcm");
    if let Some(length) = subscribe_cipher_server_key_len(&cipher) {
        let server_key = get_server_key(app_key, server.created_at, length);
        let user_key = BASE64_STANDARD.encode(uuid.as_bytes());
        return format!("{}:{}", server_key, user_key);
    }

    uuid.to_string()
}

async fn load_legacy_subscribe_servers_for_user(
    state: &AppState,
    user: &UserRow,
) -> Result<Vec<ServerNodeRow>, sqlx::Error> {
    let group_id = user.group_id;
    let Some(group_id) = group_id else {
        return Ok(Vec::new());
    };

    let rows = sqlx::query_as::<_, LegacySubscribeServerRow>(
        "SELECT
            id,
            `type` AS server_type,
            parent_id,
            group_ids,
            name,
            host,
            port,
            server_port,
            protocol_settings,
            created_at
         FROM v2_server
         WHERE `show` = 1
           AND JSON_CONTAINS(COALESCE(group_ids, JSON_ARRAY()), CAST(? AS JSON), '$')
         ORDER BY sort ASC, id ASC"
    )
    .bind(group_id)
    .fetch_all(&state.db)
    .await?;

    let rotate_credentials = get_setting_bool(state, "rotate_subscription_credentials_daily", false).await;
    let effective_uuid_value = effective_uuid(
        user.uuid.as_deref().unwrap_or_default(),
        user.subscription_credential_version.unwrap_or(0),
        rotate_credentials,
    );

    let mut result = Vec::new();
    for row in rows {
        if !legacy_server_is_available(state, &row).await {
            continue;
        }

        let normalized_type = normalize_type(&row.server_type).unwrap_or_else(|| row.server_type.clone());
        let raw_settings = row
            .protocol_settings
            .as_ref()
            .and_then(|json| json.0.as_object().cloned())
            .unwrap_or_default();
        let mut settings = normalized_protocol_settings(&normalized_type, raw_settings);

        if normalized_type == "shadowsocks" {
            let cipher = settings.get("cipher").and_then(Value::as_str).unwrap_or_default();
            let password = build_legacy_subscribe_password(&row, cipher, &effective_uuid_value, &state.app_key);
            settings.insert("legacy_password".to_string(), Value::String(password));
        }

        let port = parse_legacy_server_port(&row.port);
        result.push(ServerNodeRow {
            id: row.id.max(0) as u64,
            user_id: 0,
            name: row.name.clone(),
            host: row.host.clone(),
            port,
            service_port: Some(row.server_port),
            protocol: normalized_type,
            settings: Some(SqlxJson(Value::Object(settings))),
            access_control: None,
            device_limit: 0,
            connection_limit: 0,
            speed_limit_down: 0,
            created_at: row.created_at,
        });
    }

    Ok(result)
}

fn subscribe_protocol_prefix(
    protocol: &str,
    settings: Option<&Map<String, Value>>,
) -> Option<&'static str> {
    match normalize_type(protocol).unwrap_or_default().as_str() {
        "hysteria" => {
            let version = settings
                .and_then(|map| map.get("version"))
                .and_then(parse_i64_value)
                .unwrap_or(1);
            if version == 2 {
                Some("[Hy2]")
            } else {
                Some("[Hy]")
            }
        }
        "vless" => Some("[vless]"),
        "shadowsocks" => Some("[ss]"),
        "vmess" => Some("[vmess]"),
        "trojan" => Some("[trojan]"),
        "tuic" => Some("[tuic]"),
        "socks" => Some("[socks]"),
        "anytls" => Some("[anytls]"),
        "http" => Some("[http]"),
        "naive" => Some("[naive]"),
        "mieru" => Some("[mieru]"),
        _ => None,
    }
}

async fn subscribe_reset_days(
    state: &AppState,
    user: &UserRow,
) -> Option<i64> {
    let plan_id = sqlx::query_scalar::<_, Option<i64>>("SELECT plan_id FROM v2_user WHERE id = ? LIMIT 1")
        .bind(user.id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .flatten()?;
    let plan = load_plan_reset_info(state, plan_id).await.ok().flatten()?;
    let expired_at = sqlx::query_scalar::<_, Option<i64>>("SELECT expired_at FROM v2_user WHERE id = ? LIMIT 1")
        .bind(user.id)
        .fetch_optional(&state.db)
        .await
        .ok()
        .flatten()
        .flatten();
    let user_stub = BearerUserRow {
        id: user.id,
        invite_user_id: None,
        email: String::new(),
        transfer_enable: user.transfer_enable.unwrap_or(0),
        last_login_at: None,
        created_at: 0,
        banned: user.banned.unwrap_or(0),
        ban_reason: None,
        remind_expire: 0,
        remind_traffic: 0,
        expired_at,
        balance: 0,
        commission_balance: 0,
        plan_id: Some(plan_id),
        group_id: user.group_id,
        discount: None,
        commission_rate: None,
        telegram_id: None,
        uuid: user.uuid.clone().unwrap_or_default(),
        is_admin: 0,
        is_super_admin: user.is_super_admin.unwrap_or(0),
        trust_level: user.trust_level.unwrap_or(0),
        is_silenced: user.is_silenced.unwrap_or(0),
        linux_do_id: None,
        linux_do_username: None,
        linux_do_name: None,
        linux_do_avatar: None,
        api_key: None,
        concurrent_ip_limit: 0,
        token: user.token.clone().unwrap_or_default(),
        subscribe_path: None,
        subscribe_key: None,
        subscribe_salt: None,
        u: user.u.unwrap_or(0),
        d: user.d.unwrap_or(0),
        device_limit: None,
        speed_limit: None,
        next_reset_at: None,
    };
    let next_reset = calculate_next_reset_at_for_user_plan(state, &user_stub, &plan, Utc::now().timestamp()).await?;
    let now = Utc::now().timestamp();
    if next_reset <= now {
        return Some(0);
    }
    Some((((next_reset - now) as f64) / 86_400.0).ceil() as i64)
}

pub(crate) fn traffic_convert(value: i64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    let value = value.max(0) as f64;
    if value > GB {
        format!("{:.2} GB", value / GB)
    } else if value > MB {
        format!("{:.2} MB", value / MB)
    } else if value > KB {
        format!("{:.2} KB", value / KB)
    } else {
        format!("{:.2} B", value)
    }
}

fn subscribe_cipher_server_key_len(cipher: &str) -> Option<usize> {
    match cipher {
        "2022-blake3-aes-128-gcm" => Some(16),
        "2022-blake3-aes-256-gcm" | "2022-blake3-chacha20-poly1305" => Some(32),
        _ => None,
    }
}

fn parse_legacy_server_port(raw: &str) -> i64 {
    let raw = raw.trim();
    if raw.is_empty() {
        return 0;
    }
    if raw.contains('-') {
        let parts = raw
            .split('-')
            .filter_map(|part| part.trim().parse::<i64>().ok())
            .collect::<Vec<_>>();
        if parts.len() >= 2 {
            let start = parts[0].min(parts[1]);
            let end = parts[0].max(parts[1]);
            if start > 0 && end >= start {
                return if start == end {
                    start
                } else {
                    let span = (end - start + 1) as u64;
                    let mut bytes = [0_u8; 8];
                    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
                        let _ = file.read_exact(&mut bytes);
                    }
                    let seed = u64::from_le_bytes(bytes);
                    start + (seed % span) as i64
                };
            }
        }
        return parts.first().copied().unwrap_or(0);
    }
    raw.parse::<i64>().unwrap_or(0)
}

fn build_legacy_subscribe_password(
    server: &LegacySubscribeServerRow,
    cipher: &str,
    effective_uuid_value: &str,
    app_key: &str,
) -> String {
    if let Some(length) = subscribe_cipher_server_key_len(cipher) {
        let server_key = get_server_key(app_key, server.created_at, length);
        let user_key = BASE64_STANDARD.encode(effective_uuid_value.as_bytes());
        return format!("{}:{}", server_key, user_key);
    }

    effective_uuid_value.to_string()
}
