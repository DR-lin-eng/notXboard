use crate::*;

#[derive(Clone, sqlx::FromRow)]
pub struct PluginRow {
    pub id: u64,
    pub code: String,
    pub name: String,
    pub r#type: String,
    pub version: String,
    pub is_enabled: i8,
    pub config: Option<String>,
}

#[derive(Clone)]
pub struct PluginDirectoryConfig {
    pub code: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub r#type: String,
    pub config: Option<Value>,
    pub readme: Option<String>,
}

pub async fn load_installed_plugins(
    state: &AppState,
    type_filter: Option<&str>,
) -> Result<Vec<PluginRow>, sqlx::Error> {
    if let Some(type_filter) = type_filter {
        sqlx::query_as::<_, PluginRow>(
            "SELECT id, code, name, type, version, is_enabled, CAST(config AS CHAR) AS config
             FROM v2_plugins
             WHERE type = ?
             ORDER BY id ASC",
        )
        .bind(type_filter)
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as::<_, PluginRow>(
            "SELECT id, code, name, type, version, is_enabled, CAST(config AS CHAR) AS config
             FROM v2_plugins
             ORDER BY id ASC",
        )
        .fetch_all(&state.db)
        .await
    }
}

pub async fn load_installed_plugin_by_code(
    state: &AppState,
    code: &str,
) -> Result<Option<PluginRow>, sqlx::Error> {
    sqlx::query_as::<_, PluginRow>(
        "SELECT id, code, name, type, version, is_enabled, CAST(config AS CHAR) AS config
         FROM v2_plugins
         WHERE code = ?
         LIMIT 1",
    )
    .bind(code)
    .fetch_optional(&state.db)
    .await
}

pub async fn insert_plugin(
    state: &AppState,
    code: &str,
    name: &str,
    plugin_type: &str,
    version: &str,
    config: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO v2_plugins
            (code, name, type, version, is_enabled, config, installed_at, created_at, updated_at)
         VALUES (?, ?, ?, ?, 0, ?, NOW(), NOW(), NOW())",
    )
    .bind(code)
    .bind(name)
    .bind(plugin_type)
    .bind(version)
    .bind(config)
    .execute(&state.db)
    .await?;
    Ok(())
}

pub async fn delete_plugin_by_code(
    state: &AppState,
    code: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query("DELETE FROM v2_plugins WHERE code = ?")
        .bind(code)
        .execute(&state.db)
        .await?;
    Ok(result.rows_affected())
}

pub async fn update_plugin_metadata(
    state: &AppState,
    code: &str,
    name: &str,
    plugin_type: &str,
    version: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE v2_plugins
         SET name = ?, type = ?, version = ?, updated_at = NOW()
         WHERE code = ?",
    )
    .bind(name)
    .bind(plugin_type)
    .bind(version)
    .bind(code)
    .execute(&state.db)
    .await?;
    Ok(())
}
