<?php

namespace Database\Factories;

use App\Models\User;
use Illuminate\Database\Eloquent\Factories\Factory;
use Illuminate\Support\Str;

class UserFactory extends Factory
{
    protected $model = User::class;

    public function definition(): array
    {
        return [
            'email' => $this->faker->unique()->safeEmail(),
            'password' => bcrypt('password'),
            'password_algo' => null,
            'password_salt' => null,
            'balance' => 0,
            'discount' => null,
            'commission_type' => 0,
            'commission_rate' => 0,
            'commission_balance' => 0,
            't' => time(),
            'u' => 0,
            'd' => 0,
            'transfer_enable' => 0,
            'banned' => false,
            'is_admin' => false,
            'is_staff' => false,
            'is_super_admin' => false,
            'last_login_at' => null,
            'uuid' => (string) Str::uuid(),
            'group_id' => null,
            'plan_id' => null,
            'speed_limit' => null,
            'remind_expire' => true,
            'remind_traffic' => true,
            'token' => Str::lower(Str::random(32)),
            'expired_at' => time() + 86400 * 30,
            'remarks' => null,
            'trust_level' => 0,
            'concurrent_ip_limit' => 3,
            'created_at' => time(),
            'updated_at' => time(),
        ];
    }
}
