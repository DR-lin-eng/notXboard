use crate::*;

use super::models::{FailedJobRow, SystemLogRow};

pub async fn load_system_logs(
    state: &AppState,
    level: Option<&str>,
    keyword: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<(Vec<SystemLogRow>, i64), sqlx::Error> {
    let mut where_clauses = Vec::new();
    if level.is_some() {
        where_clauses.push("level = ?");
    }
    if keyword.is_some() {
        where_clauses.push("(data LIKE ? OR context LIKE ? OR title LIKE ? OR uri LIKE ?)");
    }
    let where_sql = if where_clauses.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", where_clauses.join(" AND "))
    };
    let select_sql = format!(
        "SELECT id, title, level, uri, data, context, created_at
         FROM v2_log{}
         ORDER BY created_at DESC
         LIMIT ? OFFSET ?",
        where_sql
    );
    let count_sql = format!("SELECT COUNT(*) FROM v2_log{}", where_sql);
    let mut select = sqlx::query_as::<_, SystemLogRow>(&select_sql);
    let mut count = sqlx::query_scalar::<_, i64>(&count_sql);
    if let Some(level) = level {
        select = select.bind(level);
        count = count.bind(level);
    }
    if let Some(keyword) = keyword {
        let pattern = format!("%{}%", keyword);
        for _ in 0..4 {
            select = select.bind(pattern.clone());
            count = count.bind(pattern.clone());
        }
    }
    let total = count.fetch_one(&state.db).await?;
    let rows = select.bind(limit).bind(offset).fetch_all(&state.db).await?;
    Ok((rows, total))
}

pub async fn load_failed_jobs(
    state: &AppState,
    offset: i64,
    limit: i64,
) -> Result<(Vec<FailedJobRow>, i64), sqlx::Error> {
    let rows = sqlx::query_as::<_, FailedJobRow>(
        "SELECT id, connection, queue, failed_at
         FROM failed_jobs
         ORDER BY failed_at DESC
         LIMIT ? OFFSET ?",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.db)
    .await?;
    let total = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM failed_jobs")
        .fetch_one(&state.db)
        .await?;
    Ok((rows, total))
}

pub async fn count_logs_to_clear(
    state: &AppState,
    cutoff: i64,
    level: Option<&str>,
) -> Result<i64, sqlx::Error> {
    if let Some(level) = level {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM v2_log WHERE created_at < ? AND level = ?")
            .bind(cutoff)
            .bind(level)
            .fetch_one(&state.db)
            .await
    } else {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM v2_log WHERE created_at < ?")
            .bind(cutoff)
            .fetch_one(&state.db)
            .await
    }
}

pub async fn load_log_ids_to_clear(
    state: &AppState,
    cutoff: i64,
    level: Option<&str>,
    limit: i64,
) -> Result<Vec<i64>, sqlx::Error> {
    if let Some(level) = level {
        sqlx::query_scalar::<_, i64>(
            "SELECT id FROM v2_log WHERE created_at < ? AND level = ? ORDER BY id ASC LIMIT ?",
        )
        .bind(cutoff)
        .bind(level)
        .bind(limit)
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_scalar::<_, i64>(
            "SELECT id FROM v2_log WHERE created_at < ? ORDER BY id ASC LIMIT ?",
        )
        .bind(cutoff)
        .bind(limit)
        .fetch_all(&state.db)
        .await
    }
}

pub async fn delete_log_ids(
    state: &AppState,
    ids: &[i64],
) -> Result<i64, sqlx::Error> {
    let placeholders = vec!["?"; ids.len()].join(",");
    let sql = format!("DELETE FROM v2_log WHERE id IN ({})", placeholders);
    let mut query = sqlx::query(&sql);
    for id in ids {
        query = query.bind(*id);
    }
    let result = query.execute(&state.db).await?;
    Ok(result.rows_affected() as i64)
}

pub async fn load_log_edge(
    state: &AppState,
    oldest: bool,
) -> Result<Option<SystemLogRow>, sqlx::Error> {
    let order = if oldest { "ASC" } else { "DESC" };
    let sql = format!(
        "SELECT id, title, level, uri, data, context, created_at
         FROM v2_log
         ORDER BY created_at {}
         LIMIT 1",
        order
    );
    sqlx::query_as::<_, SystemLogRow>(&sql)
        .fetch_optional(&state.db)
        .await
}
