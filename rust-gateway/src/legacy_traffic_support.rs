use crate::*;
use crate::db_retry_support::retry_db_write;

const MAX_LEGACY_TRAFFIC_DELTA_KB: i64 = 100_000_000;
const MAX_LEGACY_TRAFFIC_BATCH_KB: i64 = 1_000_000_000;

#[derive(Clone)]
pub(crate) struct QueuedLegacySubmitJob {
    pub(crate) server_id: u64,
    pub(crate) server_type: String,
    pub(crate) rate_basis_points: i64,
    pub(crate) record_at: i64,
    pub(crate) rows: Vec<QueuedLegacyTrafficRow>,
}

#[derive(Clone)]
pub(crate) struct QueuedLegacyTrafficRow {
    pub(crate) user_id: i64,
    pub(crate) billed_upload: i64,
    pub(crate) billed_download: i64,
}

pub(crate) fn spawn_legacy_submit_worker(
    state: AppState,
    rx: tokio::sync::mpsc::Receiver<QueuedLegacySubmitJob>,
) {
    tokio::spawn(async move {
        run_legacy_submit_worker(state, rx).await;
    });
}

pub(crate) async fn enqueue_legacy_submit_traffic(
    state: &AppState,
    server_id: u64,
    server_type: &str,
    rate_text: &str,
    rows: &[TrafficRow],
) -> Result<(), sqlx::Error> {
    if rows.is_empty() {
        return Ok(());
    }

    let rate_basis_points = parse_rate_basis_points(rate_text);
    let rate = rate_basis_points as f64 / 100.0;
    let record_at = chrono::Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp();
    let queued_rows = rows
        .iter()
        .filter_map(|row| {
            let upload = row.upload.max(0);
            let download = row.download.max(0);
            if upload
                .checked_add(download)
                .is_none_or(|total| total <= 0 || total > MAX_LEGACY_TRAFFIC_BATCH_KB)
            {
                return None;
            }
            Some(QueuedLegacyTrafficRow {
                user_id: row.user_id,
                billed_upload: scaled_legacy_traffic(upload, rate),
                billed_download: scaled_legacy_traffic(download, rate),
            })
        })
        .collect::<Vec<_>>();

    if queued_rows.is_empty() {
        return Ok(());
    }

    let queued_item_count = queued_rows.len() as u64;
    let job = QueuedLegacySubmitJob {
        server_id,
        server_type: server_type.to_string(),
        rate_basis_points,
        record_at,
        rows: queued_rows,
    };

    match state.legacy_submit_tx.send(job).await {
        Ok(()) => {
            state
                .async_queue_metrics
                .record_legacy_enqueue(queued_item_count);
            Ok(())
        }
        Err(err) => {
            state.async_queue_metrics.record_legacy_fallback_sync();
            warn!(
                server_id = server_id,
                row_count = queued_item_count,
                "legacy submit worker unavailable, falling back to synchronous write",
            );
            let merged_rows = merge_legacy_submit_rows(&err.0.rows);
            write_merged_legacy_submit_traffic(
                state,
                err.0.server_id,
                &err.0.server_type,
                err.0.rate_basis_points,
                err.0.record_at,
                &merged_rows,
            )
            .await
        }
    }
}

async fn run_legacy_submit_worker(
    state: AppState,
    mut rx: tokio::sync::mpsc::Receiver<QueuedLegacySubmitJob>,
) {
    let flush_interval = legacy_submit_flush_interval();
    let max_batch_jobs = legacy_submit_max_batch_jobs();
    let max_batch_rows = legacy_submit_max_batch_rows();

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

        if let Err(err) = flush_legacy_submit_jobs(&state, &jobs).await {
            state.async_queue_metrics.record_legacy_flush_failure();
            error!(
                job_count = jobs.len(),
                row_count = row_count,
                "legacy submit flush failed: {}",
                err
            );
        }

        if channel_closed {
            break;
        }
    }
}

async fn flush_legacy_submit_jobs(
    state: &AppState,
    jobs: &[QueuedLegacySubmitJob],
) -> Result<(), sqlx::Error> {
    let total_rows = jobs.iter().map(|job| job.rows.len() as u64).sum::<u64>();
    let mut grouped = std::collections::BTreeMap::<(u64, String, i64, i64), std::collections::BTreeMap<i64, (i64, i64)>>::new();

    for job in jobs {
        let node_rows = grouped
            .entry((
                job.server_id,
                job.server_type.clone(),
                job.rate_basis_points,
                job.record_at,
            ))
            .or_default();
        for row in &job.rows {
            let entry = node_rows.entry(row.user_id).or_insert((0, 0));
            entry.0 = entry
                .0
                .saturating_add(row.billed_upload.clamp(0, MAX_LEGACY_TRAFFIC_DELTA_KB))
                .min(MAX_LEGACY_TRAFFIC_BATCH_KB);
            entry.1 = entry
                .1
                .saturating_add(row.billed_download.clamp(0, MAX_LEGACY_TRAFFIC_DELTA_KB))
                .min(MAX_LEGACY_TRAFFIC_BATCH_KB);
        }
    }

    let mut first_error = None;
    for ((server_id, server_type, rate_basis_points, record_at), merged) in grouped {
        let merged_rows = merged
            .into_iter()
            .map(|(user_id, (upload, download))| (user_id, upload, download, 0))
            .collect::<Vec<AggregatedTrafficRow>>();
        if let Err(err) = write_merged_legacy_submit_traffic(
            state,
            server_id,
            &server_type,
            rate_basis_points,
            record_at,
            &merged_rows,
        )
        .await {
            error!(server_id, error = %err, "legacy traffic write failed for one server");
            if first_error.is_none() {
                first_error = Some(err);
            }
        }
    }

    state
        .async_queue_metrics
        .record_legacy_flush(jobs.len() as u64, total_rows);
    first_error.map_or(Ok(()), Err)
}

fn merge_legacy_submit_rows(rows: &[QueuedLegacyTrafficRow]) -> Vec<AggregatedTrafficRow> {
    let mut merged = std::collections::BTreeMap::<i64, (i64, i64)>::new();
    for row in rows {
        let entry = merged.entry(row.user_id).or_insert((0, 0));
        entry.0 = entry
            .0
            .saturating_add(row.billed_upload.clamp(0, MAX_LEGACY_TRAFFIC_DELTA_KB))
            .min(MAX_LEGACY_TRAFFIC_BATCH_KB);
        entry.1 = entry
            .1
            .saturating_add(row.billed_download.clamp(0, MAX_LEGACY_TRAFFIC_DELTA_KB))
            .min(MAX_LEGACY_TRAFFIC_BATCH_KB);
    }
    merged
        .into_iter()
        .map(|(user_id, (upload, download))| (user_id, upload, download, 0))
        .collect()
}

async fn write_merged_legacy_submit_traffic(
    state: &AppState,
    server_id: u64,
    server_type: &str,
    rate_basis_points: i64,
    record_at: i64,
    merged_rows: &[AggregatedTrafficRow],
) -> Result<(), sqlx::Error> {
    if merged_rows.is_empty() {
        return Ok(());
    }

    let now_ts = Utc::now().timestamp();
    let server_rate = rate_basis_points as f64 / 100.0;
    retry_db_write("legacy_server_submit_write", || async {
        let mut tx = state.db.begin().await?;
        batch_update_user_traffic_totals(&mut tx, now_ts, merged_rows).await?;
        batch_upsert_legacy_stat_user(&mut tx, server_rate, record_at, now_ts, merged_rows).await?;

        let total_u = merged_rows
            .iter()
            .try_fold(0_i64, |total, (_, upload, _, _)| total.checked_add(*upload))
            .ok_or_else(|| sqlx::Error::Protocol("legacy upload total overflow".to_string()))?;
        let total_d = merged_rows
            .iter()
            .try_fold(0_i64, |total, (_, _, download, _)| total.checked_add(*download))
            .ok_or_else(|| sqlx::Error::Protocol("legacy download total overflow".to_string()))?;
        sqlx::query(
            "INSERT INTO v2_stat_server (record_at, server_id, server_type, record_type, u, d, created_at, updated_at)
             VALUES (?, ?, ?, 'd', ?, ?, ?, ?)
             ON DUPLICATE KEY UPDATE
               u = u + VALUES(u),
               d = d + VALUES(d),
               updated_at = VALUES(updated_at)",
        )
        .bind(record_at)
        .bind(server_id as i64)
        .bind(server_type)
        .bind(total_u)
        .bind(total_d)
        .bind(now_ts)
        .bind(now_ts)
        .execute(&mut *tx)
        .await?;

        tx.commit().await
    })
    .await
}

fn parse_rate_basis_points(rate_text: &str) -> i64 {
    let rate = rate_text.trim().parse::<f64>().unwrap_or(1.0);
    (rate * 100.0).round() as i64
}

fn scaled_legacy_traffic(value: i64, rate: f64) -> i64 {
    ((value as f64) * rate).round() as i64
}

fn legacy_submit_flush_interval() -> Duration {
    let millis = std::env::var("LEGACY_SUBMIT_FLUSH_INTERVAL_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(|value| value.clamp(10, 1000))
        .unwrap_or(50);
    Duration::from_millis(millis)
}

fn legacy_submit_max_batch_jobs() -> usize {
    std::env::var("LEGACY_SUBMIT_MAX_BATCH_JOBS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .map(|value| value.clamp(1, 10_000))
        .unwrap_or(512)
}

fn legacy_submit_max_batch_rows() -> usize {
    std::env::var("LEGACY_SUBMIT_MAX_BATCH_ROWS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .map(|value| value.clamp(100, 200_000))
        .unwrap_or(20_000)
}
