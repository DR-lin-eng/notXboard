<?php

namespace Tests\Unit;

use App\Models\Plan;
use App\Models\User;
use App\Services\OrderService;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Tests\TestCase;

class OrderServicePurchaseTokenTest extends TestCase
{
    use RefreshDatabase;

    public function test_create_from_request_accepts_link_only_plan_with_purchase_token(): void
    {
        $user = User::factory()->create([
            'balance' => 0,
        ]);

        $plan = Plan::query()->create([
            'group_id' => null,
            'transfer_enable' => 1024,
            'name' => 'Shared Link Plan',
            'show' => false,
            'visibility_scope' => Plan::VISIBILITY_LINK_ONLY,
            'share_token' => 'share-plan-token',
            'sort' => 0,
            'renew' => true,
            'sell' => true,
            'scope' => Plan::SCOPE_NODE,
            'owner_user_id' => null,
            'node_ids' => [],
            'prices' => [
                Plan::PERIOD_MONTHLY => 10,
            ],
        ]);

        $order = OrderService::createFromRequest(
            $user,
            $plan,
            Plan::PERIOD_MONTHLY,
            null,
            'share-plan-token'
        );

        $this->assertSame($plan->id, $order->plan_id);
        $this->assertSame($user->id, $order->user_id);
        $this->assertNotEmpty($order->trade_no);
    }
}
