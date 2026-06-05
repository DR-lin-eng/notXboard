<?php

namespace Database\Factories;

use App\Models\AuditRule;
use App\Models\ServerNode;
use Illuminate\Database\Eloquent\Factories\Factory;

class AuditRuleFactory extends Factory
{
    protected $model = AuditRule::class;

    public function definition(): array
    {
        $ruleType = $this->faker->randomElement([
            AuditRule::TYPE_DOMAIN,
            AuditRule::TYPE_PROTOCOL,
            AuditRule::TYPE_IP,
        ]);

        $pattern = match($ruleType) {
            AuditRule::TYPE_DOMAIN => $this->faker->randomElement([
                '*.example.com',
                'blocked-site.com',
                '*.ads.com',
            ]),
            AuditRule::TYPE_PROTOCOL => $this->faker->randomElement([
                'HTTP',
                'HTTPS',
                'FTP',
                'SMTP',
            ]),
            AuditRule::TYPE_IP => $this->faker->randomElement([
                '192.168.1.0/24',
                '10.0.0.*',
                '172.16.0.1',
            ]),
        };

        return [
            'node_id' => ServerNode::factory(),
            'rule_type' => $ruleType,
            'rule_pattern' => $pattern,
            'action' => $this->faker->randomElement([
                AuditRule::ACTION_BLOCK,
                AuditRule::ACTION_ALLOW,
                AuditRule::ACTION_LOG,
            ]),
            'is_active' => $this->faker->boolean(80), // 80% chance of being active
        ];
    }

    public function domain(): static
    {
        return $this->state(fn (array $attributes) => [
            'rule_type' => AuditRule::TYPE_DOMAIN,
            'rule_pattern' => '*.example.com',
        ]);
    }

    public function protocol(): static
    {
        return $this->state(fn (array $attributes) => [
            'rule_type' => AuditRule::TYPE_PROTOCOL,
            'rule_pattern' => 'HTTP',
        ]);
    }

    public function ip(): static
    {
        return $this->state(fn (array $attributes) => [
            'rule_type' => AuditRule::TYPE_IP,
            'rule_pattern' => '192.168.1.0/24',
        ]);
    }

    public function block(): static
    {
        return $this->state(fn (array $attributes) => [
            'action' => AuditRule::ACTION_BLOCK,
        ]);
    }

    public function allow(): static
    {
        return $this->state(fn (array $attributes) => [
            'action' => AuditRule::ACTION_ALLOW,
        ]);
    }

    public function active(): static
    {
        return $this->state(fn (array $attributes) => [
            'is_active' => true,
        ]);
    }

    public function inactive(): static
    {
        return $this->state(fn (array $attributes) => [
            'is_active' => false,
        ]);
    }
}