<?php

namespace App\Http\Controllers\V1\User;

use App\Exceptions\ApiException;
use App\Http\Controllers\Controller;
use App\Http\Requests\User\OrderSave;
use App\Http\Resources\OrderResource;
use App\Models\Order;
use App\Models\Payment;
use App\Models\Plan;
use App\Models\User;
use App\Services\CouponService;
use App\Services\OrderService;
use App\Services\PaymentService;
use App\Services\PlanService;
use App\Services\UserService;
use App\Services\PaymentProfileService;
use Illuminate\Http\Request;
use Illuminate\Support\Facades\Crypt;
use Illuminate\Support\Facades\DB;

class OrderController extends Controller
{
    public function fetch(Request $request)
    {
        $request->validate([
            'status' => 'nullable|integer|in:0,1,2,3',
        ]);
        $orders = Order::with('plan')
            ->where('user_id', $request->user()->id)
            ->when($request->input('status') !== null, function ($query) use ($request) {
                $query->where('status', $request->input('status'));
            })
            ->orderBy('created_at', 'DESC')
            ->get();

        return $this->success(OrderResource::collection($orders));
    }

    public function detail(Request $request)
    {
        $request->validate([
            'trade_no' => 'required|string',
        ]);
        $order = Order::with(['payment', 'plan'])
            ->where('user_id', $request->user()->id)
            ->where('trade_no', $request->input('trade_no'))
            ->first();
        if (!$order) {
            return $this->fail([400, __('Order does not exist or has been paid')]);
        }
        $order['try_out_plan_id'] = (int) admin_setting('try_out_plan_id');
        if (!$order->plan) {
            return $this->fail([400, __('Subscription plan does not exist')]);
        }
        if ($order->surplus_order_ids) {
            $order['surplus_orders'] = Order::whereIn('id', $order->surplus_order_ids)->get();
        }
        return $this->success(OrderResource::make($order));
    }

    public function save(OrderSave $request)
    {
        $request->validate([
            'plan_id' => 'nullable|exists:App\Models\Plan,id',
            'purchase_token' => 'nullable|string|min:8|max:64',
            'period' => 'required|string'
        ]);

        $user = User::findOrFail($request->user()->id);
        $userService = app(UserService::class);

        if ($userService->isNotCompleteOrderByUserId($user->id)) {
            throw new ApiException(__('You have an unpaid or pending order, please try again later or cancel it'));
        }

        $purchaseToken = trim((string) $request->input('purchase_token', ''));
        if ($request->filled('plan_id')) {
            $plan = Plan::findOrFail($request->input('plan_id'));
        } elseif ($purchaseToken !== '') {
            $plan = Plan::query()
                ->where('share_token', $purchaseToken)
                ->first();
            if (!$plan) {
                throw new ApiException(__('Subscription plan does not exist'));
            }
        } else {
            throw new ApiException(__('Subscription plan does not exist'));
        }

        $planService = new PlanService($plan);

        $planService->validatePurchase($user, $request->input('period'), [
            'purchase_token' => $purchaseToken,
        ]);

        $order = OrderService::createFromRequest(
            $user,
            $plan,
            $request->input('period'),
            $request->input('coupon_code'),
            $purchaseToken
        );

        return $this->success($order->trade_no);
    }

    protected function applyCoupon(Order $order, string $couponCode): void
    {
        $couponService = new CouponService($couponCode);
        if (!$couponService->use($order)) {
            throw new ApiException(__('Coupon failed'));
        }
        $order->coupon_id = $couponService->getId();
    }

    protected function handleUserBalance(Order $order, User $user, UserService $userService): void
    {
        $remainingBalance = $user->balance - $order->total_amount;

        if ($remainingBalance > 0) {
            if (!$userService->addBalance($order->user_id, -$order->total_amount)) {
                throw new ApiException(__('Insufficient balance'));
            }
            $order->balance_amount = $order->total_amount;
            $order->total_amount = 0;
        } else {
            if (!$userService->addBalance($order->user_id, -$user->balance)) {
                throw new ApiException(__('Insufficient balance'));
            }
            $order->balance_amount = $user->balance;
            $order->total_amount = $order->total_amount - $user->balance;
        }
    }

    public function checkout(Request $request)
    {
        $tradeNo = $request->input('trade_no');
        $method = (int) $request->input('method');
        $payTo = $request->input('pay_to'); // optional; kept for API compatibility

        $payment = $method > 0 ? Payment::find($method) : null;

        $checkoutState = DB::transaction(function () use ($tradeNo, $request, $payment, $method, $payTo) {
            $order = Order::query()
                ->where('trade_no', $tradeNo)
                ->where('user_id', $request->user()->id)
                ->lockForUpdate()
                ->first();

            if (!$order) {
                return ['error' => __('Order does not exist or has been paid')];
            }
            if ((int) $order->status !== Order::STATUS_PENDING) {
                return ['error' => __('Order does not exist or has been paid')];
            }
            if ((int) $order->total_amount <= 0) {
                return [
                    'free' => true,
                    'trade_no' => (string) $order->trade_no,
                    'order_id' => (int) $order->id,
                ];
            }
            if (!$payment || !$payment->enable) {
                return ['error' => __('Payment method is not available')];
            }

            $paymentId = $order->payment_id === null ? null : (int) $order->payment_id;
            if ($paymentId !== null && $paymentId !== $method) {
                return ['error' => __('The payment method has been locked for this order')];
            }

            $shouldSave = false;
            if ($paymentId === null) {
                $order->payment_id = $method;
                $shouldSave = true;
            }

            $overrideConfig = null;
            if ((string) $payment->payment === 'EPay') {
                // 冻结快照后复用，避免重复 checkout 被改绑到其他 pid/key。
                if ($order->epay_pid && $order->epay_url && $order->epay_key_encrypted) {
                    try {
                        $overrideConfig = [
                            'pid' => (string) $order->epay_pid,
                            'key' => Crypt::decryptString((string) $order->epay_key_encrypted),
                            'url' => (string) $order->epay_url,
                            'submit_path' => '/pay/submit.php',
                            'use_post' => true,
                        ];
                    } catch (\Throwable $e) {
                        return ['error' => __('Payment snapshot is invalid, please contact support')];
                    }
                } else {
                    $epaySnapshot = null;
                    try {
                        $plan = Plan::query()->find($order->plan_id);
                        if ($plan) {
                            $epay = app(PaymentProfileService::class)->resolveEpayForPlan($plan, $payTo);
                            if ($epay) {
                                $epaySnapshot = [
                                    'pid' => (string) ($epay['pid'] ?? ''),
                                    'key' => (string) ($epay['key'] ?? ''),
                                    'url' => (string) ($epay['url'] ?? ''),
                                    'submit_path' => (string) ($epay['submit_path'] ?? '/pay/submit.php'),
                                    'use_post' => (bool) ($epay['use_post'] ?? true),
                                    'sitename' => $epay['sitename'] ?? null,
                                    'device' => $epay['device'] ?? null,
                                ];
                            }
                        }
                    } catch (\Throwable $e) {
                        $epaySnapshot = null;
                    }

                    if (!$epaySnapshot) {
                        $cfg = is_string($payment->config) ? json_decode($payment->config, true) : $payment->config;
                        if (is_array($cfg) && !empty($cfg['pid']) && !empty($cfg['key']) && !empty($cfg['url'])) {
                            $epaySnapshot = [
                                'pid' => (string) $cfg['pid'],
                                'key' => (string) $cfg['key'],
                                'url' => (string) $cfg['url'],
                                'submit_path' => '/pay/submit.php',
                                'use_post' => true,
                            ];
                        }
                    }

                    if ($epaySnapshot && $epaySnapshot['pid'] !== '' && $epaySnapshot['key'] !== '' && $epaySnapshot['url'] !== '') {
                        $order->epay_pid = $epaySnapshot['pid'];
                        $order->epay_url = $epaySnapshot['url'];
                        $order->epay_key_encrypted = Crypt::encryptString($epaySnapshot['key']);
                        $overrideConfig = $epaySnapshot;
                        $shouldSave = true;
                    }
                }
            }

            if ($order->handling_amount === null) {
                $order->handling_amount = null;
                if ($payment->handling_fee_fixed || $payment->handling_fee_percent) {
                    $order->handling_amount = (int) round(
                        ($order->total_amount * ($payment->handling_fee_percent / 100)) + $payment->handling_fee_fixed
                    );
                }
                $shouldSave = true;
            }

            if ($shouldSave && !$order->save()) {
                return ['error' => __('Request failed, please try again later')];
            }

            return [
                'order_id' => (int) $order->id,
                'trade_no' => (string) $order->trade_no,
                'user_id' => (int) $order->user_id,
                'amount' => (int) $order->total_amount + (int) ($order->handling_amount ?? 0),
                'override_config' => $overrideConfig,
            ];
        });

        if (isset($checkoutState['error'])) {
            return $this->fail([400, $checkoutState['error']]);
        }

        if (!empty($checkoutState['free'])) {
            $freeOrder = Order::query()->find($checkoutState['order_id']);
            if (!$freeOrder) {
                return $this->fail([400, __('Order does not exist or has been paid')]);
            }
            $orderService = new OrderService($freeOrder);
            if (!$orderService->paid($checkoutState['trade_no'])) {
                return $this->fail([400, '支付失败']);
            }
            return response([
                'type' => -1,
                'data' => true
            ]);
        }

        $paymentService = new PaymentService($payment->payment, $payment->id, null, $checkoutState['override_config']);
        $result = $paymentService->pay([
            'trade_no' => $checkoutState['trade_no'],
            'total_amount' => $checkoutState['amount'],
            'user_id' => $checkoutState['user_id'],
            'stripe_token' => $request->input('token')
        ]);
        return response([
            'type' => $result['type'],
            'data' => $result['data']
        ]);
    }

    public function check(Request $request)
    {
        $tradeNo = $request->input('trade_no');
        $order = Order::where('trade_no', $tradeNo)
            ->where('user_id', $request->user()->id)
            ->first();
        if (!$order) {
            return $this->fail([400, __('Order does not exist')]);
        }
        return $this->success($order->status);
    }

    public function getPaymentMethod()
    {
        $methods = Payment::select([
            'id',
            'name',
            'payment',
            'icon',
            'handling_fee_fixed',
            'handling_fee_percent'
        ])
            ->where('enable', 1)
            ->orderBy('sort', 'ASC')
            ->get();

        return $this->success($methods);
    }

    public function cancel(Request $request)
    {
        if (empty($request->input('trade_no'))) {
            return $this->fail([422, __('Invalid parameter')]);
        }
        $order = Order::where('trade_no', $request->input('trade_no'))
            ->where('user_id', $request->user()->id)
            ->first();
        if (!$order) {
            return $this->fail([400, __('Order does not exist')]);
        }
        $orderService = new OrderService($order);
        if (!$orderService->cancel()) {
            return $this->fail([400, __('You can only cancel pending orders')]);
        }
        return $this->success(true);
    }
}
