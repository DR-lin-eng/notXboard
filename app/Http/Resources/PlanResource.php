<?php


namespace App\Http\Resources;

use App\Models\Plan;
use App\Models\User;
use App\Services\PlanService;
use Illuminate\Http\Request;
use Illuminate\Http\Resources\Json\JsonResource;

class PlanResource extends JsonResource
{
    private const PRICE_MULTIPLIER = 100;

    /**
     * Transform the resource into an array.
     *
     * @return array<string, mixed>
     */
    public function toArray(Request $request): array
    {
        $visibilityScope = Plan::normalizeVisibilityScope($this->resource['visibility_scope'] ?? null);
        $shareToken = (string) ($this->resource['share_token'] ?? '');

        return [
            'id' => $this->resource['id'],
            'scope' => $this->resource['scope'] ?? Plan::SCOPE_LEGACY,
            'owner_user_id' => $this->resource['owner_user_id'] ?? null,
            'owner_display_name' => $this->resolveOwnerDisplayName(),
            'owner' => $this->resolveOwnerPayload(),
            'min_trust_level' => $this->resource['min_trust_level'] ?? null,
            'allow_trial' => $this->hasTrialQuota(),
            'free_quota_gb_by_trust_level' => $this->resource['free_quota_gb_by_trust_level'] ?? null,
            'paid_quota_gb' => (int) ($this->resource['transfer_enable'] ?? 0),
            'is_unlimited_traffic' => (bool) ($this->resource['is_unlimited_traffic'] ?? false),
            'node_ids' => $this->resource['node_ids'] ?? null,
            'visibility_scope' => $visibilityScope,
            'access_user_ids' => $this->resource['access_user_ids'] ?? [],
            'share_token' => $shareToken ?: null,
            'share_purchase_link' => $shareToken ? $this->buildSharePurchaseLink($request, $shareToken) : null,
            'group_id' => $this->resource['group_id'],
            'name' => $this->resource['name'],
            'tags' => $this->resource['tags'],
            'content' => $this->formatContent(),
            ...$this->getPeriodPrices(),
            'capacity_limit' => $this->getFormattedCapacityLimit(),
            'transfer_enable' => $this->resource['transfer_enable'],
            'speed_limit' => $this->resource['speed_limit'],
            'device_limit' => $this->resource['device_limit'],
            'show' => (bool) $this->resource['show'],
            'sell' => (bool) $this->resource['sell'],
            'renew' => (bool) $this->resource['renew'],
            'reset_traffic_method' => $this->resource['reset_traffic_method'],
            'sort' => $this->resource['sort'],
            'created_at' => $this->resource['created_at'],
            'updated_at' => $this->resource['updated_at']
        ];
    }

    /**
     * Get transformed period prices using Plan mapping
     *
     * @return array<string, float|null>
     */
    protected function getPeriodPrices(): array
    {
        return collect(Plan::LEGACY_PERIOD_MAPPING)
            ->mapWithKeys(function (string $newPeriod, string $legacyPeriod): array {
                $price = $this->resource['prices'][$newPeriod] ?? null;
                return [
                    $legacyPeriod => $price !== null
                        ? (float) $price * self::PRICE_MULTIPLIER
                        : null
                ];
            })
            ->all();
    }

    /**
     * Get formatted capacity limit value
     *
     * @return int|string|null
     */
    protected function getFormattedCapacityLimit(): int|string|null
    {
        $limit = $this->resource['capacity_limit'];

        return match (true) {
            $limit === null => null,
            $limit <= 0 => __('Sold out'),
            default => (int) $limit,
        };
    }

    /**
     * Format content with template variables
     *
     * @return string
     */
    protected function formatContent(): string
    {
        $content = $this->resource['content'] ?? '';
        
        $replacements = [
            '{{transfer}}' => $this->resource['transfer_enable'],
            '{{speed}}' => $this->resource['speed_limit'] === NULL ? __('No Limit') : $this->resource['speed_limit'],
            '{{devices}}' => $this->resource['device_limit'] === NULL ? __('No Limit') : $this->resource['device_limit'],
            '{{reset_method}}' => $this->getResetMethodText(),
        ];

        return str_replace(
            array_keys($replacements),
            array_values($replacements),
            $content
        );
    }

    /**
     * Get reset method text
     *
     * @return string
     */
    protected function getResetMethodText(): string
    {
        $method = $this->resource['reset_traffic_method'];
        
        if ($method === Plan::RESET_TRAFFIC_FOLLOW_SYSTEM) {
            $method = admin_setting('reset_traffic_method', Plan::RESET_TRAFFIC_MONTHLY);
        }
        return match ($method) {
            Plan::RESET_TRAFFIC_FIRST_DAY_MONTH => __('First Day of Month'),
            Plan::RESET_TRAFFIC_MONTHLY => __('Monthly'),
            Plan::RESET_TRAFFIC_NEVER => __('Never'),
            Plan::RESET_TRAFFIC_FIRST_DAY_YEAR => __('First Day of Year'),
            Plan::RESET_TRAFFIC_YEARLY => __('Yearly'),
            default => __('Monthly')
        };
    }

    private function hasTrialQuota(): bool
    {
        $quota = $this->resource['free_quota_gb_by_trust_level'] ?? null;
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

    private function buildSharePurchaseLink(Request $request, string $shareToken): string
    {
        $base = rtrim((string) (admin_setting('app_url') ?: $request->getSchemeAndHttpHost()), '/');
        return $base . '/app/#/plan-link/' . rawurlencode($shareToken);
    }

    private function resolveOwner(): ?User
    {
        $resource = $this->resource;
        if (!$resource || !is_object($resource)) {
            return null;
        }

        if (method_exists($resource, 'relationLoaded') && $resource->relationLoaded('owner')) {
            $owner = $resource->getRelation('owner');
            return $owner instanceof User ? $owner : null;
        }

        $owner = $resource->owner ?? null;
        return $owner instanceof User ? $owner : null;
    }

    private function resolveOwnerDisplayName(): string
    {
        $owner = $this->resolveOwner();
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

    private function resolveOwnerPayload(): ?array
    {
        $owner = $this->resolveOwner();
        if (!$owner) {
            return null;
        }

        return [
            'id' => (int) $owner->id,
            'email' => (string) ($owner->email ?? ''),
            'linux_do_username' => (string) ($owner->linux_do_username ?? ''),
            'linux_do_name' => (string) ($owner->linux_do_name ?? ''),
            'display_name' => $this->resolveOwnerDisplayName(),
        ];
    }
}
