use super::super::*;

mod catalog;
mod config;
mod lifecycle;
mod store;

const PROTECTED_PLUGINS: &[&str] = &[
    "epay",
    "alipay_f2f",
    "btcpay",
    "coinbase",
    "coin_payments",
    "mgate",
    "telegram",
];

pub async fn types(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
) -> Response<Body> {
    match build_types_response(&state, headers).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_plugins(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_plugins_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_config(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_config_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn update_config(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_update_config_response(&state, headers, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn config(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    method: Method,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match method {
        Method::GET => match build_get_config_response(&state, headers, uri).await {
            Ok(response) => response,
            Err(response) => response,
        },
        Method::POST => match build_update_config_response(&state, headers, body).await {
            Ok(response) => response,
            Err(response) => response,
        },
        _ => json_error(StatusCode::METHOD_NOT_ALLOWED, "method not allowed"),
    }
}

pub async fn enable(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Response<Body> {
    match lifecycle::build_toggle_enabled_response(&state, headers, body, true).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn disable(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Response<Body> {
    match lifecycle::build_toggle_enabled_response(&state, headers, body, false).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn install(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Response<Body> {
    match lifecycle::build_install_response(&state, headers, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn upload(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    multipart: axum::extract::Multipart,
) -> Response<Body> {
    match lifecycle::build_upload_response(&state, headers, multipart).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn uninstall(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Response<Body> {
    match lifecycle::build_uninstall_response(&state, headers, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn upgrade(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Response<Body> {
    match lifecycle::build_upgrade_response(&state, headers, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn delete(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Response<Body> {
    match lifecycle::build_delete_response(&state, headers, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_types_response(
    state: &AppState,
    headers: HeaderMap,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    Ok(json_value_response(json!({
        "data": [
            {
                "value": "feature",
                "label": "功能",
                "description": "提供功能扩展的插件，如Telegram登录、邮件通知等",
                "icon": "🔧"
            },
            {
                "value": "payment",
                "label": "支付方式",
                "description": "提供支付接口的插件，如支付宝、微信支付等",
                "icon": "💳"
            }
        ]
    })))
}

async fn build_get_plugins_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let type_filter = params
        .get("type")
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty());

    let installed_plugins = store::load_installed_plugins(state, type_filter.as_deref())
        .await
        .map_err(internal_error)?;
    let installed_map = installed_plugins
        .iter()
        .map(|row| (row.code.clone(), row.clone()))
        .collect::<HashMap<_, _>>();

    let plugin_dirs = catalog::load_plugin_directory_configs(type_filter.as_deref());
    let mut plugins = Vec::new();
    for plugin in plugin_dirs {
        let installed = installed_map.get(&plugin.code);
        let db_config = installed
            .and_then(|row| row.config.as_deref())
            .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
            .unwrap_or(Value::Null);
        let default_config = plugin
            .config
            .clone()
            .unwrap_or(Value::Object(Map::new()));
        let merged_config = config::merge_plugin_config(default_config, db_config);
        let need_upgrade = installed
            .map(|row| config::compare_plugin_versions(&plugin.version, &row.version))
            .unwrap_or(false);
        plugins.push(json!({
            "code": plugin.code,
            "name": plugin.name,
            "version": plugin.version,
            "description": plugin.description,
            "author": plugin.author,
            "type": plugin.r#type,
            "is_installed": installed.is_some(),
            "is_enabled": installed.map(|row| row.is_enabled != 0).unwrap_or(false),
            "is_protected": PROTECTED_PLUGINS.contains(&plugin.code.as_str()),
            "can_be_deleted": !PROTECTED_PLUGINS.contains(&plugin.code.as_str()),
            "config": merged_config,
            "readme": plugin.readme.unwrap_or_default(),
            "need_upgrade": need_upgrade,
        }));
    }

    Ok(json_value_response(json!({
        "data": plugins
    })))
}

async fn build_get_config_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let code = params
        .get("code")
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let dir_config = catalog::load_single_plugin_directory_config(&code)
        .ok_or_else(|| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"插件不存在"})))?;
    let db_row = store::load_installed_plugin_by_code(state, &code)
        .await
        .map_err(internal_error)?;
    let db_config = db_row
        .and_then(|row| row.config)
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .unwrap_or(Value::Null);
    let merged = config::merge_plugin_config(
        dir_config.config.unwrap_or(Value::Object(Map::new())),
        db_config,
    );
    Ok(json_value_response(json!({
        "data": merged
    })))
}

async fn build_update_config_response(
    state: &AppState,
    headers: HeaderMap,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let code = payload
        .get("code")
        .and_then(Value::as_str)
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    let input_config = payload
        .get("config")
        .and_then(Value::as_object)
        .cloned()
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let dir_config = catalog::load_single_plugin_directory_config(&code)
        .ok_or_else(|| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"插件不存在"})))?;
    let defaults = dir_config.config.unwrap_or(Value::Object(Map::new()));
    let normalized = config::normalize_plugin_config_update(defaults, input_config);
    let serialized = serde_json::to_string(&normalized)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "配置更新失败"))?;

    let affected = sqlx::query("UPDATE v2_plugins SET config = ?, updated_at = NOW() WHERE code = ?")
        .bind(&serialized)
        .bind(&code)
        .execute(&state.db)
        .await
        .map_err(internal_error)?
        .rows_affected();
    if affected == 0 {
        return Ok(json_status_response(
            StatusCode::BAD_REQUEST,
            json!({"message":"配置更新失败：插件不存在或尚未安装"}),
        ));
    }

    Ok(json_value_response(json!({
        "message": "配置更新成功"
    })))
}
