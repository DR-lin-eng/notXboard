<?php

namespace App\Providers;

use App\Services\TelegramService;
use Illuminate\Support\Facades\Cache;
use Illuminate\Support\Facades\Event;
use Illuminate\Support\Facades\Gate;
use Laravel\Horizon\HorizonApplicationServiceProvider;
use Laravel\Horizon\Events\LongWaitDetected;
use Laravel\Horizon\Events\MasterSupervisorOutOfMemory;
use Laravel\Horizon\Events\SupervisorOutOfMemory;
use Laravel\Horizon\Events\UnableToLaunchProcess;
use Throwable;

class HorizonServiceProvider extends HorizonApplicationServiceProvider
{
    /**
     * Bootstrap any application services.
     *
     * @return void
     */
    public function boot(): void
    {
        parent::boot();

        $this->registerTelegramAlerts();
    }

    /**
     * Register the Horizon gate.
     *
     * This gate determines who can access Horizon in non-local environments.
     *
     * @return void
     */
    protected function gate(): void
    {
        Gate::define('viewHorizon', function ($user = null): bool {
            if (app()->environment('local')) {
                return true;
            }

            if (!$user) {
                return false;
            }

            if ((bool) ($user->is_super_admin ?? false)) {
                return true;
            }

            $allowlist = $this->horizonAllowedEmails();
            return !empty($allowlist) && in_array((string) ($user->email ?? ''), $allowlist, true);
        });
    }

    private function horizonAllowedEmails(): array
    {
        $settingValue = '';

        try {
            $settingValue = (string) admin_setting('horizon_allowed_emails', '');
        } catch (Throwable) {
            // ignore and fallback to env only
        }

        $raw = trim((string) env('HORIZON_ALLOWED_EMAILS', $settingValue));
        if ($raw === '') {
            return [];
        }

        return array_values(array_filter(array_map(static fn (string $email): string => trim($email), explode(',', $raw))));
    }

    private function registerTelegramAlerts(): void
    {
        Event::listen(LongWaitDetected::class, function (LongWaitDetected $event): void {
            $this->dispatchOpsAlert('horizon:long-wait', sprintf(
                "Horizon queue wait is too long.\nconnection: %s\nqueue: %s\nseconds: %d",
                $event->connection,
                $event->queue,
                (int) $event->seconds
            ));
        });

        Event::listen(UnableToLaunchProcess::class, function (): void {
            $this->dispatchOpsAlert(
                'horizon:unable-to-launch-process',
                'Horizon worker process failed to launch and entered cooldown.',
                180
            );
        });

        Event::listen(SupervisorOutOfMemory::class, function (SupervisorOutOfMemory $event): void {
            $this->dispatchOpsAlert('horizon:supervisor-oom', sprintf(
                "Horizon supervisor hit memory limit.\nsupervisor: %s\nmemory_usage_mb: %s",
                $event->supervisor->name ?? 'unknown',
                number_format((float) ($event->getMemoryUsage() / 1024 / 1024), 2)
            ));
        });

        Event::listen(MasterSupervisorOutOfMemory::class, function (MasterSupervisorOutOfMemory $event): void {
            $this->dispatchOpsAlert('horizon:master-supervisor-oom', sprintf(
                "Horizon master supervisor hit memory limit.\nname: %s",
                $event->master->name ?? 'unknown'
            ));
        });
    }

    private function dispatchOpsAlert(string $event, string $message, int $cooldownSeconds = 120): void
    {
        if (!$this->shouldSendTelegramOpsAlert()) {
            return;
        }

        $cacheKey = 'ops:telegram-alert:' . md5($event . ':' . $message);
        if (!Cache::add($cacheKey, time(), $cooldownSeconds)) {
            return;
        }

        $payload = "*[Ops Alert]*\nsource: horizon\nevent: {$event}\ntime: " . now()->toDateTimeString() . "\n\n{$message}";

        try {
            app(TelegramService::class)->sendOpsAlert($payload, [
                'source' => 'horizon',
                'event' => $event,
            ]);
        } catch (Throwable $exception) {
            report($exception);
        }
    }

    private function shouldSendTelegramOpsAlert(): bool
    {
        try {
            return (bool) admin_setting('telegram_bot_enable', 0)
                && (bool) admin_setting('telegram_notify_ops_alert', 1);
        } catch (Throwable) {
            return false;
        }
    }
}
