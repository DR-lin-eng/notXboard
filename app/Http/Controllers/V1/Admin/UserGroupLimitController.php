<?php

namespace App\Http\Controllers\V1\Admin;

use App\Http\Controllers\Controller;
use App\Services\LimitControlService;
use App\Models\UserGroupLimit;
use Illuminate\Http\Request;
use Illuminate\Http\JsonResponse;
use Illuminate\Support\Facades\Log;

class UserGroupLimitController extends Controller
{
    protected LimitControlService $limitService;
    
    public function __construct(LimitControlService $limitService)
    {
        $this->limitService = $limitService;
        $this->middleware('auth:sanctum');
        $this->middleware('admin.super'); // 仅超级管理员可访问
    }
    
    /**
     * 获取所有用户组限制配置
     */
    public function index(): JsonResponse
    {
        try {
            $groupLimits = $this->limitService->getAllGroupLimits();
            
            return response()->json([
                'success' => true,
                'data' => $groupLimits->map(function ($limit) {
                    return [
                        'trust_level' => $limit->trust_level,
                        'trust_level_name' => $this->getTrustLevelName($limit->trust_level),
                        'speed_limit_up' => $limit->speed_limit_up,
                        'speed_limit_down' => $limit->speed_limit_down,
                        'device_limit' => $limit->device_limit,
                        'connection_limit' => $limit->connection_limit,
                        'created_at' => $limit->created_at,
                        'updated_at' => $limit->updated_at,
                    ];
                })
            ]);
            
        } catch (\Exception $e) {
            Log::error('Failed to get group limits', ['error' => $e->getMessage()]);
            
            return response()->json([
                'success' => false,
                'error' => 'Failed to retrieve group limits'
            ], 500);
        }
    }
    
    /**
     * 获取特定信任等级的限制配置
     */
    public function show(int $trustLevel): JsonResponse
    {
        try {
            if ($trustLevel < 0 || $trustLevel > 4) {
                return response()->json([
                    'success' => false,
                    'error' => 'Invalid trust level. Must be between 0 and 4.'
                ], 400);
            }
            
            $groupLimit = $this->limitService->getGroupLimits($trustLevel);
            
            if (!$groupLimit) {
                return response()->json([
                    'success' => false,
                    'error' => 'Group limit not found'
                ], 404);
            }
            
            return response()->json([
                'success' => true,
                'data' => [
                    'trust_level' => $groupLimit->trust_level,
                    'trust_level_name' => $this->getTrustLevelName($groupLimit->trust_level),
                    'speed_limit_up' => $groupLimit->speed_limit_up,
                    'speed_limit_down' => $groupLimit->speed_limit_down,
                    'device_limit' => $groupLimit->device_limit,
                    'connection_limit' => $groupLimit->connection_limit,
                    'created_at' => $groupLimit->created_at,
                    'updated_at' => $groupLimit->updated_at,
                ]
            ]);
            
        } catch (\Exception $e) {
            Log::error('Failed to get group limit', [
                'trust_level' => $trustLevel,
                'error' => $e->getMessage()
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'Failed to retrieve group limit'
            ], 500);
        }
    }
    
    /**
     * 创建或更新用户组限制配置
     */
    public function store(Request $request): JsonResponse
    {
        try {
            $validated = $request->validate([
                'trust_level' => 'required|integer|min:0|max:4',
                'speed_limit_up' => 'nullable|integer|min:0',
                'speed_limit_down' => 'nullable|integer|min:0',
                'device_limit' => 'nullable|integer|min:0',
                'connection_limit' => 'nullable|integer|min:0',
            ]);
            
            // 验证限制配置
            $errors = $this->limitService->validateLimits($validated);
            if (!empty($errors)) {
                return response()->json([
                    'success' => false,
                    'error' => 'Invalid limit configuration',
                    'details' => $errors
                ], 400);
            }
            
            $groupLimit = $this->limitService->setGroupLimits(
                $validated['trust_level'],
                $validated
            );
            
            Log::info('Group limit updated', [
                'trust_level' => $validated['trust_level'],
                'admin_id' => $request->user()->id,
                'limits' => $validated
            ]);
            
            return response()->json([
                'success' => true,
                'message' => 'Group limit updated successfully',
                'data' => [
                    'trust_level' => $groupLimit->trust_level,
                    'trust_level_name' => $this->getTrustLevelName($groupLimit->trust_level),
                    'speed_limit_up' => $groupLimit->speed_limit_up,
                    'speed_limit_down' => $groupLimit->speed_limit_down,
                    'device_limit' => $groupLimit->device_limit,
                    'connection_limit' => $groupLimit->connection_limit,
                    'updated_at' => $groupLimit->updated_at,
                ]
            ]);
            
        } catch (\Illuminate\Validation\ValidationException $e) {
            return response()->json([
                'success' => false,
                'error' => 'Validation failed',
                'details' => $e->errors()
            ], 422);
            
        } catch (\Exception $e) {
            Log::error('Failed to update group limit', [
                'error' => $e->getMessage(),
                'admin_id' => $request->user()->id
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'Failed to update group limit'
            ], 500);
        }
    }
    
    /**
     * 批量更新用户组限制配置
     */
    public function batchUpdate(Request $request): JsonResponse
    {
        try {
            $validated = $request->validate([
                'limits' => 'required|array',
                'limits.*.trust_level' => 'required|integer|min:0|max:4',
                'limits.*.speed_limit_up' => 'nullable|integer|min:0',
                'limits.*.speed_limit_down' => 'nullable|integer|min:0',
                'limits.*.device_limit' => 'nullable|integer|min:0',
                'limits.*.connection_limit' => 'nullable|integer|min:0',
            ]);
            
            $groupLimits = [];
            foreach ($validated['limits'] as $limitData) {
                $trustLevel = $limitData['trust_level'];
                unset($limitData['trust_level']);
                $groupLimits[$trustLevel] = $limitData;
            }
            
            $results = $this->limitService->batchSetGroupLimits($groupLimits);
            
            Log::info('Batch group limits updated', [
                'count' => count($results),
                'admin_id' => $request->user()->id
            ]);
            
            return response()->json([
                'success' => true,
                'message' => 'Group limits updated successfully',
                'data' => collect($results)->map(function ($limit) {
                    return [
                        'trust_level' => $limit->trust_level,
                        'trust_level_name' => $this->getTrustLevelName($limit->trust_level),
                        'speed_limit_up' => $limit->speed_limit_up,
                        'speed_limit_down' => $limit->speed_limit_down,
                        'device_limit' => $limit->device_limit,
                        'connection_limit' => $limit->connection_limit,
                        'updated_at' => $limit->updated_at,
                    ];
                })->values()
            ]);
            
        } catch (\Illuminate\Validation\ValidationException $e) {
            return response()->json([
                'success' => false,
                'error' => 'Validation failed',
                'details' => $e->errors()
            ], 422);
            
        } catch (\Exception $e) {
            Log::error('Failed to batch update group limits', [
                'error' => $e->getMessage(),
                'admin_id' => $request->user()->id
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'Failed to update group limits'
            ], 500);
        }
    }
    
    /**
     * 删除用户组限制配置
     */
    public function destroy(int $trustLevel): JsonResponse
    {
        try {
            if ($trustLevel < 0 || $trustLevel > 4) {
                return response()->json([
                    'success' => false,
                    'error' => 'Invalid trust level. Must be between 0 and 4.'
                ], 400);
            }
            
            $deleted = UserGroupLimit::where('trust_level', $trustLevel)->delete();
            
            if ($deleted === 0) {
                return response()->json([
                    'success' => false,
                    'error' => 'Group limit not found'
                ], 404);
            }
            
            Log::info('Group limit deleted', [
                'trust_level' => $trustLevel,
                'admin_id' => request()->user()->id
            ]);
            
            return response()->json([
                'success' => true,
                'message' => 'Group limit deleted successfully'
            ]);
            
        } catch (\Exception $e) {
            Log::error('Failed to delete group limit', [
                'trust_level' => $trustLevel,
                'error' => $e->getMessage()
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'Failed to delete group limit'
            ], 500);
        }
    }
    
    /**
     * 获取默认限制配置模板
     */
    public function getDefaults(): JsonResponse
    {
        $defaults = [
            0 => [ // 新用户
                'trust_level' => 0,
                'trust_level_name' => 'New User',
                'speed_limit_up' => 50,
                'speed_limit_down' => 100,
                'device_limit' => 2,
                'connection_limit' => 5,
            ],
            1 => [ // 基础用户
                'trust_level' => 1,
                'trust_level_name' => 'Basic User',
                'speed_limit_up' => 100,
                'speed_limit_down' => 200,
                'device_limit' => 3,
                'connection_limit' => 10,
            ],
            2 => [ // 成员
                'trust_level' => 2,
                'trust_level_name' => 'Member',
                'speed_limit_up' => 200,
                'speed_limit_down' => 500,
                'device_limit' => 5,
                'connection_limit' => 20,
            ],
            3 => [ // 常规用户
                'trust_level' => 3,
                'trust_level_name' => 'Regular',
                'speed_limit_up' => 500,
                'speed_limit_down' => 1000,
                'device_limit' => 8,
                'connection_limit' => 50,
            ],
            4 => [ // 领导者
                'trust_level' => 4,
                'trust_level_name' => 'Leader',
                'speed_limit_up' => 1000,
                'speed_limit_down' => 2000,
                'device_limit' => 15,
                'connection_limit' => 100,
            ],
        ];
        
        return response()->json([
            'success' => true,
            'data' => array_values($defaults)
        ]);
    }
    
    /**
     * 应用默认配置
     */
    public function applyDefaults(Request $request): JsonResponse
    {
        try {
            $defaults = [
                0 => ['speed_limit_up' => 50, 'speed_limit_down' => 100, 'device_limit' => 2, 'connection_limit' => 5],
                1 => ['speed_limit_up' => 100, 'speed_limit_down' => 200, 'device_limit' => 3, 'connection_limit' => 10],
                2 => ['speed_limit_up' => 200, 'speed_limit_down' => 500, 'device_limit' => 5, 'connection_limit' => 20],
                3 => ['speed_limit_up' => 500, 'speed_limit_down' => 1000, 'device_limit' => 8, 'connection_limit' => 50],
                4 => ['speed_limit_up' => 1000, 'speed_limit_down' => 2000, 'device_limit' => 15, 'connection_limit' => 100],
            ];
            
            $results = $this->limitService->batchSetGroupLimits($defaults);
            
            Log::info('Default group limits applied', [
                'count' => count($results),
                'admin_id' => $request->user()->id
            ]);
            
            return response()->json([
                'success' => true,
                'message' => 'Default group limits applied successfully',
                'data' => collect($results)->map(function ($limit) {
                    return [
                        'trust_level' => $limit->trust_level,
                        'trust_level_name' => $this->getTrustLevelName($limit->trust_level),
                        'speed_limit_up' => $limit->speed_limit_up,
                        'speed_limit_down' => $limit->speed_limit_down,
                        'device_limit' => $limit->device_limit,
                        'connection_limit' => $limit->connection_limit,
                        'updated_at' => $limit->updated_at,
                    ];
                })->values()
            ]);
            
        } catch (\Exception $e) {
            Log::error('Failed to apply default group limits', [
                'error' => $e->getMessage(),
                'admin_id' => $request->user()->id
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'Failed to apply default group limits'
            ], 500);
        }
    }
    
    /**
     * 获取信任等级名称
     */
    private function getTrustLevelName(int $trustLevel): string
    {
        return match($trustLevel) {
            0 => 'New User',
            1 => 'Basic User',
            2 => 'Member',
            3 => 'Regular',
            4 => 'Leader',
            default => 'Unknown'
        };
    }
}