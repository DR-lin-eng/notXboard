use crate::*;

const SERVER_NODE_COLUMNS: &str = "n.id, n.user_id, n.name, n.host, n.port, n.service_port, n.protocol,
    n.settings, n.access_control, n.device_limit, n.connection_limit,
    n.speed_limit_down, n.created_at";

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct ServerMachineRow {
    pub(crate) id: u64,
    pub(crate) user_id: i64,
    pub(crate) name: String,
}

pub(crate) async fn ensure_xboard_node_schema(state: &AppState) -> Result<bool, sqlx::Error> {
    let server_nodes_ready: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.tables
         WHERE table_schema = DATABASE() AND table_name = 'server_nodes'",
    )
    .fetch_one(&state.db)
    .await?;
    if server_nodes_ready == 0 {
        return Ok(false);
    }
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS `server_machines` (
          `id` bigint unsigned NOT NULL AUTO_INCREMENT,
          `user_id` int NOT NULL,
          `name` varchar(255) COLLATE utf8mb4_unicode_ci NOT NULL,
          `token` varchar(64) COLLATE utf8mb4_unicode_ci NOT NULL,
          `notes` text COLLATE utf8mb4_unicode_ci,
          `is_active` tinyint(1) NOT NULL DEFAULT '1',
          `last_seen_at` int unsigned DEFAULT NULL,
          `load_status` json DEFAULT NULL,
          `created_at` timestamp NULL DEFAULT NULL,
          `updated_at` timestamp NULL DEFAULT NULL,
          PRIMARY KEY (`id`),
          UNIQUE KEY `server_machines_token_unique` (`token`),
          KEY `server_machines_user_id_index` (`user_id`),
          CONSTRAINT `server_machines_user_id_foreign` FOREIGN KEY (`user_id`) REFERENCES `v2_user` (`id`) ON DELETE CASCADE
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
    )
    .execute(&state.db)
    .await?;
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS `server_machine_load_history` (
          `id` bigint unsigned NOT NULL AUTO_INCREMENT,
          `machine_id` bigint unsigned NOT NULL,
          `cpu` double NOT NULL DEFAULT '0',
          `mem_total` bigint unsigned NOT NULL DEFAULT '0',
          `mem_used` bigint unsigned NOT NULL DEFAULT '0',
          `disk_total` bigint unsigned NOT NULL DEFAULT '0',
          `disk_used` bigint unsigned NOT NULL DEFAULT '0',
          `net_in_speed` double DEFAULT NULL,
          `net_out_speed` double DEFAULT NULL,
          `recorded_at` int unsigned NOT NULL,
          PRIMARY KEY (`id`),
          KEY `server_machine_load_history_machine_recorded_index` (`machine_id`,`recorded_at`),
          CONSTRAINT `server_machine_load_history_machine_id_foreign` FOREIGN KEY (`machine_id`) REFERENCES `server_machines` (`id`) ON DELETE CASCADE
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
    )
    .execute(&state.db)
    .await?;
    let machine_column: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.columns
         WHERE table_schema = DATABASE() AND table_name = 'server_nodes' AND column_name = 'machine_id'",
    )
    .fetch_one(&state.db)
    .await?;
    if machine_column == 0 {
        sqlx::query(
            "ALTER TABLE `server_nodes`
             ADD COLUMN `machine_id` bigint unsigned DEFAULT NULL AFTER `v2bx_token`,
             ADD KEY `server_nodes_machine_id_index` (`machine_id`)",
        )
        .execute(&state.db)
        .await?;
    }
    Ok(true)
}

pub(crate) fn spawn_xboard_node_schema_task(state: AppState) {
    tokio::spawn(async move {
        loop {
            match ensure_xboard_node_schema(&state).await {
                Ok(true) => break,
                Ok(false) => {}
                Err(err) => error!("xboard-node schema ensure failed: {err}"),
            }
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
    });
}

pub(crate) async fn authenticate_machine(
    state: &AppState,
    machine_id: &str,
    token: &str,
) -> Result<ServerMachineRow, Response<Body>> {
    let token = token.trim();
    let machine_id = machine_id
        .trim()
        .parse::<u64>()
        .ok()
        .filter(|id| *id > 0)
        .ok_or_else(|| json_error(StatusCode::UNPROCESSABLE_ENTITY, "Missing token or machine_id"))?;
    if !crate::machine_bootstrap_support::is_strong_machine_token(token) {
        return Err(json_error(StatusCode::UNPROCESSABLE_ENTITY, "Missing token or machine_id"));
    }
    let machine = sqlx::query_as::<_, ServerMachineRow>(
        "SELECT machine.id, machine.user_id, machine.name
         FROM server_machines machine
         JOIN v2_user owner ON owner.id = machine.user_id
         WHERE machine.id = ? AND machine.token = ? AND machine.is_active = 1 AND owner.banned = 0
         LIMIT 1",
    )
    .bind(machine_id)
    .bind(token)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| json_error(StatusCode::FORBIDDEN, "Machine not found or disabled"))?;
    let _ = sqlx::query("UPDATE server_machines SET last_seen_at = ? WHERE id = ?")
        .bind(Utc::now().timestamp())
        .bind(machine.id)
        .execute(&state.db)
        .await;
    Ok(machine)
}

pub(crate) async fn load_machine_server_nodes(
    state: &AppState,
    machine: &ServerMachineRow,
) -> Result<Vec<ServerNodeRow>, Response<Body>> {
    sqlx::query_as::<_, ServerNodeRow>(&format!(
        "SELECT {SERVER_NODE_COLUMNS}
         FROM server_nodes n
         WHERE n.machine_id = ? AND n.user_id = ? AND n.status = 'active'
         ORDER BY n.id ASC"
    ))
    .bind(machine.id)
    .bind(machine.user_id)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)
}

pub(crate) async fn authenticate_machine_node(
    state: &AppState,
    machine_id: &str,
    token: &str,
    node_id: &str,
) -> Result<ServerNodeRow, Response<Body>> {
    let machine = authenticate_machine(state, machine_id, token).await?;
    let node_id = node_id
        .trim()
        .parse::<u64>()
        .map_err(|_| json_error(StatusCode::UNPROCESSABLE_ENTITY, "Missing token or node_id"))?;
    sqlx::query_as::<_, ServerNodeRow>(&format!(
        "SELECT {SERVER_NODE_COLUMNS}
         FROM server_nodes n
         WHERE n.id = ? AND n.machine_id = ? AND n.user_id = ? AND n.status = 'active'
         LIMIT 1"
    ))
    .bind(node_id)
    .bind(machine.id)
    .bind(machine.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| json_error(StatusCode::FORBIDDEN, "Node is not bound to this machine"))
}

pub(crate) fn machine_node_list(nodes: &[ServerNodeRow]) -> Value {
    Value::Array(
        nodes
            .iter()
            .filter(|node| server_node_supports_xboard_node_deploy(&node.protocol.to_ascii_lowercase()))
            .map(|node| {
                json!({
                    "id": node.id,
                    "type": node.protocol.to_ascii_lowercase(),
                    "name": node.name,
                })
            })
            .collect(),
    )
}

fn machine_credentials(uri: &Uri, payload: &Value) -> (String, String) {
    let params = parse_query(uri);
    let read = |key: &str| {
        params
            .get(key)
            .filter(|value| !value.is_empty())
            .cloned()
            .unwrap_or_else(|| match payload.get(key) {
                Some(Value::String(value)) => value.clone(),
                Some(Value::Number(value)) => value.to_string(),
                _ => String::new(),
            })
    };
    (read("machine_id"), read("token"))
}

pub(crate) async fn server_machine_nodes(
    State(state): State<Arc<AppState>>,
    method: Method,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    let payload = if method == Method::POST {
        parse_json_body(body).await.unwrap_or_else(|_| json!({}))
    } else {
        json!({})
    };
    let (machine_id, token) = machine_credentials(&uri, &payload);
    let result = async {
        let machine = authenticate_machine(&state, &machine_id, &token).await?;
        let nodes = load_machine_server_nodes(&state, &machine).await?;
        Ok::<_, Response<Body>>(json_value_response(json!({
            "nodes": machine_node_list(&nodes),
            "base_config": {
                "push_interval": get_setting_int(&state, "server_push_interval", 60).await,
                "pull_interval": get_setting_int(&state, "server_pull_interval", 60).await,
            },
        })))
    }
    .await;
    result.unwrap_or_else(|response| response)
}

pub(crate) async fn server_machine_status(
    State(state): State<Arc<AppState>>,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    let result = async {
        let payload = parse_json_body(body).await?;
        let (machine_id, token) = machine_credentials(&uri, &payload);
        let machine = authenticate_machine(&state, &machine_id, &token).await?;
        let load_status = machine_load_status(&payload)?;
        let recorded_at = Utc::now().timestamp();
        let mut status = load_status.clone();
        status["updated_at"] = json!(recorded_at);
        sqlx::query("UPDATE server_machines SET load_status = ?, last_seen_at = ? WHERE id = ?")
            .bind(SqlxJson(status))
            .bind(recorded_at)
            .bind(machine.id)
            .execute(&state.db)
            .await
            .map_err(internal_error)?;
        let number = |path: &[&str]| -> Option<f64> {
            path.iter()
                .try_fold(&load_status, |value, key| value.get(*key))
                .and_then(Value::as_f64)
        };
        sqlx::query(
            "INSERT INTO server_machine_load_history
             (machine_id, cpu, mem_total, mem_used, disk_total, disk_used, net_in_speed, net_out_speed, recorded_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(machine.id)
        .bind(number(&["cpu"]).unwrap_or(0.0))
        .bind(number(&["mem", "total"]).unwrap_or(0.0) as u64)
        .bind(number(&["mem", "used"]).unwrap_or(0.0) as u64)
        .bind(number(&["disk", "total"]).unwrap_or(0.0) as u64)
        .bind(number(&["disk", "used"]).unwrap_or(0.0) as u64)
        .bind(number(&["net", "in_speed"]))
        .bind(number(&["net", "out_speed"]))
        .bind(recorded_at)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
        if recorded_at % 20 == 0 {
            let _ = sqlx::query(
                "DELETE FROM server_machine_load_history WHERE machine_id = ? AND recorded_at < ?",
            )
            .bind(machine.id)
            .bind(recorded_at - 86_400)
            .execute(&state.db)
            .await;
        }
        Ok::<_, Response<Body>>(json_value_response(json!({ "data": true })))
    }
    .await;
    result.unwrap_or_else(|response| response)
}

fn machine_load_status(payload: &Value) -> Result<Value, Response<Body>> {
    let non_negative = |value: Option<&Value>| value.and_then(Value::as_f64).filter(|v| *v >= 0.0);
    let cpu = non_negative(payload.get("cpu"))
        .filter(|cpu| *cpu <= 100.0)
        .ok_or_else(|| json_error(StatusCode::UNPROCESSABLE_ENTITY, "Invalid cpu"))?;
    let pair = |key: &str, required: bool| -> Result<Value, Response<Body>> {
        let object = payload.get(key);
        let total = non_negative(object.and_then(|v| v.get("total")));
        let used = non_negative(object.and_then(|v| v.get("used")));
        match (total, used) {
            (Some(total), Some(used)) => Ok(json!({ "total": total as u64, "used": used as u64 })),
            _ if !required && object.is_none() => Ok(json!({ "total": 0, "used": 0 })),
            _ => Err(json_error(StatusCode::UNPROCESSABLE_ENTITY, &format!("Invalid {key}"))),
        }
    };
    let mut status = json!({
        "cpu": cpu,
        "mem": pair("mem", true)?,
        "swap": pair("swap", false)?,
        "disk": pair("disk", false)?,
    });
    let net = payload.get("net");
    if let (Some(in_speed), Some(out_speed)) = (
        non_negative(net.and_then(|v| v.get("in_speed"))),
        non_negative(net.and_then(|v| v.get("out_speed"))),
    ) {
        status["net"] = json!({ "in_speed": in_speed, "out_speed": out_speed });
    }
    Ok(status)
}

#[cfg(test)]
mod tests {
    use super::machine_load_status;
    use serde_json::json;

    #[test]
    fn machine_status_requires_cpu_and_memory() {
        assert!(machine_load_status(&json!({ "cpu": 5, "mem": { "total": 10, "used": 2 } })).is_ok());
        assert!(machine_load_status(&json!({ "mem": { "total": 10, "used": 2 } })).is_err());
        assert!(machine_load_status(&json!({ "cpu": 101, "mem": { "total": 10, "used": 2 } })).is_err());
        assert!(machine_load_status(&json!({ "cpu": 5 })).is_err());
        let status = machine_load_status(&json!({
            "cpu": 5, "mem": { "total": 10, "used": 2 }, "net": { "in_speed": 1.5, "out_speed": 2 }
        }))
        .unwrap();
        assert_eq!(status["net"]["in_speed"], json!(1.5));
        assert_eq!(status["swap"]["total"], json!(0));
    }
}
