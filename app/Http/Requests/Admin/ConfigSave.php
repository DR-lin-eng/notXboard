<?php

namespace App\Http\Requests\Admin;

use Illuminate\Foundation\Http\FormRequest;

class ConfigSave extends FormRequest
{
    const RULES = [
        // invite & commission
        'invite_force' => '',
        'invite_commission' => 'integer|nullable',
        'invite_gen_limit' => 'integer|nullable',
        'invite_never_expire' => '',
        'commission_first_time_enable' => '',
        'commission_auto_check_enable' => '',
        'commission_withdraw_limit' => 'nullable|numeric',
        'commission_withdraw_method' => 'nullable|array',
        'withdraw_close_enable' => '',
        'commission_distribution_enable' => '',
        'commission_distribution_l1' => 'nullable|numeric',
        'commission_distribution_l2' => 'nullable|numeric',
        'commission_distribution_l3' => 'nullable|numeric',
        // site
        'logo' => 'nullable|url',
        'force_https' => '',
        'stop_register' => '',
        'register_mode' => 'nullable|string|in:all,email_only,oauth_only,closed',
        'app_name' => '',
        'app_description' => '',
        'app_url' => 'nullable|url',
        'subscribe_url' => 'nullable',
        'subscribe_root_domains' => 'nullable|string|max:2000',
        'try_out_enable' => '',
        'try_out_plan_id' => 'integer',
        'try_out_hour' => 'numeric',
        'tos_url' => 'nullable|url',
        'currency' => '',
        'currency_symbol' => '',
        // subscribe
        'plan_change_enable' => '',
        'reset_traffic_method' => 'in:0,1,2,3,4',
        'surplus_enable' => '',
        'new_order_event_id' => '',
        'renew_order_event_id' => '',
        'change_order_event_id' => '',
        'show_info_to_server_enable' => '',
        'show_protocol_to_server_enable' => '',
        'subscribe_path' => '',
        // server
        'server_token' => 'nullable|min:16',
        'server_pull_interval' => 'integer',
        'server_push_interval' => 'integer',
        'device_limit_mode' => 'integer',
        // frontend
        'frontend_theme_sidebar' => 'nullable|in:dark,light',
        'frontend_theme_header' => 'nullable|in:dark,light',
        'frontend_theme_color' => 'nullable|in:default,darkblue,black,green',
        'frontend_background_url' => 'nullable|url',
        // email
        'email_template' => '',
        'email_host' => '',
        'email_port' => '',
        'email_username' => '',
        'email_password' => '',
        'email_encryption' => '',
        'email_from_address' => '',
        'remind_mail_enable' => '',
        // telegram
        'telegram_bot_enable' => 'boolean',
        'telegram_bot_token' => 'nullable|string|max:255',
        'telegram_discuss_id' => '',
        'telegram_channel_id' => '',
        'telegram_discuss_link' => 'nullable|url',
        'telegram_user_ticket_enable' => 'boolean',
        'telegram_notify_ops_alert' => 'boolean',
        'telegram_notify_ticket_created' => 'boolean',
        'telegram_notify_ticket_replied' => 'boolean',
        'telegram_notify_ticket_closed' => 'boolean',
        'telegram_notify_payment_success' => 'boolean',
        'telegram_notify_notice_published' => 'boolean',
        'telegram_notify_tcping_alert' => 'boolean',
        'telegram_notify_tcping_recover' => 'boolean',
        'telegram_notify_refund_vote' => 'boolean',
        'telegram_notify_refund_status' => 'boolean',
        'telegram_notify_user_risk_detected' => 'boolean',
        'telegram_notify_user_banned' => 'boolean',
        // oauth
        'oauth_linux_do_enable' => 'boolean',
        'oauth_linux_do_client_id' => 'nullable|string|max:255',
        'oauth_linux_do_client_secret' => 'nullable|string|max:255',
        'oauth_linux_do_redirect_uri' => 'nullable|url',
        // app
        'windows_version' => '',
        'windows_download_url' => '',
        'macos_version' => '',
        'macos_download_url' => '',
        'android_version' => '',
        'android_download_url' => '',
        // safe
        'email_whitelist_enable' => 'boolean',
        'email_whitelist_suffix' => 'nullable|array',
        'email_gmail_limit_enable' => 'boolean',
        'captcha_enable' => 'boolean',
        'captcha_type' => 'in:recaptcha,turnstile,recaptcha-v3',
        'recaptcha_enable' => 'boolean',
        'recaptcha_key' => '',
        'recaptcha_site_key' => '',
        'recaptcha_v3_secret_key' => '',
        'recaptcha_v3_site_key' => '',
        'recaptcha_v3_score_threshold' => 'numeric|min:0|max:1',
        'turnstile_secret_key' => '',
        'turnstile_site_key' => '',
        'pow_enable' => 'boolean',
        'pow_difficulty' => 'integer|min:1|max:8',
        'pow_ttl' => 'integer|min:30|max:600',
        'pow_seed_salt' => 'nullable|string|max:128',
        'pow_base_value' => 'nullable|string|max:64',
        'pow_require_ja3' => 'boolean',
        'pow_auto_scale_enable' => 'boolean',
        'pow_auto_max_difficulty' => 'integer|min:1|max:8',
        'email_verify' => 'bool',
        'safe_mode_enable' => 'boolean',
        'force_oauth2_login' => 'boolean',
        'login_token_expire_days' => 'integer|min:0|max:3650',
        'register_limit_by_ip_enable' => 'boolean',
        'register_limit_count' => 'integer',
        'register_limit_expire' => 'integer',
        'secure_path' => 'min:8|regex:/^[\w-]*$/',
        'password_limit_enable' => 'boolean',
        'password_limit_count' => 'integer',
        'password_limit_expire' => 'integer',
        'default_remind_expire' => 'boolean',
        'default_remind_traffic' => 'boolean',
        'subscribe_template_singbox' => 'nullable',
        'subscribe_template_clash' => 'nullable',
        'subscribe_template_clashmeta' => 'nullable',
        'subscribe_template_stash' => 'nullable',
        'subscribe_template_surge' => 'nullable',
        'subscribe_template_surfboard' => 'nullable',
        'rotate_subscription_credentials_daily' => 'boolean',
        'refund_dispute_enable' => 'boolean',
        'node_traffic_records_retention_days' => 'integer|min:1|max:365',
        'user_traffic_usage_logs_retention_days' => 'integer|min:1|max:365',
        'tcping_samples_retention_days' => 'integer|min:1|max:365',
        'tcping_alerts_retention_days' => 'integer|min:1|max:365',
        'audit_logs_retention_days' => 'integer|min:1|max:365',
        // risk review
        'user_risk_review_enable' => 'boolean',
        'user_risk_review_schedule_minutes' => 'integer|min:5|max:1440',
        'user_risk_review_time_window_minutes' => 'integer|min:5|max:1440',
        'user_risk_review_context_hours' => 'integer|min:1|max:168',
        'user_risk_review_min_shared_ip_users' => 'integer|min:2|max:50',
        'user_risk_review_scan_limit' => 'integer|min:1|max:200',
        'user_risk_review_notify_cooldown_minutes' => 'integer|min:5|max:10080',
        'user_risk_review_llm_enable' => 'boolean',
        'user_risk_review_llm_base_url' => 'nullable|url',
        'user_risk_review_llm_api_key' => 'nullable|string|max:500',
        'user_risk_review_llm_model' => 'nullable|string|max:255',
        'user_risk_review_llm_timeout_seconds' => 'integer|min:5|max:120',
        'user_risk_review_llm_temperature' => 'numeric|min:0|max:1'
    ];
    /**
     * Get the validation rules that apply to the request.
     *
     * @return array
     */
    public function rules()
    {
        return self::RULES;
    }

    public function messages()
    {
        // illiteracy prompt
        return [
            'app_url.url' => '站点URL格式不正确，必须携带http(s)://',
            'subscribe_url.url' => '订阅URL格式不正确，必须携带http(s)://',
            'server_token.min' => '通讯密钥长度必须大于16位',
            'tos_url.url' => '服务条款URL格式不正确，必须携带http(s)://',
            'telegram_discuss_link.url' => 'Telegram群组地址必须为URL格式，必须携带http(s)://',
            'oauth_linux_do_redirect_uri.url' => 'OAuth回调地址格式不正确，必须携带http(s)://',
            'logo.url' => 'LOGO URL格式不正确，必须携带https(s)://',
            'user_risk_review_llm_base_url.url' => 'LLM 接口地址格式不正确，必须携带 http(s)://',
            'secure_path.min' => '后台路径长度最小为8位',
            'secure_path.regex' => '后台路径只能为字母或数字',
            'captcha_type.in' => '人机验证类型只能选择 recaptcha、turnstile 或 recaptcha-v3',
            'recaptcha_v3_score_threshold.numeric' => 'reCAPTCHA v3 分数阈值必须为数字',
            'recaptcha_v3_score_threshold.min' => 'reCAPTCHA v3 分数阈值不能小于0',
            'recaptcha_v3_score_threshold.max' => 'reCAPTCHA v3 分数阈值不能大于1',
            'node_traffic_records_retention_days.min' => '节点流量记录保留天数不能小于1天',
            'user_traffic_usage_logs_retention_days.min' => '用户流量记录保留天数不能小于1天',
            'tcping_samples_retention_days.min' => 'TCPing 采样记录保留天数不能小于1天',
            'tcping_alerts_retention_days.min' => 'TCPing 告警记录保留天数不能小于1天',
            'audit_logs_retention_days.min' => '审计记录保留天数不能小于1天'
        ];
    }
}
