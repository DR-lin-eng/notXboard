use crate::*;

#[derive(Clone, Copy)]
enum EpayCheckoutSource {
    RoutedProfile,
    PaymentMethod,
}

struct ResolvedEpayCheckoutConfig {
    config: EpayConfig,
    source: EpayCheckoutSource,
}

#[derive(sqlx::FromRow)]
struct EpayCheckoutPlanRow {
    scope: Option<String>,
    owner_user_id: Option<u64>,
}

pub(crate) async fn resolve_checkout_epay_config(
    state: &AppState,
    order: &CheckoutOrderRow,
    payment: &PaymentMethodRow,
    pay_to: Option<&str>,
) -> Result<EpayConfig, Response<Body>> {
    let snapshot_columns_ready = order_epay_snapshot_columns_ready(state)
        .await
        .map_err(internal_error)?;
    let payment_config = epay_config_from_payment_method(payment);

    if snapshot_columns_ready {
        if let Some(snapshot) = load_checkout_epay_snapshot(state, order, &payment_config).await? {
            return Ok(snapshot);
        }
    }

    let resolved = resolve_fresh_epay_checkout_config(state, order, payment, pay_to).await;
    if !snapshot_columns_ready && matches!(resolved.source, EpayCheckoutSource::RoutedProfile) {
        return Err(fail_json_response(
            StatusCode::BAD_REQUEST,
            "Payment snapshot is unavailable, please contact support",
        ));
    }

    if snapshot_columns_ready && epay_config_is_complete(&resolved.config) {
        save_checkout_epay_snapshot(state, order.id, &resolved.config).await?;
    }

    Ok(resolved.config)
}

async fn load_checkout_epay_snapshot(
    state: &AppState,
    order: &CheckoutOrderRow,
    payment_config: &EpayConfig,
) -> Result<Option<EpayConfig>, Response<Body>> {
    let row = sqlx::query(
        "SELECT epay_pid, epay_url, epay_key_encrypted
         FROM v2_order
         WHERE id = ?
         LIMIT 1",
    )
    .bind(order.id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?;
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
    let key = decrypt_laravel_string(&state.app_key, &key_encrypted).map_err(|err| {
        tracing::warn!(
            order_id = order.id,
            trade_no = order.trade_no.as_str(),
            error = %err,
            "checkout epay snapshot decrypt failed"
        );
        fail_json_response(
            StatusCode::BAD_REQUEST,
            "Payment snapshot is invalid, please contact support",
        )
    })?;

    Ok(Some(EpayConfig {
        pid,
        key,
        url,
        submit_path: "/pay/submit.php".to_string(),
        use_post: true,
        sitename: payment_config.sitename.clone(),
        device: payment_config.device.clone(),
    }))
}

async fn resolve_fresh_epay_checkout_config(
    state: &AppState,
    order: &CheckoutOrderRow,
    payment: &PaymentMethodRow,
    pay_to: Option<&str>,
) -> ResolvedEpayCheckoutConfig {
    if pay_to.map(|value| value.trim()) == Some("sponsor") {
        match load_sponsor_epay_profile(state).await {
            Ok(Some(profile)) => {
                return ResolvedEpayCheckoutConfig {
                    config: EpayConfig {
                        pid: profile.pid,
                        key: profile.key,
                        url: profile.url,
                        submit_path: profile.submit_path,
                        use_post: profile.use_post,
                        sitename: profile.sitename,
                        device: profile.device,
                    },
                    source: EpayCheckoutSource::RoutedProfile,
                };
            }
            Ok(None) => {}
            Err(_) => {}
        }
        return payment_method_checkout_config(payment);
    }

    if let Some(owner_user_id) = load_node_plan_owner_for_checkout(state, order.plan_id).await {
        match load_user_epay_profile(state, owner_user_id).await {
            Ok(Some(config)) => {
                return ResolvedEpayCheckoutConfig {
                    config,
                    source: EpayCheckoutSource::RoutedProfile,
                };
            }
            Ok(None) => {}
            Err(err) => {
                tracing::warn!(
                    plan_id = order.plan_id,
                    owner_user_id,
                    error = %err,
                    "owner epay profile resolution failed; falling back to payment method config"
                );
            }
        }
    }

    payment_method_checkout_config(payment)
}

fn payment_method_checkout_config(payment: &PaymentMethodRow) -> ResolvedEpayCheckoutConfig {
    ResolvedEpayCheckoutConfig {
        config: epay_config_from_payment_method_for_snapshot(payment),
        source: EpayCheckoutSource::PaymentMethod,
    }
}

async fn load_node_plan_owner_for_checkout(state: &AppState, plan_id: i64) -> Option<i64> {
    let row = sqlx::query_as::<_, EpayCheckoutPlanRow>(
        "SELECT scope, owner_user_id
         FROM v2_plan
         WHERE id = ?
         LIMIT 1",
    )
    .bind(plan_id)
    .fetch_optional(&state.db)
    .await;
    let row = match row {
        Ok(row) => row?,
        Err(err) => {
            tracing::warn!(
                plan_id,
                error = %err,
                "load plan epay routing failed; falling back to payment method config"
            );
            return None;
        }
    };

    if row.scope.as_deref().unwrap_or("legacy") != "node" {
        return None;
    }
    row.owner_user_id
        .filter(|owner_user_id| *owner_user_id > 0)
        .and_then(|owner_user_id| i64::try_from(owner_user_id).ok())
}

fn epay_config_from_payment_method_for_snapshot(payment: &PaymentMethodRow) -> EpayConfig {
    let mut config = epay_config_from_payment_method(payment);
    if epay_config_is_complete(&config) {
        config.submit_path = "/pay/submit.php".to_string();
        config.use_post = true;
    }
    config
}

async fn save_checkout_epay_snapshot(
    state: &AppState,
    order_id: i64,
    config: &EpayConfig,
) -> Result<(), Response<Body>> {
    let key_encrypted = encrypt_laravel_string(&state.app_key, &config.key)
        .map_err(|err| json_error(StatusCode::INTERNAL_SERVER_ERROR, &err))?;
    let updated = sqlx::query(
        "UPDATE v2_order
         SET epay_pid = ?, epay_url = ?, epay_key_encrypted = ?, updated_at = ?
         WHERE id = ? AND status = 0",
    )
    .bind(config.pid.trim())
    .bind(config.url.trim_end_matches('/'))
    .bind(key_encrypted)
    .bind(Utc::now().timestamp())
    .bind(order_id)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;
    if updated.rows_affected() == 0 {
        return Err(fail_json_response(
            StatusCode::BAD_REQUEST,
            "Order does not exist or has been paid",
        ));
    }
    Ok(())
}

fn epay_config_is_complete(config: &EpayConfig) -> bool {
    !config.pid.trim().is_empty()
        && !config.key.trim().is_empty()
        && !config.url.trim().is_empty()
}
