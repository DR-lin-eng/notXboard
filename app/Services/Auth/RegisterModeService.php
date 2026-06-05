<?php

namespace App\Services\Auth;

use App\Models\User;
use App\Services\OAuth\LinuxDoOAuthService;

class RegisterModeService
{
    public const MODE_ALL = 'all';
    public const MODE_EMAIL_ONLY = 'email_only';
    public const MODE_OAUTH_ONLY = 'oauth_only';
    public const MODE_CLOSED = 'closed';

    public function getRegisterMode(): string
    {
        $raw = strtolower(trim((string) admin_setting('register_mode', '')));
        if ($raw === '') {
            return (int) admin_setting('stop_register', 0) === 1
                ? self::MODE_CLOSED
                : self::MODE_ALL;
        }

        return match ($raw) {
            self::MODE_EMAIL_ONLY,
            self::MODE_OAUTH_ONLY,
            self::MODE_CLOSED => $raw,
            default => self::MODE_ALL,
        };
    }

    public function allowsEmailRegistration(): bool
    {
        return in_array($this->getRegisterMode(), [self::MODE_ALL, self::MODE_EMAIL_ONLY], true);
    }

    public function allowsOauthRegistration(): bool
    {
        if (!LinuxDoOAuthService::isAvailable()) {
            return false;
        }

        $mode = $this->getRegisterMode();
        if ($mode === self::MODE_CLOSED) {
            return false;
        }

        return in_array($mode, [self::MODE_ALL, self::MODE_OAUTH_ONLY], true);
    }

    public function allowsAnyRegistration(): bool
    {
        return $this->allowsEmailRegistration() || $this->allowsOauthRegistration();
    }

    public function allowsEmailLogin(?User $user = null): bool
    {
        if (!(bool) admin_setting('force_oauth2_login', 0)) {
            return true;
        }

        if (!LinuxDoOAuthService::isAvailable()) {
            return true;
        }

        return (bool) ($user?->is_super_admin);
    }
}
