use crate::*;
use sqlx::{MySql, QueryBuilder};

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct BannableUserRow {
    pub(crate) id: i64,
    pub(crate) banned: i8,
    pub(crate) ban_reason: Option<String>,
}

pub(crate) async fn batch_ban_users(
    state: &AppState,
    users: &[BannableUserRow],
    reason: &str,
    admin_id: i64,
) -> Result<i64, Response<Body>> {
    let effective_reason = reason.trim().to_string();
    if effective_reason.is_empty() {
        return Err(json_status_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({"message":"封禁用户时必须填写封禁原因"}),
        ));
    }

    let candidates = users
        .iter()
        .filter(|user| user.banned == 0 || user.ban_reason.as_deref().unwrap_or_default().trim() != effective_reason)
        .map(|user| user.id)
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return Ok(0);
    }

    let now = Utc::now().timestamp();
    let mut tx = state.db.begin().await.map_err(internal_error)?;
    batch_update_ban_state(&mut tx, &candidates, &effective_reason, admin_id, now).await?;
    batch_delete_user_tokens(&mut tx, &candidates).await?;
    batch_insert_ban_records(&mut tx, &candidates, &effective_reason, admin_id, now).await?;
    tx.commit().await.map_err(internal_error)?;

    let admin_email = load_admin_email_for_ban_notice(state, admin_id)
        .await
        .map_err(internal_error)?
        .unwrap_or_else(|| "system".to_string());
    let mut lines = vec![
        "用户封禁通知".to_string(),
        format!("操作人: {}", admin_email),
        format!("封禁数量: {}", candidates.len()),
        format!("原因: {}", effective_reason),
        format!("用户ID: {}", candidates.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(", ")),
    ];
    if candidates.len() > 20 {
        lines.push("提示: 用户ID列表已截断为前 20 项展示".to_string());
    }
    let preview = candidates.iter().take(20).map(|id| id.to_string()).collect::<Vec<_>>().join(", ");
    lines[4] = format!("用户ID: {}", preview);
    let _ = send_telegram_text_to_super_admins(
        state,
        &lines.join("\n"),
        "telegram_notify_user_banned",
    )
    .await;

    Ok(candidates.len() as i64)
}

async fn load_admin_email_for_ban_notice(
    state: &AppState,
    admin_id: i64,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, String>("SELECT email FROM v2_user WHERE id = ? LIMIT 1")
        .bind(admin_id)
        .fetch_optional(&state.db)
        .await
}

async fn batch_update_ban_state(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_ids: &[i64],
    reason: &str,
    admin_id: i64,
    now: i64,
) -> Result<(), Response<Body>> {
    let mut builder = QueryBuilder::<MySql>::new("UPDATE v2_user SET banned = 1, ban_reason = ");
    builder
        .push_bind(reason)
        .push(", banned_at = ")
        .push_bind(now)
        .push(", banned_by_admin_id = ")
        .push_bind(admin_id)
        .push(", updated_at = ")
        .push_bind(now)
        .push(" WHERE id IN (");
    {
        let mut separated = builder.separated(", ");
        for user_id in user_ids {
            separated.push_bind(user_id);
        }
    }
    builder.push(")");
    builder
        .build()
        .execute(&mut **tx)
        .await
        .map_err(internal_error)?;
    Ok(())
}

async fn batch_delete_user_tokens(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_ids: &[i64],
) -> Result<(), Response<Body>> {
    let mut builder = QueryBuilder::<MySql>::new("DELETE FROM personal_access_tokens WHERE tokenable_id IN (");
    {
        let mut separated = builder.separated(", ");
        for user_id in user_ids {
            separated.push_bind(*user_id as u64);
        }
    }
    builder.push(")");
    builder
        .build()
        .execute(&mut **tx)
        .await
        .map_err(internal_error)?;
    Ok(())
}

async fn batch_insert_ban_records(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_ids: &[i64],
    reason: &str,
    admin_id: i64,
    now: i64,
) -> Result<(), Response<Body>> {
    let mut builder = QueryBuilder::<MySql>::new(
        "INSERT INTO user_ban_records (user_id, admin_id, action, reason, source, context, created_at, updated_at) ",
    );
    builder.push_values(user_ids, |mut row, user_id| {
        row.push_bind(*user_id)
            .push_bind(admin_id)
            .push_bind("ban")
            .push_bind(reason)
            .push_bind("manual")
            .push_bind(r#"{"source":"manual"}"#)
            .push_bind(now)
            .push_bind(now);
    });
    builder
        .build()
        .execute(&mut **tx)
        .await
        .map_err(internal_error)?;
    Ok(())
}
