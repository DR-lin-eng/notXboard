//! Monthly free quota a node owner grants per trust level
//! (`server_nodes.access_control.free_quota_gb_by_trust_level`).
//!
//! Traffic a user spends on someone else's node without a usable node-plan
//! subscription is counted per calendar month (UTC+8). Once it reaches the
//! quota of the user's trust level, the trust-level and individual-grant
//! access paths stop resolving for that user until the next month.

use crate::*;
use std::sync::atomic::{AtomicBool, Ordering};

const BYTES_PER_GB: f64 = 1024.0 * 1024.0 * 1024.0;
const MAX_TRUST_LEVEL: i64 = 4;
const BATCH_ROWS: usize = 500;
const UNLIMITED_TRAFFIC_ALLOWANCE_KB: i64 = 8_000_000_000_000_000;

/// Quota in bytes for trust levels 0..=4. Zero means the level is not limited.
pub(crate) type FreeQuotaLimits = [u64; 5];

static SCHEMA_READY: AtomicBool = AtomicBool::new(false);

pub(crate) fn free_quota_schema_ready() -> bool {
    SCHEMA_READY.load(Ordering::Relaxed)
}

async fn ensure_node_free_quota_schema(state: &AppState) -> Result<bool, sqlx::Error> {
    let server_nodes_ready: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.tables
         WHERE table_schema = DATABASE() AND table_name = 'server_nodes'",
    )
    .fetch_one(&state.db)
    .await?;
    if server_nodes_ready == 0 {
        return Ok(false);
    }
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS `node_free_quota_usage` (
          `node_id` bigint unsigned NOT NULL,
          `period_month` int unsigned NOT NULL,
          `user_id` int NOT NULL,
          `used_bytes` bigint unsigned NOT NULL DEFAULT '0',
          `updated_at` timestamp NULL DEFAULT NULL,
          PRIMARY KEY (`node_id`,`period_month`,`user_id`),
          KEY `node_free_quota_usage_user_period_index` (`user_id`,`period_month`),
          CONSTRAINT `node_free_quota_usage_node_id_foreign` FOREIGN KEY (`node_id`) REFERENCES `server_nodes` (`id`) ON DELETE CASCADE
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
    )
    .execute(&state.db)
    .await?;
    Ok(true)
}

/// Creates the usage table before the gateway serves requests, so quota checks are in force
/// from the first request; keeps retrying in the background if the database is not ready yet.
pub(crate) async fn init_node_free_quota_schema(state: AppState) {
    match ensure_node_free_quota_schema(&state).await {
        Ok(true) => {
            SCHEMA_READY.store(true, Ordering::Relaxed);
            return;
        }
        Ok(false) => {}
        Err(err) => error!("node free quota schema ensure failed: {err}"),
    }
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            match ensure_node_free_quota_schema(&state).await {
                Ok(true) => {
                    SCHEMA_READY.store(true, Ordering::Relaxed);
                    break;
                }
                Ok(false) => {}
                Err(err) => error!("node free quota schema ensure failed: {err}"),
            }
        }
    });
}

/// `YYYYMM` of the calendar month `now_ts` falls in, in UTC+8.
pub(crate) fn free_quota_period(now_ts: i64) -> u32 {
    use chrono::Datelike;
    let shifted =
        chrono::DateTime::from_timestamp(now_ts.saturating_add(8 * 3600), 0).unwrap_or_default();
    shifted.year().max(0) as u32 * 100 + shifted.month()
}

fn quota_gb(value: &Value) -> f64 {
    let gb = match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.trim().parse::<f64>().ok(),
        _ => None,
    }
    .unwrap_or(0.0);
    if gb.is_finite() && gb > 0.0 {
        gb
    } else {
        0.0
    }
}

fn quota_levels(value: Option<&Value>) -> [f64; 5] {
    let mut levels = [0.0; 5];
    if let Some(Value::Object(map)) = value {
        for (level, raw) in map {
            if let Ok(level) = level.trim().parse::<usize>() {
                if let Some(slot) = levels.get_mut(level) {
                    *slot = quota_gb(raw);
                }
            }
        }
    }
    levels
}

/// The object stored on the node: one non-negative GB number per trust level.
pub(crate) fn normalize_free_quota(value: Option<&Value>) -> Value {
    let mut result = Map::new();
    for (level, gb) in quota_levels(value).into_iter().enumerate() {
        result.insert(level.to_string(), Value::from(gb));
    }
    Value::Object(result)
}

/// Per-level byte limits of a node, or `None` when no level is limited.
pub(crate) fn free_quota_limits(access_control: Option<&Value>) -> Option<FreeQuotaLimits> {
    let levels =
        quota_levels(access_control.and_then(|value| value.get("free_quota_gb_by_trust_level")));
    let mut limits = [0u64; 5];
    for (slot, gb) in limits.iter_mut().zip(levels) {
        if gb > 0.0 {
            // A positive quota never rounds down to "unlimited".
            *slot = ((gb * BYTES_PER_GB).round().min(i64::MAX as f64) as u64).max(1);
        }
    }
    limits.iter().any(|limit| *limit > 0).then_some(limits)
}

fn trust_level_index(trust_level: i64) -> usize {
    trust_level.clamp(0, MAX_TRUST_LEVEL) as usize
}

pub(crate) fn free_quota_exhausted(
    limits: &FreeQuotaLimits,
    trust_level: i64,
    used_bytes: u64,
) -> bool {
    let limit = limits[trust_level_index(trust_level)];
    limit > 0 && used_bytes >= limit
}

/// SQL predicate that is true while `user_alias` (a `v2_user` row) still has free quota
/// left on the node this month. Every interpolated value is an integer.
pub(crate) fn free_quota_remaining_sql(
    node_id: u64,
    period: u32,
    limits: &FreeQuotaLimits,
    user_alias: &str,
) -> String {
    let never = i64::MAX as u64;
    let limit = |level: usize| {
        if limits[level] > 0 {
            limits[level]
        } else {
            never
        }
    };
    format!(
        "NOT EXISTS (
           SELECT 1 FROM node_free_quota_usage fq
           WHERE fq.node_id = {node_id}
             AND fq.period_month = {period}
             AND fq.user_id = {user_alias}.id
             AND fq.used_bytes >= CASE LEAST(GREATEST(COALESCE({user_alias}.trust_level, 0), 0), {MAX_TRUST_LEVEL})
               WHEN 0 THEN {} WHEN 1 THEN {} WHEN 2 THEN {} WHEN 3 THEN {} ELSE {} END
         )",
        limit(0),
        limit(1),
        limit(2),
        limit(3),
        limit(4),
    )
}

/// Nodes of other owners on which the user has used up this month's free quota.
pub(crate) async fn load_free_quota_exhausted_node_ids(
    state: &AppState,
    user_id: i64,
    trust_level: i64,
    now_ts: i64,
) -> Result<Vec<u64>, sqlx::Error> {
    if !free_quota_schema_ready() {
        return Ok(Vec::new());
    }
    let rows = sqlx::query_as::<_, (u64, u64, Option<SqlxJson<Value>>)>(
        "SELECT fq.node_id, fq.used_bytes, sn.access_control
         FROM node_free_quota_usage fq
         JOIN server_nodes sn ON sn.id = fq.node_id
         WHERE fq.user_id = ?
           AND fq.period_month = ?
           AND sn.user_id <> ?
           AND JSON_EXTRACT(sn.access_control, '$.free_quota_gb_by_trust_level') IS NOT NULL",
    )
    .bind(user_id)
    .bind(free_quota_period(now_ts))
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;
    Ok(rows
        .into_iter()
        .filter(|(_, used_bytes, access_control)| {
            free_quota_limits(access_control.as_ref().map(|json| &json.0))
                .is_some_and(|limits| free_quota_exhausted(&limits, trust_level, *used_bytes))
        })
        .map(|(node_id, _, _)| node_id)
        .collect())
}

/// SQL predicate on `sn.id` that excludes the given nodes; always true for an empty list.
pub(crate) fn free_quota_node_filter_sql(exhausted_node_ids: &[u64]) -> String {
    if exhausted_node_ids.is_empty() {
        return "1 = 1".to_string();
    }
    let ids = exhausted_node_ids
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    format!("sn.id NOT IN ({ids})")
}

/// Adds this batch's traffic to the monthly free usage of every user who is neither the node
/// owner nor covered by a node-plan subscription that still has allowance.
/// Returns true when the batch took at least one user over the node's quota.
pub(crate) async fn record_free_quota_usage(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    node_id: u64,
    now_ts: i64,
    rows: &[(i64, i64)],
) -> Result<bool, sqlx::Error> {
    if rows.is_empty() || !free_quota_schema_ready() {
        return Ok(false);
    }
    let Some((owner_id, access_control)) = sqlx::query_as::<_, (i64, Option<SqlxJson<Value>>)>(
        "SELECT CAST(user_id AS SIGNED), access_control FROM server_nodes WHERE id = ?",
    )
    .bind(node_id)
    .fetch_optional(&mut **tx)
    .await?
    else {
        return Ok(false);
    };
    let limits = free_quota_limits(access_control.as_ref().map(|json| &json.0));
    let user_ids = rows.iter().map(|(user_id, _)| *user_id).collect::<Vec<_>>();
    let covered = load_subscription_covered_user_ids(tx, node_id, now_ts, &user_ids).await?;
    let free_rows = rows
        .iter()
        .filter(|(user_id, bytes)| *user_id != owner_id && *bytes > 0 && !covered.contains(user_id))
        .collect::<Vec<_>>();
    let period = free_quota_period(now_ts);
    let mut crossed = false;
    for chunk in free_rows.chunks(BATCH_ROWS) {
        let sql = format!(
            "INSERT INTO node_free_quota_usage (node_id, period_month, user_id, used_bytes, updated_at)
             VALUES {}
             ON DUPLICATE KEY UPDATE
                used_bytes = used_bytes + VALUES(used_bytes),
                updated_at = VALUES(updated_at)",
            vec!["(?, ?, ?, ?, NOW())"; chunk.len()].join(","),
        );
        let mut query = sqlx::query(&sql);
        for (user_id, bytes) in chunk {
            query = query.bind(node_id).bind(period).bind(*user_id).bind(*bytes);
        }
        query.execute(&mut **tx).await?;

        let Some(limits) = limits.as_ref().filter(|_| !crossed) else {
            continue;
        };
        let sql = format!(
            "SELECT fq.user_id, fq.used_bytes, CAST(COALESCE(u.trust_level, 0) AS SIGNED)
             FROM node_free_quota_usage fq
             JOIN v2_user u ON u.id = fq.user_id
             WHERE fq.node_id = ? AND fq.period_month = ? AND fq.user_id IN ({})",
            vec!["?"; chunk.len()].join(","),
        );
        let mut query = sqlx::query_as::<_, (i64, u64, i64)>(&sql)
            .bind(node_id)
            .bind(period);
        for (user_id, _) in chunk {
            query = query.bind(*user_id);
        }
        let added = chunk
            .iter()
            .map(|(user_id, bytes)| (*user_id, *bytes as u64))
            .collect::<HashMap<_, _>>();
        crossed =
            query
                .fetch_all(&mut **tx)
                .await?
                .into_iter()
                .any(|(user_id, used, trust_level)| {
                    let before = used.saturating_sub(added.get(&user_id).copied().unwrap_or(0));
                    free_quota_exhausted(limits, trust_level, used)
                        && !free_quota_exhausted(limits, trust_level, before)
                });
    }
    Ok(crossed)
}

/// Users whose access to the node is paid for: same subscription test as node authorization.
async fn load_subscription_covered_user_ids(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    node_id: u64,
    now_ts: i64,
    user_ids: &[i64],
) -> Result<HashSet<i64>, sqlx::Error> {
    let mut covered = HashSet::new();
    for chunk in user_ids.chunks(BATCH_ROWS) {
        let sql = format!(
            "SELECT DISTINCT ups.user_id
             FROM user_plan_subscriptions ups
             JOIN v2_plan plan ON plan.id = ups.plan_id AND plan.scope = 'node'
             JOIN server_nodes node ON node.id = ? AND node.user_id = plan.owner_user_id
             WHERE JSON_CONTAINS(COALESCE(plan.node_ids, JSON_ARRAY()), CAST(node.id AS JSON), '$')
               AND ups.status = 1
               AND (ups.expired_at IS NULL OR ups.expired_at > ?)
               AND (
                 ups.traffic_allowance_kb >= ?
                 OR ups.traffic_allowance_kb > ups.used_traffic_kb
               )
               AND ups.user_id IN ({})",
            vec!["?"; chunk.len()].join(","),
        );
        let mut query = sqlx::query_scalar::<_, i64>(&sql)
            .bind(node_id)
            .bind(now_ts)
            .bind(UNLIMITED_TRAFFIC_ALLOWANCE_KB);
        for user_id in chunk {
            query = query.bind(*user_id);
        }
        covered.extend(query.fetch_all(&mut **tx).await?);
    }
    Ok(covered)
}

/// Drops usage of months before the previous one.
pub(crate) async fn delete_stale_free_quota_usage(
    state: &AppState,
    now_ts: i64,
) -> Result<u64, sqlx::Error> {
    if !free_quota_schema_ready() {
        return Ok(0);
    }
    let period = free_quota_period(now_ts);
    let (year, month) = (period / 100, period % 100);
    let previous = if month == 1 {
        (year - 1) * 100 + 12
    } else {
        period - 1
    };
    let result = sqlx::query("DELETE FROM node_free_quota_usage WHERE period_month < ?")
        .bind(previous)
        .execute(&state.db)
        .await?;
    Ok(result.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;

    const GB: u64 = 1024 * 1024 * 1024;

    #[test]
    fn period_rolls_over_at_utc8_midnight() {
        // 2026-11-30 15:59:59 UTC is 23:59:59 on the 30th in UTC+8.
        assert_eq!(free_quota_period(1_796_054_399), 202611);
        // One second later it is December 1st in UTC+8.
        assert_eq!(free_quota_period(1_796_054_400), 202612);
        // 2026-12-31 16:00:00 UTC is already January in UTC+8.
        assert_eq!(free_quota_period(1_798_732_800), 202701);
    }

    #[test]
    fn limits_read_numbers_and_numeric_strings() {
        let access = json!({"free_quota_gb_by_trust_level": {"0": 0, "1": 1, "2": "2.5", "3": -4, "4": "x", "9": 7}});
        let limits = free_quota_limits(Some(&access)).expect("limits");
        assert_eq!(limits, [0, GB, GB * 5 / 2, 0, 0]);
    }

    #[test]
    fn no_positive_quota_means_no_limits() {
        assert!(free_quota_limits(None).is_none());
        assert!(free_quota_limits(Some(&json!({"min_trust_level": 1}))).is_none());
        let zeros =
            json!({"free_quota_gb_by_trust_level": {"0": 0, "1": 0, "2": 0, "3": 0, "4": 0}});
        assert!(free_quota_limits(Some(&zeros)).is_none());
        assert!(
            free_quota_limits(Some(&json!({"free_quota_gb_by_trust_level": [1, 2]}))).is_none()
        );
    }

    #[test]
    fn exhausted_at_the_limit_and_only_for_limited_levels() {
        let limits = [0, GB, 0, 0, 10 * GB];
        assert!(!free_quota_exhausted(&limits, 1, GB - 1));
        assert!(free_quota_exhausted(&limits, 1, GB));
        assert!(!free_quota_exhausted(&limits, 0, u64::MAX));
        assert!(!free_quota_exhausted(&limits, -3, u64::MAX));
        assert!(!free_quota_exhausted(&limits, 2, u64::MAX));
        // Levels above the scale use the top level's quota.
        assert!(free_quota_exhausted(&limits, 9, 10 * GB));
    }

    #[test]
    fn normalized_quota_has_every_level() {
        let normalized = normalize_free_quota(Some(&json!({"1": "3", "2": -1, "7": 5})));
        assert_eq!(
            normalized,
            json!({"0": 0.0, "1": 3.0, "2": 0.0, "3": 0.0, "4": 0.0})
        );
    }

    #[test]
    fn remaining_sql_keeps_unlimited_levels_unreachable() {
        let sql = free_quota_remaining_sql(7, 202611, &[0, GB, 0, 0, 0], "fu");
        assert!(sql.contains("fq.node_id = 7"));
        assert!(sql.contains("fq.period_month = 202611"));
        assert!(sql.contains(&format!("WHEN 0 THEN {} WHEN 1 THEN {GB}", i64::MAX)));
    }

    #[test]
    fn node_filter_sql_lists_ids() {
        assert_eq!(free_quota_node_filter_sql(&[]), "1 = 1");
        assert_eq!(free_quota_node_filter_sql(&[3, 12]), "sn.id NOT IN (3,12)");
    }
}
