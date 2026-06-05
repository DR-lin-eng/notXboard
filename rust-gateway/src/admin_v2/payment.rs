use super::super::*;
use super::payment_forms::{build_payment_form_schema, supported_payment_methods};

#[derive(Clone, sqlx::FromRow)]
struct AdminPaymentRow {
    id: i64,
    uuid: String,
    payment: String,
    name: String,
    icon: Option<String>,
    config: String,
    notify_domain: Option<String>,
    handling_fee_fixed: Option<i64>,
    handling_fee_percent: Option<String>,
    enable: bool,
    sort: Option<i64>,
    created_at: i64,
    updated_at: i64,
}

pub async fn fetch(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_fetch_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_payment_methods(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_payment_methods_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_payment_form(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_get_payment_form_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn save(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_save_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn show(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_show_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn drop(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_drop_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn sort(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_sort_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_fetch_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payments = load_admin_payments(state).await.map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Array(
        payments.iter().map(|row| serialize_admin_payment(state, row)).collect::<Vec<_>>()
    ))))
}

async fn build_get_payment_methods_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    Ok(json_value_response(success_response_payload(Value::Array(
        supported_payment_methods()
            .iter()
            .copied()
            .map(Value::from)
            .collect::<Vec<_>>()
    ))))
}

async fn build_get_payment_form_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let obj = payload.as_object().ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    let payment = obj.get("payment").and_then(|value| value.as_str()).map(|value| value.trim()).filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    let payment_id = obj.get("id").and_then(parse_i64_value).filter(|value| *value > 0);

    let existing = match payment_id {
        Some(id) => load_admin_payment_by_id(state, id).await.map_err(internal_error)?,
        None => None,
    };
    let existing_config = existing
        .as_ref()
        .and_then(|row| serde_json::from_str::<Value>(&row.config).ok())
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default();

    let form = build_payment_form_schema(payment, &existing_config)
        .ok_or_else(|| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"支付方式不存在或未启用"})))?;
    Ok(json_value_response(success_response_payload(Value::Object(form))))
}

async fn build_save_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let obj = payload.as_object().ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let name = obj.get("name").and_then(|value| value.as_str()).map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "显示名称不能为空"))?;
    let payment = obj.get("payment").and_then(|value| value.as_str()).map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "网关参数不能为空"))?;
    let config = obj.get("config").cloned().filter(|value| value.is_object())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "配置参数不能为空"))?;
    let icon = parse_optional_string_field(obj.get("icon"))?;
    let notify_domain = parse_optional_string_field(obj.get("notify_domain"))?;
    let handling_fee_fixed = parse_optional_i64_field(obj.get("handling_fee_fixed"))?;
    let handling_fee_percent = parse_optional_decimal_string(obj.get("handling_fee_percent"), 0.0, 100.0)?;
    let config_text = serde_json::to_string(&config).unwrap_or_else(|_| "{}".to_string());
    let now = Utc::now().timestamp();

    if let Some(id) = obj.get("id").and_then(parse_i64_value).filter(|value| *value > 0) {
        let existing = load_admin_payment_by_id(state, id).await.map_err(internal_error)?;
        let Some(_) = existing else {
            return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"支付方式不存在"})));
        };
        sqlx::query(
            "UPDATE v2_payment
             SET name = ?, payment = ?, icon = ?, config = ?, notify_domain = ?, handling_fee_fixed = ?, handling_fee_percent = ?, updated_at = ?
             WHERE id = ?"
        )
        .bind(&name)
        .bind(&payment)
        .bind(icon.clone())
        .bind(&config_text)
        .bind(notify_domain.clone())
        .bind(handling_fee_fixed)
        .bind(handling_fee_percent.clone())
        .bind(now)
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
        return Ok(json_value_response(success_response_payload(Value::Bool(true))));
    }

    let uuid = random_hex(32);
    sqlx::query(
        "INSERT INTO v2_payment
            (uuid, payment, name, icon, config, notify_domain, handling_fee_fixed, handling_fee_percent, enable, sort, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 1, 0, ?, ?)"
    )
    .bind(&uuid)
    .bind(&payment)
    .bind(&name)
    .bind(icon.clone())
    .bind(&config_text)
    .bind(notify_domain.clone())
    .bind(handling_fee_fixed)
    .bind(handling_fee_percent.clone())
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_show_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let id = payload.get("id").and_then(parse_i64_value).filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "支付方式ID不能为空"))?;
    let payment = load_admin_payment_by_id(state, id).await.map_err(internal_error)?;
    let Some(payment) = payment else {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"支付方式不存在"})));
    };
    sqlx::query("UPDATE v2_payment SET enable = ?, updated_at = ? WHERE id = ?")
        .bind(!payment.enable)
        .bind(Utc::now().timestamp())
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_drop_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let id = payload.get("id").and_then(parse_i64_value).filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "支付方式ID不能为空"))?;
    let deleted = sqlx::query("DELETE FROM v2_payment WHERE id = ?")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    if deleted.rows_affected() == 0 {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"支付方式不存在"})));
    }
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_sort_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let ids = payload.get("ids").and_then(|value| value.as_array())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "参数有误"))?;
    if ids.is_empty() {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "参数有误"));
    }

    let parsed_ids = ids
        .iter()
        .filter_map(parse_i64_value)
        .filter(|value| *value > 0)
        .collect::<Vec<_>>();
    if parsed_ids.len() != ids.len() {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "参数有误"));
    }

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    for (index, payment_id) in parsed_ids.iter().enumerate() {
        let updated = sqlx::query("UPDATE v2_payment SET sort = ?, updated_at = ? WHERE id = ?")
            .bind((index + 1) as i64)
            .bind(Utc::now().timestamp())
            .bind(*payment_id)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
        if updated.rows_affected() == 0 {
            tx.rollback().await.ok();
            return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"支付方式不存在"})));
        }
    }
    tx.commit().await.map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn load_admin_payments(state: &AppState) -> Result<Vec<AdminPaymentRow>, sqlx::Error> {
    sqlx::query_as::<_, AdminPaymentRow>(
        "SELECT id, uuid, payment, name, icon, config, notify_domain, handling_fee_fixed,
                CAST(handling_fee_percent AS CHAR) AS handling_fee_percent,
                enable, sort, created_at, updated_at
         FROM v2_payment
         ORDER BY sort ASC, id ASC"
    )
    .fetch_all(&state.db)
    .await
}

async fn load_admin_payment_by_id(
    state: &AppState,
    payment_id: i64,
) -> Result<Option<AdminPaymentRow>, sqlx::Error> {
    sqlx::query_as::<_, AdminPaymentRow>(
        "SELECT id, uuid, payment, name, icon, config, notify_domain, handling_fee_fixed,
                CAST(handling_fee_percent AS CHAR) AS handling_fee_percent,
                enable, sort, created_at, updated_at
         FROM v2_payment
         WHERE id = ?
         LIMIT 1"
    )
    .bind(payment_id)
    .fetch_optional(&state.db)
    .await
}

fn serialize_admin_payment(state: &AppState, row: &AdminPaymentRow) -> Value {
    let notify_url = build_admin_payment_notify_url(state, row);
    let config = serde_json::from_str::<Value>(&row.config).unwrap_or_else(|_| Value::Object(Map::new()));
    json!({
        "id": row.id,
        "uuid": row.uuid,
        "payment": row.payment,
        "name": row.name,
        "icon": row.icon,
        "config": config,
        "notify_domain": row.notify_domain,
        "notify_url": notify_url,
        "handling_fee_fixed": row.handling_fee_fixed,
        "handling_fee_percent": row.handling_fee_percent.as_deref().and_then(|value| value.parse::<f64>().ok()),
        "enable": row.enable,
        "show": row.enable,
        "sort": row.sort,
        "created_at": row.created_at,
        "updated_at": row.updated_at,
    })
}

fn build_admin_payment_notify_url(state: &AppState, row: &AdminPaymentRow) -> String {
    let base = first_non_empty(&[
        env::var("APP_URL").unwrap_or_default(),
        "http://127.0.0.1:18095".to_string(),
    ]);
    let default_url = format!("{}/api/v1/guest/payment/notify/{}/{}", base.trim_end_matches('/'), row.payment, row.uuid);
    if let Some(domain) = row.notify_domain.as_deref().map(str::trim).filter(|value| !value.is_empty()) {
        if let Some(path_start) = default_url.find("/api/v1/guest/payment/notify/") {
            return format!("{}{}", domain.trim_end_matches('/'), &default_url[path_start..]);
        }
    }
    default_url
}

fn parse_optional_decimal_string(
    value: Option<&Value>,
    min: f64,
    max: f64,
) -> Result<Option<String>, Response<Body>> {
    match value {
        Some(Value::Null) | None => Ok(None),
        Some(Value::String(text)) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return Ok(None);
            }
            let parsed = trimmed.parse::<f64>().map_err(|_| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "百分比手续费范围须在0-100之间"))?;
            if parsed < min || parsed > max {
                return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "百分比手续费范围须在0-100之间"));
            }
            Ok(Some(format!("{:.2}", parsed)))
        }
        Some(Value::Number(number)) => {
            let parsed = number.as_f64().ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "百分比手续费范围须在0-100之间"))?;
            if parsed < min || parsed > max {
                return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "百分比手续费范围须在0-100之间"));
            }
            Ok(Some(format!("{:.2}", parsed)))
        }
        Some(_) => Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "百分比手续费范围须在0-100之间")),
    }
}
