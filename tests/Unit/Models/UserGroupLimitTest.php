<?php

namespace Tests\Unit\Models;

use Tests\TestCase;
use App\Models\UserGroupLimit;
use Illuminate\Foundation\Testing\RefreshDatabase;

class UserGroupLimitTest extends TestCase
{
    use RefreshDatabase;

    /** @test */
    public function it_can_create_user_group_limit()
    {
        $limit = UserGroupLimit::create([
            'trust_level' => 2,
            'speed_limit_up' => 100,
            'speed_limit_down' => 200,
            'device_limit' => 5,
            'connection_limit' => 50,
        ]);

        $this->assertDatabaseHas('user_group_limits', [
            'trust_level' => 2,
            'speed_limit_up' => 100,
            'speed_limit_down' => 200,
        ]);
    }

    /** @test */
    public function it_can_get_trust_level_name()
    {
        $newUser = UserGroupLimit::factory()->create(['trust_level' => UserGroupLimit::TRUST_LEVEL_NEW]);
        $basic = UserGroupLimit::factory()->create(['trust_level' => UserGroupLimit::TRUST_LEVEL_BASIC]);
        $member = UserGroupLimit::factory()->create(['trust_level' => UserGroupLimit::TRUST_LEVEL_MEMBER]);
        $regular = UserGroupLimit::factory()->create(['trust_level' => UserGroupLimit::TRUST_LEVEL_REGULAR]);
        $leader = UserGroupLimit::factory()->create(['trust_level' => UserGroupLimit::TRUST_LEVEL_LEADER]);
        
        $this->assertEquals('新用户', $newUser->getTrustLevelName());
        $this->assertEquals('基础用户', $basic->getTrustLevelName());
        $this->assertEquals('成员', $member->getTrustLevelName());
        $this->assertEquals('常规用户', $regular->getTrustLevelName());
        $this->assertEquals('领导者', $leader->getTrustLevelName());
    }

    /** @test */
    public function it_can_get_limits_by_trust_level()
    {
        $limit = UserGroupLimit::create([
            'trust_level' => 2,
            'speed_limit_up' => 100,
            'speed_limit_down' => 200,
        ]);
        
        $found = UserGroupLimit::getLimitsByTrustLevel(2);
        $notFound = UserGroupLimit::getLimitsByTrustLevel(3);
        
        $this->assertInstanceOf(UserGroupLimit::class, $found);
        $this->assertEquals($limit->id, $found->id);
        $this->assertNull($notFound);
    }

    /** @test */
    public function it_can_create_or_update_limit()
    {
        // 创建新限制
        $limit = UserGroupLimit::createOrUpdateLimit(2, [
            'speed_limit_up' => 100,
            'speed_limit_down' => 200,
        ]);
        
        $this->assertEquals(2, $limit->trust_level);
        $this->assertEquals(100, $limit->speed_limit_up);
        
        // 更新现有限制
        $updated = UserGroupLimit::createOrUpdateLimit(2, [
            'speed_limit_up' => 150,
            'device_limit' => 5,
        ]);
        
        $this->assertEquals($limit->id, $updated->id);
        $this->assertEquals(150, $updated->speed_limit_up);
        $this->assertEquals(5, $updated->device_limit);
    }

    /** @test */
    public function it_can_get_all_limits_ordered_by_trust_level()
    {
        UserGroupLimit::create(['trust_level' => 3, 'speed_limit_up' => 300]);
        UserGroupLimit::create(['trust_level' => 1, 'speed_limit_up' => 100]);
        UserGroupLimit::create(['trust_level' => 2, 'speed_limit_up' => 200]);
        
        $limits = UserGroupLimit::getAllLimits();
        
        $this->assertCount(3, $limits);
        $this->assertEquals(1, $limits->first()->trust_level);
        $this->assertEquals(3, $limits->last()->trust_level);
    }

    /** @test */
    public function it_has_default_limits_configuration()
    {
        $defaults = UserGroupLimit::getDefaultLimits();
        
        $this->assertIsArray($defaults);
        $this->assertArrayHasKey(UserGroupLimit::TRUST_LEVEL_NEW, $defaults);
        $this->assertArrayHasKey(UserGroupLimit::TRUST_LEVEL_LEADER, $defaults);
        
        // 检查新用户限制
        $newUserLimits = $defaults[UserGroupLimit::TRUST_LEVEL_NEW];
        $this->assertEquals(10, $newUserLimits['speed_limit_up']);
        $this->assertEquals(50, $newUserLimits['speed_limit_down']);
        
        // 检查领导者无限制
        $leaderLimits = $defaults[UserGroupLimit::TRUST_LEVEL_LEADER];
        $this->assertEquals(0, $leaderLimits['speed_limit_up']);
        $this->assertEquals(0, $leaderLimits['speed_limit_down']);
    }

    /** @test */
    public function it_can_initialize_default_limits()
    {
        UserGroupLimit::initializeDefaults();
        
        $this->assertDatabaseHas('user_group_limits', ['trust_level' => 0]);
        $this->assertDatabaseHas('user_group_limits', ['trust_level' => 1]);
        $this->assertDatabaseHas('user_group_limits', ['trust_level' => 2]);
        $this->assertDatabaseHas('user_group_limits', ['trust_level' => 3]);
        $this->assertDatabaseHas('user_group_limits', ['trust_level' => 4]);
        
        $newUserLimit = UserGroupLimit::where('trust_level', 0)->first();
        $this->assertEquals(10, $newUserLimit->speed_limit_up);
        $this->assertEquals(50, $newUserLimit->speed_limit_down);
    }

    /** @test */
    public function it_can_validate_limits()
    {
        $validLimit = UserGroupLimit::factory()->create([
            'trust_level' => 2,
            'speed_limit_up' => 100,
            'speed_limit_down' => 200,
            'device_limit' => 5,
            'connection_limit' => 50,
        ]);
        
        $this->assertTrue($validLimit->isValid());
        
        // 测试无效的信任等级
        $invalidLimit = UserGroupLimit::factory()->make(['trust_level' => -1]);
        $this->assertFalse($invalidLimit->isValid());
        
        $invalidLimit2 = UserGroupLimit::factory()->make(['trust_level' => 5]);
        $this->assertFalse($invalidLimit2->isValid());
        
        // 测试负数限制
        $invalidLimit3 = UserGroupLimit::factory()->make(['speed_limit_up' => -10]);
        $this->assertFalse($invalidLimit3->isValid());
    }

    /** @test */
    public function it_can_get_description()
    {
        $limit = UserGroupLimit::factory()->create([
            'speed_limit_up' => 100,
            'speed_limit_down' => 200,
            'device_limit' => 5,
            'connection_limit' => 50,
        ]);
        
        $description = $limit->getDescription();
        
        $this->assertStringContainsString('上传: 100Mbps', $description);
        $this->assertStringContainsString('下载: 200Mbps', $description);
        $this->assertStringContainsString('设备: 5个', $description);
        $this->assertStringContainsString('连接: 50个', $description);
    }

    /** @test */
    public function it_shows_no_limits_when_all_zero()
    {
        $limit = UserGroupLimit::factory()->create([
            'speed_limit_up' => 0,
            'speed_limit_down' => 0,
            'device_limit' => 0,
            'connection_limit' => 0,
        ]);
        
        $this->assertEquals('无限制', $limit->getDescription());
    }

    /** @test */
    public function it_casts_fields_to_integers()
    {
        $limit = UserGroupLimit::factory()->create([
            'trust_level' => '2',
            'speed_limit_up' => '100',
            'device_limit' => '5',
        ]);
        
        $this->assertIsInt($limit->trust_level);
        $this->assertIsInt($limit->speed_limit_up);
        $this->assertIsInt($limit->device_limit);
    }

    /** @test */
    public function it_has_trust_level_constants()
    {
        $this->assertEquals(0, UserGroupLimit::TRUST_LEVEL_NEW);
        $this->assertEquals(1, UserGroupLimit::TRUST_LEVEL_BASIC);
        $this->assertEquals(2, UserGroupLimit::TRUST_LEVEL_MEMBER);
        $this->assertEquals(3, UserGroupLimit::TRUST_LEVEL_REGULAR);
        $this->assertEquals(4, UserGroupLimit::TRUST_LEVEL_LEADER);
    }
}