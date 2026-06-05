use crate::*;
use sqlx::{MySql, QueryBuilder};
use std::collections::HashSet;

#[derive(Clone, Default)]
pub(crate) struct GeneratedUserPlanContext {
    pub(crate) group_id: Option<i64>,
    pub(crate) transfer_enable: i64,
    pub(crate) speed_limit: Option<i64>,
    pub(crate) effective_expired_at: i64,
}

#[derive(Clone, Default)]
pub(crate) struct GeneratedUserDefaults {
    pub(crate) remind_expire: i64,
    pub(crate) remind_traffic: i64,
}

#[derive(Clone)]
pub(crate) struct PreparedGeneratedUser {
    pub(crate) email: String,
    pub(crate) password: String,
    pub(crate) hashed_password: String,
    pub(crate) uuid: String,
    pub(crate) token: String,
    pub(crate) subscribe_path: String,
    pub(crate) subscribe_key: String,
    pub(crate) subscribe_salt: String,
    pub(crate) created_at: i64,
}

pub(crate) async fn load_generated_user_defaults(
    state: &AppState,
) -> GeneratedUserDefaults {
    GeneratedUserDefaults {
        remind_expire: get_setting_int(state, "default_remind_expire", 1).await,
        remind_traffic: get_setting_int(state, "default_remind_traffic", 1).await,
    }
}

pub(crate) async fn load_generated_user_plan_context(
    state: &AppState,
    plan_id: Option<i64>,
    expired_at: Option<i64>,
) -> Result<GeneratedUserPlanContext, Response<Body>> {
    if let Some(plan_id) = plan_id {
        let plan = sqlx::query(
            "SELECT group_id, transfer_enable, speed_limit
             FROM v2_plan
             WHERE id = ?
             LIMIT 1",
        )
        .bind(plan_id)
        .fetch_optional(&state.db)
        .await
        .map_err(internal_error)?;
        let Some(plan) = plan else {
            return Err(json_status_response(StatusCode::BAD_REQUEST, json!({"message":"订阅计划不存在"})));
        };
        let group_id = plan
            .try_get::<Option<u64>, _>("group_id")
            .ok()
            .flatten()
            .and_then(|value| i64::try_from(value).ok());
        let transfer_enable_gb = plan
            .try_get::<Option<u64>, _>("transfer_enable")
            .ok()
            .flatten()
            .unwrap_or(0);
        let speed_limit = plan
            .try_get::<Option<u64>, _>("speed_limit")
            .ok()
            .flatten()
            .and_then(|value| i64::try_from(value).ok());
        let transfer_enable =
            i64::try_from((transfer_enable_gb as u128) * 1024 * 1024 * 1024).unwrap_or(0);
        Ok(GeneratedUserPlanContext {
            group_id,
            transfer_enable,
            speed_limit,
            effective_expired_at: expired_at.unwrap_or(0),
        })
    } else {
        Ok(GeneratedUserPlanContext {
            effective_expired_at: expired_at.unwrap_or(0),
            ..GeneratedUserPlanContext::default()
        })
    }
}

pub(crate) fn unique_candidate_email(
    email_suffix: &str,
    used_emails: &mut HashSet<String>,
) -> Option<String> {
    for _ in 0..20 {
        let email = format!("{}@{}", random_letters(6).to_lowercase(), email_suffix);
        if used_emails.insert(email.clone()) {
            return Some(email);
        }
    }
    None
}

pub(crate) fn prepare_generated_user(
    email: String,
    password: String,
) -> Result<PreparedGeneratedUser, Response<Body>> {
    let subscribe_key = random_letters(8);
    let mut subscribe_salt = random_letters(6);
    while subscribe_salt == subscribe_key {
        subscribe_salt = random_letters(6);
    }
    let hashed_password = bcrypt::hash(&password, 12)
        .map_err(|_| fail_json_response(StatusCode::INTERNAL_SERVER_ERROR, "生成失败"))?;

    Ok(PreparedGeneratedUser {
        email,
        password,
        hashed_password,
        uuid: random_uuid_string(),
        token: random_hex(32),
        subscribe_path: random_letters(10),
        subscribe_key,
        subscribe_salt,
        created_at: Utc::now().timestamp(),
    })
}

pub(crate) async fn prepare_random_generated_users(
    state: &AppState,
    email_suffix: &str,
    fixed_password: Option<&str>,
    count: usize,
) -> Result<Vec<PreparedGeneratedUser>, Response<Body>> {
    let mut used_emails = HashSet::new();
    let mut candidates = Vec::with_capacity(count);
    let mut rounds = 0;

    while candidates.len() < count && rounds < 50 {
        rounds += 1;
        while candidates.len() < count {
            let Some(email) = unique_candidate_email(email_suffix, &mut used_emails) else {
                break;
            };
            candidates.push(email);
        }
        let existing = load_existing_generated_emails(state, &candidates).await?;
        if existing.is_empty() {
            break;
        }
        candidates.retain(|email| !existing.contains(email));
    }

    if candidates.len() != count {
        return Err(fail_json_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "生成失败",
        ));
    }

    candidates
        .into_iter()
        .map(|email| {
            let password = fixed_password.unwrap_or(email.as_str()).to_string();
            prepare_generated_user(email, password)
        })
        .collect()
}

pub(crate) async fn batch_insert_generated_users(
    state: &AppState,
    users: &[PreparedGeneratedUser],
    plan_id: Option<i64>,
    defaults: &GeneratedUserDefaults,
    plan_context: &GeneratedUserPlanContext,
) -> Result<(), Response<Body>> {
    if users.is_empty() {
        return Ok(());
    }

    let mut builder = QueryBuilder::<MySql>::new(
        "INSERT INTO v2_user (
            email, password, uuid, token, subscribe_path, subscribe_key, subscribe_salt,
            remind_expire, remind_traffic, expired_at, concurrent_ip_limit, created_at, updated_at,
            plan_id, group_id, transfer_enable, speed_limit
         ) ",
    );
    builder.push_values(users, |mut row, user| {
        row.push_bind(&user.email)
            .push_bind(&user.hashed_password)
            .push_bind(&user.uuid)
            .push_bind(&user.token)
            .push_bind(&user.subscribe_path)
            .push_bind(&user.subscribe_key)
            .push_bind(&user.subscribe_salt)
            .push_bind(defaults.remind_expire)
            .push_bind(defaults.remind_traffic)
            .push_bind(plan_context.effective_expired_at)
            .push_bind(3_i64)
            .push_bind(user.created_at)
            .push_bind(user.created_at)
            .push_bind(plan_id)
            .push_bind(plan_context.group_id)
            .push_bind(plan_context.transfer_enable)
            .push_bind(plan_context.speed_limit);
    });

    builder
        .build()
        .execute(&state.db)
        .await
        .map_err(|err| {
            if is_duplicate_sqlx_error(&err) {
                json_status_response(StatusCode::BAD_REQUEST, json!({"message":"邮箱已存在于系统中"}))
            } else {
                internal_error(err)
            }
        })?;
    Ok(())
}

pub(crate) async fn create_generated_user(
    state: &AppState,
    email: String,
    password: String,
    plan_id: Option<i64>,
    defaults: &GeneratedUserDefaults,
    plan_context: &GeneratedUserPlanContext,
) -> Result<PreparedGeneratedUser, Response<Body>> {
    let prepared = prepare_generated_user(email, password)?;
    batch_insert_generated_users(
        state,
        std::slice::from_ref(&prepared),
        plan_id,
        defaults,
        plan_context,
    )
    .await?;
    Ok(prepared)
}

pub(crate) fn prepared_user_to_result(
    user: PreparedGeneratedUser,
    effective_expired_at: i64,
) -> Value {
    json!({
        "email": user.email,
        "password": user.password,
        "expired_at": if effective_expired_at > 0 {
            chrono::DateTime::<Utc>::from_timestamp(effective_expired_at, 0)
                .map(|value| value.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| effective_expired_at.to_string())
        } else {
            "长期有效".to_string()
        },
        "uuid": user.uuid,
        "created_at": chrono::DateTime::<Utc>::from_timestamp(user.created_at, 0)
            .map(|value| value.format("%Y-%m-%d %H:%M:%S").to_string())
            .unwrap_or_else(|| user.created_at.to_string()),
        "subscribe_url": build_generated_subscribe_url(&user),
    })
}

fn build_generated_subscribe_url(user: &PreparedGeneratedUser) -> String {
    if std::env::var("APP_URL").unwrap_or_default().is_empty() {
        format!(
            "/s/{}?{}={}&{}=1",
            user.subscribe_path, user.subscribe_key, user.token, user.subscribe_salt
        )
    } else {
        format!(
            "{}/s/{}?{}={}&{}=1",
            std::env::var("APP_URL")
                .unwrap_or_default()
                .trim_end_matches('/'),
            user.subscribe_path,
            user.subscribe_key,
            user.token,
            user.subscribe_salt
        )
    }
}

async fn load_existing_generated_emails(
    state: &AppState,
    emails: &[String],
) -> Result<HashSet<String>, Response<Body>> {
    if emails.is_empty() {
        return Ok(HashSet::new());
    }

    let mut builder = QueryBuilder::<MySql>::new(
        "SELECT email
         FROM v2_user
         WHERE email IN (",
    );
    {
        let mut separated = builder.separated(", ");
        for email in emails {
            separated.push_bind(email);
        }
    }
    builder.push(")");

    let rows = builder
        .build()
        .fetch_all(&state.db)
        .await
        .map_err(internal_error)?;
    Ok(rows
        .into_iter()
        .filter_map(|row| row.try_get::<String, _>("email").ok())
        .collect())
}
