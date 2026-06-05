<?php

namespace App\Models;

use App\Utils\Helper;
use Illuminate\Foundation\Auth\User as Authenticatable;
use Laravel\Sanctum\HasApiTokens;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use Illuminate\Database\Eloquent\Relations\BelongsToMany;
use Illuminate\Database\Eloquent\Relations\HasOne;
use Illuminate\Support\Carbon;

/**
 * App\Models\User
 *
 * @property int $id 用户ID
 * @property string $email 邮箱
 * @property string $password 密码
 * @property string|null $password_algo 加密方式
 * @property string|null $password_salt 加密盐
 * @property string $token 邀请码
 * @property string $uuid
 * @property int|null $invite_user_id 邀请人
 * @property int|null $plan_id 订阅ID
 * @property int|null $group_id 权限组ID
 * @property int|null $transfer_enable 流量(KB)
 * @property int|null $speed_limit 限速Mbps
 * @property int|null $u 上行流量
 * @property int|null $d 下行流量
 * @property int|null $banned 是否封禁
 * @property string|null $ban_reason 当前封禁原因
 * @property int|null $banned_at 封禁时间
 * @property int|null $banned_by_admin_id 封禁操作管理员 ID
 * @property int|null $remind_expire 到期提醒
 * @property int|null $remind_traffic 流量提醒
 * @property int|null $expired_at 过期时间
 * @property int|null $balance 余额
 * @property int|null $commission_balance 佣金余额
 * @property float $commission_rate 返佣比例
 * @property int|null $commission_type 返佣类型
 * @property int|null $device_limit 设备限制数量
 * @property int $concurrent_ip_limit 跨节点并发IP限制（0表示默认策略）
 * @property int|null $discount 折扣
 * @property int|null $last_login_at 最后登录时间
 * @property int|null $parent_id 父账户ID
 * @property int|null $is_admin 是否管理员
 * @property int|null $next_reset_at 下次流量重置时间
 * @property int|null $last_reset_at 上次流量重置时间
 * @property int|null $telegram_id Telegram ID
 * @property int $reset_count 流量重置次数
 * @property string|null $linux_do_id Linux DO 用户 ID
 * @property string|null $linux_do_username Linux DO 用户名
 * @property string|null $linux_do_name Linux DO 显示名称
 * @property string|null $linux_do_avatar 头像模板 URL
 * @property int $trust_level 信任等级 (0-4)
 * @property bool $is_silenced 是否被禁言
 * @property array|null $external_ids 外部 ID
 * @property string|null $api_key 个人 API 密钥
 * @property bool $is_super_admin 是否为超级管理员
 * @property string|null $oauth_provider OAuth 提供商
 * @property string|null $oauth_access_token OAuth 访问令牌
 * @property string|null $oauth_refresh_token OAuth 刷新令牌
 * @property string|null $oauth_expires_at 令牌过期时间
 * @property string|null $subscribe_path 订阅路径校验字符串
 * @property string|null $subscribe_key 订阅查询参数名称
 * @property string|null $subscribe_salt 订阅随机参数名称
 * @property int $subscription_credential_version
 * @property int|null $last_subscription_credential_rotation_at
 * @property int $created_at
 * @property int $updated_at
 * @property bool $commission_auto_check 是否自动计算佣金
 *
 * @property-read User|null $invite_user 邀请人信息
 * @property-read \App\Models\Plan|null $plan 用户订阅计划
 * @property-read ServerGroup|null $group 权限组
 * @property-read \Illuminate\Database\Eloquent\Collection<int, InviteCode> $codes 邀请码列表
 * @property-read \Illuminate\Database\Eloquent\Collection<int, Order> $orders 订单列表
 * @property-read \Illuminate\Database\Eloquent\Collection<int, StatUser> $stat 统计信息
 * @property-read \Illuminate\Database\Eloquent\Collection<int, Ticket> $tickets 工单列表
 * @property-read \Illuminate\Database\Eloquent\Collection<int, TrafficResetLog> $trafficResetLogs 流量重置记录
 * @property-read User|null $parent 父账户
 * @property-read string $subscribe_url 订阅链接（动态生成）
 */
class User extends Authenticatable
{
    use HasApiTokens;
    use HasFactory;
    protected $table = 'v2_user';
    protected $dateFormat = 'U';
    protected $guarded = ['id'];
    protected $casts = [
        'created_at' => 'timestamp',
        'updated_at' => 'timestamp',
        'banned' => 'boolean',
        'is_admin' => 'boolean',
        'is_staff' => 'boolean',
        'is_super_admin' => 'boolean',
        'is_silenced' => 'boolean',
        'remind_expire' => 'boolean',
        'remind_traffic' => 'boolean',
        'commission_auto_check' => 'boolean',
        'commission_rate' => 'float',
        'concurrent_ip_limit' => 'integer',
        'next_reset_at' => 'timestamp',
        'last_reset_at' => 'timestamp',
        'external_ids' => 'array',
        'subscription_credential_version' => 'integer',
        'last_subscription_credential_rotation_at' => 'timestamp',
        'banned_at' => 'timestamp',
    ];
    protected $hidden = ['password', 'oauth_access_token', 'oauth_refresh_token'];

    public const COMMISSION_TYPE_SYSTEM = 0;
    public const COMMISSION_TYPE_PERIOD = 1;
    public const COMMISSION_TYPE_ONETIME = 2;

    // 获取邀请人信息
    public function invite_user(): BelongsTo
    {
        return $this->belongsTo(self::class, 'invite_user_id', 'id');
    }

    /**
     * 获取用户订阅计划
     * @return \Illuminate\Database\Eloquent\Relations\BelongsTo
     */
    public function plan(): BelongsTo
    {
        return $this->belongsTo(Plan::class, 'plan_id', 'id');
    }

    public function group(): BelongsTo
    {
        return $this->belongsTo(ServerGroup::class, 'group_id', 'id');
    }

    // 获取用户邀请码列表
    public function codes(): HasMany
    {
        return $this->hasMany(InviteCode::class, 'user_id', 'id');
    }

    public function orders(): HasMany
    {
        return $this->hasMany(Order::class, 'user_id', 'id');
    }

    public function planSubscriptions(): HasMany
    {
        return $this->hasMany(UserPlanSubscription::class, 'user_id', 'id');
    }

    public function stat(): HasMany
    {
        return $this->hasMany(StatUser::class, 'user_id', 'id');
    }

    // 关联工单列表
    public function tickets(): HasMany
    {
        return $this->hasMany(Ticket::class, 'user_id', 'id');
    }

    public function parent(): BelongsTo
    {
        return $this->belongsTo(self::class, 'parent_id', 'id');
    }

    /**
     * 关联流量重置记录
     */
    public function trafficResetLogs(): HasMany
    {
        return $this->hasMany(TrafficResetLog::class, 'user_id', 'id');
    }

    /**
     * 关联用户拥有的服务器节点
     */
    public function serverNodes(): HasMany
    {
        return $this->hasMany(ServerNode::class, 'user_id', 'id');
    }

    /**
     * 关联用户可访问的节点
     */
    public function accessibleNodes()
    {
        return $this->belongsToMany(ServerNode::class, 'user_node_access', 'user_id', 'node_id')
                    ->withPivot('access_type', 'granted_at');
    }

    /**
     * 关联用户在线会话
     */
    public function onlineSessions(): HasMany
    {
        return $this->hasMany(UserOnlineSession::class, 'user_id', 'id');
    }

    /**
     * 关联用户审计日志
     */
    public function auditLogs(): HasMany
    {
        return $this->hasMany(AuditLog::class, 'user_id', 'id');
    }

    /**
     * 关联用户流量记录
     */
    public function trafficRecords(): HasMany
    {
        return $this->hasMany(NodeTrafficRecord::class, 'user_id', 'id');
    }

    public function trafficUsageLogs(): HasMany
    {
        return $this->hasMany(UserTrafficUsageLog::class, 'user_id', 'id');
    }

    public function tcpingAgents(): HasMany
    {
        return $this->hasMany(TcpingAgent::class, 'user_id', 'id');
    }

    public function tcpingAlerts(): HasMany
    {
        return $this->hasMany(TcpingAlert::class, 'user_id', 'id');
    }

    public function banRecords(): HasMany
    {
        return $this->hasMany(UserBanRecord::class, 'user_id', 'id');
    }

    public function riskReviews(): HasMany
    {
        return $this->hasMany(UserRiskReview::class, 'user_id', 'id');
    }

    /**
     * 关联用户个人限制
     */
    public function individualLimit()
    {
        return $this->hasOne(UserIndividualLimit::class, 'user_id', 'id');
    }

    /**
     * 检查用户是否处于活跃状态
     */
    public function isActive(): bool
    {
        return !$this->banned && 
               ($this->expired_at === null || $this->expired_at > time()) &&
               $this->plan_id !== null;
    }

    public function getActiveBanReason(): ?string
    {
        $reason = trim((string) ($this->ban_reason ?? ''));
        return $reason !== '' ? $reason : null;
    }

    public function getSuspensionMessage(): string
    {
        $baseMessage = __('Your account has been suspended');
        $reason = $this->getActiveBanReason();

        if (!$reason) {
            return $baseMessage;
        }

        return $baseMessage . ': ' . $reason;
    }

    /**
     * 检查是否需要重置流量
     */
    public function shouldResetTraffic(): bool
    {
        return $this->isActive() &&
               $this->next_reset_at !== null &&
               $this->next_reset_at <= time();
    }

    /**
     * 获取总使用流量
     */
    public function getTotalUsedTraffic(): int
    {
        return ($this->u ?? 0) + ($this->d ?? 0);
    }

    /**
     * 获取剩余流量
     */
    public function getRemainingTraffic(): int
    {
        $used = $this->getTotalUsedTraffic();
        $total = $this->transfer_enable ?? 0;
        return max(0, $total - $used);
    }

    /**
     * 获取流量使用百分比
     */
    public function getTrafficUsagePercentage(): float
    {
        $total = $this->transfer_enable ?? 0;
        if ($total <= 0) {
            return 0;
        }
        
        $used = $this->getTotalUsedTraffic();
        return min(100, ($used / $total) * 100);
    }

    /**
     * 检查是否为 Linux DO Connect 用户
     */
    public function isLinuxDoUser(): bool
    {
        return !empty($this->linux_do_id) && $this->oauth_provider === 'linux_do';
    }

    /**
     * 获取用户组别（基于信任等级）
     */
    public function getUserGroup(): string
    {
        return match($this->trust_level) {
            0 => 'new_user',
            1 => 'basic_user',
            2 => 'member',
            3 => 'regular',
            4 => 'leader',
            default => 'new_user'
        };
    }

    /**
     * 检查是否需要同步用户信息
     */
    public function shouldSyncUserInfo(): bool
    {
        if (!$this->isLinuxDoUser()) {
            return false;
        }

        if ($this->updated_at === null) {
            return true;
        }

        $updatedAt = $this->updated_at instanceof Carbon
            ? $this->updated_at
            : Carbon::createFromTimestamp((int) $this->updated_at);

        // 如果超过24小时没有同步，则需要同步
        return $updatedAt->lt(now()->subDay());
    }

    /**
     * 生成新的 API 密钥
     */
    public function generateApiKey(): string
    {
        do {
            $apiKey = 'xb_' . bin2hex(random_bytes(30));
        } while (self::where('api_key', $apiKey)->exists());

        $this->api_key = $apiKey;
        $this->save();

        return $apiKey;
    }

    public function setOauthExpiresAtAttribute($value): void
    {
        if ($value === null || $value === '') {
            $this->attributes['oauth_expires_at'] = null;
            return;
        }

        if ($value instanceof \DateTimeInterface) {
            $this->attributes['oauth_expires_at'] = $value->format('Y-m-d H:i:s');
            return;
        }

        if (is_numeric($value)) {
            $this->attributes['oauth_expires_at'] = Carbon::createFromTimestamp((int) $value)->format('Y-m-d H:i:s');
            return;
        }

        $this->attributes['oauth_expires_at'] = (string) $value;
    }

    public function getOauthExpiresAtAttribute($value): ?Carbon
    {
        if (empty($value)) {
            return null;
        }

        if (is_numeric($value)) {
            return Carbon::createFromTimestamp((int) $value);
        }

        return Carbon::parse($value);
    }

    /**
     * 检查 OAuth 令牌是否过期
     */
    public function isOAuthTokenExpired(): bool
    {
        return $this->oauth_expires_at && $this->oauth_expires_at->isPast();
    }

    /**
     * 获取有效的限制配置（个人限制优先于组限制）
     */
    public function getEffectiveLimits(): array
    {
        $individualLimit = $this->individualLimit;
        $groupLimit = UserGroupLimit::where('trust_level', $this->trust_level)->first();

        return [
            'speed_limit_up' => $individualLimit?->speed_limit_up ?: $groupLimit?->speed_limit_up ?: 0,
            'speed_limit_down' => $individualLimit?->speed_limit_down ?: $groupLimit?->speed_limit_down ?: 0,
            'device_limit' => $individualLimit?->device_limit ?: $groupLimit?->device_limit ?: 0,
            'connection_limit' => $individualLimit?->connection_limit ?: $groupLimit?->connection_limit ?: 0,
        ];
    }

    public function ensureSubscribeSecrets(): void
    {
        $updated = false;
        if (!$this->subscribe_path) {
            $this->subscribe_path = Helper::randomLetters(10);
            $updated = true;
        }
        if (!$this->subscribe_key) {
            $this->subscribe_key = Helper::randomLetters(8);
            $updated = true;
        }
        if (!$this->subscribe_salt || $this->subscribe_salt === $this->subscribe_key) {
            $this->subscribe_salt = Helper::randomLetters(6);
            $updated = true;
        }
        if ($updated) {
            $this->save();
        }
    }
}
