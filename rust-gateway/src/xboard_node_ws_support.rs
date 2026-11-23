use crate::*;
use crate::uniproxy_support::{
    authenticate_node_credentials, build_node_config_payload, load_uniproxy_device_ips, node_credentials,
    process_node_alive, process_node_metrics, process_node_status,
    NodeCredentials,
};
use crate::uniproxy_user_support::{load_or_build_uniproxy_user_snapshot, UniProxyUserItem};
use crate::xboard_node_machine_support::{
    authenticate_machine, load_machine_server_nodes, machine_node_list,
};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use std::collections::HashMap;

const WS_PATH: &str = "/api/v2/server/ws";
const PING_INTERVAL_SECS: u64 = 55;

pub(crate) async fn handshake_websocket_settings(state: &AppState, headers: &HeaderMap) -> Value {
    if !get_setting_bool(state, "server_ws_enable", true).await {
        return json!({ "enabled": false });
    }
    let configured = get_setting_string(state, "server_ws_url", "").await;
    let ws_url = if !configured.trim().is_empty() {
        configured.trim().to_string()
    } else {
        let base = get_setting_string(state, "app_url", "").await;
        let base = if base.trim().is_empty() {
            request_base_url(headers)
        } else {
            Some(base.trim().trim_end_matches('/').to_string())
        };
        match base {
            Some(base) => format!("{}{}", http_to_ws_url(&base), WS_PATH),
            None => return json!({ "enabled": false }),
        }
    };
    json!({ "enabled": true, "ws_url": ws_url })
}

fn http_to_ws_url(base: &str) -> String {
    if let Some(rest) = base.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = base.strip_prefix("http://") {
        format!("ws://{rest}")
    } else {
        base.to_string()
    }
}

fn request_base_url(headers: &HeaderMap) -> Option<String> {
    let header = |name: &str| {
        headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.split(',').next().unwrap_or("").trim().to_string())
            .filter(|value| !value.is_empty())
    };
    let host = header("x-forwarded-host").or_else(|| header("host"))?;
    if !host.bytes().all(|b| b.is_ascii_alphanumeric() || b"-.:[]".contains(&b)) {
        return None;
    }
    let scheme = match header("x-forwarded-proto").as_deref() {
        Some("https") => "https",
        _ => "http",
    };
    Some(format!("{scheme}://{host}"))
}

#[derive(Clone)]
enum WsSubject {
    Node(NodeCredentials),
    Machine { machine_id: String, token: String },
}

pub(crate) async fn server_ws(
    State(state): State<Arc<AppState>>,
    uri: Uri,
    ws: WebSocketUpgrade,
) -> Response<Body> {
    if !get_setting_bool(&state, "server_ws_enable", true).await {
        return json_error(StatusCode::NOT_FOUND, "WebSocket sync is disabled");
    }
    let credentials = node_credentials(&uri, &Value::Null);
    let subject = if !credentials.machine_id.trim().is_empty() && credentials.node_id.trim().is_empty() {
        WsSubject::Machine {
            machine_id: credentials.machine_id.clone(),
            token: credentials.token.clone(),
        }
    } else {
        WsSubject::Node(credentials)
    };
    let auth_success = match resolve_nodes(&state, &subject).await {
        Ok((machine_id, nodes)) => match machine_id {
            Some(machine_id) => json!({
                "event": "auth.success",
                "data": {
                    "machine_id": machine_id,
                    "node_ids": nodes.iter().map(|node| node.id).collect::<Vec<_>>(),
                },
            }),
            None => json!({
                "event": "auth.success",
                "data": { "node_id": nodes.first().map(|node| node.id) },
            }),
        },
        Err(response) => return response,
    };
    ws.on_upgrade(move |socket| run_socket(state, socket, subject, auth_success))
}

async fn resolve_nodes(
    state: &AppState,
    subject: &WsSubject,
) -> Result<(Option<u64>, Vec<ServerNodeRow>), Response<Body>> {
    match subject {
        WsSubject::Node(credentials) => {
            let node = authenticate_node_credentials(state, credentials.clone()).await?;
            Ok((None, vec![node]))
        }
        WsSubject::Machine { machine_id, token } => {
            let machine = authenticate_machine(state, machine_id, token).await?;
            let nodes = load_machine_server_nodes(state, &machine).await?;
            Ok((Some(machine.id), nodes))
        }
    }
}

#[derive(Default)]
struct SyncState {
    node_list: Option<String>,
    config: HashMap<u64, String>,
    users: HashMap<u64, String>,
    user_items: HashMap<u64, Vec<UniProxyUserItem>>,
    devices: HashMap<u64, String>,
    nodes: Vec<ServerNodeRow>,
}

fn event_message(event: &str, data: Value) -> Message {
    Message::Text(json!({ "event": event, "data": data }).to_string().into())
}

async fn run_socket(state: Arc<AppState>, mut socket: WebSocket, subject: WsSubject, auth_success: Value) {
    if socket.send(Message::Text(auth_success.to_string().into())).await.is_err() {
        return;
    }
    let sync_secs = get_setting_int(&state, "server_ws_sync_interval", 10).await.clamp(3, 120) as u64;
    let mut sync_tick = tokio::time::interval(Duration::from_secs(sync_secs));
    let mut ping_tick = tokio::time::interval_at(
        tokio::time::Instant::now() + Duration::from_secs(PING_INTERVAL_SECS),
        Duration::from_secs(PING_INTERVAL_SECS),
    );
    let mut sync = SyncState::default();
    loop {
        tokio::select! {
            _ = sync_tick.tick() => {
                match collect_sync_messages(&state, &subject, &mut sync).await {
                    Ok(messages) => {
                        for message in messages {
                            if socket.send(message).await.is_err() {
                                return;
                            }
                        }
                    }
                    Err(message) => {
                        let _ = socket.send(event_message("error", json!({ "message": message }))).await;
                        let _ = socket.send(Message::Close(None)).await;
                        return;
                    }
                }
            }
            _ = ping_tick.tick() => {
                if socket.send(event_message("ping", Value::Null)).await.is_err() {
                    return;
                }
            }
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => handle_incoming(&state, &sync, text.as_str()).await,
                    Some(Ok(Message::Ping(payload))) => {
                        if socket.send(Message::Pong(payload)).await.is_err() {
                            return;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => return,
                    Some(Ok(_)) => {}
                }
            }
        }
    }
}

async fn collect_sync_messages(
    state: &AppState,
    subject: &WsSubject,
    sync: &mut SyncState,
) -> Result<Vec<Message>, String> {
    let (machine_id, nodes) = resolve_nodes(state, subject)
        .await
        .map_err(|_| "authentication revoked".to_string())?;
    let mut messages = Vec::new();
    let timestamp = Utc::now().timestamp();
    if machine_id.is_some() {
        let list = machine_node_list(&nodes);
        let digest = sha256_hex(&list.to_string());
        if sync.node_list.as_ref().is_some_and(|previous| previous != &digest) {
            messages.push(event_message("sync.nodes", json!({ "nodes": list })));
        }
        sync.node_list = Some(digest);
    }
    let node_ids = nodes.iter().map(|node| node.id).collect::<HashSet<_>>();
    sync.config.retain(|id, _| node_ids.contains(id));
    sync.users.retain(|id, _| node_ids.contains(id));
    sync.user_items.retain(|id, _| node_ids.contains(id));
    sync.devices.retain(|id, _| node_ids.contains(id));
    for node in &nodes {
        if let Ok(config) = build_node_config_payload(state, node).await {
            let digest = sha256_hex(&config.to_string());
            if sync.config.get(&node.id) != Some(&digest) {
                messages.push(event_message(
                    "sync.config",
                    json!({ "config": config, "node_id": node.id, "timestamp": timestamp }),
                ));
                sync.config.insert(node.id, digest);
            }
        }
        let cache_key = format!("uniproxy:user:ws:{}:node-credential-v1", node.id);
        if let Ok(snapshot) = load_or_build_uniproxy_user_snapshot(state, node, &cache_key, false).await {
            if sync.users.get(&node.id) != Some(&snapshot.version) {
                match sync
                    .user_items
                    .get(&node.id)
                    .and_then(|previous| compute_user_delta(previous, &snapshot.users))
                {
                    Some((removed, added)) => {
                        for (action, users) in [("remove", removed), ("add", added)] {
                            if !users.is_empty() {
                                messages.push(event_message(
                                    "sync.user.delta",
                                    json!({ "action": action, "users": users, "node_id": node.id, "timestamp": timestamp }),
                                ));
                            }
                        }
                    }
                    None => messages.push(event_message(
                        "sync.users",
                        json!({ "users": snapshot.users, "node_id": node.id, "timestamp": timestamp }),
                    )),
                }
                sync.users.insert(node.id, snapshot.version);
                sync.user_items.insert(node.id, snapshot.users);
            }
        }
        if let Ok(devices) = load_uniproxy_device_ips(state, node).await {
            let digest = sha256_hex(&Value::Object(devices.clone()).to_string());
            if sync.devices.get(&node.id) != Some(&digest) {
                messages.push(event_message(
                    "sync.devices",
                    json!({ "users": devices, "node_id": node.id, "timestamp": timestamp }),
                ));
                sync.devices.insert(node.id, digest);
            }
        }
    }
    sync.nodes = nodes;
    Ok(messages)
}

async fn handle_incoming(state: &AppState, sync: &SyncState, text: &str) {
    let Ok(message) = serde_json::from_str::<Value>(text) else {
        return;
    };
    let event = message.get("event").and_then(Value::as_str).unwrap_or_default();
    if event != "node.status" && event != "report.devices" {
        return;
    }
    let data = message.get("data").cloned().unwrap_or(Value::Null);
    let node_id = data
        .get("node_id")
        .and_then(Value::as_u64)
        .or_else(|| (sync.nodes.len() == 1).then(|| sync.nodes[0].id));
    let Some(node) = node_id.and_then(|id| sync.nodes.iter().find(|node| node.id == id)) else {
        return;
    };
    if event == "report.devices" {
        let devices = data.get("devices").cloned().unwrap_or(data);
        let alive = match devices {
            Value::Object(mut map) => {
                map.remove("node_id");
                Value::Object(map)
            }
            _ => return,
        };
        let _ = process_node_alive(state, node, &alive).await;
        return;
    }
    if let Some(status) = data.get("status").filter(|value| value.is_object()) {
        let _ = process_node_status(state, node, status);
    } else if data.get("mem").is_some() {
        let _ = process_node_status(state, node, &data);
    }
    process_node_metrics(state, node, &data);
}

#[cfg(test)]
mod tests {
    use super::http_to_ws_url;

    #[test]
    fn websocket_url_follows_panel_scheme() {
        assert_eq!(http_to_ws_url("https://panel.example.com"), "wss://panel.example.com");
        assert_eq!(http_to_ws_url("http://127.0.0.1:8000"), "ws://127.0.0.1:8000");
    }
}

/// Returns `(removed, added)` when the change is small enough to send as
/// `sync.user.delta`; `None` means a full `sync.users` snapshot should be sent.
fn compute_user_delta(
    previous: &[UniProxyUserItem],
    next: &[UniProxyUserItem],
) -> Option<(Vec<UniProxyUserItem>, Vec<UniProxyUserItem>)> {
    let previous_by_id = previous.iter().map(|user| (user.id, user)).collect::<HashMap<_, _>>();
    let next_ids = next.iter().map(|user| user.id).collect::<HashSet<_>>();
    let removed = previous
        .iter()
        .filter(|user| !next_ids.contains(&user.id))
        .cloned()
        .collect::<Vec<_>>();
    let added = next
        .iter()
        .filter(|user| previous_by_id.get(&user.id).is_none_or(|old| *old != *user))
        .cloned()
        .collect::<Vec<_>>();
    let changes = removed.len() + added.len();
    if changes == 0 || changes > (next.len() / 2).max(DELTA_MIN_THRESHOLD) {
        return None;
    }
    Some((removed, added))
}

const DELTA_MIN_THRESHOLD: usize = 50;

#[cfg(test)]
mod delta_tests {
    use super::*;

    fn user(id: i64, uuid: &str, speed: i64) -> UniProxyUserItem {
        UniProxyUserItem {
            id,
            uuid: uuid.to_string(),
            speed_limit: speed,
            device_limit: 0,
            connection_limit: 0,
            trust_level: 0,
            is_silenced: false,
        }
    }

    #[test]
    fn user_delta_reports_adds_changes_and_removals() {
        let previous = vec![user(1, "a", 0), user(2, "b", 0), user(3, "c", 0)];
        let next = vec![user(1, "a", 0), user(2, "b2", 10), user(4, "d", 0)];
        let (removed, added) = compute_user_delta(&previous, &next).unwrap();
        assert_eq!(removed.iter().map(|u| u.id).collect::<Vec<_>>(), vec![3]);
        assert_eq!(added.iter().map(|u| u.id).collect::<Vec<_>>(), vec![2, 4]);
        assert!(compute_user_delta(&next, &next).is_none());
    }
}
