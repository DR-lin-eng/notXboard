<?php

namespace App\Services;

use App\Models\User;

class SubscriptionCredentialService
{
    public function isRotationEnabled(): bool
    {
        return (bool) admin_setting('rotate_subscription_credentials_daily', 0);
    }

    public function getEffectiveUuid(User $user): string
    {
        $baseUuid = (string) ($user->uuid ?? '');
        if ($baseUuid === '') {
            return $baseUuid;
        }

        if (!$this->isRotationEnabled()) {
            return $baseUuid;
        }

        $version = max(0, (int) ($user->subscription_credential_version ?? 0));
        if ($version <= 0) {
            return $baseUuid;
        }

        return $this->uuidV5Like($baseUuid . '|' . $version);
    }

    private function uuidV5Like(string $seed): string
    {
        $hash = sha1($seed);
        $timeLow = substr($hash, 0, 8);
        $timeMid = substr($hash, 8, 4);
        $timeHi = substr($hash, 12, 4);
        $clockSeq = substr($hash, 16, 4);
        $node = substr($hash, 20, 12);

        $timeHi = sprintf('%04x', (hexdec($timeHi) & 0x0fff) | 0x5000);
        $clockSeq = sprintf('%04x', (hexdec($clockSeq) & 0x3fff) | 0x8000);

        return sprintf(
            '%s-%s-%s-%s-%s',
            $timeLow,
            $timeMid,
            $timeHi,
            $clockSeq,
            $node
        );
    }
}
