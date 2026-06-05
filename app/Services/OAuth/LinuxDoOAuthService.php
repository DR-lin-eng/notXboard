<?php

namespace App\Services\OAuth;

use App\Models\User;
use App\Exceptions\OAuthException;
use Illuminate\Support\Facades\Http;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Str;
use Exception;

class LinuxDoOAuthService
{
    private const AUTHORIZE_URL = 'https://connect.linux.do/oauth2/authorize';
    private const TOKEN_URL = 'https://connect.linux.do/oauth2/token';
    private const USER_INFO_URL = 'https://connect.linux.do/api/user';
    
    private string $clientId;
    private string $clientSecret;
    private bool $isConfigured;
    
    public function __construct()
    {
        $config = self::resolveOauthConfig();
        $this->clientId = $config['client_id'];
        $this->clientSecret = $config['client_secret'];
        $this->isConfigured = $config['enabled'] && $this->clientId !== '' && $this->clientSecret !== '';
    }

    private function ensureConfigured(): void
    {
        if (!$this->isConfigured) {
            throw OAuthException::configurationError('Linux DO 登录未启用或未完成配置');
        }
    }

    public function isConfigured(): bool
    {
        return $this->isConfigured;
    }

    public static function isAvailable(): bool
    {
        $config = self::resolveOauthConfig();

        return $config['enabled']
            && $config['client_id'] !== ''
            && $config['client_secret'] !== '';
    }

    private static function resolveOauthConfig(): array
    {
        return [
            'enabled' => (bool) admin_setting('oauth_linux_do_enable', 1),
            'client_id' => trim((string) admin_setting('oauth_linux_do_client_id', config('services.linux_do.client_id', ''))),
            'client_secret' => trim((string) admin_setting('oauth_linux_do_client_secret', config('services.linux_do.client_secret', ''))),
        ];
    }
    
    /**
     * 生成授权 URL
     */
    public function getAuthorizationUrl(string $redirectUri, ?string $state = null): string
    {
        $this->ensureConfigured();
        $state = $state ?: Str::random(32);
        
        $params = http_build_query([
            'client_id' => $this->clientId,
            'redirect_uri' => $redirectUri,
            'response_type' => 'code',
            'scope' => 'read',
            'state' => $state,
        ]);
        
        return self::AUTHORIZE_URL . '?' . $params;
    }
    
    /**
     * 使用授权码交换访问令牌
     */
    public function exchangeCodeForToken(string $code, string $redirectUri): array
    {
        $this->ensureConfigured();
        try {
            $response = $this->httpClient()->asForm()->post(self::TOKEN_URL, [
                'client_id' => $this->clientId,
                'client_secret' => $this->clientSecret,
                'code' => $code,
                'grant_type' => 'authorization_code',
                'redirect_uri' => $redirectUri,
            ]);
            
            if (!$response->successful()) {
                $body = $this->responseSnippet($response);
                Log::error('Linux DO token exchange failed', [
                    'status' => $response->status(),
                    'body' => $body
                ]);
                throw OAuthException::tokenExchangeFailed('Failed to exchange code for token: ' . $body);
            }
            
            $data = $response->json();
            
            if (!isset($data['access_token'])) {
                throw OAuthException::tokenExchangeFailed('Access token not found in response');
            }
            
            return [
                'access_token' => $data['access_token'],
                'refresh_token' => $data['refresh_token'] ?? null,
                'expires_in' => $data['expires_in'] ?? 3600,
                'token_type' => $data['token_type'] ?? 'Bearer',
            ];
            
        } catch (Exception $e) {
            Log::error('OAuth token exchange error', [
                'error' => $e->getMessage(),
                'code_length' => strlen($code)
            ]);
            throw $e;
        }
    }
    
    /**
     * 使用访问令牌获取用户信息
     */
    public function getUserInfo(string $accessToken): array
    {
        $this->ensureConfigured();
        try {
            $response = $this->httpClient()->withHeaders([
                'Authorization' => 'Bearer ' . $accessToken,
                'Accept' => 'application/json',
            ])->get(self::USER_INFO_URL);
            
            if (!$response->successful()) {
                $body = $this->responseSnippet($response);
                Log::error('Linux DO user info request failed', [
                    'status' => $response->status(),
                    'body' => $body
                ]);
                throw OAuthException::userInfoFailed('Failed to get user info: ' . $body);
            }
            
            $userData = $response->json();
            
            // 验证必需字段
            $requiredFields = ['id', 'username', 'name', 'avatar_template', 'active', 'trust_level'];
            foreach ($requiredFields as $field) {
                if (!isset($userData[$field])) {
                    throw OAuthException::userInfoFailed("Required field '{$field}' not found in user data");
                }
            }
            
            return [
                'id' => $userData['id'],
                'username' => $userData['username'],
                'name' => $userData['name'],
                'avatar_template' => $userData['avatar_template'],
                'active' => $userData['active'],
                'trust_level' => $userData['trust_level'],
                'silenced' => $userData['silenced'] ?? false,
                'external_ids' => $userData['external_ids'] ?? [],
            ];
            
        } catch (Exception $e) {
            Log::error('OAuth user info error', [
                'error' => $e->getMessage(),
                'token' => substr($accessToken, 0, 10) . '...'
            ]);
            throw $e;
        }
    }
    
    /**
     * 刷新访问令牌
     */
    public function refreshToken(string $refreshToken): array
    {
        $this->ensureConfigured();
        try {
            $response = $this->httpClient()->asForm()->post(self::TOKEN_URL, [
                'client_id' => $this->clientId,
                'client_secret' => $this->clientSecret,
                'refresh_token' => $refreshToken,
                'grant_type' => 'refresh_token',
            ]);
            
            if (!$response->successful()) {
                $body = $this->responseSnippet($response);
                Log::error('Linux DO token refresh failed', [
                    'status' => $response->status(),
                    'body' => $body
                ]);
                throw OAuthException::tokenRefreshFailed('Failed to refresh token: ' . $body);
            }
            
            $data = $response->json();
            
            if (!isset($data['access_token'])) {
                throw OAuthException::tokenRefreshFailed('Access token not found in refresh response');
            }
            
            return [
                'access_token' => $data['access_token'],
                'refresh_token' => $data['refresh_token'] ?? $refreshToken,
                'expires_in' => $data['expires_in'] ?? 3600,
                'token_type' => $data['token_type'] ?? 'Bearer',
            ];
            
        } catch (Exception $e) {
            Log::error('OAuth token refresh error', [
                'error' => $e->getMessage(),
                'refresh_token' => substr($refreshToken, 0, 10) . '...'
            ]);
            throw $e;
        }
    }
    
    /**
     * 验证访问令牌是否有效
     */
    public function validateToken(string $accessToken): bool
    {
        if (!$this->isConfigured) {
            Log::debug('Linux DO OAuth2 credentials not configured when validating token');
            return false;
        }

        try {
            $response = $this->httpClient()->withHeaders([
                'Authorization' => 'Bearer ' . $accessToken,
                'Accept' => 'application/json',
            ])->get(self::USER_INFO_URL);
            
            return $response->successful();
            
        } catch (Exception $e) {
            Log::warning('Token validation failed', [
                'error' => $e->getMessage(),
                'token' => substr($accessToken, 0, 10) . '...'
            ]);
            return false;
        }
    }

    private function httpClient(): \Illuminate\Http\Client\PendingRequest
    {
        return Http::timeout(15)->connectTimeout(5)->acceptJson();
    }

    private function responseSnippet($response): string
    {
        return Str::limit(trim((string) $response->body()), 240, '...');
    }
}
