use serde::Deserialize;
use serde_json::Value;

pub(crate) const MAINTAINABLE_THEME_NAME: &str = "Maintainable";
pub(crate) const PORTAL_THEME_NAME: &str = "portal";

const BUILTIN_THEMES_JSON: &str = include_str!("../resources/themes/builtin_themes.json");
const MAINTAINABLE_TEMPLATE: &str = include_str!("../resources/themes/Maintainable/dashboard.blade.php");
const PORTAL_TEMPLATE: &str = include_str!("../resources/themes/portal/dashboard.blade.php");

#[derive(Clone, Deserialize)]
pub(crate) struct BuiltinThemeConfig {
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) title: Option<String>,
    #[serde(default)]
    pub(crate) description: String,
    pub(crate) version: String,
    #[serde(default)]
    pub(crate) author: Option<String>,
    #[serde(default, rename = "images")]
    pub(crate) image: Option<String>,
    #[serde(default)]
    pub(crate) configs: Vec<Value>,
}

pub(crate) fn builtin_theme_configs() -> Vec<BuiltinThemeConfig> {
    serde_json::from_str(BUILTIN_THEMES_JSON).expect("valid builtin theme catalog")
}

pub(crate) fn builtin_theme_template(name: &str) -> Option<&'static str> {
    if name.eq_ignore_ascii_case(MAINTAINABLE_THEME_NAME) {
        Some(MAINTAINABLE_TEMPLATE)
    } else if name.eq_ignore_ascii_case(PORTAL_THEME_NAME) {
        Some(PORTAL_TEMPLATE)
    } else {
        None
    }
}

pub(crate) fn is_builtin_theme(name: &str) -> bool {
    name.eq_ignore_ascii_case(MAINTAINABLE_THEME_NAME) || name.eq_ignore_ascii_case(PORTAL_THEME_NAME)
}
