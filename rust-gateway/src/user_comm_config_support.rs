use crate::*;

#[derive(Clone, Debug, Default)]
pub(crate) struct UserCommConfigSnapshot {
    pub(crate) telegram_enabled: bool,
    pub(crate) telegram_discuss_link: Value,
    pub(crate) stripe_pk: Value,
    pub(crate) withdraw_methods: Value,
    pub(crate) withdraw_close: bool,
    pub(crate) currency: String,
    pub(crate) currency_symbol: String,
    pub(crate) commission_distribution_enable: bool,
    pub(crate) commission_distribution_l1: Value,
    pub(crate) commission_distribution_l2: Value,
    pub(crate) commission_distribution_l3: Value,
}

pub(crate) async fn load_user_comm_config_snapshot(
    state: &AppState,
) -> UserCommConfigSnapshot {
    let (
        telegram_enabled,
        telegram_discuss_link,
        stripe_pk,
        withdraw_methods,
        withdraw_close,
        currency,
        currency_symbol,
        commission_distribution_enable,
        commission_distribution_l1,
        commission_distribution_l2,
        commission_distribution_l3,
    ) = tokio::join!(
        get_setting_bool(state, "telegram_bot_enable", false),
        get_setting_value(state, "telegram_discuss_link"),
        get_setting_value(state, "stripe_pk_live"),
        get_setting_value(state, "commission_withdraw_method"),
        get_setting_bool(state, "withdraw_close_enable", false),
        get_setting_string(state, "currency", "CNY"),
        get_setting_string(state, "currency_symbol", "¥"),
        get_setting_bool(state, "commission_distribution_enable", false),
        get_setting_value(state, "commission_distribution_l1"),
        get_setting_value(state, "commission_distribution_l2"),
        get_setting_value(state, "commission_distribution_l3"),
    );

    UserCommConfigSnapshot {
        telegram_enabled,
        telegram_discuss_link,
        stripe_pk,
        withdraw_methods,
        withdraw_close,
        currency,
        currency_symbol,
        commission_distribution_enable,
        commission_distribution_l1,
        commission_distribution_l2,
        commission_distribution_l3,
    }
}
