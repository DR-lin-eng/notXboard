use serde_json::{json, Map, Value};

#[derive(Clone, Copy)]
enum PaymentFieldDefault {
    EmptyString,
    Bool(bool),
}

#[derive(Clone, Copy)]
struct PaymentFieldSpec {
    key: &'static str,
    field_type: &'static str,
    label: &'static str,
    placeholder: &'static str,
    description: &'static str,
    default: PaymentFieldDefault,
}

const EPAY_FIELDS: &[PaymentFieldSpec] = &[
    PaymentFieldSpec {
        key: "url",
        field_type: "string",
        label: "支付网关地址",
        placeholder: "",
        description: "请填写完整的支付网关地址，包括协议（http或https）",
        default: PaymentFieldDefault::EmptyString,
    },
    PaymentFieldSpec {
        key: "submit_path",
        field_type: "string",
        label: "提交接口路径",
        placeholder: "",
        description: "默认 /submit.php；credit.linux.do 为 /pay/submit.php",
        default: PaymentFieldDefault::EmptyString,
    },
    PaymentFieldSpec {
        key: "pid",
        field_type: "string",
        label: "商户ID",
        placeholder: "",
        description: "请填写商户ID",
        default: PaymentFieldDefault::EmptyString,
    },
    PaymentFieldSpec {
        key: "key",
        field_type: "string",
        label: "通信密钥",
        placeholder: "",
        description: "请填写通信密钥",
        default: PaymentFieldDefault::EmptyString,
    },
    PaymentFieldSpec {
        key: "use_post",
        field_type: "boolean",
        label: "使用POST跳转",
        placeholder: "",
        description: "部分易支付需要 POST 提交；credit.linux.do 必须使用 POST（建议开启），开启后将返回自动提交表单",
        default: PaymentFieldDefault::Bool(false),
    },
    PaymentFieldSpec {
        key: "sitename",
        field_type: "string",
        label: "站点名称（可选）",
        placeholder: "",
        description: "部分易支付会展示该字段",
        default: PaymentFieldDefault::EmptyString,
    },
    PaymentFieldSpec {
        key: "device",
        field_type: "string",
        label: "终端标识（可选）",
        placeholder: "",
        description: "部分服务支持 device 字段",
        default: PaymentFieldDefault::EmptyString,
    },
];

const STRIPE_CREDIT_FIELDS: &[PaymentFieldSpec] = &[PaymentFieldSpec {
    key: "stripe_pk_live",
    field_type: "string",
    label: "Stripe Publishable Key",
    placeholder: "",
    description: "前端使用的 Stripe 公钥",
    default: PaymentFieldDefault::EmptyString,
}];

const ALIPAY_F2F_FIELDS: &[PaymentFieldSpec] = &[
    PaymentFieldSpec {
        key: "app_id",
        field_type: "string",
        label: "支付宝APPID",
        placeholder: "",
        description: "支付宝开放平台应用的APPID",
        default: PaymentFieldDefault::EmptyString,
    },
    PaymentFieldSpec {
        key: "private_key",
        field_type: "text",
        label: "支付宝私钥",
        placeholder: "",
        description: "应用私钥，用于签名",
        default: PaymentFieldDefault::EmptyString,
    },
    PaymentFieldSpec {
        key: "public_key",
        field_type: "text",
        label: "支付宝公钥",
        placeholder: "",
        description: "支付宝公钥，用于验签",
        default: PaymentFieldDefault::EmptyString,
    },
    PaymentFieldSpec {
        key: "product_name",
        field_type: "string",
        label: "自定义商品名称",
        placeholder: "",
        description: "将会体现在支付宝账单中",
        default: PaymentFieldDefault::EmptyString,
    },
];

const BTCPAY_FIELDS: &[PaymentFieldSpec] = &[
    PaymentFieldSpec {
        key: "btcpay_url",
        field_type: "string",
        label: "API接口所在网址",
        placeholder: "",
        description: "包含最后的斜杠，例如：https://your-btcpay.com/",
        default: PaymentFieldDefault::EmptyString,
    },
    PaymentFieldSpec {
        key: "btcpay_storeId",
        field_type: "string",
        label: "Store ID",
        placeholder: "",
        description: "BTCPay商店标识符",
        default: PaymentFieldDefault::EmptyString,
    },
    PaymentFieldSpec {
        key: "btcpay_api_key",
        field_type: "string",
        label: "API KEY",
        placeholder: "",
        description: "个人设置中的API KEY(非商店设置中的)",
        default: PaymentFieldDefault::EmptyString,
    },
    PaymentFieldSpec {
        key: "btcpay_webhook_key",
        field_type: "string",
        label: "WEBHOOK KEY",
        placeholder: "",
        description: "Webhook通知密钥",
        default: PaymentFieldDefault::EmptyString,
    },
];

const COINBASE_FIELDS: &[PaymentFieldSpec] = &[
    PaymentFieldSpec {
        key: "coinbase_url",
        field_type: "string",
        label: "接口地址",
        placeholder: "",
        description: "Coinbase Commerce API地址",
        default: PaymentFieldDefault::EmptyString,
    },
    PaymentFieldSpec {
        key: "coinbase_api_key",
        field_type: "string",
        label: "API KEY",
        placeholder: "",
        description: "Coinbase Commerce API密钥",
        default: PaymentFieldDefault::EmptyString,
    },
    PaymentFieldSpec {
        key: "coinbase_webhook_key",
        field_type: "string",
        label: "WEBHOOK KEY",
        placeholder: "",
        description: "Webhook签名验证密钥",
        default: PaymentFieldDefault::EmptyString,
    },
];

const COINPAYMENTS_FIELDS: &[PaymentFieldSpec] = &[
    PaymentFieldSpec {
        key: "coinpayments_merchant_id",
        field_type: "string",
        label: "Merchant ID",
        placeholder: "",
        description: "商户 ID，填写您在 Account Settings 中得到的 ID",
        default: PaymentFieldDefault::EmptyString,
    },
    PaymentFieldSpec {
        key: "coinpayments_ipn_secret",
        field_type: "string",
        label: "IPN Secret",
        placeholder: "",
        description: "通知密钥，填写您在 Merchant Settings 中自行设置的值",
        default: PaymentFieldDefault::EmptyString,
    },
    PaymentFieldSpec {
        key: "coinpayments_currency",
        field_type: "string",
        label: "货币代码",
        placeholder: "",
        description: "填写您的货币代码（大写），建议与 Merchant Settings 中的值相同",
        default: PaymentFieldDefault::EmptyString,
    },
];

const MGATE_FIELDS: &[PaymentFieldSpec] = &[
    PaymentFieldSpec {
        key: "mgate_url",
        field_type: "string",
        label: "API地址",
        placeholder: "",
        description: "MGate支付网关API地址",
        default: PaymentFieldDefault::EmptyString,
    },
    PaymentFieldSpec {
        key: "mgate_app_id",
        field_type: "string",
        label: "APP ID",
        placeholder: "",
        description: "MGate应用标识符",
        default: PaymentFieldDefault::EmptyString,
    },
    PaymentFieldSpec {
        key: "mgate_app_secret",
        field_type: "string",
        label: "App Secret",
        placeholder: "",
        description: "MGate应用密钥",
        default: PaymentFieldDefault::EmptyString,
    },
    PaymentFieldSpec {
        key: "mgate_source_currency",
        field_type: "string",
        label: "源货币",
        placeholder: "",
        description: "默认CNY，源货币类型",
        default: PaymentFieldDefault::EmptyString,
    },
];

const SUPPORTED_PAYMENT_METHODS: &[&str] = &[
    "EPay",
    "StripeCredit",
    "AlipayF2F",
    "BTCPay",
    "Coinbase",
    "CoinPayments",
    "MGate",
];

pub(crate) fn supported_payment_methods() -> &'static [&'static str] {
    SUPPORTED_PAYMENT_METHODS
}

pub(crate) fn build_payment_form_schema(
    payment: &str,
    existing_config: &Map<String, Value>,
) -> Option<Map<String, Value>> {
    match payment {
        "EPay" => Some(build_form(existing_config, EPAY_FIELDS)),
        "StripeCredit" => Some(build_form(existing_config, STRIPE_CREDIT_FIELDS)),
        "AlipayF2F" => Some(build_form(existing_config, ALIPAY_F2F_FIELDS)),
        "BTCPay" => Some(build_form(existing_config, BTCPAY_FIELDS)),
        "Coinbase" => Some(build_form(existing_config, COINBASE_FIELDS)),
        "CoinPayments" => Some(build_form(existing_config, COINPAYMENTS_FIELDS)),
        "MGate" => Some(build_form(existing_config, MGATE_FIELDS)),
        _ => None,
    }
}

fn build_form(
    existing_config: &Map<String, Value>,
    fields: &[PaymentFieldSpec],
) -> Map<String, Value> {
    let mut map = Map::new();
    for field in fields {
        map.insert(
            field.key.to_string(),
            payment_form_field(
                field.field_type,
                field.label,
                field.placeholder,
                field.description,
                existing_config
                    .get(field.key)
                    .cloned()
                    .unwrap_or_else(|| default_payment_field_value(field.default)),
                Value::Array(Vec::new()),
            ),
        );
    }
    map
}

fn default_payment_field_value(default: PaymentFieldDefault) -> Value {
    match default {
        PaymentFieldDefault::EmptyString => Value::String(String::new()),
        PaymentFieldDefault::Bool(value) => Value::Bool(value),
    }
}

fn payment_form_field(
    field_type: &str,
    label: &str,
    placeholder: &str,
    description: &str,
    value: Value,
    options: Value,
) -> Value {
    json!({
        "type": field_type,
        "label": label,
        "placeholder": placeholder,
        "description": description,
        "value": value,
        "options": options
    })
}
