<?php

namespace App\Providers;

use App\Services\Plugin\HookManager;
use Illuminate\Support\ServiceProvider;
use Laravel\Octane\Facades\Octane;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Artisan;
use Laravel\Octane\Events\WorkerStarting;

class OctaneServiceProvider extends ServiceProvider
{
    public function register(): void
    {
    }

    public function boot(): void
    {
        if (!$this->app->bound('octane')) {
            return;
        }

        $this->app['events']->listen(WorkerStarting::class, function (): void {
            HookManager::reset();
        });

        // Default to a single scheduler source (system cron / scheduler worker).
        if (!(bool) env('OCTANE_ENABLE_TICK_SCHEDULER', false)) {
            return;
        }

        Octane::tick('scheduler', function (): void {
            $lock = Cache::lock('scheduler-lock', 30);

            if (!$lock->get()) {
                return;
            }

            try {
                Artisan::call('schedule:run');
            } finally {
                $lock->release();
            }
        })->seconds(30);
    }
}
