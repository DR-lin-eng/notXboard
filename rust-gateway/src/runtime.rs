use crate::{
    AsyncQueueMetrics, AppState, ExposureConfig, legacy_traffic_support::QueuedLegacySubmitJob, slug_for_prefix,
    uniproxy_support::{QueuedAliveSessionJob, QueuedPushTrafficJob},
};
use hyper_util::{
    client::legacy::Client,
    rt::TokioExecutor,
};
use parking_lot::RwLock;
use sqlx::mysql::MySqlPoolOptions;
use std::{
    collections::HashMap,
    env,
    time::Duration,
};
use tokio::sync::mpsc;
use tracing::warn;

pub(crate) struct RuntimeConfig {
    pub(crate) alive_session_rx: mpsc::Receiver<QueuedAliveSessionJob>,
    pub(crate) legacy_submit_rx: mpsc::Receiver<QueuedLegacySubmitJob>,
    pub(crate) port: u16,
    pub(crate) push_traffic_rx: mpsc::Receiver<QueuedPushTrafficJob>,
    pub(crate) state: AppState,
}

pub(crate) async fn build_runtime_config() -> RuntimeConfig {
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8000);

    let db_host = env::var("DB_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let db_port = env::var("DB_PORT").unwrap_or_else(|_| "3306".to_string());
    let db_name = env::var("DB_DATABASE").unwrap_or_else(|_| "xboard".to_string());
    let db_user = env::var("DB_USERNAME").unwrap_or_else(|_| "xboard".to_string());
    let db_pass = env::var("DB_PASSWORD").unwrap_or_default();
    let database_url = format!("mysql://{}:{}@{}:{}/{}", db_user, db_pass, db_host, db_port, db_name);
    let db_max_connections = env::var("DB_MAX_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .map(|v| v.clamp(16, 512))
        .unwrap_or(128);
    let db_min_connections = env::var("DB_MIN_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .map(|v| v.clamp(1, db_max_connections))
        .unwrap_or(16)
        .min(db_max_connections);
    let db_acquire_timeout_secs = env::var("DB_ACQUIRE_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(|v| v.clamp(1, 60))
        .unwrap_or(5);
    let db_idle_timeout_secs = env::var("DB_IDLE_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(|v| v.clamp(30, 3600))
        .unwrap_or(300);
    let db = MySqlPoolOptions::new()
        .max_connections(db_max_connections)
        .min_connections(db_min_connections)
        .acquire_timeout(Duration::from_secs(db_acquire_timeout_secs))
        .idle_timeout(Duration::from_secs(db_idle_timeout_secs))
        .connect(&database_url)
        .await
        .expect("connect mysql");

    let app_key = env::var("APP_KEY").unwrap_or_default();
    let redis_host = env::var("REDIS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let redis_port: u16 = env::var("REDIS_PORT").ok().and_then(|v| v.parse().ok()).unwrap_or(6379);
    let redis_password = env::var("REDIS_PASSWORD").ok().and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("null") {
            None
        } else {
            Some(trimmed)
        }
    });
    let redis_cache_db: i64 = env::var("REDIS_CACHE_DB").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
    let redis_prefix = env::var("REDIS_PREFIX").unwrap_or_else(|_| {
        let app_name = env::var("APP_NAME").unwrap_or_else(|_| "laravel".to_string());
        format!("{}_database_", slug_for_prefix(&app_name))
    });
    let cache_prefix = env::var("CACHE_PREFIX").unwrap_or_else(|_| {
        let app_name = env::var("APP_NAME").unwrap_or_else(|_| "laravel".to_string());
        format!("{}_cache", slug_for_prefix(&app_name))
    });
    let exposure = ExposureConfig::from_env()
        .unwrap_or_else(|error| panic!("invalid public exposure configuration: {error}"));
    if env::var("APP_ENV")
        .map(|value| value.eq_ignore_ascii_case("production"))
        .unwrap_or(false)
        && !exposure.web_access_enabled()
    {
        warn!(
            "WEB_ACCESS_USERNAME/WEB_ACCESS_PASSWORD are not configured; browser pages remain publicly reachable"
        );
    }
    let push_traffic_queue_capacity = env::var("PUSH_TRAFFIC_QUEUE_CAPACITY")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .map(|value| value.clamp(256, 200_000))
        .unwrap_or(20_000);
    let alive_session_queue_capacity = env::var("ALIVE_SESSION_QUEUE_CAPACITY")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .map(|value| value.clamp(256, 200_000))
        .unwrap_or(20_000);
    let legacy_submit_queue_capacity = env::var("LEGACY_SUBMIT_QUEUE_CAPACITY")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .map(|value| value.clamp(256, 200_000))
        .unwrap_or(20_000);
    let (alive_session_tx, alive_session_rx) = mpsc::channel(alive_session_queue_capacity);
    let (legacy_submit_tx, legacy_submit_rx) = mpsc::channel(legacy_submit_queue_capacity);
    let (push_traffic_tx, push_traffic_rx) = mpsc::channel(push_traffic_queue_capacity);
    let async_queue_metrics = std::sync::Arc::new(AsyncQueueMetrics::new(
        push_traffic_queue_capacity,
        alive_session_queue_capacity,
        legacy_submit_queue_capacity,
    ));

    let state = AppState {
        backend_client: Client::builder(TokioExecutor::new()).build_http(),
        alive_session_tx,
        legacy_submit_tx,
        push_traffic_tx,
        response_cache: std::sync::Arc::new(RwLock::new(HashMap::new())),
        uniproxy_user_snapshot_cache: std::sync::Arc::new(RwLock::new(HashMap::new())),
        legacy_server_availability_cache: std::sync::Arc::new(RwLock::new(HashMap::new())),
        settings_cache: std::sync::Arc::new(RwLock::new(HashMap::new())),
        node_accessible_user_ids_cache: std::sync::Arc::new(RwLock::new(HashMap::new())),
        load_status_cache: std::sync::Arc::new(RwLock::new(HashMap::new())),
        public_geo_cache: std::sync::Arc::new(RwLock::new(HashMap::new())),
        traffic_snapshot: std::sync::Arc::new(RwLock::new(None)),
        async_queue_metrics,
        db,
        app_key,
        redis_host,
        redis_port,
        redis_password,
        redis_cache_db,
        redis_prefix,
        cache_prefix,
        exposure,
    };

    RuntimeConfig {
        alive_session_rx,
        legacy_submit_rx,
        port,
        push_traffic_rx,
        state,
    }
}
