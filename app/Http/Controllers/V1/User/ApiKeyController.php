<?php

namespace App\Http\Controllers\V1\User;

use App\Http\Controllers\Controller;
use App\Services\ApiKeyService;
use Illuminate\Http\Request;
use Illuminate\Http\JsonResponse;
use Illuminate\Support\Facades\Log;

class ApiKeyController extends Controller
{
    protected ApiKeyService $apiKeyService;
    
    public function __construct(ApiKeyService $apiKeyService)
    {
        $this->apiKeyService = $apiKeyService;
        $this->middleware('auth:sanctum');
    }
    
    /**
     * 获取当前用户的 API 密钥信息
     */
    public function show(Request $request): JsonResponse
    {
        try {
            $user = $request->user();
            $stats = $this->apiKeyService->getApiKeyStats($user);
            
            return response()->json([
                'success' => true,
                'data' => [
                    'api_key' => $user->api_key,
                    'api_key_prefix' => $stats['api_key_prefix'],
                    'has_api_key' => $stats['api_key_exists'],
                    'created_at' => $stats['created_at'],
                    'last_login_at' => $stats['last_login_at'],
                    'is_active' => $stats['is_active'],
                ]
            ]);
            
        } catch (\Exception $e) {
            Log::error('Failed to get API key info', [
                'user_id' => $request->user()->id,
                'error' => $e->getMessage()
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'Failed to retrieve API key information'
            ], 500);
        }
    }
    
    /**
     * 生成新的 API 密钥
     */
    public function generate(Request $request): JsonResponse
    {
        try {
            $user = $request->user();
            
            // 如果用户已有 API 密钥，需要确认重置
            if ($user->api_key && !$request->input('force', false)) {
                return response()->json([
                    'success' => false,
                    'error' => 'API key already exists. Use reset endpoint to generate a new one.',
                    'has_existing_key' => true
                ], 400);
            }
            
            $apiKey = $this->apiKeyService->generateApiKey($user);
            
            return response()->json([
                'success' => true,
                'message' => 'API key generated successfully',
                'data' => [
                    'api_key' => $apiKey,
                    'api_key_prefix' => substr($apiKey, 0, 10) . '...',
                    'generated_at' => now()->toISOString(),
                ]
            ]);
            
        } catch (\Exception $e) {
            Log::error('Failed to generate API key', [
                'user_id' => $request->user()->id,
                'error' => $e->getMessage()
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'Failed to generate API key'
            ], 500);
        }
    }
    
    /**
     * 重置 API 密钥
     */
    public function reset(Request $request): JsonResponse
    {
        try {
            $user = $request->user();
            
            if (!$user->api_key) {
                $newApiKey = $this->apiKeyService->generateApiKey($user);

                return response()->json([
                    'success' => true,
                    'message' => 'No existing API key found. Generated a new API key.',
                    'data' => [
                        'api_key' => $newApiKey,
                        'api_key_prefix' => substr($newApiKey, 0, 10) . '...',
                        'action' => 'generated',
                        'reset_at' => now()->toISOString(),
                    ]
                ]);
            }
            
            $oldKeyPrefix = substr($user->api_key, 0, 10) . '...';
            $newApiKey = $this->apiKeyService->resetApiKey($user);
            
            return response()->json([
                'success' => true,
                'message' => 'API key reset successfully',
                'data' => [
                    'api_key' => $newApiKey,
                    'api_key_prefix' => substr($newApiKey, 0, 10) . '...',
                    'old_key_prefix' => $oldKeyPrefix,
                    'action' => 'reset',
                    'reset_at' => now()->toISOString(),
                ]
            ]);
            
        } catch (\Exception $e) {
            Log::error('Failed to reset API key', [
                'user_id' => $request->user()->id,
                'error' => $e->getMessage()
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'Failed to reset API key'
            ], 500);
        }
    }
    
    /**
     * 验证 API 密钥
     */
    public function validateApiKey(Request $request): JsonResponse
    {
        try {
            $request->validate([
                'api_key' => 'required|string'
            ]);
            
            $apiKey = $request->input('api_key');
            $isValid = $this->apiKeyService->validateApiKeyForUser($request->user(), $apiKey);
            
            if ($isValid) {
                return response()->json([
                    'success' => true,
                    'valid' => true,
                    'data' => [
                        'belongs_to_current_user' => true,
                        'is_active' => (bool) $request->user()->isActive(),
                    ]
                ]);
            } else {
                return response()->json([
                    'success' => true,
                    'valid' => false,
                    'message' => 'Invalid API key'
                ]);
            }
            
        } catch (\Illuminate\Validation\ValidationException $e) {
            return response()->json([
                'success' => false,
                'error' => 'Validation failed',
                'details' => $e->errors()
            ], 422);
            
        } catch (\Exception $e) {
            Log::error('Failed to validate API key', [
                'error' => $e->getMessage()
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'Failed to validate API key'
            ], 500);
        }
    }
}
