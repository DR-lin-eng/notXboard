<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;
use Illuminate\Database\Eloquent\Relations\HasMany;

/**
 * @property int $id
 * @property int $order_id
 * @property string $trade_no
 * @property int $user_id
 * @property int $plan_id
 * @property int|null $assigned_admin_user_id
 * @property string $status
 * @property string|null $reason
 * @property int $gateway_amount
 * @property string|null $gateway_trade_no
 * @property string|null $epay_pid
 * @property string|null $epay_url
 * @property string|null $epay_key_encrypted
 * @property int|null $used_kb
 * @property int|null $allowance_kb
 * @property int|null $refund_amount
 * @property int|null $charged_amount
 * @property int $balance_refunded_amount
 * @property int $gateway_refunded_amount
 * @property int $site_balance_fallback_amount
 * @property int $gateway_refund_pending_amount
 * @property \Illuminate\Support\Carbon|null $gateway_refund_started_at
 * @property \Illuminate\Support\Carbon|null $voting_ends_at
 * @property \Illuminate\Support\Carbon|null $refunded_at
 * @property \Illuminate\Support\Carbon|null $resolved_at
 * @property int|null $resolved_by_user_id
 * @property string|null $decision
 */
class OrderRefundRequest extends Model
{
    protected $table = 'order_refund_requests';

    protected $casts = [
        'gateway_amount' => 'integer',
        'used_kb' => 'integer',
        'allowance_kb' => 'integer',
        'refund_amount' => 'integer',
        'charged_amount' => 'integer',
        'balance_refunded_amount' => 'integer',
        'gateway_refunded_amount' => 'integer',
        'site_balance_fallback_amount' => 'integer',
        'gateway_refund_pending_amount' => 'integer',
        'gateway_refund_started_at' => 'datetime',
        'voting_ends_at' => 'datetime',
        'refunded_at' => 'datetime',
        'resolved_at' => 'datetime',
        'order_id' => 'integer',
        'user_id' => 'integer',
        'plan_id' => 'integer',
        'assigned_admin_user_id' => 'integer',
        'resolved_by_user_id' => 'integer',
    ];

    public const STATUS_PENDING = 'pending';
    public const STATUS_PROCESSING = 'processing';
    public const STATUS_VOTING = 'voting';
    public const STATUS_APPROVED = 'approved';
    public const STATUS_DENIED = 'denied';
    public const STATUS_REFUNDED = 'refunded';
    public const STATUS_FAILED = 'failed';

    public function order(): BelongsTo
    {
        return $this->belongsTo(Order::class, 'order_id', 'id');
    }

    public function user(): BelongsTo
    {
        return $this->belongsTo(User::class, 'user_id', 'id');
    }

    public function plan(): BelongsTo
    {
        return $this->belongsTo(Plan::class, 'plan_id', 'id');
    }

    public function assignedAdmin(): BelongsTo
    {
        return $this->belongsTo(User::class, 'assigned_admin_user_id', 'id');
    }

    public function evidences(): HasMany
    {
        return $this->hasMany(OrderRefundEvidence::class, 'refund_request_id', 'id');
    }

    public function votes(): HasMany
    {
        return $this->hasMany(OrderRefundVote::class, 'refund_request_id', 'id');
    }
}
