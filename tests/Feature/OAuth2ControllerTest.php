<?php

namespace Tests\Feature;

use App\Models\User;
use App\Services\OAuth\LinuxDoOAuthService;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\Http;
use Tests\TestCase;

class OAuth2ControllerTest extends TestCase
{
    use RefreshDatabase;
    
    protected function setUp(): void
    {
        parent::setUp();
        
        // 设置测试配置
        config([
            'services.linux_do.client_id' => 'test_client_id',
            'services.linux_do.client_secret' => 'test_client_secret',
            'app.url' => 'https://example.com',
        ]);
    }
    
    public function test_redirect_generates_authorization_url()
    {
        $response = $this->get('/api/v1/passport/oauth2/linux-do/redirect');
        
        $response->assertStatus(302);
        
        $location = $response->headers->get('Location');
        $this->assertStringContainsString('connect.linux.do/oauth2/authorize', $location);
        $this->assertStringContainsString('client_id=test_client_id', $location);
        $this->assertStringContainsString('response_type=code', $location);
    }

    public function test_redirect_falls_back_to_portal_login_when_credentials_are_missing_for_html_requests(): void
    {
        config([
            'services.linux_do.client_id' => null,
            'services.linux_do.client_secret' => null,
        ]);

        $response = $this->get('/api/v1/passport/oauth2/linux-do/redirect');

        $response->assertRedirect('/app#/login');
    }
    
    public function test_callback_with_valid_code_creates_user()
    {
        $headers = ['Accept' => 'application/json'];

        // Mock HTTP responses
        Http::fake([
            'connect.linux.do/oauth2/token' => Http::response([
                'access_token' => 'test_access_token',
                'refresh_token' => 'test_refresh_token',
                'expires_in' => 3600,
                'token_type' => 'Bearer',
            ], 200),
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

        $redirect = $this->withHeaders($headers)->get('/api/v1/passport/oauth2/linux-do/redirect');
        $redirect->assertStatus(302);
        $location = $redirect->headers->get('Location');
        parse_str(parse_url($location, PHP_URL_QUERY) ?? '', $query);
        $state = $query['state'] ?? null;
        $this->assertNotEmpty($state);

        $response = $this->withHeaders($headers)
            ->get('/api/v1/passport/oauth2/linux-do/callback?code=test_code&state=' . urlencode($state));
        
        $response->assertStatus(200);
        $response->assertJson([
            'success' => true,
            'message' => 'Login successful',
        ]);
        
        $responseData = $response->json();
        $this->assertArrayHasKey('data', $responseData);
        $this->assertArrayHasKey('user', $responseData['data']);
        $this->assertArrayHasKey('auth', $responseData['data']);
        $this->assertEquals('linux_do', $responseData['data']['provider']);
        
        // 验证用户已创建
        $this->assertDatabaseHas('v2_user', [
            'linux_do_id' => '123',
            'linux_do_username' => 'testuser',
            'trust_level' => 2,
        ]);
    }
    
    public function test_callback_with_invalid_state_fails()
    {
        $redirect = $this->get('/api/v1/passport/oauth2/linux-do/redirect');
        $redirect->assertStatus(302);
        $response = $this->get('/api/v1/passport/oauth2/linux-do/callback?code=test_code&state=wrong_state');
        
        $response->assertStatus(401);
        $response->assertJson([
            'success' => false,
            'error' => 'Invalid state parameter',
        ]);
    }
    
    public function test_callback_without_session_state_fails()
    {
        $response = $this->get('/api/v1/passport/oauth2/linux-do/callback?code=test_code&state=test_state');
        
        $response->assertStatus(401);
        $response->assertJson([
            'success' => false,
            'error' => 'Invalid state parameter',
        ]);
    }
    
    public function test_refresh_token_for_authenticated_user()
    {
        // 创建已认证的用户
        $user = User::create([
            'email' => 'test@example.com',
            'password' => bcrypt('password'),
            'token' => 'test_token',
            'uuid' => 'test-uuid',
            'linux_do_id' => '123',
            'oauth_provider' => 'linux_do',
            'oauth_refresh_token' => 'test_refresh_token',
            'oauth_expires_at' => now()->addHour(),
        ]);
        
        // Mock HTTP response for token refresh
        Http::fake([
            'connect.linux.do/oauth2/token' => Http::response([
                'access_token' => 'new_access_token',
                'refresh_token' => 'new_refresh_token',
                'expires_in' => 3600,
                'token_type' => 'Bearer',
            ], 200)
        ]);
        
        $response = $this->actingAs($user, 'sanctum')
                         ->post('/api/v1/passport/oauth2/refresh');
        
        $response->assertStatus(200);
        $response->assertJson([
            'success' => true,
            'message' => 'Token refreshed successfully',
        ]);
        
        // 验证用户令牌已更新
        $user->refresh();
        $this->assertEquals('new_access_token', $user->oauth_access_token);
        $this->assertEquals('new_refresh_token', $user->oauth_refresh_token);
    }
    
    public function test_refresh_token_without_authentication_fails()
    {
        $response = $this->post('/api/v1/passport/oauth2/refresh');
        
        $response->assertStatus(401);
    }
    
    public function test_sync_user_info_for_authenticated_user()
    {
        // 创建已认证的用户
        $user = User::create([
            'email' => 'test@example.com',
            'password' => bcrypt('password'),
            'token' => 'test_token',
            'uuid' => 'test-uuid',
            'linux_do_id' => '123',
            'oauth_provider' => 'linux_do',
            'oauth_access_token' => 'test_access_token',
            'oauth_expires_at' => now()->addHour(),
            'trust_level' => 1,
        ]);
        
        // Mock HTTP response for user info
        Http::fake([
            'connect.linux.do/api/user' => Http::response([
                'id' => 123,
                'username' => 'updateduser',
                'name' => 'Updated User',
                'avatar_template' => '/user_avatar/updateduser/{size}/123.png',
                'active' => true,
                'trust_level' => 3, // 信任等级变化
                'silenced' => false,
                'external_ids' => [],
            ], 200)
        ]);
        
        $response = $this->actingAs($user, 'sanctum')
                         ->post('/api/v1/passport/oauth2/sync');
        
        $response->assertStatus(200);
        $response->assertJson([
            'success' => true,
            'message' => 'User info synced successfully',
        ]);
        
        $responseData = $response->json();
        $this->assertEquals('updateduser', $responseData['data']['user']['linux_do_username']);
        $this->assertEquals(3, $responseData['data']['user']['trust_level']);
        
        // 验证用户信息已更新
        $user->refresh();
        $this->assertEquals('updateduser', $user->linux_do_username);
        $this->assertEquals(3, $user->trust_level);
    }
}
