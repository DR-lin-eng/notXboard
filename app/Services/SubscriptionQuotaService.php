<?php

namespace App\Services;

use App\Models\Plan;
use App\Models\User;
use App\Models\UserPlanSubscription;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Support\Collection;

class SubscriptionQuotaService
{
    public function allocateNodeTrafficToSubscriptions(int $userId, int $nodeId, int $trafficKb): void
    {
        if ($userId <= 0 || $nodeId <= 0 || $trafficKb <= 0) {
            return;
        }

        $this->allocateBatchNodeTrafficToSubscriptions($nodeId, [
            $userId => $trafficKb,
        ]);
    }

    public function allocateBatchNodeTrafficToSubscriptions(int $nodeId, array $trafficByUserId): void
    {
        $normalizedTraffic = collect($trafficByUserId)
            ->mapWithKeys(function ($trafficKb, $userId) {
                $userId = (int) $userId;
                $trafficKb = (int) $trafficKb;

                if ($userId <= 0 || $trafficKb <= 0) {
                    return [];
                }

                return [$userId => $trafficKb];
            })
            ->all();

        if (empty($normalizedTraffic)) {
            return;
        }

        $subscriptions = $this->getActiveNodeSubscriptionsForUsers(array_keys($normalizedTraffic), $nodeId, true)
            ->sortBy(function (UserPlanSubscription $subscription) {
                $expireAt = $subscription->expired_at === null ? PHP_INT_MAX : (int) $subscription->expired_at;
                return sprintf('%020d-%020d-%020d', (int) $subscription->user_id, $expireAt, (int) $subscription->id);
            })
            ->values();

        if ($subscriptions->isEmpty()) {
            return;
        }

        $incrementsBySubscriptionId = [];
        $subscriptionsByUserId = $subscriptions->groupBy(fn (UserPlanSubscription $subscription) => (int) $subscription->user_id);

        foreach ($subscriptionsByUserId as $userId => $userSubscriptions) {
            $remaining = (int) ($normalizedTraffic[(int) $userId] ?? 0);
            if ($remaining <= 0) {
                continue;
            }

            $firstSubscriptionId = (int) ($userSubscriptions->first()?->id ?? 0);

            foreach ($userSubscriptions as $subscription) {
                if ($remaining <= 0) {
                    break;
                }

                $allowance = max(0, (int) $subscription->traffic_allowance_kb);
                $used = max(0, (int) $subscription->used_traffic_kb);

                if (UserPlanSubscriptionService::isUnlimitedTrafficAllowance($allowance)) {
                    $incrementsBySubscriptionId[(int) $subscription->id] = ($incrementsBySubscriptionId[(int) $subscription->id] ?? 0) + $remaining;
                    $remaining = 0;
                    break;
                }

                $room = max(0, $allowance - $used);
                if ($room <= 0) {
                    continue;
                }

                $consume = min($remaining, $room);
                $incrementsBySubscriptionId[(int) $subscription->id] = ($incrementsBySubscriptionId[(int) $subscription->id] ?? 0) + $consume;
                $remaining -= $consume;
            }

            if ($remaining > 0 && $firstSubscriptionId > 0) {
                $incrementsBySubscriptionId[$firstSubscriptionId] = ($incrementsBySubscriptionId[$firstSubscriptionId] ?? 0) + $remaining;
            }
        }

        if (!empty($incrementsBySubscriptionId)) {
            $this->incrementSubscriptionUsage($incrementsBySubscriptionId);
        }
    }

    public function userHasRemainingQuotaForNode(User $user, int $nodeId): bool
    {
        if ($user->id <= 0 || $nodeId <= 0) {
            return false;
        }

        $unlimitedAllowanceKb = UserPlanSubscriptionService::unlimitedTrafficAllowanceKb();

        return $this->newActiveNodeSubscriptionQuery($nodeId)
            ->where('user_plan_subscriptions.user_id', $user->id)
            ->where(function (Builder $query) use ($unlimitedAllowanceKb) {
                $query->where('user_plan_subscriptions.traffic_allowance_kb', '>=', $unlimitedAllowanceKb)
                    ->orWhereColumn('user_plan_subscriptions.traffic_allowance_kb', '>', 'user_plan_subscriptions.used_traffic_kb');
            })
            ->exists();
    }

    public function getUserPlanUsageSummary(User $user): array
    {
        $subscriptions = UserPlanSubscription::query()
            ->with(['plan'])
            ->where('user_id', $user->id)
            ->active(time())
            ->orderByDesc('started_at')
            ->get();

        $items = $subscriptions->map(function (UserPlanSubscription $subscription) {
            $plan = $subscription->plan;
            $rawAllowance = max(0, (int) $subscription->traffic_allowance_kb);
            $isUnlimitedTraffic = UserPlanSubscriptionService::isUnlimitedTrafficAllowance($rawAllowance);
            $allowance = $isUnlimitedTraffic
                ? UserPlanSubscriptionService::unlimitedTrafficAllowanceKb()
                : $rawAllowance;
            $used = max(0, (int) $subscription->used_traffic_kb);
            $remaining = $isUnlimitedTraffic
                ? UserPlanSubscriptionService::unlimitedTrafficAllowanceKb()
                : max(0, $allowance - $used);

            return [
                'subscription_id' => (int) $subscription->id,
                'order_id' => (int) $subscription->order_id,
                'plan_id' => (int) $subscription->plan_id,
                'plan_name' => $plan?->name ?: ('套餐 #' . $subscription->plan_id),
                'scope' => $plan?->scope ?? Plan::SCOPE_LEGACY,
                'period' => (string) $subscription->period,
                'started_at' => (int) $subscription->started_at,
                'expired_at' => $subscription->expired_at !== null ? (int) $subscription->expired_at : null,
                'traffic_allowance_kb' => $allowance,
                'used_traffic_kb' => $used,
                'remaining_traffic_kb' => $remaining,
                'usage_percent' => $isUnlimitedTraffic ? null : ($allowance > 0 ? round(min(100, ($used / $allowance) * 100), 2) : 0),
                'is_unlimited_traffic' => $isUnlimitedTraffic,
                'node_ids' => is_array($plan?->node_ids) ? array_values(array_map('intval', $plan->node_ids)) : [],
            ];
        })->values()->all();

        $hasUnlimitedTraffic = collect($items)->contains(fn (array $item) => (bool) ($item['is_unlimited_traffic'] ?? false));
        $totalAllowance = $hasUnlimitedTraffic
            ? UserPlanSubscriptionService::unlimitedTrafficAllowanceKb()
            : (int) collect($items)->sum('traffic_allowance_kb');
        $totalUsed = (int) collect($items)->sum('used_traffic_kb');
        $totalRemaining = $hasUnlimitedTraffic
            ? UserPlanSubscriptionService::unlimitedTrafficAllowanceKb()
            : max(0, $totalAllowance - $totalUsed);

        return [
            'items' => $items,
            'total_allowance_kb' => $totalAllowance,
            'total_used_kb' => $totalUsed,
            'total_remaining_kb' => $totalRemaining,
            'has_unlimited_traffic' => $hasUnlimitedTraffic,
        ];
    }

    public function getUserIdsWithRemainingQuotaForNode(int $nodeId, ?array $userIds = null): array
    {
        $userIds = $this->normalizePositiveIntList($userIds);
        if ($userIds === []) {
            return [];
        }

        $rows = $this->newActiveNodeSubscriptionQuery($nodeId)
            ->when($userIds !== null, fn (Builder $query) => $query->whereIn('user_plan_subscriptions.user_id', $userIds))
            ->get([
                'user_plan_subscriptions.user_id',
                'user_plan_subscriptions.traffic_allowance_kb',
                'user_plan_subscriptions.used_traffic_kb',
            ]);

        $matchedUserIds = [];
        foreach ($rows as $row) {
            $allowance = max(0, (int) $row->traffic_allowance_kb);
            $used = max(0, (int) $row->used_traffic_kb);
            if (
                UserPlanSubscriptionService::isUnlimitedTrafficAllowance($allowance)
                || $allowance > $used
            ) {
                $matchedUserIds[(int) $row->user_id] = true;
            }
        }

        return array_map('intval', array_keys($matchedUserIds));
    }

    public function getNodeIdsWithRemainingQuotaForUser(User|int $user, ?array $nodeIds = null): array
    {
        $userId = $user instanceof User ? (int) $user->id : (int) $user;
        if ($userId <= 0) {
            return [];
        }

        $nodeIds = $this->normalizePositiveIntList($nodeIds);
        if ($nodeIds === []) {
            return [];
        }

        $rows = $this->newActiveNodePlanQueryForUser($userId)
            ->get([
                'user_node_plan_access.node_id',
                'user_plan_subscriptions.traffic_allowance_kb',
                'user_plan_subscriptions.used_traffic_kb',
            ]);

        $matchedNodeIds = [];
        $filterSet = $nodeIds === null ? null : array_fill_keys($nodeIds, true);

        foreach ($rows as $row) {
            $allowance = max(0, (int) $row->traffic_allowance_kb);
            $used = max(0, (int) $row->used_traffic_kb);
            if (
                !UserPlanSubscriptionService::isUnlimitedTrafficAllowance($allowance)
                && $allowance <= $used
            ) {
                continue;
            }

            $nodeId = (int) ($row->node_id ?? 0);
            if ($nodeId <= 0) {
                continue;
            }
            if ($filterSet !== null && !isset($filterSet[$nodeId])) {
                continue;
            }
            $matchedNodeIds[$nodeId] = true;
        }

        return array_map('intval', array_keys($matchedNodeIds));
    }

    private function getActiveNodeSubscriptionsForNode(int $userId, int $nodeId, bool $lockForUpdate = false): Collection
    {
        if ($userId <= 0) {
            return collect();
        }

        return $this->newActiveNodeSubscriptionQuery($nodeId, $lockForUpdate)
            ->where('user_plan_subscriptions.user_id', $userId)
            ->select('user_plan_subscriptions.*')
            ->get();
    }

    private function getActiveNodeSubscriptionsForUsers(array $userIds, int $nodeId, bool $lockForUpdate = false): Collection
    {
        $userIds = $this->normalizePositiveIntList($userIds);
        if (empty($userIds)) {
            return collect();
        }

        return $this->newActiveNodeSubscriptionQuery($nodeId, $lockForUpdate)
            ->whereIn('user_plan_subscriptions.user_id', $userIds)
            ->select('user_plan_subscriptions.*')
            ->get();
    }

    private function newActiveNodeSubscriptionQuery(int $nodeId, bool $lockForUpdate = false): Builder
    {
        $query = UserPlanSubscription::query()
            ->join('user_node_plan_access', function ($join) {
                $join->on('user_node_plan_access.user_id', '=', 'user_plan_subscriptions.user_id')
                    ->on('user_node_plan_access.plan_id', '=', 'user_plan_subscriptions.plan_id');
            })
            ->where('user_node_plan_access.node_id', (int) $nodeId)
            ->where('user_plan_subscriptions.status', UserPlanSubscription::STATUS_ACTIVE)
            ->where(function (Builder $builder) {
                $builder->whereNull('user_plan_subscriptions.expired_at')
                    ->orWhere('user_plan_subscriptions.expired_at', '>', time());
            });

        if ($lockForUpdate) {
            $query->lockForUpdate();
        }

        return $query;
    }

    private function newActiveNodePlanQueryForUser(int $userId): Builder
    {
        return UserPlanSubscription::query()
            ->join('user_node_plan_access', function ($join) {
                $join->on('user_node_plan_access.user_id', '=', 'user_plan_subscriptions.user_id')
                    ->on('user_node_plan_access.plan_id', '=', 'user_plan_subscriptions.plan_id');
            })
            ->where('user_plan_subscriptions.user_id', $userId)
            ->where('user_plan_subscriptions.status', UserPlanSubscription::STATUS_ACTIVE)
            ->where(function (Builder $builder) {
                $builder->whereNull('user_plan_subscriptions.expired_at')
                    ->orWhere('user_plan_subscriptions.expired_at', '>', time());
            });
    }

    private function normalizePositiveIntList(?array $values): ?array
    {
        if ($values === null) {
            return null;
        }

        $normalized = collect($values)
            ->map(fn ($value) => (int) $value)
            ->filter(fn ($value) => $value > 0)
            ->unique()
            ->values()
            ->all();

        return empty($normalized) ? [] : $normalized;
    }

    private function incrementSubscriptionUsage(array $incrementsBySubscriptionId): void
    {
        $incrementsBySubscriptionId = collect($incrementsBySubscriptionId)
            ->mapWithKeys(function ($trafficKb, $subscriptionId) {
                $subscriptionId = (int) $subscriptionId;
                $trafficKb = (int) $trafficKb;

                if ($subscriptionId <= 0 || $trafficKb <= 0) {
                    return [];
                }

                return [$subscriptionId => $trafficKb];
            })
            ->all();

        if (empty($incrementsBySubscriptionId)) {
            return;
        }

        $table = (new UserPlanSubscription())->getTable();
        $subscriptionIds = array_keys($incrementsBySubscriptionId);
        $bindings = [];
        $caseSegments = [];

        foreach ($incrementsBySubscriptionId as $subscriptionId => $trafficKb) {
            $caseSegments[] = 'WHEN ? THEN ?';
            $bindings[] = (int) $subscriptionId;
            $bindings[] = (int) $trafficKb;
        }

        $updatedAt = time();
        $wherePlaceholders = implode(', ', array_fill(0, count($subscriptionIds), '?'));
        $bindings[] = $updatedAt;
        foreach ($subscriptionIds as $subscriptionId) {
            $bindings[] = (int) $subscriptionId;
        }

        $sql = "UPDATE {$table}
            SET used_traffic_kb = used_traffic_kb + CASE id " . implode(' ', $caseSegments) . " ELSE 0 END,
                updated_at = ?
            WHERE id IN ({$wherePlaceholders})";

        \Illuminate\Support\Facades\DB::update($sql, $bindings);
    }
}
