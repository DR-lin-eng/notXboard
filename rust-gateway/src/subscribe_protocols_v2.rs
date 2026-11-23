use crate::*;

fn dg<'a>(settings: &'a Map<String, Value>, path: &[&str]) -> Option<&'a Value> {
    let (first, rest) = path.split_first()?;
    rest.iter()
        .try_fold(settings.get(*first)?, |value, key| value.get(*key))
        .filter(|value| !value.is_null())
}

fn dg_str<'a>(settings: &'a Map<String, Value>, path: &[&str]) -> Option<&'a str> {
    dg(settings, path)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn truthy(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Bool(value)) => *value,
        Some(Value::Number(value)) => value.as_f64().is_some_and(|n| n != 0.0),
        Some(Value::String(value)) => !value.is_empty() && value != "0" && value != "false",
        Some(Value::Array(value)) => !value.is_empty(),
        Some(Value::Object(value)) => !value.is_empty(),
        _ => false,
    }
}

fn dg_bool(settings: &Map<String, Value>, path: &[&str]) -> bool {
    truthy(dg(settings, path))
}

fn dg_int(settings: &Map<String, Value>, path: &[&str]) -> Option<i64> {
    match dg(settings, path)? {
        Value::Number(value) => value.as_i64(),
        Value::String(value) => value.trim().parse().ok(),
        Value::Bool(value) => Some(i64::from(*value)),
        _ => None,
    }
}

fn tls_mode(settings: &Map<String, Value>, default: i64) -> i64 {
    dg_int(settings, &["tls"]).unwrap_or(default)
}

fn strip_nulls(mut value: Value) -> Value {
    if let Some(object) = value.as_object_mut() {
        object.retain(|_, v| !v.is_null());
    }
    value
}

pub(crate) fn tls_fingerprint(settings: &Map<String, Value>) -> Option<String> {
    let utls = dg(settings, &["utls"])?;
    if !truthy(utls.get("enabled")) {
        return None;
    }
    let fingerprint = utls
        .get("fingerprint")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .unwrap_or("chrome");
    if fingerprint != "random" {
        return Some(fingerprint.to_string());
    }
    const FINGERPRINTS: [&str; 6] = ["chrome", "firefox", "safari", "ios", "edge", "qq"];
    let index = Uuid::new_v4().as_bytes()[0] as usize % FINGERPRINTS.len();
    Some(FINGERPRINTS[index].to_string())
}

struct EchSettings {
    config: Option<String>,
    query_server_name: Option<String>,
}

fn ech_settings(ech: Option<&Value>) -> Option<EchSettings> {
    let ech = ech?;
    if !ech.is_object() || !truthy(ech.get("enabled")) {
        return None;
    }
    let trimmed = |key: &str| {
        ech.get(key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    };
    Some(EchSettings {
        config: trimmed("config"),
        query_server_name: trimmed("query_server_name"),
    })
}

pub(crate) fn mihomo_ech_config(config: &str) -> Option<String> {
    let config = config.trim();
    if config.is_empty() {
        return None;
    }
    let body = if config.starts_with("-----BEGIN") {
        let start = config.find("-----BEGIN ECH CONFIGS-----")? + "-----BEGIN ECH CONFIGS-----".len();
        let end = config[start..].find("-----END ECH CONFIGS-----")? + start;
        &config[start..end]
    } else {
        config
    };
    let compact = body.split_whitespace().collect::<String>();
    (!compact.is_empty()).then_some(compact)
}

fn server_ports(settings: &Map<String, Value>) -> Option<String> {
    dg_str(settings, &["ports"]).map(str::to_string)
}

fn sb_append_utls(tls: &mut Value, settings: &Map<String, Value>) {
    if let Some(fingerprint) = tls_fingerprint(settings) {
        tls["utls"] = json!({ "enabled": true, "fingerprint": fingerprint });
    }
}

fn sb_append_ech(tls: &mut Value, ech: Option<&Value>) {
    if let Some(ech) = ech_settings(ech) {
        tls["ech"] = strip_nulls(json!({
            "enabled": true,
            "config": ech.config.map(|config| vec![config]),
            "query_server_name": ech.query_server_name,
        }));
    }
}

fn sb_append_multiplex(outbound: &mut Value, settings: &Map<String, Value>) {
    let Some(multiplex) = dg(settings, &["multiplex"]) else {
        return;
    };
    if !truthy(multiplex.get("enabled")) {
        return;
    }
    let mut value = strip_nulls(json!({
        "enabled": true,
        "protocol": multiplex.get("protocol").and_then(Value::as_str).unwrap_or("yamux"),
        "max_connections": multiplex.get("max_connections"),
        "min_streams": multiplex.get("min_streams"),
        "max_streams": multiplex.get("max_streams"),
        "padding": truthy(multiplex.get("padding")),
    }));
    if truthy(multiplex.get("brutal").and_then(|b| b.get("enabled"))) {
        value["brutal"] = json!({
            "enabled": true,
            "up_mbps": multiplex.get("brutal").and_then(|b| b.get("up_mbps")),
            "down_mbps": multiplex.get("brutal").and_then(|b| b.get("down_mbps")),
        });
    }
    outbound["multiplex"] = value;
}

fn sb_transport(settings: &Map<String, Value>, server: &ServerNodeRow) -> Option<Value> {
    let ns = |key: &[&str]| {
        let mut path = vec!["network_settings"];
        path.extend_from_slice(key);
        dg(settings, &path).cloned()
    };
    let transport = match dg_str(settings, &["network"])? {
        "tcp" => {
            if dg_str(settings, &["network_settings", "header", "type"]) != Some("http") {
                return None;
            }
            let path = match ns(&["header", "request", "path"]) {
                Some(Value::Array(paths)) => paths.first().cloned().unwrap_or(json!("/")),
                Some(Value::String(path)) => json!(path),
                _ => json!("/"),
            };
            json!({
                "type": "http",
                "path": path,
                "host": ns(&["header", "request", "headers", "Host"]).unwrap_or(json!([])),
            })
        }
        "ws" => json!({
            "type": "ws",
            "path": ns(&["path"]),
            "headers": ns(&["headers", "Host"]).map(|host| json!({ "Host": host })),
            "max_early_data": 0,
        }),
        "grpc" => json!({ "type": "grpc", "service_name": ns(&["serviceName"]) }),
        "h2" => json!({ "type": "http", "host": ns(&["host"]), "path": ns(&["path"]) }),
        "httpupgrade" => json!({
            "type": "httpupgrade",
            "path": ns(&["path"]),
            "host": ns(&["host"]).unwrap_or_else(|| json!(server.host)),
            "headers": ns(&["headers"]),
        }),
        "quic" => json!({ "type": "quic" }),
        _ => return None,
    };
    Some(strip_nulls(transport))
}

fn sb_base(kind: &str, server: &ServerNodeRow) -> Value {
    json!({
        "tag": server.name,
        "type": kind,
        "server": server.host,
        "server_port": server.port,
    })
}

pub(crate) fn singbox_vmess(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut outbound = sb_base("vmess", server);
    outbound["uuid"] = json!(uuid);
    outbound["security"] = json!("auto");
    outbound["alter_id"] = json!(0);
    if dg_bool(settings, &["tls"]) {
        let mut tls = json!({
            "enabled": true,
            "insecure": dg_bool(settings, &["tls_settings", "allow_insecure"]),
        });
        sb_append_utls(&mut tls, settings);
        sb_append_ech(&mut tls, dg(settings, &["tls_settings", "ech"]));
        if let Some(name) = dg_str(settings, &["tls_settings", "server_name"]) {
            tls["server_name"] = json!(name);
        }
        outbound["tls"] = tls;
    }
    sb_append_multiplex(&mut outbound, settings);
    if let Some(transport) = sb_transport(settings, server) {
        outbound["transport"] = transport;
    }
    outbound
}

fn sb_reality_or_tls(settings: &Map<String, Value>, mode: i64) -> Value {
    let mut tls = json!({ "enabled": true });
    if mode == 2 {
        tls["insecure"] = json!(dg_bool(settings, &["reality_settings", "allow_insecure"]));
        tls["server_name"] = json!(dg_str(settings, &["reality_settings", "server_name"]));
        tls["reality"] = json!({
            "enabled": true,
            "public_key": dg_str(settings, &["reality_settings", "public_key"]),
            "short_id": dg_str(settings, &["reality_settings", "short_id"]),
        });
    } else {
        tls["insecure"] = json!(dg_bool(settings, &["tls_settings", "allow_insecure"]));
        sb_append_ech(&mut tls, dg(settings, &["tls_settings", "ech"]));
        if let Some(name) = dg_str(settings, &["tls_settings", "server_name"]) {
            tls["server_name"] = json!(name);
        }
    }
    sb_append_utls(&mut tls, settings);
    tls
}

pub(crate) fn singbox_vless(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut outbound = sb_base("vless", server);
    outbound["uuid"] = json!(uuid);
    outbound["packet_encoding"] = json!("xudp");
    if let Some(flow) = dg_str(settings, &["flow"]) {
        outbound["flow"] = json!(flow);
    }
    let mode = tls_mode(settings, 0);
    if mode > 0 {
        outbound["tls"] = sb_reality_or_tls(settings, mode);
    }
    sb_append_multiplex(&mut outbound, settings);
    if let Some(transport) = sb_transport(settings, server) {
        outbound["transport"] = transport;
    }
    outbound
}

pub(crate) fn singbox_trojan(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut outbound = sb_base("trojan", server);
    outbound["password"] = json!(uuid);
    outbound["tls"] = sb_reality_or_tls(settings, tls_mode(settings, 1));
    if tls_mode(settings, 1) != 2 && outbound["tls"].get("server_name").is_none() {
        if let Some(name) = dg_str(settings, &["server_name"]) {
            outbound["tls"]["server_name"] = json!(name);
        }
        if dg_bool(settings, &["allow_insecure"]) {
            outbound["tls"]["insecure"] = json!(true);
        }
    }
    sb_append_multiplex(&mut outbound, settings);
    if let Some(transport) = sb_transport(settings, server) {
        outbound["transport"] = transport;
    }
    outbound
}

pub(crate) fn singbox_hysteria(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut outbound = sb_base("hysteria", server);
    let mut tls = json!({
        "enabled": true,
        "insecure": dg_bool(settings, &["tls", "allow_insecure"]),
    });
    if let Some(name) = dg_str(settings, &["tls", "server_name"]) {
        tls["server_name"] = json!(name);
    }
    sb_append_ech(&mut tls, dg(settings, &["tls", "ech"]));
    outbound["tls"] = tls;
    if let Some(ports) = server_ports(settings) {
        outbound["server_ports"] = json!([ports.replace('-', ":")]);
    }
    if let Some(interval) = dg_int(settings, &["hop_interval"]).filter(|v| *v > 0) {
        outbound["hop_interval"] = json!(format!("{interval}s"));
    }
    outbound["up_mbps"] = dg(settings, &["bandwidth", "up"]).cloned().unwrap_or(Value::Null);
    outbound["down_mbps"] = dg(settings, &["bandwidth", "down"]).cloned().unwrap_or(Value::Null);
    if dg_int(settings, &["version"]).unwrap_or(1) == 2 {
        outbound["type"] = json!("hysteria2");
        outbound["password"] = json!(uuid);
        if dg_bool(settings, &["obfs", "open"]) {
            outbound["obfs"] = json!({
                "type": dg_str(settings, &["obfs", "type"]).unwrap_or("salamander"),
                "password": dg_str(settings, &["obfs", "password"]),
            });
        }
    } else {
        outbound["auth_str"] = json!(uuid);
        outbound["obfs"] = dg(settings, &["obfs", "password"]).cloned().unwrap_or(Value::Null);
        outbound["disable_mtu_discovery"] = json!(true);
    }
    strip_nulls(outbound)
}

fn alpn_or_h3(settings: &Map<String, Value>) -> Value {
    dg(settings, &["alpn"]).cloned().unwrap_or_else(|| json!(["h3"]))
}

pub(crate) fn singbox_tuic(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut outbound = sb_base("tuic", server);
    outbound["congestion_control"] = json!(dg_str(settings, &["congestion_control"]).unwrap_or("cubic"));
    outbound["udp_relay_mode"] = json!(dg_str(settings, &["udp_relay_mode"]).unwrap_or("native"));
    outbound["zero_rtt_handshake"] = json!(true);
    outbound["heartbeat"] = json!("10s");
    let mut tls = json!({
        "enabled": true,
        "insecure": dg_bool(settings, &["tls", "allow_insecure"]),
        "alpn": alpn_or_h3(settings),
    });
    if let Some(name) = dg_str(settings, &["tls", "server_name"]) {
        tls["server_name"] = json!(name);
    }
    sb_append_ech(&mut tls, dg(settings, &["tls", "ech"]));
    outbound["tls"] = tls;
    if dg_int(settings, &["version"]) == Some(4) {
        outbound["token"] = json!(uuid);
    } else {
        outbound["uuid"] = json!(uuid);
        outbound["password"] = json!(uuid);
    }
    outbound
}

pub(crate) fn singbox_anytls(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut outbound = sb_base("anytls", server);
    outbound["password"] = json!(uuid);
    let mut tls = json!({
        "enabled": true,
        "insecure": dg_bool(settings, &["tls", "allow_insecure"]),
        "alpn": alpn_or_h3(settings),
    });
    if let Some(name) = dg_str(settings, &["tls", "server_name"]) {
        tls["server_name"] = json!(name);
    }
    sb_append_ech(&mut tls, dg(settings, &["tls", "ech"]));
    outbound["tls"] = tls;
    outbound
}

pub(crate) fn singbox_socks(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut outbound = sb_base("socks", server);
    outbound["version"] = json!("5");
    outbound["username"] = json!(uuid);
    outbound["password"] = json!(uuid);
    if dg_bool(settings, &["udp_over_tcp"]) {
        outbound["udp_over_tcp"] = json!(true);
    }
    outbound
}

fn meta_append_utls(proxy: &mut Value, settings: &Map<String, Value>) {
    if let Some(fingerprint) = tls_fingerprint(settings) {
        proxy["client-fingerprint"] = json!(fingerprint);
    }
}

fn meta_append_ech(proxy: &mut Value, ech: Option<&Value>) {
    if let Some(ech) = ech_settings(ech) {
        proxy["ech-opts"] = strip_nulls(json!({
            "enable": true,
            "config": ech.config.as_deref().and_then(mihomo_ech_config),
            "query-server-name": ech.query_server_name,
        }));
    }
}

fn meta_append_multiplex(proxy: &mut Value, settings: &Map<String, Value>) {
    let Some(multiplex) = dg(settings, &["multiplex"]) else {
        return;
    };
    if !truthy(multiplex.get("enabled")) {
        return;
    }
    let mut smux = strip_nulls(json!({
        "enabled": true,
        "protocol": multiplex.get("protocol").and_then(Value::as_str).unwrap_or("yamux"),
        "max-connections": multiplex.get("max_connections"),
        "padding": truthy(multiplex.get("padding")).then_some(true),
    }));
    if truthy(multiplex.get("brutal").and_then(|b| b.get("enabled"))) {
        smux["brutal-opts"] = json!({
            "enabled": true,
            "up": multiplex.get("brutal").and_then(|b| b.get("up_mbps")),
            "down": multiplex.get("brutal").and_then(|b| b.get("down_mbps")),
        });
    }
    proxy["smux"] = smux;
}

fn host_list(value: Value) -> Value {
    if value.is_array() { value } else { json!([value]) }
}

fn meta_transport(proxy: &mut Value, settings: &Map<String, Value>, allow_xhttp: bool, default_tcp: bool) {
    let ns = |key: &[&str]| {
        let mut path = vec!["network_settings"];
        path.extend_from_slice(key);
        dg(settings, &path).cloned().filter(|v| truthy(Some(v)))
    };
    match dg_str(settings, &["network"]) {
        Some("tcp") => {
            proxy["network"] = json!("tcp");
            if dg_str(settings, &["network_settings", "header", "type"]) == Some("http") {
                proxy["network"] = json!("http");
                let http_opts = strip_nulls(json!({
                    "headers": ns(&["header", "request", "headers"]),
                    "path": ns(&["header", "request", "path"]).unwrap_or_else(|| json!(["/"])),
                }));
                proxy["http-opts"] = http_opts;
            }
        }
        Some("ws") => {
            proxy["network"] = json!("ws");
            let opts = strip_nulls(json!({
                "path": ns(&["path"]),
                "headers": ns(&["headers", "Host"]).map(|host| json!({ "Host": host })),
            }));
            if opts.as_object().is_some_and(|o| !o.is_empty()) {
                proxy["ws-opts"] = opts;
            }
        }
        Some("grpc") => {
            proxy["network"] = json!("grpc");
            if let Some(name) = ns(&["serviceName"]) {
                proxy["grpc-opts"] = json!({ "grpc-service-name": name });
            }
        }
        Some("h2") => {
            proxy["network"] = json!("h2");
            proxy["h2-opts"] = strip_nulls(json!({
                "path": ns(&["path"]),
                "host": ns(&["host"]).map(host_list),
            }));
        }
        Some("httpupgrade") => {
            proxy["network"] = json!("ws");
            proxy["ws-opts"] = strip_nulls(json!({
                "v2ray-http-upgrade": true,
                "path": ns(&["path"]),
                "headers": ns(&["host"]).map(|host| json!({ "Host": host })),
            }));
        }
        Some("xhttp") if allow_xhttp => {
            proxy["network"] = json!("xhttp");
            let opts = strip_nulls(json!({
                "path": ns(&["path"]),
                "host": ns(&["host"]),
                "mode": ns(&["mode"]),
            }));
            if opts.as_object().is_some_and(|o| !o.is_empty()) {
                proxy["xhttp-opts"] = opts;
            }
        }
        _ if default_tcp => proxy["network"] = json!("tcp"),
        _ => {}
    }
}

fn meta_base(kind: &str, server: &ServerNodeRow) -> Value {
    json!({
        "name": server.name,
        "type": kind,
        "server": server.host,
        "port": server.port,
    })
}

pub(crate) fn meta_vmess(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut proxy = meta_base("vmess", server);
    proxy["uuid"] = json!(uuid);
    proxy["alterId"] = json!(0);
    proxy["cipher"] = json!("auto");
    proxy["udp"] = json!(true);
    if dg_bool(settings, &["tls"]) {
        proxy["tls"] = json!(true);
        proxy["skip-cert-verify"] = json!(dg_bool(settings, &["tls_settings", "allow_insecure"]));
        if let Some(name) = dg_str(settings, &["tls_settings", "server_name"]) {
            proxy["servername"] = json!(name);
        }
        meta_append_ech(&mut proxy, dg(settings, &["tls_settings", "ech"]));
    }
    meta_append_utls(&mut proxy, settings);
    meta_append_multiplex(&mut proxy, settings);
    meta_transport(&mut proxy, settings, false, false);
    proxy
}

fn vless_encryption(settings: &Map<String, Value>) -> String {
    if dg(settings, &["encryption", "enabled"]).and_then(Value::as_bool) == Some(true) {
        dg_str(settings, &["encryption", "encryption"]).unwrap_or("none").to_string()
    } else {
        "none".to_string()
    }
}

pub(crate) fn meta_vless(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut proxy = meta_base("vless", server);
    proxy["uuid"] = json!(uuid);
    proxy["alterId"] = json!(0);
    proxy["cipher"] = json!("auto");
    proxy["udp"] = json!(true);
    if let Some(flow) = dg_str(settings, &["flow"]) {
        proxy["flow"] = json!(flow);
    }
    proxy["encryption"] = json!(vless_encryption(settings));
    proxy["tls"] = json!(false);
    match tls_mode(settings, 0) {
        1 => {
            proxy["tls"] = json!(true);
            proxy["skip-cert-verify"] = json!(dg_bool(settings, &["tls_settings", "allow_insecure"]));
            if let Some(name) = dg_str(settings, &["tls_settings", "server_name"]) {
                proxy["servername"] = json!(name);
            }
            meta_append_ech(&mut proxy, dg(settings, &["tls_settings", "ech"]));
            meta_append_utls(&mut proxy, settings);
        }
        2 => {
            proxy["tls"] = json!(true);
            proxy["skip-cert-verify"] = json!(dg_bool(settings, &["reality_settings", "allow_insecure"]));
            proxy["servername"] = json!(dg_str(settings, &["reality_settings", "server_name"]));
            proxy["reality-opts"] = json!({
                "public-key": dg_str(settings, &["reality_settings", "public_key"]),
                "short-id": dg_str(settings, &["reality_settings", "short_id"]),
            });
            meta_append_utls(&mut proxy, settings);
        }
        _ => {}
    }
    meta_transport(&mut proxy, settings, true, false);
    meta_append_multiplex(&mut proxy, settings);
    proxy
}

pub(crate) fn meta_trojan(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut proxy = meta_base("trojan", server);
    proxy["password"] = json!(uuid);
    proxy["udp"] = json!(true);
    if tls_mode(settings, 1) == 2 {
        proxy["skip-cert-verify"] = json!(dg_bool(settings, &["reality_settings", "allow_insecure"]));
        if let Some(name) = dg_str(settings, &["reality_settings", "server_name"]) {
            proxy["sni"] = json!(name);
        }
        proxy["reality-opts"] = json!({
            "public-key": dg_str(settings, &["reality_settings", "public_key"]),
            "short-id": dg_str(settings, &["reality_settings", "short_id"]),
        });
    } else {
        proxy["skip-cert-verify"] = json!(
            dg_bool(settings, &["tls_settings", "allow_insecure"]) || dg_bool(settings, &["allow_insecure"])
        );
        if let Some(name) = dg_str(settings, &["tls_settings", "server_name"]).or_else(|| dg_str(settings, &["server_name"])) {
            proxy["sni"] = json!(name);
        }
        meta_append_ech(&mut proxy, dg(settings, &["tls_settings", "ech"]));
    }
    meta_append_utls(&mut proxy, settings);
    meta_append_multiplex(&mut proxy, settings);
    meta_transport(&mut proxy, settings, false, true);
    proxy
}

pub(crate) fn meta_hysteria(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut proxy = meta_base("hysteria", server);
    proxy["sni"] = json!(dg_str(settings, &["tls", "server_name"]));
    proxy["up"] = dg(settings, &["bandwidth", "up"]).cloned().unwrap_or(Value::Null);
    proxy["down"] = dg(settings, &["bandwidth", "down"]).cloned().unwrap_or(Value::Null);
    proxy["skip-cert-verify"] = json!(dg_bool(settings, &["tls", "allow_insecure"]));
    if let Some(ports) = server_ports(settings) {
        proxy["ports"] = json!(ports);
    }
    if let Some(interval) = dg_int(settings, &["hop_interval"]).filter(|v| *v > 0) {
        proxy["hop-interval"] = json!(interval);
    }
    if dg_int(settings, &["version"]).unwrap_or(1) == 2 {
        proxy["type"] = json!("hysteria2");
        proxy["password"] = json!(uuid);
        if dg_bool(settings, &["obfs", "open"]) {
            proxy["obfs"] = json!(dg_str(settings, &["obfs", "type"]).unwrap_or("salamander"));
            proxy["obfs-password"] = json!(dg_str(settings, &["obfs", "password"]));
        }
    } else {
        proxy["auth_str"] = json!(uuid);
        proxy["protocol"] = json!("udp");
        if dg_bool(settings, &["obfs", "open"]) {
            proxy["obfs"] = json!(dg_str(settings, &["obfs", "password"]));
        }
        proxy["fast-open"] = json!(true);
        proxy["disable_mtu_discovery"] = json!(true);
    }
    strip_nulls(proxy)
}

pub(crate) fn meta_tuic(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut proxy = meta_base("tuic", server);
    proxy["udp"] = json!(true);
    if dg_int(settings, &["version"]) == Some(4) {
        proxy["token"] = json!(uuid);
    } else {
        proxy["uuid"] = json!(uuid);
        proxy["password"] = json!(uuid);
    }
    proxy["skip-cert-verify"] = json!(dg_bool(settings, &["tls", "allow_insecure"]));
    if let Some(name) = dg_str(settings, &["tls", "server_name"]) {
        proxy["sni"] = json!(name);
    }
    if let Some(alpn) = dg(settings, &["alpn"]).filter(|v| truthy(Some(v))) {
        proxy["alpn"] = alpn.clone();
    }
    proxy["congestion-controller"] = json!(dg_str(settings, &["congestion_control"]).unwrap_or("cubic"));
    proxy["udp-relay-mode"] = json!(dg_str(settings, &["udp_relay_mode"]).unwrap_or("native"));
    proxy
}

pub(crate) fn meta_anytls(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut proxy = meta_base("anytls", server);
    proxy["password"] = json!(uuid);
    proxy["udp"] = json!(true);
    if let Some(name) = dg_str(settings, &["tls", "server_name"]) {
        proxy["sni"] = json!(name);
    }
    if dg_bool(settings, &["tls", "allow_insecure"]) {
        proxy["skip-cert-verify"] = json!(true);
    }
    meta_append_ech(&mut proxy, dg(settings, &["tls", "ech"]));
    proxy
}

pub(crate) fn meta_mieru(uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut proxy = meta_base("mieru", server);
    proxy["username"] = json!(uuid);
    proxy["password"] = json!(uuid);
    proxy["transport"] = json!(dg_str(settings, &["transport"]).unwrap_or("TCP").to_ascii_uppercase());
    if let Some(ports) = server_ports(settings) {
        proxy["port-range"] = json!(ports);
    }
    proxy
}

pub(crate) fn meta_socks5_or_http(kind: &str, uuid: &str, server: &ServerNodeRow, settings: &Map<String, Value>) -> Value {
    let mut proxy = meta_base(kind, server);
    if kind == "socks5" {
        proxy["udp"] = json!(true);
    }
    proxy["username"] = json!(uuid);
    proxy["password"] = json!(uuid);
    if dg_bool(settings, &["tls"]) {
        proxy["tls"] = json!(true);
        proxy["skip-cert-verify"] = json!(dg_bool(settings, &["tls_settings", "allow_insecure"]));
    }
    proxy
}

fn uri_query(params: Vec<(&str, String)>) -> String {
    serde_urlencoded::to_string(params).unwrap_or_default()
}

fn general_transport_params(params: &mut Vec<(&'static str, String)>, settings: &Map<String, Value>, server: &ServerNodeRow) {
    let ns_str = |key: &[&str]| {
        let mut path = vec!["network_settings"];
        path.extend_from_slice(key);
        dg_str(settings, &path).map(str::to_string)
    };
    let host_value = || match dg(settings, &["network_settings", "host"]) {
        Some(Value::Array(hosts)) => Some(hosts.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(",")),
        Some(Value::String(host)) if !host.is_empty() => Some(host.clone()),
        _ => None,
    };
    match dg_str(settings, &["network"]) {
        Some("ws") => {
            if let Some(path) = ns_str(&["path"]) { params.push(("path", path)); }
            if let Some(host) = ns_str(&["headers", "Host"]) { params.push(("host", host)); }
        }
        Some("grpc") => {
            if let Some(name) = ns_str(&["serviceName"]) { params.push(("serviceName", name)); }
        }
        Some("h2") => {
            params.retain(|(key, _)| *key != "type");
            params.push(("type", "http".to_string()));
            if let Some(path) = ns_str(&["path"]) { params.push(("path", path)); }
            if let Some(host) = host_value() { params.push(("host", host)); }
        }
        Some("kcp") => {
            params.retain(|(key, _)| *key != "type");
            if let Some(seed) = ns_str(&["seed"]) { params.push(("path", seed)); }
            params.push(("type", ns_str(&["header", "type"]).unwrap_or_else(|| "none".to_string())));
        }
        Some("httpupgrade") => {
            if let Some(path) = ns_str(&["path"]) { params.push(("path", path)); }
            params.push(("host", host_value().unwrap_or_else(|| server.host.clone())));
        }
        Some("xhttp") => {
            if let Some(path) = ns_str(&["path"]) { params.push(("path", path)); }
            params.push(("host", host_value().unwrap_or_else(|| server.host.clone())));
            params.retain(|(key, _)| *key != "mode");
            params.push(("mode", ns_str(&["mode"]).unwrap_or_else(|| "auto".to_string())));
            if let Some(extra) = dg(settings, &["network_settings", "extra"]).filter(|v| v.as_object().is_some_and(|o| !o.is_empty())) {
                params.push(("extra", extra.to_string()));
            }
        }
        _ => {}
    }
}

pub(crate) fn general_vless(uuid: &str, server: &ServerNodeRow, name: &str, settings: &Map<String, Value>) -> String {
    let network = dg_str(settings, &["network"]).unwrap_or("tcp").to_string();
    let mut params: Vec<(&'static str, String)> = vec![
        ("mode", "multi".to_string()),
        ("encryption", vless_encryption(settings)),
        ("type", network),
    ];
    if let Some(flow) = dg_str(settings, &["flow"]) {
        params.push(("flow", flow.to_string()));
    }
    match tls_mode(settings, 0) {
        1 => {
            params.push(("security", "tls".to_string()));
            if let Some(fp) = tls_fingerprint(settings) { params.push(("fp", fp)); }
            if let Some(sni) = dg_str(settings, &["tls_settings", "server_name"]) { params.push(("sni", sni.to_string())); }
            if dg_bool(settings, &["tls_settings", "allow_insecure"]) { params.push(("allowInsecure", "1".to_string())); }
        }
        2 => {
            params.push(("security", "reality".to_string()));
            let reality = |key: &str| dg_str(settings, &["reality_settings", key]).unwrap_or("").to_string();
            params.push(("pbk", reality("public_key")));
            params.push(("sid", reality("short_id")));
            params.push(("sni", reality("server_name")));
            params.push(("servername", reality("server_name")));
            params.push(("spx", "/".to_string()));
            if let Some(fp) = tls_fingerprint(settings) { params.push(("fp", fp)); }
        }
        _ => {}
    }
    general_transport_params(&mut params, settings, server);
    format!(
        "vless://{}@{}:{}?{}#{}\r\n",
        uuid,
        wrap_ipv6(&server.host),
        server.port,
        uri_query(params),
        urlencoding::encode(name)
    )
}

pub(crate) fn general_trojan(uuid: &str, server: &ServerNodeRow, name: &str, settings: &Map<String, Value>) -> String {
    let mut params: Vec<(&'static str, String)> = Vec::new();
    if tls_mode(settings, 1) == 2 {
        let reality = |key: &str| dg_str(settings, &["reality_settings", key]).unwrap_or("").to_string();
        params.push(("security", "reality".to_string()));
        params.push(("pbk", reality("public_key")));
        params.push(("sid", reality("short_id")));
        params.push(("sni", reality("server_name")));
    } else {
        let insecure = dg_bool(settings, &["tls_settings", "allow_insecure"]) || dg_bool(settings, &["allow_insecure"]);
        params.push(("allowInsecure", if insecure { "1" } else { "0" }.to_string()));
        if let Some(sni) = dg_str(settings, &["tls_settings", "server_name"]).or_else(|| dg_str(settings, &["server_name"])) {
            params.push(("peer", sni.to_string()));
            params.push(("sni", sni.to_string()));
        }
    }
    if let Some(fp) = tls_fingerprint(settings) { params.push(("fp", fp)); }
    if let Some(network) = dg_str(settings, &["network"]).filter(|n| *n != "tcp") {
        params.push(("type", network.to_string()));
    }
    general_transport_params(&mut params, settings, server);
    format!(
        "trojan://{}@{}:{}?{}#{}\r\n",
        uuid,
        wrap_ipv6(&server.host),
        server.port,
        uri_query(params),
        urlencoding::encode(name)
    )
}

pub(crate) fn general_hysteria(uuid: &str, server: &ServerNodeRow, name: &str, settings: &Map<String, Value>) -> String {
    let mut params: Vec<(&'static str, String)> = Vec::new();
    if let Some(sni) = dg_str(settings, &["tls", "server_name"]) {
        params.push(("sni", sni.to_string()));
    }
    params.push(("insecure", if dg_bool(settings, &["tls", "allow_insecure"]) { "1" } else { "0" }.to_string()));
    let addr = wrap_ipv6(&server.host);
    let encoded_name = urlencoding::encode(name);
    if dg_int(settings, &["version"]).unwrap_or(2) == 2 {
        if dg_bool(settings, &["obfs", "open"]) {
            params.push(("obfs", "salamander".to_string()));
            params.push(("obfs-password", dg_str(settings, &["obfs", "password"]).unwrap_or("").to_string()));
        }
        if let Some(ports) = server_ports(settings) {
            params.push(("mport", ports));
        }
        format!("hysteria2://{}@{}:{}?{}#{}\r\n", uuid, addr, server.port, uri_query(params), encoded_name)
    } else {
        params.push(("protocol", "udp".to_string()));
        params.push(("auth", uuid.to_string()));
        if let Some(up) = dg_int(settings, &["bandwidth", "up"]).filter(|v| *v > 0) { params.push(("upmbps", up.to_string())); }
        if let Some(down) = dg_int(settings, &["bandwidth", "down"]).filter(|v| *v > 0) { params.push(("downmbps", down.to_string())); }
        if dg_bool(settings, &["obfs", "open"]) {
            if let Some(password) = dg_str(settings, &["obfs", "password"]) {
                params.push(("obfs", "xplus".to_string()));
                params.push(("obfsParam", password.to_string()));
            }
        }
        format!("hysteria://{}:{}?{}#{}\r\n", addr, server.port, uri_query(params), encoded_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(protocol: &str) -> ServerNodeRow {
        ServerNodeRow {
            id: 1,
            user_id: 1,
            name: "HK 01".to_string(),
            host: "example.com".to_string(),
            port: 443,
            service_port: None,
            protocol: protocol.to_string(),
            settings: None,
            access_control: None,
            device_limit: 0,
            connection_limit: 0,
            speed_limit_down: 0,
            created_at: None,
        }
    }

    fn settings(value: Value) -> Map<String, Value> {
        value.as_object().cloned().unwrap()
    }

    #[test]
    fn vless_reality_xhttp_encryption_across_formats() {
        let s = settings(json!({
            "tls": 2,
            "flow": "xtls-rprx-vision",
            "network": "xhttp",
            "network_settings": { "path": "/xh", "host": "cdn.example.com", "mode": "stream-one" },
            "encryption": { "enabled": true, "encryption": "mlkem768x25519plus.native.0rtt.abc" },
            "reality_settings": { "server_name": "www.apple.com", "public_key": "PBK", "short_id": "ab" },
            "utls": { "enabled": true, "fingerprint": "safari" },
            "multiplex": { "enabled": true, "protocol": "h2mux", "padding": true },
        }));
        let meta = meta_vless("uuid", &node("vless"), &s);
        assert_eq!(meta["network"], json!("xhttp"));
        assert_eq!(meta["xhttp-opts"]["mode"], json!("stream-one"));
        assert_eq!(meta["encryption"], json!("mlkem768x25519plus.native.0rtt.abc"));
        assert_eq!(meta["reality-opts"]["public-key"], json!("PBK"));
        assert_eq!(meta["client-fingerprint"], json!("safari"));
        assert_eq!(meta["smux"]["protocol"], json!("h2mux"));

        let sb = singbox_vless("uuid", &node("vless"), &s);
        assert_eq!(sb["tls"]["reality"]["short_id"], json!("ab"));
        assert_eq!(sb["tls"]["utls"]["fingerprint"], json!("safari"));
        assert_eq!(sb["multiplex"]["padding"], json!(true));

        let uri = general_vless("uuid", &node("vless"), "HK 01", &s);
        assert!(uri.contains("type=xhttp"));
        assert!(uri.contains("mode=stream-one"));
        assert!(uri.contains("security=reality"));
        assert!(uri.contains("encryption=mlkem768x25519plus.native.0rtt.abc"));
    }

    #[test]
    fn trojan_reality_and_ech() {
        let reality = settings(json!({
            "tls": 2,
            "network": "grpc",
            "network_settings": { "serviceName": "svc" },
            "reality_settings": { "server_name": "www.apple.com", "public_key": "PBK", "short_id": "ab" },
        }));
        let meta = meta_trojan("pw", &node("trojan"), &reality);
        assert_eq!(meta["reality-opts"]["short-id"], json!("ab"));
        assert_eq!(meta["grpc-opts"]["grpc-service-name"], json!("svc"));
        let sb = singbox_trojan("pw", &node("trojan"), &reality);
        assert_eq!(sb["tls"]["reality"]["enabled"], json!(true));
        assert_eq!(sb["transport"]["service_name"], json!("svc"));

        let ech = settings(json!({
            "tls": 1,
            "tls_settings": {
                "server_name": "t.example.com",
                "ech": { "enabled": true, "config": "-----BEGIN ECH CONFIGS-----\nAAA\nBBB\n-----END ECH CONFIGS-----", "query_server_name": "q.example.com" }
            }
        }));
        let meta = meta_trojan("pw", &node("trojan"), &ech);
        assert_eq!(meta["ech-opts"]["config"], json!("AAABBB"));
        assert_eq!(meta["sni"], json!("t.example.com"));
        let sb = singbox_trojan("pw", &node("trojan"), &ech);
        assert_eq!(sb["tls"]["ech"]["query_server_name"], json!("q.example.com"));
    }

    #[test]
    fn hysteria2_bandwidth_ports_and_tuic_congestion() {
        let s = settings(json!({
            "version": 2,
            "bandwidth": { "up": 100, "down": 500 },
            "obfs": { "open": true, "type": "salamander", "password": "ob" },
            "tls": { "server_name": "h.example.com" },
            "ports": "20000-30000",
            "hop_interval": 30,
        }));
        let sb = singbox_hysteria("pw", &node("hysteria"), &s);
        assert_eq!(sb["type"], json!("hysteria2"));
        assert_eq!(sb["up_mbps"], json!(100));
        assert_eq!(sb["server_ports"], json!(["20000:30000"]));
        assert_eq!(sb["hop_interval"], json!("30s"));
        let meta = meta_hysteria("pw", &node("hysteria"), &s);
        assert_eq!(meta["down"], json!(500));
        assert_eq!(meta["obfs-password"], json!("ob"));
        assert!(general_hysteria("pw", &node("hysteria"), "n", &s).contains("mport=20000-30000"));

        let tuic = settings(json!({ "congestion_control": "bbr", "tls": { "server_name": "t" } }));
        assert_eq!(singbox_tuic("pw", &node("tuic"), &tuic)["congestion_control"], json!("bbr"));
        assert_eq!(meta_tuic("pw", &node("tuic"), &tuic)["congestion-controller"], json!("bbr"));
    }
}
