<?php

namespace App\Console\Commands;

use App\Models\UserOnlineSession;
use Illuminate\Console\Command;
use Illuminate\Support\Facades\Log;

class CleanupExpiredNodeSessions extends Command
{
    protected $signature = 'cleanup:expired-node-sessions {--minutes=10 : Consider sessions expired after N minutes}';

    protected $description = 'Delete expired ServerNode online sessions';

    public function handle(): int
    {
        $minutes = max(1, (int) $this->option('minutes'));

        try {
            $deleted = UserOnlineSession::query()
                ->where('last_activity', '<', now()->subMinutes($minutes))
                ->delete();

            $this->info("Expired node sessions cleaned. Deleted: {$deleted}");
            return self::SUCCESS;
        } catch (\Throwable $e) {
            Log::error('CleanupExpiredNodeSessions failed', ['error' => $e->getMessage()]);
            $this->error('Cleanup failed: ' . $e->getMessage());
            return self::FAILURE;
        }
    }
}

