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
        "SELECT id, trust_level, device_limit, linux_do_id, oauth_provider, oauth_access_token, oauth_refresh_token, oauth_expires_at, updated_at
         FROM v2_user
         WHERE oauth_provider = 'linux_do'
           AND linux_do_id IS NOT NULL
         ORDER BY id ASC
         LIMIT ?",
    )
    .bind(limit)
    .fetch_all(&state.db)
    .await
}

fn should_sync_linux_do_user(user: &LinuxDoSyncUserRow, now_ts: i64) -> bool {
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
        persist_linux_do_tokens(state, user.id, &token_data)
            .await
            .map_err(|err| format!("persist oauth tokens failed: {err}"))?;
        access_token = token_data.access_token;
    }

    let user_info = fetch_linux_do_user_info(&access_token)
        .await
        .map_err(|response| extract_response_error_message(response))?;
    apply_linux_do_user_sync(state, user.id, user.trust_level, &user_info, &access_token, Some(refresh_token.as_str()), now_ts)
        .await
        .map_err(|err| format!("apply linux do user sync failed: {err}"))?;
    Ok(true)
}

async fn apply_linux_do_user_sync(
    state: &AppState,
    user_id: i64,
    old_trust_level: i64,
    user_info: &crate::oauth_v1::linux_do::LinuxDoUserInfo,
    access_token: &str,
    refresh_token: Option<&str>,
    now_ts: i64,
) -> Result<(), sqlx::Error> {
    let oauth_expires_at = chrono::DateTime::<Utc>::from_timestamp(now_ts + 3600, 0)
        .map(|dt| dt.naive_utc());
    let mut tx = state.db.begin().await?;
    sqlx::query(
        "UPDATE v2_user
         SET linux_do_username = ?, linux_do_name = ?, linux_do_avatar = ?, trust_level = ?, is_silenced = ?, external_ids = ?,
             oauth_provider = 'linux_do', oauth_access_token = ?, oauth_refresh_token = ?, oauth_expires_at = ?, banned = ?, updated_at = ?
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
    .bind(if user_info.active { 0 } else { 1 })
    .bind(now_ts)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    if old_trust_level != user_info.trust_level {
        apply_linux_do_group_limit(&mut tx, user_id, user_info.trust_level).await?;
    }

    tx.commit().await?;
    Ok(())
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
) -> Result<(), sqlx::Error> {
    let now = Utc::now().timestamp();
    let oauth_expires_at = chrono::DateTime::<Utc>::from_timestamp(now + token_data.expires_in.max(60), 0)
        .map(|dt| dt.naive_utc());
    sqlx::query(
        "UPDATE v2_user
         SET oauth_access_token = ?, oauth_refresh_token = ?, oauth_expires_at = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(&token_data.access_token)
    .bind(token_data.refresh_token.clone())
    .bind(oauth_expires_at)
    .bind(now)
    .bind(user_id)
    .execute(&state.db)
    .await?;
    Ok(())
}

fn extract_response_error_message(response: Response<Body>) -> String {
    format!("http {}", response.status().as_u16())
}
