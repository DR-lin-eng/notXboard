use crate::*;
use chrono::{Datelike, TimeZone, Timelike, Utc};
use sqlx::{MySql, QueryBuilder};
use std::collections::{HashMap, HashSet};

const RESET_TIME_UPDATE_BATCH_SIZE: usize = 200;

pub(crate) async fn initialize_missing_user_reset_times(
    state: &AppState,
    limit: i64,
) -> Result<u64, sqlx::Error> {
    let users = load_users_missing_reset_time(state, limit.max(1)).await?;
    if users.is_empty() {
        return Ok(0);
    }

    let mut seen_plan_ids = HashSet::new();
    let plan_ids = users
        .iter()
        .filter_map(|user| user.plan_id)
        .filter(|plan_id| seen_plan_ids.insert(*plan_id))
        .collect::<Vec<_>>();
    let plan_map = load_plan_reset_info_map(state, &plan_ids).await?;
    let now_ts = Utc::now().timestamp();
    let system_reset_method = get_setting_int(state, "reset_traffic_method", 1).await;

    let mut assignments = Vec::with_capacity(users.len());
    for user in users {
        let Some(plan_id) = user.plan_id else {
            continue;
        };
        let Some(plan) = plan_map.get(&plan_id) else {
            continue;
        };
        let reset_method = resolve_reset_method_value_cached(plan.reset_traffic_method, system_reset_method);
        let Some(next_reset_at) =
            calculate_next_reset_at_for_user_plan_with_method(&user, plan, now_ts, reset_method)
        else {
            continue;
        };
        assignments.push((user.id, next_reset_at));
    }

    batch_initialize_user_reset_times(state, &assignments, now_ts).await
}

async fn load_users_missing_reset_time(
    state: &AppState,
    limit: i64,
) -> Result<Vec<BearerUserRow>, sqlx::Error> {
    sqlx::query_as::<_, BearerUserRow>(
        "SELECT id, invite_user_id, email, transfer_enable, last_login_at, created_at, banned, ban_reason,
                remind_expire, remind_traffic, expired_at, balance, commission_balance, plan_id,
                discount, commission_rate, telegram_id, uuid, is_admin, is_super_admin,
                trust_level, is_silenced, linux_do_id, linux_do_username, linux_do_name, linux_do_avatar,
                api_key, concurrent_ip_limit, token, subscribe_path, subscribe_key, subscribe_salt,
                u, d, device_limit, speed_limit, next_reset_at
         FROM v2_user
         WHERE next_reset_at IS NULL
           AND plan_id IS NOT NULL
           AND banned = 0
           AND (expired_at > ? OR expired_at IS NULL)
         ORDER BY id ASC
         LIMIT ?",
    )
    .bind(Utc::now().timestamp())
    .bind(limit)
    .fetch_all(&state.db)
    .await
}

async fn batch_initialize_user_reset_times(
    state: &AppState,
    assignments: &[(i64, i64)],
    now_ts: i64,
) -> Result<u64, sqlx::Error> {
    let mut updated = 0_u64;
    for chunk in assignments.chunks(RESET_TIME_UPDATE_BATCH_SIZE) {
        let mut builder = QueryBuilder::<MySql>::new("UPDATE v2_user u JOIN (");
        for (index, (user_id, next_reset_at)) in chunk.iter().enumerate() {
            if index > 0 {
                builder.push(" UNION ALL ");
            }
            builder
                .push("SELECT ")
                .push_bind(user_id)
                .push(" AS id, ")
                .push_bind(next_reset_at)
                .push(" AS next_reset_at");
        }
        builder.push(
            ") vals ON vals.id = u.id
             SET u.next_reset_at = vals.next_reset_at,
                 u.updated_at = ",
        );
        builder
            .push_bind(now_ts)
            .push(" WHERE u.next_reset_at IS NULL");
        let result = builder.build().execute(&state.db).await?;
        updated += result.rows_affected();
    }
    Ok(updated)
}

pub(crate) async fn reset_due_user_traffic(
    state: &AppState,
    limit: i64,
) -> Result<u64, sqlx::Error> {
    let users = load_due_traffic_reset_users(state, limit.max(1)).await?;
    let mut reset_count = 0_u64;
    for user in users {
        if reset_single_user_traffic_internal(state, &user, "cron", false).await? {
            reset_count += 1;
        }
    }
    Ok(reset_count)
}

async fn load_due_traffic_reset_users(
    state: &AppState,
    limit: i64,
) -> Result<Vec<BearerUserRow>, sqlx::Error> {
    sqlx::query_as::<_, BearerUserRow>(
        "SELECT id, invite_user_id, email, transfer_enable, last_login_at, created_at, banned, ban_reason,
                remind_expire, remind_traffic, expired_at, balance, commission_balance, plan_id,
                discount, commission_rate, telegram_id, uuid, is_admin, is_super_admin,
                trust_level, is_silenced, linux_do_id, linux_do_username, linux_do_name, linux_do_avatar,
                api_key, concurrent_ip_limit, token, subscribe_path, subscribe_key, subscribe_salt,
                u, d, device_limit, speed_limit, next_reset_at
         FROM v2_user
         WHERE next_reset_at IS NOT NULL
           AND next_reset_at <= ?
           AND plan_id IS NOT NULL
           AND banned = 0
           AND (expired_at > ? OR expired_at IS NULL)
         ORDER BY id ASC
         LIMIT ?",
    )
    .bind(Utc::now().timestamp())
    .bind(Utc::now().timestamp())
    .bind(limit)
    .fetch_all(&state.db)
    .await
}

pub(crate) async fn load_plan_reset_info(
    state: &AppState,
    plan_id: i64,
) -> Result<Option<PlanRow>, sqlx::Error> {
    sqlx::query_as::<_, PlanRow>(
        "SELECT id, scope, owner_user_id, min_trust_level, free_quota_gb_by_trust_level, node_ids, group_id,
                transfer_enable, COALESCE(is_unlimited_traffic, 0) AS is_unlimited_traffic, name, speed_limit,
                `show`, visibility_scope, access_user_ids, share_token, sort, renew, content, prices,
                reset_traffic_method, capacity_limit, sell, device_limit, tags, created_at, updated_at,
                NULL AS owner_email, NULL AS owner_linux_do_username, NULL AS owner_linux_do_name
         FROM v2_plan
         WHERE id = ?
         LIMIT 1",
    )
    .bind(plan_id)
    .fetch_optional(&state.db)
    .await
}

async fn load_plan_reset_info_map(
    state: &AppState,
    plan_ids: &[i64],
) -> Result<HashMap<i64, PlanRow>, sqlx::Error> {
    if plan_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut builder = QueryBuilder::<MySql>::new(
        "SELECT id, scope, owner_user_id, min_trust_level, free_quota_gb_by_trust_level, node_ids, group_id,
                transfer_enable, COALESCE(is_unlimited_traffic, 0) AS is_unlimited_traffic, name, speed_limit,
                `show`, visibility_scope, access_user_ids, share_token, sort, renew, content, prices,
                reset_traffic_method, capacity_limit, sell, device_limit, tags, created_at, updated_at,
                NULL AS owner_email, NULL AS owner_linux_do_username, NULL AS owner_linux_do_name
         FROM v2_plan
         WHERE id IN (",
    );
    {
        let mut separated = builder.separated(", ");
        for plan_id in plan_ids {
            separated.push_bind(plan_id);
        }
    }
    builder.push(")");

    let rows = builder.build_query_as::<PlanRow>().fetch_all(&state.db).await?;
    Ok(rows.into_iter().map(|plan| (plan.id, plan)).collect())
}

pub(crate) async fn reset_single_user_traffic(
    state: &AppState,
    user: &BearerUserRow,
    trigger_source: &str,
) -> Result<bool, sqlx::Error> {
    reset_single_user_traffic_internal(state, user, trigger_source, true).await
}

pub(crate) async fn reset_single_user_traffic_internal(
    state: &AppState,
    user: &BearerUserRow,
    trigger_source: &str,
    ignore_due: bool,
) -> Result<bool, sqlx::Error> {
    let locked_user = load_bearer_user_row_by_id(state, user.id).await?;
    let Some(user) = locked_user.as_ref() else {
        return Ok(false);
    };
    if user.plan_id.is_none() || user.next_reset_at.is_none() || user.banned != 0 {
        return Ok(false);
    }
    let now = Utc::now().timestamp();
    if !ignore_due && user.next_reset_at.unwrap_or(i64::MAX) > now {
        return Ok(false);
    }

    let plan = load_plan_reset_info(state, user.plan_id.unwrap_or_default()).await?;
    let Some(plan) = plan else {
        return Ok(false);
    };

    let next_reset_at = calculate_next_reset_at_for_user_plan(state, user, &plan, now).await;
    let reset_type = resolve_traffic_reset_type(state, &plan).await;
    let old_upload = user.u.max(0);
    let old_download = user.d.max(0);
    let old_total = old_upload + old_download;

    let mut tx = state.db.begin().await?;
    sqlx::query(
        "UPDATE v2_user
         SET u = 0,
             d = 0,
             last_reset_at = ?,
             reset_count = COALESCE(reset_count, 0) + 1,
             next_reset_at = ?,
             updated_at = ?
         WHERE id = ?",
    )
    .bind(now)
    .bind(next_reset_at)
    .bind(now)
    .bind(user.id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO v2_traffic_reset_logs
            (user_id, reset_type, reset_time, old_upload, old_download, old_total, new_upload, new_download, new_total, trigger_source, metadata, created_at, updated_at)
         VALUES (?, ?, NOW(), ?, ?, ?, 0, 0, 0, ?, NULL, NOW(), NOW())",
    )
    .bind(user.id)
    .bind(reset_type)
    .bind(old_upload)
    .bind(old_download)
    .bind(old_total)
    .bind(trigger_source)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(true)
}

pub(crate) async fn calculate_next_reset_at_for_user_plan(
    state: &AppState,
    user: &BearerUserRow,
    plan: &PlanRow,
    now_ts: i64,
) -> Option<i64> {
    let reset_method = resolve_reset_method_value(state, plan.reset_traffic_method).await;
    calculate_next_reset_at_for_user_plan_with_method(user, plan, now_ts, reset_method)
}

fn calculate_next_reset_at_for_user_plan_with_method(
    user: &BearerUserRow,
    _plan: &PlanRow,
    now_ts: i64,
    reset_method: Option<i64>,
) -> Option<i64> {
    let expired_at = user.expired_at?;
    let reset_method = reset_method?;
    let tz = chrono::FixedOffset::east_opt(8 * 3600)?;
    let now = chrono::DateTime::from_timestamp(now_ts, 0)?.with_timezone(&tz);
    let expired = chrono::DateTime::from_timestamp(expired_at, 0)?.with_timezone(&tz);
    let (hour, minute, second) = (expired.hour(), expired.minute(), expired.second());

    match reset_method {
        0 => {
            let mut next = now + chrono::Months::new(1);
            next = next.with_day(1)?.with_hour(hour)?.with_minute(minute)?.with_second(second)?;
            Some(next.timestamp())
        }
        1 => {
            let reset_day = expired.day();
            let current = now
                .with_day(reset_day)
                .and_then(|dt| dt.with_hour(hour))
                .and_then(|dt| dt.with_minute(minute))
                .and_then(|dt| dt.with_second(second));
            if let Some(candidate) = current {
                if candidate.timestamp() > now.timestamp() {
                    return Some(candidate.timestamp());
                }
            }
            let next_base = now + chrono::Months::new(1);
            let last_day = last_day_of_month(next_base.year(), next_base.month());
            let target_day = reset_day.min(last_day);
            next_base
                .with_day(target_day)
                .and_then(|dt| dt.with_hour(hour))
                .and_then(|dt| dt.with_minute(minute))
                .and_then(|dt| dt.with_second(second))
                .map(|dt| dt.timestamp())
        }
        3 => {
            let next_year = now.year() + 1;
            tz.with_ymd_and_hms(next_year, 1, 1, hour, minute, second)
                .single()
                .map(|dt| dt.timestamp())
        }
        4 => {
            let reset_month = expired.month();
            let reset_day = expired.day();
            let current = tz
                .with_ymd_and_hms(
                    now.year(),
                    reset_month,
                    reset_day.min(last_day_of_month(now.year(), reset_month)),
                    hour,
                    minute,
                    second,
                )
                .single();
            if let Some(candidate) = current {
                if candidate.timestamp() > now.timestamp() {
                    return Some(candidate.timestamp());
                }
            }
            let next_year = now.year() + 1;
            tz.with_ymd_and_hms(
                next_year,
                reset_month,
                reset_day.min(last_day_of_month(next_year, reset_month)),
                hour,
                minute,
                second,
            )
            .single()
            .map(|dt| dt.timestamp())
        }
        _ => None,
    }
}

pub(crate) async fn resolve_traffic_reset_type(
    state: &AppState,
    plan: &PlanRow,
) -> &'static str {
    let reset_method = resolve_reset_method_value(state, plan.reset_traffic_method).await;
    traffic_reset_type_from_method(reset_method)
}

fn traffic_reset_type_from_method(reset_method: Option<i64>) -> &'static str {
    match reset_method {
        Some(0) => "first_day_month",
        Some(1) => "monthly",
        Some(3) => "first_day_year",
        Some(4) => "yearly",
        _ => "manual",
    }
}

pub(crate) async fn resolve_reset_method_value(
    state: &AppState,
    plan_reset_method: Option<i64>,
) -> Option<i64> {
    let system_method = get_setting_int(state, "reset_traffic_method", 1).await;
    resolve_reset_method_value_cached(plan_reset_method, system_method)
}

fn resolve_reset_method_value_cached(
    plan_reset_method: Option<i64>,
    system_reset_method: i64,
) -> Option<i64> {
    match plan_reset_method {
        Some(2) => None,
        Some(value) => Some(value),
        None if system_reset_method == 2 => None,
        None => Some(system_reset_method),
    }
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .and_then(|date| date.pred_opt())
        .map(|date| date.day())
        .unwrap_or(28)
}
