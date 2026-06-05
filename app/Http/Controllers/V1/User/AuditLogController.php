<?php

namespace App\Http\Controllers\V1\User;

use App\Http\Controllers\Controller;
use App\Models\AuditLog;
use App\Models\ServerNode;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;

class AuditLogController extends Controller
{
    /**
     * Current user's audit logs.
     */
    public function myLogs(Request $request): JsonResponse
    {
        $user = Auth::user();
        $limit = min(500, max(1, (int) $request->query('limit', 100)));

        $query = AuditLog::query()
            ->where('user_id', $user->id)
            ->with(['node', 'rule'])
            ->orderByDesc('id');

        if ($request->filled('node_id')) {
            $query->where('node_id', (int) $request->query('node_id'));
        }

        return response()->json([
            'success' => true,
            'data' => $query->limit($limit)->get(),
        ]);
    }

    /**
     * Node owner's audit logs for a node.
     */
    public function nodeLogs(Request $request, int $nodeId): JsonResponse
    {
        $user = Auth::user();
        $limit = min(500, max(1, (int) $request->query('limit', 100)));

        $node = ServerNode::query()
            ->whereKey($nodeId)
            ->where('user_id', $user->id)
            ->first();

        if (!$node) {
            return response()->json(['success' => false, 'error' => 'Server node not found'], 404);
        }

        $logs = AuditLog::query()
            ->where('node_id', $node->id)
            ->with(['user', 'rule'])
            ->orderByDesc('id')
            ->limit($limit)
            ->get();

        return response()->json(['success' => true, 'data' => $logs]);
    }
}

