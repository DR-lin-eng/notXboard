use crate::mail_support::{send_platform_mail, MailSendRequest};
use crate::*;
use std::{env, sync::Arc};
use tokio::{sync::Semaphore, task::JoinSet};

#[derive(Clone, Debug, Serialize)]
pub(crate) struct MassMailStats {
    pub(crate) total: u64,
    pub(crate) sent: u64,
    pub(crate) failed: u64,
    pub(crate) concurrency: usize,
    pub(crate) sample_errors: Vec<String>,
}

pub(crate) async fn send_mass_mail(
    state: Arc<AppState>,
    recipients: Vec<String>,
    subject: String,
    content: String,
) -> MassMailStats {
    let total = recipients.len() as u64;
    if total == 0 {
        return MassMailStats {
            total: 0,
            sent: 0,
            failed: 0,
            concurrency: resolve_mass_mail_concurrency(0),
            sample_errors: Vec::new(),
        };
    }

    let app_name = get_setting_string(&state, "app_name", "Portal").await;
    let app_url = get_setting_string(&state, "app_url", &env::var("APP_URL").unwrap_or_default()).await;
    let concurrency = resolve_mass_mail_concurrency(recipients.len());
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut join_set = JoinSet::new();

    for email in recipients {
        let state = state.clone();
        let semaphore = semaphore.clone();
        let subject = subject.clone();
        let content = content.clone();
        let app_name = app_name.clone();
        let app_url = app_url.clone();

        join_set.spawn(async move {
            let _permit = semaphore
                .acquire_owned()
                .await
                .map_err(|err| format!("acquire mass mail permit failed: {err}"))?;

            let result = send_platform_mail(
                &state,
                MailSendRequest {
                    email: &email,
                    subject: &subject,
                    template_name: "notify",
                    template_value: &json!({
                        "name": app_name,
                        "url": app_url,
                        "content": content,
                    }),
                },
            )
            .await
            .map_err(|err| format!("{email}: {err}"))?;

            if let Some(error) = result.error {
                return Err(format!("{email}: {error}"));
            }

            Ok::<(), String>(())
        });
    }

    let mut sent = 0_u64;
    let mut failed = 0_u64;
    let mut sample_errors = Vec::new();

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(())) => sent += 1,
            Ok(Err(err)) => {
                failed += 1;
                if sample_errors.len() < 10 {
                    sample_errors.push(err);
                }
            }
            Err(err) => {
                failed += 1;
                if sample_errors.len() < 10 {
                    sample_errors.push(format!("mass mail task join failed: {err}"));
                }
            }
        }
    }

    MassMailStats {
        total,
        sent,
        failed,
        concurrency,
        sample_errors,
    }
}

fn resolve_mass_mail_concurrency(recipient_count: usize) -> usize {
    let configured = env::var("RUST_MASS_MAIL_CONCURRENCY")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(8);
    let clamped = configured.clamp(1, 64);
    if recipient_count == 0 {
        return clamped;
    }
    clamped.min(recipient_count.max(1))
}
