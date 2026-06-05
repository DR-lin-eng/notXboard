<?php

namespace Tests\Feature;

use App\Models\Order;
use App\Models\Payment;
use App\Models\User;
use Illuminate\Foundation\Testing\RefreshDatabase;
use Illuminate\Support\Str;
use Laravel\Sanctum\Sanctum;
use Tests\TestCase;

class UserFundsConsistencyApiTest extends TestCase
{
    use RefreshDatabase;

    public function test_checkout_keeps_payment_snapshot_frozen(): void
    {
        $user = User::factory()->create();
        Sanctum::actingAs($user);

        $paymentA = Payment::query()->create([
            'uuid' => Str::lower(Str::random(16)),
            'payment' => 'EPay',
            'name' => 'Payment A',
            'icon' => null,
            'config' => ['pid' => 'a', 'key' => 'a', 'url' => 'https://pay-a.test'],
            'notify_domain' => null,
            'handling_fee_fixed' => 100,
            'handling_fee_percent' => 0,
            'enable' => 1,
            'sort' => 1,
            'created_at' => time(),
            'updated_at' => time(),
        ]);
        $paymentB = Payment::query()->create([
            'uuid' => Str::lower(Str::random(16)),
            'payment' => 'EPay',
            'name' => 'Payment B',
            'icon' => null,
            'config' => ['pid' => 'b', 'key' => 'b', 'url' => 'https://pay-b.test'],
            'notify_domain' => null,
            'handling_fee_fixed' => 0,
            'handling_fee_percent' => 0,
            'enable' => 1,
            'sort' => 2,
            'created_at' => time(),
            'updated_at' => time(),
        ]);

        $order = Order::query()->create([
            'user_id' => $user->id,
            'plan_id' => 1,
            'coupon_id' => null,
            'payment_id' => $paymentA->id,
            'invite_user_id' => null,
            'type' => Order::TYPE_NEW_PURCHASE,
            'period' => 'monthly',
            'trade_no' => (string) Str::uuid(),
            'callback_no' => null,
            'total_amount' => 1000,
            'handling_amount' => 100,
            'discount_amount' => 0,
            'surplus_amount' => null,
            'refund_amount' => null,
            'balance_amount' => 0,
            'surplus_order_ids' => null,
            'status' => Order::STATUS_PENDING,
            'commission_status' => 0,
            'commission_balance' => 0,
            'actual_commission_balance' => null,
            'paid_at' => null,
            'created_at' => time(),
            'updated_at' => time(),
        ]);

        $this->postJson('/api/v1/user/order/checkout', [
            'trade_no' => $order->trade_no,
            'method' => $paymentB->id,
        ])->assertStatus(400)
            ->assertJsonPath('status', 'fail');

        $order->refresh();
        $this->assertSame((int) $paymentA->id, (int) $order->payment_id);
        $this->assertSame(100, (int) $order->handling_amount);
    }

    public function test_transfer_cannot_double_spend_commission_balance(): void
    {
        $user = User::factory()->create([
            'balance' => 20,
            'commission_balance' => 100,
        ]);
        Sanctum::actingAs($user);

        $this->postJson('/api/v1/user/transfer', [
            'transfer_amount' => 70,
        ])->assertOk();

        $this->postJson('/api/v1/user/transfer', [
            'transfer_amount' => 70,
        ])->assertStatus(400)
            ->assertJsonPath('status', 'fail');

        $user->refresh();
        $this->assertSame(90, (int) $user->balance);
        $this->assertSame(30, (int) $user->commission_balance);
    }
}

