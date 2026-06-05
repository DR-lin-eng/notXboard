use super::super::*;
use crate::guest_v1::public::public_country_counts;

pub async fn overview(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_overview_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn leaderboards(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_leaderboards_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn geo(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_geo_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_overview_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let cache_key = build_cache_key(&uri);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }

    let metrics = load_public_overview_metrics(state)
        .await
        .map_err(internal_error)?;
    let avg_bandwidth = load_public_average_bandwidth(state)
        .await
        .map_err(internal_error)?;

    Ok(json_cached_response(
        state,
        cache_key,
        json!({
            "code": 0,
            "message": "success",
            "data": {
                "active_users": metrics.active_users,
                "registered_users": metrics.registered_users,
                "node_count": metrics.node_count,
                "avg_bandwidth": avg_bandwidth,
            }
        }),
        Duration::from_secs(15),
    ))
}

async fn build_leaderboards_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let cache_key = build_cache_key(&uri);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }

    let top_users = load_public_top_users(state)
        .await
        .map_err(internal_error)?;
    let top_nodes = load_public_top_nodes(state)
        .await
        .map_err(internal_error)?;
    let regions = public_country_counts(state);

    Ok(json_cached_response(
        state,
        cache_key,
        json!({
            "code": 0,
            "message": "success",
            "data": {
                "top_users": top_users,
                "top_nodes": top_nodes,
                "regions": regions,
            }
        }),
        Duration::from_secs(15),
    ))
}

async fn build_geo_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let cache_key = build_cache_key(&uri);
    if let Some(response) = try_cached_response(state, &cache_key, &headers) {
        return Ok(response);
    }

    let regions = public_country_counts(state);

    Ok(json_cached_response(
        state,
        cache_key,
        json!({
            "code": 0,
            "message": "success",
            "data": {
                "regions": regions,
            }
        }),
        Duration::from_secs(15),
    ))
}
