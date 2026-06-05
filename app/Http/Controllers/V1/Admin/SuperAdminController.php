<?php

namespace App\Http\Controllers\V1\Admin;

use App\Http\Controllers\Controller;
use App\Models\User;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\Request;

class SuperAdminController extends Controller
{
    /**
     * Get a user's cross-node concurrent IP limit.
     * 0 means using the default policy.
     */
    public function getConcurrentIpLimit(Request $request, int $userId): JsonResponse
    {
        $user = User::query()->find($userId);
        if (!$user) {
            return response()->json(['success' => false, 'error' => 'User not found'], 404);
        }

        return response()->json([
            'success' => true,
            'data' => [
                'user_id' => $user->id,
                'concurrent_ip_limit' => (int) $user->concurrent_ip_limit,
            ],
        ]);
    }

    /**
     * Set a user's cross-node concurrent IP limit.
     * 0 means using the default policy.
     */
    public function setConcurrentIpLimit(Request $request, int $userId): JsonResponse
    {
        $data = $request->validate([
            'concurrent_ip_limit' => 'required|integer|min:0|max:1000',
        ]);

        $user = User::query()->find($userId);
        if (!$user) {
            return response()->json(['success' => false, 'error' => 'User not found'], 404);
        }

        $user->update([
            'concurrent_ip_limit' => (int) $data['concurrent_ip_limit'],
        ]);

        return response()->json([
            'success' => true,
            'message' => 'Concurrent IP limit updated successfully',
            'data' => [
                'user_id' => $user->id,
                'concurrent_ip_limit' => (int) $user->concurrent_ip_limit,
            ],
        ]);
    }
}
