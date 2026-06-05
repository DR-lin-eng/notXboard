use crate::*;

#[derive(Clone, sqlx::FromRow)]
pub(crate) struct OrderPlanRow {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) prices: Option<SqlxJson<Value>>,
    pub(crate) sell: bool,
    pub(crate) renew: bool,
    pub(crate) show: bool,
    pub(crate) visibility_scope: Option<String>,
    pub(crate) share_token: Option<String>,
    pub(crate) min_trust_level: Option<u64>,
    pub(crate) capacity_limit: Option<u64>,
}

pub(crate) struct CreateUserOrderInput<'a> {
    pub(crate) plan_id: Option<i64>,
    pub(crate) purchase_token: Option<&'a str>,
    pub(crate) coupon_code: Option<&'a str>,
    pub(crate) period: &'a str,
}

#[derive(Clone)]
pub(crate) struct PreparedOrderCoupon {
    pub(crate) coupon_id: Option<i64>,
    pub(crate) discount_amount: i64,
}

#[derive(Clone, sqlx::FromRow)]
struct InviterCommissionRow {
    commission_type: i64,
    commission_rate: Option<i64>,
}

pub(crate) async fn create_user_order(
    state: &AppState,
    user: &BearerUserRow,
    input: CreateUserOrderInput<'_>,
) -> Result<String, Response<Body>> {
    let normalized_period = validate_create_user_order_input(&input)?;
    let plan = load_requested_order_plan(state, input.plan_id, input.purchase_token).await?;

    validate_order_plan_for_user(state, &plan, user, input.purchase_token).await?;
    let price_before_discount = plan_price_for_period(&plan, normalized_period).ok_or_else(|| {
        fail_json_response(
            StatusCode::BAD_REQUEST,
            "This payment period cannot be purchased, please choose another period",
        )
    })?;
    let order_type = resolve_order_type(state, user.id, plan.id)
        .await
        .map_err(internal_error)?;
    let trade_no = generate_order_trade_no();
    let now = Utc::now().timestamp();
    let mut tx = state.db.begin().await.map_err(internal_error)?;
    let locked_balance = lock_user_balance_without_open_orders(&mut tx, user.id).await?;
    let coupon = prepare_order_coupon_locked(
        state,
        &mut tx,
        input.coupon_code,
        user.id,
        plan.id,
        normalized_period,
        price_before_discount,
        now,
    )
    .await?;
    let mut discount_amount = coupon.discount_amount;
    let coupon_id = coupon.coupon_id;
    let total_after_coupon = (price_before_discount - coupon.discount_amount).max(0);
    let (vip_discount, mut total_amount) = apply_user_discount(total_after_coupon, user.discount);
    discount_amount += vip_discount;
    let balance_amount =
        consume_locked_balance(&mut tx, user.id, locked_balance, &mut total_amount, now).await?;
    let commission_balance = compute_invite_commission_balance(state, user, total_amount)
        .await
        .map_err(internal_error)?;

    sqlx::query(
        "INSERT INTO v2_order (
            invite_user_id,user_id,plan_id,coupon_id,payment_id,type,period,trade_no,total_amount,handling_amount,
            discount_amount,surplus_amount,refund_amount,balance_amount,surplus_order_ids,status,commission_status,
            commission_balance,actual_commission_balance,paid_at,created_at,updated_at
         ) VALUES (?, ?, ?, ?, NULL, ?, ?, ?, ?, NULL, ?, NULL, NULL, ?, NULL, 0, 0, ?, NULL, NULL, ?, ?)"
    )
    .bind(user.invite_user_id)
    .bind(user.id)
    .bind(plan.id)
    .bind(coupon_id)
    .bind(order_type)
    .bind(normalized_period)
    .bind(&trade_no)
    .bind(total_amount)
    .bind(discount_amount)
    .bind(balance_amount)
    .bind(commission_balance)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;
    tx.commit().await.map_err(internal_error)?;

    Ok(trade_no)
}

pub(crate) async fn load_order_plan_by_id(
    state: &AppState,
    plan_id: i64,
) -> Result<Option<OrderPlanRow>, sqlx::Error> {
    sqlx::query_as::<_, OrderPlanRow>(
        "SELECT id, name, prices, sell, renew, `show` AS `show`, visibility_scope, share_token, min_trust_level, capacity_limit
         FROM v2_plan WHERE id = ? LIMIT 1",
    )
    .bind(plan_id)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn load_order_plan_by_share_token(
    state: &AppState,
    share_token: &str,
) -> Result<Option<OrderPlanRow>, sqlx::Error> {
    sqlx::query_as::<_, OrderPlanRow>(
        "SELECT id, name, prices, sell, renew, `show` AS `show`, visibility_scope, share_token, min_trust_level, capacity_limit
         FROM v2_plan WHERE share_token = ? LIMIT 1",
    )
    .bind(share_token)
    .fetch_optional(&state.db)
    .await
}

pub(crate) async fn prepare_order_coupon_locked(
    state: &AppState,
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    coupon_code: Option<&str>,
    user_id: i64,
    plan_id: i64,
    normalized_period: &str,
    total_amount: i64,
    now_ts: i64,
) -> Result<PreparedOrderCoupon, Response<Body>> {
    let Some(code) = coupon_code.filter(|value| !value.trim().is_empty()) else {
        return Ok(PreparedOrderCoupon {
            coupon_id: None,
            discount_amount: 0,
        });
    };

    let coupon = load_coupon_by_code_with_tx(tx, code)
        .await
        .map_err(internal_error)?;
    let Some(coupon) = coupon else {
        return Err(fail_json_response(StatusCode::BAD_REQUEST, "Invalid coupon"));
    };

    validate_coupon_for_user(state, &coupon, user_id, Some(plan_id), Some(normalized_period)).await?;
    let discount_amount = compute_coupon_discount_amount(total_amount, &coupon)
        .min(total_amount)
        .max(0);

    if let Some(limit_use) = coupon.limit_use {
        if limit_use <= 0 {
            return Err(fail_json_response(
                StatusCode::BAD_REQUEST,
                "This coupon is no longer available",
            ));
        }
        sqlx::query("UPDATE v2_coupon SET limit_use = ?, updated_at = ? WHERE id = ?")
            .bind(limit_use - 1)
            .bind(now_ts)
            .bind(coupon.id)
            .execute(&mut **tx)
            .await
            .map_err(internal_error)?;
    }

    Ok(PreparedOrderCoupon {
        coupon_id: Some(coupon.id),
        discount_amount,
    })
}

pub(crate) async fn lock_user_balance_without_open_orders(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
) -> Result<i64, Response<Body>> {
    let open_orders: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM v2_order
         WHERE user_id = ? AND status IN (0, 1)"
    )
    .bind(user_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(internal_error)?;
    if open_orders > 0 {
        return Err(fail_json_response(
            StatusCode::BAD_REQUEST,
            "You have an unpaid or pending order, please try again later or cancel it",
        ));
    }

    sqlx::query_scalar("SELECT balance FROM v2_user WHERE id = ? LIMIT 1 FOR UPDATE")
        .bind(user_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(internal_error)
}

fn validate_create_user_order_input(
    input: &CreateUserOrderInput<'_>,
) -> Result<&'static str, Response<Body>> {
    if input.plan_id.is_none() && input.purchase_token.is_none() {
        return Err(fail_json_response(
            StatusCode::BAD_REQUEST,
            "Plan ID cannot be empty",
        ));
    }
    if input.period.is_empty() {
        return Err(fail_json_response(
            StatusCode::BAD_REQUEST,
            "Plan period cannot be empty",
        ));
    }
    normalize_order_period(input.period)
        .ok_or_else(|| fail_json_response(StatusCode::BAD_REQUEST, "Wrong plan period"))
}

async fn load_requested_order_plan(
    state: &AppState,
    plan_id: Option<i64>,
    purchase_token: Option<&str>,
) -> Result<OrderPlanRow, Response<Body>> {
    let plan = if let Some(plan_id) = plan_id {
        load_order_plan_by_id(state, plan_id)
            .await
            .map_err(internal_error)?
    } else {
        load_order_plan_by_share_token(state, purchase_token.unwrap_or_default())
            .await
            .map_err(internal_error)?
    };
    plan.ok_or_else(|| {
        fail_json_response(
            StatusCode::BAD_REQUEST,
            "Subscription plan does not exist",
        )
    })
}

fn apply_user_discount(total_amount: i64, user_discount: Option<i64>) -> (i64, i64) {
    let Some(user_discount) = user_discount.filter(|value| *value > 0) else {
        return (0, total_amount);
    };
    let vip_discount =
        ((total_amount as f64) * ((user_discount as f64) / 100.0)).round() as i64;
    (vip_discount, (total_amount - vip_discount).max(0))
}

async fn consume_locked_balance(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    user_id: i64,
    locked_balance: i64,
    total_amount: &mut i64,
    now: i64,
) -> Result<i64, Response<Body>> {
    if locked_balance <= 0 || *total_amount <= 0 {
        return Ok(0);
    }

    let balance_amount = locked_balance.min(*total_amount);
    *total_amount -= balance_amount;
    sqlx::query("UPDATE v2_user SET balance = ?, updated_at = ? WHERE id = ?")
        .bind(locked_balance - balance_amount)
        .bind(now)
        .bind(user_id)
        .execute(&mut **tx)
        .await
        .map_err(internal_error)?;
    Ok(balance_amount)
}

async fn load_inviter_commission_profile(
    state: &AppState,
    user_id: i64,
) -> Result<Option<InviterCommissionRow>, sqlx::Error> {
    sqlx::query_as::<_, InviterCommissionRow>(
        "SELECT commission_type, commission_rate
         FROM v2_user
         WHERE id = ?
         LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
}

async fn user_has_valid_paid_order(state: &AppState, user_id: i64) -> Result<bool, sqlx::Error> {
    let exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM v2_order
         WHERE user_id = ?
           AND status NOT IN (0, 2)",
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await?;
    Ok(exists > 0)
}

async fn compute_invite_commission_balance(
    state: &AppState,
    user: &BearerUserRow,
    order_total_amount: i64,
) -> Result<i64, sqlx::Error> {
    if order_total_amount <= 0 {
        return Ok(0);
    }
    let Some(invite_user_id) = user.invite_user_id else {
        return Ok(0);
    };
    let inviter = load_inviter_commission_profile(state, invite_user_id).await?;
    let Some(inviter) = inviter else {
        return Ok(0);
    };

    let mut commission_type = inviter.commission_type;
    if commission_type == 0 {
        commission_type = if get_setting_bool(state, "commission_first_time_enable", true).await {
            2
        } else {
            1
        };
    }

    let is_commissionable = match commission_type {
        1 => true,
        2 => !user_has_valid_paid_order(state, user.id).await?,
        _ => false,
    };
    if !is_commissionable {
        return Ok(0);
    }

    let rate = inviter
        .commission_rate
        .unwrap_or(get_setting_int(state, "invite_commission", 10).await);
    Ok(((order_total_amount as f64) * ((rate as f64) / 100.0)).round() as i64)
}
