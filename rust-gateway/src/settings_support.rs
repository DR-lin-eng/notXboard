use crate::*;

const ALL_SETTINGS_CACHE_KEY: &str = "__notxboard_all_settings__";

pub(crate) async fn get_setting_string(state: &AppState, name: &str, default: &str) -> String {
    load_cached_setting_value(state, name)
        .await
        .unwrap_or(None)
        .unwrap_or_else(|| default.to_string())
}

pub(crate) async fn get_setting_value(state: &AppState, name: &str) -> Value {
    load_cached_setting_value(state, name)
        .await
        .unwrap_or(None)
        .map_or(Value::Null, |value| maybe_decode_setting_value(&value))
}

pub(crate) async fn get_setting_f64(state: &AppState, name: &str, default: f64) -> f64 {
    load_cached_setting_value(state, name)
        .await
        .unwrap_or(None)
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(default)
}

pub(crate) async fn get_setting_int(state: &AppState, name: &str, default: i64) -> i64 {
    load_cached_setting_value(state, name)
        .await
        .unwrap_or(None)
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(default)
}

pub(crate) async fn get_setting_bool(state: &AppState, name: &str, default: bool) -> bool {
    load_cached_setting_value(state, name)
        .await
        .unwrap_or(None)
        .map(|value| matches!(value.trim(), "1" | "true" | "TRUE" | "on" | "yes"))
        .unwrap_or(default)
}

pub(crate) async fn setting_json_or_csv_array(
    state: &AppState,
    name: &str,
    defaults: &[&str],
) -> Value {
    match load_cached_setting_value(state, name).await.unwrap_or(None) {
        Some(raw) => {
            let parsed = maybe_decode_setting_value(&raw);
            match parsed {
                Value::Array(items) => Value::Array(
                    items
                        .into_iter()
                        .filter_map(|item| {
                            item.as_str()
                                .map(|value| Value::String(value.trim().to_string()))
                        })
                        .filter(|item| {
                            item.as_str()
                                .map(|value| !value.is_empty())
                                .unwrap_or(false)
                        })
                        .collect(),
                ),
                Value::String(value) => csv_to_array_value(&value, defaults),
                Value::Null => csv_to_array_value("", defaults),
                _ => csv_to_array_value("", defaults),
            }
        }
        None => Value::Array(
            defaults
                .iter()
                .map(|value| Value::String((*value).to_string()))
                .collect(),
        ),
    }
}

pub(crate) async fn upsert_setting_string(
    state: &AppState,
    name: &str,
    value: &str,
) -> Result<(), sqlx::Error> {
    upsert_setting_string_with_metadata(state, None, None, name, value).await
}

pub(crate) async fn upsert_setting_string_with_metadata(
    state: &AppState,
    group: Option<&str>,
    setting_type: Option<&str>,
    name: &str,
    value: &str,
) -> Result<(), sqlx::Error> {
    let now = Utc::now().timestamp();
    let updated = sqlx::query(
        "UPDATE v2_settings
         SET `group` = COALESCE(?, `group`),
             `type` = COALESCE(?, `type`),
             value = ?,
             updated_at = FROM_UNIXTIME(?)
         WHERE name = ?",
    )
    .bind(group)
    .bind(setting_type)
    .bind(value)
    .bind(now)
    .bind(name)
    .execute(&state.db)
    .await?;

    if updated.rows_affected() == 0 {
        sqlx::query(
            "INSERT INTO v2_settings (`group`, `type`, `name`, `value`, created_at, updated_at)
             VALUES (?, ?, ?, ?, FROM_UNIXTIME(?), FROM_UNIXTIME(?))",
        )
        .bind(group)
        .bind(setting_type)
        .bind(name)
        .bind(value)
        .bind(now)
        .bind(now)
        .execute(&state.db)
        .await?;
    }

    invalidate_setting_cache(state, name);
    Ok(())
}

pub(crate) fn clear_settings_cache(state: &AppState) {
    state.settings_cache.write().clear();
}

pub(crate) async fn resolve_register_mode(state: &AppState) -> String {
    let raw = get_setting_string(state, "register_mode", "").await;
    let raw = raw.trim().to_lowercase();
    if raw.is_empty() {
        if get_setting_bool(state, "stop_register", false).await {
            return "closed".to_string();
        }
        return "all".to_string();
    }

    match raw.as_str() {
        "email_only" | "oauth_only" | "closed" => raw,
        _ => "all".to_string(),
    }
}

pub(crate) async fn resolve_oauth_linux_do_available(state: &AppState) -> bool {
    if !get_setting_bool(state, "oauth_linux_do_enable", true).await {
        return false;
    }
    let client_id = first_non_empty(&[
        get_setting_string(state, "oauth_linux_do_client_id", "").await,
        env::var("LINUX_DO_CLIENT_ID").unwrap_or_default(),
    ]);
    let client_secret = first_non_empty(&[
        get_setting_string(state, "oauth_linux_do_client_secret", "").await,
        env::var("LINUX_DO_CLIENT_SECRET").unwrap_or_default(),
    ]);

    !client_id.is_empty() && !client_secret.is_empty()
}

pub(crate) async fn resolve_pow_effective_difficulty(
    state: &AppState,
    configured_difficulty: i64,
) -> i64 {
    if !get_setting_bool(state, "pow_auto_scale_enable", true).await {
        return configured_difficulty;
    }

    let load_ratio = normalized_load_ratio();
    let queue_backlog = redis_queue_backlog();
    let increment = calculate_pow_increment(load_ratio, queue_backlog);
    let auto_max = get_setting_int(state, "pow_auto_max_difficulty", 7)
        .await
        .clamp(configured_difficulty, 8);
    (configured_difficulty + increment).clamp(configured_difficulty, auto_max)
}

fn maybe_decode_setting_value(value: &str) -> Value {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Value::Null;
    }
    if let Ok(parsed) = serde_json::from_str::<Value>(trimmed) {
        return parsed;
    }
    Value::String(value.to_string())
}

fn csv_to_array_value(value: &str, defaults: &[&str]) -> Value {
    let items = if value.trim().is_empty() {
        defaults.iter().map(|item| item.to_string()).collect::<Vec<_>>()
    } else {
        value
            .split(',')
            .map(|item| item.trim().to_string())
            .filter(|item| !item.is_empty())
            .collect::<Vec<_>>()
    };
    Value::Array(items.into_iter().map(Value::String).collect())
}

fn normalized_load_ratio() -> Option<f64> {
    let content = std::fs::read_to_string("/proc/loadavg").ok()?;
    let one_minute = content.split_whitespace().next()?.parse::<f64>().ok()?;
    let cpu_count = std::thread::available_parallelism().ok()?.get() as f64;
    if cpu_count <= 0.0 {
        return None;
    }
    Some(one_minute / cpu_count)
}

fn redis_queue_backlog() -> Option<i64> {
    None
}

fn calculate_pow_increment(load_ratio: Option<f64>, queue_backlog: Option<i64>) -> i64 {
    let mut increment = 0;
    if let Some(load_ratio) = load_ratio {
        if load_ratio >= 1.5 {
            increment = increment.max(3);
        } else if load_ratio >= 1.0 {
            increment = increment.max(2);
        } else if load_ratio >= 0.7 {
            increment = increment.max(1);
        }
    }
    if let Some(queue_backlog) = queue_backlog {
        if queue_backlog >= 1000 {
            increment = increment.max(3);
        } else if queue_backlog >= 300 {
            increment = increment.max(2);
        } else if queue_backlog >= 100 {
            increment = increment.max(1);
        }
    }
    increment
}

pub(crate) async fn warm_setting_cache(
    state: &AppState,
    names: &[&str],
) -> Result<(), sqlx::Error> {
    let now = Instant::now();
    let mut seen = HashSet::new();
    let normalized = names
        .iter()
        .map(|name| normalized_setting_cache_key(name))
        .filter(|name| !name.is_empty() && seen.insert(name.clone()))
        .collect::<Vec<_>>();
    if normalized.is_empty() {
        return Ok(());
    }

    let missing = {
        let cache = state.settings_cache.read();
        normalized
            .iter()
            .filter(|name| {
                cache
                    .get(*name)
                    .map(|entry| entry.expires_at <= now)
                    .unwrap_or(true)
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    if missing.is_empty() {
        return Ok(());
    }

    {
        let mut cache = state.settings_cache.write();
        for name in &missing {
            cache.remove(name);
        }
    }

    let mut query = sqlx::QueryBuilder::<sqlx::MySql>::new(
        "SELECT name, value FROM v2_settings WHERE name IN (",
    );
    {
        let mut separated = query.separated(", ");
        for name in &missing {
            separated.push_bind(name);
        }
    }
    query.push(") ORDER BY id DESC");
    let rows = query.build().fetch_all(&state.db).await?;

    let mut values = missing
        .iter()
        .map(|name| (name.clone(), None))
        .collect::<HashMap<String, Option<String>>>();
    let mut loaded = HashSet::new();
    for row in rows {
        let name = normalized_setting_cache_key(&row.try_get::<String, _>("name")?);
        if !loaded.insert(name.clone()) {
            continue;
        }
        values.insert(name, row.try_get::<Option<String>, _>("value")?);
    }

    let expires_at = Instant::now() + setting_cache_ttl();
    let mut cache = state.settings_cache.write();
    if cache.len() >= 256 {
        cache.retain(|_, entry| entry.expires_at > now);
        if cache.len() >= 512 {
            cache.clear();
        }
    }
    for (name, value) in values {
        cache.insert(
            name,
            CachedSetting {
                value,
                expires_at,
            },
        );
    }
    Ok(())
}

pub(crate) async fn warm_all_settings_cache(state: &AppState) -> Result<(), sqlx::Error> {
    let now = Instant::now();
    {
        let cache = state.settings_cache.read();
        if cache
            .get(ALL_SETTINGS_CACHE_KEY)
            .map(|entry| entry.expires_at > now)
            .unwrap_or(false)
        {
            return Ok(());
        }
    }

    let rows = sqlx::query(
        "SELECT name, value FROM v2_settings ORDER BY id DESC LIMIT 1024",
    )
    .fetch_all(&state.db)
    .await?;
    let expires_at = Instant::now() + setting_cache_ttl();
    let mut loaded = HashSet::new();
    let mut values = Vec::with_capacity(rows.len());
    for row in rows {
        let name = normalized_setting_cache_key(&row.try_get::<String, _>("name")?);
        if name.is_empty() || !loaded.insert(name.clone()) {
            continue;
        }
        values.push((name, row.try_get::<Option<String>, _>("value")?));
    }

    let mut cache = state.settings_cache.write();
    cache.retain(|_, entry| entry.expires_at > now);
    if cache.len() + values.len() >= 1024 {
        cache.clear();
    }
    for (name, value) in values {
        cache.insert(
            name,
            CachedSetting {
                value,
                expires_at,
            },
        );
    }
    cache.insert(
        ALL_SETTINGS_CACHE_KEY.to_string(),
        CachedSetting {
            value: None,
            expires_at,
        },
    );
    Ok(())
}

pub(crate) async fn load_cached_setting_value(
    state: &AppState,
    name: &str,
) -> Result<Option<String>, sqlx::Error> {
    let cache_key = normalized_setting_cache_key(name);
    let now = Instant::now();
    {
        let cache = state.settings_cache.read();
        if let Some(cached) = cache.get(&cache_key) {
            if cached.expires_at > now {
                return Ok(cached.value.clone());
            }
        }
    }
    state.settings_cache.write().remove(&cache_key);

    let value = sqlx::query_scalar::<_, Option<String>>(
        "SELECT value FROM v2_settings WHERE name = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(&cache_key)
    .fetch_optional(&state.db)
    .await?
    .flatten();

    {
        let mut cache = state.settings_cache.write();
        if cache.len() >= 256 {
            cache.retain(|_, entry| entry.expires_at > now);
            if cache.len() >= 512 {
                cache.clear();
            }
        }
        cache.insert(
            cache_key,
            CachedSetting {
                value: value.clone(),
                expires_at: now + setting_cache_ttl(),
            },
        );
    }

    Ok(value)
}

pub(crate) fn invalidate_setting_cache(state: &AppState, name: &str) {
    let mut cache = state.settings_cache.write();
    invalidate_setting_cache_entries(&mut cache, name);
}

fn invalidate_setting_cache_entries(
    cache: &mut HashMap<String, CachedSetting>,
    name: &str,
) {
    cache.remove(&normalized_setting_cache_key(name));
    cache.remove(ALL_SETTINGS_CACHE_KEY);
}

fn normalized_setting_cache_key(name: &str) -> String {
    name.trim().to_ascii_lowercase()
}

fn setting_cache_ttl() -> Duration {
    let seconds = env::var("SETTING_CACHE_TTL_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(|value| value.clamp(1, 30))
        .unwrap_or(3);
    Duration::from_secs(seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cached_setting(value: &str) -> CachedSetting {
        CachedSetting {
            value: Some(value.to_string()),
            expires_at: Instant::now() + Duration::from_secs(30),
        }
    }

    #[test]
    fn setting_cache_keys_are_trimmed_and_case_insensitive() {
        assert_eq!(normalized_setting_cache_key("  App_Name  "), "app_name");
    }

    #[test]
    fn invalidation_removes_the_setting_and_all_settings_marker_only() {
        let mut cache = HashMap::from([
            ("app_name".to_string(), cached_setting("notXboard")),
            ("theme".to_string(), cached_setting("Maintainable")),
            (
                ALL_SETTINGS_CACHE_KEY.to_string(),
                CachedSetting {
                    value: None,
                    expires_at: Instant::now() + Duration::from_secs(30),
                },
            ),
        ]);

        invalidate_setting_cache_entries(&mut cache, "  APP_NAME ");

        assert!(!cache.contains_key("app_name"));
        assert!(!cache.contains_key(ALL_SETTINGS_CACHE_KEY));
        assert_eq!(
            cache.get("theme").and_then(|entry| entry.value.as_deref()),
            Some("Maintainable")
        );
    }
}
