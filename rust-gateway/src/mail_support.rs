use crate::*;
use lettre::message::{header::ContentType, Mailbox, Message};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};

pub(crate) struct MailSendRequest<'a> {
    pub(crate) email: &'a str,
    pub(crate) subject: &'a str,
    pub(crate) template_name: &'a str,
    pub(crate) template_value: &'a Value,
}

pub(crate) struct MailSendResult {
    pub(crate) email: String,
    pub(crate) subject: String,
    pub(crate) template_name: String,
    pub(crate) error: Option<String>,
}

pub(crate) async fn send_platform_mail(
    state: &AppState,
    request: MailSendRequest<'_>,
) -> Result<MailSendResult, String> {
    let template_variant = get_setting_string(state, "email_template", "default").await;
    let resolved_template_name = resolve_template_name(request.template_name);
    let rendered = render_mail_template(&template_variant, &resolved_template_name, request.template_value)?;
    let app_name = get_setting_string(state, "app_name", "Portal").await;
    let smtp = load_mail_settings(state, &app_name).await?;

    if env_bool("TELEGRAM_ONLY_MODE", false) {
        let result = MailSendResult {
            email: request.email.to_string(),
            subject: request.subject.to_string(),
            template_name: format!("mail.{}.{}", template_variant, resolved_template_name),
            error: None,
        };
        insert_mail_log(state, &result).await.map_err(|err| format!("insert mail log failed: {err}"))?;
        return Ok(result);
    }

    let from_mailbox = Mailbox::new(
        Some(smtp.from_name.clone()),
        smtp.from_address
            .parse()
            .map_err(|err| format!("invalid from address: {err}"))?,
    );
    let to_mailbox = Mailbox::new(
        None,
        request
            .email
            .parse()
            .map_err(|err| format!("invalid recipient address: {err}"))?,
    );

    let message = Message::builder()
        .from(from_mailbox)
        .to(to_mailbox)
        .subject(request.subject)
        .header(ContentType::TEXT_HTML)
        .body(rendered)
        .map_err(|err| format!("build email failed: {err}"))?;

    let transport = build_smtp_transport(&smtp)?;
    let error = transport
        .send(message)
        .await
        .err()
        .map(|err| err.to_string());

    let result = MailSendResult {
        email: request.email.to_string(),
        subject: request.subject.to_string(),
        template_name: format!("mail.{}.{}", template_variant, resolved_template_name),
        error,
    };
    insert_mail_log(state, &result).await.map_err(|err| format!("insert mail log failed: {err}"))?;
    Ok(result)
}

fn resolve_template_name(template_name: &str) -> String {
    match template_name {
        "login" => "mailLogin".to_string(),
        other => other.to_string(),
    }
}

struct MailSettings {
    host: String,
    port: u16,
    encryption: String,
    username: String,
    password: String,
    from_address: String,
    from_name: String,
}

async fn load_mail_settings(state: &AppState, app_name: &str) -> Result<MailSettings, String> {
    let host = get_setting_string(state, "email_host", &env::var("MAIL_HOST").unwrap_or_default()).await;
    let port = get_setting_string(state, "email_port", &env::var("MAIL_PORT").unwrap_or_else(|_| "587".to_string())).await;
    let encryption = get_setting_string(state, "email_encryption", &env::var("MAIL_ENCRYPTION").unwrap_or_else(|_| "tls".to_string())).await;
    let username = get_setting_string(state, "email_username", &env::var("MAIL_USERNAME").unwrap_or_default()).await;
    let password = get_setting_string(state, "email_password", &env::var("MAIL_PASSWORD").unwrap_or_default()).await;
    let from_address = get_setting_string(state, "email_from_address", &env::var("MAIL_FROM_ADDRESS").unwrap_or_default()).await;
    let from_name = app_name.to_string();

    if host.trim().is_empty() {
        return Err("email_host is empty".to_string());
    }
    if from_address.trim().is_empty() {
        return Err("email_from_address is empty".to_string());
    }

    Ok(MailSettings {
        host,
        port: port.parse::<u16>().unwrap_or(587),
        encryption,
        username,
        password,
        from_address,
        from_name,
    })
}

fn build_smtp_transport(
    settings: &MailSettings,
) -> Result<AsyncSmtpTransport<Tokio1Executor>, String> {
    let encryption = settings.encryption.trim().to_ascii_lowercase();
    let mut builder = if encryption == "ssl" {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&settings.host)
            .map_err(|err| format!("build smtps transport failed: {err}"))?
    } else if matches!(encryption.as_str(), "" | "none" | "plain" | "off" | "false" | "0") {
        AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&settings.host)
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&settings.host)
            .map_err(|err| format!("build smtp transport failed: {err}"))?
    };

    builder = builder.port(settings.port);
    if !settings.username.trim().is_empty() {
        builder = builder.credentials(Credentials::new(
            settings.username.clone(),
            settings.password.clone(),
        ));
    }
    Ok(builder.build())
}

fn render_mail_template(
    variant: &str,
    template_name: &str,
    values: &Value,
) -> Result<String, String> {
    let path = crate::runtime_paths::resources_path(format!("views/mail/{variant}/{template_name}.html"));
    let template = std::fs::read_to_string(&path)
        .map_err(|err| format!("load mail template failed: {}: {err}", path.display()))?;

    let name = values.get("name").and_then(Value::as_str).unwrap_or("");
    let url = values.get("url").and_then(Value::as_str).unwrap_or("");
    let code = values
        .get("code")
        .and_then(|value| value.as_i64().map(|n| n.to_string()).or_else(|| value.as_str().map(|s| s.to_string())))
        .unwrap_or_default();
    let link = values.get("link").and_then(Value::as_str).unwrap_or("");
    let content = values.get("content").and_then(Value::as_str).unwrap_or("");

    let content_html = escape_html_local(content).replace('\n', "<br />");

    Ok(template
        .replace("{{$name}}", &escape_html_local(name))
        .replace("{{ $name }}", &escape_html_local(name))
        .replace("{{$url}}", &escape_html_local(url))
        .replace("{{ $url }}", &escape_html_local(url))
        .replace("{{$code}}", &escape_html_local(&code))
        .replace("{{ $code }}", &escape_html_local(&code))
        .replace("{{$link}}", &escape_html_local(link))
        .replace("{{ $link }}", &escape_html_local(link))
        .replace("{!! nl2br($content) !!}", &content_html)
        .replace("{!! nl2br(e($content)) !!}", &content_html))
}

async fn insert_mail_log(
    state: &AppState,
    result: &MailSendResult,
) -> Result<(), sqlx::Error> {
    let now = Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO v2_mail_log (email, subject, template_name, error, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&result.email)
    .bind(&result.subject)
    .bind(&result.template_name)
    .bind(result.error.clone())
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await?;
    Ok(())
}

fn escape_html_local(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
