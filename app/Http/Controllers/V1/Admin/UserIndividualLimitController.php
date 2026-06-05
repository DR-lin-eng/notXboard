<?php

namespace App\Http\Controllers\V1\Admin;

use App\Http\Controllers\Controller;
use App\Services\LimitControlService;
use App\Models\User;
use App\Models\UserIndividualLimit;
use Illuminate\Http\Request;
use Illuminate\Http\JsonResponse;
use Illuminate\Support\Facades\Log;

class UserIndividualLimitController extends Controller
{
    protected LimitControlService $limitService;
    
    public function __construct(LimitControlService $limitService)
    {
        $this->limitService = $limitService;
        $this->middleware('auth:sanctum');
    }
    
    /**
     * 获取用户的个人限制配置
     */
    public function show(Request $request, int $userId): JsonResponse
    {
        try {
            // 权限检查：超级管理员可以查看所有用户，普通用户只能查看自己
            if (!$request->user()->is_super_admin && $request->user()->id !== $userId) {
                return response()->json([
                    'success' => false,
                    'error' => 'Permission denied'
                ], 403);
            }
            
            $user = User::find($userId);
            if (!$user) {
                return response()->json([
                    'success' => false,
                    'error' => 'User not found'
                ], 404);
            }
            
            $individualLimit = $user->individualLimit;
            $effectiveLimits = $this->limitService->getEffectiveLimits($user);
            
            return response()->json([
                'success' => true,
                'data' => [
                    'user_id' => $user->id,
                    'user_email' => $user->email,
                    'trust_level' => $user->trust_level,
                    'individual_limits' => $individualLimit ? [
                        'speed_limit_up' => $individualLimit->speed_limit_up,
                        'speed_limit_down' => $individualLimit->speed_limit_down,
                        'device_limit' => $individualLimit->device_limit,
                        'connection_limit' => $individualLimit->connection_limit,
                        'created_at' => $individualLimit->created_at,
                        'updated_at' => $individualLimit->updated_at,
                    ] : null,
                    'effective_limits' => $effectiveLimits,
                    'has_individual_limits' => $individualLimit !== null,
                ]
            ]);
            
        } catch (\Exception $e) {
            Log::error('Failed to get user individual limits', [
                'user_id' => $userId,
                'error' => $e->getMessage()
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'Failed to retrieve user limits'
            ], 500);
        }
    }
    
    /**
     * 设置用户的个人限制配置
     */
    public function store(Request $request, int $userId): JsonResponse
    {
        try {
            // 权限检查：超级管理员可以设置所有用户，普通用户只能设置自己
            if (!$request->user()->is_super_admin && $request->user()->id !== $userId) {
                return response()->json([
                    'success' => false,
                    'error' => 'Permission denied'
                ], 403);
            }
            
            $validated = $request->validate([
                'speed_limit_up' => 'nullable|integer|min:0',
                'speed_limit_down' => 'nullable|integer|min:0',
                'device_limit' => 'nullable|integer|min:0',
                'connection_limit' => 'nullable|integer|min:0',
            ]);
            
            $user = User::find($userId);
            if (!$user) {
                return response()->json([
                    'success' => false,
                    'error' => 'User not found'
                ], 404);
            }
            
            // 验证限制配置
            $errors = $this->limitService->validateLimits($validated);
            if (!empty($errors)) {
                return response()->json([
                    'success' => false,
                    'error' => 'Invalid limit configuration',
                    'details' => $errors
                ], 400);
            }
            
            $individualLimit = $this->limitService->setUserLimits($user, $validated);
            
            Log::info('User individual limits updated', [
                'user_id' => $userId,
                'admin_id' => $request->user()->id,
                'limits' => $validated
            ]);
            
            return response()->json([
                'success' => true,
                'message' => 'User limits updated successfully',
                'data' => [
                    'user_id' => $user->id,
                    'individual_limits' => [
                        'speed_limit_up' => $individualLimit->speed_limit_up,
                        'speed_limit_down' => $individualLimit->speed_limit_down,
                        'device_limit' => $individualLimit->device_limit,
                        'connection_limit' => $individualLimit->connection_limit,
                        'updated_at' => $individualLimit->updated_at,
                    ],
                    'effective_limits' => $this->limitService->getEffectiveLimits($user),
                ]
            ]);
            
        } catch (\Illuminate\Validation\ValidationException $e) {
            return response()->json([
                'success' => false,
                'error' => 'Validation failed',
                'details' => $e->errors()
            ], 422);
            
        } catch (\Exception $e) {
            Log::error('Failed to update user individual limits', [
                'user_id' => $userId,
                'error' => $e->getMessage(),
                'admin_id' => $request->user()->id
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'Failed to update user limits'
            ], 500);
        }
    }
    
    /**
     * 删除用户的个人限制配置（恢复使用组限制）
     */
    public function destroy(Request $request, int $userId): JsonResponse
    {
        try {
            // 权限检查：超级管理员可以删除所有用户限制，普通用户只能删除自己的
            if (!$request->user()->is_super_admin && $request->user()->id !== $userId) {
                return response()->json([
                    'success' => false,
                    'error' => 'Permission denied'
                ], 403);
            }
            
            $user = User::find($userId);
            if (!$user) {
                return response()->json([
                    'success' => false,
                    'error' => 'User not found'
                ], 404);
            }
            
            $deleted = $this->limitService->removeUserLimits($user);
            
            if (!$deleted) {
                return response()->json([
                    'success' => false,
                    'error' => 'No individual limits found for this user'
                ], 404);
            }
            
            // 应用组限制到用户
            $this->limitService->applyLimitsToUser($user);
            
            Log::info('User individual limits removed', [
                'user_id' => $userId,
                'admin_id' => $request->user()->id
            ]);
            
            return response()->json([
                'success' => true,
                'message' => 'Individual limits removed successfully. User will now use group limits.',
                'data' => [
                    'user_id' => $user->id,
                    'effective_limits' => $this->limitService->getEffectiveLimits($user),
                ]
            ]);
            
        } catch (\Exception $e) {
            Log::error('Failed to remove user individual limits', [
                'user_id' => $userId,
                'error' => $e->getMessage()
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'Failed to remove user limits'
            ], 500);
        }
    }
    
    /**
     * 获取用户列表（带限制信息）
     */
    public function index(Request $request): JsonResponse
    {
        try {
            // 仅超级管理员可以查看所有用户
            if (!$request->user()->is_super_admin) {
                return response()->json([
                    'success' => false,
                    'error' => 'Permission denied'
                ], 403);
            }
            
            $perPage = $request->input('per_page', 20);
            $search = $request->input('search');
            
            $query = User::with('individualLimit')
                ->select(['id', 'email', 'linux_do_username', 'trust_level', 'is_super_admin', 'created_at']);
            
            if ($search) {
                $query->where(function ($q) use ($search) {
                    $q->where('email', 'like', "%{$search}%")
                      ->orWhere('linux_do_username', 'like', "%{$search}%");
                });
            }
            
            $users = $query->paginate($perPage);
            
            $users->getCollection()->transform(function ($user) {
                $effectiveLimits = $this->limitService->getEffectiveLimits($user);
                
                return [
                    'id' => $user->id,
                    'email' => $user->email,
                    'linux_do_username' => $user->linux_do_username,
                    'trust_level' => $user->trust_level,
                    'is_super_admin' => $user->is_super_admin,
                    'has_individual_limits' => $user->individualLimit !== null,
                    'effective_limits' => $effectiveLimits,
                    'created_at' => $user->created_at,
                ];
            });
            
            return response()->json([
                'success' => true,
                'data' => $users
            ]);
            
        } catch (\Exception $e) {
            Log::error('Failed to get users with limits', [
                'error' => $e->getMessage()
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'Failed to retrieve users'
            ], 500);
        }
    }
    
    /**
     * 批量设置用户限制
     */
    public function batchUpdate(Request $request): JsonResponse
    {
        try {
            // 仅超级管理员可以批量设置
            if (!$request->user()->is_super_admin) {
                return response()->json([
                    'success' => false,
                    'error' => 'Permission denied'
                ], 403);
            }
            
            $validated = $request->validate([
                'users' => 'required|array',
                'users.*.user_id' => 'required|integer|exists:v2_user,id',
                'users.*.speed_limit_up' => 'nullable|integer|min:0',
                'users.*.speed_limit_down' => 'nullable|integer|min:0',
                'users.*.device_limit' => 'nullable|integer|min:0',
                'users.*.connection_limit' => 'nullable|integer|min:0',
            ]);
            
            $results = [];
            
            foreach ($validated['users'] as $userData) {
                $userId = $userData['user_id'];
                unset($userData['user_id']);
                
                $user = User::find($userId);
                if ($user) {
                    $individualLimit = $this->limitService->setUserLimits($user, $userData);
                    $results[] = [
                        'user_id' => $user->id,
                        'email' => $user->email,
                        'limits_updated' => true,
                    ];
                }
            }
            
            Log::info('Batch user limits updated', [
                'count' => count($results),
                'admin_id' => $request->user()->id
            ]);
            
            return response()->json([
                'success' => true,
                'message' => 'User limits updated successfully',
                'data' => $results
            ]);
            
        } catch (\Illuminate\Validation\ValidationException $e) {
            return response()->json([
                'success' => false,
                'error' => 'Validation failed',
                'details' => $e->errors()
            ], 422);
            
        } catch (\Exception $e) {
            Log::error('Failed to batch update user limits', [
                'error' => $e->getMessage(),
                'admin_id' => $request->user()->id
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'Failed to update user limits'
            ], 500);
        }
    }
}