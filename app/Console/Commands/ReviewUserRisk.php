<?php

namespace App\Console\Commands;

use App\Services\UserRiskReviewService;
use Illuminate\Console\Command;

class ReviewUserRisk extends Command
{
    protected $signature = 'review:user-risk {--force : 忽略时间间隔立即执行} {--limit= : 本次最多扫描的共享 IP 数量}';

    protected $description = '审查共享 IP 风险用户并生成预审意见';

    public function handle(UserRiskReviewService $service): int
    {
        $result = $service->scanAndReview(
            limit: $this->option('limit') !== null ? (int) $this->option('limit') : null,
            force: (bool) $this->option('force')
        );

        $this->line(json_encode($result, JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES));

        return self::SUCCESS;
    }
}
