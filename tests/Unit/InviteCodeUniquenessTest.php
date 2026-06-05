<?php

namespace Tests\Unit;

use App\Models\InviteCode;
use App\Models\User;
use Illuminate\Database\QueryException;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;

class InviteCodeUniquenessTest extends TestCase
{
    use RefreshDatabase;

    public function test_invite_code_column_is_unique(): void
    {
        $ownerA = User::factory()->create();
        $ownerB = User::factory()->create();

        InviteCode::query()->create([
            'user_id' => $ownerA->id,
            'code' => 'ABC12345',
            'status' => InviteCode::STATUS_UNUSED,
            'pv' => 0,
            'created_at' => time(),
            'updated_at' => time(),
        ]);

        $this->expectException(QueryException::class);

        InviteCode::query()->create([
            'user_id' => $ownerB->id,
            'code' => 'ABC12345',
            'status' => InviteCode::STATUS_UNUSED,
            'pv' => 0,
            'created_at' => time(),
            'updated_at' => time(),
        ]);
    }
}

