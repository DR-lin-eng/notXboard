use super::super::*;

#[derive(Clone, sqlx::FromRow)]
struct AdminStatUserRow {
    id: i64,
    user_id: i64,
    server_rate: f64,
    u: i64,
    d: i64,
    record_type: String,
    record_at: i64,
    created_at: i64,
    updated_at: i64,
}

pub async fn get_stat_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_stat_user_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_override(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_override_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_stats(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_stats_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_order(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_order_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_traffic_rank(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_traffic_rank_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_server_last_rank(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_server_rank_response(&state, headers, uri, "today").await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_server_yesterday_rank(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_server_rank_response(&state, headers, uri, "yesterday").await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_stat_record(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_stat_record_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub async fn get_ranking(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
) -> Response<Body> {
    match build_get_ranking_response(&state, headers, uri).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

#[derive(Clone, sqlx::FromRow)]
struct AdminStatOrderRow {
    record_at: i64,
    paid_total: i64,
    paid_count: i64,
    commission_total: i64,
    commission_count: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct TrafficRankRow {
    id: i64,
    value: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct ServerRankRow {
    server_id: i64,
    server_type: String,
    server_name: Option<String>,
    u: i64,
    d: i64,
    total: i64,
}

async fn build_get_stat_user_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let user_id = params
        .get("user_id")
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;
    let page_size = params
        .get("pageSize")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(10)
        .clamp(1, 200);
    let page = params
        .get("page")
        .or_else(|| params.get("current"))
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(1)
        .max(1);
    let offset = (page - 1) * page_size;

    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM v2_stat_user WHERE user_id = ?")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
        .map_err(internal_error)?;

    let rows = sqlx::query_as::<_, AdminStatUserRow>(
        "SELECT id, user_id, CAST(server_rate AS DOUBLE) AS server_rate, u, d, record_type, record_at, created_at, updated_at
         FROM v2_stat_user
         WHERE user_id = ?
         ORDER BY record_at DESC
         LIMIT ? OFFSET ?"
    )
    .bind(user_id)
    .bind(page_size)
    .bind(offset)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(json_value_response(json!({
        "data": rows.into_iter().map(serialize_stat_user_row).collect::<Vec<_>>(),
        "total": total,
    })))
}

fn serialize_stat_user_row(row: AdminStatUserRow) -> Value {
    json!({
        "id": row.id,
        "user_id": row.user_id,
        "server_rate": row.server_rate,
        "u": row.u,
        "d": row.d,
        "record_type": row.record_type,
        "record_at": row.record_at,
        "created_at": row.created_at,
        "updated_at": row.updated_at,
    })
}

async fn build_get_order_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let start_date = params.get("start_date").cloned();
    let end_date = params.get("end_date").cloned();
    let stat_type = params.get("type").cloned().filter(|value| matches!(value.as_str(), "paid_total" | "paid_count" | "commission_total" | "commission_count"));

    let mut sql = String::from(
        "SELECT
            record_at,
            CAST(COALESCE(paid_total,0) AS SIGNED) AS paid_total,
            CAST(COALESCE(paid_count,0) AS SIGNED) AS paid_count,
            CAST(COALESCE(commission_total,0) AS SIGNED) AS commission_total,
            CAST(COALESCE(commission_count,0) AS SIGNED) AS commission_count
         FROM v2_stat
         WHERE record_type = 'd'"
    );
    let mut binds = Vec::<i64>::new();
    if let Some(start_date) = start_date.as_deref() {
        if let Some(ts) = parse_ymd_start_ts(start_date) {
            sql.push_str(" AND record_at >= ?");
            binds.push(ts);
        }
    }
    if let Some(end_date) = end_date.as_deref() {
        if let Some(ts) = parse_ymd_end_ts(end_date) {
            sql.push_str(" AND record_at <= ?");
            binds.push(ts);
        }
    }
    sql.push_str(" ORDER BY record_at DESC");

    let mut query = sqlx::query_as::<_, AdminStatOrderRow>(&sql);
    for bind in &binds {
        query = query.bind(*bind);
    }
    let rows = query.fetch_all(&state.db).await.map_err(internal_error)?;

    let default_start = rows.last().map(|row| format_timestamp_day(row.record_at)).unwrap_or_default();
    let default_end = rows.first().map(|row| format_timestamp_day(row.record_at)).unwrap_or_default();
    let mut summary = json!({
        "paid_total": 0,
        "paid_count": 0,
        "commission_total": 0,
        "commission_count": 0,
        "start_date": start_date.clone().unwrap_or(default_start),
        "end_date": end_date.clone().unwrap_or(default_end),
        "avg_paid_amount": 0.0,
        "avg_commission_amount": 0.0,
        "commission_rate": 0.0,
    });

    let mut daily_stats = Vec::new();
    let mut paid_total_sum = 0_i64;
    let mut paid_count_sum = 0_i64;
    let mut commission_total_sum = 0_i64;
    let mut commission_count_sum = 0_i64;

    for row in rows.iter() {
        let date = format_timestamp_day(row.record_at);
        paid_total_sum += row.paid_total;
        paid_count_sum += row.paid_count;
        commission_total_sum += row.commission_total;
        commission_count_sum += row.commission_count;
        let avg_order_amount = if row.paid_count > 0 {
            ((row.paid_total as f64) / (row.paid_count as f64) * 100.0).round() / 100.0
        } else {
            0.0
        };
        let avg_commission_amount = if row.commission_count > 0 {
            ((row.commission_total as f64) / (row.commission_count as f64) * 100.0).round() / 100.0
        } else {
            0.0
        };

        if let Some(stat_type) = stat_type.as_deref() {
            let value = match stat_type {
                "paid_total" => row.paid_total,
                "paid_count" => row.paid_count,
                "commission_total" => row.commission_total,
                "commission_count" => row.commission_count,
                _ => 0,
            };
            daily_stats.push(json!({
                "date": date,
                "value": value,
                "type": stat_type_label(stat_type),
            }));
        } else {
            daily_stats.push(json!({
                "date": date,
                "paid_total": row.paid_total,
                "paid_count": row.paid_count,
                "commission_total": row.commission_total,
                "commission_count": row.commission_count,
                "avg_order_amount": avg_order_amount,
                "avg_commission_amount": avg_commission_amount,
            }));
        }
    }

    if let Some(summary_obj) = summary.as_object_mut() {
        summary_obj.insert("paid_total".to_string(), Value::from(paid_total_sum));
        summary_obj.insert("paid_count".to_string(), Value::from(paid_count_sum));
        summary_obj.insert("commission_total".to_string(), Value::from(commission_total_sum));
        summary_obj.insert("commission_count".to_string(), Value::from(commission_count_sum));
        summary_obj.insert(
            "avg_paid_amount".to_string(),
            Value::from(if paid_count_sum > 0 {
                ((paid_total_sum as f64) / (paid_count_sum as f64) * 100.0).round() / 100.0
            } else {
                0.0
            }),
        );
        summary_obj.insert(
            "avg_commission_amount".to_string(),
            Value::from(if commission_count_sum > 0 {
                ((commission_total_sum as f64) / (commission_count_sum as f64) * 100.0).round() / 100.0
            } else {
                0.0
            }),
        );
        summary_obj.insert(
            "commission_rate".to_string(),
            Value::from(if paid_total_sum > 0 {
                (((commission_total_sum as f64) / (paid_total_sum as f64)) * 100.0 * 100.0).round() / 100.0
            } else {
                0.0
            }),
        );
    }

    daily_stats.reverse();
    Ok(json_value_response(json!({
        "code": 0,
        "message": "success",
        "data": {
            "list": daily_stats,
            "summary": summary,
        }
    })))
}

async fn build_get_traffic_rank_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let rank_type = params
        .get("type")
        .map(|value| value.as_str())
        .unwrap_or("node");
    if !matches!(rank_type, "node" | "user") {
        return Ok(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    }
    let start_at = params
        .get("start_time")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or_else(|| Utc::now().timestamp() - 7 * 86_400);
    let end_at = params
        .get("end_time")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or_else(|| Utc::now().timestamp());
    let limit = params
        .get("limit")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(10)
        .clamp(1, 100);
    let previous_start_at = start_at - (end_at - start_at);
    let previous_end_at = start_at;

    let (current_sql, previous_sql) = if rank_type == "node" {
        (
            "SELECT server_id AS id, CAST(COALESCE(SUM(u + d),0) AS SIGNED) AS value
             FROM v2_stat_server
             WHERE record_at >= ? AND record_at <= ?
             GROUP BY server_id
             ORDER BY value DESC
             LIMIT ?",
            "SELECT server_id AS id, CAST(COALESCE(SUM(u + d),0) AS SIGNED) AS value
             FROM v2_stat_server
             WHERE record_at >= ? AND record_at < ?
             GROUP BY server_id",
        )
    } else {
        (
            "SELECT user_id AS id, CAST(COALESCE(SUM(u + d),0) AS SIGNED) AS value
             FROM v2_stat_user
             WHERE record_at >= ? AND record_at <= ?
             GROUP BY user_id
             ORDER BY value DESC
             LIMIT ?",
            "SELECT user_id AS id, CAST(COALESCE(SUM(u + d),0) AS SIGNED) AS value
             FROM v2_stat_user
             WHERE record_at >= ? AND record_at < ?
             GROUP BY user_id",
        )
    };

    let current_rows = sqlx::query_as::<_, TrafficRankRow>(current_sql)
        .bind(start_at)
        .bind(end_at)
        .bind(limit)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;
    let ids = current_rows.iter().map(|row| row.id).collect::<Vec<_>>();
    let name_map = load_traffic_rank_names(state, rank_type, &ids)
        .await
        .map_err(internal_error)?;
    let previous_rows = sqlx::query_as::<_, TrafficRankRow>(previous_sql)
        .bind(previous_start_at)
        .bind(previous_end_at)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;
    let previous_map = previous_rows.into_iter().map(|row| (row.id, row.value)).collect::<HashMap<_, _>>();

    let mut data = Vec::new();
    for row in current_rows {
        let previous_value = *previous_map.get(&row.id).unwrap_or(&0);
        let change = if previous_value > 0 {
            (((row.value - previous_value) as f64) / (previous_value as f64) * 1000.0).round() / 10.0
        } else {
            0.0
        };
        let name = if rank_type == "node" {
            name_map
                .get(&row.id)
                .cloned()
                .unwrap_or_else(|| format!("Node {}", row.id))
        } else {
            name_map
                .get(&row.id)
                .cloned()
                .unwrap_or_else(|| format!("User {}", row.id))
        };
        data.push(json!({
            "id": row.id.to_string(),
            "name": name,
            "value": row.value,
            "previousValue": previous_value,
            "change": change,
            "timestamp": chrono::DateTime::<Utc>::from_timestamp(end_at, 0).map(|dt| dt.to_rfc3339()).unwrap_or_default(),
        }));
    }

    Ok(json_value_response(json!({
        "timestamp": Utc::now().to_rfc3339(),
        "data": data,
    })))
}

async fn build_get_server_rank_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    mode: &str,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let (start_at, end_at) = server_rank_time_range(mode);
    let rows = sqlx::query_as::<_, ServerRankRow>(
        "SELECT
            ss.server_id,
            ss.server_type,
            COALESCE(parent.name, s.name) AS server_name,
            CAST(COALESCE(SUM(ss.u),0) AS SIGNED) AS u,
            CAST(COALESCE(SUM(ss.d),0) AS SIGNED) AS d,
            CAST(COALESCE(SUM(ss.u + ss.d),0) AS SIGNED) AS total
         FROM v2_stat_server ss
         LEFT JOIN v2_server s ON s.id = ss.server_id
         LEFT JOIN v2_server parent ON parent.id = s.parent_id
         WHERE ss.record_at >= ? AND ss.record_at < ? AND ss.record_type = 'd'
         GROUP BY ss.server_id, ss.server_type, COALESCE(parent.name, s.name)
         ORDER BY total DESC"
    )
    .bind(start_at)
    .bind(end_at)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(json_value_response(success_response_payload(Value::Array(
        rows.into_iter()
            .map(|row| json!({
                "server_name": row.server_name.unwrap_or_else(|| format!("{}-{}", row.server_type, row.server_id)),
                "server_id": row.server_id,
                "server_type": row.server_type,
                "u": row.u,
                "d": row.d,
                "total": row.total,
            }))
            .collect(),
    ))))
}

async fn build_get_stat_record_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let record_type = params.get("type").map(|value| value.as_str()).unwrap_or("");
    let rows = match record_type {
        "paid_total" => load_stat_records_with_scaled_amount(state, "paid_total").await?,
        "commission_total" => load_stat_records_with_scaled_amount(state, "commission_total").await?,
        "register_count" => load_stat_records_plain(state).await?,
        _ => Vec::new(),
    };
    Ok(json_value_response(json!({
        "data": rows,
    })))
}

async fn build_get_ranking_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let ranking_type = params.get("type").map(|value| value.as_str()).unwrap_or("");
    let limit = params
        .get("limit")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(20)
        .clamp(1, 100);

    let data = match ranking_type {
        "server_traffic_rank" => load_server_traffic_ranking(state, limit).await?,
        "user_consumption_rank" => load_user_consumption_ranking(state, limit).await?,
        "invite_rank" => load_invite_ranking(state, limit).await?,
        _ => Vec::new(),
    };
    Ok(json_value_response(json!({
        "data": data,
    })))
}

async fn build_get_override_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let now = Utc::now().timestamp();
    let summary = load_admin_dashboard_summary(state, now)
        .await
        .map_err(internal_error)?;

    let data = json!({
        "month_income": summary.current_month_income,
        "month_register_total": summary.current_month_register_total,
        "ticket_pending_total": summary.ticket_pending_total,
        "commission_pending_total": summary.commission_pending_total,
        "day_income": summary.today_income,
        "last_month_income": summary.last_month_income,
        "commission_month_payout": summary.commission_month_payout,
        "commission_last_month_payout": summary.commission_last_month_payout,
        "online_nodes": summary.online_nodes,
        "online_devices": summary.online_devices,
        "online_users": summary.online_users,
        "today_traffic": {
            "upload": summary.today_traffic.upload,
            "download": summary.today_traffic.download,
            "total": summary.today_traffic.total,
        },
        "month_traffic": {
            "upload": summary.month_traffic.upload,
            "download": summary.month_traffic.download,
            "total": summary.month_traffic.total,
        },
        "total_traffic": {
            "upload": summary.total_traffic.upload,
            "download": summary.total_traffic.download,
            "total": summary.total_traffic.total,
        },
    });
    Ok(json_value_response(json!({ "data": data })))
}

async fn build_get_stats_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_admin_user(state, &headers).await?;
    let now = Utc::now().timestamp();
    let summary = load_admin_dashboard_summary(state, now)
        .await
        .map_err(internal_error)?;

    let data = json!({
        "today_income": summary.today_income,
        "yesterday_income": summary.yesterday_income,
        "current_month_income": summary.current_month_income,
        "last_month_income": summary.last_month_income,
        "two_months_ago_income": summary.two_months_ago_income,
        "month_income_growth": percentage_growth(summary.current_month_income, summary.last_month_income),
        "day_income_growth": percentage_growth(summary.today_income, summary.yesterday_income),
        "current_month_register_total": summary.current_month_register_total,
        "last_month_register_total": summary.last_month_register_total,
        "month_register_growth": percentage_growth(summary.current_month_register_total, summary.last_month_register_total),
        "online_nodes": summary.online_nodes,
        "online_devices": summary.online_devices,
        "online_users": summary.online_users,
        "today_traffic": {
            "upload": summary.today_traffic.upload,
            "download": summary.today_traffic.download,
            "total": summary.today_traffic.total,
        },
        "month_traffic": {
            "upload": summary.month_traffic.upload,
            "download": summary.month_traffic.download,
            "total": summary.month_traffic.total,
        },
        "total_traffic": {
            "upload": summary.total_traffic.upload,
            "download": summary.total_traffic.download,
            "total": summary.total_traffic.total,
        },
        "ticket_pending_total": summary.ticket_pending_total,
        "commission_pending_total": summary.commission_pending_total,
    });
    Ok(json_value_response(json!({ "data": data })))
}

fn parse_ymd_start_ts(value: &str) -> Option<i64> {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .ok()
        .and_then(|date| date.and_hms_opt(0, 0, 0))
        .map(|dt| dt.and_utc().timestamp())
}

fn parse_ymd_end_ts(value: &str) -> Option<i64> {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .ok()
        .and_then(|date| date.and_hms_opt(23, 59, 59))
        .map(|dt| dt.and_utc().timestamp())
}

fn format_timestamp_day(ts: i64) -> String {
    chrono::DateTime::<Utc>::from_timestamp(ts, 0)
        .map(|value| value.format("%Y-%m-%d").to_string())
        .unwrap_or_default()
}

fn stat_type_label(stat_type: &str) -> &'static str {
    match stat_type {
        "paid_total" => "收款金额",
        "paid_count" => "收款笔数",
        "commission_total" => "佣金金额(已发放)",
        "commission_count" => "佣金笔数(已发放)",
        _ => "",
    }
}

fn server_rank_time_range(mode: &str) -> (i64, i64) {
    let now = Utc::now();
    match mode {
        "yesterday" => {
            let end = now
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .map(|dt| dt.and_utc().timestamp())
                .unwrap_or(now.timestamp());
            (end - 86_400, end)
        }
        _ => {
            let start = now
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .map(|dt| dt.and_utc().timestamp())
                .unwrap_or(now.timestamp());
            (start, start + 86_400)
        }
    }
}

async fn load_stat_records_with_scaled_amount(
    state: &AppState,
    field: &str,
) -> Result<Vec<Value>, Response<Body>> {
    let sql = format!(
        "SELECT
            record_at,
            record_type,
            order_count,
            order_total,
            commission_count,
            commission_total,
            paid_count,
            paid_total,
            register_count,
            invite_count,
            transfer_used_total,
            created_at,
            updated_at,
            CAST(COALESCE({field},0) AS DOUBLE) / 100.0 AS scaled_value
         FROM v2_stat
         ORDER BY record_at ASC"
    );
    let rows = sqlx::query(&sql)
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(rows
        .into_iter()
        .map(|row| {
            let mut value = stat_record_row_to_value(&row);
            if let Some(object) = value.as_object_mut() {
                object.insert(field.to_string(), Value::from(row.try_get::<f64, _>("scaled_value").unwrap_or(0.0)));
            }
            value
        })
        .collect())
}

async fn load_stat_records_plain(
    state: &AppState,
) -> Result<Vec<Value>, Response<Body>> {
    let rows = sqlx::query(
        "SELECT
            record_at,
            record_type,
            order_count,
            order_total,
            commission_count,
            commission_total,
            paid_count,
            paid_total,
            register_count,
            invite_count,
            transfer_used_total,
            created_at,
            updated_at
         FROM v2_stat
         ORDER BY record_at ASC"
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;
    Ok(rows.into_iter().map(|row| stat_record_row_to_value(&row)).collect())
}

fn stat_record_row_to_value(row: &sqlx::mysql::MySqlRow) -> Value {
    json!({
        "record_at": row.try_get::<i64, _>("record_at").unwrap_or(0),
        "record_type": row.try_get::<String, _>("record_type").unwrap_or_default(),
        "order_count": row.try_get::<i64, _>("order_count").unwrap_or(0),
        "order_total": row.try_get::<i64, _>("order_total").unwrap_or(0),
        "commission_count": row.try_get::<i64, _>("commission_count").unwrap_or(0),
        "commission_total": row.try_get::<i64, _>("commission_total").unwrap_or(0),
        "paid_count": row.try_get::<i64, _>("paid_count").unwrap_or(0),
        "paid_total": row.try_get::<i64, _>("paid_total").unwrap_or(0),
        "register_count": row.try_get::<i64, _>("register_count").unwrap_or(0),
        "invite_count": row.try_get::<i64, _>("invite_count").unwrap_or(0),
        "transfer_used_total": row.try_get::<String, _>("transfer_used_total").unwrap_or_default(),
        "created_at": row.try_get::<i64, _>("created_at").unwrap_or(0),
        "updated_at": row.try_get::<i64, _>("updated_at").unwrap_or(0),
    })
}

async fn load_server_traffic_ranking(
    state: &AppState,
    limit: i64,
) -> Result<Vec<Value>, Response<Body>> {
    let rows = sqlx::query(
        "SELECT
            server_id,
            server_type,
            CAST(COALESCE(SUM(u),0) AS SIGNED) AS u,
            CAST(COALESCE(SUM(d),0) AS SIGNED) AS d,
            CAST(COALESCE(SUM(u + d),0) AS SIGNED) AS total
         FROM v2_stat_server
         GROUP BY server_id, server_type
         ORDER BY total DESC
         LIMIT ?"
    )
    .bind(limit)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;
    Ok(rows
        .into_iter()
        .map(|row| json!({
            "server_id": row.try_get::<i64, _>("server_id").unwrap_or(0),
            "server_type": row.try_get::<String, _>("server_type").unwrap_or_default(),
            "u": row.try_get::<i64, _>("u").unwrap_or(0),
            "d": row.try_get::<i64, _>("d").unwrap_or(0),
            "total": row.try_get::<i64, _>("total").unwrap_or(0),
        }))
        .collect())
}

async fn load_user_consumption_ranking(
    state: &AppState,
    limit: i64,
) -> Result<Vec<Value>, Response<Body>> {
    let rows = sqlx::query(
        "SELECT
            su.user_id,
            CAST(COALESCE(SUM(su.u),0) AS SIGNED) AS u,
            CAST(COALESCE(SUM(su.d),0) AS SIGNED) AS d,
            CAST(COALESCE(SUM(su.u + su.d),0) AS SIGNED) AS total,
            u.email
         FROM v2_stat_user su
         LEFT JOIN v2_user u ON u.id = su.user_id
         GROUP BY su.user_id, u.email
         ORDER BY total DESC
         LIMIT ?"
    )
    .bind(limit)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;
    Ok(rows
        .into_iter()
        .map(|row| json!({
            "user_id": row.try_get::<i64, _>("user_id").unwrap_or(0),
            "u": row.try_get::<i64, _>("u").unwrap_or(0),
            "d": row.try_get::<i64, _>("d").unwrap_or(0),
            "total": row.try_get::<i64, _>("total").unwrap_or(0),
            "email": row.try_get::<Option<String>, _>("email").ok().flatten(),
        }))
        .collect())
}

async fn load_invite_ranking(
    state: &AppState,
    limit: i64,
) -> Result<Vec<Value>, Response<Body>> {
    let rows = sqlx::query(
        "SELECT
            invite_user_id,
            COUNT(*) AS count
         FROM v2_user
         WHERE invite_user_id IS NOT NULL
         GROUP BY invite_user_id
         ORDER BY count DESC
         LIMIT ?"
    )
    .bind(limit)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;
    Ok(rows
        .into_iter()
        .map(|row| json!({
            "invite_user_id": row.try_get::<i64, _>("invite_user_id").unwrap_or(0),
            "count": row.try_get::<i64, _>("count").unwrap_or(0),
        }))
        .collect())
}
