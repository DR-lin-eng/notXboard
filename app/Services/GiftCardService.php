<?php

namespace App\Services;

use App\Exceptions\ApiException;
use App\Models\GiftCardCode;
use App\Models\GiftCardTemplate;
use App\Models\GiftCardUsage;
use App\Models\Plan;
use App\Models\User;
use App\Models\TrafficResetLog;
use Illuminate\Support\Facades\DB;
use Illuminate\Support\Facades\Log;

class GiftCardService
{
    protected readonly GiftCardCode $code;
    protected readonly GiftCardTemplate $template;
    protected ?User $user = null;

    public function __construct(string $code)
    {
        $this->code = GiftCardCode::where('code', $code)->first()
            ?? throw new ApiException('兑换码不存在');

        $this->template = $this->code->template;
    }

    /**
     * 设置使用用户
     */
    public function setUser(User $user): self
    {
        $this->user = $user;
        return $this;
    }

    /**
     * 验证兑换码
     */
    public function validate(): self
    {
        $this->validateIsActive();

        $eligibility = $this->checkUserEligibility();
        if (!$eligibility['can_redeem']) {
            throw new ApiException($eligibility['reason']);
        }

        return $this;
    }

    /**
     * 验证礼品卡本身是否可用 (不检查用户条件)
     * @throws ApiException
     */
    public function validateIsActive(): self
    {
        if (!$this->template->isAvailable()) {
            throw new ApiException('该礼品卡类型已停用');
        }

        if (!$this->code->isAvailable()) {
            throw new ApiException('兑换码不可用：' . $this->code->status_name);
        }
        return $this;
    }

    /**
     * 检查用户是否满足兑换条件 (不抛出异常)
     */
    public function checkUserEligibility(): array
    {
        if (!$this->user) {
            return [
                'can_redeem' => false,
                'reason' => '用户信息未提供'
            ];
        }

        if (!$this->template->checkUserConditions($this->user)) {
            return [
                'can_redeem' => false,
                'reason' => '您不满足此礼品卡的使用条件'
            ];
        }

        if (!$this->template->checkUsageLimit($this->user)) {
            return [
                'can_redeem' => false,
                'reason' => '您已达到此礼品卡的使用限制'
            ];
        }

        return ['can_redeem' => true, 'reason' => null];
    }

    /**
     * 使用礼品卡
     */
    public function redeem(array $options = []): array
    {
        if (!$this->user) {
            throw new ApiException('未设置使用用户');
        }

        return DB::transaction(function () use ($options) {
            $actualRewards = $this->template->calculateActualRewards($this->user);

            if ($this->template->type === GiftCardTemplate::TYPE_MYSTERY) {
                $this->code->setActualRewards($actualRewards);
            }

            $this->giveRewards($actualRewards);

            $inviteRewards = null;
            if ($this->user->invite_user_id && isset($actualRewards['invite_reward_rate'])) {
                $inviteRewards = $this->giveInviteRewards($actualRewards);
            }

            $this->code->markAsUsed($this->user);

            GiftCardUsage::createRecord(
                $this->code,
                $this->user,
                $actualRewards,
                array_merge($options, [
                    'invite_rewards' => $inviteRewards,
                    'multiplier' => $this->calculateMultiplier(),
                ])
            );

            return [
                'rewards' => $actualRewards,
                'invite_rewards' => $inviteRewards,
                'code' => $this->code->code,
                'template_name' => $this->template->name,
            ];
        });
    }

    /**
     * 发放奖励
     */
    protected function giveRewards(array $rewards): void
    {
        $userService = app(UserService::class);

        if (isset($rewards['balance']) && $rewards['balance'] > 0) {
            if (!$userService->addBalance($this->user->id, $rewards['balance'])) {
                throw new ApiException('余额发放失败');
            }
        }

        if (isset($rewards['transfer_enable']) && $rewards['transfer_enable'] > 0) {
            $this->user->transfer_enable = ($this->user->transfer_enable ?? 0) + $rewards['transfer_enable'];
        }

        if (isset($rewards['device_limit']) && $rewards['device_limit'] > 0) {
            $this->user->device_limit = ($this->user->device_limit ?? 0) + $rewards['device_limit'];
        }

        if (isset($rewards['reset_package']) && $rewards['reset_package']) {
            if ($this->user->plan_id) {
                app(TrafficResetService::class)->performReset($this->user, TrafficResetLog::SOURCE_GIFT_CARD);
            }
        }

        if (isset($rewards['plan_id'])) {
            $plan = Plan::find($rewards['plan_id']);
            if ($plan) {
                $userService->assignPlan(
                    $this->user,
                    $plan,
                    $rewards['plan_validity_days'] ?? null
                );
            }
        } else {
            // 只有在不是套餐卡的情况下，才处理独立的有效期奖励
            if (isset($rewards['expire_days']) && $rewards['expire_days'] > 0) {
                $userService->extendSubscription($this->user, $rewards['expire_days']);
            }
        }

        // 保存用户更改
        if (!$this->user->save()) {
            throw new ApiException('用户信息更新失败');
        }
    }

    /**
     * 发放邀请人奖励
     */
    protected function giveInviteRewards(array $rewards): ?array
    {
        if (!$this->user->invite_user_id) {
            return null;
        }

        $inviteUser = User::find($this->user->invite_user_id);
        if (!$inviteUser) {
            return null;
        }

        $rate = $rewards['invite_reward_rate'] ?? 0.2;
        $inviteRewards = [];

        $userService = app(UserService::class);

        // 邀请人余额奖励
        if (isset($rewards['balance']) && $rewards['balance'] > 0) {
            $inviteBalance = intval($rewards['balance'] * $rate);
            if ($inviteBalance > 0) {
                $userService->addBalance($inviteUser->id, $inviteBalance);
                $inviteRewards['balance'] = $inviteBalance;
            }
        }

        // 邀请人流量奖励
        if (isset($rewards['transfer_enable']) && $rewards['transfer_enable'] > 0) {
            $inviteTransfer = intval($rewards['transfer_enable'] * $rate);
            if ($inviteTransfer > 0) {
                $inviteUser->transfer_enable = ($inviteUser->transfer_enable ?? 0) + $inviteTransfer;
                $inviteUser->save();
                $inviteRewards['transfer_enable'] = $inviteTransfer;
            }
        }

        return $inviteRewards;
    }

    /**
     * 计算倍率
     */
    protected function calculateMultiplier(): float
    {
        return $this->getFestivalBonus();
    }

    /**
     * 获取节日加成倍率
     */
    private function getFestivalBonus(): float
    {
        $festivalConfig = $this->template->special_config ?? [];
        $now = time();

        if (
            isset($festivalConfig['start_time'], $festivalConfig['end_time']) &&
            $now >= $festivalConfig['start_time'] &&
            $now <= $festivalConfig['end_time']
        ) {
            return $festivalConfig['festival_bonus'] ?? 1.0;
        }

        return 1.0;
    }

    /**
     * 获取兑换码信息（不包含敏感信息）
     */
    public function getCodeInfo(): array
    {
        $info = [
            'code' => $this->code->code,
            'template' => [
                'name' => $this->template->name,
                'description' => $this->template->description,
                'type' => $this->template->type,
                'type_name' => $this->template->type_name,
                'icon' => $this->template->icon,
                'background_image' => $this->template->background_image,
                'theme_color' => $this->template->theme_color,
            ],
            'status' => $this->code->status,
            'status_name' => $this->code->status_name,
            'expires_at' => $this->code->expires_at,
            'usage_count' => $this->code->usage_count,
            'max_usage' => $this->code->max_usage,
        ];
        if ($this->template->type === GiftCardTemplate::TYPE_PLAN) {
            $plan = Plan::find($this->code->template->rewards['plan_id']);
            if ($plan) {
                $info['plan_info'] = $this->serializePlan($plan);
            }
        }
        return $info;
    }

    /**
     * 预览奖励（不实际发放）
     */
    public function previewRewards(): array
    {
        if (!$this->user) {
            throw new ApiException('未设置使用用户');
        }

        return $this->template->calculateActualRewards($this->user);
    }

    /**
     * 获取兑换码
     */
    public function getCode(): GiftCardCode
    {
        return $this->code;
    }

    /**
     * 获取模板
     */
    public function getTemplate(): GiftCardTemplate
    {
        return $this->template;
    }

    /**
     * 记录日志
     */
    protected function logUsage(string $action, array $data = []): void
    {
        Log::info('礼品卡使用记录', [
            'action' => $action,
            'code' => $this->code->code,
            'template_id' => $this->template->id,
            'user_id' => $this->user?->id,
            'data' => $data,
            'ip' => request()->ip(),
            'user_agent' => request()->userAgent(),
        ]);
    }

    private function serializePlan(Plan $plan): array
    {
        $visibilityScope = Plan::normalizeVisibilityScope($plan->visibility_scope ?? null);
        $shareToken = (string) ($plan->share_token ?? '');

        return [
            'id' => $plan->id,
            'scope' => $plan->scope ?? Plan::SCOPE_LEGACY,
            'owner_user_id' => $plan->owner_user_id ?? null,
            'owner_display_name' => $this->resolvePlanOwnerDisplayName($plan),
            'owner' => $this->resolvePlanOwnerPayload($plan),
            'min_trust_level' => $plan->min_trust_level ?? null,
            'allow_trial' => $this->planHasTrialQuota($plan),
            'free_quota_gb_by_trust_level' => $plan->free_quota_gb_by_trust_level ?? null,
            'paid_quota_gb' => (int) ($plan->transfer_enable ?? 0),
            'is_unlimited_traffic' => (bool) ($plan->is_unlimited_traffic ?? false),
            'node_ids' => $plan->node_ids ?? null,
            'visibility_scope' => $visibilityScope,
            'access_user_ids' => $plan->access_user_ids ?? [],
            'share_token' => $shareToken !== '' ? $shareToken : null,
            'share_purchase_link' => $shareToken !== '' ? $this->buildPlanSharePurchaseLink($shareToken) : null,
            'group_id' => $plan->group_id,
            'name' => $plan->name,
            'tags' => $plan->tags,
            'content' => $this->formatPlanContent($plan),
            ...$this->planPeriodPrices($plan),
            'capacity_limit' => $this->formatPlanCapacityLimit($plan),
            'transfer_enable' => $plan->transfer_enable,
            'speed_limit' => $plan->speed_limit,
            'device_limit' => $plan->device_limit,
            'show' => (bool) $plan->show,
            'sell' => (bool) $plan->sell,
            'renew' => (bool) $plan->renew,
            'reset_traffic_method' => $plan->reset_traffic_method,
            'sort' => $plan->sort,
            'created_at' => $plan->created_at,
            'updated_at' => $plan->updated_at,
        ];
    }

    private function planPeriodPrices(Plan $plan): array
    {
        $result = [];
        foreach (Plan::LEGACY_PERIOD_MAPPING as $legacyPeriod => $newPeriod) {
            $price = $plan->prices[$newPeriod] ?? null;
            $result[$legacyPeriod] = $price !== null ? (float) $price * 100 : null;
        }

        return $result;
    }

    private function formatPlanCapacityLimit(Plan $plan): int|string|null
    {
        $limit = $plan->capacity_limit;

        return match (true) {
            $limit === null => null,
            $limit <= 0 => __('Sold out'),
            default => (int) $limit,
        };
    }

    private function formatPlanContent(Plan $plan): string
    {
        $content = $plan->content ?? '';

        $replacements = [
            '{{transfer}}' => $plan->transfer_enable,
            '{{speed}}' => $plan->speed_limit === null ? __('No Limit') : $plan->speed_limit,
            '{{devices}}' => $plan->device_limit === null ? __('No Limit') : $plan->device_limit,
            '{{reset_method}}' => $this->planResetMethodText($plan),
        ];

        return str_replace(array_keys($replacements), array_values($replacements), $content);
    }

    private function planResetMethodText(Plan $plan): string
    {
        $method = $plan->reset_traffic_method;
        if ($method === Plan::RESET_TRAFFIC_FOLLOW_SYSTEM) {
            $method = admin_setting('reset_traffic_method', Plan::RESET_TRAFFIC_MONTHLY);
        }

        return match ($method) {
            Plan::RESET_TRAFFIC_FIRST_DAY_MONTH => __('First Day of Month'),
            Plan::RESET_TRAFFIC_MONTHLY => __('Monthly'),
            Plan::RESET_TRAFFIC_NEVER => __('Never'),
            Plan::RESET_TRAFFIC_FIRST_DAY_YEAR => __('First Day of Year'),
            Plan::RESET_TRAFFIC_YEARLY => __('Yearly'),
            default => __('Monthly'),
        };
    }

    private function planHasTrialQuota(Plan $plan): bool
    {
        $quota = $plan->free_quota_gb_by_trust_level ?? null;
        if (!is_array($quota)) {
            return false;
        }

        foreach ($quota as $value) {
            if (is_numeric($value) && (float) $value > 0) {
                return true;
            }
        }

        return false;
    }

    private function buildPlanSharePurchaseLink(string $shareToken): string
    {
        $request = request();
        $requestBase = method_exists($request, 'getSchemeAndHttpHost')
            ? (string) $request->getSchemeAndHttpHost()
            : '';
        $base = rtrim((string) (admin_setting('app_url') ?: $requestBase), '/');

        return $base . '/app/#/plan-link/' . rawurlencode($shareToken);
    }

    private function resolvePlanOwner(Plan $plan): ?User
    {
        $owner = $plan->owner ?? null;
        return $owner instanceof User ? $owner : null;
    }

    private function resolvePlanOwnerDisplayName(Plan $plan): string
    {
        $owner = $this->resolvePlanOwner($plan);
        if (!$owner) {
            return '未知';
        }

        $linuxDoName = trim((string) ($owner->linux_do_name ?? ''));
        if ($linuxDoName !== '') {
            return $linuxDoName;
        }

        $linuxDoUsername = trim((string) ($owner->linux_do_username ?? ''));
        if ($linuxDoUsername !== '') {
            return $linuxDoUsername;
        }

        $email = trim((string) ($owner->email ?? ''));
        if ($email !== '') {
            return $email;
        }

        return '用户#' . (int) ($owner->id ?? 0);
    }

    private function resolvePlanOwnerPayload(Plan $plan): ?array
    {
        $owner = $this->resolvePlanOwner($plan);
        if (!$owner) {
            return null;
        }

        return [
            'id' => (int) $owner->id,
            'email' => (string) ($owner->email ?? ''),
            'linux_do_username' => (string) ($owner->linux_do_username ?? ''),
            'linux_do_name' => (string) ($owner->linux_do_name ?? ''),
            'display_name' => $this->resolvePlanOwnerDisplayName($plan),
        ];
    }
}
