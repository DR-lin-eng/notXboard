<?php

namespace Tests\Unit;

use App\Models\Order;
use App\Models\User;
use App\Services\OrderService;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Str;
use Tests\TestCase;

class OrderServiceCancelIdempotencyTest extends TestCase
{
    use RefreshDatabase;

    public function test_cancel_is_idempotent_and_balance_is_refunded_once(): void
    {
        $user = User::factory()->create([
            'balance' => 0,
        ]);

        $order = Order::query()->create([
            'user_id' => $user->id,
            'plan_id' => 1,
            'coupon_id' => null,
            'payment_id' => null,
            'invite_user_id' => null,
            'type' => Order::TYPE_NEW_PURCHASE,
            'period' => 'monthly',
            'trade_no' => (string) Str::uuid(),
            'callback_no' => null,
            'total_amount' => 1000,
            'handling_amount' => null,
            'discount_amount' => 0,
            'surplus_amount' => null,
            'refund_amount' => null,
            'balance_amount' => 500,
            'surplus_order_ids' => null,
            'status' => Order::STATUS_PENDING,
            'commission_status' => 0,
            'commission_balance' => 0,
            'actual_commission_balance' => null,
            'paid_at' => null,
            'created_at' => time(),
            'updated_at' => time(),
        ]);

        $service = new OrderService($order);
        $this->assertTrue($service->cancel());
        $this->assertTrue($service->cancel());

        $user->refresh();
        $order->refresh();

        $this->assertSame(500, (int) $user->balance);
        $this->assertSame(Order::STATUS_CANCELLED, (int) $order->status);
    }
}

