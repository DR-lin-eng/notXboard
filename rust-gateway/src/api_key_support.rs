use crate::*;
use sqlx::{MySql, QueryBuilder, Row};
use std::collections::{HashMap, HashSet};

const API_KEY_BATCH_SIZE: usize = 200;

#[derive(Clone)]
pub(crate) struct ApiKeyAssignment {
    pub(crate) user_id: i64,
    pub(crate) api_key: String,
}

pub(crate) fn generate_api_key_candidate() -> String {
    format!("xb_{}", random_hex(60))
}

pub(crate) fn api_key_prefix(value: &str) -> String {
    format!("{}...", &value[..std::cmp::min(10, value.len())])
}

pub(crate) fn api_key_has_valid_format(value: &str) -> bool {
    value.starts_with("xb_")
        && value.len() == 63
        && value[3..]
            .chars()
            .all(|ch| ch.is_ascii_digit() || ('a'..='f').contains(&ch))
}

pub(crate) async fn load_user_api_key(
    state: &AppState,
    user_id: i64,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar::<_, Option<String>>("SELECT api_key FROM v2_user WHERE id = ? LIMIT 1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await
}

pub(crate) async fn ensure_user_api_key(
    state: &AppState,
    user: &BearerUserRow,
) -> Result<String, sqlx::Error> {
    if let Some(api_key) = &user.api_key {
        if !api_key.trim().is_empty() {
            return Ok(api_key.clone());
        }
    }

    loop {
        let candidate = generate_api_key_candidate();
        match sqlx::query(
            "UPDATE v2_user
             SET api_key = ?, updated_at = ?
             WHERE id = ? AND (api_key IS NULL OR api_key = '')",
        )
        .bind(&candidate)
        .bind(Utc::now().timestamp())
        .bind(user.id)
        .execute(&state.db)
        .await
        {
            Ok(result) if result.rows_affected() > 0 => return Ok(candidate),
            Ok(_) => {
                if let Some(existing) = load_user_api_key(state, user.id).await? {
                    if !existing.trim().is_empty() {
                        return Ok(existing);
                    }
                }
            }
            Err(err) if is_duplicate_sqlx_error(&err) => continue,
            Err(err) => return Err(err),
        }
    }
}

pub(crate) async fn reset_user_api_key(
    state: &AppState,
    user_id: i64,
) -> Result<String, sqlx::Error> {
    loop {
        let candidate = generate_api_key_candidate();
        match sqlx::query("UPDATE v2_user SET api_key = ?, updated_at = ? WHERE id = ?")
            .bind(&candidate)
            .bind(Utc::now().timestamp())
            .bind(user_id)
            .execute(&state.db)
            .await
        {
            Ok(_) => return Ok(candidate),
            Err(err) if is_duplicate_sqlx_error(&err) => continue,
            Err(err) => return Err(err),
        }
    }
}

pub(crate) async fn fill_missing_api_keys_for_users(
    state: &AppState,
    user_ids: &[i64],
) -> Result<Vec<ApiKeyAssignment>, sqlx::Error> {
    assign_api_keys_for_users(state, user_ids, true).await
}

pub(crate) async fn replace_api_keys_for_users(
    state: &AppState,
    user_ids: &[i64],
) -> Result<Vec<ApiKeyAssignment>, sqlx::Error> {
    assign_api_keys_for_users(state, user_ids, false).await
}

async fn assign_api_keys_for_users(
    state: &AppState,
    user_ids: &[i64],
    only_when_missing: bool,
) -> Result<Vec<ApiKeyAssignment>, sqlx::Error> {
    let mut used_keys = HashSet::new();
    let mut all_assignments = Vec::with_capacity(user_ids.len());

    for chunk in user_ids.chunks(API_KEY_BATCH_SIZE) {
        loop {
            let assignments = build_unique_assignments(chunk, &mut used_keys);
            match execute_assignment_batch(state, &assignments, only_when_missing).await {
                Ok(()) => {
                    if only_when_missing {
                        let actual = load_current_api_keys_for_users(state, chunk).await?;
                        all_assignments.extend(actual);
                    } else {
                        all_assignments.extend(assignments);
                    }
                    break;
                }
                Err(err) if is_duplicate_sqlx_error(&err) => continue,
                Err(err) => return Err(err),
            }
        }
    }

    Ok(all_assignments)
}

fn build_unique_assignments(
    user_ids: &[i64],
    used_keys: &mut HashSet<String>,
) -> Vec<ApiKeyAssignment> {
    let mut assignments = Vec::with_capacity(user_ids.len());
    for &user_id in user_ids {
        loop {
            let candidate = generate_api_key_candidate();
            if used_keys.insert(candidate.clone()) {
                assignments.push(ApiKeyAssignment {
                    user_id,
                    api_key: candidate,
                });
                break;
            }
        }
    }
    assignments
}

async fn execute_assignment_batch(
    state: &AppState,
    assignments: &[ApiKeyAssignment],
    only_when_missing: bool,
) -> Result<(), sqlx::Error> {
    if assignments.is_empty() {
        return Ok(());
    }

    let now_ts = Utc::now().timestamp();
    let mut builder = QueryBuilder::<MySql>::new("UPDATE v2_user u JOIN (");
    for (index, assignment) in assignments.iter().enumerate() {
        if index > 0 {
            builder.push(" UNION ALL ");
        }
        builder
            .push("SELECT ")
            .push_bind(assignment.user_id)
            .push(" AS id, ")
            .push_bind(&assignment.api_key)
            .push(" AS api_key, ")
            .push_bind(now_ts)
            .push(" AS updated_at");
    }
    builder.push(
        ") vals ON vals.id = u.id
         SET u.api_key = vals.api_key,
             u.updated_at = vals.updated_at",
    );
    if only_when_missing {
        builder.push(" WHERE u.api_key IS NULL OR u.api_key = ''");
    }

    builder.build().execute(&state.db).await?;
    Ok(())
}

async fn load_current_api_keys_for_users(
    state: &AppState,
    user_ids: &[i64],
) -> Result<Vec<ApiKeyAssignment>, sqlx::Error> {
    if user_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut builder = QueryBuilder::<MySql>::new(
        "SELECT id, api_key
         FROM v2_user
         WHERE id IN (",
    );
    {
        let mut separated = builder.separated(", ");
        for user_id in user_ids {
            separated.push_bind(user_id);
        }
    }
    builder.push(")");

    let rows = builder.build().fetch_all(&state.db).await?;
    let keyed = rows
        .into_iter()
        .filter_map(|row| {
            let user_id = row.try_get::<i64, _>("id").ok()?;
            let api_key = row.try_get::<Option<String>, _>("api_key").ok().flatten()?;
            if api_key.trim().is_empty() {
                return None;
            }
            Some((user_id, api_key))
        })
        .collect::<HashMap<_, _>>();

    let mut assignments = Vec::with_capacity(user_ids.len());
    for user_id in user_ids {
        let Some(api_key) = keyed.get(user_id) else {
            return Err(sqlx::Error::Protocol(format!(
                "missing api key after batch assignment for user_id={user_id}"
            )));
        };
        assignments.push(ApiKeyAssignment {
            user_id: *user_id,
            api_key: api_key.clone(),
        });
    }
    Ok(assignments)
}
