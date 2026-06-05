use crate::*;

#[derive(Default, Clone)]
pub(crate) struct NodePlanMutationInput {
    pub(crate) owner_user_id: Option<Option<i64>>,
    pub(crate) name: Option<String>,
    pub(crate) content: Option<Option<String>>,
    pub(crate) prices: Option<Value>,
    pub(crate) sell: Option<bool>,
    pub(crate) show: Option<bool>,
    pub(crate) renew: Option<bool>,
    pub(crate) sort: Option<i64>,
    pub(crate) capacity_limit: Option<Option<i64>>,
    pub(crate) device_limit: Option<Option<i64>>,
    pub(crate) speed_limit: Option<Option<i64>>,
    pub(crate) transfer_enable: Option<Option<i64>>,
    pub(crate) node_ids: Option<Vec<i64>>,
    pub(crate) min_trust_level: Option<Option<i64>>,
    pub(crate) allow_trial: Option<bool>,
    pub(crate) is_unlimited_traffic: Option<bool>,
    pub(crate) free_quota_gb_by_trust_level: Option<Option<Value>>,
    pub(crate) visibility_scope: Option<String>,
    pub(crate) access_user_ids: Option<Vec<i64>>,
    pub(crate) share_token: Option<Option<String>>,
}

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct AdminNodeOptionRow {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) user_id: i64,
    pub(crate) status: String,
    pub(crate) owner_email: Option<String>,
}

pub(crate) fn parse_node_plan_mutation_input(
    payload: &Value,
    is_create: bool,
) -> Result<NodePlanMutationInput, Response<Body>> {
    let mut input = NodePlanMutationInput::default();

    if is_create {
        input.name = Some(request_required_string_field(payload, "name")?);
        input.prices = normalize_node_plan_prices(payload.get("prices"));
        if input.prices.is_none() {
            return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
        }
        input.sell = Some(request_required_bool_field(payload, "sell")?);
        input.show = Some(request_required_bool_field(payload, "show")?);
        input.renew = Some(request_required_bool_field(payload, "renew")?);
        input.node_ids = Some(normalize_node_plan_node_ids(payload.get("node_ids")));
        input.min_trust_level = match payload.get("min_trust_level") {
            Some(value) if !value.is_null() => Some(parse_i64_value(value).map(Some).ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?),
            Some(_) => Some(None),
            None => None,
        };
        input.allow_trial = Some(payload.get("allow_trial").and_then(|v| v.as_bool()).unwrap_or(true));
        input.is_unlimited_traffic = Some(payload.get("is_unlimited_traffic").and_then(|v| v.as_bool()).unwrap_or(false));
    } else {
        input.name = request_optional_string_field(payload, "name").and_then(|value| value);
        input.prices = normalize_node_plan_prices(payload.get("prices"));
        input.sell = request_optional_bool_field(payload, "sell");
        input.show = request_optional_bool_field(payload, "show");
        input.renew = request_optional_bool_field(payload, "renew");
        input.node_ids = request_optional_i64_array_field(payload, "node_ids")?;
        input.min_trust_level = request_optional_i64_field(payload, "min_trust_level");
        input.allow_trial = payload.get("allow_trial").and_then(|v| v.as_bool());
        input.is_unlimited_traffic = payload.get("is_unlimited_traffic").and_then(|v| v.as_bool());
    }

    input.content = request_optional_string_field(payload, "content");
    input.owner_user_id = request_optional_i64_field(payload, "owner_user_id");
    input.sort = request_optional_i64_field(payload, "sort").unwrap_or(None);
    input.capacity_limit = request_optional_i64_field(payload, "capacity_limit");
    input.device_limit = request_optional_i64_field(payload, "device_limit");
    input.speed_limit = request_optional_i64_field(payload, "speed_limit");
    input.transfer_enable = Some(Some(payload.get("transfer_enable").and_then(parse_i64_value).unwrap_or(0)));
    input.free_quota_gb_by_trust_level = request_optional_free_quota_field(payload, "free_quota_gb_by_trust_level")?;
    input.visibility_scope = match payload.get("visibility_scope") {
        None => None,
        Some(value) if value.is_null() => Some("public".to_string()),
        Some(Value::String(v)) => {
            let normalized = v.trim().to_lowercase();
            if !matches!(normalized.as_str(), "public" | "link_only" | "assigned_only") {
                return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
            }
            Some(normalized)
        }
        Some(_) => return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed")),
    };
    input.access_user_ids = request_optional_i64_array_field(payload, "access_user_ids")?;
    input.share_token = request_optional_string_field(payload, "share_token");

    if let Some(min_trust_level) = input.min_trust_level.flatten() {
        if !(0..=4).contains(&min_trust_level) {
            return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
        }
    }

    if is_create && input.node_ids.as_ref().map(|ids| ids.is_empty()).unwrap_or(true) {
        return Err(fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"));
    }

    Ok(input)
}

pub(crate) async fn load_all_node_plans(
    state: &AppState,
) -> Result<Vec<PlanRow>, sqlx::Error> {
    sqlx::query_as::<_, PlanRow>(
        "SELECT
            p.id, p.scope, p.owner_user_id, p.min_trust_level, p.free_quota_gb_by_trust_level,
            p.node_ids, p.group_id, p.transfer_enable, COALESCE(p.is_unlimited_traffic, 0) AS is_unlimited_traffic,
            p.name, p.speed_limit, p.`show`, COALESCE(p.visibility_scope, 'public') AS visibility_scope,
            p.access_user_ids, p.share_token, p.sort, p.renew, p.content, p.prices,
            p.reset_traffic_method, p.capacity_limit, p.sell, p.device_limit, p.tags,
            p.created_at, p.updated_at,
            owner.email AS owner_email, owner.linux_do_username AS owner_linux_do_username, owner.linux_do_name AS owner_linux_do_name
         FROM v2_plan p
         LEFT JOIN v2_user owner ON owner.id = p.owner_user_id
         WHERE p.scope = 'node'
         ORDER BY p.id DESC"
    )
    .fetch_all(&state.db)
    .await
}

pub(crate) async fn load_owned_node_plan_by_id(
    state: &AppState,
    owner_user_id: i64,
    plan_id: i64,
) -> Result<Option<PlanRow>, sqlx::Error> {
    sqlx::query_as::<_, PlanRow>(
        "SELECT
            p.id, p.scope, p.owner_user_id, p.min_trust_level, p.free_quota_gb_by_trust_level,
            p.node_ids, p.group_id, p.transfer_enable, COALESCE(p.is_unlimited_traffic, 0) AS is_unlimited_traffic,
            p.name, p.speed_limit, p.`show`, COALESCE(p.visibility_scope, 'public') AS visibility_scope,
            p.access_user_ids, p.share_token, p.sort, p.renew, p.content, p.prices,
            p.reset_traffic_method, p.capacity_limit, p.sell, p.device_limit, p.tags,
            p.created_at, p.updated_at,
            owner.email AS owner_email, owner.linux_do_username AS owner_linux_do_username, owner.linux_do_name AS owner_linux_do_name
         FROM v2_plan p
         LEFT JOIN v2_user owner ON owner.id = p.owner_user_id
         WHERE p.id = ? AND p.scope = 'node' AND p.owner_user_id = ?
         LIMIT 1"
    )
    .bind(plan_id)
    .bind(owner_user_id)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn load_node_plan_by_id_any(
    state: &AppState,
    plan_id: i64,
) -> Result<Option<PlanRow>, sqlx::Error> {
    sqlx::query_as::<_, PlanRow>(
        "SELECT
            p.id, p.scope, p.owner_user_id, p.min_trust_level, p.free_quota_gb_by_trust_level,
            p.node_ids, p.group_id, p.transfer_enable, COALESCE(p.is_unlimited_traffic, 0) AS is_unlimited_traffic,
            p.name, p.speed_limit, p.`show`, COALESCE(p.visibility_scope, 'public') AS visibility_scope,
            p.access_user_ids, p.share_token, p.sort, p.renew, p.content, p.prices,
            p.reset_traffic_method, p.capacity_limit, p.sell, p.device_limit, p.tags,
            p.created_at, p.updated_at,
            owner.email AS owner_email, owner.linux_do_username AS owner_linux_do_username, owner.linux_do_name AS owner_linux_do_name
         FROM v2_plan p
         LEFT JOIN v2_user owner ON owner.id = p.owner_user_id
         WHERE p.id = ? AND p.scope = 'node'
         LIMIT 1"
    )
    .bind(plan_id)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn load_latest_owned_node_plan(
    state: &AppState,
    owner_user_id: i64,
) -> Result<Option<PlanRow>, sqlx::Error> {
    sqlx::query_as::<_, PlanRow>(
        "SELECT
            p.id, p.scope, p.owner_user_id, p.min_trust_level, p.free_quota_gb_by_trust_level,
            p.node_ids, p.group_id, p.transfer_enable, COALESCE(p.is_unlimited_traffic, 0) AS is_unlimited_traffic,
            p.name, p.speed_limit, p.`show`, COALESCE(p.visibility_scope, 'public') AS visibility_scope,
            p.access_user_ids, p.share_token, p.sort, p.renew, p.content, p.prices,
            p.reset_traffic_method, p.capacity_limit, p.sell, p.device_limit, p.tags,
            p.created_at, p.updated_at,
            owner.email AS owner_email, owner.linux_do_username AS owner_linux_do_username, owner.linux_do_name AS owner_linux_do_name
         FROM v2_plan p
         LEFT JOIN v2_user owner ON owner.id = p.owner_user_id
         WHERE p.scope = 'node' AND p.owner_user_id = ?
         ORDER BY p.id DESC
         LIMIT 1"
    )
    .bind(owner_user_id)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn load_admin_node_options(
    state: &AppState,
) -> Result<Vec<AdminNodeOptionRow>, sqlx::Error> {
    sqlx::query_as::<_, AdminNodeOptionRow>(
        "SELECT sn.id, sn.name, sn.user_id, sn.status, owner.email AS owner_email
         FROM server_nodes sn
         LEFT JOIN v2_user owner ON owner.id = sn.user_id
         ORDER BY sn.id DESC"
    )
    .fetch_all(&state.db)
    .await
}

pub(crate) fn normalize_visibility_scope(scope: &str) -> &'static str {
    match scope.trim().to_lowercase().as_str() {
        "link_only" => "link_only",
        "assigned_only" => "assigned_only",
        _ => "public",
    }
}

fn normalize_node_plan_period_key(period: &str) -> Option<&'static str> {
    match period.trim().to_lowercase().as_str() {
        "month_price" | "month" | "monthly" => Some("month_price"),
        "quarter_price" | "quarter" | "quarterly" => Some("quarter_price"),
        "half_year_price" | "half_year" | "semiannual" => Some("half_year_price"),
        "year_price" | "year" | "yearly" => Some("year_price"),
        "two_year_price" | "two_year" | "biennial" => Some("two_year_price"),
        "three_year_price" | "three_year" | "triennial" => Some("three_year_price"),
        "onetime_price" | "onetime" | "lifetime" => Some("onetime_price"),
        "reset_price" | "reset" => Some("reset_price"),
        _ => None,
    }
}

fn normalize_node_plan_prices(value: Option<&Value>) -> Option<Value> {
    let Some(Value::Object(prices)) = value else {
        return None;
    };
    let mut result = Map::new();
    for (period, raw) in prices {
        let Some(normalized_period) = normalize_node_plan_period_key(period) else {
            continue;
        };
        let Some(number) = parse_f64_value(raw) else {
            continue;
        };
        if number < 0.0 {
            continue;
        }
        result.insert(normalized_period.to_string(), Value::from(number));
    }
    if result.is_empty() {
        None
    } else {
        Some(Value::Object(result))
    }
}

fn normalize_node_plan_node_ids(value: Option<&Value>) -> Vec<i64> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(parse_i64_value)
                .filter(|value| *value > 0)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub(crate) fn normalize_node_plan_access_user_ids(value: Option<&Value>) -> Vec<i64> {
    value
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(parse_i64_value)
                .filter(|value| *value > 0)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

pub(crate) fn normalize_node_plan_free_quota(value: Option<&Value>) -> Option<Value> {
    let Some(Value::Object(map)) = value else {
        return None;
    };
    let mut result = Map::new();
    let mut has_positive = false;
    for (level, raw) in map {
        let level_num = level.trim().parse::<i64>().ok()?;
        if !(0..=4).contains(&level_num) {
            return None;
        }
        let quota = parse_f64_value(raw).unwrap_or(0.0).max(0.0);
        result.insert(level_num.to_string(), Value::from(quota));
        if quota > 0.0 {
            has_positive = true;
        }
    }
    if has_positive { Some(Value::Object(result)) } else { None }
}

pub(crate) fn normalize_node_plan_share_token(value: Option<&Value>) -> Option<String> {
    let token = value?.as_str()?.trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

pub(crate) fn normalize_node_plan_visibility_scope(value: Option<&Value>) -> String {
    value
        .and_then(|v| v.as_str())
        .map(|v| normalize_visibility_scope(v).to_string())
        .unwrap_or_else(|| "public".to_string())
}

async fn node_plan_share_token_exists(
    state: &AppState,
    token: &str,
    ignore_plan_id: Option<i64>,
) -> Result<bool, sqlx::Error> {
    let mut sql = String::from("SELECT COUNT(*) FROM v2_plan WHERE share_token = ?");
    if ignore_plan_id.is_some() {
        sql.push_str(" AND id <> ?");
    }
    let mut query = sqlx::query_scalar::<_, i64>(&sql).bind(token);
    if let Some(plan_id) = ignore_plan_id {
        query = query.bind(plan_id);
    }
    let count = query.fetch_one(&state.db).await?;
    Ok(count > 0)
}

pub(crate) async fn resolve_node_plan_share_token(
    state: &AppState,
    visibility_scope: &str,
    requested_token: Option<&str>,
    ignore_plan_id: Option<i64>,
) -> Result<Option<String>, sqlx::Error> {
    let requested_token = requested_token.map(|token| token.trim()).filter(|token| !token.is_empty()).map(|token| token.to_string());
    if visibility_scope != "link_only" {
        return Ok(requested_token);
    }

    if let Some(token) = requested_token {
        if !node_plan_share_token_exists(state, &token, ignore_plan_id).await? {
            return Ok(Some(token));
        }
    }

    loop {
        let token = random_alnum(32);
        if !node_plan_share_token_exists(state, &token, ignore_plan_id).await? {
            return Ok(Some(token));
        }
    }
}
