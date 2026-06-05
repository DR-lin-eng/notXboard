use super::super::*;

#[derive(Clone, sqlx::FromRow)]
struct AdminOrderRow {
    id: i64,
    user_id: i64,
    plan_id: i64,
    payment_id: Option<i64>,
    period: String,
    trade_no: String,
    total_amount: i64,
    handling_amount: Option<i64>,
    balance_amount: Option<i64>,
    refund_amount: Option<i64>,
    surplus_amount: Option<i64>,
    discount_amount: Option<i64>,
    type_field: i64,
    status: i64,
    surplus_order_ids: Option<String>,
    coupon_id: Option<i64>,
    created_at: i64,
    updated_at: i64,
    commission_status: i64,
    invite_user_id: Option<i64>,
    actual_commission_balance: Option<i64>,
    commission_balance: i64,
    paid_at: Option<i64>,
    callback_no: Option<String>,
    plan_name: Option<String>,
    plan_scope: Option<String>,
    user_email: Option<String>,
    invite_user_email: Option<String>,
}

#[derive(Clone, sqlx::FromRow)]
struct AssignUserRow {
    id: i64,
    email: String,
    invite_user_id: Option<i64>,
}

const FILTERABLE_ORDER_FIELDS: &[&str] = &[
    "id",
    "user_id",
    "plan_id",
    "payment_id",
    "period",
    "trade_no",
    "total_amount",
    "status",
    "coupon_id",
    "commission_status",
    "invite_user_id",
    "paid_at",
    "callback_no",
    "created_at",
    "updated_at",
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

pub async fn detail(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_detail_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn paid(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_paid_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn cancel(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_cancel_response(&state, headers, uri, body).await {
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

pub async fn assign(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_assign_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_fetch_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await.unwrap_or_else(|_| json!({}));
    let current = payload.get("current").and_then(parse_i64_value).filter(|v| *v > 0).unwrap_or(1);
    let page_size = payload.get("pageSize").and_then(parse_i64_value).filter(|v| *v > 0).unwrap_or(10);
    let is_commission = payload.get("is_commission").and_then(|value| value.as_bool()).unwrap_or(false);

    let mut sql = String::from(
        "SELECT o.id, o.user_id, o.plan_id, o.payment_id, o.period, o.trade_no, o.total_amount, o.handling_amount,
                o.balance_amount, o.refund_amount, o.surplus_amount, o.discount_amount, o.type AS type_field,
                o.status, o.surplus_order_ids, o.coupon_id, o.created_at, o.updated_at, o.commission_status,
                o.invite_user_id, o.actual_commission_balance, o.commission_balance, o.paid_at, o.callback_no,
                p.name AS plan_name, p.scope AS plan_scope, u.email AS user_email, iu.email AS invite_user_email
         FROM v2_order o
         LEFT JOIN v2_plan p ON p.id = o.plan_id
         LEFT JOIN v2_user u ON u.id = o.user_id
         LEFT JOIN v2_user iu ON iu.id = o.invite_user_id
         WHERE 1=1"
    );
    if is_commission {
        sql.push_str(" AND o.invite_user_id IS NOT NULL AND o.status NOT IN (0, 2) AND o.commission_balance > 0");
    }
    let mut total_sql = String::from("SELECT COUNT(*) FROM v2_order o WHERE 1=1");
    if is_commission {
        total_sql.push_str(" AND o.invite_user_id IS NOT NULL AND o.status NOT IN (0, 2) AND o.commission_balance > 0");
    }

    let mut binds = Vec::<OrderBind>::new();
    if let Some(filters) = payload.get("filter").and_then(|value| value.as_array()) {
        for filter in filters {
            let Some(obj) = filter.as_object() else { continue; };
            let Some(field) = obj.get("id").and_then(|value| value.as_str()).filter(|field| FILTERABLE_ORDER_FIELDS.contains(field)) else { continue; };
            append_order_filter_clause(&mut sql, &mut total_sql, &mut binds, field, obj.get("value"));
        }
    }

    let order_by = parse_order_sorts(payload.get("sort").and_then(|value| value.as_array()));
    sql.push_str(" ORDER BY ");
    if order_by.is_empty() {
        sql.push_str("o.created_at DESC");
    } else {
        sql.push_str(&order_by.join(", "));
    }
    sql.push_str(" LIMIT ? OFFSET ?");
    info!(
        current,
        page_size,
        is_commission,
        has_filter = payload.get("filter").is_some(),
        has_sort = payload.get("sort").is_some(),
        sql = %sql,
        total_sql = %total_sql,
        binds = ?binds,
        "admin order fetch query"
    );

    let mut total_query = sqlx::query_scalar::<_, i64>(&total_sql);
    for bind in &binds {
        total_query = match bind {
            OrderBind::I64(value) => total_query.bind(*value),
            OrderBind::String(value) => total_query.bind(value.clone()),
        };
    }
    let total = total_query.fetch_one(&state.db).await.map_err(internal_error)?;

    let mut query = sqlx::query_as::<_, AdminOrderRow>(&sql);
    for bind in &binds {
        query = match bind {
            OrderBind::I64(value) => query.bind(*value),
            OrderBind::String(value) => query.bind(value.clone()),
        };
    }
    let rows = query
        .bind(page_size)
        .bind((current - 1) * page_size)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;

    Ok(json_value_response(json!({
        "data": rows.into_iter().map(serialize_admin_order_summary).collect::<Vec<_>>(),
        "total": total
    })))
}

async fn build_detail_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let id = payload.get("id").and_then(parse_i64_value).filter(|v| *v > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let order = load_admin_order_by_id(state, id).await.map_err(internal_error)?;
    let Some(order) = order else {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"订单不存在"})));
    };

    let surplus_orders = if let Some(ids) = parse_order_id_list(order.surplus_order_ids.as_deref().unwrap_or_default()) {
        load_orders_by_ids_admin(state, &ids).await.map_err(internal_error)?
    } else {
        Vec::new()
    };

    let mut value = serialize_admin_order_detail(&order);
    if let Some(object) = value.as_object_mut() {
        object.insert(
            "surplus_orders".to_string(),
            Value::Array(surplus_orders.into_iter().map(serialize_admin_order_summary).collect()),
        );
    }
    Ok(json_value_response(success_response_payload(value)))
}

async fn build_paid_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let trade_no = payload.get("trade_no").and_then(|value| value.as_str()).map(str::trim).filter(|v| !v.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let updated = sqlx::query(
        "UPDATE v2_order
         SET status = 3, paid_at = COALESCE(paid_at, ?), updated_at = ?
         WHERE trade_no = ? AND status = 0"
    )
    .bind(Utc::now().timestamp())
    .bind(Utc::now().timestamp())
    .bind(trade_no)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    if updated.rows_affected() == 0 {
        let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM v2_order WHERE trade_no = ? LIMIT 1")
            .bind(trade_no)
            .fetch_optional(&state.db)
            .await
            .map_err(internal_error)?;
        if exists.is_none() {
            return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"订单不存在"})));
        }
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"只能对待支付的订单进行操作"})));
    }

    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_cancel_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let trade_no = payload.get("trade_no").and_then(|value| value.as_str()).map(str::trim).filter(|v| !v.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let updated = sqlx::query(
        "UPDATE v2_order
         SET status = 2, updated_at = ?
         WHERE trade_no = ? AND status = 0"
    )
    .bind(Utc::now().timestamp())
    .bind(trade_no)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    if updated.rows_affected() == 0 {
        let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM v2_order WHERE trade_no = ? LIMIT 1")
            .bind(trade_no)
            .fetch_optional(&state.db)
            .await
            .map_err(internal_error)?;
        if exists.is_none() {
            return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"订单不存在"})));
        }
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"只能对待支付的订单进行操作"})));
    }

    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_update_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let trade_no = payload.get("trade_no").and_then(|value| value.as_str()).map(str::trim).filter(|v| !v.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    let commission_status = payload.get("commission_status").and_then(parse_i64_value)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    if ![0_i64, 1_i64, 3_i64].contains(&commission_status) {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    }

    let updated = sqlx::query("UPDATE v2_order SET commission_status = ?, updated_at = ? WHERE trade_no = ?")
        .bind(commission_status)
        .bind(Utc::now().timestamp())
        .bind(trade_no)
        .execute(&state.db)
        .await
        .map_err(|_| json_status_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"message":"更新失败"})))?;
    if updated.rows_affected() == 0 {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"订单不存在"})));
    }

    Ok(json_value_response(success_response_payload(Value::Bool(true))))
}

async fn build_assign_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let plan_id = payload.get("plan_id").and_then(parse_i64_value).filter(|v| *v > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "订阅不能为空"))?;
    let email = payload.get("email").and_then(|value| value.as_str()).map(str::trim).filter(|v| !v.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "邮箱不能为空"))?;
    let total_amount = payload.get("total_amount").and_then(parse_i64_value)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "支付金额不能为空"))?;
    let period = payload.get("period").and_then(|value| value.as_str()).map(str::trim).filter(|v| !v.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "订阅周期不能为空"))?;
    let period_key = normalize_assign_period(period)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "订阅周期格式有误"))?;

    let user = load_user_by_email_for_assign(state, email).await.map_err(internal_error)?;
    let Some(user) = user else {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"该用户不存在"})));
    };
    let plan = load_order_plan_by_id(state, plan_id).await.map_err(internal_error)?;
    let Some(plan) = plan else {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"该订阅不存在"})));
    };
    let open_order_exists: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM v2_order WHERE user_id = ? AND status IN (0, 1)")
        .bind(user.id)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;
    if open_order_exists > 0 {
        return Ok(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"该用户还有待支付的订单，无法分配"})));
    }

    let trade_no = random_hex(32);
    let commission_balance = compute_assign_commission_balance(&state.db, &user, total_amount).await?;
    sqlx::query(
        "INSERT INTO v2_order
            (invite_user_id, user_id, plan_id, total_amount, type, period, trade_no, status, commission_balance, commission_status, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, 0, ?, ?)"
    )
    .bind(user.invite_user_id)
    .bind(user.id)
    .bind(plan.id)
    .bind(total_amount)
    .bind(determine_order_type_for_assign(&state.db, user.id, plan.id).await?)
    .bind(period_key)
    .bind(&trade_no)
    .bind(commission_balance)
    .bind(Utc::now().timestamp())
    .bind(Utc::now().timestamp())
    .execute(&state.db)
    .await
    .map_err(|_| json_status_response(StatusCode::INTERNAL_SERVER_ERROR, json!({"message":"订单创建失败"})))?;

    Ok(json_value_response(success_response_payload(Value::String(trade_no))))
}

async fn load_admin_orders_by_ids(
    state: &AppState,
    ids: &[i64],
) -> Result<Vec<AdminOrderRow>, sqlx::Error> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = vec!["?"; ids.len()].join(",");
    let sql = format!(
        "SELECT o.id, o.user_id, o.plan_id, o.payment_id, o.period, o.trade_no, o.total_amount, o.handling_amount,
                o.balance_amount, o.refund_amount, o.surplus_amount, o.discount_amount, o.type AS type_field,
                o.status, o.surplus_order_ids, o.coupon_id, o.created_at, o.updated_at, o.commission_status,
                o.invite_user_id, o.actual_commission_balance, o.commission_balance, o.paid_at, o.callback_no,
                p.name AS plan_name, p.scope AS plan_scope, u.email AS user_email, iu.email AS invite_user_email
         FROM v2_order o
         LEFT JOIN v2_plan p ON p.id = o.plan_id
         LEFT JOIN v2_user u ON u.id = o.user_id
         LEFT JOIN v2_user iu ON iu.id = o.invite_user_id
         WHERE o.id IN ({})",
        placeholders
    );
    let mut query = sqlx::query_as::<_, AdminOrderRow>(&sql);
    for id in ids {
        query = query.bind(*id);
    }
    query.fetch_all(&state.db).await
}

async fn load_orders_by_ids_admin(
    state: &AppState,
    ids: &[i64],
) -> Result<Vec<AdminOrderRow>, sqlx::Error> {
    load_admin_orders_by_ids(state, ids).await
}

async fn load_admin_order_by_id(
    state: &AppState,
    order_id: i64,
) -> Result<Option<AdminOrderRow>, sqlx::Error> {
    sqlx::query_as::<_, AdminOrderRow>(
        "SELECT o.id, o.user_id, o.plan_id, o.payment_id, o.period, o.trade_no, o.total_amount, o.handling_amount,
                o.balance_amount, o.refund_amount, o.surplus_amount, o.discount_amount, o.type AS type_field,
                o.status, o.surplus_order_ids, o.coupon_id, o.created_at, o.updated_at, o.commission_status,
                o.invite_user_id, o.actual_commission_balance, o.commission_balance, o.paid_at, o.callback_no,
                p.name AS plan_name, p.scope AS plan_scope, u.email AS user_email, iu.email AS invite_user_email
         FROM v2_order o
         LEFT JOIN v2_plan p ON p.id = o.plan_id
         LEFT JOIN v2_user u ON u.id = o.user_id
         LEFT JOIN v2_user iu ON iu.id = o.invite_user_id
         WHERE o.id = ?
         LIMIT 1"
    )
    .bind(order_id)
    .fetch_optional(&state.db)
    .await
}

async fn load_user_by_email_for_assign(
    state: &AppState,
    email: &str,
) -> Result<Option<AssignUserRow>, sqlx::Error> {
    sqlx::query_as::<_, AssignUserRow>(
        "SELECT id, email, invite_user_id
         FROM v2_user
         WHERE email = ?
         LIMIT 1"
    )
    .bind(email)
    .fetch_optional(&state.db)
    .await
}

async fn determine_order_type_for_assign(
    db: &MySqlPool,
    user_id: i64,
    plan_id: i64,
) -> Result<i64, Response<Body>> {
    let active_subscription: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM user_plan_subscriptions
         WHERE user_id = ? AND plan_id = ? AND (expired_at IS NULL OR expired_at > ?)"
    )
    .bind(user_id)
    .bind(plan_id)
    .bind(Utc::now().timestamp())
    .fetch_one(db)
    .await
    .map_err(internal_error)?;
    Ok(if active_subscription > 0 { 2 } else { 1 })
}

async fn compute_assign_commission_balance(
    db: &MySqlPool,
    user: &AssignUserRow,
    total_amount: i64,
) -> Result<i64, Response<Body>> {
    let Some(invite_user_id) = user.invite_user_id else {
        return Ok(0);
    };

    let inviter = sqlx::query(
        "SELECT commission_type, commission_rate
         FROM v2_user
         WHERE id = ?
         LIMIT 1"
    )
    .bind(invite_user_id)
    .fetch_optional(db)
    .await
    .map_err(internal_error)?;
    let Some(inviter) = inviter else {
        return Ok(0);
    };

    let commission_type = inviter.try_get::<Option<i64>, _>("commission_type").ok().flatten().unwrap_or(0);
    let commission_rate = inviter.try_get::<Option<f64>, _>("commission_rate").ok().flatten().unwrap_or(0.0);
    let is_first_paid_order: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM v2_order
         WHERE user_id = ? AND status NOT IN (0, 2)"
    )
    .bind(user.id)
    .fetch_one(db)
    .await
    .map_err(internal_error)?;

    let should_commission = match commission_type {
        1 => is_first_paid_order == 0,
        2 => true,
        _ => false,
    };
    if !should_commission {
        return Ok(0);
    }

    let rate = if commission_rate > 0.0 { commission_rate } else { 10.0 };
    Ok(((total_amount as f64) * (rate / 100.0)).round() as i64)
}

fn normalize_assign_period(period: &str) -> Option<&'static str> {
    match period {
        "month_price" => Some("monthly"),
        "quarter_price" => Some("quarterly"),
        "half_year_price" => Some("half_yearly"),
        "year_price" => Some("yearly"),
        "two_year_price" => Some("two_yearly"),
        "three_year_price" => Some("three_yearly"),
        "onetime_price" => Some("onetime"),
        "reset_price" => Some("reset_traffic"),
        _ => None,
    }
}

#[derive(Debug)]
enum OrderBind {
    I64(i64),
    String(String),
}

fn append_order_filter_clause(
    sql: &mut String,
    total_sql: &mut String,
    binds: &mut Vec<OrderBind>,
    field: &str,
    value: Option<&Value>,
) {
    let Some(value) = value else { return; };
    let db_field = format!("o.{}", field);
    match value {
        Value::Array(items) => {
            let ids = items.iter().filter_map(parse_i64_value).collect::<Vec<_>>();
            if ids.is_empty() {
                return;
            }
            let placeholders = vec!["?"; ids.len()].join(",");
            let clause = format!(" AND {} IN ({})", db_field, placeholders);
            sql.push_str(&clause);
            total_sql.push_str(&clause);
            for id in ids {
                binds.push(OrderBind::I64(id));
            }
        }
        Value::String(text) if text.contains(':') => {
            let mut parts = text.splitn(2, ':');
            let operator = parts.next().unwrap_or_default().to_ascii_lowercase();
            let raw = parts.next().unwrap_or_default().trim().to_string();
            match operator.as_str() {
                "null" => {
                    let clause = format!(" AND {} IS NULL", db_field);
                    sql.push_str(&clause);
                    total_sql.push_str(&clause);
                }
                "notnull" => {
                    let clause = format!(" AND {} IS NOT NULL", db_field);
                    sql.push_str(&clause);
                    total_sql.push_str(&clause);
                }
                "eq" | "gt" | "gte" | "lt" | "lte" => {
                    let op = match operator.as_str() {
                        "eq" => "=",
                        "gt" => ">",
                        "gte" => ">=",
                        "lt" => "<",
                        "lte" => "<=",
                        _ => "=",
                    };
                    let clause = format!(" AND {} {} ?", db_field, op);
                    sql.push_str(&clause);
                    total_sql.push_str(&clause);
                    if let Ok(num) = raw.parse::<i64>() {
                        binds.push(OrderBind::I64(num));
                    } else {
                        binds.push(OrderBind::String(raw));
                    }
                }
                "like" | "notlike" => {
                    let op = if operator == "like" { "LIKE" } else { "NOT LIKE" };
                    let clause = format!(" AND {} {} ?", db_field, op);
                    sql.push_str(&clause);
                    total_sql.push_str(&clause);
                    binds.push(OrderBind::String(format!("%{}%", raw)));
                }
                _ => {}
            }
        }
        Value::String(text) => {
            let clause = format!(" AND {} LIKE ?", db_field);
            sql.push_str(&clause);
            total_sql.push_str(&clause);
            binds.push(OrderBind::String(format!("%{}%", text)));
        }
        Value::Number(number) => {
            if let Some(num) = number.as_i64() {
                let clause = format!(" AND {} = ?", db_field);
                sql.push_str(&clause);
                total_sql.push_str(&clause);
                binds.push(OrderBind::I64(num));
            }
        }
        _ => {}
    }
}

fn parse_order_sorts(sort_values: Option<&Vec<Value>>) -> Vec<String> {
    let Some(sort_values) = sort_values else { return Vec::new(); };
    let mut order_by = Vec::new();
    for sort in sort_values {
        let Some(obj) = sort.as_object() else { continue; };
        let Some(field) = obj.get("id").and_then(|value| value.as_str()).filter(|field| FILTERABLE_ORDER_FIELDS.contains(field)) else { continue; };
        let desc = obj.get("desc").and_then(|value| value.as_bool()).unwrap_or(false);
        order_by.push(format!("o.{} {}", field, if desc { "DESC" } else { "ASC" }));
    }
    order_by
}


fn serialize_admin_order_summary(order: AdminOrderRow) -> Value {
    json!({
        "id": order.id,
        "user_id": order.user_id,
        "plan_id": order.plan_id,
        "payment_id": order.payment_id,
        "period": legacy_period(&order.period),
        "trade_no": order.trade_no,
        "total_amount": order.total_amount,
        "handling_amount": order.handling_amount,
        "balance_amount": order.balance_amount,
        "refund_amount": order.refund_amount,
        "surplus_amount": order.surplus_amount,
        "discount_amount": order.discount_amount,
        "type": order.type_field,
        "status": order.status,
        "surplus_order_ids": order.surplus_order_ids,
        "coupon_id": order.coupon_id,
        "created_at": order.created_at,
        "updated_at": order.updated_at,
        "commission_status": order.commission_status,
        "invite_user_id": order.invite_user_id,
        "invite_user": order.invite_user_email.as_ref().map(|email| json!({"email": email})).unwrap_or(Value::Null),
        "actual_commission_balance": order.actual_commission_balance,
        "commission_balance": order.commission_balance,
        "paid_at": order.paid_at,
        "callback_no": order.callback_no,
        "user": order.user_email.as_ref().map(|email| json!({"email": email})).unwrap_or(Value::Null),
        "plan": order.plan_name.as_ref().map(|name| json!({
            "id": order.plan_id,
            "name": name,
            "scope": order.plan_scope.clone().unwrap_or_else(|| "legacy".to_string()),
        })).unwrap_or(Value::Null),
    })
}

fn serialize_admin_order_detail(order: &AdminOrderRow) -> Value {
    serialize_admin_order_summary(order.clone())
}

fn legacy_period(period: &str) -> String {
    match period {
        "monthly" => "month_price",
        "quarterly" => "quarter_price",
        "half_yearly" => "half_year_price",
        "yearly" => "year_price",
        "two_yearly" => "two_year_price",
        "three_yearly" => "three_year_price",
        "onetime" => "onetime_price",
        "reset_traffic" => "reset_price",
        _ => period,
    }
    .to_string()
}

fn parse_order_id_list(raw: &str) -> Option<Vec<i64>> {
    if raw.trim().is_empty() {
        return None;
    }
    serde_json::from_str::<Vec<i64>>(raw).ok()
}
