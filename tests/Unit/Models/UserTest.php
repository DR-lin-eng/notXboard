<?php

namespace Tests\Unit\Models;

use Tests\TestCase;
use App\Models\User;
use App\Models\ServerNode;
use App\Models\UserGroupLimit;
use App\Models\UserIndividualLimit;
use Illuminate\Foundation\Testing\RefreshDatabase;

class UserTest extends TestCase
{
    use RefreshDatabase;

    protected function setUp(): void
    {
        parent::setUp();
        
        // 创建默认的用户组限制
        UserGroupLimit::factory()->create([
            'trust_level' => 0,
            'speed_limit_up' => 10,
            'speed_limit_down' => 50,
            'device_limit' => 2,
            'connection_limit' => 10,
        ]);
    }

    /** @test */
    public function it_can_create_user_with_oauth_fields()
    {
        $user = User::factory()->create([
            'linux_do_id' => '12345',
            'linux_do_username' => 'testuser',
            'linux_do_name' => 'Test User',
            'trust_level' => 2,
            'oauth_provider' => 'linux_do',
        ]);

        $this->assertDatabaseHas('v2_user', [
            'linux_do_id' => '12345',
            'linux_do_username' => 'testuser',
            'trust_level' => 2,
        ]);
    }

    /** @test */
    public function it_can_check_if_user_is_linux_do_user()
    {
        $linuxDoUser = User::factory()->create([
            'linux_do_id' => '12345',
            'oauth_provider' => 'linux_do',
        ]);

        $regularUser = User::factory()->create([
            'linux_do_id' => null,
            'oauth_provider' => null,
        ]);

        $this->assertTrue($linuxDoUser->isLinuxDoUser());
        $this->assertFalse($regularUser->isLinuxDoUser());
    }

    /** @test */
    public function it_can_get_user_group_by_trust_level()
    {
        $user = User::factory()->create(['trust_level' => 0]);
        $this->assertEquals('new_user', $user->getUserGroup());

        $user->trust_level = 1;
        $this->assertEquals('basic_user', $user->getUserGroup());

        $user->trust_level = 2;
        $this->assertEquals('member', $user->getUserGroup());

        $user->trust_level = 3;
        $this->assertEquals('regular', $user->getUserGroup());

        $user->trust_level = 4;
        $this->assertEquals('leader', $user->getUserGroup());
    }

    /** @test */
    public function it_can_generate_unique_api_key()
    {
        $user = User::factory()->create();
        
        $apiKey = $user->generateApiKey();
        
        $this->assertStringStartsWith('xb_', $apiKey);
        $this->assertEquals(63, strlen($apiKey)); // 'xb_' + 60 hex chars
        $this->assertEquals($apiKey, $user->fresh()->api_key);
    }

    /** @test */
    public function it_generates_unique_api_keys_for_different_users()
    {
        $user1 = User::factory()->create();
        $user2 = User::factory()->create();
        
        $apiKey1 = $user1->generateApiKey();
        $apiKey2 = $user2->generateApiKey();
        
        $this->assertNotEquals($apiKey1, $apiKey2);
    }

    /** @test */
    public function it_can_check_oauth_token_expiration()
    {
        $expiredUser = User::factory()->create([
            'oauth_expires_at' => now()->subHour(),
        ]);

        $validUser = User::factory()->create([
            'oauth_expires_at' => now()->addHour(),
        ]);

        $noTokenUser = User::factory()->create([
            'oauth_expires_at' => null,
        ]);

        $this->assertTrue($expiredUser->isOAuthTokenExpired());
        $this->assertFalse($validUser->isOAuthTokenExpired());
        $this->assertFalse($noTokenUser->isOAuthTokenExpired());
    }

    /** @test */
    public function it_can_get_effective_limits_from_group()
    {
        $user = User::factory()->create(['trust_level' => 0]);
        
        $limits = $user->getEffectiveLimits();
        
        $this->assertEquals(10, $limits['speed_limit_up']);
        $this->assertEquals(50, $limits['speed_limit_down']);
        $this->assertEquals(2, $limits['device_limit']);
        $this->assertEquals(10, $limits['connection_limit']);
    }

    /** @test */
    public function it_can_get_effective_limits_with_individual_override()
    {
        $user = User::factory()->create(['trust_level' => 0]);
        
        UserIndividualLimit::create([
            'user_id' => $user->id,
            'speed_limit_up' => 100,
            'speed_limit_down' => 200,
            'device_limit' => 5,
            'connection_limit' => 50,
        ]);
        
        $limits = $user->getEffectiveLimits();
        
        $this->assertEquals(100, $limits['speed_limit_up']);
        $this->assertEquals(200, $limits['speed_limit_down']);
        $this->assertEquals(5, $limits['device_limit']);
        $this->assertEquals(50, $limits['connection_limit']);
    }

    /** @test */
    public function it_can_check_if_user_should_sync_info()
    {
        // Linux DO 用户，最近更新过
        $recentUser = User::factory()->create([
            'linux_do_id' => '12345',
            'oauth_provider' => 'linux_do',
            'updated_at' => now()->subHours(12),
        ]);

        // Linux DO 用户，很久没更新
        $oldUser = User::factory()->create([
            'linux_do_id' => '12346',
            'oauth_provider' => 'linux_do',
            'updated_at' => now()->subDays(2),
        ]);

        // 非 Linux DO 用户
        $regularUser = User::factory()->create([
            'linux_do_id' => null,
            'oauth_provider' => null,
        ]);

        $this->assertFalse($recentUser->shouldSyncUserInfo());
        $this->assertTrue($oldUser->shouldSyncUserInfo());
        $this->assertFalse($regularUser->shouldSyncUserInfo());
    }

    /** @test */
    public function it_has_server_nodes_relationship()
    {
        $user = User::factory()->create();
        $node = ServerNode::factory()->create(['user_id' => $user->id]);
        
        $this->assertTrue($user->serverNodes->contains($node));
        $this->assertEquals($user->id, $node->user_id);
    }

    /** @test */
    public function it_has_individual_limit_relationship()
    {
        $user = User::factory()->create();
        $limit = UserIndividualLimit::create([
            'user_id' => $user->id,
            'speed_limit_up' => 100,
        ]);
        
        $this->assertInstanceOf(UserIndividualLimit::class, $user->individualLimit);
        $this->assertEquals($limit->id, $user->individualLimit->id);
    }

    /** @test */
    public function it_hides_sensitive_oauth_fields()
    {
        $user = User::factory()->create([
            'oauth_access_token' => 'secret_token',
            'oauth_refresh_token' => 'secret_refresh',
        ]);
        
        $array = $user->toArray();
        
        $this->assertArrayNotHasKey('oauth_access_token', $array);
        $this->assertArrayNotHasKey('oauth_refresh_token', $array);
        $this->assertArrayNotHasKey('password', $array);
    }

    /** @test */
    public function it_casts_oauth_fields_correctly()
    {
        $user = User::factory()->create([
            'external_ids' => ['github' => 123, 'discord' => 456],
            'oauth_expires_at' => '2024-12-31 23:59:59',
            'is_super_admin' => true,
            'is_silenced' => false,
        ]);
        
        $this->assertIsArray($user->external_ids);
        $this->assertInstanceOf(\Illuminate\Support\Carbon::class, $user->oauth_expires_at);
        $this->assertIsBool($user->is_super_admin);
        $this->assertIsBool($user->is_silenced);
        $this->assertTrue($user->is_super_admin);
        $this->assertFalse($user->is_silenced);
    }
}
