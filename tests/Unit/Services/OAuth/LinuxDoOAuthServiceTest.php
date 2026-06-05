<?php

namespace Tests\Unit\Services\OAuth;

use App\Services\OAuth\LinuxDoOAuthService;
use App\Exceptions\OAuthException;
use Illuminate\Support\Facades\Http;
use Tests\TestCase;

class LinuxDoOAuthServiceTest extends TestCase
{
    protected LinuxDoOAuthService $service;
    
    protected function setUp(): void
    {
        parent::setUp();
        
        // 设置测试配置
        config([
            'services.linux_do.client_id' => 'test_client_id',
            'services.linux_do.client_secret' => 'test_client_secret',
        ]);
        
        $this->service = new LinuxDoOAuthService();
    }
    
    public function test_service_can_be_instantiated_without_credentials_but_throws_on_use()
    {
        config([
            'services.linux_do.client_id' => null,
            'services.linux_do.client_secret' => null,
        ]);

        $service = new LinuxDoOAuthService();

        $this->assertFalse($service->isConfigured());
        $this->expectException(OAuthException::class);
        $this->expectExceptionMessage('Linux DO 登录未启用或未完成配置');

        $service->getAuthorizationUrl('https://example.com/callback', 'test_state');
    }
    
    public function test_get_authorization_url_generates_correct_url()
    {
        $redirectUri = 'https://example.com/callback';
        $state = 'test_state';
        
        $url = $this->service->getAuthorizationUrl($redirectUri, $state);
        
        $this->assertStringContainsString('https://connect.linux.do/oauth2/authorize', $url);
        $this->assertStringContainsString('client_id=test_client_id', $url);
        $this->assertStringContainsString('redirect_uri=' . urlencode($redirectUri), $url);
        $this->assertStringContainsString('state=' . $state, $url);
        $this->assertStringContainsString('response_type=code', $url);
        $this->assertStringContainsString('scope=read', $url);
    }
    
    public function test_exchange_code_for_token_success()
    {
        Http::fake([
            'connect.linux.do/oauth2/token' => Http::response([
                'access_token' => 'test_access_token',
                'refresh_token' => 'test_refresh_token',
                'expires_in' => 3600,
                'token_type' => 'Bearer',
            ], 200)
        ]);
        
        $result = $this->service->exchangeCodeForToken('test_code', 'https://example.com/callback');
        
        $this->assertEquals('test_access_token', $result['access_token']);
        $this->assertEquals('test_refresh_token', $result['refresh_token']);
        $this->assertEquals(3600, $result['expires_in']);
        $this->assertEquals('Bearer', $result['token_type']);
    }
    
    public function test_exchange_code_for_token_failure()
    {
        Http::fake([
            'connect.linux.do/oauth2/token' => Http::response([
                'error' => 'invalid_grant'
            ], 400)
        ]);
        
        $this->expectException(OAuthException::class);
        
        $this->service->exchangeCodeForToken('invalid_code', 'https://example.com/callback');
    }
    
    public function test_get_user_info_success()
    {
        Http::fake([
            'connect.linux.do/api/user' => Http::response([
                'id' => 123,
                'username' => 'testuser',
                'name' => 'Test User',
                'avatar_template' => '/user_avatar/testuser/{size}/123.png',
                'active' => true,
                'trust_level' => 2,
                'silenced' => false,
                'external_ids' => [],
            ], 200)
        ]);
        
        $result = $this->service->getUserInfo('test_access_token');
        
        $this->assertEquals(123, $result['id']);
        $this->assertEquals('testuser', $result['username']);
        $this->assertEquals('Test User', $result['name']);
        $this->assertEquals(2, $result['trust_level']);
        $this->assertFalse($result['silenced']);
    }
    
    public function test_get_user_info_missing_required_field()
    {
        Http::fake([
            'connect.linux.do/api/user' => Http::response([
                'id' => 123,
                'username' => 'testuser',
                // 缺少 'name' 字段
                'avatar_template' => '/user_avatar/testuser/{size}/123.png',
                'active' => true,
                'trust_level' => 2,
            ], 200)
        ]);
        
        $this->expectException(OAuthException::class);
        $this->expectExceptionMessage("Required field 'name' not found in user data");
        
        $this->service->getUserInfo('test_access_token');
    }
    
    public function test_refresh_token_success()
    {
        Http::fake([
            'connect.linux.do/oauth2/token' => Http::response([
                'access_token' => 'new_access_token',
                'refresh_token' => 'new_refresh_token',
                'expires_in' => 3600,
                'token_type' => 'Bearer',
            ], 200)
        ]);
        
        $result = $this->service->refreshToken('test_refresh_token');
        
        $this->assertEquals('new_access_token', $result['access_token']);
        $this->assertEquals('new_refresh_token', $result['refresh_token']);
    }
    
    public function test_validate_token_success()
    {
        Http::fake([
            'connect.linux.do/api/user' => Http::response([], 200)
        ]);
        
        $result = $this->service->validateToken('valid_token');
        
        $this->assertTrue($result);
    }
    
    public function test_validate_token_failure()
    {
        Http::fake([
            'connect.linux.do/api/user' => Http::response([], 401)
        ]);
        
        $result = $this->service->validateToken('invalid_token');
        
        $this->assertFalse($result);
    }
}
