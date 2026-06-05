use super::super::*;
use axum::extract::Path;
use http_body_util::BodyExt;

mod models;
pub mod maintenance;
mod retry;
mod serialize;
mod status;
mod store;

#[derive(Deserialize, Default)]
struct DatabaseBackupRunRequest {
    #[serde(default)]
    upload: Option<bool>,
}

pub async fn get_system_status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_system_status_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn monitor_stats(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
) -> Response<Body> {
    match build_monitor_stats_response(&state, headers).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_queue_stats(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_queue_stats_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn monitor_workload(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
) -> Response<Body> {
    match build_monitor_workload_response(&state, headers).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_queue_workload(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
) -> Response<Body> {
    match build_get_queue_workload_response(&state, headers).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn monitor_masters(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
) -> Response<Body> {
    match build_monitor_masters_response(&state, headers).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn monitor_job_metrics(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
) -> Response<Body> {
    match build_monitor_job_metrics_response(&state, headers).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn monitor_job_metric_detail(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    _uri: Uri,
) -> Response<Body> {
    match build_monitor_job_metric_detail_response(&state, headers, job_id).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn monitor_queue_metrics(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
) -> Response<Body> {
    match build_monitor_queue_metrics_response(&state, headers).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn monitor_queue_metric_detail(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(queue_id): Path<String>,
    _uri: Uri,
) -> Response<Body> {
    match build_monitor_queue_metric_detail_response(&state, headers, queue_id).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn monitor_jobs_pending(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_monitor_jobs_list_response(&state, headers, uri, "pending_jobs").await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn monitor_jobs_completed(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_monitor_jobs_list_response(&state, headers, uri, "completed_jobs").await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn monitor_jobs_silenced(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_monitor_jobs_list_response(&state, headers, uri, "silenced_jobs").await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn monitor_jobs_failed(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_monitor_jobs_failed_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn monitor_job_detail(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    _uri: Uri,
) -> Response<Body> {
    match build_monitor_job_detail_response(&state, headers, job_id).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn monitor_failed_job_detail(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    _uri: Uri,
) -> Response<Body> {
    match build_monitor_failed_job_detail_response(&state, headers, job_id).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn monitor_batches(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_monitor_batches_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn monitor_batch_detail(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(batch_id): Path<String>,
    _uri: Uri,
) -> Response<Body> {
    match build_monitor_batch_detail_response(&state, headers, batch_id).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn monitor_monitoring(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
) -> Response<Body> {
    match build_monitor_monitoring_response(&state, headers).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn monitor_monitoring_tag(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(tag): Path<String>,
    uri: Uri,
) -> Response<Body> {
    match build_monitor_monitoring_tag_response(&state, headers, tag, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn monitor_job_retry(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(job_id): Path<String>,
    _uri: Uri,
) -> Response<Body> {
    match build_monitor_job_retry_response(&state, headers, job_id).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn monitor_batch_retry(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(batch_id): Path<String>,
    _uri: Uri,
) -> Response<Body> {
    match build_monitor_batch_retry_response(&state, headers, batch_id).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_queue_masters(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
) -> Response<Body> {
    match build_get_queue_masters_response(&state, headers).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_system_log(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_system_log_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_horizon_failed_jobs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_horizon_failed_jobs_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn clear_system_log(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_clear_system_log_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn run_database_backup(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_run_database_backup_response(&state, headers, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_log_clear_stats(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_log_clear_stats_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_get_system_status_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let now_ts = Utc::now().timestamp();
    let status = build_command_center_system_status(state, now_ts)
        .await
        .map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(json!({
        "schedule": status.get("schedule_ok").cloned().unwrap_or(Value::Bool(false)),
        "horizon": status.get("horizon").cloned().unwrap_or(Value::Object(Map::new())),
        "schedule_last_runtime": status.get("schedule_last_runtime").cloned().unwrap_or(Value::Null),
        "logs": status.get("logs").cloned().unwrap_or(Value::Object(Map::new())),
    }))))
}

async fn build_get_queue_stats_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let stats = build_horizon_queue_stats_payload(state).await?;
    Ok(json_value_response(success_response_payload(stats)))
}

async fn build_monitor_stats_response(
    state: &AppState,
    headers: HeaderMap,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let stats = build_horizon_queue_stats_payload(state).await?;
    Ok(json_value_response(stats))
}

async fn build_horizon_queue_stats_payload(
    state: &AppState,
) -> Result<Value, Response<Body>> {
    let failed_jobs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM failed_jobs")
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    let queue_backlog = status::build_queue_workload_payload(state).await?;
    let wait = queue_backlog
        .as_array()
        .and_then(|items| items.first().cloned())
        .map(|item| Value::Array(vec![item]))
        .unwrap_or_else(|| Value::Array(Vec::new()));
    let processes = status::build_queue_process_count(state).await?;
    let paused_masters = status::build_queue_paused_master_count(state).await?;
    let master_status = status::build_queue_master_status(state).await?;
    let jobs_per_minute = status::build_jobs_processed_per_minute(state).await?;
    let queue_with_max_runtime = status::build_queue_with_max_runtime(state).await?;
    let queue_with_max_throughput = status::build_queue_with_max_throughput(state).await?;
    let recent_jobs = status::build_recent_jobs_count(state).await?;
    let rust_async_queues = serde_json::to_value(state.async_queue_metrics.snapshot())
        .unwrap_or(Value::Null);

    Ok(json!({
        "failedJobs": failed_jobs,
        "jobsPerMinute": jobs_per_minute,
        "pausedMasters": paused_masters,
        "periods": {
            "failedJobs": 10080,
            "recentJobs": 60
        },
        "processes": processes,
        "queueWithMaxRuntime": queue_with_max_runtime,
        "queueWithMaxThroughput": queue_with_max_throughput,
        "recentJobs": recent_jobs,
        "rustAsyncQueues": rust_async_queues,
        "status": master_status,
        "wait": wait,
    }))
}

async fn build_get_queue_workload_response(
    state: &AppState,
    headers: HeaderMap,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let data = status::build_queue_workload_payload(state).await?;
    Ok(json_value_response(success_response_payload(data)))
}

async fn build_monitor_workload_response(
    state: &AppState,
    headers: HeaderMap,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let data = status::build_queue_workload_payload(state).await?;
    Ok(json_value_response(data))
}

async fn build_get_queue_masters_response(
    state: &AppState,
    headers: HeaderMap,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let data = status::build_queue_masters_payload(state).await?;
    Ok(json_value_response(success_response_payload(data)))
}

async fn build_monitor_masters_response(
    state: &AppState,
    headers: HeaderMap,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let data = status::build_queue_masters_payload(state).await?;
    Ok(json_value_response(data))
}

async fn build_monitor_job_metrics_response(
    state: &AppState,
    headers: HeaderMap,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let data = status::build_measured_jobs_payload(state).await?;
    Ok(json_value_response(data))
}

async fn build_monitor_job_metric_detail_response(
    state: &AppState,
    headers: HeaderMap,
    job_id: String,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let data = status::build_job_metric_snapshots_payload(state, &job_id).await?;
    Ok(json_value_response(data))
}

async fn build_monitor_queue_metrics_response(
    state: &AppState,
    headers: HeaderMap,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let data = status::build_measured_queues_payload(state).await?;
    Ok(json_value_response(data))
}

async fn build_monitor_queue_metric_detail_response(
    state: &AppState,
    headers: HeaderMap,
    queue_id: String,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let data = status::build_queue_metric_snapshots_payload(state, &queue_id).await?;
    Ok(json_value_response(data))
}

async fn build_monitor_jobs_list_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
    list_key: &str,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let starting_at = params
        .get("starting_at")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(-1);
    let data = status::build_job_list_payload(state, list_key, starting_at).await?;
    Ok(json_value_response(data))
}

async fn build_monitor_jobs_failed_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let starting_at = params
        .get("starting_at")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(-1);
    let data = status::build_failed_job_list_payload(state, starting_at).await?;
    Ok(json_value_response(data))
}

async fn build_monitor_job_detail_response(
    state: &AppState,
    headers: HeaderMap,
    job_id: String,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let data = status::build_job_detail_payload(state, &job_id).await?;
    Ok(json_value_response(data))
}

async fn build_monitor_failed_job_detail_response(
    state: &AppState,
    headers: HeaderMap,
    job_id: String,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let data = status::build_job_detail_payload(state, &job_id).await?;
    Ok(json_value_response(data))
}

async fn build_monitor_batches_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let query = params.get("query").map(String::as_str);
    let before_id = params.get("before_id").map(String::as_str);
    let data = status::build_batches_payload(state, query, before_id).await?;
    Ok(json_value_response(data))
}

async fn build_monitor_batch_detail_response(
    state: &AppState,
    headers: HeaderMap,
    batch_id: String,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let data = status::build_batch_detail_payload(state, &batch_id).await?;
    Ok(json_value_response(data))
}

async fn build_monitor_monitoring_response(
    state: &AppState,
    headers: HeaderMap,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let data = status::build_monitored_tags_payload(state).await?;
    Ok(json_value_response(data))
}

async fn build_monitor_monitoring_tag_response(
    state: &AppState,
    headers: HeaderMap,
    tag: String,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let starting_at = params
        .get("starting_at")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(0);
    let limit = params
        .get("limit")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(25)
        .clamp(1, 100);
    let data = status::build_monitored_tag_jobs_payload(state, &tag, starting_at, limit).await?;
    Ok(json_value_response(data))
}

async fn build_monitor_job_retry_response(
    state: &AppState,
    headers: HeaderMap,
    job_id: String,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let ok = retry::retry_failed_job_payload(state, &job_id).await?;
    Ok(json_value_response(json!({ "status": ok })))
}

async fn build_monitor_batch_retry_response(
    state: &AppState,
    headers: HeaderMap,
    batch_id: String,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let ok = retry::retry_batch_payload(state, &batch_id).await?;
    Ok(json_value_response(json!({ "status": ok })))
}

async fn build_get_system_log_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let current = params.get("current").and_then(|value| value.parse::<i64>().ok()).unwrap_or(1).max(1);
    let page_size = params.get("page_size").and_then(|value| value.parse::<i64>().ok()).unwrap_or(20).max(10);
    let offset = (current - 1) * page_size;
    let level = params.get("level").map(|value| value.trim().to_uppercase()).filter(|value| !value.is_empty());
    let keyword = params.get("keyword").map(|value| value.trim().to_string()).filter(|value| !value.is_empty());

    let (rows, total) = store::load_system_logs(state, level.as_deref(), keyword.as_deref(), offset, page_size)
        .await
        .map_err(internal_error)?;
    Ok(json_value_response(json!({
        "data": rows.iter().map(serialize::serialize_system_log_row).collect::<Vec<_>>(),
        "total": total
    })))
}

async fn build_get_horizon_failed_jobs_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let current = params.get("current").and_then(|value| value.parse::<i64>().ok()).unwrap_or(1).max(1);
    let page_size = params.get("page_size").and_then(|value| value.parse::<i64>().ok()).unwrap_or(20).max(10);
    let offset = (current - 1) * page_size;

    let (rows, total) = store::load_failed_jobs(state, offset, page_size).await.map_err(internal_error)?;
    Ok(json_value_response(json!({
        "data": rows.iter().map(serialize::serialize_failed_job_row).collect::<Vec<_>>(),
        "total": total,
        "current": current,
        "page_size": page_size
    })))
}

async fn build_run_database_backup_response(
    state: &AppState,
    headers: HeaderMap,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let bytes = body
        .collect()
        .await
        .map_err(|err| json_error(StatusCode::BAD_REQUEST, &format!("read request body failed: {err}")))?
        .to_bytes();
    let request = if bytes.is_empty() {
        DatabaseBackupRunRequest::default()
    } else {
        serde_json::from_slice::<DatabaseBackupRunRequest>(&bytes)
            .map_err(|_| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?
    };
    let upload = request.upload.unwrap_or(false);
    let stats = crate::backup_support::run_database_backup_now(state, upload)
        .await
        .map_err(|err| json_error(StatusCode::INTERNAL_SERVER_ERROR, &err))?;
    Ok(json_value_response(success_response_payload(json!({
        "ran": stats.ran,
        "uploaded": stats.uploaded,
        "retained_local_copy": stats.retained_local_copy,
        "compressed_bytes": stats.compressed_bytes,
        "artifact_path": stats.artifact_path,
        "uploaded_object": stats.uploaded_object,
    }))))
}

async fn build_clear_system_log_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let obj = payload.as_object().ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    let days = obj.get("days").and_then(parse_i64_value).unwrap_or(30);
    if !(0..=365).contains(&days) {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "天数不能超过365天"));
    }
    let level = obj.get("level").and_then(|value| value.as_str()).map(|value| value.trim().to_uppercase()).unwrap_or_else(|| "ALL".to_string());
    if !matches!(level.as_str(), "ALL" | "INFO" | "WARNING" | "ERROR") {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "日志级别只能是：info、warning、error、all"));
    }
    let limit = obj.get("limit").and_then(parse_i64_value).unwrap_or(1000);
    if !(100..=10000).contains(&limit) {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "单次清除数量不能超过10000条"));
    }

    let cutoff = Utc::now().timestamp() - days * 86_400;
    let total_count = store::count_logs_to_clear(state, cutoff, if level == "ALL" { None } else { Some(level.as_str()) })
        .await
        .map_err(internal_error)?;
    if total_count == 0 {
        return Ok(json_value_response(success_response_payload(json!({
            "message": "没有找到符合条件的日志记录",
            "deleted_count": 0,
            "total_count": total_count
        }))));
    }

    let ids = store::load_log_ids_to_clear(state, cutoff, if level == "ALL" { None } else { Some(level.as_str()) }, limit)
        .await
        .map_err(internal_error)?;
    let deleted_count = if ids.is_empty() {
        0
    } else {
        store::delete_log_ids(state, &ids).await.map_err(internal_error)?
    };
    Ok(json_value_response(success_response_payload(json!({
        "message": "日志清除完成",
        "deleted_count": deleted_count,
        "total_count": total_count,
        "remaining_count": (total_count - deleted_count).max(0)
    }))))
}

async fn build_get_log_clear_stats_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let days = params
        .get("days")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(30)
        .clamp(0, 365);
    let level = params
        .get("level")
        .map(|value| value.trim().to_uppercase())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "ALL".to_string());
    if !matches!(level.as_str(), "ALL" | "INFO" | "WARNING" | "ERROR") {
        return Ok(fail_json_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            "日志级别只能是：info、warning、error、all",
        ));
    }

    let cutoff = Utc::now() - chrono::Duration::days(days);
    let cutoff_ts = cutoff.timestamp();
    let total_logs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_log")
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    let logs_to_clear = store::count_logs_to_clear(
        state,
        cutoff_ts,
        if level == "ALL" { None } else { Some(level.as_str()) },
    )
    .await
    .map_err(internal_error)?;
    let oldest_log = store::load_log_edge(state, true)
        .await
        .map_err(internal_error)?
        .as_ref()
        .map(serialize::serialize_system_log_row)
        .unwrap_or(Value::Null);
    let newest_log = store::load_log_edge(state, false)
        .await
        .map_err(internal_error)?
        .as_ref()
        .map(serialize::serialize_system_log_row)
        .unwrap_or(Value::Null);

    Ok(json_value_response(success_response_payload(json!({
        "days": days,
        "level": level.to_ascii_lowercase(),
        "cutoff_date": cutoff.format("%Y-%m-%d %H:%M:%S").to_string(),
        "total_logs": total_logs,
        "logs_to_clear": logs_to_clear,
        "oldest_log": oldest_log,
        "newest_log": newest_log,
    }))))
}
