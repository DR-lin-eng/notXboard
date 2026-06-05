<?php

namespace App\Models;

use App\Utils\Helper;
use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;

class TcpingAgent extends Model
{
    protected $table = 'tcping_agents';
    protected $guarded = ['id'];
    protected $dateFormat = 'U';

    protected $casts = [
        'created_at' => 'timestamp',
        'updated_at' => 'timestamp',
        'is_enabled' => 'boolean',
        'last_heartbeat_at' => 'integer',
        'last_sync_at' => 'integer',
    ];

    public function user(): BelongsTo
    {
        return $this->belongsTo(User::class, 'user_id', 'id');
    }

    public function samples(): HasMany
    {
        return $this->hasMany(TcpingSample::class, 'agent_id', 'id');
    }

    public function ensureToken(): void
    {
        if (!empty($this->token)) {
            return;
        }

        $this->token = Helper::randomChar(48);
    }

    public function rotateToken(): void
    {
        $this->token = Helper::randomChar(48);
    }
}
