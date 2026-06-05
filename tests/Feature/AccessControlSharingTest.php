<?php

namespace Tests\Feature;

use App\Models\ServerNode;
use App\Models\User;
use App\Services\AccessControlService;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\DB;
use Tests\TestCase;

class AccessControlSharingTest extends TestCase
{
    use RefreshDatabase;

    public function test_individual_share_makes_node_visible_and_authorized_without_plan(): void
    {
        $owner = User::factory()->create();
        $recipient = User::factory()->create([
            'trust_level' => 0,
            'expired_at' => time() + 3600,
        ]);

        $node = ServerNode::factory()->active()->create([
            'user_id' => $owner->id,
            'access_control' => ['min_trust_level' => 4],
        ]);

        DB::table('user_node_access')->insert([
            'user_id' => $recipient->id,
            'node_id' => $node->id,
            'access_type' => 'individual',
            'granted_at' => now(),
        ]);

        $service = app(AccessControlService::class);

        $this->assertTrue($service->hasAccessibleNodesForUser($recipient));
        $this->assertContains($node->id, $service->getAccessibleNodesForUser($recipient)->pluck('id')->all());
        $this->assertContains($recipient->id, $service->getAccessibleUserIdsForNode($node));
    }

    public function test_group_share_makes_node_visible_and_authorized_without_plan(): void
    {
        $owner = User::factory()->create();
        $recipient = User::factory()->create([
            'trust_level' => 2,
            'expired_at' => time() + 3600,
        ]);

        $node = ServerNode::factory()->active()->create([
            'user_id' => $owner->id,
            'access_control' => ['min_trust_level' => 2],
        ]);

        $service = app(AccessControlService::class);

        $this->assertTrue($service->hasAccessibleNodesForUser($recipient));
        $this->assertContains($node->id, $service->getAccessibleNodesForUser($recipient)->pluck('id')->all());
        $this->assertContains($recipient->id, $service->getAccessibleUserIdsForNode($node));
    }
}
