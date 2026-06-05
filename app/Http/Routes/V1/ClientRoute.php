<?php
namespace App\Http\Routes\V1;

use App\Http\Controllers\V1\Client\AppController;
use App\Http\Controllers\V1\Client\ClientController;
use Illuminate\Contracts\Routing\Registrar;

class ClientRoute
{
    public function map(Registrar $router)
    {
        $router->group([
            'prefix' => 'client',
            'middleware' => 'client'
        ], function ($router) {
            // App
            $router->get('/app/getConfig', [AppController::class, 'getConfig']);
            $router->get('/app/getVersion', [AppController::class, 'getVersion']);
            // Client (legacy path replaced with per-user random segment)
            $router->get('/{path}', [ClientController::class, 'subscribe'])
                ->where('path', '[a-zA-Z]{6,32}')
                ->middleware('throttle:60,1')
                ->name('client.subscribe.legacy');
        });
    }
}
