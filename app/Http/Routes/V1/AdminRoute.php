<?php

namespace App\Http\Routes\V1;

use App\Http\Controllers\V1\Admin\UserGroupLimitController;
use App\Http\Controllers\V1\Admin\UserIndividualLimitController;
use App\Http\Controllers\V1\Admin\ApiKeyManagementController;
use App\Http\Controllers\V1\Admin\CommandCenterController;
use App\Http\Controllers\V1\Admin\SuperAdminController;
use App\Http\Controllers\V1\Admin\NodePlanManagementController;
use App\Http\Controllers\V1\Admin\SponsorPaymentProfileController;
use App\Http\Controllers\V1\Admin\RefundManagementController;
use Illuminate\Contracts\Routing\Registrar;
use Illuminate\Http\Request;

class AdminRoute
{
    public function map(Registrar $router)
    {
        $router->group([
            'prefix' => 'admin',
            'middleware' => ['auth:sanctum', 'admin.super']
        ], function ($router) {
            
            // 用户组限制管理
            $router->get('/group-limits', [UserGroupLimitController::class, 'index']);
            $router->get('/group-limits/{trustLevel}', [UserGroupLimitController::class, 'show']);
            $router->post('/group-limits', [UserGroupLimitController::class, 'store']);
            $router->put('/group-limits/batch', [UserGroupLimitController::class, 'batchUpdate']);
            $router->delete('/group-limits/{trustLevel}', [UserGroupLimitController::class, 'destroy']);
            $router->get('/group-limits/defaults/template', [UserGroupLimitController::class, 'getDefaults']);
            $router->post('/group-limits/defaults/apply', [UserGroupLimitController::class, 'applyDefaults']);
            
            // 用户个人限制管理（超级管理员）
            $router->get('/users', [UserIndividualLimitController::class, 'index']);
            $router->get('/users/{userId}/limits', [UserIndividualLimitController::class, 'show']);
            $router->post('/users/{userId}/limits', [UserIndividualLimitController::class, 'store']);
            $router->delete('/users/{userId}/limits', [UserIndividualLimitController::class, 'destroy']);
            $router->put('/users/limits/batch', [UserIndividualLimitController::class, 'batchUpdate']);
            
            // API 密钥管理（超级管理员）
            $router->get('/api-keys/stats', [ApiKeyManagementController::class, 'stats']);
            $router->get('/api-keys/search', [ApiKeyManagementController::class, 'search']);
            $router->post('/api-keys/batch-generate', [ApiKeyManagementController::class, 'batchGenerate']);
            $router->post('/api-keys/cleanup', [ApiKeyManagementController::class, 'cleanup']);
            $router->get('/users/{userId}/api-key', [ApiKeyManagementController::class, 'getUserApiKey']);
            $router->post('/users/{userId}/api-key/generate', [ApiKeyManagementController::class, 'generateUserApiKey']);
            $router->post('/users/{userId}/api-key/reset', [ApiKeyManagementController::class, 'resetUserApiKey']);

            // Cross-node concurrent IP limit (super admin)
            $router->get('/users/{userId}/concurrent-ip-limit', [SuperAdminController::class, 'getConcurrentIpLimit']);
            $router->put('/users/{userId}/concurrent-ip-limit', [SuperAdminController::class, 'setConcurrentIpLimit']);

            // Node plan management (super admin)
            $router->get('/node-plans', [NodePlanManagementController::class, 'index']);
            $router->get('/node-plans/node-options', [NodePlanManagementController::class, 'nodeOptions']);
            $router->post('/node-plans', [NodePlanManagementController::class, 'create']);
            $router->put('/node-plans/{id}', [NodePlanManagementController::class, 'update']);
            $router->delete('/node-plans/{id}', [NodePlanManagementController::class, 'delete']);

            // Internal command center (super admin)
            $router->get('/command-center', [CommandCenterController::class, 'show']);

            // Sponsor (super admin) epay profile
            $router->get('/sponsor-epay', [SponsorPaymentProfileController::class, 'show']);
            $router->put('/sponsor-epay', [SponsorPaymentProfileController::class, 'upsert']);

            // Refunds (super admin override + finalize voting)
            $router->get('/refunds', [RefundManagementController::class, 'index']);
            $router->get('/refunds/{id}', [RefundManagementController::class, 'detail']);
            $router->post('/refunds/{id}/approve', [RefundManagementController::class, 'approve']);
            $router->post('/refunds/{id}/deny', [RefundManagementController::class, 'deny']);
            $router->post('/refunds/finalize', [RefundManagementController::class, 'finalize']);
        });
        
        // 用户个人限制管理（普通用户可以管理自己的）
        $router->group([
            'prefix' => 'user',
            'middleware' => ['auth:sanctum']
        ], function ($router) {
            $router->get('/limits', function (Request $request) {
                return app(UserIndividualLimitController::class)->show($request, $request->user()->id);
            });
            $router->post('/limits', function (Request $request) {
                return app(UserIndividualLimitController::class)->store($request, $request->user()->id);
            });
            $router->delete('/limits', function (Request $request) {
                return app(UserIndividualLimitController::class)->destroy($request, $request->user()->id);
            });
            
            // API 密钥管理（普通用户）
            $router->get('/api-key', [\App\Http\Controllers\V1\User\ApiKeyController::class, 'show']);
            $router->post('/api-key/generate', [\App\Http\Controllers\V1\User\ApiKeyController::class, 'generate']);
            $router->post('/api-key/reset', [\App\Http\Controllers\V1\User\ApiKeyController::class, 'reset']);
            $router->post('/api-key/validate', [\App\Http\Controllers\V1\User\ApiKeyController::class, 'validateApiKey']);
        });
    }
}
