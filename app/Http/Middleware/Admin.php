<?php

namespace App\Http\Middleware;

use App\Exceptions\ApiException;
use App\Services\Auth\BannedUserGuard;
use Illuminate\Support\Facades\Auth;
use Closure;
use App\Models\User;

class Admin
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
        /** @var User|null $user */
        $user = Auth::guard('sanctum')->user();
        
        if (!$user || !$user->is_admin) {
            return response()->json(['message' => 'Unauthorized'], 403);
        }

        app(BannedUserGuard::class)->rejectIfBanned($user);
        
        return $next($request);
    }
}
