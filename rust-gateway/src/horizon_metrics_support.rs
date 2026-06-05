use crate::*;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct HorizonMetricsSnapshotStats {
    pub(crate) ran: bool,
    pub(crate) measured_jobs: usize,
    pub(crate) measured_queues: usize,
    pub(crate) snapshot_time: i64,
}

pub(crate) async fn run_horizon_metrics_snapshot(
    state: &AppState,
    now_ts: i64,
) -> Result<HorizonMetricsSnapshotStats, String> {
    if now_ts <= 0 {
        return Ok(HorizonMetricsSnapshotStats::default());
    }
    if now_ts % 300 >= 60 {
        return Ok(HorizonMetricsSnapshotStats::default());
    }

    let horizon_prefix = horizon_prefix();
    let lock_key = format!("{horizon_prefix}metrics:snapshot");
    let acquired = redis_set_nx_ex_raw_on_db(state, 0, &lock_key, 270, &now_ts.to_string()).await?;
    if !acquired {
        return Ok(HorizonMetricsSnapshotStats::default());
    }

    let measured_jobs = smembers_horizon_set(state, &horizon_prefix, "measured_jobs").await?;
    let measured_queues = smembers_horizon_set(state, &horizon_prefix, "measured_queues").await?;

    for raw_job in &measured_jobs {
        let job = raw_job.strip_prefix("job:").unwrap_or(raw_job);
        store_snapshot_for_job(state, &horizon_prefix, job, now_ts).await?;
    }
    for raw_queue in &measured_queues {
        let queue = raw_queue.strip_prefix("queue:").unwrap_or(raw_queue);
        store_snapshot_for_queue(state, &horizon_prefix, queue, now_ts).await?;
    }

    set_horizon_string(state, &horizon_prefix, "last_snapshot_at", &now_ts.to_string()).await?;

    Ok(HorizonMetricsSnapshotStats {
        ran: true,
        measured_jobs: measured_jobs.len(),
        measured_queues: measured_queues.len(),
        snapshot_time: now_ts,
    })
}

async fn store_snapshot_for_job(
    state: &AppState,
    prefix: &str,
    job: &str,
    now_ts: i64,
) -> Result<(), String> {
    let key = format!("job:{job}");
    let runtime = get_horizon_hash_number(state, prefix, &key, "runtime").await?;
    let throughput = get_horizon_hash_number(state, prefix, &key, "throughput").await?;
    let payload = json!({
        "throughput": throughput,
        "runtime": runtime,
        "time": now_ts,
    })
    .to_string();
    zadd_horizon_snapshot(state, prefix, &format!("snapshot:{key}"), now_ts, &payload).await?;
    trim_horizon_snapshot(state, prefix, &format!("snapshot:{key}"), load_trim_limit("job")).await
}

async fn store_snapshot_for_queue(
    state: &AppState,
    prefix: &str,
    queue: &str,
    now_ts: i64,
) -> Result<(), String> {
    let key = format!("queue:{queue}");
    let runtime = get_horizon_hash_number(state, prefix, &key, "runtime").await?;
    let throughput = get_horizon_hash_number(state, prefix, &key, "throughput").await?;
    let wait = calculate_queue_wait_seconds(state, prefix, queue, runtime).await?;
    let payload = json!({
        "throughput": throughput,
        "runtime": runtime,
        "wait": wait,
        "time": now_ts,
    })
    .to_string();
    zadd_horizon_snapshot(state, prefix, &format!("snapshot:{key}"), now_ts, &payload).await?;
    trim_horizon_snapshot(state, prefix, &format!("snapshot:{key}"), load_trim_limit("queue")).await
}

fn load_trim_limit(kind: &str) -> i64 {
    match kind {
        "job" => env::var("HORIZON_METRICS_TRIM_JOBS")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(24),
        "queue" => env::var("HORIZON_METRICS_TRIM_QUEUES")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .filter(|v| *v > 0)
            .unwrap_or(24),
        _ => 24,
    }
}

async fn calculate_queue_wait_seconds(
    state: &AppState,
    prefix: &str,
    queue_name: &str,
    runtime_ms: f64,
) -> Result<i64, String> {
    let supervisors = load_horizon_supervisor_processes(state, prefix).await?;
    let total_processes = supervisors
        .iter()
        .map(|processes| processes.get(queue_name).copied().unwrap_or(0))
        .sum::<i64>();
    let backlog = queue_length_for_queue_key(state, queue_name).await?;
    let time_to_clear_ms = (backlog as f64) * runtime_ms;
    let wait = if total_processes <= 0 {
        (time_to_clear_ms / 1000.0).round()
    } else {
        ((time_to_clear_ms / (total_processes as f64)) / 1000.0).round()
    };
    Ok(wait as i64)
}

async fn load_horizon_supervisor_processes(
    state: &AppState,
    prefix: &str,
) -> Result<Vec<HashMap<String, i64>>, String> {
    let supervisors = smembers_horizon_set(state, prefix, "supervisors").await?;
    let mut out = Vec::new();
    for supervisor in supervisors {
        let values = hmget_horizon(
            state,
            prefix,
            &supervisor,
            &["processes"],
        )
        .await?;
        let processes = values
            .first()
            .cloned()
            .flatten()
            .and_then(|raw| serde_json::from_str::<HashMap<String, i64>>(&raw).ok())
            .unwrap_or_default();
        out.push(processes);
    }
    Ok(out)
}

async fn queue_length_for_queue_key(state: &AppState, queue_key: &str) -> Result<i64, String> {
    let mut total = 0_i64;
    for queue_name in queue_key.split(',').map(str::trim).filter(|v| !v.is_empty()) {
        let raw = redis_command_on_db(
            state,
            0,
            &[
                "LLEN".to_string(),
                format!("{}queues:{queue_name}", state.redis_prefix),
            ],
        )
        .await?;
        total += parse_resp_integer_raw(&raw).unwrap_or(0);
    }
    Ok(total)
}

async fn get_horizon_hash_number(
    state: &AppState,
    prefix: &str,
    key: &str,
    field: &str,
) -> Result<f64, String> {
    let values = hmget_horizon(state, prefix, key, &[field]).await?;
    Ok(values
        .first()
        .cloned()
        .flatten()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0))
}

async fn hmget_horizon(
    state: &AppState,
    prefix: &str,
    key: &str,
    fields: &[&str],
) -> Result<Vec<Option<String>>, String> {
    let mut cmd = vec!["HMGET".to_string(), format!("{prefix}{key}")];
    cmd.extend(fields.iter().map(|field| (*field).to_string()));
    let raw = redis_command_on_db(state, 0, &cmd).await?;
    Ok(parse_resp_array_option_strings_raw(&raw))
}

async fn smembers_horizon_set(state: &AppState, prefix: &str, key: &str) -> Result<Vec<String>, String> {
    let raw = redis_command_on_db(
        state,
        0,
        &["SMEMBERS".to_string(), format!("{prefix}{key}")],
    )
    .await?;
    Ok(parse_resp_array_strings_raw(&raw))
}

async fn zadd_horizon_snapshot(
    state: &AppState,
    prefix: &str,
    key: &str,
    score: i64,
    payload: &str,
) -> Result<(), String> {
    let raw = redis_command_on_db(
        state,
        0,
        &[
            "ZADD".to_string(),
            format!("{prefix}{key}"),
            score.to_string(),
            payload.to_string(),
        ],
    )
    .await?;
    if raw.starts_with(':') {
        return Ok(());
    }
    Err(format!("redis ZADD failed: {}", raw.trim()))
}

async fn trim_horizon_snapshot(
    state: &AppState,
    prefix: &str,
    key: &str,
    keep: i64,
) -> Result<(), String> {
    let end_rank = -((keep + 1).abs());
    let raw = redis_command_on_db(
        state,
        0,
        &[
            "ZREMRANGEBYRANK".to_string(),
            format!("{prefix}{key}"),
            "0".to_string(),
            end_rank.to_string(),
        ],
    )
    .await?;
    if raw.starts_with(':') {
        return Ok(());
    }
    Err(format!("redis ZREMRANGEBYRANK failed: {}", raw.trim()))
}

async fn set_horizon_string(state: &AppState, prefix: &str, key: &str, value: &str) -> Result<(), String> {
    let raw = redis_command_on_db(
        state,
        0,
        &[
            "SET".to_string(),
            format!("{prefix}{key}"),
            value.to_string(),
        ],
    )
    .await?;
    if raw.starts_with("+OK") {
        return Ok(());
    }
    Err(format!("redis SET failed: {}", raw.trim()))
}

async fn redis_set_nx_ex_raw_on_db(
    state: &AppState,
    db: i64,
    key: &str,
    ttl_seconds: i64,
    value: &str,
) -> Result<bool, String> {
    let raw = redis_command_on_db(
        state,
        db,
        &[
            "SET".to_string(),
            key.to_string(),
            value.to_string(),
            "NX".to_string(),
            "EX".to_string(),
            ttl_seconds.to_string(),
        ],
    )
    .await?;
    if raw.starts_with("+OK") {
        return Ok(true);
    }
    if raw.trim().is_empty() || raw.starts_with("$-1") || raw.trim() == "(nil)" {
        return Ok(false);
    }
    Err(format!("redis SET NX EX failed: {}", raw.trim()))
}

async fn redis_command_on_db(
    state: &AppState,
    db: i64,
    parts: &[String],
) -> Result<String, String> {
    let addr = format!("{}:{}", state.redis_host, state.redis_port);
    let mut stream = TcpStream::connect(&addr)
        .await
        .map_err(|err| format!("connect redis failed: {err}"))?;

    if let Some(password) = &state.redis_password {
        let auth = redis_resp_array(&[b"AUTH".as_slice(), password.as_bytes()]);
        stream.write_all(auth.as_bytes()).await.map_err(|err| format!("redis AUTH write failed: {err}"))?;
        let reply = redis_read_reply(&mut stream).await?;
        if !reply.starts_with("+OK") {
            return Err(format!("redis AUTH failed: {}", reply.trim()));
        }
    }

    let select = redis_resp_array(&[b"SELECT".as_slice(), db.to_string().as_bytes()]);
    stream.write_all(select.as_bytes()).await.map_err(|err| format!("redis SELECT write failed: {err}"))?;
    let reply = redis_read_reply(&mut stream).await?;
    if !reply.starts_with("+OK") {
        return Err(format!("redis SELECT failed: {}", reply.trim()));
    }

    let bytes = parts.iter().map(|part| part.as_bytes()).collect::<Vec<_>>();
    let command = redis_resp_array(&bytes);
    stream.write_all(command.as_bytes()).await.map_err(|err| format!("redis command write failed: {err}"))?;
    redis_read_reply(&mut stream).await
}

fn parse_resp_integer_raw(raw: &str) -> Option<i64> {
    raw.strip_prefix(':')
        .map(str::trim)
        .and_then(|value| value.parse::<i64>().ok())
}

fn parse_resp_array_strings_raw(raw: &str) -> Vec<String> {
    parse_resp_array_option_strings_raw(raw)
        .into_iter()
        .flatten()
        .collect()
}

fn parse_resp_array_option_strings_raw(raw: &str) -> Vec<Option<String>> {
    let bytes = raw.as_bytes();
    if bytes.first().copied() != Some(b'*') {
        return Vec::new();
    }
    let Some(header_end) = raw.find("\r\n") else {
        return Vec::new();
    };
    let count = raw[1..header_end].parse::<usize>().ok().unwrap_or(0);
    let mut index = header_end + 2;
    let mut values = Vec::with_capacity(count);
    for _ in 0..count {
        if index >= raw.len() {
            break;
        }
        let prefix = raw.as_bytes()[index];
        if prefix == b'$' {
            let Some(len_end_rel) = raw[index..].find("\r\n") else {
                break;
            };
            let len_end = index + len_end_rel;
            let len = raw[index + 1..len_end].parse::<isize>().ok().unwrap_or(-1);
            index = len_end + 2;
            if len < 0 {
                values.push(None);
                continue;
            }
            let len = len as usize;
            if index + len > raw.len() {
                break;
            }
            let value = raw[index..index + len].to_string();
            values.push(Some(value));
            index += len + 2;
        } else {
            break;
        }
    }
    values
}

fn horizon_prefix() -> String {
    env::var("HORIZON_PREFIX")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            let app_name = env::var("APP_NAME").unwrap_or_else(|_| "laravel".to_string());
            format!("{}_horizon:", slug_for_prefix(&app_name))
        })
}
