use crate::*;

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct SponsorMethodRow {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) payment: String,
    pub(crate) icon: Option<String>,
}

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct SponsorDonationRow {
    pub(crate) id: u64,
    pub(crate) user_id: Option<u64>,
    pub(crate) trade_no: String,
    pub(crate) total_amount: i64,
    pub(crate) payment_id: Option<i64>,
    pub(crate) callback_no: Option<String>,
    pub(crate) status: i64,
    pub(crate) paid_at: Option<i64>,
    pub(crate) created_at: Option<chrono::DateTime<Utc>>,
    pub(crate) updated_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Clone)]
pub(crate) struct SponsorEpayProfile {
    pub(crate) pid: String,
    pub(crate) key: String,
    pub(crate) url: String,
    pub(crate) submit_path: String,
    pub(crate) use_post: bool,
    pub(crate) sitename: Option<String>,
    pub(crate) device: Option<String>,
}

pub(crate) async fn load_sponsor_methods(
    state: &AppState,
) -> Result<Vec<SponsorMethodRow>, sqlx::Error> {
    sqlx::query_as::<_, SponsorMethodRow>(
        "SELECT id, name, payment, icon
         FROM v2_payment
         WHERE enable = 1
         ORDER BY sort ASC"
    )
    .fetch_all(&state.db)
    .await
}

pub(crate) async fn load_sponsor_donation_by_trade_no(
    state: &AppState,
    trade_no: &str,
    user_id: Option<u64>,
) -> Result<Option<SponsorDonationRow>, sqlx::Error> {
    let mut sql = String::from(
        "SELECT id, user_id, trade_no, total_amount, payment_id, callback_no, status, paid_at, created_at, updated_at
         FROM sponsor_donations
         WHERE trade_no = ?"
    );
    if user_id.is_some() {
        sql.push_str(" AND user_id = ?");
    }
    sql.push_str(" LIMIT 1");

    let mut query = sqlx::query_as::<_, SponsorDonationRow>(&sql).bind(trade_no);
    if let Some(user_id) = user_id {
        query = query.bind(user_id);
    }
    query.fetch_optional(&state.db).await
}

pub(crate) async fn load_pending_sponsor_donation_by_trade_no(
    state: &AppState,
    trade_no: &str,
    user_id: Option<u64>,
) -> Result<Option<SponsorDonationRow>, sqlx::Error> {
    let mut sql = String::from(
        "SELECT id, user_id, trade_no, total_amount, payment_id, callback_no, status, paid_at, created_at, updated_at
         FROM sponsor_donations
         WHERE trade_no = ? AND status = 0"
    );
    if user_id.is_some() {
        sql.push_str(" AND user_id = ?");
    }
    sql.push_str(" LIMIT 1");

    let mut query = sqlx::query_as::<_, SponsorDonationRow>(&sql).bind(trade_no);
    if let Some(user_id) = user_id {
        query = query.bind(user_id);
    }
    query.fetch_optional(&state.db).await
}

pub(crate) async fn load_sponsor_epay_profile(
    state: &AppState,
) -> Result<Option<SponsorEpayProfile>, Response<Body>> {
    let url = get_setting_string(state, "sponsor_epay_url", "").await;
    let pid = get_setting_string(state, "sponsor_epay_pid", "").await;
    let key = get_setting_string(state, "sponsor_epay_key", "").await;
    if url.trim().is_empty() || pid.trim().is_empty() || key.trim().is_empty() {
        return Ok(None);
    }
    let url = crate::url_security_support::normalize_http_url(&url, false)
        .map_err(|message| fail_json_response(StatusCode::BAD_REQUEST, &message))?;
    let raw_submit_path = get_setting_string(state, "sponsor_epay_submit_path", "/pay/submit.php").await;
    let submit_path = crate::url_security_support::normalize_relative_path(
        Some(raw_submit_path.as_str()),
        "/pay/submit.php",
    )
    .map_err(|message| fail_json_response(StatusCode::BAD_REQUEST, &message))?;

    Ok(Some(SponsorEpayProfile {
        pid,
        key,
        url,
        submit_path,
        use_post: get_setting_bool(state, "sponsor_epay_use_post", true).await,
        sitename: Some(get_setting_string(state, "sponsor_epay_sitename", "").await).filter(|v| !v.is_empty()),
        device: Some(get_setting_string(state, "sponsor_epay_device", "").await).filter(|v| !v.is_empty()),
    }))
}

pub(crate) async fn mark_sponsor_donation_paid(
    state: &AppState,
    trade_no: &str,
    callback_no: &str,
) -> Result<bool, sqlx::Error> {
    let donation = sqlx::query(
        "SELECT id, status, paid_at, callback_no
         FROM sponsor_donations
         WHERE trade_no = ?
         LIMIT 1"
    )
    .bind(trade_no)
    .fetch_optional(&state.db)
    .await?;
    let Some(donation) = donation else {
        return Ok(false);
    };

    let donation_id: u64 = donation.try_get("id").unwrap_or_default();
    let status: i64 = donation.try_get("status").unwrap_or_default();
    let paid_at: Option<i64> = donation.try_get("paid_at").unwrap_or(None);
    let existing_callback: Option<String> = donation.try_get("callback_no").unwrap_or(None);

    if status == 0 {
        sqlx::query(
            "UPDATE sponsor_donations
             SET status = 1,
                 paid_at = ?,
                 callback_no = COALESCE(callback_no, ?),
                 updated_at = NOW()
             WHERE id = ? AND status = 0"
        )
        .bind(paid_at.unwrap_or_else(|| Utc::now().timestamp()))
        .bind(if callback_no.is_empty() { None::<String> } else { Some(callback_no.to_string()) })
        .bind(donation_id)
        .execute(&state.db)
        .await?;
        return Ok(true);
    }

    if status == 1 {
        if existing_callback.as_deref().unwrap_or("").is_empty() && !callback_no.is_empty() {
            sqlx::query("UPDATE sponsor_donations SET callback_no = ?, updated_at = NOW() WHERE id = ?")
                .bind(callback_no)
                .bind(donation_id)
                .execute(&state.db)
                .await?;
        }
        return Ok(true);
    }

    Ok(true)
}

pub(crate) fn serialize_sponsor_method(method: &SponsorMethodRow) -> Value {
    json!({
        "id": method.id,
        "name": method.name,
        "payment": method.payment,
        "icon": method.icon,
    })
}

pub(crate) fn serialize_sponsor_donation(donation: &SponsorDonationRow) -> Value {
    json!({
        "id": donation.id,
        "user_id": donation.user_id,
        "trade_no": donation.trade_no,
        "total_amount": donation.total_amount,
        "payment_id": donation.payment_id,
        "callback_no": donation.callback_no,
        "status": donation.status,
        "paid_at": donation.paid_at,
        "created_at": format_optional_naive_datetime(donation.created_at),
        "updated_at": format_optional_naive_datetime(donation.updated_at),
    })
}

pub(crate) fn build_sponsor_epay_checkout_payload(
    trade_no: &str,
    total_amount: i64,
    config: &SponsorEpayProfile,
    app_url: &str,
) -> Result<Value, Response<Body>> {
    let money = format!("{:.2}", (total_amount as f64) / 100.0);
    let base_url = if app_url.trim().is_empty() {
        "http://127.0.0.1:18093".to_string()
    } else {
        app_url.trim_end_matches('/').to_string()
    };
    let notify_url = format!("{}/api/v1/guest/payment/notify/EPay/{}", base_url, "sponsor");
    let return_url = format!("{}/#/sponsor/{}", base_url, trade_no);

    let mut params = vec![
        ("money".to_string(), money),
        ("name".to_string(), trade_no.to_string()),
        ("notify_url".to_string(), notify_url),
        ("out_trade_no".to_string(), trade_no.to_string()),
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
