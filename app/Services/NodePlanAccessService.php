<?php

namespace App\Services;

use App\Models\Plan;
use App\Models\ServerNode;
use App\Models\User;
use App\Models\UserPlanSubscription;
use Illuminate\Database\Query\Builder;
use Illuminate\Support\Facades\DB;

class NodePlanAccessService
{
    public function syncAccessForUserPlan(User $user, Plan $plan): void
    {
        $this->refreshAccessForUser($user);
    }

    public function refreshAccessForUser(User $user): void
    {
        $subscriptions = UserPlanSubscription::query()
            ->with(['plan:id,scope,node_ids'])
            ->where('user_id', $user->id)
            ->active(time())
            ->get();

        $planNodeIdsMap = [];
        foreach ($subscriptions as $subscription) {
            $plan = $subscription->plan;
            if (!$plan || ($plan->scope ?? Plan::SCOPE_LEGACY) !== Plan::SCOPE_NODE) {
                continue;
            }

            $nodeIds = collect($plan->node_ids ?? [])
                ->map(fn ($v) => (int) $v)
                ->filter(fn ($v) => $v > 0)
                ->unique()
                ->values()
                ->all();

            if (empty($nodeIds)) {
                continue;
            }

            foreach ($nodeIds as $nodeId) {
                $planNodeIdsMap[$plan->id][$nodeId] = true;
            }
        }

        $allNodeIds = [];
        foreach ($planNodeIdsMap as $nodeMap) {
            $allNodeIds = array_merge($allNodeIds, array_keys($nodeMap));
        }
        $allNodeIds = array_values(array_unique(array_map('intval', $allNodeIds)));

        $validNodeIdSet = [];
        if (!empty($allNodeIds)) {
            $validNodeIds = ServerNode::query()
                ->whereIn('id', $allNodeIds)
                ->pluck('id')
                ->map(fn ($v) => (int) $v)
                ->all();
            $validNodeIdSet = array_flip($validNodeIds);
        }

        DB::transaction(function () use ($user, $planNodeIdsMap, $validNodeIdSet) {
            DB::table('user_node_plan_access')
                ->where('user_id', $user->id)
                ->delete();

            if (empty($planNodeIdsMap) || empty($validNodeIdSet)) {
                return;
            }

            $now = now();
            $rows = [];

            foreach ($planNodeIdsMap as $planId => $nodeMap) {
                foreach (array_keys($nodeMap) as $nodeId) {
                    $nodeId = (int) $nodeId;
                    if (!isset($validNodeIdSet[$nodeId])) {
                        continue;
                    }

                    $rows[] = [
                        'user_id' => $user->id,
                        'node_id' => $nodeId,
                        'plan_id' => (int) $planId,
                        'granted_at' => $now,
                    ];
                }
            }

            if (!empty($rows)) {
                DB::table('user_node_plan_access')->insert($rows);
            }
        });
    }

    public function getActiveNodeIdsForUser(User $user): array
    {
        return $this->newActivePlanAccessQuery()
            ->where('user_node_plan_access.user_id', $user->id)
            ->distinct()
            ->pluck('user_node_plan_access.node_id')
            ->map(fn ($value) => (int) $value)
            ->values()
            ->all();
    }

    public function getActiveUserIdsForNode(int $nodeId): array
    {
        $nodeId = (int) $nodeId;
        if ($nodeId <= 0) {
            return [];
        }

        return $this->newActivePlanAccessQuery()
            ->where('user_node_plan_access.node_id', $nodeId)
            ->distinct()
            ->pluck('user_node_plan_access.user_id')
            ->map(fn ($v) => (int) $v)
            ->values()
            ->all();
    }

    public function userHasActiveAccessToNode(User $user, int $nodeId): bool
    {
        $nodeId = (int) $nodeId;
        if ($user->id <= 0 || $nodeId <= 0) {
            return false;
        }

        return $this->newActivePlanAccessQuery()
            ->where('user_node_plan_access.user_id', $user->id)
            ->where('user_node_plan_access.node_id', $nodeId)
            ->exists();
    }

    public function hasNodeAccessByActivePlans(User $user, int $nodeId): bool
    {
        return $this->userHasActiveAccessToNode($user, $nodeId);
    }

    private function newActivePlanAccessQuery(): Builder
    {
        $now = time();

        return DB::table('user_node_plan_access')
            ->join('user_plan_subscriptions', function ($join) {
                $join->on('user_plan_subscriptions.user_id', '=', 'user_node_plan_access.user_id')
                    ->on('user_plan_subscriptions.plan_id', '=', 'user_node_plan_access.plan_id');
            })
            ->where('user_plan_subscriptions.status', UserPlanSubscription::STATUS_ACTIVE)
            ->where(function ($query) use ($now) {
                $query->whereNull('user_plan_subscriptions.expired_at')
                    ->orWhere('user_plan_subscriptions.expired_at', '>', $now);
            });
    }
}
