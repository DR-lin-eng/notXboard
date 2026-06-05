<?php

namespace App\Http\Controllers\V1\User;

use App\Http\Controllers\Controller;
use App\Models\NodeTrafficRecord;
use App\Models\ServerNode;
use App\Models\UserTrafficUsageLog;
use App\Services\AccessControlService;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;

class TrafficStatsController extends Controller
{
    public function __construct(
        private readonly AccessControlService $accessControlService
    ) {
    }

    /**
     * Current user's per-node traffic summary.
     */
    public function myNodeTraffic(Request $request): JsonResponse
    {
        $user = Auth::user();

        $nodes = $this->accessControlService->getAccessibleNodesForUser($user);
        if ($nodes->isEmpty()) {
            return response()->json([
                'success' => true,
                'data' => [],
            ]);
        }

        $today = now()->toDateString();
        $recordMap = NodeTrafficRecord::query()
            ->where('user_id', $user->id)
            ->whereIn('node_id', $nodes->pluck('id')->all())
            ->where('record_date', $today)
            ->get()
            ->keyBy(fn (NodeTrafficRecord $record) => (int) $record->node_id);

        $stats = $nodes->map(function (ServerNode $node) use ($recordMap) {
            $record = $recordMap->get((int) $node->id);
            $tcpingMonitorable = $node->isTcpingMonitorable();

            return [
                'node_id' => $node->id,
                'node_name' => $node->name,
                'protocol' => $node->protocol,
                'location_name' => $node->location_name,
                'status' => $node->status,
                'online_status' => $node->getOnlineStatus(),
                'is_online' => $node->isReportedOnline(),
                'last_report_at' => $node->getLastReportAt(),
                'traffic_limit' => $node->traffic_limit,
                'traffic_used' => $node->traffic_used,
                'traffic_multiplier' => $node->getEffectiveTrafficMultiplier(),
                'node_remaining_traffic' => $node->getRemainingTraffic(),
                'traffic_usage_percentage' => $node->getTrafficUsagePercentage(),
                'tcping_enabled' => $tcpingMonitorable,
                'tcping_status' => $tcpingMonitorable ? ($node->tcping_last_status ?: 'unknown') : 'unsupported',
                'tcping_last_latency_ms' => $tcpingMonitorable ? $node->tcping_last_latency_ms : null,
                'tcping_last_sampled_at' => $tcpingMonitorable ? $node->tcping_last_sampled_at : null,
                'today' => [
                    'upload' => (int) ($record?->upload_traffic ?? 0),
                    'download' => (int) ($record?->download_traffic ?? 0),
                ],
            ];
        })->values();

        return response()->json([
            'success' => true,
            'data' => $stats,
        ]);
    }

    /**
     * Node owner's aggregated traffic statistics for a node.
     */
    public function nodeTrafficStats(Request $request, int $id): JsonResponse
    {
        $user = Auth::user();

        $node = ServerNode::query()
            ->whereKey($id)
            ->where('user_id', $user->id)
            ->first();

        if (!$node) {
            return response()->json(['success' => false, 'error' => 'Server node not found'], 404);
        }

        $days = max(1, (int) $request->query('days', 30));
        $start = now()->subDays($days - 1)->startOfDay();
        $end = now()->endOfDay();

        $stats = NodeTrafficRecord::getNodeTrafficStats($node->id, $start, $end);

        return response()->json([
            'success' => true,
            'data' => [
                'node' => [
                    'id' => $node->id,
                    'name' => $node->name,
                    'protocol' => $node->protocol,
                    'location_name' => $node->location_name,
                    'status' => $node->status,
                    'online_status' => $node->getOnlineStatus(),
                    'is_online' => $node->isReportedOnline(),
                    'last_report_at' => $node->getLastReportAt(),
                    'traffic_limit' => $node->traffic_limit,
                    'traffic_used' => $node->traffic_used,
                    'traffic_multiplier' => $node->getEffectiveTrafficMultiplier(),
                    'tcping_enabled' => $node->isTcpingMonitorable(),
                    'tcping_status' => $node->isTcpingMonitorable() ? ($node->tcping_last_status ?: 'unknown') : 'unsupported',
                    'tcping_last_latency_ms' => $node->isTcpingMonitorable() ? $node->tcping_last_latency_ms : null,
                    'tcping_last_sampled_at' => $node->isTcpingMonitorable() ? $node->tcping_last_sampled_at : null,
                ],
                'stats' => $stats,
            ]
        ]);
    }

    public function usageLogs(Request $request): JsonResponse
    {
        $user = Auth::user();
        $days = max(1, min(30, (int) $request->query('days', 7)));
        $limit = max(1, min(500, (int) $request->query('limit', 200)));
        $startAt = now()->subDays($days)->timestamp;

        $logs = UserTrafficUsageLog::query()
            ->with(['node:id,name,protocol,location_name'])
            ->where('user_id', $user->id)
            ->where('recorded_at', '>=', $startAt)
            ->orderByDesc('recorded_at')
            ->limit($limit)
            ->get();

        return response()->json([
            'success' => true,
            'data' => $logs->map(function (UserTrafficUsageLog $log) {
                return [
                    'id' => (int) $log->id,
                    'node_id' => (int) $log->node_id,
                    'node_name' => $log->node?->name,
                    'protocol' => $log->node?->protocol,
                    'location_name' => $log->node?->location_name,
                    'raw_traffic_kb' => (int) $log->raw_traffic_kb,
                    'billed_traffic_kb' => (int) $log->billed_traffic_kb,
                    'multiplier_snapshot' => (float) $log->multiplier_snapshot,
                    'recorded_at' => (int) $log->recorded_at,
                    'source' => (string) $log->source,
                ];
            })->values()->all(),
        ]);
    }
}
