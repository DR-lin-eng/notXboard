<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Factories\HasFactory;

/**
 * App\Models\UserGroupLimit
 *
 * @property int $id
 * @property int $trust_level 信任等级 (0-4)
 * @property int $speed_limit_up 上传限速 (Mbps)
 * @property int $speed_limit_down 下载限速 (Mbps)
 * @property int $device_limit 设备数限制
 * @property int $connection_limit 连接数限制
 * @property \Illuminate\Support\Carbon|null $created_at
 * @property \Illuminate\Support\Carbon|null $updated_at
 */
class UserGroupLimit extends Model
{
    use HasFactory;

    protected $fillable = [
        'trust_level',
        'speed_limit_up',
        'speed_limit_down',
        'device_limit',
        'connection_limit',
    ];

    protected $casts = [
        'trust_level' => 'integer',
        'speed_limit_up' => 'integer',
        'speed_limit_down' => 'integer',
        'device_limit' => 'integer',
        'connection_limit' => 'integer',
    ];

    // 信任等级常量
    public const TRUST_LEVEL_NEW = 0;
    public const TRUST_LEVEL_BASIC = 1;
    public const TRUST_LEVEL_MEMBER = 2;
    public const TRUST_LEVEL_REGULAR = 3;
    public const TRUST_LEVEL_LEADER = 4;

    /**
     * 获取信任等级名称
     */
    public function getTrustLevelName(): string
    {
        return match($this->trust_level) {
            self::TRUST_LEVEL_NEW => '新用户',
            self::TRUST_LEVEL_BASIC => '基础用户',
            self::TRUST_LEVEL_MEMBER => '成员',
            self::TRUST_LEVEL_REGULAR => '常规用户',
            self::TRUST_LEVEL_LEADER => '领导者',
            default => '未知等级'
        };
    }

    /**
     * 获取指定信任等级的限制
     */
    public static function getLimitsByTrustLevel(int $trustLevel): ?self
    {
        return self::where('trust_level', $trustLevel)->first();
    }

    /**
     * 创建或更新用户组限制
     */
    public static function createOrUpdateLimit(int $trustLevel, array $limits): self
    {
        return self::updateOrCreate(
            ['trust_level' => $trustLevel],
            $limits
        );
    }

    /**
     * 获取所有用户组限制
     */
    public static function getAllLimits(): \Illuminate\Database\Eloquent\Collection
    {
        return self::orderBy('trust_level')->get();
    }

    /**
     * 获取默认限制配置
     */
    public static function getDefaultLimits(): array
    {
        return [
            self::TRUST_LEVEL_NEW => [
                'speed_limit_up' => 10,
                'speed_limit_down' => 50,
                'device_limit' => 2,
                'connection_limit' => 10,
            ],
            self::TRUST_LEVEL_BASIC => [
                'speed_limit_up' => 20,
                'speed_limit_down' => 100,
                'device_limit' => 3,
                'connection_limit' => 20,
            ],
            self::TRUST_LEVEL_MEMBER => [
                'speed_limit_up' => 50,
                'speed_limit_down' => 200,
                'device_limit' => 5,
                'connection_limit' => 50,
            ],
            self::TRUST_LEVEL_REGULAR => [
                'speed_limit_up' => 100,
                'speed_limit_down' => 500,
                'device_limit' => 10,
                'connection_limit' => 100,
            ],
            self::TRUST_LEVEL_LEADER => [
                'speed_limit_up' => 0, // 无限制
                'speed_limit_down' => 0, // 无限制
                'device_limit' => 0, // 无限制
                'connection_limit' => 0, // 无限制
            ],
        ];
    }

    /**
     * 初始化默认限制配置
     */
    public static function initializeDefaults(): void
    {
        $defaults = self::getDefaultLimits();
        
        foreach ($defaults as $trustLevel => $limits) {
            self::createOrUpdateLimit($trustLevel, $limits);
        }
    }

    /**
     * 检查限制是否有效
     */
    public function isValid(): bool
    {
        return $this->trust_level >= 0 && 
               $this->trust_level <= 4 &&
               $this->speed_limit_up >= 0 &&
               $this->speed_limit_down >= 0 &&
               $this->device_limit >= 0 &&
               $this->connection_limit >= 0;
    }

    /**
     * 获取限制描述
     */
    public function getDescription(): string
    {
        $parts = [];
        
        if ($this->speed_limit_up > 0) {
            $parts[] = "上传: {$this->speed_limit_up}Mbps";
        }
        
        if ($this->speed_limit_down > 0) {
            $parts[] = "下载: {$this->speed_limit_down}Mbps";
        }
        
        if ($this->device_limit > 0) {
            $parts[] = "设备: {$this->device_limit}个";
        }
        
        if ($this->connection_limit > 0) {
            $parts[] = "连接: {$this->connection_limit}个";
        }
        
        return empty($parts) ? '无限制' : implode(', ', $parts);
    }
}
