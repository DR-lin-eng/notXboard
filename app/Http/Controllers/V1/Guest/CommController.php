<?php

namespace App\Http\Controllers\V1\Guest;

use App\Http\Controllers\Controller;
use App\Services\Auth\RegisterModeService;
use App\Services\OAuth\LinuxDoOAuthService;
use App\Services\PowService;
use App\Services\Plugin\HookManager;
use App\Utils\Dict;
use App\Utils\Helper;
use Illuminate\Support\Facades\Http;

class CommController extends Controller
{
    public function config()
    {
        $registerModeService = app(RegisterModeService::class);
        $powService = app(PowService::class);
        $oauthAvailable = LinuxDoOAuthService::isAvailable();

        $data = [
            'tos_url' => admin_setting('tos_url'),
            'is_email_verify' => config('ops.telegram_only_mode')
                ? 0
                : ((int) admin_setting('email_verify', 0) ? 1 : 0),
            'is_invite_force' => (int) admin_setting('invite_force', 0) ? 1 : 0,
            'register_mode' => $registerModeService->getRegisterMode(),
            'allow_email_register' => $registerModeService->allowsEmailRegistration() ? 1 : 0,
            'allow_oauth_register' => $registerModeService->allowsOauthRegistration() ? 1 : 0,
            'email_whitelist_suffix' => (int) admin_setting('email_whitelist_enable', 0)
                ? Helper::getEmailSuffix()
                : 0,
            'is_captcha' => (int) admin_setting('captcha_enable', 0) ? 1 : 0,
            'captcha_type' => admin_setting('captcha_type', 'recaptcha'),
            'recaptcha_site_key' => admin_setting('recaptcha_site_key'),
            'recaptcha_v3_site_key' => admin_setting('recaptcha_v3_site_key'),
            'recaptcha_v3_score_threshold' => admin_setting('recaptcha_v3_score_threshold', 0.5),
            'turnstile_site_key' => admin_setting('turnstile_site_key'),
            'pow_enable' => (int) admin_setting('pow_enable', 0) ? 1 : 0,
            'pow_difficulty' => (int) admin_setting('pow_difficulty', 4),
            'pow_effective_difficulty' => $powService->getDifficulty(),
            'pow_ttl' => (int) admin_setting('pow_ttl', 120),
            'pow_algo' => 'sha256-prefix-zeros',
            'app_description' => admin_setting('app_description'),
            'app_url' => admin_setting('app_url'),
            'logo' => admin_setting('logo'),
            'windows_version' => admin_setting('windows_version'),
            'windows_download_url' => admin_setting('windows_download_url'),
            'macos_version' => admin_setting('macos_version'),
            'macos_download_url' => admin_setting('macos_download_url'),
            'android_version' => admin_setting('android_version'),
            'android_download_url' => admin_setting('android_download_url'),
            'force_oauth2_login' => (int) admin_setting('force_oauth2_login', 0) ? 1 : 0,
            'oauth_linux_do_enable' => $oauthAvailable ? 1 : 0,
            // 保持向后兼容
            'is_recaptcha' => (int) admin_setting('captcha_enable', 0) ? 1 : 0,
        ];

        $data = HookManager::filter('guest_comm_config', $data);

        return $this->success($data);
    }
}
