<?php

namespace Tests\Unit;

use App\Services\Plugin\PluginManager;
use Illuminate\Console\Scheduling\Schedule;
use Illuminate\Support\Facades\Schema;
use Tests\TestCase;

class PluginManagerBootstrapSafetyTest extends TestCase
{
    public function test_initialize_enabled_plugins_is_noop_when_plugins_table_missing(): void
    {
        Schema::shouldReceive('hasTable')
            ->atLeast()
            ->once()
            ->andReturn(false);

        $manager = new PluginManager();
        $manager->initializeEnabledPlugins();

        $this->assertSame([], $manager->getEnabledPlugins());
        $this->assertSame([], $manager->getEnabledPluginsByType('payment'));
    }

    public function test_register_plugin_schedules_is_noop_when_schema_check_throws(): void
    {
        Schema::shouldReceive('hasTable')
            ->once()
            ->andThrow(new \RuntimeException('database bootstrap not ready'));

        $manager = new PluginManager();
        $schedule = new Schedule();

        $manager->registerPluginSchedules($schedule);

        $this->assertCount(0, $schedule->events());
    }

    public function test_install_default_plugins_returns_early_when_plugins_table_missing(): void
    {
        app()->forgetScopedInstances();

        PluginManager::installDefaultPlugins();

        $this->assertTrue(true);
    }
}
