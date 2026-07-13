use crate::*;

pub(crate) type AggregatedTrafficRow = (i64, i64, i64, i64);

const BATCH_ROWS: usize = 500;

pub(crate) async fn batch_insert_node_traffic_records(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    node_id: u64,
    record_date: &str,
    rows: &[AggregatedTrafficRow],
) -> Result<(), sqlx::Error> {
    for chunk in rows.chunks(BATCH_ROWS) {
        let values_sql = vec!["(?, ?, ?, ?, ?, NOW(), NOW())"; chunk.len()].join(",");
        let sql = format!(
            "INSERT INTO node_traffic_records
                (user_id, node_id, record_date, upload_traffic, download_traffic, created_at, updated_at)
             VALUES {}
             ON DUPLICATE KEY UPDATE
                upload_traffic = upload_traffic + VALUES(upload_traffic),
                download_traffic = download_traffic + VALUES(download_traffic),
                updated_at = VALUES(updated_at)",
            values_sql,
        );
        let mut query = sqlx::query(&sql);
        for (user_id, upload, download, _) in chunk {
            query = query
                .bind(*user_id)
                .bind(node_id)
                .bind(record_date)
                .bind(*upload)
                .bind(*download);
        }
        query.execute(&mut **tx).await?;
    }
    Ok(())
}

pub(crate) async fn batch_insert_user_traffic_usage_logs(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    node_id: u64,
    now_ts: i64,
    multiplier: f64,
    rows: &[AggregatedTrafficRow],
) -> Result<(), sqlx::Error> {
    for chunk in rows.chunks(BATCH_ROWS) {
        let values_sql = vec!["(?, ?, ?, ?, ?, 'push', ?, ?, ?)"; chunk.len()].join(",");
        let sql = format!(
            "INSERT INTO user_traffic_usage_logs
                (user_id, node_id, raw_traffic_kb, billed_traffic_kb, multiplier_snapshot, source, recorded_at, created_at, updated_at)
             VALUES {}",
            values_sql,
        );
        let mut query = sqlx::query(&sql);
        for (user_id, upload, download, billed) in chunk {
            query = query
                .bind(*user_id)
                .bind(node_id)
                .bind(*upload + *download)
                .bind(*billed)
                .bind(multiplier)
                .bind(now_ts)
                .bind(now_ts)
                .bind(now_ts);
        }
        query.execute(&mut **tx).await?;
    }
    Ok(())
}

pub(crate) async fn batch_upsert_legacy_stat_user(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    server_rate: f64,
    record_at: i64,
    now_ts: i64,
    rows: &[AggregatedTrafficRow],
) -> Result<(), sqlx::Error> {
    for chunk in rows.chunks(BATCH_ROWS) {
        let values_sql = vec!["(?, ?, ?, 'd', ?, ?, ?, ?)"; chunk.len()].join(",");
        let sql = format!(
            "INSERT INTO v2_stat_user
                (user_id, server_rate, record_at, record_type, u, d, created_at, updated_at)
             VALUES {}
             ON DUPLICATE KEY UPDATE
                u = u + VALUES(u),
                d = d + VALUES(d),
                updated_at = VALUES(updated_at)",
            values_sql,
        );
        let mut query = sqlx::query(&sql);
        for (user_id, upload, download, _) in chunk {
            query = query
                .bind(*user_id)
                .bind(server_rate)
                .bind(record_at)
                .bind(*upload)
                .bind(*download)
                .bind(now_ts)
                .bind(now_ts);
        }
        query.execute(&mut **tx).await?;
    }
    Ok(())
}

pub(crate) async fn batch_update_user_traffic_totals(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    now_ts: i64,
    rows: &[AggregatedTrafficRow],
) -> Result<(), sqlx::Error> {
    for chunk in rows.chunks(BATCH_ROWS) {
        let payload = serde_json::to_string(
            &chunk
                .iter()
                .map(|(user_id, upload, download, _)| {
                    json!({
                        "user_id": user_id,
                        "upload": upload,
                        "download": download,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .map_err(|err| sqlx::Error::Protocol(format!("serialize user traffic delta failed: {err}")))?;
        sqlx::query(
            "UPDATE v2_user u
             JOIN JSON_TABLE(
                ?,
                '$[*]' COLUMNS(
                    user_id BIGINT PATH '$.user_id',
                    upload BIGINT PATH '$.upload',
                    download BIGINT PATH '$.download'
                )
             ) delta ON u.id = delta.user_id
             SET u.u = COALESCE(u.u, 0) + delta.upload,
                 u.d = COALESCE(u.d, 0) + delta.download,
                 u.t = ?",
        )
        .bind(payload)
        .bind(now_ts)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

pub(crate) async fn load_primary_subscription_ids_for_node_users(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    node_id: u64,
    now_ts: i64,
    user_ids: &[i64],
) -> Result<HashMap<i64, i64>, sqlx::Error> {
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut result = HashMap::new();
    for chunk in user_ids.chunks(BATCH_ROWS) {
        let placeholders = vec!["?"; chunk.len()].join(",");
        let sql = format!(
            "SELECT user_id, subscription_id
             FROM (
                 SELECT
                     ups.user_id AS user_id,
                     ups.id AS subscription_id,
                     ROW_NUMBER() OVER (
                         PARTITION BY ups.user_id
                         ORDER BY
                             CASE WHEN ups.expired_at IS NULL THEN 1 ELSE 0 END DESC,
                             ups.expired_at ASC,
                             ups.id ASC
                     ) AS row_num
                 FROM user_plan_subscriptions ups
                 JOIN v2_plan plan ON plan.id = ups.plan_id AND plan.scope = 'node'
                 JOIN server_nodes node ON node.id = ? AND node.user_id = plan.owner_user_id
                 WHERE JSON_CONTAINS(
                           COALESCE(plan.node_ids, JSON_ARRAY()),
                           CAST(node.id AS JSON),
                           '$'
                       )
                   AND ups.status = 1
                   AND (ups.expired_at IS NULL OR ups.expired_at > ?)
                   AND ups.user_id IN ({})
             ) ranked
             WHERE row_num = 1",
            placeholders,
        );
        let mut query = sqlx::query(&sql).bind(node_id).bind(now_ts);
        for user_id in chunk {
            query = query.bind(*user_id);
        }
        let rows = query.fetch_all(&mut **tx).await?;
        for row in rows {
            let user_id: i64 = row.get("user_id");
            let subscription_id: i64 = row.get("subscription_id");
            result.insert(user_id, subscription_id);
        }
    }

    Ok(result)
}

pub(crate) async fn batch_update_subscription_usage(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    rows: &[(i64, i64)],
) -> Result<(), sqlx::Error> {
    if rows.is_empty() {
        return Ok(());
    }

    for chunk in rows.chunks(BATCH_ROWS) {
        let payload = serde_json::to_string(
            &chunk
                .iter()
                .map(|(subscription_id, billed)| {
                    json!({
                        "subscription_id": subscription_id,
                        "billed": billed,
                    })
                })
                .collect::<Vec<_>>(),
        )
        .map_err(|err| sqlx::Error::Protocol(format!("serialize subscription usage delta failed: {err}")))?;
        sqlx::query(
            "UPDATE user_plan_subscriptions ups
             JOIN JSON_TABLE(
                ?,
                '$[*]' COLUMNS(
                    subscription_id BIGINT PATH '$.subscription_id',
                    billed BIGINT PATH '$.billed'
                )
             ) delta ON ups.id = delta.subscription_id
             SET ups.used_traffic_kb = ups.used_traffic_kb + delta.billed",
        )
        .bind(payload)
        .execute(&mut **tx)
        .await?;
    }

    Ok(())
}
