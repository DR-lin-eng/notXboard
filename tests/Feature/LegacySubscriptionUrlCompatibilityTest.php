<?php

namespace Tests\Feature;

use App\Models\User;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;

class LegacySubscriptionUrlCompatibilityTest extends TestCase
{
    use RefreshDatabase;

    public function test_legacy_token_subscription_url_still_works(): void
    {
        $user = User::factory()->create([
            'token' => str_repeat('a', 32),
            'transfer_enable' => 1024,
            'expired_at' => time() + 3600,
        ]);

        $response = $this->get('/s/' . $user->token);

        $response->assertOk();
    }

    public function test_randomized_subscription_url_still_works_after_adding_legacy_route(): void
    {
        $user = User::factory()->create([
            'token' => str_repeat('b', 32),
            'transfer_enable' => 1024,
            'expired_at' => time() + 3600,
        ]);
        $user->ensureSubscribeSecrets();

        $response = $this->get(route('client.subscribe', ['path' => $user->subscribe_path], false) . '?' . http_build_query([
            $user->subscribe_key => $user->token,
            $user->subscribe_salt => '1',
        ]));

        $response->assertOk();
    }
}
