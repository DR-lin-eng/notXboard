<?php

namespace Tests\Unit;

use App\Models\ServerNode;
use App\Models\User;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Carbon;
use Tests\TestCase;

class CheckServerCommandTest extends TestCase
{
    use RefreshDatabase;

    public function test_check_server_skips_recent_active_nodes_without_reports(): void
    {
        $owner = User::factory()->create();

        ServerNode::factory()->create([
            'user_id' => $owner->id,
            'status' => ServerNode::STATUS_ACTIVE,
            'created_at' => Carbon::now()->subSeconds(60),
            'updated_at' => Carbon::now()->subSeconds(60),
        ]);

        $this->artisan('check:server')->assertExitCode(0);
    }
}
