<?php

namespace App\Http\Middleware;

use App\Exceptions\ApiException;
use Closure;
use App\Models\User;
use Illuminate\Support\Facades\Auth;
use Illuminate\Support\Facades\Cache;

class Client
{
    private const TOKEN_REGEX = '/^[a-f0-9]{32}$/i';
    private const INVALID_TOKEN_LIMIT = 30;
    private const INVALID_TOKEN_TTL_SECONDS = 600;

    /**
     * Handle an incoming request.
     *
     * @param \Illuminate\Http\Request $request
     * @param \Closure $next
     * @return mixed
     */
    public function handle($request, Closure $next)
    {
        $pathKey = (string) $request->route('path');
        if ($pathKey !== '') {
            [$queryKey, $queryValue, $token] = $this->extractTokenFromQuery($request->query()) ?? [null, null, null];
            if (!$queryKey || !$queryValue || !$token || !preg_match(self::TOKEN_REGEX, $token)) {
                $this->recordInvalidAttempt($request);
                throw new ApiException('token is error',403);
            }

            $user = User::where('token', $token)->first();
            if (!$user) {
                $this->recordInvalidAttempt($request);
                throw new ApiException('token is error',403);
            }

            $user->ensureSubscribeSecrets();

            if ($user->subscribe_path !== $pathKey) {
                $this->recordInvalidAttempt($request);
                throw new ApiException('token is error',403);
            }
            if ($user->subscribe_key !== $queryKey) {
                $this->recordInvalidAttempt($request);
                throw new ApiException('token is error',403);
            }
            if (!str_ends_with($queryValue, $token)) {
                $this->recordInvalidAttempt($request);
                throw new ApiException('token is error',403);
            }
            if (!$user->subscribe_salt || (string) $request->query($user->subscribe_salt) !== '1') {
                $this->recordInvalidAttempt($request);
                throw new ApiException('token is error',403);
            }

            Auth::setUser($user);
            return $next($request);
        }

        $token = $request->input('token', $request->route('token'));
        if (empty($token)) {
            $this->recordInvalidAttempt($request);
            throw new ApiException('token is null',403);
        }
        if (!preg_match(self::TOKEN_REGEX, $token)) {
            $this->recordInvalidAttempt($request);
            throw new ApiException('token is error',403);
        }
        $user = User::where('token', $token)->first();
        if (!$user) {
            $this->recordInvalidAttempt($request);
            throw new ApiException('token is error',403);
        }

        Auth::setUser($user);
        return $next($request);
    }

    private function extractTokenFromQuery(array $query): ?array
    {
        foreach ($query as $key => $value) {
            if (!is_string($key) || !is_string($value)) {
                continue;
            }
            if (preg_match('/([a-f0-9]{32})$/i', $value, $matches)) {
                return [$key, $value, $matches[1]];
            }
        }
        return null;
    }

    private function recordInvalidAttempt($request): void
    {
        $ip = (string) $request->ip();
        if ($ip === '') {
            return;
        }
        $key = "subscribe:invalid:{$ip}";
        $attempts = (int) Cache::get($key, 0);
        $attempts++;
        Cache::put($key, $attempts, self::INVALID_TOKEN_TTL_SECONDS);
        if ($attempts > self::INVALID_TOKEN_LIMIT) {
            throw new ApiException('Too many invalid token requests', 429);
        }
    }
}
