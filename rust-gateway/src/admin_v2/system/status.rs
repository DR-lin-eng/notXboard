use crate::*;
use super::models::{HorizonJobRecord, HorizonJobRetryRecord};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct HorizonMasterRecord {
    name: String,
    environment: Option<String>,
    pid: Option<String>,
    status: Option<String>,
    supervisors: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct HorizonSupervisorRecord {
    name: String,
    master: Option<String>,
    pid: Option<String>,
    status: Option<String>,
    processes: HashMap<String, i64>,
    options: Option<Value>,
}

pub async fn build_queue_workload_payload(state: &AppState) -> Result<Value, Response<Body>> {
    let supervisors = load_horizon_supervisors(state).await?;
    let measured_queues = load_horizon_measured_queues(state).await?;
    let mut queue_names = supervisors
        .iter()
        .flat_map(|supervisor| supervisor.processes.keys().cloned())
        .collect::<HashSet<_>>();
    queue_names.extend(measured_queues);

    let queue_backlogs = load_ready_queue_lengths(state, &queue_names).await?;
    let mut rows = queue_names
        .into_iter()
        .collect::<Vec<_>>();
    rows.sort();

    let mut payload = Vec::new();
    for queue_key in rows {
        let total_processes = supervisors
            .iter()
            .map(|supervisor| supervisor.processes.get(&queue_key).copied().unwrap_or(0))
            .sum::<i64>();
        let split_queues = build_split_queue_rows(&queue_key, &queue_backlogs, total_processes);
        let queue_names = split_queue_names(&queue_key);
        let length = queue_names
            .iter()
            .map(|name| queue_backlogs.get(name).copied().unwrap_or(0))
            .sum::<i64>();
        let runtime_ms = horizon_runtime_for_queue(
            state,
            queue_names.first().map(|value| value.as_str()).unwrap_or(""),
        )
        .await?
        .unwrap_or(0.0);
        let wait_seconds = if total_processes <= 0 {
            ((length as f64) * runtime_ms / 1000.0).round() as i64
        } else {
            (((length as f64) * runtime_ms / (total_processes as f64)) / 1000.0).round() as i64
        };

        payload.push(json!({
            "name": queue_key.split_once(':').map(|(_, queue)| queue.to_string()).unwrap_or(queue_key.clone()),
            "length": length,
            "wait": wait_seconds.max(0),
            "processes": total_processes.max(0),
            "split_queues": if split_queues.is_empty() { Value::Null } else { Value::Array(split_queues) }
        }));
    }

    Ok(Value::Array(payload))
}

pub async fn build_queue_process_count(state: &AppState) -> Result<i64, Response<Body>> {
    let supervisors = load_horizon_supervisors(state).await?;
    Ok(supervisors
        .iter()
        .map(|supervisor| supervisor.processes.values().copied().sum::<i64>())
        .sum())
}

pub async fn build_queue_paused_master_count(state: &AppState) -> Result<i64, Response<Body>> {
    let masters = load_horizon_masters(state).await?;
    Ok(masters
        .iter()
        .filter(|master| master.status.as_deref() == Some("paused"))
        .count() as i64)
}

pub async fn build_queue_master_status(state: &AppState) -> Result<Value, Response<Body>> {
    let masters = load_horizon_masters(state).await?;
    if masters.is_empty() {
        return Ok(Value::String("inactive".to_string()));
    }
    if masters
        .iter()
        .all(|master| master.status.as_deref() == Some("paused"))
    {
        return Ok(Value::String("paused".to_string()));
    }
    Ok(Value::String("running".to_string()))
}

pub async fn build_jobs_processed_per_minute(state: &AppState) -> Result<Value, Response<Body>> {
    let measured_queues = load_horizon_measured_queues(state).await?;
    let mut throughput = 0.0;
    for queue in &measured_queues {
        throughput += horizon_throughput_for_queue(state, queue).await?;
    }
    let minutes = horizon_minutes_since_last_snapshot(state).await?.unwrap_or(1.0).max(1.0);
    Ok(Value::from((throughput / minutes).round() as i64))
}

pub async fn build_queue_with_max_runtime(state: &AppState) -> Result<Value, Response<Body>> {
    let measured_queues = load_horizon_measured_queues(state).await?;
    let mut best: Option<(String, f64)> = None;
    for queue in measured_queues {
        if let Some(snapshot) = horizon_latest_queue_snapshot(state, &queue).await? {
            let runtime = snapshot.runtime.unwrap_or(0.0);
            if best.as_ref().map(|(_, value)| runtime > *value).unwrap_or(true) {
                best = Some((queue, runtime));
            }
        }
    }
    let best = best.map(|(queue, _)| Value::String(queue));
    Ok(best.unwrap_or(Value::Null))
}

pub async fn build_queue_with_max_throughput(state: &AppState) -> Result<Value, Response<Body>> {
    let measured_queues = load_horizon_measured_queues(state).await?;
    let mut best: Option<(String, f64)> = None;
    for queue in measured_queues {
        if let Some(snapshot) = horizon_latest_queue_snapshot(state, &queue).await? {
            let throughput = snapshot.throughput.unwrap_or(0.0);
            if best.as_ref().map(|(_, value)| throughput > *value).unwrap_or(true) {
                best = Some((queue, throughput));
            }
        }
    }
    let best = best.map(|(queue, _)| Value::String(queue));
    Ok(best.unwrap_or(Value::Null))
}

pub async fn build_recent_jobs_count(state: &AppState) -> Result<i64, Response<Body>> {
    let measured_queues = load_horizon_measured_queues(state).await?;
    let mut total = 0_i64;
    for queue in &measured_queues {
        total += horizon_throughput_for_queue(state, queue).await?.round() as i64;
    }
    Ok(total)
}

pub async fn build_measured_jobs_payload(state: &AppState) -> Result<Value, Response<Body>> {
    let values = horizon_smembers(state, "measured_jobs").await?;
    let mut jobs = values
        .into_iter()
        .map(|value| value.strip_prefix("job:").unwrap_or(&value).to_string())
        .collect::<Vec<_>>();
    jobs.sort();
    Ok(Value::Array(jobs.into_iter().map(Value::String).collect()))
}

pub async fn build_measured_queues_payload(state: &AppState) -> Result<Value, Response<Body>> {
    let mut queues = load_horizon_measured_queues(state).await?;
    queues.sort();
    Ok(Value::Array(queues.into_iter().map(Value::String).collect()))
}

pub async fn build_job_metric_snapshots_payload(
    state: &AppState,
    job_id: &str,
) -> Result<Value, Response<Body>> {
    let snapshots = load_horizon_snapshots(state, &format!("snapshot:job:{job_id}")).await?;
    Ok(Value::Array(
        snapshots
            .into_iter()
            .map(|snapshot| {
                json!({
                    "throughput": snapshot.throughput.unwrap_or(0.0) as i64,
                    "runtime": ((snapshot.runtime.unwrap_or(0.0) / 1000.0) * 1000.0).round() / 1000.0,
                    "time": snapshot.time,
                })
            })
            .collect(),
    ))
}

pub async fn build_job_list_payload(
    state: &AppState,
    list_key: &str,
    starting_at: i64,
) -> Result<Value, Response<Body>> {
    let ids = horizon_zrange_slice(state, list_key, starting_at + 1, starting_at + 50).await?;
    let jobs = load_horizon_jobs(state, &ids, starting_at + 1).await?;
    Ok(json!({
        "jobs": jobs,
        "total": horizon_zcount_recent(state, list_key, horizon_minutes_for_job_list(list_key)).await?,
    }))
}

pub async fn build_failed_job_list_payload(
    state: &AppState,
    starting_at: i64,
) -> Result<Value, Response<Body>> {
    build_job_list_payload(state, "failed_jobs", starting_at).await
}

pub async fn build_job_detail_payload(
    state: &AppState,
    job_id: &str,
) -> Result<Value, Response<Body>> {
    let jobs = load_horizon_jobs(state, &[job_id.to_string()], 0).await?;
    Ok(jobs.into_iter().next().unwrap_or(Value::Null))
}

pub async fn build_monitored_tags_payload(
    state: &AppState,
) -> Result<Value, Response<Body>> {
    let tags = load_monitoring_tags(state).await?;
    let mut rows = Vec::new();
    for tag in tags {
        let active = horizon_zcard(state, &tag).await?;
        let failed = horizon_zcard(state, &format!("failed:{tag}")).await?;
        rows.push(json!({
            "tag": tag,
            "count": active + failed,
        }));
    }
    rows.sort_by(|left, right| {
        left.get("tag")
            .and_then(Value::as_str)
            .cmp(&right.get("tag").and_then(Value::as_str))
    });
    Ok(Value::Array(rows))
}

pub async fn build_monitored_tag_jobs_payload(
    state: &AppState,
    tag: &str,
    starting_at: i64,
    limit: i64,
) -> Result<Value, Response<Body>> {
    let ids = horizon_zrevrange_slice(state, tag, starting_at, starting_at + limit - 1).await?;
    let jobs = load_horizon_jobs(state, &ids, starting_at).await?;
    let total = horizon_zcard(state, tag).await?;
    Ok(json!({
        "jobs": jobs,
        "total": total,
    }))
}

pub async fn build_batches_payload(
    state: &AppState,
    query: Option<&str>,
    before_id: Option<&str>,
) -> Result<Value, Response<Body>> {
    let batches = if let Some(query) = query.filter(|value| !value.trim().is_empty()) {
        load_job_batches_by_query(state, query.trim(), before_id).await?
    } else {
        load_recent_job_batches(state, before_id, 50).await?
    };
    Ok(json!({ "batches": batches }))
}

pub async fn build_batch_detail_payload(
    state: &AppState,
    batch_id: &str,
) -> Result<Value, Response<Body>> {
    let batch = load_job_batch_by_id(state, batch_id).await?;
    let Some(batch) = batch else {
        return Ok(json!({
            "batch": Value::Null,
            "failedJobs": Value::Null,
        }));
    };

    let failed_job_ids = batch
        .get("failed_job_ids")
        .and_then(Value::as_array)
        .map(|items| {
            items.iter()
                .filter_map(Value::as_str)
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let failed_jobs = if failed_job_ids.is_empty() {
        Value::Null
    } else {
        Value::Array(load_horizon_jobs(state, &failed_job_ids, 0).await?)
    };
    Ok(json!({
        "batch": batch,
        "failedJobs": failed_jobs,
    }))
}

pub async fn build_queue_metric_snapshots_payload(
    state: &AppState,
    queue_id: &str,
) -> Result<Value, Response<Body>> {
    let snapshots = load_horizon_snapshots(state, &format!("snapshot:queue:{queue_id}")).await?;
    Ok(Value::Array(
        snapshots
            .into_iter()
            .map(|snapshot| {
                json!({
                    "throughput": snapshot.throughput.unwrap_or(0.0) as i64,
                    "runtime": ((snapshot.runtime.unwrap_or(0.0) / 1000.0) * 1000.0).round() / 1000.0,
                    "wait": snapshot.wait.unwrap_or(0.0),
                    "time": snapshot.time,
                })
            })
            .collect(),
    ))
}

pub async fn build_queue_masters_payload(state: &AppState) -> Result<Value, Response<Body>> {
    let masters = load_horizon_masters(state).await?;
    let supervisors = load_horizon_supervisors(state).await?;
    let supervisor_groups = supervisors
        .into_iter()
        .fold(HashMap::<String, Vec<Value>>::new(), |mut groups, supervisor| {
            let key = supervisor
                .master
                .clone()
                .unwrap_or_default();
            groups.entry(key).or_default().push(json!({
                "name": supervisor.name,
                "master": supervisor.master,
                "pid": supervisor.pid,
                "status": supervisor.status,
                "processes": supervisor.processes,
                "options": supervisor.options.unwrap_or(Value::Null),
            }));
            groups
        });

    let mut payload = masters
        .into_iter()
        .map(|master| {
            let name = master.name.clone();
            json!({
                "name": name,
                "environment": master.environment,
                "pid": master.pid,
                "status": master.status,
                "supervisors": supervisor_groups.get(&master.name).cloned().unwrap_or_default(),
            })
        })
        .collect::<Vec<_>>();

    payload.sort_by(|left, right| {
        left.get("name")
            .and_then(Value::as_str)
            .cmp(&right.get("name").and_then(Value::as_str))
    });

    Ok(Value::Array(payload))
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct HorizonQueueSnapshot {
    throughput: Option<f64>,
    runtime: Option<f64>,
    wait: Option<f64>,
    time: Option<i64>,
}

async fn load_horizon_masters(state: &AppState) -> Result<Vec<HorizonMasterRecord>, Response<Body>> {
    let names = load_horizon_zset_recent_members(state, "masters", 14).await?;
    if names.is_empty() {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    for name in names {
        let key = format!("master:{name}");
        let values = horizon_hmget(
            state,
            &key,
            &["name", "pid", "status", "supervisors", "environment"],
        )
        .await?;
        let Some(master_name) = values.first().cloned().flatten() else {
            continue;
        };
        out.push(HorizonMasterRecord {
            name: master_name,
            pid: values.get(1).cloned().flatten(),
            status: values.get(2).cloned().flatten(),
            supervisors: values
                .get(3)
                .cloned()
                .flatten()
                .and_then(|raw| serde_json::from_str::<Vec<String>>(&raw).ok())
                .unwrap_or_default(),
            environment: values.get(4).cloned().flatten(),
        });
    }
    Ok(out)
}

async fn load_horizon_supervisors(state: &AppState) -> Result<Vec<HorizonSupervisorRecord>, Response<Body>> {
    let names = load_horizon_zset_recent_members(state, "supervisors", 29).await?;
    if names.is_empty() {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    for name in names {
        let key = format!("supervisor:{name}");
        let values = horizon_hmget(
            state,
            &key,
            &["name", "master", "pid", "status", "processes", "options"],
        )
        .await?;
        let Some(supervisor_name) = values.first().cloned().flatten() else {
            continue;
        };
        let processes = values
            .get(4)
            .cloned()
            .flatten()
            .and_then(|raw| serde_json::from_str::<HashMap<String, i64>>(&raw).ok())
            .unwrap_or_default();
        let options = values
            .get(5)
            .cloned()
            .flatten()
            .and_then(|raw| serde_json::from_str::<Value>(&raw).ok());
        out.push(HorizonSupervisorRecord {
            name: supervisor_name,
            master: values.get(1).cloned().flatten(),
            pid: values.get(2).cloned().flatten(),
            status: values.get(3).cloned().flatten(),
            processes,
            options,
        });
    }
    Ok(out)
}

async fn load_horizon_measured_queues(
    state: &AppState,
) -> Result<Vec<String>, Response<Body>> {
    let values = horizon_smembers(state, "measured_queues").await?;
    Ok(values
        .into_iter()
        .map(|value| value.strip_prefix("queue:").unwrap_or(&value).to_string())
        .collect())
}

async fn load_ready_queue_lengths(
    state: &AppState,
    queue_keys: &HashSet<String>,
) -> Result<HashMap<String, i64>, Response<Body>> {
    let mut lengths = HashMap::new();
    for queue_key in queue_keys {
        for queue_name in split_queue_names(queue_key) {
            let redis_key = format!("queues:{queue_name}");
            let length = horizon_llen(state, &redis_key).await?;
            lengths.insert(queue_name, length);
        }
    }
    Ok(lengths)
}

fn build_split_queue_rows(
    queue_key: &str,
    queue_backlogs: &HashMap<String, i64>,
    total_processes: i64,
) -> Vec<Value> {
    let mut cumulative_wait = 0_i64;
    split_queue_names(queue_key)
        .into_iter()
        .map(|queue_name| {
            let length = queue_backlogs.get(&queue_name).copied().unwrap_or(0);
            let runtime_ms = horizon_runtime_for_queue_name(&queue_name).unwrap_or(0.0);
            let wait = if total_processes <= 0 {
                ((length as f64) * runtime_ms / 1000.0).round() as i64
            } else {
                (((length as f64) * runtime_ms / (total_processes as f64)) / 1000.0).round() as i64
            };
            cumulative_wait += wait.max(0);
            json!({
                "name": queue_name,
                "length": length,
                "wait": cumulative_wait,
            })
        })
        .collect()
}

fn split_queue_names(queue_key: &str) -> Vec<String> {
    queue_key
        .split_once(':')
        .map(|(_, queue_names)| queue_names)
        .unwrap_or(queue_key)
        .split(',')
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

async fn horizon_runtime_for_queue(state: &AppState, queue_name: &str) -> Result<Option<f64>, Response<Body>> {
    if let Some(runtime) = horizon_runtime_for_queue_name(queue_name) {
        return Ok(Some(runtime));
    }
    Ok(horizon_latest_queue_snapshot(state, queue_name)
        .await?
        .and_then(|snapshot| snapshot.runtime))
}

fn horizon_runtime_for_queue_name(queue_name: &str) -> Option<f64> {
    let _ = queue_name;
    None
}

async fn horizon_throughput_for_queue(
    state: &AppState,
    queue_name: &str,
) -> Result<f64, Response<Body>> {
    let values = horizon_hmget(state, &format!("queue:{queue_name}"), &["throughput"]).await?;
    if let Some(value) = values
        .first()
        .cloned()
        .flatten()
        .and_then(|value| value.parse::<f64>().ok())
    {
        return Ok(value);
    }
    Ok(horizon_latest_queue_snapshot(state, queue_name)
        .await?
        .and_then(|snapshot| snapshot.throughput)
        .unwrap_or(0.0))
}

async fn horizon_minutes_since_last_snapshot(state: &AppState) -> Result<Option<f64>, Response<Body>> {
    let raw = horizon_command(state, &["GET".to_string(), "last_snapshot_at".to_string()]).await?;
    let ts = parse_resp_bulk_string(&raw).and_then(|value| value.parse::<i64>().ok());
    Ok(ts.map(|last_snapshot_at| ((Utc::now().timestamp() - last_snapshot_at) as f64 / 60.0).max(1.0)))
}

async fn horizon_latest_queue_snapshot(
    state: &AppState,
    queue_name: &str,
) -> Result<Option<HorizonQueueSnapshot>, Response<Body>> {
    let raw = horizon_command(
        state,
        &[
            "ZREVRANGE".to_string(),
            format!("snapshot:queue:{queue_name}"),
            "0".to_string(),
            "0".to_string(),
        ],
    )
    .await?;
    let first = parse_resp_array_strings(&raw).into_iter().next();
    Ok(first.and_then(|value| serde_json::from_str::<HorizonQueueSnapshot>(&value).ok()))
}

async fn load_horizon_zset_recent_members(
    state: &AppState,
    key: &str,
    recent_seconds: i64,
) -> Result<Vec<String>, Response<Body>> {
    let min_score = (Utc::now().timestamp() - recent_seconds).to_string();
    let raw = horizon_command(
        state,
        &[
            "ZREVRANGEBYSCORE".to_string(),
            key.to_string(),
            "+inf".to_string(),
            min_score,
        ],
    )
    .await?;
    Ok(parse_resp_array_strings(&raw))
}

pub(super) async fn horizon_hmget(
    state: &AppState,
    key: &str,
    fields: &[&str],
) -> Result<Vec<Option<String>>, Response<Body>> {
    let mut command = vec!["HMGET".to_string(), key.to_string()];
    command.extend(fields.iter().map(|field| (*field).to_string()));
    let raw = horizon_command(state, &command).await?;
    Ok(parse_resp_array_option_strings(&raw))
}

async fn horizon_smembers(
    state: &AppState,
    key: &str,
) -> Result<Vec<String>, Response<Body>> {
    let raw = horizon_command(state, &["SMEMBERS".to_string(), key.to_string()]).await?;
    Ok(parse_resp_array_strings(&raw))
}

async fn load_horizon_snapshots(
    state: &AppState,
    key: &str,
) -> Result<Vec<HorizonQueueSnapshot>, Response<Body>> {
    let raw = horizon_command(
        state,
        &["ZRANGE".to_string(), key.to_string(), "0".to_string(), "-1".to_string()],
    )
    .await?;
    Ok(parse_resp_array_strings(&raw)
        .into_iter()
        .filter_map(|value| serde_json::from_str::<HorizonQueueSnapshot>(&value).ok())
        .collect())
}

async fn load_monitoring_tags(
    state: &AppState,
) -> Result<Vec<String>, Response<Body>> {
    horizon_smembers(state, "monitoring").await
}

async fn load_recent_job_batches(
    state: &AppState,
    before_id: Option<&str>,
    limit: i64,
) -> Result<Vec<Value>, Response<Body>> {
    let mut sql = String::from(
        "SELECT id, name, total_jobs, pending_jobs, failed_jobs, failed_job_ids, options, cancelled_at, created_at, finished_at
         FROM job_batches",
    );
    if before_id.is_some() {
        sql.push_str(" WHERE id < ?");
    }
    sql.push_str(" ORDER BY id DESC LIMIT ?");
    let mut query = sqlx::query(&sql);
    if let Some(before_id) = before_id {
        query = query.bind(before_id);
    }
    query = query.bind(limit);
    let rows = query.fetch_all(&state.db).await.map_err(internal_error)?;
    Ok(rows
        .iter()
        .map(serialize_job_batch_row)
        .collect())
}

async fn load_job_batches_by_query(
    state: &AppState,
    query_text: &str,
    before_id: Option<&str>,
) -> Result<Vec<Value>, Response<Body>> {
    let escaped = query_text.replace('%', "\\%").replace('_', "\\_");
    let pattern = format!("%{escaped}%");
    let mut sql = String::from(
        "SELECT id, name, total_jobs, pending_jobs, failed_jobs, failed_job_ids, options, cancelled_at, created_at, finished_at
         FROM job_batches
         WHERE (name LIKE ? OR id LIKE ?)",
    );
    if before_id.is_some() {
        sql.push_str(" AND id < ?");
    }
    sql.push_str(" ORDER BY id DESC LIMIT 50");
    let mut query = sqlx::query(&sql)
        .bind(&pattern)
        .bind(&pattern);
    if let Some(before_id) = before_id {
        query = query.bind(before_id);
    }
    let rows = query.fetch_all(&state.db).await.map_err(internal_error)?;
    Ok(rows
        .iter()
        .map(serialize_job_batch_row)
        .collect())
}

pub(super) async fn load_job_batch_by_id(
    state: &AppState,
    batch_id: &str,
) -> Result<Option<Value>, Response<Body>> {
    let row = sqlx::query(
        "SELECT id, name, total_jobs, pending_jobs, failed_jobs, failed_job_ids, options, cancelled_at, created_at, finished_at
         FROM job_batches
         WHERE id = ?
         LIMIT 1",
    )
    .bind(batch_id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?;
    Ok(row.as_ref().map(serialize_job_batch_row))
}

fn serialize_job_batch_row(row: &sqlx::mysql::MySqlRow) -> Value {
    let id = row.try_get::<String, _>("id").unwrap_or_default();
    let failed_job_ids = row
        .try_get::<Option<String>, _>("failed_job_ids")
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .unwrap_or_else(|| Value::Array(Vec::new()));
    let options = row
        .try_get::<Option<String>, _>("options")
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .unwrap_or(Value::Null);
    json!({
        "id": id,
        "name": row.try_get::<Option<String>, _>("name").ok().flatten(),
        "total_jobs": row.try_get::<i64, _>("total_jobs").unwrap_or(0),
        "pending_jobs": row.try_get::<i64, _>("pending_jobs").unwrap_or(0),
        "failed_jobs": row.try_get::<i64, _>("failed_jobs").unwrap_or(0),
        "failed_job_ids": failed_job_ids,
        "options": options,
        "cancelled_at": row.try_get::<Option<String>, _>("cancelled_at").ok().flatten(),
        "created_at": row.try_get::<Option<String>, _>("created_at").ok().flatten(),
        "finished_at": row.try_get::<Option<String>, _>("finished_at").ok().flatten(),
    })
}

async fn horizon_zcard(
    state: &AppState,
    key: &str,
) -> Result<i64, Response<Body>> {
    let raw = horizon_command(state, &["ZCARD".to_string(), key.to_string()]).await?;
    Ok(parse_resp_integer(&raw).unwrap_or(0))
}

pub(super) async fn horizon_hmset(
    state: &AppState,
    key: &str,
    fields: &[(String, String)],
) -> Result<(), Response<Body>> {
    let mut command = vec!["HMSET".to_string(), key.to_string()];
    for (field, value) in fields {
        command.push(field.clone());
        command.push(value.clone());
    }
    let raw = horizon_command(state, &command).await?;
    if !raw.starts_with("+OK") {
        return Err(json_error(StatusCode::BAD_GATEWAY, &format!("redis HMSET failed: {}", raw.trim())));
    }
    Ok(())
}

async fn horizon_zrevrange_slice(
    state: &AppState,
    key: &str,
    start: i64,
    end: i64,
) -> Result<Vec<String>, Response<Body>> {
    let raw = horizon_command(
        state,
        &[
            "ZREVRANGE".to_string(),
            key.to_string(),
            start.to_string(),
            end.to_string(),
        ],
    )
    .await?;
    Ok(parse_resp_array_strings(&raw))
}

pub(super) async fn load_horizon_jobs(
    state: &AppState,
    ids: &[String],
    index_from: i64,
) -> Result<Vec<Value>, Response<Body>> {
    let keys = [
        "id",
        "connection",
        "queue",
        "name",
        "status",
        "payload",
        "exception",
        "context",
        "failed_at",
        "completed_at",
        "retried_by",
        "reserved_at",
        "created_at",
        "updated_at",
    ];
    let mut jobs = Vec::new();
    let mut index = index_from;
    for id in ids {
        let values = horizon_hmget(state, id, &keys).await?;
        let Some(job_id) = values.first().cloned().flatten() else {
            continue;
        };
        let payload = values
            .get(5)
            .cloned()
            .flatten()
            .and_then(|raw| serde_json::from_str::<Value>(&raw).ok());
        let context = values
            .get(7)
            .cloned()
            .flatten()
            .and_then(|raw| serde_json::from_str::<Value>(&raw).ok());
        let retried_by = values
            .get(10)
            .cloned()
            .flatten()
            .and_then(|raw| serde_json::from_str::<Vec<HorizonJobRetryRecord>>(&raw).ok())
            .unwrap_or_default();
        let exception = values
            .get(6)
            .cloned()
            .flatten()
            .map(|value| value.to_string());
        let record = HorizonJobRecord {
            id: Some(job_id),
            connection: values.get(1).cloned().flatten(),
            queue: values.get(2).cloned().flatten(),
            name: values.get(3).cloned().flatten(),
            status: values.get(4).cloned().flatten(),
            payload,
            exception,
            context,
            failed_at: values.get(8).cloned().flatten(),
            completed_at: values.get(9).cloned().flatten(),
            retried_by,
            reserved_at: values.get(11).cloned().flatten(),
            created_at: values.get(12).cloned().flatten(),
            updated_at: values.get(13).cloned().flatten(),
            index: Some(index),
        };
        jobs.push(serde_json::to_value(record).unwrap_or(Value::Null));
        index += 1;
    }
    Ok(jobs)
}

async fn horizon_zrange_slice(
    state: &AppState,
    key: &str,
    start: i64,
    end: i64,
) -> Result<Vec<String>, Response<Body>> {
    let raw = horizon_command(
        state,
        &[
            "ZRANGE".to_string(),
            key.to_string(),
            start.to_string(),
            end.to_string(),
        ],
    )
    .await?;
    Ok(parse_resp_array_strings(&raw))
}

async fn horizon_zcount_recent(
    state: &AppState,
    key: &str,
    minutes: i64,
) -> Result<i64, Response<Body>> {
    let min_score = (Utc::now() - chrono::Duration::minutes(minutes)).timestamp() * -1;
    let raw = horizon_command(
        state,
        &[
            "ZCOUNT".to_string(),
            key.to_string(),
            "-inf".to_string(),
            min_score.to_string(),
        ],
    )
    .await?;
    Ok(parse_resp_integer(&raw).unwrap_or(0))
}

fn horizon_minutes_for_job_list(list_key: &str) -> i64 {
    match list_key {
        "failed_jobs" => 10080,
        "recent_failed_jobs" => 10080,
        "pending_jobs" => 60,
        "completed_jobs" => 60,
        "silenced_jobs" => 60,
        _ => 60,
    }
}

async fn horizon_llen(
    state: &AppState,
    key: &str,
) -> Result<i64, Response<Body>> {
    let raw = horizon_command(state, &["LLEN".to_string(), key.to_string()]).await?;
    Ok(parse_resp_integer(&raw).unwrap_or(0))
}

async fn horizon_command(
    state: &AppState,
    parts: &[String],
) -> Result<String, Response<Body>> {
    let addr = format!("{}:{}", state.redis_host, state.redis_port);
    let mut stream = TcpStream::connect(&addr)
        .await
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("connect redis failed: {err}")))?;

    if let Some(password) = &state.redis_password {
        let auth_parts = vec!["AUTH".to_string(), password.clone()];
        let auth = redis_resp_array_strings(&auth_parts);
        stream
            .write_all(auth.as_bytes())
            .await
            .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("redis AUTH write failed: {err}")))?;
        let reply = redis_read_reply(&mut stream)
            .await
            .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &err))?;
        if !reply.starts_with("+OK") {
            return Err(json_error(StatusCode::BAD_GATEWAY, &format!("redis AUTH failed: {}", reply.trim())));
        }
    }

    let select_db = env::var("HORIZON_REDIS_DB")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "0".to_string());
    let select_parts = vec!["SELECT".to_string(), select_db];
    let select = redis_resp_array_strings(&select_parts);
    stream
        .write_all(select.as_bytes())
        .await
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("redis SELECT write failed: {err}")))?;
    let reply = redis_read_reply(&mut stream)
        .await
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &err))?;
    if !reply.starts_with("+OK") {
        return Err(json_error(StatusCode::BAD_GATEWAY, &format!("redis SELECT failed: {}", reply.trim())));
    }

    let prefixed = parts
        .iter()
        .enumerate()
        .map(|(index, value)| {
            if index > 0 && should_prefix_horizon_key(parts.first().map(String::as_str).unwrap_or(""), index) {
                format!("{}{}", horizon_prefix(), value)
            } else {
                value.clone()
            }
        })
        .collect::<Vec<_>>();
    let command = redis_resp_array_strings(&prefixed);
    stream
        .write_all(command.as_bytes())
        .await
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("redis command write failed: {err}")))?;
    redis_read_reply(&mut stream)
        .await
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &err))
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

fn should_prefix_horizon_key(command: &str, index: usize) -> bool {
    match command {
        "HMGET" | "SMEMBERS" | "LLEN" => index == 1,
        "ZREVRANGEBYSCORE" => index == 1,
        _ => false,
    }
}

fn redis_resp_array_strings(parts: &[String]) -> String {
    let mut out = format!("*{}\r\n", parts.len());
    for part in parts {
        out.push_str(&format!("${}\r\n{}\r\n", part.as_bytes().len(), part));
    }
    out
}

fn parse_resp_integer(raw: &str) -> Option<i64> {
    raw.strip_prefix(':')
        .map(str::trim)
        .and_then(|value| value.parse::<i64>().ok())
}

fn parse_resp_bulk_string(raw: &str) -> Option<String> {
    let bytes = raw.as_bytes();
    if bytes.first().copied() != Some(b'$') {
        return None;
    }
    let end = raw.find("\r\n")?;
    let len = raw[1..end].parse::<isize>().ok()?;
    if len < 0 {
        return None;
    }
    let start = end + 2;
    let value_end = start + len as usize;
    if value_end > raw.len() {
        return None;
    }
    Some(raw[start..value_end].to_string())
}

fn parse_resp_array_strings(raw: &str) -> Vec<String> {
    parse_resp_array_option_strings(raw)
        .into_iter()
        .flatten()
        .collect()
}

fn parse_resp_array_option_strings(raw: &str) -> Vec<Option<String>> {
    let bytes = raw.as_bytes();
    if bytes.first().copied() != Some(b'*') {
        return Vec::new();
    }
    let mut index = match raw.find("\r\n") {
        Some(pos) => pos + 2,
        None => return Vec::new(),
    };
    let mut out = Vec::new();
    while index < bytes.len() {
        match bytes[index] {
            b'$' => {
                let end = match raw[index..].find("\r\n") {
                    Some(pos) => index + pos,
                    None => break,
                };
                let len = raw[index + 1..end].parse::<isize>().unwrap_or(-1);
                index = end + 2;
                if len < 0 {
                    out.push(None);
                    continue;
                }
                let len = len as usize;
                if index + len > bytes.len() {
                    break;
                }
                let value = String::from_utf8_lossy(&bytes[index..index + len]).to_string();
                out.push(Some(value));
                index += len + 2;
            }
            b':' | b'+' | b'-' => {
                let end = match raw[index..].find("\r\n") {
                    Some(pos) => index + pos,
                    None => break,
                };
                out.push(Some(raw[index + 1..end].to_string()));
                index = end + 2;
            }
            _ => break,
        }
    }
    out
}
