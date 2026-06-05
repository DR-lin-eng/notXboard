<?php

namespace App\Services;

use App\Exceptions\ApiException;
use App\Models\InviteCode;
use App\Models\Order;
use App\Models\Plan;
use App\Models\User;
use App\Models\UserPlanSubscription;
use App\Services\Auth\InviteCodeService;
use Illuminate\Support\Str;

class InvitePlanGrantService
{
    public function grantFromInviteCode(User $user, ?InviteCode $inviteCodeModel): ?UserPlanSubscription
    {
        if (!$inviteCodeModel) {
            return null;
        }

        $inviteCodeService = app(InviteCodeService::class);
        $plan = $inviteCodeService->resolveAssignedPlan($inviteCodeModel);
        if (!$plan) {
            return null;
        }

        return $this->grant(
            user: $user,
            plan: $plan,
            period: (string) $inviteCodeModel->assigned_period,
            inviteUserId: (int) $inviteCodeModel->user_id,
            inviteCode: (string) $inviteCodeModel->code
        );
    }

    public function grant(
        User $user,
        Plan $plan,
        string $period,
        ?int $inviteUserId = null,
        ?string $inviteCode = null
    ): UserPlanSubscription {
        if (!Plan::isValidPeriod($period) || $period === Plan::PERIOD_RESET_TRAFFIC) {
            throw new ApiException('邀请套餐周期无效');
        }

        $order = Order::query()->create([
            'invite_user_id' => $inviteUserId,
            'user_id' => $user->id,
            'plan_id' => $plan->id,
            'coupon_id' => null,
            'payment_id' => null,
            'type' => Order::TYPE_NEW_PURCHASE,
            'period' => $period,
            'trade_no' => (string) Str::uuid(),
            'callback_no' => $inviteCode ? 'invite:' . $inviteCode : 'invite:auto',
            'total_amount' => 0,
            'handling_amount' => 0,
            'discount_amount' => 0,
            'surplus_amount' => null,
            'refund_amount' => null,
            'balance_amount' => 0,
            'surplus_order_ids' => null,
            'status' => Order::STATUS_COMPLETED,
            'commission_status' => 3,
            'commission_balance' => 0,
            'actual_commission_balance' => 0,
            'paid_at' => time(),
        ]);

        $subscriptionService = app(UserPlanSubscriptionService::class);
        $subscription = $subscriptionService->activateFromOrder($user, $order, $plan);
        if (!$subscription) {
            throw new ApiException('邀请套餐发放失败');
        }

        $subscriptionService->refreshUserEntitlements($user);
        $user->save();

        return $subscription;
    }
}
