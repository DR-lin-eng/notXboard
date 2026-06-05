use crate::*;

pub fn merge_plugin_config(default_config: Value, db_config: Value) -> Value {
    let Some(defaults) = default_config.as_object() else {
        return Value::Object(Map::new());
    };
    let db_values = db_config.as_object().cloned().unwrap_or_default();
    let mut out = Map::new();
    for (key, item) in defaults {
        let mut field = item.as_object().cloned().unwrap_or_default();
        let default_value = field.get("default").cloned().unwrap_or(Value::Null);
        let stored_value = db_values.get(key).cloned().unwrap_or(default_value);
        field.insert("value".to_string(), stored_value);
        out.insert(key.clone(), Value::Object(field));
    }
    Value::Object(out)
}

pub fn normalize_plugin_config_update(
    default_config: Value,
    input_config: Map<String, Value>,
) -> Value {
    let Some(defaults) = default_config.as_object() else {
        return Value::Object(Map::new());
    };
    let mut values = Map::new();
    for (key, schema) in defaults {
        if !input_config.contains_key(key) {
            continue;
        }
        let schema_type = schema
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("string");
        let value = input_config.get(key).cloned().unwrap_or(Value::Null);
        let normalized = match schema_type {
            "boolean" => Value::Bool(value.as_bool().unwrap_or(false)),
            "number" => parse_f64_value(&value).map(Value::from).unwrap_or(Value::Null),
            "integer" => parse_i64_value(&value).map(Value::from).unwrap_or(Value::Null),
            _ => match value {
                Value::String(text) => Value::String(text),
                Value::Null => Value::String(String::new()),
                other => Value::String(other.to_string()),
            },
        };
        values.insert(key.clone(), normalized);
    }
    Value::Object(values)
}

pub fn compare_plugin_versions(local_version: &str, installed_version: &str) -> bool {
    parse_version_tuple(local_version) > parse_version_tuple(installed_version)
}

pub fn extract_default_config_values(default_config: &Value) -> Value {
    let Some(defaults) = default_config.as_object() else {
        return Value::Object(Map::new());
    };
    let mut values = Map::new();
    for (key, schema) in defaults {
        values.insert(
            key.clone(),
            schema.get("default").cloned().unwrap_or(Value::Null),
        );
    }
    Value::Object(values)
}

fn parse_version_tuple(version: &str) -> Vec<i64> {
    version
        .split('.')
        .map(|part| part.parse::<i64>().unwrap_or(0))
        .collect()
}
