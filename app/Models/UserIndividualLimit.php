<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * App\Models\UserIndividualLimit
 *
 * @property int $id
 * @property int $user_id 用户 ID
 * @property int $speed_limit_up 上传限速 (Mbps, 0表示使用组配置)
 * @property int $speed_limit_down 下载限速 (Mbps, 0表示使用组配置)
 * @property int $device_limit 设备数限制 (0表示使用组配置)
 * @property int $connection_limit 连接数限制 (0表示使用组配置)
 * @property \Illuminate\Support\Carbon|null $created_at
 * @property \Illuminate\Support\Carbon|null $updated_at
 *
 * @property-read User $user 用户
 */
class UserIndividualLimit extends Model
{
    use HasFactory;

    protected $fillable = [
        'user_id',
        'speed_limit_up',
        'speed_limit_down',
        'device_limit',
        'connection_limit',
    ];

    protected $casts = [
        'user_id' => 'integer',
        'speed_limit_up' => 'integer',
        'speed_limit_down' => 'integer',
        'device_limit' => 'integer',
        'connection_limit' => 'integer',
    ];

    /**
     * 关联用户
     */
    public function user(): BelongsTo
    {
        return $this->belongsTo(User::class, 'user_id', 'id');
    }

    /**
     * 获取用户的个人限制
     */
    public static function getUserLimit(int $userId): ?self
    {
        return self::where('user_id', $userId)->first();
    }

    /**
     * 创建或更新用户个人限制
     */
    public static function createOrUpdateLimit(int $userId, array $limits): self
    {
        return self::updateOrCreate(
            ['user_id' => $userId],
            $limits
        );
    }

    /**
     * 获取有效的限制值（如果为0则使用组配置）
     */
    public function getEffectiveLimits(): array
    {
        $user = $this->user;
        $groupLimit = UserGroupLimit::getLimitsByTrustLevel($user->trust_level);
        
        return [
            'speed_limit_up' => $this->speed_limit_up ?: ($groupLimit?->speed_limit_up ?? 0),
            'speed_limit_down' => $this->speed_limit_down ?: ($groupLimit?->speed_limit_down ?? 0),
            'device_limit' => $this->device_limit ?: ($groupLimit?->device_limit ?? 0),
            'connection_limit' => $this->connection_limit ?: ($groupLimit?->connection_limit ?? 0),
        ];
    }

    /**
     * 检查是否有自定义限制
     */
    public function hasCustomLimits(): bool
    {
        return $this->speed_limit_up > 0 ||
               $this->speed_limit_down > 0 ||
               $this->device_limit > 0 ||
               $this->connection_limit > 0;
    }

    /**
     * 重置为使用组配置
     */
    public function resetToGroupLimits(): void
    {
        $this->update([
            'speed_limit_up' => 0,
            'speed_limit_down' => 0,
            'device_limit' => 0,
            'connection_limit' => 0,
        ]);
    }

    /**
     * 获取限制描述
     */
    public function getDescription(): string
    {
        $effectiveLimits = $this->getEffectiveLimits();
        $parts = [];
        
        if ($effectiveLimits['speed_limit_up'] > 0) {
            $source = $this->speed_limit_up > 0 ? '个人' : '组';
            $parts[] = "上传: {$effectiveLimits['speed_limit_up']}Mbps ({$source})";
        }
        
        if ($effectiveLimits['speed_limit_down'] > 0) {
            $source = $this->speed_limit_down > 0 ? '个人' : '组';
            $parts[] = "下载: {$effectiveLimits['speed_limit_down']}Mbps ({$source})";
        }
        
        if ($effectiveLimits['device_limit'] > 0) {
            $source = $this->device_limit > 0 ? '个人' : '组';
            $parts[] = "设备: {$effectiveLimits['device_limit']}个 ({$source})";
        }
        
        if ($effectiveLimits['connection_limit'] > 0) {
            $source = $this->connection_limit > 0 ? '个人' : '组';
            $parts[] = "连接: {$effectiveLimits['connection_limit']}个 ({$source})";
        }
        
        return empty($parts) ? '无限制' : implode(', ', $parts);
    }

    /**
     * 检查限制是否有效
     */
    public function isValid(): bool
    {
        return $this->speed_limit_up >= 0 &&
               $this->speed_limit_down >= 0 &&
               $this->device_limit >= 0 &&
               $this->connection_limit >= 0;
    }

    /**
     * 应用限制到用户
     */
    public function applyLimits(array $limits): void
    {
        $this->update(array_intersect_key($limits, array_flip([
            'speed_limit_up',
            'speed_limit_down', 
            'device_limit',
            'connection_limit'
        ])));
    }

    /**
     * 批量设置用户限制
     */
    public static function batchSetLimits(array $userIds, array $limits): int
    {
        $count = 0;
        
        foreach ($userIds as $userId) {
            self::createOrUpdateLimit($userId, $limits);
            $count++;
        }
        
        return $count;
    }

    /**
     * 清理无效的个人限制（所有值都为0的记录）
     */
    public static function cleanupEmptyLimits(): int
    {
        return self::where('speed_limit_up', 0)
                   ->where('speed_limit_down', 0)
                   ->where('device_limit', 0)
                   ->where('connection_limit', 0)
                   ->delete();
    }
}
