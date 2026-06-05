use super::super::*;

const VALID_SERVER_TYPES: &[&str] = &[
    "hysteria",
    "vless",
    "trojan",
    "vmess",
    "tuic",
    "shadowsocks",
    "anytls",
    "socks",
    "naive",
    "http",
    "mieru",
];

pub(crate) struct ServerManageSaveInput {
    pub server_type: String,
    pub code: Option<String>,
    pub show: bool,
    pub name: String,
    pub group_ids_json: String,
    pub route_ids_json: String,
    pub parent_id: Option<i64>,
    pub host: String,
    pub port: String,
    pub server_port: i64,
    pub tags_json: String,
    pub rate: String,
    pub rate_time_enable: bool,
    pub rate_time_ranges_json: String,
    pub protocol_settings_json: String,
}

pub(crate) fn normalize_server_save_input(
    obj: &Map<String, Value>,
) -> Result<ServerManageSaveInput, Response<Body>> {
    let server_type = obj
        .get("type")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    if !VALID_SERVER_TYPES.contains(&server_type.as_str()) {
        return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    }

    let name = obj
        .get("name")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "节点名称不能为空"))?;
    let host = obj
        .get("host")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "节点地址不能为空"))?;
    let port = parse_required_string(obj.get("port"), "连接端口不能为空")?;
    let server_port = parse_required_i64(obj.get("server_port"), "后端服务端口不能为空")?;
    let rate = parse_required_rate(obj.get("rate"))?;
    let show = obj.get("show").and_then(|value| value.as_bool()).unwrap_or(false);
    let parent_id = obj
        .get("parent_id")
        .and_then(parse_i64_value)
        .filter(|value| *value > 0);
    let code = parse_optional_string_field(obj.get("code"))?;
    let group_ids = normalize_positive_i64_array(obj.get("group_ids"), "权限组格式不正确")?;
    let route_ids = normalize_positive_i64_array(obj.get("route_ids"), "路由组格式不正确")?;
    let tags = normalize_string_array(obj.get("tags"), "标签格式不正确")?;
    let rate_time_enable = obj
        .get("rate_time_enable")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let rate_time_ranges = normalize_rate_time_ranges(obj.get("rate_time_ranges"))?;
    let protocol_settings = normalize_protocol_settings_for_save(
        &server_type,
        obj.get("protocol_settings"),
    )?;

    Ok(ServerManageSaveInput {
        server_type,
        code,
        show,
        name,
        group_ids_json: serde_json::to_string(&group_ids).unwrap_or_else(|_| "[]".to_string()),
        route_ids_json: serde_json::to_string(&route_ids).unwrap_or_else(|_| "[]".to_string()),
        parent_id,
        host,
        port,
        server_port,
        tags_json: serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string()),
        rate,
        rate_time_enable,
        rate_time_ranges_json: serde_json::to_string(&rate_time_ranges).unwrap_or_else(|_| "[]".to_string()),
        protocol_settings_json: Value::Object(protocol_settings).to_string(),
    })
}

fn normalize_protocol_settings_for_save(
    server_type: &str,
    raw: Option<&Value>,
) -> Result<Map<String, Value>, Response<Body>> {
    let raw_map = match raw {
        Some(Value::Object(map)) => map.clone(),
        Some(Value::Null) | None => Map::new(),
        Some(_) => return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed")),
    };
    validate_protocol_settings(server_type, &raw_map)?;
    Ok(normalized_protocol_settings(server_type, raw_map))
}

fn validate_protocol_settings(
    server_type: &str,
    raw: &Map<String, Value>,
) -> Result<(), Response<Body>> {
    match server_type {
        "shadowsocks" => {
            require_string(raw, &["cipher"])?;
        }
        "vmess" => {
            require_i64(raw, &["tls"])?;
            require_string(raw, &["network"])?;
            optional_object(raw, &["network_settings"])?;
            optional_string(raw, &["tls_settings", "server_name"])?;
            optional_bool(raw, &["tls_settings", "allow_insecure"])?;
        }
        "trojan" => {
            require_string(raw, &["network"])?;
            optional_object(raw, &["network_settings"])?;
            optional_string(raw, &["server_name"])?;
            optional_bool(raw, &["allow_insecure"])?;
        }
        "hysteria" => {
            require_i64(raw, &["version"])?;
            optional_string(raw, &["alpn"])?;
            optional_bool(raw, &["obfs", "open"])?;
            optional_string(raw, &["obfs", "type"])?;
            optional_string(raw, &["obfs", "password"])?;
            optional_string(raw, &["tls", "server_name"])?;
            optional_bool(raw, &["tls", "allow_insecure"])?;
            optional_i64(raw, &["bandwidth", "up"])?;
            optional_i64(raw, &["bandwidth", "down"])?;
            optional_i64(raw, &["hop_interval"])?;
        }
        "vless" => {
            require_i64(raw, &["tls"])?;
            require_string(raw, &["network"])?;
            optional_object(raw, &["network_settings"])?;
            optional_string(raw, &["flow"])?;
            optional_string(raw, &["tls_settings", "server_name"])?;
            optional_bool(raw, &["tls_settings", "allow_insecure"])?;
            optional_bool(raw, &["reality_settings", "allow_insecure"])?;
            optional_string(raw, &["reality_settings", "server_name"])?;
            optional_i64(raw, &["reality_settings", "server_port"])?;
            optional_string(raw, &["reality_settings", "public_key"])?;
            optional_string(raw, &["reality_settings", "private_key"])?;
            optional_string(raw, &["reality_settings", "short_id"])?;
        }
        "naive" | "http" | "socks" => {
            if server_type != "socks" {
                require_i64(raw, &["tls"])?;
            } else {
                optional_i64(raw, &["tls"])?;
                optional_bool(raw, &["udp_over_tcp"])?;
            }
            optional_object(raw, &["tls_settings"])?;
        }
        "mieru" => {
            require_string(raw, &["transport"])?;
            require_string(raw, &["multiplexing"])?;
        }
        "anytls" => {
            optional_object(raw, &["tls"])?;
            optional_array(raw, &["padding_scheme"])?;
        }
        "tuic" => {
            optional_i64(raw, &["version"])?;
            optional_string(raw, &["congestion_control"])?;
            optional_array(raw, &["alpn"])?;
            optional_string(raw, &["udp_relay_mode"])?;
            optional_bool(raw, &["zero_rtt_handshake"])?;
            optional_string(raw, &["heartbeat"])?;
            optional_object(raw, &["tls"])?;
        }
        _ => {}
    }
    Ok(())
}

fn parse_required_string(
    value: Option<&Value>,
    message: &str,
) -> Result<String, Response<Body>> {
    value
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, message))
}

fn parse_required_i64(value: Option<&Value>, message: &str) -> Result<i64, Response<Body>> {
    value
        .and_then(parse_i64_value)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, message))
}

fn parse_required_rate(value: Option<&Value>) -> Result<String, Response<Body>> {
    let raw = value.ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "倍率不能为空"))?;
    let parsed = match raw {
        Value::String(text) => text.trim().parse::<f64>().ok(),
        Value::Number(number) => number.as_f64(),
        _ => None,
    }
    .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "倍率格式不正确"))?;
    Ok(format!("{:.2}", parsed))
}

fn normalize_positive_i64_array(
    value: Option<&Value>,
    message: &str,
) -> Result<Vec<i64>, Response<Body>> {
    match value {
        Some(Value::Null) | None => Ok(Vec::new()),
        Some(Value::Array(items)) => Ok(
            items
                .iter()
                .filter_map(parse_i64_value)
                .filter(|value| *value > 0)
                .collect::<Vec<_>>(),
        ),
        Some(_) => Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, message)),
    }
}

fn normalize_string_array(
    value: Option<&Value>,
    message: &str,
) -> Result<Vec<String>, Response<Body>> {
    match value {
        Some(Value::Null) | None => Ok(Vec::new()),
        Some(Value::Array(items)) => Ok(
            items
                .iter()
                .filter_map(|item| item.as_str())
                .map(|item| item.trim().to_string())
                .filter(|item| !item.is_empty())
                .collect::<Vec<_>>(),
        ),
        Some(_) => Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, message)),
    }
}

fn normalize_rate_time_ranges(value: Option<&Value>) -> Result<Vec<Value>, Response<Body>> {
    let items = match value {
        Some(Value::Array(items)) => items,
        Some(Value::Null) | None => return Ok(Vec::new()),
        Some(_) => return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed")),
    };
    let mut normalized = Vec::new();
    for item in items {
        let obj = item
            .as_object()
            .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
        let start = parse_required_string(obj.get("start"), "Validation failed")?;
        let end = parse_required_string(obj.get("end"), "Validation failed")?;
        let rate = parse_required_rate(obj.get("rate"))?;
        normalized.push(json!({
            "start": start,
            "end": end,
            "rate": rate.parse::<f64>().unwrap_or(0.0),
        }));
    }
    Ok(normalized)
}

fn require_string(raw: &Map<String, Value>, path: &[&str]) -> Result<(), Response<Body>> {
    let value = get_nested_value(raw, path)
        .and_then(|value| value.as_str())
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if value.is_none() {
        return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    }
    Ok(())
}

fn require_i64(raw: &Map<String, Value>, path: &[&str]) -> Result<(), Response<Body>> {
    if get_nested_value(raw, path).and_then(parse_i64_value).is_none() {
        return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    }
    Ok(())
}

fn optional_string(raw: &Map<String, Value>, path: &[&str]) -> Result<(), Response<Body>> {
    if let Some(value) = get_nested_value(raw, path) {
        match value {
            Value::Null => {}
            Value::String(_) => {}
            _ => return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed")),
        }
    }
    Ok(())
}

fn optional_i64(raw: &Map<String, Value>, path: &[&str]) -> Result<(), Response<Body>> {
    if let Some(value) = get_nested_value(raw, path) {
        match value {
            Value::Null => {}
            Value::Number(_) | Value::String(_) if parse_i64_value(value).is_some() => {}
            _ => return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed")),
        }
    }
    Ok(())
}

fn optional_bool(raw: &Map<String, Value>, path: &[&str]) -> Result<(), Response<Body>> {
    if let Some(value) = get_nested_value(raw, path) {
        match value {
            Value::Null | Value::Bool(_) => {}
            _ => return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed")),
        }
    }
    Ok(())
}

fn optional_array(raw: &Map<String, Value>, path: &[&str]) -> Result<(), Response<Body>> {
    if let Some(value) = get_nested_value(raw, path) {
        match value {
            Value::Null | Value::Array(_) => {}
            _ => return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed")),
        }
    }
    Ok(())
}

fn optional_object(raw: &Map<String, Value>, path: &[&str]) -> Result<(), Response<Body>> {
    if let Some(value) = get_nested_value(raw, path) {
        match value {
            Value::Null | Value::Object(_) => {}
            _ => return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed")),
        }
    }
    Ok(())
}
