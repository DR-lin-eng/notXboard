use crate::*;

pub async fn notify(
    State(state): State<Arc<AppState>>,
    axum::extract::Path((method, uuid)): axum::extract::Path<(String, String)>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_notify_response(&state, method, uuid, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_notify_response(
    state: &AppState,
    method: String,
    uuid: String,
    _headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let mut params = parse_query(&uri);
    let body_params = if body.size_hint().lower() > 0 {
        let bytes = body
            .collect()
            .await
            .map_err(|err| json_error(StatusCode::BAD_REQUEST, &format!("read body failed: {err}")))?
            .to_bytes();
        if bytes.is_empty() {
            HashMap::new()
        } else {
            serde_urlencoded::from_bytes::<HashMap<String, String>>(&bytes).unwrap_or_default()
        }
    } else {
        HashMap::new()
    };
    for (key, value) in body_params {
        params.insert(key, value);
    }

    if method != "EPay" {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "verify error"));
    }

    let trade_no = params
        .get("out_trade_no")
        .cloned()
        .unwrap_or_else(|| params.get("trade_no").cloned().unwrap_or_default());
    let callback_no = params.get("trade_no").cloned().unwrap_or_default();
    if trade_no.is_empty() {
        return Ok(fail_json_response(StatusCode::BAD_REQUEST, "handle error"));
    }

    let config = if uuid == "sponsor" {
        let sponsor = load_sponsor_epay_profile(state).await?;
        let Some(sponsor) = sponsor else {
            return Ok(fail_json_response(StatusCode::INTERNAL_SERVER_ERROR, "fail"));
        };
        EpayConfig {
            pid: sponsor.pid,
            key: sponsor.key,
            url: sponsor.url,
            submit_path: sponsor.submit_path,
            use_post: sponsor.use_post,
            sitename: sponsor.sitename,
            device: sponsor.device,
        }
    } else {
        let payment = load_payment_notify_method_by_uuid(state, &uuid)
            .await
            .map_err(internal_error)?;
        let Some(payment) = payment else {
            return Ok(fail_json_response(StatusCode::INTERNAL_SERVER_ERROR, "fail"));
        };
        if !payment.enable {
            return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "verify error"));
        }
        if let Some(config) = load_order_epay_config_snapshot(state, &trade_no)
            .await
            .map_err(|err| json_error(StatusCode::INTERNAL_SERVER_ERROR, &err))?
        {
            config
        } else {
            load_payment_config(state, payment.id)
                .await
                .map_err(internal_error)?
        }
    };

    if !verify_epay_notify_signature(&params, &config) {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "verify error"));
    }

    if uuid == "sponsor" {
        if !mark_sponsor_donation_paid(state, &trade_no, &callback_no)
            .await
            .map_err(internal_error)?
        {
            return Ok(fail_json_response(StatusCode::BAD_REQUEST, "handle error"));
        }
    } else {
        let Some(order_id) = mark_order_paid_processing(state, &trade_no, &callback_no)
            .await
            .map_err(internal_error)? else {
            return Ok(fail_json_response(StatusCode::BAD_REQUEST, "handle error"));
        };
        complete_processing_order_by_id(state, order_id)
            .await
            .map_err(internal_error)?;
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from("success"))
        .unwrap())
}
