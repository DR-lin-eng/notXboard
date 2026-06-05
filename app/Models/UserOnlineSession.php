<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Factories\HasFactory;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

/**
 * App\Models\UserOnlineSession
 *
 * @property int $id
 * @property int $user_id 用户 ID
 * @property int $node_id 节点 ID
 * @property string $ip_address IP 地址
 * @property string|null $user_agent 用户代理
 * @property int $connection_count 连接数
 * @property int $upload_traffic 上传流量 (KB)
 * @property int $download_traffic 下载流量 (KB)
 * @property \Illuminate\Support\Carbon $last_activity 最后活动时间
 * @property \Illuminate\Support\Carbon|null $created_at
 * @property \Illuminate\Support\Carbon|null $updated_at
 *
 * @property-read User $user 用户
 * @property-read ServerNode $node 节点
 */
class UserOnlineSession extends Model
{
    use HasFactory;

    protected $fillable = [
        'user_id',
        'node_id',
        'ip_address',
        'user_agent',
        'connection_count',
        'upload_traffic',
        'download_traffic',
        'last_activity',
    ];

    protected $casts = [
        'connection_count' => 'integer',
        'upload_traffic' => 'integer',
        'download_traffic' => 'integer',
        'last_activity' => 'datetime',
    ];

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
     * 检查会话是否在线
     */
    public function isOnline(): bool
    {
        return $this->last_activity->isAfter(now()->subMinutes(5));
    }

    /**
     * 获取总流量
     */
    public function getTotalTraffic(): int
    {
        return $this->upload_traffic + $this->download_traffic;
    }

    /**
     * 更新活动时间
     */
    public function updateActivity(): void
    {
        $this->last_activity = now();
        $this->save();
    }

    /**
     * 更新流量统计
     */
    public function updateTraffic(int $upload, int $download): void
    {
        $this->increment('upload_traffic', $upload);
        $this->increment('download_traffic', $download);
        $this->updateActivity();
    }

    /**
     * 增加连接数
     */
    public function incrementConnections(int $count = 1): void
    {
        $this->increment('connection_count', $count);
        $this->updateActivity();
    }

    /**
     * 减少连接数
     */
    public function decrementConnections(int $count = 1): void
    {
        $this->decrement('connection_count', max(0, $count));
        $this->updateActivity();
    }

    /**
     * 获取会话持续时间（分钟）
     */
    public function getDurationInMinutes(): int
    {
        return $this->created_at->diffInMinutes($this->last_activity);
    }

    /**
     * 获取平均速度 (KB/min)
     */
    public function getAverageSpeed(): float
    {
        $duration = $this->getDurationInMinutes();
        if ($duration <= 0) {
            return 0;
        }
        
        return $this->getTotalTraffic() / $duration;
    }

    /**
     * 清理过期会话
     */
    public static function cleanupExpiredSessions(): int
    {
        return self::where('last_activity', '<', now()->subHours(1))->delete();
    }

    /**
     * 获取用户在指定节点的活跃会话
     */
    public static function getActiveSession(int $userId, int $nodeId, string $ipAddress): ?self
    {
        return self::where('user_id', $userId)
                   ->where('node_id', $nodeId)
                   ->where('ip_address', $ipAddress)
                   ->where('last_activity', '>=', now()->subMinutes(5))
                   ->first();
    }

    /**
     * 创建或更新会话
     */
    public static function createOrUpdate(int $userId, int $nodeId, string $ipAddress, array $data = []): self
    {
        return self::updateOrCreate(
            [
                'user_id' => $userId,
                'node_id' => $nodeId,
                'ip_address' => $ipAddress,
            ],
            array_merge([
                'user_agent' => $data['user_agent'] ?? null,
                'connection_count' => $data['connection_count'] ?? 1,
                'upload_traffic' => $data['upload_traffic'] ?? 0,
                'download_traffic' => $data['download_traffic'] ?? 0,
                'last_activity' => now(),
            ], $data)
        );
    }
}
