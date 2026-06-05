<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Builder;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class UserPlanSubscription extends Model
{
    protected $table = 'user_plan_subscriptions';
    protected $guarded = ['id'];
    protected $dateFormat = 'U';

    public const STATUS_ACTIVE = 1;
    public const STATUS_EXPIRED = 2;
    public const STATUS_REVOKED = 3;

    protected $casts = [
        'created_at' => 'timestamp',
        'updated_at' => 'timestamp',
        'started_at' => 'integer',
        'expired_at' => 'integer',
        'traffic_allowance_kb' => 'integer',
        'used_traffic_kb' => 'integer',
        'status' => 'integer',
    ];

    public function scopeActive(Builder $query, ?int $timestamp = null): Builder
    {
        $timestamp = $timestamp ?? time();

        return $query->where('status', self::STATUS_ACTIVE)
            ->where(function (Builder $q) use ($timestamp) {
                $q->whereNull('expired_at')
                    ->orWhere('expired_at', '>', $timestamp);
            });
    }

    public function user(): BelongsTo
    {
        return $this->belongsTo(User::class, 'user_id', 'id');
    }

    public function plan(): BelongsTo
    {
        return $this->belongsTo(Plan::class, 'plan_id', 'id');
    }

    public function order(): BelongsTo
    {
        return $this->belongsTo(Order::class, 'order_id', 'id');
    }
}
