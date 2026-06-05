use crate::*;
use sqlx::QueryBuilder;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct PublicOverviewMetrics {
    pub(crate) active_users: i64,
    pub(crate) registered_users: i64,
    pub(crate) node_count: i64,
}

pub(crate) async fn load_public_overview_metrics(
    state: &AppState,
) -> Result<PublicOverviewMetrics, sqlx::Error> {
    let (active_users, registered_users, node_count) = tokio::try_join!(
        load_public_active_user_count(state),
        load_public_registered_user_count(state),
        load_public_node_count(state),
    )?;

    Ok(PublicOverviewMetrics {
        active_users,
        registered_users,
        node_count,
    })
}

pub(crate) async fn load_public_active_user_count(state: &AppState) -> Result<i64, sqlx::Error> {
    let now = Utc::now().timestamp() - 60;
    sqlx::query_scalar("SELECT COUNT(*) FROM v2_user WHERE t >= ?")
        .bind(now)
        .fetch_one(&state.db)
        .await
}

pub(crate) async fn load_public_registered_user_count(state: &AppState) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*) FROM v2_user")
        .fetch_one(&state.db)
        .await
}

pub(crate) async fn load_public_node_count(state: &AppState) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*) FROM v2_server")
        .fetch_one(&state.db)
        .await
}

pub(crate) async fn load_public_average_bandwidth(state: &AppState) -> Result<Value, sqlx::Error> {
    let row = sqlx::query(
        "SELECT CAST(COALESCE(SUM(u), 0) AS SIGNED) AS total_u,
                CAST(COALESCE(SUM(d), 0) AS SIGNED) AS total_d
         FROM v2_user"
    )
    .fetch_one(&state.db)
    .await?;
    let total_upload: i64 = row.try_get("total_u").unwrap_or(0);
    let total_download: i64 = row.try_get("total_d").unwrap_or(0);
    let now = Utc::now().timestamp();

    let mut snapshot = state.traffic_snapshot.write();
    let previous = *snapshot;
    *snapshot = Some(TrafficSnapshot {
        ts: now,
        u: total_upload,
        d: total_download,
    });

    let mut avg_up = 0_i64;
    let mut avg_down = 0_i64;
    if let Some(previous) = previous {
        let delta = now - previous.ts;
        if (20..=120).contains(&delta) {
            avg_up = ((total_upload - previous.u) / delta).max(0);
            avg_down = ((total_download - previous.d) / delta).max(0);
        }
    }

    Ok(json!({
        "upload_bps": avg_up,
        "download_bps": avg_down,
    }))
}

pub(crate) async fn load_public_top_users(state: &AppState) -> Result<Vec<Value>, sqlx::Error> {
    let rows = sqlx::query_as::<_, PublicTopUserRow>(
        "SELECT id, email, COALESCE(u, 0) AS u, COALESCE(d, 0) AS d
         FROM v2_user
         ORDER BY (COALESCE(u, 0) + COALESCE(d, 0)) DESC
         LIMIT 10"
    )
    .fetch_all(&state.db)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let value = row.u.saturating_add(row.d).max(0);
            json!({
                "id": row.id.to_string(),
                "name": mask_identifier(&row.email),
                "value": value,
                "value_text": traffic_convert(value),
            })
        })
        .collect())
}

pub(crate) async fn load_public_top_nodes(state: &AppState) -> Result<Vec<Value>, sqlx::Error> {
    let rows = sqlx::query_as::<_, PublicNodeTotalRow>(
        "SELECT server_id, CAST(COALESCE(SUM(u + d), 0) AS SIGNED INTEGER) AS total
         FROM v2_stat_server
         WHERE record_type = 'd'
         GROUP BY server_id
         ORDER BY total DESC
         LIMIT 10"
    )
    .fetch_all(&state.db)
    .await?;

    if rows.is_empty() {
        return Ok(Vec::new());
    }

    let ids = rows.iter().map(|row| row.server_id).collect::<Vec<_>>();
    let mut builder = QueryBuilder::<sqlx::MySql>::new("SELECT id, name FROM v2_server WHERE id IN (");
    {
        let mut separated = builder.separated(", ");
        for id in &ids {
            separated.push_bind(id);
        }
    }
    builder.push(")");
    let name_rows = builder.build().fetch_all(&state.db).await?;
    let name_map = name_rows
        .into_iter()
        .map(|row| {
            (
                row.try_get::<u64, _>("id").unwrap_or_default() as i64,
                row.try_get::<String, _>("name").unwrap_or_default(),
            )
        })
        .collect::<HashMap<_, _>>();

    Ok(rows
        .into_iter()
        .map(|row| {
            let value = row.total.max(0);
            json!({
                "id": row.server_id.to_string(),
                "name": name_map.get(&row.server_id).cloned().unwrap_or_else(|| format!("Node {}", row.server_id)),
                "value": value,
                "value_text": traffic_convert(value),
            })
        })
        .collect())
}
