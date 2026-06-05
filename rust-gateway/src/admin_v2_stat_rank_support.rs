use crate::*;
use sqlx::QueryBuilder;

pub(crate) async fn load_traffic_rank_names(
    state: &AppState,
    rank_type: &str,
    ids: &[i64],
) -> Result<HashMap<i64, String>, sqlx::Error> {
    if ids.is_empty() {
        return Ok(HashMap::new());
    }

    match rank_type {
        "node" => load_server_rank_names(state, ids).await,
        "user" => load_user_rank_names(state, ids).await,
        _ => Ok(HashMap::new()),
    }
}

async fn load_server_rank_names(
    state: &AppState,
    ids: &[i64],
) -> Result<HashMap<i64, String>, sqlx::Error> {
    let mut builder = QueryBuilder::<sqlx::MySql>::new(
        "SELECT s.id, COALESCE(parent.name, s.name) AS server_name
         FROM v2_server s
         LEFT JOIN v2_server parent ON parent.id = s.parent_id
         WHERE s.id IN (",
    );
    {
        let mut separated = builder.separated(", ");
        for id in ids {
            separated.push_bind(id);
        }
    }
    builder.push(")");

    let rows = builder.build().fetch_all(&state.db).await?;
    let mut names = HashMap::with_capacity(rows.len());
    for row in rows {
        let Some(id) = row.try_get::<Option<i64>, _>("id").ok().flatten() else {
            continue;
        };
        let Some(name) = row
            .try_get::<Option<String>, _>("server_name")
            .ok()
            .flatten()
        else {
            continue;
        };
        names.insert(id, name);
    }
    Ok(names)
}

async fn load_user_rank_names(
    state: &AppState,
    ids: &[i64],
) -> Result<HashMap<i64, String>, sqlx::Error> {
    let mut builder = QueryBuilder::<sqlx::MySql>::new(
        "SELECT id, email
         FROM v2_user
         WHERE id IN (",
    );
    {
        let mut separated = builder.separated(", ");
        for id in ids {
            separated.push_bind(id);
        }
    }
    builder.push(")");

    let rows = builder.build().fetch_all(&state.db).await?;
    let mut names = HashMap::with_capacity(rows.len());
    for row in rows {
        let Some(id) = row.try_get::<Option<i64>, _>("id").ok().flatten() else {
            continue;
        };
        let Some(email) = row.try_get::<Option<String>, _>("email").ok().flatten() else {
            continue;
        };
        names.insert(id, email);
    }
    Ok(names)
}
