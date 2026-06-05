use crate::*;
use crate::archive_limit_support::{
    copy_zip_entry_limited, validate_zip_metadata, PLUGIN_ARCHIVE_LIMITS,
    PLUGIN_ARCHIVE_MAX_BYTES,
};

use super::{catalog, config, store, PROTECTED_PLUGINS};

pub async fn build_install_response(
    state: &AppState,
    headers: HeaderMap,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let code = parse_plugin_code(body).await?;
    let plugin = catalog::load_single_plugin_directory_config(&code)
        .ok_or_else(|| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"插件安装失败：插件不存在"})))?;
    if store::load_installed_plugin_by_code(state, &code)
        .await
        .map_err(internal_error)?
        .is_some()
    {
        return Ok(json_status_response(
            StatusCode::BAD_REQUEST,
            json!({"message":"插件安装失败：插件已安装"}),
        ));
    }

    let default_config = config::extract_default_config_values(
        plugin
            .config
            .as_ref()
            .unwrap_or(&Value::Object(Map::new())),
    );
    let serialized = serde_json::to_string(&default_config)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "插件安装失败"))?;
    store::insert_plugin(
        state,
        &plugin.code,
        &plugin.name,
        &plugin.r#type,
        &plugin.version,
        &serialized,
    )
    .await
    .map_err(internal_error)?;

    Ok(json_value_response(json!({
        "message": "插件安装成功"
    })))
}

pub async fn build_uninstall_response(
    state: &AppState,
    headers: HeaderMap,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let code = parse_plugin_code(body).await?;
    let plugin = store::load_installed_plugin_by_code(state, &code)
        .await
        .map_err(internal_error)?;
    let Some(plugin) = plugin else {
        return Ok(json_status_response(
            StatusCode::BAD_REQUEST,
            json!({"message":"插件卸载失败：插件不存在或尚未安装"}),
        ));
    };
    if plugin.is_enabled != 0 {
        return Ok(json_status_response(
            StatusCode::BAD_REQUEST,
            json!({"message":"请先禁用插件后再卸载"}),
        ));
    }

    store::delete_plugin_by_code(state, &code)
        .await
        .map_err(internal_error)?;
    Ok(json_value_response(json!({
        "message": "插件卸载成功"
    })))
}

pub async fn build_upload_response(
    state: &AppState,
    headers: HeaderMap,
    mut multipart: axum::extract::Multipart,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;

    let mut file_name = None::<String>;
    let mut file_bytes = None::<bytes::Bytes>;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"插件上传失败：无法读取上传内容"})))?
    {
        let name = field.name().unwrap_or_default().to_string();
        if name != "file" {
            continue;
        }
        file_name = field.file_name().map(|value| value.to_string());
        let bytes = field
            .bytes()
            .await
            .map_err(|_| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"插件上传失败：无法读取插件包文件"})))?;
        file_bytes = Some(bytes);
        break;
    }

    let upload_name = file_name.unwrap_or_default();
    if !upload_name.to_lowercase().ends_with(".zip") {
        return Ok(json_status_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({"message":"插件上传失败：插件包必须是zip格式"}),
        ));
    }

    let file_bytes = file_bytes.ok_or_else(|| {
        json_status_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({"message":"插件上传失败：请选择插件包文件"}),
        )
    })?;
    if file_bytes.len() > PLUGIN_ARCHIVE_MAX_BYTES {
        return Ok(json_status_response(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({"message":"插件上传失败：插件包大小不能超过10MB"}),
        ));
    }

    let extracted = extract_plugin_archive(&file_bytes)?;
    let config = extracted.config;
    let target_dir = crate::runtime_paths::state_plugins_path(studly_plugin_dir_name(&config.code));

    if target_dir.exists() {
        let existing_config_path = target_dir.join("config.json");
        if !existing_config_path.exists() {
            return Ok(json_status_response(
                StatusCode::BAD_REQUEST,
                json!({"message":"已安装插件缺少配置文件，无法判断是否可升级"}),
            ));
        }
        let existing_raw = std::fs::read_to_string(&existing_config_path)
            .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "插件上传失败"))?;
        let existing_config = serde_json::from_str::<Value>(&existing_raw)
            .map_err(|_| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"已安装插件配置文件格式错误"})))?;
        let old_version = existing_config.get("version").and_then(Value::as_str).unwrap_or("");
        if old_version.is_empty() || compare_semver(&config.version, old_version) <= 0 {
            return Ok(json_status_response(
                StatusCode::BAD_REQUEST,
                json!({"message":"上传插件版本不高于已安装版本，无法升级"}),
            ));
        }
        std::fs::remove_dir_all(&target_dir)
            .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "插件上传失败"))?;
    }

    std::fs::create_dir_all(
        target_dir
            .parent()
            .map(std::path::Path::to_path_buf)
            .unwrap_or_else(|| crate::runtime_paths::state_plugins_path("")),
    )
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "插件上传失败"))?;
    std::fs::rename(&extracted.plugin_dir, &target_dir).or_else(|_| {
        copy_dir_all(&extracted.plugin_dir, &target_dir)?;
        std::fs::remove_dir_all(&extracted.plugin_dir)
    })
    .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "插件上传失败"))?;

    if store::load_installed_plugin_by_code(state, &config.code)
        .await
        .map_err(internal_error)?
        .is_some()
    {
        store::update_plugin_metadata(
            state,
            &config.code,
            &config.name,
            &config.r#type,
            &config.version,
        )
        .await
        .map_err(internal_error)?;
    }

    Ok(json_value_response(json!({
        "message": "插件上传成功"
    })))
}

pub async fn build_upgrade_response(
    state: &AppState,
    headers: HeaderMap,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let code = parse_plugin_code(body).await?;
    let installed = store::load_installed_plugin_by_code(state, &code)
        .await
        .map_err(internal_error)?;
    let Some(installed) = installed else {
        return Ok(json_status_response(
            StatusCode::BAD_REQUEST,
            json!({"message":"插件升级失败：插件不存在或尚未安装"}),
        ));
    };
    let plugin = catalog::load_single_plugin_directory_config(&code)
        .ok_or_else(|| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"插件升级失败：插件不存在"})))?;

    if !config::compare_plugin_versions(&plugin.version, &installed.version) {
        return Ok(json_status_response(
            StatusCode::BAD_REQUEST,
            json!({"message":"插件已经是最新版本"}),
        ));
    }

    store::update_plugin_metadata(
        state,
        &plugin.code,
        &plugin.name,
        &plugin.r#type,
        &plugin.version,
    )
    .await
    .map_err(internal_error)?;
    Ok(json_value_response(json!({
        "message": "插件升级成功"
    })))
}

pub async fn build_delete_response(
    state: &AppState,
    headers: HeaderMap,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let code = parse_plugin_code(body).await?;
    if PROTECTED_PLUGINS.contains(&code.as_str()) {
        return Ok(json_status_response(
            StatusCode::FORBIDDEN,
            json!({"message":"该插件为系统默认插件，不允许删除"}),
        ));
    }

    let deleted = store::delete_plugin_by_code(state, &code)
        .await
        .map_err(internal_error)?;
    if deleted == 0 && catalog::load_single_plugin_directory_config(&code).is_none() {
        return Ok(json_status_response(
            StatusCode::BAD_REQUEST,
            json!({"message":"插件删除失败：插件不存在"}),
        ));
    }

    Ok(json_value_response(json!({
        "message": "插件删除成功"
    })))
}

pub async fn build_toggle_enabled_response(
    state: &AppState,
    headers: HeaderMap,
    body: Body,
    enabled: bool,
) -> Result<Response<Body>, Response<Body>> {
    let _admin = authenticate_super_admin_user(state, &headers).await?;
    let code = parse_plugin_code(body).await?;
    let plugin = store::load_installed_plugin_by_code(state, &code)
        .await
        .map_err(internal_error)?;
    let Some(_) = plugin else {
        return Ok(json_status_response(
            StatusCode::BAD_REQUEST,
            json!({"message": if enabled { "插件启用失败：插件不存在" } else { "插件禁用失败：插件不存在" }}),
        ));
    };

    sqlx::query("UPDATE v2_plugins SET is_enabled = ?, updated_at = NOW() WHERE code = ?")
        .bind(if enabled { 1 } else { 0 })
        .bind(&code)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    Ok(json_value_response(json!({
        "message": if enabled { "插件启用成功" } else { "插件禁用成功" }
    })))
}

async fn parse_plugin_code(body: Body) -> Result<String, Response<Body>> {
    let payload = parse_json_body(body).await?;
    payload
        .get("code")
        .and_then(Value::as_str)
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))
}

#[derive(Clone)]
struct UploadedPluginConfig {
    code: String,
    name: String,
    version: String,
    r#type: String,
}

struct ExtractedPluginArchive {
    config: UploadedPluginConfig,
    plugin_dir: std::path::PathBuf,
}

fn extract_plugin_archive(bytes: &bytes::Bytes) -> Result<ExtractedPluginArchive, Response<Body>> {
    let reader = std::io::Cursor::new(bytes.as_ref());
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|_| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"无法打开插件包文件"})))?;
    validate_zip_metadata(&mut archive, PLUGIN_ARCHIVE_LIMITS, "插件包解压后体积过大")?;

    let temp_root = std::env::temp_dir().join(format!("notxboard-plugin-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_root)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "插件上传失败"))?;

    let mut total_written = 0_u64;
    for index in 0..archive.len() {
        let mut file = archive
            .by_index(index)
            .map_err(|_| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"无法读取插件包内容"})))?;
        let enclosed = file
            .enclosed_name()
            .map(|path| path.to_path_buf())
            .ok_or_else(|| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"插件包包含非法路径"})))?;
        let out_path = temp_root.join(enclosed);
        if file.is_dir() {
            std::fs::create_dir_all(&out_path)
                .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "插件上传失败"))?;
            continue;
        }
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "插件上传失败"))?;
        }
        let mut out_file = std::fs::File::create(&out_path)
            .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "插件上传失败"))?;
        copy_zip_entry_limited(
            &mut file,
            &mut out_file,
            &mut total_written,
            PLUGIN_ARCHIVE_LIMITS,
            "插件包解压后体积过大",
        )?;
    }

    let mut candidate_dirs = Vec::new();
    let direct_config = temp_root.join("config.json");
    if direct_config.exists() {
        candidate_dirs.push(temp_root.clone());
    }
    if let Ok(entries) = std::fs::read_dir(&temp_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("config.json").exists() {
                candidate_dirs.push(path);
            }
        }
    }

    let plugin_dir = candidate_dirs
        .into_iter()
        .next()
        .ok_or_else(|| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"插件包格式错误：缺少配置文件"})))?;
    let raw = std::fs::read_to_string(plugin_dir.join("config.json"))
        .map_err(|_| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"插件配置文件格式错误"})))?;
    let config = parse_uploaded_plugin_config(&raw)?;
    Ok(ExtractedPluginArchive { config, plugin_dir })
}

fn parse_uploaded_plugin_config(raw: &str) -> Result<UploadedPluginConfig, Response<Body>> {
    let value = serde_json::from_str::<Value>(raw)
        .map_err(|_| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"插件配置文件格式错误"})))?;
    let object = value
        .as_object()
        .ok_or_else(|| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"插件配置文件格式错误"})))?;

    let name = object
        .get("name")
        .and_then(Value::as_str)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"插件配置文件格式错误"})))?;
    let code = object
        .get("code")
        .and_then(Value::as_str)
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"插件配置文件格式错误"})))?;
    let version = object
        .get("version")
        .and_then(Value::as_str)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"插件配置文件格式错误"})))?;
    let description_ok = object
        .get("description")
        .and_then(Value::as_str)
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    let author_ok = object
        .get("author")
        .and_then(Value::as_str)
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    if !description_ok || !author_ok {
        return Err(json_status_response(
            StatusCode::BAD_REQUEST,
            json!({"message":"插件配置文件格式错误"}),
        ));
    }
    if !regex::Regex::new(r"^[a-z0-9_]+$").unwrap().is_match(&code) {
        return Err(json_status_response(
            StatusCode::BAD_REQUEST,
            json!({"message":"插件配置文件格式错误"}),
        ));
    }
    if !regex::Regex::new(r"^\d+\.\d+\.\d+$").unwrap().is_match(&version) {
        return Err(json_status_response(
            StatusCode::BAD_REQUEST,
            json!({"message":"插件配置文件格式错误"}),
        ));
    }
    let plugin_type = object
        .get("type")
        .and_then(Value::as_str)
        .map(|value| value.trim().to_lowercase())
        .unwrap_or_else(|| "feature".to_string());
    if !matches!(plugin_type.as_str(), "feature" | "payment") {
        return Err(json_status_response(
            StatusCode::BAD_REQUEST,
            json!({"message":"插件配置文件格式错误"}),
        ));
    }

    Ok(UploadedPluginConfig {
        code,
        name,
        version,
        r#type: plugin_type,
    })
}

fn studly_plugin_dir_name(code: &str) -> String {
    code.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<String>()
}

fn compare_semver(new_version: &str, old_version: &str) -> i32 {
    let parse = |value: &str| {
        value
            .split('.')
            .map(|part| part.parse::<i64>().unwrap_or(0))
            .collect::<Vec<_>>()
    };
    let left = parse(new_version);
    let right = parse(old_version);
    let max_len = left.len().max(right.len());
    for index in 0..max_len {
        let l = *left.get(index).unwrap_or(&0);
        let r = *right.get(index).unwrap_or(&0);
        if l > r {
            return 1;
        }
        if l < r {
            return -1;
        }
    }
    0
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}
