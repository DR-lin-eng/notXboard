<?php

namespace Tests\Unit;

use App\Models\Setting as SettingModel;
use App\Providers\SettingServiceProvider;
use App\Support\Setting as SettingStore;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;

class SettingServiceProviderTest extends TestCase
{
    use RefreshDatabase;

    protected function setUp(): void
    {
        parent::setUp();

        config([
            'cache.stores.redis' => ['driver' => 'array'],
        ]);

        $this->app['cache']->forgetDriver('redis');
        $this->app->forgetScopedInstances();
    }

    public function test_syncs_runtime_app_url_from_admin_settings(): void
    {
        config([
            'app.url' => 'http://127.0.0.1:8000',
            'filesystems.disks.public.url' => 'http://127.0.0.1:8000/storage',
        ]);

        SettingModel::createOrUpdate('app_url', 'https://portal.example.com/');
        cache()->store('redis')->forget(SettingStore::CACHE_KEY);
        $this->app->forgetScopedInstances();

        (new SettingServiceProvider($this->app))->boot();

        $this->assertSame('https://portal.example.com', config('app.url'));
        $this->assertSame('https://portal.example.com/storage', config('filesystems.disks.public.url'));
        $this->assertSame('https://portal.example.com/status', url('/status'));
    }

    public function test_keeps_fallback_config_when_admin_app_url_is_invalid(): void
    {
        config([
            'app.url' => 'http://127.0.0.1:8000',
            'filesystems.disks.public.url' => 'http://127.0.0.1:8000/storage',
        ]);

        SettingModel::createOrUpdate('app_url', 'not-a-valid-url');
        cache()->store('redis')->forget(SettingStore::CACHE_KEY);
        $this->app->forgetScopedInstances();

        (new SettingServiceProvider($this->app))->boot();

        $this->assertSame('http://127.0.0.1:8000', config('app.url'));
        $this->assertSame('http://127.0.0.1:8000/storage', config('filesystems.disks.public.url'));
    }
}
