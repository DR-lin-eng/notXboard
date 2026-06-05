<?php

namespace Database\Factories;

use App\Models\ServerNode;
use App\Models\User;
use Illuminate\Database\Eloquent\Factories\Factory;

class ServerNodeFactory extends Factory
{
    protected $model = ServerNode::class;

    public function definition(): array
    {
        return [
            'user_id' => User::factory(),
            'name' => $this->faker->words(2, true) . ' Node',
            'host' => $this->faker->domainName(),
            'port' => $this->faker->numberBetween(1000, 65535),
            'service_port' => null,
            'protocol' => $this->faker->randomElement([
                ServerNode::PROTOCOL_VMESS,
                ServerNode::PROTOCOL_VLESS,
                ServerNode::PROTOCOL_TROJAN,
                ServerNode::PROTOCOL_SHADOWSOCKS,
            ]),
            'location_code' => 'HK',
            'location_name' => 'Hong Kong',
            'settings' => [
                'encryption' => 'auto',
                'network' => 'tcp',
            ],
            'traffic_limit' => $this->faker->numberBetween(1000000, 10000000), // 1GB - 10GB in KB
            'traffic_used' => $this->faker->numberBetween(0, 500000), // 0 - 500MB in KB
            'traffic_multiplier' => 1,
            'access_control' => [
                'min_trust_level' => $this->faker->numberBetween(0, 4),
            ],
            'status' => $this->faker->randomElement([
                ServerNode::STATUS_ACTIVE,
                ServerNode::STATUS_INACTIVE,
                ServerNode::STATUS_MAINTENANCE,
            ]),
            'device_limit' => $this->faker->numberBetween(1, 10),
            'connection_limit' => $this->faker->numberBetween(10, 100),
            'speed_limit_up' => $this->faker->numberBetween(10, 1000),
            'speed_limit_down' => $this->faker->numberBetween(50, 2000),
            'cross_node_ip_limit' => 0,
            'concurrent_ip_limit' => 0,
            'tcping_enabled' => false,
            'tcping_host' => null,
            'tcping_port' => null,
            'tcping_interval_seconds' => 60,
            'tcping_timeout_ms' => 3000,
            'tcping_alert_after_seconds' => 300,
            'tcping_recover_after_seconds' => 120,
        ];
    }

    public function active(): static
    {
        return $this->state(fn (array $attributes) => [
            'status' => ServerNode::STATUS_ACTIVE,
        ]);
    }

    public function inactive(): static
    {
        return $this->state(fn (array $attributes) => [
            'status' => ServerNode::STATUS_INACTIVE,
        ]);
    }

    public function vmess(): static
    {
        return $this->state(fn (array $attributes) => [
            'protocol' => ServerNode::PROTOCOL_VMESS,
            'settings' => [
                'encryption' => 'auto',
                'network' => 'ws',
                'path' => '/ws',
            ],
        ]);
    }

    public function trojan(): static
    {
        return $this->state(fn (array $attributes) => [
            'protocol' => ServerNode::PROTOCOL_TROJAN,
            'settings' => [
                'sni' => $attributes['host'],
                'allowInsecure' => false,
            ],
        ]);
    }
}
