use crate::*;

#[derive(Deserialize)]
pub(crate) struct BootstrapRequest {
    #[serde(default)]
    app_name: Option<String>,
    #[serde(default)]
    app_url: Option<String>,
    #[serde(default)]
    admin_email: Option<String>,
    #[serde(default)]
    admin_password: Option<String>,
}

const FULL_SCHEMA_SQL: &str = include_str!("../resources/schema/notxboard.full.sql");
const FULL_MIGRATIONS_LIST: &str = include_str!("../resources/schema/notxboard.migrations.txt");
const PROTECTED_PLUGIN_CODES: &[&str] = &[
    "epay",
    "alipay_f2f",
    "btcpay",
    "coinbase",
    "coin_payments",
    "mgate",
    "telegram",
];

pub(crate) async fn bootstrap_status(
    State(state): State<Arc<AppState>>,
) -> Response<Body> {
    match build_bootstrap_status_response(&state).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub(crate) async fn bootstrap_minimal(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Body,
) -> Response<Body> {
    if let Err(response) = require_bootstrap_token(&headers) {
        return response;
    }
    let lock = match acquire_bootstrap_lock(&state).await {
        Ok(lock) => lock,
        Err(response) => return response,
    };
    let result = build_bootstrap_minimal_response(&state, body).await;
    let _ = lock.commit().await;
    match result {
        Ok(response) => response,
        Err(response) => response,
    }
}

pub(crate) async fn bootstrap_full(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: Body,
) -> Response<Body> {
    if let Err(response) = require_bootstrap_token(&headers) {
        return response;
    }
    let lock = match acquire_bootstrap_lock(&state).await {
        Ok(lock) => lock,
        Err(response) => return response,
    };
    let result = build_bootstrap_full_response(&state, body).await;
    let _ = lock.commit().await;
    match result {
        Ok(response) => response,
        Err(response) => response,
    }
}

fn require_bootstrap_token(headers: &HeaderMap) -> Result<(), Response<Body>> {
    let expected = std::env::var("BOOTSTRAP_TOKEN")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| value.len() >= 24)
        .ok_or_else(|| {
            json_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "Bootstrap is disabled until BOOTSTRAP_TOKEN is configured",
            )
        })?;
    let supplied = headers
        .get("x-bootstrap-token")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .unwrap_or_default();
    if !crate::secure_compare_support::constant_time_eq_str(supplied, &expected) {
        return Err(json_error(StatusCode::NOT_FOUND, "Not found"));
    }
    Ok(())
}

async fn acquire_bootstrap_lock(
    state: &AppState,
) -> Result<sqlx::Transaction<'_, sqlx::MySql>, Response<Body>> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS rust_bootstrap_lock (
            id TINYINT UNSIGNED NOT NULL PRIMARY KEY,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
         ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci"
    )
    .execute(&state.db)
    .await
    .map_err(internal_error)?;
    sqlx::query("INSERT IGNORE INTO rust_bootstrap_lock (id) VALUES (1)")
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    sqlx::query("SET innodb_lock_wait_timeout = 15")
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    sqlx::query("SELECT id FROM rust_bootstrap_lock WHERE id = 1 FOR UPDATE")
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| {
            json_error(
                StatusCode::CONFLICT,
                "Another bootstrap operation is already running",
            )
        })?;
    Ok(tx)
}

async fn build_bootstrap_status_response(
    state: &AppState,
) -> Result<Response<Body>, Response<Body>> {
    let status = bootstrap_state(state).await.map_err(internal_error)?;
    Ok(json_value_response(json!({
        "installed": status.installed,
        "tables_ready": status.tables_ready,
        "admin_exists": status.admin_exists,
        "settings_count": status.settings_count,
    })))
}

async fn build_bootstrap_minimal_response(
    state: &AppState,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    ensure_bootstrap_install_open(state).await?;
    let payload = parse_json_body(body).await?;
    let request: BootstrapRequest = serde_json::from_value(payload)
        .map_err(|_| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let admin_email = request
        .admin_email
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "admin_email is required"))?;
    let admin_password = request
        .admin_password
        .as_deref()
        .filter(|value| value.len() >= 8)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "admin_password must be at least 8 chars"))?;

    ensure_bootstrap_tables(state).await.map_err(|err| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, &err)
    })?;
    ensure_bootstrap_migration_markers(state).await.map_err(|err| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, &err)
    })?;

    let app_name = request
        .app_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("XBoard");
    let app_url = request
        .app_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("");

    upsert_bootstrap_setting(state, "app_name", app_name).await.map_err(|err| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, &err)
    })?;
    upsert_bootstrap_setting(state, "oauth_linux_do_enable", "0").await.map_err(|err| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, &err)
    })?;
    if !app_url.is_empty() {
        upsert_bootstrap_setting(state, "app_url", app_url).await.map_err(|err| {
            json_error(StatusCode::INTERNAL_SERVER_ERROR, &err)
        })?;
    }

    let secure_path = first_non_empty(&[
        get_setting_string(state, "secure_path", "").await,
        get_setting_string(state, "frontend_admin_path", "").await,
        String::new(),
    ]);
    let resolved_secure_path = if !is_valid_secure_admin_path(&secure_path) {
        let generated = random_alnum(10);
        upsert_bootstrap_setting(state, "secure_path", &generated).await.map_err(|err| {
            json_error(StatusCode::INTERNAL_SERVER_ERROR, &err)
        })?;
        upsert_bootstrap_setting(state, "frontend_admin_path", &generated).await.map_err(|err| {
            json_error(StatusCode::INTERNAL_SERVER_ERROR, &err)
        })?;
        generated
    } else {
        secure_path
    };

    let admin_exists = sqlx::query_scalar::<_, i64>(
        "SELECT EXISTS(SELECT 1 FROM v2_user WHERE is_admin = 1 OR is_super_admin = 1) AS admin_exists",
    )
    .fetch_one(&state.db)
    .await
    .map_err(internal_error)?;

    if admin_exists == 0 {
        create_bootstrap_admin(state, admin_email, admin_password).await?;
    }

    seed_group_limits(state)
        .await
        .map_err(|err| json_error(StatusCode::INTERNAL_SERVER_ERROR, &err))?;
    ensure_default_plugins(state)
        .await
        .map_err(|err| json_error(StatusCode::INTERNAL_SERVER_ERROR, &err))?;
    ensure_missing_api_keys(state)
        .await
        .map_err(|err| json_error(StatusCode::INTERNAL_SERVER_ERROR, &err))?;

    upsert_bootstrap_setting(state, "bootstrap_minimal_ready", "1").await.map_err(|err| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, &err)
    })?;
    upsert_bootstrap_setting(state, "bootstrap_mode", "rust-minimal").await.map_err(|err| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, &err)
    })?;

    Ok(json_value_response(json!({
        "installed": true,
        "mode": "rust-minimal",
        "secure_path": resolved_secure_path,
        "admin_email": admin_email,
    })))
}

async fn build_bootstrap_full_response(
    state: &AppState,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    ensure_bootstrap_install_open(state).await?;
    let payload = parse_json_body(body).await?;
    let request: BootstrapRequest = serde_json::from_value(payload)
        .map_err(|_| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))?;

    let admin_email = request
        .admin_email
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "admin_email is required"))?;
    let admin_password = request
        .admin_password
        .as_deref()
        .filter(|value| value.len() >= 8)
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "admin_password must be at least 8 chars"))?;

    apply_full_schema_baseline(state)
        .await
        .map_err(|err| json_error(StatusCode::INTERNAL_SERVER_ERROR, &err))?;
    ensure_full_migration_markers(state)
        .await
        .map_err(|err| json_error(StatusCode::INTERNAL_SERVER_ERROR, &err))?;

    let app_name = request
        .app_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("XBoard");
    let app_url = request
        .app_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("");

    upsert_bootstrap_setting(state, "app_name", app_name).await.map_err(|err| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, &err)
    })?;
    upsert_bootstrap_setting(state, "oauth_linux_do_enable", "0").await.map_err(|err| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, &err)
    })?;
    if !app_url.is_empty() {
        upsert_bootstrap_setting(state, "app_url", app_url).await.map_err(|err| {
            json_error(StatusCode::INTERNAL_SERVER_ERROR, &err)
        })?;
    }

    let secure_path = first_non_empty(&[
        get_setting_string(state, "secure_path", "").await,
        get_setting_string(state, "frontend_admin_path", "").await,
        String::new(),
    ]);
    let resolved_secure_path = if !is_valid_secure_admin_path(&secure_path) {
        let generated = random_alnum(10);
        upsert_bootstrap_setting(state, "secure_path", &generated).await.map_err(|err| {
            json_error(StatusCode::INTERNAL_SERVER_ERROR, &err)
        })?;
        upsert_bootstrap_setting(state, "frontend_admin_path", &generated).await.map_err(|err| {
            json_error(StatusCode::INTERNAL_SERVER_ERROR, &err)
        })?;
        generated
    } else {
        secure_path
    };

    let admin_exists = sqlx::query_scalar::<_, i64>(
        "SELECT EXISTS(SELECT 1 FROM v2_user WHERE is_admin = 1 OR is_super_admin = 1) AS admin_exists",
    )
    .fetch_one(&state.db)
    .await
    .map_err(internal_error)?;

    if admin_exists == 0 {
        create_bootstrap_admin(state, admin_email, admin_password).await?;
    }

    seed_group_limits(state)
        .await
        .map_err(|err| json_error(StatusCode::INTERNAL_SERVER_ERROR, &err))?;
    ensure_default_plugins(state)
        .await
        .map_err(|err| json_error(StatusCode::INTERNAL_SERVER_ERROR, &err))?;
    ensure_missing_api_keys(state)
        .await
        .map_err(|err| json_error(StatusCode::INTERNAL_SERVER_ERROR, &err))?;

    upsert_bootstrap_setting(state, "bootstrap_minimal_ready", "1").await.map_err(|err| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, &err)
    })?;
    upsert_bootstrap_setting(state, "bootstrap_mode", "rust-full-schema").await.map_err(|err| {
        json_error(StatusCode::INTERNAL_SERVER_ERROR, &err)
    })?;

    Ok(json_value_response(json!({
        "installed": true,
        "mode": "rust-full-schema",
        "secure_path": resolved_secure_path,
        "admin_email": admin_email,
    })))
}

struct BootstrapState {
    installed: bool,
    tables_ready: bool,
    admin_exists: bool,
    settings_count: i64,
}

async fn bootstrap_state(state: &AppState) -> Result<BootstrapState, sqlx::Error> {
    let tables_ready = required_bootstrap_tables_ready(state).await?;
    let settings_count = if tables_ready {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM v2_settings")
            .fetch_one(&state.db)
            .await?
    } else {
        0
    };
    let installed_flag = if tables_ready {
        sqlx::query_scalar::<_, Option<String>>(
            "SELECT value FROM v2_settings WHERE name = 'bootstrap_minimal_ready' ORDER BY id DESC LIMIT 1",
        )
        .fetch_optional(&state.db)
        .await?
        .flatten()
        .unwrap_or_default()
    } else {
        String::new()
    };
    let admin_exists = if tables_ready {
        sqlx::query_scalar::<_, i64>(
            "SELECT EXISTS(SELECT 1 FROM v2_user WHERE is_admin = 1 OR is_super_admin = 1) AS admin_exists",
        )
        .fetch_one(&state.db)
        .await?
            == 1
    } else {
        false
    };

    Ok(BootstrapState {
        installed: matches!(installed_flag.trim(), "1" | "true" | "TRUE" | "yes" | "on"),
        tables_ready,
        admin_exists,
        settings_count,
    })
}

async fn ensure_bootstrap_install_open(state: &AppState) -> Result<(), Response<Body>> {
    let status = bootstrap_state(state).await.map_err(internal_error)?;
    if status.installed || status.admin_exists {
        return Err(json_status_response(
            StatusCode::CONFLICT,
            json!({
                "message": "Application is already installed"
            }),
        ));
    }
    Ok(())
}

async fn required_bootstrap_tables_ready(state: &AppState) -> Result<bool, sqlx::Error> {
    let query = "
        SELECT
            EXISTS(
                SELECT 1
                FROM information_schema.tables
                WHERE table_schema = DATABASE()
                  AND table_name = ?
                  AND table_type IN ('BASE TABLE', 'SYSTEM VERSIONED')
            ) AS table_exists
    ";
    for table_name in ["migrations", "v2_settings", "v2_user"] {
        let exists = sqlx::query_scalar::<_, i64>(query)
            .bind(table_name)
            .fetch_one(&state.db)
            .await?;
        if exists != 1 {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(crate) async fn business_tables_ready(state: &AppState) -> Result<bool, sqlx::Error> {
    let query = "
        SELECT
            EXISTS(
                SELECT 1
                FROM information_schema.tables
                WHERE table_schema = DATABASE()
                  AND table_name = ?
                  AND table_type IN ('BASE TABLE', 'SYSTEM VERSIONED')
            ) AS table_exists
    ";
    for table_name in ["v2_plan", "v2_order", "server_nodes", "v2_ticket"] {
        let exists = sqlx::query_scalar::<_, i64>(query)
            .bind(table_name)
            .fetch_one(&state.db)
            .await?;
        if exists != 1 {
            return Ok(false);
        }
    }
    Ok(true)
}

async fn ensure_bootstrap_tables(state: &AppState) -> Result<(), String> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS migrations (
            id INT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
            migration VARCHAR(255) NOT NULL,
            batch INT NOT NULL
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
    )
    .execute(&state.db)
    .await
    .map_err(|err| format!("create migrations failed: {err}"))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS v2_settings (
            id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
            `group` VARCHAR(255) NULL,
            `type` VARCHAR(255) NULL,
            name VARCHAR(255) NOT NULL,
            value MEDIUMTEXT NULL,
            created_at TIMESTAMP NULL DEFAULT NULL,
            updated_at TIMESTAMP NULL DEFAULT NULL,
            UNIQUE KEY uniq_v2_settings_name (name),
            KEY idx_setting_name (name)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
    )
    .execute(&state.db)
    .await
    .map_err(|err| format!("create v2_settings failed: {err}"))?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS v2_user (
            id INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
            invite_user_id INT NULL,
            telegram_id BIGINT NULL,
            email VARCHAR(64) NOT NULL,
            password VARCHAR(255) NOT NULL,
            password_algo CHAR(10) NULL,
            password_salt CHAR(10) NULL,
            balance INT NOT NULL DEFAULT 0,
            discount INT NULL,
            commission_type TINYINT NOT NULL DEFAULT 0,
            commission_rate INT NULL,
            commission_balance INT NOT NULL DEFAULT 0,
            t INT NOT NULL DEFAULT 0,
            u BIGINT NOT NULL DEFAULT 0,
            d BIGINT NOT NULL DEFAULT 0,
            transfer_enable BIGINT NOT NULL DEFAULT 0,
            banned TINYINT(1) NOT NULL DEFAULT 0,
            is_admin TINYINT(1) NOT NULL DEFAULT 0,
            is_super_admin TINYINT(1) NOT NULL DEFAULT 0,
            is_staff TINYINT(1) NOT NULL DEFAULT 0,
            is_silenced TINYINT(1) NOT NULL DEFAULT 0,
            trust_level INT NOT NULL DEFAULT 0,
            last_login_at INT NULL,
            last_login_ip INT NULL,
            uuid CHAR(36) NOT NULL,
            `token` CHAR(32) NOT NULL,
            subscribe_path VARCHAR(64) NULL,
            subscribe_key VARCHAR(64) NULL,
            subscribe_salt VARCHAR(64) NULL,
            group_id INT NULL,
            plan_id INT NULL,
            speed_limit INT NULL,
            device_limit INT NULL,
            concurrent_ip_limit INT NULL,
            remind_expire TINYINT NULL DEFAULT 1,
            remind_traffic TINYINT NULL DEFAULT 1,
            expired_at BIGINT NULL DEFAULT 0,
            remarks TEXT NULL,
            api_key VARCHAR(255) NULL,
            linux_do_id BIGINT NULL,
            linux_do_username VARCHAR(255) NULL,
            linux_do_name VARCHAR(255) NULL,
            linux_do_avatar TEXT NULL,
            created_at INT NOT NULL DEFAULT 0,
            updated_at INT NOT NULL DEFAULT 0,
            UNIQUE KEY email (email),
            UNIQUE KEY uuid (uuid),
            KEY idx_user_token (`token`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
    )
    .execute(&state.db)
    .await
    .map_err(|err| format!("create v2_user failed: {err}"))?;

    Ok(())
}

async fn ensure_bootstrap_migration_markers(state: &AppState) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO migrations (migration, batch)
         SELECT ?, 1
         WHERE NOT EXISTS (
             SELECT 1 FROM migrations WHERE migration = ?
         )",
    )
    .bind("2023_08_14_221234_create_v2_settings_table")
    .bind("2023_08_14_221234_create_v2_settings_table")
    .execute(&state.db)
    .await
    .map_err(|err| format!("insert bootstrap migration marker failed: {err}"))?;

    Ok(())
}

async fn apply_full_schema_baseline(state: &AppState) -> Result<(), String> {
    let mut cleaned = String::new();
    for line in FULL_SCHEMA_SQL.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("/*!") || trimmed.starts_with("--") || trimmed.is_empty() {
            continue;
        }
        cleaned.push_str(line);
        cleaned.push('\n');
    }

    let statements = cleaned
        .split(";\n")
        .map(str::trim)
        .filter(|stmt| !stmt.is_empty())
        .filter(|stmt| !stmt.starts_with("DROP TABLE IF EXISTS "))
        .filter(|stmt| !stmt.eq_ignore_ascii_case("LOCK TABLES") && !stmt.starts_with("LOCK TABLES"))
        .filter(|stmt| !stmt.eq_ignore_ascii_case("UNLOCK TABLES"))
        .map(str::to_string)
        .collect::<Vec<_>>();

    let ordered_statements = order_schema_statements(statements)?;

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|err| format!("begin full schema tx failed: {err}"))?;
    sqlx::query("SET FOREIGN_KEY_CHECKS = 0")
        .execute(&mut *tx)
        .await
        .map_err(|err| format!("disable foreign key checks failed: {err}"))?;
    sqlx::query("SET UNIQUE_CHECKS = 0")
        .execute(&mut *tx)
        .await
        .map_err(|err| format!("disable unique checks failed: {err}"))?;
    for statement in ordered_statements {
        sqlx::query(&statement)
            .execute(&mut *tx)
            .await
            .map_err(|err| format!("apply full schema statement failed: {err}; sql={statement}"))?;
    }
    sqlx::query("SET UNIQUE_CHECKS = 1")
        .execute(&mut *tx)
        .await
        .map_err(|err| format!("restore unique checks failed: {err}"))?;
    sqlx::query("SET FOREIGN_KEY_CHECKS = 1")
        .execute(&mut *tx)
        .await
        .map_err(|err| format!("restore foreign key checks failed: {err}"))?;
    tx.commit()
        .await
        .map_err(|err| format!("commit full schema tx failed: {err}"))?;
    Ok(())
}

fn order_schema_statements(statements: Vec<String>) -> Result<Vec<String>, String> {
    let create_prefix = "CREATE TABLE `";
    let mut remaining = statements;
    let mut ordered = Vec::new();
    let mut created_tables = std::collections::HashSet::<String>::new();

    loop {
        if remaining.is_empty() {
            break;
        }

        let mut progress = false;
        let mut next_round = Vec::new();

        for statement in remaining {
            if !statement.starts_with(create_prefix) {
                ordered.push(statement);
                progress = true;
                continue;
            }

            let Some(table_name_end) = statement[create_prefix.len()..].find('`') else {
                return Err(format!("parse create table name failed: {statement}"));
            };
            let table_name = statement[create_prefix.len()..create_prefix.len() + table_name_end].to_string();
            let dependencies = extract_fk_dependencies(&statement);
            let ready = dependencies
                .iter()
                .all(|dependency| dependency == &table_name || created_tables.contains(dependency));

            if ready {
                created_tables.insert(table_name);
                ordered.push(statement);
                progress = true;
            } else {
                next_round.push(statement);
            }
        }

        if !progress {
            let unresolved = next_round
                .iter()
                .map(|stmt| {
                    let deps = extract_fk_dependencies(stmt);
                    format!("deps={deps:?} sql={stmt}")
                })
                .collect::<Vec<_>>()
                .join("\n");
            return Err(format!("order schema statements failed, unresolved dependencies:\n{unresolved}"));
        }

        remaining = next_round;
    }

    Ok(ordered)
}

fn extract_fk_dependencies(statement: &str) -> Vec<String> {
    let regex = regex::Regex::new(r"REFERENCES `([^`]+)`").unwrap();
    regex
        .captures_iter(statement)
        .filter_map(|capture| capture.get(1).map(|value| value.as_str().to_string()))
        .collect()
}

async fn ensure_full_migration_markers(state: &AppState) -> Result<(), String> {
    for migration in FULL_MIGRATIONS_LIST
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.trim_end_matches(".php"))
    {
        sqlx::query(
            "INSERT INTO migrations (migration, batch)
             SELECT ?, 1
             WHERE NOT EXISTS (
                 SELECT 1 FROM migrations WHERE migration = ?
             )",
        )
        .bind(migration)
        .bind(migration)
        .execute(&state.db)
        .await
        .map_err(|err| format!("insert full migration marker failed for {migration}: {err}"))?;
    }
    Ok(())
}

async fn upsert_bootstrap_setting(
    state: &AppState,
    name: &str,
    value: &str,
) -> Result<(), String> {
    upsert_setting_string_with_metadata(state, Some("bootstrap"), Some("text"), name, value)
        .await
        .map_err(|err| format!("upsert setting {name} failed: {err}"))?;
    Ok(())
}

async fn seed_group_limits(state: &AppState) -> Result<(), String> {
    let rows = [
        (0_i64, 50_i64, 100_i64, 2_i64, 5_i64),
        (1, 100, 200, 3, 10),
        (2, 200, 500, 5, 20),
        (3, 500, 1000, 8, 50),
        (4, 1000, 2000, 15, 100),
    ];

    for (trust_level, speed_limit_up, speed_limit_down, device_limit, connection_limit) in rows {
        sqlx::query(
            "INSERT INTO user_group_limits
                (trust_level, speed_limit_up, speed_limit_down, device_limit, connection_limit, created_at, updated_at)
             VALUES
                (?, ?, ?, ?, ?, NOW(), NOW())
             ON DUPLICATE KEY UPDATE
                speed_limit_up = VALUES(speed_limit_up),
                speed_limit_down = VALUES(speed_limit_down),
                device_limit = VALUES(device_limit),
                connection_limit = VALUES(connection_limit),
                updated_at = NOW()",
        )
        .bind(trust_level)
        .bind(speed_limit_up)
        .bind(speed_limit_down)
        .bind(device_limit)
        .bind(connection_limit)
        .execute(&state.db)
        .await
        .map_err(|err| format!("seed user_group_limits failed for trust_level={trust_level}: {err}"))?;
    }

    Ok(())
}

async fn ensure_default_plugins(state: &AppState) -> Result<(), String> {
    for plugin_code in PROTECTED_PLUGIN_CODES {
        let plugin = crate::builtin_plugin_support::builtin_plugin_config(plugin_code)
            .ok_or_else(|| format!("builtin plugin config missing for {plugin_code}"))?;
        let default_config = extract_plugin_default_config(plugin.config.as_ref());
        let config_json = serde_json::to_string(&default_config)
            .map_err(|err| format!("serialize default plugin config failed for {plugin_code}: {err}"))?;

        sqlx::query(
            "INSERT INTO v2_plugins
                (code, name, type, version, is_enabled, config, installed_at, created_at, updated_at)
             VALUES
                (?, ?, ?, ?, 1, ?, NOW(), NOW(), NOW())
             ON DUPLICATE KEY UPDATE
                name = VALUES(name),
                type = VALUES(type),
                version = VALUES(version),
                is_enabled = 1,
                config = VALUES(config),
                updated_at = NOW()",
        )
        .bind(plugin_code)
        .bind(&plugin.name)
        .bind(&plugin.plugin_type)
        .bind(&plugin.version)
        .bind(config_json)
        .execute(&state.db)
        .await
        .map_err(|err| format!("upsert default plugin failed for {plugin_code}: {err}"))?;
    }

    Ok(())
}

async fn ensure_missing_api_keys(state: &AppState) -> Result<(), String> {
    let users = sqlx::query_scalar::<_, i64>("SELECT id FROM v2_user WHERE api_key IS NULL OR api_key = ''")
        .fetch_all(&state.db)
        .await
        .map_err(|err| format!("load users without api key failed: {err}"))?;
    fill_missing_api_keys_for_users(state, &users)
        .await
        .map_err(|err| format!("assign missing api keys failed: {err}"))?;

    Ok(())
}

fn extract_plugin_default_config(config: Option<&Value>) -> Value {
    let Some(config) = config.and_then(Value::as_object) else {
        return Value::Object(Map::new());
    };

    let mut defaults = Map::new();
    for (key, value) in config {
        let default_value = value
            .as_object()
            .and_then(|object| object.get("default"))
            .cloned()
            .unwrap_or(Value::Null);
        defaults.insert(key.clone(), default_value);
    }
    Value::Object(defaults)
}

async fn create_bootstrap_admin(
    state: &AppState,
    email: &str,
    password: &str,
) -> Result<Response<Body>, Response<Body>> {
    let hashed = bcrypt::hash(password, 10)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "hash admin password failed"))?;
    let now = Utc::now().timestamp() as i64;
    let uuid = Uuid::new_v4().to_string();
    let token = random_hex(32);
    let subscribe_path = random_alnum(10);
    let subscribe_key = random_alnum(8);
    let subscribe_salt = random_alnum(6);

    sqlx::query(
        "INSERT INTO v2_user
            (email, password, uuid, `token`, subscribe_path, subscribe_key, subscribe_salt,
             is_admin, is_super_admin, trust_level, banned, transfer_enable, created_at, updated_at)
         VALUES
            (?, ?, ?, ?, ?, ?, ?, 1, 1, 4, 0, 0, ?, ?)",
    )
    .bind(email)
    .bind(hashed)
    .bind(uuid)
    .bind(token)
    .bind(subscribe_path)
    .bind(subscribe_key)
    .bind(subscribe_salt)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "create bootstrap admin failed"))?;

    Ok(Response::new(Body::empty()))
}

fn random_hex(len: usize) -> String {
    random_from_alphabet(len, b"0123456789abcdef")
}

fn random_alnum(len: usize) -> String {
    random_from_alphabet(len, b"abcdefghijklmnopqrstuvwxyz0123456789")
}

fn random_from_alphabet(len: usize, alphabet: &[u8]) -> String {
    let mut bytes = vec![0_u8; len];
    if let Ok(mut file) = std::fs::File::open("/dev/urandom") {
        let _ = file.read_exact(&mut bytes);
    } else {
        for (idx, item) in bytes.iter_mut().enumerate() {
            *item = (idx % alphabet.len()) as u8;
        }
    }

    bytes
        .into_iter()
        .map(|value| alphabet[(value as usize) % alphabet.len()] as char)
        .collect()
}
