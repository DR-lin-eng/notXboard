use crate::*;

#[derive(Clone)]
pub(crate) struct EpayConfig {
    pub(crate) pid: String,
    pub(crate) key: String,
    pub(crate) url: String,
    pub(crate) submit_path: String,
    pub(crate) use_post: bool,
    pub(crate) sitename: Option<String>,
    pub(crate) device: Option<String>,
}

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct CheckoutOrderRow {
    pub(crate) id: i64,
    pub(crate) trade_no: String,
    pub(crate) user_id: i64,
    pub(crate) plan_id: i64,
    pub(crate) total_amount: i64,
    pub(crate) handling_amount: Option<i64>,
    pub(crate) payment_id: Option<i64>,
    pub(crate) status: i64,
}

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct PaymentMethodRow {
    pub(crate) id: i64,
    pub(crate) uuid: String,
    pub(crate) name: String,
    pub(crate) payment: String,
    pub(crate) icon: Option<String>,
    pub(crate) handling_fee_fixed: Option<i64>,
    pub(crate) handling_fee_percent: Option<f64>,
    pub(crate) config: Option<String>,
    pub(crate) enable: bool,
}

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct PaymentNotifyRow {
    pub(crate) id: i64,
    pub(crate) uuid: String,
    pub(crate) payment: String,
    pub(crate) enable: bool,
}

pub(crate) async fn find_user_checkout_order(
    state: &AppState,
    user_id: i64,
    trade_no: &str,
) -> Result<Option<CheckoutOrderRow>, sqlx::Error> {
    if trade_no.is_empty() {
        return Ok(None);
    }
    sqlx::query_as::<_, CheckoutOrderRow>(
        "SELECT id, trade_no, user_id, plan_id, total_amount, handling_amount, payment_id, status
         FROM v2_order WHERE user_id = ? AND trade_no = ? LIMIT 1"
    )
    .bind(user_id)
    .bind(trade_no)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn load_payment_notify_method_by_uuid(
    state: &AppState,
    uuid: &str,
) -> Result<Option<PaymentNotifyRow>, sqlx::Error> {
    sqlx::query_as::<_, PaymentNotifyRow>(
        "SELECT id, uuid, payment, enable FROM v2_payment WHERE uuid = ? LIMIT 1",
    )
    .bind(uuid)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn load_order_epay_config_snapshot(
    state: &AppState,
    trade_no: &str,
) -> Result<Option<EpayConfig>, String> {
    if trade_no.trim().is_empty() {
        return Ok(None);
    }
    if !order_epay_snapshot_columns_ready(state)
        .await
        .map_err(|err| format!("check order epay snapshot columns failed: {err}"))?
    {
        return Ok(None);
    }

    let row = sqlx::query(
        "SELECT epay_pid, epay_url, epay_key_encrypted
         FROM v2_order
         WHERE trade_no = ?
         LIMIT 1",
    )
    .bind(trade_no)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| format!("load order epay snapshot failed: {err}"))?;
    let Some(row) = row else {
        return Ok(None);
    };

    let pid = row
        .try_get::<Option<String>, _>("epay_pid")
        .ok()
        .flatten()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let url = row
        .try_get::<Option<String>, _>("epay_url")
        .ok()
        .flatten()
        .map(|value| value.trim_end_matches('/').to_string())
        .filter(|value| !value.is_empty());
    let key_encrypted = row
        .try_get::<Option<String>, _>("epay_key_encrypted")
        .ok()
        .flatten()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let (Some(pid), Some(url), Some(key_encrypted)) = (pid, url, key_encrypted) else {
        return Ok(None);
    };

    let key = match decrypt_laravel_string(&state.app_key, &key_encrypted) {
        Ok(key) => key,
        Err(err) => {
            tracing::warn!(
                trade_no,
                error = %err,
                "order epay snapshot decrypt failed; falling back to payment config"
            );
            return Ok(None);
        }
    };

    Ok(Some(EpayConfig {
        pid,
        key,
        url,
        submit_path: "/pay/submit.php".to_string(),
        use_post: true,
        sitename: None,
        device: None,
    }))
}

pub(crate) async fn order_epay_snapshot_columns_ready(state: &AppState) -> Result<bool, sqlx::Error> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM information_schema.columns
         WHERE table_schema = DATABASE()
           AND table_name = 'v2_order'
           AND column_name IN ('epay_pid', 'epay_url', 'epay_key_encrypted')",
    )
    .fetch_one(&state.db)
    .await?;
    Ok(count == 3)
}

pub(crate) fn verify_epay_notify_signature(
    params: &HashMap<String, String>,
    config: &EpayConfig,
) -> bool {
    let trade_status = params.get("trade_status").map(|v| v.to_uppercase());
    if let Some(status) = trade_status {
        if !matches!(status.as_str(), "TRADE_SUCCESS" | "TRADE_FINISHED" | "SUCCESS") {
            return false;
        }
    }

    let sign = params.get("sign").cloned().unwrap_or_default();
    if sign.is_empty() {
        return false;
    }

    let mut items = params
        .iter()
        .filter(|(k, v)| *k != "sign" && *k != "sign_type" && !v.is_empty())
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<Vec<_>>();
    items.sort_by(|a, b| a.0.cmp(&b.0));
    let payload = items
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");
    let expected = format!("{:x}", md5::compute(format!("{}{}", payload, config.key)));
    if std::env::var("EPAY_DEBUG_VERIFY").ok().as_deref() == Some("1") {
        eprintln!("EPAY VERIFY params={:?} payload={payload} expected={expected} sign={sign}", params);
    }
    crate::secure_compare_support::constant_time_eq_str(&sign, &expected)
}

pub(crate) async fn mark_order_paid_processing(
    state: &AppState,
    trade_no: &str,
    callback_no: &str,
) -> Result<Option<i64>, sqlx::Error> {
    let order = sqlx::query(
        "SELECT id, status, paid_at, callback_no FROM v2_order WHERE trade_no = ? LIMIT 1"
    )
    .bind(trade_no)
    .fetch_optional(&state.db)
    .await?;
    let Some(order) = order else {
        return Ok(None);
    };

    let order_id: i64 = order.try_get("id").unwrap_or_default();
    let status: i64 = order.try_get("status").unwrap_or_default();
    let paid_at: Option<i64> = order.try_get("paid_at").unwrap_or(None);
    let existing_callback: Option<String> = order.try_get("callback_no").unwrap_or(None);

    if status == 0 {
        sqlx::query(
            "UPDATE v2_order SET status = 1, paid_at = ?, callback_no = COALESCE(callback_no, ?), updated_at = ? WHERE id = ? AND status = 0"
        )
        .bind(paid_at.unwrap_or_else(|| Utc::now().timestamp()))
        .bind(if callback_no.is_empty() { None::<String> } else { Some(callback_no.to_string()) })
        .bind(Utc::now().timestamp())
        .bind(order_id)
        .execute(&state.db)
        .await?;
        return Ok(Some(order_id));
    }

    if status == 1 {
        if existing_callback.as_deref().unwrap_or("").is_empty() && !callback_no.is_empty() {
            sqlx::query("UPDATE v2_order SET callback_no = ?, updated_at = ? WHERE id = ?")
                .bind(callback_no)
                .bind(Utc::now().timestamp())
                .bind(order_id)
                .execute(&state.db)
                .await?;
        }
        return Ok(Some(order_id));
    }

    Ok(Some(order_id))
}

pub(crate) async fn complete_processing_order_by_id(
    state: &AppState,
    order_id: i64,
) -> Result<(), sqlx::Error> {
    let row = sqlx::query(
        "SELECT o.id, o.user_id, o.plan_id, o.period, o.paid_at, o.status,
                p.group_id, p.transfer_enable, p.speed_limit, p.device_limit, p.is_unlimited_traffic
         FROM v2_order o
         JOIN v2_plan p ON p.id = o.plan_id
         WHERE o.id = ?
         LIMIT 1"
    )
    .bind(order_id)
    .fetch_optional(&state.db)
    .await?;

    let Some(row) = row else {
        return Ok(());
    };

    let order_id: i64 = row.try_get("id").unwrap_or_default();
    let user_id: i64 = row.try_get("user_id").unwrap_or_default();
    let plan_id: i64 = row.try_get("plan_id").unwrap_or_default();
    let period: String = row.try_get("period").unwrap_or_default();
    let paid_at: i64 = row
        .try_get::<Option<i64>, _>("paid_at")
        .unwrap_or(None)
        .unwrap_or_else(|| Utc::now().timestamp());
    let status: i64 = row.try_get("status").unwrap_or_default();
    if status != 1 {
        return Ok(());
    }

    let group_id: Option<u64> = row.try_get("group_id").unwrap_or(None);
    let transfer_enable: u64 = row.try_get::<u64, _>("transfer_enable").unwrap_or_default();
    let speed_limit: Option<u64> = row.try_get("speed_limit").unwrap_or(None);
    let device_limit: Option<u64> = row.try_get("device_limit").unwrap_or(None);
    let is_unlimited_traffic: bool = row.try_get("is_unlimited_traffic").unwrap_or(false);
    let period_key = period.as_str();
    let expired_at = compute_period_expired_at(period_key, paid_at);
    let traffic_allowance_kb = if is_unlimited_traffic {
        8_000_000_000_000_000_i64
    } else {
        transfer_enable.min(i64::MAX as u64) as i64
    };

    let mut tx = state.db.begin().await?;
    sqlx::query(
        "INSERT INTO user_plan_subscriptions
            (user_id, plan_id, order_id, period, traffic_allowance_kb, used_traffic_kb, started_at, expired_at, status, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, 0, ?, ?, 1, ?, ?)
         ON DUPLICATE KEY UPDATE
            user_id = VALUES(user_id),
            plan_id = VALUES(plan_id),
            period = VALUES(period),
            traffic_allowance_kb = VALUES(traffic_allowance_kb),
            started_at = VALUES(started_at),
            expired_at = VALUES(expired_at),
            status = VALUES(status),
            updated_at = VALUES(updated_at)"
    )
    .bind(user_id)
    .bind(plan_id)
    .bind(order_id)
    .bind(period_key)
    .bind(traffic_allowance_kb)
    .bind(paid_at)
    .bind(expired_at)
    .bind(Utc::now().timestamp())
    .bind(Utc::now().timestamp())
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "UPDATE v2_user
         SET plan_id = ?, group_id = ?, transfer_enable = ?, expired_at = ?, speed_limit = ?, device_limit = ?, updated_at = ?
         WHERE id = ?"
    )
    .bind(plan_id)
    .bind(group_id.map(|v| v as i64))
    .bind(transfer_enable.min(i64::MAX as u64) as i64)
    .bind(expired_at)
    .bind(speed_limit.map(|v| v as i64))
    .bind(device_limit.map(|v| v as i64))
    .bind(Utc::now().timestamp())
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("UPDATE v2_order SET status = 3, updated_at = ? WHERE id = ? AND status = 1")
        .bind(Utc::now().timestamp())
        .bind(order_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    let _ = notify_payment_success_by_order_id(state, order_id).await;
    Ok(())
}

pub(crate) fn compute_period_expired_at(period: &str, started_at: i64) -> Option<i64> {
    let start = chrono::DateTime::from_timestamp(started_at, 0)?;
    let months = match period {
        "monthly" => Some(1),
        "quarterly" => Some(3),
        "half_yearly" => Some(6),
        "yearly" => Some(12),
        "two_yearly" => Some(24),
        "three_yearly" => Some(36),
        "onetime" => None,
        _ => Some(1),
    };
    months.map(|m| (start + chrono::Duration::days((m * 30) as i64)).timestamp())
}

pub(crate) async fn load_payment_config(
    state: &AppState,
    payment_id: i64,
) -> Result<EpayConfig, sqlx::Error> {
    let row = sqlx::query("SELECT config FROM v2_payment WHERE id = ? LIMIT 1")
        .bind(payment_id)
        .fetch_one(&state.db)
        .await?;
    let raw = row.try_get::<String, _>("config").unwrap_or_default();
    let value: Value = serde_json::from_str(&raw).unwrap_or(Value::Null);
    let obj = value.as_object().cloned().unwrap_or_default();
    Ok(EpayConfig {
        pid: obj.get("pid").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        key: obj.get("key").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        url: obj.get("url").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        submit_path: obj.get("submit_path").and_then(|v| v.as_str()).unwrap_or("/submit.php").to_string(),
        use_post: obj.get("use_post").and_then(|v| v.as_bool()).unwrap_or(false),
        sitename: obj.get("sitename").and_then(|v| v.as_str()).map(|v| v.to_string()),
        device: obj.get("device").and_then(|v| v.as_str()).map(|v| v.to_string()),
    })
}

pub(crate) fn epay_config_from_payment_method(payment: &PaymentMethodRow) -> EpayConfig {
    let raw = payment.config.clone().unwrap_or_default();
    let value: Value = serde_json::from_str(&raw).unwrap_or(Value::Null);
    let obj = value.as_object().cloned().unwrap_or_default();
    EpayConfig {
        pid: obj.get("pid").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        key: obj.get("key").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        url: obj.get("url").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        submit_path: obj.get("submit_path").and_then(|v| v.as_str()).unwrap_or("/submit.php").to_string(),
        use_post: obj.get("use_post").and_then(|v| v.as_bool()).unwrap_or(false),
        sitename: obj.get("sitename").and_then(|v| v.as_str()).map(|v| v.to_string()),
        device: obj.get("device").and_then(|v| v.as_str()).map(|v| v.to_string()),
    }
}

pub(crate) async fn load_payment_method_by_id(
    state: &AppState,
    payment_id: i64,
) -> Result<Option<PaymentMethodRow>, sqlx::Error> {
    sqlx::query_as::<_, PaymentMethodRow>(
        "SELECT id, uuid, name, payment, icon, handling_fee_fixed,
                CAST(handling_fee_percent AS DOUBLE) AS handling_fee_percent,
                CAST(config AS CHAR) AS config,
                enable
         FROM v2_payment
         WHERE id = ?
         LIMIT 1"
    )
    .bind(payment_id)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn load_user_epay_profile(
    state: &AppState,
    user_id: i64,
) -> Result<Option<EpayConfig>, String> {
    let row = sqlx::query_as::<_, UserEpayProfileRow>(
        "SELECT pid, key_encrypted, url, submit_path, use_post, sitename, device
         FROM user_payment_profiles
         WHERE user_id = ? AND provider = 'epay'
         LIMIT 1"
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|err| format!("load user epay profile failed: {err}"))?;
    let Some(row) = row else {
        return Ok(None);
    };
    let Some(pid) = row.pid.filter(|v| !v.trim().is_empty()) else {
        return Ok(None);
    };
    let Some(url) = row.url.filter(|v| !v.trim().is_empty()) else {
        return Ok(None);
    };
    let Some(key_encrypted) = row.key_encrypted.filter(|v| !v.trim().is_empty()) else {
        return Ok(None);
    };
    let key = decrypt_laravel_string(&state.app_key, &key_encrypted)?;
    let url = crate::url_security_support::normalize_http_url(&url, true)?;
    let submit_path = crate::url_security_support::normalize_relative_path(row.submit_path.as_deref(), "/pay/submit.php")?;
    Ok(Some(EpayConfig {
        pid,
        key,
        url,
        submit_path,
        use_post: row.use_post,
        sitename: row.sitename.filter(|v| !v.trim().is_empty()),
        device: row.device.filter(|v| !v.trim().is_empty()),
    }))
}

pub(crate) async fn validate_order_plan_for_user(
    state: &AppState,
    plan: &OrderPlanRow,
    user: &BearerUserRow,
    purchase_token: Option<&str>,
) -> Result<(), Response<Body>> {
    if let Some(min_trust_level) = plan.min_trust_level {
        if (user.trust_level.max(0) as u64) < min_trust_level {
            return Err(fail_json_response(StatusCode::BAD_REQUEST, "Insufficient trust level"));
        }
    }
    let has_active_subscription = has_active_subscription(state, user.id, plan.id).await.map_err(internal_error)?;
    if has_active_subscription {
        if !plan.renew {
            return Err(fail_json_response(StatusCode::BAD_REQUEST, "This subscription cannot be renewed, please change to another subscription"));
        }
        return Ok(());
    }
    if !plan.sell {
        return Err(fail_json_response(StatusCode::BAD_REQUEST, "This subscription has expired, please change to another subscription"));
    }
    if !has_plan_capacity(state, plan.id, plan.capacity_limit).await.map_err(internal_error)? {
        return Err(fail_json_response(StatusCode::BAD_REQUEST, "Current product is sold out"));
    }
    let scope = plan.visibility_scope.as_deref().unwrap_or("public").trim().to_lowercase();
    if scope == "link_only" {
        if purchase_token != plan.share_token.as_deref() {
            return Err(fail_json_response(StatusCode::BAD_REQUEST, "This subscription has been sold out, please choose another subscription"));
        }
    } else if scope == "assigned_only" {
        return Err(fail_json_response(StatusCode::BAD_REQUEST, "This subscription has been sold out, please choose another subscription"));
    } else if !plan.show {
        return Err(fail_json_response(StatusCode::BAD_REQUEST, "This subscription has been sold out, please choose another subscription"));
    }
    Ok(())
}

pub(crate) async fn has_active_subscription(
    state: &AppState,
    user_id: i64,
    plan_id: i64,
) -> Result<bool, sqlx::Error> {
    let now = Utc::now().timestamp();
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_plan_subscriptions
         WHERE user_id = ? AND plan_id = ? AND status = 1 AND (expired_at IS NULL OR expired_at > ?)"
    )
    .bind(user_id)
    .bind(plan_id)
    .bind(now)
    .fetch_one(&state.db)
    .await
    .map(|count| count > 0)
}

pub(crate) fn normalize_order_period(period: &str) -> Option<&'static str> {
    match period {
        "month_price" => Some("monthly"),
        "quarter_price" => Some("quarterly"),
        "half_year_price" => Some("half_yearly"),
        "year_price" => Some("yearly"),
        "two_year_price" => Some("two_yearly"),
        "three_year_price" => Some("three_yearly"),
        "onetime_price" => Some("onetime"),
        "reset_price" => Some("reset_traffic"),
        _ => None,
    }
}

pub(crate) fn convert_period_to_legacy_field(period: &str) -> &str {
    match period {
        "monthly" => "month_price",
        "quarterly" => "quarter_price",
        "half_yearly" => "half_year_price",
        "yearly" => "year_price",
        "two_yearly" => "two_year_price",
        "three_yearly" => "three_year_price",
        "onetime" => "onetime_price",
        "reset_traffic" => "reset_price",
        _ => period,
    }
}

pub(crate) fn normalize_coupon_period(period: &str) -> Option<&'static str> {
    match period.trim() {
        "monthly" | "month_price" => Some("monthly"),
        "quarterly" | "quarter_price" => Some("quarterly"),
        "half_yearly" | "half_year_price" => Some("half_yearly"),
        "yearly" | "year_price" => Some("yearly"),
        "two_yearly" | "two_year_price" => Some("two_yearly"),
        "three_yearly" | "three_year_price" => Some("three_yearly"),
        "onetime" | "onetime_price" => Some("onetime"),
        "reset_traffic" | "reset_price" => Some("reset_traffic"),
        _ => None,
    }
}

pub(crate) fn plan_price_for_period(plan: &OrderPlanRow, period: &str) -> Option<i64> {
    let prices = plan.prices.as_ref()?.0.as_object()?;
    let price = prices.get(period)?.as_f64()?;
    Some((price * 100.0).round() as i64)
}

pub(crate) async fn resolve_order_type(
    state: &AppState,
    user_id: i64,
    plan_id: i64,
) -> Result<i64, sqlx::Error> {
    if has_active_subscription(state, user_id, plan_id).await? {
        Ok(2)
    } else {
        Ok(1)
    }
}

pub(crate) fn compute_handling_amount(
    total_amount: i64,
    fixed: Option<i64>,
    percent: Option<f64>,
) -> i64 {
    let fixed = fixed.unwrap_or(0);
    let percent = percent.unwrap_or(0.0);
    ((total_amount as f64) * (percent / 100.0)).round() as i64 + fixed
}

pub(crate) fn build_epay_checkout_payload(
    order: &CheckoutOrderRow,
    amount: i64,
    config: &EpayConfig,
    notify_uuid: &str,
) -> Result<Value, Response<Body>> {
    let money = format!("{:.2}", (amount as f64) / 100.0);
    let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://127.0.0.1:18087".to_string());
    let notify_url = format!("{}/api/v1/guest/payment/notify/EPay/{}", app_url.trim_end_matches('/'), notify_uuid);
    let return_url = format!("{}/app#/order/{}", app_url.trim_end_matches('/'), order.trade_no);

    let mut params = vec![
        ("money".to_string(), money),
        ("name".to_string(), order.trade_no.clone()),
        ("notify_url".to_string(), notify_url),
        ("out_trade_no".to_string(), order.trade_no.clone()),
        ("pid".to_string(), config.pid.clone()),
        ("return_url".to_string(), return_url),
        ("type".to_string(), "epay".to_string()),
    ];
    if let Some(sitename) = &config.sitename {
        params.push(("sitename".to_string(), sitename.clone()));
    }
    if let Some(device) = &config.device {
        params.push(("device".to_string(), device.clone()));
    }
    params.sort_by(|a, b| a.0.cmp(&b.0));
    let payload = params
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");
    let sign = format!("{:x}", md5::compute(format!("{}{}", payload, config.key)));
    crate::epay_render_support::ensure_http_checkout_url(&config.url)?;
    let submit_url = format!(
        "{}{}",
        config.url.trim_end_matches('/'),
        if config.submit_path.starts_with('/') {
            config.submit_path.clone()
        } else {
            format!("/{}", config.submit_path)
        }
    );

    let mut final_params = serde_json::Map::new();
    for (k, v) in params {
        final_params.insert(k, Value::String(v));
    }
    final_params.insert("sign".to_string(), Value::String(sign));
    final_params.insert("sign_type".to_string(), Value::String("MD5".to_string()));

    if config.use_post {
        Ok(Value::String(crate::epay_render_support::build_epay_auto_post_form(
            &submit_url,
            &final_params,
        )))
    } else {
        let query = final_params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v.as_str().unwrap_or_default())))
            .collect::<Vec<_>>()
            .join("&");
        Ok(Value::String(format!("{}?{}", submit_url, query)))
    }
}
