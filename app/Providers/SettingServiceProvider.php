<?php

namespace App\Providers;

use App\Support\Setting;
use Illuminate\Contracts\Config\Repository as ConfigRepository;
use Illuminate\Support\ServiceProvider;
use Illuminate\Contracts\Foundation\Application;
use Illuminate\Support\Facades\URL;

class SettingServiceProvider extends ServiceProvider
{
    /**
     * Register services.
     *
     * @return void
     */
    public function register()
    {
        $this->app->scoped(Setting::class, function (Application $app) {
            return new Setting();
        });

    }

    /**
     * Bootstrap services.
     *
     * @return void
     */
    public function boot()
    {
        $this->syncRuntimeUrlConfig();
    }

    private function syncRuntimeUrlConfig(): void
    {
        $appUrl = $this->normalizeUrl(
            $this->app->make(Setting::class)->get('app_url')
        );

        if ($appUrl === null) {
            return;
        }

        /** @var ConfigRepository $config */
        $config = $this->app->make('config');

        $config->set('app.url', $appUrl);
        $config->set('filesystems.disks.public.url', $appUrl . '/storage');

        if ($this->app->bound('url')) {
            URL::forceRootUrl($appUrl);
            URL::forceScheme((string) parse_url($appUrl, PHP_URL_SCHEME));
        }
    }

    private function normalizeUrl(mixed $value): ?string
    {
        if (!is_string($value)) {
            return null;
        }

        $url = rtrim(trim($value), '/');
        if ($url === '' || !filter_var($url, FILTER_VALIDATE_URL)) {
            return null;
        }

        $scheme = strtolower((string) parse_url($url, PHP_URL_SCHEME));
        if (!in_array($scheme, ['http', 'https'], true)) {
            return null;
        }

        return $url;
    }
}
