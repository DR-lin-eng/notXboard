<?php

namespace Tests\Feature;

use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;

class GuestCommConfigTest extends TestCase
{
    use RefreshDatabase;

    public function test_hides_linux_do_oauth_when_credentials_are_missing(): void
    {
        config([
            'services.linux_do.client_id' => null,
            'services.linux_do.client_secret' => null,
        ]);

        $response = $this->getJson('/api/v1/guest/comm/config');

        $response->assertOk();
        $response->assertJsonPath('data.oauth_linux_do_enable', 0);
        $response->assertJsonPath('data.allow_oauth_register', 0);
    }

    public function test_exposes_linux_do_oauth_when_credentials_are_configured(): void
    {
        config([
            'services.linux_do.client_id' => 'test-client-id',
            'services.linux_do.client_secret' => 'test-client-secret',
        ]);

        $response = $this->getJson('/api/v1/guest/comm/config');

        $response->assertOk();
        $response->assertJsonPath('data.oauth_linux_do_enable', 1);
        $response->assertJsonPath('data.allow_oauth_register', 1);
    }
}
