use crate::*;
use sqlx::Row;

pub(crate) async fn send_telegram_text_to_super_admins(
    state: &AppState,
    text: &str,
    required_setting: &str,
) -> Result<i64, String> {
    if !get_setting_bool(state, "telegram_bot_enable", false).await {
        return Ok(0);
    }
    if !get_setting_bool(state, required_setting, true).await {
        return Ok(0);
    }

    let bot_token = first_non_empty(&[
        get_setting_string(state, "telegram_bot_token", "").await,
        env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default(),
    ]);
    if bot_token.trim().is_empty() {
        return Ok(0);
    }

    let chat_ids = load_super_admin_chat_ids(state).await?;
    if chat_ids.is_empty() {
        return Ok(0);
    }

    send_telegram_text(state, &bot_token, &chat_ids, text).await
}

pub(crate) async fn send_telegram_text_to_admins(
    state: &AppState,
    text: &str,
    required_setting: &str,
) -> Result<i64, String> {
    if !get_setting_bool(state, "telegram_bot_enable", false).await {
        return Ok(0);
    }
    if !get_setting_bool(state, required_setting, true).await {
        return Ok(0);
    }

    let bot_token = first_non_empty(&[
        get_setting_string(state, "telegram_bot_token", "").await,
        env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default(),
    ]);
    if bot_token.trim().is_empty() {
        return Ok(0);
    }

    let chat_ids = load_admin_chat_ids(state).await?;
    if chat_ids.is_empty() {
        return Ok(0);
    }

    send_telegram_text(state, &bot_token, &chat_ids, text).await
}

pub(crate) async fn send_telegram_text_to_chat_ids(
    state: &AppState,
    chat_ids: &[i64],
    text: &str,
    required_setting: &str,
) -> Result<i64, String> {
    if !get_setting_bool(state, "telegram_bot_enable", false).await {
        return Ok(0);
    }
    if !get_setting_bool(state, required_setting, true).await {
        return Ok(0);
    }
    if chat_ids.is_empty() {
        return Ok(0);
    }

    let bot_token = first_non_empty(&[
        get_setting_string(state, "telegram_bot_token", "").await,
        env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default(),
    ]);
    if bot_token.trim().is_empty() {
        return Ok(0);
    }

    send_telegram_text(state, &bot_token, chat_ids, text).await
}

async fn load_super_admin_chat_ids(state: &AppState) -> Result<Vec<i64>, String> {
    load_chat_ids_by_condition(state, "telegram_id IS NOT NULL AND is_super_admin = 1").await
}

async fn load_admin_chat_ids(state: &AppState) -> Result<Vec<i64>, String> {
    load_chat_ids_by_condition(state, "telegram_id IS NOT NULL AND (is_admin = 1 OR is_super_admin = 1)").await
}

async fn load_chat_ids_by_condition(state: &AppState, condition: &str) -> Result<Vec<i64>, String> {
    let sql = format!("SELECT telegram_id FROM v2_user WHERE {condition}");
    let rows = sqlx::query(&sql)
        .fetch_all(&state.db)
        .await
        .map_err(|err| format!("load telegram recipients failed: {err}"))?;

    Ok(rows
        .into_iter()
        .filter_map(|row| row.try_get::<Option<i64>, _>("telegram_id").ok().flatten())
        .collect())
}

async fn send_telegram_text(
    state: &AppState,
    bot_token: &str,
    chat_ids: &[i64],
    text: &str,
) -> Result<i64, String> {
    let api_base = env::var("TELEGRAM_API_BASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "https://api.telegram.org/bot".to_string());
    let target = format!("{api_base}{bot_token}/sendMessage");
    let req_uri: Uri = target
        .parse()
        .map_err(|err| format!("invalid telegram uri: {err}"))?;

    let mut sent = 0_i64;
    for chat_id in chat_ids {
        let payload = format!("chat_id={}&text={}", chat_id, urlencoding::encode(text));
        let request = Request::builder()
            .method(Method::POST)
            .uri(req_uri.clone())
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(payload))
            .map_err(|err| format!("build telegram request failed: {err}"))?;

        let response = state
            .backend_client
            .request(request)
            .await
            .map_err(|err| format!("telegram request failed: {err}"))?;

        if response.status().is_success() {
            sent += 1;
        }
    }

    Ok(sent)
}
