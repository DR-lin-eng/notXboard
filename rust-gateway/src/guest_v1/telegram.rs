use crate::*;
use hyper_util::rt::TokioExecutor;

#[derive(Deserialize)]
struct TelegramWebhookPayload {
    message: Option<TelegramMessage>,
    callback_query: Option<TelegramCallbackQuery>,
    chat_join_request: Option<TelegramChatJoinRequest>,
}

#[derive(Deserialize)]
struct TelegramMessage {
    message_id: i64,
    text: Option<String>,
    chat: TelegramChat,
    from: Option<TelegramUser>,
    reply_to_message: Option<TelegramReplyMessage>,
}

#[derive(Deserialize)]
struct TelegramReplyMessage {
    message_id: i64,
    text: Option<String>,
}

#[derive(Deserialize)]
struct TelegramCallbackQuery {
    id: String,
    data: Option<String>,
    from: Option<TelegramUser>,
    message: Option<TelegramCallbackMessage>,
}

#[derive(Deserialize)]
struct TelegramCallbackMessage {
    message_id: i64,
    chat: TelegramChat,
}

#[derive(Deserialize)]
struct TelegramChatJoinRequest {
    chat: TelegramChat,
    from: Option<TelegramUser>,
}

#[derive(Deserialize)]
struct TelegramChat {
    id: i64,
    #[serde(rename = "type")]
    chat_type: String,
}

#[derive(Deserialize)]
struct TelegramUser {
    id: i64,
}

pub async fn webhook(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    uri: Uri,
    body: Body,
) -> Response<Body> {
    match build_webhook_response(&state, headers, uri, body).await {
        Ok(response) => response,
        Err(response) => response,
    }
}

async fn build_webhook_response(
    state: &AppState,
    headers: HeaderMap,
    _uri: Uri,
    body: Body,
) -> Result<Response<Body>, Response<Body>> {
    if !get_setting_bool(state, "telegram_bot_enable", false).await {
        return Ok(Response::builder().status(StatusCode::NO_CONTENT).body(Body::empty()).unwrap());
    }

    let token = resolve_telegram_bot_token(state).await;
    if token.is_empty() {
        return Err(json_error(StatusCode::UNAUTHORIZED, "access_token is error"));
    }

    if !telegram_webhook_authorized(&headers, &token) {
        return Err(json_error(StatusCode::UNAUTHORIZED, "access_token is error"));
    }

    let payload = parse_json_body(body).await?;
    let update: TelegramWebhookPayload = serde_json::from_value(payload)
        .map_err(|_| fail_json_response(StatusCode::BAD_REQUEST, "Invalid JSON body"))?;

    if let Some(join_request) = update.chat_join_request {
        handle_chat_join_request(state, &token, join_request).await?;
    }

    if let Some(message) = update.message {
        handle_message_update(state, &token, message).await?;
    }

    if let Some(callback_query) = update.callback_query {
        handle_callback_query(state, &token, callback_query).await?;
    }

    Ok(Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(Body::empty())
        .unwrap())
}

async fn handle_message_update(
    state: &AppState,
    token: &str,
    message: TelegramMessage,
) -> Result<(), Response<Body>> {
    let text = message.text.clone().unwrap_or_default();
    if text.trim().is_empty() {
        return Ok(());
    }

    if message.reply_to_message.is_some() {
        if handle_ticket_reply_message(state, token, &message).await? {
            return Ok(());
        }
    }

    let command_text = text.split_whitespace().next().unwrap_or_default().trim();
    let args = text
        .split_whitespace()
        .skip(1)
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    let command = normalize_command(command_text);

    match command.as_str() {
        "/start" => send_start_message(state, token, &message).await?,
        "/bind" => handle_bind_command(state, token, &message, &args).await?,
        "/traffic" => handle_traffic_command(state, token, &message).await?,
        "/getlatesturl" => handle_get_latest_url_command(state, token, &message).await?,
        "/tickets" => handle_tickets_command(state, token, &message).await?,
        "/ticket" => handle_ticket_detail_command(state, token, &message, &args).await?,
        "/close" => handle_ticket_close_command(state, token, &message, &args).await?,
        "/unbind" => handle_unbind_command(state, token, &message).await?,
        _ => send_help_message(state, token, message.chat.id).await?,
    }

    Ok(())
}

async fn handle_bind_command(
    state: &AppState,
    token: &str,
    message: &TelegramMessage,
    args: &[String],
) -> Result<(), Response<Body>> {
    if !ensure_private_chat(token, &message.chat).await? {
        send_plain_message(token, message.chat.id, "请在私聊中使用此命令。").await?;
        return Ok(());
    }

    let subscribe_url = args.first().map(|value| value.trim()).filter(|value| !value.is_empty());
    let Some(subscribe_url) = subscribe_url else {
        send_plain_message(token, message.chat.id, "参数有误，请携带订阅地址发送。").await?;
        return Ok(());
    };

    let Some(user_token) = extract_subscribe_token(state, subscribe_url).await? else {
        send_plain_message(token, message.chat.id, "订阅地址无效。").await?;
        return Ok(());
    };

    let user = load_bearer_user_by_token(state, &user_token)
        .await
        .map_err(internal_error)?;
    let Some(user) = user else {
        send_plain_message(token, message.chat.id, "用户不存在。").await?;
        return Ok(());
    };

    if let Some(bound) = user.telegram_id {
        if bound != message.chat.id {
            send_plain_message(token, message.chat.id, "该账号已经绑定了其他 Telegram 账号。").await?;
            return Ok(());
        }
    }

    sqlx::query("UPDATE v2_user SET telegram_id = ?, updated_at = ? WHERE id = ?")
        .bind(message.chat.id)
        .bind(Utc::now().timestamp())
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    send_plain_message(
        token,
        message.chat.id,
        &format!("绑定成功。\n账号：{}\n可用命令：/traffic /getlatesturl /start", user.email),
    )
    .await?;
    Ok(())
}

async fn handle_traffic_command(
    state: &AppState,
    token: &str,
    message: &TelegramMessage,
) -> Result<(), Response<Body>> {
    if !ensure_private_chat(token, &message.chat).await? {
        send_plain_message(token, message.chat.id, "请在私聊中使用此命令。").await?;
        return Ok(());
    }

    let user = match load_bound_telegram_user(state, message.chat.id).await? {
        Some(user) => user,
        None => {
            send_plain_message(token, message.chat.id, "请先绑定账号。").await?;
            return Ok(());
        }
    };

    let transfer_used = (user.u + user.d) as f64;
    let transfer_total = user.transfer_enable as f64;
    let transfer_remaining = (user.transfer_enable - user.u - user.d).max(0) as f64;
    let usage_percentage = if transfer_total > 0.0 {
        transfer_used / transfer_total * 100.0
    } else {
        0.0
    };

    let text = format!(
        "流量使用情况\n\n已用流量：{} GB\n总流量：{} GB\n剩余流量：{} GB\n使用率：{:.2}%",
        transfer_to_gb_string(transfer_used),
        transfer_to_gb_string(transfer_total),
        transfer_to_gb_string(transfer_remaining),
        usage_percentage
    );
    send_plain_message(token, message.chat.id, &text).await?;
    Ok(())
}

async fn handle_get_latest_url_command(
    state: &AppState,
    token: &str,
    message: &TelegramMessage,
) -> Result<(), Response<Body>> {
    if !ensure_private_chat(token, &message.chat).await? {
        send_plain_message(token, message.chat.id, "请在私聊中使用此命令。").await?;
        return Ok(());
    }

    let user = match load_bound_telegram_user(state, message.chat.id).await? {
        Some(user) => user,
        None => {
            send_plain_message(token, message.chat.id, "请先绑定账号。").await?;
            return Ok(());
        }
    };

    let subscribe_url = build_user_subscribe_url(state, &user).await.unwrap_or_default();
    send_plain_message(token, message.chat.id, &format!("您的订阅链接：\n\n{}", subscribe_url)).await?;
    Ok(())
}

async fn handle_unbind_command(
    state: &AppState,
    token: &str,
    message: &TelegramMessage,
) -> Result<(), Response<Body>> {
    if !ensure_private_chat(token, &message.chat).await? {
        send_plain_message(token, message.chat.id, "请在私聊中使用此命令。").await?;
        return Ok(());
    }

    let user = match load_bound_telegram_user(state, message.chat.id).await? {
        Some(user) => user,
        None => {
            send_plain_message(token, message.chat.id, "请先绑定账号。").await?;
            return Ok(());
        }
    };

    sqlx::query("UPDATE v2_user SET telegram_id = NULL, updated_at = ? WHERE id = ?")
        .bind(Utc::now().timestamp())
        .bind(user.id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    send_plain_message(token, message.chat.id, "解绑成功。").await?;
    Ok(())
}

async fn handle_ticket_reply_message(
    state: &AppState,
    token: &str,
    message: &TelegramMessage,
) -> Result<bool, Response<Body>> {
    if !ensure_private_chat(token, &message.chat).await? {
        return Ok(true);
    }

    let Some(reply_to) = message.reply_to_message.as_ref() else {
        return Ok(false);
    };
    let reply_text = reply_to.text.as_deref().unwrap_or("");
    let Some(ticket_id) = extract_ticket_id(reply_text) else {
        return Ok(false);
    };

    let user = match load_bound_telegram_user(state, message.chat.id).await? {
        Some(user) => user,
        None => {
            send_plain_message(token, message.chat.id, "请先绑定账号。").await?;
            return Ok(true);
        }
    };

    let ticket = load_user_ticket_by_id(state, user.id, ticket_id)
        .await
        .map_err(internal_error)?;
    let Some(ticket) = ticket else {
        send_plain_message(token, message.chat.id, "工单不存在。").await?;
        return Ok(true);
    };
    if ticket.status != 0 {
        send_plain_message(token, message.chat.id, "该工单已关闭，无法继续回复。").await?;
        return Ok(true);
    }

    let last_message = load_last_ticket_message(state, ticket.id, false)
        .await
        .map_err(internal_error)?;
    if let Some(last_message) = last_message {
        if last_message.user_id == user.id {
            send_plain_message(token, message.chat.id, "请等待技术人员回复后再继续发送。").await?;
            return Ok(true);
        }
    }

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let locked_ticket = load_user_ticket_by_id_for_update(&mut tx, user.id, ticket_id)
        .await
        .map_err(internal_error)?;
    let Some(locked_ticket) = locked_ticket else {
        tx.rollback().await.ok();
        send_plain_message(token, message.chat.id, "工单不存在。").await?;
        return Ok(true);
    };
    if locked_ticket.status != 0 {
        tx.rollback().await.ok();
        send_plain_message(token, message.chat.id, "该工单已关闭，无法继续回复。").await?;
        return Ok(true);
    }

    let last_message = load_last_ticket_message_with_tx(&mut tx, ticket_id)
        .await
        .map_err(internal_error)?;
    if let Some(last_message) = last_message {
        if last_message.user_id == user.id {
            tx.rollback().await.ok();
            send_plain_message(token, message.chat.id, "请等待技术人员回复后再继续发送。").await?;
            return Ok(true);
        }
    }

    let content = message.text.as_deref().unwrap_or("").trim();
    if content.is_empty() {
        tx.rollback().await.ok();
        send_plain_message(token, message.chat.id, "消息不能为空。").await?;
        return Ok(true);
    }

    let now = Utc::now().timestamp();
    sqlx::query(
        "INSERT INTO v2_ticket_message
            (user_id, ticket_id, message, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(user.id)
    .bind(ticket_id)
    .bind(content)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;

    sqlx::query(
        "UPDATE v2_ticket
         SET status = 0, reply_status = 1, last_reply_user_id = ?, updated_at = ?
         WHERE id = ? AND user_id = ?",
    )
    .bind(user.id)
    .bind(now)
    .bind(ticket_id)
    .bind(user.id)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;
    tx.commit().await.map_err(internal_error)?;

    send_plain_message(token, message.chat.id, &format!("工单 #{} 回复成功。", ticket_id)).await?;
    Ok(true)
}

async fn handle_tickets_command(
    state: &AppState,
    token: &str,
    message: &TelegramMessage,
) -> Result<(), Response<Body>> {
    if !ensure_private_chat(token, &message.chat).await? {
        return Ok(());
    }
    let user = match load_bound_telegram_user(state, message.chat.id).await? {
        Some(user) => user,
        None => {
            send_plain_message(token, message.chat.id, "请先绑定账号。").await?;
            return Ok(());
        }
    };
    let buttons = build_ticket_menu_buttons(state, &user).await.map_err(internal_error)?;
    if buttons.is_empty() {
        send_plain_message(token, message.chat.id, "当前账号未启用 Telegram 工单处理。").await?;
        return Ok(());
    }
    send_message_with_reply_markup(
        token,
        message.chat.id,
        "请选择要查看的工单视图。",
        &json!({ "inline_keyboard": buttons }),
    )
    .await?;
    Ok(())
}

async fn handle_ticket_detail_command(
    state: &AppState,
    token: &str,
    message: &TelegramMessage,
    args: &[String],
) -> Result<(), Response<Body>> {
    if !ensure_private_chat(token, &message.chat).await? {
        return Ok(());
    }
    let user = match load_bound_telegram_user(state, message.chat.id).await? {
        Some(user) => user,
        None => {
            send_plain_message(token, message.chat.id, "请先绑定账号。").await?;
            return Ok(());
        }
    };

    let ticket_id = args
        .first()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0);
    let Some(ticket_id) = ticket_id else {
        send_plain_message(token, message.chat.id, "用法：/ticket 工单ID").await?;
        return Ok(());
    };

    let ticket = load_user_ticket_by_id(state, user.id, ticket_id)
        .await
        .map_err(internal_error)?;
    let Some(ticket) = ticket else {
        send_plain_message(token, message.chat.id, "工单不存在。").await?;
        return Ok(());
    };
    let messages = load_ticket_messages(state, ticket.id)
        .await
        .map_err(internal_error)?;

    let mut lines = vec![
        format!("工单 #{}", ticket.id),
        format!("主题：{}", ticket.subject),
        format!("状态：{}", if ticket.status == 1 { "已关闭" } else { "处理中" }),
        String::new(),
        "最近消息：".to_string(),
    ];
    for item in messages.iter().rev().take(6).rev() {
        lines.push(format!(
            "[{}] {}",
            if item.user_id == user.id { "我" } else { "管理员" },
            limit_text(&item.message, 120)
        ));
    }
    if ticket.status == 0 {
        lines.push(String::new());
        lines.push("直接回复此消息即可继续工单沟通。".to_string());
        lines.push("使用 /close 工单ID 可关闭工单。".to_string());
    }

    send_plain_message(token, message.chat.id, &lines.join("\n")).await?;
    Ok(())
}

async fn handle_ticket_close_command(
    state: &AppState,
    token: &str,
    message: &TelegramMessage,
    args: &[String],
) -> Result<(), Response<Body>> {
    if !ensure_private_chat(token, &message.chat).await? {
        return Ok(());
    }
    let user = match load_bound_telegram_user(state, message.chat.id).await? {
        Some(user) => user,
        None => {
            send_plain_message(token, message.chat.id, "请先绑定账号。").await?;
            return Ok(());
        }
    };

    let ticket_id = args
        .first()
        .and_then(|value| value.parse::<i64>().ok())
        .filter(|value| *value > 0);
    let Some(ticket_id) = ticket_id else {
        send_plain_message(token, message.chat.id, "用法：/close 工单ID").await?;
        return Ok(());
    };

    let ticket = load_user_ticket_by_id(state, user.id, ticket_id)
        .await
        .map_err(internal_error)?;
    let Some(_) = ticket else {
        send_plain_message(token, message.chat.id, "工单不存在。").await?;
        return Ok(());
    };

    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let locked = load_user_ticket_by_id_for_update(&mut tx, user.id, ticket_id)
        .await
        .map_err(internal_error)?;
    let Some(locked) = locked else {
        tx.rollback().await.ok();
        send_plain_message(token, message.chat.id, "工单不存在。").await?;
        return Ok(());
    };
    if locked.status != 1 {
        sqlx::query("UPDATE v2_ticket SET status = 1, updated_at = ? WHERE id = ? AND user_id = ?")
            .bind(Utc::now().timestamp())
            .bind(ticket_id)
            .bind(user.id)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }
    tx.commit().await.map_err(internal_error)?;
    send_plain_message(token, message.chat.id, &format!("工单 #{} 已关闭。", ticket_id)).await?;
    Ok(())
}

async fn send_start_message(
    state: &AppState,
    token: &str,
    message: &TelegramMessage,
) -> Result<(), Response<Body>> {
    if !ensure_private_chat(token, &message.chat).await? {
        send_plain_message(token, message.chat.id, "请在私聊中使用此命令。").await?;
        return Ok(());
    }

    let welcome_title = telegram_plugin_string(state, "start_welcome_title", "欢迎使用 XBoard Telegram Bot").await;
    let bot_description = telegram_plugin_string(
        state,
        "start_bot_description",
        "我是您的专属助手，可以帮助您绑定账号、查看流量和获取订阅链接。",
    )
    .await;
    let footer = telegram_plugin_string(state, "start_footer", "提示：请在私聊中使用所有命令。").await;

    let mut text = format!("{}\n\n{}\n\n", welcome_title, bot_description);
    if let Some(user) = load_bound_telegram_user(state, message.chat.id).await? {
        text.push_str(&format!(
            "已绑定账号：{}\n可用命令：\n/traffic\n/getlatesturl\n/unbind",
            user.email
        ));
    } else {
        let bind_guide = telegram_plugin_string(
            state,
            "start_bind_guide",
            "请先绑定您的 XBoard 账号：\n1. 登录站点\n2. 复制订阅链接\n3. 发送 /bind + 订阅链接",
        )
        .await;
        text.push_str(&bind_guide);
        text.push_str("\n\n/bind [订阅链接] - 绑定账号");
    }
    text.push_str("\n\n");
    text.push_str(&footer);

    send_plain_message(token, message.chat.id, &text).await?;
    Ok(())
}

async fn send_help_message(
    state: &AppState,
    token: &str,
    chat_id: i64,
) -> Result<(), Response<Body>> {
    let help_text = telegram_plugin_string(
        state,
        "help_text",
        "未知命令，可用命令：\n/start\n/bind 订阅链接\n/traffic\n/getlatesturl\n/tickets\n/ticket 工单ID\n/close 工单ID\n/unbind",
    )
    .await;
    send_plain_message(token, chat_id, &help_text).await
}

async fn handle_chat_join_request(
    state: &AppState,
    token: &str,
    join_request: TelegramChatJoinRequest,
) -> Result<(), Response<Body>> {
    let Some(from) = join_request.from else {
        return Ok(());
    };

    let user = load_bound_telegram_user(state, from.id).await?;
    let Some(user) = user else {
        telegram_api_request(
            state,
            token,
            "declineChatJoinRequest",
            vec![
                ("chat_id".to_string(), join_request.chat.id.to_string()),
                ("user_id".to_string(), from.id.to_string()),
            ],
            true,
        )
        .await?;
        return Ok(());
    };

    let available = user.banned == 0
        && user.transfer_enable > 0
        && user.expired_at.map(|value| value > Utc::now().timestamp()).unwrap_or(true);
    let method = if available {
        "approveChatJoinRequest"
    } else {
        "declineChatJoinRequest"
    };
    telegram_api_request(
        state,
        token,
        method,
        vec![
            ("chat_id".to_string(), join_request.chat.id.to_string()),
            ("user_id".to_string(), from.id.to_string()),
        ],
        true,
    )
    .await?;
    Ok(())
}

async fn handle_callback_query(
    state: &AppState,
    token: &str,
    callback_query: TelegramCallbackQuery,
) -> Result<(), Response<Body>> {
    let Some(message) = callback_query.message.as_ref() else {
        return Ok(());
    };
    if message.chat.chat_type != "private" {
        answer_callback_query(state, token, &callback_query.id, "请在私聊中使用", false).await?;
        return Ok(());
    }

    let user = match load_bound_telegram_user(state, message.chat.id).await? {
        Some(user) => user,
        None => {
            answer_callback_query(state, token, &callback_query.id, "请先绑定账号", false).await?;
            return Ok(());
        }
    };

    let callback_data = callback_query.data.as_deref().unwrap_or("");
    let parts = callback_data.split(':').collect::<Vec<_>>();
    if parts.first().copied() != Some("tk") {
        return Ok(());
    }

    match parts.get(1).copied().unwrap_or_default() {
        "list" => {
            let scope = parts.get(2).copied().unwrap_or("mine");
            send_ticket_list_message(state, token, message.chat.id, &user, scope).await?;
            answer_callback_query(state, token, &callback_query.id, "已刷新工单列表", false).await?;
        }
        "view" => {
            let ticket_id = parts.get(2).and_then(|value| value.parse::<i64>().ok()).unwrap_or(0);
            if ticket_id <= 0 {
                answer_callback_query(state, token, &callback_query.id, "工单不存在", true).await?;
                return Ok(());
            }
            send_ticket_detail_message(state, token, message.chat.id, &user, ticket_id, None).await?;
            answer_callback_query(state, token, &callback_query.id, "已打开工单详情", false).await?;
        }
        "close" => {
            let ticket_id = parts.get(2).and_then(|value| value.parse::<i64>().ok()).unwrap_or(0);
            if ticket_id <= 0 {
                answer_callback_query(state, token, &callback_query.id, "工单不存在", true).await?;
                return Ok(());
            }
            close_ticket_for_user(state, user.id, ticket_id).await?;
            answer_callback_query(state, token, &callback_query.id, "工单已关闭", false).await?;
            send_ticket_detail_message(state, token, message.chat.id, &user, ticket_id, Some("工单已关闭。")).await?;
        }
        _ => {}
    }

    Ok(())
}

async fn load_bound_telegram_user(
    state: &AppState,
    telegram_id: i64,
) -> Result<Option<BearerUserRow>, Response<Body>> {
    sqlx::query_as::<_, BearerUserRow>(
        "SELECT id, invite_user_id, email, transfer_enable, last_login_at, created_at, banned, ban_reason,
                remind_expire, remind_traffic, expired_at, balance, commission_balance, plan_id,
                discount, commission_rate, telegram_id, uuid, is_admin, is_super_admin, trust_level,
                is_silenced, linux_do_id, linux_do_username, linux_do_name, linux_do_avatar, api_key,
                concurrent_ip_limit, token, subscribe_path, subscribe_key, subscribe_salt, u, d,
                device_limit, speed_limit, next_reset_at
         FROM v2_user WHERE telegram_id = ? LIMIT 1",
    )
    .bind(telegram_id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)
}

async fn load_bearer_user_by_token(
    state: &AppState,
    token: &str,
) -> Result<Option<BearerUserRow>, sqlx::Error> {
    sqlx::query_as::<_, BearerUserRow>(
        "SELECT id, invite_user_id, email, transfer_enable, last_login_at, created_at, banned, ban_reason,
                remind_expire, remind_traffic, expired_at, balance, commission_balance, plan_id,
                discount, commission_rate, telegram_id, uuid, is_admin, is_super_admin, trust_level,
                is_silenced, linux_do_id, linux_do_username, linux_do_name, linux_do_avatar, api_key,
                concurrent_ip_limit, token, subscribe_path, subscribe_key, subscribe_salt, u, d,
                device_limit, speed_limit, next_reset_at
         FROM v2_user WHERE token = ? LIMIT 1",
    )
    .bind(token)
    .fetch_optional(&state.db)
    .await
}

async fn resolve_telegram_bot_token(state: &AppState) -> String {
    let configured = get_setting_string(state, "telegram_bot_token", "").await;
    if !configured.trim().is_empty() {
        configured
    } else {
        std::env::var("TELEGRAM_BOT_TOKEN").unwrap_or_default()
    }
}

fn telegram_webhook_authorized(
    headers: &HeaderMap,
    token: &str,
) -> bool {
    let provided_secret = headers
        .get("x-telegram-bot-api-secret-token")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    crate::telegram_security_support::telegram_webhook_secret_matches(
        provided_secret,
        token,
        &std::env::var("APP_KEY").unwrap_or_default(),
    )
}

async fn telegram_plugin_string(
    state: &AppState,
    key: &str,
    default: &str,
) -> String {
    let raw = sqlx::query_scalar::<_, Option<String>>(
        "SELECT CAST(config AS CHAR) AS config
         FROM v2_plugins
         WHERE code = 'telegram'
         LIMIT 1",
    )
    .fetch_optional(&state.db)
    .await
    .ok()
    .flatten()
    .flatten();

    raw.and_then(|value| serde_json::from_str::<Value>(&value).ok())
        .and_then(|value| value.get(key).and_then(Value::as_str).map(|value| value.to_string()))
        .unwrap_or_else(|| default.to_string())
        .replace("\\n", "\n")
}

async fn send_plain_message(
    token: &str,
    chat_id: i64,
    text: &str,
) -> Result<(), Response<Body>> {
    let api_base = std::env::var("TELEGRAM_API_BASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "https://api.telegram.org/bot".to_string());
    let target = format!("{api_base}{token}/sendMessage");
    let request = Request::builder()
        .method(Method::POST)
        .uri(target)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "chat_id={}&text={}",
            chat_id,
            urlencoding::encode(text)
        )))
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("build telegram request failed: {err}")))?;

    let client = Client::builder(TokioExecutor::new()).build_http();
    let response = client
        .request(request)
        .await
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("telegram request failed: {err}")))?;
    if !response.status().is_success() {
        return Err(json_status_response(
            StatusCode::BAD_GATEWAY,
            json!({"message":"Telegram API 请求失败"}),
        ));
    }
    Ok(())
}

async fn telegram_api_request(
    state: &AppState,
    token: &str,
    method: &str,
    form_fields: Vec<(String, String)>,
    allow_failure: bool,
) -> Result<Value, Response<Body>> {
    let api_base = std::env::var("TELEGRAM_API_BASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "https://api.telegram.org/bot".to_string());
    let target = format!("{api_base}{token}/{method}");
    let uri: Uri = target
        .parse()
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("invalid telegram uri: {err}")))?;
    let payload = form_fields
        .into_iter()
        .map(|(key, value)| format!("{}={}", urlencoding::encode(&key), urlencoding::encode(&value)))
        .collect::<Vec<_>>()
        .join("&");
    let request = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(payload))
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("build telegram request failed: {err}")))?;

    let response = state
        .backend_client
        .request(request)
        .await
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("telegram request failed: {err}")))?;
    let status = response.status();
    let bytes = response_body_bytes(map_proxy_response(response).await).await;
    let body: Value = serde_json::from_slice(&bytes)
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("invalid telegram response: {err}")))?;

    if !status.is_success() && !allow_failure {
        return Err(json_status_response(
            StatusCode::BAD_GATEWAY,
            json!({"message": format!("Telegram API 请求失败: HTTP {}", status.as_u16())}),
        ));
    }
    if body.get("ok").and_then(Value::as_bool) == Some(false) && !allow_failure {
        let description = body
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("未知错误");
        return Err(json_status_response(
            StatusCode::BAD_GATEWAY,
            json!({"message": format!("Telegram API 错误: {}", description)}),
        ));
    }
    Ok(body)
}

async fn answer_callback_query(
    state: &AppState,
    token: &str,
    callback_query_id: &str,
    text: &str,
    show_alert: bool,
) -> Result<(), Response<Body>> {
    let _ = telegram_api_request(
        state,
        token,
        "answerCallbackQuery",
        vec![
            ("callback_query_id".to_string(), callback_query_id.to_string()),
            ("text".to_string(), text.to_string()),
            ("show_alert".to_string(), if show_alert { "true".to_string() } else { "false".to_string() }),
        ],
        true,
    )
    .await?;
    Ok(())
}

async fn send_message_with_reply_markup(
    token: &str,
    chat_id: i64,
    text: &str,
    reply_markup: &Value,
) -> Result<(), Response<Body>> {
    let api_base = std::env::var("TELEGRAM_API_BASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "https://api.telegram.org/bot".to_string());
    let target = format!("{api_base}{token}/sendMessage");
    let request = Request::builder()
        .method(Method::POST)
        .uri(target)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(format!(
            "chat_id={}&text={}&reply_markup={}",
            chat_id,
            urlencoding::encode(text),
            urlencoding::encode(&reply_markup.to_string())
        )))
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("build telegram request failed: {err}")))?;

    let client = Client::builder(TokioExecutor::new()).build_http();
    let response = client
        .request(request)
        .await
        .map_err(|err| json_error(StatusCode::BAD_GATEWAY, &format!("telegram request failed: {err}")))?;
    if !response.status().is_success() {
        return Err(json_status_response(
            StatusCode::BAD_GATEWAY,
            json!({"message":"Telegram API 请求失败"}),
        ));
    }
    Ok(())
}

async fn ensure_private_chat(token: &str, chat: &TelegramChat) -> Result<bool, Response<Body>> {
    if chat.chat_type == "private" {
        return Ok(true);
    }
    let _ = send_plain_message(token, chat.id, "请在私聊中使用此命令。").await;
    Ok(false)
}

fn normalize_command(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Some((command, _suffix)) = trimmed.split_once('@') {
        command.to_string()
    } else {
        trimmed.to_string()
    }
}

async fn extract_subscribe_token(
    state: &AppState,
    url: &str,
) -> Result<Option<String>, Response<Body>> {
    let parsed = http::Uri::try_from(url).ok();
    if let Some(parsed) = parsed {
        if let Some(query) = parsed.query() {
            let params = parse_query_string(query);
            if let Some(token) = params.get("token").cloned() {
                return Ok(Some(token));
            }
            if let Some(path_key) = parsed.path().rsplit('/').next().filter(|value| !value.is_empty()) {
                if let Some(token) = load_token_from_subscribe_parts(state, path_key, &params).await? {
                    return Ok(Some(token));
                }
            }
        }
        if let Some(path) = parsed.path().rsplit('/').next() {
            if !path.is_empty() {
                return Ok(Some(path.to_string()));
            }
        }
    }

    let query_part = url.split('?').nth(1).unwrap_or_default();
    let params = parse_query_string(query_part);
    if let Some(token) = params.get("token").cloned() {
        return Ok(Some(token));
    }
    Ok(url.trim_matches('/').rsplit('/').next().map(|value| value.to_string()))
}

fn transfer_to_gb_string(value: f64) -> String {
    format!("{:.2}", value / 1024.0 / 1024.0 / 1024.0)
}

async fn build_ticket_menu_buttons(
    state: &AppState,
    user: &BearerUserRow,
) -> Result<Vec<Vec<Value>>, sqlx::Error> {
    let mut buttons = Vec::new();
    if get_setting_bool(state, "telegram_user_ticket_enable", true).await {
        buttons.push(vec![json!({
            "text": "我的工单",
            "callback_data": "tk:list:mine"
        })]);
    }

    let assigned_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM v2_ticket WHERE assigned_admin_user_id = ?"
    )
    .bind(user.id)
    .fetch_one(&state.db)
    .await?;
    if assigned_count > 0 {
        buttons.push(vec![json!({
            "text": "指派给我",
            "callback_data": "tk:list:assigned"
        })]);
    }

    if user.is_admin != 0 || user.is_super_admin != 0 {
        buttons.push(vec![json!({
            "text": "后台待处理",
            "callback_data": "tk:list:admin"
        })]);
    }

    Ok(buttons)
}

async fn send_ticket_list_message(
    state: &AppState,
    token: &str,
    chat_id: i64,
    user: &BearerUserRow,
    scope: &str,
) -> Result<(), Response<Body>> {
    let scope = scope.trim().to_lowercase();
    let tickets = match scope.as_str() {
        "assigned" => load_assigned_admin_tickets(state, user.id, None)
            .await
            .map_err(internal_error)?,
        "admin" if user.is_admin != 0 || user.is_super_admin != 0 => {
            sqlx::query_as::<_, TicketRow>(
                "SELECT id, user_id, node_id, assigned_admin_user_id, subject, level, status, reply_status, last_reply_user_id, created_at, updated_at
                 FROM v2_ticket ORDER BY status ASC, updated_at DESC LIMIT 8"
            )
            .fetch_all(&state.db)
            .await
            .map_err(internal_error)?
        }
        _ => load_user_tickets(state, user.id, None)
            .await
            .map_err(internal_error)?,
    };

    if tickets.is_empty() {
        send_plain_message(token, chat_id, "当前视图暂无工单。").await?;
        return Ok(());
    }

    let mut lines = vec![format!("工单列表 - {}", humanize_ticket_scope(&scope)), String::new()];
    let mut rows = Vec::new();
    for ticket in tickets.iter().take(8) {
        lines.push(format!(
            "#{} [{}] {}",
            ticket.id,
            if ticket.status == 1 { "已关闭" } else { "处理中" },
            limit_text(&ticket.subject, 48)
        ));
        rows.push(vec![json!({
            "text": format!("#{} {}", ticket.id, limit_text(&ticket.subject, 18)),
            "callback_data": format!("tk:view:{}", ticket.id),
        })]);
    }
    let mut menu = build_ticket_menu_buttons(state, user).await.map_err(internal_error)?;
    rows.append(&mut menu);
    send_message_with_reply_markup(
        token,
        chat_id,
        &lines.join("\n"),
        &json!({ "inline_keyboard": rows }),
    )
    .await?;
    Ok(())
}

async fn send_ticket_detail_message(
    state: &AppState,
    token: &str,
    chat_id: i64,
    user: &BearerUserRow,
    ticket_id: i64,
    prefix: Option<&str>,
) -> Result<(), Response<Body>> {
    let ticket = load_user_ticket_by_id(state, user.id, ticket_id)
        .await
        .map_err(internal_error)?;
    let Some(ticket) = ticket else {
        send_plain_message(token, chat_id, "工单不存在。").await?;
        return Ok(());
    };
    let messages = load_ticket_messages(state, ticket.id)
        .await
        .map_err(internal_error)?;

    let mut lines = Vec::new();
    if let Some(prefix) = prefix {
        lines.push(prefix.to_string());
        lines.push(String::new());
    }
    lines.push(format!("工单 #{}", ticket.id));
    lines.push(format!("主题：{}", ticket.subject));
    lines.push(format!("状态：{}", if ticket.status == 1 { "已关闭" } else { "处理中" }));
    lines.push(String::new());
    lines.push("最近消息：".to_string());
    for item in messages.iter().rev().take(6).rev() {
        lines.push(format!(
            "[{}] {}",
            if item.user_id == user.id { "我" } else { "管理员" },
            limit_text(&item.message, 120)
        ));
    }
    if ticket.status == 0 {
        lines.push(String::new());
        lines.push("直接回复此消息即可继续工单沟通。".to_string());
    }

    let buttons = build_ticket_action_buttons(&ticket);
    send_message_with_reply_markup(
        token,
        chat_id,
        &lines.join("\n"),
        &json!({ "inline_keyboard": buttons }),
    )
    .await?;
    Ok(())
}

fn build_ticket_action_buttons(ticket: &TicketRow) -> Vec<Vec<Value>> {
    let mut rows = vec![vec![json!({
        "text": "刷新详情",
        "callback_data": format!("tk:view:{}", ticket.id),
    })]];
    if ticket.status != 1 {
        rows.push(vec![json!({
            "text": "关闭工单",
            "callback_data": format!("tk:close:{}", ticket.id),
        })]);
    }
    rows
}

async fn close_ticket_for_user(
    state: &AppState,
    user_id: i64,
    ticket_id: i64,
) -> Result<(), Response<Body>> {
    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let locked = load_user_ticket_by_id_for_update(&mut tx, user_id, ticket_id)
        .await
        .map_err(internal_error)?;
    let Some(locked) = locked else {
        tx.rollback().await.ok();
        return Err(fail_json_response(StatusCode::BAD_REQUEST, "Ticket does not exist"));
    };
    if locked.status != 1 {
        sqlx::query("UPDATE v2_ticket SET status = 1, updated_at = ? WHERE id = ? AND user_id = ?")
            .bind(Utc::now().timestamp())
            .bind(ticket_id)
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
    }
    tx.commit().await.map_err(internal_error)?;
    Ok(())
}

fn humanize_ticket_scope(scope: &str) -> &'static str {
    match scope {
        "assigned" => "指派给我",
        "admin" => "后台待处理",
        _ => "我的工单",
    }
}

fn limit_text(text: &str, max_chars: usize) -> String {
    let trimmed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let chars = trimmed.chars().collect::<Vec<_>>();
    if chars.len() <= max_chars {
        return trimmed;
    }
    chars.into_iter().take(max_chars.saturating_sub(1)).collect::<String>() + "…"
}

fn extract_ticket_id(text: &str) -> Option<i64> {
    let marker = "工单 #";
    let start = text.find(marker)? + marker.len();
    let digits = text[start..]
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    digits.parse::<i64>().ok().filter(|value| *value > 0)
}

fn parse_query_string(query: &str) -> HashMap<String, String> {
    query
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?;
            let value = parts.next().unwrap_or_default();
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}

async fn load_token_from_subscribe_parts(
    state: &AppState,
    subscribe_path: &str,
    params: &HashMap<String, String>,
) -> Result<Option<String>, Response<Body>> {
    let row = sqlx::query(
        "SELECT token, subscribe_key, subscribe_salt
         FROM v2_user
         WHERE subscribe_path = ?
         LIMIT 1",
    )
    .bind(subscribe_path)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?;
    let Some(row) = row else {
        return Ok(None);
    };

    let token = row.try_get::<String, _>("token").ok().unwrap_or_default();
    let subscribe_key = row
        .try_get::<Option<String>, _>("subscribe_key")
        .ok()
        .flatten()
        .unwrap_or_default();
    let subscribe_salt = row
        .try_get::<Option<String>, _>("subscribe_salt")
        .ok()
        .flatten()
        .unwrap_or_default();

    if subscribe_key.is_empty() || subscribe_salt.is_empty() {
        return Ok(None);
    }
    if params.get(&subscribe_key).map(|value| value.as_str()) != Some(token.as_str()) {
        return Ok(None);
    }
    if !params.contains_key(&subscribe_salt) {
        return Ok(None);
    }
    Ok(Some(token))
}
