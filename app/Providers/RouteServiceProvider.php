<?php

namespace App\Providers;

use Illuminate\Foundation\Support\Providers\RouteServiceProvider as ServiceProvider;
use Illuminate\Cache\RateLimiting\Limit;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\RateLimiter;
use Illuminate\Support\Facades\Route;

class RouteServiceProvider extends ServiceProvider
{
    /**
     * This namespace is applied to your controller routes.
     *
     * In addition, it is set as the URL generator's root namespace.
     *
     * @var string
     */
    protected $namespace = 'App\Http\Controllers';

    /**
     * Define your route model bindings, pattern filters, etc.
     *
     * @return void
     */
    public function boot()
    {
        $this->configureRateLimiting();

        //
        if (admin_setting('force_https')) {
            resolve(\Illuminate\Routing\UrlGenerator::class)->forceScheme('https');
        }

        parent::boot();
    }

    protected function configureRateLimiting(): void
    {
        RateLimiter::for('auth-login', function (Request $request) {
            $identity = strtolower((string) $request->input('email', 'guest'));
            return Limit::perMinute(20)->by($identity . '|' . $request->ip());
        });

        RateLimiter::for('auth-register', function (Request $request) {
            $identity = strtolower((string) $request->input('email', 'guest'));
            return Limit::perMinute(10)->by($identity . '|' . $request->ip());
        });

        RateLimiter::for('auth-reset', function (Request $request) {
            $identity = strtolower((string) $request->input('email', 'guest'));
            return Limit::perMinute(6)->by($identity . '|' . $request->ip());
        });

        RateLimiter::for('auth-email-verify', function (Request $request) {
            $identity = strtolower((string) $request->input('email', 'guest'));
            return Limit::perMinute(5)->by($identity . '|' . $request->ip());
        });

        RateLimiter::for('auth-quick-login', function (Request $request) {
            return Limit::perMinute(20)->by($request->ip());
        });

        RateLimiter::for('invite-create', function (Request $request) {
            return Limit::perMinute(10)->by((string) optional($request->user())->id ?: $request->ip());
        });

        RateLimiter::for('order-create', function (Request $request) {
            return Limit::perMinute(12)->by((string) optional($request->user())->id ?: $request->ip());
        });

        RateLimiter::for('order-checkout', function (Request $request) {
            $tradeNo = (string) $request->input('trade_no', 'none');
            return Limit::perMinute(20)->by(((string) optional($request->user())->id ?: $request->ip()) . '|' . $tradeNo);
        });

        RateLimiter::for('guest-public', function (Request $request) {
            return Limit::perMinute(120)->by($request->ip());
        });

        RateLimiter::for('payment-notify', function (Request $request) {
            $tradeNo = (string) ($request->input('out_trade_no') ?: $request->input('trade_no') ?: 'none');
            return Limit::perMinute(240)->by($request->ip() . '|' . $tradeNo);
        });
    }

    /**
     * Define the routes for the application.
     *
     * @return void
     */
    public function map()
    {
        $this->mapApiRoutes();
        $this->mapWebRoutes();

        //
    }

    /**
     * Define the "web" routes for the application.
     *
     * These routes all receive session state, CSRF protection, etc.
     *
     * @return void
     */
    protected function mapWebRoutes()
    {
        Route::middleware('web')
            ->namespace($this->namespace)
            ->group(base_path('routes/web.php'));
    }

    /**
     * Define the "api" routes for the application.
     *
     * These routes are typically stateless.
     *
     * @return void
     */
    protected function mapApiRoutes()
    {
        Route::group([
            'prefix' => '/api/v1',
            'middleware' => 'api',
            'namespace' => $this->namespace
        ], function ($router) {
            foreach (glob(app_path('Http//Routes//V1') . '/*.php') as $file) {
                if (!$this->shouldLoadPhpRouteFile($file)) {
                    continue;
                }
                $this->app->make('App\\Http\\Routes\\V1\\' . basename($file, '.php'))->map($router);
            }
        });


        Route::group([
            'prefix' => '/api/v2',
            'middleware' => 'api',
            'namespace' => $this->namespace
        ], function ($router) {
            foreach (glob(app_path('Http//Routes//V2') . '/*.php') as $file) {
                if (!$this->shouldLoadPhpRouteFile($file)) {
                    continue;
                }
                $this->app->make('App\\Http\\Routes\\V2\\' . basename($file, '.php'))->map($router);
            }
        });
    }

    private function shouldLoadPhpRouteFile(string $file): bool
    {
        $basename = basename($file, '.php');
        if (!$this->shouldLoadPhpApiCompatRoutes()) {
            return false;
        }
        if ($basename !== 'ServerRoute') {
            return true;
        }

        return filter_var(
            (string) env('PHP_SERVER_INGRESS_COMPAT', false),
            FILTER_VALIDATE_BOOLEAN
        );
    }

    private function shouldLoadPhpApiCompatRoutes(): bool
    {
        return filter_var(
            (string) env('PHP_HTTP_API_COMPAT', false),
            FILTER_VALIDATE_BOOLEAN
        );
    }
}
