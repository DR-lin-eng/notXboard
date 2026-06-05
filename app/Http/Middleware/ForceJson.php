<?php

namespace App\Http\Middleware;

use Closure;

class ForceJson
{
    /**
     * Handle an incoming request.
     *
     * @param \Illuminate\Http\Request $request
     * @param \Closure $next
     * @param string|null $guard
     * @return mixed
     */
    public function handle($request, Closure $next, $guard = null)
    {
        // OAuth browser redirect/callback endpoints need normal browser semantics.
        if (
            $request->is('api/v1/passport/oauth2/linux-do/redirect')
            || $request->is('api/v1/passport/oauth2/linux-do/callback')
        ) {
            return $next($request);
        }

        $request->headers->set('accept', 'application/json');
        return $next($request);
    }
}
