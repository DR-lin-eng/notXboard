<?php

namespace App\Services;

use App\Models\Plan;
use App\Models\User;
use App\Models\UserPlanSubscription;
use App\Exceptions\ApiException;
use Illuminate\Database\Eloquent\Collection;

class PlanService
{
    public Plan $plan;

    public function __construct(Plan $plan)
    {
        $this->plan = $plan;
    }

    /**
     * 获取所有可销售的订阅计划列表
     * 条件：show 和 sell 为 true，且容量充足
     *
     * @return Collection
     */
    public function getAvailablePlans(?User $user = null): Collection
    {
        return Plan::query()
            ->with(['owner:id,email,linux_do_username,linux_do_name'])
            ->where('show', true)
            ->where('sell', true)
            ->where(function ($query) {
                $query->whereNull('visibility_scope')
                    ->orWhere('visibility_scope', Plan::VISIBILITY_PUBLIC);
            })
            ->orderBy('sort')
            ->get()
            ->filter(function ($plan) {
                return $this->hasCapacity($plan);
            })
            ->when($user, function (Collection $plans) use ($user) {
                return $plans->filter(function (Plan $plan) use ($user) {
                    if ($plan->min_trust_level === null) {
                        return true;
                    }
                    return (int) ($user->trust_level ?? 0) >= (int) $plan->min_trust_level;
                });
            });
    }

    /**
     * 获取指定订阅计划的可用状态
     * 条件：renew 和 sell 为 true
     *
     * @param int $planId
     * @return Plan|null
     */
    public function getAvailablePlan(int $planId): ?Plan
    {
        return Plan::where('id', $planId)
            ->where('sell', true)
            ->where('renew', true)
            ->first();
    }

    /**
     * 检查指定计划是否可用于指定用户
     * 
     * @param Plan $plan
     * @param User $user
     * @return bool
     */
    public function isPlanAvailableForUser(Plan $plan, User $user, ?string $purchaseToken = null): bool
    {
        if ($plan->min_trust_level !== null && (int) ($user->trust_level ?? 0) < (int) $plan->min_trust_level) {
            return false;
        }

        // 如果有该套餐的有效订阅实例，则按续费规则
        if ($this->hasActiveSubscription($user, $plan->id)) {
            return $plan->renew;
        }

        if (!$plan->sell || !$this->hasCapacity($plan)) {
            return false;
        }

        $scope = Plan::normalizeVisibilityScope($plan->visibility_scope ?? null);
        if ($scope === Plan::VISIBILITY_LINK_ONLY) {
            $token = trim((string) $purchaseToken);
            return $token !== '' && $token === (string) ($plan->share_token ?? '');
        }

        if ($scope === Plan::VISIBILITY_ASSIGNED_ONLY) {
            return $plan->hasAssignedUser((int) $user->id);
        }

        return (bool) $plan->show;
    }

    public function validatePurchase(User $user, string $period, array $context = []): void
    {
        if (!$this->plan) {
            throw new ApiException(__('Subscription plan does not exist'));
        }

        if ($this->plan->min_trust_level !== null && (int) ($user->trust_level ?? 0) < (int) $this->plan->min_trust_level) {
            throw new ApiException(__('Insufficient trust level'));
        }

        // 转换周期格式为新版格式
        $periodKey = self::getPeriodKey($period);
        $price = $this->plan->prices[$periodKey] ?? null;

        if ($price === null) {
            throw new ApiException(__('This payment period cannot be purchased, please choose another period'));
        }

        if ($periodKey === Plan::PERIOD_RESET_TRAFFIC) {
            $this->validateResetTrafficPurchase($user);
            return;
        }

        if (!$this->hasActiveSubscription($user, $this->plan->id) && !$this->hasCapacity($this->plan)) {
            throw new ApiException(__('Current product is sold out'));
        }

        $purchaseToken = trim((string) ($context['purchase_token'] ?? ''));
        $this->validatePlanAvailability($user, $purchaseToken);
    }

    /**
     * 智能转换周期格式为新版格式
     * 如果是新版格式直接返回，如果是旧版格式则转换为新版格式
     *
     * @param string $period
     * @return string
     */
    public static function getPeriodKey(string $period): string
    {
        // 如果是新版格式直接返回
        if (in_array($period, self::getNewPeriods())) {
            return $period;
        }

        // 如果是旧版格式则转换为新版格式
        return Plan::LEGACY_PERIOD_MAPPING[$period] ?? $period;
    }
    /**
     * 只能转换周期格式为旧版本
     */
    public static function convertToLegacyPeriod(string $period): string
    {
        $flippedMapping = array_flip(Plan::LEGACY_PERIOD_MAPPING);
        return $flippedMapping[$period] ?? $period;
    }

    /**
     * 获取所有支持的新版周期格式
     *
     * @return array
     */
    public static function getNewPeriods(): array
    {
        return array_values(Plan::LEGACY_PERIOD_MAPPING);
    }

    /**
     * 获取旧版周期格式
     *
     * @param string $period
     * @return string
     */
    public static function getLegacyPeriod(string $period): string
    {
        $flipped = array_flip(Plan::LEGACY_PERIOD_MAPPING);
        return $flipped[$period] ?? $period;
    }

    protected function validateResetTrafficPurchase(User $user): void
    {
        if (!$this->hasActiveSubscription($user, $this->plan->id)) {
            throw new ApiException(__('Subscription has expired or no active subscription, unable to purchase Data Reset Package'));
        }
    }

    protected function validatePlanAvailability(User $user, ?string $purchaseToken = null): void
    {
        $hasActiveCurrentPlan = $this->hasActiveSubscription($user, $this->plan->id);

        if ($hasActiveCurrentPlan && !$this->plan->renew) {
            throw new ApiException(__('This subscription cannot be renewed, please change to another subscription'));
        }

        if ($hasActiveCurrentPlan) {
            return;
        }

        if (!$this->plan->sell) {
            throw new ApiException(__('This subscription has expired, please change to another subscription'));
        }

        $scope = Plan::normalizeVisibilityScope($this->plan->visibility_scope ?? null);
        if ($scope === Plan::VISIBILITY_LINK_ONLY) {
            $token = trim((string) $purchaseToken);
            if ($token !== '' && $token === (string) ($this->plan->share_token ?? '')) {
                return;
            }
        } elseif ($scope === Plan::VISIBILITY_ASSIGNED_ONLY && $this->plan->hasAssignedUser((int) $user->id)) {
            return;
        } elseif ((bool) $this->plan->show) {
            return;
        }

        throw new ApiException(__('This subscription has been sold out, please choose another subscription'));
    }

    public function hasCapacity(Plan $plan): bool
    {
        if ($plan->capacity_limit === null) {
            return true;
        }

        $activeUserCount = UserPlanSubscription::query()
            ->where('plan_id', $plan->id)
            ->active(time())
            ->distinct('user_id')
            ->count('user_id');

        return ($plan->capacity_limit - $activeUserCount) > 0;
    }

    private function hasActiveSubscription(User $user, int $planId): bool
    {
        return UserPlanSubscription::query()
            ->where('user_id', $user->id)
            ->where('plan_id', $planId)
            ->active(time())
            ->exists();
    }

    public function getAvailablePeriods(Plan $plan): array
    {
        return array_filter(
            $plan->getActivePeriods(),
            fn($period) => isset($plan->prices[$period])
                && is_numeric($plan->prices[$period])
                && (float) $plan->prices[$period] >= 0
        );
    }

    public function canResetTraffic(Plan $plan): bool
    {
        return $plan->reset_traffic_method !== Plan::RESET_TRAFFIC_NEVER
            && $plan->getResetTrafficPrice() > 0;
    }
}
