<?php

namespace App\Http\Controllers\V1\Admin;

use App\Http\Controllers\Controller;
use App\Services\ApiKeyService;
use App\Models\User;
use Illuminate\Http\Request;
use Illuminate\Http\JsonResponse;
use Illuminate\Support\Facades\Log;

class ApiKeyManagementController extends Controller
{
    protected ApiKeyService $apiKeyService;
    
    public function __construct(ApiKeyService $apiKeyService)
    {
        $this->apiKeyService = $apiKeyService;
        $this->middleware('auth:sanctum');
        $this->middleware('admin.super');
    }
    
    /**
     * 获取系统 API 密钥统计信息
     */
    public function stats(): JsonResponse
    {
        try {
            $stats = $this->apiKeyService->getSystemStats();
            
            return response()->json([
                'success' => true,
                'data' => $stats
            ]);
            
        } catch (\Exception $e) {
            Log::error('Failed to get API key stats', [
                'error' => $e->getMessage()
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'Failed to retrieve API key statistics'
            ], 500);
        }
    }
    
    /**
     * 获取用户的 API 密钥信息
     */
    public function getUserApiKey(int $userId): JsonResponse
    {
        try {
            $user = User::find($userId);
            
            if (!$user) {
                return response()->json([
                    'success' => false,
                    'error' => 'User not found'
                ], 404);
            }
            
            $stats = $this->apiKeyService->getApiKeyStats($user);
            
            return response()->json([
                'success' => true,
                'data' => [
                    'user_id' => $user->id,
                    'email' => $user->email,
                    'linux_do_username' => $user->linux_do_username,
                    'api_key' => $user->api_key, // 超级管理员可以看到完整密钥
                    'api_key_prefix' => $stats['api_key_prefix'],
                    'has_api_key' => $stats['api_key_exists'],
                    'created_at' => $stats['created_at'],
                    'last_login_at' => $stats['last_login_at'],
                    'is_active' => $stats['is_active'],
                ]
            ]);
            
        } catch (\Exception $e) {
            Log::error('Failed to get user API key', [
                'user_id' => $userId,
                'error' => $e->getMessage()
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'Failed to retrieve user API key'
            ], 500);
        }
    }
    
    /**
     * 为用户生成 API 密钥
     */
    public function generateUserApiKey(Request $request, int $userId): JsonResponse
    {
        try {
            $user = User::find($userId);
            
            if (!$user) {
                return response()->json([
                    'success' => false,
                    'error' => 'User not found'
                ], 404);
            }
            
            $apiKey = $this->apiKeyService->generateApiKey($user);
            
            Log::info('Admin generated API key for user', [
                'admin_id' => $request->user()->id,
                'target_user_id' => $userId,
                'key_prefix' => substr($apiKey, 0, 10) . '...'
            ]);
            
            return response()->json([
                'success' => true,
                'message' => 'API key generated successfully',
                'data' => [
                    'user_id' => $user->id,
                    'email' => $user->email,
                    'api_key' => $apiKey,
                    'api_key_prefix' => substr($apiKey, 0, 10) . '...',
                    'generated_at' => now()->toISOString(),
                ]
            ]);
            
        } catch (\Exception $e) {
            Log::error('Failed to generate user API key', [
                'admin_id' => $request->user()->id,
                'user_id' => $userId,
                'error' => $e->getMessage()
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'Failed to generate API key'
            ], 500);
        }
    }
    
    /**
     * 重置用户的 API 密钥
     */
    public function resetUserApiKey(Request $request, int $userId): JsonResponse
    {
        try {
            $user = User::find($userId);
            
            if (!$user) {
                return response()->json([
                    'success' => false,
                    'error' => 'User not found'
                ], 404);
            }
            
            $adminId = $request->user()->id;
            $hasExisting = !empty($user->api_key);

            if ($hasExisting) {
                $oldKeyPrefix = substr($user->api_key, 0, 10) . '...';
                $newApiKey = $this->apiKeyService->resetApiKey($user);

                Log::info('Admin reset API key for user', [
                    'admin_id' => $adminId,
                    'target_user_id' => $userId,
                    'old_key_prefix' => $oldKeyPrefix,
                    'new_key_prefix' => substr($newApiKey, 0, 10) . '...'
                ]);

                return response()->json([
                    'success' => true,
                    'message' => 'API key reset successfully',
                    'data' => [
                        'user_id' => $user->id,
                        'email' => $user->email,
                        'api_key' => $newApiKey,
                        'api_key_prefix' => substr($newApiKey, 0, 10) . '...',
                        'old_key_prefix' => $oldKeyPrefix,
                        'action' => 'reset',
                        'reset_at' => now()->toISOString(),
                    ]
                ]);
            }

            $newApiKey = $this->apiKeyService->generateApiKey($user);

            Log::info('Admin generated API key via reset endpoint for user without key', [
                'admin_id' => $adminId,
                'target_user_id' => $userId,
                'new_key_prefix' => substr($newApiKey, 0, 10) . '...'
            ]);

            return response()->json([
                'success' => true,
                'message' => 'User had no API key. Generated a new one.',
                'data' => [
                    'user_id' => $user->id,
                    'email' => $user->email,
                    'api_key' => $newApiKey,
                    'api_key_prefix' => substr($newApiKey, 0, 10) . '...',
                    'action' => 'generated',
                    'reset_at' => now()->toISOString(),
                ]
            ]);
            
        } catch (\Exception $e) {
            Log::error('Failed to reset user API key', [
                'admin_id' => $request->user()->id,
                'user_id' => $userId,
                'error' => $e->getMessage()
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'Failed to reset API key'
            ], 500);
        }
    }
    
    /**
     * 批量生成 API 密钥
     */
    public function batchGenerate(Request $request): JsonResponse
    {
        try {
            $results = $this->apiKeyService->batchGenerateApiKeys();
            
            Log::info('Admin initiated batch API key generation', [
                'admin_id' => $request->user()->id,
                'total_processed' => count($results)
            ]);
            
            return response()->json([
                'success' => true,
                'message' => 'Batch API key generation completed',
                'data' => [
                    'results' => $results,
                    'summary' => [
                        'total_processed' => count($results),
                        'successful' => count(array_filter($results, fn($r) => $r['api_key_generated'])),
                        'failed' => count(array_filter($results, fn($r) => !$r['api_key_generated']))
                    ]
                ]
            ]);
            
        } catch (\Exception $e) {
            Log::error('Failed to batch generate API keys', [
                'admin_id' => $request->user()->id,
                'error' => $e->getMessage()
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'Failed to batch generate API keys'
            ], 500);
        }
    }
    
    /**
     * 清理无效的 API 密钥
     */
    public function cleanup(Request $request): JsonResponse
    {
        try {
            $results = $this->apiKeyService->cleanupInvalidApiKeys();
            
            Log::info('Admin initiated API key cleanup', [
                'admin_id' => $request->user()->id,
                'results' => $results
            ]);
            
            return response()->json([
                'success' => true,
                'message' => 'API key cleanup completed',
                'data' => $results
            ]);
            
        } catch (\Exception $e) {
            Log::error('Failed to cleanup API keys', [
                'admin_id' => $request->user()->id,
                'error' => $e->getMessage()
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'Failed to cleanup API keys'
            ], 500);
        }
    }
    
    /**
     * 搜索用户 API 密钥
     */
    public function search(Request $request): JsonResponse
    {
        try {
            $request->validate([
                'query' => 'required|string|min:3',
                'per_page' => 'nullable|integer|min:1|max:100'
            ]);
            
            $query = $request->input('query');
            $perPage = $request->input('per_page', 20);
            
            $users = User::where(function ($q) use ($query) {
                $q->where('email', 'like', "%{$query}%")
                  ->orWhere('linux_do_username', 'like', "%{$query}%")
                  ->orWhere('api_key', 'like', "%{$query}%");
            })
            ->select(['id', 'email', 'linux_do_username', 'api_key', 'trust_level', 'is_super_admin', 'created_at'])
            ->paginate($perPage);
            
            $users->getCollection()->transform(function ($user) {
                return [
                    'id' => $user->id,
                    'email' => $user->email,
                    'linux_do_username' => $user->linux_do_username,
                    'api_key' => $user->api_key,
                    'api_key_prefix' => $user->api_key ? substr($user->api_key, 0, 10) . '...' : null,
                    'has_api_key' => !empty($user->api_key),
                    'trust_level' => $user->trust_level,
                    'is_super_admin' => $user->is_super_admin,
                    'created_at' => $user->created_at,
                ];
            });
            
            return response()->json([
                'success' => true,
                'data' => $users
            ]);
            
        } catch (\Illuminate\Validation\ValidationException $e) {
            return response()->json([
                'success' => false,
                'error' => 'Validation failed',
                'details' => $e->errors()
            ], 422);
            
        } catch (\Exception $e) {
            Log::error('Failed to search API keys', [
                'error' => $e->getMessage()
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'Failed to search API keys'
            ], 500);
        }
    }
}
