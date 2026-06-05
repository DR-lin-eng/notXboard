<?php

namespace App\Services;

use App\Models\User;
use App\Models\ServerNode;
use App\Models\UserPlanSubscription;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Support\Collection;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Log;

class AccessControlService
{
    /**
     * 检查用户是否可以访问节点
     */
    public function canUserAccessNode(User $user, ServerNode $node): bool
    {
        return $node->canUserAccess($user);
    }

    /**
     * 根据信任等级获取用户组
     */
    public function getUserGroupByTrustLevel(int $trustLevel): string
    {
        return match ($trustLevel) {
            0 => 'new_user',
            1 => 'basic_user',
            2 => 'member',
            3 => 'regular',
            4 => 'leader',
            default => 'unknown',
        };
    }

    /**
     * 更新用户访问权限（当信任等级变化时）
     */
    public function updateUserAccessRights(User $user): void
    {
        // 获取所有基于信任等级的节点访问权限
        $accessibleNodes = ServerNode::whereJsonContains('access_control->min_trust_level', '<=', $user->trust_level)
            ->orWhereHas('authorizedUsers', function ($query) use ($user) {
                $query->where('user_id', $user->id);
            })
            ->get();

        Log::info('Updated user access rights', [
            'user_id' => $user->id,
            'trust_level' => $user->trust_level,
            'accessible_nodes_count' => $accessibleNodes->count()
        ]);
    }

    /**
     * 获取用户可访问的节点
     */
    public function getAccessibleNodesForUser(User $user): Collection
    {
        // 站长拥有全节点可用权
        if ((bool) $user->is_super_admin) {
            return ServerNode::where('status', ServerNode::STATUS_ACTIVE)
                ->with(['owner'])
                ->get()
                ->values();
        }

        $sharedNodeIds = DB::table('user_node_access')
            ->where('user_id', $user->id)
            ->pluck('node_id')
            ->map(fn ($value) => (int) $value)
            ->all();
        $planNodeIds = app(NodePlanAccessService::class)->getActiveNodeIdsForUser($user);
        $quotaNodeIds = empty($planNodeIds)
            ? []
            : app(SubscriptionQuotaService::class)->getNodeIdsWithRemainingQuotaForUser($user, $planNodeIds);
        $sharedNodeSet = array_fill_keys($sharedNodeIds, true);
        $quotaNodeSet = array_fill_keys($quotaNodeIds, true);
        $blacklistedNodeSet = array_fill_keys(
            DB::table('user_node_blacklist')
                ->where('user_id', $user->id)
                ->pluck('node_id')
                ->map(fn ($value) => (int) $value)
                ->all(),
            true
        );
        $isUserEligible = !(bool) $user->banned
            && ($user->expired_at === null || (int) $user->expired_at >= time());

        $query = ServerNode::query()
            ->where('status', ServerNode::STATUS_ACTIVE)
            ->where(function ($q) use ($user, $sharedNodeIds, $quotaNodeIds) {
                $q->where('user_id', $user->id);
                if (!empty($sharedNodeIds)) {
                    $q->orWhereIn('id', $sharedNodeIds);
                }
                if (!empty($quotaNodeIds)) {
                    $q->orWhereIn('id', $quotaNodeIds);
                }
                $q->orWhere(function ($groupQuery) use ($user) {
                    $groupQuery->whereNotNull('access_control->min_trust_level')
                        ->where('access_control->min_trust_level', '<=', (int) $user->trust_level);
                });
            })
            ->with(['owner']);

        return $query->get()
            ->filter(function (ServerNode $node) use ($user, $sharedNodeSet, $quotaNodeSet, $blacklistedNodeSet, $isUserEligible) {
                if ((int) $node->user_id === (int) $user->id) {
                    return true;
                }

                if (!$isUserEligible || isset($blacklistedNodeSet[(int) $node->id])) {
                    return false;
                }

                $minTrustLevel = data_get($node->access_control ?? [], 'min_trust_level');
                $hasIndividualAccess = isset($sharedNodeSet[(int) $node->id]);
                $hasGroupAccess = $minTrustLevel !== null
                    && (int) $user->trust_level >= (int) $minTrustLevel;
                $hasPlanAccess = isset($quotaNodeSet[(int) $node->id]);

                return $hasIndividualAccess || $hasGroupAccess || $hasPlanAccess;
            })
            ->values();
    }

    /**
     * 轻量判断用户是否至少拥有一个可访问节点，用于订阅前置校验。
     */
    public function hasAccessibleNodesForUser(User $user): bool
    {
        if ((bool) $user->is_super_admin) {
            return true;
        }

        return $this->getAccessibleNodesForUser($user)->isNotEmpty();
    }

    /**
     * 获取节点的可访问用户
     */
    public function getAccessibleUsersForNode(ServerNode $node): Collection
    {
        return $this->newAccessibleUsersQuery($node)
            ->select([
                'id',
                'uuid',
                'subscription_credential_version',
                'trust_level',
                'is_silenced',
                'expired_at',
                'is_super_admin',
            ])
            ->with('individualLimit')
            ->get()
            ->values();
    }

    /**
     * 获取节点可访问用户 ID（用于 push/alive 快速鉴权与过滤）
     */
    public function getAccessibleUserIdsForNode(
        ServerNode $node,
        ?int $limit = null,
        ?array $candidateUserIds = null
    ): array {
        $normalizedCandidates = $this->normalizePositiveIntList($candidateUserIds);
        if ($normalizedCandidates === []) {
            return [];
        }

        $query = $this->newAccessibleUsersQuery($node, $normalizedCandidates)
            ->select('id')
            ->orderBy('id');

        if ($limit !== null && $limit > 0) {
            $query->limit((int) $limit);
        }

        return $query->pluck('id')
            ->map(fn ($value) => (int) $value)
            ->unique()
            ->values()
            ->all();
    }

    /**
     * 按节点权限过滤候选用户 ID
     */
    public function filterAccessibleUserIdsForNode(ServerNode $node, array $candidateUserIds): array
    {
        return $this->getAccessibleUserIdsForNode($node, null, $candidateUserIds);
    }

    private function newAccessibleUsersQuery(ServerNode $node, ?array $candidateUserIds = null): Builder
    {
        $userTable = (new User())->getTable();
        $minTrustLevel = data_get($node->access_control ?? [], 'min_trust_level');
        $now = time();
        $unlimitedAllowance = UserPlanSubscriptionService::unlimitedTrafficAllowanceKb();

        return User::query()
            ->where('banned', 0)
            ->when(
                is_array($candidateUserIds),
                fn (Builder $query) => $query->whereIn('id', $candidateUserIds)
            )
            ->where(function (Builder $query) use ($node) {
                // 节点所有者、站长无需依赖套餐到期时间
                $query->where('id', (int) $node->user_id)
                    ->orWhere('is_super_admin', 1)
                    ->orWhere(function (Builder $sub) {
                        $sub->where('expired_at', '>=', time())
                            ->orWhereNull('expired_at');
                    });
            })
            ->where(function (Builder $query) use ($node, $minTrustLevel, $userTable, $now, $unlimitedAllowance) {
                $query->where('id', (int) $node->user_id)
                    ->orWhere('is_super_admin', 1)
                    ->orWhere(function (Builder $sub) use ($node, $minTrustLevel, $userTable, $now, $unlimitedAllowance) {
                        $sub->whereNotExists(function ($blacklist) use ($node, $userTable) {
                            $blacklist->selectRaw('1')
                                ->from('user_node_blacklist')
                                ->where('user_node_blacklist.node_id', (int) $node->id)
                                ->whereColumn('user_node_blacklist.user_id', "{$userTable}.id");
                        });

                        $sub->where(function (Builder $access) use ($node, $minTrustLevel, $userTable, $now, $unlimitedAllowance) {
                            $access->whereExists(function ($individualShare) use ($node, $userTable) {
                                $individualShare->selectRaw('1')
                                    ->from('user_node_access')
                                    ->where('user_node_access.node_id', (int) $node->id)
                                    ->whereColumn('user_node_access.user_id', "{$userTable}.id");
                            });

                            if ($minTrustLevel !== null) {
                                $access->orWhere('trust_level', '>=', (int) $minTrustLevel);
                            }

                            $access->orWhereExists(function ($quota) use ($node, $userTable, $now, $unlimitedAllowance) {
                                $quota->selectRaw('1')
                                    ->from('user_node_plan_access')
                                    ->join('user_plan_subscriptions', function ($join) {
                                        $join->on('user_plan_subscriptions.user_id', '=', 'user_node_plan_access.user_id')
                                            ->on('user_plan_subscriptions.plan_id', '=', 'user_node_plan_access.plan_id');
                                    })
                                    ->whereColumn('user_node_plan_access.user_id', "{$userTable}.id")
                                    ->where('user_node_plan_access.node_id', (int) $node->id)
                                    ->where('user_plan_subscriptions.status', UserPlanSubscription::STATUS_ACTIVE)
                                    ->where(function ($timeQuery) use ($now) {
                                        $timeQuery->whereNull('user_plan_subscriptions.expired_at')
                                            ->orWhere('user_plan_subscriptions.expired_at', '>', $now);
                                    })
                                    ->where(function ($quotaQuery) use ($unlimitedAllowance) {
                                        $quotaQuery->where('user_plan_subscriptions.traffic_allowance_kb', '>=', $unlimitedAllowance)
                                            ->orWhereColumn('user_plan_subscriptions.traffic_allowance_kb', '>', 'user_plan_subscriptions.used_traffic_kb');
                                    });
                            });
                        });
                    });
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

    /**
     * 为节点分享给指定用户
     */
    public function shareNodeWithUser(ServerNode $node, User $targetUser): void
    {
        // 检查是否已经授权
        if (!$node->authorizedUsers()->where('user_id', $targetUser->id)->exists()) {
            $node->authorizedUsers()->attach($targetUser->id, [
                'access_type' => 'individual',
                'granted_at' => now(),
            ]);

            Log::info('Node shared with user', [
                'node_id' => $node->id,
                'target_user_id' => $targetUser->id,
                'granted_by' => $node->user_id
            ]);
        }
    }

    /**
     * 为节点分享给用户组（基于信任等级）
     */
    public function shareNodeWithGroup(ServerNode $node, int $minTrustLevel): void
    {
        $accessControl = $node->access_control ?? [];
        $accessControl['min_trust_level'] = $minTrustLevel;

        $node->update(['access_control' => $accessControl]);

        Log::info('Node shared with group', [
            'node_id' => $node->id,
            'min_trust_level' => $minTrustLevel,
            'granted_by' => $node->user_id
        ]);
    }

    /**
     * 撤销用户对节点的访问权限
     */
    public function revokeNodeAccess(ServerNode $node, User $user): void
    {
        // 移除个人授权
        $node->authorizedUsers()->detach($user->id);

        Log::info('Node access revoked', [
            'node_id' => $node->id,
            'user_id' => $user->id,
            'revoked_by' => $node->user_id
        ]);
    }

    /**
     * 设置节点访问控制
     */
    public function setAccessControl(ServerNode $node, array $accessControl): void
    {
        DB::transaction(function () use ($node, $accessControl) {
            // 更新访问控制配置
            $node->update(['access_control' => $accessControl]);

            // 处理个人授权用户
            if (isset($accessControl['authorized_users']) && is_array($accessControl['authorized_users'])) {
                // 先清除现有的个人授权
                $node->authorizedUsers()->detach();

                // 添加新的个人授权
                foreach ($accessControl['authorized_users'] as $userId) {
                    $node->authorizedUsers()->attach($userId, [
                        'access_type' => 'individual',
                        'granted_at' => now(),
                    ]);
                }
            }
        });

        Log::info('Access control updated', [
            'node_id' => $node->id,
            'access_control' => $accessControl,
            'updated_by' => $node->user_id
        ]);
    }

    /**
     * 获取用户的访问统计
     */
    public function getUserAccessStats(User $user): array
    {
        $ownedNodes = ServerNode::where('user_id', $user->id)->count();
        $accessibleNodes = $this->getAccessibleNodesForUser($user)->count();
        $sharedNodes = $accessibleNodes - $ownedNodes;

        return [
            'owned_nodes' => $ownedNodes,
            'accessible_nodes' => $accessibleNodes,
            'shared_nodes' => $sharedNodes,
            'trust_level' => $user->trust_level,
            'user_group' => $this->getUserGroupByTrustLevel($user->trust_level),
        ];
    }

    /**
     * 获取节点的访问统计
     */
    public function getNodeAccessStats(ServerNode $node): array
    {
        $accessibleUsers = $this->getAccessibleUsersForNode($node);
        $authorizedUsers = $node->authorizedUsers()->count();
        $accessControl = $node->access_control ?? [];

        return [
            'total_accessible_users' => $accessibleUsers->count(),
            'individually_authorized_users' => $authorizedUsers,
            'min_trust_level' => $accessControl['min_trust_level'] ?? null,
            'access_type' => $this->getNodeAccessType($node),
        ];
    }

    /**
     * 获取节点的访问类型
     */
    private function getNodeAccessType(ServerNode $node): string
    {
        $accessControl = $node->access_control ?? [];
        $hasMinTrustLevel = isset($accessControl['min_trust_level']);
        $hasAuthorizedUsers = $node->authorizedUsers()->exists();

        if ($hasMinTrustLevel && $hasAuthorizedUsers) {
            return 'mixed';
        } elseif ($hasMinTrustLevel) {
            return 'group_based';
        } elseif ($hasAuthorizedUsers) {
            return 'individual_based';
        } else {
            return 'owner_only';
        }
    }

    /**
     * 批量更新用户访问权限（用于信任等级变化）
     */
    public function batchUpdateUserAccessRights(Collection $users): void
    {
        foreach ($users as $user) {
            $this->updateUserAccessRights($user);
        }

        Log::info('Batch updated user access rights', [
            'user_count' => $users->count()
        ]);
    }

    /**
     * 验证访问控制配置
     */
    public function validateAccessControl(array $accessControl): array
    {
        $errors = [];

        if (isset($accessControl['min_trust_level'])) {
            $minTrustLevel = $accessControl['min_trust_level'];
            if (!is_int($minTrustLevel) || $minTrustLevel < 0 || $minTrustLevel > 4) {
                $errors[] = 'min_trust_level must be an integer between 0 and 4';
            }
        }

        if (isset($accessControl['authorized_users'])) {
            if (!is_array($accessControl['authorized_users'])) {
                $errors[] = 'authorized_users must be an array';
            } else {
                foreach ($accessControl['authorized_users'] as $userId) {
                    if (!is_int($userId) || $userId <= 0) {
                        $errors[] = 'authorized_users must contain valid user IDs';
                        break;
                    }
                }
            }
        }

        return $errors;
    }

    /**
     * 获取用户可以管理的节点（仅自己的节点）
     */
    public function getManageableNodesForUser(User $user): Collection
    {
        return ServerNode::where('user_id', $user->id)
            ->with(['auditRules', 'onlineSessions', 'authorizedUsers'])
            ->orderBy('created_at', 'desc')
            ->get();
    }

    /**
     * 检查用户是否可以管理节点
     */
    public function canUserManageNode(User $user, ServerNode $node): bool
    {
        // 只有节点所有者可以管理节点
        return $node->user_id === $user->id;
    }

    /**
     * 获取共享给用户的节点列表
     */
    public function getSharedNodesForUser(User $user): Collection
    {
        return $this->getAccessibleNodesForUser($user)
            ->filter(fn (ServerNode $node) => (int) $node->user_id !== (int) $user->id)
            ->values();
    }
}
