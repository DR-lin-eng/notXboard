use crate::*;

pub(crate) async fn send_ops_alert(state: &AppState, message: &str) -> Result<bool, String> {
    Ok(send_telegram_text_to_admins(state, message, "telegram_notify_ops_alert").await? > 0)
}
