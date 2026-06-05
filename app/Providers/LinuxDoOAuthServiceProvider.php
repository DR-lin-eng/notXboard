<?php

namespace App\Providers;

use App\Services\OAuth\LinuxDoOAuthService;
use App\Services\OAuth\UserSyncService;
use App\Services\LimitControlService;
use App\Services\ApiKeyService;
use Illuminate\Support\ServiceProvider;
use Illuminate\Support\Facades\Log;

class LinuxDoOAuthServiceProvider extends ServiceProvider
{
    private static bool $oauthConfigWarningReported = false;

    /**
     * 注册服务
     */
    public function register(): void
    {
        // 注册 OAuth2 服务
        $this->app->singleton(LinuxDoOAuthService::class, function ($app) {
            return new LinuxDoOAuthService();
        });
        
        // 注册用户同步服务
        $this->app->singleton(UserSyncService::class, function ($app) {
            return new UserSyncService();
        });
        
        // 注册限制控制服务
        $this->app->singleton(LimitControlService::class, function ($app) {
            return new LimitControlService();
        });
        
        // 注册 API 密钥服务
        $this->app->singleton(ApiKeyService::class, function ($app) {
            return new ApiKeyService();
        });
    }
    
    /**
     * 启动服务
     */
    public function boot(): void
    {
        // 验证 Linux DO OAuth2 配置
        $this->validateOAuthConfig();
        
        // 注册事件监听器
        $this->registerEventListeners();
    }
    
    /**
     * 验证 OAuth2 配置
     */
    private function validateOAuthConfig(): void
    {
        if ($this->app->runningInConsole() || self::$oauthConfigWarningReported) {
            return;
        }

        $oauthEnabled = (bool) admin_setting('oauth_linux_do_enable', 1);
        if (!$oauthEnabled) {
            return;
        }

        $clientId = (string) admin_setting('oauth_linux_do_client_id', config('services.linux_do.client_id', ''));
        $clientSecret = (string) admin_setting('oauth_linux_do_client_secret', config('services.linux_do.client_secret', ''));

        if ($clientId === '' || $clientSecret === '') {
            self::$oauthConfigWarningReported = true;
            Log::debug('Linux DO OAuth2 credentials not configured', [
                'has_client_id' => $clientId !== '',
                'has_client_secret' => $clientSecret !== ''
            ]);
        }
    }
    
    /**
     * 注册事件监听器
     */
    private function registerEventListeners(): void
    {
        // 用户登录事件监听器
        \Illuminate\Support\Facades\Event::listen(
            \Illuminate\Auth\Events\Login::class,
            function ($event) {
                $user = $event->user;
                
                // 如果是 Linux DO 用户且需要同步信息
                if ($user->isLinuxDoUser() && $user->shouldSyncUserInfo()) {
                    try {
                        $userSyncService = app(UserSyncService::class);
                        $oauthService = app(LinuxDoOAuthService::class);
                        
                        $userSyncService->syncUserInfo($user, $oauthService);
                        
                        Log::info('User info synced on login', ['user_id' => $user->id]);
                    } catch (\Exception $e) {
                        Log::error('Failed to sync user info on login', [
                            'user_id' => $user->id,
                            'error' => $e->getMessage()
                        ]);
                    }
                }
                
                // 确保用户有 API 密钥
                if (empty($user->api_key)) {
                    try {
                        $apiKeyService = app(ApiKeyService::class);
                        $apiKeyService->generateApiKey($user);
                        
                        Log::info('API key generated on login', ['user_id' => $user->id]);
                    } catch (\Exception $e) {
                        Log::error('Failed to generate API key on login', [
                            'user_id' => $user->id,
                            'error' => $e->getMessage()
                        ]);
                    }
                }
            }
        );
        
        // 用户创建事件监听器
        \Illuminate\Support\Facades\Event::listen(
            \Illuminate\Auth\Events\Registered::class,
            function ($event) {
                $user = $event->user;
                
                // 为新用户生成 API 密钥
                if (empty($user->api_key)) {
                    try {
                        $apiKeyService = app(ApiKeyService::class);
                        $apiKeyService->generateApiKey($user);
                        
                        Log::info('API key generated for new user', ['user_id' => $user->id]);
                    } catch (\Exception $e) {
                        Log::error('Failed to generate API key for new user', [
                            'user_id' => $user->id,
                            'error' => $e->getMessage()
                        ]);
                    }
                }
                
                // 应用用户组限制
                try {
                    $limitService = app(LimitControlService::class);
                    $limitService->applyLimitsToUser($user);
                    
                    Log::info('Limits applied to new user', ['user_id' => $user->id]);
                } catch (\Exception $e) {
                    Log::error('Failed to apply limits to new user', [
                        'user_id' => $user->id,
                        'error' => $e->getMessage()
                    ]);
                }
            }
        );
    }
    
    /**
     * 获取提供的服务
     */
    public function provides(): array
    {
        return [
            LinuxDoOAuthService::class,
            UserSyncService::class,
            LimitControlService::class,
            ApiKeyService::class,
        ];
    }
}
