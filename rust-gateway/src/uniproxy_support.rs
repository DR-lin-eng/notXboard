use crate::*;
use crate::db_retry_support::retry_db_write;
use crate::uniproxy_user_support::build_uniproxy_user_response;
use sqlx::QueryBuilder;

#[derive(Clone)]
struct CachedNodeAuth {
    node: ServerNodeRow,
    expires_at: Instant,
}

#[derive(Clone)]
pub(crate) struct QueuedPushTrafficJob {
    pub(crate) node_id: u64,
    pub(crate) rows: Vec<QueuedPushTrafficRow>,
}

#[derive(Clone)]
pub(crate) struct QueuedPushTrafficRow {
    pub(crate) user_id: i64,
    pub(crate) upload: i64,
    pub(crate) download: i64,
}

#[derive(Clone)]
pub(crate) struct QueuedAliveSessionJob {
    pub(crate) node_id: u64,
    pub(crate) users: Vec<QueuedAliveSessionUser>,
}

#[derive(Clone)]
pub(crate) struct QueuedAliveSessionUser {
    pub(crate) user_id: i64,
    pub(crate) ips: Vec<String>,
}

pub(crate) async fn load_uniproxy_alive_counts(
    state: &AppState,
    node: &ServerNodeRow,
) -> Result<Map<String, Value>, sqlx::Error> {
    let min_trust_level = node
        .access_control
        .as_ref()
        .and_then(|json| json.0.get("min_trust_level"))
        .and_then(|value| value.as_i64());
    let now = Utc::now().timestamp();
    let unlimited_allowance = 8_000_000_000_000_000i64;

    let mut candidate_sql = String::from(
        "SELECT ? AS user_id
         UNION
         SELECT id AS user_id
         FROM v2_user
         WHERE is_super_admin = 1
         UNION
         SELECT user_id
         FROM user_node_access
         WHERE node_id = ?
         UNION
         SELECT upa.user_id
         FROM user_node_plan_access upa
         JOIN user_plan_subscriptions ups
           ON ups.user_id = upa.user_id
          AND ups.plan_id = upa.plan_id
         WHERE upa.node_id = ?
           AND ups.status = 1
           AND (ups.expired_at IS NULL OR ups.expired_at > ?)
           AND (
             ups.traffic_allowance_kb >= ?
             OR ups.traffic_allowance_kb > ups.used_traffic_kb
           )"
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
        "SELECT
            accessible.id AS user_id,
            COUNT(DISTINCT CASE WHEN INSTR(s.ip_address, ':') = 0 THEN s.ip_address END) AS ipv4_count,
            COUNT(DISTINCT CASE WHEN INSTR(s.ip_address, ':') > 0 THEN s.ip_address END) AS ipv6_count
         FROM (
            SELECT
                u.id,
                CASE
                    WHEN ? > 0 AND (
                        COALESCE(NULLIF(uil.device_limit, 0), NULLIF(ugl.device_limit, 0), 2) <= 0
                        OR COALESCE(NULLIF(uil.device_limit, 0), NULLIF(ugl.device_limit, 0), 2) > ?
                    ) THEN ?
                    ELSE COALESCE(NULLIF(uil.device_limit, 0), NULLIF(ugl.device_limit, 0), 2)
                END AS effective_device_limit
            FROM ("
    );
    sql.push_str(&candidate_sql);
    sql.push_str(
        ") candidate_users
            JOIN v2_user u ON u.id = candidate_users.user_id
            LEFT JOIN user_individual_limits uil ON uil.user_id = u.id
            LEFT JOIN user_group_limits ugl ON ugl.trust_level = COALESCE(u.trust_level, 0)
            WHERE u.banned = 0
              AND (
                u.id = ?
                OR COALESCE(u.is_super_admin, 0) = 1
                OR (u.expired_at IS NULL OR u.expired_at >= ?)
              )
              AND (
                u.id = ?
                OR COALESCE(u.is_super_admin, 0) = 1
                OR NOT EXISTS (
                  SELECT 1 FROM user_node_blacklist ub
                  WHERE ub.node_id = ? AND ub.user_id = u.id
                )
              )
         ) accessible
         LEFT JOIN user_online_sessions s
           ON s.user_id = accessible.id
          AND s.last_activity > (NOW() - INTERVAL 5 MINUTE)
         WHERE accessible.effective_device_limit > 0
         GROUP BY accessible.id",
    );

    let mut query = sqlx::query(&sql)
        .bind(node.device_limit)
        .bind(node.device_limit)
        .bind(node.device_limit)
        .bind(node.user_id)
        .bind(node.id)
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

    let rows = query.fetch_all(&state.db).await?;
    let mut result = Map::new();
    for row in rows {
        let user_id: i64 = row.get("user_id");
        let ipv4_count = row
            .try_get::<Option<i64>, _>("ipv4_count")
            .ok()
            .flatten()
            .unwrap_or(0);
        let ipv6_count = row
            .try_get::<Option<i64>, _>("ipv6_count")
            .ok()
            .flatten()
            .unwrap_or(0);
        let alive = ipv4_count.max(ipv6_count);
        if alive > 0 {
            result.insert(user_id.to_string(), Value::from(alive));
        }
    }
    Ok(result)
}

pub(crate) fn apply_node_limit_overrides(
    node: &ServerNodeRow,
    speed_limit_down: i64,
    device_limit: i64,
    connection_limit: i64,
) -> EffectiveLimits {
    let speed_limit_down =
        if node.speed_limit_down > 0 && (speed_limit_down <= 0 || speed_limit_down > node.speed_limit_down) {
            node.speed_limit_down
        } else {
            speed_limit_down
        };
    let device_limit =
        if node.device_limit > 0 && (device_limit <= 0 || device_limit > node.device_limit) {
            node.device_limit
        } else {
            device_limit
        };
    let connection_limit =
        if node.connection_limit > 0 && (connection_limit <= 0 || connection_limit > node.connection_limit) {
            node.connection_limit
        } else {
            connection_limit
        };

    EffectiveLimits {
        speed_limit_down,
        device_limit,
        connection_limit,
    }
}

pub(crate) fn spawn_push_traffic_worker(
    state: AppState,
    rx: tokio::sync::mpsc::Receiver<QueuedPushTrafficJob>,
) {
    tokio::spawn(async move {
        run_push_traffic_worker(state, rx).await;
    });
}

pub(crate) fn spawn_alive_session_worker(
    state: AppState,
    rx: tokio::sync::mpsc::Receiver<QueuedAliveSessionJob>,
) {
    tokio::spawn(async move {
        run_alive_session_worker(state, rx).await;
    });
}

async fn run_alive_session_worker(
    state: AppState,
    mut rx: tokio::sync::mpsc::Receiver<QueuedAliveSessionJob>,
) {
    let flush_interval = alive_session_flush_interval();
    let max_batch_jobs = alive_session_max_batch_jobs();
    let max_batch_users = alive_session_max_batch_users();

    loop {
        let Some(first_job) = rx.recv().await else {
            break;
        };

        let mut jobs = vec![first_job];
        let mut user_count = jobs[0].users.len();
        let deadline = tokio::time::Instant::now() + flush_interval;
        let mut channel_closed = false;

        while jobs.len() < max_batch_jobs && user_count < max_batch_users {
            let sleep = tokio::time::sleep_until(deadline);
            tokio::pin!(sleep);

            tokio::select! {
                maybe_job = rx.recv() => {
                    match maybe_job {
                        Some(job) => {
                            user_count += job.users.len();
                            jobs.push(job);
                        }
                        None => {
                            channel_closed = true;
                            break;
                        }
                    }
                }
                _ = &mut sleep => break,
            }
        }

        if let Err(err) = flush_alive_session_jobs(&state, &jobs).await {
            state.async_queue_metrics.record_alive_flush_failure();
            error!(
                job_count = jobs.len(),
                user_count = user_count,
                "alive session flush failed: {}",
                err
            );
        }

        if channel_closed {
            break;
        }
    }
}

async fn run_push_traffic_worker(
    state: AppState,
    mut rx: tokio::sync::mpsc::Receiver<QueuedPushTrafficJob>,
) {
    let flush_interval = push_traffic_flush_interval();
    let max_batch_jobs = push_traffic_max_batch_jobs();
    let max_batch_rows = push_traffic_max_batch_rows();

    loop {
        let Some(first_job) = rx.recv().await else {
            break;
        };

        let mut jobs = vec![first_job];
        let mut row_count = jobs[0].rows.len();
        let deadline = tokio::time::Instant::now() + flush_interval;
        let mut channel_closed = false;

        while jobs.len() < max_batch_jobs && row_count < max_batch_rows {
            let sleep = tokio::time::sleep_until(deadline);
            tokio::pin!(sleep);

            tokio::select! {
                maybe_job = rx.recv() => {
                    match maybe_job {
                        Some(job) => {
                            row_count += job.rows.len();
                            jobs.push(job);
                        }
                        None => {
                            channel_closed = true;
                            break;
                        }
                    }
                }
                _ = &mut sleep => break,
            }
        }

        if let Err(err) = flush_push_traffic_jobs(&state, &jobs).await {
            state.async_queue_metrics.record_push_flush_failure();
            error!(
                job_count = jobs.len(),
                row_count = row_count,
                "push traffic flush failed: {}",
                err
            );
        }

        if channel_closed {
            break;
        }
    }
}

async fn flush_alive_session_jobs(
    state: &AppState,
    jobs: &[QueuedAliveSessionJob],
) -> Result<(), sqlx::Error> {
    let total_users = jobs.iter().map(|job| job.users.len() as u64).sum::<u64>();
    let mut per_node = std::collections::BTreeMap::<u64, std::collections::BTreeMap<i64, HashSet<String>>>::new();
    for job in jobs {
        let node_users = per_node.entry(job.node_id).or_default();
        for user in &job.users {
            node_users.insert(
                user.user_id,
                user.ips.iter().cloned().collect::<HashSet<_>>(),
            );
        }
    }

    for (node_id, reported) in per_node {
        let reported = reported.into_iter().collect::<HashMap<_, _>>();
        write_merged_alive_sessions(state, node_id, &reported).await?;
    }

    state
        .async_queue_metrics
        .record_alive_flush(jobs.len() as u64, total_users);
    Ok(())
}

async fn flush_push_traffic_jobs(
    state: &AppState,
    jobs: &[QueuedPushTrafficJob],
) -> Result<(), sqlx::Error> {
    let total_rows = jobs.iter().map(|job| job.rows.len() as u64).sum::<u64>();
    let mut per_node = std::collections::BTreeMap::<u64, std::collections::BTreeMap<i64, (i64, i64)>>::new();
    for job in jobs {
        let node_rows = per_node.entry(job.node_id).or_default();
        for row in &job.rows {
            let entry = node_rows.entry(row.user_id).or_insert((0, 0));
            entry.0 += row.upload.max(0);
            entry.1 += row.download.max(0);
        }
    }

    for (node_id, merged) in per_node {
        let merged_rows = merged
            .into_iter()
            .map(|(user_id, (upload, download))| {
                let billed = ((upload + download) as f64).ceil() as i64;
                (user_id, upload, download, billed.max(0))
            })
            .collect::<Vec<AggregatedTrafficRow>>();
        write_merged_push_traffic(state, node_id, &merged_rows).await?;
    }

    state
        .async_queue_metrics
        .record_push_flush(jobs.len() as u64, total_rows);
    Ok(())
}

fn push_traffic_flush_interval() -> Duration {
    let millis = std::env::var("PUSH_TRAFFIC_FLUSH_INTERVAL_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(|value| value.clamp(10, 1000))
        .unwrap_or(50);
    Duration::from_millis(millis)
}

fn alive_session_flush_interval() -> Duration {
    let millis = std::env::var("ALIVE_SESSION_FLUSH_INTERVAL_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(|value| value.clamp(10, 1000))
        .unwrap_or(50);
    Duration::from_millis(millis)
}

fn push_traffic_max_batch_jobs() -> usize {
    std::env::var("PUSH_TRAFFIC_MAX_BATCH_JOBS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .map(|value| value.clamp(1, 10_000))
        .unwrap_or(512)
}

fn alive_session_max_batch_jobs() -> usize {
    std::env::var("ALIVE_SESSION_MAX_BATCH_JOBS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .map(|value| value.clamp(1, 10_000))
        .unwrap_or(512)
}

fn push_traffic_max_batch_rows() -> usize {
    std::env::var("PUSH_TRAFFIC_MAX_BATCH_ROWS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .map(|value| value.clamp(100, 200_000))
        .unwrap_or(20_000)
}

fn alive_session_max_batch_users() -> usize {
    std::env::var("ALIVE_SESSION_MAX_BATCH_USERS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .map(|value| value.clamp(100, 200_000))
        .unwrap_or(20_000)
}

pub(crate) async fn uniproxy_config(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_uniproxy_config_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub(crate) async fn uniproxy_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_uniproxy_user_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub(crate) async fn uniproxy_alivelist(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_uniproxy_alivelist_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub(crate) async fn uniproxy_push(
    State(state): State<Arc<AppState>>,
    _headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_uniproxy_push_response(&state, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub(crate) async fn uniproxy_alive(
    State(state): State<Arc<AppState>>,
    _headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_uniproxy_alive_response(&state, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub(crate) async fn uniproxy_status(
    State(state): State<Arc<AppState>>,
    _headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_uniproxy_status_response(&state, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub(crate) async fn uniproxy_audit(
    State(state): State<Arc<AppState>>,
    _headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_uniproxy_audit_response(&state, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub(crate) fn build_uniproxy_read_cache_key(kind: &str, uri: &Uri) -> String {
    let params = parse_query(uri);
    let token = params.get("token").map(|value| value.trim()).unwrap_or_default();
    let node_id = params.get("node_id").map(|value| value.trim()).unwrap_or_default();

    if token.is_empty() || node_id.is_empty() {
        return format!("uniproxy:{kind}:{}", build_cache_key(uri));
    }

    format!("uniproxy:{kind}:{token}:{node_id}")
}

pub(crate) fn clamp_snapshot_ttl(value: i64, default_secs: i64, min_secs: i64, max_secs: i64) -> Duration {
    let secs = if value <= 0 { default_secs } else { value }.clamp(min_secs, max_secs) as u64;
    Duration::from_secs(secs)
}

async fn uniproxy_config_ttl(state: &AppState) -> Duration {
    clamp_snapshot_ttl(
        get_setting_int(state, "server_node_config_snapshot_ttl", 5).await,
        5,
        1,
        120,
    )
}

async fn uniproxy_alive_ttl(state: &AppState) -> Duration {
    clamp_snapshot_ttl(
        get_setting_int(state, "server_node_alive_snapshot_ttl", 5).await,
        5,
        1,
        120,
    )
}

async fn build_uniproxy_config_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let cache_key = build_uniproxy_read_cache_key("config", &uri);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }
    let node = authenticate_node(state, &uri).await?;

    let push_interval = get_setting_int(state, "server_push_interval", 60).await;
    let pull_interval = get_setting_int(state, "server_pull_interval", 60).await;

    let mut payload = build_server_node_panel_config(state, &node)?;
    if let Some(object) = payload.as_object_mut() {
        object.insert(
            "base_config".to_string(),
            json!({
                "push_interval": push_interval,
                "pull_interval": pull_interval,
            }),
        );
    }

    Ok(json_cached_response(
        state,
        cache_key,
        payload,
        uniproxy_config_ttl(state).await,
    ))
}

async fn build_uniproxy_alivelist_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let cache_key = build_uniproxy_read_cache_key("alivelist", &uri);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }
    let node = authenticate_node(state, &uri).await?;

    let alive = load_uniproxy_alive_counts(state, &node)
        .await
        .map_err(internal_error)?;
    Ok(json_cached_response(
        state,
        cache_key,
        json!({ "alive": alive }),
        uniproxy_alive_ttl(state).await,
    ))
}

async fn build_uniproxy_push_response(
    state: &AppState,
    uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let node = authenticate_node(state, &uri).await?;
    let payload = parse_json_body(body).await?;
    let rows = normalize_push_payload(&payload)
        .ok_or_else(|| json_error(StatusCode::UNPROCESSABLE_ENTITY, "Invalid data format"))?;
    if rows.is_empty() {
        return Ok(json_value_response(json!({ "data": true })));
    }

    let candidate_user_ids = rows.iter().map(|row| row.user_id).collect::<Vec<_>>();
    let allowed_user_ids = get_accessible_user_ids_for_node(state, &node, None, Some(&candidate_user_ids))
        .await
        .map_err(internal_error)?;
    let allowed_set = allowed_user_ids.into_iter().collect::<HashSet<_>>();
    let filtered_rows = rows
        .into_iter()
        .filter(|row| allowed_set.contains(&row.user_id))
        .collect::<Vec<_>>();

    if filtered_rows.is_empty() {
        return Ok(json_value_response(json!({ "data": true })));
    }

    enqueue_push_traffic(state, node.id, &filtered_rows)
        .await
        .map_err(internal_error)?;
    Ok(json_value_response(json!({ "data": true })))
}

async fn build_uniproxy_alive_response(
    state: &AppState,
    uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let node = authenticate_node(state, &uri).await?;
    let payload = parse_json_body(body).await?;
    let map = payload
        .as_object()
        .cloned()
        .ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "Invalid online data"))?;

    let candidate_user_ids = map
        .keys()
        .filter_map(|key| key.parse::<i64>().ok())
        .filter(|id| *id > 0)
        .collect::<Vec<_>>();
    let allowed_user_ids = get_accessible_user_ids_for_node(state, &node, None, Some(&candidate_user_ids))
        .await
        .map_err(internal_error)?;
    let allowed_set = allowed_user_ids.into_iter().collect::<HashSet<_>>();

    enqueue_alive_sessions(state, node.id, &map, &allowed_set)
        .await
        .map_err(internal_error)?;
    Ok(json_value_response(json!({ "data": true })))
}

async fn build_uniproxy_status_response(
    state: &AppState,
    uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let node = authenticate_node(state, &uri).await?;
    let payload = parse_json_body(body).await?;
    let object = payload
        .as_object()
        .cloned()
        .ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "Invalid status data"))?;

    let cpu = object
        .get("cpu")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "Invalid cpu"))?;
    let mem_total = nested_i64(&object, &["mem", "total"])
        .ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "Invalid mem.total"))?;
    let mem_used = nested_i64(&object, &["mem", "used"])
        .ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "Invalid mem.used"))?;
    let swap_total = nested_i64(&object, &["swap", "total"])
        .ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "Invalid swap.total"))?;
    let swap_used = nested_i64(&object, &["swap", "used"])
        .ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "Invalid swap.used"))?;
    let disk_total = nested_i64(&object, &["disk", "total"])
        .ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "Invalid disk.total"))?;
    let disk_used = nested_i64(&object, &["disk", "used"])
        .ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "Invalid disk.used"))?;

    state.load_status_cache.write().insert(
        node.id,
        json!({
            "cpu": cpu,
            "mem": { "total": mem_total, "used": mem_used },
            "swap": { "total": swap_total, "used": swap_used },
            "disk": { "total": disk_total, "used": disk_used },
            "updated_at": Utc::now().timestamp(),
        }),
    );

    Ok(json_value_response(json!({
        "data": true,
        "code": 0,
        "message": "success"
    })))
}

async fn build_uniproxy_audit_response(
    state: &AppState,
    uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let node = authenticate_node(state, &uri).await?;
    let payload = parse_json_body(body).await?;
    let object = payload
        .as_object()
        .cloned()
        .ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "Invalid audit data"))?;

    let user_id = object
        .get("user_id")
        .and_then(|v| v.as_i64())
        .filter(|v| *v > 0)
        .ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "Invalid user_id"))?;
    let ip_address = object
        .get("ip_address")
        .and_then(|v| v.as_str())
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| json_error(StatusCode::BAD_REQUEST, "Invalid ip_address"))?;
    let target_domain = object
        .get("target_domain")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());
    let target_protocol = object
        .get("target_protocol")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());
    let action_taken = object
        .get("action_taken")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());

    let audit_result = if let Some(action) = action_taken {
        let log_id = insert_audit_log(
            state,
            user_id,
            node.id,
            None,
            ip_address,
            target_domain.as_deref(),
            target_protocol.as_deref(),
            &action,
        )
        .await
        .map_err(internal_error)?;
        Some(log_id)
    } else {
        evaluate_and_insert_audit_log(
            state,
            node.id,
            user_id,
            ip_address,
            target_domain.as_deref(),
            target_protocol.as_deref(),
        )
        .await
        .map_err(internal_error)?
    };

    Ok(json_value_response(json!({
        "success": true,
        "data": {
            "logged": audit_result.is_some(),
            "log_id": audit_result,
        }
    })))
}

pub(crate) async fn authenticate_node(
    state: &AppState,
    uri: &Uri,
) -> Result<ServerNodeRow, Response<Body>> {
    let params = parse_query(uri);
    let token = params.get("token").cloned().unwrap_or_default();
    let node_id = params.get("node_id").cloned().unwrap_or_default();

    if token.trim().is_empty() || node_id.trim().is_empty() {
        return Err(json_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Missing token or node_id",
        ));
    }

    let node_id_value = node_id
        .parse::<i64>()
        .map_err(|_| json_error(StatusCode::UNAUTHORIZED, "Invalid server token"))?;

    let cache_key = format!("{}:{}", token, node_id_value);
    if let Some(node) = cached_authenticated_node(&cache_key) {
        return Ok(node);
    }

    let node = sqlx::query_as::<_, ServerNodeRow>(
        "SELECT id, user_id, name, host, port, service_port, protocol, settings, access_control, device_limit, connection_limit, speed_limit_down, created_at
         FROM (
             SELECT
                 id, user_id, name, host, port, service_port, protocol, settings, access_control, device_limit, connection_limit, speed_limit_down, created_at
             FROM server_nodes
             WHERE v2bx_token = ? AND id = ?
             UNION ALL
             SELECT
                 id, user_id, name, host, port, service_port, protocol, settings, access_control, device_limit, connection_limit, speed_limit_down, created_at
             FROM server_nodes
             WHERE v2bx_token = ? AND v2bx_node_id = ? AND id <> ?
         ) matched_nodes
         LIMIT 1",
    )
    .bind(&token)
    .bind(node_id_value)
    .bind(&token)
    .bind(node_id_value)
    .bind(node_id_value)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?;

    let node =
        node.ok_or_else(|| json_error(StatusCode::UNAUTHORIZED, "Invalid server token"))?;
    cache_authenticated_node(cache_key, node.clone());
    Ok(node)
}

fn node_auth_cache() -> &'static parking_lot::RwLock<HashMap<String, CachedNodeAuth>> {
    static NODE_AUTH_CACHE: OnceLock<parking_lot::RwLock<HashMap<String, CachedNodeAuth>>> =
        OnceLock::new();
    NODE_AUTH_CACHE.get_or_init(|| parking_lot::RwLock::new(HashMap::new()))
}

fn cached_authenticated_node(cache_key: &str) -> Option<ServerNodeRow> {
    {
        let cache = node_auth_cache().read();
        let cached = cache.get(cache_key)?;
        if cached.expires_at > Instant::now() {
            return Some(cached.node.clone());
        }
    }
    node_auth_cache().write().remove(cache_key);
    None
}

fn cache_authenticated_node(cache_key: String, node: ServerNodeRow) {
    let mut cache = node_auth_cache().write();
    if cache.len() >= 4096 {
        let now = Instant::now();
        cache.retain(|_, value| value.expires_at > now);
        if cache.len() >= 8192 {
            cache.clear();
        }
    }
    cache.insert(
        cache_key,
        CachedNodeAuth {
            node,
            expires_at: Instant::now() + Duration::from_secs(3),
        },
    );
}

fn build_server_node_panel_config(
    state: &AppState,
    node: &ServerNodeRow,
) -> Result<Value, Response<Body>> {
    let protocol = normalize_type(&node.protocol).unwrap_or_else(|| "vmess".to_string());
    let settings = normalized_protocol_settings(
        &protocol,
        node.settings
            .as_ref()
            .and_then(|json| json.0.as_object().cloned())
            .unwrap_or_default(),
    );
    let host = node.host.clone();
    let service_port = effective_service_port(node);
    let server_name = resolve_panel_server_name(&settings, &host);

    let base = json!({
        "host": host,
        "server_port": service_port,
        "server_name": server_name,
    });

    let base_map = base.as_object().cloned().unwrap_or_default();
    let payload = match protocol.as_str() {
        "vmess" => merge_json_object(
            base_map,
            json!({
                "tls": json_int(&settings, "tls"),
                "network": json_string(&settings, "network", "tcp"),
                "network_settings": normalize_object_value(settings.get("network_settings").or_else(|| settings.get("networkSettings")), true),
                "tls_settings": normalize_object_value(settings.get("tls_settings").or_else(|| settings.get("tlsSettings")), true),
            }),
        ),
        "vless" => {
            let tls = json_int(&settings, "tls");
            let tls_settings = if tls == 2 {
                normalize_object_value(settings.get("reality_settings"), true)
            } else {
                normalize_object_value(
                    settings
                        .get("tls_settings")
                        .or_else(|| settings.get("tlsSettings")),
                    true,
                )
            };
            merge_json_object(
                base_map,
                json!({
                    "tls": tls,
                    "network": json_string(&settings, "network", "tcp"),
                    "network_settings": normalize_object_value(settings.get("network_settings").or_else(|| settings.get("networkSettings")), true),
                    "flow": settings.get("flow").cloned().unwrap_or(Value::Null),
                    "tls_settings": tls_settings,
                }),
            )
        }
        "trojan" => merge_json_object(
            base_map,
            json!({
                "network": json_string(&settings, "network", "tcp"),
                "networkSettings": normalize_object_value(settings.get("network_settings").or_else(|| settings.get("networkSettings")), true),
            }),
        ),
        "shadowsocks" => {
            let cipher = json_string(&settings, "cipher", "aes-128-gcm");
            let server_key = match cipher.as_str() {
                "2022-blake3-aes-128-gcm" => Some(get_server_key(&state.app_key, node.created_at, 16)),
                "2022-blake3-aes-256-gcm" => Some(get_server_key(&state.app_key, node.created_at, 32)),
                _ => None,
            };
            merge_json_object(
                base_map,
                json!({
                    "cipher": cipher,
                    "server_key": server_key,
                }),
            )
        }
        "hysteria" | "hysteria2" => {
            let mut version = json_int(&settings, "version");
            if protocol == "hysteria2" {
                version = 2;
            }
            let base_hy = merge_json_object(
                base_map.clone(),
                json!({
                    "version": version,
                    "up_mbps": json_nested_int(&settings, &["bandwidth", "up"]),
                    "down_mbps": json_nested_int(&settings, &["bandwidth", "down"]),
                }),
            );
            if version >= 2 {
                let obfs_enabled = json_nested_bool(&settings, &["obfs", "open"]);
                merge_json_object(
                    base_hy,
                    json!({
                        "obfs": if obfs_enabled { json_nested_string(&settings, &["obfs", "type"], "salamander") } else { String::new() },
                        "obfs-password": if obfs_enabled { json_nested_string(&settings, &["obfs", "password"], "") } else { String::new() },
                        "ignore_client_bandwidth": json_bool(&settings, "ignore_client_bandwidth"),
                    }),
                )
            } else {
                merge_json_object(
                    base_hy,
                    json!({
                        "obfs": json_nested_string(&settings, &["obfs", "password"], ""),
                    }),
                )
            }
        }
        "tuic" => merge_json_object(
            base_map,
            json!({
                "congestion_control": json_string(&settings, "congestion_control", "cubic"),
                "zero_rtt_handshake": json_bool(&settings, "zero_rtt_handshake"),
                "heartbeat": json_string(&settings, "heartbeat", "10s"),
            }),
        ),
        "anytls" => merge_json_object(
            base_map,
            json!({
                "padding_scheme": settings.get("padding_scheme").cloned().unwrap_or_else(|| json!([])),
            }),
        ),
        _ => Value::Object(base_map),
    };

    Ok(payload)
}

fn effective_service_port(node: &ServerNodeRow) -> i64 {
    node.service_port.unwrap_or(node.port)
}

fn normalize_push_payload(value: &Value) -> Option<Vec<TrafficRow>> {
    if let Some(list) = value.as_array() {
        let mut rows = Vec::new();
        for item in list {
            let item = item.as_array()?;
            if item.len() < 2 {
                continue;
            }
            let user_id = item.first()?.as_i64()?;
            let traffic = item.get(1)?.as_i64()?;
            if user_id <= 0 || traffic <= 0 {
                continue;
            }
            rows.push(TrafficRow {
                user_id,
                upload: 0,
                download: traffic,
            });
        }
        return Some(rows);
    }

    let object = value.as_object()?;
    let mut rows = Vec::new();
    for (user_id, row) in object {
        let user_id = user_id.parse::<i64>().ok()?;
        let row = row.as_array()?;
        if row.len() < 2 {
            continue;
        }
        let upload = row.first().and_then(|v| v.as_i64()).unwrap_or(0);
        let download = row.get(1).and_then(|v| v.as_i64()).unwrap_or(0);
        if user_id <= 0 || upload + download <= 0 {
            continue;
        }
        rows.push(TrafficRow {
            user_id,
            upload: upload.max(0),
            download: download.max(0),
        });
    }
    Some(rows)
}

async fn enqueue_push_traffic(
    state: &AppState,
    node_id: u64,
    rows: &[TrafficRow],
) -> Result<(), sqlx::Error> {
    if rows.is_empty() {
        return Ok(());
    }

    let job = QueuedPushTrafficJob {
        node_id,
        rows: rows
            .iter()
            .map(|row| QueuedPushTrafficRow {
                user_id: row.user_id,
                upload: row.upload.max(0),
                download: row.download.max(0),
            })
            .collect(),
    };

    match state.push_traffic_tx.send(job).await {
        Ok(()) => {
            state
                .async_queue_metrics
                .record_push_enqueue(rows.len() as u64);
            Ok(())
        }
        Err(err) => {
            state.async_queue_metrics.record_push_fallback_sync();
            warn!(
                node_id = node_id,
                row_count = rows.len(),
                "push traffic worker unavailable, falling back to synchronous write",
            );
            let merged_rows = merge_push_traffic_rows(&err.0.rows);
            write_merged_push_traffic(state, err.0.node_id, &merged_rows).await
        }
    }
}

fn merge_push_traffic_rows(rows: &[QueuedPushTrafficRow]) -> Vec<AggregatedTrafficRow> {
    let mut merged = HashMap::<i64, (i64, i64)>::new();
    for row in rows {
        let entry = merged.entry(row.user_id).or_insert((0, 0));
        entry.0 += row.upload.max(0);
        entry.1 += row.download.max(0);
    }
    let mut merged_rows = merged
        .into_iter()
        .map(|(user_id, (upload, download))| {
            let billed = ((upload + download) as f64).ceil() as i64;
            (user_id, upload, download, billed.max(0))
        })
        .collect::<Vec<AggregatedTrafficRow>>();
    merged_rows.sort_unstable_by_key(|(user_id, _, _, _)| *user_id);
    merged_rows
}

async fn write_merged_push_traffic(
    state: &AppState,
    node_id: u64,
    merged_rows: &[AggregatedTrafficRow],
) -> Result<(), sqlx::Error> {
    if merged_rows.is_empty() {
        return Ok(());
    }

    let today = Utc::now().date_naive().to_string();
    let now_ts = Utc::now().timestamp();
    let multiplier = 1.0_f64;

    retry_db_write("uniproxy_push_write", || async {
        execute_record_push_traffic(state, node_id, &today, now_ts, multiplier, merged_rows).await
    })
    .await
}

async fn execute_record_push_traffic(
    state: &AppState,
    node_id: u64,
    today: &str,
    now_ts: i64,
    multiplier: f64,
    merged_rows: &[AggregatedTrafficRow],
) -> Result<(), sqlx::Error> {
    let mut tx = state.db.begin().await?;
    batch_update_user_traffic_totals(&mut tx, now_ts, merged_rows).await?;

    let total_delta = merged_rows
        .iter()
        .map(|(_, upload, download, _)| upload + download)
        .sum::<i64>();
    sqlx::query("UPDATE server_nodes SET traffic_used = traffic_used + ? WHERE id = ?")
        .bind(total_delta)
        .bind(node_id)
        .execute(&mut *tx)
        .await?;

    batch_insert_node_traffic_records(&mut tx, node_id, today, merged_rows).await?;
    batch_insert_user_traffic_usage_logs(&mut tx, node_id, now_ts, multiplier, merged_rows)
        .await?;

    let user_ids = merged_rows
        .iter()
        .map(|(user_id, _, _, _)| *user_id)
        .collect::<Vec<_>>();
    let subscription_ids =
        load_primary_subscription_ids_for_node_users(&mut tx, node_id, now_ts, &user_ids).await?;
    let mut subscription_usage_rows = merged_rows
        .iter()
        .filter_map(|(user_id, _upload, _download, billed)| {
            subscription_ids
                .get(user_id)
                .copied()
                .map(|subscription_id| (subscription_id, *billed))
        })
        .collect::<Vec<_>>();
    subscription_usage_rows.sort_unstable_by_key(|(subscription_id, _)| *subscription_id);
    if !subscription_usage_rows.is_empty() {
        batch_update_subscription_usage(&mut tx, &subscription_usage_rows).await?;
    }

    tx.commit().await
}

async fn enqueue_alive_sessions(
    state: &AppState,
    node_id: u64,
    payload: &Map<String, Value>,
    allowed_set: &HashSet<i64>,
) -> Result<(), sqlx::Error> {
    let reported = normalize_alive_payload(payload, allowed_set);
    if reported.is_empty() {
        return Ok(());
    }

    let mut users = reported
        .into_iter()
        .map(|(user_id, ips)| {
            let mut ips = ips.into_iter().collect::<Vec<_>>();
            ips.sort();
            QueuedAliveSessionUser { user_id, ips }
        })
        .collect::<Vec<_>>();
    users.sort_unstable_by_key(|user| user.user_id);

    let queued_user_count = users.len() as u64;
    let job = QueuedAliveSessionJob { node_id, users };
    match state.alive_session_tx.send(job).await {
        Ok(()) => {
            state
                .async_queue_metrics
                .record_alive_enqueue(queued_user_count);
            Ok(())
        }
        Err(err) => {
            state.async_queue_metrics.record_alive_fallback_sync();
            warn!(
                node_id = node_id,
                user_count = err.0.users.len(),
                "alive session worker unavailable, falling back to synchronous write",
            );
            let reported = err
                .0
                .users
                .into_iter()
                .map(|user| (user.user_id, user.ips.into_iter().collect::<HashSet<_>>()))
                .collect::<HashMap<_, _>>();
            write_merged_alive_sessions(state, err.0.node_id, &reported).await
        }
    }
}

fn normalize_alive_payload(
    payload: &Map<String, Value>,
    allowed_set: &HashSet<i64>,
) -> HashMap<i64, HashSet<String>> {
    let mut reported = HashMap::<i64, HashSet<String>>::new();
    for (user_id, ips_value) in payload {
        let user_id = match user_id.parse::<i64>() {
            Ok(id) if id > 0 && allowed_set.contains(&id) => id,
            _ => continue,
        };
        let ips = ips_value
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|value| value.as_str().map(|v| v.trim().to_string()))
                    .filter(|value| !value.is_empty())
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default();
        if ips.is_empty() {
            continue;
        }
        reported.insert(user_id, ips);
    }
    reported
}

async fn write_merged_alive_sessions(
    state: &AppState,
    node_id: u64,
    reported: &HashMap<i64, HashSet<String>>,
) -> Result<(), sqlx::Error> {
    if reported.is_empty() {
        return Ok(());
    }

    retry_db_write("uniproxy_alive_write", || async {
        execute_upsert_alive_sessions(state, node_id, reported).await
    })
    .await
}

async fn execute_upsert_alive_sessions(
    state: &AppState,
    node_id: u64,
    reported: &HashMap<i64, HashSet<String>>,
) -> Result<(), sqlx::Error> {
    let mut tx = state.db.begin().await?;
    let mut user_ids = reported.keys().copied().collect::<Vec<_>>();
    user_ids.sort_unstable();
    let existing = load_existing_alive_sessions(&mut tx, node_id, &user_ids).await?;

    let mut stale_rows = Vec::<(i64, String)>::new();
    let mut insert_rows = Vec::<(i64, String)>::new();
    let mut sorted_user_ids = reported.keys().copied().collect::<Vec<_>>();
    sorted_user_ids.sort_unstable();
    for user_id in sorted_user_ids {
        let Some(ips) = reported.get(&user_id) else {
            continue;
        };
        let existing_ips = existing.get(&user_id).cloned().unwrap_or_default();

        let mut stale_ip_list = existing_ips
            .iter()
            .filter(|ip| !ips.contains(*ip))
            .cloned()
            .collect::<Vec<_>>();
        stale_ip_list.sort();
        for stale_ip in stale_ip_list {
            stale_rows.push((user_id, stale_ip));
        }

        let mut insert_ip_list = ips.iter().cloned().collect::<Vec<_>>();
        insert_ip_list.sort();
        for ip in insert_ip_list {
            insert_rows.push((user_id, ip));
        }
    }

    batch_delete_stale_alive_sessions(&mut tx, node_id, &stale_rows).await?;
    batch_upsert_alive_sessions(&mut tx, node_id, &insert_rows).await?;
    batch_update_user_online_counts(&mut tx, reported).await?;

    tx.commit().await
}

async fn load_existing_alive_sessions(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    node_id: u64,
    user_ids: &[i64],
) -> Result<HashMap<i64, HashSet<String>>, sqlx::Error> {
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut result = HashMap::<i64, HashSet<String>>::new();
    for chunk in user_ids.chunks(500) {
        let placeholders = vec!["?"; chunk.len()].join(",");
        let sql = format!(
            "SELECT user_id, ip_address
             FROM user_online_sessions
             WHERE node_id = ?
               AND user_id IN ({})",
            placeholders
        );
        let mut query = sqlx::query(&sql).bind(node_id);
        for user_id in chunk {
            query = query.bind(*user_id);
        }
        let rows = query.fetch_all(&mut **tx).await?;
        for row in rows {
            let user_id: i64 = row.get("user_id");
            let ip_address: String = row.get("ip_address");
            result.entry(user_id).or_default().insert(ip_address);
        }
    }
    Ok(result)
}

async fn batch_delete_stale_alive_sessions(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    node_id: u64,
    rows: &[(i64, String)],
) -> Result<(), sqlx::Error> {
    if rows.is_empty() {
        return Ok(());
    }

    for chunk in rows.chunks(500) {
        let conditions = vec!["(user_id = ? AND ip_address = ?)"; chunk.len()].join(" OR ");
        let sql = format!(
            "DELETE FROM user_online_sessions
             WHERE node_id = ?
               AND ({})",
            conditions
        );
        let mut query = sqlx::query(&sql).bind(node_id);
        for (user_id, ip) in chunk {
            query = query.bind(*user_id).bind(ip);
        }
        query.execute(&mut **tx).await?;
    }

    Ok(())
}

async fn batch_upsert_alive_sessions(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    node_id: u64,
    rows: &[(i64, String)],
) -> Result<(), sqlx::Error> {
    if rows.is_empty() {
        return Ok(());
    }

    for chunk in rows.chunks(500) {
        let values_sql = vec!["(?, ?, ?, 1, 0, 0, NOW(), NOW(), NOW())"; chunk.len()].join(",");
        let sql = format!(
            "INSERT INTO user_online_sessions
                (user_id, node_id, ip_address, connection_count, upload_traffic, download_traffic, last_activity, created_at, updated_at)
             VALUES {}
             ON DUPLICATE KEY UPDATE
                connection_count = 1,
                last_activity = NOW(),
                updated_at = NOW()",
            values_sql,
        );
        let mut query = sqlx::query(&sql);
        for (user_id, ip) in chunk {
            query = query.bind(*user_id).bind(node_id).bind(ip);
        }
        query.execute(&mut **tx).await?;
    }

    Ok(())
}

async fn batch_update_user_online_counts(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    reported: &HashMap<i64, HashSet<String>>,
) -> Result<(), sqlx::Error> {
    if reported.is_empty() {
        return Ok(());
    }

    let mut rows = reported
        .iter()
        .map(|(user_id, ips)| {
            let mut ipv4 = HashSet::new();
            let mut ipv6 = HashSet::new();
            for ip in ips {
                if ip.contains(':') {
                    ipv6.insert(ip);
                } else {
                    ipv4.insert(ip);
                }
            }
            (*user_id, ipv4.len().max(ipv6.len()) as i64)
        })
        .collect::<Vec<_>>();
    rows.sort_unstable_by_key(|(user_id, _)| *user_id);

    let user_ids = rows.iter().map(|(user_id, _)| *user_id).collect::<Vec<_>>();
    let mut reset_builder = QueryBuilder::<sqlx::MySql>::new("UPDATE v2_user SET online_count = 0 WHERE id IN (");
    {
        let mut separated = reset_builder.separated(", ");
        for user_id in &user_ids {
            separated.push_bind(*user_id);
        }
    }
    reset_builder.push(")");
    reset_builder.build().execute(&mut **tx).await?;

    for (user_id, count) in rows {
        sqlx::query(
            "UPDATE v2_user
             SET online_count = ?, last_online_at = NOW()
             WHERE id = ?"
        )
        .bind(count)
        .bind(user_id)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}
