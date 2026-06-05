<?php

namespace App\Console\Commands;

use App\Models\User;
use App\Services\OAuth\LinuxDoOAuthService;
use App\Services\OAuth\UserSyncService;
use Illuminate\Console\Command;
use Illuminate\Support\Facades\Log;

class SyncLinuxDoUsers extends Command
{
    protected $signature = 'oauth:sync-linux-do-users {--chunk=200 : Chunk size}';

    protected $description = 'Sync Linux DO Connect user info (trust_level, active, silenced) and refresh tokens when needed';

    public function handle(UserSyncService $userSyncService): int
    {
        if (empty(config('services.linux_do.client_id')) || empty(config('services.linux_do.client_secret'))) {
            $this->warn('Linux DO OAuth2 credentials are not configured; skipping.');
            return self::SUCCESS;
        }

        $oauthService = new LinuxDoOAuthService();
        $chunkSize = max(50, (int) $this->option('chunk'));

        $total = 0;
        $synced = 0;
        $failed = 0;

        User::query()
            ->where('oauth_provider', 'linux_do')
            ->whereNotNull('linux_do_id')
            ->orderBy('id')
            ->chunkById($chunkSize, function ($users) use (
                &$total,
                &$synced,
                &$failed,
                $oauthService,
                $userSyncService
            ) {
                foreach ($users as $user) {
                    $total++;
                    try {
                        if (!$userSyncService->shouldUpdateUserInfo($user)) {
                            continue;
                        }

                        if ($userSyncService->syncUserInfo($user, $oauthService)) {
                            $synced++;
                        } else {
                            $failed++;
                        }
                    } catch (\Throwable $e) {
                        $failed++;
                        Log::warning('Linux DO user sync failed', [
                            'user_id' => $user->id,
                            'error' => $e->getMessage(),
                        ]);
                    }
                }
            });

        $this->info("Linux DO sync complete. total={$total} synced={$synced} failed={$failed}");
        return $failed > 0 ? self::FAILURE : self::SUCCESS;
    }
}

