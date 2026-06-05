<?php

namespace App\Services\Auth;

use App\Exceptions\ApiException;
use App\Models\User;
use App\Services\AuthService;

class BannedUserGuard
{
    public function rejectIfBanned(?User $user): void
    {
        if (!$user || !$user->banned) {
            return;
        }

        app(AuthService::class, ['user' => $user])->removeAllSessions();
        throw new ApiException($user->getSuspensionMessage(), 403);
    }
}
