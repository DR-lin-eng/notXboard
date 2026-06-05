<?php

namespace Database\Factories;

use App\Models\UserIndividualLimit;
use App\Models\User;
use Illuminate\Database\Eloquent\Factories\Factory;

class UserIndividualLimitFactory extends Factory
{
    protected $model = UserIndividualLimit::class;

    public function definition(): array
    {
        return [
            'user_id' => User::factory(),
            'speed_limit_up' => $this->faker->numberBetween(0, 1000),
            'speed_limit_down' => $this->faker->numberBetween(0, 2000),
            'device_limit' => $this->faker->numberBetween(0, 10),
            'connection_limit' => $this->faker->numberBetween(0, 100),
        ];
    }

    public function withCustomLimits(): static
    {
        return $this->state(fn (array $attributes) => [
            'speed_limit_up' => $this->faker->numberBetween(100, 1000),
            'speed_limit_down' => $this->faker->numberBetween(200, 2000),
            'device_limit' => $this->faker->numberBetween(5, 10),
            'connection_limit' => $this->faker->numberBetween(50, 100),
        ]);
    }

    public function useGroupDefaults(): static
    {
        return $this->state(fn (array $attributes) => [
            'speed_limit_up' => 0,
            'speed_limit_down' => 0,
            'device_limit' => 0,
            'connection_limit' => 0,
        ]);
    }

    public function partialCustom(): static
    {
        return $this->state(fn (array $attributes) => [
            'speed_limit_up' => $this->faker->numberBetween(100, 500),
            'speed_limit_down' => 0, // 使用组配置
            'device_limit' => $this->faker->numberBetween(5, 8),
            'connection_limit' => 0, // 使用组配置
        ]);
    }
}