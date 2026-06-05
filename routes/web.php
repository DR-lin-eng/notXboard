<?php

use App\Services\ThemeService;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Route;
use Illuminate\Support\Facades\Log;
use Illuminate\Support\Facades\File;
use Illuminate\Support\Facades\Schema;

/*
|--------------------------------------------------------------------------
| Web Routes
|--------------------------------------------------------------------------
|
| Here is where you can register web routes for your application. These
| routes are loaded by the RouteServiceProvider within a group which
| contains the "web" middleware group. Now create something great!
|
*/


Route::get('/healthz', function () {
    try {
        if (!Schema::hasTable('migrations') || !Schema::hasTable('v2_settings')) {
            return response('not ready', 503, ['Cache-Control' => 'no-store']);
        }
    } catch (\Throwable) {
        return response('not ready', 503, ['Cache-Control' => 'no-store']);
    }

    return response('ok', 200, ['Cache-Control' => 'no-store']);
})->name('healthz');

if (!filter_var((string) env('PHP_WEB_COMPAT', false), FILTER_VALIDATE_BOOLEAN)) {
    return;
}

Route::get('/', function (Request $request) {
    if (admin_setting('app_url') && admin_setting('safe_mode_enable', 0)) {
        if ($request->server('HTTP_HOST') !== parse_url(admin_setting('app_url'))['host']) {
            abort(403);
        }
    }

    return view('public-dashboard', [
        'title' => admin_setting('app_name', 'Portal'),
        'version' => config('app.version', '1.0.0'),
        'description' => admin_setting('app_description', 'Secure access portal'),
        'logo' => admin_setting('logo'),
    ]);
});

$renderPortal = function (Request $request) {
    if (admin_setting('app_url') && admin_setting('safe_mode_enable', 0)) {
        if ($request->server('HTTP_HOST') !== parse_url(admin_setting('app_url'))['host']) {
            abort(403);
        }
    }

    $defaultTheme = ThemeService::DEFAULT_THEME;
    $theme = $defaultTheme;
    $themeService = new ThemeService();

    try {
        if (!$themeService->exists($theme)) {
            if ($defaultTheme !== 'portal' && $themeService->exists('portal')) {
                Log::warning('Default theme not found, falling back to portal', ['theme' => $theme]);
                $theme = 'portal';
            } else {
                throw new Exception('主题视图文件不存在');
            }
        }

        if (!$themeService->getThemeViewPath($theme)) {
            throw new Exception('主题视图文件不存在');
        }

        $publicThemePath = public_path('theme/' . $theme);
        if (!File::exists($publicThemePath)) {
            $themePath = $themeService->getThemePath($theme);
            if (!$themePath || !File::copyDirectory($themePath, $publicThemePath)) {
                throw new Exception('主题初始化失败');
            }
            Log::info('Theme initialized in public directory', ['theme' => $theme]);
        }

        $renderParams = [
            'title' => admin_setting('app_name', 'Portal'),
            'theme' => $theme,
            'version' => config('app.version', '1.0.0'),
            'asset_version' => (function () use ($theme) {
                $defaultVersion = (string) config('app.version', '1.0.0');
                $publicThemePath = public_path('theme/' . $theme);
                $jsPath = $publicThemePath . '/app.js';
                $cssPath = $publicThemePath . '/app.css';
                $timestamps = [];

                if (File::exists($jsPath)) {
                    $timestamps[] = (int) @filemtime($jsPath);
                }
                if (File::exists($cssPath)) {
                    $timestamps[] = (int) @filemtime($cssPath);
                }

                $timestamps = array_values(array_filter($timestamps));
                return count($timestamps) ? (string) max($timestamps) : $defaultVersion;
            })(),
            'description' => admin_setting('app_description', 'Secure access portal'),
            'logo' => admin_setting('logo'),
            'theme_config' => $themeService->getConfig($theme)
        ];
        return view('theme::' . $theme . '.dashboard', $renderParams);
    } catch (Exception $e) {
        Log::error('Theme rendering failed', [
            'theme' => $theme,
            'error' => $e->getMessage()
        ]);
        abort(500, '主题加载失败');
    }
};

Route::get('/app', $renderPortal);

Route::get('/login/linux-do', function () {
    return view('login-linux-do');
});

$securePath = admin_setting('secure_path', admin_setting('frontend_admin_path', hash('crc32b', config('app.key'))));

Route::get('/' . $securePath, function () {
    return view('admin', [
        'title' => admin_setting('app_name', 'Portal'),
        'theme_sidebar' => admin_setting('frontend_theme_sidebar', 'light'),
        'theme_header' => admin_setting('frontend_theme_header', 'dark'),
        'theme_color' => admin_setting('frontend_theme_color', 'default'),
        'background_url' => admin_setting('frontend_background_url'),
        'version' => config('app.version', '1.0.0'),
        'logo' => admin_setting('logo'),
        'secure_path' => admin_setting('secure_path', admin_setting('frontend_admin_path', hash('crc32b', config('app.key'))))
    ]);
});

Route::get('/' . $securePath . '/command-center', function (Request $request) use ($securePath) {
    if (admin_setting('app_url') && admin_setting('safe_mode_enable', 0)) {
        if ($request->server('HTTP_HOST') !== parse_url(admin_setting('app_url'))['host']) {
            abort(403);
        }
    }

    return view('admin-command-center', [
        'title' => admin_setting('app_name', 'Portal'),
        'version' => config('app.version', '1.0.0'),
        'logo' => admin_setting('logo'),
        'secure_path' => $securePath,
        'description' => admin_setting('app_description', 'Super admin command center'),
    ]);
});

Route::get('/' . $securePath . '/leaderboards', function (Request $request) use ($securePath) {
    if (admin_setting('app_url') && admin_setting('safe_mode_enable', 0)) {
        if ($request->server('HTTP_HOST') !== parse_url(admin_setting('app_url'))['host']) {
            abort(403);
        }
    }

    return redirect('/' . $securePath . '/command-center');
});

Route::get('/' . (admin_setting('subscribe_path', 's')) . '/{token}', [\App\Http\Controllers\V1\Client\ClientController::class, 'subscribe'])
    ->where('token', '[A-Fa-f0-9]{32}')
    ->middleware(['client', 'throttle:60,1'])
    ->name('client.subscribe.token');

Route::get('/' . (admin_setting('subscribe_path', 's')) . '/{path}', [\App\Http\Controllers\V1\Client\ClientController::class, 'subscribe'])
    ->where('path', '[a-zA-Z]{6,32}')
    ->middleware(['client', 'throttle:60,1'])
    ->name('client.subscribe');
