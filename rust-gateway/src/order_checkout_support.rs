use crate::*;

pub(crate) enum PreparedUserCheckout {
    NoPaymentRequired,
    Pending(UserCheckoutSession),
}

pub(crate) struct UserCheckoutSession {
    pub(crate) order: CheckoutOrderRow,
    pub(crate) payment: PaymentMethodRow,
    pub(crate) amount: i64,
}

pub(crate) async fn prepare_user_checkout(
    state: &AppState,
    user_id: i64,
    trade_no: &str,
    payment: Option<PaymentMethodRow>,
) -> Result<PreparedUserCheckout, Response<Body>> {
    let order = find_user_checkout_order(state, user_id, trade_no)
        .await
        .map_err(internal_error)?;
    let Some(mut order) = order else {
        return Err(fail_json_response(
            StatusCode::BAD_REQUEST,
            "Order does not exist or has been paid",
        ));
    };
    if order.status != 0 {
        return Err(fail_json_response(
            StatusCode::BAD_REQUEST,
            "Order does not exist or has been paid",
        ));
    }

    if order.total_amount <= 0 {
        finalize_zero_amount_checkout(state, order.id).await?;
        return Ok(PreparedUserCheckout::NoPaymentRequired);
    }

    let payment = validate_checkout_payment(order.payment_id, payment)?;
    lock_checkout_payment_method(state, &mut order, &payment).await?;
    let amount = order.total_amount + order.handling_amount.unwrap_or(0);

    Ok(PreparedUserCheckout::Pending(UserCheckoutSession {
        order,
        payment,
        amount,
    }))
}

async fn finalize_zero_amount_checkout(
    state: &AppState,
    order_id: i64,
) -> Result<(), Response<Body>> {
    let now = Utc::now().timestamp();
    let updated = sqlx::query(
        "UPDATE v2_order
         SET status = 1, paid_at = ?, updated_at = ?
         WHERE id = ? AND status = 0",
    )
    .bind(now)
    .bind(now)
    .bind(order_id)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;
    if updated.rows_affected() == 1 {
        complete_processing_order_by_id(state, order_id)
            .await
            .map_err(internal_error)?;
    }
    Ok(())
}

fn validate_checkout_payment(
    existing_payment_id: Option<i64>,
    payment: Option<PaymentMethodRow>,
) -> Result<PaymentMethodRow, Response<Body>> {
    let Some(payment) = payment else {
        return Err(fail_json_response(
            StatusCode::BAD_REQUEST,
            "Payment method is not available",
        ));
    };
    if !payment.enable {
        return Err(fail_json_response(
            StatusCode::BAD_REQUEST,
            "Payment method is not available",
        ));
    }
    if let Some(existing_payment_id) = existing_payment_id {
        if existing_payment_id != payment.id {
            return Err(fail_json_response(
                StatusCode::BAD_REQUEST,
                "The payment method has been locked for this order",
            ));
        }
    }
    Ok(payment)
}

async fn lock_checkout_payment_method(
    state: &AppState,
    order: &mut CheckoutOrderRow,
    payment: &PaymentMethodRow,
) -> Result<(), Response<Body>> {
    let mut should_save = false;
    if order.payment_id.is_none() {
        order.payment_id = Some(payment.id);
        should_save = true;
    }
    if order.handling_amount.is_none() {
        order.handling_amount = Some(compute_handling_amount(
            order.total_amount,
            payment.handling_fee_fixed,
            payment.handling_fee_percent,
        ));
        should_save = true;
    }

    if should_save {
        sqlx::query(
            "UPDATE v2_order
             SET payment_id = ?, handling_amount = ?, updated_at = ?
             WHERE id = ?",
        )
        .bind(order.payment_id)
        .bind(order.handling_amount)
        .bind(Utc::now().timestamp())
        .bind(order.id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;
    }
    Ok(())
}
