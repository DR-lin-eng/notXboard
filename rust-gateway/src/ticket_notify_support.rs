use crate::mail_support::{send_platform_mail, MailSendRequest};
use crate::*;
use std::env;
use tracing::warn;

pub(crate) async fn notify_ticket_created_to_assigned_admin(
    state: &AppState,
    assigned_admin_user_id: Option<i64>,
    node_id: Option<u64>,
    subject: &str,
    message: &str,
) -> Result<(), String> {
    if env_bool("TELEGRAM_ONLY_MODE", false) {
        return Ok(());
    }

    let Some(admin_user_id) = assigned_admin_user_id else {
        return Ok(());
    };

    let cache_key = format!("ticket_notify_admin_{}_{}", admin_user_id, node_id.unwrap_or(0));
    if redis_get_string(state, &cache_key).await.ok().flatten().is_some() {
        return Ok(());
    }

    let admin_email = sqlx::query_scalar::<_, String>("SELECT email FROM v2_user WHERE id = ? LIMIT 1")
        .bind(admin_user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| format!("load assigned admin email failed: {err}"))?;
    let Some(email) = admin_email else {
        return Ok(());
    };

    let app_name = get_setting_string(state, "app_name", "Portal").await;
    let app_url = get_setting_string(state, "app_url", &env::var("APP_URL").unwrap_or_default()).await;
    let content = format!(
        "节点ID：{}\r\n主题：{}\r\n内容：{}",
        node_id.map(|value| value.to_string()).unwrap_or_else(|| "-".to_string()),
        subject,
        message
    );

    let result = send_platform_mail(
        state,
        MailSendRequest {
            email: &email,
            subject: &format!("节点工单通知 - {}", app_name),
            template_name: "notify",
            template_value: &json!({
                "name": app_name,
                "url": app_url,
                "content": content,
            }),
        },
    )
    .await
    .map_err(|err| {
        warn!(
            admin_user_id,
            node_id = node_id.unwrap_or(0),
            subject,
            error = %err,
            "ticket assigned admin mail notify failed"
        );
        format!("notify assigned admin mail failed: {err}")
    })?;
    if let Some(error) = result.error {
        warn!(
            admin_user_id,
            node_id = node_id.unwrap_or(0),
            subject,
            error = %error,
            "ticket assigned admin mail delivery returned error"
        );
        return Err(error);
    }

    let _ = redis_setex_string(state, &cache_key, 600, "1").await;
    Ok(())
}

pub(crate) async fn notify_ticket_reply_to_user(
    state: &AppState,
    ticket_user_id: i64,
    subject: &str,
    message: &str,
) -> Result<(), String> {
    if env_bool("TELEGRAM_ONLY_MODE", false) {
        return Ok(());
    }

    let cache_key = format!("ticket_sendEmailNotify_{}", ticket_user_id);
    if redis_get_string(state, &cache_key).await.ok().flatten().is_some() {
        return Ok(());
    }

    let user_email = sqlx::query_scalar::<_, String>("SELECT email FROM v2_user WHERE id = ? LIMIT 1")
        .bind(ticket_user_id)
        .fetch_optional(&state.db)
        .await
        .map_err(|err| format!("load ticket user email failed: {err}"))?;
    let Some(email) = user_email else {
        return Ok(());
    };

    let app_name = get_setting_string(state, "app_name", "Portal").await;
    let app_url = get_setting_string(state, "app_url", &env::var("APP_URL").unwrap_or_default()).await;
    let content = format!("主题：{}\r\n回复内容：{}", subject, message);

    let result = send_platform_mail(
        state,
        MailSendRequest {
            email: &email,
            subject: &format!("您在{}的工单得到了回复", app_name),
            template_name: "notify",
            template_value: &json!({
                "name": app_name,
                "url": app_url,
                "content": content,
            }),
        },
    )
    .await
    .map_err(|err| {
        warn!(
            ticket_user_id,
            subject,
            error = %err,
            "ticket reply user mail notify failed"
        );
        format!("notify ticket user mail failed: {err}")
    })?;
    if let Some(error) = result.error {
        warn!(
            ticket_user_id,
            subject,
            error = %error,
            "ticket reply user mail delivery returned error"
        );
        return Err(error);
    }

    let _ = redis_setex_string(state, &cache_key, 1800, "1").await;
    Ok(())
}
