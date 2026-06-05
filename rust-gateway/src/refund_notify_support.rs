use crate::*;
use tracing::{info, warn};

pub(crate) async fn notify_refund_vote_started(
    state: &AppState,
    refund_id: u64,
) -> Result<i64, String> {
    if !get_setting_bool(state, "telegram_notify_refund_vote", true).await {
        return Ok(0);
    }

    let req = load_refund_request_detail_any(state, refund_id)
        .await
        .map_err(|err| format!("load refund request failed: {err}"))?;
    let Some(req) = req else {
        return Ok(0);
    };

    let recipients = load_refund_recipient_chat_ids(state, &req, true).await?;
    info!(
        refund_id = req.id,
        status = req.status.as_str(),
        recipient_count = recipients.len(),
        "refund vote started telegram notify resolved recipients"
    );
    if recipients.is_empty() {
        return Ok(0);
    }

    let message = format!(
        "退款争议投票已开启\n申请单：#{}\n用户：{}\n套餐：{}\n截止时间：{}",
        req.id,
        req.user_email.clone().unwrap_or_else(|| "-".to_string()),
        req.plan_name.clone().unwrap_or_else(|| "-".to_string()),
        format_optional_naive_datetime(req.voting_ends_at).unwrap_or_else(|| "-".to_string())
    );

    send_telegram_text_to_chat_ids(
        state,
        &recipients,
        &message,
        "telegram_notify_refund_vote",
    )
    .await
    .map_err(|err| {
        warn!(refund_id = req.id, error = %err, "refund vote started telegram notify failed");
        err
    })
}

pub(crate) async fn notify_refund_vote_cast(
    state: &AppState,
    refund_id: u64,
    voter_user_id: u64,
    vote: &str,
) -> Result<i64, String> {
    if !get_setting_bool(state, "telegram_notify_refund_vote", true).await {
        return Ok(0);
    }

    let req = load_refund_request_detail_any(state, refund_id)
        .await
        .map_err(|err| format!("load refund request failed: {err}"))?;
    let Some(req) = req else {
        return Ok(0);
    };

    let recipients = load_refund_admin_chat_ids(state, req.assigned_admin_user_id).await?;
    info!(
        refund_id = req.id,
        vote,
        recipient_count = recipients.len(),
        "refund vote cast telegram notify resolved admin recipients"
    );
    if recipients.is_empty() {
        return Ok(0);
    }

    let voter_email = sqlx::query_scalar::<_, String>("SELECT email FROM v2_user WHERE id = ? LIMIT 1")
        .bind(voter_user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| format!("load voter email failed: {err}"))?
        .unwrap_or_else(|| "-".to_string());

    let vote_text = if vote == "approve" { "支持退款" } else { "反对退款" };
    let message = format!(
        "退款争议有新投票\n申请单：#{}\n用户：{}\n投票人：{}\n结果：{}",
        req.id,
        req.user_email.clone().unwrap_or_else(|| "-".to_string()),
        voter_email,
        vote_text
    );

    send_telegram_text_to_chat_ids(
        state,
        &recipients,
        &message,
        "telegram_notify_refund_vote",
    )
    .await
    .map_err(|err| {
        warn!(refund_id = req.id, vote, error = %err, "refund vote cast telegram notify failed");
        err
    })
}

pub(crate) async fn notify_refund_status_changed(
    state: &AppState,
    refund_id: u64,
) -> Result<i64, String> {
    if !get_setting_bool(state, "telegram_notify_refund_status", true).await {
        return Ok(0);
    }

    let req = load_refund_request_detail_any(state, refund_id)
        .await
        .map_err(|err| format!("load refund request failed: {err}"))?;
    let Some(req) = req else {
        return Ok(0);
    };

    let recipients = load_refund_recipient_chat_ids(state, &req, true).await?;
    info!(
        refund_id = req.id,
        status = req.status.as_str(),
        recipient_count = recipients.len(),
        "refund status changed telegram notify resolved recipients"
    );
    if recipients.is_empty() {
        return Ok(0);
    }

    let mut message = format!(
        "退款状态更新\n申请单：#{}\n用户：{}\n套餐：{}\n状态：{}",
        req.id,
        req.user_email.clone().unwrap_or_else(|| "-".to_string()),
        req.plan_name.clone().unwrap_or_else(|| "-".to_string()),
        humanize_refund_status_local(&req.status)
    );
    if let Some(reason) = req.reason.as_deref().map(str::trim).filter(|value| !value.is_empty()) {
        message.push_str(&format!("\n说明：{}", limit_text_local(reason, 180)));
    }

    send_telegram_text_to_chat_ids(
        state,
        &recipients,
        &message,
        "telegram_notify_refund_status",
    )
    .await
    .map_err(|err| {
        warn!(refund_id = req.id, status = req.status.as_str(), error = %err, "refund status changed telegram notify failed");
        err
    })
}

async fn load_refund_recipient_chat_ids(
    state: &AppState,
    req: &RefundRequestRow,
    include_user: bool,
) -> Result<Vec<i64>, String> {
    let mut chat_ids = load_refund_admin_chat_ids(state, req.assigned_admin_user_id).await?;
    if include_user {
        let user_chat_id = sqlx::query_scalar::<_, i64>(
            "SELECT telegram_id FROM v2_user WHERE id = ? AND telegram_id IS NOT NULL LIMIT 1",
        )
        .bind(req.user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| format!("load refund user telegram id failed: {err}"))?;
        if let Some(chat_id) = user_chat_id {
            chat_ids.push(chat_id);
        }
    }
    chat_ids.sort_unstable();
    chat_ids.dedup();
    Ok(chat_ids)
}

async fn load_refund_admin_chat_ids(
    state: &AppState,
    assigned_admin_user_id: Option<u64>,
) -> Result<Vec<i64>, String> {
    if let Some(admin_id) = assigned_admin_user_id {
        let chat_id = sqlx::query_scalar::<_, i64>(
            "SELECT telegram_id FROM v2_user WHERE id = ? AND telegram_id IS NOT NULL LIMIT 1",
        )
        .bind(admin_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| format!("load assigned refund admin telegram id failed: {err}"))?;
        if let Some(chat_id) = chat_id {
            return Ok(vec![chat_id]);
        }
    }

    let rows = sqlx::query("SELECT telegram_id FROM v2_user WHERE telegram_id IS NOT NULL AND (is_admin = 1 OR is_super_admin = 1 OR is_staff = 1)")
        .fetch_all(&state.db)
        .await
        .map_err(|err| format!("load refund fallback admins failed: {err}"))?;
    Ok(rows
        .into_iter()
        .filter_map(|row| row.try_get::<Option<i64>, _>("telegram_id").ok().flatten())
        .collect())
}

fn humanize_refund_status_local(status: &str) -> &'static str {
    match status {
        "pending" => "待处理",
        "voting" => "投票中",
        "approved" => "已批准",
        "denied" => "已拒绝",
        "processing" => "处理中",
        "refunded" => "已退款",
        "failed" => "失败",
        _ => "未知状态",
    }
}

fn limit_text_local(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }
    format!("{}...", input.chars().take(max_chars).collect::<String>())
}
