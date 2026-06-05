<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class TcpingSample extends Model
{
    protected $table = 'tcping_samples';
    protected $guarded = ['id'];
    protected $dateFormat = 'U';

    protected $casts = [
        'created_at' => 'timestamp',
        'updated_at' => 'timestamp',
        'is_reachable' => 'boolean',
        'is_timeout' => 'boolean',
        'sampled_at' => 'integer',
        'latency_ms' => 'integer',
    ];

    public function node(): BelongsTo
    {
        return $this->belongsTo(ServerNode::class, 'node_id', 'id');
    }

    public function agent(): BelongsTo
    {
        return $this->belongsTo(TcpingAgent::class, 'agent_id', 'id');
    }

    public static function cleanupOldRecords(int $daysToKeep = 7): int
    {
        return self::query()
            ->where('sampled_at', '<', now()->subDays($daysToKeep)->timestamp)
            ->delete();
    }
}
