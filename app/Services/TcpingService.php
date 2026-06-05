<?php

namespace App\Services;

use App\Models\ServerNode;
use App\Models\TcpingAgent;
use App\Models\TcpingAlert;
use App\Models\TcpingSample;
use App\Models\User;
use App\Services\AccessControlService;
use App\Services\Plugin\HookManager;
use Illuminate\Support\Collection;
use Illuminate\Support\Facades\DB;

class TcpingService
{
    public function findAgentByToken(string $token): ?TcpingAgent
    {
        return TcpingAgent::query()
            ->where('token', $token)
            ->first();
    }

    public function buildAgentConfig(TcpingAgent $agent): array
    {
        $nodes = $this->getMonitorableNodesForAgent($agent);

        return [
            'agent' => [
                'id' => (int) $agent->id,
                'name' => (string) $agent->name,
                'server_time' => time(),
                'pull_interval_seconds' => 60,
            ],
            'targets' => $nodes->map(fn (ServerNode $node) => $this->serializeTarget($node))->values()->all(),
        ];
    }

    public function touchHeartbeat(TcpingAgent $agent): void
    {
        $agent->forceFill([
            'last_heartbeat_at' => time(),
        ])->save();
    }

    public function ingestSamples(TcpingAgent $agent, array $samples): array
    {
        $normalized = collect($samples)
            ->map(fn ($sample) => is_array($sample) ? $sample : null)
            ->filter()
            ->values();

        if ($normalized->isEmpty()) {
            return ['accepted' => 0];
        }

        $nodeIds = $normalized->pluck('node_id')
            ->map(fn ($value) => (int) $value)
            ->filter(fn ($value) => $value > 0)
            ->unique()
            ->values()
            ->all();

        $nodes = $this->getMonitorableNodesForAgent($agent)
            ->whereIn('id', $nodeIds)
            ->keyBy('id');

        $accepted = 0;

        DB::transaction(function () use ($normalized, $nodes, $agent, &$accepted) {
            foreach ($normalized as $item) {
                $nodeId = (int) ($item['node_id'] ?? 0);
                /** @var ServerNode|null $node */
                $node = $nodes->get($nodeId);
                if (!$node) {
                    continue;
                }

                $sampledAt = (int) ($item['sampled_at'] ?? time());
                if ($sampledAt <= 0) {
                    $sampledAt = time();
                }

                $reachable = (bool) ($item['is_reachable'] ?? false);
                $latencyMs = $reachable ? max(0, (int) ($item['latency_ms'] ?? 0)) : null;
                $errorMessage = trim((string) ($item['error_message'] ?? ''));
                $isTimeout = (bool) ($item['is_timeout'] ?? false);

                TcpingSample::query()->create([
                    'node_id' => $node->id,
                    'agent_id' => $agent->id,
                    'is_reachable' => $reachable,
                    'latency_ms' => $latencyMs,
                    'is_timeout' => $isTimeout,
                    'error_message' => $errorMessage !== '' ? $errorMessage : null,
                    'sampled_at' => $sampledAt,
                ]);

                $this->updateNodeStateAndAlerts($node, $sampledAt, $reachable, $latencyMs, $errorMessage, $isTimeout);
                $accepted++;
            }

            $agent->forceFill([
                'last_sync_at' => time(),
            ])->save();
        });

        return ['accepted' => $accepted];
    }

    public function getNodeOverviewForUser(User $user, ServerNode $node, int $hours = 24): array
    {
        $hours = max(1, min(24 * 30, $hours));
        $startAt = now()->subHours($hours)->timestamp;

        $samples = TcpingSample::query()
            ->where('node_id', $node->id)
            ->where('sampled_at', '>=', $startAt)
            ->orderBy('sampled_at')
            ->get([
                'id',
                'agent_id',
                'is_reachable',
                'latency_ms',
                'is_timeout',
                'error_message',
                'sampled_at',
            ]);

        $alerts = TcpingAlert::query()
            ->where('node_id', $node->id)
            ->orderByDesc('triggered_at')
            ->limit(20)
            ->get([
                'id',
                'status',
                'started_at',
                'triggered_at',
                'recovered_at',
                'latest_error',
            ]);

        $reachableCount = $samples->where('is_reachable', true)->count();
        $totalCount = $samples->count();

        $agentIds = $samples->pluck('agent_id')
            ->filter(fn ($value) => $value !== null)
            ->map(fn ($value) => (int) $value)
            ->filter(fn ($value) => $value > 0)
            ->unique()
            ->values()
            ->all();

        $agents = count($agentIds)
            ? TcpingAgent::query()
                ->whereIn('id', $agentIds)
                ->get(['id', 'location_code', 'location_name', 'location_province'])
            : collect();

        return [
            'node' => $this->serializeNodeSummary($node),
            'range_hours' => $hours,
            'stats' => [
                'total_samples' => $totalCount,
                'reachable_samples' => $reachableCount,
                'reachability_rate' => $totalCount > 0 ? round(($reachableCount / $totalCount) * 100, 2) : null,
            ],
            'agents' => $agents->map(function (TcpingAgent $agent) {
                return [
                    'id' => (int) $agent->id,
                    'location_code' => $agent->location_code,
                    'location_name' => $agent->location_name,
                    'location_province' => $agent->location_province,
                    'location_display' => $this->formatAgentLocationDisplay($agent),
                ];
            })->values()->all(),
            'samples' => $samples->map(function (TcpingSample $sample) {
                return [
                    'id' => (int) $sample->id,
                    'agent_id' => $sample->agent_id !== null ? (int) $sample->agent_id : null,
                    'is_reachable' => (bool) $sample->is_reachable,
                    'latency_ms' => $sample->latency_ms !== null ? (int) $sample->latency_ms : null,
                    'is_timeout' => (bool) $sample->is_timeout,
                    'error_message' => $sample->error_message,
                    'sampled_at' => (int) $sample->sampled_at,
                ];
            })->values()->all(),
            'alerts' => $alerts->map(function (TcpingAlert $alert) {
                return [
                    'id' => (int) $alert->id,
                    'status' => (string) $alert->status,
                    'started_at' => (int) $alert->started_at,
                    'triggered_at' => (int) $alert->triggered_at,
                    'recovered_at' => $alert->recovered_at !== null ? (int) $alert->recovered_at : null,
                    'latest_error' => $alert->latest_error,
                ];
            })->values()->all(),
        ];
    }

    public function getAgentsForUser(User $user): Collection
    {
        return TcpingAgent::query()
            ->where('user_id', $user->id)
            ->orderByDesc('id')
            ->get();
    }

    public function createAgentForUser(
        User $user,
        string $name,
        ?string $locationCode = null,
        ?string $locationName = null,
        ?string $locationProvince = null
    ): TcpingAgent
    {
        $agent = new TcpingAgent([
            'user_id' => $user->id,
            'name' => $name,
            'is_enabled' => true,
            'location_code' => $locationCode !== null ? strtoupper(trim($locationCode)) : null,
            'location_name' => $locationName !== null ? trim($locationName) : null,
            'location_province' => $locationProvince,
        ]);
        $agent->ensureToken();
        $agent->save();

        return $agent;
    }

    public function serializeAgent(TcpingAgent $agent): array
    {
        return [
            'id' => (int) $agent->id,
            'name' => (string) $agent->name,
            // TCPing 监控由系统统一调度，用户侧不再允许停用。
            'is_enabled' => true,
            'token' => (string) $agent->token,
            'location_code' => $agent->location_code,
            'location_name' => $agent->location_name,
            'location_province' => $agent->location_province,
            'location_display' => $this->formatAgentLocationDisplay($agent),
            'last_heartbeat_at' => $agent->last_heartbeat_at !== null ? (int) $agent->last_heartbeat_at : null,
            'last_sync_at' => $agent->last_sync_at !== null ? (int) $agent->last_sync_at : null,
        ];
    }

    private function formatAgentLocationDisplay(TcpingAgent $agent): string
    {
        $code = strtoupper(trim((string) ($agent->location_code ?? '')));
        $name = trim((string) ($agent->location_name ?? ''));
        $province = trim((string) ($agent->location_province ?? ''));

        if ($code === 'CN') {
            if ($province !== '') {
                return '中国大陆 · ' . $province;
            }
            if ($name !== '') {
                return $name;
            }
            return '中国大陆';
        }

        if ($name !== '') {
            return $name;
        }
        if ($code !== '') {
            return $code;
        }

        return '未设置';
    }

    public function serializeNodeSummary(ServerNode $node): array
    {
        $activeAlert = TcpingAlert::query()
            ->where('node_id', $node->id)
            ->where('status', TcpingAlert::STATUS_ACTIVE)
            ->orderByDesc('triggered_at')
            ->first();

        return [
            'id' => (int) $node->id,
            'name' => (string) $node->name,
            'protocol' => (string) $node->protocol,
            'location_name' => $node->location_name,
            'tcping_enabled' => $node->isTcpingMonitorable(),
            'tcping_status' => $node->isTcpingMonitorable()
                ? (string) ($node->tcping_last_status ?: 'unknown')
                : 'unsupported',
            'tcping_last_latency_ms' => $node->isTcpingMonitorable() && $node->tcping_last_latency_ms !== null
                ? (int) $node->tcping_last_latency_ms
                : null,
            'tcping_last_error' => $node->tcping_last_error,
            'tcping_last_sampled_at' => $node->isTcpingMonitorable() && $node->tcping_last_sampled_at !== null
                ? (int) $node->tcping_last_sampled_at
                : null,
            'tcping_alert_active' => (bool) $activeAlert,
            'tcping_active_alert' => $activeAlert ? [
                'id' => (int) $activeAlert->id,
                'started_at' => (int) $activeAlert->started_at,
                'triggered_at' => (int) $activeAlert->triggered_at,
                'latest_error' => $activeAlert->latest_error,
            ] : null,
        ];
    }

    private function getMonitorableNodesForAgent(TcpingAgent $agent): Collection
    {
        $user = User::query()->whereKey((int) $agent->user_id)->first();
        if (!$user) {
            return collect();
        }

        // 超管 Agent 探测全部节点；普通用户探测“自己拥有 + 自己可用(订阅/权限)”的节点。
        if ((bool) $user->is_super_admin) {
            return ServerNode::query()
                ->orderByDesc('id')
                ->get()
                ->filter(fn (ServerNode $node) => $node->isTcpingMonitorable())
                ->values();
        }

        $ownedNodes = ServerNode::query()
            ->where('user_id', $user->id)
            ->orderByDesc('id')
            ->get();

        $accessibleNodes = app(AccessControlService::class)->getAccessibleNodesForUser($user);

        return $ownedNodes
            ->concat($accessibleNodes)
            ->unique(fn (ServerNode $node) => (int) $node->id)
            ->sortByDesc(fn (ServerNode $node) => (int) $node->id)
            ->filter(fn (ServerNode $node) => $node->isTcpingMonitorable())
            ->values();
    }

    private function serializeTarget(ServerNode $node): array
    {
        return [
            'node_id' => (int) $node->id,
            'name' => (string) $node->name,
            'host' => (string) ($node->tcping_host ?: $node->host),
            'port' => (int) ($node->tcping_port ?: $node->port),
            'interval_seconds' => max(15, (int) ($node->tcping_interval_seconds ?: 60)),
            'timeout_ms' => max(500, (int) ($node->tcping_timeout_ms ?: 3000)),
            'alert_after_seconds' => max(60, (int) ($node->tcping_alert_after_seconds ?: 300)),
            'recover_after_seconds' => max(30, (int) ($node->tcping_recover_after_seconds ?: 120)),
        ];
    }

    private function updateNodeStateAndAlerts(
        ServerNode $node,
        int $sampledAt,
        bool $reachable,
        ?int $latencyMs,
        string $errorMessage,
        bool $isTimeout
    ): void {
        $triggeredAlert = null;
        $resolvedAlert = null;

        $node->tcping_last_status = $reachable ? 'online' : 'offline';
        $node->tcping_last_latency_ms = $reachable ? $latencyMs : null;
        $node->tcping_last_error = $reachable ? null : (($errorMessage !== '' ? $errorMessage : ($isTimeout ? 'timeout' : 'unreachable')));
        $node->tcping_last_sampled_at = $sampledAt;

        $activeAlert = TcpingAlert::query()
            ->where('node_id', $node->id)
            ->where('status', TcpingAlert::STATUS_ACTIVE)
            ->latest('triggered_at')
            ->first();

        if ($reachable) {
            $node->tcping_recovered_since = $node->tcping_recovered_since ?: $sampledAt;
            $node->tcping_outage_since = null;

            if ($activeAlert) {
                $recoverAfter = max(30, (int) ($node->tcping_recover_after_seconds ?: 120));
                if (($sampledAt - (int) ($node->tcping_recovered_since ?? $sampledAt)) >= $recoverAfter) {
                    $activeAlert->forceFill([
                        'status' => TcpingAlert::STATUS_RESOLVED,
                        'recovered_at' => $sampledAt,
                        'latest_error' => null,
                    ])->save();
                    $resolvedAlert = $activeAlert->fresh();
                }
            }
        } else {
            $node->tcping_recovered_since = null;
            $node->tcping_outage_since = $node->tcping_outage_since ?: $sampledAt;

            if ($activeAlert) {
                $activeAlert->forceFill([
                    'latest_error' => $node->tcping_last_error,
                ])->save();
            } else {
                $alertAfter = max(60, (int) ($node->tcping_alert_after_seconds ?: 300));
                if (($sampledAt - (int) ($node->tcping_outage_since ?? $sampledAt)) >= $alertAfter) {
                    $triggeredAlert = TcpingAlert::query()->create([
                        'node_id' => $node->id,
                        'user_id' => $node->user_id,
                        'status' => TcpingAlert::STATUS_ACTIVE,
                        'started_at' => (int) ($node->tcping_outage_since ?? $sampledAt),
                        'triggered_at' => $sampledAt,
                        'latest_error' => $node->tcping_last_error,
                    ]);
                }
            }
        }

        $node->save();

        if ($triggeredAlert) {
            HookManager::call('tcping.alert.triggered', [
                'node' => $node->fresh(['user']),
                'alert' => $triggeredAlert->fresh(),
            ]);
        }

        if ($resolvedAlert) {
            HookManager::call('tcping.alert.recovered', [
                'node' => $node->fresh(['user']),
                'alert' => $resolvedAlert,
            ]);
        }
    }
}
