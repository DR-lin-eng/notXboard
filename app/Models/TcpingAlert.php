<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class TcpingAlert extends Model
{
    protected $table = 'tcping_alerts';
    protected $guarded = ['id'];
    protected $dateFormat = 'U';

    public const STATUS_ACTIVE = 'active';
    public const STATUS_RESOLVED = 'resolved';

    protected $casts = [
        'created_at' => 'timestamp',
        'updated_at' => 'timestamp',
        'started_at' => 'integer',
        'triggered_at' => 'integer',
        'recovered_at' => 'integer',
    ];

    public function user(): BelongsTo
    {
        return $this->belongsTo(User::class, 'user_id', 'id');
    }

    public function node(): BelongsTo
    {
        return $this->belongsTo(ServerNode::class, 'node_id', 'id');
    }

    public static function cleanupOldRecords(int $daysToKeep = 7): int
    {
        return self::query()
            ->where('triggered_at', '<', now()->subDays($daysToKeep)->timestamp)
            ->delete();
    }
}
