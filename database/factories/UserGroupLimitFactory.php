<?php

namespace Database\Factories;

use App\Models\UserGroupLimit;
use Illuminate\Database\Eloquent\Factories\Factory;

class UserGroupLimitFactory extends Factory
{
    protected $model = UserGroupLimit::class;

    public function definition(): array
    {
        return [
            'trust_level' => $this->faker->numberBetween(0, 4),
            'speed_limit_up' => $this->faker->numberBetween(10, 1000),
            'speed_limit_down' => $this->faker->numberBetween(50, 2000),
            'device_limit' => $this->faker->numberBetween(1, 10),
            'connection_limit' => $this->faker->numberBetween(10, 100),
        ];
    }

    public function newUser(): static
    {
        return $this->state(fn (array $attributes) => [
            'trust_level' => UserGroupLimit::TRUST_LEVEL_NEW,
            'speed_limit_up' => 10,
            'speed_limit_down' => 50,
            'device_limit' => 2,
            'connection_limit' => 10,
        ]);
    }

    public function basicUser(): static
    {
        return $this->state(fn (array $attributes) => [
            'trust_level' => UserGroupLimit::TRUST_LEVEL_BASIC,
            'speed_limit_up' => 20,
            'speed_limit_down' => 100,
            'device_limit' => 3,
            'connection_limit' => 20,
        ]);
    }

    public function member(): static
    {
        return $this->state(fn (array $attributes) => [
            'trust_level' => UserGroupLimit::TRUST_LEVEL_MEMBER,
            'speed_limit_up' => 50,
            'speed_limit_down' => 200,
            'device_limit' => 5,
            'connection_limit' => 50,
        ]);
    }

    public function regular(): static
    {
        return $this->state(fn (array $attributes) => [
            'trust_level' => UserGroupLimit::TRUST_LEVEL_REGULAR,
            'speed_limit_up' => 100,
            'speed_limit_down' => 500,
            'device_limit' => 10,
            'connection_limit' => 100,
        ]);
    }

    public function leader(): static
    {
        return $this->state(fn (array $attributes) => [
            'trust_level' => UserGroupLimit::TRUST_LEVEL_LEADER,
            'speed_limit_up' => 0, // 无限制
            'speed_limit_down' => 0, // 无限制
            'device_limit' => 0, // 无限制
            'connection_limit' => 0, // 无限制
        ]);
    }
}