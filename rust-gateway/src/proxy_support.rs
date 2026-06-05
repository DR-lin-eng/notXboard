use crate::*;

pub async fn response_body_bytes(response: Response<Body>) -> bytes::Bytes {
    let (_, body) = response.into_parts();
    match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(_) => bytes::Bytes::new(),
    }
}

pub async fn map_proxy_response(resp: Response<hyper::body::Incoming>) -> Response<Body> {
    let status = resp.status();
    let headers = resp.headers().clone();
    let body = resp.into_body();
    let bytes = match body.collect().await {
        Ok(bytes) => bytes.to_bytes(),
        Err(err) => {
            return json_error(
                StatusCode::BAD_GATEWAY,
                &format!("read backend body failed: {err}"),
            );
        }
    };

    let mut response = Response::builder().status(status);
    for (name, value) in headers.iter() {
        if is_hop_header(name) {
            continue;
        }
        response = response.header(name, value);
    }

    response
        .body(Body::from(bytes))
        .unwrap_or_else(|_| Response::new(Body::from("internal error")))
}
