<?php

namespace App\Http\Routes\V1;

use App\Http\Controllers\V1\Tcping\AgentController;
use Illuminate\Contracts\Routing\Registrar;

class TcpingRoute
{
    public function map(Registrar $router)
    {
        $router->group([
            'prefix' => 'tcping/agent',
        ], function ($router) {
            $router->get('/config', [AgentController::class, 'config']);
            $router->post('/heartbeat', [AgentController::class, 'heartbeat']);
            $router->post('/samples', [AgentController::class, 'samples']);
        });
    }
}
