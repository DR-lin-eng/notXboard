<?php

namespace App\Models;

use App\Utils\Helper;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use InvalidArgumentException;
use Carbon\Carbon;
use Illuminate\Database\Eloquent\Relations\HasOne;

/**
 * App\Models\Plan
 *
 * @property int $id
 * @property string $name 套餐名称
 * @property string $scope legacy|node
 * @property int|null $owner_user_id node plan publisher user id
 * @property int|null $min_trust_level Minimum Linux DO trust level required
 * @property array|null $free_quota_gb_by_trust_level Free quota per trust_level (GB/month)
 * @property array|null $node_ids server_nodes ids for node plan
 * @property int|null $group_id 权限组ID
 * @property int $transfer_enable 流量(KB)
 * @property bool $is_unlimited_traffic 是否无限流量
 * @property int|null $speed_limit 速度限制Mbps
 * @property bool $show 是否显示
 * @property bool $renew 是否允许续费
 * @property bool $sell 是否允许购买
 * @property array|null $prices 价格配置
 * @property array|null $tags 标签
 * @property int $sort 排序
 * @property string|null $content 套餐描述
 * @property int|null $reset_traffic_method 流量重置方式
 * @property int|null $capacity_limit 订阅人数限制
 * @property int|null $device_limit 设备数量限制
 * @property int $created_at
 * @property int $updated_at
 * 
 * @property-read ServerGroup|null $group 关联的权限组
 * @property-read \Illuminate\Database\Eloquent\Collection<int, Order> $order 关联的订单
 */
class Plan extends Model
{
    use HasFactory;

    protected $table = 'v2_plan';
    protected $dateFormat = 'U';

    // 定义流量重置方式
    public const RESET_TRAFFIC_FOLLOW_SYSTEM = null;    // 跟随系统设置
    public const RESET_TRAFFIC_FIRST_DAY_MONTH = 0;  // 每月1号
    public const RESET_TRAFFIC_MONTHLY = 1;          // 按月重置
    public const RESET_TRAFFIC_NEVER = 2;            // 不重置
    public const RESET_TRAFFIC_FIRST_DAY_YEAR = 3;   // 每年1月1日
    public const RESET_TRAFFIC_YEARLY = 4;           // 按年重置

    // 定义价格类型
    public const PRICE_TYPE_RESET_TRAFFIC = 'reset_traffic';  // 重置流量价格

    // 定义可用的订阅周期
    public const PERIOD_MONTHLY = 'monthly';
    public const PERIOD_QUARTERLY = 'quarterly';
    public const PERIOD_HALF_YEARLY = 'half_yearly';
    public const PERIOD_YEARLY = 'yearly';
    public const PERIOD_TWO_YEARLY = 'two_yearly';
    public const PERIOD_THREE_YEARLY = 'three_yearly';
    public const PERIOD_ONETIME = 'onetime';
    public const PERIOD_RESET_TRAFFIC = 'reset_traffic';

    // 定义旧版周期映射
    public const LEGACY_PERIOD_MAPPING = [
        'month_price' => self::PERIOD_MONTHLY,
        'quarter_price' => self::PERIOD_QUARTERLY,
        'half_year_price' => self::PERIOD_HALF_YEARLY,
        'year_price' => self::PERIOD_YEARLY,
        'two_year_price' => self::PERIOD_TWO_YEARLY,
        'three_year_price' => self::PERIOD_THREE_YEARLY,
        'onetime_price' => self::PERIOD_ONETIME,
        'reset_price' => self::PERIOD_RESET_TRAFFIC
    ];

    protected $fillable = [
        'scope',
        'owner_user_id',
        'min_trust_level',
        'free_quota_gb_by_trust_level',
        'node_ids',
        'group_id',
        'transfer_enable',
        'is_unlimited_traffic',
        'name',
        'speed_limit',
        'show',
        'visibility_scope',
        'access_user_ids',
        'share_token',
        'share_discount_type',
        'share_discount_value',
        'sort',
        'renew',
        'content',
        'prices',
        'reset_traffic_method',
        'capacity_limit',
        'sell',
        'device_limit',
        'tags'
    ];

    protected $casts = [
        'show' => 'boolean',
        'sell' => 'boolean',
        'renew' => 'boolean',
        'created_at' => 'timestamp',
        'updated_at' => 'timestamp',
        'group_id' => 'integer',
        'owner_user_id' => 'integer',
        'min_trust_level' => 'integer',
        'is_unlimited_traffic' => 'boolean',
        'prices' => 'array',
        'tags' => 'array',
        'reset_traffic_method' => 'integer',
        'node_ids' => 'array',
        'free_quota_gb_by_trust_level' => 'array',
        'access_user_ids' => 'array',
        'share_discount_type' => 'integer',
        'share_discount_value' => 'integer',
    ];

    public const SCOPE_LEGACY = 'legacy';
    public const SCOPE_NODE = 'node';

    public const VISIBILITY_PUBLIC = 'public';
    public const VISIBILITY_LINK_ONLY = 'link_only';
    public const VISIBILITY_ASSIGNED_ONLY = 'assigned_only';

    public const SHARE_DISCOUNT_NONE = 0;
    public const SHARE_DISCOUNT_FIXED = 1;
    public const SHARE_DISCOUNT_PERCENT = 2;

    /**
     * 获取所有可用的流量重置方式
     *
     * @return array
     */
    public static function getResetTrafficMethods(): array
    {
        return [
            self::RESET_TRAFFIC_FOLLOW_SYSTEM => '跟随系统设置',
            self::RESET_TRAFFIC_FIRST_DAY_MONTH => '每月1号',
            self::RESET_TRAFFIC_MONTHLY => '按月重置',
            self::RESET_TRAFFIC_NEVER => '不重置',
            self::RESET_TRAFFIC_FIRST_DAY_YEAR => '每年1月1日',
            self::RESET_TRAFFIC_YEARLY => '按年重置',
        ];
    }

    /**
     * 获取所有可用的订阅周期
     *
     * @return array
     */
    public static function getAvailablePeriods(): array
    {
        return [
            self::PERIOD_MONTHLY => [
                'name' => '月付',
                'days' => 30,
                'value' => 1
            ],
            self::PERIOD_QUARTERLY => [
                'name' => '季付',
                'days' => 90,
                'value' => 3
            ],
            self::PERIOD_HALF_YEARLY => [
                'name' => '半年付',
                'days' => 180,
                'value' => 6
            ],
            self::PERIOD_YEARLY => [
                'name' => '年付',
                'days' => 365,
                'value' => 12
            ],
            self::PERIOD_TWO_YEARLY => [
                'name' => '两年付',
                'days' => 730,
                'value' => 24
            ],
            self::PERIOD_THREE_YEARLY => [
                'name' => '三年付',
                'days' => 1095,
                'value' => 36
            ],
            self::PERIOD_ONETIME => [
                'name' => '一次性',
                'days' => -1,
                'value' => -1
            ],
            self::PERIOD_RESET_TRAFFIC => [
                'name' => '重置流量',
                'days' => -1,
                'value' => -1
            ],
        ];
    }

    /**
     * 获取指定周期的价格
     *
     * @param string $period
     * @return int|null
     */
    public function getPriceByPeriod(string $period): ?int
    {
        return $this->prices[$period] ?? null;
    }

    /**
     * 获取所有已设置价格的周期
     *
     * @return array
     */
    public function getActivePeriods(): array
    {
        return array_filter(
            self::getAvailablePeriods(),
            fn($period) => isset($this->prices[$period])
            && is_numeric($this->prices[$period])
            && (float) $this->prices[$period] >= 0,
            ARRAY_FILTER_USE_KEY
        );
    }

    /**
     * 设置指定周期的价格
     *
     * @param string $period
     * @param int $price
     * @return void
     * @throws InvalidArgumentException
     */
    public function setPeriodPrice(string $period, int $price): void
    {
        if (!array_key_exists($period, self::getAvailablePeriods())) {
            throw new InvalidArgumentException("Invalid period: {$period}");
        }

        $prices = $this->prices ?? [];
        $prices[$period] = $price;
        $this->prices = $prices;
    }

    /**
     * 移除指定周期的价格
     *
     * @param string $period
     * @return void
     */
    public function removePeriodPrice(string $period): void
    {
        $prices = $this->prices ?? [];
        unset($prices[$period]);
        $this->prices = $prices;
    }

    /**
     * 获取所有价格及其对应的周期信息
     *
     * @return array
     */
    public function getPriceList(): array
    {
        $prices = $this->prices ?? [];
        $periods = self::getAvailablePeriods();

        $priceList = [];
        foreach ($prices as $period => $price) {
            if (isset($periods[$period]) && is_numeric($price) && (float) $price >= 0) {
                $priceList[$period] = [
                    'period' => $periods[$period],
                    'price' => $price,
                    'average_price' => $periods[$period]['value'] > 0
                        ? round($price / $periods[$period]['value'], 2)
                        : $price
                ];
            }
        }

        return $priceList;
    }

    /**
     * 检查是否可以重置流量
     *
     * @return bool
     */
    public function canResetTraffic(): bool
    {
        return $this->reset_traffic_method !== self::RESET_TRAFFIC_NEVER
            && $this->getResetTrafficPrice() > 0;
    }

    /**
     * 获取重置流量的价格
     *
     * @return int
     */
    public function getResetTrafficPrice(): int
    {
        return $this->prices[self::PRICE_TYPE_RESET_TRAFFIC] ?? 0;
    }

    /**
     * 计算指定周期的有效天数
     *
     * @param string $period
     * @return int -1表示永久有效
     * @throws InvalidArgumentException
     */
    public static function getPeriodDays(string $period): int
    {
        $periods = self::getAvailablePeriods();
        if (!isset($periods[$period])) {
            throw new InvalidArgumentException("Invalid period: {$period}");
        }

        return $periods[$period]['days'];
    }

    /**
     * 检查周期是否有效
     *
     * @param string $period
     * @return bool
     */
    public static function isValidPeriod(string $period): bool
    {
        return array_key_exists($period, self::getAvailablePeriods());
    }

    public function users(): HasMany
    {
        return $this->hasMany(User::class);
    }

    public function owner(): BelongsTo
    {
        return $this->belongsTo(User::class, 'owner_user_id', 'id');
    }

    public function group(): HasOne
    {
        return $this->hasOne(ServerGroup::class, 'id', 'group_id');
    }

    public function orders(): HasMany
    {
        return $this->hasMany(Order::class);
    }

    /**
     * 设置流量重置方式
     *
     * @param int $method
     * @return void
     * @throws InvalidArgumentException
     */
    public function setResetTrafficMethod(int $method): void
    {
        if (!array_key_exists($method, self::getResetTrafficMethods())) {
            throw new InvalidArgumentException("Invalid reset traffic method: {$method}");
        }

        $this->reset_traffic_method = $method;
    }

    /**
     * 设置重置流量价格
     *
     * @param int $price
     * @return void
     */
    public function setResetTrafficPrice(int $price): void
    {
        $prices = $this->prices ?? [];
        $prices[self::PRICE_TYPE_RESET_TRAFFIC] = max(0, $price);
        $this->prices = $prices;
    }

    public function order(): HasMany
    {
        return $this->hasMany(Order::class);
    }

    public function ensureShareToken(bool $force = false): string
    {
        if (!$force && !empty($this->share_token)) {
            return (string) $this->share_token;
        }

        do {
            $token = Helper::randomChar(24);
            $exists = self::query()
                ->where('share_token', $token)
                ->exists();
        } while ($exists);

        $this->share_token = $token;
        return $token;
    }

    public static function normalizeVisibilityScope(?string $scope): string
    {
        $raw = strtolower(trim((string) $scope));
        return match ($raw) {
            self::VISIBILITY_LINK_ONLY => self::VISIBILITY_LINK_ONLY,
            self::VISIBILITY_ASSIGNED_ONLY => self::VISIBILITY_ASSIGNED_ONLY,
            default => self::VISIBILITY_PUBLIC,
        };
    }

    public function hasAssignedUser(int $userId): bool
    {
        if ($userId <= 0) {
            return false;
        }

        $ids = collect($this->access_user_ids ?? [])
            ->map(fn ($value) => (int) $value)
            ->filter(fn ($value) => $value > 0)
            ->unique()
            ->values()
            ->all();

        return in_array($userId, $ids, true);
    }
}
