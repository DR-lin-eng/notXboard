<?php

namespace Tests\Unit\Models;

use Tests\TestCase;
use App\Models\User;
use App\Models\UserIndividualLimit;
use App\Models\UserGroupLimit;
use Illuminate\Foundation\Testing\RefreshDatabase;

class UserIndividualLimitTest extends TestCase
{
    use RefreshDatabase;

    protected function setUp(): void
    {
        parent::setUp();
        
        // 创建默认的用户组限制
        UserGroupLimit::create([
            'trust_level' => 2,
            'speed_limit_up' => 50,
            'speed_limit_down' => 100,
            'device_limit' => 3,
            'connection_limit' => 30,
        ]);
    }

    /** @test */
    public function it_can_create_individual_limit()
    {
        $user = User::factory()->create();
        
        $limit = UserIndividualLimit::create([
            'user_id' => $user->id,
            'speed_limit_up' => 200,
            'speed_limit_down' => 500,
            'device_limit' => 10,
            'connection_limit' => 100,
        ]);

        $this->assertDatabaseHas('user_individual_limits', [
            'user_id' => $user->id,
            'speed_limit_up' => 200,
        ]);
    }

    /** @test */
    public function it_belongs_to_user()
    {
        $user = User::factory()->create();
        $limit = UserIndividualLimit::factory()->create(['user_id' => $user->id]);
        
        $this->assertInstanceOf(User::class, $limit->user);
        $this->assertEquals($user->id, $limit->user->id);
    }

    /** @test */
    public function it_can_get_user_limit()
    {
        $user = User::factory()->create();
        $limit = UserIndividualLimit::create(['user_id' => $user->id]);
        
        $found = UserIndividualLimit::getUserLimit($user->id);
        $notFound = UserIndividualLimit::getUserLimit(999);
        
        $this->assertInstanceOf(UserIndividualLimit::class, $found);
        $this->assertEquals($limit->id, $found->id);
        $this->assertNull($notFound);
    }

    /** @test */
    public function it_can_create_or_update_limit()
    {
        $user = User::factory()->create();
        
        // 创建新限制
        $limit = UserIndividualLimit::createOrUpdateLimit($user->id, [
            'speed_limit_up' => 100,
        ]);
        
        $this->assertEquals($user->id, $limit->user_id);
        $this->assertEquals(100, $limit->speed_limit_up);
        
        // 更新现有限制
        $updated = UserIndividualLimit::createOrUpdateLimit($user->id, [
            'speed_limit_up' => 200,
            'device_limit' => 5,
        ]);
        
        $this->assertEquals($limit->id, $updated->id);
        $this->assertEquals(200, $updated->speed_limit_up);
        $this->assertEquals(5, $updated->device_limit);
    }

    /** @test */
    public function it_can_get_effective_limits_using_group_defaults()
    {
        $user = User::factory()->create(['trust_level' => 2]);
        $limit = UserIndividualLimit::create([
            'user_id' => $user->id,
            'speed_limit_up' => 0, // 使用组配置
            'speed_limit_down' => 200, // 个人配置
            'device_limit' => 0, // 使用组配置
            'connection_limit' => 50, // 个人配置
        ]);
        
        $effectiveLimits = $limit->getEffectiveLimits();
        
        $this->assertEquals(50, $effectiveLimits['speed_limit_up']); // 来自组
        $this->assertEquals(200, $effectiveLimits['speed_limit_down']); // 个人
        $this->assertEquals(3, $effectiveLimits['device_limit']); // 来自组
        $this->assertEquals(50, $effectiveLimits['connection_limit']); // 个人
    }

    /** @test */
    public function it_can_check_if_has_custom_limits()
    {
        $user = User::factory()->create();
        
        $noCustomLimits = UserIndividualLimit::create([
            'user_id' => $user->id,
            'speed_limit_up' => 0,
            'speed_limit_down' => 0,
            'device_limit' => 0,
            'connection_limit' => 0,
        ]);
        
        $hasCustomLimits = UserIndividualLimit::create([
            'user_id' => User::factory()->create()->id,
            'speed_limit_up' => 100,
            'speed_limit_down' => 0,
        ]);
        
        $this->assertFalse($noCustomLimits->hasCustomLimits());
        $this->assertTrue($hasCustomLimits->hasCustomLimits());
    }

    /** @test */
    public function it_can_reset_to_group_limits()
    {
        $user = User::factory()->create();
        $limit = UserIndividualLimit::create([
            'user_id' => $user->id,
            'speed_limit_up' => 100,
            'speed_limit_down' => 200,
            'device_limit' => 5,
            'connection_limit' => 50,
        ]);
        
        $limit->resetToGroupLimits();
        
        $fresh = $limit->fresh();
        $this->assertEquals(0, $fresh->speed_limit_up);
        $this->assertEquals(0, $fresh->speed_limit_down);
        $this->assertEquals(0, $fresh->device_limit);
        $this->assertEquals(0, $fresh->connection_limit);
    }

    /** @test */
    public function it_can_get_description_with_source_indication()
    {
        $user = User::factory()->create(['trust_level' => 2]);
        $limit = UserIndividualLimit::create([
            'user_id' => $user->id,
            'speed_limit_up' => 100, // 个人配置
            'speed_limit_down' => 0, // 使用组配置
            'device_limit' => 5, // 个人配置
            'connection_limit' => 0, // 使用组配置
        ]);
        
        $description = $limit->getDescription();
        
        $this->assertStringContainsString('上传: 100Mbps (个人)', $description);
        $this->assertStringContainsString('下载: 100Mbps (组)', $description);
        $this->assertStringContainsString('设备: 5个 (个人)', $description);
        $this->assertStringContainsString('连接: 30个 (组)', $description);
    }

    /** @test */
    public function it_can_validate_limits()
    {
        $user = User::factory()->create();
        
        $validLimit = UserIndividualLimit::create([
            'user_id' => $user->id,
            'speed_limit_up' => 100,
            'speed_limit_down' => 200,
        ]);
        
        $this->assertTrue($validLimit->isValid());
        
        $invalidLimit = UserIndividualLimit::make([
            'user_id' => $user->id,
            'speed_limit_up' => -10,
        ]);
        
        $this->assertFalse($invalidLimit->isValid());
    }

    /** @test */
    public function it_can_apply_limits()
    {
        $user = User::factory()->create();
        $limit = UserIndividualLimit::create(['user_id' => $user->id]);
        
        $limit->applyLimits([
            'speed_limit_up' => 150,
            'device_limit' => 8,
            'invalid_field' => 999, // 应该被忽略
        ]);
        
        $fresh = $limit->fresh();
        $this->assertEquals(150, $fresh->speed_limit_up);
        $this->assertEquals(8, $fresh->device_limit);
        $this->assertNull($fresh->getAttribute('invalid_field'));
    }

    /** @test */
    public function it_can_batch_set_limits()
    {
        $users = User::factory()->count(3)->create();
        $userIds = $users->pluck('id')->toArray();
        
        $count = UserIndividualLimit::batchSetLimits($userIds, [
            'speed_limit_up' => 100,
            'device_limit' => 5,
        ]);
        
        $this->assertEquals(3, $count);
        
        foreach ($users as $user) {
            $limit = UserIndividualLimit::getUserLimit($user->id);
            $this->assertEquals(100, $limit->speed_limit_up);
            $this->assertEquals(5, $limit->device_limit);
        }
    }

    /** @test */
    public function it_can_cleanup_empty_limits()
    {
        $user1 = User::factory()->create();
        $user2 = User::factory()->create();
        $user3 = User::factory()->create();
        
        // 创建空限制（所有值都为0）
        UserIndividualLimit::create([
            'user_id' => $user1->id,
            'speed_limit_up' => 0,
            'speed_limit_down' => 0,
            'device_limit' => 0,
            'connection_limit' => 0,
        ]);
        
        // 创建有效限制 
       UserIndividualLimit::create([
            'user_id' => $user2->id,
            'speed_limit_up' => 100,
        ]);
        
        // 创建另一个空限制
        UserIndividualLimit::create([
            'user_id' => $user3->id,
            'speed_limit_up' => 0,
            'speed_limit_down' => 0,
            'device_limit' => 0,
            'connection_limit' => 0,
        ]);
        
        $deletedCount = UserIndividualLimit::cleanupEmptyLimits();
        
        $this->assertEquals(2, $deletedCount);
        $this->assertNull(UserIndividualLimit::getUserLimit($user1->id));
        $this->assertNotNull(UserIndividualLimit::getUserLimit($user2->id));
        $this->assertNull(UserIndividualLimit::getUserLimit($user3->id));
    }

    /** @test */
    public function it_casts_fields_to_integers()
    {
        $user = User::factory()->create();
        $limit = UserIndividualLimit::create([
            'user_id' => $user->id,
            'speed_limit_up' => '100',
            'device_limit' => '5',
        ]);
        
        $this->assertIsInt($limit->user_id);
        $this->assertIsInt($limit->speed_limit_up);
        $this->assertIsInt($limit->device_limit);
    }
}
