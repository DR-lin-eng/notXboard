<?php

namespace Tests\Unit\Models;

use Tests\TestCase;
use App\Models\AuditRule;
use App\Models\ServerNode;
use Illuminate\Foundation\Testing\RefreshDatabase;

class AuditRuleTest extends TestCase
{
    use RefreshDatabase;

    /** @test */
    public function it_can_create_audit_rule()
    {
        $node = ServerNode::factory()->create();
        
        $rule = AuditRule::create([
            'node_id' => $node->id,
            'rule_type' => AuditRule::TYPE_DOMAIN,
            'rule_pattern' => '*.example.com',
            'action' => AuditRule::ACTION_BLOCK,
            'is_active' => true,
        ]);

        $this->assertDatabaseHas('audit_rules', [
            'node_id' => $node->id,
            'rule_type' => 'domain',
            'rule_pattern' => '*.example.com',
            'action' => 'block',
        ]);
    }

    /** @test */
    public function it_belongs_to_server_node()
    {
        $node = ServerNode::factory()->create();
        $rule = AuditRule::factory()->create(['node_id' => $node->id]);
        
        $this->assertInstanceOf(ServerNode::class, $rule->node);
        $this->assertEquals($node->id, $rule->node->id);
    }

    /** @test */
    public function it_can_match_domain_patterns()
    {
        $rule = AuditRule::factory()->create([
            'rule_type' => AuditRule::TYPE_DOMAIN,
            'rule_pattern' => '*.example.com',
            'is_active' => true,
        ]);
        
        $this->assertTrue($rule->matches('sub.example.com'));
        $this->assertTrue($rule->matches('test.example.com'));
        $this->assertFalse($rule->matches('example.org'));
        $this->assertFalse($rule->matches('notexample.com'));
    }

    /** @test */
    public function it_can_match_exact_domain()
    {
        $rule = AuditRule::factory()->create([
            'rule_type' => AuditRule::TYPE_DOMAIN,
            'rule_pattern' => 'example.com',
            'is_active' => true,
        ]);
        
        $this->assertTrue($rule->matches('example.com'));
        $this->assertFalse($rule->matches('sub.example.com'));
        $this->assertFalse($rule->matches('example.org'));
    }

    /** @test */
    public function it_can_match_protocol_patterns()
    {
        $rule = AuditRule::factory()->create([
            'rule_type' => AuditRule::TYPE_PROTOCOL,
            'rule_pattern' => 'HTTP',
            'is_active' => true,
        ]);
        
        $this->assertTrue($rule->matches('http'));
        $this->assertTrue($rule->matches('HTTP'));
        $this->assertFalse($rule->matches('https'));
        $this->assertFalse($rule->matches('ftp'));
    }

    /** @test */
    public function it_can_match_ip_patterns()
    {
        $rule = AuditRule::factory()->create([
            'rule_type' => AuditRule::TYPE_IP,
            'rule_pattern' => '192.168.1.*',
            'is_active' => true,
        ]);
        
        $this->assertTrue($rule->matches('192.168.1.1'));
        $this->assertTrue($rule->matches('192.168.1.255'));
        $this->assertFalse($rule->matches('192.168.2.1'));
        $this->assertFalse($rule->matches('10.0.0.1'));
    }

    /** @test */
    public function it_can_match_cidr_patterns()
    {
        $rule = AuditRule::factory()->create([
            'rule_type' => AuditRule::TYPE_IP,
            'rule_pattern' => '192.168.1.0/24',
            'is_active' => true,
        ]);
        
        $this->assertTrue($rule->matches('192.168.1.1'));
        $this->assertTrue($rule->matches('192.168.1.100'));
        $this->assertTrue($rule->matches('192.168.1.255'));
        $this->assertFalse($rule->matches('192.168.2.1'));
        $this->assertFalse($rule->matches('10.0.0.1'));
    }

    /** @test */
    public function it_can_match_ipv6_cidr_patterns()
    {
        $rule = AuditRule::factory()->create([
            'rule_type' => AuditRule::TYPE_IP,
            'rule_pattern' => '2001:db8::/32',
            'is_active' => true,
        ]);
        
        $this->assertTrue($rule->matches('2001:db8::1'));
        $this->assertTrue($rule->matches('2001:db8:1234::1'));
        $this->assertFalse($rule->matches('2001:db9::1'));
        $this->assertFalse($rule->matches('::1'));
    }

    /** @test */
    public function it_does_not_match_when_inactive()
    {
        $rule = AuditRule::factory()->create([
            'rule_type' => AuditRule::TYPE_DOMAIN,
            'rule_pattern' => 'example.com',
            'is_active' => false,
        ]);
        
        $this->assertFalse($rule->matches('example.com'));
    }

    /** @test */
    public function it_can_get_rule_description()
    {
        $domainRule = AuditRule::factory()->create([
            'rule_type' => AuditRule::TYPE_DOMAIN,
            'rule_pattern' => '*.example.com',
            'action' => AuditRule::ACTION_BLOCK,
        ]);
        
        $protocolRule = AuditRule::factory()->create([
            'rule_type' => AuditRule::TYPE_PROTOCOL,
            'rule_pattern' => 'HTTP',
            'action' => AuditRule::ACTION_ALLOW,
        ]);
        
        $ipRule = AuditRule::factory()->create([
            'rule_type' => AuditRule::TYPE_IP,
            'rule_pattern' => '192.168.1.0/24',
            'action' => AuditRule::ACTION_LOG,
        ]);
        
        $this->assertEquals('阻止 域名: *.example.com', $domainRule->getDescription());
        $this->assertEquals('允许 协议: HTTP', $protocolRule->getDescription());
        $this->assertEquals('记录 IP地址: 192.168.1.0/24', $ipRule->getDescription());
    }

    /** @test */
    public function it_has_rule_type_constants()
    {
        $this->assertEquals('domain', AuditRule::TYPE_DOMAIN);
        $this->assertEquals('protocol', AuditRule::TYPE_PROTOCOL);
        $this->assertEquals('ip', AuditRule::TYPE_IP);
    }

    /** @test */
    public function it_has_action_constants()
    {
        $this->assertEquals('block', AuditRule::ACTION_BLOCK);
        $this->assertEquals('allow', AuditRule::ACTION_ALLOW);
        $this->assertEquals('log', AuditRule::ACTION_LOG);
    }

    /** @test */
    public function it_casts_is_active_to_boolean()
    {
        $rule = AuditRule::factory()->create(['is_active' => 1]);
        
        $this->assertIsBool($rule->is_active);
        $this->assertTrue($rule->is_active);
        
        $rule->update(['is_active' => 0]);
        $this->assertFalse($rule->fresh()->is_active);
    }
}