use crate::*;

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct UserPanelStatRow {
    pub(crate) pending_orders: i64,
    pub(crate) open_tickets: i64,
    pub(crate) invitees: i64,
}

pub(crate) async fn load_user_panel_stat_row(
    state: &AppState,
    user_id: i64,
) -> Result<UserPanelStatRow, sqlx::Error> {
    sqlx::query_as::<_, UserPanelStatRow>(
        "SELECT
            CAST(COALESCE(SUM(CASE WHEN source.kind = 'order' THEN source.total ELSE 0 END), 0) AS SIGNED) AS pending_orders,
            CAST(COALESCE(SUM(CASE WHEN source.kind = 'ticket' THEN source.total ELSE 0 END), 0) AS SIGNED) AS open_tickets,
            CAST(COALESCE(SUM(CASE WHEN source.kind = 'invite' THEN source.total ELSE 0 END), 0) AS SIGNED) AS invitees
         FROM (
            SELECT 'order' AS kind, COUNT(*) AS total
            FROM v2_order
            WHERE user_id = ? AND status = 0
            UNION ALL
            SELECT 'ticket' AS kind, COUNT(*) AS total
            FROM v2_ticket
            WHERE user_id = ? AND status = 0
            UNION ALL
            SELECT 'invite' AS kind, COUNT(*) AS total
            FROM v2_user
            WHERE invite_user_id = ?
         ) AS source",
    )
    .bind(user_id)
    .bind(user_id)
    .bind(user_id)
    .fetch_one(&state.db)
    .await
}

pub(crate) async fn load_guest_available_plans(
    state: &AppState,
) -> Result<Vec<PlanRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, PlanRow>(
        "SELECT
            p.id,
            p.scope,
            p.owner_user_id,
            p.min_trust_level,
            p.free_quota_gb_by_trust_level,
            p.node_ids,
            p.group_id,
            p.transfer_enable,
            p.is_unlimited_traffic,
            p.name,
            p.speed_limit,
            p.show,
            p.visibility_scope,
            p.access_user_ids,
            p.share_token,
            p.sort,
            p.renew,
            p.content,
            p.prices,
            p.reset_traffic_method,
            p.capacity_limit,
            p.sell,
            p.device_limit,
            p.tags,
            p.created_at,
            p.updated_at,
            owner.email AS owner_email,
            owner.linux_do_username AS owner_linux_do_username,
            owner.linux_do_name AS owner_linux_do_name
         FROM v2_plan p
         LEFT JOIN v2_user owner ON owner.id = p.owner_user_id
         WHERE p.show = 1
           AND p.sell = 1
           AND (p.visibility_scope IS NULL OR p.visibility_scope = '' OR p.visibility_scope = 'public')
         ORDER BY p.sort ASC, p.id ASC",
    )
    .fetch_all(&state.db)
    .await?;

    let now = Utc::now().timestamp();
    let limited_plan_ids = rows
        .iter()
        .filter_map(|row| match row.capacity_limit {
            Some(limit) if limit > 0 => Some(row.id),
            _ => None,
        })
        .collect::<Vec<_>>();
    let active_counts = load_plan_active_subscription_counts(state, &limited_plan_ids, now).await?;

    let mut result = Vec::new();
    for row in rows {
        let active_count = active_counts.get(&row.id).copied().unwrap_or(0);
        if plan_has_remaining_capacity(row.capacity_limit, active_count) {
            result.push(row);
        }
    }
    Ok(result)
}

pub(crate) async fn load_user_visible_plan_by_share_token(
    state: &AppState,
    share_token: &str,
) -> Result<Option<PlanRow>, sqlx::Error> {
    if !valid_node_plan_share_token(share_token) {
        return Ok(None);
    }
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
         WHERE p.share_token = ?
         LIMIT 1",
    )
    .bind(share_token)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn load_user_visible_plan_by_id(
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
         WHERE p.id = ?
         LIMIT 1",
    )
    .bind(plan_id)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn load_user_public_sellable_plans(
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
         WHERE p.`show` = 1
           AND p.sell = 1
           AND (p.visibility_scope IS NULL OR p.visibility_scope = 'public')
         ORDER BY p.sort",
    )
    .fetch_all(&state.db)
    .await
}

pub(crate) async fn load_owned_node_plans(
    state: &AppState,
    owner_user_id: i64,
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
         WHERE p.scope = 'node' AND p.owner_user_id = ?
         ORDER BY p.id DESC",
    )
    .bind(owner_user_id)
    .fetch_all(&state.db)
    .await
}

pub(crate) async fn plan_available_for_user(
    plan: &PlanRow,
    user: &BearerUserRow,
    purchase_token: Option<&str>,
    state: &AppState,
) -> Result<bool, sqlx::Error> {
    if let Some(min_trust_level) = plan.min_trust_level {
        if (user.trust_level.max(0) as u64) < min_trust_level {
            return Ok(false);
        }
    }

    if user_has_active_plan_subscription(state, user.id, plan.id).await? {
        return Ok(plan.renew);
    }

    if !plan.sell || !plan_has_capacity(state, plan).await? {
        return Ok(false);
    }

    let scope = normalize_visibility_scope(plan.visibility_scope.as_str());
    if scope == "link_only" {
        let token = purchase_token.unwrap_or("").trim();
        return Ok(valid_node_plan_share_token(token)
            && plan
                .share_token
                .as_deref()
                .is_some_and(|expected| valid_node_plan_share_token(expected) && expected == token));
    }
    if scope == "assigned_only" {
        let assigned = plan
            .access_user_ids
            .as_ref()
            .and_then(|json| json.0.as_array().cloned())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| parse_i64_value(&value))
            .any(|id| id == user.id);
        return Ok(assigned);
    }

    Ok(plan.show)
}

pub(crate) async fn load_active_user_plan_subscriptions(
    state: &AppState,
    user_id: i64,
) -> Result<Vec<UserQuotaRow>, sqlx::Error> {
    let now = Utc::now().timestamp();
    sqlx::query_as::<_, UserQuotaRow>(
        "SELECT ups.id, ups.order_id, ups.plan_id, ups.period, ups.traffic_allowance_kb, ups.used_traffic_kb,
                ups.started_at, ups.expired_at,
                p.name AS plan_name, p.scope AS plan_scope, p.node_ids
         FROM user_plan_subscriptions ups
         LEFT JOIN v2_plan p ON p.id = ups.plan_id
         WHERE ups.user_id = ?
           AND ups.status = 1
           AND (ups.expired_at IS NULL OR ups.expired_at > ?)
         ORDER BY ups.started_at DESC",
    )
    .bind(user_id)
    .bind(now)
    .fetch_all(&state.db)
    .await
}

pub(crate) async fn has_plan_capacity(
    state: &AppState,
    plan_id: i64,
    capacity_limit: Option<u64>,
) -> Result<bool, sqlx::Error> {
    let now = Utc::now().timestamp();
    let active_counts = load_plan_active_subscription_counts(state, &[plan_id], now).await?;
    Ok(plan_has_remaining_capacity(
        capacity_limit,
        active_counts.get(&plan_id).copied().unwrap_or(0),
    ))
}

async fn plan_has_capacity(
    state: &AppState,
    plan: &PlanRow,
) -> Result<bool, sqlx::Error> {
    has_plan_capacity(state, plan.id, plan.capacity_limit).await
}

async fn user_has_active_plan_subscription(
    state: &AppState,
    user_id: i64,
    plan_id: i64,
) -> Result<bool, sqlx::Error> {
    let now = Utc::now().timestamp();
    let exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM user_plan_subscriptions
         WHERE user_id = ?
           AND plan_id = ?
           AND status = 1
           AND (expired_at IS NULL OR expired_at > ?)",
    )
    .bind(user_id)
    .bind(plan_id)
    .bind(now)
    .fetch_one(&state.db)
    .await?;
    Ok(exists > 0)
}
