<?php

namespace App\Console\Commands;

use App\Services\OrderRefundService;
use Illuminate\Console\Command;

class FinalizeRefundVotes extends Command
{
    protected $signature = 'refunds:finalize-votes {--limit=200}';
    protected $description = 'Finalize expired refund votings and auto-resolve them.';

    public function handle(): int
    {
        $limit = (int) $this->option('limit');
        $count = app(OrderRefundService::class)->finalizeExpiredVotings(max(1, $limit));
        $this->info("processed={$count}");
        return self::SUCCESS;
    }
}

