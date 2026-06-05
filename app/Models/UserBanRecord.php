<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class UserBanRecord extends Model
{
    public const ACTION_BAN = 'ban';
    public const ACTION_UNBAN = 'unban';

    protected $table = 'user_ban_records';
    protected $guarded = ['id'];
    protected $dateFormat = 'U';

    protected $casts = [
        'admin_id' => 'integer',
        'created_at' => 'timestamp',
        'updated_at' => 'timestamp',
        'context' => 'array',
    ];

    public function user(): BelongsTo
    {
        return $this->belongsTo(User::class, 'user_id', 'id');
    }

    public function admin(): BelongsTo
    {
        return $this->belongsTo(User::class, 'admin_id', 'id');
    }
}
