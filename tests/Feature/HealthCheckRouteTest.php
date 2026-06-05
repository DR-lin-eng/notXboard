<?php

namespace Tests\Feature;

use App\Services\Plugin\PluginManager;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\Schema;
use Mockery;
use Tests\TestCase;

class HealthCheckRouteTest extends TestCase
{
    use RefreshDatabase;

    public function test_health_check_returns_ok_when_core_tables_exist(): void
    {
        $this->get('/healthz')
            ->assertOk()
            ->assertSeeText('ok');
    }

    public function test_health_check_returns_service_unavailable_when_core_table_is_missing(): void
    {
        Schema::drop('v2_plugins');

        $this->get('/healthz')
            ->assertStatus(503)
            ->assertSeeText('not ready');
    }

    public function test_health_check_bypasses_plugin_initialization(): void
    {
        $pluginManager = Mockery::mock(PluginManager::class);
        $pluginManager->shouldReceive('initializeEnabledPlugins')->never();

        $this->app->instance(PluginManager::class, $pluginManager);

        $this->get('/healthz')
            ->assertOk();
    }
}
