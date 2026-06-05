use axum::{
    body::Body,
    http::{HeaderMap, HeaderName, HeaderValue, Response, StatusCode, Uri},
};
use http::header::{CONTENT_TYPE, ETAG};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tracing::error;

use crate::{AppState, CachedResponse};

pub(crate) async fn parse_json_body(body: Body) -> Result<Value, Response<Body>> {
    let bytes = body
        .collect()
        .await
        .map_err(|err| json_error(StatusCode::BAD_REQUEST, &format!("read body failed: {err}")))?
        .to_bytes();

    serde_json::from_slice::<Value>(&bytes)
        .map_err(|_| json_error(StatusCode::BAD_REQUEST, "Invalid JSON body"))
}

pub(crate) fn json_value_response(value: Value) -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(value.to_string()))
        .unwrap()
}

pub(crate) fn json_status_response(status: StatusCode, value: Value) -> Response<Body> {
    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(value.to_string()))
        .unwrap()
}

pub(crate) fn build_cache_key(uri: &Uri) -> String {
    format!("{}?{}", uri.path(), uri.query().unwrap_or_default())
}

pub(crate) fn success_response_payload(data: Value) -> Value {
    json!({
        "status": "success",
        "message": "操作成功",
        "data": data,
        "error": Value::Null
    })
}

pub(crate) fn fail_response_payload(message: &str, data: Value) -> Value {
    json!({
        "status": "fail",
        "message": message,
        "data": data,
        "error": Value::Null
    })
}

pub(crate) fn json_cached_response(
    state: &AppState,
    cache_key: String,
    payload: Value,
    ttl: Duration,
) -> Response<Body> {
    let payload_string = payload.to_string();
    let etag_value = format!("\"{:x}\"", md5::compute(payload_string.as_bytes()));
    let bytes = bytes::Bytes::from(payload_string);
    let headers = vec![
        (
            HeaderName::from_static("content-type"),
            HeaderValue::from_static("application/json"),
        ),
        (
            HeaderName::from_static("etag"),
            HeaderValue::from_str(&etag_value)
                .unwrap_or_else(|_| HeaderValue::from_static("\"invalid\"")),
        ),
    ];

    state.response_cache.write().insert(
        cache_key,
        CachedResponse {
            status: StatusCode::OK,
            headers: headers.clone(),
            body: bytes.clone(),
            expires_at: Instant::now() + ttl,
        },
    );

    let mut response = Response::builder().status(StatusCode::OK);
    for (name, value) in headers {
        response = response.header(name, value);
    }
    response.body(Body::from(bytes)).unwrap()
}

pub(crate) fn success_cached_response(
    state: &AppState,
    cache_key: String,
    data: Value,
    ttl: Duration,
) -> Response<Body> {
    json_cached_response(state, cache_key, success_response_payload(data), ttl)
}

pub(crate) fn fail_json_response(status: StatusCode, message: &str) -> Response<Body> {
    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(fail_response_payload(message, Value::Null).to_string()))
        .unwrap()
}

pub(crate) fn cached_plain_response(
    state: &AppState,
    cache_key: String,
    bytes: bytes::Bytes,
    ttl: Duration,
    content_type: &str,
) -> Response<Body> {
    cached_plain_response_with_headers(state, cache_key, bytes, ttl, content_type, Vec::new())
}

pub(crate) fn cached_plain_response_with_headers(
    state: &AppState,
    cache_key: String,
    bytes: bytes::Bytes,
    ttl: Duration,
    content_type: &str,
    extra_headers: Vec<(HeaderName, HeaderValue)>,
) -> Response<Body> {
    let mut headers = vec![(
        HeaderName::from_static("content-type"),
        HeaderValue::from_str(content_type)
            .unwrap_or_else(|_| HeaderValue::from_static("text/plain")),
    )];
    headers.extend(extra_headers);

    state.response_cache.write().insert(
        cache_key,
        CachedResponse {
            status: StatusCode::OK,
            headers: headers.clone(),
            body: bytes.clone(),
            expires_at: Instant::now() + ttl,
        },
    );

    let mut response = Response::builder().status(StatusCode::OK);
    for (name, value) in headers {
        response = response.header(name, value);
    }
    response.body(Body::from(bytes)).unwrap()
}

pub(crate) fn parse_query(uri: &Uri) -> HashMap<String, String> {
    uri.query()
        .unwrap_or_default()
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?.trim();
            if key.is_empty() {
                return None;
            }
            let value = parts.next().unwrap_or_default();
            Some((key.to_string(), url_decode(value)))
        })
        .collect()
}

pub(crate) fn url_decode(value: &str) -> String {
    let replaced = value.replace('+', " ");
    percent_encoding::percent_decode_str(&replaced)
        .decode_utf8_lossy()
        .to_string()
}

pub(crate) fn try_cached_response(
    state: &AppState,
    cache_key: &str,
    request_headers: &HeaderMap,
) -> Option<Response<Body>> {
    if request_bypasses_response_cache(request_headers) {
        return None;
    }

    let cached = {
        let cache = state.response_cache.read();
        cache.get(cache_key).cloned()
    }?;

    if cached.expires_at <= Instant::now() {
        state.response_cache.write().remove(cache_key);
        return None;
    }

    if let Some(if_none_match) = request_headers.get("if-none-match") {
        if let Some((_, etag_value)) = cached.headers.iter().find(|(name, _)| name == &ETAG) {
            if if_none_match == etag_value {
                return Some(
                    Response::builder()
                        .status(StatusCode::NOT_MODIFIED)
                        .body(Body::empty())
                        .unwrap(),
                );
            }
        }
    }

    let mut response = Response::builder().status(cached.status);
    for (name, value) in &cached.headers {
        response = response.header(name, value);
    }
    Some(response.body(Body::from(cached.body)).unwrap())
}

fn request_bypasses_response_cache(request_headers: &HeaderMap) -> bool {
    let cache_control = request_headers
        .get("cache-control")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_default();
    if cache_control.contains("no-cache") || cache_control.contains("no-store") {
        return true;
    }

    request_headers
        .get("pragma")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.eq_ignore_ascii_case("no-cache"))
        .unwrap_or(false)
}

pub(crate) fn is_hop_header(name: &HeaderName) -> bool {
    matches!(
        name.as_str().to_ascii_lowercase().as_str(),
        "connection"
            | "keep-alive"
            | "proxy-authenticate"
            | "proxy-authorization"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
    )
}

pub(crate) fn json_error(status: StatusCode, message: &str) -> Response<Body> {
    let payload = json!({ "message": message }).to_string();
    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(payload))
        .unwrap()
}

pub(crate) fn internal_error(err: sqlx::Error) -> Response<Body> {
    error!("sql error: {}", err);
    json_error(StatusCode::INTERNAL_SERVER_ERROR, "database error")
}
