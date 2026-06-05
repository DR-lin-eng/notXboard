<?php

namespace App\Http\Controllers\V1\User;

use App\Http\Controllers\Controller;
use App\Models\ServerNode;
use App\Models\User;
use App\Services\AccessControlService;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;

class AccessControlController extends Controller
{
    public function __construct(
        private readonly AccessControlService $accessControlService
    ) {
    }

    public function myStats(Request $request): JsonResponse
    {
        $user = Auth::user();
        return response()->json([
            'success' => true,
            'data' => $this->accessControlService->getUserAccessStats($user),
        ]);
    }

    public function nodeStats(Request $request, int $nodeId): JsonResponse
    {
        $user = Auth::user();
        $node = ServerNode::query()->whereKey($nodeId)->where('user_id', $user->id)->first();
        if (!$node) {
            return response()->json(['success' => false, 'error' => 'Server node not found'], 404);
        }

        return response()->json([
            'success' => true,
            'data' => $this->accessControlService->getNodeAccessStats($node),
        ]);
    }

    public function shareWithUser(Request $request, int $nodeId): JsonResponse
    {
        $user = Auth::user();
        $node = ServerNode::query()->whereKey($nodeId)->where('user_id', $user->id)->first();
        if (!$node) {
            return response()->json(['success' => false, 'error' => 'Server node not found'], 404);
        }

        $data = $request->validate([
            'user_id' => 'required|integer|min:1|exists:v2_user,id',
        ]);

        $targetUser = User::query()->find((int) $data['user_id']);
        $this->accessControlService->shareNodeWithUser($node, $targetUser);

        return response()->json(['success' => true]);
    }

    public function shareWithGroup(Request $request, int $nodeId): JsonResponse
    {
        $user = Auth::user();
        $node = ServerNode::query()->whereKey($nodeId)->where('user_id', $user->id)->first();
        if (!$node) {
            return response()->json(['success' => false, 'error' => 'Server node not found'], 404);
        }

        $data = $request->validate([
            'min_trust_level' => 'required|integer|min:0|max:4',
        ]);

        $this->accessControlService->shareNodeWithGroup($node, (int) $data['min_trust_level']);

        return response()->json(['success' => true]);
    }

    public function revoke(Request $request, int $nodeId): JsonResponse
    {
        $user = Auth::user();
        $node = ServerNode::query()->whereKey($nodeId)->where('user_id', $user->id)->first();
        if (!$node) {
            return response()->json(['success' => false, 'error' => 'Server node not found'], 404);
        }

        $data = $request->validate([
            'user_id' => 'required|integer|min:1|exists:v2_user,id',
        ]);

        $targetUser = User::query()->find((int) $data['user_id']);
        $this->accessControlService->revokeNodeAccess($node, $targetUser);

        return response()->json(['success' => true]);
    }

    public function accessibleNodes(Request $request): JsonResponse
    {
        $user = Auth::user();
        $nodes = $this->accessControlService->getAccessibleNodesForUser($user);

        $data = $nodes->map(function (ServerNode $node) {
            $tcpingMonitorable = $node->isTcpingMonitorable();
            return [
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
                'traffic_usage_percentage' => $node->getTrafficUsagePercentage(),
                'traffic_multiplier' => $node->getEffectiveTrafficMultiplier(),
                'tcping_enabled' => $tcpingMonitorable,
                'tcping_status' => $tcpingMonitorable ? ($node->tcping_last_status ?: 'unknown') : 'unsupported',
                'tcping_last_latency_ms' => $tcpingMonitorable ? $node->tcping_last_latency_ms : null,
                'tcping_last_sampled_at' => $tcpingMonitorable ? $node->tcping_last_sampled_at : null,
                'user_id' => $node->user_id,
                'owner_email' => $node->owner?->email,
            ];
        })->values();

        return response()->json([
            'success' => true,
            'data' => $data,
        ]);
    }
}
