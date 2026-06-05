use crate::{
    legacy_traffic_support::QueuedLegacySubmitJob,
    uniproxy_support::{QueuedAliveSessionJob, QueuedPushTrafficJob},
    uniproxy_user_support::CachedUniProxyUserSnapshot,
};
use axum::{
    body::Body,
    http::{HeaderName, HeaderValue, StatusCode},
};
use hyper_util::client::legacy::{connect::HttpConnector, Client};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::MySqlPool;
use std::{
    collections::HashMap,
    sync::Arc,
    sync::atomic::{AtomicI64, AtomicU64, Ordering},
    time::Instant,
};
use tokio::sync::mpsc;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) backend_client: Client<HttpConnector, Body>,
    pub(crate) alive_session_tx: mpsc::Sender<QueuedAliveSessionJob>,
    pub(crate) legacy_submit_tx: mpsc::Sender<QueuedLegacySubmitJob>,
    pub(crate) push_traffic_tx: mpsc::Sender<QueuedPushTrafficJob>,
    pub(crate) response_cache: Arc<RwLock<HashMap<String, CachedResponse>>>,
    pub(crate) uniproxy_user_snapshot_cache: Arc<RwLock<HashMap<String, CachedUniProxyUserSnapshot>>>,
    pub(crate) legacy_server_availability_cache: Arc<RwLock<HashMap<String, CachedLegacyAvailability>>>,
    pub(crate) settings_cache: Arc<RwLock<HashMap<String, CachedSetting>>>,
    pub(crate) node_accessible_user_ids_cache: Arc<RwLock<HashMap<u64, CachedIdList>>>,
    pub(crate) load_status_cache: Arc<RwLock<HashMap<u64, Value>>>,
    pub(crate) public_geo_cache: Arc<RwLock<HashMap<String, CountryHit>>>,
    pub(crate) traffic_snapshot: Arc<RwLock<Option<TrafficSnapshot>>>,
    pub(crate) async_queue_metrics: Arc<AsyncQueueMetrics>,
    pub(crate) db: MySqlPool,
    pub(crate) app_key: String,
    pub(crate) redis_host: String,
    pub(crate) redis_port: u16,
    pub(crate) redis_password: Option<String>,
    pub(crate) redis_cache_db: i64,
    pub(crate) redis_prefix: String,
    pub(crate) cache_prefix: String,
}

#[derive(Clone)]
pub(crate) struct CachedResponse {
    pub(crate) status: StatusCode,
    pub(crate) headers: Vec<(HeaderName, HeaderValue)>,
    pub(crate) body: bytes::Bytes,
    pub(crate) expires_at: Instant,
}

#[derive(Clone)]
pub(crate) struct CachedSetting {
    pub(crate) value: Option<String>,
    pub(crate) expires_at: Instant,
}

#[derive(Clone)]
pub(crate) struct CachedLegacyAvailability {
    pub(crate) available: bool,
    pub(crate) expires_at: Instant,
}

#[derive(Clone)]
pub(crate) struct CachedIdList {
    pub(crate) values: Vec<i64>,
    pub(crate) expires_at: Instant,
}

#[derive(Clone)]
pub(crate) struct CountryHit {
    pub(crate) country: String,
    pub(crate) expires_at: Instant,
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub(crate) struct TrafficSnapshot {
    pub(crate) ts: i64,
    pub(crate) u: i64,
    pub(crate) d: i64,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct AsyncQueueSnapshot {
    pub(crate) push_traffic: AsyncQueueLaneSnapshot,
    pub(crate) alive_session: AsyncQueueLaneSnapshot,
    pub(crate) legacy_submit: AsyncQueueLaneSnapshot,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct AsyncQueueLaneSnapshot {
    pub(crate) capacity: u64,
    pub(crate) queued_jobs: i64,
    pub(crate) queued_items: i64,
    pub(crate) enqueued_jobs_total: u64,
    pub(crate) enqueued_items_total: u64,
    pub(crate) flushed_jobs_total: u64,
    pub(crate) flushed_items_total: u64,
    pub(crate) flush_failures_total: u64,
    pub(crate) fallback_sync_total: u64,
    pub(crate) last_flush_at: i64,
}

pub(crate) struct AsyncQueueMetrics {
    push_traffic: AsyncQueueLaneMetrics,
    alive_session: AsyncQueueLaneMetrics,
    legacy_submit: AsyncQueueLaneMetrics,
}

struct AsyncQueueLaneMetrics {
    capacity: u64,
    queued_jobs: AtomicI64,
    queued_items: AtomicI64,
    enqueued_jobs_total: AtomicU64,
    enqueued_items_total: AtomicU64,
    flushed_jobs_total: AtomicU64,
    flushed_items_total: AtomicU64,
    flush_failures_total: AtomicU64,
    fallback_sync_total: AtomicU64,
    last_flush_at: AtomicI64,
}

impl AsyncQueueMetrics {
    pub(crate) fn new(
        push_traffic_capacity: usize,
        alive_session_capacity: usize,
        legacy_submit_capacity: usize,
    ) -> Self {
        Self {
            push_traffic: AsyncQueueLaneMetrics::new(push_traffic_capacity as u64),
            alive_session: AsyncQueueLaneMetrics::new(alive_session_capacity as u64),
            legacy_submit: AsyncQueueLaneMetrics::new(legacy_submit_capacity as u64),
        }
    }

    pub(crate) fn snapshot(&self) -> AsyncQueueSnapshot {
        AsyncQueueSnapshot {
            push_traffic: self.push_traffic.snapshot(),
            alive_session: self.alive_session.snapshot(),
            legacy_submit: self.legacy_submit.snapshot(),
        }
    }

    pub(crate) fn record_push_enqueue(&self, item_count: u64) {
        self.push_traffic.record_enqueue(item_count);
    }

    pub(crate) fn record_push_flush(&self, job_count: u64, item_count: u64) {
        self.push_traffic.record_flush(job_count, item_count);
    }

    pub(crate) fn record_push_flush_failure(&self) {
        self.push_traffic.record_flush_failure();
    }

    pub(crate) fn record_push_fallback_sync(&self) {
        self.push_traffic.record_fallback_sync();
    }

    pub(crate) fn record_alive_enqueue(&self, item_count: u64) {
        self.alive_session.record_enqueue(item_count);
    }

    pub(crate) fn record_alive_flush(&self, job_count: u64, item_count: u64) {
        self.alive_session.record_flush(job_count, item_count);
    }

    pub(crate) fn record_alive_flush_failure(&self) {
        self.alive_session.record_flush_failure();
    }

    pub(crate) fn record_alive_fallback_sync(&self) {
        self.alive_session.record_fallback_sync();
    }

    pub(crate) fn record_legacy_enqueue(&self, item_count: u64) {
        self.legacy_submit.record_enqueue(item_count);
    }

    pub(crate) fn record_legacy_flush(&self, job_count: u64, item_count: u64) {
        self.legacy_submit.record_flush(job_count, item_count);
    }

    pub(crate) fn record_legacy_flush_failure(&self) {
        self.legacy_submit.record_flush_failure();
    }

    pub(crate) fn record_legacy_fallback_sync(&self) {
        self.legacy_submit.record_fallback_sync();
    }
}

impl AsyncQueueLaneMetrics {
    fn new(capacity: u64) -> Self {
        Self {
            capacity,
            queued_jobs: AtomicI64::new(0),
            queued_items: AtomicI64::new(0),
            enqueued_jobs_total: AtomicU64::new(0),
            enqueued_items_total: AtomicU64::new(0),
            flushed_jobs_total: AtomicU64::new(0),
            flushed_items_total: AtomicU64::new(0),
            flush_failures_total: AtomicU64::new(0),
            fallback_sync_total: AtomicU64::new(0),
            last_flush_at: AtomicI64::new(0),
        }
    }

    fn snapshot(&self) -> AsyncQueueLaneSnapshot {
        AsyncQueueLaneSnapshot {
            capacity: self.capacity,
            queued_jobs: self.queued_jobs.load(Ordering::Relaxed),
            queued_items: self.queued_items.load(Ordering::Relaxed),
            enqueued_jobs_total: self.enqueued_jobs_total.load(Ordering::Relaxed),
            enqueued_items_total: self.enqueued_items_total.load(Ordering::Relaxed),
            flushed_jobs_total: self.flushed_jobs_total.load(Ordering::Relaxed),
            flushed_items_total: self.flushed_items_total.load(Ordering::Relaxed),
            flush_failures_total: self.flush_failures_total.load(Ordering::Relaxed),
            fallback_sync_total: self.fallback_sync_total.load(Ordering::Relaxed),
            last_flush_at: self.last_flush_at.load(Ordering::Relaxed),
        }
    }

    fn record_enqueue(&self, item_count: u64) {
        self.queued_jobs.fetch_add(1, Ordering::Relaxed);
        self.queued_items
            .fetch_add(item_count.min(i64::MAX as u64) as i64, Ordering::Relaxed);
        self.enqueued_jobs_total.fetch_add(1, Ordering::Relaxed);
        self.enqueued_items_total.fetch_add(item_count, Ordering::Relaxed);
    }

    fn record_flush(&self, job_count: u64, item_count: u64) {
        self.queued_jobs
            .fetch_sub(job_count.min(i64::MAX as u64) as i64, Ordering::Relaxed);
        self.queued_items
            .fetch_sub(item_count.min(i64::MAX as u64) as i64, Ordering::Relaxed);
        self.flushed_jobs_total.fetch_add(job_count, Ordering::Relaxed);
        self.flushed_items_total.fetch_add(item_count, Ordering::Relaxed);
        self.last_flush_at
            .store(chrono::Utc::now().timestamp(), Ordering::Relaxed);
    }

    fn record_flush_failure(&self) {
        self.flush_failures_total.fetch_add(1, Ordering::Relaxed);
    }

    fn record_fallback_sync(&self) {
        self.fallback_sync_total.fetch_add(1, Ordering::Relaxed);
    }
}
