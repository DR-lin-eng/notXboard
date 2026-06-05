<?php

namespace App\Http\Middleware;

use App\Services\Auth\BannedUserGuard;
use Closure;
use Illuminate\Http\Request;
use Illuminate\Http\JsonResponse;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Log;

class SuperAdminMiddleware
{
    /**
     * 处理传入的请求
     *
     * @param  \Illuminate\Http\Request  $request
     * @param  \Closure  $next
     * @return mixed
     */
    public function handle(Request $request, Closure $next)
    {
        $user = $request->user() ?? Auth::guard('sanctum')->user();
        
        // 检查用户是否已认证
        if (!$user) {
            Log::warning('Unauthenticated access attempt to super admin route', [
                'ip' => $request->ip(),
                'route' => $request->route()?->getName(),
                'url' => $request->url()
            ]);
            
            return $this->unauthorizedResponse('Authentication required');
        }

        app(BannedUserGuard::class)->rejectIfBanned($user);
        
        // 检查用户是否为超级管理员
        if (!$user->is_super_admin) {
            Log::warning('Non-super-admin access attempt', [
                'user_id' => $user->id,
                'email' => $user->email,
                'is_admin' => $user->is_admin,
                'route' => $request->route()?->getName(),
                'url' => $request->url(),
                'ip' => $request->ip()
            ]);
            
            return $this->forbiddenResponse('Super administrator privileges required');
        }
        
        // 记录超级管理员操作
        Log::info('Super admin access granted', [
            'user_id' => $user->id,
            'email' => $user->email,
            'route' => $request->route()?->getName(),
            'method' => $request->method(),
            'url' => $request->url(),
            'ip' => $request->ip()
        ]);
        
        return $next($request);
    }
    
    /**
     * 返回未认证响应
     */
    private function unauthorizedResponse(string $message): JsonResponse
    {
        return response()->json([
            'success' => false,
            'error' => $message,
            'error_code' => 'AUTHENTICATION_REQUIRED',
            'redirect_url' => '/login'
        ], 401);
    }
    
    /**
     * 返回权限不足响应
     */
    private function forbiddenResponse(string $message): JsonResponse
    {
        return response()->json([
            'success' => false,
            'error' => $message,
            'error_code' => 'INSUFFICIENT_PRIVILEGES',
        ], 403);
    }
}
