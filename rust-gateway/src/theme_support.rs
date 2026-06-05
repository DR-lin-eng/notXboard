use crate::*;

pub(crate) const DEFAULT_THEME_NAME: &str = crate::builtin_theme_support::MAINTAINABLE_THEME_NAME;
pub(crate) const PORTAL_THEME_NAME: &str = crate::builtin_theme_support::PORTAL_THEME_NAME;
const ACTIVE_THEME_KEYS: [&str; 2] = ["frontend_theme", "current_theme"];
const CURRENT_THEME_KEYS: [&str; 2] = ["current_theme", "frontend_theme"];
const PORTAL_LEGACY_KEYS: [(&str, &str); 4] = [
    ("frontend_theme_color", "theme_color"),
    ("frontend_background_url", "background_url"),
    ("frontend_theme_sidebar", "theme_sidebar"),
    ("frontend_theme_header", "theme_header"),
];

#[derive(Clone)]
pub(crate) struct ThemeDefinition {
    name: String,
    title: Option<String>,
    description: String,
    version: String,
    author: Option<String>,
    image: Option<String>,
    configs: Vec<ThemeConfigField>,
    is_system: bool,
    can_delete: bool,
}

#[derive(Clone)]
pub(crate) struct ThemeConfigField {
    label: String,
    placeholder: String,
    field_name: String,
    field_type: String,
    default_value: Option<String>,
    select_options: Vec<ThemeSelectOption>,
}

#[derive(Clone)]
pub(crate) struct ThemeSelectOption {
    value: String,
    label: String,
}

impl ThemeDefinition {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn version(&self) -> &str {
        &self.version
    }

    pub(crate) fn configs(&self) -> &[ThemeConfigField] {
        &self.configs
    }

    pub(crate) fn default_config_map(&self) -> Map<String, Value> {
        let mut map = Map::new();
        for field in &self.configs {
            map.insert(
                field.field_name.clone(),
                field
                    .default_value
                    .clone()
                    .map(Value::String)
                    .unwrap_or_else(|| Value::String(String::new())),
            );
        }
        map
    }

    pub(crate) fn to_catalog_value(&self, current_theme: &str) -> Value {
        let mut object = Map::new();
        object.insert("name".to_string(), Value::String(self.name.clone()));
        if let Some(title) = &self.title {
            object.insert("title".to_string(), Value::String(title.clone()));
        }
        object.insert(
            "description".to_string(),
            Value::String(self.description.clone()),
        );
        object.insert("version".to_string(), Value::String(self.version.clone()));
        if let Some(author) = &self.author {
            object.insert("author".to_string(), Value::String(author.clone()));
        }
        if let Some(image) = &self.image {
            object.insert("images".to_string(), Value::String(image.clone()));
        }
        object.insert(
            "configs".to_string(),
            Value::Array(self.configs.iter().map(ThemeConfigField::to_value).collect()),
        );
        object.insert("is_system".to_string(), Value::Bool(self.is_system));
        object.insert("can_delete".to_string(), Value::Bool(self.can_delete));
        object.insert(
            "is_active".to_string(),
            Value::Bool(self.name.eq_ignore_ascii_case(current_theme)),
        );
        Value::Object(object)
    }
}

impl ThemeConfigField {
    pub(crate) fn field_name(&self) -> &str {
        &self.field_name
    }

    fn to_value(&self) -> Value {
        let mut object = Map::new();
        object.insert("label".to_string(), Value::String(self.label.clone()));
        object.insert(
            "placeholder".to_string(),
            Value::String(self.placeholder.clone()),
        );
        object.insert(
            "field_name".to_string(),
            Value::String(self.field_name.clone()),
        );
        object.insert(
            "field_type".to_string(),
            Value::String(self.field_type.clone()),
        );
        if let Some(default_value) = &self.default_value {
            object.insert(
                "default_value".to_string(),
                Value::String(default_value.clone()),
            );
        }
        if !self.select_options.is_empty() {
            let select_options = self
                .select_options
                .iter()
                .map(|option| (option.value.clone(), Value::String(option.label.clone())))
                .collect::<Map<String, Value>>();
            object.insert("select_options".to_string(), Value::Object(select_options));
        }
        Value::Object(object)
    }
}

pub(crate) async fn load_active_theme_name(state: &AppState) -> String {
    load_first_non_empty_string(state, &ACTIVE_THEME_KEYS, DEFAULT_THEME_NAME).await
}

pub(crate) async fn load_current_theme_name(state: &AppState) -> String {
    load_first_non_empty_string(state, &CURRENT_THEME_KEYS, DEFAULT_THEME_NAME).await
}

pub(crate) async fn resolve_runtime_theme_name(state: &AppState) -> String {
    let configured = load_current_theme_name(state).await;
    if theme_template_exists(&configured) {
        return configured;
    }
    if theme_template_exists(DEFAULT_THEME_NAME) {
        return DEFAULT_THEME_NAME.to_string();
    }
    if theme_template_exists(PORTAL_THEME_NAME) {
        return PORTAL_THEME_NAME.to_string();
    }
    configured
}

pub(crate) fn build_theme_catalog(current_theme: &str) -> Vec<Value> {
    scan_theme_definitions(current_theme)
        .into_iter()
        .map(|theme| theme.to_catalog_value(current_theme))
        .collect()
}

pub(crate) fn find_theme(name: &str) -> Option<ThemeDefinition> {
    scan_theme_definitions("")
        .into_iter()
        .find(|theme| theme.name.eq_ignore_ascii_case(name))
}

pub(crate) fn is_system_theme(name: &str) -> bool {
    crate::builtin_theme_support::is_builtin_theme(name)
}

pub(crate) fn load_theme_template(name: &str) -> Option<String> {
    if let Some(template) = crate::builtin_theme_support::builtin_theme_template(name) {
        return Some(template.to_string());
    }
    std::fs::read_to_string(theme_template_path(name)?).ok()
}

fn theme_template_exists(name: &str) -> bool {
    crate::builtin_theme_support::builtin_theme_template(name).is_some() || theme_template_path(name).is_some()
}

fn theme_template_path(name: &str) -> Option<std::path::PathBuf> {
    let source = theme_source_path(name)?;
    let path = source.join("dashboard.blade.php");
    path.is_file().then_some(path)
}

pub(crate) fn theme_asset_version(name: &str, default_version: &str) -> String {
    let candidates = [
        "app.js",
        "app.css",
        "assets/umi.js",
        "assets/umi.css",
        "assets/vendors.async.js",
        "assets/components.async.js",
        "assets/components.chunk.css",
    ];
    let public_root = crate::runtime_paths::public_path(std::path::Path::new("theme").join(name));
    let source_root = theme_source_path(name);
    let newest_public = candidates
        .iter()
        .map(|relative| public_root.join(relative))
        .filter_map(|path| std::fs::metadata(path).ok())
        .filter_map(|meta| meta.modified().ok())
        .filter_map(|mtime| mtime.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
        .max();
    let newest_source = source_root
        .map(|root| {
            candidates
                .iter()
                .map(|relative| root.join(relative))
                .filter_map(|path| std::fs::metadata(path).ok())
                .filter_map(|meta| meta.modified().ok())
                .filter_map(|mtime| mtime.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs())
                .max()
        })
        .flatten();

    newest_public
        .into_iter()
        .chain(newest_source)
        .max()
        .map(|value| value.to_string())
        .unwrap_or_else(|| default_version.to_string())
}

pub(crate) fn theme_has_asset(name: &str, relative_path: &str) -> bool {
    theme_asset_path(name, relative_path).is_some()
}

pub(crate) fn resolve_theme_asset_path(raw_path: &str) -> Option<std::path::PathBuf> {
    let safe_path = normalize_relative_path(raw_path)?;
    let public_root = crate::runtime_paths::public_path("theme");
    if let Some(public_path) = canonical_file_under(&public_root, &safe_path) {
        return Some(public_path);
    }

    let mut components = safe_path.components();
    let theme_component = match components.next()? {
        std::path::Component::Normal(value) => value.to_string_lossy().to_string(),
        _ => return None,
    };
    if !is_valid_theme_name(&theme_component) {
        return None;
    }

    let remainder = components.as_path();
    if remainder.as_os_str().is_empty() {
        return None;
    }

    let source_root = theme_source_path(&theme_component)?;
    canonical_file_under(&source_root, remainder)
}

pub(crate) async fn load_theme_config(
    state: &AppState,
    theme: &ThemeDefinition,
) -> Map<String, Value> {
    let mut merged = theme.default_config_map();
    if theme.name().eq_ignore_ascii_case(PORTAL_THEME_NAME) {
        overlay_portal_legacy_settings(state, &mut merged).await;
    }
    if let Some(saved) = load_setting_object(state, &theme_setting_key(theme.name())).await {
        merge_values(&mut merged, saved);
    }
    merged
}

pub(crate) async fn save_theme_config(
    state: &AppState,
    theme: &ThemeDefinition,
    payload: Map<String, Value>,
) -> Result<Map<String, Value>, Response<Body>> {
    let mut merged = load_theme_config(state, theme).await;
    for field in theme.configs() {
        if let Some(value) = payload.get(field.field_name()) {
            merged.insert(field.field_name().to_string(), value.clone());
        }
    }
    save_setting_value(
        state,
        &theme_setting_key(theme.name()),
        &Value::Object(merged.clone()),
    )
    .await?;
    Ok(merged)
}

pub(crate) async fn save_theme_config_value(
    state: &AppState,
    key: &str,
    value: Value,
) -> Result<(), Response<Body>> {
    save_setting_value(state, key, &value).await
}

fn scan_theme_definitions(current_theme: &str) -> Vec<ThemeDefinition> {
    let mut themes = builtin_theme_definitions();
    let mut seen = themes
        .iter()
        .map(|theme| theme.name.to_ascii_lowercase())
        .collect::<std::collections::HashSet<_>>();
    themes.extend(
        scan_theme_root(crate::runtime_paths::themes_path(""), true, current_theme)
            .into_iter()
            .filter(|theme| seen.insert(theme.name.to_ascii_lowercase())),
    );
    themes.extend(
        scan_theme_root(crate::runtime_paths::state_path("theme"), false, current_theme)
            .into_iter()
            .filter(|theme| seen.insert(theme.name.to_ascii_lowercase())),
    );
    themes.sort_by(|a, b| a.name.cmp(&b.name));
    themes
}

fn builtin_theme_definitions() -> Vec<ThemeDefinition> {
    crate::builtin_theme_support::builtin_theme_configs()
        .into_iter()
        .map(|config| ThemeDefinition {
            name: config.name.clone(),
            title: config.title,
            description: config.description,
            version: config.version,
            author: config.author,
            image: config.image,
            configs: config.configs.iter().filter_map(parse_theme_config_field).collect(),
            is_system: true,
            can_delete: false,
        })
        .collect()
}

fn scan_theme_root(root: std::path::PathBuf, is_system: bool, current_theme: &str) -> Vec<ThemeDefinition> {
    let Ok(entries) = std::fs::read_dir(&root) else {
        return Vec::new();
    };

    let mut themes = Vec::new();
    for entry in entries.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let config_path = dir.join("config.json");
        let dashboard_path = dir.join("dashboard.blade.php");
        if !config_path.exists() || !dashboard_path.exists() {
            continue;
        }
        let Ok(raw) = std::fs::read_to_string(&config_path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(&raw) else {
            continue;
        };
        let Some(object) = value.as_object() else {
            continue;
        };
        let name = object
            .get("name")
            .and_then(Value::as_str)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| dir.file_name().unwrap_or_default().to_string_lossy().to_string());
        let configs = object
            .get("configs")
            .and_then(Value::as_array)
            .map(|items| {
                items.iter()
                    .filter_map(parse_theme_config_field)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        themes.push(ThemeDefinition {
            name: name.clone(),
            title: object
                .get("title")
                .and_then(Value::as_str)
                .map(|value| value.to_string()),
            description: object
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            version: object
                .get("version")
                .and_then(Value::as_str)
                .unwrap_or("0.0.0")
                .to_string(),
            author: object
                .get("author")
                .and_then(Value::as_str)
                .map(|value| value.to_string()),
            image: object
                .get("images")
                .and_then(Value::as_str)
                .map(|value| value.to_string()),
            configs,
            is_system,
            can_delete: !is_system && !current_theme.is_empty() && !name.eq_ignore_ascii_case(current_theme),
        });
    }

    themes
}

fn parse_theme_config_field(value: &Value) -> Option<ThemeConfigField> {
    let object = value.as_object()?;
    let select_options = object
        .get("select_options")
        .and_then(Value::as_object)
        .map(|items| {
            items.iter()
                .filter_map(|(key, value)| {
                    Some(ThemeSelectOption {
                        value: key.clone(),
                        label: value.as_str()?.to_string(),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Some(ThemeConfigField {
        label: object
            .get("label")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        placeholder: object
            .get("placeholder")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        field_name: object
            .get("field_name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        field_type: object
            .get("field_type")
            .and_then(Value::as_str)
            .unwrap_or("input")
            .to_string(),
        default_value: object
            .get("default_value")
            .and_then(Value::as_str)
            .map(|value| value.to_string()),
        select_options,
    })
}

fn theme_source_path(name: &str) -> Option<std::path::PathBuf> {
    if !is_valid_theme_name(name) {
        return None;
    }
    let system_path = crate::runtime_paths::themes_path(name);
    if system_path.is_dir() {
        return Some(system_path);
    }

    let user_path = crate::runtime_paths::state_path(std::path::Path::new("theme").join(name));
    if user_path.is_dir() {
        return Some(user_path);
    }

    None
}

fn theme_asset_path(name: &str, relative_path: &str) -> Option<std::path::PathBuf> {
    if relative_path.trim().is_empty() {
        return None;
    }
    let safe_path = normalize_relative_path(relative_path)?;
    let public_root = crate::runtime_paths::public_path(std::path::Path::new("theme").join(name));
    if let Some(public_path) = canonical_file_under(&public_root, &safe_path) {
        return Some(public_path);
    }

    let source_root = theme_source_path(name)?;
    canonical_file_under(&source_root, &safe_path)
}

fn is_valid_theme_name(name: &str) -> bool {
    regex::Regex::new(r"^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$")
        .unwrap()
        .is_match(name)
        && !name.contains("..")
        && !name.contains('/')
        && !name.contains('\\')
}

fn normalize_relative_path(raw_path: &str) -> Option<std::path::PathBuf> {
    let trimmed = raw_path.trim_start_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    let path = std::path::Path::new(trimmed);
    if path.components().any(|component| {
        matches!(
            component,
            std::path::Component::ParentDir
                | std::path::Component::RootDir
                | std::path::Component::Prefix(_)
        )
    }) {
        return None;
    }
    Some(path.to_path_buf())
}

fn canonical_file_under(base_dir: &std::path::Path, relative_path: &std::path::Path) -> Option<std::path::PathBuf> {
    let base = std::fs::canonicalize(base_dir).ok()?;
    let file = std::fs::canonicalize(base_dir.join(relative_path)).ok()?;
    (file.starts_with(&base) && file.is_file()).then_some(file)
}

async fn overlay_portal_legacy_settings(
    state: &AppState,
    merged: &mut Map<String, Value>,
) {
    for (legacy_key, field_name) in PORTAL_LEGACY_KEYS {
        if let Some(value) = load_setting_value(state, legacy_key).await {
            merged.insert(field_name.to_string(), value);
        }
    }
}

async fn load_first_non_empty_string(
    state: &AppState,
    keys: &[&str],
    default: &str,
) -> String {
    for key in keys {
        let value = load_setting_string(state, key).await;
        if !value.is_empty() {
            return value;
        }
    }
    default.to_string()
}

async fn load_setting_string(state: &AppState, key: &str) -> String {
    match load_setting_value(state, key).await {
        Some(Value::String(value)) => value.trim().to_string(),
        Some(Value::Number(value)) => value.to_string(),
        Some(Value::Bool(value)) => {
            if value {
                "1".to_string()
            } else {
                "0".to_string()
            }
        }
        Some(Value::Null) | None => String::new(),
        Some(other) => other.to_string(),
    }
}

async fn load_setting_object(
    state: &AppState,
    key: &str,
) -> Option<Map<String, Value>> {
    load_setting_value(state, key).await.and_then(|value| match value {
        Value::Object(object) => Some(object),
        _ => None,
    })
}

async fn load_setting_value(state: &AppState, key: &str) -> Option<Value> {
    let raw = sqlx::query_scalar::<_, Option<String>>(
        "SELECT value FROM v2_settings WHERE name = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(key.to_ascii_lowercase())
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .flatten();

    raw.and_then(|value| parse_setting_value(&value))
}

async fn save_setting_value(
    state: &AppState,
    key: &str,
    value: &Value,
) -> Result<(), Response<Body>> {
    let serialized = serialize_setting_value(value);
    let normalized_key = key.to_ascii_lowercase();
    let affected = sqlx::query("UPDATE v2_settings SET value = ?, updated_at = NOW() WHERE name = ?")
        .bind(&serialized)
        .bind(&normalized_key)
        .execute(&state.db)
        .await
        .map_err(internal_error)?
        .rows_affected();

    if affected == 0 {
        sqlx::query(
            "INSERT INTO v2_settings (name, value, created_at, updated_at)
             VALUES (?, ?, NOW(), NOW())",
        )
        .bind(&normalized_key)
        .bind(&serialized)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    }

    Ok(())
}

fn theme_setting_key(name: &str) -> String {
    format!("theme_{}", name.to_ascii_lowercase())
}

fn merge_values(target: &mut Map<String, Value>, overlay: Map<String, Value>) {
    for (key, value) in overlay {
        target.insert(key, value);
    }
}

fn parse_setting_value(raw: &str) -> Option<Value> {
    if let Ok(value) = serde_json::from_str::<Value>(raw) {
        return Some(value);
    }
    Some(Value::String(raw.to_string()))
}

fn serialize_setting_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Null => String::new(),
        Value::Bool(flag) => {
            if *flag {
                "1".to_string()
            } else {
                "0".to_string()
            }
        }
        Value::Number(number) => number.to_string(),
        Value::Array(_) | Value::Object(_) => {
            serde_json::to_string(value).unwrap_or_else(|_| String::new())
        }
    }
}
