use crate::*;

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct SchedulableOrderRow {
    pub(crate) id: i64,
    pub(crate) user_id: i64,
    pub(crate) trade_no: String,
    pub(crate) status: i64,
    pub(crate) created_at: i64,
    pub(crate) balance_amount: Option<i64>,
}

pub(crate) async fn load_schedulable_orders(
    state: &AppState,
    limit: i64,
) -> Result<Vec<SchedulableOrderRow>, sqlx::Error> {
    sqlx::query_as::<_, SchedulableOrderRow>(
        "SELECT id, user_id, trade_no, status, created_at, balance_amount
         FROM v2_order
         WHERE status IN (0, 1)
         ORDER BY created_at ASC
         LIMIT ?"
    )
    .bind(limit.max(1))
    .fetch_all(&state.db)
    .await
}
