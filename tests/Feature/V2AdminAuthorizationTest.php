<?php

namespace Tests\Feature;

use App\Models\User;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Laravel\Sanctum\Sanctum;
use Tests\TestCase;

class V2AdminAuthorizationTest extends TestCase
{
    use RefreshDatabase;

    private function v2Path(string $endpoint): string
    {
        $securePath = (string) admin_setting('secure_path', admin_setting('frontend_admin_path', hash('crc32b', config('app.key'))));
        return '/api/v2/' . $securePath . '/' . ltrim($endpoint, '/');
    }

    public function test_regular_admin_cannot_access_super_admin_v2_config_routes(): void
    {
        $admin = User::factory()->create([
            'is_admin' => true,
            'is_super_admin' => false,
        ]);

        Sanctum::actingAs($admin);

        $this->getJson($this->v2Path('config/fetch'))
            ->assertStatus(403);
    }

    public function test_regular_admin_cannot_access_super_admin_v2_plugin_routes(): void
    {
        $admin = User::factory()->create([
            'is_admin' => true,
            'is_super_admin' => false,
        ]);

        Sanctum::actingAs($admin);

        $this->getJson($this->v2Path('plugin/getPlugins'))
            ->assertStatus(403);
    }

    public function test_regular_admin_cannot_access_super_admin_v2_payment_routes(): void
    {
        $admin = User::factory()->create([
            'is_admin' => true,
            'is_super_admin' => false,
        ]);

        Sanctum::actingAs($admin);

        $this->getJson($this->v2Path('payment/fetch'))
            ->assertStatus(403);
    }

    public function test_regular_admin_cannot_access_super_admin_v2_system_routes(): void
    {
        $admin = User::factory()->create([
            'is_admin' => true,
            'is_super_admin' => false,
        ]);

        Sanctum::actingAs($admin);

        $this->getJson($this->v2Path('system/getSystemStatus'))
            ->assertStatus(403);
    }

    public function test_regular_admin_cannot_access_super_admin_v2_user_routes(): void
    {
        $admin = User::factory()->create([
            'is_admin' => true,
            'is_super_admin' => false,
        ]);

        Sanctum::actingAs($admin);

        $this->getJson($this->v2Path('user/fetch'))
            ->assertStatus(403);
    }

    public function test_super_admin_can_access_super_admin_v2_routes(): void
    {
        $admin = User::factory()->create([
            'is_admin' => true,
            'is_super_admin' => true,
        ]);

        Sanctum::actingAs($admin);

        $this->getJson($this->v2Path('config/fetch'))
            ->assertStatus(200);
    }
}
