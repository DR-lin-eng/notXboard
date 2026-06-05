<?php

namespace App\Http\Controllers\V1\Passport;

use App\Http\Controllers\Controller;
use App\Models\User;
use App\Services\OAuth\LinuxDoOAuthService;
use App\Services\OAuth\UserSyncService;
use App\Services\Auth\RegisterModeService;
use App\Services\AuthService;
use App\Exceptions\OAuthException;
use Illuminate\Http\Request;
use Illuminate\Http\JsonResponse;
use Illuminate\Http\RedirectResponse;
use Symfony\Component\HttpFoundation\Response;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Str;

class OAuth2Controller extends Controller
{
    protected LinuxDoOAuthService $oauthService;
    protected UserSyncService $userSyncService;
    
    public function __construct(
        LinuxDoOAuthService $oauthService,
        UserSyncService $userSyncService
    ) {
        $this->oauthService = $oauthService;
        $this->userSyncService = $userSyncService;
    }
    
    /**
     * 重定向到 Linux DO Connect 授权页面
     */
    public function redirect(Request $request): Response
    {
        try {
            $redirectUri = $this->getRedirectUri($request);
            $state = Str::random(32);
            $inviteCode = trim((string) $request->input('invite_code', ''));

            // Store state in cache to validate callback (API routes are stateless)
            Cache::put($this->getStateCacheKey($state), [
                'valid' => true,
                'invite_code' => $inviteCode !== '' ? $inviteCode : null,
            ], now()->addMinutes(10));
            
            $authUrl = $this->oauthService->getAuthorizationUrl($redirectUri, $state);
            
            Log::info('OAuth redirect initiated', [
                'redirect_uri' => $redirectUri,
                'state' => $state
            ]);
            
            return redirect($authUrl);
            
        } catch (OAuthException $e) {
            Log::error('OAuth redirect failed', ['error' => $e->getMessage()]);

            if ($request->expectsJson()) {
                return $e->render($request);
            }

            return redirect('/app#/login');
        }
    }
    
    /**
     * 处理 OAuth2 授权回调
     */
    public function callback(Request $request)
    {
        try {
            // 验证必需参数
            $request->validate([
                'code' => 'required|string',
                'state' => 'required|string',
            ]);
            
            // 验证 state 参数防止 CSRF 攻击
            $state = (string) $request->input('state');
            $stateKey = $this->getStateCacheKey($state);
            $statePayload = Cache::pull($stateKey);
            if (is_array($statePayload)) {
                $isStateValid = (bool) ($statePayload['valid'] ?? false);
                $inviteCode = trim((string) ($statePayload['invite_code'] ?? ''));
            } else {
                $isStateValid = (bool) $statePayload;
                $inviteCode = '';
            }

            if (!$isStateValid) {
                throw OAuthException::authorizationFailed('Invalid state parameter');
            }

            $code = $request->input('code');
            $redirectUri = $this->getRedirectUri($request);
            
            // 交换授权码获取访问令牌
            $tokenData = $this->oauthService->exchangeCodeForToken($code, $redirectUri);
            
            // 获取用户信息
            $userInfo = $this->oauthService->getUserInfo($tokenData['access_token']);

            $existingUser = User::query()
                ->where('linux_do_id', $userInfo['id'] ?? null)
                ->first();

            if (!$existingUser && !app(RegisterModeService::class)->allowsOauthRegistration()) {
                return $this->renderRegistrationBlocked(
                    request: $request,
                    message: 'OAuth2 registration is disabled'
                );
            }

            if (
                !$existingUser
                && (int) admin_setting('invite_force', 0) === 1
                && $inviteCode === ''
            ) {
                return $this->renderRegistrationBlocked(
                    request: $request,
                    message: 'You must use the invitation code to register'
                );
            }
            
            // 同步或创建用户
            $user = $this->userSyncService->syncFromLinuxDo($userInfo, $tokenData, $inviteCode);

            if ($user->banned) {
                return $this->renderRegistrationBlocked(
                    request: $request,
                    message: $user->getSuspensionMessage()
                );
            }
            
            // 生成认证数据
            $authService = new AuthService($user);
            $authData = $authService->generateAuthData();
            
            Log::info('OAuth login successful', [
                'user_id' => $user->id,
                'linux_do_id' => $user->linux_do_id,
                'trust_level' => $user->trust_level
            ]);

            $payload = [
                'success' => true,
                'message' => 'Login successful',
                'data' => [
                    'user' => [
                        'id' => $user->id,
                        'email' => $user->email,
                        'linux_do_username' => $user->linux_do_username,
                        'linux_do_name' => $user->linux_do_name,
                        'linux_do_avatar' => $user->linux_do_avatar,
                        'trust_level' => $user->trust_level,
                        'is_admin' => $user->is_admin,
                        'is_super_admin' => $user->is_super_admin,
                        'api_key' => $user->api_key,
                    ],
                    'auth' => $authData,
                    'provider' => 'linux_do',
                ]
            ];

            $accept = (string) $request->header('Accept', '');
            $wantsHtml = str_contains($accept, 'text/html') && !str_contains($accept, 'application/json');
            if ($wantsHtml) {
                return response()->view('oauth-linux-do-callback', ['payload' => $payload], 200);
            }

            return response()->json($payload, 200);
            
        } catch (OAuthException $e) {
            Log::error('OAuth callback failed', [
                'error' => $e->getMessage(),
                'code' => $request->input('code', 'N/A')
            ]);
            
            return $e->render($request);
            
        } catch (\Exception $e) {
            Log::error('OAuth callback unexpected error', [
                'error' => $e->getMessage(),
                'trace' => $e->getTraceAsString()
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'An unexpected error occurred during login',
                'redirect_url' => '/login'
            ], 500);
        }
    }
    
    /**
     * 刷新访问令牌
     */
    public function refresh(Request $request): JsonResponse
    {
        try {
            $user = $request->user();
            
            if (!$user || !$user->isLinuxDoUser()) {
                throw OAuthException::authorizationFailed('User not authenticated with Linux DO');
            }
            
            if (!$user->oauth_refresh_token) {
                throw OAuthException::tokenRefreshFailed('No refresh token available');
            }
            
            // 刷新令牌
            $success = $this->userSyncService->refreshUserToken($user, $this->oauthService);
            
            if (!$success) {
                throw OAuthException::tokenRefreshFailed('Failed to refresh token');
            }
            
            // 重新生成认证数据
            $authService = new AuthService($user);
            $authData = $authService->generateAuthData();
            
            Log::info('Token refresh successful', ['user_id' => $user->id]);
            
            return response()->json([
                'success' => true,
                'message' => 'Token refreshed successfully',
                'data' => [
                    'auth' => $authData,
                    'expires_at' => $user->oauth_expires_at->toISOString(),
                ]
            ]);
            
        } catch (OAuthException $e) {
            Log::error('Token refresh failed', [
                'error' => $e->getMessage(),
                'user_id' => $request->user()?->id
            ]);
            
            return $e->render($request);
            
        } catch (\Exception $e) {
            Log::error('Token refresh unexpected error', [
                'error' => $e->getMessage(),
                'user_id' => $request->user()?->id
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'An unexpected error occurred during token refresh'
            ], 500);
        }
    }
    
    /**
     * 同步用户信息
     */
    public function syncUserInfo(Request $request): JsonResponse
    {
        try {
            $user = $request->user();
            
            if (!$user || !$user->isLinuxDoUser()) {
                throw OAuthException::authorizationFailed('User not authenticated with Linux DO');
            }
            
            // 同步用户信息
            $success = $this->userSyncService->syncUserInfo($user, $this->oauthService);
            
            if (!$success) {
                throw OAuthException::userInfoFailed('Failed to sync user info');
            }
            
            Log::info('User info sync successful', ['user_id' => $user->id]);

            $updatedAt = $user->updated_at;
            if ($updatedAt instanceof \DateTimeInterface) {
                $updatedAtIso = $updatedAt->format(\DateTimeInterface::ATOM);
            } elseif ($updatedAt !== null) {
                $updatedAtIso = \Illuminate\Support\Carbon::createFromTimestamp((int) $updatedAt)->toISOString();
            } else {
                $updatedAtIso = null;
            }
            
            return response()->json([
                'success' => true,
                'message' => 'User info synced successfully',
                'data' => [
                    'user' => [
                        'id' => $user->id,
                        'linux_do_username' => $user->linux_do_username,
                        'linux_do_name' => $user->linux_do_name,
                        'linux_do_avatar' => $user->linux_do_avatar,
                        'trust_level' => $user->trust_level,
                        'is_silenced' => $user->is_silenced,
                        'updated_at' => $updatedAtIso,
                    ]
                ]
            ]);
            
        } catch (OAuthException $e) {
            Log::error('User info sync failed', [
                'error' => $e->getMessage(),
                'user_id' => $request->user()?->id
            ]);
            
            return $e->render($request);
            
        } catch (\Exception $e) {
            Log::error('User info sync unexpected error', [
                'error' => $e->getMessage(),
                'user_id' => $request->user()?->id
            ]);
            
            return response()->json([
                'success' => false,
                'error' => 'An unexpected error occurred during user info sync'
            ], 500);
        }
    }
    
    /**
     * 获取重定向 URI
     */
    private function getRedirectUri(Request $request): string
    {
        $configuredUri = (string) admin_setting('oauth_linux_do_redirect_uri', '');
        if (!blank($configuredUri)) {
            return $configuredUri;
        }

        $baseUrl = (string) (admin_setting('app_url', config('app.url')) ?: config('app.url'));
        $baseUrl = rtrim($baseUrl, '/');
        return $baseUrl . '/api/v1/passport/oauth2/linux-do/callback';
    }

    private function getStateCacheKey(string $state): string
    {
        return 'oauth:linux_do:state:' . $state;
    }

    private function renderRegistrationBlocked(Request $request, string $message)
    {
        $payload = [
            'success' => false,
            'error' => $message,
            'message' => $message,
        ];

        $accept = (string) $request->header('Accept', '');
        $wantsHtml = str_contains($accept, 'text/html') && !str_contains($accept, 'application/json');
        if ($wantsHtml) {
            return response()->view('oauth-linux-do-callback', ['payload' => $payload], 403);
        }

        return response()->json($payload, 403);
    }
}
