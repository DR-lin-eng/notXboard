use crate::*;

use super::store::PluginDirectoryConfig;

pub fn load_plugin_directory_configs(type_filter: Option<&str>) -> Vec<PluginDirectoryConfig> {
    let mut plugins = scan_plugin_directories();
    if let Some(type_filter) = type_filter {
        plugins.retain(|plugin| plugin.r#type == type_filter);
    }
    plugins.sort_by(|a, b| a.code.cmp(&b.code));
    plugins
}

pub fn load_single_plugin_directory_config(code: &str) -> Option<PluginDirectoryConfig> {
    load_plugin_directory_configs(None)
        .into_iter()
        .find(|plugin| plugin.code == code)
}

fn scan_plugin_directories() -> Vec<PluginDirectoryConfig> {
    let mut plugins = Vec::new();
    let mut seen = std::collections::HashSet::new();
    add_builtin_plugins(&mut seen, &mut plugins);
    scan_plugin_root(&crate::runtime_paths::state_plugins_path(""), &mut seen, &mut plugins);
    scan_plugin_root(&crate::runtime_paths::plugins_path(""), &mut seen, &mut plugins);

    plugins
}

fn add_builtin_plugins(
    seen: &mut std::collections::HashSet<String>,
    plugins: &mut Vec<PluginDirectoryConfig>,
) {
    for config in crate::builtin_plugin_support::builtin_plugin_configs() {
        if !seen.insert(config.code.clone()) {
            continue;
        }
        plugins.push(PluginDirectoryConfig {
            code: config.code,
            name: config.name,
            version: config.version,
            description: config.description,
            author: config.author,
            r#type: config.plugin_type,
            config: config.config,
            readme: None,
        });
    }
}

fn scan_plugin_root(
    plugin_root: &std::path::Path,
    seen: &mut std::collections::HashSet<String>,
    plugins: &mut Vec<PluginDirectoryConfig>,
) {
    let Ok(entries) = std::fs::read_dir(plugin_root) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let config_path = path.join("config.json");
        if !config_path.exists() {
            continue;
        }

        let Ok(raw) = std::fs::read_to_string(&config_path) else {
            continue;
        };
        let Ok(config) = serde_json::from_str::<Value>(&raw) else {
            continue;
        };
        let Some(object) = config.as_object() else {
            continue;
        };

        let code = object
            .get("code")
            .and_then(Value::as_str)
            .map(|value| value.trim().to_lowercase())
            .filter(|value| !value.is_empty());
        let name = object
            .get("name")
            .and_then(Value::as_str)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let version = object
            .get("version")
            .and_then(Value::as_str)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let description = object
            .get("description")
            .and_then(Value::as_str)
            .map(|value| value.trim().to_string())
            .unwrap_or_default();
        let author = object
            .get("author")
            .and_then(Value::as_str)
            .map(|value| value.trim().to_string())
            .unwrap_or_default();

        let (Some(code), Some(name), Some(version)) = (code, name, version) else {
            continue;
        };
        if super::PROTECTED_PLUGINS.contains(&code.as_str()) {
            continue;
        }
        if !seen.insert(code.clone()) {
            continue;
        }

        let plugin_type = object
            .get("type")
            .and_then(Value::as_str)
            .map(|value| value.trim().to_lowercase())
            .filter(|value| matches!(value.as_str(), "feature" | "payment"))
            .unwrap_or_else(|| "feature".to_string());

        let readme = ["README.md", "readme.md"]
            .iter()
            .map(|name| path.join(name))
            .find(|candidate| candidate.exists())
            .and_then(|candidate| std::fs::read_to_string(candidate).ok());

        plugins.push(PluginDirectoryConfig {
            code,
            name,
            version,
            description,
            author,
            r#type: plugin_type,
            config: object.get("config").cloned(),
            readme,
        });
    }
}
