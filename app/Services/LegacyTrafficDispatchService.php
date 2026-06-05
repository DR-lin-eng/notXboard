<?php

namespace App\Services;

use App\Services\LegacyTraffic\LegacyOnlineStatusService;
use App\Services\LegacyTraffic\LegacyTrafficStatsService;
use App\Services\LegacyTraffic\LegacyTrafficUsageService;

class LegacyTrafficDispatchService
{
    public function __construct(
        private readonly LegacyTrafficUsageService $trafficUsageService,
        private readonly LegacyTrafficStatsService $trafficStatsService,
        private readonly LegacyOnlineStatusService $onlineStatusService,
    ) {
    }

    public function applyTrafficFetch(array $server, array $data): void
    {
        $this->trafficUsageService->applyTrafficFetch($server, $data);
    }

    public function applyUserStat(array $server, array $data, string $recordType = 'd'): void
    {
        $this->trafficStatsService->applyUserStat($server, $data, $recordType);
    }

    public function applyServerStat(array $server, array $data, string $protocol, string $recordType = 'd'): void
    {
        $this->trafficStatsService->applyServerStat($server, $data, $protocol, $recordType);
    }

    public function updateAliveData(array $data, string $nodeType, int $nodeId): void
    {
        $this->onlineStatusService->updateAliveData($data, $nodeType, $nodeId);
    }
}
