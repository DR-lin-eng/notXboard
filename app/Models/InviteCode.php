<?php

namespace App\Models;

use Illuminate\Database\Eloquent\Model;

class InviteCode extends Model
{
    protected $table = 'v2_invite_code';
    protected $dateFormat = 'U';
    protected $guarded = ['id'];
    protected $casts = [
        'created_at' => 'timestamp',
        'updated_at' => 'timestamp',
        'status' => 'boolean',
        'assigned_plan_id' => 'integer',
    ];

    const STATUS_UNUSED = 0;
    const STATUS_USED = 1;

    public function plan()
    {
        return $this->belongsTo(Plan::class, 'assigned_plan_id', 'id');
    }
}
