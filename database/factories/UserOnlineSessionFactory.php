<?php

namespace Database\Factories;

use App\Models\UserOnlineSession;
use App\Models\User;
use App\Models\ServerNode;
use Illuminate\Database\Eloquent\Factories\Factory;

class UserOnlineSessionFactory extends Factory
{
    protected $model = UserOnlineSession::class;

    public function definition(): array
    {
        return [
            'user_id' => User::factory(),
            'node_id' => ServerNode::factory(),
            'ip_address' => $this->faker->ipv4(),
            'user_agent' => $this->faker->userAgent(),
            'connection_count' => $this->faker->numberBetween(1, 10),
            'upload_traffic' => $this->faker->numberBetween(0, 1000000), // KB
            'download_traffic' => $this->faker->numberBetween(0, 5000000), // KB
            'last_activity' => $this->faker->dateTimeBetween('-1 hour', 'now'),
        ];
    }

    public function active(): static
    {
        return $this->state(fn (array $attributes) => [
            'last_activity' => now()->subMinutes($this->faker->numberBetween(1, 4)),
        ]);
    }

    public function expired(): static
    {
        return $this->state(fn (array $attributes) => [
            'last_activity' => now()->subMinutes($this->faker->numberBetween(10, 60)),
        ]);
    }

    public function withTraffic(): static
    {
        return $this->state(fn (array $attributes) => [
            'upload_traffic' => $this->faker->numberBetween(100000, 1000000),
            'download_traffic' => $this->faker->numberBetween(500000, 5000000),
        ]);
    }
}