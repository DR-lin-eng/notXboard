<?php

namespace App\Console;

use App\Services\Plugin\PluginManager;
use App\Utils\CacheKey;
use Illuminate\Console\Scheduling\Schedule;
use Illuminate\Foundation\Console\Kernel as ConsoleKernel;
use Illuminate\Support\Facades\Cache;

class Kernel extends ConsoleKernel
{
    /**
     * The Artisan commands provided by your application.
     *
     * @var array
     */
    protected $commands = [
        //
    ];

    /**
     * Define the application's command schedule.
     *
     * @param \Illuminate\Console\Scheduling\Schedule $schedule
     * @return void
     */
    protected function schedule(Schedule $schedule): void
    {
        if (!$this->rustGatewayOwnsCoreScheduler()) {
            $this->registerCompatSchedulerHeartbeat($schedule);
            $this->registerCompatCoreSchedules($schedule);
        }

        $this->registerCompatBackupSchedule($schedule);
        $this->registerPluginSchedules($schedule);
    }

    private function rustGatewayOwnsCoreScheduler(): bool
    {
        return filter_var(
            (string) env('RUST_GATEWAY_OWNS_SCHEDULER', false),
            FILTER_VALIDATE_BOOLEAN
        );
    }

    private function registerCompatSchedulerHeartbeat(Schedule $schedule): void
    {
        $schedule->call(function (): void {
            Cache::put(
                CacheKey::get('SCHEDULE_LAST_CHECK_AT', null),
                time(),
                now()->addMinutes(15)
            );
        })->name('schedule:heartbeat')->everyMinute()->onOneServer()->withoutOverlapping();
    }

    private function registerCompatCoreSchedules(Schedule $schedule): void
    {
        $this->registerCompatStatisticsAndCheckSchedules($schedule);
        $this->registerCompatResetAndNotificationSchedules($schedule);
        $this->registerCompatMaintenanceSchedules($schedule);
    }

    private function registerCompatStatisticsAndCheckSchedules(Schedule $schedule): void
    {
        $schedule->command('xboard:statistics')->dailyAt('0:10')->onOneServer();
        $schedule->command('check:order')->everyMinute()->onOneServer()->withoutOverlapping(10);
        $schedule->command('check:commission')->everyMinute()->onOneServer()->withoutOverlapping(10);
        $schedule->command('check:ticket')->everyMinute()->onOneServer()->withoutOverlapping(10);
        $schedule->command('check:server')->everyFiveMinutes()->onOneServer()->withoutOverlapping(10);
    }

    private function registerCompatResetAndNotificationSchedules(Schedule $schedule): void
    {
        $schedule->command('reset:traffic')->everyMinute()->onOneServer()->withoutOverlapping(10);
        $schedule->command('reset:log')->daily()->onOneServer();
        $schedule->command('subscription:rotate-credentials')->dailyAt('01:10')->onOneServer()->withoutOverlapping(30);

        if ((bool) env('ENABLE_SCHEDULED_MAIL_REMINDERS', false)) {
            $schedule->command('send:remindMail', ['--force'])->dailyAt('11:30')->onOneServer()->withoutOverlapping(30);
        }
    }

    private function registerCompatMaintenanceSchedules(Schedule $schedule): void
    {
        $schedule->command('horizon:snapshot')->everyFiveMinutes()->onOneServer();
        $schedule->command('cleanup:expired-online-status')->everyMinute()->onOneServer()->withoutOverlapping(4);
        $schedule->command('cleanup:expired-node-sessions')->everyMinute()->onOneServer()->withoutOverlapping(4);
        $schedule->command('oauth:sync-linux-do-users')->hourly()->onOneServer()->withoutOverlapping(10);
        $schedule->command('refunds:finalize-votes')->everyMinute()->onOneServer()->withoutOverlapping(4);
        $schedule->command('review:user-risk')->everyFiveMinutes()->onOneServer()->withoutOverlapping(10);
    }

    private function registerCompatBackupSchedule(Schedule $schedule): void
    {
        if (!(bool) env('ENABLE_AUTO_BACKUP_AND_UPDATE', false)) {
            return;
        }

        $schedule->command('backup:database', ['upload' => 'true'])
            ->dailyAt((string) env('BACKUP_SCHEDULE_AT', '03:30'))
            ->onOneServer()
            ->withoutOverlapping(180);
    }

    private function registerPluginSchedules(Schedule $schedule): void
    {
        app(PluginManager::class)->registerPluginSchedules($schedule);
    }

    /**
     * Register the commands for the application.
     *
     * @return void
     */
    protected function commands()
    {
        $this->load(__DIR__ . '/Commands');

        try {
            app(PluginManager::class)->initializeEnabledPlugins();
        } catch (\Exception $e) {
        }
        require base_path('routes/console.php');
    }
}
