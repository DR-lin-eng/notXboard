use crate::*;
use sqlx::{QueryBuilder, Row};
use tracing::info;

pub(crate) async fn notify_notice_published(
    state: &AppState,
    notice_id: i64,
) -> Result<i64, String> {
    if !get_setting_bool(state, "telegram_notify_notice_published", true).await {
        return Ok(0);
    }

    let notice = sqlx::query(
        "SELECT id, title, content, tags, scope_type, target_plan_ids, `show`
         FROM v2_notice
         WHERE id = ?
         LIMIT 1",
    )
    .bind(notice_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| format!("load notice failed: {err}"))?;
    let Some(notice) = notice else {
        return Ok(0);
    };

    let show = notice.try_get::<i64, _>("show").unwrap_or(0);
    if show == 0 {
        return Ok(0);
    }

    let title = notice.try_get::<String, _>("title").unwrap_or_default();
    let content = notice.try_get::<String, _>("content").unwrap_or_default();
    let scope_type = notice
        .try_get::<Option<String>, _>("scope_type")
        .ok()
        .flatten()
        .unwrap_or_else(|| "global".to_string());
    let tags_json = notice.try_get::<Option<String>, _>("tags").ok().flatten();
    let target_plan_ids_json = notice
        .try_get::<Option<String>, _>("target_plan_ids")
        .ok()
        .flatten();

    let tags = tags_json
        .as_deref()
        .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|item| item.as_str().map(str::to_string))
        .collect::<Vec<_>>();

    let recipients =
        load_notice_recipient_chat_ids(state, &scope_type, target_plan_ids_json.as_deref()).await?;
    info!(
        notice_id,
        scope_type,
        recipient_count = recipients.len(),
        "notice publish telegram notify resolved recipients"
    );
    if recipients.is_empty() {
        return Ok(0);
    }

    let mut lines = vec!["公告发布".to_string(), format!("标题：{}", title)];
    if !tags.is_empty() {
        lines.push(format!("标签：{}", tags.join(" / ")));
    }
    lines.push(format!("内容摘要：{}", limit_text_local(&content, 180)));

    send_telegram_text_to_chat_ids(
        state,
        &recipients,
        &lines.join("\n"),
        "telegram_notify_notice_published",
    )
    .await
}

async fn load_notice_recipient_chat_ids(
    state: &AppState,
    scope_type: &str,
    target_plan_ids_json: Option<&str>,
) -> Result<Vec<i64>, String> {
    if scope_type == "plan_subscribers" {
        let plan_ids = target_plan_ids_json
            .and_then(|raw| serde_json::from_str::<Vec<i64>>(raw).ok())
            .unwrap_or_default();
        if plan_ids.is_empty() {
            return Ok(Vec::new());
        }

        let mut builder = QueryBuilder::<sqlx::MySql>::new(
            "SELECT DISTINCT u.telegram_id
             FROM v2_user u
             JOIN user_plan_subscriptions ups ON ups.user_id = u.id
             WHERE u.telegram_id IS NOT NULL
               AND ups.status = 1
               AND ups.plan_id IN (",
        );
        {
            let mut separated = builder.separated(", ");
            for plan_id in &plan_ids {
                separated.push_bind(*plan_id);
            }
        }
        builder.push(")");
        let rows = builder
            .build()
            .fetch_all(&state.db)
            .await
            .map_err(|err| format!("load notice plan recipients failed: {err}"))?;

        return Ok(rows
            .into_iter()
            .filter_map(|row| row.try_get::<Option<i64>, _>("telegram_id").ok().flatten())
            .collect());
    }

    let rows = sqlx::query("SELECT telegram_id FROM v2_user WHERE telegram_id IS NOT NULL")
        .fetch_all(&state.db)
        .await
        .map_err(|err| format!("load notice recipients failed: {err}"))?;

    Ok(rows
        .into_iter()
        .filter_map(|row| row.try_get::<Option<i64>, _>("telegram_id").ok().flatten())
        .collect())
}

fn limit_text_local(input: &str, max_chars: usize) -> String {
    let text = input.trim();
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let truncated = text.chars().take(max_chars).collect::<String>();
    format!("{}...", truncated)
}
