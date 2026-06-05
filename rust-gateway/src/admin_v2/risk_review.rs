use super::super::*;

#[derive(Clone, sqlx::FromRow)]
struct RiskReviewRow {
    id: u64,
    user_id: i64,
    shared_ip: String,
    matched_user_count: i64,
    matched_user_ids: Option<String>,
    risk_level: String,
    suspicion_score: i64,
    llm_model: Option<String>,
    summary: Option<String>,
    recommendation: Option<String>,
    evidence: Option<String>,
    reviewed_at: i64,
    created_at: i64,
    user_email: Option<String>,
    user_banned: Option<i8>,
    user_ban_reason: Option<String>,
}

#[derive(Clone, sqlx::FromRow)]
struct RiskMatchedUserRow {
    id: i64,
    email: String,
}

#[derive(Clone, sqlx::FromRow)]
struct RiskGroupRow {
    ip_address: String,
    matched_user_count: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct RiskSessionRow {
    user_id: i64,
    node_id: i64,
    connection_count: i64,
    upload_traffic: i64,
    download_traffic: i64,
    last_activity: i64,
    node_name: Option<String>,
    node_protocol: Option<String>,
    node_location_name: Option<String>,
}

#[derive(Clone, sqlx::FromRow)]
struct RiskUserProfileRow {
    id: i64,
    email: String,
    banned: i8,
    ban_reason: Option<String>,
    invite_user_id: Option<i64>,
    plan_id: Option<i64>,
    plan_name: Option<String>,
    telegram_id: Option<i64>,
}

#[derive(Clone, sqlx::FromRow)]
struct AuditStatRow {
    action_taken: Option<String>,
    ip_address: Option<String>,
    target_domain: Option<String>,
}

#[derive(Clone, sqlx::FromRow)]
struct TrafficUsageAggRow {
    node_id: i64,
    billed_traffic_kb: i64,
    raw_traffic_kb: i64,
    node_name: Option<String>,
}

#[derive(Clone, sqlx::FromRow)]
struct DailyTrafficRow {
    record_date: String,
    total_kb: i64,
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

pub async fn run(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_run_response(&state, headers, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_fetch_response(
    state: &AppState,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let params = parse_query(&uri);
    let current = params
        .get("current")
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(1)
        .max(1);
    let page_size = params
        .get("pageSize")
        .or_else(|| params.get("page_size"))
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(20)
        .clamp(1, 100);
    let risk_level = params
        .get("risk_level")
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty());
    let user_id_filter = params
        .get("user_id")
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0);
    let offset = (current - 1) * page_size;

    let (rows, total) = load_risk_review_rows(state, offset, page_size, risk_level.as_deref(), user_id_filter)
        .await
        .map_err(internal_error)?;

    let mut matched_ids = Vec::new();
    for row in &rows {
        matched_ids.extend(parse_matched_user_ids(row.matched_user_ids.as_deref()));
    }
    matched_ids.sort_unstable();
    matched_ids.dedup();
    let matched_user_map = load_matched_user_map(state, &matched_ids)
        .await
        .map_err(internal_error)?;

    let data = rows
        .iter()
        .map(|row| serialize_risk_review_row(row, &matched_user_map))
        .collect::<Vec<_>>();

    Ok(json_value_response(json!({
        "data": data,
        "total": total,
        "current_page": current,
        "last_page": if total <= 0 { 1 } else { ((total + page_size - 1) / page_size).max(1) },
    })))
}

async fn build_run_response(
    state: &AppState,
    headers: HeaderMap,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let payload = parse_json_body(body).await?;
    let limit = payload
        .get("limit")
        .and_then(parse_i64_value)
        .unwrap_or(20)
        .clamp(1, 200);

    let summary = run_risk_review_scan(state, limit)
        .await
        .map_err(internal_error)?;
    Ok(json_value_response(success_response_payload(summary)))
}

async fn load_risk_review_rows(
    state: &AppState,
    offset: i64,
    limit: i64,
    risk_level: Option<&str>,
    user_id_filter: Option<i64>,
) -> Result<(Vec<RiskReviewRow>, i64), sqlx::Error> {
    let mut where_clauses = Vec::new();
    if risk_level.is_some() {
        where_clauses.push("r.risk_level = ?");
    }
    if user_id_filter.is_some() {
        where_clauses.push("r.user_id = ?");
    }
    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", where_clauses.join(" AND "))
    };

    let data_sql = format!(
        "SELECT r.id, r.user_id, r.shared_ip, r.matched_user_count, r.matched_user_ids, r.risk_level,
                r.suspicion_score, r.llm_model, r.summary, r.recommendation, r.evidence, r.reviewed_at,
                r.created_at, u.email AS user_email, u.banned AS user_banned, u.ban_reason AS user_ban_reason
         FROM user_risk_reviews r
         LEFT JOIN v2_user u ON u.id = r.user_id
         {}
         ORDER BY r.reviewed_at DESC
         LIMIT ? OFFSET ?",
        where_sql
    );
    let total_sql = format!("SELECT COUNT(*) FROM user_risk_reviews r{}", where_sql);

    let mut data_query = sqlx::query_as::<_, RiskReviewRow>(&data_sql);
    let mut total_query = sqlx::query_scalar::<_, i64>(&total_sql);
    if let Some(risk_level) = risk_level {
        data_query = data_query.bind(risk_level);
        total_query = total_query.bind(risk_level);
    }
    if let Some(user_id) = user_id_filter {
        data_query = data_query.bind(user_id);
        total_query = total_query.bind(user_id);
    }

    let total = total_query.fetch_one(&state.db).await?;
    let rows = data_query.bind(limit).bind(offset).fetch_all(&state.db).await?;
    Ok((rows, total))
}

async fn load_matched_user_map(
    state: &AppState,
    user_ids: &[i64],
) -> Result<HashMap<i64, String>, sqlx::Error> {
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let placeholders = vec!["?"; user_ids.len()].join(",");
    let sql = format!("SELECT id, email FROM v2_user WHERE id IN ({})", placeholders);
    let mut query = sqlx::query_as::<_, RiskMatchedUserRow>(&sql);
    for user_id in user_ids {
        query = query.bind(*user_id);
    }
    Ok(query
        .fetch_all(&state.db)
        .await?
        .into_iter()
        .map(|row| (row.id, row.email))
        .collect())
}

fn serialize_risk_review_row(
    row: &RiskReviewRow,
    matched_user_map: &HashMap<i64, String>,
) -> Value {
    let matched_users = parse_matched_user_ids(row.matched_user_ids.as_deref())
        .into_iter()
        .map(|id| {
            json!({
                "id": id,
                "email": matched_user_map.get(&id).cloned().unwrap_or_else(|| format!("#{}", id)),
                "plan": "-",
                "banned": false
            })
        })
        .collect::<Vec<_>>();
    json!({
        "id": row.id,
        "user_id": row.user_id,
        "user_email": row.user_email.clone().unwrap_or_else(|| "-".to_string()),
        "user_banned": row.user_banned.unwrap_or(0) != 0,
        "user_ban_reason": row.user_ban_reason.clone(),
        "shared_ip": row.shared_ip,
        "matched_user_count": row.matched_user_count,
        "matched_users": matched_users,
        "risk_level": row.risk_level,
        "suspicion_score": row.suspicion_score,
        "summary": row.summary.clone().unwrap_or_default(),
        "recommendation": row.recommendation.clone().unwrap_or_default(),
        "llm_model": row.llm_model.clone(),
        "reviewed_at": row.reviewed_at,
        "created_at": row.created_at,
        "evidence": row.evidence.as_deref().and_then(|value| serde_json::from_str::<Value>(value).ok()).unwrap_or(Value::Null),
    })
}

fn parse_matched_user_ids(raw: Option<&str>) -> Vec<i64> {
    let Some(raw) = raw else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<Value>(raw) else {
        return Vec::new();
    };
    value
        .as_array()
        .map(|items| {
            items.iter()
                .filter_map(|item| match item {
                    Value::Object(object) => object.get("id").and_then(parse_i64_value),
                    _ => parse_i64_value(item),
                })
                .filter(|id| *id > 0)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub(crate) async fn run_risk_review_scan(
    state: &AppState,
    limit: i64,
) -> Result<Value, sqlx::Error> {
    let group_limit = limit.max(1);
    let shared_ip_groups = find_shared_ip_groups(state, group_limit).await?;

    let mut summary = json!({
        "skipped": false,
        "groups_scanned": shared_ip_groups.len(),
        "reviews_created": 0,
        "users_skipped": 0,
        "telegram_notifications": 0,
        "shared_ips": [],
    });

    let mut shared_results = Vec::new();
    let mut total_reviews = 0_i64;
    let mut total_skipped = 0_i64;
    let mut total_notifications = 0_i64;
    for group in shared_ip_groups {
        let result = review_shared_ip_group(state, &group.ip_address, group.matched_user_count).await?;
        total_reviews += result.reviews_created;
        total_skipped += result.users_skipped;
        total_notifications += result.telegram_notifications;
        shared_results.push(json!({
            "shared_ip": result.shared_ip,
            "matched_user_count": result.matched_user_count,
            "reviews_created": result.reviews_created,
            "users_skipped": result.users_skipped,
            "telegram_notifications": result.telegram_notifications,
        }));
    }

    summary["reviews_created"] = Value::from(total_reviews);
    summary["users_skipped"] = Value::from(total_skipped);
    summary["telegram_notifications"] = Value::from(total_notifications);
    summary["shared_ips"] = Value::Array(shared_results);
    Ok(summary)
}

async fn find_shared_ip_groups(
    state: &AppState,
    limit: i64,
) -> Result<Vec<RiskGroupRow>, sqlx::Error> {
    let window_minutes = get_setting_int(state, "user_risk_review_time_window_minutes", 60)
        .await
        .max(5);
    let min_shared_users = get_setting_int(state, "user_risk_review_min_shared_ip_users", 2)
        .await
        .max(2);
    let cutoff = Utc::now().timestamp() - window_minutes * 60;

    sqlx::query_as::<_, RiskGroupRow>(
        "SELECT ip_address, COUNT(DISTINCT user_id) AS matched_user_count
         FROM user_online_sessions
         WHERE last_activity >= ?
         GROUP BY ip_address
         HAVING COUNT(DISTINCT user_id) >= ?
         ORDER BY matched_user_count DESC, MAX(last_activity) DESC
         LIMIT ?"
    )
    .bind(cutoff)
    .bind(min_shared_users)
    .bind(limit)
    .fetch_all(&state.db)
    .await
}

struct ReviewGroupResult {
    shared_ip: String,
    matched_user_count: i64,
    reviews_created: i64,
    users_skipped: i64,
    telegram_notifications: i64,
}

async fn review_shared_ip_group(
    state: &AppState,
    shared_ip: &str,
    matched_user_count: i64,
) -> Result<ReviewGroupResult, sqlx::Error> {
    let window_minutes = get_setting_int(state, "user_risk_review_time_window_minutes", 60)
        .await
        .max(5);
    let cooldown_minutes = get_setting_int(state, "user_risk_review_notify_cooldown_minutes", 60)
        .await
        .max(5);
    let cutoff = Utc::now().timestamp() - window_minutes * 60;
    let sessions = load_risk_sessions(state, shared_ip, cutoff).await?;
    let user_ids = sessions
        .iter()
        .map(|session| session.user_id)
        .collect::<Vec<_>>();
    let profiles = load_risk_user_profiles(state, &user_ids).await?;
    let profile_map = profiles
        .into_iter()
        .map(|profile| (profile.id, profile))
        .collect::<HashMap<_, _>>();

    let mut created = Vec::new();
    let mut skipped = 0_i64;
    for user_id in user_ids {
        let Some(profile) = profile_map.get(&user_id) else {
            skipped += 1;
            continue;
        };
        if profile.banned != 0 {
            skipped += 1;
            continue;
        }
        if was_reviewed_recently(state, user_id, shared_ip, cooldown_minutes).await? {
            skipped += 1;
            continue;
        }

        let review_context = build_review_context(state, profile, shared_ip, matched_user_count, &sessions, &profile_map)
            .await?;
        let result = review_user_context(state, &review_context).await;
        let now = Utc::now().timestamp();
        let matched_users = review_context
            .get("matched_users")
            .cloned()
            .unwrap_or(Value::Array(Vec::new()));
        let evidence = review_context
            .get("evidence")
            .cloned()
            .unwrap_or(Value::Null);
        let risk_level = result
            .get("risk_level")
            .and_then(Value::as_str)
            .unwrap_or("medium");
        let suspicion_score = result
            .get("suspicion_score")
            .and_then(Value::as_i64)
            .unwrap_or(50);
        let summary = result
            .get("summary")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let recommendation = result
            .get("recommendation")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let llm_model = result.get("llm_model").and_then(Value::as_str).map(|value| value.to_string());
        let raw_response = result.get("raw_response").cloned().unwrap_or(Value::Null);

        let inserted_id = sqlx::query(
            "INSERT INTO user_risk_reviews
                (user_id, source, shared_ip, matched_user_count, matched_user_ids, risk_level,
                 suspicion_score, llm_model, summary, recommendation, raw_response, evidence, reviewed_at, created_at, updated_at)
             VALUES (?, 'shared_ip', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(user_id)
        .bind(shared_ip)
        .bind(matched_user_count)
        .bind(serde_json::to_string(&matched_users).unwrap_or_else(|_| "[]".to_string()))
        .bind(risk_level)
        .bind(suspicion_score)
        .bind(llm_model.clone())
        .bind(summary.clone())
        .bind(recommendation.clone())
        .bind(match raw_response {
            Value::Null => None::<String>,
            Value::String(text) => Some(text),
            other => Some(other.to_string()),
        })
        .bind(match evidence {
            Value::Null => None::<String>,
            other => Some(other.to_string()),
        })
        .bind(now)
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await?
        .last_insert_id();

        created.push((inserted_id, user_id, summary, recommendation, llm_model));
    }

    let telegram_notifications = if created.is_empty() {
        0
    } else {
        send_risk_review_telegram_alert(state, shared_ip, matched_user_count, &created, &profile_map)
            .await
            .unwrap_or(0)
    };

    Ok(ReviewGroupResult {
        shared_ip: shared_ip.to_string(),
        matched_user_count,
        reviews_created: created.len() as i64,
        users_skipped: skipped,
        telegram_notifications,
    })
}

async fn load_risk_sessions(
    state: &AppState,
    shared_ip: &str,
    cutoff: i64,
) -> Result<Vec<RiskSessionRow>, sqlx::Error> {
    sqlx::query_as::<_, RiskSessionRow>(
        "SELECT s.user_id, CAST(s.node_id AS SIGNED) AS node_id, CAST(s.connection_count AS SIGNED) AS connection_count,
                CAST(s.upload_traffic AS SIGNED) AS upload_traffic, CAST(s.download_traffic AS SIGNED) AS download_traffic,
                CAST(s.last_activity AS SIGNED) AS last_activity,
                n.name AS node_name, n.protocol AS node_protocol, n.location_name AS node_location_name
         FROM user_online_sessions s
         LEFT JOIN server_nodes n ON n.id = s.node_id
         WHERE s.ip_address = ?
           AND s.last_activity >= ?
         ORDER BY s.last_activity DESC"
    )
    .bind(shared_ip)
    .bind(cutoff)
    .fetch_all(&state.db)
    .await
}

async fn load_risk_user_profiles(
    state: &AppState,
    user_ids: &[i64],
) -> Result<Vec<RiskUserProfileRow>, sqlx::Error> {
    if user_ids.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = vec!["?"; user_ids.len()].join(",");
    let sql = format!(
        "SELECT u.id, u.email, u.banned, u.ban_reason, u.invite_user_id, u.plan_id, u.telegram_id,
                p.name AS plan_name
         FROM v2_user u
         LEFT JOIN v2_plan p ON p.id = u.plan_id
         WHERE u.id IN ({})",
        placeholders
    );
    let mut query = sqlx::query_as::<_, RiskUserProfileRow>(&sql);
    for user_id in user_ids {
        query = query.bind(*user_id);
    }
    query.fetch_all(&state.db).await
}

async fn was_reviewed_recently(
    state: &AppState,
    user_id: i64,
    shared_ip: &str,
    cooldown_minutes: i64,
) -> Result<bool, sqlx::Error> {
    let cutoff = Utc::now().timestamp() - cooldown_minutes * 60;
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_risk_reviews
         WHERE user_id = ? AND shared_ip = ? AND reviewed_at >= ?"
    )
    .bind(user_id)
    .bind(shared_ip)
    .bind(cutoff)
    .fetch_one(&state.db)
    .await?;
    Ok(count > 0)
}

async fn build_review_context(
    state: &AppState,
    profile: &RiskUserProfileRow,
    shared_ip: &str,
    matched_user_count: i64,
    sessions: &[RiskSessionRow],
    profile_map: &HashMap<i64, RiskUserProfileRow>,
) -> Result<Value, sqlx::Error> {
    let context_hours = get_setting_int(state, "user_risk_review_context_hours", 24)
        .await
        .max(1);
    let context_cutoff = Utc::now().timestamp() - context_hours * 3600;
    let user_sessions = sessions
        .iter()
        .filter(|session| session.user_id == profile.id)
        .collect::<Vec<_>>();
    let matched_users = profile_map
        .values()
        .map(|item| {
            json!({
                "id": item.id,
                "email": item.email,
                "plan": item.plan_name.clone().unwrap_or_else(|| "-".to_string()),
                "banned": item.banned != 0,
            })
        })
        .collect::<Vec<_>>();

    let audit_rows = load_risk_audit_rows(state, profile.id, context_cutoff).await?;
    let traffic_usage_rows = load_risk_traffic_usage_aggs(state, profile.id, context_cutoff).await?;
    let daily_traffic_rows = load_risk_daily_traffic(state, profile.id, context_cutoff).await?;
    let other_active_ip_count = load_other_active_ip_count(state, profile.id, shared_ip).await?;

    let audit_total = audit_rows.len() as i64;
    let blocked = audit_rows
        .iter()
        .filter(|row| row.action_taken.as_deref() == Some("blocked"))
        .count() as i64;
    let allowed = audit_rows
        .iter()
        .filter(|row| row.action_taken.as_deref() == Some("allowed"))
        .count() as i64;
    let logged = audit_rows
        .iter()
        .filter(|row| row.action_taken.as_deref() == Some("logged"))
        .count() as i64;
    let shared_ip_hits = audit_rows
        .iter()
        .filter(|row| row.ip_address.as_deref() == Some(shared_ip))
        .count() as i64;

    let mut domain_counts = HashMap::<String, i64>::new();
    for row in &audit_rows {
        if let Some(domain) = row.target_domain.as_deref().filter(|value| !value.trim().is_empty()) {
            *domain_counts.entry(domain.to_string()).or_insert(0) += 1;
        }
    }
    let mut top_domains = domain_counts
        .into_iter()
        .map(|(domain, count)| json!({ "domain": domain, "count": count }))
        .collect::<Vec<_>>();
    top_domains.sort_by(|a, b| {
        b.get("count")
            .and_then(Value::as_i64)
            .unwrap_or(0)
            .cmp(&a.get("count").and_then(Value::as_i64).unwrap_or(0))
    });
    top_domains.truncate(5);

    let usage_log_count = traffic_usage_rows.len() as i64;
    let billed_traffic_kb = traffic_usage_rows.iter().map(|row| row.billed_traffic_kb).sum::<i64>();
    let raw_traffic_kb = traffic_usage_rows.iter().map(|row| row.raw_traffic_kb).sum::<i64>();
    let distinct_nodes = traffic_usage_rows.len() as i64;
    let top_nodes = traffic_usage_rows
        .iter()
        .map(|row| {
            json!({
                "node_id": row.node_id,
                "node_name": row.node_name.clone().unwrap_or_else(|| format!("#{}", row.node_id)),
                "billed_traffic_kb": row.billed_traffic_kb,
            })
        })
        .collect::<Vec<_>>();
    let daily_node_traffic_kb = daily_traffic_rows
        .iter()
        .map(|row| (row.record_date.clone(), Value::from(row.total_kb)))
        .collect::<Map<String, Value>>();

    let session_nodes = user_sessions
        .iter()
        .map(|session| {
            json!({
                "id": session.node_id,
                "name": session.node_name.clone().unwrap_or_else(|| format!("#{}", session.node_id)),
                "protocol": session.node_protocol.clone().unwrap_or_else(|| "-".to_string()),
                "location_name": session.node_location_name.clone().unwrap_or_else(|| "-".to_string()),
                "connection_count": session.connection_count,
                "last_activity": session.last_activity,
            })
        })
        .collect::<Vec<_>>();
    let shared_ip_connection_count = user_sessions.iter().map(|session| session.connection_count).sum::<i64>();
    let shared_ip_upload_traffic = user_sessions.iter().map(|session| session.upload_traffic).sum::<i64>();
    let shared_ip_download_traffic = user_sessions.iter().map(|session| session.download_traffic).sum::<i64>();

    Ok(json!({
        "shared_ip": shared_ip,
        "matched_user_count": matched_user_count,
        "matched_users": matched_users,
        "user": {
            "id": profile.id,
            "email": profile.email,
            "plan": profile.plan_name.clone().unwrap_or_else(|| "-".to_string()),
            "current_shared_sessions": user_sessions.len(),
            "shared_ip_connection_count": shared_ip_connection_count,
            "shared_ip_upload_traffic": shared_ip_upload_traffic,
            "shared_ip_download_traffic": shared_ip_download_traffic,
            "shared_ip_nodes": session_nodes,
            "other_active_ips": [],
        },
        "evidence": {
            "context_hours": context_hours,
            "audit_stats": {
                "total": audit_total,
                "blocked": blocked,
                "allowed": allowed,
                "logged": logged,
                "shared_ip_hits": shared_ip_hits,
                "top_domains": top_domains,
            },
            "traffic_stats": {
                "raw_traffic_kb": raw_traffic_kb,
                "billed_traffic_kb": billed_traffic_kb,
                "usage_log_count": usage_log_count,
                "distinct_nodes": distinct_nodes,
                "top_nodes": top_nodes,
                "daily_node_traffic_kb": daily_node_traffic_kb,
            },
            "shared_ip_nodes": session_nodes,
            "other_active_ip_count": other_active_ip_count,
        }
    }))
}

async fn load_risk_audit_rows(
    state: &AppState,
    user_id: i64,
    cutoff: i64,
) -> Result<Vec<AuditStatRow>, sqlx::Error> {
    sqlx::query_as::<_, AuditStatRow>(
        "SELECT action_taken, ip_address, target_domain
         FROM audit_logs
         WHERE user_id = ? AND created_at >= ?
         ORDER BY created_at DESC
         LIMIT 50"
    )
    .bind(user_id)
    .bind(cutoff)
    .fetch_all(&state.db)
    .await
}

async fn load_risk_traffic_usage_aggs(
    state: &AppState,
    user_id: i64,
    cutoff: i64,
) -> Result<Vec<TrafficUsageAggRow>, sqlx::Error> {
    sqlx::query_as::<_, TrafficUsageAggRow>(
        "SELECT l.node_id, CAST(COALESCE(SUM(l.billed_traffic_kb),0) AS SIGNED) AS billed_traffic_kb,
                CAST(COALESCE(SUM(l.raw_traffic_kb),0) AS SIGNED) AS raw_traffic_kb,
                n.name AS node_name
         FROM user_traffic_usage_logs l
         LEFT JOIN server_nodes n ON n.id = l.node_id
         WHERE l.user_id = ? AND l.recorded_at >= ?
         GROUP BY l.node_id, n.name
         ORDER BY billed_traffic_kb DESC
         LIMIT 5"
    )
    .bind(user_id)
    .bind(cutoff)
    .fetch_all(&state.db)
    .await
}

async fn load_risk_daily_traffic(
    state: &AppState,
    user_id: i64,
    cutoff: i64,
) -> Result<Vec<DailyTrafficRow>, sqlx::Error> {
    let start_date = chrono::DateTime::from_timestamp(cutoff, 0)
        .map(|value| value.date_naive().to_string())
        .unwrap_or_else(|| Utc::now().date_naive().to_string());
    sqlx::query_as::<_, DailyTrafficRow>(
        "SELECT CAST(record_date AS CHAR) AS record_date,
                CAST(COALESCE(SUM(upload_traffic + download_traffic),0) AS SIGNED) AS total_kb
         FROM node_traffic_records
         WHERE user_id = ? AND record_date >= ?
         GROUP BY record_date
         ORDER BY record_date DESC
         LIMIT 30"
    )
    .bind(user_id)
    .bind(start_date)
    .fetch_all(&state.db)
    .await
}

async fn load_other_active_ip_count(
    state: &AppState,
    user_id: i64,
    shared_ip: &str,
) -> Result<i64, sqlx::Error> {
    let window_minutes = get_setting_int(state, "user_risk_review_time_window_minutes", 60)
        .await
        .max(5);
    let cutoff = Utc::now().timestamp() - window_minutes * 60;
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(DISTINCT ip_address)
         FROM user_online_sessions
         WHERE user_id = ? AND ip_address != ? AND last_activity >= ?"
    )
    .bind(user_id)
    .bind(shared_ip)
    .bind(cutoff)
    .fetch_one(&state.db)
    .await
}

async fn review_user_context(
    state: &AppState,
    review_context: &Value,
) -> Value {
    let heuristic = build_heuristic_result(review_context);
    if !llm_review_is_configured(state).await {
        return heuristic;
    }
    match request_llm_risk_review(state, review_context).await {
        Ok(value) => merge_heuristic_with_llm(&heuristic, value),
        Err(_) => heuristic,
    }
}

fn build_heuristic_result(review_context: &Value) -> Value {
    let matched_user_count = review_context
        .get("matched_user_count")
        .and_then(Value::as_i64)
        .unwrap_or(2);
    let audit_total = review_context
        .get("evidence")
        .and_then(|value| value.get("audit_stats"))
        .and_then(|value| value.get("total"))
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let blocked = review_context
        .get("evidence")
        .and_then(|value| value.get("audit_stats"))
        .and_then(|value| value.get("blocked"))
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let other_ip_count = review_context
        .get("evidence")
        .and_then(|value| value.get("other_active_ip_count"))
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let billed_traffic_kb = review_context
        .get("evidence")
        .and_then(|value| value.get("traffic_stats"))
        .and_then(|value| value.get("billed_traffic_kb"))
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let shared_ip = review_context
        .get("shared_ip")
        .and_then(Value::as_str)
        .unwrap_or("");
    let user_id = review_context
        .get("user")
        .and_then(|value| value.get("id"))
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let user_email = review_context
        .get("user")
        .and_then(|value| value.get("email"))
        .and_then(Value::as_str)
        .unwrap_or("-");

    let mut score = 50_i64;
    score += ((matched_user_count - 1).max(0) * 10).min(25);
    score += ((audit_total / 20) * 5).min(10);
    score += ((blocked / 5) * 5).min(10);
    if other_ip_count == 0 {
        score += 5;
    }
    if billed_traffic_kb >= 1024 * 1024 {
        score += 10;
    }
    score = score.clamp(0, 100);
    let risk_level = if score >= 80 { "high" } else { "medium" };

    json!({
        "risk_level": risk_level,
        "suspicion_score": score,
        "summary": format!(
            "检测到用户 #{} ({}) 在共享 IP {} 上与 {} 个用户存在重叠使用行为，已按至少中风险处理。",
            user_id, user_email, shared_ip, matched_user_count
        ),
        "recommendation": if risk_level == "high" {
            "建议立即人工复核该用户的最近节点、流量和审计行为，如确认异常可执行封禁。"
        } else {
            "建议人工复核该用户近期在线 IP、节点分布和审计行为，确认是否存在账号共享或滥用。"
        },
        "llm_model": Value::Null,
        "raw_response": Value::Null,
    })
}

async fn llm_review_is_configured(state: &AppState) -> bool {
    get_setting_bool(state, "user_risk_review_llm_enable", false).await
        && !get_setting_string(state, "user_risk_review_llm_base_url", "").await.trim().is_empty()
        && !get_setting_string(state, "user_risk_review_llm_api_key", "").await.trim().is_empty()
        && !get_setting_string(state, "user_risk_review_llm_model", "").await.trim().is_empty()
}

async fn request_llm_risk_review(
    state: &AppState,
    review_context: &Value,
) -> Result<Value, String> {
    let base_url = get_setting_string(state, "user_risk_review_llm_base_url", "").await;
    let api_key = get_setting_string(state, "user_risk_review_llm_api_key", "").await;
    let model = get_setting_string(state, "user_risk_review_llm_model", "").await;
    let timeout_seconds = get_setting_int(state, "user_risk_review_llm_timeout_seconds", 20)
        .await
        .clamp(5, 120);
    let temperature = get_setting_f64(state, "user_risk_review_llm_temperature", 0.2)
        .await
        .clamp(0.0, 1.0);
    let endpoint = resolve_llm_endpoint(&base_url);
    let body = if endpoint.1 == "responses" {
        json!({
            "model": model,
            "temperature": temperature,
            "input": [
                { "role": "system", "content": build_risk_review_system_prompt() },
                { "role": "user", "content": review_context.to_string() }
            ]
        })
    } else {
        json!({
            "model": model,
            "temperature": temperature,
            "messages": [
                { "role": "system", "content": build_risk_review_system_prompt() },
                { "role": "user", "content": review_context.to_string() }
            ]
        })
    };
    let uri: Uri = endpoint
        .0
        .parse()
        .map_err(|err| format!("invalid llm uri: {err}"))?;
    let request = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(CONTENT_TYPE, "application/json")
        .header("accept", "application/json")
        .header("authorization", format!("Bearer {}", api_key))
        .body(Body::from(body.to_string()))
        .map_err(|err| format!("build llm request failed: {err}"))?;

    let response = tokio::time::timeout(
        Duration::from_secs(timeout_seconds as u64),
        state.backend_client.request(request),
    )
    .await
    .map_err(|_| "llm request timeout".to_string())?
    .map_err(|err| format!("llm request failed: {err}"))?;
    let status = response.status();
    let bytes = response_body_bytes(map_proxy_response(response).await).await;
    let payload: Value = serde_json::from_slice(&bytes)
        .map_err(|err| format!("decode llm response failed: {err}"))?;
    if !status.is_success() {
        return Err(format!("llm response status {}", status));
    }

    let content = extract_llm_text_content(&payload)?;
    let parsed = parse_llm_structured_payload(&content)?;
    Ok(json!({
        "risk_level": parsed.get("risk_level").cloned().unwrap_or(Value::String("medium".to_string())),
        "suspicion_score": parsed.get("suspicion_score").cloned().unwrap_or(Value::from(50)),
        "summary": parsed.get("summary").cloned().unwrap_or(Value::String(String::new())),
        "recommendation": parsed.get("recommendation").cloned().unwrap_or(Value::String(String::new())),
        "llm_model": payload.get("model").cloned().unwrap_or(Value::String(model)),
        "raw_response": content,
    }))
}

fn resolve_llm_endpoint(base_url: &str) -> (String, &'static str) {
    let normalized = base_url.trim().trim_end_matches('/').to_string();
    let path = normalized
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(normalized.as_str());
    let path = path.split_once('/').map(|(_, rest)| rest).unwrap_or("");
    if path.ends_with("responses") {
        return (normalized, "responses");
    }
    if path.ends_with("chat/completions") {
        return (normalized, "chat_completions");
    }
    if path.ends_with("/v1") {
        return (format!("{}/chat/completions", normalized), "chat_completions");
    }
    if path.is_empty() {
        return (format!("{}/v1/chat/completions", normalized), "chat_completions");
    }
    (format!("{}/chat/completions", normalized), "chat_completions")
}

fn build_risk_review_system_prompt() -> &'static str {
    "你是代理节点滥用风控助手，需要根据共享 IP、流量和审计行为给出预审结论。\n共享 IP 命中多个用户时，最低风险等级不能低于 medium。\n请只返回 JSON，不要返回 Markdown。\nJSON 字段必须包含：risk_level, suspicion_score, summary, recommendation。\nrisk_level 只能是 low、medium、high。\nsuspicion_score 为 0 到 100 的整数。\nsummary 和 recommendation 要用简体中文，简洁明确。"
}

fn extract_llm_text_content(payload: &Value) -> Result<String, String> {
    if let Some(content) = payload
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(|item| item.get("message"))
        .and_then(|item| item.get("content"))
    {
        match content {
            Value::String(text) => return Ok(text.trim().to_string()),
            Value::Array(items) => {
                let text = items
                    .iter()
                    .filter_map(|item| item.get("text").and_then(Value::as_str).or_else(|| item.as_str()))
                    .collect::<Vec<_>>()
                    .join("\n");
                if !text.trim().is_empty() {
                    return Ok(text.trim().to_string());
                }
            }
            _ => {}
        }
    }

    if let Some(output_text) = payload.get("output_text").and_then(Value::as_str) {
        if !output_text.trim().is_empty() {
            return Ok(output_text.trim().to_string());
        }
    }

    if let Some(output) = payload.get("output").and_then(Value::as_array) {
        let mut parts = Vec::new();
        for item in output {
            if let Some(text) = item.as_str() {
                if !text.trim().is_empty() {
                    parts.push(text.trim().to_string());
                }
                continue;
            }
            if let Some(content_items) = item.get("content").and_then(Value::as_array) {
                for content_item in content_items {
                    if let Some(text) = content_item.get("text").and_then(Value::as_str).or_else(|| content_item.as_str()) {
                        if !text.trim().is_empty() {
                            parts.push(text.trim().to_string());
                        }
                    }
                }
            }
        }
        if !parts.is_empty() {
            return Ok(parts.join("\n"));
        }
    }

    Err("llm returned empty content".to_string())
}

fn parse_llm_structured_payload(content: &str) -> Result<Value, String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Err("empty llm content".to_string());
    }
    let mut candidates = vec![trimmed.to_string()];
    if let Some(start) = trimmed.find('{') {
        if let Some(end) = trimmed.rfind('}') {
            if end > start {
                candidates.push(trimmed[start..=end].to_string());
            }
        }
    }
    for candidate in candidates {
        if let Ok(value) = serde_json::from_str::<Value>(&candidate) {
            if value.is_object() {
                return Ok(value);
            }
        }
    }
    Err("llm payload is not valid json".to_string())
}

fn merge_heuristic_with_llm(
    heuristic: &Value,
    llm: Value,
) -> Value {
    let heuristic_score = heuristic
        .get("suspicion_score")
        .and_then(Value::as_i64)
        .unwrap_or(50);
    let llm_score = llm
        .get("suspicion_score")
        .and_then(Value::as_i64)
        .unwrap_or(heuristic_score)
        .clamp(0, 100)
        .max(heuristic_score);
    let llm_risk_level = llm
        .get("risk_level")
        .and_then(Value::as_str)
        .map(|value| value.to_lowercase())
        .unwrap_or_else(|| heuristic.get("risk_level").and_then(Value::as_str).unwrap_or("medium").to_string());
    let normalized_level = match llm_risk_level.as_str() {
        "high" => "high",
        "medium" | "low" => "medium",
        _ => heuristic.get("risk_level").and_then(Value::as_str).unwrap_or("medium"),
    };
    json!({
        "risk_level": normalized_level,
        "suspicion_score": llm_score,
        "summary": llm.get("summary").and_then(Value::as_str).filter(|value| !value.trim().is_empty()).unwrap_or_else(|| heuristic.get("summary").and_then(Value::as_str).unwrap_or("")),
        "recommendation": llm.get("recommendation").and_then(Value::as_str).filter(|value| !value.trim().is_empty()).unwrap_or_else(|| heuristic.get("recommendation").and_then(Value::as_str).unwrap_or("")),
        "llm_model": llm.get("llm_model").cloned().unwrap_or(Value::Null),
        "raw_response": llm.get("raw_response").cloned().unwrap_or(Value::Null),
    })
}

async fn send_risk_review_telegram_alert(
    state: &AppState,
    shared_ip: &str,
    matched_user_count: i64,
    reviews: &[(u64, i64, String, String, Option<String>)],
    profile_map: &HashMap<i64, RiskUserProfileRow>,
) -> Result<i64, sqlx::Error> {
    let mut lines = vec![
        "用户风险审查提醒".to_string(),
        format!("共享 IP: {}", shared_ip),
        format!("命中用户数: {}", matched_user_count),
        format!("审查记录数: {}", reviews.len()),
        String::new(),
    ];
    for (_review_id, user_id, summary, recommendation, _llm_model) in reviews.iter().take(8) {
        let user = profile_map.get(user_id);
        lines.push(format!(
            "#{} {}",
            user_id,
            user.map(|item| item.email.clone()).unwrap_or_else(|| "-".to_string())
        ));
        if !summary.trim().is_empty() {
            lines.push(format!("摘要: {}", summary));
        }
        if !recommendation.trim().is_empty() {
            lines.push(format!("建议: {}", recommendation));
        }
        lines.push(String::new());
    }
    let text = lines.join("\n");
    send_telegram_text_to_super_admins(state, &text, "telegram_notify_user_risk_detected")
        .await
        .map_err(sqlx::Error::Protocol)
}
