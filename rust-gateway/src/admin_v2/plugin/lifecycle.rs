use crate::*;
use crate::archive_limit_support::{
    copy_zip_entry_limited, validate_zip_metadata, PrivateTempDirectory,
    PLUGIN_ARCHIVE_LIMITS, PLUGIN_ARCHIVE_MAX_BYTES,
};

use super::{catalog, config, store, PROTECTED_PLUGINS};

static PLUGIN_UPLOAD_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

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
    let config = extracted.config.clone();
    if PROTECTED_PLUGINS.contains(&config.code.as_str()) {
        return Ok(json_status_response(
            StatusCode::FORBIDDEN,
            json!({"message":"系统内置插件不允许通过上传覆盖"}),
        ));
    }
    let plugin_root = crate::runtime_paths::state_plugins_path("");
    std::fs::create_dir_all(&plugin_root)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "插件上传失败"))?;
    let _upload_guard = PLUGIN_UPLOAD_LOCK.lock().await;
    let target_dir = plugin_target_dir(&config.code)?;
    reject_plugin_directory_collisions(&plugin_root, &target_dir, &config.code)?;

    if let Some(existing_config) =
        load_existing_plugin_config(&plugin_root, &target_dir, &config.code)?
    {
        let old_version = existing_config
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or("");
        if old_version.is_empty() || compare_semver(&config.version, old_version) <= 0 {
            return Ok(json_status_response(
                StatusCode::BAD_REQUEST,
                json!({"message":"上传插件版本不高于已安装版本，无法升级"}),
            ));
        }
    }
    install_plugin_directory_atomically(
        &plugin_root,
        &target_dir,
        &extracted.plugin_dir,
        load_existing_plugin_config(&plugin_root, &target_dir, &config.code)?.is_some(),
    )?;

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
        .filter(|value| is_valid_plugin_code(value))
        .ok_or_else(|| fail_json_response(StatusCode::UNPROCESSABLE_ENTITY, "Validation failed"))
}

fn is_valid_plugin_code(code: &str) -> bool {
    (1..=64).contains(&code.len())
        && code.split('_').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
}

fn plugin_target_dir(code: &str) -> Result<std::path::PathBuf, Response<Body>> {
    if !is_valid_plugin_code(code) {
        return Err(json_status_response(
            StatusCode::BAD_REQUEST,
            json!({"message":"插件代码格式错误"}),
        ));
    }

    let directory_name = studly_plugin_dir_name(code);
    let relative = std::path::Path::new(&directory_name);
    let mut components = relative.components();
    if directory_name.is_empty()
        || !matches!(components.next(), Some(std::path::Component::Normal(_)))
        || components.next().is_some()
    {
        return Err(json_status_response(
            StatusCode::BAD_REQUEST,
            json!({"message":"插件目录名称无效"}),
        ));
    }

    let plugin_root = crate::runtime_paths::state_plugins_path("");
    let target_dir = plugin_root.join(relative);
    if !is_direct_child(&plugin_root, &target_dir) {
        return Err(json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "插件上传失败",
        ));
    }
    Ok(target_dir)
}

fn is_direct_child(root: &std::path::Path, candidate: &std::path::Path) -> bool {
    candidate != root
        && candidate.parent() == Some(root)
        && matches!(
            candidate.file_name(),
            Some(name) if !name.is_empty() && name != std::ffi::OsStr::new(".") && name != std::ffi::OsStr::new("..")
        )
}

fn reject_plugin_directory_collisions(
    plugin_root: &std::path::Path,
    target_dir: &std::path::Path,
    expected_code: &str,
) -> Result<(), Response<Body>> {
    if !is_direct_child(plugin_root, target_dir) {
        return Err(json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "插件上传失败",
        ));
    }
    let target_name = target_dir
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .ok_or_else(|| json_error(StatusCode::INTERNAL_SERVER_ERROR, "插件上传失败"))?;
    let entries = std::fs::read_dir(plugin_root)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "插件上传失败"))?;

    for entry in entries {
        let entry =
            entry.map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "插件上传失败"))?;
        let entry_name = entry.file_name();
        let entry_name_text = entry_name.to_string_lossy();
        if entry_name_text.eq_ignore_ascii_case(target_name) && entry_name_text != target_name {
            return Err(json_status_response(
                StatusCode::BAD_REQUEST,
                json!({"message":"插件目录与现有插件发生大小写重名冲突"}),
            ));
        }

        let path = entry.path();
        if path == target_dir {
            continue;
        }
        let Ok(metadata) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            continue;
        }
        let config_path = path.join("config.json");
        let Ok(config_metadata) = std::fs::symlink_metadata(&config_path) else {
            continue;
        };
        if config_metadata.file_type().is_symlink() || !config_metadata.is_file() {
            continue;
        }
        let Ok(raw) = std::fs::read_to_string(&config_path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(&raw) else {
            continue;
        };
        let sibling_code = value
            .get("code")
            .and_then(Value::as_str)
            .map(|value| value.trim().to_lowercase());
        if sibling_code.as_deref() == Some(expected_code) {
            return Err(json_status_response(
                StatusCode::BAD_REQUEST,
                json!({"message":"插件代码已存在于其他目录"}),
            ));
        }
    }
    Ok(())
}

fn load_existing_plugin_config(
    plugin_root: &std::path::Path,
    target_dir: &std::path::Path,
    expected_code: &str,
) -> Result<Option<Value>, Response<Body>> {
    if !is_direct_child(plugin_root, target_dir) {
        return Err(json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "插件上传失败",
        ));
    }
    let metadata = match std::fs::symlink_metadata(target_dir) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => {
            return Err(json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "插件上传失败",
            ))
        }
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(json_status_response(
            StatusCode::BAD_REQUEST,
            json!({"message":"插件目标路径不是安全目录"}),
        ));
    }

    let existing_config_path = target_dir.join("config.json");
    let config_metadata = std::fs::symlink_metadata(&existing_config_path).map_err(|_| {
        json_status_response(
            StatusCode::BAD_REQUEST,
            json!({"message":"已安装插件缺少配置文件，无法判断是否可升级"}),
        )
    })?;
    if config_metadata.file_type().is_symlink() || !config_metadata.is_file() {
        return Err(json_status_response(
            StatusCode::BAD_REQUEST,
            json!({"message":"已安装插件配置文件路径不安全"}),
        ));
    }
    let existing_raw = std::fs::read_to_string(&existing_config_path)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "插件上传失败"))?;
    let existing_config = serde_json::from_str::<Value>(&existing_raw).map_err(|_| {
        json_status_response(
            StatusCode::BAD_REQUEST,
            json!({"message":"已安装插件配置文件格式错误"}),
        )
    })?;
    let existing_code = existing_config
        .get("code")
        .and_then(Value::as_str)
        .map(|value| value.trim().to_lowercase());
    if existing_code.as_deref() != Some(expected_code) {
        return Err(json_status_response(
            StatusCode::BAD_REQUEST,
            json!({"message":"插件目录别名冲突，拒绝覆盖其他插件"}),
        ));
    }
    Ok(Some(existing_config))
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
    temp_root: std::path::PathBuf,
}

impl Drop for ExtractedPluginArchive {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.temp_root);
    }
}

fn extract_plugin_archive(bytes: &bytes::Bytes) -> Result<ExtractedPluginArchive, Response<Body>> {
    let reader = std::io::Cursor::new(bytes.as_ref());
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|_| json_status_response(StatusCode::BAD_REQUEST, json!({"message":"无法打开插件包文件"})))?;
    validate_zip_metadata(&mut archive, PLUGIN_ARCHIVE_LIMITS, "插件包解压后体积过大")?;

    let temp_root = std::env::temp_dir().join(format!("notxboard-plugin-{}", uuid::Uuid::new_v4()));
    let temp_directory = PrivateTempDirectory::create(temp_root)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "插件上传失败"))?;
    let temp_root = temp_directory.path();

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
        candidate_dirs.push(temp_root.to_path_buf());
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
    Ok(ExtractedPluginArchive {
        config,
        plugin_dir,
        temp_root: temp_directory.into_path(),
    })
}

fn install_plugin_directory_atomically(
    plugin_root: &std::path::Path,
    target_dir: &std::path::Path,
    source_dir: &std::path::Path,
    replacing_existing: bool,
) -> Result<(), Response<Body>> {
    if !is_direct_child(plugin_root, target_dir) {
        return Err(json_error(StatusCode::INTERNAL_SERVER_ERROR, "插件上传失败"));
    }
    let staging_root = crate::runtime_paths::state_root();
    let staging_path = staging_root.join(format!(".plugin-staging-{}", uuid::Uuid::new_v4().simple()));
    if !is_direct_child(&staging_root, &staging_path) {
        return Err(json_error(StatusCode::INTERNAL_SERVER_ERROR, "插件上传失败"));
    }
    let staging_directory = PrivateTempDirectory::create(staging_path)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "插件上传失败"))?;
    let staging = staging_directory.path();
    if let Err(error) = copy_dir_all(source_dir, staging) {
        error!("plugin staging copy failed: {error}");
        return Err(json_error(StatusCode::INTERNAL_SERVER_ERROR, "插件上传失败"));
    }

    if !replacing_existing {
        return std::fs::rename(staging, target_dir).map_err(|_| {
            json_status_response(
                StatusCode::CONFLICT,
                json!({"message":"插件目标已变化，请重新上传"}),
            )
        });
    }

    let metadata = std::fs::symlink_metadata(target_dir)
        .map_err(|_| json_status_response(StatusCode::CONFLICT, json!({"message":"插件目标已变化，请重新上传"})))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(json_status_response(
            StatusCode::BAD_REQUEST,
            json!({"message":"插件目标路径不是安全目录"}),
        ));
    }
    let backup = staging_root.join(format!(".plugin-backup-{}", uuid::Uuid::new_v4().simple()));
    if !is_direct_child(&staging_root, &backup) {
        return Err(json_error(StatusCode::INTERNAL_SERVER_ERROR, "插件上传失败"));
    }
    std::fs::rename(target_dir, &backup)
        .map_err(|_| json_error(StatusCode::INTERNAL_SERVER_ERROR, "插件上传失败"))?;
    if std::fs::rename(staging, target_dir).is_err() {
        let _ = std::fs::rename(&backup, target_dir);
        return Err(json_error(StatusCode::INTERNAL_SERVER_ERROR, "插件上传失败"));
    }
    let _ = std::fs::remove_dir_all(backup);
    Ok(())
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
        .filter(|value| is_valid_plugin_code(value))
        .ok_or_else(|| {
            json_status_response(
                StatusCode::BAD_REQUEST,
                json!({"message":"插件配置文件格式错误"}),
            )
        })?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_code_rejects_empty_segments_and_noncanonical_underscores() {
        for code in ["_", "__", "_foo", "foo_", "foo__bar"] {
            assert!(!is_valid_plugin_code(code), "{code} should be rejected");
            assert!(plugin_target_dir(code).is_err());
        }
    }

    #[test]
    fn plugin_code_accepts_canonical_segments() {
        for code in ["foo", "foo_bar", "foo1"] {
            assert!(is_valid_plugin_code(code), "{code} should be accepted");
            assert!(plugin_target_dir(code).is_ok());
        }
    }

    #[test]
    fn plugin_target_is_always_a_direct_child_and_never_the_root() {
        let root = crate::runtime_paths::state_plugins_path("");
        for code in ["foo", "foo_bar", "foo_1"] {
            let target = plugin_target_dir(code).expect("valid plugin target");
            assert_ne!(target, root);
            assert!(is_direct_child(&root, &target));
        }
    }

    #[test]
    fn noncanonical_double_underscore_cannot_alias_canonical_code() {
        assert_eq!(
            studly_plugin_dir_name("foo_bar"),
            studly_plugin_dir_name("foo__bar")
        );
        assert!(is_valid_plugin_code("foo_bar"));
        assert!(!is_valid_plugin_code("foo__bar"));
    }

    #[test]
    fn distinct_canonical_codes_that_share_studly_name_are_detected() {
        assert_eq!(
            studly_plugin_dir_name("foo1"),
            studly_plugin_dir_name("foo_1")
        );
        let root = std::env::temp_dir().join(format!(
            "notxboard-plugin-collision-test-{}",
            uuid::Uuid::new_v4()
        ));
        let target = root.join(studly_plugin_dir_name("foo_1"));
        std::fs::create_dir_all(&target).expect("create target");
        std::fs::write(
            target.join("config.json"),
            r#"{"code":"foo1","version":"1.0.0"}"#,
        )
        .expect("write config");

        let result = load_existing_plugin_config(&root, &target, "foo_1");
        let _ = std::fs::remove_dir_all(&root);
        assert!(result.is_err());
    }
}
