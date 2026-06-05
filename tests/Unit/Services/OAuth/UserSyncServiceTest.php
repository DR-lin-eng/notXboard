<?php

namespace Tests\Unit\Services\OAuth;

use App\Services\OAuth\UserSyncService;
use App\Services\OAuth\LinuxDoOAuthService;
use App\Models\User;
use App\Models\UserGroupLimit;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\DB;
use Tests\TestCase;

class UserSyncServiceTest extends TestCase
{
    use RefreshDatabase;
    
    protected UserSyncService $service;
    
    protected function setUp(): void
    {
        parent::setUp();
        $this->service = new UserSyncService();
    }
    
    public function test_sync_from_linux_do_creates_new_user()
    {
        $linuxDoUserData = [
            'id' => 123,
            'username' => 'testuser',
            'name' => 'Test User',
            'avatar_template' => '/user_avatar/testuser/{size}/123.png',
            'active' => true,
            'trust_level' => 2,
            'silenced' => false,
            'external_ids' => ['github' => 456],
        ];
        
        $tokenData = [
            'access_token' => 'test_access_token',
            'refresh_token' => 'test_refresh_token',
            'expires_in' => 3600,
        ];
        
        $user = $this->service->syncFromLinuxDo($linuxDoUserData, $tokenData);
        
        $this->assertInstanceOf(User::class, $user);
        $this->assertEquals('123', $user->linux_do_id);
        $this->assertEquals('testuser', $user->linux_do_username);
        $this->assertEquals('Test User', $user->linux_do_name);
        $this->assertEquals(2, $user->trust_level);
        $this->assertEquals('linux_do', $user->oauth_provider);
        $this->assertNotNull($user->api_key);
        $this->assertStringStartsWith('xb_', $user->api_key);
    }
    
    public function test_sync_from_linux_do_updates_existing_user()
    {
        // 创建现有用户
        $existingUser = User::create([
            'email' => 'test@example.com',
            'password' => bcrypt('password'),
            'token' => 'existing_token',
            'uuid' => 'existing-uuid',
            'linux_do_id' => '123',
            'linux_do_username' => 'oldusername',
            'trust_level' => 1,
            'oauth_provider' => 'linux_do',
        ]);
        
        $linuxDoUserData = [
            'id' => 123,
            'username' => 'newusername',
            'name' => 'Updated User',
            'avatar_template' => '/user_avatar/newusername/{size}/123.png',
            'active' => true,
            'trust_level' => 3, // 信任等级变化
            'silenced' => false,
            'external_ids' => [],
        ];
        
        $tokenData = [
            'access_token' => 'new_access_token',
            'refresh_token' => 'new_refresh_token',
            'expires_in' => 3600,
        ];
        
        $user = $this->service->syncFromLinuxDo($linuxDoUserData, $tokenData);
        
        $this->assertEquals($existingUser->id, $user->id);
        $this->assertEquals('newusername', $user->linux_do_username);
        $this->assertEquals('Updated User', $user->linux_do_name);
        $this->assertEquals(3, $user->trust_level);
        $this->assertEquals('new_access_token', $user->oauth_access_token);
    }
    
    public function test_update_user_group_applies_group_limits()
    {
        // 创建用户组限制
        UserGroupLimit::create([
            'trust_level' => 2,
            'speed_limit_up' => 100,
            'speed_limit_down' => 200,
            'device_limit' => 5,
            'connection_limit' => 10,
        ]);
        
        $user = User::create([
            'email' => 'test@example.com',
            'password' => bcrypt('password'),
            'token' => 'test_token',
            'uuid' => 'test-uuid',
            'trust_level' => 2,
        ]);
        
        $this->service->updateUserGroup($user);
        
        $user->refresh();
        $this->assertEquals(5, $user->device_limit);
    }
    
    public function test_should_update_user_info_returns_true_for_old_data()
    {
        $user = User::create([
            'email' => 'test@example.com',
            'password' => bcrypt('password'),
            'token' => 'test_token',
            'uuid' => 'test-uuid',
            'linux_do_id' => '123',
            'oauth_provider' => 'linux_do',
            'updated_at' => now()->subDays(2), // 2天前更新
        ]);
        
        $result = $this->service->shouldUpdateUserInfo($user);
        
        $this->assertTrue($result);
    }
    
    public function test_should_update_user_info_returns_false_for_recent_data()
    {
        $user = User::create([
            'email' => 'test@example.com',
            'password' => bcrypt('password'),
            'token' => 'test_token',
            'uuid' => 'test-uuid',
            'linux_do_id' => '123',
            'oauth_provider' => 'linux_do',
            'updated_at' => now()->subHours(12), // 12小时前更新
        ]);
        
        $result = $this->service->shouldUpdateUserInfo($user);
        
        $this->assertFalse($result);
    }
    
    public function test_should_update_user_info_returns_false_for_non_linux_do_user()
    {
        $user = User::create([
            'email' => 'test@example.com',
            'password' => bcrypt('password'),
            'token' => 'test_token',
            'uuid' => 'test-uuid',
            'updated_at' => now()->subDays(2),
        ]);
        
        $result = $this->service->shouldUpdateUserInfo($user);
        
        $this->assertFalse($result);
    }
    
    public function test_generates_unique_api_key()
    {
        $linuxDoUserData = [
            'id' => 123,
            'username' => 'testuser',
            'name' => 'Test User',
            'avatar_template' => '/user_avatar/testuser/{size}/123.png',
            'active' => true,
            'trust_level' => 2,
            'silenced' => false,
            'external_ids' => [],
        ];
        
        $tokenData = [
            'access_token' => 'test_access_token',
            'refresh_token' => 'test_refresh_token',
            'expires_in' => 3600,
        ];
        
        // 创建两个用户，确保 API 密钥唯一
        $user1 = $this->service->syncFromLinuxDo($linuxDoUserData, $tokenData);
        
        $linuxDoUserData['id'] = 124;
        $linuxDoUserData['username'] = 'testuser2';
        $user2 = $this->service->syncFromLinuxDo($linuxDoUserData, $tokenData);
        
        $this->assertNotEquals($user1->api_key, $user2->api_key);
        $this->assertStringStartsWith('xb_', $user1->api_key);
        $this->assertStringStartsWith('xb_', $user2->api_key);
    }
    
    public function test_handles_duplicate_email()
    {
        // 创建现有用户
        User::create([
            'email' => 'testuser@linux.do',
            'password' => bcrypt('password'),
            'token' => 'existing_token',
            'uuid' => 'existing-uuid',
        ]);
        
        $linuxDoUserData = [
            'id' => 123,
            'username' => 'testuser',
            'name' => 'Test User',
            'avatar_template' => '/user_avatar/testuser/{size}/123.png',
            'active' => true,
            'trust_level' => 2,
            'silenced' => false,
            'external_ids' => [],
        ];
        
        $tokenData = [
            'access_token' => 'test_access_token',
            'refresh_token' => 'test_refresh_token',
            'expires_in' => 3600,
        ];
        
        $user = $this->service->syncFromLinuxDo($linuxDoUserData, $tokenData);
        
        // 应该生成不同的邮箱
        $this->assertNotEquals('testuser@linux.do', $user->email);
        $this->assertStringContainsString('+1@', $user->email);
    }
}
