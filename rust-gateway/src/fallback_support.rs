use crate::*;

pub async fn rust_fallback(
    State(state): State<Arc<AppState>>,
    method: Method,
    headers: HeaderMap,
    uri: Uri,
    _body: Body,
) -> Response<Body> {
    if method == Method::GET {
        if let Some(response) = web_pages::try_render_page(&state, &headers, &uri).await {
            return exposure_control::protect_fallback_web_response(&state, &headers, response);
        }
    }

    json_error(StatusCode::NOT_FOUND, "Not found")
}
