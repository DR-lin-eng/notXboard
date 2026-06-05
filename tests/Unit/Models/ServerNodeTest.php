<?php

namespace Tests\Unit\Models;

use Tests\TestCase;
use App\Models\User;
use App\Models\ServerNode;
use App\Models\AuditRule;
use App\Models\UserOnlineSession;
use Illuminate\Foundation\Testing\RefreshDatabase;

class ServerNodeTest extends TestCase
{
    use RefreshDatabase;

    /** @test */
    public function it_can_create_server_node_with_all_fields()
    {
        $user = User::factory()->create();
        
        $node = ServerNode::create([
            'user_id' => $user->id,
            'name' => 'Test Node',
            'host' => 'example.com',
            'port' => 443,
            'protocol' => ServerNode::PROTOCOL_VMESS,
            'settings' => ['encryption' => 'auto'],
            'traffic_limit' => 1000000,
            'status' => ServerNode::STATUS_ACTIVE,
            'v2bx_node_id' => 1,
            'device_limit' => 5,
            'connection_limit' => 100,
        ]);

        $this->assertDatabaseHas('server_nodes', [
            'name' => 'Test Node',
            'host' => 'example.com',
            'protocol' => 'vmess',
            'status' => 'active',
        ]);
    }

    /** @test */
    public function it_belongs_to_user()
    {
        $user = User::factory()->create();
        $node = ServerNode::factory()->create(['user_id' => $user->id]);
        
        $this->assertInstanceOf(User::class, $node->owner);
        $this->assertEquals($user->id, $node->owner->id);
    }

    /** @test */
    public function it_has_audit_rules_relationship()
    {
        $node = ServerNode::factory()->create();
        $rule = AuditRule::factory()->create(['node_id' => $node->id]);
        
        $this->assertTrue($node->auditRules->contains($rule));
    }

    /** @test */
    public function it_has_online_sessions_relationship()
    {
        $user = User::factory()->create();
        $node = ServerNode::factory()->create();
        $session = UserOnlineSession::factory()->create([
            'user_id' => $user->id,
            'node_id' => $node->id,
        ]);
        
        $this->assertTrue($node->onlineSessions->contains($session));
    }

    /** @test */
    public function it_can_check_if_node_is_active()
    {
        $activeNode = ServerNode::factory()->create(['status' => ServerNode::STATUS_ACTIVE]);
        $inactiveNode = ServerNode::factory()->create(['status' => ServerNode::STATUS_INACTIVE]);
        
        $this->assertTrue($activeNode->isActive());
        $this->assertFalse($inactiveNode->isActive());
    }

    /** @test */
    public function it_can_check_traffic_exceeded()
    {
        $node = ServerNode::factory()->create([
            'traffic_limit' => 1000,
            'traffic_used' => 1200,
        ]);
        
        $nodeWithinLimit = ServerNode::factory()->create([
            'traffic_limit' => 1000,
            'traffic_used' => 800,
        ]);
        
        $unlimitedNode = ServerNode::factory()->create([
            'traffic_limit' => 0,
            'traffic_used' => 9999999,
        ]);
        
        $this->assertTrue($node->isTrafficExceeded());
        $this->assertFalse($nodeWithinLimit->isTrafficExceeded());
        $this->assertFalse($unlimitedNode->isTrafficExceeded());
    }

    /** @test */
    public function it_can_calculate_traffic_usage_percentage()
    {
        $node = ServerNode::factory()->create([
            'traffic_limit' => 1000,
            'traffic_used' => 250,
        ]);
        
        $this->assertEquals(25.0, $node->getTrafficUsagePercentage());
    }

    /** @test */
    public function it_can_calculate_remaining_traffic()
    {
        $node = ServerNode::factory()->create([
            'traffic_limit' => 1000,
            'traffic_used' => 300,
        ]);
        
        $this->assertEquals(700, $node->getRemainingTraffic());
    }

    /** @test */
    public function it_can_check_user_access_permission()
    {
        $owner = User::factory()->create();
        $user = User::factory()->create(['trust_level' => 2]);
        $node = ServerNode::factory()->create([
            'user_id' => $owner->id,
            'access_control' => ['min_trust_level' => 1],
        ]);
        
        // 节点所有者总是有访问权限
        $this->assertTrue($node->canUserAccess($owner));
        
        // 满足信任等级要求的用户有访问权限
        $this->assertTrue($node->canUserAccess($user));
        
        // 不满足信任等级要求的用户没有访问权限
        $lowTrustUser = User::factory()->create(['trust_level' => 0]);
        $this->assertFalse($node->canUserAccess($lowTrustUser));
    }

    /** @test */
    public function it_can_update_traffic_usage()
    {
        $node = ServerNode::factory()->create(['traffic_used' => 100]);
        
        $node->updateTrafficUsage(50, 30);
        
        $this->assertEquals(180, $node->fresh()->traffic_used);
    }

    /** @test */
    public function it_can_get_online_user_count()
    {
        $node = ServerNode::factory()->create();
        $user1 = User::factory()->create();
        $user2 = User::factory()->create();
        
        // 创建活跃会话
        UserOnlineSession::factory()->create([
            'user_id' => $user1->id,
            'node_id' => $node->id,
            'last_activity' => now()->subMinutes(2),
        ]);
        
        UserOnlineSession::factory()->create([
            'user_id' => $user2->id,
            'node_id' => $node->id,
            'last_activity' => now()->subMinutes(3),
        ]);
        
        // 创建过期会话
        UserOnlineSession::factory()->create([
            'user_id' => $user1->id,
            'node_id' => $node->id,
            'last_activity' => now()->subMinutes(10),
        ]);
        
        $this->assertEquals(2, $node->getOnlineUserCount());
    }

    /** @test */
    public function it_can_get_v2bx_config()
    {
        $node = ServerNode::factory()->create([
            'name' => 'Test Node',
            'host' => 'example.com',
            'port' => 443,
            'protocol' => 'vmess',
            'settings' => ['encryption' => 'auto'],
            'v2bx_node_id' => 123,
            'v2bx_config' => ['custom' => 'value'],
        ]);
        
        $config = $node->getV2bXConfig();
        
        $this->assertEquals(123, $config['id']);
        $this->assertEquals('Test Node', $config['name']);
        $this->assertEquals('example.com', $config['host']);
        $this->assertEquals(443, $config['port']);
        $this->assertEquals('vmess', $config['protocol']);
        $this->assertEquals(['encryption' => 'auto'], $config['settings']);
        $this->assertEquals('value', $config['custom']);
    }

    /** @test */
    public function it_casts_json_fields_correctly()
    {
        $node = ServerNode::factory()->create([
            'settings' => ['encryption' => 'auto', 'network' => 'ws'],
            'access_control' => ['min_trust_level' => 2],
            'v2bx_config' => ['custom_field' => 'value'],
        ]);
        
        $this->assertIsArray($node->settings);
        $this->assertIsArray($node->access_control);
        $this->assertIsArray($node->v2bx_config);
        $this->assertEquals('auto', $node->settings['encryption']);
        $this->assertEquals(2, $node->access_control['min_trust_level']);
    }

    /** @test */
    public function it_has_protocol_constants()
    {
        $this->assertEquals('vmess', ServerNode::PROTOCOL_VMESS);
        $this->assertEquals('vless', ServerNode::PROTOCOL_VLESS);
        $this->assertEquals('trojan', ServerNode::PROTOCOL_TROJAN);
        $this->assertEquals('shadowsocks', ServerNode::PROTOCOL_SHADOWSOCKS);
        $this->assertEquals('hysteria', ServerNode::PROTOCOL_HYSTERIA);
        $this->assertEquals('hysteria2', ServerNode::PROTOCOL_HYSTERIA2);
    }

    /** @test */
    public function it_has_status_constants()
    {
        $this->assertEquals('active', ServerNode::STATUS_ACTIVE);
        $this->assertEquals('inactive', ServerNode::STATUS_INACTIVE);
        $this->assertEquals('maintenance', ServerNode::STATUS_MAINTENANCE);
        $this->assertEquals('deploying', ServerNode::STATUS_DEPLOYING);
    }
}