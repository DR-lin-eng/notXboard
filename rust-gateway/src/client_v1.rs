use crate::*;

pub async fn app_get_config(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_app_get_config_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn app_get_version(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_app_get_version_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn subscribe_legacy(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(path): axum::extract::Path<String>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_subscribe_legacy_response(&state, path, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_app_get_config_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let cache_key = build_cache_key(&uri);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }

    let params = parse_query(&uri);
    let token = params
        .get("token")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| json_error(StatusCode::FORBIDDEN, "token is null"))?;
    if token.len() != 32 || !token.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return Err(json_error(StatusCode::FORBIDDEN, "token is error"));
    }

    let user = load_bearer_user_by_token(state, token)
        .await
        .map_err(internal_error)?;
    let Some(user) = user else {
        return Err(json_error(StatusCode::FORBIDDEN, "token is error"));
    };

    let mut servers = if user_is_available(&user) {
        let user_row = UserRow {
            id: user.id,
            token: Some(user.token.clone()),
            group_id: user.group_id,
            subscribe_key: user.subscribe_key.clone(),
            subscribe_salt: user.subscribe_salt.clone(),
            uuid: Some(user.uuid.clone()),
            u: Some(user.u),
            d: Some(user.d),
            transfer_enable: Some(user.transfer_enable),
            expired_at: user.expired_at,
            trust_level: Some(user.trust_level),
            banned: Some(user.banned),
            is_super_admin: Some(user.is_super_admin),
            is_silenced: Some(user.is_silenced),
            subscription_credential_version: Some(0),
        };
        load_subscribe_servers_for_user(state, &user_row)
            .await
            .map_err(internal_error)?
    } else {
        Vec::new()
    };

    servers.retain(|server| {
        matches!(
            normalize_type(&server.protocol).unwrap_or_default().as_str(),
            "shadowsocks" | "vmess" | "trojan"
        )
    });

    let rotate_credentials = get_setting_bool(state, "rotate_subscription_credentials_daily", false).await;
    let uuid = effective_uuid(&user.uuid, 0, rotate_credentials);
    let proxies = build_clash_proxies(&servers, &uuid, state).await
        .into_iter()
        .filter(|proxy| {
            proxy.get("type").and_then(|value| value.as_str()).map(|value| {
                matches!(value, "ss" | "vmess" | "trojan")
            }).unwrap_or(false)
        })
        .collect::<Vec<_>>();
    let proxy_names = proxies
        .iter()
        .filter_map(|proxy| proxy.get("name").and_then(|value| value.as_str()).map(|value| value.to_string()))
        .collect::<Vec<_>>();

    let custom_path = crate::runtime_paths::resources_path("rules/custom.app.clash.yaml");
    let default_path = crate::runtime_paths::resources_path("rules/app.clash.yaml");
    let config_path = if custom_path.exists() {
        custom_path
    } else {
        default_path
    };
    let raw = std::fs::read_to_string(&config_path)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "load clash config failed"))?;
    let mut config = serde_yaml::from_str::<serde_yaml::Value>(&raw)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "invalid clash config"))?;

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
            for group in groups {
                if let Some(group_map) = group.as_mapping_mut() {
                    if let Some(group_proxies) = group_map
                        .get_mut(serde_yaml::Value::String("proxies".to_string()))
                        .and_then(|value| value.as_sequence_mut())
                    {
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
    }

    let payload = serde_yaml::to_string(&config)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "yaml serialize failed"))?;
    let response = cached_plain_response(
        state,
        cache_key,
        bytes::Bytes::from(payload),
        Duration::from_secs(15),
        "text/yaml; charset=utf-8",
    );
    Ok(response)
}

async fn build_app_get_version_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let cache_key = build_cache_key(&uri);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }

    let user_agent = headers
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_lowercase();

    let data = if user_agent.contains("tidalab/4.0.0") || user_agent.contains("tunnelab/4.0.0") {
        if user_agent.contains("win64") {
            json!({
                "version": get_setting_value(state, "windows_version").await,
                "download_url": get_setting_value(state, "windows_download_url").await,
            })
        } else {
            json!({
                "version": get_setting_value(state, "macos_version").await,
                "download_url": get_setting_value(state, "macos_download_url").await,
            })
        }
    } else {
        json!({
            "windows_version": get_setting_value(state, "windows_version").await,
            "windows_download_url": get_setting_value(state, "windows_download_url").await,
            "macos_version": get_setting_value(state, "macos_version").await,
            "macos_download_url": get_setting_value(state, "macos_download_url").await,
            "android_version": get_setting_value(state, "android_version").await,
            "android_download_url": get_setting_value(state, "android_download_url").await,
        })
    };

    Ok(success_cached_response(
        state,
        cache_key,
        data,
        Duration::from_secs(15),
    ))
}

async fn build_subscribe_legacy_response(
    state: &AppState,
    path: String,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    if path.trim().is_empty() {
        return Err(json_error(StatusCode::NOT_FOUND, "Not found"));
    }

    let cache_key = format!("legacy-subscribe:{}?{}", path, uri.query().unwrap_or_default());
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }

    let params = parse_query(&uri);
    let ip_hint = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|value| value.to_str().ok());
    let user = authenticate_subscribe_obfuscated(state, &path, &params, ip_hint).await?;

    if !subscribe_user_is_available(state, &user).await.map_err(internal_error)? {
        return Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header(CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(Body::from(""))
            .unwrap());
    }

    let mode = rust_subscribe_mode(&params, &headers).unwrap_or(RustSubscribeMode::General);
    let payload = build_rust_subscribe_payload(state, &user, &params, mode).await?;
    Ok(cached_plain_response(
        state,
        cache_key,
        payload,
        Duration::from_secs(15),
        "text/plain; charset=utf-8",
    ))
}
