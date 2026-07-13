use crate::*;
use crate::oauth_v1::linux_do::{
    fetch_linux_do_user_info, refresh_linux_do_token, resolve_linux_do_oauth_config,
};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct LinuxDoUserSyncStats {
    pub(crate) ran: bool,
    pub(crate) total: u64,
    pub(crate) synced: u64,
    pub(crate) failed: u64,
}

#[derive(Clone, sqlx::FromRow)]
struct LinuxDoSyncUserRow {
    id: i64,
    banned: i8,
    trust_level: i64,
    device_limit: Option<i64>,
    linux_do_id: Option<String>,
    oauth_provider: Option<String>,
    oauth_access_token: Option<String>,
    oauth_refresh_token: Option<String>,
    oauth_expires_at: Option<chrono::DateTime<Utc>>,
    updated_at: Option<i64>,
}

#[derive(Clone, sqlx::FromRow)]
struct UserGroupLimitRowLite {
    device_limit: i64,
    speed_limit_down: i64,
}

pub(crate) async fn run_linux_do_user_sync(
    state: &AppState,
    now_ts: i64,
) -> Result<LinuxDoUserSyncStats, String> {
    let enabled = get_setting_bool(state, "oauth_linux_do_enable", true).await;
    if !enabled {
        return Ok(LinuxDoUserSyncStats::default());
    }

    let oauth = resolve_linux_do_oauth_config(state)
        .await
        .map_err(|response| extract_response_error_message(response))?;
    if oauth.client_id.trim().is_empty() || oauth.client_secret.trim().is_empty() {
        return Ok(LinuxDoUserSyncStats::default());
    }

    let interval_minutes = 60_i64;
    let lock_key = "scheduler:oauth-sync-linux-do-users:last-run-at";
    let already_ran = redis_get_string(state, lock_key)
        .await
        .map_err(|err| format!("load oauth sync marker failed: {err}"))?;
    if already_ran
        .as_deref()
        .and_then(|value| value.parse::<i64>().ok())
        .map(|last| (now_ts - last) < interval_minutes * 60)
        .unwrap_or(false)
    {
        return Ok(LinuxDoUserSyncStats::default());
    }

    let chunk_size = 200_i64;
    let users = load_linux_do_sync_users(state, chunk_size)
        .await
        .map_err(|err| format!("load linux do users failed: {err}"))?;

    let mut stats = LinuxDoUserSyncStats {
        ran: true,
        total: users.len() as u64,
        synced: 0,
        failed: 0,
    };

    for user in users {
        if !should_sync_linux_do_user(&user, now_ts) {
            continue;
        }
        match sync_linux_do_user(state, &oauth, &user, now_ts).await {
            Ok(true) => stats.synced += 1,
            Ok(false) => {}
            Err(_) => stats.failed += 1,
        }
    }

    redis_setex_string(state, lock_key, 86_400, &now_ts.to_string())
        .await
        .map_err(|err| format!("store oauth sync marker failed: {err}"))?;
    Ok(stats)
}

async fn load_linux_do_sync_users(
    state: &AppState,
    limit: i64,
) -> Result<Vec<LinuxDoSyncUserRow>, sqlx::Error> {
    sqlx::query_as::<_, LinuxDoSyncUserRow>(
        "SELECT id, banned, trust_level, device_limit, linux_do_id, oauth_provider, oauth_access_token, oauth_refresh_token, oauth_expires_at, updated_at
         FROM v2_user
         WHERE oauth_provider = 'linux_do'
           AND linux_do_id IS NOT NULL
           AND banned = 0
         ORDER BY id ASC
         LIMIT ?",
    )
    .bind(limit)
    .fetch_all(&state.db)
    .await
}

fn should_sync_linux_do_user(user: &LinuxDoSyncUserRow, now_ts: i64) -> bool {
    if user.banned != 0 {
        return false;
    }
    if user.oauth_provider.as_deref() != Some("linux_do") {
        return false;
    }
    if user.linux_do_id.as_deref().unwrap_or("").trim().is_empty() {
        return false;
    }
    if user
        .oauth_expires_at
        .map(|value| (value.timestamp() - now_ts) <= 3600)
        .unwrap_or(false)
    {
        return true;
    }
    user.updated_at
        .map(|updated_at| updated_at < (now_ts - 86_400))
        .unwrap_or(true)
}

async fn sync_linux_do_user(
    state: &AppState,
    oauth: &crate::oauth_v1::linux_do::LinuxDoOauthConfig,
    user: &LinuxDoSyncUserRow,
    now_ts: i64,
) -> Result<bool, String> {
    let mut access_token = user.oauth_access_token.clone().unwrap_or_default();
    if access_token.trim().is_empty() {
        return Err("missing oauth access token".to_string());
    }
    let refresh_token = user.oauth_refresh_token.clone().unwrap_or_default();

    if user
        .oauth_expires_at
        .map(|value| value.timestamp() <= now_ts)
        .unwrap_or(false)
    {
        if refresh_token.trim().is_empty() {
            return Err("missing oauth refresh token".to_string());
        }
        let token_data = refresh_linux_do_token(oauth, &refresh_token)
            .await
            .map_err(|response| extract_response_error_message(response))?;
        let persisted = persist_linux_do_tokens(state, user.id, &token_data)
            .await
            .map_err(|err| format!("persist oauth tokens failed: {err}"))?;
        if !persisted {
            return Ok(false);
        }
        access_token = token_data.access_token;
    }

    let user_info = fetch_linux_do_user_info(&access_token)
        .await
        .map_err(|response| extract_response_error_message(response))?;
    apply_linux_do_user_sync(state, user.id, user.trust_level, &user_info, &access_token, Some(refresh_token.as_str()), now_ts)
        .await
        .map_err(|err| format!("apply linux do user sync failed: {err}"))
}

async fn apply_linux_do_user_sync(
    state: &AppState,
    user_id: i64,
    old_trust_level: i64,
    user_info: &crate::oauth_v1::linux_do::LinuxDoUserInfo,
    access_token: &str,
    refresh_token: Option<&str>,
    now_ts: i64,
) -> Result<bool, sqlx::Error> {
    let oauth_expires_at = chrono::DateTime::<Utc>::from_timestamp(now_ts + 3600, 0)
        .map(|dt| dt.naive_utc());
    let mut tx = state.db.begin().await?;
    let current = crate::ban_support::lock_bannable_user_for_update(&mut tx, user_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    if current.banned != 0 {
        return Ok(false);
    }
    sqlx::query(
        "UPDATE v2_user
         SET linux_do_username = ?, linux_do_name = ?, linux_do_avatar = ?, trust_level = ?, is_silenced = ?, external_ids = ?,
             oauth_provider = 'linux_do', oauth_access_token = ?, oauth_refresh_token = ?, oauth_expires_at = ?,
             ban_reason = CASE WHEN ? <> 0 THEN ban_reason ELSE ? END,
             banned_at = CASE WHEN ? <> 0 THEN banned_at ELSE ? END,
             banned_by_admin_id = CASE WHEN ? <> 0 THEN banned_by_admin_id ELSE NULL END,
             banned = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(&user_info.username)
    .bind(&user_info.name)
    .bind(&user_info.avatar_template)
    .bind(user_info.trust_level)
    .bind(if user_info.silenced { 1 } else { 0 })
    .bind(serde_json::to_string(&user_info.external_ids).unwrap_or_else(|_| "[]".to_string()))
    .bind(access_token)
    .bind(refresh_token)
    .bind(oauth_expires_at)
    .bind(if user_info.active { 1 } else { 0 })
    .bind(crate::ban_support::LINUX_DO_INACTIVE_BAN_REASON)
    .bind(if user_info.active { 1 } else { 0 })
    .bind(now_ts)
    .bind(if user_info.active { 1 } else { 0 })
    .bind(if user_info.active { 0 } else { 1 })
    .bind(now_ts)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    if old_trust_level != user_info.trust_level {
        apply_linux_do_group_limit(&mut tx, user_id, user_info.trust_level).await?;
    }
    if !user_info.active {
        sqlx::query(
            "INSERT INTO user_ban_records
                (user_id, admin_id, action, reason, source, context, created_at, updated_at)
             VALUES (?, NULL, 'ban', ?, 'linux_do_oauth_sync', ?, ?, ?)",
        )
        .bind(user_id)
        .bind(crate::ban_support::LINUX_DO_INACTIVE_BAN_REASON)
        .bind(r#"{"source":"linux_do_sync","upstream_active":false}"#)
        .bind(now_ts)
        .bind(now_ts)
        .execute(&mut *tx)
        .await?;
        sqlx::query("DELETE FROM personal_access_tokens WHERE tokenable_id = ? AND tokenable_type = 'App\\\\Models\\\\User'")
            .bind(user_id as u64)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE server_nodes SET status = 'inactive', updated_at = NOW() WHERE user_id = ?")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE tcping_agents SET is_enabled = 0, updated_at = UNIX_TIMESTAMP() WHERE user_id = ?")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    clear_all_authorization_caches(state);
    Ok(true)
}

async fn apply_linux_do_group_limit(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    trust_level: i64,
) -> Result<(), sqlx::Error> {
    let group_limit = sqlx::query_as::<_, UserGroupLimitRowLite>(
        "SELECT device_limit, speed_limit_down
         FROM user_group_limits
         WHERE trust_level = ?
         LIMIT 1",
    )
    .bind(trust_level)
    .fetch_optional(&mut **tx)
    .await?;

    if let Some(group_limit) = group_limit {
        sqlx::query(
            "UPDATE v2_user
             SET device_limit = CASE WHEN ? > 0 THEN ? ELSE device_limit END,
                 speed_limit = CASE WHEN ? > 0 THEN ? ELSE speed_limit END
             WHERE id = ?",
        )
        .bind(group_limit.device_limit)
        .bind(group_limit.device_limit)
        .bind(group_limit.speed_limit_down)
        .bind(group_limit.speed_limit_down)
        .bind(user_id)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}

async fn persist_linux_do_tokens(
    state: &AppState,
    user_id: i64,
    token_data: &crate::oauth_v1::linux_do::LinuxDoTokenResponse,
) -> Result<bool, sqlx::Error> {
    let now = Utc::now().timestamp();
    let oauth_expires_at = chrono::DateTime::<Utc>::from_timestamp(now + token_data.expires_in.max(60), 0)
        .map(|dt| dt.naive_utc());
    let result = sqlx::query(
        "UPDATE v2_user
         SET oauth_access_token = ?, oauth_refresh_token = ?, oauth_expires_at = ?, updated_at = ?
         WHERE id = ? AND banned = 0",
    )
    .bind(&token_data.access_token)
    .bind(token_data.refresh_token.clone())
    .bind(oauth_expires_at)
    .bind(now)
    .bind(user_id)
    .execute(&state.db)
    .await?;
    Ok(result.rows_affected() == 1)
}

fn extract_response_error_message(response: Response<Body>) -> String {
    format!("http {}", response.status().as_u16())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sync_user(banned: i8) -> LinuxDoSyncUserRow {
        LinuxDoSyncUserRow {
            id: 1,
            banned,
            trust_level: 2,
            device_limit: None,
            linux_do_id: Some("42".to_string()),
            oauth_provider: Some("linux_do".to_string()),
            oauth_access_token: Some("access".to_string()),
            oauth_refresh_token: Some("refresh".to_string()),
            oauth_expires_at: None,
            updated_at: None,
        }
    }

    #[test]
    fn scheduler_never_selects_a_banned_oauth_user_for_sync() {
        assert!(should_sync_linux_do_user(&sync_user(0), 1_000_000));
        assert!(!should_sync_linux_do_user(&sync_user(1), 1_000_000));
    }

    #[test]
    fn scheduled_writes_lock_ban_state_and_never_refresh_banned_rows() {
        let source = include_str!("oauth_sync_support.rs");
        let load = source
            .split_once("async fn load_linux_do_sync_users")
            .and_then(|(_, tail)| tail.split_once("fn should_sync_linux_do_user").map(|(body, _)| body))
            .expect("scheduled OAuth load must remain present");
        assert!(load.contains("AND banned = 0"));

        let apply = source
            .split_once("async fn apply_linux_do_user_sync")
            .and_then(|(_, tail)| tail.split_once("async fn apply_linux_do_group_limit").map(|(body, _)| body))
            .expect("scheduled OAuth mutation must remain present");
        let lock = apply
            .find("lock_bannable_user_for_update")
            .expect("scheduled mutation must lock current ban state");
        let rejected = apply
            .find("current.banned != 0")
            .expect("scheduled mutation must reject banned users");
        let update = apply
            .find("UPDATE v2_user")
            .expect("scheduled mutation must update the user");
        assert!(lock < rejected && rejected < update);

        let persist = source
            .split_once("async fn persist_linux_do_tokens")
            .and_then(|(_, tail)| tail.split_once("fn extract_response_error_message").map(|(body, _)| body))
            .expect("scheduled token persistence must remain present");
        assert!(persist.contains("WHERE id = ? AND banned = 0"));
    }
}
