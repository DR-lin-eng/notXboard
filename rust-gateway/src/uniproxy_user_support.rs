use crate::*;
use crate::uniproxy_support::{
    apply_node_limit_overrides, authenticate_node, build_uniproxy_read_cache_key,
    clamp_snapshot_ttl,
};
use http::HeaderName;
use sqlx::{MySql, QueryBuilder};

#[derive(Clone)]
pub(crate) struct CachedUniProxyUserSnapshot {
    pub(crate) version: String,
    pub(crate) users: Vec<UniProxyUserItem>,
    pub(crate) expires_at: Instant,
}

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct UniProxyAccessibleUserRow {
    pub(crate) id: i64,
    pub(crate) uuid: Option<String>,
    pub(crate) trust_level: Option<i64>,
    pub(crate) is_silenced: Option<i8>,
    pub(crate) subscription_credential_version: Option<i64>,
    pub(crate) effective_speed_limit_down: i64,
    pub(crate) effective_device_limit: i64,
    pub(crate) effective_connection_limit: i64,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct UniProxyUserItem {
    pub(crate) id: i64,
    pub(crate) uuid: String,
    pub(crate) speed_limit: i64,
    pub(crate) device_limit: i64,
    pub(crate) connection_limit: i64,
    pub(crate) trust_level: i64,
    pub(crate) is_silenced: bool,
}

#[derive(Serialize)]
struct UniProxyUserFullPayload {
    users: Vec<UniProxyUserItem>,
}

#[derive(Serialize)]
struct UniProxyUserDeltaPayload {
    mode: &'static str,
    version: String,
    upserts: Vec<UniProxyUserItem>,
    removed_ids: Vec<i64>,
}

pub(crate) async fn build_uniproxy_user_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let node = authenticate_node(state, &uri).await?;
    let response_format = requested_uniproxy_user_format(&headers);
    let snapshot_cache_key = format!(
        "{}:node-credential-v1",
        build_uniproxy_read_cache_key("user", &uri),
    );
    let response_cache_key = format!("{}:{}", snapshot_cache_key, response_format);

    if !request_wants_delta(&headers) {
        if let Some(response) = try_cached_response(state, &response_cache_key, &headers) {
            return Ok(response);
        }
    }

    let requested_snapshot_version = requested_snapshot_version(&headers);
    let must_refresh_current_snapshot =
        request_wants_delta(&headers) && requested_snapshot_version.is_some();
    let user_snapshot = load_or_build_uniproxy_user_snapshot(
        state,
        &node,
        &snapshot_cache_key,
        must_refresh_current_snapshot,
    )
    .await?;

    if requested_snapshot_version.as_deref() == Some(user_snapshot.version.as_str()) {
        return Ok(
            Response::builder()
                .status(StatusCode::NOT_MODIFIED)
                .header(ETAG, etag_for_version(&user_snapshot.version))
                .body(Body::empty())
                .unwrap(),
        );
    }

    let payload_mode = if let Some(previous_version) = requested_snapshot_version.as_deref() {
        if request_wants_delta(&headers) {
            let previous_snapshot =
                load_cached_uniproxy_user_snapshot(state, &snapshot_cache_key, previous_version);
            if let Some(previous_snapshot) = previous_snapshot {
                let delta = build_user_delta_payload(&previous_snapshot.users, &user_snapshot.users, &user_snapshot.version);
                if !delta.upserts.is_empty() || !delta.removed_ids.is_empty() {
                    UserPayloadMode::Delta(delta)
                } else {
                    return Ok(
                        Response::builder()
                            .status(StatusCode::NOT_MODIFIED)
                            .header(ETAG, etag_for_version(&user_snapshot.version))
                            .body(Body::empty())
                            .unwrap(),
                    );
                }
            } else {
                UserPayloadMode::Full(UniProxyUserFullPayload {
                    users: user_snapshot.users.clone(),
                })
            }
        } else {
            let _ = previous_version;
            UserPayloadMode::Full(UniProxyUserFullPayload {
                users: user_snapshot.users.clone(),
            })
        }
    } else {
        UserPayloadMode::Full(UniProxyUserFullPayload {
            users: user_snapshot.users.clone(),
        })
    };

    let response = build_user_payload_response(
        state,
        response_cache_key,
        user_snapshot.version.as_str(),
        &headers,
        payload_mode,
        uniproxy_user_ttl(state).await,
    )?;
    Ok(response)
}

pub(crate) async fn load_accessible_uniproxy_users(
    state: &AppState,
    node: &ServerNodeRow,
    limit: Option<i64>,
) -> Result<Vec<UniProxyAccessibleUserRow>, sqlx::Error> {
    let user_ids = get_accessible_user_ids_for_node(state, node, limit, None).await?;
    load_uniproxy_user_profiles(state, &user_ids).await
}

pub(crate) fn requested_uniproxy_user_format(headers: &HeaderMap) -> &'static str {
    if wants_msgpack(headers) {
        "msgpack"
    } else {
        "json"
    }
}

pub(crate) fn wants_msgpack(headers: &HeaderMap) -> bool {
    headers
        .get("x-response-format")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.eq_ignore_ascii_case("msgpack"))
        .unwrap_or(false)
}

pub(crate) async fn uniproxy_user_ttl(state: &AppState) -> Duration {
    clamp_snapshot_ttl(
        get_setting_int(state, "server_node_user_snapshot_ttl", 15).await,
        15,
        5,
        120,
    )
}

fn request_accepts_gzip(headers: &HeaderMap) -> bool {
    headers
        .get("accept-encoding")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_ascii_lowercase().contains("gzip"))
        .unwrap_or(false)
}

fn request_wants_delta(headers: &HeaderMap) -> bool {
    headers
        .get("x-user-sync-mode")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.eq_ignore_ascii_case("delta"))
        .unwrap_or(false)
}

fn requested_snapshot_version(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-user-snapshot-version")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn gzip_bytes(raw: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    use std::io::Write;

    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(raw)?;
    encoder.finish()
}

async fn load_or_build_uniproxy_user_snapshot(
    state: &AppState,
    node: &ServerNodeRow,
    cache_key: &str,
    force_refresh: bool,
) -> Result<CachedUniProxyUserSnapshot, Response<Body>> {
    if !force_refresh {
        if let Some(snapshot) = load_latest_uniproxy_user_snapshot(state, cache_key) {
            return Ok(snapshot);
        }
    }

    let response_limit = get_setting_int(state, "server_node_user_response_limit", 20_000)
        .await
        .clamp(1, 100_000);
    let rotate_credentials =
        get_setting_bool(state, "rotate_subscription_credentials_daily", false).await;
    let users = load_accessible_uniproxy_users(state, node, Some(response_limit + 1))
        .await
        .map_err(internal_error)?;

    let truncated = users.len() > response_limit as usize;
    let users = if truncated {
        users
            .into_iter()
            .take(response_limit as usize)
            .collect::<Vec<_>>()
    } else {
        users
    };

    let rows = users
        .iter()
        .map(|user| {
            let limit = apply_node_limit_overrides(
                node,
                user.effective_speed_limit_down,
                user.effective_device_limit,
                user.effective_connection_limit,
            );

            UniProxyUserItem {
                id: user.id,
                uuid: node_scoped_uuid(
                    &state.app_key,
                    &effective_uuid(
                        &user.uuid.clone().unwrap_or_default(),
                        user.subscription_credential_version.unwrap_or(0),
                        rotate_credentials,
                    ),
                    node,
                ),
                speed_limit: limit.speed_limit_down,
                device_limit: limit.device_limit,
                connection_limit: limit.connection_limit,
                trust_level: user.trust_level.unwrap_or(0),
                is_silenced: user.is_silenced.unwrap_or(0) != 0,
            }
        })
        .collect::<Vec<_>>();

    if truncated {
        warn!("UniProxy user response truncated for node {}", node.id);
    }

    let version = compute_uniproxy_user_snapshot_version(&rows);
    let snapshot = CachedUniProxyUserSnapshot {
        version: version.clone(),
        users: rows,
        expires_at: Instant::now() + uniproxy_user_ttl(state).await,
    };

    store_uniproxy_user_snapshot(state, cache_key.to_string(), snapshot.clone());

    Ok(snapshot)
}

fn build_user_payload_response(
    state: &AppState,
    cache_key: String,
    version: &str,
    headers: &HeaderMap,
    payload_mode: UserPayloadMode,
    ttl: Duration,
) -> Result<Response<Body>, Response<Body>> {
    let response_format = requested_uniproxy_user_format(headers);
    let accepts_gzip = request_accepts_gzip(headers);
    let is_full = matches!(payload_mode, UserPayloadMode::Full(_));

    let (payload, content_type) = match (response_format, &payload_mode) {
        ("msgpack", UserPayloadMode::Full(payload)) => (
            msgpack::to_vec_named(&payload)
                .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "serialize user payload failed"))?,
            "application/x-msgpack",
        ),
        ("msgpack", UserPayloadMode::Delta(payload)) => (
            msgpack::to_vec_named(&payload)
                .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "serialize user payload failed"))?,
            "application/x-msgpack",
        ),
        (_, UserPayloadMode::Full(payload)) => (
            serde_json::to_vec(&payload)
                .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "serialize user payload failed"))?,
            "application/json",
        ),
        (_, UserPayloadMode::Delta(payload)) => (
            serde_json::to_vec(&payload)
                .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "serialize user payload failed"))?,
            "application/json",
        ),
    };

    let mut extra_headers = vec![
        (
            HeaderName::from_static("etag"),
            HeaderValue::from_str(&etag_for_version(version))
                .unwrap_or_else(|_| HeaderValue::from_static("\"invalid\"")),
        ),
        (
            HeaderName::from_static("x-user-snapshot-version"),
            HeaderValue::from_str(version).unwrap_or_else(|_| HeaderValue::from_static("invalid")),
        ),
    ];

    let body = if accepts_gzip {
        let compressed = gzip_bytes(&payload)
            .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "compress user payload failed"))?;
        extra_headers.push((
            HeaderName::from_static("content-encoding"),
            HeaderValue::from_static("gzip"),
        ));
        bytes::Bytes::from(compressed)
    } else {
        bytes::Bytes::from(payload)
    };

    match is_full {
        true => Ok(cached_plain_response_with_headers(
            state,
            cache_key,
            body,
            ttl,
            content_type,
            extra_headers,
        )),
        false => {
            let mut response = Response::builder().status(StatusCode::OK);
            response = response.header(CONTENT_TYPE, content_type);
            for (name, value) in extra_headers {
                response = response.header(name, value);
            }
            Ok(response.body(Body::from(body)).unwrap())
        }
    }
}

fn build_user_delta_payload(
    previous: &[UniProxyUserItem],
    current: &[UniProxyUserItem],
    version: &str,
) -> UniProxyUserDeltaPayload {
    let previous_map = previous
        .iter()
        .cloned()
        .map(|user| (user.id, user))
        .collect::<HashMap<_, _>>();
    let current_map = current
        .iter()
        .cloned()
        .map(|user| (user.id, user))
        .collect::<HashMap<_, _>>();

    let mut upserts = current
        .iter()
        .filter(|user| previous_map.get(&user.id) != Some(*user))
        .cloned()
        .collect::<Vec<_>>();
    upserts.sort_unstable_by_key(|user| user.id);

    let mut removed_ids = previous_map
        .keys()
        .filter(|id| !current_map.contains_key(id))
        .copied()
        .collect::<Vec<_>>();
    removed_ids.sort_unstable();

    UniProxyUserDeltaPayload {
        mode: "delta",
        version: version.to_string(),
        upserts,
        removed_ids,
    }
}

fn compute_uniproxy_user_snapshot_version(users: &[UniProxyUserItem]) -> String {
    let payload = serde_json::to_vec(users).unwrap_or_default();
    format!("{:x}", md5::compute(payload))
}

fn etag_for_version(version: &str) -> String {
    format!("\"{version}\"")
}

fn load_cached_uniproxy_user_snapshot(
    state: &AppState,
    cache_key: &str,
    version: &str,
) -> Option<CachedUniProxyUserSnapshot> {
    let versioned_key = snapshot_version_cache_key(cache_key, version);
    {
        let cache = state.uniproxy_user_snapshot_cache.read();
        let snapshot = cache.get(&versioned_key)?;
        if snapshot.expires_at > Instant::now() && snapshot.version == version {
            return Some(snapshot.clone());
        }
    }

    let mut cache = state.uniproxy_user_snapshot_cache.write();
    if let Some(snapshot) = cache.get(&versioned_key) {
        if snapshot.expires_at <= Instant::now() || snapshot.version != version {
            cache.remove(&versioned_key);
        }
    }
    None
}

fn load_latest_uniproxy_user_snapshot(
    state: &AppState,
    cache_key: &str,
) -> Option<CachedUniProxyUserSnapshot> {
    let latest_key = snapshot_latest_cache_key(cache_key);
    {
        let cache = state.uniproxy_user_snapshot_cache.read();
        let snapshot = cache.get(&latest_key)?;
        if snapshot.expires_at > Instant::now() {
            return Some(snapshot.clone());
        }
    }

    let mut cache = state.uniproxy_user_snapshot_cache.write();
    if let Some(snapshot) = cache.get(&latest_key) {
        if snapshot.expires_at <= Instant::now() {
            cache.remove(&latest_key);
        }
    }
    None
}

fn store_uniproxy_user_snapshot(
    state: &AppState,
    cache_key: String,
    snapshot: CachedUniProxyUserSnapshot,
) {
    let mut cache = state.uniproxy_user_snapshot_cache.write();
    if cache.len() >= 4096 {
        let now = Instant::now();
        cache.retain(|_, value| value.expires_at > now);
        if cache.len() >= 8192 {
            cache.clear();
        }
    }
    cache.insert(snapshot_latest_cache_key(&cache_key), snapshot.clone());
    cache.insert(snapshot_version_cache_key(&cache_key, &snapshot.version), snapshot);
}

fn snapshot_latest_cache_key(cache_key: &str) -> String {
    format!("{cache_key}:latest")
}

fn snapshot_version_cache_key(cache_key: &str, version: &str) -> String {
    format!("{cache_key}:version:{version}")
}

async fn load_uniproxy_user_profiles(
    state: &AppState,
    user_ids: &[i64],
) -> Result<Vec<UniProxyAccessibleUserRow>, sqlx::Error> {
    if user_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut builder = QueryBuilder::<MySql>::new(
        "SELECT
            u.id,
            u.uuid,
            u.trust_level,
            u.is_silenced,
            u.subscription_credential_version,
            COALESCE(NULLIF(uil.speed_limit_down, 0), NULLIF(ugl.speed_limit_down, 0), 0) AS effective_speed_limit_down,
            COALESCE(NULLIF(uil.device_limit, 0), NULLIF(ugl.device_limit, 0), 2) AS effective_device_limit,
            COALESCE(NULLIF(uil.connection_limit, 0), NULLIF(ugl.connection_limit, 0), 10) AS effective_connection_limit
         FROM v2_user u
         LEFT JOIN user_individual_limits uil ON uil.user_id = u.id
         LEFT JOIN user_group_limits ugl ON ugl.trust_level = COALESCE(u.trust_level, 0)
         WHERE u.id IN (",
    );

    {
        let mut separated = builder.separated(", ");
        for user_id in user_ids {
            separated.push_bind(user_id);
        }
    }
    builder.push(") ORDER BY u.id");

    builder
        .build_query_as::<UniProxyAccessibleUserRow>()
        .fetch_all(&state.db)
        .await
}

enum UserPayloadMode {
    Full(UniProxyUserFullPayload),
    Delta(UniProxyUserDeltaPayload),
}
