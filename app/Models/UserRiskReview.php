<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;
use Illuminate\Database\Eloquent\Relations\BelongsTo;

class UserRiskReview extends Model
{
    public const LEVEL_LOW = 'low';
    public const LEVEL_MEDIUM = 'medium';
    public const LEVEL_HIGH = 'high';

    protected $table = 'user_risk_reviews';
    protected $guarded = ['id'];
    protected $dateFormat = 'U';

    protected $casts = [
        'matched_user_count' => 'integer',
        'matched_user_ids' => 'array',
        'suspicion_score' => 'integer',
        'evidence' => 'array',
        'reviewed_at' => 'integer',
        'created_at' => 'timestamp',
        'updated_at' => 'timestamp',
    ];

    public function user(): BelongsTo
    {
        return $this->belongsTo(User::class, 'user_id', 'id');
    }
}
