use crate::*;
use sqlx::{MySql, QueryBuilder};

const SECURITY_RESET_BATCH_SIZE: usize = 200;

#[derive(Clone)]
pub(crate) struct UserSecurityResetData {
    pub(crate) uuid: String,
    pub(crate) token: String,
    pub(crate) subscribe_path: String,
    pub(crate) subscribe_key: String,
    pub(crate) subscribe_salt: String,
}

pub(crate) fn generate_user_security_reset_data() -> UserSecurityResetData {
    let subscribe_key = random_letters(8);
    let mut subscribe_salt = random_letters(6);
    while subscribe_salt == subscribe_key {
        subscribe_salt = random_letters(6);
    }

    UserSecurityResetData {
        uuid: random_uuid_string(),
        token: random_hex(32),
        subscribe_path: random_letters(10),
        subscribe_key,
        subscribe_salt,
    }
}

pub(crate) async fn reset_single_user_security(
    state: &AppState,
    user_id: i64,
) -> Result<UserSecurityResetData, sqlx::Error> {
    let data = generate_user_security_reset_data();
    let updated = sqlx::query(
        "UPDATE v2_user
         SET uuid = ?, token = ?, subscribe_path = ?, subscribe_key = ?, subscribe_salt = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(&data.uuid)
    .bind(&data.token)
    .bind(&data.subscribe_path)
    .bind(&data.subscribe_key)
    .bind(&data.subscribe_salt)
    .bind(Utc::now().timestamp())
    .bind(user_id)
    .execute(&state.db)
    .await?;
    if updated.rows_affected() > 0 {
        clear_all_authorization_caches(state);
    }
    Ok(data)
}

pub(crate) async fn reset_many_user_security(
    state: &AppState,
    user_ids: &[i64],
) -> Result<u64, sqlx::Error> {
    let now_ts = Utc::now().timestamp();
    let mut updated = 0_u64;

    for chunk in user_ids.chunks(SECURITY_RESET_BATCH_SIZE) {
        let mut builder = QueryBuilder::<MySql>::new("UPDATE v2_user u JOIN (");
        for (index, user_id) in chunk.iter().enumerate() {
            let data = generate_user_security_reset_data();
            if index > 0 {
                builder.push(" UNION ALL ");
            }
            builder
                .push("SELECT ")
                .push_bind(user_id)
                .push(" AS id, ")
                .push_bind(data.uuid)
                .push(" AS uuid, ")
                .push_bind(data.token)
                .push(" AS token, ")
                .push_bind(data.subscribe_path)
                .push(" AS subscribe_path, ")
                .push_bind(data.subscribe_key)
                .push(" AS subscribe_key, ")
                .push_bind(data.subscribe_salt)
                .push(" AS subscribe_salt");
        }
        builder.push(
            ") vals ON vals.id = u.id
             SET u.uuid = vals.uuid,
                 u.token = vals.token,
                 u.subscribe_path = vals.subscribe_path,
                 u.subscribe_key = vals.subscribe_key,
                 u.subscribe_salt = vals.subscribe_salt,
                 u.updated_at = ",
        );
        builder.push_bind(now_ts);
        let result = builder.build().execute(&state.db).await?;
        updated += result.rows_affected();
    }

    if updated > 0 {
        clear_all_authorization_caches(state);
    }
    Ok(updated)
}
