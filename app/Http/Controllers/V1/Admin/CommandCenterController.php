<?php

namespace App\Http\Controllers\V1\Admin;

use App\Http\Controllers\Controller;
use App\Models\AuditLog;
use App\Models\Log as LogModel;
use App\Models\NodeTrafficRecord;
use App\Models\Order;
use App\Models\OrderRefundRequest;
use App\Models\ServerNode;
use App\Models\TcpingAgent;
use App\Models\TcpingAlert;
use App\Models\TcpingSample;
use App\Models\Ticket;
use App\Models\User;
use App\Models\UserOnlineSession;
use App\Models\UserPlanSubscription;
use App\Utils\CacheKey;
use Carbon\CarbonImmutable;
use Illuminate\Http\JsonResponse;
use Illuminate\Support\Collection;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\DB;

class CommandCenterController extends Controller
{
    private const SNAPSHOT_CACHE_KEY = 'admin:command-center:snapshot';
    private const SNAPSHOT_CACHE_TTL_SECONDS = 10;
    private const TRAFFIC_SNAPSHOT_KEY = 'admin:command-center:throughput-snapshot';
    private const REFRESH_INTERVAL_SECONDS = 20;

    public function __construct()
    {
        $this->middleware('auth:sanctum');
        $this->middleware('admin.super');
    }

    public function show(): JsonResponse
    {
        $refreshInterval = self::REFRESH_INTERVAL_SECONDS;

        if (app()->runningUnitTests()) {
            return $this->success($this->buildSnapshot($refreshInterval));
        }

        $snapshot = Cache::remember(
            self::SNAPSHOT_CACHE_KEY,
            now()->addSeconds(self::SNAPSHOT_CACHE_TTL_SECONDS),
            fn () => $this->buildSnapshot($refreshInterval)
        );

        return $this->success($snapshot);
    }

    private function buildSnapshot(int $refreshInterval): array
    {
        $now = CarbonImmutable::now();
        $today = $now->toDateString();
        $windowStart = $now->subDays(6)->toDateString();

        $onlineSessionsByNode = DB::table('user_online_sessions')
            ->select(
                'node_id',
                DB::raw('COUNT(DISTINCT user_id) AS active_users'),
                DB::raw('COALESCE(SUM(connection_count), 0) AS active_connections')
            )
            ->where('last_activity', '>=', $now->subMinutes(5)->toDateTimeString())
            ->groupBy('node_id')
            ->get()
            ->mapWithKeys(fn ($row) => [
                (int) $row->node_id => [
                    'active_users' => (int) ($row->active_users ?? 0),
                    'active_connections' => (int) ($row->active_connections ?? 0),
                ],
            ]);

        $weeklyTrafficByNode = NodeTrafficRecord::query()
            ->select(
                'node_id',
                DB::raw('COALESCE(SUM(upload_traffic + download_traffic), 0) AS traffic_kb'),
                DB::raw('COUNT(DISTINCT user_id) AS unique_users')
            )
            ->whereBetween('record_date', [$windowStart, $today])
            ->groupBy('node_id')
            ->get()
            ->mapWithKeys(fn ($row) => [
                (int) $row->node_id => [
                    'traffic_kb' => (int) ($row->traffic_kb ?? 0),
                    'unique_users' => (int) ($row->unique_users ?? 0),
                ],
            ]);

        $nodes = ServerNode::query()
            ->with(['owner:id,email,linux_do_username,linux_do_name'])
            ->orderBy('id')
            ->get([
                'id',
                'user_id',
                'name',
                'host',
                'port',
                'protocol',
                'location_code',
                'location_name',
                'traffic_limit',
                'traffic_used',
                'traffic_multiplier',
                'status',
                'tcping_enabled',
                'tcping_port',
                'tcping_last_status',
                'tcping_last_latency_ms',
                'tcping_last_error',
                'tcping_last_sampled_at',
                'created_at',
                'updated_at',
            ]);

        $monitorableNodeIds = $nodes
            ->filter(fn (ServerNode $node) => $node->isTcpingMonitorable())
            ->pluck('id')
            ->map(fn ($id) => (int) $id)
            ->values()
            ->all();

        $activeAlertRows = TcpingAlert::query()
            ->where('status', TcpingAlert::STATUS_ACTIVE)
            ->when(
                $monitorableNodeIds !== [],
                fn ($query) => $query->whereIn('node_id', $monitorableNodeIds),
                fn ($query) => $query->whereRaw('1=0')
            )
            ->orderByDesc('triggered_at')
            ->get(['node_id', 'triggered_at', 'latest_error']);

        $activeAlertCounts = $activeAlertRows->countBy(fn (TcpingAlert $alert) => (int) $alert->node_id);
        $latestActiveAlertByNode = $activeAlertRows
            ->unique(fn (TcpingAlert $alert) => (int) $alert->node_id)
            ->keyBy(fn (TcpingAlert $alert) => (int) $alert->node_id);

        $nodeSnapshots = $nodes->map(function (ServerNode $node) use (
            $now,
            $onlineSessionsByNode,
            $weeklyTrafficByNode,
            $latestActiveAlertByNode,
            $activeAlertCounts
        ) {
            $sessionInfo = $onlineSessionsByNode->get((int) $node->id, [
                'active_users' => 0,
                'active_connections' => 0,
            ]);
            $trafficInfo = $weeklyTrafficByNode->get((int) $node->id, [
                'traffic_kb' => 0,
                'unique_users' => 0,
            ]);
            $activeAlert = $latestActiveAlertByNode->get((int) $node->id);
            $onlineStatus = $node->getOnlineStatus();
            $trafficUsagePercentage = round($node->getTrafficUsagePercentage(), 2);
            $tcpingMonitorable = $node->isTcpingMonitorable();
            $tcpingStatus = $tcpingMonitorable
                ? strtolower((string) ($node->tcping_last_status ?: ($activeAlert ? 'offline' : 'unknown')))
                : 'unsupported';

            $watchScore = 0;
            if ($onlineStatus !== 'online') {
                $watchScore += 140;
            }
            if ($tcpingMonitorable && $tcpingStatus === 'offline') {
                $watchScore += 90;
            }
            if ($activeAlert) {
                $watchScore += 110;
            }
            if ($node->status === ServerNode::STATUS_MAINTENANCE) {
                $watchScore += 35;
            }
            if ($node->status === ServerNode::STATUS_DEPLOYING) {
                $watchScore += 18;
            }
            $watchScore += min(100, (int) round($trafficUsagePercentage));
            $watchScore += min(50, (int) ($trafficInfo['unique_users'] * 6));
            $watchScore += min(45, (int) ($sessionInfo['active_users'] * 5));

            return [
                'id' => (int) $node->id,
                'name' => (string) $node->name,
                'host' => (string) $node->host,
                'port' => (int) $node->port,
                'protocol' => (string) $node->protocol,
                'protocol_label' => $this->protocolLabel($node->protocol),
                'location_code' => $node->location_code ? (string) $node->location_code : '',
                'location_name' => $node->location_name ? (string) $node->location_name : '未标注地区',
                'status' => (string) $node->status,
                'online_status' => $onlineStatus,
                'traffic_limit_kb' => (int) ($node->traffic_limit ?? 0),
                'traffic_used_kb' => (int) ($node->traffic_used ?? 0),
                'traffic_usage_percentage' => $trafficUsagePercentage,
                'traffic_remaining_kb' => $node->traffic_limit > 0 ? (int) $node->getRemainingTraffic() : null,
                'traffic_multiplier' => (float) ($node->traffic_multiplier ?? 1),
                'weekly_traffic_kb' => (int) $trafficInfo['traffic_kb'],
                'weekly_unique_users' => (int) $trafficInfo['unique_users'],
                'online_users' => (int) $sessionInfo['active_users'],
                'active_connections' => (int) $sessionInfo['active_connections'],
                'owner_email' => (string) ($node->owner?->email ?? '-'),
                'owner_name' => (string) ($node->owner?->linux_do_name ?: $node->owner?->linux_do_username ?: $node->owner?->email ?: '-'),
                'tcping_enabled' => $tcpingMonitorable,
                'tcping_status' => $tcpingStatus,
                'tcping_last_latency_ms' => $tcpingMonitorable && $node->tcping_last_latency_ms !== null
                    ? (int) $node->tcping_last_latency_ms
                    : null,
                'tcping_last_sampled_at' => $tcpingMonitorable && $node->tcping_last_sampled_at !== null
                    ? (int) $node->tcping_last_sampled_at
                    : null,
                'tcping_last_error' => $tcpingMonitorable
                    ? ($node->tcping_last_error ? (string) $node->tcping_last_error : (string) ($activeAlert?->latest_error ?? ''))
                    : '',
                'active_alert' => $activeAlert ? [
                    'count' => (int) ($activeAlertCounts->get((int) $node->id, 1)),
                    'triggered_at' => (int) ($activeAlert->triggered_at ?? 0),
                    'duration_seconds' => max(0, $now->timestamp - (int) ($activeAlert->triggered_at ?? $now->timestamp)),
                    'latest_error' => (string) ($activeAlert->latest_error ?? ''),
                ] : null,
                'watch_score' => $watchScore,
            ];
        });

        $watchlist = $this->decorateWatchlistWithSamples(
            $nodeSnapshots,
            $now
        );

        $todayTraffic = NodeTrafficRecord::query()
            ->selectRaw(
                'COALESCE(SUM(upload_traffic), 0) AS upload_kb, ' .
                'COALESCE(SUM(download_traffic), 0) AS download_kb, ' .
                'COALESCE(SUM(upload_traffic + download_traffic), 0) AS total_kb, ' .
                'COUNT(DISTINCT user_id) AS unique_users'
            )
            ->where('record_date', $today)
            ->first();

        $completedOrders24h = Order::query()
            ->where('status', Order::STATUS_COMPLETED)
            ->where('paid_at', '>=', $now->subDay()->timestamp);

        $throughput = $this->resolveThroughputSnapshot($now);

        $activeSubscriptions = UserPlanSubscription::query()
            ->active($now->timestamp)
            ->count();

        $overview = [
            'total_users' => User::query()->count(),
            'live_users' => User::query()->where('t', '>=', $now->subMinutes(5)->timestamp)->count(),
            'active_subscriptions' => $activeSubscriptions,
            'total_nodes' => $nodeSnapshots->count(),
            'online_nodes' => $nodeSnapshots->where('online_status', 'online')->count(),
            'maintenance_nodes' => $nodeSnapshots->where('status', ServerNode::STATUS_MAINTENANCE)->count(),
            'traffic_hot_nodes' => $nodeSnapshots->filter(fn (array $node) => (float) $node['traffic_usage_percentage'] >= 80)->count(),
            'tcping_enabled_nodes' => $nodeSnapshots->where('tcping_enabled', true)->count(),
            'tcping_alerts_active' => $activeAlertRows->count(),
            'tcping_agents_total' => TcpingAgent::query()->count(),
            'tcping_agents_online' => TcpingAgent::query()
                ->whereNotNull('last_heartbeat_at')
                ->where('last_heartbeat_at', '>=', $now->subMinutes(3)->timestamp)
                ->count(),
            'open_tickets' => Ticket::query()->where('status', Ticket::STATUS_OPENING)->count(),
            'pending_refunds' => OrderRefundRequest::query()
                ->whereIn('status', [OrderRefundRequest::STATUS_PENDING, OrderRefundRequest::STATUS_VOTING])
                ->count(),
            'completed_orders_24h' => $completedOrders24h->count(),
            'revenue_24h_amount' => (int) ($completedOrders24h->sum('total_amount') ?? 0),
            'traffic_today_kb' => (int) ($todayTraffic->total_kb ?? 0),
            'traffic_today_unique_users' => (int) ($todayTraffic->unique_users ?? 0),
            'throughput_upload_bps' => (int) ($throughput['upload_bps'] ?? 0),
            'throughput_download_bps' => (int) ($throughput['download_bps'] ?? 0),
        ];

        return [
            'generated_at' => $now->timestamp,
            'refresh_interval_seconds' => $refreshInterval,
            'overview' => $overview,
            'system' => $this->buildSystemStatus($now),
            'traffic_trend' => $this->buildTrafficTrend($now),
            'protocol_distribution' => $this->buildProtocolDistribution($nodeSnapshots),
            'region_distribution' => $this->buildRegionDistribution($nodeSnapshots),
            'top_users' => $this->buildTopUsers($now, $windowStart, $today),
            'node_watchlist' => $watchlist,
            'hot_nodes' => $watchlist,
            'tickets' => $this->buildOpenTickets(),
            'refunds' => $this->buildPendingRefunds(),
            'tcping_agents' => $this->buildTcpingAgents($now),
            'tcping_alerts' => $this->buildActiveTcpingAlerts($now),
            'audit_stream' => $this->buildAuditStream(),
        ];
    }

    private function buildSystemStatus(CarbonImmutable $now): array
    {
        $scheduleLastRuntime = Cache::get(CacheKey::get('SCHEDULE_LAST_CHECK_AT', null));
        $scheduleLastRuntime = is_numeric($scheduleLastRuntime) ? (int) $scheduleLastRuntime : null;
        $errorsLast24h = LogModel::query()
            ->where('level', 'ERROR')
            ->where('created_at', '>=', $now->subDay()->timestamp)
            ->count();
        $warningsLast24h = LogModel::query()
            ->where('level', 'WARNING')
            ->where('created_at', '>=', $now->subDay()->timestamp)
            ->count();

        return [
            'schedule_ok' => $scheduleLastRuntime !== null && ($now->timestamp - $scheduleLastRuntime) < 120,
            'schedule_last_runtime' => $scheduleLastRuntime,
            'horizon' => $this->buildHorizonStatus(),
            'logs' => [
                'info' => LogModel::query()->where('level', 'INFO')->count(),
                'warning' => LogModel::query()->where('level', 'WARNING')->count(),
                'error' => LogModel::query()->where('level', 'ERROR')->count(),
                'total' => LogModel::query()->count(),
                'errors_last_24h' => $errorsLast24h,
                'warnings_last_24h' => $warningsLast24h,
            ],
        ];
    }

    private function buildHorizonStatus(): array
    {
        if (!interface_exists('Laravel\\Horizon\\Contracts\\MasterSupervisorRepository')) {
            return [
                'available' => false,
                'ok' => null,
                'master_count' => 0,
                'paused_masters' => 0,
            ];
        }

        try {
            $masters = collect(app('Laravel\\Horizon\\Contracts\\MasterSupervisorRepository')->all());
        } catch (\Throwable $exception) {
            return [
                'available' => false,
                'ok' => null,
                'master_count' => 0,
                'paused_masters' => 0,
            ];
        }

        $paused = $masters->filter(fn ($master) => (string) ($master->status ?? '') === 'paused')->count();

        return [
            'available' => true,
            'ok' => $masters->isNotEmpty() ? $paused === 0 : false,
            'master_count' => $masters->count(),
            'paused_masters' => $paused,
        ];
    }

    private function buildTrafficTrend(CarbonImmutable $now): array
    {
        $start = $now->subDays(6)->toDateString();
        $end = $now->toDateString();

        $rows = NodeTrafficRecord::query()
            ->select(
                'record_date',
                DB::raw('COALESCE(SUM(upload_traffic), 0) AS upload_kb'),
                DB::raw('COALESCE(SUM(download_traffic), 0) AS download_kb'),
                DB::raw('COUNT(DISTINCT user_id) AS unique_users')
            )
            ->whereBetween('record_date', [$start, $end])
            ->groupBy('record_date')
            ->orderBy('record_date')
            ->get()
            ->keyBy(fn (NodeTrafficRecord $row) => (string) $row->record_date->toDateString());

        $trend = [];
        for ($i = 6; $i >= 0; $i--) {
            $date = $now->subDays($i)->toDateString();
            /** @var NodeTrafficRecord|null $row */
            $row = $rows->get($date);
            $upload = (int) ($row?->upload_kb ?? 0);
            $download = (int) ($row?->download_kb ?? 0);
            $trend[] = [
                'date' => $date,
                'upload_kb' => $upload,
                'download_kb' => $download,
                'total_kb' => $upload + $download,
                'unique_users' => (int) ($row?->unique_users ?? 0),
            ];
        }

        return $trend;
    }

    private function buildProtocolDistribution(Collection $nodeSnapshots): array
    {
        return $nodeSnapshots
            ->groupBy('protocol')
            ->map(function (Collection $rows, string $protocol) {
                return [
                    'protocol' => $protocol,
                    'label' => $this->protocolLabel($protocol),
                    'total' => $rows->count(),
                    'online' => $rows->where('online_status', 'online')->count(),
                    'tcping_enabled' => $rows->where('tcping_enabled', true)->count(),
                ];
            })
            ->sortByDesc('total')
            ->values()
            ->all();
    }

    private function buildRegionDistribution(Collection $nodeSnapshots): array
    {
        return $nodeSnapshots
            ->groupBy(fn (array $node) => $node['location_code'] ?: $node['location_name'])
            ->map(function (Collection $rows) {
                $first = $rows->first();
                return [
                    'location_code' => (string) ($first['location_code'] ?: ''),
                    'location_name' => (string) ($first['location_name'] ?: '未标注地区'),
                    'total' => $rows->count(),
                    'online' => $rows->where('online_status', 'online')->count(),
                ];
            })
            ->sortByDesc('total')
            ->values()
            ->take(10)
            ->all();
    }

    private function buildTopUsers(CarbonImmutable $now, string $startDate, string $endDate): array
    {
        $trafficRows = NodeTrafficRecord::query()
            ->select('user_id', DB::raw('COALESCE(SUM(upload_traffic + download_traffic), 0) AS traffic_kb'))
            ->whereBetween('record_date', [$startDate, $endDate])
            ->groupBy('user_id')
            ->orderByDesc('traffic_kb')
            ->limit(8)
            ->get();

        $userIds = $trafficRows->pluck('user_id')->map(fn ($id) => (int) $id)->all();
        $users = User::query()
            ->whereIn('id', $userIds)
            ->get(['id', 'email', 'linux_do_name', 'linux_do_username', 'trust_level', 'last_login_at'])
            ->keyBy('id');

        $activeSubscriptions = UserPlanSubscription::query()
            ->active($now->timestamp)
            ->with('plan:id,name')
            ->whereIn('user_id', $userIds)
            ->orderByDesc('expired_at')
            ->get()
            ->unique('user_id')
            ->keyBy('user_id');

        return $trafficRows->map(function ($row) use ($users, $activeSubscriptions) {
            /** @var User|null $user */
            $user = $users->get((int) $row->user_id);
            /** @var UserPlanSubscription|null $subscription */
            $subscription = $activeSubscriptions->get((int) $row->user_id);

            return [
                'id' => (int) ($user?->id ?? $row->user_id),
                'email' => (string) ($user?->email ?? ('user-' . $row->user_id)),
                'display_name' => (string) ($user?->linux_do_name ?: $user?->linux_do_username ?: $user?->email ?: ('User ' . $row->user_id)),
                'traffic_kb' => (int) ($row->traffic_kb ?? 0),
                'trust_level' => (int) ($user?->trust_level ?? 0),
                'last_login_at' => $user?->last_login_at ? (int) $user->last_login_at : null,
                'plan_name' => (string) ($subscription?->plan?->name ?? '无有效套餐'),
                'subscription_expired_at' => $subscription?->expired_at ? (int) $subscription->expired_at : null,
            ];
        })->values()->all();
    }

    private function decorateWatchlistWithSamples(Collection $nodeSnapshots, CarbonImmutable $now): array
    {
        $watchlist = $nodeSnapshots
            ->sortByDesc('watch_score')
            ->take(8)
            ->values();

        $watchIds = $watchlist->pluck('id')->map(fn ($id) => (int) $id)->all();
        if (!$watchIds) {
            return [];
        }

        $samplesByNode = TcpingSample::query()
            ->whereIn('node_id', $watchIds)
            ->where('sampled_at', '>=', $now->subHours(24)->timestamp)
            ->orderByDesc('sampled_at')
            ->get(['node_id', 'sampled_at', 'latency_ms', 'is_reachable'])
            ->groupBy('node_id')
            ->map(function (Collection $rows) {
                $trimmed = $rows
                    ->sortBy('sampled_at')
                    ->values();

                if ($trimmed->count() > 18) {
                    $trimmed = $trimmed->slice($trimmed->count() - 18)->values();
                }

                return $trimmed->map(fn (TcpingSample $sample) => [
                    'sampled_at' => (int) ($sample->sampled_at ?? 0),
                    'latency_ms' => $sample->latency_ms === null ? null : (int) $sample->latency_ms,
                    'is_reachable' => (bool) $sample->is_reachable,
                ])->all();
            });

        return $watchlist->map(function (array $node) use ($samplesByNode) {
            $node['tcping_samples'] = $samplesByNode->get($node['id'], []);
            return $node;
        })->all();
    }

    private function buildOpenTickets(): array
    {
        return Ticket::query()
            ->with([
                'user:id,email,linux_do_name,linux_do_username',
                'node:id,name',
                'assignedAdmin:id,email',
            ])
            ->where('status', Ticket::STATUS_OPENING)
            ->orderByDesc('updated_at')
            ->limit(6)
            ->get()
            ->map(function (Ticket $ticket) {
                return [
                    'id' => (int) $ticket->id,
                    'subject' => (string) ($ticket->subject ?? '-'),
                    'level' => (string) ($ticket->level ?? '-'),
                    'status' => (int) ($ticket->status ?? Ticket::STATUS_OPENING),
                    'user_email' => (string) ($ticket->user?->email ?? '-'),
                    'user_name' => (string) ($ticket->user?->linux_do_name ?: $ticket->user?->linux_do_username ?: $ticket->user?->email ?: '-'),
                    'node_name' => (string) ($ticket->node?->name ?? '-'),
                    'assigned_admin_email' => (string) ($ticket->assignedAdmin?->email ?? '-'),
                    'updated_at' => $this->toUnixTimestamp($ticket->updated_at),
                    'created_at' => $this->toUnixTimestamp($ticket->created_at),
                ];
            })
            ->values()
            ->all();
    }

    private function buildPendingRefunds(): array
    {
        return OrderRefundRequest::query()
            ->with([
                'user:id,email,linux_do_name,linux_do_username',
                'plan:id,name',
                'assignedAdmin:id,email',
            ])
            ->whereIn('status', [OrderRefundRequest::STATUS_PENDING, OrderRefundRequest::STATUS_VOTING])
            ->orderByDesc('updated_at')
            ->limit(6)
            ->get()
            ->map(function (OrderRefundRequest $refund) {
                return [
                    'id' => (int) $refund->id,
                    'trade_no' => (string) ($refund->trade_no ?? '-'),
                    'status' => (string) ($refund->status ?? OrderRefundRequest::STATUS_PENDING),
                    'user_email' => (string) ($refund->user?->email ?? '-'),
                    'user_name' => (string) ($refund->user?->linux_do_name ?: $refund->user?->linux_do_username ?: $refund->user?->email ?: '-'),
                    'plan_name' => (string) ($refund->plan?->name ?? '-'),
                    'assigned_admin_email' => (string) ($refund->assignedAdmin?->email ?? '-'),
                    'gateway_amount' => (int) ($refund->gateway_amount ?? 0),
                    'refund_amount' => (int) ($refund->refund_amount ?? 0),
                    'updated_at' => $this->toUnixTimestamp($refund->updated_at),
                    'created_at' => $this->toUnixTimestamp($refund->created_at),
                ];
            })
            ->values()
            ->all();
    }

    private function buildTcpingAgents(CarbonImmutable $now): array
    {
        return TcpingAgent::query()
            ->with(['user:id,email,linux_do_name,linux_do_username'])
            ->orderByRaw('COALESCE(last_heartbeat_at, 0) DESC')
            ->limit(6)
            ->get()
            ->map(function (TcpingAgent $agent) use ($now) {
                $isOnline = $agent->last_heartbeat_at !== null
                    && (int) $agent->last_heartbeat_at >= $now->subMinutes(3)->timestamp;

                return [
                    'id' => (int) $agent->id,
                    'name' => (string) ($agent->name ?? '-'),
                    // TCPing 监控由系统统一调度，用户侧不再允许停用。
                    'is_enabled' => true,
                    'is_online' => $isOnline,
                    'owner_email' => (string) ($agent->user?->email ?? '-'),
                    'owner_name' => (string) ($agent->user?->linux_do_name ?: $agent->user?->linux_do_username ?: $agent->user?->email ?: '-'),
                    'last_heartbeat_at' => $agent->last_heartbeat_at === null ? null : (int) $agent->last_heartbeat_at,
                    'last_sync_at' => $agent->last_sync_at === null ? null : (int) $agent->last_sync_at,
                ];
            })
            ->values()
            ->all();
    }

    private function buildActiveTcpingAlerts(CarbonImmutable $now): array
    {
        return TcpingAlert::query()
            ->with([
                'user:id,email,linux_do_name,linux_do_username',
                'node:id,name,protocol,location_code,location_name',
            ])
            ->where('status', TcpingAlert::STATUS_ACTIVE)
            ->orderByDesc('triggered_at')
            ->limit(8)
            ->get()
            ->map(function (TcpingAlert $alert) use ($now) {
                return [
                    'id' => (int) $alert->id,
                    'status' => (string) ($alert->status ?? TcpingAlert::STATUS_ACTIVE),
                    'user_email' => (string) ($alert->user?->email ?? '-'),
                    'user_name' => (string) ($alert->user?->linux_do_name ?: $alert->user?->linux_do_username ?: $alert->user?->email ?: '-'),
                    'node_name' => (string) ($alert->node?->name ?? '-'),
                    'node_protocol' => (string) ($alert->node?->protocol ?? '-'),
                    'node_location_code' => (string) ($alert->node?->location_code ?? ''),
                    'node_location_name' => (string) ($alert->node?->location_name ?? '-'),
                    'triggered_at' => (int) ($alert->triggered_at ?? 0),
                    'duration_seconds' => max(0, $now->timestamp - (int) ($alert->triggered_at ?? $now->timestamp)),
                    'latest_error' => (string) ($alert->latest_error ?? ''),
                ];
            })
            ->values()
            ->all();
    }

    private function buildAuditStream(): array
    {
        return AuditLog::query()
            ->with([
                'user:id,email,linux_do_name,linux_do_username',
                'node:id,name,protocol,location_code,location_name',
            ])
            ->orderByDesc('created_at')
            ->limit(12)
            ->get()
            ->map(function (AuditLog $log) {
                return [
                    'id' => (int) $log->id,
                    'action_taken' => (string) ($log->action_taken ?? AuditLog::ACTION_LOGGED),
                    'user_email' => (string) ($log->user?->email ?? '-'),
                    'user_name' => (string) ($log->user?->linux_do_name ?: $log->user?->linux_do_username ?: $log->user?->email ?: '-'),
                    'node_name' => (string) ($log->node?->name ?? '-'),
                    'node_protocol' => (string) ($log->node?->protocol ?? '-'),
                    'node_location_code' => (string) ($log->node?->location_code ?? ''),
                    'node_location_name' => (string) ($log->node?->location_name ?? '-'),
                    'ip_address' => (string) ($log->ip_address ?? '-'),
                    'target_domain' => $log->target_domain ? (string) $log->target_domain : null,
                    'target_protocol' => $log->target_protocol ? (string) $log->target_protocol : null,
                    'created_at' => $this->toUnixTimestamp($log->created_at),
                ];
            })
            ->values()
            ->all();
    }

    private function toUnixTimestamp(mixed $value): ?int
    {
        if ($value === null || $value === '') {
            return null;
        }

        if ($value instanceof \DateTimeInterface) {
            return $value->getTimestamp();
        }

        if (is_numeric($value)) {
            return (int) $value;
        }

        $parsed = strtotime((string) $value);

        return $parsed === false ? null : $parsed;
    }

    private function resolveThroughputSnapshot(CarbonImmutable $now): array
    {
        $totals = User::query()
            ->selectRaw('COALESCE(SUM(u), 0) AS total_upload, COALESCE(SUM(d), 0) AS total_download')
            ->first();

        $currentUpload = (int) ($totals->total_upload ?? 0);
        $currentDownload = (int) ($totals->total_download ?? 0);
        $previous = Cache::get(self::TRAFFIC_SNAPSHOT_KEY);

        Cache::put(self::TRAFFIC_SNAPSHOT_KEY, [
            'ts' => $now->timestamp,
            'u' => $currentUpload,
            'd' => $currentDownload,
        ], now()->addMinutes(2));

        $uploadBps = 0;
        $downloadBps = 0;
        if (is_array($previous) && isset($previous['ts'], $previous['u'], $previous['d'])) {
            $delta = $now->timestamp - (int) $previous['ts'];
            if ($delta >= 10 && $delta <= 180) {
                $uploadBps = max(0, (int) (($currentUpload - (int) $previous['u']) / $delta));
                $downloadBps = max(0, (int) (($currentDownload - (int) $previous['d']) / $delta));
            }
        }

        return [
            'upload_bps' => $uploadBps,
            'download_bps' => $downloadBps,
        ];
    }

    private function protocolLabel(?string $protocol): string
    {
        return match (strtolower((string) $protocol)) {
            'vmess' => 'VMess',
            'vless' => 'VLESS',
            'trojan' => 'Trojan',
            'shadowsocks' => 'Shadowsocks',
            'hysteria' => 'Hysteria',
            'hysteria2' => 'Hysteria2',
            'tuic' => 'TUIC',
            'anytls' => 'AnyTLS',
            'socks' => 'SOCKS',
            'http' => 'HTTP',
            'naive' => 'Naive',
            'mieru' => 'Mieru',
            default => strtoupper((string) $protocol),
        };
    }
}
