<?php

namespace App\Services;

use App\Exceptions\ApiException;
use App\Models\Order;
use App\Models\Plan;
use App\Models\User;
use App\Models\UserPlanSubscription;
use Carbon\Carbon;
use Illuminate\Support\Collection;

class UserPlanSubscriptionService
{
    private const GB_TO_KB = 1073741824;
    private const UNLIMITED_TRAFFIC_ALLOWANCE_KB = 8000000000000000;

    private const PERIOD_MONTH_MAP = [
        Plan::PERIOD_MONTHLY => 1,
        Plan::PERIOD_QUARTERLY => 3,
        Plan::PERIOD_HALF_YEARLY => 6,
        Plan::PERIOD_YEARLY => 12,
        Plan::PERIOD_TWO_YEARLY => 24,
        Plan::PERIOD_THREE_YEARLY => 36,
    ];

    public function activateFromOrder(User $user, Order $order, Plan $plan): ?UserPlanSubscription
    {
        $periodKey = PlanService::getPeriodKey((string) $order->period);
        if ($periodKey === Plan::PERIOD_RESET_TRAFFIC) {
            return null;
        }

        $startedAt = (int) ($order->paid_at ?: time());
        $expiredAt = $this->calculateExpiredAt($periodKey, $startedAt);
        $trafficAllowanceKb = (bool) ($plan->is_unlimited_traffic ?? false)
            ? self::UNLIMITED_TRAFFIC_ALLOWANCE_KB
            : max(0, (int) $plan->transfer_enable) * self::GB_TO_KB;

        $subscription = UserPlanSubscription::query()->firstOrNew([
            'order_id' => $order->id,
        ]);

        $subscription->user_id = $user->id;
        $subscription->plan_id = $plan->id;
        $subscription->period = $periodKey;
        $subscription->traffic_allowance_kb = $trafficAllowanceKb;
        if (!$subscription->exists) {
            $subscription->used_traffic_kb = 0;
        }
        $subscription->started_at = $startedAt;
        $subscription->expired_at = $expiredAt;
        $subscription->status = UserPlanSubscription::STATUS_ACTIVE;
        $subscription->save();

        return $subscription;
    }

    public function hasActiveSubscription(User $user, int $planId): bool
    {
        return UserPlanSubscription::query()
            ->where('user_id', $user->id)
            ->where('plan_id', $planId)
            ->active(time())
            ->exists();
    }

    public function getActiveSubscriptions(User $user): Collection
    {
        $this->expireSubscriptions($user->id);

        return UserPlanSubscription::query()
            ->with(['plan'])
            ->where('user_id', $user->id)
            ->active(time())
            ->get();
    }

    public function revokeByOrder(User $user, int $orderId): void
    {
        UserPlanSubscription::query()
            ->where('user_id', $user->id)
            ->where('order_id', $orderId)
            ->where('status', UserPlanSubscription::STATUS_ACTIVE)
            ->update([
                'status' => UserPlanSubscription::STATUS_REVOKED,
                'updated_at' => time(),
            ]);
    }

    public function refreshUserEntitlements(User $user): void
    {
        $activeSubscriptions = $this->getActiveSubscriptions($user);

        if ($activeSubscriptions->isEmpty()) {
            $user->plan_id = null;
            $user->group_id = null;
            $user->transfer_enable = 0;
            $user->expired_at = 0;
            $user->speed_limit = null;
            $user->device_limit = null;
            $user->next_reset_at = null;
            app(NodePlanAccessService::class)->refreshAccessForUser($user);
            return;
        }

        $plans = $activeSubscriptions->pluck('plan')->filter();
        $primaryPlan = $this->pickPrimaryPlan($activeSubscriptions);
        $hasUnlimitedTraffic = $activeSubscriptions->contains(function (UserPlanSubscription $subscription) {
            return self::isUnlimitedTrafficAllowance((int) $subscription->traffic_allowance_kb);
        });

        $user->plan_id = $primaryPlan?->id;
        $user->group_id = $primaryPlan?->group_id;
        $user->transfer_enable = $hasUnlimitedTraffic
            ? self::UNLIMITED_TRAFFIC_ALLOWANCE_KB
            : (int) $activeSubscriptions->sum(function (UserPlanSubscription $subscription) {
                return max(0, (int) $subscription->traffic_allowance_kb);
            });
        $user->expired_at = $this->resolveExpiredAt($activeSubscriptions);
        $user->speed_limit = $this->resolveLimit($plans, 'speed_limit');
        $user->device_limit = $this->resolveLimit($plans, 'device_limit');

        if ($primaryPlan) {
            $user->setRelation('plan', $primaryPlan);
            $nextResetTime = app(TrafficResetService::class)->calculateNextResetTime($user);
            $user->next_reset_at = $nextResetTime?->timestamp;
        } else {
            $user->next_reset_at = null;
        }

        app(NodePlanAccessService::class)->refreshAccessForUser($user);
    }

    public static function unlimitedTrafficAllowanceKb(): int
    {
        return self::UNLIMITED_TRAFFIC_ALLOWANCE_KB;
    }

    public static function isUnlimitedTrafficAllowance(int $allowanceKb): bool
    {
        return $allowanceKb >= self::UNLIMITED_TRAFFIC_ALLOWANCE_KB;
    }

    private function calculateExpiredAt(string $periodKey, int $startedAt): ?int
    {
        if ($periodKey === Plan::PERIOD_ONETIME) {
            return null;
        }

        $months = self::PERIOD_MONTH_MAP[$periodKey] ?? null;
        if ($months === null) {
            throw new ApiException('无效的套餐周期');
        }

        return Carbon::createFromTimestamp($startedAt)->addMonths($months)->timestamp;
    }

    private function expireSubscriptions(int $userId): void
    {
        $now = time();

        UserPlanSubscription::query()
            ->where('user_id', $userId)
            ->where('status', UserPlanSubscription::STATUS_ACTIVE)
            ->whereNotNull('expired_at')
            ->where('expired_at', '<=', $now)
            ->update([
                'status' => UserPlanSubscription::STATUS_EXPIRED,
                'updated_at' => $now,
            ]);
    }

    private function resolveExpiredAt(Collection $subscriptions): ?int
    {
        $hasLifetime = $subscriptions->contains(fn (UserPlanSubscription $s) => $s->expired_at === null);
        if ($hasLifetime) {
            return null;
        }

        return (int) $subscriptions->max('expired_at');
    }

    private function pickPrimaryPlan(Collection $subscriptions): ?Plan
    {
        /** @var UserPlanSubscription|null $selected */
        $selected = $subscriptions
            ->sortByDesc(function (UserPlanSubscription $subscription) {
                $expireRank = $subscription->expired_at === null ? 9999999999 : (int) $subscription->expired_at;
                $orderRank = (int) $subscription->order_id;
                return sprintf('%010d%010d', $expireRank, $orderRank);
            })
            ->first();

        return $selected?->plan;
    }

    private function resolveLimit(Collection $plans, string $field): ?int
    {
        if ($plans->isEmpty()) {
            return null;
        }

        $hasUnlimited = $plans->contains(function (Plan $plan) use ($field) {
            return $plan->{$field} === null;
        });
        if ($hasUnlimited) {
            return null;
        }

        return (int) $plans->max(function (Plan $plan) use ($field) {
            return (int) $plan->{$field};
        });
    }
}
