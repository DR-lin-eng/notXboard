<?php

namespace Tests\Feature;

use App\Models\User;
use App\Models\UserGroupLimit;
use App\Services\OAuth\LinuxDoOAuthService;
use App\Services\ApiKeyService;
use App\Services\LimitControlService;
use Illuminate\Support\Facades\Auth;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\Http;
use Tests\TestCase;

class LinuxDoOAuthIntegrationTest extends TestCase
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
        
        // 创建用户组限制
        UserGroupLimit::create([
            'trust_level' => 2,
            'speed_limit_up' => 200,
            'speed_limit_down' => 500,
            'device_limit' => 5,
            'connection_limit' => 20,
        ]);
    }
    
    public function test_complete_oauth_login_flow()
    {
        // Mock HTTP responses for OAuth flow
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
        
        // 1. 开始 OAuth 重定向
        $response = $this->get('/api/v1/passport/oauth2/linux-do/redirect');
        $response->assertStatus(302);
        $location = $response->headers->get('Location');
        parse_str(parse_url($location, PHP_URL_QUERY) ?? '', $query);
        $state = $query['state'] ?? null;
        $this->assertNotEmpty($state);
        
        // 2. 模拟 OAuth 回调
        $response = $this->withHeaders([
                'Accept' => 'application/json',
            ])
            ->get('/api/v1/passport/oauth2/linux-do/callback?code=test_code&state=' . urlencode($state));
        
        $response->assertStatus(200);
        $response->assertJson([
            'success' => true,
            'message' => 'Login successful',
        ]);
        
        // 3. 验证用户已创建
        $user = User::where('linux_do_id', '123')->first();
        $this->assertNotNull($user);
        $this->assertEquals('testuser', $user->linux_do_username);
        $this->assertEquals(2, $user->trust_level);
        $this->assertNotNull($user->api_key);
        $this->assertStringStartsWith('xb_', $user->api_key);
        
        // 4. 验证用户限制已应用
        $limitService = app(LimitControlService::class);
        $effectiveLimits = $limitService->getEffectiveLimits($user);
        $this->assertEquals(200, $effectiveLimits['speed_limit_up']);
        $this->assertEquals(500, $effectiveLimits['speed_limit_down']);
        $this->assertEquals(5, $effectiveLimits['device_limit']);
        
        return $user;
    }
    
    public function test_api_key_management_flow()
    {
        $user = User::factory()->create([
            'linux_do_id' => '123',
            'oauth_provider' => 'linux_do',
            'trust_level' => 2,
        ]);
        app(ApiKeyService::class)->generateApiKey($user);
        
        // 1. 获取 API 密钥信息
        $response = $this->actingAs($user, 'sanctum')
                         ->get('/api/v1/user/api-key');
        
        $response->assertStatus(200);
        $response->assertJsonStructure([
            'success',
            'data' => [
                'api_key',
                'has_api_key',
                'created_at'
            ]
        ]);
        
        // 2. 重置 API 密钥
        $oldApiKey = $user->api_key;
        
        $response = $this->actingAs($user, 'sanctum')
                         ->post('/api/v1/user/api-key/reset');
        
        $response->assertStatus(200);
        $response->assertJson([
            'success' => true,
            'message' => 'API key reset successfully',
        ]);
        
        $user->refresh();
        $this->assertNotEquals($oldApiKey, $user->api_key);
        $this->assertStringStartsWith('xb_', $user->api_key);
        
        // 3. 验证 API 密钥
        $response = $this->actingAs($user, 'sanctum')
                         ->post('/api/v1/user/api-key/validate', [
                             'api_key' => $user->api_key
                         ]);
        
        $response->assertStatus(200);
        $response->assertJson([
            'success' => true,
            'valid' => true,
        ]);
    }
    
    public function test_user_limits_management_flow()
    {
        $superAdmin = User::factory()->create([
            'is_super_admin' => true,
            'trust_level' => 4,
        ]);
        
        $regularUser = User::factory()->create([
            'trust_level' => 2,
        ]);
        
        // 1. 超级管理员查看用户组限制
        $response = $this->actingAs($superAdmin, 'sanctum')
                         ->get('/api/v1/admin/group-limits');
        
        $response->assertStatus(200);
        $response->assertJsonStructure([
            'success',
            'data' => [
                '*' => [
                    'trust_level',
                    'speed_limit_up',
                    'speed_limit_down',
                    'device_limit',
                    'connection_limit'
                ]
            ]
        ]);
        
        // 2. 设置个人限制
        $response = $this->actingAs($superAdmin, 'sanctum')
                         ->post("/api/v1/admin/users/{$regularUser->id}/limits", [
                             'speed_limit_up' => 300,
                             'speed_limit_down' => 600,
                             'device_limit' => 8,
                             'connection_limit' => 30,
                         ]);
        
        $response->assertStatus(200);
        $response->assertJson([
            'success' => true,
            'message' => 'User limits updated successfully',
        ]);
        
        // 3. 验证个人限制优先于组限制
        $limitService = app(LimitControlService::class);
        $effectiveLimits = $limitService->getEffectiveLimits($regularUser);
        
        $this->assertEquals(300, $effectiveLimits['speed_limit_up']);
        $this->assertEquals(600, $effectiveLimits['speed_limit_down']);
        $this->assertEquals(8, $effectiveLimits['device_limit']);
        
        // 4. 普通用户查看自己的限制
        $response = $this->actingAs($regularUser, 'sanctum')
                         ->get('/api/v1/user/limits');
        
        $response->assertStatus(200);
        $response->assertJsonPath('data.effective_limits.speed_limit_up', 300);
    }
    
    public function test_super_admin_permissions()
    {
        $superAdmin = User::factory()->create([
            'is_super_admin' => true,
        ]);
        
        $regularUser = User::factory()->create([
            'is_admin' => true, // 普通管理员
            'is_super_admin' => false,
        ]);
        
        // 1. 超级管理员可以访问管理功能
        $response = $this->actingAs($superAdmin, 'sanctum')
                         ->get('/api/v1/admin/group-limits');
        
        $response->assertStatus(200);
        
        // 2. 普通管理员无法访问超级管理员功能
        $response = $this->actingAs($regularUser, 'sanctum')
                         ->get('/api/v1/admin/group-limits');
        
        $response->assertStatus(403);
        $response->assertJson([
            'success' => false,
            'error' => 'Super administrator privileges required',
        ]);
        
        // 3. 未认证用户无法访问
        Auth::forgetGuards();
        $response = $this->get('/api/v1/admin/group-limits');
        
        $response->assertStatus(401);
    }
    
    public function test_oauth_token_refresh_flow()
    {
        $user = User::factory()->create([
            'linux_do_id' => '123',
            'oauth_provider' => 'linux_do',
            'oauth_refresh_token' => 'test_refresh_token',
            'oauth_expires_at' => now()->addHour(),
        ]);
        
        // Mock token refresh response
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
        
        // 验证令牌已更新
        $user->refresh();
        $this->assertEquals('new_access_token', $user->oauth_access_token);
        $this->assertEquals('new_refresh_token', $user->oauth_refresh_token);
    }
    
    public function test_user_info_sync_flow()
    {
        $user = User::factory()->create([
            'linux_do_id' => '123',
            'oauth_provider' => 'linux_do',
            'oauth_access_token' => 'test_access_token',
            'oauth_expires_at' => now()->addHour(),
            'trust_level' => 1,
        ]);
        
        // Mock user info response with updated trust level
        Http::fake([
            'connect.linux.do/api/user' => Http::response([
                'id' => 123,
                'username' => 'updateduser',
                'name' => 'Updated User',
                'avatar_template' => '/user_avatar/updateduser/{size}/123.png',
                'active' => true,
                'trust_level' => 3, // 信任等级提升
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
        
        // 验证用户信息已更新
        $user->refresh();
        $this->assertEquals('updateduser', $user->linux_do_username);
        $this->assertEquals(3, $user->trust_level);
    }
}
