<?php

namespace App\Services;

use App\Models\Plan;
use App\Models\User;
use App\Models\UserPaymentProfile;
use App\Support\UrlSecurity;
use Illuminate\Support\Facades\Crypt;
use InvalidArgumentException;

class PaymentProfileService
{
    public const PROVIDER_EPAY = 'epay';

    public function getEpayProfileForUser(?int $userId): ?array
    {
        if (!$userId) {
            return null;
        }

        $profile = UserPaymentProfile::query()
            ->where('user_id', $userId)
            ->where('provider', self::PROVIDER_EPAY)
            ->first();

        if (!$profile || !$profile->pid || !$profile->key_encrypted || !$profile->url) {
            return null;
        }

        try {
            $url = UrlSecurity::normalizeHttpUrl((string) $profile->url, true);
            $submitPath = UrlSecurity::normalizeRelativePath($profile->submit_path, '/pay/submit.php');
        } catch (InvalidArgumentException) {
            return null;
        }

        return [
            'pid' => $profile->pid,
            'key' => Crypt::decryptString($profile->key_encrypted),
            'url' => $url,
            'submit_path' => $submitPath,
            'use_post' => (bool) $profile->use_post,
            'sitename' => $profile->sitename,
            'device' => $profile->device,
        ];
    }

    public function getSponsorEpayProfile(): ?array
    {
        $url = (string) admin_setting('sponsor_epay_url', '');
        $pid = (string) admin_setting('sponsor_epay_pid', '');
        $key = (string) admin_setting('sponsor_epay_key', '');

        if ($url === '' || $pid === '' || $key === '') {
            return null;
        }

        try {
            $url = UrlSecurity::normalizeHttpUrl($url);
            $submitPath = UrlSecurity::normalizeRelativePath((string) admin_setting('sponsor_epay_submit_path', '/pay/submit.php'), '/pay/submit.php');
        } catch (InvalidArgumentException) {
            return null;
        }

        return [
            'pid' => $pid,
            'key' => $key,
            'url' => $url,
            'submit_path' => $submitPath,
            'use_post' => (bool) admin_setting('sponsor_epay_use_post', 1),
            'sitename' => (string) admin_setting('sponsor_epay_sitename', ''),
            'device' => (string) admin_setting('sponsor_epay_device', ''),
        ];
    }

    /**
     * Decide which epay credentials should be used to pay for a plan.
     *
     * - For node plans: default to plan owner (personal admin) profile.
     * - When $payTo === 'sponsor': use sponsor profile (super admin configured).
     */
    public function resolveEpayForPlan(Plan $plan, ?string $payTo = null): ?array
    {
        if ($payTo === 'sponsor') {
            return $this->getSponsorEpayProfile();
        }

        if (($plan->scope ?? Plan::SCOPE_LEGACY) === Plan::SCOPE_NODE) {
            $ownerId = (int) ($plan->owner_user_id ?? 0);
            $ownerProfile = $this->getEpayProfileForUser($ownerId);
            if ($ownerProfile) {
                return $ownerProfile;
            }
        }

        return null;
    }
}
