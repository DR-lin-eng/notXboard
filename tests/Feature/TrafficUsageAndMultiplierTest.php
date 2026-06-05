<?php

namespace Tests\Feature;

use App\Models\Plan;
use App\Models\ServerNode;
use App\Models\User;
use App\Models\UserPlanSubscription;
use App\Services\NodeTrafficService;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Facades\DB;
use Laravel\Sanctum\Sanctum;
use Tests\TestCase;

class TrafficUsageAndMultiplierTest extends TestCase
{
    use RefreshDatabase;

    public function test_node_multiplier_only_changes_subscription_billing_and_usage_logs(): void
    {
        $owner = User::factory()->create();
        $consumer = User::factory()->create([
            'trust_level' => 2,
        ]);

        $node = ServerNode::factory()->active()->create([
            'user_id' => $owner->id,
            'traffic_multiplier' => 2.5,
            'traffic_used' => 0,
        ]);

        $plan = Plan::query()->create([
            'group_id' => 1,
            'transfer_enable' => 1024 * 1024,
            'name' => 'Owner Node Plan',
            'show' => true,
            'renew' => true,
            'sell' => true,
            'sort' => 0,
            'scope' => Plan::SCOPE_NODE,
            'owner_user_id' => $owner->id,
            'node_ids' => [$node->id],
            'prices' => [
                Plan::PERIOD_MONTHLY => 1000,
            ],
        ]);

        $subscription = UserPlanSubscription::query()->create([
            'user_id' => $consumer->id,
            'plan_id' => $plan->id,
            'order_id' => 1001,
            'period' => Plan::PERIOD_MONTHLY,
            'traffic_allowance_kb' => 10_000,
            'used_traffic_kb' => 0,
            'started_at' => time(),
            'expired_at' => time() + 86400,
            'status' => UserPlanSubscription::STATUS_ACTIVE,
        ]);

        DB::table('user_node_plan_access')->insert([
            'user_id' => $consumer->id,
            'node_id' => $node->id,
            'plan_id' => $plan->id,
            'granted_at' => now(),
        ]);

        app(NodeTrafficService::class)->recordTrafficFromPush($node, [
            $consumer->id => [100, 200],
        ]);

        $node->refresh();
        $subscription->refresh();

        $this->assertSame(300, (int) $node->traffic_used);
        $this->assertSame(750, (int) $subscription->used_traffic_kb);

        $this->assertDatabaseHas('user_traffic_usage_logs', [
            'user_id' => $consumer->id,
            'node_id' => $node->id,
            'raw_traffic_kb' => 300,
            'billed_traffic_kb' => 750,
        ]);

        Sanctum::actingAs($consumer);
        $response = $this->getJson('/api/v1/user/traffic-usage-logs?days=7&limit=20');

        $response->assertOk()
            ->assertJsonPath('data.0.raw_traffic_kb', 300)
            ->assertJsonPath('data.0.billed_traffic_kb', 750)
            ->assertJsonPath('data.0.multiplier_snapshot', 2.5);
    }
}
