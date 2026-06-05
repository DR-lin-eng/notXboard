<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class SponsorDonation extends Model
{
    protected $fillable = [
        'user_id',
        'trade_no',
        'total_amount',
        'payment_id',
        'callback_no',
        'status',
        'paid_at',
    ];

    protected $casts = [
        'user_id' => 'integer',
        'total_amount' => 'integer',
        'payment_id' => 'integer',
        'status' => 'integer',
        'paid_at' => 'integer',
    ];

    public const STATUS_PENDING = 0;
    public const STATUS_COMPLETED = 1;
    public const STATUS_CANCELLED = 2;

    public function user(): BelongsTo
    {
        return $this->belongsTo(User::class, 'user_id', 'id');
    }
}

