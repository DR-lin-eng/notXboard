<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;

class Notice extends Model
{
    public const SCOPE_GLOBAL = 'global';
    public const SCOPE_PLAN_SUBSCRIBERS = 'plan_subscribers';

    protected $table = 'v2_notice';
    protected $dateFormat = 'U';
    protected $guarded = ['id'];
    protected $casts = [
        'created_at' => 'timestamp',
        'updated_at' => 'timestamp',
        'tags' => 'array',
        'target_plan_ids' => 'array',
        'show' => 'boolean',
        'popup' => 'boolean',
        'author_user_id' => 'integer',
    ];
}
