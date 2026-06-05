<?php

namespace App\Services\Auth;

use App\Exceptions\ApiException;
use App\Models\InviteCode;
use App\Models\Plan;

class InviteCodeService
{
    public function resolveRegistrationInvite(?string $inviteCode, bool $lockForUpdate = false): ?InviteCode
    {
        $trimmedCode = trim((string) $inviteCode);
        if ($trimmedCode === '') {
            if ((int) admin_setting('invite_force', 0) === 1) {
                throw new ApiException(__('You must use the invitation code to register'));
            }

            return null;
        }

        $query = InviteCode::query()
            ->where('code', $trimmedCode)
            ->where('status', InviteCode::STATUS_UNUSED);

        if ($lockForUpdate) {
            $query->lockForUpdate();
        }

        $inviteCodeModel = $query->first();
        if (!$inviteCodeModel) {
            if ((int) admin_setting('invite_force', 0) === 1) {
                throw new ApiException(__('Invalid invitation code'));
            }

            return null;
        }

        return $inviteCodeModel;
    }

    public function consumeInviteCode(?InviteCode $inviteCodeModel): void
    {
        if (!$inviteCodeModel) {
            return;
        }

        if ((int) admin_setting('invite_never_expire', 0) === 1) {
            return;
        }

        $inviteCodeModel->status = InviteCode::STATUS_USED;
        $inviteCodeModel->save();
    }

    public function validateAssignablePlan(int $ownerUserId, int $planId, string $period): Plan
    {
        $normalizedPeriod = trim($period);
        if (!Plan::isValidPeriod($normalizedPeriod) || $normalizedPeriod === Plan::PERIOD_RESET_TRAFFIC) {
            throw new ApiException('邀请套餐周期无效');
        }

        $plan = Plan::query()
            ->whereKey($planId)
            ->where('scope', Plan::SCOPE_NODE)
            ->where('owner_user_id', $ownerUserId)
            ->first();

        if (!$plan) {
            throw new ApiException('只能选择你自己创建的节点套餐');
        }

        $prices = is_array($plan->prices) ? $plan->prices : [];
        if (!array_key_exists($normalizedPeriod, $prices)) {
            throw new ApiException('所选套餐未启用该周期');
        }

        return $plan;
    }

    public function resolveAssignedPlan(?InviteCode $inviteCodeModel): ?Plan
    {
        if (!$inviteCodeModel) {
            return null;
        }

        $planId = (int) ($inviteCodeModel->assigned_plan_id ?? 0);
        $period = trim((string) ($inviteCodeModel->assigned_period ?? ''));
        if ($planId <= 0 || $period === '') {
            return null;
        }

        try {
            return $this->validateAssignablePlan((int) $inviteCodeModel->user_id, $planId, $period);
        } catch (ApiException) {
            return null;
        }
    }
}
