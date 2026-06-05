<?php

namespace Tests\Unit;

use App\Models\Order;
use App\Models\OrderRefundRequest;
use App\Models\Plan;
use App\Models\User;
use App\Services\OrderRefundService;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Str;
use Tests\TestCase;

class OrderRefundServiceIdempotencyTest extends TestCase
{
    use RefreshDatabase;

    public function test_execute_refund_is_idempotent_for_refunded_requests(): void
    {
        $user = User::factory()->create([
            'balance' => 200,
        ]);

        $plan = Plan::query()->create([
            'group_id' => null,
            'transfer_enable' => 50,
            'name' => 'Node Test Plan',
            'speed_limit' => null,
            'show' => 1,
            'sort' => 1,
            'renew' => 1,
            'sell' => 1,
            'content' => null,
            'prices' => ['monthly' => 10],
            'reset_traffic_method' => 0,
            'capacity_limit' => 0,
            'device_limit' => null,
            'scope' => Plan::SCOPE_NODE,
            'owner_user_id' => null,
            'node_ids' => [],
        ]);

        $order = Order::query()->create([
            'user_id' => $user->id,
            'plan_id' => $plan->id,
            'coupon_id' => null,
            'payment_id' => null,
            'invite_user_id' => null,
            'type' => Order::TYPE_NEW_PURCHASE,
            'period' => Plan::PERIOD_MONTHLY,
            'trade_no' => (string) Str::uuid(),
            'callback_no' => null,
            'total_amount' => 0,
            'handling_amount' => 0,
            'discount_amount' => 0,
            'surplus_amount' => null,
            'refund_amount' => null,
            'balance_amount' => 100,
            'surplus_order_ids' => null,
            'status' => Order::STATUS_COMPLETED,
            'commission_status' => 0,
            'commission_balance' => 0,
            'actual_commission_balance' => null,
            'paid_at' => time(),
            'created_at' => time(),
            'updated_at' => time(),
        ]);

        $request = new OrderRefundRequest();
        $request->order_id = $order->id;
        $request->trade_no = $order->trade_no;
        $request->user_id = $user->id;
        $request->plan_id = $plan->id;
        $request->assigned_admin_user_id = null;
        $request->status = OrderRefundRequest::STATUS_APPROVED;
        $request->decision = 'approve';
        $request->reason = null;
        $request->gateway_amount = 0;
        $request->gateway_trade_no = null;
        $request->epay_pid = null;
        $request->epay_url = null;
        $request->epay_key_encrypted = null;
        $request->save();

        $service = app(OrderRefundService::class);
        $service->executeRefund($request, null);
        $service->executeRefund($request->fresh(), null);

        $request->refresh();
        $user->refresh();

        $this->assertSame(OrderRefundRequest::STATUS_REFUNDED, (string) $request->status);
        $this->assertSame(300, (int) $user->balance);
    }
}
