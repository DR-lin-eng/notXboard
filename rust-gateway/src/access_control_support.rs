use crate::*;

#[derive(Clone, sqlx::FromRow)]
struct AccessUserRow {
    id: i64,
    group_id: Option<i64>,
    trust_level: i64,
}

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct AccessibleNodeRow {
    pub(crate) id: u64,
    pub(crate) user_id: i64,
    pub(crate) name: String,
    pub(crate) protocol: String,
    pub(crate) location_name: Option<String>,
    pub(crate) status: String,
    pub(crate) traffic_limit: u64,
    pub(crate) traffic_used: u64,
    pub(crate) traffic_multiplier: String,
    pub(crate) tcping_enabled: bool,
    pub(crate) tcping_last_status: Option<String>,
    pub(crate) tcping_last_latency_ms: Option<i64>,
    pub(crate) tcping_last_sampled_at: Option<i64>,
    pub(crate) owner_email: Option<String>,
}

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct AccessStatsNodeRow {
    pub(crate) id: u64,
    pub(crate) access_control: Option<SqlxJson<Value>>,
}

pub(crate) async fn load_node_blacklisted_user_ids(
    state: &AppState,
    node_id: u64,
) -> Result<Vec<i64>, sqlx::Error> {
    sqlx::query_scalar::<_, i64>("SELECT user_id FROM user_node_blacklist WHERE node_id = ?")
        .bind(node_id)
        .fetch_all(&state.db)
        .await
}

pub(crate) fn distinct_positive_user_ids<I>(
    values: I,
) -> Vec<i64>
where
    I: IntoIterator<Item = i64>,
{
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();
    for user_id in values {
        if user_id > 0 && seen.insert(user_id) {
            normalized.push(user_id);
        }
    }
    normalized
}

async fn load_access_user_row(
    state: &AppState,
    user_id: i64,
) -> Result<Option<AccessUserRow>, sqlx::Error> {
    sqlx::query_as::<_, AccessUserRow>(
        "SELECT id, group_id, trust_level
         FROM v2_user
         WHERE id = ?
         LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn load_access_stats_node(
    state: &AppState,
    node_id: u64,
) -> Result<Option<AccessStatsNodeRow>, sqlx::Error> {
    sqlx::query_as::<_, AccessStatsNodeRow>(
        "SELECT id, access_control
         FROM server_nodes
         WHERE id = ?
         LIMIT 1",
    )
    .bind(node_id)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn load_accessible_nodes_for_user_payload(
    state: &AppState,
    user_id: i64,
) -> Result<Vec<Value>, sqlx::Error> {
    let user = load_access_user_row(state, user_id).await?;
    let Some(user) = user else {
        return Ok(Vec::new());
    };
    let user_row = UserRow {
        id: user.id,
        token: None,
        group_id: user.group_id,
        subscribe_key: None,
        subscribe_salt: None,
        uuid: None,
        u: None,
        d: None,
        transfer_enable: None,
        expired_at: None,
        trust_level: Some(user.trust_level),
        banned: None,
        is_super_admin: None,
        is_silenced: None,
        subscription_credential_version: None,
    };
    let nodes = load_accessible_nodes_for_user_rows(state, &user_row).await?;
    let push_interval = get_setting_int(state, "server_push_interval", 60).await.max(60);
    let status_cache = state.load_status_cache.read().clone();
    Ok(nodes
        .into_iter()
        .map(|node| {
            let last_report_at = access_node_last_report_at(&status_cache, node.id);
            let is_online = last_report_at
                .map(|value| (Utc::now().timestamp() - value) <= push_interval * 3)
                .unwrap_or(false);
            let online_status = if is_online { "online" } else { "offline" };
            let traffic_usage_percentage = if node.traffic_limit == 0 {
                0.0
            } else {
                ((node.traffic_used as f64 / node.traffic_limit as f64) * 100.0).min(100.0)
            };
            json!({
                "id": node.id,
                "name": node.name,
                "protocol": node.protocol,
                "location_name": node.location_name,
                "status": node.status,
                "online_status": online_status,
                "is_online": is_online,
                "last_report_at": last_report_at,
                "traffic_limit": node.traffic_limit,
                "traffic_used": node.traffic_used,
                "traffic_usage_percentage": traffic_usage_percentage,
                "traffic_multiplier": node.traffic_multiplier.parse::<f64>().unwrap_or(1.0),
                "tcping_enabled": node.tcping_enabled,
                "tcping_status": if node.tcping_enabled { node.tcping_last_status.clone().unwrap_or_else(|| "unknown".to_string()) } else { "unsupported".to_string() },
                "tcping_last_latency_ms": if node.tcping_enabled { node.tcping_last_latency_ms } else { None },
                "tcping_last_sampled_at": if node.tcping_enabled { node.tcping_last_sampled_at } else { None },
                "user_id": node.user_id,
                "owner_email": node.owner_email,
            })
        })
        .collect())
}

pub(crate) async fn load_accessible_nodes_for_user_rows(
    state: &AppState,
    user: &UserRow,
) -> Result<Vec<AccessibleNodeRow>, sqlx::Error> {
    let user_id = user.id;
    let trust_level = user.trust_level.unwrap_or(0);
    let now = Utc::now().timestamp();
    let unlimited_allowance = 8_000_000_000_000_000i64;
    sqlx::query_as::<_, AccessibleNodeRow>(
        "SELECT DISTINCT sn.id, CAST(sn.user_id AS SIGNED) AS user_id, sn.name, sn.protocol, sn.location_name, sn.status,
                sn.traffic_limit, sn.traffic_used, CAST(sn.traffic_multiplier AS CHAR) AS traffic_multiplier,
                sn.tcping_enabled, sn.tcping_last_status, sn.tcping_last_latency_ms, sn.tcping_last_sampled_at,
                owner.email AS owner_email
         FROM server_nodes sn
         JOIN v2_user owner ON owner.id = sn.user_id
         WHERE sn.status = 'active'
           AND owner.banned = 0
           AND (
             sn.user_id = ?
             OR EXISTS (
               SELECT 1 FROM user_node_access ua
               WHERE ua.node_id = sn.id
                 AND ua.user_id = ?
                 AND owner.is_super_admin = 1
             )
             OR EXISTS (
               SELECT 1
               FROM user_plan_subscriptions ups
               JOIN v2_plan plan ON plan.id = ups.plan_id AND plan.scope = 'node'
               WHERE ups.user_id = ?
                 AND plan.owner_user_id = sn.user_id
                 AND JSON_CONTAINS(
                     COALESCE(plan.node_ids, JSON_ARRAY()),
                     CAST(sn.id AS JSON),
                     '$'
                 )
                 AND ups.status = 1
                 AND (ups.expired_at IS NULL OR ups.expired_at > ?)
                 AND (
                   ups.traffic_allowance_kb >= ?
                   OR ups.traffic_allowance_kb > ups.used_traffic_kb
                 )
             )
             OR (
               owner.is_super_admin = 1
               AND JSON_EXTRACT(sn.access_control, '$.min_trust_level') IS NOT NULL
               AND CAST(JSON_UNQUOTE(JSON_EXTRACT(sn.access_control, '$.min_trust_level')) AS SIGNED) <= ?
             )
           )
           AND NOT EXISTS (
             SELECT 1 FROM user_node_blacklist ub
             WHERE ub.node_id = sn.id AND ub.user_id = ?
           )
         ORDER BY sn.id",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(user_id)
    .bind(now)
    .bind(unlimited_allowance)
    .bind(trust_level)
    .bind(user_id)
    .fetch_all(&state.db)
    .await
}

pub(crate) async fn get_accessible_user_ids_for_node(
    state: &AppState,
    node: &ServerNodeRow,
    limit: Option<i64>,
    candidate_user_ids: Option<&[i64]>,
) -> Result<Vec<i64>, sqlx::Error> {
    if limit.is_none() {
        if let Some(values) = cached_accessible_user_ids_for_node(state, node.id) {
            return Ok(filter_accessible_user_ids(values, candidate_user_ids, limit));
        }
        let values = query_accessible_user_ids_for_node(state, node, None, None).await?;
        cache_accessible_user_ids_for_node(state, node.id, values.clone());
        return Ok(filter_accessible_user_ids(values, candidate_user_ids, limit));
    }

    query_accessible_user_ids_for_node(state, node, limit, candidate_user_ids).await
}

async fn query_accessible_user_ids_for_node(
    state: &AppState,
    node: &ServerNodeRow,
    limit: Option<i64>,
    candidate_user_ids: Option<&[i64]>,
) -> Result<Vec<i64>, sqlx::Error> {
    let owner_can_publish_by_trust = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM v2_user WHERE id = ? AND is_super_admin = 1 AND banned = 0",
    )
    .bind(node.user_id)
    .fetch_one(&state.db)
    .await?
        == 1;
    let min_trust_level = owner_can_publish_by_trust
        .then(|| {
            node.access_control
                .as_ref()
                .and_then(|json| json.0.get("min_trust_level"))
                .and_then(Value::as_i64)
        })
        .flatten();
    let now = Utc::now().timestamp();
    let unlimited_allowance = 8_000_000_000_000_000i64;

    let mut candidate_sql = String::from("SELECT ? AS user_id");
    if owner_can_publish_by_trust {
        candidate_sql.push_str(
            " UNION
              SELECT user_id
              FROM user_node_access
              WHERE node_id = ?",
        );
    }
    candidate_sql.push_str(
        " UNION
         SELECT ups.user_id
         FROM user_plan_subscriptions ups
         JOIN v2_plan plan ON plan.id = ups.plan_id AND plan.scope = 'node'
         WHERE plan.owner_user_id = ?
           AND JSON_CONTAINS(
                   COALESCE(plan.node_ids, JSON_ARRAY()),
                   CAST(? AS JSON),
                   '$'
               )
           AND ups.status = 1
           AND (ups.expired_at IS NULL OR ups.expired_at > ?)
           AND (
             ups.traffic_allowance_kb >= ?
             OR ups.traffic_allowance_kb > ups.used_traffic_kb
           )",
    );

    if min_trust_level.is_some() {
        candidate_sql.push_str(
            " UNION
              SELECT id AS user_id
              FROM v2_user
              WHERE trust_level >= ?",
        );
    }

    let mut sql = String::from(
        "SELECT u.id
         FROM (",
    );
    sql.push_str(&candidate_sql);
    sql.push_str(
        ") candidate_users
         JOIN v2_user u ON u.id = candidate_users.user_id
         WHERE u.banned = 0
           AND (
             u.id = ?
             OR (u.expired_at IS NULL OR u.expired_at >= ?)
           )
           AND (
             u.id = ?
             OR NOT EXISTS (
               SELECT 1 FROM user_node_blacklist ub
               WHERE ub.node_id = ? AND ub.user_id = u.id
             )
           )",
    );

    if let Some(candidates) = candidate_user_ids {
        if !candidates.is_empty() {
            sql.push_str(" AND u.id IN (");
            sql.push_str(&vec!["?"; candidates.len()].join(","));
            sql.push(')');
        }
    }

    sql.push_str(" ORDER BY u.id");
    if limit.unwrap_or(0) > 0 {
        sql.push_str(" LIMIT ?");
    }

    let mut query = sqlx::query_scalar::<_, i64>(&sql).bind(node.user_id);
    if owner_can_publish_by_trust {
        query = query.bind(node.id);
    }
    query = query
        .bind(node.user_id)
        .bind(node.id)
        .bind(now)
        .bind(unlimited_allowance);

    if let Some(min) = min_trust_level {
        query = query.bind(min);
    }

    query = query
        .bind(node.user_id)
        .bind(now)
        .bind(node.user_id)
        .bind(node.id);

    if let Some(candidates) = candidate_user_ids {
        for candidate in candidates {
            query = query.bind(*candidate);
        }
    }

    if let Some(limit) = limit {
        if limit > 0 {
            query = query.bind(limit);
        }
    }

    query.fetch_all(&state.db).await
}

fn filter_accessible_user_ids(
    values: Vec<i64>,
    candidate_user_ids: Option<&[i64]>,
    limit: Option<i64>,
) -> Vec<i64> {
    let mut filtered = if let Some(candidates) = candidate_user_ids.filter(|values| !values.is_empty()) {
        let candidate_set = candidates.iter().copied().collect::<HashSet<_>>();
        values
            .into_iter()
            .filter(|user_id| candidate_set.contains(user_id))
            .collect::<Vec<_>>()
    } else {
        values
    };

    if let Some(limit) = limit.filter(|value| *value > 0) {
        filtered.truncate(limit as usize);
    }

    filtered
}

pub(crate) async fn load_authorized_user_ids_for_node(
    state: &AppState,
    node_id: u64,
) -> Result<Vec<i64>, sqlx::Error> {
    let rows = sqlx::query("SELECT user_id FROM user_node_access WHERE node_id = ? ORDER BY user_id")
        .bind(node_id)
        .fetch_all(&state.db)
        .await?;
    Ok(rows
        .into_iter()
        .filter_map(|row| row.try_get::<i64, _>("user_id").ok())
        .collect())
}

pub(crate) async fn get_accessible_user_ids_for_owner_node(
    state: &AppState,
    node_id: u64,
) -> Result<Vec<i64>, sqlx::Error> {
    let node = sqlx::query_as::<_, ServerNodeRow>(
        "SELECT id, user_id, name, host, port, service_port, protocol, settings, access_control, device_limit, connection_limit, speed_limit_down, created_at
         FROM server_nodes
         WHERE id = ?
         LIMIT 1",
    )
    .bind(node_id)
    .fetch_one(&state.db)
    .await?;
    get_accessible_user_ids_for_node(state, &node, None, None).await
}

pub(crate) fn access_node_last_report_at(
    status_cache: &HashMap<u64, Value>,
    node_id: u64,
) -> Option<i64> {
    status_cache
        .get(&node_id)
        .and_then(|value| value.get("updated_at"))
        .and_then(|value| value.as_i64())
}

pub(crate) async fn user_can_access_tcping_node(
    state: &AppState,
    user: &BearerUserRow,
    node: &TcpingNodeOverviewRow,
) -> Result<bool, sqlx::Error> {
    if user.is_super_admin == 1 || node.user_id == user.id {
        return Ok(true);
    }
    let user_row = UserRow {
        id: user.id,
        token: None,
        group_id: user.group_id,
        subscribe_key: None,
        subscribe_salt: None,
        uuid: None,
        u: None,
        d: None,
        transfer_enable: None,
        expired_at: user.expired_at,
        trust_level: Some(user.trust_level),
        banned: Some(user.banned),
        is_super_admin: Some(user.is_super_admin),
        is_silenced: Some(user.is_silenced),
        subscription_credential_version: None,
    };
    let accessible_nodes = load_accessible_nodes_for_user_rows(state, &user_row).await?;
    Ok(accessible_nodes.iter().any(|candidate| candidate.id == node.id))
}

pub(crate) fn clear_accessible_user_ids_cache(state: &AppState, node_id: u64) {
    state.node_accessible_user_ids_cache.write().remove(&node_id);
    state.uniproxy_user_snapshot_cache.write().clear();
    state.response_cache.write().clear();
}

pub(crate) fn clear_all_authorization_caches(state: &AppState) {
    state.node_accessible_user_ids_cache.write().clear();
    state.uniproxy_user_snapshot_cache.write().clear();
    state.response_cache.write().clear();
}

fn cached_accessible_user_ids_for_node(state: &AppState, node_id: u64) -> Option<Vec<i64>> {
    let now = Instant::now();
    {
        let cache = state.node_accessible_user_ids_cache.read();
        let cached = cache.get(&node_id)?;
        if cached.expires_at > now {
            return Some(cached.values.clone());
        }
    }
    state.node_accessible_user_ids_cache.write().remove(&node_id);
    None
}

fn cache_accessible_user_ids_for_node(state: &AppState, node_id: u64, values: Vec<i64>) {
    let now = Instant::now();
    let ttl = node_access_cache_ttl();
    let mut cache = state.node_accessible_user_ids_cache.write();
    if cache.len() >= 256 {
        cache.retain(|_, entry| entry.expires_at > now);
        if cache.len() >= 512 {
            cache.clear();
        }
    }
    cache.insert(
        node_id,
        CachedIdList {
            values,
            expires_at: now + ttl,
        },
    );
}

fn node_access_cache_ttl() -> Duration {
    let seconds = std::env::var("NODE_ACCESS_CACHE_TTL_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(|value| value.clamp(1, 30))
        .unwrap_or(3);
    Duration::from_secs(seconds)
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_node_plan_authorization_path_binds_plan_owner_to_node_owner() {
        let access = include_str!("access_control_support.rs");
        assert!(access.contains("CAST(sn.user_id AS SIGNED) AS user_id"));
        assert!(access.contains("plan.owner_user_id = sn.user_id"));
        assert!(access.contains("WHERE plan.owner_user_id = ?"));
        assert!(include_str!("main.rs").contains("plan.owner_user_id = sn.user_id"));
        assert!(include_str!("tickets_support.rs").contains("AND plan.owner_user_id = ?"));
        assert!(
            include_str!("traffic_ingest_support.rs")
                .contains("node.user_id = plan.owner_user_id")
        );
    }
}
