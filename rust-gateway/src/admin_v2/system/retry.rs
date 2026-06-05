use crate::*;

use super::status;

pub async fn retry_failed_job_payload(
    state: &AppState,
    failed_job_id: &str,
) -> Result<bool, Response<Body>> {
    let jobs = status::load_horizon_jobs(state, &[failed_job_id.to_string()], 0).await?;
    let Some(job) = jobs.into_iter().next() else {
        return Ok(false);
    };
    let Some(object) = job.as_object() else {
        return Ok(false);
    };
    if object.get("status").and_then(Value::as_str) != Some("failed") {
        return Ok(false);
    }

    let payload = object.get("payload").cloned().unwrap_or(Value::Null);
    let payload_object = payload.as_object().cloned().unwrap_or_default();
    let queue = object.get("queue").and_then(Value::as_str).unwrap_or("default");
    let retry_id = Uuid::new_v4().to_string();

    let mut retry_payload = payload_object;
    retry_payload.insert("id".to_string(), Value::String(retry_id.clone()));
    retry_payload.insert("uuid".to_string(), Value::String(retry_id.clone()));
    retry_payload.insert("attempts".to_string(), Value::from(0));
    retry_payload.insert("retry_of".to_string(), Value::String(failed_job_id.to_string()));
    retry_payload.insert(
        "retryUntil".to_string(),
        compute_retry_until(
            retry_payload
                .get("retryUntil")
                .or_else(|| retry_payload.get("timeoutAt")),
            retry_payload.get("pushedAt"),
        ),
    );

    let payload_text = serde_json::to_string(&Value::Object(retry_payload))
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "encode retry payload failed"))?;
    let queue_key = format!("queues:{queue}");
    let notify_key = format!("queues:{queue}:notify");
    redis_rpush_with_notify(state, &queue_key, &notify_key, &payload_text)
        .await
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &err))?;
    append_retry_reference(state, failed_job_id, &retry_id).await?;
    Ok(true)
}

pub async fn retry_batch_payload(
    state: &AppState,
    batch_id: &str,
) -> Result<bool, Response<Body>> {
    let batch = status::load_job_batch_by_id(state, batch_id).await?;
    let Some(batch) = batch else {
        return Ok(false);
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
    if failed_job_ids.is_empty() {
        return Ok(false);
    }

    let failed_jobs = status::load_horizon_jobs(state, &failed_job_ids, 0).await?;
    let mut retried = false;
    for failed_job in failed_jobs {
        let Some(object) = failed_job.as_object() else {
            continue;
        };
        if object
            .get("payload")
            .and_then(Value::as_object)
            .and_then(|payload| payload.get("retry_of"))
            .is_some()
        {
            continue;
        }
        if let Some(id) = object.get("id").and_then(Value::as_str) {
            if retry_failed_job_payload(state, id).await? {
                retried = true;
            }
        }
    }
    Ok(retried)
}

async fn append_retry_reference(
    state: &AppState,
    failed_job_id: &str,
    retry_id: &str,
) -> Result<(), Response<Body>> {
    let values = status::horizon_hmget(state, failed_job_id, &["retried_by"]).await?;
    let mut retries = values
        .first()
        .cloned()
        .flatten()
        .and_then(|raw| serde_json::from_str::<Vec<Value>>(&raw).ok())
        .unwrap_or_default();
    retries.push(json!({
        "id": retry_id,
        "status": "pending",
        "retried_at": Utc::now().timestamp(),
    }));
    let serialized = serde_json::to_string(&retries)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "encode retry reference failed"))?;
    status::horizon_hmset(
        state,
        failed_job_id,
        &[("retried_by".to_string(), serialized)],
    )
    .await
}

fn compute_retry_until(
    retry_until: Option<&Value>,
    pushed_at: Option<&Value>,
) -> Value {
    let retry_until = retry_until.and_then(Value::as_i64);
    let pushed_at = pushed_at
        .and_then(|value| value.as_str().and_then(|item| item.parse::<f64>().ok()))
        .unwrap_or_else(|| (Utc::now().timestamp_millis() as f64) / 1000.0);
    match retry_until {
        Some(until) => {
            let delta = (until as f64) - pushed_at;
            Value::from((Utc::now().timestamp() as f64 + delta.ceil()) as i64)
        }
        None => Value::Null,
    }
}
