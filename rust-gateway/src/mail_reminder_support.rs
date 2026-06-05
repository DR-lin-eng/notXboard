use crate::*;
use crate::mail_support::{send_platform_mail, MailSendRequest};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct SendRemindMailStats {
    pub(crate) ran: bool,
    pub(crate) processed_users: i64,
    pub(crate) expire_emails: i64,
    pub(crate) traffic_emails: i64,
    pub(crate) errors: i64,
    pub(crate) skipped: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct ReminderUserRow {
    id: i64,
    email: String,
    expired_at: Option<i64>,
    transfer_enable: i64,
    u: i64,
    d: i64,
    remind_expire: i8,
    remind_traffic: i8,
}

pub(crate) async fn run_send_remind_mail(
    state: &AppState,
    now_ts: i64,
) -> Result<SendRemindMailStats, sqlx::Error> {
    if env_bool("TELEGRAM_ONLY_MODE", false) {
        return Ok(SendRemindMailStats::default());
    }
    if !get_setting_bool(state, "remind_mail_enable", false).await {
        return Ok(SendRemindMailStats::default());
    }

    let tz = chrono::FixedOffset::east_opt(8 * 3600)
        .ok_or_else(|| sqlx::Error::Protocol("invalid timezone offset".to_string()))?;
    let now = chrono::DateTime::from_timestamp(now_ts, 0)
        .ok_or_else(|| sqlx::Error::Protocol("invalid current timestamp".to_string()))?
        .with_timezone(&tz);
    let scheduled = tz
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 11, 30, 0)
        .single()
        .ok_or_else(|| sqlx::Error::Protocol("invalid remind mail schedule".to_string()))?;
    if now < scheduled {
        return Ok(SendRemindMailStats::default());
    }

    let marker_key = "scheduler:send-remind-mail:last-run-at";
    let last_run_at = redis_get_string(state, marker_key)
        .await
        .ok()
        .flatten()
        .and_then(|value| value.parse::<i64>().ok())
        .unwrap_or(0);
    if last_run_at >= scheduled.timestamp() {
        return Ok(SendRemindMailStats::default());
    }

    let users = load_reminder_users(state).await?;
    let app_name = get_setting_string(state, "app_name", "Portal").await;
    let app_url = get_setting_string(state, "app_url", &env::var("APP_URL").unwrap_or_default()).await;

    let mut stats = SendRemindMailStats {
        ran: true,
        ..Default::default()
    };

    for user in users {
        stats.processed_users += 1;
        let mut sent = 0_i64;

        if user.remind_expire != 0 && should_send_expire_remind(&user, now_ts) {
            let subject = format!("The service in {} is about to expire", app_name);
            let result = send_platform_mail(
                state,
                MailSendRequest {
                    email: &user.email,
                    subject: &subject,
                    template_name: "remindExpire",
                    template_value: &json!({
                        "name": app_name,
                        "url": app_url,
                    }),
                },
            )
            .await;
            match result {
                Ok(mail) if mail.error.is_none() => {
                    stats.expire_emails += 1;
                    sent += 1;
                }
                Ok(_) | Err(_) => stats.errors += 1,
            }
        }

        if user.remind_traffic != 0 && should_send_traffic_remind(&user) {
            let cache_key = format!("LAST_SEND_EMAIL_REMIND_TRAFFIC_{}", user.id);
            let already_sent = redis_get_string(state, &cache_key)
                .await
                .ok()
                .flatten()
                .is_some();
            if !already_sent {
                let subject = format!("The traffic usage in {} has reached 80%", app_name);
                let result = send_platform_mail(
                    state,
                    MailSendRequest {
                        email: &user.email,
                        subject: &subject,
                        template_name: "remindTraffic",
                        template_value: &json!({
                            "name": app_name,
                            "url": app_url,
                        }),
                    },
                )
                .await;
                match result {
                    Ok(mail) if mail.error.is_none() => {
                        let _ = redis_setex_string(state, &cache_key, 24 * 3600, &now_ts.to_string()).await;
                        stats.traffic_emails += 1;
                        sent += 1;
                    }
                    Ok(_) | Err(_) => stats.errors += 1,
                }
            }
        }

        if sent == 0 {
            stats.skipped += 1;
        }
    }

    let _ = redis_setex_string(state, marker_key, 172_800, &scheduled.timestamp().to_string()).await;
    Ok(stats)
}

async fn load_reminder_users(
    state: &AppState,
) -> Result<Vec<ReminderUserRow>, sqlx::Error> {
    sqlx::query_as::<_, ReminderUserRow>(
        "SELECT id, email, expired_at, COALESCE(transfer_enable, 0) AS transfer_enable,
                COALESCE(u, 0) AS u, COALESCE(d, 0) AS d, COALESCE(remind_expire, 0) AS remind_expire,
                COALESCE(remind_traffic, 0) AS remind_traffic
         FROM v2_user
         WHERE (remind_expire = 1 OR remind_traffic = 1)
           AND banned = 0
           AND email IS NOT NULL
           AND email != ''",
    )
    .fetch_all(&state.db)
    .await
}

fn should_send_expire_remind(user: &ReminderUserRow, now_ts: i64) -> bool {
    let Some(expired_at) = user.expired_at else {
        return false;
    };
    (expired_at - 86_400) < now_ts && expired_at > now_ts
}

fn should_send_traffic_remind(user: &ReminderUserRow) -> bool {
    if user.transfer_enable <= 0 {
        return false;
    }
    let used = user.u + user.d;
    if used <= 0 {
        return false;
    }
    let percentage = (used as f64 / user.transfer_enable as f64) * 100.0;
    percentage >= 80.0 && percentage < 100.0
}
