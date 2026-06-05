use super::super::*;
#[derive(Clone, sqlx::FromRow)]
struct AdminUserRow {
    id: i64,
    invite_user_id: Option<i64>,
    telegram_id: Option<i64>,
    email: String,
    balance: i64,
    discount: Option<i64>,
    commission_type: i64,
    commission_rate: Option<i64>,
    commission_balance: i64,
    t: i64,
    u: i64,
    d: i64,
    transfer_enable: i64,
    banned: i8,
    ban_reason: Option<String>,
    banned_at: Option<i64>,
    banned_by_admin_id: Option<i64>,
    is_admin: i8,
    is_staff: i8,
    is_super_admin: i8,
    last_login_at: Option<i64>,
    uuid: String,
    group_id: Option<i64>,
    plan_id: Option<i64>,
    speed_limit: Option<i64>,
    remind_expire: Option<i8>,
    remind_traffic: Option<i8>,
    token: String,
    subscribe_path: Option<String>,
    subscribe_key: Option<String>,
    subscribe_salt: Option<String>,
    expired_at: Option<i64>,
    remarks: Option<String>,
    linux_do_id: Option<String>,
    linux_do_username: Option<String>,
    linux_do_name: Option<String>,
    trust_level: i64,
    is_silenced: i8,
    api_key: Option<String>,
    device_limit: Option<i64>,
    concurrent_ip_limit: u64,
    created_at: i64,
    updated_at: i64,
    plan_name: Option<String>,
    group_name: Option<String>,
    invite_user_email: Option<String>,
}

#[derive(Debug)]
enum UserBind {
    I64(i64),
    String(String),
}

const FILTERABLE_USER_FIELDS: &[&str] = &[
    "id",
    "invite_user_id",
    "telegram_id",
    "email",
    "balance",
    "discount",
    "commission_type",
    "commission_rate",
    "commission_balance",
    "t",
    "u",
    "d",
    "transfer_enable",
    "banned",
    "is_admin",
    "is_staff",
    "is_super_admin",
    "last_login_at",
    "uuid",
    "group_id",
    "group_ids",
    "plan_id",
    "speed_limit",
    "remind_expire",
    "remind_traffic",
    "token",
    "subscribe_path",
    "subscribe_key",
    "subscribe_salt",
    "expired_at",
    "remarks",
    "linux_do_id",
    "linux_do_username",
    "linux_do_name",
    "trust_level",
    "is_silenced",
    "api_key",
    "device_limit",
    "concurrent_ip_limit",
    "created_at",
    "updated_at",
    "total_used",
];

const SORTABLE_USER_FIELDS: &[&str] = &[
    "id",
    "invite_user_id",
    "telegram_id",
    "email",
    "balance",
    "discount",
    "commission_type",
    "commission_rate",
    "commission_balance",
    "t",
    "u",
    "d",
    "transfer_enable",
    "banned",
    "is_admin",
    "is_staff",
    "is_super_admin",
    "last_login_at",
    "group_id",
    "plan_id",
    "speed_limit",
    "remind_expire",
    "remind_traffic",
    "expired_at",
    "trust_level",
    "is_silenced",
    "device_limit",
    "concurrent_ip_limit",
    "created_at",
    "updated_at",
    "total_used",
];

pub async fn fetch(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_fetch_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_update_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn set_invite_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_set_invite_user_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn reset_secret(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_reset_secret_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_user_info_by_id(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_user_info_by_id_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn generate(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_admin_user_generate_response(&state, headers, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn destroy(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_destroy_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn ban(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_ban_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn ban_records(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_ban_records_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn send_mail(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_send_mail_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn dump_csv(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_dump_csv_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

#[derive(Clone, sqlx::FromRow)]
struct UserBanRecordRow {
    id: u64,
    user_id: i64,
    admin_id: Option<i64>,
    action: String,
    reason: String,
    source: String,
    context: Option<String>,
    created_at: i64,
    user_email: Option<String>,
    admin_email: Option<String>,
}

async fn build_fetch_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await.unwrap_or_else(|_| json!({}));
    let current = payload
        .get("current")
        .and_then(parse_i64_value)
        .filter(|value| *value > 0)
        .unwrap_or(1);
    let page_size = payload
        .get("pageSize")
        .and_then(parse_i64_value)
        .filter(|value| *value > 0)
        .unwrap_or(10)
        .clamp(1, 200);

    let mut sql = String::from(
        "SELECT
            u.id, u.invite_user_id, u.telegram_id, u.email, u.balance, u.discount, u.commission_type, u.commission_rate,
            u.commission_balance, u.t, u.u, u.d, u.transfer_enable, u.banned, u.ban_reason, u.banned_at,
            u.banned_by_admin_id, u.is_admin, u.is_staff, u.is_super_admin, u.last_login_at, u.uuid, u.group_id,
            u.plan_id, u.speed_limit, u.remind_expire, u.remind_traffic, u.token, u.subscribe_path, u.subscribe_key,
            u.subscribe_salt, u.expired_at, u.remarks, u.linux_do_id, u.linux_do_username, u.linux_do_name,
            u.trust_level, u.is_silenced, u.api_key, u.device_limit, u.concurrent_ip_limit, u.created_at, u.updated_at,
            p.name AS plan_name, g.name AS group_name, iu.email AS invite_user_email
         FROM v2_user u
         LEFT JOIN v2_plan p ON p.id = u.plan_id
         LEFT JOIN v2_server_group g ON g.id = u.group_id
         LEFT JOIN v2_user iu ON iu.id = u.invite_user_id
         WHERE 1=1"
    );
    let mut total_sql = String::from("SELECT COUNT(*) FROM v2_user u WHERE 1=1");

    let mut binds = Vec::<UserBind>::new();
    if let Some(filters) = payload.get("filter").and_then(|value| value.as_array()) {
        for filter in filters {
            let Some(obj) = filter.as_object() else { continue; };
            let Some(field) = obj.get("id").and_then(|value| value.as_str()) else { continue; };
            append_user_filter_clause(&mut sql, &mut total_sql, &mut binds, field, obj.get("value"));
        }
    }

    let mut order_by = parse_user_sorts(payload.get("sort").and_then(|value| value.as_array()));
    if order_by.is_empty() {
        order_by.push("u.id DESC".to_string());
    }
    sql.push_str(" ORDER BY ");
    sql.push_str(&order_by.join(", "));
    sql.push_str(" LIMIT ? OFFSET ?");

    let mut total_query = sqlx::query_scalar::<_, i64>(&total_sql);
    for bind in &binds {
        total_query = match bind {
            UserBind::I64(value) => total_query.bind(*value),
            UserBind::String(value) => total_query.bind(value.clone()),
        };
    }
    let total = total_query.fetch_one(&state.db).await.map_err(internal_error)?;

    let mut query = sqlx::query_as::<_, AdminUserRow>(&sql);
    for bind in &binds {
        query = match bind {
            UserBind::I64(value) => query.bind(*value),
            UserBind::String(value) => query.bind(value.clone()),
        };
    }
    let rows = query
        .bind(page_size)
        .bind((current - 1) * page_size)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;

    Ok(json_value_response(json!({
        "total": total,
        "current_page": current,
        "per_page": page_size,
        "last_page": if total <= 0 { 1 } else { ((total + page_size - 1) / page_size).max(1) },
        "data": rows.into_iter().map(|row| serialize_admin_user_summary(state, row)).collect::<Vec<_>>(),
    })))
}

async fn build_get_user_info_by_id_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let user_id = params
        .get("id")
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "用户ID不能为空"))?;

    let user = load_admin_user_by_id(state, user_id).await.map_err(internal_error)?;
    let Some(user) = user else {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"用户不存在"})));
    };
    Ok(json_value_response(success_response_payload(serialize_admin_user_detail(state, user))))
}

async fn build_reset_secret_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let user_id = payload
        .get("id")
        .and_then(parse_i64_value)
        .filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "用户ID不能为空"))?;

    let existing = load_admin_user_by_id(state, user_id).await.map_err(internal_error)?;
    let Some(_) = existing else {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"用户不存在"})));
    };

    reset_single_user_security(state, user_id)
        .await
        .map_err(internal_error)?;

    let updated_user = load_bearer_user_by_id(state, user_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::BAD_REQUEST, "The user does not exist"))?;
    let subscribe_url = build_user_subscribe_url(state, &updated_user).await.unwrap_or_default();
    Ok(json_value_response(success_response_payload(Value::Bool(!subscribe_url.is_empty()))))
}

async fn build_update_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let admin = authenticate_super_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let obj = payload
        .as_object()
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    let user_id = obj
        .get("id")
        .and_then(parse_i64_value)
        .filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "用户ID不能为空"))?;

    let existing = load_admin_user_by_id(state, user_id).await.map_err(internal_error)?;
    let Some(existing) = existing else {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"用户不存在"})));
    };

    let mut tx = state.db.begin().await.map_err(internal_error)?;

    if let Some(email) = request_optional_string_field(&payload, "email") {
        let email = email.unwrap_or_default();
        if email.is_empty() {
            tx.rollback().await.ok();
            return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "邮箱不能为空"));
        }
        let duplicated: Option<i64> = sqlx::query_scalar("SELECT id FROM v2_user WHERE email = ? AND id <> ? LIMIT 1")
            .bind(&email)
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(internal_error)?;
        if duplicated.is_some() {
            tx.rollback().await.ok();
            return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"邮箱已被使用"})));
        }
    }

    if let Some(plan_id) = request_optional_i64_field(&payload, "plan_id") {
        if let Some(plan_id) = plan_id {
            let plan_group_id: Option<Option<u64>> = sqlx::query_scalar("SELECT group_id FROM v2_plan WHERE id = ? LIMIT 1")
                .bind(plan_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(internal_error)?;
            if plan_group_id.is_none() {
                tx.rollback().await.ok();
                return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"订阅计划不存在"})));
            }
        }
    }

    let resolved_invite_user_id = resolve_invite_user_id(&mut tx, &payload, user_id).await?;

    let mut updates = Vec::<(&str, Value)>::new();
    push_optional_string_update(&payload, "email", &mut updates);
    push_optional_i64_update(&payload, "transfer_enable", &mut updates);
    push_optional_i64_update(&payload, "expired_at", &mut updates);
    push_optional_i64_update(&payload, "plan_id", &mut updates);
    push_optional_i64_update(&payload, "commission_rate", &mut updates);
    push_optional_i64_update(&payload, "discount", &mut updates);
    push_optional_i64_update(&payload, "u", &mut updates);
    push_optional_i64_update(&payload, "d", &mut updates);
    push_optional_i64_update(&payload, "commission_type", &mut updates);
    push_optional_i64_update(&payload, "speed_limit", &mut updates);
    push_optional_i64_update(&payload, "device_limit", &mut updates);
    push_optional_i64_update(&payload, "concurrent_ip_limit", &mut updates);
    push_optional_i64_update(&payload, "telegram_id", &mut updates);
    push_optional_i64_update(&payload, "trust_level", &mut updates);
    push_optional_bool_update(&payload, "is_admin", &mut updates);
    push_optional_bool_update(&payload, "is_staff", &mut updates);
    push_optional_bool_update(&payload, "is_silenced", &mut updates);
    push_optional_bool_update(&payload, "remind_expire", &mut updates);
    push_optional_bool_update(&payload, "remind_traffic", &mut updates);
    push_optional_string_nullable_update(&payload, "remarks", &mut updates);

    if let Some(balance_value) = request_optional_decimal_cents_field(&payload, "balance")? {
        updates.push(("balance", Value::from(balance_value)));
    }
    if let Some(balance_value) = request_optional_decimal_cents_field(&payload, "commission_balance")? {
        updates.push(("commission_balance", Value::from(balance_value)));
    }
    if let Some(invite_user_id) = resolved_invite_user_id {
        updates.push(("invite_user_id", invite_user_id.map(Value::from).unwrap_or(Value::Null)));
    }

    if let Some(plan_id) = request_optional_i64_field(&payload, "plan_id") {
        let group_id = if let Some(plan_id) = plan_id {
            sqlx::query_scalar::<_, Option<u64>>("SELECT group_id FROM v2_plan WHERE id = ? LIMIT 1")
                .bind(plan_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(internal_error)?
                .flatten()
                .and_then(|value| i64::try_from(value).ok())
        } else {
            None
        };
        updates.push(("group_id", group_id.map(Value::from).unwrap_or(Value::Null)));
    }

    if let Some(password) = request_optional_string_field(&payload, "password") {
        let password = password.unwrap_or_default();
        if !password.is_empty() {
            if password.chars().count() < 8 {
                tx.rollback().await.ok();
                return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "密码长度最小8位"));
            }
            let hashed = bcrypt::hash(password, 12)
                .map_err(|_| fail_json_response(StatusCode::INTERNAL_SERVER_ERROR, "保存失败"))?;
            updates.push(("password", Value::from(hashed)));
            updates.push(("password_algo", Value::Null));
            updates.push(("password_salt", Value::Null));
        }
    }

    let target_banned = request_optional_bool_field(&payload, "banned");
    let ban_reason = request_optional_string_field(&payload, "ban_reason")
        .flatten()
        .unwrap_or_default();
    apply_ban_updates(&mut tx, user_id, existing.banned != 0, existing.ban_reason.as_deref(), target_banned, ban_reason.as_str(), admin.id).await?;

    if !updates.is_empty() {
        apply_user_updates(&mut tx, user_id, updates).await?;
    }

    tx.commit().await.map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_set_invite_user_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let object = payload
        .as_object()
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let user_id = object
        .get("id")
        .or_else(|| object.get("user_id"))
        .and_then(parse_i64_value)
        .filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "用户ID不能为空"))?;

    let existing = load_admin_user_by_id(state, user_id).await.map_err(internal_error)?;
    let Some(_) = existing else {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"用户不存在"})));
    };

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let resolved_invite_user_id = resolve_invite_user_id(&mut tx, &payload, user_id).await?;
    let Some(invite_user_id) = resolved_invite_user_id else {
        tx.rollback().await.ok();
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    };

    apply_user_updates(
        &mut tx,
        user_id,
        vec![("invite_user_id", invite_user_id.map(Value::from).unwrap_or(Value::Null))],
    )
    .await?;
    tx.commit().await.map_err(internal_error)?;

    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_destroy_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let user_id = payload
        .get("id")
        .and_then(parse_i64_value)
        .filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "用户ID不能为空"))?;

    let existing = load_admin_user_by_id(state, user_id).await.map_err(internal_error)?;
    let Some(_) = existing else {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"用户不存在"})));
    };

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    sqlx::query("DELETE FROM v2_order WHERE user_id = ?")
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    sqlx::query("DELETE FROM v2_invite_code WHERE user_id = ?")
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    sqlx::query("DELETE FROM v2_stat_user WHERE user_id = ?")
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    sqlx::query("DELETE FROM v2_ticket WHERE user_id = ?")
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    sqlx::query("DELETE FROM personal_access_tokens WHERE tokenable_id = ?")
        .bind(user_id as u64)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    sqlx::query("DELETE FROM user_ban_records WHERE user_id = ?")
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    sqlx::query("DELETE FROM v2_user WHERE id = ?")
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    tx.commit().await.map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_ban_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let admin = authenticate_super_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let reason = payload
        .get("reason")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "批量封禁时必须填写封禁原因"))?;

    let users = load_bannable_users(state, &payload).await?;
    if users.is_empty() {
        return Ok(json_value_response(success_response_payload(json!({"updated": 0}))));
    }

    let updated = batch_ban_users(state, &users, &reason, admin.id).await?;

    Ok(json_value_response(success_response_payload(json!({"updated": updated}))))
}

async fn build_ban_records_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let current = params.get("current").and_then(|value| value.parse::<i64>().ok()).unwrap_or(1).max(1);
    let page_size = params.get("pageSize").and_then(|value| value.parse::<i64>().ok()).unwrap_or(20).clamp(1, 100);
    let offset = (current - 1) * page_size;
    let user_id_filter = params.get("user_id").and_then(|value| value.parse::<i64>().ok()).filter(|value| *value > 0);
    let action_filter = params.get("action").map(|value| value.trim().to_string()).filter(|value| !value.is_empty());

    let mut where_clause = String::from(" WHERE 1=1");
    let mut binds = Vec::<UserBind>::new();
    if let Some(user_id) = user_id_filter {
        where_clause.push_str(" AND r.user_id = ?");
        binds.push(UserBind::I64(user_id));
    }
    if let Some(action) = action_filter {
        where_clause.push_str(" AND r.action = ?");
        binds.push(UserBind::String(action));
    }

    let total_sql = format!("SELECT COUNT(*) FROM user_ban_records r{}", where_clause);
    let data_sql = format!(
        "SELECT
            r.id, r.user_id, r.admin_id, r.action, r.reason, r.source, r.context, r.created_at,
            u.email AS user_email, a.email AS admin_email
         FROM user_ban_records r
         LEFT JOIN v2_user u ON u.id = r.user_id
         LEFT JOIN v2_user a ON a.id = r.admin_id
         {}
         ORDER BY r.id DESC
         LIMIT ? OFFSET ?",
        where_clause
    );

    let mut total_query = sqlx::query_scalar::<_, i64>(&total_sql);
    for bind in &binds {
        total_query = match bind {
            UserBind::I64(value) => total_query.bind(*value),
            UserBind::String(value) => total_query.bind(value.clone()),
        };
    }
    let total = total_query.fetch_one(&state.db).await.map_err(internal_error)?;

    let mut query = sqlx::query_as::<_, UserBanRecordRow>(&data_sql);
    for bind in &binds {
        query = match bind {
            UserBind::I64(value) => query.bind(*value),
            UserBind::String(value) => query.bind(value.clone()),
        };
    }
    let rows = query
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;

    Ok(json_value_response(json!({
        "total": total,
        "current_page": current,
        "per_page": page_size,
        "last_page": if total <= 0 { 1 } else { ((total + page_size - 1) / page_size).max(1) },
        "data": rows.into_iter().map(serialize_ban_record).collect::<Vec<_>>(),
    })))
}

async fn build_send_mail_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let subject = payload
        .get("subject")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "主题不能为空"))?;
    let content = payload
        .get("content")
        .and_then(|value| value.as_str())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "发送内容不能为空"))?;

    let recipients = load_user_emails_for_mass_action(state, &payload).await?;
    let stats = send_mass_mail(Arc::new(state.clone()), recipients, subject, content).await;
    Ok(json_value_response(success_response_payload(json!({
        "success": stats.failed == 0,
        "total": stats.total,
        "sent": stats.sent,
        "failed": stats.failed,
        "concurrency": stats.concurrency,
        "sample_errors": stats.sample_errors,
    }))))
}

async fn build_dump_csv_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await.unwrap_or_else(|_| json!({}));
    let users = load_users_for_csv(state, &payload).await?;

    let mut csv = String::from("\u{feff}邮箱,余额,推广佣金,总流量,剩余流量,套餐到期时间,订阅计划,订阅地址\n");
    for user in users {
        let subscribe_url = build_subscribe_url_from_admin_row(state, &user).unwrap_or_default();
        let total_traffic = traffic_convert(user.transfer_enable);
        let remaining = traffic_convert((user.transfer_enable - (user.u + user.d)).max(0));
        let expired_at = if user.expired_at.unwrap_or(0) > 0 {
            format_timestamp(user.expired_at.unwrap_or(0))
        } else {
            "长期有效".to_string()
        };
        let plan_name = user.plan_name.clone().unwrap_or_else(|| "无订阅".to_string());
        csv.push_str(&format!(
            "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
            sanitize_for_csv(&user.email),
            sanitize_for_csv(&format!("{:.2}", (user.balance as f64) / 100.0)),
            sanitize_for_csv(&format!("{:.2}", (user.commission_balance as f64) / 100.0)),
            sanitize_for_csv(&total_traffic),
            sanitize_for_csv(&remaining),
            sanitize_for_csv(&expired_at),
            sanitize_for_csv(&plan_name),
            sanitize_for_csv(&subscribe_url),
        ));
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "text/csv; charset=utf-8")
        .header("Content-Disposition", "attachment; filename=\"users.csv\"")
        .body(Body::from(csv))
        .unwrap())
}

async fn load_admin_user_by_id(
    state: &AppState,
    user_id: i64,
) -> Result<Option<AdminUserRow>, sqlx::Error> {
    sqlx::query_as::<_, AdminUserRow>(
        "SELECT
            u.id, u.invite_user_id, u.telegram_id, u.email, u.balance, u.discount, u.commission_type, u.commission_rate,
            u.commission_balance, u.t, u.u, u.d, u.transfer_enable, u.banned, u.ban_reason, u.banned_at,
            u.banned_by_admin_id, u.is_admin, u.is_staff, u.is_super_admin, u.last_login_at, u.uuid, u.group_id,
            u.plan_id, u.speed_limit, u.remind_expire, u.remind_traffic, u.token, u.subscribe_path, u.subscribe_key,
            u.subscribe_salt, u.expired_at, u.remarks, u.linux_do_id, u.linux_do_username, u.linux_do_name,
            u.trust_level, u.is_silenced, u.api_key, u.device_limit, u.concurrent_ip_limit, u.created_at, u.updated_at,
            p.name AS plan_name, g.name AS group_name, iu.email AS invite_user_email
         FROM v2_user u
         LEFT JOIN v2_plan p ON p.id = u.plan_id
         LEFT JOIN v2_server_group g ON g.id = u.group_id
         LEFT JOIN v2_user iu ON iu.id = u.invite_user_id
         WHERE u.id = ?
         LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
}

fn append_user_filter_clause(
    sql: &mut String,
    total_sql: &mut String,
    binds: &mut Vec<UserBind>,
    field: &str,
    value: Option<&Value>,
) {
    let Some(value) = value else { return; };
    match field {
        "group_ids" | "group_id" => {
            append_basic_user_filter(sql, total_sql, binds, "u.group_id", value, false);
        }
        "plan_id" => append_basic_user_filter(sql, total_sql, binds, "u.plan_id", value, false),
        "email" => append_basic_user_filter(sql, total_sql, binds, "u.email", value, true),
        "invite_user.email" => append_basic_user_filter(sql, total_sql, binds, "iu.email", value, true),
        "plan.name" => append_basic_user_filter(sql, total_sql, binds, "p.name", value, true),
        "group.name" => append_basic_user_filter(sql, total_sql, binds, "g.name", value, true),
        "invite_user.id" => append_basic_user_filter(sql, total_sql, binds, "iu.id", value, false),
        "plan.id" => append_basic_user_filter(sql, total_sql, binds, "p.id", value, false),
        "group.id" => append_basic_user_filter(sql, total_sql, binds, "g.id", value, false),
        "total_used" => append_total_used_filter(sql, total_sql, binds, value),
        _ if FILTERABLE_USER_FIELDS.contains(&field) => {
            append_basic_user_filter(sql, total_sql, binds, &format!("u.{}", field), value, field == "remarks" || field.contains("email") || field.contains("token") || field.contains("uuid") || field.contains("subscribe_") || field.contains("linux_do_") || field == "api_key");
        }
        _ => {}
    }
}

fn append_total_used_filter(
    sql: &mut String,
    total_sql: &mut String,
    binds: &mut Vec<UserBind>,
    value: &Value,
) {
    append_basic_user_filter(sql, total_sql, binds, "(u.u + u.d)", value, false);
}

fn append_basic_user_filter(
    sql: &mut String,
    total_sql: &mut String,
    binds: &mut Vec<UserBind>,
    field_sql: &str,
    value: &Value,
    default_like: bool,
) {
    match value {
        Value::Array(items) => {
            let parsed = items.iter().filter_map(parse_i64_value).collect::<Vec<_>>();
            if parsed.is_empty() {
                return;
            }
            let placeholders = vec!["?"; parsed.len()].join(",");
            let clause = format!(" AND {} IN ({})", field_sql, placeholders);
            sql.push_str(&clause);
            total_sql.push_str(&clause);
            for item in parsed {
                binds.push(UserBind::I64(item));
            }
        }
        Value::String(text) if text.contains(':') => {
            let mut parts = text.splitn(2, ':');
            let operator = parts.next().unwrap_or_default().to_ascii_lowercase();
            let raw = parts.next().unwrap_or_default().trim().to_string();
            let (clause, bind) = match operator.as_str() {
                "null" => (format!(" AND {} IS NULL", field_sql), None),
                "notnull" => (format!(" AND {} IS NOT NULL", field_sql), None),
                "eq" => (format!(" AND {} = ?", field_sql), Some(raw)),
                "gt" => (format!(" AND {} > ?", field_sql), Some(raw)),
                "gte" => (format!(" AND {} >= ?", field_sql), Some(raw)),
                "lt" => (format!(" AND {} < ?", field_sql), Some(raw)),
                "lte" => (format!(" AND {} <= ?", field_sql), Some(raw)),
                "like" => (format!(" AND {} LIKE ?", field_sql), Some(format!("%{}%", raw))),
                "notlike" => (format!(" AND {} NOT LIKE ?", field_sql), Some(format!("%{}%", raw))),
                _ => return,
            };
            sql.push_str(&clause);
            total_sql.push_str(&clause);
            if let Some(bind) = bind {
                if let Ok(number) = bind.parse::<i64>() {
                    binds.push(UserBind::I64(number));
                } else {
                    binds.push(UserBind::String(bind));
                }
            }
        }
        Value::String(text) => {
            let clause = if default_like {
                format!(" AND {} LIKE ?", field_sql)
            } else {
                format!(" AND {} = ?", field_sql)
            };
            sql.push_str(&clause);
            total_sql.push_str(&clause);
            if default_like {
                binds.push(UserBind::String(format!("%{}%", text)));
            } else if let Ok(number) = text.trim().parse::<i64>() {
                binds.push(UserBind::I64(number));
            } else {
                binds.push(UserBind::String(text.trim().to_string()));
            }
        }
        Value::Bool(flag) => {
            let clause = format!(" AND {} = ?", field_sql);
            sql.push_str(&clause);
            total_sql.push_str(&clause);
            binds.push(UserBind::I64(if *flag { 1 } else { 0 }));
        }
        Value::Number(number) => {
            if let Some(number) = number.as_i64() {
                let clause = format!(" AND {} = ?", field_sql);
                sql.push_str(&clause);
                total_sql.push_str(&clause);
                binds.push(UserBind::I64(number));
            }
        }
        _ => {}
    }
}

fn parse_user_sorts(values: Option<&Vec<Value>>) -> Vec<String> {
    let Some(values) = values else {
        return Vec::new();
    };
    let mut items = Vec::new();
    for sort in values {
        let Some(obj) = sort.as_object() else { continue; };
        let Some(field) = obj.get("id").and_then(|value| value.as_str()) else { continue; };
        if !SORTABLE_USER_FIELDS.contains(&field) {
            continue;
        }
        let desc = obj.get("desc").and_then(|value| value.as_bool()).unwrap_or(false);
        if field == "total_used" {
            items.push(format!("(u.u + u.d) {}", if desc { "DESC" } else { "ASC" }));
        } else {
            items.push(format!("u.{} {}", field, if desc { "DESC" } else { "ASC" }));
        }
    }
    items
}

fn serialize_admin_user_summary(state: &AppState, user: AdminUserRow) -> Value {
    let subscribe_url = build_subscribe_url_from_admin_row(state, &user).unwrap_or_default();
    let mut map = Map::new();
    map.insert("id".to_string(), Value::from(user.id));
    map.insert(
        "invite_user_id".to_string(),
        user.invite_user_id.map(Value::from).unwrap_or(Value::Null),
    );
    map.insert(
        "invite_user".to_string(),
        match (user.invite_user_id, user.invite_user_email.as_deref()) {
            (Some(id), Some(email)) => json!({"id": id, "email": email}),
            _ => Value::Null,
        },
    );
    map.insert(
        "telegram_id".to_string(),
        user.telegram_id.map(Value::from).unwrap_or(Value::Null),
    );
    map.insert("email".to_string(), Value::from(user.email));
    map.insert("balance".to_string(), cents_to_decimal(user.balance));
    map.insert("discount".to_string(), user.discount.map(Value::from).unwrap_or(Value::Null));
    map.insert("commission_type".to_string(), Value::from(user.commission_type));
    map.insert(
        "commission_rate".to_string(),
        user.commission_rate.map(Value::from).unwrap_or(Value::Null),
    );
    map.insert(
        "commission_balance".to_string(),
        cents_to_decimal(user.commission_balance),
    );
    map.insert("t".to_string(), Value::from(user.t));
    map.insert("u".to_string(), Value::from(user.u));
    map.insert("d".to_string(), Value::from(user.d));
    map.insert("total_used".to_string(), Value::from(user.u + user.d));
    map.insert("transfer_enable".to_string(), Value::from(user.transfer_enable));
    map.insert("banned".to_string(), Value::Bool(user.banned != 0));
    map.insert(
        "ban_reason".to_string(),
        user.ban_reason.map(Value::from).unwrap_or(Value::Null),
    );
    map.insert(
        "banned_at".to_string(),
        user.banned_at.map(Value::from).unwrap_or(Value::Null),
    );
    map.insert(
        "banned_by_admin_id".to_string(),
        user.banned_by_admin_id.map(Value::from).unwrap_or(Value::Null),
    );
    map.insert("is_admin".to_string(), Value::Bool(user.is_admin != 0));
    map.insert("is_staff".to_string(), Value::Bool(user.is_staff != 0));
    map.insert(
        "is_super_admin".to_string(),
        Value::Bool(user.is_super_admin != 0),
    );
    map.insert(
        "last_login_at".to_string(),
        user.last_login_at.map(Value::from).unwrap_or(Value::Null),
    );
    map.insert("uuid".to_string(), Value::from(user.uuid));
    map.insert(
        "group_id".to_string(),
        user.group_id.map(Value::from).unwrap_or(Value::Null),
    );
    map.insert(
        "group".to_string(),
        user.group_id
            .map(|id| json!({"id": id, "name": user.group_name.clone().unwrap_or_else(|| format!("Group-{}", id))}))
            .unwrap_or(Value::Null),
    );
    map.insert(
        "plan_id".to_string(),
        user.plan_id.map(Value::from).unwrap_or(Value::Null),
    );
    map.insert(
        "plan".to_string(),
        user.plan_id
            .map(|id| json!({"id": id, "name": user.plan_name.clone().unwrap_or_else(|| format!("Plan-{}", id))}))
            .unwrap_or(Value::Null),
    );
    map.insert(
        "speed_limit".to_string(),
        user.speed_limit.map(Value::from).unwrap_or(Value::Null),
    );
    map.insert(
        "remind_expire".to_string(),
        Value::Bool(user.remind_expire.unwrap_or(1) != 0),
    );
    map.insert(
        "remind_traffic".to_string(),
        Value::Bool(user.remind_traffic.unwrap_or(1) != 0),
    );
    map.insert("token".to_string(), Value::from(user.token));
    map.insert(
        "subscribe_path".to_string(),
        user.subscribe_path.map(Value::from).unwrap_or(Value::Null),
    );
    map.insert(
        "subscribe_key".to_string(),
        user.subscribe_key.map(Value::from).unwrap_or(Value::Null),
    );
    map.insert(
        "subscribe_salt".to_string(),
        user.subscribe_salt.map(Value::from).unwrap_or(Value::Null),
    );
    map.insert("subscribe_url".to_string(), Value::from(subscribe_url));
    map.insert(
        "expired_at".to_string(),
        user.expired_at.map(Value::from).unwrap_or(Value::Null),
    );
    map.insert(
        "remarks".to_string(),
        user.remarks.map(Value::from).unwrap_or(Value::Null),
    );
    map.insert(
        "linux_do_id".to_string(),
        user.linux_do_id.map(Value::from).unwrap_or(Value::Null),
    );
    map.insert(
        "linux_do_username".to_string(),
        user.linux_do_username.map(Value::from).unwrap_or(Value::Null),
    );
    map.insert(
        "linux_do_name".to_string(),
        user.linux_do_name.map(Value::from).unwrap_or(Value::Null),
    );
    map.insert("trust_level".to_string(), Value::from(user.trust_level));
    map.insert("is_silenced".to_string(), Value::Bool(user.is_silenced != 0));
    map.insert(
        "api_key".to_string(),
        user.api_key.map(Value::from).unwrap_or(Value::Null),
    );
    map.insert(
        "device_limit".to_string(),
        user.device_limit.map(Value::from).unwrap_or(Value::Null),
    );
    map.insert(
        "concurrent_ip_limit".to_string(),
        Value::from(if user.concurrent_ip_limit > 0 {
            user.concurrent_ip_limit
        } else {
            3
        }),
    );
    map.insert("created_at".to_string(), Value::from(user.created_at));
    map.insert("updated_at".to_string(), Value::from(user.updated_at));
    Value::Object(map)
}

fn serialize_admin_user_detail(state: &AppState, user: AdminUserRow) -> Value {
    serialize_admin_user_summary(state, user)
}

fn build_subscribe_url_from_admin_row(_state: &AppState, user: &AdminUserRow) -> Option<String> {
    let path = user.subscribe_path.as_deref()?;
    let key = user.subscribe_key.as_deref()?;
    let salt = user.subscribe_salt.as_deref()?;
    let app_url = std::env::var("APP_URL").unwrap_or_default();
    let base = if app_url.trim().is_empty() {
        return Some(format!("/s/{}?{}={}&{}=1", path, key, user.token, salt));
    } else {
        app_url.trim_end_matches('/').to_string()
    };
    Some(format!("{}/s/{}?{}={}&{}=1", base, path, key, user.token, salt))
}

fn cents_to_decimal(value: i64) -> Value {
    Value::from((value as f64) / 100.0)
}

fn push_optional_string_update(payload: &Value, key: &'static str, updates: &mut Vec<(&'static str, Value)>) {
    if let Some(value) = request_optional_string_field(payload, key) {
        if let Some(value) = value {
            updates.push((key, Value::from(value)));
        }
    }
}

fn push_optional_string_nullable_update(payload: &Value, key: &'static str, updates: &mut Vec<(&'static str, Value)>) {
    let Some(object) = payload.as_object() else {
        return;
    };
    if !object.contains_key(key) {
        return;
    }
    let value = object.get(key).cloned().unwrap_or(Value::Null);
    if value.is_null() {
        updates.push((key, Value::Null));
    } else if let Some(text) = value.as_str() {
        updates.push((key, Value::from(text.trim().to_string())));
    }
}

fn push_optional_i64_update(payload: &Value, key: &'static str, updates: &mut Vec<(&'static str, Value)>) {
    if let Some(value) = request_optional_i64_field(payload, key) {
        updates.push((key, value.map(Value::from).unwrap_or(Value::Null)));
    }
}

fn push_optional_bool_update(payload: &Value, key: &'static str, updates: &mut Vec<(&'static str, Value)>) {
    let Some(object) = payload.as_object() else {
        return;
    };
    if !object.contains_key(key) {
        return;
    }
    if let Some(value) = object.get(key).and_then(|value| value.as_bool()) {
        updates.push((key, Value::from(if value { 1 } else { 0 })));
    }
}

fn request_optional_decimal_cents_field(payload: &Value, key: &str) -> Result<Option<i64>, Response<Body>> {
    let Some(object) = payload.as_object() else {
        return Ok(None);
    };
    if !object.contains_key(key) {
        return Ok(None);
    }
    let Some(value) = object.get(key) else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(Some(0));
    }
    let amount = parse_f64_value(value).ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    Ok(Some((amount * 100.0).round() as i64))
}

async fn resolve_invite_user_id(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    payload: &Value,
    user_id: i64,
) -> Result<Option<Option<i64>>, Response<Body>> {
    let Some(object) = payload.as_object() else {
        return Ok(None);
    };
    if object.contains_key("invite_user_email") {
        let invite_email = object
            .get("invite_user_email")
            .and_then(|value| value.as_str())
            .map(|value| value.trim())
            .unwrap_or("");
        if invite_email.is_empty() {
            return Ok(Some(None));
        }
        let invite_user_id: Option<i64> = sqlx::query_scalar("SELECT id FROM v2_user WHERE email = ? AND id <> ? LIMIT 1")
            .bind(invite_email)
            .bind(user_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(internal_error)?;
        return Ok(Some(invite_user_id));
    }
    if object.contains_key("invite_user_id") {
        let invite_user_id = parse_optional_i64_field(object.get("invite_user_id"))?;
        if let Some(invite_user_id) = invite_user_id {
            if invite_user_id == user_id {
                return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "邀请人不能是自己"));
            }
            let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM v2_user WHERE id = ? LIMIT 1")
                .bind(invite_user_id)
                .fetch_optional(&mut **tx)
                .await
                .map_err(internal_error)?;
            if exists.is_none() {
                return Err(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"邀请用户不存在"})));
            }
            return Ok(Some(Some(invite_user_id)));
        }
        return Ok(Some(None));
    }
    Ok(None)
}

async fn apply_user_updates(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    mut updates: Vec<(&'static str, Value)>,
) -> Result<(), Response<Body>> {
    updates.push(("updated_at", Value::from(Utc::now().timestamp())));
    let assignments = updates
        .iter()
        .map(|(field, _)| format!("{field} = ?"))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!("UPDATE v2_user SET {} WHERE id = ?", assignments);
    let mut query = sqlx::query(&sql);
    for (_, value) in updates {
        query = bind_json_value(query, value);
    }
    query
        .bind(user_id)
        .execute(&mut **tx)
        .await
        .map_err(internal_error)?;
    Ok(())
}

fn bind_json_value<'q>(
    query: sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments>,
    value: Value,
) -> sqlx::query::Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments> {
    match value {
        Value::Null => query.bind(None::<String>),
        Value::Bool(flag) => query.bind(if flag { 1_i64 } else { 0_i64 }),
        Value::Number(number) => {
            if let Some(value) = number.as_i64() {
                query.bind(value)
            } else if let Some(value) = number.as_u64() {
                if let Ok(value) = i64::try_from(value) {
                    query.bind(value)
                } else {
                    query.bind(value.to_string())
                }
            } else if let Some(value) = number.as_f64() {
                query.bind(value)
            } else {
                query.bind(number.to_string())
            }
        }
        Value::String(text) => query.bind(text),
        other => query.bind(other.to_string()),
    }
}

async fn apply_ban_updates(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    currently_banned: bool,
    current_reason: Option<&str>,
    target_banned: Option<bool>,
    ban_reason: &str,
    admin_id: i64,
) -> Result<(), Response<Body>> {
    let Some(target_banned) = target_banned else {
        return Ok(());
    };
    let now = Utc::now().timestamp();
    if target_banned {
        let effective_reason = if !ban_reason.trim().is_empty() {
            ban_reason.trim().to_string()
        } else {
            current_reason.unwrap_or_default().trim().to_string()
        };
        if effective_reason.is_empty() {
            return Err(json_status_response(StatusCode::UNPROCESSABLE_ENTITY, json!({"message":"封禁用户时必须填写封禁原因"})));
        }
        if !currently_banned || current_reason.unwrap_or_default().trim() != effective_reason {
            sqlx::query(
                "UPDATE v2_user
                 SET banned = 1, ban_reason = ?, banned_at = ?, banned_by_admin_id = ?, updated_at = ?
                 WHERE id = ?"
            )
            .bind(&effective_reason)
            .bind(now)
            .bind(admin_id)
            .bind(now)
            .bind(user_id)
            .execute(&mut **tx)
            .await
            .map_err(internal_error)?;
            sqlx::query("DELETE FROM personal_access_tokens WHERE tokenable_id = ?")
                .bind(user_id as u64)
                .execute(&mut **tx)
                .await
                .map_err(internal_error)?;
            sqlx::query(
                "INSERT INTO user_ban_records (user_id, admin_id, action, reason, source, context, created_at, updated_at)
                 VALUES (?, ?, 'ban', ?, 'manual', ?, ?, ?)"
            )
            .bind(user_id)
            .bind(admin_id)
            .bind(&effective_reason)
            .bind(r#"{"source":"manual"}"#)
            .bind(now)
            .bind(now)
            .execute(&mut **tx)
            .await
            .map_err(internal_error)?;
        }
    } else if currently_banned {
        let effective_reason = if ban_reason.trim().is_empty() {
            "manual unban".to_string()
        } else {
            ban_reason.trim().to_string()
        };
        sqlx::query(
            "UPDATE v2_user
             SET banned = 0, ban_reason = NULL, banned_at = NULL, banned_by_admin_id = NULL, updated_at = ?
             WHERE id = ?"
        )
        .bind(now)
        .bind(user_id)
        .execute(&mut **tx)
        .await
        .map_err(internal_error)?;
        let context = json!({
            "source": "manual",
            "previous_ban_reason": current_reason.unwrap_or_default(),
        })
        .to_string();
        sqlx::query(
            "INSERT INTO user_ban_records (user_id, admin_id, action, reason, source, context, created_at, updated_at)
             VALUES (?, ?, 'unban', ?, 'manual', ?, ?, ?)"
        )
        .bind(user_id)
        .bind(admin_id)
        .bind(&effective_reason)
        .bind(context)
        .bind(now)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(internal_error)?;
    }
    Ok(())
}

fn format_timestamp(ts: i64) -> String {
    chrono::DateTime::<Utc>::from_timestamp(ts, 0)
        .map(|value| value.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| ts.to_string())
}

fn serialize_ban_record(row: UserBanRecordRow) -> Value {
    let context = row
        .context
        .as_deref()
        .and_then(|value| serde_json::from_str::<Value>(value).ok())
        .unwrap_or(Value::Null);
    json!({
        "id": row.id,
        "user_id": row.user_id,
        "user_email": row.user_email.unwrap_or_else(|| "-".to_string()),
        "admin_id": row.admin_id,
        "admin_email": row.admin_email.unwrap_or_else(|| "system".to_string()),
        "action": row.action,
        "reason": row.reason,
        "source": row.source,
        "context": context,
        "created_at": row.created_at,
    })
}

async fn load_bannable_users(
    state: &AppState,
    payload: &Value,
) -> Result<Vec<BannableUserRow>, Response<Body>> {
    let mut sql = String::from(
        "SELECT u.id, u.banned, u.ban_reason
         FROM v2_user u
         WHERE u.banned = 0",
    );
    let mut binds = Vec::<UserBind>::new();
    if let Some(filters) = payload.get("filter").and_then(|value| value.as_array()) {
        for filter in filters {
            let Some(obj) = filter.as_object() else { continue; };
            let Some(field) = obj.get("id").and_then(|value| value.as_str()) else { continue; };
            append_user_filter_clause(&mut sql, &mut String::new(), &mut binds, field, obj.get("value"));
        }
    }
    sql.push_str(" ORDER BY u.id ASC");

    let mut query = sqlx::query_as::<_, BannableUserRow>(&sql);
    for bind in &binds {
        query = match bind {
            UserBind::I64(value) => query.bind(*value),
            UserBind::String(value) => query.bind(value.clone()),
        };
    }
    query.fetch_all(&state.db).await.map_err(internal_error)
}

async fn load_user_emails_for_mass_action(
    state: &AppState,
    payload: &Value,
) -> Result<Vec<String>, Response<Body>> {
    let mut sql = String::from("SELECT u.email FROM v2_user u WHERE u.email IS NOT NULL AND u.email <> ''");
    let mut binds = Vec::<UserBind>::new();
    if let Some(filters) = payload.get("filter").and_then(|value| value.as_array()) {
        for filter in filters {
            let Some(obj) = filter.as_object() else { continue; };
            let Some(field) = obj.get("id").and_then(|value| value.as_str()) else { continue; };
            append_user_filter_clause(&mut sql, &mut String::new(), &mut binds, field, obj.get("value"));
        }
    }
    append_user_sort_clause(&mut sql, payload.get("sort").and_then(|value| value.as_str()), payload.get("sort_type").and_then(|value| value.as_str()));

    let mut query = sqlx::query_scalar::<_, String>(&sql);
    for bind in &binds {
        query = match bind {
            UserBind::I64(value) => query.bind(*value),
            UserBind::String(value) => query.bind(value.clone()),
        };
    }
    query.fetch_all(&state.db).await.map_err(internal_error)
}

async fn load_users_for_csv(
    state: &AppState,
    payload: &Value,
) -> Result<Vec<AdminUserRow>, Response<Body>> {
    let mut sql = String::from(
        "SELECT
            u.id, u.invite_user_id, u.telegram_id, u.email, u.balance, u.discount, u.commission_type, u.commission_rate,
            u.commission_balance, u.t, u.u, u.d, u.transfer_enable, u.banned, u.ban_reason, u.banned_at,
            u.banned_by_admin_id, u.is_admin, u.is_staff, u.is_super_admin, u.last_login_at, u.uuid, u.group_id,
            u.plan_id, u.speed_limit, u.remind_expire, u.remind_traffic, u.token, u.subscribe_path, u.subscribe_key,
            u.subscribe_salt, u.expired_at, u.remarks, u.linux_do_id, u.linux_do_username, u.linux_do_name,
            u.trust_level, u.is_silenced, u.api_key, u.device_limit, u.concurrent_ip_limit, u.created_at, u.updated_at,
            p.name AS plan_name, g.name AS group_name, iu.email AS invite_user_email
         FROM v2_user u
         LEFT JOIN v2_plan p ON p.id = u.plan_id
         LEFT JOIN v2_server_group g ON g.id = u.group_id
         LEFT JOIN v2_user iu ON iu.id = u.invite_user_id
         WHERE 1=1"
    );
    let mut binds = Vec::<UserBind>::new();
    if let Some(filters) = payload.get("filter").and_then(|value| value.as_array()) {
        for filter in filters {
            let Some(obj) = filter.as_object() else { continue; };
            let Some(field) = obj.get("id").and_then(|value| value.as_str()) else { continue; };
            append_user_filter_clause(&mut sql, &mut String::new(), &mut binds, field, obj.get("value"));
        }
    }
    append_user_sort_clause(&mut sql, payload.get("sort").and_then(|value| value.as_str()), payload.get("sort_type").and_then(|value| value.as_str()));

    let mut query = sqlx::query_as::<_, AdminUserRow>(&sql);
    for bind in &binds {
        query = match bind {
            UserBind::I64(value) => query.bind(*value),
            UserBind::String(value) => query.bind(value.clone()),
        };
    }
    query.fetch_all(&state.db).await.map_err(internal_error)
}

fn append_user_sort_clause(sql: &mut String, sort_field: Option<&str>, sort_type: Option<&str>) {
    let field = sort_field
        .filter(|value| SORTABLE_USER_FIELDS.contains(value))
        .unwrap_or("created_at");
    let direction = match sort_type.unwrap_or("DESC").to_ascii_uppercase().as_str() {
        "ASC" => "ASC",
        _ => "DESC",
    };
    if field == "total_used" {
        sql.push_str(&format!(" ORDER BY (u.u + u.d) {}", direction));
    } else {
        sql.push_str(&format!(" ORDER BY u.{} {}", field, direction));
    }
}


fn sanitize_for_csv(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    let trimmed = value.trim_start();
    if let Some(first) = trimmed.chars().next() {
        if matches!(first, '=' | '+' | '-' | '@' | '\t') {
            return format!("'{}", value);
        }
    }
    value.to_string()
}

fn traffic_convert(byte: i64) -> String {
    if byte < 0 {
        return "0".to_string();
    }
    let byte = byte as f64;
    let kb = 1024.0;
    let mb = 1048576.0;
    let gb = 1073741824.0;
    if byte > gb {
        format!("{:.2} GB", byte / gb)
    } else if byte > mb {
        format!("{:.2} MB", byte / mb)
    } else if byte > kb {
        format!("{:.2} KB", byte / kb)
    } else {
        format!("{:.2} B", byte)
    }
}
