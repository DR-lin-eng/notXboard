use crate::*;

pub(crate) async fn notify_payment_success_by_order_id(
    state: &AppState,
    order_id: i64,
) -> Result<i64, String> {
    if !get_setting_bool(state, "telegram_notify_payment_success", true).await {
        return Ok(0);
    }

    let row = sqlx::query(
        "SELECT o.id, o.trade_no, o.user_id, o.plan_id, o.total_amount, o.payment_id,
                u.email AS user_email, u.telegram_id,
                p.name AS plan_name,
                pm.name AS payment_name
         FROM v2_order o
         LEFT JOIN v2_user u ON u.id = o.user_id
         LEFT JOIN v2_plan p ON p.id = o.plan_id
         LEFT JOIN v2_payment pm ON pm.id = o.payment_id
         WHERE o.id = ?
         LIMIT 1",
    )
    .bind(order_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| format!("load payment notify order failed: {err}"))?;
    let Some(row) = row else {
        return Ok(0);
    };

    let trade_no = row.try_get::<String, _>("trade_no").unwrap_or_default();
    let total_amount = row.try_get::<i64, _>("total_amount").unwrap_or_default();
    let user_email = row
        .try_get::<Option<String>, _>("user_email")
        .ok()
        .flatten()
        .unwrap_or_else(|| "-".to_string());
    let user_telegram_id = row.try_get::<Option<i64>, _>("telegram_id").ok().flatten();
    let plan_name = row
        .try_get::<Option<String>, _>("plan_name")
        .ok()
        .flatten()
        .unwrap_or_else(|| "-".to_string());
    let payment_name = row
        .try_get::<Option<String>, _>("payment_name")
        .ok()
        .flatten()
        .unwrap_or_else(|| "-".to_string());

    let message = format!(
        "支付成功\n订单号：{}\n金额：{:.2} 元\n套餐：{}\n支付渠道：{}",
        trade_no,
        (total_amount as f64) / 100.0,
        plan_name,
        payment_name
    );

    let mut sent = 0_i64;
    if let Some(chat_id) = user_telegram_id {
        sent += send_telegram_text_to_chat_ids(
            state,
            &[chat_id],
            &message,
            "telegram_notify_payment_success",
        )
        .await?;
    }

    let admin_message = format!("用户支付成功\n用户：{}\n{}", user_email, message);
    sent += send_telegram_text_to_admins(
        state,
        &admin_message,
        "telegram_notify_payment_success",
    )
    .await?;

    Ok(sent)
}
