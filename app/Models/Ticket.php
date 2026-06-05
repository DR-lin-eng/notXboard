<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;
use App\Models\ServerNode;

/**
 * App\Models\Ticket
 *
 * @property int $id
 * @property int $user_id 用户ID
 * @property int|null $node_id 节点ID
 * @property int|null $assigned_admin_user_id 负责人（个人管理员/超管）
 * @property string $subject 工单主题
 * @property string|null $level 工单等级
 * @property int $status 工单状态
 * @property int|null $reply_status 回复状态
 * @property int|null $last_reply_user_id 最后回复人
 * @property int $created_at
 * @property int $updated_at
 * 
 * @property-read User $user 关联的用户
 * @property-read \Illuminate\Database\Eloquent\Collection<int, TicketMessage> $messages 关联的工单消息
 */
class Ticket extends Model
{
    protected $table = 'v2_ticket';
    protected $dateFormat = 'U';
    protected $guarded = ['id'];
    protected $casts = [
        'created_at' => 'timestamp',
        'updated_at' => 'timestamp',
        'node_id' => 'integer',
        'assigned_admin_user_id' => 'integer',
    ];

    const STATUS_OPENING = 0;
    const STATUS_CLOSED = 1;
    public static $statusMap = [
        self::STATUS_OPENING => '开启',
        self::STATUS_CLOSED => '关闭'
    ];

    public function user(): BelongsTo
    {
        return $this->belongsTo(User::class, 'user_id', 'id');
    }

    public function node(): BelongsTo
    {
        return $this->belongsTo(ServerNode::class, 'node_id', 'id');
    }

    public function assignedAdmin(): BelongsTo
    {
        return $this->belongsTo(User::class, 'assigned_admin_user_id', 'id');
    }
    
    /**
     * 关联的工单消息
     */
    public function messages(): HasMany
    {
        return $this->hasMany(TicketMessage::class, 'ticket_id', 'id');
    }
    
    // 即将删除
    public function message(): HasMany
    {
        return $this->hasMany(TicketMessage::class, 'ticket_id', 'id');
    }
}
