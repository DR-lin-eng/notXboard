use crate::*;

pub(crate) async fn load_user_refund_requests(
    state: &AppState,
    user_id: i64,
) -> Result<Vec<RefundRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, RefundRequestRow>(
        "SELECT rr.id, rr.order_id, rr.trade_no, rr.user_id, rr.plan_id, rr.assigned_admin_user_id,
                rr.status, rr.reason, rr.gateway_amount, rr.gateway_trade_no, rr.epay_pid, rr.epay_url,
                rr.epay_key_encrypted, rr.used_kb, rr.allowance_kb, rr.refund_amount, rr.charged_amount,
                rr.balance_refunded_amount, rr.gateway_refunded_amount, rr.site_balance_fallback_amount,
                rr.gateway_refund_pending_amount, rr.gateway_refund_started_at, rr.voting_ends_at,
                rr.refunded_at, rr.resolved_at, rr.resolved_by_user_id, rr.decision, rr.created_at, rr.updated_at,
                p.name AS plan_name, p.scope AS plan_scope, u.email AS user_email
         FROM order_refund_requests rr
         LEFT JOIN v2_plan p ON p.id = rr.plan_id
         LEFT JOIN v2_user u ON u.id = rr.user_id
         WHERE rr.user_id = ?
         ORDER BY rr.id DESC
         LIMIT 200"
    )
    .bind(user_id as u64)
    .fetch_all(&state.db)
    .await
}

pub(crate) async fn load_user_refund_request_detail(
    state: &AppState,
    refund_id: u64,
    user_id: i64,
) -> Result<Option<RefundRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, RefundRequestRow>(
        "SELECT rr.id, rr.order_id, rr.trade_no, rr.user_id, rr.plan_id, rr.assigned_admin_user_id,
                rr.status, rr.reason, rr.gateway_amount, rr.gateway_trade_no, rr.epay_pid, rr.epay_url,
                rr.epay_key_encrypted, rr.used_kb, rr.allowance_kb, rr.refund_amount, rr.charged_amount,
                rr.balance_refunded_amount, rr.gateway_refunded_amount, rr.site_balance_fallback_amount,
                rr.gateway_refund_pending_amount, rr.gateway_refund_started_at, rr.voting_ends_at,
                rr.refunded_at, rr.resolved_at, rr.resolved_by_user_id, rr.decision, rr.created_at, rr.updated_at,
                p.name AS plan_name, p.scope AS plan_scope, u.email AS user_email
         FROM order_refund_requests rr
         LEFT JOIN v2_plan p ON p.id = rr.plan_id
         LEFT JOIN v2_user u ON u.id = rr.user_id
         WHERE rr.id = ? AND rr.user_id = ?
         LIMIT 1"
    )
    .bind(refund_id)
    .bind(user_id as u64)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn load_assigned_admin_refund_requests(
    state: &AppState,
    admin_user_id: u64,
) -> Result<Vec<RefundRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, RefundRequestRow>(
        "SELECT rr.id, rr.order_id, rr.trade_no, rr.user_id, rr.plan_id, rr.assigned_admin_user_id,
                rr.status, rr.reason, rr.gateway_amount, rr.gateway_trade_no, rr.epay_pid, rr.epay_url,
                rr.epay_key_encrypted, rr.used_kb, rr.allowance_kb, rr.refund_amount, rr.charged_amount,
                rr.balance_refunded_amount, rr.gateway_refunded_amount, rr.site_balance_fallback_amount,
                rr.gateway_refund_pending_amount, rr.gateway_refund_started_at, rr.voting_ends_at,
                rr.refunded_at, rr.resolved_at, rr.resolved_by_user_id, rr.decision, rr.created_at, rr.updated_at,
                p.name AS plan_name, p.scope AS plan_scope, u.email AS user_email
         FROM order_refund_requests rr
         LEFT JOIN v2_plan p ON p.id = rr.plan_id
         LEFT JOIN v2_user u ON u.id = rr.user_id
         WHERE rr.assigned_admin_user_id = ?
         ORDER BY rr.id DESC
         LIMIT 200"
    )
    .bind(admin_user_id)
    .fetch_all(&state.db)
    .await
}

pub(crate) async fn load_all_refund_requests(
    state: &AppState,
    limit: i64,
) -> Result<Vec<RefundRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, RefundRequestRow>(
        "SELECT rr.id, rr.order_id, rr.trade_no, rr.user_id, rr.plan_id, rr.assigned_admin_user_id,
                rr.status, rr.reason, rr.gateway_amount, rr.gateway_trade_no, rr.epay_pid, rr.epay_url,
                rr.epay_key_encrypted, rr.used_kb, rr.allowance_kb, rr.refund_amount, rr.charged_amount,
                rr.balance_refunded_amount, rr.gateway_refunded_amount, rr.site_balance_fallback_amount,
                rr.gateway_refund_pending_amount, rr.gateway_refund_started_at, rr.decision, rr.voting_ends_at,
                rr.resolved_at, rr.resolved_by_user_id, rr.refunded_at, rr.created_at, rr.updated_at,
                p.name AS plan_name, p.scope AS plan_scope,
                u.email AS user_email, u.linux_do_name AS user_linux_do_name, u.linux_do_username AS user_linux_do_username,
                a.email AS assigned_admin_email
         FROM order_refund_requests rr
         LEFT JOIN v2_plan p ON p.id = rr.plan_id
         LEFT JOIN v2_user u ON u.id = rr.user_id
         LEFT JOIN v2_user a ON a.id = rr.assigned_admin_user_id
         ORDER BY rr.id DESC
         LIMIT ?"
    )
    .bind(limit)
    .fetch_all(&state.db)
    .await
}

pub(crate) async fn load_assigned_admin_refund_request_detail(
    state: &AppState,
    refund_id: u64,
    admin_user_id: u64,
) -> Result<Option<RefundRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, RefundRequestRow>(
        "SELECT rr.id, rr.order_id, rr.trade_no, rr.user_id, rr.plan_id, rr.assigned_admin_user_id,
                rr.status, rr.reason, rr.gateway_amount, rr.gateway_trade_no, rr.epay_pid, rr.epay_url,
                rr.epay_key_encrypted, rr.used_kb, rr.allowance_kb, rr.refund_amount, rr.charged_amount,
                rr.balance_refunded_amount, rr.gateway_refunded_amount, rr.site_balance_fallback_amount,
                rr.gateway_refund_pending_amount, rr.gateway_refund_started_at, rr.voting_ends_at,
                rr.refunded_at, rr.resolved_at, rr.resolved_by_user_id, rr.decision, rr.created_at, rr.updated_at,
                p.name AS plan_name, p.scope AS plan_scope, u.email AS user_email
         FROM order_refund_requests rr
         LEFT JOIN v2_plan p ON p.id = rr.plan_id
         LEFT JOIN v2_user u ON u.id = rr.user_id
         WHERE rr.id = ? AND rr.assigned_admin_user_id = ?
         LIMIT 1"
    )
    .bind(refund_id)
    .bind(admin_user_id)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn load_voting_refund_requests(
    state: &AppState,
) -> Result<Vec<RefundRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, RefundRequestRow>(
        "SELECT rr.id, rr.order_id, rr.trade_no, rr.user_id, rr.plan_id, rr.assigned_admin_user_id,
                rr.status, rr.reason, rr.gateway_amount, rr.gateway_trade_no, rr.epay_pid, rr.epay_url,
                rr.epay_key_encrypted, rr.used_kb, rr.allowance_kb, rr.refund_amount, rr.charged_amount,
                rr.balance_refunded_amount, rr.gateway_refunded_amount, rr.site_balance_fallback_amount,
                rr.gateway_refund_pending_amount, rr.gateway_refund_started_at, rr.voting_ends_at,
                rr.refunded_at, rr.resolved_at, rr.resolved_by_user_id, rr.decision, rr.created_at, rr.updated_at,
                p.name AS plan_name, p.scope AS plan_scope, u.email AS user_email
         FROM order_refund_requests rr
         LEFT JOIN v2_plan p ON p.id = rr.plan_id
         LEFT JOIN v2_user u ON u.id = rr.user_id
         WHERE rr.status = 'voting'
           AND (rr.voting_ends_at IS NULL OR rr.voting_ends_at > NOW())
         ORDER BY rr.id DESC
         LIMIT 200"
    )
    .fetch_all(&state.db)
    .await
}

pub(crate) async fn load_refund_request_detail_any(
    state: &AppState,
    refund_id: u64,
) -> Result<Option<RefundRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, RefundRequestRow>(
        "SELECT rr.id, rr.order_id, rr.trade_no, rr.user_id, rr.plan_id, rr.assigned_admin_user_id,
                rr.status, rr.reason, rr.gateway_amount, rr.gateway_trade_no, rr.epay_pid, rr.epay_url,
                rr.epay_key_encrypted, rr.used_kb, rr.allowance_kb, rr.refund_amount, rr.charged_amount,
                rr.balance_refunded_amount, rr.gateway_refunded_amount, rr.site_balance_fallback_amount,
                rr.gateway_refund_pending_amount, rr.gateway_refund_started_at, rr.voting_ends_at,
                rr.refunded_at, rr.resolved_at, rr.resolved_by_user_id, rr.decision, rr.created_at, rr.updated_at,
                p.name AS plan_name, p.scope AS plan_scope, u.email AS user_email
         FROM order_refund_requests rr
         LEFT JOIN v2_plan p ON p.id = rr.plan_id
         LEFT JOIN v2_user u ON u.id = rr.user_id
         WHERE rr.id = ?
         LIMIT 1"
    )
    .bind(refund_id)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn load_open_voting_refund_request_detail(
    state: &AppState,
    refund_id: u64,
) -> Result<Option<RefundRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, RefundRequestRow>(
        "SELECT rr.id, rr.order_id, rr.trade_no, rr.user_id, rr.plan_id, rr.assigned_admin_user_id,
                rr.status, rr.reason, rr.gateway_amount, rr.gateway_trade_no, rr.epay_pid, rr.epay_url,
                rr.epay_key_encrypted, rr.used_kb, rr.allowance_kb, rr.refund_amount, rr.charged_amount,
                rr.balance_refunded_amount, rr.gateway_refunded_amount, rr.site_balance_fallback_amount,
                rr.gateway_refund_pending_amount, rr.gateway_refund_started_at, rr.voting_ends_at,
                rr.refunded_at, rr.resolved_at, rr.resolved_by_user_id, rr.decision, rr.created_at, rr.updated_at,
                p.name AS plan_name, p.scope AS plan_scope, u.email AS user_email
         FROM order_refund_requests rr
         LEFT JOIN v2_plan p ON p.id = rr.plan_id
         LEFT JOIN v2_user u ON u.id = rr.user_id
         WHERE rr.id = ?
           AND rr.status = 'voting'
           AND (rr.voting_ends_at IS NULL OR rr.voting_ends_at > NOW())
         LIMIT 1"
    )
    .bind(refund_id)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn load_expired_voting_refund_requests(
    state: &AppState,
    now: chrono::DateTime<Utc>,
    limit: i64,
) -> Result<Vec<RefundRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, RefundRequestRow>(
        "SELECT rr.id, rr.order_id, rr.trade_no, rr.user_id, rr.plan_id, rr.assigned_admin_user_id,
                rr.status, rr.reason, rr.gateway_amount, rr.gateway_trade_no, rr.epay_pid, rr.epay_url,
                rr.epay_key_encrypted, rr.used_kb, rr.allowance_kb, rr.refund_amount, rr.charged_amount,
                rr.balance_refunded_amount, rr.gateway_refunded_amount, rr.site_balance_fallback_amount,
                rr.gateway_refund_pending_amount, rr.gateway_refund_started_at, rr.voting_ends_at,
                rr.refunded_at, rr.resolved_at, rr.resolved_by_user_id, rr.decision, rr.created_at, rr.updated_at,
                p.name AS plan_name, p.scope AS plan_scope, u.email AS user_email
         FROM order_refund_requests rr
         LEFT JOIN v2_plan p ON p.id = rr.plan_id
         LEFT JOIN v2_user u ON u.id = rr.user_id
         WHERE rr.status = 'voting'
           AND rr.voting_ends_at IS NOT NULL
           AND rr.voting_ends_at <= ?
         ORDER BY rr.id ASC
         LIMIT ?"
    )
    .bind(now)
    .bind(limit)
    .fetch_all(&state.db)
    .await
}

pub(crate) async fn load_refund_evidences(
    state: &AppState,
    refund_id: u64,
) -> Result<Vec<RefundEvidenceRow>, sqlx::Error> {
    sqlx::query_as::<_, RefundEvidenceRow>(
        "SELECT e.id, e.refund_request_id, e.user_id, e.role, e.content, e.created_at, e.updated_at,
                u.email AS user_email
         FROM order_refund_evidences e
         LEFT JOIN v2_user u ON u.id = e.user_id
         WHERE e.refund_request_id = ?
         ORDER BY e.id ASC"
    )
    .bind(refund_id)
    .fetch_all(&state.db)
    .await
}

pub(crate) async fn load_refund_votes(
    state: &AppState,
    refund_id: u64,
) -> Result<Vec<RefundVoteRow>, sqlx::Error> {
    sqlx::query_as::<_, RefundVoteRow>(
        "SELECT id, refund_request_id, user_id, vote, created_at, updated_at
         FROM order_refund_votes
         WHERE refund_request_id = ?
         ORDER BY id ASC"
    )
    .bind(refund_id)
    .fetch_all(&state.db)
    .await
}

pub(crate) fn serialize_refund_request_summary(req: &RefundRequestRow) -> Value {
    json!({
        "id": req.id,
        "order_id": req.order_id,
        "trade_no": req.trade_no,
        "user_id": req.user_id,
        "plan_id": req.plan_id,
        "assigned_admin_user_id": req.assigned_admin_user_id,
        "status": req.status,
        "reason": req.reason,
        "gateway_amount": req.gateway_amount,
        "gateway_trade_no": req.gateway_trade_no,
        "used_kb": req.used_kb,
        "allowance_kb": req.allowance_kb,
        "refund_amount": req.refund_amount,
        "charged_amount": req.charged_amount,
        "balance_refunded_amount": req.balance_refunded_amount,
        "gateway_refunded_amount": req.gateway_refunded_amount,
        "site_balance_fallback_amount": req.site_balance_fallback_amount,
        "gateway_refund_pending_amount": req.gateway_refund_pending_amount,
        "gateway_refund_started_at": format_optional_naive_datetime(req.gateway_refund_started_at),
        "voting_ends_at": format_optional_naive_datetime(req.voting_ends_at),
        "refunded_at": format_optional_naive_datetime(req.refunded_at),
        "resolved_at": format_optional_naive_datetime(req.resolved_at),
        "resolved_by_user_id": req.resolved_by_user_id,
        "decision": req.decision,
        "created_at": format_optional_naive_datetime(req.created_at),
        "updated_at": format_optional_naive_datetime(req.updated_at),
        "plan": {
            "id": req.plan_id,
            "name": req.plan_name,
            "scope": req.plan_scope,
        },
        "user": {
            "id": req.user_id,
            "email": req.user_email,
        }
    })
}

pub(crate) fn serialize_public_refund_vote_summary(req: &RefundRequestRow) -> Value {
    json!({
        "id": req.id,
        "status": req.status,
        "reason": req.reason,
        "used_kb": req.used_kb,
        "allowance_kb": req.allowance_kb,
        "refund_amount": req.refund_amount,
        "charged_amount": req.charged_amount,
        "voting_ends_at": format_optional_naive_datetime(req.voting_ends_at),
        "created_at": format_optional_naive_datetime(req.created_at),
        "updated_at": format_optional_naive_datetime(req.updated_at),
        "plan": {
            "id": req.plan_id,
            "name": req.plan_name,
            "scope": req.plan_scope,
        },
    })
}

pub(crate) fn serialize_public_refund_vote_detail(
    req: &RefundRequestRow,
    evidences: &[RefundEvidenceRow],
    votes: &[RefundVoteRow],
    viewer_user_id: u64,
) -> Value {
    let approve = votes.iter().filter(|vote| vote.vote == "approve").count() as i64;
    let deny = votes.iter().filter(|vote| vote.vote == "deny").count() as i64;
    let my_vote = votes
        .iter()
        .find(|vote| vote.user_id == viewer_user_id)
        .map(|vote| vote.vote.clone());
    let mut value = serialize_public_refund_vote_summary(req);
    value["evidences"] = Value::Array(
        evidences
            .iter()
            .map(|evidence| {
                json!({
                    "id": evidence.id,
                    "role": evidence.role,
                    "content": evidence.content,
                    "is_mine": evidence.user_id == viewer_user_id,
                    "created_at": format_optional_naive_datetime(evidence.created_at),
                    "updated_at": format_optional_naive_datetime(evidence.updated_at),
                })
            })
            .collect(),
    );
    value["vote_counts"] = json!({ "approve": approve, "deny": deny });
    value["my_vote"] = my_vote.map(Value::String).unwrap_or(Value::Null);
    value
}

pub(crate) fn serialize_refund_request_detail(
    req: &RefundRequestRow,
    evidences: &[RefundEvidenceRow],
    votes: &[RefundVoteRow],
    my_vote_user_id: Option<u64>,
) -> Value {
    let approve = votes.iter().filter(|vote| vote.vote == "approve").count() as i64;
    let deny = votes.iter().filter(|vote| vote.vote == "deny").count() as i64;
    let my_vote = my_vote_user_id
        .and_then(|user_id| votes.iter().find(|vote| vote.user_id == user_id))
        .map(|vote| vote.vote.clone());

    let mut value = serialize_refund_request_summary(req);
    value["evidences"] = Value::Array(
        evidences
            .iter()
            .map(|evidence| {
                json!({
                    "id": evidence.id,
                    "refund_request_id": evidence.refund_request_id,
                    "user_id": evidence.user_id,
                    "role": evidence.role,
                    "content": evidence.content,
                    "created_at": format_optional_naive_datetime(evidence.created_at),
                    "updated_at": format_optional_naive_datetime(evidence.updated_at),
                    "user": {
                        "id": evidence.user_id,
                        "email": evidence.user_email,
                    }
                })
            })
            .collect(),
    );
    value["votes"] = Value::Array(
        votes.iter()
            .map(|vote| {
                json!({
                    "id": vote.id,
                    "refund_request_id": vote.refund_request_id,
                    "user_id": vote.user_id,
                    "vote": vote.vote,
                    "created_at": format_optional_naive_datetime(vote.created_at),
                    "updated_at": format_optional_naive_datetime(vote.updated_at),
                })
            })
            .collect(),
    );
    value["vote_counts"] = json!({
        "approve": approve,
        "deny": deny,
    });
    if let Some(my_vote) = my_vote {
        value["my_vote"] = Value::String(my_vote);
    } else if my_vote_user_id.is_some() {
        value["my_vote"] = Value::Null;
    }
    value
}

pub(crate) async fn finalize_single_expired_refund_voting(
    state: &AppState,
    refund_id: u64,
    now: chrono::DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let mut tx = state.db.begin().await?;
    let locked = sqlx::query(
        "SELECT id, status, voting_ends_at
         FROM order_refund_requests
         WHERE id = ?
         LIMIT 1
         FOR UPDATE"
    )
    .bind(refund_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(locked) = locked else {
        tx.commit().await?;
        return Ok(());
    };

    let status = locked.try_get::<String, _>("status").unwrap_or_default();
    let voting_ends_at = locked
        .try_get::<Option<chrono::DateTime<Utc>>, _>("voting_ends_at")
        .unwrap_or(None);
    if status != "voting" || voting_ends_at.map(|ts| ts > now).unwrap_or(true) {
        tx.commit().await?;
        return Ok(());
    }

    let approve: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM order_refund_votes WHERE refund_request_id = ? AND vote = 'approve'"
    )
    .bind(refund_id)
    .fetch_one(&mut *tx)
    .await?;
    let deny: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM order_refund_votes WHERE refund_request_id = ? AND vote = 'deny'"
    )
    .bind(refund_id)
    .fetch_one(&mut *tx)
    .await?;
    let decision = if approve > deny { "approve" } else { "deny" };
    let next_status = if decision == "approve" { "approved" } else { "denied" };

    sqlx::query(
        "UPDATE order_refund_requests
         SET decision = ?,
             resolved_at = NOW(),
             resolved_by_user_id = NULL,
             status = ?,
             updated_at = NOW()
         WHERE id = ?"
    )
    .bind(decision)
    .bind(next_status)
    .bind(refund_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    if decision == "approve" {
        auto_refund_approved_request(state, refund_id).await?;
    }

    Ok(())
}

pub(crate) async fn deny_refund_request_as_admin(
    state: &AppState,
    refund_id: u64,
    admin_user_id: u64,
    reason: Option<String>,
    required_assignee_user_id: Option<u64>,
) -> Result<(), Response<Body>> {
    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let locked = if let Some(assignee_user_id) = required_assignee_user_id {
        load_assigned_admin_refund_request_detail_for_update(
            &mut tx,
            refund_id,
            assignee_user_id,
        )
        .await
        .map_err(internal_error)?
    } else {
        load_refund_request_detail_any_for_update(&mut tx, refund_id)
            .await
            .map_err(internal_error)?
    };
    let Some(locked) = locked else {
        tx.rollback().await.ok();
        return Err(fail_json_response(StatusCode::NOT_FOUND, "Refund request not found"));
    };
    if !matches!(locked.status.as_str(), "pending" | "voting") {
        tx.rollback().await.ok();
        return Err(fail_json_response(StatusCode::BAD_REQUEST, "Refund request is not actionable"));
    }

    let mut update_sql = String::from(
        "UPDATE order_refund_requests
         SET status = 'denied',
             decision = 'deny',
             resolved_at = COALESCE(resolved_at, NOW()),
             resolved_by_user_id = ?,
             reason = COALESCE(?, reason),
             updated_at = NOW()
         WHERE id = ? AND status IN ('pending', 'voting')"
    );
    if required_assignee_user_id.is_some() {
        update_sql.push_str(" AND assigned_admin_user_id = ?");
    }
    let mut update = sqlx::query(&update_sql)
    .bind(admin_user_id)
    .bind(reason.as_deref())
    .bind(refund_id);
    if let Some(assignee_user_id) = required_assignee_user_id {
        update = update.bind(assignee_user_id);
    }
    let updated = update.execute(&mut *tx)
    .await
    .map_err(internal_error)?;
    if updated.rows_affected() != 1 {
        tx.rollback().await.ok();
        return Err(fail_json_response(StatusCode::CONFLICT, "Refund assignment changed"));
    }

    tx.commit().await.map_err(internal_error)?;
    Ok(())
}

pub(crate) async fn approve_refund_request_as_admin(
    state: &AppState,
    refund_id: u64,
    admin_user_id: u64,
    required_assignee_user_id: Option<u64>,
) -> Result<(), Response<Body>> {
    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let locked = if let Some(assignee_user_id) = required_assignee_user_id {
        load_assigned_admin_refund_request_detail_for_update(
            &mut tx,
            refund_id,
            assignee_user_id,
        )
        .await
        .map_err(internal_error)?
    } else {
        load_refund_request_detail_any_for_update(&mut tx, refund_id)
            .await
            .map_err(internal_error)?
    };
    let Some(locked) = locked else {
        tx.rollback().await.ok();
        return Err(fail_json_response(StatusCode::NOT_FOUND, "Refund request not found"));
    };
    if locked.status == "refunded" {
        tx.rollback().await.ok();
        return Ok(());
    }
    if !matches!(locked.status.as_str(), "pending" | "voting" | "approved" | "failed" | "processing") {
        tx.rollback().await.ok();
        return Err(fail_json_response(StatusCode::BAD_REQUEST, "Refund request is not actionable"));
    }

    let order = load_refund_execution_order_for_update(&mut tx, locked.order_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::INTERNAL_SERVER_ERROR, "Refund request data not found"))?;
    let plan = load_refund_create_plan_by_id(state, locked.plan_id as i64)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::INTERNAL_SERVER_ERROR, "Refund request data not found"))?;
    let user_row = load_refund_execution_user_for_update(&mut tx, locked.user_id)
        .await
        .map_err(internal_error)?
        .ok_or_else(|| fail_json_response(StatusCode::INTERNAL_SERVER_ERROR, "Refund request data not found"))?;

    let refund_order = RefundCreateOrderRow {
        id: order.id as i64,
        trade_no: order.trade_no.clone(),
        user_id: order.user_id as i64,
        plan_id: order.plan_id as i64,
        status: order.status,
        total_amount: order.total_amount,
        handling_amount: order.handling_amount,
        balance_amount: order.balance_amount,
        callback_no: order.callback_no.clone(),
        epay_pid: order.epay_pid.clone(),
        epay_url: order.epay_url.clone(),
        epay_key_encrypted: order.epay_key_encrypted.clone(),
        paid_at: order.paid_at,
        created_at: order.created_at,
    };

    let (used_kb, allowance_kb) = calculate_refund_usage_kb(state, &refund_order, &plan)
        .await
        .map_err(internal_error)?;
    let (target_refund_amount, charged_cents) = calculate_refund_cents(&refund_order, used_kb, allowance_kb);

    let balance_refund_cents = order.balance_amount.unwrap_or(0).min(target_refund_amount);
    let gateway_target_refund = (target_refund_amount - balance_refund_cents).max(0);
    let gateway_paid_cents = order.total_amount + order.handling_amount.unwrap_or(0);
    let gateway_target_refund = gateway_target_refund.min(gateway_paid_cents);

    let should_fallback_to_balance = order.callback_no.as_deref().unwrap_or("").is_empty()
        || order.epay_pid.as_deref().unwrap_or("").is_empty()
        || order.epay_url.as_deref().unwrap_or("").is_empty()
        || order.epay_key_encrypted.as_deref().unwrap_or("").is_empty();

    let additional_balance_credit = if should_fallback_to_balance { gateway_target_refund } else { 0 };
    let total_balance_credit = balance_refund_cents + additional_balance_credit;
    let settled_gateway_refund = if should_fallback_to_balance { 0 } else { gateway_target_refund };
    let site_balance_fallback_amount = if should_fallback_to_balance { gateway_target_refund } else { 0 };
    let final_status = if total_balance_credit + settled_gateway_refund >= target_refund_amount {
        "refunded"
    } else {
        "processing"
    };

    sqlx::query("UPDATE v2_user SET balance = ?, updated_at = ? WHERE id = ?")
        .bind(user_row.balance + total_balance_credit)
        .bind(Utc::now().timestamp())
        .bind(user_row.id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;

    sqlx::query(
        "UPDATE user_plan_subscriptions
         SET status = 3, updated_at = ?
         WHERE user_id = ? AND order_id = ?"
    )
    .bind(Utc::now().timestamp())
    .bind(order.user_id)
    .bind(order.id)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;

    let mut update_sql = String::from(
        "UPDATE order_refund_requests
         SET status = ?,
             decision = 'approve',
             used_kb = ?,
             allowance_kb = ?,
             refund_amount = ?,
             charged_amount = ?,
             balance_refunded_amount = ?,
             gateway_refunded_amount = ?,
             site_balance_fallback_amount = ?,
             gateway_refund_pending_amount = 0,
             gateway_refund_started_at = NULL,
             refunded_at = CASE WHEN ? = 'refunded' THEN COALESCE(refunded_at, NOW()) ELSE refunded_at END,
             resolved_at = COALESCE(resolved_at, NOW()),
             resolved_by_user_id = COALESCE(resolved_by_user_id, ?),
             reason = CASE
                 WHEN ? > 0 THEN TRIM(CONCAT(COALESCE(reason, ''), CASE WHEN COALESCE(reason, '') = '' THEN '' ELSE '\n' END, 'Gateway refund failed; credited to site balance instead'))
                 ELSE reason
             END,
             updated_at = NOW()
         WHERE id = ?
           AND status IN ('pending', 'voting', 'approved', 'failed', 'processing')"
    );
    if required_assignee_user_id.is_some() {
        update_sql.push_str(" AND assigned_admin_user_id = ?");
    }
    let mut update = sqlx::query(&update_sql)
    .bind(final_status)
    .bind(used_kb)
    .bind(allowance_kb)
    .bind(target_refund_amount)
    .bind(charged_cents)
    .bind(total_balance_credit)
    .bind(settled_gateway_refund)
    .bind(site_balance_fallback_amount)
    .bind(final_status)
    .bind(admin_user_id)
    .bind(site_balance_fallback_amount)
    .bind(refund_id);
    if let Some(assignee_user_id) = required_assignee_user_id {
        update = update.bind(assignee_user_id);
    }
    let updated = update.execute(&mut *tx)
    .await
    .map_err(internal_error)?;
    if updated.rows_affected() != 1 {
        tx.rollback().await.ok();
        return Err(fail_json_response(StatusCode::CONFLICT, "Refund assignment changed"));
    }

    tx.commit().await.map_err(internal_error)?;
    clear_all_authorization_caches(state);
    Ok(())
}

pub(crate) async fn load_refund_create_order_by_trade_no(
    state: &AppState,
    user_id: i64,
    trade_no: &str,
) -> Result<Option<RefundCreateOrderRow>, sqlx::Error> {
    sqlx::query_as::<_, RefundCreateOrderRow>(
        "SELECT id, trade_no, user_id, plan_id, status, total_amount, handling_amount, balance_amount,
                callback_no, epay_pid, epay_url, epay_key_encrypted, paid_at, created_at
         FROM v2_order
         WHERE trade_no = ? AND user_id = ?
         LIMIT 1"
    )
    .bind(trade_no)
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn load_refund_create_plan_by_id(
    state: &AppState,
    plan_id: i64,
) -> Result<Option<RefundCreatePlanRow>, sqlx::Error> {
    sqlx::query_as::<_, RefundCreatePlanRow>(
        "SELECT id, scope, owner_user_id, transfer_enable, node_ids
         FROM v2_plan
         WHERE id = ?
         LIMIT 1"
    )
    .bind(plan_id)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn load_assigned_admin_refund_request_detail_for_update(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    refund_id: u64,
    admin_user_id: u64,
) -> Result<Option<RefundRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, RefundRequestRow>(
        "SELECT rr.id, rr.order_id, rr.trade_no, rr.user_id, rr.plan_id, rr.assigned_admin_user_id,
                rr.status, rr.reason, rr.gateway_amount, rr.gateway_trade_no, rr.epay_pid, rr.epay_url,
                rr.epay_key_encrypted, rr.used_kb, rr.allowance_kb, rr.refund_amount, rr.charged_amount,
                rr.balance_refunded_amount, rr.gateway_refunded_amount, rr.site_balance_fallback_amount,
                rr.gateway_refund_pending_amount, rr.gateway_refund_started_at, rr.voting_ends_at,
                rr.refunded_at, rr.resolved_at, rr.resolved_by_user_id, rr.decision, rr.created_at, rr.updated_at,
                p.name AS plan_name, p.scope AS plan_scope, u.email AS user_email
         FROM order_refund_requests rr
         LEFT JOIN v2_plan p ON p.id = rr.plan_id
         LEFT JOIN v2_user u ON u.id = rr.user_id
         WHERE rr.id = ? AND rr.assigned_admin_user_id = ?
         LIMIT 1
         FOR UPDATE"
    )
    .bind(refund_id)
    .bind(admin_user_id)
    .fetch_optional(&mut **tx)
    .await
}

pub(crate) async fn calculate_refund_usage_kb(
    state: &AppState,
    order: &RefundCreateOrderRow,
    plan: &RefundCreatePlanRow,
) -> Result<(i64, i64), sqlx::Error> {
    let node_ids = plan
        .node_ids
        .as_ref()
        .and_then(|value| value.0.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| value.as_i64())
        .filter(|value| *value > 0)
        .collect::<Vec<_>>();

    let valid_node_ids = if node_ids.is_empty() {
        Vec::new()
    } else {
        let placeholders = vec!["?"; node_ids.len()].join(",");
        let sql = format!("SELECT id FROM server_nodes WHERE id IN ({})", placeholders);
        let mut query = sqlx::query_scalar::<_, u64>(&sql);
        for node_id in &node_ids {
            query = query.bind(*node_id);
        }
        query
            .fetch_all(&state.db)
            .await?
            .into_iter()
            .map(|value| value as i64)
            .collect::<Vec<_>>()
    };

    let order_ts = order.paid_at.unwrap_or(order.created_at);
    let start = chrono::DateTime::from_timestamp(order_ts, 0)
        .map(|dt| dt.date_naive())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());
    let end = chrono::Utc::now().date_naive();

    let used_kb = if valid_node_ids.is_empty() {
        0
    } else {
        let placeholders = vec!["?"; valid_node_ids.len()].join(",");
        let sql = format!(
            "SELECT CAST(COALESCE(SUM(upload_traffic + download_traffic), 0) AS SIGNED)
             FROM node_traffic_records
             WHERE user_id = ?
               AND node_id IN ({})
               AND record_date BETWEEN ? AND ?",
            placeholders
        );
        let mut query = sqlx::query_scalar::<_, i64>(&sql)
            .bind(order.user_id)
            .bind(start.to_string())
            .bind(end.to_string());
        for node_id in &valid_node_ids {
            query = query.bind(*node_id);
        }
        query.fetch_one(&state.db).await?
    };

    let allowance_kb = plan
        .transfer_enable
        .unwrap_or_default()
        .min(i64::MAX as u64) as i64
        * 1024
        * 1024;

    Ok((used_kb, allowance_kb))
}

pub(crate) fn calculate_refund_cents(
    order: &RefundCreateOrderRow,
    used_kb: i64,
    allowance_kb: i64,
) -> (i64, i64) {
    let paid_cents = order.total_amount + order.handling_amount.unwrap_or(0) + order.balance_amount.unwrap_or(0);
    if paid_cents <= 0 {
        return (0, 0);
    }
    if allowance_kb <= 0 {
        return (paid_cents, 0);
    }

    let ratio = (used_kb as f64 / allowance_kb as f64).clamp(0.0, 1.0);
    let charged = ((paid_cents as f64) * ratio).round() as i64;
    let charged = charged.clamp(0, paid_cents);
    let refund = paid_cents - charged;
    (refund, charged)
}

pub(crate) async fn load_refund_execution_order_for_update(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    order_id: u64,
) -> Result<Option<RefundExecutionOrderRow>, sqlx::Error> {
    sqlx::query_as::<_, RefundExecutionOrderRow>(
        "SELECT id, trade_no, user_id, plan_id, status, total_amount, handling_amount, balance_amount,
                callback_no, epay_pid, epay_url, epay_key_encrypted, paid_at, created_at
         FROM v2_order
         WHERE id = ?
         LIMIT 1
         FOR UPDATE"
    )
    .bind(order_id)
    .fetch_optional(&mut **tx)
    .await
}

pub(crate) async fn load_refund_execution_user_for_update(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: u64,
) -> Result<Option<RefundExecutionUserRow>, sqlx::Error> {
    sqlx::query_as::<_, RefundExecutionUserRow>(
        "SELECT id, balance
         FROM v2_user
         WHERE id = ?
         LIMIT 1
         FOR UPDATE"
    )
    .bind(user_id)
    .fetch_optional(&mut **tx)
    .await
}

pub(crate) async fn auto_refund_approved_request(
    state: &AppState,
    refund_id: u64,
) -> Result<(), sqlx::Error> {
    let mut tx = state.db.begin().await?;
    let locked = load_refund_request_detail_any_for_update(&mut tx, refund_id).await?;
    let Some(locked) = locked else {
        tx.commit().await?;
        return Ok(());
    };
    if locked.status == "refunded" {
        tx.commit().await?;
        return Ok(());
    }
    if locked.decision.as_deref() != Some("approve") {
        tx.commit().await?;
        return Ok(());
    }
    if !matches!(locked.status.as_str(), "approved" | "failed" | "processing") {
        tx.commit().await?;
        return Ok(());
    }

    let order = load_refund_execution_order_for_update(&mut tx, locked.order_id).await?;
    let Some(order) = order else {
        tx.commit().await?;
        return Ok(());
    };
    let plan = load_refund_create_plan_by_id(state, locked.plan_id as i64).await?;
    let Some(plan) = plan else {
        tx.commit().await?;
        return Ok(());
    };
    let user_row = load_refund_execution_user_for_update(&mut tx, locked.user_id).await?;
    let Some(user_row) = user_row else {
        tx.commit().await?;
        return Ok(());
    };

    let refund_order = RefundCreateOrderRow {
        id: order.id as i64,
        trade_no: order.trade_no.clone(),
        user_id: order.user_id as i64,
        plan_id: order.plan_id as i64,
        status: order.status,
        total_amount: order.total_amount,
        handling_amount: order.handling_amount,
        balance_amount: order.balance_amount,
        callback_no: order.callback_no.clone(),
        epay_pid: order.epay_pid.clone(),
        epay_url: order.epay_url.clone(),
        epay_key_encrypted: order.epay_key_encrypted.clone(),
        paid_at: order.paid_at,
        created_at: order.created_at,
    };
    let (used_kb, allowance_kb) = calculate_refund_usage_kb(state, &refund_order, &plan).await?;
    let (refund_cents, charged_cents) = calculate_refund_cents(&refund_order, used_kb, allowance_kb);

    let balance_refund_cents = order.balance_amount.unwrap_or(0).min(refund_cents);
    let gateway_target_refund = (refund_cents - balance_refund_cents).max(0);
    let gateway_paid_cents = order.total_amount + order.handling_amount.unwrap_or(0);
    let gateway_target_refund = gateway_target_refund.min(gateway_paid_cents);
    let should_fallback_to_balance = order.callback_no.as_deref().unwrap_or("").is_empty()
        || order.epay_pid.as_deref().unwrap_or("").is_empty()
        || order.epay_url.as_deref().unwrap_or("").is_empty()
        || order.epay_key_encrypted.as_deref().unwrap_or("").is_empty();
    let additional_balance_credit = if should_fallback_to_balance {
        gateway_target_refund
    } else {
        0
    };
    let total_balance_credit = balance_refund_cents + additional_balance_credit;
    let settled_gateway_refund = if should_fallback_to_balance { 0 } else { gateway_target_refund };
    let site_balance_fallback_amount = if should_fallback_to_balance { gateway_target_refund } else { 0 };
    let final_status = if total_balance_credit + settled_gateway_refund >= refund_cents {
        "refunded"
    } else {
        "processing"
    };

    sqlx::query("UPDATE v2_user SET balance = ?, updated_at = ? WHERE id = ?")
        .bind(user_row.balance + total_balance_credit)
        .bind(Utc::now().timestamp())
        .bind(user_row.id)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "UPDATE user_plan_subscriptions
         SET status = 3, updated_at = ?
         WHERE user_id = ? AND order_id = ?"
    )
    .bind(Utc::now().timestamp())
    .bind(order.user_id)
    .bind(order.id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE order_refund_requests
         SET status = ?,
             used_kb = ?,
             allowance_kb = ?,
             refund_amount = ?,
             charged_amount = ?,
             balance_refunded_amount = ?,
             gateway_refunded_amount = ?,
             site_balance_fallback_amount = ?,
             gateway_refund_pending_amount = 0,
             gateway_refund_started_at = NULL,
             refunded_at = CASE WHEN ? = 'refunded' THEN COALESCE(refunded_at, NOW()) ELSE refunded_at END,
             updated_at = NOW()
         WHERE id = ?"
    )
    .bind(final_status)
    .bind(used_kb)
    .bind(allowance_kb)
    .bind(refund_cents)
    .bind(charged_cents)
    .bind(total_balance_credit)
    .bind(settled_gateway_refund)
    .bind(site_balance_fallback_amount)
    .bind(final_status)
    .bind(refund_id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    clear_all_authorization_caches(state);
    Ok(())
}

pub(crate) async fn load_refund_request_detail_any_for_update(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    refund_id: u64,
) -> Result<Option<RefundRequestRow>, sqlx::Error> {
    sqlx::query_as::<_, RefundRequestRow>(
        "SELECT rr.id, rr.order_id, rr.trade_no, rr.user_id, rr.plan_id, rr.assigned_admin_user_id,
                rr.status, rr.reason, rr.gateway_amount, rr.gateway_trade_no, rr.epay_pid, rr.epay_url,
                rr.epay_key_encrypted, rr.used_kb, rr.allowance_kb, rr.refund_amount, rr.charged_amount,
                rr.balance_refunded_amount, rr.gateway_refunded_amount, rr.site_balance_fallback_amount,
                rr.gateway_refund_pending_amount, rr.gateway_refund_started_at, rr.voting_ends_at,
                rr.refunded_at, rr.resolved_at, rr.resolved_by_user_id, rr.decision, rr.created_at, rr.updated_at,
                p.name AS plan_name, p.scope AS plan_scope, u.email AS user_email
         FROM order_refund_requests rr
         LEFT JOIN v2_plan p ON p.id = rr.plan_id
         LEFT JOIN v2_user u ON u.id = rr.user_id
         WHERE rr.id = ?
         LIMIT 1
         FOR UPDATE"
    )
    .bind(refund_id)
    .fetch_optional(&mut **tx)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscription_revoking_refunds_invalidate_authorization_caches_after_commit() {
        let source = include_str!("refunds_support.rs");
        for (start, end) in [
            (
                "pub(crate) async fn approve_refund_request_as_admin",
                "pub(crate) async fn load_refund_create_order_by_trade_no",
            ),
            (
                "pub(crate) async fn auto_refund_approved_request",
                "pub(crate) async fn load_refund_request_detail_any_for_update",
            ),
        ] {
            let section = source
                .split_once(start)
                .and_then(|(_, tail)| tail.split_once(end).map(|(body, _)| body))
                .expect("refund execution handler must remain present");
            assert!(
                section.contains("UPDATE user_plan_subscriptions"),
                "{start} must revoke the refunded subscription"
            );
            let commit = section.rfind("tx.commit().await").expect("writes must commit");
            let invalidate = section
                .find("clear_all_authorization_caches(state);")
                .expect("subscription revocation must invalidate authorization caches");
            assert!(commit < invalidate, "{start} must invalidate only after commit");
        }
    }

    fn refund_request_fixture() -> RefundRequestRow {
        RefundRequestRow {
            id: 41,
            order_id: 501,
            trade_no: "private-trade-no".to_string(),
            user_id: 7,
            plan_id: 9,
            assigned_admin_user_id: Some(13),
            status: "voting".to_string(),
            reason: Some("service unavailable".to_string()),
            gateway_amount: 2_000,
            gateway_trade_no: Some("private-gateway-trade-no".to_string()),
            epay_pid: Some("private-pid".to_string()),
            epay_url: Some("https://payments.example.test".to_string()),
            epay_key_encrypted: Some("private-encrypted-key".to_string()),
            used_kb: Some(100),
            allowance_kb: Some(1_000),
            refund_amount: Some(1_800),
            charged_amount: Some(200),
            balance_refunded_amount: 0,
            gateway_refunded_amount: 0,
            site_balance_fallback_amount: 0,
            gateway_refund_pending_amount: 0,
            gateway_refund_started_at: None,
            voting_ends_at: None,
            refunded_at: None,
            resolved_at: None,
            resolved_by_user_id: Some(17),
            decision: None,
            created_at: None,
            updated_at: None,
            plan_name: Some("Node plan".to_string()),
            plan_scope: Some("node".to_string()),
            user_email: Some("private@example.test".to_string()),
        }
    }

    fn assert_key_absent_recursively(value: &Value, forbidden: &str) {
        match value {
            Value::Object(object) => {
                assert!(
                    !object.contains_key(forbidden),
                    "unexpected key: {forbidden}"
                );
                for child in object.values() {
                    assert_key_absent_recursively(child, forbidden);
                }
            }
            Value::Array(items) => {
                for child in items {
                    assert_key_absent_recursively(child, forbidden);
                }
            }
            _ => {}
        }
    }

    #[test]
    fn public_refund_vote_detail_redacts_identity_and_payment_fields() {
        let request = refund_request_fixture();
        let evidences = vec![RefundEvidenceRow {
            id: 3,
            refund_request_id: request.id,
            user_id: 7,
            role: "user".to_string(),
            content: "supporting evidence".to_string(),
            created_at: None,
            updated_at: None,
            user_email: Some("evidence-private@example.test".to_string()),
        }];
        let votes = vec![RefundVoteRow {
            id: 5,
            refund_request_id: request.id,
            user_id: 23,
            vote: "approve".to_string(),
            created_at: None,
            updated_at: None,
        }];

        let payload = serialize_public_refund_vote_detail(&request, &evidences, &votes, 23);
        for forbidden in [
            "order_id",
            "trade_no",
            "user_id",
            "assigned_admin_user_id",
            "gateway_amount",
            "gateway_trade_no",
            "epay_pid",
            "epay_url",
            "epay_key_encrypted",
            "resolved_by_user_id",
            "user",
            "votes",
        ] {
            assert_key_absent_recursively(&payload, forbidden);
        }
        assert_eq!(payload["my_vote"], Value::String("approve".to_string()));
        assert_eq!(payload["evidences"][0]["is_mine"], Value::Bool(false));
    }
}
