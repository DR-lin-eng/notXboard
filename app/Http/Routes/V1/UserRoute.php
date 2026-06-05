<?php
namespace App\Http\Routes\V1;

use App\Http\Controllers\V1\User\CommController;
use App\Http\Controllers\V1\User\CouponController;
use App\Http\Controllers\V1\User\GiftCardController;
use App\Http\Controllers\V1\User\InviteController;
use App\Http\Controllers\V1\User\KnowledgeController;
use App\Http\Controllers\V1\User\NoticeController;
use App\Http\Controllers\V1\User\OrderController;
use App\Http\Controllers\V1\User\PlanController;
use App\Http\Controllers\V1\User\ServerController;
use App\Http\Controllers\V1\User\ServerNodeController;
use App\Http\Controllers\V1\User\StatController;
use App\Http\Controllers\V1\User\TelegramController;
use App\Http\Controllers\V1\User\TicketController;
use App\Http\Controllers\V1\User\TcpingController;
use App\Http\Controllers\V1\User\TrafficStatsController;
use App\Http\Controllers\V1\User\AuditRuleController;
use App\Http\Controllers\V1\User\AuditLogController;
use App\Http\Controllers\V1\User\AccessControlController;
use App\Http\Controllers\V1\User\MeController;
use App\Http\Controllers\V1\User\NodeAdminTicketController;
use App\Http\Controllers\V1\User\NodeAdminController;
use App\Http\Controllers\V1\User\NodePlanController;
use App\Http\Controllers\V1\User\PaymentProfileController;
use App\Http\Controllers\V1\User\SponsorController;
use App\Http\Controllers\V1\User\RefundController;
use App\Http\Controllers\V1\User\RefundAdminController;
use App\Http\Controllers\V1\User\RefundVoteController;
use App\Http\Controllers\V1\User\UserController;
use Illuminate\Contracts\Routing\Registrar;

class UserRoute
{
    public function map(Registrar $router)
    {
        $router->group([
            'prefix' => 'user',
            'middleware' => 'user'
        ], function ($router) {
            // User
            $router->get('/resetSecurity', [UserController::class, 'resetSecurity']);
            $router->get('/info', [UserController::class, 'info']);
            $router->get('/me', [MeController::class, 'show']);
            $router->post('/changePassword', [UserController::class, 'changePassword']);
            $router->post('/update', [UserController::class, 'update']);
            $router->get('/getSubscribe', [UserController::class, 'getSubscribe']);
            $router->get('/plan/quota', [UserController::class, 'getPlanQuotaUsage']);
            $router->get('/getStat', [UserController::class, 'getStat']);
            $router->get('/checkLogin', [UserController::class, 'checkLogin']);
            $router->post('/transfer', [UserController::class, 'transfer'])->middleware('throttle:10,1');
            $router->post('/getQuickLoginUrl', [UserController::class, 'getQuickLoginUrl']);
            $router->get('/getActiveSession', [UserController::class, 'getActiveSession']);
            $router->post('/removeActiveSession', [UserController::class, 'removeActiveSession']);
            // Order
            $router->post('/order/save', [OrderController::class, 'save'])->middleware('throttle:10,1');
            $router->post('/order/checkout', [OrderController::class, 'checkout'])->middleware('throttle:20,1');
            $router->get('/order/check', [OrderController::class, 'check']);
            $router->get('/order/detail', [OrderController::class, 'detail']);
            $router->get('/order/fetch', [OrderController::class, 'fetch']);
            $router->get('/order/getPaymentMethod', [OrderController::class, 'getPaymentMethod']);
            $router->post('/order/cancel', [OrderController::class, 'cancel']);
            // Plan
            $router->get('/plan/fetch', [PlanController::class, 'fetch']);
            // Node plan publisher (personal admin)
            $router->get('/node-plans', [NodePlanController::class, 'myPlans']);
            $router->post('/node-plans', [NodePlanController::class, 'create']);
            $router->put('/node-plans/{id}', [NodePlanController::class, 'update']);
            $router->delete('/node-plans/{id}', [NodePlanController::class, 'delete']);
            // Invite
            $router->post('/invite/save', [InviteController::class, 'save'])->middleware('throttle:10,1');
            $router->get('/invite/fetch', [InviteController::class, 'fetch']);
            $router->get('/invite/details', [InviteController::class, 'details']);
            // Notice
            $router->get('/notice/fetch', [NoticeController::class, 'fetch']);
            $router->post('/notice', [NoticeController::class, 'save']);
            $router->post('/notice/{id}/toggle', [NoticeController::class, 'toggle']);
            $router->delete('/notice/{id}', [NoticeController::class, 'drop']);
            // Ticket
            $router->post('/ticket/reply', [TicketController::class, 'reply']);
            $router->post('/ticket/close', [TicketController::class, 'close']);
            $router->post('/ticket/save', [TicketController::class, 'save']);
            $router->get('/ticket/fetch', [TicketController::class, 'fetch']);
            // Node admin ticket inbox (assigned by node)
            $router->get('/node-admin/tickets', [NodeAdminTicketController::class, 'inbox']);
            $router->get('/node-admin/ticket/detail', [NodeAdminTicketController::class, 'detail']);
            $router->post('/node-admin/ticket/reply', [NodeAdminTicketController::class, 'reply']);
            $router->post('/node-admin/ticket/close', [NodeAdminTicketController::class, 'close']);
            // Node admin stats & blacklist
            $router->get('/node-admin/server-nodes/{id}/users-traffic', [NodeAdminController::class, 'nodeUsersTraffic']);
            $router->post('/node-admin/server-nodes/{id}/blacklist', [NodeAdminController::class, 'blacklistUser']);
            $router->post('/node-admin/server-nodes/{id}/unblacklist', [NodeAdminController::class, 'unblacklistUser']);
            $router->post('/ticket/withdraw', [TicketController::class, 'withdraw']);
            // Server
            $router->get('/server/fetch', [ServerController::class, 'fetch']);
            // Server Node Management
            $router->get('/server-nodes', [ServerNodeController::class, 'index']);
            $router->get('/server-nodes/protocols', [ServerNodeController::class, 'protocols']);
            $router->post('/server-nodes', [ServerNodeController::class, 'store']);
            $router->get('/server-nodes/{id}', [ServerNodeController::class, 'show']);
            $router->put('/server-nodes/{id}', [ServerNodeController::class, 'update']);
            $router->delete('/server-nodes/{id}', [ServerNodeController::class, 'destroy']);
            $router->post('/server-nodes/{id}/deploy', [ServerNodeController::class, 'deploy']);
            $router->get('/server-nodes/{id}/deploy-command', [ServerNodeController::class, 'deployCommand']);
            $router->get('/server-nodes/{id}/status', [ServerNodeController::class, 'status']);
            $router->post('/server-nodes/{id}/access', [ServerNodeController::class, 'configureAccess']);
            $router->get('/server-nodes/{id}/traffic', [TrafficStatsController::class, 'nodeTrafficStats']);
            $router->get('/server-nodes/{id}/tcping', [TcpingController::class, 'nodeOverview']);
            $router->get('/node-traffic', [TrafficStatsController::class, 'myNodeTraffic']);
            $router->get('/traffic-usage-logs', [TrafficStatsController::class, 'usageLogs']);
            $router->get('/tcping/agents', [TcpingController::class, 'agents']);
            $router->post('/tcping/agents', [TcpingController::class, 'createAgent']);
            $router->post('/tcping/agents/{id}/rotate-token', [TcpingController::class, 'rotateToken']);
            $router->post('/tcping/agents/{id}/toggle', [TcpingController::class, 'toggle']);
            $router->get('/tcping/agents/{id}/install-command', [TcpingController::class, 'installCommand']);
            // Audit rules & logs (node owner)
            $router->get('/server-nodes/{id}/audit-rules', [AuditRuleController::class, 'index']);
            $router->post('/server-nodes/{id}/audit-rules', [AuditRuleController::class, 'store']);
            $router->put('/server-nodes/{id}/audit-rules/{ruleId}', [AuditRuleController::class, 'update']);
            $router->delete('/server-nodes/{id}/audit-rules/{ruleId}', [AuditRuleController::class, 'destroy']);
            $router->get('/server-nodes/{id}/audit-logs', [AuditLogController::class, 'nodeLogs']);
            // Audit logs (user)
            $router->get('/audit-logs', [AuditLogController::class, 'myLogs']);
            // Access control management
            $router->get('/access/stats', [AccessControlController::class, 'myStats']);
            $router->get('/accessible-nodes', [AccessControlController::class, 'accessibleNodes']);
            $router->get('/server-nodes/{id}/access/stats', [AccessControlController::class, 'nodeStats']);
            $router->post('/server-nodes/{id}/share/user', [AccessControlController::class, 'shareWithUser']);
            $router->post('/server-nodes/{id}/share/group', [AccessControlController::class, 'shareWithGroup']);
            $router->post('/server-nodes/{id}/share/revoke', [AccessControlController::class, 'revoke']);
            // Coupon
            $router->post('/coupon/check', [CouponController::class, 'check']);
            // Gift Card
            $router->post('/gift-card/check', [GiftCardController::class, 'check']);
            $router->post('/gift-card/redeem', [GiftCardController::class, 'redeem']);
            $router->get('/gift-card/history', [GiftCardController::class, 'history']);
            $router->get('/gift-card/detail', [GiftCardController::class, 'detail']);
            $router->get('/gift-card/types', [GiftCardController::class, 'types']);
            // Telegram
            $router->get('/telegram/getBotInfo', [TelegramController::class, 'getBotInfo']);
            // Comm
            $router->get('/comm/config', [CommController::class, 'config']);
            $router->Post('/comm/getStripePublicKey', [CommController::class, 'getStripePublicKey']);
            // Knowledge
            $router->get('/knowledge/fetch', [KnowledgeController::class, 'fetch']);
            $router->get('/knowledge/getCategory', [KnowledgeController::class, 'getCategory']);
            // Stat
            $router->get('/stat/getTrafficLog', [StatController::class, 'getTrafficLog']);
            // Payment profiles (personal admin)
            $router->get('/payment-profiles/epay', [PaymentProfileController::class, 'showEpay']);
            $router->put('/payment-profiles/epay', [PaymentProfileController::class, 'upsertEpay']);
            // Sponsor (independent)
            $router->get('/sponsor/methods', [SponsorController::class, 'methods']);
            $router->post('/sponsor', [SponsorController::class, 'create']);
            $router->get('/sponsor/{tradeNo}', [SponsorController::class, 'detail']);
            $router->post('/sponsor/checkout', [SponsorController::class, 'checkout']);

            // Refunds (node plans)
            $router->get('/refunds', [RefundController::class, 'myRequests']);
            $router->post('/refunds', [RefundController::class, 'create']);
            $router->get('/refunds/{id}', [RefundController::class, 'detail']);
            $router->post('/refunds/{id}/evidence', [RefundController::class, 'addEvidence']);

            // Refund inbox for personal admins (node plan owners)
            $router->get('/node-admin/refunds', [RefundAdminController::class, 'inbox']);
            $router->get('/node-admin/refunds/{id}', [RefundAdminController::class, 'detail']);
            $router->post('/node-admin/refunds/{id}/approve', [RefundAdminController::class, 'approve']);
            $router->post('/node-admin/refunds/{id}/deny', [RefundAdminController::class, 'deny']);
            $router->post('/node-admin/refunds/{id}/dispute', [RefundAdminController::class, 'dispute']);
            $router->post('/node-admin/refunds/{id}/evidence', [RefundAdminController::class, 'addEvidence']);

            // Voting (all members)
            $router->get('/refund-votes', [RefundVoteController::class, 'open']);
            $router->get('/refund-votes/{id}', [RefundVoteController::class, 'detail']);
            $router->post('/refund-votes/{id}', [RefundVoteController::class, 'vote']);
            $router->post('/refund-votes/{id}/evidence', [RefundVoteController::class, 'addEvidence']);
        });
    }
}
