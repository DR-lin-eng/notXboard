<?php
namespace App\Http\Routes\V1;

use App\Http\Controllers\V1\Guest\CommController;
use App\Http\Controllers\V1\Guest\PaymentController;
use App\Http\Controllers\V1\Guest\PlanController;
use App\Http\Controllers\V1\Guest\TelegramController;
use App\Http\Controllers\V1\Guest\PublicDashboardController;
use Illuminate\Contracts\Routing\Registrar;

class GuestRoute
{
    public function map(Registrar $router)
    {
        $router->group([
            'prefix' => 'guest'
        ], function ($router) {
            // Plan
            $router->get('/plan/fetch', [PlanController::class, 'fetch'])->middleware('throttle:120,1');
            // Telegram
            $router->post('/telegram/webhook', [TelegramController::class, 'webhook'])->middleware('throttle:240,1');
            // Payment
            $router->match(['get', 'post'], '/payment/notify/{method}/{uuid}', [PaymentController::class, 'notify'])->middleware('throttle:600,1');
            // Comm
            $router->get('/comm/config', [CommController::class, 'config'])->middleware('throttle:120,1');

            // Public dashboard
            $router->get('/public/overview', [PublicDashboardController::class, 'overview'])->middleware('throttle:120,1');
            $router->get('/public/leaderboards', [PublicDashboardController::class, 'leaderboards'])->middleware('throttle:120,1');
            $router->get('/public/geo', [PublicDashboardController::class, 'geo'])->middleware('throttle:120,1');
        });
    }
}
