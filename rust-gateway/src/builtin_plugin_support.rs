use serde::Deserialize;
use serde_json::Value;

const BUILTIN_PLUGINS_JSON: &str = include_str!("../resources/plugins/builtin_plugins.json");

#[derive(Clone, Deserialize)]
pub(crate) struct BuiltinPluginConfig {
    pub(crate) name: String,
    pub(crate) code: String,
    #[serde(rename = "type", default = "default_plugin_type")]
    pub(crate) plugin_type: String,
    pub(crate) version: String,
    #[serde(default)]
    pub(crate) description: String,
    #[serde(default)]
    pub(crate) author: String,
    #[serde(default)]
    pub(crate) config: Option<Value>,
    #[serde(default)]
    pub(crate) hooks: Vec<String>,
}

pub(crate) fn builtin_plugin_configs() -> Vec<BuiltinPluginConfig> {
    serde_json::from_str(BUILTIN_PLUGINS_JSON).expect("valid builtin plugin catalog")
}

pub(crate) fn builtin_plugin_config(code: &str) -> Option<BuiltinPluginConfig> {
    let normalized = code.trim().to_ascii_lowercase();
    builtin_plugin_configs()
        .into_iter()
        .find(|plugin| plugin.code == normalized)
}

pub(crate) fn builtin_plugin_hooks() -> impl Iterator<Item = String> {
    builtin_plugin_configs()
        .into_iter()
        .flat_map(|plugin| plugin.hooks.into_iter())
}

fn default_plugin_type() -> String {
    "feature".to_string()
}
