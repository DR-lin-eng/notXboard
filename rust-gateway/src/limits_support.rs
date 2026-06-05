use crate::*;

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct IndividualLimitRow {
    user_id: i64,
    speed_limit_down: i64,
    device_limit: i64,
    connection_limit: i64,
}

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct GroupLimitRow {
    trust_level: i64,
    speed_limit_down: i64,
    device_limit: i64,
    connection_limit: i64,
}

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct AdminGroupLimitRow {
    pub(crate) trust_level: i64,
    pub(crate) speed_limit_up: i64,
    pub(crate) speed_limit_down: i64,
    pub(crate) device_limit: i64,
    pub(crate) connection_limit: i64,
    pub(crate) created_at: Option<chrono::DateTime<Utc>>,
    pub(crate) updated_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Clone)]
pub(crate) struct EffectiveLimits {
    pub(crate) speed_limit_down: i64,
    pub(crate) device_limit: i64,
    pub(crate) connection_limit: i64,
}

#[derive(Clone)]
pub(crate) struct UserLimitProfile {
    pub(crate) user_id: i64,
    pub(crate) trust_level: i64,
}

pub(crate) fn admin_trust_level_display_name(trust_level: i64) -> &'static str {
    match trust_level {
        0 => "New User",
        1 => "Basic User",
        2 => "Member",
        3 => "Regular",
        4 => "Leader",
        _ => "Unknown",
    }
}

pub(crate) fn serialize_admin_group_limit_row(row: AdminGroupLimitRow) -> Value {
    json!({
        "trust_level": row.trust_level,
        "trust_level_name": admin_trust_level_display_name(row.trust_level),
        "speed_limit_up": row.speed_limit_up,
        "speed_limit_down": row.speed_limit_down,
        "device_limit": row.device_limit,
        "connection_limit": row.connection_limit,
        "created_at": row.created_at.map(|value| value.timestamp()),
        "updated_at": row.updated_at.map(|value| value.timestamp()),
    })
}

pub(crate) fn admin_group_limit_defaults() -> Vec<Value> {
    vec![
        json!({"trust_level": 0, "trust_level_name": "New User", "speed_limit_up": 50, "speed_limit_down": 100, "device_limit": 2, "connection_limit": 5}),
        json!({"trust_level": 1, "trust_level_name": "Basic User", "speed_limit_up": 100, "speed_limit_down": 200, "device_limit": 3, "connection_limit": 10}),
        json!({"trust_level": 2, "trust_level_name": "Member", "speed_limit_up": 200, "speed_limit_down": 500, "device_limit": 5, "connection_limit": 20}),
        json!({"trust_level": 3, "trust_level_name": "Regular", "speed_limit_up": 500, "speed_limit_down": 1000, "device_limit": 8, "connection_limit": 50}),
        json!({"trust_level": 4, "trust_level_name": "Leader", "speed_limit_up": 1000, "speed_limit_down": 2000, "device_limit": 15, "connection_limit": 100}),
    ]
}

pub(crate) fn validate_admin_group_limit_values(
    trust_level: i64,
    speed_limit_up: i64,
    speed_limit_down: i64,
    device_limit: i64,
    connection_limit: i64,
) -> Result<(), Response<Body>> {
    if !(0..=4).contains(&trust_level)
        || speed_limit_up < 0
        || speed_limit_down < 0
        || device_limit < 0
        || connection_limit < 0
    {
        return Err(json_status_response(StatusCode::BAD_REQUEST, json!({
            "success": false,
            "error": "Invalid limit configuration"
        })));
    }
    Ok(())
}

fn effective_limit_value(individual: Option<i64>, group: Option<i64>, default_value: i64) -> i64 {
    if let Some(value) = individual {
        if value > 0 {
            return value;
        }
    }
    if let Some(value) = group {
        if value > 0 {
            return value;
        }
    }
    default_value
}

pub(crate) async fn load_effective_limits(
    state: &AppState,
    users: &[UserRow],
) -> Result<HashMap<i64, EffectiveLimits>, sqlx::Error> {
    if users.is_empty() {
        return Ok(HashMap::new());
    }

    let profiles = users
        .iter()
        .map(|user| UserLimitProfile {
            user_id: user.id,
            trust_level: user.trust_level.unwrap_or(0),
        })
        .collect::<Vec<_>>();
    load_effective_limits_by_profiles(state, &profiles).await
}

pub(crate) async fn load_effective_limits_by_profiles(
    state: &AppState,
    users: &[UserLimitProfile],
) -> Result<HashMap<i64, EffectiveLimits>, sqlx::Error> {
    if users.is_empty() {
        return Ok(HashMap::new());
    }

    let user_ids = users.iter().map(|user| user.user_id).collect::<Vec<_>>();
    let placeholders = vec!["?"; user_ids.len()].join(",");
    let sql = format!(
        "SELECT user_id, speed_limit_down, device_limit, connection_limit
         FROM user_individual_limits
         WHERE user_id IN ({})",
        placeholders
    );
    let mut query = sqlx::query_as::<_, IndividualLimitRow>(&sql);
    for user_id in &user_ids {
        query = query.bind(*user_id);
    }
    let individual_limits = query.fetch_all(&state.db).await?;
    let individual_map = individual_limits
        .into_iter()
        .map(|limit| (limit.user_id, limit))
        .collect::<HashMap<_, _>>();

    let trust_levels = users
        .iter()
        .map(|user| user.trust_level)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let group_map = if trust_levels.is_empty() {
        HashMap::new()
    } else {
        let placeholders = vec!["?"; trust_levels.len()].join(",");
        let sql = format!(
            "SELECT trust_level, speed_limit_down, device_limit, connection_limit
             FROM user_group_limits
             WHERE trust_level IN ({})",
            placeholders
        );
        let mut query = sqlx::query_as::<_, GroupLimitRow>(&sql);
        for trust_level in &trust_levels {
            query = query.bind(*trust_level);
        }
        query
            .fetch_all(&state.db)
            .await?
            .into_iter()
            .map(|limit| (limit.trust_level, limit))
            .collect::<HashMap<_, _>>()
    };

    let mut result = HashMap::new();
    for user in users {
        let individual = individual_map.get(&user.user_id);
        let group = group_map.get(&user.trust_level);
        result.insert(
            user.user_id,
            EffectiveLimits {
                speed_limit_down: effective_limit_value(individual.map(|v| v.speed_limit_down), group.map(|v| v.speed_limit_down), 0),
                device_limit: effective_limit_value(individual.map(|v| v.device_limit), group.map(|v| v.device_limit), 2),
                connection_limit: effective_limit_value(individual.map(|v| v.connection_limit), group.map(|v| v.connection_limit), 10),
            },
        );
    }

    Ok(result)
}

pub(crate) async fn load_user_individual_limit_map(
    state: &AppState,
    user_ids: &[i64],
) -> Result<HashMap<i64, AdminGroupLimitRow>, sqlx::Error> {
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let placeholders = vec!["?"; user_ids.len()].join(",");
    let sql = format!(
        "SELECT user_id AS trust_level, speed_limit_up, speed_limit_down, device_limit, connection_limit, created_at, updated_at
         FROM user_individual_limits
         WHERE user_id IN ({})",
        placeholders
    );
    let mut query = sqlx::query_as::<_, AdminGroupLimitRow>(&sql);
    for user_id in user_ids {
        query = query.bind(*user_id);
    }
    Ok(query
        .fetch_all(&state.db)
        .await?
        .into_iter()
        .map(|row| (row.trust_level, row))
        .collect::<HashMap<_, _>>())
}
