<?php

namespace App\Console\Commands;

use App\Models\User;
use Illuminate\Console\Command;

class RotateSubscriptionCredentials extends Command
{
    protected $signature = 'subscription:rotate-credentials';

    protected $description = 'Rotate delivered subscription credentials without changing subscription URLs';

    public function handle(): int
    {
        if (!(bool) admin_setting('rotate_subscription_credentials_daily', 0)) {
            $this->info('Rotation disabled.');
            return self::SUCCESS;
        }

        $rotated = 0;
        User::query()
            ->orderBy('id')
            ->chunkById(500, function ($users) use (&$rotated) {
                foreach ($users as $user) {
                    $user->forceFill([
                        'subscription_credential_version' => max(0, (int) ($user->subscription_credential_version ?? 0)) + 1,
                        'last_subscription_credential_rotation_at' => time(),
                    ])->save();
                    $rotated++;
                }
            });

        $this->info("Rotated {$rotated} users.");
        return self::SUCCESS;
    }
}
