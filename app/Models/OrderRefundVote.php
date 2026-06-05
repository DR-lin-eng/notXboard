<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class OrderRefundVote extends Model
{
    protected $table = 'order_refund_votes';

    protected $fillable = [
        'refund_request_id',
        'user_id',
        'vote',
    ];

    protected $casts = [
        'refund_request_id' => 'integer',
        'user_id' => 'integer',
    ];

    public function refundRequest(): BelongsTo
    {
        return $this->belongsTo(OrderRefundRequest::class, 'refund_request_id', 'id');
    }

    public function user(): BelongsTo
    {
        return $this->belongsTo(User::class, 'user_id', 'id');
    }
}

