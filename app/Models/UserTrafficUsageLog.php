<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class UserTrafficUsageLog extends Model
{
    protected $table = 'user_traffic_usage_logs';
    protected $guarded = ['id'];
    protected $dateFormat = 'U';

    protected $casts = [
        'created_at' => 'timestamp',
        'updated_at' => 'timestamp',
        'recorded_at' => 'integer',
        'raw_traffic_kb' => 'integer',
        'billed_traffic_kb' => 'integer',
        'multiplier_snapshot' => 'float',
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
            ->where('recorded_at', '<', now()->subDays($daysToKeep)->timestamp)
            ->delete();
    }
}
