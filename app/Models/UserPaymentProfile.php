<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class UserPaymentProfile extends Model
{
    protected $fillable = [
        'user_id',
        'provider',
        'pid',
        'key_encrypted',
        'url',
        'submit_path',
        'use_post',
        'sitename',
        'device',
    ];

    protected $casts = [
        'user_id' => 'integer',
        'use_post' => 'boolean',
    ];

    public function user(): BelongsTo
    {
        return $this->belongsTo(User::class, 'user_id', 'id');
    }
}

