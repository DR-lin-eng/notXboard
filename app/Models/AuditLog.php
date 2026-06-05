<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * App\Models\AuditLog
 *
 * @property int $id
 * @property int $user_id 用户 ID
 * @property int $node_id 节点 ID
 * @property int|null $rule_id 触发的规则 ID
 * @property string $ip_address 源 IP 地址
 * @property string|null $target_domain 目标域名
 * @property string|null $target_protocol 目标协议
 * @property string $action_taken 执行的动作
 * @property \Illuminate\Support\Carbon|null $created_at
 * @property \Illuminate\Support\Carbon|null $updated_at
 *
 * @property-read User $user 用户
 * @property-read ServerNode $node 节点
 * @property-read AuditRule|null $rule 触发的规则
 */
class AuditLog extends Model
{
    protected $fillable = [
        'user_id',
        'node_id',
        'rule_id',
        'ip_address',
        'target_domain',
        'target_protocol',
        'action_taken',
    ];

    // 动作常量
    public const ACTION_BLOCKED = 'blocked';
    public const ACTION_ALLOWED = 'allowed';
    public const ACTION_LOGGED = 'logged';

    /**
     * 关联用户
     */
    public function user(): BelongsTo
    {
        return $this->belongsTo(User::class, 'user_id', 'id');
    }

    /**
     * 关联节点
     */
    public function node(): BelongsTo
    {
        return $this->belongsTo(ServerNode::class, 'node_id', 'id');
    }

    /**
     * 关联触发的规则
     */
    public function rule(): BelongsTo
    {
        return $this->belongsTo(AuditRule::class, 'rule_id', 'id');
    }

    /**
     * 获取动作描述
     */
    public function getActionDescription(): string
    {
        return match($this->action_taken) {
            self::ACTION_BLOCKED => '已阻止',
            self::ACTION_ALLOWED => '已允许',
            self::ACTION_LOGGED => '已记录',
            default => $this->action_taken
        };
    }

    /**
     * 获取目标描述
     */
    public function getTargetDescription(): string
    {
        if ($this->target_domain) {
            return "域名: {$this->target_domain}";
        }
        
        if ($this->target_protocol) {
            return "协议: {$this->target_protocol}";
        }
        
        return "IP: {$this->ip_address}";
    }

    /**
     * 创建审计日志
     */
    public static function createLog(
        int $userId,
        int $nodeId,
        string $ipAddress,
        string $actionTaken,
        ?int $ruleId = null,
        ?string $targetDomain = null,
        ?string $targetProtocol = null
    ): self {
        return self::create([
            'user_id' => $userId,
            'node_id' => $nodeId,
            'rule_id' => $ruleId,
            'ip_address' => $ipAddress,
            'target_domain' => $targetDomain,
            'target_protocol' => $targetProtocol,
            'action_taken' => $actionTaken,
        ]);
    }

    /**
     * 获取指定时间范围内的统计
     */
    public static function getStatistics(int $nodeId, \DateTime $startDate, \DateTime $endDate): array
    {
        $logs = self::where('node_id', $nodeId)
                   ->whereBetween('created_at', [$startDate, $endDate])
                   ->get();

        return [
            'total' => $logs->count(),
            'blocked' => $logs->where('action_taken', self::ACTION_BLOCKED)->count(),
            'allowed' => $logs->where('action_taken', self::ACTION_ALLOWED)->count(),
            'logged' => $logs->where('action_taken', self::ACTION_LOGGED)->count(),
            'unique_users' => $logs->pluck('user_id')->unique()->count(),
            'unique_ips' => $logs->pluck('ip_address')->unique()->count(),
            'top_domains' => $logs->whereNotNull('target_domain')
                                 ->groupBy('target_domain')
                                 ->map->count()
                                 ->sortDesc()
                                 ->take(10)
                                 ->toArray(),
        ];
    }

    /**
     * 清理旧日志
     */
    public static function cleanupOldLogs(int $daysToKeep = 30): int
    {
        return self::where('created_at', '<', now()->subDays($daysToKeep))->delete();
    }

    /**
     * 获取用户的审计日志
     */
    public static function getUserLogs(int $userId, int $limit = 100): \Illuminate\Database\Eloquent\Collection
    {
        return self::where('user_id', $userId)
                   ->with(['node', 'rule'])
                   ->orderBy('created_at', 'desc')
                   ->limit($limit)
                   ->get();
    }

    /**
     * 获取节点的审计日志
     */
    public static function getNodeLogs(int $nodeId, int $limit = 100): \Illuminate\Database\Eloquent\Collection
    {
        return self::where('node_id', $nodeId)
                   ->with(['user', 'rule'])
                   ->orderBy('created_at', 'desc')
                   ->limit($limit)
                   ->get();
    }
}