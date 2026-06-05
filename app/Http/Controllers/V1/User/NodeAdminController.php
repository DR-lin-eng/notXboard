<?php

namespace App\Http\Controllers\V1\User;

use App\Http\Controllers\Controller;
use App\Models\NodeTrafficRecord;
use App\Models\ServerNode;
use App\Models\User;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\DB;

class NodeAdminController extends Controller
{
    public function nodeUsersTraffic(Request $request, int $id): JsonResponse
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

        $records = NodeTrafficRecord::query()
            ->where('node_id', $node->id)
            ->whereBetween('record_date', [$start->toDateString(), $end->toDateString()])
            ->selectRaw('user_id, SUM(upload_traffic) as upload, SUM(download_traffic) as download')
            ->groupBy('user_id')
            ->orderByRaw('(SUM(upload_traffic)+SUM(download_traffic)) DESC')
            ->limit(5000)
            ->get();

        $userIds = $records->pluck('user_id')->map(fn ($v) => (int) $v)->all();
        $users = User::query()
            ->whereIn('id', $userIds)
            ->get(['id', 'email', 'linux_do_name', 'trust_level', 'is_silenced', 'banned'])
            ->keyBy('id');

        $blacklistedIds = DB::table('user_node_blacklist')
            ->where('node_id', $node->id)
            ->pluck('user_id')
            ->map(fn ($v) => (int) $v)
            ->all();
        $blacklistedSet = array_fill_keys($blacklistedIds, true);

        $data = $records->map(function ($row) use ($users, $blacklistedSet) {
            $u = $users->get((int) $row->user_id);
            return [
                'user_id' => (int) $row->user_id,
                'email' => $u?->email,
                'name' => $u?->linux_do_name,
                'trust_level' => (int) ($u?->trust_level ?? 0),
                'is_silenced' => (bool) ($u?->is_silenced ?? false),
                'banned' => (bool) ($u?->banned ?? false),
                'upload' => (int) $row->upload,
                'download' => (int) $row->download,
                'total' => (int) $row->upload + (int) $row->download,
                'is_blacklisted' => isset($blacklistedSet[(int) $row->user_id]),
            ];
        })->values();

        return response()->json([
            'success' => true,
            'data' => [
                'node' => [
                    'id' => $node->id,
                    'name' => $node->name,
                ],
                'days' => $days,
                'users' => $data,
            ],
        ]);
    }

    public function blacklistUser(Request $request, int $id): JsonResponse
    {
        $user = Auth::user();
        $node = ServerNode::query()->whereKey($id)->where('user_id', $user->id)->first();
        if (!$node) {
            return response()->json(['success' => false, 'error' => 'Server node not found'], 404);
        }

        $data = $request->validate([
            'user_id' => 'required|integer|min:1|exists:v2_user,id',
            'reason' => 'nullable|string|max:255',
        ]);

        DB::table('user_node_blacklist')->updateOrInsert(
            ['node_id' => $node->id, 'user_id' => (int) $data['user_id']],
            ['reason' => $data['reason'] ?? null, 'created_at' => now()]
        );

        return response()->json(['success' => true]);
    }

    public function unblacklistUser(Request $request, int $id): JsonResponse
    {
        $user = Auth::user();
        $node = ServerNode::query()->whereKey($id)->where('user_id', $user->id)->first();
        if (!$node) {
            return response()->json(['success' => false, 'error' => 'Server node not found'], 404);
        }

        $data = $request->validate([
            'user_id' => 'required|integer|min:1|exists:v2_user,id',
        ]);

        DB::table('user_node_blacklist')
            ->where('node_id', $node->id)
            ->where('user_id', (int) $data['user_id'])
            ->delete();

        return response()->json(['success' => true]);
    }
}

