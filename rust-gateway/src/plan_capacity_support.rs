use crate::*;
use sqlx::QueryBuilder;

pub(crate) async fn load_user_active_subscription_plan_ids(
    state: &AppState,
    user_id: i64,
    now: i64,
) -> Result<HashSet<i64>, sqlx::Error> {
    let plan_ids = sqlx::query_scalar::<_, i64>(
        "SELECT DISTINCT plan_id
         FROM user_plan_subscriptions
         WHERE user_id = ?
           AND status = 1
           AND (expired_at IS NULL OR expired_at > ?)"
    )
    .bind(user_id)
    .bind(now)
    .fetch_all(&state.db)
    .await?;
    Ok(plan_ids.into_iter().collect())
}

pub(crate) async fn load_plan_active_subscription_counts(
    state: &AppState,
    plan_ids: &[i64],
    now: i64,
) -> Result<HashMap<i64, i64>, sqlx::Error> {
    if plan_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut builder = QueryBuilder::<sqlx::MySql>::new(
        "SELECT plan_id, CAST(COUNT(DISTINCT user_id) AS SIGNED) AS active_count
         FROM user_plan_subscriptions
         WHERE status = 1
           AND (expired_at IS NULL OR expired_at > ",
    );
    builder.push_bind(now);
    builder.push(") AND plan_id IN (");
    {
        let mut separated = builder.separated(", ");
        for plan_id in plan_ids {
            separated.push_bind(plan_id);
        }
    }
    builder.push(") GROUP BY plan_id");

    let rows = builder.build().fetch_all(&state.db).await?;
    let mut counts = HashMap::with_capacity(rows.len());
    for row in rows {
        let plan_id = row.try_get::<i64, _>("plan_id").unwrap_or_default();
        let active_count = row.try_get::<i64, _>("active_count").unwrap_or_default();
        counts.insert(plan_id, active_count.max(0));
    }
    Ok(counts)
}

pub(crate) fn plan_has_remaining_capacity(
    capacity_limit: Option<u64>,
    active_count: i64,
) -> bool {
    match capacity_limit {
        None => true,
        Some(limit) if limit == 0 => true,
        Some(limit) => (limit as i64 - active_count.max(0)) > 0,
    }
}
