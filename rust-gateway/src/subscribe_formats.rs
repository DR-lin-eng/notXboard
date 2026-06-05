use crate::*;

pub(crate) async fn build_surge_config_payload(
    state: &AppState,
    user: &UserRow,
    servers: &[ServerNodeRow],
    uuid: &str,
) -> Result<String, Response<Body>> {
    let app_name = get_setting_string(state, "app_name", "Portal").await;
    let subs_link = subscribe_link_placeholder();
    let subs_domain = subscribe_host_placeholder();

    let mut proxies = String::new();
    let mut proxy_names = Vec::new();
    for server in servers {
        let normalized_type = normalize_type(&server.protocol).unwrap_or_default();
        let settings = normalized_protocol_settings(
            &normalized_type,
            server
                .settings
                .as_ref()
                .and_then(|json| json.0.as_object().cloned())
                .unwrap_or_default(),
        );

        let line = match normalized_type.as_str() {
            "shadowsocks" => {
                let cipher = json_string(&settings, "cipher", "aes-128-gcm");
                if !matches!(
                    cipher.as_str(),
                    "aes-128-gcm" | "aes-192-gcm" | "aes-256-gcm" | "chacha20-ietf-poly1305"
                ) {
                    String::new()
                } else {
                    build_surge_shadowsocks(uuid, server, &settings, &state.app_key)
                }
            }
            "vmess" => build_surge_vmess(uuid, server, &settings),
            "trojan" => build_surge_trojan(uuid, server, &settings),
            "hysteria" => build_surge_hysteria(uuid, server, &settings),
            _ => String::new(),
        };

        if !line.is_empty() {
            proxies.push_str(&line);
            proxy_names.push(server.name.clone());
        }
    }

    let proxy_group = proxy_names.join(", ");
    let custom_surge = crate::runtime_paths::resources_path("rules/custom.surge.conf");
    let default_surge = crate::runtime_paths::resources_path("rules/default.surge.conf");
    let template_path = if custom_surge.exists() {
        custom_surge
    } else {
        default_surge
    };
    let mut config = std::fs::read_to_string(&template_path)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "load surge config failed"))?;
    config = config.replace("$subs_link", &subs_link);
    config = config.replace("$subs_domain", &subs_domain);
    config = config.replace("$proxies", &proxies);
    config = config.replace("$proxy_group", &proxy_group);
    config = config.replace("$subscribe_info", &surge_panel_subscribe_info(&app_name, user));

    Ok(config)
}

pub(crate) async fn build_surfboard_config_payload(
    state: &AppState,
    user: &UserRow,
    servers: &[ServerNodeRow],
    uuid: &str,
) -> Result<String, Response<Body>> {
    let app_name = get_setting_string(state, "app_name", "Portal").await;
    let subs_link = subscribe_link_placeholder();
    let subs_domain = subscribe_host_placeholder();

    let mut proxies = String::new();
    let mut proxy_names = Vec::new();
    for server in servers {
        let normalized_type = normalize_type(&server.protocol).unwrap_or_default();
        let settings = normalized_protocol_settings(
            &normalized_type,
            server
                .settings
                .as_ref()
                .and_then(|json| json.0.as_object().cloned())
                .unwrap_or_default(),
        );

        let line = match normalized_type.as_str() {
            "shadowsocks" => {
                let cipher = json_string(&settings, "cipher", "aes-128-gcm");
                if !matches!(
                    cipher.as_str(),
                    "aes-128-gcm" | "aes-192-gcm" | "aes-256-gcm" | "chacha20-ietf-poly1305"
                ) {
                    String::new()
                } else {
                    build_surfboard_shadowsocks(uuid, server, &settings, &state.app_key)
                }
            }
            "vmess" => build_surfboard_vmess(uuid, server, &settings),
            "trojan" => build_surfboard_trojan(uuid, server, &settings),
            _ => String::new(),
        };

        if !line.is_empty() {
            proxies.push_str(&line);
            proxy_names.push(server.name.clone());
        }
    }

    let proxy_group = proxy_names.join(", ");
    let template_path = crate::runtime_paths::resources_path("rules/default.surfboard.conf");
    let mut config = std::fs::read_to_string(&template_path)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "load surfboard config failed"))?;
    config = config.replace("$subs_link", &subs_link);
    config = config.replace("$subs_domain", &subs_domain);
    config = config.replace("$proxies", &proxies);
    config = config.replace("$proxy_group", &proxy_group);
    config = config.replace("$subscribe_info", &surfboard_panel_subscribe_info(&app_name, user));

    Ok(config)
}

pub(crate) fn build_shadowsocks_sip008_payload(
    state: &AppState,
    user: &UserRow,
    servers: &[ServerNodeRow],
    uuid: &str,
) -> String {
    let rows = servers
        .iter()
        .filter_map(|server| {
            let normalized_type = normalize_type(&server.protocol).unwrap_or_default();
            if normalized_type != "shadowsocks" {
                return None;
            }
            let settings = normalized_protocol_settings(
                &normalized_type,
                server
                    .settings
                    .as_ref()
                    .and_then(|json| json.0.as_object().cloned())
                    .unwrap_or_default(),
            );
            let cipher = json_string(&settings, "cipher", "aes-128-gcm");
            if !matches!(
                cipher.as_str(),
                "aes-128-gcm" | "aes-192-gcm" | "aes-256-gcm" | "chacha20-ietf-poly1305"
            ) {
                return None;
            }
            Some(json!({
                "id": server.id,
                "remarks": server.name,
                "server": server.host,
                "server_port": server.port,
                "password": subscribe_shadowsocks_password(uuid, server, &settings, &state.app_key),
                "method": cipher,
            }))
        })
        .collect::<Vec<_>>();

    json!({
        "version": 1,
        "bytes_used": user.u.unwrap_or(0) + user.d.unwrap_or(0),
        "bytes_remaining": (user.transfer_enable.unwrap_or(0) - (user.u.unwrap_or(0) + user.d.unwrap_or(0))).max(0),
        "servers": rows,
    })
    .to_string()
}

pub(crate) async fn build_stash_yaml_payload(
    state: &AppState,
    user: &UserRow,
    servers: &[ServerNodeRow],
    uuid: &str,
) -> Result<String, Response<Body>> {
    let app_name = get_setting_string(state, "app_name", "Portal").await;
    let custom_stash = crate::runtime_paths::resources_path("rules/custom.stash.yaml");
    let custom_clash = crate::runtime_paths::resources_path("rules/custom.clash.yaml");
    let default_clash = crate::runtime_paths::resources_path("rules/default.clash.yaml");
    let template_path = if custom_stash.exists() {
        custom_stash
    } else if custom_clash.exists() {
        custom_clash
    } else {
        default_clash
    };
    let raw = std::fs::read_to_string(&template_path)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "load stash config failed"))?;
    let mut config = serde_yaml::from_str::<serde_yaml::Value>(&raw)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "invalid stash template"))?;

    let mut proxies = Vec::new();
    let mut proxy_names = Vec::new();
    for server in servers {
        let normalized_type = normalize_type(&server.protocol).unwrap_or_default();
        let settings = normalized_protocol_settings(
            &normalized_type,
            server
                .settings
                .as_ref()
                .and_then(|json| json.0.as_object().cloned())
                .unwrap_or_default(),
        );

        let proxy = match normalized_type.as_str() {
            "shadowsocks" => Some(build_stash_shadowsocks(uuid, server, &settings, &state.app_key)),
            "vmess" => Some(build_stash_vmess(uuid, server, &settings)),
            "vless" => build_stash_vless(uuid, server, &settings),
            "hysteria" => Some(build_stash_hysteria(uuid, server, &settings)),
            "trojan" => Some(build_stash_trojan(uuid, server, &settings)),
            "tuic" => Some(build_stash_tuic(uuid, server, &settings)),
            "socks" => Some(build_stash_socks5(uuid, server, &settings)),
            "http" => Some(build_stash_http(uuid, server, &settings)),
            _ => None,
        };

        if let Some(proxy) = proxy {
            proxies.push(proxy);
            proxy_names.push(server.name.clone());
        }
    }

    if let Some(map) = config.as_mapping_mut() {
        let proxies_key = serde_yaml::Value::String("proxies".to_string());
        let existing_proxies = map
            .get(&proxies_key)
            .and_then(|value| value.as_sequence())
            .cloned()
            .unwrap_or_default();
        let mut merged_proxies = existing_proxies;
        merged_proxies.extend(
            proxies
                .iter()
                .filter_map(|proxy| serde_yaml::to_value(proxy).ok())
        );
        map.insert(proxies_key, serde_yaml::Value::Sequence(merged_proxies));

        if let Some(groups) = map
            .get_mut(serde_yaml::Value::String("proxy-groups".to_string()))
            .and_then(|value| value.as_sequence_mut())
        {
            for group in groups.iter_mut() {
                if let Some(group_map) = group.as_mapping_mut() {
                    if let Some(group_proxies) = group_map
                        .get_mut(serde_yaml::Value::String("proxies".to_string()))
                        .and_then(|value| value.as_sequence_mut())
                    {
                        let raw_items = group_proxies
                            .iter()
                            .filter_map(|value| value.as_str().map(|item| item.to_string()))
                            .collect::<Vec<_>>();
                        let has_regex = raw_items.iter().any(|item| is_stash_regex(item));
                        if has_regex {
                            let mut expanded = Vec::new();
                            for item in raw_items {
                                if is_stash_regex(&item) {
                                    for proxy_name in &proxy_names {
                                        if stash_regex_matches(&item, proxy_name) {
                                            expanded.push(serde_yaml::Value::String(proxy_name.clone()));
                                        }
                                    }
                                } else {
                                    expanded.push(serde_yaml::Value::String(item));
                                }
                            }
                            *group_proxies = expanded;
                        } else {
                            group_proxies.extend(
                                proxy_names
                                    .iter()
                                    .cloned()
                                    .map(serde_yaml::Value::String)
                            );
                        }
                    }
                }
            }
            groups.retain(|group| {
                group.as_mapping()
                    .and_then(|value| value.get(serde_yaml::Value::String("proxies".to_string())))
                    .and_then(|value| value.as_sequence())
                    .map(|items| !items.is_empty())
                    .unwrap_or(true)
            });
        }

        if let Some(rules) = map
            .get_mut(serde_yaml::Value::String("rules".to_string()))
            .and_then(|value| value.as_sequence_mut())
        {
            rules.insert(0, serde_yaml::Value::String("DOMAIN,$subs_domain,DIRECT".to_string()));
        }
    }

    let mut out = serde_yaml::to_string(&config)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "yaml serialize failed"))?;
    out = out.replace("$app_name", &app_name);
    if let Some(header) = build_userinfo_header(user) {
        out = format!("{}\n{}", header, out);
    }
    Ok(out)
}

pub(crate) fn build_loon_payload(
    state: &AppState,
    user: &UserRow,
    servers: &[ServerNodeRow],
    uuid: &str,
) -> String {
    let mut buffer = String::new();
    for server in servers {
        let normalized_type = normalize_type(&server.protocol).unwrap_or_default();
        let settings = normalized_protocol_settings(
            &normalized_type,
            server
                .settings
                .as_ref()
                .and_then(|json| json.0.as_object().cloned())
                .unwrap_or_default(),
        );

        let line = match normalized_type.as_str() {
            "shadowsocks" => build_loon_shadowsocks(uuid, server, &settings, &state.app_key),
            "vmess" => build_loon_vmess(uuid, server, &settings),
            "trojan" => build_loon_trojan(uuid, server, &settings),
            "hysteria" => build_loon_hysteria(uuid, server, &settings, user),
            _ => String::new(),
        };
        if !line.is_empty() {
            buffer.push_str(&line);
        }
    }
    buffer
}

fn build_loon_shadowsocks(
    uuid: &str,
    server: &ServerNodeRow,
    settings: &Map<String, Value>,
    app_key: &str,
) -> String {
    let cipher = json_string(settings, "cipher", "aes-128-gcm");
    let password = subscribe_shadowsocks_password(uuid, server, settings, app_key);
    let mut parts = vec![
        format!("{}=Shadowsocks", server.name),
        server.host.clone(),
        server.port.to_string(),
        cipher,
        password,
        "fast-open=false".to_string(),
        "udp=true".to_string(),
    ];

    if let Some(plugin) = settings.get("plugin").and_then(Value::as_str) {
        if plugin == "obfs" {
            if let Some(opts) = settings.get("plugin_opts").and_then(Value::as_str) {
                let parsed = parse_plugin_opts(opts);
                if let Some(obfs) = parsed.get("obfs") {
                    parts.push(format!("obfs-name={}", obfs));
                }
                if let Some(host) = parsed.get("obfs-host") {
                    parts.push(format!("obfs-host={}", host));
                }
                if let Some(path) = parsed.get("path") {
                    parts.push(format!("obfs-uri={}", path));
                }
            }
        }
    }

    format!("{}\r\n", parts.join(","))
}

fn build_loon_vmess(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> String {
    let mut parts = vec![
        format!("{}=vmess", server.name),
        server.host.clone(),
        server.port.to_string(),
        "auto".to_string(),
        uuid.to_string(),
        "fast-open=false".to_string(),
        "udp=true".to_string(),
        "alterId=0".to_string(),
    ];

    if json_int(settings, "tls") > 0 {
        if json_string(settings, "network", "tcp") == "tcp" {
            parts.push("over-tls=true".to_string());
        }
        if let Some(tls_settings) = settings.get("tls_settings").and_then(Value::as_object) {
            parts.push(format!(
                "skip-cert-verify={}",
                tls_settings
                    .get("allow_insecure")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                    .to_string()
            ));
            if let Some(server_name) = tls_settings.get("server_name").and_then(Value::as_str) {
                parts.push(format!("tls-name={}", server_name));
            }
        }
    }

    match json_string(settings, "network", "tcp").as_str() {
        "tcp" => {
            if let Some(network_settings) = settings.get("network_settings").and_then(Value::as_object) {
                if let Some(header_type) = network_settings
                    .get("header")
                    .and_then(Value::as_object)
                    .and_then(|header| header.get("type"))
                    .and_then(Value::as_str)
                    .filter(|value| *value != "none")
                {
                    if let Some(transport) = parts.iter_mut().find(|item| *item == "alterId=0") {
                        let _ = transport;
                    }
                    parts.push(format!("transport={}", header_type));
                    if let Some(path) = pick_path_from_request_header(network_settings) {
                        parts.push(format!("path={}", path));
                    }
                    if let Some(host) = pick_host_from_request_header(network_settings) {
                        parts.push(format!("host={}", host));
                    }
                } else {
                    parts.push("transport=tcp".to_string());
                }
            } else {
                parts.push("transport=tcp".to_string());
            }
        }
        "ws" => {
            parts.push("transport=ws".to_string());
            if let Some(path) = settings
                .get("network_settings")
                .and_then(Value::as_object)
                .and_then(|value| value.get("path"))
                .and_then(Value::as_str)
            {
                parts.push(format!("path={}", path));
            }
            if let Some(host) = settings
                .get("network_settings")
                .and_then(Value::as_object)
                .and_then(|value| value.get("headers"))
                .and_then(Value::as_object)
                .and_then(|value| value.get("Host"))
                .and_then(Value::as_str)
            {
                parts.push(format!("host={}", host));
            }
        }
        _ => {}
    }

    format!("{}\r\n", parts.join(","))
}

fn build_loon_trojan(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> String {
    let mut parts = vec![
        format!("{}=trojan", server.name),
        server.host.clone(),
        server.port.to_string(),
        uuid.to_string(),
        "fast-open=false".to_string(),
        "udp=true".to_string(),
    ];
    if let Some(server_name) = settings.get("server_name").and_then(Value::as_str) {
        parts.push(format!("tls-name={}", server_name));
    }
    if settings
        .get("allow_insecure")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        parts.push("skip-cert-verify=true".to_string());
    }
    format!("{}\r\n", parts.join(","))
}

fn build_loon_hysteria(
    uuid: &str,
    server: &ServerNodeRow,
    settings: &Map<String, Value>,
    user: &UserRow,
) -> String {
    if json_int(settings, "version") != 2 {
        return String::new();
    }
    let mut parts = vec![
        format!("{}=Hysteria2", server.name),
        server.host.clone(),
        server.port.to_string(),
        uuid.to_string(),
        settings
            .get("tls")
            .and_then(Value::as_object)
            .and_then(|tls| tls.get("server_name"))
            .and_then(Value::as_str)
            .map(|value| format!("sni={}", value))
            .unwrap_or_else(|| "(null)".to_string()),
    ];
    if settings
        .get("tls")
        .and_then(Value::as_object)
        .and_then(|tls| tls.get("allow_insecure"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        parts.push("skip-cert-verify=true".to_string());
    }
    if let Some(download_bandwidth) = settings
        .get("bandwidth")
        .and_then(Value::as_object)
        .and_then(|value| value.get("download_bandwidth"))
        .and_then(parse_i64_value)
    {
        parts.push(format!("download-bandwidth={}", download_bandwidth));
    } else {
        let total_gb = traffic_to_gb(user.transfer_enable.unwrap_or(0));
        parts.push(format!("download-bandwidth={}", total_gb));
    }
    parts.push("udp=true".to_string());
    format!("{}\r\n", parts.join(","))
}

fn build_surge_shadowsocks(
    uuid: &str,
    server: &ServerNodeRow,
    settings: &Map<String, Value>,
    app_key: &str,
) -> String {
    let cipher = json_string(settings, "cipher", "aes-128-gcm");
    let password = subscribe_shadowsocks_password(uuid, server, settings, app_key);
    let mut parts = vec![
        format!("{}=ss", server.name),
        server.host.clone(),
        server.port.to_string(),
        format!("encrypt-method={}", cipher),
        format!("password={}", password),
        "tfo=true".to_string(),
        "udp-relay=true".to_string(),
    ];
    append_plain_obfs_opts(settings, &mut parts);
    format!("{}\r\n", parts.join(","))
}

fn build_surge_vmess(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> String {
    let mut parts = vec![
        format!("{}=vmess", server.name),
        server.host.clone(),
        server.port.to_string(),
        format!("username={}", uuid),
        "vmess-aead=true".to_string(),
        "tfo=true".to_string(),
        "udp-relay=true".to_string(),
    ];

    if json_int(settings, "tls") > 0 {
        parts.push("tls=true".to_string());
        if let Some(tls_settings) = settings.get("tls_settings").and_then(Value::as_object) {
            if let Some(allow_insecure) = tls_settings.get("allow_insecure").and_then(Value::as_bool) {
                if allow_insecure {
                    parts.push("skip-cert-verify=true".to_string());
                }
            }
            if let Some(server_name) = tls_settings.get("server_name").and_then(Value::as_str) {
                parts.push(format!("sni={}", server_name));
            }
        }
    }

    if json_string(settings, "network", "tcp") == "ws" {
        parts.push("ws=true".to_string());
        if let Some(path) = settings
            .get("network_settings")
            .and_then(Value::as_object)
            .and_then(|value| value.get("path"))
            .and_then(Value::as_str)
        {
            parts.push(format!("ws-path={}", path));
        }
        if let Some(host) = settings
            .get("network_settings")
            .and_then(Value::as_object)
            .and_then(|value| value.get("headers"))
            .and_then(Value::as_object)
            .and_then(|value| value.get("Host"))
            .and_then(Value::as_str)
        {
            parts.push(format!("ws-headers=Host:{}", host));
        }
    }

    format!("{}\r\n", parts.join(","))
}

fn build_surge_trojan(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> String {
    let mut parts = vec![
        format!("{}=trojan", server.name),
        server.host.clone(),
        server.port.to_string(),
        format!("password={}", uuid),
        "tfo=true".to_string(),
        "udp-relay=true".to_string(),
    ];
    if let Some(server_name) = settings.get("server_name").and_then(Value::as_str) {
        parts.push(format!("sni={}", server_name));
    }
    if settings
        .get("allow_insecure")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        parts.push("skip-cert-verify=true".to_string());
    }
    format!("{}\r\n", parts.join(","))
}

fn build_surge_hysteria(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> String {
    if json_int(settings, "version") != 2 {
        return String::new();
    }
    let mut parts = vec![
        format!("{}=hysteria2", server.name),
        server.host.clone(),
        server.port.to_string(),
        format!("password={}", uuid),
        "udp-relay=true".to_string(),
    ];
    if let Some(server_name) = settings
        .get("tls")
        .and_then(Value::as_object)
        .and_then(|tls| tls.get("server_name"))
        .and_then(Value::as_str)
    {
        parts.push(format!("sni={}", server_name));
    }
    if let Some(up) = settings
        .get("bandwidth")
        .and_then(Value::as_object)
        .and_then(|value| value.get("up"))
        .and_then(parse_i64_value)
    {
        parts.push(format!("upload-bandwidth={}", up));
    }
    if let Some(down) = settings
        .get("bandwidth")
        .and_then(Value::as_object)
        .and_then(|value| value.get("down"))
        .and_then(parse_i64_value)
    {
        parts.push(format!("download-bandwidth={}", down));
    }
    if settings
        .get("tls")
        .and_then(Value::as_object)
        .and_then(|tls| tls.get("allow_insecure"))
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        parts.push("skip-cert-verify=true".to_string());
    }
    format!("{}\r\n", parts.join(","))
}

fn build_surfboard_shadowsocks(
    uuid: &str,
    server: &ServerNodeRow,
    settings: &Map<String, Value>,
    app_key: &str,
) -> String {
    build_surge_shadowsocks(uuid, server, settings, app_key)
}

fn build_surfboard_vmess(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> String {
    let mut parts = vec![
        format!("{}=vmess", server.name),
        server.host.clone(),
        server.port.to_string(),
        format!("username={}", uuid),
        "vmess-aead=true".to_string(),
        "tfo=true".to_string(),
        "udp-relay=true".to_string(),
    ];

    if json_int(settings, "tls") > 0 {
        parts.push("tls=true".to_string());
        if let Some(tls_settings) = settings.get("tls_settings").and_then(Value::as_object) {
            if tls_settings
                .get("allowInsecure")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                parts.push("skip-cert-verify=true".to_string());
            }
            if let Some(server_name) = tls_settings
                .get("serverName")
                .and_then(Value::as_str)
                .or_else(|| tls_settings.get("server_name").and_then(Value::as_str))
            {
                parts.push(format!("sni={}", server_name));
            }
        }
    }

    if json_string(settings, "network", "tcp") == "ws" {
        parts.push("ws=true".to_string());
        if let Some(path) = settings
            .get("network_settings")
            .and_then(Value::as_object)
            .and_then(|value| value.get("path"))
            .and_then(Value::as_str)
        {
            parts.push(format!("ws-path={}", path));
        }
        if let Some(host) = settings
            .get("network_settings")
            .and_then(Value::as_object)
            .and_then(|value| value.get("headers"))
            .and_then(Value::as_object)
            .and_then(|value| value.get("Host"))
            .and_then(Value::as_str)
        {
            parts.push(format!("ws-headers=Host:{}", host));
        }
    }

    format!("{}\r\n", parts.join(","))
}

fn build_surfboard_trojan(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> String {
    build_surge_trojan(uuid, server, settings)
}

fn append_plain_obfs_opts(settings: &Map<String, Value>, parts: &mut Vec<String>) {
    if let Some(plugin) = settings.get("plugin").and_then(Value::as_str) {
        if plugin == "obfs" {
            if let Some(opts) = settings.get("plugin_opts").and_then(Value::as_str) {
                let parsed = parse_plugin_opts(opts);
                if let Some(obfs) = parsed.get("obfs") {
                    parts.push(format!("obfs={}", obfs));
                }
                if let Some(host) = parsed.get("obfs-host") {
                    parts.push(format!("obfs-host={}", host));
                }
                if let Some(path) = parsed.get("path") {
                    parts.push(format!("obfs-uri={}", path));
                }
            }
        }
    }
}

fn parse_plugin_opts(raw: &str) -> HashMap<String, String> {
    raw.split(';')
        .filter_map(|pair| pair.split_once('='))
        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
        .collect()
}

fn pick_path_from_request_header(network_settings: &Map<String, Value>) -> Option<String> {
    network_settings
        .get("header")
        .and_then(Value::as_object)
        .and_then(|header| header.get("request"))
        .and_then(Value::as_object)
        .and_then(|request| request.get("path"))
        .and_then(Value::as_array)
        .and_then(|paths| paths.first())
        .and_then(Value::as_str)
        .map(|value| value.to_string())
}

fn pick_host_from_request_header(network_settings: &Map<String, Value>) -> Option<String> {
    network_settings
        .get("header")
        .and_then(Value::as_object)
        .and_then(|header| header.get("request"))
        .and_then(Value::as_object)
        .and_then(|request| request.get("headers"))
        .and_then(Value::as_object)
        .and_then(|headers| headers.get("Host"))
        .and_then(Value::as_array)
        .and_then(|hosts| hosts.first())
        .and_then(Value::as_str)
        .map(|value| value.to_string())
}

fn surge_panel_subscribe_info(app_name: &str, user: &UserRow) -> String {
    let upload = traffic_to_gb(user.u.unwrap_or(0));
    let download = traffic_to_gb(user.d.unwrap_or(0));
    let total_traffic = traffic_to_gb(user.transfer_enable.unwrap_or(0));
    let unused = format!(
        "{:.2}",
        (user.transfer_enable.unwrap_or(0) - (user.u.unwrap_or(0) + user.d.unwrap_or(0))).max(0) as f64
            / (1024_f64 * 1024_f64 * 1024_f64)
    );
    let expire_date = user
        .expired_at
        .map(|ts| chrono::DateTime::from_timestamp(ts, 0))
        .flatten()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "长期有效".to_string());
    format!(
        "title={}订阅信息, content=上传流量：{}GB\\n下载流量：{}GB\\n剩余流量：{{ {} }}GB\\n套餐流量：{}GB\\n到期时间：{}",
        app_name, upload, download, unused, total_traffic, expire_date
    )
}

fn surfboard_panel_subscribe_info(app_name: &str, user: &UserRow) -> String {
    let upload = traffic_to_gb(user.u.unwrap_or(0));
    let download = traffic_to_gb(user.d.unwrap_or(0));
    let total_traffic = traffic_to_gb(user.transfer_enable.unwrap_or(0));
    let unused = format!(
        "{:.2}",
        (user.transfer_enable.unwrap_or(0) - (user.u.unwrap_or(0) + user.d.unwrap_or(0))).max(0) as f64
            / (1024_f64 * 1024_f64 * 1024_f64)
    );
    let expire_date = user
        .expired_at
        .map(|ts| chrono::DateTime::from_timestamp(ts, 0))
        .flatten()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "长期有效".to_string());
    format!(
        "title={}订阅信息, content=上传流量：{}GB\\n下载流量：{}GB\\n剩余流量: {{ {} }}GB\\n套餐流量：{}GB\\n到期时间：{}",
        app_name, upload, download, unused, total_traffic, expire_date
    )
}

fn subscribe_link_placeholder() -> String {
    "$subscribe_link".to_string()
}

fn subscribe_host_placeholder() -> String {
    "$subs_domain".to_string()
}

fn build_stash_shadowsocks(
    uuid: &str,
    server: &ServerNodeRow,
    settings: &Map<String, Value>,
    app_key: &str,
) -> Value {
    let cipher = json_string(settings, "cipher", "aes-128-gcm");
    let password = subscribe_shadowsocks_password(uuid, server, settings, app_key);
    let mut value = json!({
        "name": server.name,
        "type": "ss",
        "server": server.host,
        "port": server.port,
        "cipher": cipher,
        "password": password,
        "udp": true,
    });
    if let Some(plugin) = settings.get("plugin").and_then(Value::as_str) {
        if let Some(opts) = settings.get("plugin_opts").and_then(Value::as_str) {
            let parsed = parse_plugin_opts(opts);
            value["plugin"] = Value::String(plugin.to_string());
            if plugin == "obfs" {
                let mut plugin_opts = Map::new();
                if let Some(mode) = parsed.get("obfs") {
                    plugin_opts.insert("mode".to_string(), Value::String(mode.clone()));
                }
                if let Some(host) = parsed.get("obfs-host") {
                    plugin_opts.insert("host".to_string(), Value::String(host.clone()));
                }
                if let Some(path) = parsed.get("path") {
                    plugin_opts.insert("path".to_string(), Value::String(path.clone()));
                }
                value["plugin-opts"] = Value::Object(plugin_opts);
            }
        }
    }
    value
}

fn build_stash_vmess(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut value = json!({
        "name": server.name,
        "type": "vmess",
        "server": server.host,
        "port": server.port,
        "uuid": uuid,
        "alterId": 0,
        "cipher": "auto",
        "udp": true,
        "tls": json_int(settings, "tls") > 0,
        "skip-cert-verify": settings.get("tls_settings").and_then(Value::as_object).and_then(|v| v.get("allow_insecure")).and_then(Value::as_bool).unwrap_or(false),
    });
    if let Some(server_name) = settings
        .get("tls_settings")
        .and_then(Value::as_object)
        .and_then(|v| v.get("server_name"))
        .and_then(Value::as_str)
    {
        value["servername"] = Value::String(server_name.to_string());
    }
    match json_string(settings, "network", "tcp").as_str() {
        "tcp" => {
            let header_type = settings
                .get("network_settings")
                .and_then(Value::as_object)
                .and_then(|network| network.get("header"))
                .and_then(Value::as_object)
                .and_then(|header| header.get("type"))
                .and_then(Value::as_str)
                .unwrap_or("http");
            value["network"] = Value::String(header_type.to_string());
            let mut http_opts = Map::new();
            if let Some(paths) = settings
                .get("network_settings")
                .and_then(Value::as_object)
                .and_then(|network| network.get("header"))
                .and_then(Value::as_object)
                .and_then(|header| header.get("request"))
                .and_then(Value::as_object)
                .and_then(|request| request.get("path"))
            {
                http_opts.insert("path".to_string(), paths.clone());
            }
            if let Some(hosts) = settings
                .get("network_settings")
                .and_then(Value::as_object)
                .and_then(|network| network.get("header"))
                .and_then(Value::as_object)
                .and_then(|header| header.get("request"))
                .and_then(Value::as_object)
                .and_then(|request| request.get("headers"))
                .and_then(Value::as_object)
                .and_then(|headers| headers.get("Host"))
            {
                let mut headers_obj = Map::new();
                headers_obj.insert("Host".to_string(), hosts.clone());
                http_opts.insert("headers".to_string(), Value::Object(headers_obj));
            }
            if !http_opts.is_empty() {
                value["http-opts"] = Value::Object(http_opts);
            }
        }
        "ws" => {
            value["network"] = Value::String("ws".to_string());
            let mut ws_opts = Map::new();
            if let Some(path) = settings
                .get("network_settings")
                .and_then(Value::as_object)
                .and_then(|network| network.get("path"))
                .and_then(Value::as_str)
            {
                ws_opts.insert("path".to_string(), Value::String(path.to_string()));
            }
            if let Some(host) = settings
                .get("network_settings")
                .and_then(Value::as_object)
                .and_then(|network| network.get("headers"))
                .and_then(Value::as_object)
                .and_then(|headers| headers.get("Host"))
                .and_then(Value::as_str)
            {
                let mut headers_obj = Map::new();
                headers_obj.insert("Host".to_string(), Value::String(host.to_string()));
                ws_opts.insert("headers".to_string(), Value::Object(headers_obj));
            }
            value["ws-opts"] = Value::Object(ws_opts);
        }
        "grpc" => {
            value["network"] = Value::String("grpc".to_string());
            let mut grpc_opts = Map::new();
            if let Some(service_name) = settings
                .get("network_settings")
                .and_then(Value::as_object)
                .and_then(|network| network.get("serviceName"))
                .and_then(Value::as_str)
            {
                grpc_opts.insert(
                    "grpc-service-name".to_string(),
                    Value::String(service_name.to_string()),
                );
            }
            value["grpc-opts"] = Value::Object(grpc_opts);
        }
        _ => {}
    }
    value
}

fn build_stash_vless(
    uuid: &str,
    server: &ServerNodeRow,
    settings: &Map<String, Value>,
) -> Option<Value> {
    let mut value = json!({
        "name": server.name,
        "type": "vless",
        "server": server.host,
        "port": server.port,
        "uuid": uuid,
        "udp": true,
        "client-fingerprint": "chrome",
    });

    match json_int(settings, "tls") {
        1 => {
            value["tls"] = Value::Bool(true);
            value["skip-cert-verify"] = Value::Bool(
                settings
                    .get("tls_settings")
                    .and_then(Value::as_object)
                    .and_then(|tls| tls.get("allow_insecure"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            );
            if let Some(server_name) = settings
                .get("tls_settings")
                .and_then(Value::as_object)
                .and_then(|tls| tls.get("server_name"))
                .and_then(Value::as_str)
            {
                value["servername"] = Value::String(server_name.to_string());
            }
        }
        2 => {
            value["tls"] = Value::Bool(true);
            if let Some(server_name) = settings
                .get("reality_settings")
                .and_then(Value::as_object)
                .and_then(|reality| reality.get("server_name"))
                .and_then(Value::as_str)
            {
                value["servername"] = Value::String(server_name.to_string());
                value["sni"] = Value::String(server_name.to_string());
            }
            if let Some(flow) = settings.get("flow").and_then(Value::as_str) {
                value["flow"] = Value::String(flow.to_string());
            }
            let mut reality_opts = Map::new();
            if let Some(public_key) = settings
                .get("reality_settings")
                .and_then(Value::as_object)
                .and_then(|reality| reality.get("public_key"))
                .and_then(Value::as_str)
            {
                reality_opts.insert("public-key".to_string(), Value::String(public_key.to_string()));
            }
            if let Some(short_id) = settings
                .get("reality_settings")
                .and_then(Value::as_object)
                .and_then(|reality| reality.get("short_id"))
                .and_then(Value::as_str)
            {
                reality_opts.insert("short-id".to_string(), Value::String(short_id.to_string()));
            }
            value["reality-opts"] = Value::Object(reality_opts);
        }
        _ => {}
    }

    match json_string(settings, "network", "tcp").as_str() {
        "tcp" => {
            let header_type = settings
                .get("network_settings")
                .and_then(Value::as_object)
                .and_then(|network| network.get("header"))
                .and_then(Value::as_object)
                .and_then(|header| header.get("type"))
                .and_then(Value::as_str)
                .unwrap_or("tcp");
            if header_type != "tcp" {
                value["network"] = Value::String(header_type.to_string());
                let mut http_opts = Map::new();
                if let Some(headers) = settings
                    .get("network_settings")
                    .and_then(Value::as_object)
                    .and_then(|network| network.get("header"))
                    .and_then(Value::as_object)
                    .and_then(|header| header.get("request"))
                    .and_then(Value::as_object)
                    .and_then(|request| request.get("headers"))
                {
                    http_opts.insert("headers".to_string(), headers.clone());
                }
                if let Some(path) = settings
                    .get("network_settings")
                    .and_then(Value::as_object)
                    .and_then(|network| network.get("header"))
                    .and_then(Value::as_object)
                    .and_then(|header| header.get("request"))
                    .and_then(Value::as_object)
                    .and_then(|request| request.get("path"))
                {
                    http_opts.insert("path".to_string(), path.clone());
                }
                value["http-opts"] = Value::Object(http_opts);
            }
        }
        "ws" => {
            value["network"] = Value::String("ws".to_string());
            let mut ws_opts = Map::new();
            if let Some(path) = settings
                .get("network_settings")
                .and_then(Value::as_object)
                .and_then(|network| network.get("path"))
            {
                ws_opts.insert("path".to_string(), path.clone());
            }
            if let Some(host) = settings
                .get("network_settings")
                .and_then(Value::as_object)
                .and_then(|network| network.get("headers"))
                .and_then(Value::as_object)
                .and_then(|headers| headers.get("Host"))
                .and_then(Value::as_str)
            {
                let mut headers = Map::new();
                headers.insert("Host".to_string(), Value::String(host.to_string()));
                ws_opts.insert("headers".to_string(), Value::Object(headers));
            }
            value["ws-opts"] = Value::Object(ws_opts);
        }
        "grpc" => {
            value["network"] = Value::String("grpc".to_string());
            let mut grpc_opts = Map::new();
            if let Some(service_name) = settings
                .get("network_settings")
                .and_then(Value::as_object)
                .and_then(|network| network.get("serviceName"))
                .and_then(Value::as_str)
            {
                grpc_opts.insert(
                    "grpc-service-name".to_string(),
                    Value::String(service_name.to_string()),
                );
            }
            value["grpc-opts"] = Value::Object(grpc_opts);
        }
        _ => {}
    }

    Some(value)
}

fn build_stash_trojan(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut value = json!({
        "name": server.name,
        "type": "trojan",
        "server": server.host,
        "port": server.port,
        "password": uuid,
        "udp": true,
        "skip-cert-verify": settings.get("allow_insecure").and_then(Value::as_bool).unwrap_or(false),
    });
    if let Some(server_name) = settings.get("server_name").and_then(Value::as_str) {
        value["sni"] = Value::String(server_name.to_string());
    }
    match json_string(settings, "network", "tcp").as_str() {
        "tcp" => {
            if let Some(header_type) = settings
                .get("network_settings")
                .and_then(Value::as_object)
                .and_then(|network| network.get("header"))
                .and_then(Value::as_object)
                .and_then(|header| header.get("type"))
                .and_then(Value::as_str)
            {
                value["network"] = Value::String(header_type.to_string());
            }
            if let Some(path) = settings
                .get("network_settings")
                .and_then(Value::as_object)
                .and_then(|network| network.get("header"))
                .and_then(Value::as_object)
                .and_then(|header| header.get("request"))
                .and_then(Value::as_object)
                .and_then(|request| request.get("path"))
            {
                value["http-opts"] = json!({ "path": path.clone() });
            }
        }
        "ws" => {
            value["network"] = Value::String("ws".to_string());
            let mut ws_opts = Map::new();
            if let Some(path) = settings
                .get("network_settings")
                .and_then(Value::as_object)
                .and_then(|network| network.get("path"))
                .and_then(Value::as_str)
            {
                ws_opts.insert("path".to_string(), Value::String(path.to_string()));
            }
            if let Some(host) = settings
                .get("network_settings")
                .and_then(Value::as_object)
                .and_then(|network| network.get("headers"))
                .and_then(Value::as_object)
                .and_then(|headers| headers.get("Host"))
                .and_then(Value::as_str)
            {
                let mut headers = Map::new();
                headers.insert("Host".to_string(), Value::String(host.to_string()));
                ws_opts.insert("headers".to_string(), Value::Object(headers));
            }
            value["ws-opts"] = Value::Object(ws_opts);
        }
        _ => {}
    }
    value
}

fn build_stash_hysteria(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let version = json_int(settings, "version");
    let mut value = json!({
        "name": server.name,
        "server": server.host,
        "port": server.port,
        "skip-cert-verify": settings.get("tls").and_then(Value::as_object).and_then(|tls| tls.get("allow_insecure")).and_then(Value::as_bool).unwrap_or(false),
    });
    if let Some(server_name) = settings
        .get("tls")
        .and_then(Value::as_object)
        .and_then(|tls| tls.get("server_name"))
        .and_then(Value::as_str)
    {
        value["sni"] = Value::String(server_name.to_string());
    }
    if let Some(up) = settings
        .get("bandwidth")
        .and_then(Value::as_object)
        .and_then(|value| value.get("up"))
        .and_then(parse_i64_value)
    {
        value["up-speed"] = Value::from(up);
    }
    if let Some(down) = settings
        .get("bandwidth")
        .and_then(Value::as_object)
        .and_then(|value| value.get("down"))
        .and_then(parse_i64_value)
    {
        value["down-speed"] = Value::from(down);
    }
    match version {
        1 => {
            value["type"] = Value::String("hysteria".to_string());
            value["auth-str"] = Value::String(uuid.to_string());
            value["protocol"] = Value::String("udp".to_string());
            if settings
                .get("obfs")
                .and_then(Value::as_object)
                .and_then(|obfs| obfs.get("open"))
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                if let Some(kind) = settings
                    .get("obfs")
                    .and_then(Value::as_object)
                    .and_then(|obfs| obfs.get("type"))
                    .and_then(Value::as_str)
                {
                    value["obfs"] = Value::String(kind.to_string());
                }
            }
        }
        _ => {
            value["type"] = Value::String("hysteria2".to_string());
            value["auth"] = Value::String(uuid.to_string());
            value["fast-open"] = Value::Bool(true);
        }
    }
    value
}

fn build_stash_tuic(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut value = json!({
        "name": server.name,
        "type": "tuic",
        "server": server.host,
        "port": server.port,
        "uuid": uuid,
        "password": uuid,
        "congestion-controller": settings.get("congestion_control").and_then(Value::as_str).unwrap_or("cubic"),
        "udp-relay-mode": settings.get("udp_relay_mode").and_then(Value::as_str).unwrap_or("native"),
        "alpn": settings.get("alpn").cloned().unwrap_or_else(|| json!(["h3"])),
        "reduce-rtt": true,
        "fast-open": true,
        "heartbeat-interval": 10000,
        "request-timeout": 8000,
        "max-udp-relay-packet-size": 1500,
        "version": settings.get("version").and_then(parse_i64_value).unwrap_or(5),
        "skip-cert-verify": settings.get("tls").and_then(Value::as_object).and_then(|tls| tls.get("allow_insecure")).and_then(Value::as_bool).unwrap_or(false),
    });
    if let Some(server_name) = settings
        .get("tls")
        .and_then(Value::as_object)
        .and_then(|tls| tls.get("server_name"))
        .and_then(Value::as_str)
    {
        value["sni"] = Value::String(server_name.to_string());
    }
    value
}

fn build_stash_socks5(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut value = json!({
        "name": server.name,
        "type": "socks5",
        "server": server.host,
        "port": server.port,
        "username": uuid,
        "password": uuid,
        "udp": true,
    });
    if json_int(settings, "tls") > 0 {
        value["tls"] = Value::Bool(true);
        value["skip-cert-verify"] = Value::Bool(
            settings
                .get("tls_settings")
                .and_then(Value::as_object)
                .and_then(|tls| tls.get("allow_insecure"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
        );
        if let Some(server_name) = settings
            .get("tls_settings")
            .and_then(Value::as_object)
            .and_then(|tls| tls.get("server_name"))
            .and_then(Value::as_str)
        {
            value["sni"] = Value::String(server_name.to_string());
        }
    }
    value
}

fn build_stash_http(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut value = json!({
        "name": server.name,
        "type": "http",
        "server": server.host,
        "port": server.port,
        "username": uuid,
        "password": uuid,
    });
    if json_int(settings, "tls") > 0 {
        value["tls"] = Value::Bool(true);
        value["skip-cert-verify"] = Value::Bool(
            settings
                .get("tls_settings")
                .and_then(Value::as_object)
                .and_then(|tls| tls.get("allow_insecure"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
        );
        if let Some(server_name) = settings
            .get("tls_settings")
            .and_then(Value::as_object)
            .and_then(|tls| tls.get("server_name"))
            .and_then(Value::as_str)
        {
            value["sni"] = Value::String(server_name.to_string());
        }
    }
    value
}

fn is_stash_regex(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    value.starts_with('/') && value.ends_with('/') && value.len() > 1
}

fn stash_regex_matches(pattern: &str, candidate: &str) -> bool {
    let regex = pattern.trim_matches('/');
    regex::Regex::new(regex)
        .ok()
        .map(|compiled| compiled.is_match(candidate))
        .unwrap_or(false)
}
