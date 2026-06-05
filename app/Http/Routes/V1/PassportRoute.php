<?php
namespace App\Http\Routes\V1;

use App\Http\Controllers\V1\Passport\AuthController;
use App\Http\Controllers\V1\Passport\CommController;
use App\Http\Controllers\V1\Passport\OAuth2Controller;
use Illuminate\Contracts\Routing\Registrar;

class PassportRoute
{
    public function map(Registrar $router)
    {
        $router->group([
            'prefix' => 'passport'
        ], function ($router) {
            // Auth
            $router->post('/auth/register', [AuthController::class, 'register'])->middleware('throttle:5,1');
            $router->post('/auth/login', [AuthController::class, 'login'])->middleware('throttle:15,1');
            $router->get('/auth/pow-challenge', [AuthController::class, 'powChallenge'])->middleware('throttle:30,1');
            $router->get('/auth/token2Login', [AuthController::class, 'token2Login'])->middleware('throttle:10,1');
            $router->post('/auth/forget', [AuthController::class, 'forget'])->middleware('throttle:5,1');
            $router->post('/auth/getQuickLoginUrl', [AuthController::class, 'getQuickLoginUrl'])->middleware('throttle:5,1');
            $router->post('/auth/loginWithMailLink', [AuthController::class, 'loginWithMailLink'])->middleware('throttle:5,1');
            // OAuth2
            $router->get('/oauth2/linux-do/redirect', [OAuth2Controller::class, 'redirect']);
            $router->get('/oauth2/linux-do/callback', [OAuth2Controller::class, 'callback']);
            $router->post('/oauth2/refresh', [OAuth2Controller::class, 'refresh'])->middleware('auth:sanctum');
            $router->post('/oauth2/sync', [OAuth2Controller::class, 'syncUserInfo'])->middleware('auth:sanctum');

            // Comm
            $router->post('/comm/sendEmailVerify', [CommController::class, 'sendEmailVerify'])->middleware('throttle:5,1');
            $router->post('/comm/pv', [CommController::class, 'pv'])->middleware('throttle:30,1');
        });
    }
}
