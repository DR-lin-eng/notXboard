<?php

namespace Tests\Unit;

use App\Models\User;
use App\Services\Auth\RegisterModeService;
use Tests\TestCase;

class RegisterModeServiceTest extends TestCase
{
    public function test_force_oauth_login_does_not_lock_out_email_when_oauth_is_unavailable(): void
    {
        config([
            'v2board.force_oauth2_login' => 1,
            'services.linux_do.client_id' => null,
            'services.linux_do.client_secret' => null,
        ]);

        $service = app(RegisterModeService::class);

        $this->assertTrue($service->allowsEmailLogin(User::factory()->make([
            'is_super_admin' => false,
        ])));
    }

    public function test_force_oauth_login_still_limits_regular_users_when_oauth_is_available(): void
    {
        config([
            'v2board.force_oauth2_login' => 1,
            'services.linux_do.client_id' => 'test-client-id',
            'services.linux_do.client_secret' => 'test-client-secret',
        ]);

        $service = app(RegisterModeService::class);

        $this->assertFalse($service->allowsEmailLogin(User::factory()->make([
            'is_super_admin' => false,
        ])));
        $this->assertTrue($service->allowsEmailLogin(User::factory()->make([
            'is_super_admin' => true,
        ])));
    }
}
