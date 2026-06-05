use crate::*;

#[derive(Clone, Debug)]
pub(crate) struct CommissionPayoutConfig {
    pub(crate) distribution_levels: Vec<i64>,
    pub(crate) withdraw_close_enable: bool,
}

pub(crate) async fn load_commission_payout_config(
    state: &AppState,
) -> CommissionPayoutConfig {
    let commission_distribution_enabled =
        get_setting_bool(state, "commission_distribution_enable", false).await;
    let distribution_levels = if commission_distribution_enabled {
        vec![
            get_setting_int(state, "commission_distribution_l1", 0).await,
            get_setting_int(state, "commission_distribution_l2", 0).await,
            get_setting_int(state, "commission_distribution_l3", 0).await,
        ]
    } else {
        vec![100]
    };

    CommissionPayoutConfig {
        distribution_levels,
        withdraw_close_enable: get_setting_bool(state, "withdraw_close_enable", false).await,
    }
}

pub(crate) async fn load_inviter_chain_with_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    first_invite_user_id: i64,
    max_depth: usize,
) -> Result<Vec<i64>, sqlx::Error> {
    let mut chain = Vec::new();
    let mut current = Some(first_invite_user_id);

    while let Some(invite_user_id) = current {
        if chain.len() >= max_depth {
            break;
        }
        chain.push(invite_user_id);
        current = sqlx::query_scalar::<_, Option<i64>>(
            "SELECT invite_user_id FROM v2_user WHERE id = ? LIMIT 1"
        )
        .bind(invite_user_id)
        .fetch_optional(&mut **tx)
        .await?
        .flatten();
    }

    Ok(chain)
}

pub(crate) async fn load_existing_commission_inviter_ids_with_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    inviter_ids: &[i64],
    user_id: i64,
    trade_no: &str,
) -> Result<HashSet<i64>, sqlx::Error> {
    if inviter_ids.is_empty() {
        return Ok(HashSet::new());
    }

    let mut builder = sqlx::QueryBuilder::<sqlx::MySql>::new(
        "SELECT DISTINCT invite_user_id
         FROM v2_commission_log
         WHERE user_id = ",
    );
    builder.push_bind(user_id);
    builder.push(" AND trade_no = ");
    builder.push_bind(trade_no);
    builder.push(" AND invite_user_id IN (");
    {
        let mut separated = builder.separated(", ");
        for inviter_id in inviter_ids {
            separated.push_bind(inviter_id);
        }
    }
    builder.push(")");

    let rows = builder.build().fetch_all(&mut **tx).await?;
    let mut result = HashSet::with_capacity(rows.len());
    for row in rows {
        let inviter_id = row.try_get::<i64, _>("invite_user_id").unwrap_or_default();
        result.insert(inviter_id);
    }
    Ok(result)
}
