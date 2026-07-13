use crate::*;

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct TicketRow {
    pub(crate) id: i64,
    pub(crate) user_id: i64,
    pub(crate) node_id: Option<u64>,
    pub(crate) assigned_admin_user_id: Option<u64>,
    pub(crate) subject: String,
    pub(crate) level: i64,
    pub(crate) status: i64,
    pub(crate) reply_status: i64,
    pub(crate) last_reply_user_id: Option<i64>,
    pub(crate) created_at: i64,
    pub(crate) updated_at: i64,
}

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct TicketMessageRow {
    pub(crate) id: i64,
    pub(crate) user_id: i64,
    pub(crate) ticket_id: i64,
    pub(crate) message: String,
    pub(crate) created_at: i64,
    pub(crate) updated_at: i64,
}

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct TicketNodeRow {
    pub(crate) id: u64,
    pub(crate) user_id: i64,
    pub(crate) access_control: Option<SqlxJson<Value>>,
    pub(crate) status: String,
    pub(crate) owner_banned: i8,
    pub(crate) owner_is_super_admin: i8,
}

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct AdminTicketListRow {
    pub(crate) id: i64,
    pub(crate) user_id: i64,
    pub(crate) node_id: Option<u64>,
    pub(crate) assigned_admin_user_id: Option<u64>,
    pub(crate) subject: String,
    pub(crate) level: i64,
    pub(crate) status: i64,
    pub(crate) reply_status: i64,
    pub(crate) last_reply_user_id: Option<i64>,
    pub(crate) created_at: i64,
    pub(crate) updated_at: i64,
    pub(crate) user_email: Option<String>,
}

pub(crate) async fn load_user_tickets(
    state: &AppState,
    user_id: i64,
    node_id: Option<u64>,
) -> Result<Vec<TicketRow>, sqlx::Error> {
    let mut sql = String::from(
        "SELECT id, user_id, node_id, assigned_admin_user_id, subject, level, status, reply_status, last_reply_user_id, created_at, updated_at
         FROM v2_ticket
         WHERE user_id = ?"
    );
    if node_id.is_some() {
        sql.push_str(" AND node_id = ?");
    }
    sql.push_str(" ORDER BY created_at DESC");

    let mut query = sqlx::query_as::<_, TicketRow>(&sql).bind(user_id);
    if let Some(node_id) = node_id {
        query = query.bind(node_id);
    }
    query.fetch_all(&state.db).await
}

pub(crate) async fn load_assigned_admin_tickets(
    state: &AppState,
    assigned_admin_user_id: i64,
    status_filter: Option<i64>,
) -> Result<Vec<TicketRow>, sqlx::Error> {
    let mut sql = String::from(
        "SELECT id, user_id, node_id, assigned_admin_user_id, subject, level, status, reply_status, last_reply_user_id, created_at, updated_at
         FROM v2_ticket
         WHERE assigned_admin_user_id = ?"
    );
    if status_filter.is_some() {
        sql.push_str(" AND status = ?");
    }
    sql.push_str(" ORDER BY created_at DESC");

    let mut query = sqlx::query_as::<_, TicketRow>(&sql).bind(assigned_admin_user_id);
    if let Some(status_filter) = status_filter {
        query = query.bind(status_filter);
    }
    query.fetch_all(&state.db).await
}

pub(crate) async fn load_user_ticket_by_id(
    state: &AppState,
    user_id: i64,
    ticket_id: i64,
) -> Result<Option<TicketRow>, sqlx::Error> {
    sqlx::query_as::<_, TicketRow>(
        "SELECT id, user_id, node_id, assigned_admin_user_id, subject, level, status, reply_status, last_reply_user_id, created_at, updated_at
         FROM v2_ticket
         WHERE id = ? AND user_id = ?
         LIMIT 1"
    )
    .bind(ticket_id)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn load_assigned_admin_ticket_by_id(
    state: &AppState,
    assigned_admin_user_id: i64,
    ticket_id: i64,
) -> Result<Option<TicketRow>, sqlx::Error> {
    sqlx::query_as::<_, TicketRow>(
        "SELECT id, user_id, node_id, assigned_admin_user_id, subject, level, status, reply_status, last_reply_user_id, created_at, updated_at
         FROM v2_ticket
         WHERE id = ? AND assigned_admin_user_id = ?
         LIMIT 1"
    )
    .bind(ticket_id)
    .bind(assigned_admin_user_id)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn load_user_ticket_by_id_for_update(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    ticket_id: i64,
) -> Result<Option<TicketRow>, sqlx::Error> {
    sqlx::query_as::<_, TicketRow>(
        "SELECT id, user_id, node_id, assigned_admin_user_id, subject, level, status, reply_status, last_reply_user_id, created_at, updated_at
         FROM v2_ticket
         WHERE id = ? AND user_id = ?
         LIMIT 1
         FOR UPDATE"
    )
    .bind(ticket_id)
    .bind(user_id)
    .fetch_optional(&mut **tx)
    .await
}

pub(crate) async fn load_assigned_admin_ticket_by_id_for_update(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    assigned_admin_user_id: i64,
    ticket_id: i64,
) -> Result<Option<TicketRow>, sqlx::Error> {
    sqlx::query_as::<_, TicketRow>(
        "SELECT id, user_id, node_id, assigned_admin_user_id, subject, level, status, reply_status, last_reply_user_id, created_at, updated_at
         FROM v2_ticket
         WHERE id = ? AND assigned_admin_user_id = ?
         LIMIT 1
         FOR UPDATE"
    )
    .bind(ticket_id)
    .bind(assigned_admin_user_id)
    .fetch_optional(&mut **tx)
    .await
}

pub(crate) async fn load_ticket_messages(
    state: &AppState,
    ticket_id: i64,
) -> Result<Vec<TicketMessageRow>, sqlx::Error> {
    sqlx::query_as::<_, TicketMessageRow>(
        "SELECT id, user_id, ticket_id, message, created_at, updated_at
         FROM v2_ticket_message
         WHERE ticket_id = ?
         ORDER BY id ASC"
    )
    .bind(ticket_id)
    .fetch_all(&state.db)
    .await
}

pub(crate) async fn load_last_ticket_message(
    state: &AppState,
    ticket_id: i64,
    for_update: bool,
) -> Result<Option<TicketMessageRow>, sqlx::Error> {
    let sql = if for_update {
        "SELECT id, user_id, ticket_id, message, created_at, updated_at
         FROM v2_ticket_message
         WHERE ticket_id = ?
         ORDER BY id DESC
         LIMIT 1
         FOR UPDATE"
    } else {
        "SELECT id, user_id, ticket_id, message, created_at, updated_at
         FROM v2_ticket_message
         WHERE ticket_id = ?
         ORDER BY id DESC
         LIMIT 1"
    };
    sqlx::query_as::<_, TicketMessageRow>(sql)
        .bind(ticket_id)
        .fetch_optional(&state.db)
        .await
}

pub(crate) async fn load_last_ticket_message_with_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    ticket_id: i64,
) -> Result<Option<TicketMessageRow>, sqlx::Error> {
    sqlx::query_as::<_, TicketMessageRow>(
        "SELECT id, user_id, ticket_id, message, created_at, updated_at
         FROM v2_ticket_message
         WHERE ticket_id = ?
         ORDER BY id DESC
         LIMIT 1
         FOR UPDATE"
    )
    .bind(ticket_id)
    .fetch_optional(&mut **tx)
    .await
}

pub(crate) async fn load_ticket_node_by_id(
    state: &AppState,
    node_id: u64,
) -> Result<Option<TicketNodeRow>, sqlx::Error> {
    sqlx::query_as::<_, TicketNodeRow>(
        "SELECT node.id, node.user_id, node.access_control, node.status,
                owner.banned AS owner_banned,
                owner.is_super_admin AS owner_is_super_admin
         FROM server_nodes node
         JOIN v2_user owner ON owner.id = node.user_id
         WHERE node.id = ?
         LIMIT 1"
    )
    .bind(node_id)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn user_can_access_ticket_node(
    state: &AppState,
    user: &BearerUserRow,
    node: &TicketNodeRow,
) -> Result<bool, sqlx::Error> {
    if node.status != "active" || node.owner_banned != 0 {
        return Ok(false);
    }
    if user.is_super_admin != 0 || node.user_id == user.id {
        return Ok(true);
    }

    let blacklisted = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_node_blacklist WHERE node_id = ? AND user_id = ?"
    )
    .bind(node.id)
    .bind(user.id)
    .fetch_one(&state.db)
    .await?;
    if blacklisted > 0 {
        return Ok(false);
    }

    if let Some(expired_at) = user.expired_at {
        if expired_at < Utc::now().timestamp() {
            return Ok(false);
        }
    }

    let has_individual_access = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_node_access WHERE node_id = ? AND user_id = ?"
    )
    .bind(node.id)
    .bind(user.id)
    .fetch_one(&state.db)
    .await?;
    if node.owner_is_super_admin != 0 && has_individual_access > 0 {
        return Ok(true);
    }

    if node.owner_is_super_admin != 0 {
        if let Some(min_trust_level) = node
        .access_control
        .as_ref()
        .and_then(|value| value.0.as_object())
        .and_then(|object| object.get("min_trust_level"))
        .and_then(|value| value.as_i64())
        {
            if user.trust_level >= min_trust_level {
                return Ok(true);
            }
        }
    }

    let now = Utc::now().timestamp();
    let unlimited_allowance = 8_000_000_000_000_000_i64;
    let has_plan_access = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)
         FROM user_plan_subscriptions ups
         JOIN v2_plan plan ON plan.id = ups.plan_id AND plan.scope = 'node'
         WHERE ups.user_id = ?
           AND plan.owner_user_id = ?
           AND JSON_CONTAINS(
               COALESCE(plan.node_ids, JSON_ARRAY()),
               CAST(? AS JSON),
               '$'
           )
           AND ups.status = 1
           AND (ups.expired_at IS NULL OR ups.expired_at > ?)
           AND (
                ups.traffic_allowance_kb >= ?
                OR ups.traffic_allowance_kb > ups.used_traffic_kb
           )"
    )
    .bind(user.id)
    .bind(node.user_id)
    .bind(node.id)
    .bind(now)
    .bind(unlimited_allowance)
    .fetch_one(&state.db)
    .await?;

    Ok(has_plan_access > 0)
}

pub(crate) fn ticket_to_value(ticket: &TicketRow, messages: Option<&[TicketMessageRow]>) -> Value {
    let message_value = messages.map(|messages| {
        Value::Array(
            messages
                .iter()
                .map(|message| ticket_message_to_value(ticket.user_id, message))
                .collect::<Vec<_>>(),
        )
    });

    json!({
        "id": ticket.id,
        "user_id": ticket.user_id,
        "node_id": ticket.node_id,
        "assigned_admin_user_id": ticket.assigned_admin_user_id,
        "level": ticket.level,
        "reply_status": ticket.reply_status,
        "status": ticket.status,
        "subject": ticket.subject,
        "message": message_value.unwrap_or(Value::Null),
        "created_at": ticket.created_at,
        "updated_at": ticket.updated_at,
        "last_reply_user_id": ticket.last_reply_user_id,
    })
}

pub(crate) fn assigned_admin_ticket_to_value(
    ticket: &TicketRow,
    messages: Option<&[TicketMessageRow]>,
) -> Value {
    let message_value = messages.map(|messages| {
        Value::Array(
            messages
                .iter()
                .map(|message| {
                    ticket_message_to_value(
                        ticket
                            .assigned_admin_user_id
                            .and_then(|value| i64::try_from(value).ok())
                            .unwrap_or_default(),
                        message,
                    )
                })
                .collect::<Vec<_>>(),
        )
    });

    json!({
        "id": ticket.id,
        "user_id": ticket.user_id,
        "node_id": ticket.node_id,
        "assigned_admin_user_id": ticket.assigned_admin_user_id,
        "level": ticket.level,
        "reply_status": ticket.reply_status,
        "status": ticket.status,
        "subject": ticket.subject,
        "message": message_value.unwrap_or(Value::Null),
        "created_at": ticket.created_at,
        "updated_at": ticket.updated_at,
        "last_reply_user_id": ticket.last_reply_user_id,
    })
}

pub(crate) async fn load_admin_tickets(
    state: &AppState,
    status_filter: Option<i64>,
    reply_status_filter: Option<&[i64]>,
    email_filter: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<(Vec<AdminTicketListRow>, i64), sqlx::Error> {
    let mut where_clauses = Vec::new();
    if status_filter.is_some() {
        where_clauses.push("t.status = ?");
    }
    if reply_status_filter.is_some() {
        let len = reply_status_filter.map(|items| items.len()).unwrap_or(0);
        where_clauses.push(Box::leak(format!("t.reply_status IN ({})", vec!["?"; len].join(",")).into_boxed_str()));
    }
    if email_filter.is_some() {
        where_clauses.push("u.email LIKE ?");
    }
    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", where_clauses.join(" AND "))
    };

    let base_from = format!(
        " FROM v2_ticket t
          LEFT JOIN v2_user u ON u.id = t.user_id{}",
        where_sql
    );
    let select_sql = format!(
        "SELECT t.id, t.user_id, t.node_id, t.assigned_admin_user_id, t.subject, t.level, t.status, t.reply_status,
                t.last_reply_user_id, t.created_at, t.updated_at, u.email AS user_email
         {}
         ORDER BY t.updated_at DESC
         LIMIT ? OFFSET ?",
        base_from
    );
    let count_sql = format!("SELECT COUNT(*) {}", base_from);

    let mut select = sqlx::query_as::<_, AdminTicketListRow>(&select_sql);
    let mut count = sqlx::query_scalar::<_, i64>(&count_sql);
    if let Some(value) = status_filter {
        select = select.bind(value);
        count = count.bind(value);
    }
    if let Some(values) = reply_status_filter {
        for value in values {
            select = select.bind(*value);
            count = count.bind(*value);
        }
    }
    if let Some(value) = email_filter {
        let pattern = format!("%{}%", value);
        select = select.bind(pattern.clone());
        count = count.bind(pattern);
    }
    let total = count.fetch_one(&state.db).await?;
    let rows = select.bind(limit).bind(offset).fetch_all(&state.db).await?;
    Ok((rows, total))
}

pub(crate) async fn load_admin_ticket_by_id(
    state: &AppState,
    ticket_id: i64,
) -> Result<Option<TicketRow>, sqlx::Error> {
    sqlx::query_as::<_, TicketRow>(
        "SELECT id, user_id, node_id, assigned_admin_user_id, subject, level, status, reply_status, last_reply_user_id, created_at, updated_at
         FROM v2_ticket
         WHERE id = ?
         LIMIT 1"
    )
    .bind(ticket_id)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn load_admin_ticket_by_id_for_update(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    ticket_id: i64,
) -> Result<Option<TicketRow>, sqlx::Error> {
    sqlx::query_as::<_, TicketRow>(
        "SELECT id, user_id, node_id, assigned_admin_user_id, subject, level, status, reply_status, last_reply_user_id, created_at, updated_at
         FROM v2_ticket
         WHERE id = ?
         LIMIT 1
         FOR UPDATE"
    )
    .bind(ticket_id)
    .fetch_optional(&mut **tx)
    .await
}

pub(crate) fn serialize_admin_ticket_list_item(row: &AdminTicketListRow) -> Value {
    json!({
        "id": row.id,
        "user_id": row.user_id,
        "node_id": row.node_id,
        "assigned_admin_user_id": row.assigned_admin_user_id,
        "subject": row.subject,
        "level": row.level,
        "status": row.status,
        "reply_status": row.reply_status,
        "last_reply_user_id": row.last_reply_user_id,
        "created_at": row.created_at,
        "updated_at": row.updated_at,
        "user": {
            "email": row.user_email,
        }
    })
}

pub(crate) async fn serialize_admin_ticket_detail(
    state: &AppState,
    ticket: &TicketRow,
    messages: &[TicketMessageRow],
    user: Option<&BearerUserRow>,
) -> Value {
    let user_value = match user {
        Some(user) => {
            let subscribe_url = build_user_subscribe_url(state, user).await;
            json!({
                "id": user.id,
                "email": user.email,
                "balance": (user.balance as f64) / 100.0,
                "commission_balance": (user.commission_balance as f64) / 100.0,
                "subscribe_url": subscribe_url,
            })
        }
        None => Value::Null,
    };

    json!({
        "id": ticket.id,
        "user_id": ticket.user_id,
        "node_id": ticket.node_id,
        "assigned_admin_user_id": ticket.assigned_admin_user_id,
        "subject": ticket.subject,
        "level": ticket.level,
        "status": ticket.status,
        "reply_status": ticket.reply_status,
        "last_reply_user_id": ticket.last_reply_user_id,
        "created_at": ticket.created_at,
        "updated_at": ticket.updated_at,
        "user": user_value,
        "messages": messages.iter().map(|message| ticket_message_to_value(ticket.user_id, message)).collect::<Vec<_>>(),
    })
}

pub(crate) fn parse_csv_i64_list(value: Option<&String>) -> Vec<i64> {
    value
        .map(|value| {
            value
                .split(|ch| ch == ',' || ch == '|' || ch == '/')
                .filter_map(|item| item.trim().parse::<i64>().ok())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn ticket_message_to_value(ticket_user_id: i64, message: &TicketMessageRow) -> Value {
    json!({
        "id": message.id,
        "ticket_id": message.ticket_id,
        "is_me": message.user_id == ticket_user_id,
        "message": message.message,
        "created_at": message.created_at,
        "updated_at": message.updated_at,
    })
}
