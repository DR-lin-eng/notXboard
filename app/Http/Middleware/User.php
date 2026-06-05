<?php

namespace App\Http\Middleware;

use App\Exceptions\ApiException;
use App\Services\Auth\BannedUserGuard;
use Auth;
use Closure;

class User
{
    /**
     * Handle an incoming request.
     *
     * @param \Illuminate\Http\Request $request
     * @param \Closure $next
     * @return mixed
     */
    public function handle($request, Closure $next)
    {
        if (!Auth::guard('sanctum')->check()) {
            throw new ApiException('未登录或登陆已过期', 403);
        }

        $user = Auth::guard('sanctum')->user();
        app(BannedUserGuard::class)->rejectIfBanned($user);

        return $next($request);
    }
}
