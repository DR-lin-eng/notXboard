use crate::*;

pub(crate) async fn build_admin_user_generate_response(
    state: &AppState,
    headers: HeaderMap,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let obj = payload
        .as_object()
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let email_suffix = obj
        .get("email_suffix")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().trim_start_matches('@').to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "邮箱后缀不能为空"))?;

    if let Some(email_prefix) = request_optional_string_field(&payload, "email_prefix").flatten() {
        return build_single_generate_response(state, &payload, &email_suffix, &email_prefix).await;
    }

    build_batch_generate_response(state, &payload, obj, &email_suffix).await
}

async fn build_single_generate_response(
    state: &AppState,
    payload: &Value,
    email_suffix: &str,
    email_prefix: &str,
) -> Result<Response<Body>, Response<Body>> {
    let email = format!("{}@{}", email_prefix, email_suffix);
    let password = request_optional_string_field(payload, "password")
        .flatten()
        .unwrap_or_else(|| email.clone());
    let plan_id = request_optional_i64_field(payload, "plan_id").unwrap_or(None);
    let expired_at = request_optional_i64_field(payload, "expired_at").unwrap_or(None);
    let defaults = load_generated_user_defaults(state).await;
    let plan_context = load_generated_user_plan_context(state, plan_id, expired_at).await?;
    create_generated_user(state, email, password, plan_id, &defaults, &plan_context).await?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_batch_generate_response(
    state: &AppState,
    payload: &Value,
    obj: &serde_json::Map<String, Value>,
    email_suffix: &str,
) -> Result<Response<Body>, Response<Body>> {
    let generate_count = obj
        .get("generate_count")
        .and_then(parse_i64_value)
        .filter(|value| *value > 0 && *value <= 500)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "生成数量最大为500个"))?;
    let plan_id = request_optional_i64_field(payload, "plan_id").unwrap_or(None);
    let expired_at = request_optional_i64_field(payload, "expired_at").unwrap_or(None);
    let fixed_password = request_optional_string_field(payload, "password").flatten();
    let download_csv = obj
        .get("download_csv")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let defaults = load_generated_user_defaults(state).await;
    let plan_context = load_generated_user_plan_context(state, plan_id, expired_at).await?;

    let prepared = prepare_random_generated_users(
        state,
        email_suffix,
        fixed_password.as_deref(),
        generate_count as usize,
    )
    .await?;
    batch_insert_generated_users(state, &prepared, plan_id, &defaults, &plan_context).await?;
    let generated = prepared
        .into_iter()
        .map(|user| prepared_user_to_result(user, plan_context.effective_expired_at))
        .collect::<Vec<_>>();

    if download_csv {
        return Ok(build_generated_users_csv_response(&generated));
    }

    Ok(json_value_response(json!({
        "code": 0,
        "message": "批量生成成功",
        "data": generated,
    })))
}

fn build_generated_users_csv_response(rows: &[Value]) -> Response<Body> {
    let mut csv = String::from("账号,密码,过期时间,UUID,创建时间,订阅地址\n");
    for row in rows {
        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
            row.get("email").and_then(Value::as_str).unwrap_or_default(),
            row.get("password").and_then(Value::as_str).unwrap_or_default(),
            row.get("expired_at").and_then(Value::as_str).unwrap_or_default(),
            row.get("uuid").and_then(Value::as_str).unwrap_or_default(),
            row.get("created_at").and_then(Value::as_str).unwrap_or_default(),
            row.get("subscribe_url").and_then(Value::as_str).unwrap_or_default(),
        ));
    }

    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/csv; charset=utf-8")
        .header("Content-Disposition", "attachment; filename=\"users.csv\"")
        .body(Body::from(csv))
        .unwrap()
}
